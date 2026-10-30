// pattern: Imperative Shell
//! Loads the hash-bound selected Meter provider member roster for its parent receipt.

use std::{
    error::Error,
    fmt, fs,
    path::{Component, Path},
};

use jocforge_domain::meter::provider_set_identity::{
    ExpectedMeterProviderMember, MeterProviderSetIdentityError, SelectedMeterProviderRoster,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const PACKAGE_ID: &str = "METER-PROVIDER-SET-IDENTITY-APPLICATION-V1";
const PACKAGE_MANIFEST_SHA256: &str =
    "cf402d604138bed23a3328fc29ce3b89ddcfdc5fe5d270431d40f806627f04a7";
const SOURCE_CLOSURE_MANIFEST_SHA256: &str =
    "67852ad4cc89c757f97bd9102b65415e9187b8ad20bd7dcbe38691721f1251f0";

#[derive(Debug)]
pub enum MeterProviderRosterLoadError {
    PackageRoot(std::io::Error),
    FileRead {
        path: String,
        source: std::io::Error,
    },
    PackageManifestIdentity,
    PackageManifestSyntax,
    PackageManifestShape,
    PackageManifestSidecar,
    PayloadIdentity(String),
    RosterShape,
    RosterCount,
    SourceClosureIdentity,
    MemberSourceHash,
    Domain(MeterProviderSetIdentityError),
}

impl fmt::Display for MeterProviderRosterLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PackageRoot(error) => write!(formatter, "Meter roster package root: {error}"),
            Self::FileRead { path, source } => {
                write!(formatter, "Meter roster package file {path}: {source}")
            }
            Self::PackageManifestIdentity => {
                formatter.write_str("Meter roster manifest identity mismatch")
            }
            Self::PackageManifestSyntax => {
                formatter.write_str("Meter roster manifest syntax invalid")
            }
            Self::PackageManifestShape => {
                formatter.write_str("Meter roster manifest shape invalid")
            }
            Self::PackageManifestSidecar => {
                formatter.write_str("Meter roster manifest sidecar mismatch")
            }
            Self::PayloadIdentity(path) => {
                write!(formatter, "Meter roster payload identity mismatch: {path}")
            }
            Self::RosterShape => {
                formatter.write_str("selected Meter provider roster shape invalid")
            }
            Self::RosterCount => {
                formatter.write_str("selected Meter provider roster count mismatch")
            }
            Self::SourceClosureIdentity => {
                formatter.write_str("selected Meter roster source closure identity mismatch")
            }
            Self::MemberSourceHash => {
                formatter.write_str("selected Meter roster member source hash invalid")
            }
            Self::Domain(error) => write!(formatter, "selected Meter provider roster: {error:?}"),
        }
    }
}

impl Error for MeterProviderRosterLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::PackageRoot(error) | Self::FileRead { source: error, .. } => Some(error),
            _ => None,
        }
    }
}

/// Loads the exact Wash roster. This is expected membership only; it does not create any member
/// validation receipt or claim that a provider set is complete at runtime.
pub fn load_selected_meter_provider_roster(
    package_root: impl AsRef<Path>,
) -> Result<SelectedMeterProviderRoster, MeterProviderRosterLoadError> {
    let root = fs::canonicalize(package_root).map_err(MeterProviderRosterLoadError::PackageRoot)?;
    let manifest_bytes = read_member(&root, "MANIFEST.json")?;
    let expected_manifest_sha256 = decode_sha256(PACKAGE_MANIFEST_SHA256)
        .ok_or(MeterProviderRosterLoadError::PackageManifestIdentity)?;
    let expected_source_closure_sha256 = decode_sha256(SOURCE_CLOSURE_MANIFEST_SHA256)
        .ok_or(MeterProviderRosterLoadError::SourceClosureIdentity)?;
    if sha256(&manifest_bytes) != expected_manifest_sha256 {
        return Err(MeterProviderRosterLoadError::PackageManifestIdentity);
    }
    let manifest: Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|_| MeterProviderRosterLoadError::PackageManifestSyntax)?;
    if manifest["package_id"] != PACKAGE_ID
        || manifest["schema"] != "washed-implementer-supplement.v1"
    {
        return Err(MeterProviderRosterLoadError::PackageManifestShape);
    }

    let sidecar = read_member(&root, "MANIFEST.sha256")?;
    let sidecar = std::str::from_utf8(&sidecar)
        .map_err(|_| MeterProviderRosterLoadError::PackageManifestSidecar)?
        .split_whitespace()
        .collect::<Vec<_>>();
    if sidecar != [PACKAGE_MANIFEST_SHA256, "MANIFEST.json"] {
        return Err(MeterProviderRosterLoadError::PackageManifestSidecar);
    }

    let files = manifest["files"]
        .as_array()
        .filter(|files| files.len() == 3)
        .ok_or(MeterProviderRosterLoadError::PackageManifestShape)?;
    for file in files {
        let path = file["path"]
            .as_str()
            .ok_or(MeterProviderRosterLoadError::PackageManifestShape)?;
        let bytes = file["bytes"]
            .as_u64()
            .ok_or(MeterProviderRosterLoadError::PackageManifestShape)?;
        let expected = file["sha256"]
            .as_str()
            .ok_or(MeterProviderRosterLoadError::PackageManifestShape)?;
        let payload = read_member(&root, path)?;
        if payload.len() as u64 != bytes || sha256_hex(&payload) != expected {
            return Err(MeterProviderRosterLoadError::PayloadIdentity(
                path.to_owned(),
            ));
        }
    }

    let roster_bytes = read_member(&root, "EXPECTED-MEMBER-ROSTER.json")?;
    let roster: Value = serde_json::from_slice(&roster_bytes)
        .map_err(|_| MeterProviderRosterLoadError::RosterShape)?;
    if roster["schema"] != "expected-selected-meter-provider-roster.v1"
        || roster["source_closure_manifest_sha256"] != SOURCE_CLOSURE_MANIFEST_SHA256
    {
        return Err(MeterProviderRosterLoadError::SourceClosureIdentity);
    }
    let d11 = roster["selected_d11_prepass_rows"]
        .as_array()
        .filter(|rows| rows.len() == 11)
        .ok_or(MeterProviderRosterLoadError::RosterCount)?;
    let families = roster["meter_numeric_and_rule_families"]
        .as_array()
        .filter(|rows| rows.len() == 8)
        .ok_or(MeterProviderRosterLoadError::RosterCount)?;

    let mut members = Vec::with_capacity(19);
    for row in d11 {
        let member_id = row["id"]
            .as_str()
            .ok_or(MeterProviderRosterLoadError::RosterShape)?;
        let source_hash = row["provider_sha256"]
            .as_str()
            .and_then(decode_sha256)
            .ok_or(MeterProviderRosterLoadError::MemberSourceHash)?;
        members.push(ExpectedMeterProviderMember::new(member_id, source_hash));
    }
    for row in families {
        let member_id = row["id"]
            .as_str()
            .ok_or(MeterProviderRosterLoadError::RosterShape)?;
        let source_hash = row["manifest_sha256"]
            .as_str()
            .and_then(decode_sha256)
            .ok_or(MeterProviderRosterLoadError::MemberSourceHash)?;
        members.push(ExpectedMeterProviderMember::new(member_id, source_hash));
    }

    SelectedMeterProviderRoster::new(
        expected_manifest_sha256,
        expected_source_closure_sha256,
        members,
    )
    .map_err(MeterProviderRosterLoadError::Domain)
}

fn read_member(root: &Path, relative: &str) -> Result<Vec<u8>, MeterProviderRosterLoadError> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(MeterProviderRosterLoadError::FileRead {
            path: relative.to_owned(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid package member path",
            ),
        });
    }
    let path = root.join(relative_path);
    let canonical =
        fs::canonicalize(&path).map_err(|source| MeterProviderRosterLoadError::FileRead {
            path: relative.to_owned(),
            source,
        })?;
    if !canonical.starts_with(root) {
        return Err(MeterProviderRosterLoadError::FileRead {
            path: relative.to_owned(),
            source: std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "package member escaped root",
            ),
        });
    }
    fs::read(canonical).map_err(|source| MeterProviderRosterLoadError::FileRead {
        path: relative.to_owned(),
        source,
    })
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = sha256(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
    }
    hex
}

fn decode_sha256(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let mut result = [0; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let high = decode_nibble(pair[0])?;
        let low = decode_nibble(pair[1])?;
        result[index] = (high << 4) | low;
    }
    Some(result)
}

fn decode_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::load_selected_meter_provider_roster;
    use std::path::PathBuf;

    #[test]
    fn loads_the_exact_selected_meter_member_roster_from_approved_package() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../outputs/IMPLEMENTER_SUPPLEMENTS/METER-PROVIDER-SET-IDENTITY-APPLICATION-V1",
        );
        let roster = load_selected_meter_provider_roster(root)
            .expect("the approved selected-provider roster is hash-bound and well-shaped");
        assert_eq!(roster.members().len(), 19);
        assert_eq!(roster.members()[0].member_id(), "D11-FFTS");
        assert_eq!(roster.members()[10].member_id(), "D11-DP");
        assert_eq!(roster.members()[11].member_id(), "D12-R");
        assert_eq!(
            roster.members()[18].member_id(),
            "F06W N-dependent callback, block and window schedule; runtime N/output spans are dynamic"
        );
    }
}
