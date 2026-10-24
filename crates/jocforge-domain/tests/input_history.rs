// pattern: Functional Core
use jocforge_domain::analysis::input_history::{
    INPUT_HISTORY_BLOCK_SAMPLES, INPUT_HISTORY_SAMPLES, InputHistory,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/input_history";
const SOURCE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const SOURCE_SPEC_SHA256: &str = "61a957410a05e1e44c15d7a3f6622d2ff4c2a1135020f35261c0dee1979f1eac";
const SOURCE_VECTORS_SHA256: &str =
    "a14dd6c3700fae1765399c72c816dd3146d57ab7dd026b375efb6aed7408cfa8";
const SOURCE_ADAPTER_CONTRACT_SHA256: &str =
    "9bce2e35ce47147c76c13e58ee976d44c5109938f0b19e1f3a81f1d8a61fa1ae";
const SOURCE_UNIT_README_SHA256: &str =
    "a455c6b33e582b0d359face09667e91622404fd203d376e446d615f9a52bbcce";
const INPUT_SHA256: &str = "489c73405f5fdc1e078901fc555650d4d46cc5c7beb9e9e2a7159f3eb83e0124";
const DELAYED_OUTPUT_SHA256: &str =
    "e81243dc815fa9f324bf10ccb37723251c171036f00b5e08c5469e1f4e447371";
const TERMINAL_STATE_SHA256: &str =
    "2d6fb9249269bca846da5d32f82cbec3ffaed04032886bcc489b437a07eac1eb";
const LANE_COUNT: usize = 6;
const FRAMES: usize = 2;
const SAMPLES_PER_FRAME: usize = 1_536;
const BLOCKS_PER_FRAME: usize = SAMPLES_PER_FRAME / INPUT_HISTORY_BLOCK_SAMPLES;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(crate_root().join(FIXTURE_DIRECTORY).join(name)).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn decode_words(bytes: &[u8]) -> Vec<u32> {
    assert_eq!(bytes.len() % 4, 0);
    bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect()
}

#[test]
fn f07_input_history_vectors_are_hash_bound_and_scoped() {
    let provenance_bytes = fixture_bytes("PROVENANCE.json");
    let provenance: Value = serde_json::from_slice(&provenance_bytes).unwrap();
    let input = fixture_bytes("input-f32le.bin");
    let delayed_output = fixture_bytes("delayed-output-f32le.bin");
    let terminal_state = fixture_bytes("terminal-state-f32le.bin");

    assert_eq!(
        provenance["slice"],
        "F07_1_SIX_LANE_INPUT_HISTORY_BYTE_COPY_ONLY"
    );
    assert_eq!(
        provenance["source_package_manifest_sha256"],
        SOURCE_MANIFEST_SHA256
    );
    assert_eq!(provenance["source_spec_sha256"], SOURCE_SPEC_SHA256);
    assert_eq!(
        provenance["source_unit_vectors_sha256"],
        SOURCE_VECTORS_SHA256
    );
    assert_eq!(
        provenance["source_adapter_contract_sha256"],
        SOURCE_ADAPTER_CONTRACT_SHA256
    );
    assert_eq!(
        provenance["source_unit_readme_sha256"],
        SOURCE_UNIT_README_SHA256
    );
    assert_eq!(provenance["case_id"], "U-HISTORY-01");
    assert_eq!(provenance["frames"], FRAMES);
    assert_eq!(provenance["lanes"], LANE_COUNT);
    assert_eq!(provenance["samples_per_frame_per_lane"], SAMPLES_PER_FRAME);
    assert_eq!(
        provenance["samples_per_history_per_lane"],
        INPUT_HISTORY_SAMPLES
    );
    assert_eq!(provenance["cold"], true);
    assert_eq!(
        provenance["frame_layout"],
        "frame-major, then six external lane slots L/R/C/LFE/Ls/Rs, then F32LE samples"
    );
    assert_eq!(sha256(&input), INPUT_SHA256);
    assert_eq!(sha256(&delayed_output), DELAYED_OUTPUT_SHA256);
    assert_eq!(sha256(&terminal_state), TERMINAL_STATE_SHA256);
}

#[test]
fn two_frames_match_delayed_output_and_terminal_state_for_six_lanes() {
    assert_eq!(BLOCKS_PER_FRAME, 6);
    let input_words = decode_words(&fixture_bytes("input-f32le.bin"));
    let frame_words = LANE_COUNT * SAMPLES_PER_FRAME;
    let expected_input_words = FRAMES * frame_words;
    assert_eq!(input_words.len(), expected_input_words);

    let mut actual_output = vec![0; expected_input_words * 4];
    let mut actual_terminal = vec![0; LANE_COUNT * INPUT_HISTORY_SAMPLES * 4];

    for lane in 0..LANE_COUNT {
        let mut history = InputHistory::cold();
        for frame in 0..FRAMES {
            let lane_start = frame * frame_words + lane * SAMPLES_PER_FRAME;
            let lane_frame = &input_words[lane_start..lane_start + SAMPLES_PER_FRAME];
            for block_index in 0..BLOCKS_PER_FRAME {
                let block_start = block_index * INPUT_HISTORY_BLOCK_SAMPLES;
                let current: &[u32; INPUT_HISTORY_BLOCK_SAMPLES] = lane_frame
                    [block_start..block_start + INPUT_HISTORY_BLOCK_SAMPLES]
                    .try_into()
                    .unwrap();
                let (delayed, next_history) = history.advance_block(current);
                let output_start = frame * frame_words + lane * SAMPLES_PER_FRAME + block_start;
                for (offset, word) in delayed.words().iter().enumerate() {
                    let byte_start = (output_start + offset) * 4;
                    actual_output[byte_start..byte_start + 4].copy_from_slice(&word.to_le_bytes());
                }
                history = next_history;
            }
        }
        let terminal_start = lane * INPUT_HISTORY_SAMPLES * 4;
        for (offset, word) in history.words().iter().enumerate() {
            let byte_start = terminal_start + offset * 4;
            actual_terminal[byte_start..byte_start + 4].copy_from_slice(&word.to_le_bytes());
        }
    }

    let expected_output = fixture_bytes("delayed-output-f32le.bin");
    let expected_terminal = fixture_bytes("terminal-state-f32le.bin");
    assert_eq!(actual_output, expected_output);
    assert_eq!(actual_terminal, expected_terminal);
    let output_words = decode_words(&actual_output);
    for lane in 0..LANE_COUNT {
        let lane_start = lane * SAMPLES_PER_FRAME;
        assert!(
            output_words[lane_start..lane_start + 1_024]
                .iter()
                .all(|word| *word == 0)
        );
    }
}

#[test]
fn one_block_transition_is_bitwise_and_consumes_the_prior_state() {
    let cold = InputHistory::cold();
    let current = [0x8000_0001; INPUT_HISTORY_BLOCK_SAMPLES];
    let (delayed, next) = cold.advance_block(&current);
    assert!(delayed.words().iter().all(|word| *word == 0));
    assert_eq!(
        &next.words()[..INPUT_HISTORY_SAMPLES - INPUT_HISTORY_BLOCK_SAMPLES],
        &[0; INPUT_HISTORY_SAMPLES - INPUT_HISTORY_BLOCK_SAMPLES]
    );
    assert_eq!(
        &next.words()[INPUT_HISTORY_SAMPLES - INPUT_HISTORY_BLOCK_SAMPLES..],
        &current
    );

    let mut state = next;
    let zero_block = [0; INPUT_HISTORY_BLOCK_SAMPLES];
    for _ in 0..3 {
        let (still_delayed, next_state) = state.advance_block(&zero_block);
        assert!(still_delayed.words().iter().all(|word| *word == 0));
        state = next_state;
    }
    let (released, _) = state.advance_block(&zero_block);
    assert_eq!(released.words(), &current);
}
