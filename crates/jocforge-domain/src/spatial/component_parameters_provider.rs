// pattern: Functional Core
use super::window_q15_provider::{
    F04ParameterProviderError, F04ParameterProviderLineage, ValidatedF04ParameterPackageManifest,
    exact_keys, exact_u64_field, provider_lineage, q15_array, string_field,
    validate_provider_identity,
};
use crate::frontend::{ProviderFileEvidence, ProviderSha256};
use crate::profile::{EncodingProfile, ResolvedEncodingProfile, resolve_selected_profile};
use serde_json::{Map, Value};

const COMPONENT_SCHEMA: &str = "component-parameter-mathematical-generation.v1";
const COMPONENT_ROUTE: &str = "COMPONENT_FIXED_48K_Q15_PARAMETER_MATH_V1";
const COMPONENT_FIELDS: &[&str] = &[
    "schema",
    "route",
    "sample_rate",
    "block_samples",
    "coefficient_rules",
    "dt_rule",
    "reciprocal_pivot_rule",
    "target_pivot_rule",
    "envelope_weight_rule",
    "q15_integers",
    "coefficient_words",
    "reciprocal_pivot_word",
    "target_pivot_word",
    "envelope_weight_word",
    "original_design_rationale",
    "source_table_copied",
    "public_or_third_party_distribution",
    "implementation_ready",
];
const COEFFICIENT_RULES: [&str; 5] = [
    "Q15_RNE(2^(-dt/(1/25)))",
    "1-c0",
    "Q15_RNE(2^(-dt/(4/25)))",
    "1-c2",
    "Q15_RNE(2^(-dt/(1/200)))",
];
const DT_RULE: &str = "64/48000 seconds";
const RECIPROCAL_PIVOT_RULE: &str = "ceil(32768/sqrt(2))/32768";
const TARGET_PIVOT_RULE: &str = "Q15_RNE(10^(-3/40))";
const ENVELOPE_WEIGHT_RULE: &str = "Q15_RNE(0.5*10^(3/40))";
const ORIGINAL_DESIGN_RATIONALE: &str = "not claimed; equivalent generation settings only";
const AUTHORIZED_SCOPE: &str =
    "INTERNAL_SELECTED_PROFILE_PROVIDER_VALIDATION_AND_FUTURE_CONSUMPTION";
const DSP_CONSUMPTION_STATUS: &str = "NOT_YET_VALIDATED";

/// Immutable receipt for the exact selected component parameter provider.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentParameterProviderReceipt {
    validated_manifest_lineage: F04ParameterProviderLineage,
    provider_file_sha256: ProviderSha256,
    provider_file_bytes: u64,
    schema: &'static str,
    route: &'static str,
    selected_profile: ResolvedEncodingProfile,
    asset_implementation_ready: bool,
    authorized_scope: &'static str,
    dsp_consumption_status: &'static str,
}

impl ComponentParameterProviderReceipt {
    pub const fn validated_manifest_lineage(&self) -> &F04ParameterProviderLineage {
        &self.validated_manifest_lineage
    }

    pub const fn provider_file_sha256(&self) -> ProviderSha256 {
        self.provider_file_sha256
    }

    pub const fn provider_file_bytes(&self) -> u64 {
        self.provider_file_bytes
    }

    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    pub const fn route(&self) -> &'static str {
        self.route
    }

    pub const fn selected_profile(&self) -> ResolvedEncodingProfile {
        self.selected_profile
    }

    pub const fn asset_implementation_ready(&self) -> bool {
        self.asset_implementation_ready
    }

    pub const fn authorized_scope(&self) -> &'static str {
        self.authorized_scope
    }

    pub const fn dsp_consumption_status(&self) -> &'static str {
        self.dsp_consumption_status
    }
}

/// Authenticated component parameters; provider fields are private and read-only.
pub struct ValidatedComponentParameterProvider {
    q15_integers: [u16; 8],
    coefficient_words: [u32; 5],
    reciprocal_pivot_word: u32,
    target_pivot_word: u32,
    envelope_weight_word: u32,
    receipt: ComponentParameterProviderReceipt,
}

impl ValidatedComponentParameterProvider {
    pub const fn q15_integers(&self) -> &[u16; 8] {
        &self.q15_integers
    }

    pub const fn coefficient_words(&self) -> &[u32; 5] {
        &self.coefficient_words
    }

    pub const fn reciprocal_pivot_word(&self) -> u32 {
        self.reciprocal_pivot_word
    }

    pub const fn target_pivot_word(&self) -> u32 {
        self.target_pivot_word
    }

    pub const fn envelope_weight_word(&self) -> u32 {
        self.envelope_weight_word
    }

    pub const fn receipt(&self) -> &ComponentParameterProviderReceipt {
        &self.receipt
    }
}

struct ParsedComponentParameterProvider {
    q15_integers: [u16; 8],
    coefficient_words: [u32; 5],
    reciprocal_pivot_word: u32,
    target_pivot_word: u32,
    envelope_weight_word: u32,
}

/// Validates the selected-profile binding and exact component parameter provider bytes.
pub fn validate_component_parameter_provider(
    evidence: &ProviderFileEvidence,
    manifest: &ValidatedF04ParameterPackageManifest,
    profile: &EncodingProfile,
) -> Result<ValidatedComponentParameterProvider, F04ParameterProviderError> {
    let entry = manifest.component_parameters_entry();
    let provider_sha = validate_provider_identity(evidence, entry)?;
    let selected_profile = resolve_selected_profile(profile)
        .map_err(|_| F04ParameterProviderError::UnsupportedProfile)?;
    let parsed = parse_component_parameter_provider(evidence.bytes())?;
    let lineage = provider_lineage(manifest.manifest_sha256(), entry, provider_sha);
    let receipt = ComponentParameterProviderReceipt {
        validated_manifest_lineage: lineage,
        provider_file_sha256: provider_sha,
        provider_file_bytes: evidence.bytes().len() as u64,
        schema: COMPONENT_SCHEMA,
        route: COMPONENT_ROUTE,
        selected_profile,
        asset_implementation_ready: false,
        authorized_scope: AUTHORIZED_SCOPE,
        dsp_consumption_status: DSP_CONSUMPTION_STATUS,
    };

    Ok(ValidatedComponentParameterProvider {
        q15_integers: parsed.q15_integers,
        coefficient_words: parsed.coefficient_words,
        reciprocal_pivot_word: parsed.reciprocal_pivot_word,
        target_pivot_word: parsed.target_pivot_word,
        envelope_weight_word: parsed.envelope_weight_word,
        receipt,
    })
}

fn parse_component_parameter_provider(
    bytes: &[u8],
) -> Result<ParsedComponentParameterProvider, F04ParameterProviderError> {
    std::str::from_utf8(bytes).map_err(|_| F04ParameterProviderError::Utf8)?;
    let document: Value =
        serde_json::from_slice(bytes).map_err(|_| F04ParameterProviderError::JsonSyntax)?;
    let fields = document
        .as_object()
        .ok_or(F04ParameterProviderError::Schema)?;
    if !exact_keys(fields, COMPONENT_FIELDS) {
        return Err(F04ParameterProviderError::Schema);
    }
    if string_field(fields, "schema")? != COMPONENT_SCHEMA
        || string_field(fields, "route")? != COMPONENT_ROUTE
        || exact_u64_field(fields, "sample_rate")? != 48_000
        || exact_u64_field(fields, "block_samples")? != 64
        || string_array::<5>(
            fields
                .get("coefficient_rules")
                .ok_or(F04ParameterProviderError::Schema)?,
        )? != COEFFICIENT_RULES
        || string_field(fields, "dt_rule")? != DT_RULE
        || string_field(fields, "reciprocal_pivot_rule")? != RECIPROCAL_PIVOT_RULE
        || string_field(fields, "target_pivot_rule")? != TARGET_PIVOT_RULE
        || string_field(fields, "envelope_weight_rule")? != ENVELOPE_WEIGHT_RULE
        || string_field(fields, "original_design_rationale")? != ORIGINAL_DESIGN_RATIONALE
        || string_field(fields, "public_or_third_party_distribution")? != "NOT_AUTHORIZED"
        || string_field(fields, "implementation_ready")? != "NO"
        || fields.get("source_table_copied").and_then(Value::as_bool) != Some(false)
    {
        return Err(F04ParameterProviderError::Schema);
    }

    let q15_integers = q15_array::<8>(
        fields
            .get("q15_integers")
            .ok_or(F04ParameterProviderError::Schema)?,
    )?;
    let coefficient_words = f32_word_array::<5>(
        fields
            .get("coefficient_words")
            .ok_or(F04ParameterProviderError::Schema)?,
    )?;
    for word in coefficient_words {
        require_f32_range(word, 0.0, 1.0, false)?;
    }
    let reciprocal_pivot_word = f32_word_field(fields, "reciprocal_pivot_word")?;
    require_f32_range(reciprocal_pivot_word, 0.5, 1.0, true)?;
    let target_pivot_word = f32_word_field(fields, "target_pivot_word")?;
    require_f32_range(target_pivot_word, 0.0, 1.0, false)?;
    let envelope_weight_word = f32_word_field(fields, "envelope_weight_word")?;
    require_f32_range(envelope_weight_word, 0.0, 1.0, false)?;

    Ok(ParsedComponentParameterProvider {
        q15_integers,
        coefficient_words,
        reciprocal_pivot_word,
        target_pivot_word,
        envelope_weight_word,
    })
}

fn string_array<const N: usize>(value: &Value) -> Result<[&str; N], F04ParameterProviderError> {
    let values = value.as_array().ok_or(F04ParameterProviderError::Shape)?;
    if values.len() != N {
        return Err(F04ParameterProviderError::Shape);
    }
    let mut output = [""; N];
    for (output, value) in output.iter_mut().zip(values) {
        *output = value.as_str().ok_or(F04ParameterProviderError::Schema)?;
    }
    Ok(output)
}

fn f32_word_array<const N: usize>(value: &Value) -> Result<[u32; N], F04ParameterProviderError> {
    let values = value.as_array().ok_or(F04ParameterProviderError::Shape)?;
    if values.len() != N {
        return Err(F04ParameterProviderError::Shape);
    }
    let mut output = [0; N];
    for (output, value) in output.iter_mut().zip(values) {
        let encoded = value.as_str().ok_or(F04ParameterProviderError::Schema)?;
        *output = parse_f32_word(encoded)?;
    }
    Ok(output)
}

fn f32_word_field(
    fields: &Map<String, Value>,
    name: &str,
) -> Result<u32, F04ParameterProviderError> {
    let encoded = string_field(fields, name)?;
    parse_f32_word(encoded)
}

fn parse_f32_word(encoded: &str) -> Result<u32, F04ParameterProviderError> {
    if encoded.len() != 8
        || !encoded
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(F04ParameterProviderError::Schema);
    }
    let bits = u32::from_str_radix(encoded, 16).map_err(|_| F04ParameterProviderError::Schema)?;
    if !f32::from_bits(bits).is_finite() {
        return Err(F04ParameterProviderError::Range);
    }
    Ok(bits)
}

fn require_f32_range(
    bits: u32,
    minimum: f32,
    maximum: f32,
    maximum_exclusive: bool,
) -> Result<(), F04ParameterProviderError> {
    let value = f32::from_bits(bits);
    let maximum_exceeded = if maximum_exclusive {
        value >= maximum
    } else {
        value > maximum
    };
    if value < minimum || maximum_exceeded {
        return Err(F04ParameterProviderError::Range);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{F04ParameterProviderError, parse_component_parameter_provider};
    use serde_json::Value;

    const COMPONENT_PARAMETERS: &[u8] = include_bytes!(
        "../../tests/fixtures/f04_parameters/component-parameter-mathematical-v1.json"
    );

    fn fixture() -> Value {
        serde_json::from_slice(COMPONENT_PARAMETERS)
            .expect("component parameter fixture is valid JSON")
    }

    fn parse_after(
        mut document: Value,
        mutate: impl FnOnce(&mut Value),
    ) -> Result<(), F04ParameterProviderError> {
        mutate(&mut document);
        let bytes = serde_json::to_vec(&document).expect("mutated component document serializes");
        parse_component_parameter_provider(&bytes).map(|_| ())
    }

    #[test]
    fn parses_eight_q15_integers_and_eight_original_f32_words() {
        assert!(parse_component_parameter_provider(COMPONENT_PARAMETERS).is_ok());
    }

    #[test]
    fn rejects_unknown_fields_and_route_or_generation_drift() {
        assert!(matches!(
            parse_after(fixture(), |document| document["extra"] = Value::Null),
            Err(F04ParameterProviderError::Schema)
        ));
        assert!(matches!(
            parse_after(fixture(), |document| {
                document["route"] = Value::String("OTHER".to_owned());
            }),
            Err(F04ParameterProviderError::Schema)
        ));
        assert!(matches!(
            parse_after(fixture(), |document| {
                document["coefficient_rules"][0] = Value::String("regenerate".to_owned());
            }),
            Err(F04ParameterProviderError::Schema)
        ));
    }

    #[test]
    fn rejects_non_integer_and_out_of_range_q15_values() {
        for invalid in [
            Value::Bool(true),
            serde_json::json!(1.0),
            serde_json::json!(32_769),
        ] {
            assert!(matches!(
                parse_after(fixture(), |document| document["q15_integers"][0] = invalid),
                Err(F04ParameterProviderError::Range)
            ));
        }
    }

    #[test]
    fn rejects_missing_or_malformed_f32_word_fields() {
        assert!(matches!(
            parse_after(fixture(), |document| {
                document["coefficient_words"].as_array_mut().unwrap().pop();
            }),
            Err(F04ParameterProviderError::Shape)
        ));
        assert!(matches!(
            parse_after(fixture(), |document| {
                document["coefficient_words"][0] = Value::String("3F7A2800".to_owned());
            }),
            Err(F04ParameterProviderError::Schema)
        ));
    }

    #[test]
    fn rejects_nonfinite_and_out_of_range_f32_words() {
        assert!(matches!(
            parse_after(fixture(), |document| {
                document["coefficient_words"][0] = Value::String("7f800000".to_owned());
            }),
            Err(F04ParameterProviderError::Range)
        ));
        assert!(matches!(
            parse_after(fixture(), |document| {
                document["reciprocal_pivot_word"] = Value::String("3f800000".to_owned());
            }),
            Err(F04ParameterProviderError::Range)
        ));
    }

    #[test]
    fn retains_no_readiness_overclaim_or_source_table_claim() {
        assert!(matches!(
            parse_after(fixture(), |document| {
                document["implementation_ready"] = Value::String("YES".to_owned());
            }),
            Err(F04ParameterProviderError::Schema)
        ));
        assert!(matches!(
            parse_after(fixture(), |document| {
                document["source_table_copied"] = Value::Bool(true);
            }),
            Err(F04ParameterProviderError::Schema)
        ));
    }
}
