// pattern: Functional Core
//! One Flat7x D0 `audblk` from typed spectrum, D15, BAP, and frame-control inputs.

pub use super::flat7x_d0_exponents::FLAT7X_D0_D15_STRATEGY_CODE;

use super::{
    bap::{FLAT7X_D0_BAP_CHANNEL_COUNT, Flat7xD0BapFrame},
    bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
    d0_frame_assembler::{
        D0Block2SkipFieldInput, D0BlockTwoAudioSpans, D0FrameAudioBlockInput, OpaqueD0AudioBitSpan,
    },
    exponent_packing::Eac3ExponentRowWriteError,
    flat7x_d0_exponents::{
        FLAT7X_D0_D15_GROUP_COUNT, Flat7xD0ExponentRow, write_flat7x_d0_d15_exponent_row,
    },
    flat7x_d0_frame_controls::Flat7xD0AudioFrameControlPolicy,
    flat7x_d0_mantissa::{
        Flat7xD0AudioBlockIndex, Flat7xD0MantissaBlock, Flat7xD0MantissaInput,
        pack_flat7x_d0_mantissa_block,
    },
    flat7x_d0_spectrum::Flat7xD0SpectrumBlock,
    grouped_quantizer::MantissaBlockError,
    mantissa::MANTISSA_WIDTH_BY_BAP,
};

pub const FLAT7X_D0_AUDIO_BLOCK_POLICY_ID: &str = "JOCFORGE_FLAT7X_D0_LONG_D15_NO_CHANNEL_TOOLS_V1";
pub const FLAT7X_D0_DEFAULT_FAST_GAIN_CODE: u8 = 4;
pub const FLAT7X_D0_CHANNEL_BANDWIDTH_CODE: u8 = 60;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0AudioBlockInput<'a> {
    pub block_index: Flat7xD0AudioBlockIndex,
    pub spectrum: &'a Flat7xD0SpectrumBlock,
    pub exponent_rows: &'a [Flat7xD0ExponentRow; FLAT7X_D0_BAP_CHANNEL_COUNT],
    pub bap_frame: &'a Flat7xD0BapFrame,
    pub frame_control_policy: Flat7xD0AudioFrameControlPolicy,
    pub dynamic_range_code: Option<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0AudioBlockReceipt {
    block_index: Flat7xD0AudioBlockIndex,
    bit_length: u32,
    prefix_before_skipfield_bits: u32,
    mantissa_bits: u32,
    nonzero_bap_bins: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0AudioBlockBitCost {
    bit_length: u32,
    prefix_before_skipfield_bits: u32,
    mantissa_bits: u32,
    nonzero_bap_bins: u16,
}

impl Flat7xD0AudioBlockBitCost {
    pub const fn bit_length(self) -> u32 {
        self.bit_length
    }

    pub const fn prefix_before_skipfield_bits(self) -> u32 {
        self.prefix_before_skipfield_bits
    }

    pub const fn mantissa_bits(self) -> u32 {
        self.mantissa_bits
    }

    pub const fn nonzero_bap_bins(self) -> u16 {
        self.nonzero_bap_bins
    }
}

impl Flat7xD0AudioBlockReceipt {
    pub const fn block_index(self) -> Flat7xD0AudioBlockIndex {
        self.block_index
    }

    pub const fn bit_length(self) -> u32 {
        self.bit_length
    }

    pub const fn prefix_before_skipfield_bits(self) -> u32 {
        self.prefix_before_skipfield_bits
    }

    pub const fn mantissa_bits(self) -> u32 {
        self.mantissa_bits
    }

    pub const fn nonzero_bap_bins(self) -> u16 {
        self.nonzero_bap_bins
    }

    pub const fn production_allowed(self) -> bool {
        false
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Flat7xD0AudioBlockPayload {
    Ordinary {
        bytes: Vec<u8>,
        bit_length: u32,
    },
    BlockTwo {
        before_skipfield: Vec<u8>,
        before_skipfield_bits: u32,
        after_skipfield: Vec<u8>,
        after_skipfield_bits: u32,
    },
}

/// Complete block syntax excluding the external block-two EMDF skip-field insertion.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xD0AudioBlockCandidate {
    payload: Flat7xD0AudioBlockPayload,
    receipt: Flat7xD0AudioBlockReceipt,
}

impl Flat7xD0AudioBlockCandidate {
    pub const fn receipt(&self) -> Flat7xD0AudioBlockReceipt {
        self.receipt
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }

    pub fn as_frame_audio_block_input<'a>(
        &'a self,
        skip_fields: &'a [D0Block2SkipFieldInput<'a>],
    ) -> Option<D0FrameAudioBlockInput<'a>> {
        match &self.payload {
            Flat7xD0AudioBlockPayload::Ordinary { bytes, bit_length } => {
                if self.receipt.block_index.value() == 2 {
                    return None;
                }
                Some(D0FrameAudioBlockInput::Opaque(OpaqueD0AudioBitSpan::new(
                    bytes,
                    0,
                    *bit_length,
                )))
            }
            Flat7xD0AudioBlockPayload::BlockTwo {
                before_skipfield,
                before_skipfield_bits,
                after_skipfield,
                after_skipfield_bits,
            } => {
                if self.receipt.block_index.value() != 2 || skip_fields.len() != 1 {
                    return None;
                }
                Some(D0FrameAudioBlockInput::BlockTwo(D0BlockTwoAudioSpans {
                    before_skipfield: OpaqueD0AudioBitSpan::new(
                        before_skipfield,
                        0,
                        *before_skipfield_bits,
                    ),
                    skip_fields,
                    after_skipfield: OpaqueD0AudioBitSpan::new(
                        after_skipfield,
                        0,
                        *after_skipfield_bits,
                    ),
                }))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0AudioBlockError {
    BapCandidateMismatch { frame_q: u16, control_q: u16 },
    Mantissa(MantissaBlockError),
    ExponentRow(Eac3ExponentRowWriteError),
    BitCountOverflow,
    BitWriter(FrameBitWriterError),
}

impl From<MantissaBlockError> for Flat7xD0AudioBlockError {
    fn from(error: MantissaBlockError) -> Self {
        Self::Mantissa(error)
    }
}

impl From<Eac3ExponentRowWriteError> for Flat7xD0AudioBlockError {
    fn from(error: Eac3ExponentRowWriteError) -> Self {
        Self::ExponentRow(error)
    }
}

impl From<FrameBitWriterError> for Flat7xD0AudioBlockError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Writes one D0 block with long-transform, D15, no-coupling, no-AHT, and no-SPX policy.
///
/// Frame-level fields and the D0 block-two skip-field are owned by their frame assemblers. Block
/// two returns separate before/after spans so the caller can place its single skip-field before
/// mantissas. Other blocks return one complete span containing `skiple=0`. Per ETSI TS 102 366
/// V1.4.1 Annex E.1.2.4, dither flags precede the dynamic-range presence/value fields.
pub fn write_flat7x_d0_audio_block(
    input: &Flat7xD0AudioBlockInput<'_>,
) -> Result<Flat7xD0AudioBlockCandidate, Flat7xD0AudioBlockError> {
    let bit_cost = measure_flat7x_d0_audio_block_bit_cost(input)?;

    let mantissas = pack_flat7x_d0_mantissa_block(&Flat7xD0MantissaInput {
        spectrum: input.spectrum,
        exponent_rows: input.exponent_rows,
        bap_frame: input.bap_frame,
        block_index: input.block_index,
    })?;
    let mantissa_bits = u32::try_from(mantissas.bit_count())
        .map_err(|_| Flat7xD0AudioBlockError::BitCountOverflow)?;
    if mantissa_bits != bit_cost.mantissa_bits() {
        return Err(Flat7xD0AudioBlockError::BitCountOverflow);
    }

    let is_block_two = input.block_index.value() == 2;
    let prefix_bits = bit_cost.prefix_before_skipfield_bits();
    let complete_bits = bit_cost.bit_length();

    if is_block_two {
        let mut prefix_bytes = zeroed_bit_buffer(prefix_bits)?;
        {
            let mut writer = FrameBitWriter::new(&mut prefix_bytes, FrameBitCursor::new(0))?;
            write_prefix_before_skipfield(&mut writer, input)?;
            if writer.cursor().bit_offset() != prefix_bits {
                return Err(Flat7xD0AudioBlockError::BitCountOverflow);
            }
        }
        Ok(Flat7xD0AudioBlockCandidate {
            payload: Flat7xD0AudioBlockPayload::BlockTwo {
                before_skipfield: prefix_bytes,
                before_skipfield_bits: prefix_bits,
                after_skipfield: mantissas.packed_bytes().to_vec(),
                after_skipfield_bits: mantissa_bits,
            },
            receipt: Flat7xD0AudioBlockReceipt {
                block_index: input.block_index,
                bit_length: complete_bits,
                prefix_before_skipfield_bits: prefix_bits,
                mantissa_bits,
                nonzero_bap_bins: bit_cost.nonzero_bap_bins(),
            },
        })
    } else {
        let mut bytes = zeroed_bit_buffer(complete_bits)?;
        {
            let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0))?;
            write_prefix_before_skipfield(&mut writer, input)?;
            writer.write_bit(false)?; // skiple=0 for every block except block two.
            write_packed_mantissa_bits(&mut writer, &mantissas)?;
            if writer.cursor().bit_offset() != complete_bits {
                return Err(Flat7xD0AudioBlockError::BitCountOverflow);
            }
        }
        Ok(Flat7xD0AudioBlockCandidate {
            payload: Flat7xD0AudioBlockPayload::Ordinary {
                bytes,
                bit_length: complete_bits,
            },
            receipt: Flat7xD0AudioBlockReceipt {
                block_index: input.block_index,
                bit_length: complete_bits,
                prefix_before_skipfield_bits: prefix_bits,
                mantissa_bits,
                nonzero_bap_bins: bit_cost.nonzero_bap_bins(),
            },
        })
    }
}

/// Measures a D0 block from its BAP rows without quantizing coefficient mantissas.
///
/// The grouped field widths depend only on BAP codes and positions. This is the exact cheap
/// preflight used by the frame q search; the selected candidate is still passed through the real
/// mantissa quantizer and bit writer before frame assembly.
pub fn measure_flat7x_d0_audio_block_bit_cost(
    input: &Flat7xD0AudioBlockInput<'_>,
) -> Result<Flat7xD0AudioBlockBitCost, Flat7xD0AudioBlockError> {
    let control_q = input.frame_control_policy.allocation_candidate().value();
    let frame_q = input.bap_frame.candidate();
    if frame_q != control_q {
        return Err(Flat7xD0AudioBlockError::BapCandidateMismatch { frame_q, control_q });
    }

    let block = &input.bap_frame.blocks()[usize::from(input.block_index.value())];
    let mut bap_counts = [0_u32; 16];
    for bap in block.rows().iter().flatten().copied() {
        let count =
            bap_counts
                .get_mut(usize::from(bap))
                .ok_or(Flat7xD0AudioBlockError::Mantissa(
                    MantissaBlockError::Quantizer(
                        super::mantissa::MantissaQuantizerError::UnsupportedBap,
                    ),
                ))?;
        *count = count
            .checked_add(1)
            .ok_or(Flat7xD0AudioBlockError::BitCountOverflow)?;
    }

    let mut mantissa_bits = 0_u32;
    for (bap, count) in bap_counts.into_iter().enumerate() {
        if count == 0 || bap == 0 {
            continue;
        }
        let width = u32::from(MANTISSA_WIDTH_BY_BAP[bap]);
        let fields = match bap {
            1 | 2 => count.div_ceil(3),
            4 => count.div_ceil(2),
            _ => count,
        };
        mantissa_bits = mantissa_bits
            .checked_add(
                fields
                    .checked_mul(width)
                    .ok_or(Flat7xD0AudioBlockError::BitCountOverflow)?,
            )
            .ok_or(Flat7xD0AudioBlockError::BitCountOverflow)?;
    }
    let prefix_bits = measure_prefix_before_skipfield(input)?;
    let skip_flag_bits = u32::from(input.block_index.value() != 2);
    let bit_length = prefix_bits
        .checked_add(skip_flag_bits)
        .and_then(|value| value.checked_add(mantissa_bits))
        .ok_or(Flat7xD0AudioBlockError::BitCountOverflow)?;
    let nonzero_bap_bins = u16::try_from(
        block
            .rows()
            .iter()
            .flatten()
            .filter(|bap| **bap != 0)
            .count(),
    )
    .map_err(|_| Flat7xD0AudioBlockError::BitCountOverflow)?;

    Ok(Flat7xD0AudioBlockBitCost {
        bit_length,
        prefix_before_skipfield_bits: prefix_bits,
        mantissa_bits,
        nonzero_bap_bins,
    })
}

fn measure_prefix_before_skipfield(
    input: &Flat7xD0AudioBlockInput<'_>,
) -> Result<u32, Flat7xD0AudioBlockError> {
    let dynamic_range_bits = 1_u32
        .checked_add(u32::from(input.dynamic_range_code.is_some()) * 8)
        .ok_or(Flat7xD0AudioBlockError::BitCountOverflow)?;
    let per_channel_exponent_and_gain_bits = 4_u32
        .checked_add(
            u32::try_from(FLAT7X_D0_D15_GROUP_COUNT)
                .map_err(|_| Flat7xD0AudioBlockError::BitCountOverflow)?
                .checked_mul(7)
                .ok_or(Flat7xD0AudioBlockError::BitCountOverflow)?,
        )
        .and_then(|bits| bits.checked_add(2))
        .ok_or(Flat7xD0AudioBlockError::BitCountOverflow)?;
    let channel_bits = per_channel_exponent_and_gain_bits
        .checked_mul(
            u32::try_from(FLAT7X_D0_BAP_CHANNEL_COUNT)
                .map_err(|_| Flat7xD0AudioBlockError::BitCountOverflow)?,
        )
        .and_then(|bits| bits.checked_add(24)) // four six-bit chbwcod=60 fields
        .ok_or(Flat7xD0AudioBlockError::BitCountOverflow)?;
    let block_start_fields = 1_u32; // block-zero spxinu; later blocks carry spxstre.
    dynamic_range_bits
        .checked_add(4) // four dithflag=0 fields
        .and_then(|bits| bits.checked_add(block_start_fields))
        .and_then(|bits| bits.checked_add(channel_bits))
        .ok_or(Flat7xD0AudioBlockError::BitCountOverflow)
}

fn write_prefix_before_skipfield(
    writer: &mut FrameBitWriter<'_>,
    input: &Flat7xD0AudioBlockInput<'_>,
) -> Result<(), Flat7xD0AudioBlockError> {
    for _ in 0..FLAT7X_D0_BAP_CHANNEL_COUNT {
        writer.write_bit(false)?; // dithflag=0
    }
    writer.write_bit(input.dynamic_range_code.is_some())?;
    if let Some(code) = input.dynamic_range_code {
        writer.write_unsigned(u32::from(code), 8)?;
    }

    if input.block_index.value() == 0 {
        writer.write_bit(false)?; // spxinu=0; spxstre is implicitly set in block zero.
    } else {
        writer.write_bit(false)?; // spxstre=0; reuse the disabled block-zero SPX state.
    }

    for _ in 0..FLAT7X_D0_BAP_CHANNEL_COUNT {
        writer.write_unsigned(u32::from(FLAT7X_D0_CHANNEL_BANDWIDTH_CODE), 6)?;
    }
    for (channel, exponents) in input.exponent_rows.iter().enumerate() {
        write_flat7x_d0_d15_exponent_row(writer, exponents)?;
        writer.write_unsigned(u32::from(input.spectrum.gain_range_codes()[channel]), 2)?;
    }
    Ok(())
}

fn write_packed_mantissa_bits(
    writer: &mut FrameBitWriter<'_>,
    mantissas: &Flat7xD0MantissaBlock,
) -> Result<(), Flat7xD0AudioBlockError> {
    let bytes = mantissas.packed_bytes();
    for bit_offset in 0..mantissas.bit_count() {
        let byte_index = usize::try_from(bit_offset / 8)
            .map_err(|_| Flat7xD0AudioBlockError::BitCountOverflow)?;
        let bit_in_byte =
            u8::try_from(bit_offset % 8).map_err(|_| Flat7xD0AudioBlockError::BitCountOverflow)?;
        let byte = *bytes
            .get(byte_index)
            .ok_or(Flat7xD0AudioBlockError::BitCountOverflow)?;
        writer.write_bit(byte & (1_u8 << (7 - bit_in_byte)) != 0)?;
    }
    Ok(())
}

fn zeroed_bit_buffer(bit_count: u32) -> Result<Vec<u8>, Flat7xD0AudioBlockError> {
    let byte_count = usize::try_from(bit_count.div_ceil(8))
        .map_err(|_| Flat7xD0AudioBlockError::BitCountOverflow)?;
    Ok(vec![0; byte_count])
}
