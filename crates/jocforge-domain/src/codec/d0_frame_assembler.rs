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
    frame_builder::{
        Flat7xD0BsiStructure, Flat7xD0BsiStructureError, write_flat7x_d0_bsi_structure,
    },
};
use crate::{integrity::crc16::eac3_crc16, profile::Flat7xI0D0TargetPlan};

pub const D0_TARGET_FRAME_BYTES: usize = 4_096;
pub const D0_TARGET_FRAME_BITS: u32 = 32_768;
pub const D0_CRC2_FIELD_BITS: u32 = 16;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D0Block2SkipFieldInput<'a> {
    pub insertion_point: CallerOwnedUnboundD0Block2InsertionPoint,
    pub complete_emdf_container: &'a [u8],
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
    pub audio_blocks: [D0FrameAudioBlockInput<'a>; D0_AUDIO_BLOCK_COUNT],
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
    crc2_span: D0Crc2BitSpan,
    crc2_integrity: D0Crc2IntegrityReceipt,
}

impl Flat7xD0FrameAssemblyReceipt {
    pub const fn layout(self) -> Flat7xD0SixBlockLayout {
        self.layout
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
    bsi_structure: Flat7xD0BsiStructure<'_>,
    input: Flat7xD0FrameBitInput<'_>,
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
    let container_bytes =
        u16::try_from(block_two_input.skip_fields[0].complete_emdf_container.len()).map_err(
            |_| Flat7xD0FrameAssemblyError::SkipField(D0EmdfSkipFieldWriteError::ContainerTooLarge),
        )?;
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
    let frame_bytes = target.d0().frame_bytes();
    let required_frame_bits = u32::from(frame_bytes).checked_mul(8).ok_or(
        Flat7xD0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: layout.total_frame_span_bits(),
            required_bits: D0_TARGET_FRAME_BITS,
        },
    )?;
    let full_span_bits = layout
        .total_frame_span_bits()
        .checked_add(D0_CRC2_FIELD_BITS)
        .ok_or(Flat7xD0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: layout.total_frame_span_bits(),
            required_bits: required_frame_bits,
        })?;
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
                        block_two.skip_fields[0].complete_emdf_container,
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

    let crc2_start_bit = layout.total_frame_span_bits();
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
        crc2_span: D0Crc2BitSpan {
            start_bit: crc2_start_bit,
        },
        crc2_integrity: D0Crc2IntegrityReceipt {
            generated_value: crc2_value,
            verified_residual,
        },
    })
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
