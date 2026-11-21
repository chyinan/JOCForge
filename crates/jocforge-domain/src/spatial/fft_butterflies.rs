// pattern: Functional Core
//! Radix-two butterflies over interleaved real/imaginary binary32 pairs.

pub(super) fn apply_stage(samples: &mut [f32], roots: &[[u32; 2]]) {
    if roots.len() >= 4 && roots.len() % 4 == 0 {
        four_pair_stage(samples, roots);
        return;
    }
    scalar_stage(samples, roots);
}

fn scalar_stage(samples: &mut [f32], roots: &[[u32; 2]]) {
    let half = roots.len() * 2;
    for group in samples.chunks_exact_mut(half * 2) {
        let (upper, lower) = group.split_at_mut(half);
        for ((left, right), &[c, d]) in upper
            .chunks_exact_mut(2)
            .zip(lower.chunks_exact_mut(2))
            .zip(roots)
        {
            let (ar, ai) = (left[0], left[1]);
            let (br, bi) = (right[0], right[1]);
            let (c, d) = (f32::from_bits(c), f32::from_bits(d));
            let tr = c * br - d * bi;
            let ti = c * bi + d * br;
            left[0] = ar + tr;
            left[1] = ai + ti;
            right[0] = ar - tr;
            right[1] = ai - ti;
        }
    }
}

/// Fixed-size groups let LLVM vectorize across complex bins. The dispatcher selects
/// the AVX2 specialization where available; other processors use their baseline ISA.
#[multiversion::multiversion(targets("x86_64+avx2"))]
fn four_pair_stage(samples: &mut [f32], roots: &[[u32; 2]]) {
    let half = roots.len() * 2;
    for group in samples.chunks_exact_mut(half * 2) {
        let (upper, lower) = group.split_at_mut(half);
        for ((upper, lower), twiddles) in upper
            .chunks_exact_mut(8)
            .zip(lower.chunks_exact_mut(8))
            .zip(roots.chunks_exact(4))
        {
            let upper: &mut [f32; 8] = upper.try_into().expect("four complex upper values");
            let lower: &mut [f32; 8] = lower.try_into().expect("four complex lower values");
            let twiddles: &[[u32; 2]; 4] = twiddles.try_into().expect("four twiddle pairs");
            for (k, &[real, imag]) in twiddles.iter().enumerate() {
                let (c, d) = (f32::from_bits(real), f32::from_bits(imag));
                let i = 2 * k;
                let tr = c * lower[i] - d * lower[i + 1];
                let ti = c * lower[i + 1] + d * lower[i];
                lower[i] = upper[i] - tr;
                lower[i + 1] = upper[i + 1] - ti;
                upper[i] += tr;
                upper[i + 1] += ti;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn dispatched_butterflies_match_scalar_signed_zeros_and_nonzero_words() {
        for half in [1usize, 2, 4, 8, 16, 32, 256, 2048] {
            let roots: Vec<_> = (0..half)
                .map(|i| [(i as f32).cos().to_bits(), (i as f32).sin().to_bits()])
                .collect();
            let mut actual: Vec<_> = (0..half * 12)
                .map(|i| {
                    if i % 11 == 0 {
                        -0.0
                    } else if i % 17 == 0 {
                        f32::from_bits(1)
                    } else {
                        (i as f32).sin()
                    }
                })
                .collect();
            let mut expected = actual.clone();
            super::scalar_stage(&mut expected, &roots);
            super::apply_stage(&mut actual, &roots);
            assert!(
                actual
                    .iter()
                    .zip(expected)
                    .all(|(a, b)| a.to_bits() == b.to_bits())
            );
        }
    }
}
