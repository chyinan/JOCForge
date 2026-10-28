// pattern: Imperative Shell
//! CLI composition root joining the in-memory reader cursor to application staging.

use jocforge_application::{
    PreparedJobInput, PreparedJobInputError, ProgrammeId, SelectedAdmResourcePayloads,
    SelectedWaveDescriptor,
    meter_source_bridge::{
        self as application_bridge, IndependentMeterPrepass, MeterCallbackPhase,
        MeterSourceBridgeError, MeterSourceCallbackFrame, MeterSourceCallbackFrameParts,
        MeterSourceCallbackView, MeterSourceIdentity, MeterSourceOarHandoff, MeterSourceOarReceipt,
        MeterSourcePosition,
    },
    prepare_selected_job_input,
};
use jocforge_io::pcm::{
    meter_source_cursor::{
        MeterPcmSourceCursor, MeterPcmSourceIdentity, MeterPcmSourceProposal,
        MeterSourceCursorError,
    },
    riff_pcm24::{ParsedRiffPcm24, RiffPcm24Error},
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedAdmPreparationError {
    MetadataSelection(RiffPcm24Error),
    PreparedJobInput(PreparedJobInputError),
    SourceIdentityMismatch,
}

/// Builds the admitted application input from metadata and PCM parsed out of one RIFF resource.
/// `prepare_selected_job_input` verifies that the selected metadata slices belong to `resource`;
/// the final digest comparison also binds the parser's PCM extent to that same whole resource.
pub fn prepare_selected_adm_bwf_input(
    programme_id: ProgrammeId,
    job_xml: &[u8],
    resource: Arc<[u8]>,
    parsed: &ParsedRiffPcm24<'_>,
) -> Result<PreparedJobInput, SelectedAdmPreparationError> {
    let metadata = parsed
        .selected_adm_payloads()
        .map_err(SelectedAdmPreparationError::MetadataSelection)?;
    let wave = SelectedWaveDescriptor::new(parsed.sample_rate_hz(), parsed.channels(), 24);
    let prepared = prepare_selected_job_input(
        programme_id,
        job_xml,
        wave,
        parsed.total_samples(),
        resource,
        SelectedAdmResourcePayloads::new(metadata.axml(), metadata.chna(), metadata.dbmd()),
    )
    .map_err(SelectedAdmPreparationError::PreparedJobInput)?;
    let resource_bytes = u64::try_from(prepared.resource().len())
        .map_err(|_| SelectedAdmPreparationError::SourceIdentityMismatch)?;
    if prepared.resource_identity().input_sha256() != parsed.source_sha256()
        || prepared.resource_identity().resource_bytes() != resource_bytes
    {
        return Err(SelectedAdmPreparationError::SourceIdentityMismatch);
    }
    Ok(prepared)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterSourceCoordinatorError {
    SourceCursor(MeterSourceCursorError),
    Bridge(MeterSourceBridgeError),
    ResourceIdentityMismatch,
    SourceExtentMismatch,
    CursorMismatch,
    PrepassClockMismatch,
}

pub fn view_for_source_proposal<'a>(
    prepared: &'a PreparedJobInput,
    proposal: &'a MeterPcmSourceProposal,
) -> Result<MeterSourceCallbackView<'a>, MeterSourceCoordinatorError> {
    let frame = proposal.frame();
    let after = proposal.after();
    let source_frame = MeterSourceCallbackFrame::new(MeterSourceCallbackFrameParts {
        callback_index: frame.callback_index(),
        upstream_generation: frame.upstream_generation(),
        phase: frame.phase(),
        source_identity: MeterSourceIdentity::from_sha256(after.source_identity().sha256()),
        source_samples: after.source_samples(),
        normal_callbacks: after.normal_callbacks(),
        source_cursor_before: MeterSourcePosition::new(frame.source_cursor_before().value()),
        source_cursor_after: MeterSourcePosition::new(frame.source_cursor_after().value()),
        host_input_count: frame.host_input_count(),
        source_padding_count: frame.source_padding_count(),
        rows: frame.rows(),
    });
    application_bridge::MeterSourceCallbackView::new(prepared, source_frame)
        .map_err(MeterSourceCoordinatorError::Bridge)
}

#[must_use]
pub struct MeterSourceOarStage<'a> {
    source_cursor: &'a mut MeterPcmSourceCursor,
    cursor_after: MeterPcmSourceCursor,
    application_stage: application_bridge::MeterSourceOarStage<'a>,
}

impl MeterSourceOarStage<'_> {
    /// Installs the source cursor after the application-owned meter stage commits.
    pub fn commit(
        self,
    ) -> (
        jocforge_application::meter_source_bridge::PostRenderMeterReceipt,
        MeterSourceOarReceipt,
    ) {
        let receipts = self.application_stage.commit();
        *self.source_cursor = self.cursor_after;
        receipts
    }
}

pub fn stage_meter_source_oar_callback<'a>(
    source_cursor: &'a mut MeterPcmSourceCursor,
    prepass: &'a mut IndependentMeterPrepass,
    proposal: MeterPcmSourceProposal,
    handoff: MeterSourceOarHandoff,
) -> Result<MeterSourceOarStage<'a>, MeterSourceCoordinatorError> {
    let (cursor_after, frame) = proposal.into_parts();
    let callback_index = frame.callback_index();
    let expected_next = callback_index
        .value()
        .checked_add(1)
        .ok_or(MeterSourceCoordinatorError::CursorMismatch)?;
    let receipt = handoff.source_receipt();

    if source_cursor.callback_index() != callback_index
        || source_cursor.source_cursor().value() != frame.source_cursor_before().value()
        || source_cursor.source_identity() != cursor_after.source_identity()
        || source_cursor.source_samples() != cursor_after.source_samples()
        || source_cursor.normal_callbacks() != cursor_after.normal_callbacks()
        || cursor_after.callback_index().value() != expected_next
        || cursor_after.source_cursor().value() != frame.source_cursor_after().value()
        || receipt.callback_index() != callback_index
        || receipt.upstream_generation().value() != frame.upstream_generation().value()
        || receipt.phase() != frame.phase()
        || receipt.source_cursor_before().value() != frame.source_cursor_before().value()
        || receipt.source_cursor_after().value() != frame.source_cursor_after().value()
        || receipt.source_samples() != cursor_after.source_samples()
        || receipt.normal_callbacks() != cursor_after.normal_callbacks()
        || receipt.host_input_count() != frame.host_input_count()
        || receipt.source_padding_count() != frame.source_padding_count()
        || receipt.resource_identity().sha256() != cursor_after.source_identity().sha256()
    {
        return Err(MeterSourceCoordinatorError::CursorMismatch);
    }
    if source_cursor.source_samples() != prepass.source_samples()
        || source_cursor.normal_callbacks() != prepass.normal_callback_count()
        || prepass.next_callback_index() != callback_index
    {
        return Err(MeterSourceCoordinatorError::PrepassClockMismatch);
    }
    let expected_phase = if callback_index.value() < source_cursor.normal_callbacks().value() {
        MeterCallbackPhase::NormalSource
    } else {
        MeterCallbackPhase::ZeroInputDrain
    };
    if frame.phase() != expected_phase {
        return Err(MeterSourceCoordinatorError::CursorMismatch);
    }

    let application_stage = application_bridge::stage_meter_source_oar_callback(prepass, handoff)
        .map_err(MeterSourceCoordinatorError::Bridge)?;
    Ok(MeterSourceOarStage {
        source_cursor,
        cursor_after,
        application_stage,
    })
}

pub fn reset_meter_source_and_prepass(
    source_cursor: &mut MeterPcmSourceCursor,
    prepass: &mut IndependentMeterPrepass,
    prepared: &PreparedJobInput,
    parsed_source: &ParsedRiffPcm24<'_>,
) -> Result<(), MeterSourceCoordinatorError> {
    let identity = MeterPcmSourceIdentity::from_selected_resource_sha256(
        prepared.resource_identity().input_sha256(),
    );
    if source_cursor.source_identity() != identity
        || parsed_source.source_sha256() != identity.sha256()
    {
        return Err(MeterSourceCoordinatorError::ResourceIdentityMismatch);
    }
    if parsed_source.total_samples() != source_cursor.source_samples().value()
        || prepass.source_samples() != source_cursor.source_samples()
    {
        return Err(MeterSourceCoordinatorError::SourceExtentMismatch);
    }
    if prepass.normal_callback_count() != source_cursor.normal_callbacks() {
        return Err(MeterSourceCoordinatorError::PrepassClockMismatch);
    }

    let cold_cursor =
        MeterPcmSourceCursor::cold(parsed_source, identity, source_cursor.source_samples())
            .map_err(MeterSourceCoordinatorError::SourceCursor)?;
    prepass.reset().map_err(|error| {
        MeterSourceCoordinatorError::Bridge(MeterSourceBridgeError::Prepass(error))
    })?;
    *source_cursor = cold_cursor;
    Ok(())
}

pub fn ready_to_finalize(
    source_cursor: &MeterPcmSourceCursor,
    prepass: &IndependentMeterPrepass,
) -> bool {
    application_bridge::ready_to_finalize(
        prepass,
        source_cursor.callback_index(),
        MeterSourcePosition::new(source_cursor.source_cursor().value()),
        source_cursor.source_samples(),
        source_cursor.normal_callbacks(),
    )
}
