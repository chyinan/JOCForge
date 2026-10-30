// pattern: Functional Core
use super::window_q15_provider::{
    F04ParameterProviderError, F04ParameterProviderLineage, ValidatedF04ParameterPackageManifest,
    exact_keys, provider_lineage, validate_provider_identity,
};
use crate::frontend::{ProviderFileEvidence, ProviderSha256};
use serde_json::{Map, Value};

const SCHEMA: &str = "descriptor-mathematical-parameters.v1";
const ORIGIN: &str = "MATHEMATICAL_FORMULAS_GENERATED_IN_THIS_TASK";
const GAIN_FORMULA: &str = "(32+j%32)*2^(floor(j/32))/512";
const THRESHOLD_FORMULA: &str = "R32(-log2(16*gain[j])/8), j0..126";
const GAIN_RESEARCH_COMPARISON: &str = "PASS_ALL128_WORDS";
const THRESHOLD_RESEARCH_COMPARISON: &str = "36_WORD_DIFFERENCES_NOT_IDENTICAL";
const THRESHOLD_ROUTE_STATUS: &str = "MATHEMATICAL_CANDIDATE_AUDIO_ACCEPTANCE_PENDING";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DescriptorMathematicalProviderReceipt {
    validated_manifest_lineage: F04ParameterProviderLineage,
    provider_file_sha256: ProviderSha256,
    provider_file_bytes: u64,
    schema: &'static str,
    origin: &'static str,
    gain_formula: &'static str,
    threshold_formula: &'static str,
    gain_research_comparison: &'static str,
    threshold_research_comparison: &'static str,
    threshold_route_status: &'static str,
    private_vendor_key_included: bool,
}

impl DescriptorMathematicalProviderReceipt {
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

    pub const fn origin(&self) -> &'static str {
        self.origin
    }

    pub const fn gain_formula(&self) -> &'static str {
        self.gain_formula
    }

    pub const fn threshold_formula(&self) -> &'static str {
        self.threshold_formula
    }

    pub const fn gain_research_comparison(&self) -> &'static str {
        self.gain_research_comparison
    }

    pub const fn threshold_research_comparison(&self) -> &'static str {
        self.threshold_research_comparison
    }

    pub const fn threshold_route_status(&self) -> &'static str {
        self.threshold_route_status
    }

    pub const fn private_vendor_key_included(&self) -> bool {
        self.private_vendor_key_included
    }
}

/// Read-only descriptor parameter words; this type does not implement descriptor control.
pub struct DescriptorMathematicalProvider {
    gain_u32: [u32; 128],
    threshold_u32: [u32; 127],
    receipt: DescriptorMathematicalProviderReceipt,
}

impl DescriptorMathematicalProvider {
    pub const fn gain_words(&self) -> &[u32; 128] {
        &self.gain_u32
    }

    pub const fn threshold_words(&self) -> &[u32; 127] {
        &self.threshold_u32
    }

    pub const fn receipt(&self) -> &DescriptorMathematicalProviderReceipt {
        &self.receipt
    }
}

/// Validates the descriptor mathematical provider without running descriptor/audio logic.
pub fn validate_descriptor_mathematical_provider(
    evidence: &ProviderFileEvidence,
    manifest: &ValidatedF04ParameterPackageManifest,
) -> Result<DescriptorMathematicalProvider, F04ParameterProviderError> {
    let entry = manifest.descriptor_mathematical_entry();
    let provider_sha = validate_provider_identity(evidence, entry)?;
    let (gain_u32, threshold_u32) = parse_descriptor_mathematical_provider(evidence.bytes())?;
    let lineage = provider_lineage(
        manifest.manifest_sha256(),
        Some(manifest.approved_source_manifest_sha256()),
        entry,
        provider_sha,
    );
    let receipt = DescriptorMathematicalProviderReceipt {
        validated_manifest_lineage: lineage,
        provider_file_sha256: provider_sha,
        provider_file_bytes: evidence.bytes().len() as u64,
        schema: SCHEMA,
        origin: ORIGIN,
        gain_formula: GAIN_FORMULA,
        threshold_formula: THRESHOLD_FORMULA,
        gain_research_comparison: GAIN_RESEARCH_COMPARISON,
        threshold_research_comparison: THRESHOLD_RESEARCH_COMPARISON,
        threshold_route_status: THRESHOLD_ROUTE_STATUS,
        private_vendor_key_included: false,
    };

    Ok(DescriptorMathematicalProvider {
        gain_u32,
        threshold_u32,
        receipt,
    })
}

fn parse_descriptor_mathematical_provider(
    bytes: &[u8],
) -> Result<([u32; 128], [u32; 127]), F04ParameterProviderError> {
    std::str::from_utf8(bytes).map_err(|_| F04ParameterProviderError::Utf8)?;
    let document: Value =
        serde_json::from_slice(bytes).map_err(|_| F04ParameterProviderError::JsonSyntax)?;
    let fields = document
        .as_object()
        .ok_or(F04ParameterProviderError::Schema)?;
    if !exact_keys(
        fields,
        &[
            "schema",
            "origin",
            "gain_formula",
            "gain_u32",
            "threshold_formula",
            "threshold_u32",
            "gain_research_comparison",
            "threshold_research_comparison",
            "threshold_route_status",
            "private_vendor_key_included",
        ],
    ) || text(fields, "schema")? != SCHEMA
        || text(fields, "origin")? != ORIGIN
        || text(fields, "gain_formula")? != GAIN_FORMULA
        || text(fields, "threshold_formula")? != THRESHOLD_FORMULA
        || text(fields, "gain_research_comparison")? != GAIN_RESEARCH_COMPARISON
        || text(fields, "threshold_research_comparison")? != THRESHOLD_RESEARCH_COMPARISON
        || text(fields, "threshold_route_status")? != THRESHOLD_ROUTE_STATUS
        || fields
            .get("private_vendor_key_included")
            .and_then(Value::as_bool)
            != Some(false)
    {
        return Err(F04ParameterProviderError::Schema);
    }

    let gain_u32 = finite_array::<128>(fields, "gain_u32", true)?;
    let threshold_u32 = finite_array::<127>(fields, "threshold_u32", false)?;
    Ok((gain_u32, threshold_u32))
}

fn finite_array<const N: usize>(
    fields: &Map<String, Value>,
    name: &str,
    positive: bool,
) -> Result<[u32; N], F04ParameterProviderError> {
    let values = fields
        .get(name)
        .and_then(Value::as_array)
        .ok_or(F04ParameterProviderError::Shape)?;
    if values.len() != N {
        return Err(F04ParameterProviderError::Shape);
    }
    let mut words = [0_u32; N];
    for (word, value) in words.iter_mut().zip(values) {
        let raw = value.as_u64().ok_or(F04ParameterProviderError::Range)?;
        let raw = u32::try_from(raw).map_err(|_| F04ParameterProviderError::Range)?;
        let f32_value = f32::from_bits(raw);
        if !f32_value.is_finite() || (positive && f32_value <= 0.0) {
            return Err(F04ParameterProviderError::Range);
        }
        *word = raw;
    }
    Ok(words)
}

fn text<'a>(
    fields: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a str, F04ParameterProviderError> {
    fields
        .get(name)
        .and_then(Value::as_str)
        .ok_or(F04ParameterProviderError::Schema)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const FIXTURE: &[u8] =
        include_bytes!("../../tests/fixtures/f04_parameters/descriptor-mathematical.json");

    fn mutated_bytes(mutate: impl FnOnce(&mut Value)) -> Vec<u8> {
        let mut value: Value = serde_json::from_slice(FIXTURE).expect("fixture JSON parses");
        mutate(&mut value);
        serde_json::to_vec(&value).expect("mutated JSON serializes")
    }

    #[test]
    fn parses_gain_and_threshold_words_with_distinct_qualification() {
        let (gain, threshold) = parse_descriptor_mathematical_provider(FIXTURE)
            .expect("the pinned descriptor provider has the exact mapped shape");
        assert_eq!(gain.len(), 128);
        assert_eq!(threshold.len(), 127);
        assert!(
            gain.iter()
                .all(|word| f32::from_bits(*word).is_finite() && f32::from_bits(*word) > 0.0)
        );
        assert!(
            threshold
                .iter()
                .all(|word| f32::from_bits(*word).is_finite())
        );
    }

    #[test]
    fn rejects_descriptor_threshold_status_or_formula_overclaim() {
        let status_overclaim = mutated_bytes(|value| {
            value["threshold_route_status"] = json!("AUDIO_PASS");
        });
        assert_eq!(
            parse_descriptor_mathematical_provider(&status_overclaim).unwrap_err(),
            F04ParameterProviderError::Schema
        );
        let formula_drift = mutated_bytes(|value| {
            value["gain_formula"] = json!("generated another way");
        });
        assert_eq!(
            parse_descriptor_mathematical_provider(&formula_drift).unwrap_err(),
            F04ParameterProviderError::Schema
        );
        let vendor_key = mutated_bytes(|value| value["private_vendor_key_included"] = json!(true));
        assert_eq!(
            parse_descriptor_mathematical_provider(&vendor_key).unwrap_err(),
            F04ParameterProviderError::Schema
        );
    }

    #[test]
    fn rejects_descriptor_count_word_type_and_gain_range_drift() {
        let threshold_count = mutated_bytes(|value| {
            value["threshold_u32"]
                .as_array_mut()
                .unwrap()
                .push(json!(0));
        });
        assert_eq!(
            parse_descriptor_mathematical_provider(&threshold_count).unwrap_err(),
            F04ParameterProviderError::Shape
        );
        for invalid_gain in [
            json!(true),
            json!(1.0),
            json!(0x7f80_0000_u32),
            json!(0_u32),
        ] {
            let invalid = mutated_bytes(|value| value["gain_u32"][0] = invalid_gain);
            assert_eq!(
                parse_descriptor_mathematical_provider(&invalid).unwrap_err(),
                F04ParameterProviderError::Range
            );
        }
        let invalid_threshold = mutated_bytes(|value| value["threshold_u32"][0] = json!(1.0));
        assert_eq!(
            parse_descriptor_mathematical_provider(&invalid_threshold).unwrap_err(),
            F04ParameterProviderError::Range
        );
        let extra_field = mutated_bytes(|value| value["audio_acceptance"] = json!("NOT_RUN"));
        assert_eq!(
            parse_descriptor_mathematical_provider(&extra_field).unwrap_err(),
            F04ParameterProviderError::Schema
        );
    }
}
