// pattern: Functional Core
//! Identity-first validation for the approved F01E cold provider.

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::sync::Arc;

const PROVIDER_BYTES: u64 = 20_138;
const PROVIDER_SHA: [u8; 32] = [
    0xb0, 0x54, 0x8b, 0x0c, 0x45, 0xe3, 0x21, 0x79, 0x54, 0x55, 0xc6, 0x80, 0x95, 0xe8, 0x04, 0xde,
    0x55, 0x07, 0xaf, 0x31, 0x86, 0x36, 0x67, 0x63, 0x53, 0xa7, 0x3f, 0x11, 0x9a, 0x41, 0xf7, 0xcb,
];
const PACKAGE_SHA: [u8; 32] = [
    0xc7, 0xcd, 0xdb, 0x7d, 0x5a, 0xc0, 0x73, 0xc0, 0xf4, 0x10, 0xfe, 0x44, 0xbe, 0x9a, 0x43, 0x1c,
    0xfb, 0xb2, 0x1c, 0x3d, 0xf4, 0x69, 0xd8, 0xb4, 0x3c, 0x32, 0xa4, 0x29, 0x72, 0x98, 0x67, 0x4a,
];
const COLD_SHA: [u8; 32] = [
    0x9c, 0x94, 0x38, 0x79, 0xcd, 0x36, 0xcb, 0x45, 0xbc, 0x75, 0x38, 0x34, 0x6c, 0xfd, 0xf4, 0xf7,
    0x1d, 0x09, 0x98, 0xd8, 0xf2, 0x61, 0xf7, 0x5c, 0x13, 0x50, 0x45, 0xfc, 0xb5, 0x98, 0x70, 0xaa,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderSha256([u8; 32]);
impl ProviderSha256 {
    pub fn digest(bytes: &[u8]) -> Self {
        let d = Sha256::digest(bytes);
        let mut v = [0; 32];
        v.copy_from_slice(&d);
        Self(v)
    }
    pub fn to_hex(self) -> String {
        self.0
            .iter()
            .fold(String::with_capacity(64), |mut hex, byte| {
                write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
                hex
            })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderIdentity {
    provider_id: &'static str,
    schema_id: &'static str,
    manifest_path: &'static str,
    manifest_bytes: u64,
    manifest_sha: ProviderSha256,
    package_sha: ProviderSha256,
    delivery_scope: &'static str,
}
impl ProviderIdentity {
    pub const fn f01e_selected() -> Self {
        Self {
            provider_id: "F01E_SELECTED_COLD_V1",
            schema_id: "f01e-selected-constructor.v1",
            manifest_path: "parameters/f01e-selected-constructor-v1.json",
            manifest_bytes: PROVIDER_BYTES,
            manifest_sha: ProviderSha256(PROVIDER_SHA),
            package_sha: ProviderSha256(PACKAGE_SHA),
            delivery_scope: "INTERNAL_RESEARCH_AND_INTERNAL_IMPLEMENTATION_ONLY",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProviderFileEvidence {
    bytes: Arc<[u8]>,
    diagnostic_path: Option<String>,
    observed_byte_length: u64,
    observed_sha256: ProviderSha256,
}

#[derive(Clone, Debug)]
pub struct PackageManifestEvidence {
    bytes: Arc<[u8]>,
    diagnostic_path: Option<String>,
    observed_byte_length: u64,
    observed_sha256: ProviderSha256,
}
impl PackageManifestEvidence {
    pub fn new(bytes: Arc<[u8]>, diagnostic_path: Option<String>) -> Self {
        let observed_byte_length = bytes.len() as u64;
        let observed_sha256 = ProviderSha256::digest(&bytes);
        Self {
            bytes,
            diagnostic_path,
            observed_byte_length,
            observed_sha256,
        }
    }
    pub fn from_observed(
        bytes: Arc<[u8]>,
        diagnostic_path: Option<String>,
        observed_byte_length: u64,
        observed_sha256: ProviderSha256,
    ) -> Self {
        Self {
            bytes,
            diagnostic_path,
            observed_byte_length,
            observed_sha256,
        }
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn diagnostic_path(&self) -> Option<&str> {
        self.diagnostic_path.as_deref()
    }
    pub const fn observed_byte_length(&self) -> u64 {
        self.observed_byte_length
    }
    pub const fn observed_sha256(&self) -> ProviderSha256 {
        self.observed_sha256
    }
}

#[derive(Clone, Debug)]
pub struct ValidatedF01EPackageManifest {
    manifest_sha256: ProviderSha256,
    provider_path: &'static str,
    provider_bytes: u64,
    provider_sha256: ProviderSha256,
}
impl ProviderFileEvidence {
    pub fn new(bytes: Arc<[u8]>, diagnostic_path: Option<String>) -> Self {
        let len = bytes.len() as u64;
        let sha = ProviderSha256::digest(&bytes);
        Self {
            bytes,
            diagnostic_path,
            observed_byte_length: len,
            observed_sha256: sha,
        }
    }
    pub fn from_observed(
        bytes: Arc<[u8]>,
        diagnostic_path: Option<String>,
        observed_byte_length: u64,
        observed_sha256: ProviderSha256,
    ) -> Self {
        Self {
            bytes,
            diagnostic_path,
            observed_byte_length,
            observed_sha256,
        }
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn diagnostic_path(&self) -> Option<&str> {
        self.diagnostic_path.as_deref()
    }
    pub const fn observed_byte_length(&self) -> u64 {
        self.observed_byte_length
    }
    pub const fn observed_sha256(&self) -> ProviderSha256 {
        self.observed_sha256
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum F01EProviderError {
    ObservedManifestEvidenceMismatch,
    PackageManifestIdentityMismatch,
    PackageManifestSyntax,
    PackageManifestEntry,
    ObservedEvidenceMismatch,
    ProviderIdentityMismatch,
    Utf8,
    JsonSyntax,
    Schema,
    Shape,
    CanonicalPreimage,
    CanonicalDigest,
}

#[derive(Clone, Copy, Debug)]
pub struct F01EProviderLineage {
    expected_identity: ProviderIdentity,
    manifest_sha256: ProviderSha256,
}

#[derive(Clone, Debug)]
pub(super) struct ObjectCold {
    pub(super) ordinal: u32,
}
#[derive(Clone, Debug)]
pub(super) struct NormalizedConfig {
    pub(super) sample_rate_hz: u32,
    pub(super) channels: u32,
    pub(super) pcm_bits: u32,
    pub(super) source_samples: u64,
    pub(super) bit_rate_kbps: u32,
    pub(super) bit_rate_enum: u32,
    pub(super) layout_enum: u32,
    pub(super) target_capacity: u32,
    pub(super) frame_samples: u32,
    pub(super) spatial_config_mode: u32,
    pub(super) dispatch_mode: u32,
    pub(super) virtual_prefix_count: u32,
    pub(super) bed_count: u32,
    pub(super) object_count: u32,
    pub(super) event_start_samples: u64,
    pub(super) line_drc: String,
    pub(super) rf_drc: String,
    pub(super) time_base: String,
}
#[derive(Clone, Debug)]
pub(super) struct F01ECold {
    pub(super) callback_index: u64,
    pub(super) normal_callbacks: u64,
    pub(super) remaining_cached_drains: u32,
    pub(super) objects: Vec<ObjectCold>,
    pub(super) config: NormalizedConfig,
}

#[derive(Clone, Debug)]
pub struct F01EProviderReceipt {
    lineage: F01EProviderLineage,
    provider_file_sha256: ProviderSha256,
    provider_file_bytes: u64,
    schema_id: &'static str,
    schema_version: u32,
    value_pointer: &'static str,
    canonical_sha256: ProviderSha256,
    canonical_bytes: u64,
    provider_generation: u64,
}
impl F01EProviderReceipt {
    pub const fn provider_identity(&self) -> ProviderIdentity {
        self.lineage.expected_identity
    }
    pub const fn manifest_sha256(&self) -> ProviderSha256 {
        self.lineage.manifest_sha256
    }
    pub const fn provider_file_sha256(&self) -> ProviderSha256 {
        self.provider_file_sha256
    }
    pub const fn provider_file_bytes(&self) -> u64 {
        self.provider_file_bytes
    }
    pub const fn canonical_sha256(&self) -> ProviderSha256 {
        self.canonical_sha256
    }
    pub const fn schema_id(&self) -> &'static str {
        self.schema_id
    }
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }
    pub const fn value_pointer(&self) -> &'static str {
        self.value_pointer
    }
    pub const fn canonical_bytes(&self) -> u64 {
        self.canonical_bytes
    }
    pub const fn provider_generation(&self) -> u64 {
        self.provider_generation
    }
}

pub fn validate_f01e_package_manifest(
    evidence: &PackageManifestEvidence,
) -> Result<ValidatedF01EPackageManifest, F01EProviderError> {
    let actual = ProviderSha256::digest(evidence.bytes());
    if evidence.observed_byte_length != evidence.bytes.len() as u64
        || evidence.observed_sha256 != actual
    {
        return Err(F01EProviderError::ObservedManifestEvidenceMismatch);
    }
    if actual != ProviderSha256(PACKAGE_SHA) {
        return Err(F01EProviderError::PackageManifestIdentityMismatch);
    }
    let entry = parse_f01e_manifest_entry(evidence.bytes())?;
    Ok(ValidatedF01EPackageManifest {
        manifest_sha256: actual,
        provider_path: entry.0,
        provider_bytes: entry.1,
        provider_sha256: entry.2,
    })
}

fn parse_f01e_manifest_entry(
    bytes: &[u8],
) -> Result<(&'static str, u64, ProviderSha256), F01EProviderError> {
    let root: Value =
        serde_json::from_slice(bytes).map_err(|_| F01EProviderError::PackageManifestSyntax)?;
    let files = root
        .as_object()
        .and_then(|root| root.get("files"))
        .and_then(Value::as_array)
        .ok_or(F01EProviderError::PackageManifestEntry)?;
    let expected = ProviderIdentity::f01e_selected();
    let mut matches = files.iter().filter(|entry| {
        entry
            .as_object()
            .and_then(|entry| entry.get("file"))
            .and_then(Value::as_str)
            == Some(expected.manifest_path)
    });
    let entry = matches
        .next()
        .and_then(Value::as_object)
        .ok_or(F01EProviderError::PackageManifestEntry)?;
    if matches.next().is_some() {
        return Err(F01EProviderError::PackageManifestEntry);
    }
    exact_keys(entry, &["bytes", "file", "sha256"])
        .map_err(|_| F01EProviderError::PackageManifestEntry)?;
    let entry_bytes = u64v(entry, "bytes").map_err(|_| F01EProviderError::PackageManifestEntry)?;
    let entry_sha = string(entry, "sha256")
        .map_err(|_| F01EProviderError::PackageManifestEntry)
        .and_then(parse_sha256)?;
    if entry_bytes != expected.manifest_bytes || entry_sha != expected.manifest_sha {
        return Err(F01EProviderError::PackageManifestEntry);
    }
    Ok((expected.manifest_path, entry_bytes, entry_sha))
}

fn parse_sha256(value: &str) -> Result<ProviderSha256, F01EProviderError> {
    let bytes = decode_hex(value).map_err(|_| F01EProviderError::PackageManifestEntry)?;
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| F01EProviderError::PackageManifestEntry)?;
    Ok(ProviderSha256(bytes))
}

#[derive(Clone, Debug)]
pub struct ValidatedF01EColdProvider {
    value: F01ECold,
    canonical: Arc<[u8]>,
    receipt: F01EProviderReceipt,
}
impl ValidatedF01EColdProvider {
    pub const fn callback_index(&self) -> u64 {
        self.value.callback_index
    }
    pub fn object_count(&self) -> u32 {
        self.value.objects.len() as u32
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }
    pub const fn receipt(&self) -> &F01EProviderReceipt {
        &self.receipt
    }
    pub(super) fn into_parts(self) -> (F01ECold, Arc<[u8]>, F01EProviderReceipt) {
        (self.value, self.canonical, self.receipt)
    }
}

pub fn validate_f01e_provider(
    evidence: &ProviderFileEvidence,
    manifest: &ValidatedF01EPackageManifest,
) -> Result<ValidatedF01EColdProvider, F01EProviderError> {
    let actual = ProviderSha256::digest(evidence.bytes());
    if evidence.observed_byte_length != evidence.bytes.len() as u64
        || evidence.observed_sha256 != actual
    {
        return Err(F01EProviderError::ObservedEvidenceMismatch);
    }
    if manifest.provider_path != ProviderIdentity::f01e_selected().manifest_path
        || manifest.provider_bytes != evidence.observed_byte_length
        || manifest.provider_sha256 != actual
    {
        return Err(F01EProviderError::ProviderIdentityMismatch);
    }
    std::str::from_utf8(evidence.bytes()).map_err(|_| F01EProviderError::Utf8)?;
    let root: Value =
        serde_json::from_slice(evidence.bytes()).map_err(|_| F01EProviderError::JsonSyntax)?;
    let map = root.as_object().ok_or(F01EProviderError::Schema)?;
    exact_keys(
        map,
        &[
            "actual_default_fixture_not_cold",
            "canonical_serialization",
            "cold_bytes",
            "cold_preimage_hex",
            "cold_sha256",
            "derived_scratch_adapter",
            "nested_schema",
            "private_key_required",
            "schema",
            "value",
        ],
    )?;
    if string(map, "schema")? != "f01e-selected-constructor.v1"
        || u64v(map, "cold_bytes")? != 4402
        || string(map, "cold_sha256")?
            != "9c943879cd36cb45bc7538346cfdf4f71d0998d8f261f75c135045fcb59870aa"
        || map.get("private_key_required") != Some(&Value::Bool(false))
    {
        return Err(F01EProviderError::Schema);
    }
    let value = parse_value(map.get("value").ok_or(F01EProviderError::Schema)?)?;
    let canonical = canonical_value(&value);
    if canonical.len() != 4402 {
        return Err(F01EProviderError::CanonicalPreimage);
    }
    let expected = decode_hex(string(map, "cold_preimage_hex")?)?;
    if canonical != expected {
        return Err(F01EProviderError::CanonicalPreimage);
    }
    let cold = ProviderSha256::digest(&canonical);
    if cold != ProviderSha256(COLD_SHA) {
        return Err(F01EProviderError::CanonicalDigest);
    }
    let receipt = F01EProviderReceipt {
        lineage: F01EProviderLineage {
            expected_identity: ProviderIdentity::f01e_selected(),
            manifest_sha256: manifest.manifest_sha256,
        },
        provider_file_sha256: actual,
        provider_file_bytes: evidence.observed_byte_length,
        schema_id: "f01e-selected-constructor.v1",
        schema_version: 1,
        value_pointer: "/value",
        canonical_sha256: cold,
        canonical_bytes: canonical.len() as u64,
        provider_generation: 0,
    };
    Ok(ValidatedF01EColdProvider {
        value,
        canonical: canonical.into(),
        receipt,
    })
}

fn parse_value(v: &Value) -> Result<F01ECold, F01EProviderError> {
    let m = v.as_object().ok_or(F01EProviderError::Shape)?;
    exact_keys(
        m,
        &[
            "callback_index",
            "dbmd_binding",
            "last_control38_snapshot",
            "last_regular37_snapshot",
            "normal_callbacks",
            "normalized_config",
            "objects",
            "phase",
            "physical_binding",
            "remaining_cached_drains",
            "schema",
        ],
    )?;
    if string(m, "schema")? != "F01E_SELECTED_COLD_V1"
        || string(m, "phase")? != "BEFORE_FIRST_NORMAL"
    {
        return Err(F01EProviderError::Shape);
    }
    for k in [
        "dbmd_binding",
        "last_control38_snapshot",
        "last_regular37_snapshot",
        "physical_binding",
    ] {
        if m.get(k) != Some(&Value::Null) {
            return Err(F01EProviderError::Shape);
        }
    }
    let a = m
        .get("objects")
        .and_then(Value::as_array)
        .ok_or(F01EProviderError::Shape)?;
    if a.len() != 28 {
        return Err(F01EProviderError::Shape);
    }
    let mut objects = Vec::with_capacity(28);
    for (i, o) in a.iter().enumerate() {
        let x = o.as_object().ok_or(F01EProviderError::Shape)?;
        exact_keys(
            x,
            &[
                "event_cache",
                "host_channel_index",
                "object_ordinal",
                "sampler_previous_present",
                "sampler_previous_xyz_u32",
            ],
        )?;
        if u64v(x, "object_ordinal")? != i as u64
            || x.get("event_cache") != Some(&Value::Null)
            || x.get("host_channel_index") != Some(&Value::Null)
            || x.get("sampler_previous_present") != Some(&Value::Bool(false))
        {
            return Err(F01EProviderError::Shape);
        }
        let xyz = x
            .get("sampler_previous_xyz_u32")
            .and_then(Value::as_array)
            .ok_or(F01EProviderError::Shape)?;
        if xyz.len() != 3 || xyz.iter().any(|v| v.as_u64() != Some(0)) {
            return Err(F01EProviderError::Shape);
        }
        objects.push(ObjectCold { ordinal: i as u32 });
    }
    let config = parse_config(m.get("normalized_config").ok_or(F01EProviderError::Shape)?)?;
    Ok(F01ECold {
        callback_index: u64v(m, "callback_index")?,
        normal_callbacks: u64v(m, "normal_callbacks")?,
        remaining_cached_drains: u64v(m, "remaining_cached_drains")?
            .try_into()
            .map_err(|_| F01EProviderError::Shape)?,
        objects,
        config,
    })
}
fn parse_config(v: &Value) -> Result<NormalizedConfig, F01EProviderError> {
    let m = v.as_object().ok_or(F01EProviderError::Shape)?;
    exact_keys(
        m,
        &[
            "bed_count",
            "bit_rate_enum",
            "bit_rate_kbps",
            "channels",
            "dispatch_mode",
            "event_start_samples",
            "frame_samples",
            "layout_enum",
            "line_drc",
            "object_count",
            "pcm_bits",
            "rf_drc",
            "sample_rate_hz",
            "source_samples",
            "spatial_config_mode",
            "target_capacity",
            "time_base",
            "virtual_prefix_count",
        ],
    )?;
    let c = NormalizedConfig {
        sample_rate_hz: u32v(m, "sample_rate_hz")?,
        channels: u32v(m, "channels")?,
        pcm_bits: u32v(m, "pcm_bits")?,
        source_samples: u64v(m, "source_samples")?,
        bit_rate_kbps: u32v(m, "bit_rate_kbps")?,
        bit_rate_enum: u32v(m, "bit_rate_enum")?,
        layout_enum: u32v(m, "layout_enum")?,
        target_capacity: u32v(m, "target_capacity")?,
        frame_samples: u32v(m, "frame_samples")?,
        spatial_config_mode: u32v(m, "spatial_config_mode")?,
        dispatch_mode: u32v(m, "dispatch_mode")?,
        virtual_prefix_count: u32v(m, "virtual_prefix_count")?,
        bed_count: u32v(m, "bed_count")?,
        object_count: u32v(m, "object_count")?,
        event_start_samples: u64v(m, "event_start_samples")?,
        line_drc: string(m, "line_drc")?.to_owned(),
        rf_drc: string(m, "rf_drc")?.to_owned(),
        time_base: string(m, "time_base")?.to_owned(),
    };
    if c.sample_rate_hz != 48_000
        || c.channels != 38
        || c.pcm_bits != 24
        || c.source_samples != 6_522_906
        || c.bit_rate_kbps != 1024
        || c.bit_rate_enum != 5
        || c.layout_enum != 8
        || c.target_capacity != 15
        || c.frame_samples != 1536
        || c.spatial_config_mode != 0
        || c.dispatch_mode != 2
        || c.virtual_prefix_count != 11
        || c.bed_count != 10
        || c.object_count != 28
        || c.event_start_samples != 0
        || c.line_drc != "none"
        || c.rf_drc != "none"
        || c.time_base != "file_position"
    {
        return Err(F01EProviderError::Shape);
    }
    Ok(c)
}
fn exact_keys(m: &Map<String, Value>, keys: &[&str]) -> Result<(), F01EProviderError> {
    if m.len() != keys.len() || keys.iter().any(|k| !m.contains_key(*k)) {
        Err(F01EProviderError::Shape)
    } else {
        Ok(())
    }
}
fn string<'a>(m: &'a Map<String, Value>, k: &str) -> Result<&'a str, F01EProviderError> {
    m.get(k)
        .and_then(Value::as_str)
        .ok_or(F01EProviderError::Shape)
}
fn u64v(m: &Map<String, Value>, k: &str) -> Result<u64, F01EProviderError> {
    m.get(k)
        .and_then(Value::as_u64)
        .ok_or(F01EProviderError::Shape)
}
fn u32v(m: &Map<String, Value>, k: &str) -> Result<u32, F01EProviderError> {
    u64v(m, k)?.try_into().map_err(|_| F01EProviderError::Shape)
}
fn decode_hex(s: &str) -> Result<Vec<u8>, F01EProviderError> {
    if s.len() % 2 != 0 {
        return Err(F01EProviderError::CanonicalPreimage);
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| F01EProviderError::CanonicalPreimage)
        })
        .collect()
}
fn canonical_value(v: &F01ECold) -> Vec<u8> {
    let mut o = Vec::new();
    macro_rules! raw {
        ($x:expr) => {
            o.extend_from_slice($x.as_bytes())
        };
    }
    raw!("{\"callback_index\":");
    raw!(&v.callback_index.to_string());
    raw!(
        ",\"dbmd_binding\":null,\"last_control38_snapshot\":null,\"last_regular37_snapshot\":null,\"normal_callbacks\":"
    );
    raw!(&v.normal_callbacks.to_string());
    raw!(",\"normalized_config\":");
    canonical_config(&mut o, &v.config);
    raw!(",\"objects\":[");
    for (i, x) in v.objects.iter().enumerate() {
        if i > 0 {
            o.push(b',');
        }
        raw!("{\"event_cache\":null,\"host_channel_index\":null,\"object_ordinal\":");
        raw!(&x.ordinal.to_string());
        raw!(",\"sampler_previous_present\":false,\"sampler_previous_xyz_u32\":[0,0,0]}");
    }
    raw!(
        "],\"phase\":\"BEFORE_FIRST_NORMAL\",\"physical_binding\":null,\"remaining_cached_drains\":"
    );
    raw!(&v.remaining_cached_drains.to_string());
    raw!(",\"schema\":\"F01E_SELECTED_COLD_V1\"}");
    o
}
fn canonical_config(o: &mut Vec<u8>, c: &NormalizedConfig) {
    macro_rules! raw {
        ($x:expr) => {
            o.extend_from_slice($x.as_bytes())
        };
    }
    raw!("{\"bed_count\":");
    raw!(&c.bed_count.to_string());
    raw!(",\"bit_rate_enum\":");
    raw!(&c.bit_rate_enum.to_string());
    raw!(",\"bit_rate_kbps\":");
    raw!(&c.bit_rate_kbps.to_string());
    raw!(",\"channels\":");
    raw!(&c.channels.to_string());
    raw!(",\"dispatch_mode\":");
    raw!(&c.dispatch_mode.to_string());
    raw!(",\"event_start_samples\":");
    raw!(&c.event_start_samples.to_string());
    raw!(",\"frame_samples\":");
    raw!(&c.frame_samples.to_string());
    raw!(",\"layout_enum\":");
    raw!(&c.layout_enum.to_string());
    raw!(",\"line_drc\":\"");
    raw!(&c.line_drc);
    raw!("\",\"object_count\":");
    raw!(&c.object_count.to_string());
    raw!(",\"pcm_bits\":");
    raw!(&c.pcm_bits.to_string());
    raw!(",\"rf_drc\":\"");
    raw!(&c.rf_drc);
    raw!("\",\"sample_rate_hz\":");
    raw!(&c.sample_rate_hz.to_string());
    raw!(",\"source_samples\":");
    raw!(&c.source_samples.to_string());
    raw!(",\"spatial_config_mode\":");
    raw!(&c.spatial_config_mode.to_string());
    raw!(",\"target_capacity\":");
    raw!(&c.target_capacity.to_string());
    raw!(",\"time_base\":\"");
    raw!(&c.time_base);
    raw!("\",\"virtual_prefix_count\":");
    raw!(&c.virtual_prefix_count.to_string());
    raw!("}");
}

#[cfg(test)]
mod tests {
    use super::{
        F01EProviderError, PackageManifestEvidence, ProviderFileEvidence, ProviderSha256,
        parse_f01e_manifest_entry, validate_f01e_package_manifest, validate_f01e_provider,
    };
    use crate::{
        frontend::{FrontendControlError, FrontendControlMachine},
        timeline::FrontendCallbackIndex,
        transaction::{AttemptId, CallbackLineageIdentity},
    };
    use std::path::PathBuf;

    const PATH: &str = "parameters/f01e-selected-constructor-v1.json";
    const SHA: &str = "b0548b0c45e321795455c68095e804de5507af318636676353a73f119a41f7cb";

    fn manifest(entries: &str) -> Vec<u8> {
        format!(r#"{{"files":[{entries}]}}"#).into_bytes()
    }

    fn entry(path: &str, bytes: u64, sha: &str) -> String {
        format!(r#"{{"file":"{path}","bytes":{bytes},"sha256":"{sha}"}}"#)
    }

    #[test]
    fn manifest_entry_must_match_path_bytes_sha_and_be_unique() {
        let valid = entry(PATH, 20_138, SHA);
        assert!(parse_f01e_manifest_entry(&manifest(&valid)).is_ok());

        for invalid in [
            entry("parameters/other.json", 20_138, SHA),
            entry(PATH, 20_139, SHA),
            entry(PATH, 20_138, &"00".repeat(32)),
            format!("{valid},{valid}"),
        ] {
            assert!(matches!(
                parse_f01e_manifest_entry(&manifest(&invalid)),
                Err(F01EProviderError::PackageManifestEntry)
            ));
        }
    }

    #[test]
    fn frontend_prepare_rejects_a_receipt_with_different_provider_lineage() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/f01e_provider");
        let manifest = PackageManifestEvidence::new(
            std::fs::read(root.join("MANIFEST.json")).unwrap().into(),
            None,
        );
        let provider = ProviderFileEvidence::new(
            std::fs::read(root.join("f01e-selected-constructor-v1.json"))
                .unwrap()
                .into(),
            None,
        );
        let manifest = validate_f01e_package_manifest(&manifest).unwrap();
        let validated = validate_f01e_provider(&provider, &manifest).unwrap();
        let mut wrong_receipt = validated.receipt().clone();
        wrong_receipt.lineage.manifest_sha256 = ProviderSha256::digest(b"different manifest");
        let programme_id = crate::transaction::test_programme_id("provider-test-programme");
        let attempt = AttemptId::new(0);
        let machine =
            FrontendControlMachine::from_provider(validated, programme_id.clone(), attempt)
                .unwrap();
        assert!(matches!(
            machine.prepare(
                CallbackLineageIdentity::new(
                    programme_id.clone(),
                    attempt,
                    FrontendCallbackIndex::new(0),
                ),
                0,
                &wrong_receipt,
            ),
            Err(FrontendControlError::ProviderMismatch)
        ));
    }
}
