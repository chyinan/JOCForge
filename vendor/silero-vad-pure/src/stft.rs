//! STFT magnitude via the model's fixed DFT basis (a Conv1d, k = kernel,
//! stride = kernel/2, no pad) on a right-only reflect-padded signal.
//!
//! Semantics verified against the ONNX graph: the model pads reflect on the
//! RIGHT side only (16k: 64 samples, 8k: 32), unlike PLAN.md §1.2's
//! two-sided pad.

use crate::math::sgemm;

/// Reusable buffers for [`stft_mag`], allocated once per `SileroVad`.
#[derive(Debug, Default)]
pub(crate) struct StftScratch {
    /// Right-only reflect-padded input, [chunk + pad].
    pub xp: Vec<f32>,
    /// im2col frames, [kernel, t_out] (non-model-geometry fallback path).
    pub frames: Vec<f32>,
    /// Basis @ frames result, [2*n_bins, t_out] (fallback path only).
    pub conv: Vec<f32>,
}

/// chunk: [chunk] samples. basis: [2*n_bins, kernel] row-major.
/// out: resized to [n_bins, t_out] row-major magnitudes.
/// Returns t_out.
pub(crate) fn stft_mag(
    chunk: &[f32],
    pad: usize,
    basis: &[f32],
    n_bins: usize,
    kernel: usize,
    scratch: &mut StftScratch,
    out: &mut Vec<f32>,
) -> usize {
    let n = chunk.len();
    assert!(n > pad + 1, "chunk too short for reflect pad");
    let hop = kernel / 2;
    let padded_len = n + pad;
    let t_out = (padded_len - kernel) / hop + 1;

    // right-only reflect pad (torch F.pad(mode='reflect') semantics: the edge
    // sample itself is not repeated): xp[n + i] = chunk[n - 2 - i].
    scratch.xp.clear();
    scratch.xp.extend_from_slice(chunk);
    for i in 0..pad {
        scratch.xp.push(chunk[n - 2 - i]);
    }
    let xp = &scratch.xp;

    #[cfg(target_arch = "aarch64")]
    {
        // Direct NEON kernel for the model's fixed geometry: no im2col
        // frames, no sgemm call; the xp window feeds the re/im dots
        // directly. Const-generic over the kernel so LLVM sees the trip
        // count and hop strides. Anything else falls through to im2col.
        let neon_t = match (kernel, t_out) {
            (256, 3) => Some(unsafe {
                out.clear();
                out.resize(n_bins * 3, 0.0);
                // SAFETY: basis has 2*n_bins rows of 256 floats; xp has
                // padded_len = 2*hop + kernel floats by construction.
                crate::neon::stft_mag_neon::<256>(xp, basis, n_bins, out);
                t_out
            }),
            (128, 3) => Some(unsafe {
                out.clear();
                out.resize(n_bins * 3, 0.0);
                // SAFETY: as above, with 128-float rows.
                crate::neon::stft_mag_neon::<128>(xp, basis, n_bins, out);
                t_out
            }),
            _ => None,
        };
        if let Some(t) = neon_t {
            return t;
        }
    }

    // frames: [kernel, t_out] column t = xp[t*hop .. t*hop + kernel]
    scratch.frames.clear();
    scratch.frames.resize(kernel * t_out, 0.0);
    for t in 0..t_out {
        for j in 0..kernel {
            scratch.frames[j * t_out + t] = xp[t * hop + j];
        }
    }

    // conv: [2*n_bins, t_out] = basis @ frames
    scratch.conv.clear();
    scratch.conv.resize(2 * n_bins * t_out, 0.0);
    sgemm(
        2 * n_bins,
        t_out,
        kernel,
        basis,
        &scratch.frames,
        &mut scratch.conv,
    );

    out.resize(n_bins * t_out, 0.0);
    for i in 0..n_bins {
        for t in 0..t_out {
            let re = scratch.conv[i * t_out + t];
            let im = scratch.conv[(n_bins + i) * t_out + t];
            out[i * t_out + t] = (re * re + im * im).sqrt();
        }
    }
    t_out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Naive DFT-basis convolution reference.
    fn naive(chunk: &[f32], pad: usize, basis: &[f32], n_bins: usize, kernel: usize) -> Vec<f32> {
        let n = chunk.len();
        let hop = kernel / 2;
        let padded_len = n + pad;
        let t_out = (padded_len - kernel) / hop + 1;
        let mut xp = chunk.to_vec();
        for i in 0..pad {
            xp.push(chunk[n - 2 - i]);
        }
        let mut out = vec![0.0f32; n_bins * t_out];
        for t in 0..t_out {
            for i in 0..n_bins {
                let mut re = 0.0f32;
                let mut im = 0.0f32;
                for j in 0..kernel {
                    re += basis[i * kernel + j] * xp[t * hop + j];
                    im += basis[(n_bins + i) * kernel + j] * xp[t * hop + j];
                }
                out[i * t_out + t] = (re * re + im * im).sqrt();
            }
        }
        out
    }

    #[test]
    fn matches_naive() {
        let mut seed = 42u32;
        let mut rand = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / (1 << 24) as f32 - 0.5
        };
        for (n, pad, n_bins, kernel) in [(512, 64, 129, 256), (256, 32, 65, 128)] {
            let chunk: Vec<f32> = (0..n).map(|_| rand()).collect();
            let basis: Vec<f32> = (0..2 * n_bins * kernel).map(|_| rand()).collect();
            let mut scratch = StftScratch::default();
            let mut out = Vec::new();
            stft_mag(&chunk, pad, &basis, n_bins, kernel, &mut scratch, &mut out);
            let expect = naive(&chunk, pad, &basis, n_bins, kernel);
            assert_eq!(out.len(), expect.len());
            for (x, y) in out.iter().zip(&expect) {
                assert!((x - y).abs() <= 1e-3, "{x} vs {y}");
            }
        }
    }
}
