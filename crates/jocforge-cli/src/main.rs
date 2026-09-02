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
    factorize_source_derived, generate_synthetic_scene_fixture, package_source_derived_access_unit,
    parse_bw64_adm_metadata, profile_spec, reduce_scene_with_previous, write_one_au_proof,
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
        Some("inspect-adm") => run_inspect_adm(&arguments[1..]),
        Some("encode") => run_encode(&arguments[1..]),
        Some(command) => Err(format!("unsupported command '{command}'")),
        None => Err("usage: jocforge proof|fixture|inspect-adm ...".to_owned()),
    }
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

fn run_inspect_adm(arguments: &[String]) -> Result<(), String> {
    let [input] = arguments else {
        return Err("usage: jocforge inspect-adm INPUT.bw64".to_owned());
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
        "form={} chunks={} chunk_ids={} data_offset={} data_bytes={} dbmd_bytes={} sample_rate={} channels={} samples={} tracks={} track_uids={} objects={} dynamic_objects={} direct_speakers={}",
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
        parsed.direct_speaker_count()
    );
    Ok(())
}

fn run_encode(arguments: &[String]) -> Result<(), String> {
    let [input, output, flag, profile_name] = arguments else {
        return Err(
            "usage: jocforge encode INPUT.bw64 OUTPUT.ec3 --profile idx0|idx1|idx2".to_owned(),
        );
    };
    if flag != "--profile" {
        return Err(
            "usage: jocforge encode INPUT.bw64 OUTPUT.ec3 --profile idx0|idx1|idx2".to_owned(),
        );
    }
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
        .map(|object| object.track_index)
        .collect::<Vec<_>>();
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
    let reduction_config = ReductionConfig::default();
    let mut metrics = String::from(
        "record,frame,source_index,object_id,kind,coded_object_index,contribution_gain,source_energy,represented_energy,relative_error,spatial_error,source_count,coded_count,silent_drops,source_reconstruction_error,spatial_representation_error,cross_object_leakage,max_coded_peak,headroom_margin,assignment_changes\n",
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
        let sources = reduction_sources(&supported_objects, source_pcm)?;
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
        raw_file
            .write_all(&interleaved_f32le(
                &factorization.carriers[..5],
                JOC_FRAME_SAMPLES,
            ))
            .map_err(|error| format!("failed to write carrier PCM frame {frame_index}: {error}"))?;
        append_reduction_metrics(&mut metrics, frame_index, &sources, &reduction);
        previous_reduction = Some(reduction);
    }
    drop(raw_file);
    encode_ffmpeg_carrier(&i0_raw_path, &i0_carrier_path, 5, "1024k")?;
    let i0_carrier_bytes = read_carrier_frames(&i0_carrier_path, 4096, frame_count)?;
    let mut output_bytes = Vec::with_capacity(frame_count * (4096 + 2560));
    previous_reduction = None;
    for frame_index in 0..frame_count {
        let i0_start = frame_index * 4096;
        let i0_frame = &i0_carrier_bytes[i0_start..i0_start + 4096];
        let source_pcm = read_pcm_frame(&input_path, &scanned, frame_index, &track_indices)?;
        let sources = reduction_sources(&supported_objects, source_pcm)?;
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
        let access_unit = package_source_derived_access_unit(
            profile,
            sequence,
            &factorization.scaled_matrix,
            &positions,
            i0_frame,
            &d0_carriers,
            0,
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
        "encoded profile={profile_name} frames={frame_count} samples={encoded_samples} authored_sources={} dynamic_objects={} direct_speakers={} coded_objects_last_frame={} output_bytes={output_bytes_len} target_bitrate=1664000 reduction=deterministic:farthest-point+weighted-aggregate metrics={}",
        supported_objects.len(),
        metadata.dynamic_object_count(),
        metadata.direct_speaker_count(),
        previous_reduction
            .as_ref()
            .map_or(0, |result| result.coded_objects.len()),
        metrics_path.display(),
    );
    Ok(())
}

fn reduction_sources(
    objects: &[&AdmObject],
    pcm: Vec<Vec<f64>>,
) -> Result<Vec<ReductionSource>, String> {
    if objects.len() != pcm.len() {
        return Err(format!(
            "ADM source and PCM counts differ: {} objects, {} tracks",
            objects.len(),
            pcm.len()
        ));
    }
    objects
        .iter()
        .zip(pcm)
        .map(|(object, signal)| {
            Ok(ReductionSource {
                source_index: object.ordinal,
                object_id: object.object_id.clone(),
                kind: object.kind,
                position: object.position,
                signal,
            })
        })
        .collect()
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
        let _ = writeln!(
            output,
            "source,{frame_index},{},{object_id},{kind},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{},{},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{}",
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
        );
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
