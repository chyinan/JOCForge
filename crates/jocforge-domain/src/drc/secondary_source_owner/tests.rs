use super::*;
use crate::{
    analysis::{
        input_history::InputHistory,
        source_frame::{
            AnalysisOwnerGeneration, F07SourceFrameBeforeState, InstalledAudioSourceFrame,
            SharedAnalysisReceipt, SourceStageKey, stage_f07a_source_frame,
        },
        transient_channel::TransientChannelState,
        transient_parameters::load_transient_parameters,
    },
    meter::measurement_receipt::{ProgrammeMeasurementReceipt, ProgrammeMeasurementReceiptInput},
    timeline::FrontendCallbackIndex,
    transaction::{AttemptId, CallbackTransactionId, ProgrammeId},
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const SOURCE_INPUT_HASHES: &[u8] = include_bytes!(
    "../../../../../outputs/IMPLEMENTER_SUPPLEMENTS/F06NR-SOURCE-FRAME-INPUT-ONLY-CROSSWALK-V1/SOURCE-VECTOR-HASHES.json"
);
const D05_PARAMETERS: &[u8] =
    include_bytes!("../../../tests/fixtures/transient_channel/PARAMETERS.json");
const F07_BLOCK_SIZE: usize = 256;

fn synthetic_source(source_index: usize) -> (Box<DelayedPrimaryRows>, ShortConsumedFlags) {
    let mut rows = Vec::with_capacity(EXTERNAL_ROW_COUNT);
    for row in 0..EXTERNAL_ROW_COUNT {
        let mut row_words = [0_u32; SOURCE_SAMPLE_COUNT];
        for (sample, word) in row_words.iter_mut().enumerate() {
            let numerator =
                i32::try_from((17 * source_index + 11 * row + 5 * (sample % 257)) % 257).unwrap()
                    - 128;
            *word = (numerator as f32 / 32_768.0).to_bits();
        }
        rows.push(row_words);
    }
    let samples = Box::<DelayedPrimaryRows>::try_from(rows.into_boxed_slice()).unwrap();
    let flags = std::array::from_fn(|block| {
        std::array::from_fn(|lane| ((source_index + block + lane) % 2) as u8)
    });
    (samples, flags)
}

fn source_input_sha256(samples: &DelayedPrimaryRows, flags: &ShortConsumedFlags) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"SECONDARY-SOURCE-INPUT-V1\0");
    for row in samples {
        for word in row {
            hasher.update(word.to_le_bytes());
        }
    }
    for block in flags {
        hasher.update(block);
    }
    hasher.finalize().into()
}

fn expected_input_hash(source_index: usize) -> String {
    let document: Value = serde_json::from_slice(SOURCE_INPUT_HASHES).unwrap();
    document["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["source_frame"] == source_index)
        .unwrap()["input_sha256"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(*byte >> 4)]));
        output.push(char::from(HEX[usize::from(*byte & 0x0f)]));
    }
    output
}

fn measurement_receipt(attempt: u64) -> ProgrammeMeasurementReceipt {
    ProgrammeMeasurementReceipt::from_verified_prepass(ProgrammeMeasurementReceiptInput {
        programme_id: ProgrammeId::new("secondary-source-owner-test").unwrap(),
        attempt: AttemptId::new(attempt),
        input_sha256: [0x11; 32],
        config_sha256: [0x22; 32],
        dialogue_word: (-23.0_f32).to_bits(),
        itu_word: (-24.0_f32).to_bits(),
        true_peak_word: (-1.0_f32).to_bits(),
        gating_kind: "LEVEL",
        selected_lufs_word: (-24.0_f32).to_bits(),
        meter_terminal_sha256: [0x33; 32],
        normal_callbacks: 4_247,
        zero_drains: 3,
        energy_windows: 6_794,
    })
    .unwrap()
}

fn callback_identity(source_index: u64, attempt: AttemptId) -> CallbackLineageIdentity {
    let callback_index = u32::try_from(source_index + 2).unwrap();
    CallbackLineageIdentity::new(
        ProgrammeId::new("secondary-source-owner-test").unwrap(),
        attempt,
        FrontendCallbackIndex::new(callback_index),
    )
}

fn f07_shared_analysis(source_index: u64, attempt: AttemptId) -> SharedAnalysisReceipt {
    let transaction_id = format!("secondary-source-{source_index}");
    let transaction = CallbackTransactionId::new(&transaction_id).unwrap();
    let before = F07SourceFrameBeforeState::new(
        std::array::from_fn(|_| InputHistory::cold()),
        std::array::from_fn(|_| TransientChannelState::cold()),
        [false; SECONDARY_LANE_COUNT],
    );
    let mut installed_rows = Vec::with_capacity(EXTERNAL_ROW_COUNT);
    for row in 0..EXTERNAL_ROW_COUNT {
        let mut blocks = [[0_u32; F07_BLOCK_SIZE]; BLOCK_COUNT];
        for (block, block_words) in blocks.iter_mut().enumerate() {
            for (sample, word) in block_words.iter_mut().enumerate() {
                let sample_index = block * F07_BLOCK_SIZE + sample;
                let numerator = i32::try_from(
                    (17 * usize::try_from(source_index).unwrap()
                        + 11 * row
                        + 5 * (sample_index % 257))
                        % 257,
                )
                .unwrap()
                    - 128;
                *word = (numerator as f32 / 32_768.0).to_bits();
            }
        }
        installed_rows.push(blocks);
    }
    let installed_audio =
        Box::<InstalledAudioSourceFrame>::try_from(installed_rows.into_boxed_slice()).unwrap();
    let transient_parameters = load_transient_parameters(Some(D05_PARAMETERS)).unwrap();
    let f07_proposal = stage_f07a_source_frame(&before, &installed_audio, &transient_parameters)
        .expect("the synthetic installed frame has valid finite F07 inputs");
    let source_index = CodecSourceIndex::new(source_index);
    let before_generation = AnalysisOwnerGeneration::from_parent_snapshot(source_index.value());
    let source_key =
        SourceStageKey::from_parent_snapshot(attempt, transaction, source_index, before_generation);
    SharedAnalysisReceipt::from_parent(
        source_key,
        AnalysisOwnerGeneration::from_parent_snapshot(source_index.value() + 1),
        f07_proposal,
    )
}

fn assert_carrier_zero_slots(carrier: &SecondaryCarrier) {
    let rows = carrier.rows();
    assert_eq!(rows.len(), EXTERNAL_ROW_COUNT);
    assert!(rows[3].iter().all(|word| *word == 0));
    for row in rows {
        assert_eq!(row.len(), SOURCE_SAMPLE_COUNT);
        assert!(row[5 * BLOCK_SAMPLE_COUNT..].iter().all(|word| *word == 0));
    }
}

#[test]
fn input_only_source_recipes_match_hashes_and_chain_f06nr_state() {
    let parameters = SecondarySourceParameters::selected().unwrap();
    let mut secondary_state = NoneDrcSecondaryState::cold();

    for source_value in 0..3_u64 {
        let (samples, flags) = synthetic_source(source_value as usize);
        assert_eq!(
            hex(&source_input_sha256(&samples, &flags)),
            expected_input_hash(source_value as usize),
            "the Wash vector hash covers only generated inputs and flags"
        );
        let proposal = stage_secondary_frame(
            &secondary_state,
            CodecSourceIndex::new(source_value),
            &samples,
            &flags,
            &parameters,
        )
        .unwrap();

        assert_eq!(proposal.after_state.frame_index(), source_value + 1);
        assert_carrier_zero_slots(&proposal.secondary_carrier);
        assert!(
            proposal
                .after_state
                .forward_history()
                .iter()
                .all(|history| history
                    .words()
                    .iter()
                    .all(|word| f32::from_bits(*word).is_finite()))
        );
        assert!(proposal.after_state.inverse_carry().iter().all(|carry| {
            carry
                .words()
                .iter()
                .all(|word| f32::from_bits(*word).is_finite())
        }));

        secondary_state = proposal.after_state;
    }
}

#[test]
fn f08_primary_fold_and_spectrum_ignore_companion_history_scratch() {
    let parameters = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
    let (samples, _) = synthetic_source(1);
    let mut input_block = [0_u32; BLOCK_SAMPLE_COUNT];
    input_block.copy_from_slice(&samples[2][2 * BLOCK_SAMPLE_COUNT..3 * BLOCK_SAMPLE_COUNT]);
    let primary_history = [0_u32; 128];
    let companion_zero = [0_u32; 128];
    let companion_other = [0x3f00_0000_u32; 128];

    let folded_zero =
        f08_window_fold_block(&input_block, &primary_history, &companion_zero, &parameters)
            .unwrap();
    let folded_other = f08_window_fold_block(
        &input_block,
        &primary_history,
        &companion_other,
        &parameters,
    )
    .unwrap();
    assert_eq!(
        folded_zero.primary_fold_bits(),
        folded_other.primary_fold_bits()
    );
    assert_eq!(
        folded_zero.primary_history_after(),
        folded_other.primary_history_after()
    );

    for short_flag in [0, 1] {
        let transformed_zero =
            f08_fbw_transform_block(&folded_zero, short_flag, &parameters).unwrap();
        let transformed_other =
            f08_fbw_transform_block(&folded_other, short_flag, &parameters).unwrap();
        assert_eq!(
            transformed_zero.primary_spectrum_bits(),
            transformed_other.primary_spectrum_bits(),
            "PRIMARY long and short routes depend only on the DRC-owned primary history"
        );
    }
}

#[test]
fn block_zero_advances_both_private_histories_without_a_carrier_destination() {
    let parameters = SecondarySourceParameters::selected().unwrap();
    let (samples, flags) = synthetic_source(0);
    let mut input_block = [0_u32; BLOCK_SAMPLE_COUNT];
    input_block.copy_from_slice(&samples[0][..BLOCK_SAMPLE_COUNT]);
    let forward_before = LongTransformHistory::zeroed();
    let inverse_before = SecondaryOverlapCarry::cold();

    let proposal = stage_secondary_lane_block(
        &input_block,
        &forward_before,
        &inverse_before,
        flags[0][0],
        0,
        &parameters,
    )
    .unwrap();

    assert_eq!(proposal.disposition, CarrierDisposition::SuppressWarmup);
    assert_ne!(proposal.forward_history_after, forward_before);
    assert_ne!(proposal.inverse_carry_after, inverse_before);
    assert_eq!(carrier_disposition(1), CarrierDisposition::WriteBlock(0));
    assert_eq!(carrier_disposition(5), CarrierDisposition::WriteBlock(4));
}

#[test]
fn long_and_short_flags_complete_the_matching_f08_and_f06ni_paths() {
    let parameters = SecondarySourceParameters::selected().unwrap();
    let (samples, _) = synthetic_source(2);
    let mut input_block = [0_u32; BLOCK_SAMPLE_COUNT];
    input_block.copy_from_slice(&samples[4][BLOCK_SAMPLE_COUNT..2 * BLOCK_SAMPLE_COUNT]);
    let forward_before = LongTransformHistory::zeroed();
    let inverse_before = SecondaryOverlapCarry::cold();

    for short_consumed in [0, 1] {
        let proposal = stage_secondary_lane_block(
            &input_block,
            &forward_before,
            &inverse_before,
            short_consumed,
            1,
            &parameters,
        )
        .unwrap();
        assert_eq!(proposal.disposition, CarrierDisposition::WriteBlock(0));
        assert!(
            proposal
                .forward_history_after
                .words()
                .iter()
                .all(|word| f32::from_bits(*word).is_finite())
        );
        assert!(
            proposal
                .inverse_carry_after
                .words()
                .iter()
                .all(|word| f32::from_bits(*word).is_finite())
        );
        assert!(
            proposal
                .overlap_words
                .iter()
                .all(|word| f32::from_bits(*word).is_finite())
        );
    }
}

#[test]
fn f06nd_receives_the_same_source_carrier_and_only_a_proposal_is_returned() {
    let measurement = measurement_receipt(4);
    let root = NoneDrcOwnerRoot::cold_after_meter_finalize(&measurement).unwrap();
    let callback = callback_identity(0, measurement.attempt());
    let shared = f07_shared_analysis(0, measurement.attempt());
    let (delayed_rows, short_flags) = source_inputs_from_receipt(&shared).unwrap();
    let f07_output = shared.f07_proposal().output();
    let f07_audio = f07_output.delayed_audio();
    for external_row in 0..EXTERNAL_ROW_COUNT {
        for block in 0..BLOCK_COUNT {
            let start = block * BLOCK_SAMPLE_COUNT;
            assert_eq!(
                &delayed_rows[external_row][start..start + BLOCK_SAMPLE_COUNT],
                &f07_audio[external_row][block],
                "F06NR consumes the exact delayed row/block from its SharedAnalysisReceipt"
            );
        }
    }
    assert_eq!(&short_flags, f07_output.short_consumed());
    assert_eq!(
        delayed_rows.len(),
        EXTERNAL_ROW_COUNT,
        "F06NR rows come directly from this receipt's F07 output"
    );
    let before_reference = root.state_reference().unwrap();
    let parameters = SecondarySourceParameters::selected().unwrap();
    let direct_secondary = stage_secondary_frame(
        root.state().secondary(),
        CodecSourceIndex::new(0),
        &delayed_rows,
        &short_flags,
        &parameters,
    )
    .unwrap();

    let proposal = root.prepare_source_frame(&shared, &callback).unwrap();

    assert_eq!(proposal.source_index(), CodecSourceIndex::new(0));
    assert_eq!(
        proposal.dispatch_index(),
        CodecSourceIndex::new(0).dispatch().unwrap()
    );
    assert_eq!(proposal.callback_identity(), &callback);
    assert_eq!(
        proposal
            .after_root()
            .shared_analysis_identity()
            .source_index(),
        CodecSourceIndex::new(0)
    );
    assert_eq!(
        proposal.after_root().source_index(),
        CodecSourceIndex::new(0)
    );
    assert_eq!(
        proposal.after_root().state().source_generation().unwrap(),
        CodecSourceIndex::new(1)
    );
    assert_eq!(proposal.after_root().state().secondary().frame_index(), 1);
    assert_eq!(proposal.after_root().state().scheduler().generation(), 1);
    assert_eq!(
        proposal.after_root().state().secondary(),
        &direct_secondary.after_state,
        "the owner proposal carries the after-state staged from its own F07 receipt"
    );
    assert_eq!(
        proposal.secondary_carrier().rows(),
        direct_secondary.secondary_carrier.rows()
    );
    assert_eq!(
        proposal
            .after_root()
            .state()
            .override_fifo()
            .frames_processed(),
        1
    );
    assert_carrier_zero_slots(proposal.secondary_carrier());

    assert_eq!(root.state_reference().unwrap(), before_reference);
    assert_eq!(
        root.state().source_generation().unwrap(),
        CodecSourceIndex::new(0)
    );
}

#[test]
fn generation_mismatch_and_invalid_inputs_leave_the_committed_root_unchanged() {
    let measurement = measurement_receipt(8);
    let root = NoneDrcOwnerRoot::cold_after_meter_finalize(&measurement).unwrap();
    let before_reference = root.state_reference().unwrap();
    let skipped = root.prepare_source_frame(
        &f07_shared_analysis(1, measurement.attempt()),
        &callback_identity(1, measurement.attempt()),
    );
    assert!(matches!(
        skipped,
        Err(SecondarySourceError::Owner(
            NoneDrcOwnerError::StateGenerationMismatch
        ))
    ));

    let (mut source_zero, mut flags_zero) = synthetic_source(0);
    flags_zero[0][0] = 2;
    assert_eq!(
        validate_inputs(&source_zero, &flags_zero),
        Err(SecondarySourceError::ShortFlagOutOfRange)
    );

    source_zero[0][0] = f32::INFINITY.to_bits();
    flags_zero[0][0] = 0;
    assert_eq!(
        validate_inputs(&source_zero, &flags_zero),
        Err(SecondarySourceError::InputNonFinite)
    );
    assert_eq!(root.state_reference().unwrap(), before_reference);
}

#[test]
fn forward_history_bit_constructor_preserves_words_and_rejects_nonfinite_values() {
    let words = std::array::from_fn(|index| {
        if index % 2 == 0 {
            0x3f00_0000
        } else {
            0x8000_0000
        }
    });
    let reconstructed = LongTransformHistory::try_from_words(&words).unwrap();
    assert_eq!(reconstructed.words(), &words);
    assert_eq!(
        LongTransformHistory::try_from_words(&[0_u32; 128]).unwrap(),
        LongTransformHistory::zeroed()
    );

    let mut nonfinite = [0_u32; 128];
    nonfinite[73] = f32::INFINITY.to_bits();
    assert!(LongTransformHistory::try_from_words(&nonfinite).is_none());
    nonfinite[73] = f32::NAN.to_bits();
    assert!(LongTransformHistory::try_from_words(&nonfinite).is_none());
}
