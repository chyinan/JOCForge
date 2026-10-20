// pattern: Functional Core; test-only supplied-receipt lifecycle validator
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

const FIXTURE_MANIFEST_SHA256: &str =
    "87b538aa1ca36a55693115f29bfdfd86a0682086578862dcc320545c523a2686";
const PROVIDER_FILE: &str = "none-drc-provider.json";
const VECTORS_FILE: &str = "NONE-DRC-OWNER-VECTORS.json";

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/none_drc_lifecycle")
}

fn digest_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut text = String::with_capacity(digest.len() * 2);
    for byte in digest {
        text.push(char::from(DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    text
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push(char::from(DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    text
}

fn authorized13_words_sha256(provider: &Value) -> String {
    let words = provider["authorized13_u32"]
        .as_array()
        .expect("authorized thirteen-word array");
    let mut bytes = Vec::with_capacity(words.len() * 4);
    for word in words {
        bytes.extend_from_slice(
            &u32::try_from(word.as_u64().expect("authorized word is unsigned"))
                .expect("authorized word fits U32")
                .to_le_bytes(),
        );
    }
    digest_hex(&bytes)
}

fn canonical_json_sha256(value: &Value) -> String {
    digest_hex(&serde_json::to_vec(value).expect("canonical fixture JSON"))
}

fn read_fixture(name: &str) -> Value {
    let bytes = fs::read(fixture_root().join(name)).expect("fixture file exists");
    serde_json::from_slice(&bytes).expect("fixture JSON parses")
}

fn has_exact_keys(value: &Value, keys: &[&str]) -> bool {
    value.as_object().is_some_and(|object| {
        object.len() == keys.len() && keys.iter().all(|key| object.contains_key(*key))
    })
}

fn is_u64(value: &Value) -> bool {
    value.as_u64().is_some()
}

fn is_u32(value: &Value) -> bool {
    value
        .as_u64()
        .is_some_and(|word| u32::try_from(word).is_ok())
}

fn is_u16(value: &Value) -> bool {
    value
        .as_u64()
        .is_some_and(|word| u16::try_from(word).is_ok())
}

fn has_u32_words(value: &Value, length: usize) -> bool {
    value
        .as_array()
        .is_some_and(|words| words.len() == length && words.iter().all(is_u32))
}

fn state_capacity_valid(state: &Value) -> bool {
    const STATE_KEYS: [&str; 13] = [
        "schema",
        "profile_id",
        "provider_sha256",
        "source_index",
        "generation",
        "measurement_sha256",
        "programme_dialnorm",
        "ordinary_history_u32_words",
        "secondary",
        "statistics",
        "scheduler",
        "override_fifo",
        "finalized",
    ];
    const SECONDARY_KEYS: [&str; 3] = [
        "forward_history_u32_words",
        "inverse_carry_u32_words",
        "frame_index",
    ];
    const STATISTICS_KEYS: [&str; 4] = [
        "filter_pairs_u32_words",
        "primary_peak_u32_words",
        "secondary_peak_u32_words",
        "downmix_peak_u32",
    ];
    const SCHEDULER_KEYS: [&str; 6] = [
        "line_dynamics",
        "rf_dynamics",
        "line_limiter",
        "rf_limiter",
        "peak_logs_u32_words",
        "generation",
    ];
    const FIFO_KEYS: [&str; 4] = [
        "records_u32",
        "metadata_delay",
        "append_slot",
        "frames_processed",
    ];
    const DYNAMICS_KEYS: [&str; 3] = ["level_u32", "hold_u16", "log_gain_u32"];
    const LIMITER_KEYS: [&str; 2] = ["gain_u32", "hold_u16"];

    if !has_exact_keys(state, &STATE_KEYS)
        || state["schema"] != "none-drc-owner-state.v1"
        || !is_u64(&state["source_index"])
        || !is_u64(&state["generation"])
        || !is_u16(&state["programme_dialnorm"])
        || !state["finalized"].is_boolean()
        || !has_u32_words(&state["ordinary_history_u32_words"], 7680)
    {
        return false;
    }

    let secondary = &state["secondary"];
    if !has_exact_keys(secondary, &SECONDARY_KEYS)
        || !has_u32_words(&secondary["forward_history_u32_words"], 640)
        || !has_u32_words(&secondary["inverse_carry_u32_words"], 640)
        || !is_u64(&secondary["frame_index"])
    {
        return false;
    }

    let statistics = &state["statistics"];
    if !has_exact_keys(statistics, &STATISTICS_KEYS)
        || !has_u32_words(&statistics["filter_pairs_u32_words"], 10)
        || !has_u32_words(&statistics["primary_peak_u32_words"], 6)
        || !has_u32_words(&statistics["secondary_peak_u32_words"], 6)
        || !is_u32(&statistics["downmix_peak_u32"])
    {
        return false;
    }

    let scheduler = &state["scheduler"];
    if !has_exact_keys(scheduler, &SCHEDULER_KEYS)
        || !has_exact_keys(&scheduler["line_dynamics"], &DYNAMICS_KEYS)
        || !has_exact_keys(&scheduler["rf_dynamics"], &DYNAMICS_KEYS)
        || !has_exact_keys(&scheduler["line_limiter"], &LIMITER_KEYS)
        || !has_exact_keys(&scheduler["rf_limiter"], &LIMITER_KEYS)
        || !has_u32_words(&scheduler["peak_logs_u32_words"], 6)
        || !is_u64(&scheduler["generation"])
    {
        return false;
    }
    for dynamics in [&scheduler["line_dynamics"], &scheduler["rf_dynamics"]] {
        if !is_u32(&dynamics["level_u32"])
            || !is_u16(&dynamics["hold_u16"])
            || !is_u32(&dynamics["log_gain_u32"])
        {
            return false;
        }
    }
    for limiter in [&scheduler["line_limiter"], &scheduler["rf_limiter"]] {
        if !is_u32(&limiter["gain_u32"]) || !is_u16(&limiter["hold_u16"]) {
            return false;
        }
    }

    let fifo = &state["override_fifo"];
    has_exact_keys(fifo, &FIFO_KEYS)
        && has_u32_words(&fifo["records_u32"], 4)
        && fifo["metadata_delay"].as_u64() == Some(3072)
        && fifo["append_slot"].as_u64() == Some(3)
        && is_u64(&fifo["frames_processed"])
}

fn error_result(code: &str) -> Value {
    json!({
        "status": "ERROR",
        "error": code,
        "receipt": null,
        "state_delta": null,
        "commit_count": 0,
    })
}

fn assert_fixture_manifest() {
    let root = fixture_root();
    let bytes = fs::read(root.join("NONE-DRC-FIXTURE-MANIFEST.json"))
        .expect("none-DRC fixture manifest exists");
    assert_eq!(digest_hex(&bytes), FIXTURE_MANIFEST_SHA256);
    let manifest: Value = serde_json::from_slice(&bytes).expect("manifest JSON");
    let files = manifest["files"].as_array().expect("manifest files array");
    assert_eq!(files.len(), 7);
    for entry in files {
        let name = entry["file"].as_str().expect("manifest file name");
        let bytes = fs::read(root.join(name)).expect("manifested file exists");
        assert_eq!(bytes.len() as u64, entry["bytes"].as_u64().unwrap());
        assert_eq!(digest_hex(&bytes), entry["sha256"].as_str().unwrap());
    }
}

fn validate_synthetic_stage(
    before_state: &Value,
    stage_receipt: &Value,
    provider_sha256: &str,
    measurement_sha256: &str,
) -> Value {
    const STAGE_KEYS: [&str; 15] = [
        "schema",
        "source_index",
        "generation",
        "provider_sha256",
        "measurement_sha256",
        "primary_shape",
        "secondary_shape",
        "short_shape",
        "finite",
        "input_recipe",
        "rf_u16",
        "line_u16",
        "before_sha256",
        "after_sha256",
        "after_state",
    ];
    const SELECTED_PROFILE: &str = "selected-none-drc-5.1-48k.v1";

    if !state_capacity_valid(before_state) {
        return error_result("DRC_STATE_CAPACITY");
    }
    if !has_exact_keys(stage_receipt, &STAGE_KEYS)
        || stage_receipt["schema"] != "synthetic-none-drc-stage-receipt.v1"
    {
        return error_result("DRC_IDENTITY");
    }

    let Some(source_index) = stage_receipt["source_index"].as_u64() else {
        return error_result("DRC_CLOCK");
    };
    let Some(generation) = stage_receipt["generation"].as_u64() else {
        return error_result("DRC_CLOCK");
    };
    let Some(next_generation) = source_index.checked_add(1) else {
        return error_result("DRC_CLOCK");
    };
    if before_state["source_index"].as_u64() != Some(source_index)
        || before_state["generation"].as_u64() != Some(source_index)
        || generation != next_generation
    {
        return error_result("DRC_CLOCK");
    }

    if before_state["profile_id"] != SELECTED_PROFILE
        || before_state["provider_sha256"] != provider_sha256
        || before_state["measurement_sha256"] != measurement_sha256
        || stage_receipt["provider_sha256"] != provider_sha256
        || stage_receipt["measurement_sha256"] != measurement_sha256
    {
        return error_result("DRC_IDENTITY");
    }

    if stage_receipt["primary_shape"] != json!([6, 1536])
        || stage_receipt["secondary_shape"] != json!([6, 1536])
        || stage_receipt["short_shape"] != json!([6, 5])
    {
        return error_result("DRC_INPUT_SHAPE");
    }
    if stage_receipt["finite"] != true {
        return error_result("DRC_NUMERIC");
    }
    if !stage_receipt["input_recipe"].is_string() {
        return error_result("DRC_INPUT_SHAPE");
    }

    let Some(rf_u16) = stage_receipt["rf_u16"].as_u64() else {
        return error_result("DRC_CONTROL_SHAPE");
    };
    let line_u16 = &stage_receipt["line_u16"];
    if rf_u16 > u64::from(u16::MAX)
        || !line_u16.as_array().is_some_and(|values| {
            values.len() == 6
                && values.iter().all(|value| {
                    value
                        .as_u64()
                        .is_some_and(|word| u16::try_from(word).is_ok())
                })
        })
    {
        return error_result("DRC_CONTROL_SHAPE");
    }

    let before_sha256 = canonical_json_sha256(before_state);
    if stage_receipt["before_sha256"] != before_sha256 {
        return error_result("DRC_IDENTITY");
    }
    let after_state = &stage_receipt["after_state"];
    if !state_capacity_valid(after_state)
        || after_state["profile_id"] != SELECTED_PROFILE
        || after_state["provider_sha256"] != provider_sha256
        || after_state["measurement_sha256"] != measurement_sha256
        || after_state["source_index"].as_u64() != source_index.checked_add(1)
        || after_state["generation"].as_u64() != Some(generation)
        || after_state["scheduler"]["generation"].as_u64() != Some(generation)
        || after_state["finalized"] != false
    {
        return error_result("DRC_AFTER_STATE");
    }
    let after_sha256 = canonical_json_sha256(after_state);
    if stage_receipt["after_sha256"] != after_sha256 {
        return error_result("DRC_AFTER_STATE");
    }

    let receipt = json!({
        "schema": "none-drc-lifecycle-validation-receipt.v1",
        "mode": "SYNTHETIC_LIFECYCLE_ONLY",
        "production_allowed": false,
        "source_index": source_index,
        "provider_sha256": provider_sha256,
        "measurement_sha256": measurement_sha256,
        "before_sha256": before_sha256,
        "recorded_after_sha256": after_sha256,
        "rf_u16": rf_u16,
        "line_u16": line_u16,
        "state_delta_committed": false,
        "commit_count": 0,
    });
    json!({
        "status": "PASS",
        "receipt": receipt,
        "state_delta": null,
        "commit_count": 0,
    })
}

fn validate_eof_projection(
    before: &Value,
    after: &Value,
    expected_sources: u64,
) -> Result<(), &'static str> {
    if before["source_index"].as_u64() != Some(expected_sources)
        || before["generation"].as_u64() != Some(expected_sources)
        || after["source_index"].as_u64() != Some(expected_sources)
        || after["generation"].as_u64() != Some(expected_sources)
    {
        return Err("DRC_CLOCK");
    }
    if after["finalized"] != true {
        return Err("DRC_AFTER_STATE");
    }
    Ok(())
}

#[test]
fn none_drc_lifecycle_vectors_validate_without_execution_or_commit() {
    assert_fixture_manifest();
    let provider_file = read_fixture(PROVIDER_FILE);
    let machine_map = read_fixture("NONE-DRC-MACHINE-MAP.json");
    let vectors = read_fixture(VECTORS_FILE);
    let independent = read_fixture("INDEPENDENT-CHECK.json");

    let provider_sha256 = vectors["provider"]["canonical_sha256"]
        .as_str()
        .expect("canonical provider SHA-256");
    let measurement_sha256 = vectors["measurement"]["canonical_sha256"]
        .as_str()
        .expect("canonical measurement SHA-256");
    assert_eq!(provider_file, vectors["provider"]["value"]);
    assert_eq!(canonical_json_sha256(&provider_file), provider_sha256);
    assert_eq!(provider_file["production_allowed"], false);
    assert_eq!(
        authorized13_words_sha256(&provider_file),
        provider_file["authorized13_words_sha256"]
    );
    assert_eq!(
        provider_file["authorized13_provider_sha256"],
        vectors["provider"]["value"]["authorized13_provider_sha256"]
    );
    assert_eq!(
        provider_file["mathematical34_provider_sha256"],
        vectors["provider"]["value"]["mathematical34_provider_sha256"]
    );
    assert_eq!(
        canonical_json_sha256(&vectors["measurement"]["value"]),
        measurement_sha256
    );

    let cold = &vectors["cold"];
    let cold_state = &cold["state"];
    assert!(state_capacity_valid(cold_state));
    let cold_bytes = serde_json::to_vec(cold_state).expect("cold state canonical JSON");
    assert_eq!(
        hex(&cold_bytes),
        cold["canonical_utf8_hex"]
            .as_str()
            .expect("cold canonical bytes")
    );
    assert_eq!(digest_hex(&cold_bytes), cold["sha256"]);
    assert_eq!(cold_state["provider_sha256"], provider_sha256);
    assert_eq!(cold_state["measurement_sha256"], measurement_sha256);
    assert_eq!(
        cold_state["programme_dialnorm"],
        vectors["measurement"]["value"]["dialnorm"]
    );

    let success_calls = vectors["success_calls"].as_array().expect("success calls");
    assert_eq!(success_calls.len(), 3);
    let mut previous_after_state = cold_state.clone();
    for (expected_source, call) in success_calls.iter().enumerate() {
        assert_eq!(call["before_state"], previous_after_state);
        assert_eq!(
            call["stage_receipt"]["source_index"].as_u64(),
            Some(u64::try_from(expected_source).unwrap())
        );
        let result = validate_synthetic_stage(
            &call["before_state"],
            &call["stage_receipt"],
            provider_sha256,
            measurement_sha256,
        );
        assert_eq!(result, call["expected"]);
        assert_eq!(result["status"], "PASS");
        assert_eq!(result["state_delta"], Value::Null);
        assert_eq!(result["commit_count"], 0);
        assert_eq!(result["receipt"]["production_allowed"], false);
        assert_eq!(result["receipt"]["state_delta_committed"], false);
        previous_after_state = call["stage_receipt"]["after_state"].clone();
    }
    assert_eq!(provider_file["suppressed_sources"], 2);

    let failures = vectors["failure_cases"].as_array().expect("failure cases");
    assert_eq!(failures.len(), 11);
    for failure in failures {
        let result = validate_synthetic_stage(
            &failure["before_state"],
            &failure["stage_receipt"],
            provider_sha256,
            measurement_sha256,
        );
        assert_eq!(
            result, failure["expected"],
            "failure vector {}",
            failure["id"]
        );
        assert_eq!(result["receipt"], Value::Null);
        assert_eq!(result["state_delta"], Value::Null);
        assert_eq!(result["commit_count"], 0);
    }

    let reset = &vectors["reset"];
    assert_eq!(reset["equals_cold"], true);
    assert_eq!(reset["state"], *cold_state);
    assert!(state_capacity_valid(&reset["state"]));
    assert_eq!(canonical_json_sha256(&reset["state"]), cold["sha256"]);

    let eof = &vectors["eof"];
    validate_eof_projection(
        &eof["before"],
        &eof["after"],
        provider_file["expected_sources"].as_u64().unwrap(),
    )
    .expect("EOF adds no source or generation");
    assert_eq!(eof["extra_outputs"], 0);
    assert_eq!(eof["extra_flushes"], 0);

    assert_eq!(vectors["expected_used_as_seed"], false);
    assert_eq!(
        vectors["blocked"],
        json!([
            "whole meter/frontend/audio path",
            "actual secondary production API",
            "other DRC profiles",
            "real 4249 replay"
        ])
    );
    assert_eq!(
        machine_map["blocked"],
        json!([
            "whole meter root",
            "secondary production API",
            "other DRC profiles",
            "global CallbackTransaction"
        ])
    );
    assert_eq!(machine_map["fixture_seam"]["production_allowed"], false);
    assert_eq!(machine_map["fixture_seam"]["commit_count"], 0);
    assert_eq!(machine_map["fixture_seam"]["state_delta"], Value::Null);
    assert_eq!(machine_map["owner_id"], "drc");
    assert_eq!(
        machine_map["production_inputs"].as_array().unwrap().len(),
        8
    );
    assert_eq!(independent["checks"].as_array().unwrap().len(), 21);
    assert!(
        independent["checks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|check| check["pass"] == true)
    );
    assert_eq!(independent["all_pass"], true);
    assert_eq!(independent["imports_lifecycle_model"], false);
    assert_eq!(independent["independent_numeric_algorithm_replay"], false);
    assert_eq!(independent["expected_used_as_seed"], false);
}
