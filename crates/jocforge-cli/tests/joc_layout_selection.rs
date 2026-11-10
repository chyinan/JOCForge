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

        match (
            vector.get("expected_profile").and_then(Value::as_str),
            vector.get("expected_result").and_then(Value::as_str),
        ) {
            (Some("existing_selected_1024_5_1" | "flat7x_1024_5_1_core"), None) => {
                assert!(stderr.contains("encode requires absolute input and output file paths"));
            }
            (None, Some("reject before encoder execution")) => {
                assert!(stderr.contains("joc-layout must be flat7x"));
            }
            (None, Some("reject as incompatible with approved profile row")) => {
                assert!(
                    stderr
                        .contains("joc-layout flat7x requires the selected 1024 kbps profile row")
                );
            }
            disposition => panic!("unexpected Wash CLI selector vector: {disposition:?}"),
        }
    }
}
