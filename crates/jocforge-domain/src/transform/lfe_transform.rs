// pattern: Functional Core
//! Selected long-block LFE transforms, dedicated recurrence paths, and terminal projections.

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

const PRETRANSFORM_L_COEFFICIENTS_BITS: [u32; 128] = [
    0x390e9922, 0x397fbf9c, 0x39c02722, 0x3a051ad5, 0x3a2fa573, 0x3a603cef, 0x3a8bb306, 0x3aaad318,
    0x3acdc2b0, 0x3af4c794, 0x3b1014bd, 0x3b28190d, 0x3b42969b, 0x3b5fb449, 0x3b7f99f5, 0x3b91383a,
    0x3ba430c2, 0x3bb8cbe7, 0x3bcf1f67, 0x3be74167, 0x3c00a436, 0x3c0ea5aa, 0x3c1db0a7, 0x3c2dd0ec,
    0x3c3f1258, 0x3c5180e4, 0x3c6528a7, 0x3c7a15c9, 0x3c882a40, 0x3c93f889, 0x3ca07be2, 0x3cadba70,
    0x3cbbba55, 0x3cca81ad, 0x3cda168e, 0x3cea7f00, 0x3cfbc102, 0x3d06f13f, 0x3d1074a7, 0x3d1a6d9a,
    0x3d24deec, 0x3d2fcb64, 0x3d3b35b8, 0x3d47208e, 0x3d538e78, 0x3d6081f5, 0x3d6dfd6d, 0x3d7c0332,
    0x3d854abe, 0x3d8cdb35, 0x3d94b400, 0x3d9cd612, 0x3da5424d, 0x3dadf987, 0x3db6fc83, 0x3dc04bf2,
    0x3dc9e876, 0x3dd3d29b, 0x3dde0ade, 0x3de891a5, 0x3df36744, 0x3dfe8bfa, 0x3e04fff8, 0x3e0ae19d,
    0x3e10eaec, 0x3e171bd8, 0x3e1d744b, 0x3e23f422, 0x3e2a9b2e, 0x3e316936, 0x3e385df4, 0x3e3f7917,
    0x3e46ba41, 0x3e4e2108, 0x3e55acf8, 0x3e5d5d90, 0x3e653241, 0x3e6d2a74, 0x3e754584, 0x3e7d82bf,
    0x3e82f0b5, 0x3e873060, 0x3e8b7ff5, 0x3e8fdf07, 0x3e944d20, 0x3e98c9c8, 0x3e9d5480, 0x3ea1ecc3,
    0x3ea6920a, 0x3eab43c5, 0x3eb00165, 0x3eb4ca51, 0x3eb99df0, 0x3ebe7ba1, 0x3ec362c4, 0x3ec852b1,
    0x3ecd4abf, 0x3ed24a41, 0x3ed75087, 0x3edc5cde, 0x3ee16e92, 0x3ee684ec, 0x3eeb9f31, 0x3ef0bca8,
    0x3ef5dc93, 0x3efafe35, 0x3f001068, 0x3f02a1d3, 0x3f0532fb, 0x3f07c380, 0x3f0a5303, 0x3f0ce124,
    0x3f0f6d85, 0x3f11f7c7, 0x3f147f8c, 0x3f170477, 0x3f19862c, 0x3f1c0450, 0x3f1e7e8a, 0x3f20f481,
    0x3f2365dd, 0x3f25d24b, 0x3f283975, 0x3f2a9b0b, 0x3f2cf6bc, 0x3f2f4c3b, 0x3f319b3c, 0x3f33e377,
];

const PRETRANSFORM_R_COEFFICIENTS_BITS: [u32; 128] = [
    0x3f3624a4, 0x3f385e7e, 0x3f3a90c5, 0x3f3cbb3a, 0x3f3edd9f, 0x3f40f7bc, 0x3f43095a, 0x3f451245,
    0x3f47124d, 0x3f490945, 0x3f4af702, 0x3f4cdb5e, 0x3f4eb633, 0x3f508762, 0x3f524ecd, 0x3f540c5a,
    0x3f55bff3, 0x3f576983, 0x3f5908fb, 0x3f5a9e4d, 0x3f5c2970, 0x3f5daa5d, 0x3f5f2112, 0x3f608d8d,
    0x3f61efd1, 0x3f6347e6, 0x3f6495d4, 0x3f65d9a6, 0x3f67136d, 0x3f684339, 0x3f69691f, 0x3f6a8537,
    0x3f6b9799, 0x3f6ca062, 0x3f6d9fb0, 0x3f6e95a5, 0x3f6f8263, 0x3f70660f, 0x3f7140d1, 0x3f7212d0,
    0x3f72dc37, 0x3f739d33, 0x3f7455f1, 0x3f7506a0, 0x3f75af70, 0x3f765093, 0x3f76ea39, 0x3f777c98,
    0x3f7807e1, 0x3f788c4b, 0x3f790a09, 0x3f798151, 0x3f79f258, 0x3f7a5d55, 0x3f7ac27c, 0x3f7b2203,
    0x3f7b7c1f, 0x3f7bd106, 0x3f7c20ec, 0x3f7c6c06, 0x3f7cb285, 0x3f7cf49f, 0x3f7d3284, 0x3f7d6c65,
    0x3f7da274, 0x3f7dd4de, 0x3f7e03d4, 0x3f7e2f80, 0x3f7e5810, 0x3f7e7daf, 0x3f7ea085, 0x3f7ec0bb,
    0x3f7ede78, 0x3f7ef9e2, 0x3f7f131c, 0x3f7f2a4a, 0x3f7f3f8c, 0x3f7f5304, 0x3f7f64d0, 0x3f7f750d,
    0x3f7f83d7, 0x3f7f9148, 0x3f7f9d7b, 0x3f7fa887, 0x3f7fb282, 0x3f7fbb83, 0x3f7fc39d, 0x3f7fcae3,
    0x3f7fd168, 0x3f7fd73b, 0x3f7fdc6d, 0x3f7fe10c, 0x3f7fe525, 0x3f7fe8c6, 0x3f7febf9, 0x3f7feeca,
    0x3f7ff143, 0x3f7ff36c, 0x3f7ff54f, 0x3f7ff6f2, 0x3f7ff85d, 0x3f7ff997, 0x3f7ffaa4, 0x3f7ffb8b,
    0x3f7ffc50, 0x3f7ffcf7, 0x3f7ffd84, 0x3f7ffdfb, 0x3f7ffe5e, 0x3f7ffeb1, 0x3f7ffef5, 0x3f7fff2d,
    0x3f7fff5b, 0x3f7fff80, 0x3f7fff9e, 0x3f7fffb6, 0x3f7fffc9, 0x3f7fffd7, 0x3f7fffe3, 0x3f7fffeb,
    0x3f7ffff2, 0x3f7ffff6, 0x3f7ffffa, 0x3f7ffffc, 0x3f7ffffe, 0x3f7fffff, 0x3f800000, 0x3f800000,
];
/// The two history halves for one selected selector-zero LFE record.
///
/// Construct with [`LfeSelectorZeroHistory::new_record_zeroed`] only for a newly carved record.
/// Reuse the returned after-state for later blocks; the evidence does not support per-block reset
/// or arbitrary lane/slot selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LfeSelectorZeroHistory {
    main_history_bits: [u32; 128],
    companion_history_bits: [u32; 128],
}

impl LfeSelectorZeroHistory {
    /// Creates the zero history bound to a newly carved record.
    pub const fn new_record_zeroed() -> Self {
        Self {
            main_history_bits: [0; 128],
            companion_history_bits: [0; 128],
        }
    }

    pub const fn main_history_bits(&self) -> &[u32; 128] {
        &self.main_history_bits
    }

    pub const fn companion_history_bits(&self) -> &[u32; 128] {
        &self.companion_history_bits
    }
}

/// Noncommitting outputs of the selector-zero LFE pre-transform for one 256-sample block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LfeSelectorZeroPretransformResult {
    main_work_bits: [u32; 256],
    companion_work_bits: [u32; 256],
    after_history: LfeSelectorZeroHistory,
}

impl LfeSelectorZeroPretransformResult {
    pub const fn main_work_bits(&self) -> &[u32; 256] {
        &self.main_work_bits
    }

    pub const fn companion_work_bits(&self) -> &[u32; 256] {
        &self.companion_work_bits
    }

    pub const fn after_history(&self) -> &LfeSelectorZeroHistory {
        &self.after_history
    }
}

/// Applies the hash-bound main and companion selector-zero LFE pre-transforms to one block.
///
/// Each work array is overwritten in full. Main and companion consume independent history
/// halves; the caller owns the returned history and carries it to the next block. This function
/// implements the captured layout-7 pre-transform only. It does not select a lane or slot, run the
/// following 64-group recurrence, execute wrapper gates, apply the 6x6 matrix branch, or emit frame
/// data. The pre-transform's runtime rounding/FMA policy was not captured.
#[must_use]
pub fn pretransform_selector_zero_lfe_block(
    before: &LfeSelectorZeroHistory,
    input_bits: &[u32; 256],
) -> LfeSelectorZeroPretransformResult {
    let mut main_work_bits = [0_u32; 256];
    let mut companion_work_bits = [0_u32; 256];
    let mut after_history = *before;

    for iteration in 0..64 {
        let even = iteration * 2;
        let odd = even + 1;
        let tail_even = 255 - even;
        let tail_odd = 254 - even;
        let right_even = 127 - even;
        let right_odd = 126 - even;

        let input_even = f32::from_bits(input_bits[even]);
        let input_odd = f32::from_bits(input_bits[odd]);
        let input_tail_even = f32::from_bits(input_bits[tail_even]);
        let input_tail_odd = f32::from_bits(input_bits[tail_odd]);
        let left_even = f32::from_bits(PRETRANSFORM_L_COEFFICIENTS_BITS[even]);
        let left_odd = f32::from_bits(PRETRANSFORM_L_COEFFICIENTS_BITS[odd]);
        let right_even_value = f32::from_bits(PRETRANSFORM_R_COEFFICIENTS_BITS[right_even]);
        let right_odd_value = f32::from_bits(PRETRANSFORM_R_COEFFICIENTS_BITS[right_odd]);

        let old_main_upper = f32::from_bits(before.main_history_bits[64 + iteration]);
        let old_main_lower = f32::from_bits(before.main_history_bits[63 - iteration]);
        let old_companion_upper = f32::from_bits(before.companion_history_bits[64 + iteration]);
        let old_companion_lower = f32::from_bits(before.companion_history_bits[63 - iteration]);

        main_work_bits[128 + even] =
            (input_even * right_even_value + input_tail_even * left_even).to_bits();
        main_work_bits[129 + even] = old_main_upper.to_bits();
        main_work_bits[127 - even] =
            (input_odd * right_odd_value + input_tail_odd * left_odd).to_bits();
        main_work_bits[126 - even] = old_main_lower.to_bits();
        after_history.main_history_bits[64 + iteration] =
            (input_tail_even * right_even_value - left_even * input_even).to_bits();
        after_history.main_history_bits[63 - iteration] =
            (input_tail_odd * right_odd_value - left_odd * input_odd).to_bits();

        companion_work_bits[128 + even] = old_companion_upper.to_bits();
        companion_work_bits[129 + even] =
            (input_even * right_even_value - input_tail_even * left_even).to_bits();
        companion_work_bits[127 - even] = old_companion_lower.to_bits();
        companion_work_bits[126 - even] =
            (input_odd * right_odd_value - input_tail_odd * left_odd).to_bits();
        after_history.companion_history_bits[64 + iteration] =
            (input_tail_even * right_even_value + left_even * input_even).to_bits();
        after_history.companion_history_bits[63 - iteration] =
            (input_tail_odd * right_odd_value + left_odd * input_odd).to_bits();
    }

    LfeSelectorZeroPretransformResult {
        main_work_bits,
        companion_work_bits,
        after_history,
    }
}
const LFE_16POINT_W0_BITS: [u32; 8] = [
    0x3f80_0000,
    0x3f7f_b10f,
    0x3f7e_c46d,
    0x3f7d_3aac,
    0x3f7b_14be,
    0x3f7d_3aac,
    0x3f7e_c46d,
    0x3f7f_b10f,
];
const LFE_16POINT_W1_BITS: [u32; 8] = [
    0x0000_0000,
    0x3d48_fb30,
    0x3dc8_bd36,
    0x3e16_4083,
    0xbe47_c5c2,
    0xbe16_4083,
    0xbdc8_bd36,
    0xbd48_fb30,
];
const LFE_16POINT_C_BITS: [u32; 8] = [
    0x4000_0000,
    0x3fff_b10f,
    0x3ffe_c46d,
    0x3ffd_3aac,
    0x3ffb_14be,
    0x3ffd_3aac,
    0x3ffe_c46d,
    0x3fff_b10f,
];
const LFE_16POINT_BIN_PERMUTATION: [usize; 8] = [0, 4, 2, 6, 1, 5, 3, 7];
const LFE_16POINT_OUTPUT_SCALE_BITS: u32 = 0x3d80_0000;
const LFE_16POINT_SIGN_MASK: u32 = 0x8000_0000;
const LFE_16POINT_ROTATION_Q_BITS: u32 = 0x3f35_04f3;

/// Applies the sanitized 16-point butterfly to one gathered F32 row.
///
/// Input and output words are IEEE-754 binary32 bits. The equations use separate F32
/// multiply/add/subtract operations and sign-bit flips; this helper does not apply wrapper gates.
#[must_use]
pub fn transform_lfe_16point_butterfly_row(input_bits: &[u32; 16]) -> [u32; 16] {
    let v = input_bits.map(f32::from_bits);
    let mut a = [0.0_f32; 16];

    for index in 0..4 {
        a[index] = v[index] + v[8 + index];
        a[4 + index] = v[4 + index] + v[12 + index];
        a[8 + index] = v[index] - v[8 + index];
    }
    a[12] = f32::from_bits((v[5] - v[13]).to_bits() ^ LFE_16POINT_SIGN_MASK);
    a[13] = v[4] - v[12];
    a[14] = f32::from_bits((v[7] - v[15]).to_bits() ^ LFE_16POINT_SIGN_MASK);
    a[15] = v[6] - v[14];

    let mut b = [0.0_f32; 16];
    b[0] = a[0] + a[4];
    b[4] = a[0] - a[4];
    b[1] = a[1] + a[5];
    b[5] = a[1] - a[5];
    b[2] = a[2] + a[6];
    b[3] = a[2] + a[7];
    b[6] = f32::from_bits((a[3] - a[7]).to_bits() ^ LFE_16POINT_SIGN_MASK);
    b[7] = a[2] - a[6];
    b[8..].copy_from_slice(&a[8..]);

    let mut c = b;
    for index in 0..4 {
        c[8 + index] = b[8 + index] - b[12 + index];
        c[12 + index] = b[12 + index] + b[8 + index];
    }

    let rotation = f32::from_bits(LFE_16POINT_ROTATION_Q_BITS);
    let q10 = rotation * c[10];
    let q11 = rotation * c[11];
    let q14 = rotation * c[14];
    let q15 = rotation * c[15];
    c[10] = q10 + q11;
    c[11] = q11 - q10;
    c[14] = q15 - q14;
    c[15] = f32::from_bits(q14.to_bits() ^ LFE_16POINT_SIGN_MASK) - q15;

    c.map(f32::to_bits)
}

/// Applies the sanitized 256-to-16 F32 stage to one already-prepared block.
///
/// The caller owns pre-transform/history and selector-zero selection. This stage gathers the
/// sixteen 16-value rows, applies the butterfly, runs the three-pass recurrence, and returns raw
/// F32 bits. Runtime MXCSR mode, the separate 6x6 matrix branch, wrapper gates, LFE wire fields,
/// and PCM parity remain outside this value function.
#[must_use]
pub fn transform_lfe_16point_block(input_bits: &[u32; 256]) -> [u32; 16] {
    let mut rows = [[0.0_f32; 16]; 16];
    for row in 0..16 {
        let mut gathered_bits = [0_u32; 16];
        for group in 0..8 {
            gathered_bits[2 * group] = input_bits[32 * group + 2 * row];
            gathered_bits[2 * group + 1] = input_bits[32 * group + 2 * row + 1];
        }
        rows[row] = transform_lfe_16point_butterfly_row(&gathered_bits).map(f32::from_bits);
    }

    let output_scale = f32::from_bits(LFE_16POINT_OUTPUT_SCALE_BITS);
    let mut output = [0_u32; 16];
    for bin in 0..8 {
        let column = 2 * LFE_16POINT_BIN_PERMUTATION[bin];
        let cosine = f32::from_bits(LFE_16POINT_C_BITS[bin]);
        let window_0 = f32::from_bits(LFE_16POINT_W0_BITS[bin]);
        let window_1 = f32::from_bits(LFE_16POINT_W1_BITS[bin]);
        let mut state_r = 0.0_f32;
        let mut state_q = 0.0_f32;
        let mut state_a = rows[15][column];
        let mut state_b = rows[15][column + 1];

        for top in [14, 9, 4] {
            let next_r = (cosine * state_a - state_r) + rows[top][column];
            let next_q = (cosine * state_b - state_q) + rows[top][column + 1];
            let next_a = (cosine * next_r - state_a) + rows[top - 1][column];
            let next_u = (cosine * next_q - state_b) + rows[top - 1][column + 1];
            let next_b = (cosine * next_a - next_r) + rows[top - 2][column];
            let next_v = (cosine * next_u - next_q) + rows[top - 2][column + 1];
            state_r = (cosine * next_b - next_a) + rows[top - 3][column];
            state_q = (cosine * next_v - next_u) + rows[top - 3][column + 1];
            state_a = (cosine * state_r - next_b) + rows[top - 4][column];
            state_b = (cosine * state_q - next_v) + rows[top - 4][column + 1];
        }

        output[2 * bin] =
            (((state_a - state_r) * window_0 + state_q * window_1) * output_scale).to_bits();
        output[2 * bin + 1] =
            (((state_b - state_r * window_1) - state_q * window_0) * output_scale).to_bits();
    }
    output
}
const LFE_SELECTOR_ZERO_SCALE_BITS: [u32; 4] = [0xbf80_0000, 0x3f80_0000, 0xbf80_0000, 0x3f80_0000];

/// Exact Wash-bound primary A words for the selector-zero recurrence.
pub const SELECTOR_ZERO_LFE_RECURRENCE_PRIMARY_COEFFICIENT_BITS: [u32; 128] = [
    0xbf7f_ffec,
    0xbf7f_f9c1,
    0xbf7f_e9b8,
    0xbf7f_cfd1,
    0xbf7f_ac0d,
    0xbf7f_7e6d,
    0xbf7f_46f4,
    0xbf7f_05a4,
    0xbf7e_ba7f,
    0xbf7e_6588,
    0xbf7e_06c2,
    0xbf7d_9e30,
    0xbf7d_2bd8,
    0xbf7c_afbd,
    0xbf7c_29e5,
    0xbf7b_9a53,
    0xbf7b_010e,
    0xbf7a_5e1c,
    0xbf79_b183,
    0xbf78_fb4a,
    0xbf78_3b77,
    0xbf77_7213,
    0xbf76_9f24,
    0xbf75_c2b3,
    0xbf74_dcc9,
    0xbf73_ed6e,
    0xbf72_f4ac,
    0xbf71_f28c,
    0xbf70_e717,
    0xbf6f_d25a,
    0xbf6e_b45d,
    0xbf6d_8d2d,
    0xbf6c_5cd4,
    0xbf6b_235e,
    0xbf69_e0d7,
    0xbf68_954c,
    0xbf67_40ca,
    0xbf65_e35e,
    0xbf64_7d14,
    0xbf63_0dfc,
    0xbf61_9622,
    0xbf60_1596,
    0xbf5e_8c67,
    0xbf5c_faa3,
    0xbf5b_605a,
    0xbf59_bd9c,
    0xbf58_1279,
    0xbf56_5f02,
    0xbf54_a347,
    0xbf52_df59,
    0xbf51_134a,
    0xbf4f_3f2b,
    0xbf4d_6310,
    0xbf4b_7f09,
    0xbf49_9329,
    0xbf47_9f84,
    0xbf45_a42d,
    0xbf43_a138,
    0xbf41_96b7,
    0xbf3f_84c0,
    0xbf3d_6b67,
    0xbf3b_4ac1,
    0xbf39_22e1,
    0xbf36_f3df,
    0xbf34_bdcf,
    0xbf32_80c7,
    0xbf30_3cde,
    0xbf2d_f229,
    0xbf2b_a0bf,
    0xbf29_48b8,
    0xbf26_ea2a,
    0xbf24_852c,
    0xbf22_19d7,
    0xbf1f_a841,
    0xbf1d_3084,
    0xbf1a_b2b8,
    0xbf18_2ef5,
    0xbf15_a554,
    0xbf13_15ee,
    0xbf10_80dc,
    0xbf0d_e638,
    0xbf0b_461c,
    0xbf08_a0a1,
    0xbf05_f5e2,
    0xbf03_45f8,
    0xbf00_90ff,
    0xbefb_ae22,
    0xbef6_3093,
    0xbef0_a985,
    0xbeeb_1930,
    0xbee5_7fcb,
    0xbedf_dd8d,
    0xbeda_32ad,
    0xbed4_7f64,
    0xbece_c3ea,
    0xbec9_0077,
    0xbec3_3544,
    0xbebd_628b,
    0xbeb7_8884,
    0xbeb1_a76b,
    0xbeab_bf77,
    0xbea5_d0e5,
    0xbe9f_dbee,
    0xbe99_e0ce,
    0xbe93_dfbf,
    0xbe8d_d8fc,
    0xbe87_ccc1,
    0xbe81_bb4a,
    0xbe77_49a6,
    0xbe6b_132f,
    0xbe5e_d3a8,
    0xbe52_8b89,
    0xbe46_3b4d,
    0xbe39_e36c,
    0xbe2d_8461,
    0xbe21_1ea5,
    0xbe14_b2b2,
    0xbe08_4105,
    0xbdf7_942c,
    0xbdde_9cc4,
    0xbdc5_9cc6,
    0xbdac_952b,
    0xbd93_86e7,
    0xbd74_e5e9,
    0xbd42_b491,
    0xbd10_7bb8,
    0xbcbc_7a9b,
    0xbc2f_ed02,
];

/// Exact Wash-bound secondary B words for the selector-zero recurrence.
pub const SELECTOR_ZERO_LFE_RECURRENCE_SECONDARY_COEFFICIENT_BITS: [u32; 128] = [
    0xbac9_0fd5,
    0xbc62_2fff,
    0xbcd5_9aa6,
    0xbd1d_0a88,
    0xbd4f_41af,
    0xbd80_b86c,
    0xbd99_cb0a,
    0xbdb2_d7bb,
    0xbdcb_dd86,
    0xbde4_db76,
    0xbdfd_d092,
    0xbe0b_5df3,
    0xbe17_ce3d,
    0xbe24_38ad,
    0xbe30_9cc9,
    0xbe3c_fa15,
    0xbe49_5018,
    0xbe55_9e58,
    0xbe61_e45c,
    0xbe6e_21ab,
    0xbe7a_55cb,
    0xbe83_4022,
    0xbe89_5050,
    0xbe8f_5b32,
    0xbe95_608d,
    0xbe9b_6027,
    0xbea1_59c2,
    0xbea7_4d25,
    0xbead_3a15,
    0xbeb3_2058,
    0xbeb8_ffb2,
    0xbebe_d7eb,
    0xbec4_a8c8,
    0xbeca_7210,
    0xbed0_338a,
    0xbed5_ecfd,
    0xbedb_9e31,
    0xbee1_46ee,
    0xbee6_e6fb,
    0xbeec_7e21,
    0xbef2_0c29,
    0xbef7_90dc,
    0xbefd_0c04,
    0xbf01_3eb5,
    0xbf03_f26d,
    0xbf06_a10e,
    0xbf09_4a7f,
    0xbf0b_eea5,
    0xbf0e_8d65,
    0xbf11_26a7,
    0xbf13_ba50,
    0xbf16_4847,
    0xbf18_d073,
    0xbf1b_52bb,
    0xbf1d_cf06,
    0xbf20_453b,
    0xbf22_b543,
    0xbf25_1f04,
    0xbf27_8268,
    0xbf29_df57,
    0xbf2c_35b9,
    0xbf2e_8578,
    0xbf30_ce7c,
    0xbf33_10af,
    0xbf35_4bfb,
    0xbf37_804a,
    0xbf39_ad85,
    0xbf3b_d398,
    0xbf3d_f26e,
    0xbf40_09f0,
    0xbf42_1a0b,
    0xbf44_22ab,
    0xbf46_23bb,
    0xbf48_1d27,
    0xbf4a_0edc,
    0xbf4b_f8c7,
    0xbf4d_dad5,
    0xbf4f_b4f4,
    0xbf51_8710,
    0xbf53_5118,
    0xbf55_12fb,
    0xbf56_cca7,
    0xbf58_7e0b,
    0xbf5a_2716,
    0xbf5b_c7b8,
    0xbf5d_5fe1,
    0xbf5e_ef81,
    0xbf60_7689,
    0xbf61_f4ea,
    0xbf63_6a95,
    0xbf64_d77b,
    0xbf66_3b8e,
    0xbf67_96c1,
    0xbf68_e907,
    0xbf6a_3252,
    0xbf6b_7295,
    0xbf6c_a9c5,
    0xbf6d_d7d5,
    0xbf6e_fcba,
    0xbf70_1867,
    0xbf71_2ad4,
    0xbf72_33f4,
    0xbf73_33be,
    0xbf74_2a27,
    0xbf75_1727,
    0xbf75_fab3,
    0xbf76_d4c4,
    0xbf77_a551,
    0xbf78_6c52,
    0xbf79_29bf,
    0xbf79_dd91,
    0xbf7a_87c1,
    0xbf7b_2848,
    0xbf7b_bf20,
    0xbf7c_4c44,
    0xbf7c_cfae,
    0xbf7d_4959,
    0xbf7d_b940,
    0xbf7e_1f5e,
    0xbf7e_7bb1,
    0xbf7e_ce34,
    0xbf7f_16e4,
    0xbf7f_55bf,
    0xbf7f_8ac2,
    0xbf7f_b5ea,
    0xbf7f_d737,
    0xbf7f_eea7,
    0xbf7f_fc39,
];

/// Applies the washed 64-group selector-zero recurrence with caller-supplied A/B tables.
///
/// The table arguments are raw IEEE-754 binary32 words and must come from the hash-bound
/// selector-zero coefficient package. This function does not choose a selector or supply any
/// nonzero-selector table.
#[must_use]
pub fn apply_selector_zero_lfe_recurrence(
    before_bits: &[u32; 256],
    selector_zero_primary_words: &[u32; 128],
    selector_zero_secondary_words: &[u32; 128],
) -> [u32; 256] {
    let scale = LFE_SELECTOR_ZERO_SCALE_BITS.map(f32::from_bits);
    let mut after_bits = [0_u32; 256];

    for group in 0..64 {
        let base = group * 4;
        let value_0 = f32::from_bits(before_bits[base]);
        let value_1 = f32::from_bits(before_bits[base + 1]);
        let value_2 = f32::from_bits(before_bits[base + 2]);
        let value_3 = f32::from_bits(before_bits[base + 3]);
        let a_even = f32::from_bits(selector_zero_primary_words[group * 2]);
        let a_odd = f32::from_bits(selector_zero_primary_words[group * 2 + 1]);
        let b_even = f32::from_bits(selector_zero_secondary_words[group * 2]);
        let b_odd = f32::from_bits(selector_zero_secondary_words[group * 2 + 1]);

        let first_product = a_even * value_0;
        let first_scaled = first_product * scale[0];
        let second_product = b_even * value_1;
        after_bits[base] = (first_scaled + second_product).to_bits();

        let first_product = a_even * value_1;
        let first_scaled = first_product * scale[1];
        let second_product = b_even * value_0;
        after_bits[base + 1] = (first_scaled + second_product).to_bits();

        let first_product = a_odd * value_2;
        let first_scaled = first_product * scale[2];
        let second_product = b_odd * value_3;
        after_bits[base + 2] = (first_scaled + second_product).to_bits();

        let first_product = a_odd * value_3;
        let first_scaled = first_product * scale[3];
        let second_product = b_odd * value_2;
        after_bits[base + 3] = (first_scaled + second_product).to_bits();
    }

    after_bits
}
/// Final main/companion values and the carried selector-zero history for one LFE block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LfeSelectorZeroBlockResult {
    main_terminal_bits: [u32; 7],
    companion_terminal_bits: [u32; 7],
    after_history: LfeSelectorZeroHistory,
}

impl LfeSelectorZeroBlockResult {
    pub const fn main_terminal_bits(&self) -> &[u32; 7] {
        &self.main_terminal_bits
    }

    pub const fn companion_terminal_bits(&self) -> &[u32; 7] {
        &self.companion_terminal_bits
    }

    pub const fn after_history(&self) -> &LfeSelectorZeroHistory {
        &self.after_history
    }
}

/// Composes the selector-zero pre-transform, recurrence with the module-owned Wash-bound tables,
/// 16-point stage, and terminal projection for one 256-sample block.
///
/// The caller owns record/slot selection and history carry. This function does not select a runtime
/// branch, run wrapper gates, apply the conditional 6x6 matrix branch, or emit frame/wire output; it
/// also does not reproduce an uncaptured runtime rounding mode.
#[must_use]
pub fn transform_selector_zero_lfe_block(
    before_history: &LfeSelectorZeroHistory,
    input_bits: &[u32; 256],
) -> LfeSelectorZeroBlockResult {
    let pretransform = pretransform_selector_zero_lfe_block(before_history, input_bits);
    let main_recurrence = apply_selector_zero_lfe_recurrence(
        pretransform.main_work_bits(),
        &SELECTOR_ZERO_LFE_RECURRENCE_PRIMARY_COEFFICIENT_BITS,
        &SELECTOR_ZERO_LFE_RECURRENCE_SECONDARY_COEFFICIENT_BITS,
    );
    let companion_recurrence = apply_selector_zero_lfe_recurrence(
        pretransform.companion_work_bits(),
        &SELECTOR_ZERO_LFE_RECURRENCE_PRIMARY_COEFFICIENT_BITS,
        &SELECTOR_ZERO_LFE_RECURRENCE_SECONDARY_COEFFICIENT_BITS,
    );
    let main_window_bits = transform_lfe_16point_block(&main_recurrence);
    let companion_window_bits = transform_lfe_16point_block(&companion_recurrence);
    let main_window = main_window_bits.map(f32::from_bits);
    let companion_window = companion_window_bits.map(f32::from_bits);
    let main_terminal_bits = project_main_lfe_terminal_window(&main_window).map(f32::to_bits);
    let companion_terminal_bits =
        project_companion_lfe_terminal_window(&companion_window).map(f32::to_bits);

    LfeSelectorZeroBlockResult {
        main_terminal_bits,
        companion_terminal_bits,
        after_history: *pretransform.after_history(),
    }
}
const TERMINAL_A_0_BITS: u32 = 0xbf7f_ffec;
const TERMINAL_A_1_BITS: u32 = 0xbf7f_f9c1;
const TERMINAL_A_2_BITS: u32 = 0xbf7f_e9b8;
const TERMINAL_A_3_BITS: u32 = 0xbf7f_cfd1;
const TERMINAL_A_125_BITS: u32 = 0xbd10_7bb8;
const TERMINAL_A_126_BITS: u32 = 0xbcbc_7a9b;
const TERMINAL_A_127_BITS: u32 = 0xbc2f_ed02;
const TERMINAL_B_0_BITS: u32 = 0xbac9_0fd5;
const TERMINAL_B_1_BITS: u32 = 0xbc62_2fff;
const TERMINAL_B_2_BITS: u32 = 0xbcd5_9aa6;
const TERMINAL_B_3_BITS: u32 = 0xbd1d_0a88;
const TERMINAL_B_125_BITS: u32 = 0xbf7f_d737;
const TERMINAL_B_126_BITS: u32 = 0xbf7f_eea7;
const TERMINAL_B_127_BITS: u32 = 0xbf7f_fc39;

/// Static destination order reported for both terminal seven-value bodies.
pub const LFE_TERMINAL_WRITE_ORDER: [u8; 7] = [0, 2, 4, 6, 5, 3, 1];

/// Applies the hash-bound main terminal projection to one 16-value F32 window.
///
/// This standalone value function does not run the preceding recurrence, wrapper gates, or
/// conditional matrix branch, and it does not write an LFE frame record. F32 rounding/FMA policy
/// remains unclaimed.
#[must_use]
pub fn project_main_lfe_terminal_window(source: &[f32; 16]) -> [f32; 7] {
    let a0 = f32::from_bits(TERMINAL_A_0_BITS);
    let a1 = f32::from_bits(TERMINAL_A_1_BITS);
    let a2 = f32::from_bits(TERMINAL_A_2_BITS);
    let a3 = f32::from_bits(TERMINAL_A_3_BITS);
    let a125 = f32::from_bits(TERMINAL_A_125_BITS);
    let a126 = f32::from_bits(TERMINAL_A_126_BITS);
    let a127 = f32::from_bits(TERMINAL_A_127_BITS);
    let b0 = f32::from_bits(TERMINAL_B_0_BITS);
    let b1 = f32::from_bits(TERMINAL_B_1_BITS);
    let b2 = f32::from_bits(TERMINAL_B_2_BITS);
    let b3 = f32::from_bits(TERMINAL_B_3_BITS);
    let b125 = f32::from_bits(TERMINAL_B_125_BITS);
    let b126 = f32::from_bits(TERMINAL_B_126_BITS);
    let b127 = f32::from_bits(TERMINAL_B_127_BITS);
    let mut output = [0.0_f32; 7];

    output[0] = a0 * source[1] - b0 * source[0];
    output[2] = a1 * source[3] - b1 * source[2];
    output[4] = a2 * source[5] - b2 * source[4];
    output[6] = a3 * source[7] - b3 * source[6];
    output[5] = a125 * source[10] + b125 * source[11];
    output[3] = a126 * source[12] + b126 * source[13];
    output[1] = a127 * source[14] + b127 * source[15];

    output
}

/// Applies the hash-bound companion terminal projection to one 16-value F32 window.
#[must_use]
pub fn project_companion_lfe_terminal_window(source: &[f32; 16]) -> [f32; 7] {
    let a0 = f32::from_bits(TERMINAL_A_0_BITS);
    let a1 = f32::from_bits(TERMINAL_A_1_BITS);
    let a2 = f32::from_bits(TERMINAL_A_2_BITS);
    let a3 = f32::from_bits(TERMINAL_A_3_BITS);
    let a125 = f32::from_bits(TERMINAL_A_125_BITS);
    let a126 = f32::from_bits(TERMINAL_A_126_BITS);
    let a127 = f32::from_bits(TERMINAL_A_127_BITS);
    let b0 = f32::from_bits(TERMINAL_B_0_BITS);
    let b1 = f32::from_bits(TERMINAL_B_1_BITS);
    let b2 = f32::from_bits(TERMINAL_B_2_BITS);
    let b3 = f32::from_bits(TERMINAL_B_3_BITS);
    let b125 = f32::from_bits(TERMINAL_B_125_BITS);
    let b126 = f32::from_bits(TERMINAL_B_126_BITS);
    let b127 = f32::from_bits(TERMINAL_B_127_BITS);
    let mut output = [0.0_f32; 7];

    output[0] = b0 * source[0] - a0 * source[1];
    output[2] = b1 * source[2] - a1 * source[3];
    output[4] = b2 * source[4] - a2 * source[5];
    output[6] = b3 * source[6] - a3 * source[7];
    output[5] = a125 * source[10] + b125 * source[11];
    output[3] = a126 * source[12] + b126 * source[13];
    output[1] = a127 * source[14] + b127 * source[15];

    output
}
