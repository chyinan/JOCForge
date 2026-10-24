use super::*;
use serde_json::Value;
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/integrity_owner";

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURE_DIRECTORY)
        .join(name)
}

fn fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(fixture_path(name)).expect("integrity fixture is present")
}

fn vectors() -> Value {
    serde_json::from_slice(&fixture_bytes("INTEGRITY-ROOT-VECTORS.json"))
        .expect("integrity vectors are valid JSON")
}

fn provider() -> Vec<u8> {
    fixture_bytes("integrity-public-provider.json")
}

fn decode_hex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0, "hex input has whole bytes");
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).expect("hex digit");
            let low = (pair[1] as char).to_digit(16).expect("hex digit");
            u8::try_from((high << 4) | low).expect("hex byte")
        })
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex(&Sha256::digest(bytes))
}

struct TestInput {
    identity: IntegrityIdentity,
    zero_slot_container: Vec<u8>,
    primary_slot_offset: usize,
    secondary_slot_offset: usize,
    frame_template: Vec<u8>,
    container_offset: usize,
}

impl TestInput {
    fn from_vector(input: &Value) -> Self {
        let identity = &input["identity"];
        let source = identity["source_index"].as_u64().expect("source index");
        let dispatch = identity["dispatch"].as_u64().expect("dispatch index");
        let au_index = identity["au_index"].as_u64().map(PublishedAuIndex::new);
        Self {
            identity: IntegrityIdentity {
                programme_id: identity["programme_id"]
                    .as_str()
                    .expect("programme id")
                    .into(),
                attempt: identity["attempt"].as_u64().expect("attempt"),
                transaction_id: identity["transaction_id"]
                    .as_str()
                    .expect("transaction id")
                    .into(),
                source_index: CodecSourceIndex::new(source),
                dispatch: JocDispatchIndex::new(dispatch),
                au_index,
            },
            zero_slot_container: decode_hex(
                input["zero_container_hex"]
                    .as_str()
                    .expect("zero-container bytes"),
            ),
            primary_slot_offset: usize::try_from(
                input["primary_slot"]["offset"]
                    .as_u64()
                    .expect("primary slot offset"),
            )
            .expect("primary offset fits usize"),
            secondary_slot_offset: usize::try_from(
                input["secondary_slot"]["offset"]
                    .as_u64()
                    .expect("secondary slot offset"),
            )
            .expect("secondary offset fits usize"),
            frame_template: decode_hex(
                input["frame_template_hex"]
                    .as_str()
                    .expect("frame template bytes"),
            ),
            container_offset: usize::try_from(
                input["container_offset"]
                    .as_u64()
                    .expect("container offset"),
            )
            .expect("container offset fits usize"),
        }
    }

    fn as_input(&self) -> IntegrityInput<'_> {
        IntegrityInput {
            identity: self.identity.clone(),
            zero_slot_container: &self.zero_slot_container,
            primary_slot_offset: self.primary_slot_offset,
            secondary_slot_offset: self.secondary_slot_offset,
            frame_template: &self.frame_template,
            container_offset: self.container_offset,
        }
    }
}

fn assert_materialized_input(input: &Value, decoded: &TestInput) {
    assert_eq!(
        sha256_hex(&decoded.zero_slot_container),
        input["zero_container_sha256"]
    );
    assert_eq!(
        sha256_hex(&decoded.frame_template),
        input["frame_template_sha256"]
    );
    assert_eq!(input["primary_slot"]["bytes"], 4);
    assert_eq!(input["primary_slot"]["range"], serde_json::json!([8, 12]));
    assert_eq!(input["secondary_slot"]["bytes"], 1);
    assert_eq!(
        input["secondary_slot"]["range"],
        serde_json::json!([12, 13])
    );
    assert_eq!(
        input["crc32_message_range"],
        serde_json::json!([4, decoded.zero_slot_container.len()])
    );
    assert_eq!(
        input["carrier_range"],
        serde_json::json!([
            decoded.container_offset,
            decoded.container_offset + decoded.zero_slot_container.len()
        ])
    );
    assert_eq!(input["crc16_generate_range"], serde_json::json!([2, 4094]));
    assert_eq!(input["crc16_verify_range"], serde_json::json!([2, 4096]));
}

fn assert_state(state: &IntegrityState, expected: &Value) {
    assert_eq!(state.profile_id, expected["profile_id"].as_str().unwrap());
    assert_eq!(state.provider_sha256.to_hex(), expected["provider_sha256"]);
    assert_eq!(state.generation, expected["generation"].as_u64().unwrap());
    assert_eq!(
        state.next_source_index.value(),
        expected["next_source_index"].as_u64().unwrap()
    );
    assert_eq!(
        state.published_au_count,
        expected["published_au_count"].as_u64().unwrap()
    );
    assert_eq!(state.finalized, expected["finalized"].as_bool().unwrap());
}

fn assert_receipt(actual: &IntegrityStageReceipt, expected: &Value) {
    assert_eq!(actual.schema, expected["schema"]);
    assert_eq!(actual.owner_id, expected["owner_id"]);
    assert_eq!(actual.stage.as_str(), expected["stage"]);
    assert_eq!(actual.identity.programme_id, expected["programme_id"]);
    assert_eq!(
        actual.identity.attempt,
        expected["attempt"].as_u64().unwrap()
    );
    assert_eq!(actual.identity.transaction_id, expected["transaction_id"]);
    assert_eq!(
        actual.identity.source_index.value(),
        expected["source_index"].as_u64().unwrap()
    );
    assert_eq!(
        actual.identity.dispatch.value(),
        expected["dispatch"].as_u64().unwrap()
    );
    assert_eq!(
        actual.identity.au_index.map(PublishedAuIndex::value),
        expected["au_index"].as_u64()
    );
    assert_eq!(actual.before_generation, expected["before_generation"]);
    assert_eq!(actual.before_sha256.to_hex(), expected["before_sha256"]);
    assert_eq!(
        actual.proposed_after_generation,
        expected["proposed_after_generation"]
    );
    assert_eq!(
        actual.proposed_after_sha256.to_hex(),
        expected["proposed_after_sha256"]
    );
    assert_eq!(actual.provider_id, expected["provider_id"]);
    assert_eq!(actual.provider_sha256.to_hex(), expected["provider_sha256"]);
    assert_eq!(actual.input_sha256.to_hex(), expected["input_sha256"]);
    assert_eq!(actual.output_sha256.to_hex(), expected["output_sha256"]);

    match (&actual.details, expected["stage"].as_str().unwrap()) {
        (
            IntegrityStageDetails::Source {
                primary,
                secondary,
                message_sha256,
                container_bytes,
            },
            "SOURCE",
        ) => {
            assert_eq!(hex(primary), expected["primary_hex"]);
            assert_eq!(hex(secondary), expected["secondary_hex"]);
            assert_eq!(message_sha256.to_hex(), expected["message_sha256"]);
            assert_eq!(
                *container_bytes,
                usize::try_from(expected["container_bytes"].as_u64().unwrap()).unwrap()
            );
        }
        (
            IntegrityStageDetails::Dispatch {
                container_offset,
                frame_bytes,
            },
            "DISPATCH",
        ) => {
            assert_eq!(
                *container_offset,
                usize::try_from(expected["container_offset"].as_u64().unwrap()).unwrap()
            );
            assert_eq!(
                *frame_bytes,
                usize::try_from(expected["frame_bytes"].as_u64().unwrap()).unwrap()
            );
        }
        (
            IntegrityStageDetails::Au {
                crc16,
                crc_residual,
                published,
            },
            "AU",
        ) => {
            assert_eq!(u64::from(*crc16), expected["crc16"].as_u64().unwrap());
            assert_eq!(
                u64::from(*crc_residual),
                expected["crc_residual"].as_u64().unwrap()
            );
            assert_eq!(*published, expected["published"].as_bool().unwrap());
        }
        _ => panic!("receipt detail variant must match its stage"),
    }
}

fn new_owner(attempt: u64) -> IntegrityProgrammeOwner {
    IntegrityProgrammeOwner::cold("P0", attempt, 5, &provider())
        .expect("validated public keyless provider admits a cold owner")
}

#[test]
fn public_crc_primitives_match_published_checks() {
    assert_eq!(crc32_iso_hdlc(b"123456789"), 0xcbf4_3926);
    assert_eq!(crc8_smbus(b"123456789"), 0xf4);
    let v = vectors();
    let call = TestInput::from_vector(&v["continuous_calls"][0]["input"]);
    let provider = provider();
    let mut owner = new_owner(0);
    let result = owner
        .process(call.as_input(), &provider)
        .expect("first materialized keyless call succeeds");
    let expected = &v["continuous_calls"][0]["expected"]["receipts"][2];
    assert_eq!(
        result.receipts[2].details.crc16().map(u64::from),
        expected["crc16"].as_u64()
    );
    assert_eq!(result.receipts[2].details.crc_residual(), Some(0));
    assert_eq!(eac3_crc16(&result.frame[2..]), 0);
}

#[test]
fn provider_identity_and_cold_state_match_the_materialized_vector() {
    let v = vectors();
    let provider = provider();
    let profile = validate_keyless_provider(&provider).expect("provider is selected");
    assert_eq!(
        profile.provider_sha256().to_hex(),
        v["provider_sha256"].as_str().unwrap()
    );
    let owner = new_owner(0);
    assert_state(&owner.state, &v["cold"]["state"]);
    assert_eq!(
        hex(&owner.state.canonical_bytes()),
        v["cold"]["canonical_hex"]
    );
    assert_eq!(owner.state.sha256().to_hex(), v["cold"]["sha256"]);

    let mut unsupported = serde_json::from_slice::<Value>(&provider).unwrap();
    unsupported["profile_id"] = Value::String("private-auth.v1".into());
    let unsupported = serde_json::to_vec(&unsupported).unwrap();
    assert_eq!(
        IntegrityProgrammeOwner::cold("P0", 0, 5, &unsupported)
            .unwrap_err()
            .code()
            .as_str(),
        "UNSUPPORTED_PROFILE"
    );
}

#[test]
fn five_continuous_calls_match_receipts_frames_and_state_transitions() {
    let v = vectors();
    assert_eq!(v["schema"], "integrity-owner-root-vectors.v2");
    assert_eq!(v["input_only"], true);
    assert_eq!(v["expected_used_as_seed"], false);
    let provider = provider();
    let mut owner = new_owner(0);

    for (index, call_vector) in v["continuous_calls"].as_array().unwrap().iter().enumerate() {
        let input = TestInput::from_vector(&call_vector["input"]);
        assert_materialized_input(&call_vector["input"], &input);
        let result = owner
            .process(input.as_input(), &provider)
            .unwrap_or_else(|error| panic!("call {index} failed: {error:?}"));
        let expected = &call_vector["expected"];
        assert_eq!(
            u64::from(result.commit_count),
            expected["commit_count"].as_u64().unwrap()
        );
        assert_eq!(result.before_sha256.to_hex(), expected["before_sha256"]);
        assert_eq!(result.after_sha256.to_hex(), expected["after_sha256"]);
        assert_eq!(sha256_hex(&result.frame), expected["frame_sha256"]);
        assert_eq!(
            result.receipts.len(),
            expected["receipts"].as_array().unwrap().len()
        );
        for (actual, expected_receipt) in result
            .receipts
            .iter()
            .zip(expected["receipts"].as_array().unwrap())
        {
            assert_receipt(actual, expected_receipt);
        }
        assert_state(&owner.state, &expected["state"]);
    }
}

#[test]
fn production_stage_returns_owned_after_without_mutating_the_committed_owner() {
    let v = vectors();
    let provider = provider();
    let owner = new_owner(0);
    let before = owner.state.sha256();
    let call = &v["continuous_calls"][0];
    let input = TestInput::from_vector(&call["input"]);
    let proposal = owner.stage_source(input.as_input(), &provider).unwrap();
    assert_eq!(owner.state.sha256(), before);
    assert_eq!(
        proposal.after_state().sha256().to_hex(),
        call["expected"]["after_sha256"]
    );
    assert_eq!(
        sha256_hex(proposal.result().frame()),
        call["expected"]["frame_sha256"]
    );
    let (after_owner, result) = proposal.into_parts();
    assert_state(&after_owner.state, &call["expected"]["state"]);
    assert_eq!(result.commit_count(), 1);
}

#[test]
fn every_materialized_failure_retires_without_committing_or_publishing() {
    let v = vectors();
    let provider = provider();

    for case in v["failure_cases"].as_array().unwrap() {
        let mut input = TestInput::from_vector(&case["input"]);
        assert_materialized_input(&case["input"], &input);
        let mut call_provider = provider.clone();
        let mut fault = None;
        let mutation = &case["mutation"];
        match case["id"].as_str().unwrap() {
            "wrong_source" => {
                input.identity.source_index =
                    CodecSourceIndex::new(mutation["to"].as_u64().unwrap());
            }
            "wrong_dispatch" => {
                input.identity.dispatch = JocDispatchIndex::new(mutation["to"].as_u64().unwrap());
            }
            "wrong_au" => {
                input.identity.au_index = mutation["to"].as_u64().map(PublishedAuIndex::new);
            }
            "provider_mismatch" => {
                call_provider = serde_json::to_vec(&mutation["mutated_provider"]).unwrap();
            }
            "nonzero_checksum_slot" => {
                let offset = usize::try_from(mutation["offset"].as_u64().unwrap()).unwrap();
                input.zero_slot_container[offset] =
                    u8::from_str_radix(mutation["to_hex"].as_str().unwrap(), 16).unwrap();
            }
            "frame_length" => {
                input
                    .frame_template
                    .truncate(usize::try_from(mutation["to_bytes"].as_u64().unwrap()).unwrap());
            }
            "carrier_overflow" => {
                input.container_offset = usize::try_from(mutation["to"].as_u64().unwrap()).unwrap();
            }
            "injected_source" => fault = Some(IntegrityStage::Source),
            "injected_dispatch" => fault = Some(IntegrityStage::Dispatch),
            "injected_au" => fault = Some(IntegrityStage::Au),
            unknown => panic!("unknown failure vector {unknown}"),
        }

        let mut owner = new_owner(0);
        let before = owner.state.sha256();
        let error = match fault {
            Some(stage) => owner.process_with_test_fault(input.as_input(), &call_provider, stage),
            None => owner.process(input.as_input(), &call_provider),
        }
        .expect_err("failure vector must fail");
        assert_eq!(error.code().as_str(), case["expected"]["error"]);
        assert!(owner.retired);
        assert_eq!(owner.state.sha256(), before);
        assert_eq!(
            owner.state.sha256().to_hex(),
            case["expected"]["before_sha256"]
        );
        assert_eq!(
            case["expected"]["after_sha256"],
            case["expected"]["before_sha256"]
        );
        assert_eq!(case["expected"]["receipts"].as_array().unwrap().len(), 0);
        assert_eq!(case["expected"]["commit_count"], 0);
        assert_eq!(case["expected"]["frame_sha256"], Value::Null);
    }
}

#[test]
fn reset_retry_and_finalize_follow_materialized_sequences() {
    let v = vectors();
    let provider = provider();
    let calls = v["continuous_calls"].as_array().unwrap();
    let reset_state = &v["reset"]["state"];

    let mut owner = new_owner(0);
    let first = TestInput::from_vector(&calls[0]["input"]);
    owner.process(first.as_input(), &provider).unwrap();
    owner = owner
        .reset(1, &provider)
        .expect("reset starts a fresh attempt");
    assert_state(&owner.state, reset_state);
    assert_eq!(
        hex(&owner.state.canonical_bytes()),
        v["reset"]["canonical_hex"]
    );
    assert_eq!(owner.state.sha256().to_hex(), v["reset"]["sha256"]);

    let retry = &v["retry"];
    let mut failed_owner = new_owner(0);
    let retry_input = TestInput::from_vector(&calls[0]["input"]);
    let error = failed_owner
        .process_with_test_fault(retry_input.as_input(), &provider, IntegrityStage::Au)
        .expect_err("AU fault retires the old attempt");
    assert_eq!(error.code().as_str(), retry["failure"]["error"]);
    assert!(failed_owner.retired);
    assert_eq!(
        failed_owner.state.sha256().to_hex(),
        retry["failure"]["before_sha256"]
    );
    let mut retried = failed_owner
        .reset(1, &provider)
        .expect("new attempt resets cold");
    assert_state(&retried.state, &retry["new_attempt_reset_state"]);
    let new_attempt_input = TestInput::from_vector(&retry["new_attempt_input"]);
    let result = retried
        .process(new_attempt_input.as_input(), &provider)
        .expect("new attempt accepts the retry");
    assert_eq!(result.commit_count, 1);
    assert_eq!(
        result.after_sha256.to_hex(),
        retry["new_attempt_reset_retry"]["after_sha256"]
    );
    assert_eq!(
        sha256_hex(&result.frame),
        retry["new_attempt_reset_retry"]["frame_sha256"]
    );

    let mut finalized = new_owner(0);
    for call in calls {
        let input = TestInput::from_vector(&call["input"]);
        finalized.process(input.as_input(), &provider).unwrap();
    }
    assert_eq!(
        finalized.state.sha256().to_hex(),
        calls[4]["expected"]["after_sha256"]
    );
    finalized
        .finalize()
        .expect("complete source count may finalize without a receipt");
    assert_eq!(
        finalized.state.sha256().to_hex(),
        v["finalize"]["after_sha256"]
    );
    assert_eq!(
        hex(&finalized.state.canonical_bytes()),
        v["finalize"]["after_canonical_hex"]
    );
    assert_state(&finalized.state, &v["finalize"]["after"]);
    assert_eq!(v["finalize"]["extra_flush_outputs"], 0);
    let finalized_sha256 = finalized.state.sha256();
    assert!(finalized.finalize().is_err(), "second finalize fails");
    assert_eq!(finalized.state.sha256(), finalized_sha256);
    let after_eof_input = TestInput::from_vector(&calls[0]["input"]);
    assert!(
        finalized
            .process(after_eof_input.as_input(), &provider)
            .is_err()
    );
    assert_eq!(finalized.state.sha256(), finalized_sha256);
}

#[test]
fn crc16_residual_is_zero_for_each_published_frame() {
    let v = vectors();
    let provider = provider();
    let mut owner = new_owner(0);
    for call in v["continuous_calls"].as_array().unwrap() {
        let input = TestInput::from_vector(&call["input"]);
        let result = owner.process(input.as_input(), &provider).unwrap();
        assert_eq!(eac3_crc16(&result.frame[2..]), 0);
        assert_eq!(
            &result.frame[4094..4096],
            &result.receipts[2].details.crc16().unwrap().to_be_bytes()
        );
    }
}
