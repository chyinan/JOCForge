use jocforge_domain::frontend::adm_file_storage_map::{
    AdmFileStorageMapError, AdmFileStorageTrackSlot, build_adm_file_storage_channel_map,
};
use jocforge_domain::frontend::selected_adm_graph::{SelectedTrackKind, parse_selected_adm_graph};
use serde_json::Value;
use sha2::{Digest, Sha256};

const AXML: &[u8] = include_bytes!("fixtures/selected_adm_graph/source_metadata.xml");
const CHNA: &[u8] = include_bytes!("fixtures/selected_adm_graph/source_track_associations.bin");
const TRACK_EXPECTED: &str =
    include_str!("fixtures/selected_adm_graph/REAL-GRAPH-TRACK-IDENTITIES.json");
const AXML_SHA256: &str = "5973f94db1bc995465b33f6d9be7fd55d3e1bd3941130a913c58e7e5cf57bbfc";
const CHNA_SHA256: &str = "f82b14e4c1b5b3b1750ed4104aa53f1d595e3ff2d4dbaef76a76c0e35f31b7a6";
const TRACK_EXPECTED_SHA256: &str =
    "f573a280401ba43a312bea65f793899965bdca62800c1e716677701c1bca7269";

#[test]
fn selected_graph_projects_chna_slots_and_converter_channel_values_separately() {
    assert_eq!(hex(&Sha256::digest(AXML)), AXML_SHA256);
    assert_eq!(hex(&Sha256::digest(CHNA)), CHNA_SHA256);
    assert_eq!(
        hex(&Sha256::digest(TRACK_EXPECTED.as_bytes())),
        TRACK_EXPECTED_SHA256
    );

    let graph = parse_selected_adm_graph(AXML, CHNA).unwrap();
    let map = build_adm_file_storage_channel_map(&graph).unwrap();
    let expected: Value = serde_json::from_str(TRACK_EXPECTED).unwrap();
    let expected_rows = expected["rows"].as_array().unwrap();
    assert_eq!(graph.tracks().len(), 38);
    assert_eq!(map.entries_by_track_slot().len(), 38);

    for track in graph.tracks() {
        let expected_row = expected_rows
            .iter()
            .find(|row| row["track_index"].as_u64() == Some(u64::from(track.track_index().value())))
            .unwrap();
        let track_slot_value = track.track_index().value() - 1;
        let slot = AdmFileStorageTrackSlot::new(u8::try_from(track_slot_value).unwrap()).unwrap();
        let entry = map.entry_for_slot(slot);
        assert_eq!(entry.track_slot(), slot);
        assert_eq!(
            u64::from(entry.track_slot().value()),
            expected_row["track_index"].as_u64().unwrap() - 1,
        );

        let expected_channel_input_id = match track.kind() {
            SelectedTrackKind::DirectSpeakersBed {
                bed_input_id,
                bed_pack_ordinal,
                ..
            } => {
                assert_eq!(expected_row["kind"], "DirectSpeakers");
                let object_id = expected_row["object_id"].as_str().unwrap();
                let decoded_channel_input_id =
                    u32::from_str_radix(object_id.strip_prefix("AO_").unwrap(), 16).unwrap()
                        - 0x1001;
                assert_eq!(
                    u64::from(bed_pack_ordinal),
                    expected_row["bed_pack_ordinal"]
                );
                assert_eq!(
                    u64::from(bed_input_id),
                    u64::from(decoded_channel_input_id + u32::from(bed_pack_ordinal)),
                );
                assert_eq!(u64::from(bed_input_id), expected_row["object_semantic_key"]);
                bed_input_id
            }
            SelectedTrackKind::Object { semantic_key } => {
                assert_eq!(expected_row["kind"], "Objects");
                assert_eq!(u64::from(semantic_key), expected_row["object_semantic_key"],);
                semantic_key
            }
        };
        assert_eq!(entry.channel_input_id().value(), expected_channel_input_id);
    }

    for (index, entry) in map.entries_by_track_slot().iter().enumerate() {
        assert_eq!(usize::from(entry.track_slot().value()), index);
    }
}

#[test]
fn converter_storage_slots_reject_values_outside_the_selected_track_domain() {
    assert_eq!(
        AdmFileStorageTrackSlot::new(38),
        Err(AdmFileStorageMapError::TrackSlotOutOfRange(38)),
    );
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}
