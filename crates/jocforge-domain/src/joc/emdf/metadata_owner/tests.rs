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
        validate_current_container: false,
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
