// pattern: Imperative Shell
use jocforge_domain::frontend::dbmd_sections::parse_selected_dbmd_sections;
use jocforge_domain::frontend::object_control::build_selected_object_control52;
use jocforge_domain::frontend::selected_adm_graph::{SelectedTrackKind, parse_selected_adm_graph};
use jocforge_domain::timeline::AdmSampleCount;
use serde_json::Value;
use sha2::{Digest, Sha256};

const AXML: &[u8] = include_bytes!("fixtures/selected_adm_graph/source_metadata.xml");
const CHNA: &[u8] = include_bytes!("fixtures/selected_adm_graph/source_track_associations.bin");
const DBMD: &[u8] = include_bytes!("fixtures/selected_adm_graph/source_metadata.dbmd");
const GRAPH_EXPECTED: &str =
    include_str!("fixtures/selected_adm_graph/REAL-GRAPH-TRACK-IDENTITIES.json");
const GRAPH_PROFILE: &str = include_str!("fixtures/selected_adm_graph/INPUT-ONLY-PROFILE.json");
const SOURCE_RECEIPT: &str =
    include_str!("fixtures/selected_adm_graph/SOURCE-METADATA-RECEIPT.json");
const DBMD_PARSER_EXPECTED: &str =
    include_str!("fixtures/selected_adm_graph/DBMD-PARSER-EXPECTED.json");
const DBMD_SOURCE_RECEIPT: &str =
    include_str!("fixtures/selected_adm_graph/DBMD-SOURCE-RECEIPT.json");
const DBMD_PARSER_INPUT: &str = include_str!("fixtures/selected_adm_graph/DBMD-PARSER-INPUT.json");
const SOURCE_FIELD_BOUNDARIES: &str =
    include_str!("fixtures/selected_adm_graph/SOURCE-FIELD-BOUNDARY-VECTORS.json");
const UID_DESCRIPTOR_OBSERVATIONS: &str =
    include_str!("fixtures/selected_adm_graph/NATIVE-UID-DESCRIPTOR-OBSERVATIONS.json");
const UID_LOOKUP_INPUTS: &str =
    include_str!("fixtures/selected_adm_graph/UID-LOOKUP-INPUT-ONLY.json");
const UID_LOOKUP_OBSERVATIONS: &str =
    include_str!("fixtures/selected_adm_graph/UID-LOOKUP-NATIVE-OBSERVATIONS.json");
const SUPPLEMENT_PROVENANCE: &str = include_str!("fixtures/selected_adm_graph/PROVENANCE.json");
const F01E_VALIDATION_FIXTURES: &str =
    include_str!("fixtures/selected_adm_graph/F01E-VALIDATION-FIXTURE-IDENTITIES.json");
const F01E_VALIDATION_PROFILE: &str =
    include_str!("fixtures/selected_adm_graph/F01E-VALIDATION-PROFILE.json");

const AXML_SHA256: &str = "5973f94db1bc995465b33f6d9be7fd55d3e1bd3941130a913c58e7e5cf57bbfc";
const CHNA_SHA256: &str = "f82b14e4c1b5b3b1750ed4104aa53f1d595e3ff2d4dbaef76a76c0e35f31b7a6";
const DBMD_SHA256: &str = "bc3f7199da679b43d1fd6f770c2caae08250d6ee545858e759fc12afc2232194";
const GRAPH_EXPECTED_SHA256: &str =
    "f573a280401ba43a312bea65f793899965bdca62800c1e716677701c1bca7269";
const GRAPH_PROFILE_SHA256: &str =
    "1c9d6ae0e4437eab2a1ee1587f37d8dfe620bf4876d822eaa47eda04d41fa822";
const SOURCE_RECEIPT_SHA256: &str =
    "b284151464f0174d8e10a178991209cc7d4d28174e027116975ec5a12f7c957e";
const DBMD_PARSER_EXPECTED_SHA256: &str =
    "dafc7ab91530da6716c20a239ba1e4a510458a0fb8208311c84c7eb7047e7075";
const DBMD_SOURCE_RECEIPT_SHA256: &str =
    "bbb780669d0bf9c112dbb8f8a60f12807f12d9c7562a61809917adeaff1d14e9";
const DBMD_PARSER_INPUT_SHA256: &str =
    "53e5f89e37f3a99175bd54e1e91ac8613b3b492b88244d9062ae5ce0d60e14ee";
const SOURCE_FIELD_BOUNDARIES_SHA256: &str =
    "ebba56bc0228b8ce6cc7aa218106c7304222ceb66bb1e73a1bc8d830fcedab43";
const UID_DESCRIPTOR_OBSERVATIONS_SHA256: &str =
    "ae6223a3e8f6c0976f04f5c1ef4c839d08b2074201b7177ee83b52d15c445be9";
const UID_LOOKUP_INPUTS_SHA256: &str =
    "7c98671657db6459af45b02a82f08fe5d32e9f76659363f4d14b2b667fa35468";
const UID_LOOKUP_OBSERVATIONS_SHA256: &str =
    "077c16450bf45d1a60ecd541307f13b32e02723f2244455afbe488c7c42f7111";
const SUPPLEMENT_PROVENANCE_SHA256: &str =
    "6a00fdda59d66f78cc992ba6731c9560f41fc7d98d10888e562bd220f99ef776";
const F01E_VALIDATION_FIXTURES_SHA256: &str =
    "551c8202aaae33efbad0cc2fa47cecb2908df1f2d7436dac92186482cf154fc9";
const F01E_VALIDATION_PROFILE_SHA256: &str =
    "5d0fa84a3bf01182f933a066d59c9db874bff4dc84b51755061e85e60ece6ebf";

#[test]
fn selected_actual_axml_chna_produce_the_38_track_identity_graph() {
    assert_eq!(hex(&Sha256::digest(AXML)), AXML_SHA256);
    assert_eq!(hex(&Sha256::digest(CHNA)), CHNA_SHA256);
    assert_eq!(hex(&Sha256::digest(DBMD)), DBMD_SHA256);
    assert_bound_fixture_hashes();

    let profile: Value = serde_json::from_str(GRAPH_PROFILE).unwrap();
    assert_eq!(profile["source_count"].as_u64(), Some(11));
    assert_eq!(profile["renderer_window_samples"].as_u64(), Some(3_072));
    let source_receipt: Value = serde_json::from_str(SOURCE_RECEIPT).unwrap();
    assert_eq!(source_receipt["sample_rate"].as_u64(), Some(48_000));
    assert_eq!(source_receipt["physical_channels"].as_u64(), Some(38));
    assert_eq!(source_receipt["pcm_bits"].as_u64(), Some(24));
    assert!(source_receipt["metadata_reads_only"].as_bool().unwrap());
    assert_eq!(source_receipt["pcm_data_bytes_read"].as_u64(), Some(0));
    assert!(
        !source_receipt["pcm_content_digest_computed"]
            .as_bool()
            .unwrap()
    );
    assert_eq!(source_receipt["chunks"][3]["metadata_sha256"], AXML_SHA256);
    assert_eq!(source_receipt["chunks"][4]["metadata_sha256"], CHNA_SHA256);

    let graph = parse_selected_adm_graph(AXML, CHNA).unwrap();
    let expected: Value = serde_json::from_str(GRAPH_EXPECTED).unwrap();
    let expected_rows = expected["rows"].as_array().unwrap();
    assert_eq!(graph.tracks().len(), 38);
    assert_eq!(expected_rows.len(), graph.tracks().len());

    for (track, expected) in graph.tracks().iter().zip(expected_rows) {
        assert_eq!(
            u64::from(track.physical_host().value()),
            expected["physical_host"]
        );
        assert_eq!(
            u64::from(track.track_index().value()),
            expected["track_index"]
        );
        assert_eq!(track.uid(), expected["uid"].as_str().unwrap());
        assert_eq!(
            track.track_format(),
            expected["track_format"].as_str().unwrap()
        );
        assert_eq!(
            track.stream_format(),
            expected["stream_format"].as_str().unwrap()
        );
        assert_eq!(
            track.channel_format(),
            expected["channel_format"].as_str().unwrap()
        );
        assert_eq!(
            track.pack_format(),
            expected["pack_format"].as_str().unwrap()
        );
        assert_eq!(track.object_id(), expected["object_id"].as_str().unwrap());
        assert_eq!(
            u64::from(track.descriptor_semantic_key()),
            expected["object_semantic_key"]
        );
        assert_eq!(
            u64::from(track.block_count()),
            expected["block_count"].as_u64().unwrap()
        );
        match track.kind() {
            SelectedTrackKind::DirectSpeakersBed {
                bed_id,
                bed_pack_ordinal,
                ..
            } => {
                assert_eq!(expected["kind"], "DirectSpeakers");
                assert_eq!(u64::from(bed_id), expected["bed_id"]);
                assert_eq!(u64::from(bed_pack_ordinal), expected["bed_pack_ordinal"]);
            }
            SelectedTrackKind::Object { semantic_key } => {
                assert_eq!(expected["kind"], "Objects");
                assert!(expected["bed_id"].is_null());
                assert!(expected["bed_pack_ordinal"].is_null());
                assert_eq!(u64::from(semantic_key), expected["object_semantic_key"]);
            }
        }
    }

    assert_eq!(graph.receipt().physical_track_count(), 38);
    assert_eq!(graph.receipt().bed_track_count(), 10);
    assert_eq!(graph.receipt().object_track_count(), 28);
    assert_eq!(graph.receipt().direct_lfe_host().value(), 3);
    assert_eq!(graph.receipt().virtual_prefix_source_count(), 11);
    assert_eq!(graph.receipt().total_source_count(), 48);
    assert_eq!(graph.bed_channel_join().lfe_host(), 3);
    let expected_object_hosts = expected["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["kind"] == "Objects")
        .map(|row| u8::try_from(row["physical_host"].as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        graph.bed_channel_join().object_hosts().as_slice(),
        expected_object_hosts.as_slice()
    );

    for (binding, expected) in graph
        .regular_source_bindings()
        .iter()
        .zip(expected["regular_source_bindings"].as_array().unwrap())
    {
        assert_eq!(
            u64::from(binding.physical_host().value()),
            expected["physical_host"]
        );
        assert_eq!(
            binding.global_source().value(),
            expected["global_source"].as_u64().unwrap()
        );
    }
}

#[test]
fn selected_uid_descriptor_and_live_lookup_observations_bind_physical_rows() {
    let graph = parse_selected_adm_graph(AXML, CHNA).unwrap();
    let native: Value = serde_json::from_str(UID_DESCRIPTOR_OBSERVATIONS).unwrap();
    let events = native["events"].as_array().unwrap();
    let first_load = events
        .iter()
        .filter(|event| event["load_invocation"].as_u64() == Some(1))
        .collect::<Vec<_>>();
    let lookups = first_load
        .iter()
        .filter(|event| event["event"] == "native_channel_track_lookup")
        .collect::<Vec<_>>();
    assert_eq!(lookups.len(), 38);
    for observation in lookups {
        let uid = observation["track_uid"].as_str().unwrap();
        let track = graph
            .tracks()
            .iter()
            .find(|track| track.uid() == uid)
            .unwrap();
        assert_eq!(
            u64::from(track.track_index().value()),
            observation["track_index_1based"]
        );
        assert_eq!(graph.track_index_for_uid(uid), Some(track.track_index()));
    }

    let descriptor_events = first_load
        .iter()
        .filter(|event| {
            event["event"] == "native_bed_channel_construct"
                || event["event"] == "native_object_channel_construct"
        })
        .collect::<Vec<_>>();
    assert_eq!(descriptor_events.len(), 38);
    for observation in descriptor_events {
        let track = graph
            .tracks()
            .iter()
            .find(|track| track.uid() == observation["track_uid"].as_str().unwrap())
            .unwrap();
        assert_eq!(
            u64::from(track.descriptor_semantic_key()),
            observation["installed_key"]
        );
        assert_eq!(
            u64::from(track.physical_host().value()),
            observation["installed_host"]
        );
    }

    let inputs: Value = serde_json::from_str(UID_LOOKUP_INPUTS).unwrap();
    let outputs: Value = serde_json::from_str(UID_LOOKUP_OBSERVATIONS).unwrap();
    let input_cases = inputs["cases"].as_array().unwrap();
    let output_cases = outputs["cases"].as_array().unwrap();
    assert_eq!(input_cases.len(), 5);
    assert_eq!(output_cases.len(), input_cases.len());
    for (input, output) in input_cases.iter().zip(output_cases) {
        let result = graph.track_index_for_uid(input["uid"].as_str().unwrap());
        match output["return_u8"].as_u64().unwrap() {
            0 => assert_eq!(result, None),
            1 => assert_eq!(
                result.map(|index| u64::from(index.value())),
                output["output_after"].as_u64()
            ),
            other => panic!("unexpected washed lookup result {other}"),
        }
        assert!(output["input_string_preserved"].as_bool().unwrap());
    }
}

#[test]
fn actual_axml_object_events_admit_one_complete_block_and_match_native_source_boundary_words() {
    let graph = parse_selected_adm_graph(AXML, CHNA).unwrap();
    let source_receipt: Value = serde_json::from_str(SOURCE_RECEIPT).unwrap();
    let source_sample_count = source_receipt["source_frames_from_data_extent"]
        .as_u64()
        .unwrap();
    let source_samples = AdmSampleCount::new(source_sample_count).unwrap();
    let admitted = graph.admit_object_events(source_samples).unwrap();
    let boundaries: Value = serde_json::from_str(SOURCE_FIELD_BOUNDARIES).unwrap();
    let boundary_rows = boundaries["cases"].as_array().unwrap();
    assert_eq!(admitted.events().len(), 28);
    assert_eq!(boundary_rows.len(), 296);

    let mut compared_rows = 0;
    for event in admitted.events() {
        assert_eq!(
            event.track_uid(),
            graph.tracks()[usize::from(event.physical_host())].uid()
        );
        assert_eq!(
            event.object_id(),
            graph.tracks()[usize::from(event.physical_host())].object_id()
        );
        assert_eq!(event.object_start().value(), 0);
        assert_eq!(event.block_rtime().value(), 0);
        assert_eq!(event.interpolation_length().value(), 0);
        assert_eq!(event.object_duration().value(), source_sample_count);
        assert_eq!(event.block_duration().value(), source_sample_count);

        let row = build_selected_object_control52(*event.control_event()).unwrap();
        let native_consumptions = boundary_rows
            .iter()
            .filter(|native| {
                native["physical_host"].as_u64() == Some(u64::from(event.physical_host()))
            })
            .collect::<Vec<_>>();
        assert_eq!(native_consumptions.len(), 8);
        for native in native_consumptions {
            let expected_words: [u32; 13] = native["control52_words"]
                .as_array()
                .unwrap()
                .iter()
                .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            assert_eq!(
                row.words(),
                expected_words,
                "invocation {}",
                native["invocation"]
            );
            compared_rows += 1;
        }
    }
    assert_eq!(compared_rows, 224);
}

#[test]
fn candidate_graph_events_join_same_source_dbmd_parser_and_control_set() {
    let graph = parse_selected_adm_graph(AXML, CHNA).unwrap();
    let source_receipt: Value = serde_json::from_str(SOURCE_RECEIPT).unwrap();
    let source_sample_count = source_receipt["source_frames_from_data_extent"]
        .as_u64()
        .unwrap();
    let source_samples = AdmSampleCount::new(source_sample_count).unwrap();
    let dbmd_source: Value = serde_json::from_str(DBMD_SOURCE_RECEIPT).unwrap();
    let dbmd_input: Value = serde_json::from_str(DBMD_PARSER_INPUT).unwrap();
    assert_eq!(dbmd_source["dbmd_bytes"].as_u64(), Some(580));
    assert_eq!(dbmd_source["dbmd_sha256"], DBMD_SHA256);
    assert_eq!(dbmd_input["file"], "source_metadata.dbmd");
    assert_eq!(dbmd_input["bytes"].as_u64(), Some(580));
    assert_eq!(dbmd_input["sha256"], DBMD_SHA256);
    assert_eq!(
        dbmd_source["source_graph_receipt_sha256"],
        hex(&Sha256::digest(SOURCE_RECEIPT.as_bytes()))
    );
    assert_eq!(dbmd_source["pcm_bytes_read"].as_u64(), Some(0));

    let dbmd = parse_selected_dbmd_sections(DBMD).unwrap();
    let expected_dbmd: Value = serde_json::from_str(DBMD_PARSER_EXPECTED).unwrap();
    assert_eq!(expected_dbmd["track_count"].as_u64(), Some(38));
    assert_eq!(dbmd.track_fields().len(), 38);
    assert_eq!(dbmd.warm_trim_mode(), 3);
    assert_eq!(dbmd.authored_source_mode_byte(), 4);
    assert_eq!(dbmd.surround_ex_mode(), 0);
    for expected in expected_dbmd["tracks"].as_array().unwrap() {
        let host = u16::try_from(expected["track_index_zero_based"].as_u64().unwrap()).unwrap();
        let actual = dbmd.fields_for_physical_host(host).unwrap();
        assert_eq!(
            u64::from(actual.track_index()),
            expected["track_index_zero_based"]
        );
        assert_eq!(u64::from(actual.raw_flag_low2()), expected["raw_flag"]);
        assert_eq!(u64::from(actual.raw_packed()), expected["raw_packed"]);
        assert_eq!(i64::from(actual.enum48()), expected["enum48"]);
        assert_eq!(i64::from(actual.enum52()), expected["enum52"]);
        assert_eq!(u64::from(actual.flag64()), expected["flag64"]);
    }

    let admitted = graph.admit_object_events(source_samples).unwrap();
    let controls = graph
        .build_candidate_object_controls(source_samples, &dbmd)
        .unwrap();
    let source_fields: Value = serde_json::from_str(SOURCE_FIELD_BOUNDARIES).unwrap();
    let source_cases = source_fields["cases"].as_array().unwrap();
    let object_hosts = graph.bed_channel_join().object_hosts();
    assert_eq!(controls.rows().len(), 28);
    assert_eq!(controls.dbmd_tracks().len(), 28);
    assert_eq!(admitted.events().len(), controls.rows().len());

    for (index, (event, (control, dbmd_track))) in admitted
        .events()
        .iter()
        .zip(controls.rows().iter().zip(controls.dbmd_tracks()))
        .enumerate()
    {
        let host = object_hosts[index];
        assert_eq!(event.physical_host(), host);
        assert_eq!(u16::from(host), dbmd_track.track_index());
        assert_eq!(
            dbmd_track,
            &dbmd.fields_for_physical_host(u16::from(host)).unwrap()
        );
        let boundary_rows = source_cases
            .iter()
            .filter(|row| row["physical_host"].as_u64() == Some(u64::from(host)))
            .collect::<Vec<_>>();
        assert_eq!(boundary_rows.len(), 8);
        for boundary in boundary_rows {
            let boundary_words: [u32; 13] = boundary["control52_words"]
                .as_array()
                .unwrap()
                .iter()
                .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            assert_eq!(control.words(), boundary_words);
        }
    }
}

#[test]
fn candidate_graph_source_binding_to_main_f01e_job_remains_unresolved() {
    let candidate: Value = serde_json::from_str(SOURCE_RECEIPT).unwrap();
    let f01e_profile: Value = serde_json::from_str(F01E_VALIDATION_PROFILE).unwrap();
    let f01e_fixtures: Value = serde_json::from_str(F01E_VALIDATION_FIXTURES).unwrap();
    let f01e_input = f01e_fixtures
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["role"] == "validation_only")
        .unwrap();

    assert_eq!(candidate["source_file_name"], "hanabi.wav");
    assert_eq!(
        candidate["source_frames_from_data_extent"].as_u64(),
        Some(13_888_071)
    );
    assert_eq!(candidate["source_file_bytes"].as_u64(), Some(1_583_325_474));
    assert_eq!(
        f01e_profile["input"]["file"],
        "test-vectors/full-song/input-adm.wav"
    );
    assert_eq!(
        f01e_profile["input"]["source_samples_per_channel"].as_u64(),
        Some(6_522_906)
    );
    assert_eq!(f01e_input["file"], f01e_profile["input"]["file"]);
    assert_eq!(f01e_input["bytes"].as_u64(), Some(743_696_526));
    assert_eq!(f01e_input["fresh_verification"], "PASS");
    assert_eq!(f01e_input["role"], "validation_only");
    assert_ne!(candidate["source_file_name"], f01e_input["file"]);
    assert_ne!(
        candidate["source_frames_from_data_extent"],
        f01e_profile["input"]["source_samples_per_channel"]
    );
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut result, "{byte:02x}").unwrap();
    }
    result
}

fn assert_bound_fixture_hashes() {
    assert_eq!(
        hex(&Sha256::digest(GRAPH_EXPECTED.as_bytes())),
        GRAPH_EXPECTED_SHA256
    );
    assert_eq!(
        hex(&Sha256::digest(GRAPH_PROFILE.as_bytes())),
        GRAPH_PROFILE_SHA256
    );
    assert_eq!(
        hex(&Sha256::digest(SOURCE_RECEIPT.as_bytes())),
        SOURCE_RECEIPT_SHA256
    );
    assert_eq!(
        hex(&Sha256::digest(DBMD_PARSER_EXPECTED.as_bytes())),
        DBMD_PARSER_EXPECTED_SHA256
    );
    assert_eq!(
        hex(&Sha256::digest(DBMD_SOURCE_RECEIPT.as_bytes())),
        DBMD_SOURCE_RECEIPT_SHA256
    );
    assert_eq!(
        hex(&Sha256::digest(DBMD_PARSER_INPUT.as_bytes())),
        DBMD_PARSER_INPUT_SHA256
    );
    assert_eq!(
        hex(&Sha256::digest(SOURCE_FIELD_BOUNDARIES.as_bytes())),
        SOURCE_FIELD_BOUNDARIES_SHA256
    );
    assert_eq!(
        hex(&Sha256::digest(UID_DESCRIPTOR_OBSERVATIONS.as_bytes())),
        UID_DESCRIPTOR_OBSERVATIONS_SHA256
    );
    assert_eq!(
        hex(&Sha256::digest(UID_LOOKUP_INPUTS.as_bytes())),
        UID_LOOKUP_INPUTS_SHA256
    );
    assert_eq!(
        hex(&Sha256::digest(UID_LOOKUP_OBSERVATIONS.as_bytes())),
        UID_LOOKUP_OBSERVATIONS_SHA256
    );
    assert_eq!(
        hex(&Sha256::digest(SUPPLEMENT_PROVENANCE.as_bytes())),
        SUPPLEMENT_PROVENANCE_SHA256
    );
    assert_eq!(
        hex(&Sha256::digest(F01E_VALIDATION_FIXTURES.as_bytes())),
        F01E_VALIDATION_FIXTURES_SHA256
    );
    assert_eq!(
        hex(&Sha256::digest(F01E_VALIDATION_PROFILE.as_bytes())),
        F01E_VALIDATION_PROFILE_SHA256
    );
}
