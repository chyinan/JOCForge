// pattern: Functional Core
//! Pure frontend contracts and provider validation.

mod current_features_provider;
mod provider;
mod state;
mod target_cold_provider;

pub use current_features_provider::{
    CurrentFeaturesProviderError, CurrentFeaturesProviderLineage,
    FeatureFft4096MathProviderReceipt, FeatureFft4096MathRoute,
    FeatureProcessorMathProviderReceipt, ValidatedCurrentFeaturesPackageManifest,
    ValidatedFeatureFft4096MathProvider, ValidatedFeatureFft4096Stage,
    ValidatedFeatureProcessorMathProvider, validate_current_features_package_manifest,
    validate_feature_fft4096_math_provider, validate_feature_processor_math_provider,
};
pub use provider::{
    F01EProviderError, F01EProviderReceipt, PackageManifestEvidence, ProviderFileEvidence,
    ProviderIdentity, ProviderSha256, ValidatedF01EColdProvider, ValidatedF01EPackageManifest,
    validate_f01e_package_manifest, validate_f01e_provider,
};
pub use state::{
    FrontendControlDigest, FrontendControlError, FrontendControlJoinMetadata,
    FrontendControlMachine, FrontendControlPhase, FrontendControlPrepared, FrontendControlStaged,
    FrontendControlState, FrontendNormalizedConfig, FrontendObjectState,
};
pub use target_cold_provider::{
    F03TargetProviderError, F03TargetProviderLineage, F03TargetProviderReceipt,
    ValidatedF03PackageManifest, ValidatedF03TargetColdProvider, validate_f03_package_manifest,
    validate_f03_target_cold_provider,
};
