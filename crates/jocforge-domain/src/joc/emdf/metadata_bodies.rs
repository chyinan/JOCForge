// pattern: Functional Core
//! Same-source metadata body barrier for the selected JOC path.

use super::{
    id2_wire::{Id2WireError, build_id2_body},
    metadata_owner::MetadataBodies,
};
use crate::{
    frontend::job_configuration::SelectedJobConfiguration,
    joc::id14_body::{Id14BodyProposal, Id14ProducerProvenance},
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
    Id14Provenance,
    Id14ObjectCount,
    TransactionIdentity,
    Id2(Id2WireError),
}

impl MetadataBodiesError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Lineage => "METADATA_BODIES_LINEAGE",
            Self::WriterGeneration => "METADATA_BODIES_WRITER_GENERATION",
            Self::Id11Body => "METADATA_BODIES_ID11",
            Self::Id14Provenance => "METADATA_BODIES_ID14_PROVENANCE",
            Self::Id14ObjectCount => "METADATA_BODIES_ID14_OBJECT_COUNT",
            Self::TransactionIdentity => "METADATA_BODIES_TRANSACTION_IDENTITY",
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

    pub const fn id14_provenance(&self) -> Option<&Id14ProducerProvenance> {
        self.bodies.id14_provenance.as_ref()
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
            bodies: MetadataBodies {
                id11: bodies.id11,
                id2: bodies.id2,
                id14: bodies.id14,
                id14_provenance: None,
            },
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test_with_id14_provenance(
        programme_id: SourceFlagsProgrammeId,
        attempt: SourceFlagsAttempt,
        transaction_id: CallbackLineageIdentity,
        source_index: CodecSourceIndex,
        bodies: MetadataBodies,
        provenance: Id14ProducerProvenance,
    ) -> Self {
        let mut proposal =
            Self::for_test(programme_id, attempt, transaction_id, source_index, bodies);
        proposal.bodies.id14_provenance = Some(provenance);
        proposal
    }
}

/// Binds the three currently available bodies to one admitted J05 projection.
/// The function does not invoke the metadata owner, advance a queue, or create
/// the still-unavailable ID1 measurement configuration.
pub fn assemble_metadata_bodies(
    configuration: &SelectedJobConfiguration,
    projection: &SelectedJocQmfProjection,
    reader_receipt: &ReaderTargetUpdateReceipt,
    id14: Id14BodyProposal,
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
    let (id14, id14_provenance) = id14.into_parts();
    if id14.writer_generation != source_index {
        return Err(MetadataBodiesError::WriterGeneration);
    }
    let producer_lineage = id14_provenance.lineage();
    if producer_lineage.programme_id() != projection.programme_id()
        || producer_lineage.attempt() != projection.attempt()
        || producer_lineage.transaction_id() != projection.transaction_id()
        || producer_lineage.source_index() != source_index
        || producer_lineage.dispatch() != expected_dispatch
        || id14_provenance.producer_generation() != source_index
        || id14_provenance.dispatch() != expected_dispatch
        || id14_provenance.writer().value() != source_index.value()
        || !id14_provenance.matches_body(source_index, &id14.bytes)
    {
        return Err(MetadataBodiesError::Id14Provenance);
    }
    if id14_provenance.producer_transaction_id().is_none() {
        return Err(MetadataBodiesError::TransactionIdentity);
    }
    if id14_provenance.object_count().value() != 15 {
        return Err(MetadataBodiesError::Id14ObjectCount);
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
            id14_provenance: Some(id14_provenance),
        },
    })
}
