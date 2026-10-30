// pattern: Imperative Shell
//! Builds the static Meter provider-set receipt from owner-validated selected values.

use std::path::Path;

use jocforge_application::{
    meter_source_admission::{MeterSourceAdmissionError, MeterSourceProviderSetAdmissionReceipt},
    meter_source_bridge::{
        EmdfCarriageProfile, ExpectedMeterProviderMember, KMatrixProvider,
        MeterDirectProviderAssetId, MeterDirectProviderInventory, MeterProviderMemberReceipt,
        MeterProviderSetIdentityError, SelectedMeterProviderRoster,
        SelectedMeterProviderSetIdentity, SharedGainDelayDirectParameters, TruePeakParameters,
        ValidatedComponentParameterProvider, ValidatedFeatureFft4096MathProvider,
        ValidatedWindowQ15Provider, validated_value_words_sha256,
    },
};
use jocforge_io::provider::load_selected_meter_provider_roster;

use crate::selected_spatial_callback_providers::SelectedIndependentMeterCallbackProviders;

#[derive(Debug)]
pub enum SelectedMeterProviderIdentityError {
    UnsupportedCarriage,
    MissingRosterMember(String),
    MissingDirectAsset(MeterDirectProviderAssetId),
    MissingSpatialLineage(String),
    SourceIdentityMismatch(String),
    RosterLoad(jocforge_io::provider::MeterProviderRosterLoadError),
    ProviderSet(MeterProviderSetIdentityError),
    SourceAdmission(MeterSourceAdmissionError),
}

/// Loads the exact hash-bound roster, then constructs the selected profile's static identity.
pub fn load_selected_meter_provider_identity(
    roster_package_root: &Path,
    providers: &SelectedIndependentMeterCallbackProviders,
    direct: &MeterDirectProviderInventory,
    selected_profile_id: &str,
) -> Result<SelectedMeterProviderSetIdentity, SelectedMeterProviderIdentityError> {
    let roster = load_selected_meter_provider_roster(roster_package_root)
        .map_err(SelectedMeterProviderIdentityError::RosterLoad)?;
    build_selected_meter_provider_identity(&roster, providers, direct, selected_profile_id)
}

/// Profile identity to use before combining an exact selected-profile roster.
pub fn selected_meter_profile_id(
    carriage: EmdfCarriageProfile,
) -> Result<&'static str, SelectedMeterProviderIdentityError> {
    jocforge_application::meter_source_admission::selected_meter_profile_identity(carriage)
        .map_err(|_| SelectedMeterProviderIdentityError::UnsupportedCarriage)
}

/// Builds member receipts from the actual validated selected callback and direct Meter values.
///
/// This produces a local static provider identity only. It does not attach source callbacks,
/// warm owner states, a terminal measurement, or READY authority.
pub fn build_selected_meter_provider_identity(
    roster: &SelectedMeterProviderRoster,
    providers: &SelectedIndependentMeterCallbackProviders,
    direct: &MeterDirectProviderInventory,
    selected_profile_id: &str,
) -> Result<SelectedMeterProviderSetIdentity, SelectedMeterProviderIdentityError> {
    let values = providers.providers();
    let fft_source = values
        .feature_fft
        .receipt()
        .provider_file_sha256()
        .to_bytes();
    let window_source = values.window.receipt().provider_file_sha256().to_bytes();
    let component_source = values
        .component_parameters
        .receipt()
        .provider_file_sha256()
        .to_bytes();

    let diffusion_math_source =
        f03w_source_hash(providers, "parameters/diffusion-mathematical-v1.json")?;
    let diffusion_constructor_source = f03w_source_hash(
        providers,
        "parameters/diffusion-constructor-mathematical-v1.json",
    )?;

    let fft_value_words = feature_fft_value_words(&values.feature_fft);
    let window_value_words = window_q15_value_words(&values.window);
    let component_value_words = component_value_words(&values.component_parameters);
    let diffusion_math_words = values.diffusion.mathematical_value_words();
    let diffusion_constructor_words = values.diffusion.constructor_value_words();

    let mut receipts = Vec::with_capacity(roster.members().len());
    for expected in roster.members() {
        let member_id = expected.member_id();
        let receipt = match member_id {
            "D11-FFTS" | "D11-FFTT" | "D11-FFTP" | "D11-FFTR" | "D11-FFTC" => member_receipt(
                expected,
                "spatial_control",
                "ValidatedFeatureFft4096MathProvider",
                "v1",
                selected_profile_id,
                fft_source,
                &fft_value_words,
            )?,
            "D11-K" => member_receipt(
                expected,
                "spatial_audio",
                "ValidatedWindowQ15Provider",
                "v1",
                selected_profile_id,
                window_source,
                &window_value_words,
            )?,
            "D11-F" => member_receipt(
                expected,
                "spatial_audio",
                "ValidatedComponentParameterProvider",
                "v1",
                selected_profile_id,
                component_source,
                &component_value_words,
            )?,
            "D11-PM" | "D11-PL" | "D11-DC" => member_receipt(
                expected,
                "spatial_audio",
                "validate_diffusion_filter_parameters_from_f03w_bundle",
                "v1",
                selected_profile_id,
                diffusion_math_source,
                &diffusion_value_words(&diffusion_math_words),
            )?,
            "D11-DP" => member_receipt(
                expected,
                "spatial_audio",
                "validate_diffusion_filter_parameters_from_f03w_bundle",
                "v1",
                selected_profile_id,
                diffusion_constructor_source,
                &diffusion_value_words(&diffusion_constructor_words),
            )?,
            "D12-R" => {
                let source = direct_asset_hash(direct, MeterDirectProviderAssetId::OarGeometry)?;
                let words = i32_matrix_words(direct.oar_geometry().generated_i32_rows());
                member_receipt(
                    expected,
                    "meter",
                    "validate_oar_geometry",
                    "v1",
                    selected_profile_id,
                    source,
                    &words,
                )?
            }
            "D12-DV" => {
                let source = direct_asset_hash(direct, MeterDirectProviderAssetId::SharedDv32)?;
                let words = dv32_direct_value_words(direct.shared_dv32_parameters());
                member_receipt(
                    expected,
                    "meter",
                    "SharedGainDelayDirectParameters::validate_approved_asset",
                    "v1",
                    selected_profile_id,
                    source,
                    &words,
                )?
            }
            "D12-K" => {
                let source = direct_asset_hash(direct, MeterDirectProviderAssetId::KMatrix)?;
                let words = k_matrix_value_words(direct.k_matrix());
                member_receipt(
                    expected,
                    "meter",
                    "generate_k_matrix_from_approved_direct_assets",
                    "v1",
                    selected_profile_id,
                    source,
                    &words,
                )?
            }
            "D12-TP" => {
                let source = direct_asset_hash(direct, MeterDirectProviderAssetId::TruePeak)?;
                let words = true_peak_value_words(direct.true_peak_parameters());
                member_receipt(
                    expected,
                    "meter",
                    "validate_approved_provider_assets",
                    "v1",
                    selected_profile_id,
                    source,
                    &words,
                )?
            }
            "D12-L" => {
                let source = direct_asset_hash(direct, MeterDirectProviderAssetId::LogConstants)?;
                let words = u32_value_words(direct.integrated_meter_parameters().log_words());
                member_receipt(
                    expected,
                    "meter",
                    "IntegratedMeterParameters::from_approved_log_asset",
                    "v1",
                    selected_profile_id,
                    source,
                    &words,
                )?
            }
            id if id.starts_with("F06I ") => {
                rule_member_receipt(expected, "meter", selected_profile_id)?
            }
            id if id.starts_with("F06L ") => {
                rule_member_receipt(expected, "meter", selected_profile_id)?
            }
            id if id.starts_with("F06W ") => {
                rule_member_receipt(expected, "meter", selected_profile_id)?
            }
            _ => {
                return Err(SelectedMeterProviderIdentityError::MissingRosterMember(
                    member_id.to_owned(),
                ));
            }
        };
        receipts.push(receipt);
    }

    SelectedMeterProviderSetIdentity::new(roster.members(), selected_profile_id, receipts)
        .map_err(SelectedMeterProviderIdentityError::ProviderSet)
}

/// Binds the static receipt to the exact prepared programme attempt and source extent.
pub fn bind_provider_identity_to_source(
    source_admission: jocforge_application::meter_source_admission::MeterSourceAdmissionReceipt,
    provider_identity: SelectedMeterProviderSetIdentity,
) -> Result<MeterSourceProviderSetAdmissionReceipt, SelectedMeterProviderIdentityError> {
    source_admission
        .with_provider_set_identity(provider_identity)
        .map_err(SelectedMeterProviderIdentityError::SourceAdmission)
}

fn member_receipt(
    expected: &ExpectedMeterProviderMember,
    owner_id: &str,
    validator_id: &str,
    validator_version: &str,
    selected_profile_id: &str,
    actual_source_sha256: [u8; 32],
    value_words: &[u8],
) -> Result<MeterProviderMemberReceipt, SelectedMeterProviderIdentityError> {
    if expected.approved_source_sha256() != &actual_source_sha256 {
        return Err(SelectedMeterProviderIdentityError::SourceIdentityMismatch(
            expected.member_id().to_owned(),
        ));
    }
    MeterProviderMemberReceipt::new(
        expected.member_id(),
        owner_id,
        actual_source_sha256,
        validator_id,
        validator_version,
        selected_profile_id,
        validated_value_words_sha256(value_words),
    )
    .map_err(SelectedMeterProviderIdentityError::ProviderSet)
}

fn rule_member_receipt(
    expected: &ExpectedMeterProviderMember,
    owner_id: &str,
    selected_profile_id: &str,
) -> Result<MeterProviderMemberReceipt, SelectedMeterProviderIdentityError> {
    let source = *expected.approved_source_sha256();
    if expected.approved_source_sha256() != &source {
        return Err(SelectedMeterProviderIdentityError::SourceIdentityMismatch(
            expected.member_id().to_owned(),
        ));
    }
    MeterProviderMemberReceipt::new(
        expected.member_id(),
        owner_id,
        source,
        "approved-contract-member-hash",
        "v1",
        selected_profile_id,
        source,
    )
    .map_err(SelectedMeterProviderIdentityError::ProviderSet)
}

fn direct_asset_hash(
    direct: &MeterDirectProviderInventory,
    expected_id: MeterDirectProviderAssetId,
) -> Result<[u8; 32], SelectedMeterProviderIdentityError> {
    direct
        .direct_assets()
        .iter()
        .find(|asset| asset.id() == expected_id)
        .map(|asset| *asset.sha256())
        .ok_or(SelectedMeterProviderIdentityError::MissingDirectAsset(
            expected_id,
        ))
}

fn f03w_source_hash(
    providers: &SelectedIndependentMeterCallbackProviders,
    path: &str,
) -> Result<[u8; 32], SelectedMeterProviderIdentityError> {
    providers
        .consumed_spatial_provider_entries()
        .iter()
        .find(|entry| entry.path() == path)
        .and_then(|entry| decode_sha256(entry.payload_sha256_hex()))
        .ok_or_else(|| SelectedMeterProviderIdentityError::MissingSpatialLineage(path.to_owned()))
}

fn feature_fft_value_words(provider: &ValidatedFeatureFft4096MathProvider) -> Vec<u8> {
    let mut words = Vec::new();
    append_u32(&mut words, provider.fft_length());
    append_u32(
        &mut words,
        u32::try_from(provider.stages().len()).expect("FFT stage count fits u32"),
    );
    for stage in provider.stages() {
        append_u32(&mut words, stage.length());
        append_u32(
            &mut words,
            u32::try_from(stage.roots_u32().len()).expect("FFT root count fits u32"),
        );
        for root in stage.roots_u32() {
            append_u32(&mut words, root[0]);
            append_u32(&mut words, root[1]);
        }
    }
    words
}

fn window_q15_value_words(provider: &ValidatedWindowQ15Provider) -> Vec<u8> {
    let mut words = Vec::new();
    append_u32(&mut words, 32);
    for value in provider.receiver_blend32() {
        append_u16(&mut words, *value);
    }
    append_u32(&mut words, 64);
    for value in provider.component_kernel64() {
        append_u16(&mut words, *value);
    }
    words
}

fn component_value_words(provider: &ValidatedComponentParameterProvider) -> Vec<u8> {
    let mut words = Vec::new();
    append_u32(&mut words, 8);
    for value in provider.q15_integers() {
        append_u16(&mut words, *value);
    }
    append_u32_words(&mut words, provider.coefficient_words());
    append_u32(&mut words, provider.reciprocal_pivot_word());
    append_u32(&mut words, provider.target_pivot_word());
    append_u32(&mut words, provider.envelope_weight_word());
    words
}

fn diffusion_value_words(words: &[u32]) -> Vec<u8> {
    u32_value_words(words)
}

fn u32_value_words(words: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(words.len() * 4);
    append_u32_words(&mut bytes, words);
    bytes
}

fn i32_matrix_words(rows: &[[i32; 4]; 8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8 * 4 * 4);
    for row in rows {
        for value in row {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    bytes
}

fn dv32_direct_value_words(direct: &SharedGainDelayDirectParameters) -> Vec<u8> {
    let mut bytes = Vec::new();
    append_u32(&mut bytes, 8);
    for value in direct.q15_integers() {
        append_u16(&mut bytes, *value);
    }
    append_u32_words(&mut bytes, direct.coefficient_words());
    append_u32(&mut bytes, direct.reciprocal_pivot_word());
    append_u32(&mut bytes, direct.target_pivot_word());
    append_u32(&mut bytes, direct.envelope_weight_word());
    bytes
}

fn k_matrix_value_words(provider: &KMatrixProvider) -> Vec<u8> {
    let mut bytes = Vec::new();
    append_u32_words(&mut bytes, provider.complete_words());
    append_u32_words(&mut bytes, provider.lower_words());
    append_u32_words(&mut bytes, provider.upper_words());
    append_u32(&mut bytes, provider.noise_word());
    bytes
}

fn true_peak_value_words(provider: &TruePeakParameters) -> Vec<u8> {
    let mut bytes = Vec::new();
    append_u32_words(&mut bytes, provider.stage_a_words());
    append_u32_words(&mut bytes, provider.stage_b_words());
    bytes
}

fn append_u32_words(bytes: &mut Vec<u8>, words: &[u32]) {
    for word in words {
        append_u32(bytes, *word);
    }
}

fn append_u32(bytes: &mut Vec<u8>, word: u32) {
    bytes.extend_from_slice(&word.to_le_bytes());
}

fn append_u16(bytes: &mut Vec<u8>, word: u16) {
    bytes.extend_from_slice(&word.to_le_bytes());
}

fn decode_sha256(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let mut digest = [0; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let high = decode_nibble(pair[0])?;
        let low = decode_nibble(pair[1])?;
        digest[index] = (high << 4) | low;
    }
    Some(digest)
}

fn decode_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}
