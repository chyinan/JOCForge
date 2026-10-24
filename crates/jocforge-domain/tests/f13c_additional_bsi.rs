// pattern: Functional Core
use jocforge_domain::joc::additional_bsi::{
    ADDITIONAL_BSI_PAYLOAD_BYTES, ValidatedAdditionalBsiCount, admit_selected_configuration_count,
    derive_selected_bsi_count, import_selected_additional_bsi, produce_selected_additional_bsi,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const BSI_FIXTURE_DIRECTORY: &str = "tests/fixtures/f13c_bsi";
const ID14_R2_FIXTURE_DIRECTORY: &str = "tests/fixtures/id14_dependency_r2";
const ROOT_PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const BSI_HANDOFF_MANIFEST_SHA256: &str =
    "70f90cd5472299f66bba22c362e87fe2645b6f480d71dfeb763fab2eb769fe00";
const BSI_HANDOFF_CHECK_SHA256: &str =
    "ad609b6e1e0f4975d900320c50d7e405f97324db88e7ffacc5c9e8c250cf8a1e";
const ID14_R2_HANDOFF_MANIFEST_SHA256: &str =
    "9985e8f321741d596a1675499d8764765fc9ecc864dc9824e979da4559b5d684";
const ID14_R2_HANDOFF_CHECK_SHA256: &str =
    "16abeaa22363cb4726d1a1220843fe50db64a713056fabd5fb9e9dd7efb7e0f8";
const ID14_R2_INPUTS_SHA256: &str =
    "4547fc8be5b08b9868893998cd94f136a59f05d132c0386e01a0f640ac0d6abe";
const ID14_R2_EXPECTED_SHA256: &str =
    "59e5ae092c4d249928284d2462649c8884581d666692557b708cd314d21911b1";
const ID14_R2_REPORT_SHA256: &str =
    "366179ec8e1669657f213b695b1b2ff2c5713109c6b7fec68cb01bb1807ac683";
const ID14_R2_OVERLAY_SHA256: &str =
    "ee8b425bd5d9d5cbc8ee401bef83f7c233ac38cd077fbc57dc570d149f4c77a0";
const ID14_R2_WASH_CHECK_SHA256: &str =
    "10eaf28e5e89e825b1d2ab028b5c0cd3f03bf1ce694dfa5787bac83c5d185c2f";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn bsi_bytes(name: &str) -> Vec<u8> {
    std::fs::read(crate_root().join(BSI_FIXTURE_DIRECTORY).join(name)).unwrap()
}

fn id14_r2_bytes(name: &str) -> Vec<u8> {
    std::fs::read(crate_root().join(ID14_R2_FIXTURE_DIRECTORY).join(name)).unwrap()
}

fn bsi_json(name: &str) -> Value {
    serde_json::from_slice(&bsi_bytes(name)).unwrap()
}

fn id14_r2_json(name: &str) -> Value {
    serde_json::from_slice(&id14_r2_bytes(name)).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn u8_array<const N: usize>(value: &Value) -> [u8; N] {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| u8::try_from(item.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn i16_array(value: &Value) -> Vec<i16> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| i16::try_from(item.as_i64().unwrap()).unwrap())
        .collect()
}

fn assert_handoff_payload(manifest: &Value, name: &str, bytes: &[u8]) {
    let entry = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == name)
        .unwrap();
    assert_eq!(
        entry["bytes"].as_u64().unwrap(),
        u64::try_from(bytes.len()).unwrap()
    );
    assert_eq!(entry["sha256"], sha256(bytes));
}

#[test]
fn selected_count_and_payload_inputs_are_hash_bound_to_corrected_f13c_contracts() {
    let handoff_manifest_bytes = bsi_bytes("HANDOFF-MANIFEST.json");
    let handoff_check_bytes = bsi_bytes("HANDOFF-CHECK.json");
    let bsi_manifest: Value = serde_json::from_slice(&handoff_manifest_bytes).unwrap();
    let bsi_check: Value = serde_json::from_slice(&handoff_check_bytes).unwrap();

    assert_eq!(sha256(&handoff_manifest_bytes), BSI_HANDOFF_MANIFEST_SHA256);
    assert_eq!(sha256(&handoff_check_bytes), BSI_HANDOFF_CHECK_SHA256);
    assert_eq!(bsi_manifest["files"].as_array().unwrap().len(), 18);
    assert_eq!(
        bsi_check["handoff_manifest_sha256"],
        BSI_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(bsi_check["payload_hashes_match"], true);
    assert_eq!(bsi_check["whole_F13C_ready"], false);
    assert_eq!(bsi_check["implementation_ready"], false);
    assert_eq!(
        bsi_json("INDEPENDENT-CHECK.json")["selected_scalar_producer_and_import_copy_ready"],
        true
    );
    assert_eq!(
        bsi_check["base_package_manifest_sha256"],
        ROOT_PACKAGE_MANIFEST_SHA256
    );

    for name in [
        "CONTRACT.json",
        "CONTRACT-ADDENDUM.md",
        "CURRENT-OVERLAY.json",
        "SPEC-CORRECTION.md",
        "INPUTS.json",
        "EXPECTED.json",
        "INDEPENDENT-CHECK.json",
        "WASH-INDEPENDENT-CHECK.json",
        "NATIVE-JOINS.json",
        "RUNTIME-RECEIPTS.json",
        "SOURCE-IDENTITIES.json",
        "FAILURE-OBSERVATIONS.json",
        "UPSTREAM-WATCH-RECEIPT.json",
    ] {
        assert_handoff_payload(&bsi_manifest, name, &bsi_bytes(name));
    }

    let overlay = bsi_json("CURRENT-OVERLAY.json");
    assert_eq!(
        overlay["base_package_manifest_sha256"],
        ROOT_PACKAGE_MANIFEST_SHA256
    );
    assert_eq!(
        overlay["superseded_claim"],
        "Section 1 EMDF/ID14-derived complexity producer and ID14-presence gate"
    );
    assert_eq!(
        overlay["replacement_scope"],
        "selected additional-BSI scalar producer from configuration count plus literal 64-byte importer copy"
    );
    assert_eq!(overlay["f13c_parent_status"], "IMPLEMENTATION_READY=NO");
    assert_eq!(bsi_json("CONTRACT.json")["whole_F13C_ready"], false);
    let provenance = bsi_json("PROVENANCE.json");
    assert_eq!(
        provenance["bsi_supplement_handoff_manifest_sha256"],
        BSI_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(
        provenance["id14_r2_handoff_manifest_sha256"],
        ID14_R2_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(provenance["constructor_prefix_bytes_materialized"], false);
    assert_eq!(provenance["id14_parser_implemented"], false);
    assert_eq!(provenance["whole_f13c_ready"], false);

    let r2_manifest_bytes = id14_r2_bytes("HANDOFF-MANIFEST.json");
    let r2_check_bytes = id14_r2_bytes("HANDOFF-CHECK.json");
    let r2_manifest: Value = serde_json::from_slice(&r2_manifest_bytes).unwrap();
    let r2_check: Value = serde_json::from_slice(&r2_check_bytes).unwrap();
    assert_eq!(sha256(&r2_manifest_bytes), ID14_R2_HANDOFF_MANIFEST_SHA256);
    assert_eq!(sha256(&r2_check_bytes), ID14_R2_HANDOFF_CHECK_SHA256);
    assert_eq!(r2_manifest["files"].as_array().unwrap().len(), 15);
    assert_eq!(
        r2_check["handoff_manifest_sha256"],
        ID14_R2_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(r2_check["payload_hashes_match"], true);
    assert_eq!(
        r2_check["base_package_manifest_sha256"],
        ROOT_PACKAGE_MANIFEST_SHA256
    );
    for name in [
        "CURRENT-OVERLAY.json",
        "INDEPENDENT-CHECK.json",
        "NO-ID14-EXPECTED.json",
        "NO-ID14-INPUTS.json",
        "REPORT.json",
        "SPEC-CORRECTION.md",
        "WASH-INDEPENDENT-CHECK.json",
    ] {
        assert_handoff_payload(&r2_manifest, name, &id14_r2_bytes(name));
    }
    assert_eq!(
        id14_r2_json("CURRENT-OVERLAY.json")["id14_body_generation_and_serialization"],
        "REQUIRED_SEPARATE_CHAIN"
    );
    assert_eq!(
        id14_r2_json("REPORT.json")["selected_path_decision"],
        "ENCODED_ID14_SEMANTIC_PARSE_NOT_REQUIRED_BY_CONFIRMED_BSI_COUNT_PRODUCER"
    );
    assert_eq!(id14_r2_json("REPORT.json")["whole_F13C_ready"], false);
}

#[test]
fn ordinary_wrapper_count_matches_the_three_washed_cases_and_natural_join() {
    let inputs = bsi_json("INPUTS.json");
    let expected = bsi_json("EXPECTED.json");
    let input_cases = inputs["ordinary_wrapper_cases"].as_array().unwrap();
    let expected_cases = expected["ordinary_wrapper_cases"].as_array().unwrap();
    assert_eq!(input_cases.len(), 3);
    assert_eq!(expected_cases.len(), input_cases.len());

    for input in input_cases {
        let case_id = input["case_id"].as_str().unwrap();
        let rate = u8::try_from(input["rate_code"].as_u64().unwrap()).unwrap();
        let labels = i16_array(&input["channel_codes_i16"]);
        let count = derive_selected_bsi_count(rate, &labels).unwrap();
        let admission = admit_selected_configuration_count(Some(count), -1_515_870_811);
        assert_eq!(admission.return_i32(), 0, "{case_id}");
        assert!(admission.prefix_write_observed(), "{case_id}");
        let validated = admission.validated_count().unwrap();
        assert_eq!(validated.as_i32(), count, "{case_id}");

        let before = u8_array::<ADDITIONAL_BSI_PAYLOAD_BYTES>(&input["payload_before_u8"]);
        let produced = produce_selected_additional_bsi(validated, &before);
        let imported = import_selected_additional_bsi(&produced);
        let result = expected_cases
            .iter()
            .find(|entry| entry["case_id"] == case_id)
            .unwrap();
        assert_eq!(count, result["count_i32"].as_i64().unwrap() as i32);
        assert_eq!(produced.enable_u16(), 1);
        assert_eq!(produced.length_minus_one_u16(), 1);
        assert_eq!(
            produced.payload_u8(),
            &u8_array::<64>(&result["payload_after_u8"])
        );
        assert_eq!(
            imported.enable_u16(),
            result["enable_u16"].as_u64().unwrap() as u16
        );
        assert_eq!(
            imported.length_minus_one_u16(),
            result["length_minus_one_u16"].as_u64().unwrap() as u16
        );
        assert_eq!(
            imported.payload_u8(),
            &u8_array::<64>(&result["import_payload_after_u8"])
        );
        assert_eq!(result["import_return"], 0);
    }

    let native_joins = bsi_json("NATIVE-JOINS.json");
    let selected_shape = native_joins["observations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| {
            event["event"] == "wrapper_shape_source"
                && event["rate_code"].as_u64() == Some(5)
                && event["channel_codes_i16"].as_array().unwrap().len() == 38
        })
        .unwrap();
    let selected_labels = i16_array(&selected_shape["channel_codes_i16"]);
    assert_eq!(derive_selected_bsi_count(5, &selected_labels), Some(16));
}

#[test]
fn selected_configuration_count_preserves_the_observed_partial_write_partition() {
    let inputs = bsi_json("INPUTS.json");
    let expected = bsi_json("EXPECTED.json");
    let input_cases = inputs["configuration_cases"].as_array().unwrap();
    let expected_cases = expected["configuration_cases"].as_array().unwrap();
    assert_eq!(input_cases.len(), 7);
    assert_eq!(expected_cases.len(), input_cases.len());

    for input in input_cases {
        let explicit = if input["explicit_i32"].is_null() {
            None
        } else {
            Some(i32::try_from(input["explicit_i32"].as_i64().unwrap()).unwrap())
        };
        let before = i32::try_from(input["target_before_i32"].as_i64().unwrap()).unwrap();
        let actual = admit_selected_configuration_count(explicit, before);
        let expected = expected_cases
            .iter()
            .find(|entry| entry["explicit_i32"] == input["explicit_i32"])
            .unwrap();
        assert_eq!(
            actual.return_i32(),
            expected["return_i32"].as_i64().unwrap() as i32
        );
        assert_eq!(
            actual.count_slot_after_i32(),
            expected["target_after_i32"].as_i64().unwrap() as i32
        );
        assert_eq!(
            actual.prefix_write_observed(),
            expected["prefix_changed"].as_bool().unwrap()
        );
        assert_eq!(
            actual
                .validated_count()
                .map(ValidatedAdditionalBsiCount::as_i32),
            if expected["return_i32"] == 0 {
                Some(expected["target_after_i32"].as_i64().unwrap() as i32)
            } else {
                None
            }
        );
    }
}

#[test]
fn selected_rate_classes_and_lfe_labels_follow_the_bounded_count_formula() {
    let two_lfe_labels = [1024_i16, 1152, -1];
    assert_eq!(derive_selected_bsi_count(0, &two_lfe_labels), Some(12));
    for rate_code in 1..=5 {
        assert_eq!(
            derive_selected_bsi_count(rate_code, &two_lfe_labels),
            Some(16)
        );
    }
    assert_eq!(derive_selected_bsi_count(5, &[-1, -32768]), Some(15));

    assert_eq!(derive_selected_bsi_count(6, &[1024]), None);
    assert_eq!(derive_selected_bsi_count(5, &[]), None);
    assert_eq!(derive_selected_bsi_count(5, &[0; 129]), None);
}

#[test]
fn validated_endpoints_copy_two_bytes_and_import_all_64_payload_bytes() {
    let inputs = bsi_json("INPUTS.json");
    let expected = bsi_json("EXPECTED.json");
    let cases = inputs["validated_pipeline_cases"].as_array().unwrap();
    let expected_cases = expected["validated_pipeline_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 2);
    for input in cases {
        let count = i32::try_from(input["explicit_i32"].as_i64().unwrap()).unwrap();
        let admission = admit_selected_configuration_count(Some(count), -1_515_870_811);
        assert_eq!(admission.return_i32(), 0);
        assert!(admission.prefix_write_observed());
        let validated = admission.validated_count().unwrap();
        let payload_before = u8_array::<64>(&input["payload_before_u8"]);
        let produced = produce_selected_additional_bsi(validated, &payload_before);
        let imported = import_selected_additional_bsi(&produced);
        let expected = expected_cases
            .iter()
            .find(|entry| entry["explicit_i32"] == input["explicit_i32"])
            .unwrap();
        assert_eq!(expected["configuration_return_i32"], 0);
        assert_eq!(expected["producer_return_i32"], 0);
        assert_eq!(expected["import_return_i32"], 0);
        assert_eq!(
            produced.enable_u16(),
            expected["enable_u16"].as_u64().unwrap() as u16
        );
        assert_eq!(
            produced.length_minus_one_u16(),
            expected["length_minus_one_u16"].as_u64().unwrap() as u16
        );
        assert_eq!(
            produced.payload_u8(),
            &u8_array::<64>(&expected["payload_after_u8"])
        );
        assert_eq!(
            imported.enable_u16(),
            expected["import_enable_u16"].as_u64().unwrap() as u16
        );
        assert_eq!(
            imported.length_minus_one_u16(),
            expected["import_length_minus_one_u16"].as_u64().unwrap() as u16
        );
        assert_eq!(
            imported.payload_u8(),
            &u8_array::<64>(&expected["import_payload_after_u8"])
        );
    }
}

#[test]
fn id14_absence_evidence_is_kept_separate_from_the_bsi_helper() {
    let handoff_manifest_bytes = id14_r2_bytes("HANDOFF-MANIFEST.json");
    let handoff_check_bytes = id14_r2_bytes("HANDOFF-CHECK.json");
    let handoff_manifest: Value = serde_json::from_slice(&handoff_manifest_bytes).unwrap();
    let handoff_check: Value = serde_json::from_slice(&handoff_check_bytes).unwrap();
    assert_eq!(
        sha256(&handoff_manifest_bytes),
        ID14_R2_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(sha256(&handoff_check_bytes), ID14_R2_HANDOFF_CHECK_SHA256);
    assert_eq!(handoff_manifest["files"].as_array().unwrap().len(), 15);
    assert_eq!(
        handoff_check["handoff_manifest_sha256"],
        ID14_R2_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(handoff_check["payload_hashes_match"], true);
    assert_eq!(handoff_check["whole_F13C_ready"], false);
    for (name, expected_sha256) in [
        ("NO-ID14-INPUTS.json", ID14_R2_INPUTS_SHA256),
        ("NO-ID14-EXPECTED.json", ID14_R2_EXPECTED_SHA256),
        ("REPORT.json", ID14_R2_REPORT_SHA256),
        ("CURRENT-OVERLAY.json", ID14_R2_OVERLAY_SHA256),
        ("WASH-INDEPENDENT-CHECK.json", ID14_R2_WASH_CHECK_SHA256),
    ] {
        let bytes = id14_r2_bytes(name);
        assert_eq!(sha256(&bytes), expected_sha256);
        assert_handoff_payload(&handoff_manifest, name, &bytes);
    }

    let overlay = id14_r2_json("CURRENT-OVERLAY.json");
    assert_eq!(
        overlay["superseded_claim"],
        "Selected BSI count must be derived by parsing serialized EMDF/ID14"
    );
    assert_eq!(
        overlay["replacement"],
        "Selected BSI count comes from validated configuration; encoded ID14 is not an input dependency of this producer"
    );
    assert_eq!(
        overlay["id14_body_generation_and_serialization"],
        "REQUIRED_SEPARATE_CHAIN"
    );

    let inputs = id14_r2_json("NO-ID14-INPUTS.json");
    let expected = id14_r2_json("NO-ID14-EXPECTED.json");
    let report = id14_r2_json("REPORT.json");
    let wash_check = id14_r2_json("WASH-INDEPENDENT-CHECK.json");
    assert_eq!(
        report["selected_path_decision"],
        "ENCODED_ID14_SEMANTIC_PARSE_NOT_REQUIRED_BY_CONFIRMED_BSI_COUNT_PRODUCER"
    );
    assert_eq!(report["whole_F13C_ready"], false);
    assert_eq!(
        wash_check["selected_bsi_count_requires_encoded_id14"],
        false
    );
    assert_eq!(wash_check["global_parser_absence_claim"], false);
    assert_eq!(wash_check["id14_body_generation_required"], true);
    assert_eq!(wash_check["whole_F13C_ready"], false);
    let cases = inputs["cases"].as_array().unwrap();
    let expected_cases = expected["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 2);
    assert_eq!(expected_cases.len(), 2);
    for case in cases {
        assert_eq!(case["id14_body_bytes"], Value::Null);
        assert_eq!(case["id14_queue_present"], false);
        assert_eq!(case["emdf_serializer_present"], false);
        assert_eq!(case["joc_body_holder_present"], false);
        assert_eq!(case["joc_body_length"], 0);
        let count = i32::try_from(case["count_i32"].as_i64().unwrap()).unwrap();
        assert!(matches!(count, 1 | 16));
        let admission = admit_selected_configuration_count(Some(count), 0);
        let validated = admission.validated_count().unwrap();
        let record =
            produce_selected_additional_bsi(validated, &u8_array::<64>(&case["payload_before_u8"]));
        let imported = import_selected_additional_bsi(&record);
        let expected = expected_cases
            .iter()
            .find(|entry| entry["count_i32"] == case["count_i32"])
            .unwrap();
        assert_eq!(
            admission.return_i32(),
            expected["constructor_status_i32"].as_i64().unwrap() as i32
        );
        assert_eq!(
            record.enable_u16(),
            expected["enable_u16"].as_u64().unwrap() as u16
        );
        assert_eq!(
            record.length_minus_one_u16(),
            expected["length_minus_one_u16"].as_u64().unwrap() as u16
        );
        assert_eq!(
            record.payload_u8(),
            &u8_array::<64>(&expected["payload_after_u8"])
        );
        assert_eq!(
            imported.payload_u8(),
            &u8_array::<64>(&expected["payload_after_u8"])
        );
        assert_eq!(expected["id14_storage_absent_before"]["id14_queue"], true);
        assert_eq!(expected["id14_storage_absent_after"]["id14_queue"], true);
    }
}
