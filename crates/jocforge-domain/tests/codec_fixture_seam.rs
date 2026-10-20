// pattern: Functional Core + Imperative Shell; fixture-only integration test
use jocforge_domain::integrity::crc16::eac3_crc16;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

const FIXTURE_MANIFEST_SHA256: &str =
    "077070af8cfbe9a2451f768cfdbefe05ffa44c07b6551330bf5d95ee44d34c44";
const SOURCE_VECTOR_SHA256: &str =
    "698c2ec69a8250251a3b3295ce135d1178079a5a2750a9ea5381c4e688db527a";
const SOURCE_VERIFICATION_SHA256: &str =
    "4615b59e39dfd1606f205e833b95f3f99b135c502ad15f5b8ebd94c7f7261fff";
const PROVIDER_FILE: &str = "codec-selected-provider.json";
const VECTORS_FILE: &str = "CODEC-OWNER-VECTORS.json";

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/codec_fixture")
}

fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn digest_hex(bytes: &[u8]) -> String {
    hex(&digest(bytes))
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

fn decode_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16)?;
            let low = (pair[1] as char).to_digit(16)?;
            Some(((high << 4) | low) as u8)
        })
        .collect()
}

fn canonical_json_sha256(value: &Value) -> String {
    digest_hex(&serde_json::to_vec(value).expect("fixture JSON serializes canonically"))
}

fn fixture_path(name: &str) -> PathBuf {
    fixture_root().join(name)
}

fn read_fixture(name: &str) -> Value {
    let bytes = fs::read(fixture_path(name)).expect("fixture file exists");
    serde_json::from_slice(&bytes).expect("fixture JSON parses")
}

fn assert_fixture_manifest() {
    let root = fixture_root();
    let manifest_bytes =
        fs::read(root.join("CODEC-FIXTURE-MANIFEST.json")).expect("codec fixture manifest exists");
    assert_eq!(digest_hex(&manifest_bytes), FIXTURE_MANIFEST_SHA256);
    let manifest: Value = serde_json::from_slice(&manifest_bytes).expect("manifest JSON");
    let files = manifest["files"].as_array().expect("manifest files array");
    assert_eq!(files.len(), 7);
    for entry in files {
        let name = entry["file"].as_str().expect("manifest file name");
        let bytes = fs::read(root.join(name)).expect("manifested file exists");
        assert_eq!(
            bytes.len() as u64,
            entry["bytes"].as_u64().expect("manifest byte count")
        );
        assert_eq!(
            digest_hex(&bytes),
            entry["sha256"].as_str().expect("manifest file SHA-256")
        );
    }
}

fn has_exact_keys(value: &Value, expected: &[&str]) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.len() == expected.len() && expected.iter().all(|key| object.contains_key(*key))
}

fn array_has_len(value: &Value, expected: usize) -> bool {
    value
        .as_array()
        .is_some_and(|items| items.len() == expected)
}

fn is_integer(value: &Value) -> bool {
    value.as_i64().is_some() || value.as_u64().is_some()
}

fn is_integer_array(value: &Value, expected_len: usize) -> bool {
    value
        .as_array()
        .is_some_and(|items| items.len() == expected_len && items.iter().all(is_integer))
}

fn is_bool_array(value: &Value, expected_len: usize) -> bool {
    value
        .as_array()
        .is_some_and(|items| items.len() == expected_len && items.iter().all(Value::is_boolean))
}

fn is_short_flag_shape(value: &Value) -> bool {
    value.as_array().is_some_and(|channels| {
        channels.len() == 6 && channels.iter().all(|channel| is_bool_array(channel, 5))
    })
}

fn input_error(input: &Value) -> Option<&'static str> {
    const INPUT_KEYS: [&str; 12] = [
        "case_id",
        "short_flags",
        "previous_short",
        "plans",
        "db_per_bit_codes",
        "main_frame_codes",
        "converter_frame_codes",
        "converter_snr_offset",
        "exponent_format",
        "bsi",
        "emdf_payload_hex",
        "joc_complexity",
    ];
    const PLAN_KEYS: [&str; 9] = [
        "fullband_strategy",
        "fullband_base",
        "fullband_packed_groups",
        "fullband_decoded_exponents",
        "fullband_gain",
        "lfe_strategy",
        "lfe_base",
        "lfe_packed_groups",
        "lfe_decoded_exponents",
    ];
    const BSI_KEYS: [&str; 5] = [
        "dialnorm",
        "rf_compression_code",
        "mixing",
        "information",
        "line_dynamic_range_codes",
    ];

    if !has_exact_keys(input, &INPUT_KEYS) {
        return Some("CODEC_FIXTURE_INPUT_INCOMPLETE");
    }
    if !is_short_flag_shape(&input["short_flags"]) || !is_bool_array(&input["previous_short"], 5) {
        return Some("CODEC_FIXTURE_SHORT_SHAPE");
    }

    let plans = &input["plans"];
    if !has_exact_keys(plans, &PLAN_KEYS)
        || !array_has_len(&plans["fullband_strategy"], 6)
        || !is_integer(&plans["fullband_base"])
        || !array_has_len(&plans["fullband_packed_groups"], 72)
        || !array_has_len(&plans["fullband_decoded_exponents"], 217)
        || !is_integer(&plans["fullband_gain"])
        || !array_has_len(&plans["lfe_strategy"], 6)
        || !is_integer(&plans["lfe_base"])
        || !array_has_len(&plans["lfe_packed_groups"], 2)
        || !array_has_len(&plans["lfe_decoded_exponents"], 7)
        || !is_integer_array(&input["db_per_bit_codes"], 6)
        || !is_integer_array(&input["main_frame_codes"], 5)
        || !is_integer_array(&input["converter_frame_codes"], 5)
        || !is_integer(&input["converter_snr_offset"])
        || !is_integer(&input["exponent_format"])
    {
        return Some("CODEC_FIXTURE_INPUT_INCOMPLETE");
    }

    if input["main_frame_codes"] == input["converter_frame_codes"] {
        return Some("CODEC_FIXTURE_PLAN_ALIAS");
    }

    let bsi = &input["bsi"];
    if !has_exact_keys(bsi, &BSI_KEYS)
        || !is_integer(&bsi["dialnorm"])
        || !(bsi["rf_compression_code"].is_null() || is_integer(&bsi["rf_compression_code"]))
        || !(bsi["mixing"].is_null() || bsi["mixing"].is_object())
        || !(bsi["information"].is_null() || bsi["information"].is_object())
        || !array_has_len(&bsi["line_dynamic_range_codes"], 6)
    {
        return Some("CODEC_FIXTURE_INPUT_INCOMPLETE");
    }

    let emdf_payload = input["emdf_payload_hex"]
        .as_str()
        .expect("exact input keys include an EMDF payload");
    if decode_hex(emdf_payload).is_none()
        || !(input["joc_complexity"].is_null() || is_integer(&input["joc_complexity"]))
    {
        return Some("CODEC_FIXTURE_INPUT_INCOMPLETE");
    }
    None
}

#[derive(Debug)]
struct VerifiedFixtureFrame {
    case_id: String,
    bytes: Vec<u8>,
    sha256: String,
    crc_residual: u16,
}

#[derive(Debug)]
struct FixtureValidation {
    frame: VerifiedFixtureFrame,
    result: Value,
}

fn error_result(code: &str) -> Value {
    json!({
        "status": "ERROR",
        "error": code,
        "frame_hex": null,
        "receipt": null,
        "state_delta": null,
        "commit_count": 0,
    })
}

fn validate_supplied_fixture(
    case: &Value,
    source_vector_sha256: &str,
) -> Result<FixtureValidation, &'static str> {
    let case_id = case["case_id"]
        .as_str()
        .ok_or("CODEC_FIXTURE_INPUT_INCOMPLETE")?;
    let input = &case["input"];
    if input["case_id"] != case["case_id"] {
        return Err("CODEC_FIXTURE_INPUT_INCOMPLETE");
    }
    if let Some(error) = input_error(input) {
        return Err(error);
    }

    let expected = &case["expected"];
    let frame_hex = expected["frame_hex"]
        .as_str()
        .ok_or("CODEC_FIXTURE_FRAME_HASH")?;
    let frame = decode_hex(frame_hex).ok_or("CODEC_FIXTURE_FRAME_HASH")?;
    if frame.len() != 4096
        || expected["frame_bytes"].as_u64() != Some(frame.len() as u64)
        || expected["frame_sha256"].as_str() != Some(digest_hex(&frame).as_str())
    {
        return Err("CODEC_FIXTURE_FRAME_HASH");
    }
    let crc_residual = eac3_crc16(&frame[2..]);
    if crc_residual != 0 || expected["crc_residual"].as_u64() != Some(u64::from(crc_residual)) {
        return Err("CODEC_FIXTURE_CRC");
    }

    let recorded_commit_count = expected["state_commit_count"]
        .as_u64()
        .ok_or("CODEC_FIXTURE_INPUT_INCOMPLETE")?;
    assert_eq!(
        recorded_commit_count, 1,
        "model observation is not seam authority"
    );
    assert_eq!(expected["failed_trial_state_commit_count"], 0);
    assert!(expected["failed_trial_count"].as_u64().is_some());
    let visited_candidates = expected["visited_candidates"]
        .as_array()
        .expect("recorded visited-candidate sequence");
    let candidate = expected["candidate"]
        .as_u64()
        .expect("recorded selected candidate");
    assert!(!visited_candidates.is_empty());
    assert!(candidate <= 1023);
    assert!(
        visited_candidates
            .iter()
            .any(|value| value.as_u64() == Some(candidate))
    );

    let receipt = json!({
        "schema": "codec-fixture-validation-receipt.v1",
        "mode": "SUPPLIED_FIXTURE_ONLY",
        "production_allowed": false,
        "source_vector_sha256": source_vector_sha256,
        "case_id": case_id,
        "input_sha256": canonical_json_sha256(input),
        "frame_bytes": frame.len(),
        "frame_sha256": digest_hex(&frame),
        "crc_residual": crc_residual,
        "recorded_model_state_commit_count": recorded_commit_count,
        "codec_state_delta_committed": false,
        "commit_count": 0,
    });
    let result = json!({
        "status": "PASS",
        "frame_hex": hex(&frame),
        "receipt": receipt,
        "state_delta": null,
        "commit_count": 0,
    });

    Ok(FixtureValidation {
        frame: VerifiedFixtureFrame {
            case_id: case_id.to_owned(),
            bytes: frame,
            sha256: digest_hex(&decode_hex(frame_hex).expect("already decoded frame")),
            crc_residual,
        },
        result,
    })
}

fn codec_projection_bytes(state: &Value) -> Vec<u8> {
    let mut bytes = b"CodecStateV1\0".to_vec();
    let profile_id = state["profile_id"].as_str().expect("projection profile ID");
    let profile_bytes = profile_id.as_bytes();
    bytes.extend_from_slice(
        &u16::try_from(profile_bytes.len())
            .expect("profile ID fits U16")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(profile_bytes);
    bytes.extend_from_slice(&decode_hex(state["provider_sha256"].as_str().unwrap()).unwrap());
    bytes.extend_from_slice(&state["generation"].as_u64().unwrap().to_le_bytes());
    bytes.extend_from_slice(&state["next_source_index"].as_u64().unwrap().to_le_bytes());
    let history = state["transform_history_u32_words"].as_array().unwrap();
    assert_eq!(history.len(), 256);
    for word in history {
        bytes.extend_from_slice(
            &u32::try_from(word.as_u64().unwrap())
                .expect("history word fits U32")
                .to_le_bytes(),
        );
    }
    let groups = state["mantissa_group_state_u16_words"].as_array().unwrap();
    assert_eq!(groups.len(), 18);
    for word in groups {
        bytes.extend_from_slice(
            &u16::try_from(word.as_u64().unwrap())
                .expect("group word fits U16")
                .to_le_bytes(),
        );
    }
    bytes.extend_from_slice(
        &u32::try_from(state["bitwriter_cursor"].as_u64().unwrap())
            .expect("cursor fits U32")
            .to_le_bytes(),
    );
    let previous_short = state["previous_short"].as_array().unwrap();
    assert_eq!(previous_short.len(), 5);
    bytes.extend(
        previous_short
            .iter()
            .map(|value| u8::try_from(value.as_u64().unwrap()).expect("I1 byte")),
    );
    bytes.push(u8::from(state["finalized"].as_bool().unwrap()));
    bytes
}

fn frozen_failure_case(failure: &Value) -> Value {
    let case = failure["case"].clone();
    match failure["id"].as_str().expect("failure ID") {
        "frame_hash" => {
            assert_eq!(failure["mutation"]["value"], "00x32");
            assert_eq!(
                case["expected"]["frame_sha256"],
                "0000000000000000000000000000000000000000000000000000000000000000"
            );
        }
        "frame_crc" => {
            assert_eq!(
                failure["mutation"]["operation"],
                "xor byte100 with01 and update declared frame SHA"
            );
            assert_eq!(
                decode_hex(case["expected"]["frame_hex"].as_str().unwrap())
                    .unwrap()
                    .len(),
                4096
            );
        }
        "short_shape" => {
            assert_eq!(failure["mutation"]["operation"], "remove");
            assert_eq!(failure["mutation"]["path"], "/input/short_flags/5");
            assert_eq!(case["input"]["short_flags"].as_array().unwrap().len(), 5);
        }
        "plan_alias" => {
            assert_eq!(failure["mutation"]["path"], "/input/converter_frame_codes");
            assert_eq!(failure["mutation"]["value_from"], "/input/main_frame_codes");
            assert_eq!(
                case["input"]["converter_frame_codes"],
                case["input"]["main_frame_codes"]
            );
        }
        other => panic!("unexpected frozen codec fixture failure {other}"),
    }
    case
}

#[test]
fn codec_fixture_seam_recomputes_all_frozen_checks_without_codec_execution() {
    assert_fixture_manifest();
    let provider = read_fixture(PROVIDER_FILE);
    let machine_map = read_fixture("CODEC-OWNER-MACHINE-MAP.json");
    let vectors = read_fixture(VECTORS_FILE);
    let independent = read_fixture("INDEPENDENT-CHECK.json");

    let provider_sha256 = canonical_json_sha256(&provider);
    assert_eq!(provider_sha256, vectors["provider"]["canonical_sha256"]);
    assert_eq!(provider, vectors["provider"]["value"]);
    assert_eq!(provider["production_allowed"], false);
    assert_eq!(provider["frame_bytes"], 4096);
    assert_eq!(provider["expected_sources"], 4249);
    assert_eq!(provider["expected_aus"], 4247);

    let source_identity = &vectors["source_identity"];
    assert_eq!(
        source_identity["path"],
        "test-vectors/CODEC-WIRE-COMPLETE-FRAME-INPUT-VECTORS.json"
    );
    assert_eq!(source_identity["bytes"], 4_996_244);
    assert_eq!(source_identity["sha256"], SOURCE_VECTOR_SHA256);
    assert_eq!(
        source_identity["verification_path"],
        "test-vectors/CODEC-WIRE-COMPLETE-FRAME-VERIFICATION.json"
    );
    assert_eq!(source_identity["verification_bytes"], 641);
    assert_eq!(
        source_identity["verification_sha256"],
        SOURCE_VERIFICATION_SHA256
    );
    assert_eq!(vectors["input_only_source"], true);
    assert_eq!(vectors["expected_used_as_seed"], false);

    let cold = &vectors["cold_projection"];
    let cold_bytes = codec_projection_bytes(&cold["state"]);
    assert_eq!(cold_bytes.len(), 1169);
    assert_eq!(
        hex(&cold_bytes),
        cold["canonical_hex"]
            .as_str()
            .expect("cold canonical bytes")
    );
    assert_eq!(digest_hex(&cold_bytes), cold["sha256"]);
    assert_eq!(cold["state"]["provider_sha256"], provider_sha256);

    let source_sha256 = source_identity["sha256"]
        .as_str()
        .expect("source vector SHA-256");
    for case in vectors["success_cases"].as_array().expect("success cases") {
        let validated = validate_supplied_fixture(&case["case"], source_sha256)
            .expect("supplied fixture has a complete input and valid recorded frame");
        assert_eq!(validated.result, case["validation"]);
        assert_eq!(validated.frame.case_id, case["case"]["case_id"]);
        assert_eq!(validated.frame.bytes.len(), 4096);
        assert_eq!(digest_hex(&validated.frame.bytes), validated.frame.sha256);
        assert_eq!(validated.frame.crc_residual, 0);
        assert_eq!(validated.result["state_delta"], Value::Null);
        assert_eq!(validated.result["commit_count"], 0);
        assert_eq!(validated.result["receipt"]["production_allowed"], false);
        assert_eq!(
            validated.result["receipt"]["codec_state_delta_committed"],
            false
        );
    }

    for failure in vectors["failure_cases"].as_array().expect("failure cases") {
        let case = frozen_failure_case(failure);
        let error = validate_supplied_fixture(&case, source_sha256)
            .expect_err("frozen fixture mutation rejects before a proposal");
        let result = error_result(error);
        assert_eq!(result, failure["expected"]);
    }

    let payload_boundary = &vectors["payload_512_rejection"];
    assert_eq!(
        payload_boundary["case_id"],
        "payload_512_rejected_before_trial"
    );
    assert_eq!(payload_boundary["expected_error"], "CODEC_INPUT_ERROR");
    assert_eq!(payload_boundary["expected_trial_count"], 0);
    assert_eq!(payload_boundary["expected_state_commit_count"], 0);
    assert_eq!(
        payload_boundary["expected_previous_short_after"],
        json!([false, false, false, false, false])
    );

    assert_eq!(
        machine_map["production_required_receipts"],
        json!([
            "F06NDReceipt",
            "SharedAnalysisReceipt",
            "NumericPlanReceipt",
            "F05JLReceipt",
            "F13MetadataReceipt"
        ])
    );
    assert_eq!(machine_map["fixture_seam"]["production_allowed"], false);
    assert_eq!(machine_map["fixture_seam"]["commit_count"], 0);
    assert_eq!(machine_map["fixture_seam"]["state_delta"], Value::Null);
    assert_eq!(machine_map["production_root"]["status"], "BLOCKED");
    assert_eq!(
        machine_map["production_root"]["blocker"],
        "selected 256-word codec projection is not reconciled with the six transform histories required by F12S"
    );
    assert_eq!(vectors["real_4249_replay"], "NOT_RUN");
    assert_eq!(vectors["product_acceptance"], "NOT_RUN");
    assert_eq!(independent["checks"].as_array().unwrap().len(), 15);
    assert!(
        independent["checks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|check| check["pass"] == true)
    );
    assert_eq!(independent["all_pass"], true);
    assert_eq!(independent["imports_fixture_validator"], false);
    assert_eq!(independent["independent_codec_algorithm_replay"], false);
    assert_eq!(independent["expected_used_as_seed"], false);
}
