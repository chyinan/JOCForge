// pattern: Imperative Shell
//! Loads the exact CurrentFeatures math-provider pair from the approved non-exhaustive asset set.

use super::tree::{ProviderReadError, ProviderTree};
use jocforge_domain::frontend::{
    CurrentFeaturesProviderError, PackageManifestEvidence, ProviderFileEvidence,
    ValidatedFeatureFft4096MathProvider, ValidatedFeatureProcessorMathProvider,
    validate_current_features_asset_subset_manifest, validate_feature_fft4096_math_provider,
    validate_feature_processor_math_provider,
};
use std::{fmt, io, sync::Arc};

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
    tree: ProviderTree<'_>,
) -> Result<SelectedCurrentFeaturesMathProviders, SelectedCurrentFeaturesMathProviderLoadError> {
    let manifest_evidence = PackageManifestEvidence::new(
        read_member(tree, PACKAGE_MANIFEST)?,
        Some(tree.display(PACKAGE_MANIFEST)),
    );
    let manifest = validate_current_features_asset_subset_manifest(&manifest_evidence)
        .map_err(SelectedCurrentFeaturesMathProviderLoadError::Manifest)?;

    let processor_evidence = read_provider_evidence(tree, PROCESSOR_PARAMETERS)?;
    let fft_evidence = read_provider_evidence(tree, FFT_PARAMETERS)?;
    let processor = validate_feature_processor_math_provider(&processor_evidence, &manifest)
        .map_err(SelectedCurrentFeaturesMathProviderLoadError::Processor)?;
    let fft = validate_feature_fft4096_math_provider(&fft_evidence, &manifest)
        .map_err(SelectedCurrentFeaturesMathProviderLoadError::Fft)?;

    Ok(SelectedCurrentFeaturesMathProviders { processor, fft })
}

fn read_provider_evidence(
    tree: ProviderTree<'_>,
    relative_path: &'static str,
) -> Result<ProviderFileEvidence, SelectedCurrentFeaturesMathProviderLoadError> {
    Ok(ProviderFileEvidence::new(
        read_member(tree, relative_path)?,
        Some(tree.display(relative_path)),
    ))
}

fn read_member(
    tree: ProviderTree<'_>,
    relative_path: &'static str,
) -> Result<Arc<[u8]>, SelectedCurrentFeaturesMathProviderLoadError> {
    tree.read(relative_path).map_err(|error| match error {
        ProviderReadError::EscapesRoot => {
            SelectedCurrentFeaturesMathProviderLoadError::PackagePathEscapesRoot { relative_path }
        }
        error => SelectedCurrentFeaturesMathProviderLoadError::FileRead {
            relative_path,
            source: error.into_io_error(relative_path),
        },
    })
}
