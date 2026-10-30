// pattern: Imperative Shell
//! Joins approved spatial/cold and CurrentFeatures values into the existing callback input shape.

use jocforge_application::{
    PreparedJobInput,
    independent_meter_callback::{
        IndependentMeterG0Providers, SelectedSpatialMathGeometry, StandaloneMeterRoot,
    },
};
use jocforge_io::{
    pcm::{meter_source_cursor::MeterPcmSourceCursor, riff_pcm24::ParsedRiffPcm24},
    provider::{
        F03wProviderEntryLineage, SelectedCurrentFeaturesMathProviderLoadError,
        SelectedSpatialAudioProviderBundle, SelectedSpatialAudioProviderLoadError,
        SelectedSpatialColdProviderLineage, SelectedSpatialColdProviderLoadError,
        SelectedSpatialColdProviderRoots, load_selected_current_features_math_providers,
        load_selected_spatial_audio_provider_bundle,
        load_selected_spatial_cold_provider_candidate_for_selected_profile,
    },
};
use std::{error::Error, fmt, path::Path};

use super::independent_meter_callback::{
    IndependentMeterPrepassCandidate, IndependentMeterPrepassError, stage_independent_meter_prepass,
};

const METER_PREPASS_STACK_BYTES: usize = 16 * 1024 * 1024;

/// Load failures while composing approved selected callback provider values.
#[derive(Debug)]
pub enum SelectedIndependentMeterProviderLoadError {
    SpatialCold(SelectedSpatialColdProviderLoadError),
    CurrentFeatures(SelectedCurrentFeaturesMathProviderLoadError),
}

/// Load failures while composing all selected MAIN callback provider values, including F04 receiver.
#[derive(Debug)]
pub enum SelectedMainSpatialProviderLoadError {
    CallbackValues(SelectedIndependentMeterProviderLoadError),
    SpatialAudio(SelectedSpatialAudioProviderLoadError),
}

impl fmt::Display for SelectedMainSpatialProviderLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CallbackValues(error) => write!(formatter, "MAIN callback values: {error}"),
            Self::SpatialAudio(error) => write!(formatter, "F04 receiver providers: {error}"),
        }
    }
}

impl Error for SelectedMainSpatialProviderLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CallbackValues(error) => Some(error),
            Self::SpatialAudio(error) => Some(error),
        }
    }
}

impl fmt::Display for SelectedIndependentMeterProviderLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SpatialCold(error) => write!(formatter, "spatial cold/provider load: {error}"),
            Self::CurrentFeatures(error) => write!(formatter, "CurrentFeatures math load: {error}"),
        }
    }
}

impl Error for SelectedIndependentMeterProviderLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SpatialCold(error) => Some(error),
            Self::CurrentFeatures(error) => Some(error),
        }
    }
}

/// Validated mathematical/cold values for the existing callback API.
///
/// This does not prove runtime layout selection, active-index/point-table identity, callback
/// lineage, a complete provider set, or a production callback result. The P11/P15 geometry
/// inputs remain caller-supplied selected mathematical values.
#[must_use]
pub struct SelectedIndependentMeterCallbackProviders {
    providers: IndependentMeterG0Providers,
    spatial_cold_lineage: SelectedSpatialColdProviderLineage,
    consumed_spatial_provider_entries: [F03wProviderEntryLineage; 3],
    declared_spatial_provider_entries: usize,
    open_spatial_provider_entries: usize,
    declared_current_features_assets: usize,
    validated_current_features_assets: usize,
    open_current_features_assets: usize,
}

impl SelectedIndependentMeterCallbackProviders {
    pub const fn providers(&self) -> &IndependentMeterG0Providers {
        &self.providers
    }

    pub const fn spatial_cold_lineage(&self) -> SelectedSpatialColdProviderLineage {
        self.spatial_cold_lineage
    }

    pub const fn consumed_spatial_provider_entries(&self) -> &[F03wProviderEntryLineage; 3] {
        &self.consumed_spatial_provider_entries
    }

    pub const fn declared_spatial_provider_entries(&self) -> usize {
        self.declared_spatial_provider_entries
    }

    pub const fn open_spatial_provider_entries(&self) -> usize {
        self.open_spatial_provider_entries
    }

    pub const fn declared_current_features_assets(&self) -> usize {
        self.declared_current_features_assets
    }

    pub const fn validated_current_features_assets(&self) -> usize {
        self.validated_current_features_assets
    }

    pub const fn open_current_features_assets(&self) -> usize {
        self.open_current_features_assets
    }

    pub const fn transitive_provider_dependencies_are_open(&self) -> bool {
        true
    }

    pub const fn runtime_layout_binding_is_observed(&self) -> bool {
        false
    }

    pub const fn callback_lineage_is_bound(&self) -> bool {
        false
    }

    pub const fn is_runtime_provider_set(&self) -> bool {
        false
    }

    /// Runs the existing noncommitting selected Meter/F03W callback schedule with these values.
    ///
    /// The caller still supplies the same-resource parsed PCM, source cursor and Meter root. The
    /// returned candidate grants no owner installation, READY, codec AU, or output authority.
    pub fn stage_prepass(
        &self,
        prepared: &PreparedJobInput,
        source_cursor_before: &MeterPcmSourceCursor,
        parsed_source: &ParsedRiffPcm24<'_>,
        meter_before: &StandaloneMeterRoot,
    ) -> Result<IndependentMeterPrepassCandidate, IndependentMeterPrepassError> {
        std::thread::scope(|scope| {
            let worker = std::thread::Builder::new()
                .name("jocforge-independent-meter-prepass".to_owned())
                .stack_size(METER_PREPASS_STACK_BYTES)
                .spawn_scoped(scope, || {
                    stage_independent_meter_prepass(
                        prepared,
                        source_cursor_before,
                        parsed_source,
                        meter_before,
                        &self.providers,
                    )
                })
                .map_err(|_| IndependentMeterPrepassError::WorkerUnavailable)?;
            worker
                .join()
                .map_err(|_| IndependentMeterPrepassError::WorkerPanicked)?
        })
    }
}

/// Typed selected MAIN spatial callback provider values, with each supplied root validated.
///
/// The wrapped F03W and CurrentFeatures loaders retain their explicit open-coverage counts.
#[must_use]
pub struct SelectedMainSpatialCallbackProviders {
    callback_values: SelectedIndependentMeterCallbackProviders,
    spatial_audio: SelectedSpatialAudioProviderBundle,
}

impl SelectedMainSpatialCallbackProviders {
    pub const fn callback_values(&self) -> &SelectedIndependentMeterCallbackProviders {
        &self.callback_values
    }

    pub const fn providers(&self) -> &IndependentMeterG0Providers {
        self.callback_values.providers()
    }

    pub const fn spatial_audio(&self) -> &SelectedSpatialAudioProviderBundle {
        &self.spatial_audio
    }
}

/// Loads approved cold/spatial and CurrentFeatures values into the existing callback input shape.
///
/// The two F03W/cold roots and the separate CurrentFeatures asset subset are independently
/// authenticated. No layout or callback identity is inferred from their manifests.
pub fn load_selected_independent_meter_callback_providers(
    f03w_provider_root: &Path,
    cold_state_root: &Path,
    current_features_root: &Path,
    geometry: SelectedSpatialMathGeometry,
) -> Result<SelectedIndependentMeterCallbackProviders, SelectedIndependentMeterProviderLoadError> {
    let spatial_candidate = load_selected_spatial_cold_provider_candidate_for_selected_profile(
        SelectedSpatialColdProviderRoots::new(f03w_provider_root, cold_state_root),
    )
    .map_err(SelectedIndependentMeterProviderLoadError::SpatialCold)?;
    let spatial_cold_lineage = spatial_candidate.lineage();
    let consumed_spatial_provider_entries = *spatial_candidate.consumed_provider_lineages();
    let declared_spatial_provider_entries = spatial_candidate.declared_provider_entry_count();
    let open_spatial_provider_entries = spatial_candidate.unvalidated_provider_entry_count_open();
    let spatial_values = spatial_candidate.into_callback_values();

    let current_features = load_selected_current_features_math_providers(current_features_root)
        .map_err(SelectedIndependentMeterProviderLoadError::CurrentFeatures)?;
    let declared_current_features_assets = current_features.declared_asset_count();
    let validated_current_features_assets = current_features.validated_asset_count();
    let open_current_features_assets = current_features.unvalidated_asset_count_open();
    let (feature_processor, feature_fft) = current_features.into_parts();

    let providers = IndependentMeterG0Providers::from_selected_values(
        spatial_values.f01e_cold,
        spatial_values.f03_target_cold,
        feature_processor,
        feature_fft,
        spatial_values.window_q15,
        spatial_values.component_parameters,
        spatial_values.diffusion_parameters,
        geometry,
    );

    Ok(SelectedIndependentMeterCallbackProviders {
        providers,
        spatial_cold_lineage,
        consumed_spatial_provider_entries,
        declared_spatial_provider_entries,
        open_spatial_provider_entries,
        declared_current_features_assets,
        validated_current_features_assets,
        open_current_features_assets,
    })
}

/// Loads selected F03W/cold/CurrentFeatures values and the separately approved F04 receiver bundle.
///
/// Each source root is caller-selected and independently validated; tests/fixtures are not read.
/// The returned value does not claim exhaustive transitive F03W or CurrentFeatures provider coverage.
pub fn load_selected_main_spatial_callback_providers(
    f03w_provider_root: &Path,
    cold_state_root: &Path,
    current_features_root: &Path,
    f04_receiver_provider_root: &Path,
    geometry: SelectedSpatialMathGeometry,
) -> Result<SelectedMainSpatialCallbackProviders, SelectedMainSpatialProviderLoadError> {
    let callback_values = load_selected_independent_meter_callback_providers(
        f03w_provider_root,
        cold_state_root,
        current_features_root,
        geometry,
    )
    .map_err(SelectedMainSpatialProviderLoadError::CallbackValues)?;
    let spatial_audio = load_selected_spatial_audio_provider_bundle(f04_receiver_provider_root)
        .map_err(SelectedMainSpatialProviderLoadError::SpatialAudio)?;
    Ok(SelectedMainSpatialCallbackProviders {
        callback_values,
        spatial_audio,
    })
}
