// pattern: Functional Core
//! Selected 5.1 LFE coefficient-pair exponent route.
//!
//! The caller supplies two blocks of seven finite real/imaginary coefficient
//! pairs. This helper does not read full-band mode, format, or bias; it does
//! not produce coefficients or define parent lifecycle behavior.

/// Number of bins processed by the selected LFE coefficient route.
pub const LFE_COEFFICIENT_BINS: usize = 7;

/// Number of LFE coefficient blocks processed by the selected wrapper.
pub const LFE_COEFFICIENT_BLOCKS: usize = 2;

/// One real or imaginary coefficient block.
pub type LfeCoefficientBlock = [f32; LFE_COEFFICIENT_BINS];

/// One output block with a caller-owned trailing canary slot.
pub type LfeExponentOutputBlock = [i16; LFE_COEFFICIENT_BINS + 1];

/// Computes the selected LFE raw exponent directly from coefficient pairs.
///
/// The output's first seven values are overwritten for both blocks. The final
/// element in each output block is outside the written range and remains owned
/// by the caller. Inputs are bounded to the finite coefficient recipes in the
/// washed package; invalid pointers, lengths, non-finite values, and wider
/// domains are outside this API contract.
pub fn lfe_coefficients_to_exponents(
    real: &[LfeCoefficientBlock; LFE_COEFFICIENT_BLOCKS],
    imaginary: &[LfeCoefficientBlock; LFE_COEFFICIENT_BLOCKS],
    output: &mut [LfeExponentOutputBlock; LFE_COEFFICIENT_BLOCKS],
) {
    for block in 0..LFE_COEFFICIENT_BLOCKS {
        lfe_coefficient_pair_to_raw(&real[block], &imaginary[block], &mut output[block]);
    }
}

/// Computes one selected LFE raw-exponent row directly from seven coefficient pairs.
///
/// Writes only the first seven output entries and preserves the trailing caller canary.
/// This single-row boundary is used by the washed long-block LFE transform composition.
pub fn lfe_coefficient_pair_to_raw(
    real: &LfeCoefficientBlock,
    imaginary: &LfeCoefficientBlock,
    output: &mut LfeExponentOutputBlock,
) {
    const CAP: i16 = 48;

    for bin in 0..LFE_COEFFICIENT_BINS {
        let real_square: f32 = real[bin] * real[bin];
        let imaginary_square: f32 = imaginary[bin] * imaginary[bin];
        let energy: f32 = real_square + imaginary_square;
        let energy = energy.clamp(-1.0_f32, 1.0_f32);

        output[bin] = if energy == 0.0 {
            CAP / 2
        } else {
            let doubled_word = energy.to_bits().wrapping_mul(2);
            let source_exponent = if doubled_word == 0 {
                -127
            } else {
                i16::from(doubled_word.to_be_bytes()[0]) - 126
            };
            let difference = (-source_exponent).clamp(0, 60);
            difference.min(CAP) >> 1
        };
    }
}
