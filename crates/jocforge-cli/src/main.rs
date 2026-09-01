// pattern: Imperative Shell

use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::ExitCode,
};

use jocforge_core::{
    JocProfile, factorize_rank_controlled, generate_synthetic_fixture, package_ffmpeg_access_unit,
    parse_bw64_adm, profile_spec, write_one_au_proof,
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
    let bytes = read_adm_bytes(&input)?;
    let parsed =
        parse_bw64_adm(&bytes).map_err(|error| format!("failed to inspect ADM: {error}"))?;
    println!(
        "sample_rate={} channels={} samples={} objects={}",
        parsed.sample_rate,
        parsed.channel_count,
        parsed.sample_count,
        parsed.objects.len()
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
    let object_pcm = parsed
        .objects
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
    let positions = parsed
        .objects
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
    let raw_path = work_root.join("carrier.f32le");
    let carrier_path = work_root.join("carrier.ec3");
    let raw = interleaved_f32le(&factorization.carriers[..rank.min(5)], complete_samples);
    fs::write(&raw_path, raw).map_err(|error| format!("failed to write carrier PCM: {error}"))?;
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
            "5",
            "-i",
        ])
        .arg(&raw_path)
        .args(["-c:a", "eac3", "-b:a", "1024k", "-f", "eac3"])
        .arg(&carrier_path)
        .status()
        .map_err(|error| format!("failed to start FFmpeg E-AC-3 encoder: {error}"))?;
    if !ffmpeg.success() {
        let _ = fs::remove_dir_all(&work_root);
        return Err("FFmpeg E-AC-3 carrier encoding failed".to_owned());
    }
    let carrier_bytes = fs::read(&carrier_path)
        .map_err(|error| format!("failed to read FFmpeg carrier output: {error}"))?;
    if carrier_bytes.len() % 4096 != 0 {
        let _ = fs::remove_dir_all(&work_root);
        return Err("FFmpeg carrier output is not a sequence of 4096-byte frames".to_owned());
    }
    let expected_frames = complete_samples / 1536;
    if carrier_bytes.len() / 4096 != expected_frames {
        let _ = fs::remove_dir_all(&work_root);
        return Err(format!(
            "FFmpeg carrier frame count differs: expected {expected_frames}, got {}",
            carrier_bytes.len() / 4096
        ));
    }
    let mut output_bytes = Vec::with_capacity(expected_frames * 8192);
    for frame_index in 0..expected_frames {
        let start = frame_index * 4096;
        let frame = &carrier_bytes[start..start + 4096];
        let sequence = u16::try_from(frame_index % 1023 + 1).unwrap_or(1);
        let access_unit = package_ffmpeg_access_unit(
            profile,
            sequence,
            &factorization.scaled_matrix,
            &positions,
            frame,
        )?;
        output_bytes.extend_from_slice(&access_unit);
    }
    let output_path = PathBuf::from(output);
    write_output(&output_path, output_bytes, "encoded")?;
    let _ = fs::remove_dir_all(work_root);
    println!(
        "encoded profile={profile_name} frames={expected_frames} sample_rate=48000 carrier=ffmpeg:eac3:1024k"
    );
    Ok(())
}

fn read_adm_bytes(path: &PathBuf) -> Result<Vec<u8>, String> {
    let length = fs::metadata(path)
        .map_err(|error| format!("failed to stat ADM input '{}': {error}", path.display()))?
        .len();
    if length > 64 * 1024 * 1024 {
        let mut file = fs::File::open(path)
            .map_err(|error| format!("failed to open ADM input '{}': {error}", path.display()))?;
        let mut prefix = Vec::with_capacity(1024 * 1024);
        file.by_ref()
            .take(1024 * 1024)
            .read_to_end(&mut prefix)
            .map_err(|error| format!("failed to read ADM header '{}': {error}", path.display()))?;
        return Ok(prefix);
    }
    fs::read(path)
        .map_err(|error| format!("failed to read ADM input '{}': {error}", path.display()))
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
