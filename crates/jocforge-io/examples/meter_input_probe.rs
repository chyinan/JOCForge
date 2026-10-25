// pattern: Imperative Shell
//! Read-only in-memory probe for a selected RIFF source and its first meter hops.

use jocforge_application::{SelectedAdmResourcePayloads, prepare_selected_job_input};
use jocforge_domain::{
    frontend::{
        PackageManifestEvidence, ProviderFileEvidence,
        dbmd_sections::parse_selected_dbmd_sections,
        job_configuration::{
            PrimaryMetadataChunkHashes, SelectedInputKind, SelectedPrimaryRuntimeIdentity,
            SelectedWaveDescriptor, bind_admitted_selected_input,
        },
        primary_metadata_control::{PrimaryControl38Fields, produce_primary_metadata_control_rows},
        selected_adm_graph::parse_selected_adm_graph,
    },
    meter::{
        installed48_renderer::{
            MeterInstalled48Input, MeterInstalled48State, stage_meter_installed48,
        },
        sample_block_queue::{MeterCallbackIndex, MeterSourceSampleCount},
    },
    profile::ResolvedEncodingProfile,
    spatial::{
        diffusion_filter::{
            DiffusionFilterParameters, validate_diffusion_filter_package_manifest,
            validate_diffusion_filter_parameters,
        },
        installed48_activity::{
            PrimaryActivityInput, PrimaryActivityOutput, PrimaryInstalled48ActivityProducer,
        },
        installed48_source_map::{
            Installed48SourceMap, PrimaryPrefixRoute, build_meter_installed48_source_map,
        },
        selected_p11_geometry::generate_selected_p11_geometry,
        source_flags::{
            HostRole, HostRouteRecord, RegularControls, SelectedFeatureFlagsControl,
            SourceFlagsProducer, ValidatedSemanticRoute,
        },
    },
    timeline::{AdmSampleCount, AdmSampleIndex, FrontendCallbackIndex},
    transaction::ProgrammeId,
};
use jocforge_io::pcm::{
    meter_source_cursor::{
        MeterPcmSourceCursor, MeterPcmSourceIdentity, MeterRawInputFrame,
        stage_meter_pcm_source_frame,
    },
    riff_pcm24::parse_selected_riff_pcm24,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{env, error::Error, fs, io, path::Path, sync::Arc};

const F04_PACKAGE_MANIFEST: &[u8] =
    include_bytes!("../../jocforge-domain/tests/fixtures/f01e_provider/MANIFEST.json");
const DIFFUSION_COEFFICIENTS: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/diffusion_filter/diffusion-mathematical-v1.json"
);
const DIFFUSION_CONSTRUCTOR: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/diffusion_filter/diffusion-constructor-mathematical-v1.json"
);
// Explicit diagnostic caller identity; not derived from source/config bytes or reused by a product job.
fn diagnostic_programme_id() -> ProgrammeId {
    ProgrammeId::new("JOCForgeProbe-v1").expect("diagnostic programme ID is nonempty")
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args_os()
        .nth(1)
        .ok_or_else(|| io::Error::other("pass a selected RIFF/WAVE path"))?;
    let bytes = fs::read(path)?;
    let resource: Arc<[u8]> = Arc::from(bytes.into_boxed_slice());
    let parsed = parse_selected_riff_pcm24(resource.as_ref(), None)
        .map_err(|_| io::Error::other("selected RIFF/PCM24 parse failed"))?;
    let source_identity =
        MeterPcmSourceIdentity::from_selected_resource_sha256(parsed.source_sha256());
    let source_samples = MeterSourceSampleCount::new(parsed.total_samples())
        .map_err(|_| io::Error::other("invalid selected PCM source extent"))?;
    let metadata = parsed
        .selected_adm_payloads()
        .map_err(|_| io::Error::other("selected AXML/CHNA/DBMD spans are missing or duplicated"))?;
    let axml = metadata.axml();
    let chna = metadata.chna();
    let dbmd_payload = metadata.dbmd();
    let graph = parse_selected_adm_graph(axml, chna)
        .map_err(|_| io::Error::other("selected ADM graph parse failed"))?;
    let dbmd = parse_selected_dbmd_sections(dbmd_payload)
        .map_err(|_| io::Error::other("selected DBMD parse failed"))?;
    let admitted_events = graph
        .admit_object_events(
            AdmSampleCount::new(parsed.total_samples())
                .map_err(|_| io::Error::other("invalid ADM sample count"))?,
        )
        .map_err(|_| io::Error::other("selected ADM event admission failed"))?;
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))?;
    let mut job_xml = vectors["config_cases"][0]["input_xml"]
        .as_str()
        .ok_or_else(|| io::Error::other("selected F01E caller fixture is invalid"))?
        .replace(
            "<file_name>input.wav</file_name>",
            "<file_name>input-adm.wav</file_name>",
        );
    job_xml = job_xml.replacen(
        "<path>fixture</path>",
        "<path>test-vectors/full-song</path>",
        1,
    );
    let prepared = prepare_selected_job_input(
        diagnostic_programme_id(),
        job_xml.as_bytes(),
        SelectedWaveDescriptor::new(parsed.sample_rate_hz(), parsed.channels(), 24),
        parsed.total_samples(),
        Arc::clone(&resource),
        SelectedAdmResourcePayloads::new(axml, chna, dbmd_payload),
    )
    .map_err(|error| io::Error::other(format!("selected F01E input admission failed: {error}")))?;
    if prepared.configuration().input_kind() != SelectedInputKind::Adm
        || prepared.resource().as_ptr() != resource.as_ptr()
        || prepared.resource_identity().input_sha256() != source_identity.sha256()
        || prepared.graph().provenance().parent_resource() != prepared.resource_identity()
    {
        return Err(io::Error::other("prepared input identity binding mismatch").into());
    }
    let metadata_hashes = PrimaryMetadataChunkHashes::new(
        *prepared.graph().graph().receipt().axml_sha256(),
        *prepared.graph().graph().receipt().chna_sha256(),
        *prepared.dbmd().payload_sha256(),
    );
    let data_bytes = parsed
        .total_samples()
        .checked_mul(114)
        .ok_or_else(|| io::Error::other("selected PCM data length overflow"))?;
    let primary_binding = bind_admitted_selected_input(
        prepared.configuration(),
        SelectedPrimaryRuntimeIdentity {
            source_file_bytes: prepared.resource_identity().resource_bytes(),
            data_bytes,
            metadata_hashes,
            resource_sha256: prepared.resource_identity().input_sha256(),
            config_sha256: prepared.config_sha256(),
            job_xml_sha256: prepared.job_xml_sha256(),
            job_xml_bytes: u64::try_from(job_xml.len())
                .map_err(|_| io::Error::other("job XML length overflow"))?,
        },
    )
    .map_err(|error| io::Error::other(format!("primary input binding failed: {error:?}")))?;
    let primary_controls = produce_primary_metadata_control_rows(
        &primary_binding,
        prepared.graph().graph(),
        prepared.dbmd(),
    )
    .map_err(|error| io::Error::other(format!("primary controls failed: {error:?}")))?;
    let mut diffuse_hosts = Vec::new();
    let mut component_hosts = Vec::new();
    for row in primary_controls.rows() {
        if let PrimaryControl38Fields::Object { control52 } = row.fields() {
            let words = control52.words();
            if words[10] != 0 {
                diffuse_hosts.push(row.track().physical_host().value());
            }
            if words[11] != 0 {
                component_hosts.push(row.track().physical_host().value());
            }
        }
    }
    println!("adm_control52_word10_nonzero_hosts={diffuse_hosts:?}");
    println!("adm_control52_word11_nonzero_hosts={component_hosts:?}");
    let geometry = generate_selected_p11_geometry(0x111b, true)
        .map_err(|_| io::Error::other("selected P11 geometry generation failed"))?;
    let source_map = build_meter_installed48_source_map(&primary_controls, &geometry)
        .map_err(|_| io::Error::other("meter installed48 map construction failed"))?;
    let host10_binding = source_map
        .physical_to_global_source()
        .iter()
        .find(|binding| binding.physical_host().value() == 10)
        .ok_or_else(|| io::Error::other("selected host 10 source binding is missing"))?;
    let host10_audio_event_diffuse = source_map
        .row_for_source(host10_binding.source_index())
        .and_then(jocforge_domain::spatial::installed48_source_map::Installed48SourceRow::control52)
        .map(|control| control.words()[10])
        .ok_or_else(|| io::Error::other("selected host 10 Control52 is missing"))?;
    if host10_audio_event_diffuse != 1 {
        return Err(io::Error::other("selected host 10 event diffuse is not one").into());
    }
    let activity = PrimaryInstalled48ActivityProducer::new(&source_map)
        .map_err(|_| io::Error::other("primary installed48 activity setup failed"))?;
    let mut activity_state = activity
        .cold_state()
        .map_err(|_| io::Error::other("primary installed48 activity cold state failed"))?;
    let selected_profile = prepared.configuration().selected_profile();
    let (diffusion_parameters, diffusion_manifest_sha256) = selected_diffusion_parameters()?;
    let renderer_source_sha256 = *source_map.source_binding().source_receipt_sha256();
    let mut renderer_state = MeterInstalled48State::cold(renderer_source_sha256, source_samples);
    println!("caller_job=F01E_fixture_settings_with_selected_file_reference");
    println!(
        "caller_job_xml_sha256={}",
        hex_lower(&prepared.job_xml_sha256())
    );
    println!(
        "caller_config_sha256={}",
        hex_lower(&prepared.config_sha256())
    );
    println!("resource_bytes={}", resource.len());
    println!("resource_sha256={}", hex_lower(&source_identity.sha256()));
    println!("sample_rate_hz={}", parsed.sample_rate_hz());
    println!("channels={}", parsed.channels());
    println!("source_samples_per_channel={}", parsed.total_samples());
    println!("data_offset_bytes={}", parsed.data_offset());
    println!("axml_sha256={}", hex_lower(&Sha256::digest(axml)));
    println!("chna_sha256={}", hex_lower(&Sha256::digest(chna)));
    println!("dbmd_sha256={}", hex_lower(&Sha256::digest(dbmd_payload)));
    println!(
        "physical_track_count={}",
        prepared.graph().graph().receipt().physical_track_count()
    );
    println!(
        "admitted_object_event_count={}",
        admitted_events.events().len()
    );
    println!("dbmd_track_count={}", dbmd.track_fields().len());
    println!("f04a_diffusion_manifest_sha256={diffusion_manifest_sha256}");
    println!("meter_installed48_source_rows={}", source_map.rows().len());
    println!(
        "meter_primary_control_rows={}",
        primary_controls.row_count()
    );

    let mut cursor = MeterPcmSourceCursor::cold(&parsed, source_identity, source_samples)
        .map_err(|_| io::Error::other("selected meter source cursor cold admission failed"))?;
    let (route, _) =
        source_flags_inputs(&source_map, selected_profile, FrontendCallbackIndex::new(0))?;
    let mut previous_regular_rows = vec![vec![0_u32; 1_536]; route.regular_hosts().len()];
    let mut f04g_actual_callbacks = Vec::with_capacity(3);
    for callback in 0..3_u64 {
        let proposal =
            stage_meter_pcm_source_frame(&cursor, MeterCallbackIndex::new(callback), &parsed)
                .map_err(|_| io::Error::other("selected meter source frame staging failed"))?;
        let (next_cursor, frame) = proposal.into_parts();
        if frame.rows().len() != 38 || frame.rows().iter().any(|row| row.len() != 1_536) {
            return Err(io::Error::other("meter raw input frame shape mismatch").into());
        }
        let mut frame_digest = Sha256::new();
        for row in frame.rows() {
            for word in row {
                frame_digest.update(word.to_le_bytes());
            }
        }
        let activity_proposal = activity
            .produce_next(
                &source_map,
                &activity_state,
                PrimaryActivityInput::NormalRead {
                    reader_relative_sample: AdmSampleIndex::new(
                        frame.source_cursor_before().value(),
                    ),
                    pcm_word_rows: frame.rows().to_vec(),
                },
            )
            .map_err(|_| io::Error::other("meter primary activity stage failed"))?;
        let (activity_output, next_activity_state, membership_route) =
            activity_proposal.into_parts();
        if u64::from(activity_output.callback_index().value()) != callback {
            return Err(io::Error::other("activity callback generation mismatch").into());
        }
        f04g_actual_callbacks.push(f04g_actual_callback_record(
            &frame,
            &activity_output,
            &membership_route,
            &source_map,
        )?);
        let callback_index = FrontendCallbackIndex::new(
            u32::try_from(callback)
                .map_err(|_| io::Error::other("callback index exceeds its U32 domain"))?,
        );
        let host10_feature = SelectedFeatureFlagsControl::from_admitted_profile(
            selected_profile,
            callback_index,
            10,
        )
        .map_err(|error| {
            io::Error::other(format!(
                "selected host 10 feature control: {}",
                error.code()
            ))
        })?;
        let (route, controls) = source_flags_inputs(&source_map, selected_profile, callback_index)?;
        let prefix_writer_sources = membership_route.writer_indices();
        let values = SourceFlagsProducer::produce_meter_values(
            frame.rows(),
            &previous_regular_rows,
            &controls,
            &route,
            &prefix_writer_sources,
        )
        .map_err(|error| {
            io::Error::other(format!(
                "selected F03R value stage failed: {}",
                error.code()
            ))
        })?;
        let mut flag_pair_counts = [0_usize; 4];
        for pair in values.payload() {
            let bucket = match pair {
                [0, 0] => 0,
                [0, 1] => 1,
                [1, 0] => 2,
                [1, 1] => 3,
                _ => return Err(io::Error::other("F03R payload flag pair is outside U1xU1").into()),
            };
            flag_pair_counts[bucket] += 1;
        }
        if flag_pair_counts.iter().sum::<usize>() != 48 {
            return Err(io::Error::other("F03R payload count is not 48 pairs").into());
        }
        let mut flags_digest = Sha256::new();
        for pair in values.payload() {
            flags_digest.update(pair);
        }
        previous_regular_rows = values.raw_history_rows().map(|row| row.to_vec()).collect();
        let render_proposal = stage_meter_installed48(
            &renderer_state,
            MeterInstalled48Input {
                source_sha256: renderer_source_sha256,
                callback_index: frame.callback_index(),
                source_cursor_before: AdmSampleIndex::new(frame.source_cursor_before().value()),
                host_input_samples: u16::try_from(frame.host_input_count().value())
                    .map_err(|_| io::Error::other("meter host sample count exceeds u16"))?,
                source_padding_samples: u16::try_from(frame.source_padding_count().value())
                    .map_err(|_| io::Error::other("meter padding sample count exceeds u16"))?,
                raw_pcm_rows: frame.rows(),
                activity: &activity_output,
                membership_route: &membership_route,
                source_map: &source_map,
                controls: &primary_controls,
                diffusion_parameters: Some(&diffusion_parameters),
            },
        )
        .map_err(|error| {
            io::Error::other(format!("selected F04A leaf stage failed: {}", error.code()))
        })?;
        let (next_renderer_state, render_frame) = render_proposal.into_parts();
        if render_frame.callback_index() != frame.callback_index()
            || render_frame.upstream_generation() != frame.upstream_generation().value()
        {
            return Err(io::Error::other("F04A callback generation mismatch").into());
        }
        let mut render_digest = Sha256::new();
        for row in render_frame.rows() {
            for word in row {
                render_digest.update(word.to_le_bytes());
            }
        }
        println!(
            "callback={} phase={:?} generation={} cursor_before={} cursor_after={} host_samples={} padded_samples={} rows=38x1536 f32_words_sha256={} quiet37_sha256={} prefix_writers={:?} membership_rows={} f03r_payload=48x2 sha256={} pair_counts_00_01_10_11={:?} host10_feature_host={} host10_feature_callback={} host10_feature_component_enum={} host10_feature_diffuse={} f04a_leaf=48x3072 generation={} sha256={}",
            frame.callback_index().value(),
            frame.phase(),
            frame.upstream_generation().value(),
            frame.source_cursor_before().value(),
            frame.source_cursor_after().value(),
            frame.host_input_count().value(),
            frame.source_padding_count().value(),
            hex_lower(&frame_digest.finalize()),
            hex_lower(&Sha256::digest(activity_output.quiet37())),
            prefix_writer_sources,
            membership_route.contributions().len(),
            hex_lower(&flags_digest.finalize()),
            flag_pair_counts,
            host10_feature.host().value(),
            host10_feature.callback_index().value(),
            host10_feature.component_enum(),
            host10_feature.diffuse(),
            render_frame.upstream_generation(),
            hex_lower(&render_digest.finalize()),
        );
        cursor = next_cursor;
        activity_state = next_activity_state;
        renderer_state = next_renderer_state;
    }
    let host10_audio_event_diffuse_after = source_map
        .row_for_source(host10_binding.source_index())
        .and_then(jocforge_domain::spatial::installed48_source_map::Installed48SourceRow::control52)
        .map(|control| control.words()[10])
        .ok_or_else(|| io::Error::other("selected host 10 Control52 disappeared"))?;
    if host10_audio_event_diffuse_after != host10_audio_event_diffuse {
        return Err(io::Error::other("F03R input changed F01E Control52 word10").into());
    }
    println!(
        "host10_audio_event_diffuse_word10={host10_audio_event_diffuse_after} preserved_for_F04A"
    );
    println!("f03r_input=selected_profile_typed_controls; event_diffuse_not_reused");
    println!("value_stages=F03R_VALUE_ONLY_AND_F04A_LEAF");
    println!("source_flags_packet=NOT_CREATED");
    println!("f04a_leaf=separate_noncommitting_selected_value_transition");
    println!("callback_commit=NOT_COMMITTED");
    if let Some(output_directory) = env::var_os("JOCFORGE_F04G_ACTUAL_INPUTS_OUT") {
        write_f04g_actual_input_package(
            Path::new(&output_directory),
            &prepared,
            &geometry,
            &hex_lower(&Sha256::digest(axml)),
            &hex_lower(&Sha256::digest(chna)),
            &hex_lower(&Sha256::digest(dbmd_payload)),
            &f04g_actual_callbacks,
        )?;
    }
    Ok(())
}

fn f04g_actual_callback_record(
    frame: &MeterRawInputFrame,
    activity: &PrimaryActivityOutput,
    membership_route: &PrimaryPrefixRoute,
    source_map: &Installed48SourceMap,
) -> Result<serde_json::Value, io::Error> {
    let mut physical_source_rows = Vec::with_capacity(37);
    for (regular_ordinal, binding) in source_map.physical_to_global_source().iter().enumerate() {
        let source_index = binding.source_index().value();
        let row = source_map
            .row_for_source(binding.source_index())
            .ok_or_else(|| io::Error::other("F04G actual input source row is missing"))?;
        let quiet = *activity
            .quiet37()
            .get(regular_ordinal)
            .ok_or_else(|| io::Error::other("F04G actual quiet flag is missing"))?;
        let input_mute = row
            .input_mute()
            .ok_or_else(|| io::Error::other("F04G actual input mute is missing"))?;
        let control52 = row.control52();
        let membership_words = control52.map(|control| {
            let words = control.words();
            [
                words[5], words[6], words[7], words[4], words[9], words[8], words[11], words[12],
            ]
        });
        let membership_actual = membership_route
            .contributions()
            .iter()
            .find(|contribution| contribution.source_index().value() == source_index)
            .map(|contribution| {
                serde_json::json!({
                    "gain_words_u32": contribution.result().gain_words(),
                    "positive_members": contribution.result().positive_members()
                })
            });
        physical_source_rows.push(serde_json::json!({
            "physical_host": binding.physical_host().value(),
            "source_index": source_index,
            "kind": row.kind(),
            "quiet": quiet,
            "input_mute": input_mute,
            "control52_membership_words_u32": membership_words,
            "membership_actual_output": membership_actual
        }));
    }
    Ok(serde_json::json!({
        "frontend_callback_g": frame.callback_index().value(),
        "phase": format!("{:?}", frame.phase()),
        "upstream_generation": frame.upstream_generation().value(),
        "source_interval": [frame.source_cursor_before().value(), frame.source_cursor_after().value()],
        "actual_source_samples": frame.host_input_count().value(),
        "quiet37_u8": activity.quiet37().to_vec(),
        "prefix_writer_sources": membership_route.writer_indices(),
        "membership_contribution_count": membership_route.contributions().len(),
        "physical_source_rows": physical_source_rows
    }))
}

fn write_f04g_actual_input_package(
    output_directory: &Path,
    prepared: &jocforge_application::PreparedJobInput,
    geometry: &jocforge_domain::spatial::selected_p11_geometry::SelectedP11Geometry,
    axml_sha256: &str,
    chna_sha256: &str,
    dbmd_sha256: &str,
    callbacks: &[serde_json::Value],
) -> Result<(), io::Error> {
    fs::create_dir_all(output_directory)?;
    let payload = serde_json::json!({
        "schema": "f04g-actual-membership-inputs.v1",
        "package_id": "F04G_ACTUAL_MEMBERSHIP_INPUTS_V1",
        "source_identity": {
            "resource_bytes": prepared.resource_identity().resource_bytes(),
            "resource_sha256": hex_lower(&prepared.resource_identity().input_sha256()),
            "axml_sha256": axml_sha256,
            "chna_sha256": chna_sha256,
            "dbmd_sha256": dbmd_sha256,
            "job_xml_sha256": hex_lower(&prepared.job_xml_sha256()),
            "config_sha256": hex_lower(&prepared.config_sha256())
        },
        "selected_profile": "ADM / 48 kHz / 38-channel PCM24 / 5.1",
        "p11_geometry_f32le_sha256": hex_lower(geometry.f32le_sha256()),
        "pcm_policy": {
            "raw_pcm_included": false,
            "raw_pcm_persisted": false,
            "quiet37_values_are_activity_outputs": true
        },
        "expected_outputs_included": false,
        "callbacks": callbacks
    });
    let payload_bytes = serde_json::to_vec_pretty(&payload)
        .map_err(|_| io::Error::other("F04G actual-input JSON serialization failed"))?;
    let payload_sha256 = hex_lower(&Sha256::digest(&payload_bytes));
    fs::write(
        output_directory.join("F04G-ACTUAL-INPUTS.json"),
        &payload_bytes,
    )?;
    let manifest = serde_json::json!({
        "schema": "f04g-actual-input-capture-manifest.v1",
        "payloads": [{
            "file": "F04G-ACTUAL-INPUTS.json",
            "bytes": payload_bytes.len(),
            "sha256": payload_sha256
        }],
        "expected_outputs_included": false,
        "raw_pcm_included": false
    });
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|_| io::Error::other("F04G actual-input manifest serialization failed"))?;
    let manifest_sha256 = hex_lower(&Sha256::digest(&manifest_bytes));
    fs::write(output_directory.join("MANIFEST.json"), manifest_bytes)?;
    println!(
        "f04g_actual_input_package=F04G-ACTUAL-INPUTS.json bytes={} sha256={} manifest_sha256={}",
        payload_bytes.len(),
        payload_sha256,
        manifest_sha256
    );
    Ok(())
}

fn selected_diffusion_parameters() -> Result<(DiffusionFilterParameters, String), io::Error> {
    let package_manifest = PackageManifestEvidence::new(Arc::from(F04_PACKAGE_MANIFEST), None);
    let manifest =
        validate_diffusion_filter_package_manifest(&package_manifest).map_err(|error| {
            io::Error::other(format!("diffusion package validation: {}", error.code()))
        })?;
    let manifest_sha256 = manifest.manifest_sha256().to_hex();
    let coefficients = ProviderFileEvidence::new(Arc::from(DIFFUSION_COEFFICIENTS), None);
    let constructor = ProviderFileEvidence::new(Arc::from(DIFFUSION_CONSTRUCTOR), None);
    let parameters = validate_diffusion_filter_parameters(&coefficients, &constructor, &manifest)
        .map_err(|error| {
        io::Error::other(format!("diffusion parameter validation: {}", error.code()))
    })?;
    Ok((parameters, manifest_sha256))
}

fn source_flags_inputs(
    source_map: &Installed48SourceMap,
    profile: ResolvedEncodingProfile,
    callback_index: FrontendCallbackIndex,
) -> Result<(ValidatedSemanticRoute, RegularControls), io::Error> {
    let mut records_by_host = [None; 38];
    for binding in source_map.physical_to_global_source() {
        let host = binding.physical_host().value();
        let destination = u8::try_from(binding.source_index().value())
            .map_err(|_| io::Error::other("installed source index is out of range"))?;
        records_by_host[usize::from(host)] = Some(
            HostRouteRecord::new(host, HostRole::Regular, true, Some(destination))
                .map_err(|error| io::Error::other(format!("semantic route: {}", error.code())))?,
        );
    }
    let mut records = Vec::with_capacity(38);
    for (host, record) in records_by_host.iter().enumerate() {
        let host = u8::try_from(host)
            .map_err(|_| io::Error::other("physical host index is out of range"))?;
        records.push(
            record.unwrap_or(
                HostRouteRecord::new(host, HostRole::Lfe, true, None)
                    .map_err(|error| io::Error::other(format!("LFE route: {}", error.code())))?,
            ),
        );
    }
    let route = ValidatedSemanticRoute::new(records)
        .map_err(|error| io::Error::other(format!("semantic route: {}", error.code())))?;
    let mut controls = Vec::with_capacity(37);
    for host in route.regular_hosts() {
        let binding = source_map
            .physical_to_global_source()
            .iter()
            .find(|binding| binding.physical_host().value() == host.value())
            .ok_or_else(|| io::Error::other("physical host source binding is missing"))?;
        let row = source_map
            .row_for_source(binding.source_index())
            .ok_or_else(|| io::Error::other("installed source row is missing"))?;
        let mute = row
            .input_mute()
            .ok_or_else(|| io::Error::other("installed source mute is missing"))?;
        let width = f32::from_bits(row.metadata_words_8_to_55()[8]);
        let feature = SelectedFeatureFlagsControl::from_admitted_profile(
            profile,
            callback_index,
            host.value(),
        )
        .map_err(|error| {
            io::Error::other(format!("selected F03R feature control: {}", error.code()))
        })?;
        controls.push(feature.into_regular_control(mute, width).map_err(|error| {
            io::Error::other(format!("selected F03R regular control: {}", error.code()))
        })?);
    }
    let controls = RegularControls::new(controls)
        .map_err(|error| io::Error::other(format!("regular controls: {}", error.code())))?;
    Ok((route, controls))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
