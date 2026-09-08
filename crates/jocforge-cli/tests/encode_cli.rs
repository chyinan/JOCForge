// pattern: Imperative Shell

use std::{env, fs, process::Command};

#[test]
fn encode_cli_uses_ffmpeg_for_the_carrier_and_writes_joc_output() {
    let root = env::temp_dir().join(format!("jocforge-encode-{}", std::process::id()));
    fs::create_dir_all(&root).expect("temporary directory");
    let input = root.join("fixture.bw64");
    let output = root.join("idx1.ec3");
    let fixture = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "fixture",
            "--seconds",
            "4",
            input.to_str().expect("input path is UTF-8"),
        ])
        .status()
        .expect("fixture CLI starts");
    assert!(fixture.success());

    let encoded = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "encode",
            input.to_str().expect("input path is UTF-8"),
            output.to_str().expect("output path is UTF-8"),
            "--profile",
            "idx1",
        ])
        .status()
        .expect("encode CLI starts");
    assert!(encoded.success());
    assert_eq!(
        fs::metadata(&output).expect("encoded output").len() % 6656,
        0
    );
    let metrics =
        fs::read_to_string(format!("{}.metrics.csv", output.display())).expect("reduction metrics");
    let header = metrics.lines().next().expect("metrics header");
    for field in [
        "unexplained_cluster_leakage",
        "speaker_render_error",
        "factorization_reconstruction_error",
        "matrix_coefficient_jump",
        "carrier_coefficient_jump",
    ] {
        assert!(
            header.split(',').any(|column| column == field),
            "missing {field}"
        );
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn encode_cli_writes_a_real_grouped_short_partition_and_exact_sample_accounting() {
    let root = env::temp_dir().join(format!("jocforge-short-encode-{}", std::process::id()));
    fs::create_dir_all(&root).expect("temporary directory");
    let input = root.join("fixture.bw64");
    let output = root.join("idx1-3-3.ec3");
    let fixture = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "fixture",
            "--seconds",
            "1",
            "--objects",
            "1",
            input.to_str().expect("input path is UTF-8"),
        ])
        .status()
        .expect("fixture CLI starts");
    assert!(fixture.success());

    let encoded = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "encode",
            input.to_str().expect("input path is UTF-8"),
            output.to_str().expect("output path is UTF-8"),
            "--profile",
            "idx1",
            "--block-partition",
            "3+3",
            "--bitrate",
            "1664",
        ])
        .status()
        .expect("short encode CLI starts");
    assert!(encoded.success());
    assert_eq!(
        fs::metadata(&output).expect("short encoded output").len(),
        32 * 6656
    );

    let accounting = fs::read_to_string(format!("{}.sample-accounting.csv", output.display()))
        .expect("sample accounting");
    let rows = accounting.lines().skip(1).collect::<Vec<_>>();
    assert_eq!(rows.len(), 64);
    let mut source_samples = 0_usize;
    let mut padding_samples = 0_usize;
    let mut previous_end = 0_usize;
    for row in rows {
        let fields = row.split(',').collect::<Vec<_>>();
        assert_eq!(fields.len(), 7);
        let start = fields[3].parse::<usize>().expect("start sample");
        let end = fields[4].parse::<usize>().expect("end sample");
        let source = fields[5].parse::<usize>().expect("source sample count");
        let padding = fields[6].parse::<usize>().expect("padding sample count");
        assert_eq!(start, previous_end);
        assert_eq!(source + padding, end - start);
        previous_end = end;
        source_samples += source;
        padding_samples += padding;
    }
    assert_eq!(previous_end, 32 * 1536);
    assert_eq!(source_samples, 48_000);
    assert_eq!(padding_samples, 1_152);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn encode_cli_removes_stale_output_when_partition_validation_fails() {
    let root = env::temp_dir().join(format!("jocforge-short-invalid-{}", std::process::id()));
    fs::create_dir_all(&root).expect("temporary directory");
    let output = root.join("stale.ec3");
    fs::write(&output, b"stale").expect("stale output");

    let failed = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "encode",
            root.join("missing.bw64")
                .to_str()
                .expect("input path is UTF-8"),
            output.to_str().expect("output path is UTF-8"),
            "--profile",
            "idx1",
            "--block-partition",
            "4+2",
        ])
        .status()
        .expect("invalid short encode CLI starts");
    assert!(!failed.success());
    assert!(!output.exists());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn encode_cli_accepts_the_validated_legacy_ac3_flat7x_carriage() {
    let root = env::temp_dir().join(format!("jocforge-legacy-core-{}", std::process::id()));
    fs::create_dir_all(&root).expect("temporary directory");
    let input = root.join("fixture.bw64");
    let output = root.join("legacy.ec3");
    let fixture = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "fixture",
            "--seconds",
            "1",
            input.to_str().expect("input path is UTF-8"),
        ])
        .status()
        .expect("fixture CLI starts");
    assert!(fixture.success());

    let encoded = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "encode",
            input.to_str().expect("input path is UTF-8"),
            output.to_str().expect("output path is UTF-8"),
            "--profile",
            "idx1",
            "--carriage",
            "legacy-ac3",
            "--bitrate",
            "1280",
        ])
        .status()
        .expect("legacy encode CLI starts");
    assert!(encoded.success());
    assert_eq!(
        fs::metadata(&output).expect("legacy output").len() % 5120,
        0
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn encode_cli_accepts_a_clean_fixture_above_the_old_object_limit() {
    let root = env::temp_dir().join(format!("jocforge-reduction-{}", std::process::id()));
    fs::create_dir_all(&root).expect("temporary directory");
    let input = root.join("fixture-20.bw64");
    let output = root.join("idx1.ec3");
    let fixture = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "fixture",
            "--seconds",
            "1",
            "--objects",
            "20",
            input.to_str().expect("input path is UTF-8"),
        ])
        .status()
        .expect("fixture CLI starts");
    assert!(fixture.success());

    let encoded = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "encode",
            input.to_str().expect("input path is UTF-8"),
            output.to_str().expect("output path is UTF-8"),
            "--profile",
            "idx1",
        ])
        .status()
        .expect("encode CLI starts");
    assert!(encoded.success());
    assert_eq!(
        fs::metadata(&output).expect("encoded output").len() % 6656,
        0
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn encode_cli_retains_an_lfe_as_a_separate_core_channel() {
    let root = env::temp_dir().join(format!("jocforge-directspeakers-{}", std::process::id()));
    fs::create_dir_all(&root).expect("temporary directory");
    let input = root.join("directspeakers.bw64");
    let output = root.join("directspeakers.ec3");
    let fixture = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "fixture-directspeakers",
            "--seconds",
            "1",
            input.to_str().expect("input path is UTF-8"),
        ])
        .status()
        .expect("DirectSpeakers fixture CLI starts");
    assert!(fixture.success());

    let encoded = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "encode",
            input.to_str().expect("input path is UTF-8"),
            output.to_str().expect("output path is UTF-8"),
            "--profile",
            "idx1",
        ])
        .status()
        .expect("DirectSpeakers encode CLI starts");
    assert!(encoded.success());

    let probe = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=channels",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            output.to_str().expect("output path is UTF-8"),
        ])
        .output()
        .expect("ffprobe starts");
    assert!(probe.status.success());
    assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(), "8");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn encode_cli_accepts_lfe_in_a_real_short_partition() {
    let root = env::temp_dir().join(format!(
        "jocforge-directspeakers-short-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temporary directory");
    let input = root.join("directspeakers.bw64");
    let output = root.join("directspeakers-short.ec3");
    let fixture = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "fixture-directspeakers",
            "--seconds",
            "1",
            input.to_str().expect("input path is UTF-8"),
        ])
        .status()
        .expect("DirectSpeakers fixture CLI starts");
    assert!(fixture.success());

    let encoded = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "encode",
            input.to_str().expect("input path is UTF-8"),
            output.to_str().expect("output path is UTF-8"),
            "--profile",
            "idx2",
            "--bitrate",
            "1280",
            "--block-partition",
            "2+2+2",
        ])
        .status()
        .expect("short LFE encode CLI starts");
    assert!(encoded.success());
    assert_eq!(
        fs::metadata(&output).expect("short LFE output").len(),
        32 * 5120
    );

    let accounting = fs::read_to_string(format!("{}.sample-accounting.csv", output.display()))
        .expect("short LFE sample accounting");
    assert_eq!(accounting.lines().count(), 1 + 32 * 3);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn encode_cli_reports_a_bounded_lfe_peak_and_headroom_margin() {
    let root = env::temp_dir().join(format!("jocforge-lfe-headroom-{}", std::process::id()));
    fs::create_dir_all(&root).expect("temporary directory");
    let input = root.join("directspeakers.bw64");
    let output = root.join("directspeakers.ec3");
    let fixture = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "fixture-directspeakers",
            "--seconds",
            "1",
            input.to_str().expect("input path is UTF-8"),
        ])
        .status()
        .expect("DirectSpeakers fixture CLI starts");
    assert!(fixture.success());

    let encoded = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "encode",
            input.to_str().expect("input path is UTF-8"),
            output.to_str().expect("output path is UTF-8"),
            "--profile",
            "idx1",
        ])
        .status()
        .expect("DirectSpeakers encode CLI starts");
    assert!(encoded.success());

    let metrics = fs::read_to_string(format!("{}.metrics.csv", output.display()))
        .expect("LFE metrics sidecar");
    let mut rows = metrics.lines();
    let header = rows
        .next()
        .expect("metrics header")
        .split(',')
        .collect::<Vec<_>>();
    let peak_index = header
        .iter()
        .position(|field| *field == "max_coded_peak")
        .expect("LFE peak column");
    let margin_index = header
        .iter()
        .position(|field| *field == "headroom_margin")
        .expect("LFE headroom column");
    let lfe_rows = rows
        .filter(|row| row.starts_with("lfe,"))
        .collect::<Vec<_>>();
    assert!(!lfe_rows.is_empty());
    for row in lfe_rows {
        let fields = row.split(',').collect::<Vec<_>>();
        let peak = fields[peak_index].parse::<f64>().expect("finite LFE peak");
        let margin = fields[margin_index]
            .parse::<f64>()
            .expect("finite LFE headroom");
        assert!(peak <= 0.90 + 1.0e-12);
        assert!(margin > 0.0);
    }

    let _ = fs::remove_dir_all(root);
}
