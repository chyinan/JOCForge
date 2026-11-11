// pattern: Functional Core
//! Production-disabled composition of caller-supplied Flat7x access-unit byte spans.

use super::bitio::{FrameBitWriter, FrameBitWriterError};
use crate::profile::Flat7xI0D0TargetPlan;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xOpaqueAccessUnitBytes<'a>(&'a [u8]);

impl<'a> Flat7xOpaqueAccessUnitBytes<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(self) -> &'a [u8] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xElementaryStreamExtent {
    access_unit_count: u32,
    byte_length: u32,
    bit_length: u32,
}

impl Flat7xElementaryStreamExtent {
    pub const fn access_unit_count(self) -> u32 {
        self.access_unit_count
    }

    pub const fn byte_length(self) -> u32 {
        self.byte_length
    }

    pub const fn bit_length(self) -> u32 {
        self.bit_length
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xElementaryStreamCompositionReceipt {
    start_byte: u32,
    extent: Flat7xElementaryStreamExtent,
}

impl Flat7xElementaryStreamCompositionReceipt {
    pub const fn start_byte(self) -> u32 {
        self.start_byte
    }

    pub const fn extent(self) -> Flat7xElementaryStreamExtent {
        self.extent
    }

    pub const fn production_allowed(self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xElementaryStreamCompositionError {
    EmptyAccessUnitSequence,
    UnsupportedTargetContext,
    SequenceExtentOverflow,
    AccessUnitExtentMismatch {
        au_index: u32,
        expected: u32,
        actual: usize,
    },
    WriterCursorNotByteAligned {
        actual_bit_offset: u32,
    },
    StagingAllocationFailed,
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for Flat7xElementaryStreamCompositionError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Measures a nonempty Flat7x sequence before inspecting or staging its AU bytes.
pub fn measure_flat7x_elementary_stream(
    target: Flat7xI0D0TargetPlan,
    access_unit_count: usize,
) -> Result<Flat7xElementaryStreamExtent, Flat7xElementaryStreamCompositionError> {
    if access_unit_count == 0 {
        return Err(Flat7xElementaryStreamCompositionError::EmptyAccessUnitSequence);
    }
    if target != Flat7xI0D0TargetPlan::approved_contract() {
        return Err(Flat7xElementaryStreamCompositionError::UnsupportedTargetContext);
    }

    let au_byte_length = usize::try_from(target.aggregate_frame_bytes())
        .map_err(|_| Flat7xElementaryStreamCompositionError::SequenceExtentOverflow)?;
    let byte_length = access_unit_count
        .checked_mul(au_byte_length)
        .ok_or(Flat7xElementaryStreamCompositionError::SequenceExtentOverflow)?;
    let bit_length = byte_length
        .checked_mul(8)
        .ok_or(Flat7xElementaryStreamCompositionError::SequenceExtentOverflow)?;

    Ok(Flat7xElementaryStreamExtent {
        access_unit_count: u32::try_from(access_unit_count)
            .map_err(|_| Flat7xElementaryStreamCompositionError::SequenceExtentOverflow)?,
        byte_length: u32::try_from(byte_length)
            .map_err(|_| Flat7xElementaryStreamCompositionError::SequenceExtentOverflow)?,
        bit_length: u32::try_from(bit_length)
            .map_err(|_| Flat7xElementaryStreamCompositionError::SequenceExtentOverflow)?,
    })
}

/// Stages and appends a complete ordered sequence of opaque Flat7x access units.
///
/// Every AU must match the approved 6656-byte target. The complete sequence size and caller
/// capacity are checked before allocating private staging. The writer cursor must be byte-aligned;
/// successful composition uses one writer call and returns a noncommitting layout receipt.
pub fn compose_flat7x_elementary_stream(
    target: Flat7xI0D0TargetPlan,
    access_units: &[Flat7xOpaqueAccessUnitBytes<'_>],
    writer: &mut FrameBitWriter<'_>,
) -> Result<Flat7xElementaryStreamCompositionReceipt, Flat7xElementaryStreamCompositionError> {
    let extent = measure_flat7x_elementary_stream(target, access_units.len())?;
    let au_byte_length = target.aggregate_frame_bytes();
    let au_byte_length_usize = usize::try_from(au_byte_length)
        .map_err(|_| Flat7xElementaryStreamCompositionError::SequenceExtentOverflow)?;

    for (index, access_unit) in access_units.iter().enumerate() {
        if access_unit.as_bytes().len() != au_byte_length_usize {
            return Err(
                Flat7xElementaryStreamCompositionError::AccessUnitExtentMismatch {
                    au_index: u32::try_from(index).map_err(|_| {
                        Flat7xElementaryStreamCompositionError::SequenceExtentOverflow
                    })?,
                    expected: au_byte_length,
                    actual: access_unit.as_bytes().len(),
                },
            );
        }
    }

    let start_bit_offset = writer.cursor().bit_offset();
    if start_bit_offset % 8 != 0 {
        return Err(
            Flat7xElementaryStreamCompositionError::WriterCursorNotByteAligned {
                actual_bit_offset: start_bit_offset,
            },
        );
    }
    start_bit_offset.checked_add(extent.bit_length()).ok_or(
        Flat7xElementaryStreamCompositionError::BitWriter(FrameBitWriterError::CursorOverflow),
    )?;
    writer.preflight_capacity(extent.bit_length())?;

    let staging_length = usize::try_from(extent.byte_length())
        .map_err(|_| Flat7xElementaryStreamCompositionError::SequenceExtentOverflow)?;
    let mut staging = Vec::new();
    staging
        .try_reserve_exact(staging_length)
        .map_err(|_| Flat7xElementaryStreamCompositionError::StagingAllocationFailed)?;
    for access_unit in access_units {
        staging.extend_from_slice(access_unit.as_bytes());
    }

    writer.write_bytes(&staging)?;

    Ok(Flat7xElementaryStreamCompositionReceipt {
        start_byte: start_bit_offset / 8,
        extent,
    })
}
