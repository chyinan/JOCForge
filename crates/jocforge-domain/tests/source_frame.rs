use jocforge_domain::analysis::input_history::{
    INPUT_HISTORY_BLOCK_SAMPLES, INPUT_HISTORY_SAMPLES, InputHistory,
};
use jocforge_domain::analysis::source_frame::{
    F07SourceFrameBeforeState, F07SourceFrameError, InstalledAudioSourceFrame,
    stage_f07a_source_frame,
};
use jocforge_domain::analysis::transient_channel::{
    TransientChannelError, TransientChannelState, process_transient_channel_block,
};
use jocforge_domain::analysis::transient_parameters::load_transient_parameters;
use serde_json::Value;
use sha2::{Digest, Sha256};

const SOURCE_INPUTS_SHA256: &str =
    "59f7e83438bd1e7bd955cb4a9fe9c6dd7717939a651eb150d190edb2d91432d5";
const D05_PARAMETER_SHA256: &str =
    "130cd1a446ea8413a1d0d4c5eea631e890b7f1cd675975efa92697e258402991";
const SOURCE_INPUTS: &str =
    include_str!("fixtures/source_frame/F07-F12-SOURCE-COMPOSITION-INPUT-VECTORS.json");
const D05_PARAMETERS: &[u8] = include_bytes!("fixtures/transient_channel/PARAMETERS.json");
const FBW_EXTERNAL_ROWS: [usize; 5] = [0, 2, 1, 4, 5];

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn installed_audio(recipe: &Value) -> Box<InstalledAudioSourceFrame> {
    let audio = &recipe["audio_recipe"];
    assert_eq!(audio["schema"], "installed-audio-modular-v1");
    assert_eq!(
        audio["channel_order"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>(),
        ["L", "R", "C", "LFE", "Ls", "Rs"]
    );
    assert_eq!(audio["channels"], 6);
    assert_eq!(audio["samples_per_channel"], 1536);
    assert_eq!(
        audio["sample_formula"]["numerator"],
        "((sample*37 + channel*53 + source*71 + 17) mod 257)-128"
    );
    assert_eq!(audio["sample_formula"]["power_of_two_denominator"], 4096);

    let source = recipe["source"].as_u64().unwrap();
    let mut frame: Box<InstalledAudioSourceFrame> = vec![[[0_u32; 256]; 6]; 6]
        .into_boxed_slice()
        .try_into()
        .unwrap();
    for (channel, channel_blocks) in frame.iter_mut().enumerate() {
        for (block, block_words) in channel_blocks.iter_mut().enumerate() {
            for (sample, word) in block_words.iter_mut().enumerate() {
                let sample_index =
                    u64::try_from(block * INPUT_HISTORY_BLOCK_SAMPLES + sample).unwrap();
                let channel_index = u64::try_from(channel).unwrap();
                let residue = (sample_index * 37 + channel_index * 53 + source * 71 + 17) % 257;
                let numerator = i16::try_from(residue).unwrap() - 128;
                *word = (f32::from(numerator) / 4096.0).to_bits();
            }
        }
    }
    frame
}

fn cold_before_state() -> F07SourceFrameBeforeState {
    F07SourceFrameBeforeState::new(
        std::array::from_fn(|_| InputHistory::cold()),
        std::array::from_fn(|_| TransientChannelState::cold()),
        [false; 5],
    )
}

fn before_snapshots(
    before: &F07SourceFrameBeforeState,
) -> ([[u32; INPUT_HISTORY_SAMPLES]; 6], [[u8; 124]; 5], [bool; 5]) {
    (
        std::array::from_fn(|lane| *before.input_history_by_external_lane()[lane].words()),
        std::array::from_fn(|lane| before.transient_state_by_fbw_lane()[lane].canonical_bytes()),
        *before.previous_short_by_fbw_lane(),
    )
}

#[test]
fn f07a_source_frame_runs_five_fbw_lanes_and_six_delayed_rows_without_commit() {
    let input_bytes = SOURCE_INPUTS.as_bytes();
    assert_eq!(sha256(input_bytes), SOURCE_INPUTS_SHA256);
    assert_eq!(sha256(D05_PARAMETERS), D05_PARAMETER_SHA256);
    let vectors: Value = serde_json::from_slice(input_bytes).unwrap();
    assert_eq!(vectors["contains_pcm"], false);
    let recipe = &vectors["recipes"][0];
    assert_eq!(recipe["id"], "S0-LONG-MIX");
    let installed = installed_audio(recipe);
    let parameters = load_transient_parameters(Some(D05_PARAMETERS)).unwrap();
    let before = cold_before_state();
    let (history_before, transient_before, previous_short_before) = before_snapshots(&before);

    let proposal = stage_f07a_source_frame(&before, &installed, &parameters).unwrap();
    let output = proposal.output();

    assert_eq!(
        before_snapshots(&before),
        (history_before, transient_before, previous_short_before)
    );
    assert_eq!(output.delayed_audio()[0][0], [0_u32; 256]);
    for (external_lane, installed_blocks) in installed.iter().enumerate() {
        for delayed_block in &output.delayed_audio()[external_lane][..4] {
            assert_eq!(delayed_block, &[0_u32; 256]);
        }
        assert_eq!(
            output.delayed_audio()[external_lane][4],
            installed_blocks[0]
        );
        assert_eq!(
            output.delayed_audio()[external_lane][5],
            installed_blocks[1]
        );

        let mut expected_history = [0_u32; INPUT_HISTORY_SAMPLES];
        for (block, installed_block) in installed_blocks.iter().enumerate().skip(1) {
            let start = (block - 1) * INPUT_HISTORY_BLOCK_SAMPLES;
            expected_history[start..start + INPUT_HISTORY_BLOCK_SAMPLES]
                .copy_from_slice(installed_block);
        }
        assert_eq!(
            proposal.after_state().input_history_by_external_lane()[external_lane].words(),
            &expected_history
        );
    }

    for (fbw_lane, external_lane) in FBW_EXTERNAL_ROWS.into_iter().enumerate() {
        let mut direct_state = TransientChannelState::cold();
        let mut previous_short = false;
        for (block, installed_block) in installed[external_lane].iter().enumerate() {
            let direct =
                process_transient_channel_block(direct_state, installed_block, &parameters)
                    .unwrap();
            assert_eq!(
                output.short_source()[block][fbw_lane],
                direct.short_source()
            );
            assert_eq!(
                output.other_source()[block][fbw_lane],
                direct.other_source()
            );
            assert_eq!(
                output.short_consumed()[block][fbw_lane],
                direct.short_consumed()
            );
            assert_eq!(
                output.other_consumed()[block][fbw_lane],
                direct.other_consumed()
            );

            let short_now = direct.short_consumed() == 1;
            let other_now = direct.other_consumed() == 1;
            let expected_refresh =
                block == 0 || other_now || (block > 0 && short_now != previous_short);
            assert_eq!(
                output.structure_refresh()[block][fbw_lane],
                expected_refresh
            );
            assert_eq!(
                output.dither_flags()[block][fbw_lane],
                !short_now && !previous_short
            );
            previous_short = short_now;
            direct_state = direct.into_next_state();
        }
        assert_eq!(
            proposal.after_state().transient_state_by_fbw_lane()[fbw_lane].canonical_bytes(),
            direct_state.canonical_bytes()
        );
        assert_eq!(
            proposal.after_state().previous_short_by_fbw_lane()[fbw_lane],
            previous_short
        );
    }
    assert_eq!(
        output.dither_syntax_present(),
        output.dither_flags().iter().flatten().any(|flag| !flag)
    );
}

#[test]
fn f07a_source_frame_failure_does_not_change_caller_before_state() {
    let vectors: Value = serde_json::from_str(SOURCE_INPUTS).unwrap();
    let mut installed = installed_audio(&vectors["recipes"][0]);
    installed[FBW_EXTERNAL_ROWS[0]][0][17] = f32::NAN.to_bits();
    let parameters = load_transient_parameters(Some(D05_PARAMETERS)).unwrap();
    let before = cold_before_state();
    let snapshot = before_snapshots(&before);

    assert_eq!(
        stage_f07a_source_frame(&before, &installed, &parameters),
        Err(F07SourceFrameError::Transient(
            TransientChannelError::InputError
        ))
    );
    assert_eq!(before_snapshots(&before), snapshot);
}
