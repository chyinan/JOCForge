// pattern: Imperative Shell
//! Loads the Wash-approved MetadataOwner provider and wrapper package from a caller-selected root.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

const PACKAGE_MANIFEST: &str = "MANIFEST.json";
const PACKAGE_IDENTITY: &str = "metadata-owner-root.selected-48k-1536-four-queue.v2";
const PACKAGE_MANIFEST_SHA256: &str =
    "94774c412ce8457f3220d84e4b3386c0b48c21837d4a4943570791bb66f034b3";
const PROVIDER_FILE: &str = "metadata-public-provider.json";
const WRAPPER_FILE: &str = "metadata-wrapper-config.json";

const PACKAGE_FILES: [PackageFileSpec; 11] = [
    PackageFileSpec {
        relative_path: "METADATA-OWNER-CONTRACT.md",
        bytes: 8_767,
        sha256: "6b5584a57c0fc22b0f3b7a9f4d101f5a92404468c96ad7d0270b52f46dc467c2",
    },
    PackageFileSpec {
        relative_path: "METADATA-OWNER-MACHINE-MAP.json",
        bytes: 2_221,
        sha256: "499e155c55cd770f0184477a02ced516d07f1b3cc0010ba5ac5aba431b1b7e72",
    },
    PackageFileSpec {
        relative_path: PROVIDER_FILE,
        bytes: 1_399,
        sha256: "6d94c55c8c011fd4762e60ec92ed491f19b17540c72ba82094a36f6e666f2ef7",
    },
    PackageFileSpec {
        relative_path: WRAPPER_FILE,
        bytes: 1_754,
        sha256: "722e93ad671e4faab6ae9f33125bd49f8f369f717db0c9a3600cd861cdbd5a56",
    },
    PackageFileSpec {
        relative_path: "METADATA-SOURCE-RECEIPT.schema.json",
        bytes: 1_940,
        sha256: "3ce5bf0eda0f32bf387241ba3255160791f6ecb3cc40de7781cff375123f7510",
    },
    PackageFileSpec {
        relative_path: "METADATA-OWNER-VECTORS.json",
        bytes: 227_297,
        sha256: "4271ce47e911ba107c1f76931cf872d8240e7152381d0f12bd5b17bf3e2eb023",
    },
    PackageFileSpec {
        relative_path: "REPLAY-CHECK.json",
        bytes: 1_940,
        sha256: "14eb467adc2d7c61db8169698610f149735dd6c1767d6273f302891d97a92c91",
    },
    PackageFileSpec {
        relative_path: "EMDF-TERMINAL-FIXTURES.json",
        bytes: 4_953,
        sha256: "18081b8176e969ff37fe6d6134d843b4a8057e152ac6ea6ddd57da435ebea924",
    },
    PackageFileSpec {
        relative_path: "EMDF-TERMINAL-INDEPENDENT-CHECK.json",
        bytes: 455,
        sha256: "f8c45a1100ec6b53cbe120eb0a06291148774c2664e0f6331d4feb40575532f0",
    },
    PackageFileSpec {
        relative_path: "TIMER-DIGEST-INDEPENDENT-CHECK.json",
        bytes: 1_370,
        sha256: "187c8678158b3f4e10dac14a237fca2ef2638b99d7a65fded049cf9eda99364c",
    },
    PackageFileSpec {
        relative_path: "OCCUPIED-DESTINATION-INDEPENDENT-CHECK.json",
        bytes: 2_325,
        sha256: "596175ddc8db659eef8e73bb3ed58d3551e72762c11fbb3588071b2586380dda",
    },
];

#[derive(Clone, Copy)]
struct PackageFileSpec {
    relative_path: &'static str,
    bytes: usize,
    sha256: &'static str,
}

/// Validated provider bytes passed to the pure MetadataOwner constructor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataOwnerProviderBundle {
    manifest_sha256: &'static str,
    provider_file: Vec<u8>,
    wrapper_file: Vec<u8>,
}

impl MetadataOwnerProviderBundle {
    pub const fn manifest_sha256(&self) -> &'static str {
        self.manifest_sha256
    }

    pub fn provider_file(&self) -> &[u8] {
        &self.provider_file
    }

    pub fn wrapper_file(&self) -> &[u8] {
        &self.wrapper_file
    }
}

/// Errors while loading the pinned MetadataOwner package.
#[derive(Debug)]
pub enum MetadataOwnerProviderLoadError {
    PackageRoot(io::Error),
    PackagePathEscapesRoot {
        relative_path: &'static str,
    },
    FileRead {
        relative_path: &'static str,
        source: io::Error,
    },
    ManifestIdentity,
    ManifestShape,
    MemberIdentity {
        relative_path: &'static str,
    },
}

impl fmt::Display for MetadataOwnerProviderLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PackageRoot(error) => write!(formatter, "MetadataOwner package root: {error}"),
            Self::PackagePathEscapesRoot { relative_path } => {
                write!(
                    formatter,
                    "MetadataOwner package path escapes root: {relative_path}"
                )
            }
            Self::FileRead {
                relative_path,
                source,
            } => write!(
                formatter,
                "MetadataOwner package file {relative_path}: {source}"
            ),
            Self::ManifestIdentity => {
                formatter.write_str("MetadataOwner package manifest identity mismatch")
            }
            Self::ManifestShape => {
                formatter.write_str("MetadataOwner package manifest shape mismatch")
            }
            Self::MemberIdentity { relative_path } => {
                write!(
                    formatter,
                    "MetadataOwner package member identity mismatch: {relative_path}"
                )
            }
        }
    }
}

impl std::error::Error for MetadataOwnerProviderLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PackageRoot(error) | Self::FileRead { source: error, .. } => Some(error),
            Self::PackagePathEscapesRoot { .. }
            | Self::ManifestIdentity
            | Self::ManifestShape
            | Self::MemberIdentity { .. } => None,
        }
    }
}

/// Reads and verifies the complete Wash-approved MetadataOwner package, then returns its two
/// runtime provider payloads. No provider is installed and no owner state is created here.
pub fn load_metadata_owner_provider_bundle(
    package_root: impl AsRef<Path>,
) -> Result<MetadataOwnerProviderBundle, MetadataOwnerProviderLoadError> {
    let package_root = package_root
        .as_ref()
        .canonicalize()
        .map_err(MetadataOwnerProviderLoadError::PackageRoot)?;
    let manifest_path = package_file(&package_root, PACKAGE_MANIFEST)?;
    let manifest_bytes = read_package_file(&manifest_path, PACKAGE_MANIFEST)?;
    if sha256_hex(&manifest_bytes) != PACKAGE_MANIFEST_SHA256 {
        return Err(MetadataOwnerProviderLoadError::ManifestIdentity);
    }

    let manifest: Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|_| MetadataOwnerProviderLoadError::ManifestShape)?;
    let files = manifest["files"]
        .as_array()
        .filter(|files| files.len() == PACKAGE_FILES.len())
        .ok_or(MetadataOwnerProviderLoadError::ManifestShape)?;
    if manifest["schema"] != "washed-implementer-supplement-manifest.v1"
        || manifest["identity"] != PACKAGE_IDENTITY
        || manifest["status"] != "SOURCE_RECEIPT_AND_EOF_PROPOSAL_READY_CANDIDATE"
        || manifest["delivery_scope"] != "internal implementation only"
    {
        return Err(MetadataOwnerProviderLoadError::ManifestShape);
    }

    let mut provider_file = None;
    let mut wrapper_file = None;
    for spec in PACKAGE_FILES {
        let entries: Vec<_> = files
            .iter()
            .filter(|entry| entry["file"] == spec.relative_path)
            .collect();
        if entries.len() != 1
            || entries[0]["bytes"].as_u64() != u64::try_from(spec.bytes).ok()
            || entries[0]["sha256"].as_str() != Some(spec.sha256)
        {
            return Err(MetadataOwnerProviderLoadError::MemberIdentity {
                relative_path: spec.relative_path,
            });
        }

        let path = package_file(&package_root, spec.relative_path)?;
        let payload = read_package_file(&path, spec.relative_path)?;
        if payload.len() != spec.bytes || sha256_hex(&payload) != spec.sha256 {
            return Err(MetadataOwnerProviderLoadError::MemberIdentity {
                relative_path: spec.relative_path,
            });
        }
        match spec.relative_path {
            PROVIDER_FILE => provider_file = Some(payload),
            WRAPPER_FILE => wrapper_file = Some(payload),
            _ => {}
        }
    }

    Ok(MetadataOwnerProviderBundle {
        manifest_sha256: PACKAGE_MANIFEST_SHA256,
        provider_file: provider_file.ok_or(MetadataOwnerProviderLoadError::ManifestShape)?,
        wrapper_file: wrapper_file.ok_or(MetadataOwnerProviderLoadError::ManifestShape)?,
    })
}

fn package_file(
    package_root: &Path,
    relative_path: &'static str,
) -> Result<PathBuf, MetadataOwnerProviderLoadError> {
    let resolved = package_root
        .join(relative_path)
        .canonicalize()
        .map_err(|source| MetadataOwnerProviderLoadError::FileRead {
            relative_path,
            source,
        })?;
    if !resolved.starts_with(package_root) {
        return Err(MetadataOwnerProviderLoadError::PackagePathEscapesRoot { relative_path });
    }
    Ok(resolved)
}

fn read_package_file(
    path: &Path,
    relative_path: &'static str,
) -> Result<Vec<u8>, MetadataOwnerProviderLoadError> {
    fs::read(path).map_err(|source| MetadataOwnerProviderLoadError::FileRead {
        relative_path,
        source,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("formatting into String is infallible");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{MetadataOwnerProviderLoadError, load_metadata_owner_provider_bundle};
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    const MANIFEST_DIRECTORY: &str = env!("CARGO_MANIFEST_DIR");

    fn approved_package_root() -> PathBuf {
        Path::new(MANIFEST_DIRECTORY)
            .join("../../outputs/IMPLEMENTER_SUPPLEMENTS/METADATA-OWNER-ROOT-V1")
    }

    struct TemporaryPackage(PathBuf);

    impl TemporaryPackage {
        fn copy_approved_package() -> Self {
            let source = approved_package_root();
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time follows Unix epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "jocforge-metadata-owner-provider-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir(&root).expect("temporary package root is unique");
            let manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(source.join("MANIFEST.json")).unwrap()).unwrap();
            fs::copy(source.join("MANIFEST.json"), root.join("MANIFEST.json")).unwrap();
            for entry in manifest["files"].as_array().unwrap() {
                let relative = entry["file"].as_str().unwrap();
                let destination = root.join(relative);
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                fs::copy(source.join(relative), destination).unwrap();
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
    fn loads_the_full_approved_package_and_returns_only_runtime_provider_payloads() {
        let bundle = load_metadata_owner_provider_bundle(approved_package_root()).unwrap();
        assert_eq!(
            bundle.manifest_sha256(),
            "94774c412ce8457f3220d84e4b3386c0b48c21837d4a4943570791bb66f034b3"
        );
        assert_eq!(bundle.provider_file().len(), 1_399);
        assert_eq!(bundle.wrapper_file().len(), 1_754);
    }

    #[test]
    fn rejects_a_changed_provider_member_without_returning_a_bundle() {
        let package = TemporaryPackage::copy_approved_package();
        let provider_path = package.root().join("metadata-public-provider.json");
        let mut provider = fs::read(&provider_path).unwrap();
        provider[0] ^= 1;
        fs::write(provider_path, provider).unwrap();
        assert!(matches!(
            load_metadata_owner_provider_bundle(package.root()),
            Err(MetadataOwnerProviderLoadError::MemberIdentity {
                relative_path: "metadata-public-provider.json"
            })
        ));
    }
}
