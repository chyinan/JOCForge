// pattern: Imperative Shell
//! File-reading shell for immutable provider evidence.

use jocforge_domain::frontend::{PackageManifestEvidence, ProviderFileEvidence};
use std::{io, path::Path, sync::Arc};

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
