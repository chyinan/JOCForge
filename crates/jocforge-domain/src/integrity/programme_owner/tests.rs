use super::*;
use crate::timeline::FrontendCallbackIndex;
use serde_json::Value;
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/integrity_owner";
const FRAME_BYTES: usize = 4_096;
const FRAME_CRC_START: usize = FRAME_BYTES - 2;
const FRAME_CRC_INPUT_END: usize = FRAME_CRC_START;

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

fn crc_closed_frame(codec_rate: crate::profile::CodecRateProfile) -> Vec<u8> {
    let size = crate::codec::frame_size::Eac3FrameSize::from_codec_rate(codec_rate).unwrap();
    let mut frame = vec![0_u8; size.bytes()];
    frame[..2].copy_from_slice(&[0x0b, 0x77]);
    for offset in 0..11 {
        let value = (size.code().value() >> (10 - offset)) & 1;
        if value != 0 {
            let bit = 21 + offset;
            frame[bit / 8] |= 1 << (7 - (bit % 8));
        }
    }
    let crc_start = frame.len() - 2;
    let crc = eac3_crc16(&frame[FRAME_CRC_INPUT_START..crc_start]);
    frame[crc_start..].copy_from_slice(&crc.to_be_bytes());
    frame
}

#[test]
fn integrity_owner_rate_identity_and_crc_window_follow_each_f12_frame_extent() {
    let provider = validate_keyless_provider(&provider()).unwrap();
    let rows = [
        (crate::profile::CodecRateProfile::Kbps384, 1_536),
        (crate::profile::CodecRateProfile::Kbps448, 1_792),
        (crate::profile::CodecRateProfile::Kbps576, 2_304),
        (crate::profile::CodecRateProfile::Kbps640, 2_560),
        (crate::profile::CodecRateProfile::Kbps768, 3_072),
        (crate::profile::CodecRateProfile::Kbps1024, 4_096),
    ];
    let mut owner_state_hashes = Vec::new();

    for (codec_rate, frame_bytes) in rows {
        let owner = IntegrityProgrammeOwner::cold_from_provider_for_rate(
            "integrity-rate-test",
            0,
            3,
            &provider,
            codec_rate,
        )
        .unwrap();
        assert_eq!(owner.codec_rate(), codec_rate);
        assert_eq!(
            owner.committed_state().profile_id(),
            integrity_profile_id(codec_rate)
        );
        let owner_state_hash = owner
            .owner_state_snapshot()
            .state_sha256()
            .as_bytes()
            .to_vec();
        assert!(!owner_state_hashes.contains(&owner_state_hash));
        owner_state_hashes.push(owner_state_hash);

        let frame = crc_closed_frame(codec_rate);
        assert_eq!(frame.len(), frame_bytes);
        let (crc_start, expected_crc) = validate_completed_f12_frame(&frame, codec_rate).unwrap();
        assert_eq!(crc_start, frame_bytes - 2);
        assert_eq!(
            u16::from_be_bytes([frame[crc_start], frame[crc_start + 1]]),
            expected_crc
        );
        assert_eq!(eac3_crc16(&frame[FRAME_CRC_INPUT_START..]), 0);

        if codec_rate != crate::profile::CodecRateProfile::Kbps1024 {
            assert_eq!(
                validate_completed_f12_frame(&frame, crate::profile::CodecRateProfile::Kbps1024)
                    .unwrap_err()
                    .code(),
                IntegrityErrorCode::FrameShape
            );
        }
    }
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

fn gtc_callback_identity(
    identity: &IntegrityIdentity,
) -> (
    ProgrammeId,
    AttemptId,
    CallbackTransactionId,
    CallbackLineageIdentity,
) {
    let programme_id = ProgrammeId::new(&identity.programme_id).expect("programme is nonempty");
    let attempt = AttemptId::new(identity.attempt);
    let transaction_id = CallbackTransactionId::new(&identity.transaction_id)
        .expect("transaction identity is nonempty");
    let callback = identity
        .source_index
        .value()
        .checked_add(2)
        .and_then(|value| u32::try_from(value).ok())
        .expect("selected source maps to a frontend callback index");
    let callback_identity = CallbackLineageIdentity::new(
        programme_id.clone(),
        attempt,
        FrontendCallbackIndex::new(callback),
    );
    (programme_id, attempt, transaction_id, callback_identity)
}

fn completed_frame_with(container_offset: Option<usize>, container: Option<&[u8]>) -> Vec<u8> {
    let mut frame = vec![0_u8; FRAME_BYTES];
    if let (Some(offset), Some(container)) = (container_offset, container) {
        frame[offset..offset + container.len()].copy_from_slice(container);
    }
    let crc16 = eac3_crc16(&frame[FRAME_CRC_INPUT_START..FRAME_CRC_INPUT_END]);
    frame[FRAME_CRC_START..FRAME_BYTES].copy_from_slice(&crc16.to_be_bytes());
    frame
}

fn completed_frame_with_bit_cursor(bit_offset: u32, container: &[u8]) -> Vec<u8> {
    let mut frame = vec![0_u8; FRAME_BYTES];
    for (byte_index, source_byte) in container.iter().copied().enumerate() {
        for bit_index in 0..8_u32 {
            let frame_bit = bit_offset + u32::try_from(byte_index).unwrap() * 8 + bit_index;
            let frame_byte = usize::try_from(frame_bit / 8).unwrap();
            let frame_shift = 7 - (frame_bit % 8);
            let source_shift = 7 - bit_index;
            if (source_byte >> source_shift) & 1 != 0 {
                frame[frame_byte] |= 1 << frame_shift;
            }
        }
    }
    let crc16 = eac3_crc16(&frame[FRAME_CRC_INPUT_START..FRAME_CRC_INPUT_END]);
    frame[FRAME_CRC_START..FRAME_BYTES].copy_from_slice(&crc16.to_be_bytes());
    frame
}

fn f12_identity(source_index: u64) -> IntegrityIdentity {
    IntegrityIdentity {
        programme_id: "P0".to_owned(),
        attempt: 0,
        transaction_id: format!("f12-source-{source_index}"),
        source_index: CodecSourceIndex::new(source_index),
        dispatch: JocDispatchIndex::new(source_index + 1),
        au_index: if source_index >= 2 {
            Some(PublishedAuIndex::new(source_index - 2))
        } else {
            None
        },
    }
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
fn cold_owner_reuses_the_admitted_typed_provider() {
    let provider_bytes = provider();
    let admitted_provider = validate_keyless_provider(&provider_bytes).unwrap();
    let from_json = IntegrityProgrammeOwner::cold("P0", 0, 5, &provider_bytes).unwrap();
    let from_admitted =
        IntegrityProgrammeOwner::cold_from_provider("P0", 0, 5, &admitted_provider).unwrap();
    assert_eq!(from_json.committed_state(), from_admitted.committed_state());
}

#[test]
fn f12_completed_frame_stage_advances_suppressed_sources_without_publishing_aus() {
    let provider_bytes = provider();
    let provider_value = validate_keyless_provider(&provider_bytes).unwrap();
    let mut owner = new_owner(0);

    for source_index in 0..2 {
        let identity = f12_identity(source_index);
        let (programme_id, attempt, transaction_id, callback_identity) =
            gtc_callback_identity(&identity);
        let frame = completed_frame_with(None, None);
        let proposal = owner
            .stage_f12_source_for_callback(
                IntegrityF12SourceInput {
                    identity,
                    payload: F14kPayloadInput::AbsentPublishedPayloadV1,
                    completed_frame: F12CompletedFrameInput {
                        frame: &frame,
                        emdf_container_start: None,
                    },
                },
                &provider_value,
                &programme_id,
                attempt,
                &transaction_id,
                &callback_identity,
            )
            .expect("F12 produces CRC-valid frames for priming sources");
        let (next_owner, receipts, owner_receipt) = proposal.into_parts();
        assert_eq!(
            receipts.source().payload(),
            IntegrityPayloadEvidence::AbsentPublishedPayloadV1
        );
        assert!(!receipts.au().published());
        assert_eq!(receipts.au().crc_residual(), 0);
        assert_eq!(
            next_owner.committed_state().next_source_index().value(),
            source_index + 1
        );
        assert_eq!(next_owner.committed_state().published_au_count(), 0);
        assert_eq!(owner_receipt.after_generation(), source_index + 1);
        owner = next_owner;
    }
}

#[test]
fn f12_completed_frame_stage_binds_protected_emdf_at_dynamic_bit_cursor() {
    let provider_bytes = provider();
    let provider_value = validate_keyless_provider(&provider_bytes).unwrap();
    let mut owner = new_owner(0);
    for source_index in 0..2 {
        let identity = f12_identity(source_index);
        let (programme_id, attempt, transaction_id, callback_identity) =
            gtc_callback_identity(&identity);
        let frame = completed_frame_with(None, None);
        let proposal = owner
            .stage_f12_source_for_callback(
                IntegrityF12SourceInput {
                    identity,
                    payload: F14kPayloadInput::AbsentPublishedPayloadV1,
                    completed_frame: F12CompletedFrameInput {
                        frame: &frame,
                        emdf_container_start: None,
                    },
                },
                &provider_value,
                &programme_id,
                attempt,
                &transaction_id,
                &callback_identity,
            )
            .unwrap();
        owner = proposal.into_parts().0;
    }

    let zero_slot_container = [0x58, 0x38, 0x00, 0x07, 0x04, 0x80, 0, 0, 0, 0, 0];
    let layout = serde_json::json!({
        "bit_order": "MSB-first",
        "terminal_id": { "value": 0, "start_bit": 32, "width_bits": 5 },
        "primary_length_code": { "value": 2, "start_bit": 37, "width_bits": 2, "byte_count": 4 },
        "secondary_length_code": { "value": 1, "start_bit": 39, "width_bits": 2, "byte_count": 1 },
        "primary_protection_range_bits": [41, 73],
        "secondary_protection_range_bits": [73, 81],
        "terminal_padding_range_bits": [81, 88],
        "terminal_padding_bits": 7,
        "wire_bytes": 11,
        "body_bytes": 7
    });
    let protected = super::super::keyless_checksum::protect_emdf_container(
        &provider_value,
        &zero_slot_container,
        &layout,
    )
    .unwrap();
    let carrier_bit_offset = 517;
    let frame = completed_frame_with_bit_cursor(carrier_bit_offset, protected.bytes());
    let identity = f12_identity(2);
    let (programme_id, attempt, transaction_id, callback_identity) =
        gtc_callback_identity(&identity);
    let proposal = owner
        .stage_f12_source_for_callback(
            IntegrityF12SourceInput {
                identity,
                payload: F14kPayloadInput::ProtectedEmdf {
                    zero_slot_container: &zero_slot_container,
                    protected_container: &protected,
                },
                completed_frame: F12CompletedFrameInput {
                    frame: &frame,
                    emdf_container_start: Some(FrameBitCursor::new(carrier_bit_offset)),
                },
            },
            &provider_value,
            &programme_id,
            attempt,
            &transaction_id,
            &callback_identity,
        )
        .expect("F12 frame cursor binds the exact protected bytes");
    let (next_owner, receipts, owner_receipt) = proposal.into_parts();
    assert_eq!(receipts.source().identity().source_index.value(), 2);
    assert_eq!(
        receipts.dispatch().emdf_container_start(),
        Some(FrameBitCursor::new(carrier_bit_offset))
    );
    assert!(receipts.au().published());
    assert_eq!(receipts.au().crc_residual(), 0);
    assert_eq!(next_owner.committed_state().next_source_index().value(), 3);
    assert_eq!(next_owner.committed_state().published_au_count(), 1);
    assert_eq!(owner_receipt.after_generation(), 3);
}

#[test]
fn flat7x_pair_f12_validates_role_rates_and_returns_one_shared_owner_transition() {
    let provider_bytes = provider();
    let provider_value = validate_keyless_provider(&provider_bytes).unwrap();
    let mut owner = new_owner(0);
    for source in 0..2 {
        let identity = f12_identity(source);
        let (programme_id, attempt, transaction_id, callback_identity) =
            gtc_callback_identity(&identity);
        let frame = crc_closed_frame(crate::profile::CodecRateProfile::Kbps1024);
        owner = owner
            .stage_f12_source_for_callback(
                IntegrityF12SourceInput {
                    identity,
                    payload: F14kPayloadInput::AbsentPublishedPayloadV1,
                    completed_frame: F12CompletedFrameInput {
                        frame: &frame,
                        emdf_container_start: None,
                    },
                },
                &provider_value,
                &programme_id,
                attempt,
                &transaction_id,
                &callback_identity,
            )
            .unwrap()
            .into_parts()
            .0;
    }

    let zero_slot_container = [0x58, 0x38, 0x00, 0x07, 0x04, 0x80, 0, 0, 0, 0, 0];
    let layout = serde_json::json!({
        "bit_order": "MSB-first",
        "terminal_id": { "value": 0, "start_bit": 32, "width_bits": 5 },
        "primary_length_code": { "value": 2, "start_bit": 37, "width_bits": 2, "byte_count": 4 },
        "secondary_length_code": { "value": 1, "start_bit": 39, "width_bits": 2, "byte_count": 1 },
        "primary_protection_range_bits": [41, 73],
        "secondary_protection_range_bits": [73, 81],
        "terminal_padding_range_bits": [81, 88],
        "terminal_padding_bits": 7,
        "wire_bytes": 11,
        "body_bytes": 7
    });
    let protected = super::super::keyless_checksum::protect_emdf_container(
        &provider_value,
        &zero_slot_container,
        &layout,
    )
    .unwrap();
    let d0_emdf_container_start = FrameBitCursor::new(517);
    let i0_frame = crc_closed_frame(crate::profile::CodecRateProfile::Kbps640);
    let d0_frame =
        completed_frame_with_bit_cursor(d0_emdf_container_start.bit_offset(), protected.bytes());
    let identity = f12_identity(2);
    let (programme_id, attempt, transaction_id, callback_identity) =
        gtc_callback_identity(&identity);
    let owner_before = owner.owner_state_snapshot();

    let proposal = owner
        .stage_flat7x_i0_d0_f12_pair_for_callback(
            Flat7xI0D0F12PairInput {
                identity: identity.clone(),
                rate_plan: Flat7xIntegrityPairRatePlan::flat7x_i0_640_d0_1024(),
                i0_frame: &i0_frame,
                d0_frame: &d0_frame,
                d0_emdf_container_start,
                d0_zero_slot_container: &zero_slot_container,
                d0_protected_container: &protected,
            },
            &provider_value,
            &programme_id,
            attempt,
            &transaction_id,
            &callback_identity,
        )
        .unwrap();

    let i0 = proposal.receipts().i0();
    let d0 = proposal.receipts().d0();
    assert_eq!(i0.au().identity(), &identity);
    assert_eq!(d0.au().identity(), &identity);
    assert_eq!(i0.dispatch().frame_bytes(), 2_560);
    assert_eq!(d0.dispatch().frame_bytes(), 4_096);
    assert_eq!(i0.dispatch().emdf_container_start(), None);
    assert_eq!(
        d0.dispatch().emdf_container_start(),
        Some(d0_emdf_container_start)
    );
    assert_eq!(
        i0.source().payload(),
        IntegrityPayloadEvidence::AbsentPublishedPayloadV1
    );
    assert!(matches!(
        d0.source().payload(),
        IntegrityPayloadEvidence::ProtectedEmdf { .. }
    ));
    assert!(i0.au().published());
    assert!(d0.au().published());
    assert_eq!(i0.au().crc_residual(), 0);
    assert_eq!(d0.au().crc_residual(), 0);
    assert_eq!(i0.source().before_generation(), owner_before.generation());
    assert_eq!(d0.source().before_generation(), owner_before.generation());
    assert_eq!(
        i0.source().proposed_after_generation(),
        d0.source().proposed_after_generation()
    );
    assert_eq!(
        proposal.owner_receipt_proposal().before_generation(),
        owner_before.generation()
    );
    assert_eq!(
        proposal.owner_receipt_proposal().after_generation(),
        owner_before.generation() + 1
    );
    assert_eq!(
        owner.committed_state().generation(),
        owner_before.generation()
    );
    assert_eq!(
        proposal
            .after_owner()
            .committed_state()
            .published_au_count(),
        1
    );

    let rejected = owner
        .stage_flat7x_i0_d0_f12_pair_for_callback(
            Flat7xI0D0F12PairInput {
                identity,
                rate_plan: Flat7xIntegrityPairRatePlan::flat7x_i0_640_d0_1024(),
                i0_frame: &i0_frame,
                d0_frame: &d0_frame,
                d0_emdf_container_start: FrameBitCursor::new(
                    d0_emdf_container_start.bit_offset() + 1,
                ),
                d0_zero_slot_container: &zero_slot_container,
                d0_protected_container: &protected,
            },
            &provider_value,
            &programme_id,
            attempt,
            &transaction_id,
            &callback_identity,
        )
        .unwrap_err();
    assert_eq!(rejected.code(), IntegrityErrorCode::ChecksumSlots);
    assert_eq!(
        owner.committed_state().generation(),
        owner_before.generation()
    );
}

#[test]
fn f12_completed_frame_stage_rejects_absent_payload_with_unexpected_cursor() {
    let provider_bytes = provider();
    let provider_value = validate_keyless_provider(&provider_bytes).unwrap();
    let owner = new_owner(0);
    let identity = f12_identity(0);
    let (programme_id, attempt, transaction_id, callback_identity) =
        gtc_callback_identity(&identity);
    let frame = completed_frame_with(None, None);
    let error = owner
        .stage_f12_source_for_callback(
            IntegrityF12SourceInput {
                identity,
                payload: F14kPayloadInput::AbsentPublishedPayloadV1,
                completed_frame: F12CompletedFrameInput {
                    frame: &frame,
                    emdf_container_start: Some(FrameBitCursor::new(17)),
                },
            },
            &provider_value,
            &programme_id,
            attempt,
            &transaction_id,
            &callback_identity,
        )
        .unwrap_err();
    assert_eq!(error.code(), IntegrityErrorCode::AuOrder);
}

#[test]
fn frame_payload_bit_comparison_accepts_non_byte_cursor_and_rejects_wrong_cursor() {
    let payload = [0xa5, 0x69, 0xf0];
    let frame = completed_frame_with_bit_cursor(21, &payload);
    assert!(super::frame_contains_payload_at_cursor(
        &frame,
        FrameBitCursor::new(21),
        &payload,
    ));
    assert!(!super::frame_contains_payload_at_cursor(
        &frame,
        FrameBitCursor::new(22),
        &payload,
    ));
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
fn gtc_owner_snapshot_contains_exact_state_bytes_hashes_and_reference() {
    let v = vectors();
    let owner = new_owner(0);
    let snapshot = owner.owner_state_snapshot();
    let gtc_snapshot = snapshot.gtc_snapshot().unwrap();
    let expected = &v["cold"];

    assert_eq!(snapshot.owner_id(), "integrity");
    assert_eq!(snapshot.state_ref(), "owners.integrity");
    assert_eq!(snapshot.generation(), 0);
    assert_eq!(snapshot.state(), owner.committed_state());
    assert_eq!(
        snapshot.canonical_bytes(),
        owner.committed_state().canonical_bytes()
    );
    assert_eq!(hex(snapshot.canonical_bytes()), expected["canonical_hex"]);
    assert_eq!(snapshot.state_sha256().to_hex(), expected["sha256"]);
    assert_eq!(snapshot.cold_sha256(), snapshot.state_sha256());
    assert_eq!(gtc_snapshot.owner_id().as_str(), "integrity");
    assert_eq!(gtc_snapshot.generation(), snapshot.generation());
    assert_eq!(gtc_snapshot.canonical_bytes(), snapshot.canonical_bytes());
    assert_eq!(
        gtc_snapshot.state_sha256(),
        *snapshot.state_sha256().as_bytes()
    );
    assert_eq!(
        gtc_snapshot.cold_sha256(),
        *snapshot.cold_sha256().as_bytes()
    );
}

#[test]
fn gtc_receipt_proposals_bind_suppressed_and_published_source_stages() {
    let v = vectors();
    let provider = provider();
    let calls = v["continuous_calls"].as_array().unwrap();
    let mut owner = new_owner(0);

    for (index, call) in calls.iter().take(3).enumerate() {
        let input = TestInput::from_vector(&call["input"]);
        let (programme_id, attempt, transaction_id, callback_identity) =
            gtc_callback_identity(&input.identity);
        let before = owner.owner_state_snapshot();
        let proposal = owner
            .stage_source_for_callback(
                input.as_input(),
                &provider,
                &programme_id,
                attempt,
                &transaction_id,
                &callback_identity,
            )
            .expect("typed callback lineage admits the same integrity source");

        assert_eq!(owner.owner_state_snapshot(), before);
        let receipt = proposal.owner_receipt_proposal();
        assert_eq!(receipt.owner_id(), "integrity");
        assert_eq!(receipt.programme_id(), &programme_id);
        assert_eq!(receipt.attempt(), attempt);
        assert_eq!(receipt.transaction_id(), &transaction_id);
        assert_eq!(receipt.callback_identity(), &callback_identity);
        assert_eq!(receipt.before(), &before);
        assert_eq!(receipt.before_generation(), before.generation());
        assert_eq!(receipt.before_sha256(), before.state_sha256());
        assert!(receipt.changed());
        assert_eq!(receipt.after().generation(), before.generation() + 1);
        assert_eq!(
            receipt.after().state_sha256(),
            proposal.source_proposal().result().after_sha256()
        );
        assert_eq!(
            receipt.after().state_sha256().to_hex(),
            call["expected"]["after_sha256"]
        );
        assert_eq!(receipt.after().state_ref(), "owners.integrity");

        let source_identity = proposal.source_proposal().result().receipts()[0].identity();
        assert_eq!(source_identity.transaction_id, transaction_id.as_str());
        assert_eq!(source_identity.source_index, input.identity.source_index);
        assert!(
            proposal.source_proposal().result().receipts()[2]
                .details()
                .crc16()
                .is_some()
        );
        if index < 2 {
            assert_eq!(source_identity.au_index, None);
            assert!(matches!(
                proposal.source_proposal().result().receipts()[2].details(),
                IntegrityStageDetails::Au {
                    crc_residual: 0,
                    published: false,
                    ..
                }
            ));
        } else {
            assert_eq!(source_identity.au_index, Some(PublishedAuIndex::new(0)));
            assert!(matches!(
                proposal.source_proposal().result().receipts()[2].details(),
                IntegrityStageDetails::Au {
                    published: true,
                    crc_residual: 0,
                    ..
                }
            ));
        }

        let (after_owner, result, receipt) = proposal.into_parts();
        assert_eq!(result.commit_count(), 1, "local source-stage count only");
        assert_eq!(after_owner.committed_state(), receipt.after().state());
        assert_eq!(owner.committed_state().sha256(), before.state_sha256());
        owner = after_owner;
    }
}

#[test]
fn gtc_receipt_proposal_rejects_identity_mismatches_without_mutating_owner() {
    let v = vectors();
    let provider = provider();
    let input = TestInput::from_vector(&v["continuous_calls"][0]["input"]);
    let owner = new_owner(0);
    let before = owner.committed_state().sha256();
    let (programme_id, attempt, transaction_id, callback_identity) =
        gtc_callback_identity(&input.identity);

    let mismatched_transaction = CallbackTransactionId::new("different-transaction")
        .expect("nonempty mismatch remains a typed transaction id");
    assert_eq!(
        owner
            .stage_source_for_callback(
                input.as_input(),
                &provider,
                &programme_id,
                attempt,
                &mismatched_transaction,
                &callback_identity,
            )
            .unwrap_err()
            .code(),
        IntegrityErrorCode::TransactionIdentity
    );

    let mismatched_callback = CallbackLineageIdentity::new(
        programme_id.clone(),
        attempt,
        FrontendCallbackIndex::new(callback_identity.callback_index().value() + 1),
    );
    assert_eq!(
        owner
            .stage_source_for_callback(
                input.as_input(),
                &provider,
                &programme_id,
                attempt,
                &transaction_id,
                &mismatched_callback,
            )
            .unwrap_err()
            .code(),
        IntegrityErrorCode::SourceOrder
    );

    let mismatched_programme = ProgrammeId::new("different-programme")
        .expect("nonempty mismatch remains a typed programme id");
    assert_eq!(
        owner
            .stage_source_for_callback(
                input.as_input(),
                &provider,
                &mismatched_programme,
                attempt,
                &transaction_id,
                &callback_identity,
            )
            .unwrap_err()
            .code(),
        IntegrityErrorCode::ProgrammeIdentity
    );

    assert_eq!(owner.committed_state().sha256(), before);
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
