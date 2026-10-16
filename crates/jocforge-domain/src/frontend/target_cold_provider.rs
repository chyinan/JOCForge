// pattern: Functional Core
//! Identity-first validation for the approved F03 selected-target cold provider.

use super::{PackageManifestEvidence, ProviderFileEvidence, ProviderSha256};
use serde_json::{Map, Value};

const PACKAGE_SHA256: &str = "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const PROVIDER_PATH: &str = "parameters/f03-selected-target-cold-v1.json";
const PROVIDER_BYTES: u64 = 3139;
const PROVIDER_SHA256: &str = "a4b53854822a5f17a5413d6f2ded127a7366257c2a171377afcea4a84a3eaf00";
const SCHEMA_ID: &str = "f03-selected-target-cold.v1";
const PROJECTION_BYTES: usize = 630;
const PROJECTION_SHA256: &str = "b1acfa3ce43b6ce987836330a3db3945c81b3d62d96d4973f18b302dc6d2606c";

const TOP_LEVEL_FIELDS: &[&str] = &[
    "schema",
    "profile_dimensions",
    "provider_kind",
    "current_xyz_u32",
    "current_id_u32",
    "current_attribute_u32",
    "previous_xyz_u32",
    "previous_id_u32",
    "previous_attribute_u32",
    "active_u8",
    "previous_marker_active_u8",
    "source",
    "private_key_required",
    "evidence",
    "local_review",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum F03TargetProviderError {
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
    TargetProjectionMismatch,
}

/// Opaque proof that the complete approved package manifest and its unique F03 entry were checked.
pub struct ValidatedF03PackageManifest {
    manifest_sha256: ProviderSha256,
    provider_path: &'static str,
    provider_bytes: u64,
    provider_sha256: &'static str,
}

/// Immutable, typed lineage for the provider file authorized by a validated package manifest.
pub struct F03TargetProviderLineage {
    manifest_sha256: ProviderSha256,
    provider_path: &'static str,
    provider_bytes: u64,
    provider_sha256: ProviderSha256,
}

impl F03TargetProviderLineage {
    pub const fn manifest_sha256(&self) -> ProviderSha256 {
        self.manifest_sha256
    }

    pub const fn provider_path(&self) -> &'static str {
        self.provider_path
    }

    pub const fn provider_bytes(&self) -> u64 {
        self.provider_bytes
    }

    pub const fn provider_sha256(&self) -> ProviderSha256 {
        self.provider_sha256
    }
}

/// Immutable receipt for the target-only 630-byte projection, not a spatial-control root digest.
pub struct F03TargetProviderReceipt {
    validated_manifest_lineage: F03TargetProviderLineage,
    provider_file_sha256: ProviderSha256,
    provider_file_bytes: u64,
    schema_id: &'static str,
    target_projection_sha256: ProviderSha256,
    target_projection_bytes: u64,
    provider_generation: u64,
}

impl F03TargetProviderReceipt {
    pub const fn validated_manifest_lineage(&self) -> &F03TargetProviderLineage {
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

    /// SHA-256 of the target-only projection; this is not the complete spatial-control root digest.
    pub const fn target_projection_sha256(&self) -> ProviderSha256 {
        self.target_projection_sha256
    }

    /// Byte length of the target-only projection; this is not a complete root-state serialization.
    pub const fn target_projection_bytes(&self) -> u64 {
        self.target_projection_bytes
    }

    pub const fn provider_generation(&self) -> u64 {
        self.provider_generation
    }
}

/// Opaque, read-only F03 cold arrays. Current and previous arrays are independent storage.
pub struct ValidatedF03TargetColdProvider {
    current_xyz_u32: [u32; 45],
    current_id_u32: [u32; 15],
    current_attribute_u32: [u32; 15],
    previous_xyz_u32: [u32; 45],
    previous_id_u32: [u32; 15],
    previous_attribute_u32: [u32; 15],
    active_u8: [u8; 15],
    previous_marker_active_u8: [u8; 15],
    target_projection: Vec<u8>,
    receipt: F03TargetProviderReceipt,
}

impl ValidatedF03TargetColdProvider {
    pub const fn current_xyz_u32(&self) -> &[u32; 45] {
        &self.current_xyz_u32
    }

    pub const fn current_id_u32(&self) -> &[u32; 15] {
        &self.current_id_u32
    }

    pub const fn current_attribute_u32(&self) -> &[u32; 15] {
        &self.current_attribute_u32
    }

    pub const fn previous_xyz_u32(&self) -> &[u32; 45] {
        &self.previous_xyz_u32
    }

    pub const fn previous_id_u32(&self) -> &[u32; 15] {
        &self.previous_id_u32
    }

    pub const fn previous_attribute_u32(&self) -> &[u32; 15] {
        &self.previous_attribute_u32
    }

    pub const fn active_u8(&self) -> &[u8; 15] {
        &self.active_u8
    }

    pub const fn previous_marker_active_u8(&self) -> &[u8; 15] {
        &self.previous_marker_active_u8
    }

    pub fn target_projection_bytes(&self) -> &[u8] {
        &self.target_projection
    }

    pub const fn receipt(&self) -> &F03TargetProviderReceipt {
        &self.receipt
    }
}

struct F03TargetArrays {
    current_xyz_u32: [u32; 45],
    current_id_u32: [u32; 15],
    current_attribute_u32: [u32; 15],
    previous_xyz_u32: [u32; 45],
    previous_id_u32: [u32; 15],
    previous_attribute_u32: [u32; 15],
    active_u8: [u8; 15],
    previous_marker_active_u8: [u8; 15],
}

pub fn validate_f03_package_manifest(
    evidence: &PackageManifestEvidence,
) -> Result<ValidatedF03PackageManifest, F03TargetProviderError> {
    let actual_manifest_sha = ProviderSha256::digest(evidence.bytes());
    if actual_manifest_sha.to_hex() != PACKAGE_SHA256 {
        return Err(F03TargetProviderError::PackageManifestIdentityMismatch);
    }
    if evidence.observed_byte_length() != evidence.bytes().len() as u64
        || evidence.observed_sha256() != actual_manifest_sha
    {
        return Err(F03TargetProviderError::ObservedManifestEvidenceMismatch);
    }

    let (provider_path, provider_bytes, provider_sha256) =
        parse_f03_manifest_entry(evidence.bytes())?;
    Ok(ValidatedF03PackageManifest {
        manifest_sha256: actual_manifest_sha,
        provider_path,
        provider_bytes,
        provider_sha256,
    })
}

fn parse_f03_manifest_entry(
    bytes: &[u8],
) -> Result<(&'static str, u64, &'static str), F03TargetProviderError> {
    let root: Value =
        serde_json::from_slice(bytes).map_err(|_| F03TargetProviderError::PackageManifestSyntax)?;
    let files = root
        .as_object()
        .and_then(|manifest| manifest.get("files"))
        .and_then(Value::as_array)
        .ok_or(F03TargetProviderError::PackageManifestEntry)?;
    let mut matches = files.iter().filter(|entry| {
        entry
            .as_object()
            .and_then(|entry| entry.get("file"))
            .and_then(Value::as_str)
            == Some(PROVIDER_PATH)
    });
    let entry = matches
        .next()
        .and_then(Value::as_object)
        .ok_or(F03TargetProviderError::PackageManifestEntry)?;
    if matches.next().is_some() {
        return Err(F03TargetProviderError::PackageManifestEntry);
    }
    exact_keys(entry, &["bytes", "file", "sha256"])
        .map_err(|_| F03TargetProviderError::PackageManifestEntry)?;
    if string(entry, "file").map_err(|_| F03TargetProviderError::PackageManifestEntry)?
        != PROVIDER_PATH
        || u64_value(entry, "bytes").map_err(|_| F03TargetProviderError::PackageManifestEntry)?
            != PROVIDER_BYTES
        || string(entry, "sha256").map_err(|_| F03TargetProviderError::PackageManifestEntry)?
            != PROVIDER_SHA256
    {
        return Err(F03TargetProviderError::PackageManifestEntry);
    }
    Ok((PROVIDER_PATH, PROVIDER_BYTES, PROVIDER_SHA256))
}

pub fn validate_f03_target_cold_provider(
    evidence: &ProviderFileEvidence,
    manifest: &ValidatedF03PackageManifest,
) -> Result<ValidatedF03TargetColdProvider, F03TargetProviderError> {
    let actual_provider_sha = ProviderSha256::digest(evidence.bytes());
    if evidence.observed_byte_length() != evidence.bytes().len() as u64
        || evidence.observed_sha256() != actual_provider_sha
    {
        return Err(F03TargetProviderError::ObservedProviderEvidenceMismatch);
    }
    if manifest.provider_path != PROVIDER_PATH
        || manifest.provider_bytes != evidence.bytes().len() as u64
        || manifest.provider_sha256 != PROVIDER_SHA256
        || actual_provider_sha.to_hex() != manifest.provider_sha256
    {
        return Err(F03TargetProviderError::ProviderIdentityMismatch);
    }

    let arrays = parse_f03_provider(evidence.bytes())?;
    let target_projection = write_target_projection(&arrays);
    let target_projection_sha = ProviderSha256::digest(&target_projection);
    if target_projection.len() != PROJECTION_BYTES
        || target_projection_sha.to_hex() != PROJECTION_SHA256
    {
        return Err(F03TargetProviderError::TargetProjectionMismatch);
    }

    let lineage = F03TargetProviderLineage {
        manifest_sha256: manifest.manifest_sha256,
        provider_path: manifest.provider_path,
        provider_bytes: manifest.provider_bytes,
        provider_sha256: actual_provider_sha,
    };
    let receipt = F03TargetProviderReceipt {
        validated_manifest_lineage: lineage,
        provider_file_sha256: actual_provider_sha,
        provider_file_bytes: evidence.bytes().len() as u64,
        schema_id: SCHEMA_ID,
        target_projection_sha256: target_projection_sha,
        target_projection_bytes: target_projection.len() as u64,
        provider_generation: 0,
    };

    Ok(ValidatedF03TargetColdProvider {
        current_xyz_u32: arrays.current_xyz_u32,
        current_id_u32: arrays.current_id_u32,
        current_attribute_u32: arrays.current_attribute_u32,
        previous_xyz_u32: arrays.previous_xyz_u32,
        previous_id_u32: arrays.previous_id_u32,
        previous_attribute_u32: arrays.previous_attribute_u32,
        active_u8: arrays.active_u8,
        previous_marker_active_u8: arrays.previous_marker_active_u8,
        target_projection,
        receipt,
    })
}

fn parse_f03_provider(bytes: &[u8]) -> Result<F03TargetArrays, F03TargetProviderError> {
    std::str::from_utf8(bytes).map_err(|_| F03TargetProviderError::Utf8)?;
    let root: Value =
        serde_json::from_slice(bytes).map_err(|_| F03TargetProviderError::JsonSyntax)?;
    let fields = root.as_object().ok_or(F03TargetProviderError::Schema)?;
    exact_keys(fields, TOP_LEVEL_FIELDS).map_err(|_| F03TargetProviderError::Schema)?;
    if string(fields, "schema")? != SCHEMA_ID
        || string(fields, "provider_kind")? != "exact_parseable_functional_constant"
        || fields.get("private_key_required") != Some(&Value::Bool(false))
        || fields.get("source").and_then(Value::as_str).is_none()
        || fields.get("evidence").and_then(Value::as_object).is_none()
        || fields
            .get("local_review")
            .and_then(Value::as_object)
            .is_none()
    {
        return Err(F03TargetProviderError::Schema);
    }

    let dimensions = fields
        .get("profile_dimensions")
        .and_then(Value::as_object)
        .ok_or(F03TargetProviderError::Shape)?;
    exact_keys(
        dimensions,
        &[
            "source_count_N",
            "feature_bands_B",
            "target_capacity_T",
            "axes",
            "active_target_count_n",
        ],
    )?;
    if u64_value(dimensions, "source_count_N")? != 48
        || u64_value(dimensions, "feature_bands_B")? != 24
        || u64_value(dimensions, "target_capacity_T")? != 15
        || u64_value(dimensions, "axes")? != 3
        || string(dimensions, "active_target_count_n")?
            != "runtime integer 0..15; arrays allocate T=15 and consume prefix [0:n]"
    {
        return Err(F03TargetProviderError::Shape);
    }

    let arrays = F03TargetArrays {
        current_xyz_u32: u32_array(fields, "current_xyz_u32")?,
        current_id_u32: u32_array(fields, "current_id_u32")?,
        current_attribute_u32: u32_array(fields, "current_attribute_u32")?,
        previous_xyz_u32: u32_array(fields, "previous_xyz_u32")?,
        previous_id_u32: u32_array(fields, "previous_id_u32")?,
        previous_attribute_u32: u32_array(fields, "previous_attribute_u32")?,
        active_u8: u8_array(fields, "active_u8")?,
        previous_marker_active_u8: u8_array(fields, "previous_marker_active_u8")?,
    };

    if arrays.current_xyz_u32 != [0; 45]
        || arrays.current_id_u32 != [u32::MAX; 15]
        || arrays.current_attribute_u32 != [0x0000_0b00; 15]
        || arrays.previous_xyz_u32 != [0; 45]
        || arrays.previous_id_u32 != [u32::MAX; 15]
        || arrays.previous_attribute_u32 != [0x0000_0b00; 15]
        || arrays.active_u8 != [0; 15]
        || arrays.previous_marker_active_u8 != [0; 15]
    {
        return Err(F03TargetProviderError::Shape);
    }
    Ok(arrays)
}

fn u32_array<const N: usize>(
    fields: &Map<String, Value>,
    name: &str,
) -> Result<[u32; N], F03TargetProviderError> {
    let values = fields
        .get(name)
        .and_then(Value::as_array)
        .ok_or(F03TargetProviderError::Shape)?;
    if values.len() != N {
        return Err(F03TargetProviderError::Shape);
    }
    let mut result = [0; N];
    for (destination, value) in result.iter_mut().zip(values) {
        let value = value.as_u64().ok_or(F03TargetProviderError::Shape)?;
        *destination = u32::try_from(value).map_err(|_| F03TargetProviderError::Shape)?;
    }
    Ok(result)
}

fn u8_array<const N: usize>(
    fields: &Map<String, Value>,
    name: &str,
) -> Result<[u8; N], F03TargetProviderError> {
    let values = fields
        .get(name)
        .and_then(Value::as_array)
        .ok_or(F03TargetProviderError::Shape)?;
    if values.len() != N {
        return Err(F03TargetProviderError::Shape);
    }
    let mut result = [0; N];
    for (destination, value) in result.iter_mut().zip(values) {
        let value = value.as_u64().ok_or(F03TargetProviderError::Shape)?;
        *destination = u8::try_from(value).map_err(|_| F03TargetProviderError::Shape)?;
    }
    Ok(result)
}

fn exact_keys(map: &Map<String, Value>, keys: &[&str]) -> Result<(), F03TargetProviderError> {
    if map.len() == keys.len() && keys.iter().all(|key| map.contains_key(*key)) {
        Ok(())
    } else {
        Err(F03TargetProviderError::Shape)
    }
}

fn string<'a>(map: &'a Map<String, Value>, key: &str) -> Result<&'a str, F03TargetProviderError> {
    map.get(key)
        .and_then(Value::as_str)
        .ok_or(F03TargetProviderError::Shape)
}

fn u64_value(map: &Map<String, Value>, key: &str) -> Result<u64, F03TargetProviderError> {
    map.get(key)
        .and_then(Value::as_u64)
        .ok_or(F03TargetProviderError::Shape)
}

fn write_target_projection(arrays: &F03TargetArrays) -> Vec<u8> {
    let mut output = Vec::with_capacity(PROJECTION_BYTES);
    for value in arrays
        .current_xyz_u32
        .iter()
        .chain(arrays.current_id_u32.iter())
        .chain(arrays.current_attribute_u32.iter())
        .chain(arrays.previous_xyz_u32.iter())
        .chain(arrays.previous_id_u32.iter())
        .chain(arrays.previous_attribute_u32.iter())
    {
        output.extend_from_slice(&value.to_le_bytes());
    }
    output.extend_from_slice(&arrays.active_u8);
    output.extend_from_slice(&arrays.previous_marker_active_u8);
    output
}

#[cfg(test)]
mod tests {
    use super::{
        F03TargetProviderError, PROVIDER_BYTES, PROVIDER_PATH, PROVIDER_SHA256,
        parse_f03_manifest_entry, parse_f03_provider,
    };

    #[test]
    fn manifest_entry_lookup_requires_one_unique_exact_f03_path() {
        let manifest = format!(
            r#"{{"files":[{{"file":"{PROVIDER_PATH}","bytes":{PROVIDER_BYTES},"sha256":"{PROVIDER_SHA256}"}},{{"file":"{PROVIDER_PATH}","bytes":{PROVIDER_BYTES},"sha256":"{PROVIDER_SHA256}"}}]}}"#
        );
        assert!(matches!(
            parse_f03_manifest_entry(manifest.as_bytes()),
            Err(F03TargetProviderError::PackageManifestEntry)
        ));
    }

    #[test]
    fn exact_pinned_payload_parses_to_fixed_cold_arrays() {
        let payload =
            include_bytes!("../../tests/fixtures/target_cold/f03-selected-target-cold-v1.json");
        let arrays = parse_f03_provider(payload).unwrap();
        assert_eq!(arrays.current_id_u32, [u32::MAX; 15]);
        assert_eq!(arrays.current_attribute_u32, [0x0000_0b00; 15]);
        assert_eq!(arrays.active_u8, [0; 15]);
        assert_eq!(arrays.previous_marker_active_u8, [0; 15]);
    }

    #[test]
    fn target_parser_rejects_unknown_fields_and_invalid_widths_or_cold_words() {
        let payload =
            include_str!("../../tests/fixtures/target_cold/f03-selected-target-cold-v1.json");
        let mut unknown: serde_json::Value = serde_json::from_str(payload).unwrap();
        unknown
            .as_object_mut()
            .unwrap()
            .insert("extra".into(), serde_json::Value::Null);
        assert!(matches!(
            parse_f03_provider(unknown.to_string().as_bytes()),
            Err(F03TargetProviderError::Schema)
        ));

        let mut active_width: serde_json::Value = serde_json::from_str(payload).unwrap();
        active_width["active_u8"][0] = serde_json::json!(256);
        assert!(matches!(
            parse_f03_provider(active_width.to_string().as_bytes()),
            Err(F03TargetProviderError::Shape)
        ));

        let mut cold_attribute: serde_json::Value = serde_json::from_str(payload).unwrap();
        cold_attribute["current_attribute_u32"][0] = serde_json::json!(0);
        assert!(matches!(
            parse_f03_provider(cold_attribute.to_string().as_bytes()),
            Err(F03TargetProviderError::Shape)
        ));

        let mut active_count: serde_json::Value = serde_json::from_str(payload).unwrap();
        active_count["profile_dimensions"]["active_target_count_n"] = serde_json::json!(15);
        assert!(matches!(
            parse_f03_provider(active_count.to_string().as_bytes()),
            Err(F03TargetProviderError::Shape)
        ));
    }

    #[test]
    fn target_parser_rejects_nonpositive_zero_and_invalid_u32_values() {
        let payload =
            include_str!("../../tests/fixtures/target_cold/f03-selected-target-cold-v1.json");
        let mut signed_zero: serde_json::Value = serde_json::from_str(payload).unwrap();
        signed_zero["previous_xyz_u32"][0] = serde_json::json!(2147483648_u64);
        assert!(matches!(
            parse_f03_provider(signed_zero.to_string().as_bytes()),
            Err(F03TargetProviderError::Shape)
        ));

        let mut out_of_range: serde_json::Value = serde_json::from_str(payload).unwrap();
        out_of_range["current_id_u32"][0] = serde_json::json!(4294967296_u64);
        assert!(matches!(
            parse_f03_provider(out_of_range.to_string().as_bytes()),
            Err(F03TargetProviderError::Shape)
        ));

        let mut warm_id: serde_json::Value = serde_json::from_str(payload).unwrap();
        warm_id["current_id_u32"][0] = serde_json::json!(47);
        assert!(matches!(
            parse_f03_provider(warm_id.to_string().as_bytes()),
            Err(F03TargetProviderError::Shape)
        ));
    }
}
