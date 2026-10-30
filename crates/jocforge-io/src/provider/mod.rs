// pattern: Imperative Shell
//! File-reading shell for immutable provider evidence.

use jocforge_domain::frontend::{PackageManifestEvidence, ProviderFileEvidence};
use std::{io, path::Path, sync::Arc};

pub mod current_features;
pub mod metadata_owner;
pub mod meter;
pub mod meter_provider_roster;
pub mod selected_spatial_cold_inputs;
pub mod spatial_audio;

pub use current_features::{
    SelectedCurrentFeaturesMathProviderLoadError, SelectedCurrentFeaturesMathProviders,
    load_selected_current_features_math_providers,
};
pub use jocforge_domain::frontend::F03wProviderEntryLineage;
pub use metadata_owner::{
    MetadataOwnerProviderBundle, MetadataOwnerProviderLoadError,
    load_metadata_owner_provider_bundle,
};
pub use meter::{MeterDirectProviderLoadError, load_meter_direct_provider_inventory};
pub use meter_provider_roster::{
    MeterProviderRosterLoadError, load_selected_meter_provider_roster,
};
pub use selected_spatial_cold_inputs::{
    SelectedProviderRoot, SelectedSpatialColdProviderCallbackValues,
    SelectedSpatialColdProviderCandidate, SelectedSpatialColdProviderLineage,
    SelectedSpatialColdProviderLoadError, SelectedSpatialColdProviderRoots,
    load_selected_spatial_cold_provider_candidate,
    load_selected_spatial_cold_provider_candidate_for_selected_profile,
};
pub use spatial_audio::{
    SelectedSpatialAudioProviderBundle, SelectedSpatialAudioProviderLoadError,
    load_selected_spatial_audio_provider_bundle,
};

pub fn read_provider_file(path: impl AsRef<Path>) -> io::Result<ProviderFileEvidence> {
    let path = path.as_ref();
    let bytes: Arc<[u8]> = std::fs::read(path)?.into();
    Ok(ProviderFileEvidence::new(
        bytes,
        Some(path.display().to_string()),
    ))
}

pub fn read_package_manifest_file(path: impl AsRef<Path>) -> io::Result<PackageManifestEvidence> {
    let path = path.as_ref();
    let bytes: Arc<[u8]> = std::fs::read(path)?.into();
    Ok(PackageManifestEvidence::new(
        bytes,
        Some(path.display().to_string()),
    ))
}
