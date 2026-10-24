// pattern: Functional Core
//! Bounded 256-word long transform with caller-owned 128-word history.

use super::parameters::{
    BUTTERFLY_F32_BITS, COSINE_F32_BITS, PERMUTATION_I16, SINE_F32_BITS, WINDOW_F32_BITS,
};
use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};

/// Number of interleaved F32 words in one input/output block.
pub const LONG_TRANSFORM_WORDS: usize = 256;

/// Number of F32 values in one primary or companion history row.
pub const LONG_TRANSFORM_HISTORY_WORDS: usize = 128;

/// Selects the long-transform branch whose history and even spectrum bins are used.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransformBranch {
    Primary,
    Companion,
}

/// Caller-owned branch history. `zeroed` creates the synthetic zero fixture state; it does
/// not assert that a product/native cold or reset owner initializes this value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LongTransformHistory {
    words: [u32; LONG_TRANSFORM_HISTORY_WORDS],
}

impl LongTransformHistory {
    pub const fn zeroed() -> Self {
        Self {
            words: [0; LONG_TRANSFORM_HISTORY_WORDS],
        }
    }

    pub const fn words(&self) -> &[u32; LONG_TRANSFORM_HISTORY_WORDS] {
        &self.words
    }
}

/// Stage outputs and next caller-owned history from one bounded transform call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LongTransformOutput {
    folded_bits: [u32; LONG_TRANSFORM_WORDS],
    pre_rotated_bits: [u32; LONG_TRANSFORM_WORDS],
    fft_bits: [u32; LONG_TRANSFORM_WORDS],
    post_rotated_bits: [u32; LONG_TRANSFORM_WORDS],
    wrapper_spectrum_bits: [u32; LONG_TRANSFORM_WORDS],
    history_after: LongTransformHistory,
}

impl LongTransformOutput {
    pub const fn folded_bits(&self) -> &[u32; LONG_TRANSFORM_WORDS] {
        &self.folded_bits
    }

    pub const fn pre_rotated_bits(&self) -> &[u32; LONG_TRANSFORM_WORDS] {
        &self.pre_rotated_bits
    }

    pub const fn fft_bits(&self) -> &[u32; LONG_TRANSFORM_WORDS] {
        &self.fft_bits
    }

    pub const fn post_rotated_bits(&self) -> &[u32; LONG_TRANSFORM_WORDS] {
        &self.post_rotated_bits
    }

    pub const fn wrapper_spectrum_bits(&self) -> &[u32; LONG_TRANSFORM_WORDS] {
        &self.wrapper_spectrum_bits
    }

    pub const fn history_after(&self) -> &LongTransformHistory {
        &self.history_after
    }
}

pub(super) struct FoldPreRotation {
    pub(super) folded: [f32; LONG_TRANSFORM_WORDS],
    pub(super) pre_rotated: [f32; LONG_TRANSFORM_WORDS],
    pub(super) history_after: LongTransformHistory,
}

pub(super) fn fold_and_pre_rotation(
    input_bits: &[u32; LONG_TRANSFORM_WORDS],
    history: &LongTransformHistory,
    branch: TransformBranch,
) -> FoldPreRotation {
    let input = input_bits.map(f32::from_bits);
    let mut folded = [0.0; LONG_TRANSFORM_WORDS];
    let mut history_after = history.clone();

    for index in 0..64 {
        let tuples = [
            (
                2 * index,
                255 - 2 * index,
                128 + 2 * index,
                129 + 2 * index,
                64 + index,
            ),
            (
                2 * index + 1,
                254 - 2 * index,
                127 - 2 * index,
                126 - 2 * index,
                63 - index,
            ),
        ];

        for (a, b, output, side, history_index) in tuples {
            let old = f32::from_bits(history.words[history_index]);
            let a_value = multiply(input[a], f32::from_bits(WINDOW_F32_BITS[b]));
            let b_value = multiply(input[b], f32::from_bits(WINDOW_F32_BITS[a]));
            let c_value = multiply(input[b], f32::from_bits(WINDOW_F32_BITS[b]));
            let d_value = multiply(f32::from_bits(WINDOW_F32_BITS[a]), input[a]);

            match branch {
                TransformBranch::Primary => {
                    folded[output] = add(a_value, b_value);
                    folded[side] = old;
                    history_after.words[history_index] = subtract(c_value, d_value).to_bits();
                }
                TransformBranch::Companion => {
                    folded[output] = old;
                    folded[side] = subtract(a_value, b_value);
                    history_after.words[history_index] = add(c_value, d_value).to_bits();
                }
            }
        }
    }

    let mut pre_rotated = [0.0; LONG_TRANSFORM_WORDS];
    for index in 0..128 {
        let cosine = f32::from_bits(COSINE_F32_BITS[index]);
        let sine = f32::from_bits(SINE_F32_BITS[index]);
        let x = folded[2 * index];
        let y = folded[2 * index + 1];
        pre_rotated[2 * index] = add(-multiply(cosine, x), multiply(sine, y));
        pre_rotated[2 * index + 1] = add(multiply(cosine, y), multiply(sine, x));
    }

    FoldPreRotation {
        folded,
        pre_rotated,
        history_after,
    }
}

/// Computes one washed synthetic-profile long transform call.
///
/// The caller supplies exactly 256 finite F32 bit words and one branch-owned 128-word
/// history. The function returns all directly specified stage boundaries and the next
/// history without mutating the caller's state. It does not select a parent route, produce
/// input samples, or define an ADM/sample clock.
///
/// # Panics
///
/// Panics if an input or intermediate violates the package's finite synthetic profile,
/// or if its fixed parameter tables cannot be represented as finite F32 values.
pub fn long_transform(
    input_bits: &[u32; LONG_TRANSFORM_WORDS],
    history: &LongTransformHistory,
    branch: TransformBranch,
) -> LongTransformOutput {
    let prefix = fold_and_pre_rotation(input_bits, history, branch);
    let folded = prefix.folded;
    let pre_rotated = prefix.pre_rotated;
    let history_after = prefix.history_after;

    let mut fft = pre_rotated;
    fft_in_place(&mut fft, 7);

    let mut post_rotated = [0.0; LONG_TRANSFORM_WORDS];
    for index in 0..128 {
        let source = usize::try_from(PERMUTATION_I16[index])
            .expect("washed permutation indices are nonnegative");
        let x = fft[2 * source];
        let y = fft[2 * source + 1];
        let cosine = f32::from_bits(COSINE_F32_BITS[index]);
        let sine = f32::from_bits(SINE_F32_BITS[index]);
        let cosine_y = multiply(cosine, y);
        let sine_x = multiply(sine, x);
        let first_component = match branch {
            TransformBranch::Primary => subtract(cosine_y, sine_x),
            TransformBranch::Companion => subtract(sine_x, cosine_y),
        };
        post_rotated[2 * index] = first_component;
        post_rotated[255 - 2 * index] = add(multiply(cosine, x), multiply(sine, y));
    }

    LongTransformOutput {
        folded_bits: folded.map(f32::to_bits),
        pre_rotated_bits: pre_rotated.map(f32::to_bits),
        fft_bits: fft.map(f32::to_bits),
        post_rotated_bits: post_rotated.map(f32::to_bits),
        wrapper_spectrum_bits: post_rotated.map(f32::to_bits),
        history_after,
    }
}

fn finite(value: f32) -> FiniteF32 {
    FiniteF32::new(value).expect("long-transform input and intermediates are finite by profile")
}

pub(super) fn multiply(left: f32, right: f32) -> f32 {
    r32_mul(finite(left), finite(right))
        .expect("long-transform products remain finite by profile")
        .value()
}

pub(super) fn add(left: f32, right: f32) -> f32 {
    r32_add(finite(left), finite(right))
        .expect("long-transform sums remain finite by profile")
        .value()
}

pub(super) fn subtract(left: f32, right: f32) -> f32 {
    r32_sub(finite(left), finite(right))
        .expect("long-transform differences remain finite by profile")
        .value()
}

fn fft_in_place(values: &mut [f32], stage_count: usize) {
    let complex_count = values.len() / 2;
    let mut half = complex_count / 2;
    let mut group_count = 1;

    for _stage in 0..stage_count {
        for group in 0..group_count {
            let coefficients = if group >= 2 {
                let cosine = f32::from_bits(BUTTERFLY_F32_BITS[group - 1]);
                let sine = if group % 2 == 1 {
                    f32::from_bits(BUTTERFLY_F32_BITS[group - 2])
                } else {
                    -f32::from_bits(BUTTERFLY_F32_BITS[group])
                };
                Some((cosine, sine))
            } else {
                None
            };

            for pair in 0..half {
                let input_a = 2 * (2 * half * group + pair);
                let input_b = input_a + 2 * half;
                let ar = values[input_a];
                let ai = values[input_a + 1];
                let br = values[input_b];
                let bi = values[input_b + 1];

                let (new_real, new_imaginary) = match group {
                    0 => (multiply(add(ar, br), 0.5), multiply(add(ai, bi), 0.5)),
                    1 => (multiply(add(ar, bi), 0.5), multiply(subtract(ai, br), 0.5)),
                    _ => {
                        let (cosine, sine) = coefficients
                            .expect("groups >= 2 use one fixed coefficient pair per group");
                        let cosine_real = multiply(cosine, br);
                        let sine_imaginary = multiply(sine, bi);
                        let real_before_scale = subtract(subtract(ar, cosine_real), sine_imaginary);
                        let cosine_imaginary = multiply(cosine, bi);
                        let sine_real = multiply(sine, br);
                        let imaginary_before_scale =
                            subtract(subtract(ai, cosine_imaginary), -sine_real);
                        (
                            multiply(real_before_scale, 0.5),
                            multiply(imaginary_before_scale, 0.5),
                        )
                    }
                };

                let other_real = subtract(ar, new_real);
                let other_imaginary = subtract(ai, new_imaginary);
                values[input_a] = new_real;
                values[input_a + 1] = new_imaginary;
                values[input_b] = other_real;
                values[input_b + 1] = other_imaginary;
            }
        }
        half /= 2;
        group_count *= 2;
    }
}

#[cfg(test)]
mod tests {
    use super::fft_in_place;
    use serde_json::Value;
    use sha2::{Digest, Sha256};
    use std::path::PathBuf;

    const FIXTURE_DIRECTORY: &str = "tests/fixtures/long_transform_refresh";
    const CANARY_INPUT_SHA256: &str =
        "f8043b255bc1bba291584fba156b51a74ca3911ce84e095662aa11c44f2f92ae";
    const CANARY_EXPECTED_SHA256: &str =
        "ad457c9cc79bdd11522cdc2ca69529fba8442abf690a811789890575d9b906b2";

    fn fixture_bytes(name: &str) -> Vec<u8> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(FIXTURE_DIRECTORY)
            .join(name);
        std::fs::read(path).unwrap()
    }

    fn sha256(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    #[test]
    fn sixteen_complex_group_index_canary_matches_bound_outputs_and_preserves_tail() {
        let input_bytes = fixture_bytes("FFT-CANARY-INPUT.json");
        let expected_bytes = fixture_bytes("FFT-CANARY-EXPECTED.json");
        let input: Value = serde_json::from_slice(&input_bytes).unwrap();
        let expected: Value = serde_json::from_slice(&expected_bytes).unwrap();
        assert_eq!(sha256(&input_bytes), CANARY_INPUT_SHA256);
        assert_eq!(sha256(&expected_bytes), CANARY_EXPECTED_SHA256);
        assert_eq!(input["expected_used_as_seed"], false);
        assert_eq!(input["recipe"]["complex_count"], 16);
        assert_eq!(input["recipe"]["float_count"], 32);
        assert_eq!(input["recipe"]["exact_binary32"], true);

        let mut values = [0.0_f32; 33];
        for (word_index, destination) in values[..32].iter_mut().enumerate() {
            let residue = u8::try_from(word_index * 5 % 17).unwrap();
            let numerator = i16::from(residue) - 8;
            *destination = f32::from(numerator) / 16.0;
        }
        values[32] =
            f32::from_bits(u32::try_from(expected["tail_canary"].as_u64().unwrap()).unwrap());
        fft_in_place(&mut values[..32], 4);

        let expected_words = expected["output_bits"].as_array().unwrap();
        assert_eq!(expected_words.len(), 32);
        for (index, (actual, expected)) in values[..32].iter().zip(expected_words).enumerate() {
            assert_eq!(
                actual.to_bits(),
                u32::try_from(expected.as_u64().unwrap()).unwrap(),
                "FFT index canary output {index}"
            );
        }
        assert_eq!(values[32].to_bits(), 305_419_896);
        assert_eq!(expected["return_value"], "0");

        let canary_check: Value =
            serde_json::from_slice(&fixture_bytes("FFT-CANARY-CHECK.json")).unwrap();
        assert_eq!(canary_check["native_values"], 32);
        assert_eq!(canary_check["closed_form_match"], true);
        assert_eq!(
            canary_check["incorrect_variant_mismatch_counts"]["preincrement"],
            24
        );
        assert_eq!(
            canary_check["incorrect_variant_mismatch_counts"]["per_pair"],
            16
        );
        assert_eq!(
            canary_check["incorrect_variant_mismatch_counts"]["no_stage_reset"],
            24
        );
    }
}
