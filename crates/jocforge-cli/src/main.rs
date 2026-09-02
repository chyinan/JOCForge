// pattern: Imperative Shell

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

mod file_scan;

use file_scan::scan_bw64_file;

use jocforge_core::{
    JocProfile, factorize_rank_controlled, generate_synthetic_fixture,
    package_source_derived_access_unit, parse_bw64_adm, parse_bw64_adm_metadata, profile_spec,
    validate_source_derived_object_limit, write_one_au_proof,
};

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
    let [flag, seconds, output] = arguments else {
        return Err("usage: jocforge fixture --seconds N OUTPUT.bw64".to_owned());
    };
    if flag != "--seconds" {
        return Err("usage: jocforge fixture --seconds N OUTPUT.bw64".to_owned());
    }
    let seconds = seconds
        .parse::<u32>()
        .map_err(|error| format!("invalid fixture duration '{seconds}': {error}"))?;
    let fixture = generate_synthetic_fixture(seconds)?;
    let output = PathBuf::from(output);
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
    validate_source_derived_object_limit(metadata.dynamic_object_count())
        .map_err(|error| error.to_string())?;
    let input_bytes = fs::read(&input_path).map_err(|error| {
        format!(
            "failed to read ADM input '{}': {error}",
            input_path.display()
        )
    })?;
    let parsed = parse_bw64_adm(&input_bytes).map_err(|error| {
        format!(
            "failed to parse ADM input '{}': {error}",
            input_path.display()
        )
    })?;
    let dynamic_objects = parsed
        .objects
        .iter()
        .filter(|object| object.kind.is_dynamic())
        .collect::<Vec<_>>();
    let object_pcm = dynamic_objects
        .iter()
        .map(|object| {
            parsed
                .pcm
                .get(object.track_index.saturating_sub(1))
                .cloned()
                .ok_or_else(|| format!("object {} maps outside PCM tracks", object.object_id))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let rank = profile_spec(profile).input_labels.len();
    let positions = dynamic_objects
        .iter()
        .map(|object| object.position)
        .collect::<Vec<_>>();
    let factorization = factorize_rank_controlled(&object_pcm, rank)
        .map_err(|error| format!("failed to factorize ADM objects: {error}"))?;
    let complete_samples = parsed.sample_count / 1536 * 1536;
    if complete_samples == 0 || complete_samples != parsed.sample_count {
        return Err(format!(
            "ADM duration must contain complete 1536-sample JOC units; got {} samples",
            parsed.sample_count
        ));
    }
    let work_root = env::temp_dir().join(format!("jocforge-encode-{}", std::process::id()));
    fs::create_dir_all(&work_root)
        .map_err(|error| format!("failed to create encoder work directory: {error}"))?;
    let i0_raw_path = work_root.join("carrier-i0.f32le");
    let i0_carrier_path = work_root.join("carrier-i0.ec3");
    let i0_bytes = interleaved_f32le(&factorization.carriers[..5], complete_samples);
    let d0_carriers = if rank == 7 {
        factorization.carriers[5..7].to_vec()
    } else {
        factorization.carriers[..2].to_vec()
    };
    let expected_frames = complete_samples / 1536;
    encode_ffmpeg_carrier(&i0_raw_path, &i0_carrier_path, i0_bytes, 5, "1024k")?;
    let i0_carrier_bytes = read_carrier_frames(&i0_carrier_path, 4096, expected_frames)?;
    let mut output_bytes = Vec::with_capacity(expected_frames * 8192);
    for frame_index in 0..expected_frames {
        let i0_start = frame_index * 4096;
        let i0_frame = &i0_carrier_bytes[i0_start..i0_start + 4096];
        let sequence = u16::try_from(frame_index % 1023 + 1).unwrap_or(1);
        let access_unit = package_source_derived_access_unit(
            profile,
            sequence,
            &factorization.scaled_matrix,
            &positions,
            i0_frame,
            &d0_carriers,
            frame_index * 1536,
        )?;
        output_bytes.extend_from_slice(&access_unit);
    }
    let output_path = PathBuf::from(output);
    write_output(&output_path, output_bytes, "encoded")?;
    let _ = fs::remove_dir_all(work_root);
    println!(
        "encoded profile={profile_name} frames={expected_frames} sample_rate=48000 carriers=source-derived:ffmpeg:i0=5x1024k,jocforge:d0=2xpcm-derived"
    );
    Ok(())
}

fn encode_ffmpeg_carrier(
    raw_path: &Path,
    carrier_path: &Path,
    raw: Vec<u8>,
    channels: usize,
    bitrate: &str,
) -> Result<(), String> {
    fs::write(raw_path, raw).map_err(|error| format!("failed to write carrier PCM: {error}"))?;
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
