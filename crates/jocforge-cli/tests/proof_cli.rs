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
