// pattern: Imperative Shell

use std::{
    env,
    fmt::Write as _,
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::ExitCode,
};

mod file_scan;

use file_scan::{read_pcm_frame, scan_bw64_file};

use jocforge_core::{
    AdmObject, AdmObjectKind, JocProfile, ReductionConfig, ReductionSource,
    factorize_source_derived, generate_synthetic_scene_fixture,
    package_source_derived_access_unit_with_frame_sizes, parse_bw64_adm_metadata, profile_spec,
    rate_budget, reduce_scene_with_previous, write_one_au_proof,
};

const JOC_FRAME_SAMPLES: usize = 1536;

fn main() -> ExitCode {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match run(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("jocforge: {error}");
            ExitCode::from(2)
        }
    }
}

fn run(arguments: &[String]) -> Result<(), String> {
    match arguments.first().map(String::as_str) {
        Some("proof") => run_proof(&arguments[1..]),
        Some("fixture") => run_fixture(&arguments[1..]),
        Some("fixture-directspeakers") => run_direct_speaker_fixture(&arguments[1..]),
        Some("fixture-unsupported") => run_unsupported_fixture(&arguments[1..]),
        Some("inspect-adm") => run_inspect_adm(&arguments[1..]),
        Some("encode") => run_encode(&arguments[1..]),
        Some("temporal") => run_temporal(&arguments[1..]),
        Some("temporal-adversarial") => run_temporal_adversarial(&arguments[1..]),
        Some(command) => Err(format!("unsupported command '{command}'")),
        None => Err(
            "usage: jocforge proof|fixture|fixture-directspeakers|fixture-unsupported|inspect-adm|encode|temporal ..."
                .to_owned(),
        ),
    }
}

fn run_temporal(arguments: &[String]) -> Result<(), String> {
    let [output] = arguments else {
        return Err("usage: jocforge temporal OUTPUT.csv".to_owned());
    };
    write_temporal_metrics(
        &PathBuf::from(output),
        jocforge_core::temporal_corpus(),
        ReductionConfig::default(),
    )
}

fn run_temporal_adversarial(arguments: &[String]) -> Result<(), String> {
    let (output, config) = match arguments {
        [output] => (output.as_str(), ReductionConfig::default()),
        [flag, output] if flag == "--no-continuity" => (
            output.as_str(),
            ReductionConfig {
                temporal_continuity_weight: 0.0,
                ..ReductionConfig::default()
            },
        ),
        _ => {
            return Err(
                "usage: jocforge temporal-adversarial [--no-continuity] OUTPUT.csv".to_owned(),
            );
        }
    };
    write_temporal_metrics(
        &PathBuf::from(output),
        jocforge_core::adversarial_temporal_corpus(),
        config,
    )
}

fn write_temporal_metrics(
    output: &Path,
    scenes: Vec<jocforge_core::TemporalScene>,
    config: ReductionConfig,
) -> Result<(), String> {
    let mut csv = String::from(
        "scene,frame_count,authored_source_count,maximum_coded_object_count,silent_object_drops,maximum_source_reconstruction_error,maximum_spatial_representation_error,maximum_cross_object_leakage,minimum_headroom_margin,total_assignment_changes,total_coded_identity_changes,assignment_changes_per_minute,max_sliding_window_changes,minimum_coded_identity_lifetime_frames,median_coded_identity_lifetime_frames,reassignment_burstiness,maximum_position_jump,maximum_trajectory_error,maximum_gain_jump,maximum_pcm_boundary_discontinuity,maximum_reconstruction_error_change,maximum_speaker_render_discontinuity,stationary_assignment_churn_after_initialization\n",
    );
    for scene in scenes {
        let metrics = jocforge_core::analyze_temporal_scene_with_config(&scene, config)?;
        let _ = writeln!(
            csv,
            "{},{},{},{},{},{:.17e},{:.17e},{:.17e},{:.17e},{},{},{:.17e},{},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{}",
            metrics.scene,
            metrics.frame_count,
            metrics.authored_source_count,
            metrics.maximum_coded_object_count,
            metrics.silent_object_drops,
            metrics.maximum_source_reconstruction_error,
            metrics.maximum_spatial_representation_error,
            metrics.maximum_cross_object_leakage,
            metrics.minimum_headroom_margin,
            metrics.total_assignment_changes,
            metrics.total_coded_identity_changes,
            metrics.assignment_changes_per_minute,
            metrics.max_sliding_window_changes,
            metrics.minimum_coded_identity_lifetime_frames,
            metrics.median_coded_identity_lifetime_frames,
            metrics.reassignment_burstiness,
            metrics.maximum_position_jump,
            metrics.maximum_trajectory_error,
            metrics.maximum_gain_jump,
            metrics.maximum_pcm_boundary_discontinuity,
            metrics.maximum_reconstruction_error_change,
            metrics.maximum_speaker_render_discontinuity,
            metrics.stationary_assignment_churn_after_initialization,
        );
    }
    write_output(output, csv.into_bytes(), "temporal metrics")
}

fn run_proof(arguments: &[String]) -> Result<(), String> {
    let [profile, output] = arguments else {
        return Err("usage: jocforge proof idx0|idx1|idx2 OUTPUT.ec3".to_owned());
    };
    let profile = parse_profile(profile)?;
    let bytes = write_one_au_proof(profile)?;
    let output = PathBuf::from(output);
    write_output(&output, bytes, "proof")
}

fn run_fixture(arguments: &[String]) -> Result<(), String> {
    if arguments.len() != 3 && arguments.len() != 5 {
        return Err("usage: jocforge fixture --seconds N OUTPUT.bw64".to_owned());
    }
    if arguments[0] != "--seconds" {
        return Err("usage: jocforge fixture --seconds N OUTPUT.bw64".to_owned());
    }
    let seconds = arguments[1]
        .parse::<u32>()
        .map_err(|error| format!("invalid fixture duration '{}': {error}", arguments[1]))?;
    let (object_count, output_index) = if arguments.len() == 3 {
        (15, 2)
    } else {
        if arguments[2] != "--objects" {
            return Err("usage: jocforge fixture --seconds N [--objects N] OUTPUT.bw64".to_owned());
        }
        let object_count = arguments[3]
            .parse::<usize>()
            .map_err(|error| format!("invalid fixture object count '{}': {error}", arguments[3]))?;
        (object_count, 4)
    };
    let fixture = generate_synthetic_scene_fixture(seconds, object_count)?;
    let output = PathBuf::from(&arguments[output_index]);
    write_output(&output, fixture.bw64_header_and_pcm, "fixture")
}

fn run_direct_speaker_fixture(arguments: &[String]) -> Result<(), String> {
    let [flag, seconds, output] = arguments else {
        return Err("usage: jocforge fixture-directspeakers --seconds N OUTPUT.bw64".to_owned());
    };
    if flag != "--seconds" {
        return Err("usage: jocforge fixture-directspeakers --seconds N OUTPUT.bw64".to_owned());
    }
    let seconds = seconds
        .parse::<u32>()
        .map_err(|error| format!("invalid fixture duration '{seconds}': {error}"))?;
    let fixture = jocforge_core::generate_synthetic_direct_speaker_fixture(seconds)?;
    write_output(
        &PathBuf::from(output),
        fixture.bw64_header_and_pcm,
        "DirectSpeakers fixture",
    )
}

fn run_unsupported_fixture(arguments: &[String]) -> Result<(), String> {
    let [flag, seconds, output] = arguments else {
        return Err("usage: jocforge fixture-unsupported --seconds N OUTPUT.bw64".to_owned());
    };
    if flag != "--seconds" {
        return Err("usage: jocforge fixture-unsupported --seconds N OUTPUT.bw64".to_owned());
    }
    let seconds = seconds
        .parse::<u32>()
        .map_err(|error| format!("invalid fixture duration '{seconds}': {error}"))?;
    let fixture = jocforge_core::generate_synthetic_unsupported_fixture(seconds)?;
    write_output(
        &PathBuf::from(output),
        fixture.bw64_header_and_pcm,
        "unsupported fixture",
    )
}

fn run_inspect_adm(arguments: &[String]) -> Result<(), String> {
    let (input, inventory_output) = match arguments {
        [input] => (input.as_str(), None),
        [input, flag, output] if flag == "--direct-speakers-csv" => {
            (input.as_str(), Some(output.as_str()))
        }
        _ => {
            return Err(
                "usage: jocforge inspect-adm INPUT.bw64 [--direct-speakers-csv OUTPUT.csv]"
                    .to_owned(),
            );
        }
    };
    let input = PathBuf::from(input);
    let scanned = scan_bw64_file(&input)?;
    let parsed = parse_bw64_adm_metadata(
        &scanned.fmt,
        &scanned.axml,
        &scanned.chna,
        scanned.data_size,
    )
    .map_err(|error| format!("failed to inspect ADM: {error}"))?;
    let chunk_ids = scanned
        .chunks
        .iter()
        .map(|chunk| String::from_utf8_lossy(&chunk.id).to_string())
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "form={} chunks={} chunk_ids={} data_offset={} data_bytes={} dbmd_bytes={} sample_rate={} channels={} samples={} tracks={} track_uids={} objects={} dynamic_objects={} direct_speakers={} direct_speaker_channels={}",
        String::from_utf8_lossy(&scanned.form),
        scanned.chunks.len(),
        chunk_ids,
        scanned.data_offset,
        scanned.data_size,
        scanned.dbmd.len(),
        parsed.sample_rate,
        parsed.channel_count,
        parsed.sample_count,
        parsed.track_count,
        parsed.track_uid_count,
        parsed.objects.len(),
        parsed.dynamic_object_count(),
        parsed.direct_speaker_count(),
        parsed.direct_speaker_channel_count()
    );
    if let Some(output) = inventory_output {
        write_direct_speaker_inventory(&PathBuf::from(output), &parsed)?;
    }
    Ok(())
}

fn write_direct_speaker_inventory(
    path: &Path,
    metadata: &jocforge_core::ParsedAdmMetadata,
) -> Result<(), String> {
    let programme_relationship = metadata
        .programmes
        .iter()
        .flat_map(|programme| {
            programme
                .content_id_refs
                .iter()
                .map(|content_id| format!("{} -> {content_id}", programme.programme_id))
        })
        .collect::<Vec<_>>()
        .join(";");
    let mut output = String::from(
        "object_id,object_name,pack_format_id,pack_format_name,channel_id,channel_name,track_uid,track_index,is_lfe,azimuth,elevation,distance,object_start,object_duration,block_rtime,block_duration,gain,programme_relationship\n",
    );
    for object in metadata
        .objects
        .iter()
        .filter(|object| object.kind == AdmObjectKind::DirectSpeakers)
    {
        for channel in &object.channels {
            let _ = writeln!(
                output,
                "{},{},{},{},{},{},{},{},{},{:.17e},{:.17e},{:.17e},{},{},{},{},{},{}",
                csv_cell(&object.object_id),
                csv_cell(&object.object_name),
                csv_cell(&object.pack_format_id),
                csv_cell(&object.pack_format_name),
                csv_cell(&channel.channel_id),
                csv_cell(&channel.channel_name),
                csv_cell(&channel.track_uid),
                channel.track_index,
                channel.is_lfe,
                channel.position.azimuth,
                channel.position.elevation,
                channel.position.distance,
                csv_cell(object.start.as_deref().unwrap_or("")),
                csv_cell(object.duration.as_deref().unwrap_or("")),
                csv_cell(channel.block_rtime.as_deref().unwrap_or("")),
                csv_cell(channel.block_duration.as_deref().unwrap_or("")),
                csv_cell(channel.gain.as_deref().unwrap_or("")),
                csv_cell(&programme_relationship),
            );
        }
    }
    write_output(path, output.into_bytes(), "DirectSpeakers inventory")
}

fn csv_cell(value: &str) -> String {
    if value.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn run_encode(arguments: &[String]) -> Result<(), String> {
    if arguments.len() < 4 || (arguments.len() - 4) % 2 != 0 {
        return Err(
            "usage: jocforge encode INPUT.bw64 OUTPUT.ec3 --profile idx0|idx1|idx2 [--bitrate KBPS] [--temporal-weight VALUE]"
                .to_owned(),
        );
    }
    let input = &arguments[0];
    let output = &arguments[1];
    let flag = &arguments[2];
    let profile_name = &arguments[3];
    if flag != "--profile" {
        return Err(
            "usage: jocforge encode INPUT.bw64 OUTPUT.ec3 --profile idx0|idx1|idx2 [--bitrate KBPS] [--temporal-weight VALUE]"
                .to_owned(),
        );
    }
    let mut target_bitrate_kbps = 1664;
    let mut temporal_continuity_weight = ReductionConfig::default().temporal_continuity_weight;
    let mut option_index = 4;
    while option_index < arguments.len() {
        let option = &arguments[option_index];
        let value = arguments
            .get(option_index + 1)
            .ok_or_else(|| "encode option is missing its value".to_owned())?;
        match option.as_str() {
            "--bitrate" => {
                target_bitrate_kbps = value
                    .parse::<u32>()
                    .map_err(|error| format!("invalid total bitrate '{value}': {error}"))?;
            }
            "--temporal-weight" => {
                temporal_continuity_weight = value.parse::<f64>().map_err(|error| {
                    format!("invalid temporal continuity weight '{value}': {error}")
                })?;
            }
            _ => {
                return Err(
                    "usage: jocforge encode INPUT.bw64 OUTPUT.ec3 --profile idx0|idx1|idx2 [--bitrate KBPS] [--temporal-weight VALUE]"
                        .to_owned(),
                );
            }
        }
        option_index += 2;
    }
    let rate = rate_budget(target_bitrate_kbps)
        .map_err(|error| format!("invalid encode budget: {error}"))?;
    let profile = parse_profile(profile_name)?;
    let input_path = PathBuf::from(input);
    let scanned = scan_bw64_file(&input_path)?;
    let metadata = parse_bw64_adm_metadata(
        &scanned.fmt,
        &scanned.axml,
        &scanned.chna,
        scanned.data_size,
    )
    .map_err(|error| {
        format!(
            "failed to parse ADM metadata '{}': {error}",
            input_path.display()
        )
    })?;
    if let Some(object) = metadata
        .objects
        .iter()
        .find(|object| object.kind == AdmObjectKind::Other)
    {
        return Err(format!(
            "unsupported ADM object kind for {}: only Objects and DirectSpeakers are supported",
            object.object_id
        ));
    }
    let supported_objects = metadata
        .objects
        .iter()
        .filter(|object| object.kind != AdmObjectKind::Other)
        .collect::<Vec<_>>();
    if supported_objects.is_empty() {
        return Err("ADM scene has no supported authored objects".to_owned());
    }
    let track_indices = supported_objects
        .iter()
        .flat_map(|object| object.channels.iter().map(|channel| channel.track_index))
        .collect::<Vec<_>>();
    let lfe_source_indices = supported_objects
        .iter()
        .flat_map(|object| object.channels.iter())
        .enumerate()
        .filter_map(|(index, channel)| channel.is_lfe.then_some(index))
        .collect::<Vec<_>>();
    let i0_channels = 5 + usize::from(!lfe_source_indices.is_empty());
    let rank = profile_spec(profile).input_labels.len();
    let frame_count = metadata.sample_count.div_ceil(JOC_FRAME_SAMPLES);
    let encoded_samples = frame_count
        .checked_mul(JOC_FRAME_SAMPLES)
        .ok_or_else(|| "ADM encoded sample count overflows".to_owned())?;
    let work_root = env::temp_dir().join(format!("jocforge-encode-{}", std::process::id()));
    fs::create_dir_all(&work_root)
        .map_err(|error| format!("failed to create encoder work directory: {error}"))?;
    let i0_raw_path = work_root.join("carrier-i0.f32le");
    let i0_carrier_path = work_root.join("carrier-i0.ec3");
    let reduction_config = ReductionConfig {
        temporal_continuity_weight,
        ..ReductionConfig::default()
    };
    let mut metrics = String::from(
        "record,frame,source_index,object_id,kind,coded_object_index,contribution_gain,source_energy,represented_energy,relative_error,spatial_error,source_count,coded_count,silent_drops,source_reconstruction_error,spatial_representation_error,cross_object_leakage,max_coded_peak,headroom_margin,assignment_changes,coded_identity_changes,coded_representative_source_index,source_azimuth,source_elevation,source_distance,coded_azimuth,coded_elevation,coded_distance,coded_first_sample,coded_last_sample\n",
    );
    let mut previous_reduction = None;
    let mut raw_file = fs::File::create(&i0_raw_path).map_err(|error| {
        format!(
            "failed to create carrier PCM '{}': {error}",
            i0_raw_path.display()
        )
    })?;
    for frame_index in 0..frame_count {
        let source_pcm = read_pcm_frame(&input_path, &scanned, frame_index, &track_indices)?;
        let lfe_pcm = lfe_carrier(&source_pcm, &lfe_source_indices)?;
        let sources = reduction_sources(&supported_objects, &source_pcm)?;
        let reduction =
            reduce_scene_with_previous(&sources, reduction_config, previous_reduction.as_ref())
                .map_err(|error| {
                    format!("failed to reduce ADM scene at frame {frame_index}: {error}")
                })?;
        let object_pcm = reduction
            .coded_objects
            .iter()
            .map(|object| object.signal.clone())
            .collect::<Vec<_>>();
        let factorization = factorize_source_derived(&object_pcm, rank).map_err(|error| {
            format!("failed to factorize reduced objects at frame {frame_index}: {error}")
        })?;
        let mut i0_carriers = factorization.carriers[..5].to_vec();
        if !lfe_source_indices.is_empty() {
            i0_carriers.push(lfe_pcm);
        }
        raw_file
            .write_all(&interleaved_f32le(&i0_carriers, JOC_FRAME_SAMPLES))
            .map_err(|error| format!("failed to write carrier PCM frame {frame_index}: {error}"))?;
        append_reduction_metrics(&mut metrics, frame_index, &sources, &reduction);
        append_lfe_metrics(
            &mut metrics,
            frame_index,
            &supported_objects,
            &source_pcm,
            &lfe_source_indices,
            reduction.coded_objects.len(),
        );
        previous_reduction = Some(reduction);
    }
    drop(raw_file);
    encode_ffmpeg_carrier(
        &i0_raw_path,
        &i0_carrier_path,
        i0_channels,
        &format!("{}k", rate.carrier_bitrate_kbps),
    )?;
    let i0_carrier_bytes =
        read_carrier_frames(&i0_carrier_path, rate.carrier_frame_bytes, frame_count)?;
    let mut output_bytes =
        Vec::with_capacity(frame_count * (rate.carrier_frame_bytes + rate.d0_frame_bytes));
    previous_reduction = None;
    for frame_index in 0..frame_count {
        let i0_start = frame_index * rate.carrier_frame_bytes;
        let i0_frame = &i0_carrier_bytes[i0_start..i0_start + rate.carrier_frame_bytes];
        let source_pcm = read_pcm_frame(&input_path, &scanned, frame_index, &track_indices)?;
        let sources = reduction_sources(&supported_objects, &source_pcm)?;
        let reduction =
            reduce_scene_with_previous(&sources, reduction_config, previous_reduction.as_ref())
                .map_err(|error| {
                    format!("failed to repeat ADM reduction at frame {frame_index}: {error}")
                })?;
        let object_pcm = reduction
            .coded_objects
            .iter()
            .map(|object| object.signal.clone())
            .collect::<Vec<_>>();
        let factorization = factorize_source_derived(&object_pcm, rank).map_err(|error| {
            format!("failed to repeat reduced factorization at frame {frame_index}: {error}")
        })?;
        let sequence = u16::try_from(frame_index % 1023 + 1).unwrap_or(1);
        let positions = reduction
            .coded_objects
            .iter()
            .map(|object| object.position)
            .collect::<Vec<_>>();
        let d0_carriers = if rank == 7 {
            factorization.carriers[5..7].to_vec()
        } else {
            factorization.carriers[..2].to_vec()
        };
        let access_unit = package_source_derived_access_unit_with_frame_sizes(
            profile,
            sequence,
            &factorization.scaled_matrix,
            &positions,
            i0_frame,
            &d0_carriers,
            0,
            rate.d0_frame_bytes,
        )
        .map_err(|error| format!("failed to package frame {frame_index}: {error}"))?;
        output_bytes.extend_from_slice(&access_unit);
        previous_reduction = Some(reduction);
    }
    let output_path = PathBuf::from(output);
    let output_bytes_len = output_bytes.len();
    write_output(&output_path, output_bytes, "encoded")?;
    let metrics_path = PathBuf::from(format!("{}.metrics.csv", output_path.display()));
    write_output(&metrics_path, metrics.into_bytes(), "reduction metrics")?;
    let _ = fs::remove_dir_all(work_root);
    println!(
        "encoded profile={profile_name} frames={frame_count} samples={encoded_samples} authored_sources={} authored_channels={} dynamic_objects={} direct_speakers={} direct_speaker_channels={} lfe_core_channels={} coded_objects_last_frame={} output_bytes={output_bytes_len} target_bitrate={} carrier_bitrate={} d0_bitrate={} temporal_weight={:.17e} reduction=deterministic:farthest-point+weighted-aggregate metrics={}",
        supported_objects.len(),
        track_indices.len(),
        metadata.dynamic_object_count(),
        metadata.direct_speaker_count(),
        metadata.direct_speaker_channel_count(),
        usize::from(!lfe_source_indices.is_empty()),
        previous_reduction
            .as_ref()
            .map_or(0, |result| result.coded_objects.len()),
        rate.total_bitrate_kbps * 1000,
        rate.carrier_bitrate_kbps * 1000,
        rate.d0_bitrate_kbps * 1000,
        reduction_config.temporal_continuity_weight,
        metrics_path.display(),
    );
    Ok(())
}

fn reduction_sources(
    objects: &[&AdmObject],
    pcm: &[Vec<f64>],
) -> Result<Vec<ReductionSource>, String> {
    let expected_sources = objects
        .iter()
        .map(|object| object.channels.len())
        .sum::<usize>();
    if expected_sources != pcm.len() {
        return Err(format!(
            "ADM source and PCM counts differ: {} channel bindings, {} tracks",
            expected_sources,
            pcm.len()
        ));
    }
    let mut sources = Vec::with_capacity(
        pcm.len().saturating_sub(
            objects
                .iter()
                .flat_map(|object| object.channels.iter())
                .filter(|channel| channel.is_lfe)
                .count(),
        ),
    );
    let mut flat_index = 0;
    for object in objects {
        for channel in &object.channels {
            let signal = pcm
                .get(flat_index)
                .cloned()
                .ok_or_else(|| "ADM source and PCM channel bindings are misaligned".to_owned())?;
            if channel.is_lfe {
                flat_index += 1;
                continue;
            }
            let object_id = if object.channels.len() == 1 {
                object.object_id.clone()
            } else {
                format!("{}:{}", object.object_id, channel.channel_id)
            };
            sources.push(ReductionSource {
                source_index: flat_index,
                object_id,
                kind: object.kind,
                position: channel.position,
                signal,
            });
            flat_index += 1;
        }
    }
    Ok(sources)
}

fn lfe_carrier(pcm: &[Vec<f64>], lfe_indices: &[usize]) -> Result<Vec<f64>, String> {
    let mut carrier = vec![0.0; JOC_FRAME_SAMPLES];
    for index in lfe_indices {
        let channel = pcm
            .get(*index)
            .ok_or_else(|| "LFE PCM binding is outside the selected frame".to_owned())?;
        if channel.len() != JOC_FRAME_SAMPLES {
            return Err("LFE PCM frame does not have the public frame length".to_owned());
        }
        for (output, value) in carrier.iter_mut().zip(channel) {
            *output += *value;
        }
    }
    Ok(carrier)
}

fn append_reduction_metrics(
    output: &mut String,
    frame_index: usize,
    sources: &[ReductionSource],
    reduction: &jocforge_core::ReductionResult,
) {
    for accounting in &reduction.source_accounting {
        let source = sources
            .iter()
            .find(|source| source.source_index == accounting.source_index);
        let (object_id, kind) = source.map_or(("unknown", "unknown"), |source| {
            (
                source.object_id.as_str(),
                match source.kind {
                    AdmObjectKind::Objects => "Objects",
                    AdmObjectKind::DirectSpeakers => "DirectSpeakers",
                    AdmObjectKind::Other => "Other",
                },
            )
        });
        let coded = reduction.coded_objects.get(accounting.coded_object_index);
        let source_position = source.map_or(
            jocforge_core::AdmPosition {
                azimuth: 0.0,
                elevation: 0.0,
                distance: 0.0,
            },
            |source| source.position,
        );
        let _ = writeln!(
            output,
            "source,{frame_index},{},{object_id},{kind},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{},{},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{},{},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
            accounting.source_index,
            accounting.coded_object_index,
            accounting.contribution_gain,
            accounting.source_energy,
            accounting.represented_energy,
            accounting.relative_error,
            accounting.spatial_error,
            reduction.diagnostics.authored_source_count,
            reduction.diagnostics.coded_object_count,
            reduction.diagnostics.silent_object_drops,
            reduction.diagnostics.source_reconstruction_error,
            reduction.diagnostics.spatial_representation_error,
            reduction.diagnostics.cross_object_leakage,
            reduction.diagnostics.max_coded_peak,
            reduction.diagnostics.headroom_margin,
            reduction.diagnostics.assignment_changes,
            reduction.diagnostics.coded_identity_changes,
            coded.map_or(usize::MAX, |object| object.representative_source_index),
            source_position.azimuth,
            source_position.elevation,
            source_position.distance,
            coded.map_or(0.0, |object| object.position.azimuth),
            coded.map_or(0.0, |object| object.position.elevation),
            coded.map_or(0.0, |object| object.position.distance),
            coded
                .and_then(|object| object.signal.first())
                .copied()
                .unwrap_or(0.0),
            coded
                .and_then(|object| object.signal.last())
                .copied()
                .unwrap_or(0.0),
        );
    }
}

fn append_lfe_metrics(
    output: &mut String,
    frame_index: usize,
    objects: &[&AdmObject],
    pcm: &[Vec<f64>],
    lfe_source_indices: &[usize],
    coded_count: usize,
) {
    let mut flat_index = 0;
    for object in objects {
        for channel in &object.channels {
            if !lfe_source_indices.contains(&flat_index) {
                flat_index += 1;
                continue;
            }
            let signal = &pcm[flat_index];
            let energy =
                signal.iter().map(|value| value * value).sum::<f64>() / signal.len().max(1) as f64;
            let _ = writeln!(
                output,
                "lfe,{frame_index},{},{}:{},DirectSpeakers,-1,1.00000000000000000e0,{:.17e},{:.17e},0.00000000000000000e0,0.00000000000000000e0,{},{},0,0.00000000000000000e0,0.00000000000000000e0,0.00000000000000000e0,0.00000000000000000e0,0.00000000000000000e0,0,0,0,{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                flat_index,
                object.object_id,
                channel.channel_id,
                energy,
                energy,
                pcm.len(),
                coded_count,
                channel.position.azimuth,
                channel.position.elevation,
                channel.position.distance,
                channel.position.azimuth,
                channel.position.elevation,
                channel.position.distance,
                signal.first().copied().unwrap_or(0.0),
                signal.last().copied().unwrap_or(0.0),
            );
            flat_index += 1;
        }
    }
}

fn encode_ffmpeg_carrier(
    raw_path: &Path,
    carrier_path: &Path,
    channels: usize,
    bitrate: &str,
) -> Result<(), String> {
    let ffmpeg = std::process::Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "f32le",
            "-ar",
            "48000",
            "-ac",
        ])
        .arg(channels.to_string())
        .args(["-i"])
        .arg(raw_path)
        .args(["-c:a", "eac3", "-b:a", bitrate, "-f", "eac3"])
        .arg(carrier_path)
        .status()
        .map_err(|error| format!("failed to start FFmpeg E-AC-3 encoder: {error}"))?;
    if !ffmpeg.success() {
        return Err(format!(
            "FFmpeg E-AC-3 {channels}-channel carrier encoding failed"
        ));
    }
    Ok(())
}

fn read_carrier_frames(
    path: &Path,
    frame_bytes: usize,
    expected_frames: usize,
) -> Result<Vec<u8>, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read FFmpeg carrier output '{}': {error}",
            path.display()
        )
    })?;
    if bytes.len() % frame_bytes != 0 {
        return Err(format!(
            "FFmpeg carrier output '{}' is not a sequence of {frame_bytes}-byte frames",
            path.display()
        ));
    }
    if bytes.len() / frame_bytes != expected_frames {
        return Err(format!(
            "FFmpeg carrier '{}' frame count differs: expected {expected_frames}, got {}",
            path.display(),
            bytes.len() / frame_bytes
        ));
    }
    Ok(bytes)
}

fn interleaved_f32le(carriers: &[Vec<f64>], samples: usize) -> Vec<u8> {
    let channels = carriers.len();
    let mut bytes = Vec::with_capacity(samples * channels * 4);
    for sample in 0..samples {
        for carrier in carriers {
            bytes.extend_from_slice(&(carrier[sample] as f32).to_le_bytes());
        }
    }
    bytes
}

fn parse_profile(value: &str) -> Result<JocProfile, String> {
    match value {
        "idx0" => Ok(JocProfile::Idx0),
        "idx1" => Ok(JocProfile::Idx1),
        "idx2" => Ok(JocProfile::Idx2),
        other => Err(format!("unsupported JOC profile '{other}'")),
    }
}

fn write_output(path: &Path, bytes: Vec<u8>, description: &str) -> Result<(), String> {
    fs::write(path, bytes).map_err(|error| {
        format!(
            "failed to write {description} output '{}': {error}",
            path.display()
        )
    })
}
