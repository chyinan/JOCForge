// pattern: Functional Core
//! Caller-owned Flat7x D0 six-block bit layout proposal.

use super::{
    emdf_skip_field::{D0EmdfSkipFieldWriteError, flat7x_d0_block2_skipfld_bit_length},
    flat7x_d0_frame_controls::FLAT7X_D0_AUDIO_FRAME_CONTROL_BITS,
    frame_builder::{
        Flat7xD0BsiStructure, Flat7xD0BsiStructureError, measure_flat7x_d0_bsi_structure,
    },
};
use crate::profile::Flat7xI0D0TargetPlan;

const D0_AUDIO_BLOCK_COUNT: usize = 6;
const D0_BLOCK_TWO_INDEX: usize = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D0AudioBlockIndex(u8);

impl D0AudioBlockIndex {
    pub const fn zero_based_value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D0AudioBlockBitSpan {
    bit_length: u32,
}

impl D0AudioBlockBitSpan {
    pub const fn new(bit_length: u32) -> Self {
        Self { bit_length }
    }

    pub const fn bit_length(self) -> u32 {
        self.bit_length
    }
}

/// An insertion offset supplied by the caller; no native block cursor is implied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CallerOwnedUnboundD0Block2InsertionPoint {
    offset_within_audio_span_bits: u32,
}

impl CallerOwnedUnboundD0Block2InsertionPoint {
    pub const fn new(offset_within_audio_span_bits: u32) -> Self {
        Self {
            offset_within_audio_span_bits,
        }
    }

    pub const fn offset_within_audio_span_bits(self) -> u32 {
        self.offset_within_audio_span_bits
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D0Block2SkipFieldLayoutInput {
    pub insertion_point: CallerOwnedUnboundD0Block2InsertionPoint,
    pub payload_bytes: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D0AudioBlockLayoutInput<'a> {
    pub audio_span: D0AudioBlockBitSpan,
    pub skip_fields: &'a [D0Block2SkipFieldLayoutInput],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D0SixBlockLayoutInput<'a> {
    pub blocks: [D0AudioBlockLayoutInput<'a>; D0_AUDIO_BLOCK_COUNT],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D0FrameBitSpan {
    start_bit: u32,
    bit_length: u32,
}

impl D0FrameBitSpan {
    pub(super) const fn new(start_bit: u32, bit_length: u32) -> Self {
        Self {
            start_bit,
            bit_length,
        }
    }

    pub const fn start_bit(self) -> u32 {
        self.start_bit
    }

    pub const fn bit_length(self) -> u32 {
        self.bit_length
    }

    pub const fn end_bit(self) -> u32 {
        self.start_bit + self.bit_length
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D0AudioBlockLayout {
    block_index: D0AudioBlockIndex,
    leading_audio_span: D0FrameBitSpan,
    trailing_audio_span: Option<D0FrameBitSpan>,
}

impl D0AudioBlockLayout {
    pub const fn block_index(self) -> D0AudioBlockIndex {
        self.block_index
    }

    /// The complete span for blocks other than block two; block two's prefix otherwise.
    pub const fn leading_audio_span(self) -> D0FrameBitSpan {
        self.leading_audio_span
    }

    /// Block two's post-field audio span; absent for every other block.
    pub const fn trailing_audio_span(self) -> Option<D0FrameBitSpan> {
        self.trailing_audio_span
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D0Block2SkipFieldLayout {
    insertion_point: CallerOwnedUnboundD0Block2InsertionPoint,
    bit_span: D0FrameBitSpan,
    payload_bytes: u16,
}

impl D0Block2SkipFieldLayout {
    pub const fn insertion_point(self) -> CallerOwnedUnboundD0Block2InsertionPoint {
        self.insertion_point
    }

    pub const fn bit_span(self) -> D0FrameBitSpan {
        self.bit_span
    }

    pub const fn payload_bytes(self) -> u16 {
        self.payload_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum D0SixBlockLayoutError {
    UnsupportedTargetContext,
    BsiStructure(Flat7xD0BsiStructureError),
    SkipField(D0EmdfSkipFieldWriteError),
    SkipFieldOnNonBlockTwo { block_index: D0AudioBlockIndex },
    MissingBlockTwoSkipField,
    DuplicateBlockTwoSkipFields,
    InsertionPointOutsideAudioSpan { offset_bits: u32, span_bits: u32 },
    BitSpanOverflow,
    FrameSpanExceedsTarget { span_bits: u32, maximum_bits: u32 },
    InternalBlockCountMismatch,
}

impl From<Flat7xD0BsiStructureError> for D0SixBlockLayoutError {
    fn from(error: Flat7xD0BsiStructureError) -> Self {
        Self::BsiStructure(error)
    }
}

impl From<D0EmdfSkipFieldWriteError> for D0SixBlockLayoutError {
    fn from(error: D0EmdfSkipFieldWriteError) -> Self {
        Self::SkipField(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0SixBlockLayout {
    bsi_span: D0FrameBitSpan,
    audio_frame_control_span: D0FrameBitSpan,
    audio_blocks: [D0AudioBlockLayout; D0_AUDIO_BLOCK_COUNT],
    block_two_skip_field: D0Block2SkipFieldLayout,
    total_frame_span_bits: u32,
    target_frame_bytes: u16,
}

impl Flat7xD0SixBlockLayout {
    pub const fn bsi_span(self) -> D0FrameBitSpan {
        self.bsi_span
    }

    pub const fn audio_frame_control_span(self) -> D0FrameBitSpan {
        self.audio_frame_control_span
    }

    pub const fn audio_blocks(self) -> [D0AudioBlockLayout; D0_AUDIO_BLOCK_COUNT] {
        self.audio_blocks
    }

    pub const fn block_two_skip_field(self) -> D0Block2SkipFieldLayout {
        self.block_two_skip_field
    }

    pub const fn total_frame_span_bits(self) -> u32 {
        self.total_frame_span_bits
    }

    pub const fn target_frame_bytes(self) -> u16 {
        self.target_frame_bytes
    }

    pub const fn production_allowed(self) -> bool {
        false
    }
}

/// Proposes a six-block D0 layout using only opaque audio-span lengths.
///
/// The block array is ordered 0 through 5. Blocks other than block two must have
/// no skip-field inputs; block two must have exactly one complete field and an
/// explicit caller-owned insertion point. The function validates the BSI
/// structure, skip-field extent, span arithmetic, and the 4096-byte target
/// extent before returning a plan. It reads no audio span bytes and mutates no
/// caller state. The insertion point remains caller-owned and unbound.
pub fn propose_flat7x_d0_six_block_layout(
    target: Flat7xI0D0TargetPlan,
    bsi_structure: Flat7xD0BsiStructure<'_>,
    input: D0SixBlockLayoutInput<'_>,
) -> Result<Flat7xD0SixBlockLayout, D0SixBlockLayoutError> {
    if target != Flat7xI0D0TargetPlan::approved_contract() {
        return Err(D0SixBlockLayoutError::UnsupportedTargetContext);
    }

    let bsi_end_bit = measure_flat7x_d0_bsi_structure(target, &bsi_structure)?.value();

    for (index, block) in input.blocks.iter().enumerate() {
        if index != D0_BLOCK_TWO_INDEX && !block.skip_fields.is_empty() {
            return Err(D0SixBlockLayoutError::SkipFieldOnNonBlockTwo {
                block_index: D0AudioBlockIndex(
                    u8::try_from(index)
                        .map_err(|_| D0SixBlockLayoutError::InternalBlockCountMismatch)?,
                ),
            });
        }
    }

    let block_two = input.blocks[D0_BLOCK_TWO_INDEX];
    let [skip_field] = block_two.skip_fields else {
        return Err(if block_two.skip_fields.is_empty() {
            D0SixBlockLayoutError::MissingBlockTwoSkipField
        } else {
            D0SixBlockLayoutError::DuplicateBlockTwoSkipFields
        });
    };
    let insertion_offset = skip_field.insertion_point.offset_within_audio_span_bits();
    let block_two_audio_bits = block_two.audio_span.bit_length();
    if insertion_offset > block_two_audio_bits {
        return Err(D0SixBlockLayoutError::InsertionPointOutsideAudioSpan {
            offset_bits: insertion_offset,
            span_bits: block_two_audio_bits,
        });
    }

    let skip_field_bits = flat7x_d0_block2_skipfld_bit_length(skip_field.payload_bytes)?;
    let audio_frame_control_span =
        D0FrameBitSpan::new(bsi_end_bit, FLAT7X_D0_AUDIO_FRAME_CONTROL_BITS);
    let audio_frame_control_end = audio_frame_control_span.end_bit();
    let audio_bits = input.blocks.iter().try_fold(0_u32, |total, block| {
        total
            .checked_add(block.audio_span.bit_length())
            .ok_or(D0SixBlockLayoutError::BitSpanOverflow)
    })?;
    let total_frame_span_bits = audio_frame_control_end
        .checked_add(audio_bits)
        .and_then(|bits| bits.checked_add(skip_field_bits))
        .ok_or(D0SixBlockLayoutError::BitSpanOverflow)?;
    let target_frame_bytes = target.d0().frame_bytes();
    let maximum_bits = u32::from(target_frame_bytes)
        .checked_mul(8)
        .ok_or(D0SixBlockLayoutError::BitSpanOverflow)?;
    if total_frame_span_bits > maximum_bits {
        return Err(D0SixBlockLayoutError::FrameSpanExceedsTarget {
            span_bits: total_frame_span_bits,
            maximum_bits,
        });
    }

    let mut current_bit = audio_frame_control_end;
    let mut audio_layouts = Vec::with_capacity(D0_AUDIO_BLOCK_COUNT);
    let mut skip_field_layout = None;
    for (index, block) in input.blocks.iter().enumerate() {
        let block_index = D0AudioBlockIndex(
            u8::try_from(index).map_err(|_| D0SixBlockLayoutError::InternalBlockCountMismatch)?,
        );
        let audio_bits = block.audio_span.bit_length();
        if index == D0_BLOCK_TWO_INDEX {
            let leading = D0FrameBitSpan::new(current_bit, insertion_offset);
            let field_start = leading.end_bit();
            let field = D0FrameBitSpan::new(field_start, skip_field_bits);
            let trailing_start = field.end_bit();
            let trailing_bits = audio_bits - insertion_offset;
            let trailing = D0FrameBitSpan::new(trailing_start, trailing_bits);
            current_bit = trailing.end_bit();
            skip_field_layout = Some(D0Block2SkipFieldLayout {
                insertion_point: skip_field.insertion_point,
                bit_span: field,
                payload_bytes: skip_field.payload_bytes,
            });
            audio_layouts.push(D0AudioBlockLayout {
                block_index,
                leading_audio_span: leading,
                trailing_audio_span: Some(trailing),
            });
        } else {
            let span = D0FrameBitSpan::new(current_bit, audio_bits);
            current_bit = span.end_bit();
            audio_layouts.push(D0AudioBlockLayout {
                block_index,
                leading_audio_span: span,
                trailing_audio_span: None,
            });
        }
    }
    if current_bit != total_frame_span_bits {
        return Err(D0SixBlockLayoutError::BitSpanOverflow);
    }
    let audio_blocks = audio_layouts
        .try_into()
        .map_err(|_| D0SixBlockLayoutError::InternalBlockCountMismatch)?;
    let block_two_skip_field =
        skip_field_layout.ok_or(D0SixBlockLayoutError::MissingBlockTwoSkipField)?;

    Ok(Flat7xD0SixBlockLayout {
        bsi_span: D0FrameBitSpan::new(0, bsi_end_bit),
        audio_frame_control_span,
        audio_blocks,
        block_two_skip_field,
        total_frame_span_bits,
        target_frame_bytes,
    })
}
