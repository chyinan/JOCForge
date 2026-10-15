// pattern: Functional Core
//! Pure frontend contracts and provider validation.

mod provider;
mod state;

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
