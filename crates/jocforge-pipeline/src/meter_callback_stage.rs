// pattern: Imperative Shell
//! Noncommitting probe of MAIN spatial outputs through one Meter OAR callback.

use jocforge_application::{
    PreparedJobInput,
    meter_source_bridge::{
        FrontendCallbackIndex, FrontendControlReplacementCandidate, GtcOwnerId, MeterCallbackCount,
        MeterCallbackIndex, MeterCallbackPhase, MeterComponentOutputRows,
        MeterRootCallbackProgress, MeterSampleCount, MeterSourceBridgeError,
        MeterSourceCallbackFrame, MeterSourceCallbackView, MeterSourceOarReceipt,
        MeterSourcePosition, MeterSourceSampleCount, SpatialAudioCallbackOutcome,
        SpatialAudioReplacementCandidate, SpatialControlReplacementCandidate,
        StandaloneMeterCallbackReceipt, StandaloneMeterIdentity, StandaloneMeterRoot,
        StandaloneMeterRootError,
    },
};
use jocforge_io::pcm::meter_source_cursor::{MeterPcmSourceCursor, MeterPcmSourceProposal};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterCallbackStageError {
    Source(MeterSourceStageError),
    SourceBridge(MeterSourceBridgeError),
    UnsupportedInitialGeneration,
    PreparedIdentityMismatch,
    SourceCursorMismatch,
    FrontendLineageMismatch,
    SpatialLineageMismatch,
    MeterIdentityMismatch,
    MeterRoot(StandaloneMeterRootError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterSourceStageError {
    ResourceIdentityMismatch,
    SourceExtentMismatch,
    SourceReadIndexMismatch,
    SourceShapeMismatch,
}

/// Candidate values from the initial MAIN-output-to-OAR probe.
///
/// This verifies a same-resource OAR handoff and one standalone meter callback
/// root proposal. Its component and target rows come from the MAIN spatial
/// producer; it does not satisfy the independent cold-producer prepass contract.
/// The full V4 root remains noncommitting, and this candidate has no production
/// measurement receipt, READY, or commit authority.
#[must_use]
pub struct MainOutputOarMeterProbeCandidate {
    source_cursor_after: MeterPcmSourceCursor,
    frontend_candidate: FrontendControlReplacementCandidate,
    spatial_control_candidate: SpatialControlReplacementCandidate,
    spatial_audio_candidate: SpatialAudioReplacementCandidate,
    component_rows: MeterComponentOutputRows,
    spatial_audio_outcome: SpatialAudioCallbackOutcome,
    meter_after: StandaloneMeterRoot,
    source_receipt: MeterSourceOarReceipt,
    meter_receipt: StandaloneMeterCallbackReceipt,
}

impl MainOutputOarMeterProbeCandidate {
    pub const fn source_cursor_after(&self) -> &MeterPcmSourceCursor {
        &self.source_cursor_after
    }

    pub const fn frontend_candidate(&self) -> &FrontendControlReplacementCandidate {
        &self.frontend_candidate
    }

    pub const fn spatial_control_candidate(&self) -> &SpatialControlReplacementCandidate {
        &self.spatial_control_candidate
    }

    pub const fn spatial_audio_candidate(&self) -> &SpatialAudioReplacementCandidate {
        &self.spatial_audio_candidate
    }

    pub const fn component_rows(&self) -> &MeterComponentOutputRows {
        &self.component_rows
    }

    pub const fn spatial_audio_outcome(&self) -> &SpatialAudioCallbackOutcome {
        &self.spatial_audio_outcome
    }

    pub const fn meter_after(&self) -> &StandaloneMeterRoot {
        &self.meter_after
    }

    pub const fn source_receipt(&self) -> &MeterSourceOarReceipt {
        &self.source_receipt
    }

    pub const fn meter_receipt(&self) -> &StandaloneMeterCallbackReceipt {
        &self.meter_receipt
    }

    pub fn into_parts(self) -> MainOutputOarMeterProbeParts {
        MainOutputOarMeterProbeParts {
            source_cursor_after: self.source_cursor_after,
            frontend_candidate: self.frontend_candidate,
            spatial_control_candidate: self.spatial_control_candidate,
            spatial_audio_candidate: self.spatial_audio_candidate,
            component_rows: self.component_rows,
            spatial_audio_outcome: self.spatial_audio_outcome,
            meter_after: self.meter_after,
            source_receipt: self.source_receipt,
            meter_receipt: self.meter_receipt,
        }
    }
}

#[must_use]
pub struct MainOutputOarMeterProbeParts {
    pub source_cursor_after: MeterPcmSourceCursor,
    pub frontend_candidate: FrontendControlReplacementCandidate,
    pub spatial_control_candidate: SpatialControlReplacementCandidate,
    pub spatial_audio_candidate: SpatialAudioReplacementCandidate,
    pub component_rows: MeterComponentOutputRows,
    pub spatial_audio_outcome: SpatialAudioCallbackOutcome,
    pub meter_after: StandaloneMeterRoot,
    pub source_receipt: MeterSourceOarReceipt,
    pub meter_receipt: StandaloneMeterCallbackReceipt,
}

/// Inputs staged together for the MAIN-output-to-OAR probe.
pub struct MainOutputOarMeterProbeInput<'a> {
    pub prepared: &'a PreparedJobInput,
    pub source_cursor_before: &'a MeterPcmSourceCursor,
    pub source_proposal: MeterPcmSourceProposal,
    pub frontend_candidate: FrontendControlReplacementCandidate,
    pub spatial_control_candidate: SpatialControlReplacementCandidate,
    pub spatial_audio_candidate: SpatialAudioReplacementCandidate,
    pub component_rows: MeterComponentOutputRows,
    pub spatial_audio_outcome: SpatialAudioCallbackOutcome,
    pub meter_before: &'a StandaloneMeterRoot,
    /// Parent-supplied source-read and K-filter progress; never derived here.
    pub meter_progress: MeterRootCallbackProgress,
}

/// Probes source callback zero from admitted MAIN spatial values through OAR and
/// one standalone meter callback. This is not a production independent-prepass
/// entry point because it does not create fresh meter-owned frontend/spatial rows.
pub fn stage_main_output_oar_meter_probe(
    input: MainOutputOarMeterProbeInput<'_>,
) -> Result<MainOutputOarMeterProbeCandidate, MeterCallbackStageError> {
    let MainOutputOarMeterProbeInput {
        prepared,
        source_cursor_before,
        source_proposal,
        frontend_candidate,
        spatial_control_candidate,
        spatial_audio_candidate,
        component_rows,
        spatial_audio_outcome,
        meter_before,
        meter_progress,
    } = input;
    let reader_frame = source_proposal.frame();
    let cursor_after = source_proposal.after();
    let callback_index = reader_frame.callback_index();
    if callback_index != MeterCallbackIndex::new(0) {
        return Err(MeterCallbackStageError::UnsupportedInitialGeneration);
    }

    let source_frame = super::meter_source_bridge::source_frame_for_proposal(&source_proposal);
    let source_view = MeterSourceCallbackView::new(prepared, source_frame)
        .map_err(MeterCallbackStageError::SourceBridge)?;
    validate_source_cursor(prepared, source_cursor_before, cursor_after, &source_frame)?;
    validate_frontend_lineage(prepared, meter_before.identity(), &frontend_candidate)?;
    validate_spatial_lineage(
        prepared,
        meter_before.identity(),
        &spatial_control_candidate,
        &spatial_audio_candidate,
        &component_rows,
        &spatial_audio_outcome,
    )?;
    validate_meter_identity(prepared, meter_before, source_frame.normal_callbacks())?;

    let handoff = source_view
        .bind_spatial_outputs(
            &component_rows,
            spatial_control_candidate.outputs().reader_target_controls(),
        )
        .map_err(|_| MeterCallbackStageError::Source(MeterSourceStageError::SourceShapeMismatch))?;
    let (meter_input, source_receipt) = handoff.into_parts();
    validate_source_receipt(
        &source_receipt,
        callback_index,
        source_frame.upstream_generation().value(),
        source_frame.phase(),
        source_frame.source_cursor_before(),
        source_frame.source_cursor_after(),
        source_frame.source_samples(),
        source_frame.normal_callbacks(),
        source_frame.host_input_count(),
        source_frame.source_padding_count(),
        prepared.resource_identity().input_sha256(),
    )?;

    let meter_proposal = meter_before
        .prepare_callback(meter_input, meter_progress)
        .map_err(MeterCallbackStageError::MeterRoot)?;
    let (meter_after, meter_receipt) = meter_proposal.into_parts();
    let expected_next = MeterCallbackIndex::new(1);
    if meter_receipt.callback_index() != callback_index
        || meter_receipt.next_callback_index() != expected_next
        || meter_receipt.identity() != meter_before.identity()
        || meter_after.meter_callback_index() != expected_next
    {
        return Err(MeterCallbackStageError::MeterIdentityMismatch);
    }

    let (source_cursor_after, _source_frame) = source_proposal.into_parts();
    Ok(MainOutputOarMeterProbeCandidate {
        source_cursor_after,
        frontend_candidate,
        spatial_control_candidate,
        spatial_audio_candidate,
        component_rows,
        spatial_audio_outcome,
        meter_after,
        source_receipt,
        meter_receipt,
    })
}

fn validate_source_cursor(
    prepared: &PreparedJobInput,
    cursor_before: &MeterPcmSourceCursor,
    cursor_after: &MeterPcmSourceCursor,
    frame: &MeterSourceCallbackFrame<'_>,
) -> Result<(), MeterCallbackStageError> {
    let prepared_hash = prepared.resource_identity().input_sha256();
    let expected_samples = prepared.configuration().source_samples();
    if cursor_before.source_identity().sha256() != prepared_hash
        || cursor_after.source_identity() != cursor_before.source_identity()
    {
        return Err(MeterCallbackStageError::Source(
            MeterSourceStageError::ResourceIdentityMismatch,
        ));
    }
    if cursor_before.source_samples().value() != expected_samples
        || cursor_after.source_samples() != cursor_before.source_samples()
        || cursor_before.normal_callbacks() != cursor_after.normal_callbacks()
        || cursor_before.normal_callbacks().value() != frame.normal_callbacks().value()
    {
        return Err(MeterCallbackStageError::Source(
            MeterSourceStageError::SourceExtentMismatch,
        ));
    }
    let expected_source_after = cursor_before
        .source_cursor()
        .value()
        .checked_add(frame.host_input_count().value())
        .ok_or(MeterCallbackStageError::SourceCursorMismatch)?;
    if cursor_before.read_index() != frame.source_read_index_before()
        || cursor_after.read_index() != frame.source_read_index_after()
    {
        return Err(MeterCallbackStageError::Source(
            MeterSourceStageError::SourceReadIndexMismatch,
        ));
    }
    if cursor_before.callback_index() != frame.callback_index()
        || cursor_before.source_cursor().value() != frame.source_cursor_before().value()
        || cursor_after.callback_index().value() != 1
        || cursor_after.source_cursor().value() != expected_source_after
        || cursor_after.source_cursor().value() != frame.source_cursor_after().value()
        || frame.phase() != MeterCallbackPhase::NormalSource
        || frame.host_input_count().value() != 1_536
        || frame.source_padding_count().value() != 0
    {
        return Err(MeterCallbackStageError::SourceCursorMismatch);
    }
    Ok(())
}

fn validate_frontend_lineage(
    prepared: &PreparedJobInput,
    meter_identity: &StandaloneMeterIdentity,
    frontend_candidate: &FrontendControlReplacementCandidate,
) -> Result<(), MeterCallbackStageError> {
    let receipt = frontend_candidate.receipt();
    let next_frontend = frontend_candidate.next_machine();
    let Some(prepared_identity) = next_frontend.committed().prepared_identity() else {
        return Err(MeterCallbackStageError::FrontendLineageMismatch);
    };
    let binding = prepared
        .primary_control_binding()
        .ok_or(MeterCallbackStageError::PreparedIdentityMismatch)?;
    if !prepared_identity.matches_selected_input(binding, prepared.graph().graph(), prepared.dbmd())
    {
        return Err(MeterCallbackStageError::PreparedIdentityMismatch);
    }
    if next_frontend.committed().programme_id() != prepared.programme_id()
        || prepared.programme_id() != meter_identity.programme_id()
        || next_frontend.committed().attempt_id() != meter_identity.attempt()
        || next_frontend.committed().callback_index() != FrontendCallbackIndex::new(1)
        || receipt.owner_id() != GtcOwnerId::FrontendControl
        || receipt.before_generation() != 0
        || receipt.after_generation() != 1
        || !receipt.changed()
        || receipt.commit_count() != 1
        || next_frontend.generation() != 1
        || next_frontend.committed().programme_id() != prepared.programme_id()
        || next_frontend.committed().attempt_id() != meter_identity.attempt()
        || next_frontend.committed().callback_index() != FrontendCallbackIndex::new(1)
    {
        return Err(MeterCallbackStageError::FrontendLineageMismatch);
    }
    Ok(())
}

fn validate_spatial_lineage(
    prepared: &PreparedJobInput,
    meter_identity: &StandaloneMeterIdentity,
    spatial_control: &SpatialControlReplacementCandidate,
    spatial_audio: &SpatialAudioReplacementCandidate,
    component_rows: &MeterComponentOutputRows,
    audio_outcome: &SpatialAudioCallbackOutcome,
) -> Result<(), MeterCallbackStageError> {
    let callback = FrontendCallbackIndex::new(0);
    let target_controls = spatial_control.outputs().reader_target_controls();
    let spatial_transaction = target_controls.transaction_id();
    let control_receipt = spatial_control.receipt();
    let audio_receipt = spatial_audio.receipt();
    if control_receipt.owner_id() != GtcOwnerId::SpatialControl
        || control_receipt.before_generation() != 0
        || control_receipt.after_generation() != 1
        || !control_receipt.changed()
        || control_receipt.commit_count() != 1
        || spatial_control.next_machine().generation() != 1
        || spatial_control.next_machine().committed().callback_index()
            != FrontendCallbackIndex::new(1)
        || target_controls.callback_index() != callback
        || spatial_transaction.programme_id() != prepared.programme_id()
        || spatial_transaction.programme_id() != meter_identity.programme_id()
        || spatial_transaction.attempt_id() != meter_identity.attempt()
        || spatial_transaction.callback_index() != callback
        || component_rows.callback_index() != callback
        || audio_receipt.owner_id() != GtcOwnerId::SpatialAudio
        || audio_receipt.before_generation() != 0
        || audio_receipt.after_generation() != 1
        || !audio_receipt.changed()
        || audio_receipt.commit_count() != 1
        || spatial_audio.next_machine().generation() != 1
        || spatial_audio.next_machine().committed().callback_index()
            != FrontendCallbackIndex::new(1)
        || !matches!(audio_outcome, SpatialAudioCallbackOutcome::Priming)
    {
        return Err(MeterCallbackStageError::SpatialLineageMismatch);
    }
    Ok(())
}

fn validate_meter_identity(
    prepared: &PreparedJobInput,
    meter_before: &StandaloneMeterRoot,
    normal_callbacks: MeterCallbackCount,
) -> Result<(), MeterCallbackStageError> {
    let identity = meter_before.identity();
    if identity.programme_id() != prepared.programme_id()
        || identity.input_sha256() != &prepared.resource_identity().input_sha256()
        || identity.config_sha256() != &prepared.config_sha256()
        || meter_before.source_samples().value() != prepared.configuration().source_samples()
        || meter_before.normal_callback_count() != normal_callbacks
        || meter_before.meter_callback_index() != MeterCallbackIndex::new(0)
    {
        return Err(MeterCallbackStageError::MeterIdentityMismatch);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_source_receipt(
    receipt: &MeterSourceOarReceipt,
    callback_index: MeterCallbackIndex,
    upstream_generation: u64,
    phase: MeterCallbackPhase,
    cursor_before: MeterSourcePosition,
    cursor_after: MeterSourcePosition,
    source_samples: MeterSourceSampleCount,
    normal_callbacks: MeterCallbackCount,
    host_input_count: MeterSampleCount,
    source_padding_count: MeterSampleCount,
    input_sha256: [u8; 32],
) -> Result<(), MeterCallbackStageError> {
    if receipt.callback_index() != callback_index
        || receipt.upstream_generation().value() != upstream_generation
        || receipt.phase() != phase
        || receipt.source_cursor_before() != cursor_before
        || receipt.source_cursor_after() != cursor_after
        || receipt.source_samples() != source_samples
        || receipt.normal_callbacks() != normal_callbacks
        || receipt.host_input_count() != host_input_count
        || receipt.source_padding_count() != source_padding_count
        || receipt.resource_identity().sha256() != input_sha256
        || receipt.spatial_callback_index() != Some(FrontendCallbackIndex::new(0))
        || receipt.target_control_lineages().is_none()
    {
        return Err(MeterCallbackStageError::Source(
            MeterSourceStageError::SourceShapeMismatch,
        ));
    }
    Ok(())
}
