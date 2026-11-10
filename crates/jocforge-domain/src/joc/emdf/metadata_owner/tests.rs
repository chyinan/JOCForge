use super::*;
use crate::{
    joc::emdf::{
        id1_configuration::Id1MeasurementConfiguration, metadata_bodies::MetadataBodiesProposal,
    },
    joc::{
        emdf_carrier::{CarrierBitCursor, CarrierPlannerState},
        metadata_carrier::stage_metadata_carrier,
    },
    meter::measurement_receipt::{ProgrammeMeasurementReceipt, ProgrammeMeasurementReceiptInput},
};
use serde_json::{Value, json};
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/metadata_owner")
}

fn vectors() -> Value {
    serde_json::from_str(include_str!(
        "../../../../tests/fixtures/metadata_owner/METADATA-OWNER-VECTORS.json"
    ))
    .expect("metadata owner vectors must be valid JSON")
}

fn id1_configuration_for_kind(
    attempt: u64,
    gating_kind: &'static str,
) -> Id1MeasurementConfiguration {
    let measurement =
        ProgrammeMeasurementReceipt::from_verified_prepass(ProgrammeMeasurementReceiptInput {
            programme_id: crate::transaction::test_programme_id("P0"),
            attempt: crate::transaction::AttemptId::new(attempt),
            input_sha256: [0x11; 32],
            config_sha256: [0x22; 32],
            dialogue_word: (-18.0_f32).to_bits(),
            itu_word: (-17.0_f32).to_bits(),
            true_peak_word: (-4.0_f32).to_bits(),
            gating_kind,
            selected_lufs_word: (-17.0_f32).to_bits(),
            meter_terminal_sha256: [0x33; 32],
            normal_callbacks: 4_247,
            zero_drains: 3,
            energy_windows: 6_794,
        })
        .unwrap();
    Id1MeasurementConfiguration::from_measurement_for_test(&measurement).unwrap()
}

fn id1_configuration(attempt: u64) -> Id1MeasurementConfiguration {
    id1_configuration_for_kind(attempt, "DIALOGUE")
}

fn owner_for_attempt(attempt: u64) -> MetadataOwner {
    let root = fixture_root();
    MetadataOwner::new(
        id1_configuration(attempt),
        5,
        &std::fs::read(root.join("metadata-public-provider.json")).unwrap(),
        &std::fs::read(root.join("metadata-wrapper-config.json")).unwrap(),
    )
    .unwrap()
}

fn owner() -> MetadataOwner {
    owner_for_attempt(0)
}

fn owner_after_all_sources() -> MetadataOwner {
    let vectors = vectors();
    let mut owner = owner();
    for call in vectors["continuous_calls"].as_array().unwrap() {
        owner.process_source(source_call(call)).unwrap();
    }
    owner
}

fn prepared_eof() -> (MetadataOwner, MetadataEofProposal) {
    let mut owner = owner_after_all_sources();
    let proposal = owner.prepare_eof(5).unwrap();
    (owner, proposal)
}

fn hex_bytes(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digit = |byte: u8| match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                _ => panic!("invalid fixture hex"),
            };
            (digit(pair[0]) << 4) | digit(pair[1])
        })
        .collect()
}

fn source_call(case: &Value) -> MetadataSourceCall {
    let input = &case["input"];
    let identity = &input["identity"];
    let body = |id: &str| {
        input["bodies"].get(id).map(|body| MetadataPayloadBody {
            writer_generation: CodecSourceIndex::new(body["writer_generation"].as_u64().unwrap()),
            bytes: hex_bytes(body["payload_hex"].as_str().unwrap()),
        })
    };

    let programme_id =
        crate::transaction::test_programme_id(identity["programme_id"].as_str().unwrap());
    let attempt = crate::transaction::AttemptId::new(identity["attempt"].as_u64().unwrap());
    let source_index = CodecSourceIndex::new(identity["source_index"].as_u64().unwrap());
    let callback_index = crate::timeline::FrontendCallbackIndex::new(
        u32::try_from(source_index.value() + 2).unwrap(),
    );
    let source_lineage = crate::transaction::CallbackLineageIdentity::new(
        programme_id.clone(),
        attempt,
        callback_index,
    );
    MetadataSourceCall {
        programme_id,
        attempt,
        source_lineage,
        consumer_transaction_id: None,
        source_index,
        dispatch: JocDispatchIndex::new(identity["dispatch"].as_u64().unwrap()),
        au_index: identity["au_index"].as_u64().map(PublishedAuIndex::new),
        config: input["config"].as_array().unwrap().clone(),
        bodies: MetadataBodies {
            id11: body("11"),
            id2: body("2"),
            id14: body("14"),
            id14_provenance: None,
        },
        wrapper_configuration: input["wrapper_config"].clone(),
    }
}

fn bind_test_id14_provenance(
    call: &mut MetadataSourceCall,
    spatial_digest: crate::spatial::SpatialAudioDigest,
) {
    let body = call.bodies.id14.as_ref().expect("fixture ID14 body");
    let lineage = crate::joc::coefficient_fitting::FitSourceLineage::for_test(
        call.programme_id.clone(),
        call.attempt,
        call.source_lineage.clone(),
        call.source_index,
        call.dispatch,
        call.source_index.value(),
        spatial_digest,
    );
    let object_count = crate::joc::id14_body::Id14ObjectCount::from_selected_slots(15)
        .expect("selected ID14 count");
    // These queue fixtures are not complete ID14 writer vectors; the test-only
    // provenance exercises identity transport and never enters the parser path.
    let sequence_count = 0_u16;
    let producer_transaction_id = crate::transaction::CallbackTransactionId::new(&format!(
        "metadata-test-source-{}",
        call.source_index.value()
    ))
    .unwrap();
    call.consumer_transaction_id = Some(producer_transaction_id.clone());
    call.bodies.id14_provenance = Some(
        crate::joc::id14_body::Id14ProducerProvenance::new(
            crate::joc::id14_body::Id14ProducerIdentity {
                lineage,
                producer_transaction_id: Some(producer_transaction_id),
                producer_generation: call.source_index,
                dispatch: call.dispatch,
                writer: crate::timeline::JocGeneratedWriterIndex::new(call.source_index.value()),
                sequence_count: crate::joc::id14_body::Id14SequenceCount::new(sequence_count)
                    .expect("selected sequence count"),
                object_count,
            },
            &body.bytes,
        )
        .expect("same-source ID14 provenance"),
    );
}

fn bytes_hex(bytes: &[u8]) -> String {
    hex_encode(bytes)
}

fn test_callback_transaction_id(
    source_index: CodecSourceIndex,
) -> crate::transaction::CallbackTransactionId {
    crate::transaction::CallbackTransactionId::new(&format!(
        "metadata-test-consumer-{}",
        source_index.value()
    ))
    .unwrap()
}

fn flat7x_receipt_pair_vector() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../outputs/IMPLEMENTER_SUPPLEMENTS/FLAT7X-F15-RECEIPT-PAIR-INPUT-ONLY-V1/INPUT-ONLY-VECTORS.json",
    );
    serde_json::from_slice(&std::fs::read(path).expect("Wash receipt-pair vector is present"))
        .expect("Wash receipt-pair vector is valid JSON")
}

fn write_test_bits(bytes: &mut [u8], start_bit: usize, width: usize, value: u64) {
    for offset in 0..width {
        let bit_index = start_bit + offset;
        let mask = 1_u8 << (7 - bit_index % 8);
        let set = (value >> (width - offset - 1)) & 1 != 0;
        if set {
            bytes[bit_index / 8] |= mask;
        } else {
            bytes[bit_index / 8] &= !mask;
        }
    }
}

fn synthetic_receipt_gate_emdf() -> (
    Vec<u8>,
    crate::integrity::keyless_checksum::ProtectedEmdfContainer,
) {
    let mut zero_slot = vec![0_u8; 16];
    zero_slot[..2].copy_from_slice(&[0x58, 0x38]);
    zero_slot[2..4].copy_from_slice(&12_u16.to_be_bytes());
    write_test_bits(&mut zero_slot, 37, 2, 2);
    write_test_bits(&mut zero_slot, 39, 2, 1);

    let layout = json!({
        "bit_order": "MSB-first",
        "terminal_id": { "value": 0, "start_bit": 32, "width_bits": 5 },
        "primary_length_code": { "value": 2, "start_bit": 37, "width_bits": 2, "byte_count": 4 },
        "secondary_length_code": { "value": 1, "start_bit": 39, "width_bits": 2, "byte_count": 1 },
        "primary_protection_range_bits": [41, 73],
        "secondary_protection_range_bits": [73, 81],
        "terminal_padding_range_bits": [81, 128],
        "terminal_padding_bits": 47,
        "wire_bytes": 16,
        "body_bytes": 12
    });
    let provider_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/integrity_owner/integrity-public-provider.json");
    let provider_bytes = std::fs::read(provider_path).expect("keyless provider fixture is present");
    let provider = crate::integrity::keyless_checksum::validate_keyless_provider(&provider_bytes)
        .expect("selected public keyless provider fixture validates");
    let protected =
        crate::integrity::keyless_checksum::protect_emdf_container(&provider, &zero_slot, &layout)
            .expect("synthetic 16-byte checksum input is protectable");
    (zero_slot, protected)
}

fn receipt_gate_crc_valid_frame(frame_bytes: usize, marker: u8) -> Vec<u8> {
    let size = crate::codec::frame_size::Eac3FrameSize::from_frame_bytes(frame_bytes).unwrap();
    let mut bytes = vec![0_u8; frame_bytes];
    bytes[..2].copy_from_slice(&[0x0b, 0x77]);
    bytes[100] = marker;
    for offset in 0..11 {
        let value = (size.code().value() >> (10 - offset)) & 1;
        if value != 0 {
            let bit = 21 + offset;
            bytes[bit / 8] |= 1 << (7 - (bit % 8));
        }
    }
    let crc_offset = bytes.len() - 2;
    let crc = crate::integrity::crc16::eac3_crc16(&bytes[2..crc_offset]);
    bytes[crc_offset..].copy_from_slice(&crc.to_be_bytes());
    assert_eq!(crate::integrity::crc16::eac3_crc16(&bytes[2..]), 0);
    bytes
}

fn receipt_gate_sequence(
    source_generation: u64,
    container_bytes: usize,
) -> crate::profile::Flat7xAuByteSequenceCandidate {
    use crate::profile::{
        D0EmdfSkipFieldExtent, D0FrameExtent, Flat7xAuCandidate, Flat7xI0D0TargetPlan,
        Flat7xSourceGeneration, I0FrameExtent, compose_flat7x_au_byte_sequence,
        validate_flat7x_au_candidate,
    };

    let generation = Flat7xSourceGeneration::new(source_generation);
    let candidate = Flat7xAuCandidate::new(
        Some(I0FrameExtent::new(generation, 2_560)),
        Some(D0FrameExtent::new(generation, 4_096)),
        Some(D0EmdfSkipFieldExtent::new(
            generation,
            2,
            1,
            Some(u16::try_from(container_bytes).unwrap()),
            false,
        )),
    );
    let validated =
        validate_flat7x_au_candidate(Flat7xI0D0TargetPlan::approved_contract(), candidate).unwrap();
    let i0 = receipt_gate_crc_valid_frame(2_560, 0x31);
    let d0 = receipt_gate_crc_valid_frame(4_096, 0x72);
    compose_flat7x_au_byte_sequence(&validated, &i0, &d0).unwrap()
}

struct Flat7xReceiptGateFixture {
    sequence: crate::profile::Flat7xAuByteSequenceCandidate,
    metadata: MetadataSourceProposal,
    i0: crate::integrity::programme_owner::IntegrityF12SourceReceipts,
    d0: crate::integrity::programme_owner::IntegrityF12SourceReceipts,
}

fn flat7x_receipt_gate_fixture(
    vector: &Value,
    carrier_block: u8,
    f13_container_bytes: usize,
    d0_transaction_override: Option<&str>,
) -> Flat7xReceiptGateFixture {
    use crate::{
        codec::bitio::FrameBitCursor,
        integrity::{
            keyless_checksum::sha256,
            programme_owner::{
                IntegrityIdentity, IntegrityPayloadEvidence, synthetic_f12_receipts_for_flat7x_test,
            },
        },
        profile::Flat7xSourceGeneration,
        timeline::{CodecSourceIndex, JocDispatchIndex, PublishedAuIndex},
    };

    let pair = &vector["positive_pair"];
    let identity = &pair["integrity_identity"];
    let source_index = CodecSourceIndex::new(identity["source_index"].as_u64().unwrap());
    let dispatch = JocDispatchIndex::new(identity["dispatch"].as_u64().unwrap());
    let au_index = PublishedAuIndex::new(identity["published_au_index"].as_u64().unwrap());
    let transaction_id = identity["transaction_id"].as_str().unwrap();
    let programme_id = identity["programme_id"].as_str().unwrap();
    let attempt = identity["attempt"].as_u64().unwrap();

    let (mut zero_slot, protected) = synthetic_receipt_gate_emdf();
    let mut protected_bytes = protected.bytes().to_vec();
    if f13_container_bytes != zero_slot.len() {
        zero_slot.resize(f13_container_bytes, 0);
        zero_slot[2..4].copy_from_slice(
            &u16::try_from(f13_container_bytes - 4)
                .unwrap()
                .to_be_bytes(),
        );
        protected_bytes.resize(f13_container_bytes, 0);
        protected_bytes[2..4].copy_from_slice(
            &u16::try_from(f13_container_bytes - 4)
                .unwrap()
                .to_be_bytes(),
        );
    }
    let metadata = flat7x_synthetic_metadata_proposal(
        identity,
        &pair["f13_metadata"],
        &zero_slot,
        &protected_bytes,
        carrier_block,
    );

    let generation = pair["f15_candidate"]["i0_source_generation_label"]
        .as_u64()
        .unwrap();
    let sequence = receipt_gate_sequence(generation, 16);
    let bytes = sequence.bytes();
    let i0_frame = &bytes[..2_560];
    let d0_frame = &bytes[2_560..];
    let i0_input = &pair["i0_integrity_receipt"];
    let d0_input = &pair["d0_integrity_receipt"];

    let shared_identity = IntegrityIdentity {
        programme_id: programme_id.to_owned(),
        attempt,
        transaction_id: transaction_id.to_owned(),
        source_index,
        dispatch,
        au_index: Some(au_index),
    };
    let i0_generation = i0_input["integrity_owner_generation"].as_array().unwrap();
    let i0_receipt = synthetic_f12_receipts_for_flat7x_test(
        shared_identity.clone(),
        (
            i0_generation[0].as_u64().unwrap(),
            i0_generation[1].as_u64().unwrap(),
        ),
        i0_frame,
        IntegrityPayloadEvidence::AbsentPublishedPayloadV1,
        None,
        None,
    );

    let mut d0_identity = shared_identity;
    if let Some(transaction) = d0_transaction_override {
        d0_identity.transaction_id = transaction.to_owned();
    }
    let d0_generation = d0_input["integrity_owner_generation"].as_array().unwrap();
    let d0_evidence = IntegrityPayloadEvidence::ProtectedEmdf {
        zero_slot_sha256: protected.input_sha256(),
        protected_sha256: protected.output_sha256(),
        message_sha256: sha256(&zero_slot[4..]),
        primary_crc32: protected.primary_crc32(),
        secondary_crc8: protected.secondary_crc8(),
        primary_range: protected.primary_range(),
        secondary_range: protected.secondary_range(),
        container_bytes: pair["f13_metadata"]["protected_container_bytes"]
            .as_u64()
            .unwrap() as usize,
    };
    let d0_receipt = synthetic_f12_receipts_for_flat7x_test(
        d0_identity,
        (
            d0_generation[0].as_u64().unwrap(),
            d0_generation[1].as_u64().unwrap(),
        ),
        d0_frame,
        d0_evidence,
        Some(FrameBitCursor::new(512)),
        Some(protected.output_sha256()),
    );

    // The target candidate's caller label is sourced from the Wash vector; it
    // remains distinct from each owner generation and the PublishedAuIndex.
    assert_eq!(
        sequence.source_generation(),
        Flat7xSourceGeneration::new(generation)
    );
    Flat7xReceiptGateFixture {
        sequence,
        metadata,
        i0: i0_receipt,
        d0: d0_receipt,
    }
}

fn flat7x_synthetic_metadata_proposal(
    identity: &Value,
    f13: &Value,
    zero_slot_container: &[u8],
    protected_container: &[u8],
    carrier_block: u8,
) -> MetadataSourceProposal {
    let (_before, mut proposal, _) = source_zero_proposal();
    let programme_id =
        crate::transaction::ProgrammeId::new(identity["programme_id"].as_str().unwrap()).unwrap();
    let attempt = crate::transaction::AttemptId::new(identity["attempt"].as_u64().unwrap());
    let transaction_id = crate::transaction::CallbackTransactionId::new(
        identity["transaction_id"].as_str().unwrap(),
    )
    .unwrap();
    let source_index = CodecSourceIndex::new(identity["source_index"].as_u64().unwrap());
    let dispatch = JocDispatchIndex::new(identity["dispatch"].as_u64().unwrap());
    let au_index = PublishedAuIndex::new(identity["published_au_index"].as_u64().unwrap());
    let callback_index = crate::timeline::FrontendCallbackIndex::new(
        u32::try_from(source_index.value() + 2).unwrap(),
    );
    let lineage = crate::transaction::CallbackLineageIdentity::new(
        programme_id.clone(),
        attempt,
        callback_index,
    );

    let id14_queue = proposal.id14_queue_view().unwrap();
    let queued_id14 = id14_queue.slots()[1].as_ref().unwrap();
    let producer = match queued_id14.producer_provenance().unwrap() {
        MetadataQueueProducerProvenance::Id14(provenance) => provenance.clone(),
        MetadataQueueProducerProvenance::MetadataBody(_) => {
            panic!("synthetic F13 proposal retains typed ID14 producer provenance")
        }
    };
    let queue_pop = Id14QueuePopReceipt {
        producer: producer.clone(),
        payload: queued_id14.bytes().to_vec().into_boxed_slice(),
        consumer_programme_id: programme_id.clone(),
        consumer_attempt: attempt,
        consumer_transaction_id: transaction_id.clone(),
        consumer_source_index: source_index,
        consumer_dispatch: dispatch,
        consumer_au_index: Some(au_index),
        disposition: Id14PopDisposition::Published,
    };
    let parsed_container =
        crate::joc::emdf::protected_container::synthetic_receipt_gate_container_for_test(
            protected_container.to_vec(),
            crate::joc::emdf::protected_container::CarrierBlock::new(carrier_block).unwrap(),
            producer,
        );

    proposal.receipt.programme_id = programme_id.clone();
    proposal.receipt.attempt = attempt;
    proposal.receipt.transaction_id = lineage.clone();
    proposal.receipt.consumer_transaction_id = Some(transaction_id.clone());
    proposal.receipt.source_index = source_index;
    proposal.receipt.dispatch = dispatch;
    proposal.receipt.au_index = Some(au_index);
    proposal.receipt.before_generation = f13["metadata_owner_generation_before"].as_u64().unwrap();
    proposal.receipt.proposed_after_generation = f13["metadata_owner_generation_after_candidate"]
        .as_u64()
        .unwrap();
    proposal.receipt.id14_queue_pop = Some(queue_pop.clone());
    proposal.receipt.emdf = Some(zero_slot_container.to_vec());
    proposal.emdf = Some(zero_slot_container.to_vec());
    proposal.consumer_transaction_id = Some(transaction_id.clone());
    proposal.parsed_container = Some(ParsedMetadataSourceContainer {
        parsed_container,
        consumer_programme_id: programme_id,
        consumer_attempt: attempt,
        consumer_transaction_id: transaction_id,
        consumer_source_index: source_index,
        consumer_dispatch: dispatch,
        consumer_au_index: au_index,
        id14_queue_pop: queue_pop,
        queue_pop_receipts: Vec::new(),
    });
    proposal
}

#[test]
fn flat7x_f15_wash_pair_vector_checks_receipt_mechanics_only() {
    let vector = flat7x_receipt_pair_vector();
    let pair = flat7x_receipt_gate_fixture(&vector, 2, 16, None);
    let proposal = crate::publication::Flat7xAuOutputOwner::cold("wash-f15-positive", 1)
        .unwrap()
        .stage_next_access_unit(
            pair.sequence,
            crate::publication::Flat7xAuSourceReceiptInput::new(
                Some(&pair.metadata),
                Some(crate::publication::Flat7xAuIntegrityFrameReceipt::independent_i0(&pair.i0)),
                Some(crate::publication::Flat7xAuIntegrityFrameReceipt::dependent_d0(&pair.d0)),
            ),
        )
        .expect("synthetic Wash pair satisfies the F15 receipt predicates");

    let entry = proposal.ledger_entry();
    assert_eq!(entry.au_index().value(), 0);
    assert_eq!(entry.source_generation().value(), 42);
    assert_eq!(
        entry.metadata_identity(),
        (
            CodecSourceIndex::new(11),
            JocDispatchIndex::new(12),
            PublishedAuIndex::new(0)
        )
    );
    assert_eq!(entry.length(), 6_656);
    assert_eq!(entry.frame_lengths(), (2_560, 4_096));
    assert_eq!(entry.f13_emdf_receipt().0, (4, 5));
    assert_eq!(entry.f13_emdf_receipt().2, 16);
    assert_eq!(entry.integrity_owner_generations(), ((10, 11), (20, 21)));
    assert_eq!(entry.frame_crc_residuals(), (0, 0));
    assert!(!proposal.production_allowed());
}

#[test]
fn flat7x_f15_wash_rejection_vectors_retire_the_whole_attempt() {
    use crate::publication::{
        Flat7xAuAttemptRetirementReason, Flat7xAuIntegrityFrameReceipt, Flat7xAuOutputOwner,
        Flat7xAuOutputOwnerError, Flat7xAuSourceReceiptInput,
    };

    let vector = flat7x_receipt_pair_vector();
    let identity_mismatch =
        flat7x_receipt_gate_fixture(&vector, 2, 16, Some("wash-synthetic-transaction-mismatch"));
    let retirement = Flat7xAuOutputOwner::cold("wash-f15-identity-mismatch", 1)
        .unwrap()
        .stage_next_access_unit(
            identity_mismatch.sequence,
            Flat7xAuSourceReceiptInput::new(
                Some(&identity_mismatch.metadata),
                Some(Flat7xAuIntegrityFrameReceipt::independent_i0(
                    &identity_mismatch.i0,
                )),
                Some(Flat7xAuIntegrityFrameReceipt::dependent_d0(
                    &identity_mismatch.d0,
                )),
            ),
        )
        .unwrap_err();
    assert_eq!(
        retirement.reason(),
        Flat7xAuAttemptRetirementReason::StageRejected(
            Flat7xAuOutputOwnerError::IntegritySourceIdentityMismatch
        )
    );

    let wrong_carrier = flat7x_receipt_gate_fixture(&vector, 1, 16, None);
    let retirement = Flat7xAuOutputOwner::cold("wash-f15-wrong-carrier", 1)
        .unwrap()
        .stage_next_access_unit(
            wrong_carrier.sequence,
            Flat7xAuSourceReceiptInput::new(
                Some(&wrong_carrier.metadata),
                Some(Flat7xAuIntegrityFrameReceipt::independent_i0(
                    &wrong_carrier.i0,
                )),
                Some(Flat7xAuIntegrityFrameReceipt::dependent_d0(
                    &wrong_carrier.d0,
                )),
            ),
        )
        .unwrap_err();
    assert_eq!(
        retirement.reason(),
        Flat7xAuAttemptRetirementReason::StageRejected(
            Flat7xAuOutputOwnerError::MetadataCarrierBlockMismatch
        )
    );

    let oversized = flat7x_receipt_gate_fixture(&vector, 2, 512, None);
    let retirement = Flat7xAuOutputOwner::cold("wash-f15-oversized-emdf", 1)
        .unwrap()
        .stage_next_access_unit(
            oversized.sequence,
            Flat7xAuSourceReceiptInput::new(
                Some(&oversized.metadata),
                Some(Flat7xAuIntegrityFrameReceipt::independent_i0(&oversized.i0)),
                Some(Flat7xAuIntegrityFrameReceipt::dependent_d0(&oversized.d0)),
            ),
        )
        .unwrap_err();
    assert_eq!(
        retirement.reason(),
        Flat7xAuAttemptRetirementReason::StageRejected(
            Flat7xAuOutputOwnerError::MetadataContainerExtentMismatch
        )
    );
}

#[test]
fn cold_state_and_five_source_receipts_match_the_complete_vectors() {
    let vectors = vectors();
    let mut owner = owner();
    let programme_configuration = owner.id1_configuration.clone();
    assert_eq!(
        bytes_hex(&owner.state.canonical_bytes()),
        vectors["cold"]["canonical_hex"]
    );
    assert_eq!(owner.state.sha256_hex(), vectors["cold"]["sha256"]);

    for call in vectors["continuous_calls"].as_array().unwrap() {
        let result = owner.process_source(source_call(call)).unwrap();
        assert_eq!(owner.id1_configuration, programme_configuration);
        assert_eq!(result.receipt.as_json(), call["expected"]["receipt"]);
        assert_eq!(
            bytes_hex(&owner.state.canonical_bytes()),
            call["expected"]["state_canonical_hex"]
        );
        assert_eq!(owner.state.sha256_hex(), call["expected"]["after_sha256"]);
        assert_eq!(
            result.emdf.as_ref().map(|bytes| hex_encode(bytes)),
            call["expected"]["receipt"]["emdf_hex"]
                .as_str()
                .map(str::to_owned)
        );
    }
    let restarted = owner_for_attempt(1);
    assert_eq!(
        restarted.id1_configuration.attempt(),
        crate::transaction::AttemptId::new(1)
    );
    assert_eq!(
        bytes_hex(&restarted.state.canonical_bytes()),
        vectors["cold"]["canonical_hex"]
    );
    let first_published: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/metadata_owner/EMDF-TERMINAL-FIXTURES.json"
    ))
    .unwrap();
    assert_eq!(
        vectors["continuous_calls"][2]["expected"]["receipt"]["emdf_hex"],
        first_published["first_published"]["wire_hex"]
    );
    assert_eq!(
        vectors["continuous_calls"][2]["expected"]["receipt"]["emdf_sha256"],
        first_published["first_published"]["wire_sha256"]
    );
}

#[test]
fn id14_producer_provenance_survives_fifo_and_source_one_is_suppressed() {
    let vectors = vectors();
    let calls = vectors["continuous_calls"].as_array().unwrap();
    let spatial_digest =
        crate::spatial::SpatialAudioMachine::new_cold(crate::profile::EncodingProfile::selected())
            .unwrap()
            .digest();
    let mut owner = owner();

    let mut source_zero = source_call(&calls[0]);
    bind_test_id14_provenance(&mut source_zero, spatial_digest);
    let source_zero_result = owner.process_source(source_zero).unwrap();
    assert!(source_zero_result.receipt.id14_queue_pop().is_none());

    let mut source_one = source_call(&calls[1]);
    bind_test_id14_provenance(&mut source_one, spatial_digest);
    let source_one_result = owner.process_source(source_one).unwrap();
    let suppressed = source_one_result
        .receipt
        .id14_queue_pop()
        .expect("source one consumes writer zero");
    assert_eq!(suppressed.producer().producer_generation().value(), 0);
    assert_eq!(suppressed.producer().lineage().source_index().value(), 0);
    assert_eq!(suppressed.consumer_source_index().value(), 1);
    assert_eq!(
        suppressed
            .producer()
            .lineage()
            .transaction_id()
            .callback_index()
            .value(),
        2
    );
    assert_eq!(
        suppressed.consumer_transaction_id().as_str(),
        "metadata-test-source-1"
    );
    assert_eq!(
        suppressed
            .producer()
            .producer_transaction_id()
            .unwrap()
            .as_str(),
        "metadata-test-source-0"
    );
    assert_eq!(suppressed.disposition(), Id14PopDisposition::Suppressed);
    assert_eq!(source_one_result.receipt.queue_pop_receipts().len(), 1);
    assert_eq!(
        source_one_result.receipt.queue_pop_receipts()[0].queue_id(),
        14
    );
    assert_eq!(source_one_result.receipt.as_json()["published"], false);
    let source_zero_payload = calls[0]["input"]["bodies"]["14"]["payload_hex"]
        .as_str()
        .map(hex_bytes)
        .unwrap();
    assert_eq!(suppressed.payload(), source_zero_payload.as_slice());

    let mut source_two = source_call(&calls[2]);
    bind_test_id14_provenance(&mut source_two, spatial_digest);
    let source_two_result = owner.process_source(source_two).unwrap();
    let published = source_two_result
        .receipt
        .id14_queue_pop()
        .expect("source two consumes writer one");
    assert_eq!(published.producer().producer_generation().value(), 1);
    assert_eq!(published.producer().lineage().source_index().value(), 1);
    assert_eq!(published.consumer_source_index().value(), 2);
    assert_eq!(
        published.consumer_au_index(),
        Some(PublishedAuIndex::new(0))
    );
    assert_eq!(
        published
            .producer()
            .lineage()
            .transaction_id()
            .callback_index()
            .value(),
        3
    );
    assert_eq!(
        published.consumer_transaction_id().as_str(),
        "metadata-test-source-2"
    );
    assert_eq!(
        published
            .producer()
            .producer_transaction_id()
            .unwrap()
            .as_str(),
        "metadata-test-source-1"
    );
    assert_eq!(published.disposition(), Id14PopDisposition::Published);
    let source_one_payload = calls[1]["input"]["bodies"]["14"]["payload_hex"]
        .as_str()
        .map(hex_bytes)
        .unwrap();
    assert_eq!(published.payload(), source_one_payload.as_slice());
    assert_eq!(
        source_two_result
            .receipt
            .queue_pop_receipts()
            .iter()
            .map(MetadataQueuePopReceipt::queue_id)
            .collect::<Vec<_>>(),
        vec![11, 14, 2, 1]
    );
    assert!(source_two_result.receipt.as_json()["id14_queue_pop"]["producer"].is_object());
    for queue_id in [11_u8, 1, 2] {
        let provenance = source_two_result
            .receipt
            .queue_pop_provenance(queue_id)
            .expect("ordinary queue pop retains producer provenance");
        let MetadataQueueProducerProvenance::MetadataBody(provenance) = provenance else {
            panic!("ordinary metadata queue uses metadata producer provenance");
        };
        assert_eq!(provenance.producer_generation().value(), 0);
        assert_eq!(provenance.writer().value(), 0);
        assert_eq!(
            provenance.producer_transaction_id().as_str(),
            "metadata-test-source-0"
        );
    }
    for operation in source_two_result.receipt.as_json()["queue_operations"]
        .as_array()
        .unwrap()
    {
        assert_eq!(
            operation["consumer_transaction_id"],
            "metadata-test-source-2"
        );
        assert_eq!(operation["consumer_source_index"], 2);
        assert_eq!(operation["consumer_dispatch"], 3);
    }
    assert!(
        owner
            .state
            .canonical_bytes()
            .windows(b"MetadataQueueProvenanceV1\0".len())
            .any(|window| window == b"MetadataQueueProvenanceV1\0")
    );
}

#[test]
fn id14_same_source_barrier_rejects_a_foreign_producer_transaction() {
    let vectors = vectors();
    let calls = vectors["continuous_calls"].as_array().unwrap();
    let spatial_digest =
        crate::spatial::SpatialAudioMachine::new_cold(crate::profile::EncodingProfile::selected())
            .unwrap()
            .digest();

    let mut current_source = source_call(&calls[0]);
    bind_test_id14_provenance(&mut current_source, spatial_digest);
    let current_provenance = current_source.bodies.id14_provenance.as_ref().unwrap();
    let body = current_source.bodies.id14.as_ref().unwrap();
    current_source.bodies.id14_provenance = Some(
        crate::joc::id14_body::Id14ProducerProvenance::new(
            crate::joc::id14_body::Id14ProducerIdentity {
                lineage: current_provenance.lineage().clone(),
                producer_transaction_id: Some(
                    crate::transaction::CallbackTransactionId::new("foreign-producer-token")
                        .unwrap(),
                ),
                producer_generation: current_provenance.producer_generation(),
                dispatch: current_provenance.dispatch(),
                writer: current_provenance.writer(),
                sequence_count: crate::joc::id14_body::Id14SequenceCount::new(
                    current_provenance.sequence_count(),
                )
                .unwrap(),
                object_count: current_provenance.object_count(),
            },
            &body.bytes,
        )
        .unwrap(),
    );
    let error = owner().process_source(current_source).unwrap_err();
    assert_eq!(error, MetadataOwnerError::Id14Provenance);
}

#[test]
fn metadata_stage_binds_the_current_parent_transaction_to_id14_producer() {
    let vectors = vectors();
    let call = &vectors["continuous_calls"][0];
    let spatial_digest =
        crate::spatial::SpatialAudioMachine::new_cold(crate::profile::EncodingProfile::selected())
            .unwrap()
            .digest();
    let mut source = source_call(call);
    bind_test_id14_provenance(&mut source, spatial_digest);
    let producer = source.bodies.id14_provenance.clone().unwrap();
    let producer_transaction_id = producer.producer_transaction_id().unwrap().clone();
    let bodies = MetadataBodiesProposal::for_test_with_id14_provenance(
        source.programme_id.clone(),
        source.attempt,
        source.source_lineage.clone(),
        source.source_index,
        source.bodies.clone(),
        producer,
    );

    let proposal = owner()
        .stage_source(MetadataStageInput {
            au_index: source.au_index,
            transaction_id: producer_transaction_id.clone(),
            bodies: bodies.clone(),
        })
        .unwrap();
    assert_eq!(
        proposal.consumer_transaction_id(),
        Some(&producer_transaction_id)
    );
    assert!(proposal.parsed_container().is_none());
    let id14_queue = proposal.id14_queue_view().unwrap();
    let queued = id14_queue.slots()[1].as_ref().unwrap();
    assert_eq!(queued.writer_generation().value(), 0);
    assert!(matches!(
        queued.producer_provenance(),
        Some(MetadataQueueProducerProvenance::Id14(_))
    ));

    let mismatch =
        crate::transaction::CallbackTransactionId::new("different-source-transaction").unwrap();
    let Err(error) = owner().stage_source(MetadataStageInput {
        au_index: source.au_index,
        transaction_id: mismatch,
        bodies,
    }) else {
        panic!("mismatched parent token must not stage metadata");
    };
    assert_eq!(error, MetadataOwnerError::Id14Provenance);
}

fn source_zero_proposal() -> (MetadataOwner, MetadataSourceProposal, CallbackTransactionId) {
    let vectors = vectors();
    let source = source_call(&vectors["continuous_calls"][0]);
    let spatial_digest =
        crate::spatial::SpatialAudioMachine::new_cold(crate::profile::EncodingProfile::selected())
            .unwrap()
            .digest();
    let mut source_with_provenance = source.clone();
    bind_test_id14_provenance(&mut source_with_provenance, spatial_digest);
    let producer = source_with_provenance
        .bodies
        .id14_provenance
        .clone()
        .unwrap();
    let transaction_id = producer.producer_transaction_id().unwrap().clone();
    let bodies = MetadataBodiesProposal::for_test_with_id14_provenance(
        source.programme_id.clone(),
        source.attempt,
        source.source_lineage.clone(),
        source.source_index,
        source_with_provenance.bodies,
        producer,
    );
    let before = owner();
    let proposal = before
        .stage_source(MetadataStageInput {
            au_index: source.au_index,
            transaction_id: transaction_id.clone(),
            bodies,
        })
        .unwrap();
    (before, proposal, transaction_id)
}

#[test]
fn local_receipt_projection_binds_source_transaction_and_exact_roots() {
    let (before, proposal, transaction_id) = source_zero_proposal();
    let receipt = proposal
        .gtc_owner_receipt(&before, transaction_id.clone())
        .unwrap();
    assert_eq!(receipt.owner_id(), crate::transaction::GtcOwnerId::Metadata);
    assert_eq!(receipt.before_generation(), 0);
    assert_eq!(receipt.after_generation(), 1);
    assert!(receipt.changed());
    assert_eq!(receipt.commit_count(), 1);
    assert_eq!(receipt.transaction_id(), &transaction_id);

    let wrong_transaction = CallbackTransactionId::new("different-parent-transaction").unwrap();
    assert_eq!(
        proposal.gtc_owner_receipt(&before, wrong_transaction),
        Err(MetadataGtcReceiptError::ParentTransactionMismatch)
    );

    let (_, proposal, transaction_id) = source_zero_proposal();
    let mut wrong_before = owner();
    wrong_before.state.generation = 1;
    assert_eq!(
        proposal.gtc_owner_receipt(&wrong_before, transaction_id),
        Err(MetadataGtcReceiptError::BeforeStateMismatch)
    );
}

#[test]
fn source_proposal_transfers_exact_after_owner_and_canonical_receipts_to_parent() {
    let (before, proposal, transaction_id) = source_zero_proposal();
    let before_state = before.state_reference();
    let handoff = proposal
        .into_parent_handoff(&before, transaction_id.clone())
        .unwrap();

    assert_eq!(
        handoff.owner_receipt().owner_id(),
        crate::transaction::GtcOwnerId::Metadata
    );
    assert_eq!(handoff.owner_receipt().transaction_id(), &transaction_id);
    assert_eq!(handoff.owner_receipt().before_generation(), 0);
    assert_eq!(handoff.owner_receipt().after_generation(), 1);
    assert_eq!(handoff.before_state(), &before_state);
    assert_eq!(handoff.before_state().generation(), 0);
    assert_eq!(handoff.after_state().generation(), 1);
    assert_eq!(
        handoff.after_owner().state_reference(),
        handoff.after_state().clone()
    );
    assert_eq!(before.state_reference(), before_state);

    let (after_owner, source_receipt, owner_receipt, before_state, after_state, _, _, _) =
        handoff.into_parts();
    assert_eq!(source_receipt.source_index().value(), 0);
    assert_eq!(owner_receipt.after_generation(), after_state.generation());
    assert_eq!(before_state.generation(), 0);
    assert_eq!(after_owner.state_reference(), after_state);
}

#[test]
fn source_proposal_uses_consumer_token_and_parent_callback_receipt_separately() {
    let (before, proposal, consumer_transaction) = source_zero_proposal();
    let parent_callback_transaction =
        CallbackTransactionId::new("jocforge-main-callback-g2").unwrap();

    let handoff = proposal
        .into_parent_handoff_for_parent_callback(
            &before,
            &consumer_transaction,
            parent_callback_transaction.clone(),
        )
        .unwrap();

    assert_eq!(
        handoff.source_receipt().consumer_transaction_id(),
        Some(&consumer_transaction)
    );
    assert_eq!(
        handoff.owner_receipt().transaction_id(),
        &parent_callback_transaction
    );
    assert_eq!(handoff.owner_receipt().before_generation(), 0);
    assert_eq!(handoff.owner_receipt().after_generation(), 1);
}

#[test]
fn source_proposal_rejects_wrong_consumer_before_issuing_parent_receipt() {
    let (before, proposal, _) = source_zero_proposal();
    let wrong_consumer = CallbackTransactionId::new("jocforge-source-1").unwrap();
    let parent_callback_transaction =
        CallbackTransactionId::new("jocforge-main-callback-g2").unwrap();

    assert_eq!(
        proposal.gtc_owner_receipt_for_parent_callback(
            &before,
            &wrong_consumer,
            parent_callback_transaction,
        ),
        Err(MetadataGtcReceiptError::ParentTransactionMismatch)
    );
    assert_eq!(before.state_reference().generation(), 0);
}

#[test]
fn local_receipt_projection_allows_only_exact_cold_priming_clocks() {
    let cold = owner();
    for callback in 0..=1 {
        let transaction_id =
            CallbackTransactionId::new(&format!("priming-parent-{callback}")).unwrap();
        let receipt = cold
            .gtc_unchanged_callback_receipt(
                FrontendCallbackIndex::new(callback),
                transaction_id.clone(),
            )
            .unwrap();
        assert_eq!(receipt.owner_id(), crate::transaction::GtcOwnerId::Metadata);
        assert_eq!(receipt.before_generation(), 0);
        assert_eq!(receipt.after_generation(), 0);
        assert!(!receipt.changed());
        assert_eq!(receipt.commit_count(), 0);
        assert_eq!(receipt.transaction_id(), &transaction_id);
    }

    assert_eq!(
        cold.gtc_unchanged_callback_receipt(
            FrontendCallbackIndex::new(2),
            CallbackTransactionId::new("source-parent-transaction").unwrap(),
        ),
        Err(MetadataGtcReceiptError::CallbackHasSourceStage)
    );

    let (_, proposal, _) = source_zero_proposal();
    let non_cold = proposal.into_after_owner();
    assert_eq!(
        non_cold.gtc_unchanged_callback_receipt(
            FrontendCallbackIndex::new(0),
            CallbackTransactionId::new("priming-parent-transaction").unwrap(),
        ),
        Err(MetadataGtcReceiptError::CallbackHasUnexpectedSource)
    );
}

#[test]
fn cold_state_reference_carries_the_complete_metadata_state_v1_value() {
    let vectors = vectors();
    let owner = owner();
    let committed_before = owner.state.canonical_bytes();
    let reference = owner.state_reference();

    assert_eq!(reference.owner_id(), "metadata");
    assert_eq!(reference.state_ref(), "owners.metadata");
    assert_eq!(reference.generation(), 0);
    assert_eq!(reference.canonical_bytes(), committed_before);
    assert_eq!(
        hex_encode(reference.canonical_bytes()),
        vectors["cold"]["canonical_hex"]
    );
    assert_eq!(
        hex_encode(reference.state_sha256()),
        vectors["cold"]["sha256"]
    );
    assert_eq!(reference.state_sha256(), reference.cold_sha256());
    assert_eq!(owner.state.canonical_bytes(), committed_before);
}

#[test]
fn staged_state_reference_carries_full_after_bytes_without_installing_them() {
    let vectors = vectors();
    let owner = owner();
    let cold_reference = owner.state_reference();
    let case = &vectors["continuous_calls"][0];
    let call = source_call(case);
    let bodies = MetadataBodiesProposal::for_test(
        call.programme_id.clone(),
        call.attempt,
        call.source_lineage.clone(),
        call.source_index,
        call.bodies,
    );
    let proposal = owner
        .stage_source(MetadataStageInput {
            au_index: call.au_index,
            transaction_id: test_callback_transaction_id(call.source_index),
            bodies,
        })
        .expect("source proposal computes its complete metadata after-state");
    let after_reference = proposal.after_owner.state_reference();

    assert_eq!(after_reference.owner_id(), "metadata");
    assert_eq!(after_reference.state_ref(), "owners.metadata");
    assert_eq!(after_reference.generation(), 1);
    assert_eq!(
        hex_encode(after_reference.canonical_bytes()),
        case["expected"]["state_canonical_hex"]
    );
    assert_eq!(
        hex_encode(after_reference.state_sha256()),
        case["expected"]["after_sha256"]
    );
    assert_eq!(after_reference.cold_sha256(), cold_reference.cold_sha256());
    assert_eq!(
        owner.state.canonical_bytes(),
        cold_reference.canonical_bytes()
    );
}

#[test]
fn typed_stage_uses_pinned_wrapper_and_leaves_committed_leaf_unchanged() {
    let vectors = vectors();
    let case = &vectors["continuous_calls"][0];
    let call = source_call(case);
    let before = owner();
    assert_eq!(
        Value::Array(before.id1_configuration.json_values()),
        case["input"]["config"]
    );
    let before_bytes = before.state.canonical_bytes();
    let bodies = MetadataBodiesProposal::for_test(
        call.programme_id.clone(),
        call.attempt,
        call.source_lineage.clone(),
        call.source_index,
        call.bodies,
    );
    let proposal = before
        .stage_source(MetadataStageInput {
            au_index: call.au_index,
            transaction_id: test_callback_transaction_id(call.source_index),
            bodies,
        })
        .unwrap();
    assert_eq!(before.state.canonical_bytes(), before_bytes);
    assert_eq!(proposal.receipt().as_json(), case["expected"]["receipt"]);
    assert_eq!(
        proposal.proposed_after_sha256(),
        case["expected"]["after_sha256"]
    );
    assert_eq!(
        proposal.bodies_lineage().transaction_id(),
        &call.source_lineage
    );
    assert_eq!(proposal.before_snapshot().generation(), 0);
    assert_eq!(proposal.before_snapshot().next_source_index().value(), 0);
    assert_eq!(proposal.after_snapshot().generation(), 1);
    assert_eq!(proposal.after_snapshot().next_source_index().value(), 1);
    assert_ne!(
        proposal.before_snapshot().state_sha256(),
        proposal.after_snapshot().state_sha256()
    );
    let id14 = proposal.id14_queue_view().unwrap();
    assert_eq!(id14.buffer_ids(), &[1, 0]);
    assert_eq!(id14.next_writer().value(), 1);
    assert!(id14.slots()[0].is_none());
    let writer0 = id14.slots()[1].as_ref().unwrap();
    assert_eq!(writer0.writer_generation().value(), 0);
    assert_eq!(writer0.bytes(), &[0x0e, 0x00, 0x0e]);
}

#[test]
fn typed_metadata_programme_reuses_one_public_kind_configuration_until_eof() {
    let vectors = vectors();
    let root = fixture_root();
    let programme_configuration = id1_configuration_for_kind(0, "LEVEL");
    assert_eq!(programme_configuration.values()[2], 0);
    let mut owner = MetadataOwner::new(
        programme_configuration.clone(),
        5,
        &std::fs::read(root.join("metadata-public-provider.json")).unwrap(),
        &std::fs::read(root.join("metadata-wrapper-config.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        bytes_hex(&owner.state.canonical_bytes()),
        vectors["cold"]["canonical_hex"]
    );

    for case in vectors["continuous_calls"].as_array().unwrap() {
        let call = source_call(case);
        let bodies = MetadataBodiesProposal::for_test(
            call.programme_id.clone(),
            call.attempt,
            call.source_lineage.clone(),
            call.source_index,
            call.bodies,
        );
        let proposal = owner
            .stage_source(MetadataStageInput {
                au_index: call.au_index,
                transaction_id: test_callback_transaction_id(call.source_index),
                bodies,
            })
            .unwrap();
        assert_eq!(
            proposal.after_owner.id1_configuration,
            programme_configuration
        );
        owner = proposal.into_after_owner();
        assert_eq!(owner.id1_configuration, programme_configuration);
    }

    let eof = owner.prepare_eof(5).unwrap();
    assert_eq!(owner.id1_configuration, programme_configuration);
    assert_eq!(eof.as_json()["timer_advance_count"], 0);
    assert_eq!(eof.as_json()["push_count"], 0);
}

#[test]
fn published_metadata_proposal_stages_into_the_selected_carrier() {
    let vectors = vectors();
    let calls = vectors["continuous_calls"].as_array().unwrap();
    let mut metadata = owner();
    for call in &calls[..2] {
        metadata.process_source(source_call(call)).unwrap();
    }
    let call = source_call(&calls[2]);
    let bodies = MetadataBodiesProposal::for_test(
        call.programme_id.clone(),
        call.attempt,
        call.source_lineage.clone(),
        call.source_index,
        call.bodies,
    );
    let metadata_proposal = metadata
        .stage_source(MetadataStageInput {
            au_index: call.au_index,
            transaction_id: test_callback_transaction_id(call.source_index),
            bodies,
        })
        .unwrap();
    let emdf = metadata_proposal.emdf().expect("source2 publishes EMDF");
    let carrier = stage_metadata_carrier(
        &CarrierPlannerState::cold(),
        emdf,
        &[0xa5; 4_096],
        CarrierBitCursor::new(10_048),
    )
    .unwrap();
    assert_eq!(carrier.complete_container(), emdf);
    assert_eq!(
        carrier.plan().segment_lengths()[2],
        u16::try_from(emdf.len() - 4).unwrap()
    );
    assert_eq!(carrier.payload_cursor().bit_offset(), 10_058);
}

#[test]
fn every_failure_vector_is_atomic_and_retires_only_the_attempt() {
    let vectors = vectors();
    for case in vectors["failure_cases"].as_array().unwrap() {
        let mut owner = owner();
        let id = case["id"].as_str().unwrap();

        if id == "occupied_destination" {
            let prestate = &case["prestate"]["state"];
            let occupied = &prestate["queues"]["11"]["slots"][0];
            owner.state.queues.get_mut(&11).unwrap().slots[0] = Some(QueuedBody {
                writer_generation: CodecSourceIndex::new(
                    occupied["writer_generation"].as_u64().unwrap(),
                ),
                payload: hex_bytes(occupied["payload_hex"].as_str().unwrap()),
                producer_provenance: None,
            });
            assert_eq!(
                bytes_hex(&owner.state.canonical_bytes()),
                case["prestate"]["canonical_hex"]
            );
        } else if id == "post_final_source" {
            for call in vectors["continuous_calls"].as_array().unwrap() {
                owner.process_source(source_call(call)).unwrap();
            }
            let eof = owner.prepare_eof(5).unwrap();
            owner.state = eof.after_state;
        }

        let before = owner.state.canonical_bytes();
        assert_eq!(owner.state.sha256_hex(), case["expected"]["before_sha256"]);
        let error = owner.process_source(source_call(case)).unwrap_err();
        assert_eq!(error.code(), case["expected"]["error"], "{id}");
        assert_eq!(owner.state.canonical_bytes(), before, "{id}");
        assert!(owner.is_retired(), "{id}");
        assert_eq!(case["expected"]["commit_count"], 0, "{id}");
        assert_eq!(case["expected"]["receipt"], Value::Null, "{id}");
        assert_eq!(owner.state.sha256_hex(), case["expected"]["after_sha256"]);
    }
}

#[test]
fn eof_is_a_leaf_proposal_with_no_source_receipt_or_flush() {
    let vectors = vectors();
    let mut owner = owner();
    let programme_configuration = owner.id1_configuration.clone();
    for call in vectors["continuous_calls"].as_array().unwrap() {
        owner.process_source(source_call(call)).unwrap();
        assert_eq!(owner.id1_configuration, programme_configuration);
    }

    let eof = owner.prepare_eof(5).unwrap();
    assert_eq!(owner.id1_configuration, programme_configuration);
    assert_eq!(eof.programme_id(), &owner.programme_id);
    assert_eq!(eof.attempt(), owner.attempt);
    assert_eq!(
        eof.as_json(),
        json!({
            "expected_sources": vectors["eof"]["expected_sources"],
            "before": vectors["eof"]["before"],
            "after": vectors["eof"]["after"],
            "after_canonical_hex": vectors["eof"]["after_canonical_hex"],
            "after_sha256": vectors["eof"]["after_sha256"],
            "push_count": 0,
            "pop_count": 0,
            "timer_advance_count": 0,
            "extra_outputs": 0,
        })
    );
    assert!(!eof.has_source_receipt());
    assert_eq!(eof.expected_sources(), 5);
    assert!(eof.after_is_finalized());
    assert!(eof.queues_are_empty());
    assert!(eof.pending_writers_are_drop_at_done());
    let mut pending = eof
        .pending_writer_dispositions()
        .into_iter()
        .map(|(queue_id, writer, disposition)| (queue_id, writer.value(), disposition))
        .collect::<Vec<_>>();
    pending.sort_by_key(|(queue_id, writer, disposition)| (*queue_id, *writer, *disposition));
    assert_eq!(
        pending,
        vec![
            (1, 3, "DROP_AT_DONE"),
            (1, 4, "DROP_AT_DONE"),
            (2, 3, "DROP_AT_DONE"),
            (2, 4, "DROP_AT_DONE"),
            (11, 3, "DROP_AT_DONE"),
            (11, 4, "DROP_AT_DONE"),
            (14, 4, "DROP_AT_DONE"),
        ]
    );
    assert_eq!(
        owner.state.sha256_hex(),
        vectors["continuous_calls"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["expected"]["after_sha256"]
    );
}

#[test]
fn eof_parent_handoff_returns_an_uninstalled_after_root_and_full_state_receipt() {
    let mut before = owner_after_all_sources();
    let before_state = before.state_reference();
    let before_bytes = before.state.canonical_bytes();
    let proposal = before.prepare_eof(5).unwrap();
    let transaction_id =
        crate::transaction::CallbackTransactionId::new("metadata-eof-source-5").unwrap();

    let handoff = proposal
        .into_parent_handoff(&before, transaction_id.clone())
        .unwrap();
    assert_eq!(handoff.before_state(), &before_state);
    assert_eq!(handoff.owner_receipt().owner_id(), GtcOwnerId::Metadata);
    assert_eq!(handoff.owner_receipt().transaction_id(), &transaction_id);
    assert_eq!(
        handoff.owner_receipt().before_generation(),
        before_state.generation()
    );
    assert_eq!(
        handoff.owner_receipt().before_sha256(),
        *before_state.state_sha256()
    );
    assert_eq!(
        handoff.owner_receipt().after_generation(),
        before_state.generation() + 1
    );
    assert_eq!(
        handoff.owner_receipt().after_sha256(),
        *handoff.after_state().state_sha256()
    );
    assert_eq!(handoff.after_state().owner_id(), "metadata");
    assert_eq!(handoff.after_state().state_ref(), "owners.metadata");
    assert!(handoff.after_owner().state.finalized);
    assert_eq!(handoff.after_owner().state.next_source_index, 5);
    assert!(
        handoff
            .after_owner()
            .state
            .queues
            .values()
            .all(|queue| { queue.slots.iter().all(Option::is_none) })
    );
    assert!(
        handoff
            .after_owner()
            .state
            .pending
            .iter()
            .all(|pending| pending.disposition == "DROP_AT_DONE")
    );
    let expected_receipt = handoff.owner_receipt().clone();

    assert_eq!(before.state.canonical_bytes(), before_bytes);
    assert_eq!(before.state_reference(), before_state);
    assert!(!before.state.finalized);

    let (after, receipt) = handoff.into_parts();
    assert!(after.state.finalized);
    assert_eq!(receipt, expected_receipt);
}

#[test]
fn eof_parent_handoff_rejects_identity_clock_and_terminal_state_mismatches() {
    let transaction_id =
        crate::transaction::CallbackTransactionId::new("metadata-eof-invalid").unwrap();

    let (mut before, proposal) = prepared_eof();
    before.programme_id = crate::transaction::test_programme_id("other-programme");
    assert!(matches!(
        proposal.into_parent_handoff(&before, transaction_id.clone()),
        Err(MetadataEofHandoffError::ProgrammeMismatch)
    ));

    let (mut before, proposal) = prepared_eof();
    before.attempt = crate::transaction::AttemptId::new(1);
    assert!(matches!(
        proposal.into_parent_handoff(&before, transaction_id.clone()),
        Err(MetadataEofHandoffError::AttemptMismatch)
    ));

    let (mut before, proposal) = prepared_eof();
    before.state.generation += 1;
    assert!(matches!(
        proposal.into_parent_handoff(&before, transaction_id.clone()),
        Err(MetadataEofHandoffError::BeforeStateMismatch)
    ));

    let (before, mut proposal) = prepared_eof();
    proposal.expected_sources -= 1;
    assert!(matches!(
        proposal.into_parent_handoff(&before, transaction_id.clone()),
        Err(MetadataEofHandoffError::BeforeSourceCountMismatch)
    ));

    let (before, mut proposal) = prepared_eof();
    proposal.after_state.next_source_index -= 1;
    assert!(matches!(
        proposal.into_parent_handoff(&before, transaction_id.clone()),
        Err(MetadataEofHandoffError::AfterSourceCountMismatch)
    ));

    let (before, mut proposal) = prepared_eof();
    proposal.after_state.generation -= 1;
    assert!(matches!(
        proposal.into_parent_handoff(&before, transaction_id.clone()),
        Err(MetadataEofHandoffError::AfterGenerationMismatch)
    ));

    let (before, mut proposal) = prepared_eof();
    proposal.after_state.finalized = false;
    assert!(matches!(
        proposal.into_parent_handoff(&before, transaction_id.clone()),
        Err(MetadataEofHandoffError::AfterStateNotFinalized)
    ));

    let (before, mut proposal) = prepared_eof();
    proposal.after_state.queues.get_mut(&11).unwrap().slots[0] = Some(QueuedBody {
        writer_generation: CodecSourceIndex::new(5),
        payload: vec![0x01],
        producer_provenance: None,
    });
    assert!(matches!(
        proposal.into_parent_handoff(&before, transaction_id.clone()),
        Err(MetadataEofHandoffError::AfterQueuesNotEmpty)
    ));

    let (before, mut proposal) = prepared_eof();
    proposal.after_state.pending[0].disposition = "RETAIN";
    assert!(matches!(
        proposal.into_parent_handoff(&before, transaction_id),
        Err(MetadataEofHandoffError::PendingDispositionMismatch)
    ));
}

#[test]
fn cold_terminal_layout_matches_the_independent_bit_position_fixture() {
    let root = fixture_root();
    let wrapper: Value =
        serde_json::from_slice(&std::fs::read(root.join("metadata-wrapper-config.json")).unwrap())
            .unwrap();
    let fixtures: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/metadata_owner/EMDF-TERMINAL-FIXTURES.json"
    ))
    .unwrap();
    let expected = &fixtures["cold_empty"];
    let output = emdf_serializer::serialize(&wrapper, &[]).unwrap();
    assert_eq!(hex_encode(&output.bytes), expected["wire_hex"]);
    assert_eq!(
        emdf_serializer::sha256_hex(&output.bytes),
        expected["wire_sha256"]
    );
    assert_eq!(
        output.terminal_layout["terminal_id"]["start_bit"],
        expected["trace"]["terminal_id_start_bit"]
    );
    assert_eq!(
        output.terminal_layout["primary_length_code"]["start_bit"],
        expected["trace"]["primary_code_start_bit"]
    );
    assert_eq!(
        output.terminal_layout["secondary_length_code"]["start_bit"],
        expected["trace"]["secondary_code_start_bit"]
    );
    assert_eq!(
        output.terminal_layout["primary_protection_range_bits"],
        expected["trace"]["primary_protection_range_bits"]
    );
    assert_eq!(
        output.terminal_layout["secondary_protection_range_bits"],
        expected["trace"]["secondary_protection_range_bits"]
    );
    assert_eq!(
        output.terminal_layout["terminal_padding_range_bits"],
        expected["trace"]["padding_range_bits"]
    );
    let independent: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/metadata_owner/EMDF-TERMINAL-INDEPENDENT-CHECK.json"
    ))
    .unwrap();
    assert!(independent["all_pass"].as_bool().unwrap());
    assert_eq!(
        independent["checks"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|check| check["pass"] == Value::Bool(true))
            .count(),
        4
    );
}

#[test]
fn terminal_container_rejects_total_wire_length_above_the_carrier_limit() {
    let root = fixture_root();
    let wrapper: Value =
        serde_json::from_slice(&std::fs::read(root.join("metadata-wrapper-config.json")).unwrap())
            .unwrap();
    let bodies = [
        (11, vec![0; 160]),
        (14, vec![0; 543]),
        (2, vec![0; 8]),
        (1, vec![0; 10]),
    ];
    let payloads = bodies
        .iter()
        .map(|(id, payload)| {
            (
                *id,
                QueuedBody {
                    writer_generation: CodecSourceIndex::new(0),
                    payload: payload.clone(),
                    producer_provenance: None,
                },
            )
        })
        .collect::<Vec<_>>();
    let payloads = payloads
        .iter()
        .map(|(id, body)| (*id, body))
        .collect::<Vec<_>>();
    assert!(matches!(
        emdf_serializer::serialize(&wrapper, &payloads),
        Err(MetadataOwnerError::EmdfSerialization)
    ));
}

#[test]
fn cold_reset_and_retry_are_pinned_without_extra_clock_advances() {
    let vectors = vectors();
    let mut retired_owner = owner();
    retired_owner.prepare_eof(0).unwrap_err();
    let retry_configuration = id1_configuration(1);
    assert_eq!(
        retry_configuration.programme_id(),
        &retired_owner.programme_id
    );
    assert_eq!(
        retry_configuration.attempt(),
        crate::transaction::AttemptId::new(1)
    );
    let retired = retired_owner
        .reset_for_attempt(crate::transaction::AttemptId::new(1), retry_configuration)
        .unwrap();
    assert_eq!(
        bytes_hex(&retired.state.canonical_bytes()),
        vectors["cold"]["canonical_hex"]
    );
    assert_eq!(retired.state.sha256_hex(), vectors["cold"]["sha256"]);
    assert_eq!(
        retired.id1_configuration.attempt(),
        crate::transaction::AttemptId::new(1)
    );
    let mut retry = owner();
    let result = retry
        .process_source(source_call(&vectors["reset_retry"]))
        .unwrap();
    assert_eq!(
        result.receipt.as_json(),
        vectors["reset_retry"]["expected"]["receipt"]
    );
}

#[test]
fn eof_source_count_failure_retires_without_changing_committed_state() {
    let mut owner = owner();
    let programme_configuration = owner.id1_configuration.clone();
    let before = owner.state.canonical_bytes();
    let error = owner.prepare_eof(0).unwrap_err();
    assert_eq!(error.code(), "METADATA_EOF_SOURCE_COUNT");
    assert_eq!(owner.state.canonical_bytes(), before);
    assert_eq!(owner.id1_configuration, programme_configuration);
    assert!(owner.is_retired());
}

#[test]
fn provider_and_wrapper_identity_are_checked_before_use() {
    let root = fixture_root();
    let mut provider = std::fs::read(root.join("metadata-public-provider.json")).unwrap();
    provider[0] ^= 1;
    let wrapper = std::fs::read(root.join("metadata-wrapper-config.json")).unwrap();
    assert!(matches!(
        MetadataOwner::new(id1_configuration(0), 5, &provider, &wrapper,),
        Err(MetadataOwnerError::ProviderIdentity)
    ));

    let provider = std::fs::read(root.join("metadata-public-provider.json")).unwrap();
    let mut wrapper = wrapper;
    wrapper.push(b' ');
    assert!(matches!(
        MetadataOwner::new(id1_configuration(0), 5, &provider, &wrapper,),
        Err(MetadataOwnerError::WrapperIdentity)
    ));
}

#[test]
fn per_call_wrapper_configuration_must_match_the_approved_profile() {
    let vectors = vectors();
    let mut owner = owner();
    let before = owner.state.canonical_bytes();
    let mut call = source_call(&vectors["continuous_calls"][0]);
    call.wrapper_configuration["payloads"]["11"]["group_id"] = Value::from(1);
    let error = owner.process_source(call).unwrap_err();
    assert_eq!(error.code(), "METADATA_WRAPPER_CONFIGURATION");
    assert_eq!(owner.state.canonical_bytes(), before);
    assert!(owner.is_retired());
}

#[test]
fn current_emdf_validation_reason_preserves_the_stable_error_code() {
    let reason = CurrentEmdfValidationReason::ParserRejected {
        parser_code: "EMDF_INVALID_SYNC_WORD",
    };
    let error = MetadataOwnerError::CurrentEmdfValidation(reason);

    assert_eq!(error.code(), "METADATA_CURRENT_EMDF_VALIDATION");
    assert_eq!(error.to_string(), "METADATA_CURRENT_EMDF_VALIDATION");
    assert_eq!(error.current_emdf_validation_reason(), Some(reason));
    assert_eq!(reason.code(), "PROTECTED_CONTAINER_PARSER_REJECTED");
    assert!(format!("{error:?}").contains("EMDF_INVALID_SYNC_WORD"));
}

#[test]
fn current_emdf_queue_predicate_reason_contains_only_queue_identity() {
    let reason = CurrentEmdfValidationReason::QueuePopReceiptMismatch { queue_id: 14 };

    assert_eq!(reason.code(), "QUEUE_POP_RECEIPT_MISMATCH");
    assert_eq!(
        format!("{reason:?}"),
        "QueuePopReceiptMismatch { queue_id: 14 }"
    );
}
