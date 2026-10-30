// pattern: Imperative Shell
//! Loads the exact CurrentFeatures math-provider pair from the approved non-exhaustive asset set.

use jocforge_domain::frontend::{
    CurrentFeaturesProviderError, PackageManifestEvidence, ProviderFileEvidence,
    ValidatedFeatureFft4096MathProvider, ValidatedFeatureProcessorMathProvider,
    validate_current_features_asset_subset_manifest, validate_feature_fft4096_math_provider,
    validate_feature_processor_math_provider,
};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

const PACKAGE_MANIFEST: &str = "MANIFEST.json";
const PROCESSOR_PARAMETERS: &str = "parameters/feature-processor-math-v1.json";
const FFT_PARAMETERS: &str = "parameters/feature-fft4096-math-v1.json";

/// Errors while loading the two exact CurrentFeatures math assets.
#[derive(Debug)]
pub enum SelectedCurrentFeaturesMathProviderLoadError {
    PackageRoot(io::Error),
    PackagePathEscapesRoot {
        relative_path: &'static str,
    },
    FileRead {
        relative_path: &'static str,
        source: io::Error,
    },
    Manifest(CurrentFeaturesProviderError),
    Processor(CurrentFeaturesProviderError),
    Fft(CurrentFeaturesProviderError),
}

impl fmt::Display for SelectedCurrentFeaturesMathProviderLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PackageRoot(error) => write!(formatter, "CurrentFeatures provider root: {error}"),
            Self::PackagePathEscapesRoot { relative_path } => {
                write!(
                    formatter,
                    "CurrentFeatures provider path escapes root: {relative_path}"
                )
            }
            Self::FileRead {
                relative_path,
                source,
            } => write!(
                formatter,
                "CurrentFeatures provider file {relative_path}: {source}"
            ),
            Self::Manifest(error) => {
                write!(
                    formatter,
                    "CurrentFeatures subset manifest rejected: {error:?}"
                )
            }
            Self::Processor(error) => {
                write!(
                    formatter,
                    "CurrentFeatures processor provider rejected: {error:?}"
                )
            }
            Self::Fft(error) => write!(
                formatter,
                "CurrentFeatures FFT provider rejected: {error:?}"
            ),
        }
    }
}

impl std::error::Error for SelectedCurrentFeaturesMathProviderLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PackageRoot(error) | Self::FileRead { source: error, .. } => Some(error),
            Self::PackagePathEscapesRoot { .. }
            | Self::Manifest(_)
            | Self::Processor(_)
            | Self::Fft(_) => None,
        }
    }
}

/// The two typed inputs consumed by CurrentFeatures, without claiming any wider provider set.
pub struct SelectedCurrentFeaturesMathProviders {
    processor: ValidatedFeatureProcessorMathProvider,
    fft: ValidatedFeatureFft4096MathProvider,
}

impl SelectedCurrentFeaturesMathProviders {
    pub const fn processor(&self) -> &ValidatedFeatureProcessorMathProvider {
        &self.processor
    }

    pub const fn fft(&self) -> &ValidatedFeatureFft4096MathProvider {
        &self.fft
    }

    pub const fn declared_asset_count(&self) -> usize {
        18
    }

    pub const fn validated_asset_count(&self) -> usize {
        2
    }

    pub const fn unvalidated_asset_count_open(&self) -> usize {
        16
    }

    pub const fn transitive_provider_dependencies_are_open(&self) -> bool {
        true
    }

    pub fn into_parts(
        self,
    ) -> (
        ValidatedFeatureProcessorMathProvider,
        ValidatedFeatureFft4096MathProvider,
    ) {
        (self.processor, self.fft)
    }
}

/// Loads and validates exactly the processor and FFT assets required by CurrentFeatures.
///
/// The manifest's own identity and its approved source-manifest binding are checked first. This
/// adapter reads no other entries from the 18-asset subset and produces no callback receipt,
/// CurrentFeatures result, or product-readiness claim.
pub fn load_selected_current_features_math_providers(
    package_root: impl AsRef<Path>,
) -> Result<SelectedCurrentFeaturesMathProviders, SelectedCurrentFeaturesMathProviderLoadError> {
    let package_root = package_root.as_ref();
    let canonical_root = package_root
        .canonicalize()
        .map_err(SelectedCurrentFeaturesMathProviderLoadError::PackageRoot)?;

    let manifest_path = package_file(&canonical_root, PACKAGE_MANIFEST)?;
    let manifest_bytes = read_package_file(&manifest_path, PACKAGE_MANIFEST)?;
    let manifest_evidence = PackageManifestEvidence::new(
        manifest_bytes.into(),
        Some(manifest_path.display().to_string()),
    );
    let manifest = validate_current_features_asset_subset_manifest(&manifest_evidence)
        .map_err(SelectedCurrentFeaturesMathProviderLoadError::Manifest)?;

    let processor_evidence = read_provider_evidence(&canonical_root, PROCESSOR_PARAMETERS)?;
    let fft_evidence = read_provider_evidence(&canonical_root, FFT_PARAMETERS)?;
    let processor = validate_feature_processor_math_provider(&processor_evidence, &manifest)
        .map_err(SelectedCurrentFeaturesMathProviderLoadError::Processor)?;
    let fft = validate_feature_fft4096_math_provider(&fft_evidence, &manifest)
        .map_err(SelectedCurrentFeaturesMathProviderLoadError::Fft)?;

    Ok(SelectedCurrentFeaturesMathProviders { processor, fft })
}

fn read_provider_evidence(
    package_root: &Path,
    relative_path: &'static str,
) -> Result<ProviderFileEvidence, SelectedCurrentFeaturesMathProviderLoadError> {
    let path = package_file(package_root, relative_path)?;
    let bytes = read_package_file(&path, relative_path)?;
    Ok(ProviderFileEvidence::new(
        bytes.into(),
        Some(path.display().to_string()),
    ))
}

fn package_file(
    package_root: &Path,
    relative_path: &'static str,
) -> Result<PathBuf, SelectedCurrentFeaturesMathProviderLoadError> {
    let resolved = package_root
        .join(relative_path)
        .canonicalize()
        .map_err(
            |source| SelectedCurrentFeaturesMathProviderLoadError::FileRead {
                relative_path,
                source,
            },
        )?;
    if !resolved.starts_with(package_root) {
        return Err(
            SelectedCurrentFeaturesMathProviderLoadError::PackagePathEscapesRoot { relative_path },
        );
    }
    Ok(resolved)
}

fn read_package_file(
    path: &Path,
    relative_path: &'static str,
) -> Result<Vec<u8>, SelectedCurrentFeaturesMathProviderLoadError> {
    fs::read(path).map_err(
        |source| SelectedCurrentFeaturesMathProviderLoadError::FileRead {
            relative_path,
            source,
        },
    )
}
