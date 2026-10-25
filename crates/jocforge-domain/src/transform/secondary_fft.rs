// pattern: Functional Core
//! Stateless secondary 64/128-point complex FFT using the selected F06NF network.

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};

const PARAMETER_BYTES: &[u8] = include_bytes!("secondary_fft_parameters.json");
const PARAMETER_SHA256: &str = "b452a929d7e4f7090f9466930e5c6047a8007192a30a3ed7c3cd7447dba073b1";
const PARAMETER_SCHEMA: &str = "secondary-inverse-mathematical-parameters.v1";
const PARAMETER_ORIGIN: &str = "INDEPENDENT_MATH_RULES_VALIDATED_WITHIN_DEFINED_SELECTED_DOMAIN";
const SHARED_WINDOW: &str =
    "F08 mathematical KBD window. Full original-window equality is not claimed.";
const FFT_WORDS_PER_COMPLEX: usize = 2;
const FFT_LANES: usize = 4;
const FFT_TWIDDLE_ROW_WORDS: usize = 256;
const FFT_TWIDDLE_WORDS: usize = 6 * FFT_TWIDDLE_ROW_WORDS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecondaryFftError {
    InputError,
    ParameterUnavailable,
    ParameterError,
    NumericFailure,
    UnsupportedProfile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecondaryFftSize {
    Points64,
    Points128,
}

impl SecondaryFftSize {
    pub fn new(complex_points: u16) -> Result<Self, SecondaryFftError> {
        match complex_points {
            64 => Ok(Self::Points64),
            128 => Ok(Self::Points128),
            _ => Err(SecondaryFftError::UnsupportedProfile),
        }
    }

    pub const fn complex_points(self) -> u16 {
        match self {
            Self::Points64 => 64,
            Self::Points128 => 128,
        }
    }

    fn points_usize(self) -> usize {
        usize::from(self.complex_points())
    }
}

impl SecondaryFftError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputError => "FFT_INPUT_ERROR",
            Self::ParameterUnavailable => "PARAMETER_UNAVAILABLE",
            Self::ParameterError => "FFT_PARAMETER_ERROR",
            Self::NumericFailure => "NUMERIC_FAILURE",
            Self::UnsupportedProfile => "UNSUPPORTED_PROFILE",
        }
    }
}

/// Validated provider fields consumed by the stateless F06NF FFT.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecondaryFftParameters {
    twiddles: [u32; FFT_TWIDDLE_WORDS],
    radix_scale: [u32; FFT_LANES],
    permutation64: [u32; 4],
    permutation128: [u32; 8],
    provider_sha256: [u8; 32],
}

impl SecondaryFftParameters {
    /// Validates the embedded selected mathematical provider before use.
    pub fn selected() -> Result<Self, SecondaryFftError> {
        if sha256_hex(PARAMETER_BYTES) != PARAMETER_SHA256 {
            return Err(SecondaryFftError::ParameterUnavailable);
        }
        let root: Value = serde_json::from_slice(PARAMETER_BYTES)
            .map_err(|_| SecondaryFftError::ParameterError)?;
        let root = root.as_object().ok_or(SecondaryFftError::ParameterError)?;
        if !has_exact_keys(
            root,
            &[
                "schema",
                "origin",
                "internal_only",
                "fields_u32",
                "formula_rules",
                "shared_window",
            ],
        ) || text(root, "schema")? != PARAMETER_SCHEMA
            || text(root, "origin")? != PARAMETER_ORIGIN
            || root.get("internal_only").and_then(Value::as_bool) != Some(true)
            || text(root, "shared_window")? != SHARED_WINDOW
        {
            return Err(SecondaryFftError::ParameterError);
        }

        let formula_rules = root
            .get("formula_rules")
            .and_then(Value::as_object)
            .ok_or(SecondaryFftError::ParameterError)?;
        validate_formula_rules(formula_rules)?;

        let fields = root
            .get("fields_u32")
            .and_then(Value::as_object)
            .ok_or(SecondaryFftError::ParameterError)?;
        if !has_exact_keys(
            fields,
            &[
                "long_pre_a",
                "long_pre_b",
                "long_post",
                "permutation128",
                "short_pre_a",
                "short_pre_b",
                "short_post",
                "permutation64",
                "twiddles",
                "radix_scale",
            ],
        ) {
            return Err(SecondaryFftError::ParameterError);
        }

        // These rotations belong to the surrounding inverse pipeline. This
        // FFT leaf validates their provider fields without consuming them.
        let _long_pre_a = float_words::<128>(fields, "long_pre_a")?;
        let _long_pre_b = float_words::<128>(fields, "long_pre_b")?;
        let _long_post = float_words::<128>(fields, "long_post")?;
        let _short_pre_a = float_words::<64>(fields, "short_pre_a")?;
        let _short_pre_b = float_words::<64>(fields, "short_pre_b")?;
        let _short_post = float_words::<64>(fields, "short_post")?;
        let twiddles = float_words::<FFT_TWIDDLE_WORDS>(fields, "twiddles")?;
        let radix_scale = float_words::<FFT_LANES>(fields, "radix_scale")?;
        if radix_scale.iter().any(|word| *word != radix_scale[0]) {
            return Err(SecondaryFftError::ParameterError);
        }
        let permutation64 = unsigned_words::<4>(fields, "permutation64")?;
        let permutation128 = unsigned_words::<8>(fields, "permutation128")?;
        if permutation64 != selected_permutation::<4>(64)
            || permutation128 != selected_permutation::<8>(128)
        {
            return Err(SecondaryFftError::ParameterError);
        }

        Ok(Self {
            twiddles,
            radix_scale,
            permutation64,
            permutation128,
            provider_sha256: Sha256::digest(PARAMETER_BYTES).into(),
        })
    }

    pub const fn provider_sha256(&self) -> &[u8; 32] {
        &self.provider_sha256
    }
}

/// Executes the selected FFT64 or FFT128 over interleaved finite F32 words.
/// Every operation uses its own F32 rounding step; the function owns no state.
pub fn execute_secondary_fft(
    size: SecondaryFftSize,
    input_words: &[u32],
    parameters: &SecondaryFftParameters,
) -> Result<Vec<u32>, SecondaryFftError> {
    let complex_points = size.points_usize();
    if input_words.len() != complex_points * FFT_WORDS_PER_COMPLEX
        || input_words
            .iter()
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(SecondaryFftError::InputError);
    }

    let initial = initial_stage(complex_points, input_words, parameters)?;
    let intermediate = intermediate_stage(complex_points, &initial, parameters)?;
    final_stage(complex_points, &intermediate, parameters)
}

#[derive(Clone, Copy, Debug, Default)]
struct ComplexVector {
    real: [u32; FFT_LANES],
    imaginary: [u32; FFT_LANES],
}

#[derive(Clone, Copy)]
struct TwiddleVectors {
    c1: [u32; FFT_LANES],
    s1: [u32; FFT_LANES],
    c2: [u32; FFT_LANES],
    s2: [u32; FFT_LANES],
    c3: [u32; FFT_LANES],
    s3: [u32; FFT_LANES],
}

fn initial_stage(
    complex_points: usize,
    input_words: &[u32],
    parameters: &SecondaryFftParameters,
) -> Result<Vec<u32>, SecondaryFftError> {
    let group_width = 16;
    let mut scratch = vec![0_u32; input_words.len()];
    for offset in (0..group_width).step_by(FFT_LANES) {
        if complex_points == 64 {
            let input = std::array::from_fn(|group| {
                load_input_group(input_words, group, group_width, offset)
            });
            let output = four_way(input)?;
            for (block, values) in output.into_iter().enumerate() {
                store_initial_block(&mut scratch, group_width, block, offset, values);
            }
        } else {
            let input: [ComplexVector; 8] = std::array::from_fn(|group| {
                load_input_group(input_words, group, group_width, offset)
            });
            let even = [input[0], input[2], input[4], input[6]];
            let odd = [input[1], input[3], input[5], input[7]];
            let even_output = four_way(even)?;
            let odd_output = four_way(odd)?;
            let mut blocks = [ComplexVector::default(); 8];
            for lane in 0..FFT_LANES {
                let r1 = input[1].real[lane];
                let r3 = input[3].real[lane];
                let r5 = input[5].real[lane];
                let r7 = input[7].real[lane];
                let i1 = input[1].imaginary[lane];
                let i3 = input[3].imaginary[lane];
                let i5 = input[5].imaginary[lane];
                let i7 = input[7].imaginary[lane];
                let real_difference_1_5 = subtract_words(r1, r5)?;
                let real_difference_3_7 = subtract_words(r3, r7)?;
                let imaginary_difference_1_5 = subtract_words(i1, i5)?;
                let imaginary_difference_3_7 = subtract_words(i3, i7)?;
                let scaled_i_plus_s = multiply_words(
                    add_words(imaginary_difference_1_5, real_difference_3_7)?,
                    parameters.radix_scale[lane],
                )?;
                let scaled_i_minus_s = multiply_words(
                    subtract_words(imaginary_difference_1_5, real_difference_3_7)?,
                    parameters.radix_scale[lane],
                )?;
                let scaled_j_plus_r = multiply_words(
                    add_words(imaginary_difference_3_7, real_difference_1_5)?,
                    parameters.radix_scale[lane],
                )?;
                let scaled_r_minus_j = multiply_words(
                    subtract_words(real_difference_1_5, imaginary_difference_3_7)?,
                    parameters.radix_scale[lane],
                )?;
                let combined_b_d = add_words(scaled_i_minus_s, scaled_j_plus_r)?;
                let difference_b_d = subtract_words(scaled_i_minus_s, scaled_j_plus_r)?;
                let difference_e_a = subtract_words(scaled_r_minus_j, scaled_i_plus_s)?;
                let sum_a_e = add_words(scaled_i_plus_s, scaled_r_minus_j)?;

                blocks[0].real[lane] =
                    add_words(odd_output[0].real[lane], even_output[0].real[lane])?;
                blocks[0].imaginary[lane] = add_words(
                    odd_output[0].imaginary[lane],
                    even_output[0].imaginary[lane],
                )?;
                blocks[1].real[lane] =
                    subtract_words(even_output[0].real[lane], odd_output[0].real[lane])?;
                blocks[1].imaginary[lane] = subtract_words(
                    even_output[0].imaginary[lane],
                    odd_output[0].imaginary[lane],
                )?;
                blocks[2].real[lane] =
                    add_words(odd_output[1].imaginary[lane], even_output[1].real[lane])?;
                blocks[2].imaginary[lane] =
                    subtract_words(even_output[1].imaginary[lane], odd_output[1].real[lane])?;
                blocks[3].real[lane] =
                    subtract_words(even_output[1].real[lane], odd_output[1].imaginary[lane])?;
                blocks[3].imaginary[lane] =
                    add_words(odd_output[1].real[lane], even_output[1].imaginary[lane])?;
                blocks[4].real[lane] = add_words(combined_b_d, even_output[2].real[lane])?;
                blocks[4].imaginary[lane] =
                    add_words(difference_b_d, even_output[2].imaginary[lane])?;
                blocks[5].real[lane] = subtract_words(even_output[2].real[lane], combined_b_d)?;
                blocks[5].imaginary[lane] =
                    subtract_words(even_output[2].imaginary[lane], difference_b_d)?;
                blocks[6].real[lane] = subtract_words(even_output[3].real[lane], difference_e_a)?;
                blocks[6].imaginary[lane] =
                    subtract_words(even_output[3].imaginary[lane], sum_a_e)?;
                blocks[7].real[lane] = add_words(difference_e_a, even_output[3].real[lane])?;
                blocks[7].imaginary[lane] = add_words(sum_a_e, even_output[3].imaginary[lane])?;
            }
            for (block, values) in blocks.into_iter().enumerate() {
                store_initial_block(&mut scratch, group_width, block, offset, values);
            }
        }
    }
    Ok(scratch)
}

fn intermediate_stage(
    complex_points: usize,
    before: &[u32],
    parameters: &SecondaryFftParameters,
) -> Result<Vec<u32>, SecondaryFftError> {
    let block_count = complex_points / 16;
    let mut after = before.to_vec();
    for block in 0..block_count {
        let input = load_block(before, block);
        let output = if block == 0 {
            four_way(input)?
        } else {
            four_way_with_twiddles(input, broadcast_twiddles(parameters, block), false)?
        };
        store_block(&mut after, block, output);
    }
    Ok(after)
}

fn final_stage(
    complex_points: usize,
    before: &[u32],
    parameters: &SecondaryFftParameters,
) -> Result<Vec<u32>, SecondaryFftError> {
    let permutation = if complex_points == 64 {
        &parameters.permutation64[..]
    } else {
        &parameters.permutation128[..]
    };
    let mut output = vec![0_u32; complex_points * FFT_WORDS_PER_COMPLEX];
    let mut written = vec![false; output.len()];
    for (block, permutation_base) in permutation.iter().copied().enumerate() {
        let input = load_transposed_block(before, block);
        let twiddles = vector_twiddles(parameters, block);
        let bit_reversed = four_way_with_twiddles(input, twiddles, true)?;
        let natural_order = [0_usize, 2, 1, 3];
        let lane_order = [0_usize, 2, 1, 3];
        for band in 0..FFT_LANES {
            let values = bit_reversed[natural_order[band]];
            for (output_lane, lane) in lane_order.into_iter().enumerate() {
                let destination = usize::try_from(permutation_base)
                    .map_err(|_| SecondaryFftError::ParameterError)?
                    + band * (complex_points / 2)
                    + output_lane * (complex_points / 8);
                let imaginary = destination + 1;
                if destination % 2 != 0
                    || imaginary >= output.len()
                    || written[destination]
                    || written[imaginary]
                {
                    return Err(SecondaryFftError::ParameterError);
                }
                output[destination] = values.real[lane];
                output[imaginary] = values.imaginary[lane];
                written[destination] = true;
                written[imaginary] = true;
            }
        }
    }
    if written.iter().any(|was_written| !was_written) {
        return Err(SecondaryFftError::ParameterError);
    }
    Ok(output)
}

fn four_way(
    input: [ComplexVector; FFT_LANES],
) -> Result<[ComplexVector; FFT_LANES], SecondaryFftError> {
    let [r0, r1, r2, r3] = input;
    let mut output = [ComplexVector::default(); FFT_LANES];
    for lane in 0..FFT_LANES {
        let real_sum_even = add_words(r0.real[lane], r2.real[lane])?;
        let real_sum_odd = add_words(r1.real[lane], r3.real[lane])?;
        let imaginary_sum_even = add_words(r0.imaginary[lane], r2.imaginary[lane])?;
        let imaginary_sum_odd = add_words(r1.imaginary[lane], r3.imaginary[lane])?;
        let real_difference_even = subtract_words(r0.real[lane], r2.real[lane])?;
        let real_difference_odd = subtract_words(r1.real[lane], r3.real[lane])?;
        let imaginary_difference_even = subtract_words(r0.imaginary[lane], r2.imaginary[lane])?;
        let imaginary_difference_odd = subtract_words(r1.imaginary[lane], r3.imaginary[lane])?;
        output[0].real[lane] = add_words(real_sum_odd, real_sum_even)?;
        output[0].imaginary[lane] = add_words(imaginary_sum_odd, imaginary_sum_even)?;
        output[1].real[lane] = subtract_words(real_sum_even, real_sum_odd)?;
        output[1].imaginary[lane] = subtract_words(imaginary_sum_even, imaginary_sum_odd)?;
        output[2].real[lane] = add_words(imaginary_difference_odd, real_difference_even)?;
        output[2].imaginary[lane] = subtract_words(imaginary_difference_even, real_difference_odd)?;
        output[3].real[lane] = subtract_words(real_difference_even, imaginary_difference_odd)?;
        output[3].imaginary[lane] = add_words(imaginary_difference_even, real_difference_odd)?;
    }
    Ok(output)
}

fn four_way_with_twiddles(
    input: [ComplexVector; FFT_LANES],
    twiddles: TwiddleVectors,
    final_stage: bool,
) -> Result<[ComplexVector; FFT_LANES], SecondaryFftError> {
    let [r0, r1, r2, r3] = input;
    let mut output = [ComplexVector::default(); FFT_LANES];
    for lane in 0..FFT_LANES {
        let twiddled_real_2 = subtract_words(
            multiply_words(twiddles.c2[lane], r2.real[lane])?,
            multiply_words(twiddles.s2[lane], r2.imaginary[lane])?,
        )?;
        let twiddled_imaginary_2 = add_words(
            multiply_words(twiddles.c2[lane], r2.imaginary[lane])?,
            multiply_words(twiddles.s2[lane], r2.real[lane])?,
        )?;
        let twiddled_real_1 = subtract_words(
            multiply_words(twiddles.c1[lane], r1.real[lane])?,
            multiply_words(twiddles.s1[lane], r1.imaginary[lane])?,
        )?;
        let twiddled_imaginary_1 = add_words(
            multiply_words(twiddles.c1[lane], r1.imaginary[lane])?,
            multiply_words(twiddles.s1[lane], r1.real[lane])?,
        )?;
        let twiddled_real_3 = subtract_words(
            multiply_words(twiddles.c3[lane], r3.real[lane])?,
            multiply_words(twiddles.s3[lane], r3.imaginary[lane])?,
        )?;
        let twiddled_imaginary_3 = if final_stage {
            add_words(
                multiply_words(twiddles.s3[lane], r3.real[lane])?,
                multiply_words(twiddles.c3[lane], r3.imaginary[lane])?,
            )?
        } else {
            add_words(
                multiply_words(twiddles.c3[lane], r3.imaginary[lane])?,
                multiply_words(twiddles.s3[lane], r3.real[lane])?,
            )?
        };
        let even_real_sum = add_words(twiddled_real_2, r0.real[lane])?;
        let even_imaginary_sum = add_words(twiddled_imaginary_2, r0.imaginary[lane])?;
        let even_real_difference = subtract_words(r0.real[lane], twiddled_real_2)?;
        let even_imaginary_difference = subtract_words(r0.imaginary[lane], twiddled_imaginary_2)?;
        let odd_real_sum = add_words(twiddled_real_3, twiddled_real_1)?;
        let odd_imaginary_sum = add_words(twiddled_imaginary_3, twiddled_imaginary_1)?;
        let odd_real_difference = subtract_words(twiddled_real_1, twiddled_real_3)?;
        let odd_imaginary_difference = subtract_words(twiddled_imaginary_1, twiddled_imaginary_3)?;
        output[0].real[lane] = add_words(odd_real_sum, even_real_sum)?;
        output[0].imaginary[lane] = add_words(odd_imaginary_sum, even_imaginary_sum)?;
        output[1].real[lane] = subtract_words(even_real_sum, odd_real_sum)?;
        output[1].imaginary[lane] = subtract_words(even_imaginary_sum, odd_imaginary_sum)?;
        output[2].real[lane] = add_words(odd_imaginary_difference, even_real_difference)?;
        output[2].imaginary[lane] = subtract_words(even_imaginary_difference, odd_real_difference)?;
        output[3].real[lane] = subtract_words(even_real_difference, odd_imaginary_difference)?;
        output[3].imaginary[lane] = add_words(odd_real_difference, even_imaginary_difference)?;
    }
    Ok(output)
}

fn load_input_group(
    input: &[u32],
    group: usize,
    group_width: usize,
    offset: usize,
) -> ComplexVector {
    let mut output = ComplexVector::default();
    for lane in 0..FFT_LANES {
        let source = FFT_WORDS_PER_COMPLEX * (group * group_width + offset + lane);
        output.real[lane] = input[source];
        output.imaginary[lane] = input[source + 1];
    }
    output
}

fn store_initial_block(
    output: &mut [u32],
    group_width: usize,
    block: usize,
    offset: usize,
    values: ComplexVector,
) {
    let destination = 2 * group_width * block + 2 * offset;
    output[destination..destination + FFT_LANES].copy_from_slice(&values.real);
    output[destination + FFT_LANES..destination + 2 * FFT_LANES].copy_from_slice(&values.imaginary);
}

fn load_block(words: &[u32], block: usize) -> [ComplexVector; FFT_LANES] {
    let mut output = [ComplexVector::default(); FFT_LANES];
    let base = block * 8 * FFT_LANES;
    for (group, group_output) in output.iter_mut().enumerate() {
        let offset = base + group * 2 * FFT_LANES;
        group_output
            .real
            .copy_from_slice(&words[offset..offset + FFT_LANES]);
        group_output
            .imaginary
            .copy_from_slice(&words[offset + FFT_LANES..offset + 2 * FFT_LANES]);
    }
    output
}

fn store_block(words: &mut [u32], block: usize, values: [ComplexVector; FFT_LANES]) {
    let base = block * 8 * FFT_LANES;
    for (group, group_values) in values.iter().enumerate() {
        let offset = base + group * 2 * FFT_LANES;
        words[offset..offset + FFT_LANES].copy_from_slice(&group_values.real);
        words[offset + FFT_LANES..offset + 2 * FFT_LANES].copy_from_slice(&group_values.imaginary);
    }
}

fn load_transposed_block(words: &[u32], block: usize) -> [ComplexVector; FFT_LANES] {
    let base = block * 8 * FFT_LANES;
    let mut output = [ComplexVector::default(); FFT_LANES];
    for transform_input in 0..FFT_LANES {
        for lane in 0..FFT_LANES {
            output[transform_input].real[lane] =
                words[base + 2 * FFT_LANES * lane + transform_input];
            output[transform_input].imaginary[lane] =
                words[base + 2 * FFT_LANES * lane + FFT_LANES + transform_input];
        }
    }
    output
}

fn broadcast_twiddles(parameters: &SecondaryFftParameters, index: usize) -> TwiddleVectors {
    TwiddleVectors {
        c1: [parameters.twiddles[index]; FFT_LANES],
        s1: [parameters.twiddles[FFT_TWIDDLE_ROW_WORDS + index]; FFT_LANES],
        c2: [parameters.twiddles[2 * FFT_TWIDDLE_ROW_WORDS + index]; FFT_LANES],
        s2: [parameters.twiddles[3 * FFT_TWIDDLE_ROW_WORDS + index]; FFT_LANES],
        c3: [parameters.twiddles[4 * FFT_TWIDDLE_ROW_WORDS + index]; FFT_LANES],
        s3: [parameters.twiddles[5 * FFT_TWIDDLE_ROW_WORDS + index]; FFT_LANES],
    }
}

fn vector_twiddles(parameters: &SecondaryFftParameters, group: usize) -> TwiddleVectors {
    let start = 4 * group;
    TwiddleVectors {
        c1: std::array::from_fn(|lane| parameters.twiddles[start + lane]),
        s1: std::array::from_fn(|lane| parameters.twiddles[FFT_TWIDDLE_ROW_WORDS + start + lane]),
        c2: std::array::from_fn(|lane| {
            parameters.twiddles[2 * FFT_TWIDDLE_ROW_WORDS + start + lane]
        }),
        s2: std::array::from_fn(|lane| {
            parameters.twiddles[3 * FFT_TWIDDLE_ROW_WORDS + start + lane]
        }),
        c3: std::array::from_fn(|lane| {
            parameters.twiddles[4 * FFT_TWIDDLE_ROW_WORDS + start + lane]
        }),
        s3: std::array::from_fn(|lane| {
            parameters.twiddles[5 * FFT_TWIDDLE_ROW_WORDS + start + lane]
        }),
    }
}

fn float_words<const N: usize>(
    fields: &Map<String, Value>,
    name: &str,
) -> Result<[u32; N], SecondaryFftError> {
    let values = fields
        .get(name)
        .and_then(Value::as_array)
        .ok_or(SecondaryFftError::ParameterError)?;
    if values.len() != N {
        return Err(SecondaryFftError::ParameterError);
    }
    let mut output = [0_u32; N];
    for (index, value) in values.iter().enumerate() {
        let word = value
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(SecondaryFftError::ParameterError)?;
        if !f32::from_bits(word).is_finite() {
            return Err(SecondaryFftError::ParameterError);
        }
        output[index] = word;
    }
    Ok(output)
}

fn unsigned_words<const N: usize>(
    fields: &Map<String, Value>,
    name: &str,
) -> Result<[u32; N], SecondaryFftError> {
    let values = fields
        .get(name)
        .and_then(Value::as_array)
        .ok_or(SecondaryFftError::ParameterError)?;
    if values.len() != N {
        return Err(SecondaryFftError::ParameterError);
    }
    let mut output = [0_u32; N];
    for (index, value) in values.iter().enumerate() {
        output[index] = value
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(SecondaryFftError::ParameterError)?;
    }
    Ok(output)
}

fn validate_formula_rules(rules: &Map<String, Value>) -> Result<(), SecondaryFftError> {
    let expected = [
        ("pre_a", "R32(sin((j+1/4)*pi/(2N))/2)"),
        ("pre_b", "R32(cos((j+1/4)*pi/(2N))/2)"),
        ("post", "R32(sin(j*pi/(2N)))"),
        (
            "twiddles",
            "rows 2*(m-1) and 2*(m-1)+1 = R32(cos(theta)), R32(-sin(theta)); theta=2*pi*((m*reverse8(j)) mod1024)/1024; m1..3,j0..255; cardinal components exact positive zero",
        ),
        ("scale", "four identical R32(sqrt(1/2))"),
        ("permutation", "2*reverse_bits(g,log2(N/16)), g0..N/16-1"),
    ];
    let expected_keys = ["pre_a", "pre_b", "post", "twiddles", "scale", "permutation"];
    if !has_exact_keys(rules, &expected_keys)
        || expected
            .iter()
            .any(|(name, value)| rules.get(*name).and_then(Value::as_str) != Some(*value))
    {
        return Err(SecondaryFftError::ParameterError);
    }
    Ok(())
}

fn has_exact_keys(object: &Map<String, Value>, expected: &[&str]) -> bool {
    object.len() == expected.len() && expected.iter().all(|key| object.contains_key(*key))
}

fn text<'a>(object: &'a Map<String, Value>, name: &str) -> Result<&'a str, SecondaryFftError> {
    object
        .get(name)
        .and_then(Value::as_str)
        .ok_or(SecondaryFftError::ParameterError)
}

fn selected_permutation<const N: usize>(complex_points: usize) -> [u32; N] {
    let groups = complex_points / 16;
    let bits = groups.trailing_zeros() as usize;
    std::array::from_fn(|group| (2 * reverse_low_bits(group, bits)) as u32)
}

fn reverse_low_bits(value: usize, bits: usize) -> usize {
    let mut source = value;
    let mut reversed = 0;
    for _ in 0..bits {
        reversed = (reversed << 1) | (source & 1);
        source >>= 1;
    }
    reversed
}

fn add_words(left: u32, right: u32) -> Result<u32, SecondaryFftError> {
    let left = finite_word(left)?;
    let right = finite_word(right)?;
    r32_add(left, right)
        .map(|value| value.value().to_bits())
        .map_err(|_| SecondaryFftError::NumericFailure)
}

fn subtract_words(left: u32, right: u32) -> Result<u32, SecondaryFftError> {
    let left = finite_word(left)?;
    let right = finite_word(right)?;
    r32_sub(left, right)
        .map(|value| value.value().to_bits())
        .map_err(|_| SecondaryFftError::NumericFailure)
}

fn multiply_words(left: u32, right: u32) -> Result<u32, SecondaryFftError> {
    let left = finite_word(left)?;
    let right = finite_word(right)?;
    r32_mul(left, right)
        .map(|value| value.value().to_bits())
        .map_err(|_| SecondaryFftError::NumericFailure)
}

fn finite_word(word: u32) -> Result<FiniteF32, SecondaryFftError> {
    FiniteF32::new(f32::from_bits(word)).map_err(|_| SecondaryFftError::NumericFailure)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}
