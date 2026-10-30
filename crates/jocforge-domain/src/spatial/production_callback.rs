// pattern: Functional Core
//! Noncommitting F03W continuation from the current frontend feature proposal.

pub mod selected_audio_provider_bundle;
use self::selected_audio_provider_bundle::SelectedSpatialAudioProviderBundle;

use super::{
    ValidatedComponentParameterProvider, ValidatedWindowQ15Provider,
    candidate_selector::{
        CandidateSelectorInput, CandidateSelectorResult, SelectorError, candidate_selector,
    },
    frontend_stages::{
        FrontendFeatureStageProposal, RenderedFrontendFeatureStageProposal,
        SpatialControlFrontendFeatureStageProposal,
    },
    point_response::PointResponseProduction,
    post_window_fanout::CallbackPhase,
    selected_p15_geometry::SelectedP15Geometry,
    selector_u04_handoff::{
        SelectorU04HandoffError, finish_selector_u04_handoff_with_xyz, prepare_selector_u04_handoff,
    },
    source_flags::{HostId, SourceControlSnapshotClock, SourceFlagsAttempt},
    source_geometry_outer::{
        CallbackPointResponseError, CompactOuterMatrix, GlobalSourceGeometrySnapshot,
        NormalSourceGeometryStage, produce_callback_point_response,
    },
    spatial_audio_state::{
        ComponentStageError, MeterComponentOutputRows, MeterComponentStageProposal,
        SpatialAudioAfterProposal, SpatialAudioJoinMetadata, SpatialAudioMachine,
        direct_lfe::{DirectLfeInputView, produce_direct_lfe_stage},
        spatial_audio_composer::{
            ReaderAttemptBinding, SpatialAudioCallbackOutcome, SpatialAudioJocProviders,
            stage_spatial_audio_callback,
        },
        type10_assembler::assemble_type10,
    },
    spatial_control::{
        SpatialControlAfterProposal, SpatialControlAfterStateParts, SpatialControlError,
        SpatialControlInstalled48AfterProjection, SpatialControlJoinMetadata,
        SpatialControlMachine,
    },
    target_context::{
        TargetContextError, TargetContextProjection, build_target_context_projection,
    },
    target_slot_install::{
        SelectedTargetSlots, TargetSlotInstallError, TargetSlotInstallResult, TargetSlotState,
        install_target_slots,
    },
    u04_refinement::{U04RefinementError, U04RefinementProposal, produce_u04_refinement},
};
use crate::{
    frontend::normal_control_snapshot::NormalControlSnapshotOutput,
    frontend::normal_control_snapshot::{
        NormalControlFreshZeroMeterView, NormalControlHeldSourceView, NormalControlSnapshotProposal,
    },
    joc::oamd::reader_target::{
        ReaderTargetCallbackInputs, ReaderTargetControlRows, ReaderTargetError,
        ReaderTargetStageProposal, SELECTED_PROFILE_ID, stage_reader_target,
    },
    meter::installed48_renderer::MeterInstalled48State,
    spatial::installed48_source_map::Installed48SourceMap,
    timeline::{CodecSourceIndex, FrontendCallbackIndex, NormalCallbackCount},
    transaction::CallbackLineageIdentity,
};

/// The delivery callback may carry either a current normal snapshot or a typed held Q-1 snapshot.
#[derive(Debug)]
pub enum NormalControlCallbackSource {
    Normal(Box<NormalControlSnapshotProposal>),
    NormalOutput(Box<NormalControlSnapshotOutput>),
    CachedHold(Box<NormalControlHeldSourceView>),
    FreshZeroMeterHold(Box<NormalControlFreshZeroMeterView>),
}

impl NormalControlCallbackSource {
    pub fn transaction_id(&self) -> &CallbackLineageIdentity {
        match self {
            Self::Normal(snapshot) => snapshot.transaction_id(),
            Self::NormalOutput(snapshot) => snapshot.transaction_id(),
            Self::CachedHold(snapshot) => snapshot.delivery_transaction_id(),
            Self::FreshZeroMeterHold(snapshot) => snapshot.delivery_transaction_id(),
        }
    }

    pub fn source_origin_transaction_id(&self) -> &CallbackLineageIdentity {
        match self {
            Self::Normal(snapshot) => snapshot.transaction_id(),
            Self::NormalOutput(snapshot) => snapshot.transaction_id(),
            Self::CachedHold(snapshot) => snapshot.source_origin_transaction_id(),
            Self::FreshZeroMeterHold(snapshot) => snapshot.source_origin_transaction_id(),
        }
    }

    pub const fn delivery_generation(&self) -> FrontendCallbackIndex {
        match self {
            Self::Normal(snapshot) => snapshot.value_generation(),
            Self::NormalOutput(snapshot) => snapshot.value_generation(),
            Self::CachedHold(snapshot) => snapshot.delivery_generation(),
            Self::FreshZeroMeterHold(snapshot) => snapshot.delivery_generation(),
        }
    }

    pub const fn source_origin_generation(&self) -> FrontendCallbackIndex {
        match self {
            Self::Normal(snapshot) => snapshot.normal_origin_generation(),
            Self::NormalOutput(snapshot) => snapshot.normal_origin_generation(),
            Self::CachedHold(snapshot) => snapshot.source_origin_generation(),
            Self::FreshZeroMeterHold(snapshot) => snapshot.source_origin_generation(),
        }
    }

    pub const fn control38_rows(
        &self,
    ) -> &[super::super::frontend::normal_control_snapshot::NormalControlRow; 38] {
        match self {
            Self::Normal(snapshot) => snapshot.control38_rows(),
            Self::NormalOutput(snapshot) => snapshot.control38_rows(),
            Self::CachedHold(snapshot) => snapshot.control38_rows(),
            Self::FreshZeroMeterHold(snapshot) => snapshot.control38_rows(),
        }
    }

    pub const fn normal_callbacks(&self) -> NormalCallbackCount {
        match self {
            Self::Normal(snapshot) => snapshot.after_state().next_normal_callbacks(),
            Self::NormalOutput(snapshot) => snapshot.normal_callbacks(),
            Self::CachedHold(snapshot) => snapshot.normal_callbacks(),
            Self::FreshZeroMeterHold(snapshot) => snapshot.normal_callbacks(),
        }
    }
}

/// Errors from the ordered noncommitting continuation of the selected spatial callback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialProductionCallbackError {
    CallbackIdentity,
    SourcePointResponse(CallbackPointResponseError),
    CandidateSourceIdentity,
    CandidateResponseColumn,
    Selector(SelectorError),
    SelectorU04(SelectorU04HandoffError),
    U04Refinement(U04RefinementError),
    U04DomainMismatch,
    TargetInstall(TargetSlotInstallError),
    SpatialControlProjection,
    TargetContext(TargetContextError),
    OuterMatrix(super::source_geometry_outer::CompactOuterMatrixError),
    SpatialControl(SpatialControlError),
    Component(ComponentStageError),
    DirectLfe(&'static str),
    Type10(&'static str),
    SpatialAudioComposition(&'static str),
    ReaderTarget(ReaderTargetError),
    Numeric,
}

/// Inputs already staged by the F03W frontend and owner-specific providers.
pub struct SpatialProductionCallbackInputs<'a> {
    pub frontend: SpatialControlFrontendFeatureStageProposal,
    pub source_snapshot: NormalControlCallbackSource,
    pub source_geometry: GlobalSourceGeometrySnapshot,
    pub source_control_clock: SourceControlSnapshotClock,
    pub source_map: &'a Installed48SourceMap,
    pub p15_geometry: &'a SelectedP15Geometry,
    pub spatial_control: &'a SpatialControlMachine,
    pub spatial_audio: &'a SpatialAudioMachine,
    pub spatial_audio_joc_providers: Option<SpatialAudioJocProviders<'a>>,
    pub window: &'a ValidatedWindowQ15Provider,
    pub parameters: &'a ValidatedComponentParameterProvider,
}

/// Meter-only F03W/F04A/F05R56 callback inputs with no F04C or receiver provider surface.
pub struct MeterSpatialProductionCallbackInputs<'a> {
    pub frontend: RenderedFrontendFeatureStageProposal,
    pub source_snapshot: NormalControlCallbackSource,
    pub source_geometry: GlobalSourceGeometrySnapshot,
    pub source_control_clock: SourceControlSnapshotClock,
    pub source_map: &'a Installed48SourceMap,
    pub p15_geometry: &'a SelectedP15Geometry,
    pub spatial_control: &'a SpatialControlMachine,
    pub spatial_audio: &'a SpatialAudioMachine,
    pub window: &'a ValidatedWindowQ15Provider,
    pub parameters: &'a ValidatedComponentParameterProvider,
}

/// Same-callback continuation when F01E controls and source geometry were staged together.
pub struct SpatialProductionNormalCallbackInputs<'a> {
    pub frontend: SpatialControlFrontendFeatureStageProposal,
    pub normal_source_geometry: NormalSourceGeometryStage,
    pub source_control_clock: SourceControlSnapshotClock,
    pub source_map: &'a Installed48SourceMap,
    pub p15_geometry: &'a SelectedP15Geometry,
    pub spatial_control: &'a SpatialControlMachine,
    pub spatial_audio: &'a SpatialAudioMachine,
    pub spatial_audio_joc_providers: Option<SpatialAudioJocProviders<'a>>,
    pub window: &'a ValidatedWindowQ15Provider,
    pub parameters: &'a ValidatedComponentParameterProvider,
}

/// Complete pending values for stages 5–14. No owner is committed or receipt admitted here.
#[must_use]
pub struct SpatialProductionCallbackProposal {
    transaction_id: CallbackLineageIdentity,
    source_geometry: GlobalSourceGeometrySnapshot,
    response: PointResponseProduction,
    selector: Option<CandidateSelectorResult>,
    u04_refinement: Option<U04RefinementProposal>,
    selected_targets: SelectedTargetSlots,
    installation: TargetSlotInstallResult,
    target_context: TargetContextProjection,
    matrix: CompactOuterMatrix,
    dense_d_f32_bits: Box<[u32]>,
    active_target_slots: Box<[u8]>,
    spatial_control_after: super::spatial_control::SpatialControlAfterProposal,
    spatial_audio_after: SpatialAudioAfterProposal,
    spatial_audio_outcome: SpatialAudioCallbackOutcome,
    component_rows: MeterComponentOutputRows,
    component_blocks_processed: i64,
}

/// Consumed callback proposal parts retained for a parent transaction.
/// The owner after-states and all same-callback projections remain available without cloning.
#[must_use]
pub struct SpatialProductionCallbackParentParts {
    pub transaction_id: CallbackLineageIdentity,
    pub source_geometry: GlobalSourceGeometrySnapshot,
    pub response: PointResponseProduction,
    pub selector: Option<CandidateSelectorResult>,
    pub u04_refinement: Option<U04RefinementProposal>,
    pub selected_targets: SelectedTargetSlots,
    pub installation: TargetSlotInstallResult,
    pub target_context: TargetContextProjection,
    pub matrix: CompactOuterMatrix,
    pub dense_d_f32_bits: Box<[u32]>,
    pub active_target_slots: Box<[u8]>,
    pub spatial_control_after: super::spatial_control::SpatialControlAfterProposal,
    pub spatial_audio_after: SpatialAudioAfterProposal,
    pub spatial_audio_outcome: SpatialAudioCallbackOutcome,
    pub component_rows: MeterComponentOutputRows,
    pub component_blocks_processed: i64,
}

/// Component-only Meter spatial callback result; F04C, direct LFE, and Type10 are absent.
#[must_use]
pub struct MeterSpatialProductionCallbackProposal {
    transaction_id: CallbackLineageIdentity,
    spatial_control_after: SpatialControlAfterProposal,
    spatial_audio_after: SpatialAudioAfterProposal,
    component_rows: MeterComponentOutputRows,
    component_blocks_processed: i64,
    installed48_after: MeterInstalled48State,
}

/// Pending owner values exposed to the Meter parent without granting commit authority.
#[must_use]
pub struct MeterSpatialProductionCallbackParentParts {
    pub transaction_id: CallbackLineageIdentity,
    pub spatial_control_after: SpatialControlAfterProposal,
    pub spatial_audio_after: SpatialAudioAfterProposal,
    pub component_rows: MeterComponentOutputRows,
    pub component_blocks_processed: i64,
    pub installed48_after: MeterInstalled48State,
}

/// Same-callback F03W target, marker, assignment, matrix, and writer-set values.
/// This is a pending value projection; it has no owner commit authority.
#[derive(Debug)]
pub(super) struct SpatialControlCallbackProjection {
    target_state: TargetSlotState,
    assignment_i32: [i32; 15],
    markers_i32: [i32; 15],
    matrix_f32_bits: Box<[u32]>,
    writer_set: Box<[u8]>,
}

impl SpatialControlCallbackProjection {
    pub(super) const fn target_state(&self) -> &TargetSlotState {
        &self.target_state
    }

    pub(super) const fn assignment_i32(&self) -> &[i32; 15] {
        &self.assignment_i32
    }

    pub(super) const fn markers_i32(&self) -> &[i32; 15] {
        &self.markers_i32
    }

    pub(super) fn matrix_f32_bits(&self) -> &[u32] {
        &self.matrix_f32_bits
    }

    pub(super) fn writer_set(&self) -> &[u8] {
        &self.writer_set
    }

    /// Test-only cold projection for validating SpatialControl state transitions. It is not a
    /// producer receipt, callback candidate, or production constructor.
    #[cfg(test)]
    pub(in crate::spatial) fn fixture_zero() -> Self {
        Self {
            target_state: TargetSlotState {
                xyz_words: [[0; 3]; 15],
                ids_u32: [u32::MAX; 15],
                attributes_u32: [0; 15],
                previous_xyz_words: [[0; 3]; 15],
                previous_ids_u32: [u32::MAX; 15],
                previous_attributes_u32: [0; 15],
                active_flags: [0; 15],
            },
            assignment_i32: [-1; 15],
            markers_i32: [0; 15],
            matrix_f32_bits: vec![0; 15 * 48].into_boxed_slice(),
            writer_set: Box::new([]),
        }
    }
}

/// Continues the existing spatial pipeline from the indivisible normal-control/geometry stage.
/// The final proposal still contains only pending values; this function commits no owner.
pub fn continue_spatial_production_callback_from_normal_stage(
    input: SpatialProductionNormalCallbackInputs<'_>,
) -> Result<SpatialProductionCallbackProposal, SpatialProductionCallbackError> {
    let SpatialProductionNormalCallbackInputs {
        frontend,
        normal_source_geometry,
        source_control_clock,
        source_map,
        p15_geometry,
        spatial_control,
        spatial_audio,
        spatial_audio_joc_providers,
        window,
        parameters,
    } = input;
    let (normal_control, source_geometry) = normal_source_geometry.into_parts();
    continue_spatial_production_callback(SpatialProductionCallbackInputs {
        frontend,
        source_snapshot: NormalControlCallbackSource::Normal(Box::new(normal_control)),
        source_geometry,
        source_control_clock,
        source_map,
        p15_geometry,
        spatial_control,
        spatial_audio,
        spatial_audio_joc_providers,
        window,
        parameters,
    })
}

/// Selected-profile normal callback wrapper using one validated F04 provider bundle.
///
/// The owned normal-stage proposals are moved exactly once into the existing low-level path.
/// This wrapper supplies only the validated provider view; it performs no additional DSP or
/// owner commit.
pub fn continue_spatial_production_callback_from_normal_stage_with_provider_bundle<'a>(
    input: SpatialProductionNormalCallbackInputs<'a>,
    provider_bundle: &'a SelectedSpatialAudioProviderBundle,
) -> Result<SpatialProductionCallbackProposal, SpatialProductionCallbackError> {
    let SpatialProductionNormalCallbackInputs {
        frontend,
        normal_source_geometry,
        source_control_clock,
        source_map,
        p15_geometry,
        spatial_control,
        spatial_audio,
        window,
        parameters,
        ..
    } = input;

    continue_spatial_production_callback_from_normal_stage(SpatialProductionNormalCallbackInputs {
        frontend,
        normal_source_geometry,
        source_control_clock,
        source_map,
        p15_geometry,
        spatial_control,
        spatial_audio,
        spatial_audio_joc_providers: Some(provider_bundle.spatial_audio_joc_providers()),
        window,
        parameters,
    })
}

impl SpatialProductionCallbackProposal {
    pub fn into_parent_parts(self) -> SpatialProductionCallbackParentParts {
        SpatialProductionCallbackParentParts {
            transaction_id: self.transaction_id,
            source_geometry: self.source_geometry,
            response: self.response,
            selector: self.selector,
            u04_refinement: self.u04_refinement,
            selected_targets: self.selected_targets,
            installation: self.installation,
            target_context: self.target_context,
            matrix: self.matrix,
            dense_d_f32_bits: self.dense_d_f32_bits,
            active_target_slots: self.active_target_slots,
            spatial_control_after: self.spatial_control_after,
            spatial_audio_after: self.spatial_audio_after,
            spatial_audio_outcome: self.spatial_audio_outcome,
            component_rows: self.component_rows,
            component_blocks_processed: self.component_blocks_processed,
        }
    }

    pub fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.transaction_id.callback_index()
    }

    pub const fn response(&self) -> &PointResponseProduction {
        &self.response
    }

    pub const fn source_geometry(&self) -> &GlobalSourceGeometrySnapshot {
        &self.source_geometry
    }

    pub const fn selector(&self) -> Option<&CandidateSelectorResult> {
        self.selector.as_ref()
    }

    pub const fn u04_refinement(&self) -> Option<&U04RefinementProposal> {
        self.u04_refinement.as_ref()
    }

    pub const fn selected_targets(&self) -> &SelectedTargetSlots {
        &self.selected_targets
    }

    pub const fn installation(&self) -> &TargetSlotInstallResult {
        &self.installation
    }

    pub const fn target_context(&self) -> &TargetContextProjection {
        &self.target_context
    }

    pub const fn compact_matrix(&self) -> &CompactOuterMatrix {
        &self.matrix
    }

    /// Stage-12 canonical D remains paired with the pre-quiet active target slots and callback g.
    pub const fn canonical_dense_d_f32_bits(&self) -> &[u32] {
        &self.dense_d_f32_bits
    }

    pub fn active_target_slots(&self) -> &[u8] {
        &self.active_target_slots
    }

    pub const fn spatial_control_after_proposal(
        &self,
    ) -> &super::spatial_control::SpatialControlAfterProposal {
        &self.spatial_control_after
    }

    pub const fn spatial_audio_after_proposal(&self) -> &SpatialAudioAfterProposal {
        &self.spatial_audio_after
    }

    pub const fn spatial_audio_outcome(&self) -> &SpatialAudioCallbackOutcome {
        &self.spatial_audio_outcome
    }

    pub const fn reader_target_controls(&self) -> &ReaderTargetControlRows {
        self.spatial_control_after.reader_target_controls()
    }

    pub const fn component_rows(&self) -> &MeterComponentOutputRows {
        &self.component_rows
    }

    pub const fn component_blocks_processed(&self) -> i64 {
        self.component_blocks_processed
    }
}

impl MeterSpatialProductionCallbackProposal {
    pub fn into_parent_parts(self) -> MeterSpatialProductionCallbackParentParts {
        MeterSpatialProductionCallbackParentParts {
            transaction_id: self.transaction_id,
            spatial_control_after: self.spatial_control_after,
            spatial_audio_after: self.spatial_audio_after,
            component_rows: self.component_rows,
            component_blocks_processed: self.component_blocks_processed,
            installed48_after: self.installed48_after,
        }
    }

    pub fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.transaction_id.callback_index()
    }

    pub const fn spatial_control_after_proposal(&self) -> &SpatialControlAfterProposal {
        &self.spatial_control_after
    }

    pub const fn spatial_audio_after_proposal(&self) -> &SpatialAudioAfterProposal {
        &self.spatial_audio_after
    }

    pub const fn component_rows(&self) -> &MeterComponentOutputRows {
        &self.component_rows
    }

    pub const fn reader_target_controls(&self) -> &ReaderTargetControlRows {
        self.spatial_control_after.reader_target_controls()
    }

    pub const fn component_blocks_processed(&self) -> i64 {
        self.component_blocks_processed
    }

    pub const fn installed48_after(&self) -> &MeterInstalled48State {
        &self.installed48_after
    }
}

/// Shared F03W/F04 value stages. Owner-specific after-state assembly happens in typed callbacks.
struct SpatialProductionCallbackCore<'a> {
    transaction_id: CallbackLineageIdentity,
    source_snapshot: NormalControlCallbackSource,
    source_geometry: GlobalSourceGeometrySnapshot,
    frontend: SpatialProductionFrontend,
    response: PointResponseProduction,
    selector: Option<CandidateSelectorResult>,
    u04_refinement: Option<U04RefinementProposal>,
    selected_targets: SelectedTargetSlots,
    installation: TargetSlotInstallResult,
    target_context: TargetContextProjection,
    matrix: CompactOuterMatrix,
    dense_d_f32_bits: Box<[u32]>,
    active_target_slots: Box<[u8]>,
    spatial_control: &'a SpatialControlMachine,
    spatial_control_join: SpatialControlJoinMetadata,
    spatial_audio_component: MeterComponentStageProposal,
    component_blocks_processed: i64,
    spatial_audio: &'a SpatialAudioMachine,
    spatial_audio_join: SpatialAudioJoinMetadata,
    spatial_audio_joc_providers: Option<SpatialAudioJocProviders<'a>>,
    source_map: &'a Installed48SourceMap,
    window: &'a ValidatedWindowQ15Provider,
    parameters: &'a ValidatedComponentParameterProvider,
    callback_phase: CallbackPhase,
    reader_attempt: SourceFlagsAttempt,
    reader_target_state: crate::joc::oamd::reader_target::ReaderTargetState,
    reader_target_receipt: Option<crate::joc::oamd::reader_target::ReaderTargetUpdateReceipt>,
    reader_target_controls: ReaderTargetControlRows,
    spatial_control_projection: SpatialControlCallbackProjection,
}

struct SpatialProductionFrontend {
    frontend: FrontendFeatureStageProposal,
    spatial_audio: super::post_window_fanout::SpatialAudioPostWindowView,
}

struct SpatialProductionCallbackCoreInput<'a> {
    frontend: SpatialProductionFrontend,
    source_snapshot: NormalControlCallbackSource,
    source_geometry: GlobalSourceGeometrySnapshot,
    source_control_clock: SourceControlSnapshotClock,
    source_map: &'a Installed48SourceMap,
    p15_geometry: &'a SelectedP15Geometry,
    spatial_control: &'a SpatialControlMachine,
    spatial_audio: &'a SpatialAudioMachine,
    spatial_audio_joc_providers: Option<SpatialAudioJocProviders<'a>>,
    window: &'a ValidatedWindowQ15Provider,
    parameters: &'a ValidatedComponentParameterProvider,
}

pub fn continue_spatial_production_callback(
    input: SpatialProductionCallbackInputs<'_>,
) -> Result<SpatialProductionCallbackProposal, SpatialProductionCallbackError> {
    let SpatialProductionCallbackInputs {
        frontend,
        source_snapshot,
        source_geometry,
        source_control_clock,
        source_map,
        p15_geometry,
        spatial_control,
        spatial_audio,
        spatial_audio_joc_providers,
        window,
        parameters,
    } = input;
    let (post_window, installed48_after) = frontend.into_parts();
    let core = prepare_spatial_production_core(SpatialProductionCallbackCoreInput {
        frontend: SpatialProductionFrontend {
            frontend: post_window.frontend,
            spatial_audio: post_window.spatial_audio,
        },
        source_snapshot,
        source_geometry,
        source_control_clock,
        source_map,
        p15_geometry,
        spatial_control,
        spatial_audio,
        spatial_audio_joc_providers,
        window,
        parameters,
    })?;
    finish_spatial_production_callback(core, installed48_after)
}

/// Produces the Meter F03W/F04A/F05R56 proposal without entering F04C or receiver work.
pub fn continue_meter_spatial_production_callback(
    input: MeterSpatialProductionCallbackInputs<'_>,
) -> Result<MeterSpatialProductionCallbackProposal, SpatialProductionCallbackError> {
    let MeterSpatialProductionCallbackInputs {
        frontend,
        source_snapshot,
        source_geometry,
        source_control_clock,
        source_map,
        p15_geometry,
        spatial_control,
        spatial_audio,
        window,
        parameters,
    } = input;
    let mut frontend = frontend;
    let installed48_after = frontend
        .take_installed48_after()
        .ok_or(SpatialProductionCallbackError::CallbackIdentity)?;
    let core = prepare_spatial_production_core(SpatialProductionCallbackCoreInput {
        frontend: SpatialProductionFrontend {
            frontend: frontend.frontend,
            spatial_audio: frontend.spatial_audio,
        },
        source_snapshot,
        source_geometry,
        source_control_clock,
        source_map,
        p15_geometry,
        spatial_control,
        spatial_audio,
        spatial_audio_joc_providers: None,
        window,
        parameters,
    })?;
    finish_meter_spatial_production_callback(core, installed48_after)
}

/// Selected-profile callback wrapper for the general source-snapshot form.
///
/// `NormalControlCallbackSource::CachedHold` remains available through this form. The validated
/// provider bundle does not change callback phase or relax the existing low-level phase checks;
/// the wrapper only supplies the provider view and delegates the moved inputs.
pub fn continue_spatial_production_callback_with_provider_bundle<'a>(
    input: SpatialProductionCallbackInputs<'a>,
    provider_bundle: &'a SelectedSpatialAudioProviderBundle,
) -> Result<SpatialProductionCallbackProposal, SpatialProductionCallbackError> {
    let SpatialProductionCallbackInputs {
        frontend,
        source_snapshot,
        source_geometry,
        source_control_clock,
        source_map,
        p15_geometry,
        spatial_control,
        spatial_audio,
        window,
        parameters,
        ..
    } = input;

    continue_spatial_production_callback(SpatialProductionCallbackInputs {
        frontend,
        source_snapshot,
        source_geometry,
        source_control_clock,
        source_map,
        p15_geometry,
        spatial_control,
        spatial_audio,
        spatial_audio_joc_providers: Some(provider_bundle.spatial_audio_joc_providers()),
        window,
        parameters,
    })
}

fn prepare_spatial_production_core(
    input: SpatialProductionCallbackCoreInput<'_>,
) -> Result<SpatialProductionCallbackCore<'_>, SpatialProductionCallbackError> {
    let SpatialProductionCallbackCoreInput {
        frontend,
        source_snapshot,
        source_geometry,
        source_control_clock,
        source_map,
        p15_geometry,
        spatial_control,
        spatial_audio,
        spatial_audio_joc_providers,
        window,
        parameters,
    } = input;
    let callback_phase = frontend.frontend.current_features.lineage.callback_phase;
    let allowed_phase = matches!(
        (callback_phase, &source_snapshot),
        (
            CallbackPhase::Normal,
            NormalControlCallbackSource::Normal(_) | NormalControlCallbackSource::NormalOutput(_)
        ) | (
            CallbackPhase::CachedDrain,
            NormalControlCallbackSource::CachedHold(_)
        ) | (
            CallbackPhase::FreshZeroInputDrain,
            NormalControlCallbackSource::FreshZeroMeterHold(_)
        )
    );
    if !allowed_phase {
        return Err(SpatialProductionCallbackError::CallbackIdentity);
    }
    let transaction_id = frontend
        .frontend
        .current_features
        .lineage
        .transaction_id
        .clone();
    let callback_index = transaction_id.callback_index();
    if source_geometry.value_generation() != callback_index
        || source_geometry.transaction_id() != &transaction_id
        || source_snapshot.delivery_generation() != callback_index
        || source_snapshot.source_origin_generation().value() > callback_index.value()
        || source_snapshot.transaction_id() != &transaction_id
        || source_control_clock.callback_index() != callback_index
        || source_control_clock.source_origin() != source_snapshot.source_origin_generation()
        || source_control_clock.normal_callbacks() != source_snapshot.normal_callbacks()
        || source_snapshot
            .source_origin_transaction_id()
            .callback_index()
            != source_control_clock.source_origin()
        || source_snapshot
            .source_origin_transaction_id()
            .programme_id()
            != transaction_id.programme_id()
        || source_snapshot.source_origin_transaction_id().attempt_id()
            != transaction_id.attempt_id()
        || spatial_control.committed().callback_index() != callback_index
        || spatial_audio.generation() != u64::from(callback_index.value())
        || spatial_control.committed().reader_targets_callback_index() != callback_index
    {
        return Err(SpatialProductionCallbackError::CallbackIdentity);
    }

    let response_gate = frontend
        .frontend
        .point_response_gate()
        .map_err(|_| SpatialProductionCallbackError::CallbackIdentity)?;
    let response = produce_callback_point_response(
        &source_geometry,
        source_control_clock,
        p15_geometry,
        response_gate,
    )
    .map_err(SpatialProductionCallbackError::SourcePointResponse)?;

    let target_before = target_state_from_spatial_control(spatial_control);
    let selector_input = candidate_selector_input(
        &frontend,
        &source_geometry,
        source_map,
        &response,
        &target_before,
    )?;
    let (selector, u04_refinement, selected_targets) = if selector_input.candidate_ids.is_empty() {
        (
            None,
            None,
            SelectedTargetSlots {
                xyz_words: Vec::new(),
                ids_u32: Vec::new(),
                attributes_u32: Vec::new(),
            },
        )
    } else {
        let selector = candidate_selector(selector_input.clone())
            .map_err(SpatialProductionCallbackError::Selector)?;
        let (refinement, selected) = finish_selection_with_u04(&selector_input, &selector)?;
        (Some(selector), refinement, selected)
    };

    let installation = install_target_slots(&target_before, &selected_targets)
        .map_err(SpatialProductionCallbackError::TargetInstall)?;
    let target_context = build_target_context_projection(
        p15_geometry,
        &installation,
        transaction_id.clone(),
        callback_index,
    )
    .map_err(SpatialProductionCallbackError::TargetContext)?;
    let matrix = super::source_geometry_outer::produce_compact_outer_matrix(
        &target_context,
        &source_geometry,
        &response,
    )
    .map_err(SpatialProductionCallbackError::OuterMatrix)?;

    let spatial_audio_join = SpatialAudioJoinMetadata::new(
        transaction_id.programme_id().clone(),
        transaction_id.attempt_id(),
        transaction_id.clone(),
        callback_index,
        spatial_audio.generation(),
        spatial_audio.digest(),
        spatial_control.profile(),
    );
    let spatial_control_join =
        SpatialControlJoinMetadata::pending_callback(spatial_control, transaction_id.clone())
            .map_err(SpatialProductionCallbackError::SpatialControl)?;
    let installed48_f32_bits = installed48_bits(&frontend)?;
    let component_preview = spatial_audio
        .prepare_component_stage_preview(
            &spatial_audio_join,
            &spatial_control_join,
            spatial_control,
            frontend.frontend.current_features.lineage.callback_phase,
            window,
            parameters,
            installed48_f32_bits,
            &matrix,
        )
        .map_err(SpatialProductionCallbackError::Component)?;

    let current_xyz_words = flatten_target_xyz(installation.next_state().xyz_words);
    let active_target_slots = matrix
        .destinations()
        .iter()
        .map(|destination| destination.target_slot().value())
        .collect::<Vec<_>>();
    let compact_matrix = component_preview.compact_matrix_f32();
    let source_flags = std::array::from_fn(|source| {
        frontend.frontend.current_features.source_flags.payload()[source][0]
    });
    let previous_overlap_rows = component_preview.previous_overlap_rows();
    let current_mixer_rows = component_preview.current_mixer_rows();
    let reader_inputs = ReaderTargetCallbackInputs {
        callback_index,
        reader_enabled: callback_index.value() >= 2,
        snapshot_keys_complete: source_geometry.rows().len() == 48,
        programme_id: transaction_id.programme_id().clone(),
        attempt_id: transaction_id.attempt_id(),
        transaction_id: transaction_id.clone(),
        profile_id: SELECTED_PROFILE_ID,
        current_xyz_words: &current_xyz_words,
        current_ids: &installation.next_state().ids_u32,
        current_attributes: &installation.next_state().attributes_u32,
        assignment_active: &installation.next_state().active_flags,
        active_target_indices: &active_target_slots,
        matrix: &compact_matrix,
        source_flags: &source_flags,
        previous_overlap_rows,
        current_mixer_rows,
    };
    let reader_target = stage_reader_target(
        spatial_control.committed().reader_target_state(),
        &reader_inputs,
    )
    .map_err(SpatialProductionCallbackError::ReaderTarget)?;
    let spatial_control_projection =
        spatial_control_callback_projection(&installation, &matrix, &reader_target)?;
    let (reader_target_state, reader_target_receipt, reader_target_controls, _) =
        reader_target.into_parent_parts();
    let dense_d_f32_bits = component_preview
        .canonical_dense_matrix_f32_bits()
        .to_vec()
        .into_boxed_slice();
    let component = component_preview
        .execute(
            spatial_audio,
            &spatial_audio_join,
            &spatial_control_join,
            spatial_control,
            window,
            parameters,
        )
        .map_err(SpatialProductionCallbackError::Component)?;

    let component_blocks_processed = component.blocks_processed();
    let current_lineage = &frontend.frontend.current_features.lineage;
    let reader_attempt = current_lineage.attempt;
    Ok(SpatialProductionCallbackCore {
        transaction_id,
        source_snapshot,
        source_geometry,
        frontend,
        response,
        selector,
        u04_refinement,
        selected_targets,
        installation,
        target_context,
        matrix,
        dense_d_f32_bits,
        active_target_slots: active_target_slots.into_boxed_slice(),
        spatial_control,
        spatial_control_join,
        spatial_audio_component: component,
        component_blocks_processed,
        spatial_audio,
        spatial_audio_join,
        spatial_audio_joc_providers,
        source_map,
        window,
        parameters,
        callback_phase,
        reader_attempt,
        reader_target_state,
        reader_target_receipt,
        reader_target_controls,
        spatial_control_projection,
    })
}

fn finish_spatial_production_callback(
    core: SpatialProductionCallbackCore<'_>,
    installed48_after: SpatialControlInstalled48AfterProjection,
) -> Result<SpatialProductionCallbackProposal, SpatialProductionCallbackError> {
    let SpatialProductionCallbackCore {
        transaction_id,
        source_snapshot,
        source_geometry,
        frontend,
        response,
        selector,
        u04_refinement,
        selected_targets,
        installation,
        target_context,
        matrix,
        dense_d_f32_bits,
        active_target_slots,
        spatial_control,
        spatial_control_join,
        spatial_audio_component,
        component_blocks_processed,
        spatial_audio,
        spatial_audio_join,
        spatial_audio_joc_providers,
        source_map,
        callback_phase,
        reader_attempt,
        reader_target_state,
        reader_target_receipt,
        reader_target_controls,
        spatial_control_projection,
        ..
    } = core;
    let parts = installed48_after.into_parts();
    let callback_index = transaction_id.callback_index();
    let regular_raw_history = current_regular_raw_history(&frontend.frontend);
    if parts.callback_index != callback_index
        || parts.callback_phase != callback_phase
        || parts.normal_callbacks != source_snapshot.normal_callbacks()
        || parts.regular_raw_history_bits.as_ref() != regular_raw_history.as_ref()
    {
        return Err(SpatialProductionCallbackError::CallbackIdentity);
    }

    let lfe_host = source_map.direct_lfe_host();
    let lfe_samples = parts
        .source_pcm_rows
        .get(usize::from(lfe_host.value()))
        .ok_or(SpatialProductionCallbackError::CallbackIdentity)?
        .iter()
        .copied()
        .map(f32::from_bits)
        .collect::<Vec<_>>();
    let control38_rows = source_snapshot
        .control38_rows()
        .to_vec()
        .try_into()
        .map_err(|_| SpatialProductionCallbackError::CallbackIdentity)?;
    let spatial_control_after = spatial_control
        .propose_callback_after(
            transaction_id.clone(),
            SpatialControlAfterStateParts {
                callback_index,
                callback_phase,
                source_cursor_samples: parts.source_cursor,
                normal_callbacks: parts.normal_callbacks,
                source_pcm_rows: parts.source_pcm_rows,
                control38_rows,
                regular_raw_history_bits: parts.regular_raw_history_bits,
                previous_current_rows_bits: parts.current_rows_bits,
                diffusion_tails: parts.diffusion_after,
                reader_target_state,
                projection: spatial_control_projection,
            },
            frontend.frontend.current_features,
            reader_target_receipt,
            reader_target_controls,
        )
        .map_err(SpatialProductionCallbackError::SpatialControl)?;

    let lfe_host_id = HostId::new(lfe_host.value())
        .map_err(|_| SpatialProductionCallbackError::CallbackIdentity)?;
    let lfe_input = DirectLfeInputView::try_from_spatial_control_host(
        SpatialControlMachine::lfe_ingress_authority(),
        transaction_id.programme_id().clone(),
        transaction_id.attempt_id(),
        transaction_id.clone(),
        callback_index,
        callback_phase,
        spatial_control.profile(),
        spatial_control,
        lfe_host_id,
        lfe_samples,
    )
    .map_err(|error| SpatialProductionCallbackError::DirectLfe(error.code()))?;
    let lfe_stage = produce_direct_lfe_stage(
        spatial_audio,
        &spatial_audio_join,
        &spatial_control_join,
        spatial_control,
        callback_phase,
        lfe_input,
    )
    .map_err(|error| SpatialProductionCallbackError::DirectLfe(error.code()))?;
    let (component_stage, component_rows) = spatial_audio_component.into_parts();
    let type10 = assemble_type10(
        &spatial_audio_join,
        callback_phase,
        component_stage,
        lfe_stage,
    )
    .map_err(|error| SpatialProductionCallbackError::Type10(error.code()))?;

    let attempt_binding = spatial_control_after
        .reader_target_receipt()
        .map(|_| ReaderAttemptBinding::new(reader_attempt, reader_attempt))
        .transpose()
        .map_err(|error| SpatialProductionCallbackError::SpatialAudioComposition(error.code()))?;
    let spatial_audio_callback = stage_spatial_audio_callback(
        spatial_audio,
        spatial_audio_join,
        callback_phase,
        type10,
        spatial_control_after.reader_target_receipt(),
        attempt_binding,
        spatial_audio_joc_providers,
    )
    .map_err(|error| SpatialProductionCallbackError::SpatialAudioComposition(error.code()))?;
    let (spatial_audio_after, spatial_audio_outcome) = spatial_audio_callback.into_parts();

    Ok(SpatialProductionCallbackProposal {
        transaction_id,
        source_geometry,
        response,
        selector,
        u04_refinement,
        selected_targets,
        installation,
        target_context,
        matrix,
        dense_d_f32_bits,
        active_target_slots,
        spatial_control_after,
        spatial_audio_after,
        spatial_audio_outcome,
        component_rows,
        component_blocks_processed,
    })
}

fn finish_meter_spatial_production_callback(
    core: SpatialProductionCallbackCore<'_>,
    installed48_after: MeterInstalled48State,
) -> Result<MeterSpatialProductionCallbackProposal, SpatialProductionCallbackError> {
    let SpatialProductionCallbackCore {
        transaction_id,
        source_snapshot,
        source_geometry: _,
        frontend,
        response: _,
        selector: _,
        u04_refinement: _,
        selected_targets: _,
        installation: _,
        target_context: _,
        matrix: _,
        dense_d_f32_bits: _,
        active_target_slots: _,
        spatial_control,
        spatial_control_join: _,
        spatial_audio_component,
        component_blocks_processed,
        spatial_audio,
        spatial_audio_join,
        spatial_audio_joc_providers: _,
        source_map,
        window,
        parameters,
        callback_phase,
        reader_attempt: _,
        reader_target_state,
        reader_target_receipt,
        reader_target_controls,
        spatial_control_projection,
    } = core;
    if callback_phase == CallbackPhase::CachedDrain {
        return Err(SpatialProductionCallbackError::CallbackIdentity);
    }
    let projection = installed48_after.spatial_control_projection();
    let mut diffusion_by_host = projection.diffusion_states;
    let mut diffusion_after = std::array::from_fn(|_| None);
    for (tail, binding) in source_map.physical_to_global_source().iter().enumerate() {
        diffusion_after[tail] = diffusion_by_host.remove(&binding.physical_host().value());
    }
    if !diffusion_by_host.is_empty() {
        return Err(SpatialProductionCallbackError::CallbackIdentity);
    }
    let control38_rows = source_snapshot
        .control38_rows()
        .to_vec()
        .try_into()
        .map_err(|_| SpatialProductionCallbackError::CallbackIdentity)?;
    let spatial_control_after = spatial_control
        .propose_callback_after(
            transaction_id.clone(),
            SpatialControlAfterStateParts {
                callback_index: transaction_id.callback_index(),
                callback_phase,
                source_cursor_samples: projection.source_cursor,
                normal_callbacks: source_snapshot.normal_callbacks(),
                source_pcm_rows: projection.previous_pcm_rows,
                control38_rows,
                regular_raw_history_bits: current_regular_raw_history(&frontend.frontend),
                previous_current_rows_bits: projection
                    .previous_current_rows
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                diffusion_tails: diffusion_after,
                reader_target_state,
                projection: spatial_control_projection,
            },
            frontend.frontend.current_features,
            reader_target_receipt,
            reader_target_controls,
        )
        .map_err(SpatialProductionCallbackError::SpatialControl)?;
    let provider_lineage = [
        *window.receipt().validated_manifest_lineage(),
        *parameters.receipt().validated_manifest_lineage(),
    ];
    let (spatial_audio_after, component_rows) = spatial_audio
        .stage_meter_component_after(
            &spatial_audio_join,
            provider_lineage,
            spatial_audio_component,
        )
        .map_err(SpatialProductionCallbackError::Component)?
        .into_parts();

    Ok(MeterSpatialProductionCallbackProposal {
        transaction_id,
        spatial_control_after,
        spatial_audio_after,
        component_rows,
        component_blocks_processed,
        installed48_after,
    })
}

fn current_regular_raw_history(frontend: &FrontendFeatureStageProposal) -> Box<[u32]> {
    frontend
        .raw_history
        .rows()
        .iter()
        .flat_map(|row| row.iter().copied())
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn spatial_control_callback_projection(
    installation: &TargetSlotInstallResult,
    matrix: &CompactOuterMatrix,
    reader_target: &ReaderTargetStageProposal,
) -> Result<SpatialControlCallbackProjection, SpatialProductionCallbackError> {
    if installation.mapping().len() != 15
        || matrix.destinations().len() != matrix.c_rows_u32().len()
        || matrix.c_rows_u32().iter().any(|row| row.len() != 48)
    {
        return Err(SpatialProductionCallbackError::SpatialControlProjection);
    }

    let mut assignment_i32 = [0_i32; 15];
    for (target, mapped_source) in installation.mapping().iter().copied().enumerate() {
        assignment_i32[target] = i32::from(mapped_source);
    }

    let mut matrix_f32_bits = vec![0_u32; 15 * 48].into_boxed_slice();
    let mut writer_set = Vec::with_capacity(matrix.destinations().len());
    for (compact_row, destination) in matrix.destinations().iter().enumerate() {
        let target = usize::from(destination.target_slot().value());
        let row = matrix
            .c_rows_u32()
            .get(compact_row)
            .ok_or(SpatialProductionCallbackError::SpatialControlProjection)?;
        if target >= 15
            || writer_set
                .last()
                .is_some_and(|previous| *previous >= target as u8)
        {
            return Err(SpatialProductionCallbackError::SpatialControlProjection);
        }
        matrix_f32_bits[target * 48..(target + 1) * 48].copy_from_slice(row);
        writer_set.push(
            u8::try_from(target)
                .map_err(|_| SpatialProductionCallbackError::SpatialControlProjection)?,
        );
    }

    Ok(SpatialControlCallbackProjection {
        target_state: installation.next_state().clone(),
        assignment_i32,
        markers_i32: *reader_target.markers_i32(),
        matrix_f32_bits,
        writer_set: writer_set.into_boxed_slice(),
    })
}

fn finish_selection_with_u04(
    candidates: &CandidateSelectorInput,
    selector: &CandidateSelectorResult,
) -> Result<(Option<U04RefinementProposal>, SelectedTargetSlots), SpatialProductionCallbackError> {
    let handoff = prepare_selector_u04_handoff(candidates, selector)
        .map_err(SpatialProductionCallbackError::SelectorU04)?;
    if handoff.admission().refinement_called() {
        let refinement = produce_u04_refinement(candidates, handoff.selected_targets())
            .map_err(SpatialProductionCallbackError::U04Refinement)?;
        if refinement.source_ids() != candidates.candidate_ids
            || refinement.source_xyz_u32() != candidates.positions_u32
            || refinement.source_attributes_u32() != candidates.attributes_u32
            || refinement.selected_target_ids() != handoff.selected_targets().ids_u32
            || refinement.initial_target_xyz_u32() != handoff.selected_targets().xyz_words
            || refinement.selected_target_attributes() != handoff.selected_targets().attributes_u32
        {
            return Err(SpatialProductionCallbackError::U04DomainMismatch);
        }
        let selected =
            finish_selector_u04_handoff_with_xyz(handoff, Some(refinement.projected_xyz_u32()))
                .map_err(SpatialProductionCallbackError::SelectorU04)?;
        Ok((Some(refinement), selected))
    } else {
        let selected = finish_selector_u04_handoff_with_xyz(handoff, None)
            .map_err(SpatialProductionCallbackError::SelectorU04)?;
        Ok((None, selected))
    }
}

fn candidate_selector_input(
    frontend: &SpatialProductionFrontend,
    source_geometry: &GlobalSourceGeometrySnapshot,
    source_map: &Installed48SourceMap,
    response: &PointResponseProduction,
    target_before: &TargetSlotState,
) -> Result<CandidateSelectorInput, SpatialProductionCallbackError> {
    let gate = &frontend.frontend.gate;
    if gate.candidate_ids.as_ref() != response.candidate_ids()
        || gate.candidate_ids.len() != response.candidate_feature_width_f32_bits().len()
    {
        return Err(SpatialProductionCallbackError::CandidateResponseColumn);
    }
    let positions_u32 = gate
        .candidate_ids
        .iter()
        .map(|source| {
            source_geometry
                .row_for_source(CodecSourceIndex::new(u64::from(*source)))
                .map(super::source_geometry_outer::GlobalSourceGeometryRow::xyz_f32_bits)
                .ok_or(SpatialProductionCallbackError::CandidateSourceIdentity)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let attributes_u32 = gate
        .candidate_ids
        .iter()
        .map(|source| {
            source_map
                .row_for_source(CodecSourceIndex::new(u64::from(*source)))
                .map(super::installed48_source_map::Installed48SourceRow::packed_attribute)
                .ok_or(SpatialProductionCallbackError::CandidateSourceIdentity)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let response_u32 = (0..15)
        .map(|basis| {
            gate.candidate_ids
                .iter()
                .map(|source| {
                    response
                        .columns()
                        .iter()
                        .find(|column| column.source_index().value() == u64::from(*source))
                        .map(|column| column.response_f32_bits()[basis])
                        .ok_or(SpatialProductionCallbackError::CandidateResponseColumn)
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CandidateSelectorInput {
        candidate_ids: gate.candidate_ids.to_vec(),
        feature_u32: gate
            .candidate_features
            .iter()
            .map(|band| band.to_vec())
            .collect(),
        positions_u32,
        width_u32: response.candidate_feature_width_f32_bits().to_vec(),
        attributes_u32,
        response_u32,
        previous_ids: target_before.ids_u32.into(),
    })
}

fn target_state_from_spatial_control(owner: &SpatialControlMachine) -> TargetSlotState {
    let committed = owner.committed();
    TargetSlotState {
        xyz_words: std::array::from_fn(|slot| {
            let start = slot * 3;
            [
                committed.target_current_xyz_u32()[start],
                committed.target_current_xyz_u32()[start + 1],
                committed.target_current_xyz_u32()[start + 2],
            ]
        }),
        ids_u32: *committed.target_current_id_u32(),
        attributes_u32: *committed.target_current_attribute_u32(),
        previous_xyz_words: std::array::from_fn(|slot| {
            let start = slot * 3;
            [
                committed.target_previous_xyz_u32()[start],
                committed.target_previous_xyz_u32()[start + 1],
                committed.target_previous_xyz_u32()[start + 2],
            ]
        }),
        previous_ids_u32: *committed.target_previous_id_u32(),
        previous_attributes_u32: *committed.target_previous_attribute_u32(),
        active_flags: *committed.target_active_u8(),
    }
}

fn flatten_target_xyz(xyz: [[u32; 3]; 15]) -> [u32; 45] {
    xyz.into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn installed48_bits(
    frontend: &SpatialProductionFrontend,
) -> Result<Vec<u32>, SpatialProductionCallbackError> {
    let rows = (0..48)
        .map(|source| {
            frontend
                .spatial_audio
                .source_samples(source)
                .ok_or(SpatialProductionCallbackError::CallbackIdentity)
                .map(|samples| {
                    samples
                        .iter()
                        .map(|sample| sample.to_bits())
                        .collect::<Vec<_>>()
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if rows.iter().any(|row| row.len() != 3_072) {
        return Err(SpatialProductionCallbackError::CallbackIdentity);
    }
    Ok(rows.into_iter().flatten().collect())
}

#[cfg(test)]
mod tests;
