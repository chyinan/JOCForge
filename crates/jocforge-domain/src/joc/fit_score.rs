// pattern: Functional Core
//! Identity-bound generated score provider and selected JOC score kernel.

use crate::joc::coefficient_normal_equations::CoefficientNormalEquation;
use serde_json::Value;
use sha2::{Digest, Sha256};

const FUNCTIONAL_SHA256: &str = "af08ef531f8708ab81cd6af0bf8ae71504ffb19ba76d5fb8571b1ae8db53c747";
const MATH_SHA256: &str = "d42b7e8ebaabf27bdf519148986b70aa6de56f4eb6bfd0179cd0bd011e284fe7";
const SLOT_COUNT: usize = 48;
const BIN_COUNT: usize = 64;
const COMPONENT_COUNT: usize = 2;
const CHANNEL_COUNT: usize = 5;
const EPSILON: f64 = 9.999_999_960_041_972e-13; // F64(R32(1e-12))

#[derive(Debug)]
pub struct FitScoreProvider {
    near_one: [f64; 4],
    log_a: Vec<f64>,
    log_b: Vec<f64>,
    log_c: Vec<f64>,
    base_correction_a_64: f64,
    base_correction_b_64: f64,
    exp_a: Vec<f64>,
    exp_b: Vec<f64>,
    scalars: ScoreScalars,
}

#[derive(Clone, Copy, Debug)]
struct ScoreScalars {
    e: f64,
    eh: f64,
    el: f64,
    lh: f64,
    ll: f64,
    nh: f64,
    nl: f64,
    rh: f64,
    rl: f64,
    k: f64,
}

impl FitScoreProvider {
    pub fn from_approved_bytes(
        functional_bytes: &[u8],
        math_bytes: &[u8],
    ) -> Result<Self, FitScoreError> {
        if sha256_hex(functional_bytes) != FUNCTIONAL_SHA256
            || sha256_hex(math_bytes) != MATH_SHA256
        {
            return Err(FitScoreError::ProviderIdentity);
        }
        let functional: Value =
            serde_json::from_slice(functional_bytes).map_err(|_| FitScoreError::ProviderShape)?;
        let math: Value =
            serde_json::from_slice(math_bytes).map_err(|_| FitScoreError::ProviderShape)?;
        let near_values = words(&functional["near_one4"]["words_u64_hex"], 4)?;
        let near_one: [f64; 4] = near_values
            .try_into()
            .map_err(|_| FitScoreError::ProviderShape)?;
        let tables = &math["score_tables"];
        let log_a = table(tables, "log10_general_table_a", 257)?;
        let log_b = table(tables, "log10_general_table_b", 257)?;
        let log_c = table(tables, "log10_general_table_c", 257)?;
        let base_reduction_a = table(tables, "pow_base10_reduction_a", 65)?;
        let base_reduction_b = table(tables, "pow_base10_reduction_b", 65)?;
        let base_correction_a = table(tables, "pow_base10_correction_a", 65)?;
        let base_correction_b = table(tables, "pow_base10_correction_b", 65)?;
        let exp_a = table(tables, "pow_exp_reduction_a", 64)?;
        let exp_b = table(tables, "pow_exp_reduction_b", 64)?;
        std::mem::drop((base_reduction_a, base_reduction_b));
        let scalar = |name: &str| parse_f64_word(&math["score_scalars"][name]);
        let scalars = ScoreScalars {
            e: scalar("E")?,
            eh: scalar("EH")?,
            el: scalar("EL")?,
            lh: scalar("LH")?,
            ll: scalar("LL")?,
            nh: scalar("NH")?,
            nl: scalar("NL")?,
            rh: scalar("RH")?,
            rl: scalar("RL")?,
            k: scalar("K")?,
        };
        Ok(Self {
            near_one,
            log_a,
            log_b,
            log_c,
            base_correction_a_64: base_correction_a[64],
            base_correction_b_64: base_correction_b[64],
            exp_a,
            exp_b,
            scalars,
        })
    }

    fn score(
        &self,
        error: f64,
        model_energy: f64,
        target_energy: f64,
    ) -> Result<f64, FitScoreError> {
        if !error.is_finite()
            || !model_energy.is_finite()
            || !target_energy.is_finite()
            || error < 0.0
            || model_energy < 0.0
            || target_energy < 0.0
        {
            return Err(FitScoreError::NumericDomain);
        }
        let ratio = (model_energy + EPSILON) / (target_energy + EPSILON) + 1.0 / 20.0;
        if !ratio.is_finite() || ratio <= 0.0 {
            return Err(FitScoreError::NumericDomain);
        }
        let logarithm = self.log10(ratio)?;
        let magnitude = self.pow10(logarithm.abs())?;
        let score = magnitude * (error + EPSILON);
        if score.is_finite() && score >= 0.0 {
            Ok(score)
        } else {
            Err(FitScoreError::NumericDomain)
        }
    }

    #[allow(clippy::many_single_char_names)] // Names mirror the approved polynomial contract.
    fn log10(&self, value: f64) -> Result<f64, FitScoreError> {
        if !value.is_finite() || value <= 0.0 {
            return Err(FitScoreError::NumericDomain);
        }
        if (value - 1.0).abs() < 1.0 / 16.0 {
            let u = value - 1.0;
            let t = u / (u + 2.0);
            let s = t + t;
            let d = u * t;
            let s2 = s * s;
            let mut a = self.near_one[0].mul_add(s2, self.near_one[1]);
            let mut b = self.near_one[2].mul_add(s2, self.near_one[3]);
            let s3 = s2 * s;
            a *= s3;
            let s6 = s3 * s3;
            let s7 = s6 * s;
            b *= s7;
            let mut remainder = (a + b) - d;
            let high_u = f64::from_bits(u.to_bits() & 0xffff_ffff_0000_0000);
            let low_u = u - high_u;
            remainder += low_u;
            return Ok(
                (((high_u * self.scalars.el) + (remainder * self.scalars.el))
                    + (remainder * self.scalars.eh))
                    + (high_u * self.scalars.eh),
            );
        }

        let word = value.to_bits();
        let exponent_word = (word >> 52) & 0x7ff;
        if exponent_word == 0 || exponent_word == 0x7ff {
            return Err(FitScoreError::NumericDomain);
        }
        let fraction = word & ((1_u64 << 52) - 1);
        let window = 2 * (word & (1_u64 << 43)) + (word & 0x000f_f000_0000_0000);
        let index = usize::try_from(window >> 44).map_err(|_| FitScoreError::NumericDomain)?;
        let anchor = f64::from_bits(0x3fe0_0000_0000_0000 | window);
        let input = f64::from_bits(0x3fe0_0000_0000_0000 | fraction);
        let z = (anchor - input) * self.log_c[index];
        let z2 = z * z;
        let mut a = (1.0 / 6.0_f64).mul_add(z, 1.0 / 5.0);
        let b = (1.0 / 3.0_f64).mul_add(z, 1.0 / 2.0);
        a = a.mul_add(z, 1.0 / 4.0);
        let z4 = z2 * z2;
        let mut polynomial = b.mul_add(z2, z);
        polynomial = a.mul_add(z4, polynomial);
        polynomial *= self.scalars.e;
        let exponent = f64::from(
            i32::try_from(exponent_word).map_err(|_| FitScoreError::NumericDomain)? - 1023,
        );
        let correction = exponent.mul_add(self.scalars.ll, -polynomial);
        let high = exponent.mul_add(self.scalars.lh, self.log_a[index]);
        let low = self.log_b[index] + correction;
        let result = high + low;
        if result.is_finite() {
            Ok(result)
        } else {
            Err(FitScoreError::NumericDomain)
        }
    }

    #[allow(clippy::float_cmp)] // Exact 0 and 1 branches are part of the approved kernel.
    fn pow10(&self, value: f64) -> Result<f64, FitScoreError> {
        if !value.is_finite() || value < 0.0 {
            return Err(FitScoreError::NumericDomain);
        }
        if value == 0.0 {
            return Ok(1.0);
        }
        if value == 1.0 {
            return Ok(10.0);
        }

        let correction = 3.0_f64.mul_add(self.scalars.nl, -0.0);
        let low0 = correction + self.base_correction_b_64;
        let low_delta = low0;
        let high0 = 3.0_f64.mul_add(self.scalars.nh, self.base_correction_a_64);
        let sum0 = high0 + low_delta;
        let high = clear_low_fraction_bits(sum0, 27);
        let zlow = low_delta;
        let mut error0 = high0 - sum0;
        let error1 = low0 - zlow;
        error0 += low_delta;
        error0 += error1;
        let sumlow = sum0 - high;
        let loglow = error0 + sumlow;

        let value_high = clear_low_fraction_bits(value, 27);
        let value_low = value - value_high;
        let product_high = high * value_high;
        let product_low = ((value_low * loglow) + (value_low * high)) + (loglow * value_high);
        let product_hi = product_high + product_low;
        let product_error = (product_high - product_hi) + product_low;
        let scaled = product_hi * self.scalars.k;
        if !scaled.is_finite() || scaled < i64::MIN as f64 || scaled > i64::MAX as f64 {
            return Err(FitScoreError::NumericDomain);
        }
        let rounded = scaled.round_ties_even();
        let integer = rounded as i64;
        let reduction_index =
            usize::try_from(integer & 63).map_err(|_| FitScoreError::NumericDomain)?;
        let exponent = (integer - i64::try_from(reduction_index).unwrap_or_default()) >> 6;
        if !(-1022..1024).contains(&exponent) {
            return Err(FitScoreError::NumericDomain);
        }
        let integer_f64 = integer as f64;
        let remainder_high = (-integer_f64).mul_add(self.scalars.rh, product_hi);
        let low_product = integer_f64 * self.scalars.rl;
        let remainder = (remainder_high + low_product) + product_error;

        let mut polynomial = 1.0 / 720.0_f64;
        for coefficient in [1.0 / 120.0, 1.0 / 24.0, 1.0 / 6.0, 1.0 / 2.0, 1.0] {
            polynomial = polynomial.mul_add(remainder, coefficient);
        }
        polynomial *= remainder;
        let mut tail_b = polynomial * self.exp_b[reduction_index];
        let mut tail_a = polynomial * self.exp_a[reduction_index];
        tail_b += self.exp_b[reduction_index];
        tail_a += tail_b;
        let approximation = tail_a + self.exp_a[reduction_index];
        let scale = f64::from_bits(
            u64::try_from(exponent + 1023).map_err(|_| FitScoreError::NumericDomain)? << 52,
        );
        let result = approximation * scale;
        if result.is_finite() {
            Ok(result)
        } else {
            Err(FitScoreError::NumericDomain)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FitScoreEvaluation {
    error: f64,
    model_energy: f64,
    target_energy: f64,
    ratio: f64,
    score: f64,
}

impl FitScoreEvaluation {
    pub const fn error(self) -> f64 {
        self.error
    }

    pub const fn model_energy(self) -> f64 {
        self.model_energy
    }

    pub const fn target_energy(self) -> f64 {
        self.target_energy
    }

    pub const fn ratio(self) -> f64 {
        self.ratio
    }

    pub const fn score(self) -> f64 {
        self.score
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FitScoreError {
    ProviderIdentity,
    ProviderShape,
    InputShape,
    InputNonFinite,
    NumericDomain,
}

impl FitScoreError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ProviderIdentity => "JOC_SCORE_PROVIDER_IDENTITY",
            Self::ProviderShape => "JOC_SCORE_PROVIDER_SHAPE",
            Self::InputShape => "JOC_SCORE_INPUT_SHAPE",
            Self::InputNonFinite => "JOC_SCORE_INPUT_NONFINITE",
            Self::NumericDomain => "JOC_SCORE_NUMERIC_FAILURE",
        }
    }
}

impl std::fmt::Display for FitScoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for FitScoreError {}

/// Evaluates the selected score for one trial-restored coefficient row.
#[allow(clippy::needless_range_loop)] // Score accumulation order is index-defined.
pub fn score_trial_coefficients(
    provider: &FitScoreProvider,
    equation: &CoefficientNormalEquation,
    target_words: &[u32],
    carrier_words: &[u32],
    weight_words: &[u32; SLOT_COUNT],
    trial_coefficient_words: &[u32; CHANNEL_COUNT],
) -> Result<FitScoreEvaluation, FitScoreError> {
    if target_words.len() != 15 * SLOT_COUNT * BIN_COUNT * COMPONENT_COUNT
        || carrier_words.len() != CHANNEL_COUNT * SLOT_COUNT * BIN_COUNT * COMPONENT_COUNT
    {
        return Err(FitScoreError::InputShape);
    }
    let object = usize::from(equation.object());
    let mut error = 0.0_f64;
    let mut model_energy = 0.0_f64;
    let mut target_energy = 0.0_f64;
    for slot in 0..SLOT_COUNT {
        let weight = finite_f32(weight_words[slot])?;
        for bin in usize::from(equation.begin_bin())..usize::from(equation.end_bin()) {
            let target = complex(target_words, object, 15, slot, bin)?;
            let mut residual_real = -f64::from(weighted(target[0], weight)?);
            let mut residual_imaginary = -f64::from(weighted(target[1], weight)?);
            let old_real = residual_real;
            let old_imaginary = residual_imaginary;
            for channel in 0..CHANNEL_COUNT {
                let coefficient = finite_f32(trial_coefficient_words[channel])?;
                if coefficient == 0.0 {
                    continue;
                }
                let carrier = complex(carrier_words, channel, CHANNEL_COUNT, slot, bin)?;
                residual_real += f64::from(weighted(carrier[0], weight)?) * f64::from(coefficient);
                residual_imaginary +=
                    f64::from(weighted(carrier[1], weight)?) * f64::from(coefficient);
            }
            error =
                (residual_real * residual_real + error) + residual_imaginary * residual_imaginary;
            let model_real = residual_real - old_real;
            let model_imaginary = residual_imaginary - old_imaginary;
            model_energy =
                (model_real * model_real + model_energy) + model_imaginary * model_imaginary;
            target_energy = (old_real * old_real + target_energy) + old_imaginary * old_imaginary;
        }
    }
    let ratio = (model_energy + EPSILON) / (target_energy + EPSILON) + 1.0 / 20.0;
    let score = provider.score(error, model_energy, target_energy)?;
    Ok(FitScoreEvaluation {
        error,
        model_energy,
        target_energy,
        ratio,
        score,
    })
}

fn table(root: &Value, name: &str, expected: usize) -> Result<Vec<f64>, FitScoreError> {
    let entry = &root[name];
    if entry["count"].as_u64() != Some(expected as u64) || entry["type"] != "F64" {
        return Err(FitScoreError::ProviderShape);
    }
    words(&entry["words_u64_hex"], expected)
}

fn words(value: &Value, expected: usize) -> Result<Vec<f64>, FitScoreError> {
    let values = value.as_array().ok_or(FitScoreError::ProviderShape)?;
    if values.len() != expected {
        return Err(FitScoreError::ProviderShape);
    }
    values.iter().map(parse_f64_word).collect()
}

fn parse_f64_word(value: &Value) -> Result<f64, FitScoreError> {
    let text = value.as_str().ok_or(FitScoreError::ProviderShape)?;
    if text.len() != 16 {
        return Err(FitScoreError::ProviderShape);
    }
    let word = u64::from_str_radix(text, 16).map_err(|_| FitScoreError::ProviderShape)?;
    let value = f64::from_bits(word);
    if value.is_finite() {
        Ok(value)
    } else {
        Err(FitScoreError::ProviderShape)
    }
}

fn clear_low_fraction_bits(value: f64, count: u32) -> f64 {
    f64::from_bits(value.to_bits() & !((1_u64 << count) - 1))
}

fn complex(
    words: &[u32],
    row: usize,
    row_count: usize,
    slot: usize,
    bin: usize,
) -> Result<[f32; 2], FitScoreError> {
    if row >= row_count || slot >= SLOT_COUNT || bin >= BIN_COUNT {
        return Err(FitScoreError::InputShape);
    }
    let offset = ((row * SLOT_COUNT + slot) * BIN_COUNT + bin) * COMPONENT_COUNT;
    Ok([finite_f32(words[offset])?, finite_f32(words[offset + 1])?])
}

fn finite_f32(word: u32) -> Result<f32, FitScoreError> {
    let value = f32::from_bits(word);
    if value.is_finite() {
        Ok(value)
    } else {
        Err(FitScoreError::InputNonFinite)
    }
}

fn weighted(value: f32, weight: f32) -> Result<f32, FitScoreError> {
    let product = value * weight;
    if product.is_finite() {
        Ok(product)
    } else {
        Err(FitScoreError::NumericDomain)
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FUNCTIONAL: &[u8] = include_bytes!(
        "../../tests/fixtures/synthetic_qmf_fit_checkpoints/approved-joc-functional.json"
    );
    const MATH: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_qmf_fit_checkpoints/approved-joc-math.json");

    #[test]
    fn approved_provider_is_identity_and_shape_gated() {
        FitScoreProvider::from_approved_bytes(FUNCTIONAL, MATH).unwrap();
        let mut changed = MATH.to_vec();
        changed[0] ^= 1;
        assert_eq!(
            FitScoreProvider::from_approved_bytes(FUNCTIONAL, &changed).unwrap_err(),
            FitScoreError::ProviderIdentity
        );
    }
}
