// pattern: Functional Core

use jocforge_core::{
    AdmObjectKind, generate_synthetic_direct_speaker_fixture, generate_synthetic_fixture,
    generate_synthetic_scene_fixture, generate_synthetic_unsupported_fixture, parse_bw64_adm,
    parse_bw64_adm_metadata, validate_source_derived_object_limit,
};

#[test]
fn generated_fixture_round_trips_bw64_tracks_relationships_and_positions() {
    let fixture = generate_synthetic_fixture(4).expect("synthetic ADM fixture");
    let parsed = parse_bw64_adm(&fixture.bw64_header_and_pcm).expect("parse fixture");

    assert_eq!(parsed.sample_rate, 48_000);
    assert_eq!(parsed.channel_count, 15);
    assert_eq!(parsed.objects.len(), 15);
    assert_eq!(parsed.objects[0].track_index, 1);
    assert_eq!(parsed.objects[14].track_index, 15);
    assert!(
        parsed
            .objects
            .iter()
            .all(|object| object.position.is_finite())
    );
    assert_eq!(fixture.matrix.len(), 15);
    assert_eq!(fixture.matrix[0].len(), 7);
}

#[test]
fn plain_riff_without_adm_chunks_is_rejected_explicitly() {
    let bytes = b"RIFF\x00\x00\x00\x00WAVEfmt ".to_vec();
    let error = parse_bw64_adm(&bytes).expect_err("plain RIFF is not ADM");

    assert!(error.to_string().contains("axml"));
}

#[test]
fn metadata_census_preserves_object_kinds_and_relationship_counts() {
    let fixture = generate_synthetic_fixture(1).expect("synthetic ADM fixture");
    let parsed = parse_bw64_adm(&fixture.bw64_header_and_pcm).expect("parse fixture");

    assert_eq!(parsed.objects.len(), 15);
    assert!(
        parsed
            .objects
            .iter()
            .all(|object| object.kind == AdmObjectKind::Objects)
    );
}

#[test]
fn source_derived_admission_rejects_more_than_fifteen_authored_objects() {
    assert!(validate_source_derived_object_limit(15).is_ok());
    let error = validate_source_derived_object_limit(28).expect_err("28 objects are out of scope");

    assert_eq!(
        error.to_string(),
        "Unsupported ADM scene: 28 authored dynamic objects exceed current <=15 source-derived MVP profile; scene reduction is not implemented."
    );
}

#[test]
fn generated_clean_corpus_supports_over_budget_object_counts() {
    for object_count in [16, 20, 28] {
        let fixture =
            generate_synthetic_scene_fixture(1, object_count).expect("over-budget clean fixture");
        let parsed = parse_bw64_adm(&fixture.bw64_header_and_pcm).expect("parse clean fixture");

        assert_eq!(parsed.objects.len(), object_count);
        assert_eq!(parsed.channel_count, object_count);
        assert_eq!(fixture.matrix.len(), object_count);
        assert!(
            parsed
                .pcm
                .iter()
                .all(|channel| channel.iter().any(|value| *value != 0.0))
        );
    }
}

#[test]
fn direct_speakers_preserve_all_channel_bindings_and_positions() {
    let fmt = [1, 0, 2, 0, 0x80, 0xbb, 0, 0, 0, 0x65, 4, 0, 6, 0, 24, 0];
    let axml = br#"<audioFormatExtended><audioPackFormat audioPackFormatID="AP_00011001" typeLabel="0001" typeDefinition="DirectSpeakers"><audioChannelFormatIDRef>AC_00011001</audioChannelFormatIDRef><audioChannelFormatIDRef>AC_00011002</audioChannelFormatIDRef></audioPackFormat><audioChannelFormat audioChannelFormatID="AC_00011001" audioChannelFormatName="RoomCentricLeft"><audioBlockFormat><position coordinate="X">-1</position><position coordinate="Y">0</position></audioBlockFormat></audioChannelFormat><audioChannelFormat audioChannelFormatID="AC_00011002" audioChannelFormatName="RoomCentricRight"><audioBlockFormat><position coordinate="X">1</position><position coordinate="Y">0</position></audioBlockFormat></audioChannelFormat><audioObject audioObjectID="AO_0001"><audioPackFormatIDRef>AP_00011001</audioPackFormatIDRef><audioTrackUIDRef>ATU_00000001</audioTrackUIDRef><audioTrackUIDRef>ATU_00000002</audioTrackUIDRef></audioObject></audioFormatExtended>"#;
    let mut chna = vec![2, 0, 2, 0];
    for index in 1_u16..=2 {
        chna.extend_from_slice(&index.to_le_bytes());
        let track_uid = format!("ATU_{index:08X}");
        chna.extend_from_slice(&padded_ascii(&track_uid, 12));
        chna.extend_from_slice(&padded_ascii("AT_00000001_01", 14));
        chna.extend_from_slice(&padded_ascii("AP_00011001", 11));
        chna.push(0);
    }

    let parsed = parse_bw64_adm_metadata(&fmt, axml, &chna, 12).expect("metadata parses");
    assert_eq!(parsed.objects.len(), 1);
    assert_eq!(parsed.direct_speaker_count(), 1);
    let object = &parsed.objects[0];
    assert_eq!(object.kind, AdmObjectKind::DirectSpeakers);
    assert_eq!(object.channels.len(), 2);
    assert_eq!(object.channels[0].track_index, 1);
    assert_eq!(object.channels[1].track_index, 2);
    assert!((object.channels[0].position.azimuth + 1.0).abs() < 1.0e-12);
    assert!((object.channels[1].position.azimuth - 1.0).abs() < 1.0e-12);
}

#[test]
fn controlled_direct_speaker_fixture_contains_non_lfe_and_lfe_paths() {
    let fixture = generate_synthetic_direct_speaker_fixture(1).expect("DirectSpeakers fixture");
    let parsed = parse_bw64_adm(&fixture.bw64_header_and_pcm).expect("fixture parses");

    assert_eq!(
        parsed
            .objects
            .iter()
            .filter(|object| object.kind == AdmObjectKind::Objects)
            .count(),
        2
    );
    assert_eq!(
        parsed
            .objects
            .iter()
            .filter(|object| object.kind == AdmObjectKind::DirectSpeakers)
            .count(),
        1
    );
    assert_eq!(parsed.objects[0].channels.len(), 3);
    assert_eq!(
        parsed.objects[0]
            .channels
            .iter()
            .filter(|channel| !channel.is_lfe)
            .count(),
        2
    );
    assert_eq!(
        parsed.objects[0]
            .channels
            .iter()
            .filter(|channel| channel.is_lfe)
            .count(),
        1
    );
}

#[test]
fn unsupported_fixture_is_classified_as_other_for_explicit_rejection() {
    let fixture = generate_synthetic_unsupported_fixture(1).expect("unsupported fixture");
    let parsed = parse_bw64_adm(&fixture.bw64_header_and_pcm).expect("fixture parses");

    assert_eq!(parsed.objects.len(), 1);
    assert_eq!(parsed.objects[0].kind, AdmObjectKind::Other);
}

fn padded_ascii(value: &str, width: usize) -> Vec<u8> {
    let mut bytes = vec![0_u8; width];
    let copy = value.len().min(width);
    bytes[..copy].copy_from_slice(&value.as_bytes()[..copy]);
    bytes
}
