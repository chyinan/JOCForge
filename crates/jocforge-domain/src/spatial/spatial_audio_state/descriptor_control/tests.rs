// pattern: Imperative Shell
use super::{
    DescriptorControlInputView, DescriptorGainInputView, DescriptorParameterInput,
    apply_descriptor_gain, stage_descriptor_control,
};
use crate::{
    frontend::{PackageManifestEvidence, ProviderFileEvidence},
    profile::EncodingProfile,
    spatial::{
        DescriptorMathematicalProvider, QmfReceiverFunctionalProvider,
        post_window_fanout::CallbackPhase,
        source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
        spatial_audio_state::{
            DescriptorState, RawWord, SpatialAudioJoinMetadata, SpatialAudioMachine,
        },
        validate_descriptor_mathematical_provider, validate_f04_parameter_package_manifest,
        validate_qmf_receiver_functional_provider,
    },
    timeline::FrontendCallbackIndex,
    transaction::CallbackTransactionId,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fmt::Write as FmtWrite, sync::Arc};

const MANIFEST_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f01e_provider/MANIFEST.json");
const QMF_RECEIVER_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f04_parameters/qmf-receiver-functional-v1.json");
const DESCRIPTOR_MATH_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/f04_parameters/descriptor-mathematical.json");
const DIRECT_INPUT_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/descriptor_control/input-vectors.json");
const DIRECT_ACTUAL_BYTES: &[u8] =
    include_bytes!("../../../../tests/fixtures/descriptor_control/actual-literals.json");

fn document(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("identity-pinned descriptor fixture parses")
}

fn case<'a>(document: &'a Value, case_id: &str) -> &'a Value {
    document["cases"]
        .as_array()
        .expect("case list")
        .iter()
        .find(|case| case["case_id"] == case_id)
        .expect("case id exists")
}

fn providers() -> (
    QmfReceiverFunctionalProvider,
    DescriptorMathematicalProvider,
) {
    let manifest_evidence = PackageManifestEvidence::new(Arc::from(MANIFEST_BYTES), None);
    let manifest = validate_f04_parameter_package_manifest(&manifest_evidence)
        .expect("shared package manifest validates");
    let receiver = validate_qmf_receiver_functional_provider(
        &ProviderFileEvidence::new(Arc::from(QMF_RECEIVER_BYTES), None),
        &manifest,
    )
    .expect("selected receiver functional provider validates");
    let math = validate_descriptor_mathematical_provider(
        &ProviderFileEvidence::new(Arc::from(DESCRIPTOR_MATH_BYTES), None),
        &manifest,
    )
    .expect("PG03 mathematical provider validates");
    (receiver, math)
}

fn parent() -> SpatialAudioJoinMetadata {
    let profile = EncodingProfile::selected();
    let state = SpatialAudioMachine::new_cold(profile).expect("selected spatial-audio root");
    SpatialAudioJoinMetadata::new(
        SourceFlagsProgrammeId::new("f04d-direct-test").expect("test programme id"),
        SourceFlagsAttempt::new(7),
        CallbackTransactionId::new("f04d-direct-transaction").expect("test transaction id"),
        FrontendCallbackIndex::new(2),
        state.generation(),
        state.digest(),
        profile,
    )
}

fn word(value: &Value) -> u32 {
    u32::try_from(value.as_u64().expect("raw F32 U32 word")).expect("F32 word fits U32")
}

fn state(input: &Value) -> DescriptorState {
    let before = &input["before_state"];
    DescriptorState {
        tail_peak: RawWord(word(&before["tail_peak_u32"]).to_le_bytes()),
        hold_count: before["hold_u32"].as_i64().expect("semantic hold value"),
        smoothed: RawWord(word(&before["smoothed_u32"]).to_le_bytes()),
        exponent: before["exponent_u32"].as_i64().expect("exponent value"),
        reference: RawWord(word(&before["reference_u32"]).to_le_bytes()),
        gain: RawWord(word(&before["gain_u32"]).to_le_bytes()),
    }
}

fn lane_words_to_f32(value: &Value) -> Vec<Vec<f32>> {
    value["u32_words"]
        .as_array()
        .expect("six lane word arrays")
        .iter()
        .map(|lane| {
            lane.as_array()
                .expect("lane word array")
                .iter()
                .map(|word_value| f32::from_bits(word(word_value)))
                .collect()
        })
        .collect()
}

fn lane_slices(lanes: &[Vec<f32>]) -> Vec<&[f32]> {
    lanes.iter().map(Vec::as_slice).collect()
}

fn words_sha256(lanes: impl IntoIterator<Item = u32>) -> String {
    let mut bytes = Vec::new();
    for word in lanes {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    let digest = Sha256::digest(&bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
    }
    hex
}

fn assert_control_trace(actual: &Value, computed: &super::DescriptorControlTrace) {
    let expected = &actual["control_trace"];
    assert_eq!(computed.head_u32, word(&expected["head_u32"]));
    assert_eq!(computed.tail_u32, word(&expected["tail_u32"]));
    assert_eq!(computed.peak_u32, word(&expected["peak_u32"]));
    assert_eq!(
        computed.clamped_peak_u32,
        word(&expected["clamped_peak_u32"])
    );
    assert_eq!(
        computed.exponent_estimate,
        expected["exponent_estimate"].as_i64().unwrap()
    );
    assert_eq!(computed.shift, expected["shift"].as_u64().unwrap());
    assert_eq!(
        computed.n_u32,
        expected["n_u32"].as_u64().map(|value| value as u32)
    );
    assert_eq!(
        computed.poly_u32,
        expected["poly_u32"].as_u64().map(|value| value as u32)
    );
    assert_eq!(computed.level_u32, word(&expected["level_u32"]));
    assert_eq!(computed.slope_u32, word(&expected["slope_u32"]));
    assert_eq!(computed.candidate_u32, word(&expected["candidate_u32"]));
    assert_eq!(computed.rise_u32, word(&expected["rise_u32"]));
    assert_eq!(
        computed.hold_branch,
        expected["hold_branch"].as_str().unwrap()
    );
    assert_eq!(computed.lookup_u32, word(&expected["lookup_u32"]));
    assert_eq!(computed.ordinal, expected["ordinal"].as_u64().unwrap());
    assert_eq!(computed.index, expected["index"].as_u64().unwrap());
    assert_eq!(computed.target_gain_u32, word(&expected["target_gain_u32"]));
    let comparisons = expected["comparisons"].as_array().unwrap();
    assert_eq!(computed.comparisons.len(), comparisons.len());
    for (actual, expected) in computed.comparisons.iter().zip(comparisons) {
        assert_eq!(actual.ordinal, expected["ordinal"].as_u64().unwrap());
        assert_eq!(actual.lookup_u32, word(&expected["lookup_u32"]));
        assert_eq!(actual.threshold_u32, word(&expected["threshold_u32"]));
        assert_eq!(
            actual.strict_less,
            expected["strict_less"].as_bool().unwrap()
        );
    }
}

fn assert_ramp_trace(actual: &Value, computed: &super::DescriptorRampTrace) {
    let expected = &actual["ramp_trace"];
    assert_eq!(computed.scale_u32, word(&expected["scale_u32"]));
    assert_eq!(computed.delta_u32, word(&expected["delta_u32"]));
    let current = expected["current_before_each_ramp_sample_u32"]
        .as_array()
        .expect("256 ramp current words");
    assert_eq!(
        computed.current_before_each_ramp_sample_u32.len(),
        current.len()
    );
    for (actual, expected) in computed
        .current_before_each_ramp_sample_u32
        .iter()
        .zip(current)
    {
        assert_eq!(*actual, word(expected));
    }
    assert_eq!(
        computed.current_after_256_adds_u32,
        word(&expected["current_after_256_adds_u32"])
    );
    assert_eq!(computed.tail_alpha_u32, word(&expected["tail_alpha_u32"]));
    assert_eq!(
        computed.committed_gain_u32,
        word(&expected["committed_gain_u32"])
    );
    assert_eq!(
        computed.committed_gain_uses_target_not_current_after_256,
        expected["committed_gain_uses_target_not_current_after_256"]
            .as_bool()
            .unwrap()
    );
}

#[test]
fn selected_provider_direct_cases_match_every_postcomputation_literal() {
    let inputs = document(DIRECT_INPUT_BYTES);
    let literals = document(DIRECT_ACTUAL_BYTES);
    let (receiver, math) = providers();
    assert_eq!(inputs["input_only"].as_bool(), Some(true));
    let success_ids = [
        "cold_zero",
        "head_tail_lfe_excluded",
        "threshold_equal_hold_decrement",
        "lookup_last_ordinal",
        "attack_and_nontrivial_ramp",
        "old_tail_peak_carry_current_tail_zero",
    ];
    assert_eq!(inputs["cases"].as_array().unwrap().len(), success_ids.len());
    assert_eq!(
        literals["cases"].as_array().unwrap().len(),
        success_ids.len()
    );

    for case_id in success_ids {
        let input = case(&inputs, case_id);
        assert_eq!(input["case_id"], case_id);
        let literal_input = &case(&literals, case_id)["input_projection"];
        let control_lanes = lane_words_to_f32(&literal_input["control_scratch"]);
        let ramp_lanes = lane_words_to_f32(&literal_input["ramp_audio"]);
        assert_eq!(
            literal_input["control_scratch"]["f32le_bytes"].as_u64(),
            Some(36_864)
        );
        assert_eq!(
            literal_input["ramp_audio"]["f32le_bytes"].as_u64(),
            Some(36_864)
        );
        assert_eq!(
            words_sha256(control_lanes.iter().flatten().map(|value| value.to_bits())),
            literal_input["control_scratch"]["f32le_sha256"]
        );
        assert_eq!(
            words_sha256(ramp_lanes.iter().flatten().map(|value| value.to_bits())),
            literal_input["ramp_audio"]["f32le_sha256"]
        );
        let control_slices = lane_slices(&control_lanes);
        let ramp_slices = lane_slices(&ramp_lanes);
        let parent = parent();
        let phase = CallbackPhase::Normal;
        let control =
            DescriptorControlInputView::from_f04r_temporal2(&parent, phase, &control_slices);
        let parameters =
            DescriptorParameterInput::from_validated_providers(Some(&receiver), Some(&math));
        let selection = stage_descriptor_control(&state(input), &control, &parameters)
            .expect("direct F04D control stage succeeds");
        let gain_input = DescriptorGainInputView::from_f04r_temporal3(selection, &ramp_slices);
        let proposal =
            apply_descriptor_gain(gain_input).expect("direct F04D gain-ramp stage succeeds");

        let expected = case(&literals, case_id);
        assert_eq!(expected["case_id"], case_id);
        assert_eq!(
            u64::from(proposal.selected_index),
            expected["selected_index"].as_u64().unwrap()
        );
        assert_control_trace(expected, &proposal.control_trace);
        assert_ramp_trace(expected, &proposal.ramp_trace);
        let output_words = proposal
            .s09_return
            .iter()
            .flat_map(|lane| lane.iter().map(|sample| sample.to_bits()))
            .collect::<Vec<_>>();
        let expected_words = expected["ramp_output"]["u32_words"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|lane| lane.as_array().unwrap().iter().map(word))
            .collect::<Vec<_>>();
        assert_eq!(output_words, expected_words);
        assert_eq!(
            expected["ramp_output"]["f32le_bytes"].as_u64(),
            Some(36_864)
        );
        assert_eq!(
            words_sha256(output_words.iter().copied()),
            expected["ramp_output"]["f32le_sha256"]
        );
        assert_eq!(
            proposal.after.tail_peak.0,
            word(&expected["after_state"]["tail_peak_u32"]).to_le_bytes()
        );
        assert_eq!(
            proposal.after.hold_count,
            expected["after_state"]["hold_u32"].as_i64().unwrap()
        );
        assert_eq!(
            proposal.after.smoothed.0,
            word(&expected["after_state"]["smoothed_u32"]).to_le_bytes()
        );
        assert_eq!(
            proposal.after.exponent,
            expected["after_state"]["exponent_u32"].as_i64().unwrap()
        );
        assert_eq!(
            proposal.after.reference.0,
            word(&expected["after_state"]["reference_u32"]).to_le_bytes()
        );
        assert_eq!(
            proposal.after.gain.0,
            word(&expected["after_state"]["gain_u32"]).to_le_bytes()
        );
        let hold_bytes = proposal.after.hold_count.to_le_bytes();
        let mut hold_hex = String::with_capacity(16);
        for byte in hold_bytes {
            write!(&mut hold_hex, "{byte:02x}").expect("writing to String cannot fail");
        }
        assert_eq!(
            hold_hex,
            expected["after_hold_count_i64le_hex"].as_str().unwrap()
        );
    }
}

#[test]
fn direct_error_cases_obey_approved_precedence_and_return_no_proposal() {
    let inputs = document(DIRECT_INPUT_BYTES);
    let (receiver, math) = providers();
    let cold = case(&inputs, "cold_zero");
    let zeros = vec![vec![0.0_f32; 1536]; 6];
    let mut observed = Vec::new();
    for error_case in inputs["error_cases"].as_array().unwrap() {
        let case_id = error_case["case_id"].as_str().unwrap();
        let mutation = &error_case["mutation"];
        let mut before = state(cold);
        if let Some(hold) = mutation.get("hold") {
            before.hold_count = hold.as_i64().unwrap();
        }
        let mut lanes = zeros.clone();
        if let Some(shape) = mutation.get("control_shape").and_then(Value::as_array) {
            let lane_count = shape[0].as_u64().unwrap() as usize;
            let sample_count = shape[1].as_u64().unwrap() as usize;
            lanes.truncate(lane_count);
            for lane in &mut lanes {
                lane.truncate(sample_count);
            }
        }
        if let Some(point) = mutation.get("control_point").and_then(Value::as_array) {
            let lane = point[0].as_u64().unwrap() as usize;
            let sample = point[1].as_u64().unwrap() as usize;
            lanes[lane][sample] = f32::from_bits(word(&point[2]));
        }
        let slices = lane_slices(&lanes);
        let parent = parent();
        let view = DescriptorControlInputView::from_f04r_temporal2(
            &parent,
            CallbackPhase::Normal,
            &slices,
        );
        let mut parameters =
            DescriptorParameterInput::from_validated_providers(Some(&receiver), Some(&math));
        if case_id == "missing_threshold_precedes_bad_input" {
            parameters.selected_threshold_words = None;
        }
        if case_id == "threshold_dimension_precedes_bad_state" {
            parameters.selected_threshold_words =
                Some(&receiver.descriptor_threshold_words()[..126]);
        }
        let altered_gains = mutation.get("gain_u32").map(|gain| {
            let mut values = math.gain_words().to_vec();
            values[mutation["gain_ordinal"].as_u64().unwrap() as usize] = word(gain);
            values
        });
        if let Some(gains) = altered_gains.as_ref() {
            parameters.gain_words = Some(gains);
        }
        let result = super::stage_descriptor_control(&before, &view, &parameters);
        let error = result.expect_err("the approved malformed direct case must fail");
        assert_eq!(
            error.code(),
            error_case["expected"]["error"].as_str().unwrap(),
            "{case_id}"
        );
        observed.push(case_id);
    }
    assert_eq!(observed.len(), 8);
}
