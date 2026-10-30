// pattern: Functional Core
//! Validates the selected F04 providers as one profile-bound, read-only bundle.

use crate::{
    frontend::{PackageManifestEvidence, ProviderFileEvidence, ProviderSha256},
    profile::EncodingProfile,
};

use crate::spatial::{
    DescriptorMathematicalProvider, F04ParameterProviderError, QmfMathematicalFftProvider,
    QmfReceiverFunctionalProvider, SpatialAudioJocProviders, ValidatedComponentParameterProvider,
    ValidatedWindowQ15Provider, validate_component_parameter_provider,
    validate_descriptor_mathematical_provider, validate_f04_parameter_package_manifest,
    validate_qmf_mathematical_fft_provider, validate_qmf_receiver_functional_provider,
    validate_window_q15_provider,
};

/// Caller-supplied bytes from the approved F04 parameter package.
///
/// This evidence value grants no file or process I/O. The caller owns package loading;
/// validation binds each provider to the exact approved manifest before the bundle exists.
pub struct SelectedSpatialAudioProviderEvidence<'a> {
    package_manifest: &'a PackageManifestEvidence,
    component_parameters: &'a ProviderFileEvidence,
    window_q15: &'a ProviderFileEvidence,
    qmf_receiver_functional: &'a ProviderFileEvidence,
    qmf_mathematical_fft: &'a ProviderFileEvidence,
    descriptor_mathematical: &'a ProviderFileEvidence,
}

impl<'a> SelectedSpatialAudioProviderEvidence<'a> {
    pub const fn new(
        package_manifest: &'a PackageManifestEvidence,
        component_parameters: &'a ProviderFileEvidence,
        window_q15: &'a ProviderFileEvidence,
        qmf_receiver_functional: &'a ProviderFileEvidence,
        qmf_mathematical_fft: &'a ProviderFileEvidence,
        descriptor_mathematical: &'a ProviderFileEvidence,
    ) -> Self {
        Self {
            package_manifest,
            component_parameters,
            window_q15,
            qmf_receiver_functional,
            qmf_mathematical_fft,
            descriptor_mathematical,
        }
    }
}

/// Owned, identity-validated selected F04 providers for one runtime composition.
///
/// The QMF receiver functional provider retains its approved `internal isolated
/// implementation only` delivery scope. This type validates and exposes providers for the
/// isolated runtime; it does not authorize public or third-party redistribution.
pub struct SelectedSpatialAudioProviderBundle {
    manifest_sha256: ProviderSha256,
    component_parameters: ValidatedComponentParameterProvider,
    window_q15: ValidatedWindowQ15Provider,
    qmf_receiver_functional: QmfReceiverFunctionalProvider,
    qmf_mathematical_fft: QmfMathematicalFftProvider,
    descriptor_mathematical: DescriptorMathematicalProvider,
}

impl SelectedSpatialAudioProviderBundle {
    /// Validates the exact provider set for the default selected profile.
    pub fn validate_selected(
        evidence: &SelectedSpatialAudioProviderEvidence<'_>,
    ) -> Result<Self, F04ParameterProviderError> {
        Self::validate_for_profile(EncodingProfile::selected(), evidence)
    }

    /// Validates all F04 providers against one approved package manifest and profile.
    pub fn validate_for_profile(
        profile: EncodingProfile,
        evidence: &SelectedSpatialAudioProviderEvidence<'_>,
    ) -> Result<Self, F04ParameterProviderError> {
        let manifest = validate_f04_parameter_package_manifest(evidence.package_manifest)?;
        let component_parameters = validate_component_parameter_provider(
            evidence.component_parameters,
            &manifest,
            &profile,
        )?;
        let window_q15 = validate_window_q15_provider(evidence.window_q15, &manifest)?;
        let qmf_receiver_functional =
            validate_qmf_receiver_functional_provider(evidence.qmf_receiver_functional, &manifest)?;
        let qmf_mathematical_fft =
            validate_qmf_mathematical_fft_provider(evidence.qmf_mathematical_fft, &manifest)?;
        let descriptor_mathematical =
            validate_descriptor_mathematical_provider(evidence.descriptor_mathematical, &manifest)?;

        Ok(Self {
            manifest_sha256: manifest.manifest_sha256(),
            component_parameters,
            window_q15,
            qmf_receiver_functional,
            qmf_mathematical_fft,
            descriptor_mathematical,
        })
    }

    pub const fn manifest_sha256(&self) -> ProviderSha256 {
        self.manifest_sha256
    }

    pub const fn component_parameters(&self) -> &ValidatedComponentParameterProvider {
        &self.component_parameters
    }

    pub const fn window_q15(&self) -> &ValidatedWindowQ15Provider {
        &self.window_q15
    }

    pub const fn qmf_receiver_functional(&self) -> &QmfReceiverFunctionalProvider {
        &self.qmf_receiver_functional
    }

    pub const fn qmf_mathematical_fft(&self) -> &QmfMathematicalFftProvider {
        &self.qmf_mathematical_fft
    }

    pub const fn descriptor_mathematical(&self) -> &DescriptorMathematicalProvider {
        &self.descriptor_mathematical
    }

    /// Creates the non-optional provider view required by the selected F04 ready path.
    pub fn spatial_audio_joc_providers(&self) -> SpatialAudioJocProviders<'_> {
        SpatialAudioJocProviders::new(
            &self.qmf_receiver_functional,
            &self.window_q15,
            &self.qmf_mathematical_fft,
            &self.descriptor_mathematical,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{SelectedSpatialAudioProviderBundle, SelectedSpatialAudioProviderEvidence};
    use crate::{
        frontend::{PackageManifestEvidence, ProviderFileEvidence, ProviderSha256},
        profile::{
            AuthoredInputProfile, CodecRateProfile, CoreCarrierProfile, JocDownmixProfile,
            SubstreamTopology,
        },
        spatial::F04ParameterProviderError,
    };
    use std::sync::Arc;

    const PACKAGE_MANIFEST: &[u8] =
        include_bytes!("../../../tests/fixtures/f01e_provider/MANIFEST.json");
    const COMPONENT_PARAMETERS: &[u8] = include_bytes!(
        "../../../tests/fixtures/f04_parameters/component-parameter-mathematical-v1.json"
    );
    const WINDOW_Q15: &[u8] =
        include_bytes!("../../../tests/fixtures/f04_parameters/window-q15.json");
    const QMF_RECEIVER_FUNCTIONAL: &[u8] =
        include_bytes!("../../../tests/fixtures/f04_parameters/qmf-receiver-functional-v1.json");
    const QMF_MATHEMATICAL_FFT: &[u8] =
        include_bytes!("../../../tests/fixtures/f04_parameters/qmf-mathematical-fft-v1.json");
    const DESCRIPTOR_MATHEMATICAL: &[u8] =
        include_bytes!("../../../tests/fixtures/f04_parameters/descriptor-mathematical.json");

    fn fixture_evidence() -> (
        PackageManifestEvidence,
        ProviderFileEvidence,
        ProviderFileEvidence,
        ProviderFileEvidence,
        ProviderFileEvidence,
        ProviderFileEvidence,
    ) {
        (
            PackageManifestEvidence::new(Arc::from(PACKAGE_MANIFEST), None),
            ProviderFileEvidence::new(Arc::from(COMPONENT_PARAMETERS), None),
            ProviderFileEvidence::new(Arc::from(WINDOW_Q15), None),
            ProviderFileEvidence::new(Arc::from(QMF_RECEIVER_FUNCTIONAL), None),
            ProviderFileEvidence::new(Arc::from(QMF_MATHEMATICAL_FFT), None),
            ProviderFileEvidence::new(Arc::from(DESCRIPTOR_MATHEMATICAL), None),
        )
    }

    #[test]
    fn selected_provider_bundle_validates_one_approved_manifest_and_profile() {
        let (manifest, component, window, qmf, fft, descriptor) = fixture_evidence();
        let evidence = SelectedSpatialAudioProviderEvidence::new(
            &manifest,
            &component,
            &window,
            &qmf,
            &fft,
            &descriptor,
        );
        let bundle = SelectedSpatialAudioProviderBundle::validate_selected(&evidence)
            .expect("approved selected F04 provider bundle validates");

        assert_eq!(
            bundle.manifest_sha256().to_hex(),
            "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
        );
        assert_eq!(
            bundle.qmf_receiver_functional().receipt().delivery_scope(),
            "internal isolated implementation only; no public or third-party redistribution"
        );
        assert!(
            bundle
                .qmf_receiver_functional()
                .receipt()
                .not_included()
                .contains(&"vendor-secret credentials")
        );
        assert_eq!(
            bundle.component_parameters().receipt().authorized_scope(),
            "INTERNAL_SELECTED_PROFILE_PROVIDER_VALIDATION_AND_FUTURE_CONSUMPTION"
        );
        assert_eq!(
            bundle
                .component_parameters()
                .receipt()
                .dsp_consumption_status(),
            "NOT_YET_VALIDATED"
        );

        // Constructing the borrowed view proves the bundle supplies every non-optional F04
        // ready-path dependency without executing receiver or DSP logic.
        let _providers = bundle.spatial_audio_joc_providers();
    }

    #[test]
    fn bundle_rejects_mutated_provider_bytes_before_exposing_receiver_view() {
        let (manifest, component, window, qmf, _fft, descriptor) = fixture_evidence();
        let mut changed = QMF_MATHEMATICAL_FFT.to_vec();
        changed[0] ^= 1;
        let changed = ProviderFileEvidence::new(Arc::from(changed), None);

        let evidence = SelectedSpatialAudioProviderEvidence::new(
            &manifest,
            &component,
            &window,
            &qmf,
            &changed,
            &descriptor,
        );
        let result = SelectedSpatialAudioProviderBundle::validate_selected(&evidence);
        assert!(matches!(
            result,
            Err(F04ParameterProviderError::ProviderIdentityMismatch)
        ));
    }

    #[test]
    fn bundle_rejects_unsupported_profile_before_spatial_audio_use() {
        let (manifest, component, window, qmf, fft, descriptor) = fixture_evidence();
        let profile = crate::profile::EncodingProfile::new(
            AuthoredInputProfile::Unsupported,
            CoreCarrierProfile::SelectedFivePointOneSixCarrier,
            JocDownmixProfile::SelectedMode0Phase1Index3,
            SubstreamTopology::SelectedSingleIndependentZeroDependent,
            CodecRateProfile::Selected48Khz1024Kbps6Blocks1536Samples4096Bytes,
            crate::profile::EmdfCarriageProfile::DeployedCompat,
        );

        let evidence = SelectedSpatialAudioProviderEvidence::new(
            &manifest,
            &component,
            &window,
            &qmf,
            &fft,
            &descriptor,
        );
        let result = SelectedSpatialAudioProviderBundle::validate_for_profile(profile, &evidence);
        assert!(matches!(
            result,
            Err(F04ParameterProviderError::UnsupportedProfile)
        ));
    }

    #[test]
    fn provider_package_identity_is_not_derived_from_provider_fields() {
        let (manifest, component, window, qmf, fft, descriptor) = fixture_evidence();
        let evidence = SelectedSpatialAudioProviderEvidence::new(
            &manifest,
            &component,
            &window,
            &qmf,
            &fft,
            &descriptor,
        );
        let bundle = SelectedSpatialAudioProviderBundle::validate_selected(&evidence).unwrap();
        assert_eq!(
            bundle.manifest_sha256(),
            ProviderSha256::digest(PACKAGE_MANIFEST)
        );
    }
}
