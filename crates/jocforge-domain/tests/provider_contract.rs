// pattern: Functional Core
use jocforge_domain::frontend::{
    F01EProviderError, PackageManifestEvidence, ProviderFileEvidence, ProviderSha256,
    validate_f01e_package_manifest, validate_f01e_provider,
};
use std::path::PathBuf;

fn package_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/f01e_provider")
}

fn approved_evidence() -> (PackageManifestEvidence, ProviderFileEvidence) {
    let root = package_root();
    let manifest_path = root.join("MANIFEST.json");
    let provider_path = root.join("f01e-selected-constructor-v1.json");
    (
        PackageManifestEvidence::new(
            std::fs::read(&manifest_path).unwrap().into(),
            Some(manifest_path.display().to_string()),
        ),
        ProviderFileEvidence::new(
            std::fs::read(&provider_path).unwrap().into(),
            Some(provider_path.display().to_string()),
        ),
    )
}

#[test]
fn exact_manifest_and_provider_validate_with_manifest_lineage() {
    let (manifest_evidence, provider_evidence) = approved_evidence();
    let manifest = validate_f01e_package_manifest(&manifest_evidence).unwrap();
    let validated = validate_f01e_provider(&provider_evidence, &manifest).unwrap();
    assert_eq!(validated.callback_index(), 0);
    assert_eq!(validated.object_count(), 28);
    assert_eq!(validated.canonical_bytes().len(), 4402);
    assert_eq!(validated.receipt().provider_file_bytes(), 20138);
    assert_eq!(
        validated.receipt().manifest_sha256().to_hex(),
        "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
    );
    assert_eq!(
        validated.receipt().canonical_sha256().to_hex(),
        "9c943879cd36cb45bc7538346cfdf4f71d0998d8f261f75c135045fcb59870aa"
    );
}

#[test]
fn forged_manifest_observation_is_rejected_before_parse() {
    let (manifest, _) = approved_evidence();
    let forged = PackageManifestEvidence::from_observed(
        manifest.bytes().into(),
        None,
        manifest.observed_byte_length(),
        ProviderSha256::digest(b"not the manifest"),
    );
    assert!(matches!(
        validate_f01e_package_manifest(&forged),
        Err(F01EProviderError::ObservedManifestEvidenceMismatch)
    ));
}

#[test]
fn any_manifest_byte_variant_or_other_manifest_is_rejected_before_parse() {
    let (manifest, _) = approved_evidence();
    let mut changed = manifest.bytes().to_vec();
    changed[0] ^= 1;
    for bytes in [changed, br#"{"files":[]}"#.to_vec()] {
        let evidence = PackageManifestEvidence::new(bytes.into(), None);
        assert!(matches!(
            validate_f01e_package_manifest(&evidence),
            Err(F01EProviderError::PackageManifestIdentityMismatch)
        ));
    }
}

#[test]
fn provider_observation_and_bytes_must_match_validated_manifest_entry() {
    let (manifest_evidence, provider) = approved_evidence();
    let manifest = validate_f01e_package_manifest(&manifest_evidence).unwrap();
    let forged_observation = ProviderFileEvidence::from_observed(
        provider.bytes().into(),
        None,
        provider.observed_byte_length() + 1,
        provider.observed_sha256(),
    );
    assert!(matches!(
        validate_f01e_provider(&forged_observation, &manifest),
        Err(F01EProviderError::ObservedEvidenceMismatch)
    ));

    let mut changed = provider.bytes().to_vec();
    changed[0] ^= 1;
    let changed = ProviderFileEvidence::new(changed.into(), None);
    assert!(matches!(
        validate_f01e_provider(&changed, &manifest),
        Err(F01EProviderError::ProviderIdentityMismatch)
    ));
}
