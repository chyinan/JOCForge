// pattern: Functional Core
use jocforge_domain::meter::integrated_meter::{
    IntegratedMeterError, IntegratedMeterInput, IntegratedMeterParameters, IntegratedMeterState,
    integrated_histogram_bin, process_integrated_meter,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const SCHEDULE_VECTORS_SHA256: &str =
    "a565e513970670967aa3aebd0372f9dbb5f9bbab670a54744ae454926b3f2671";
const STATE_REVIEW_SHA256: &str =
    "020f88cafa892119bac5fea5bc808d251798e98aec69897c6ab068da80637307";
const FIXTURE_DIRECTORY: &str = "tests/fixtures/integrated_meter";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_json(path: &str) -> (Vec<u8>, Value) {
    let bytes = std::fs::read(crate_root().join(path)).unwrap();
    let value = serde_json::from_slice(&bytes).unwrap();
    (bytes, value)
}

fn f32_word(value: &Value) -> u32 {
    (value.as_f64().unwrap() as f32).to_bits()
}

fn assert_manifest_entry(manifest: &Value, file: &str, bytes: usize, hash: &str) {
    let entry = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["file"] == file)
        .unwrap();
    assert_eq!(entry["bytes"].as_u64(), Some(bytes as u64), "{file}");
    assert_eq!(entry["sha256"].as_str(), Some(hash), "{file}");
}

fn patch_state(mut state: IntegratedMeterState, patch: &Value) -> IntegratedMeterState {
    if let Some(value) = patch.get("counts_fill") {
        state = state.with_counts_fill(value.as_u64().unwrap() as u32);
    }
    if let Some(value) = patch.get("absolute_count") {
        state = state.with_absolute_count(value.as_u64().unwrap() as u32);
    }
    if let Some(value) = patch.get("absolute_mean") {
        state = state.with_absolute_mean_word(f32_word(value));
    }
    if let Some(value) = patch.get("absolute_exponent") {
        state = state.with_absolute_exponent(value.as_i64().unwrap() as i32);
    }
    state
}

#[test]
fn source_package_and_exact_case_scope_are_hash_bound() {
    let (manifest_bytes, manifest) = read_json("tests/fixtures/f01e_provider/MANIFEST.json");
    let (vector_bytes, vectors) = read_json(&format!(
        "{FIXTURE_DIRECTORY}/METER-INTEGRATED-SCHEDULE-VERIFICATION.json"
    ));
    let (review_bytes, review) = read_json(&format!(
        "{FIXTURE_DIRECTORY}/METER-STATE-CLOCK-LOCAL-REVIEW-STATUS.json"
    ));

    assert_eq!(sha256(&manifest_bytes), PACKAGE_MANIFEST_SHA256);
    assert_eq!(sha256(&vector_bytes), SCHEDULE_VECTORS_SHA256);
    assert_eq!(sha256(&review_bytes), STATE_REVIEW_SHA256);
    assert_manifest_entry(
        &manifest,
        "specs/06-integrated-meter-state.md",
        9_368,
        "e5caf84805cb24850dd0cf85e18d1dc4a4ee91fcb9e334cfc6f9881952efdb72",
    );
    assert_manifest_entry(
        &manifest,
        "test-vectors/METER-INTEGRATED-SCHEDULE-VERIFICATION.json",
        2_051_223,
        SCHEDULE_VECTORS_SHA256,
    );
    assert_manifest_entry(
        &manifest,
        "test-vectors/METER-STATE-CLOCK-LOCAL-REVIEW-STATUS.json",
        10_646,
        STATE_REVIEW_SHA256,
    );
    assert_eq!(
        vectors["schema"],
        "meter-integrated-and-schedule-verification.v1"
    );
    assert_eq!(vectors["cases"].as_array().unwrap().len(), 10);
    assert_eq!(vectors["continuous_blocks"], 345);
    assert_eq!(vectors["bin_cases"].as_array().unwrap().len(), 7);
    assert_eq!(vectors["schedule_cases"].as_array().unwrap().len(), 5);
    assert_eq!(vectors["cases"][0]["id"], "silence40");
    assert_eq!(vectors["cases"][9]["id"], "reset_repro35");
    assert_eq!(
        review["verdict"],
        "MST_REV_001_CLOSED_LOCAL_SCOPE_NO_NEW_LOCAL_ISSUES"
    );
    assert_eq!(review["recalculation"]["integrated_sequences"], 10);
    assert_eq!(review["recalculation"]["integrated_blocks"], 345);
    assert_eq!(review["recalculation"]["bin_cases"], 7);
    assert_eq!(review["recalculation"]["schedule_cases"], 5);
}

#[test]
fn ten_synthetic_state_sequences_match_full_canonical_state_and_event_traces() {
    let (_, vectors) = read_json(&format!(
        "{FIXTURE_DIRECTORY}/METER-INTEGRATED-SCHEDULE-VERIFICATION.json"
    ));
    let parameters = IntegratedMeterParameters::selected().unwrap();
    assert_eq!(parameters.period(), 5);

    for case in vectors["cases"].as_array().unwrap() {
        assert_eq!(case["input"]["initial_state"], "COLD", "{}", case["id"]);
        assert_eq!(case["input"]["reset_before_blocks"][0], 0, "{}", case["id"]);
        let reset_indices = case["input"]["reset_before_blocks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        let pairs = case["input"]["ordered_energy_pairs"].as_array().unwrap();
        let trace = case["trace"].as_array().unwrap();
        assert_eq!(pairs.len(), trace.len(), "{}", case["id"]);
        let mut state = IntegratedMeterState::cold();

        for (index, (pair, trace)) in pairs.iter().zip(trace).enumerate() {
            if reset_indices.contains(&index) {
                state = IntegratedMeterState::cold();
                if index == 0 {
                    state = patch_state(state, &case["input"]["state_patch"]);
                }
            }
            assert_eq!(
                state.canonical_sha256(),
                trace["before_state_sha256"].as_str().unwrap(),
                "{} block {index} before",
                case["id"]
            );
            let input = IntegratedMeterInput {
                mantissa_word: f32_word(&pair[0]),
                exponent: i32::try_from(pair[1].as_i64().unwrap()).unwrap(),
            };
            let step = process_integrated_meter(&state, input, &parameters)
                .unwrap_or_else(|error| panic!("{} block {index}: {error:?}", case["id"]));
            let next = step.next_state();
            assert_eq!(
                next.canonical_bytes().len(),
                24_512,
                "{} block {index} state size",
                case["id"]
            );
            assert_eq!(
                next.result_word(),
                trace["expected_result_word"].as_u64().unwrap() as u32,
                "{} block {index} result",
                case["id"]
            );
            assert_eq!(
                next.counter(),
                trace["counter"].as_u64().unwrap() as u32,
                "{} block {index} counter",
                case["id"]
            );
            assert_eq!(
                next.absolute_count(),
                trace["absolute_count"].as_u64().unwrap() as u32,
                "{} block {index} absolute count",
                case["id"]
            );
            assert_eq!(
                next.histogram_samples(),
                trace["histogram_samples"].as_u64().unwrap(),
                "{} block {index} histogram count",
                case["id"]
            );
            assert_event(
                step.event(),
                &trace["event"],
                case["id"].as_str().unwrap(),
                index,
            );
            assert_eq!(
                next.canonical_sha256(),
                trace["expected_state_sha256"].as_str().unwrap(),
                "{} block {index} state",
                case["id"]
            );
            state = step.into_next_state();
        }
        assert_eq!(case["terminal_state_bytes"], 24_512);
        assert!(case["pass"].as_bool().unwrap());
    }
}

#[test]
fn seven_f06i_histogram_bin_boundary_cases_match() {
    let (_, vectors) = read_json(&format!(
        "{FIXTURE_DIRECTORY}/METER-INTEGRATED-SCHEDULE-VERIFICATION.json"
    ));
    for (index, case) in vectors["bin_cases"].as_array().unwrap().iter().enumerate() {
        assert_eq!(
            integrated_histogram_bin(f32_word(&case["input_metric"])).unwrap(),
            case["expected_index"].as_u64().unwrap() as u16,
            "bin case {index}"
        );
    }
}

#[test]
fn integrated_meter_rejects_invalid_pair_and_state_values_without_mutation() {
    let parameters = IntegratedMeterParameters::selected().unwrap();
    let cold = IntegratedMeterState::cold();
    let cold_digest = cold.canonical_sha256();
    for input in [
        IntegratedMeterInput {
            mantissa_word: (-1.0_f32).to_bits(),
            exponent: 4,
        },
        IntegratedMeterInput {
            mantissa_word: f32::NAN.to_bits(),
            exponent: 4,
        },
        IntegratedMeterInput {
            mantissa_word: 0x3f00_0000,
            exponent: -1,
        },
        IntegratedMeterInput {
            mantissa_word: 0x3f00_0000,
            exponent: 31,
        },
    ] {
        assert_eq!(
            process_integrated_meter(&cold, input, &parameters),
            Err(IntegratedMeterError::InvalidInput)
        );
        assert_eq!(cold.canonical_sha256(), cold_digest);
    }
    let invalid_counter = cold.clone().with_counter(20);
    assert_eq!(
        process_integrated_meter(
            &invalid_counter,
            IntegratedMeterInput {
                mantissa_word: 0x3f00_0000,
                exponent: 4,
            },
            &parameters,
        ),
        Err(IntegratedMeterError::InvalidState)
    );
    let invalid_period = cold.clone().with_period(6);
    assert_eq!(
        process_integrated_meter(
            &invalid_period,
            IntegratedMeterInput {
                mantissa_word: 0x3f00_0000,
                exponent: 4,
            },
            &parameters,
        ),
        Err(IntegratedMeterError::InvalidState)
    );
    assert_eq!(
        integrated_histogram_bin(f32::NAN.to_bits()),
        Err(IntegratedMeterError::InvalidInput)
    );
}

fn assert_event(
    actual: Option<&jocforge_domain::meter::integrated_meter::IntegratedMeterEvent>,
    expected: &Value,
    case_id: &str,
    block_index: usize,
) {
    if expected.is_null() {
        assert!(
            actual.is_none(),
            "{case_id} block {block_index} unexpected event"
        );
        return;
    }
    let actual = actual.unwrap();
    let context = format!("{case_id} block {block_index}");
    assert_eq!(
        actual.bucket(),
        expected["bucket"].as_u64().unwrap() as u16,
        "{context}"
    );
    assert_eq!(
        actual.counts_halved(),
        expected["counts_halved"].as_bool().unwrap(),
        "{context}"
    );
    assert_eq!(
        actual.absolute_admitted(),
        expected["absolute_admitted"].as_bool().unwrap(),
        "{context}"
    );
    assert_eq!(
        actual.threshold_word(),
        f32_word(&expected["threshold"]),
        "{context}"
    );
    assert_eq!(
        actual.begin(),
        expected["begin"].as_u64().unwrap() as u16,
        "{context}"
    );
    assert_eq!(
        actual.total(),
        expected["total"].as_u64().unwrap(),
        "{context}"
    );
}
