// pattern: Imperative Shell
use super::*;
use crate::{
    profile::{EmdfCarriageProfile, EncodingProfile},
    spatial::source_flags::SourceFlagsAttempt,
    timeline::FrontendCallbackIndex,
    transaction::CallbackLineageIdentity,
};
use serde_json::Value;
use std::fs;

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/spatial_audio_cold")
        .join(name)
}

#[test]
fn owner_state_reference_exposes_complete_cold_bytes_without_mutating_machine() {
    let machine = SpatialAudioMachine::new_cold(EncodingProfile::selected())
        .expect("selected profile constructs the complete cold root");
    let before_digest = machine.digest();
    let reference = machine.state_reference();

    assert_eq!(reference.owner_id(), "spatial_audio");
    assert_eq!(reference.state_ref(), "owners.spatial_audio");
    assert_eq!(reference.generation(), 0);
    assert_eq!(reference.canonical_bytes().len(), EXPECTED_COLD_BYTES);
    assert_eq!(
        reference.canonical_bytes(),
        machine.committed().canonical_bytes()
    );
    assert_eq!(reference.state_sha256(), before_digest);
    assert_eq!(reference.cold_sha256(), before_digest);
    assert_eq!(
        reference.cold_sha256().to_hex(),
        "35cfac927bbdca64f28954844938fe2936531f59f049dfca911c44d3a38b0b5e"
    );
    assert_eq!(machine.generation(), 0);
    assert_eq!(machine.digest(), before_digest);
}

#[test]
fn primitive_encoder_matches_every_input_only_case() {
    let _state = SpatialAudioState::new_cold().expect("cold state constructor succeeds");
    let vectors: Value = serde_json::from_slice(
        &fs::read(fixture("INPUT-ONLY-VECTORS.json")).expect("input vectors exist"),
    )
    .expect("input vectors are valid JSON");
    for case in vectors["primitive_cases"]
        .as_array()
        .expect("primitive cases are listed")
    {
        let input = case["input_value"].as_str().expect("primitive input id");
        let mut encoder = F04StateEncoder::new();
        match input {
            "none" => encoder.encode_none(),
            "int_zero" => encoder.encode_i64(0),
            "int_minus_one" => encoder.encode_i64(-1),
            "bool_false" => encoder.encode_bool(false),
            "bool_true" => encoder.encode_bool(true),
            "F32_plus_zero" => encoder.encode_f32(0.0).expect("F32 encodes"),
            "F32_minus_zero" => encoder
                .encode_f32(f32::from_bits(0x8000_0000))
                .expect("F32 preserves signed zero"),
            "empty_bytes" => encoder.encode_bytes(&[]).expect("B encodes"),
            "four_zero_bytes" => encoder.encode_bytes(&[0; 4]).expect("B remains raw bytes"),
            "empty_tuple" => encoder.encode_tuple_length(0).expect("T encodes"),
            "three_none_tuple" | "list_three_none" => {
                encoder.encode_tuple_length(3).expect("T encodes");
                for _ in 0..3 {
                    encoder.encode_none();
                }
            }
            unknown => panic!("unhandled primitive input case: {unknown}"),
        }
        let expected = decode_hex(case["expected_hex"].as_str().expect("expected hex"));
        assert_eq!(encoder.finish(), expected, "{input}");
        assert_eq!(
            case["expected_bytes"]
                .as_u64()
                .expect("expected byte count") as usize,
            expected.len(),
            "{input}"
        );
    }
}

#[test]
fn cold_validation_rejects_negative_zero_while_the_wire_encoder_preserves_it() {
    let mut state = SpatialAudioState::new_cold().expect("cold state constructor succeeds");
    let mut encoder = F04StateEncoder::new();
    encoder
        .encode_f32(f32::from_bits(0x8000_0000))
        .expect("generic F32 wire encoding preserves negative zero");
    assert_eq!(encoder.finish(), [b'F', 0, 0, 0, 0x80]);

    state.root.component_parameter_state.peak = f32::from_bits(0x8000_0000);
    assert_eq!(
        state.validate_cold(),
        Err(SpatialAudioError::NegativeZeroColdFloat)
    );
}

#[test]
fn equal_span_temporal_nodes_keep_distinct_buffers() {
    let state = SpatialAudioState::new_cold().expect("cold state constructor succeeds");
    let receiver = &state.root.receiver_state.temporal_states[4];
    let fitting = &state.root.fitting_qmf_state.target_temporal;

    assert_eq!(receiver.span, 896);
    assert_eq!(fitting.span, 896);
    assert_eq!(receiver.buffers.len(), 15);
    assert_eq!(fitting.buffers.len(), 15);
    assert_eq!(receiver.buffers[0].bytes.len(), 7_168);
    assert_eq!(fitting.buffers[0].bytes.len(), 7_168);
    assert_ne!(
        receiver.buffers[0].bytes.as_ptr(),
        fitting.buffers[0].bytes.as_ptr()
    );
    let mut receiver_encoder = F04StateEncoder::new();
    receiver
        .encode(&mut receiver_encoder)
        .expect("receiver temporal node encodes");
    let mut fitting_encoder = F04StateEncoder::new();
    fitting
        .encode(&mut fitting_encoder)
        .expect("fitting temporal node encodes");
    assert_eq!(receiver_encoder.finish(), fitting_encoder.finish());
}

#[test]
fn prepared_and_staged_capabilities_do_not_commit_the_machine() {
    let profile = EncodingProfile::selected();
    let machine = SpatialAudioMachine::new_cold(profile).expect("selected profile is supported");
    let before_digest = machine.digest();
    let proposal = SpatialAudioAfterProposal::new(
        SpatialAudioState::new_cold().expect("cold after proposal is typed"),
        join_metadata(profile, machine.generation(), before_digest),
    );
    let prepared = machine
        .prepare(proposal)
        .expect("matching lineage prepares");
    let staged = machine.stage(prepared).expect("matching lineage stages");

    assert_eq!(machine.generation(), 0);
    assert_eq!(machine.digest(), before_digest);
    assert_eq!(staged.before_digest(), before_digest);
    assert_eq!(staged.after_digest(), before_digest);
}

#[test]
fn staging_rejects_wrong_profile_and_before_generation() {
    let profile = EncodingProfile::selected();
    let machine = SpatialAudioMachine::new_cold(profile).expect("selected profile is supported");
    let state = SpatialAudioState::new_cold().expect("cold after proposal is typed");
    let wrong_profile = EncodingProfile::selected_with_carriage(EmdfCarriageProfile::EtsiEmdf);
    let wrong_profile_proposal =
        SpatialAudioAfterProposal::new(state, join_metadata(wrong_profile, 0, machine.digest()));
    assert_eq!(
        machine.prepare(wrong_profile_proposal).err(),
        Some(SpatialAudioError::ProfileBindingMismatch)
    );

    let stale_state = SpatialAudioState::new_cold().expect("cold after proposal is typed");
    let stale_proposal =
        SpatialAudioAfterProposal::new(stale_state, join_metadata(profile, 1, machine.digest()));
    assert_eq!(
        machine.prepare(stale_proposal).err(),
        Some(SpatialAudioError::GenerationMismatch)
    );
}

fn join_metadata(
    profile: EncodingProfile,
    before_generation: u64,
    before_digest: SpatialAudioDigest,
) -> SpatialAudioJoinMetadata {
    SpatialAudioJoinMetadata::new(
        crate::transaction::test_programme_id("cold-test-programme"),
        SourceFlagsAttempt::new(0),
        CallbackLineageIdentity::new(
            crate::transaction::test_programme_id("cold-test-programme"),
            SourceFlagsAttempt::new(0),
            FrontendCallbackIndex::new(0),
        ),
        FrontendCallbackIndex::new(0),
        before_generation,
        before_digest,
        profile,
    )
}

fn decode_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).expect("hex digit");
            let low = (pair[1] as char).to_digit(16).expect("hex digit");
            ((high << 4) | low) as u8
        })
        .collect()
}
