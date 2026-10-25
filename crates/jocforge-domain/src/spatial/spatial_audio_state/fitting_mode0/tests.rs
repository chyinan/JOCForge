// pattern: Imperative Shell

use super::*;
use crate::{
    frontend::{PackageManifestEvidence, ProviderFileEvidence},
    profile::EncodingProfile,
    spatial::source_flags::SourceFlagsAttempt,
    spatial::{
        DescriptorMathematicalProvider, QmfMathematicalFftProvider, QmfReceiverFunctionalProvider,
        post_window_fanout::CallbackPhase,
        qmf_analysis_synthesis::{QmfAnalysisHistory, analyze_qmf_slot},
        spatial_audio_state::{
            F04StateEncoder, QmfWindowState, RawBytes, SpatialAudioJoinMetadata,
            SpatialAudioMachine,
        },
        validate_descriptor_mathematical_provider, validate_f04_parameter_package_manifest,
        validate_qmf_mathematical_fft_provider, validate_qmf_receiver_functional_provider,
    },
    timeline::{FrontendCallbackIndex, JocDispatchIndex, ReceiverCallbackCount},
    transaction::CallbackLineageIdentity,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, sync::Arc};

const MANIFEST_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f01e_provider/MANIFEST.json");
const QMF_FUNCTIONAL_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f04_parameters/qmf-receiver-functional-v1.json");
const QMF_FFT_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f04_parameters/qmf-mathematical-fft-v1.json");
const DESCRIPTOR_MATH_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f04_parameters/descriptor-mathematical.json");

type Providers = (
    QmfReceiverFunctionalProvider,
    QmfMathematicalFftProvider,
    DescriptorMathematicalProvider,
);

fn providers() -> Providers {
    let manifest = validate_f04_parameter_package_manifest(&PackageManifestEvidence::new(
        Arc::<[u8]>::from(MANIFEST_BYTES),
        Some("shared package manifest fixture".to_owned()),
    ))
    .expect("the tracked package manifest validates");
    let functional = validate_qmf_receiver_functional_provider(
        &ProviderFileEvidence::new(
            Arc::<[u8]>::from(QMF_FUNCTIONAL_BYTES),
            Some("approved QMF functional fixture".to_owned()),
        ),
        &manifest,
    )
    .expect("the approved QMF functional provider validates");
    let fft = validate_qmf_mathematical_fft_provider(
        &ProviderFileEvidence::new(
            Arc::<[u8]>::from(QMF_FFT_BYTES),
            Some("approved QMF FFT fixture".to_owned()),
        ),
        &manifest,
    )
    .expect("the approved QMF FFT provider validates");
    let descriptor = validate_descriptor_mathematical_provider(
        &ProviderFileEvidence::new(
            Arc::<[u8]>::from(DESCRIPTOR_MATH_BYTES),
            Some("approved PG03 descriptor fixture".to_owned()),
        ),
        &manifest,
    )
    .expect("the approved descriptor gain provider validates");
    (functional, fft, descriptor)
}

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/mode0_fitting")
        .join(name);
    serde_json::from_slice(&fs::read(path).expect("mode0 fixture is readable"))
        .expect("mode0 fixture JSON parses")
}

fn case<'a>(document: &'a Value, id: &str) -> &'a Value {
    document["cases"]
        .as_array()
        .expect("fixture contains cases")
        .iter()
        .find(|item| item["case_id"].as_str() == Some(id))
        .unwrap_or_else(|| panic!("missing mode0 case {id}"))
}

fn parent(g: u64) -> SpatialAudioJoinMetadata {
    let profile = EncodingProfile::selected();
    let state = SpatialAudioMachine::new_cold(profile).expect("selected parent profile");
    let programme_id = crate::transaction::test_programme_id("f04rb-mode0-direct-test");
    let attempt = SourceFlagsAttempt::new(7);
    let callback_index = FrontendCallbackIndex::new(u32::try_from(g).unwrap());
    SpatialAudioJoinMetadata::new(
        programme_id.clone(),
        attempt,
        CallbackLineageIdentity::new(programme_id, attempt, callback_index),
        callback_index,
        g,
        state.digest(),
        profile,
    )
}

fn recipe(recipe: &Value, row: usize, sample: usize) -> f32 {
    let seed = recipe["seed"].as_u64().expect("recipe seed") as usize;
    match recipe["kind"].as_str().expect("recipe kind") {
        "SPARSE" => {
            let impulse = (row * 37 + seed * 11) % 1_536;
            if sample == impulse {
                (i32::try_from(row).expect("small input row") - 7) as f32 / 64.0
            } else if (sample + row + seed) % 509 == 0 {
                -0.0
            } else {
                0.0
            }
        }
        "DYADIC" => {
            let numerator = (sample * (row + 3) + 17 * row + seed * 23) % 257;
            (i32::try_from(numerator).expect("bounded dyadic numerator") - 128) as f32 / 8_192.0
        }
        other => panic!("unsupported input recipe {other}"),
    }
}

fn make_rows(recipe_value: &Value, rows: usize) -> Vec<Vec<f32>> {
    (0..rows)
        .map(|row| {
            (0..1_536)
                .map(|sample| recipe(recipe_value, row, sample))
                .collect()
        })
        .collect()
}

fn raw_matrix_to_words<const ROWS: usize, const ROWS_OF_HISTORY: usize>(
    matrix: &[[RawBytes; ROWS_OF_HISTORY]; ROWS],
) -> Vec<u32> {
    matrix
        .iter()
        .flat_map(|row| row.iter())
        .flat_map(|history| {
            history
                .bytes
                .chunks_exact(4)
                .map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
        })
        .collect()
}

fn history_from_raw(rows: &[RawBytes; 9]) -> QmfAnalysisHistory {
    let mut words = [[0_u32; 64]; 9];
    for (row, raw) in rows.iter().enumerate() {
        for (sample, bytes) in raw.bytes.chunks_exact(4).enumerate() {
            words[row][sample] = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        }
    }
    QmfAnalysisHistory::from_words(words).expect("receiver analysis history is finite")
}

fn histories_from_receiver(
    before: &[[RawBytes; 9]; 5],
    s09_rows: &[Vec<f32>],
    functional: &QmfReceiverFunctionalProvider,
    fft: &QmfMathematicalFftProvider,
) -> [[RawBytes; 9]; 5] {
    let lanes = [0_usize, 1, 2, 4, 5];
    let mut after = crate::spatial::spatial_audio_state::zero_raw_matrix::<5, 9>(256);
    for output_lane in 0..5 {
        let mut history = history_from_raw(&before[output_lane]);
        for slot in 0..24 {
            let mut samples = [0.0_f32; 64];
            for (sample, value) in samples.iter_mut().enumerate() {
                *value = s09_rows[lanes[output_lane]][slot * 64 + sample];
            }
            let result = analyze_qmf_slot(&samples, &history, functional, fft)
                .expect("independent receiver analysis succeeds");
            history = QmfAnalysisHistory::from_words(*result.history_after().rows())
                .expect("analysis after remains finite");
        }
        for (row, words) in history.rows().iter().enumerate() {
            for (sample, word) in words.iter().copied().enumerate() {
                let start = sample * 4;
                after[output_lane][row].bytes[start..start + 4]
                    .copy_from_slice(&word.to_le_bytes());
            }
        }
    }
    after
}

fn assert_literal(actual: &[u32], literal: &Value, field: &str) {
    let expected = literal[field]["u32_words"]
        .as_array()
        .unwrap_or_else(|| panic!("{field} has U32 words"));
    assert_eq!(actual.len(), expected.len(), "{field} word count");
    for (index, (actual_word, expected_word)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(
            *actual_word,
            u32::try_from(expected_word.as_u64().expect("literal U32")).expect("literal fits U32"),
            "{field}[{index}]"
        );
    }
    let mut bytes = Vec::with_capacity(actual.len() * 4);
    for word in actual {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    assert_eq!(
        format!("{:x}", Sha256::digest(bytes)),
        literal[field]["f32le_sha256"]
            .as_str()
            .expect("literal SHA"),
        "{field} F32LE SHA-256"
    );
}

fn assert_state_literal(actual: &[u32], expected: &Value, field: &str) {
    let expected_words = expected["u32_words"]
        .as_array()
        .unwrap_or_else(|| panic!("after-state {field} contains U32 words"));
    assert_eq!(
        actual.len(),
        expected_words.len(),
        "after-state {field} word count"
    );
    for (index, (actual_word, expected_word)) in actual.iter().zip(expected_words).enumerate() {
        assert_eq!(
            *actual_word,
            u32::try_from(expected_word.as_u64().expect("after-state U32"))
                .expect("after-state word fits U32"),
            "after-state {field}[{index}]"
        );
    }
    let mut bytes = Vec::with_capacity(actual.len() * 4);
    for word in actual {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    assert_eq!(
        bytes.len(),
        expected["f32le_bytes"].as_u64().unwrap() as usize
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(bytes)),
        expected["f32le_sha256"]
            .as_str()
            .expect("after-state F32LE SHA"),
        "after-state {field} F32LE SHA-256"
    );
}

fn raw_words<'a>(buffers: impl IntoIterator<Item = &'a RawBytes>) -> Vec<u32> {
    buffers
        .into_iter()
        .flat_map(|buffer| {
            buffer
                .bytes
                .chunks_exact(4)
                .map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
        })
        .collect()
}

fn canonical_after(state: &QmfWindowState) -> Vec<u8> {
    let mut encoder = F04StateEncoder::with_capacity(421_039);
    state
        .encode(&mut encoder)
        .expect("QmfWindowState canonical encoding");
    encoder.finish()
}

fn stage_success_case(
    input_case: &Value,
    fitting_before: &QmfWindowState,
    receiver_before: &[[RawBytes; 9]; 5],
    provider_set: &Providers,
    s09_rows: &[Vec<f32>],
    target_rows: &[Vec<f32>],
) -> Mode0FittingProposal {
    let g = input_case["clock"]["frontend_g"]
        .as_u64()
        .expect("frontend g");
    let r = input_case["clock"]["receiver_r"]
        .as_u64()
        .expect("receiver r");
    let generation = input_case["clock"]["receiver_generation"]
        .as_u64()
        .expect("receiver generation");
    let phase = match input_case["clock"]["phase"].as_str() {
        Some("NORMAL") => CallbackPhase::Normal,
        Some("CACHED_DRAIN") => CallbackPhase::CachedDrain,
        other => panic!("unexpected callback phase {other:?}"),
    };
    let parent = parent(g);
    let s09_lanes = s09_rows.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let target_lanes = target_rows.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let expected_receiver_after =
        histories_from_receiver(receiver_before, s09_rows, &provider_set.0, &provider_set.1);
    let descriptor = &input_case["descriptor_after"];
    let input = Mode0FittingInput {
        expected_parent: &parent,
        expected_phase: phase,
        fitting_before,
        fitting_before_generation: ReceiverCallbackCount::new(r),
        receiver_analysis_before: receiver_before,
        receiver_analysis_before_generation: ReceiverCallbackCount::new(r),
        receiver_analysis_after: ReceiverAnalysisAfterView {
            parent: &parent,
            phase,
            generation: ReceiverCallbackCount::new(generation),
            histories: &expected_receiver_after,
        },
        s09: S09InputView {
            parent: &parent,
            phase,
            lanes: &s09_lanes,
        },
        targets: TargetInputView {
            parent: &parent,
            phase,
            rows: &target_lanes,
        },
        descriptor_after: DescriptorAfterView {
            parent: &parent,
            phase,
            generation: ReceiverCallbackCount::new(generation),
            selected_index: descriptor["selected_index"]
                .as_u64()
                .expect("descriptor selected index") as u8,
            target_gain_u32: u32::try_from(
                descriptor["target_gain_u32"]
                    .as_u64()
                    .expect("descriptor gain"),
            )
            .expect("descriptor gain fits U32"),
        },
    };
    stage_mode0_fitting(
        input,
        Mode0FittingProviderInput::Ready {
            functional: &provider_set.0,
            fft: &provider_set.1,
            descriptor: &provider_set.2,
        },
    )
    .expect("direct mode0 fitting succeeds")
}

#[test]
fn three_generation_actual_first_kats_match_every_stage_and_complete_after_state() {
    let inputs = fixture("input-vectors.json");
    let literals = fixture("actual-literals.json");
    let provider_set = providers();
    let mut fitting_before = QmfWindowState::cold();
    let mut receiver_before = crate::spatial::spatial_audio_state::zero_raw_matrix::<5, 9>(256);

    for id in ["cold_normal_g2", "warm_normal_g3", "cached_drain_g4"] {
        let input_case = case(&inputs, id);
        let literal = case(&literals, id);
        let s09_rows = make_rows(&input_case["s09_input_recipe"], 6);
        let target_rows = make_rows(&input_case["target_preconditioned_recipe"], 15);
        let proposal = stage_success_case(
            input_case,
            &fitting_before,
            &receiver_before,
            &provider_set,
            &s09_rows,
            &target_rows,
        );

        assert_eq!(
            proposal.receiver_index,
            ReceiverCallbackCount::new(input_case["clock"]["receiver_r"].as_u64().unwrap())
        );
        assert_eq!(
            proposal.dispatch,
            JocDispatchIndex::new(input_case["clock"]["joc_dispatch_d"].as_u64().unwrap())
        );
        assert_eq!(
            u64::from(proposal.clip_index),
            literal["clip_index"].as_u64().unwrap()
        );
        assert_literal(
            &proposal.target_temporal_output,
            literal,
            "target_temporal_output",
        );
        assert_literal(
            &proposal.target_raw_analysis,
            literal,
            "target_raw_analysis",
        );
        assert_literal(&proposal.target_windows48, literal, "target_windows48");
        assert_literal(
            &proposal.carrier_raw_analysis,
            literal,
            "carrier_raw_analysis",
        );
        assert_literal(
            &proposal.carrier_mode0_current24,
            literal,
            "carrier_mode0_current24",
        );
        assert_literal(&proposal.carrier_windows48, literal, "carrier_windows48");
        assert_literal(
            &raw_matrix_to_words(&proposal.receiver_analysis_after_join),
            literal,
            "receiver_analysis_after_join",
        );

        let expected_after = &literal["after_state"];
        assert_eq!(
            proposal.after_state.target_temporal.span,
            expected_after["target_temporal"]["span"].as_i64().unwrap()
        );
        assert_eq!(
            proposal.after_state.target_temporal.index_28,
            expected_after["target_temporal"]["read"].as_i64().unwrap()
        );
        assert_eq!(
            proposal.after_state.target_temporal.index_32,
            expected_after["target_temporal"]["write"].as_i64().unwrap()
        );
        assert_state_literal(
            &raw_words(proposal.after_state.target_temporal.buffers.iter()),
            &expected_after["target_temporal"]["buffers"],
            "target_temporal.buffers",
        );
        assert_state_literal(
            &raw_words(
                proposal
                    .after_state
                    .target_history
                    .iter()
                    .flat_map(|row| row.iter()),
            ),
            &expected_after["target_history"],
            "target_history",
        );
        assert_state_literal(
            &raw_words(
                proposal
                    .after_state
                    .previous_targets
                    .iter()
                    .flat_map(|row| row.iter()),
            ),
            &expected_after["previous_targets"],
            "previous_targets",
        );
        assert_state_literal(
            &raw_words(
                proposal
                    .after_state
                    .previous_carriers
                    .iter()
                    .flat_map(|row| row.iter()),
            ),
            &expected_after["previous_carriers"],
            "previous_carriers",
        );
        assert_state_literal(
            &raw_words(
                proposal
                    .after_state
                    .fitting_delay
                    .iter()
                    .flat_map(|row| row.iter()),
            ),
            &expected_after["fitting_delay"],
            "fitting_delay",
        );
        assert_state_literal(
            &raw_words(proposal.after_state.fitting_fir.iter()),
            &expected_after["fitting_fir"],
            "fitting_fir",
        );

        let canonical = canonical_after(&proposal.after_state);
        assert_eq!(
            canonical.len(),
            literal["canonical_after"]["bytes"].as_u64().unwrap() as usize
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(&canonical)),
            literal["canonical_after"]["sha256"].as_str().unwrap()
        );

        receiver_before = proposal.receiver_analysis_after_join;
        fitting_before = proposal.after_state;
    }

    let warm = case(&inputs, "warm_normal_g3");
    let drain = case(&inputs, "cached_drain_g4");
    assert_eq!(warm["s09_input_recipe"], drain["s09_input_recipe"]);
    assert_eq!(
        warm["target_preconditioned_recipe"],
        drain["target_preconditioned_recipe"]
    );
    assert_eq!(drain["clock"]["phase"], "CACHED_DRAIN");
}

#[test]
fn all_nine_input_only_errors_are_fail_stop_with_the_mapped_code() {
    let inputs = fixture("input-vectors.json");
    let errors = inputs["error_cases"].as_array().expect("error vectors");
    assert_eq!(errors.len(), 9);
    let expected = [
        (
            "provider_missing_precedes_stale",
            "RECEIVER_PARAMETER_UNAVAILABLE",
        ),
        ("provider_identity_invalid", "RECEIVER_PARAMETER_INVALID"),
        ("stale_lineage", "F04RB_LINEAGE_ERROR"),
        (
            "descriptor_projection_mismatch",
            "F04RB_DESCRIPTOR_PROJECTION",
        ),
        ("state_delay_shape", "F04RB_MODE0_STATE_SHAPE"),
        ("s09_shape", "F04RB_INPUT_SHAPE"),
        ("target_shape", "F04RB_TARGET_TEMPORAL_SHAPE"),
        ("nonfinite_input", "F04RB_INPUT_NONFINITE"),
        ("finite_numeric_overflow", "F04RB_NUMERIC_NONFINITE"),
    ];
    assert_eq!(
        expected.len(),
        errors.len(),
        "every mapped error has one test outcome"
    );
    for (vector, (id, code)) in errors.iter().zip(expected) {
        assert_eq!(vector["id"], id);
        let error = run_error_case(id);
        assert_eq!(error.code(), code, "{id}");
    }
    assert_eq!(
        run_error_case("nonfinite_lfe_input").code(),
        "F04RB_INPUT_NONFINITE"
    );
    assert_eq!(
        run_error_case("receiver_analysis_after_join_mismatch").code(),
        "F04RB_LINEAGE_ERROR"
    );
}

fn run_error_case(id: &str) -> Mode0FittingError {
    let input_vectors = fixture("input-vectors.json");
    let base = case(&input_vectors, "warm_normal_g3");
    let provider_set = providers();
    let g = 3;
    let r = 1;
    let phase = CallbackPhase::Normal;
    let parent = parent(g);
    let mut fitting_before = QmfWindowState::cold();
    let receiver_before = crate::spatial::spatial_audio_state::zero_raw_matrix::<5, 9>(256);
    let mut s09_rows = make_rows(&base["s09_input_recipe"], 6);
    let mut receiver_after = histories_from_receiver(
        &receiver_before,
        &s09_rows,
        &provider_set.0,
        &provider_set.1,
    );
    let target_rows = make_rows(&base["target_preconditioned_recipe"], 15);
    if id == "nonfinite_input" {
        s09_rows[0][0] = f32::from_bits(2_143_289_344);
    }
    if id == "nonfinite_lfe_input" {
        s09_rows[3][0] = f32::from_bits(2_143_289_344);
    }
    if id == "receiver_analysis_after_join_mismatch" {
        receiver_after[0][0].bytes[0] ^= 1;
    }
    if id == "finite_numeric_overflow" {
        s09_rows[0].fill(f32::from_bits(2_139_095_039));
    }
    let mut s09_lanes = s09_rows.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let mut target_lanes = target_rows.iter().map(Vec::as_slice).collect::<Vec<_>>();
    if id == "s09_shape" {
        s09_lanes[0] = &s09_rows[0][..1_535];
    }
    if id == "target_shape" {
        target_lanes.pop();
    }
    let descriptor = &base["descriptor_after"];
    let mut fitting_generation = ReceiverCallbackCount::new(r);
    let mut gain = u32::try_from(descriptor["target_gain_u32"].as_u64().unwrap()).unwrap();
    match id {
        "provider_missing_precedes_stale" | "provider_identity_invalid" => {
            fitting_generation = ReceiverCallbackCount::new(99);
        }
        "stale_lineage" => fitting_generation = ReceiverCallbackCount::new(0),
        "descriptor_projection_mismatch" => gain = 0,
        "state_delay_shape" => {
            fitting_before.fitting_delay[0][0].bytes = vec![0_u8; 1].into_boxed_slice();
        }
        _ => {}
    }
    let input = Mode0FittingInput {
        expected_parent: &parent,
        expected_phase: phase,
        fitting_before: &fitting_before,
        fitting_before_generation: fitting_generation,
        receiver_analysis_before: &receiver_before,
        receiver_analysis_before_generation: ReceiverCallbackCount::new(r),
        receiver_analysis_after: ReceiverAnalysisAfterView {
            parent: &parent,
            phase,
            generation: ReceiverCallbackCount::new(2),
            histories: &receiver_after,
        },
        s09: S09InputView {
            parent: &parent,
            phase,
            lanes: &s09_lanes,
        },
        targets: TargetInputView {
            parent: &parent,
            phase,
            rows: &target_lanes,
        },
        descriptor_after: DescriptorAfterView {
            parent: &parent,
            phase,
            generation: ReceiverCallbackCount::new(2),
            selected_index: descriptor["selected_index"].as_u64().unwrap() as u8,
            target_gain_u32: gain,
        },
    };
    let provider_input = match id {
        "provider_missing_precedes_stale" => Mode0FittingProviderInput::Missing,
        "provider_identity_invalid" => Mode0FittingProviderInput::Invalid,
        _ => Mode0FittingProviderInput::Ready {
            functional: &provider_set.0,
            fft: &provider_set.1,
            descriptor: &provider_set.2,
        },
    };
    match stage_mode0_fitting(input, provider_input) {
        Err(error) => error,
        Ok(_) => panic!("{id} must fail without a proposal"),
    }
}
