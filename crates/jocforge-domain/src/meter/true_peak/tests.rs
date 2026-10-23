// pattern: Functional Core
use super::{
    CHANNEL_COUNT, HOST_SCALE_WORD, SOURCE_SAMPLES, TruePeakError, TruePeakState,
    accumulate_normalized_energy, energy_metric, first_absolute_maximum, multiply_words,
    normalize_energy_window, ordered_energy_reduction, ordered_log, process_true_peak_block,
    sample_peak_metric, selected_log_constants, true_peak_metric,
};
use crate::numeric::{FiniteF32, r32_mul};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const TRUEPEAK_SPEC_SHA256: &str =
    "e0491288c3a9beb59ecb7b017d505ee16eb9f401d53bd0fd918573644c1876b3";
const TRUEPEAK_PARAMETERS_SHA256: &str =
    "f67e6fe86533af74acc261bf3908890e77a461198116c64b624dc91b69049757";
const TRUEPEAK_VECTORS_SHA256: &str =
    "510b980acf92519e0fbced2cf79b6b9f72dc47dcc8316f59506048288982d528";
const TRUEPEAK_RESULT_SHA256: &str =
    "337c80ee3826f83ca5ca4b769200b3d370b5ba3a69b85a185360f3bf3d038bd8";
const TRUEPEAK_REVIEW_SHA256: &str =
    "261e043ba80c2d8011ae638c345fd811aa26d60838696553bd8f80f4c9b032c4";
const LOG_SPEC_SHA256: &str = "c8a9aec8c931391bfbb4ecaca01a8e5cbec3ba680899f516727c6574a170e38f";
const LOG_PARAMETERS_SHA256: &str =
    "044d42327f471b183b0a29caaa9db85f694a40c436a5afe3b83734dbf29bc266";
const LOG_VECTORS_SHA256: &str = "909ea8e6abc24f37a9e180871b1c9d4b67e151bb4dc70b439a68d40d06f077f9";
const LOG_REVIEW_SHA256: &str = "66d5d1cffac1f60066747ba8dbd4647e7403958c8002c04ae488095bf527d1b0";

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/meter_true_peak")
}

fn log_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/meter_log")
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_json(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(fixture_root().join(name)).unwrap()).unwrap()
}

fn read_log_json(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(log_fixture_root().join(name)).unwrap()).unwrap()
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

#[test]
fn authorized_parameters_specs_and_nine_recipes_are_hash_bound() {
    let manifest_bytes = std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/f01e_provider/MANIFEST.json"),
    )
    .unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    assert_eq!(sha256(&manifest_bytes), PACKAGE_MANIFEST_SHA256);

    for (path, bytes, expected_hash, local_name) in [
        (
            "specs/06-true-peak-cascade-and-metrics.md",
            9_824,
            TRUEPEAK_SPEC_SHA256,
            "true-peak-spec.md",
        ),
        (
            "parameters/meter-truepeak-authorized-v1.json",
            2_584,
            TRUEPEAK_PARAMETERS_SHA256,
            "truepeak-parameters.json",
        ),
        (
            "test-vectors/METER-TRUEPEAK-AUTHORIZED-VERIFICATION.json",
            2_078_540,
            TRUEPEAK_VECTORS_SHA256,
            "input-vectors.json",
        ),
        (
            "test-vectors/METER-TRUEPEAK-AUTHORIZED-RESULT.json",
            2_198,
            TRUEPEAK_RESULT_SHA256,
            "verification-result.json",
        ),
        (
            "test-vectors/METER-TRUEPEAK-LOCAL-REVIEW-STATUS.json",
            43_052,
            TRUEPEAK_REVIEW_SHA256,
            "local-review-status.json",
        ),
        (
            "specs/06-meter-log-and-peak.md",
            7_525,
            LOG_SPEC_SHA256,
            "log-and-peak-spec.md",
        ),
        (
            "parameters/meter-log-mathematical-v1.json",
            780,
            LOG_PARAMETERS_SHA256,
            "log-parameters.json",
        ),
    ] {
        let entry = manifest["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["file"] == path)
            .unwrap();
        let bytes_local = std::fs::read(fixture_root().join(local_name)).unwrap();
        assert_eq!(entry["bytes"], bytes);
        assert_eq!(entry["sha256"], expected_hash);
        assert_eq!(u64::try_from(bytes_local.len()).unwrap(), bytes);
        assert_eq!(sha256(&bytes_local), expected_hash);
    }

    let vectors = read_json("input-vectors.json");
    assert_eq!(vectors["parameter_sha256"], TRUEPEAK_PARAMETERS_SHA256);
    assert_eq!(vectors["cases"].as_array().unwrap().len(), 9);
    assert_eq!(vectors["delivery_scope"], "INTERNAL_ONLY");
    assert_eq!(
        vectors["public_or_third_party_distribution"],
        "NOT_AUTHORIZED"
    );

    let log_vectors_bytes =
        std::fs::read(log_fixture_root().join("METER-LOG-GENERATION-VERIFICATION.json")).unwrap();
    let log_review_bytes =
        std::fs::read(log_fixture_root().join("METER-LOG-LOCAL-REVIEW-STATUS.json")).unwrap();
    for (file, bytes, hash, local_bytes) in [
        (
            "test-vectors/METER-LOG-GENERATION-VERIFICATION.json",
            18_188,
            LOG_VECTORS_SHA256,
            &log_vectors_bytes,
        ),
        (
            "test-vectors/METER-LOG-LOCAL-REVIEW-STATUS.json",
            8_941,
            LOG_REVIEW_SHA256,
            &log_review_bytes,
        ),
    ] {
        let entry = manifest["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["file"] == file)
            .unwrap();
        assert_eq!(entry["bytes"], bytes);
        assert_eq!(entry["sha256"], hash);
        assert_eq!(u64::try_from(local_bytes.len()).unwrap(), bytes);
        assert_eq!(sha256(local_bytes), hash);
    }
}

#[test]
fn f06l_log_energy_peak_and_accumulator_vectors_match() {
    let vectors = read_log_json("METER-LOG-GENERATION-VERIFICATION.json");
    let review = read_log_json("METER-LOG-LOCAL-REVIEW-STATUS.json");
    assert_eq!(vectors["schema"], "meter-log-generation-verification.v2");
    assert_eq!(
        review["verdict"],
        "MLG_REV_001_002_CLOSED_LOCAL_SCOPE_NO_NEW_LOCAL_ISSUES"
    );
    assert_eq!(
        review["delivery_scope"],
        "INTERNAL_ONLY_NO_PUBLIC_OR_THIRD_PARTY_DISTRIBUTION"
    );
    assert_eq!(review["implementation_ready"], "NO");
    let parameters = selected_log_constants().unwrap();
    let cases = vectors["cases"].as_array().unwrap();
    let mut counts = [0; 7];

    for case in cases {
        let id = case["id"].as_str().unwrap();
        let input = &case["input"];
        let expected = &case["expected"];
        if id.starts_with("log_") {
            counts[0] += 1;
            let value = (input["value"].as_f64().unwrap() as f32).to_bits();
            let offset = i32::try_from(input["offset"].as_i64().unwrap()).unwrap();
            assert_eq!(
                ordered_log(value, offset, &parameters).unwrap(),
                expected.as_u64().unwrap() as u32,
                "{id}"
            );
        } else if id.starts_with("energy_") {
            counts[1] += 1;
            let mantissa = (input["mantissa"].as_f64().unwrap() as f32).to_bits();
            let offset = i32::try_from(input["offset"].as_i64().unwrap()).unwrap();
            assert_eq!(
                energy_metric(mantissa, offset, &parameters).unwrap(),
                expected.as_u64().unwrap() as u32,
                "{id}"
            );
        } else if id.starts_with("sample_peak_") {
            counts[2] += 1;
            let sample_count = usize::try_from(input["sample_count"].as_u64().unwrap()).unwrap();
            assert_eq!(sample_count, SOURCE_SAMPLES, "{id}");
            let peak = input["peak"].as_f64().unwrap() as f32;
            let mut samples = vec![0; sample_count];
            let negative_position =
                usize::try_from(input["negative_position"].as_u64().unwrap()).unwrap();
            let positive_position =
                usize::try_from(input["positive_equal_position"].as_u64().unwrap()).unwrap();
            samples[negative_position] = (-peak).to_bits();
            samples[positive_position] = peak.to_bits();
            let (maximum, first_index) = first_absolute_maximum(&samples).unwrap();
            let internal = sample_peak_metric(maximum, &parameters).unwrap();
            let host = multiply_words(internal, HOST_SCALE_WORD).unwrap();
            assert_eq!(
                maximum,
                expected["maximum_abs_sample_word"].as_u64().unwrap() as u32,
                "{id} maximum"
            );
            assert_eq!(
                first_index,
                usize::try_from(expected["first_index"].as_u64().unwrap()).unwrap(),
                "{id} first index"
            );
            assert_eq!(
                internal,
                expected["internal_word"].as_u64().unwrap() as u32,
                "{id} internal metric"
            );
            assert_eq!(
                host,
                expected["host_word"].as_u64().unwrap() as u32,
                "{id} host metric"
            );
        } else if id.starts_with("true_peak_") {
            counts[3] += 1;
            let raw_peak = (input["raw_peak"].as_f64().unwrap() as f32).to_bits();
            let internal = true_peak_metric(raw_peak, &parameters).unwrap();
            let host = multiply_words(internal, HOST_SCALE_WORD).unwrap();
            assert_eq!(
                internal,
                expected["internal_word"].as_u64().unwrap() as u32,
                "{id} internal metric"
            );
            assert_eq!(
                host,
                expected["host_word"].as_u64().unwrap() as u32,
                "{id} host metric"
            );
        } else if id.starts_with("accumulator_") {
            counts[4] += 1;
            let mantissa = (input["mantissa"].as_f64().unwrap() as f32).to_bits();
            let exponent = i32::try_from(input["exponent"].as_i64().unwrap()).unwrap();
            let power_sum = (input["power_sum"].as_f64().unwrap() as f32).to_bits();
            let (actual_mantissa, actual_exponent) =
                accumulate_normalized_energy(mantissa, exponent, power_sum)
                    .unwrap_or_else(|error| panic!("{id}: {error:?}"));
            assert_eq!(
                actual_mantissa,
                expected[0].as_u64().unwrap() as u32,
                "{id} mantissa"
            );
            assert_eq!(
                actual_exponent,
                i32::try_from(expected[1].as_i64().unwrap()).unwrap(),
                "{id} exponent"
            );
        } else if id.starts_with("window_") {
            counts[5] += 1;
        } else if id.starts_with("reduction_") {
            counts[6] += 1;
            let exponents = input["exponents"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| i32::try_from(value.as_i64().unwrap()).unwrap())
                .collect::<Vec<_>>();
            let weights = input["weights"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| (value.as_f64().unwrap() as f32).to_bits())
                .collect::<Vec<_>>();
            let energies = input["energies"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| (value.as_f64().unwrap() as f32).to_bits())
                .collect::<Vec<_>>();
            let (actual_mantissa, actual_exponent) =
                ordered_energy_reduction(&exponents, &weights, &energies)
                    .unwrap_or_else(|error| panic!("{id}: {error:?}"));
            assert_eq!(
                actual_mantissa,
                expected[0].as_u64().unwrap() as u32,
                "{id} mantissa"
            );
            assert_eq!(
                actual_exponent,
                i32::try_from(expected[1].as_i64().unwrap()).unwrap(),
                "{id} exponent"
            );
        } else {
            panic!("unexpected vector group {id}");
        }
    }

    assert_eq!(counts, [14, 8, 6, 8, 7, 6, 7]);
    assert_eq!(vectors["cases"].as_array().unwrap().len(), 56);
    assert_eq!(vectors["rejection_cases"].as_array().unwrap().len(), 3);
    assert_eq!(vectors["constant_entries"], 11);
}

#[test]
fn normalized_energy_accumulator_rejects_negative_and_nonfinite_inputs() {
    for (mantissa, exponent, power_sum) in [
        ((-1.0_f32).to_bits(), 4, 0.5_f32.to_bits()),
        (0.5_f32.to_bits(), 4, (-1.0_f32).to_bits()),
        (f32::NAN.to_bits(), 4, 0.5_f32.to_bits()),
        (0.5_f32.to_bits(), 4, f32::INFINITY.to_bits()),
    ] {
        assert_eq!(
            accumulate_normalized_energy(mantissa, exponent, power_sum),
            Err(TruePeakError::Input)
        );
    }

    assert!(accumulate_normalized_energy(0x8000_0000, 4, 0x8000_0000).is_ok());
}

#[test]
fn f06l_window_normalization_cases_and_entry_rejections_match() {
    let vectors = read_log_json("METER-LOG-GENERATION-VERIFICATION.json");
    let mut successes = 0;
    for case in vectors["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        if !id.starts_with("window_") {
            continue;
        }
        successes += 1;
        let input = &case["input"];
        let mantissa_word = (input["mantissa"].as_f64().unwrap() as f32).to_bits();
        let exponent = i32::try_from(input["exponent"].as_i64().unwrap()).unwrap();
        let window_samples = u32::try_from(input["window_samples"].as_u64().unwrap()).unwrap();
        let (actual_mantissa, actual_exponent) =
            normalize_energy_window(mantissa_word, exponent, window_samples)
                .unwrap_or_else(|error| panic!("{id}: {error:?}"));
        assert_eq!(
            actual_mantissa,
            case["expected"][0].as_u64().unwrap() as u32,
            "{id} mantissa"
        );
        assert_eq!(
            actual_exponent,
            i32::try_from(case["expected"][1].as_i64().unwrap()).unwrap(),
            "{id} exponent"
        );
    }
    assert_eq!(successes, 6);

    let rejections = vectors["rejection_cases"].as_array().unwrap();
    assert_eq!(rejections.len(), 3);
    for case in rejections {
        let input = &case["input"];
        assert!(case["expected_rejected"].as_bool().unwrap());
        if input["exponent"].as_i64().is_none() {
            assert_eq!(case["id"], "window_fractional_e");
            continue;
        }
        let mantissa_word = (input["mantissa"].as_f64().unwrap() as f32).to_bits();
        let exponent = i32::try_from(input["exponent"].as_i64().unwrap()).unwrap();
        let window_samples = u32::try_from(input["window_samples"].as_u64().unwrap()).unwrap();
        assert_eq!(
            normalize_energy_window(mantissa_word, exponent, window_samples),
            Err(TruePeakError::Input),
            "{}",
            case["id"]
        );
    }

    for (mantissa, exponent, window_samples) in [
        (0x3f00_0000, 4, 0),
        (0x3f00_0000, 4, 48_001),
        (f32::NAN.to_bits(), 4, 1024),
    ] {
        assert_eq!(
            normalize_energy_window(mantissa, exponent, window_samples),
            Err(TruePeakError::Input)
        );
    }
}

#[test]
fn ordered_energy_reduction_rejects_invalid_shapes_and_domains() {
    assert_eq!(
        ordered_energy_reduction(&[0], &[], &[0]),
        Err(TruePeakError::Input)
    );
    let too_many_exponents = vec![0; 17];
    let too_many_values = vec![0; 17];
    assert_eq!(
        ordered_energy_reduction(&too_many_exponents, &too_many_values, &too_many_values),
        Err(TruePeakError::Input)
    );
    for exponent in [-1, 61] {
        assert_eq!(
            ordered_energy_reduction(&[exponent], &[0x3f80_0000], &[0x3f00_0000]),
            Err(TruePeakError::Input)
        );
    }
    for (weight, energy) in [
        ((-1.0_f32).to_bits(), 0x3f00_0000),
        (0x3f80_0000, (-1.0_f32).to_bits()),
        (f32::NAN.to_bits(), 0x3f00_0000),
        (0x3f80_0000, f32::INFINITY.to_bits()),
    ] {
        assert_eq!(
            ordered_energy_reduction(&[0], &[weight], &[energy]),
            Err(TruePeakError::Input)
        );
    }
    assert!(ordered_energy_reduction(&[0], &[0x8000_0000], &[0x8000_0000]).is_ok());
    assert_eq!(
        ordered_energy_reduction(&[0], &[(3.5e38_f64 as f32).to_bits()], &[0x3f00_0000]),
        Err(TruePeakError::Input)
    );
}

#[test]
fn nine_authorized_sequences_match_output_state_peak_and_programme_vectors() {
    let vectors = read_json("input-vectors.json");
    let recipes = &vectors["source_recipes"];
    let mut state = TruePeakState::cold();
    for case in vectors["cases"].as_array().unwrap() {
        let input = build_source_block(
            recipes,
            case["input"]["source_recipe_id"].as_u64().unwrap(),
            &case["input"]["row_scale_rationals"],
        )
        .unwrap_or_else(|error| panic!("case {} source construction: {error:?}", case["case_id"]));
        assert_eq!(
            sha256(&source_bytes(&input)),
            case["source_f32le_sha256"].as_str().unwrap(),
            "source recipe {}",
            case["case_id"]
        );

        if case["input"]["reset_before_case"] == true {
            state = TruePeakState::cold();
            if case["sequence_id"] == "independent_nonzero_lfe_state" {
                let lane = &mut state.lanes[3].words;
                for (index, sample) in lane.iter_mut().enumerate() {
                    let numerator = i16::try_from(index % 7).unwrap() - 3;
                    *sample = (f32::from(numerator) / 32.0).to_bits();
                }
            }
        }
        let previous_lfe_history = state.lanes[3].words.clone();
        let output = process_true_peak_block(&state, &input).unwrap();

        for (channel, expected_hash) in case["expected_output_sha256s"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            assert_eq!(
                sha256(&word_bytes(&output.measurements[channel])),
                expected_hash.as_str().unwrap(),
                "output channel {channel}, case {}",
                case["case_id"]
            );
        }
        assert_eq!(
            hex(&output.next_state.history_f32le_bytes()),
            case["expected_next_state_f32le_hex"].as_str().unwrap(),
            "complete history state, case {}",
            case["case_id"]
        );
        assert_eq!(
            sha256(&output.next_state.history_f32le_bytes()),
            case["expected_next_state_f32le_sha256"].as_str().unwrap()
        );
        assert_peak_projection(&output, &case["expected_peak_projection"]);
        assert_programme_projection(&output.next_state, &case["expected_programme_after"]);
        assert_eq!(
            output.next_state.lanes[3].words == previous_lfe_history,
            case["LFE_state_unchanged"].as_bool().unwrap()
        );
        state = output.next_state;
    }
}

fn assert_peak_projection(output: &super::TruePeakBlockResult, expected: &Value) {
    let metrics = &output.lane_metrics;
    let expected = expected.as_array().unwrap();
    assert_eq!(metrics.len(), expected.len());
    for (actual, expected) in metrics.iter().zip(expected) {
        assert_eq!(
            actual.raw_peak_word,
            expected["raw_peak_word"].as_u64().unwrap() as u32
        );
        assert_eq!(
            actual.first_absolute_maximum_index,
            usize::try_from(expected["first_absolute_maximum_index"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            actual.grouped_reported_index,
            i32::try_from(expected["grouped_reported_index"].as_i64().unwrap()).unwrap()
        );
        assert_eq!(actual.factor, expected["factor"].as_u64().unwrap() as u8);
        assert_eq!(
            actual.true_peak_internal_word,
            expected["true_peak_internal_word"].as_u64().unwrap() as u32
        );
        assert_eq!(
            actual.true_peak_host_word,
            expected["true_peak_host_word"].as_u64().unwrap() as u32
        );
        assert_eq!(
            actual.sample_peak_raw_word,
            expected["sample_peak_raw_word"].as_u64().unwrap() as u32
        );
        assert_eq!(
            actual.sample_peak_first_index,
            usize::try_from(expected["sample_peak_first_index"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            actual.sample_peak_internal_word,
            expected["sample_peak_internal_word"].as_u64().unwrap() as u32
        );
        assert_eq!(
            actual.sample_peak_host_word,
            expected["sample_peak_host_word"].as_u64().unwrap() as u32
        );
    }
}

fn assert_programme_projection(state: &TruePeakState, expected: &Value) {
    assert_eq!(
        state.true_peak_internal_words.as_slice(),
        expected["true_peak_internal_words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        state.sample_peak_internal_words.as_slice(),
        expected["sample_peak_internal_words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        state.final_true_peak_host_word().unwrap(),
        expected["true_peak_host_word"].as_u64().unwrap() as u32
    );
    assert_eq!(
        state.final_sample_peak_host_word().unwrap(),
        expected["sample_peak_host_word"].as_u64().unwrap() as u32
    );
}

fn source_bytes(input: &[Vec<u32>]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(input.len() * 1024 * 4);
    for row in input {
        bytes.extend_from_slice(&word_bytes(row));
    }
    bytes
}

fn word_bytes(words: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(words.len() * 4);
    for word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes
}

#[allow(clippy::cast_possible_truncation)]
fn build_source_block(
    recipes: &Value,
    recipe_id: u64,
    row_scales: &Value,
) -> Result<Vec<Vec<u32>>, TruePeakError> {
    let recipe = recipes[recipe_id.to_string()]
        .as_str()
        .ok_or(TruePeakError::Input)?;
    let base = match recipe {
        "1024 positive zero" | "1024 positive zero; prior block history remains" => {
            vec![0; SOURCE_SAMPLES]
        }
        "i=0 positive unity impulse, other samples positive zero" => {
            let mut row = vec![0; SOURCE_SAMPLES];
            row[0] = 0x3f80_0000;
            row
        }
        "i=1023 negative three-quarter impulse, other samples positive zero" => {
            let mut row = vec![0; SOURCE_SAMPLES];
            row[SOURCE_SAMPLES - 1] = 0xbf40_0000;
            row
        }
        "R32((((37*i+11)%257)-128)/256), integer i=0..1023" => (0..SOURCE_SAMPLES)
            .map(|index| {
                let numerator = i16::try_from((37 * index + 11) % 257).unwrap() - 128;
                (f32::from(numerator) / 256.0).to_bits()
            })
            .collect(),
        "R32((-1)^i/2), integer i=0..1023" => (0..SOURCE_SAMPLES)
            .map(|index| {
                if index % 2 == 0 {
                    0x3f00_0000
                } else {
                    0xbf00_0000
                }
            })
            .collect(),
        "1024 negative zero" => vec![0x8000_0000; SOURCE_SAMPLES],
        "R32(R64(R64(1e-24)*R64((i%7)-3))), integer i=0..1023; decimal 1e-24 first rounds to binary64" => {
            (0..SOURCE_SAMPLES)
                .map(|index| {
                    let integer = i16::try_from(index % 7).unwrap() - 3;
                    ((1.0e-24_f64 * f64::from(integer)) as f32).to_bits()
                })
                .collect()
        }
        _ => return Err(TruePeakError::Input),
    };
    let scales = row_scales.as_array().ok_or(TruePeakError::Input)?;
    if scales.len() != CHANNEL_COUNT {
        return Err(TruePeakError::Input);
    }
    let mut rows = Vec::with_capacity(CHANNEL_COUNT);
    for scale in scales {
        let pair = scale.as_array().ok_or(TruePeakError::Input)?;
        if pair.len() != 2 {
            return Err(TruePeakError::Input);
        }
        let numerator = i16::try_from(pair[0].as_i64().ok_or(TruePeakError::Input)?)
            .map_err(|_| TruePeakError::Input)?;
        let denominator = i16::try_from(pair[1].as_i64().ok_or(TruePeakError::Input)?)
            .map_err(|_| TruePeakError::Input)?;
        if denominator == 0 {
            return Err(TruePeakError::Input);
        }
        let scale = f32::from(numerator) / f32::from(denominator);
        let scale = FiniteF32::new(scale).map_err(|_| TruePeakError::Input)?;
        let row = base
            .iter()
            .map(|word| {
                let sample =
                    FiniteF32::new(f32::from_bits(*word)).map_err(|_| TruePeakError::Input)?;
                r32_mul(sample, scale)
                    .map(FiniteF32::to_bits)
                    .map_err(|_| TruePeakError::Input)
            })
            .collect::<Result<Vec<_>, _>>()?;
        rows.push(row);
    }
    Ok(rows)
}
