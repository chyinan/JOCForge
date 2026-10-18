// pattern: Functional Core
use jocforge_domain::frontend::{PackageManifestEvidence, ProviderFileEvidence, ProviderSha256};
use jocforge_domain::profile::{
    AuthoredInputProfile, CodecRateProfile, CoreCarrierProfile, EmdfCarriageProfile,
    EncodingProfile, JocDownmixProfile, SubstreamTopology,
};
use jocforge_domain::spatial::{
    F04ParameterProviderError, ValidatedF04ParameterPackageManifest,
    validate_component_parameter_provider, validate_f04_parameter_package_manifest,
    validate_window_q15_provider,
};
use std::sync::Arc;

const MANIFEST_BYTES: &[u8] = include_bytes!("fixtures/f01e_provider/MANIFEST.json");
const WINDOW_Q15_BYTES: &[u8] = include_bytes!("fixtures/f04_parameters/window-q15.json");
const COMPONENT_PARAMETERS_BYTES: &[u8] =
    include_bytes!("fixtures/f04_parameters/component-parameter-mathematical-v1.json");

fn validated_manifest() -> ValidatedF04ParameterPackageManifest {
    let evidence = PackageManifestEvidence::new(
        Arc::<[u8]>::from(MANIFEST_BYTES),
        Some("repository manifest fixture".to_owned()),
    );
    validate_f04_parameter_package_manifest(&evidence).expect("approved manifest fixture validates")
}

fn window_evidence() -> ProviderFileEvidence {
    ProviderFileEvidence::new(
        Arc::<[u8]>::from(WINDOW_Q15_BYTES),
        Some("window-q15 fixture".to_owned()),
    )
}

fn component_evidence() -> ProviderFileEvidence {
    ProviderFileEvidence::new(
        Arc::<[u8]>::from(COMPONENT_PARAMETERS_BYTES),
        Some("component parameter fixture".to_owned()),
    )
}

#[test]
fn validates_window_q15_provider_and_both_named_arrays() {
    let manifest = validated_manifest();
    let provider = validate_window_q15_provider(&window_evidence(), &manifest)
        .expect("identity-matched window provider validates");

    assert_eq!(provider.receiver_blend32().len(), 32);
    assert_eq!(provider.component_kernel64().len(), 64);
    assert!(
        provider
            .receiver_blend32()
            .iter()
            .chain(provider.component_kernel64())
            .all(|value| *value <= 32_768)
    );
    assert_eq!(provider.receipt().provider_file_bytes(), 2_704);
    assert_eq!(
        provider.receipt().provider_file_sha256().to_hex(),
        "a12f9cbf91f1cf540c2eae77e4d236970defa70409f00ddf62ba87bfeafbb130"
    );
}

#[test]
fn validates_component_parameter_words_and_readiness_qualification() {
    let manifest = validated_manifest();
    let profile = EncodingProfile::selected();
    let provider =
        validate_component_parameter_provider(&component_evidence(), &manifest, &profile)
            .expect("identity-matched selected-profile component parameters validate");
    let receipt = provider.receipt();

    assert_eq!(provider.q15_integers().len(), 8);
    assert!(provider.q15_integers().iter().all(|value| *value <= 32_768));
    assert_eq!(provider.coefficient_words().len(), 5);
    let mut words = provider.coefficient_words().iter().copied().chain([
        provider.reciprocal_pivot_word(),
        provider.target_pivot_word(),
        provider.envelope_weight_word(),
    ]);
    assert!(words.all(|word| f32::from_bits(word).is_finite()));
    assert!(!receipt.asset_implementation_ready());
    assert_eq!(
        receipt.authorized_scope(),
        "INTERNAL_SELECTED_PROFILE_PROVIDER_VALIDATION_AND_FUTURE_CONSUMPTION"
    );
    assert_eq!(receipt.dsp_consumption_status(), "NOT_YET_VALIDATED");
    assert_eq!(receipt.provider_file_bytes(), 1_077);
}

#[test]
fn accepts_both_approved_carriage_variants_without_claiming_dsp_validation() {
    let manifest = validated_manifest();
    for profile in [
        EncodingProfile::selected(),
        EncodingProfile::selected_with_carriage(EmdfCarriageProfile::EtsiEmdf),
    ] {
        let provider =
            validate_component_parameter_provider(&component_evidence(), &manifest, &profile)
                .expect("both approved carriage variants share the selected codec profile");
        assert_eq!(
            provider.receipt().dsp_consumption_status(),
            "NOT_YET_VALIDATED"
        );
    }
}

#[test]
fn rejects_component_provider_for_an_unsupported_profile() {
    let manifest = validated_manifest();
    let unsupported = EncodingProfile::new(
        AuthoredInputProfile::Unsupported,
        CoreCarrierProfile::SelectedFivePointOneSixCarrier,
        JocDownmixProfile::SelectedMode0Phase1Index3,
        SubstreamTopology::SelectedSingleIndependentZeroDependent,
        CodecRateProfile::Selected48Khz1024Kbps6Blocks1536Samples4096Bytes,
        EmdfCarriageProfile::DeployedCompat,
    );

    assert!(matches!(
        validate_component_parameter_provider(&component_evidence(), &manifest, &unsupported),
        Err(F04ParameterProviderError::UnsupportedProfile)
    ));
}

#[test]
fn rejects_crosswired_provider_bytes_before_json_parsing() {
    let manifest = validated_manifest();
    let component_bytes = ProviderFileEvidence::new(
        Arc::<[u8]>::from(COMPONENT_PARAMETERS_BYTES),
        Some("intentionally cross-wired".to_owned()),
    );

    assert!(matches!(
        validate_window_q15_provider(&component_bytes, &manifest),
        Err(F04ParameterProviderError::ProviderIdentityMismatch)
    ));
}

#[test]
fn rejects_inconsistent_observed_provider_identity_before_json_parsing() {
    let manifest = validated_manifest();
    let evidence = ProviderFileEvidence::from_observed(
        Arc::<[u8]>::from(WINDOW_Q15_BYTES),
        Some("intentionally wrong observation".to_owned()),
        WINDOW_Q15_BYTES.len() as u64,
        ProviderSha256::digest(b"different observed bytes"),
    );

    assert!(matches!(
        validate_window_q15_provider(&evidence, &manifest),
        Err(F04ParameterProviderError::ObservedProviderEvidenceMismatch)
    ));
}

#[test]
fn rejects_inconsistent_observed_manifest_identity_before_json_parsing() {
    let evidence = PackageManifestEvidence::from_observed(
        Arc::<[u8]>::from(MANIFEST_BYTES),
        Some("intentionally wrong manifest observation".to_owned()),
        MANIFEST_BYTES.len() as u64 + 1,
        ProviderSha256::digest(MANIFEST_BYTES),
    );

    assert!(matches!(
        validate_f04_parameter_package_manifest(&evidence),
        Err(F04ParameterProviderError::ObservedManifestEvidenceMismatch)
    ));
}
