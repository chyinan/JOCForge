// pattern: Functional Core
//! Noncommitting F03W continuation from the current frontend feature proposal.

use super::{
    ValidatedComponentParameterProvider, ValidatedWindowQ15Provider,
    candidate_selector::{
        CandidateSelectorInput, CandidateSelectorResult, SelectorError, candidate_selector,
    },
    frontend_stages::RenderedFrontendFeatureStageProposal,
    point_response::PointResponseProduction,
    selected_p15_geometry::SelectedP15Geometry,
    selector_u04_handoff::{
        SelectorU04HandoffError, finish_selector_u04_handoff_with_xyz, prepare_selector_u04_handoff,
    },
    source_flags::SourceControlSnapshotClock,
    source_geometry_outer::{
        CallbackPointResponseError, CompactOuterMatrix, GlobalSourceGeometrySnapshot,
        NormalSourceGeometryStage, produce_callback_point_response,
    },
    spatial_audio_state::{
        ComponentStageError, MeterComponentOutputRows, MeterComponentStageProposal,
        SpatialAudioJoinMetadata, SpatialAudioMachine,
    },
    spatial_control::{
        SpatialControlAfterStateParts, SpatialControlError, SpatialControlJoinMetadata,
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
    frontend::normal_control_snapshot::{
        NormalControlHeldSourceView, NormalControlSnapshotProposal,
    },
    joc::oamd::reader_target::{
        ReaderTargetCallbackInputs, ReaderTargetControlRows, ReaderTargetError,
        ReaderTargetStageProposal, SELECTED_PROFILE_ID, stage_reader_target,
    },
    spatial::installed48_source_map::Installed48SourceMap,
    timeline::{CodecSourceIndex, FrontendCallbackIndex, NormalCallbackCount},
    transaction::CallbackLineageIdentity,
};

/// The delivery callback may carry either a current normal snapshot or a typed held Q-1 snapshot.
#[derive(Debug)]
pub enum NormalControlCallbackSource {
    Normal(Box<NormalControlSnapshotProposal>),
    CachedHold(Box<NormalControlHeldSourceView>),
}

impl NormalControlCallbackSource {
    pub fn transaction_id(&self) -> &CallbackLineageIdentity {
        match self {
            Self::Normal(snapshot) => snapshot.transaction_id(),
            Self::CachedHold(snapshot) => snapshot.delivery_transaction_id(),
        }
    }

    pub const fn delivery_generation(&self) -> FrontendCallbackIndex {
        match self {
            Self::Normal(snapshot) => snapshot.value_generation(),
            Self::CachedHold(snapshot) => snapshot.delivery_generation(),
        }
    }

    pub const fn source_origin_generation(&self) -> FrontendCallbackIndex {
        match self {
            Self::Normal(snapshot) => snapshot.normal_origin_generation(),
            Self::CachedHold(snapshot) => snapshot.source_origin_generation(),
        }
    }

    pub const fn control38_rows(
        &self,
    ) -> &[super::super::frontend::normal_control_snapshot::NormalControlRow; 38] {
        match self {
            Self::Normal(snapshot) => snapshot.control38_rows(),
            Self::CachedHold(snapshot) => snapshot.control38_rows(),
        }
    }

    pub const fn normal_callbacks(&self) -> NormalCallbackCount {
        match self {
            Self::Normal(snapshot) => snapshot.after_state().next_normal_callbacks(),
            Self::CachedHold(snapshot) => snapshot.normal_callbacks(),
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
    ReaderTarget(ReaderTargetError),
    Numeric,
}

/// Inputs already staged by the F03W frontend and owner-specific providers.
pub struct SpatialProductionCallbackInputs<'a> {
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
    pub frontend: RenderedFrontendFeatureStageProposal,
    pub normal_source_geometry: NormalSourceGeometryStage,
    pub source_control_clock: SourceControlSnapshotClock,
    pub source_map: &'a Installed48SourceMap,
    pub p15_geometry: &'a SelectedP15Geometry,
    pub spatial_control: &'a SpatialControlMachine,
    pub spatial_audio: &'a SpatialAudioMachine,
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
    component: MeterComponentStageProposal,
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
        window,
        parameters,
    })
}

impl SpatialProductionCallbackProposal {
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

    pub const fn reader_target_controls(&self) -> &ReaderTargetControlRows {
        self.spatial_control_after.reader_target_controls()
    }

    pub const fn component_rows(&self) -> &MeterComponentOutputRows {
        self.component.meter_rows()
    }

    pub const fn component_proposal(&self) -> &MeterComponentStageProposal {
        &self.component
    }
}

/// Runs the same-callback source response, selector/U04 handoff, installation, C/d, marker,
/// history and component stages once each, returning every owner after-value still pending.
pub fn continue_spatial_production_callback(
    input: SpatialProductionCallbackInputs<'_>,
) -> Result<SpatialProductionCallbackProposal, SpatialProductionCallbackError> {
    let SpatialProductionCallbackInputs {
        mut frontend,
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

    if frontend.frontend.current_features.lineage.callback_phase
        != super::post_window_fanout::CallbackPhase::Normal
    {
        return Err(SpatialProductionCallbackError::CallbackIdentity);
    }
    let installed48_after = frontend
        .take_installed48_after()
        .ok_or(SpatialProductionCallbackError::CallbackIdentity)?;
    let installed48_projection = installed48_after.into_spatial_control_parts();
    let source_cursor_samples = installed48_projection.source_cursor;
    let source_pcm_rows = installed48_projection.previous_pcm_rows;
    let previous_current_rows_bits = installed48_projection.previous_current_rows;
    let mut regular_physical_hosts = [false; 38];
    for binding in source_map.physical_to_global_source() {
        let host = usize::from(binding.physical_host().value());
        let slot = regular_physical_hosts
            .get_mut(host)
            .ok_or(SpatialProductionCallbackError::CallbackIdentity)?;
        if std::mem::replace(slot, true) {
            return Err(SpatialProductionCallbackError::CallbackIdentity);
        }
    }
    let lfe_host = regular_physical_hosts
        .iter()
        .position(|regular| !regular)
        .ok_or(SpatialProductionCallbackError::CallbackIdentity)?;
    let mut diffusion_tails = std::array::from_fn(|_| None);
    let mut regular_tail = 0;
    let mut diffusion_by_host = installed48_projection.diffusion_states;
    for host in 0..38 {
        if host == lfe_host {
            continue;
        }
        diffusion_tails[regular_tail] = diffusion_by_host.remove(&(host as u8));
        regular_tail += 1;
    }
    if regular_tail != 37 || !diffusion_by_host.is_empty() {
        return Err(SpatialProductionCallbackError::CallbackIdentity);
    }
    let raw_history_bits = frontend
        .frontend
        .raw_history
        .rows()
        .iter()
        .flat_map(|row| row.iter().copied())
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let control38_rows = source_snapshot
        .control38_rows()
        .to_vec()
        .try_into()
        .map_err(|_| SpatialProductionCallbackError::CallbackIdentity)?;
    let current_features = frontend.frontend.current_features;
    let spatial_control_after = spatial_control
        .propose_callback_after(
            transaction_id.clone(),
            SpatialControlAfterStateParts {
                callback_index,
                source_cursor_samples,
                normal_callbacks: source_snapshot.normal_callbacks(),
                source_pcm_rows,
                control38_rows,
                regular_raw_history_bits: raw_history_bits,
                previous_current_rows_bits: previous_current_rows_bits
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                diffusion_tails,
                reader_target_state,
                projection: spatial_control_projection,
            },
            current_features,
            reader_target_receipt,
            reader_target_controls,
        )
        .map_err(SpatialProductionCallbackError::SpatialControl)?;

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
        active_target_slots: active_target_slots.into_boxed_slice(),
        spatial_control_after,
        component,
    })
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
    frontend: &RenderedFrontendFeatureStageProposal,
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
    frontend: &RenderedFrontendFeatureStageProposal,
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
