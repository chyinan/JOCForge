// pattern: Functional Core

use jocforge_core::{generate_synthetic_fixture, parse_bw64_adm};

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
