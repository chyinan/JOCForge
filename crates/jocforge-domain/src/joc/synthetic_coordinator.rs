#![cfg(test)]

use serde_json::Value;
use serde_json::json;
use sha2::{Digest, Sha256};

const PROVIDER_FILE_SHA256: &str =
    "97ef1034d01802994e4c23744c7fcea084f242c5ffc532cd12b8266433b8e748";
const PROVIDER_CANONICAL_SHA256: &str =
    "73c5e66f9b16052b5a2d715fcbb7affd9278458ea4bac636e1761d7b5c58f51c";
const SOURCE_VECTOR_PATH: &str =
    "delegated-live-joc-coordinator/CLEAN-DRAFT-LIVE-JOC-COORDINATOR-VECTORS.json";
const SOURCE_VECTOR_BYTES: u64 = 2_805_177;
const SOURCE_VECTOR_SHA256: &str =
    "de22607f2c2d7c957ee201a16a4d9fd6a47ec8c7f718432c0324a4c888153c50";
const ID14_CAPACITY_BYTES: u64 = 372;
const STAGE_ORDER: [&str; 6] = [
    "receiver_qmf",
    "temporal",
    "geometry",
    "fit180",
    "profile_retry",
    "huffman_id14",
];
const STAGE_PROVIDER_BINDINGS: [(&str, &str, &str); 6] = [
    (
        "receiver_qmf",
        "test-vectors/QMF-RECEIVER-WHOLE-BINDING-VERIFICATION.json",
        "cases/0:QRB-CONT-0",
    ),
    (
        "temporal",
        "test-vectors/JOC-TEMPORAL-WEIGHT-VERIFICATION.json",
        "cases/0:step0",
    ),
    (
        "geometry",
        "test-vectors/QMF-RECEIVER-WHOLE-BINDING-VERIFICATION.json",
        "cases/0:geometry-composition",
    ),
    (
        "fit180",
        "test-vectors/JOC-AUTHORIZED-PROVIDER-VERIFICATION.json",
        "fit_cases/0",
    ),
    (
        "profile_retry",
        "test-vectors/JOC-PROFILE-RETRY-VERIFICATION.json",
        "capacity_cases/0:INITIAL_FINE",
    ),
    (
        "huffman_id14",
        "test-vectors/JOC-HUFFMAN-VERIFICATION-RESULT.json",
        "stream_results/0:full_coarse-all-ascending",
    ),
];

#[derive(Debug)]
struct SyntheticCoordinator {
    provider: Value,
}

#[derive(Debug, Eq, PartialEq)]
struct SyntheticCoordinatorError {
    code: &'static str,
    failure_stage: &'static str,
}

impl SyntheticCoordinatorError {
    fn new(code: &'static str, failure_stage: &'static str) -> Self {
        Self {
            code,
            failure_stage,
        }
    }

    fn code(&self) -> &'static str {
        self.code
    }

    fn as_json(&self) -> Value {
        json!({
            "status": "ERROR",
            "error": self.code,
            "failure_stage": self.failure_stage,
            "body_hex": null,
            "receipt": null,
            "commit_count": 0,
            "state_delta": null,
        })
    }
}

#[derive(Debug)]
struct SyntheticValidation {
    body_hex: String,
    receipt: Value,
}

impl SyntheticValidation {
    fn body_hex(&self) -> &str {
        &self.body_hex
    }

    fn as_json(&self) -> Value {
        json!({
            "status": "PASS",
            "body_hex": self.body_hex,
            "receipt": self.receipt,
            "commit_count": 0,
            "state_delta": null,
        })
    }
}

impl SyntheticCoordinator {
    fn from_provider_bytes(bytes: &[u8]) -> Result<Self, SyntheticCoordinatorError> {
        if sha256_hex(bytes) != PROVIDER_FILE_SHA256 {
            return Err(SyntheticCoordinatorError::new(
                "SYNTHETIC_PROVIDER_IDENTITY",
                "provider",
            ));
        }
        let provider: Value = serde_json::from_slice(bytes).map_err(|_| {
            SyntheticCoordinatorError::new("SYNTHETIC_PROVIDER_IDENTITY", "provider")
        })?;
        if canonical_sha256(&provider) != PROVIDER_CANONICAL_SHA256
            || provider["schema"] != "joc-coordinator-synthetic-seam-provider.v1"
            || provider["profile_id"] != "selected-f05jl-input-only.v1"
            || provider["production_allowed"] != false
            || provider["persistent_state_commit_allowed"] != false
            || provider["stage_order"] != json!(STAGE_ORDER)
            || provider["canonical_json"]
                != json!({
                    "sort_keys": true,
                    "separators": [",", ":"],
                    "ensure_ascii": false,
                    "encoding": "UTF-8",
                })
        {
            return Err(SyntheticCoordinatorError::new(
                "SYNTHETIC_PROVIDER_IDENTITY",
                "provider",
            ));
        }
        Ok(Self { provider })
    }

    fn validate(
        &self,
        source_identity: &Value,
        input: &Value,
    ) -> Result<SyntheticValidation, SyntheticCoordinatorError> {
        if source_identity
            != &json!({
                "path": SOURCE_VECTOR_PATH,
                "bytes": SOURCE_VECTOR_BYTES,
                "sha256": SOURCE_VECTOR_SHA256,
                "case": "continuous_success[0]",
            })
        {
            return Err(SyntheticCoordinatorError::new(
                "SYNTHETIC_SOURCE_IDENTITY",
                "source_identity",
            ));
        }

        let dispatch = input["dispatch"]
            .as_u64()
            .filter(|value| *value > 0)
            .ok_or_else(|| SyntheticCoordinatorError::new("SYNTHETIC_CLOCK", "clock"))?;
        let expected_clock = json!({
            "callback_one_based": dispatch + 2,
            "callback_zero_based": dispatch + 1,
            "codec_source": dispatch - 1,
            "writer": dispatch - 1,
        });
        if input["clock"] != expected_clock {
            return Err(SyntheticCoordinatorError::new("SYNTHETIC_CLOCK", "clock"));
        }

        if input["capacity_bytes"].as_u64() != Some(ID14_CAPACITY_BYTES) {
            return Err(SyntheticCoordinatorError::new(
                "SYNTHETIC_BODY",
                "huffman_id14",
            ));
        }
        let receipts = input["receipts"].as_object().ok_or_else(|| {
            SyntheticCoordinatorError::new("SYNTHETIC_STAGE_ORDER", "receipt_set")
        })?;
        let actual_names = receipts
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        let expected_names = STAGE_ORDER
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        if actual_names != expected_names {
            return Err(SyntheticCoordinatorError::new(
                "SYNTHETIC_STAGE_ORDER",
                "receipt_set",
            ));
        }

        let mut stage_hashes = Vec::with_capacity(STAGE_ORDER.len());
        let mut previous_output_sha256 = None;
        for (stage, expected_file, expected_selector) in STAGE_PROVIDER_BINDINGS {
            let receipt = &input["receipts"][stage];
            let hashes = self.validate_stage(
                stage,
                expected_file,
                expected_selector,
                dispatch,
                previous_output_sha256.as_deref(),
                receipt,
            )?;
            previous_output_sha256 = Some(hashes.output_sha256.clone());
            stage_hashes.push(json!({
                "stage": stage,
                "input_sha256": hashes.input_sha256,
                "output_sha256": hashes.output_sha256,
            }));
        }

        let huffman_receipt = &input["receipts"]["huffman_id14"];
        let body_hex = input["id14_body_hex"]
            .as_str()
            .ok_or_else(|| SyntheticCoordinatorError::new("SYNTHETIC_BODY", "huffman_id14"))?;
        let body = decode_hex(body_hex)
            .ok_or_else(|| SyntheticCoordinatorError::new("SYNTHETIC_BODY", "huffman_id14"))?;
        let huffman_shape = &huffman_receipt["output"]["shape"];
        let huffman_provider_input = &huffman_receipt["provider"]["canonical_input_object"];
        let padded_byte_count = huffman_provider_input["padded_byte_count"].as_u64();
        let valid_bit_count = huffman_provider_input["valid_bit_count"].as_u64();
        let padding_zero_bits = huffman_provider_input["padding_zero_bits"].as_u64();
        let valid_padding = padded_byte_count
            .zip(valid_bit_count)
            .zip(padding_zero_bits)
            .is_some_and(|((byte_count, valid_bits), padding_bits)| {
                byte_count * 8 == valid_bits + padding_bits && padding_bits <= 7
            });
        let padding_is_zero = padding_zero_bits.is_some_and(|padding_bits| {
            padding_bits == 0
                || body
                    .last()
                    .is_some_and(|last_byte| last_byte & ((1_u8 << padding_bits) - 1) == 0)
        });
        let expected_body_bytes = huffman_shape["id14_body_bytes"][0].as_u64();
        let expected_body_bits = huffman_shape["id14_body_bits"][0].as_u64();
        if huffman_receipt["output"]["material"].as_str() != Some(body_hex)
            || expected_body_bytes != Some(body.len() as u64)
            || expected_body_bits != Some(body.len() as u64 * 8)
            || body.len() as u64 > ID14_CAPACITY_BYTES
            || !valid_padding
            || !padding_is_zero
            || sha256_hex(&body) != huffman_receipt["output"]["sha256"]
        {
            return Err(SyntheticCoordinatorError::new(
                "SYNTHETIC_BODY",
                "huffman_id14",
            ));
        }

        let receipt = json!({
            "schema": "joc-coordinator-synthetic-validation-receipt.v1",
            "mode": "COORDINATOR_ONLY_INPUT_TEST",
            "production_allowed": false,
            "source_vector_sha256": SOURCE_VECTOR_SHA256,
            "dispatch": dispatch,
            "callback_zero_based": expected_clock["callback_zero_based"],
            "callback_one_based": expected_clock["callback_one_based"],
            "codec_source": expected_clock["codec_source"],
            "writer": expected_clock["writer"],
            "published_au": if dispatch < 3 { Value::Null } else { json!(dispatch - 3) },
            "validated_stage_order": STAGE_ORDER,
            "stage_hashes": stage_hashes,
            "id14_body_bytes": body.len(),
            "id14_body_sha256": sha256_hex(&body),
            "producer_execution": "CALLER_PROVIDED_SYNTHETIC_RECEIPTS_VALIDATED",
            "joc_state_delta_committed": false,
            "commit_count": 0,
        });
        Ok(SyntheticValidation {
            body_hex: body_hex.to_owned(),
            receipt,
        })
    }

    fn validate_stage(
        &self,
        stage: &'static str,
        expected_file: &'static str,
        expected_selector: &'static str,
        dispatch: u64,
        expected_upstream_sha256: Option<&str>,
        receipt: &Value,
    ) -> Result<StageHashes, SyntheticCoordinatorError> {
        let fail = |code| SyntheticCoordinatorError::new(code, stage);
        if receipt["status"] != "PASS" || receipt["executed"] != true {
            return Err(fail("SYNTHETIC_NOT_EXECUTED"));
        }
        if receipt["generation"].as_u64() != Some(dispatch) {
            return Err(fail("SYNTHETIC_GENERATION"));
        }
        let expected_owner_token = format!("JOC_DISPATCH_{dispatch}:{stage}");
        if receipt["owner_token"].as_str() != Some(expected_owner_token.as_str()) {
            return Err(fail("SYNTHETIC_OWNER_TOKEN"));
        }
        if receipt["provider"]["file"].as_str() != Some(expected_file)
            || receipt["provider"]["selector"].as_str() != Some(expected_selector)
        {
            return Err(fail("SYNTHETIC_PROVIDER_HASH"));
        }
        if receipt["input"]["encoding"] != "JSON_CANONICAL_V1" {
            return Err(fail("SYNTHETIC_SHAPE"));
        }
        let expected_shapes = &self.provider["stage_shapes"][stage];
        let input_shape = &expected_shapes["input"];
        let output_shape = &expected_shapes["output"];
        if receipt["input"]["shape"] != *input_shape
            || receipt["output"]["shape"] != *output_shape
            || receipt["input"]["material"]["shape"] != *input_shape
            || (stage != "huffman_id14" && receipt["output"]["material"]["shape"] != *output_shape)
        {
            return Err(fail("SYNTHETIC_SHAPE"));
        }
        if receipt["input"]["upstream_output_sha256"].as_str() != expected_upstream_sha256 {
            return Err(fail("SYNTHETIC_CHAIN"));
        }

        let provider = &receipt["provider"];
        let provider_input_sha = canonical_sha256(&provider["canonical_input_object"]);
        let provider_output_sha = canonical_sha256(&provider["canonical_output_object"]);
        if provider_input_sha != provider["input_sha256"]
            || provider_output_sha != provider["output_sha256"]
            || receipt["input"]["material"]["provider_input"] != provider["canonical_input_object"]
            || (stage != "huffman_id14"
                && receipt["output"]["material"]["provider_output"]
                    != provider["canonical_output_object"])
        {
            return Err(fail("SYNTHETIC_PROVIDER_HASH"));
        }

        let input_material = &receipt["input"]["material"];
        if input_material["generation"].as_u64() != Some(dispatch)
            || input_material["owner_token"].as_str() != Some(expected_owner_token.as_str())
            || input_material["upstream_output_sha256"]
                != receipt["input"]["upstream_output_sha256"]
        {
            return Err(fail("SYNTHETIC_CHAIN"));
        }
        let input_sha256 = canonical_sha256(input_material);
        if receipt["input"]["sha256"].as_str() != Some(input_sha256.as_str()) {
            return Err(fail("SYNTHETIC_INPUT_HASH"));
        }

        let output_material = &receipt["output"]["material"];
        if stage != "huffman_id14"
            && (output_material["generation"].as_u64() != Some(dispatch)
                || output_material["owner_token"].as_str() != Some(expected_owner_token.as_str()))
        {
            return Err(fail("SYNTHETIC_GENERATION"));
        }
        let output_sha256 = match receipt["output"]["encoding"].as_str() {
            Some("JSON_CANONICAL_V1") => canonical_sha256(output_material),
            Some("HEX_BYTES") => {
                let bytes = output_material
                    .as_str()
                    .and_then(decode_hex)
                    .ok_or_else(|| fail("SYNTHETIC_BODY"))?;
                sha256_hex(&bytes)
            }
            _ => return Err(fail("SYNTHETIC_PROVIDER_HASH")),
        };
        if receipt["output"]["sha256"].as_str() != Some(output_sha256.as_str()) {
            return Err(fail("SYNTHETIC_PROVIDER_HASH"));
        }
        if receipt["output"]["encoding"] == "JSON_CANONICAL_V1"
            && output_material["input_sha256"] != receipt["input"]["sha256"]
        {
            return Err(fail("SYNTHETIC_CHAIN"));
        }

        Ok(StageHashes {
            input_sha256,
            output_sha256,
        })
    }
}

#[derive(Debug)]
struct StageHashes {
    input_sha256: String,
    output_sha256: String,
}

fn canonical_sha256(value: &Value) -> String {
    sha256_hex(&serde_json::to_vec(value).expect("JSON values serialize canonically"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
const PROVIDER_BYTES: &[u8] =
    include_bytes!("../../tests/fixtures/joc_synthetic_seam/synthetic-seam-provider.json");
const VECTORS_JSON: &str =
    include_str!("../../tests/fixtures/joc_synthetic_seam/SYNTHETIC-SEAM-VECTORS.json");
const INDEPENDENT_CHECK_JSON: &str =
    include_str!("../../tests/fixtures/joc_synthetic_seam/INDEPENDENT-CHECK.json");

fn vectors() -> Value {
    serde_json::from_str(VECTORS_JSON).expect("synthetic seam vectors are valid JSON")
}

fn test_coordinator() -> SyntheticCoordinator {
    SyntheticCoordinator::from_provider_bytes(PROVIDER_BYTES).unwrap()
}

fn apply_failure_patch(input: &mut Value, patch: &Value) {
    let operation = patch["op"].as_str().unwrap();
    let json_pointer = patch["path"].as_str().unwrap();
    if operation == "replace-last-byte" {
        let mut bytes = decode_hex(input["id14_body_hex"].as_str().unwrap()).unwrap();
        let final_byte = bytes.last_mut().unwrap();
        *final_byte ^= u8::try_from(patch["xor"].as_u64().unwrap()).unwrap();
        input["id14_body_hex"] = Value::String(encode_hex(&bytes));
        return;
    }

    let mut parts = json_pointer
        .trim_start_matches('/')
        .split('/')
        .collect::<Vec<_>>();
    let leaf = parts.pop().unwrap();
    let mut parent = input;
    for part in parts {
        parent = &mut parent[part];
    }
    match operation {
        "remove" => {
            parent.as_object_mut().unwrap().remove(leaf).unwrap();
        }
        "replace" => parent[leaf] = patch["value"].clone(),
        _ => panic!("unsupported frozen patch operation"),
    }
}

#[test]
fn validates_the_complete_success_receipt_without_production_commit() {
    let vectors = vectors();
    let result = test_coordinator()
        .validate(&vectors["source_identity"], &vectors["success_input"])
        .unwrap();

    assert_eq!(result.as_json(), vectors["success_expected"]);
    assert_eq!(
        result.body_hex(),
        vectors["success_expected"]["body_hex"].as_str().unwrap()
    );
    assert_eq!(
        result.as_json()["receipt"]["production_allowed"],
        Value::Bool(false)
    );
    assert_eq!(result.as_json()["receipt"]["commit_count"], 0);
    assert_eq!(result.as_json()["receipt"]["state_delta"], Value::Null);
}

#[test]
fn every_synthetic_failure_returns_no_body_receipt_or_state_delta() {
    let vectors = vectors();
    let coordinator = test_coordinator();
    for failure in vectors["failure_vectors"].as_array().unwrap() {
        let mut input = vectors["success_input"].clone();
        apply_failure_patch(&mut input, &failure["patch"]);
        let error = coordinator
            .validate(&vectors["source_identity"], &input)
            .unwrap_err();
        assert_eq!(error.as_json(), failure["expected"], "{}", failure["id"]);
    }
}

#[test]
fn provider_file_and_selector_are_bound_to_the_frozen_source_case() {
    let vectors = vectors();
    let coordinator = test_coordinator();
    for (field, replacement) in [("file", "unlisted-provider.json"), ("selector", "case/999")] {
        let mut input = vectors["success_input"].clone();
        input["receipts"]["temporal"]["provider"][field] = Value::String(replacement.to_owned());
        let error = coordinator
            .validate(&vectors["source_identity"], &input)
            .unwrap_err();
        assert_eq!(error.code(), "SYNTHETIC_PROVIDER_HASH");
        assert_eq!(error.failure_stage, "temporal");
    }
}

#[test]
fn provider_identity_is_rejected_before_json_parse() {
    let mut changed_provider = PROVIDER_BYTES.to_vec();
    changed_provider[0] ^= 1;
    let error = SyntheticCoordinator::from_provider_bytes(&changed_provider).unwrap_err();
    assert_eq!(error.code(), "SYNTHETIC_PROVIDER_IDENTITY");
}

#[test]
fn source_vector_identity_must_match_the_frozen_case() {
    let vectors = vectors();
    let mut source_identity = vectors["source_identity"].clone();
    source_identity["sha256"] = Value::String("0".repeat(64));
    let error = test_coordinator()
        .validate(&source_identity, &vectors["success_input"])
        .unwrap_err();
    assert_eq!(error.code(), "SYNTHETIC_SOURCE_IDENTITY");
    assert_eq!(error.failure_stage, "source_identity");
}

#[test]
fn washed_vectors_are_input_only_and_do_not_self_seed_expected_results() {
    let vectors = vectors();
    let independent: Value = serde_json::from_str(INDEPENDENT_CHECK_JSON).unwrap();
    assert!(vectors["input_only"].as_bool().unwrap());
    assert!(!vectors["expected_used_as_seed"].as_bool().unwrap());
    assert!(vectors.get("private_key_used").is_none());
    assert!(independent["all_pass"].as_bool().unwrap());
    assert!(!independent["expected_used_as_seed"].as_bool().unwrap());
    assert!(!independent["imports_validator"].as_bool().unwrap());
    assert_eq!(independent["checks"].as_array().unwrap().len(), 15);
}

fn decode_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digit = |byte: u8| match byte {
                b'0'..=b'9' => Some(byte - b'0'),
                b'a'..=b'f' => Some(byte - b'a' + 10),
                _ => None,
            };
            Some((digit(pair[0])? << 4) | digit(pair[1])?)
        })
        .collect()
}

fn encode_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
