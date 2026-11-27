// pattern: Imperative Shell
use std::{path::PathBuf, process::Command};

use serde_json::Value;

const CLI_SELECTOR_VECTORS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/JOCFORGE-FLAT7X-CLI-SELECTION-PARTIAL-V1/INPUT-ONLY-VECTORS.json"
);

fn invocation_paths(case_number: usize) -> (PathBuf, PathBuf) {
    let prefix = format!(
        "jocforge-flat7x-selector-{}-{case_number}",
        std::process::id()
    );
    let root = std::env::temp_dir();
    (
        root.join(format!("{prefix}-missing-input.wav")),
        root.join(format!("{prefix}-output.ec3")),
    )
}

#[test]
fn aggregate_rate_requires_flat7x_and_excludes_a_second_rate() {
    let (input, output) = invocation_paths(100);
    for (args, expected) in [
        (&["--total-rate", "1024"][..], "--joc-layout"),
        (
            &[
                "--joc-layout",
                "flat7x",
                "--total-rate",
                "1024",
                "--data-rate",
                "1024",
            ],
            "cannot be used with",
        ),
        (
            &["--joc-layout", "flat7x", "--total-rate", "1200"],
            "invalid value",
        ),
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_jocforge"))
            .arg("encode")
            .arg(&input)
            .arg(&output)
            .args(args)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(2));
        let message = String::from_utf8_lossy(&result.stderr);
        assert!(message.contains(expected), "{args:?}: {message}");
    }
}

#[test]
fn wash_cli_vectors_select_or_reject_layout_before_source_admission() {
    let vectors: Value = serde_json::from_slice(CLI_SELECTOR_VECTORS).unwrap();
    assert!(!vectors["contains_media"].as_bool().unwrap());
    assert!(!vectors["contains_pcm"].as_bool().unwrap());
    assert!(!vectors["contains_payload"].as_bool().unwrap());
    assert!(!vectors["contains_keys"].as_bool().unwrap());

    for (case_number, vector) in vectors["vectors"].as_array().unwrap().iter().enumerate() {
        let argv = vector["argv"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>();
        let (input, output) = invocation_paths(case_number);
        assert!(!input.exists());
        assert!(!output.exists());

        let result = Command::new(env!("CARGO_BIN_EXE_jocforge"))
            .arg("encode")
            .arg(&input)
            .arg(&output)
            .args(&argv)
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert_eq!(result.status.code(), Some(2), "args {argv:?}: {stderr}");
        assert!(!output.exists(), "args {argv:?} created an output");

        let missing_input = format!("could not resolve input file {}:", input.display());
        match (
            vector.get("expected_profile").and_then(Value::as_str),
            vector.get("expected_result").and_then(Value::as_str),
        ) {
            (Some("existing_selected_1024_5_1"), None) => {
                assert!(stderr.contains(&missing_input));
            }
            (Some("flat7x_1024_5_1_core"), None) => {
                // The fixture supplies no source file, so CLI path admission stops first.
                assert!(stderr.contains(&missing_input));
            }
            (None, Some("reject before encoder execution")) => {
                assert!(stderr.contains("possible values: flat7x"));
            }
            (None, Some("reject as incompatible with approved profile row")) => {
                // This frozen vector preceded the additional D0 rate presets.
                assert!(stderr.contains(&missing_input));
            }
            disposition => panic!("unexpected Wash CLI selector vector: {disposition:?}"),
        }
    }
}
