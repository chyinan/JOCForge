// pattern: Functional Core
use super::window_q15_provider::{
    F04ParameterProviderError, F04ParameterProviderLineage, ValidatedF04ParameterPackageManifest,
    exact_keys, provider_lineage, validate_provider_identity,
};
use crate::frontend::{ProviderFileEvidence, ProviderSha256};
use serde_json::{Map, Value};

const SCHEMA: &str = "qmf-receiver-authorized-functional-parameters-v1";
const DELIVERY_SCOPE: &str =
    "internal isolated implementation only; no public or third-party redistribution";
const AUTHORIZATION: &str = "all ordinary non-secret functional constants may be delivered; vendor-secret credentials remain excluded";
const NOT_INCLUDED: [&str; 6] = [
    "vendor-secret credentials",
    "authentication secrets",
    "research source",
    "raw original table artifacts",
    "compiled images",
    "addresses",
];
const ARRAY_FIELDS: [&str; 5] = [
    "qwin_synthesis_640",
    "band0_real_mode1_21",
    "band0_imag_mode1_21",
    "lfe_post_filter_4x5",
    "descriptor_threshold_127",
];
const ARRAY_COUNTS: [usize; 5] = [640, 21, 21, 20, 127];
const ARRAY_SOURCE_CLASSES: [&str; 5] = [
    "PUBLIC_STANDARD_FUNCTIONAL_VALUES_WITH_VERIFIED_EXISTING_COPY",
    "AUTHORIZED_NONSECRET_COMPILED_AUDIO_FUNCTIONAL_CONSTANTS",
    "AUTHORIZED_NONSECRET_COMPILED_AUDIO_FUNCTIONAL_CONSTANTS",
    "AUTHORIZED_NONSECRET_FUNCTIONAL_CONSTANTS_BOUND_TO_FROZEN_BITEXACT_MODEL",
    "AUTHORIZED_NONSECRET_COMPILED_AUDIO_FUNCTIONAL_CONSTANTS",
];
const ARRAY_CONSUMERS: [&str; 5] = [
    "F04Q synthesis prototype; analysis is exact reverse, so no second delivered table",
    "F04Q mode=1 audio surround band0 FIR and mode=0 fitting history update contract",
    "F04Q mode=1 audio surround band0 FIR; current selected fitting mode=0 does not multiply taps",
    "F04R lane3 four-section pair-order post-filter",
    "F04D strict lookup for ordinals 0..126",
];
const ARRAY_F32LE_SHA256: [&str; 5] = [
    "870d3c80e4fe7ee368b5da1f7db487dfa84be44c4cf8c82813b78fb06a8e94a2",
    "b352accaefeb0ff0bac1981dcbe9112ff9bd9437e14f1662076b41cf9fd70c70",
    "946593788adebf7dd8beafb057c77e167e5b066607306d2a4e31d99244b8b3be",
    "ddd163cfb2116be81bf283746b3faf38fe771b0d01cee1bf31ea3b0fd04a16eb",
    "5ba787b1ee814e20bbc86290eca1daa6af33968e4e2458b5a3011c87a78fceac",
];
const SCALAR_FIELDS: [&str; 12] = [
    "preconditioning_alpha",
    "preconditioning_beta",
    "preconditioning_tiny_strict",
    "accumulator_magnitude_threshold",
    "accumulator_delta_threshold",
    "accumulator_smoothing",
    "descriptor_level_offset",
    "descriptor_level_linear",
    "descriptor_level_quadratic",
    "descriptor_headroom_step",
    "descriptor_result_scale",
    "descriptor_rise",
];
const GENERATED_PROVIDERS: [(&str, &str); 3] = [
    (
        "QMF_FFT_RADIX2_MATH_V1",
        "parameters/qmf-mathematical-fft-v1.json",
    ),
    (
        "descriptor_gain_PG03",
        "parameters/descriptor-mathematical.json",
    ),
    ("receiver_blend32_PG01", "parameters/window-q15.json"),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QmfFunctionalArraySourceClass {
    PublicStandardFunctionalValues,
    AuthorizedNonsecretCompiledAudioFunctionalConstants,
    AuthorizedNonsecretFunctionalConstantsBoundToFrozenBitExactModel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QmfReceiverFunctionalScalars {
    preconditioning_alpha: u32,
    preconditioning_beta: u32,
    preconditioning_tiny_strict: u32,
    accumulator_magnitude_threshold: u32,
    accumulator_delta_threshold: u32,
    accumulator_smoothing: u32,
    descriptor_level_offset: u32,
    descriptor_level_linear: u32,
    descriptor_level_quadratic: u32,
    descriptor_headroom_step: u32,
    descriptor_result_scale: u32,
    descriptor_rise: u32,
}

impl QmfReceiverFunctionalScalars {
    pub const fn words(&self) -> [u32; 12] {
        [
            self.preconditioning_alpha,
            self.preconditioning_beta,
            self.preconditioning_tiny_strict,
            self.accumulator_magnitude_threshold,
            self.accumulator_delta_threshold,
            self.accumulator_smoothing,
            self.descriptor_level_offset,
            self.descriptor_level_linear,
            self.descriptor_level_quadratic,
            self.descriptor_headroom_step,
            self.descriptor_result_scale,
            self.descriptor_rise,
        ]
    }

    pub const fn preconditioning_alpha(&self) -> u32 {
        self.preconditioning_alpha
    }

    pub const fn preconditioning_beta(&self) -> u32 {
        self.preconditioning_beta
    }

    pub const fn preconditioning_tiny_strict(&self) -> u32 {
        self.preconditioning_tiny_strict
    }

    pub const fn accumulator_magnitude_threshold(&self) -> u32 {
        self.accumulator_magnitude_threshold
    }

    pub const fn accumulator_delta_threshold(&self) -> u32 {
        self.accumulator_delta_threshold
    }

    pub const fn accumulator_smoothing(&self) -> u32 {
        self.accumulator_smoothing
    }

    pub const fn descriptor_level_offset(&self) -> u32 {
        self.descriptor_level_offset
    }

    pub const fn descriptor_level_linear(&self) -> u32 {
        self.descriptor_level_linear
    }

    pub const fn descriptor_level_quadratic(&self) -> u32 {
        self.descriptor_level_quadratic
    }

    pub const fn descriptor_headroom_step(&self) -> u32 {
        self.descriptor_headroom_step
    }

    pub const fn descriptor_result_scale(&self) -> u32 {
        self.descriptor_result_scale
    }

    pub const fn descriptor_rise(&self) -> u32 {
        self.descriptor_rise
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QmfReceiverFunctionalProviderReceipt {
    validated_manifest_lineage: F04ParameterProviderLineage,
    provider_file_sha256: ProviderSha256,
    provider_file_bytes: u64,
    schema: &'static str,
    delivery_scope: &'static str,
    authorization: &'static str,
    not_included: [&'static str; 6],
    array_source_classes: [QmfFunctionalArraySourceClass; 5],
    qmf_fft_provider: &'static str,
    descriptor_gain_provider: &'static str,
    receiver_blend_provider: &'static str,
}

impl QmfReceiverFunctionalProviderReceipt {
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

    pub const fn delivery_scope(&self) -> &'static str {
        self.delivery_scope
    }

    pub const fn authorization(&self) -> &'static str {
        self.authorization
    }

    pub const fn not_included(&self) -> &[&'static str; 6] {
        &self.not_included
    }

    pub const fn array_source_classes(&self) -> &[QmfFunctionalArraySourceClass; 5] {
        &self.array_source_classes
    }

    pub const fn qmf_fft_provider(&self) -> &'static str {
        self.qmf_fft_provider
    }

    pub const fn descriptor_gain_provider(&self) -> &'static str {
        self.descriptor_gain_provider
    }

    pub const fn receiver_blend_provider(&self) -> &'static str {
        self.receiver_blend_provider
    }
}

/// Read-only functional words after exact manifest and provider identity validation.
pub struct QmfReceiverFunctionalProvider {
    qwin_synthesis_640: [u32; 640],
    band0_real_mode1_21: [u32; 21],
    band0_imag_mode1_21: [u32; 21],
    lfe_post_filter_4x5: [u32; 20],
    descriptor_threshold_127: [u32; 127],
    scalars: QmfReceiverFunctionalScalars,
    receipt: QmfReceiverFunctionalProviderReceipt,
}

impl QmfReceiverFunctionalProvider {
    pub const fn qwin_synthesis_words(&self) -> &[u32; 640] {
        &self.qwin_synthesis_640
    }

    pub const fn band0_real_mode1_words(&self) -> &[u32; 21] {
        &self.band0_real_mode1_21
    }

    pub const fn band0_imag_mode1_words(&self) -> &[u32; 21] {
        &self.band0_imag_mode1_21
    }

    pub const fn lfe_post_filter_words(&self) -> &[u32; 20] {
        &self.lfe_post_filter_4x5
    }

    pub const fn descriptor_threshold_words(&self) -> &[u32; 127] {
        &self.descriptor_threshold_127
    }

    pub const fn scalar_words(&self) -> [u32; 12] {
        self.scalars.words()
    }

    pub const fn scalars(&self) -> &QmfReceiverFunctionalScalars {
        &self.scalars
    }

    pub const fn array_source_classes(&self) -> &[QmfFunctionalArraySourceClass; 5] {
        self.receipt.array_source_classes()
    }

    pub const fn receipt(&self) -> &QmfReceiverFunctionalProviderReceipt {
        &self.receipt
    }
}

#[derive(Debug)]
struct ParsedQmfReceiverFunctionalProvider {
    qwin_synthesis_640: [u32; 640],
    band0_real_mode1_21: [u32; 21],
    band0_imag_mode1_21: [u32; 21],
    lfe_post_filter_4x5: [u32; 20],
    descriptor_threshold_127: [u32; 127],
    scalars: QmfReceiverFunctionalScalars,
}

/// Validates the selected QMF receiver functional-parameter file without running a consumer.
pub fn validate_qmf_receiver_functional_provider(
    evidence: &ProviderFileEvidence,
    manifest: &ValidatedF04ParameterPackageManifest,
) -> Result<QmfReceiverFunctionalProvider, F04ParameterProviderError> {
    let entry = manifest.qmf_receiver_functional_entry();
    let provider_sha = validate_provider_identity(evidence, entry)?;
    let parsed = parse_qmf_receiver_functional_provider(evidence.bytes())?;
    let lineage = provider_lineage(
        manifest.manifest_sha256(),
        Some(manifest.approved_source_manifest_sha256()),
        entry,
        provider_sha,
    );
    let receipt = QmfReceiverFunctionalProviderReceipt {
        validated_manifest_lineage: lineage,
        provider_file_sha256: provider_sha,
        provider_file_bytes: evidence.bytes().len() as u64,
        schema: SCHEMA,
        delivery_scope: DELIVERY_SCOPE,
        authorization: AUTHORIZATION,
        not_included: NOT_INCLUDED,
        array_source_classes: [
            QmfFunctionalArraySourceClass::PublicStandardFunctionalValues,
            QmfFunctionalArraySourceClass::AuthorizedNonsecretCompiledAudioFunctionalConstants,
            QmfFunctionalArraySourceClass::AuthorizedNonsecretCompiledAudioFunctionalConstants,
            QmfFunctionalArraySourceClass::AuthorizedNonsecretFunctionalConstantsBoundToFrozenBitExactModel,
            QmfFunctionalArraySourceClass::AuthorizedNonsecretCompiledAudioFunctionalConstants,
        ],
        qmf_fft_provider: "parameters/qmf-mathematical-fft-v1.json",
        descriptor_gain_provider: "parameters/descriptor-mathematical.json",
        receiver_blend_provider: "parameters/window-q15.json",
    };

    Ok(QmfReceiverFunctionalProvider {
        qwin_synthesis_640: parsed.qwin_synthesis_640,
        band0_real_mode1_21: parsed.band0_real_mode1_21,
        band0_imag_mode1_21: parsed.band0_imag_mode1_21,
        lfe_post_filter_4x5: parsed.lfe_post_filter_4x5,
        descriptor_threshold_127: parsed.descriptor_threshold_127,
        scalars: parsed.scalars,
        receipt,
    })
}

fn parse_qmf_receiver_functional_provider(
    bytes: &[u8],
) -> Result<ParsedQmfReceiverFunctionalProvider, F04ParameterProviderError> {
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
            "delivery_scope",
            "authorization",
            "not_included",
            "arrays",
            "scalar_u32",
            "generated_providers_reused",
            "counts",
        ],
    ) || text(fields, "schema")? != SCHEMA
        || text(fields, "delivery_scope")? != DELIVERY_SCOPE
        || text(fields, "authorization")? != AUTHORIZATION
    {
        return Err(F04ParameterProviderError::Schema);
    }

    let excluded = fields
        .get("not_included")
        .and_then(Value::as_array)
        .ok_or(F04ParameterProviderError::Shape)?;
    if excluded.len() != NOT_INCLUDED.len()
        || excluded
            .iter()
            .zip(NOT_INCLUDED)
            .any(|(value, expected)| value.as_str() != Some(expected))
    {
        return Err(F04ParameterProviderError::Schema);
    }

    let arrays = fields
        .get("arrays")
        .and_then(Value::as_object)
        .ok_or(F04ParameterProviderError::Shape)?;
    if !exact_keys(arrays, &ARRAY_FIELDS) {
        return Err(F04ParameterProviderError::Schema);
    }

    let qwin_synthesis_640 = read_array::<640>(arrays, 0)?;
    let band0_real_mode1_21 = read_array::<21>(arrays, 1)?;
    let band0_imag_mode1_21 = read_array::<21>(arrays, 2)?;
    let lfe_post_filter_4x5 = read_array::<20>(arrays, 3)?;
    let descriptor_threshold_127 = read_array::<127>(arrays, 4)?;

    let scalar_fields = fields
        .get("scalar_u32")
        .and_then(Value::as_object)
        .ok_or(F04ParameterProviderError::Shape)?;
    if !exact_keys(scalar_fields, &SCALAR_FIELDS) {
        return Err(F04ParameterProviderError::Schema);
    }
    let scalars = QmfReceiverFunctionalScalars {
        preconditioning_alpha: finite_word(scalar_fields, SCALAR_FIELDS[0])?,
        preconditioning_beta: finite_word(scalar_fields, SCALAR_FIELDS[1])?,
        preconditioning_tiny_strict: finite_word(scalar_fields, SCALAR_FIELDS[2])?,
        accumulator_magnitude_threshold: finite_word(scalar_fields, SCALAR_FIELDS[3])?,
        accumulator_delta_threshold: finite_word(scalar_fields, SCALAR_FIELDS[4])?,
        accumulator_smoothing: finite_word(scalar_fields, SCALAR_FIELDS[5])?,
        descriptor_level_offset: finite_word(scalar_fields, SCALAR_FIELDS[6])?,
        descriptor_level_linear: finite_word(scalar_fields, SCALAR_FIELDS[7])?,
        descriptor_level_quadratic: finite_word(scalar_fields, SCALAR_FIELDS[8])?,
        descriptor_headroom_step: finite_word(scalar_fields, SCALAR_FIELDS[9])?,
        descriptor_result_scale: finite_word(scalar_fields, SCALAR_FIELDS[10])?,
        descriptor_rise: finite_word(scalar_fields, SCALAR_FIELDS[11])?,
    };

    let generated = fields
        .get("generated_providers_reused")
        .and_then(Value::as_object)
        .ok_or(F04ParameterProviderError::Shape)?;
    if !exact_keys(
        generated,
        &[
            "QMF_FFT_RADIX2_MATH_V1",
            "descriptor_gain_PG03",
            "receiver_blend32_PG01",
        ],
    ) || GENERATED_PROVIDERS
        .iter()
        .any(|(name, expected)| generated.get(*name).and_then(Value::as_str) != Some(*expected))
    {
        return Err(F04ParameterProviderError::Schema);
    }

    let counts = fields
        .get("counts")
        .and_then(Value::as_object)
        .ok_or(F04ParameterProviderError::Shape)?;
    if !exact_keys(
        counts,
        &[
            "delivered_array_f32_words",
            "delivered_scalar_f32_words",
            "excluded_unused_mode_other_imag_words",
        ],
    ) || unsigned(counts, "delivered_array_f32_words")? != 829
        || unsigned(counts, "delivered_scalar_f32_words")? != 12
        || unsigned(counts, "excluded_unused_mode_other_imag_words")? != 21
        || ARRAY_COUNTS.iter().sum::<usize>() != 829
    {
        return Err(F04ParameterProviderError::Shape);
    }

    Ok(ParsedQmfReceiverFunctionalProvider {
        qwin_synthesis_640,
        band0_real_mode1_21,
        band0_imag_mode1_21,
        lfe_post_filter_4x5,
        descriptor_threshold_127,
        scalars,
    })
}

fn read_array<const N: usize>(
    arrays: &Map<String, Value>,
    index: usize,
) -> Result<[u32; N], F04ParameterProviderError> {
    let name = ARRAY_FIELDS[index];
    let record = arrays
        .get(name)
        .and_then(Value::as_object)
        .ok_or(F04ParameterProviderError::Shape)?;
    if !exact_keys(
        record,
        &["source_class", "consumer", "u32_words", "f32le_sha256"],
    ) || text(record, "source_class")? != ARRAY_SOURCE_CLASSES[index]
        || text(record, "consumer")? != ARRAY_CONSUMERS[index]
    {
        return Err(F04ParameterProviderError::Schema);
    }
    let encoded = record
        .get("u32_words")
        .and_then(Value::as_array)
        .ok_or(F04ParameterProviderError::Shape)?;
    if encoded.len() != N || encoded.len() != ARRAY_COUNTS[index] {
        return Err(F04ParameterProviderError::Shape);
    }
    let mut words = [0_u32; N];
    let mut f32le = Vec::with_capacity(N * 4);
    for (word, value) in words.iter_mut().zip(encoded) {
        let raw = value.as_u64().ok_or(F04ParameterProviderError::Range)?;
        let raw = u32::try_from(raw).map_err(|_| F04ParameterProviderError::Range)?;
        if !f32::from_bits(raw).is_finite() {
            return Err(F04ParameterProviderError::Range);
        }
        *word = raw;
        f32le.extend_from_slice(&raw.to_le_bytes());
    }
    let expected_digest = ARRAY_F32LE_SHA256[index];
    if text(record, "f32le_sha256")? != expected_digest
        || ProviderSha256::digest(&f32le).to_hex() != expected_digest
    {
        return Err(F04ParameterProviderError::ProviderIdentityMismatch);
    }
    Ok(words)
}

fn finite_word(fields: &Map<String, Value>, name: &str) -> Result<u32, F04ParameterProviderError> {
    let raw = unsigned(fields, name)?;
    let word = u32::try_from(raw).map_err(|_| F04ParameterProviderError::Range)?;
    if !f32::from_bits(word).is_finite() {
        return Err(F04ParameterProviderError::Range);
    }
    Ok(word)
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
        include_bytes!("../../tests/fixtures/f04_parameters/qmf-receiver-functional-v1.json");

    fn mutated_bytes(mutate: impl FnOnce(&mut Value)) -> Vec<u8> {
        let mut value: Value = serde_json::from_slice(FIXTURE).expect("fixture JSON parses");
        mutate(&mut value);
        serde_json::to_vec(&value).expect("mutated JSON serializes")
    }

    #[test]
    fn parses_exact_receiver_arrays_scalars_and_source_classes() {
        let parsed = parse_qmf_receiver_functional_provider(FIXTURE)
            .expect("the pinned functional provider has the exact mapped shape");
        assert_eq!(parsed.qwin_synthesis_640.len(), 640);
        assert_eq!(parsed.band0_real_mode1_21.len(), 21);
        assert_eq!(parsed.band0_imag_mode1_21.len(), 21);
        assert_eq!(parsed.lfe_post_filter_4x5.len(), 20);
        assert_eq!(parsed.descriptor_threshold_127.len(), 127);
        assert!(
            parsed
                .scalars
                .words()
                .iter()
                .all(|word| f32::from_bits(*word).is_finite())
        );
        assert_eq!(ARRAY_SOURCE_CLASSES.len(), ARRAY_FIELDS.len());
    }

    #[test]
    fn rejects_unknown_receiver_fields_and_classification_drift() {
        let unknown = mutated_bytes(|value| value["unexpected"] = json!(true));
        assert_eq!(
            parse_qmf_receiver_functional_provider(&unknown).unwrap_err(),
            F04ParameterProviderError::Schema
        );
        let changed_class = mutated_bytes(|value| {
            value["arrays"]["qwin_synthesis_640"]["source_class"] = json!("GENERATED");
        });
        assert_eq!(
            parse_qmf_receiver_functional_provider(&changed_class).unwrap_err(),
            F04ParameterProviderError::Schema
        );
    }

    #[test]
    fn rejects_receiver_scalar_array_misclassification() {
        let wrong_scalar_set = mutated_bytes(|value| {
            value["scalar_u32"]
                .as_object_mut()
                .unwrap()
                .remove("descriptor_rise");
        });
        assert_eq!(
            parse_qmf_receiver_functional_provider(&wrong_scalar_set).unwrap_err(),
            F04ParameterProviderError::Schema
        );
        let wrong_array_length = mutated_bytes(|value| {
            value["arrays"]["lfe_post_filter_4x5"]["u32_words"]
                .as_array_mut()
                .unwrap()
                .pop();
        });
        assert_eq!(
            parse_qmf_receiver_functional_provider(&wrong_array_length).unwrap_err(),
            F04ParameterProviderError::Shape
        );
    }

    #[test]
    fn rejects_receiver_boolean_float_nonfinite_and_digest_drift() {
        for raw in [json!(true), json!(1.0), json!(0x7f80_0000_u32)] {
            let invalid = mutated_bytes(|value| {
                value["scalar_u32"]["preconditioning_alpha"] = raw;
            });
            assert_eq!(
                parse_qmf_receiver_functional_provider(&invalid).unwrap_err(),
                F04ParameterProviderError::Range
            );
        }
        let invalid_array = mutated_bytes(|value| {
            value["arrays"]["qwin_synthesis_640"]["u32_words"][0] = json!(false);
        });
        assert_eq!(
            parse_qmf_receiver_functional_provider(&invalid_array).unwrap_err(),
            F04ParameterProviderError::Range
        );
        let wrong_digest = mutated_bytes(|value| {
            value["arrays"]["qwin_synthesis_640"]["f32le_sha256"] = json!("00".repeat(32));
        });
        assert_eq!(
            parse_qmf_receiver_functional_provider(&wrong_digest).unwrap_err(),
            F04ParameterProviderError::ProviderIdentityMismatch
        );
    }

    #[test]
    fn rejects_generated_provider_substitution_and_totals_drift() {
        let substituted = mutated_bytes(|value| {
            value["generated_providers_reused"]["QMF_FFT_RADIX2_MATH_V1"] =
                json!("parameters/descriptor-mathematical.json");
        });
        assert_eq!(
            parse_qmf_receiver_functional_provider(&substituted).unwrap_err(),
            F04ParameterProviderError::Schema
        );
        let changed_total = mutated_bytes(|value| {
            value["counts"]["delivered_array_f32_words"] = json!(828);
        });
        assert_eq!(
            parse_qmf_receiver_functional_provider(&changed_total).unwrap_err(),
            F04ParameterProviderError::Shape
        );
    }
}
