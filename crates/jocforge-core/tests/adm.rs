// pattern: Functional Core

use jocforge_core::{
    AdmObjectKind, generate_synthetic_direct_speaker_fixture, generate_synthetic_fixture,
    generate_synthetic_scene_fixture, generate_synthetic_unsupported_fixture, parse_bw64_adm,
    parse_bw64_adm_metadata, validate_source_derived_adm_metadata,
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
    let axml = br#"<audioFormatExtended><audioPackFormat audioPackFormatID="AP_00011001" typeLabel="0001" typeDefinition="DirectSpeakers"><audioChannelFormatIDRef>AC_00011001</audioChannelFormatIDRef><audioChannelFormatIDRef>AC_00011002</audioChannelFormatIDRef></audioPackFormat><audioChannelFormat audioChannelFormatID="AC_00011001" audioChannelFormatName="RoomCentricLeft"><audioBlockFormat><speakerLabel>M-030</speakerLabel><position coordinate="X">-1</position><position coordinate="Y">0</position></audioBlockFormat></audioChannelFormat><audioChannelFormat audioChannelFormatID="AC_00011002" audioChannelFormatName="RoomCentricRight"><audioBlockFormat><speakerLabel>M+030</speakerLabel><position coordinate="X">1</position><position coordinate="Y">0</position></audioBlockFormat></audioChannelFormat><audioObject audioObjectID="AO_0001"><audioPackFormatIDRef>AP_00011001</audioPackFormatIDRef><audioTrackUIDRef>ATU_00000001</audioTrackUIDRef><audioTrackUIDRef>ATU_00000002</audioTrackUIDRef></audioObject></audioFormatExtended>"#;
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
fn semantic_lfe_routing_ignores_human_readable_channel_names() {
    let lfe_xml = direct_speaker_xml("Piano", "RC_LFE");
    let lfe = parse_bw64_adm_metadata(
        &one_channel_fmt(),
        lfe_xml.as_bytes(),
        &one_channel_chna(),
        3,
    )
    .expect("semantic LFE parses");
    assert!(lfe.objects[0].channels[0].is_lfe);

    let non_lfe_xml = direct_speaker_xml("LFE", "M+000");
    let non_lfe = parse_bw64_adm_metadata(
        &one_channel_fmt(),
        non_lfe_xml.as_bytes(),
        &one_channel_chna(),
        3,
    )
    .expect("semantic non-LFE parses");
    assert!(!non_lfe.objects[0].channels[0].is_lfe);

    let unicode_lfe_xml = direct_speaker_xml("低音⚙", "LFE2");
    let unicode_lfe = parse_bw64_adm_metadata(
        &one_channel_fmt(),
        unicode_lfe_xml.as_bytes(),
        &one_channel_chna(),
        3,
    )
    .expect("arbitrary Unicode LFE name parses");
    assert!(unicode_lfe.objects[0].channels[0].is_lfe);

    let low_pass_xml = direct_speaker_xml("Bass", "").replace(
        "<position coordinate=\"X\">0</position>",
        "<frequency><lowPass>120.0</lowPass></frequency><position coordinate=\"X\">0</position>",
    );
    let low_pass_lfe = parse_bw64_adm_metadata(
        &one_channel_fmt(),
        low_pass_xml.as_bytes(),
        &one_channel_chna(),
        3,
    )
    .expect("lowPass LFE semantics parse");
    assert!(low_pass_lfe.objects[0].channels[0].is_lfe);
}

#[test]
fn direct_speaker_without_lfe_semantics_is_rejected_closed() {
    let xml = direct_speaker_xml("LFE", "");
    let error = parse_bw64_adm_metadata(&one_channel_fmt(), xml.as_bytes(), &one_channel_chna(), 3)
        .expect_err("missing DirectSpeakers semantic metadata must fail closed");

    assert!(error.to_string().contains("LFE semantic metadata"));
}

#[test]
fn duplicate_chna_bindings_are_rejected_without_overwrite() {
    let cases = [
        vec![
            ("ATU_00000001", 1_u16, "AT_00000001_01", "AP_00000001"),
            ("ATU_00000001", 1_u16, "AT_00000001_01", "AP_00000001"),
        ],
        vec![
            ("ATU_00000001", 1_u16, "AT_00000001_01", "AP_00000001"),
            ("ATU_00000002", 1_u16, "AT_00000002_01", "AP_00000001"),
        ],
        vec![
            ("ATU_00000001", 1_u16, "AT_00000001_01", "AP_00000001"),
            ("ATU_00000001", 2_u16, "AT_other_01", "AP_other01"),
        ],
    ];

    for rows in cases {
        let error = parse_bw64_adm_metadata(
            &one_channel_fmt(),
            one_object_xml(
                "",
                "<audioBlockFormat><position coordinate=\"X\">0</position><position coordinate=\"Y\">0</position></audioBlockFormat>",
            )
            .as_bytes(),
            &chna_rows(&rows),
            3,
        )
        .expect_err("duplicate CHNA bindings must be rejected");

        assert!(error.to_string().contains("duplicate CHNA"));
    }
}

#[test]
fn chna_pack_relation_conflicts_are_rejected() {
    let error = parse_bw64_adm_metadata(
        &one_channel_fmt(),
        one_object_xml(
            "",
            "<audioBlockFormat><position coordinate=\"X\">0</position><position coordinate=\"Y\">0</position></audioBlockFormat>",
        )
        .as_bytes(),
        &chna_rows(&[(
            "ATU_00000001",
            1,
            "AT_00000001_01",
            "AP_CONFLICT",
        )]),
        3,
    )
    .expect_err("CHNA pack relation conflict must be rejected");

    assert!(error.to_string().contains("pack relation conflicts"));
}

#[test]
fn metadata_rejects_multiple_audio_blocks_instead_of_selecting_the_first() {
    let error = parse_bw64_adm_metadata(
        &one_channel_fmt(),
        one_object_xml(
            "",
            "<audioBlockFormat><position coordinate=\"X\">0</position><position coordinate=\"Y\">0</position></audioBlockFormat><audioBlockFormat><position coordinate=\"X\">1</position><position coordinate=\"Y\">0</position></audioBlockFormat>",
        )
        .as_bytes(),
        &one_channel_chna(),
        3,
    )
    .expect_err("multiple blocks must be rejected by the narrow parser");

    assert!(error.to_string().contains("multiple audioBlockFormat"));
}

#[test]
fn metadata_rejects_nonzero_object_start_in_full_track_subset() {
    let error = parse_bw64_adm_metadata(
        &one_channel_fmt(),
        one_object_xml(
            "start=\"00:00:01.00000\"",
            "<audioBlockFormat><position coordinate=\"X\">0</position><position coordinate=\"Y\">0</position></audioBlockFormat>",
        )
        .as_bytes(),
        &one_channel_chna(),
        3,
    )
    .expect_err("non-zero object timing must be rejected by the narrow parser");

    assert!(error.to_string().contains("object timing"));
}

#[test]
fn metadata_rejects_nonzero_block_runtime_in_full_track_subset() {
    let error = parse_bw64_adm_metadata(
        &one_channel_fmt(),
        one_object_xml(
            "",
            "<audioBlockFormat rtime=\"00:00:00.50000\"><position coordinate=\"X\">0</position><position coordinate=\"Y\">0</position></audioBlockFormat>",
        )
        .as_bytes(),
        &one_channel_chna(),
        3,
    )
    .expect_err("non-zero block runtime must be rejected by the narrow parser");

    assert!(error.to_string().contains("block timing"));
}

#[test]
fn metadata_rejects_objects_outside_the_selected_programme() {
    let error = parse_bw64_adm_metadata(
        &one_channel_fmt(),
        one_object_with_programme_xml().as_bytes(),
        &one_channel_chna(),
        3,
    )
    .expect_err("unassociated object must not be silently encoded");

    assert!(
        error
            .to_string()
            .contains("not referenced by the programme")
    );
}

#[test]
fn metadata_accepts_nonzero_programme_timecode_when_duration_matches_pcm() {
    let metadata = parse_bw64_adm_metadata(
        &one_channel_fmt(),
        one_object_with_offset_programme_xml().as_bytes(),
        &one_channel_chna(),
        48_000 * 3,
    )
    .expect("programme timecode offset is not object motion");

    assert_eq!(
        metadata.programmes[0].start.as_deref(),
        Some("01:00:10.00000")
    );
}

#[test]
fn source_derived_encode_rejects_metadata_without_a_programme() {
    let metadata = parse_bw64_adm_metadata(
        &one_channel_fmt(),
        one_object_xml(
            "",
            "<audioBlockFormat><position coordinate=\"X\">0</position><position coordinate=\"Y\">0</position></audioBlockFormat>",
        )
        .as_bytes(),
        &one_channel_chna(),
        3,
    )
    .expect("metadata parses for inventory");

    let error = validate_source_derived_adm_metadata(&metadata)
        .expect_err("source-derived encode needs programme membership");

    assert!(error.to_string().contains("requires an audioProgramme"));
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

fn one_channel_fmt() -> [u8; 16] {
    [1, 0, 1, 0, 0x80, 0xbb, 0, 0, 0x65, 4, 0, 0, 3, 0, 24, 0]
}

fn one_channel_chna() -> Vec<u8> {
    let mut chna = vec![1, 0, 1, 0];
    chna.extend_from_slice(&1_u16.to_le_bytes());
    chna.extend_from_slice(&padded_ascii("ATU_00000001", 12));
    chna.extend_from_slice(&padded_ascii("AT_00000001_01", 14));
    chna.extend_from_slice(&padded_ascii("AP_00000001", 11));
    chna.push(0);
    chna
}

fn chna_rows(rows: &[(&str, u16, &str, &str)]) -> Vec<u8> {
    let mut chna = Vec::with_capacity(4 + rows.len() * 40);
    chna.extend_from_slice(&(rows.len() as u16).to_le_bytes());
    chna.extend_from_slice(&(rows.len() as u16).to_le_bytes());
    for (uid, track_index, track_ref, pack_ref) in rows {
        chna.extend_from_slice(&track_index.to_le_bytes());
        chna.extend_from_slice(&padded_ascii(uid, 12));
        chna.extend_from_slice(&padded_ascii(track_ref, 14));
        chna.extend_from_slice(&padded_ascii(pack_ref, 11));
        chna.push(0);
    }
    chna
}

fn one_object_xml(object_attributes: &str, block_formats: &str) -> String {
    format!(
        "<audioFormatExtended><audioPackFormat audioPackFormatID=\"AP_00000001\" typeLabel=\"0003\" typeDefinition=\"Objects\"><audioChannelFormatIDRef>AC_00000001</audioChannelFormatIDRef></audioPackFormat><audioChannelFormat audioChannelFormatID=\"AC_00000001\" audioChannelFormatName=\"Object\">{block_formats}</audioChannelFormat><audioObject audioObjectID=\"AO_0001\" {object_attributes}><audioPackFormatIDRef>AP_00000001</audioPackFormatIDRef><audioTrackUIDRef>ATU_00000001</audioTrackUIDRef></audioObject></audioFormatExtended>"
    )
}

fn direct_speaker_xml(channel_name: &str, speaker_label: &str) -> String {
    let semantic = if speaker_label.is_empty() {
        String::new()
    } else {
        format!("<speakerLabel>{speaker_label}</speakerLabel>")
    };
    format!(
        "<audioFormatExtended><audioPackFormat audioPackFormatID=\"AP_00000001\" typeLabel=\"0002\" typeDefinition=\"DirectSpeakers\"><audioChannelFormatIDRef>AC_00000001</audioChannelFormatIDRef></audioPackFormat><audioChannelFormat audioChannelFormatID=\"AC_00000001\" audioChannelFormatName=\"{channel_name}\"><audioBlockFormat>{semantic}<position coordinate=\"X\">0</position><position coordinate=\"Y\">0</position></audioBlockFormat></audioChannelFormat><audioObject audioObjectID=\"AO_0001\" audioObjectName=\"{channel_name}\"><audioPackFormatIDRef>AP_00000001</audioPackFormatIDRef><audioTrackUIDRef>ATU_00000001</audioTrackUIDRef></audioObject></audioFormatExtended>"
    )
}

fn one_object_with_programme_xml() -> String {
    "<audioFormatExtended><audioProgramme audioProgrammeID=\"APR_0001\"><audioContentIDRef>ACO_0001</audioContentIDRef></audioProgramme><audioContent audioContentID=\"ACO_0001\"><audioObjectIDRef>AO_OTHER</audioObjectIDRef></audioContent><audioPackFormat audioPackFormatID=\"AP_00000001\" typeLabel=\"0003\" typeDefinition=\"Objects\"><audioChannelFormatIDRef>AC_00000001</audioChannelFormatIDRef></audioPackFormat><audioChannelFormat audioChannelFormatID=\"AC_00000001\" audioChannelFormatName=\"Object\"><audioBlockFormat><position coordinate=\"X\">0</position><position coordinate=\"Y\">0</position></audioBlockFormat></audioChannelFormat><audioObject audioObjectID=\"AO_0001\"><audioPackFormatIDRef>AP_00000001</audioPackFormatIDRef><audioTrackUIDRef>ATU_00000001</audioTrackUIDRef></audioObject></audioFormatExtended>"
        .to_owned()
}

fn one_object_with_offset_programme_xml() -> String {
    "<audioFormatExtended><audioProgramme audioProgrammeID=\"APR_0001\" start=\"01:00:10.00000\" end=\"01:00:11.00000\"><audioContentIDRef>ACO_0001</audioContentIDRef></audioProgramme><audioContent audioContentID=\"ACO_0001\"><audioObjectIDRef>AO_0001</audioObjectIDRef></audioContent><audioPackFormat audioPackFormatID=\"AP_00000001\" typeLabel=\"0003\" typeDefinition=\"Objects\"><audioChannelFormatIDRef>AC_00000001</audioChannelFormatIDRef></audioPackFormat><audioChannelFormat audioChannelFormatID=\"AC_00000001\" audioChannelFormatName=\"Object\"><audioBlockFormat><position coordinate=\"X\">0</position><position coordinate=\"Y\">0</position></audioBlockFormat></audioChannelFormat><audioObject audioObjectID=\"AO_0001\"><audioPackFormatIDRef>AP_00000001</audioPackFormatIDRef><audioTrackUIDRef>ATU_00000001</audioTrackUIDRef></audioObject></audioFormatExtended>"
        .to_owned()
}
