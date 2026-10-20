// Imperative Shell: validate the exact washed blocker fixture without loading research material.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

const MANIFEST_SHA256: &str = "7be31dd9a33f35edd0f7f339c0d33e55a2d0b4103ad6508d6155c14361bf89c2";

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
    assert_eq!(
        manifest["schema"],
        "washed-implementer-blocker-manifest.v21"
    );
    assert_eq!(
        manifest["identity"],
        "spatial-control-cold-serializer.blocked.v26"
    );
    assert_eq!(manifest["status"], "BLOCKED_NO_EXECUTABLE_SERIALIZER");
    assert_eq!(
        manifest["policy29_36_identity"],
        json!({
            "property29": "adconvtyp",
            "property29_value": 0,
            "property29_config_offset": 192,
            "property36": "roomtyp",
            "property36_value": 0,
            "property36_config_offset": 220,
            "identity_join_status": "CLOSED_CURRENT_RUN109"
        })
    );
    assert_eq!(
        manifest["remaining_semantics_no_hit"],
        json!({
            "measurement_result_offsets": [8, 12, 16],
            "measurement_names": "NOT_CAPTURED",
            "portable_order_endian_version_optional_checksum": "NOT_OBSERVED",
            "serializer_restore_roundtrip": "NOT_RUN",
            "product_wide_absence_claim": false
        })
    );
    assert_eq!(manifest["status"], "BLOCKED_NO_EXECUTABLE_SERIALIZER");

    let files = manifest["files"].as_array().expect("manifest files array");
    assert_eq!(
        files
            .iter()
            .map(|entry| entry["file"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "AUTHORIZED-PE-PORTABLE-API-AND-MEASUREMENT-NO-HIT.json",
            "CHANNEL-EXTERNAL-METADATA-TRIM-SEMANTICS.json",
            "COLD-SERIALIZER-EVIDENCE.json",
            "COLD-SERIALIZER-FINAL-BLOCKER.md",
            "EXTERNAL-METADATA-HELPER-SEMANTICS.json",
            "EXTERNAL-METADATA-STRUCTURE-AND-REMAINING-FIELDS.json",
            "FILEWRITER-FWRITE-DYNAMIC-JOIN.json",
            "INDEPENDENT-CHECK.json",
            "MEASUREMENT-ALIAS-RESOLUTION.json",
            "MEASUREMENT-INTERNAL-STRUCT-EXTENDED-NO-HIT.json",
            "MINIMUM-EXTERNAL-EVIDENCE.json",
            "PARENT-COLD-CONFIG-BOUNDARY.json",
            "PARENT-COLD-CONFIG-SEMANTICS.json",
            "PARENT-COLD-LIFECYCLE.json",
            "PARENT-COLD-SEMANTIC-CORRECTIONS.json",
            "PARENT-FILEWRITER-DYNAMIC-JOIN.json",
            "PARENT-OUTPUT-DATAFLOW.json",
            "PARENT-OUTPUT-IMPLEMENTER-SUPPLEMENT.json",
            "PARENT-PROPERTY-PAIRS-AUTHORITY.json",
            "POLICY29-36-IDENTITY-JOIN.json",
            "PORTABLE-CONTRACT-AND-REMAINING-SEMANTICS-NO-HIT.json",
            "PROPERTY-NAME-AND-PORTABLE-CONTRACT-EXTENDED-NO-HIT.json",
            "REQUIRED-FRAMING-RULES.json",
            "TRIM-PAYLOAD-PRODUCER-CONSUMER-EVIDENCE.json",
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
        "PARENT-FILEWRITER-DYNAMIC-JOIN.json",
        "FILEWRITER-FWRITE-DYNAMIC-JOIN.json",
        "PARENT-COLD-LIFECYCLE.json",
        "PARENT-COLD-CONFIG-BOUNDARY.json",
        "PARENT-COLD-CONFIG-SEMANTICS.json",
        "PARENT-COLD-SEMANTIC-CORRECTIONS.json",
        "MEASUREMENT-ALIAS-RESOLUTION.json",
        "CHANNEL-EXTERNAL-METADATA-TRIM-SEMANTICS.json",
        "TRIM-PAYLOAD-PRODUCER-CONSUMER-EVIDENCE.json",
        "EXTERNAL-METADATA-STRUCTURE-AND-REMAINING-FIELDS.json",
        "EXTERNAL-METADATA-HELPER-SEMANTICS.json",
        "PORTABLE-CONTRACT-AND-REMAINING-SEMANTICS-NO-HIT.json",
        "PARENT-PROPERTY-PAIRS-AUTHORITY.json",
        "PROPERTY-NAME-AND-PORTABLE-CONTRACT-EXTENDED-NO-HIT.json",
        "POLICY29-36-IDENTITY-JOIN.json",
        "AUTHORIZED-PE-PORTABLE-API-AND-MEASUREMENT-NO-HIT.json",
        "MEASUREMENT-INTERNAL-STRUCT-EXTENDED-NO-HIT.json"
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
        "9a5e7398a2d0bae99e8caae011ac730167a104679931416ef8a5866f4b68f101"
    );
    assert_eq!(
        source["research_independent_check_sha256"],
        "8e74aa17c77519e1eb091d23cf937d009077bcbdbba625f1ce318e22230edd06"
    );
    assert_eq!(
        source["measurement_internal_struct_no_hit_sha256"],
        "39a3f6d937fb8c24eb2c1459ec3863ce1ff303bc914cf732778347b3296beb9b"
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
        manifest["parent_output_dataflow"]["file_writer_core_write_rva"],
        "0x189c80"
    );
    assert_eq!(
        manifest["parent_output_dataflow"]["file_writer_fwrite_callsite_rva"],
        "0x189d3a"
    );
    assert_eq!(
        manifest["parent_output_dataflow"]["module_local_fwrite_target_rva"],
        "0x5aea6c"
    );
    assert_eq!(
        manifest["parent_output_dataflow"]["direct_fwrite_hook"],
        "CAPTURED_FOUR_EXACT_FULL_WRITES"
    );
    assert_eq!(
        manifest["parent_output_dataflow"]["bounded_snapshot_receipt_persistence_calls"],
        0
    );
    let cold_lifecycle_summary = &manifest["parent_cold_lifecycle"];
    assert_eq!(cold_lifecycle_summary["query_rva"], "0x1a8920");
    assert_eq!(cold_lifecycle_summary["constructor_rva"], "0x1a7d50");
    assert_eq!(cold_lifecycle_summary["first_consumer_rva"], "0x1a8840");
    assert_eq!(cold_lifecycle_summary["allocation_bytes"], 7_805_609);
    assert_eq!(cold_lifecycle_summary["owner_offset_bytes"], 0);
    assert_eq!(
        cold_lifecycle_summary["constructor_to_first_consumer_digest_match"],
        true
    );
    assert_eq!(
        cold_lifecycle_summary["portable_canonical_bytes"],
        "NOT_CAPTURED"
    );
    assert_eq!(cold_lifecycle_summary["raw_allocation_delivered"], false);
    let descriptor_summary = &manifest["descriptor_semantics"];
    assert_eq!(descriptor_summary["descriptor_bytes"], 120);
    assert_eq!(descriptor_summary["layout_complete"], true);
    assert_eq!(descriptor_summary["portable_reconstruction_ready"], false);
    assert_eq!(
        descriptor_summary["native_portable_contract"],
        "NOT_OBSERVED"
    );
    assert_eq!(
        descriptor_summary["native_serializer_restore_roundtrip"],
        "NOT_RUN"
    );
    assert_eq!(descriptor_summary["policy_block_semantics_complete"], true);
    assert_eq!(descriptor_summary["measurement_semantics_complete"], false);
    assert_eq!(
        descriptor_summary["channel_record_semantics_complete"],
        true
    );
    assert_eq!(
        descriptor_summary["external_metadata_semantics_complete"],
        false
    );
    assert_eq!(descriptor_summary["trim_record_semantics_complete"], true);
    assert_eq!(
        descriptor_summary["native_portable_contract"],
        "NOT_OBSERVED"
    );
    assert_eq!(
        descriptor_summary["native_serializer_restore_roundtrip"],
        "NOT_RUN"
    );
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
    assert_eq!(
        dataflow["UNKNOWN"]["fwrite_dynamic_hit"],
        "CAPTURED_FOUR_EXACT_FULL_WRITES"
    );
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
        "18158256d3e8ed6b774d9fa6afa89ebdc1ed38ca5297cb847a38d006cc742644"
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
        "CAPTURED_FOUR_EXACT_FULL_WRITES"
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
        "CAPTURED_SEPARATE_RUN_106"
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

    let fwrite_join = read_json("FILEWRITER-FWRITE-DYNAMIC-JOIN.json");
    assert_eq!(
        fwrite_join["schema"],
        "sanitized-filewriter-fwrite-dynamic-join.v1"
    );
    assert_eq!(fwrite_join["run_identity"], "music_rectangular_native_106");
    assert_ne!(dynamic_join["run_identity"], fwrite_join["run_identity"]);
    assert_eq!(
        fwrite_join["native_boundaries"]["scope"],
        "first four FileWriter writes and their immediately nested local fwrite calls"
    );
    assert_eq!(fwrite_join["capture_accounting"]["observer_errors"], 0);
    assert_eq!(
        fwrite_join["capture_accounting"]["postrun_ec3_baseline_match"],
        true
    );
    assert_eq!(
        fwrite_join["native_boundaries"]["file_writer_core_rva"],
        "0x189c80"
    );
    assert_eq!(
        fwrite_join["native_boundaries"]["fwrite_caller_return_rva"],
        "0x189d3a"
    );
    assert_eq!(
        fwrite_join["native_boundaries"]["fwrite_local_crt_rva"],
        "0x5aea6c"
    );
    assert_eq!(
        fwrite_join["reference_output"]["extracted_ec3_sha256"],
        "34d18a6fb0f477cbccd0b7d633a80b4191c0c535385ff66b8d9cbf7ff1440448"
    );
    assert_eq!(fwrite_join["reference_output"]["frames"], 4247);
    assert_eq!(fwrite_join["reference_output"]["bytes"], 17_395_712);
    assert_eq!(fwrite_join["reference_output"]["frame_bytes"], 4096);
    assert_eq!(fwrite_join["reference_output"]["baseline_match"], true);
    assert_eq!(fwrite_join["joined_writes"].as_array().unwrap().len(), 4);
    let expected_fwrite_pairs = [
        (0, "e48334d7"),
        (1, "88d0f270"),
        (2, "6020ebbb"),
        (3, "ef5e9494"),
    ];
    for (write, (ordinal, crc32)) in fwrite_join["joined_writes"]
        .as_array()
        .unwrap()
        .iter()
        .zip(expected_fwrite_pairs)
    {
        assert_eq!(write["frame_ordinal"], ordinal);
        assert_eq!(write["byte_count"], 4096);
        assert_eq!(write["crc32"], crc32);
        assert_eq!(write["buffer_pointer_match"], true);
        assert_eq!(write["length_match"], true);
        assert_eq!(write["crc32_match"], true);
        assert_eq!(write["file_pointer_match"], true);
        assert_eq!(write["returned_items"], 4096);
        assert_eq!(write["full_write"], true);
    }
    assert_eq!(
        fwrite_join["NATIVE_OBSERVED"]["parent_state_snapshot_or_receipt_at_writer_boundary"],
        false
    );
    assert_eq!(fwrite_join["product_wide_absence_claim"], false);
    assert_eq!(fwrite_join["production_allowed"], false);
    assert_eq!(fwrite_join["raw_encoded_bytes_delivered"], false);
    assert_eq!(fwrite_join["raw_pcm_or_metadata_delivered"], false);

    let cold_lifecycle = read_json("PARENT-COLD-LIFECYCLE.json");
    assert_eq!(
        cold_lifecycle["schema"],
        "sanitized-parent-cold-lifecycle.v1"
    );
    assert_eq!(
        cold_lifecycle["run_identity"],
        "music_rectangular_native_107"
    );
    assert_eq!(
        cold_lifecycle["module"]["sha256"],
        evidence["dll"]["sha256"]
    );
    assert_eq!(
        cold_lifecycle["reference_output"]["extracted_ec3_sha256"],
        "34d18a6fb0f477cbccd0b7d633a80b4191c0c535385ff66b8d9cbf7ff1440448"
    );
    assert_eq!(cold_lifecycle["reference_output"]["baseline_match"], true);
    assert_eq!(cold_lifecycle["reference_output"]["frames"], 4247);
    assert_eq!(cold_lifecycle["reference_output"]["bytes"], 17_395_712);
    assert_eq!(cold_lifecycle["native_chain"].as_array().unwrap().len(), 4);
    assert_eq!(cold_lifecycle["native_chain"][0]["rva"], "0x1788a0");
    assert_eq!(
        cold_lifecycle["native_chain"][0]["classification"],
        "HOST_LAZY_INITIALIZER"
    );
    assert_eq!(cold_lifecycle["native_chain"][1]["rva"], "0x1a8920");
    assert_eq!(
        cold_lifecycle["native_chain"][1]["classification"],
        "PARENT_MEMORY_QUERY"
    );
    assert_eq!(cold_lifecycle["native_chain"][2]["rva"], "0x1a7d50");
    assert_eq!(
        cold_lifecycle["native_chain"][2]["classification"],
        "PARENT_CONSTRUCTOR"
    );
    assert_eq!(cold_lifecycle["native_chain"][3]["rva"], "0x1a8840");
    assert_eq!(
        cold_lifecycle["native_chain"][3]["classification"],
        "PARENT_FIRST_CONSUMER"
    );
    let native_lifecycle = &cold_lifecycle["NATIVE_OBSERVED"];
    assert_eq!(native_lifecycle["query_return"], 0);
    assert_eq!(native_lifecycle["allocation_bytes"], 7_805_609);
    assert_eq!(
        native_lifecycle["constructor_entry_allocation_all_zero"],
        true
    );
    assert_eq!(native_lifecycle["constructor_entry_crc32"], "f97e2af0");
    assert_eq!(native_lifecycle["constructor_return"], 0);
    assert_eq!(native_lifecycle["owner_offset_bytes"], 0);
    assert_eq!(native_lifecycle["owner_equals_allocation_base"], true);
    assert_eq!(native_lifecycle["owner_inside_allocation"], true);
    assert_eq!(native_lifecycle["constructor_return_crc32"], "83714dd6");
    assert_eq!(native_lifecycle["constructor_changed_allocation"], true);
    assert_eq!(native_lifecycle["first_consumer_rva"], "0x1a8840");
    assert_eq!(native_lifecycle["first_consumer_count"], 1);
    assert_eq!(native_lifecycle["first_consumer_crc32"], "83714dd6");
    assert_eq!(
        native_lifecycle["constructor_to_first_consumer_crc32_match"],
        true
    );
    assert_eq!(native_lifecycle["same_thread_lifecycle"], true);
    assert_eq!(
        cold_lifecycle["first_writer_classification"]["full_allocation_zero_writer"],
        "host memset after query and allocation, before constructor"
    );
    assert_eq!(
        cold_lifecycle["first_writer_classification"]["constructor_aggregate_writer"],
        "0x1a7d50"
    );
    assert_eq!(
        cold_lifecycle["first_writer_classification"]["exact_first_nonzero_store"],
        "NATIVE_STATIC_OBSERVED_AT_0x1a7e0e"
    );
    assert_eq!(
        cold_lifecycle["first_writer_classification"]["exact_first_nonzero_store_dynamic_hit"],
        "NOT_CAPTURED"
    );
    assert_eq!(
        cold_lifecycle["cold_material_classification"]["authoritative_runtime_cold_instance"],
        "CAPTURED_BY_SIZE_OWNER_OFFSET_AND_FULL_ALLOCATION_DIGEST"
    );
    assert_eq!(
        cold_lifecycle["cold_material_classification"]["portable_canonical_bytes"],
        "NOT_CAPTURED"
    );
    assert_eq!(
        cold_lifecycle["cold_material_classification"]["reason"],
        "the runtime allocation contains process-relative pointers and nested runtime objects; its memory image is not established as a portable canonical serialization"
    );
    assert_eq!(
        cold_lifecycle["cold_material_classification"]["raw_allocation_delivered"],
        false
    );
    assert_eq!(
        cold_lifecycle["admission_boundary"]["constructor_and_first_consumer_join"],
        "CLOSED_FOR_THIS_RUN_AND_CONFIGURATION"
    );
    assert_eq!(
        cold_lifecycle["admission_boundary"]["canonical_parent_preimage"],
        "NOT_DELIVERED"
    );
    assert_eq!(
        cold_lifecycle["admission_boundary"]["canonical_parent_serializer"],
        "NOT_RUN"
    );
    assert_eq!(
        cold_lifecycle["admission_boundary"]["production_receipt"],
        "NOT_GENERATED"
    );
    assert_eq!(
        cold_lifecycle["admission_boundary"]["production_allowed"],
        false
    );
    assert_eq!(cold_lifecycle["product_wide_absence_claim"], false);
    assert_eq!(cold_lifecycle["raw_pcm_or_metadata_delivered"], false);
    assert_eq!(cold_lifecycle["private_auth"], "EXCLUDED");

    let cold_config = read_json("PARENT-COLD-CONFIG-BOUNDARY.json");
    assert_eq!(
        cold_config["schema"],
        "sanitized-parent-cold-config-boundary.v1"
    );
    assert_eq!(cold_config["run_identity"], "music_rectangular_native_108");
    assert_eq!(cold_config["module_sha256"], evidence["dll"]["sha256"]);
    assert_eq!(
        cold_config["reference_output"]["extracted_ec3_sha256"],
        "34d18a6fb0f477cbccd0b7d633a80b4191c0c535385ff66b8d9cbf7ff1440448"
    );
    assert_eq!(cold_config["reference_output"]["baseline_match"], true);
    assert_eq!(cold_config["reference_output"]["frames"], 4247);
    assert_eq!(cold_config["reference_output"]["bytes"], 17_395_712);
    let descriptor = &cold_config["native_descriptor"];
    assert_eq!(descriptor["producer_rva"], "0x178dc0");
    assert_eq!(descriptor["query_consumer_rva"], "0x1a8920");
    assert_eq!(descriptor["constructor_consumer_rva"], "0x1a7d50");
    assert_eq!(descriptor["bytes"], 120);
    assert_eq!(descriptor["mode"], 0);
    assert_eq!(descriptor["source_count"], 38);
    assert_eq!(descriptor["data_rate_code"], 5);
    assert_eq!(descriptor["pointer_offsets"], json!([16, 48, 56, 64]));
    assert_eq!(
        descriptor["pointer_zeroed_u32_words"]
            .as_array()
            .unwrap()
            .len(),
        30
    );
    assert_eq!(descriptor["pointer_zeroed_crc32"], "150fb9d7");
    let referenced_blocks = &cold_config["referenced_blocks"];
    assert_eq!(referenced_blocks["channel_records"]["bytes"], 228);
    assert_eq!(referenced_blocks["channel_records"]["crc32"], "d04aae00");
    assert_eq!(referenced_blocks["channel_records"]["present"], true);
    assert_eq!(referenced_blocks["policy"]["bytes"], 44);
    assert_eq!(referenced_blocks["policy"]["crc32"], "83456504");
    assert_eq!(referenced_blocks["policy"]["present"], true);
    assert_eq!(referenced_blocks["mix"]["bytes"], 120);
    assert_eq!(referenced_blocks["mix"]["crc32"], "2f7eb001");
    assert_eq!(referenced_blocks["mix"]["present"], true);
    assert_eq!(referenced_blocks["trim_records"]["bytes"], 216);
    assert_eq!(referenced_blocks["trim_records"]["crc32"], "dc368594");
    assert_eq!(referenced_blocks["trim_records"]["present"], true);
    let projection_assessment = &cold_config["portable_projection_assessment"];
    assert_eq!(
        projection_assessment["diagnostic_pointer_zeroed_projection"],
        "CAPTURED"
    );
    assert_eq!(
        projection_assessment["native_canonical_format_claim"],
        false
    );
    assert_eq!(
        projection_assessment["field_complete_semantic_schema"],
        "NOT_CAPTURED"
    );
    assert_eq!(
        projection_assessment["referenced_block_bytes_delivered"],
        false
    );
    assert_eq!(
        projection_assessment["referenced_block_hashes_are_reconstruction_material"],
        false
    );
    assert_eq!(
        projection_assessment["portable_reconstruction_ready"],
        false
    );
    let first_store = &cold_config["exact_first_nonzero_store"];
    assert_eq!(first_store["classification"], "NATIVE_STATIC_OBSERVED");
    assert_eq!(
        first_store["preceding_full_region_zero_callsite_rva"],
        "0x1a7df9"
    );
    assert_eq!(first_store["store_rva"], "0x1a7e0e");
    assert_eq!(first_store["destination"], "allocation_base + 0x8d8");
    assert_eq!(first_store["value"], "allocation_base + 0x8e0");
    assert_eq!(
        first_store["following_nested_zero_callsite_rva"],
        "0x1a7e1e"
    );
    assert_eq!(first_store["dynamic_instruction_hit"], "NOT_CAPTURED");
    assert_eq!(
        cold_config["bounded_api_audit"]["constructor_trace_maximum_depth"],
        2
    );
    assert_eq!(
        cold_config["bounded_api_audit"]["constructor_trace_functions_visited"],
        74
    );
    assert_eq!(
        cold_config["bounded_api_audit"]["serializer_or_restore_api_observed"],
        false
    );
    assert_eq!(
        cold_config["bounded_api_audit"]["product_wide_absence_claim"],
        false
    );
    assert_eq!(cold_config["production_allowed"], false);
    assert_eq!(cold_config["raw_referenced_blocks_delivered"], false);
    assert_eq!(cold_config["raw_pcm_or_metadata_delivered"], false);
    assert_eq!(cold_config["private_auth"], "EXCLUDED");

    let cold_semantics = read_json("PARENT-COLD-CONFIG-SEMANTICS.json");
    assert_eq!(
        cold_semantics["schema"],
        "sanitized-parent-cold-config-semantics.v1"
    );
    assert_eq!(cold_semantics["module_sha256"], evidence["dll"]["sha256"]);
    let descriptor_fields = cold_semantics["descriptor_fields"].as_array().unwrap();
    assert_eq!(descriptor_fields.len(), 13);
    assert_eq!(
        descriptor_fields
            .iter()
            .map(|field| field["offset"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [0, 4, 8, 12, 16, 24, 40, 44, 48, 56, 64, 72, 116]
    );
    assert_eq!(
        descriptor_fields[0]["semantic"],
        "source descriptor mode flag"
    );
    assert_eq!(descriptor_fields[0]["status"], "BOUND_BRANCH_SEMANTIC");
    assert_eq!(descriptor_fields[2]["semantic"], "source record count");
    assert_eq!(descriptor_fields[2]["observed"], 38);
    assert_eq!(
        descriptor_fields[4]["semantic"],
        "source_count records of six bytes"
    );
    assert_eq!(descriptor_fields[4]["status"], "POINTER_BOUND");
    assert_eq!(
        descriptor_fields[5]["status"],
        "MODE0_OBSERVED_ZERO_MODE1_PARTIAL"
    );
    assert_eq!(descriptor_fields[6]["semantic"], "data-rate code");
    assert_eq!(
        descriptor_fields[6]["mapping"],
        json!({"0":384,"1":448,"2":576,"3":640,"4":768,"5":1024})
    );
    assert_eq!(descriptor_fields[6]["observed"], 5);
    assert_eq!(
        descriptor_fields[9]["status"],
        "POINTER_BOUND_SEMANTICS_PARTIAL"
    );
    assert_eq!(
        descriptor_fields[10]["status"],
        "POINTER_BOUND_SEMANTICS_PARTIAL"
    );
    assert_eq!(descriptor_fields[11]["status"], "PARTIALLY_BOUND");

    let policy_fields = cold_semantics["policy_block_44"].as_array().unwrap();
    assert_eq!(policy_fields.len(), 15);
    assert_eq!(
        policy_fields
            .iter()
            .map(|field| field["offset"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [0, 1, 2, 3, 4, 8, 12, 16, 20, 24, 28, 32, 33, 36, 40]
    );
    assert_eq!(policy_fields[0]["semantic"], "copyright flag / property33");
    assert_eq!(policy_fields[0]["observed"], 1);
    assert_eq!(policy_fields[1]["semantic"], "original flag / property34");
    assert_eq!(policy_fields[1]["observed"], 1);
    assert_eq!(
        policy_fields[2]["semantic"],
        "mixlevel source byte transformed by adding 80"
    );
    assert_eq!(policy_fields[2]["observed"], 0);
    assert_eq!(policy_fields[2]["submitted_value"], 80);
    assert_eq!(policy_fields[2]["property_id"], 31);
    assert_eq!(policy_fields[2]["status"], "BOUND");
    assert_ne!(policy_fields[2]["semantic"], "property12 source byte");
    assert_eq!(policy_fields[3]["semantic"], "padding/reserved");
    assert_eq!(policy_fields[3]["status"], "OBSERVED_ZERO");
    assert_eq!(policy_fields[4]["semantic"], "roomtyp / property36");
    assert_eq!(policy_fields[4]["status"], "BOUND");
    assert_eq!(
        policy_fields[4]["mapping"],
        json!({"0":"not_indicated","1":"large_room","2":"small_room","other":"not_indicated"})
    );
    assert_eq!(
        policy_fields[5]["semantic"],
        "LoRo center mix code / property25"
    );
    assert_eq!(policy_fields[5]["observed"], 4);
    assert_eq!(
        policy_fields[6]["semantic"],
        "LoRo surround mix code / property26"
    );
    assert_eq!(policy_fields[7]["semantic"], "preferred downmix mode code");
    assert_eq!(policy_fields[7]["observed"], 2);
    assert_eq!(
        policy_fields[8]["semantic"],
        "LtRt center mix code / property23"
    );
    assert_eq!(
        policy_fields[9]["semantic"],
        "LtRt surround mix code / property24"
    );
    assert_eq!(policy_fields[10]["semantic"], "dsurexmod code / property27");
    assert_eq!(policy_fields[11]["semantic"], "adconvtyp / property29");
    assert_eq!(policy_fields[11]["status"], "BOUND");
    assert_eq!(
        policy_fields[11]["mapping"],
        json!({"0":"standard","1":"reserved_or_alternate","other":"standard"})
    );
    assert_eq!(policy_fields[12]["semantic"], "padding/reserved");
    assert_eq!(policy_fields[12]["bytes"], 3);
    assert_eq!(policy_fields[12]["status"], "OBSERVED_ZERO");
    assert_eq!(
        policy_fields[13]["semantic"],
        "RF DRC profile code / property15"
    );
    assert_eq!(policy_fields[13]["observed"], 0);
    assert_eq!(
        policy_fields[14]["semantic"],
        "Line DRC profile code / property13"
    );
    assert_eq!(policy_fields[14]["observed"], 0);
    assert_eq!(
        cold_semantics["portable_contract_status"]["policy_block_semantics_complete"],
        true
    );

    let measurement_fields = cold_semantics["measurement_summary_known_fields"]
        .as_array()
        .unwrap();
    assert_eq!(measurement_fields.len(), 11);
    assert_eq!(measurement_fields[0]["descriptor_offset"], 72);
    assert_eq!(measurement_fields[0]["semantic"], "dialogue loudness");
    assert_eq!(measurement_fields[0]["observed"], -120.0);
    assert_eq!(measurement_fields[1]["descriptor_offset"], 76);
    assert_eq!(measurement_fields[1]["semantic"], "ITU loudness");
    assert_eq!(measurement_fields[1]["observed"], -16.784912109375);
    assert_eq!(measurement_fields[2]["descriptor_offset"], 80);
    assert_eq!(
        measurement_fields[2]["semantic"],
        "measurement result field"
    );
    assert_eq!(
        measurement_fields[2]["status"],
        "SEMANTIC_NAME_NOT_CAPTURED"
    );
    assert_eq!(measurement_fields[3]["descriptor_offset"], 84);
    assert_eq!(
        measurement_fields[3]["status"],
        "SEMANTIC_NAME_NOT_CAPTURED"
    );
    assert_eq!(measurement_fields[4]["descriptor_offset"], 88);
    assert_eq!(
        measurement_fields[4]["status"],
        "SEMANTIC_NAME_NOT_CAPTURED"
    );
    assert_eq!(measurement_fields[5]["descriptor_offset"], 92);
    assert_eq!(measurement_fields[5]["measurement_result_offset"], 20);
    assert_eq!(
        measurement_fields[5]["field_id"],
        "measurement_result.measured_loudness"
    );
    assert_eq!(measurement_fields[5]["producer_name"], "measured_loudness");
    assert_eq!(
        measurement_fields[5]["consumer_role"],
        "selected_loudness_for_dialnorm"
    );
    assert_eq!(
        measurement_fields[5]["status"],
        "BOUND_SAME_STORAGE_ROLE_ALIAS"
    );
    assert_eq!(measurement_fields[5]["observed"], -16.784912109375);
    assert_eq!(measurement_fields[6]["descriptor_offset"], 96);
    assert_eq!(
        measurement_fields[6]["mapping"],
        json!({"0":"ungated","1":"dialogue","2":"level","other":"invalid"})
    );
    assert_eq!(measurement_fields[7]["descriptor_offset"], 100);
    assert_eq!(measurement_fields[7]["observed"], -1.0);
    assert_eq!(measurement_fields[8]["descriptor_offset"], 104);
    assert_eq!(measurement_fields[8]["observed"], 5.73073673248291);
    assert_eq!(measurement_fields[9]["descriptor_offset"], 108);
    assert_eq!(measurement_fields[9]["observed"], -4.125058174133301);
    assert_eq!(measurement_fields[10]["descriptor_offset"], 112);
    assert_eq!(measurement_fields[10]["observed"], -3.954448699951172);
    assert_eq!(
        cold_semantics["portable_contract_status"]["measurement_semantics_complete"],
        false
    );

    let record_semantics = &cold_semantics["referenced_record_semantics"];
    assert_eq!(record_semantics["channel_records"]["record_bytes"], 6);
    assert_eq!(
        record_semantics["channel_records"]["layout"],
        "U16 input_channel_key, U16 host_index, U16 source_ordinal"
    );
    assert_eq!(
        record_semantics["channel_records"]["semantic_names"],
        "FIELD_COMPLETE"
    );
    assert_eq!(record_semantics["external_metadata_control"]["bytes"], 120);
    assert_eq!(
        record_semantics["external_metadata_control"]["semantic_names"],
        "NOT_FIELD_COMPLETE"
    );
    assert_eq!(record_semantics["trim_records"]["input_record_bytes"], 28);
    assert_eq!(record_semantics["trim_records"]["count"], 9);
    assert_eq!(
        record_semantics["trim_records"]["portable_candidate_payload_bytes"],
        24
    );
    assert_eq!(
        record_semantics["trim_records"]["copy_rule"],
        "discard each input record's first four bytes and copy the following 24 bytes in input order"
    );
    assert_eq!(
        record_semantics["trim_records"]["record_prefix"],
        "byte0 key plus three unspecified padding bytes"
    );
    assert_eq!(
        record_semantics["trim_records"]["key_order"],
        json!([0, 1, 2, 3, 4, 5, 6, 7, 8])
    );
    assert_eq!(
        record_semantics["trim_records"]["upstream_dbmd_permutation"],
        json!([0, 3, 6, 1, 4, 7, 2, 5, 8])
    );
    assert_eq!(
        record_semantics["trim_records"]["payload_semantics"],
        "FIELD_COMPLETE_FLAG_PADDING3_THREE_F32_GAINS_TWO_F32_BALANCES"
    );
    assert_eq!(
        record_semantics["trim_records"]["supersedes"],
        "PARTIAL_FROM_FLAG_THREE_GAINS_TWO_BALANCES"
    );

    let portable_contract = &cold_semantics["portable_contract_status"];
    assert_eq!(portable_contract["descriptor_layout_complete"], true);
    assert_eq!(portable_contract["channel_record_semantics_complete"], true);
    assert_eq!(
        portable_contract["external_metadata_semantics_complete"],
        false
    );
    assert_eq!(portable_contract["trim_record_semantics_complete"], true);
    assert_eq!(
        portable_contract["native_portable_field_order"],
        "NOT_OBSERVED"
    );
    assert_eq!(portable_contract["native_endian_contract"], "NOT_OBSERVED");
    assert_eq!(portable_contract["native_version_contract"], "NOT_OBSERVED");
    assert_eq!(
        portable_contract["native_optional_field_contract"],
        "NOT_OBSERVED"
    );
    assert_eq!(
        portable_contract["native_checksum_contract"],
        "NOT_OBSERVED"
    );
    assert_eq!(
        portable_contract["native_serializer_restore_roundtrip"],
        "NOT_RUN"
    );
    assert_eq!(portable_contract["implementation_ready"], false);
    assert_eq!(cold_semantics["production_allowed"], false);
    assert_eq!(cold_semantics["raw_referenced_blocks_delivered"], false);
    assert_eq!(cold_semantics["private_auth"], "EXCLUDED");

    let channel_external_trim = read_json("CHANNEL-EXTERNAL-METADATA-TRIM-SEMANTICS.json");
    assert_eq!(
        channel_external_trim["schema"],
        "sanitized-channel-external-metadata-trim-semantics.v1"
    );
    let channel_record = &channel_external_trim["channel_record_6"];
    assert_eq!(channel_record["bytes"], 6);
    assert_eq!(channel_record["fields"].as_array().unwrap().len(), 3);
    assert_eq!(channel_record["fields"][0]["offset"], 0);
    assert_eq!(channel_record["fields"][0]["semantic"], "input_channel_key");
    assert_eq!(channel_record["fields"][1]["offset"], 2);
    assert_eq!(channel_record["fields"][1]["semantic"], "host_index");
    assert_eq!(
        channel_record["fields"][1]["status"],
        "BOUND_NOT_CURRENT_ADAPTER_CONSUMED"
    );
    assert_eq!(channel_record["fields"][2]["offset"], 4);
    assert_eq!(channel_record["fields"][2]["semantic"], "source_ordinal");
    assert_eq!(
        channel_record["fields"][2]["status"],
        "BOUND_NOT_CURRENT_ADAPTER_CONSUMED"
    );
    assert_eq!(
        channel_record["source_ordinal_order"]["objects"],
        "ascending resolved TrackUID byte-string order, stable for equal keys"
    );
    assert_eq!(
        channel_record["source_ordinal_order"]["beds"],
        "resolver encounter order"
    );
    assert_eq!(channel_record["field_complete"], true);

    let external_metadata = &channel_external_trim["external_metadata_120"];
    assert_eq!(
        external_metadata["copy_rule"],
        "0x178dc0 copies all 120 bytes byte-for-byte when metadata is present and source mode is not one"
    );
    assert_eq!(external_metadata["known_field"]["offset"], 117);
    assert_eq!(
        external_metadata["known_field"]["semantic"],
        "warm-mode host enum"
    );
    assert_eq!(external_metadata["known_field"]["observed"], 4);
    assert_eq!(external_metadata["full_field_schema"], "NOT_CAPTURED");
    assert_eq!(external_metadata["field_complete"], false);

    let trim_lineage = &channel_external_trim["trim_lineage"];
    assert_eq!(trim_lineage["dbmd_section9_bytes"], 248);
    assert_eq!(trim_lineage["dbmd_section9_warm_mode"]["offset"], 152);
    assert_eq!(trim_lineage["dbmd_section9_warm_mode"]["bits"], "low3");
    assert_eq!(trim_lineage["dbmd_section9_warm_mode"]["observed"], 3);
    assert_eq!(trim_lineage["dbmd_section10"]["record_count"], 9);
    assert_eq!(trim_lineage["dbmd_section10"]["record_bytes"], 15);
    assert_eq!(trim_lineage["dbmd_section10"]["record_offset"], 7);
    assert_eq!(
        trim_lineage["dbmd_section10"]["known_fields"],
        json!([
            "flag byte",
            "three F32 gains",
            "two signed-magnitude balance bytes"
        ])
    );
    assert_eq!(
        trim_lineage["dbmd_install_key_permutation"],
        json!([0, 3, 6, 1, 4, 7, 2, 5, 8])
    );
    assert_eq!(trim_lineage["host_record_28"]["count"], 9);
    assert_eq!(trim_lineage["host_record_28"]["prefix_bytes"], 4);
    assert_eq!(
        trim_lineage["host_record_28"]["prefix_layout"],
        "byte0 key plus bytes1..3 unspecified padding"
    );
    assert_eq!(
        trim_lineage["host_record_28"]["emitted_key_order"],
        json!([0, 1, 2, 3, 4, 5, 6, 7, 8])
    );
    assert_eq!(trim_lineage["host_record_28"]["payload_bytes"], 24);
    assert_eq!(
        trim_lineage["consumer"],
        "0x1acfe0 installs host ordinal i under the matching permutation, restoring original DBMD ordinal"
    );
    assert_eq!(trim_lineage["current_downstream_trim"]["mode"], 3);
    assert_eq!(
        trim_lineage["current_downstream_trim"]["flags"],
        json!([1, 0, 0, 1, 0, 1, 1, 0, 1])
    );
    assert_eq!(
        trim_lineage["current_downstream_trim"]["groups"]
            .as_array()
            .unwrap()
            .len(),
        9
    );
    assert!(
        trim_lineage["current_downstream_trim"]["groups"]
            .as_array()
            .unwrap()
            .iter()
            .all(|group| group == &json!([16, 16, 16, 0, 0]))
    );
    assert_eq!(trim_lineage["current_downstream_trim"]["per_row_flags"], 0);
    assert_eq!(trim_lineage["host_payload_field_offsets"], "FIELD_COMPLETE");
    assert_eq!(trim_lineage["field_complete"], true);
    assert_eq!(
        trim_lineage["payload_24_fields"].as_array().unwrap().len(),
        7
    );
    assert_eq!(trim_lineage["payload_24_fields"][0]["offset"], 0);
    assert_eq!(trim_lineage["payload_24_fields"][0]["type"], "u8");
    assert_eq!(
        trim_lineage["payload_24_fields"][0]["semantic"],
        "trim group flag"
    );
    assert_eq!(trim_lineage["payload_24_fields"][1]["offset"], 1);
    assert_eq!(trim_lineage["payload_24_fields"][1]["bytes"], 3);
    assert_eq!(
        trim_lineage["payload_24_fields"][1]["semantic"],
        "unspecified padding"
    );
    for (index, offset, semantic) in [
        (2, 4, "gain0"),
        (3, 8, "gain1"),
        (4, 12, "gain2"),
        (
            5,
            16,
            "balance0 converted from signed-magnitude source byte",
        ),
        (
            6,
            20,
            "balance1 converted from signed-magnitude source byte",
        ),
    ] {
        assert_eq!(trim_lineage["payload_24_fields"][index]["offset"], offset);
        assert_eq!(trim_lineage["payload_24_fields"][index]["bytes"], 4);
        assert_eq!(trim_lineage["payload_24_fields"][index]["type"], "f32");
        assert_eq!(
            trim_lineage["payload_24_fields"][index]["semantic"],
            semantic
        );
    }
    assert_eq!(
        trim_lineage["payload_consumer_rule"],
        "0x1acfe0 reads the flag, quantizes gain0/gain1/gain2 through 0x1ad700, and quantizes balance0/balance1 to the closed [-16,16] code domain"
    );
    assert_eq!(
        channel_external_trim["portable_contract_effect"]["channel_record_semantics_complete"],
        true
    );
    assert_eq!(
        channel_external_trim["portable_contract_effect"]["external_metadata_semantics_complete"],
        false
    );
    assert_eq!(
        channel_external_trim["portable_contract_effect"]["trim_payload_semantics_complete"],
        true
    );
    assert_eq!(
        channel_external_trim["portable_contract_effect"]["native_portable_order_endian_version_optional_checksum"],
        "NOT_OBSERVED"
    );
    assert_eq!(
        channel_external_trim["portable_contract_effect"]["native_serializer_restore_roundtrip"],
        "NOT_RUN"
    );
    assert_eq!(
        channel_external_trim["portable_contract_effect"]["production_allowed"],
        false
    );
    assert_eq!(channel_external_trim["product_wide_absence_claim"], false);
    assert_eq!(
        channel_external_trim["raw_metadata_or_blocks_delivered"],
        false
    );

    let trim_payload_evidence = read_json("TRIM-PAYLOAD-PRODUCER-CONSUMER-EVIDENCE.json");
    assert_eq!(
        trim_payload_evidence["schema"],
        "sanitized-trim-payload-producer-consumer-evidence.v1"
    );
    assert_eq!(
        trim_payload_evidence["producer"]["function_rva"],
        "0x1828c0"
    );
    assert_eq!(trim_payload_evidence["producer"]["record_bytes"], 28);
    assert_eq!(
        trim_payload_evidence["producer"]["writes"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        trim_payload_evidence["producer"]["writes"][0]["record_offset"],
        0
    );
    assert_eq!(
        trim_payload_evidence["producer"]["writes"][1]["record_offset"],
        1
    );
    assert_eq!(trim_payload_evidence["producer"]["writes"][1]["bytes"], 3);
    assert_eq!(
        trim_payload_evidence["producer"]["writes"][2]["record_offset"],
        4
    );
    assert_eq!(trim_payload_evidence["producer"]["writes"][2]["bytes"], 16);
    assert_eq!(
        trim_payload_evidence["producer"]["writes"][3]["record_offset"],
        20
    );
    assert_eq!(trim_payload_evidence["producer"]["writes"][3]["bytes"], 8);
    assert_eq!(
        trim_payload_evidence["producer"]["emitted_key_order"],
        json!([0, 1, 2, 3, 4, 5, 6, 7, 8])
    );
    assert_eq!(trim_payload_evidence["adapter"]["function_rva"], "0x178dc0");
    assert_eq!(trim_payload_evidence["adapter"]["input_record_bytes"], 28);
    assert_eq!(trim_payload_evidence["adapter"]["input_record_count"], 9);
    assert_eq!(
        trim_payload_evidence["consumer"]["function_rva"],
        "0x1acfe0"
    );
    assert_eq!(
        trim_payload_evidence["consumer"]["payload_fields"]
            .as_array()
            .unwrap()
            .len(),
        7
    );
    assert_eq!(
        trim_payload_evidence["consumer"]["key_permutation"],
        json!([0, 3, 6, 1, 4, 7, 2, 5, 8])
    );
    assert_eq!(
        trim_payload_evidence["status"]["producer_record_assembly"],
        "NATIVE_STATIC_OBSERVED"
    );
    assert_eq!(
        trim_payload_evidence["status"]["adapter_copy_boundary"],
        "NATIVE_STATIC_OBSERVED"
    );
    assert_eq!(
        trim_payload_evidence["status"]["consumer_field_offsets_and_types"],
        "NATIVE_STATIC_OBSERVED"
    );
    assert_eq!(
        trim_payload_evidence["status"]["payload_semantics_complete"],
        true
    );
    assert_eq!(
        trim_payload_evidence["status"]["native_portable_parent_contract"],
        "NOT_OBSERVED"
    );
    assert_eq!(
        trim_payload_evidence["status"]["serializer_restore_roundtrip"],
        "NOT_RUN"
    );
    assert_eq!(trim_payload_evidence["status"]["production_allowed"], false);
    assert_eq!(trim_payload_evidence["raw_trim_records_delivered"], false);
    assert_eq!(trim_payload_evidence["private_auth"], "EXCLUDED");

    let semantic_corrections = read_json("PARENT-COLD-SEMANTIC-CORRECTIONS.json");
    assert_eq!(
        semantic_corrections["schema"],
        "sanitized-parent-cold-semantic-corrections.v1"
    );
    assert_eq!(
        semantic_corrections["supersedes"],
        "policy and measurement labels in v12 PARENT-COLD-CONFIG-SEMANTICS.json"
    );
    let corrections = semantic_corrections["corrections"].as_array().unwrap();
    assert_eq!(corrections.len(), 7);
    assert_eq!(corrections[0]["old"], "policy+4 bsmod/property35");
    assert_eq!(
        corrections[0]["new"],
        "policy+4 property36 value; semantic name unknown"
    );
    assert_eq!(corrections[1]["old"], "policy+32 sourcefscod/control word");
    assert_eq!(
        corrections[1]["new"],
        "policy+32 byte is property29 value; semantic name unknown"
    );
    assert_eq!(corrections[2]["old"], "policy+36 Line and +40 RF");
    assert_eq!(
        corrections[2]["new"],
        "policy+36 RF/property15 and +40 Line/property13"
    );
    assert_eq!(
        corrections[3]["old"],
        "measurement gating value name partial"
    );
    assert_eq!(corrections[4]["old"], "data rate submitted as property12");
    assert_eq!(
        corrections[4]["new"],
        "current 0x1ab520 table submits property3=1024; property31=80 is mixlevel"
    );
    assert_eq!(corrections[5]["new"], "property36=roomtyp");
    assert_eq!(corrections[6]["new"], "property29=adconvtyp");
    assert_eq!(
        corrections[3]["new"],
        "gating enum 0=ungated,1=dialogue,2=level,other=invalid"
    );
    assert_eq!(
        semantic_corrections["measurement_44_layout"]
            .as_array()
            .unwrap()
            .len(),
        11
    );
    assert_eq!(
        semantic_corrections["measurement_44_layout"][2]["semantic"],
        "unknown measurement field"
    );
    assert_eq!(
        semantic_corrections["measurement_44_layout"][2]["status"],
        "NOT_CAPTURED"
    );
    assert_eq!(
        semantic_corrections["measurement_44_layout"][6]["semantic"],
        "gating enum"
    );
    assert_eq!(
        semantic_corrections["measurement_44_layout"][5]["offset"],
        20
    );
    assert_eq!(
        semantic_corrections["measurement_44_layout"][5]["semantic"],
        "one field with stage-specific aliases"
    );
    assert_eq!(
        semantic_corrections["measurement_44_layout"][5]["field_id"],
        "measurement_result.measured_loudness"
    );
    assert_eq!(
        semantic_corrections["measurement_alias_resolution"]["distinct_field_claim"],
        false
    );
    assert_eq!(
        semantic_corrections["measurement_alias_resolution"]["stable_field_id"],
        "measurement_result.measured_loudness"
    );
    assert_eq!(
        semantic_corrections["measurement_alias_resolution"]["same_field"],
        true
    );
    assert_eq!(
        semantic_corrections["measurement_alias_resolution"]["measurement_result_offset"],
        20
    );
    assert_eq!(
        semantic_corrections["measurement_alias_resolution"]["descriptor_offset"],
        92
    );
    assert_eq!(
        semantic_corrections["remaining_bounded_no_hit"]["policy_unknown_property_semantics"],
        json!([])
    );
    assert_eq!(
        semantic_corrections["remaining_bounded_no_hit"]["portable_order_endian_version_optional_checksum"],
        "NOT_OBSERVED"
    );
    assert_eq!(semantic_corrections["production_allowed"], false);
    assert_eq!(semantic_corrections["private_auth"], "EXCLUDED");

    let measurement_alias = read_json("MEASUREMENT-ALIAS-RESOLUTION.json");
    assert_eq!(
        measurement_alias["schema"],
        "sanitized-measurement-alias-resolution.v1"
    );
    assert_eq!(
        measurement_alias["field_id"],
        "measurement_result.measured_loudness"
    );
    assert_eq!(measurement_alias["same_storage"], true);
    assert_eq!(measurement_alias["producer"]["function_rva"], "0x1769a0");
    assert_eq!(measurement_alias["producer"]["result_offset"], 20);
    assert_eq!(
        measurement_alias["producer"]["log_name"],
        "measured_loudness"
    );
    assert_eq!(measurement_alias["adapter"]["function_rva"], "0x178dc0");
    assert_eq!(
        measurement_alias["adapter"]["copy_rule"],
        "measurement result +20 copied unchanged to descriptor +92"
    );
    assert_eq!(measurement_alias["consumer"]["function_rva"], "0x1ab520");
    assert_eq!(measurement_alias["consumer"]["descriptor_offset"], 92);
    assert_eq!(
        measurement_alias["consumer"]["role"],
        "selected_loudness_for_dialnorm"
    );
    assert_eq!(measurement_alias["observed_f32"], -16.784912109375);
    assert_eq!(
        measurement_alias["alias_policy"]["canonical_field_id"],
        "measurement_result.measured_loudness"
    );
    assert_eq!(
        measurement_alias["alias_policy"]["must_not_materialize_two_fields"],
        true
    );
    assert_eq!(measurement_alias["native_portable_contract_claim"], false);
    assert_eq!(measurement_alias["production_allowed"], false);

    let external_structure = read_json("EXTERNAL-METADATA-STRUCTURE-AND-REMAINING-FIELDS.json");
    assert_eq!(
        external_structure["schema"],
        "sanitized-external-metadata-structure-and-remaining-fields.v1"
    );
    let intervals = external_structure["external_metadata_120_structure"]
        .as_array()
        .unwrap();
    assert_eq!(intervals.len(), 29);
    let interval_values = |key: &str| {
        intervals
            .iter()
            .map(|interval| interval[key].as_u64().unwrap())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        interval_values("offset"),
        vec![
            0, 1, 64, 65, 68, 69, 72, 73, 76, 77, 78, 88, 96, 97, 98, 99, 100, 102, 104, 106, 107,
            108, 112, 113, 114, 115, 116, 117, 118
        ]
    );
    assert_eq!(
        interval_values("bytes"),
        vec![
            1, 63, 1, 3, 1, 3, 1, 3, 1, 1, 10, 8, 1, 1, 1, 1, 2, 2, 2, 1, 1, 4, 1, 1, 1, 1, 1, 1, 2
        ]
    );
    let expected_types = [
        "u8",
        "char[63]",
        "u8",
        "three u8",
        "converted u8",
        "three u8",
        "converted u8",
        "three u8",
        "converted u8",
        "converted u8",
        "unwritten in bounded producer",
        "f64",
        "bool",
        "converted u8",
        "u8",
        "unwritten in bounded producer",
        "converted u16",
        "converted u16",
        "converted u16",
        "converted u8",
        "unwritten in bounded producer",
        "u32 enum",
        "u8",
        "converted u8",
        "converted u8",
        "converted u8",
        "converted u8",
        "converted u8",
        "unwritten in bounded producer",
    ];
    assert_eq!(
        intervals
            .iter()
            .map(|interval| interval["type"].as_str().unwrap())
            .collect::<Vec<_>>(),
        expected_types
    );
    let expected_producers = [
        "copy source+0",
        "strncpy source+4",
        "literal zero terminator",
        "copy source+72,+76,+80",
        "0x4dfd30(source+84)",
        "copy source+88,+92,+96",
        "0x4dfae0(source+100)",
        "copy source+104,+108,+112",
        "0x4def80(source+116)",
        "0x4df640(source+120)",
        "no store observed in 0x4e0890",
        "hours*3600 + minutes*60 + fractional units/100000, or -1",
        "source+124 != 0",
        "0x4df8b0(source+140)",
        "source+144 clamped to0 when >255",
        "no store observed in 0x4e0890",
        "0x4deb10(pointer source+148)",
        "0x4deb10(pointer source+156)",
        "0x4deb10(pointer source+164)",
        "0x4e0650(source+172)",
        "no store observed in 0x4e0890",
        "0x4e1eb0(source+176), invalid default8",
        "source+180, invalid default0",
        "0x4df1b0(source+184)",
        "0x4dff70(source+188)",
        "0x4df400(source+192)",
        "0x4ded50(source+196)",
        "0x4e1c60(source+200)",
        "no store observed in 0x4e0890",
    ];
    assert_eq!(
        intervals
            .iter()
            .map(|interval| interval["producer"].as_str().unwrap())
            .collect::<Vec<_>>(),
        expected_producers
    );
    assert_eq!(intervals[27]["semantic"], "warm-mode host enum");
    assert_eq!(intervals[27]["observed"], 4);
    let data_rate_resolution = &external_structure["data_rate_property_resolution"];
    assert_eq!(data_rate_resolution["property_id"], 3);
    assert_eq!(
        data_rate_resolution["source"],
        "descriptor+40 data_rate_enum"
    );
    assert_eq!(
        data_rate_resolution["rule"],
        json!({"0":384,"1":448,"2":576,"3":640,"4":768,"5":1024})
    );
    assert_eq!(data_rate_resolution["current_descriptor_enum"], 5);
    assert_eq!(data_rate_resolution["current_property_value"], 1024);
    assert_eq!(data_rate_resolution["policy_header_byte2_property_id"], 31);
    assert_eq!(data_rate_resolution["supersedes_property12_claim"], true);
    assert_eq!(
        external_structure["policy_remaining"]["policy_header_byte2_property_id"],
        31
    );
    assert_eq!(
        external_structure["policy_remaining"]["policy_header_byte2_rule"],
        "observed0 transformed by adding80 to submitted value80"
    );
    assert_eq!(
        external_structure["measurement_remaining"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        external_structure["measurement_remaining"][0]["measurement_offset"],
        8
    );
    assert_eq!(
        external_structure["measurement_remaining"][0]["producer_source"],
        "meter internal result+12"
    );
    assert_eq!(
        external_structure["measurement_remaining"][0]["semantic_name"],
        "NOT_CAPTURED"
    );
    assert_eq!(
        external_structure["measurement_remaining"][1]["measurement_offset"],
        12
    );
    assert_eq!(
        external_structure["measurement_remaining"][2]["measurement_offset"],
        16
    );
    assert_eq!(
        external_structure["bounded_no_hit"]["external_metadata_structural_layout_complete"],
        true
    );
    assert_eq!(
        external_structure["bounded_no_hit"]["external_metadata_semantic_schema_complete"],
        false
    );
    assert_eq!(
        external_structure["bounded_no_hit"]["portable_order_endian_version_optional_checksum"],
        "NOT_OBSERVED"
    );
    assert_eq!(
        external_structure["bounded_no_hit"]["serializer_restore_roundtrip"],
        "NOT_RUN"
    );
    assert_eq!(external_structure["production_allowed"], false);
    assert_eq!(external_structure["raw_external_metadata_delivered"], false);
    assert_eq!(external_structure["private_auth"], "EXCLUDED");

    let helper_semantics = read_json("EXTERNAL-METADATA-HELPER-SEMANTICS.json");
    assert_eq!(
        helper_semantics["schema"],
        "sanitized-external-metadata-helper-semantics.v1"
    );
    let helpers = helper_semantics["helpers"].as_array().unwrap();
    assert_eq!(helpers.len(), 13);
    assert_eq!(helpers[0]["output_offset"], 68);
    assert_eq!(helpers[0]["rva"], "0x4dfd30");
    assert_eq!(helpers[0]["semantic"], "monitor renderer");
    assert_eq!(
        helpers[0]["mapping"],
        json!({"0":0,"1":1,"3":2,"5":3,"other":0})
    );
    assert_eq!(helpers[1]["output_offset"], 72);
    assert_eq!(helpers[1]["semantic"], "DBMD master type");
    assert_eq!(
        helpers[1]["mapping"],
        json!({"0":0,"1":1,"3":2,"5":6,"7":7,"other":0})
    );
    assert_eq!(helpers[2]["output_offset"], 76);
    assert_eq!(helpers[2]["mapping"], json!({"0":0,"1":1,"2":2,"other":0}));
    assert_eq!(helpers[3]["output_offset"], 77);
    assert_eq!(
        helpers[3]["mapping"],
        json!({"0":0,"1":1,"2":2,"3":3,"4":4,"5":5,"6":6,"other":0})
    );
    assert_eq!(helpers[4]["output_offset"], 97);
    assert_eq!(helpers[4]["mapping"], json!({"0":1,"1":2,"3":0,"other":0}));
    assert_eq!(helpers[5]["output_offsets"], json!([100, 102, 104]));
    assert_eq!(
        helpers[5]["rule"],
        "100*low32 + high32, values above65535 become0"
    );
    assert_eq!(helpers[6]["output_offset"], 106);
    assert_eq!(
        helpers[6]["mapping"],
        json!({"0":1,"1":2,"14":3,"15":0,"other":0})
    );
    assert_eq!(helpers[7]["output_offset"], 108);
    assert_eq!(
        helpers[7]["rule"],
        "bit remap defined by helper; invalid source handled by caller as default8"
    );
    assert_eq!(helpers[8]["output_offset"], 113);
    assert_eq!(
        helpers[8]["mapping"],
        json!({"0":0,"1":1,"2":2,"3":3,"4":4,"other":0})
    );
    assert_eq!(helpers[9]["output_offset"], 114);
    assert_eq!(helpers[9]["mapping"], json!({"0":1,"1":2,"3":0,"other":0}));
    assert_eq!(helpers[10]["output_offset"], 115);
    assert_eq!(
        helpers[10]["mapping"],
        json!({"0":0,"1":1,"2":2,"3":3,"other":0})
    );
    assert_eq!(helpers[11]["output_offset"], 116);
    assert_eq!(helpers[11]["mapping"], json!({"0":1,"1":2,"2":0,"other":0}));
    assert_eq!(helpers[12]["output_offset"], 117);
    assert_eq!(
        helpers[12]["mapping"],
        json!({"0":1,"1":2,"2":3,"3":4,"4":0,"other":0})
    );
    assert_eq!(
        helper_semantics["remaining_unknown_offsets"],
        json!([
            0, 65, 66, 67, 69, 70, 71, 73, 74, 75, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 96, 99,
            107, 118, 119
        ])
    );
    assert_eq!(helper_semantics["text_field_name_at_1_63"], "NOT_CAPTURED");
    assert_eq!(helper_semantics["semantic_schema_complete"], false);
    assert_eq!(helper_semantics["portable_contract"], "NOT_OBSERVED");
    assert_eq!(helper_semantics["serializer_restore_roundtrip"], "NOT_RUN");
    assert_eq!(helper_semantics["product_wide_absence_claim"], false);
    assert_eq!(helper_semantics["production_allowed"], false);
    let portable_no_hit = read_json("PORTABLE-CONTRACT-AND-REMAINING-SEMANTICS-NO-HIT.json");
    assert_eq!(
        portable_no_hit["schema"],
        "sanitized-portable-contract-and-remaining-semantics-no-hit.v1"
    );
    assert_eq!(
        portable_no_hit["superseded_current_property_claims"]["property29_property36_names_unknown"],
        "SUPERSEDED_BY_POLICY29_36_IDENTITY_JOIN"
    );
    let property_flows = portable_no_hit["policy_property_dataflow"]
        .as_array()
        .unwrap();
    assert_eq!(property_flows.len(), 2);
    assert_eq!(property_flows[0]["property_id"], 29);
    assert_eq!(property_flows[0]["assignment_function_rva"], "0x1ff1e0");
    assert_eq!(property_flows[0]["assignment_case"], "0x1d");
    assert_eq!(property_flows[0]["encoder_config_offset"], 192);
    assert_eq!(property_flows[0]["config_copy_function_rva"], "0x1f3170");
    assert_eq!(property_flows[0]["copy_source_offset"], 200);
    assert_eq!(property_flows[0]["copy_destination_offset"], 192);
    assert_eq!(property_flows[0]["semantic_name"], "adconvtyp");
    assert_eq!(property_flows[1]["property_id"], 36);
    assert_eq!(property_flows[1]["assignment_case"], "0x24");
    assert_eq!(property_flows[1]["encoder_config_offset"], 220);
    assert_eq!(property_flows[1]["copy_source_offset"], 228);
    assert_eq!(property_flows[1]["copy_destination_offset"], 220);
    assert_eq!(property_flows[1]["semantic_name"], "roomtyp");
    let measurement_flows = portable_no_hit["measurement_dataflow"].as_array().unwrap();
    assert_eq!(measurement_flows.len(), 3);
    assert_eq!(
        measurement_flows
            .iter()
            .map(|flow| flow["result_offset"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [8, 12, 16]
    );
    assert!(measurement_flows.iter().all(|flow| {
        flow["export_function_rva"] == "0x17a830" && flow["semantic_name"] == "NOT_CAPTURED"
    }));
    assert_eq!(
        measurement_flows
            .iter()
            .map(|flow| flow["internal_source_offset"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [12, 16, 20]
    );
    let external_coverage = &portable_no_hit["external_metadata_coverage"];
    assert_eq!(external_coverage["producer_rva"], "0x4e0890");
    assert_eq!(external_coverage["structural_bytes_covered"], 120);
    assert_eq!(
        external_coverage["helper_semantic_offsets_closed"],
        json!([
            68, 72, 76, 77, 97, 100, 102, 104, 106, 108, 113, 114, 115, 116, 117
        ])
    );
    assert_eq!(
        external_coverage["remaining_unknown_offsets"],
        json!([
            0, 65, 66, 67, 69, 70, 71, 73, 74, 75, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 96, 99,
            107, 118, 119
        ])
    );
    assert_eq!(external_coverage["text_field_offset"], 1);
    assert_eq!(external_coverage["text_field_bytes"], 63);
    assert_eq!(
        external_coverage["text_field_semantic_name"],
        "NOT_CAPTURED"
    );
    assert_eq!(external_coverage["semantic_schema_complete"], false);
    let portable_audit = &portable_no_hit["portable_contract_audit"];
    assert_eq!(portable_audit["authorized_pe_files_scanned"], 64);
    assert_eq!(portable_audit["bounded_parent_functions_visited"], 73);
    assert_eq!(portable_audit["bounded_parent_call_edges"], 211);
    assert_eq!(
        portable_audit["constructor_depth_two_functions_visited"],
        74
    );
    assert_eq!(portable_audit["native_portable_order"], "NOT_OBSERVED");
    assert_eq!(portable_audit["native_endian_contract"], "NOT_OBSERVED");
    assert_eq!(portable_audit["native_version_contract"], "NOT_OBSERVED");
    assert_eq!(
        portable_audit["native_optional_field_contract"],
        "NOT_OBSERVED"
    );
    assert_eq!(portable_audit["native_checksum_contract"], "NOT_OBSERVED");
    assert_eq!(
        portable_audit["canonical_serializer_entrypoint"],
        "NOT_FOUND_IN_BOUNDED_AUTHORIZED_EVIDENCE"
    );
    assert_eq!(
        portable_audit["restore_entrypoint"],
        "NOT_FOUND_IN_BOUNDED_AUTHORIZED_EVIDENCE"
    );
    assert_eq!(portable_audit["serializer_restore_roundtrip"], "NOT_RUN");
    assert_eq!(portable_audit["product_wide_absence_claim"], false);
    assert_eq!(portable_no_hit["production_allowed"], false);
    assert_eq!(portable_no_hit["private_auth"], "EXCLUDED");

    let expanded_no_hit = read_json("AUTHORIZED-PE-PORTABLE-API-AND-MEASUREMENT-NO-HIT.json");
    assert_eq!(
        expanded_no_hit["schema"],
        "sanitized-authorized-pe-portable-api-and-measurement-no-hit.v1"
    );
    assert_eq!(
        expanded_no_hit["authorized_pe_scan"]["pe_files_scanned"],
        64
    );
    assert_eq!(expanded_no_hit["authorized_pe_scan"]["matching_files"], 22);
    assert_eq!(
        expanded_no_hit["authorized_pe_scan"]["parent_cold_serializer_restore_string_hit"],
        0
    );
    assert_eq!(
        expanded_no_hit["authorized_pe_scan"]["parent_cold_serializer_restore_export_hit"],
        0
    );
    assert_eq!(
        expanded_no_hit["authorized_pe_scan"]["classifications"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
    assert!(
        expanded_no_hit["authorized_pe_scan"]["classifications"]
            .as_array()
            .unwrap()
            .iter()
            .all(|classification| classification["parent_cold_state_relevance"] != "INCLUDED")
    );
    assert_eq!(
        expanded_no_hit["authorized_pe_scan"]["limitation"],
        "string and export no-hit cannot prove absence of unnamed internal APIs"
    );
    assert_eq!(
        expanded_no_hit["authorized_pe_scan"]["product_wide_absence_claim"],
        false
    );
    assert_eq!(
        expanded_no_hit["measurement_new_route"]["functions_covered"],
        json!([
            "0x17ff30", "0x17ffa0", "0x1801b0", "0x17f9c0", "0x1ae790", "0x17a830", "0x1769a0"
        ])
    );
    assert_eq!(
        expanded_no_hit["measurement_new_route"]["result_sources"],
        json!([
            {"result_offset":8,"internal_source_offset":12,"semantic_name":"NOT_CAPTURED"},
            {"result_offset":12,"internal_source_offset":16,"semantic_name":"NOT_CAPTURED"},
            {"result_offset":16,"internal_source_offset":20,"semantic_name":"NOT_CAPTURED"}
        ])
    );
    assert_eq!(
        expanded_no_hit["measurement_new_route"]["named_string_reads_found_for_unknown_offsets"],
        0
    );
    assert_eq!(
        expanded_no_hit["measurement_new_route"]["status"],
        "BOUNDED_NO_HIT"
    );
    assert_eq!(
        expanded_no_hit["portable_contract_status"]["native_order"],
        "NOT_OBSERVED"
    );
    assert_eq!(
        expanded_no_hit["portable_contract_status"]["native_endian"],
        "NOT_OBSERVED"
    );
    assert_eq!(
        expanded_no_hit["portable_contract_status"]["native_version"],
        "NOT_OBSERVED"
    );
    assert_eq!(
        expanded_no_hit["portable_contract_status"]["native_optional_fields"],
        "NOT_OBSERVED"
    );
    assert_eq!(
        expanded_no_hit["portable_contract_status"]["native_checksum"],
        "NOT_OBSERVED"
    );
    assert_eq!(
        expanded_no_hit["portable_contract_status"]["roundtrip"],
        "NOT_RUN"
    );
    assert_eq!(
        expanded_no_hit["portable_contract_status"]["implementation_ready"],
        false
    );
    assert_eq!(
        expanded_no_hit["minimum_external_evidence"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(expanded_no_hit["production_allowed"], false);
    assert_eq!(expanded_no_hit["private_auth"], "EXCLUDED");

    let measurement_internal = read_json("MEASUREMENT-INTERNAL-STRUCT-EXTENDED-NO-HIT.json");
    assert_eq!(
        measurement_internal["schema"],
        "sanitized-measurement-internal-struct-extended-no-hit.v1"
    );
    assert_eq!(
        measurement_internal["caller_chain"],
        json!([
            {"caller_rva":"0x1aebf0","core_writer":"0x1b7710 -> 0x1b7d10","mapper":"0x1ae790","role":"flush/drain path"},
            {"caller_rva":"0x1aee20","core_writer":"0x1b7d10","mapper":"0x1ae790","role":"normal input path"}
        ])
    );
    assert_eq!(
        measurement_internal["field_dataflow"],
        json!([
            {"external_result_offset":8,"mapped_result_offset":8,"core_result_offset":28,"scale":128.0,"semantic_name":"NOT_CAPTURED"},
            {"external_result_offset":12,"mapped_result_offset":12,"core_result_offset":8,"scale":128.0,"semantic_name":"NOT_CAPTURED"},
            {"external_result_offset":16,"mapped_result_offset":16,"core_result_offset":12,"scale":128.0,"semantic_name":"NOT_CAPTURED"}
        ])
    );
    assert_eq!(
        measurement_internal["core_writer_observations"]["function_rva"],
        "0x1b7d10"
    );
    assert_eq!(
        measurement_internal["core_writer_observations"]["writes_examined"],
        json!([4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 48, 52])
    );
    assert_eq!(
        measurement_internal["core_writer_observations"]["unknown_target_slots"],
        json!([8, 12, 28])
    );
    assert_eq!(
        measurement_internal["core_writer_observations"]["field_name_strings_for_offsets8_12_28"],
        0
    );
    assert_eq!(
        measurement_internal["status"]["offset_and_scale_mapping"],
        "CLOSED"
    );
    assert_eq!(
        measurement_internal["status"]["semantic_names"],
        "NOT_CAPTURED"
    );
    assert_eq!(
        measurement_internal["status"]["field_dictionary"],
        "NOT_FOUND_IN_BOUNDED_CALLERS"
    );
    assert_eq!(
        measurement_internal["status"]["implementation_ready"],
        false
    );
    assert_eq!(
        measurement_internal["minimum_external_evidence"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(measurement_internal["production_allowed"], false);
    assert_eq!(measurement_internal["private_auth"], "EXCLUDED");

    let property_pair_authority = read_json("PARENT-PROPERTY-PAIRS-AUTHORITY.json");
    assert_eq!(
        property_pair_authority["schema"],
        "sanitized-parent-property-pairs-authority.v2"
    );
    assert_eq!(
        property_pair_authority["run_identity"],
        "music_rectangular_native_109"
    );
    assert_eq!(
        property_pair_authority["config_identity"]["job_xml_sha256"],
        "d0a68be5fc5e6046c64a19beb01a23a4c6a7af11ab3dbde6e96ad96b707b6fee"
    );
    assert_eq!(
        property_pair_authority["config_identity"]["adm_input_sha256"],
        "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1"
    );
    assert_eq!(
        property_pair_authority["config_identity"]["data_rate_kbps"],
        1024
    );
    assert_eq!(
        property_pair_authority["config_identity"]["line_mode_drc_profile"],
        "none"
    );
    assert_eq!(
        property_pair_authority["config_identity"]["rf_mode_drc_profile"],
        "none"
    );
    assert_eq!(
        property_pair_authority["native_boundary"]["producer_rva"],
        "0x1ab520"
    );
    assert_eq!(
        property_pair_authority["native_boundary"]["consumer_rva"],
        "0x1b2100"
    );
    assert_eq!(
        property_pair_authority["native_boundary"]["callsite_rva"],
        "0x1ab7ac"
    );
    assert_eq!(property_pair_authority["native_boundary"]["pair_count"], 16);
    assert_eq!(
        property_pair_authority["pairs"],
        json!([
            {"property_id":3,"value":1024,"semantic":"data rate kbps"},
            {"property_id":18,"value":17,"semantic":"dialnorm derived from current measured loudness"},
            {"property_id":33,"value":1,"semantic":"copyright flag"},
            {"property_id":34,"value":1,"semantic":"original flag"},
            {"property_id":31,"value":80,"semantic":"mixlevel; policy byte+2 observed0 plus80"},
            {"property_id":36,"value":0,"semantic":"roomtyp not_indicated"},
            {"property_id":25,"value":4,"semantic":"LoRo center mix code"},
            {"property_id":26,"value":4,"semantic":"LoRo surround mix code"},
            {"property_id":20,"value":2,"semantic":"preferred downmix mode LoRo"},
            {"property_id":23,"value":4,"semantic":"LtRt center mix code"},
            {"property_id":24,"value":4,"semantic":"LtRt surround mix code"},
            {"property_id":27,"value":0,"semantic":"dsurexmod"},
            {"property_id":29,"value":0,"semantic":"adconvtyp standard"},
            {"property_id":15,"value":0,"semantic":"RF DRC profile none"},
            {"property_id":13,"value":0,"semantic":"Line DRC profile none"},
            {"property_id":7,"value":1,"semantic":"phase control literal"}
        ])
    );
    assert_eq!(
        property_pair_authority["property29_semantic_name"],
        "adconvtyp"
    );
    assert_eq!(
        property_pair_authority["property36_semantic_name"],
        "roomtyp"
    );
    assert_eq!(
        property_pair_authority["scope_partition"]["current_run109_none_drc"],
        json!({"property3":1024,"property15":0,"property13":0})
    );
    assert_eq!(
        property_pair_authority["scope_partition"]["older_music_light_scope"]["property15"],
        4
    );
    assert_eq!(
        property_pair_authority["scope_partition"]["older_music_light_scope"]["property13"],
        4
    );
    assert_eq!(
        property_pair_authority["scope_partition"]["cross_scope_substitution_allowed"],
        false
    );
    assert_eq!(property_pair_authority["production_allowed"], false);
    assert_eq!(
        property_pair_authority["raw_pcm_or_metadata_delivered"],
        false
    );
    assert_eq!(property_pair_authority["private_auth"], "EXCLUDED");

    let policy_identity = read_json("POLICY29-36-IDENTITY-JOIN.json");
    assert_eq!(
        policy_identity["schema"],
        "sanitized-policy29-36-identity-join.v1"
    );
    assert_eq!(policy_identity["property29"]["name"], "adconvtyp");
    assert_eq!(
        policy_identity["property29"]["source_field"],
        "policy block byte+32"
    );
    assert_eq!(
        policy_identity["property29"]["producer"],
        "0x4e14c0 calls ad-conversion-type converter0x4e1f70"
    );
    assert_eq!(
        policy_identity["property29"]["runtime_pair"],
        json!({"property_id":29,"value":0})
    );
    assert_eq!(
        policy_identity["property29"]["assignment"],
        "0x1ff1e0 case0x1d -> encoder config+192"
    );
    assert_eq!(
        policy_identity["property29"]["mapping"],
        json!({"0":"standard","1":"alternate_or_reserved","other":"standard"})
    );
    assert_eq!(
        policy_identity["property29"]["status"],
        "NATIVE_IDENTITY_JOIN_CLOSED"
    );
    assert_eq!(policy_identity["property36"]["name"], "roomtyp");
    assert_eq!(
        policy_identity["property36"]["source_field"],
        "policy block DWORD+4"
    );
    assert_eq!(
        policy_identity["property36"]["producer"],
        "0x4e14c0 calls room-type converter0x4e2c90"
    );
    assert_eq!(
        policy_identity["property36"]["runtime_pair"],
        json!({"property_id":36,"value":0})
    );
    assert_eq!(
        policy_identity["property36"]["assignment"],
        "0x1ff1e0 case0x24 -> encoder config+220"
    );
    assert_eq!(
        policy_identity["property36"]["mapping"],
        json!({"0":"not_indicated","1":"large_room","2":"small_room","other":"not_indicated"})
    );
    assert_eq!(
        policy_identity["property36"]["status"],
        "NATIVE_IDENTITY_JOIN_CLOSED"
    );
    assert_eq!(
        policy_identity["scope"]["run_identity"],
        "music_rectangular_native_109"
    );
    assert_eq!(policy_identity["portable_parent_contract"], "NOT_OBSERVED");
    assert_eq!(policy_identity["serializer_restore_roundtrip"], "NOT_RUN");
    assert_eq!(policy_identity["production_allowed"], false);

    let extended_no_hit = read_json("PROPERTY-NAME-AND-PORTABLE-CONTRACT-EXTENDED-NO-HIT.json");
    assert_eq!(
        extended_no_hit["schema"],
        "sanitized-property-name-and-portable-contract-extended-no-hit.v1"
    );
    assert_eq!(
        extended_no_hit["confirmed_name_join"]["property31"],
        "mixlevel"
    );
    assert_eq!(extended_no_hit["confirmed_name_join"]["value"], 80);
    assert_eq!(
        extended_no_hit["name_join_status"],
        "SUPERSEDED_BY_POLICY29_36_IDENTITY_JOIN"
    );
    let unresolved_names = extended_no_hit["unresolved_name_join"]
        .as_array()
        .expect("unresolved name joins");
    assert_eq!(unresolved_names.len(), 2);
    assert_eq!(unresolved_names[0]["property_id"], 29);
    assert_eq!(unresolved_names[0]["config_offset"], 192);
    assert_eq!(
        unresolved_names[0]["candidate_names_seen"],
        json!(["roomtyp", "langcod"])
    );
    assert_eq!(unresolved_names[0]["assigned_name"], "adconvtyp");
    assert!(
        unresolved_names[0]["reason"]
            .as_str()
            .unwrap()
            .contains("SUPERSEDED")
    );
    assert_eq!(unresolved_names[1]["property_id"], 36);
    assert_eq!(unresolved_names[1]["config_offset"], 220);
    assert_eq!(
        unresolved_names[1]["candidate_names_seen"],
        json!(["roomtyp", "langcod"])
    );
    assert_eq!(unresolved_names[1]["assigned_name"], "roomtyp");
    assert!(
        unresolved_names[1]["reason"]
            .as_str()
            .unwrap()
            .contains("SUPERSEDED")
    );
    assert_eq!(
        extended_no_hit["measurement_name_search"]["unresolved_offsets"],
        json!([8, 12, 16])
    );
    assert_eq!(
        extended_no_hit["measurement_name_search"]["candidate_name_assignment"],
        "NONE"
    );
    assert_eq!(
        extended_no_hit["measurement_name_search"]["status"],
        "NOT_CAPTURED"
    );
    assert_eq!(
        extended_no_hit["coverage"]["metadata_name_table_rva_range"],
        json!(["0x9f7d00", "0x9f7d80"])
    );
    assert_eq!(
        extended_no_hit["coverage"]["authorized_pe_files_scanned"],
        64
    );
    for field in ["order", "endian", "version", "optional_fields", "checksum"] {
        assert_eq!(extended_no_hit["portable_contract"][field], "NOT_OBSERVED");
    }
    assert_eq!(extended_no_hit["portable_contract"]["roundtrip"], "NOT_RUN");
    assert_eq!(
        extended_no_hit["portable_contract"]["product_wide_absence_claim"],
        false
    );
    assert_eq!(
        extended_no_hit["minimum_next_evidence"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(extended_no_hit["production_allowed"], false);
    assert_eq!(extended_no_hit["private_auth"], "EXCLUDED");

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
        "spatial-control-cold-serializer-blocker-independent-check.v26"
    );
    assert_eq!(independent["all_pass"], true);
    let checks = independent["checks"].as_array().expect("checks array");
    assert_eq!(checks.len(), 33);
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
            "dynamic_filewriter_fwrite_join",
            "dynamic_parent_cold_lifecycle",
            "parent_cold_config_boundary",
            "parent_cold_config_semantics",
            "semantic_corrections",
            "measurement_alias_resolution",
            "channel_external_trim_semantics",
            "trim_payload_producer_consumer_evidence",
            "external_metadata_structure_and_remaining_fields",
            "external_metadata_helper_semantics",
            "portable_contract_and_remaining_semantics_no_hit",
            "parent_property_pairs_authority",
            "property_name_and_portable_contract_extended_no_hit",
            "policy29_36_identity_join",
            "authorized_pe_portable_api_and_measurement_no_hit",
            "measurement_internal_struct_extended_no_hit",
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
