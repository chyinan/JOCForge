// pattern: Imperative Shell
use jocforge_domain::frontend::{
    CurrentFeaturesProviderError, FeatureFft4096MathRoute, PackageManifestEvidence,
    ProviderFileEvidence, ProviderSha256, validate_current_features_asset_subset_manifest,
    validate_current_features_package_manifest, validate_feature_fft4096_math_provider,
    validate_feature_processor_math_provider,
};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;

const METER_ASSET_SUBSET_MANIFEST: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-APPROVED-PARAMETER-ASSETS-V1/MANIFEST.json"
);
const METER_FEATURE_PROCESSOR: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-APPROVED-PARAMETER-ASSETS-V1/parameters/feature-processor-math-v1.json"
);
const METER_FEATURE_FFT: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-APPROVED-PARAMETER-ASSETS-V1/parameters/feature-fft4096-math-v1.json"
);

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/current_features")
}

fn manifest_evidence() -> PackageManifestEvidence {
    let path = fixture_root().join("../f01e_provider/MANIFEST.json");
    PackageManifestEvidence::new(Arc::from(std::fs::read(path).unwrap()), None)
}

fn provider_evidence(name: &str) -> ProviderFileEvidence {
    ProviderFileEvidence::new(
        Arc::from(std::fs::read(fixture_root().join(name)).unwrap()),
        None,
    )
}

#[test]
fn approved_non_exhaustive_asset_manifest_validates_the_exact_current_features_pair() {
    let manifest_evidence = PackageManifestEvidence::new(
        Arc::from(METER_ASSET_SUBSET_MANIFEST),
        Some("approved Meter asset subset manifest".to_owned()),
    );
    let manifest = validate_current_features_asset_subset_manifest(&manifest_evidence)
        .expect("the exact approved subset manifest authenticates the CurrentFeatures entries");
    assert_eq!(
        manifest.manifest_sha256().to_hex(),
        "f802dd4b0576fe72b9f6be5538974cd91149df26c8aa01f9efa6822783d4df04"
    );

    let processor_evidence = ProviderFileEvidence::new(
        Arc::from(METER_FEATURE_PROCESSOR),
        Some("approved feature processor asset".to_owned()),
    );
    let processor = validate_feature_processor_math_provider(&processor_evidence, &manifest)
        .expect("the exact feature processor asset validates");
    let fft_evidence = ProviderFileEvidence::new(
        Arc::from(METER_FEATURE_FFT),
        Some("approved feature FFT asset".to_owned()),
    );
    let fft = validate_feature_fft4096_math_provider(&fft_evidence, &manifest)
        .expect("the exact feature FFT asset validates");

    for lineage in [
        processor.receipt().validated_manifest_lineage(),
        fft.receipt().validated_manifest_lineage(),
    ] {
        assert_eq!(
            lineage.manifest_sha256().to_hex(),
            "f802dd4b0576fe72b9f6be5538974cd91149df26c8aa01f9efa6822783d4df04"
        );
        assert_eq!(
            lineage.approved_source_manifest_sha256(),
            "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
        );
    }
    assert_eq!(processor.sample_rate(), 48_000);
    assert_eq!(processor.coefficient_u32().len(), 2_048);
    assert_eq!(fft.fft_length(), 4_096);
    assert_eq!(fft.stages().len(), 12);

    // These exact inputs remain input-only: the provider assets themselves retain NOT_RUN.
    let processor_json: Value = serde_json::from_slice(METER_FEATURE_PROCESSOR).unwrap();
    let fft_json: Value = serde_json::from_slice(METER_FEATURE_FFT).unwrap();
    assert_eq!(processor_json["real_ADM_new_route_acceptance"], "NOT_RUN");
    assert_eq!(fft_json["real_ADM_new_route_acceptance"], "NOT_RUN");
    let subset_json: Value = serde_json::from_slice(METER_ASSET_SUBSET_MANIFEST).unwrap();
    assert_eq!(subset_json["files"].as_array().unwrap().len(), 18);
    assert_eq!(
        subset_json["scope"],
        "approved nonsecret assets; layout and remaining transitive dependencies not claimed complete"
    );
}

#[test]
fn approved_asset_subset_rejects_unbound_manifest_metadata_and_provider_bytes() {
    let forged_manifest = PackageManifestEvidence::from_observed(
        Arc::from(METER_ASSET_SUBSET_MANIFEST),
        None,
        METER_ASSET_SUBSET_MANIFEST.len() as u64 + 1,
        ProviderSha256::digest(METER_ASSET_SUBSET_MANIFEST),
    );
    assert!(matches!(
        validate_current_features_asset_subset_manifest(&forged_manifest),
        Err(CurrentFeaturesProviderError::ObservedManifestEvidenceMismatch)
    ));

    let manifest_evidence =
        PackageManifestEvidence::new(Arc::from(METER_ASSET_SUBSET_MANIFEST), None);
    let manifest = validate_current_features_asset_subset_manifest(&manifest_evidence).unwrap();
    let mut changed = METER_FEATURE_PROCESSOR.to_vec();
    changed[0] ^= 1;
    let changed_processor = ProviderFileEvidence::new(Arc::from(changed), None);
    assert!(matches!(
        validate_feature_processor_math_provider(&changed_processor, &manifest),
        Err(CurrentFeaturesProviderError::ProviderIdentityMismatch)
    ));
}

#[test]
fn approved_math_providers_yield_exact_typed_words_and_lineage_receipts() {
    let manifest = validate_current_features_package_manifest(&manifest_evidence()).unwrap();
    let processor_evidence = provider_evidence("feature-processor-math-v1.json");
    let processor =
        validate_feature_processor_math_provider(&processor_evidence, &manifest).unwrap();
    assert_eq!(processor.window_length(), 3072);
    assert_eq!(processor.sample_rate(), 48_000);
    assert_eq!(processor.fft_length(), 4096);
    assert_eq!(processor.component_count(), 2048);
    assert_eq!(processor.vector_count(), 24);
    assert_eq!(processor.coefficient_u32().len(), 2048);
    assert_eq!(
        processor.window_counts(),
        &[
            4, 5, 6, 8, 9, 11, 14, 17, 20, 25, 30, 37, 45, 54, 66, 80, 97, 118, 144, 175, 213, 259,
            315, 296
        ]
    );
    assert_eq!(processor.fast_decay_u32(), 0x3F06_FCA3);
    assert_eq!(processor.slow_retain_u32(), 0x3F4E_D1AB);
    let processor_json: Value = serde_json::from_slice(processor_evidence.bytes()).unwrap();
    let expected_coefficients: Vec<u32> = processor_json["coefficient_u32"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_u64().unwrap() as u32)
        .collect();
    assert_eq!(
        processor.coefficient_u32().as_slice(),
        expected_coefficients
    );
    assert_eq!(
        processor.receipt().provider_file_sha256().to_hex(),
        "c2cc80eed698285a7a022324bfed36e04c3b6e682cb21674b6316b3ec6a2ec0b"
    );
    assert_eq!(processor.receipt().provider_file_bytes(), 35_441);
    assert_eq!(
        processor
            .receipt()
            .validated_manifest_lineage()
            .manifest_sha256()
            .to_hex(),
        "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
    );
    assert_eq!(
        processor
            .receipt()
            .validated_manifest_lineage()
            .provider_path(),
        "parameters/feature-processor-math-v1.json"
    );
    assert_eq!(
        processor
            .receipt()
            .validated_manifest_lineage()
            .manifest_entry_bytes(),
        35_441
    );
    assert_eq!(
        processor
            .receipt()
            .validated_manifest_lineage()
            .manifest_entry_sha256(),
        processor.receipt().provider_file_sha256()
    );
    assert_eq!(
        processor.receipt().schema_id(),
        "feature-processor-math-parameters.v1"
    );

    let fft_evidence = provider_evidence("feature-fft4096-math-v1.json");
    let fft = validate_feature_fft4096_math_provider(&fft_evidence, &manifest).unwrap();
    assert_eq!(fft.fft_length(), 4096);
    assert_eq!(
        fft.route(),
        FeatureFft4096MathRoute::RealFft4096Radix2MathV1
    );
    assert_eq!(fft.stages().len(), 12);
    let fft_json: Value = serde_json::from_slice(fft_evidence.bytes()).unwrap();
    for (stage, expected) in fft
        .stages()
        .iter()
        .zip(fft_json["stages"].as_array().unwrap())
    {
        assert_eq!(stage.length(), expected["length"].as_u64().unwrap() as u32);
        let expected_roots: Vec<[u32; 2]> = expected["roots_u32"]
            .as_array()
            .unwrap()
            .iter()
            .map(|pair| {
                [
                    pair[0].as_u64().unwrap() as u32,
                    pair[1].as_u64().unwrap() as u32,
                ]
            })
            .collect();
        assert_eq!(stage.roots_u32(), expected_roots);
    }
    assert_eq!(
        fft.receipt().provider_file_sha256().to_hex(),
        "aea85e4fb5dcb0354ac11790466d324f1020a8bf5db3f9d981bc46755751fb33"
    );
    assert_eq!(fft.receipt().provider_file_bytes(), 279_305);
    assert_eq!(
        fft.receipt().validated_manifest_lineage().provider_path(),
        "parameters/feature-fft4096-math-v1.json"
    );
    assert_eq!(
        fft.receipt()
            .validated_manifest_lineage()
            .manifest_entry_bytes(),
        279_305
    );
    assert_eq!(
        fft.receipt()
            .validated_manifest_lineage()
            .manifest_entry_sha256(),
        fft.receipt().provider_file_sha256()
    );
    assert_eq!(
        fft.receipt().schema_id(),
        "feature-fft4096-math-parameters.v1"
    );
    assert_eq!(
        fft.receipt().route(),
        FeatureFft4096MathRoute::RealFft4096Radix2MathV1
    );
}

#[test]
fn package_manifest_identity_rejects_before_json_parsing() {
    let malformed = PackageManifestEvidence::new(Arc::from(&b"{\"files\":["[..]), None);
    assert!(matches!(
        validate_current_features_package_manifest(&malformed),
        Err(CurrentFeaturesProviderError::PackageManifestIdentityMismatch)
    ));
}

#[test]
fn observed_manifest_metadata_must_match_actual_bytes() {
    let evidence = manifest_evidence();
    let forged = PackageManifestEvidence::from_observed(
        Arc::from(evidence.bytes()),
        None,
        evidence.observed_byte_length() + 1,
        evidence.observed_sha256(),
    );
    assert!(matches!(
        validate_current_features_package_manifest(&forged),
        Err(CurrentFeaturesProviderError::ObservedManifestEvidenceMismatch)
    ));
}

#[test]
fn provider_identity_rejects_changed_bytes_before_utf8_or_json_parsing() {
    let manifest = validate_current_features_package_manifest(&manifest_evidence()).unwrap();
    let changed = ProviderFileEvidence::new(Arc::from(&b"\xff"[..]), None);
    assert!(matches!(
        validate_feature_processor_math_provider(&changed, &manifest),
        Err(CurrentFeaturesProviderError::ProviderIdentityMismatch)
    ));
}

#[test]
fn provider_observation_must_match_actual_bytes() {
    let manifest = validate_current_features_package_manifest(&manifest_evidence()).unwrap();
    let evidence = provider_evidence("feature-processor-math-v1.json");
    let forged = ProviderFileEvidence::from_observed(
        Arc::from(evidence.bytes()),
        None,
        evidence.observed_byte_length() + 1,
        evidence.observed_sha256(),
    );
    assert!(matches!(
        validate_feature_processor_math_provider(&forged, &manifest),
        Err(CurrentFeaturesProviderError::ObservedProviderEvidenceMismatch)
    ));
}

#[test]
fn sampled_byte_changes_never_cross_manifest_or_provider_identity_gates() {
    let approved_manifest = manifest_evidence();
    for offset in [
        0,
        1,
        approved_manifest.bytes().len() / 3,
        approved_manifest.bytes().len() / 2,
        approved_manifest.bytes().len() - 1,
    ] {
        let mut changed = approved_manifest.bytes().to_vec();
        changed[offset] ^= 1;
        let evidence = PackageManifestEvidence::new(Arc::from(changed), None);
        assert!(matches!(
            validate_current_features_package_manifest(&evidence),
            Err(CurrentFeaturesProviderError::PackageManifestIdentityMismatch)
        ));
    }

    let manifest = validate_current_features_package_manifest(&approved_manifest).unwrap();
    let processor = provider_evidence("feature-processor-math-v1.json");
    for offset in [
        0,
        1,
        processor.bytes().len() / 3,
        processor.bytes().len() / 2,
        processor.bytes().len() - 1,
    ] {
        let mut changed = processor.bytes().to_vec();
        changed[offset] ^= 1;
        let evidence = ProviderFileEvidence::new(Arc::from(changed), None);
        assert!(matches!(
            validate_feature_processor_math_provider(&evidence, &manifest),
            Err(CurrentFeaturesProviderError::ProviderIdentityMismatch)
        ));
    }

    let fft = provider_evidence("feature-fft4096-math-v1.json");
    for offset in [
        0,
        1,
        fft.bytes().len() / 3,
        fft.bytes().len() / 2,
        fft.bytes().len() - 1,
    ] {
        let mut changed = fft.bytes().to_vec();
        changed[offset] ^= 1;
        let evidence = ProviderFileEvidence::new(Arc::from(changed), None);
        assert!(matches!(
            validate_feature_fft4096_math_provider(&evidence, &manifest),
            Err(CurrentFeaturesProviderError::ProviderIdentityMismatch)
        ));
    }
}
