// pattern: Imperative Shell

use std::process::Command;

#[test]
fn help_and_version_are_available() {
    for argument in ["--help", "--version"] {
        let output = Command::new(env!("CARGO_BIN_EXE_jocforge"))
            .arg(argument)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("jocforge")
        );
    }
}

#[test]
fn encode_cannot_modify_an_existing_destination() {
    let directory = std::env::temp_dir().join(format!("jocforge-migration-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let destination = directory.join("sentinel.ec3");
    std::fs::write(&destination, b"preserve existing output").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .args(["encode", "missing.bw64"])
        .arg(&destination)
        .args(["--architecture", "compat", "--carriage", "legacy-ac3"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("encoding backend unavailable")
    );
    assert_eq!(
        std::fs::read(&destination).unwrap(),
        b"preserve existing output"
    );
    let names: Vec<_> = std::fs::read_dir(&directory).unwrap().collect();
    assert_eq!(names.len(), 1);
    std::fs::remove_file(destination).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn retired_commands_are_rejected() {
    for command in [
        "proof",
        "clean-run",
        "conformance-short",
        "conformance-cmaf",
        "--architecture",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_jocforge"))
            .arg(command)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
    }
}
