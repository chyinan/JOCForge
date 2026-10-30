// pattern: Functional Core
//! Typed producer provenance for one successful selected ID14 representation.
//!
//! The F05 callback-lineage identity is retained separately from the optional parent-issued
//! callback transaction token. F13 admission requires the latter to be present.

use crate::{
    joc::{coefficient_fitting::FitSourceLineage, emdf::metadata_owner::MetadataPayloadBody},
    timeline::{CodecSourceIndex, JocDispatchIndex, JocGeneratedWriterIndex},
    transaction::CallbackTransactionId,
};
use sha2::{Digest, Sha256};
use std::fmt::Write as FmtWrite;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Id14ObjectCount(u8);

impl Id14ObjectCount {
    pub(crate) fn from_selected_slots(value: usize) -> Option<Self> {
        let value = u8::try_from(value).ok()?;
        (1..=15).contains(&value).then_some(Self(value))
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Id14SequenceCount(u16);

impl Id14SequenceCount {
    pub(crate) fn new(value: u16) -> Option<Self> {
        (value <= 1023).then_some(Self(value))
    }

    pub const fn value(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Id14ProducerProvenance {
    lineage: FitSourceLineage,
    producer_transaction_id: Option<CallbackTransactionId>,
    producer_generation: CodecSourceIndex,
    dispatch: JocDispatchIndex,
    writer: JocGeneratedWriterIndex,
    sequence_count: Id14SequenceCount,
    object_count: Id14ObjectCount,
    byte_length: u16,
    body_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Id14ProducerIdentity {
    pub(crate) lineage: FitSourceLineage,
    pub(crate) producer_transaction_id: Option<CallbackTransactionId>,
    pub(crate) producer_generation: CodecSourceIndex,
    pub(crate) dispatch: JocDispatchIndex,
    pub(crate) writer: JocGeneratedWriterIndex,
    pub(crate) sequence_count: Id14SequenceCount,
    pub(crate) object_count: Id14ObjectCount,
}

impl Id14ProducerProvenance {
    pub(crate) fn new(identity: Id14ProducerIdentity, body: &[u8]) -> Option<Self> {
        let byte_length = u16::try_from(body.len()).ok()?;
        if identity.lineage.source_index() != identity.producer_generation
            || identity.lineage.dispatch() != identity.dispatch
            || identity.writer.value() != identity.producer_generation.value()
            || identity.producer_generation.value().checked_add(1)
                != Some(identity.dispatch.value())
        {
            return None;
        }
        Some(Self {
            lineage: identity.lineage,
            producer_transaction_id: identity.producer_transaction_id,
            producer_generation: identity.producer_generation,
            dispatch: identity.dispatch,
            writer: identity.writer,
            sequence_count: identity.sequence_count,
            object_count: identity.object_count,
            byte_length,
            body_sha256: Sha256::digest(body).into(),
        })
    }

    pub const fn lineage(&self) -> &FitSourceLineage {
        &self.lineage
    }

    pub fn producer_transaction_id(&self) -> Option<&CallbackTransactionId> {
        self.producer_transaction_id.as_ref()
    }

    pub const fn producer_generation(&self) -> CodecSourceIndex {
        self.producer_generation
    }

    pub const fn dispatch(&self) -> JocDispatchIndex {
        self.dispatch
    }

    pub const fn writer(&self) -> JocGeneratedWriterIndex {
        self.writer
    }

    pub const fn sequence_count(&self) -> u16 {
        self.sequence_count.value()
    }

    pub const fn object_count(&self) -> Id14ObjectCount {
        self.object_count
    }

    pub const fn byte_length(&self) -> u16 {
        self.byte_length
    }

    pub const fn body_sha256(&self) -> &[u8; 32] {
        &self.body_sha256
    }

    pub(crate) fn matches_body(&self, writer_generation: CodecSourceIndex, body: &[u8]) -> bool {
        self.producer_generation == writer_generation
            && self.writer.value() == writer_generation.value()
            && u16::try_from(body.len()).ok() == Some(self.byte_length)
            && self.body_sha256 == <[u8; 32]>::from(Sha256::digest(body))
    }

    pub(crate) fn as_json(&self) -> serde_json::Value {
        serde_json::json!({
            "programme_id": self.lineage.programme_id().as_str(),
            "attempt": self.lineage.attempt().value(),
            "f05_callback_lineage_index": self.lineage.transaction_id().callback_index().value(),
            "producer_transaction_id": self.producer_transaction_id.as_ref().map(CallbackTransactionId::as_str),
            "producer_generation": self.producer_generation.value(),
            "dispatch": self.dispatch.value(),
            "writer": self.writer.value(),
            "sequence_count": self.sequence_count.value(),
            "object_count": self.object_count.value(),
            "byte_length": self.byte_length,
            "body_sha256": digest_hex(&self.body_sha256),
            "spatial_before_generation": self.lineage.spatial_before_generation(),
            "spatial_before_digest": self.lineage.spatial_before_digest().to_hex(),
        })
    }

    pub(crate) fn append_canonical(&self, bytes: &mut Vec<u8>) {
        let programme_id = self.lineage.programme_id().as_str().as_bytes();
        bytes.extend_from_slice(&(programme_id.len() as u32).to_le_bytes());
        bytes.extend_from_slice(programme_id);
        if let Some(transaction_id) = &self.producer_transaction_id {
            bytes.push(1);
            let transaction_bytes = transaction_id.as_str().as_bytes();
            bytes.extend_from_slice(&(transaction_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(transaction_bytes);
        } else {
            bytes.push(0);
        }
        bytes.extend_from_slice(&self.lineage.attempt().value().to_le_bytes());
        bytes.extend_from_slice(
            &self
                .lineage
                .transaction_id()
                .callback_index()
                .value()
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&self.producer_generation.value().to_le_bytes());
        bytes.extend_from_slice(&self.dispatch.value().to_le_bytes());
        bytes.extend_from_slice(&self.writer.value().to_le_bytes());
        bytes.extend_from_slice(&self.sequence_count.value().to_le_bytes());
        bytes.push(self.object_count.value());
        bytes.extend_from_slice(&self.byte_length.to_le_bytes());
        bytes.extend_from_slice(&self.body_sha256);
        bytes.extend_from_slice(&self.lineage.spatial_before_generation().to_le_bytes());
        bytes.extend_from_slice(&self.lineage.spatial_before_digest().bytes());
    }
}

fn digest_hex(digest: &[u8; 32]) -> String {
    let mut output = String::with_capacity(64);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Id14BodyProposal {
    body: MetadataPayloadBody,
    provenance: Id14ProducerProvenance,
}

impl Id14BodyProposal {
    pub(crate) fn from_successful_writer(
        body: MetadataPayloadBody,
        lineage: FitSourceLineage,
        producer_transaction_id: Option<CallbackTransactionId>,
        dispatch: JocDispatchIndex,
        sequence_count: u16,
        object_count: Id14ObjectCount,
    ) -> Option<Self> {
        let provenance = Id14ProducerProvenance::new(
            Id14ProducerIdentity {
                lineage,
                producer_transaction_id,
                producer_generation: body.writer_generation,
                dispatch,
                writer: JocGeneratedWriterIndex::new(body.writer_generation.value()),
                sequence_count: Id14SequenceCount::new(sequence_count)?,
                object_count,
            },
            &body.bytes,
        )?;
        Some(Self { body, provenance })
    }

    pub const fn body(&self) -> &MetadataPayloadBody {
        &self.body
    }

    pub const fn provenance(&self) -> &Id14ProducerProvenance {
        &self.provenance
    }

    pub(crate) fn into_parts(self) -> (MetadataPayloadBody, Id14ProducerProvenance) {
        (self.body, self.provenance)
    }
}
