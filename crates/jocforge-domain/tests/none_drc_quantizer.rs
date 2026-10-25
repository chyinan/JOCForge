use jocforge_domain::drc::none_drc_quantizer::{
    NoneDrcQuantizerError, NoneDrcQuantizerKind, PackedControlCode, quantize_none_drc_control,
};

const AUTHORIZED_PARAMETERS: &str = include_str!("fixtures/none_drc/authorized_parameters.json");
const AUTHORIZED_VERIFICATION: &str =
    include_str!("fixtures/none_drc/authorized_verification.json");

#[test]
fn selected_quantizer_parameter_words_match_the_approved_artifact() {
    let parameters: serde_json::Value = serde_json::from_str(AUTHORIZED_PARAMETERS).unwrap();
    let words = parameters["groups"]["DRC_QUANTIZE3"]["f32_words_u32"]
        .as_array()
        .unwrap()
        .iter()
        .map(|word| word.as_u64().unwrap() as u32)
        .collect::<Vec<_>>();

    assert_eq!(words, [0x3E2F_7469, 0x3F2A_DD8F, 0x3F7F_7A01]);
}

#[test]
fn all_approved_quantizer_input_recipes_recompute_before_expected_comparison() {
    let verification: serde_json::Value = serde_json::from_str(AUTHORIZED_VERIFICATION).unwrap();
    assert_eq!(
        verification["expected_outputs_are_never_inputs"],
        serde_json::Value::Bool(true)
    );

    let groups = verification["quantizer_groups"].as_array().unwrap();
    assert_eq!(groups.len(), 2);

    for group in groups {
        let kind = match group["kind"].as_u64().unwrap() {
            0 => NoneDrcQuantizerKind::Rf,
            1 => NoneDrcQuantizerKind::Line,
            other => panic!("unapproved quantizer kind {other}"),
        };
        let input_start = group["q_start"].as_i64().unwrap();
        let input_stop = group["q_stop_exclusive"].as_i64().unwrap();
        let stride = group["q_step"].as_i64().unwrap();
        assert_eq!(group["input_formula"].as_str(), Some("R32(q/32768)"));
        assert_eq!(stride, 17);

        // Build every input and calculate every result before reading the
        // expected-output array, keeping it strictly out of the input path.
        let inputs = (input_start..input_stop)
            .step_by(usize::try_from(stride).unwrap())
            .map(|q| q as f32 / 32_768.0_f32)
            .collect::<Vec<_>>();
        let calculated = inputs
            .iter()
            .map(|&input| {
                quantize_none_drc_control(input, kind)
                    .unwrap()
                    .storage_word()
            })
            .collect::<Vec<_>>();

        let expected = group["expected_packed_u16"].as_array().unwrap();
        assert_eq!(calculated.len(), expected.len());
        for (index, (actual, expected)) in calculated.iter().zip(expected).enumerate() {
            assert_eq!(
                *actual,
                expected.as_u64().unwrap() as u16,
                "kind={} input_index={index}",
                kind.wire_value()
            );
        }
        assert_eq!(calculated.len(), 3_856);
    }
}

#[test]
fn rejects_nonfinite_values_and_unselected_wire_kinds() {
    assert_eq!(
        quantize_none_drc_control(f32::NAN, NoneDrcQuantizerKind::Rf),
        Err(NoneDrcQuantizerError::NonFiniteInput)
    );
    assert_eq!(
        quantize_none_drc_control(f32::INFINITY, NoneDrcQuantizerKind::Line),
        Err(NoneDrcQuantizerError::NonFiniteInput)
    );
    assert_eq!(
        NoneDrcQuantizerKind::try_from(2),
        Err(NoneDrcQuantizerError::UnsupportedKind)
    );
}

#[test]
fn rf_sentinel_keeps_its_full_callee_word_and_u16_storage_word() {
    let mut sentinel: Option<PackedControlCode> = None;
    for q in -32_768_i32..32_768 {
        let input = q as f32 / 32_768.0_f32;
        let output = quantize_none_drc_control(input, NoneDrcQuantizerKind::Rf).unwrap();
        if output.callee_word() == 0xFFFF_FF00 {
            sentinel = Some(output);
            break;
        }
    }

    let output = sentinel.expect("the selected RF domain must exercise its zero sentinel");
    assert_eq!(output.callee_word(), 0xFFFF_FF00);
    assert_eq!(output.storage_word(), 0xFF00);
}

#[test]
fn line_zero_is_not_rewritten_to_the_rf_sentinel() {
    let mut line_zero: Option<PackedControlCode> = None;
    for q in -32_768_i32..32_768 {
        let input = q as f32 / 32_768.0_f32;
        let output = quantize_none_drc_control(input, NoneDrcQuantizerKind::Line).unwrap();
        if output.storage_word() == 0 {
            line_zero = Some(output);
            break;
        }
    }

    let output = line_zero.expect("the selected line domain must include a zero code");
    assert_eq!(output.callee_word(), 0);
    assert_eq!(output.storage_word(), 0);
}
