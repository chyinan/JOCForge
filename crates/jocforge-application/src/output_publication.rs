// pattern: Imperative Shell
//! Joins the current metadata and complete-frame proposals into one output-owner proposal.

use jocforge_domain::{
    codec::selected_main_q_search::{SelectedMainQOutcome, SelectedMainQSearchResult},
    joc::emdf::metadata_owner::{MetadataEofProposal, MetadataSourceProposal},
    publication::OutputSourceIdentity,
    timeline::{CodecSourceIndex, PublishedAuIndex},
};
use sha2::{Digest, Sha256};

pub use jocforge_domain::publication::{
    OutputFinalizeProposal, OutputFinalizedProposal, OutputOwner, OutputOwnerError,
    OutputSourceAfterProposal, OutputSourceProposal,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputPublicationPreparationError {
    NoBestFrame,
    FrameSourceMismatch,
    MetadataSourceMismatch,
    MetadataDispatchMismatch,
    MetadataAuMismatch,
    MetadataContainerDisposition,
    MetadataEofDisposition,
    OutputOwner(OutputOwnerError),
}

impl std::fmt::Display for OutputPublicationPreparationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoBestFrame => formatter.write_str("OUTPUT_NO_BEST_FRAME"),
            Self::FrameSourceMismatch => formatter.write_str("OUTPUT_FRAME_SOURCE_MISMATCH"),
            Self::MetadataSourceMismatch => formatter.write_str("OUTPUT_METADATA_SOURCE_MISMATCH"),
            Self::MetadataDispatchMismatch => {
                formatter.write_str("OUTPUT_METADATA_DISPATCH_MISMATCH")
            }
            Self::MetadataAuMismatch => formatter.write_str("OUTPUT_METADATA_AU_MISMATCH"),
            Self::MetadataContainerDisposition => {
                formatter.write_str("OUTPUT_METADATA_CONTAINER_DISPOSITION")
            }
            Self::MetadataEofDisposition => formatter.write_str("OUTPUT_METADATA_EOF_DISPOSITION"),
            Self::OutputOwner(error) => write!(formatter, "{}", error.code()),
        }
    }
}

impl std::error::Error for OutputPublicationPreparationError {}

/// Prepares the F15P source proposal from the already staged F13 owner result
/// and selected F12 q-search winner. It does not perform IO or install state.
pub fn prepare_output_source(
    output_owner: &OutputOwner,
    metadata: &MetadataSourceProposal,
    q_search: &SelectedMainQSearchResult,
) -> Result<OutputSourceProposal, OutputPublicationPreparationError> {
    let SelectedMainQOutcome::Best(candidate) = q_search.outcome() else {
        return Err(OutputPublicationPreparationError::NoBestFrame);
    };
    let lineage = metadata.bodies_lineage();
    let source_index = lineage.source_index();
    if candidate.source_index() != source_index {
        return Err(OutputPublicationPreparationError::FrameSourceMismatch);
    }
    let expected_dispatch = source_index
        .dispatch()
        .map_err(|_| OutputPublicationPreparationError::MetadataDispatchMismatch)?;
    if metadata.receipt().programme_id() != lineage.programme_id()
        || metadata.receipt().attempt() != lineage.attempt()
        || metadata.receipt().transaction_id() != lineage.transaction_id()
        || metadata.receipt().source_index() != source_index
    {
        return Err(OutputPublicationPreparationError::MetadataSourceMismatch);
    }
    if lineage.dispatch() != expected_dispatch || metadata.receipt().dispatch() != expected_dispatch
    {
        return Err(OutputPublicationPreparationError::MetadataDispatchMismatch);
    }

    let expected_au = source_index
        .value()
        .checked_sub(2)
        .map(PublishedAuIndex::new);
    if metadata.receipt().au_index() != expected_au {
        return Err(OutputPublicationPreparationError::MetadataAuMismatch);
    }
    if metadata.emdf().is_some() != expected_au.is_some() {
        return Err(OutputPublicationPreparationError::MetadataContainerDisposition);
    }

    let frame = candidate.frame_bytes();
    let declared_frame_sha256: [u8; 32] = Sha256::digest(frame).into();
    output_owner
        .prepare_source(
            OutputSourceIdentity::new(source_index, expected_dispatch, expected_au),
            frame,
            &declared_frame_sha256,
        )
        .map_err(OutputPublicationPreparationError::OutputOwner)
}

/// Binds metadata EOF's exact pending-writer disposition to the runtime output
/// plan before creating a terminal F15P proposal. No queue is popped or flushed.
pub fn prepare_output_finalize(
    output_owner: &OutputOwner,
    metadata_eof: &MetadataEofProposal,
) -> Result<OutputFinalizeProposal, OutputPublicationPreparationError> {
    let expected_sources = output_owner.configuration().expected_sources();
    let ordinary_tail = expected_sources
        .checked_sub(1)
        .ok_or(OutputPublicationPreparationError::MetadataEofDisposition)?;
    let au_count = expected_sources
        .checked_sub(2)
        .ok_or(OutputPublicationPreparationError::MetadataEofDisposition)?;
    let mut expected_pending = Vec::with_capacity(7);
    for queue_id in [1_u8, 2, 11] {
        expected_pending.push((queue_id, CodecSourceIndex::new(au_count), "DROP_AT_DONE"));
        expected_pending.push((
            queue_id,
            CodecSourceIndex::new(ordinary_tail),
            "DROP_AT_DONE",
        ));
    }
    expected_pending.push((14, CodecSourceIndex::new(ordinary_tail), "DROP_AT_DONE"));
    let mut actual_pending = metadata_eof.pending_writer_dispositions();
    expected_pending
        .sort_by_key(|(queue_id, writer, disposition)| (*queue_id, writer.value(), *disposition));
    actual_pending
        .sort_by_key(|(queue_id, writer, disposition)| (*queue_id, writer.value(), *disposition));
    if metadata_eof.expected_sources() != output_owner.configuration().expected_sources()
        || !metadata_eof.after_is_finalized()
        || !metadata_eof.queues_are_empty()
        || !metadata_eof.pending_writers_are_drop_at_done()
        || actual_pending != expected_pending
    {
        return Err(OutputPublicationPreparationError::MetadataEofDisposition);
    }
    output_owner
        .prepare_finalize()
        .map_err(OutputPublicationPreparationError::OutputOwner)
}
