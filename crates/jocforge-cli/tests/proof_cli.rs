// pattern: Imperative Shell

use std::{env, fs, process::Command};

#[test]
fn proof_cli_writes_requested_idx0_access_unit() {
    let output_path = env::temp_dir().join(format!(
        "jocforge-proof-{}-{}.ec3",
        std::process::id(),
        "idx0"
    ));
    let status = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "proof",
            "idx0",
            output_path.to_str().expect("temporary path is UTF-8"),
        ])
        .status()
        .expect("proof CLI starts");

    assert!(status.success());
    let bytes = fs::read(&output_path).expect("proof output exists");
    assert_eq!(bytes.len(), 4096);
    assert_eq!(&bytes[0..2], &[0x0b, 0x77]);
    let _ = fs::remove_file(output_path);
}

#[test]
fn fixture_cli_writes_an_adm_bw64_file() {
    let output_path = env::temp_dir().join(format!("jocforge-fixture-{}.bw64", std::process::id()));
    let status = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "fixture",
            "--seconds",
            "1",
            output_path.to_str().expect("temporary path is UTF-8"),
        ])
        .status()
        .expect("fixture CLI starts");

    assert!(status.success());
    let bytes = fs::read(&output_path).expect("fixture exists");
    assert_eq!(&bytes[0..4], b"BW64");
    assert!(bytes.windows(4).any(|window| window == b"axml"));
    assert!(bytes.windows(4).any(|window| window == b"chna"));
    let _ = fs::remove_file(output_path);
}

#[test]
fn quantization_stress_cli_reports_all_profiles_and_full_domain() {
    let output_path = env::temp_dir().join(format!(
        "jocforge-quantization-stress-{}.json",
        std::process::id()
    ));
    let status = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "quantization-stress",
            output_path.to_str().expect("temporary path is UTF-8"),
        ])
        .status()
        .expect("quantization stress CLI starts");

    assert!(status.success());
    let report = fs::read_to_string(&output_path).expect("quantization report exists");
    assert!(report.contains("\"pass\": true"));
    assert!(report.contains("\"profile_count\": 3"));
    assert!(report.contains("\"domain_values_checked\": 96"));
    let _ = fs::remove_file(output_path);
}

#[test]
fn spectral_phase_cli_reports_the_controlled_corpus() {
    let output_path = env::temp_dir().join(format!(
        "jocforge-spectral-phase-{}.json",
        std::process::id()
    ));
    let status = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "spectral-phase",
            output_path.to_str().expect("temporary path is UTF-8"),
        ])
        .status()
        .expect("spectral phase CLI starts");

    assert!(status.success());
    let report = fs::read_to_string(&output_path).expect("spectral report exists");
    assert!(report.contains("\"pass\": true"));
    assert!(report.contains("\"fixture_count\": 10"));
    assert!(report.contains("magnitude_spectrum_deviation"));
    assert!(report.contains("phase_difference"));
    let _ = fs::remove_file(output_path);
}
