// pattern: Functional Core
//! Hash-bound non-exhaustive direct Meter provider assets and their typed projections.

use crate::meter::{
    integrated_meter::{IntegratedMeterError, IntegratedMeterParameters},
    k_matrix::{
        KMatrixError, KMatrixMode, KMatrixProfile, KMatrixProvider,
        generate_k_matrix_from_approved_direct_assets,
    },
    object_render_metadata::SecondaryGeometry,
    post_render_meter::PostRenderMeterParameters,
    shared_gain_delay::{SharedGainDelayDirectParameters, SharedGainDelayError},
    true_peak::{TruePeakError, TruePeakParameters, validate_approved_provider_assets},
    ungated_mean::{UngatedMeanError, UngatedMeanParameters},
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const PACKAGE_ID: &str = "METER-APPROVED-PARAMETER-ASSETS-V1";
const PACKAGE_MANIFEST_SHA256: &str =
    "f802dd4b0576fe72b9f6be5538974cd91149df26c8aa01f9efa6822783d4df04";
const APPROVED_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";

const DIRECT_ASSETS: [DirectAssetSpec; 7] = [
    DirectAssetSpec {
        id: MeterDirectProviderAssetId::TruePeak,
        path: "parameters/meter-truepeak-authorized-v1.json",
        bytes: 2_584,
        sha256: "f67e6fe86533af74acc261bf3908890e77a461198116c64b624dc91b69049757",
    },
    DirectAssetSpec {
        id: MeterDirectProviderAssetId::OarGeometry,
        path: "parameters/meter-oar-secondary8-math-v1.json",
        bytes: 1_757,
        sha256: "d1a2f3d3a7f1a7e67933acc781be406814f63a2e842975d34960d135a5307a77",
    },
    DirectAssetSpec {
        id: MeterDirectProviderAssetId::LogConstants,
        path: "parameters/meter-log-mathematical-v1.json",
        bytes: 780,
        sha256: "044d42327f471b183b0a29caaa9db85f694a40c436a5afe3b83734dbf29bc266",
    },
    DirectAssetSpec {
        id: MeterDirectProviderAssetId::KStateBounds,
        path: "parameters/meter-k-state-bounds-v1.json",
        bytes: 970,
        sha256: "2faaa9d0aeefb105a913589590fcbafb97362a54192b0b21c5fdda7da1974e68",
    },
    DirectAssetSpec {
        id: MeterDirectProviderAssetId::KMatrix,
        path: "parameters/meter-k-matrix-math-v1.json",
        bytes: 3_985,
        sha256: "c8b4ffd52fffd62f5d0fdeba36a993f02188933a43593abe6969243f44f7f536",
    },
    DirectAssetSpec {
        id: MeterDirectProviderAssetId::SharedDv32,
        path: "parameters/shared-dv32-mathematical-v1.json",
        bytes: 1_236,
        sha256: "af189ffb34f703fdebe9a37bc46ffe972d97432b7f2e3f2a463b82682c96d4fa",
    },
    DirectAssetSpec {
        id: MeterDirectProviderAssetId::Installed48DependencyIndex,
        path: "parameters/adm-installed48-providers-v1.json",
        bytes: 14_372,
        sha256: "51305803b05daff498935b3283e56e27bb3e3966f52727bcf803a226871d5e91",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterDirectProviderAssetId {
    TruePeak,
    OarGeometry,
    LogConstants,
    KStateBounds,
    KMatrix,
    SharedDv32,
    Installed48DependencyIndex,
}

/// Identity receipt for one directly inventoried asset; it does not imply transitive coverage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeterDirectProviderAssetReceipt {
    id: MeterDirectProviderAssetId,
    bytes: u64,
    sha256: [u8; 32],
}

impl MeterDirectProviderAssetReceipt {
    pub const fn id(&self) -> MeterDirectProviderAssetId {
        self.id
    }

    pub const fn bytes(&self) -> u64 {
        self.bytes
    }

    pub const fn sha256(&self) -> &[u8; 32] {
        &self.sha256
    }
}

/// Selected OAR's exact eight-slot geometry values from its direct approved asset.
///
/// This is an immutable asset projection only. It is not bound to an admitted layout, callback,
/// OAR owner generation or same-callback PCM/spatial values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterOarSecondaryGeometryProvider {
    generated_i32_rows: [[i32; 4]; 8],
    render_geometry: SecondaryGeometry,
}

impl MeterOarSecondaryGeometryProvider {
    pub const fn generated_i32_rows(&self) -> &[[i32; 4]; 8] {
        &self.generated_i32_rows
    }

    pub const fn render_geometry(&self) -> &SecondaryGeometry {
        &self.render_geometry
    }
}

/// Typed, hash-bound direct Meter assets. This inventory is deliberately not the transitive
/// Meter provider set and cannot authorize a Meter root, prepass, READY result, or receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterDirectProviderInventory {
    package_manifest_sha256: [u8; 32],
    approved_manifest_sha256: [u8; 32],
    assets: [MeterDirectProviderAssetReceipt; 7],
    oar_geometry: MeterOarSecondaryGeometryProvider,
    shared_dv32: SharedGainDelayDirectParameters,
    k_matrix: KMatrixProvider,
    integrated_meter_parameters: IntegratedMeterParameters,
    ungated_mean_parameters: UngatedMeanParameters,
    true_peak: TruePeakParameters,
}

impl MeterDirectProviderInventory {
    pub const fn package_manifest_sha256(&self) -> &[u8; 32] {
        &self.package_manifest_sha256
    }

    pub const fn approved_manifest_sha256(&self) -> &[u8; 32] {
        &self.approved_manifest_sha256
    }

    pub const fn direct_assets(&self) -> &[MeterDirectProviderAssetReceipt; 7] {
        &self.assets
    }

    /// This exact seven-file inventory is not the complete transitive Meter provider set.
    pub const fn transitive_provider_set_complete(&self) -> bool {
        false
    }

    pub const fn oar_geometry(&self) -> &MeterOarSecondaryGeometryProvider {
        &self.oar_geometry
    }

    pub const fn shared_dv32_parameters(&self) -> &SharedGainDelayDirectParameters {
        &self.shared_dv32
    }

    pub const fn k_matrix(&self) -> &KMatrixProvider {
        &self.k_matrix
    }

    pub const fn integrated_meter_parameters(&self) -> &IntegratedMeterParameters {
        &self.integrated_meter_parameters
    }

    pub const fn ungated_mean_parameters(&self) -> &UngatedMeanParameters {
        &self.ungated_mean_parameters
    }

    pub const fn true_peak_parameters(&self) -> &TruePeakParameters {
        &self.true_peak
    }

    /// Composes the validated direct owner values consumed by the numeric meter leaves.
    pub fn post_render_parameters(&self) -> PostRenderMeterParameters {
        PostRenderMeterParameters::new(
            self.true_peak.clone(),
            self.integrated_meter_parameters.clone(),
            self.ungated_mean_parameters.clone(),
            self.oar_geometry.render_geometry().clone(),
        )
    }
}

/// Borrowed direct provider payloads supplied by the IO adapter.
#[derive(Clone, Copy)]
pub struct MeterDirectProviderPayloads<'a> {
    pub true_peak: &'a [u8],
    pub oar_geometry: &'a [u8],
    pub log_constants: &'a [u8],
    pub k_state_bounds: &'a [u8],
    pub k_matrix: &'a [u8],
    pub shared_dv32: &'a [u8],
    pub installed48_dependency_index: &'a [u8],
}

impl<'a> MeterDirectProviderPayloads<'a> {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        true_peak: &'a [u8],
        oar_geometry: &'a [u8],
        log_constants: &'a [u8],
        k_state_bounds: &'a [u8],
        k_matrix: &'a [u8],
        shared_dv32: &'a [u8],
        installed48_dependency_index: &'a [u8],
    ) -> Self {
        Self {
            true_peak,
            oar_geometry,
            log_constants,
            k_state_bounds,
            k_matrix,
            shared_dv32,
            installed48_dependency_index,
        }
    }

    fn by_id(&self, id: MeterDirectProviderAssetId) -> &'a [u8] {
        match id {
            MeterDirectProviderAssetId::TruePeak => self.true_peak,
            MeterDirectProviderAssetId::OarGeometry => self.oar_geometry,
            MeterDirectProviderAssetId::LogConstants => self.log_constants,
            MeterDirectProviderAssetId::KStateBounds => self.k_state_bounds,
            MeterDirectProviderAssetId::KMatrix => self.k_matrix,
            MeterDirectProviderAssetId::SharedDv32 => self.shared_dv32,
            MeterDirectProviderAssetId::Installed48DependencyIndex => {
                self.installed48_dependency_index
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterDirectProviderInventoryError {
    PackageManifestIdentity,
    PackageManifestShape,
    DirectAssetIdentity(MeterDirectProviderAssetId),
    OarGeometryShape,
    KMatrix(KMatrixError),
    SharedDv32(SharedGainDelayError),
    IntegratedMeter(IntegratedMeterError),
    UngatedMean(UngatedMeanError),
    TruePeak(TruePeakError),
}

/// Validates the selected seven-asset direct subset and binds supported owner values.
pub fn validate_meter_direct_provider_inventory(
    manifest_bytes: &[u8],
    payloads: MeterDirectProviderPayloads<'_>,
) -> Result<MeterDirectProviderInventory, MeterDirectProviderInventoryError> {
    if sha256_hex(manifest_bytes) != PACKAGE_MANIFEST_SHA256 {
        return Err(MeterDirectProviderInventoryError::PackageManifestIdentity);
    }
    let manifest: Value = serde_json::from_slice(manifest_bytes)
        .map_err(|_| MeterDirectProviderInventoryError::PackageManifestShape)?;
    if manifest["package_id"] != PACKAGE_ID
        || manifest["scope"]
            != "approved nonsecret assets; layout and remaining transitive dependencies not claimed complete"
        || manifest["approved_manifest_sha256"] != APPROVED_MANIFEST_SHA256
    {
        return Err(MeterDirectProviderInventoryError::PackageManifestShape);
    }

    let files = manifest["files"]
        .as_array()
        .ok_or(MeterDirectProviderInventoryError::PackageManifestShape)?;
    let mut receipts = Vec::with_capacity(DIRECT_ASSETS.len());
    for spec in DIRECT_ASSETS {
        let payload = payloads.by_id(spec.id);
        let expected_bytes = u64::try_from(spec.bytes)
            .map_err(|_| MeterDirectProviderInventoryError::PackageManifestShape)?;
        let entry = files
            .iter()
            .find(|entry| entry["file"] == spec.path)
            .ok_or(MeterDirectProviderInventoryError::DirectAssetIdentity(
                spec.id,
            ))?;
        let actual_sha256 = sha256_hex(payload);
        if payload.len() != spec.bytes
            || actual_sha256 != spec.sha256
            || entry["bytes"].as_u64() != Some(expected_bytes)
            || entry["sha256"].as_str() != Some(spec.sha256)
        {
            return Err(MeterDirectProviderInventoryError::DirectAssetIdentity(
                spec.id,
            ));
        }
        receipts.push(MeterDirectProviderAssetReceipt {
            id: spec.id,
            bytes: expected_bytes,
            sha256: sha256_bytes(payload),
        });
    }
    let assets: [MeterDirectProviderAssetReceipt; 7] = receipts
        .try_into()
        .map_err(|_| MeterDirectProviderInventoryError::PackageManifestShape)?;
    let approved_manifest_sha256 = decode_hex::<32>(APPROVED_MANIFEST_SHA256)
        .ok_or(MeterDirectProviderInventoryError::PackageManifestShape)?;

    let oar_geometry = validate_oar_geometry(payloads.oar_geometry)?;
    let shared_dv32 =
        SharedGainDelayDirectParameters::validate_approved_asset(payloads.shared_dv32)
            .map_err(MeterDirectProviderInventoryError::SharedDv32)?;
    let k_matrix = generate_k_matrix_from_approved_direct_assets(
        KMatrixProfile {
            sample_rate_hz: 48_000,
            mode: KMatrixMode::Modern,
            stage_count: 2,
        },
        payloads.k_matrix,
        payloads.k_state_bounds,
    )
    .map_err(MeterDirectProviderInventoryError::KMatrix)?;
    let integrated_meter_parameters =
        IntegratedMeterParameters::from_approved_log_asset(payloads.log_constants)
            .map_err(MeterDirectProviderInventoryError::IntegratedMeter)?;
    let ungated_mean_parameters =
        UngatedMeanParameters::from_approved_log_asset(payloads.log_constants)
            .map_err(MeterDirectProviderInventoryError::UngatedMean)?;
    let true_peak = validate_approved_provider_assets(payloads.true_peak, payloads.log_constants)
        .map_err(MeterDirectProviderInventoryError::TruePeak)?;

    Ok(MeterDirectProviderInventory {
        package_manifest_sha256: sha256_bytes(manifest_bytes),
        approved_manifest_sha256,
        assets,
        oar_geometry,
        shared_dv32,
        k_matrix,
        integrated_meter_parameters,
        ungated_mean_parameters,
        true_peak,
    })
}

fn validate_oar_geometry(
    bytes: &[u8],
) -> Result<MeterOarSecondaryGeometryProvider, MeterDirectProviderInventoryError> {
    let document: Value = serde_json::from_slice(bytes)
        .map_err(|_| MeterDirectProviderInventoryError::OarGeometryShape)?;
    if document["schema"] != "meter-oar-secondary8-mathematical-geometry.v1"
        || document["generated_entries"] != 32
        || document["raw_source_table_copied"] != false
        || document["delivery_scope"] != "INTERNAL_ONLY"
        || document["implementation_ready"] != "NO"
        || document["original_design_rationale"]
            != "NOT_CLAIMED; explicit independent canonical grid definition and identity verification"
        || document["full_panner_metadata_and_renderer"] != "NOT_CLOSED_BY_GEOMETRY_IDENTITY"
        || document["coordinate_domain"] != "normalized layout canvas, not claimed acoustic angles"
        || document["coordinate_rule"]
            != "active ideal positions 0, 1/2 or1 to Q15: min(32767,max(0,RNE_integer(32768*x))); inactive coded LFE uses I32 -1 sentinel in each coordinate"
        || document["partner_rule"]
            != "side_left/right link to corresponding back slot6/7; other slots use sentinel35, one-past global output code domain0..34"
        || document["z_rule"] != "positive zero for every active point"
        || document["slot_names"]
            != serde_json::json!([
                "front_left",
                "front_right",
                "front_center",
                "coded_lfe",
                "side_left",
                "side_right",
                "back_left",
                "back_right"
            ])
        || document["ideal_xy_rule"]
            != serde_json::json!([
                "(0,0)", "(1,0)", "(1/2,0)", "INACTIVE", "(0,1/2)", "(1,1/2)", "(0,1)", "(1,1)"
            ])
        || document["ideal_xy_rule"].as_array().map(Vec::len) != Some(8)
        || document["slot_names"].as_array().map(Vec::len) != Some(8)
        || document["i32le_sha256"]
            != "3f8b8b5d4880052f66516455883e4c3aee026e2b94f99191bfb0ad74a8f25fc3"
    {
        return Err(MeterDirectProviderInventoryError::OarGeometryShape);
    }
    let values = document["generated_i32_rows"]
        .as_array()
        .filter(|rows| rows.len() == 8)
        .ok_or(MeterDirectProviderInventoryError::OarGeometryShape)?;
    let rows = values
        .iter()
        .map(|row| {
            let words = row
                .as_array()
                .filter(|words| words.len() == 4)
                .ok_or(MeterDirectProviderInventoryError::OarGeometryShape)?;
            let values = words
                .iter()
                .map(|word| {
                    word.as_i64()
                        .and_then(|word| i32::try_from(word).ok())
                        .ok_or(MeterDirectProviderInventoryError::OarGeometryShape)
                })
                .collect::<Result<Vec<_>, _>>()?;
            values
                .try_into()
                .map_err(|_| MeterDirectProviderInventoryError::OarGeometryShape)
        })
        .collect::<Result<Vec<[i32; 4]>, _>>()?;
    let generated_i32_rows: [[i32; 4]; 8] = rows
        .try_into()
        .map_err(|_| MeterDirectProviderInventoryError::OarGeometryShape)?;
    let serialized_i32le = generated_i32_rows
        .iter()
        .flatten()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>();
    if sha256_hex(&serialized_i32le)
        != "3f8b8b5d4880052f66516455883e4c3aee026e2b94f99191bfb0ad74a8f25fc3"
    {
        return Err(MeterDirectProviderInventoryError::OarGeometryShape);
    }
    let render_geometry = SecondaryGeometry::try_from_rows(generated_i32_rows)
        .map_err(|_| MeterDirectProviderInventoryError::OarGeometryShape)?;
    Ok(MeterOarSecondaryGeometryProvider {
        generated_i32_rows,
        render_geometry,
    })
}

#[derive(Clone, Copy)]
struct DirectAssetSpec {
    id: MeterDirectProviderAssetId,
    path: &'static str,
    bytes: usize,
    sha256: &'static str,
}

fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = sha256_bytes(bytes);
    hex_lower(&digest)
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn decode_hex<const N: usize>(value: &str) -> Option<[u8; N]> {
    if value.len() != N * 2 {
        return None;
    }
    let mut decoded = [0; N];
    for (index, byte) in decoded.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(decoded)
}

#[cfg(test)]
mod tests {
    use super::{
        MeterDirectProviderInventoryError, MeterDirectProviderPayloads,
        validate_meter_direct_provider_inventory,
    };

    const PACKAGE_MANIFEST: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-APPROVED-PARAMETER-ASSETS-V1/MANIFEST.json"
    );
    const TRUE_PEAK: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-APPROVED-PARAMETER-ASSETS-V1/parameters/meter-truepeak-authorized-v1.json"
    );
    const OAR_GEOMETRY: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-APPROVED-PARAMETER-ASSETS-V1/parameters/meter-oar-secondary8-math-v1.json"
    );
    const LOG_CONSTANTS: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-APPROVED-PARAMETER-ASSETS-V1/parameters/meter-log-mathematical-v1.json"
    );
    const K_STATE_BOUNDS: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-APPROVED-PARAMETER-ASSETS-V1/parameters/meter-k-state-bounds-v1.json"
    );
    const K_MATRIX: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-APPROVED-PARAMETER-ASSETS-V1/parameters/meter-k-matrix-math-v1.json"
    );
    const SHARED_DV32: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-APPROVED-PARAMETER-ASSETS-V1/parameters/shared-dv32-mathematical-v1.json"
    );
    const INSTALLED48_DEPENDENCY_INDEX: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-APPROVED-PARAMETER-ASSETS-V1/parameters/adm-installed48-providers-v1.json"
    );
    const DIRECT_INVENTORY_INPUT_ONLY: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-DIRECT-PROVIDER-INVENTORY-V1/INPUT-ONLY.json"
    );

    fn payloads() -> MeterDirectProviderPayloads<'static> {
        MeterDirectProviderPayloads::new(
            TRUE_PEAK,
            OAR_GEOMETRY,
            LOG_CONSTANTS,
            K_STATE_BOUNDS,
            K_MATRIX,
            SHARED_DV32,
            INSTALLED48_DEPENDENCY_INDEX,
        )
    }

    #[test]
    fn approved_assets_bind_actual_typed_direct_values_only() {
        let inventory = validate_meter_direct_provider_inventory(PACKAGE_MANIFEST, payloads())
            .expect("approved direct parameter subset validates");
        assert_eq!(inventory.direct_assets().len(), 7);
        assert!(!inventory.transitive_provider_set_complete());
        assert_eq!(inventory.k_matrix().complete_words().len(), 72);
        assert_eq!(inventory.shared_dv32_parameters().q15_integers().len(), 8);
        assert_eq!(inventory.oar_geometry().generated_i32_rows().len(), 8);
        assert_eq!(
            inventory.integrated_meter_parameters().log_words().len(),
            11
        );
        assert_eq!(inventory.ungated_mean_parameters().log_words().len(), 11);
        assert_eq!(inventory.true_peak_parameters().stage_a_words().len(), 12);
        assert_eq!(
            inventory.approved_manifest_sha256(),
            &hex_bytes("c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a")
        );
        // The inventory exposes no transitive provider set, root, receipt or READY constructor.
    }

    #[test]
    fn altered_direct_parameter_bytes_are_rejected() {
        let mut changed = K_MATRIX.to_vec();
        changed[0] ^= 1;
        let payloads = MeterDirectProviderPayloads::new(
            TRUE_PEAK,
            OAR_GEOMETRY,
            LOG_CONSTANTS,
            K_STATE_BOUNDS,
            &changed,
            SHARED_DV32,
            INSTALLED48_DEPENDENCY_INDEX,
        );
        assert_eq!(
            validate_meter_direct_provider_inventory(PACKAGE_MANIFEST, payloads).unwrap_err(),
            MeterDirectProviderInventoryError::DirectAssetIdentity(
                super::MeterDirectProviderAssetId::KMatrix
            )
        );
    }

    #[test]
    fn manifest_scope_remains_a_non_exhaustive_direct_subset() {
        let manifest: serde_json::Value = serde_json::from_slice(PACKAGE_MANIFEST).unwrap();
        assert_eq!(
            manifest["scope"],
            "approved nonsecret assets; layout and remaining transitive dependencies not claimed complete"
        );
        assert_eq!(manifest["files"].as_array().unwrap().len(), 18);
    }

    #[test]
    fn contract_input_only_profile_and_wrong_k_profile_cases_are_enforced() {
        let cases: serde_json::Value = serde_json::from_slice(DIRECT_INVENTORY_INPUT_ONLY).unwrap();
        assert_eq!(
            super::sha256_hex(DIRECT_INVENTORY_INPUT_ONLY),
            "b26b86295da0c4c64eb476ca49905710c8d26dc60531c6d665a5853c6809d667"
        );
        assert_eq!(cases["input_only"], true);
        assert_eq!(cases["status"], "NOT_RUN");
        assert_eq!(cases["cases"][0]["sample_rate"], 48_000);
        assert_eq!(cases["cases"][0]["channels"], 6);
        assert_eq!(cases["cases"][0]["dv_block"], 32);
        assert_eq!(cases["cases"][1]["manifest_present"], true);
        assert_eq!(cases["cases"][1]["oar_values_loaded"], false);

        let result = super::generate_k_matrix_from_approved_direct_assets(
            super::KMatrixProfile {
                sample_rate_hz: 48_000,
                mode: super::KMatrixMode::Modern,
                stage_count: 3,
            },
            K_MATRIX,
            K_STATE_BOUNDS,
        );
        assert_eq!(result, Err(super::KMatrixError::UnsupportedProfile));
        assert_eq!(cases["cases"][2]["k_stage_count"], 3);
    }

    #[test]
    fn direct_subset_keeps_upstream_index_identity_without_loading_its_values() {
        let inventory = validate_meter_direct_provider_inventory(PACKAGE_MANIFEST, payloads())
            .expect("approved direct parameter subset validates");
        let receipt = inventory
            .direct_assets()
            .iter()
            .find(|receipt| {
                receipt.id() == super::MeterDirectProviderAssetId::Installed48DependencyIndex
            })
            .unwrap();
        assert_eq!(receipt.bytes(), 14_372);
        assert_eq!(
            receipt.sha256(),
            &hex_bytes("51305803b05daff498935b3283e56e27bb3e3966f52727bcf803a226871d5e91")
        );
        let cases: serde_json::Value = serde_json::from_slice(DIRECT_INVENTORY_INPUT_ONLY).unwrap();
        assert_eq!(
            cases["status"], "NOT_RUN",
            "an upstream dependency index is not a numerical provider or production receipt"
        );
    }

    #[test]
    fn dv32_wrong_profile_case_rejects_before_returning_typed_parameters() {
        let mut provider: serde_json::Value = serde_json::from_slice(SHARED_DV32).unwrap();
        provider["profile"]["block_samples"] = serde_json::json!(64);
        let changed = serde_json::to_vec(&provider).unwrap();
        assert_eq!(
            super::SharedGainDelayDirectParameters::validate_approved_asset(&changed),
            Err(super::SharedGainDelayError::ParameterUnavailable)
        );
        let cases: serde_json::Value = serde_json::from_slice(DIRECT_INVENTORY_INPUT_ONLY).unwrap();
        assert_eq!(cases["cases"][3]["dv_block"], 64);
        assert_eq!(cases["cases"][4]["production_allowed"], false);
        assert_eq!(cases["cases"][5]["attempt_mismatch"], true);
        assert_eq!(cases["cases"][6]["completed_drains"], 2);
    }

    fn hex_bytes(value: &str) -> [u8; 32] {
        let mut bytes = [0; 32];
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).unwrap();
        }
        bytes
    }
}
