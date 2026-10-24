// pattern: Functional Core
//! Four-lane 256-sample binary32 mean-square reducer.

use crate::numeric::{FiniteF32, r32_add, r32_mul};

/// Required sample count for the selected mean-square helper.
pub const MEAN_SQUARE_SAMPLE_COUNT: usize = 256;

/// Failure from the checked four-lane mean-square calculation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeanSquareError {
    NonFiniteInput,
    NonFiniteIntermediate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MeanSquareTrace {
    lane_accumulation: [[u32; 64]; 4],
    final_lanes: [u32; 4],
    reduction_trace: [u32; 5],
    mean: u32,
}

/// Returns one selected-profile 256-sample four-lane mean-square value.
///
/// The caller supplies exactly 256 finite normal binary32 samples with magnitude at most
/// one, as in the washed input profile. The function has no persistent state and does not
/// produce gain peaks, cross-block history, or a selector. Inputs outside the declared
/// profile are not assigned a native error/status outcome.
///
/// # Panics
///
/// Panics if a sample or intermediate violates the finite bounded profile.
pub fn mean_square_256(samples: &[f32; MEAN_SQUARE_SAMPLE_COUNT]) -> f32 {
    f32::from_bits(mean_square_trace(samples).mean)
}

/// Returns mean-square bits with finite-input and intermediate failures reported.
pub fn checked_mean_square_256(
    samples: &[f32; MEAN_SQUARE_SAMPLE_COUNT],
) -> Result<u32, MeanSquareError> {
    checked_mean_square_trace(samples).map(|trace| trace.mean)
}

fn mean_square_trace(samples: &[f32; MEAN_SQUARE_SAMPLE_COUNT]) -> MeanSquareTrace {
    checked_mean_square_trace(samples).expect("selected finite mean-square profile is bounded")
}

fn checked_mean_square_trace(
    samples: &[f32; MEAN_SQUARE_SAMPLE_COUNT],
) -> Result<MeanSquareTrace, MeanSquareError> {
    let positive_zero = FiniteF32::new(0.0).map_err(|_| MeanSquareError::NonFiniteIntermediate)?;
    let mut lanes = [positive_zero; 4];
    let mut lane_accumulation_bits = [[0; 64]; 4];

    for (index, sample) in samples.iter().enumerate() {
        let sample = FiniteF32::new(*sample).map_err(|_| MeanSquareError::NonFiniteInput)?;
        let square = r32_mul(sample, sample).map_err(|_| MeanSquareError::NonFiniteIntermediate)?;
        let lane = index % 4;
        lanes[lane] =
            r32_add(lanes[lane], square).map_err(|_| MeanSquareError::NonFiniteIntermediate)?;
        lane_accumulation_bits[lane][index / 4] = lanes[lane].to_bits();
    }

    let final_lane_bits = lanes.map(FiniteF32::to_bits);
    let mut total = positive_zero;
    let mut reduction_trace_bits = [0; 5];
    reduction_trace_bits[0] = total.to_bits();
    for (index, lane) in lanes.into_iter().enumerate() {
        total = r32_add(total, lane).map_err(|_| MeanSquareError::NonFiniteIntermediate)?;
        reduction_trace_bits[index + 1] = total.to_bits();
    }

    let mean = FiniteF32::new(total.value() / 256.0_f32)
        .map_err(|_| MeanSquareError::NonFiniteIntermediate)?;
    Ok(MeanSquareTrace {
        lane_accumulation: lane_accumulation_bits,
        final_lanes: final_lane_bits,
        reduction_trace: reduction_trace_bits,
        mean: mean.to_bits(),
    })
}

#[cfg(test)]
mod tests {
    use super::{MEAN_SQUARE_SAMPLE_COUNT, mean_square_256, mean_square_trace};
    use serde_json::Value;
    use sha2::{Digest, Sha256};
    use std::path::PathBuf;

    const FIXTURE_DIRECTORY: &str = "tests/fixtures/four_lane_mean_square";
    const PACKAGE_MANIFEST_SHA256: &str =
        "7a5da39f1b20b8ffcebfe0925dd236df36c2d29552362985680c3f3b8505dbd1";
    const INDEPENDENT_CHECK_SHA256: &str =
        "fac6afc662b532d19e0e7e3d4932236e3e8235c397331fcd0017dfdbc634a7c0";
    const PACKAGE_PAYLOADS: [(&str, usize, &str); 6] = [
        (
            "CONTRACT.md",
            2301,
            "661a26a39e9b77793c8bf97453627f396b5c59466c97c3d43fcd023789259efa",
        ),
        (
            "CONTRACT.json",
            2466,
            "c8b2f7e5489d1258a7a71c1e618b54031b304d24067c20f8f27a69082d390b80",
        ),
        (
            "INPUTS.json",
            34032,
            "a755df024cfd968e4a4f067d8a43a7a20caf136fad4b55ac65aaa9383846e657",
        ),
        (
            "EXPECTED.json",
            42295,
            "75489266e33fd2d65eb5b53c6b7906e533a1158a613dbb5b7e9d19ce0a5360c3",
        ),
        (
            "RUNTIME-RECEIPT.json",
            1970,
            "1da17a0ddf289fbcb6623064b024c166f4419e485c737c51d9e6d55466676f16",
        ),
        (
            "PROVENANCE.json",
            2875,
            "17916c7a4ce1fc662e46658206ee77b185f81ad0fd355265bceeae2e3ecd9b02",
        ),
    ];

    fn fixture_bytes(name: &str) -> Vec<u8> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(FIXTURE_DIRECTORY)
            .join(name);
        std::fs::read(path).unwrap()
    }

    fn fixture_json(name: &str) -> Value {
        serde_json::from_slice(&fixture_bytes(name)).unwrap()
    }

    fn sha256(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    fn expected_u32s(value: &Value) -> Vec<u32> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
            .collect()
    }

    #[test]
    fn package_payloads_and_check_identity_are_hash_bound() {
        let check_bytes = fixture_bytes("INDEPENDENT-CHECK.json");
        let check: Value = serde_json::from_slice(&check_bytes).unwrap();
        assert_eq!(sha256(&check_bytes), INDEPENDENT_CHECK_SHA256);
        assert_eq!(check["manifest_sha256"], PACKAGE_MANIFEST_SHA256);
        assert_eq!(check["payload_count"], 6);
        assert_eq!(check["payload_hashes_match"], true);
        assert_eq!(check["source_manifest_check_and_payloads_match"], true);
        assert_eq!(check["input_recipes"], 7);
        assert_eq!(check["input_words_replayed"], 1792);
        assert_eq!(check["lane_accumulation_steps_replayed"], 1792);
        assert_eq!(check["native_final_mean_calls"], 7);
        assert_eq!(
            check["independent_formula_replay"],
            "PASS all lane and reduction bits"
        );
        assert_eq!(check["native_lane_register_capture"], "NOT_RUN");
        assert_eq!(check["independent_product_audio_integration"], "NOT_RUN");
        assert_eq!(check["whole_f09_closed"], false);
        assert_eq!(check["whole_implementation_ready"], "NO");

        for (name, expected_size, expected_hash) in PACKAGE_PAYLOADS {
            let bytes = fixture_bytes(name);
            assert_eq!(bytes.len(), expected_size, "{name} size");
            assert_eq!(sha256(&bytes), expected_hash, "{name} SHA-256");
        }
    }

    #[test]
    fn seven_hash_bound_inputs_match_all_lane_steps_ordered_reduction_and_runtime_mean() {
        let inputs_bytes = fixture_bytes("INPUTS.json");
        let expected_bytes = fixture_bytes("EXPECTED.json");
        let inputs: Value = serde_json::from_slice(&inputs_bytes).unwrap();
        let expected: Value = serde_json::from_slice(&expected_bytes).unwrap();
        let provenance = fixture_json("PROVENANCE.json");
        let recipes = inputs["recipes"].as_array().unwrap();
        let cases = expected["cases"].as_array().unwrap();
        assert_eq!(inputs["expected_used_as_seed"], false);
        assert_eq!(
            provenance["source_manifest_sha256"],
            "6a10c327a39f806580f61203d6186ee9919a1b7b86872b0e3a0b8f4e2f48b47d"
        );
        assert_eq!(
            provenance["source_independent_check_sha256"],
            "a07ab929cdab22826a8f3279f209989e95d14302303161267d3dfed1f50f09f2"
        );
        assert_eq!(recipes.len(), 7);
        assert_eq!(cases.len(), recipes.len());

        let mut input_words_checked = 0;
        let mut lane_steps_checked = 0;
        for (recipe, expected_case) in recipes.iter().zip(cases) {
            assert_eq!(recipe["count"], MEAN_SQUARE_SAMPLE_COUNT);
            assert_eq!(recipe["stride"], 1);
            let input_words = expected_u32s(&recipe["input_bits"]);
            assert_eq!(input_words.len(), MEAN_SQUARE_SAMPLE_COUNT);
            let encoded_input_bytes = input_words
                .iter()
                .flat_map(|word| word.to_le_bytes())
                .collect::<Vec<_>>();
            assert_eq!(
                sha256(&encoded_input_bytes),
                recipe["input_sha256"].as_str().unwrap()
            );
            let input_word_count = input_words.len();
            let samples: [f32; MEAN_SQUARE_SAMPLE_COUNT] = input_words
                .iter()
                .map(|word| f32::from_bits(*word))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            let trace = mean_square_trace(&samples);
            assert_eq!(
                trace.lane_accumulation.len(),
                expected_case["computed_lane_accumulation_bits"]
                    .as_array()
                    .unwrap()
                    .len()
            );
            for lane in 0..4 {
                assert_eq!(
                    trace.lane_accumulation[lane].as_slice(),
                    expected_u32s(&expected_case["computed_lane_accumulation_bits"][lane])
                        .as_slice(),
                    "{} lane {lane} accumulation trace",
                    recipe["kind"]
                );
                lane_steps_checked += trace.lane_accumulation[lane].len();
            }
            assert_eq!(
                trace.final_lanes.as_slice(),
                expected_u32s(&expected_case["computed_final_lane_bits"]).as_slice(),
                "{} final lanes",
                recipe["kind"]
            );
            assert_eq!(
                trace.reduction_trace.as_slice(),
                expected_u32s(&expected_case["computed_reduction_trace_bits"]).as_slice(),
                "{} ordered reduction",
                recipe["kind"]
            );
            assert_eq!(
                trace.mean,
                u32::try_from(expected_case["computed_mean_bits"].as_u64().unwrap()).unwrap(),
                "{} computed mean",
                recipe["kind"]
            );
            assert_eq!(
                trace.mean,
                u32::try_from(expected_case["native_mean_bits"].as_u64().unwrap()).unwrap(),
                "{} native final mean",
                recipe["kind"]
            );
            assert_eq!(
                expected_case["intermediate_grade"],
                "COMPUTED_FROM_INPUT_AND_STATIC_RULES_NOT_NATIVE_REGISTER_CAPTURE"
            );
            assert_eq!(expected_case["native_final_grade"], "RUNTIME_CONFIRMED");
            assert_eq!(mean_square_256(&samples).to_bits(), trace.mean);
            input_words_checked += input_word_count;
        }

        assert_eq!(input_words_checked, 1792);
        assert_eq!(lane_steps_checked, 1792);

        let partition_order = &cases[5];
        assert_ne!(
            partition_order["computed_mean_bits"],
            partition_order["global_serial_bits"]
        );
        let reduction_order = &cases[6];
        assert_ne!(
            reduction_order["computed_mean_bits"],
            reduction_order["balanced_lane_bits"]
        );
        assert_ne!(
            reduction_order["computed_mean_bits"],
            reduction_order["reversed_lane_bits"]
        );
    }
}
