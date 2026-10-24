// pattern: Imperative Shell
use jocforge_io::pcm::reader_input_padding::{
    SELECTED_READER_ACTUAL_SAMPLES, SELECTED_READER_CHANNELS, SELECTED_READER_FRAME_SAMPLES,
    SELECTED_READER_TAIL_SAMPLES, SELECTED_READER_WORD_BYTES, SelectedReaderPaddingError,
    SelectedReaderReadPrefix, prepare_selected_reader_input_frame,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/partial_frame_padding";
const HANDOFF_MANIFEST_SHA256: &str =
    "7e839dd2c4ff414651754056c40075360b1cb896422b926a64f4619b8af314f2";
const HANDOFF_CHECK_SHA256: &str =
    "d15b4f168cbb16d4041a8190a3898cf287f003aff2bcaf4bfc5c093c1313c768";
const SOURCE_MANIFEST_SHA256: &str =
    "47dd26dc64d91de60a8735ee8cc9885da386a58b0c56a9cabc23d97e1303f0d9";
const BASE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const READER_HANDOFF_MANIFEST_SHA256: &str =
    "7f6adb938741b61ac15c7aeca1dac759504e291af302ab3d460bd41fc64aa7b6";
const LIFECYCLE_HANDOFF_MANIFEST_SHA256: &str =
    "ef7fa983440fd11784fe9f24f64284240f00fd053853dc1cc28067d3b1cf0d42";

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURE_DIRECTORY)
        .join(name)
}

fn fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(fixture_path(name)).unwrap()
}

fn fixture_json(name: &str) -> Value {
    serde_json::from_slice(&fixture_bytes(name)).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn assert_handoff_payload(manifest: &Value, name: &str) {
    let bytes = fixture_bytes(name);
    let entry = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == name)
        .unwrap();
    assert_eq!(entry["bytes"].as_u64().unwrap(), bytes.len() as u64);
    assert_eq!(entry["sha256"], sha256(&bytes), "{name}");
}

#[test]
fn selected_padding_handoff_and_aggregate_observations_are_hash_bound() {
    let handoff_manifest_bytes = fixture_bytes("HANDOFF-MANIFEST.json");
    let handoff_check_bytes = fixture_bytes("HANDOFF-CHECK.json");
    let handoff_manifest: Value = serde_json::from_slice(&handoff_manifest_bytes).unwrap();
    let handoff_check: Value = serde_json::from_slice(&handoff_check_bytes).unwrap();
    assert_eq!(sha256(&handoff_manifest_bytes), HANDOFF_MANIFEST_SHA256);
    assert_eq!(sha256(&handoff_check_bytes), HANDOFF_CHECK_SHA256);
    assert_eq!(handoff_manifest["files"].as_array().unwrap().len(), 16);
    assert_eq!(
        handoff_manifest["source_package_manifest_sha256"],
        SOURCE_MANIFEST_SHA256
    );
    assert_eq!(
        handoff_manifest["base_package_manifest_sha256"],
        BASE_MANIFEST_SHA256
    );
    assert_eq!(
        handoff_check["handoff_manifest_sha256"],
        HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(handoff_check["payload_hashes_match"], true);
    assert_eq!(handoff_check["whole_F13C_ready"], false);
    assert_eq!(handoff_check["implementation_ready"], false);

    for name in [
        "INPUTS.json",
        "EXPECTED.json",
        "NATIVE-STATISTICS.json",
        "CONTRACT.json",
        "PADDING-ADDENDUM.md",
        "CURRENT-OVERLAY.json",
        "INDEPENDENT-CHECK.json",
        "WASH-INDEPENDENT-CHECK.json",
    ] {
        assert_handoff_payload(&handoff_manifest, name);
    }

    let inputs = fixture_json("INPUTS.json");
    assert_eq!(inputs["input_only"], true);
    assert_eq!(inputs["channels"], 38);
    assert_eq!(inputs["frame_samples"], 1_536);
    assert_eq!(inputs["actual_samples"], 1_392);
    assert_eq!(inputs["valid_prefix_is_opaque"], true);

    let expected = fixture_json("EXPECTED.json");
    assert_eq!(expected["tail_samples"], 144);
    assert_eq!(expected["tail_words"], 5_472);
    assert_eq!(expected["tail_bytes"], 21_888);
    assert_eq!(expected["tail_counts"]["positive_zero"], 5_472);
    assert_eq!(expected["tail_counts"]["negative_zero"], 0);
    assert_eq!(expected["tail_counts"]["finite_nonzero"], 0);
    assert_eq!(expected["tail_counts"]["subnormal"], 0);
    assert_eq!(expected["tail_counts"]["infinity"], 0);
    assert_eq!(expected["tail_counts"]["nan"], 0);

    let native = fixture_json("NATIVE-STATISTICS.json");
    assert_eq!(native["zero_rule_matching_observations"], 11);
    assert_eq!(native["PCM_sample_values_exported"], false);
    assert_eq!(native["PCM_content_digests_exported"], false);
    let before_clear = native["observations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["event"] == "tail_statistics" && row["stage"] == "before_clear")
        .unwrap();
    assert_eq!(before_clear["counts"]["positive_zero"], 1_747);
    assert_eq!(before_clear["counts"]["finite_nonzero"], 3_725);

    let overlay = fixture_json("CURRENT-OVERLAY.json");
    assert_eq!(
        overlay["reader_padding_rule"],
        "clear entire selected input row before read; 1392 actual samples plus 144 positive-zero samples per channel in 1536-sample packet"
    );
    assert_eq!(
        overlay["DSP_output_tail"],
        "not implied; observed DSP output tail is generally nonzero"
    );
    assert_eq!(overlay["whole_F13C_ready"], "NO");
    let wash = fixture_json("WASH-INDEPENDENT-CHECK.json");
    assert_eq!(
        wash["related_reader_handoff_manifest_sha256"],
        READER_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(
        wash["related_eof_handoff_manifest_sha256"],
        LIFECYCLE_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(wash["DSP_tail_model_comparison"], false);
    assert_eq!(wash["whole_F13C_ready"], false);
}

#[test]
fn selected_reader_provider_copies_opaque_prefix_and_fills_only_input_tail() {
    // Synthetic words test bit-preserving prefix copy. They are not PCM values.
    let synthetic_prefix = (0..SELECTED_READER_CHANNELS)
        .map(|channel| {
            (0..SELECTED_READER_ACTUAL_SAMPLES)
                .map(|sample| {
                    (u32::try_from(channel + 1).unwrap() << 20) | u32::try_from(sample + 1).unwrap()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let frame =
        prepare_selected_reader_input_frame(SelectedReaderReadPrefix::new(&synthetic_prefix))
            .unwrap();
    assert_eq!(frame.channels(), 38);
    assert_eq!(frame.samples_per_channel(), 1_536);
    assert_eq!(
        frame.tail_samples_per_channel(),
        SELECTED_READER_TAIL_SAMPLES
    );
    assert_eq!(frame.rows().len(), SELECTED_READER_CHANNELS);

    let mut positive_zero_tail_words = 0_usize;
    for (input_row, output_row) in synthetic_prefix.iter().zip(frame.rows()) {
        assert_eq!(output_row.len(), SELECTED_READER_FRAME_SAMPLES);
        assert_eq!(
            &output_row[..SELECTED_READER_ACTUAL_SAMPLES],
            input_row.as_slice()
        );
        for word in &output_row[SELECTED_READER_ACTUAL_SAMPLES..] {
            assert_eq!(*word, 0_u32);
            positive_zero_tail_words += 1;
        }
    }
    assert_eq!(positive_zero_tail_words, 5_472);
    assert_eq!(
        positive_zero_tail_words * SELECTED_READER_WORD_BYTES,
        21_888
    );
}

#[test]
fn selected_reader_provider_rejects_wrong_shapes_without_touching_dsp_outputs() {
    let wrong_channels =
        vec![vec![0_u32; SELECTED_READER_ACTUAL_SAMPLES]; SELECTED_READER_CHANNELS - 1];
    assert_eq!(
        prepare_selected_reader_input_frame(SelectedReaderReadPrefix::new(&wrong_channels)),
        Err(SelectedReaderPaddingError::ChannelCountMismatch)
    );

    let wrong_rows =
        vec![vec![0_u32; SELECTED_READER_ACTUAL_SAMPLES - 1]; SELECTED_READER_CHANNELS];
    assert_eq!(
        prepare_selected_reader_input_frame(SelectedReaderReadPrefix::new(&wrong_rows)),
        Err(SelectedReaderPaddingError::ActualReadLengthMismatch)
    );

    let contract = fixture_json("CONTRACT.json");
    assert!(contract["limits"].as_array().unwrap().iter().any(|item| {
        item == "Full renderer/component tail model output is NOT_RUN; its nonzero tails are native diagnostics."
    }));
}
