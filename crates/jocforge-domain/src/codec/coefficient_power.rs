// pattern: Functional Core
//! Fixed-profile coefficient-pair to POWER-row calculation.
//!
//! The caller must provide 253 finite real and imaginary F32 values per row,
//! each with magnitude in `[2^-20, 2]` or equal to zero. This module does not
//! generate coefficients, choose a mode, set the floating-point environment,
//! validate caller buffer shapes, or define behavior outside that profile.

/// Number of coefficient bins written by the selected producer.
pub const POWER_ROW_BINS: usize = 253;

/// One caller-provided coefficient plane row.
pub type CoefficientRow = [f32; POWER_ROW_BINS];

/// One output row containing binary32 POWER bit patterns.
pub type PowerRow = [u32; POWER_ROW_BINS];

/// Computes one POWER row from corresponding real and imaginary coefficients.
///
/// Each square and sum is a separate F32 operation. The returned row contains
/// the exact U32 bit pattern of each capped POWER value.
pub fn coefficient_pair_to_power_row(
    real: &CoefficientRow,
    imaginary: &CoefficientRow,
) -> PowerRow {
    let mut power = [0; POWER_ROW_BINS];

    for bin in 0..POWER_ROW_BINS {
        let real_square: f32 = real[bin] * real[bin];
        let imaginary_square: f32 = imaginary[bin] * imaginary[bin];
        let energy: f32 = real_square + imaginary_square;
        let capped_power = energy.min(1.0_f32);
        power[bin] = capped_power.to_bits();
    }

    power
}

/// Computes the selected single-block, single-channel row and clears its bias.
pub fn produce_single_power_row(
    real: &CoefficientRow,
    imaginary: &CoefficientRow,
    bias: &mut i16,
    power: &mut PowerRow,
) {
    *bias = 0;
    *power = coefficient_pair_to_power_row(real, imaginary);
}

/// Computes the selected two-block, two-channel wrapper.
///
/// Coefficients are indexed `[block][channel][bin]`; output and bias are
/// indexed `[channel][block][bin]` and `[channel][block]`. Processing order is
/// ascending block, then ascending channel.
pub fn produce_two_by_two_power_rows(
    real: &[[CoefficientRow; 2]; 2],
    imaginary: &[[CoefficientRow; 2]; 2],
    bias: &mut [[i16; 2]; 2],
    power: &mut [[PowerRow; 2]; 2],
) {
    for block in 0..2 {
        for channel in 0..2 {
            bias[channel][block] = 0;
            power[channel][block] =
                coefficient_pair_to_power_row(&real[block][channel], &imaginary[block][channel]);
        }
    }
}
