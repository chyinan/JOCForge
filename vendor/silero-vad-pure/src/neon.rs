//! AArch64 NEON kernels: vectorized exp/sigmoid/tanh approximations and the
//! fused LSTM gate computation. Only compiled on `aarch64`; every other
//! target keeps the scalar/BLAS paths.
//!
//! Accuracy contract: sigmoid/tanh approximations stay within 1e-6 absolute
//! error of the f64 libm reference on [-30, 30] (verified by tests below);
//! inputs outside that range saturate naturally.

#![cfg(target_arch = "aarch64")]

use core::arch::aarch64::*;

/// Vectorized e^x, Cephes-style range reduction + polynomial.
/// Relative error ~1e-7; input clamped to [-87, 87] so 2^k never overflows.
///
/// # Safety
/// Requires NEON (baseline on aarch64).
// The Cephes constants below are written with more digits than f32
// round-trips; they round to the intended values, so keep the canonical
// spelling.
#[allow(clippy::excessive_precision)]
#[inline]
unsafe fn exp4(x: float32x4_t) -> float32x4_t {
    unsafe {
        let x = vminq_f32(vmaxq_f32(x, vdupq_n_f32(-87.0)), vdupq_n_f32(87.0));
        // k = round(x / ln2), g = x - k*ln2 (hi/lo split for exactness)
        let k = vrndnq_f32(vmulq_f32(x, vdupq_n_f32(std::f32::consts::LOG2_E)));
        let g = vfmsq_f32(x, k, vdupq_n_f32(0.693_359_375));
        let g = vfmsq_f32(g, k, vdupq_n_f32(-2.121_944_40e-4));
        // Cephes expf polynomial: e^g ~= P(g)*g*g + g + 1
        let mut p = vdupq_n_f32(1.987_569_150_0e-4);
        p = vfmaq_f32(vdupq_n_f32(1.398_199_950_7e-3), p, g);
        p = vfmaq_f32(vdupq_n_f32(8.333_451_907_3e-3), p, g);
        p = vfmaq_f32(vdupq_n_f32(4.166_579_589_4e-2), p, g);
        p = vfmaq_f32(vdupq_n_f32(1.666_666_545_9e-1), p, g);
        p = vfmaq_f32(vdupq_n_f32(5.000_000_120_1e-1), p, g);
        let poly = vfmaq_f32(vaddq_f32(g, vdupq_n_f32(1.0)), p, vmulq_f32(g, g));
        // 2^k via exponent-bit injection
        let ki = vcvtq_s32_f32(k);
        let scale = vreinterpretq_f32_s32(vaddq_s32(vshlq_n_s32(ki, 23), vdupq_n_s32(127 << 23)));
        vmulq_f32(poly, scale)
    }
}

/// Vectorized 1 / (1 + e^-x).
///
/// # Safety
/// Requires NEON (baseline on aarch64).
#[inline]
pub(crate) unsafe fn sigmoid4(x: float32x4_t) -> float32x4_t {
    unsafe {
        let e = exp4(vnegq_f32(x));
        vdivq_f32(vdupq_n_f32(1.0), vaddq_f32(e, vdupq_n_f32(1.0)))
    }
}

/// Vectorized tanh(x) = 1 - 2 / (e^{2x} + 1).
///
/// # Safety
/// Requires NEON (baseline on aarch64).
#[inline]
pub(crate) unsafe fn tanh4(x: float32x4_t) -> float32x4_t {
    unsafe {
        let e = exp4(vaddq_f32(x, x));
        let two_over = vdivq_f32(vdupq_n_f32(2.0), vaddq_f32(e, vdupq_n_f32(1.0)));
        vsubq_f32(vdupq_n_f32(1.0), two_over)
    }
}

/// NEON GEMV for the LSTM gate projection: gates = w @ xh, `w`: [512, 256]
/// rows of [w_ih | w_hh], `xh`: [256] = [x | h], `gates`: [512].
///
/// The [512, 256] row-major weight layout is already streaming-friendly for
/// the 8-row-block kernel (eight contiguous 1 KiB rows per block), so no
/// repacking is done at load time.
///
/// Note: with the Accelerate backend on Apple silicon, cblas_sgemm runs this
/// GEMV on the AMX coprocessor (~1.2 µs vs ~5 µs for this NEON kernel —
/// NEON is streaming-load bound on the 512 KiB weight matrix) and is used
/// instead; this kernel wins on targets without AMX (measured 6x faster
/// than the `matrixmultiply` fallback).
///
/// # Safety
/// `w` must hold 512*256 floats, `xh` 256.
#[cfg(any(test, not(all(feature = "accelerate", target_os = "macos"))))]
pub(crate) unsafe fn gemv_512x256(w: &[f32], xh: &[f32; 256], gates: &mut [f32; 512]) {
    unsafe {
        const ROWS: usize = 512;
        const COLS: usize = 256;
        debug_assert_eq!(w.len(), ROWS * COLS);

        // 8 rows per block share each xh load; 8 independent FMA chains
        // hide the 4-cycle FMA latency (4 chains stalled at ~1 FMA/cycle,
        // which made a 4-row variant slower than the BLAS GEMV).
        for rb in 0..ROWS / 8 {
            let r = w.as_ptr().add(rb * 8 * COLS);
            let mut acc0 = vdupq_n_f32(0.0);
            let mut acc1 = vdupq_n_f32(0.0);
            let mut acc2 = vdupq_n_f32(0.0);
            let mut acc3 = vdupq_n_f32(0.0);
            let mut acc4 = vdupq_n_f32(0.0);
            let mut acc5 = vdupq_n_f32(0.0);
            let mut acc6 = vdupq_n_f32(0.0);
            let mut acc7 = vdupq_n_f32(0.0);
            let mut xp = xh.as_ptr();
            let mut w0 = r;
            let mut w1 = r.add(COLS);
            let mut w2 = r.add(2 * COLS);
            let mut w3 = r.add(3 * COLS);
            let mut w4 = r.add(4 * COLS);
            let mut w5 = r.add(5 * COLS);
            let mut w6 = r.add(6 * COLS);
            let mut w7 = r.add(7 * COLS);
            for _ in 0..COLS / 4 {
                let xv = vld1q_f32(xp);
                acc0 = vfmaq_f32(acc0, vld1q_f32(w0), xv);
                acc1 = vfmaq_f32(acc1, vld1q_f32(w1), xv);
                acc2 = vfmaq_f32(acc2, vld1q_f32(w2), xv);
                acc3 = vfmaq_f32(acc3, vld1q_f32(w3), xv);
                acc4 = vfmaq_f32(acc4, vld1q_f32(w4), xv);
                acc5 = vfmaq_f32(acc5, vld1q_f32(w5), xv);
                acc6 = vfmaq_f32(acc6, vld1q_f32(w6), xv);
                acc7 = vfmaq_f32(acc7, vld1q_f32(w7), xv);
                xp = xp.add(4);
                w0 = w0.add(4);
                w1 = w1.add(4);
                w2 = w2.add(4);
                w3 = w3.add(4);
                w4 = w4.add(4);
                w5 = w5.add(4);
                w6 = w6.add(4);
                w7 = w7.add(4);
            }
            let gp = gates.as_mut_ptr().add(rb * 8);
            *gp = vaddvq_f32(acc0);
            *gp.add(1) = vaddvq_f32(acc1);
            *gp.add(2) = vaddvq_f32(acc2);
            *gp.add(3) = vaddvq_f32(acc3);
            *gp.add(4) = vaddvq_f32(acc4);
            *gp.add(5) = vaddvq_f32(acc5);
            *gp.add(6) = vaddvq_f32(acc6);
            *gp.add(7) = vaddvq_f32(acc7);
        }
    }
}

/// Vectorized LSTM gate update (gate order i, f, g, o), bias add fused:
/// c = sigmoid(f+b_f)*c + sigmoid(i+b_i)*tanh(g+b_g), h = sigmoid(o+b_o)*tanh(c).
///
/// # Safety
/// `gates`/`b` must hold 512 floats.
pub(crate) unsafe fn gate_update(
    gates: &[f32; 512],
    b: &[f32],
    h: &mut [f32; 128],
    c: &mut [f32; 128],
) {
    unsafe {
        debug_assert_eq!(b.len(), 512);
        for j in (0..128).step_by(4) {
            let gi = vaddq_f32(
                vld1q_f32(gates.as_ptr().add(j)),
                vld1q_f32(b.as_ptr().add(j)),
            );
            let gf = vaddq_f32(
                vld1q_f32(gates.as_ptr().add(128 + j)),
                vld1q_f32(b.as_ptr().add(128 + j)),
            );
            let gg = vaddq_f32(
                vld1q_f32(gates.as_ptr().add(256 + j)),
                vld1q_f32(b.as_ptr().add(256 + j)),
            );
            let go = vaddq_f32(
                vld1q_f32(gates.as_ptr().add(384 + j)),
                vld1q_f32(b.as_ptr().add(384 + j)),
            );
            let i = sigmoid4(gi);
            let f = sigmoid4(gf);
            let g = tanh4(gg);
            let o = sigmoid4(go);
            let c_new = vfmaq_f32(vmulq_f32(i, g), f, vld1q_f32(c.as_ptr().add(j)));
            vst1q_f32(c.as_mut_ptr().add(j), c_new);
            vst1q_f32(h.as_mut_ptr().add(j), vmulq_f32(o, tanh4(c_new)));
        }
    }
}

/// Fused LSTM step: [`gemv_512x256`] + [`gate_update`]. Used on aarch64
/// targets without the Accelerate backend.
///
/// # Safety
/// `w` must hold 512*256 floats, `b` 512, `xh` 256.
#[cfg(any(test, not(all(feature = "accelerate", target_os = "macos"))))]
pub(crate) unsafe fn lstm_gates(
    w: &[f32],
    b: &[f32],
    xh: &[f32; 256],
    h: &mut [f32; 128],
    c: &mut [f32; 128],
) {
    unsafe {
        let mut gates = [0.0f32; 512];
        gemv_512x256(w, xh, &mut gates);
        gate_update(&gates, b, h, c);
    }
}

/// t_out = 3): out[i, t] = |dot(basis[i], window_t) +
/// i*dot(basis[n_bins+i], window_t)| where window_t = xp[t*hop..+KERNEL].
/// Skips the im2col frames copy and the sgemm call entirely.
///
/// Const-generic over KERNEL: monomorphizing the two model geometries
/// (256/128) lets LLVM unroll the k-loop and constant-fold the hop strides
/// — worth ~1.8x over the runtime-shaped version in measurement.
///
/// Structure notes, from measurement: 2 bins x 3 time steps = 12
/// accumulators; per 4-wide k-chunk the 4 weight loads (2 bins x re/im) are
/// shared across all 3 window loads, giving 12 independent FMA chains at a
/// 7:12 load:FMA ratio.
///
/// # Safety
/// `basis` must hold 2*n_bins rows of KERNEL floats; `xp` must hold at
/// least KERNEL + 2*(KERNEL/2) floats; `out` n_bins*3.
pub(crate) unsafe fn stft_mag_neon<const KERNEL: usize>(
    xp: &[f32],
    basis: &[f32],
    n_bins: usize,
    out: &mut [f32],
) {
    unsafe {
        let hop = KERNEL / 2;
        debug_assert_eq!(basis.len(), 2 * n_bins * KERNEL);
        debug_assert_eq!(out.len(), n_bins * 3);
        debug_assert!(KERNEL.is_multiple_of(4));
        let kchunks = KERNEL / 4;

        let mut i = 0;
        while i + 2 <= n_bins {
            let re0 = basis.as_ptr().add(i * KERNEL);
            let im0 = basis.as_ptr().add((n_bins + i) * KERNEL);
            let re1 = re0.add(KERNEL);
            let im1 = im0.add(KERNEL);
            let zero = vdupq_n_f32(0.0);
            let (mut a00, mut a01, mut a02, mut a03) = (zero, zero, zero, zero);
            let (mut a10, mut a11, mut a12, mut a13) = (zero, zero, zero, zero);
            let (mut a20, mut a21, mut a22, mut a23) = (zero, zero, zero, zero);
            let mut x = xp.as_ptr();
            let (mut pr0, mut pi0, mut pr1, mut pi1) = (re0, im0, re1, im1);
            // 2 chunks per iteration: halves loop overhead and gives the
            // scheduler two independent load/FMA groups (KERNEL/4 is even
            // for both model geometries, 64 and 32).
            for _ in 0..kchunks / 2 {
                for off in [0usize, 4] {
                    let w0 = vld1q_f32(pr0.add(off));
                    let w1 = vld1q_f32(pi0.add(off));
                    let w2 = vld1q_f32(pr1.add(off));
                    let w3 = vld1q_f32(pi1.add(off));
                    let x0 = vld1q_f32(x.add(off));
                    a00 = vfmaq_f32(a00, w0, x0);
                    a01 = vfmaq_f32(a01, w1, x0);
                    a02 = vfmaq_f32(a02, w2, x0);
                    a03 = vfmaq_f32(a03, w3, x0);
                    let x1 = vld1q_f32(x.add(off + hop));
                    a10 = vfmaq_f32(a10, w0, x1);
                    a11 = vfmaq_f32(a11, w1, x1);
                    a12 = vfmaq_f32(a12, w2, x1);
                    a13 = vfmaq_f32(a13, w3, x1);
                    let x2 = vld1q_f32(x.add(off + 2 * hop));
                    a20 = vfmaq_f32(a20, w0, x2);
                    a21 = vfmaq_f32(a21, w1, x2);
                    a22 = vfmaq_f32(a22, w2, x2);
                    a23 = vfmaq_f32(a23, w3, x2);
                }
                x = x.add(8);
                pr0 = pr0.add(8);
                pi0 = pi0.add(8);
                pr1 = pr1.add(8);
                pi1 = pi1.add(8);
            }
            let dots = [
                vaddvq_f32(a00),
                vaddvq_f32(a01),
                vaddvq_f32(a02),
                vaddvq_f32(a03),
                vaddvq_f32(a10),
                vaddvq_f32(a11),
                vaddvq_f32(a12),
                vaddvq_f32(a13),
                vaddvq_f32(a20),
                vaddvq_f32(a21),
                vaddvq_f32(a22),
                vaddvq_f32(a23),
            ];
            for t in 0..3 {
                let (r0, m0) = (dots[t * 4], dots[t * 4 + 1]);
                let (r1, m1) = (dots[t * 4 + 2], dots[t * 4 + 3]);
                out[i * 3 + t] = r0.mul_add(r0, m0 * m0).sqrt();
                out[(i + 1) * 3 + t] = r1.mul_add(r1, m1 * m1).sqrt();
            }
            i += 2;
        }
        // Remainder bin (n_bins odd): same 3-t blocking, 2 x re/im chains.
        if i < n_bins {
            let re0 = basis.as_ptr().add(i * KERNEL);
            let im0 = basis.as_ptr().add((n_bins + i) * KERNEL);
            let zero = vdupq_n_f32(0.0);
            let (mut ar0, mut ai0, mut ar1, mut ai1, mut ar2, mut ai2) =
                (zero, zero, zero, zero, zero, zero);
            let mut x = xp.as_ptr();
            let (mut pr, mut pi) = (re0, im0);
            for _ in 0..kchunks {
                let wr = vld1q_f32(pr);
                let wi = vld1q_f32(pi);
                ar0 = vfmaq_f32(ar0, wr, vld1q_f32(x));
                ai0 = vfmaq_f32(ai0, wi, vld1q_f32(x));
                ar1 = vfmaq_f32(ar1, wr, vld1q_f32(x.add(hop)));
                ai1 = vfmaq_f32(ai1, wi, vld1q_f32(x.add(hop)));
                ar2 = vfmaq_f32(ar2, wr, vld1q_f32(x.add(2 * hop)));
                ai2 = vfmaq_f32(ai2, wi, vld1q_f32(x.add(2 * hop)));
                x = x.add(4);
                pr = pr.add(4);
                pi = pi.add(4);
            }
            for (t, (ar, ai)) in [(ar0, ai0), (ar1, ai1), (ar2, ai2)].into_iter().enumerate() {
                let r = vaddvq_f32(ar);
                let m = vaddvq_f32(ai);
                out[i * 3 + t] = r.mul_add(r, m * m).sqrt();
            }
        }
    }
}

/// Direct conv1d (k = 3) + bias + ReLU for tiny time spans: all output
/// positions t*stride must fit one 4-lane window, i.e.
/// (t_out-1)*stride <= 3. Accumulator lane j corresponds to input position
/// j, so output t lands in lane t*stride; garbage lanes are dropped in the
/// epilogue. Weights are the prepacked [cout, cin, 4] layout (4th lane 0),
/// applied with lane-indexed FMAs (no separate broadcast ops).
///
/// Structure notes, from measurement: 4 output channels per block share the
/// three shifted window loads, and each (oc, kk) pair gets its own
/// accumulator — a single acc per oc chains 3 FMAs (~12 cycles/channel
/// group) and doubles the kernel's runtime.
///
/// `xpad`: scratch, resized to cin * row_stride and filled with the input
/// rows plus `pad` zeros on each side, zero-padded to row_stride so the
/// 4-lane loads at offsets 0..k-1 never go out of bounds.
///
/// # Safety
/// `w_packed` must hold cout*cin*4 floats, `bias` cout, `input` cin*t_in.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn conv1d_relu_neon(
    input: &[f32],
    cin: usize,
    t_in: usize,
    w_packed: &[f32],
    bias: &[f32],
    cout: usize,
    k: usize,
    pad: usize,
    stride: usize,
    xpad: &mut Vec<f32>,
    out: &mut Vec<f32>,
) -> usize {
    unsafe {
        let t_out = (t_in + 2 * pad - k) / stride + 1;
        debug_assert_eq!(k, 3);
        debug_assert!((t_out - 1) * stride <= 3);
        debug_assert_eq!(w_packed.len(), cout * cin * 4);
        debug_assert_eq!(input.len(), cin * t_in);

        // 4-lane loads at offsets 0..k-1 touch up to (k-1)+3; keep rows
        // zero-padded to a 4-multiple past that.
        let row_stride = (t_in + 2 * pad).max(k + 3).next_multiple_of(4);
        xpad.clear();
        xpad.resize(cin * row_stride, 0.0);
        for c in 0..cin {
            xpad[c * row_stride + pad..c * row_stride + pad + t_in]
                .copy_from_slice(&input[c * t_in..(c + 1) * t_in]);
        }

        out.clear();
        out.resize(cout * t_out, 0.0);
        let zero = vdupq_n_f32(0.0);
        let mut oc = 0;
        while oc + 4 <= cout {
            let mut acc = [[zero; 3]; 4]; // [oc-block][kk]
            let w0 = w_packed.as_ptr().add(oc * cin * 4);
            let w1 = w0.add(cin * 4);
            let w2 = w1.add(cin * 4);
            let w3 = w2.add(cin * 4);
            for c in 0..cin {
                let row = xpad.as_ptr().add(c * row_stride);
                let u0 = vld1q_f32(row);
                let u1 = vld1q_f32(row.add(1));
                let u2 = vld1q_f32(row.add(2));
                let a = vld1q_f32(w0.add(c * 4));
                acc[0][0] = vfmaq_laneq_f32::<0>(acc[0][0], u0, a);
                acc[0][1] = vfmaq_laneq_f32::<1>(acc[0][1], u1, a);
                acc[0][2] = vfmaq_laneq_f32::<2>(acc[0][2], u2, a);
                let b = vld1q_f32(w1.add(c * 4));
                acc[1][0] = vfmaq_laneq_f32::<0>(acc[1][0], u0, b);
                acc[1][1] = vfmaq_laneq_f32::<1>(acc[1][1], u1, b);
                acc[1][2] = vfmaq_laneq_f32::<2>(acc[1][2], u2, b);
                let d = vld1q_f32(w2.add(c * 4));
                acc[2][0] = vfmaq_laneq_f32::<0>(acc[2][0], u0, d);
                acc[2][1] = vfmaq_laneq_f32::<1>(acc[2][1], u1, d);
                acc[2][2] = vfmaq_laneq_f32::<2>(acc[2][2], u2, d);
                let e = vld1q_f32(w3.add(c * 4));
                acc[3][0] = vfmaq_laneq_f32::<0>(acc[3][0], u0, e);
                acc[3][1] = vfmaq_laneq_f32::<1>(acc[3][1], u1, e);
                acc[3][2] = vfmaq_laneq_f32::<2>(acc[3][2], u2, e);
            }
            for (b, accb) in acc.iter().enumerate() {
                let v = vaddq_f32(vaddq_f32(accb[0], accb[1]), accb[2]);
                let v = vmaxq_f32(vaddq_f32(v, vdupq_n_f32(bias[oc + b])), zero);
                let mut tmp = [0.0f32; 4];
                vst1q_f32(tmp.as_mut_ptr(), v);
                for t in 0..t_out {
                    out[(oc + b) * t_out + t] = tmp[t * stride];
                }
            }
            oc += 4;
        }
        // Remainder output channels (< 4), same structure without blocking.
        while oc < cout {
            let mut acc = [zero; 3];
            let wp = w_packed.as_ptr().add(oc * cin * 4);
            for c in 0..cin {
                let row = xpad.as_ptr().add(c * row_stride);
                let a = vld1q_f32(wp.add(c * 4));
                acc[0] = vfmaq_laneq_f32::<0>(acc[0], vld1q_f32(row), a);
                acc[1] = vfmaq_laneq_f32::<1>(acc[1], vld1q_f32(row.add(1)), a);
                acc[2] = vfmaq_laneq_f32::<2>(acc[2], vld1q_f32(row.add(2)), a);
            }
            let v = vaddq_f32(vaddq_f32(acc[0], acc[1]), acc[2]);
            let v = vmaxq_f32(vaddq_f32(v, vdupq_n_f32(bias[oc])), zero);
            let mut tmp = [0.0f32; 4];
            vst1q_f32(tmp.as_mut_ptr(), v);
            for t in 0..t_out {
                out[oc * t_out + t] = tmp[t * stride];
            }
            oc += 1;
        }
        t_out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Evaluate a vectorized unary op lane-wise over scalar samples.
    fn map4(f: unsafe fn(float32x4_t) -> float32x4_t, xs: &[f32]) -> Vec<f32> {
        xs.chunks(4)
            .flat_map(|c| {
                let v = unsafe { f(vld1q_f32(c.as_ptr())) };
                let mut out = [0.0f32; 4];
                unsafe { vst1q_f32(out.as_mut_ptr(), v) };
                out
            })
            .collect()
    }

    fn samples() -> Vec<f32> {
        // Dense sweep plus saturation extremes.
        let mut v: Vec<f32> = (0..24000)
            .map(|i| -30.0 + 60.0 * i as f32 / 23999.0)
            .collect();
        v.extend_from_slice(&[-100.0, -87.5, -30.0, 0.0, 30.0, 87.5, 100.0]);
        let pad = (4 - v.len() % 4) % 4;
        v.extend(std::iter::repeat_n(0.5, pad));
        v
    }

    #[test]
    fn sigmoid_max_err_within_1e_6() {
        let xs = samples();
        let got = map4(sigmoid4, &xs);
        let mut max_err = 0.0f64;
        for (x, y) in xs.iter().zip(&got) {
            let reference = 1.0 / (1.0 + (-*x as f64).exp());
            max_err = max_err.max((*y as f64 - reference).abs());
        }
        assert!(max_err <= 1e-6, "sigmoid max abs err {max_err:e}");
        eprintln!("[neon] sigmoid max abs err on [-30,30]: {max_err:.3e}");
    }

    #[test]
    fn tanh_max_err_within_1e_6() {
        let xs = samples();
        let got = map4(tanh4, &xs);
        let mut max_err = 0.0f64;
        for (x, y) in xs.iter().zip(&got) {
            let reference = (*x as f64).tanh();
            max_err = max_err.max((*y as f64 - reference).abs());
        }
        assert!(max_err <= 1e-6, "tanh max abs err {max_err:e}");
        eprintln!("[neon] tanh max abs err on [-30,30]: {max_err:.3e}");
    }

    #[test]
    fn lstm_gates_matches_naive() {
        let mut seed = 99u32;
        let mut rand = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            ((seed >> 8) as f32 / (1 << 24) as f32 - 0.5) * 0.5
        };
        let w: Vec<f32> = (0..512 * 256).map(|_| rand()).collect();
        let b: Vec<f32> = (0..512).map(|_| rand()).collect();
        let xh: [f32; 256] = std::array::from_fn(|_| rand());
        let mut h = [0.0f32; 128];
        let mut c: [f32; 128] = std::array::from_fn(|_| rand());
        let c0 = c;
        unsafe { lstm_gates(&w, &b, &xh, &mut h, &mut c) };
        for j in 0..128 {
            let dot = |row: usize| {
                (0..256)
                    .map(|p| (w[row * 256 + p] as f64) * (xh[p] as f64))
                    .sum::<f64>()
                    + b[row] as f64
            };
            let (gi, gf, gg, go) = (dot(j), dot(128 + j), dot(256 + j), dot(384 + j));
            let sig = |v: f64| 1.0 / (1.0 + (-v).exp());
            let c_ref = sig(gf) * c0[j] as f64 + sig(gi) * gg.tanh();
            let h_ref = sig(go) * c_ref.tanh();
            assert!(
                (c[j] as f64 - c_ref).abs() <= 1e-4,
                "c[{j}]: {} vs {c_ref}",
                c[j]
            );
            assert!(
                (h[j] as f64 - h_ref).abs() <= 1e-4,
                "h[{j}]: {} vs {h_ref}",
                h[j]
            );
        }
    }
}
