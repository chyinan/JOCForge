// pattern: Functional Core
//! Same-source metadata body barrier for the selected JOC path.

use super::{
    id2_wire::{Id2WireError, build_id2_body},
    metadata_owner::{MetadataBodies, MetadataPayloadBody},
};
use crate::{
    frontend::job_configuration::SelectedJobConfiguration,
    joc::oamd::reader_target::ReaderTargetUpdateReceipt,
    spatial::{
        source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
        spatial_audio_state::SelectedJocQmfProjection,
    },
    timeline::{CodecSourceIndex, JocDispatchIndex},
    transaction::CallbackLineageIdentity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataBodiesError {
    Lineage,
    WriterGeneration,
    Id11Body,
    Id2(Id2WireError),
}

impl MetadataBodiesError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Lineage => "METADATA_BODIES_LINEAGE",
            Self::WriterGeneration => "METADATA_BODIES_WRITER_GENERATION",
            Self::Id11Body => "METADATA_BODIES_ID11",
            Self::Id2(_) => "METADATA_BODIES_ID2",
        }
    }
}

impl std::fmt::Display for MetadataBodiesError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for MetadataBodiesError {}

/// Immutable lineage retained alongside three same-source body values. ID1 is
/// intentionally absent until its measurement projection is available.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataBodiesLineage {
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: CallbackLineageIdentity,
    source_index: CodecSourceIndex,
    dispatch: JocDispatchIndex,
    reader_state_before_sha256: [u8; 32],
    reader_state_after_sha256: [u8; 32],
    reader_semantic_sha256: [u8; 32],
}

impl MetadataBodiesLineage {
    pub fn programme_id(&self) -> &SourceFlagsProgrammeId {
        &self.programme_id
    }

    pub const fn attempt(&self) -> SourceFlagsAttempt {
        self.attempt
    }

    pub const fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn dispatch(&self) -> JocDispatchIndex {
        self.dispatch
    }

    pub const fn reader_state_before_sha256(&self) -> &[u8; 32] {
        &self.reader_state_before_sha256
    }

    pub const fn reader_state_after_sha256(&self) -> &[u8; 32] {
        &self.reader_state_after_sha256
    }

    pub const fn reader_semantic_sha256(&self) -> &[u8; 32] {
        &self.reader_semantic_sha256
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataBodiesProposal {
    lineage: MetadataBodiesLineage,
    bodies: MetadataBodies,
}

impl MetadataBodiesProposal {
    pub const fn lineage(&self) -> &MetadataBodiesLineage {
        &self.lineage
    }

    pub const fn bodies(&self) -> &MetadataBodies {
        &self.bodies
    }

    pub fn into_parts(self) -> (MetadataBodiesLineage, MetadataBodies) {
        (self.lineage, self.bodies)
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        programme_id: SourceFlagsProgrammeId,
        attempt: SourceFlagsAttempt,
        transaction_id: CallbackLineageIdentity,
        source_index: CodecSourceIndex,
        bodies: MetadataBodies,
    ) -> Self {
        Self {
            lineage: MetadataBodiesLineage {
                programme_id,
                attempt,
                transaction_id,
                source_index,
                dispatch: source_index.dispatch().expect("bounded test source"),
                reader_state_before_sha256: [0; 32],
                reader_state_after_sha256: [0; 32],
                reader_semantic_sha256: [0; 32],
            },
            bodies,
        }
    }
}

/// Binds the three currently available bodies to one admitted J05 projection.
/// The function does not invoke the metadata owner, advance a queue, or create
/// the still-unavailable ID1 measurement configuration.
pub fn assemble_metadata_bodies(
    configuration: &SelectedJobConfiguration,
    projection: &SelectedJocQmfProjection,
    reader_receipt: &ReaderTargetUpdateReceipt,
    id14: MetadataPayloadBody,
) -> Result<MetadataBodiesProposal, MetadataBodiesError> {
    let source_index = projection.source_index();
    let expected_dispatch = source_index
        .dispatch()
        .map_err(|_| MetadataBodiesError::Lineage)?;
    if projection.dispatch() != expected_dispatch
        || u64::from(projection.callback_index().value()) != source_index.value() + 2
        || projection.component_origin_generation() != projection.callback_index()
        || u64::from(projection.lfe_origin_generation().value()) != source_index.value()
        || projection.target_reader_invocation().value() != source_index.value()
        || projection.programme_id() != reader_receipt.programme_id()
        || projection.transaction_id() != reader_receipt.transaction_id()
        || projection.callback_index() != reader_receipt.callback_generation()
        || projection.target_reader_invocation() != reader_receipt.reader_invocation()
        || projection.target_history_producer_generations()
            != reader_receipt.history_producer_generations()
        || projection.target_state_before_sha256() != reader_receipt.state_before_sha256()
        || projection.target_state_after_sha256() != reader_receipt.state_after_sha256()
        || projection.target_sha256() != reader_receipt.target_sha256()
        || projection.target_semantic_sha256() != reader_receipt.semantic_sha256()
        || projection.target_update_words() != &reader_receipt.target_words()
    {
        return Err(MetadataBodiesError::Lineage);
    }
    if id14.writer_generation != source_index {
        return Err(MetadataBodiesError::WriterGeneration);
    }
    let id11_bytes = reader_receipt.id11_body();
    if id11_bytes.is_empty() {
        return Err(MetadataBodiesError::Id11Body);
    }
    let id11 = reader_receipt.id11_metadata_body(source_index);
    let id2 = build_id2_body(
        configuration.line_drc_profile(),
        configuration.rf_drc_profile(),
        id11_bytes,
    )
    .map_err(MetadataBodiesError::Id2)?
    .into_metadata_body(source_index);
    Ok(MetadataBodiesProposal {
        lineage: MetadataBodiesLineage {
            programme_id: projection.programme_id().clone(),
            attempt: projection.attempt(),
            transaction_id: projection.transaction_id().clone(),
            source_index,
            dispatch: projection.dispatch(),
            reader_state_before_sha256: *reader_receipt.state_before_sha256(),
            reader_state_after_sha256: *reader_receipt.state_after_sha256(),
            reader_semantic_sha256: *reader_receipt.semantic_sha256(),
        },
        bodies: MetadataBodies {
            id11: Some(id11),
            id2: Some(id2),
            id14: Some(id14),
        },
    })
}
