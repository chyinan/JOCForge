// pattern: Functional Core
//! Exact, identity-first providers for the selected CurrentFeatures math route.

use super::{PackageManifestEvidence, ProviderFileEvidence, ProviderSha256};
use serde_json::{Map, Value};
use std::collections::HashSet;

const PACKAGE_BYTES: u64 = 76_302;
const PACKAGE_SHA256: &str = "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";

const PROCESSOR_PATH: &str = "parameters/feature-processor-math-v1.json";
const PROCESSOR_BYTES: u64 = 35_441;
const PROCESSOR_SHA256: &str = "c2cc80eed698285a7a022324bfed36e04c3b6e682cb21674b6316b3ec6a2ec0b";
const PROCESSOR_SCHEMA: &str = "feature-processor-math-parameters.v1";
const PROCESSOR_SOURCE: &str = "MATHEMATICAL_L3072_FS48000_GENERATION";
const PROCESSOR_FIELDS: &[&str] = &[
    "schema",
    "source",
    "window_length",
    "sample_rate",
    "fft_length",
    "component_count",
    "vector_count",
    "coefficient_u32",
    "window_counts",
    "fast_decay_u32",
    "slow_retain_u32",
    "real_ADM_new_route_acceptance",
];
const WINDOW_COUNTS: [u32; 24] = [
    4, 5, 6, 8, 9, 11, 14, 17, 20, 25, 30, 37, 45, 54, 66, 80, 97, 118, 144, 175, 213, 259, 315,
    296,
];

const FFT_PATH: &str = "parameters/feature-fft4096-math-v1.json";
const FFT_BYTES: u64 = 279_305;
const FFT_SHA256: &str = "aea85e4fb5dcb0354ac11790466d324f1020a8bf5db3f9d981bc46755751fb33";
const FFT_SCHEMA: &str = "feature-fft4096-math-parameters.v1";
const FFT_ROUTE: &str = "REAL_FFT4096_RADIX2_MATH_V1";
const FFT_SOURCE: &str = "MATHEMATICAL_ROOT_GENERATION_NO_COMPILED_TABLE_INPUT";
const FFT_FIELDS: &[&str] = &[
    "schema",
    "route",
    "source",
    "fft_length",
    "stages",
    "real_ADM_new_route_acceptance",
];
const FFT_STAGE_FIELDS: &[&str] = &["length", "roots_u32"];
const FFT_STAGE_LENGTHS: [u32; 12] = [2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

#[derive(Debug, Eq, PartialEq)]
pub enum CurrentFeaturesProviderError {
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
    RouteMismatch,
}

#[derive(Clone, Copy)]
struct ProviderManifestEntry {
    path: &'static str,
    bytes: u64,
    sha256: &'static str,
}

/// Proof that the approved package manifest and both unique provider entries were validated.
pub struct ValidatedCurrentFeaturesPackageManifest {
    manifest_sha256: ProviderSha256,
    processor: ProviderManifestEntry,
    fft: ProviderManifestEntry,
}

/// Immutable identity lineage for one math provider entry in the approved package manifest.
pub struct CurrentFeaturesProviderLineage {
    manifest_sha256: ProviderSha256,
    provider_path: &'static str,
    manifest_entry_bytes: u64,
    manifest_entry_sha256: ProviderSha256,
}

impl CurrentFeaturesProviderLineage {
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

/// Immutable receipt for the exact feature-processor math provider file.
pub struct FeatureProcessorMathProviderReceipt {
    validated_manifest_lineage: CurrentFeaturesProviderLineage,
    provider_file_sha256: ProviderSha256,
    provider_file_bytes: u64,
    schema_id: &'static str,
}

impl FeatureProcessorMathProviderReceipt {
    pub const fn validated_manifest_lineage(&self) -> &CurrentFeaturesProviderLineage {
        &self.validated_manifest_lineage
    }

    pub const fn provider_file_sha256(&self) -> ProviderSha256 {
        self.provider_file_sha256
    }

    pub const fn provider_file_bytes(&self) -> u64 {
        self.provider_file_bytes
    }

    pub const fn schema_id(&self) -> &'static str {
        self.schema_id
    }
}

/// Immutable receipt for the exact selected radix-2 FFT roots provider file.
pub struct FeatureFft4096MathProviderReceipt {
    validated_manifest_lineage: CurrentFeaturesProviderLineage,
    provider_file_sha256: ProviderSha256,
    provider_file_bytes: u64,
    schema_id: &'static str,
    route: FeatureFft4096MathRoute,
}

impl FeatureFft4096MathProviderReceipt {
    pub const fn validated_manifest_lineage(&self) -> &CurrentFeaturesProviderLineage {
        &self.validated_manifest_lineage
    }

    pub const fn provider_file_sha256(&self) -> ProviderSha256 {
        self.provider_file_sha256
    }

    pub const fn provider_file_bytes(&self) -> u64 {
        self.provider_file_bytes
    }

    pub const fn schema_id(&self) -> &'static str {
        self.schema_id
    }

    pub const fn route(&self) -> FeatureFft4096MathRoute {
        self.route
    }
}

/// Opaque, immutable coefficient and scalar values from the authenticated processor asset.
pub struct ValidatedFeatureProcessorMathProvider {
    window_length: u32,
    sample_rate: u32,
    fft_length: u32,
    component_count: u32,
    vector_count: u32,
    coefficient_u32: [u32; 2048],
    window_counts: [u32; 24],
    fast_decay_u32: u32,
    slow_retain_u32: u32,
    receipt: FeatureProcessorMathProviderReceipt,
}

impl ValidatedFeatureProcessorMathProvider {
    pub const fn window_length(&self) -> u32 {
        self.window_length
    }

    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub const fn fft_length(&self) -> u32 {
        self.fft_length
    }

    pub const fn component_count(&self) -> u32 {
        self.component_count
    }

    pub const fn vector_count(&self) -> u32 {
        self.vector_count
    }

    pub const fn coefficient_u32(&self) -> &[u32; 2048] {
        &self.coefficient_u32
    }

    pub const fn window_counts(&self) -> &[u32; 24] {
        &self.window_counts
    }

    pub const fn fast_decay_u32(&self) -> u32 {
        self.fast_decay_u32
    }

    pub const fn slow_retain_u32(&self) -> u32 {
        self.slow_retain_u32
    }

    pub const fn receipt(&self) -> &FeatureProcessorMathProviderReceipt {
        &self.receipt
    }
}

/// The only admitted route carried by the authenticated FFT provider.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FeatureFft4096MathRoute {
    RealFft4096Radix2MathV1,
}

/// One validated radix-2 stage; roots remain the original provider F32 bit words.
pub struct ValidatedFeatureFft4096Stage {
    length: u32,
    roots_u32: Vec<[u32; 2]>,
}

impl ValidatedFeatureFft4096Stage {
    pub const fn length(&self) -> u32 {
        self.length
    }

    pub fn roots_u32(&self) -> &[[u32; 2]] {
        &self.roots_u32
    }
}

/// Opaque, immutable radix-2 roots loaded from the authenticated FFT provider asset.
pub struct ValidatedFeatureFft4096MathProvider {
    fft_length: u32,
    stages: Vec<ValidatedFeatureFft4096Stage>,
    receipt: FeatureFft4096MathProviderReceipt,
}

impl ValidatedFeatureFft4096MathProvider {
    pub const fn fft_length(&self) -> u32 {
        self.fft_length
    }

    pub const fn route(&self) -> FeatureFft4096MathRoute {
        self.receipt.route()
    }

    pub fn stages(&self) -> &[ValidatedFeatureFft4096Stage] {
        &self.stages
    }

    pub const fn receipt(&self) -> &FeatureFft4096MathProviderReceipt {
        &self.receipt
    }
}

/// Authenticates the complete package manifest and its unique entries for both exact providers.
pub fn validate_current_features_package_manifest(
    evidence: &PackageManifestEvidence,
) -> Result<ValidatedCurrentFeaturesPackageManifest, CurrentFeaturesProviderError> {
    let actual_manifest_sha = ProviderSha256::digest(evidence.bytes());
    if actual_manifest_sha.to_hex() != PACKAGE_SHA256 {
        return Err(CurrentFeaturesProviderError::PackageManifestIdentityMismatch);
    }
    if evidence.observed_byte_length() != evidence.bytes().len() as u64
        || evidence.observed_sha256() != actual_manifest_sha
        || evidence.bytes().len() as u64 != PACKAGE_BYTES
    {
        return Err(CurrentFeaturesProviderError::ObservedManifestEvidenceMismatch);
    }

    let root: Value = serde_json::from_slice(evidence.bytes())
        .map_err(|_| CurrentFeaturesProviderError::PackageManifestSyntax)?;
    let files = root
        .as_object()
        .and_then(|manifest| manifest.get("files"))
        .and_then(Value::as_array)
        .ok_or(CurrentFeaturesProviderError::PackageManifestEntry)?;
    let processor =
        unique_manifest_entry(files, PROCESSOR_PATH, PROCESSOR_BYTES, PROCESSOR_SHA256)?;
    let fft = unique_manifest_entry(files, FFT_PATH, FFT_BYTES, FFT_SHA256)?;

    Ok(ValidatedCurrentFeaturesPackageManifest {
        manifest_sha256: actual_manifest_sha,
        processor,
        fft,
    })
}

/// Validates the exact processor asset after its bytes match the authenticated manifest entry.
pub fn validate_feature_processor_math_provider(
    evidence: &ProviderFileEvidence,
    manifest: &ValidatedCurrentFeaturesPackageManifest,
) -> Result<ValidatedFeatureProcessorMathProvider, CurrentFeaturesProviderError> {
    let actual_sha = validate_provider_identity(evidence, manifest.processor)?;
    let parsed = parse_feature_processor_math_provider(evidence.bytes())?;
    let lineage = provider_lineage(manifest.manifest_sha256, manifest.processor, actual_sha);
    let receipt = FeatureProcessorMathProviderReceipt {
        validated_manifest_lineage: lineage,
        provider_file_sha256: actual_sha,
        provider_file_bytes: evidence.bytes().len() as u64,
        schema_id: PROCESSOR_SCHEMA,
    };

    Ok(ValidatedFeatureProcessorMathProvider {
        window_length: parsed.window_length,
        sample_rate: parsed.sample_rate,
        fft_length: parsed.fft_length,
        component_count: parsed.component_count,
        vector_count: parsed.vector_count,
        coefficient_u32: parsed.coefficient_u32,
        window_counts: parsed.window_counts,
        fast_decay_u32: parsed.fast_decay_u32,
        slow_retain_u32: parsed.slow_retain_u32,
        receipt,
    })
}

/// Validates the exact FFT asset and selected route after byte identity succeeds.
pub fn validate_feature_fft4096_math_provider(
    evidence: &ProviderFileEvidence,
    manifest: &ValidatedCurrentFeaturesPackageManifest,
) -> Result<ValidatedFeatureFft4096MathProvider, CurrentFeaturesProviderError> {
    let actual_sha = validate_provider_identity(evidence, manifest.fft)?;
    let parsed = parse_feature_fft4096_math_provider(evidence.bytes())?;
    let lineage = provider_lineage(manifest.manifest_sha256, manifest.fft, actual_sha);
    let route = FeatureFft4096MathRoute::RealFft4096Radix2MathV1;
    let receipt = FeatureFft4096MathProviderReceipt {
        validated_manifest_lineage: lineage,
        provider_file_sha256: actual_sha,
        provider_file_bytes: evidence.bytes().len() as u64,
        schema_id: FFT_SCHEMA,
        route,
    };

    Ok(ValidatedFeatureFft4096MathProvider {
        fft_length: parsed.fft_length,
        stages: parsed.stages,
        receipt,
    })
}

fn unique_manifest_entry(
    files: &[Value],
    expected_path: &'static str,
    expected_bytes: u64,
    expected_sha256: &'static str,
) -> Result<ProviderManifestEntry, CurrentFeaturesProviderError> {
    let mut matches = files.iter().filter(|entry| {
        entry
            .as_object()
            .and_then(|entry| entry.get("file"))
            .and_then(Value::as_str)
            == Some(expected_path)
    });
    let entry = matches
        .next()
        .and_then(Value::as_object)
        .ok_or(CurrentFeaturesProviderError::PackageManifestEntry)?;
    if matches.next().is_some() || !exact_keys(entry, &["bytes", "file", "sha256"]) {
        return Err(CurrentFeaturesProviderError::PackageManifestEntry);
    }
    if entry.get("file").and_then(Value::as_str) != Some(expected_path)
        || entry.get("bytes").and_then(Value::as_u64) != Some(expected_bytes)
        || entry.get("sha256").and_then(Value::as_str) != Some(expected_sha256)
    {
        return Err(CurrentFeaturesProviderError::PackageManifestEntry);
    }
    Ok(ProviderManifestEntry {
        path: expected_path,
        bytes: expected_bytes,
        sha256: expected_sha256,
    })
}

fn validate_provider_identity(
    evidence: &ProviderFileEvidence,
    entry: ProviderManifestEntry,
) -> Result<ProviderSha256, CurrentFeaturesProviderError> {
    let actual_sha = ProviderSha256::digest(evidence.bytes());
    if evidence.observed_byte_length() != evidence.bytes().len() as u64
        || evidence.observed_sha256() != actual_sha
    {
        return Err(CurrentFeaturesProviderError::ObservedProviderEvidenceMismatch);
    }
    if evidence.bytes().len() as u64 != entry.bytes || actual_sha.to_hex() != entry.sha256 {
        return Err(CurrentFeaturesProviderError::ProviderIdentityMismatch);
    }
    Ok(actual_sha)
}

fn provider_lineage(
    manifest_sha256: ProviderSha256,
    entry: ProviderManifestEntry,
    provider_sha256: ProviderSha256,
) -> CurrentFeaturesProviderLineage {
    CurrentFeaturesProviderLineage {
        manifest_sha256,
        provider_path: entry.path,
        manifest_entry_bytes: entry.bytes,
        manifest_entry_sha256: provider_sha256,
    }
}

struct ParsedFeatureProcessorMathProvider {
    window_length: u32,
    sample_rate: u32,
    fft_length: u32,
    component_count: u32,
    vector_count: u32,
    coefficient_u32: [u32; 2048],
    window_counts: [u32; 24],
    fast_decay_u32: u32,
    slow_retain_u32: u32,
}

fn parse_feature_processor_math_provider(
    bytes: &[u8],
) -> Result<ParsedFeatureProcessorMathProvider, CurrentFeaturesProviderError> {
    let document = provider_document(bytes)?;
    let fields = document
        .as_object()
        .ok_or(CurrentFeaturesProviderError::Schema)?;
    if !exact_keys(fields, PROCESSOR_FIELDS) {
        return Err(CurrentFeaturesProviderError::Schema);
    }
    if string(fields, "schema")? != PROCESSOR_SCHEMA
        || string(fields, "source")? != PROCESSOR_SOURCE
        || string(fields, "real_ADM_new_route_acceptance")? != "NOT_RUN"
    {
        return Err(CurrentFeaturesProviderError::Schema);
    }

    let window_length = exact_dimension(fields, "window_length", 3072)?;
    let sample_rate = exact_dimension(fields, "sample_rate", 48_000)?;
    let fft_length = exact_dimension(fields, "fft_length", 4096)?;
    let component_count = exact_dimension(fields, "component_count", 2048)?;
    let vector_count = exact_dimension(fields, "vector_count", 24)?;
    let coefficient_u32 = u32_array::<2048>(
        fields
            .get("coefficient_u32")
            .ok_or(CurrentFeaturesProviderError::Schema)?,
    )?;
    let window_counts = u32_array::<24>(
        fields
            .get("window_counts")
            .ok_or(CurrentFeaturesProviderError::Schema)?,
    )?;
    if window_counts != WINDOW_COUNTS {
        return Err(CurrentFeaturesProviderError::Schema);
    }
    let fast_decay_u32 = u32_field(fields, "fast_decay_u32")?;
    let slow_retain_u32 = u32_field(fields, "slow_retain_u32")?;
    if fast_decay_u32 != 0x3F06_FCA3 || slow_retain_u32 != 0x3F4E_D1AB {
        return Err(CurrentFeaturesProviderError::Schema);
    }

    Ok(ParsedFeatureProcessorMathProvider {
        window_length,
        sample_rate,
        fft_length,
        component_count,
        vector_count,
        coefficient_u32,
        window_counts,
        fast_decay_u32,
        slow_retain_u32,
    })
}

struct ParsedFeatureFft4096MathProvider {
    fft_length: u32,
    stages: Vec<ValidatedFeatureFft4096Stage>,
}

fn parse_feature_fft4096_math_provider(
    bytes: &[u8],
) -> Result<ParsedFeatureFft4096MathProvider, CurrentFeaturesProviderError> {
    let document = provider_document(bytes)?;
    let fields = document
        .as_object()
        .ok_or(CurrentFeaturesProviderError::Schema)?;
    if !exact_keys(fields, FFT_FIELDS) {
        return Err(CurrentFeaturesProviderError::Schema);
    }
    if string(fields, "schema")? != FFT_SCHEMA
        || string(fields, "source")? != FFT_SOURCE
        || string(fields, "real_ADM_new_route_acceptance")? != "NOT_RUN"
    {
        return Err(CurrentFeaturesProviderError::Schema);
    }
    if string(fields, "route")? != FFT_ROUTE {
        return Err(CurrentFeaturesProviderError::RouteMismatch);
    }
    let fft_length = exact_dimension(fields, "fft_length", 4096)?;
    let stage_values = fields
        .get("stages")
        .and_then(Value::as_array)
        .ok_or(CurrentFeaturesProviderError::Shape)?;
    if stage_values.len() != FFT_STAGE_LENGTHS.len() {
        return Err(CurrentFeaturesProviderError::Shape);
    }

    let mut stages = Vec::with_capacity(FFT_STAGE_LENGTHS.len());
    let mut total_pairs = 0usize;
    for (index, expected_length) in FFT_STAGE_LENGTHS.iter().copied().enumerate() {
        let stage = stage_values[index]
            .as_object()
            .ok_or(CurrentFeaturesProviderError::Shape)?;
        if !exact_keys(stage, FFT_STAGE_FIELDS) {
            return Err(CurrentFeaturesProviderError::Schema);
        }
        let length = u32_field(stage, "length")?;
        if length != expected_length {
            return Err(CurrentFeaturesProviderError::Shape);
        }
        let roots = stage
            .get("roots_u32")
            .and_then(Value::as_array)
            .ok_or(CurrentFeaturesProviderError::Shape)?;
        if roots.len() != (length / 2) as usize {
            return Err(CurrentFeaturesProviderError::Shape);
        }
        let mut roots_u32 = Vec::with_capacity(roots.len());
        for pair in roots {
            let pair = pair.as_array().ok_or(CurrentFeaturesProviderError::Shape)?;
            if pair.len() != 2 {
                return Err(CurrentFeaturesProviderError::Shape);
            }
            roots_u32.push([u32_value(&pair[0])?, u32_value(&pair[1])?]);
        }
        total_pairs += roots_u32.len();
        stages.push(ValidatedFeatureFft4096Stage { length, roots_u32 });
    }
    if total_pairs != 4095 || total_pairs * 2 != 8190 {
        return Err(CurrentFeaturesProviderError::Shape);
    }

    Ok(ParsedFeatureFft4096MathProvider { fft_length, stages })
}

fn provider_document(bytes: &[u8]) -> Result<Value, CurrentFeaturesProviderError> {
    std::str::from_utf8(bytes).map_err(|_| CurrentFeaturesProviderError::Utf8)?;
    let value: Value =
        serde_json::from_slice(bytes).map_err(|_| CurrentFeaturesProviderError::JsonSyntax)?;
    if !json_object_keys_are_unique(bytes).map_err(|()| CurrentFeaturesProviderError::JsonSyntax)? {
        return Err(CurrentFeaturesProviderError::Schema);
    }
    Ok(value)
}

fn json_object_keys_are_unique(bytes: &[u8]) -> Result<bool, ()> {
    let mut cursor = 0;
    let mut unique = true;
    scan_json_value(bytes, &mut cursor, &mut unique)?;
    skip_json_whitespace(bytes, &mut cursor);
    if cursor != bytes.len() {
        return Err(());
    }
    Ok(unique)
}

fn scan_json_value(bytes: &[u8], cursor: &mut usize, unique: &mut bool) -> Result<(), ()> {
    skip_json_whitespace(bytes, cursor);
    match bytes.get(*cursor).copied().ok_or(())? {
        b'{' => {
            *cursor += 1;
            skip_json_whitespace(bytes, cursor);
            let mut keys = HashSet::new();
            if bytes.get(*cursor) == Some(&b'}') {
                *cursor += 1;
                return Ok(());
            }
            loop {
                skip_json_whitespace(bytes, cursor);
                let key_end = scan_json_string(bytes, *cursor).ok_or(())?;
                let key: String =
                    serde_json::from_slice(&bytes[*cursor..key_end]).map_err(|_| ())?;
                if !keys.insert(key) {
                    *unique = false;
                }
                *cursor = key_end;
                skip_json_whitespace(bytes, cursor);
                if bytes.get(*cursor) != Some(&b':') {
                    return Err(());
                }
                *cursor += 1;
                scan_json_value(bytes, cursor, unique)?;
                skip_json_whitespace(bytes, cursor);
                match bytes.get(*cursor) {
                    Some(b',') => *cursor += 1,
                    Some(b'}') => {
                        *cursor += 1;
                        return Ok(());
                    }
                    _ => return Err(()),
                }
            }
        }
        b'[' => {
            *cursor += 1;
            skip_json_whitespace(bytes, cursor);
            if bytes.get(*cursor) == Some(&b']') {
                *cursor += 1;
                return Ok(());
            }
            loop {
                scan_json_value(bytes, cursor, unique)?;
                skip_json_whitespace(bytes, cursor);
                match bytes.get(*cursor) {
                    Some(b',') => *cursor += 1,
                    Some(b']') => {
                        *cursor += 1;
                        return Ok(());
                    }
                    _ => return Err(()),
                }
            }
        }
        b'"' => {
            *cursor = scan_json_string(bytes, *cursor).ok_or(())?;
            Ok(())
        }
        _ => {
            let start = *cursor;
            while let Some(byte) = bytes.get(*cursor) {
                if matches!(byte, b',' | b']' | b'}' | b' ' | b'\n' | b'\r' | b'\t') {
                    break;
                }
                *cursor += 1;
            }
            if *cursor == start { Err(()) } else { Ok(()) }
        }
    }
}

fn scan_json_string(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start) != Some(&b'"') {
        return None;
    }
    let mut cursor = start + 1;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => cursor = cursor.checked_add(2)?,
            b'"' => return Some(cursor + 1),
            _ => cursor += 1,
        }
    }
    None
}

fn skip_json_whitespace(bytes: &[u8], cursor: &mut usize) {
    while matches!(bytes.get(*cursor), Some(b' ' | b'\n' | b'\r' | b'\t')) {
        *cursor += 1;
    }
}

fn exact_keys(fields: &Map<String, Value>, expected: &[&str]) -> bool {
    fields.len() == expected.len() && expected.iter().all(|field| fields.contains_key(*field))
}

fn string<'a>(
    fields: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a str, CurrentFeaturesProviderError> {
    fields
        .get(field)
        .and_then(Value::as_str)
        .ok_or(CurrentFeaturesProviderError::Schema)
}

fn unsigned(value: &Value) -> Result<u64, CurrentFeaturesProviderError> {
    value.as_u64().ok_or(CurrentFeaturesProviderError::Shape)
}

fn u32_value(value: &Value) -> Result<u32, CurrentFeaturesProviderError> {
    u32::try_from(unsigned(value)?).map_err(|_| CurrentFeaturesProviderError::Shape)
}

fn u32_field(
    fields: &Map<String, Value>,
    field: &str,
) -> Result<u32, CurrentFeaturesProviderError> {
    u32_value(
        fields
            .get(field)
            .ok_or(CurrentFeaturesProviderError::Schema)?,
    )
}

fn exact_dimension(
    fields: &Map<String, Value>,
    field: &str,
    expected: u32,
) -> Result<u32, CurrentFeaturesProviderError> {
    let value = u32_field(fields, field)?;
    if value != expected {
        return Err(CurrentFeaturesProviderError::Schema);
    }
    Ok(value)
}

fn u32_array<const N: usize>(value: &Value) -> Result<[u32; N], CurrentFeaturesProviderError> {
    let values = value
        .as_array()
        .ok_or(CurrentFeaturesProviderError::Shape)?;
    if values.len() != N {
        return Err(CurrentFeaturesProviderError::Shape);
    }
    let parsed: Vec<u32> = values.iter().map(u32_value).collect::<Result<_, _>>()?;
    parsed
        .try_into()
        .map_err(|_| CurrentFeaturesProviderError::Shape)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROCESSOR_FIXTURE_BYTES: &[u8] =
        include_bytes!("../../tests/fixtures/current_features/feature-processor-math-v1.json");
    const FFT_FIXTURE_BYTES: &[u8] =
        include_bytes!("../../tests/fixtures/current_features/feature-fft4096-math-v1.json");

    fn processor_value() -> Value {
        serde_json::from_slice(PROCESSOR_FIXTURE_BYTES).unwrap()
    }

    fn fft_value() -> Value {
        serde_json::from_slice(FFT_FIXTURE_BYTES).unwrap()
    }

    fn json_bytes(value: &Value) -> Vec<u8> {
        serde_json::to_vec(value).unwrap()
    }

    #[test]
    fn processor_parser_rejects_unknown_and_duplicate_keys() {
        let mut unknown = processor_value();
        unknown["not_registered"] = Value::Null;
        assert_eq!(
            parse_feature_processor_math_provider(&json_bytes(&unknown)).err(),
            Some(CurrentFeaturesProviderError::Schema)
        );

        let mut duplicate = br#"{"schema":"feature-processor-math-parameters.v1", "#.to_vec();
        duplicate.extend_from_slice(&PROCESSOR_FIXTURE_BYTES[1..]);
        assert_eq!(
            parse_feature_processor_math_provider(&duplicate).err(),
            Some(CurrentFeaturesProviderError::Schema)
        );

        let mut wrong_schema = processor_value();
        wrong_schema["schema"] = Value::from("feature-processor-math-parameters.v2");
        assert_eq!(
            parse_feature_processor_math_provider(&json_bytes(&wrong_schema)).err(),
            Some(CurrentFeaturesProviderError::Schema)
        );
    }

    #[test]
    fn processor_parser_rejects_boolean_float_wrong_dimension_and_counts() {
        let mut boolean = processor_value();
        boolean["window_length"] = Value::Bool(true);
        assert_eq!(
            parse_feature_processor_math_provider(&json_bytes(&boolean)).err(),
            Some(CurrentFeaturesProviderError::Shape)
        );

        let mut float = processor_value();
        float["window_length"] = serde_json::Number::from_f64(3072.0)
            .map(Value::Number)
            .unwrap();
        assert_eq!(
            parse_feature_processor_math_provider(&json_bytes(&float)).err(),
            Some(CurrentFeaturesProviderError::Shape)
        );

        let mut dimension = processor_value();
        dimension["window_length"] = Value::from(3071);
        assert_eq!(
            parse_feature_processor_math_provider(&json_bytes(&dimension)).err(),
            Some(CurrentFeaturesProviderError::Schema)
        );

        let mut counts = processor_value();
        counts["window_counts"][0] = Value::from(5);
        assert_eq!(
            parse_feature_processor_math_provider(&json_bytes(&counts)).err(),
            Some(CurrentFeaturesProviderError::Schema)
        );
    }

    #[test]
    fn processor_parser_rejects_wrong_array_shapes_and_u32_domain() {
        let mut short_coefficients = processor_value();
        short_coefficients["coefficient_u32"]
            .as_array_mut()
            .unwrap()
            .pop();
        assert_eq!(
            parse_feature_processor_math_provider(&json_bytes(&short_coefficients)).err(),
            Some(CurrentFeaturesProviderError::Shape)
        );

        let mut overflow = processor_value();
        overflow["coefficient_u32"][0] = Value::from(u64::from(u32::MAX) + 1);
        assert_eq!(
            parse_feature_processor_math_provider(&json_bytes(&overflow)).err(),
            Some(CurrentFeaturesProviderError::Shape)
        );

        let mut boolean_word = processor_value();
        boolean_word["coefficient_u32"][0] = Value::Bool(false);
        assert_eq!(
            parse_feature_processor_math_provider(&json_bytes(&boolean_word)).err(),
            Some(CurrentFeaturesProviderError::Shape)
        );
    }

    #[test]
    fn fft_parser_rejects_route_stage_and_root_shape_drift() {
        let mut route = fft_value();
        route["route"] = Value::from("OLD_SIMD_FFT4096");
        assert_eq!(
            parse_feature_fft4096_math_provider(&json_bytes(&route)).err(),
            Some(CurrentFeaturesProviderError::RouteMismatch)
        );

        let mut wrong_schema = fft_value();
        wrong_schema["schema"] = Value::from("feature-fft4096-math-parameters.v2");
        assert_eq!(
            parse_feature_fft4096_math_provider(&json_bytes(&wrong_schema)).err(),
            Some(CurrentFeaturesProviderError::Schema)
        );

        let mut stage_length = fft_value();
        stage_length["stages"][0]["length"] = Value::from(4);
        assert_eq!(
            parse_feature_fft4096_math_provider(&json_bytes(&stage_length)).err(),
            Some(CurrentFeaturesProviderError::Shape)
        );

        let mut stage_count = fft_value();
        stage_count["stages"].as_array_mut().unwrap().pop();
        assert_eq!(
            parse_feature_fft4096_math_provider(&json_bytes(&stage_count)).err(),
            Some(CurrentFeaturesProviderError::Shape)
        );

        let mut pair_count = fft_value();
        pair_count["stages"][0]["roots_u32"]
            .as_array_mut()
            .unwrap()
            .pop();
        assert_eq!(
            parse_feature_fft4096_math_provider(&json_bytes(&pair_count)).err(),
            Some(CurrentFeaturesProviderError::Shape)
        );

        let mut pair_width = fft_value();
        pair_width["stages"][0]["roots_u32"][0]
            .as_array_mut()
            .unwrap()
            .pop();
        assert_eq!(
            parse_feature_fft4096_math_provider(&json_bytes(&pair_width)).err(),
            Some(CurrentFeaturesProviderError::Shape)
        );

        let mut unknown_stage_field = fft_value();
        unknown_stage_field["stages"][0]["extra"] = Value::Null;
        assert_eq!(
            parse_feature_fft4096_math_provider(&json_bytes(&unknown_stage_field)).err(),
            Some(CurrentFeaturesProviderError::Schema)
        );
    }

    #[test]
    fn fft_parser_rejects_boolean_and_out_of_range_root_words() {
        let mut boolean_word = fft_value();
        boolean_word["stages"][0]["roots_u32"][0][0] = Value::Bool(true);
        assert_eq!(
            parse_feature_fft4096_math_provider(&json_bytes(&boolean_word)).err(),
            Some(CurrentFeaturesProviderError::Shape)
        );

        let mut overflow = fft_value();
        overflow["stages"][0]["roots_u32"][0][0] = Value::from(u64::from(u32::MAX) + 1);
        assert_eq!(
            parse_feature_fft4096_math_provider(&json_bytes(&overflow)).err(),
            Some(CurrentFeaturesProviderError::Shape)
        );
    }

    #[test]
    fn manifest_provider_entries_must_be_unique_and_exact() {
        let entry: Value = serde_json::from_str(
            r#"{"file":"parameters/feature-processor-math-v1.json","bytes":35441,"sha256":"c2cc80eed698285a7a022324bfed36e04c3b6e682cb21674b6316b3ec6a2ec0b"}"#,
        )
        .unwrap();
        let files = vec![entry.clone()];
        assert!(
            unique_manifest_entry(&files, PROCESSOR_PATH, PROCESSOR_BYTES, PROCESSOR_SHA256)
                .is_ok()
        );

        let duplicate = vec![entry.clone(), entry];
        assert_eq!(
            unique_manifest_entry(
                &duplicate,
                PROCESSOR_PATH,
                PROCESSOR_BYTES,
                PROCESSOR_SHA256
            )
            .err(),
            Some(CurrentFeaturesProviderError::PackageManifestEntry)
        );
    }
}
