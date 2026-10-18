// pattern: Functional Core
use jocforge_domain::frontend::{PackageManifestEvidence, ProviderFileEvidence, ProviderSha256};
use jocforge_domain::spatial::{
    F04ParameterProviderError, QmfFunctionalArraySourceClass, ValidatedF04ParameterPackageManifest,
    validate_descriptor_mathematical_provider, validate_f04_parameter_package_manifest,
    validate_qmf_mathematical_fft_provider, validate_qmf_receiver_functional_provider,
};
use std::sync::Arc;

const MANIFEST_BYTES: &[u8] = include_bytes!("fixtures/f01e_provider/MANIFEST.json");
const QMF_RECEIVER_BYTES: &[u8] =
    include_bytes!("fixtures/f04_parameters/qmf-receiver-functional-v1.json");
const QMF_FFT_BYTES: &[u8] = include_bytes!("fixtures/f04_parameters/qmf-mathematical-fft-v1.json");
const DESCRIPTOR_BYTES: &[u8] =
    include_bytes!("fixtures/f04_parameters/descriptor-mathematical.json");

fn validated_manifest() -> ValidatedF04ParameterPackageManifest {
    let evidence = PackageManifestEvidence::new(
        Arc::<[u8]>::from(MANIFEST_BYTES),
        Some("shared repository manifest fixture".to_owned()),
    );
    validate_f04_parameter_package_manifest(&evidence)
        .expect("the existing package manifest validates all five F04 provider entries")
}

fn provider_evidence(bytes: &'static [u8], name: &str) -> ProviderFileEvidence {
    ProviderFileEvidence::new(Arc::<[u8]>::from(bytes), Some(name.to_owned()))
}

fn assert_finite_words(words: impl IntoIterator<Item = u32>) {
    assert!(
        words
            .into_iter()
            .all(|word| f32::from_bits(word).is_finite())
    );
}

#[test]
fn validates_qmf_receiver_functional_words_and_source_qualification() {
    let manifest = validated_manifest();
    let provider = validate_qmf_receiver_functional_provider(
        &provider_evidence(QMF_RECEIVER_BYTES, "qmf receiver fixture"),
        &manifest,
    )
    .expect("identity-matched receiver functional parameters validate");

    assert_eq!(provider.qwin_synthesis_words().len(), 640);
    assert_eq!(provider.band0_real_mode1_words().len(), 21);
    assert_eq!(provider.band0_imag_mode1_words().len(), 21);
    assert_eq!(provider.lfe_post_filter_words().len(), 20);
    assert_eq!(provider.descriptor_threshold_words().len(), 127);
    assert_finite_words(
        provider
            .qwin_synthesis_words()
            .iter()
            .chain(provider.band0_real_mode1_words())
            .chain(provider.band0_imag_mode1_words())
            .chain(provider.lfe_post_filter_words())
            .chain(provider.descriptor_threshold_words())
            .copied()
            .chain(provider.scalar_words().iter().copied()),
    );
    assert_eq!(
        provider.array_source_classes(),
        &[
            QmfFunctionalArraySourceClass::PublicStandardFunctionalValues,
            QmfFunctionalArraySourceClass::AuthorizedNonsecretCompiledAudioFunctionalConstants,
            QmfFunctionalArraySourceClass::AuthorizedNonsecretCompiledAudioFunctionalConstants,
            QmfFunctionalArraySourceClass::AuthorizedNonsecretFunctionalConstantsBoundToFrozenBitExactModel,
            QmfFunctionalArraySourceClass::AuthorizedNonsecretCompiledAudioFunctionalConstants,
        ]
    );

    let receipt = provider.receipt();
    assert_eq!(receipt.provider_file_bytes(), 20_325);
    assert_eq!(
        receipt.provider_file_sha256().to_hex(),
        "30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c"
    );
    assert_eq!(
        receipt.delivery_scope(),
        "internal isolated implementation only; no public or third-party redistribution"
    );
    assert_eq!(
        receipt.authorization(),
        "all ordinary non-secret functional constants may be delivered; vendor-secret credentials remain excluded"
    );
    assert_eq!(
        receipt.qmf_fft_provider(),
        "parameters/qmf-mathematical-fft-v1.json"
    );
    assert_eq!(
        receipt.descriptor_gain_provider(),
        "parameters/descriptor-mathematical.json"
    );
    assert_eq!(
        receipt.receiver_blend_provider(),
        "parameters/window-q15.json"
    );
    assert_eq!(
        receipt.not_included(),
        &[
            "vendor-secret credentials",
            "authentication secrets",
            "research source",
            "raw original table artifacts",
            "compiled images",
            "addresses",
        ]
    );
    assert_eq!(
        receipt.validated_manifest_lineage().provider_path(),
        "parameters/qmf-receiver-functional-v1.json"
    );
}

#[test]
fn validates_qmf_fft_shape_route_and_unrun_qualification() {
    let manifest = validated_manifest();
    let provider = validate_qmf_mathematical_fft_provider(
        &provider_evidence(QMF_FFT_BYTES, "qmf mathematical FFT fixture"),
        &manifest,
    )
    .expect("identity-matched mathematical provider validates without running an FFT");

    assert_eq!(
        provider
            .stages()
            .iter()
            .map(jocforge_domain::spatial::QmfFftRootStage::length)
            .collect::<Vec<_>>(),
        [2, 4, 8, 16, 32, 64]
    );
    assert_eq!(
        provider
            .stages()
            .iter()
            .map(|stage| stage.roots().len())
            .collect::<Vec<_>>(),
        [1, 2, 4, 8, 16, 32]
    );
    assert!(
        provider
            .stages()
            .iter()
            .flat_map(|stage| stage.roots().iter().flatten())
            .chain(provider.rotation_words())
            .all(|word| f32::from_bits(*word).is_finite())
    );
    assert_eq!(provider.rotation_words().len(), 128);
    assert_eq!(provider.receipt().route(), "QMF_FFT_RADIX2_MATH_V1");
    assert_eq!(
        provider.receipt().source(),
        "MATHEMATICAL_GENERATION_NO_COMPILED_VALUES_USED"
    );
    assert_eq!(provider.receipt().audio_acceptance(), "NOT_RUN");
    assert_eq!(provider.receipt().provider_file_bytes(), 6_983);
    assert_eq!(
        provider
            .receipt()
            .validated_manifest_lineage()
            .provider_path(),
        "parameters/qmf-mathematical-fft-v1.json"
    );
}

#[test]
fn validates_descriptor_words_without_erasing_candidate_status() {
    let manifest = validated_manifest();
    let provider = validate_descriptor_mathematical_provider(
        &provider_evidence(DESCRIPTOR_BYTES, "descriptor mathematical fixture"),
        &manifest,
    )
    .expect("identity-matched descriptor words validate without claiming acceptance");

    assert_eq!(provider.gain_words().len(), 128);
    assert_eq!(provider.threshold_words().len(), 127);
    assert_finite_words(provider.gain_words().iter().copied());
    assert_finite_words(provider.threshold_words().iter().copied());
    assert!(
        provider
            .gain_words()
            .iter()
            .all(|word| f32::from_bits(*word) > 0.0)
    );
    let receipt = provider.receipt();
    assert_eq!(receipt.gain_research_comparison(), "PASS_ALL128_WORDS");
    assert_eq!(
        receipt.threshold_research_comparison(),
        "36_WORD_DIFFERENCES_NOT_IDENTICAL"
    );
    assert_eq!(
        receipt.threshold_route_status(),
        "MATHEMATICAL_CANDIDATE_AUDIO_ACCEPTANCE_PENDING"
    );
    assert_eq!(receipt.provider_file_bytes(), 4_861);
    assert_eq!(
        receipt.validated_manifest_lineage().provider_path(),
        "parameters/descriptor-mathematical.json"
    );
}

#[test]
fn rejects_crosswired_provider_files_before_json_parsing() {
    let manifest = validated_manifest();
    let descriptor = provider_evidence(DESCRIPTOR_BYTES, "wrong provider file");
    let receiver = provider_evidence(QMF_RECEIVER_BYTES, "wrong provider file");
    let fft = provider_evidence(QMF_FFT_BYTES, "wrong provider file");

    assert!(matches!(
        validate_qmf_receiver_functional_provider(&descriptor, &manifest),
        Err(F04ParameterProviderError::ProviderIdentityMismatch)
    ));
    assert!(matches!(
        validate_qmf_mathematical_fft_provider(&receiver, &manifest),
        Err(F04ParameterProviderError::ProviderIdentityMismatch)
    ));
    assert!(matches!(
        validate_descriptor_mathematical_provider(&fft, &manifest),
        Err(F04ParameterProviderError::ProviderIdentityMismatch)
    ));
}

#[test]
fn rejects_inconsistent_observed_provider_identity_before_parsing() {
    let manifest = validated_manifest();
    let evidence = ProviderFileEvidence::from_observed(
        Arc::<[u8]>::from(QMF_RECEIVER_BYTES),
        Some("wrong observed identity".to_owned()),
        QMF_RECEIVER_BYTES.len() as u64,
        ProviderSha256::digest(b"not the observed provider bytes"),
    );

    assert!(matches!(
        validate_qmf_receiver_functional_provider(&evidence, &manifest),
        Err(F04ParameterProviderError::ObservedProviderEvidenceMismatch)
    ));
}

#[test]
fn rejects_inconsistent_observed_manifest_identity_before_parsing() {
    let evidence = PackageManifestEvidence::from_observed(
        Arc::<[u8]>::from(MANIFEST_BYTES),
        Some("wrong observed manifest identity".to_owned()),
        MANIFEST_BYTES.len() as u64 + 1,
        ProviderSha256::digest(MANIFEST_BYTES),
    );

    assert!(matches!(
        validate_f04_parameter_package_manifest(&evidence),
        Err(F04ParameterProviderError::ObservedManifestEvidenceMismatch)
    ));
}

#[test]
fn rejects_malformed_provider_bytes_at_the_identity_gate_before_json_parse() {
    let manifest = validated_manifest();
    let malformed = ProviderFileEvidence::new(
        Arc::<[u8]>::from(&b"not json"[..]),
        Some("malformed bytes with no approved identity".to_owned()),
    );

    assert!(matches!(
        validate_descriptor_mathematical_provider(&malformed, &manifest),
        Err(F04ParameterProviderError::ProviderIdentityMismatch)
    ));
}
