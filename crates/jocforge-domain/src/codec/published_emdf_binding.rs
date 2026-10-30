// pattern: Functional Core
//! Binds one parsed F13 container to its exact current F12 source identity.

use crate::{
    analysis::source_frame::SharedAnalysisReceipt,
    codec::frame_syntax::SelectedComplexityIndexTypeA,
    joc::{
        emdf::{
            metadata_owner::{
                Id14PopDisposition, MetadataQueuePopReceipt, MetadataQueueProducerProvenance,
                ParsedMetadataSourceContainer,
            },
            protected_container::{
                CurrentEmdfCarriageMode, CurrentPayloadId, ParsedCurrentEmdfContainer,
            },
        },
        model_complexity::derive_model_joc_complexity,
    },
    meter::prepared_job_identity::PreparedJobInputIdentity,
    timeline::{CodecSourceIndex, PublishedAuIndex},
    transaction::ProgrammeId,
};
use std::sync::Arc;

use super::selected_frame_trial::{EmdfFrameInput, PublishedEmdfInput};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublishedEmdfBindingError {
    UnexpectedSuppressedPayload,
    PublishedPayloadMissing,
    ProgrammeIdentityMismatch,
    PreparedInputIdentityMissing,
    PreparedInputIdentityMismatch,
    ConsumerIdentityMismatch,
    CarriageMismatch,
    QueueReceiptMismatch,
    JocValidationMissing,
    ComplexityOutOfRange,
}

/// Binds an F13 parsed container to a same-source SharedAnalysis receipt.
///
/// The caller supplies the typed programme ID and admission-created prepared-input
/// capability from the same `PreparedJobInput`; the capability must be pointer-identical
/// to the one retained by SharedAnalysis. No arbitrary source bytes or complexity value
/// can enter the returned frame input.
pub fn emdf_frame_input_for_source<'a>(
    parsed: Option<&'a ParsedMetadataSourceContainer>,
    shared_analysis: &SharedAnalysisReceipt,
    programme_id: &ProgrammeId,
    prepared_job_input_identity: &Arc<PreparedJobInputIdentity>,
) -> Result<EmdfFrameInput<'a>, PublishedEmdfBindingError> {
    let shared_identity = shared_analysis.identity();
    let source_key = shared_analysis.identity().source_stage_key();
    let admitted_identity = source_key
        .prepared_job_input_identity()
        .ok_or(PublishedEmdfBindingError::PreparedInputIdentityMissing)?;
    if !Arc::ptr_eq(admitted_identity, prepared_job_input_identity) {
        return Err(PublishedEmdfBindingError::PreparedInputIdentityMismatch);
    }
    let source_index = shared_identity.source_index();
    if source_key.source_index() != source_index {
        return Err(PublishedEmdfBindingError::ConsumerIdentityMismatch);
    }
    if source_index.value() < 2 {
        if parsed.is_some() {
            return Err(PublishedEmdfBindingError::UnexpectedSuppressedPayload);
        }
        return Ok(EmdfFrameInput::AbsentPublishedPayloadV1);
    }
    let parsed = parsed.ok_or(PublishedEmdfBindingError::PublishedPayloadMissing)?;
    let au_index = source_index
        .value()
        .checked_sub(2)
        .map(PublishedAuIndex::new)
        .ok_or(PublishedEmdfBindingError::ConsumerIdentityMismatch)?;
    if parsed.consumer_au_index() != au_index {
        return Err(PublishedEmdfBindingError::ConsumerIdentityMismatch);
    }
    if parsed.consumer_programme_id() != programme_id {
        return Err(PublishedEmdfBindingError::ProgrammeIdentityMismatch);
    }

    let expected_dispatch = source_key
        .source_index()
        .dispatch()
        .map_err(|_| PublishedEmdfBindingError::ConsumerIdentityMismatch)?;
    if source_key.attempt_id() != parsed.consumer_attempt()
        || source_key.transaction_id() != parsed.consumer_transaction_id()
        || source_key.source_index() != source_index
        || shared_analysis.identity().source_index() != source_index
        || parsed.consumer_dispatch() != expected_dispatch
    {
        return Err(PublishedEmdfBindingError::ConsumerIdentityMismatch);
    }

    let parsed_container = parsed.parsed_container();
    if parsed_container.mode() != CurrentEmdfCarriageMode::DeployedCompat
        || parsed_container.carrier_block().value() != 2
    {
        return Err(PublishedEmdfBindingError::CarriageMismatch);
    }
    if parsed_container.id14_provenance() != parsed.id14_queue_pop().producer() {
        return Err(PublishedEmdfBindingError::QueueReceiptMismatch);
    }
    validate_queue_receipts(
        parsed,
        parsed_container,
        programme_id,
        source_index,
        expected_dispatch,
        au_index,
    )?;

    let joc = parsed_container
        .payloads()
        .iter()
        .find(|payload| payload.id() == CurrentPayloadId::Joc)
        .and_then(|payload| payload.joc_validation())
        .ok_or(PublishedEmdfBindingError::JocValidationMissing)?;
    let id14_pop = parsed.id14_queue_pop();
    if joc.object_count() != id14_pop.producer().object_count().value()
        || joc.sequence_count() != id14_pop.producer().sequence_count()
        || joc.object_count() != 15
    {
        return Err(PublishedEmdfBindingError::JocValidationMissing);
    }
    let model_complexity = derive_model_joc_complexity(u32::from(joc.object_count()))
        .ok_or(PublishedEmdfBindingError::ComplexityOutOfRange)?;
    let complexity = SelectedComplexityIndexTypeA::from_model_value(model_complexity.value());
    let published =
        PublishedEmdfInput::from_parent_values(shared_analysis, parsed.bytes(), complexity);
    Ok(EmdfFrameInput::Published(published))
}

fn validate_queue_receipts(
    parsed: &ParsedMetadataSourceContainer,
    parsed_container: &ParsedCurrentEmdfContainer,
    programme_id: &ProgrammeId,
    source_index: CodecSourceIndex,
    dispatch: crate::timeline::JocDispatchIndex,
    au_index: PublishedAuIndex,
) -> Result<(), PublishedEmdfBindingError> {
    const PAYLOAD_ORDER: [u8; 4] = [11, 14, 2, 1];
    let expected_ordinary_writer = source_index
        .value()
        .checked_sub(2)
        .map(CodecSourceIndex::new)
        .ok_or(PublishedEmdfBindingError::ConsumerIdentityMismatch)?;
    let expected_joc_writer = source_index
        .value()
        .checked_sub(1)
        .map(CodecSourceIndex::new)
        .ok_or(PublishedEmdfBindingError::ConsumerIdentityMismatch)?;
    let queue_receipts = parsed.queue_pop_receipts();
    let payloads = parsed_container.payloads();
    if queue_receipts.len() != PAYLOAD_ORDER.len() || payloads.len() != PAYLOAD_ORDER.len() {
        return Err(PublishedEmdfBindingError::QueueReceiptMismatch);
    }

    for ((payload_id, queue_receipt), payload) in PAYLOAD_ORDER
        .iter()
        .copied()
        .zip(queue_receipts)
        .zip(payloads)
    {
        let expected_writer = if payload_id == 14 {
            expected_joc_writer
        } else {
            expected_ordinary_writer
        };
        if queue_receipt.queue_id() != payload_id
            || payload.id().wire_value() != payload_id
            || queue_receipt.writer_generation() != expected_writer
            || queue_receipt.payload() != payload.bytes()
            || queue_receipt.consumer_programme_id() != programme_id
            || queue_receipt.consumer_attempt() != parsed.consumer_attempt()
            || queue_receipt.consumer_transaction_id() != parsed.consumer_transaction_id()
            || queue_receipt.consumer_source_index() != source_index
            || queue_receipt.consumer_dispatch() != dispatch
            || queue_receipt.consumer_au_index() != Some(au_index)
            || queue_receipt.disposition() != Id14PopDisposition::Published
        {
            return Err(PublishedEmdfBindingError::QueueReceiptMismatch);
        }
        validate_producer_receipt(queue_receipt, programme_id, expected_writer, payload_id)?;
    }

    Ok(())
}

fn validate_producer_receipt(
    queue_receipt: &MetadataQueuePopReceipt,
    programme_id: &ProgrammeId,
    expected_writer: CodecSourceIndex,
    payload_id: u8,
) -> Result<(), PublishedEmdfBindingError> {
    let producer = queue_receipt.producer();
    if producer.producer_transaction_id().is_none() {
        return Err(PublishedEmdfBindingError::QueueReceiptMismatch);
    }
    let identity_matches = match producer {
        MetadataQueueProducerProvenance::MetadataBody(producer) => {
            payload_id != 14
                && producer.programme_id() == programme_id
                && producer.attempt() == queue_receipt.consumer_attempt()
                && producer.producer_generation() == expected_writer
                && producer.writer().value() == expected_writer.value()
                && expected_writer.value().checked_add(1) == Some(producer.dispatch().value())
                && expected_writer.value().checked_add(2)
                    == Some(u64::from(
                        producer.source_lineage().callback_index().value(),
                    ))
        }
        MetadataQueueProducerProvenance::Id14(producer) => {
            payload_id == 14
                && producer.lineage().programme_id() == programme_id
                && producer.lineage().attempt() == queue_receipt.consumer_attempt()
                && producer.producer_generation() == expected_writer
                && producer.writer().value() == expected_writer.value()
                && expected_writer.value().checked_add(1) == Some(producer.dispatch().value())
                && expected_writer.value().checked_add(2)
                    == Some(u64::from(
                        producer.lineage().transaction_id().callback_index().value(),
                    ))
        }
    };
    if identity_matches {
        Ok(())
    } else {
        Err(PublishedEmdfBindingError::QueueReceiptMismatch)
    }
}
