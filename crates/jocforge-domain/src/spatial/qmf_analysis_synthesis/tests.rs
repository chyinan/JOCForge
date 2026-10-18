// pattern: Functional Core

use super::*;
use crate::frontend::{PackageManifestEvidence, ProviderFileEvidence};
use crate::spatial::{
    F04ParameterProviderError, QmfMathematicalFftProvider, QmfReceiverFunctionalProvider,
    ValidatedF04ParameterPackageManifest, validate_f04_parameter_package_manifest,
    validate_qmf_mathematical_fft_provider, validate_qmf_receiver_functional_provider,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::Arc;

const MANIFEST_BYTES: &[u8] = include_bytes!("../../../tests/fixtures/f01e_provider/MANIFEST.json");
const QMF_FUNCTIONAL_BYTES: &[u8] =
    include_bytes!("../../../tests/fixtures/f04_parameters/qmf-receiver-functional-v1.json");
const QMF_FFT_BYTES: &[u8] =
    include_bytes!("../../../tests/fixtures/f04_parameters/qmf-mathematical-fft-v1.json");
const ANALYSIS_INPUTS: &[u8] =
    include_bytes!("../../../tests/fixtures/qmf_direct/analysis/F04Q2-ANALYSIS-DIRECT-INPUTS.json");
const ANALYSIS_ACTUAL: &[u8] =
    include_bytes!("../../../tests/fixtures/qmf_direct/analysis/F04Q2-ANALYSIS-DIRECT-ACTUAL.json");
const SYNTHESIS_LITERALS: &[u8] = include_bytes!(
    "../../../tests/fixtures/qmf_direct/synthesis/F04Q5-SYNTHESIS-DIRECT-LITERALS.json"
);

fn validated_manifest() -> ValidatedF04ParameterPackageManifest {
    let evidence = PackageManifestEvidence::new(
        Arc::<[u8]>::from(MANIFEST_BYTES),
        Some("shared repository manifest fixture".to_owned()),
    );
    validate_f04_parameter_package_manifest(&evidence)
        .expect("the shared manifest validates the QMF provider entries")
}

fn validated_providers() -> (QmfReceiverFunctionalProvider, QmfMathematicalFftProvider) {
    let manifest = validated_manifest();
    let qmf_functional = validate_qmf_receiver_functional_provider(
        &ProviderFileEvidence::new(
            Arc::<[u8]>::from(QMF_FUNCTIONAL_BYTES),
            Some("approved QMF functional provider fixture".to_owned()),
        ),
        &manifest,
    )
    .expect("the identity-matched QMF functional provider validates");
    let qmf_fft = validate_qmf_mathematical_fft_provider(
        &ProviderFileEvidence::new(
            Arc::<[u8]>::from(QMF_FFT_BYTES),
            Some("approved QMF mathematical FFT provider fixture".to_owned()),
        ),
        &manifest,
    )
    .expect("the identity-matched QMF FFT provider validates");
    (qmf_functional, qmf_fft)
}

fn parse(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("the pinned vector JSON parses")
}

fn case<'a>(document: &'a Value, id: &str) -> &'a Value {
    document["cases"]
        .as_array()
        .expect("vector document has cases")
        .iter()
        .find(|candidate| candidate["case_id"].as_str() == Some(id))
        .unwrap_or_else(|| panic!("missing case {id}"))
}

fn words(value: &Value) -> Vec<u32> {
    let values = value.get("u32_words").unwrap_or(value);
    values
        .as_array()
        .expect("vector field has U32 words")
        .iter()
        .map(|word| {
            u32::try_from(word.as_u64().expect("U32 word is an unsigned integer"))
                .expect("U32 word is in range")
        })
        .collect()
}

fn word_array<const N: usize>(value: &Value) -> [u32; N] {
    words(value)
        .try_into()
        .unwrap_or_else(|values: Vec<u32>| panic!("expected {N} words, got {}", values.len()))
}

fn word_matrix<const ROWS: usize, const COLS: usize>(value: &Value) -> [[u32; COLS]; ROWS] {
    let flat = words(value);
    assert_eq!(flat.len(), ROWS * COLS);
    let mut matrix = [[0_u32; COLS]; ROWS];
    for (index, word) in flat.into_iter().enumerate() {
        matrix[index / COLS][index % COLS] = word;
    }
    matrix
}

fn assert_words(actual: &[u32], expected: &Value, context: &str) {
    assert_eq!(actual, words(expected).as_slice(), "{context}");
}

fn flattened<const ROWS: usize, const COLS: usize>(words: &[[u32; COLS]; ROWS]) -> Vec<u32> {
    words.iter().flatten().copied().collect()
}

fn source_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn pinned_direct_vector_files_keep_the_washed_identities() {
    assert_eq!(
        source_sha256(ANALYSIS_INPUTS),
        "5a05987ec1d2bd3707234d2d7842f7ad8d6e8e18f9d549266ad2447c1aa0b229"
    );
    assert_eq!(
        source_sha256(ANALYSIS_ACTUAL),
        "24bf73c9d5f27f5bd4938155920edc9933b8d7d967e4d2ef84d4154f8515840f"
    );
    assert_eq!(
        source_sha256(SYNTHESIS_LITERALS),
        "36f4ba70a9840701f56ae2c90e53b325c390de79b8a7e4b616990f558d4f5843"
    );
}

#[test]
fn analysis_direct_literals_assert_every_stage_after_fresh_computation() {
    let (qmf_functional, qmf_fft) = validated_providers();
    let inputs = parse(ANALYSIS_INPUTS);
    let actuals = parse(ANALYSIS_ACTUAL);

    for input_case in inputs["cases"].as_array().expect("analysis cases exist") {
        let case_id = input_case["case_id"].as_str().expect("case id exists");
        let actual_case = case(&actuals, case_id);
        let current_words = word_array::<64>(&input_case["current_64"]);
        let current = current_words.map(f32::from_bits);
        let history_before = word_matrix::<9, 64>(&input_case["history_before_9x64"]);
        let history = QmfAnalysisHistory::from_words(history_before)
            .expect("the vector before-history is finite");

        let trace = analyze_qmf_slot_trace(&current, &history, &qmf_functional, &qmf_fft)
            .expect("the selected lane slot analyzes successfully");

        assert_words(&trace.fold128, &actual_case["fold128"], case_id);
        assert_words(&trace.rotated128, &actual_case["rotated128"], case_id);
        assert_words(
            &trace.fft_provider_input128,
            &actual_case["analysis_fft_provider_input128"],
            case_id,
        );
        assert_words(
            &trace.bitrev_scaled_fft_input128,
            &actual_case["analysis_fft_input128_bitrev_scaled"],
            case_id,
        );
        assert_words(&trace.fft_output128, &actual_case["fft_output128"], case_id);
        assert_words(
            &trace.final_reorder_output128,
            &actual_case["final_reorder_output128"],
            case_id,
        );
        assert_words(
            &flattened(&trace.history_after_9x64),
            &actual_case["history_after_9x64"],
            case_id,
        );
        assert!(
            actual_case["identity_assertions"]["provider_input_wordwise_equals_rotated128"]
                .as_bool()
                .expect("provider-input assertion exists")
        );
        assert!(
            actual_case["identity_assertions"]["history_after_row0_equals_current"]
                .as_bool()
                .expect("history row0 assertion exists")
        );
        assert!(
            actual_case["identity_assertions"]["history_after_rows1_to8_equal_before_rows0_to7"]
                .as_bool()
                .expect("history shift assertion exists")
        );
    }
}

#[test]
fn synthesis_direct_literals_assert_every_stage_after_fresh_computation() {
    let (qmf_functional, qmf_fft) = validated_providers();
    let vectors = parse(SYNTHESIS_LITERALS);

    for case_id in ["SYNTH-COLD-SPARSE-SIGNEDZERO", "SYNTH-WARM-HISTORY-SHIFT"] {
        let vector_case = case(&vectors, case_id);
        let input_words = word_array::<128>(&vector_case["input"]["complex_slot"]);
        let before_history = word_array::<576>(&vector_case["input"]["synthesis_history"]);
        let trace =
            synthesize_qmf_slot_trace(&input_words, &before_history, &qmf_functional, &qmf_fft)
                .expect("the fixed synthesis slot processes successfully");
        let expected = &vector_case["expected"];

        assert_words(
            &trace.synthesis_preformat128,
            &expected["synthesis_preformat"],
            case_id,
        );
        assert_words(
            &trace.inverse_fft_input128,
            &expected["inverse_fft_input"],
            case_id,
        );
        assert_words(
            &trace.inverse_fft_output128,
            &expected["inverse_fft_output"],
            case_id,
        );
        assert_words(
            &trace.postmodulation128,
            &expected["postmodulation"],
            case_id,
        );
        assert_words(
            &trace.polyphase_output64,
            &expected["polyphase_output"],
            case_id,
        );
        assert_words(
            &trace.next_synthesis_history576,
            &expected["next_synthesis_history"],
            case_id,
        );
    }
}

#[test]
fn receiver_mode1_direct_literals_assert_phase_slots_and_callback_after_state() {
    let (qmf_functional, qmf_fft) = validated_providers();
    let vectors = parse(SYNTHESIS_LITERALS);
    let vector_case = case(&vectors, "MODE1-COLD-24-BAND0-FIR");
    let input = &vector_case["input"];
    assert_eq!(input["mode"].as_u64(), Some(1));

    let analysis_slots = word_matrix::<24, 128>(&input["complex_slots"]);
    let delay_before = word_matrix::<10, 128>(&input["delay_history"]);
    let fir_before = word_matrix::<20, 2>(&input["fir_history"]);
    let synthesis_before = word_array::<576>(&input["synthesis_history"]);
    let before = QmfReceiverMode1Histories::from_words(delay_before, fir_before, synthesis_before)
        .expect("the vector mode=1 before histories are finite");
    let phase = qmf_mode1_delay_fir_trace(&analysis_slots, &before, &qmf_functional)
        .expect("the fixed mode=1 phase and FIR transition succeeds");
    let expected = &vector_case["expected"];

    assert_words(
        &flattened(&phase.phase_output24x128),
        &expected["mode1_phase_output"],
        "mode1 phase output",
    );
    assert_words(
        &flattened(&phase.next_delay_history10x128),
        &expected["next_delay_history"],
        "mode1 next delay history",
    );
    assert_words(
        &flattened(&phase.next_fir_history20x2),
        &expected["next_fir_history"],
        "mode1 next FIR history",
    );

    let mut synthesis_history = synthesis_before;
    let expected_slot_stages = expected["slot_stage_literals"]
        .as_array()
        .expect("the vector has a literal for each QMF slot");
    assert_eq!(expected_slot_stages.len(), 24);
    let mut pcm_slots = [[0.0_f32; 64]; 24];
    for (slot, pcm_slot) in pcm_slots.iter_mut().enumerate() {
        let trace = synthesize_qmf_slot_trace(
            &phase.phase_output24x128[slot],
            &synthesis_history,
            &qmf_functional,
            &qmf_fft,
        )
        .expect("each fixed mode=1 slot synthesizes");
        let expected_slot = &expected_slot_stages[slot];
        assert_words(
            &trace.synthesis_preformat128,
            &expected_slot["synthesis_preformat"],
            "mode1 synthesis preformat",
        );
        assert_words(
            &trace.inverse_fft_input128,
            &expected_slot["inverse_fft_input"],
            "mode1 inverse FFT input",
        );
        assert_words(
            &trace.inverse_fft_output128,
            &expected_slot["inverse_fft_output"],
            "mode1 inverse FFT output",
        );
        assert_words(
            &trace.postmodulation128,
            &expected_slot["postmodulation"],
            "mode1 postmodulation",
        );
        assert_words(
            &trace.polyphase_output64,
            &expected_slot["polyphase_output"],
            "mode1 polyphase output",
        );
        assert_words(
            &trace.next_synthesis_history576,
            &expected_slot["next_synthesis_history"],
            "mode1 synthesis history after slot",
        );
        for (sample, pcm_sample) in pcm_slot.iter_mut().enumerate() {
            *pcm_sample = f32::from_bits(trace.polyphase_output64[sample]);
        }
        synthesis_history = trace.next_synthesis_history576;
    }
    assert_words(
        &pcm_slots
            .iter()
            .flatten()
            .map(|sample| sample.to_bits())
            .collect::<Vec<_>>(),
        &expected["polyphase_output_1536"],
        "mode1 callback PCM",
    );
    assert_words(
        &synthesis_history,
        &expected["next_synthesis_history"],
        "mode1 callback synthesis after-state",
    );

    let result =
        process_qmf_receiver_mode1_callback(&analysis_slots, &before, &qmf_functional, &qmf_fft)
            .expect("the public callback core returns only a complete after-result");
    assert_eq!(
        result
            .pcm_slots
            .iter()
            .flatten()
            .map(|sample| sample.to_bits())
            .collect::<Vec<_>>(),
        pcm_slots
            .iter()
            .flatten()
            .map(|sample| sample.to_bits())
            .collect::<Vec<_>>()
    );
    assert_eq!(result.history_after.delay, phase.next_delay_history10x128);
    assert_eq!(result.history_after.band0_fir, phase.next_fir_history20x2);
    assert_eq!(result.history_after.synthesis, synthesis_history);
    assert_eq!(before.delay, delay_before);
    assert_eq!(before.band0_fir, fir_before);
    assert_eq!(before.synthesis, synthesis_before);
}

#[test]
fn analysis_rejects_nonfinite_pcm_and_mode1_rejects_nonfinite_bands() {
    let (qmf_functional, qmf_fft) = validated_providers();
    let history = QmfAnalysisHistory::cold();
    let mut samples = [0.0_f32; 64];
    samples[37] = f32::INFINITY;
    assert_eq!(
        analyze_qmf_slot(&samples, &history, &qmf_functional, &qmf_fft).unwrap_err(),
        QmfAnalysisSynthesisError::InvalidInput
    );

    let mut slots = [[0_u32; 128]; 24];
    slots[7][17] = 0x7f80_0000;
    let before = QmfReceiverMode1Histories::cold();
    assert_eq!(
        process_qmf_receiver_mode1_callback(&slots, &before, &qmf_functional, &qmf_fft)
            .unwrap_err(),
        QmfAnalysisSynthesisError::InvalidInput
    );
}

#[test]
fn finite_inputs_that_overflow_a_qmf_stage_fail_without_a_result() {
    let (qmf_functional, qmf_fft) = validated_providers();
    let maximum = f32::MAX.to_bits();
    let bands = [maximum; 128];
    let history = [0_u32; 576];

    assert!(matches!(
        synthesize_qmf_slot_trace(&bands, &history, &qmf_functional, &qmf_fft),
        Err(QmfAnalysisSynthesisError::NumericFailure)
    ));
}

#[test]
fn typed_provider_identity_failure_stays_at_the_provider_boundary() {
    let manifest = validated_manifest();
    let result = validate_qmf_receiver_functional_provider(
        &ProviderFileEvidence::new(
            Arc::<[u8]>::from(MANIFEST_BYTES),
            Some("crosswired QMF bytes".to_owned()),
        ),
        &manifest,
    );
    assert!(matches!(
        result,
        Err(F04ParameterProviderError::ProviderIdentityMismatch)
    ));
}
