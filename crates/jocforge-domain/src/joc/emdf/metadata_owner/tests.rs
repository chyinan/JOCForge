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
    let transaction_id = crate::transaction::CallbackLineageIdentity::new(
        programme_id.clone(),
        attempt,
        callback_index,
    );
    MetadataSourceCall {
        programme_id,
        attempt,
        transaction_id,
        source_index,
        dispatch: JocDispatchIndex::new(identity["dispatch"].as_u64().unwrap()),
        au_index: identity["au_index"].as_u64().map(PublishedAuIndex::new),
        config: input["config"].as_array().unwrap().clone(),
        bodies: MetadataBodies {
            id11: body("11"),
            id2: body("2"),
            id14: body("14"),
        },
        wrapper_configuration: input["wrapper_config"].clone(),
    }
}

fn bytes_hex(bytes: &[u8]) -> String {
    hex_encode(bytes)
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
        call.transaction_id.clone(),
        call.source_index,
        call.bodies,
    );
    let proposal = before
        .stage_source(MetadataStageInput {
            au_index: call.au_index,
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
        &call.transaction_id
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
            call.transaction_id.clone(),
            call.source_index,
            call.bodies,
        );
        let proposal = owner
            .stage_source(MetadataStageInput {
                au_index: call.au_index,
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
        call.transaction_id.clone(),
        call.source_index,
        call.bodies,
    );
    let metadata_proposal = metadata
        .stage_source(MetadataStageInput {
            au_index: call.au_index,
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
