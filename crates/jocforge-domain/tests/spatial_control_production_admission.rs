// Functional Core: validate the sanitized admission package, not a callback execution.
// Imperative Shell: read the exact bundled package assets; no ADM or PCM is loaded.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

const MANIFEST_SHA256: &str = "6b06dc200da8896987b250cefb5fe2eb225d0e4ee400a4d5912f0a1cd8b391a0";
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
    "authoritative cold parent serializer and before-state preimage",
    "installed48 full bytes",
    "selector/U04/outer complete materials",
    "matrix/destinations/markers/writer-set complete materials",
    "target previous/current/active and transition complete materials",
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

fn u32le_rows(rows: &Value) -> Vec<u8> {
    let rows = rows.as_array().expect("control38 rows array");
    assert_eq!(rows.len(), 38);
    let mut bytes = Vec::with_capacity(38 * 13 * 4);
    for row in rows {
        let words = row.as_array().expect("control38 row array");
        assert_eq!(words.len(), 13);
        for word in words {
            let value =
                u32::try_from(word.as_u64().expect("U32 word")).expect("control38 word fits U32");
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    bytes
}

fn u16le_values(values: &Value) -> Vec<u8> {
    let values = values.as_array().expect("physical mapping values");
    assert_eq!(values.len(), 38);
    let mut bytes = Vec::with_capacity(38 * 2);
    for value in values {
        let value = u16::try_from(value.as_u64().expect("physical mapping U16"))
            .expect("physical mapping value fits U16");
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn assert_package_manifest() {
    let root = fixture_root();
    let manifest_bytes = fs::read(root.join("SPATIAL-CONTROL-PRODUCTION-ADMISSION-MANIFEST.json"))
        .expect("admission manifest exists");
    assert_eq!(digest_hex(&manifest_bytes), MANIFEST_SHA256);
    let manifest: Value = serde_json::from_slice(&manifest_bytes).expect("manifest JSON");
    assert_eq!(
        manifest["schema"],
        "sanitized-spatial-control-production-admission-manifest.v10"
    );
    assert_eq!(
        manifest["scope"],
        "real cold callback0 direct preconditions"
    );
    assert_eq!(manifest["raw_adm_identity"], "PINNED");
    assert_eq!(
        manifest["callback0_input_lineage"],
        "BOUND_TO_PINNED_REAL_ADM_BY_BYTES_AND_SHA256"
    );
    assert_eq!(manifest["admission_failure_vectors"], 10);
    assert_eq!(manifest["execution_fault_plans"], 4);
    assert_eq!(manifest["native_receipt_schema"], "NOT_OBSERVED");
    assert_eq!(manifest["private_auth"], "EXCLUDED");
    assert_eq!(manifest["remaining_blocked_material_count"], 6);

    let files = manifest["files"].as_array().expect("manifest files array");
    assert_eq!(files.len(), 12);
    for entry in files {
        let name = entry["path"].as_str().expect("manifest file path");
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

    assert_eq!(manifest["raw_adm_identity"], "PINNED");
    assert_eq!(
        read_fixture("REAL-ADM-INPUT-IDENTITY.json")["file_sha256"],
        "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1"
    );

    let boundaries = &manifest["hard_boundaries"];
    assert_eq!(boundaries["raw_adm_input_identity"], "PINNED");
    assert_eq!(
        boundaries["callback0_input_lineage"],
        "BOUND_TO_PINNED_REAL_ADM_BY_BYTES_AND_SHA256"
    );
    assert_eq!(
        boundaries["physical_control38_material"],
        "CAPTURED_NATIVE_CALLBACK0_INPUT_LINEAGE_BOUND"
    );
    assert_eq!(
        boundaries["dbmd_track_to_bed_object_semantic_binding_map"],
        "CAPTURED_NATIVE_PLUS_ADM_DBMD_JOIN"
    );
    assert_eq!(boundaries["callback0_numerical_execution"], "NOT_RUN");
    assert_eq!(boundaries["callback_semantic_admission"], "NOT_RUN");
    assert_eq!(
        boundaries["control_mapping_admission_boundary"],
        "MAPPING_CAPTURED_ADMISSION_NOT_RUN"
    );
    assert_eq!(
        boundaries["parent_cold_serializer"],
        "BLOCKED_NO_EXECUTABLE_SERIALIZER"
    );
    assert_eq!(boundaries["authoritative_parent_before_state"], "BLOCKED");
    assert_eq!(boundaries["authoritative_parent_after_state"], "BLOCKED");
    assert_eq!(boundaries["production_spatial_control_receipt"], "BLOCKED");
    assert_eq!(boundaries["spatial_audio_proposal"], "BLOCKED");
    assert_eq!(
        boundaries["spatial_render_selector_u04_outer"],
        "BLOCKED_NOT_RUN"
    );
    assert_eq!(boundaries["production_allowed"], false);
    assert_eq!(boundaries["success_vectors"], 0);
    assert_eq!(boundaries["global_transaction"], "BLOCKED");
    assert_eq!(boundaries["native_receipt_schema"], "NOT_OBSERVED");
    assert_eq!(boundaries["private_auth"], "EXCLUDED");
    let canonical_boundaries = serde_json::to_vec(boundaries).expect("canonical JSON bytes");
    assert_eq!(
        digest_hex(&canonical_boundaries),
        manifest["hard_boundaries_sha256"]
    );
    assert_eq!(
        manifest["hard_boundaries_sha256"],
        "8301ff69fd12fb3d4c8ed97f4e79b83bc9885283c4aec1d4914246e54b47c374"
    );
    assert_eq!(
        manifest["detached_independent_check"]["included_in_manifest_files"],
        false
    );
    assert_eq!(
        manifest["detached_independent_check"]["path"],
        "INDEPENDENT-CHECK.json"
    );
    let independent = read_fixture("INDEPENDENT-CHECK.json");
    assert_eq!(
        independent["manifest_hard_boundaries"],
        manifest["hard_boundaries"]
    );
}

#[test]
fn real_adm_control_identity_and_semantic_mapping_are_bound_but_execution_remains_blocked() {
    assert_package_manifest();
    let input = read_fixture("REAL-ADM-INPUT-IDENTITY.json");
    let providers = read_fixture("PROVIDER-IDENTITIES.json");
    let schema = read_fixture("SPATIAL-CONTROL-PRODUCTION.schema.json");
    let vectors = read_fixture("PRODUCTION-ADMISSION-VECTORS.json");
    let independent = read_fixture("INDEPENDENT-CHECK.json");
    let control_status = read_fixture("CONTROL38-MATERIAL-STATUS.json");
    let control_material = read_fixture("CONTROL38-CALLBACK0-MATERIAL.json");
    let control_lineage = read_fixture("CONTROL38-INPUT-LINEAGE.json");
    let semantic_map = read_fixture("CONTROL38-SEMANTIC-MAPPING.json");
    let cold_status = read_fixture("COLD-PARENT-STATE-STATUS.json");

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
    assert_eq!(
        input["control_identity"]["parsed_callback0_control38_sha256"],
        "2f1b942f1306700de77c987241e5f7a0b759cc906b601e153e667e3f58b4a679"
    );
    assert_eq!(
        input["control_identity"]["physical_mapping_sha256"],
        "d61e31ecfed16405bf7854e0c0e6872585c47a7a9beb2311c1f97c4b653ceb9b"
    );
    assert_eq!(
        input["control_identity"]["physical_status"],
        "CAPTURED_NATIVE_CALLBACK0"
    );
    assert_eq!(
        input["control_identity"]["semantic_binding_status"],
        "MAPPING_CAPTURED_ADMISSION_NOT_RUN"
    );
    assert_eq!(input["capture_lineage"]["same_input_sha256_proven"], true);
    assert_eq!(
        input["capture_lineage"]["lineage_id"],
        "control38-lineage-adm-7826dcbef37ef696-run102"
    );
    assert_eq!(
        input["capture_lineage"]["lineage_file"],
        "CONTROL38-INPUT-LINEAGE.json"
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

    assert_eq!(
        control_status["schema"],
        "sanitized-control38-material-status.v1"
    );
    assert_eq!(control_status["owner"], "frontend_control");
    assert_eq!(control_status["callback"], 0);
    assert_eq!(control_status["generation"], 1);
    assert_eq!(control_status["type"], "U32LE");
    assert_eq!(control_status["shape"], json!([38, 13]));
    assert_eq!(control_status["word_count"], 494);
    assert_eq!(control_status["byte_count"], 1976);
    assert_eq!(control_status["row_order"], "physical host index 0..37");
    assert_eq!(
        control_status["material_id"],
        "control38-callback0-2f1b942f1306700d"
    );
    assert_eq!(
        control_status["sha256"],
        "2f1b942f1306700de77c987241e5f7a0b759cc906b601e153e667e3f58b4a679"
    );
    assert_eq!(
        control_status["producer_address"],
        "dee_audio_filter_ddp_atmos.dll+0x1acd10"
    );
    assert_eq!(
        control_status["producer_caller"],
        "dee_audio_filter_ddp_atmos.dll+0x1a780d"
    );
    assert_eq!(
        control_status["status"],
        "PHYSICAL_CONTROL_AND_SEMANTIC_MAPPING_CAPTURED"
    );
    assert_eq!(
        control_status["physical_control38_status"],
        "CAPTURED_NATIVE_CALLBACK0"
    );
    assert_eq!(
        control_status["physical_mapping_status"],
        "CAPTURED_NATIVE_CALLBACK0"
    );
    assert_eq!(
        control_status["dbmd_track_to_bed_object_semantic_binding_map_status"],
        "CAPTURED_NATIVE_PLUS_ADM_DBMD_JOIN"
    );
    assert_eq!(
        control_status["dbmd_track_to_bed_object_semantic_binding_map_definition"]["bed_rows"],
        "physical bed row -> semantic bed identity and DBMD track binding"
    );
    assert_eq!(
        control_status["dbmd_track_to_bed_object_semantic_binding_map_definition"]["object_rows"],
        "physical object row -> object ordinal, event, sampler and DBMD track binding"
    );
    assert_eq!(control_status["missing"], json!([]));
    assert_eq!(control_status["old_fixture_hash_accepted"], false);
    assert_eq!(control_status["default_host_mapping_accepted"], false);
    assert_eq!(
        control_status["derived_binding_records_are_semantic_proof"],
        false
    );
    assert_eq!(
        control_status["semantic_mapping_file"],
        "CONTROL38-SEMANTIC-MAPPING.json"
    );
    assert_eq!(
        control_status["semantic_mapping_rows_sha256"],
        semantic_map["rows_sha256"]
    );

    assert_eq!(
        control_material["schema"],
        "sanitized-callback0-control38-material.v1"
    );
    assert_eq!(
        control_material["material_id"],
        control_status["material_id"]
    );
    assert_eq!(
        control_material["input_lineage_id"],
        input["capture_lineage"]["lineage_id"]
    );
    assert_eq!(control_material["input_id"], input["input_id"]);
    assert_eq!(control_material["input_sha256"], input["file_sha256"]);
    assert_eq!(control_material["callback"], 0);
    assert_eq!(control_material["generation"], 1);
    assert_eq!(control_material["owner"], "frontend_control");
    assert_eq!(control_material["type"], "U32LE");
    assert_eq!(control_material["shape"], json!([38, 13]));
    assert_eq!(control_material["row_stride_bytes"], 52);
    assert_eq!(control_material["word_count"], 494);
    assert_eq!(control_material["byte_count"], 1976);
    let control_bytes = u32le_rows(&control_material["u32_rows"]);
    assert_eq!(control_bytes.len(), 1976);
    assert_eq!(digest_hex(&control_bytes), control_material["sha256"]);
    let physical_bytes = u16le_values(&control_material["physical_binding"]["values"]);
    assert_eq!(physical_bytes.len(), 76);
    assert_eq!(
        digest_hex(&physical_bytes),
        control_material["physical_binding"]["sha256"]
    );
    assert_eq!(control_material["dbmd_event_binding"]["record_count"], 28);
    assert_eq!(control_material["dbmd_event_binding"]["record_bytes"], 2464);
    assert_eq!(
        control_material["dbmd_event_binding"]["record_stride_bytes"],
        88
    );
    assert_eq!(
        control_material["dbmd_event_binding"]["material"],
        "NOT_DELIVERED"
    );
    assert_eq!(
        control_material["dbmd_event_binding"]["identity_kind"],
        "DERIVED_RUNTIME_RECORDS_HASH_ONLY"
    );
    assert_eq!(
        control_material["dbmd_event_binding"]["dbmd_track_to_bed_object_semantic_binding_map"],
        "CAPTURED_NATIVE_PLUS_ADM_DBMD_JOIN"
    );
    assert_eq!(
        control_material["dbmd_event_binding"]["semantic_mapping_file"],
        "CONTROL38-SEMANTIC-MAPPING.json"
    );
    assert_eq!(
        control_material["dbmd_event_binding"]["semantic_mapping_rows_sha256"],
        semantic_map["rows_sha256"]
    );
    assert_eq!(control_material["producer"]["function_rva"], "0x1acd10");
    assert_eq!(control_material["producer"]["caller_rva"], "0x1a780d");
    assert_eq!(control_material["producer"]["parent_rva"], "0x1a7710");
    assert_eq!(control_material["producer"]["return_value"], "0x0");
    assert_eq!(control_material["capture"]["observer_errors"], 0);
    assert_eq!(control_material["capture"]["business_memory_writes"], false);
    assert_eq!(
        control_material["capture"]["raw_adm_pcm_metadata_delivered"],
        false
    );
    assert_eq!(control_material["capture"]["reference_frames"], 4247);
    assert_eq!(
        control_material["capture"]["interference_control"],
        "BYTE_EXACT_BASELINE_MATCH"
    );

    assert_eq!(
        semantic_map["schema"],
        "sanitized-control38-semantic-mapping.v1"
    );
    assert_eq!(semantic_map["input_id"], input["input_id"]);
    assert_eq!(semantic_map["callback"], 0);
    assert_eq!(semantic_map["generation"], 1);
    assert_eq!(semantic_map["row_count"], 38);
    assert_eq!(semantic_map["bed_row_count"], 10);
    assert_eq!(semantic_map["object_row_count"], 28);
    let semantic_rows = semantic_map["rows"].as_array().unwrap();
    assert_eq!(semantic_rows.len(), 38);
    let mut track_uids = HashSet::new();
    let mut bed_roles = HashSet::new();
    for (index, row) in semantic_rows.iter().enumerate() {
        let physical_host = u64::try_from(index).unwrap();
        let row_hash = digest_hex(&control_bytes[index * 52..(index + 1) * 52]);
        assert_eq!(row["physical_host"], physical_host);
        assert_eq!(row["track_index"], physical_host + 1);
        assert_eq!(row["dbmd_track"]["host_channel_index"], physical_host);
        assert_eq!(
            row["native_mapping_u16"],
            control_material["physical_binding"]["values"][index]
        );
        assert_eq!(row["control_row_sha256"], row_hash);
        assert!(track_uids.insert(row["track_uid"].as_str().unwrap().to_owned()));
        if index < 10 {
            assert_eq!(row["kind"], "bed");
            assert_eq!(row["bed_id"], physical_host);
            assert!(bed_roles.insert(row["semantic_role"].as_str().unwrap().to_owned()));
        } else {
            let object_ordinal = u64::try_from(index - 10).unwrap();
            assert_eq!(row["kind"], "object");
            assert_eq!(row["object_ordinal"], object_ordinal);
            assert_eq!(row["native_record_index"], object_ordinal);
            assert_eq!(row["sampler_control52_sha256"], row_hash);
            assert_eq!(row["native_control_wordwise_equal"], true);
            assert!(!row["audio_object_id"].as_str().unwrap().is_empty());
            assert_eq!(row["event"]["start_sample"], 0);
            assert_eq!(
                row["event"]["duration_samples"],
                input["wave"]["sample_frames"]
            );
        }
    }
    assert_eq!(track_uids.len(), 38);
    assert_eq!(bed_roles.len(), 10);
    assert_eq!(
        semantic_map["evidence"]["native_mapping_sha256"],
        control_material["physical_binding"]["sha256"]
    );
    assert_eq!(
        semantic_map["evidence"]["derived_records_sha256"],
        control_material["dbmd_event_binding"]["sha256"]
    );
    assert_eq!(
        semantic_map["evidence"]["dbmd_chunk_sha256"],
        input["chunks"]["dbmd"]["sha256"]
    );
    assert_eq!(
        semantic_map["evidence"]["axml_chunk_sha256"],
        input["chunks"]["axml"]["sha256"]
    );
    assert_eq!(
        semantic_map["evidence"]["chna_chunk_sha256"],
        input["chunks"]["chna"]["sha256"]
    );
    assert_eq!(
        semantic_map["evidence"]["object_control_rows_wordwise_equal"],
        28
    );
    assert!(is_sha256(&semantic_map["rows_sha256"]));
    assert_eq!(semantic_map["raw_metadata_included"], false);

    assert_eq!(
        control_lineage["schema"],
        "sanitized-control38-input-lineage.v1"
    );
    assert_eq!(
        control_lineage["lineage_id"],
        input["capture_lineage"]["lineage_id"]
    );
    assert_eq!(
        control_lineage["pinned_input"]["input_id"],
        input["input_id"]
    );
    assert_eq!(
        control_lineage["pinned_input"]["bytes"],
        input["file_bytes"]
    );
    assert_eq!(
        control_lineage["pinned_input"]["sha256"],
        input["file_sha256"]
    );
    assert_eq!(
        control_lineage["capture_command_input"]["byte_identical_alias"],
        true
    );
    assert_eq!(
        control_lineage["capture_command_input"]["sha256"],
        input["file_sha256"]
    );
    assert_eq!(
        control_lineage["original_fixture_identity_in_run_preflight"]["sha256"],
        input["file_sha256"]
    );
    assert_eq!(
        control_lineage["asset_identity"]["before_capture_matches"],
        true
    );
    assert_eq!(
        control_lineage["asset_identity"]["after_capture_matches"],
        true
    );
    assert_eq!(
        control_lineage["asset_identity"]["asset_identity_stable"],
        true
    );
    assert_eq!(control_lineage["capture_event"]["callback"], 0);
    assert_eq!(control_lineage["capture_event"]["generation"], 1);
    assert_eq!(
        control_lineage["capture_event"]["control_material_id"],
        control_material["material_id"]
    );
    assert_eq!(
        control_lineage["capture_event"]["control_sha256"],
        control_material["sha256"]
    );
    assert_eq!(
        control_lineage["interference_control"]["accepted_baseline_match"],
        true
    );
    assert_eq!(control_lineage["raw_paths_included"], false);
    assert_eq!(control_lineage["raw_input_material_included"], false);

    assert_eq!(
        cold_status["schema"],
        "sanitized-spatial-control-cold-state-status.v1"
    );
    assert_eq!(
        cold_status["field_order"],
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
    assert_eq!(cold_status["known_constructor"]["f01e_cold_bytes"], 4402);
    assert_eq!(
        cold_status["known_constructor"]["f01e_cold_sha256"],
        "9c943879cd36cb45bc7538346cfdf4f71d0998d8f261f75c135045fcb59870aa"
    );
    assert_eq!(cold_status["canonical_preimage_hex"], Value::Null);
    assert_eq!(cold_status["canonical_bytes"], Value::Null);
    assert_eq!(cold_status["sha256"], Value::Null);
    assert_eq!(cold_status["status"], "BLOCKED_NO_EXECUTABLE_SERIALIZER");
    assert_eq!(
        cold_status["missing"],
        json!([
            "top-level envelope/version and section framing",
            "nested F01E embedding/framing",
            "optional diffusion/source-cache record framing",
            "persistent serialization decision for current_flags/current_features",
            "writer-set record framing",
            "authoritative serializer execution with section offsets",
            "executable serializer entrypoint"
        ])
    );
    assert_eq!(
        cold_status["static_ida_evidence"]["serializer_call_observed"],
        false
    );
    assert_eq!(
        cold_status["static_ida_evidence"]["schema_magic_strings_found"],
        false
    );
    assert_eq!(cold_status["scratch_included"], false);
    assert_eq!(cold_status["unknown_zero_fill_allowed"], false);
    assert_eq!(cold_status["ai48st01_accepted"], false);
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
    assert_eq!(
        vectors["unified_status_v10"]["physical_control38_material"],
        "CAPTURED_NATIVE_CALLBACK0_INPUT_LINEAGE_BOUND"
    );
    assert_eq!(
        vectors["unified_status_v10"]["dbmd_track_to_bed_object_semantic_binding_map"],
        "CAPTURED_NATIVE_PLUS_ADM_DBMD_JOIN"
    );
    assert_eq!(
        vectors["unified_status_v10"]["callback0_numerical_execution"],
        "NOT_RUN"
    );
    assert_eq!(
        vectors["unified_status_v10"]["callback_semantic_admission"],
        "NOT_RUN"
    );
    assert_eq!(
        vectors["unified_status_v10"]["control_mapping_admission_boundary"],
        "MAPPING_CAPTURED_ADMISSION_NOT_RUN"
    );
    assert_eq!(
        vectors["unified_status_v10"]["parent_cold_serializer"],
        "BLOCKED_NO_EXECUTABLE_SERIALIZER"
    );
    assert_eq!(vectors["unified_status_v10"]["success_vectors"], 0);
    assert_eq!(vectors["unified_status_v10"]["production_allowed"], false);
    let captured = &vectors["captured_precondition"];
    assert_eq!(captured["input_id"], input["input_id"]);
    assert_eq!(captured["input_sha256"], input["file_sha256"]);
    assert_eq!(captured["input_lineage_id"], control_lineage["lineage_id"]);
    assert_eq!(
        captured["control_material_id"],
        control_material["material_id"]
    );
    assert_eq!(captured["control_sha256"], control_material["sha256"]);
    assert_eq!(
        captured["status"],
        "CAPTURED_NATIVE_CALLBACK0_INPUT_LINEAGE_BOUND"
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
    assert_eq!(checks.len(), 16);
    assert!(checks.iter().all(|check| check["pass"] == true));
    assert_eq!(
        independent["schema"],
        "spatial-control-production-admission-independent-check.v10"
    );
    assert_eq!(independent["all_pass"], true);
    assert_eq!(
        independent["manifest_file"],
        "SPATIAL-CONTROL-PRODUCTION-ADMISSION-MANIFEST.json"
    );
    assert_eq!(
        independent["manifest_hard_boundaries_sha256"],
        "8301ff69fd12fb3d4c8ed97f4e79b83bc9885283c4aec1d4914246e54b47c374"
    );
    assert_eq!(
        independent["input_files"],
        json!([
            "SPATIAL-CONTROL-PRODUCTION-ADMISSION-MANIFEST.json",
            "REAL-ADM-INPUT-IDENTITY.json",
            "CONTROL38-INPUT-LINEAGE.json",
            "PROVIDER-IDENTITIES.json",
            "PRODUCTION-ADMISSION-VECTORS.json",
            "SPATIAL-CONTROL-PRODUCTION.schema.json",
            "CONTROL38-MATERIAL-STATUS.json",
            "CONTROL38-CALLBACK0-MATERIAL.json",
            "CONTROL38-SEMANTIC-MAPPING.json",
            "COLD-PARENT-STATE-STATUS.json"
        ])
    );
    assert_eq!(independent["raw_adm_read"], false);
    assert_eq!(independent["research_material_accessed"], false);
    assert_eq!(independent["production_execution"], false);
    assert_eq!(independent["expected_used_as_seed"], false);
}
