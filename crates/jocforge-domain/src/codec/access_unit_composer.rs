// pattern: Functional Core
//! Production-disabled byte ordering for caller-supplied Flat7x I0/D0 frame spans.

use super::{
    bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
    d0_frame_assembler::{
        D0_TARGET_FRAME_BYTES, Flat7xD0FrameAssemblyError, Flat7xD0FrameAssemblyReceipt,
        Flat7xD0FrameBitInput, assemble_flat7x_d0_frame_bits,
    },
    frame_builder::Flat7xD0BsiStructure,
    i0_frame_assembler::Flat7xI0Ac3AudioSeriesFrameCandidate,
};
use crate::{analysis::source_frame::SharedAnalysisIdentity, profile::Flat7xI0D0TargetPlan};

const FLAT7X_I0_FRAME_BYTES: usize = 2_560;
const FLAT7X_D0_FRAME_BYTES: usize = 4_096;
pub const FLAT7X_AU_FRAME_BYTES: usize = 6_656;
const FLAT7X_AU_FRAME_BITS: u32 = 53_248;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpaqueI0FrameBytes<'a>(&'a [u8]);

impl<'a> OpaqueI0FrameBytes<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(self) -> &'a [u8] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpaqueD0FrameBytes<'a>(&'a [u8]);

impl<'a> OpaqueD0FrameBytes<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(self) -> &'a [u8] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xI0D0OpaqueFrameSpans<'a> {
    pub i0: OpaqueI0FrameBytes<'a>,
    pub d0: OpaqueD0FrameBytes<'a>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xAuByteSpan {
    start_byte: u16,
    byte_length: u16,
}

impl Flat7xAuByteSpan {
    pub const fn start_byte(self) -> u16 {
        self.start_byte
    }

    pub const fn byte_length(self) -> u16 {
        self.byte_length
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xI0D0AuCompositionReceipt {
    i0_span: Flat7xAuByteSpan,
    d0_span: Flat7xAuByteSpan,
}

impl Flat7xI0D0AuCompositionReceipt {
    pub const fn i0_span(self) -> Flat7xAuByteSpan {
        self.i0_span
    }

    pub const fn d0_span(self) -> Flat7xAuByteSpan {
        self.d0_span
    }

    pub const fn total_byte_length(self) -> u32 {
        FLAT7X_AU_FRAME_BYTES as u32
    }

    pub const fn total_bit_length(self) -> u32 {
        FLAT7X_AU_FRAME_BITS
    }

    pub const fn production_allowed(self) -> bool {
        false
    }
}

/// Complete, nonpublishing I0→D0 byte candidate with both frame-integrity receipts.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xI0D0AuCandidate {
    target: Flat7xI0D0TargetPlan,
    i0_frame: Flat7xI0Ac3AudioSeriesFrameCandidate,
    d0_frame_bytes: [u8; D0_TARGET_FRAME_BYTES],
    d0_frame_receipt: Flat7xD0FrameAssemblyReceipt,
    access_unit_bytes: [u8; FLAT7X_AU_FRAME_BYTES],
    access_unit_receipt: Flat7xI0D0AuCompositionReceipt,
}

impl Flat7xI0D0AuCandidate {
    pub const fn target(&self) -> Flat7xI0D0TargetPlan {
        self.target
    }

    pub fn source_identity(&self) -> &SharedAnalysisIdentity {
        self.i0_frame.source_identity()
    }

    pub const fn i0_frame(&self) -> &Flat7xI0Ac3AudioSeriesFrameCandidate {
        &self.i0_frame
    }

    pub const fn d0_frame_bytes(&self) -> &[u8; D0_TARGET_FRAME_BYTES] {
        &self.d0_frame_bytes
    }

    pub const fn d0_frame_receipt(&self) -> Flat7xD0FrameAssemblyReceipt {
        self.d0_frame_receipt
    }

    pub const fn access_unit_bytes(&self) -> &[u8; FLAT7X_AU_FRAME_BYTES] {
        &self.access_unit_bytes
    }

    pub const fn access_unit_receipt(&self) -> Flat7xI0D0AuCompositionReceipt {
        self.access_unit_receipt
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xI0D0AuCompositionError {
    CursorMismatch { actual: u32 },
    UnsupportedTargetContext,
    I0FrameExtentMismatch { expected: u16, actual: usize },
    D0FrameExtentMismatch { expected: u16, actual: usize },
    AggregateExtentMismatch { expected: u32, actual: u32 },
    BitWriter(FrameBitWriterError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xI0D0AuCandidateError {
    UnsupportedTargetContext,
    SourceIdentityMismatch,
    D0FrameAssembly(Flat7xD0FrameAssemblyError),
    AuComposition(Flat7xI0D0AuCompositionError),
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for Flat7xI0D0AuCompositionError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

impl From<FrameBitWriterError> for Flat7xI0D0AuCandidateError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Builds one in-memory I0→D0 AU candidate from a source-bound I0 frame and caller-owned D0 spans.
///
/// The D0 assembler enforces the approved 1024-kbps/4096-byte target, the six ordered blocks and
/// one complete block-index-2 skip-field. The existing AU composer then lays out I0 before D0.
/// Both frames and the AU remain nonpublishing candidates; no audio or EMDF payload is decoded.
pub fn assemble_flat7x_i0_d0_au_candidate(
    target: Flat7xI0D0TargetPlan,
    expected_source_identity: &SharedAnalysisIdentity,
    i0_frame: Flat7xI0Ac3AudioSeriesFrameCandidate,
    d0_bsi: Flat7xD0BsiStructure<'_>,
    d0_audio_blocks: Flat7xD0FrameBitInput<'_>,
) -> Result<Flat7xI0D0AuCandidate, Flat7xI0D0AuCandidateError> {
    if target != Flat7xI0D0TargetPlan::approved_contract() || i0_frame.target() != target {
        return Err(Flat7xI0D0AuCandidateError::UnsupportedTargetContext);
    }
    if i0_frame.source_identity() != expected_source_identity {
        return Err(Flat7xI0D0AuCandidateError::SourceIdentityMismatch);
    }

    let mut d0_frame_bytes = [0_u8; D0_TARGET_FRAME_BYTES];
    let d0_frame_receipt = {
        let mut d0_writer = FrameBitWriter::new(&mut d0_frame_bytes, FrameBitCursor::new(0))?;
        assemble_flat7x_d0_frame_bits(target, d0_bsi, d0_audio_blocks, &mut d0_writer)
            .map_err(Flat7xI0D0AuCandidateError::D0FrameAssembly)?
    };

    let d0_skip_target = target.d0_emdf_skip_field();
    let d0_layout = d0_frame_receipt.layout();
    if d0_frame_receipt.target_frame_bytes() != D0_TARGET_FRAME_BYTES as u16
        || d0_layout.audio_blocks()[2].block_index().zero_based_value()
            != d0_skip_target.zero_based_audio_block()
        || d0_skip_target.zero_based_audio_block() != 2
        || d0_skip_target.field_count() != 1
        || d0_layout.block_two_skip_field().payload_bytes() == 0
        || d0_layout.block_two_skip_field().payload_bytes()
            > d0_skip_target.maximum_complete_container_bytes()
        || d0_skip_target.fragmentation_allowed()
    {
        return Err(Flat7xI0D0AuCandidateError::UnsupportedTargetContext);
    }

    let mut access_unit_bytes = [0_u8; FLAT7X_AU_FRAME_BYTES];
    let access_unit_receipt = {
        let mut au_writer = FrameBitWriter::new(&mut access_unit_bytes, FrameBitCursor::new(0))?;
        write_flat7x_i0_d0_access_unit_bytes(
            target,
            Flat7xI0D0OpaqueFrameSpans {
                i0: OpaqueI0FrameBytes::new(i0_frame.frame_bytes()),
                d0: OpaqueD0FrameBytes::new(&d0_frame_bytes),
            },
            &mut au_writer,
        )
        .map_err(Flat7xI0D0AuCandidateError::AuComposition)?
    };

    Ok(Flat7xI0D0AuCandidate {
        target,
        i0_frame,
        d0_frame_bytes,
        d0_frame_receipt,
        access_unit_bytes,
        access_unit_receipt,
    })
}

/// Writes an opaque I0 frame followed by an opaque D0 frame as a model AU byte sequence.
///
/// This validates only the approved target/extents, stages the complete 6656-byte ordering,
/// preflights caller capacity, then commits once. It does not parse BSID, validate CRC, infer a
/// source generation, or generate/transform frame bytes. The caller-supplied frame spans remain
/// opaque and this proposal is never production-authorized.
pub fn write_flat7x_i0_d0_access_unit_bytes(
    target: Flat7xI0D0TargetPlan,
    frames: Flat7xI0D0OpaqueFrameSpans<'_>,
    writer: &mut FrameBitWriter<'_>,
) -> Result<Flat7xI0D0AuCompositionReceipt, Flat7xI0D0AuCompositionError> {
    let actual_cursor = writer.cursor().bit_offset();
    if actual_cursor != 0 {
        return Err(Flat7xI0D0AuCompositionError::CursorMismatch {
            actual: actual_cursor,
        });
    }
    if target != Flat7xI0D0TargetPlan::approved_contract()
        || target.i0().frame_bytes() as usize != FLAT7X_I0_FRAME_BYTES
        || target.d0().frame_bytes() as usize != FLAT7X_D0_FRAME_BYTES
    {
        return Err(Flat7xI0D0AuCompositionError::UnsupportedTargetContext);
    }
    if frames.i0.as_bytes().len() != FLAT7X_I0_FRAME_BYTES {
        return Err(Flat7xI0D0AuCompositionError::I0FrameExtentMismatch {
            expected: FLAT7X_I0_FRAME_BYTES as u16,
            actual: frames.i0.as_bytes().len(),
        });
    }
    if frames.d0.as_bytes().len() != FLAT7X_D0_FRAME_BYTES {
        return Err(Flat7xI0D0AuCompositionError::D0FrameExtentMismatch {
            expected: FLAT7X_D0_FRAME_BYTES as u16,
            actual: frames.d0.as_bytes().len(),
        });
    }
    let aggregate_bytes = target.aggregate_frame_bytes();
    if aggregate_bytes != FLAT7X_AU_FRAME_BYTES as u32 {
        return Err(Flat7xI0D0AuCompositionError::AggregateExtentMismatch {
            expected: FLAT7X_AU_FRAME_BYTES as u32,
            actual: aggregate_bytes,
        });
    }
    writer.preflight_capacity(FLAT7X_AU_FRAME_BITS)?;

    let mut staging = [0_u8; FLAT7X_AU_FRAME_BYTES];
    staging[..FLAT7X_I0_FRAME_BYTES].copy_from_slice(frames.i0.as_bytes());
    staging[FLAT7X_I0_FRAME_BYTES..].copy_from_slice(frames.d0.as_bytes());
    writer.write_bytes(&staging)?;

    Ok(Flat7xI0D0AuCompositionReceipt {
        i0_span: Flat7xAuByteSpan {
            start_byte: 0,
            byte_length: FLAT7X_I0_FRAME_BYTES as u16,
        },
        d0_span: Flat7xAuByteSpan {
            start_byte: FLAT7X_I0_FRAME_BYTES as u16,
            byte_length: FLAT7X_D0_FRAME_BYTES as u16,
        },
    })
}
