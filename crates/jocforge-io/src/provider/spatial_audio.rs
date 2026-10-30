// pattern: Imperative Shell
//! Loads the caller-selected F04 provider package and validates its approved bytes.

pub use jocforge_domain::spatial::production_callback::selected_audio_provider_bundle::SelectedSpatialAudioProviderBundle;
use jocforge_domain::{
    frontend::{PackageManifestEvidence, ProviderFileEvidence},
    spatial::{
        F04ParameterProviderError,
        production_callback::selected_audio_provider_bundle::SelectedSpatialAudioProviderEvidence,
        validate_f04_parameter_package_manifest,
    },
};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

const PACKAGE_MANIFEST: &str = "MANIFEST.json";
const COMPONENT_PARAMETERS: &str = "parameters/component-parameter-mathematical-v1.json";
const WINDOW_Q15: &str = "parameters/window-q15.json";
const QMF_RECEIVER_FUNCTIONAL: &str = "parameters/qmf-receiver-functional-v1.json";
const QMF_MATHEMATICAL_FFT: &str = "parameters/qmf-mathematical-fft-v1.json";
const DESCRIPTOR_MATHEMATICAL: &str = "parameters/descriptor-mathematical.json";

/// Errors while reading and validating one caller-supplied F04 provider package.
#[derive(Debug)]
pub enum SelectedSpatialAudioProviderLoadError {
    PackageRoot(io::Error),
    PackagePathEscapesRoot {
        relative_path: &'static str,
    },
    FileRead {
        relative_path: &'static str,
        source: io::Error,
    },
    Manifest(F04ParameterProviderError),
    ProviderBundle(F04ParameterProviderError),
}

impl fmt::Display for SelectedSpatialAudioProviderLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PackageRoot(error) => write!(formatter, "provider package root: {error}"),
            Self::PackagePathEscapesRoot { relative_path } => {
                write!(
                    formatter,
                    "provider package path escapes root: {relative_path}"
                )
            }
            Self::FileRead {
                relative_path,
                source,
            } => write!(formatter, "provider package file {relative_path}: {source}"),
            Self::Manifest(error) => write!(formatter, "F04 package manifest rejected: {error:?}"),
            Self::ProviderBundle(error) => {
                write!(formatter, "F04 provider bundle rejected: {error:?}")
            }
        }
    }
}

impl std::error::Error for SelectedSpatialAudioProviderLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PackageRoot(error) | Self::FileRead { source: error, .. } => Some(error),
            Self::PackagePathEscapesRoot { .. } | Self::Manifest(_) | Self::ProviderBundle(_) => {
                None
            }
        }
    }
}

/// Loads the exact approved five-file F04 provider set from a caller-supplied package root.
///
/// The manifest is read and identity-validated before any provider file is read. Relative
/// provider paths are fixed by the approved selected package; no alternate filename is accepted.
/// The validated domain bundle keeps the QMF provider's internal-only distribution
/// boundary and grants no DSP, state, or commit authority.
pub fn load_selected_spatial_audio_provider_bundle(
    package_root: impl AsRef<Path>,
) -> Result<SelectedSpatialAudioProviderBundle, SelectedSpatialAudioProviderLoadError> {
    let package_root = package_root.as_ref();
    let canonical_root = package_root
        .canonicalize()
        .map_err(SelectedSpatialAudioProviderLoadError::PackageRoot)?;

    let manifest_path = package_file(&canonical_root, PACKAGE_MANIFEST)?;
    let manifest_bytes = read_package_file(&manifest_path, PACKAGE_MANIFEST)?;
    let package_manifest =
        PackageManifestEvidence::new(manifest_bytes, Some(PACKAGE_MANIFEST.to_owned()));

    // Fail the exact package identity before reading any individual provider bytes.
    validate_f04_parameter_package_manifest(&package_manifest)
        .map_err(SelectedSpatialAudioProviderLoadError::Manifest)?;

    let component_parameters = read_provider_evidence(&canonical_root, COMPONENT_PARAMETERS)?;
    let window_q15 = read_provider_evidence(&canonical_root, WINDOW_Q15)?;
    let qmf_receiver_functional = read_provider_evidence(&canonical_root, QMF_RECEIVER_FUNCTIONAL)?;
    let qmf_mathematical_fft = read_provider_evidence(&canonical_root, QMF_MATHEMATICAL_FFT)?;
    let descriptor_mathematical = read_provider_evidence(&canonical_root, DESCRIPTOR_MATHEMATICAL)?;
    let evidence = SelectedSpatialAudioProviderEvidence::new(
        &package_manifest,
        &component_parameters,
        &window_q15,
        &qmf_receiver_functional,
        &qmf_mathematical_fft,
        &descriptor_mathematical,
    );

    SelectedSpatialAudioProviderBundle::validate_selected(&evidence)
        .map_err(SelectedSpatialAudioProviderLoadError::ProviderBundle)
}

fn read_provider_evidence(
    package_root: &Path,
    relative_path: &'static str,
) -> Result<ProviderFileEvidence, SelectedSpatialAudioProviderLoadError> {
    let path = package_file(package_root, relative_path)?;
    let bytes = read_package_file(&path, relative_path)?;
    Ok(ProviderFileEvidence::new(
        bytes,
        Some(path.display().to_string()),
    ))
}

fn package_file(
    package_root: &Path,
    relative_path: &'static str,
) -> Result<PathBuf, SelectedSpatialAudioProviderLoadError> {
    let candidate = package_root.join(relative_path);
    let resolved = candidate.canonicalize().map_err(|source| {
        SelectedSpatialAudioProviderLoadError::FileRead {
            relative_path,
            source,
        }
    })?;
    if !resolved.starts_with(package_root) {
        return Err(
            SelectedSpatialAudioProviderLoadError::PackagePathEscapesRoot { relative_path },
        );
    }
    Ok(resolved)
}

fn read_package_file(
    path: &Path,
    relative_path: &'static str,
) -> Result<Arc<[u8]>, SelectedSpatialAudioProviderLoadError> {
    fs::read(path).map(Arc::<[u8]>::from).map_err(|source| {
        SelectedSpatialAudioProviderLoadError::FileRead {
            relative_path,
            source,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{
        SelectedSpatialAudioProviderLoadError, load_selected_spatial_audio_provider_bundle,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    const FIXTURE_ROOT: &str = "../jocforge-domain/tests/fixtures";
    const PROVIDERS: [(&str, &str); 5] = [
        (
            "f04_parameters/component-parameter-mathematical-v1.json",
            "parameters/component-parameter-mathematical-v1.json",
        ),
        (
            "f04_parameters/window-q15.json",
            "parameters/window-q15.json",
        ),
        (
            "f04_parameters/qmf-receiver-functional-v1.json",
            "parameters/qmf-receiver-functional-v1.json",
        ),
        (
            "f04_parameters/qmf-mathematical-fft-v1.json",
            "parameters/qmf-mathematical-fft-v1.json",
        ),
        (
            "f04_parameters/descriptor-mathematical.json",
            "parameters/descriptor-mathematical.json",
        ),
    ];

    struct TemporaryProviderPackage(PathBuf);

    impl TemporaryProviderPackage {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time is after the Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "jocforge-f04-provider-loader-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("unique temporary provider directory is created");
            Self(path)
        }

        fn root(&self) -> &Path {
            &self.0
        }

        fn copy_approved_fixtures(&self) {
            let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_ROOT);
            fs::copy(
                fixture_root.join("f01e_provider/MANIFEST.json"),
                self.0.join("MANIFEST.json"),
            )
            .expect("approved package manifest fixture is copied for the loader test");
            for (fixture, relative) in PROVIDERS {
                let destination = self.0.join(relative);
                fs::create_dir_all(destination.parent().unwrap())
                    .expect("approved provider parent directory is created");
                fs::copy(fixture_root.join(fixture), &destination)
                    .expect("approved provider fixture is copied for the loader test");
            }
        }
    }

    impl Drop for TemporaryProviderPackage {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn caller_package_loads_after_manifest_and_each_provider_identity_are_validated() {
        let package = TemporaryProviderPackage::new();
        package.copy_approved_fixtures();

        let bundle = load_selected_spatial_audio_provider_bundle(package.root())
            .expect("approved selected provider package validates");

        assert_eq!(
            bundle.manifest_sha256().to_hex(),
            "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
        );
        let _providers = bundle.spatial_audio_joc_providers();
    }

    #[test]
    fn bad_manifest_is_rejected_before_missing_provider_files_are_read() {
        let package = TemporaryProviderPackage::new();
        fs::write(package.root().join("MANIFEST.json"), b"not-json")
            .expect("invalid manifest fixture is written");

        assert!(matches!(
            load_selected_spatial_audio_provider_bundle(package.root()),
            Err(SelectedSpatialAudioProviderLoadError::Manifest(_))
        ));
    }

    #[test]
    fn valid_manifest_with_a_missing_provider_fails_without_alternate_filename() {
        let package = TemporaryProviderPackage::new();
        let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_ROOT);
        fs::copy(
            fixture_root.join("f01e_provider/MANIFEST.json"),
            package.root().join("MANIFEST.json"),
        )
        .expect("approved manifest fixture is copied");

        assert!(matches!(
            load_selected_spatial_audio_provider_bundle(package.root()),
            Err(SelectedSpatialAudioProviderLoadError::FileRead {
                relative_path: "parameters/component-parameter-mathematical-v1.json",
                ..
            })
        ));
    }
}
