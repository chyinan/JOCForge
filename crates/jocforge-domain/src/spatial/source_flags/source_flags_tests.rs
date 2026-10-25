// pattern: Imperative Shell
#[cfg(test)]
mod tests {
    use super::super::{
        BeforeRegularHistory, CurrentHostPcm, FRAME_SAMPLES, HOST_CHANNELS, HostRole,
        HostRouteRecord, PREFIX_SOURCE_COUNT, PrefixWriterSet, REGULAR_HOSTS, RegularControl,
        RegularControls, SOURCE_COUNT, SelectedFeatureFlagsControl, SourceFlagsAttempt,
        SourceFlagsError, SourceFlagsHistoryGeneration, SourceFlagsInput, SourceFlagsParentLineage,
        SourceFlagsProducer, SourceFlagsProgrammeId, ValidatedSemanticRoute,
    };
    use crate::{
        frontend::{
            PackageManifestEvidence, ProviderFileEvidence, validate_f01e_package_manifest,
            validate_f01e_provider, validate_f03_package_manifest,
            validate_f03_target_cold_provider,
        },
        profile::{EncodingProfile, resolve_selected_profile},
        spatial::SpatialControlMachine,
        timeline::FrontendCallbackIndex,
        transaction::CallbackTransactionId,
    };
    use serde_json::Value;
    use sha2::{Digest, Sha256};
    use std::{
        env,
        fmt::Write as FmtWrite,
        fs::File,
        io::{Read, Seek, SeekFrom},
        sync::Arc,
    };

    const MANIFEST_BYTES: &[u8] =
        include_bytes!("../../../tests/fixtures/f01e_provider/MANIFEST.json");
    const F01E_PROVIDER_BYTES: &[u8] =
        include_bytes!("../../../tests/fixtures/f01e_provider/f01e-selected-constructor-v1.json");
    const F03_PROVIDER_BYTES: &[u8] =
        include_bytes!("../../../tests/fixtures/target_cold/f03-selected-target-cold-v1.json");
    const SOURCE_FLAGS_VECTORS: &[u8] =
        include_bytes!("../../../tests/fixtures/source_flags/INPUT-ONLY-VECTORS.json");

    fn selected_machine() -> SpatialControlMachine {
        let f01e_manifest_evidence = PackageManifestEvidence::new(Arc::from(MANIFEST_BYTES), None);
        let f01e_manifest = validate_f01e_package_manifest(&f01e_manifest_evidence).unwrap();
        let f01e_evidence = ProviderFileEvidence::new(Arc::from(F01E_PROVIDER_BYTES), None);
        let f01e = validate_f01e_provider(&f01e_evidence, &f01e_manifest).unwrap();

        let f03_manifest_evidence = PackageManifestEvidence::new(Arc::from(MANIFEST_BYTES), None);
        let f03_manifest = validate_f03_package_manifest(&f03_manifest_evidence).unwrap();
        let f03_evidence = ProviderFileEvidence::new(Arc::from(F03_PROVIDER_BYTES), None);
        let f03 = validate_f03_target_cold_provider(&f03_evidence, &f03_manifest).unwrap();
        SpatialControlMachine::new_cold(EncodingProfile::selected(), &f01e, &f03).unwrap()
    }

    fn zero_rows(row_count: usize) -> Vec<Vec<f32>> {
        (0..row_count).map(|_| vec![0.0; FRAME_SAMPLES]).collect()
    }

    fn route_records(
        lfe_host: u8,
        enabled: bool,
        duplicate_destination: bool,
    ) -> Vec<HostRouteRecord> {
        let mut next_source = 11_u8;
        let mut first_destination = None;
        (0..HOST_CHANNELS)
            .map(|host| {
                let role = if host as u8 == lfe_host {
                    HostRole::Lfe
                } else {
                    HostRole::Regular
                };
                let destination = if role == HostRole::Lfe {
                    None
                } else {
                    let value = if duplicate_destination && next_source == 12 {
                        first_destination.unwrap()
                    } else {
                        next_source
                    };
                    first_destination.get_or_insert(value);
                    next_source += 1;
                    Some(value)
                };
                HostRouteRecord::new(host as u8, role, enabled, destination).unwrap()
            })
            .collect()
    }

    fn controls_for(
        route: &ValidatedSemanticRoute,
        overrides: Option<&Value>,
    ) -> Result<RegularControls, SourceFlagsError> {
        let controls = route
            .regular_hosts()
            .iter()
            .enumerate()
            .map(|(row, host)| {
                let override_value = overrides.and_then(|all| all.get(row.to_string()));
                let mute = override_value
                    .and_then(|item| item.get("mute"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as u8;
                let width_bits = override_value
                    .and_then(|item| item.get("width_u32"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as u32;
                let component = override_value
                    .and_then(|item| item.get("component_enum"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as u8;
                let diffuse = override_value
                    .and_then(|item| item.get("diffuse"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as u8;
                RegularControl::from_raw(
                    host.value(),
                    mute,
                    f32::from_bits(width_bits),
                    component,
                    diffuse,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        RegularControls::new(controls)
    }

    fn parent_lineage(
        machine: &SpatialControlMachine,
        transaction_id: &str,
        callback_index: u64,
    ) -> SourceFlagsParentLineage {
        SourceFlagsParentLineage::new(
            SourceFlagsProgrammeId::new("SYNTHETIC_FLAGS").unwrap(),
            SourceFlagsAttempt::new(0),
            CallbackTransactionId::new(transaction_id).unwrap(),
            FrontendCallbackIndex::new(callback_index),
            machine,
        )
        .unwrap()
    }

    fn test_history(
        machine: &SpatialControlMachine,
        route: &ValidatedSemanticRoute,
        owner_generation: u64,
        rows: Vec<Vec<f32>>,
    ) -> Result<BeforeRegularHistory, SourceFlagsError> {
        let provider_lineage = machine.provider_lineage();
        BeforeRegularHistory::from_rows_with_receipt(
            SourceFlagsHistoryGeneration::new(owner_generation),
            machine.digest(),
            machine.profile(),
            &provider_lineage,
            route,
            rows,
        )
    }

    fn case_input(
        case: &Value,
        machine: &SpatialControlMachine,
        transaction_id: &str,
    ) -> Result<SourceFlagsInput, SourceFlagsError> {
        let callback_index = case["input"]
            .get("packet_callback_index")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        case_input_at(case, machine, transaction_id, callback_index)
    }

    fn case_input_at(
        case: &Value,
        machine: &SpatialControlMachine,
        transaction_id: &str,
        callback_index: u64,
    ) -> Result<SourceFlagsInput, SourceFlagsError> {
        let input = &case["input"];
        let lfe_host = input.get("lfe_host").and_then(Value::as_u64).unwrap_or(3) as u8;
        let disabled = input
            .get("route_enabled")
            .and_then(Value::as_bool)
            .is_some_and(|enabled| !enabled);
        let duplicate_destination = input
            .get("mapping_duplicate_destination")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let route =
            ValidatedSemanticRoute::new(route_records(lfe_host, !disabled, duplicate_destination))?;

        let pcm_shape = input.get("PCM_shape");
        let mut current_rows = zero_rows(HOST_CHANNELS);
        if pcm_shape.is_some() {
            current_rows[0].pop();
        }
        if let Some(overrides) = input.get("current_overrides").and_then(Value::as_array) {
            for override_value in overrides {
                let host = override_value[0].as_u64().unwrap() as usize;
                let sample = override_value[1].as_u64().unwrap() as usize;
                let bits = override_value[2].as_u64().unwrap() as u32;
                current_rows[host][sample] = f32::from_bits(bits);
            }
        }
        let current_pcm = CurrentHostPcm::new(current_rows)?;

        let mut history_rows = zero_rows(REGULAR_HOSTS);
        if let Some(overrides) = input.get("history_overrides").and_then(Value::as_array) {
            for override_value in overrides {
                let row = override_value[0].as_u64().unwrap() as usize;
                let sample = override_value[1].as_u64().unwrap() as usize;
                let bits = override_value[2].as_u64().unwrap() as u32;
                history_rows[row][sample] = f32::from_bits(bits);
            }
        }
        if input
            .get("missing_history")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            return Ok(SourceFlagsInput::new(
                current_pcm,
                None,
                controls_for(&route, input.get("control_overrides"))?,
                route,
                PrefixWriterSet::new(
                    input
                        .get("prefix_written_sources")
                        .and_then(Value::as_array)
                        .map(|values| {
                            values
                                .iter()
                                .map(|value| value.as_u64().unwrap() as u8)
                                .collect()
                        })
                        .unwrap_or_default(),
                )?,
                parent_lineage(machine, transaction_id, callback_index),
            ));
        }

        let mut history = test_history(
            machine,
            &route,
            input
                .get("history_owner_generation")
                .and_then(Value::as_u64)
                .unwrap_or(machine.generation()),
            history_rows,
        )?;
        if input
            .get("next_history_wrong_generation")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            history.owner_generation = SourceFlagsHistoryGeneration::new(1);
        }
        let history = Some(history);
        let controls = controls_for(&route, input.get("control_overrides"))?;
        let prefix_writers = PrefixWriterSet::new(
            input
                .get("prefix_written_sources")
                .and_then(Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .map(|value| value.as_u64().unwrap() as u8)
                        .collect()
                })
                .unwrap_or_default(),
        )?;
        Ok(SourceFlagsInput::new(
            current_pcm,
            history,
            controls,
            route,
            prefix_writers,
            parent_lineage(machine, transaction_id, callback_index),
        ))
    }

    fn digest_hex(bytes: &[u8]) -> String {
        hex_encode(&Sha256::digest(bytes))
    }

    fn hex_encode(bytes: &[u8]) -> String {
        let mut output = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
        }
        output
    }

    fn read_authorized_first_pcm(path: &str, expected_sha256: &str) -> Vec<Vec<f32>> {
        let mut file = File::open(path).expect("authorized WAV cannot be opened");
        let mut hasher = Sha256::new();
        let mut hash_buffer = vec![0_u8; 1024 * 1024];
        loop {
            let read = file.read(&mut hash_buffer).expect("WAV hash read failed");
            if read == 0 {
                break;
            }
            hasher.update(&hash_buffer[..read]);
        }
        let actual_sha256 = hex_encode(&hasher.finalize());
        assert_eq!(actual_sha256, expected_sha256);

        file.seek(SeekFrom::Start(0)).expect("WAV rewind failed");
        let mut riff = [0_u8; 12];
        file.read_exact(&mut riff).expect("WAV header is truncated");
        assert_eq!(&riff[0..4], b"RIFF");
        assert_eq!(&riff[8..12], b"WAVE");

        let mut format = None;
        let (data_start, data_bytes) = loop {
            let mut chunk_header = [0_u8; 8];
            file.read_exact(&mut chunk_header)
                .expect("WAV chunk header is missing");
            let chunk_size = u32::from_le_bytes(chunk_header[4..8].try_into().unwrap());
            let body_start = file.stream_position().expect("WAV position unavailable");
            if &chunk_header[0..4] == b"fmt " {
                assert!((16..=128).contains(&chunk_size));
                let mut body = vec![0_u8; chunk_size as usize];
                file.read_exact(&mut body)
                    .expect("WAV format chunk truncated");
                let tag = u16::from_le_bytes(body[0..2].try_into().unwrap());
                let channels = u16::from_le_bytes(body[2..4].try_into().unwrap());
                let sample_rate = u32::from_le_bytes(body[4..8].try_into().unwrap());
                let block_align = u16::from_le_bytes(body[12..14].try_into().unwrap());
                let bits_per_sample = u16::from_le_bytes(body[14..16].try_into().unwrap());
                if tag == 0xfffe {
                    assert!(body.len() >= 40);
                    assert_eq!(&body[24..26], &[1, 0]);
                } else {
                    assert_eq!(tag, 1);
                }
                assert_eq!(channels, HOST_CHANNELS as u16);
                assert_eq!(sample_rate, 48_000);
                assert_eq!(block_align, (HOST_CHANNELS * 3) as u16);
                assert_eq!(bits_per_sample, 24);
                format = Some((channels as usize, block_align as usize));
                if chunk_size & 1 == 1 {
                    file.seek(SeekFrom::Current(1))
                        .expect("WAV padding missing");
                }
            } else if &chunk_header[0..4] == b"data" {
                break (body_start, chunk_size);
            } else {
                let skip = i64::from(chunk_size + (chunk_size & 1));
                file.seek(SeekFrom::Current(skip))
                    .expect("WAV chunk seek failed");
            }
        };

        let (channels, block_align) = format.expect("WAV format chunk missing");
        assert!(data_bytes as usize >= FRAME_SAMPLES * block_align);
        file.seek(SeekFrom::Start(data_start))
            .expect("WAV data seek failed");
        let mut interleaved = vec![0_u8; FRAME_SAMPLES * block_align];
        file.read_exact(&mut interleaved)
            .expect("first callback PCM is truncated");
        let mut rows = vec![vec![0.0_f32; FRAME_SAMPLES]; channels];
        for (sample_index, frame) in interleaved.chunks_exact(block_align).enumerate() {
            for (channel, row) in rows.iter_mut().enumerate() {
                let offset = channel * 3;
                let raw = u32::from(frame[offset])
                    | (u32::from(frame[offset + 1]) << 8)
                    | (u32::from(frame[offset + 2]) << 16);
                let sign = if raw & 0x0080_0000 != 0 { 0xff } else { 0 };
                let signed =
                    i32::from_le_bytes([frame[offset], frame[offset + 1], frame[offset + 2], sign]);
                row[sample_index] = signed as f32 / 8_388_608.0_f32;
            }
        }
        rows
    }

    fn authorized_route(real_case: &Value) -> ValidatedSemanticRoute {
        let row_to_source = real_case["input"]["row_to_source"].as_object().unwrap();
        let lfe_host = vectors()["synthetic_base"]["default_lfe_host"]
            .as_u64()
            .unwrap() as u8;
        let records = (0..HOST_CHANNELS)
            .map(|host| {
                if host as u8 == lfe_host {
                    HostRouteRecord::new(host as u8, HostRole::Lfe, true, None).unwrap()
                } else {
                    let destination = row_to_source[&host.to_string()].as_u64().unwrap() as u8;
                    HostRouteRecord::new(host as u8, HostRole::Regular, true, Some(destination))
                        .unwrap()
                }
            })
            .collect();
        ValidatedSemanticRoute::new(records).unwrap()
    }

    #[test]
    #[ignore = "requires the authorized WAV supplied via JOCFORGE_AUTHORIZED_WAV"]
    fn authorized_wav_boundary_matches_the_literal_96_byte_vector() {
        let path = env::var("JOCFORGE_AUTHORIZED_WAV")
            .expect("set JOCFORGE_AUTHORIZED_WAV for the authorized boundary replay");
        let vectors = vectors();
        let case = &vectors["real_first_callback_case"];
        let input = &case["input"];
        assert_eq!(input["start_sample"], 0);
        assert_eq!(input["samples"], FRAME_SAMPLES);
        assert_eq!(input["channel_count"], HOST_CHANNELS);
        let current = CurrentHostPcm::new(read_authorized_first_pcm(
            &path,
            input["customer_PCM24_WAVE_sha256"].as_str().unwrap(),
        ))
        .unwrap();
        let machine = selected_machine();
        let route = authorized_route(case);
        let controls = input["controls"]
            .as_array()
            .unwrap()
            .iter()
            .map(|control| {
                RegularControl::from_raw(
                    control["control_row_index"].as_u64().unwrap() as u8,
                    control["mute"].as_u64().unwrap() as u8,
                    control["width"].as_f64().unwrap() as f32,
                    control["component_enum"].as_u64().unwrap() as u8,
                    control["diffuse"].as_u64().unwrap() as u8,
                )
                .unwrap()
            })
            .collect();
        let controls = RegularControls::new(controls).unwrap();
        let history = BeforeRegularHistory::from_owner(&machine, &route).unwrap();
        let prefix_writers = PrefixWriterSet::new(
            input["prefix_written_sources"]
                .as_array()
                .unwrap()
                .iter()
                .map(|source| source.as_u64().unwrap() as u8)
                .collect(),
        )
        .unwrap();
        let parent = SourceFlagsParentLineage::new(
            SourceFlagsProgrammeId::new("AUTHORIZED_FLAGS_BOUNDARY").unwrap(),
            SourceFlagsAttempt::new(0),
            CallbackTransactionId::new("FLAGS-AUTHORIZED-BOUNDARY-0").unwrap(),
            FrontendCallbackIndex::new(0),
            &machine,
        )
        .unwrap();
        let produced = SourceFlagsProducer::produce(SourceFlagsInput::new(
            current,
            Some(history),
            controls,
            route,
            prefix_writers,
            parent,
        ))
        .unwrap();
        assert_eq!(
            payload_hex(produced.packet.payload()),
            case["expected"]["flags_u8_48x2_hex"].as_str().unwrap()
        );
        assert_eq!(
            digest_hex(
                produced
                    .packet
                    .payload()
                    .iter()
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>()
                    .as_slice()
            ),
            case["expected"]["sha256"].as_str().unwrap()
        );
        assert_eq!(produced.raw_history_proposal.rows().len(), REGULAR_HOSTS);
        assert_eq!(machine.generation(), 0);
    }

    fn payload_hex(payload: &[[u8; 2]; SOURCE_COUNT]) -> String {
        let bytes: Vec<u8> = payload.iter().flatten().copied().collect();
        hex_encode(&bytes)
    }

    fn vectors() -> Value {
        serde_json::from_slice(SOURCE_FLAGS_VECTORS).unwrap()
    }

    #[test]
    fn supplement_identity_and_case_counts_are_pinned() {
        assert_eq!(SOURCE_FLAGS_VECTORS.len(), 27_933);
        assert_eq!(
            digest_hex(SOURCE_FLAGS_VECTORS),
            "ba7fb477931bc6f7127b63631b8048e127ecd78514133749ce3b0acdf61d3aaf"
        );
        let vectors = vectors();
        assert_eq!(vectors["positive_cases"].as_array().unwrap().len(), 20);
        assert_eq!(vectors["negative_cases"].as_array().unwrap().len(), 11);
        assert_eq!(vectors["real_first_callback_case"]["expected"]["bytes"], 96);
    }

    #[test]
    fn twenty_synthetic_success_vectors_match_literal_payloads_and_raw_history() {
        let machine = selected_machine();
        let before_digest = machine.digest();
        let before_generation = machine.generation();
        let vectors = vectors();
        for case in vectors["positive_cases"].as_array().unwrap() {
            let input = case_input(case, &machine, "FLAGS-CALLBACK-0").unwrap();
            let expected_raw_history: Vec<Vec<u32>> = input
                .route
                .regular_hosts()
                .iter()
                .map(|host| input.current_pcm.row_bits(host.value()))
                .collect();
            let produced = SourceFlagsProducer::produce(input).unwrap();
            let payload = produced.packet.payload();
            let expected = &case["expected"];
            assert_eq!(payload_hex(payload), expected["flags_u8_48x2_hex"]);
            assert_eq!(
                payload.len() * 2,
                expected["bytes"].as_u64().unwrap() as usize
            );
            assert_eq!(
                digest_hex(
                    payload
                        .iter()
                        .flatten()
                        .copied()
                        .collect::<Vec<_>>()
                        .as_slice()
                ),
                expected["sha256"]
            );
            assert!(payload.iter().all(|pair| *pair != [0, 1]));
            assert!(
                payload
                    .iter()
                    .all(|pair| pair.iter().all(|flag| *flag <= 1))
            );

            let history = produced.raw_history_proposal.rows();
            assert_eq!(history.len(), REGULAR_HOSTS);
            for (actual, expected) in history.iter().zip(expected_raw_history) {
                assert_eq!(actual.as_ref().as_slice(), expected.as_slice());
            }
        }
        assert_eq!(machine.digest(), before_digest);
        assert_eq!(machine.generation(), before_generation);
    }

    #[test]
    fn eleven_preflight_and_lineage_vectors_reject_without_packet_or_history_change() {
        let machine = selected_machine();
        let before_digest = machine.digest();
        let vectors = vectors();
        for case in vectors["negative_cases"].as_array().unwrap() {
            let id = case["case_id"].as_str().unwrap();
            let expected = case["expected"]["error"].as_str().unwrap();
            let result = if id == "stale_packet" {
                let first_case = vectors["positive_cases"][0].clone();
                let packet_callback = case["input"]["packet_callback_index"].as_u64().unwrap();
                let input =
                    case_input_at(&first_case, &machine, "FLAGS-CALLBACK-0", packet_callback)
                        .unwrap();
                let produced = SourceFlagsProducer::produce(input).unwrap();
                let mut stale = parent_lineage(&machine, "FLAGS-CALLBACK-0", 0);
                stale.callback_index = FrontendCallbackIndex::new(
                    case["input"]["expected_callback_index"].as_u64().unwrap(),
                );
                produced.packet.validate_parent_binding(&stale)
            } else {
                case_input(case, &machine, "FLAGS-CALLBACK-0")
                    .and_then(SourceFlagsProducer::produce)
                    .map(|_| ())
            };
            assert_eq!(result.unwrap_err().code(), expected, "{id}");
            assert_eq!(machine.generation(), 0, "{id}");
            assert_eq!(machine.digest(), before_digest, "{id}");
        }
    }

    #[test]
    fn equal_payloads_from_distinct_parent_lineages_are_not_reusable() {
        let machine = selected_machine();
        let case = &vectors()["positive_cases"][0];
        let first =
            SourceFlagsProducer::produce(case_input(case, &machine, "FLAGS-CALLBACK-0").unwrap())
                .unwrap();
        let second = SourceFlagsProducer::produce(
            case_input(case, &machine, "FLAGS-CALLBACK-OTHER").unwrap(),
        )
        .unwrap();
        assert_eq!(first.packet.payload(), second.packet.payload());
        assert_ne!(first.packet.lineage(), second.packet.lineage());
        assert!(matches!(
            first.packet.validate_lineage(second.packet.lineage()),
            Err(SourceFlagsError::LineageMismatch)
        ));
    }

    #[test]
    fn semantic_lfe_and_prefix_writers_control_disjoint_output_regions() {
        let machine = selected_machine();
        let vectors = vectors();
        let rebound = &vectors["positive_cases"][18];
        let rebound_output = SourceFlagsProducer::produce(
            case_input(rebound, &machine, "FLAGS-CALLBACK-0").unwrap(),
        )
        .unwrap();
        let default_route = ValidatedSemanticRoute::new(route_records(3, true, false)).unwrap();
        let mut current = zero_rows(HOST_CHANNELS);
        current[3][767] = 1.0;
        let parent = parent_lineage(&machine, "FLAGS-CALLBACK-0", 0);
        let before = test_history(
            &machine,
            &default_route,
            machine.generation(),
            zero_rows(REGULAR_HOSTS),
        )
        .unwrap();
        let controls = controls_for(&default_route, None).unwrap();
        let lfe_only = SourceFlagsProducer::produce(SourceFlagsInput::new(
            CurrentHostPcm::new(current).unwrap(),
            Some(before),
            controls,
            default_route,
            PrefixWriterSet::new(vec![]).unwrap(),
            parent,
        ))
        .unwrap();
        assert_eq!(
            payload_hex(rebound_output.packet.payload()),
            vectors["positive_cases"][18]["expected"]["flags_u8_48x2_hex"]
        );
        assert!(
            lfe_only.packet.payload()[11..]
                .iter()
                .all(|pair| *pair == [1, 0])
        );

        let written = &vectors["positive_cases"][16];
        let produced = SourceFlagsProducer::produce(
            case_input(written, &machine, "FLAGS-CALLBACK-0").unwrap(),
        )
        .unwrap();
        for source in 0..11 {
            let expected = if [0, 3, 10].contains(&source) {
                [0, 0]
            } else {
                [1, 1]
            };
            assert_eq!(produced.packet.payload()[source], expected);
        }
    }

    #[test]
    fn prefix_writer_flags_are_independent_of_pcm_amplitude() {
        let machine = selected_machine();
        let vectors = vectors();
        let base = vectors["positive_cases"][16].clone();
        let mut changed_audio = base.clone();
        changed_audio["input"]["current_overrides"] =
            serde_json::from_str::<Value>("[[0, 767, 1065353216], [4, 255, 3212836864]]").unwrap();

        let silent =
            SourceFlagsProducer::produce(case_input(&base, &machine, "FLAGS-CALLBACK-0").unwrap())
                .unwrap();
        let active = SourceFlagsProducer::produce(
            case_input(&changed_audio, &machine, "FLAGS-CALLBACK-0").unwrap(),
        )
        .unwrap();
        assert_eq!(
            &silent.packet.payload()[..PREFIX_SOURCE_COUNT],
            &active.packet.payload()[..PREFIX_SOURCE_COUNT]
        );
        assert_eq!(
            active.packet.payload()[0],
            [0, 0],
            "writer membership, not input amplitude, determines prefix source flags"
        );
    }

    #[test]
    fn triangle_window_known_answer_is_exact() {
        assert_eq!(
            SourceFlagsProducer::window_sha256().unwrap(),
            "134a5a788d42b0532f3ec46112ad978cc14fb3c46cb0d82f02fee5f129b7c919"
        );
    }

    #[test]
    fn raw_history_proposal_preserves_signed_zero_before_flag_processing() {
        let machine = selected_machine();
        let case = &vectors()["positive_cases"][9];
        let produced =
            SourceFlagsProducer::produce(case_input(case, &machine, "FLAGS-CALLBACK-0").unwrap())
                .unwrap();
        assert_eq!(produced.raw_history_proposal.rows()[0][767], 0x8000_0000);
        assert_eq!(produced.packet.payload()[11], [1, 0]);
    }

    #[test]
    fn malformed_semantic_routes_reject_duplicate_host_lfe_and_destination_records() {
        let duplicate_host = {
            let mut records = route_records(3, true, false);
            records[1] = records[0];
            ValidatedSemanticRoute::new(records)
        };
        assert_eq!(duplicate_host.unwrap_err().code(), "ROUTE_HOST_DUPLICATE");

        let multiple_lfe = {
            let mut records = route_records(3, true, false);
            records[4] = HostRouteRecord::new(4, HostRole::Lfe, true, None).unwrap();
            ValidatedSemanticRoute::new(records)
        };
        assert_eq!(multiple_lfe.unwrap_err().code(), "ROUTE_LFE_COUNT");

        let missing_host =
            ValidatedSemanticRoute::new(route_records(3, true, false)[..37].to_vec());
        assert_eq!(missing_host.unwrap_err().code(), "ROUTE_HOST_COUNT");
    }

    #[test]
    fn all_input_shapes_and_regular_host_controls_are_exact() {
        let machine = selected_machine();
        assert_eq!(
            CurrentHostPcm::new(zero_rows(HOST_CHANNELS - 1))
                .unwrap_err()
                .code(),
            "PCM_SHAPE"
        );
        assert_eq!(
            test_history(
                &machine,
                &ValidatedSemanticRoute::new(route_records(3, true, false)).unwrap(),
                machine.generation(),
                zero_rows(REGULAR_HOSTS - 1),
            )
            .unwrap_err()
            .code(),
            "HISTORY_SHAPE"
        );
    }

    #[test]
    fn meter_values_transition_reuses_the_same_flags_without_a_main_packet_owner() {
        let route = ValidatedSemanticRoute::new(route_records(3, true, false)).unwrap();
        let controls = controls_for(&route, None).unwrap();
        let current_pcm = vec![vec![0_u32; FRAME_SAMPLES]; HOST_CHANNELS];
        let previous_regular = vec![vec![0_u32; FRAME_SAMPLES]; REGULAR_HOSTS];
        let values = SourceFlagsProducer::produce_meter_values(
            &current_pcm,
            &previous_regular,
            &controls,
            &route,
            &[0, 4],
        )
        .unwrap();

        for source in 0..PREFIX_SOURCE_COUNT {
            assert_eq!(
                values.payload()[source],
                if source == 0 || source == 4 {
                    [0, 0]
                } else {
                    [1, 1]
                }
            );
        }
        assert!(
            values.payload()[PREFIX_SOURCE_COUNT..]
                .iter()
                .all(|flags| *flags == [1, 0])
        );
    }

    #[test]
    fn selected_feature_control_keeps_authored_audio_diffuse_distinct() {
        let profile = resolve_selected_profile(&EncodingProfile::selected()).unwrap();
        let feature = SelectedFeatureFlagsControl::from_admitted_profile(
            profile,
            FrontendCallbackIndex::new(0),
            10,
        )
        .unwrap();
        let mut authored_control52 = [0_u32; 13];
        authored_control52[10] = 1;
        let copied_event_diffuse = u8::try_from(authored_control52[10]).unwrap();
        assert_eq!(
            RegularControl::from_raw(10, 0, 0.1, 0, copied_event_diffuse),
            Err(SourceFlagsError::UnsupportedComponentOrDiffuse)
        );
        let postroute_source56_diffuse = 1;
        assert_eq!(
            RegularControl::from_raw(10, 0, 0.1, 0, postroute_source56_diffuse),
            Err(SourceFlagsError::UnsupportedComponentOrDiffuse)
        );

        let regular = feature.into_regular_control(0, 0.0).unwrap();
        assert_eq!(authored_control52[10], 1);
        assert_eq!(feature.profile_identity(), profile);
        assert_eq!(feature.callback_index(), FrontendCallbackIndex::new(0));
        assert_eq!(feature.host().value(), 10);
        assert_eq!(feature.component_enum(), 0);
        assert_eq!(feature.diffuse(), 0);
        assert!(!regular.muted);
        assert_eq!(regular.width.to_bits(), 0);
        assert_eq!(regular.host.value(), 10);
    }
}
