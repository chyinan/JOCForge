// pattern: Imperative Shell
//! Loads independently rooted selected cold schemas and spatial parameters.
use jocforge_domain::{
    frontend::{
        F01EProviderError, F03TargetProviderError, F03wAssetManifestError,
        F03wProviderEntryLineage, PackageManifestEvidence, ProviderFileEvidence, ProviderSha256,
        SelectedColdContractError, ValidatedF01EColdProvider, ValidatedF03TargetColdProvider,
        ValidatedInstalled48StateSchema, ValidatedSelectedOwnerProjection,
        validate_f01b_cold_asset_manifest, validate_f01e_provider_from_f01b_cold_assets,
        validate_f03_target_cold_provider_from_f01b_assets, validate_f03w_provider_manifest,
        validate_installed48_state_schema, validate_selected_owner_projection,
    },
    profile::EncodingProfile,
    spatial::{
        F04ParameterProviderError, ValidatedComponentParameterProvider, ValidatedWindowQ15Provider,
        component_parameters_provider::validate_component_parameter_provider_from_f03w_bundle,
        diffusion_filter::{
            DiffusionFilterError, DiffusionFilterParameters,
            validate_diffusion_filter_parameters_from_f03w_bundle,
        },
        window_q15_provider::validate_window_q15_provider_from_f01b_cold_assets,
    },
};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

const MANIFEST: &str = "MANIFEST.json";
const INSTALLED48_STATE: &str = "parameters/adm-installed48-state-v1.json";
const F03_TARGET_COLD: &str = "parameters/f03-selected-target-cold-v1.json";
const OWNER_PROJECTION: &str = "parameters/selected-owner-projection-v1.json";
const WINDOW_Q15: &str = "parameters/window-q15.json";
const F01E_COLD: &str = "parameters/f01e-selected-constructor-v1.json";
const COMPONENT_PARAMETERS: &str = "parameters/component-parameter-mathematical-v1.json";
const DIFFUSION_PARAMETERS: &str = "parameters/diffusion-mathematical-v1.json";
const DIFFUSION_CONSTRUCTOR: &str = "parameters/diffusion-constructor-mathematical-v1.json";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedProviderRoot {
    F03w,
    ColdState,
}

/// Caller-owned locations for the two independently authenticated bundle roots.
#[derive(Clone, Copy)]
pub struct SelectedSpatialColdProviderRoots<'a> {
    f03w_provider_root: &'a Path,
    cold_state_root: &'a Path,
}

impl<'a> SelectedSpatialColdProviderRoots<'a> {
    pub const fn new(f03w_provider_root: &'a Path, cold_state_root: &'a Path) -> Self {
        Self {
            f03w_provider_root,
            cold_state_root,
        }
    }
}

/// Independent identities for the provider and cold-state manifests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedSpatialColdProviderLineage {
    f03w_manifest_sha256: ProviderSha256,
    cold_state_manifest_sha256: ProviderSha256,
}

impl SelectedSpatialColdProviderLineage {
    pub const fn f03w_manifest_sha256(self) -> ProviderSha256 {
        self.f03w_manifest_sha256
    }

    pub const fn cold_state_manifest_sha256(self) -> ProviderSha256 {
        self.cold_state_manifest_sha256
    }
}

/// Static immutable cold/provider values; no layout, callback, commit, READY, or AU is created.
pub struct SelectedSpatialColdProviderCandidate {
    lineage: SelectedSpatialColdProviderLineage,
    consumed_provider_lineages: [F03wProviderEntryLineage; 3],
    installed48_schema: ValidatedInstalled48StateSchema,
    owner_projection: ValidatedSelectedOwnerProjection,
    f01e_cold: ValidatedF01EColdProvider,
    f03_target_cold: ValidatedF03TargetColdProvider,
    window_q15: ValidatedWindowQ15Provider,
    component_parameters: ValidatedComponentParameterProvider,
    diffusion_parameters: DiffusionFilterParameters,
}

impl SelectedSpatialColdProviderCandidate {
    pub const fn lineage(&self) -> SelectedSpatialColdProviderLineage {
        self.lineage
    }

    pub const fn consumed_provider_lineages(&self) -> &[F03wProviderEntryLineage; 3] {
        &self.consumed_provider_lineages
    }

    pub const fn declared_provider_entry_count(&self) -> usize {
        48
    }

    pub const fn unvalidated_provider_entry_count_open(&self) -> usize {
        45
    }

    pub const fn transitive_provider_consumption_is_open(&self) -> bool {
        true
    }

    pub const fn is_runtime_provider_set(&self) -> bool {
        false
    }

    pub const fn installed48_schema(&self) -> &ValidatedInstalled48StateSchema {
        &self.installed48_schema
    }

    pub const fn owner_projection(&self) -> &ValidatedSelectedOwnerProjection {
        &self.owner_projection
    }

    pub const fn f01e_cold(&self) -> &ValidatedF01EColdProvider {
        &self.f01e_cold
    }

    pub const fn f03_target_cold(&self) -> &ValidatedF03TargetColdProvider {
        &self.f03_target_cold
    }

    pub const fn window_q15(&self) -> &ValidatedWindowQ15Provider {
        &self.window_q15
    }

    pub const fn component_parameters(&self) -> &ValidatedComponentParameterProvider {
        &self.component_parameters
    }

    pub const fn diffusion_parameters(&self) -> &DiffusionFilterParameters {
        &self.diffusion_parameters
    }

    /// Transfers the typed values consumed by a selected spatial callback.
    ///
    /// The caller must retain this candidate's lineage/open-scope metadata before consuming it.
    /// The returned values do not represent a runtime layout selection or callback receipt.
    pub fn into_callback_values(self) -> SelectedSpatialColdProviderCallbackValues {
        SelectedSpatialColdProviderCallbackValues {
            f01e_cold: self.f01e_cold,
            f03_target_cold: self.f03_target_cold,
            window_q15: self.window_q15,
            component_parameters: self.component_parameters,
            diffusion_parameters: self.diffusion_parameters,
        }
    }
}

/// Validated cold/provider values consumed by the existing spatial callback stage.
#[must_use]
pub struct SelectedSpatialColdProviderCallbackValues {
    pub f01e_cold: ValidatedF01EColdProvider,
    pub f03_target_cold: ValidatedF03TargetColdProvider,
    pub window_q15: ValidatedWindowQ15Provider,
    pub component_parameters: ValidatedComponentParameterProvider,
    pub diffusion_parameters: DiffusionFilterParameters,
}

#[derive(Debug)]
pub enum SelectedSpatialColdProviderLoadError {
    PackageRoot {
        bundle: SelectedProviderRoot,
        source: io::Error,
    },
    PackageRootsNotDistinct,
    PackagePathEscapesRoot {
        bundle: SelectedProviderRoot,
        relative_path: &'static str,
    },
    FileRead {
        bundle: SelectedProviderRoot,
        relative_path: &'static str,
        source: io::Error,
    },
    F03wManifest(F03wAssetManifestError),
    ColdManifest(F03wAssetManifestError),
    Installed48Schema(SelectedColdContractError),
    OwnerProjection(SelectedColdContractError),
    F01eCold(F01EProviderError),
    F03TargetCold(F03TargetProviderError),
    WindowQ15(F04ParameterProviderError),
    ComponentParameters(F04ParameterProviderError),
    DiffusionParameters(DiffusionFilterError),
}

impl fmt::Display for SelectedSpatialColdProviderLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PackageRoot { bundle, source } => {
                write!(formatter, "{bundle:?} provider root: {source}")
            }
            Self::PackageRootsNotDistinct => {
                formatter.write_str("F03W and cold-state roots must be distinct")
            }
            Self::PackagePathEscapesRoot {
                bundle,
                relative_path,
            } => write!(formatter, "{bundle:?} path escapes root: {relative_path}"),
            Self::FileRead {
                bundle,
                relative_path,
                source,
            } => write!(formatter, "{bundle:?} file {relative_path}: {source}"),
            Self::F03wManifest(error) => write!(formatter, "F03W manifest rejected: {error:?}"),
            Self::ColdManifest(error) => {
                write!(formatter, "cold-state manifest rejected: {error:?}")
            }
            Self::Installed48Schema(error) => {
                write!(formatter, "installed48 schema rejected: {error:?}")
            }
            Self::OwnerProjection(error) => {
                write!(formatter, "owner projection rejected: {error:?}")
            }
            Self::F01eCold(error) => write!(formatter, "F01E cold rejected: {error:?}"),
            Self::F03TargetCold(error) => write!(formatter, "F03 target cold rejected: {error:?}"),
            Self::WindowQ15(error) => write!(formatter, "Q15 window rejected: {error:?}"),
            Self::ComponentParameters(error) => {
                write!(formatter, "component parameters rejected: {error:?}")
            }
            Self::DiffusionParameters(error) => {
                write!(formatter, "diffusion parameters rejected: {error:?}")
            }
        }
    }
}

impl std::error::Error for SelectedSpatialColdProviderLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PackageRoot { source, .. } | Self::FileRead { source, .. } => Some(source),
            Self::PackageRootsNotDistinct
            | Self::PackagePathEscapesRoot { .. }
            | Self::F03wManifest(_)
            | Self::ColdManifest(_)
            | Self::Installed48Schema(_)
            | Self::OwnerProjection(_)
            | Self::F01eCold(_)
            | Self::F03TargetCold(_)
            | Self::WindowQ15(_)
            | Self::ComponentParameters(_)
            | Self::DiffusionParameters(_) => None,
        }
    }
}

/// Authenticates the two distinct bundle roots and validates typed cold/provider values.
/// It does not construct an owner machine, layout, callback source, or runtime provider set.
pub fn load_selected_spatial_cold_provider_candidate(
    roots: SelectedSpatialColdProviderRoots<'_>,
    profile: EncodingProfile,
) -> Result<SelectedSpatialColdProviderCandidate, SelectedSpatialColdProviderLoadError> {
    let f03w_root = canonical_root(roots.f03w_provider_root, SelectedProviderRoot::F03w)?;
    let cold_root = canonical_root(roots.cold_state_root, SelectedProviderRoot::ColdState)?;
    if f03w_root == cold_root {
        return Err(SelectedSpatialColdProviderLoadError::PackageRootsNotDistinct);
    }

    let f03w_manifest_evidence = read_manifest(&f03w_root, SelectedProviderRoot::F03w)?;
    let f03w_manifest = validate_f03w_provider_manifest(&f03w_manifest_evidence)
        .map_err(SelectedSpatialColdProviderLoadError::F03wManifest)?;
    let cold_manifest_evidence = read_manifest(&cold_root, SelectedProviderRoot::ColdState)?;
    let cold_manifest = validate_f01b_cold_asset_manifest(&cold_manifest_evidence)
        .map_err(SelectedSpatialColdProviderLoadError::ColdManifest)?;

    let installed48_schema_evidence = read_provider(
        &cold_root,
        SelectedProviderRoot::ColdState,
        INSTALLED48_STATE,
    )?;
    let installed48_schema =
        validate_installed48_state_schema(&installed48_schema_evidence, &cold_manifest)
            .map_err(SelectedSpatialColdProviderLoadError::Installed48Schema)?;
    let owner_projection_evidence = read_provider(
        &cold_root,
        SelectedProviderRoot::ColdState,
        OWNER_PROJECTION,
    )?;
    let owner_projection =
        validate_selected_owner_projection(&owner_projection_evidence, &cold_manifest)
            .map_err(SelectedSpatialColdProviderLoadError::OwnerProjection)?;
    let f03_target_evidence =
        read_provider(&cold_root, SelectedProviderRoot::ColdState, F03_TARGET_COLD)?;
    let f03_target_cold =
        validate_f03_target_cold_provider_from_f01b_assets(&f03_target_evidence, &cold_manifest)
            .map_err(SelectedSpatialColdProviderLoadError::F03TargetCold)?;
    let f01e_evidence = read_provider(&cold_root, SelectedProviderRoot::ColdState, F01E_COLD)?;
    let f01e_cold = validate_f01e_provider_from_f01b_cold_assets(&f01e_evidence, &cold_manifest)
        .map_err(SelectedSpatialColdProviderLoadError::F01eCold)?;
    let window_evidence = read_provider(&cold_root, SelectedProviderRoot::ColdState, WINDOW_Q15)?;
    let window_q15 =
        validate_window_q15_provider_from_f01b_cold_assets(&window_evidence, &cold_manifest)
            .map_err(SelectedSpatialColdProviderLoadError::WindowQ15)?;

    let component_evidence =
        read_provider(&f03w_root, SelectedProviderRoot::F03w, COMPONENT_PARAMETERS)?;
    let component_parameters = validate_component_parameter_provider_from_f03w_bundle(
        &component_evidence,
        &f03w_manifest,
        profile,
    )
    .map_err(SelectedSpatialColdProviderLoadError::ComponentParameters)?;
    let diffusion_evidence =
        read_provider(&f03w_root, SelectedProviderRoot::F03w, DIFFUSION_PARAMETERS)?;
    let diffusion_constructor_evidence = read_provider(
        &f03w_root,
        SelectedProviderRoot::F03w,
        DIFFUSION_CONSTRUCTOR,
    )?;
    let diffusion_parameters = validate_diffusion_filter_parameters_from_f03w_bundle(
        &diffusion_evidence,
        &diffusion_constructor_evidence,
        &f03w_manifest,
    )
    .map_err(SelectedSpatialColdProviderLoadError::DiffusionParameters)?;

    let lineage = SelectedSpatialColdProviderLineage {
        f03w_manifest_sha256: f03w_manifest.manifest_sha256(),
        cold_state_manifest_sha256: cold_manifest.manifest_sha256(),
    };
    Ok(SelectedSpatialColdProviderCandidate {
        lineage,
        consumed_provider_lineages: f03w_manifest.consumed_entry_lineages(),
        installed48_schema,
        owner_projection,
        f01e_cold,
        f03_target_cold,
        window_q15,
        component_parameters,
        diffusion_parameters,
    })
}

/// Loads the selected profile's cold/provider values without a caller-created profile variant.
pub fn load_selected_spatial_cold_provider_candidate_for_selected_profile(
    roots: SelectedSpatialColdProviderRoots<'_>,
) -> Result<SelectedSpatialColdProviderCandidate, SelectedSpatialColdProviderLoadError> {
    load_selected_spatial_cold_provider_candidate(roots, EncodingProfile::selected())
}

fn canonical_root(
    root: &Path,
    bundle: SelectedProviderRoot,
) -> Result<PathBuf, SelectedSpatialColdProviderLoadError> {
    root.canonicalize()
        .map_err(|source| SelectedSpatialColdProviderLoadError::PackageRoot { bundle, source })
}

fn read_manifest(
    root: &Path,
    bundle: SelectedProviderRoot,
) -> Result<PackageManifestEvidence, SelectedSpatialColdProviderLoadError> {
    let path = package_file(root, bundle, MANIFEST)?;
    let bytes = read_file(&path, bundle, MANIFEST)?;
    Ok(PackageManifestEvidence::new(
        bytes,
        Some(path.display().to_string()),
    ))
}

fn read_provider(
    root: &Path,
    bundle: SelectedProviderRoot,
    relative: &'static str,
) -> Result<ProviderFileEvidence, SelectedSpatialColdProviderLoadError> {
    let path = package_file(root, bundle, relative)?;
    let bytes = read_file(&path, bundle, relative)?;
    Ok(ProviderFileEvidence::new(
        bytes,
        Some(path.display().to_string()),
    ))
}

fn package_file(
    root: &Path,
    bundle: SelectedProviderRoot,
    relative: &'static str,
) -> Result<PathBuf, SelectedSpatialColdProviderLoadError> {
    let path = root.join(relative).canonicalize().map_err(|source| {
        SelectedSpatialColdProviderLoadError::FileRead {
            bundle,
            relative_path: relative,
            source,
        }
    })?;
    if !path.starts_with(root) {
        return Err(
            SelectedSpatialColdProviderLoadError::PackagePathEscapesRoot {
                bundle,
                relative_path: relative,
            },
        );
    }
    Ok(path)
}

fn read_file(
    path: &Path,
    bundle: SelectedProviderRoot,
    relative: &'static str,
) -> Result<Arc<[u8]>, SelectedSpatialColdProviderLoadError> {
    fs::read(path).map(Arc::<[u8]>::from).map_err(|source| {
        SelectedSpatialColdProviderLoadError::FileRead {
            bundle,
            relative_path: relative,
            source,
        }
    })
}
