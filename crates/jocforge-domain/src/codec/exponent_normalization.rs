// pattern: Functional Core
//! Bounded selected-profile full-band exponent grouping and normalization.

use super::policy_parameters::PolicyParameters;
use crate::numeric::{FiniteF32, r32_mul};

/// Full-band exponent row length in the selected profile.
pub const FULLBAND_EXPONENT_BINS: usize = 217;
const GROUPED_EXPONENT_BINS: usize = FULLBAND_EXPONENT_BINS - 1;
const I16_MAX_EXPONENT: i16 = 24;
const I16_BASE_MAX: i16 = 15;

/// Contract failures for a bounded exponent-normalization call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExponentNormalizationError {
    ExponentInputError,
    ParameterUnavailable,
    NumericFailure,
}

impl ExponentNormalizationError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ExponentInputError => "EXPONENT_INPUT_ERROR",
            Self::ParameterUnavailable => "PARAMETER_UNAVAILABLE",
            Self::NumericFailure => "NUMERIC_FAILURE",
        }
    }
}

/// Result of normalizing one full-band I16 exponent row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedExponentRow {
    base_exponent: i16,
    expanded_exponents: [i16; FULLBAND_EXPONENT_BINS],
    grouped_sample_count_i16: i16,
    initial_sum_i32: Option<i32>,
    first_pass_square_error_i32: Option<i32>,
    error_shift_i32: Option<i32>,
    scaled_count_f32_bits: Option<u32>,
    lrint_threshold_i32: Option<i32>,
    second_pass_triggered: bool,
}

impl NormalizedExponentRow {
    pub const fn base_exponent(&self) -> i16 {
        self.base_exponent
    }

    pub const fn expanded_exponents(&self) -> &[i16; FULLBAND_EXPONENT_BINS] {
        &self.expanded_exponents
    }

    pub const fn grouped_sample_count_i16(&self) -> i16 {
        self.grouped_sample_count_i16
    }

    pub const fn initial_sum_i32(&self) -> Option<i32> {
        self.initial_sum_i32
    }

    pub const fn first_pass_square_error_i32(&self) -> Option<i32> {
        self.first_pass_square_error_i32
    }

    pub const fn error_shift_i32(&self) -> Option<i32> {
        self.error_shift_i32
    }

    pub const fn scaled_count_f32_bits(&self) -> Option<u32> {
        self.scaled_count_f32_bits
    }

    pub const fn lrint_threshold_i32(&self) -> Option<i32> {
        self.lrint_threshold_i32
    }

    pub const fn second_pass_triggered(&self) -> bool {
        self.second_pass_triggered
    }
}

/// Normalizes one selected-profile full-band 217-bin raw exponent row.
///
/// This pure helper receives an already-formed raw row. It does not perform six-block reuse
/// merging, serialize bitstream fields, update owner state, or invoke the LFE path. The caller
/// supplies the hash-pinned policy parameters through the F09PA provider.
pub fn normalize_fullband_exponents(
    raw: &[i16; FULLBAND_EXPONENT_BINS],
    span: u8,
    option: u8,
    gate: i16,
    parameters: Option<&PolicyParameters>,
) -> Result<NormalizedExponentRow, ExponentNormalizationError> {
    if raw
        .iter()
        .any(|value| !(0..=I16_MAX_EXPONENT).contains(value))
        || !matches!(span, 1 | 2 | 4)
        || option > 1
    {
        return Err(ExponentNormalizationError::ExponentInputError);
    }

    let span = usize::from(span);
    let group_count = GROUPED_EXPONENT_BINS / span;
    let mut initial_representatives = [0_i16; GROUPED_EXPONENT_BINS];
    let mut initial_grouped = [0_i16; GROUPED_EXPONENT_BINS];
    for group in 0..group_count {
        let start = 1 + group * span;
        let mut representative = raw[start];
        for value in raw.iter().skip(start + 1).take(span - 1) {
            representative = representative.min(*value);
        }
        representative = representative.min(I16_MAX_EXPONENT);
        initial_representatives[group] = representative;
        initial_grouped[group * span..(group + 1) * span].fill(representative);
    }

    let grouped_sample_count_i16 = i16::try_from(GROUPED_EXPONENT_BINS)
        .map_err(|_| ExponentNormalizationError::NumericFailure)?;
    let initial_sum_i32 = initial_grouped
        .iter()
        .fold(0_i32, |sum, value| sum.wrapping_add(i32::from(*value)));
    let mut base = raw[0].min(I16_BASE_MAX);
    let first_representatives =
        repair_differences(base, initial_representatives, group_count, span, false);
    base = first_representatives.0;
    let first_grouped = expand_groups(&first_representatives.1, group_count, span);
    let mut first_pass_square_error = 0_i32;
    for (normalized, original) in first_grouped.iter().zip(initial_grouped) {
        let difference = i32::from(*normalized) - i32::from(original);
        first_pass_square_error =
            first_pass_square_error.wrapping_add(difference.wrapping_mul(difference));
    }

    let initial_sum_output = if option == 1 {
        Some(initial_sum_i32)
    } else {
        None
    };
    let first_pass_error_output = if option == 1 {
        Some(first_pass_square_error)
    } else {
        None
    };
    let mut error_shift_output = None;
    let mut scaled_count_output = None;
    let mut threshold_output = None;
    let mut second_pass_triggered = false;
    let mut final_grouped = first_grouped;

    if option == 1 {
        let error_shift = first_pass_square_error.wrapping_shl(16);
        let factor = if gate != 0 {
            0.0_f32
        } else {
            let parameters = parameters.ok_or(ExponentNormalizationError::ParameterUnavailable)?;
            f32::from_bits(parameters.d06_normalization_factor_bits())
        };
        let scaled_count = r32_mul(
            finite_f32(f32::from(grouped_sample_count_i16))?,
            finite_f32(factor)?,
        )
        .map_err(|_| ExponentNormalizationError::NumericFailure)?;
        let threshold = if scaled_count.value() < 1.0_f32 {
            let scaled_threshold = r32_mul(scaled_count, finite_f32(f32::from_bits(0x4f00_0000))?)
                .map_err(|_| ExponentNormalizationError::NumericFailure)?;
            let lower_bound = f32::from_bits(0xcf00_0000);
            lrint_rne_i32(scaled_threshold.value().max(lower_bound))?
        } else {
            i32::MAX
        };
        second_pass_triggered = error_shift < threshold
            && initial_sum_i32 < i32::from(grouped_sample_count_i16).wrapping_mul(16);
        error_shift_output = Some(error_shift);
        scaled_count_output = Some(scaled_count.to_bits());
        threshold_output = Some(threshold);

        if second_pass_triggered {
            let second_representatives = regroup_minimum(&first_grouped, group_count, span);
            let repaired =
                repair_differences(base, second_representatives, group_count, span, true);
            base = repaired.0;
            final_grouped = expand_groups(&repaired.1, group_count, span);
        }
    }

    if !(0..=I16_BASE_MAX).contains(&base)
        || final_grouped
            .iter()
            .any(|value| !(0..=I16_MAX_EXPONENT).contains(value))
    {
        return Err(ExponentNormalizationError::ExponentInputError);
    }

    let mut expanded_exponents = [0_i16; FULLBAND_EXPONENT_BINS];
    expanded_exponents[0] = base;
    expanded_exponents[1..].copy_from_slice(&final_grouped);
    Ok(NormalizedExponentRow {
        base_exponent: base,
        expanded_exponents,
        grouped_sample_count_i16,
        initial_sum_i32: initial_sum_output,
        first_pass_square_error_i32: first_pass_error_output,
        error_shift_i32: error_shift_output,
        scaled_count_f32_bits: scaled_count_output,
        lrint_threshold_i32: threshold_output,
        second_pass_triggered,
    })
}

fn repair_differences(
    mut base: i16,
    mut representatives: [i16; GROUPED_EXPONENT_BINS],
    group_count: usize,
    span: usize,
    second_pass: bool,
) -> (i16, [i16; GROUPED_EXPONENT_BINS]) {
    let mut differences = [0_i16; GROUPED_EXPONENT_BINS];
    let mut previous = base;
    for group in 0..group_count {
        let current = representatives[group];
        differences[group] = current.wrapping_sub(previous);
        previous = current;
    }

    let mut group = 0;
    while group < group_count {
        let difference = differences[group];
        if difference > 2 {
            differences[group] = difference.wrapping_sub(1);
            if group + 1 < group_count {
                differences[group + 1] = differences[group + 1].wrapping_add(1);
            }
            continue;
        }

        let negative_requires_repair = if second_pass {
            difference < -1 && group * span < 20
        } else {
            difference < -2
        };
        if negative_requires_repair {
            if group > 0 {
                differences[group - 1] = differences[group - 1].wrapping_sub(1);
                differences[group] = difference.wrapping_add(1);
                group -= 1;
            } else {
                base = base.wrapping_sub(1);
                differences[group] = difference.wrapping_add(1);
            }
            continue;
        }

        group += 1;
    }

    let mut previous = base;
    for group in 0..group_count {
        previous = previous.wrapping_add(differences[group]);
        representatives[group] = previous;
    }
    (base, representatives)
}

fn expand_groups(
    representatives: &[i16; GROUPED_EXPONENT_BINS],
    group_count: usize,
    span: usize,
) -> [i16; GROUPED_EXPONENT_BINS] {
    let mut expanded = [0_i16; GROUPED_EXPONENT_BINS];
    for group in 0..group_count {
        expanded[group * span..(group + 1) * span].fill(representatives[group]);
    }
    expanded
}

fn regroup_minimum(
    expanded: &[i16; GROUPED_EXPONENT_BINS],
    group_count: usize,
    span: usize,
) -> [i16; GROUPED_EXPONENT_BINS] {
    let mut representatives = [0_i16; GROUPED_EXPONENT_BINS];
    for (group, representative) in representatives.iter_mut().enumerate().take(group_count) {
        let start = group * span;
        let mut value = expanded[start];
        for candidate in expanded.iter().skip(start + 1).take(span - 1) {
            value = value.min(*candidate);
        }
        *representative = value;
    }
    representatives
}

fn finite_f32(value: f32) -> Result<FiniteF32, ExponentNormalizationError> {
    FiniteF32::new(value).map_err(|_| ExponentNormalizationError::NumericFailure)
}

fn lrint_rne_i32(value: f32) -> Result<i32, ExponentNormalizationError> {
    let word = value.to_bits();
    let negative = word >> 31 != 0;
    let exponent_field = (word >> 23) & 0xff;
    let fraction = word & 0x007f_ffff;
    let (significand, exponent) = if exponent_field == 0 {
        (u64::from(fraction), -126_i32)
    } else {
        (
            u64::from(fraction | 0x0080_0000),
            i32::try_from(exponent_field)
                .map_err(|_| ExponentNormalizationError::NumericFailure)?
                - 127,
        )
    };
    let magnitude = if exponent < -1 {
        0_u64
    } else if exponent == -1 {
        u64::from(significand > 0x0080_0000)
    } else if exponent >= 23 {
        significand
            << usize::try_from(exponent - 23)
                .map_err(|_| ExponentNormalizationError::NumericFailure)?
    } else {
        let shift =
            u32::try_from(23 - exponent).map_err(|_| ExponentNormalizationError::NumericFailure)?;
        let integer = significand >> shift;
        let remainder = significand & ((1_u64 << shift) - 1);
        let halfway = 1_u64 << (shift - 1);
        if remainder > halfway || (remainder == halfway && integer % 2 == 1) {
            integer + 1
        } else {
            integer
        }
    };
    let signed = if negative {
        -i64::try_from(magnitude).map_err(|_| ExponentNormalizationError::NumericFailure)?
    } else {
        i64::try_from(magnitude).map_err(|_| ExponentNormalizationError::NumericFailure)?
    };
    i32::try_from(signed).map_err(|_| ExponentNormalizationError::NumericFailure)
}
