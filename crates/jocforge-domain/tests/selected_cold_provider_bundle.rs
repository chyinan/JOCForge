use jocforge_domain::frontend::{
    PackageManifestEvidence, ProviderFileEvidence, SelectedColdContractError, SelectedCommitOwner,
    SelectedOwnerId, validate_f01b_cold_asset_manifest,
    validate_f01e_provider_from_f01b_cold_assets,
    validate_f03_target_cold_provider_from_f01b_assets, validate_f03w_provider_manifest,
    validate_installed48_state_schema, validate_selected_owner_projection,
};
use jocforge_domain::spatial::window_q15_provider::validate_window_q15_provider_from_f01b_cold_assets;
use std::sync::Arc;

const F03W_MANIFEST: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/ADM-F03W-APPROVED-PROVIDER-BUNDLE-V1/MANIFEST.json"
);
const COLD_MANIFEST: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/F01B-F03W-COLD-STATE-ASSETS-V1/MANIFEST.json"
);
const INSTALLED48_STATE: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/F01B-F03W-COLD-STATE-ASSETS-V1/parameters/adm-installed48-state-v1.json"
);
const TARGET_COLD: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/F01B-F03W-COLD-STATE-ASSETS-V1/parameters/f03-selected-target-cold-v1.json"
);
const OWNER_PROJECTION: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/F01B-F03W-COLD-STATE-ASSETS-V1/parameters/selected-owner-projection-v1.json"
);
const WINDOW_Q15: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/F01B-F03W-COLD-STATE-ASSETS-V1/parameters/window-q15.json"
);
const F01E_COLD: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/F01B-F03W-COLD-STATE-ASSETS-V1/parameters/f01e-selected-constructor-v1.json"
);

#[test]
fn independently_rooted_approved_bundles_bind_the_typed_cold_contracts() {
    let provider_manifest_evidence = PackageManifestEvidence::new(Arc::from(F03W_MANIFEST), None);
    let provider_manifest = validate_f03w_provider_manifest(&provider_manifest_evidence).unwrap();
    assert_eq!(
        provider_manifest.manifest_sha256().to_hex(),
        "9b81b736e9dd110441d3b59333afac9cbb98886f8bf4ac9854b5d74bdf57783f"
    );
    assert_eq!(provider_manifest.declared_provider_entry_count(), 48);
    assert_eq!(provider_manifest.validated_consumed_entry_count(), 3);
    assert_eq!(provider_manifest.remaining_entry_count_open(), 45);

    let cold_manifest_evidence = PackageManifestEvidence::new(Arc::from(COLD_MANIFEST), None);
    let cold_manifest = validate_f01b_cold_asset_manifest(&cold_manifest_evidence).unwrap();
    assert_eq!(
        cold_manifest.manifest_sha256().to_hex(),
        "d15672b723f8091b576c2228ed7685744f522706d89078050b6486a72a068372"
    );

    let installed48_evidence = ProviderFileEvidence::new(Arc::from(INSTALLED48_STATE), None);
    let installed48 =
        validate_installed48_state_schema(&installed48_evidence, &cold_manifest).unwrap();
    assert_eq!(installed48.dimensions().source_count(), 48);
    assert_eq!(installed48.dimensions().feature_bands(), 24);
    assert_eq!(installed48.dimensions().target_capacity(), 15);
    assert_eq!(installed48.dimensions().axes(), 3);

    let projection_evidence = ProviderFileEvidence::new(Arc::from(OWNER_PROJECTION), None);
    let projection =
        validate_selected_owner_projection(&projection_evidence, &cold_manifest).unwrap();
    assert_eq!(projection.groups().len(), 7);
    assert_eq!(
        projection.groups()[0].commit_owner(),
        SelectedCommitOwner::SpatialControl
    );
    assert_eq!(projection.groups()[5].receipt_owner(), SelectedOwnerId::F13);
    assert_eq!(projection.groups()[6].receipt_owner(), SelectedOwnerId::F15);

    let target_evidence = ProviderFileEvidence::new(Arc::from(TARGET_COLD), None);
    let target =
        validate_f03_target_cold_provider_from_f01b_assets(&target_evidence, &cold_manifest)
            .unwrap();
    assert_eq!(target.current_attribute_u32(), &[0x0000_0b00; 15]);
    let f01e_provider_evidence = ProviderFileEvidence::new(Arc::from(F01E_COLD), None);
    let validated_f01e =
        validate_f01e_provider_from_f01b_cold_assets(&f01e_provider_evidence, &cold_manifest)
            .unwrap();
    assert_eq!(validated_f01e.object_count(), 28);
    let window_evidence = ProviderFileEvidence::new(Arc::from(WINDOW_Q15), None);
    let window =
        validate_window_q15_provider_from_f01b_cold_assets(&window_evidence, &cold_manifest)
            .unwrap();
    assert_eq!(window.receiver_blend32().len(), 32);
    assert_eq!(window.component_kernel64().len(), 64);
}

#[test]
fn roots_and_payloads_cannot_be_substituted_across_the_two_bundles() {
    let provider_manifest_evidence = PackageManifestEvidence::new(Arc::from(F03W_MANIFEST), None);
    let cold_manifest_evidence = PackageManifestEvidence::new(Arc::from(COLD_MANIFEST), None);
    assert!(validate_f01b_cold_asset_manifest(&provider_manifest_evidence).is_err());
    assert!(validate_f03w_provider_manifest(&cold_manifest_evidence).is_err());

    let cold_manifest = validate_f01b_cold_asset_manifest(&cold_manifest_evidence).unwrap();
    let mut changed_schema = INSTALLED48_STATE.to_vec();
    changed_schema[0] ^= 1;
    let changed_schema = ProviderFileEvidence::new(Arc::from(changed_schema), None);
    assert!(matches!(
        validate_installed48_state_schema(&changed_schema, &cold_manifest),
        Err(SelectedColdContractError::Manifest(_))
    ));

    let mut changed_projection = OWNER_PROJECTION.to_vec();
    changed_projection[0] ^= 1;
    let changed_projection = ProviderFileEvidence::new(Arc::from(changed_projection), None);
    assert!(matches!(
        validate_selected_owner_projection(&changed_projection, &cold_manifest),
        Err(SelectedColdContractError::Manifest(_))
    ));
}
