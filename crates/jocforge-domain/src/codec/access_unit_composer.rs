// pattern: Functional Core
//! Production-disabled byte ordering for caller-supplied Flat7x I0/D0 frame spans.

use super::bitio::{FrameBitWriter, FrameBitWriterError};
use crate::profile::Flat7xI0D0TargetPlan;

const FLAT7X_I0_FRAME_BYTES: usize = 2_560;
const FLAT7X_D0_FRAME_BYTES: usize = 4_096;
const FLAT7X_AU_FRAME_BYTES: usize = 6_656;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xI0D0AuCompositionError {
    CursorMismatch { actual: u32 },
    UnsupportedTargetContext,
    I0FrameExtentMismatch { expected: u16, actual: usize },
    D0FrameExtentMismatch { expected: u16, actual: usize },
    AggregateExtentMismatch { expected: u32, actual: u32 },
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for Flat7xI0D0AuCompositionError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
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
