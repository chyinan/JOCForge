//! Synthetic Flat7x F13 values for downstream IO integration tests.
//!
//! This module is enabled only by the workspace `test-support` feature and bypasses
//! production F13/F12 producer integration. Its output can exercise receipt mechanics only.

use super::{
    Id14PopDisposition, Id14QueuePopReceipt, MetadataBodies, MetadataOwner, MetadataPayloadBody,
    MetadataQueueProducerProvenance, MetadataSourceCall, MetadataSourceProposal,
    MetadataStageInput, ParsedMetadataSourceContainer,
};
use crate::timeline::{CodecSourceIndex, JocDispatchIndex, PublishedAuIndex};
use crate::{
    joc::emdf::{
        id1_configuration::Id1MeasurementConfiguration, metadata_bodies::MetadataBodiesProposal,
    },
    meter::measurement_receipt::{ProgrammeMeasurementReceipt, ProgrammeMeasurementReceiptInput},
    spatial::SpatialAudioDigest,
};
use serde_json::{Value, json};

pub struct Flat7xReceiptGateMetadata {
    pub proposal: MetadataSourceProposal,
    pub zero_slot_container: Vec<u8>,
    pub protected_container: crate::integrity::keyless_checksum::ProtectedEmdfContainer,
}

#[derive(Clone, Copy)]
pub struct Flat7xReceiptGateMetadataTestInputs<'a> {
    pub f15_pair_vectors: &'a [u8],
    pub metadata_owner_vectors: &'a [u8],
    pub metadata_public_provider: &'a [u8],
    pub metadata_wrapper_config: &'a [u8],
    pub keyless_integrity_provider: &'a [u8],
}

fn metadata_vectors(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("metadata owner vectors must be valid JSON")
}

fn id1_configuration(attempt: u64) -> Id1MeasurementConfiguration {
    let measurement =
        ProgrammeMeasurementReceipt::from_verified_prepass(ProgrammeMeasurementReceiptInput {
            programme_id: crate::transaction::ProgrammeId::new("P0").unwrap(),
            attempt: crate::transaction::AttemptId::new(attempt),
            input_sha256: [0x11; 32],
            config_sha256: [0x22; 32],
            dialogue_word: (-18.0_f32).to_bits(),
            itu_word: (-17.0_f32).to_bits(),
            true_peak_word: (-4.0_f32).to_bits(),
            gating_kind: "DIALOGUE",
            selected_lufs_word: (-17.0_f32).to_bits(),
            meter_terminal_sha256: [0x33; 32],
            normal_callbacks: 4_247,
            zero_drains: 3,
            energy_windows: 6_794,
        })
        .unwrap();
    Id1MeasurementConfiguration::from_measurement_for_test(&measurement).unwrap()
}

fn metadata_owner(
    metadata_public_provider: &[u8],
    metadata_wrapper_config: &[u8],
) -> MetadataOwner {
    MetadataOwner::new(
        id1_configuration(0),
        5,
        metadata_public_provider,
        metadata_wrapper_config,
    )
    .unwrap()
}

fn decode_hex(text: &str) -> Vec<u8> {
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
            bytes: decode_hex(body["payload_hex"].as_str().unwrap()),
        })
    };

    let programme_id =
        crate::transaction::ProgrammeId::new(identity["programme_id"].as_str().unwrap()).unwrap();
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

fn bind_id14_provenance(call: &mut MetadataSourceCall, spatial_digest: SpatialAudioDigest) {
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
                sequence_count: crate::joc::id14_body::Id14SequenceCount::new(0)
                    .expect("selected sequence count"),
                object_count,
            },
            &body.bytes,
        )
        .expect("same-source ID14 provenance"),
    );
}

fn source_zero_proposal(
    input: Flat7xReceiptGateMetadataTestInputs<'_>,
) -> (MetadataOwner, MetadataSourceProposal) {
    let vectors = metadata_vectors(input.metadata_owner_vectors);
    let source = source_call(&vectors["continuous_calls"][0]);
    let spatial_digest =
        crate::spatial::SpatialAudioMachine::new_cold(crate::profile::EncodingProfile::selected())
            .unwrap()
            .digest();
    let mut source = source;
    bind_id14_provenance(&mut source, spatial_digest);
    let producer = source.bodies.id14_provenance.clone().unwrap();
    let transaction_id = producer.producer_transaction_id().unwrap().clone();
    let bodies = MetadataBodiesProposal::for_test_with_id14_provenance(
        source.programme_id.clone(),
        source.attempt,
        source.source_lineage.clone(),
        source.source_index,
        source.bodies,
        producer,
    );
    let before = metadata_owner(
        input.metadata_public_provider,
        input.metadata_wrapper_config,
    );
    let proposal = before
        .stage_source(MetadataStageInput {
            au_index: source.au_index,
            transaction_id,
            bodies,
        })
        .unwrap();
    (before, proposal)
}

fn receipt_gate_emdf(
    keyless_integrity_provider: &[u8],
) -> (
    Vec<u8>,
    crate::integrity::keyless_checksum::ProtectedEmdfContainer,
) {
    let mut zero_slot = vec![0_u8; 16];
    zero_slot[..2].copy_from_slice(&[0x58, 0x38]);
    zero_slot[2..4].copy_from_slice(&12_u16.to_be_bytes());
    for (start, width, value) in [(37_usize, 2_usize, 2_u64), (39, 2, 1)] {
        for offset in 0..width {
            let bit_index = start + offset;
            let mask = 1_u8 << (7 - bit_index % 8);
            if (value >> (width - offset - 1)) & 1 != 0 {
                zero_slot[bit_index / 8] |= mask;
            } else {
                zero_slot[bit_index / 8] &= !mask;
            }
        }
    }
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
    let provider =
        crate::integrity::keyless_checksum::validate_keyless_provider(keyless_integrity_provider)
            .expect("selected public keyless provider fixture validates");
    let protected =
        crate::integrity::keyless_checksum::protect_emdf_container(&provider, &zero_slot, &layout)
            .expect("synthetic 16-byte checksum input is protectable");
    (zero_slot, protected)
}

fn synthetic_f13_metadata_proposal(
    input: Flat7xReceiptGateMetadataTestInputs<'_>,
    identity: &Value,
    f13: &Value,
    zero_slot_container: &[u8],
    protected_container: &[u8],
    carrier_block: u8,
) -> MetadataSourceProposal {
    let (_before, mut proposal) = source_zero_proposal(input);
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
    proposal.receipt.transaction_id = lineage;
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

/// Returns synthetic values for F15 pair receipt tests only.
///
/// # Panics
///
/// Panics when a checked-in fixture is missing, malformed, or no longer matches the test contract.
pub fn flat7x_receipt_gate_metadata_for_test(
    input: Flat7xReceiptGateMetadataTestInputs<'_>,
) -> (
    MetadataSourceProposal,
    Vec<u8>,
    crate::integrity::keyless_checksum::ProtectedEmdfContainer,
) {
    let vectors: Value = serde_json::from_slice(input.f15_pair_vectors)
        .expect("Wash receipt-pair vector is valid JSON");
    let pair = &vectors["positive_pair"];
    let identity = &pair["integrity_identity"];
    let (mut zero_slot_container, protected) = receipt_gate_emdf(input.keyless_integrity_provider);
    let mut protected_bytes = protected.bytes().to_vec();
    if let Some(expected_len) = pair["f13_metadata"]["protected_container_bytes"].as_u64() {
        let expected_len = usize::try_from(expected_len).unwrap();
        if expected_len != protected_bytes.len() {
            zero_slot_container.resize(expected_len, 0);
            zero_slot_container[2..4]
                .copy_from_slice(&u16::try_from(expected_len - 4).unwrap().to_be_bytes());
            protected_bytes.resize(expected_len, 0);
            protected_bytes[2..4]
                .copy_from_slice(&u16::try_from(expected_len - 4).unwrap().to_be_bytes());
        }
    }
    let metadata = synthetic_f13_metadata_proposal(
        input,
        identity,
        &pair["f13_metadata"],
        &zero_slot_container,
        &protected_bytes,
        2,
    );
    (metadata, zero_slot_container, protected)
}
