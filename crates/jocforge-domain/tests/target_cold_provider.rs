// pattern: Functional Core
use jocforge_domain::frontend::{
    F03TargetProviderError, PackageManifestEvidence, ProviderFileEvidence,
    validate_f03_package_manifest, validate_f03_target_cold_provider,
};
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn approved_evidence() -> (PackageManifestEvidence, ProviderFileEvidence) {
    let root = fixture_root();
    let manifest_path = root.join("f01e_provider/MANIFEST.json");
    let provider_path = root.join("target_cold/f03-selected-target-cold-v1.json");
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
fn approved_f03_provider_has_private_typed_fields_and_target_only_receipt() {
    let (manifest_evidence, provider_evidence) = approved_evidence();
    let manifest = validate_f03_package_manifest(&manifest_evidence).unwrap();
    let provider = validate_f03_target_cold_provider(&provider_evidence, &manifest).unwrap();

    assert_eq!(provider.current_xyz_u32(), &[0; 45]);
    assert_eq!(provider.previous_xyz_u32(), &[0; 45]);
    assert!(
        provider
            .current_id_u32()
            .iter()
            .all(|value| *value == u32::MAX)
    );
    assert!(
        provider
            .previous_id_u32()
            .iter()
            .all(|value| *value == u32::MAX)
    );
    assert!(
        provider
            .current_attribute_u32()
            .iter()
            .all(|value| *value == 0x0000_0b00)
    );
    assert!(
        provider
            .previous_attribute_u32()
            .iter()
            .all(|value| *value == 0x0000_0b00)
    );
    assert_eq!(provider.active_u8(), &[0; 15]);
    assert_eq!(provider.previous_marker_active_u8(), &[0; 15]);

    assert!(!std::ptr::eq(
        provider.current_xyz_u32(),
        provider.previous_xyz_u32()
    ));
    assert!(!std::ptr::eq(
        provider.current_id_u32(),
        provider.previous_id_u32()
    ));
    assert!(!std::ptr::eq(
        provider.current_attribute_u32(),
        provider.previous_attribute_u32()
    ));

    assert_eq!(provider.target_projection_bytes().len(), 630);
    assert_eq!(
        provider.receipt().target_projection_sha256().to_hex(),
        "b1acfa3ce43b6ce987836330a3db3945c81b3d62d96d4973f18b302dc6d2606c"
    );
    assert_eq!(provider.receipt().target_projection_bytes(), 630);
    assert_eq!(provider.receipt().provider_generation(), 0);
    assert_eq!(
        provider.receipt().schema_id(),
        "f03-selected-target-cold.v1"
    );
    assert_eq!(
        provider
            .receipt()
            .validated_manifest_lineage()
            .manifest_sha256()
            .to_hex(),
        "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
    );
    assert_eq!(provider.receipt().provider_file_bytes(), 3139);
    assert_eq!(
        provider.receipt().provider_file_sha256().to_hex(),
        "a4b53854822a5f17a5413d6f2ded127a7366257c2a171377afcea4a84a3eaf00"
    );
}

#[test]
fn f03_manifest_is_identity_checked_before_json_parse_and_observation_is_bound() {
    let (manifest_evidence, _) = approved_evidence();
    let forged_observation = PackageManifestEvidence::from_observed(
        manifest_evidence.bytes().into(),
        None,
        manifest_evidence.observed_byte_length(),
        jocforge_domain::frontend::ProviderSha256::digest(b"not the manifest"),
    );
    assert!(matches!(
        validate_f03_package_manifest(&forged_observation),
        Err(F03TargetProviderError::ObservedManifestEvidenceMismatch)
    ));

    let malformed_bytes = br#"{"files":["#.to_vec();
    let malformed = PackageManifestEvidence::from_observed(
        malformed_bytes.into(),
        None,
        0,
        jocforge_domain::frontend::ProviderSha256::digest(b"forged observation"),
    );
    assert!(matches!(
        validate_f03_package_manifest(&malformed),
        Err(F03TargetProviderError::PackageManifestIdentityMismatch)
    ));
}

#[test]
fn f03_provider_observation_and_manifest_identity_precede_utf8_or_json_parse() {
    let (manifest_evidence, provider_evidence) = approved_evidence();
    let manifest = validate_f03_package_manifest(&manifest_evidence).unwrap();
    let forged_observation = ProviderFileEvidence::from_observed(
        provider_evidence.bytes().into(),
        None,
        provider_evidence.observed_byte_length() + 1,
        provider_evidence.observed_sha256(),
    );
    assert!(matches!(
        validate_f03_target_cold_provider(&forged_observation, &manifest),
        Err(F03TargetProviderError::ObservedProviderEvidenceMismatch)
    ));

    let changed = ProviderFileEvidence::new(vec![0xff].into(), None);
    assert!(matches!(
        validate_f03_target_cold_provider(&changed, &manifest),
        Err(F03TargetProviderError::ProviderIdentityMismatch)
    ));

    let mut duplicate_schema = br#"{"schema":"f03-selected-target-cold.v1","#.to_vec();
    duplicate_schema.extend_from_slice(&provider_evidence.bytes()[1..]);
    let duplicate_schema = ProviderFileEvidence::new(duplicate_schema.into(), None);
    assert!(matches!(
        validate_f03_target_cold_provider(&duplicate_schema, &manifest),
        Err(F03TargetProviderError::ProviderIdentityMismatch)
    ));
}
