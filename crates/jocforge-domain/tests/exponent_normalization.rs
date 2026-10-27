// pattern: Functional Core
use jocforge_domain::codec::exponent_normalization::{
    ExponentNormalizationError, FULLBAND_EXPONENT_BINS, LFE_EXPONENT_BINS,
    normalize_fullband_exponents, normalize_lfe_exponents,
};
use jocforge_domain::codec::policy_parameters::{PolicyProfile, load_policy_parameters};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/policy_authorized";
const ROOT_PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const NORMALIZATION_SPEC_SHA256: &str =
    "a538f0af5c70c8e8f4e4b5212438a1e5abd304c3fb2efa7165617014a55fc5fc";
const POLICY_PARAMETER_SHA256: &str =
    "a99d077900058eb4a862f5ea29bc87408bf6d7a4f4cc42e3da66ae6c68a93c6a";
const INPUT_VECTORS_SHA256: &str =
    "afed7b0fdd7b80374a63cfb0b5922d6c988ea0c4a4417f09d2f28af5de462bc5";
const RESULT_SHA256: &str = "2cbca405e9af2cf6f95e8bc36546a387ed7f26ff73afd222715e9bf720136849";
const REVIEW_STATUS_SHA256: &str =
    "4ee71cd301c607b953f0dfc1a4f19894a1a97939925bbce1e1fa3fa4d67f151d";
const RECIPE_FIXTURE_DIRECTORY: &str = "tests/fixtures/f10_normalization_recipes";
const RECIPE_HANDOFF_MANIFEST_SHA256: &str =
    "84240832bceebb711605e6c0b56016fc2948e309ec17d03d3805b82ab815df95";
const RECIPE_HANDOFF_CHECK_SHA256: &str =
    "228cd199ee0c602ba2b2487d051d5f39ac512afd88677c64c677e1c1d25d2087";
const RECIPE_INPUTS_SHA256: &str =
    "bfde8f605857982bcdf02996f382cc872a58f3faca093e8ccb1e5b8d4b197524";
const RECIPE_EXPECTED_SHA256: &str =
    "a93d4c68181f6cf5a16cdcaff74a06dd2fa27152ae2351ced4de9fb61f008179";
const RECIPE_CONTRACT_SHA256: &str =
    "a70789d492fabc2a2b6f0ea086c5141e00f9ed6f05457b6aa0b4a2ebfd5b8d4a";
const CURRENT_SPEC_REVIEW_SHA256: &str =
    "c2f3a6c318abf705b041c99f151947facc355dc6724d3966369ebb7b8f775572";
const WASH_CHECK_SHA256: &str = "ce8c21da4ad08b857a94b274a3e44fa73ebb7d7e6bcc0316ced4837853e64b15";
const CURRENT_SPEC_PARAMETER_SHA256: &str =
    "a99d077900058eb4a862f5ea29bc87408bf6d7a4f4cc42e3da66ae6c68a93c6a";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(crate_root().join(FIXTURE_DIRECTORY).join(name)).unwrap()
}

fn fixture_json(name: &str) -> Value {
    serde_json::from_slice(&fixture_bytes(name)).unwrap()
}

fn recipe_fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(crate_root().join(RECIPE_FIXTURE_DIRECTORY).join(name)).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex_bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = char::from(pair[0]).to_digit(16).unwrap();
            let low = char::from(pair[1]).to_digit(16).unwrap();
            u8::try_from(high * 16 + low).unwrap()
        })
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

fn optional_i32(value: &Value) -> Option<i32> {
    value.as_i64().map(|value| i32::try_from(value).unwrap())
}

fn optional_u32(value: &Value) -> Option<u32> {
    value.as_u64().map(|value| u32::try_from(value).unwrap())
}

fn expanded_row_sha256(row: &[i16; FULLBAND_EXPONENT_BINS]) -> String {
    let bytes: Vec<u8> = row.iter().flat_map(|value| value.to_le_bytes()).collect();
    sha256(&bytes)
}

fn recipe_raw_row(case: &Value, contract: &Value) -> [i16; FULLBAND_EXPONENT_BINS] {
    let recipe = &case["raw_recipe"];
    let values: Vec<i16> = match recipe["kind"].as_str().unwrap() {
        "mod25" => {
            let formula = contract["mod25"].as_str().unwrap();
            assert_eq!(formula, "raw[i]=(i*7+3)%25");
            let multiplier = usize::try_from(recipe["mul"].as_u64().unwrap()).unwrap();
            let addend = usize::try_from(recipe["add"].as_u64().unwrap()).unwrap();
            (0..FULLBAND_EXPONENT_BINS)
                .map(|index| i16::try_from((index * multiplier + addend) % 25).unwrap())
                .collect()
        }
        "zigzag24" => {
            let formula = contract["zigzag24"].as_str().unwrap();
            assert_eq!(
                formula,
                "q=floor(i/25); t=i%25; raw[i]=t if q%2==0 else 24-t"
            );
            (0..FULLBAND_EXPONENT_BINS)
                .map(|index| {
                    let q = index / 25;
                    let t = index % 25;
                    i16::try_from(if q % 2 == 0 { t } else { 24 - t }).unwrap()
                })
                .collect()
        }
        kind => panic!("unexpected washed normalization recipe {kind}"),
    };
    values.try_into().unwrap()
}

#[test]
fn constant_fullband_recipes_match_the_four_closed_normalization_vectors() {
    let parameter_bytes = fixture_bytes("PARAMETERS.json");
    let input_bytes = fixture_bytes("POLICY-AUTHORIZED-INPUT-VECTORS.json");
    let result_bytes = fixture_bytes("POLICY-AUTHORIZED-RESULT.json");
    let review_bytes = fixture_bytes("POLICY-ACTUAL-LOCAL-REVIEW-STATUS.json");
    let input: Value = serde_json::from_slice(&input_bytes).unwrap();
    let result: Value = serde_json::from_slice(&result_bytes).unwrap();
    let review: Value = serde_json::from_slice(&review_bytes).unwrap();

    assert_eq!(sha256(&input_bytes), INPUT_VECTORS_SHA256);
    assert_eq!(sha256(&result_bytes), RESULT_SHA256);
    assert_eq!(sha256(&review_bytes), REVIEW_STATUS_SHA256);
    assert_eq!(sha256(&parameter_bytes), POLICY_PARAMETER_SHA256);
    let provenance = fixture_json("EXPONENT-NORMALIZATION-PROVENANCE.json");
    assert_eq!(
        provenance["source_manifest_sha256"],
        ROOT_PACKAGE_MANIFEST_SHA256
    );
    assert_eq!(
        provenance["normalization_spec_sha256"],
        NORMALIZATION_SPEC_SHA256
    );
    assert_eq!(
        provenance["numeric_semantics_spec_sha256"],
        "53cb67d6c9ddced1fb909dda806bb478a2926a66f2e38cecaada6f6e95d4d135"
    );
    assert_eq!(
        provenance["parameter_artifact_sha256"],
        POLICY_PARAMETER_SHA256
    );
    assert_eq!(provenance["input_vectors_sha256"], INPUT_VECTORS_SHA256);
    assert_eq!(provenance["result_sha256"], RESULT_SHA256);
    assert_eq!(provenance["review_status_sha256"], REVIEW_STATUS_SHA256);
    assert_eq!(provenance["implemented_cases"], 6);
    assert_eq!(provenance["root_policy_vector_cases"], 4);
    assert_eq!(provenance["recipe_supplement_cases"], 2);
    assert_eq!(provenance["deferred_input_recipes"], serde_json::json!([]));
    assert_eq!(provenance["whole_f10_ready"], false);
    assert_eq!(provenance["whole_f09_ready"], false);
    assert_eq!(
        provenance["r4_review_binding_status"],
        "OLD_SPEC_HASH_FOR_CURRENT_F10_SPEC"
    );
    assert_eq!(provenance["r4_reviews_current_f10_spec"], false);
    assert_eq!(provenance["current_recipe_review_covers_all_f10"], false);
    assert_eq!(
        provenance["recipe_review"]["review_result"],
        "PASS_FOR_TWO_FBW_RECIPE_REPLAYS_ONLY"
    );
    assert_eq!(provenance["recipe_review"]["native_output_words"], 434);
    assert_eq!(provenance["recipe_review"]["native_mismatch_count"], 0);
    assert_eq!(
        provenance["recipe_review"]["natural_producer_equivalence"],
        false
    );
    let reviewed_normalization_spec = review["current_bindings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|binding| binding["file"] == "specs/10-exponent-normalization.md")
        .unwrap();
    assert_eq!(
        reviewed_normalization_spec["reviewed_sha256"],
        provenance["reviewed_normalization_spec_sha256"]
    );
    assert_eq!(
        reviewed_normalization_spec["current_sha256"],
        NORMALIZATION_SPEC_SHA256
    );
    assert_eq!(reviewed_normalization_spec["same_bytes"], false);

    let parameters = load_policy_parameters(PolicyProfile::selected(), Some(&parameter_bytes))
        .expect("normalization consumes the hash-pinned selected D06-N provider");
    let inputs = input["cases"]["normalization"].as_array().unwrap();
    let expected = result["outputs"]["normalization"].as_array().unwrap();
    let constant_cases: Vec<_> = inputs
        .iter()
        .filter(|case| case["raw_recipe"]["kind"] == "constant")
        .collect();
    assert_eq!(constant_cases.len(), 4);

    for case in constant_cases {
        let case_id = case["case_id"].as_str().unwrap();
        let level = i16::try_from(case["raw_recipe"]["level"].as_i64().unwrap()).unwrap();
        let raw = [level; FULLBAND_EXPONENT_BINS];
        let actual = normalize_fullband_exponents(
            &raw,
            u8::try_from(case["span"].as_u64().unwrap()).unwrap(),
            u8::try_from(case["option"].as_u64().unwrap()).unwrap(),
            i16::try_from(case["gate"].as_i64().unwrap()).unwrap(),
            Some(&parameters),
        )
        .unwrap();

        let expected = expected
            .iter()
            .find(|entry| entry["case_id"] == case_id)
            .unwrap();
        let computed = &expected["computed"];
        assert_eq!(expected["frozen_model_equal"], true, "{case_id}");
        assert_eq!(
            actual.base_exponent(),
            i16::try_from(computed["base_exponent"].as_i64().unwrap()).unwrap(),
            "{case_id}"
        );
        assert_eq!(
            expanded_row_sha256(actual.expanded_exponents()),
            computed["expanded_i16le_sha256"].as_str().unwrap(),
            "{case_id}"
        );
        assert_eq!(
            actual.initial_sum_i32(),
            optional_i32(&computed["initial_sum_i32"]),
            "{case_id}"
        );
        assert_eq!(
            actual.first_pass_square_error_i32(),
            optional_i32(&computed["first_pass_square_error_i32"]),
            "{case_id}"
        );
        assert_eq!(
            actual.error_shift_i32(),
            optional_i32(&computed["error_shift_i32"]),
            "{case_id}"
        );
        assert_eq!(
            actual.grouped_sample_count_i16(),
            i16::try_from(computed["grouped_sample_count_i16"].as_i64().unwrap()).unwrap(),
            "{case_id}"
        );
        assert_eq!(
            actual.scaled_count_f32_bits(),
            optional_u32(&computed["scaled_count_f32_word"]),
            "{case_id}"
        );
        assert_eq!(
            actual.lrint_threshold_i32(),
            optional_i32(&computed["lrint_threshold_i32"]),
            "{case_id}"
        );
        assert_eq!(
            actual.second_pass_triggered(),
            computed["second_pass_triggered"].as_bool().unwrap(),
            "{case_id}"
        );
    }
}

#[test]
fn normalization_only_requires_d06_n_when_option_one_and_gate_zero() {
    let raw = [16; FULLBAND_EXPONENT_BINS];

    let no_parameter_gate_zero = normalize_fullband_exponents(&raw, 1, 1, 0, None);
    assert_eq!(
        no_parameter_gate_zero,
        Err(ExponentNormalizationError::ParameterUnavailable)
    );
    assert_eq!(
        no_parameter_gate_zero.unwrap_err().code(),
        "PARAMETER_UNAVAILABLE"
    );

    let option_zero = normalize_fullband_exponents(&raw, 1, 0, 0, None).unwrap();
    assert_eq!(option_zero.initial_sum_i32(), None);
    assert!(!option_zero.second_pass_triggered());

    let gate_nonzero = normalize_fullband_exponents(&raw, 1, 1, 1, None).unwrap();
    assert_eq!(gate_nonzero.scaled_count_f32_bits(), Some(0));
    assert_eq!(gate_nonzero.lrint_threshold_i32(), Some(0));
}

#[test]
fn normalization_rejects_input_domains_outside_the_selected_contract() {
    let raw = [0; FULLBAND_EXPONENT_BINS];
    let parameters_bytes = fixture_bytes("PARAMETERS.json");
    let parameters = load_policy_parameters(PolicyProfile::selected(), Some(&parameters_bytes))
        .expect("normalization provider fixture is valid");

    assert_eq!(
        normalize_fullband_exponents(&raw, 3, 1, 0, Some(&parameters)),
        Err(ExponentNormalizationError::ExponentInputError)
    );
    assert_eq!(
        normalize_fullband_exponents(&raw, 1, 2, 0, Some(&parameters)),
        Err(ExponentNormalizationError::ExponentInputError)
    );
    let mut invalid_raw = raw;
    invalid_raw[216] = 25;
    assert_eq!(
        normalize_fullband_exponents(&invalid_raw, 1, 0, 0, None),
        Err(ExponentNormalizationError::ExponentInputError)
    );
}

#[test]
fn selected_lfe_normalizer_uses_its_seven_bin_span_one_route() {
    let parameter_bytes = fixture_bytes("PARAMETERS.json");
    let parameters = load_policy_parameters(PolicyProfile::selected(), Some(&parameter_bytes))
        .expect("LFE normalization uses the selected policy provider");
    let raw = [0_i16; LFE_EXPONENT_BINS];

    let normalized = normalize_lfe_exponents(&raw, Some(&parameters)).unwrap();
    assert_eq!(normalized.base_exponent(), 0);
    assert_eq!(normalized.expanded_exponents(), &raw);
    assert_eq!(
        normalize_lfe_exponents(&raw, None),
        Err(ExponentNormalizationError::ParameterUnavailable)
    );

    let mut invalid = raw;
    invalid[6] = 25;
    assert_eq!(
        normalize_lfe_exponents(&invalid, Some(&parameters)),
        Err(ExponentNormalizationError::ExponentInputError)
    );
}

#[test]
fn current_spec_recipe_supplement_reconstructs_and_matches_both_217_bin_cases() {
    let handoff_manifest_bytes = recipe_fixture_bytes("HANDOFF-MANIFEST.json");
    let handoff_check_bytes = recipe_fixture_bytes("HANDOFF-CHECK.json");
    let inputs_bytes = recipe_fixture_bytes("INPUTS.json");
    let contract_bytes = recipe_fixture_bytes("CONTRACT.json");

    assert_eq!(
        sha256(&handoff_manifest_bytes),
        RECIPE_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(sha256(&handoff_check_bytes), RECIPE_HANDOFF_CHECK_SHA256);
    assert_eq!(sha256(&inputs_bytes), RECIPE_INPUTS_SHA256);
    assert_eq!(sha256(&contract_bytes), RECIPE_CONTRACT_SHA256);

    let handoff_manifest: Value = serde_json::from_slice(&handoff_manifest_bytes).unwrap();
    let handoff_check: Value = serde_json::from_slice(&handoff_check_bytes).unwrap();
    let inputs: Value = serde_json::from_slice(&inputs_bytes).unwrap();
    let contract: Value = serde_json::from_slice(&contract_bytes).unwrap();

    assert_eq!(
        handoff_manifest["source_package_manifest_sha256"],
        "8e03bcbcec78c17e1bc8305e889bd1d2dae083f49de13b8f7570d5f196bd7ffd"
    );
    assert_eq!(handoff_manifest["files"].as_array().unwrap().len(), 13);
    assert_handoff_payload(&handoff_manifest, "INPUTS.json", &inputs_bytes);
    assert_handoff_payload(&handoff_manifest, "CONTRACT.json", &contract_bytes);
    assert_eq!(
        handoff_check["handoff_manifest_sha256"],
        RECIPE_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(
        handoff_check["source_package_manifest_sha256"],
        handoff_manifest["source_package_manifest_sha256"]
    );
    assert_eq!(handoff_check["payload_hashes_match"], true);
    assert_eq!(handoff_check["recipe_scope_ready"], true);
    assert_eq!(handoff_check["whole_F10_ready"], false);
    assert_eq!(
        handoff_check["current_spec_sha256"],
        NORMALIZATION_SPEC_SHA256
    );
    assert_eq!(
        handoff_check["authorized_policy_parameter_sha256"],
        POLICY_PARAMETER_SHA256
    );
    assert_eq!(inputs["input_only"], true);
    assert_eq!(inputs["cases"].as_array().unwrap().len(), 2);
    assert_eq!(contract["length"], FULLBAND_EXPONENT_BINS);
    let root_inputs = fixture_json("POLICY-AUTHORIZED-INPUT-VECTORS.json");
    let root_cases = root_inputs["cases"]["normalization"].as_array().unwrap();
    let parameter_bytes = fixture_bytes("PARAMETERS.json");
    assert_eq!(sha256(&parameter_bytes), CURRENT_SPEC_PARAMETER_SHA256);
    let parameters = load_policy_parameters(PolicyProfile::selected(), Some(&parameter_bytes))
        .expect("the selected profile provides the exact authorized D06-N bits");

    let mut actual_rows = Vec::new();
    for case in inputs["cases"].as_array().unwrap() {
        let case_id = case["case_id"].as_str().unwrap();
        let raw = recipe_raw_row(case, &contract);
        let stated_words: Vec<i16> = case["input_i16"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| i16::try_from(value.as_i64().unwrap()).unwrap())
            .collect();
        assert_eq!(
            stated_words.as_slice(),
            raw.as_slice(),
            "{case_id} input recipe"
        );
        let raw_i16le_bytes: Vec<u8> = raw.iter().flat_map(|word| word.to_le_bytes()).collect();
        assert_eq!(
            sha256(&raw_i16le_bytes),
            case["input_i16le_sha256"].as_str().unwrap()
        );
        assert_eq!(
            raw_i16le_bytes,
            hex_bytes(case["input_i16le_hex"].as_str().unwrap())
        );

        let root_case = root_cases
            .iter()
            .find(|candidate| candidate["raw_recipe"]["kind"] == case["raw_recipe"]["kind"])
            .unwrap();
        assert_eq!(root_case["span"], case["span"]);
        assert_eq!(root_case["option"], case["option"]);
        assert_eq!(root_case["gate"], case["gate"]);

        let normalized = normalize_fullband_exponents(
            &raw,
            u8::try_from(case["span"].as_u64().unwrap()).unwrap(),
            u8::try_from(case["option"].as_u64().unwrap()).unwrap(),
            i16::try_from(case["gate"].as_i64().unwrap()).unwrap(),
            Some(&parameters),
        )
        .unwrap();
        actual_rows.push((case_id.to_owned(), normalized));
    }

    let expected_bytes = recipe_fixture_bytes("EXPECTED.json");
    let current_review_bytes = recipe_fixture_bytes("CURRENT-SPEC-REVIEW.json");
    let independent_check_bytes = recipe_fixture_bytes("INDEPENDENT-CHECK.json");
    let wash_check_bytes = recipe_fixture_bytes("WASH-INDEPENDENT-CHECK.json");
    let runtime_receipt_bytes = recipe_fixture_bytes("RUNTIME-RECEIPT.json");
    let checker_replay_bytes = recipe_fixture_bytes("CHECKER-REPLAY.json");
    assert_handoff_payload(&handoff_manifest, "EXPECTED.json", &expected_bytes);
    assert_handoff_payload(
        &handoff_manifest,
        "CURRENT-SPEC-REVIEW.json",
        &current_review_bytes,
    );
    assert_handoff_payload(
        &handoff_manifest,
        "INDEPENDENT-CHECK.json",
        &independent_check_bytes,
    );
    assert_handoff_payload(
        &handoff_manifest,
        "WASH-INDEPENDENT-CHECK.json",
        &wash_check_bytes,
    );
    assert_handoff_payload(
        &handoff_manifest,
        "RUNTIME-RECEIPT.json",
        &runtime_receipt_bytes,
    );
    assert_handoff_payload(
        &handoff_manifest,
        "CHECKER-REPLAY.json",
        &checker_replay_bytes,
    );
    assert_eq!(sha256(&expected_bytes), RECIPE_EXPECTED_SHA256);
    assert_eq!(sha256(&current_review_bytes), CURRENT_SPEC_REVIEW_SHA256);
    assert_eq!(sha256(&wash_check_bytes), WASH_CHECK_SHA256);
    assert_eq!(
        sha256(&independent_check_bytes),
        "999a0e8a2770e00e5f2d10229906397f240abe21a2027cd2fe54bd9252fb7faa"
    );
    assert_eq!(
        sha256(&runtime_receipt_bytes),
        "bed589cc628e0f14b2d9f9f20a3feb66c274b251064ea79f90ade24f8e6bad6d"
    );
    assert_eq!(
        sha256(&checker_replay_bytes),
        "6700d90812e5338f3b3f8b2b9f3f2daa658eaf0f116a7b54c6c60617f7d6cd30"
    );
    let current_review: Value = serde_json::from_slice(&current_review_bytes).unwrap();
    let independent_check: Value = serde_json::from_slice(&independent_check_bytes).unwrap();
    let wash_check: Value = serde_json::from_slice(&wash_check_bytes).unwrap();
    let runtime_receipt: Value = serde_json::from_slice(&runtime_receipt_bytes).unwrap();
    let checker_replay: Value = serde_json::from_slice(&checker_replay_bytes).unwrap();

    assert_eq!(
        current_review["current_spec_sha256"],
        NORMALIZATION_SPEC_SHA256
    );
    assert_eq!(
        current_review["parameter_artifact_sha256"],
        CURRENT_SPEC_PARAMETER_SHA256
    );
    assert_eq!(
        current_review["fresh_review_result"],
        "PASS_FOR_TWO_FBW_RECIPE_REPLAYS_ONLY"
    );
    assert_eq!(current_review["current_hash_bound"], true);
    assert_eq!(
        current_review["old_review_claim_extended_to_current"],
        false
    );
    assert_eq!(
        current_review["LFE_producer_statement_independently_native_verified_this_run"],
        false
    );
    assert_eq!(independent_check["recipe_ready"], true);
    assert_eq!(independent_check["checker_replay"]["native_rerun"], false);
    assert_eq!(
        independent_check["checker_replay"]["expected_loaded_after_computation"],
        true
    );
    assert_eq!(wash_check["result"], "PASS");
    assert_eq!(wash_check["total_input_words"], 434);
    assert_eq!(wash_check["total_native_output_words"], 434);
    assert_eq!(wash_check["total_native_output_mismatches"], 0);
    assert_eq!(runtime_receipt["native_controlled_cases"], 2);
    assert_eq!(runtime_receipt["compared_output_words"], 434);
    assert_eq!(runtime_receipt["mismatch_count"], 0);
    assert_eq!(
        runtime_receipt["native_business_parameters_modified"],
        false
    );
    assert_eq!(
        checker_replay["verification_kind"],
        "FRESH_INDEPENDENT_REPLAY_OF_NATIVE_OUTPUTS"
    );
    assert_eq!(checker_replay["expected_loaded_after_computation"], true);
    assert_eq!(checker_replay["mismatch_count"], 0);

    let expected: Value = serde_json::from_slice(&expected_bytes).unwrap();
    let expected_cases = expected["cases"].as_array().unwrap();
    assert_eq!(expected_cases.len(), actual_rows.len());
    for (case_id, actual) in actual_rows {
        let expected = expected_cases
            .iter()
            .find(|case| case["case_id"] == case_id)
            .unwrap();
        let expected_words: Vec<i16> = expected["native_output_i16"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| i16::try_from(value.as_i64().unwrap()).unwrap())
            .collect();
        assert_eq!(expected_words.len(), FULLBAND_EXPONENT_BINS, "{case_id}");
        assert_eq!(
            actual.expanded_exponents().as_slice(),
            expected_words,
            "{case_id}"
        );
        assert_eq!(
            expanded_row_sha256(actual.expanded_exponents()),
            expected["output_i16le_sha256"].as_str().unwrap(),
            "{case_id}"
        );
        let wash_case = wash_check["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == case_id)
            .unwrap();
        assert_eq!(
            wash_case["native_output_sha256"],
            expected["output_i16le_sha256"]
        );
        assert_eq!(wash_case["tail_canaries_match"], true);

        let current_review_case = current_review["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"] == case_id)
            .unwrap();
        assert_eq!(
            current_review_case["output_words_native_vs_research_model_mismatch_count"],
            0
        );
        assert_eq!(
            current_review_case["native_output_matches_independent_checker_hash"],
            true
        );
        assert_eq!(
            current_review_case["input_words_match_source_generator"],
            true
        );

        if case_id == "mod25" {
            let actual_summary = wash_case;
            assert_eq!(actual.base_exponent(), 3);
            assert_eq!(actual.initial_sum_i32(), Some(1514));
            assert_eq!(actual.first_pass_square_error_i32(), Some(8006));
            assert_eq!(actual.error_shift_i32(), Some(524_681_216));
            assert_eq!(actual.scaled_count_f32_bits(), Some(0x39ac_cccd));
            assert_eq!(actual.lrint_threshold_i32(), Some(707_789));
            assert!(!actual.second_pass_triggered());
            assert_eq!(actual_summary["input_words"], 217);
            assert_eq!(actual_summary["output_words"], 217);
        } else {
            assert_eq!(actual.base_exponent(), 0);
            assert_eq!(actual.initial_sum_i32(), None);
            assert_eq!(actual.first_pass_square_error_i32(), None);
            assert_eq!(actual.error_shift_i32(), None);
            assert_eq!(actual.scaled_count_f32_bits(), None);
            assert_eq!(actual.lrint_threshold_i32(), None);
            assert!(!actual.second_pass_triggered());
            assert_eq!(wash_case["scratch_untouched"], true);
        }
    }
}
