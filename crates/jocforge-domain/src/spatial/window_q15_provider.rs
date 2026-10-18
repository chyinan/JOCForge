// pattern: Functional Core
use crate::frontend::{PackageManifestEvidence, ProviderFileEvidence, ProviderSha256};
use serde_json::{Map, Value};

const PACKAGE_BYTES: u64 = 76_302;
const PACKAGE_SHA256: &str = "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";

const WINDOW_PATH: &str = "parameters/window-q15.json";
const WINDOW_BYTES: u64 = 2_704;
const WINDOW_SHA256: &str = "a12f9cbf91f1cf540c2eae77e4d236970defa70409f00ddf62ba87bfeafbb130";
const WINDOW_SCHEMA: &str = "generated-window-parameters.v1";
const WINDOW_RECORD_FIELDS: &[&str] = &[
    "parameter_id",
    "count",
    "representation",
    "q15_values",
    "generation",
    "to_F32",
    "F32LE_sha256",
    "origin",
    "all_words_research_comparison",
    "raw_asset_or_source_code_copied",
];
const WINDOW_GENERATION: &str = "RNE_integer(32768*sin((j+1/2)*pi/(2*N))^2)";
const WINDOW_TO_F32: &str = "R32(q15_value/32768), exact";
const WINDOW_ORIGIN: &str = "MATHEMATICAL_FORMULA_GENERATED_THIS_TASK";
const RECEIVER_BLEND32_F32LE_SHA256: &str =
    "3968b2666444f0395b008e8d8eb9d84eb2663b4d11199a486b8cbadcf5c142b2";
const COMPONENT_KERNEL64_F32LE_SHA256: &str =
    "19d2e9b70eae909b40027337bda0131132308eabe800cea731ee3e2c1d150b31";

const COMPONENT_PARAMETERS_PATH: &str = "parameters/component-parameter-mathematical-v1.json";
const COMPONENT_PARAMETERS_BYTES: u64 = 1_077;
const COMPONENT_PARAMETERS_SHA256: &str =
    "5d0fec6f62a33eeef5c64016760fcbe94a1872a564b6d0e03f959c95c37eb0f5";
const QMF_RECEIVER_FUNCTIONAL_PATH: &str = "parameters/qmf-receiver-functional-v1.json";
const QMF_RECEIVER_FUNCTIONAL_BYTES: u64 = 20_325;
const QMF_RECEIVER_FUNCTIONAL_SHA256: &str =
    "30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c";
const QMF_MATHEMATICAL_FFT_PATH: &str = "parameters/qmf-mathematical-fft-v1.json";
const QMF_MATHEMATICAL_FFT_BYTES: u64 = 6_983;
const QMF_MATHEMATICAL_FFT_SHA256: &str =
    "e8d3fb1514a5ffd857740222eacc8995208a29c3447e48c1b63083af368c0431";
const DESCRIPTOR_MATHEMATICAL_PATH: &str = "parameters/descriptor-mathematical.json";
const DESCRIPTOR_MATHEMATICAL_BYTES: u64 = 4_861;
const DESCRIPTOR_MATHEMATICAL_SHA256: &str =
    "a493fbdaf2dab30e940cae9976ed3305f3af083e5614ab0cbd6889c71a02b197";

#[derive(Debug, Eq, PartialEq)]
pub enum F04ParameterProviderError {
    ObservedManifestEvidenceMismatch,
    PackageManifestIdentityMismatch,
    PackageManifestSyntax,
    PackageManifestEntry,
    ObservedProviderEvidenceMismatch,
    ProviderIdentityMismatch,
    Utf8,
    JsonSyntax,
    Schema,
    Shape,
    Range,
    UnsupportedProfile,
}

#[derive(Clone, Copy)]
pub(crate) struct ProviderManifestEntry {
    path: &'static str,
    bytes: u64,
    sha256: &'static str,
}

/// Authenticated manifest entries for the two scoped F04 parameter providers.
pub struct ValidatedF04ParameterPackageManifest {
    manifest_sha256: ProviderSha256,
    window_q15: ProviderManifestEntry,
    component_parameters: ProviderManifestEntry,
    qmf_receiver_functional: ProviderManifestEntry,
    qmf_mathematical_fft: ProviderManifestEntry,
    descriptor_mathematical: ProviderManifestEntry,
}

impl ValidatedF04ParameterPackageManifest {
    pub const fn manifest_sha256(&self) -> ProviderSha256 {
        self.manifest_sha256
    }

    pub(crate) const fn window_q15_entry(&self) -> ProviderManifestEntry {
        self.window_q15
    }

    pub(crate) const fn component_parameters_entry(&self) -> ProviderManifestEntry {
        self.component_parameters
    }

    pub(crate) const fn qmf_receiver_functional_entry(&self) -> ProviderManifestEntry {
        self.qmf_receiver_functional
    }

    pub(crate) const fn qmf_mathematical_fft_entry(&self) -> ProviderManifestEntry {
        self.qmf_mathematical_fft
    }

    pub(crate) const fn descriptor_mathematical_entry(&self) -> ProviderManifestEntry {
        self.descriptor_mathematical
    }
}

/// Immutable lineage proving which manifest and provider entry were validated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct F04ParameterProviderLineage {
    manifest_sha256: ProviderSha256,
    provider_path: &'static str,
    manifest_entry_bytes: u64,
    manifest_entry_sha256: ProviderSha256,
}

impl F04ParameterProviderLineage {
    pub const fn manifest_sha256(&self) -> ProviderSha256 {
        self.manifest_sha256
    }

    pub const fn provider_path(&self) -> &'static str {
        self.provider_path
    }

    pub const fn manifest_entry_bytes(&self) -> u64 {
        self.manifest_entry_bytes
    }

    pub const fn manifest_entry_sha256(&self) -> ProviderSha256 {
        self.manifest_entry_sha256
    }
}

/// Immutable receipt for the authenticated Q15 window provider.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowQ15ProviderReceipt {
    validated_manifest_lineage: F04ParameterProviderLineage,
    provider_file_sha256: ProviderSha256,
    provider_file_bytes: u64,
    schema: &'static str,
    parameter_ids: [&'static str; 2],
}

impl WindowQ15ProviderReceipt {
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

    pub const fn parameter_ids(&self) -> &[&'static str; 2] {
        &self.parameter_ids
    }
}

/// Q15 window values copied from the identity-validated provider file.
pub struct ValidatedWindowQ15Provider {
    receiver_blend32: [u16; 32],
    component_kernel64: [u16; 64],
    receipt: WindowQ15ProviderReceipt,
}

impl ValidatedWindowQ15Provider {
    pub const fn receiver_blend32(&self) -> &[u16; 32] {
        &self.receiver_blend32
    }

    pub const fn component_kernel64(&self) -> &[u16; 64] {
        &self.component_kernel64
    }

    pub const fn receipt(&self) -> &WindowQ15ProviderReceipt {
        &self.receipt
    }
}

struct ParsedWindowQ15Provider {
    receiver_blend32: [u16; 32],
    component_kernel64: [u16; 64],
}

/// Verifies the complete approved package manifest before parsing any F04 entry.
pub fn validate_f04_parameter_package_manifest(
    evidence: &PackageManifestEvidence,
) -> Result<ValidatedF04ParameterPackageManifest, F04ParameterProviderError> {
    let actual_sha = ProviderSha256::digest(evidence.bytes());
    if actual_sha.to_hex() != PACKAGE_SHA256 {
        return Err(F04ParameterProviderError::PackageManifestIdentityMismatch);
    }
    if evidence.observed_byte_length() != evidence.bytes().len() as u64
        || evidence.observed_sha256() != actual_sha
        || evidence.bytes().len() as u64 != PACKAGE_BYTES
    {
        return Err(F04ParameterProviderError::ObservedManifestEvidenceMismatch);
    }

    let root: Value = serde_json::from_slice(evidence.bytes())
        .map_err(|_| F04ParameterProviderError::PackageManifestSyntax)?;
    let files = root
        .as_object()
        .and_then(|manifest| manifest.get("files"))
        .and_then(Value::as_array)
        .ok_or(F04ParameterProviderError::PackageManifestEntry)?;
    let window_q15 = unique_manifest_entry(files, WINDOW_PATH, WINDOW_BYTES, WINDOW_SHA256)?;
    let component_parameters = unique_manifest_entry(
        files,
        COMPONENT_PARAMETERS_PATH,
        COMPONENT_PARAMETERS_BYTES,
        COMPONENT_PARAMETERS_SHA256,
    )?;
    let qmf_receiver_functional = unique_manifest_entry(
        files,
        QMF_RECEIVER_FUNCTIONAL_PATH,
        QMF_RECEIVER_FUNCTIONAL_BYTES,
        QMF_RECEIVER_FUNCTIONAL_SHA256,
    )?;
    let qmf_mathematical_fft = unique_manifest_entry(
        files,
        QMF_MATHEMATICAL_FFT_PATH,
        QMF_MATHEMATICAL_FFT_BYTES,
        QMF_MATHEMATICAL_FFT_SHA256,
    )?;
    let descriptor_mathematical = unique_manifest_entry(
        files,
        DESCRIPTOR_MATHEMATICAL_PATH,
        DESCRIPTOR_MATHEMATICAL_BYTES,
        DESCRIPTOR_MATHEMATICAL_SHA256,
    )?;

    Ok(ValidatedF04ParameterPackageManifest {
        manifest_sha256: actual_sha,
        window_q15,
        component_parameters,
        qmf_receiver_functional,
        qmf_mathematical_fft,
        descriptor_mathematical,
    })
}

/// Validates both Q15 arrays after their manifest-bound bytes pass identity checks.
pub fn validate_window_q15_provider(
    evidence: &ProviderFileEvidence,
    manifest: &ValidatedF04ParameterPackageManifest,
) -> Result<ValidatedWindowQ15Provider, F04ParameterProviderError> {
    let entry = manifest.window_q15_entry();
    let provider_sha = validate_provider_identity(evidence, entry)?;
    let parsed = parse_window_q15_provider(evidence.bytes())?;
    let lineage = provider_lineage(manifest.manifest_sha256, entry, provider_sha);
    let receipt = WindowQ15ProviderReceipt {
        validated_manifest_lineage: lineage,
        provider_file_sha256: provider_sha,
        provider_file_bytes: evidence.bytes().len() as u64,
        schema: WINDOW_SCHEMA,
        parameter_ids: ["receiver_blend32", "component_kernel64"],
    };

    Ok(ValidatedWindowQ15Provider {
        receiver_blend32: parsed.receiver_blend32,
        component_kernel64: parsed.component_kernel64,
        receipt,
    })
}

pub(crate) fn validate_provider_identity(
    evidence: &ProviderFileEvidence,
    entry: ProviderManifestEntry,
) -> Result<ProviderSha256, F04ParameterProviderError> {
    let actual_sha = ProviderSha256::digest(evidence.bytes());
    if evidence.observed_byte_length() != evidence.bytes().len() as u64
        || evidence.observed_sha256() != actual_sha
    {
        return Err(F04ParameterProviderError::ObservedProviderEvidenceMismatch);
    }
    if evidence.bytes().len() as u64 != entry.bytes || actual_sha.to_hex() != entry.sha256 {
        return Err(F04ParameterProviderError::ProviderIdentityMismatch);
    }
    Ok(actual_sha)
}

pub(crate) fn provider_lineage(
    manifest_sha256: ProviderSha256,
    entry: ProviderManifestEntry,
    provider_sha256: ProviderSha256,
) -> F04ParameterProviderLineage {
    F04ParameterProviderLineage {
        manifest_sha256,
        provider_path: entry.path,
        manifest_entry_bytes: entry.bytes,
        manifest_entry_sha256: provider_sha256,
    }
}

fn unique_manifest_entry(
    files: &[Value],
    expected_path: &'static str,
    expected_bytes: u64,
    expected_sha256: &'static str,
) -> Result<ProviderManifestEntry, F04ParameterProviderError> {
    let mut matching = files.iter().filter(|entry| {
        entry
            .as_object()
            .and_then(|fields| fields.get("file"))
            .and_then(Value::as_str)
            == Some(expected_path)
    });
    let entry = matching
        .next()
        .and_then(Value::as_object)
        .ok_or(F04ParameterProviderError::PackageManifestEntry)?;
    if matching.next().is_some() || !exact_keys(entry, &["bytes", "file", "sha256"]) {
        return Err(F04ParameterProviderError::PackageManifestEntry);
    }
    if entry.get("file").and_then(Value::as_str) != Some(expected_path)
        || entry.get("bytes").and_then(Value::as_u64) != Some(expected_bytes)
        || entry.get("sha256").and_then(Value::as_str) != Some(expected_sha256)
    {
        return Err(F04ParameterProviderError::PackageManifestEntry);
    }
    Ok(ProviderManifestEntry {
        path: expected_path,
        bytes: expected_bytes,
        sha256: expected_sha256,
    })
}

fn parse_window_q15_provider(
    bytes: &[u8],
) -> Result<ParsedWindowQ15Provider, F04ParameterProviderError> {
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
            "product_role",
            "private_vendor_key_included",
            "parameters",
        ],
    ) {
        return Err(F04ParameterProviderError::Schema);
    }
    if string_field(fields, "schema")? != WINDOW_SCHEMA
        || string_field(fields, "product_role")? != "ALGORITHM_PARAMETER_NOT_SONG_LOOKUP"
        || bool_field(fields, "private_vendor_key_included")?
    {
        return Err(F04ParameterProviderError::Schema);
    }

    let parameters = fields
        .get("parameters")
        .and_then(Value::as_array)
        .ok_or(F04ParameterProviderError::Shape)?;
    if parameters.len() != 2 {
        return Err(F04ParameterProviderError::Shape);
    }

    let mut receiver_blend32 = None;
    let mut component_kernel64 = None;
    for parameter in parameters {
        let parameter_fields = parameter
            .as_object()
            .ok_or(F04ParameterProviderError::Shape)?;
        let id = string_field(parameter_fields, "parameter_id")?;
        match id {
            "receiver_blend32" if receiver_blend32.is_none() => {
                receiver_blend32 = Some(parse_window_record::<32>(
                    parameter_fields,
                    "receiver_blend32",
                    RECEIVER_BLEND32_F32LE_SHA256,
                )?);
            }
            "component_kernel64" if component_kernel64.is_none() => {
                component_kernel64 = Some(parse_window_record::<64>(
                    parameter_fields,
                    "component_kernel64",
                    COMPONENT_KERNEL64_F32LE_SHA256,
                )?);
            }
            _ => return Err(F04ParameterProviderError::Shape),
        }
    }

    Ok(ParsedWindowQ15Provider {
        receiver_blend32: receiver_blend32.ok_or(F04ParameterProviderError::Shape)?,
        component_kernel64: component_kernel64.ok_or(F04ParameterProviderError::Shape)?,
    })
}

fn parse_window_record<const N: usize>(
    fields: &Map<String, Value>,
    expected_id: &'static str,
    expected_f32le_sha256: &'static str,
) -> Result<[u16; N], F04ParameterProviderError> {
    if !exact_keys(fields, WINDOW_RECORD_FIELDS) {
        return Err(F04ParameterProviderError::Schema);
    }
    if string_field(fields, "parameter_id")? != expected_id
        || exact_u64_field(fields, "count")? != N as u64
        || string_field(fields, "representation")? != "q15_unsigned_integer"
        || string_field(fields, "generation")? != WINDOW_GENERATION
        || string_field(fields, "to_F32")? != WINDOW_TO_F32
        || string_field(fields, "F32LE_sha256")? != expected_f32le_sha256
        || string_field(fields, "origin")? != WINDOW_ORIGIN
        || string_field(fields, "all_words_research_comparison")? != "PASS"
        || bool_field(fields, "raw_asset_or_source_code_copied")?
    {
        return Err(F04ParameterProviderError::Schema);
    }
    q15_array::<N>(
        fields
            .get("q15_values")
            .ok_or(F04ParameterProviderError::Schema)?,
    )
}

pub(crate) fn exact_keys(fields: &Map<String, Value>, expected: &[&str]) -> bool {
    fields.len() == expected.len() && expected.iter().all(|key| fields.contains_key(*key))
}

pub(crate) fn string_field<'a>(
    fields: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a str, F04ParameterProviderError> {
    fields
        .get(name)
        .and_then(Value::as_str)
        .ok_or(F04ParameterProviderError::Schema)
}

pub(crate) fn bool_field(
    fields: &Map<String, Value>,
    name: &str,
) -> Result<bool, F04ParameterProviderError> {
    fields
        .get(name)
        .and_then(Value::as_bool)
        .ok_or(F04ParameterProviderError::Schema)
}

pub(crate) fn exact_u64_field(
    fields: &Map<String, Value>,
    name: &str,
) -> Result<u64, F04ParameterProviderError> {
    fields
        .get(name)
        .and_then(Value::as_u64)
        .ok_or(F04ParameterProviderError::Range)
}

pub(crate) fn q15_array<const N: usize>(
    value: &Value,
) -> Result<[u16; N], F04ParameterProviderError> {
    let values = value.as_array().ok_or(F04ParameterProviderError::Shape)?;
    if values.len() != N {
        return Err(F04ParameterProviderError::Shape);
    }
    let mut output = [0; N];
    for (output, value) in output.iter_mut().zip(values) {
        let value = value
            .as_u64()
            .filter(|value| *value <= 32_768)
            .ok_or(F04ParameterProviderError::Range)?;
        *output = value as u16;
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{F04ParameterProviderError, parse_window_q15_provider};
    use serde_json::Value;

    const WINDOW_Q15: &[u8] = include_bytes!("../../tests/fixtures/f04_parameters/window-q15.json");

    fn fixture() -> Value {
        serde_json::from_slice(WINDOW_Q15).expect("window fixture is valid JSON")
    }

    fn parse_after(
        mut document: Value,
        mutate: impl FnOnce(&mut Value),
    ) -> Result<(), F04ParameterProviderError> {
        mutate(&mut document);
        let bytes = serde_json::to_vec(&document).expect("mutated window document serializes");
        parse_window_q15_provider(&bytes).map(|_| ())
    }

    #[test]
    fn parses_the_two_identity_pinned_q15_records() {
        assert!(parse_window_q15_provider(WINDOW_Q15).is_ok());
    }

    #[test]
    fn rejects_unknown_top_level_fields() {
        assert!(matches!(
            parse_after(fixture(), |document| document["unexpected"] =
                Value::Bool(false)),
            Err(F04ParameterProviderError::Schema)
        ));
    }

    #[test]
    fn rejects_duplicate_parameter_ids() {
        assert!(matches!(
            parse_after(fixture(), |document| {
                document["parameters"][1]["parameter_id"] =
                    Value::String("receiver_blend32".to_owned());
            }),
            Err(F04ParameterProviderError::Shape)
        ));
    }

    #[test]
    fn rejects_wrong_q15_array_length() {
        assert!(matches!(
            parse_after(fixture(), |document| {
                document["parameters"][0]["q15_values"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }),
            Err(F04ParameterProviderError::Shape)
        ));
    }

    #[test]
    fn rejects_boolean_float_and_out_of_range_q15_values() {
        for invalid in [
            Value::Bool(true),
            serde_json::json!(1.0),
            serde_json::json!(32_769),
        ] {
            assert!(matches!(
                parse_after(fixture(), |document| {
                    document["parameters"][0]["q15_values"][0] = invalid;
                }),
                Err(F04ParameterProviderError::Range)
            ));
        }
    }

    #[test]
    fn rejects_generation_and_f32le_digest_metadata_drift() {
        assert!(matches!(
            parse_after(fixture(), |document| {
                document["parameters"][0]["generation"] = Value::String("platform libm".to_owned());
            }),
            Err(F04ParameterProviderError::Schema)
        ));
        assert!(matches!(
            parse_after(fixture(), |document| {
                document["parameters"][0]["F32LE_sha256"] = Value::String("ABC".to_owned());
            }),
            Err(F04ParameterProviderError::Schema)
        ));
    }
}
