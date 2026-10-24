// pattern: Functional Core
//! Selected-profile mean-to-db-code calculation from precomputed channel means.

use super::policy_parameters::PolicyParameters;

/// Number of caller-supplied F32 channel means in the selected profile.
pub const DB_MEAN_COUNT: usize = 6;

/// Result of calculating one db-per-bit code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DbSelectorResult {
    selector_after: u16,
    maximum_mean_f32_bits: u32,
    level_f32_bits: u32,
    difference_f32_bits: u32,
    strict_hysteresis_triggered: bool,
}

impl DbSelectorResult {
    pub const fn selector_after(self) -> u16 {
        self.selector_after
    }

    pub const fn maximum_mean_f32_bits(self) -> u32 {
        self.maximum_mean_f32_bits
    }

    pub const fn level_f32_bits(self) -> u32 {
        self.level_f32_bits
    }

    pub const fn difference_f32_bits(self) -> u32 {
        self.difference_f32_bits
    }

    pub const fn strict_hysteresis_triggered(self) -> bool {
        self.strict_hysteresis_triggered
    }
}

/// Input-domain and numerical failures for the precomputed-mean selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DbSelectorError {
    ExponentInputError,
    NumericFailure,
}

impl DbSelectorError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ExponentInputError => "EXPONENT_INPUT_ERROR",
            Self::NumericFailure => "NUMERIC_FAILURE",
        }
    }
}

/// Computes one selected-profile db code from six already-computed mean words.
///
/// The first five means participate in the maximum scan; ties select the later
/// visited value. The sixth, LFE mean is input-domain checked but excluded from
/// this maximum. The helper does not compute channel means or commit selector
/// state; it returns the next selector as a value.
pub fn db_selector_from_means(
    mean_f32_bits: &[u32; DB_MEAN_COUNT],
    selector_before: u16,
    parameters: &PolicyParameters,
) -> Result<DbSelectorResult, DbSelectorError> {
    if selector_before > 3 {
        return Err(DbSelectorError::ExponentInputError);
    }

    let mut means = [0.0_f32; DB_MEAN_COUNT];
    for (destination, word) in means.iter_mut().zip(mean_f32_bits) {
        let mean = f32::from_bits(*word);
        if !mean.is_finite() || mean < 0.0 {
            return Err(DbSelectorError::ExponentInputError);
        }
        *destination = mean;
    }

    let mut maximum_mean = means[0];
    for mean in means.iter().take(DB_MEAN_COUNT - 1).skip(1) {
        if *mean >= maximum_mean {
            maximum_mean = *mean;
        }
    }
    let maximum_mean_f32_bits = maximum_mean.to_bits();
    let headroom = headroom_from_f32_bits(maximum_mean_f32_bits);

    let level = if maximum_mean == 0.0 || headroom >= 48 {
        1.0_f32
    } else {
        let scale_exponent =
            u32::try_from(headroom + 127).map_err(|_| DbSelectorError::NumericFailure)?;
        let scale = f32::from_bits(scale_exponent << 23);
        let value = maximum_mean * scale;
        require_finite(value)?;

        let linear = value * f32::from_bits(parameters.d07_polynomial_bits()[1]);
        require_finite(linear)?;

        let square = value * value;
        require_finite(square)?;
        let quadratic = square * f32::from_bits(parameters.d07_polynomial_bits()[2]);
        require_finite(quadratic)?;

        let offset_minus_linear = f32::from_bits(parameters.d07_polynomial_bits()[0]) - linear;
        require_finite(offset_minus_linear)?;
        let polynomial = offset_minus_linear + quadratic;
        require_finite(polynomial)?;

        let scaled_polynomial = polynomial * f32::from_bits(parameters.d07_polynomial_bits()[4]);
        require_finite(scaled_polynomial)?;
        let headroom_term = (headroom as f32) * f32::from_bits(parameters.d07_polynomial_bits()[3]);
        require_finite(headroom_term)?;
        let level = scaled_polynomial + headroom_term;
        require_finite(level)?;
        level
    };

    let anchor = f32::from_bits(parameters.d07_anchors_bits()[usize::from(selector_before)]);
    let difference = level - anchor;
    require_finite(difference)?;
    let hysteresis_triggered = difference.abs() > f32::from_bits(parameters.d07_hysteresis_bits());

    let selector_after = if hysteresis_triggered {
        let mut selector = 0_usize;
        while selector < 3 && f32::from_bits(parameters.d07_thresholds_bits()[selector]) > level {
            selector += 1;
        }
        u16::try_from(selector).map_err(|_| DbSelectorError::NumericFailure)?
    } else {
        selector_before
    };

    Ok(DbSelectorResult {
        selector_after,
        maximum_mean_f32_bits,
        level_f32_bits: level.to_bits(),
        difference_f32_bits: difference.to_bits(),
        strict_hysteresis_triggered: hysteresis_triggered,
    })
}

fn headroom_from_f32_bits(word: u32) -> i32 {
    let doubled = word.wrapping_mul(2);
    let source_exponent = if doubled == 0 {
        -127
    } else {
        i32::from(doubled.to_be_bytes()[0]) - 126
    };
    (-source_exponent).clamp(0, 60)
}

fn require_finite(value: f32) -> Result<(), DbSelectorError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(DbSelectorError::NumericFailure)
    }
}
