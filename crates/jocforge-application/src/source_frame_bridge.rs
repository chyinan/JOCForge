// pattern: Functional Core
//! Binds admitted input, same-source F06ND/F13 proposals, and the selected F12 frame candidate.

use std::sync::Arc;

pub use jocforge_domain::codec::{
    owner::{CodecOwnerProposal, CodecOwnerRoot, CodecOwnerStageError},
    source_frame_writer::SelectedSourceFrameCandidate,
};
pub use jocforge_domain::{
    analysis::owner::AnalysisNumericPlanStage, drc::NoneDrcOwnerAfterProposal,
    joc::emdf::metadata_owner::MetadataSourceProposal,
};

use jocforge_domain::{
    analysis::source_frame::SharedAnalysisReceipt,
    codec::{
        bap::BapParameters,
        published_emdf_binding::{PublishedEmdfBindingError, emdf_frame_input_for_source},
        source_frame_writer::{
            SelectedSourceFrameWriterError, prepare_selected_source_frame_candidate,
        },
    },
    publication::OutputSourceAfterProposal,
    timeline::{PublicationDisposition, publication_for_source, source_for_callback},
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
    F12EmdfCursorMissing,
    F12EmdfCursorUnexpected,
    CodecSource(CodecSourceBridgeError),
    PublishedEmdf(PublishedEmdfBindingError),
    FrameWriter(SelectedSourceFrameWriterError),
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
