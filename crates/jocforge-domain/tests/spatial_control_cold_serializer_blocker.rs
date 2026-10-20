// Imperative Shell: validate the exact washed blocker fixture without loading research material.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

const MANIFEST_SHA256: &str = "052c292608b0a3f3de60954dde270ce9aa1c994c35515b904c422dc4dbcb7fde";

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
    assert_eq!(manifest["schema"], "washed-implementer-blocker-manifest.v3");
    assert_eq!(
        manifest["identity"],
        "spatial-control-cold-serializer.blocked.v8"
    );
    assert_eq!(manifest["status"], "BLOCKED_NO_EXECUTABLE_SERIALIZER");
    assert_eq!(
        manifest["evidence_classification"],
        json!({
            "native_observed": "bounded parent output to FileWriter static chain plus four dynamic exact joins",
            "unknown": "alternate host/job/wrapper/other DLL paths and product-wide state persistence outside bounded run",
            "clean_room_design": "parent canonical bytes and receipt schema not native observed"
        })
    );

    let files = manifest["files"].as_array().expect("manifest files array");
    assert_eq!(
        files
            .iter()
            .map(|entry| entry["file"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "COLD-SERIALIZER-EVIDENCE.json",
            "COLD-SERIALIZER-FINAL-BLOCKER.md",
            "INDEPENDENT-CHECK.json",
            "MINIMUM-EXTERNAL-EVIDENCE.json",
            "PARENT-FILEWRITER-DYNAMIC-JOIN.json",
            "PARENT-OUTPUT-DATAFLOW.json",
            "PARENT-OUTPUT-IMPLEMENTER-SUPPLEMENT.json",
            "REQUIRED-FRAMING-RULES.json",
            "UNCHANGED-BLOCKER-AUDIT.json"
        ]
    );
    let declared_independent_inputs = json!([
        "COLD-SERIALIZER-EVIDENCE.json",
        "REQUIRED-FRAMING-RULES.json",
        "MINIMUM-EXTERNAL-EVIDENCE.json",
        "UNCHANGED-BLOCKER-AUDIT.json",
        "PARENT-OUTPUT-DATAFLOW.json",
        "PARENT-OUTPUT-IMPLEMENTER-SUPPLEMENT.json",
        "PARENT-FILEWRITER-DYNAMIC-JOIN.json"
    ]);
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
        "299ab9df4d11ba867933884cff38aaa21198be59a386dfa7bf33287f49f5a370"
    );
    assert_eq!(
        source["research_independent_check_sha256"],
        "1d9b67849e6ba3e993ad064b24973d9d5781638588a233f5fdc79efbe71f5804"
    );
    assert_eq!(
        source["dynamic_join_sha256"],
        "3e4b4d02c2d7421fe8ecd713ed969da185d0759ae1fe5182bfd0087ca0995b69"
    );
    assert_eq!(
        source["complete_chain_identity_sha256"],
        "db114ca682e283dc351551e1a722a91ddaaee1d8f5d90fdf62b36262545cd7f2"
    );
    assert_eq!(source["serializer_execution"], "NOT_RUN");
    assert_eq!(source["canonical_preimage"], Value::Null);
    assert_eq!(source["authoritative_length"], Value::Null);
    assert_eq!(source["authoritative_sha256"], Value::Null);
    assert_eq!(source["bytes_guessed"], false);
    assert_eq!(source["AI48ST01_used"], false);
    assert_eq!(source["private_auth"], "excluded");
    assert_eq!(source["native_research_artifacts_in_package"], false);
    assert_eq!(source["product_wide_absence_claim"], false);
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
    assert_eq!(
        manifest["hard_boundaries"]["serializer_execution"],
        "NOT_RUN"
    );
    assert_eq!(
        manifest["hard_boundaries"]["parent_canonical_bytes"],
        "NOT_DELIVERED"
    );
    assert_eq!(
        manifest["hard_boundaries"]["direct_fwrite_hook"],
        "NOT_CAPTURED"
    );
    assert_eq!(manifest["production_allowed"], false);
    assert_eq!(manifest["dataflow"]["chain"].as_array().unwrap().len(), 13);
    assert_eq!(
        manifest["dataflow"]["file_writer_core_consumer"],
        "0x189c80"
    );
    assert_eq!(
        manifest["dataflow"]["dynamic_join_result"],
        "length and CRC32 exact for all four pairs"
    );
    assert_eq!(
        manifest["dataflow"]["bounded_snapshot_receipt_persistence_calls"],
        0
    );
    assert_eq!(
        manifest["reference_output"]["extracted_ec3_sha256"],
        "34d18a6fb0f477cbccd0b7d633a80b4191c0c535385ff66b8d9cbf7ff1440448"
    );
    assert_eq!(manifest["reference_output"]["frames"], 4247);
    assert_eq!(manifest["reference_output"]["bytes"], 17_395_712);
    assert_eq!(manifest["reference_output"]["baseline_match"], true);

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

    let audit = read_json("UNCHANGED-BLOCKER-AUDIT.json");
    assert_eq!(
        audit["schema"],
        "sanitized-spatial-control-cold-serializer-incremental-audit.v1"
    );
    assert_eq!(
        audit["classification"],
        "NATIVE_OBSERVED_PLUS_AUTHORIZED_EVIDENCE_NO_HIT"
    );
    assert_eq!(
        audit["evidence_classification"]["NATIVE_OBSERVED"]["scope"],
        "bounded parent callback path rooted at RVA 0x1a7710"
    );
    assert_eq!(
        audit["evidence_classification"]["NATIVE_OBSERVED"]["coverage"]["functions_visited"],
        73
    );
    assert_eq!(
        audit["evidence_classification"]["NATIVE_OBSERVED"]["coverage"]["call_edges_observed"],
        211
    );
    assert_eq!(
        audit["evidence_classification"]["UNKNOWN"]["serializer_existence"],
        "UNKNOWN"
    );
    assert_eq!(
        audit["evidence_classification"]["UNKNOWN"]["product_wide_absence_claim"],
        false
    );
    assert_eq!(
        audit["evidence_classification"]["CLEAN_ROOM_DESIGN"]["parent_canonical_bytes"],
        "DESIGN_ONLY_UNAPPROVED_AND_NOT_DELIVERED"
    );
    assert_eq!(
        audit["evidence_classification"]["CLEAN_ROOM_DESIGN"]["receipt_schema"],
        "DESIGN_ONLY_NOT_NATIVE_OBSERVED"
    );
    assert_eq!(
        audit["evidence_classification"]["CLEAN_ROOM_DESIGN"]["native_format_claim"],
        false
    );
    let ida = &audit["ida_direct_execution"];
    assert_eq!(ida["module_sha256"], evidence["dll"]["sha256"]);
    assert_eq!(ida["root_rva"], "0x1a7710");
    assert_eq!(ida["maximum_call_depth"], 3);
    assert_eq!(ida["functions_visited"], 73);
    assert_eq!(ida["call_edges_observed"], 211);
    assert_eq!(ida["serialization_entrypoint_observed"], false);
    assert_eq!(
        ida["only_memory_primitive_hits"],
        json!(["memset", "memmove"])
    );
    assert_eq!(
        ida["native_behavior"],
        "parent callback advances descriptors, control rows, planar working buffers and downstream runtime processing in memory"
    );
    assert_eq!(
        audit["excluded_authorized_candidate"]["classification"],
        "EXCLUDED_DOWNSTREAM_PAYLOAD_AND_SPATIAL_OUTPUT_SERIALIZER"
    );
    assert_eq!(audit["new_authoritative_serializer_evidence_count"], 0);
    assert_eq!(audit["new_complete_framing_evidence_count"], 0);
    assert_eq!(
        audit["blocker_status"],
        "UNCHANGED_BLOCKED_NO_EXECUTABLE_SERIALIZER_OR_COMPLETE_FRAMING"
    );
    assert_eq!(audit["bytes_guessed"], false);

    let dataflow = read_json("PARENT-OUTPUT-DATAFLOW.json");
    assert_eq!(
        dataflow["schema"],
        "sanitized-spatial-control-parent-output-dataflow.v1"
    );
    assert_eq!(dataflow["module"]["sha256"], evidence["dll"]["sha256"]);
    assert_eq!(
        dataflow["NATIVE_OBSERVED"]["dynamic_owner_join"]["frame_samples"],
        1536
    );
    assert_eq!(
        dataflow["NATIVE_OBSERVED"]["dynamic_owner_join"]["raw_audio_or_metadata_persisted"],
        false
    );
    let output_flow = &dataflow["NATIVE_OBSERVED"]["static_dataflow"];
    assert_eq!(output_flow["host_process_rva"], "0x179ab0");
    assert_eq!(output_flow["parent_api_rva"], "0x1a8840");
    assert_eq!(output_flow["parent_callback_rva"], "0x1a7710");
    assert_eq!(output_flow["runtime_packet_builder_rva"], "0x1b1590");
    assert_eq!(
        output_flow["parent_output_storage"],
        "4096-byte host stack buffer plus returned byte count"
    );
    assert_eq!(
        output_flow["immediate_host_action"],
        "copy produced byte range into a transient owned byte vector"
    );
    assert_eq!(output_flow["file_writer_forwarder_rva"], "0x189c70");
    assert_eq!(output_flow["file_writer_core_write_rva"], "0x189c80");
    assert_eq!(
        output_flow["file_writer_action"],
        "select FILE by frame-ordinal thresholds and require fwrite(encoded_pointer,1,encoded_size,selected_file) to equal encoded_size"
    );
    assert_eq!(output_flow["parent_state_object_persisted"], false);
    assert_eq!(
        dataflow["NATIVE_OBSERVED"]["bounded_chain_conclusion"],
        "the joined parent owner advances in memory and returns encoded bytes; the traced host chain forwards those bytes to its sink rather than saving the parent state object"
    );
    assert_eq!(dataflow["raw_pcm_or_metadata_delivered"], false);
    assert_eq!(
        dataflow["NATIVE_OBSERVED"]["bounded_chain_snapshot_receipt_persistence_calls"],
        0
    );
    assert_eq!(
        dataflow["UNKNOWN"]["product_wide_snapshot_or_serializer_existence"],
        "UNKNOWN"
    );
    assert_eq!(
        dataflow["UNKNOWN"]["file_writer_dynamic_hit"],
        "CAPTURED_FOUR_EXACT_BYTE_RANGE_JOINS"
    );
    assert_eq!(dataflow["UNKNOWN"]["fwrite_dynamic_hit"], "NOT_CAPTURED");
    assert_eq!(dataflow["UNKNOWN"]["product_wide_absence_claim"], false);
    assert_eq!(
        dataflow["CLEAN_ROOM_DESIGN"]["canonical_parent_bytes"],
        "DESIGN_ONLY_UNAPPROVED_AND_NOT_DELIVERED"
    );
    assert!(
        dataflow["NOT_RUN"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item == "new native execution")
    );

    let output_supplement = read_json("PARENT-OUTPUT-IMPLEMENTER-SUPPLEMENT.json");
    assert_eq!(
        output_supplement["schema"],
        "sanitized-parent-output-implementer-supplement.v1"
    );
    let chain_identity = &output_supplement["complete_chain_identity"];
    assert_eq!(chain_identity["module_sha256"], evidence["dll"]["sha256"]);
    assert_eq!(chain_identity["ordered_rvas"].as_array().unwrap().len(), 13);
    assert_eq!(chain_identity["ordered_rvas"][10], "0x189c70");
    assert_eq!(chain_identity["ordered_rvas"][11], "0x189c80");
    assert_eq!(
        output_supplement["chain_nodes"][10]["classification"],
        "FILE_WRITER_FORWARDER"
    );
    assert_eq!(
        output_supplement["chain_nodes"][11]["classification"],
        "FILE_WRITER_CORE_WRITE"
    );
    assert_eq!(
        output_supplement["complete_chain_identity_sha256"],
        "db114ca682e283dc351551e1a722a91ddaaee1d8f5d90fdf62b36262545cd7f2"
    );
    assert_eq!(
        output_supplement["observed_storage_boundaries"]["temporary_output"]["capacity_bytes"],
        4096
    );
    assert_eq!(
        output_supplement["observed_storage_boundaries"]["file_output"]["write_rule"],
        "fwrite(encoded_pointer, 1, encoded_size, selected_file) must equal encoded_size"
    );
    assert_eq!(
        output_supplement["state_and_persistence_scope"]["bounded_chain_parent_state_persistence_calls_observed"],
        0
    );
    assert_eq!(
        output_supplement["state_and_persistence_scope"]["encoded_bitstream_file_persistence_observed_statically"],
        true
    );
    assert_eq!(
        output_supplement["state_and_persistence_scope"]["product_wide_absence_claim"],
        false
    );
    assert_eq!(output_supplement["UNKNOWN"].as_array().unwrap().len(), 6);
    assert_eq!(
        output_supplement["CLEAN_ROOM_DESIGN"]["parent_canonical_bytes"],
        "DESIGN_ONLY_UNAPPROVED_AND_NOT_DELIVERED"
    );
    assert_eq!(output_supplement["production_allowed"], false);
    assert_eq!(
        output_supplement["dynamic_evidence"]["file_writer_runtime_hit"],
        "CAPTURED_FOUR_EXACT_BYTE_RANGE_JOINS"
    );
    assert_eq!(
        output_supplement["dynamic_evidence"]["fwrite_runtime_hit"],
        "NOT_CAPTURED"
    );
    let final_blocker = fs::read_to_string(root.join("COLD-SERIALIZER-FINAL-BLOCKER.md"))
        .expect("human-readable blocker is manifested");
    let normalized_final_blocker = final_blocker
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        normalized_final_blocker
            .contains("This does not prove that no internal serialization could ever be written")
    );

    let dynamic_join = read_json("PARENT-FILEWRITER-DYNAMIC-JOIN.json");
    assert_eq!(
        dynamic_join["schema"],
        "sanitized-parent-filewriter-dynamic-join.v1"
    );
    assert_eq!(dynamic_join["run_identity"], "music_rectangular_native_104");
    assert_eq!(
        dynamic_join["run_scope"],
        "first four nonzero parent outputs and first four FileWriter writes"
    );
    assert_eq!(
        dynamic_join["reference_output"]["extracted_ec3_sha256"],
        "34d18a6fb0f477cbccd0b7d633a80b4191c0c535385ff66b8d9cbf7ff1440448"
    );
    assert_eq!(dynamic_join["reference_output"]["baseline_match"], true);
    assert_eq!(dynamic_join["reference_output"]["frames"], 4247);
    assert_eq!(dynamic_join["reference_output"]["bytes"], 17_395_712);
    assert_eq!(dynamic_join["reference_output"]["frame_bytes"], 4096);
    assert_eq!(
        dynamic_join["joined_pairs"],
        json!([
            {"parent_generation":5,"file_writer_frame_ordinal":0,"byte_count":4096,"crc32":"e48334d7","length_match":true,"crc32_match":true},
            {"parent_generation":6,"file_writer_frame_ordinal":1,"byte_count":4096,"crc32":"88d0f270","length_match":true,"crc32_match":true},
            {"parent_generation":7,"file_writer_frame_ordinal":2,"byte_count":4096,"crc32":"6020ebbb","length_match":true,"crc32_match":true},
            {"parent_generation":8,"file_writer_frame_ordinal":3,"byte_count":4096,"crc32":"ef5e9494","length_match":true,"crc32_match":true}
        ])
    );
    assert_eq!(dynamic_join["join"]["thread_identity_match"], true);
    assert_eq!(
        dynamic_join["join"]["parent_stack_and_writer_vector_pointers_distinct"],
        true
    );
    assert_eq!(dynamic_join["join"]["all_four_pairs_exact"], true);
    assert_eq!(dynamic_join["file_writer"]["core_write_rva"], "0x189c80");
    assert_eq!(dynamic_join["file_writer"]["closed_before_all_writes"], 0);
    assert_eq!(dynamic_join["file_writer"]["selected_file_index"], 0);
    assert_eq!(
        dynamic_join["file_writer"]["all_four_returned_normally"],
        true
    );
    assert_eq!(
        dynamic_join["file_writer"]["direct_fwrite_hook"],
        "NOT_CAPTURED"
    );
    assert_eq!(
        dynamic_join["NATIVE_OBSERVED"]["writer_boundary_parent_state_pointer"],
        "NOT_PRESENT"
    );
    assert_eq!(
        dynamic_join["NATIVE_OBSERVED"]["writer_boundary_control_or_target_pointer"],
        "NOT_PRESENT"
    );
    assert_eq!(
        dynamic_join["NATIVE_OBSERVED"]["parent_state_snapshot_or_receipt_observed"],
        false
    );
    assert!(
        dynamic_join["UNKNOWN"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item == "direct fwrite runtime call identity")
    );
    assert_eq!(dynamic_join["product_wide_absence_claim"], false);
    assert_eq!(dynamic_join["production_allowed"], false);
    assert_eq!(dynamic_join["raw_encoded_bytes_delivered"], false);
    assert_eq!(dynamic_join["raw_pcm_or_metadata_delivered"], false);

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
        "spatial-control-cold-serializer-blocker-independent-check.v8"
    );
    assert_eq!(independent["all_pass"], true);
    let checks = independent["checks"].as_array().expect("checks array");
    assert_eq!(checks.len(), 17);
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
            "incremental_ida_audit",
            "evidence_classification",
            "parent_output_dataflow",
            "implementer_supplement",
            "dynamic_parent_filewriter_join",
            "no_entrypoint",
            "no_bytes",
            "framing_rules",
            "field_order",
            "minimum_external_evidence",
            "declared_inputs_present"
        ]
    );
    assert_eq!(independent["input_files"], declared_independent_inputs);
    assert_eq!(independent["research_files_read"], false);
    assert_eq!(independent["serializer_execution"], false);
    assert_eq!(independent["bytes_guessed"], false);
    assert_eq!(independent["expected_used_as_seed"], false);
    assert_eq!(independent["production_allowed"], false);
}
