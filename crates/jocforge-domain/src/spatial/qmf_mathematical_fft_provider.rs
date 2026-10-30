// pattern: Functional Core
use super::window_q15_provider::{
    F04ParameterProviderError, F04ParameterProviderLineage, ValidatedF04ParameterPackageManifest,
    exact_keys, provider_lineage, validate_provider_identity,
};
use crate::frontend::{ProviderFileEvidence, ProviderSha256};
use serde_json::{Map, Value};

const SCHEMA: &str = "qmf-mathematical-fft-parameters.v1";
const ROUTE: &str = "QMF_FFT_RADIX2_MATH_V1";
const SOURCE: &str = "MATHEMATICAL_GENERATION_NO_COMPILED_VALUES_USED";
const AUDIO_ACCEPTANCE: &str = "NOT_RUN";
const STAGE_LENGTHS: [u64; 6] = [2, 4, 8, 16, 32, 64];
const ROOT_COUNTS: [usize; 6] = [1, 2, 4, 8, 16, 32];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QmfMathematicalFftProviderReceipt {
    validated_manifest_lineage: F04ParameterProviderLineage,
    provider_file_sha256: ProviderSha256,
    provider_file_bytes: u64,
    schema: &'static str,
    route: &'static str,
    source: &'static str,
    audio_acceptance: &'static str,
}

impl QmfMathematicalFftProviderReceipt {
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

    pub const fn source(&self) -> &'static str {
        self.source
    }

    pub const fn audio_acceptance(&self) -> &'static str {
        self.audio_acceptance
    }
}

/// One supplied root stage. This is parameter storage only, not an FFT stage executor.
#[derive(Debug)]
pub struct QmfFftRootStage {
    length: u8,
    roots: Vec<[u32; 2]>,
}

impl QmfFftRootStage {
    pub const fn length(&self) -> u8 {
        self.length
    }

    pub fn roots(&self) -> &[[u32; 2]] {
        &self.roots
    }
}

/// Read-only supplied roots and rotations after exact provider identity validation.
pub struct QmfMathematicalFftProvider {
    stages: [QmfFftRootStage; 6],
    rotation_u32: [u32; 128],
    receipt: QmfMathematicalFftProviderReceipt,
}

impl QmfMathematicalFftProvider {
    pub const fn stages(&self) -> &[QmfFftRootStage; 6] {
        &self.stages
    }

    pub const fn rotation_words(&self) -> &[u32; 128] {
        &self.rotation_u32
    }

    pub const fn receipt(&self) -> &QmfMathematicalFftProviderReceipt {
        &self.receipt
    }
}

/// Validates the selected generated QMF FFT parameter file without executing FFT or QMF logic.
pub fn validate_qmf_mathematical_fft_provider(
    evidence: &ProviderFileEvidence,
    manifest: &ValidatedF04ParameterPackageManifest,
) -> Result<QmfMathematicalFftProvider, F04ParameterProviderError> {
    let entry = manifest.qmf_mathematical_fft_entry();
    let provider_sha = validate_provider_identity(evidence, entry)?;
    let (stages, rotation_u32) = parse_qmf_mathematical_fft_provider(evidence.bytes())?;
    let lineage = provider_lineage(
        manifest.manifest_sha256(),
        Some(manifest.approved_source_manifest_sha256()),
        entry,
        provider_sha,
    );
    let receipt = QmfMathematicalFftProviderReceipt {
        validated_manifest_lineage: lineage,
        provider_file_sha256: provider_sha,
        provider_file_bytes: evidence.bytes().len() as u64,
        schema: SCHEMA,
        route: ROUTE,
        source: SOURCE,
        audio_acceptance: AUDIO_ACCEPTANCE,
    };

    Ok(QmfMathematicalFftProvider {
        stages,
        rotation_u32,
        receipt,
    })
}

fn parse_qmf_mathematical_fft_provider(
    bytes: &[u8],
) -> Result<([QmfFftRootStage; 6], [u32; 128]), F04ParameterProviderError> {
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
            "route",
            "source",
            "stages",
            "rotation_u32",
            "audio_acceptance",
        ],
    ) || text(fields, "schema")? != SCHEMA
        || text(fields, "route")? != ROUTE
        || text(fields, "source")? != SOURCE
        || text(fields, "audio_acceptance")? != AUDIO_ACCEPTANCE
    {
        return Err(F04ParameterProviderError::Schema);
    }

    let encoded_stages = fields
        .get("stages")
        .and_then(Value::as_array)
        .ok_or(F04ParameterProviderError::Shape)?;
    if encoded_stages.len() != STAGE_LENGTHS.len() {
        return Err(F04ParameterProviderError::Shape);
    }

    let mut stages = Vec::with_capacity(STAGE_LENGTHS.len());
    for (index, (encoded, expected_length)) in encoded_stages.iter().zip(STAGE_LENGTHS).enumerate()
    {
        let stage = encoded
            .as_object()
            .ok_or(F04ParameterProviderError::Shape)?;
        if !exact_keys(stage, &["length", "roots_u32"])
            || unsigned(stage, "length")? != expected_length
        {
            return Err(F04ParameterProviderError::Schema);
        }
        let roots = stage
            .get("roots_u32")
            .and_then(Value::as_array)
            .ok_or(F04ParameterProviderError::Shape)?;
        if roots.len() != ROOT_COUNTS[index] {
            return Err(F04ParameterProviderError::Shape);
        }
        let mut parsed_roots = Vec::with_capacity(roots.len());
        for pair in roots {
            let pair = pair.as_array().ok_or(F04ParameterProviderError::Shape)?;
            if pair.len() != 2 {
                return Err(F04ParameterProviderError::Shape);
            }
            let real = finite_word(&pair[0])?;
            let imaginary = finite_word(&pair[1])?;
            parsed_roots.push([real, imaginary]);
        }
        stages.push(QmfFftRootStage {
            length: expected_length as u8,
            roots: parsed_roots,
        });
    }
    let stages: [QmfFftRootStage; 6] = stages
        .try_into()
        .map_err(|_| F04ParameterProviderError::Shape)?;

    let rotation_u32 = array::<128>(
        fields
            .get("rotation_u32")
            .ok_or(F04ParameterProviderError::Shape)?,
    )?;

    Ok((stages, rotation_u32))
}

fn array<const N: usize>(value: &Value) -> Result<[u32; N], F04ParameterProviderError> {
    let values = value.as_array().ok_or(F04ParameterProviderError::Shape)?;
    if values.len() != N {
        return Err(F04ParameterProviderError::Shape);
    }
    let mut result = [0_u32; N];
    for (out, value) in result.iter_mut().zip(values) {
        *out = finite_word(value)?;
    }
    Ok(result)
}

fn finite_word(value: &Value) -> Result<u32, F04ParameterProviderError> {
    let raw = value.as_u64().ok_or(F04ParameterProviderError::Range)?;
    let raw = u32::try_from(raw).map_err(|_| F04ParameterProviderError::Range)?;
    if !f32::from_bits(raw).is_finite() {
        return Err(F04ParameterProviderError::Range);
    }
    Ok(raw)
}

fn unsigned(fields: &Map<String, Value>, name: &str) -> Result<u64, F04ParameterProviderError> {
    fields
        .get(name)
        .and_then(Value::as_u64)
        .ok_or(F04ParameterProviderError::Range)
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
        include_bytes!("../../tests/fixtures/f04_parameters/qmf-mathematical-fft-v1.json");

    fn mutated_bytes(mutate: impl FnOnce(&mut Value)) -> Vec<u8> {
        let mut value: Value = serde_json::from_slice(FIXTURE).expect("fixture JSON parses");
        mutate(&mut value);
        serde_json::to_vec(&value).expect("mutated JSON serializes")
    }

    #[test]
    fn parses_only_the_six_mapped_root_stages_and_rotation_words() {
        let (stages, rotations) = parse_qmf_mathematical_fft_provider(FIXTURE)
            .expect("the pinned mathematical provider has the exact mapped shape");
        assert_eq!(
            stages.iter().map(|stage| stage.length).collect::<Vec<_>>(),
            vec![2, 4, 8, 16, 32, 64]
        );
        assert_eq!(
            stages
                .iter()
                .map(|stage| stage.roots.len())
                .collect::<Vec<_>>(),
            vec![1, 2, 4, 8, 16, 32]
        );
        assert_eq!(rotations.len(), 128);
        assert!(
            stages
                .iter()
                .flat_map(|stage| stage.roots.iter().flatten())
                .chain(rotations.iter())
                .all(|word| f32::from_bits(*word).is_finite())
        );
    }

    #[test]
    fn rejects_wrong_route_source_or_acceptance_qualification() {
        for (field, replacement) in [
            ("route", "QMF_FFT_OTHER_V1"),
            ("source", "COMPILED_VALUES"),
            ("audio_acceptance", "PASS"),
        ] {
            let invalid = mutated_bytes(|value| value[field] = json!(replacement));
            assert_eq!(
                parse_qmf_mathematical_fft_provider(&invalid).unwrap_err(),
                F04ParameterProviderError::Schema
            );
        }
    }

    #[test]
    fn rejects_stage_order_field_or_root_shape_drift() {
        let wrong_order = mutated_bytes(|value| value["stages"].as_array_mut().unwrap().swap(0, 1));
        assert_eq!(
            parse_qmf_mathematical_fft_provider(&wrong_order).unwrap_err(),
            F04ParameterProviderError::Schema
        );
        let wrong_root_count = mutated_bytes(|value| {
            value["stages"][0]["roots_u32"]
                .as_array_mut()
                .unwrap()
                .pop();
        });
        assert_eq!(
            parse_qmf_mathematical_fft_provider(&wrong_root_count).unwrap_err(),
            F04ParameterProviderError::Shape
        );
        let wrong_pair = mutated_bytes(|value| {
            value["stages"][0]["roots_u32"][0]
                .as_array_mut()
                .unwrap()
                .pop();
        });
        assert_eq!(
            parse_qmf_mathematical_fft_provider(&wrong_pair).unwrap_err(),
            F04ParameterProviderError::Shape
        );
    }

    #[test]
    fn rejects_non_u32_and_nonfinite_fft_words() {
        for invalid_word in [json!(true), json!(1.0), json!(0x7f80_0000_u32)] {
            let invalid = mutated_bytes(|value| value["rotation_u32"][0] = invalid_word);
            assert_eq!(
                parse_qmf_mathematical_fft_provider(&invalid).unwrap_err(),
                F04ParameterProviderError::Range
            );
        }
        let unknown = mutated_bytes(|value| value["unmapped"] = json!(0));
        assert_eq!(
            parse_qmf_mathematical_fft_provider(&unknown).unwrap_err(),
            F04ParameterProviderError::Schema
        );
    }
}
