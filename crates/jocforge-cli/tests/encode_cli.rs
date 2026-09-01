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

    let _ = fs::remove_dir_all(root);
}
