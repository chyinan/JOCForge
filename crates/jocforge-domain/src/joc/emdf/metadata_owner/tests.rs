use super::*;
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

fn owner() -> MetadataOwner {
    let root = fixture_root();
    MetadataOwner::new(
        "P0",
        0,
        5,
        &std::fs::read(root.join("metadata-public-provider.json")).unwrap(),
        &std::fs::read(root.join("metadata-wrapper-config.json")).unwrap(),
    )
    .unwrap()
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

    MetadataSourceCall {
        programme_id: identity["programme_id"].as_str().unwrap().to_owned(),
        attempt: identity["attempt"].as_u64().unwrap(),
        transaction_id: identity["transaction_id"].as_str().unwrap().to_owned(),
        source_index: CodecSourceIndex::new(identity["source_index"].as_u64().unwrap()),
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
    assert_eq!(
        bytes_hex(&owner.state.canonical_bytes()),
        vectors["cold"]["canonical_hex"]
    );
    assert_eq!(owner.state.sha256_hex(), vectors["cold"]["sha256"]);

    for call in vectors["continuous_calls"].as_array().unwrap() {
        let result = owner.process_source(source_call(call)).unwrap();
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
    for call in vectors["continuous_calls"].as_array().unwrap() {
        owner.process_source(source_call(call)).unwrap();
    }

    let eof = owner.prepare_eof(5).unwrap();
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
    let retired = owner().reset_for_attempt(1).unwrap();
    assert_eq!(
        bytes_hex(&retired.state.canonical_bytes()),
        vectors["cold"]["canonical_hex"]
    );
    assert_eq!(retired.state.sha256_hex(), vectors["cold"]["sha256"]);
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
    let before = owner.state.canonical_bytes();
    let error = owner.prepare_eof(0).unwrap_err();
    assert_eq!(error.code(), "METADATA_EOF_SOURCE_COUNT");
    assert_eq!(owner.state.canonical_bytes(), before);
    assert!(owner.is_retired());
}

#[test]
fn provider_and_wrapper_identity_are_checked_before_use() {
    let root = fixture_root();
    let mut provider = std::fs::read(root.join("metadata-public-provider.json")).unwrap();
    provider[0] ^= 1;
    let wrapper = std::fs::read(root.join("metadata-wrapper-config.json")).unwrap();
    assert!(matches!(
        MetadataOwner::new("P0", 0, 5, &provider, &wrapper),
        Err(MetadataOwnerError::ProviderIdentity)
    ));

    let provider = std::fs::read(root.join("metadata-public-provider.json")).unwrap();
    let mut wrapper = wrapper;
    wrapper.push(b' ');
    assert!(matches!(
        MetadataOwner::new("P0", 0, 5, &provider, &wrapper),
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
