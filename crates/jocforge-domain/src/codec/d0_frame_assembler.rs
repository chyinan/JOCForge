// pattern: Functional Core
//! Production-disabled caller-supplied Flat7x D0 frame bit assembly.

use super::{
    bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
    d0_frame_layout::{
        CallerOwnedUnboundD0Block2InsertionPoint, D0AudioBlockBitSpan, D0AudioBlockLayoutInput,
        D0Block2SkipFieldLayoutInput, D0FrameBitSpan, D0SixBlockLayoutError, D0SixBlockLayoutInput,
        Flat7xD0SixBlockLayout, propose_flat7x_d0_six_block_layout,
    },
    emdf_skip_field::{D0EmdfSkipFieldWriteError, write_flat7x_d0_block2_skipfld},
    flat7x_d0_frame_controls::{
        Flat7xD0AudioFrameControlError, Flat7xD0AudioFrameControlPolicy,
        Flat7xD0AudioFrameControlReceipt, write_flat7x_d0_audio_frame_controls,
    },
    frame_builder::{
        Flat7xD0BsiStructure, Flat7xD0BsiStructureError, write_flat7x_d0_bsi_structure,
    },
    owner::Flat7xD0SourceGenerationReceipt,
};
use crate::{
    analysis::source_frame::AnalysisOwnerGeneration,
    integrity::keyless_checksum::ProtectedEmdfContainer,
    joc::emdf::metadata_owner::MetadataSourceProposal,
    timeline::{
        CodecSourceIndex, FrontendCallbackIndex, JocDispatchIndex, JocGeneratedWriterIndex,
        PublicationDisposition, PublishedAuIndex, publication_for_source, source_for_callback,
    },
    transaction::{AttemptId, CallbackLineageIdentity, CallbackTransactionId, ProgrammeId},
};
use crate::{integrity::crc16::eac3_crc16, profile::Flat7xI0D0TargetPlan};

pub const D0_TARGET_FRAME_BYTES: usize = 4_096;
pub const D0_TARGET_FRAME_BITS: u32 = 32_768;
pub const D0_CRC2_FIELD_BITS: u32 = 16;
const D0_FRAME_TAIL_TERMINAL_BITS: u32 = 2;
const D0_CRC2_FIELD_BYTES: usize = 2;
const D0_CRC2_COVERAGE_START_BYTE: usize = 2;
const D0_BLOCK_TWO_INDEX: usize = 2;
const D0_AUDIO_BLOCK_COUNT: usize = 6;

/// A bit-range view over caller-provided opaque coded-block bytes.
///
/// The assembler copies these bits verbatim and does not decode or transform them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpaqueD0AudioBitSpan<'a> {
    bytes: &'a [u8],
    source_start_bit: u32,
    bit_length: u32,
}

impl<'a> OpaqueD0AudioBitSpan<'a> {
    pub const fn new(bytes: &'a [u8], source_start_bit: u32, bit_length: u32) -> Self {
        Self {
            bytes,
            source_start_bit,
            bit_length,
        }
    }

    pub const fn source_start_bit(self) -> u32 {
        self.source_start_bit
    }

    pub const fn bit_length(self) -> u32 {
        self.bit_length
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct D0Block2F13SourceLineage {
    consumer_transaction_id: CallbackTransactionId,
    callback_lineage: CallbackLineageIdentity,
    source_index: CodecSourceIndex,
    dispatch: JocDispatchIndex,
    au_index: PublishedAuIndex,
    metadata_owner_before_generation: u64,
    metadata_owner_after_generation: u64,
    id14_producer_transaction_id: CallbackTransactionId,
    id14_producer_generation: CodecSourceIndex,
    id14_writer_generation: JocGeneratedWriterIndex,
}

impl D0Block2F13SourceLineage {
    /// Extracts the exact callback, owner-generation, and ID14 producer receipt from F13.
    pub fn from_metadata_source(
        metadata_source: &MetadataSourceProposal,
    ) -> Result<Self, D0Block2SourceReceiptError> {
        let receipt = metadata_source.receipt();
        let consumer_transaction_id = metadata_source
            .consumer_transaction_id()
            .cloned()
            .ok_or(D0Block2SourceReceiptError::F13SourceTransactionMissing)?;
        let id14_queue_pop = receipt
            .id14_queue_pop()
            .ok_or(D0Block2SourceReceiptError::Id14QueueReceiptMissing)?;
        if id14_queue_pop.consumer_transaction_id() != &consumer_transaction_id
            || id14_queue_pop.consumer_source_index() != receipt.source_index()
            || id14_queue_pop.consumer_dispatch() != receipt.dispatch()
            || id14_queue_pop.consumer_au_index() != receipt.au_index()
        {
            return Err(D0Block2SourceReceiptError::Id14ConsumerIdentityMismatch);
        }
        let id14_provenance = id14_queue_pop.producer();
        let id14_producer_transaction_id = id14_provenance
            .producer_transaction_id()
            .cloned()
            .ok_or(D0Block2SourceReceiptError::Id14ProducerTransactionMissing)?;
        let au_index = receipt
            .au_index()
            .ok_or(D0Block2SourceReceiptError::AccessUnitMismatch)?;
        Ok(Self {
            consumer_transaction_id,
            callback_lineage: receipt.transaction_id().clone(),
            source_index: receipt.source_index(),
            dispatch: receipt.dispatch(),
            au_index,
            metadata_owner_before_generation: receipt.before_generation(),
            metadata_owner_after_generation: receipt.proposed_after_generation(),
            id14_producer_transaction_id,
            id14_producer_generation: id14_provenance.producer_generation(),
            id14_writer_generation: id14_provenance.writer(),
        })
    }

    pub const fn metadata_owner_before_generation(&self) -> u64 {
        self.metadata_owner_before_generation
    }

    pub fn consumer_transaction_id(&self) -> &CallbackTransactionId {
        &self.consumer_transaction_id
    }

    pub fn id14_producer_transaction_id(&self) -> &CallbackTransactionId {
        &self.id14_producer_transaction_id
    }

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn dispatch(&self) -> JocDispatchIndex {
        self.dispatch
    }

    pub const fn au_index(&self) -> PublishedAuIndex {
        self.au_index
    }

    pub const fn metadata_owner_after_generation(&self) -> u64 {
        self.metadata_owner_after_generation
    }

    pub const fn id14_producer_generation(&self) -> CodecSourceIndex {
        self.id14_producer_generation
    }

    pub const fn id14_writer_generation(&self) -> JocGeneratedWriterIndex {
        self.id14_writer_generation
    }

    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub fn with_owner_generations_for_test(&self, before: u64, after: u64) -> Self {
        let mut changed = self.clone();
        changed.metadata_owner_before_generation = before;
        changed.metadata_owner_after_generation = after;
        changed
    }

    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub fn with_id14_generations_for_test(
        &self,
        producer: CodecSourceIndex,
        writer: JocGeneratedWriterIndex,
    ) -> Self {
        let mut changed = self.clone();
        changed.id14_producer_generation = producer;
        changed.id14_writer_generation = writer;
        changed
    }

    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub fn with_id14_producer_generation_for_test(&self, producer: CodecSourceIndex) -> Self {
        let mut changed = self.clone();
        changed.id14_producer_generation = producer;
        changed
    }

    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub fn with_id14_writer_generation_for_test(&self, writer: JocGeneratedWriterIndex) -> Self {
        let mut changed = self.clone();
        changed.id14_writer_generation = writer;
        changed
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct D0Block2F14kSourceLineage {
    source_index: CodecSourceIndex,
    dispatch: JocDispatchIndex,
    au_index: PublishedAuIndex,
}

/// Untrusted, typed inputs from the F14K candidate; the assembler receipt constructor validates
/// each field against the admitted F13 proposal and Codec-owner-issued generation receipt.
#[derive(Clone, Copy, Debug)]
pub struct D0Block2F14kSourceReceiptInput<'a> {
    pub f13_lineage: &'a D0Block2F13SourceLineage,
    pub source_index: CodecSourceIndex,
    pub dispatch: JocDispatchIndex,
    pub au_index: PublishedAuIndex,
    pub expected_source_generation_receipt: &'a Flat7xD0SourceGenerationReceipt,
    pub f14k_source_generation_receipt: &'a Flat7xD0SourceGenerationReceipt,
    pub protected_container: &'a ProtectedEmdfContainer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum D0Block2SourceReceiptError {
    SourceClockMismatch,
    AccessUnitMismatch,
    F13F14kSourceMismatch,
    F13F14kDispatchMismatch,
    F13F14kAccessUnitMismatch,
    F13F14kReceiptMismatch,
    F13SourceTransactionMissing,
    Id14QueueReceiptMissing,
    Id14ConsumerIdentityMismatch,
    Id14ProducerTransactionMissing,
    SourceGenerationMismatch,
    ProtectedContainerMissing,
}

/// Source, owner-generation, and protected-container evidence required before block-two assembly.
///
/// Its Codec-owner-issued D0 source generation is distinct from the later F15
/// `Flat7xSourceGeneration` label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct D0Block2SourceReceipt<'a> {
    f13: D0Block2F13SourceLineage,
    f14k: D0Block2F14kSourceLineage,
    source_generation_receipt: Flat7xD0SourceGenerationReceipt,
    complete_emdf_container: &'a [u8],
}

impl<'a> D0Block2SourceReceipt<'a> {
    /// Binds an F13 proposal to the exact F13 receipt copied through F14K and the protected
    /// container produced from that same source.
    pub fn from_f13_f14k_source_receipts(
        f13_source: &MetadataSourceProposal,
        f14k: D0Block2F14kSourceReceiptInput<'a>,
    ) -> Result<Self, D0Block2SourceReceiptError> {
        let f13 = D0Block2F13SourceLineage::from_metadata_source(f13_source)?;
        if &f13 != f14k.f13_lineage {
            return Err(D0Block2SourceReceiptError::F13F14kReceiptMismatch);
        }
        Self::new(
            f13,
            D0Block2F14kSourceLineage {
                source_index: f14k.source_index,
                dispatch: f14k.dispatch,
                au_index: f14k.au_index,
            },
            f14k.expected_source_generation_receipt,
            f14k.f14k_source_generation_receipt,
            f14k.protected_container.bytes(),
        )
    }

    fn new(
        f13: D0Block2F13SourceLineage,
        f14k: D0Block2F14kSourceLineage,
        expected_source_generation_receipt: &Flat7xD0SourceGenerationReceipt,
        f14k_source_generation_receipt: &Flat7xD0SourceGenerationReceipt,
        complete_emdf_container: &'a [u8],
    ) -> Result<Self, D0Block2SourceReceiptError> {
        if f13.source_index.dispatch().ok() != Some(f13.dispatch)
            || source_for_callback(f13.callback_lineage.callback_index()) != Some(f13.source_index)
        {
            return Err(D0Block2SourceReceiptError::SourceClockMismatch);
        }
        let expected_au = match publication_for_source(f13.source_index) {
            Ok(PublicationDisposition::Published(consumption)) => consumption.au,
            _ => return Err(D0Block2SourceReceiptError::AccessUnitMismatch),
        };
        if expected_au != f13.au_index {
            return Err(D0Block2SourceReceiptError::AccessUnitMismatch);
        }
        if f13.source_index != f14k.source_index {
            return Err(D0Block2SourceReceiptError::F13F14kSourceMismatch);
        }
        if f13.dispatch != f14k.dispatch {
            return Err(D0Block2SourceReceiptError::F13F14kDispatchMismatch);
        }
        if f13.au_index != f14k.au_index {
            return Err(D0Block2SourceReceiptError::F13F14kAccessUnitMismatch);
        }
        if expected_source_generation_receipt != f14k_source_generation_receipt
            || f14k_source_generation_receipt.source_transaction_id()
                != &f13.consumer_transaction_id
            || f14k_source_generation_receipt.callback_lineage() != &f13.callback_lineage
            || f14k_source_generation_receipt.source_index() != f13.source_index
        {
            return Err(D0Block2SourceReceiptError::SourceGenerationMismatch);
        }
        if complete_emdf_container.is_empty() {
            return Err(D0Block2SourceReceiptError::ProtectedContainerMissing);
        }

        Ok(Self {
            f13,
            f14k,
            source_generation_receipt: f14k_source_generation_receipt.clone(),
            complete_emdf_container,
        })
    }

    pub const fn source_generation_receipt(&self) -> &Flat7xD0SourceGenerationReceipt {
        &self.source_generation_receipt
    }

    pub const fn au_index(&self) -> PublishedAuIndex {
        self.f13.au_index
    }

    pub const fn complete_emdf_container(&self) -> &'a [u8] {
        self.complete_emdf_container
    }

    pub fn matches_source_identity(
        &self,
        programme_id: &ProgrammeId,
        attempt_id: AttemptId,
        callback_index: FrontendCallbackIndex,
        source_transaction_id: &CallbackTransactionId,
        source_index: CodecSourceIndex,
        analysis_generation: AnalysisOwnerGeneration,
    ) -> bool {
        let lineage = &self.f13.callback_lineage;
        lineage.programme_id() == programme_id
            && lineage.attempt_id() == attempt_id
            && lineage.callback_index() == callback_index
            && &self.f13.consumer_transaction_id == source_transaction_id
            && self.f13.source_index == source_index
            && self.f14k.source_index == source_index
            && self.source_generation_receipt.source_transaction_id() == source_transaction_id
            && self.source_generation_receipt.callback_lineage() == lineage
            && self.source_generation_receipt.source_index() == source_index
            && self.source_generation_receipt.analysis_generation() == analysis_generation
    }

    #[cfg(any(test, feature = "test-support"))]
    /// Creates deterministic lineage for syntax-only assembler tests.
    ///
    /// # Panics
    ///
    /// Panics only if the fixed test identifiers become invalid under their constructors.
    pub fn synthetic_for_test(
        container: &'a [u8],
        producer_generation: u64,
        writer_generation: u64,
    ) -> Result<Self, D0Block2SourceReceiptError> {
        let source_index = CodecSourceIndex::new(2);
        let attempt = AttemptId::new(1);
        let callback_index = FrontendCallbackIndex::new(4);
        let transaction_id = CallbackTransactionId::new("synthetic-source-s2")
            .expect("test transaction id is valid");
        let f13 = D0Block2F13SourceLineage {
            consumer_transaction_id: transaction_id.clone(),
            callback_lineage: CallbackLineageIdentity::new(
                ProgrammeId::new("synthetic-programme").expect("test programme id is valid"),
                attempt,
                callback_index,
            ),
            source_index,
            dispatch: JocDispatchIndex::new(3),
            au_index: PublishedAuIndex::new(0),
            metadata_owner_before_generation: 0,
            metadata_owner_after_generation: 1,
            id14_producer_transaction_id: transaction_id,
            id14_producer_generation: CodecSourceIndex::new(producer_generation),
            id14_writer_generation: JocGeneratedWriterIndex::new(writer_generation),
        };
        Self::new(
            f13,
            D0Block2F14kSourceLineage {
                source_index,
                dispatch: JocDispatchIndex::new(3),
                au_index: PublishedAuIndex::new(0),
            },
            &Flat7xD0SourceGenerationReceipt::synthetic_for_test(
                CallbackTransactionId::new("synthetic-source-s2")
                    .expect("test transaction id is valid"),
                1,
                CallbackLineageIdentity::new(
                    ProgrammeId::new("synthetic-programme").expect("test programme id is valid"),
                    attempt,
                    callback_index,
                ),
                source_index,
                0,
            ),
            &Flat7xD0SourceGenerationReceipt::synthetic_for_test(
                CallbackTransactionId::new("synthetic-source-s2")
                    .expect("test transaction id is valid"),
                1,
                CallbackLineageIdentity::new(
                    ProgrammeId::new("synthetic-programme").expect("test programme id is valid"),
                    attempt,
                    callback_index,
                ),
                source_index,
                0,
            ),
            container,
        )
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn synthetic_for_source_for_test(
        expected_source_generation_receipt: &Flat7xD0SourceGenerationReceipt,
        f14k_source_generation_receipt: &Flat7xD0SourceGenerationReceipt,
        metadata_owner_generations: (u64, u64),
        id14_generations: (CodecSourceIndex, JocGeneratedWriterIndex),
        container: &'a [u8],
    ) -> Result<Self, D0Block2SourceReceiptError> {
        Self::synthetic_with_source_receipts_for_test(
            expected_source_generation_receipt,
            f14k_source_generation_receipt,
            metadata_owner_generations,
            id14_generations,
            container,
        )
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn synthetic_with_source_receipts_for_test(
        expected_source_generation_receipt: &Flat7xD0SourceGenerationReceipt,
        f14k_source_generation_receipt: &Flat7xD0SourceGenerationReceipt,
        metadata_owner_generations: (u64, u64),
        id14_generations: (CodecSourceIndex, JocGeneratedWriterIndex),
        container: &'a [u8],
    ) -> Result<Self, D0Block2SourceReceiptError> {
        let source_index = expected_source_generation_receipt.source_index();
        let dispatch = source_index
            .dispatch()
            .map_err(|_| D0Block2SourceReceiptError::SourceClockMismatch)?;
        let au_index = match publication_for_source(source_index) {
            Ok(PublicationDisposition::Published(consumption)) => consumption.au,
            _ => return Err(D0Block2SourceReceiptError::AccessUnitMismatch),
        };
        let f13 = D0Block2F13SourceLineage {
            consumer_transaction_id: expected_source_generation_receipt
                .source_transaction_id()
                .clone(),
            callback_lineage: expected_source_generation_receipt
                .callback_lineage()
                .clone(),
            source_index,
            dispatch,
            au_index,
            metadata_owner_before_generation: metadata_owner_generations.0,
            metadata_owner_after_generation: metadata_owner_generations.1,
            id14_producer_transaction_id: expected_source_generation_receipt
                .source_transaction_id()
                .clone(),
            id14_producer_generation: id14_generations.0,
            id14_writer_generation: id14_generations.1,
        };
        Self::new(
            f13,
            D0Block2F14kSourceLineage {
                source_index,
                dispatch,
                au_index,
            },
            expected_source_generation_receipt,
            f14k_source_generation_receipt,
            container,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct D0Block2SkipFieldInput<'a> {
    pub insertion_point: CallerOwnedUnboundD0Block2InsertionPoint,
    pub source_receipt: D0Block2SourceReceipt<'a>,
}

impl<'a> D0Block2SkipFieldInput<'a> {
    #[cfg(any(test, feature = "test-support"))]
    pub fn synthetic_for_test(
        insertion_point: CallerOwnedUnboundD0Block2InsertionPoint,
        container: &'a [u8],
    ) -> Result<Self, D0Block2SourceReceiptError> {
        Ok(Self {
            insertion_point,
            source_receipt: D0Block2SourceReceipt::synthetic_for_test(container, 2, 2)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D0BlockTwoAudioSpans<'a> {
    pub before_skipfield: OpaqueD0AudioBitSpan<'a>,
    pub skip_fields: &'a [D0Block2SkipFieldInput<'a>],
    pub after_skipfield: OpaqueD0AudioBitSpan<'a>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum D0FrameAudioBlockInput<'a> {
    Opaque(OpaqueD0AudioBitSpan<'a>),
    BlockTwo(D0BlockTwoAudioSpans<'a>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0FrameBitInput<'a> {
    pub audio_frame_controls: Flat7xD0AudioFrameControlPolicy,
    pub audio_blocks: [D0FrameAudioBlockInput<'a>; D0_AUDIO_BLOCK_COUNT],
}

/// Opaque caller-owned frame-level aux-data syntax located after all six `audblk`s and before CRC2.
///
/// The span includes the caller's `auxdatae`/`auxbits` representation. This type assigns no
/// default value and does not parse the frame-level tail.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpaqueD0FrameAuxDataSpan<'a> {
    bytes: &'a [u8],
    source_start_bit: u32,
    bit_length: u32,
}

impl<'a> OpaqueD0FrameAuxDataSpan<'a> {
    pub const fn new(bytes: &'a [u8], source_start_bit: u32, bit_length: u32) -> Self {
        Self {
            bytes,
            source_start_bit,
            bit_length,
        }
    }

    pub const fn source_start_bit(self) -> u32 {
        self.source_start_bit
    }

    pub const fn bit_length(self) -> u32 {
        self.bit_length
    }
}

/// The selected D0 frame-tail policy for unused frame-level aux-bit capacity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0FrameTailPolicy {
    /// Zero-fill nauxbits, then emit `auxdatae=0` and `crcrsv=0` before CRC2.
    ZeroFillUnusedAuxBits,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D0Crc2BitSpan {
    start_bit: u32,
}

impl D0Crc2BitSpan {
    pub const fn start_bit(self) -> u32 {
        self.start_bit
    }

    pub const fn bit_length(self) -> u32 {
        D0_CRC2_FIELD_BITS
    }

    pub const fn end_bit(self) -> u32 {
        self.start_bit + D0_CRC2_FIELD_BITS
    }
}

/// Generated E-AC-3 CRC2 and its completed-frame residual check.
///
/// Coverage follows the approved keyless frame contract: generate over
/// `frame[2..4094]`, store big-endian at `frame[4094..4096]`, then verify
/// `frame[2..4096]` with a zero residual.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D0Crc2IntegrityReceipt {
    generated_value: u16,
    verified_residual: u16,
}

impl D0Crc2IntegrityReceipt {
    pub const fn generated_value(self) -> u16 {
        self.generated_value
    }

    pub const fn verified_residual(self) -> u16 {
        self.verified_residual
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0FrameAssemblyReceipt {
    layout: Flat7xD0SixBlockLayout,
    audio_frame_controls: Flat7xD0AudioFrameControlReceipt,
    frame_auxbits_span: D0FrameBitSpan,
    frame_tail_policy: Option<Flat7xD0FrameTailPolicy>,
    zero_filled_auxbits: u32,
    crc2_span: D0Crc2BitSpan,
    crc2_integrity: D0Crc2IntegrityReceipt,
}

impl Flat7xD0FrameAssemblyReceipt {
    pub const fn layout(self) -> Flat7xD0SixBlockLayout {
        self.layout
    }

    pub const fn audio_frame_controls(self) -> Flat7xD0AudioFrameControlReceipt {
        self.audio_frame_controls
    }

    pub const fn frame_auxbits_span(self) -> D0FrameBitSpan {
        self.frame_auxbits_span
    }

    pub const fn frame_tail_policy(self) -> Option<Flat7xD0FrameTailPolicy> {
        self.frame_tail_policy
    }

    pub const fn auxdatae_value(self) -> Option<bool> {
        match self.frame_tail_policy {
            Some(Flat7xD0FrameTailPolicy::ZeroFillUnusedAuxBits) => Some(false),
            None => None,
        }
    }

    pub const fn crcrsv_value(self) -> Option<bool> {
        match self.frame_tail_policy {
            Some(Flat7xD0FrameTailPolicy::ZeroFillUnusedAuxBits) => Some(false),
            None => None,
        }
    }

    pub const fn zero_filled_auxbits(self) -> u32 {
        self.zero_filled_auxbits
    }

    pub const fn crc2_span(self) -> D0Crc2BitSpan {
        self.crc2_span
    }

    pub const fn crc2_integrity(self) -> D0Crc2IntegrityReceipt {
        self.crc2_integrity
    }

    pub const fn total_frame_span_bits(self) -> u32 {
        D0_TARGET_FRAME_BITS
    }

    pub const fn target_frame_bytes(self) -> u16 {
        D0_TARGET_FRAME_BYTES as u16
    }

    pub const fn production_allowed(self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0FrameAssemblyError {
    CursorMismatch {
        actual: u32,
    },
    BlockTwoMissing,
    BlockTwoAtWrongIndex {
        actual: u8,
    },
    DuplicateBlockTwoSkipFields,
    BlockTwoInsertionPointMismatch {
        declared_bits: u32,
        pre_span_bits: u32,
    },
    SourceSpanOverflow {
        block_index: u8,
    },
    SourceSpanOutOfBounds {
        block_index: u8,
        required_end_bit: u32,
        available_source_bits: u64,
    },
    FrameAuxDataSourceSpanOverflow,
    FrameAuxDataSourceSpanOutOfBounds {
        required_end_bit: u32,
        available_source_bits: u64,
    },
    FrameTailNoSpace {
        available_bits: u32,
    },
    FrameSpanMismatch {
        actual_bits: u32,
        required_bits: u32,
    },
    Crc2FieldNotByteAligned {
        start_bit: u32,
    },
    Crc2Residual {
        residual: u16,
    },
    Layout(D0SixBlockLayoutError),
    BsiStructure(Flat7xD0BsiStructureError),
    AudioFrameControls(Flat7xD0AudioFrameControlError),
    SkipField(D0EmdfSkipFieldWriteError),
    BitWriter(FrameBitWriterError),
}

impl From<D0SixBlockLayoutError> for Flat7xD0FrameAssemblyError {
    fn from(error: D0SixBlockLayoutError) -> Self {
        Self::Layout(error)
    }
}

impl From<Flat7xD0BsiStructureError> for Flat7xD0FrameAssemblyError {
    fn from(error: Flat7xD0BsiStructureError) -> Self {
        Self::BsiStructure(error)
    }
}

impl From<Flat7xD0AudioFrameControlError> for Flat7xD0FrameAssemblyError {
    fn from(error: Flat7xD0AudioFrameControlError) -> Self {
        Self::AudioFrameControls(error)
    }
}

impl From<D0EmdfSkipFieldWriteError> for Flat7xD0FrameAssemblyError {
    fn from(error: D0EmdfSkipFieldWriteError) -> Self {
        Self::SkipField(error)
    }
}

impl From<FrameBitWriterError> for Flat7xD0FrameAssemblyError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Assembles one caller-supplied D0 bit frame into a production-disabled scratch writer.
///
/// The six block inputs are ordered by array position. Only block two may use the split-span
/// input with one complete skip-field container and an explicit caller-owned/unbound insertion
/// point. Audio spans are copied bit-for-bit as opaque data. After the complete 4096-byte frame
/// body is staged, the approved E-AC-3 CRC16 rule generates CRC2 and checks the completed-frame
/// residual. All shapes, source extents, skip-field length, exact 32768-bit frame extent, CRC
/// integrity, and writer capacity are checked before caller writer bytes or cursor change.
pub fn assemble_flat7x_d0_frame_bits(
    target: Flat7xI0D0TargetPlan,
    bsi_structure: Flat7xD0BsiStructure,
    input: Flat7xD0FrameBitInput<'_>,
    writer: &mut FrameBitWriter<'_>,
) -> Result<Flat7xD0FrameAssemblyReceipt, Flat7xD0FrameAssemblyError> {
    assemble_flat7x_d0_frame_bits_inner(target, bsi_structure, &input, None, None, writer)
}

/// Assembles the selected D0 frame with a typed, zero-filled frame-level aux-data tail.
///
/// The writer computes nauxbits after accounting for all six audio blocks and the block-two
/// skipfld, zero-fills nauxbits, then writes `auxdatae=0` and `crcrsv=0` before CRC2.
pub fn assemble_flat7x_d0_frame_bits_with_tail_policy(
    target: Flat7xI0D0TargetPlan,
    bsi_structure: Flat7xD0BsiStructure,
    input: Flat7xD0FrameBitInput<'_>,
    tail_policy: Flat7xD0FrameTailPolicy,
    writer: &mut FrameBitWriter<'_>,
) -> Result<Flat7xD0FrameAssemblyReceipt, Flat7xD0FrameAssemblyError> {
    assemble_flat7x_d0_frame_bits_inner(
        target,
        bsi_structure,
        &input,
        None,
        Some(tail_policy),
        writer,
    )
}

/// Assembles a D0 frame with a caller-owned frame-level aux-data tail.
///
/// The complete opaque tail is placed after the six ordered audio blocks and before generated
/// CRC2. The caller supplies all encoded bits, including any presence/padding fields.
#[cfg(feature = "test-support")]
pub fn assemble_flat7x_d0_frame_bits_with_aux_data(
    target: Flat7xI0D0TargetPlan,
    bsi_structure: Flat7xD0BsiStructure,
    input: Flat7xD0FrameBitInput<'_>,
    frame_aux_data: OpaqueD0FrameAuxDataSpan<'_>,
    writer: &mut FrameBitWriter<'_>,
) -> Result<Flat7xD0FrameAssemblyReceipt, Flat7xD0FrameAssemblyError> {
    assemble_flat7x_d0_frame_bits_inner(
        target,
        bsi_structure,
        &input,
        Some(frame_aux_data),
        None,
        writer,
    )
}

fn assemble_flat7x_d0_frame_bits_inner(
    target: Flat7xI0D0TargetPlan,
    bsi_structure: Flat7xD0BsiStructure,
    input: &Flat7xD0FrameBitInput<'_>,
    frame_aux_data: Option<OpaqueD0FrameAuxDataSpan<'_>>,
    frame_tail_policy: Option<Flat7xD0FrameTailPolicy>,
    writer: &mut FrameBitWriter<'_>,
) -> Result<Flat7xD0FrameAssemblyReceipt, Flat7xD0FrameAssemblyError> {
    let actual_cursor = writer.cursor().bit_offset();
    if actual_cursor != 0 {
        return Err(Flat7xD0FrameAssemblyError::CursorMismatch {
            actual: actual_cursor,
        });
    }

    let mut block_lengths = [0_u32; D0_AUDIO_BLOCK_COUNT];
    let mut block_two_input = None;
    for (index, block) in input.audio_blocks.iter().copied().enumerate() {
        let block_index =
            u8::try_from(index).map_err(|_| Flat7xD0FrameAssemblyError::SourceSpanOverflow {
                block_index: u8::MAX,
            })?;
        match (index, block) {
            (D0_BLOCK_TWO_INDEX, D0FrameAudioBlockInput::BlockTwo(block_two)) => {
                if block_two.skip_fields.is_empty() {
                    return Err(Flat7xD0FrameAssemblyError::BlockTwoMissing);
                }
                if block_two.skip_fields.len() != 1 {
                    return Err(Flat7xD0FrameAssemblyError::DuplicateBlockTwoSkipFields);
                }
                let declared = block_two.skip_fields[0]
                    .insertion_point
                    .offset_within_audio_span_bits();
                let pre_length = block_two.before_skipfield.bit_length();
                if declared != pre_length {
                    return Err(Flat7xD0FrameAssemblyError::BlockTwoInsertionPointMismatch {
                        declared_bits: declared,
                        pre_span_bits: pre_length,
                    });
                }
                validate_source_span(block_index, block_two.before_skipfield)?;
                validate_source_span(block_index, block_two.after_skipfield)?;
                block_lengths[index] = pre_length
                    .checked_add(block_two.after_skipfield.bit_length())
                    .ok_or(Flat7xD0FrameAssemblyError::SourceSpanOverflow { block_index })?;
                block_two_input = Some(block_two);
            }
            (D0_BLOCK_TWO_INDEX, D0FrameAudioBlockInput::Opaque(_)) => {
                return Err(Flat7xD0FrameAssemblyError::BlockTwoMissing);
            }
            (_, D0FrameAudioBlockInput::BlockTwo(_)) => {
                return Err(Flat7xD0FrameAssemblyError::BlockTwoAtWrongIndex {
                    actual: block_index,
                });
            }
            (index, D0FrameAudioBlockInput::Opaque(span)) => {
                validate_source_span(block_index, span)?;
                block_lengths[index] = span.bit_length();
            }
        }
    }
    let block_two_input = block_two_input.ok_or(Flat7xD0FrameAssemblyError::BlockTwoMissing)?;
    let container_bytes = u16::try_from(
        block_two_input.skip_fields[0]
            .source_receipt
            .complete_emdf_container()
            .len(),
    )
    .map_err(|_| {
        Flat7xD0FrameAssemblyError::SkipField(D0EmdfSkipFieldWriteError::ContainerTooLarge)
    })?;
    let block_two_layout_field = [D0Block2SkipFieldLayoutInput {
        insertion_point: block_two_input.skip_fields[0].insertion_point,
        payload_bytes: container_bytes,
    }];
    let no_skip_fields: &[D0Block2SkipFieldLayoutInput] = &[];
    let layout_input = D0SixBlockLayoutInput {
        blocks: std::array::from_fn(|index| D0AudioBlockLayoutInput {
            audio_span: D0AudioBlockBitSpan::new(block_lengths[index]),
            skip_fields: if index == D0_BLOCK_TWO_INDEX {
                &block_two_layout_field
            } else {
                no_skip_fields
            },
        }),
    };
    let layout = propose_flat7x_d0_six_block_layout(target, bsi_structure, layout_input)?;
    if let Some(frame_aux_data) = frame_aux_data {
        validate_frame_aux_data_source_span(frame_aux_data)?;
    }
    let frame_bytes = target.d0().frame_bytes();
    let required_frame_bits = u32::from(frame_bytes).checked_mul(8).ok_or(
        Flat7xD0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: layout.total_frame_span_bits(),
            required_bits: D0_TARGET_FRAME_BITS,
        },
    )?;
    let required_crc2_start_bit = required_frame_bits.checked_sub(D0_CRC2_FIELD_BITS).ok_or(
        Flat7xD0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: required_frame_bits,
            required_bits: D0_CRC2_FIELD_BITS,
        },
    )?;
    let (frame_aux_data_bits, crc2_start_bit, zero_filled_auxbits) =
        if let Some(Flat7xD0FrameTailPolicy::ZeroFillUnusedAuxBits) = frame_tail_policy {
            let available_bits = required_crc2_start_bit
                .checked_sub(layout.total_frame_span_bits())
                .ok_or(Flat7xD0FrameAssemblyError::FrameTailNoSpace { available_bits: 0 })?;
            if available_bits < D0_FRAME_TAIL_TERMINAL_BITS {
                return Err(Flat7xD0FrameAssemblyError::FrameTailNoSpace { available_bits });
            }
            let nauxbits = available_bits - D0_FRAME_TAIL_TERMINAL_BITS;
            (nauxbits, required_crc2_start_bit, nauxbits)
        } else {
            let frame_aux_data_bits =
                frame_aux_data.map_or(0, OpaqueD0FrameAuxDataSpan::bit_length);
            let crc2_start_bit = layout
                .total_frame_span_bits()
                .checked_add(frame_aux_data_bits)
                .ok_or(Flat7xD0FrameAssemblyError::FrameSpanMismatch {
                    actual_bits: layout.total_frame_span_bits(),
                    required_bits: required_frame_bits,
                })?;
            (frame_aux_data_bits, crc2_start_bit, 0)
        };
    let full_span_bits = crc2_start_bit.checked_add(D0_CRC2_FIELD_BITS).ok_or(
        Flat7xD0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: crc2_start_bit,
            required_bits: required_frame_bits,
        },
    )?;
    if required_frame_bits != D0_TARGET_FRAME_BITS || full_span_bits != required_frame_bits {
        return Err(Flat7xD0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: full_span_bits,
            required_bits: D0_TARGET_FRAME_BITS,
        });
    }
    writer.preflight_capacity(required_frame_bits)?;

    let mut staging = [0_u8; D0_TARGET_FRAME_BYTES];
    let bsi_end = {
        let mut staging_writer = FrameBitWriter::new(&mut staging, FrameBitCursor::new(0))?;
        write_flat7x_d0_bsi_structure(&mut staging_writer, target, bsi_structure)?.bit_offset()
    };
    if bsi_end != layout.bsi_span().end_bit() {
        return Err(Flat7xD0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: bsi_end,
            required_bits: layout.bsi_span().end_bit(),
        });
    }
    let audio_frame_control_span = layout.audio_frame_control_span();
    let audio_frame_controls = {
        let mut controls_writer = FrameBitWriter::new(
            &mut staging,
            FrameBitCursor::new(audio_frame_control_span.start_bit()),
        )?;
        write_flat7x_d0_audio_frame_controls(&mut controls_writer, input.audio_frame_controls)?
    };
    if audio_frame_controls.start_bit() != audio_frame_control_span.start_bit()
        || audio_frame_controls.end_bit() != audio_frame_control_span.end_bit()
    {
        return Err(Flat7xD0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: audio_frame_controls.end_bit(),
            required_bits: audio_frame_control_span.end_bit(),
        });
    }

    let block_layouts = layout.audio_blocks();
    for (index, block) in input.audio_blocks.iter().copied().enumerate() {
        let block_index =
            u8::try_from(index).map_err(|_| Flat7xD0FrameAssemblyError::SourceSpanOverflow {
                block_index: u8::MAX,
            })?;
        match (index, block) {
            (D0_BLOCK_TWO_INDEX, D0FrameAudioBlockInput::BlockTwo(block_two)) => {
                let block_layout = block_layouts[index];
                copy_opaque_audio_span(
                    &mut staging,
                    block_index,
                    block_layout.leading_audio_span(),
                    block_two.before_skipfield,
                )?;
                let skip_span = layout.block_two_skip_field().bit_span();
                let skipfield_end = {
                    let mut field_writer = FrameBitWriter::new(
                        &mut staging,
                        FrameBitCursor::new(skip_span.start_bit()),
                    )?;
                    write_flat7x_d0_block2_skipfld(
                        &mut field_writer,
                        block_two.skip_fields[0]
                            .source_receipt
                            .complete_emdf_container(),
                    )?
                    .bit_offset()
                };
                if skipfield_end != skip_span.end_bit() {
                    return Err(Flat7xD0FrameAssemblyError::FrameSpanMismatch {
                        actual_bits: skipfield_end,
                        required_bits: skip_span.end_bit(),
                    });
                }
                let trailing = block_layout
                    .trailing_audio_span()
                    .ok_or(Flat7xD0FrameAssemblyError::BlockTwoMissing)?;
                copy_opaque_audio_span(
                    &mut staging,
                    block_index,
                    trailing,
                    block_two.after_skipfield,
                )?;
            }
            (index, D0FrameAudioBlockInput::Opaque(span)) => {
                copy_opaque_audio_span(
                    &mut staging,
                    block_index,
                    block_layouts[index].leading_audio_span(),
                    span,
                )?;
            }
            (index, D0FrameAudioBlockInput::BlockTwo(_)) => {
                return Err(Flat7xD0FrameAssemblyError::BlockTwoAtWrongIndex {
                    actual: u8::try_from(index).map_err(|_| {
                        Flat7xD0FrameAssemblyError::SourceSpanOverflow {
                            block_index: u8::MAX,
                        }
                    })?,
                });
            }
        }
    }

    let frame_aux_data_span =
        D0FrameBitSpan::new(layout.total_frame_span_bits(), frame_aux_data_bits);
    if frame_tail_policy.is_some() {
        write_zero_filled_frame_tail(&mut staging, frame_aux_data_span)?;
        let mut tail_flag_writer = FrameBitWriter::new(
            &mut staging,
            FrameBitCursor::new(frame_aux_data_span.end_bit()),
        )?;
        tail_flag_writer.write_bit(false)?; // auxdatae=0; no trailing auxdatal U14.
        tail_flag_writer.write_bit(false)?; // crcrsv=0.
        if tail_flag_writer.cursor().bit_offset() != crc2_start_bit {
            return Err(Flat7xD0FrameAssemblyError::FrameSpanMismatch {
                actual_bits: tail_flag_writer.cursor().bit_offset(),
                required_bits: crc2_start_bit,
            });
        }
    } else if let Some(frame_aux_data) = frame_aux_data {
        copy_opaque_frame_aux_data(&mut staging, frame_aux_data_span, frame_aux_data)?;
    }

    if crc2_start_bit % 8 != 0 {
        return Err(Flat7xD0FrameAssemblyError::Crc2FieldNotByteAligned {
            start_bit: crc2_start_bit,
        });
    }
    let crc2_start_byte = usize::try_from(crc2_start_bit / 8).map_err(|_| {
        Flat7xD0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: crc2_start_bit,
            required_bits: required_frame_bits,
        }
    })?;
    let crc2_end_byte = crc2_start_byte.checked_add(D0_CRC2_FIELD_BYTES).ok_or(
        Flat7xD0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: crc2_start_bit,
            required_bits: required_frame_bits,
        },
    )?;
    if crc2_end_byte != staging.len() {
        return Err(Flat7xD0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: crc2_start_bit + D0_CRC2_FIELD_BITS,
            required_bits: required_frame_bits,
        });
    }

    let crc2_value = eac3_crc16(&staging[D0_CRC2_COVERAGE_START_BYTE..crc2_start_byte]);
    let crc2_end_bit = {
        let mut crc_writer =
            FrameBitWriter::new(&mut staging, FrameBitCursor::new(crc2_start_bit))?;
        crc_writer.write_unsigned(u32::from(crc2_value), D0_CRC2_FIELD_BITS as u8)?;
        crc_writer.cursor().bit_offset()
    };
    if crc2_end_bit != required_frame_bits {
        return Err(Flat7xD0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: crc2_end_bit,
            required_bits: required_frame_bits,
        });
    }
    let verified_residual = eac3_crc16(&staging[D0_CRC2_COVERAGE_START_BYTE..]);
    if verified_residual != 0 {
        return Err(Flat7xD0FrameAssemblyError::Crc2Residual {
            residual: verified_residual,
        });
    }

    writer.write_bytes(&staging)?;
    Ok(Flat7xD0FrameAssemblyReceipt {
        layout,
        audio_frame_controls,
        frame_auxbits_span: frame_aux_data_span,
        frame_tail_policy,
        zero_filled_auxbits,
        crc2_span: D0Crc2BitSpan {
            start_bit: crc2_start_bit,
        },
        crc2_integrity: D0Crc2IntegrityReceipt {
            generated_value: crc2_value,
            verified_residual,
        },
    })
}

fn write_zero_filled_frame_tail(
    output: &mut [u8],
    span: D0FrameBitSpan,
) -> Result<(), Flat7xD0FrameAssemblyError> {
    let mut writer = FrameBitWriter::new(output, FrameBitCursor::new(span.start_bit()))?;
    for _ in 0..span.bit_length() {
        writer.write_bit(false)?;
    }
    Ok(())
}

fn validate_source_span(
    block_index: u8,
    span: OpaqueD0AudioBitSpan<'_>,
) -> Result<(), Flat7xD0FrameAssemblyError> {
    let required_end_bit = span
        .source_start_bit()
        .checked_add(span.bit_length())
        .ok_or(Flat7xD0FrameAssemblyError::SourceSpanOverflow { block_index })?;
    let available_source_bits = u64::try_from(span.bytes.len())
        .unwrap_or(u64::MAX)
        .saturating_mul(8);
    if u64::from(required_end_bit) > available_source_bits {
        return Err(Flat7xD0FrameAssemblyError::SourceSpanOutOfBounds {
            block_index,
            required_end_bit,
            available_source_bits,
        });
    }
    Ok(())
}

fn validate_frame_aux_data_source_span(
    span: OpaqueD0FrameAuxDataSpan<'_>,
) -> Result<(), Flat7xD0FrameAssemblyError> {
    let required_end_bit = span
        .source_start_bit()
        .checked_add(span.bit_length())
        .ok_or(Flat7xD0FrameAssemblyError::FrameAuxDataSourceSpanOverflow)?;
    let available_source_bits = u64::try_from(span.bytes.len())
        .unwrap_or(u64::MAX)
        .saturating_mul(8);
    if u64::from(required_end_bit) > available_source_bits {
        return Err(
            Flat7xD0FrameAssemblyError::FrameAuxDataSourceSpanOutOfBounds {
                required_end_bit,
                available_source_bits,
            },
        );
    }
    Ok(())
}

fn copy_opaque_audio_span(
    output: &mut [u8],
    block_index: u8,
    destination: D0FrameBitSpan,
    source: OpaqueD0AudioBitSpan<'_>,
) -> Result<(), Flat7xD0FrameAssemblyError> {
    if destination.bit_length() != source.bit_length() {
        return Err(Flat7xD0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: source.bit_length(),
            required_bits: destination.bit_length(),
        });
    }
    validate_source_span(block_index, source)?;
    let mut writer = FrameBitWriter::new(output, FrameBitCursor::new(destination.start_bit()))?;
    writer.preflight_capacity(destination.bit_length())?;
    for offset in 0..source.bit_length() {
        let source_bit = source
            .source_start_bit()
            .checked_add(offset)
            .ok_or(Flat7xD0FrameAssemblyError::SourceSpanOverflow { block_index })?;
        let source_byte = usize::try_from(source_bit / 8)
            .map_err(|_| Flat7xD0FrameAssemblyError::SourceSpanOverflow { block_index })?;
        let shift = 7 - source_bit % 8;
        let value = source.bytes[source_byte] & (1_u8 << shift) != 0;
        writer.write_bit(value)?;
    }
    Ok(())
}

fn copy_opaque_frame_aux_data(
    output: &mut [u8],
    destination: D0FrameBitSpan,
    source: OpaqueD0FrameAuxDataSpan<'_>,
) -> Result<(), Flat7xD0FrameAssemblyError> {
    if destination.bit_length() != source.bit_length() {
        return Err(Flat7xD0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: source.bit_length(),
            required_bits: destination.bit_length(),
        });
    }
    validate_frame_aux_data_source_span(source)?;
    let mut writer = FrameBitWriter::new(output, FrameBitCursor::new(destination.start_bit()))?;
    writer.preflight_capacity(destination.bit_length())?;
    for offset in 0..source.bit_length() {
        let source_bit = source
            .source_start_bit()
            .checked_add(offset)
            .ok_or(Flat7xD0FrameAssemblyError::FrameAuxDataSourceSpanOverflow)?;
        let source_byte = usize::try_from(source_bit / 8)
            .map_err(|_| Flat7xD0FrameAssemblyError::FrameAuxDataSourceSpanOverflow)?;
        let shift = 7 - source_bit % 8;
        let value = source.bytes[source_byte] & (1_u8 << shift) != 0;
        writer.write_bit(value)?;
    }
    Ok(())
}
