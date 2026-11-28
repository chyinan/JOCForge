//! Conv1d (kernel 3, pad 1, stride 1|2) + bias + ReLU, im2col + sgemm.

use crate::math::sgemm;

/// out[c_out, t] = relu(bias[c_out] + sum_{c_in,kk} w[c_out, c_in, kk] *
///                 input[c_in, t * stride + kk - pad])
///
/// `input`: [cin, t_in] row-major. `w`: [cout, cin, k] row-major (already the
/// im2col-compatible layout). `im2col`: scratch, resized to [cin*k, t_out].
/// `out`: resized to [cout, t_out] row-major.
/// Returns t_out = (t_in + 2*pad - k) / stride + 1 (floor, ONNX semantics).
#[allow(clippy::too_many_arguments)]
pub(crate) fn conv1d_relu(
    input: &[f32],
    cin: usize,
    t_in: usize,
    w: &[f32],
    bias: &[f32],
    cout: usize,
    k: usize,
    pad: usize,
    stride: usize,
    im2col: &mut Vec<f32>,
    out: &mut Vec<f32>,
) -> usize {
    assert!(pad + pad + t_in >= k, "input shorter than kernel");
    let t_out = (t_in + 2 * pad - k) / stride + 1;
    assert_eq!(input.len(), cin * t_in);
    assert_eq!(w.len(), cout * cin * k);
    assert_eq!(bias.len(), cout);

    // im2col: [cin*k, t_out], row-major; row (c*k + kk) picks input channel c
    // at offset t*stride + kk - pad (zero outside).
    im2col.clear();
    im2col.resize(cin * k * t_out, 0.0);
    for t in 0..t_out {
        let base = t * stride;
        for c in 0..cin {
            for kk in 0..k {
                let pos = base + kk;
                if pos >= pad && pos - pad < t_in {
                    im2col[(c * k + kk) * t_out + t] = input[c * t_in + pos - pad];
                }
            }
        }
    }

    out.resize(cout * t_out, 0.0);
    sgemm(cout, t_out, cin * k, w, im2col, out);
    for oc in 0..cout {
        let b = bias[oc];
        for v in &mut out[oc * t_out..(oc + 1) * t_out] {
            *v = (*v + b).max(0.0);
        }
    }
    t_out
}

/// Dispatching wrapper: on aarch64, shapes whose outputs fit one 4-lane
/// time window ((t_out-1)*stride <= 3, k <= 4) go to the direct NEON kernel
/// with prepacked weights — skipping im2col and, with the Accelerate
/// backend, its slow tiny-GEMM path (measured: n = 2..3 GEMMs run ~20x
/// slower per MAC than its n = 1 GEMV path). With Accelerate, t_out == 1
/// stays on cblas (its GEMV path is faster than the lane-wasting NEON
/// kernel); on the matrixmultiply fallback the NEON kernel wins there too.
#[allow(clippy::too_many_arguments)]
pub(crate) fn conv1d_relu_auto(
    input: &[f32],
    cin: usize,
    t_in: usize,
    w: &[f32],
    #[cfg(target_arch = "aarch64")] w_packed: &[f32],
    bias: &[f32],
    cout: usize,
    k: usize,
    pad: usize,
    stride: usize,
    #[cfg(target_arch = "aarch64")] xpad: &mut Vec<f32>,
    im2col: &mut Vec<f32>,
    out: &mut Vec<f32>,
) -> usize {
    #[cfg(target_arch = "aarch64")]
    {
        let t_out = (t_in + 2 * pad - k) / stride + 1;
        let accelerate = cfg!(all(feature = "accelerate", target_os = "macos"));
        if k == 3 && (t_out - 1) * stride <= 3 && (t_out >= 2 || !accelerate) {
            // SAFETY: w_packed/bias/input extents are checked by the kernel's
            // debug asserts; the dispatch condition is the kernel's contract.
            return unsafe {
                crate::neon::conv1d_relu_neon(
                    input, cin, t_in, w_packed, bias, cout, k, pad, stride, xpad, out,
                )
            };
        }
        let _ = (w_packed, xpad);
    }
    conv1d_relu(input, cin, t_in, w, bias, cout, k, pad, stride, im2col, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Naive direct convolution reference.
    #[allow(clippy::too_many_arguments)]
    fn naive(
        input: &[f32],
        cin: usize,
        t_in: usize,
        w: &[f32],
        bias: &[f32],
        cout: usize,
        k: usize,
        pad: usize,
        stride: usize,
    ) -> Vec<f32> {
        let t_out = (t_in + 2 * pad - k) / stride + 1;
        let mut out = vec![0.0f32; cout * t_out];
        for oc in 0..cout {
            for t in 0..t_out {
                let mut acc = bias[oc];
                for c in 0..cin {
                    for kk in 0..k {
                        let pos = t * stride + kk;
                        if pos >= pad && pos - pad < t_in {
                            acc += w[(oc * cin + c) * k + kk] * input[c * t_in + pos - pad];
                        }
                    }
                }
                out[oc * t_out + t] = acc.max(0.0);
            }
        }
        out
    }

    #[test]
    fn matches_naive() {
        let mut seed = 0xABCDEFu32;
        let mut rand = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / (1 << 24) as f32 - 0.5
        };
        for (cin, t_in, cout, k, pad, stride) in [
            (65, 3, 128, 3, 1, 1),
            (128, 3, 64, 3, 1, 2),
            (64, 2, 64, 3, 1, 2),
            (64, 1, 128, 3, 1, 1),
            (129, 3, 128, 3, 1, 1),
            (1, 640, 130, 128, 0, 64), // STFT-basis-like shape
        ] {
            let input: Vec<f32> = (0..cin * t_in).map(|_| rand()).collect();
            let w: Vec<f32> = (0..cout * cin * k).map(|_| rand()).collect();
            let bias: Vec<f32> = (0..cout).map(|_| rand()).collect();
            let mut im2col = Vec::new();
            let mut out = Vec::new();
            conv1d_relu(
                &input,
                cin,
                t_in,
                &w,
                &bias,
                cout,
                k,
                pad,
                stride,
                &mut im2col,
                &mut out,
            );
            let expect = naive(&input, cin, t_in, &w, &bias, cout, k, pad, stride);
            assert_eq!(out.len(), expect.len());
            for (x, y) in out.iter().zip(&expect) {
                assert!((x - y).abs() <= 1e-3, "{x} vs {y}");
            }
        }
    }

    /// The dispatching wrapper (NEON direct kernel on aarch64) against the
    /// naive reference, on the real encoder shapes.
    #[test]
    fn auto_matches_naive() {
        let mut seed = 0x12345u32;
        let mut rand = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / (1 << 24) as f32 - 0.5
        };
        for (cin, t_in, cout, k, pad, stride) in [
            (129, 3, 128, 3, 1, 1),    // enc0 16k
            (128, 3, 64, 3, 1, 2),     // enc1 16k
            (64, 2, 64, 3, 1, 2),      // enc2 16k
            (64, 1, 128, 3, 1, 1),     // enc3 16k
            (65, 3, 128, 3, 1, 1),     // enc0 8k
            (1, 640, 130, 128, 0, 64), // too wide for NEON: exercises fallback
        ] {
            let input: Vec<f32> = (0..cin * t_in).map(|_| rand()).collect();
            let w: Vec<f32> = (0..cout * cin * k).map(|_| rand()).collect();
            let bias: Vec<f32> = (0..cout).map(|_| rand()).collect();
            let mut im2col = Vec::new();
            let mut out = Vec::new();
            #[cfg(target_arch = "aarch64")]
            let mut xpad = Vec::new();
            #[cfg(target_arch = "aarch64")]
            let packed = if k <= 4 {
                crate::weights::pack_conv_w(&w, cout, cin, k)
            } else {
                Vec::new() // shape exercises the fallback; packed unused
            };
            conv1d_relu_auto(
                &input,
                cin,
                t_in,
                &w,
                #[cfg(target_arch = "aarch64")]
                &packed,
                &bias,
                cout,
                k,
                pad,
                stride,
                #[cfg(target_arch = "aarch64")]
                &mut xpad,
                &mut im2col,
                &mut out,
            );
            let expect = naive(&input, cin, t_in, &w, &bias, cout, k, pad, stride);
            assert_eq!(out.len(), expect.len());
            for (x, y) in out.iter().zip(&expect) {
                assert!((x - y).abs() <= 1e-3, "{x} vs {y}");
            }
        }
    }
}
