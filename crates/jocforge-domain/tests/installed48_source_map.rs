// pattern: Imperative Shell
use jocforge_domain::frontend::dbmd_sections::parse_selected_dbmd_sections;
use jocforge_domain::frontend::job_configuration::bind_selected_primary_control_projection;
use jocforge_domain::frontend::primary_metadata_control::produce_primary_metadata_control_rows;
use jocforge_domain::frontend::selected_adm_graph::parse_selected_adm_graph;
use jocforge_domain::spatial::installed48_source_map::{
    Installed48SourceOwner, PrimaryPrefixWriterSet, build_primary_installed48_source_map,
};
use jocforge_domain::spatial::prefix_membership::PrefixMembershipProducer;
use jocforge_domain::spatial::selected_p11_geometry::generate_selected_p11_geometry;
use serde_json::Value;

const AXML: &[u8] = include_bytes!("fixtures/primary_metadata_control/source_metadata.xml");
const CHNA: &[u8] =
    include_bytes!("fixtures/primary_metadata_control/source_track_associations.bin");
const DBMD: &[u8] = include_bytes!("fixtures/primary_metadata_control/source_metadata.dbmd");
const JOB_PROJECTION: &[u8] =
    include_bytes!("fixtures/primary_metadata_control/JOB-FIELD-PROJECTION.json");
const JOB_PROJECTION_RECEIPT: &[u8] =
    include_bytes!("fixtures/primary_metadata_control/JOB-PROJECTION-RECEIPT.json");
const FROZEN_JOB_BINDING: &[u8] =
    include_bytes!("fixtures/primary_metadata_control/FROZEN-JOB-BINDING.json");
const PRIMARY_SOURCE_RECEIPT: &[u8] =
    include_bytes!("fixtures/primary_metadata_control/PRIMARY-SOURCE-RECEIPT.json");
const SOURCE_MAP_EXPECTED: &str =
    include_str!("fixtures/primary_installed48/INSTALLED48-SOURCE-MAP.json");
const POSTROUTE_EXPECTED: &str =
    include_str!("fixtures/primary_installed48/POSTROUTE-VARIANTS.json");

#[test]
fn primary_graph_and_p11_generate_the_48_source_map() {
    let source_map = primary_source_map();

    // This is a post-computation comparison against the sealed source-map receipt.
    let expected: Value = serde_json::from_str(SOURCE_MAP_EXPECTED).unwrap();
    assert_eq!(source_map.rows().len(), 48);
    assert_eq!(expected["source_count"].as_u64(), Some(48));
    assert_eq!(expected["prefix_count"].as_u64(), Some(11));
    assert_eq!(expected["regular_count"].as_u64(), Some(37));
    assert_eq!(expected["direct_lfe_host"].as_u64(), Some(3));
    let expected_rows = expected["rows"].as_array().unwrap();
    assert_eq!(expected_rows.len(), source_map.rows().len());

    for (row, expected_row) in source_map.rows().iter().zip(expected_rows) {
        assert_eq!(
            row.source_index().value(),
            expected_row["source_index"].as_u64().unwrap()
        );
        assert_eq!(row.kind(), expected_row["kind"].as_str().unwrap());
        assert_eq!(
            *row.metadata_words_8_to_55(),
            expected_words(&expected_row["metadata_words_8_to_55"])
        );
        assert_eq!(
            row.packed_attribute(),
            expected_row["packed_attribute"].as_u64().unwrap() as u32
        );
        match row.owner() {
            Installed48SourceOwner::P11 { layout_ordinal, .. } => {
                assert_eq!(expected_row["owner"]["kind"], "P11_geometry");
                assert_eq!(
                    u64::from(layout_ordinal.value()),
                    expected_row["owner"]["layout_ordinal"].as_u64().unwrap()
                );
            }
            Installed48SourceOwner::PhysicalTrack { track, dbmd } => {
                let owner = &expected_row["owner"];
                assert_eq!(
                    u64::from(track.physical_host().value()),
                    owner["physical_host"].as_u64().unwrap()
                );
                assert_eq!(track.uid(), owner["track_uid"].as_str().unwrap());
                assert_eq!(
                    track.track_format(),
                    owner["track_format_id"].as_str().unwrap()
                );
                assert_eq!(
                    track.stream_format(),
                    owner["stream_format_id"].as_str().unwrap()
                );
                assert_eq!(
                    track.channel_format(),
                    owner["channel_format_id"].as_str().unwrap()
                );
                assert_eq!(
                    track.pack_format(),
                    owner["pack_format_id"].as_str().unwrap()
                );
                assert_eq!(track.object_id(), owner["object_id"].as_str().unwrap());
                let dbmd_expected = &owner["dbmd_owner"];
                assert_eq!(
                    dbmd.track_index(),
                    dbmd_expected["physical_track_index"].as_u64().unwrap() as u16
                );
                assert_eq!(
                    dbmd.enum48(),
                    dbmd_expected["enum48"].as_i64().unwrap() as i8
                );
                assert_eq!(
                    dbmd.enum52(),
                    dbmd_expected["enum52"].as_i64().unwrap() as i8
                );
                assert_eq!(
                    dbmd.flag64(),
                    dbmd_expected["flag64"].as_u64().unwrap() as u8
                );
            }
        }
        if expected_row["kind"] == "virtual_prefix" {
            assert_eq!(row.input_mute(), None);
        } else {
            assert_eq!(row.input_mute(), Some(0));
            assert_eq!(row.packed_attribute(), 2_816);
        }
    }

    let expected_physical = expected["physical_to_global_source"].as_array().unwrap();
    for (binding, expected) in source_map
        .physical_to_global_source()
        .iter()
        .zip(expected_physical)
    {
        assert_eq!(
            u64::from(binding.physical_host().value()),
            expected["physical_host"].as_u64().unwrap()
        );
        assert_eq!(
            binding.source_index().value(),
            expected["source_index"].as_u64().unwrap()
        );
    }
}

#[test]
fn selected_postroute_variants_match_all_defined_bytes() {
    let source_map = primary_source_map();
    let expected: Value = serde_json::from_str(POSTROUTE_EXPECTED).unwrap();
    let variants = expected["variants"].as_array().unwrap();
    assert_eq!(variants.len(), 96);
    for source in 0_u8..11 {
        for written in [false, true] {
            let writers = if written {
                PrimaryPrefixWriterSet::from_positive_members(&[source]).unwrap()
            } else {
                PrimaryPrefixWriterSet::empty()
            };
            let rows = source_map.postroute_rows(&[1; 37], writers).unwrap();
            let variant_id = usize::from(source) * 2 + usize::from(written);
            compare_variant_bytes(&rows[usize::from(source)], &variants[variant_id]);
        }
    }

    for source in 11_u8..48 {
        for quiet in [0_u8, 1] {
            let mut quiet37 = [1_u8; 37];
            quiet37[usize::from(source - 11)] = quiet;
            let rows = source_map
                .postroute_rows(&quiet37, PrimaryPrefixWriterSet::empty())
                .unwrap();
            let variant_id = 22 + usize::from(source - 11) * 2 + usize::from(quiet);
            compare_variant_bytes(&rows[usize::from(source)], &variants[variant_id]);
        }
    }
}

#[test]
fn primary_object_memberships_recompute_full_gain_vectors_before_writer_union() {
    let source_map = primary_source_map();
    let membership =
        PrefixMembershipProducer::for_selected_primary_p11(source_map.p11_geometry()).unwrap();
    let route = source_map
        .produce_primary_prefix_route(&[0; 37], &membership)
        .unwrap();
    let expected_call_count = source_map.rows()[11..]
        .iter()
        .filter(|row| {
            row.kind() == "object"
                && row
                    .control52()
                    .is_some_and(|control| f32::from_bits(control.words()[9]) > 0.0)
        })
        .count();
    assert_eq!(route.contributions().len(), expected_call_count);
    assert!(
        route
            .contributions()
            .windows(2)
            .all(|pair| { pair[0].source_index().value() < pair[1].source_index().value() })
    );
    for contribution in route.contributions() {
        assert_eq!(contribution.result().gain_words().len(), 11);
        assert!(
            contribution
                .result()
                .positive_members()
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        );
    }
    let writers = route.writer_indices();
    assert!(writers.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(writers.iter().all(|writer| *writer < 11));
}

fn primary_source_map() -> jocforge_domain::spatial::installed48_source_map::Installed48SourceMap {
    let binding = bind_selected_primary_control_projection(
        JOB_PROJECTION,
        JOB_PROJECTION_RECEIPT,
        FROZEN_JOB_BINDING,
        PRIMARY_SOURCE_RECEIPT,
    )
    .unwrap();
    let graph = parse_selected_adm_graph(AXML, CHNA).unwrap();
    let dbmd = parse_selected_dbmd_sections(DBMD).unwrap();
    let controls = produce_primary_metadata_control_rows(&binding, &graph, &dbmd).unwrap();
    let geometry = generate_selected_p11_geometry(0x111b, true).unwrap();
    build_primary_installed48_source_map(&controls, &geometry).unwrap()
}

fn compare_variant_bytes(
    row: &jocforge_domain::spatial::installed48_source_map::Installed48PostRouteRow,
    expected: &Value,
) {
    assert_eq!(
        row.source_index().value(),
        expected["source_index"].as_u64().unwrap()
    );
    let actual = row.defined_bytes();
    let expected_bytes = decode_hex(expected["defined_ranges"][0]["hex"].as_str().unwrap());
    assert_eq!(
        actual,
        expected_bytes,
        "source={} variant={}",
        row.source_index().value(),
        expected["variant_id"]
    );
    assert_eq!(
        row.packed_attribute(),
        expected["packed_attribute"].as_u64().unwrap() as u32
    );
}

fn decode_hex(value: &str) -> [u8; 56] {
    assert_eq!(value.len(), 112);
    let mut bytes = [0_u8; 56];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let offset = index * 2;
        *byte = u8::from_str_radix(&value[offset..offset + 2], 16).unwrap();
    }
    bytes
}

fn expected_words(value: &Value) -> [u32; 12] {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|word| word.as_u64().unwrap() as u32)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
