// pattern: Imperative Shell
use jocforge_domain::frontend::selected_adm_graph::{SelectedTrackKind, parse_selected_adm_graph};
use jocforge_domain::spatial::bed_source_points::selected_bed_source_points;
use serde_json::Value;

const AXML: &[u8] = include_bytes!("fixtures/selected_adm_graph/source_metadata.xml");
const CHNA: &[u8] = include_bytes!("fixtures/selected_adm_graph/source_track_associations.bin");
const SOURCE_BOUNDARIES: &str =
    include_str!("fixtures/selected_adm_graph/SOURCE-FIELD-BOUNDARY-VECTORS.json");

#[test]
fn candidate_bed_role_points_match_eight_same_source_native_field_inputs() {
    let graph = parse_selected_adm_graph(AXML, CHNA).unwrap();
    let vectors: Value = serde_json::from_str(SOURCE_BOUNDARIES).unwrap();
    let cases = vectors["cases"].as_array().unwrap();
    let points = selected_bed_source_points();
    assert_eq!(points.len(), 9);
    assert_eq!(vectors["bed_literal_cases"].as_u64(), Some(72));

    for point in points {
        let graph_row = graph
            .tracks()
            .iter()
            .find(|track| {
                matches!(track.kind(), SelectedTrackKind::DirectSpeakersBed { bed_id, .. } if bed_id == point.bed_id())
            })
            .unwrap();
        let host = graph_row.physical_host().value();
        let host_cases = cases
            .iter()
            .filter(|case| case["physical_host"].as_u64() == Some(u64::from(host)))
            .collect::<Vec<_>>();
        assert_eq!(host_cases.len(), 8, "bed_id {}", point.bed_id());
        for case in host_cases {
            let words: [u32; 13] = case["source_defined_words_4_to_55"]
                .as_array()
                .unwrap()
                .iter()
                .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            assert_eq!(
                point.defined_metadata_words(),
                words,
                "bed_id {}",
                point.bed_id()
            );
        }
    }
}
