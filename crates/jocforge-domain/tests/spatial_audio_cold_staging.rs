// pattern: Imperative Shell
use jocforge_domain::{
    profile::{
        AuthoredInputProfile, CodecRateProfile, CoreCarrierProfile, EmdfCarriageProfile,
        EncodingProfile, JocDownmixProfile, SubstreamTopology,
    },
    spatial::{SpatialAudioError, SpatialAudioMachine},
    timeline::{FrontendCallbackIndex, ReceiverCallbackCount},
};
use serde_json::Value;
use std::{fs, path::PathBuf};

const COLD_BYTES: usize = 813_733;
const COLD_SHA256: &str = "35cfac927bbdca64f28954844938fe2936531f59f049dfca911c44d3a38b0b5e";

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/spatial_audio_cold")
        .join(name)
}

fn cold_machine() -> SpatialAudioMachine {
    SpatialAudioMachine::new_cold(EncodingProfile::selected())
        .expect("selected profile has an approved cold spatial_audio root")
}

#[test]
fn fresh_typed_constructor_bytes_match_the_complete_cold_kat() {
    let machine = cold_machine();
    let state = machine.committed();
    let actual = state.canonical_bytes();

    assert_eq!(machine.generation(), 0);
    assert_eq!(state.callback_index(), FrontendCallbackIndex::new(0));
    assert_eq!(
        state.receiver_callback_count(),
        ReceiverCallbackCount::new(0)
    );
    assert_eq!(actual.len(), COLD_BYTES);
    assert_eq!(state.digest().to_hex(), COLD_SHA256);

    // Read the KAT only after the fresh state and project encoder have produced actual bytes.
    let expected = fs::read(fixture("F04-COLD-PREIMAGE.bin")).expect("cold KAT fixture exists");
    assert_eq!(actual, expected);
}

#[test]
fn six_temporal_nodes_match_independent_kats_at_their_root_offsets() {
    let machine = cold_machine();
    let actual = machine.committed().canonical_bytes();
    let input_vectors: Value = serde_json::from_slice(
        &fs::read(fixture("INPUT-ONLY-VECTORS.json")).expect("input vectors exist"),
    )
    .expect("input vectors are valid JSON");

    let cases = input_vectors["temporal_cases"]
        .as_array()
        .expect("temporal cases are listed");
    assert_eq!(cases.len(), 6);
    for (index, case) in cases.iter().enumerate() {
        let path = case["path"].as_str().expect("temporal fixture path");
        let offset = case["position_in_full_preimage"]["offset"]
            .as_u64()
            .expect("temporal offset") as usize;
        let byte_count = case["node_wire_bytes"]
            .as_u64()
            .expect("temporal byte count") as usize;
        let expected = fs::read(fixture(&format!("TEMPORAL-COLD-{index}.bin")))
            .expect("temporal KAT fixture exists");
        let end = offset.checked_add(byte_count).expect("fixture range fits");

        assert_eq!(expected.len(), byte_count, "{path}");
        assert_eq!(&actual[offset..end], expected, "{path}");
    }
}

#[test]
fn every_washed_wire_mutation_changes_the_fresh_encoder_output() {
    let machine = cold_machine();
    let canonical = machine.committed().canonical_bytes();
    let vectors: Value = serde_json::from_slice(
        &fs::read(fixture("MUTATION-AND-TEMPORAL-VECTORS.json")).expect("mutation vectors exist"),
    )
    .expect("mutation vectors are valid JSON");

    for case in vectors["cases"]
        .as_array()
        .expect("mutation cases are listed")
    {
        assert_eq!(case["expected"]["reject"], true);
        let id = case["id"].as_str().expect("mutation case id");
        let candidate = mutate_wire_candidate(canonical, id);
        assert_ne!(
            candidate, canonical,
            "mutation must change the encoded candidate: {id}"
        );
    }
}

#[test]
fn unsupported_profile_rows_cannot_construct_a_spatial_audio_root() {
    let profile = EncodingProfile::new(
        AuthoredInputProfile::Unsupported,
        CoreCarrierProfile::Unsupported,
        JocDownmixProfile::Unsupported,
        SubstreamTopology::Unsupported,
        CodecRateProfile::Unsupported,
        EmdfCarriageProfile::Unsupported,
    );

    assert_eq!(
        SpatialAudioMachine::new_cold(profile).err(),
        Some(SpatialAudioError::UnsupportedProfile)
    );
}

fn mutate_wire_candidate(canonical: &[u8], id: &str) -> Vec<u8> {
    let mut bytes = canonical.to_vec();
    match id {
        "class_drift" => {
            let class_start = find_bytes(&bytes, b"F04CompositionState", 0);
            bytes[class_start + b"F04CompositionState".len() - 1] = b'x';
        }
        "field_name_drift" => {
            let field_start = find_bytes(&bytes, b"callback_index", 0);
            bytes[field_start + b"callback_index".len() - 1] = b'y';
        }
        "field_order_drift" => {
            let (first_start, first_end) = field_record(&bytes, b"callback_index", 0);
            let (second_start, second_end) = field_record(&bytes, b"retired", first_end);
            let mut reordered = Vec::with_capacity(bytes.len());
            reordered.extend_from_slice(&bytes[..first_start]);
            reordered.extend_from_slice(&bytes[second_start..second_end]);
            reordered.extend_from_slice(&bytes[first_start..first_end]);
            reordered.extend_from_slice(&bytes[second_end..]);
            bytes = reordered;
        }
        "field_count_drift" => {
            let class_start = find_bytes(&bytes, b"F04CompositionState", 0);
            let count_start = class_start + b"F04CompositionState".len();
            bytes[count_start..count_start + 2].copy_from_slice(&8_u16.to_le_bytes());
            let (last_field_start, _) = field_record(&bytes, b"receiver_callback_count", 0);
            bytes.truncate(last_field_start);
        }
        "temporal_indices_swapped" => {
            let temporal_start = find_bytes(&bytes, b"TemporalState", 0) - 3;
            let read_offset = integer_payload(&bytes, b"index_28", temporal_start);
            let write_offset = integer_payload(&bytes, b"index_32", temporal_start);
            let read = bytes[read_offset..read_offset + 8].to_vec();
            let write = bytes[write_offset..write_offset + 8].to_vec();
            bytes[read_offset..read_offset + 8].copy_from_slice(&write);
            bytes[write_offset..write_offset + 8].copy_from_slice(&read);
        }
        "buffer_length_drift" => {
            let temporal_start = find_bytes(&bytes, b"TemporalState", 0) - 3;
            let buffers_tag = field_tag(&bytes, b"buffers", temporal_start);
            let first_buffer_tag = buffers_tag + 1 + 8;
            assert_eq!(bytes[first_buffer_tag], b'B');
            let length_start = first_buffer_tag + 1;
            bytes[length_start..length_start + 8].copy_from_slice(&9_735_u64.to_le_bytes());
        }
        "None_to_int_zero" => {
            replace_first_lfe_none(&mut bytes, b'I', &0_i64.to_le_bytes());
        }
        "None_to_zero_PCM" => {
            replace_first_lfe_none(&mut bytes, b'B', &[0_u8; 6_144]);
        }
        "bool_to_int_zero" => {
            let retired_tag = field_tag(&bytes, b"retired", 0);
            assert_eq!(bytes[retired_tag], b'Z');
            bytes.splice(retired_tag..retired_tag + 2, [b'I', 0, 0, 0, 0, 0, 0, 0, 0]);
        }
        "negative_zero_F32" => {
            let parameter_class = find_bytes(&bytes, b"ComponentFilterParameterState", 0);
            let peak_tag = field_tag(&bytes, b"peak", parameter_class);
            assert_eq!(bytes[peak_tag], b'F');
            bytes[peak_tag + 1..peak_tag + 5].copy_from_slice(&0x8000_0000_u32.to_le_bytes());
        }
        "F32_endian_drift" => {
            let parameter_class = find_bytes(&bytes, b"ComponentFilterParameterState", 0);
            let value_tag = field_tag(&bytes, b"previous_gain", parameter_class);
            bytes[value_tag + 1..value_tag + 5].reverse();
        }
        "I64_endian_drift" => {
            let temporal_start = find_bytes(&bytes, b"TemporalState", 0) - 3;
            let span = integer_payload(&bytes, b"span", temporal_start);
            bytes[span..span + 8].reverse();
        }
        "class_length_endian_drift" => bytes[1..3].reverse(),
        "trailing_padding" => bytes.push(0),
        "truncated_root" => {
            bytes.pop();
        }
        unknown => panic!("unhandled washed mutation case: {unknown}"),
    }
    bytes
}

fn find_bytes(bytes: &[u8], needle: &[u8], start: usize) -> usize {
    start
        + bytes[start..]
            .windows(needle.len())
            .position(|window| window == needle)
            .unwrap_or_else(|| panic!("missing wire bytes: {}", String::from_utf8_lossy(needle)))
}

fn field_tag(bytes: &[u8], name: &[u8], start: usize) -> usize {
    find_bytes(bytes, name, start) + name.len()
}

fn field_record(bytes: &[u8], name: &[u8], start: usize) -> (usize, usize) {
    let name_start = find_bytes(bytes, name, start);
    let record_start = name_start - 2;
    let tag = name_start + name.len();
    let node_bytes = match bytes[tag] {
        b'I' => 9,
        b'Z' => 2,
        b'F' => 5,
        _ => panic!("root field test helper expects scalar field {name:?}"),
    };
    (record_start, tag + node_bytes)
}

fn integer_payload(bytes: &[u8], name: &[u8], start: usize) -> usize {
    let tag = field_tag(bytes, name, start);
    assert_eq!(bytes[tag], b'I');
    tag + 1
}

fn replace_first_lfe_none(bytes: &mut Vec<u8>, replacement_tag: u8, payload: &[u8]) {
    let class_start = find_bytes(bytes, b"LfeDelayRingState", 0);
    let rows_tag = field_tag(bytes, b"rows", class_start);
    assert_eq!(bytes[rows_tag], b'T');
    let none_tag = rows_tag + 1 + 8;
    assert_eq!(bytes[none_tag], b'N');
    let mut replacement = vec![replacement_tag];
    if replacement_tag == b'I' {
        replacement.extend_from_slice(payload);
    } else {
        replacement.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        replacement.extend_from_slice(payload);
    }
    bytes.splice(none_tag..=none_tag, replacement);
}
