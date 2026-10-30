// pattern: Imperative Shell
//! Loads the approved non-exhaustive direct Meter provider inventory from a caller package root.

use jocforge_domain::meter::direct_provider_inventory::{
    MeterDirectProviderInventory, MeterDirectProviderInventoryError, MeterDirectProviderPayloads,
    validate_meter_direct_provider_inventory,
};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

const PACKAGE_MANIFEST: &str = "MANIFEST.json";
const TRUE_PEAK: &str = "parameters/meter-truepeak-authorized-v1.json";
const OAR_GEOMETRY: &str = "parameters/meter-oar-secondary8-math-v1.json";
const LOG_CONSTANTS: &str = "parameters/meter-log-mathematical-v1.json";
const K_STATE_BOUNDS: &str = "parameters/meter-k-state-bounds-v1.json";
const K_MATRIX: &str = "parameters/meter-k-matrix-math-v1.json";
const SHARED_DV32: &str = "parameters/shared-dv32-mathematical-v1.json";
const INSTALLED48_DEPENDENCY_INDEX: &str = "parameters/adm-installed48-providers-v1.json";

/// Errors while reading and validating the approved direct Meter asset subset.
#[derive(Debug)]
pub enum MeterDirectProviderLoadError {
    PackageRoot(io::Error),
    PackagePathEscapesRoot {
        relative_path: &'static str,
    },
    FileRead {
        relative_path: &'static str,
        source: io::Error,
    },
    Inventory(MeterDirectProviderInventoryError),
}

impl fmt::Display for MeterDirectProviderLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PackageRoot(error) => write!(formatter, "Meter provider package root: {error}"),
            Self::PackagePathEscapesRoot { relative_path } => {
                write!(
                    formatter,
                    "Meter provider package path escapes root: {relative_path}"
                )
            }
            Self::FileRead {
                relative_path,
                source,
            } => write!(formatter, "Meter provider file {relative_path}: {source}"),
            Self::Inventory(error) => {
                write!(formatter, "Meter direct inventory rejected: {error:?}")
            }
        }
    }
}

impl std::error::Error for MeterDirectProviderLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PackageRoot(error) | Self::FileRead { source: error, .. } => Some(error),
            Self::PackagePathEscapesRoot { .. } | Self::Inventory(_) => None,
        }
    }
}

/// Reads exactly the seven inventoried assets and returns their typed direct subset.
///
/// The `adm-installed48` entry is preserved as an identity receipt only; this adapter does not
/// treat that upstream dependency index as a loaded numerical provider. The returned inventory
/// is intentionally not the complete transitive Meter provider set.
pub fn load_meter_direct_provider_inventory(
    package_root: impl AsRef<Path>,
) -> Result<MeterDirectProviderInventory, MeterDirectProviderLoadError> {
    let package_root = package_root.as_ref();
    let canonical_root = package_root
        .canonicalize()
        .map_err(MeterDirectProviderLoadError::PackageRoot)?;
    let manifest_path = package_file(&canonical_root, PACKAGE_MANIFEST)?;
    let manifest = read_package_file(&manifest_path, PACKAGE_MANIFEST)?;
    let true_peak = read_provider_file(&canonical_root, TRUE_PEAK)?;
    let oar_geometry = read_provider_file(&canonical_root, OAR_GEOMETRY)?;
    let log_constants = read_provider_file(&canonical_root, LOG_CONSTANTS)?;
    let k_state_bounds = read_provider_file(&canonical_root, K_STATE_BOUNDS)?;
    let k_matrix = read_provider_file(&canonical_root, K_MATRIX)?;
    let shared_dv32 = read_provider_file(&canonical_root, SHARED_DV32)?;
    let installed48_dependency_index =
        read_provider_file(&canonical_root, INSTALLED48_DEPENDENCY_INDEX)?;

    validate_meter_direct_provider_inventory(
        &manifest,
        MeterDirectProviderPayloads::new(
            &true_peak,
            &oar_geometry,
            &log_constants,
            &k_state_bounds,
            &k_matrix,
            &shared_dv32,
            &installed48_dependency_index,
        ),
    )
    .map_err(MeterDirectProviderLoadError::Inventory)
}

fn read_provider_file(
    package_root: &Path,
    relative_path: &'static str,
) -> Result<Vec<u8>, MeterDirectProviderLoadError> {
    let path = package_file(package_root, relative_path)?;
    read_package_file(&path, relative_path)
}

fn package_file(
    package_root: &Path,
    relative_path: &'static str,
) -> Result<PathBuf, MeterDirectProviderLoadError> {
    let resolved = package_root
        .join(relative_path)
        .canonicalize()
        .map_err(|source| MeterDirectProviderLoadError::FileRead {
            relative_path,
            source,
        })?;
    if !resolved.starts_with(package_root) {
        return Err(MeterDirectProviderLoadError::PackagePathEscapesRoot { relative_path });
    }
    Ok(resolved)
}

fn read_package_file(
    path: &Path,
    relative_path: &'static str,
) -> Result<Vec<u8>, MeterDirectProviderLoadError> {
    fs::read(path).map_err(|source| MeterDirectProviderLoadError::FileRead {
        relative_path,
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::{MeterDirectProviderLoadError, load_meter_direct_provider_inventory};
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    const ASSETS_ROOT: &str = env!("CARGO_MANIFEST_DIR");
    const PACKAGE_FILES: [(&str, &str); 8] = [
        ("MANIFEST.json", "MANIFEST.json"),
        (
            "parameters/meter-truepeak-authorized-v1.json",
            "parameters/meter-truepeak-authorized-v1.json",
        ),
        (
            "parameters/meter-oar-secondary8-math-v1.json",
            "parameters/meter-oar-secondary8-math-v1.json",
        ),
        (
            "parameters/meter-log-mathematical-v1.json",
            "parameters/meter-log-mathematical-v1.json",
        ),
        (
            "parameters/meter-k-state-bounds-v1.json",
            "parameters/meter-k-state-bounds-v1.json",
        ),
        (
            "parameters/meter-k-matrix-math-v1.json",
            "parameters/meter-k-matrix-math-v1.json",
        ),
        (
            "parameters/shared-dv32-mathematical-v1.json",
            "parameters/shared-dv32-mathematical-v1.json",
        ),
        (
            "parameters/adm-installed48-providers-v1.json",
            "parameters/adm-installed48-providers-v1.json",
        ),
    ];

    struct TemporaryPackage(PathBuf);

    impl TemporaryPackage {
        fn from_approved_assets() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("the system clock is after the Unix epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "jocforge-meter-direct-provider-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir(&root).expect("unique temporary package root is created");
            for (relative_path, source_path) in PACKAGE_FILES {
                let destination = root.join(relative_path);
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent).expect("parameter parent directory is created");
                }
                fs::copy(
                    Path::new(ASSETS_ROOT)
                        .join("../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-APPROVED-PARAMETER-ASSETS-V1")
                        .join(source_path),
                    destination,
                )
                    .expect("approved package asset is copied for loader testing");
            }
            Self(root)
        }

        fn root(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TemporaryPackage {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn loads_the_actual_approved_direct_assets_from_a_caller_package_root() {
        let package = TemporaryPackage::from_approved_assets();
        let inventory = load_meter_direct_provider_inventory(package.root())
            .expect("the approved direct subset loads and validates");
        assert_eq!(inventory.direct_assets().len(), 7);
        assert_eq!(inventory.k_matrix().complete_words().len(), 72);
        assert_eq!(
            inventory.shared_dv32_parameters().coefficient_words().len(),
            5
        );
        assert_eq!(inventory.true_peak_parameters().stage_b_words().len(), 12);
    }

    #[test]
    fn changed_payload_is_rejected_by_its_direct_asset_identity() {
        let package = TemporaryPackage::from_approved_assets();
        let changed_path = package
            .root()
            .join("parameters/meter-k-matrix-math-v1.json");
        let mut bytes = fs::read(&changed_path).unwrap();
        bytes[0] ^= 1;
        fs::write(changed_path, bytes).unwrap();
        assert!(matches!(
            load_meter_direct_provider_inventory(package.root()),
            Err(MeterDirectProviderLoadError::Inventory(_))
        ));
    }
}
