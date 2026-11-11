// pattern: Functional Core
//! Production-disabled Flat7x I0 AC-3 frame assembly from caller-owned syntax spans.

use super::{
    ac3_bsi_tail::I0_BSI_TAIL_MINIMUM_BITS,
    bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
    frame_builder::{
        Flat7xI0Ac3HeaderSegmentError, Flat7xI0Ac3HeaderValues,
        write_flat7x_i0_ac3_header_through_compression,
    },
};
use crate::{
    integrity::{
        ac3_crc16::{
            Ac3Crc16GenerationError, generate_ac3_crc1_from_suffix, generate_ac3_crc2_for_frame,
        },
        crc16::eac3_crc16,
    },
    profile::{Flat7xI0D0TargetPlan, Flat7xSubstreamRole},
};

const I0_TARGET_FRAME_BYTES: usize = 2_560;
const I0_TARGET_FRAME_BITS: u32 = 20_480;
const I0_AUDIO_BLOCK_COUNT: usize = 6;
const I0_CRC1_START_BIT: u32 = 16;
const I0_CRC_FIELD_BITS: u32 = 16;
const I0_CRC1_SUFFIX_START_BYTE: usize = 4;
const I0_CRC1_BOUNDARY_BYTE: usize = 1_600;
const I0_CRCRSV_FIELD_BITS: u32 = 1;
const I0_ERRORCHECK_FIELD_BITS: u32 = I0_CRCRSV_FIELD_BITS + I0_CRC_FIELD_BITS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpaqueI0Ac3BitSpan<'a> {
    bytes: &'a [u8],
    source_start_bit: u32,
    bit_length: u32,
}

impl<'a> OpaqueI0Ac3BitSpan<'a> {
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
pub enum I0AuxDataInput<'a> {
    Absent,
    Present(OpaqueI0Ac3BitSpan<'a>),
}

impl I0AuxDataInput<'_> {
    const fn payload_bit_length(self) -> u32 {
        match self {
            Self::Absent => 0,
            Self::Present(span) => span.bit_length(),
        }
    }

    const fn is_present(self) -> bool {
        matches!(self, Self::Present(_))
    }
}

/// Caller-owned initial value for the AC-3 `crcrsv` bit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CallerOwnedI0CrcrsvSeed {
    pub value: bool,
}

/// Variable fields in the approved I0 header segment, excluding generated CRC1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xI0Ac3FrameHeaderFields {
    pub frmsizecod: u8,
    pub bsmod: u8,
    pub cmixlev: u8,
    pub surmixlev: u8,
    pub dialnorm: u8,
    pub compr: Option<u8>,
}

impl Flat7xI0Ac3FrameHeaderFields {
    const fn with_crc1_placeholder(self) -> Flat7xI0Ac3HeaderValues {
        Flat7xI0Ac3HeaderValues {
            crc1: 0,
            frmsizecod: self.frmsizecod,
            bsmod: self.bsmod,
            cmixlev: self.cmixlev,
            surmixlev: self.surmixlev,
            dialnorm: self.dialnorm,
            compr: self.compr,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xI0FrameBitInput<'a> {
    pub header_fields: Flat7xI0Ac3FrameHeaderFields,
    /// Opaque AC-3 BSI bits from `langcode` through the end of BSI.
    pub bsi_tail: OpaqueI0Ac3BitSpan<'a>,
    /// Six caller-owned spans in AC-3 `audblk[0]` through `audblk[5]` order.
    pub audio_blocks: [OpaqueI0Ac3BitSpan<'a>; I0_AUDIO_BLOCK_COUNT],
    /// Opaque serialized `auxbits` and optional `auxdatal`; the assembler emits `auxdatae`.
    pub auxdata: I0AuxDataInput<'a>,
    pub crcrsv_seed: CallerOwnedI0CrcrsvSeed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct I0FrameBitSpan {
    start_bit: u32,
    bit_length: u32,
}

impl I0FrameBitSpan {
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
pub enum I0CrcValidationStatus {
    GeneratedAndResidualVerified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct I0CrcFieldReceipt {
    value: u16,
    span: I0FrameBitSpan,
    validation_status: I0CrcValidationStatus,
    residual: u16,
}

impl I0CrcFieldReceipt {
    pub const fn value(self) -> u16 {
        self.value
    }

    pub const fn span(self) -> I0FrameBitSpan {
        self.span
    }

    pub const fn validation_status(self) -> I0CrcValidationStatus {
        self.validation_status
    }

    pub const fn residual(self) -> u16 {
        self.residual
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct I0FrameIntegrityReceipts {
    crc1: I0CrcFieldReceipt,
    crc2: I0CrcFieldReceipt,
    crcrsv_initial_value: bool,
    crcrsv_value: bool,
    crcrsv_toggled: bool,
    crcrsv_span: I0FrameBitSpan,
}

impl I0FrameIntegrityReceipts {
    pub const fn crc1(self) -> I0CrcFieldReceipt {
        self.crc1
    }

    pub const fn crc2(self) -> I0CrcFieldReceipt {
        self.crc2
    }

    pub const fn crcrsv_value(self) -> bool {
        self.crcrsv_value
    }

    pub const fn crcrsv_initial_value(self) -> bool {
        self.crcrsv_initial_value
    }

    pub const fn crcrsv_toggled(self) -> bool {
        self.crcrsv_toggled
    }

    pub const fn crcrsv_span(self) -> I0FrameBitSpan {
        self.crcrsv_span
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xI0FrameAssemblyReceipt {
    header_span: I0FrameBitSpan,
    bsi_tail_span: I0FrameBitSpan,
    audio_block_spans: [I0FrameBitSpan; I0_AUDIO_BLOCK_COUNT],
    auxdata_span: I0FrameBitSpan,
    errorcheck_span: I0FrameBitSpan,
    integrity_receipts: I0FrameIntegrityReceipts,
}

impl Flat7xI0FrameAssemblyReceipt {
    pub const fn header_span(self) -> I0FrameBitSpan {
        self.header_span
    }

    pub const fn bsi_tail_span(self) -> I0FrameBitSpan {
        self.bsi_tail_span
    }

    pub const fn audio_block_spans(self) -> [I0FrameBitSpan; I0_AUDIO_BLOCK_COUNT] {
        self.audio_block_spans
    }

    pub const fn auxdata_span(self) -> I0FrameBitSpan {
        self.auxdata_span
    }

    pub const fn errorcheck_span(self) -> I0FrameBitSpan {
        self.errorcheck_span
    }

    pub const fn integrity_receipts(self) -> I0FrameIntegrityReceipts {
        self.integrity_receipts
    }

    pub const fn target_frame_bytes(self) -> u16 {
        I0_TARGET_FRAME_BYTES as u16
    }

    pub const fn total_frame_span_bits(self) -> u32 {
        I0_TARGET_FRAME_BITS
    }

    pub const fn production_allowed(self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum I0FramePart {
    BsiTail,
    AudioBlock { index: u8 },
    AuxData,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xI0FrameAssemblyError {
    UnsupportedTargetContext,
    CursorMismatch {
        actual: u32,
    },
    BsiTailTooShort {
        actual_bits: u32,
        minimum_bits: u32,
    },
    SourceSpanOverflow {
        part: I0FramePart,
    },
    SourceSpanOutOfBounds {
        part: I0FramePart,
        required_end_bit: u32,
        available_source_bits: u64,
    },
    FrameSpanMismatch {
        actual_bits: u32,
        required_bits: u32,
    },
    Crc1ResidualNonzero {
        residual: u16,
    },
    CrcGeneration,
    HeaderSegment(Flat7xI0Ac3HeaderSegmentError),
    BitWriter(FrameBitWriterError),
}

impl From<Ac3Crc16GenerationError> for Flat7xI0FrameAssemblyError {
    fn from(_: Ac3Crc16GenerationError) -> Self {
        Self::CrcGeneration
    }
}

impl From<Flat7xI0Ac3HeaderSegmentError> for Flat7xI0FrameAssemblyError {
    fn from(error: Flat7xI0Ac3HeaderSegmentError) -> Self {
        Self::HeaderSegment(error)
    }
}

impl From<FrameBitWriterError> for Flat7xI0FrameAssemblyError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Assembles one selected Flat7x I0 frame from caller-owned, already encoded syntax spans.
///
/// The Wash contract fixes I0 frame parts as syncinfo, BSI, six ordered audio blocks,
/// auxdata and errorcheck. The BSI tail and audio block contents remain caller-owned.
/// CRC1/CRC2 are generated with the approved ETSI AC-3 rule after all frame parts
/// are staged. The caller supplies the initial `crcrsv` bit; the syncword-collision
/// adjustment is applied from the CRC contract. This does not generate audio.
pub fn assemble_flat7x_i0_ac3_frame(
    target: Flat7xI0D0TargetPlan,
    input: Flat7xI0FrameBitInput<'_>,
    writer: &mut FrameBitWriter<'_>,
) -> Result<Flat7xI0FrameAssemblyReceipt, Flat7xI0FrameAssemblyError> {
    validate_target(target)?;
    let actual_cursor = writer.cursor().bit_offset();
    if actual_cursor != 0 {
        return Err(Flat7xI0FrameAssemblyError::CursorMismatch {
            actual: actual_cursor,
        });
    }

    if input.bsi_tail.bit_length() < I0_BSI_TAIL_MINIMUM_BITS {
        return Err(Flat7xI0FrameAssemblyError::BsiTailTooShort {
            actual_bits: input.bsi_tail.bit_length(),
            minimum_bits: I0_BSI_TAIL_MINIMUM_BITS,
        });
    }
    validate_source_span(I0FramePart::BsiTail, input.bsi_tail)?;
    for (index, span) in input.audio_blocks.iter().copied().enumerate() {
        validate_source_span(
            I0FramePart::AudioBlock {
                index: u8::try_from(index).unwrap_or(u8::MAX),
            },
            span,
        )?;
    }
    if let I0AuxDataInput::Present(span) = input.auxdata {
        validate_source_span(I0FramePart::AuxData, span)?;
    }

    let header_bits: u32 = if input.header_fields.compr.is_some() {
        70
    } else {
        62
    };
    let audio_bits = input
        .audio_blocks
        .iter()
        .try_fold(0_u32, |total, span| total.checked_add(span.bit_length()))
        .ok_or(Flat7xI0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: u32::MAX,
            required_bits: I0_TARGET_FRAME_BITS,
        })?;
    let bsi_end_bit = header_bits.checked_add(input.bsi_tail.bit_length()).ok_or(
        Flat7xI0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: u32::MAX,
            required_bits: I0_TARGET_FRAME_BITS,
        },
    )?;
    let aux_payload_bits = input.auxdata.payload_bit_length();
    let aux_span_bits =
        aux_payload_bits
            .checked_add(1)
            .ok_or(Flat7xI0FrameAssemblyError::FrameSpanMismatch {
                actual_bits: u32::MAX,
                required_bits: I0_TARGET_FRAME_BITS,
            })?;
    let total_bits = bsi_end_bit
        .checked_add(audio_bits)
        .and_then(|value| value.checked_add(aux_span_bits))
        .and_then(|value| value.checked_add(I0_ERRORCHECK_FIELD_BITS))
        .ok_or(Flat7xI0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: u32::MAX,
            required_bits: I0_TARGET_FRAME_BITS,
        })?;
    if total_bits != I0_TARGET_FRAME_BITS {
        return Err(Flat7xI0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: total_bits,
            required_bits: I0_TARGET_FRAME_BITS,
        });
    }

    writer.preflight_capacity(I0_TARGET_FRAME_BITS)?;
    let mut staging = [0_u8; I0_TARGET_FRAME_BYTES];
    let header_end = {
        let mut header_writer = FrameBitWriter::new(&mut staging, FrameBitCursor::new(0))?;
        write_flat7x_i0_ac3_header_through_compression(
            &mut header_writer,
            input.header_fields.with_crc1_placeholder(),
        )?
        .bit_offset()
    };
    if header_end != header_bits {
        return Err(Flat7xI0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: header_end,
            required_bits: header_bits,
        });
    }

    let bsi_tail_span = copy_i0_span(
        &mut staging,
        header_end,
        I0FramePart::BsiTail,
        input.bsi_tail,
    )?;
    let mut frame_cursor = bsi_tail_span.end_bit();
    let mut audio_block_spans = [I0FrameBitSpan {
        start_bit: 0,
        bit_length: 0,
    }; I0_AUDIO_BLOCK_COUNT];
    for (index, source) in input.audio_blocks.iter().copied().enumerate() {
        let part = I0FramePart::AudioBlock {
            index: u8::try_from(index).unwrap_or(u8::MAX),
        };
        audio_block_spans[index] = copy_i0_span(&mut staging, frame_cursor, part, source)?;
        frame_cursor = audio_block_spans[index].end_bit();
    }

    let aux_payload_span = match input.auxdata {
        I0AuxDataInput::Absent => None,
        I0AuxDataInput::Present(span) => Some(copy_i0_span(
            &mut staging,
            frame_cursor,
            I0FramePart::AuxData,
            span,
        )?),
    };
    frame_cursor = frame_cursor.checked_add(aux_payload_bits).ok_or(
        Flat7xI0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: u32::MAX,
            required_bits: I0_TARGET_FRAME_BITS,
        },
    )?;
    let auxdatae_start_bit = frame_cursor;
    let auxdatae_end_bit = {
        let mut auxdatae_writer =
            FrameBitWriter::new(&mut staging, FrameBitCursor::new(auxdatae_start_bit))?;
        auxdatae_writer.write_bit(input.auxdata.is_present())?;
        auxdatae_writer.cursor().bit_offset()
    };
    if auxdatae_end_bit != auxdatae_start_bit + 1 {
        return Err(Flat7xI0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: auxdatae_end_bit,
            required_bits: auxdatae_start_bit + 1,
        });
    }
    let auxdata_span = I0FrameBitSpan {
        start_bit: aux_payload_span.map_or(auxdatae_start_bit, I0FrameBitSpan::start_bit),
        bit_length: aux_span_bits,
    };
    frame_cursor = auxdatae_end_bit;

    let errorcheck_start_bit = frame_cursor;
    if errorcheck_start_bit != I0_TARGET_FRAME_BITS - I0_ERRORCHECK_FIELD_BITS {
        return Err(Flat7xI0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: errorcheck_start_bit + I0_ERRORCHECK_FIELD_BITS,
            required_bits: I0_TARGET_FRAME_BITS,
        });
    }
    let crcrsv_start_bit = frame_cursor;
    {
        let mut crcrsv_writer =
            FrameBitWriter::new(&mut staging, FrameBitCursor::new(crcrsv_start_bit))?;
        crcrsv_writer.write_bit(input.crcrsv_seed.value)?;
    }

    let crc1_generation =
        generate_ac3_crc1_from_suffix(&staging[I0_CRC1_SUFFIX_START_BYTE..I0_CRC1_BOUNDARY_BYTE])?;
    {
        let mut crc1_writer =
            FrameBitWriter::new(&mut staging, FrameBitCursor::new(I0_CRC1_START_BIT))?;
        crc1_writer.write_unsigned(u32::from(crc1_generation.value()), I0_CRC_FIELD_BITS as u8)?;
    }
    let crc1_residual = eac3_crc16(&staging[2..I0_CRC1_BOUNDARY_BYTE]);
    if crc1_residual != 0 {
        return Err(Flat7xI0FrameAssemblyError::Crc1ResidualNonzero {
            residual: crc1_residual,
        });
    }

    let crc2_generation = generate_ac3_crc2_for_frame(&mut staging)?;
    let crc2_start_bit = u32::from(crc2_generation.field_start_byte()) * 8;
    let errorcheck_end_bit = crc2_start_bit + I0_CRC_FIELD_BITS;
    if errorcheck_end_bit != I0_TARGET_FRAME_BITS
        || crc2_start_bit != I0_TARGET_FRAME_BITS - I0_CRC_FIELD_BITS
    {
        return Err(Flat7xI0FrameAssemblyError::FrameSpanMismatch {
            actual_bits: errorcheck_end_bit,
            required_bits: I0_TARGET_FRAME_BITS,
        });
    }

    writer.write_bytes(&staging)?;
    Ok(Flat7xI0FrameAssemblyReceipt {
        header_span: I0FrameBitSpan {
            start_bit: 0,
            bit_length: header_end,
        },
        bsi_tail_span,
        audio_block_spans,
        auxdata_span,
        errorcheck_span: I0FrameBitSpan {
            start_bit: errorcheck_start_bit,
            bit_length: I0_ERRORCHECK_FIELD_BITS,
        },
        integrity_receipts: I0FrameIntegrityReceipts {
            crc1: I0CrcFieldReceipt {
                value: crc1_generation.value(),
                span: I0FrameBitSpan {
                    start_bit: I0_CRC1_START_BIT,
                    bit_length: I0_CRC_FIELD_BITS,
                },
                validation_status: I0CrcValidationStatus::GeneratedAndResidualVerified,
                residual: crc1_residual,
            },
            crc2: I0CrcFieldReceipt {
                value: crc2_generation.value(),
                span: I0FrameBitSpan {
                    start_bit: crc2_start_bit,
                    bit_length: I0_CRC_FIELD_BITS,
                },
                validation_status: I0CrcValidationStatus::GeneratedAndResidualVerified,
                residual: crc2_generation.residual(),
            },
            crcrsv_initial_value: crc2_generation.crcrsv_before(),
            crcrsv_value: crc2_generation.crcrsv_after(),
            crcrsv_toggled: crc2_generation.crcrsv_toggled(),
            crcrsv_span: I0FrameBitSpan {
                start_bit: crc2_generation.crcrsv_bit_offset(),
                bit_length: I0_CRCRSV_FIELD_BITS,
            },
        },
    })
}

fn validate_target(target: Flat7xI0D0TargetPlan) -> Result<(), Flat7xI0FrameAssemblyError> {
    let i0 = target.i0();
    if target.sample_rate_hz() != 48_000
        || target.audio_blocks_per_frame() != I0_AUDIO_BLOCK_COUNT as u8
        || target.samples_per_frame() != 1_536
        || target.aggregate_rate_kbps() != Some(1_664)
        || i0.role() != Flat7xSubstreamRole::IndependentI0
        || i0.rate().kbps() != Some(640)
        || i0.substream_id().value() != 0
        || i0.acmod().value() != 7
        || !i0.lfeon()
        || i0.frame_bytes() as usize != I0_TARGET_FRAME_BYTES
        || target.d0().rate().kbps() != Some(1_024)
        || target.d0().frame_bytes() != 4_096
    {
        return Err(Flat7xI0FrameAssemblyError::UnsupportedTargetContext);
    }
    Ok(())
}

fn validate_source_span(
    part: I0FramePart,
    span: OpaqueI0Ac3BitSpan<'_>,
) -> Result<(), Flat7xI0FrameAssemblyError> {
    let required_end_bit = span
        .source_start_bit()
        .checked_add(span.bit_length())
        .ok_or(Flat7xI0FrameAssemblyError::SourceSpanOverflow { part })?;
    let available_source_bits = u64::try_from(span.bytes.len())
        .unwrap_or(u64::MAX)
        .saturating_mul(8);
    if u64::from(required_end_bit) > available_source_bits {
        return Err(Flat7xI0FrameAssemblyError::SourceSpanOutOfBounds {
            part,
            required_end_bit,
            available_source_bits,
        });
    }
    Ok(())
}

fn copy_i0_span(
    output: &mut [u8],
    destination_start_bit: u32,
    part: I0FramePart,
    source: OpaqueI0Ac3BitSpan<'_>,
) -> Result<I0FrameBitSpan, Flat7xI0FrameAssemblyError> {
    validate_source_span(part, source)?;
    let end_bit = destination_start_bit
        .checked_add(source.bit_length())
        .ok_or(Flat7xI0FrameAssemblyError::SourceSpanOverflow { part })?;
    let mut writer = FrameBitWriter::new(output, FrameBitCursor::new(destination_start_bit))?;
    writer.preflight_capacity(source.bit_length())?;
    for offset in 0..source.bit_length() {
        let source_bit = source
            .source_start_bit()
            .checked_add(offset)
            .ok_or(Flat7xI0FrameAssemblyError::SourceSpanOverflow { part })?;
        let source_byte = usize::try_from(source_bit / 8)
            .map_err(|_| Flat7xI0FrameAssemblyError::SourceSpanOverflow { part })?;
        let shift = 7 - source_bit % 8;
        writer.write_bit(source.bytes[source_byte] & (1_u8 << shift) != 0)?;
    }
    Ok(I0FrameBitSpan {
        start_bit: destination_start_bit,
        bit_length: end_bit - destination_start_bit,
    })
}
