// Functional Core: validate the sanitized admission package, not a callback execution.
// Imperative Shell: read the exact bundled package assets; no ADM or PCM is loaded.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

const MANIFEST_SHA256: &str = "593648f7d63ce3e595510e16a067cf19e2d5c2a56a53a68c5bea6eb014c96d61";
const ADMISSION_FAILURES: &[(&str, &str)] = &[
    ("missing_control38", "SPATIAL_CONTROL_INPUT_MATERIAL"),
    (
        "missing_parent_before_state",
        "SPATIAL_CONTROL_INPUT_MATERIAL",
    ),
    ("generation_mismatch", "SPATIAL_CONTROL_CLOCK"),
    ("provider_mismatch", "SPATIAL_CONTROL_PROVIDER"),
    ("installed48_shape", "SPATIAL_CONTROL_SHAPE"),
    ("missing_u04_material", "SPATIAL_CONTROL_STAGE_MATERIAL"),
    ("missing_outer_material", "SPATIAL_CONTROL_STAGE_MATERIAL"),
    ("destination_count", "SPATIAL_CONTROL_SHAPE"),
    (
        "missing_target_transition",
        "SPATIAL_CONTROL_STAGE_MATERIAL",
    ),
    ("receiver_owner_violation", "SPATIAL_AUDIO_OWNER_VIOLATION"),
];
const EXECUTION_FAULTS: &[&str] = &["after_selector", "after_u04", "after_outer", "after_target"];
const BLOCKED_MATERIAL: &[&str] = &[
    "parsed callback0 control38",
    "authoritative parent before-state bytes",
    "installed48 full bytes",
    "selector/U04/outer complete materials",
    "matrix/destinations/targets complete materials",
    "authoritative parent after-state bytes",
];

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/spatial_control_production_admission")
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

fn read_fixture(name: &str) -> Value {
    let bytes = fs::read(fixture_root().join(name)).expect("fixture file exists");
    serde_json::from_slice(&bytes).expect("fixture JSON parses")
}

fn is_sha256(value: &Value) -> bool {
    value.as_str().is_some_and(|text| {
        text.len() == 64
            && text
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn assert_package_manifest() {
    let root = fixture_root();
    let manifest_bytes = fs::read(root.join("SPATIAL-CONTROL-PRODUCTION-ADMISSION-MANIFEST.json"))
        .expect("admission manifest exists");
    assert_eq!(digest_hex(&manifest_bytes), MANIFEST_SHA256);
    let manifest: Value = serde_json::from_slice(&manifest_bytes).expect("manifest JSON");
    assert_eq!(
        manifest["schema"],
        "washed-implementer-supplement-manifest.v1"
    );
    assert_eq!(
        manifest["identity"],
        "spatial-control.production-admission.callback0.v1"
    );
    assert_eq!(
        manifest["status"],
        "PRODUCTION_ADMISSION_IDENTITY_READY_BUT_EXECUTION_BLOCKED"
    );
    assert_eq!(
        manifest["delivery_scope"],
        "internal admission contract only"
    );

    let files = manifest["files"].as_array().expect("manifest files array");
    assert_eq!(files.len(), 6);
    for entry in files {
        let name = entry["file"].as_str().expect("manifest file name");
        let bytes = fs::read(root.join(name)).expect("manifested admission file exists");
        assert_eq!(
            bytes.len() as u64,
            entry["bytes"].as_u64().unwrap(),
            "{name}"
        );
        assert_eq!(
            digest_hex(&bytes),
            entry["sha256"].as_str().unwrap(),
            "{name}"
        );
    }

    assert_eq!(manifest["source_evidence"]["raw_adm_bytes"], 743_696_526);
    assert_eq!(
        manifest["source_evidence"]["raw_adm_sha256"],
        "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1"
    );
    assert_eq!(
        manifest["source_evidence"]["raw_pcm_material"],
        "not_delivered"
    );
    assert_eq!(manifest["source_evidence"]["private_key_auth"], "excluded");

    let boundaries = &manifest["hard_boundaries"];
    assert_eq!(boundaries["raw_adm_input_identity"], "PINNED");
    assert_eq!(boundaries["parsed_callback0_control38"], "NOT_RUN");
    assert_eq!(boundaries["authoritative_parent_before_state"], "BLOCKED");
    assert_eq!(boundaries["callback0_numerical_execution"], "NOT_RUN");
    assert_eq!(boundaries["authoritative_parent_after_state"], "BLOCKED");
    assert_eq!(boundaries["spatial_control_receipt"], "BLOCKED");
    assert_eq!(boundaries["spatial_audio_proposal"], "BLOCKED");
    assert_eq!(boundaries["production_allowed"], false);
    assert_eq!(boundaries["success_vectors"], 0);
    assert_eq!(boundaries["global_callback_transaction"], "BLOCKED");
}

#[test]
fn real_adm_identity_is_pinned_but_parent_execution_remains_blocked() {
    assert_package_manifest();
    let input = read_fixture("REAL-ADM-INPUT-IDENTITY.json");
    let providers = read_fixture("PROVIDER-IDENTITIES.json");
    let schema = read_fixture("SPATIAL-CONTROL-PRODUCTION.schema.json");
    let vectors = read_fixture("PRODUCTION-ADMISSION-VECTORS.json");
    let independent = read_fixture("INDEPENDENT-CHECK.json");

    assert_eq!(input["schema"], "sanitized-real-adm-input-identity.v1");
    assert_eq!(input["input_id"], "adm-sha256-7826dcbef37ef696");
    assert_eq!(input["file_bytes"], 743_696_526);
    assert_eq!(
        input["file_sha256"],
        "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1"
    );
    assert_eq!(input["wave"]["format"], "PCM");
    assert_eq!(input["wave"]["channels"], 38);
    assert_eq!(input["wave"]["sample_rate"], 48_000);
    assert_eq!(input["wave"]["bits_per_sample"], 24);
    assert_eq!(input["wave"]["byte_rate"], 5_472_000);
    assert_eq!(input["wave"]["block_align"], 114);
    assert_eq!(input["wave"]["sample_frames"], 6_522_906);
    assert_eq!(
        input["chunks"]["data"]["bytes"],
        input["wave"]["sample_frames"].as_u64().unwrap()
            * input["wave"]["block_align"].as_u64().unwrap()
    );
    assert_eq!(input["control_identity"]["owner"], "frontend_control");
    assert_eq!(input["control_identity"]["status"], "NOT_RUN");
    assert_eq!(
        input["control_identity"]["parsed_callback0_control38_sha256"],
        Value::Null
    );
    assert_eq!(input["pcm_material_included"], false);
    assert_eq!(input["metadata_material_included"], false);
    for chunk in ["data", "axml", "chna", "dbmd"] {
        assert!(input["chunks"][chunk]["bytes"].as_u64().is_some());
        assert!(is_sha256(&input["chunks"][chunk]["sha256"]));
    }

    assert_eq!(
        providers["schema"],
        "spatial-control-production-provider-identities.v1"
    );
    for name in [
        "frontend_event_control_constructor",
        "selected_target_cold_constructor",
        "provider_manifest",
        "parent_state_schema",
    ] {
        assert!(providers[name]["bytes"].as_u64().is_some());
        assert!(is_sha256(&providers[name]["sha256"]));
    }
    assert_eq!(
        providers["authoritative_parent_cold_bytes"],
        "NOT_DELIVERED"
    );
    assert_eq!(
        providers["authoritative_parent_callback0_after_bytes"],
        "NOT_DELIVERED"
    );

    assert_eq!(schema["$id"], "spatial-control-production-admission.v1");
    assert_eq!(
        schema["$defs"]["callback0_input"]["properties"]["callback"]["const"],
        0
    );
    assert_eq!(
        schema["$defs"]["callback0_input"]["properties"]["generation"]["const"],
        1
    );
    assert_eq!(
        schema["$defs"]["callback0_input"]["properties"]["phase"]["const"],
        "NORMAL"
    );
    for name in [
        "host38",
        "control38",
        "parent_before_state",
        "provider_sha256",
    ] {
        assert!(
            schema["$defs"]["callback0_input"]["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|field| field == name)
        );
    }
    assert_eq!(
        schema["$defs"]["spatial_control_receipt"]["properties"]["commit_authority"]["const"],
        "ENCLOSING_CALLBACK_TRANSACTION_ONCE"
    );
    assert_eq!(
        schema["$defs"]["spatial_audio_proposal"]["properties"]["receiver_qmf_mutated"]["const"],
        false
    );

    assert_eq!(
        vectors["schema"],
        "spatial-control-production-admission-vectors.v1"
    );
    assert_eq!(vectors["success_recipe"]["status"], "NOT_RUN");
    assert_eq!(vectors["success_recipe"]["callback"], 0);
    assert_eq!(vectors["success_recipe"]["generation"], 1);
    assert_eq!(
        vectors["success_recipe"]["expected_output_oracle_included"],
        false
    );
    let failures = vectors["admission_failures"].as_array().unwrap();
    assert_eq!(failures.len(), ADMISSION_FAILURES.len());
    for ((expected_id, expected_error), failure) in ADMISSION_FAILURES.iter().zip(failures) {
        assert_eq!(failure["id"], *expected_id);
        assert_eq!(failure["error"], *expected_error);
        assert_eq!(failure["commit_count"], 0);
    }
    let fault_plans = vectors["execution_fault_plans"].as_array().unwrap();
    assert_eq!(fault_plans.len(), EXECUTION_FAULTS.len());
    for (expected_id, fault) in EXECUTION_FAULTS.iter().zip(fault_plans) {
        assert_eq!(fault["id"], *expected_id);
        assert!(
            fault["trigger"]
                .as_str()
                .is_some_and(|text| !text.is_empty())
        );
    }
    assert_eq!(vectors["fault_expected"]["test_harness_only"], true);
    assert_eq!(vectors["fault_expected"]["production_parameter"], false);
    assert_eq!(vectors["fault_expected"]["parent_state_unchanged"], true);
    assert_eq!(vectors["fault_expected"]["f04_before_unchanged"], true);
    assert_eq!(
        vectors["fault_expected"]["spatial_control_receipt"],
        Value::Null
    );
    assert_eq!(
        vectors["fault_expected"]["spatial_audio_proposal"],
        Value::Null
    );
    assert_eq!(vectors["fault_expected"]["commit_count"], 0);
    assert_eq!(vectors["fault_expected"]["programme_result"], "RETIRED");
    assert_eq!(vectors["blocked_material"], json!(BLOCKED_MATERIAL));
    assert_eq!(vectors["expected_used_as_seed"], false);

    let checks = independent["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 10);
    assert!(checks.iter().all(|check| check["pass"] == true));
    assert_eq!(independent["all_pass"], true);
    assert_eq!(independent["raw_adm_read"], false);
    assert_eq!(independent["research_material_accessed"], false);
    assert_eq!(independent["production_execution"], false);
    assert_eq!(independent["expected_used_as_seed"], false);
}
