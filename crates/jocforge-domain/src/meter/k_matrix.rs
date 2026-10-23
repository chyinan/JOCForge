// pattern: Functional Core
//! Exact-rational selected 48 kHz modern two-stage K-matrix parameter provider.

use num_bigint::BigInt;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;

const MATRIX_INPUTS: &str = include_str!("k_matrix_parameters.json");
const MATRIX_INPUTS_SHA256: &str =
    "7396e0a9e54b97ac7c6cd02ae8e4b519815a195fdeda3042f1a46b30691ba363";
const STATE_BOUNDS: &str = include_str!("k_matrix_bounds.json");
const STATE_BOUNDS_SHA256: &str =
    "2faaa9d0aeefb105a913589590fcbafb97362a54192b0b21c5fdda7da1974e68";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KMatrixMode {
    Modern,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KMatrixProfile {
    pub sample_rate_hz: u32,
    pub mode: KMatrixMode,
    pub stage_count: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KMatrixError {
    UnsupportedProfile,
    ParameterUnavailable,
    NumericFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KMatrixProvider {
    complete_words: [u32; 72],
    lower_words: [u32; 4],
    upper_words: [u32; 4],
    noise_word: u32,
}

impl KMatrixProvider {
    /// Returns the 64 non-cold matrix words in stage/row/lane order.
    pub fn matrix_words(&self) -> [u32; 64] {
        let mut result = [0; 64];
        let mut output = 0;
        for stage in 0..2 {
            let stage_start = stage * 36;
            for row in 0..8 {
                for lane in 0..4 {
                    result[output] = self.complete_words[stage_start + row * 4 + lane];
                    output += 1;
                }
            }
        }
        result
    }

    /// Returns the eight cold words, stage-major and lane-major.
    pub fn cold_state_words(&self) -> [u32; 8] {
        let mut result = [0; 8];
        result[..4].copy_from_slice(&self.complete_words[32..36]);
        result[4..].copy_from_slice(&self.complete_words[68..72]);
        result
    }

    pub const fn complete_words(&self) -> &[u32; 72] {
        &self.complete_words
    }

    pub const fn lower_words(&self) -> &[u32; 4] {
        &self.lower_words
    }

    pub const fn upper_words(&self) -> &[u32; 4] {
        &self.upper_words
    }

    pub const fn noise_word(&self) -> u32 {
        self.noise_word
    }

    pub fn complete_f32le_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(288);
        for word in self.complete_words {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes
    }
}

/// Generates the selected K matrix and cold/bounds provider from input parameters only.
pub fn generate_k_matrix(profile: KMatrixProfile) -> Result<KMatrixProvider, KMatrixError> {
    if profile.sample_rate_hz != 48_000
        || profile.mode != KMatrixMode::Modern
        || profile.stage_count != 2
    {
        return Err(KMatrixError::UnsupportedProfile);
    }

    let inputs = parse_matrix_inputs()?;
    let bounds = parse_state_bounds()?;
    let mut complete_words = [0; 72];
    for (stage_index, stage) in inputs.stages.iter().enumerate() {
        let matrix = generate_stage_matrix(stage)?;
        let stage_start = stage_index * 36;
        complete_words[stage_start..stage_start + 32].copy_from_slice(&matrix);
        complete_words[stage_start + 32..stage_start + 36]
            .copy_from_slice(&bounds.cold_words[stage_index * 4..stage_index * 4 + 4]);
    }

    Ok(KMatrixProvider {
        complete_words,
        lower_words: bounds.lower_words,
        upper_words: bounds.upper_words,
        noise_word: bounds.noise_word,
    })
}

struct MatrixInputs {
    stages: [StageCoefficients; 2],
}

#[derive(Clone)]
struct StageCoefficients {
    b0: Rational,
    b1: Rational,
    b2: Rational,
    f1: Rational,
    f2: Rational,
}

fn parse_matrix_inputs() -> Result<MatrixInputs, KMatrixError> {
    if sha256(MATRIX_INPUTS.as_bytes()) != MATRIX_INPUTS_SHA256 {
        return Err(KMatrixError::ParameterUnavailable);
    }
    let root: Value =
        serde_json::from_str(MATRIX_INPUTS).map_err(|_| KMatrixError::ParameterUnavailable)?;
    if root["schema"] != "meter-k-matrix-mathematical-parameter.v1"
        || root["parameter_route"] != "K_MATRIX_PUBLIC_BIQUAD_RATIONAL_DECIMAL_V1"
        || !matrix_rules_are_selected(&root["matrix_generation"])
    {
        return Err(KMatrixError::ParameterUnavailable);
    }

    let gain_proof = &root["shelf_gain_rounding_proof"];
    if gain_proof["expression"] != "nearest_decimal_1e-6(10^(-3691/20000))"
        || gain_proof["lower_power_comparison"] != -1
        || gain_proof["upper_power_comparison"] != 1
        || gain_proof["tie"] != false
    {
        return Err(KMatrixError::ParameterUnavailable);
    }
    let gain = Rational::from_fraction(
        gain_proof["exact_rational_result"]
            .as_str()
            .ok_or(KMatrixError::ParameterUnavailable)?,
    )
    .ok_or(KMatrixError::ParameterUnavailable)?;

    let stage_values = root["stages"]
        .as_array()
        .filter(|stages| stages.len() == 2)
        .ok_or(KMatrixError::ParameterUnavailable)?;
    if stage_values[0]["name"] != "shelf"
        || stage_values[0]["numerator_scale"] != "nearest_decimal_1e-6(10^(-(3+691/1000)/20))"
        || stage_values[1]["name"] != "highpass"
        || stage_values[1]["numerator_scale"] != "1"
    {
        return Err(KMatrixError::ParameterUnavailable);
    }

    let shelf = parse_stage(&stage_values[0], Some(&gain))?;
    let highpass = parse_stage(&stage_values[1], None)?;
    Ok(MatrixInputs {
        stages: [shelf, highpass],
    })
}

fn matrix_rules_are_selected(rules: &Value) -> bool {
    let basis = [
        "x0", "x1", "x2", "x3", "x_minus1", "x_minus2", "y_minus1", "y_minus2",
    ];
    let columns = ["y0", "y1", "y2", "y3"];
    rules["recurrence"] == "y[n]=b0*x[n]+b1*x[n-1]+b2*x[n-2]-a1*y[n-1]-a2*y[n-2]"
        && rules["intermediate_arithmetic"] == "exact rational"
        && rules["entry_quantization"]
            == "nearest decimal 1e-8 ties to even, then exact F32 round to nearest ties to even"
        && rules["storage_order"] == "stage, row, lane; F32LE"
        && rules["result_shape"] == serde_json::json!([2, 9, 4])
        && string_array_matches(&rules["basis_order"], &basis)
        && string_array_matches(&rules["column_order"], &columns)
        && rules["cold_row"].as_array().is_some_and(|items| {
            items.len() == 4 && items.iter().all(|value| value == "positive_zero")
        })
}

fn string_array_matches(value: &Value, expected: &[&str]) -> bool {
    value.as_array().is_some_and(|items| {
        items.len() == expected.len()
            && items
                .iter()
                .zip(expected)
                .all(|(item, expected)| item.as_str() == Some(*expected))
    })
}

fn parse_stage(
    value: &Value,
    numerator_scale: Option<&Rational>,
) -> Result<StageCoefficients, KMatrixError> {
    let coefficient = |field: &str| {
        Rational::from_decimal(
            value[field]
                .as_str()
                .ok_or(KMatrixError::ParameterUnavailable)?,
        )
        .ok_or(KMatrixError::ParameterUnavailable)
    };
    let mut b0 = coefficient("b0")?;
    let mut b1 = coefficient("b1")?;
    let mut b2 = coefficient("b2")?;
    if let Some(scale) = numerator_scale {
        b0 = b0.mul(scale);
        b1 = b1.mul(scale);
        b2 = b2.mul(scale);
    }
    Ok(StageCoefficients {
        b0,
        b1,
        b2,
        f1: coefficient("a1")?.negated(),
        f2: coefficient("a2")?.negated(),
    })
}

fn parse_state_bounds() -> Result<StateBounds, KMatrixError> {
    if sha256(STATE_BOUNDS.as_bytes()) != STATE_BOUNDS_SHA256 {
        return Err(KMatrixError::ParameterUnavailable);
    }
    let root: Value =
        serde_json::from_str(STATE_BOUNDS).map_err(|_| KMatrixError::ParameterUnavailable)?;
    if root["schema"] != "meter-k-basic-state-and-bounds.v1"
        || root["stage_count"] != 2
        || root["cold_state_rule"] != "positive zero for every stage/lane"
        || root["matrix_provider"] != "parameters/meter-k-matrix-math-v1.json"
        || root["delivery_scope"] != "INTERNAL_ONLY"
    {
        return Err(KMatrixError::ParameterUnavailable);
    }
    let cold_words = parse_hex_array::<8>(
        root["cold_state_words"]
            .as_array()
            .ok_or(KMatrixError::ParameterUnavailable)?,
    )?;
    if cold_words.iter().any(|word| *word != 0) {
        return Err(KMatrixError::ParameterUnavailable);
    }
    Ok(StateBounds {
        cold_words,
        lower_words: parse_hex_array::<4>(
            root["lower_words"]
                .as_array()
                .ok_or(KMatrixError::ParameterUnavailable)?,
        )?,
        upper_words: parse_hex_array::<4>(
            root["upper_words"]
                .as_array()
                .ok_or(KMatrixError::ParameterUnavailable)?,
        )?,
        noise_word: parse_hex(
            root["noise_word"]
                .as_str()
                .ok_or(KMatrixError::ParameterUnavailable)?,
        )?,
    })
}

struct StateBounds {
    cold_words: [u32; 8],
    lower_words: [u32; 4],
    upper_words: [u32; 4],
    noise_word: u32,
}

fn parse_hex_array<const N: usize>(value: &[Value]) -> Result<[u32; N], KMatrixError> {
    let words = value
        .iter()
        .flat_map(|row| {
            row.as_array()
                .map_or_else(|| vec![row], |values| values.iter().collect::<Vec<_>>())
        })
        .map(|word| parse_hex(word.as_str().ok_or(KMatrixError::ParameterUnavailable)?))
        .collect::<Result<Vec<_>, _>>()?;
    words
        .try_into()
        .map_err(|_| KMatrixError::ParameterUnavailable)
}

fn parse_hex(value: &str) -> Result<u32, KMatrixError> {
    u32::from_str_radix(value, 16).map_err(|_| KMatrixError::ParameterUnavailable)
}

fn generate_stage_matrix(stage: &StageCoefficients) -> Result<[u32; 32], KMatrixError> {
    let mut words = [0; 32];
    for basis in 0..8 {
        let mut x: [Rational; 4] = std::array::from_fn(|_| Rational::zero());
        let mut x_minus1 = Rational::zero();
        let mut x_minus2 = Rational::zero();
        let mut y_minus1 = Rational::zero();
        let mut y_minus2 = Rational::zero();
        match basis {
            0..=3 => x[basis] = Rational::one(),
            4 => x_minus1 = Rational::one(),
            5 => x_minus2 = Rational::one(),
            6 => y_minus1 = Rational::one(),
            7 => y_minus2 = Rational::one(),
            _ => return Err(KMatrixError::NumericFailure),
        }

        for lane in 0..4 {
            let mut y = stage.b0.mul(&x[lane]);
            y = y.add(&stage.b1.mul(&x_minus1));
            y = y.add(&stage.b2.mul(&x_minus2));
            y = y.add(&stage.f1.mul(&y_minus1));
            y = y.add(&stage.f2.mul(&y_minus2));
            words[basis * 4 + lane] = y.quantized_f32_word()?;

            x_minus2 = x_minus1;
            x_minus1 = x[lane].clone();
            y_minus2 = y_minus1;
            y_minus1 = y;
        }
    }
    Ok(words)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Rational {
    numerator: BigInt,
    denominator: BigInt,
}

impl Rational {
    fn new(numerator: BigInt, denominator: BigInt) -> Result<Self, KMatrixError> {
        if denominator == BigInt::from(0) {
            return Err(KMatrixError::NumericFailure);
        }
        let (mut numerator, mut denominator) = if denominator < BigInt::from(0) {
            (-numerator, -denominator)
        } else {
            (numerator, denominator)
        };
        let divisor = gcd(magnitude(&numerator), denominator.clone());
        if divisor != BigInt::from(0) {
            numerator /= &divisor;
            denominator /= divisor;
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }

    fn zero() -> Self {
        Self {
            numerator: BigInt::from(0),
            denominator: BigInt::from(1),
        }
    }

    fn one() -> Self {
        Self {
            numerator: BigInt::from(1),
            denominator: BigInt::from(1),
        }
    }

    fn from_fraction(value: &str) -> Option<Self> {
        let (numerator, denominator) = value.split_once('/')?;
        Self::new(
            BigInt::parse_bytes(numerator.as_bytes(), 10)?,
            BigInt::parse_bytes(denominator.as_bytes(), 10)?,
        )
        .ok()
    }

    fn from_decimal(value: &str) -> Option<Self> {
        let (negative, unsigned) = value
            .strip_prefix('-')
            .map_or((false, value), |rest| (true, rest));
        let (whole, fractional) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        if whole.is_empty()
            || !whole.bytes().all(|byte| byte.is_ascii_digit())
            || !fractional.bytes().all(|byte| byte.is_ascii_digit())
        {
            return None;
        }
        let digits = format!("{whole}{fractional}");
        let mut numerator = BigInt::parse_bytes(digits.as_bytes(), 10)?;
        if negative {
            numerator = -numerator;
        }
        let denominator = power_of_ten(fractional.len());
        Self::new(numerator, denominator).ok()
    }

    fn add(&self, other: &Self) -> Self {
        let numerator = &self.numerator * &other.denominator + &other.numerator * &self.denominator;
        let denominator = &self.denominator * &other.denominator;
        Self::new(numerator, denominator).expect("positive nonzero rational denominator")
    }

    fn mul(&self, other: &Self) -> Self {
        Self::new(
            &self.numerator * &other.numerator,
            &self.denominator * &other.denominator,
        )
        .expect("positive nonzero rational denominator")
    }

    fn negated(&self) -> Self {
        Self {
            numerator: -self.numerator.clone(),
            denominator: self.denominator.clone(),
        }
    }

    fn quantized_f32_word(&self) -> Result<u32, KMatrixError> {
        let scaled_numerator = &self.numerator * power_of_ten(8);
        let rounded = round_ratio_to_integer(&scaled_numerator, &self.denominator);
        let decimal = Self::new(rounded, power_of_ten(8))?;
        decimal.to_f32_word()
    }

    fn to_f32_word(&self) -> Result<u32, KMatrixError> {
        if self.numerator == BigInt::from(0) {
            return Ok(0);
        }
        let negative = self.numerator < BigInt::from(0);
        let sign = if negative { 0x8000_0000 } else { 0 };
        let numerator = magnitude(&self.numerator);
        let denominator = self.denominator.clone();
        let mut exponent = i32::try_from(numerator.bits())
            .map_err(|_| KMatrixError::NumericFailure)?
            - i32::try_from(denominator.bits()).map_err(|_| KMatrixError::NumericFailure)?;
        let comparison = compare_ratio_to_power_of_two(&numerator, &denominator, exponent)?;
        if comparison == Ordering::Less {
            exponent -= 1;
        }

        if exponent < -126 {
            let significand = round_scaled_ratio(&numerator, &denominator, 149)?;
            let significand = nonnegative_u32(&significand)?;
            if significand == 0 {
                return Ok(sign);
            }
            if significand >= (1 << 23) {
                return Ok(sign | (1 << 23));
            }
            return Ok(sign | significand);
        }

        let mut significand = round_scaled_ratio(&numerator, &denominator, 23 - exponent)?;
        let two_to_24 = BigInt::from(1_u32 << 24);
        if significand == two_to_24 {
            significand >>= 1;
            exponent += 1;
        }
        if exponent > 127 {
            return Err(KMatrixError::NumericFailure);
        }
        let significand = nonnegative_u32(&significand)?;
        let fraction = significand
            .checked_sub(1 << 23)
            .filter(|value| *value < (1 << 23))
            .ok_or(KMatrixError::NumericFailure)?;
        let exponent_bits =
            u32::try_from(exponent + 127).map_err(|_| KMatrixError::NumericFailure)?;
        Ok(sign | (exponent_bits << 23) | fraction)
    }
}

fn compare_ratio_to_power_of_two(
    numerator: &BigInt,
    denominator: &BigInt,
    exponent: i32,
) -> Result<Ordering, KMatrixError> {
    if exponent >= 0 {
        let shift = usize::try_from(exponent).map_err(|_| KMatrixError::NumericFailure)?;
        Ok(numerator.cmp(&(denominator << shift)))
    } else {
        let shift = usize::try_from(-exponent).map_err(|_| KMatrixError::NumericFailure)?;
        Ok((numerator << shift).cmp(denominator))
    }
}

fn round_scaled_ratio(
    numerator: &BigInt,
    denominator: &BigInt,
    power: i32,
) -> Result<BigInt, KMatrixError> {
    if power >= 0 {
        let shift = usize::try_from(power).map_err(|_| KMatrixError::NumericFailure)?;
        Ok(round_ratio_to_integer(&(numerator << shift), denominator))
    } else {
        let shift = usize::try_from(-power).map_err(|_| KMatrixError::NumericFailure)?;
        Ok(round_ratio_to_integer(numerator, &(denominator << shift)))
    }
}

fn round_ratio_to_integer(numerator: &BigInt, denominator: &BigInt) -> BigInt {
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    let twice_remainder = magnitude(&remainder) << 1_usize;
    let comparison = twice_remainder.cmp(denominator);
    let odd = (&quotient % BigInt::from(2)) != BigInt::from(0);
    if comparison == Ordering::Greater || (comparison == Ordering::Equal && odd) {
        if numerator < &BigInt::from(0) {
            quotient - 1
        } else {
            quotient + 1
        }
    } else {
        quotient
    }
}

fn gcd(mut left: BigInt, mut right: BigInt) -> BigInt {
    while right != BigInt::from(0) {
        let remainder = &left % &right;
        left = right;
        right = remainder;
    }
    left
}

fn magnitude(value: &BigInt) -> BigInt {
    if value < &BigInt::from(0) {
        -value
    } else {
        value.clone()
    }
}

fn power_of_ten(power: usize) -> BigInt {
    (0..power).fold(BigInt::from(1), |value, _| value * 10)
}

fn nonnegative_u32(value: &BigInt) -> Result<u32, KMatrixError> {
    if value < &BigInt::from(0) {
        return Err(KMatrixError::NumericFailure);
    }
    value
        .to_str_radix(10)
        .parse::<u32>()
        .map_err(|_| KMatrixError::NumericFailure)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::{Rational, round_ratio_to_integer};
    use num_bigint::BigInt;

    #[test]
    fn integer_rounding_is_nearest_ties_to_even_for_both_signs() {
        assert_eq!(
            round_ratio_to_integer(&BigInt::from(1), &BigInt::from(2)),
            BigInt::from(0)
        );
        assert_eq!(
            round_ratio_to_integer(&BigInt::from(3), &BigInt::from(2)),
            BigInt::from(2)
        );
        assert_eq!(
            round_ratio_to_integer(&BigInt::from(-1), &BigInt::from(2)),
            BigInt::from(0)
        );
        assert_eq!(
            round_ratio_to_integer(&BigInt::from(-3), &BigInt::from(2)),
            BigInt::from(-2)
        );
        assert_eq!(
            round_ratio_to_integer(&BigInt::from(-5), &BigInt::from(2)),
            BigInt::from(-2)
        );
    }

    #[test]
    fn binary32_rounding_preserves_ties_subnormals_and_signed_underflow_zero() {
        let half_ulp_at_one =
            Rational::new(BigInt::from((1_u32 << 24) + 1), BigInt::from(1_u32 << 24)).unwrap();
        let second_half_ulp_at_one =
            Rational::new(BigInt::from((1_u32 << 24) + 3), BigInt::from(1_u32 << 24)).unwrap();
        assert_eq!(half_ulp_at_one.to_f32_word().unwrap(), 0x3f80_0000);
        assert_eq!(second_half_ulp_at_one.to_f32_word().unwrap(), 0x3f80_0002);
        assert_eq!(
            Rational::new(BigInt::from(1), BigInt::from(1_u32) << 149_usize)
                .unwrap()
                .to_f32_word()
                .unwrap(),
            0x0000_0001
        );
        assert_eq!(
            Rational::new(BigInt::from(-1), BigInt::from(1_u32) << 150_usize)
                .unwrap()
                .to_f32_word()
                .unwrap(),
            0x8000_0000
        );
    }

    #[test]
    fn decimal_entry_quantization_is_ties_to_even() {
        let lower_tie = Rational::new(BigInt::from(1), BigInt::from(200_000_000)).unwrap();
        let upper_tie = Rational::new(BigInt::from(3), BigInt::from(200_000_000)).unwrap();
        assert_eq!(
            round_ratio_to_integer(
                &(&lower_tie.numerator * BigInt::from(100_000_000)),
                &lower_tie.denominator
            ),
            BigInt::from(0)
        );
        assert_eq!(
            round_ratio_to_integer(
                &(&upper_tie.numerator * BigInt::from(100_000_000)),
                &upper_tie.denominator
            ),
            BigInt::from(2)
        );
    }
}
