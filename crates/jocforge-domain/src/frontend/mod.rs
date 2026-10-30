// pattern: Functional Core
//! Pure frontend contracts and provider validation.

pub mod adm_file_storage_map;
pub mod adm_object_events;
pub mod bed_channel_join;
mod current_features_provider;
pub mod dbmd_sections;
pub mod job_configuration;
pub mod normal_control_snapshot;
pub mod object_control;
pub mod output_publication_gate;
pub mod primary_metadata_control;
mod provider;
pub mod selected_adm_graph;
mod selected_cold_contracts;
mod selected_provider_manifests;
mod state;
mod target_cold_provider;

pub use current_features_provider::{
    CurrentFeaturesProviderError, CurrentFeaturesProviderLineage,
    FeatureFft4096MathProviderReceipt, FeatureFft4096MathRoute,
    FeatureProcessorMathProviderReceipt, ValidatedCurrentFeaturesPackageManifest,
    ValidatedFeatureFft4096MathProvider, ValidatedFeatureFft4096Stage,
    ValidatedFeatureProcessorMathProvider, validate_current_features_asset_subset_manifest,
    validate_current_features_package_manifest, validate_feature_fft4096_math_provider,
    validate_feature_processor_math_provider,
};
pub use provider::{
    F01EProviderError, F01EProviderReceipt, PackageManifestEvidence, ProviderFileEvidence,
    ProviderIdentity, ProviderSha256, ValidatedF01EColdProvider, ValidatedF01EPackageManifest,
    validate_f01e_package_manifest, validate_f01e_provider,
    validate_f01e_provider_from_f01b_cold_assets,
};
pub use selected_cold_contracts::{
    SelectedColdContractError, SelectedCommitOwner, SelectedInstalled48Dimensions,
    SelectedOwnerExecutor, SelectedOwnerId, SelectedOwnerProjectionGroup,
    SelectedOwnerProjectionGroupId, ValidatedInstalled48StateSchema,
    ValidatedSelectedOwnerProjection, validate_installed48_state_schema,
    validate_selected_owner_projection,
};
pub(crate) use selected_provider_manifests::F03wManifestEntry;
pub use selected_provider_manifests::{
    F03wAssetManifestError, F03wProviderEntryLineage, HashBoundF01bColdSchemaAsset,
    HashBoundF03wProviderAsset, ValidatedF01bColdAssetManifest, ValidatedF03wProviderManifest,
    validate_f01b_cold_asset_manifest, validate_f03w_provider_manifest,
};
pub use state::{
    FrontendControlDigest, FrontendControlError, FrontendControlJoinMetadata,
    FrontendControlMachine, FrontendControlNormalStage, FrontendControlOwnerStateReference,
    FrontendControlPhase, FrontendControlPrepared, FrontendControlReplacementCandidate,
    FrontendControlStaged, FrontendControlState, FrontendNormalizedConfig, FrontendObjectState,
};
pub use target_cold_provider::{
    F03TargetProviderError, F03TargetProviderLineage, F03TargetProviderReceipt,
    ValidatedF03PackageManifest, ValidatedF03TargetColdProvider, validate_f03_package_manifest,
    validate_f03_target_cold_provider, validate_f03_target_cold_provider_from_f01b_assets,
};
