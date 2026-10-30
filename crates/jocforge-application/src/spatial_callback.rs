// pattern: Imperative Shell
//! Production composition of one selected MAIN F03W/F04 callback from admitted input values.

use crate::PreparedJobInput;
use jocforge_domain::meter::prepared_job_identity::PreparedJobInputIdentity;
use jocforge_domain::{
    frontend::{
        ValidatedFeatureFft4096MathProvider, ValidatedFeatureProcessorMathProvider,
        normal_control_snapshot::{NormalControlHeldSourceView, NormalControlSnapshotOutput},
    },
    profile::resolve_selected_profile,
    spatial::{
        SpatialAudioMachine, SpatialControlMachine, SpatialProductionCallbackParentParts,
        SpatialProductionCallbackProposal, ValidatedComponentParameterProvider,
        ValidatedWindowQ15Provider,
        diffusion_filter::DiffusionFilterParameters,
        frontend_stages::{
            FrontendFeatureStageError, stage_spatial_control_frontend_features_from_installed48,
        },
        installed48_source_map::{
            Installed48SourceMapError, build_selected_installed48_source_map,
        },
        production_callback::selected_audio_provider_bundle::SelectedSpatialAudioProviderBundle,
        production_callback::{
            NormalControlCallbackSource, SpatialProductionCallbackError,
            SpatialProductionCallbackInputs, continue_spatial_production_callback,
        },
        selected_p11_geometry::SelectedP11Geometry,
        selected_p15_geometry::SelectedP15Geometry,
        source_flags::{
            BeforeRegularHistory, CurrentHostPcm, PrefixWriterSet, SourceFlagsError,
            SourceFlagsInput, SourceFlagsParentLineage,
            project_held_normal_control_snapshot_source_controls,
            project_normal_control_snapshot_output_source_controls,
        },
        source_geometry_outer::{
            GlobalSourceGeometryError, build_global_source_geometry_snapshot_from_output,
            build_held_global_source_geometry_snapshot,
        },
        spatial_control::{
            SpatialControlInstalled48Callback, SpatialControlInstalled48Error,
            SpatialControlInstalled48Input, stage_spatial_control_installed48,
        },
    },
};
use std::{ops::Deref, sync::Arc};

const MAIN_SPATIAL_CALLBACK_STACK_BYTES: usize = 16 * 1024 * 1024;

/// Inputs for one normal MAIN callback. PCM has already been read, padded, and identity-bound by IO.
pub struct MainSpatialCallbackInput<'a> {
    pub prepared: &'a PreparedJobInput,
    pub controls: NormalControlSnapshotOutput,
    pub physical_pcm_rows: &'a [Vec<u32>],
    pub spatial_control: &'a SpatialControlMachine,
    pub spatial_audio: &'a SpatialAudioMachine,
    pub p11_geometry: &'a SelectedP11Geometry,
    pub p15_geometry: &'a SelectedP15Geometry,
    pub diffusion_parameters: &'a DiffusionFilterParameters,
    pub feature_processor: &'a ValidatedFeatureProcessorMathProvider,
    pub feature_fft: &'a ValidatedFeatureFft4096MathProvider,
    pub window: &'a ValidatedWindowQ15Provider,
    pub component_parameters: &'a ValidatedComponentParameterProvider,
    pub spatial_audio_providers: Option<&'a SelectedSpatialAudioProviderBundle>,
}

/// Inputs for a MAIN cached-drain callback. PCM is read only from SpatialControl's committed cache.
pub struct MainSpatialCachedCallbackInput<'a> {
    pub prepared: &'a PreparedJobInput,
    pub controls: NormalControlHeldSourceView,
    pub spatial_control: &'a SpatialControlMachine,
    pub spatial_audio: &'a SpatialAudioMachine,
    pub p11_geometry: &'a SelectedP11Geometry,
    pub p15_geometry: &'a SelectedP15Geometry,
    pub diffusion_parameters: &'a DiffusionFilterParameters,
    pub feature_processor: &'a ValidatedFeatureProcessorMathProvider,
    pub feature_fft: &'a ValidatedFeatureFft4096MathProvider,
    pub window: &'a ValidatedWindowQ15Provider,
    pub component_parameters: &'a ValidatedComponentParameterProvider,
    pub spatial_audio_providers: Option<&'a SelectedSpatialAudioProviderBundle>,
}

enum MainF01eCallback<'a> {
    Normal {
        controls: Box<NormalControlSnapshotOutput>,
        physical_pcm_rows: &'a [Vec<u32>],
    },
    CachedDrain {
        controls: Box<NormalControlHeldSourceView>,
    },
}

struct MainSpatialCallbackWork<'a> {
    prepared: &'a PreparedJobInput,
    callback: MainF01eCallback<'a>,
    spatial_control: &'a SpatialControlMachine,
    spatial_audio: &'a SpatialAudioMachine,
    p11_geometry: &'a SelectedP11Geometry,
    p15_geometry: &'a SelectedP15Geometry,
    diffusion_parameters: &'a DiffusionFilterParameters,
    feature_processor: &'a ValidatedFeatureProcessorMathProvider,
    feature_fft: &'a ValidatedFeatureFft4096MathProvider,
    window: &'a ValidatedWindowQ15Provider,
    component_parameters: &'a ValidatedComponentParameterProvider,
    spatial_audio_providers: Option<&'a SelectedSpatialAudioProviderBundle>,
}

#[derive(Debug)]
pub enum MainSpatialCallbackError {
    UnsupportedProfile,
    ReceiverProvidersRequired,
    WorkerUnavailable,
    WorkerPanicked,
    PrimaryControlsUnavailable,
    SourceMap(Installed48SourceMapError),
    SourceFlags(SourceFlagsError),
    Installed48(SpatialControlInstalled48Error),
    Frontend(FrontendFeatureStageError),
    SourceGeometry(GlobalSourceGeometryError),
    Callback(SpatialProductionCallbackError),
}

/// Noncommitting MAIN callback proposal bound to the exact admitted input identity capability.
#[must_use]
pub struct MainSpatialCallbackCandidate {
    prepared_identity: Arc<PreparedJobInputIdentity>,
    proposal: SpatialProductionCallbackProposal,
}

/// Parent-facing F03W/F04 values paired with their admitted input identity capability.
#[must_use]
pub struct MainSpatialCallbackParentParts {
    prepared_identity: Arc<PreparedJobInputIdentity>,
    spatial: SpatialProductionCallbackParentParts,
}

impl MainSpatialCallbackParentParts {
    pub const fn spatial(&self) -> &SpatialProductionCallbackParentParts {
        &self.spatial
    }

    pub const fn prepared_identity(&self) -> &Arc<PreparedJobInputIdentity> {
        &self.prepared_identity
    }

    pub fn into_parts(
        self,
    ) -> (
        Arc<PreparedJobInputIdentity>,
        SpatialProductionCallbackParentParts,
    ) {
        (self.prepared_identity, self.spatial)
    }
}

impl MainSpatialCallbackCandidate {
    pub const fn proposal(&self) -> &SpatialProductionCallbackProposal {
        &self.proposal
    }

    pub fn into_parent_parts(self) -> MainSpatialCallbackParentParts {
        MainSpatialCallbackParentParts {
            prepared_identity: self.prepared_identity,
            spatial: self.proposal.into_parent_parts(),
        }
    }

    pub(crate) fn is_bound_to(&self, prepared: &PreparedJobInput) -> bool {
        Arc::ptr_eq(
            &self.prepared_identity,
            &prepared.domain_identity_capability(),
        )
    }
}

impl Deref for MainSpatialCallbackCandidate {
    type Target = SpatialProductionCallbackProposal;

    fn deref(&self) -> &Self::Target {
        &self.proposal
    }
}

/// Stages one real MAIN callback through F03W, F03F, selector/outer, component and receiver.
///
/// The returned proposal retains both owner after-values for the enclosing programme transaction.
/// This function neither installs an owner root nor authorizes output publication.
pub fn stage_main_spatial_callback(
    input: MainSpatialCallbackInput<'_>,
) -> Result<MainSpatialCallbackCandidate, MainSpatialCallbackError> {
    let prepared_identity = input.prepared.domain_identity_capability();
    let MainSpatialCallbackInput {
        prepared,
        controls,
        physical_pcm_rows,
        spatial_control,
        spatial_audio,
        p11_geometry,
        p15_geometry,
        diffusion_parameters,
        feature_processor,
        feature_fft,
        window,
        component_parameters,
        spatial_audio_providers,
    } = input;
    let proposal = stage_main_spatial_callback_work(MainSpatialCallbackWork {
        prepared,
        callback: MainF01eCallback::Normal {
            controls: Box::new(controls),
            physical_pcm_rows,
        },
        spatial_control,
        spatial_audio,
        p11_geometry,
        p15_geometry,
        diffusion_parameters,
        feature_processor,
        feature_fft,
        window,
        component_parameters,
        spatial_audio_providers,
    })?;
    Ok(MainSpatialCallbackCandidate {
        prepared_identity,
        proposal,
    })
}

/// Stages one cached MAIN callback using the exact last-normal F01E controls and SpatialControl PCM cache.
pub fn stage_main_spatial_cached_callback(
    input: MainSpatialCachedCallbackInput<'_>,
) -> Result<MainSpatialCallbackCandidate, MainSpatialCallbackError> {
    let prepared_identity = input.prepared.domain_identity_capability();
    let MainSpatialCachedCallbackInput {
        prepared,
        controls,
        spatial_control,
        spatial_audio,
        p11_geometry,
        p15_geometry,
        diffusion_parameters,
        feature_processor,
        feature_fft,
        window,
        component_parameters,
        spatial_audio_providers,
    } = input;
    let proposal = stage_main_spatial_callback_work(MainSpatialCallbackWork {
        prepared,
        callback: MainF01eCallback::CachedDrain {
            controls: Box::new(controls),
        },
        spatial_control,
        spatial_audio,
        p11_geometry,
        p15_geometry,
        diffusion_parameters,
        feature_processor,
        feature_fft,
        window,
        component_parameters,
        spatial_audio_providers,
    })?;
    Ok(MainSpatialCallbackCandidate {
        prepared_identity,
        proposal,
    })
}

fn stage_main_spatial_callback_work(
    input: MainSpatialCallbackWork<'_>,
) -> Result<SpatialProductionCallbackProposal, MainSpatialCallbackError> {
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("jocforge-main-spatial-callback".to_owned())
            .stack_size(MAIN_SPATIAL_CALLBACK_STACK_BYTES)
            .spawn_scoped(scope, move || stage_main_spatial_callback_inner(input))
            .map_err(|_| MainSpatialCallbackError::WorkerUnavailable)?;
        worker
            .join()
            .map_err(|_| MainSpatialCallbackError::WorkerPanicked)?
    })
}

fn stage_main_spatial_callback_inner(
    input: MainSpatialCallbackWork<'_>,
) -> Result<SpatialProductionCallbackProposal, MainSpatialCallbackError> {
    let MainSpatialCallbackWork {
        prepared,
        callback,
        spatial_control,
        spatial_audio,
        p11_geometry,
        p15_geometry,
        diffusion_parameters,
        feature_processor,
        feature_fft,
        window,
        component_parameters,
        spatial_audio_providers,
    } = input;
    let profile = resolve_selected_profile(&spatial_control.profile())
        .map_err(|_| MainSpatialCallbackError::UnsupportedProfile)?;
    if profile != prepared.configuration().selected_profile() {
        return Err(MainSpatialCallbackError::UnsupportedProfile);
    }
    let primary_controls = prepared
        .primary_metadata_controls()
        .ok_or(MainSpatialCallbackError::PrimaryControlsUnavailable)?;
    let source_map = build_selected_installed48_source_map(primary_controls, p11_geometry)
        .map_err(MainSpatialCallbackError::SourceMap)?;
    let (transaction_id, source_control_clock, source_geometry, source_snapshot, frontend) =
        match callback {
            MainF01eCallback::Normal {
                controls,
                physical_pcm_rows,
            } => {
                let source_projection = project_normal_control_snapshot_output_source_controls(
                    &controls,
                    prepared.graph().graph(),
                    profile,
                )
                .map_err(MainSpatialCallbackError::SourceFlags)?;
                let source_control_clock = source_projection.clock();
                let source_geometry = build_global_source_geometry_snapshot_from_output(
                    prepared.graph().graph(),
                    &source_map,
                    &controls,
                )
                .map_err(MainSpatialCallbackError::SourceGeometry)?;
                let before_history =
                    BeforeRegularHistory::from_owner(spatial_control, source_projection.route())
                        .map_err(MainSpatialCallbackError::SourceFlags)?;
                let transaction_id = controls.transaction_id().clone();
                let parent = SourceFlagsParentLineage::new(transaction_id.clone(), spatial_control)
                    .map_err(MainSpatialCallbackError::SourceFlags)?;
                let installed48 = stage_spatial_control_installed48(
                    spatial_control,
                    SpatialControlInstalled48Input {
                        callback: SpatialControlInstalled48Callback::Normal {
                            controls: &controls,
                            physical_pcm_rows,
                        },
                        source_map: &source_map,
                        diffusion_parameters: Some(diffusion_parameters),
                    },
                )
                .map_err(MainSpatialCallbackError::Installed48)?;
                let prefix_writers = PrefixWriterSet::new(installed48.prefix_writer_indices())
                    .map_err(MainSpatialCallbackError::SourceFlags)?;
                let current_pcm = physical_pcm_rows
                    .iter()
                    .map(|row| row.iter().copied().map(f32::from_bits).collect())
                    .collect();
                let source_flags = SourceFlagsInput::from_primary_regular_source_controls(
                    CurrentHostPcm::new(current_pcm)
                        .map_err(MainSpatialCallbackError::SourceFlags)?,
                    Some(before_history),
                    source_projection,
                    prefix_writers,
                    parent,
                )
                .map_err(MainSpatialCallbackError::SourceFlags)?;
                let frontend = stage_spatial_control_frontend_features_from_installed48(
                    spatial_control,
                    source_flags,
                    installed48,
                    feature_processor,
                    feature_fft,
                )
                .map_err(MainSpatialCallbackError::Frontend)?;
                (
                    transaction_id,
                    source_control_clock,
                    source_geometry,
                    NormalControlCallbackSource::NormalOutput(controls),
                    frontend,
                )
            }
            MainF01eCallback::CachedDrain { controls } => {
                let source_projection = project_held_normal_control_snapshot_source_controls(
                    &controls,
                    prepared.graph().graph(),
                    profile,
                )
                .map_err(MainSpatialCallbackError::SourceFlags)?;
                let source_control_clock = source_projection.clock();
                let source_geometry = build_held_global_source_geometry_snapshot(
                    prepared.graph().graph(),
                    &source_map,
                    &controls,
                )
                .map_err(MainSpatialCallbackError::SourceGeometry)?;
                let before_history =
                    BeforeRegularHistory::from_owner(spatial_control, source_projection.route())
                        .map_err(MainSpatialCallbackError::SourceFlags)?;
                let transaction_id = controls.delivery_transaction_id().clone();
                let parent = SourceFlagsParentLineage::new(transaction_id.clone(), spatial_control)
                    .map_err(MainSpatialCallbackError::SourceFlags)?;
                let installed48 = stage_spatial_control_installed48(
                    spatial_control,
                    SpatialControlInstalled48Input {
                        callback: SpatialControlInstalled48Callback::CachedDrain {
                            controls: &controls,
                        },
                        source_map: &source_map,
                        diffusion_parameters: Some(diffusion_parameters),
                    },
                )
                .map_err(MainSpatialCallbackError::Installed48)?;
                let prefix_writers = PrefixWriterSet::new(installed48.prefix_writer_indices())
                    .map_err(MainSpatialCallbackError::SourceFlags)?;
                let current_pcm = installed48
                    .source_pcm_rows()
                    .iter()
                    .map(|row| row.iter().copied().map(f32::from_bits).collect())
                    .collect();
                let source_flags = SourceFlagsInput::from_primary_regular_source_controls(
                    CurrentHostPcm::new(current_pcm)
                        .map_err(MainSpatialCallbackError::SourceFlags)?,
                    Some(before_history),
                    source_projection,
                    prefix_writers,
                    parent,
                )
                .map_err(MainSpatialCallbackError::SourceFlags)?;
                let frontend = stage_spatial_control_frontend_features_from_installed48(
                    spatial_control,
                    source_flags,
                    installed48,
                    feature_processor,
                    feature_fft,
                )
                .map_err(MainSpatialCallbackError::Frontend)?;
                (
                    transaction_id,
                    source_control_clock,
                    source_geometry,
                    NormalControlCallbackSource::CachedHold(controls),
                    frontend,
                )
            }
        };

    let spatial_audio_joc_providers = if transaction_id.callback_index().value() >= 2 {
        Some(
            spatial_audio_providers
                .ok_or(MainSpatialCallbackError::ReceiverProvidersRequired)?
                .spatial_audio_joc_providers(),
        )
    } else {
        spatial_audio_providers.map(SelectedSpatialAudioProviderBundle::spatial_audio_joc_providers)
    };

    continue_spatial_production_callback(SpatialProductionCallbackInputs {
        frontend,
        source_snapshot,
        source_geometry,
        source_control_clock,
        source_map: &source_map,
        p15_geometry,
        spatial_control,
        spatial_audio,
        spatial_audio_joc_providers,
        window,
        parameters: component_parameters,
    })
    .map_err(MainSpatialCallbackError::Callback)
}
