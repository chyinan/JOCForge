use jocforge_domain::drc::none_drc_normalizer::{
    NoneDrcNormalizationError, normalize_none_drc_metric,
};

const AUTHORIZED_PARAMETERS: &str = include_str!("fixtures/none_drc/authorized_parameters.json");
const AUTHORIZED_VERIFICATION: &str =
    include_str!("fixtures/none_drc/authorized_verification.json");
const PUBLIC_DISTRIBUTION_SCOPE: &str =
    include_str!("fixtures/none_drc/public_distribution_scope_v2/SCOPE.json");

#[test]
fn selected_normalization_parameter_words_match_the_approved_artifact() {
    let parameters: serde_json::Value = serde_json::from_str(AUTHORIZED_PARAMETERS).unwrap();
    let words = parameters["groups"]["DRC_NORMALIZE3"]["f32_words_u32"]
        .as_array()
        .unwrap()
        .iter()
        .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(words, [0x3CAA_5BD7, 0x3C62_2BCA, 0x3BE6_35DE]);

    let distribution_scope: serde_json::Value =
        serde_json::from_str(PUBLIC_DISTRIBUTION_SCOPE).unwrap();
    assert_eq!(
        distribution_scope["decision"].as_str(),
        Some("PUBLIC_DISTRIBUTION_AUTHORIZED_FOR_NONPRIVATE_FUNCTIONAL_AND_MATHEMATICAL_MATERIAL")
    );
    assert_eq!(
        distribution_scope["functional_source_package"]["parameter_file"]["sha256"].as_str(),
        Some("35b28cc27a0bd903cdf58df431c1b2f17769d534425765eb1588573e1fba101f")
    );
    assert_eq!(
        distribution_scope["previous_internal_only_labels"].as_str(),
        Some("SUPERSEDED_FOR_THIS_SCOPE_BY_USER_CLARIFICATION")
    );
    assert_eq!(
        distribution_scope["original_hash_sealed_packages_modified"],
        serde_json::Value::Bool(false)
    );
}

#[test]
fn all_approved_normalized_input_words_recompute_before_expected_comparison() {
    let verification: serde_json::Value = serde_json::from_str(AUTHORIZED_VERIFICATION).unwrap();
    assert_eq!(
        verification["expected_outputs_are_never_inputs"],
        serde_json::Value::Bool(true)
    );

    let cases = verification["normalized_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 12);

    let calculated_words = cases
        .iter()
        .map(|case| {
            let input_word = u32::try_from(case["input_word_u32"].as_u64().unwrap()).unwrap();
            normalize_none_drc_metric(f32::from_bits(input_word)).map(f32::to_bits)
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let expected_words = cases
        .iter()
        .map(|case| u32::try_from(case["expected_word_u32"].as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(calculated_words, expected_words);
}

#[test]
fn rejects_negative_and_nonfinite_inputs_and_reports_nonfinite_intermediates() {
    assert_eq!(
        normalize_none_drc_metric(-1.0),
        Err(NoneDrcNormalizationError::NegativeInput)
    );
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            normalize_none_drc_metric(value),
            Err(NoneDrcNormalizationError::NonFiniteInput)
        );
    }
    assert_eq!(
        normalize_none_drc_metric(f32::MAX),
        Err(NoneDrcNormalizationError::NumericNonFinite)
    );
}

#[test]
fn negative_zero_uses_the_zero_input_branch() {
    assert_eq!(normalize_none_drc_metric(-0.0), Ok(1.0));
}
