// pattern: Imperative Shell
use jocforge_domain::frontend::bed_channel_join::{PhysicalBedTrack, join_selected_bed_channels};
use jocforge_domain::frontend::dbmd_sections::parse_selected_dbmd_sections;
use jocforge_domain::frontend::job_configuration::{
    JobConfigurationError, PrimaryMetadataChunkHashes, SelectedDrcProfile, SelectedHeightTrim,
    SelectedInputKind, SelectedPrimaryRuntimeIdentity, SelectedSurroundTrim,
    SelectedWaveDescriptor, bind_admitted_selected_input, parse_selected_job_configuration,
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
fn selected_job_configuration_normalizes_inputs_and_rejects_unsupported_cases() {
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
        let source_xml = case["input_xml"].as_str().unwrap();
        // 768 is now a supported rate. Exercise the local negative gate with an
        // unregistered 512 token without mutating the hash-bound source vector.
        let input_xml = if case["id"] == "reject_other_rate" {
            source_xml.replace("<data_rate>768</data_rate>", "<data_rate>512</data_rate>")
        } else {
            source_xml.to_owned()
        };
        let result = parse_selected_job_configuration(input_xml.as_bytes(), wave, source_samples);
        let expected = &case["expected"];

        if case["id"] == "reject_wrong_pcm_profile" {
            // The source vector predates variable-count ADM input support. Keep its
            // hash-bound provenance while treating its 36-channel rejection as superseded.
            let config = result.expect("36-channel ADM input is now admitted");
            assert_eq!(config.input_kind(), SelectedInputKind::Adm);
            assert_eq!(config.wave().channels(), 36);
            accepted_count += 1;
            continue;
        }

        if case["id"] == "reject_nonzero_start" {
            // The hash-bound vector predates lexical range admission. The parser
            // now preserves the string; the selected CLI effect gate still rejects it.
            let config = result.expect("start text is admitted without applying a range");
            assert_eq!(config.start(), "00:00:01.000000");
            assert_eq!(
                config.normalized_identity_value()["start"],
                "00:00:01.000000"
            );
            accepted_count += 1;
            continue;
        }

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
            assert_eq!(config.line_drc_profile(), SelectedDrcProfile::None);
            assert_eq!(config.rf_drc_profile(), SelectedDrcProfile::None);
            assert_eq!(config.downmix().preferred_mode(), "loro");
            assert_eq!(config.downmix().mix_levels(), [-3; 4]);
            assert_eq!(config.surround_trim_5_1(), SelectedSurroundTrim::Auto);
            assert_eq!(config.height_trim_5_1(), SelectedHeightTrim::Auto);
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
    assert_eq!(accepted_count, 5);
    assert_eq!(rejected_count, 6);
}

#[test]
fn authored_input_timecode_tokens_are_preserved_without_conversion() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "accepted_full_adm")
        .unwrap();
    let descriptor = &case["pcm_descriptor"];
    let wave = SelectedWaveDescriptor::new(
        descriptor["sample_rate"].as_u64().unwrap() as u32,
        descriptor["channels"].as_u64().unwrap() as u16,
        descriptor["pcm_bits"].as_u64().unwrap() as u16,
    );
    let source_samples = descriptor["source_samples"].as_u64().unwrap();
    let base_xml = case["input_xml"].as_str().unwrap();
    let input_field = "<timecode_frame_rate>not_indicated</timecode_frame_rate>";
    let tokens = [
        "not_indicated",
        "23.976",
        "24",
        "25",
        "29.97",
        "30",
        "48",
        "50",
        "59.94",
        "60",
    ];

    for token in tokens {
        let replacement = format!("<timecode_frame_rate>{token}</timecode_frame_rate>");
        let authored_xml = base_xml.replacen(input_field, &replacement, 1);
        let configuration =
            parse_selected_job_configuration(authored_xml.as_bytes(), wave, source_samples)
                .unwrap_or_else(|error| panic!("input token {token}: {error:?}"));

        assert_eq!(configuration.input_timecode_frame_rate(), token);
        let identity = configuration.normalized_identity_value();
        assert_eq!(identity["input_timecode_frame_rate"], token);
        assert_eq!(identity["encode_timecode_frame_rate"], "not_indicated");
    }

    let unsupported_xml = base_xml.replacen(
        input_field,
        "<timecode_frame_rate>29.97df</timecode_frame_rate>",
        1,
    );
    assert_eq!(
        parse_selected_job_configuration(unsupported_xml.as_bytes(), wave, source_samples)
            .unwrap_err(),
        JobConfigurationError::TimeDomain
    );
}

#[test]
fn authored_time_base_tokens_are_preserved_without_range_effect() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "accepted_full_adm")
        .unwrap();
    let descriptor = &case["pcm_descriptor"];
    let wave = SelectedWaveDescriptor::new(
        descriptor["sample_rate"].as_u64().unwrap() as u32,
        descriptor["channels"].as_u64().unwrap() as u16,
        descriptor["pcm_bits"].as_u64().unwrap() as u16,
    );
    let source_samples = descriptor["source_samples"].as_u64().unwrap();
    let base_xml = case["input_xml"].as_str().unwrap();
    let file_position = "<time_base>file_position</time_base>";

    let embedded_xml =
        base_xml.replacen(file_position, "<time_base>embedded_timecode</time_base>", 1);
    let configuration =
        parse_selected_job_configuration(embedded_xml.as_bytes(), wave, source_samples)
            .expect("embedded_timecode is admitted as an authored XML token");
    assert_eq!(configuration.time_base(), "embedded_timecode");
    let identity = configuration.normalized_identity_value();
    assert_eq!(identity["time_base"], "embedded_timecode");
    assert_eq!(identity["input_timecode_frame_rate"], "not_indicated");

    let unsupported_xml =
        base_xml.replacen(file_position, "<time_base>source_samples</time_base>", 1);
    assert_eq!(
        parse_selected_job_configuration(unsupported_xml.as_bytes(), wave, source_samples)
            .unwrap_err(),
        JobConfigurationError::TimeDomain
    );
}

#[test]
fn authored_input_offset_and_ffoa_strings_are_preserved_independently() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "accepted_full_adm")
        .unwrap();
    let descriptor = &case["pcm_descriptor"];
    let wave = SelectedWaveDescriptor::new(
        descriptor["sample_rate"].as_u64().unwrap() as u32,
        descriptor["channels"].as_u64().unwrap() as u16,
        descriptor["pcm_bits"].as_u64().unwrap() as u16,
    );
    let source_samples = descriptor["source_samples"].as_u64().unwrap();
    let base_xml = case["input_xml"].as_str().unwrap();
    let authored_xml = base_xml
        .replace("<offset>auto</offset>", "<offset>authored-offset</offset>")
        .replace("<ffoa>auto</ffoa>", "<ffoa>authored-ffoa</ffoa>");
    let configuration =
        parse_selected_job_configuration(authored_xml.as_bytes(), wave, source_samples)
            .expect("explicit xs:string input offset and FFOA are lexically admitted");

    assert_eq!(configuration.input_offset(), "authored-offset");
    assert_eq!(configuration.input_ffoa(), "authored-ffoa");
    let identity = configuration.normalized_identity_value();
    assert_eq!(identity["input_offset"], "authored-offset");
    assert_eq!(identity["input_ffoa"], "authored-ffoa");
}

#[test]
fn authored_range_endpoint_strings_are_preserved_without_range_effect() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "accepted_full_adm")
        .unwrap();
    let descriptor = &case["pcm_descriptor"];
    let wave = SelectedWaveDescriptor::new(
        descriptor["sample_rate"].as_u64().unwrap() as u32,
        descriptor["channels"].as_u64().unwrap() as u16,
        descriptor["pcm_bits"].as_u64().unwrap() as u16,
    );
    let source_samples = descriptor["source_samples"].as_u64().unwrap();
    let base_xml = case["input_xml"].as_str().unwrap();
    let authored_xml = base_xml
        .replace("<start></start>", "<start>0</start>")
        .replace("<end></end>", "<end>47</end>");
    let configuration =
        parse_selected_job_configuration(authored_xml.as_bytes(), wave, source_samples)
            .expect("start and end XML strings are lexically admitted");

    assert_eq!(configuration.start(), "0");
    assert_eq!(configuration.end(), "47");
    let identity = configuration.normalized_identity_value();
    assert_eq!(identity["start"], "0");
    assert_eq!(identity["end"], "47");
}

#[test]
fn every_declared_preferred_downmix_token_is_admitted_and_preserved() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "accepted_full_adm")
        .unwrap();
    let descriptor = &case["pcm_descriptor"];
    let wave = SelectedWaveDescriptor::new(
        descriptor["sample_rate"].as_u64().unwrap() as u32,
        descriptor["channels"].as_u64().unwrap() as u16,
        descriptor["pcm_bits"].as_u64().unwrap() as u16,
    );
    let source_samples = descriptor["source_samples"].as_u64().unwrap();
    let base_xml = case["input_xml"].as_str().unwrap();
    let downmix_field = "<preferred_downmix_mode>loro</preferred_downmix_mode>";

    for token in ["loro", "ltrt", "ltrt-pl2", "not_indicated"] {
        let replacement = format!("<preferred_downmix_mode>{token}</preferred_downmix_mode>");
        let authored_xml = base_xml.replacen(downmix_field, &replacement, 1);
        let configuration =
            parse_selected_job_configuration(authored_xml.as_bytes(), wave, source_samples)
                .unwrap_or_else(|error| panic!("preferred downmix {token}: {error:?}"));

        assert_eq!(configuration.downmix().preferred_mode(), token);
        assert_eq!(
            configuration.normalized_identity_value()["downmix"]["preferred_mode"],
            token
        );
    }

    let unsupported_xml = base_xml.replacen(
        downmix_field,
        "<preferred_downmix_mode>mono</preferred_downmix_mode>",
        1,
    );
    assert_eq!(
        parse_selected_job_configuration(unsupported_xml.as_bytes(), wave, source_samples)
            .unwrap_err(),
        JobConfigurationError::UnsupportedFieldValue
    );
}

#[test]
fn every_declared_metering_mode_token_is_admitted_and_preserved() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "accepted_full_adm")
        .unwrap();
    let descriptor = &case["pcm_descriptor"];
    let wave = SelectedWaveDescriptor::new(
        descriptor["sample_rate"].as_u64().unwrap() as u32,
        descriptor["channels"].as_u64().unwrap() as u16,
        descriptor["pcm_bits"].as_u64().unwrap() as u16,
    );
    let source_samples = descriptor["source_samples"].as_u64().unwrap();
    let base_xml = case["input_xml"].as_str().unwrap();
    let metering_field = "<metering_mode>1770-4</metering_mode>";

    for mode in ["1770-4", "1770-3", "1770-2", "1770-1", "LeqA"] {
        let replacement = format!("<metering_mode>{mode}</metering_mode>");
        let authored_xml = base_xml.replacen(metering_field, &replacement, 1);
        let configuration =
            parse_selected_job_configuration(authored_xml.as_bytes(), wave, source_samples)
                .unwrap_or_else(|error| panic!("metering mode {mode}: {error:?}"));

        assert_eq!(configuration.loudness().metering_mode(), mode);
        assert_eq!(
            configuration.normalized_identity_value()["loudness"]["metering_mode"],
            mode
        );
    }

    let unsupported_xml =
        base_xml.replacen(metering_field, "<metering_mode>1770-5</metering_mode>", 1);
    assert_eq!(
        parse_selected_job_configuration(unsupported_xml.as_bytes(), wave, source_samples)
            .unwrap_err(),
        JobConfigurationError::UnsupportedFieldValue
    );
}

#[test]
fn authored_silence_seconds_truncate_to_sample_counts_and_keep_append_presence() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "accepted_full_adm")
        .unwrap();
    let descriptor = &case["pcm_descriptor"];
    let wave = SelectedWaveDescriptor::new(
        descriptor["sample_rate"].as_u64().unwrap() as u32,
        descriptor["channels"].as_u64().unwrap() as u16,
        descriptor["pcm_bits"].as_u64().unwrap() as u16,
    );
    let source_samples = descriptor["source_samples"].as_u64().unwrap();
    let authored = case["input_xml"]
        .as_str()
        .unwrap()
        .replace(
            "<prepend_silence_duration>0.0</prepend_silence_duration>",
            "<prepend_silence_duration>0.00390625</prepend_silence_duration>",
        )
        .replace(
            "<append_silence_duration>0.0</append_silence_duration>",
            "<append_silence_duration>0.000001</append_silence_duration>",
        );
    let configuration = parse_selected_job_configuration(authored.as_bytes(), wave, source_samples)
        .expect("selected numeric seconds parse");

    assert_eq!(configuration.prepend_silence_samples().value(), 187);
    assert_eq!(configuration.append_silence_samples().value(), 0);
    assert!(configuration.append_silence_duration_explicit());

    let omitted_append = case["input_xml"]
        .as_str()
        .unwrap()
        .replace("<append_silence_duration>0.0</append_silence_duration>", "");
    let configuration =
        parse_selected_job_configuration(omitted_append.as_bytes(), wave, source_samples)
            .expect("append omission remains a typed default at parsing");
    assert_eq!(configuration.append_silence_samples().value(), 0);
    assert!(!configuration.append_silence_duration_explicit());

    let frame_duration = case["input_xml"].as_str().unwrap().replace(
        "<prepend_silence_duration>0.0</prepend_silence_duration>",
        "<prepend_silence_duration>1f</prepend_silence_duration>",
    );
    assert_eq!(
        parse_selected_job_configuration(frame_duration.as_bytes(), wave, source_samples)
            .unwrap_err(),
        JobConfigurationError::TimeDomain
    );
}

#[test]
fn selected_job_admission_carries_each_washed_single_rate_into_the_profile() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "accepted_full_adm")
        .unwrap();
    let descriptor = &case["pcm_descriptor"];
    let wave = SelectedWaveDescriptor::new(
        descriptor["sample_rate"].as_u64().unwrap() as u32,
        descriptor["channels"].as_u64().unwrap() as u16,
        descriptor["pcm_bits"].as_u64().unwrap() as u16,
    );
    let source_samples = descriptor["source_samples"].as_u64().unwrap();
    let base_xml = case["input_xml"].as_str().unwrap();

    let rows = [
        (384, 0, 767, 1_536),
        (448, 1, 895, 1_792),
        (576, 2, 1_151, 2_304),
        (640, 3, 1_279, 2_560),
        (768, 4, 1_535, 3_072),
        (1_024, 5, 2_047, 4_096),
    ];

    for (rate, selector_code, frame_size_code, frame_bytes) in rows {
        let xml = base_xml.replace(
            "<data_rate>1024</data_rate>",
            &format!("<data_rate>{rate}</data_rate>"),
        );
        let configuration =
            parse_selected_job_configuration(xml.as_bytes(), wave, source_samples).unwrap();
        let codec_rate = configuration.selected_profile().codec_rate();
        let frame_size =
            jocforge_domain::codec::frame_size::Eac3FrameSize::from_codec_rate(codec_rate).unwrap();

        assert_eq!(configuration.bit_rate_kbps(), rate);
        assert_eq!(configuration.bit_rate_enum(), selector_code);
        assert_eq!(codec_rate.kbps(), Some(rate));
        assert_eq!(codec_rate.selector_code(), Some(selector_code));
        assert_eq!(frame_size.code().value(), frame_size_code);
        assert_eq!(frame_size.bytes(), frame_bytes);
    }

    for token in ["384 448", "0384", "512"] {
        let xml = base_xml.replace(
            "<data_rate>1024</data_rate>",
            &format!("<data_rate>{token}</data_rate>"),
        );
        assert!(matches!(
            parse_selected_job_configuration(xml.as_bytes(), wave, source_samples),
            Err(JobConfigurationError::BitRate)
        ));
    }
}

#[test]
fn adm_job_configuration_accepts_actual_pcm_channel_count() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let cases = vectors["config_cases"].as_array().unwrap();
    let adm_case = cases
        .iter()
        .find(|case| {
            case["expected"]["status"] == "ACCEPT" && case["expected"]["input_kind"] == "adm"
        })
        .expect("approved ADM configuration vector");
    let descriptor = &adm_case["pcm_descriptor"];
    let parsed = parse_selected_job_configuration(
        adm_case["input_xml"].as_str().unwrap().as_bytes(),
        SelectedWaveDescriptor::new(
            descriptor["sample_rate"].as_u64().unwrap() as u32,
            36,
            descriptor["pcm_bits"].as_u64().unwrap() as u16,
        ),
        descriptor["source_samples"].as_u64().unwrap(),
    )
    .unwrap();
    assert_eq!(parsed.wave().channels(), 36);
    assert_eq!(parsed.input_kind(), SelectedInputKind::Adm);
    let data_bytes = parsed.source_samples() * 36 * 3;
    let binding = bind_admitted_selected_input(
        &parsed,
        SelectedPrimaryRuntimeIdentity {
            source_file_bytes: data_bytes + 44,
            data_bytes,
            metadata_hashes: PrimaryMetadataChunkHashes::new([1; 32], [2; 32], [3; 32]),
            resource_sha256: [4; 32],
            config_sha256: [5; 32],
            job_xml_sha256: [6; 32],
            job_xml_bytes: 1,
        },
    )
    .unwrap();
    assert_eq!(binding.wave().channels(), 36);
    assert_eq!(binding.data_bytes(), data_bytes);
}

#[test]
fn atmos_mezz_configuration_accepts_authorized_36_and_38_row_metadata_dimensions() {
    let dimensions = fixture_json("authorized-adm-input-dimensions.json");
    let vectors = fixture_json("job-configuration-vectors.json");
    let cases = vectors["config_cases"].as_array().unwrap();
    assert_eq!(
        dimensions["source_package_id"],
        "AUTHORIZED-ADM-36-38-TRACK-METADATA-CROSSWALK-V2"
    );
    assert_eq!(
        dimensions["source_package_manifest_sha256"],
        "2e238835a0edd84a90764a2c74778f50b2c12e760d7ad051446a772f2efc8113"
    );
    assert_eq!(dimensions["metadata_only"], true);

    let atmos_mezz_case = cases
        .iter()
        .find(|case| {
            case["expected"]["status"] == "ACCEPT" && case["expected"]["input_kind"] == "atmos_mezz"
        })
        .expect("approved Atmos mezzanine configuration vector");
    for sample in dimensions["samples"].as_array().unwrap() {
        let channels = sample["wave_channels"].as_u64().unwrap() as u16;
        assert_eq!(
            sample["chna_track_count"].as_u64().unwrap(),
            u64::from(channels)
        );
        assert_eq!(sample["chna_records_size_valid"], true);
        assert_eq!(
            sample["chna_record_area_bytes"].as_u64().unwrap(),
            sample["chna_track_count"].as_u64().unwrap()
                * sample["chna_record_bytes"].as_u64().unwrap()
        );
        assert_eq!(
            sample["block_align_bytes"].as_u64().unwrap(),
            u64::from(channels) * 3
        );
        assert!(sample["data_frames"].as_u64().unwrap() > 0);
        assert_eq!(
            sample["axml_track_uid_count"].as_u64().unwrap(),
            u64::from(channels)
        );
        assert_ne!(
            sample["axml_audio_object_elements"].as_u64().unwrap(),
            u64::from(channels),
            "AXML object elements are not physical WAVE rows"
        );

        let parsed = parse_selected_job_configuration(
            atmos_mezz_case["input_xml"].as_str().unwrap().as_bytes(),
            SelectedWaveDescriptor::new(
                sample["sample_rate_hz"].as_u64().unwrap() as u32,
                channels,
                sample["pcm_bits"].as_u64().unwrap() as u16,
            ),
            sample["data_frames"].as_u64().unwrap(),
        )
        .unwrap_or_else(|error| panic!("{}: {error:?}", sample["id"]));

        assert_eq!(parsed.input_kind(), SelectedInputKind::AtmosMezz);
        assert_eq!(parsed.wave().channels(), channels);
        assert_eq!(parsed.wave().sample_rate_hz(), 48_000);
        assert_eq!(parsed.wave().pcm_bits(), 24);
        assert_eq!(parsed.layout_enum(), 8);
        assert_eq!(parsed.frame_samples(), 1_536);
        let identity = parsed.normalized_identity_value();
        assert_eq!(
            identity["profile_dimensions"]["authored_input"],
            "selected_adm_bwf_pcm24_48khz"
        );
        assert_eq!(
            identity["wave"]["channels"].as_u64(),
            Some(u64::from(channels))
        );
        assert_eq!(
            identity["profile_dimensions"]["core_carrier"],
            "selected_5_1_six_carriers"
        );
    }

    for (sample_rate, channels, pcm_bits) in [
        (48_000, 0, 24),
        (48_000, 257, 24),
        (44_100, 36, 24),
        (48_000, 36, 16),
    ] {
        let descriptor = &atmos_mezz_case["pcm_descriptor"];
        let result = parse_selected_job_configuration(
            atmos_mezz_case["input_xml"].as_str().unwrap().as_bytes(),
            SelectedWaveDescriptor::new(sample_rate, channels, pcm_bits),
            descriptor["source_samples"].as_u64().unwrap(),
        );
        assert_eq!(
            result.unwrap_err().code(),
            "CONFIG_PCM_PROFILE",
            "sample_rate={sample_rate} channels={channels} pcm_bits={pcm_bits}"
        );
    }
}

#[test]
fn accepts_selected_film_light_drc_pair_with_selected_meter_controls() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["expected"]["status"] == "ACCEPT")
        .unwrap();
    let input_xml = case["input_xml"]
        .as_str()
        .unwrap()
        .replace(
            "<line_mode_drc_profile>none</line_mode_drc_profile><rf_mode_drc_profile>none</rf_mode_drc_profile>",
            "<line_mode_drc_profile>film_light</line_mode_drc_profile><rf_mode_drc_profile>film_light</rf_mode_drc_profile>",
        );
    let descriptor = &case["pcm_descriptor"];
    let wave = SelectedWaveDescriptor::new(
        descriptor["sample_rate"].as_u64().unwrap() as u32,
        descriptor["channels"].as_u64().unwrap() as u16,
        descriptor["pcm_bits"].as_u64().unwrap() as u16,
    );

    let config = parse_selected_job_configuration(
        input_xml.as_bytes(),
        wave,
        descriptor["source_samples"].as_u64().unwrap(),
    )
    .unwrap();

    assert!(!config.loudness().dialogue_intelligence());
    assert_eq!(config.loudness().speech_threshold(), 100);
    assert_eq!(config.line_drc_profile(), SelectedDrcProfile::FilmLight);
    assert_eq!(config.rf_drc_profile(), SelectedDrcProfile::FilmLight);
}

#[test]
fn parses_washed_dialogue_intelligence_inputs_without_claiming_measurement_effect() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["expected"]["status"] == "ACCEPT")
        .unwrap();
    let base_xml = case["input_xml"].as_str().unwrap();
    let descriptor = &case["pcm_descriptor"];
    let wave = SelectedWaveDescriptor::new(
        descriptor["sample_rate"].as_u64().unwrap() as u32,
        descriptor["channels"].as_u64().unwrap() as u16,
        descriptor["pcm_bits"].as_u64().unwrap() as u16,
    );
    let source_samples = descriptor["source_samples"].as_u64().unwrap();

    for dialogue_intelligence in ["false", "true"] {
        for threshold in [0, 15, 20, 100] {
            let xml = base_xml
                .replace(
                    "<dialogue_intelligence>false</dialogue_intelligence>",
                    &format!(
                        "<dialogue_intelligence>{dialogue_intelligence}</dialogue_intelligence>"
                    ),
                )
                .replace(
                    "<speech_threshold>100</speech_threshold>",
                    &format!("<speech_threshold>{threshold}</speech_threshold>"),
                );
            let config =
                parse_selected_job_configuration(xml.as_bytes(), wave, source_samples).unwrap();
            assert_eq!(
                config.loudness().dialogue_intelligence(),
                dialogue_intelligence == "true"
            );
            assert_eq!(config.loudness().speech_threshold(), threshold);
            let identity = config.normalized_identity_value();
            assert_eq!(
                identity["loudness"]["dialogue_intelligence"],
                dialogue_intelligence == "true"
            );
            assert_eq!(identity["loudness"]["speech_threshold"], threshold);
        }
    }

    for invalid_threshold in ["-1", "101", "15.0"] {
        let xml = base_xml.replace(
            "<speech_threshold>100</speech_threshold>",
            &format!("<speech_threshold>{invalid_threshold}</speech_threshold>"),
        );
        assert_eq!(
            parse_selected_job_configuration(xml.as_bytes(), wave, source_samples)
                .unwrap_err()
                .code(),
            "CONFIG_FIELD_VALUE"
        );
    }

    let invalid_boolean = base_xml.replace(
        "<dialogue_intelligence>false</dialogue_intelligence>",
        "<dialogue_intelligence>TRUE</dialogue_intelligence>",
    );
    assert_eq!(
        parse_selected_job_configuration(invalid_boolean.as_bytes(), wave, source_samples)
            .unwrap_err()
            .code(),
        "CONFIG_FIELD_VALUE"
    );
}

#[test]
fn every_declared_trim_xml_choice_is_typed_and_preserved_independently() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["expected"]["status"] == "ACCEPT")
        .unwrap();
    let base_xml = case["input_xml"].as_str().unwrap();
    let descriptor = &case["pcm_descriptor"];
    let wave = SelectedWaveDescriptor::new(
        descriptor["sample_rate"].as_u64().unwrap() as u32,
        descriptor["channels"].as_u64().unwrap() as u16,
        descriptor["pcm_bits"].as_u64().unwrap() as u16,
    );
    let source_samples = descriptor["source_samples"].as_u64().unwrap();
    let surround_values = ["auto", "0", "-3", "-6", "-9"];
    let height_values = ["auto", "-3", "-6", "-9", "-12"];

    for surround in surround_values {
        for height in height_values {
            let xml = base_xml
                .replace(
                    "<surround_trim_5_1>auto</surround_trim_5_1>",
                    &format!("<surround_trim_5_1>{surround}</surround_trim_5_1>"),
                )
                .replace(
                    "<height_trim_5_1>auto</height_trim_5_1>",
                    &format!("<height_trim_5_1>{height}</height_trim_5_1>"),
                );
            let config =
                parse_selected_job_configuration(xml.as_bytes(), wave, source_samples).unwrap();
            assert_eq!(
                config.surround_trim_5_1(),
                SelectedSurroundTrim::from_xml_value(surround).unwrap()
            );
            assert_eq!(
                config.height_trim_5_1(),
                SelectedHeightTrim::from_xml_value(height).unwrap()
            );
            let identity = config.normalized_identity_value();
            assert_eq!(identity["surround_trim_5_1"], surround);
            assert_eq!(identity["height_trim_5_1"], height);
        }
    }

    for (field, invalid_value, xml) in [
        (
            "surround_trim_5_1",
            "-12",
            base_xml.replace(
                "<surround_trim_5_1>auto</surround_trim_5_1>",
                "<surround_trim_5_1>-12</surround_trim_5_1>",
            ),
        ),
        (
            "height_trim_5_1",
            "0",
            base_xml.replace(
                "<height_trim_5_1>auto</height_trim_5_1>",
                "<height_trim_5_1>0</height_trim_5_1>",
            ),
        ),
    ] {
        assert_eq!(
            parse_selected_job_configuration(xml.as_bytes(), wave, source_samples)
                .unwrap_err()
                .code(),
            "CONFIG_FIELD_VALUE",
            "field={field} invalid_value={invalid_value}"
        );
    }
}

#[test]
fn every_line_and_rf_drc_xml_value_is_typed_and_preserved_independently() {
    let vectors = fixture_json("job-configuration-vectors.json");
    let case = vectors["config_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["expected"]["status"] == "ACCEPT")
        .unwrap();
    let base_xml = case["input_xml"].as_str().unwrap();
    let descriptor = &case["pcm_descriptor"];
    let wave = SelectedWaveDescriptor::new(
        descriptor["sample_rate"].as_u64().unwrap() as u32,
        descriptor["channels"].as_u64().unwrap() as u16,
        descriptor["pcm_bits"].as_u64().unwrap() as u16,
    );
    let values = [
        "film_standard",
        "film_light",
        "music_standard",
        "music_light",
        "speech",
        "none",
    ];

    for line in values {
        for rf in values {
            let line_field = format!("<line_mode_drc_profile>{line}</line_mode_drc_profile>");
            let rf_field = format!("<rf_mode_drc_profile>{rf}</rf_mode_drc_profile>");
            let xml = base_xml
                .replace(
                    "<line_mode_drc_profile>none</line_mode_drc_profile>",
                    &line_field,
                )
                .replace("<rf_mode_drc_profile>none</rf_mode_drc_profile>", &rf_field);
            let config = parse_selected_job_configuration(
                xml.as_bytes(),
                wave,
                descriptor["source_samples"].as_u64().unwrap(),
            )
            .unwrap_or_else(|error| panic!("Line={line}, RF={rf}: {error:?}"));

            assert_eq!(
                config.line_drc_profile(),
                SelectedDrcProfile::from_xml_value(line).unwrap()
            );
            assert_eq!(
                config.rf_drc_profile(),
                SelectedDrcProfile::from_xml_value(rf).unwrap()
            );
            let identity = config.normalized_identity_value();
            assert_eq!(identity["line_drc_profile"], line);
            assert_eq!(identity["rf_drc_profile"], rf);
        }
    }

    let unknown = base_xml.replace(
        "<line_mode_drc_profile>none</line_mode_drc_profile>",
        "<line_mode_drc_profile>automatic</line_mode_drc_profile>",
    );
    assert_eq!(
        parse_selected_job_configuration(
            unknown.as_bytes(),
            wave,
            descriptor["source_samples"].as_u64().unwrap(),
        )
        .unwrap_err()
        .code(),
        "CONFIG_FIELD_VALUE"
    );
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
            assert_eq!(actual.regular_control_order(), expected_regular.as_slice());
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
