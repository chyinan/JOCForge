// pattern: Functional Core
//! Binds admitted input, same-source F06ND/F13 proposals, and the selected F12 frame candidate.

use std::sync::Arc;

use jocforge_domain::{
    analysis::{owner::AnalysisNumericPlanStage, source_frame::SharedAnalysisReceipt},
    codec::{
        bap::BapParameters,
        published_emdf_binding::{PublishedEmdfBindingError, emdf_frame_input_for_source},
        source_frame_writer::{
            SelectedSourceFrameCandidate, SelectedSourceFrameWriterError,
            prepare_selected_source_frame_candidate,
        },
    },
    drc::NoneDrcOwnerAfterProposal,
    joc::emdf::metadata_owner::MetadataSourceProposal,
    timeline::{PublicationDisposition, publication_for_source, source_for_callback},
};

use crate::{
    PreparedJobInput,
    codec_source_bridge::{CodecSourceBridgeError, bind_selected_codec_bsi_source},
};

/// Failures while joining the current source's application-owned codec inputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceFrameBridgeError {
    AnalysisSharedReceiptMissing,
    MetadataSourceLineageMismatch,
    CodecSource(CodecSourceBridgeError),
    PublishedEmdf(PublishedEmdfBindingError),
    FrameWriter(SelectedSourceFrameWriterError),
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

    prepare_selected_source_frame_candidate(analysis_stage, &bsi, &emdf, bap_parameters)
        .map_err(SourceFrameBridgeError::FrameWriter)
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
