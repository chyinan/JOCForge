// pattern: Imperative Shell
use std::collections::BTreeMap;
use std::fmt::Write as _;

use jocforge_domain::frontend::dbmd_sections::parse_selected_dbmd_sections;
use jocforge_domain::frontend::job_configuration::bind_selected_primary_control_projection;
use jocforge_domain::frontend::primary_metadata_control::{
    PrimaryControl38Fields, PrimaryMetadataControlError, produce_primary_metadata_control_rows,
};
use jocforge_domain::frontend::selected_adm_graph::parse_selected_adm_graph;
use jocforge_domain::spatial::source_flags::{
    SourceControlSnapshotClock, SourceFlagsProducer, project_primary_regular_source_controls,
};
use jocforge_domain::timeline::{FrontendCallbackIndex, NormalCallbackCount};
use serde_json::Value;
use sha2::{Digest, Sha256};

const AXML: &[u8] = include_bytes!("fixtures/primary_metadata_control/source_metadata.xml");
const CHNA: &[u8] =
    include_bytes!("fixtures/primary_metadata_control/source_track_associations.bin");
const DBMD: &[u8] = include_bytes!("fixtures/primary_metadata_control/source_metadata.dbmd");
const PROVENANCE: &str = include_str!("fixtures/primary_metadata_control/PROVENANCE.json");
const JOB_PROJECTION: &[u8] =
    include_bytes!("fixtures/primary_metadata_control/JOB-FIELD-PROJECTION.json");
const JOB_PROJECTION_RECEIPT: &[u8] =
    include_bytes!("fixtures/primary_metadata_control/JOB-PROJECTION-RECEIPT.json");
const FROZEN_JOB_BINDING: &[u8] =
    include_bytes!("fixtures/primary_metadata_control/FROZEN-JOB-BINDING.json");
const PRIMARY_SOURCE_RECEIPT: &[u8] =
    include_bytes!("fixtures/primary_metadata_control/PRIMARY-SOURCE-RECEIPT.json");
const CONTROL38_EXPECTED: &str =
    include_str!("fixtures/primary_metadata_control/MODEL-DEFINED-CONTROL38.json");
const OBJECT_EXPECTED: &str =
    include_str!("fixtures/primary_metadata_control/F01E-EXPECTED-COMPARISON-DATA.json");
const DBMD_EXPECTED: &str =
    include_str!("fixtures/primary_metadata_control/PRIMARY-DBMD-DECODED.json");

#[test]
fn primary_projection_binds_metadata_and_reproduces_defined_control_rows() {
    let provenance: Value = serde_json::from_str(PROVENANCE).unwrap();
    assert_eq!(
        provenance["source_supplement_manifest_sha256"],
        "f210abc9fb0d92f239ebef228cb3c3e15b9f55a820623aecf650b96d537102f4"
    );
    assert_eq!(
        provenance["detached_delivery_envelope_sha256"],
        "a6a3bc2b970cb4e90bc1b28c18744e7742c5e1004331a66ab20dbba257542273"
    );
    assert_eq!(provenance["checker_replay"]["normal"], "PASS");
    assert_eq!(provenance["checker_replay"]["optimized"], "PASS");
    assert_eq!(
        provenance["source_identity"]["full_pcm_identity"],
        "NOT_REHASHED"
    );
    assert_eq!(
        provenance["source_identity"]["original_job_identity"],
        "RESEARCH_DECLARATION_ONLY"
    );
    assert_eq!(
        provenance["scope"],
        "Primary source metadata/control verification only; original XML and PCM bytes were not provided/read. No main encode or end-to-end native run is claimed."
    );

    let binding = bind_selected_primary_control_projection(
        JOB_PROJECTION,
        JOB_PROJECTION_RECEIPT,
        FROZEN_JOB_BINDING,
        PRIMARY_SOURCE_RECEIPT,
    )
    .unwrap();
    assert_eq!(binding.data_bytes() / 114, 6_522_906);
    assert_eq!(binding.source_file_bytes(), 743_696_526);
    assert!(!binding.original_job_bytes_available());
    assert!(!binding.full_pcm_identity_rehashed());
    let projection: Value = serde_json::from_slice(JOB_PROJECTION).unwrap();
    assert_eq!(
        binding.source_file_name(),
        projection["input"]["file_name"].as_str().unwrap()
    );

    let graph = parse_selected_adm_graph(AXML, CHNA).unwrap();
    let dbmd = parse_selected_dbmd_sections(DBMD).unwrap();
    assert_eq!(
        hex(&Sha256::digest(AXML)),
        "7e4e61d4e2bec581ff2b05133281198a220602290597950e301a6079701c1659"
    );
    assert_eq!(
        hex(&Sha256::digest(CHNA)),
        "f82b14e4c1b5b3b1750ed4104aa53f1d595e3ff2d4dbaef76a76c0e35f31b7a6"
    );
    assert_eq!(
        hex(&Sha256::digest(DBMD)),
        "ea961b79063bc68f64b1210dba002a5729d1003427196123f93b0890939b75d9"
    );

    let produced = produce_primary_metadata_control_rows(&binding, &graph, &dbmd).unwrap();
    let control_expected: Value = serde_json::from_str(CONTROL38_EXPECTED).unwrap();
    let object_expected: Value = serde_json::from_str(OBJECT_EXPECTED).unwrap();
    let dbmd_expected: Value = serde_json::from_str(DBMD_EXPECTED).unwrap();
    let expected_objects: BTreeMap<u8, Vec<u32>> = object_expected["object_rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["physical_host"].as_u64().unwrap() as u8,
                row["expected_control_words"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|word| word.as_u64().unwrap() as u32)
                    .collect(),
            )
        })
        .collect();
    let expected_dbmd: BTreeMap<u16, &Value> = dbmd_expected["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["host_channel_index"].as_u64().unwrap() as u16, row))
        .collect();
    let expected_rows = control_expected["rows"].as_array().unwrap();
    assert_eq!(produced.row_count(), 38);
    assert_eq!(produced.object_control_word_count(), 364);
    assert_eq!(produced.defined_control_word_count(), 424);
    assert_eq!(produced.rows().len(), expected_rows.len());
    assert_eq!(expected_rows.len(), 38);

    let mut compared_object_words = 0;
    for (row, expected) in produced.rows().iter().zip(expected_rows) {
        let host = row.track().physical_host().value();
        assert_eq!(u64::from(host), expected["physical_host"].as_u64().unwrap());
        assert_eq!(row.track().uid(), expected["track_uid"].as_str().unwrap());
        assert_eq!(
            row.dbmd().track_index(),
            expected_dbmd[&u16::from(host)]["host_channel_index"]
                .as_u64()
                .unwrap() as u16
        );
        assert_eq!(
            row.dbmd().enum48(),
            expected_dbmd[&u16::from(host)]["enum48"].as_i64().unwrap() as i8
        );
        assert_eq!(
            row.dbmd().enum52(),
            expected_dbmd[&u16::from(host)]["enum52"].as_i64().unwrap() as i8
        );
        assert_eq!(
            row.dbmd().flag64(),
            expected_dbmd[&u16::from(host)]["flag64"].as_u64().unwrap() as u8
        );
        assert_eq!(
            row.dbmd().raw_flag_low2(),
            expected_dbmd[&u16::from(host)]["raw_flag_low2"]
                .as_u64()
                .unwrap() as u8
        );
        assert_eq!(
            row.dbmd().raw_packed(),
            expected_dbmd[&u16::from(host)]["raw_packed"]
                .as_u64()
                .unwrap() as u8
        );

        match row.fields() {
            PrimaryControl38Fields::Bed {
                words,
                unproduced_tail,
            } => {
                assert_eq!(expected["kind"], "DirectSpeakers");
                assert_eq!(expected["defined_u32_words"].as_u64(), Some(6));
                assert_eq!(
                    words_to_le_hex(&words),
                    expected["defined_ranges"][0]["hex"]
                );
                assert_eq!(unproduced_tail.offset(), 24);
                assert_eq!(unproduced_tail.byte_count(), 28);
                assert_eq!(expected["unknown_unconsumed_ranges"][0]["offset"], 24);
                assert_eq!(expected["unknown_unconsumed_ranges"][0]["bytes"], 28);
            }
            PrimaryControl38Fields::Object { control52 } => {
                assert_eq!(expected["kind"], "Objects");
                assert_eq!(expected["defined_u32_words"].as_u64(), Some(13));
                let words = control52.words();
                assert_eq!(words.as_slice(), expected_objects[&host].as_slice());
                compared_object_words += words.len();
            }
        }
    }
    assert_eq!(compared_object_words, 364);
}

#[test]
fn primary_regular_controls_use_role_fields_and_hold_last_normal_origin_on_drain() {
    let binding = bind_selected_primary_control_projection(
        JOB_PROJECTION,
        JOB_PROJECTION_RECEIPT,
        FROZEN_JOB_BINDING,
        PRIMARY_SOURCE_RECEIPT,
    )
    .unwrap();
    let graph = parse_selected_adm_graph(AXML, CHNA).unwrap();
    let dbmd = parse_selected_dbmd_sections(DBMD).unwrap();
    let control_rows = produce_primary_metadata_control_rows(&binding, &graph, &dbmd).unwrap();
    let normal_callbacks = NormalCallbackCount::new(4_247);
    let last_normal = FrontendCallbackIndex::new(4_246);
    let normal_clock =
        SourceControlSnapshotClock::for_normal_callback(last_normal, normal_callbacks).unwrap();
    let projection = project_primary_regular_source_controls(
        &control_rows,
        &graph,
        binding.selected_profile(),
        normal_clock,
    )
    .unwrap();

    assert_eq!(projection.route().regular_hosts().len(), 37);
    assert_eq!(projection.controls().rows().len(), 37);
    assert_eq!(projection.clock().source_origin(), last_normal);
    let direct_lfe_host = graph.receipt().direct_lfe_host().value();
    let regular_rows = control_rows
        .rows()
        .iter()
        .filter(|row| row.track().physical_host().value() != direct_lfe_host)
        .collect::<Vec<_>>();
    assert_eq!(regular_rows.len(), 37);
    let mut distinguished_object_width = false;
    for (source_ordinal, (row, control)) in regular_rows
        .iter()
        .zip(projection.controls().rows())
        .enumerate()
    {
        let host = row.track().physical_host();
        assert_eq!(control.host().value(), host.value());
        let binding = graph
            .regular_source_bindings()
            .iter()
            .find(|item| item.physical_host() == host)
            .unwrap();
        assert_eq!(binding.global_source().value(), 11 + source_ordinal as u64);

        match row.fields() {
            PrimaryControl38Fields::Bed { words, .. } => {
                assert_eq!(control.is_muted(), words[0] == 1);
                assert_eq!(control.width().to_bits(), 0);
            }
            PrimaryControl38Fields::Object { control52 } => {
                let words = control52.words();
                assert_eq!(control.is_muted(), words[0] == 1);
                assert_eq!(control.width().to_bits(), words[9]);
                distinguished_object_width |= words[9] != words[10];
            }
        }
    }
    assert!(distinguished_object_width);

    let current_pcm = vec![vec![0_u32; 1_536]; 38];
    let previous_history = vec![vec![0_u32; 1_536]; 37];
    let prefix_writers = (0..11).collect::<Vec<u8>>();
    let flags = SourceFlagsProducer::produce_meter_values(
        &current_pcm,
        &previous_history,
        projection.controls(),
        projection.route(),
        &prefix_writers,
    )
    .unwrap();
    assert!(flags.payload()[..11].iter().all(|pair| *pair == [0, 0]));
    for (source_ordinal, control) in projection.controls().rows().iter().enumerate() {
        assert_eq!(
            flags.payload()[11 + source_ordinal],
            [1, u8::from(control.is_muted())]
        );
    }

    let drain_callback = FrontendCallbackIndex::new(4_247);
    let drain_clock =
        SourceControlSnapshotClock::for_cached_drain(drain_callback, normal_callbacks).unwrap();
    let drain_projection = project_primary_regular_source_controls(
        &control_rows,
        &graph,
        binding.selected_profile(),
        drain_clock,
    )
    .unwrap();
    assert!(drain_projection.clock().is_cached_drain());
    assert_eq!(drain_projection.clock().callback_index(), drain_callback);
    assert_eq!(drain_projection.clock().source_origin(), last_normal);
    assert_eq!(
        SourceControlSnapshotClock::for_normal_callback(drain_callback, normal_callbacks),
        Err(jocforge_domain::spatial::source_flags::SourceFlagsError::SourceControlClock)
    );
}

#[test]
fn changed_job_projection_identity_is_rejected_before_binding() {
    let mut changed_projection = JOB_PROJECTION.to_vec();
    changed_projection[0] ^= 1;

    let result = bind_selected_primary_control_projection(
        &changed_projection,
        JOB_PROJECTION_RECEIPT,
        FROZEN_JOB_BINDING,
        PRIMARY_SOURCE_RECEIPT,
    );

    assert_eq!(
        result.unwrap_err().code(),
        "PRIMARY_BINDING_ARTIFACT_IDENTITY"
    );
}

#[test]
fn semantically_equal_axml_with_different_hash_is_rejected_by_producer() {
    let binding = bind_selected_primary_control_projection(
        JOB_PROJECTION,
        JOB_PROJECTION_RECEIPT,
        FROZEN_JOB_BINDING,
        PRIMARY_SOURCE_RECEIPT,
    )
    .unwrap();
    let original_graph = parse_selected_adm_graph(AXML, CHNA).unwrap();
    let dbmd = parse_selected_dbmd_sections(DBMD).unwrap();
    let mut axml_with_trailing_whitespace = AXML.to_vec();
    axml_with_trailing_whitespace.extend_from_slice(b" \n");
    let whitespace_graph = parse_selected_adm_graph(&axml_with_trailing_whitespace, CHNA).unwrap();

    assert_eq!(whitespace_graph.tracks(), original_graph.tracks());
    assert_ne!(
        whitespace_graph.receipt().axml_sha256(),
        original_graph.receipt().axml_sha256()
    );
    assert_eq!(
        produce_primary_metadata_control_rows(&binding, &whitespace_graph, &dbmd),
        Err(PrimaryMetadataControlError::SourceMetadataIdentity)
    );
}

fn words_to_le_hex(words: &[u32]) -> String {
    let mut output = String::with_capacity(words.len() * 8);
    for byte in words.iter().flat_map(|word| word.to_le_bytes()) {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}
