// pattern: Functional Core
//! Writes the numeric E-AC-3 channel-map field for the selected Flat7x D0 target.
//!
//! The caller owns the starting bit cursor and surrounding header syntax. This
//! module emits exactly the typed 16-bit `chanmap` value in MSB-first order.

use crate::{
    codec::bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
    profile::DependentChannelMap,
};

const CHANMAP_FIELD_WIDTH_BITS: u8 = 16;

/// Writes one Flat7x D0 `chanmap` numeric field at the caller-provided cursor.
pub fn write_flat7x_d0_chanmap_field(
    writer: &mut FrameBitWriter<'_>,
    channel_map: DependentChannelMap,
) -> Result<FrameBitCursor, FrameBitWriterError> {
    writer.write_unsigned(u32::from(channel_map.value()), CHANMAP_FIELD_WIDTH_BITS)?;
    Ok(writer.cursor())
}
