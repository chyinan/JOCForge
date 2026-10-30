//! Functional Core owner for selected-profile metadata queues, timer and EMDF proposals.
//! It performs no file, process, environment or durable-output I/O.

mod emdf_serializer;
mod payload_queue;
mod timer;

#[cfg(test)]
mod tests;

use emdf_serializer::{EmdfOutput, serialize as serialize_emdf};
use payload_queue::{PayloadQueue, QueueOperation, QueuedBody};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use timer::{TimerState, encode_id1_payload};

use super::{
    id1_configuration::Id1MeasurementConfiguration,
    metadata_bodies::{MetadataBodiesLineage, MetadataBodiesProposal},
};
use crate::joc::id14_body::Id14ProducerProvenance;
use crate::{
    spatial::source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
    timeline::{
        CodecSourceIndex, FrontendCallbackIndex, JocDispatchIndex, OrdinaryPayloadWriterIndex,
        PublicationDisposition, PublishedAuIndex, publication_for_source, source_for_callback,
    },
    transaction::{
        AttemptId, CallbackTransactionId, GtcOwnerId, GtcOwnerReceipt, GtcOwnerReceiptError,
        GtcOwnerStateSnapshot, ProgrammeId,
    },
};

const PROFILE_ID: &str = "selected-48k-1536-four-queue.v1";
const CARRIAGE_MODE: &str = "deployed_compat";
const PROVIDER_FILE_SHA256: &str =
    "6d94c55c8c011fd4762e60ec92ed491f19b17540c72ba82094a36f6e666f2ef7";
const PROVIDER_CANONICAL_SHA256: &str =
    "0132eef416f50c029fc248b0580b2b20fef27688c9dee75210ee4ea1547aed3b";
const WRAPPER_FILE_SHA256: &str =
    "722e93ad671e4faab6ae9f33125bd49f8f369f717db0c9a3600cd861cdbd5a56";
const WRAPPER_CANONICAL_SHA256: &str =
    "e5d4a8f50324b4d1a7af7631b76244ea91da12cf9fa1f1b668c045c2ac6ad352";
const QUEUE_MUTATION_ORDER: [u8; 4] = [11, 1, 2, 14];
const PUBLICATION_ORDER: [u8; 4] = [11, 14, 2, 1];
const CANONICAL_QUEUE_ORDER: [u8; 4] = [1, 2, 11, 14];
const METADATA_OWNER_ID: &str = "metadata";
const METADATA_STATE_REF: &str = "owners.metadata";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataOwnerError {
    ProviderIdentity,
    WrapperIdentity,
    WrapperConfiguration,
    ProgrammeIdentity,
    SourceOrder,
    DispatchOrder,
    AuOrder,
    BodyReadiness,
    WriterGeneration,
    Id14Provenance,
    Timer,
    WireField,
    Capacity,
    QueueOccupied,
    EmdfSerialization,
    CurrentEmdfValidation,
    EofSourceCount,
    Finalized,
    Retired,
    AttemptSequence,
}

impl MetadataOwnerError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ProviderIdentity => "METADATA_PROVIDER_IDENTITY",
            Self::WrapperIdentity => "METADATA_WRAPPER_IDENTITY",
            Self::WrapperConfiguration => "METADATA_WRAPPER_CONFIGURATION",
            Self::ProgrammeIdentity => "METADATA_PROGRAMME_IDENTITY",
            Self::SourceOrder => "METADATA_SOURCE_ORDER",
            Self::DispatchOrder => "METADATA_DISPATCH_ORDER",
            Self::AuOrder => "METADATA_AU_ORDER",
            Self::BodyReadiness => "METADATA_BODY_READINESS",
            Self::WriterGeneration => "METADATA_WRITER_GENERATION",
            Self::Id14Provenance => "METADATA_ID14_PROVENANCE",
            Self::Timer => "METADATA_TIMER",
            Self::WireField => "METADATA_WIRE_FIELD",
            Self::Capacity => "METADATA_CAPACITY",
            Self::QueueOccupied => "METADATA_QUEUE_OCCUPIED",
            Self::EmdfSerialization => "METADATA_EMDF_SERIALIZATION",
            Self::CurrentEmdfValidation => "METADATA_CURRENT_EMDF_VALIDATION",
            Self::EofSourceCount => "METADATA_EOF_SOURCE_COUNT",
            Self::Finalized => "METADATA_FINALIZED",
            Self::Retired => "METADATA_RETIRED",
            Self::AttemptSequence => "METADATA_ATTEMPT_SEQUENCE",
        }
    }
}

impl std::fmt::Display for MetadataOwnerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for MetadataOwnerError {}

fn metadata_body_provenance_for_source(
    identity: &MetadataSourceIdentity,
    writer_generation: CodecSourceIndex,
    payload: &[u8],
) -> Result<Option<MetadataQueueProducerProvenance>, MetadataOwnerError> {
    if !identity.id14_provenance_present {
        return Ok(None);
    }
    let provenance = MetadataBodyProducerProvenance::new(identity, writer_generation, payload)
        .ok_or(MetadataOwnerError::Id14Provenance)?;
    Ok(Some(MetadataQueueProducerProvenance::MetadataBody(
        provenance,
    )))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataPayloadBody {
    pub writer_generation: CodecSourceIndex,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataBodyProducerProvenance {
    producer_transaction_id: CallbackTransactionId,
    source_lineage: crate::transaction::CallbackLineageIdentity,
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    producer_generation: CodecSourceIndex,
    dispatch: JocDispatchIndex,
    writer: OrdinaryPayloadWriterIndex,
    byte_length: u16,
    payload_sha256: [u8; 32],
}

impl MetadataBodyProducerProvenance {
    fn new(
        identity: &MetadataSourceIdentity,
        writer_generation: CodecSourceIndex,
        payload: &[u8],
    ) -> Option<Self> {
        let byte_length = u16::try_from(payload.len()).ok()?;
        let producer_transaction_id = identity.transaction_id.clone()?;
        if identity.source_lineage.programme_id() != &identity.programme_id
            || identity.source_lineage.attempt_id() != identity.attempt
            || u64::from(identity.source_lineage.callback_index().value())
                != identity.source_index.value().checked_add(2)?
            || identity.dispatch.value() != identity.source_index.value().checked_add(1)?
            || writer_generation != identity.source_index
        {
            return None;
        }
        Some(Self {
            producer_transaction_id,
            source_lineage: identity.source_lineage.clone(),
            programme_id: identity.programme_id.clone(),
            attempt: identity.attempt,
            producer_generation: identity.source_index,
            dispatch: identity.dispatch,
            writer: OrdinaryPayloadWriterIndex::new(writer_generation.value()),
            byte_length,
            payload_sha256: Sha256::digest(payload).into(),
        })
    }

    pub fn producer_transaction_id(&self) -> &CallbackTransactionId {
        &self.producer_transaction_id
    }

    pub const fn source_lineage(&self) -> &crate::transaction::CallbackLineageIdentity {
        &self.source_lineage
    }

    pub fn programme_id(&self) -> &SourceFlagsProgrammeId {
        &self.programme_id
    }

    pub const fn attempt(&self) -> SourceFlagsAttempt {
        self.attempt
    }

    pub const fn producer_generation(&self) -> CodecSourceIndex {
        self.producer_generation
    }

    pub const fn dispatch(&self) -> JocDispatchIndex {
        self.dispatch
    }

    pub const fn writer(&self) -> OrdinaryPayloadWriterIndex {
        self.writer
    }

    pub const fn byte_length(&self) -> u16 {
        self.byte_length
    }

    pub const fn payload_sha256(&self) -> &[u8; 32] {
        &self.payload_sha256
    }

    fn matches_body(&self, writer_generation: CodecSourceIndex, payload: &[u8]) -> bool {
        self.producer_generation == writer_generation
            && self.writer.value() == writer_generation.value()
            && u16::try_from(payload.len()).ok() == Some(self.byte_length)
            && self.payload_sha256 == <[u8; 32]>::from(Sha256::digest(payload))
    }

    fn as_json(&self) -> Value {
        json!({
            "kind": "metadata_body",
            "producer_transaction_id": self.producer_transaction_id.as_str(),
            "programme_id": self.programme_id.as_str(),
            "attempt": self.attempt.value(),
            "source_lineage_callback": self.source_lineage.callback_index().value(),
            "producer_generation": self.producer_generation.value(),
            "dispatch": self.dispatch.value(),
            "writer": self.writer.value(),
            "byte_length": self.byte_length,
            "payload_sha256": hex_encode(&self.payload_sha256),
        })
    }

    fn append_canonical(&self, bytes: &mut Vec<u8>) {
        let transaction_id = self.producer_transaction_id.as_str().as_bytes();
        bytes.extend_from_slice(&(transaction_id.len() as u32).to_le_bytes());
        bytes.extend_from_slice(transaction_id);
        let programme_id = self.programme_id.as_str().as_bytes();
        bytes.extend_from_slice(&(programme_id.len() as u32).to_le_bytes());
        bytes.extend_from_slice(programme_id);
        bytes.extend_from_slice(&self.attempt.value().to_le_bytes());
        bytes.extend_from_slice(&self.source_lineage.callback_index().value().to_le_bytes());
        bytes.extend_from_slice(&self.producer_generation.value().to_le_bytes());
        bytes.extend_from_slice(&self.dispatch.value().to_le_bytes());
        bytes.extend_from_slice(&self.writer.value().to_le_bytes());
        bytes.extend_from_slice(&self.byte_length.to_le_bytes());
        bytes.extend_from_slice(&self.payload_sha256);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MetadataQueueProducerProvenance {
    MetadataBody(MetadataBodyProducerProvenance),
    Id14(Id14ProducerProvenance),
}

impl MetadataQueueProducerProvenance {
    pub fn producer_transaction_id(&self) -> Option<&CallbackTransactionId> {
        match self {
            Self::MetadataBody(provenance) => Some(provenance.producer_transaction_id()),
            Self::Id14(provenance) => provenance.producer_transaction_id(),
        }
    }

    fn matches_body(&self, writer_generation: CodecSourceIndex, payload: &[u8]) -> bool {
        match self {
            Self::MetadataBody(provenance) => provenance.matches_body(writer_generation, payload),
            Self::Id14(provenance) => provenance.matches_body(writer_generation, payload),
        }
    }

    fn as_json(&self) -> Value {
        match self {
            Self::MetadataBody(provenance) => provenance.as_json(),
            Self::Id14(provenance) => {
                let mut value = provenance.as_json();
                value["kind"] = Value::String("id14".to_owned());
                value
            }
        }
    }

    fn append_canonical(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::MetadataBody(provenance) => {
                bytes.push(1);
                provenance.append_canonical(bytes);
            }
            Self::Id14(provenance) => {
                bytes.push(14);
                provenance.append_canonical(bytes);
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataBodies {
    pub id11: Option<MetadataPayloadBody>,
    pub id2: Option<MetadataPayloadBody>,
    pub id14: Option<MetadataPayloadBody>,
    pub(crate) id14_provenance: Option<Id14ProducerProvenance>,
}

#[derive(Clone, Debug, PartialEq)]
struct MetadataSourceCall {
    pub programme_id: SourceFlagsProgrammeId,
    pub attempt: SourceFlagsAttempt,
    pub source_lineage: crate::transaction::CallbackLineageIdentity,
    pub consumer_transaction_id: Option<CallbackTransactionId>,
    pub source_index: CodecSourceIndex,
    pub dispatch: JocDispatchIndex,
    pub au_index: Option<PublishedAuIndex>,
    config: Vec<Value>,
    pub bodies: MetadataBodies,
    pub wrapper_configuration: Value,
    validate_current_container: bool,
}

#[derive(Clone, Debug)]
struct MetadataSourceIdentity {
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    source_lineage: crate::transaction::CallbackLineageIdentity,
    transaction_id: Option<CallbackTransactionId>,
    source_index: CodecSourceIndex,
    dispatch: JocDispatchIndex,
    au_index: Option<PublishedAuIndex>,
    id14_provenance_present: bool,
}

impl MetadataSourceIdentity {
    fn from_call(call: &MetadataSourceCall) -> Self {
        Self {
            programme_id: call.programme_id.clone(),
            attempt: call.attempt,
            source_lineage: call.source_lineage.clone(),
            transaction_id: call.consumer_transaction_id.clone(),
            source_index: call.source_index,
            dispatch: call.dispatch,
            au_index: call.au_index,
            id14_provenance_present: call.bodies.id14_provenance.is_some(),
        }
    }
}

/// Typed per-source input. The ID1 programme configuration and wrapper are
/// constructor-pinned and are not retransmitted or recomputed for each source.
pub struct MetadataStageInput {
    pub au_index: Option<PublishedAuIndex>,
    pub transaction_id: CallbackTransactionId,
    pub bodies: MetadataBodiesProposal,
}

#[must_use]
pub struct MetadataSourceProposal {
    after_owner: MetadataOwner,
    receipt: MetadataSourceReceipt,
    emdf: Option<Vec<u8>>,
    bodies_lineage: MetadataBodiesLineage,
    consumer_transaction_id: Option<CallbackTransactionId>,
    parsed_container: Option<ParsedMetadataSourceContainer>,
    before_snapshot: MetadataOwnerSnapshot,
    after_snapshot: MetadataOwnerSnapshot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)] // Read by the future twelve-root source preflight.
pub(crate) struct MetadataOwnerSnapshot {
    generation: u64,
    state_sha256: [u8; 32],
    next_source_index: CodecSourceIndex,
}

#[allow(dead_code)] // Read by the future twelve-root source preflight.
impl MetadataOwnerSnapshot {
    pub(crate) const fn generation(self) -> u64 {
        self.generation
    }

    pub(crate) const fn state_sha256(self) -> [u8; 32] {
        self.state_sha256
    }

    pub(crate) const fn next_source_index(self) -> CodecSourceIndex {
        self.next_source_index
    }
}

/// Complete immutable canonical state material for one GTC Metadata owner snapshot.
///
/// The snapshot carries the full `MetadataStateV1` bytes. It is not a source
/// receipt, a GTC `OwnerReceipt`, or a parent commit capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataOwnerStateReference {
    generation: u64,
    state_sha256: [u8; 32],
    cold_sha256: [u8; 32],
    canonical_bytes: Vec<u8>,
}

impl MetadataOwnerStateReference {
    pub const fn owner_id(&self) -> &'static str {
        METADATA_OWNER_ID
    }

    pub const fn state_ref(&self) -> &'static str {
        METADATA_STATE_REF
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn state_sha256(&self) -> &[u8; 32] {
        &self.state_sha256
    }

    pub const fn cold_sha256(&self) -> &[u8; 32] {
        &self.cold_sha256
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Adapts this complete canonical value to the registered GTC owner slot.
    ///
    /// The returned value is a snapshot only; it grants no metadata or programme commit
    /// authority.
    pub fn gtc_snapshot(&self) -> Result<GtcOwnerStateSnapshot<'_>, GtcOwnerReceiptError> {
        GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::Metadata,
            self.generation,
            self.cold_sha256,
            &self.canonical_bytes,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataGtcReceiptError {
    Clock,
    OwnerRetired,
    CallbackHasSourceStage,
    CallbackHasUnexpectedSource,
    ProgrammeMismatch,
    AttemptMismatch,
    ParentTransactionMismatch,
    SourceLineageMismatch,
    DispatchMismatch,
    AuMismatch,
    BeforeStateMismatch,
    AfterStateMismatch,
    GtcReceipt(GtcOwnerReceiptError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)] // Immutable bridge for the selected JOC root view adapter.
pub(crate) struct MetadataQueueBodyView {
    writer_generation: CodecSourceIndex,
    bytes: Box<[u8]>,
    producer_provenance: Option<MetadataQueueProducerProvenance>,
}

#[allow(dead_code)] // Immutable bridge for the selected JOC root view adapter.
impl MetadataQueueBodyView {
    pub(crate) const fn writer_generation(&self) -> CodecSourceIndex {
        self.writer_generation
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) const fn producer_provenance(&self) -> Option<&MetadataQueueProducerProvenance> {
        self.producer_provenance.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)] // Immutable bridge for the selected JOC root view adapter.
pub(crate) struct MetadataId14QueueView {
    slots: [Option<MetadataQueueBodyView>; 2],
    buffer_ids: [u16; 2],
    next_writer: CodecSourceIndex,
}

#[allow(dead_code)] // Immutable bridge for the selected JOC root view adapter.
impl MetadataId14QueueView {
    pub(crate) const fn slots(&self) -> &[Option<MetadataQueueBodyView>; 2] {
        &self.slots
    }

    pub(crate) const fn buffer_ids(&self) -> &[u16; 2] {
        &self.buffer_ids
    }

    pub(crate) const fn next_writer(&self) -> CodecSourceIndex {
        self.next_writer
    }
}

impl MetadataSourceProposal {
    pub const fn receipt(&self) -> &MetadataSourceReceipt {
        &self.receipt
    }

    /// Returns the measurement-bound ID1 configuration retained by this source owner proposal.
    pub const fn id1_configuration(&self) -> &Id1MeasurementConfiguration {
        &self.after_owner.id1_configuration
    }

    pub fn emdf(&self) -> Option<&[u8]> {
        self.emdf.as_deref()
    }

    pub const fn bodies_lineage(&self) -> &MetadataBodiesLineage {
        &self.bodies_lineage
    }

    pub fn consumer_transaction_id(&self) -> Option<&CallbackTransactionId> {
        self.consumer_transaction_id.as_ref()
    }

    pub const fn parsed_container(&self) -> Option<&ParsedMetadataSourceContainer> {
        self.parsed_container.as_ref()
    }

    /// Projects this noncommitting source proposal into its owner-level GTC receipt.
    ///
    /// The caller supplies the exact committed before-root and parent transaction token. This
    /// validates the source clock, F13 consumer token, actual before/after owner snapshots, and
    /// canonical state digests. It does not install metadata state or admit a 12-owner callback.
    pub fn gtc_owner_receipt(
        &self,
        before: &MetadataOwner,
        transaction_id: CallbackTransactionId,
    ) -> Result<GtcOwnerReceipt, MetadataGtcReceiptError> {
        let receipt = &self.receipt;
        let source_index = receipt.source_index();
        let source_key_callback = receipt.transaction_id().callback_index();
        let expected_dispatch = source_index
            .dispatch()
            .map_err(|_| MetadataGtcReceiptError::Clock)?;
        let expected_au = match publication_for_source(source_index)
            .map_err(|_| MetadataGtcReceiptError::Clock)?
        {
            PublicationDisposition::Suppressed { .. } => None,
            PublicationDisposition::Published(consumption) => Some(consumption.au),
        };
        let next_source_index = source_index
            .value()
            .checked_add(1)
            .map(CodecSourceIndex::new)
            .ok_or(MetadataGtcReceiptError::Clock)?;

        if before.retired || before.state.finalized || self.after_owner.retired {
            return Err(MetadataGtcReceiptError::OwnerRetired);
        }
        if &before.programme_id != receipt.programme_id()
            || self.after_owner.programme_id != before.programme_id
            || receipt.transaction_id().programme_id() != &before.programme_id
            || self.bodies_lineage.programme_id() != &before.programme_id
        {
            return Err(MetadataGtcReceiptError::ProgrammeMismatch);
        }
        if before.attempt != receipt.attempt()
            || self.after_owner.attempt != before.attempt
            || receipt.transaction_id().attempt_id() != before.attempt
            || self.bodies_lineage.attempt() != before.attempt
        {
            return Err(MetadataGtcReceiptError::AttemptMismatch);
        }
        if self.consumer_transaction_id.as_ref() != Some(&transaction_id)
            || receipt.consumer_transaction_id() != Some(&transaction_id)
        {
            return Err(MetadataGtcReceiptError::ParentTransactionMismatch);
        }
        if receipt.transaction_id() != self.bodies_lineage.transaction_id()
            || source_for_callback(source_key_callback) != Some(source_index)
            || before.state.next_source_index != source_index.value()
            || self.before_snapshot.next_source_index() != source_index
            || self.after_owner.state.next_source_index != next_source_index.value()
            || self.after_snapshot.next_source_index() != next_source_index
            || self.bodies_lineage.source_index() != source_index
        {
            return Err(MetadataGtcReceiptError::SourceLineageMismatch);
        }
        if receipt.dispatch() != expected_dispatch
            || self.bodies_lineage.dispatch() != expected_dispatch
        {
            return Err(MetadataGtcReceiptError::DispatchMismatch);
        }
        if receipt.au_index() != expected_au
            || self.parsed_container.is_some() != expected_au.is_some()
            || self.emdf.is_some() != expected_au.is_some()
        {
            return Err(MetadataGtcReceiptError::AuMismatch);
        }

        let before_reference = before.state_reference();
        let after_reference = self.after_owner.state_reference();
        if before_reference.generation() != self.before_snapshot.generation()
            || before_reference.state_sha256() != &self.before_snapshot.state_sha256()
            || before_reference.generation() != receipt.before_generation
            || hex_encode(before_reference.state_sha256()) != receipt.before_sha256
        {
            return Err(MetadataGtcReceiptError::BeforeStateMismatch);
        }
        if after_reference.generation() != self.after_snapshot.generation()
            || after_reference.state_sha256() != &self.after_snapshot.state_sha256()
            || after_reference.generation() != receipt.proposed_after_generation
            || hex_encode(after_reference.state_sha256()) != receipt.proposed_after_sha256
            || before_reference.cold_sha256() != after_reference.cold_sha256()
        {
            return Err(MetadataGtcReceiptError::AfterStateMismatch);
        }

        let before_snapshot = before_reference
            .gtc_snapshot()
            .map_err(MetadataGtcReceiptError::GtcReceipt)?;
        let after_snapshot = after_reference
            .gtc_snapshot()
            .map_err(MetadataGtcReceiptError::GtcReceipt)?;
        GtcOwnerReceipt::from_transition(before_snapshot, after_snapshot, transaction_id)
            .map_err(MetadataGtcReceiptError::GtcReceipt)
    }

    pub const fn id14_queue_pop(&self) -> Option<&Id14QueuePopReceipt> {
        self.receipt.id14_queue_pop()
    }

    #[allow(dead_code)] // Read by the future twelve-root source preflight.
    pub(crate) const fn before_snapshot(&self) -> MetadataOwnerSnapshot {
        self.before_snapshot
    }

    #[allow(dead_code)] // Read by the future twelve-root source preflight.
    pub(crate) const fn after_snapshot(&self) -> MetadataOwnerSnapshot {
        self.after_snapshot
    }

    #[allow(dead_code)] // Read by the selected JOC root view adapter.
    pub(crate) fn id14_queue_view(&self) -> Result<MetadataId14QueueView, MetadataOwnerError> {
        self.after_owner.id14_queue_view()
    }

    pub fn proposed_after_sha256(&self) -> String {
        self.after_owner.state.sha256_hex()
    }

    #[allow(dead_code)] // Consumed only by the future programme source transaction installer.
    pub(crate) fn into_after_owner(self) -> MetadataOwner {
        self.after_owner
    }

    #[allow(dead_code)] // Consumed by the future programme source transaction preflight.
    pub(crate) fn into_parts(
        self,
    ) -> (
        MetadataOwner,
        MetadataSourceReceipt,
        Option<Vec<u8>>,
        MetadataBodiesLineage,
        MetadataOwnerSnapshot,
        MetadataOwnerSnapshot,
        Option<ParsedMetadataSourceContainer>,
    ) {
        (
            self.after_owner,
            self.receipt,
            self.emdf,
            self.bodies_lineage,
            self.before_snapshot,
            self.after_snapshot,
            self.parsed_container,
        )
    }
}

#[derive(Debug)]
pub struct MetadataSourceResult {
    pub receipt: MetadataSourceReceipt,
    pub emdf: Option<Vec<u8>>,
}

struct MetadataSourceProcessed {
    after_state: MetadataState,
    receipt: MetadataSourceReceipt,
    emdf: Option<Vec<u8>>,
    parsed_container: Option<ParsedMetadataSourceContainer>,
}

/// Current-container parser output joined to the separate consumer source transaction.
///
/// This is not F12-ready. It binds parsed ID1/ID2 config and bytes to their queue
/// producers, but does not claim full ID1/ID2 body-semantic decoding. It is not a
/// frame or publication capability. The parsed container remains bound to the
/// popped ID14 producer and current consumer lineage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedMetadataSourceContainer {
    parsed_container: crate::joc::emdf::protected_container::ParsedCurrentEmdfContainer,
    consumer_programme_id: SourceFlagsProgrammeId,
    consumer_attempt: SourceFlagsAttempt,
    consumer_transaction_id: CallbackTransactionId,
    consumer_source_index: CodecSourceIndex,
    consumer_dispatch: JocDispatchIndex,
    consumer_au_index: PublishedAuIndex,
    id14_queue_pop: Id14QueuePopReceipt,
    queue_pop_receipts: Vec<MetadataQueuePopReceipt>,
}

impl ParsedMetadataSourceContainer {
    pub const fn parsed_container(
        &self,
    ) -> &crate::joc::emdf::protected_container::ParsedCurrentEmdfContainer {
        &self.parsed_container
    }

    pub fn bytes(&self) -> &[u8] {
        self.parsed_container.as_bytes()
    }

    pub const fn consumer_programme_id(&self) -> &SourceFlagsProgrammeId {
        &self.consumer_programme_id
    }

    pub const fn consumer_attempt(&self) -> SourceFlagsAttempt {
        self.consumer_attempt
    }

    pub fn consumer_transaction_id(&self) -> &CallbackTransactionId {
        &self.consumer_transaction_id
    }

    pub const fn consumer_source_index(&self) -> CodecSourceIndex {
        self.consumer_source_index
    }

    pub const fn consumer_dispatch(&self) -> JocDispatchIndex {
        self.consumer_dispatch
    }

    pub const fn consumer_au_index(&self) -> PublishedAuIndex {
        self.consumer_au_index
    }

    pub const fn id14_queue_pop(&self) -> &Id14QueuePopReceipt {
        &self.id14_queue_pop
    }

    pub fn queue_pop_receipts(&self) -> &[MetadataQueuePopReceipt] {
        &self.queue_pop_receipts
    }
}

#[derive(Debug)]
pub struct MetadataSourceReceipt {
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: crate::transaction::CallbackLineageIdentity,
    consumer_transaction_id: Option<CallbackTransactionId>,
    source_index: CodecSourceIndex,
    dispatch: JocDispatchIndex,
    au_index: Option<PublishedAuIndex>,
    before_generation: u64,
    proposed_after_generation: u64,
    before_sha256: String,
    proposed_after_sha256: String,
    provider_sha256: String,
    wrapper_config_sha256: String,
    timer_before_sha256: String,
    timer_after_sha256: String,
    timer_refreshed_field: Option<u8>,
    queue_operations: Vec<QueueOperation>,
    published_payloads: Vec<PublishedPayload>,
    queue_pop_receipts: Vec<MetadataQueuePopReceipt>,
    id14_queue_pop: Option<Id14QueuePopReceipt>,
    emdf: Option<Vec<u8>>,
    terminal_layout: Option<Value>,
}

impl MetadataSourceReceipt {
    pub fn programme_id(&self) -> &SourceFlagsProgrammeId {
        &self.programme_id
    }

    pub const fn attempt(&self) -> SourceFlagsAttempt {
        self.attempt
    }

    /// Returns the F04/F05 source callback lineage, not the parent GTC transaction token.
    pub const fn transaction_id(&self) -> &crate::transaction::CallbackLineageIdentity {
        &self.transaction_id
    }

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn dispatch(&self) -> JocDispatchIndex {
        self.dispatch
    }

    pub const fn au_index(&self) -> Option<PublishedAuIndex> {
        self.au_index
    }

    pub const fn id14_queue_pop(&self) -> Option<&Id14QueuePopReceipt> {
        self.id14_queue_pop.as_ref()
    }

    pub fn queue_pop_provenance(&self, queue_id: u8) -> Option<&MetadataQueueProducerProvenance> {
        self.queue_operations
            .iter()
            .find(|operation| operation.queue_id == queue_id)
            .and_then(|operation| operation.popped.as_ref())
            .and_then(|body| body.producer_provenance.as_ref())
    }

    pub fn queue_pop_receipts(&self) -> &[MetadataQueuePopReceipt] {
        &self.queue_pop_receipts
    }

    pub fn consumer_transaction_id(&self) -> Option<&CallbackTransactionId> {
        self.consumer_transaction_id.as_ref()
    }

    pub fn as_json(&self) -> Value {
        let mut value = json!({
            "schema": "metadata-source-receipt.v1",
            "owner_id": "metadata",
            "source_index": self.source_index.value(),
            "dispatch": self.dispatch.value(),
            "au_index": self.au_index.map(PublishedAuIndex::value),
            "before_generation": self.before_generation,
            "before_sha256": self.before_sha256,
            "proposed_after_generation": self.proposed_after_generation,
            "proposed_after_sha256": self.proposed_after_sha256,
            "provider_sha256": self.provider_sha256,
            "wrapper_config_sha256": self.wrapper_config_sha256,
            "timer_before_sha256": self.timer_before_sha256,
            "timer_after_sha256": self.timer_after_sha256,
            "timer_refreshed_field": self.timer_refreshed_field,
            "queue_operations": self.queue_operations.iter().map(QueueOperation::as_json).collect::<Vec<_>>(),
            "publication_order": PUBLICATION_ORDER,
            "published": self.au_index.is_some(),
            "published_payloads": self.published_payloads.iter().map(PublishedPayload::as_json).collect::<Vec<_>>(),
            "emdf_hex": self.emdf.as_ref().map(|bytes| hex_encode(bytes)),
            "emdf_sha256": self.emdf.as_ref().map(|bytes| sha256_hex(bytes)),
            "terminal_layout": self.terminal_layout,
            "commit_count": 1,
        });
        if let Some(id14_queue_pop) = &self.id14_queue_pop {
            value["id14_queue_pop"] = id14_queue_pop.as_json();
            if let Some(transaction_id) = &self.consumer_transaction_id {
                value["consumer_transaction_id"] =
                    Value::String(transaction_id.as_str().to_owned());
            }
        }
        if !self.queue_pop_receipts.is_empty() {
            value["queue_pop_receipts"] = Value::Array(
                self.queue_pop_receipts
                    .iter()
                    .map(MetadataQueuePopReceipt::as_json)
                    .collect(),
            );
        }
        value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Id14PopDisposition {
    Published,
    Suppressed,
}

impl Id14PopDisposition {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Published => "PUBLISHED",
            Self::Suppressed => "SUPPRESSED",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Id14QueuePopReceipt {
    producer: Id14ProducerProvenance,
    payload: Box<[u8]>,
    consumer_programme_id: SourceFlagsProgrammeId,
    consumer_attempt: SourceFlagsAttempt,
    consumer_transaction_id: CallbackTransactionId,
    consumer_source_index: CodecSourceIndex,
    consumer_dispatch: JocDispatchIndex,
    consumer_au_index: Option<PublishedAuIndex>,
    disposition: Id14PopDisposition,
}

impl Id14QueuePopReceipt {
    pub const fn producer(&self) -> &Id14ProducerProvenance {
        &self.producer
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn consumer_programme_id(&self) -> &SourceFlagsProgrammeId {
        &self.consumer_programme_id
    }

    pub const fn consumer_attempt(&self) -> SourceFlagsAttempt {
        self.consumer_attempt
    }

    pub fn consumer_transaction_id(&self) -> &CallbackTransactionId {
        &self.consumer_transaction_id
    }

    pub const fn consumer_source_index(&self) -> CodecSourceIndex {
        self.consumer_source_index
    }

    pub const fn consumer_dispatch(&self) -> JocDispatchIndex {
        self.consumer_dispatch
    }

    pub const fn consumer_au_index(&self) -> Option<PublishedAuIndex> {
        self.consumer_au_index
    }

    pub const fn disposition(&self) -> Id14PopDisposition {
        self.disposition
    }

    fn as_json(&self) -> Value {
        json!({
            "producer": self.producer.as_json(),
            "producer_callback_lineage_index": self.producer.lineage().transaction_id().callback_index().value(),
            "payload_hex": hex_encode(&self.payload),
            "consumer_programme_id": self.consumer_programme_id.as_str(),
            "consumer_attempt": self.consumer_attempt.value(),
            "consumer_transaction_id": self.consumer_transaction_id.as_str(),
            "consumer_source_index": self.consumer_source_index.value(),
            "consumer_dispatch": self.consumer_dispatch.value(),
            "consumer_au_index": self.consumer_au_index.map(PublishedAuIndex::value),
            "disposition": self.disposition.as_str(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataQueuePopReceipt {
    queue_id: u8,
    writer_generation: CodecSourceIndex,
    payload: Box<[u8]>,
    producer: MetadataQueueProducerProvenance,
    consumer_programme_id: SourceFlagsProgrammeId,
    consumer_attempt: SourceFlagsAttempt,
    consumer_transaction_id: CallbackTransactionId,
    consumer_source_index: CodecSourceIndex,
    consumer_dispatch: JocDispatchIndex,
    consumer_au_index: Option<PublishedAuIndex>,
    disposition: Id14PopDisposition,
}

impl MetadataQueuePopReceipt {
    pub const fn queue_id(&self) -> u8 {
        self.queue_id
    }

    pub const fn writer_generation(&self) -> CodecSourceIndex {
        self.writer_generation
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub const fn producer(&self) -> &MetadataQueueProducerProvenance {
        &self.producer
    }

    pub const fn consumer_programme_id(&self) -> &SourceFlagsProgrammeId {
        &self.consumer_programme_id
    }

    pub const fn consumer_attempt(&self) -> SourceFlagsAttempt {
        self.consumer_attempt
    }

    pub fn consumer_transaction_id(&self) -> &CallbackTransactionId {
        &self.consumer_transaction_id
    }

    pub const fn consumer_source_index(&self) -> CodecSourceIndex {
        self.consumer_source_index
    }

    pub const fn consumer_dispatch(&self) -> JocDispatchIndex {
        self.consumer_dispatch
    }

    pub const fn consumer_au_index(&self) -> Option<PublishedAuIndex> {
        self.consumer_au_index
    }

    pub const fn disposition(&self) -> Id14PopDisposition {
        self.disposition
    }

    fn as_json(&self) -> Value {
        json!({
            "queue_id": self.queue_id,
            "writer_generation": self.writer_generation.value(),
            "payload_hex": hex_encode(&self.payload),
            "producer": self.producer.as_json(),
            "consumer_programme_id": self.consumer_programme_id.as_str(),
            "consumer_attempt": self.consumer_attempt.value(),
            "consumer_transaction_id": self.consumer_transaction_id.as_str(),
            "consumer_source_index": self.consumer_source_index.value(),
            "consumer_dispatch": self.consumer_dispatch.value(),
            "consumer_au_index": self.consumer_au_index.map(PublishedAuIndex::value),
            "disposition": self.disposition.as_str(),
        })
    }
}

#[derive(Debug)]
struct PublishedPayload {
    id: u8,
    writer_generation: CodecSourceIndex,
    payload: Vec<u8>,
}

impl PublishedPayload {
    fn as_json(&self) -> Value {
        json!({
            "id": self.id,
            "writer_generation": self.writer_generation.value(),
            "payload_hex": hex_encode(&self.payload),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingPayload {
    queue_id: u8,
    writer_generation: CodecSourceIndex,
    payload_sha256: [u8; 32],
    disposition: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MetadataState {
    profile_id: &'static str,
    provider_sha256: [u8; 32],
    generation: u64,
    next_source_index: u64,
    carriage_mode: &'static str,
    timer: TimerState,
    queues: BTreeMap<u8, PayloadQueue>,
    finalized: bool,
    pending: Vec<PendingPayload>,
}

impl MetadataState {
    fn cold(provider_sha256: [u8; 32]) -> Self {
        let queues = [
            PayloadQueue::cold(11, 160, 4608, 3),
            PayloadQueue::cold(1, 10, 4608, 3),
            PayloadQueue::cold(2, 8, 4608, 3),
            PayloadQueue::cold(14, 543, 3072, 2),
        ]
        .into_iter()
        .map(|queue| (queue.id, queue))
        .collect();
        Self {
            profile_id: PROFILE_ID,
            provider_sha256,
            generation: 0,
            next_source_index: 0,
            carriage_mode: CARRIAGE_MODE,
            timer: TimerState::cold(),
            queues,
            finalized: false,
            pending: Vec::new(),
        }
    }

    fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = b"MetadataStateV1\0".to_vec();
        append_text(&mut bytes, self.profile_id);
        bytes.extend_from_slice(&self.provider_sha256);
        bytes.extend_from_slice(&self.generation.to_le_bytes());
        bytes.extend_from_slice(&self.next_source_index.to_le_bytes());
        append_text(&mut bytes, self.carriage_mode);
        for value in self
            .timer
            .remaining
            .into_iter()
            .chain(self.timer.cached)
            .chain([self.timer.mode, self.timer.counter])
        {
            bytes.extend_from_slice(&value.to_le_bytes());
        }

        for queue_id in CANONICAL_QUEUE_ORDER {
            let queue = &self.queues[&queue_id];
            bytes.push(queue.id);
            bytes.extend_from_slice(&queue.capacity.to_le_bytes());
            bytes.extend_from_slice(&queue.delay.to_le_bytes());
            bytes.extend_from_slice(&queue.frame.to_le_bytes());
            bytes.extend_from_slice(&queue.write_slot.to_le_bytes());
            bytes.extend_from_slice(&(queue.slots.len() as u16).to_le_bytes());
            for slot in &queue.slots {
                match slot {
                    None => bytes.push(0),
                    Some(body) => {
                        bytes.push(1);
                        bytes.extend_from_slice(&body.writer_generation.value().to_le_bytes());
                        bytes.extend_from_slice(&(body.payload.len() as u16).to_le_bytes());
                        bytes.extend_from_slice(&body.payload);
                    }
                }
            }
            for buffer_id in &queue.buffer_ids {
                bytes.extend_from_slice(&buffer_id.to_le_bytes());
            }
        }

        bytes.push(u8::from(self.finalized));
        bytes.extend_from_slice(&(self.pending.len() as u16).to_le_bytes());
        for pending in &self.pending {
            bytes.push(pending.queue_id);
            bytes.extend_from_slice(&pending.writer_generation.value().to_le_bytes());
            bytes.extend_from_slice(&pending.payload_sha256);
            append_text(&mut bytes, pending.disposition);
        }
        let provenance_slots =
            CANONICAL_QUEUE_ORDER
                .iter()
                .flat_map(|&queue_id| {
                    self.queues[&queue_id].slots.iter().enumerate().filter_map(
                        move |(slot, body)| {
                            body.as_ref()
                                .and_then(|body| body.producer_provenance.as_ref())
                                .map(|provenance| (queue_id, slot, provenance))
                        },
                    )
                })
                .collect::<Vec<_>>();
        if !provenance_slots.is_empty() {
            bytes.extend_from_slice(b"MetadataQueueProvenanceV1\0");
            bytes.extend_from_slice(&(provenance_slots.len() as u16).to_le_bytes());
            for (queue_id, slot, provenance) in provenance_slots {
                bytes.push(queue_id);
                bytes.extend_from_slice(&(slot as u16).to_le_bytes());
                provenance.append_canonical(&mut bytes);
            }
        }
        bytes
    }

    fn sha256_hex(&self) -> String {
        sha256_hex(&self.canonical_bytes())
    }

    fn digest(&self) -> [u8; 32] {
        Sha256::digest(self.canonical_bytes()).into()
    }

    fn as_json(&self) -> Value {
        let queues = self
            .queues
            .iter()
            .map(|(id, queue)| (id.to_string(), queue.state_json()))
            .collect::<serde_json::Map<_, _>>();
        json!({
            "profile_id": self.profile_id,
            "provider_sha256": hex_encode(&self.provider_sha256),
            "generation": self.generation,
            "next_source_index": self.next_source_index,
            "carriage_mode": self.carriage_mode,
            "timer": {
                "remaining": self.timer.remaining,
                "cached": self.timer.cached,
                "mode": self.timer.mode,
                "counter": self.timer.counter,
            },
            "queues": queues,
            "finalized": self.finalized,
            "pending": self.pending.iter().map(PendingPayload::as_json).collect::<Vec<_>>(),
        })
    }
}

impl PendingPayload {
    fn as_json(&self) -> Value {
        json!({
            "queue_id": self.queue_id,
            "writer_generation": self.writer_generation.value(),
            "payload_sha256": hex_encode(&self.payload_sha256),
            "disposition": self.disposition,
        })
    }
}

/// Private single-owner metadata state. Source calls stage on a clone and replace this root once.
pub struct MetadataOwner {
    programme_id: ProgrammeId,
    attempt: AttemptId,
    expected_sources: u64,
    id1_configuration: Id1MeasurementConfiguration,
    provider_file: Vec<u8>,
    wrapper_file: Vec<u8>,
    provider_sha256: [u8; 32],
    wrapper_configuration: Value,
    wrapper_config_sha256: String,
    state: MetadataState,
    retired: bool,
}

impl MetadataOwner {
    /// Creates fresh timer and queue state around one finalized programme ID1 configuration.
    ///
    /// The same immutable configuration is reused for every source staged on this owner.
    pub fn new(
        id1_configuration: Id1MeasurementConfiguration,
        expected_sources: u64,
        provider_file: &[u8],
        wrapper_file: &[u8],
    ) -> Result<Self, MetadataOwnerError> {
        if sha256_hex(provider_file) != PROVIDER_FILE_SHA256 {
            return Err(MetadataOwnerError::ProviderIdentity);
        }
        if sha256_hex(wrapper_file) != WRAPPER_FILE_SHA256 {
            return Err(MetadataOwnerError::WrapperIdentity);
        }

        let provider: Value = serde_json::from_slice(provider_file)
            .map_err(|_| MetadataOwnerError::ProviderIdentity)?;
        let wrapper_configuration: Value = serde_json::from_slice(wrapper_file)
            .map_err(|_| MetadataOwnerError::WrapperIdentity)?;
        let provider_canonical =
            serde_json::to_vec(&provider).map_err(|_| MetadataOwnerError::ProviderIdentity)?;
        let wrapper_canonical = serde_json::to_vec(&wrapper_configuration)
            .map_err(|_| MetadataOwnerError::WrapperIdentity)?;
        if sha256_hex(&provider_canonical) != PROVIDER_CANONICAL_SHA256 {
            return Err(MetadataOwnerError::ProviderIdentity);
        }
        let wrapper_config_sha256 = sha256_hex(&wrapper_canonical);
        if wrapper_config_sha256 != WRAPPER_CANONICAL_SHA256
            || provider["wrapper_config_sha256"].as_str() != Some(&wrapper_config_sha256)
        {
            return Err(MetadataOwnerError::WrapperIdentity);
        }

        let provider_sha256 =
            decode_sha256(PROVIDER_CANONICAL_SHA256).ok_or(MetadataOwnerError::ProviderIdentity)?;
        Ok(Self {
            programme_id: id1_configuration.programme_id().clone(),
            attempt: id1_configuration.attempt(),
            expected_sources,
            id1_configuration,
            provider_file: provider_file.to_vec(),
            wrapper_file: wrapper_file.to_vec(),
            provider_sha256,
            wrapper_configuration,
            wrapper_config_sha256,
            state: MetadataState::cold(provider_sha256),
            retired: false,
        })
    }

    /// Stages one complete metadata source on a private clone. It neither
    /// replaces the committed leaf state nor retires the programme on failure.
    pub fn stage_source(
        &self,
        input: MetadataStageInput,
    ) -> Result<MetadataSourceProposal, MetadataOwnerError> {
        if self.retired {
            return Err(MetadataOwnerError::Retired);
        }
        let (bodies_lineage, bodies) = input.bodies.into_parts();
        let validate_current_container = bodies.id14_provenance.is_some();
        if self.id1_configuration.programme_id() != bodies_lineage.programme_id()
            || self.id1_configuration.attempt() != bodies_lineage.attempt()
        {
            return Err(MetadataOwnerError::ProgrammeIdentity);
        }
        if let Some(provenance) = &bodies.id14_provenance {
            if provenance.producer_transaction_id() != Some(&input.transaction_id) {
                return Err(MetadataOwnerError::Id14Provenance);
            }
        }
        let call = MetadataSourceCall {
            programme_id: bodies_lineage.programme_id().clone(),
            attempt: bodies_lineage.attempt(),
            source_lineage: bodies_lineage.transaction_id().clone(),
            consumer_transaction_id: Some(input.transaction_id),
            source_index: bodies_lineage.source_index(),
            dispatch: bodies_lineage.dispatch(),
            au_index: input.au_index,
            config: self.id1_configuration.json_values(),
            bodies,
            wrapper_configuration: self.wrapper_configuration.clone(),
            validate_current_container,
        };
        let before_snapshot = self.snapshot();
        let processed = self.process_source_inner(call)?;
        let after_owner = self.with_after_state(processed.after_state);
        let after_snapshot = after_owner.snapshot();
        let consumer_transaction_id = processed.receipt.consumer_transaction_id.clone();
        Ok(MetadataSourceProposal {
            after_owner,
            receipt: processed.receipt,
            emdf: processed.emdf,
            bodies_lineage,
            consumer_transaction_id,
            parsed_container: processed.parsed_container,
            before_snapshot,
            after_snapshot,
        })
    }

    pub(crate) fn snapshot(&self) -> MetadataOwnerSnapshot {
        MetadataOwnerSnapshot {
            generation: self.state.generation,
            state_sha256: self.state.digest(),
            next_source_index: CodecSourceIndex::new(self.state.next_source_index),
        }
    }

    /// Returns the complete current owner value under the approved logical GTC state_ref.
    pub fn state_reference(&self) -> MetadataOwnerStateReference {
        let canonical_bytes = self.state.canonical_bytes();
        let cold_state = MetadataState::cold(self.provider_sha256);
        MetadataOwnerStateReference {
            generation: self.state.generation,
            state_sha256: self.state.digest(),
            cold_sha256: cold_state.digest(),
            canonical_bytes,
        }
    }

    /// Projects an unchanged receipt for a parent-supplied priming callback clock.
    ///
    /// Only callbacks 0 and 1 have no codec source stage. The owner must still be its exact cold
    /// root; this receipt carries no install or parent-admission authority.
    pub fn gtc_unchanged_callback_receipt(
        &self,
        callback_index: FrontendCallbackIndex,
        transaction_id: CallbackTransactionId,
    ) -> Result<GtcOwnerReceipt, MetadataGtcReceiptError> {
        if self.retired || self.state.finalized {
            return Err(MetadataGtcReceiptError::OwnerRetired);
        }
        if callback_index.value() > 1 || source_for_callback(callback_index).is_some() {
            return Err(MetadataGtcReceiptError::CallbackHasSourceStage);
        }
        if self.state != MetadataState::cold(self.provider_sha256) {
            return Err(MetadataGtcReceiptError::CallbackHasUnexpectedSource);
        }

        let reference = self.state_reference();
        let snapshot = reference
            .gtc_snapshot()
            .map_err(MetadataGtcReceiptError::GtcReceipt)?;
        GtcOwnerReceipt::from_transition(snapshot, snapshot, transaction_id)
            .map_err(MetadataGtcReceiptError::GtcReceipt)
    }

    #[allow(dead_code)] // Read through MetadataSourceProposal by the selected JOC root adapter.
    fn id14_queue_view(&self) -> Result<MetadataId14QueueView, MetadataOwnerError> {
        let queue = self
            .state
            .queues
            .get(&14)
            .ok_or(MetadataOwnerError::EmdfSerialization)?;
        let slots: [Option<MetadataQueueBodyView>; 2] = queue
            .slots
            .iter()
            .map(|slot| {
                slot.as_ref().map(|body| MetadataQueueBodyView {
                    writer_generation: body.writer_generation,
                    bytes: body.payload.clone().into_boxed_slice(),
                    producer_provenance: body.producer_provenance.clone(),
                })
            })
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| MetadataOwnerError::EmdfSerialization)?;
        let buffer_ids: [u16; 2] = queue
            .buffer_ids
            .clone()
            .try_into()
            .map_err(|_| MetadataOwnerError::EmdfSerialization)?;
        Ok(MetadataId14QueueView {
            slots,
            buffer_ids,
            next_writer: CodecSourceIndex::new(self.state.next_source_index),
        })
    }

    fn with_after_state(&self, state: MetadataState) -> Self {
        Self {
            programme_id: self.programme_id.clone(),
            attempt: self.attempt,
            expected_sources: self.expected_sources,
            id1_configuration: self.id1_configuration.clone(),
            provider_file: self.provider_file.clone(),
            wrapper_file: self.wrapper_file.clone(),
            provider_sha256: self.provider_sha256,
            wrapper_configuration: self.wrapper_configuration.clone(),
            wrapper_config_sha256: self.wrapper_config_sha256.clone(),
            state,
            retired: self.retired,
        }
    }

    #[cfg(test)]
    fn process_source(
        &mut self,
        call: MetadataSourceCall,
    ) -> Result<MetadataSourceResult, MetadataOwnerError> {
        if self.retired {
            return Err(MetadataOwnerError::Retired);
        }

        let result = self.process_source_inner(call);
        match result {
            Ok(processed) => {
                self.state = processed.after_state;
                Ok(MetadataSourceResult {
                    receipt: processed.receipt,
                    emdf: processed.emdf,
                })
            }
            Err(error) => {
                self.retired = true;
                Err(error)
            }
        }
    }

    fn process_source_inner(
        &self,
        call: MetadataSourceCall,
    ) -> Result<MetadataSourceProcessed, MetadataOwnerError> {
        if self.state.finalized {
            return Err(MetadataOwnerError::Finalized);
        }
        if call.programme_id != self.programme_id || call.attempt != self.attempt {
            return Err(MetadataOwnerError::ProgrammeIdentity);
        }

        let source = call.source_index.value();
        if source != self.state.next_source_index || source >= self.expected_sources {
            return Err(MetadataOwnerError::SourceOrder);
        }
        if call.dispatch.value() != source + 1 {
            return Err(MetadataOwnerError::DispatchOrder);
        }
        let expected_au = source.checked_sub(2).map(PublishedAuIndex::new);
        if call.au_index != expected_au {
            return Err(MetadataOwnerError::AuOrder);
        }
        let source_identity = MetadataSourceIdentity::from_call(&call);

        let wrapper_canonical = serde_json::to_vec(&call.wrapper_configuration)
            .map_err(|_| MetadataOwnerError::WrapperConfiguration)?;
        if sha256_hex(&wrapper_canonical) != self.wrapper_config_sha256
            || call.wrapper_configuration != self.wrapper_configuration
        {
            return Err(MetadataOwnerError::WrapperConfiguration);
        }

        let id11 = call.bodies.id11.ok_or(MetadataOwnerError::BodyReadiness)?;
        let id2 = call.bodies.id2.ok_or(MetadataOwnerError::BodyReadiness)?;
        let id14 = call.bodies.id14.ok_or(MetadataOwnerError::BodyReadiness)?;
        for body in [&id11, &id2, &id14] {
            if body.writer_generation.value() != source {
                return Err(MetadataOwnerError::WriterGeneration);
            }
        }
        for (body, capacity) in [(&id11, 160), (&id2, 8), (&id14, 543)] {
            if body.bytes.len() > capacity {
                return Err(MetadataOwnerError::Capacity);
            }
        }
        if let Some(provenance) = &call.bodies.id14_provenance {
            let lineage = provenance.lineage();
            if lineage.programme_id() != &call.programme_id
                || lineage.attempt() != call.attempt
                || lineage.transaction_id() != &call.source_lineage
                || lineage.source_index() != call.source_index
                || lineage.dispatch() != call.dispatch
                || provenance.producer_generation() != call.source_index
                || provenance.dispatch() != call.dispatch
                || provenance.writer().value() != call.source_index.value()
                || provenance.producer_transaction_id() != call.consumer_transaction_id.as_ref()
                || !provenance.matches_body(call.source_index, &id14.bytes)
            {
                return Err(MetadataOwnerError::Id14Provenance);
            }
        }

        let timer_before_sha256 = hex_encode(&self.state.timer.digest());
        let timer_advance = self.state.timer.advance(&call.config, 1536, false)?;
        let id1_payload = encode_id1_payload(&timer_advance.output)?;
        if id1_payload.len() > 10 {
            return Err(MetadataOwnerError::Capacity);
        }

        let type11_producer_provenance = metadata_body_provenance_for_source(
            &source_identity,
            id11.writer_generation,
            &id11.bytes,
        )?;
        let programme_producer_provenance =
            metadata_body_provenance_for_source(&source_identity, call.source_index, &id1_payload)?;
        let additional_bsi_producer_provenance = metadata_body_provenance_for_source(
            &source_identity,
            id2.writer_generation,
            &id2.bytes,
        )?;
        let id14_provenance = call
            .bodies
            .id14_provenance
            .clone()
            .map(MetadataQueueProducerProvenance::Id14);

        let before_generation = self.state.generation;
        let before_sha256 = self.state.sha256_hex();
        let mut next_state = self.state.clone();
        next_state.timer = timer_advance.next;

        let bodies = BTreeMap::from([
            (
                11,
                QueuedBody {
                    writer_generation: id11.writer_generation,
                    payload: id11.bytes,
                    producer_provenance: type11_producer_provenance,
                },
            ),
            (
                1,
                QueuedBody {
                    writer_generation: call.source_index,
                    payload: id1_payload,
                    producer_provenance: programme_producer_provenance,
                },
            ),
            (
                2,
                QueuedBody {
                    writer_generation: id2.writer_generation,
                    payload: id2.bytes,
                    producer_provenance: additional_bsi_producer_provenance,
                },
            ),
            (
                14,
                QueuedBody {
                    writer_generation: id14.writer_generation,
                    payload: id14.bytes,
                    producer_provenance: id14_provenance,
                },
            ),
        ]);

        let mut operations = Vec::with_capacity(QUEUE_MUTATION_ORDER.len());
        for queue_id in QUEUE_MUTATION_ORDER {
            let body = bodies
                .get(&queue_id)
                .ok_or(MetadataOwnerError::BodyReadiness)?;
            let queue = next_state
                .queues
                .get_mut(&queue_id)
                .ok_or(MetadataOwnerError::EmdfSerialization)?;
            operations.push(queue.push_then_pop(body)?);
        }
        if call.bodies.id14_provenance.is_some() {
            let consumer_transaction_id = call
                .consumer_transaction_id
                .clone()
                .ok_or(MetadataOwnerError::Id14Provenance)?;
            for operation in &mut operations {
                operation.bind_consumer(
                    consumer_transaction_id.clone(),
                    call.source_index,
                    call.dispatch,
                );
            }
        }

        let ordinary_writer = source.checked_sub(2).map(CodecSourceIndex::new);
        let joc_writer = source.checked_sub(1).map(CodecSourceIndex::new);
        for operation in &operations {
            let expected_writer = if operation.queue_id == 14 {
                joc_writer
            } else {
                ordinary_writer
            };
            if operation.popped_writer != expected_writer {
                return Err(MetadataOwnerError::WriterGeneration);
            }
        }

        let id14_popped_body = operations
            .iter()
            .find(|operation| operation.queue_id == 14)
            .and_then(|operation| operation.popped.as_ref());
        let id14_queue_pop = if let Some(body) = id14_popped_body {
            match body.producer_provenance.as_ref() {
                Some(MetadataQueueProducerProvenance::Id14(provenance)) => {
                    let producer_lineage = provenance.lineage();
                    if provenance.producer_generation() != body.writer_generation
                        || provenance.writer().value() != body.writer_generation.value()
                        || producer_lineage.source_index() != body.writer_generation
                        || body.writer_generation.value().checked_add(1)
                            != Some(producer_lineage.dispatch().value())
                        || producer_lineage.programme_id() != &call.programme_id
                        || producer_lineage.attempt() != call.attempt
                        || provenance.producer_transaction_id().is_none()
                        || !provenance.matches_body(body.writer_generation, &body.payload)
                    {
                        return Err(MetadataOwnerError::Id14Provenance);
                    }
                    Some(Id14QueuePopReceipt {
                        producer: provenance.clone(),
                        payload: body.payload.clone().into_boxed_slice(),
                        consumer_programme_id: call.programme_id.clone(),
                        consumer_attempt: call.attempt,
                        consumer_transaction_id: call
                            .consumer_transaction_id
                            .clone()
                            .ok_or(MetadataOwnerError::Id14Provenance)?,
                        consumer_source_index: call.source_index,
                        consumer_dispatch: call.dispatch,
                        consumer_au_index: call.au_index,
                        disposition: if call.au_index.is_some() {
                            Id14PopDisposition::Published
                        } else {
                            Id14PopDisposition::Suppressed
                        },
                    })
                }
                Some(MetadataQueueProducerProvenance::MetadataBody(_)) => {
                    return Err(MetadataOwnerError::Id14Provenance);
                }
                None => None,
            }
        } else {
            None
        };

        let mut queue_pop_receipts = Vec::new();
        for queue_id in PUBLICATION_ORDER {
            let Some(operation) = operations
                .iter()
                .find(|operation| operation.queue_id == queue_id)
            else {
                return Err(MetadataOwnerError::EmdfSerialization);
            };
            let Some(body) = operation.popped.as_ref() else {
                continue;
            };
            let Some(producer) = body.producer_provenance.as_ref() else {
                continue;
            };
            let consumer_transaction_id = call
                .consumer_transaction_id
                .as_ref()
                .ok_or(MetadataOwnerError::Id14Provenance)?;
            if !producer.matches_body(body.writer_generation, &body.payload)
                || operation.consumer_transaction_id.as_ref() != Some(consumer_transaction_id)
                || operation.consumer_source_index != Some(call.source_index)
                || operation.consumer_dispatch != Some(call.dispatch)
            {
                return Err(MetadataOwnerError::Id14Provenance);
            }
            queue_pop_receipts.push(MetadataQueuePopReceipt {
                queue_id,
                writer_generation: body.writer_generation,
                payload: body.payload.clone().into_boxed_slice(),
                producer: producer.clone(),
                consumer_programme_id: call.programme_id.clone(),
                consumer_attempt: call.attempt,
                consumer_transaction_id: consumer_transaction_id.clone(),
                consumer_source_index: call.source_index,
                consumer_dispatch: call.dispatch,
                consumer_au_index: call.au_index,
                disposition: if call.au_index.is_some() {
                    Id14PopDisposition::Published
                } else {
                    Id14PopDisposition::Suppressed
                },
            });
        }

        let mut published_payloads = Vec::new();
        let mut emdf_output: Option<EmdfOutput> = None;
        if call.au_index.is_some() {
            for queue_id in PUBLICATION_ORDER {
                let operation = operations
                    .iter()
                    .find(|operation| operation.queue_id == queue_id)
                    .ok_or(MetadataOwnerError::EmdfSerialization)?;
                let body = operation
                    .popped
                    .as_ref()
                    .ok_or(MetadataOwnerError::WriterGeneration)?;
                published_payloads.push(PublishedPayload {
                    id: queue_id,
                    writer_generation: body.writer_generation,
                    payload: body.payload.clone(),
                });
            }
            let serialization_inputs = published_payloads
                .iter()
                .map(|payload| {
                    let body = operations
                        .iter()
                        .find(|operation| operation.queue_id == payload.id)
                        .and_then(|operation| operation.popped.as_ref())
                        .ok_or(MetadataOwnerError::EmdfSerialization)?;
                    Ok((payload.id, body))
                })
                .collect::<Result<Vec<_>, MetadataOwnerError>>()?;
            emdf_output = Some(serialize_emdf(
                &self.wrapper_configuration,
                &serialization_inputs,
            )?);
        }

        next_state.generation = next_state
            .generation
            .checked_add(1)
            .ok_or(MetadataOwnerError::SourceOrder)?;
        next_state.next_source_index = next_state
            .next_source_index
            .checked_add(1)
            .ok_or(MetadataOwnerError::SourceOrder)?;
        let proposed_after_generation = next_state.generation;
        let proposed_after_sha256 = next_state.sha256_hex();
        let timer_after_sha256 = hex_encode(&next_state.timer.digest());
        let terminal_layout = emdf_output
            .as_ref()
            .map(|output| output.terminal_layout.clone());
        let emdf = emdf_output.map(|output| output.bytes);
        let parsed_container = if call.validate_current_container && source >= 2 {
            let bytes = emdf
                .as_deref()
                .ok_or(MetadataOwnerError::CurrentEmdfValidation)?;
            let queue_pop = id14_queue_pop
                .as_ref()
                .ok_or(MetadataOwnerError::CurrentEmdfValidation)?;
            let consumer_transaction_id = call
                .consumer_transaction_id
                .as_ref()
                .ok_or(MetadataOwnerError::CurrentEmdfValidation)?;
            let consumer_au_index = call
                .au_index
                .ok_or(MetadataOwnerError::CurrentEmdfValidation)?;
            if queue_pop.disposition() != Id14PopDisposition::Published
                || queue_pop.consumer_transaction_id() != consumer_transaction_id
                || queue_pop.consumer_source_index() != call.source_index
                || queue_pop.consumer_dispatch() != call.dispatch
                || queue_pop.consumer_au_index() != Some(consumer_au_index)
            {
                return Err(MetadataOwnerError::CurrentEmdfValidation);
            }
            let carriage_mode = match CARRIAGE_MODE {
                "deployed_compat" => {
                    crate::joc::emdf::protected_container::CurrentEmdfCarriageMode::DeployedCompat
                }
                "etsi_emdf" => {
                    crate::joc::emdf::protected_container::CurrentEmdfCarriageMode::EtsiEmdf
                }
                _ => return Err(MetadataOwnerError::CurrentEmdfValidation),
            };
            let carrier_block = crate::joc::emdf::protected_container::CarrierBlock::new(2)
                .map_err(|_| MetadataOwnerError::CurrentEmdfValidation)?;
            let parsed_container =
                crate::joc::emdf::protected_container::parse_current_emdf_container(
                    bytes,
                    carriage_mode,
                    carrier_block,
                    queue_pop.producer(),
                )
                .map_err(|_| MetadataOwnerError::CurrentEmdfValidation)?;
            if parsed_container.id14_provenance() != queue_pop.producer() {
                return Err(MetadataOwnerError::CurrentEmdfValidation);
            }
            validate_current_container_pop_bindings(
                &parsed_container,
                &queue_pop_receipts,
                &source_identity,
            )?;
            Some(ParsedMetadataSourceContainer {
                parsed_container,
                consumer_programme_id: call.programme_id.clone(),
                consumer_attempt: call.attempt,
                consumer_transaction_id: consumer_transaction_id.clone(),
                consumer_source_index: call.source_index,
                consumer_dispatch: call.dispatch,
                consumer_au_index,
                id14_queue_pop: queue_pop.clone(),
                queue_pop_receipts: queue_pop_receipts.clone(),
            })
        } else {
            None
        };

        let receipt = MetadataSourceReceipt {
            programme_id: call.programme_id,
            attempt: call.attempt,
            transaction_id: call.source_lineage,
            consumer_transaction_id: call.consumer_transaction_id,
            source_index: call.source_index,
            dispatch: call.dispatch,
            au_index: call.au_index,
            before_generation,
            proposed_after_generation,
            before_sha256,
            proposed_after_sha256,
            provider_sha256: hex_encode(&self.provider_sha256),
            wrapper_config_sha256: self.wrapper_config_sha256.clone(),
            timer_before_sha256,
            timer_after_sha256,
            timer_refreshed_field: timer_advance.refreshed_field,
            queue_operations: operations,
            published_payloads,
            queue_pop_receipts,
            id14_queue_pop,
            emdf: emdf.clone(),
            terminal_layout,
        };
        Ok(MetadataSourceProcessed {
            after_state: next_state,
            receipt,
            emdf,
            parsed_container,
        })
    }

    pub fn prepare_eof(
        &mut self,
        actual_sources: u64,
    ) -> Result<MetadataEofProposal, MetadataOwnerError> {
        if self.retired {
            return Err(MetadataOwnerError::Retired);
        }
        if self.state.finalized {
            return Err(MetadataOwnerError::Finalized);
        }
        if actual_sources != self.expected_sources || actual_sources != self.state.next_source_index
        {
            self.retired = true;
            return Err(MetadataOwnerError::EofSourceCount);
        }

        let before = self.state.as_json();
        let mut after_state = self.state.clone();
        after_state.generation = after_state
            .generation
            .checked_add(1)
            .ok_or(MetadataOwnerError::EofSourceCount)?;
        after_state.finalized = true;
        after_state.pending.clear();
        for queue_id in CANONICAL_QUEUE_ORDER {
            let queue = after_state
                .queues
                .get_mut(&queue_id)
                .ok_or(MetadataOwnerError::EmdfSerialization)?;
            for body in queue.slots.iter().flatten() {
                after_state.pending.push(PendingPayload {
                    queue_id,
                    writer_generation: body.writer_generation,
                    payload_sha256: Sha256::digest(&body.payload).into(),
                    disposition: "DROP_AT_DONE",
                });
            }
            queue.slots.fill(None);
        }

        Ok(MetadataEofProposal {
            programme_id: self.programme_id.clone(),
            attempt: self.attempt,
            expected_sources: actual_sources,
            before,
            after_state,
        })
    }

    /// Starts a fresh cold owner with the finalized ID1 configuration for the new attempt.
    ///
    /// Measurement itself is supplied by the enclosing programme model and is not rerun here.
    pub fn reset_for_attempt(
        &self,
        attempt: AttemptId,
        id1_configuration: Id1MeasurementConfiguration,
    ) -> Result<Self, MetadataOwnerError> {
        let expected_attempt = self
            .attempt
            .value()
            .checked_add(1)
            .ok_or(MetadataOwnerError::AttemptSequence)?;
        if !self.retired || attempt.value() != expected_attempt {
            return Err(MetadataOwnerError::AttemptSequence);
        }
        if id1_configuration.programme_id() != &self.programme_id
            || id1_configuration.attempt() != attempt
        {
            return Err(MetadataOwnerError::ProgrammeIdentity);
        }
        Self::new(
            id1_configuration,
            self.expected_sources,
            &self.provider_file,
            &self.wrapper_file,
        )
    }

    pub const fn is_retired(&self) -> bool {
        self.retired
    }
}

fn validate_current_container_pop_bindings(
    parsed: &crate::joc::emdf::protected_container::ParsedCurrentEmdfContainer,
    queue_pops: &[MetadataQueuePopReceipt],
    consumer: &MetadataSourceIdentity,
) -> Result<(), MetadataOwnerError> {
    let consumer_transaction_id = consumer
        .transaction_id
        .as_ref()
        .ok_or(MetadataOwnerError::CurrentEmdfValidation)?;
    let consumer_au_index = consumer
        .au_index
        .ok_or(MetadataOwnerError::CurrentEmdfValidation)?;
    if parsed.payloads().len() != PUBLICATION_ORDER.len()
        || queue_pops.len() != PUBLICATION_ORDER.len()
    {
        return Err(MetadataOwnerError::CurrentEmdfValidation);
    }
    let ordinary_writer = consumer
        .source_index
        .value()
        .checked_sub(2)
        .map(CodecSourceIndex::new)
        .ok_or(MetadataOwnerError::CurrentEmdfValidation)?;
    let joc_writer = consumer
        .source_index
        .value()
        .checked_sub(1)
        .map(CodecSourceIndex::new)
        .ok_or(MetadataOwnerError::CurrentEmdfValidation)?;

    for ((expected_id, parsed_payload), queue_pop) in PUBLICATION_ORDER
        .iter()
        .copied()
        .zip(parsed.payloads())
        .zip(queue_pops)
    {
        let expected_writer = if expected_id == 14 {
            joc_writer
        } else {
            ordinary_writer
        };
        if parsed_payload.id().wire_value() != expected_id
            || queue_pop.queue_id() != expected_id
            || queue_pop.writer_generation() != expected_writer
            || parsed_payload.bytes() != queue_pop.payload()
            || queue_pop.consumer_programme_id() != &consumer.programme_id
            || queue_pop.consumer_attempt() != consumer.attempt
            || queue_pop.consumer_transaction_id() != consumer_transaction_id
            || queue_pop.consumer_source_index() != consumer.source_index
            || queue_pop.consumer_dispatch() != consumer.dispatch
            || queue_pop.consumer_au_index() != Some(consumer_au_index)
            || queue_pop.disposition() != Id14PopDisposition::Published
            || queue_pop.producer().producer_transaction_id().is_none()
        {
            return Err(MetadataOwnerError::CurrentEmdfValidation);
        }

        let producer_identity_matches = match queue_pop.producer() {
            MetadataQueueProducerProvenance::MetadataBody(producer) => {
                expected_id != 14
                    && producer.programme_id() == &consumer.programme_id
                    && producer.attempt() == consumer.attempt
                    && producer.producer_generation() == expected_writer
                    && producer.writer().value() == expected_writer.value()
                    && expected_writer.value().checked_add(1) == Some(producer.dispatch().value())
            }
            MetadataQueueProducerProvenance::Id14(producer) => {
                expected_id == 14
                    && producer.lineage().programme_id() == &consumer.programme_id
                    && producer.lineage().attempt() == consumer.attempt
                    && producer.producer_generation() == expected_writer
                    && producer.writer().value() == expected_writer.value()
                    && expected_writer.value().checked_add(1) == Some(producer.dispatch().value())
                    && producer.lineage().source_index() == expected_writer
                    && producer.lineage().dispatch().value() == producer.dispatch().value()
            }
        };
        if !producer_identity_matches {
            return Err(MetadataOwnerError::CurrentEmdfValidation);
        }

        let configuration = parsed_payload.configuration();
        let wrapper_configuration_matches = match expected_id {
            1 => {
                configuration.sample_offset().is_none()
                    && configuration.duration() == Some(1536)
                    && configuration.group_id().is_none()
                    && configuration.codec_data_reserved().is_none()
                    && !configuration.discard_unknown()
                    && configuration.frame_aligned() == Some(false)
                    && configuration.create_duplicate().is_none()
                    && configuration.remove_duplicate().is_none()
                    && configuration.priority().is_none()
                    && configuration.processing_allowed().is_none()
            }
            2 => {
                configuration.sample_offset().is_none()
                    && configuration.duration().is_none()
                    && configuration.group_id().is_none()
                    && configuration.codec_data_reserved().is_none()
                    && configuration.discard_unknown()
                    && configuration.frame_aligned().is_none()
                    && configuration.create_duplicate().is_none()
                    && configuration.remove_duplicate().is_none()
                    && configuration.priority().is_none()
                    && configuration.processing_allowed().is_none()
            }
            _ => true,
        };
        if !wrapper_configuration_matches {
            return Err(MetadataOwnerError::CurrentEmdfValidation);
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct MetadataEofProposal {
    programme_id: ProgrammeId,
    attempt: AttemptId,
    expected_sources: u64,
    before: Value,
    after_state: MetadataState,
}

impl MetadataEofProposal {
    pub fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub const fn attempt(&self) -> AttemptId {
        self.attempt
    }

    pub const fn expected_sources(&self) -> u64 {
        self.expected_sources
    }

    pub const fn after_is_finalized(&self) -> bool {
        self.after_state.finalized
    }

    pub fn queues_are_empty(&self) -> bool {
        self.after_state
            .queues
            .values()
            .all(|queue| queue.slots.iter().all(Option::is_none))
    }

    pub fn pending_writers_are_drop_at_done(&self) -> bool {
        self.after_state
            .pending
            .iter()
            .all(|pending| pending.disposition == "DROP_AT_DONE")
    }

    pub fn pending_writer_dispositions(&self) -> Vec<(u8, CodecSourceIndex, &'static str)> {
        self.after_state
            .pending
            .iter()
            .map(|pending| {
                (
                    pending.queue_id,
                    pending.writer_generation,
                    pending.disposition,
                )
            })
            .collect()
    }

    pub fn as_json(&self) -> Value {
        json!({
            "expected_sources": self.expected_sources,
            "before": self.before,
            "after": self.after_state.as_json(),
            "after_canonical_hex": hex_encode(&self.after_state.canonical_bytes()),
            "after_sha256": self.after_state.sha256_hex(),
            "push_count": 0,
            "pop_count": 0,
            "timer_advance_count": 0,
            "extra_outputs": 0,
        })
    }

    pub const fn has_source_receipt(&self) -> bool {
        false
    }
}

fn append_text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u16).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn decode_sha256(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let mut digest = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let digit = |byte: u8| match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            _ => None,
        };
        digest[index] = (digit(pair[0])? << 4) | digit(pair[1])?;
    }
    Some(digest)
}

pub(super) fn hex_encode(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex_encode(&Sha256::digest(bytes))
}
