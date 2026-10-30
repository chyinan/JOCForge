// pattern: Functional Core
//! Exact manifest bindings for the selected F01B/F03W asset bundles.

use super::{PackageManifestEvidence, ProviderFileEvidence, ProviderSha256};
use serde_json::{Map, Value};
use std::sync::Arc;

const APPROVED_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const PROVIDER_MANIFEST_SHA256: &str =
    "9b81b736e9dd110441d3b59333afac9cbb98886f8bf4ac9854b5d74bdf57783f";
const PROVIDER_SOURCE_MANIFEST_SHA256: &str =
    "51305803b05daff498935b3283e56e27bb3e3966f52727bcf803a226871d5e91";
const PROVIDER_CONTRACT_ENTRY: F03wManifestEntry = F03wManifestEntry {
    path: "CONTRACT.md",
    bytes: 1_251,
    sha256: "f158c952c0f52aa775601bb2fa8b0daabd2893bbf5b09a74e1c6dee14e3d92d7",
    role: Some("wash_scope_contract"),
};
const COLD_MANIFEST_SHA256: &str =
    "d15672b723f8091b576c2228ed7685744f522706d89078050b6486a72a068372";

const PROVIDER_SCOPE: &str = "all 48 approved provider assets for selected F01B/F03W installed48 implementation; runtime and dynamic geometry not claimed";
const COLD_SCOPE: &str = "approved selected profile cold schemas and Q15 kernel assets";

const PROVIDER_ENTRIES: [F03wManifestEntry; 3] = [
    F03wManifestEntry {
        path: "parameters/diffusion-mathematical-v1.json",
        bytes: 1_467,
        sha256: "d690512cd204e97fc04a35c03519ed3005e26cb57408432fe30bfde63c2319aa",
        role: Some("approved_parameter"),
    },
    F03wManifestEntry {
        path: "parameters/diffusion-constructor-mathematical-v1.json",
        bytes: 1_770,
        sha256: "7a710d68b00c14d6b9b3f9fa3313579c5739ccce408655ea3252ad1ee4acd106",
        role: Some("approved_parameter"),
    },
    F03wManifestEntry {
        path: "parameters/component-parameter-mathematical-v1.json",
        bytes: 1_077,
        sha256: "5d0fec6f62a33eeef5c64016760fcbe94a1872a564b6d0e03f959c95c37eb0f5",
        role: Some("approved_parameter"),
    },
];

const COLD_ENTRIES: [F03wManifestEntry; 5] = [
    F03wManifestEntry {
        path: "parameters/adm-installed48-state-v1.json",
        bytes: 7_237,
        sha256: "5138318de304b83d26e47271c1661bcc903cebcd2cea457ae98e0048b161ed46",
        role: None,
    },
    F03wManifestEntry {
        path: "parameters/f03-selected-target-cold-v1.json",
        bytes: 3_139,
        sha256: "a4b53854822a5f17a5413d6f2ded127a7366257c2a171377afcea4a84a3eaf00",
        role: None,
    },
    F03wManifestEntry {
        path: "parameters/selected-owner-projection-v1.json",
        bytes: 3_637,
        sha256: "b8e78dd1d15091e88877b2ead3e0608dbc1554c13afbcde46d5da24a2fd136dd",
        role: None,
    },
    F03wManifestEntry {
        path: "parameters/window-q15.json",
        bytes: 2_704,
        sha256: "a12f9cbf91f1cf540c2eae77e4d236970defa70409f00ddf62ba87bfeafbb130",
        role: None,
    },
    F03wManifestEntry {
        path: "parameters/f01e-selected-constructor-v1.json",
        bytes: 20_138,
        sha256: "b0548b0c45e321795455c68095e804de5507af318636676353a73f119a41f7cb",
        role: None,
    },
];

const COLD_CONTRACT_ENTRY: F03wManifestEntry = F03wManifestEntry {
    path: "CONTRACT.md",
    bytes: 664,
    sha256: "ad0b564931c057235daf5518bfd6b2d15c14cdaeb8e6d13399dc1f47cab8e5d2",
    role: None,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum F03wAssetManifestError {
    ObservedManifestEvidenceMismatch,
    ManifestIdentityMismatch,
    ManifestSyntax,
    ManifestShape,
    ManifestEntry,
    ObservedPayloadEvidenceMismatch,
    PayloadIdentityMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct F03wManifestEntry {
    path: &'static str,
    bytes: u64,
    sha256: &'static str,
    role: Option<&'static str>,
}

impl F03wManifestEntry {
    pub(crate) const fn path(self) -> &'static str {
        self.path
    }

    pub(crate) const fn bytes(self) -> u64 {
        self.bytes
    }

    pub(crate) const fn sha256(self) -> &'static str {
        self.sha256
    }
}

/// Identity for the three approved-bundle files consumed by currently typed F03W/F04 providers.
/// This is not the full provider index and does not close transitive provider consumption.
pub struct ValidatedF03wProviderManifest {
    manifest_sha256: ProviderSha256,
    entries: [F03wManifestEntry; 3],
}

/// Immutable identity for one consumed payload, retaining its exact provider-bundle lineage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct F03wProviderEntryLineage {
    manifest_sha256: ProviderSha256,
    path: &'static str,
    bytes: u64,
    payload_sha256: &'static str,
}

impl F03wProviderEntryLineage {
    pub const fn manifest_sha256(self) -> ProviderSha256 {
        self.manifest_sha256
    }

    pub const fn path(self) -> &'static str {
        self.path
    }

    pub const fn bytes(self) -> u64 {
        self.bytes
    }

    pub const fn payload_sha256_hex(self) -> &'static str {
        self.payload_sha256
    }
}

impl ValidatedF03wProviderManifest {
    pub const fn manifest_sha256(&self) -> ProviderSha256 {
        self.manifest_sha256
    }

    pub const fn approved_manifest_sha256(&self) -> &'static str {
        APPROVED_MANIFEST_SHA256
    }

    /// Declared installed48-index manifest identity retained from the F03W manifest.
    /// This loader does not read that index or require a third bundle root.
    pub const fn installed48_provider_index_sha256(&self) -> &'static str {
        PROVIDER_SOURCE_MANIFEST_SHA256
    }

    pub const fn declared_provider_entry_count(&self) -> usize {
        48
    }

    pub const fn validated_consumed_entry_count(&self) -> usize {
        self.entries.len()
    }

    pub const fn remaining_entry_count_open(&self) -> usize {
        45
    }

    pub const fn transitive_consumption_is_open(&self) -> bool {
        true
    }

    pub const fn is_runtime_provider_set(&self) -> bool {
        false
    }

    pub fn consumed_entry_lineages(&self) -> [F03wProviderEntryLineage; 3] {
        self.entries.map(|entry| F03wProviderEntryLineage {
            manifest_sha256: self.manifest_sha256,
            path: entry.path,
            bytes: entry.bytes,
            payload_sha256: entry.sha256,
        })
    }

    pub(crate) fn entry(&self, path: &str) -> Option<F03wManifestEntry> {
        self.entries
            .iter()
            .copied()
            .find(|entry| entry.path == path)
    }

    /// Binds exact bytes where no existing typed owner validator accepts this bundle manifest.
    pub fn bind_provider_asset(
        &self,
        evidence: &ProviderFileEvidence,
        path: &'static str,
    ) -> Result<HashBoundF03wProviderAsset, F03wAssetManifestError> {
        let entry = self
            .entry(path)
            .ok_or(F03wAssetManifestError::ManifestEntry)?;
        validate_payload_identity(evidence, entry)?;
        Ok(HashBoundF03wProviderAsset {
            bytes: Arc::from(evidence.bytes()),
            manifest_sha256: self.manifest_sha256,
            entry,
        })
    }
}

/// Immutable exact bytes for a provider entry without an adapter to its existing typed validator.
pub struct HashBoundF03wProviderAsset {
    bytes: Arc<[u8]>,
    manifest_sha256: ProviderSha256,
    entry: F03wManifestEntry,
}

impl HashBoundF03wProviderAsset {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub const fn manifest_sha256(&self) -> ProviderSha256 {
        self.manifest_sha256
    }

    pub const fn path(&self) -> &'static str {
        self.entry.path
    }

    pub const fn payload_bytes(&self) -> u64 {
        self.entry.bytes
    }

    pub const fn payload_sha256_hex(&self) -> &'static str {
        self.entry.sha256
    }
}

/// Identity for the five cold/state payloads in the separate F01B bundle.
pub struct ValidatedF01bColdAssetManifest {
    manifest_sha256: ProviderSha256,
    entries: [F03wManifestEntry; 5],
}

impl ValidatedF01bColdAssetManifest {
    pub const fn manifest_sha256(&self) -> ProviderSha256 {
        self.manifest_sha256
    }

    pub(crate) fn entry(&self, path: &str) -> Option<F03wManifestEntry> {
        self.entries
            .iter()
            .copied()
            .find(|entry| entry.path == path)
    }

    /// Binds an opaque schema asset to its exact cold-bundle entry without interpreting it.
    pub fn bind_schema_asset(
        &self,
        evidence: &ProviderFileEvidence,
        path: &'static str,
    ) -> Result<HashBoundF01bColdSchemaAsset, F03wAssetManifestError> {
        let entry = self
            .entry(path)
            .ok_or(F03wAssetManifestError::ManifestEntry)?;
        validate_payload_identity(evidence, entry)?;
        Ok(HashBoundF01bColdSchemaAsset {
            bytes: Arc::from(evidence.bytes()),
            manifest_sha256: self.manifest_sha256,
            entry,
        })
    }
}

/// Immutable hash-bound bytes for a cold schema with no existing typed owner validator.
pub struct HashBoundF01bColdSchemaAsset {
    bytes: Arc<[u8]>,
    manifest_sha256: ProviderSha256,
    entry: F03wManifestEntry,
}

impl HashBoundF01bColdSchemaAsset {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub const fn manifest_sha256(&self) -> ProviderSha256 {
        self.manifest_sha256
    }

    pub const fn path(&self) -> &'static str {
        self.entry.path
    }

    pub const fn payload_bytes(&self) -> u64 {
        self.entry.bytes
    }

    pub const fn payload_sha256_hex(&self) -> &'static str {
        self.entry.sha256
    }
}

/// Validates only the five entries currently consumed by typed selected F03W providers.
/// The SHA-256 pins the complete manifest bytes; the other 43 entries remain unconsumed here.
pub fn validate_f03w_provider_manifest(
    evidence: &PackageManifestEvidence,
) -> Result<ValidatedF03wProviderManifest, F03wAssetManifestError> {
    let manifest_sha256 = validate_manifest_evidence(evidence, PROVIDER_MANIFEST_SHA256)?;
    let root = parse_manifest(evidence.bytes())?;
    let fields = root
        .as_object()
        .ok_or(F03wAssetManifestError::ManifestShape)?;
    if !exact_keys(
        fields,
        &[
            "approved_package_manifest_sha256",
            "files",
            "package_id",
            "provider_manifest_sha256",
            "scope",
        ],
    ) || string_field(fields, "package_id")? != "ADM-F03W-APPROVED-PROVIDER-BUNDLE-V1"
        || string_field(fields, "scope")? != PROVIDER_SCOPE
        || string_field(fields, "approved_package_manifest_sha256")? != APPROVED_MANIFEST_SHA256
        || string_field(fields, "provider_manifest_sha256")? != PROVIDER_SOURCE_MANIFEST_SHA256
    {
        return Err(F03wAssetManifestError::ManifestShape);
    }
    let files = fields
        .get("files")
        .and_then(Value::as_array)
        .filter(|files| files.len() == 49)
        .ok_or(F03wAssetManifestError::ManifestShape)?;
    for expected in PROVIDER_ENTRIES
        .into_iter()
        .chain([PROVIDER_CONTRACT_ENTRY])
    {
        validate_manifest_entry(files, expected)?;
    }
    Ok(ValidatedF03wProviderManifest {
        manifest_sha256,
        entries: PROVIDER_ENTRIES,
    })
}

/// Validates the complete cold manifest identity and its five cold/state payload entries.
pub fn validate_f01b_cold_asset_manifest(
    evidence: &PackageManifestEvidence,
) -> Result<ValidatedF01bColdAssetManifest, F03wAssetManifestError> {
    let manifest_sha256 = validate_manifest_evidence(evidence, COLD_MANIFEST_SHA256)?;
    let root = parse_manifest(evidence.bytes())?;
    let fields = root
        .as_object()
        .ok_or(F03wAssetManifestError::ManifestShape)?;
    if !exact_keys(
        fields,
        &["approved_manifest_sha256", "files", "package_id", "scope"],
    ) || string_field(fields, "package_id")? != "F01B-F03W-COLD-STATE-ASSETS-V1"
        || string_field(fields, "scope")? != COLD_SCOPE
        || string_field(fields, "approved_manifest_sha256")? != APPROVED_MANIFEST_SHA256
    {
        return Err(F03wAssetManifestError::ManifestShape);
    }
    let files = fields
        .get("files")
        .and_then(Value::as_array)
        .filter(|files| files.len() == 6)
        .ok_or(F03wAssetManifestError::ManifestShape)?;
    for expected in COLD_ENTRIES.into_iter().chain([COLD_CONTRACT_ENTRY]) {
        validate_manifest_entry(files, expected)?;
    }
    Ok(ValidatedF01bColdAssetManifest {
        manifest_sha256,
        entries: COLD_ENTRIES,
    })
}

fn validate_manifest_evidence(
    evidence: &PackageManifestEvidence,
    expected_sha256: &str,
) -> Result<ProviderSha256, F03wAssetManifestError> {
    let actual = ProviderSha256::digest(evidence.bytes());
    if evidence.observed_byte_length() != evidence.bytes().len() as u64
        || evidence.observed_sha256() != actual
    {
        return Err(F03wAssetManifestError::ObservedManifestEvidenceMismatch);
    }
    if actual.to_hex() != expected_sha256 {
        return Err(F03wAssetManifestError::ManifestIdentityMismatch);
    }
    Ok(actual)
}

fn parse_manifest(bytes: &[u8]) -> Result<Value, F03wAssetManifestError> {
    serde_json::from_slice(bytes).map_err(|_| F03wAssetManifestError::ManifestSyntax)
}

fn validate_manifest_entry(
    files: &[Value],
    expected: F03wManifestEntry,
) -> Result<(), F03wAssetManifestError> {
    let mut matches = files.iter().filter(|value| {
        value
            .as_object()
            .and_then(|fields| fields.get("file"))
            .and_then(Value::as_str)
            == Some(expected.path)
    });
    let fields = matches
        .next()
        .and_then(Value::as_object)
        .ok_or(F03wAssetManifestError::ManifestEntry)?;
    if matches.next().is_some() {
        return Err(F03wAssetManifestError::ManifestEntry);
    }
    match expected.role {
        Some(role) => {
            if !exact_keys(fields, &["bytes", "file", "role", "sha256"])
                || string_field(fields, "role")? != role
            {
                return Err(F03wAssetManifestError::ManifestEntry);
            }
        }
        None => {
            if !exact_keys(fields, &["bytes", "file", "sha256"]) {
                return Err(F03wAssetManifestError::ManifestEntry);
            }
        }
    }
    if string_field(fields, "file")? != expected.path
        || u64_field(fields, "bytes")? != expected.bytes
        || string_field(fields, "sha256")? != expected.sha256
    {
        return Err(F03wAssetManifestError::ManifestEntry);
    }
    Ok(())
}

fn validate_payload_identity(
    evidence: &ProviderFileEvidence,
    entry: F03wManifestEntry,
) -> Result<(), F03wAssetManifestError> {
    let actual = ProviderSha256::digest(evidence.bytes());
    if evidence.observed_byte_length() != evidence.bytes().len() as u64
        || evidence.observed_sha256() != actual
    {
        return Err(F03wAssetManifestError::ObservedPayloadEvidenceMismatch);
    }
    if evidence.bytes().len() as u64 != entry.bytes || actual.to_hex() != entry.sha256 {
        return Err(F03wAssetManifestError::PayloadIdentityMismatch);
    }
    Ok(())
}

fn exact_keys(fields: &Map<String, Value>, keys: &[&str]) -> bool {
    fields.len() == keys.len() && keys.iter().all(|key| fields.contains_key(*key))
}

fn string_field<'a>(
    fields: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a str, F03wAssetManifestError> {
    fields
        .get(name)
        .and_then(Value::as_str)
        .ok_or(F03wAssetManifestError::ManifestShape)
}

fn u64_field(fields: &Map<String, Value>, name: &str) -> Result<u64, F03wAssetManifestError> {
    fields
        .get(name)
        .and_then(Value::as_u64)
        .ok_or(F03wAssetManifestError::ManifestShape)
}
