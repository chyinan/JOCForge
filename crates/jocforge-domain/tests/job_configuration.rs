// pattern: Imperative Shell
use jocforge_domain::frontend::bed_channel_join::{PhysicalBedTrack, join_selected_bed_channels};
use jocforge_domain::frontend::dbmd_sections::parse_selected_dbmd_sections;
use jocforge_domain::frontend::job_configuration::{
    SelectedInputKind, SelectedWaveDescriptor, parse_selected_job_configuration,
};
use jocforge_domain::profile::EmdfCarriageProfile;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/job_configuration";
const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const VECTOR_SHA256: &str = "493eb904467d22f9ca6dc50b86bf05bcd9f6643f16fa352766cac26d15e156eb";

fn fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(FIXTURE_DIRECTORY)
            .join(name),
    )
    .unwrap()
}

fn fixture_json(name: &str) -> Value {
    serde_json::from_slice(&fixture_bytes(name)).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn decode_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn selected_job_configuration_vectors_are_hash_bound_to_the_approved_package() {
    let vectors = fixture_bytes("job-configuration-vectors.json");
    let provenance = fixture_json("PROVENANCE.json");
    assert_eq!(sha256(&vectors), VECTOR_SHA256);
    assert_eq!(
        provenance["source_package_manifest_sha256"],
        PACKAGE_MANIFEST_SHA256
    );
    assert_eq!(provenance["source_sha256"], VECTOR_SHA256);
    assert_eq!(provenance["local_sha256"], VECTOR_SHA256);
    assert_eq!(
        provenance["scope"],
        "Selected F01E.1 job XML normalization and selected profile gate only. No file I/O, ADM graph/CHNA parsing, DBMD, object event/control rows, or whole F01E/F01 claim."
    );
}

#[test]
fn selected_job_configuration_normalizes_three_accepted_inputs_and_rejects_eight_cases() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let cases = vectors["config_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 11);
    let mut accepted_count = 0_u32;
    let mut rejected_count = 0_u32;

    for case in cases {
        let descriptor = &case["pcm_descriptor"];
        let wave = SelectedWaveDescriptor::new(
            descriptor["sample_rate"].as_u64().unwrap() as u32,
            descriptor["channels"].as_u64().unwrap() as u16,
            descriptor["pcm_bits"].as_u64().unwrap() as u16,
        );
        let source_samples = descriptor["source_samples"].as_u64().unwrap();
        let result = parse_selected_job_configuration(
            case["input_xml"].as_str().unwrap().as_bytes(),
            wave,
            source_samples,
        );
        let expected = &case["expected"];

        if expected["status"] == "ACCEPT" {
            let config = result.unwrap_or_else(|error| panic!("{}: {error:?}", case["id"]));
            accepted_count += 1;
            assert_eq!(
                config.input_kind(),
                match expected["input_kind"].as_str().unwrap() {
                    "adm" => SelectedInputKind::Adm,
                    "atmos_mezz" => SelectedInputKind::AtmosMezz,
                    other => panic!("unexpected input kind {other}"),
                },
                "{}",
                case["id"]
            );
            assert_eq!(
                config.bit_rate_kbps(),
                expected["bit_rate_kbps"].as_u64().unwrap() as u32
            );
            assert_eq!(
                config.bit_rate_enum(),
                expected["bit_rate_enum"].as_u64().unwrap() as u8
            );
            assert_eq!(
                config.layout_enum(),
                expected["layout_enum"].as_u64().unwrap() as u8
            );
            assert_eq!(
                config.target_capacity(),
                expected["target_capacity"].as_u64().unwrap() as u8
            );
            assert_eq!(config.sample_length_enum(), 1);
            assert_eq!(
                config.frame_samples(),
                expected["frame_samples"].as_u64().unwrap() as u16
            );
            assert_eq!(
                config.spatial_config_mode(),
                expected["spatial_config_mode"].as_u64().unwrap() as u8
            );
            assert_eq!(
                config.dispatch_mode(),
                expected["dispatch_mode"].as_u64().unwrap() as u8
            );
            assert_eq!(
                config.route_virtual_prefix_count(),
                expected["route_virtual_prefix_count"].as_u64().unwrap() as u8
            );
            assert_eq!(config.wave(), wave);
            assert_eq!(config.source_samples(), source_samples);
            assert_eq!(
                config.selected_profile().carriage(),
                EmdfCarriageProfile::DeployedCompat
            );
            assert_eq!(config.loudness().metering_mode(), "1770-4");
            assert!(!config.loudness().dialogue_intelligence());
            assert_eq!(config.loudness().speech_threshold(), 100);
            assert_eq!(config.line_drc_profile(), "none");
            assert_eq!(config.rf_drc_profile(), "none");
            assert_eq!(config.downmix().preferred_mode(), "loro");
            assert_eq!(config.downmix().mix_levels(), [-3; 4]);
            assert_eq!(config.surround_trim_5_1(), "auto");
            assert_eq!(config.height_trim_5_1(), "auto");
            assert_eq!(config.time_base(), "file_position");
            assert_eq!(config.prepend_silence_duration(), "0.0");
            assert_eq!(config.append_silence_duration(), "0.0");
            assert_eq!(
                config.input_reference().file_name(),
                if case["id"] == "adm_precedes_mezz" {
                    "chosen.wav"
                } else {
                    "input.wav"
                }
            );
            assert_eq!(config.output_reference().file_name(), "out.ec3");
            assert_eq!(config.input_offset(), "auto");
            assert_eq!(config.input_ffoa(), "auto");
            assert_eq!(config.start(), "");
            assert_eq!(config.end(), "");
        } else {
            let error = result.expect_err(case["id"].as_str().unwrap());
            rejected_count += 1;
            assert_eq!(
                error.code(),
                expected["error"].as_str().unwrap(),
                "{}",
                case["id"]
            );
            assert_eq!(expected["state_commit"], false);
        }
    }
    assert_eq!(accepted_count, 3);
    assert_eq!(rejected_count, 8);
}

#[test]
fn selected_dbmd_sections_decode_authored_mode_and_physical_track_fields() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let mode_cases = vectors["dbmd_mode_cases"].as_array().unwrap();
    assert_eq!(mode_cases.len(), 4);

    for case in mode_cases {
        let dbmd_chunk = decode_hex(case["dbmd_chunk_hex"].as_str().unwrap());
        let parsed = parse_selected_dbmd_sections(&dbmd_chunk).unwrap();
        let expected = &case["expected"];
        assert_eq!(
            parsed.warm_trim_mode(),
            expected["warm_trim_mode"].as_u64().unwrap() as u8
        );
        assert_eq!(
            parsed.authored_source_mode_byte(),
            expected["authored_source_mode_byte"].as_u64().unwrap() as u8
        );
        assert_eq!(
            parsed.surround_ex_mode(),
            expected["surround_ex_mode"].as_u64().unwrap() as u8
        );
        assert_eq!(
            parsed.track_fields().len(),
            expected["track_count"].as_u64().unwrap() as usize
        );
    }

    let packed_cases = vectors["dbmd_track_packed_cases"].as_array().unwrap();
    assert_eq!(packed_cases.len(), 6);
    for case in packed_cases {
        let dbmd_chunk = decode_hex(case["dbmd_chunk_hex"].as_str().unwrap());
        let parsed = parse_selected_dbmd_sections(&dbmd_chunk).unwrap();
        let expected = &case["expected_track"];
        let host_channel_index = expected["host_channel_index"].as_u64().unwrap() as u16;
        let track = parsed.fields_for_physical_host(host_channel_index).unwrap();
        assert_eq!(
            u64::from(track.track_index()),
            expected["host_channel_index"].as_u64().unwrap()
        );
        assert_eq!(
            i64::from(track.enum48()),
            expected["enum48"].as_i64().unwrap()
        );
        assert_eq!(
            i64::from(track.enum52()),
            expected["enum52"].as_i64().unwrap()
        );
        assert_eq!(
            u64::from(track.flag64()),
            expected["flag64"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(track.raw_flag_low2()),
            expected["raw_flag_low2"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(track.raw_packed()),
            expected["raw_packed"].as_u64().unwrap()
        );
    }
}

#[test]
fn selected_dbmd_fail_stop_cases_return_the_mapped_error_without_partial_output() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let cases = vectors["dbmd_malformed_fail_stop_cases"]
        .as_array()
        .unwrap();
    assert_eq!(cases.len(), 10);

    for case in cases {
        let dbmd_chunk = decode_hex(case["dbmd_chunk_hex"].as_str().unwrap());
        let error = parse_selected_dbmd_sections(&dbmd_chunk).unwrap_err();
        assert_eq!(
            error.code(),
            case["expected"]["error"].as_str().unwrap(),
            "{}",
            case["id"]
        );
        assert_eq!(case["expected"]["state_commit"], false);
        assert_eq!(case["expected"]["partial_output"], Value::Null);
    }
}

#[test]
fn selected_bed_pack_order_joins_by_channel_format_identity_to_physical_hosts() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let cases = vectors["bed_join_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 4);

    for case in cases {
        let pack_ids = case["pack_channel_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        let physical_tracks = case["physical_bed_tracks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|track| {
                PhysicalBedTrack::new(
                    track["channel_format_id"].as_str().unwrap().to_owned(),
                    track["host_channel_index"].as_u64().unwrap() as u8,
                    track["semantic_role"].as_str().unwrap().to_owned(),
                    track["bed_id"].as_u64().unwrap() as u8,
                )
            })
            .collect::<Vec<_>>();
        let object_hosts = case["object_host_slots"]
            .as_array()
            .unwrap()
            .iter()
            .map(|host| host.as_u64().unwrap() as u8)
            .collect::<Vec<_>>();
        let result = join_selected_bed_channels(&pack_ids, &physical_tracks, &object_hosts);
        let expected = &case["expected"];

        if expected["status"] == "ACCEPT" {
            let actual = result.unwrap_or_else(|error| panic!("{}: {error:?}", case["id"]));
            let expected_pack_slots = expected["slots_by_pack"]
                .as_array()
                .unwrap()
                .iter()
                .map(|slot| slot.as_u64().unwrap() as u8)
                .collect::<Vec<_>>();
            assert_eq!(actual.slots_by_pack().as_slice(), expected_pack_slots);
            let expected_regular = expected["regular_control_order"]
                .as_array()
                .unwrap()
                .iter()
                .map(|host| host.as_u64().unwrap() as u8)
                .collect::<Vec<_>>();
            assert_eq!(actual.regular_control_order().as_slice(), expected_regular);
            let lfe_host = case["physical_bed_tracks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|track| track["semantic_role"] == "lfe")
                .unwrap()["host_channel_index"]
                .as_u64()
                .unwrap() as u8;
            assert_eq!(actual.lfe_host(), lfe_host);
        } else {
            let error = result.expect_err(case["id"].as_str().unwrap());
            assert_eq!(error.code(), expected["error"].as_str().unwrap());
            assert_eq!(expected["state_commit"], false);
        }
    }
}
