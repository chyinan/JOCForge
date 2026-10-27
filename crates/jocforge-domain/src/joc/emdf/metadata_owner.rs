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
use crate::{
    spatial::source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
    timeline::{CodecSourceIndex, JocDispatchIndex, PublishedAuIndex},
    transaction::{AttemptId, ProgrammeId},
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
    Timer,
    WireField,
    Capacity,
    QueueOccupied,
    EmdfSerialization,
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
            Self::Timer => "METADATA_TIMER",
            Self::WireField => "METADATA_WIRE_FIELD",
            Self::Capacity => "METADATA_CAPACITY",
            Self::QueueOccupied => "METADATA_QUEUE_OCCUPIED",
            Self::EmdfSerialization => "METADATA_EMDF_SERIALIZATION",
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataPayloadBody {
    pub writer_generation: CodecSourceIndex,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataBodies {
    pub id11: Option<MetadataPayloadBody>,
    pub id2: Option<MetadataPayloadBody>,
    pub id14: Option<MetadataPayloadBody>,
}

#[derive(Clone, Debug, PartialEq)]
struct MetadataSourceCall {
    pub programme_id: SourceFlagsProgrammeId,
    pub attempt: SourceFlagsAttempt,
    pub transaction_id: crate::transaction::CallbackLineageIdentity,
    pub source_index: CodecSourceIndex,
    pub dispatch: JocDispatchIndex,
    pub au_index: Option<PublishedAuIndex>,
    config: Vec<Value>,
    pub bodies: MetadataBodies,
    pub wrapper_configuration: Value,
}

/// Typed per-source input. The ID1 programme configuration and wrapper are
/// constructor-pinned and are not retransmitted or recomputed for each source.
pub struct MetadataStageInput {
    pub au_index: Option<PublishedAuIndex>,
    pub bodies: MetadataBodiesProposal,
}

#[must_use]
pub struct MetadataSourceProposal {
    after_owner: MetadataOwner,
    receipt: MetadataSourceReceipt,
    emdf: Option<Vec<u8>>,
    bodies_lineage: MetadataBodiesLineage,
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

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)] // Immutable bridge for the selected JOC root view adapter.
pub(crate) struct MetadataQueueBodyView {
    writer_generation: CodecSourceIndex,
    bytes: Box<[u8]>,
}

#[allow(dead_code)] // Immutable bridge for the selected JOC root view adapter.
impl MetadataQueueBodyView {
    pub(crate) const fn writer_generation(&self) -> CodecSourceIndex {
        self.writer_generation
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
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

    pub fn emdf(&self) -> Option<&[u8]> {
        self.emdf.as_deref()
    }

    pub const fn bodies_lineage(&self) -> &MetadataBodiesLineage {
        &self.bodies_lineage
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
    ) {
        (
            self.after_owner,
            self.receipt,
            self.emdf,
            self.bodies_lineage,
            self.before_snapshot,
            self.after_snapshot,
        )
    }
}

#[derive(Debug)]
pub struct MetadataSourceResult {
    pub receipt: MetadataSourceReceipt,
    pub emdf: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct MetadataSourceReceipt {
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: crate::transaction::CallbackLineageIdentity,
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

    pub fn as_json(&self) -> Value {
        json!({
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
        if self.id1_configuration.programme_id() != bodies_lineage.programme_id()
            || self.id1_configuration.attempt() != bodies_lineage.attempt()
        {
            return Err(MetadataOwnerError::ProgrammeIdentity);
        }
        let call = MetadataSourceCall {
            programme_id: bodies_lineage.programme_id().clone(),
            attempt: bodies_lineage.attempt(),
            transaction_id: bodies_lineage.transaction_id().clone(),
            source_index: bodies_lineage.source_index(),
            dispatch: bodies_lineage.dispatch(),
            au_index: input.au_index,
            config: self.id1_configuration.json_values(),
            bodies,
            wrapper_configuration: self.wrapper_configuration.clone(),
        };
        let before_snapshot = self.snapshot();
        let (after_state, receipt, emdf) = self.process_source_inner(call)?;
        let after_owner = self.with_after_state(after_state);
        let after_snapshot = after_owner.snapshot();
        Ok(MetadataSourceProposal {
            after_owner,
            receipt,
            emdf,
            bodies_lineage,
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
            Ok((next_state, receipt, emdf)) => {
                self.state = next_state;
                Ok(MetadataSourceResult { receipt, emdf })
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
    ) -> Result<(MetadataState, MetadataSourceReceipt, Option<Vec<u8>>), MetadataOwnerError> {
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

        let timer_before_sha256 = hex_encode(&self.state.timer.digest());
        let timer_advance = self.state.timer.advance(&call.config, 1536, false)?;
        let id1_payload = encode_id1_payload(&timer_advance.output)?;
        if id1_payload.len() > 10 {
            return Err(MetadataOwnerError::Capacity);
        }

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
                },
            ),
            (
                1,
                QueuedBody {
                    writer_generation: call.source_index,
                    payload: id1_payload,
                },
            ),
            (
                2,
                QueuedBody {
                    writer_generation: id2.writer_generation,
                    payload: id2.bytes,
                },
            ),
            (
                14,
                QueuedBody {
                    writer_generation: id14.writer_generation,
                    payload: id14.bytes,
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

        let receipt = MetadataSourceReceipt {
            programme_id: call.programme_id,
            attempt: call.attempt,
            transaction_id: call.transaction_id,
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
            emdf: emdf.clone(),
            terminal_layout,
        };
        Ok((next_state, receipt, emdf))
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
