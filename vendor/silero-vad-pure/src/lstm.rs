//! Single-layer LSTM cell, hidden 128, PyTorch gate order i, f, g, o.

#[cfg(any(
    not(target_arch = "aarch64"),
    all(feature = "accelerate", target_os = "macos")
))]
use crate::math::sgemm;

pub(crate) const HIDDEN: usize = 128;
const GATES: usize = 4 * HIDDEN; // 512
const INPUT: usize = 2 * HIDDEN; // [x | h]

#[derive(Debug, Clone)]
pub(crate) struct LstmState {
    pub h: [f32; HIDDEN],
    pub c: [f32; HIDDEN],
}

impl LstmState {
    pub fn new() -> Self {
        Self {
            h: [0.0; HIDDEN],
            c: [0.0; HIDDEN],
        }
    }

    pub fn reset(&mut self) {
        self.h = [0.0; HIDDEN];
        self.c = [0.0; HIDDEN];
    }

    /// One time step. `w`: [512, 256] = [w_ih | w_hh] rows; `b`: [512]
    /// (b_ih + b_hh pre-added); `x`: [128].
    pub fn step(&mut self, w: &[f32], b: &[f32], x: &[f32; HIDDEN]) {
        debug_assert_eq!(w.len(), GATES * INPUT);
        debug_assert_eq!(b.len(), GATES);

        let mut xh = [0.0f32; INPUT];
        xh[..HIDDEN].copy_from_slice(x);
        xh[HIDDEN..].copy_from_slice(&self.h);

        // aarch64 + Accelerate: cblas GEMV runs on the AMX coprocessor
        // (~4x faster than our NEON GEMV, which is streaming-load bound on
        // the 512 KiB weight matrix); NEON only for the transcendentals.
        // The three cfg blocks are mutually exclusive and exhaustive.
        #[cfg(all(target_arch = "aarch64", feature = "accelerate", target_os = "macos"))]
        {
            let mut gates = [0.0f32; GATES];
            sgemm(GATES, 1, INPUT, w, &xh, &mut gates);
            // SAFETY: gates/b extents checked above.
            unsafe {
                crate::neon::gate_update(&gates, b, &mut self.h, &mut self.c);
            }
        }

        // aarch64 without Accelerate: NEON GEMV beats the matrixmultiply
        // fallback GEMV by ~6x.
        #[cfg(all(
            target_arch = "aarch64",
            not(all(feature = "accelerate", target_os = "macos"))
        ))]
        {
            // SAFETY: w/b extents checked above; xh is exactly INPUT floats.
            unsafe {
                crate::neon::lstm_gates(w, b, &xh, &mut self.h, &mut self.c);
            }
        }

        #[cfg(not(target_arch = "aarch64"))]
        {
            // gates = w @ xh + b  (sgemm as [512,256] @ [256,1])
            let mut gates = [0.0f32; GATES];
            sgemm(GATES, 1, INPUT, w, &xh, &mut gates);

            for j in 0..HIDDEN {
                let i = sigmoid(gates[j] + b[j]);
                let f = sigmoid(gates[HIDDEN + j] + b[HIDDEN + j]);
                let g = (gates[2 * HIDDEN + j] + b[2 * HIDDEN + j]).tanh();
                let o = sigmoid(gates[3 * HIDDEN + j] + b[3 * HIDDEN + j]);
                let c = f * self.c[j] + i * g;
                self.c[j] = c;
                self.h[j] = o * c.tanh();
            }
        }
    }
}

#[cfg(any(test, not(target_arch = "aarch64")))]
fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_scalar_reference() {
        let mut seed = 7u32;
        let mut rand = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            ((seed >> 8) as f32 / (1 << 24) as f32 - 0.5) * 0.5
        };
        let w: Vec<f32> = (0..GATES * INPUT).map(|_| rand()).collect();
        let b: Vec<f32> = (0..GATES).map(|_| rand()).collect();
        let x: [f32; HIDDEN] = std::array::from_fn(|_| rand());
        let mut st = LstmState {
            h: std::array::from_fn(|_| rand()),
            c: std::array::from_fn(|_| rand()),
        };
        let (h0, c0) = (st.h, st.c);
        st.step(&w, &b, &x);
        // scalar reference
        for j in 0..HIDDEN {
            let mut gi = b[j];
            let mut gf = b[HIDDEN + j];
            let mut gg = b[2 * HIDDEN + j];
            let mut go = b[3 * HIDDEN + j];
            for p in 0..HIDDEN {
                gi += w[j * INPUT + p] * x[p];
                gf += w[(HIDDEN + j) * INPUT + p] * x[p];
                gg += w[(2 * HIDDEN + j) * INPUT + p] * x[p];
                go += w[(3 * HIDDEN + j) * INPUT + p] * x[p];
            }
            for p in 0..HIDDEN {
                gi += w[j * INPUT + HIDDEN + p] * h0[p];
                gf += w[(HIDDEN + j) * INPUT + HIDDEN + p] * h0[p];
                gg += w[(2 * HIDDEN + j) * INPUT + HIDDEN + p] * h0[p];
                go += w[(3 * HIDDEN + j) * INPUT + HIDDEN + p] * h0[p];
            }
            let c = sigmoid(gf) * c0[j] + sigmoid(gi) * gg.tanh();
            let h = sigmoid(go) * c.tanh();
            assert!((st.c[j] - c).abs() <= 1e-4, "c[{j}]: {} vs {c}", st.c[j]);
            assert!((st.h[j] - h).abs() <= 1e-4, "h[{j}]: {} vs {h}", st.h[j]);
        }
    }
}
