// pattern: Functional Core
//! Selected 5.1 long-block LFE transform, dedicated FFT8 recurrence, and raw wrapper.

use super::fbw_transform::{F08FbwTransformError, long_pre_rotation};
use super::long_transform::{
    FoldPreRotation, LongTransformHistory, TransformBranch, add, fold_and_pre_rotation, multiply,
    subtract,
};
use super::math_generated_parameters::MathGeneratedTransformParameters;
use super::parameters::{COSINE_F32_BITS, SINE_F32_BITS};
use super::window_fold::F08WindowFoldOutput;
use crate::codec::lfe_coefficient_exponent::{
    LFE_COEFFICIENT_BINS, LfeCoefficientBlock, LfeExponentOutputBlock, lfe_coefficient_pair_to_raw,
};
use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};

const RECURRENCE_COSINE_F32_BITS: [u32; 8] = [
    1_065_353_216,
    1_065_333_007,
    1_065_272_429,
    1_065_171_628,
    1_065_030_846,
    1_065_171_628,
    1_065_272_429,
    1_065_333_007,
];
const RECURRENCE_SINE_F32_BITS: [u32; 8] = [
    0,
    1_028_193_072,
    1_036_565_814,
    1_041_645_699,
    3_192_374_722,
    3_189_129_347,
    3_184_049_462,
    3_175_676_720,
];
const RECURRENCE_MULTIPLIER_F32_BITS: [u32; 8] = [
    1_073_741_824,
    1_073_721_615,
    1_073_661_037,
    1_073_560_236,
    1_073_419_454,
    1_073_560_236,
    1_073_661_037,
    1_073_721_615,
];
const GATHER_PERMUTATION: [usize; 8] = [0, 4, 2, 6, 1, 5, 3, 7];
const ROOT_HALF_F32_BITS: u32 = 1_060_439_283;

/// Result of one branch's fold/pre/FFT8/post stages and history transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LfeTransformBranchOutput {
    folded_bits: [u32; 256],
    pre_bits: [u32; 256],
    low_bits: [u32; 16],
    spectrum_bits: [u32; LFE_COEFFICIENT_BINS],
    wrapper_bits: [u32; LFE_COEFFICIENT_BINS],
    history_after: LongTransformHistory,
}

impl LfeTransformBranchOutput {
    pub const fn folded_bits(&self) -> &[u32; 256] {
        &self.folded_bits
    }

    pub const fn pre_bits(&self) -> &[u32; 256] {
        &self.pre_bits
    }

    pub const fn low_bits(&self) -> &[u32; 16] {
        &self.low_bits
    }

    pub const fn spectrum_bits(&self) -> &[u32; LFE_COEFFICIENT_BINS] {
        &self.spectrum_bits
    }

    pub const fn wrapper_bits(&self) -> &[u32; LFE_COEFFICIENT_BINS] {
        &self.wrapper_bits
    }

    pub const fn history_after(&self) -> &LongTransformHistory {
        &self.history_after
    }
}

/// Both transform branches from one synthetic block, with caller-owned histories returned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LfeTransformToRawOutput {
    primary: LfeTransformBranchOutput,
    companion: LfeTransformBranchOutput,
}

impl LfeTransformToRawOutput {
    pub const fn primary(&self) -> &LfeTransformBranchOutput {
        &self.primary
    }

    pub const fn companion(&self) -> &LfeTransformBranchOutput {
        &self.companion
    }
}

/// Runs one selected long-block LFE transform call and writes its seven raw exponents.
///
/// The caller supplies one finite 256-word input, independent primary/companion histories,
/// and an eight-entry output whose last slot is caller-owned. The path shares only the
/// approved fold/pre prefix with full-band transform code; it uses its own 16-gather FFT8,
/// recurrence and seven-bin post-rotation. It does not select `short_flag`, produce source
/// samples, or establish history cold/reset/clock ownership.
///
/// # Panics
///
/// Panics if the caller supplies values outside the washed finite synthetic profile or an
/// intermediate becomes non-finite within that bounded computation.
pub fn lfe_transform_to_raw(
    input_bits: &[u32; 256],
    primary_history: &LongTransformHistory,
    companion_history: &LongTransformHistory,
    raw_output: &mut LfeExponentOutputBlock,
) -> LfeTransformToRawOutput {
    let primary = lfe_transform_branch(input_bits, primary_history, TransformBranch::Primary);
    let companion = lfe_transform_branch(input_bits, companion_history, TransformBranch::Companion);
    let real: LfeCoefficientBlock = primary.spectrum_bits.map(f32::from_bits);
    let imaginary: LfeCoefficientBlock = companion.spectrum_bits.map(f32::from_bits);
    lfe_coefficient_pair_to_raw(&real, &imaginary, raw_output);

    LfeTransformToRawOutput { primary, companion }
}

fn lfe_transform_branch(
    input_bits: &[u32; 256],
    history: &LongTransformHistory,
    branch: TransformBranch,
) -> LfeTransformBranchOutput {
    let FoldPreRotation {
        folded,
        pre_rotated,
        history_after,
    } = fold_and_pre_rotation(input_bits, history, branch);
    let mut gathered = [[0.0_f32; 16]; 16];

    for group in 0..16 {
        for pair in 0..8 {
            gathered[group][2 * pair] = pre_rotated[2 * group + 32 * pair];
            gathered[group][2 * pair + 1] = pre_rotated[2 * group + 32 * pair + 1];
        }
        fft8(&mut gathered[group]);
    }

    let low = recurrence(&gathered);
    let spectrum = post_rotate_lfe(&low, branch);
    let wrapper_bits = spectrum.map(f32::to_bits);

    LfeTransformBranchOutput {
        folded_bits: folded.map(f32::to_bits),
        pre_bits: pre_rotated.map(f32::to_bits),
        low_bits: low.map(f32::to_bits),
        spectrum_bits: wrapper_bits,
        wrapper_bits,
        history_after,
    }
}

fn fft8(values: &mut [f32; 16]) {
    let mut vectors = [[0.0_f32; 4]; 4];
    for (vector_index, vector) in vectors.iter_mut().enumerate() {
        vector.copy_from_slice(&values[vector_index * 4..vector_index * 4 + 4]);
    }

    {
        let [v0, v1, v2, v3] = &mut vectors;
        for (v0_component, v2_component) in v0.iter_mut().zip(v2.iter_mut()) {
            let old_v0 = *v0_component;
            let old_v2 = *v2_component;
            *v0_component = add(old_v2, old_v0);
            *v2_component = subtract(old_v0, old_v2);
        }
        for (v1_component, v3_component) in v1.iter_mut().zip(v3.iter_mut()) {
            let old_v1 = *v1_component;
            let old_v3 = *v3_component;
            *v1_component = add(old_v3, old_v1);
            *v3_component = subtract(old_v1, old_v3);
        }
    }

    let old_v3 = vectors[3];
    vectors[3] = [-old_v3[1], old_v3[0], -old_v3[3], old_v3[2]];

    {
        let [v0, v1, _, _] = &mut vectors;
        for (v0_component, v1_component) in v0.iter_mut().zip(v1.iter_mut()) {
            let old_v0 = *v0_component;
            let old_v1 = *v1_component;
            *v0_component = add(old_v1, old_v0);
            *v1_component = subtract(old_v0, old_v1);
        }
    }
    let old_v1_real = vectors[1][2];
    let old_v1_imaginary = vectors[1][3];
    vectors[1][2] = -old_v1_imaginary;
    vectors[1][3] = old_v1_real;

    let old_v2 = vectors[2];
    let old_v3 = vectors[3];
    for component in 0..4 {
        vectors[2][component] = subtract(old_v2[component], old_v3[component]);
        vectors[3][component] = add(old_v3[component], old_v2[component]);
    }

    let root_half = f32::from_bits(ROOT_HALF_F32_BITS);
    for vector_index in [2, 3] {
        let a = multiply(root_half, vectors[vector_index][2]);
        let b = multiply(root_half, vectors[vector_index][3]);
        vectors[vector_index][2] = add(a, b);
        vectors[vector_index][3] = subtract(b, a);
        if vector_index == 3 {
            vectors[3][2] = subtract(b, a);
            vectors[3][3] = -add(a, b);
        }
    }

    for vector in &mut vectors {
        let [ar, ai, br, bi] = *vector;
        vector[0] = multiply(add(ar, br), 0.125);
        vector[1] = multiply(add(ai, bi), 0.125);
        vector[2] = multiply(subtract(ar, br), 0.125);
        vector[3] = multiply(subtract(ai, bi), 0.125);
    }

    vectors[1].swap(0, 2);
    vectors[1].swap(1, 3);
    for (vector_index, vector) in vectors.into_iter().enumerate() {
        values[vector_index * 4..vector_index * 4 + 4].copy_from_slice(&vector);
    }
}

fn recurrence(gathered: &[[f32; 16]; 16]) -> [f32; 16] {
    let mut low = [0.0_f32; 16];

    for k in 0..8 {
        let j = GATHER_PERMUTATION[k];
        let mut previous = (0.0_f32, 0.0_f32);
        let mut current = (gathered[15][2 * j], gathered[15][2 * j + 1]);
        let multiplier = f32::from_bits(RECURRENCE_MULTIPLIER_F32_BITS[k]);

        for group in (0..15).rev() {
            let old_current = current;
            current = (
                add(
                    subtract(multiply(multiplier, old_current.0), previous.0),
                    gathered[group][2 * j],
                ),
                add(
                    subtract(multiply(multiplier, old_current.1), previous.1),
                    gathered[group][2 * j + 1],
                ),
            );
            previous = old_current;
        }

        let cosine = f32::from_bits(RECURRENCE_COSINE_F32_BITS[k]);
        let sine = f32::from_bits(RECURRENCE_SINE_F32_BITS[k]);
        let real_cosine = multiply(previous.0, cosine);
        let real_sine = multiply(previous.1, sine);
        let imaginary_sine = multiply(previous.0, sine);
        let imaginary_cosine = multiply(previous.1, cosine);
        let a = add(subtract(current.0, real_cosine), real_sine);
        let b = subtract(subtract(current.1, imaginary_sine), imaginary_cosine);
        low[2 * k] = multiply(a, 0.0625);
        low[2 * k + 1] = multiply(b, 0.0625);
    }

    low
}

fn post_rotate_lfe(low: &[f32; 16], branch: TransformBranch) -> [f32; LFE_COEFFICIENT_BINS] {
    let mut output = [0.0_f32; LFE_COEFFICIENT_BINS];
    for index in 0..4 {
        let cosine = f32::from_bits(COSINE_F32_BITS[index]);
        let sine = f32::from_bits(SINE_F32_BITS[index]);
        let cosine_imaginary = multiply(cosine, low[2 * index + 1]);
        let sine_real = multiply(sine, low[2 * index]);
        output[2 * index] = match branch {
            TransformBranch::Primary => subtract(cosine_imaginary, sine_real),
            TransformBranch::Companion => subtract(sine_real, cosine_imaginary),
        };
    }

    for (target, source, rotation) in [(5, 5, 125), (3, 6, 126), (1, 7, 127)] {
        let cosine = f32::from_bits(COSINE_F32_BITS[rotation]);
        let sine = f32::from_bits(SINE_F32_BITS[rotation]);
        output[target] = add(
            multiply(cosine, low[2 * source]),
            multiply(sine, low[2 * source + 1]),
        );
    }

    output
}

#[cfg(test)]
mod tests {
    use super::{
        GATHER_PERMUTATION, RECURRENCE_COSINE_F32_BITS, RECURRENCE_MULTIPLIER_F32_BITS,
        RECURRENCE_SINE_F32_BITS, ROOT_HALF_F32_BITS,
    };
    use crate::transform::parameters::{COSINE_F32_BITS, SINE_F32_BITS, WINDOW_F32_BITS};
    use serde_json::Value;
    use std::path::PathBuf;

    fn parameters() -> Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/lfe_transform_to_raw/PARAMETERS.json");
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
    }

    fn assert_u32_array(actual: &[u32], expected: &Value) {
        let expected = expected
            .as_array()
            .unwrap()
            .iter()
            .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected.as_slice());
    }

    #[test]
    fn selected_low_transform_tables_match_the_hash_bound_parameter_file() {
        let parameters = parameters();
        assert_u32_array(&WINDOW_F32_BITS, &parameters["window_f32_bits"]);
        assert_u32_array(&COSINE_F32_BITS, &parameters["long_cosine_f32_bits"]);
        assert_u32_array(&SINE_F32_BITS, &parameters["long_sine_f32_bits"]);
        assert_u32_array(
            &RECURRENCE_COSINE_F32_BITS,
            &parameters["recurrence_cosine_f32_bits"],
        );
        assert_u32_array(
            &RECURRENCE_SINE_F32_BITS,
            &parameters["recurrence_sine_f32_bits"],
        );
        assert_u32_array(
            &RECURRENCE_MULTIPLIER_F32_BITS,
            &parameters["recurrence_multiplier_f32_bits"],
        );
        let expected_gather: [usize; 8] = parameters["gather_permutation_u32"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| usize::try_from(value.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        assert_eq!(GATHER_PERMUTATION, expected_gather);
        assert_eq!(
            ROOT_HALF_F32_BITS,
            u32::try_from(parameters["root_half_f32_bits"].as_u64().unwrap()).unwrap()
        );
    }
}

const F08_LFE_SPECTRUM_WORDS: usize = 7;
const F08_LFE_FFT_WORDS: usize = 16;
const POSITIVE_ZERO_F32: f32 = f32::from_bits(0x0000_0000);
const EIGHTH: f32 = 0.125;
const SIXTEENTH: f32 = 0.0625;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum F08LfeTransformError {
    NumericFailure,
}

impl F08LfeTransformError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::NumericFailure => "F08_NUMERIC_FAILURE",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct F08LfeTransformOutput {
    primary_spectrum_bits: [u32; F08_LFE_SPECTRUM_WORDS],
    companion_spectrum_bits: [u32; F08_LFE_SPECTRUM_WORDS],
}

impl F08LfeTransformOutput {
    pub const fn primary_spectrum_bits(&self) -> &[u32; F08_LFE_SPECTRUM_WORDS] {
        &self.primary_spectrum_bits
    }

    pub const fn companion_spectrum_bits(&self) -> &[u32; F08_LFE_SPECTRUM_WORDS] {
        &self.companion_spectrum_bits
    }
}

/// Functional Core: applies the selected math-generated F08.7-F08.9 LFE route.
///
/// The caller supplies the already completed two-bank F08.2 fold and validated provider.
/// This operation returns seven primary and seven companion spectrum words and owns no
/// source clock, transform history, exponent state, or parent commit.
pub fn f08_lfe_transform_block(
    folded: &F08WindowFoldOutput,
    parameters: &MathGeneratedTransformParameters,
) -> Result<F08LfeTransformOutput, F08LfeTransformError> {
    let primary_spectrum_bits =
        f08_lfe_spectrum(folded.primary_fold_bits(), F08LfeBank::Primary, parameters)?;
    let companion_spectrum_bits = f08_lfe_spectrum(
        folded.companion_fold_bits(),
        F08LfeBank::Companion,
        parameters,
    )?;
    Ok(F08LfeTransformOutput {
        primary_spectrum_bits,
        companion_spectrum_bits,
    })
}

fn f08_lfe_spectrum(
    folded_bits: &[u32; 256],
    bank: F08LfeBank,
    parameters: &MathGeneratedTransformParameters,
) -> Result<[u32; F08_LFE_SPECTRUM_WORDS], F08LfeTransformError> {
    let pre_rotated = long_pre_rotation(folded_bits, parameters).map_err(map_f08_lfe_pre_error)?;
    let mut groups = [[POSITIVE_ZERO_F32; F08_LFE_FFT_WORDS]; 16];
    for (group_index, group_values) in groups.iter_mut().enumerate() {
        for frequency in 0..8 {
            let complex_index = group_index + 16 * frequency;
            group_values[2 * frequency] = pre_rotated[2 * complex_index];
            group_values[2 * frequency + 1] = pre_rotated[2 * complex_index + 1];
        }
        f08_lfe_fft8(group_values, parameters)?;
    }

    let low_frequency = f08_lfe_recurrence(&groups, parameters)?;
    f08_lfe_post_rotation(&low_frequency, bank, parameters)
}

#[derive(Clone, Copy)]
enum F08LfeBank {
    Primary,
    Companion,
}

fn f08_lfe_fft8(
    values: &mut [f32; F08_LFE_FFT_WORDS],
    parameters: &MathGeneratedTransformParameters,
) -> Result<(), F08LfeTransformError> {
    let mut vectors = [[POSITIVE_ZERO_F32; 4]; 4];
    for (vector_index, vector) in vectors.iter_mut().enumerate() {
        vector.copy_from_slice(&values[vector_index * 4..vector_index * 4 + 4]);
    }

    for (left, right) in [(0, 2), (1, 3)] {
        let old_left = vectors[left];
        let old_right = vectors[right];
        for component in 0..4 {
            vectors[left][component] = f08_lfe_add(old_right[component], old_left[component])?;
            vectors[right][component] =
                f08_lfe_subtract(old_left[component], old_right[component])?;
        }
    }

    let old_v3 = vectors[3];
    vectors[3] = [
        f08_lfe_negate(old_v3[1]),
        old_v3[0],
        f08_lfe_negate(old_v3[3]),
        old_v3[2],
    ];

    let old_v0 = vectors[0];
    let old_v1 = vectors[1];
    for component in 0..4 {
        vectors[0][component] = f08_lfe_add(old_v1[component], old_v0[component])?;
        vectors[1][component] = f08_lfe_subtract(old_v0[component], old_v1[component])?;
    }

    let old_v1 = vectors[1];
    vectors[1][2] = f08_lfe_negate(old_v1[3]);
    vectors[1][3] = old_v1[2];

    let old_v2 = vectors[2];
    let old_v3 = vectors[3];
    for component in 0..4 {
        vectors[2][component] = f08_lfe_subtract(old_v2[component], old_v3[component])?;
        vectors[3][component] = f08_lfe_add(old_v3[component], old_v2[component])?;
    }

    let rotation = f32::from_bits(parameters.butterfly_f32_bits()[2]);
    let old_v2 = vectors[2];
    let a = f08_lfe_multiply(rotation, old_v2[2])?;
    let b = f08_lfe_multiply(rotation, old_v2[3])?;
    vectors[2][2] = f08_lfe_add(a, b)?;
    vectors[2][3] = f08_lfe_subtract(b, a)?;

    let old_v3 = vectors[3];
    let a = f08_lfe_multiply(rotation, old_v3[2])?;
    let b = f08_lfe_multiply(rotation, old_v3[3])?;
    vectors[3][2] = f08_lfe_subtract(b, a)?;
    vectors[3][3] = f08_lfe_subtract(f08_lfe_negate(a), b)?;

    for vector in &mut vectors {
        let old = *vector;
        vector[0] = f08_lfe_multiply(f08_lfe_add(old[0], old[2])?, EIGHTH)?;
        vector[1] = f08_lfe_multiply(f08_lfe_add(old[1], old[3])?, EIGHTH)?;
        vector[2] = f08_lfe_multiply(f08_lfe_subtract(old[0], old[2])?, EIGHTH)?;
        vector[3] = f08_lfe_multiply(f08_lfe_subtract(old[1], old[3])?, EIGHTH)?;
    }

    let old_v1 = vectors[1];
    vectors[1] = [old_v1[2], old_v1[3], old_v1[0], old_v1[1]];
    for (vector_index, vector) in vectors.into_iter().enumerate() {
        values[vector_index * 4..vector_index * 4 + 4].copy_from_slice(&vector);
    }
    Ok(())
}

fn f08_lfe_recurrence(
    groups: &[[f32; F08_LFE_FFT_WORDS]; 16],
    parameters: &MathGeneratedTransformParameters,
) -> Result<[f32; F08_LFE_FFT_WORDS], F08LfeTransformError> {
    let mut low_frequency = [POSITIVE_ZERO_F32; F08_LFE_FFT_WORDS];
    for frequency in 0..8 {
        let index = usize::try_from(parameters.lfe_permutation()[frequency])
            .map_err(|_| F08LfeTransformError::NumericFailure)?;
        let recurrence = f32::from_bits(parameters.lfe_recurrence_f32_bits()[frequency]);
        let mut prior = [POSITIVE_ZERO_F32; 2];
        let mut current = [groups[15][2 * index], groups[15][2 * index + 1]];

        for group in (0..15).rev() {
            let old_current = current;
            for part in 0..2 {
                let scaled = f08_lfe_multiply(recurrence, old_current[part])?;
                let difference = f08_lfe_subtract(scaled, prior[part])?;
                current[part] = f08_lfe_add(difference, groups[group][2 * index + part])?;
            }
            prior = old_current;
        }

        let cosine = f32::from_bits(parameters.lfe_cosine_f32_bits()[frequency]);
        let sine = f32::from_bits(parameters.lfe_sine_f32_bits()[frequency]);
        let real_prior_cosine = f08_lfe_multiply(prior[0], cosine)?;
        let real_prior_sine = f08_lfe_multiply(prior[1], sine)?;
        let real = f08_lfe_multiply(
            f08_lfe_add(
                f08_lfe_subtract(current[0], real_prior_cosine)?,
                real_prior_sine,
            )?,
            SIXTEENTH,
        )?;

        let imaginary_prior_sine = f08_lfe_multiply(prior[0], sine)?;
        let imaginary_prior_cosine = f08_lfe_multiply(prior[1], cosine)?;
        let imaginary = f08_lfe_multiply(
            f08_lfe_subtract(
                f08_lfe_subtract(current[1], imaginary_prior_sine)?,
                imaginary_prior_cosine,
            )?,
            SIXTEENTH,
        )?;
        low_frequency[2 * frequency] = real;
        low_frequency[2 * frequency + 1] = imaginary;
    }
    Ok(low_frequency)
}

fn f08_lfe_post_rotation(
    low_frequency: &[f32; F08_LFE_FFT_WORDS],
    bank: F08LfeBank,
    parameters: &MathGeneratedTransformParameters,
) -> Result<[u32; F08_LFE_SPECTRUM_WORDS], F08LfeTransformError> {
    let mut spectrum = [POSITIVE_ZERO_F32; F08_LFE_SPECTRUM_WORDS];
    for frequency in 0..4 {
        let cosine = f32::from_bits(parameters.long_cosine_f32_bits()[frequency]);
        let sine = f32::from_bits(parameters.long_sine_f32_bits()[frequency]);
        let real = low_frequency[2 * frequency];
        let imaginary = low_frequency[2 * frequency + 1];
        let cosine_imaginary = f08_lfe_multiply(cosine, imaginary)?;
        let sine_real = f08_lfe_multiply(sine, real)?;
        spectrum[2 * frequency] = match bank {
            F08LfeBank::Primary => f08_lfe_subtract(cosine_imaginary, sine_real)?,
            F08LfeBank::Companion => f08_lfe_subtract(sine_real, cosine_imaginary)?,
        };
    }

    for (output_index, frequency, rotation_index) in [(5, 5, 125), (3, 6, 126), (1, 7, 127)] {
        let cosine = f32::from_bits(parameters.long_cosine_f32_bits()[rotation_index]);
        let sine = f32::from_bits(parameters.long_sine_f32_bits()[rotation_index]);
        let real = low_frequency[2 * frequency];
        let imaginary = low_frequency[2 * frequency + 1];
        let rotated = f08_lfe_add(
            f08_lfe_multiply(cosine, real)?,
            f08_lfe_multiply(sine, imaginary)?,
        )?;
        spectrum[output_index] = rotated;
    }
    Ok(spectrum.map(f32::to_bits))
}

fn map_f08_lfe_pre_error(_: F08FbwTransformError) -> F08LfeTransformError {
    F08LfeTransformError::NumericFailure
}

fn f08_lfe_multiply(left: f32, right: f32) -> Result<f32, F08LfeTransformError> {
    let left = FiniteF32::new(left).map_err(|_| F08LfeTransformError::NumericFailure)?;
    let right = FiniteF32::new(right).map_err(|_| F08LfeTransformError::NumericFailure)?;
    r32_mul(left, right)
        .map(FiniteF32::value)
        .map_err(|_| F08LfeTransformError::NumericFailure)
}

fn f08_lfe_add(left: f32, right: f32) -> Result<f32, F08LfeTransformError> {
    let left = FiniteF32::new(left).map_err(|_| F08LfeTransformError::NumericFailure)?;
    let right = FiniteF32::new(right).map_err(|_| F08LfeTransformError::NumericFailure)?;
    r32_add(left, right)
        .map(FiniteF32::value)
        .map_err(|_| F08LfeTransformError::NumericFailure)
}

fn f08_lfe_subtract(left: f32, right: f32) -> Result<f32, F08LfeTransformError> {
    let left = FiniteF32::new(left).map_err(|_| F08LfeTransformError::NumericFailure)?;
    let right = FiniteF32::new(right).map_err(|_| F08LfeTransformError::NumericFailure)?;
    r32_sub(left, right)
        .map(FiniteF32::value)
        .map_err(|_| F08LfeTransformError::NumericFailure)
}

fn f08_lfe_negate(value: f32) -> f32 {
    f32::from_bits(value.to_bits() ^ 0x8000_0000)
}
