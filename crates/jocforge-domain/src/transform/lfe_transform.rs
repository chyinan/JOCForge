// pattern: Functional Core
//! Selected 5.1 long-block LFE transform, dedicated FFT8 recurrence, and raw wrapper.

use super::long_transform::{
    FoldPreRotation, LongTransformHistory, TransformBranch, add, fold_and_pre_rotation, multiply,
    subtract,
};
use super::parameters::{COSINE_F32_BITS, SINE_F32_BITS};
use crate::codec::lfe_coefficient_exponent::{
    LFE_COEFFICIENT_BINS, LfeCoefficientBlock, LfeExponentOutputBlock, lfe_coefficient_pair_to_raw,
};

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
