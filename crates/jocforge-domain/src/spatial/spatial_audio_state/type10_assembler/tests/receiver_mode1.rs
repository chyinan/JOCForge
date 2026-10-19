// pattern: Imperative Shell

use super::*;
use crate::{
    frontend::{PackageManifestEvidence, ProviderFileEvidence},
    numeric::FiniteF32,
    spatial::{
        QmfMathematicalFftProvider, QmfReceiverFunctionalProvider,
        qmf_analysis_synthesis::{
            QmfAnalysisHistory, QmfReceiverMode1Histories, analyze_qmf_slot,
            process_qmf_receiver_mode1_callback,
        },
        spatial_audio_state::{
            ModelState, RawBytes,
            receiver_mode1::{
                CarrierAnalysisSpectra, ReceiverMode1AfterState, ReceiverMode1CallbackOutput,
                ReceiverMode1Error, ReceiverMode1ReadyOutput, ReceiverS09InputView,
                stage_receiver_mode1,
            },
        },
    },
    timeline::{FrontendCallbackIndex, ReceiverCallbackCount},
};
use std::sync::Arc;

const MANIFEST_BYTES: &[u8] =
    include_bytes!("../../../../../tests/fixtures/f01e_provider/MANIFEST.json");
const QMF_FUNCTIONAL_BYTES: &[u8] =
    include_bytes!("../../../../../tests/fixtures/f04_parameters/qmf-receiver-functional-v1.json");
const QMF_FFT_BYTES: &[u8] =
    include_bytes!("../../../../../tests/fixtures/f04_parameters/qmf-mathematical-fft-v1.json");
const RECEIVER_LANES: [usize; 5] = [0, 1, 2, 4, 5];
const FRAME_SAMPLES: usize = 1536;
const SLOT_SAMPLES: usize = 64;
const SLOT_COUNT: usize = 24;
const SPECTRUM_WORDS: usize = 128;

struct ExpectedAnalysis {
    spectra: CarrierAnalysisSpectra,
    histories_after: Vec<[[u32; 64]; 9]>,
}

fn validated_qmf_providers() -> (QmfReceiverFunctionalProvider, QmfMathematicalFftProvider) {
    let manifest_evidence = PackageManifestEvidence::new(
        Arc::<[u8]>::from(MANIFEST_BYTES),
        Some("shared package manifest fixture".to_owned()),
    );
    let manifest = crate::spatial::validate_f04_parameter_package_manifest(&manifest_evidence)
        .expect("the shared manifest validates the selected QMF provider entries");
    let functional = crate::spatial::qmf_receiver_functional_provider::
        validate_qmf_receiver_functional_provider(
            &ProviderFileEvidence::new(
                Arc::<[u8]>::from(QMF_FUNCTIONAL_BYTES),
                Some("selected QMF functional provider fixture".to_owned()),
            ),
            &manifest,
        )
        .expect("the identity-matched QMF functional provider validates");
    let fft = crate::spatial::validate_qmf_mathematical_fft_provider(
        &ProviderFileEvidence::new(
            Arc::<[u8]>::from(QMF_FFT_BYTES),
            Some("selected QMF mathematical FFT fixture".to_owned()),
        ),
        &manifest,
    )
    .expect("the identity-matched QMF mathematical FFT provider validates");
    (functional, fft)
}

fn distinct_s09_lanes(seed: u32) -> [Vec<FiniteF32>; 6] {
    std::array::from_fn(|lane| {
        (0..FRAME_SAMPLES)
            .map(|sample| {
                let word = seed
                    .checked_add(u32::try_from(lane * 16_384 + sample).unwrap())
                    .unwrap();
                FiniteF32::new((word as f32) / 65_536.0).unwrap()
            })
            .collect()
    })
}

fn lane_slices(lanes: &[Vec<FiniteF32>; 6]) -> [&[FiniteF32]; 6] {
    std::array::from_fn(|lane| lanes[lane].as_slice())
}

fn analysis_history_from_raw(rows: &[RawBytes; 9]) -> QmfAnalysisHistory {
    let words = rows.iter().map(raw_words::<64>).collect::<Vec<_>>();
    QmfAnalysisHistory::from_words(words.try_into().expect("nine fixed analysis rows"))
        .expect("before analysis history contains finite words")
}

fn raw_words<const COUNT: usize>(raw: &RawBytes) -> [u32; COUNT] {
    assert_eq!(raw.bytes.len(), COUNT * 4);
    std::array::from_fn(|word| {
        let offset = word * 4;
        u32::from_le_bytes(raw.bytes[offset..offset + 4].try_into().unwrap())
    })
}

fn mode1_histories_from_raw(receiver: &ModelState, surround: usize) -> QmfReceiverMode1Histories {
    let delay_history = receiver.qmf_delay_history[surround]
        .iter()
        .map(raw_words::<SPECTRUM_WORDS>)
        .collect::<Vec<_>>()
        .try_into()
        .expect("ten fixed mode=1 delay rows");
    let fir_words = raw_words::<40>(&receiver.qmf_fir_history[surround]);
    let fir_history =
        std::array::from_fn(|history| [fir_words[history * 2], fir_words[history * 2 + 1]]);
    let synthesis_history = raw_words::<576>(&receiver.synthesis_history[surround]);
    QmfReceiverMode1Histories::from_words(delay_history, fir_history, synthesis_history)
        .expect("before mode=1 histories contain finite words")
}

fn expected_analysis(
    receiver_before: &ModelState,
    s09_input: &[&[FiniteF32]; 6],
    functional: &QmfReceiverFunctionalProvider,
    fft: &QmfMathematicalFftProvider,
) -> ExpectedAnalysis {
    let mut spectra: CarrierAnalysisSpectra =
        std::array::from_fn(|_| Box::new([[0_u32; SPECTRUM_WORDS]; SLOT_COUNT]));
    let mut histories_after = Vec::with_capacity(RECEIVER_LANES.len());
    for (carrier, receiver_lane) in RECEIVER_LANES.into_iter().enumerate() {
        let mut history = analysis_history_from_raw(&receiver_before.analysis_history[carrier]);
        for (slot, spectrum_slot) in spectra[carrier].iter_mut().enumerate() {
            let start = slot * SLOT_SAMPLES;
            let current =
                std::array::from_fn(|sample| s09_input[receiver_lane][start + sample].value());
            let result = analyze_qmf_slot(&current, &history, functional, fft)
                .expect("one finite receiver analysis slot succeeds");
            *spectrum_slot = *result.spectrum_words();
            history = QmfAnalysisHistory::from_words(*result.history_after().rows())
                .expect("analysis after history remains finite");
        }
        histories_after.push(*history.rows());
    }
    ExpectedAnalysis {
        spectra,
        histories_after,
    }
}

fn receiver_type10_fixture(
    context: &TestContext,
    callback: u64,
    phase: CallbackPhase,
) -> (
    SpatialAudioJoinMetadata,
    super::super::Type10AssemblyProposal,
) {
    let parent = selected_parent(context, callback);
    let component = component_proposal(
        context,
        &parent,
        phase,
        None,
        COMPONENT_ROWS * FRAME_SAMPLES,
    );
    let after = ring_after(callback, None);
    let row0 = callback
        .checked_sub(2)
        .map(|origin| lfe_row0(callback, origin, None));
    let lfe = lfe_proposal(context, &parent, phase, row0, after, None);
    let proposal = assemble_type10(&parent, phase, component, lfe)
        .expect("the selected E2/E3 callback pair assembles Type10");
    (parent, proposal)
}

fn assert_analysis_history(actual: &[[RawBytes; 9]; 5], expected: &[[[u32; 64]; 9]]) {
    assert_eq!(expected.len(), 5);
    for carrier in 0..5 {
        for row in 0..9 {
            assert_eq!(
                raw_words::<64>(&actual[carrier][row]),
                expected[carrier][row]
            );
        }
    }
}

#[test]
fn g0_g1_priming_preserves_component_lfe_and_skips_receiver_qmf() {
    let context = context();
    let (functional, fft) = validated_qmf_providers();
    for callback in 0..=1 {
        let (parent, type10) = receiver_type10_fixture(&context, callback, CallbackPhase::Normal);
        let before = ModelState::cold();
        let expected_component = component_proposal(
            &context,
            &parent,
            CallbackPhase::Normal,
            None,
            COMPONENT_ROWS * FRAME_SAMPLES,
        )
        .into_parts()
        .0;
        let expected_lfe = lfe_proposal(
            &context,
            &parent,
            CallbackPhase::Normal,
            None,
            ring_after(callback, None),
            None,
        )
        .into_parts()
        .0;
        let proposal = stage_receiver_mode1(
            &parent,
            CallbackPhase::Normal,
            &before,
            ReceiverCallbackCount::new(0),
            type10,
            None,
            &functional,
            &fft,
        )
        .expect("Type10 priming produces a receiver Priming proposal");
        let (component_after, lfe_after, type10_outcome, receiver_outcome) = proposal.into_parts();
        assert_eq!(component_after.audio, expected_component.audio);
        assert_eq!(component_after.parameters, expected_component.parameters);
        assert_eq!(
            component_after.filter_history,
            expected_component.filter_history
        );
        assert_eq!(lfe_after, expected_lfe);
        assert!(matches!(
            type10_outcome,
            Type10AssemblyOutcome::Priming { .. }
        ));
        assert!(matches!(
            receiver_outcome,
            ReceiverMode1CallbackOutput::Priming {
                receiver_callback_count_after
            } if receiver_callback_count_after == ReceiverCallbackCount::new(0)
        ));
    }
}

#[test]
fn normal_receiver_callback_analyzes_five_carriers_and_synthesizes_only_surrounds() {
    let context = context();
    let (parent, type10) = receiver_type10_fixture(&context, 2, CallbackPhase::Normal);
    let before = ModelState::cold();
    let lanes = distinct_s09_lanes(0);
    let s09_lanes = lane_slices(&lanes);
    let s09_input =
        ReceiverS09InputView::from_post_lfe_stage(&parent, CallbackPhase::Normal, s09_lanes)
            .unwrap();
    let (functional, fft) = validated_qmf_providers();
    let expected = expected_analysis(&before, &s09_lanes, &functional, &fft);

    let proposal = stage_receiver_mode1(
        &parent,
        CallbackPhase::Normal,
        &before,
        ReceiverCallbackCount::new(0),
        type10,
        Some(&s09_input),
        &functional,
        &fft,
    )
    .expect("the first ready receiver callback stages the fixed QMF routes");
    let (_, _, type10_outcome, receiver_outcome) = proposal.into_parts();
    let Type10AssemblyOutcome::Ready { packet, receipt } = type10_outcome else {
        panic!("g=2 has a ready Type10 packet")
    };
    assert_eq!(packet.callback_index(), FrontendCallbackIndex::new(2));
    assert_eq!(
        receipt.lfe_origin_generation(),
        Some(FrontendCallbackIndex::new(0))
    );
    assert_eq!(
        receipt.component_origin_generation(),
        FrontendCallbackIndex::new(2)
    );
    assert_eq!(packet.shape(), [16, FRAME_SAMPLES]);

    let ReceiverMode1CallbackOutput::Ready(ready) = receiver_outcome else {
        panic!("g=2 runs receiver mode=1 QMF")
    };
    let ReceiverMode1ReadyOutput {
        analysis_spectra,
        surround_pcm,
        after,
    } = *ready;
    assert_eq!(analysis_spectra, expected.spectra);
    assert_analysis_history(&after.analysis_history, &expected.histories_after);
    assert_eq!(after.receiver_callback_count, ReceiverCallbackCount::new(1));
    assert_eq!(surround_pcm[0].len(), FRAME_SAMPLES);
    assert_eq!(surround_pcm[1].len(), FRAME_SAMPLES);

    for (surround, carrier) in [(0, 3), (1, 4)] {
        let expected = process_qmf_receiver_mode1_callback(
            &expected.spectra[carrier],
            &QmfReceiverMode1Histories::cold(),
            &functional,
            &fft,
        )
        .expect("direct surround mode=1 core accepts 24 analysis slots");
        let expected_pcm = expected
            .pcm_slots()
            .iter()
            .flatten()
            .map(|sample| FiniteF32::new(*sample).expect("finite direct QMF output"))
            .collect::<Vec<_>>();
        assert_eq!(surround_pcm[surround].as_ref(), expected_pcm.as_slice());

        let actual_delay = std::array::from_fn(|slot| {
            raw_words::<SPECTRUM_WORDS>(&after.qmf_delay_history[surround][slot])
        });
        assert_eq!(&actual_delay, expected.history_after().delay_history());
        let actual_fir = raw_words::<40>(&after.qmf_fir_history[surround]);
        let expected_fir = expected
            .history_after()
            .band0_fir_history()
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(actual_fir.as_slice(), expected_fir.as_slice());
        assert_eq!(
            raw_words::<576>(&after.synthesis_history[surround]),
            *expected.history_after().synthesis_history()
        );
    }
}

#[test]
fn cached_drain_consumes_warm_histories_once_and_advances_receiver_count_once() {
    let context = context();
    let (functional, fft) = validated_qmf_providers();
    let first_lanes = distinct_s09_lanes(0);
    let first_s09_lanes = lane_slices(&first_lanes);
    let (first_parent, first_type10) = receiver_type10_fixture(&context, 2, CallbackPhase::Normal);
    let first_s09 = ReceiverS09InputView::from_post_lfe_stage(
        &first_parent,
        CallbackPhase::Normal,
        first_s09_lanes,
    )
    .unwrap();
    let first = stage_receiver_mode1(
        &first_parent,
        CallbackPhase::Normal,
        &ModelState::cold(),
        ReceiverCallbackCount::new(0),
        first_type10,
        Some(&first_s09),
        &functional,
        &fft,
    )
    .expect("normal receiver callback produces warm QMF after histories");
    let (_, _, _, first_outcome) = first.into_parts();
    let ReceiverMode1CallbackOutput::Ready(first_ready) = first_outcome else {
        panic!("g=2 is a ready receiver callback")
    };
    let ReceiverMode1AfterState {
        analysis_history,
        qmf_delay_history,
        qmf_fir_history,
        synthesis_history,
        receiver_callback_count: count,
    } = first_ready.after;
    assert_eq!(count, ReceiverCallbackCount::new(1));
    let mut warm_before = ModelState::cold();
    warm_before.analysis_history = analysis_history;
    warm_before.qmf_delay_history = qmf_delay_history;
    warm_before.qmf_fir_history = qmf_fir_history;
    warm_before.synthesis_history = synthesis_history;

    let drain_lanes = distinct_s09_lanes(8_192);
    let drain_s09_lanes = lane_slices(&drain_lanes);
    let (drain_parent, drain_type10) =
        receiver_type10_fixture(&context, 3, CallbackPhase::CachedDrain);
    let drain_s09 = ReceiverS09InputView::from_post_lfe_stage(
        &drain_parent,
        CallbackPhase::CachedDrain,
        drain_s09_lanes,
    )
    .unwrap();
    let expected_drain_analysis =
        expected_analysis(&warm_before, &drain_s09_lanes, &functional, &fft);
    let drain = stage_receiver_mode1(
        &drain_parent,
        CallbackPhase::CachedDrain,
        &warm_before,
        count,
        drain_type10,
        Some(&drain_s09),
        &functional,
        &fft,
    )
    .expect("cached drain is a real receiver callback over warm histories");
    let (_, _, type10_outcome, receiver_outcome) = drain.into_parts();
    assert!(matches!(
        type10_outcome,
        Type10AssemblyOutcome::Ready { .. }
    ));
    let ReceiverMode1CallbackOutput::Ready(ready) = receiver_outcome else {
        panic!("g=3 cached drain runs receiver QMF")
    };
    assert_eq!(ready.analysis_spectra, expected_drain_analysis.spectra);
    assert_analysis_history(
        &ready.after.analysis_history,
        &expected_drain_analysis.histories_after,
    );
    assert_eq!(ready.surround_pcm[0].len(), FRAME_SAMPLES);
    assert_eq!(ready.surround_pcm[1].len(), FRAME_SAMPLES);
    assert_eq!(
        ready.after.receiver_callback_count,
        ReceiverCallbackCount::new(2)
    );
    for (carrier, receiver_lane) in RECEIVER_LANES.into_iter().enumerate() {
        let last_slot_start = (SLOT_COUNT - 1) * SLOT_SAMPLES;
        let last_history = raw_words::<64>(&ready.after.analysis_history[carrier][0]);
        for sample in 0..SLOT_SAMPLES {
            assert_eq!(
                last_history[sample],
                drain_s09_lanes[receiver_lane][last_slot_start + sample].to_bits()
            );
        }
    }
    for (surround, carrier) in [(0, 3), (1, 4)] {
        let warm_expected = process_qmf_receiver_mode1_callback(
            &expected_drain_analysis.spectra[carrier],
            &mode1_histories_from_raw(&warm_before, surround),
            &functional,
            &fft,
        )
        .expect("cached drain processes the actual warm receiver histories");
        let cold_expected = process_qmf_receiver_mode1_callback(
            &expected_drain_analysis.spectra[carrier],
            &QmfReceiverMode1Histories::cold(),
            &functional,
            &fft,
        )
        .expect("cold comparison callback remains valid");
        assert_ne!(
            warm_expected.pcm_slots(),
            cold_expected.pcm_slots(),
            "the chosen callback input distinguishes warm and cold mode=1 histories"
        );
        let expected_pcm = warm_expected
            .pcm_slots()
            .iter()
            .flatten()
            .map(|sample| FiniteF32::new(*sample).expect("finite warm QMF output"))
            .collect::<Vec<_>>();
        assert_eq!(
            ready.surround_pcm[surround].as_ref(),
            expected_pcm.as_slice()
        );

        let actual_delay = std::array::from_fn(|slot| {
            raw_words::<SPECTRUM_WORDS>(&ready.after.qmf_delay_history[surround][slot])
        });
        assert_eq!(&actual_delay, warm_expected.history_after().delay_history());
        let actual_fir = raw_words::<40>(&ready.after.qmf_fir_history[surround]);
        let expected_fir = warm_expected
            .history_after()
            .band0_fir_history()
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(actual_fir.as_slice(), expected_fir.as_slice());
        assert_eq!(
            raw_words::<576>(&ready.after.synthesis_history[surround]),
            *warm_expected.history_after().synthesis_history()
        );
    }
}

#[test]
fn stale_receiver_count_rejects_before_returning_after_state() {
    let context = context();
    let (parent, type10) = receiver_type10_fixture(&context, 2, CallbackPhase::Normal);
    let lanes = distinct_s09_lanes(0);
    let s09_lanes = lane_slices(&lanes);
    let s09_input =
        ReceiverS09InputView::from_post_lfe_stage(&parent, CallbackPhase::Normal, s09_lanes)
            .unwrap();
    let (functional, fft) = validated_qmf_providers();
    let result = stage_receiver_mode1(
        &parent,
        CallbackPhase::Normal,
        &ModelState::cold(),
        ReceiverCallbackCount::new(1),
        type10,
        Some(&s09_input),
        &functional,
        &fft,
    );
    assert!(matches!(result, Err(ReceiverMode1Error::State)));
}

#[test]
fn phase_and_type10_callback_mismatch_reject_before_qmf_work() {
    let context = context();
    let (parent, type10) = receiver_type10_fixture(&context, 2, CallbackPhase::Normal);
    let lanes = distinct_s09_lanes(0);
    let s09_lanes = lane_slices(&lanes);
    let s09_input =
        ReceiverS09InputView::from_post_lfe_stage(&parent, CallbackPhase::Normal, s09_lanes)
            .unwrap();
    let (functional, fft) = validated_qmf_providers();
    let wrong_phase = stage_receiver_mode1(
        &parent,
        CallbackPhase::CachedDrain,
        &ModelState::cold(),
        ReceiverCallbackCount::new(0),
        type10,
        Some(&s09_input),
        &functional,
        &fft,
    );
    assert!(matches!(
        wrong_phase,
        Err(ReceiverMode1Error::CallbackOrder)
    ));

    let (parent, type10) = receiver_type10_fixture(&context, 2, CallbackPhase::CachedDrain);
    let s09_lanes = lane_slices(&lanes);
    let mismatched_phase_s09 =
        ReceiverS09InputView::from_post_lfe_stage(&parent, CallbackPhase::Normal, s09_lanes)
            .unwrap();
    let s09_phase_mismatch = stage_receiver_mode1(
        &parent,
        CallbackPhase::CachedDrain,
        &ModelState::cold(),
        ReceiverCallbackCount::new(0),
        type10,
        Some(&mismatched_phase_s09),
        &functional,
        &fft,
    );
    assert!(matches!(
        s09_phase_mismatch,
        Err(ReceiverMode1Error::CallbackOrder)
    ));

    let (parent, type10) = receiver_type10_fixture(&context, 2, CallbackPhase::Normal);
    let wrong_parent = parent_join(
        &context,
        3,
        "type10-test-programme",
        7,
        "type10-test-transaction",
        context.profile,
        context.spatial_audio.generation(),
        context.spatial_audio.digest(),
    );
    let wrong_s09 =
        ReceiverS09InputView::from_post_lfe_stage(&wrong_parent, CallbackPhase::Normal, s09_lanes)
            .unwrap();
    let wrong_generation = stage_receiver_mode1(
        &parent,
        CallbackPhase::Normal,
        &ModelState::cold(),
        ReceiverCallbackCount::new(0),
        type10,
        Some(&wrong_s09),
        &functional,
        &fft,
    );
    assert!(matches!(
        wrong_generation,
        Err(ReceiverMode1Error::CallbackOrder)
    ));

    let (parent, _) = receiver_type10_fixture(&context, 2, CallbackPhase::Normal);
    let lanes = distinct_s09_lanes(0);
    let s09_lanes = lane_slices(&lanes);
    let s09_input =
        ReceiverS09InputView::from_post_lfe_stage(&parent, CallbackPhase::Normal, s09_lanes)
            .unwrap();
    let (_, wrong_type10_callback) = receiver_type10_fixture(&context, 3, CallbackPhase::Normal);
    let wrong_type10_callback = stage_receiver_mode1(
        &parent,
        CallbackPhase::Normal,
        &ModelState::cold(),
        ReceiverCallbackCount::new(0),
        wrong_type10_callback,
        Some(&s09_input),
        &functional,
        &fft,
    );
    assert!(matches!(
        wrong_type10_callback,
        Err(ReceiverMode1Error::CallbackOrder)
    ));
}

#[test]
fn priming_and_ready_input_shapes_are_checked_before_qmf_work() {
    let context = context();
    let (functional, fft) = validated_qmf_providers();
    let lanes = distinct_s09_lanes(0);
    let s09_lanes = lane_slices(&lanes);
    let (parent, type10) = receiver_type10_fixture(&context, 0, CallbackPhase::Normal);
    let priming_s09 =
        ReceiverS09InputView::from_post_lfe_stage(&parent, CallbackPhase::Normal, s09_lanes)
            .unwrap();
    let unexpected_priming_audio = stage_receiver_mode1(
        &parent,
        CallbackPhase::Normal,
        &ModelState::cold(),
        ReceiverCallbackCount::new(0),
        type10,
        Some(&priming_s09),
        &functional,
        &fft,
    );
    assert!(matches!(
        unexpected_priming_audio,
        Err(ReceiverMode1Error::Input)
    ));

    let (parent, type10) = receiver_type10_fixture(&context, 2, CallbackPhase::Normal);
    let missing_ready_audio = stage_receiver_mode1(
        &parent,
        CallbackPhase::Normal,
        &ModelState::cold(),
        ReceiverCallbackCount::new(0),
        type10,
        None,
        &functional,
        &fft,
    );
    assert!(matches!(
        missing_ready_audio,
        Err(ReceiverMode1Error::Input)
    ));

    let (parent, _) = receiver_type10_fixture(&context, 2, CallbackPhase::Normal);
    let mut malformed_lanes = distinct_s09_lanes(0);
    malformed_lanes[4].pop();
    let malformed_s09 = lane_slices(&malformed_lanes);
    let malformed_ready_audio =
        ReceiverS09InputView::from_post_lfe_stage(&parent, CallbackPhase::Normal, malformed_s09);
    assert!(matches!(
        malformed_ready_audio,
        Err(ReceiverMode1Error::Input)
    ));
}
