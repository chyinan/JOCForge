//! sgemm abstraction: Accelerate.framework on macOS (default), matrixmultiply
//! fallback everywhere else.

/// C = A @ B, row-major, beta = 0 (overwrite).
/// a: [m, k], b: [k, n], c: [m, n].
pub(crate) fn sgemm(m: usize, n: usize, k: usize, a: &[f32], b: &[f32], c: &mut [f32]) {
    debug_assert_eq!(a.len(), m * k);
    debug_assert_eq!(b.len(), k * n);
    debug_assert_eq!(c.len(), m * n);
    backend::sgemm(m, n, k, a, b, c)
}

#[cfg(all(feature = "accelerate", target_os = "macos"))]
mod backend {
    // Pull in accelerate-src so its `rustc-link-lib=framework=Accelerate`
    // directive is applied (the crate itself is link-only, no API).
    use accelerate_src as _;

    const CBLAS_ROW_MAJOR: i32 = 101;
    const CBLAS_NO_TRANS: i32 = 111;

    unsafe extern "C" {
        #[allow(clippy::too_many_arguments)]
        fn cblas_sgemm(
            order: i32,
            trans_a: i32,
            trans_b: i32,
            m: i32,
            n: i32,
            k: i32,
            alpha: f32,
            a: *const f32,
            lda: i32,
            b: *const f32,
            ldb: i32,
            beta: f32,
            c: *mut f32,
            ldc: i32,
        );
    }

    pub(super) fn sgemm(m: usize, n: usize, k: usize, a: &[f32], b: &[f32], c: &mut [f32]) {
        // SAFETY: a/b/c are validated by the caller's debug_asserts to have
        // the exact extents passed here; cblas_sgemm with RowMajor/NoTrans
        // only reads m*k / k*n and writes m*n elements respectively.
        unsafe {
            cblas_sgemm(
                CBLAS_ROW_MAJOR,
                CBLAS_NO_TRANS,
                CBLAS_NO_TRANS,
                m as i32,
                n as i32,
                k as i32,
                1.0,
                a.as_ptr(),
                k as i32,
                b.as_ptr(),
                n as i32,
                0.0,
                c.as_mut_ptr(),
                n as i32,
            );
        }
    }
}

#[cfg(not(all(feature = "accelerate", target_os = "macos")))]
mod backend {
    pub(super) fn sgemm(m: usize, n: usize, k: usize, a: &[f32], b: &[f32], c: &mut [f32]) {
        // SAFETY: extents validated by the caller; strides describe row-major
        // layouts with unit column stride. beta = 0.0 overwrites c.
        unsafe {
            matrixmultiply::sgemm(
                m,
                k,
                n,
                1.0,
                a.as_ptr(),
                k as isize,
                1,
                b.as_ptr(),
                n as isize,
                1,
                0.0,
                c.as_mut_ptr(),
                n as isize,
                1,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn naive(m: usize, n: usize, k: usize, a: &[f32], b: &[f32], c: &mut [f32]) {
        for i in 0..m {
            for j in 0..n {
                let mut acc = 0.0f32;
                for p in 0..k {
                    acc += a[i * k + p] * b[p * n + j];
                }
                c[i * n + j] = acc;
            }
        }
    }

    #[test]
    fn sgemm_matches_naive() {
        // deterministic pseudo-random inputs
        let mut seed = 0x12345678u32;
        let mut rand = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / (1 << 24) as f32 - 0.5
        };
        for (m, n, k) in [
            (1, 1, 1),
            (3, 5, 7),
            (128, 3, 387),
            (512, 1, 256),
            (64, 2, 384),
        ] {
            let a: Vec<f32> = (0..m * k).map(|_| rand()).collect();
            let b: Vec<f32> = (0..k * n).map(|_| rand()).collect();
            let mut c = vec![0.0f32; m * n];
            let mut c_ref = vec![0.0f32; m * n];
            sgemm(m, n, k, &a, &b, &mut c);
            naive(m, n, k, &a, &b, &mut c_ref);
            for (x, y) in c.iter().zip(&c_ref) {
                assert!((x - y).abs() <= 1e-4, "m={m} n={n} k={k}: {x} vs {y}");
            }
        }
    }
}
