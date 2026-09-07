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
fn advanced_multi_dependent_cli_writes_sequential_dependents() {
    let output_path = env::temp_dir().join(format!(
        "jocforge-multi-dependent-{}.ec3",
        std::process::id()
    ));
    let status = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "conformance-multi-dependent",
            "idx0",
            "2",
            output_path.to_str().expect("temporary path is UTF-8"),
        ])
        .status()
        .expect("multi-dependent CLI starts");

    assert!(!status.success());
    assert!(!output_path.exists());
}

#[test]
fn advanced_short_block_cli_writes_one_grouped_interval() {
    let output_path =
        env::temp_dir().join(format!("jocforge-short-block-{}.ec3", std::process::id()));
    let status = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "conformance-short",
            "idx0",
            "1+1+1+1+1+1",
            output_path.to_str().expect("temporary path is UTF-8"),
        ])
        .status()
        .expect("short-block CLI starts");

    assert!(status.success());
    assert_eq!(
        fs::metadata(&output_path)
            .expect("short-block output")
            .len(),
        6144
    );
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
    assert!(report.contains("\"profile_count\": 5"));
    assert!(report.contains("\"domain_values_checked\": 96"));
    let _ = fs::remove_file(output_path);
}

#[test]
fn carrier_causality_cli_reports_each_profile_source_set() {
    let output_path = env::temp_dir().join(format!(
        "jocforge-carrier-causality-{}.json",
        std::process::id()
    ));
    let status = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "carrier-causality",
            output_path.to_str().expect("temporary path is UTF-8"),
        ])
        .status()
        .expect("carrier causality CLI starts");

    assert!(status.success());
    let report = fs::read_to_string(&output_path).expect("carrier causality report exists");
    assert!(report.contains("\"pass\": true"));
    assert!(report.contains("\"profile\":\"idx0\""));
    assert!(report.contains("\"profile\":\"idx1\""));
    assert!(report.contains("\"profile\":\"idx2\""));
    assert!(report.contains("\"profile\":\"idx3\""));
    assert!(report.contains("\"profile\":\"idx4\""));
    let _ = fs::remove_file(output_path);
}

#[test]
fn spectral_phase_cli_rejects_incomplete_final_render_evidence() {
    let root = env::temp_dir().join(format!(
        "jocforge-spectral-final-render-reject-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temporary directory");
    let output_path = root.join("spectral.json");
    let evidence_path = root.join("forged-final-render.json");
    fs::write(
        &evidence_path,
        r#"{
  "stage": "FINAL_DECODED_RENDER",
  "decoder": "PINNED_OPENJOC",
  "final_render_pass": true,
  "profiles": [
    {"profile": "idx0"},
    {"profile": "idx1"},
    {"profile": "idx2"}
  ]
}"#,
    )
    .expect("forged evidence");
    let status = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args([
            "spectral-phase",
            output_path.to_str().expect("output path is UTF-8"),
            "--final-render-report",
            evidence_path.to_str().expect("evidence path is UTF-8"),
        ])
        .status()
        .expect("spectral phase CLI starts");

    assert!(!status.success());
    let _ = fs::remove_dir_all(root);
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
