// pattern: Functional Core
use jocforge_domain::meter::ungated_mean::{
    DiOffGetterError, DiOffGetterOutput, UngatedMeanError, UngatedMeanInput, UngatedMeanParameters,
    UngatedMeanState, di_off_getter, process_ungated_mean,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const UNGATED_VECTORS_SHA256: &str =
    "7dc8f1405d6d1cbbcadba19b498e5d0fcb95a11cc221af58a64c81acb2a89d41";
const UNGATED_REVIEW_SHA256: &str =
    "e1ae098cacd11d8c36bb7b1b370371bba87a5ce405c1f95d99b25f8a6ccb5fac";
const GETTER_VECTORS_SHA256: &str =
    "78c9834a428a2f2385e15a22505b26870c603485821bafecada13feeda003dd9";
const FIXTURE_DIRECTORY: &str = "tests/fixtures/ungated_mean";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn manifest() -> Value {
    let bytes =
        std::fs::read(crate_root().join("tests/fixtures/f01e_provider/MANIFEST.json")).unwrap();
    assert_eq!(sha256(&bytes), PACKAGE_MANIFEST_SHA256);
    serde_json::from_slice(&bytes).unwrap()
}

fn fixture(name: &str) -> (Vec<u8>, Value) {
    let bytes = std::fs::read(crate_root().join(FIXTURE_DIRECTORY).join(name)).unwrap();
    let json = serde_json::from_slice(&bytes).unwrap();
    (bytes, json)
}

fn f32_word(value: &Value) -> u32 {
    (value.as_f64().unwrap() as f32).to_bits()
}

fn manifest_entry<'a>(manifest: &'a Value, file: &str) -> &'a Value {
    manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["file"] == file)
        .unwrap()
}

fn assert_manifest_file(manifest: &Value, file: &str, bytes: usize, sha256: &str) {
    let entry = manifest_entry(manifest, file);
    assert_eq!(entry["bytes"].as_u64(), Some(bytes as u64), "{file}");
    assert_eq!(entry["sha256"].as_str(), Some(sha256), "{file}");
}

fn assert_state(actual: &UngatedMeanState, expected: &Value) {
    assert_eq!(actual.count(), expected["count"].as_u64().unwrap() as u32);
    assert_eq!(actual.mean_word(), f32_word(&expected["mean"]));
    assert_eq!(
        actual.exponent(),
        expected["exponent"].as_i64().unwrap() as i32
    );
    assert_eq!(actual.phase(), expected["phase"].as_u64().unwrap() as u32);
    assert_eq!(actual.cursor(), expected["cursor"].as_u64().unwrap() as u32);
    assert_eq!(actual.filled(), expected["filled"].as_u64().unwrap() as u32);
    assert_eq!(
        actual.linear_energy_word(),
        f32_word(&expected["linear_energy"])
    );
    assert_eq!(actual.result_word(), f32_word(&expected["result"]));
}

fn patch_state(mut state: UngatedMeanState, patch: &Value) -> UngatedMeanState {
    if let Some(value) = patch.get("count") {
        state = state.with_count(value.as_u64().unwrap() as u32);
    }
    if let Some(value) = patch.get("mean") {
        state = state.with_mean_word(f32_word(value));
    }
    if let Some(value) = patch.get("exponent") {
        state = state.with_exponent(value.as_i64().unwrap() as i32);
    }
    if let Some(value) = patch.get("filled") {
        state = state.with_filled(value.as_u64().unwrap() as u32);
    }
    state
}

fn input_pair(pair: &Value) -> UngatedMeanInput {
    UngatedMeanInput {
        mantissa_word: f32_word(&pair[0]),
        exponent: pair[1].as_i64().unwrap() as i32,
        ungated_enabled: true,
    }
}

fn update_state_digest(state: &UngatedMeanState, expected: &Value, context: &str) {
    assert_eq!(
        state.canonical_sha256(),
        expected.as_str().unwrap(),
        "{context}"
    );
}

#[test]
fn package_bindings_and_fixture_case_inventory_are_exact() {
    let manifest = manifest();
    let (vectors_bytes, vectors) = fixture("METER-UNGATED-FALLBACK-VERIFICATION.json");
    let (getter_bytes, getter) = fixture("METER-DI-OFF-GETTER-VERIFICATION.json");
    let (review_bytes, review) = fixture("METER-UNGATED-LOCAL-REVIEW-STATUS.json");

    assert_eq!(sha256(&vectors_bytes), UNGATED_VECTORS_SHA256);
    assert_eq!(sha256(&getter_bytes), GETTER_VECTORS_SHA256);
    assert_eq!(sha256(&review_bytes), UNGATED_REVIEW_SHA256);
    assert_manifest_file(
        &manifest,
        "specs/06-ungated-mean-and-fallback.md",
        8_673,
        "07c850be31dbe42a1396634bd96dfd4a54fbd80aa27b3d39a36998545cdc3b9e",
    );
    assert_manifest_file(
        &manifest,
        "test-vectors/METER-UNGATED-FALLBACK-VERIFICATION.json",
        135_066,
        UNGATED_VECTORS_SHA256,
    );
    assert_manifest_file(
        &manifest,
        "test-vectors/METER-UNGATED-LOCAL-REVIEW-STATUS.json",
        8_828,
        UNGATED_REVIEW_SHA256,
    );
    assert_manifest_file(
        &manifest,
        "test-vectors/METER-DI-OFF-GETTER-VERIFICATION.json",
        5_845,
        GETTER_VECTORS_SHA256,
    );
    assert_manifest_file(
        &manifest,
        "specs/00-numeric-semantics.md",
        3_369,
        "53cb67d6c9ddced1fb909dda806bb478a2926a66f2e38cecaada6f6e95d4d135",
    );
    assert_manifest_file(
        &manifest,
        "specs/06-meter-bsi.md",
        6_337,
        "5402250feaee342af79e6ff53b59a33833722e2123c602a21ccd9ec2af0bc752",
    );
    assert_manifest_file(
        &manifest,
        "specs/06-meter-log-and-peak.md",
        7_525,
        "c8a9aec8c931391bfbb4ecaca01a8e5cbec3ba680899f516727c6574a170e38f",
    );
    assert_manifest_file(
        &manifest,
        "parameters/meter-log-mathematical-v1.json",
        780,
        "044d42327f471b183b0a29caaa9db85f694a40c436a5afe3b83734dbf29bc266",
    );
    assert_eq!(vectors["cases"].as_array().unwrap().len(), 10);
    assert_eq!(vectors["continuous_blocks"], 102);
    assert_eq!(vectors["coupled_cases"].as_array().unwrap().len(), 4);
    assert_eq!(vectors["rejection_cases"].as_array().unwrap().len(), 4);
    assert_eq!(getter["cases"].as_array().unwrap().len(), 4);
    assert_eq!(getter["rejection_cases"].as_array().unwrap().len(), 3);
    assert_eq!(review["verdict"], "PASS_FIXED_PROFILE_LOCAL_NO_NEW_ISSUES");
    assert_eq!(review["recalculation"]["sequences"], 10);
    assert_eq!(review["recalculation"]["blocks"], 102);
    assert_eq!(review["recalculation"]["coupled_cases"], 4);
    assert_eq!(review["recalculation"]["entry_rejections"], 4);
}

#[test]
fn single_slot_sequences_match_each_metric_and_complete_state() {
    let (_, vectors) = fixture("METER-UNGATED-FALLBACK-VERIFICATION.json");
    let parameters = UngatedMeanParameters::selected().unwrap();
    let cases = vectors["cases"].as_array().unwrap();

    for case in cases {
        assert_eq!(case["input"]["initial_state"], "COLD", "{}", case["id"]);
        let reset_indices = case["input"]["reset_before_blocks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        let mut state = UngatedMeanState::cold();
        let pairs = case["input"]["ordered_energy_pairs"].as_array().unwrap();
        let trace = case["trace"].as_array().unwrap();
        assert_eq!(pairs.len(), trace.len(), "{}", case["id"]);

        for (index, (pair, trace)) in pairs.iter().zip(trace).enumerate() {
            if reset_indices.contains(&index) {
                state = UngatedMeanState::cold();
                if index == 0 {
                    state = patch_state(state, &case["input"]["state_patch"]);
                }
            }
            update_state_digest(
                &state,
                &trace["before_state_sha256"],
                case["id"].as_str().unwrap(),
            );
            let actual = process_ungated_mean(&state, input_pair(pair), &parameters)
                .unwrap_or_else(|error| panic!("{} block {index}: {error:?}", case["id"]));
            assert_eq!(
                actual.input_metric_word(),
                trace["expected_input_metric_word"].as_u64().unwrap() as u32,
                "{} block {index} metric",
                case["id"]
            );
            assert_eq!(
                actual.writer_called(),
                trace["expected_writer_called"].as_bool().unwrap(),
                "{} block {index} admission",
                case["id"]
            );
            assert_state(actual.next_state(), &trace["expected_state"]);
            update_state_digest(
                actual.next_state(),
                &trace["expected_state_sha256"],
                &format!("{} block {index}", case["id"]),
            );
            state = actual.into_next_state();
        }
    }
}

#[test]
fn coupled_sequences_cover_mean_state_and_unavailable_itu_fallback_only() {
    let (_, vectors) = fixture("METER-UNGATED-FALLBACK-VERIFICATION.json");
    let parameters = UngatedMeanParameters::selected().unwrap();
    for case in vectors["coupled_cases"].as_array().unwrap() {
        let mut state = UngatedMeanState::cold();
        for pair in case["input"]["ordered_energy_pairs"].as_array().unwrap() {
            state = process_ungated_mean(&state, input_pair(pair), &parameters)
                .unwrap_or_else(|error| panic!("{}: {error:?}", case["id"]))
                .into_next_state();
        }
        update_state_digest(
            &state,
            &case["terminal_ungated_state_sha256"],
            &case["id"].to_string(),
        );

        if case["unavailable_itu_fallback_used"].as_bool().unwrap() {
            let selected = di_off_getter(
                None,
                state.result_word(),
                "1770-4",
                false,
                100.0_f32.to_bits(),
            )
            .unwrap();
            assert_getter(&selected, &case["expected"]);
        }
    }
}

#[test]
fn di_off_getter_cases_match_and_reject_unsupported_configuration() {
    let (_, vectors) = fixture("METER-DI-OFF-GETTER-VERIFICATION.json");
    let cold_result = UngatedMeanState::cold().result_word();
    for case in vectors["cases"].as_array().unwrap() {
        let input = &case["input"];
        let actual = di_off_getter(
            Some(f32_word(&input["ITU_host"])),
            cold_result,
            input["metering_mode"].as_str().unwrap(),
            input["dialogue_intelligence"].as_bool().unwrap(),
            f32_word(&input["speech_threshold_percent"]),
        )
        .unwrap_or_else(|error| panic!("{}: {error:?}", case["id"]));
        assert_getter(&actual, &case["expected"]);
    }

    for case in vectors["rejection_cases"].as_array().unwrap() {
        let input = &case["input"];
        assert_eq!(
            di_off_getter(
                Some(f32_word(&input["ITU_host"])),
                cold_result,
                input["metering_mode"].as_str().unwrap(),
                input["dialogue_intelligence"].as_bool().unwrap(),
                f32_word(&input["speech_threshold_percent"]),
            ),
            Err(DiOffGetterError::UnsupportedConfiguration),
            "{}",
            case["id"]
        );
    }
}

#[test]
fn cold_mean_fallback_and_mean_rejection_boundaries_are_explicit() {
    let parameters = UngatedMeanParameters::selected().unwrap();
    let cold = UngatedMeanState::cold();
    let cold_result = cold.result_word();
    assert_eq!(cold.canonical_bytes().len(), 40);
    assert_eq!(
        cold.canonical_sha256(),
        "b88b758653c4d20b30f6cddce816b1260da2ae85d739e60303821eaf4507386e"
    );
    let selected = di_off_getter(
        Some(0xc2f0_0000),
        cold.result_word(),
        "1770-4",
        false,
        100.0_f32.to_bits(),
    )
    .unwrap();
    assert_eq!(selected.selected_internal_word(), cold.result_word());
    assert_eq!(selected.selected_host_word(), 0xc2f0_0000);

    let invalid = [
        UngatedMeanInput {
            mantissa_word: (-1.0_f32).to_bits(),
            exponent: 4,
            ungated_enabled: true,
        },
        UngatedMeanInput {
            mantissa_word: f32::NAN.to_bits(),
            exponent: 4,
            ungated_enabled: true,
        },
        UngatedMeanInput {
            mantissa_word: 0x3f00_0000,
            exponent: 31,
            ungated_enabled: true,
        },
        UngatedMeanInput {
            mantissa_word: 0x3f00_0000,
            exponent: 4,
            ungated_enabled: false,
        },
    ];
    for input in invalid {
        let error = process_ungated_mean(&cold, input, &parameters).unwrap_err();
        assert!(matches!(
            error,
            UngatedMeanError::InvalidInput | UngatedMeanError::UnsupportedConfiguration
        ));
    }

    let (_, vectors) = fixture("METER-UNGATED-FALLBACK-VERIFICATION.json");
    for case in vectors["rejection_cases"].as_array().unwrap() {
        assert!(
            case["expected_rejected"].as_bool().unwrap(),
            "{}",
            case["id"]
        );
        let input = &case["input"];
        let Some(exponent) = input["exponent"].as_i64() else {
            assert_eq!(case["id"], "fractional_e");
            continue;
        };
        let value = UngatedMeanInput {
            mantissa_word: f32_word(&input["mantissa"]),
            exponent: i32::try_from(exponent).unwrap(),
            ungated_enabled: input["ungated_enabled"].as_bool().unwrap(),
        };
        let error = process_ungated_mean(&cold, value, &parameters).unwrap_err();
        if value.ungated_enabled {
            assert_eq!(error, UngatedMeanError::InvalidInput, "{}", case["id"]);
        } else {
            assert_eq!(
                error,
                UngatedMeanError::UnsupportedConfiguration,
                "{}",
                case["id"]
            );
        }
    }

    let (_, getter_vectors) = fixture("METER-DI-OFF-GETTER-VERIFICATION.json");
    let legacy = &getter_vectors["legacy_helper_dependency_observation"];
    assert_eq!(legacy["dependency"], "UNGATED_CONTINUOUS_MEAN_PRODUCER");
    assert_eq!(
        legacy["current_package_dependency_status"],
        "NORMATIVE_MEAN_AND_FALLBACK_PROVIDED_F06U; SEPARATE_LOCAL_REVIEW_PENDING"
    );
    let input = &legacy["input"];
    let cold_fallback = di_off_getter(
        Some(f32_word(&input["ITU_host"])),
        cold_result,
        input["metering_mode"].as_str().unwrap(),
        input["dialogue_intelligence"].as_bool().unwrap(),
        f32_word(&input["speech_threshold_percent"]),
    )
    .unwrap();
    assert_eq!(cold_fallback.selected_internal_word(), cold_result);
    assert_eq!(cold_fallback.selected_host_word(), 0xc2f0_0000);
}

fn assert_getter(actual: &DiOffGetterOutput, expected: &Value) {
    assert_eq!(
        actual.dialogue_word(),
        expected["dialogue_word"].as_u64().unwrap() as u32
    );
    assert_eq!(
        actual.speech_percentage_word(),
        expected["speech_percentage_word"].as_u64().unwrap() as u32
    );
    assert_eq!(
        actual.selected_internal_word(),
        expected
            .get("internal_selected_word")
            .or_else(|| expected.get("selected_internal_word"))
            .unwrap()
            .as_u64()
            .unwrap() as u32
    );
    assert_eq!(
        actual.selected_host_word(),
        expected
            .get("selected_word")
            .or_else(|| expected.get("selected_host_word"))
            .unwrap()
            .as_u64()
            .unwrap() as u32
    );
    assert_eq!(
        actual.internal_gating_kind(),
        expected["internal_gating_kind"].as_u64().unwrap() as u32
    );
    assert_eq!(
        actual.public_gating_kind(),
        expected["public_gating_kind"].as_u64().unwrap() as u32
    );
    if let Some(value) = expected.get("wrapper_config_mode") {
        assert_eq!(actual.wrapper_config_mode(), value.as_u64().unwrap() as u32);
    }
    if let Some(value) = expected.get("normalized_mode") {
        assert_eq!(actual.normalized_mode(), value.as_u64().unwrap() as u32);
    }
    if let Some(value) = expected.get("speech_classifier_executed") {
        assert_eq!(
            actual.speech_classifier_executed(),
            value.as_bool().unwrap()
        );
    }
}
