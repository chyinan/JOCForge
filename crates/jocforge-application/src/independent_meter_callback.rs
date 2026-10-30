// pattern: Functional Core
//! Independent Meter-phase g=0 producers from physical input through F04A/F05R56 and OAR.

use crate::{
    PreparedJobInput,
    meter_source_bridge::{
        FrontendControlReplacementCandidate, MeterCallbackIndex, MeterCallbackPhase,
        MeterComponentOutputRows, MeterRootCallbackProgress, MeterSourceBridgeError,
        MeterSourceCallbackView, MeterSourceOarReceipt, MeterSourcePosition,
        MeterSourceSampleCount, NormalControlSnapshotOutput, SpatialAudioReplacementCandidate,
        SpatialControlReplacementCandidate,
    },
};
pub use jocforge_domain::meter::owner::MeterCallbackGtcBinding;
pub use jocforge_domain::meter::standalone_root::{
    MeterKFilterExponentState, StandaloneMeterMeasurementProposal, StandaloneMeterRoot,
};
pub use jocforge_domain::transaction::{
    AttemptId, CallbackTransactionId, GtcOwnerReceipt, ProgrammeId,
};

use jocforge_domain::{
    frontend::{
        FrontendControlMachine, ValidatedF01EColdProvider, ValidatedF03TargetColdProvider,
        ValidatedFeatureFft4096MathProvider, ValidatedFeatureProcessorMathProvider,
        normal_control_snapshot::{
            NormalControlBeforeState, NormalControlFreshZeroMeterView,
            propose_normal_control_snapshot,
        },
    },
    joc::oamd::reader_target::ReaderTargetControlRows,
    meter::{
        installed48_renderer::{
            MeterInstalled48Input, MeterInstalled48State, stage_meter_installed48,
        },
        owner::{MeterCallbackGtcProposal as DomainMeterCallbackGtcProposal, MeterOwnerGtcError},
    },
    profile::{EncodingProfile, resolve_selected_profile},
    spatial::{
        SpatialAudioMachine, SpatialControlMachine, ValidatedComponentParameterProvider,
        ValidatedWindowQ15Provider,
        diffusion_filter::DiffusionFilterParameters,
        frontend_stages::RenderedFrontendFeatureStageProposal,
        frontend_stages::{
            stage_frontend_features_from_installed48,
            stage_meter_frontend_features_from_fresh_zero_input,
        },
        installed48_activity::{
            PrimaryActivityError, PrimaryActivityInput, PrimaryActivityState,
            PrimaryInstalled48ActivityProducer,
        },
        installed48_source_map::Installed48SourceMap,
        installed48_source_map::build_meter_installed48_source_map,
        post_window_fanout::CallbackPhase,
        production_callback::{
            MeterSpatialProductionCallbackInputs, MeterSpatialProductionCallbackProposal,
            NormalControlCallbackSource, SpatialProductionCallbackError,
            SpatialProductionCallbackInputs, SpatialProductionCallbackProposal,
            continue_meter_spatial_production_callback, continue_spatial_production_callback,
        },
        selected_p11_geometry::{
            SelectedP11Geometry, SelectedP11GeometryError, generate_selected_p11_geometry,
        },
        selected_p15_geometry::{
            SelectedP15Geometry, SelectedP15GeometryError, generate_selected_p15_geometry,
        },
        source_flags::{
            BeforeRegularHistory, CurrentHostPcm, PrefixWriterSet,
            PrimaryRegularSourceControlProjection, SourceControlSnapshotClock, SourceFlagsError,
            SourceFlagsInput, SourceFlagsParentLineage,
            project_meter_fresh_zero_normal_control_snapshot_source_controls,
            project_normal_control_snapshot_output_source_controls,
        },
        source_geometry_outer::{
            GlobalSourceGeometrySnapshot, build_global_source_geometry_snapshot_from_output,
            build_meter_fresh_zero_global_source_geometry_snapshot,
        },
    },
    timeline::{AdmSampleIndex, FrontendCallbackIndex, NormalCallbackCount},
    transaction::CallbackLineageIdentity,
};

/// Caller-validated inputs for the independent Meter g=0 producer chain.
///
/// These are actual validated provider values and selected typed geometry. The application does
/// not load fixture bytes, use main-pipeline candidates, or manufacture expected producer rows.
pub struct IndependentMeterG0Providers {
    pub f01e: ValidatedF01EColdProvider,
    pub f03_target: ValidatedF03TargetColdProvider,
    pub feature_processor: ValidatedFeatureProcessorMathProvider,
    pub feature_fft: ValidatedFeatureFft4096MathProvider,
    pub window: ValidatedWindowQ15Provider,
    pub component_parameters: ValidatedComponentParameterProvider,
    pub diffusion: DiffusionFilterParameters,
    pub p11_geometry: SelectedP11Geometry,
    pub p15_geometry: SelectedP15Geometry,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialCallbackExecutionError {
    WorkerUnavailable,
    WorkerPanicked,
    Spatial(SpatialProductionCallbackError),
}

/// Executes one spatial producer callback on a dedicated worker with a 16 MiB stack.
///
/// The callback still only returns pending values; this adapter provides stack capacity and no
/// owner commit, receipt admission or output authority.
pub fn continue_spatial_production_callback_on_worker(
    input: SpatialProductionCallbackInputs<'_>,
) -> Result<SpatialProductionCallbackProposal, SpatialCallbackExecutionError> {
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("jocforge-spatial-callback".to_owned())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, || continue_spatial_production_callback(input))
            .map_err(|_| SpatialCallbackExecutionError::WorkerUnavailable)?;
        worker
            .join()
            .map_err(|_| SpatialCallbackExecutionError::WorkerPanicked)?
            .map_err(SpatialCallbackExecutionError::Spatial)
    })
}

/// Selected mathematical P11/P15 values for callback inputs, without runtime layout evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedSpatialMathGeometry {
    p11: SelectedP11Geometry,
    p15: SelectedP15Geometry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedSpatialMathGeometryError {
    P11(SelectedP11GeometryError),
    P15(SelectedP15GeometryError),
}

/// Generates the approved selected P11/P15 mathematical basis values.
///
/// The result binds only the selected model recipes. It does not attest that a native/runtime
/// callsite selected these layouts or supplied the same point table.
pub fn generate_selected_spatial_math_geometry()
-> Result<SelectedSpatialMathGeometry, SelectedSpatialMathGeometryError> {
    let p11 = generate_selected_p11_geometry(0x111b, true)
        .map_err(SelectedSpatialMathGeometryError::P11)?;
    let p15 = generate_selected_p15_geometry(0x159b, true)
        .map_err(SelectedSpatialMathGeometryError::P15)?;
    Ok(SelectedSpatialMathGeometry { p11, p15 })
}

impl SelectedSpatialMathGeometry {
    pub const fn p11(&self) -> &SelectedP11Geometry {
        &self.p11
    }

    pub const fn p15(&self) -> &SelectedP15Geometry {
        &self.p15
    }

    pub fn into_parts(self) -> (SelectedP11Geometry, SelectedP15Geometry) {
        (self.p11, self.p15)
    }
}

impl IndependentMeterG0Providers {
    /// Joins validated cold/math assets with the selected mathematical geometry inputs.
    // Each argument is a separately authenticated provider value owned by this callback seam.
    #[allow(clippy::too_many_arguments)]
    pub fn from_selected_values(
        f01e: ValidatedF01EColdProvider,
        f03_target: ValidatedF03TargetColdProvider,
        feature_processor: ValidatedFeatureProcessorMathProvider,
        feature_fft: ValidatedFeatureFft4096MathProvider,
        window: ValidatedWindowQ15Provider,
        component_parameters: ValidatedComponentParameterProvider,
        diffusion: DiffusionFilterParameters,
        geometry: SelectedSpatialMathGeometry,
    ) -> Self {
        let (p11_geometry, p15_geometry) = geometry.into_parts();
        Self {
            f01e,
            f03_target,
            feature_processor,
            feature_fft,
            window,
            component_parameters,
            diffusion,
            p11_geometry,
            p15_geometry,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndependentMeterG0Error {
    SourceBridge(MeterSourceBridgeError),
    MeterGtc(MeterOwnerGtcError),
    UnsupportedClock,
    PreparedIdentityMismatch,
    PrimaryMetadataMissing,
    SourceMap,
    Provider,
    FrontendControl,
    FrontendControlBeforeState,
    FrontendControlProposal,
    FrontendControlStage,
    FrontendControlCandidate,
    Activity(PrimaryActivityError),
    MeterInstalled48,
    FreshZeroControlsMissing,
    SourceFlags(SourceFlagsError),
    FrontendFeatures,
    SpatialProduction(SpatialProductionCallbackError),
    SpatialControl,
    SpatialAudio,
    ActivityAfterStateMissing,
    NormalOutputMissing,
    SourceGeometryMissing,
    SourceProjectionMissing,
}

/// Noncommitting after-values from an independent Meter g=0 producer callback.
///
/// The frontend, activity, spatial, and Meter roots remain proposals. In particular this value
/// cannot set READY or commit the Meter root. The parent must retain every after-owner state when
/// it later constructs the complete programme root.
#[must_use]
pub struct IndependentMeterG0Candidate {
    frontend: FrontendControlReplacementCandidate,
    primary_activity_after: PrimaryActivityState,
    installed48_after: MeterInstalled48State,
    spatial_control: SpatialControlReplacementCandidate,
    spatial_audio: SpatialAudioReplacementCandidate,
    component_rows: MeterComponentOutputRows,
    reader_target_controls: ReaderTargetControlRows,
    fresh_zero_controls: Option<NormalControlFreshZeroMeterView>,
    source_receipt: MeterSourceOarReceipt,
    meter: DomainMeterCallbackGtcProposal,
}

impl IndependentMeterG0Candidate {
    pub const fn frontend_candidate(&self) -> &FrontendControlReplacementCandidate {
        &self.frontend
    }

    pub const fn primary_activity_after(&self) -> &PrimaryActivityState {
        &self.primary_activity_after
    }

    pub const fn installed48_after(&self) -> &MeterInstalled48State {
        &self.installed48_after
    }

    pub const fn spatial_control_candidate(&self) -> &SpatialControlReplacementCandidate {
        &self.spatial_control
    }

    pub const fn spatial_audio_candidate(&self) -> &SpatialAudioReplacementCandidate {
        &self.spatial_audio
    }

    pub const fn component_rows(&self) -> &MeterComponentOutputRows {
        &self.component_rows
    }

    pub const fn reader_target_controls(&self) -> &ReaderTargetControlRows {
        &self.reader_target_controls
    }

    pub const fn fresh_zero_controls(&self) -> Option<&NormalControlFreshZeroMeterView> {
        self.fresh_zero_controls.as_ref()
    }

    pub const fn source_receipt(&self) -> &MeterSourceOarReceipt {
        &self.source_receipt
    }

    pub const fn meter_proposal(&self) -> &DomainMeterCallbackGtcProposal {
        &self.meter
    }
}

struct IndependentMeterG0NormalStage {
    frontend: FrontendControlReplacementCandidate,
    spatial_control: SpatialControlMachine,
    spatial_audio: SpatialAudioMachine,
    source_map: Installed48SourceMap,
    normal_output: Option<NormalControlSnapshotOutput>,
    fresh_zero_controls: Option<NormalControlFreshZeroMeterView>,
    source_geometry: Option<GlobalSourceGeometrySnapshot>,
    source_projection: Option<PrimaryRegularSourceControlProjection>,
}

/// Produces one fresh Meter-phase g=0 callback through F01E, F03W, F04A/F05R56 and OAR.
///
/// The input view carries the physical PCM and IO-owned read index. This function creates a new
/// Meter-specific F01E frontend and fresh spatial owners, computes Installed48/activity and the
/// same-callback component/target rows, then stages the OAR and Meter candidates. It accepts no
/// main-programme owner after-state and returns no commit authority. Only the selected normal g=0
/// path is implemented here; the independent zero-input drain producers remain separate work.
pub fn stage_independent_meter_g0_callback(
    prepared: &PreparedJobInput,
    source_view: MeterSourceCallbackView<'_>,
    meter_before: &StandaloneMeterRoot,
    meter_binding: MeterCallbackGtcBinding,
    k_filter_exponents_after: MeterKFilterExponentState,
    providers: &IndependentMeterG0Providers,
) -> Result<IndependentMeterG0Candidate, IndependentMeterG0Error> {
    validate_callback_identity(prepared, &source_view, meter_before, &meter_binding)?;
    let mut normal_stage = stage_independent_meter_g0_normal(prepared, &meter_binding, providers)?;
    let (rendered_frontend, primary_activity_after, source_control_clock) =
        stage_independent_meter_g0_features(prepared, &source_view, &mut normal_stage, providers)?;
    let spatial_stage = continue_independent_meter_g0_spatial(
        &mut normal_stage,
        rendered_frontend,
        source_control_clock,
        providers,
    )?;
    finish_independent_meter_g0_callback(
        source_view,
        meter_before,
        meter_binding,
        k_filter_exponents_after,
        normal_stage,
        primary_activity_after,
        spatial_stage,
    )
}

/// Advances the independent Meter chain by one subsequent normal source callback.
///
/// Every owner starts from the previous candidate's after-state. The function has no cold-start
/// path and returns another candidate without installing any owner.
#[inline(never)]
pub fn stage_independent_meter_normal_callback(
    prepared: &PreparedJobInput,
    source_view: MeterSourceCallbackView<'_>,
    previous: IndependentMeterG0Candidate,
    meter_binding: MeterCallbackGtcBinding,
    k_filter_exponents_after: MeterKFilterExponentState,
    providers: &IndependentMeterG0Providers,
) -> Result<IndependentMeterG0Candidate, IndependentMeterG0Error> {
    let previous = Box::new(previous);
    validate_normal_callback_identity(prepared, &source_view, &previous, &meter_binding)?;
    let mut normal_stage = stage_independent_meter_normal_source(
        prepared,
        &source_view,
        &previous,
        &meter_binding,
        providers,
    )?;
    let (rendered_frontend, activity_after, source_control_clock) =
        stage_independent_meter_normal_features(
            prepared,
            &source_view,
            &previous,
            &mut normal_stage,
            providers,
        )?;
    let spatial_stage = continue_independent_meter_normal_spatial(
        &previous,
        &mut normal_stage,
        rendered_frontend,
        source_control_clock,
        providers,
    )?;
    finish_independent_meter_normal_callback(
        source_view,
        &previous,
        meter_binding,
        k_filter_exponents_after,
        normal_stage,
        activity_after,
        spatial_stage,
    )
}

struct IndependentMeterNormalStage {
    frontend: FrontendControlReplacementCandidate,
    source_map: Installed48SourceMap,
    normal_output: Option<NormalControlSnapshotOutput>,
    fresh_zero_controls: Option<NormalControlFreshZeroMeterView>,
    source_geometry: Option<GlobalSourceGeometrySnapshot>,
    source_projection: Option<PrimaryRegularSourceControlProjection>,
}

#[inline(never)]
fn stage_independent_meter_normal_source(
    prepared: &PreparedJobInput,
    source_view: &MeterSourceCallbackView<'_>,
    previous: &IndependentMeterG0Candidate,
    meter_binding: &MeterCallbackGtcBinding,
    providers: &IndependentMeterG0Providers,
) -> Result<Box<IndependentMeterNormalStage>, IndependentMeterG0Error> {
    let metadata_controls = prepared
        .primary_metadata_controls()
        .ok_or(IndependentMeterG0Error::PrimaryMetadataMissing)?;
    let profile = EncodingProfile::selected();
    let resolved_profile =
        resolve_selected_profile(&profile).map_err(|_| IndependentMeterG0Error::Provider)?;
    let source_map = build_meter_installed48_source_map(metadata_controls, &providers.p11_geometry)
        .map_err(|_| IndependentMeterG0Error::SourceMap)?;
    let callback = source_view.callback_index();
    let callback_index = FrontendCallbackIndex::new(
        u32::try_from(callback.value()).map_err(|_| IndependentMeterG0Error::UnsupportedClock)?,
    );
    let frontend_before = previous.frontend_candidate().next_machine();
    let frontend_state = frontend_before.committed();
    let current_prepared = prepared
        .normal_control_prepared_snapshot(frontend_state)
        .map_err(|_| IndependentMeterG0Error::FrontendControl)?;
    let normal_before =
        NormalControlBeforeState::from_frontend_state(frontend_state, frontend_before.generation())
            .map_err(|_| IndependentMeterG0Error::FrontendControlBeforeState)?;
    let lineage = CallbackLineageIdentity::new(
        prepared.programme_id().clone(),
        meter_binding.attempt(),
        callback_index,
    );
    let normal = propose_normal_control_snapshot(&normal_before, &current_prepared, lineage)
        .map_err(|_| IndependentMeterG0Error::FrontendControlProposal)?;
    let fresh_zero_controls =
        if callback.value().checked_add(1) == Some(source_view.normal_callbacks().value()) {
            let next_callback = u32::try_from(source_view.normal_callbacks().value())
                .map_err(|_| IndependentMeterG0Error::UnsupportedClock)?;
            let delivery = CallbackLineageIdentity::new(
                meter_binding.programme_id().clone(),
                meter_binding.attempt(),
                FrontendCallbackIndex::new(next_callback),
            );
            Some(
                normal
                    .after_state()
                    .held_source_view_for_meter_fresh_zero_drain(
                        delivery,
                        NormalCallbackCount::new(source_view.normal_callbacks().value()),
                    )
                    .map_err(|_| IndependentMeterG0Error::FrontendControl)?,
            )
        } else {
            None
        };
    let frontend_normal_stage = frontend_before
        .stage_normal_snapshot(normal)
        .map_err(|_| IndependentMeterG0Error::FrontendControlStage)?;
    let (frontend_staged, normal_output) = frontend_normal_stage.into_parts();
    let frontend = frontend_before
        .replacement_candidate(frontend_staged, meter_binding.transaction_id().clone())
        .map_err(|_| IndependentMeterG0Error::FrontendControlCandidate)?;
    let source_geometry = build_global_source_geometry_snapshot_from_output(
        prepared.graph().graph(),
        &source_map,
        &normal_output,
    )
    .map_err(|_| IndependentMeterG0Error::SourceMap)?;
    let source_projection = project_normal_control_snapshot_output_source_controls(
        &normal_output,
        prepared.graph().graph(),
        resolved_profile,
    )
    .map_err(IndependentMeterG0Error::SourceFlags)?;

    Ok(Box::new(IndependentMeterNormalStage {
        frontend,
        source_map,
        normal_output: Some(normal_output),
        fresh_zero_controls,
        source_geometry: Some(source_geometry),
        source_projection: Some(source_projection),
    }))
}

#[inline(never)]
fn stage_independent_meter_normal_features(
    prepared: &PreparedJobInput,
    source_view: &MeterSourceCallbackView<'_>,
    previous: &IndependentMeterG0Candidate,
    normal_stage: &mut IndependentMeterNormalStage,
    providers: &IndependentMeterG0Providers,
) -> Result<
    (
        RenderedFrontendFeatureStageProposal,
        PrimaryActivityState,
        SourceControlSnapshotClock,
    ),
    IndependentMeterG0Error,
> {
    let metadata_controls = prepared
        .primary_metadata_controls()
        .ok_or(IndependentMeterG0Error::PrimaryMetadataMissing)?;
    let source_projection = normal_stage
        .source_projection
        .take()
        .ok_or(IndependentMeterG0Error::SourceProjectionMissing)?;
    let source_control_clock = source_projection.clock();
    let spatial_control_before = previous.spatial_control_candidate().next_machine();
    let before_history =
        BeforeRegularHistory::from_owner(spatial_control_before, source_projection.route())
            .map_err(IndependentMeterG0Error::SourceFlags)?;
    let normal_output = normal_stage
        .normal_output
        .as_ref()
        .ok_or(IndependentMeterG0Error::NormalOutputMissing)?;
    let source_flags_parent = SourceFlagsParentLineage::new(
        normal_output.transaction_id().clone(),
        spatial_control_before,
    )
    .map_err(IndependentMeterG0Error::SourceFlags)?;
    let current_pcm = source_view
        .physical_rows()
        .iter()
        .map(|row| row.iter().copied().map(f32::from_bits).collect())
        .collect();
    let activity_pcm_rows = activity_pcm_rows_for_normal_callback(source_view)?;
    let activity = PrimaryInstalled48ActivityProducer::new_meter(&normal_stage.source_map)
        .map_err(IndependentMeterG0Error::Activity)?
        .produce_next(
            &normal_stage.source_map,
            previous.primary_activity_after(),
            PrimaryActivityInput::NormalRead {
                reader_relative_sample: AdmSampleIndex::new(
                    source_view.source_cursor_before().value(),
                ),
                pcm_word_rows: activity_pcm_rows,
            },
        )
        .map_err(IndependentMeterG0Error::Activity)?;
    let source_flags = SourceFlagsInput::from_primary_regular_source_controls(
        CurrentHostPcm::new(current_pcm).map_err(IndependentMeterG0Error::SourceFlags)?,
        Some(before_history),
        source_projection,
        PrefixWriterSet::new(activity.output().prefix_writer_indices().to_vec())
            .map_err(IndependentMeterG0Error::SourceFlags)?,
        source_flags_parent,
    )
    .map_err(IndependentMeterG0Error::SourceFlags)?;
    let installed48 = stage_meter_installed48(
        previous.installed48_after(),
        MeterInstalled48Input {
            source_sha256: *normal_stage
                .source_map
                .source_binding()
                .source_receipt_sha256(),
            callback_index: source_view.callback_index(),
            source_cursor_before: AdmSampleIndex::new(source_view.source_cursor_before().value()),
            host_input_samples: u16::try_from(source_view.host_input_count().value())
                .map_err(|_| IndependentMeterG0Error::MeterInstalled48)?,
            source_padding_samples: u16::try_from(source_view.source_padding_count().value())
                .map_err(|_| IndependentMeterG0Error::MeterInstalled48)?,
            raw_pcm_rows: source_view.physical_rows(),
            activity: activity.output(),
            membership_route: activity.membership_route(),
            source_map: &normal_stage.source_map,
            controls: metadata_controls,
            diffusion_parameters: Some(&providers.diffusion),
        },
    )
    .map_err(|_| IndependentMeterG0Error::MeterInstalled48)?;
    let mut rendered_frontend = stage_frontend_features_from_installed48(
        spatial_control_before,
        normal_output.transaction_id().clone(),
        activity,
        installed48,
        source_flags,
        CallbackPhase::Normal,
        &providers.feature_processor,
        &providers.feature_fft,
    )
    .map_err(|_| IndependentMeterG0Error::FrontendFeatures)?;
    let activity_after = rendered_frontend
        .take_primary_activity_after_state()
        .ok_or(IndependentMeterG0Error::ActivityAfterStateMissing)?;
    Ok((rendered_frontend, activity_after, source_control_clock))
}

#[inline(never)]
fn continue_independent_meter_normal_spatial(
    previous: &IndependentMeterG0Candidate,
    normal_stage: &mut IndependentMeterNormalStage,
    rendered_frontend: RenderedFrontendFeatureStageProposal,
    source_control_clock: SourceControlSnapshotClock,
    providers: &IndependentMeterG0Providers,
) -> Result<MeterSpatialProductionCallbackProposal, IndependentMeterG0Error> {
    let normal_output = normal_stage
        .normal_output
        .take()
        .ok_or(IndependentMeterG0Error::NormalOutputMissing)?;
    let source_geometry = normal_stage
        .source_geometry
        .take()
        .ok_or(IndependentMeterG0Error::SourceGeometryMissing)?;
    continue_meter_spatial_production_callback(MeterSpatialProductionCallbackInputs {
        frontend: rendered_frontend,
        source_snapshot: NormalControlCallbackSource::NormalOutput(Box::new(normal_output)),
        source_geometry,
        source_control_clock,
        source_map: &normal_stage.source_map,
        p15_geometry: &providers.p15_geometry,
        spatial_control: previous.spatial_control_candidate().next_machine(),
        spatial_audio: previous.spatial_audio_candidate().next_machine(),
        window: &providers.window,
        parameters: &providers.component_parameters,
    })
    .map_err(IndependentMeterG0Error::SpatialProduction)
}

#[inline(never)]
fn finish_independent_meter_normal_callback(
    source_view: MeterSourceCallbackView<'_>,
    previous: &IndependentMeterG0Candidate,
    meter_binding: MeterCallbackGtcBinding,
    k_filter_exponents_after: MeterKFilterExponentState,
    normal_stage: Box<IndependentMeterNormalStage>,
    activity_after: PrimaryActivityState,
    spatial_stage: MeterSpatialProductionCallbackProposal,
) -> Result<IndependentMeterG0Candidate, IndependentMeterG0Error> {
    let callback_index = FrontendCallbackIndex::new(
        u32::try_from(source_view.callback_index().value())
            .map_err(|_| IndependentMeterG0Error::UnsupportedClock)?,
    );
    if spatial_stage.callback_index() != callback_index
        || spatial_stage.component_rows().callback_index() != callback_index
        || spatial_stage.reader_target_controls().callback_index() != callback_index
    {
        return Err(IndependentMeterG0Error::UnsupportedClock);
    }
    let handoff = source_view
        .bind_spatial_outputs(
            spatial_stage.component_rows(),
            spatial_stage.reader_target_controls(),
        )
        .map_err(IndependentMeterG0Error::SourceBridge)?;
    let (meter_input, source_receipt) = handoff.into_parts();
    let progress = MeterRootCallbackProgress::new(
        source_receipt.callback_index(),
        MeterSourcePosition::new(source_receipt.source_cursor_before().value()),
        MeterSourcePosition::new(source_receipt.source_cursor_after().value()),
        source_receipt.source_read_index_before(),
        source_receipt.source_read_index_after(),
        source_receipt.host_input_count(),
        k_filter_exponents_after,
    );
    let transaction_id = meter_binding.transaction_id().clone();
    let meter = DomainMeterCallbackGtcProposal::stage(
        previous.meter_proposal().after_root(),
        meter_input,
        progress,
        meter_binding,
    )
    .map_err(IndependentMeterG0Error::MeterGtc)?;
    let component_rows = spatial_stage.component_rows().clone();
    let reader_target_controls = spatial_stage.reader_target_controls().clone();
    let spatial_parts = spatial_stage.into_parent_parts();

    let spatial_control_before = previous.spatial_control_candidate().next_machine();
    let spatial_control_staged = spatial_control_before
        .stage(
            spatial_control_before
                .prepare(spatial_parts.spatial_control_after)
                .map_err(|_| IndependentMeterG0Error::SpatialControl)?,
        )
        .map_err(|_| IndependentMeterG0Error::SpatialControl)?;
    let spatial_control = spatial_control_before
        .replacement_candidate(spatial_control_staged, transaction_id.clone())
        .map_err(|_| IndependentMeterG0Error::SpatialControl)?;

    let spatial_audio_before = previous.spatial_audio_candidate().next_machine();
    let spatial_audio_staged = spatial_audio_before
        .stage(
            spatial_audio_before
                .prepare(spatial_parts.spatial_audio_after)
                .map_err(|_| IndependentMeterG0Error::SpatialAudio)?,
        )
        .map_err(|_| IndependentMeterG0Error::SpatialAudio)?;
    let spatial_audio = spatial_audio_before
        .replacement_candidate(spatial_audio_staged, transaction_id)
        .map_err(|_| IndependentMeterG0Error::SpatialAudio)?;

    Ok(IndependentMeterG0Candidate {
        frontend: normal_stage.frontend,
        primary_activity_after: activity_after,
        installed48_after: spatial_parts.installed48_after,
        spatial_control,
        spatial_audio,
        component_rows,
        reader_target_controls,
        fresh_zero_controls: normal_stage.fresh_zero_controls,
        source_receipt,
        meter,
    })
}

/// Advances one independent fresh-zero Meter delivery from the previous callback candidate.
///
/// The prior proposal supplies every after-root. This stage never recreates a cold frontend,
/// activity, Installed48, spatial-control, spatial-audio, or Meter root.
pub fn stage_independent_meter_fresh_zero_drain_callback(
    prepared: &PreparedJobInput,
    source_view: MeterSourceCallbackView<'_>,
    previous: IndependentMeterG0Candidate,
    meter_binding: MeterCallbackGtcBinding,
    k_filter_exponents_after: MeterKFilterExponentState,
    providers: &IndependentMeterG0Providers,
) -> Result<IndependentMeterG0Candidate, IndependentMeterG0Error> {
    validate_fresh_zero_callback_identity(prepared, &source_view, &previous, &meter_binding)?;
    let metadata_controls = prepared
        .primary_metadata_controls()
        .ok_or(IndependentMeterG0Error::PrimaryMetadataMissing)?;
    let source_map = build_meter_installed48_source_map(metadata_controls, &providers.p11_geometry)
        .map_err(|_| IndependentMeterG0Error::SourceMap)?;
    let spatial_control_before = previous.spatial_control_candidate().next_machine();
    let spatial_audio_before = previous.spatial_audio_candidate().next_machine();
    let (rendered_frontend, activity_after, source_control_clock, source_geometry, held_controls) =
        stage_independent_meter_fresh_zero_features(
            prepared,
            &source_view,
            &previous,
            &source_map,
            providers,
        )?;
    let spatial_stage =
        continue_meter_spatial_production_callback(MeterSpatialProductionCallbackInputs {
            frontend: rendered_frontend,
            source_snapshot: NormalControlCallbackSource::FreshZeroMeterHold(Box::new(
                held_controls,
            )),
            source_geometry,
            source_control_clock,
            source_map: &source_map,
            p15_geometry: &providers.p15_geometry,
            spatial_control: spatial_control_before,
            spatial_audio: spatial_audio_before,
            window: &providers.window,
            parameters: &providers.component_parameters,
        })
        .map_err(IndependentMeterG0Error::SpatialProduction)?;

    let handoff = source_view
        .bind_spatial_outputs(
            spatial_stage.component_rows(),
            spatial_stage.reader_target_controls(),
        )
        .map_err(IndependentMeterG0Error::SourceBridge)?;
    let (meter_input, source_receipt) = handoff.into_parts();
    let meter_progress = MeterRootCallbackProgress::new(
        source_receipt.callback_index(),
        MeterSourcePosition::new(source_receipt.source_cursor_before().value()),
        MeterSourcePosition::new(source_receipt.source_cursor_after().value()),
        source_receipt.source_read_index_before(),
        source_receipt.source_read_index_after(),
        source_receipt.host_input_count(),
        k_filter_exponents_after,
    );
    let parent_transaction_id = meter_binding.transaction_id().clone();
    let meter = DomainMeterCallbackGtcProposal::stage(
        previous.meter_proposal().after_root(),
        meter_input,
        meter_progress,
        meter_binding,
    )
    .map_err(IndependentMeterG0Error::MeterGtc)?;

    let component_rows = spatial_stage.component_rows().clone();
    let reader_target_controls = spatial_stage.reader_target_controls().clone();
    let spatial_parts = spatial_stage.into_parent_parts();
    let spatial_control_staged = spatial_control_before
        .stage(
            spatial_control_before
                .prepare(spatial_parts.spatial_control_after)
                .map_err(|_| IndependentMeterG0Error::SpatialControl)?,
        )
        .map_err(|_| IndependentMeterG0Error::SpatialControl)?;
    let spatial_control = spatial_control_before
        .replacement_candidate(spatial_control_staged, parent_transaction_id.clone())
        .map_err(|_| IndependentMeterG0Error::SpatialControl)?;
    let spatial_audio_staged = spatial_audio_before
        .stage(
            spatial_audio_before
                .prepare(spatial_parts.spatial_audio_after)
                .map_err(|_| IndependentMeterG0Error::SpatialAudio)?,
        )
        .map_err(|_| IndependentMeterG0Error::SpatialAudio)?;
    let spatial_audio = spatial_audio_before
        .replacement_candidate(spatial_audio_staged, parent_transaction_id)
        .map_err(|_| IndependentMeterG0Error::SpatialAudio)?;

    Ok(IndependentMeterG0Candidate {
        frontend: previous.frontend,
        primary_activity_after: activity_after,
        installed48_after: spatial_parts.installed48_after,
        spatial_control,
        spatial_audio,
        component_rows,
        reader_target_controls,
        fresh_zero_controls: previous.fresh_zero_controls,
        source_receipt,
        meter,
    })
}

#[inline(never)]
fn stage_independent_meter_fresh_zero_features(
    prepared: &PreparedJobInput,
    source_view: &MeterSourceCallbackView<'_>,
    previous: &IndependentMeterG0Candidate,
    source_map: &Installed48SourceMap,
    providers: &IndependentMeterG0Providers,
) -> Result<
    (
        RenderedFrontendFeatureStageProposal,
        PrimaryActivityState,
        SourceControlSnapshotClock,
        GlobalSourceGeometrySnapshot,
        NormalControlFreshZeroMeterView,
    ),
    IndependentMeterG0Error,
> {
    let base_controls = previous
        .fresh_zero_controls()
        .ok_or(IndependentMeterG0Error::FreshZeroControlsMissing)?;
    let callback = source_view.callback_index();
    let callback_index = FrontendCallbackIndex::new(
        u32::try_from(callback.value()).map_err(|_| IndependentMeterG0Error::UnsupportedClock)?,
    );
    let delivery = CallbackLineageIdentity::new(
        prepared.programme_id().clone(),
        previous.meter_proposal().after_root().identity().attempt(),
        callback_index,
    );
    let held_controls = base_controls
        .for_delivery(delivery)
        .map_err(|_| IndependentMeterG0Error::FrontendControl)?;
    let profile = EncodingProfile::selected();
    let resolved_profile =
        resolve_selected_profile(&profile).map_err(|_| IndependentMeterG0Error::Provider)?;
    let source_projection = project_meter_fresh_zero_normal_control_snapshot_source_controls(
        &held_controls,
        prepared.graph().graph(),
        resolved_profile,
    )
    .map_err(IndependentMeterG0Error::SourceFlags)?;
    let source_control_clock = source_projection.clock();
    let spatial_control_before = previous.spatial_control_candidate().next_machine();
    let before_history =
        BeforeRegularHistory::from_owner(spatial_control_before, source_projection.route())
            .map_err(IndependentMeterG0Error::SourceFlags)?;
    let source_flags_parent = SourceFlagsParentLineage::new(
        held_controls.delivery_transaction_id().clone(),
        spatial_control_before,
    )
    .map_err(IndependentMeterG0Error::SourceFlags)?;
    let zero_pcm = source_view
        .physical_rows()
        .iter()
        .map(|row| row.iter().copied().map(f32::from_bits).collect())
        .collect();
    let activity_producer = PrimaryInstalled48ActivityProducer::new_meter(source_map)
        .map_err(IndependentMeterG0Error::Activity)?;
    let activity = activity_producer
        .produce_next(
            source_map,
            previous.primary_activity_after(),
            PrimaryActivityInput::FreshZeroInputDrain {
                pcm_word_rows: source_view.physical_rows().to_vec(),
            },
        )
        .map_err(IndependentMeterG0Error::Activity)?;
    let source_flags = SourceFlagsInput::from_primary_regular_source_controls(
        CurrentHostPcm::new(zero_pcm).map_err(IndependentMeterG0Error::SourceFlags)?,
        Some(before_history),
        source_projection,
        PrefixWriterSet::new(activity.output().prefix_writer_indices().to_vec())
            .map_err(IndependentMeterG0Error::SourceFlags)?,
        source_flags_parent,
    )
    .map_err(IndependentMeterG0Error::SourceFlags)?;

    let metadata_controls = prepared
        .primary_metadata_controls()
        .ok_or(IndependentMeterG0Error::PrimaryMetadataMissing)?;
    let installed48 = stage_meter_installed48(
        previous.installed48_after(),
        MeterInstalled48Input {
            source_sha256: *source_map.source_binding().source_receipt_sha256(),
            callback_index: MeterCallbackIndex::new(callback.value()),
            source_cursor_before: AdmSampleIndex::new(source_view.source_cursor_before().value()),
            host_input_samples: 0,
            source_padding_samples: 0,
            raw_pcm_rows: source_view.physical_rows(),
            activity: activity.output(),
            membership_route: activity.membership_route(),
            source_map,
            controls: metadata_controls,
            diffusion_parameters: Some(&providers.diffusion),
        },
    )
    .map_err(|_| IndependentMeterG0Error::MeterInstalled48)?;
    let rendered_frontend = stage_meter_frontend_features_from_fresh_zero_input(
        spatial_control_before,
        held_controls.delivery_transaction_id().clone(),
        activity,
        installed48,
        source_flags,
        &providers.feature_processor,
        &providers.feature_fft,
    )
    .map_err(|_| IndependentMeterG0Error::FrontendFeatures)?;
    let mut rendered_frontend = rendered_frontend;
    let primary_activity_after = rendered_frontend
        .take_primary_activity_after_state()
        .ok_or(IndependentMeterG0Error::ActivityAfterStateMissing)?;
    let source_geometry = build_meter_fresh_zero_global_source_geometry_snapshot(
        prepared.graph().graph(),
        source_map,
        &held_controls,
    )
    .map_err(|_| IndependentMeterG0Error::SourceGeometryMissing)?;
    Ok((
        rendered_frontend,
        primary_activity_after,
        source_control_clock,
        source_geometry,
        held_controls,
    ))
}

#[inline(never)]
fn stage_independent_meter_g0_normal(
    prepared: &PreparedJobInput,
    meter_binding: &MeterCallbackGtcBinding,
    providers: &IndependentMeterG0Providers,
) -> Result<Box<IndependentMeterG0NormalStage>, IndependentMeterG0Error> {
    let profile = EncodingProfile::selected();
    let resolved_profile =
        resolve_selected_profile(&profile).map_err(|_| IndependentMeterG0Error::Provider)?;
    let metadata_controls = prepared
        .primary_metadata_controls()
        .ok_or(IndependentMeterG0Error::PrimaryMetadataMissing)?;
    let source_map = build_meter_installed48_source_map(metadata_controls, &providers.p11_geometry)
        .map_err(|_| IndependentMeterG0Error::SourceMap)?;

    let programme_id = prepared.programme_id().clone();
    let attempt = meter_binding.attempt();
    let meter_frontend = FrontendControlMachine::from_provider(
        providers.f01e.clone(),
        programme_id.clone(),
        attempt,
    )
    .map_err(|_| IndependentMeterG0Error::FrontendControl)?;
    let meter_spatial_control =
        SpatialControlMachine::new_cold(profile, &providers.f01e, &providers.f03_target)
            .map_err(|_| IndependentMeterG0Error::SpatialControl)?;
    let meter_spatial_audio = SpatialAudioMachine::new_cold(profile)
        .map_err(|_| IndependentMeterG0Error::SpatialAudio)?;
    if meter_frontend.generation() != 0
        || meter_frontend.committed().callback_index().value() != 0
        || meter_spatial_control.generation() != 0
        || meter_spatial_control.committed().callback_index().value() != 0
        || meter_spatial_audio.generation() != 0
        || meter_spatial_audio.committed().callback_index().value() != 0
    {
        return Err(IndependentMeterG0Error::UnsupportedClock);
    }

    let lineage =
        CallbackLineageIdentity::new(programme_id, attempt, FrontendCallbackIndex::new(0));
    let snapshot = prepared
        .normal_control_prepared_snapshot(meter_frontend.committed())
        .map_err(|_| IndependentMeterG0Error::FrontendControl)?;
    let before = NormalControlBeforeState::from_frontend_state(
        meter_frontend.committed(),
        meter_frontend.generation(),
    )
    .map_err(|_| IndependentMeterG0Error::FrontendControl)?;
    let normal = propose_normal_control_snapshot(&before, &snapshot, lineage)
        .map_err(|_| IndependentMeterG0Error::FrontendControl)?;
    let normal_callbacks = prepared.configuration().source_samples().div_ceil(1_536);
    let fresh_zero_controls = if normal_callbacks == 1 {
        let delivery = CallbackLineageIdentity::new(
            meter_binding.programme_id().clone(),
            meter_binding.attempt(),
            FrontendCallbackIndex::new(1),
        );
        Some(
            normal
                .after_state()
                .held_source_view_for_meter_fresh_zero_drain(
                    delivery,
                    NormalCallbackCount::new(normal_callbacks),
                )
                .map_err(|_| IndependentMeterG0Error::FrontendControl)?,
        )
    } else {
        None
    };
    let frontend_stage = meter_frontend
        .stage_normal_snapshot(normal)
        .map_err(|_| IndependentMeterG0Error::FrontendControl)?;
    let (frontend_staged, normal_output) = frontend_stage.into_parts();
    let frontend = meter_frontend
        .replacement_candidate(frontend_staged, meter_binding.transaction_id().clone())
        .map_err(|_| IndependentMeterG0Error::FrontendControl)?;
    let source_geometry = build_global_source_geometry_snapshot_from_output(
        prepared.graph().graph(),
        &source_map,
        &normal_output,
    )
    .map_err(|_| IndependentMeterG0Error::SourceMap)?;
    let source_projection = project_normal_control_snapshot_output_source_controls(
        &normal_output,
        prepared.graph().graph(),
        resolved_profile,
    )
    .map_err(IndependentMeterG0Error::SourceFlags)?;

    Ok(Box::new(IndependentMeterG0NormalStage {
        frontend,
        spatial_control: meter_spatial_control,
        spatial_audio: meter_spatial_audio,
        source_map,
        normal_output: Some(normal_output),
        fresh_zero_controls,
        source_geometry: Some(source_geometry),
        source_projection: Some(source_projection),
    }))
}

#[inline(never)]
fn stage_independent_meter_g0_features(
    prepared: &PreparedJobInput,
    source_view: &MeterSourceCallbackView<'_>,
    normal_stage: &mut IndependentMeterG0NormalStage,
    providers: &IndependentMeterG0Providers,
) -> Result<
    (
        RenderedFrontendFeatureStageProposal,
        PrimaryActivityState,
        SourceControlSnapshotClock,
    ),
    IndependentMeterG0Error,
> {
    let metadata_controls = prepared
        .primary_metadata_controls()
        .ok_or(IndependentMeterG0Error::PrimaryMetadataMissing)?;
    let source_projection = normal_stage
        .source_projection
        .take()
        .ok_or(IndependentMeterG0Error::SourceProjectionMissing)?;
    let source_control_clock = source_projection.clock();
    let before_history =
        BeforeRegularHistory::from_owner(&normal_stage.spatial_control, source_projection.route())
            .map_err(IndependentMeterG0Error::SourceFlags)?;
    let current_pcm = source_view
        .physical_rows()
        .iter()
        .map(|row| row.iter().copied().map(f32::from_bits).collect())
        .collect();
    let normal_output = normal_stage
        .normal_output
        .as_ref()
        .ok_or(IndependentMeterG0Error::NormalOutputMissing)?;
    let source_flags_parent = SourceFlagsParentLineage::new(
        normal_output.transaction_id().clone(),
        &normal_stage.spatial_control,
    )
    .map_err(IndependentMeterG0Error::SourceFlags)?;

    let source_samples = MeterSourceSampleCount::new(prepared.configuration().source_samples())
        .map_err(|_| IndependentMeterG0Error::PreparedIdentityMismatch)?;
    let source_binding = normal_stage.source_map.source_binding();
    let meter_installed48_before =
        MeterInstalled48State::cold(*source_binding.source_receipt_sha256(), source_samples);
    let activity_producer = PrimaryInstalled48ActivityProducer::new_meter(&normal_stage.source_map)
        .map_err(IndependentMeterG0Error::Activity)?;
    let activity_before = activity_producer
        .cold_state()
        .map_err(IndependentMeterG0Error::Activity)?;
    let activity_pcm_rows = activity_pcm_rows_for_normal_callback(source_view)?;
    let activity = activity_producer
        .produce_next(
            &normal_stage.source_map,
            &activity_before,
            PrimaryActivityInput::NormalRead {
                reader_relative_sample: AdmSampleIndex::new(
                    source_view.source_cursor_before().value(),
                ),
                pcm_word_rows: activity_pcm_rows,
            },
        )
        .map_err(IndependentMeterG0Error::Activity)?;
    let source_flags = SourceFlagsInput::from_primary_regular_source_controls(
        CurrentHostPcm::new(current_pcm).map_err(IndependentMeterG0Error::SourceFlags)?,
        Some(before_history),
        source_projection,
        PrefixWriterSet::new(activity.output().prefix_writer_indices().to_vec())
            .map_err(IndependentMeterG0Error::SourceFlags)?,
        source_flags_parent,
    )
    .map_err(IndependentMeterG0Error::SourceFlags)?;
    let meter_installed48 = stage_meter_installed48(
        &meter_installed48_before,
        MeterInstalled48Input {
            source_sha256: *source_binding.source_receipt_sha256(),
            callback_index: MeterCallbackIndex::new(0),
            source_cursor_before: AdmSampleIndex::new(source_view.source_cursor_before().value()),
            host_input_samples: u16::try_from(source_view.host_input_count().value())
                .map_err(|_| IndependentMeterG0Error::MeterInstalled48)?,
            source_padding_samples: u16::try_from(source_view.source_padding_count().value())
                .map_err(|_| IndependentMeterG0Error::MeterInstalled48)?,
            raw_pcm_rows: source_view.physical_rows(),
            activity: activity.output(),
            membership_route: activity.membership_route(),
            source_map: &normal_stage.source_map,
            controls: metadata_controls,
            diffusion_parameters: Some(&providers.diffusion),
        },
    )
    .map_err(|_| IndependentMeterG0Error::MeterInstalled48)?;
    let mut rendered_frontend = stage_frontend_features_from_installed48(
        &normal_stage.spatial_control,
        normal_output.transaction_id().clone(),
        activity,
        meter_installed48,
        source_flags,
        CallbackPhase::Normal,
        &providers.feature_processor,
        &providers.feature_fft,
    )
    .map_err(|_| IndependentMeterG0Error::FrontendFeatures)?;
    let primary_activity_after = rendered_frontend
        .take_primary_activity_after_state()
        .ok_or(IndependentMeterG0Error::ActivityAfterStateMissing)?;
    Ok((
        rendered_frontend,
        primary_activity_after,
        source_control_clock,
    ))
}

#[inline(never)]
fn continue_independent_meter_g0_spatial(
    normal_stage: &mut IndependentMeterG0NormalStage,
    rendered_frontend: RenderedFrontendFeatureStageProposal,
    source_control_clock: SourceControlSnapshotClock,
    providers: &IndependentMeterG0Providers,
) -> Result<MeterSpatialProductionCallbackProposal, IndependentMeterG0Error> {
    let normal_output = normal_stage
        .normal_output
        .take()
        .ok_or(IndependentMeterG0Error::NormalOutputMissing)?;
    let source_geometry = normal_stage
        .source_geometry
        .take()
        .ok_or(IndependentMeterG0Error::SourceGeometryMissing)?;
    continue_meter_spatial_production_callback(MeterSpatialProductionCallbackInputs {
        frontend: rendered_frontend,
        source_snapshot: NormalControlCallbackSource::NormalOutput(Box::new(normal_output)),
        source_geometry,
        source_control_clock,
        source_map: &normal_stage.source_map,
        p15_geometry: &providers.p15_geometry,
        spatial_control: &normal_stage.spatial_control,
        spatial_audio: &normal_stage.spatial_audio,
        window: &providers.window,
        parameters: &providers.component_parameters,
    })
    .map_err(IndependentMeterG0Error::SpatialProduction)
}

#[inline(never)]
fn finish_independent_meter_g0_callback(
    source_view: MeterSourceCallbackView<'_>,
    meter_before: &StandaloneMeterRoot,
    meter_binding: MeterCallbackGtcBinding,
    k_filter_exponents_after: MeterKFilterExponentState,
    normal_stage: Box<IndependentMeterG0NormalStage>,
    primary_activity_after: PrimaryActivityState,
    spatial_stage: MeterSpatialProductionCallbackProposal,
) -> Result<IndependentMeterG0Candidate, IndependentMeterG0Error> {
    if spatial_stage.callback_index() != FrontendCallbackIndex::new(0)
        || spatial_stage.component_rows().callback_index() != FrontendCallbackIndex::new(0)
        || spatial_stage.reader_target_controls().callback_index() != FrontendCallbackIndex::new(0)
    {
        return Err(IndependentMeterG0Error::UnsupportedClock);
    }
    let handoff = source_view
        .bind_spatial_outputs(
            spatial_stage.component_rows(),
            spatial_stage.reader_target_controls(),
        )
        .map_err(IndependentMeterG0Error::SourceBridge)?;
    let (meter_input, source_receipt) = handoff.into_parts();
    let meter_progress = MeterRootCallbackProgress::new(
        source_receipt.callback_index(),
        MeterSourcePosition::new(source_receipt.source_cursor_before().value()),
        MeterSourcePosition::new(source_receipt.source_cursor_after().value()),
        source_receipt.source_read_index_before(),
        source_receipt.source_read_index_after(),
        source_receipt.host_input_count(),
        k_filter_exponents_after,
    );
    let parent_transaction_id = meter_binding.transaction_id().clone();
    let meter = DomainMeterCallbackGtcProposal::stage(
        meter_before,
        meter_input,
        meter_progress,
        meter_binding,
    )
    .map_err(IndependentMeterG0Error::MeterGtc)?;

    let component_rows = spatial_stage.component_rows().clone();
    let reader_target_controls = spatial_stage.reader_target_controls().clone();
    let spatial_parts = spatial_stage.into_parent_parts();
    let spatial_control_staged = normal_stage
        .spatial_control
        .stage(
            normal_stage
                .spatial_control
                .prepare(spatial_parts.spatial_control_after)
                .map_err(|_| IndependentMeterG0Error::SpatialControl)?,
        )
        .map_err(|_| IndependentMeterG0Error::SpatialControl)?;
    let spatial_control = normal_stage
        .spatial_control
        .replacement_candidate(spatial_control_staged, parent_transaction_id.clone())
        .map_err(|_| IndependentMeterG0Error::SpatialControl)?;
    let spatial_audio_staged = normal_stage
        .spatial_audio
        .stage(
            normal_stage
                .spatial_audio
                .prepare(spatial_parts.spatial_audio_after)
                .map_err(|_| IndependentMeterG0Error::SpatialAudio)?,
        )
        .map_err(|_| IndependentMeterG0Error::SpatialAudio)?;
    let spatial_audio = normal_stage
        .spatial_audio
        .replacement_candidate(spatial_audio_staged, parent_transaction_id)
        .map_err(|_| IndependentMeterG0Error::SpatialAudio)?;

    Ok(IndependentMeterG0Candidate {
        frontend: normal_stage.frontend,
        primary_activity_after,
        installed48_after: spatial_parts.installed48_after,
        spatial_control,
        spatial_audio,
        fresh_zero_controls: normal_stage.fresh_zero_controls,
        component_rows,
        reader_target_controls,
        source_receipt,
        meter,
    })
}

fn validate_callback_identity(
    prepared: &PreparedJobInput,
    source_view: &MeterSourceCallbackView<'_>,
    meter_root: &StandaloneMeterRoot,
    binding: &MeterCallbackGtcBinding,
) -> Result<(), IndependentMeterG0Error> {
    let input_sha256 = prepared.resource_identity().input_sha256();
    let source_samples = prepared.configuration().source_samples();
    let meter_identity = meter_root.identity();
    if source_view.callback_index() != MeterCallbackIndex::new(0)
        || source_view.phase() != MeterCallbackPhase::NormalSource
        || source_view.source_cursor_before().value() != 0
        || source_view.source_read_index_before().value() != 0
        || source_view.source_read_index_after().value() != 1
        || source_view.host_input_count().value() != 1_536
        || source_view.source_padding_count().value() != 0
        || source_view.source_samples().value() != source_samples
        || source_view.source_identity().sha256() != input_sha256
        || parsed_identity_matches(meter_identity, prepared, binding).is_err()
        || meter_root.source_samples().value() != source_samples
        || meter_root.meter_callback_index() != MeterCallbackIndex::new(0)
    {
        return Err(IndependentMeterG0Error::PreparedIdentityMismatch);
    }
    Ok(())
}

fn activity_pcm_rows_for_normal_callback(
    source_view: &MeterSourceCallbackView<'_>,
) -> Result<Vec<Vec<u32>>, IndependentMeterG0Error> {
    let actual_samples = usize::try_from(source_view.host_input_count().value())
        .map_err(|_| IndependentMeterG0Error::Activity(PrimaryActivityError::ReaderFrameShape))?;
    if actual_samples == 0 {
        return Err(IndependentMeterG0Error::Activity(
            PrimaryActivityError::ReaderFrameShape,
        ));
    }
    source_view
        .physical_rows()
        .iter()
        .map(|row| {
            row.get(..actual_samples)
                .map(<[u32]>::to_vec)
                .ok_or(IndependentMeterG0Error::Activity(
                    PrimaryActivityError::ReaderFrameShape,
                ))
        })
        .collect()
}

fn validate_normal_callback_identity(
    prepared: &PreparedJobInput,
    source_view: &MeterSourceCallbackView<'_>,
    previous: &IndependentMeterG0Candidate,
    binding: &MeterCallbackGtcBinding,
) -> Result<(), IndependentMeterG0Error> {
    let meter_before = previous.meter_proposal().after_root();
    let identity = meter_before.identity();
    let callback = source_view.callback_index();
    let callback_value = callback.value();
    let next_read_index = source_view
        .source_read_index_before()
        .value()
        .checked_add(1)
        .ok_or(IndependentMeterG0Error::UnsupportedClock)?;
    let source_cursor_before = source_view.source_cursor_before().value();
    let source_samples = prepared.configuration().source_samples();
    let expected_host_input = source_samples
        .checked_sub(source_cursor_before)
        .ok_or(IndependentMeterG0Error::UnsupportedClock)?
        .min(1_536);
    let expected_source_after = source_cursor_before
        .checked_add(expected_host_input)
        .ok_or(IndependentMeterG0Error::UnsupportedClock)?;
    let frontend_callback = FrontendCallbackIndex::new(
        u32::try_from(callback_value).map_err(|_| IndependentMeterG0Error::UnsupportedClock)?,
    );
    if callback_value == 0
        || callback_value >= source_view.normal_callbacks().value()
        || source_view.phase() != MeterCallbackPhase::NormalSource
        || binding.callback_index() != callback
        || binding.programme_id() != prepared.programme_id()
        || binding.attempt() != identity.attempt()
        || identity.programme_id() != prepared.programme_id()
        || identity.input_sha256() != &prepared.resource_identity().input_sha256()
        || identity.config_sha256() != &prepared.config_sha256()
        || meter_before.source_samples().value() != source_samples
        || meter_before.normal_callback_count().value() != source_view.normal_callbacks().value()
        || meter_before.meter_callback_index() != callback
        || meter_before.source_cursor().value() != source_cursor_before
        || meter_before.source_read_index() != source_view.source_read_index_before()
        || source_view.source_cursor_after().value() != expected_source_after
        || source_view.source_read_index_before().value() != callback_value
        || source_view.source_read_index_after().value() != next_read_index
        || source_view.host_input_count().value() != expected_host_input
        || source_view.source_padding_count().value() != 1_536 - expected_host_input
        || source_view.source_samples().value() != source_samples
        || source_view.source_identity().sha256() != prepared.resource_identity().input_sha256()
        || previous.frontend_candidate().next_machine().generation() != callback_value
        || previous
            .frontend_candidate()
            .next_machine()
            .committed()
            .callback_index()
            != frontend_callback
        || previous.primary_activity_after().next_callback() != frontend_callback
        || previous.installed48_after().callback_index() != callback
        || previous.installed48_after().source_cursor().value() != source_cursor_before
        || u64::from(
            previous
                .spatial_control_candidate()
                .next_machine()
                .committed()
                .callback_index()
                .value(),
        ) != callback_value
        || previous
            .spatial_audio_candidate()
            .next_machine()
            .generation()
            != callback_value
    {
        return Err(IndependentMeterG0Error::PreparedIdentityMismatch);
    }
    Ok(())
}

fn validate_fresh_zero_callback_identity(
    prepared: &PreparedJobInput,
    source_view: &MeterSourceCallbackView<'_>,
    previous: &IndependentMeterG0Candidate,
    binding: &MeterCallbackGtcBinding,
) -> Result<(), IndependentMeterG0Error> {
    let meter_before = previous.meter_proposal().after_root();
    let identity = meter_before.identity();
    let callback = source_view.callback_index().value();
    let normal_callbacks = source_view.normal_callbacks().value();
    let callback_end = normal_callbacks
        .checked_add(3)
        .ok_or(IndependentMeterG0Error::UnsupportedClock)?;
    if source_view.phase() != MeterCallbackPhase::ZeroInputDrain
        || callback < normal_callbacks
        || callback >= callback_end
        || binding.callback_index() != source_view.callback_index()
        || binding.programme_id() != prepared.programme_id()
        || binding.attempt() != identity.attempt()
        || identity.programme_id() != prepared.programme_id()
        || identity.input_sha256() != &prepared.resource_identity().input_sha256()
        || identity.config_sha256() != &prepared.config_sha256()
        || meter_before.source_samples().value() != prepared.configuration().source_samples()
        || meter_before.normal_callback_count().value() != normal_callbacks
        || meter_before.meter_callback_index() != source_view.callback_index()
        || meter_before.source_cursor().value() != prepared.configuration().source_samples()
        || meter_before.source_read_index() != source_view.source_read_index_before()
        || source_view.source_cursor_before().value() != prepared.configuration().source_samples()
        || source_view.source_cursor_after() != source_view.source_cursor_before()
        || source_view.source_read_index_after() != source_view.source_read_index_before()
        || source_view.source_read_index_before().value() != normal_callbacks
        || source_view.host_input_count().value() != 0
        || source_view.source_padding_count().value() != 1_536
        || source_view.source_samples().value() != prepared.configuration().source_samples()
        || source_view.source_identity().sha256() != prepared.resource_identity().input_sha256()
        || source_view.physical_rows().len() != 38
        || source_view
            .physical_rows()
            .iter()
            .any(|row| row.len() != 1_536 || row.iter().any(|word| *word != 0))
        || u64::from(
            previous
                .spatial_control_candidate()
                .next_machine()
                .committed()
                .callback_index()
                .value(),
        ) != callback
        || previous
            .spatial_audio_candidate()
            .next_machine()
            .generation()
            != callback
        || u64::from(previous.primary_activity_after().next_callback().value()) != callback
        || previous.installed48_after().callback_index().value() != callback
        || previous.installed48_after().source_cursor().value()
            != prepared.configuration().source_samples()
        || previous
            .fresh_zero_controls()
            .is_none_or(|held| held.normal_callbacks().value() != normal_callbacks)
    {
        return Err(IndependentMeterG0Error::UnsupportedClock);
    }
    Ok(())
}

fn parsed_identity_matches(
    meter_identity: &jocforge_domain::meter::standalone_root::StandaloneMeterIdentity,
    prepared: &PreparedJobInput,
    binding: &MeterCallbackGtcBinding,
) -> Result<(), ()> {
    if meter_identity.programme_id() != prepared.programme_id()
        || meter_identity.programme_id() != binding.programme_id()
        || meter_identity.attempt() != binding.attempt()
        || meter_identity.input_sha256() != &prepared.resource_identity().input_sha256()
        || meter_identity.config_sha256() != &prepared.config_sha256()
        || binding.callback_index() != MeterCallbackIndex::new(0)
    {
        return Err(());
    }
    Ok(())
}
