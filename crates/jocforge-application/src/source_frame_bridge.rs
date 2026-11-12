// pattern: Functional Core
//! Binds admitted input, same-source F06ND/F13 proposals, and the selected F12 frame candidate.

use std::sync::Arc;

pub use jocforge_domain::codec::{
    owner::{CodecOwnerProposal, CodecOwnerRoot, CodecOwnerStageError},
    source_frame_writer::SelectedSourceFrameCandidate,
};
pub use jocforge_domain::transaction::CallbackLineageIdentity;
pub use jocforge_domain::{
    analysis::owner::AnalysisNumericPlanStage, drc::NoneDrcOwnerAfterProposal,
    joc::emdf::metadata_owner::MetadataSourceProposal,
};

use jocforge_domain::{
    analysis::source_frame::{SharedAnalysisIdentity, SharedAnalysisReceipt},
    codec::{
        bap::BapParameters,
        published_emdf_binding::{PublishedEmdfBindingError, emdf_frame_input_for_source},
        source_frame_writer::{
            SelectedSourceFrameWriterError, prepare_selected_source_frame_candidate,
        },
    },
    profile::{
        Flat7xD0PcmRows, Flat7xI0D0TargetPlan, Flat7xI0PcmRows, Flat7xProvisionalSourcePcmView,
        Flat7xProvisionalSourceRoleMap, Flat7xSourceRoleMapError,
        build_flat7x_provisional_source_role_map,
        project_flat7x_source_pcm_rows as project_profile_source_pcm_rows,
    },
    publication::OutputSourceAfterProposal,
    timeline::{
        CodecSourceIndex, FrontendCallbackIndex, PublicationDisposition, publication_for_source,
        source_for_callback,
    },
    transaction::{AttemptId, CallbackTransactionId, ProgrammeId},
};

use crate::{
    PreparedJobInput,
    codec_source_bridge::{CodecSourceBridgeError, bind_selected_codec_bsi_source},
};

/// Failures while joining the current source's application-owned codec inputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceFrameBridgeError {
    F15AdmissionRejected,
    AnalysisSharedReceiptMissing,
    MetadataSourceLineageMismatch,
    Flat7xSourceRoleMap(Flat7xSourceRoleMapError),
    Flat7xSourceRoleMapLineageMismatch,
    F12EmdfCursorMissing,
    F12EmdfCursorUnexpected,
    CodecSource(CodecSourceBridgeError),
    PublishedEmdf(PublishedEmdfBindingError),
    FrameWriter(SelectedSourceFrameWriterError),
}

/// Source-generation-bound receipt for the replaceable Flat7x ADM-row role projection.
///
/// The map keeps physical PCM rows separate from I0 carrier roles and D0 `chanmap` locations.
/// Its row-to-role policy is provisional; the identity fields bind this exact projection to the
/// prepared input, attempt, callback, source transaction, and selected F07/F12 source generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xSourceRoleMapReceipt {
    prepared_identity: Arc<jocforge_domain::meter::prepared_job_identity::PreparedJobInputIdentity>,
    programme_id: ProgrammeId,
    attempt_id: AttemptId,
    callback_index: FrontendCallbackIndex,
    source_index: CodecSourceIndex,
    source_transaction_id: CallbackTransactionId,
    shared_analysis_identity: SharedAnalysisIdentity,
    target: Flat7xI0D0TargetPlan,
    role_map: Flat7xProvisionalSourceRoleMap,
}

impl Flat7xSourceRoleMapReceipt {
    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub fn source_transaction_id(&self) -> &CallbackTransactionId {
        &self.source_transaction_id
    }

    pub const fn target(&self) -> Flat7xI0D0TargetPlan {
        self.target
    }

    pub const fn role_map(&self) -> &Flat7xProvisionalSourceRoleMap {
        &self.role_map
    }

    pub const fn shared_analysis_identity(&self) -> &SharedAnalysisIdentity {
        &self.shared_analysis_identity
    }

    pub fn is_bound_to_source(
        &self,
        prepared: &PreparedJobInput,
        programme_id: &ProgrammeId,
        attempt_id: AttemptId,
        callback_lineage: &CallbackLineageIdentity,
        source_index: CodecSourceIndex,
        source_transaction_id: &CallbackTransactionId,
    ) -> bool {
        let identity_capability = prepared.domain_identity_capability();
        let source_key = self.shared_analysis_identity.source_stage_key();
        &self.programme_id == programme_id
            && prepared.programme_id() == programme_id
            && self.attempt_id == attempt_id
            && callback_lineage.programme_id() == programme_id
            && callback_lineage.attempt_id() == attempt_id
            && callback_lineage.callback_index() == self.callback_index
            && self.source_index == source_index
            && &self.source_transaction_id == source_transaction_id
            && self.shared_analysis_identity.source_index() == source_index
            && source_key.attempt_id() == attempt_id
            && source_key.transaction_id() == source_transaction_id
            && source_key.source_index() == source_index
            && source_key
                .prepared_job_input_identity()
                .is_some_and(|identity| Arc::ptr_eq(identity, &identity_capability))
            && Arc::ptr_eq(&self.prepared_identity, &identity_capability)
            && usize::from(self.role_map.physical_row_count())
                == usize::from(prepared.configuration().wave().channels())
    }
}

/// Same-source projection of the actual callback PCM rows through the provisional role map.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xSourcePcmProjection<'a> {
    role_map_receipt: Flat7xSourceRoleMapReceipt,
    pcm_view: Flat7xProvisionalSourcePcmView<'a>,
}

impl Flat7xSourcePcmProjection<'_> {
    pub const fn role_map_receipt(&self) -> &Flat7xSourceRoleMapReceipt {
        &self.role_map_receipt
    }

    pub const fn i0_rows(&self) -> Flat7xI0PcmRows<'_> {
        self.pcm_view.i0()
    }

    pub const fn d0_rows(&self) -> Flat7xD0PcmRows<'_> {
        self.pcm_view.d0()
    }

    pub const fn physical_row_count(&self) -> u16 {
        self.pcm_view.physical_row_count()
    }
}

/// Builds the profile-specific provisional ADM-row map and binds it to one already staged
/// same-source analysis result. This is a value-only producer seam; it does not install state.
pub fn bind_flat7x_source_role_map(
    prepared: &PreparedJobInput,
    target: Flat7xI0D0TargetPlan,
    callback_lineage: &CallbackLineageIdentity,
    source_index: CodecSourceIndex,
    source_transaction_id: &CallbackTransactionId,
    shared_analysis_identity: &SharedAnalysisIdentity,
) -> Result<Flat7xSourceRoleMapReceipt, SourceFrameBridgeError> {
    let prepared_identity = prepared.domain_identity_capability();
    let source_key = shared_analysis_identity.source_stage_key();
    if callback_lineage.programme_id() != prepared.programme_id()
        || callback_lineage.attempt_id() != source_key.attempt_id()
        || source_index != shared_analysis_identity.source_index()
        || source_key.source_index() != source_index
        || source_key.transaction_id() != source_transaction_id
        || source_key
            .prepared_job_input_identity()
            .is_none_or(|identity| !Arc::ptr_eq(identity, &prepared_identity))
    {
        return Err(SourceFrameBridgeError::Flat7xSourceRoleMapLineageMismatch);
    }
    let role_map = build_flat7x_provisional_source_role_map(prepared.graph().graph(), target)
        .map_err(SourceFrameBridgeError::Flat7xSourceRoleMap)?;
    if usize::from(role_map.physical_row_count())
        != usize::from(prepared.configuration().wave().channels())
    {
        return Err(SourceFrameBridgeError::Flat7xSourceRoleMapLineageMismatch);
    }

    Ok(Flat7xSourceRoleMapReceipt {
        prepared_identity,
        programme_id: prepared.programme_id().clone(),
        attempt_id: source_key.attempt_id(),
        callback_index: callback_lineage.callback_index(),
        source_index,
        source_transaction_id: source_transaction_id.clone(),
        shared_analysis_identity: shared_analysis_identity.clone(),
        target,
        role_map,
    })
}

/// Projects this callback's physical PCM rows and retains their source-bound role receipt for
/// downstream I0/D0 producer stages.
pub fn build_flat7x_source_pcm_projection<'a>(
    prepared: &PreparedJobInput,
    target: Flat7xI0D0TargetPlan,
    callback_lineage: &CallbackLineageIdentity,
    source_index: CodecSourceIndex,
    source_transaction_id: &CallbackTransactionId,
    shared_analysis_identity: &SharedAnalysisIdentity,
    physical_pcm_rows: &'a [Vec<u32>],
) -> Result<Flat7xSourcePcmProjection<'a>, SourceFrameBridgeError> {
    let role_map_receipt = bind_flat7x_source_role_map(
        prepared,
        target,
        callback_lineage,
        source_index,
        source_transaction_id,
        shared_analysis_identity,
    )?;
    let pcm_view = project_profile_source_pcm_rows(role_map_receipt.role_map(), physical_pcm_rows)
        .map_err(SourceFrameBridgeError::Flat7xSourceRoleMap)?;
    Ok(Flat7xSourcePcmProjection {
        role_map_receipt,
        pcm_view,
    })
}

/// Prepares a selected F12 frame candidate using the BAP value retained by this request's F15
/// admission. The caller supplies the same-source Analysis, DRC and Metadata proposals produced
/// by the approved callback stages; this bridge adds no codec behavior or owner commit.
pub fn prepare_admitted_source_frame<'a>(
    prepared: &PreparedJobInput,
    admitted_plan: &crate::output_publication::AdmittedProductionPlan,
    analysis_stage: &'a mut AnalysisNumericPlanStage,
    drc_after: &NoneDrcOwnerAfterProposal,
    metadata: &'a MetadataSourceProposal,
) -> Result<SelectedSourceFrameCandidate<'a>, SourceFrameBridgeError> {
    if !admitted_plan.is_bound_to(prepared) {
        return Err(SourceFrameBridgeError::F15AdmissionRejected);
    }
    let bap_parameters = Arc::new(admitted_plan.source_providers().bap().clone());
    prepare_source_frame_from_proposals(
        prepared,
        analysis_stage,
        drc_after,
        metadata,
        bap_parameters,
    )
}

/// Binds real production proposals for one source into a complete, noncommitting F12 candidate.
///
/// The metadata proposal is required for suppressed sources too: those sources still advance
/// F13's producer/timer/queue state even though they carry no published EMDF or AU. This function
/// validates that receipt against the exact `SharedAnalysisReceipt`, then delegates BSI/EMDF
/// validation and candidate assembly to the existing typed bridges. It performs no owner commit,
/// root swap, output append, or filesystem operation.
pub fn prepare_source_frame_from_proposals<'a>(
    prepared: &PreparedJobInput,
    analysis_stage: &'a mut AnalysisNumericPlanStage,
    drc_after: &NoneDrcOwnerAfterProposal,
    metadata: &'a MetadataSourceProposal,
    bap_parameters: Arc<BapParameters>,
) -> Result<SelectedSourceFrameCandidate<'a>, SourceFrameBridgeError> {
    let shared_analysis = analysis_stage
        .shared_analysis()
        .ok_or(SourceFrameBridgeError::AnalysisSharedReceiptMissing)?;
    validate_metadata_source_lineage(prepared, shared_analysis, drc_after, metadata)?;

    let bsi = bind_selected_codec_bsi_source(prepared, shared_analysis, drc_after)
        .map_err(SourceFrameBridgeError::CodecSource)?;
    let prepared_identity = prepared.domain_identity_capability();
    let emdf = emdf_frame_input_for_source(
        metadata.parsed_container(),
        shared_analysis,
        prepared.programme_id(),
        &prepared_identity,
    )
    .map_err(SourceFrameBridgeError::PublishedEmdf)?;

    let candidate = prepare_selected_source_frame_candidate(
        analysis_stage,
        prepared.configuration().selected_profile().codec_rate(),
        &bsi,
        &emdf,
        bap_parameters,
    )
    .map_err(SourceFrameBridgeError::FrameWriter)?;
    validate_f12_emdf_cursor(&candidate)?;
    Ok(candidate)
}

fn validate_f12_emdf_cursor(
    candidate: &SelectedSourceFrameCandidate<'_>,
) -> Result<(), SourceFrameBridgeError> {
    use jocforge_domain::codec::selected_frame_trial::EmdfFrameInput;

    let published = matches!(candidate.emdf_input(), EmdfFrameInput::Published(_));
    validate_f12_cursor_shape(published, candidate.frame().emdf_container_start())
}

fn validate_f12_cursor_shape(
    published: bool,
    cursor: Option<jocforge_domain::codec::bitio::FrameBitCursor>,
) -> Result<(), SourceFrameBridgeError> {
    match (published, cursor) {
        (false, None) => Ok(()),
        (false, Some(_)) => Err(SourceFrameBridgeError::F12EmdfCursorUnexpected),
        (true, None) => Err(SourceFrameBridgeError::F12EmdfCursorMissing),
        (true, Some(_)) => Ok(()),
    }
}

/// Stages the codec-owner proposal from the exact F12 candidate and its output disposition.
///
/// The short flags are taken from the retained numeric plan that produced this candidate. A
/// published source therefore needs the matching durable output after-proposal; suppressed
/// sources pass no output proposal. The function only stages an immutable proposal and leaves
/// installation to the parent callback transaction.
pub fn stage_codec_owner_after_source_frame(
    codec_owner: &CodecOwnerRoot,
    candidate: &SelectedSourceFrameCandidate<'_>,
    output_after: Option<&OutputSourceAfterProposal>,
) -> Result<CodecOwnerProposal, CodecOwnerStageError> {
    let short_flags = candidate.codec_short_flags();
    let accepted_frame = candidate.accepted_codec_frame();
    codec_owner.stage_source(&accepted_frame, &short_flags, output_after)
}

fn validate_metadata_source_lineage(
    prepared: &PreparedJobInput,
    shared_analysis: &SharedAnalysisReceipt,
    drc_after: &NoneDrcOwnerAfterProposal,
    metadata: &MetadataSourceProposal,
) -> Result<(), SourceFrameBridgeError> {
    let shared = shared_analysis.identity();
    let source_key = shared.source_stage_key();
    let source_index = shared.source_index();
    let metadata_receipt = metadata.receipt();
    let expected_dispatch = source_index
        .dispatch()
        .map_err(|_| SourceFrameBridgeError::MetadataSourceLineageMismatch)?;
    let expected_au = match publication_for_source(source_index)
        .map_err(|_| SourceFrameBridgeError::MetadataSourceLineageMismatch)?
    {
        PublicationDisposition::Suppressed { .. } => None,
        PublicationDisposition::Published(consumption) => Some(consumption.au),
    };
    let metadata_callback = metadata_receipt.transaction_id();
    let prepared_identity = prepared.domain_identity_capability();
    let metadata_id1 = metadata.id1_configuration();
    let Some(drc_prepared_identity) = drc_after.prepared_job_input_identity() else {
        return Err(SourceFrameBridgeError::MetadataSourceLineageMismatch);
    };
    let Some(analysis_prepared_identity) = shared.prepared_job_input_identity() else {
        return Err(SourceFrameBridgeError::MetadataSourceLineageMismatch);
    };

    if metadata_receipt.programme_id() != prepared.programme_id()
        || metadata_callback.programme_id() != prepared.programme_id()
        || metadata_receipt.attempt() != source_key.attempt_id()
        || metadata_callback.attempt_id() != source_key.attempt_id()
        || metadata_callback != drc_after.callback_identity()
        || source_for_callback(metadata_callback.callback_index()) != Some(source_index)
        || metadata_receipt.source_index() != source_index
        || metadata_receipt.dispatch() != expected_dispatch
        || metadata_receipt.au_index() != expected_au
        || metadata_receipt.consumer_transaction_id() != Some(source_key.transaction_id())
        || !Arc::ptr_eq(drc_prepared_identity, &prepared_identity)
        || !Arc::ptr_eq(analysis_prepared_identity, &prepared_identity)
        || metadata_id1.programme_id() != prepared.programme_id()
        || metadata_id1.attempt() != source_key.attempt_id()
        || metadata_id1.measurement_input_sha256()
            != &prepared.domain_identity().resource().input_sha256()
        || metadata_id1.measurement_config_sha256()
            != &prepared
                .domain_identity()
                .normalized_configuration()
                .config_sha256()
        || metadata_id1.meter_terminal_sha256() != drc_after.meter_terminal_sha256()
    {
        return Err(SourceFrameBridgeError::MetadataSourceLineageMismatch);
    }

    let proposal_transaction = metadata.consumer_transaction_id();
    let proposal_has_published_container = metadata.parsed_container().is_some();
    let proposal_has_emdf_bytes = metadata.emdf().is_some();
    if proposal_transaction != Some(source_key.transaction_id())
        || proposal_has_published_container != expected_au.is_some()
        || proposal_has_emdf_bytes != expected_au.is_some()
    {
        return Err(SourceFrameBridgeError::MetadataSourceLineageMismatch);
    }

    Ok(())
}

#[cfg(test)]
mod integrity_cursor_tests {
    use super::*;
    use jocforge_domain::codec::bitio::FrameBitCursor;

    #[test]
    fn published_f12_cursor_must_be_present_but_may_be_bit_aligned() {
        assert_eq!(
            validate_f12_cursor_shape(true, Some(FrameBitCursor::new(24))),
            Ok(())
        );
        assert_eq!(
            validate_f12_cursor_shape(true, Some(FrameBitCursor::new(25))),
            Ok(())
        );
        assert_eq!(
            validate_f12_cursor_shape(true, None),
            Err(SourceFrameBridgeError::F12EmdfCursorMissing)
        );
    }

    #[test]
    fn absent_f12_payload_has_no_carrier_cursor() {
        assert_eq!(validate_f12_cursor_shape(false, None), Ok(()));
        assert_eq!(
            validate_f12_cursor_shape(false, Some(FrameBitCursor::new(24))),
            Err(SourceFrameBridgeError::F12EmdfCursorUnexpected)
        );
    }
}
