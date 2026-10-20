// Imperative Shell: validate the exact washed blocker fixture without loading research material.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

const MANIFEST_SHA256: &str = "2ba58412fe9d23bd2c2eb0f262e4b1c05ecda83a7f5980349c6b38d7bcdf8848";

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("spatial_control_cold_serializer_blocker")
}

fn read_json(name: &str) -> Value {
    let bytes = fs::read(fixture_root().join(name)).expect("washed blocker asset exists");
    serde_json::from_slice(&bytes).expect("washed blocker asset is JSON")
}

fn digest_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

#[test]
fn missing_authoritative_serializer_keeps_parent_state_blocked_without_guessing() {
    let root = fixture_root();
    let manifest_bytes =
        fs::read(root.join("SPATIAL-CONTROL-COLD-SERIALIZER-BLOCKER-MANIFEST.json"))
            .expect("washed blocker manifest exists");
    assert_eq!(digest_hex(&manifest_bytes), MANIFEST_SHA256);

    let manifest: Value = serde_json::from_slice(&manifest_bytes).expect("manifest JSON");
    assert_eq!(manifest["schema"], "washed-implementer-blocker-manifest.v1");
    assert_eq!(
        manifest["identity"],
        "spatial-control-cold-serializer.blocked.v2"
    );
    assert_eq!(manifest["status"], "BLOCKED_NO_EXECUTABLE_SERIALIZER");

    let files = manifest["files"].as_array().expect("manifest files array");
    assert_eq!(
        files
            .iter()
            .map(|entry| entry["file"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "COLD-SERIALIZER-EVIDENCE.json",
            "REQUIRED-FRAMING-RULES.json",
            "MINIMUM-EXTERNAL-EVIDENCE.json",
            "INDEPENDENT-CHECK.json"
        ]
    );
    for entry in files {
        let name = entry["file"].as_str().expect("manifest file name");
        let bytes = fs::read(root.join(name)).expect("manifested blocker asset exists");
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

    let source = &manifest["source_evidence"];
    assert_eq!(
        source["research_manifest_sha256"],
        "4aaacedda0677c94b4608dbee0aa15a5a4c89372a82f774e74183905c1ce5837"
    );
    assert_eq!(source["serializer_execution"], "NOT_RUN");
    assert_eq!(source["canonical_preimage"], Value::Null);
    assert_eq!(source["authoritative_length"], Value::Null);
    assert_eq!(source["authoritative_sha256"], Value::Null);
    assert_eq!(source["bytes_guessed"], false);
    assert_eq!(source["AI48ST01_used"], false);
    assert_eq!(source["private_auth"], "excluded");
    assert_eq!(
        manifest["missing_framing"],
        json!([
            "top_level_envelope",
            "F01E_embedding",
            "optional_diffusion_source_cache",
            "current_flags_current_features_persistence",
            "writer_set_framing"
        ])
    );
    assert_eq!(
        manifest["hard_boundaries"]["spatial_render_selector_u04_outer"],
        "NOT_RUN"
    );
    assert_eq!(
        manifest["hard_boundaries"]["production_spatial_control_receipt"],
        "BLOCKED"
    );
    assert_eq!(manifest["hard_boundaries"]["success_vectors"], 0);

    let evidence = read_json("COLD-SERIALIZER-EVIDENCE.json");
    assert_eq!(evidence["entrypoint"], Value::Null);
    assert_eq!(evidence["minimum_run_method"], Value::Null);
    assert_eq!(evidence["serializer_execution"], "NOT_RUN");
    assert_eq!(evidence["canonical_preimage"], Value::Null);
    assert_eq!(evidence["section_offsets"], Value::Null);
    assert_eq!(evidence["authoritative_length"], Value::Null);
    assert_eq!(evidence["authoritative_sha256"], Value::Null);
    assert_eq!(evidence["bytes_guessed"], false);
    assert_eq!(evidence["ai48st01_used"], false);
    assert_eq!(evidence["dll"]["serializer_related_imports"], json!([]));
    assert_eq!(evidence["dll"]["schema_magic_strings_found"], false);
    assert_eq!(evidence["ida"]["serializer_call_observed"], false);
    assert_eq!(evidence["binary_suite_audit"]["pe_files_scanned"], 64);
    assert_eq!(evidence["binary_suite_audit"]["schema_magic_matches"], 0);
    assert_eq!(
        evidence["binary_suite_audit"]["serializer_snapshot_cold_persist_exports"],
        0
    );
    assert_eq!(evidence["binary_suite_audit"]["parse_errors"], 0);
    assert_eq!(
        evidence["binary_suite_audit"]["conclusion"],
        "no executable spatial-control canonical serializer entrypoint found in authorized DEE binary suite"
    );

    let framing = read_json("REQUIRED-FRAMING-RULES.json");
    assert_eq!(framing["status"], "BLOCKED_INCOMPLETE");
    assert_eq!(
        framing["known_field_order"],
        json!([
            "clocks",
            "source_cache",
            "event_control",
            "render_audio",
            "source_flags",
            "feature",
            "selection_outer",
            "matrix_assignment"
        ])
    );
    assert_eq!(framing["unknown_zero_fill_allowed"], false);
    assert_eq!(framing["scratch_allowed"], false);
    let rules = framing["rules"].as_array().expect("framing rules array");
    assert_eq!(rules.len(), 5);
    assert_eq!(
        rules
            .iter()
            .map(|rule| rule["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "top_level_envelope",
            "f01e_embedding",
            "optional_diffusion_source_cache",
            "current_flags_current_features",
            "writer_set"
        ]
    );
    assert_eq!(
        rules
            .iter()
            .map(|rule| rule["status"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "MISSING",
            "MISSING",
            "PARTIALLY_DEFINED_BLOCKED",
            "CONFLICT_BLOCKED",
            "PARTIALLY_DEFINED_BLOCKED"
        ]
    );
    assert!(
        rules
            .iter()
            .all(|rule| rule["required"].as_array().is_some())
    );

    let minimum = read_json("MINIMUM-EXTERNAL-EVIDENCE.json");
    assert_eq!(
        minimum["schema"],
        "spatial-control-cold-minimum-external-evidence.v1"
    );
    let resolution_paths = minimum["accepted_resolution_paths"]
        .as_array()
        .expect("accepted resolution paths");
    assert_eq!(resolution_paths.len(), 2);
    assert_eq!(
        resolution_paths
            .iter()
            .map(|path| path["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["executable_serializer", "normative_framing_spec"]
    );
    assert_eq!(minimum["current_resolution_path_available"], Value::Null);
    assert_eq!(
        minimum["roundtrip_invariants"],
        json!([
            "all declared fields consumed exactly once",
            "no scratch fields",
            "no pointers",
            "no structure padding",
            "no unknown zero-fill",
            "re-serialization byte exact"
        ])
    );

    let independent = read_json("INDEPENDENT-CHECK.json");
    assert_eq!(
        independent["schema"],
        "spatial-control-cold-serializer-blocker-independent-check.v2"
    );
    assert_eq!(independent["all_pass"], true);
    let checks = independent["checks"].as_array().expect("checks array");
    assert_eq!(checks.len(), 11);
    assert!(checks.iter().all(|check| check["pass"] == true));
    assert_eq!(
        checks
            .iter()
            .map(|check| check["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "dll_identity",
            "exports",
            "binary_suite",
            "ida_parent",
            "research_evidence_identity",
            "constructors",
            "no_entrypoint",
            "no_bytes",
            "framing_rules",
            "field_order",
            "minimum_external_evidence"
        ]
    );
    assert_eq!(
        independent["input_files"],
        json!([
            "COLD-SERIALIZER-EVIDENCE.json",
            "REQUIRED-FRAMING-RULES.json",
            "MINIMUM-EXTERNAL-EVIDENCE.json"
        ])
    );
    assert_eq!(independent["research_files_read"], false);
    assert_eq!(independent["serializer_execution"], false);
    assert_eq!(independent["bytes_guessed"], false);
    assert_eq!(independent["expected_used_as_seed"], false);
}
