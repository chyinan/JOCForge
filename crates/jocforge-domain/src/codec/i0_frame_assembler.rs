// pattern: Functional Core
//! Production-disabled Flat7x I0 AC-3 frame assembly from caller-owned syntax spans.

use super::{
    ac3_bsi_tail::{
        Flat7xI0Ac3BsiTailError, Flat7xI0Ac3BsiTailInput, Flat7xI0Ac3BsiTailReceipt,
        I0_BSI_TAIL_MINIMUM_BITS, write_flat7x_i0_ac3_bsi_tail,
    },
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
const I0_AC3_FULLBAND_CHANNEL_COUNT_BITS: u8 = 5;
const I0_CRC1_START_BIT: u32 = 16;
const I0_CRC_FIELD_BITS: u32 = 16;
const I0_CRC1_SUFFIX_START_BYTE: usize = 4;
const I0_CRC1_BOUNDARY_BYTE: usize = 1_600;
const I0_CRCRSV_FIELD_BITS: u32 = 1;
const I0_ERRORCHECK_FIELD_BITS: u32 = I0_CRCRSV_FIELD_BITS + I0_CRC_FIELD_BITS;

/// Explicit model values for the AC-3 audblk fields before coupling continuation.
///
/// Values may be projected from same-source F07/F12 typed values, but this input carries no
/// source receipt and asserts no native or DEE behavior. Its serializer stays production-disabled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct I0Ac3AudioBlockPrefixInput {
    /// AC-3 acmod 7 has five full-band channels in L, C, R, Ls, Rs order.
    pub block_switch_flags: [bool; 5],
    pub dither_flags: [bool; 5],
    /// The proposed AC-3 dynrng value; None emits dynrnge=0.
    pub dynamic_range_code: Option<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct I0Ac3AudioBlockFieldExtent {
    start: FrameBitCursor,
    bit_length: u8,
}

impl I0Ac3AudioBlockFieldExtent {
    pub const fn start(self) -> FrameBitCursor {
        self.start
    }

    pub const fn bit_length(self) -> u8 {
        self.bit_length
    }
}

/// Field extents through the mandatory first-block cplstre bit.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct I0Ac3AudioBlockPrefixReceipt {
    block_switch: I0Ac3AudioBlockFieldExtent,
    dither: I0Ac3AudioBlockFieldExtent,
    dynamic_range_exists: I0Ac3AudioBlockFieldExtent,
    dynamic_range: Option<I0Ac3AudioBlockFieldExtent>,
    coupling_strategy_exists: I0Ac3AudioBlockFieldExtent,
    next_field_cursor: FrameBitCursor,
}

impl I0Ac3AudioBlockPrefixReceipt {
    pub const fn block_switch(self) -> I0Ac3AudioBlockFieldExtent {
        self.block_switch
    }

    pub const fn dither(self) -> I0Ac3AudioBlockFieldExtent {
        self.dither
    }

    pub const fn dynamic_range_exists(self) -> I0Ac3AudioBlockFieldExtent {
        self.dynamic_range_exists
    }

    pub const fn dynamic_range(self) -> Option<I0Ac3AudioBlockFieldExtent> {
        self.dynamic_range
    }

    pub const fn coupling_strategy_exists(self) -> I0Ac3AudioBlockFieldExtent {
        self.coupling_strategy_exists
    }

    /// The next required AC-3 field is cplinu; it is outside this bounded prefix.
    pub const fn next_field_cursor(self) -> FrameBitCursor {
        self.next_field_cursor
    }

    pub const fn production_allowed(self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum I0Ac3AudioBlockPrefixError {
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for I0Ac3AudioBlockPrefixError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Writes the AC-3 first-audblk prefix from explicitly supplied model values.
///
/// ETSI TS 102 366 v1.4.1 clause 4.3.3 orders five blksw bits, five dithflag bits,
/// dynrnge and optional dynrng before cplstre. Clause 4.4.3.7 requires cplstre=1
/// in block 0. This serializer stops after that bit, before caller-owned cplinu.
/// It writes no exponents, bit allocation, mantissas, or source bytes.
pub fn write_i0_ac3_first_audio_block_prefix(
    writer: &mut FrameBitWriter<'_>,
    input: I0Ac3AudioBlockPrefixInput,
) -> Result<I0Ac3AudioBlockPrefixReceipt, I0Ac3AudioBlockPrefixError> {
    let dynamic_range_bits = if input.dynamic_range_code.is_some() {
        8_u32
    } else {
        0
    };
    let total_bits = u32::from(I0_AC3_FULLBAND_CHANNEL_COUNT_BITS)
        .checked_mul(2)
        .and_then(|value| value.checked_add(2))
        .ok_or(I0Ac3AudioBlockPrefixError::BitWriter(
            FrameBitWriterError::CursorOverflow,
        ))?
        .checked_add(dynamic_range_bits)
        .ok_or(I0Ac3AudioBlockPrefixError::BitWriter(
            FrameBitWriterError::CursorOverflow,
        ))?;
    writer.preflight_capacity(total_bits)?;

    let block_switch_start = writer.cursor();
    for value in input.block_switch_flags {
        writer.write_bit(value)?;
    }
    let dither_start = writer.cursor();
    for value in input.dither_flags {
        writer.write_bit(value)?;
    }
    let dynamic_range_exists_start = writer.cursor();
    writer.write_bit(input.dynamic_range_code.is_some())?;
    let dynamic_range = if let Some(value) = input.dynamic_range_code {
        let start = writer.cursor();
        writer.write_unsigned(u32::from(value), 8)?;
        Some(I0Ac3AudioBlockFieldExtent {
            start,
            bit_length: 8,
        })
    } else {
        None
    };
    let coupling_strategy_exists_start = writer.cursor();
    writer.write_bit(true)?;

    Ok(I0Ac3AudioBlockPrefixReceipt {
        block_switch: I0Ac3AudioBlockFieldExtent {
            start: block_switch_start,
            bit_length: I0_AC3_FULLBAND_CHANNEL_COUNT_BITS,
        },
        dither: I0Ac3AudioBlockFieldExtent {
            start: dither_start,
            bit_length: I0_AC3_FULLBAND_CHANNEL_COUNT_BITS,
        },
        dynamic_range_exists: I0Ac3AudioBlockFieldExtent {
            start: dynamic_range_exists_start,
            bit_length: 1,
        },
        dynamic_range,
        coupling_strategy_exists: I0Ac3AudioBlockFieldExtent {
            start: coupling_strategy_exists_start,
            bit_length: 1,
        },
        next_field_cursor: writer.cursor(),
    })
}

/// Caller-owned value for the one-bit AC-3 cplinu field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallerOwnedAc3Cplinu {
    Independent,
    Coupled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ac3CplinuValueError {
    OutOfRange { value: u8 },
}

impl TryFrom<u8> for CallerOwnedAc3Cplinu {
    type Error = Ac3CplinuValueError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Independent),
            1 => Ok(Self::Coupled),
            value => Err(Ac3CplinuValueError::OutOfRange { value }),
        }
    }
}

/// The emitted cplinu bit and its immediately following caller cursor.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct I0Ac3CplinuReceipt {
    value: CallerOwnedAc3Cplinu,
    field: I0Ac3AudioBlockFieldExtent,
    next_field_cursor: FrameBitCursor,
}

impl I0Ac3CplinuReceipt {
    pub const fn value(self) -> CallerOwnedAc3Cplinu {
        self.value
    }

    pub const fn field(self) -> I0Ac3AudioBlockFieldExtent {
        self.field
    }

    pub const fn next_field_cursor(self) -> FrameBitCursor {
        self.next_field_cursor
    }

    pub const fn production_allowed(self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum I0Ac3CplinuWriteError {
    PrefixCursorMismatch {
        expected: FrameBitCursor,
        actual: FrameBitCursor,
    },
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for I0Ac3CplinuWriteError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Writes the caller-owned AC-3 cplinu choice immediately after the block-0 cplstre prefix.
///
/// ETSI TS 102 366 V1.4.1 clause 4.4.3.8 assigns 0 to independent channels and 1 to coupling
/// in use. Any coupling parameters that follow remain caller-owned. This serializer chooses no
/// coupling mode and is not connected to an audio producer or production frame path.
pub fn write_i0_ac3_first_block_cplinu(
    writer: &mut FrameBitWriter<'_>,
    prefix: I0Ac3AudioBlockPrefixReceipt,
    input: CallerOwnedAc3Cplinu,
) -> Result<I0Ac3CplinuReceipt, I0Ac3CplinuWriteError> {
    let actual = writer.cursor();
    let expected = prefix.next_field_cursor();
    if actual != expected {
        return Err(I0Ac3CplinuWriteError::PrefixCursorMismatch { expected, actual });
    }
    writer.preflight_capacity(1)?;
    let field_start = writer.cursor();
    writer.write_bit(matches!(input, CallerOwnedAc3Cplinu::Coupled))?;
    Ok(I0Ac3CplinuReceipt {
        value: input,
        field: I0Ac3AudioBlockFieldExtent {
            start: field_start,
            bit_length: 1,
        },
        next_field_cursor: writer.cursor(),
    })
}

/// Caller-authored coupling fields following cplinu=1 for the selected I0 acmod 7.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct I0Ac3CouplingFieldsInput<'a> {
    /// Five full-band channel flags in L, C, R, Ls, Rs order.
    pub channel_in_coupling: [bool; 5],
    /// Four-bit AC-3 sub-band code, in the range 0..=15.
    pub cplbegf: u8,
    /// Four-bit AC-3 end-frequency code, in the range 0..=15.
    pub cplendf: u8,
    /// Caller-supplied cplbndstrc[1..ncplsubnd]; element 0 is implicit and omitted.
    pub band_structure_bits: &'a [bool],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum I0Ac3CouplingFrequencyField {
    Begin,
    End,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct I0Ac3CouplingFieldsReceipt {
    channel_in_coupling: I0Ac3AudioBlockFieldExtent,
    cplbegf: I0Ac3AudioBlockFieldExtent,
    cplendf: I0Ac3AudioBlockFieldExtent,
    band_structure: Option<I0Ac3AudioBlockFieldExtent>,
    active_subband_count: u8,
    next_field_cursor: FrameBitCursor,
}

impl I0Ac3CouplingFieldsReceipt {
    pub const fn channel_in_coupling(self) -> I0Ac3AudioBlockFieldExtent {
        self.channel_in_coupling
    }

    pub const fn cplbegf(self) -> I0Ac3AudioBlockFieldExtent {
        self.cplbegf
    }

    pub const fn cplendf(self) -> I0Ac3AudioBlockFieldExtent {
        self.cplendf
    }

    pub const fn band_structure(self) -> Option<I0Ac3AudioBlockFieldExtent> {
        self.band_structure
    }

    pub const fn active_subband_count(self) -> u8 {
        self.active_subband_count
    }

    /// The next AC-3 coupling fields, beginning with cplcoe[ch], remain caller-owned.
    pub const fn next_field_cursor(self) -> FrameBitCursor {
        self.next_field_cursor
    }

    pub const fn production_allowed(self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum I0Ac3CouplingFieldsError {
    CouplingNotInUse,
    PrefixCursorMismatch {
        expected: FrameBitCursor,
        actual: FrameBitCursor,
    },
    FewerThanTwoChannelsSelected,
    FrequencyCodeOutOfRange {
        field: I0Ac3CouplingFrequencyField,
        value: u8,
    },
    ActiveSubbandCountOutOfRange {
        value: i16,
    },
    BandStructureLengthMismatch {
        expected: u8,
        actual: usize,
    },
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for I0Ac3CouplingFieldsError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Writes the caller-owned minimum coupling-strategy fields after a block-0 cplinu=1.
///
/// ETSI TS 102 366 V1.4.1 clause 4.3.3 orders chincpl[5], cplbegf, cplendf and then
/// cplbndstrc[1..ncplsubnd]. For I0 acmod 7, phsflginu is absent. The active count is
/// the standard expression 3 + cplendf - cplbegf and must yield 1..=18. The writer
/// serializes only caller-supplied choices and stops before coupling-coordinate fields.
pub fn write_i0_ac3_first_block_coupling_fields(
    writer: &mut FrameBitWriter<'_>,
    cplinu: I0Ac3CplinuReceipt,
    input: I0Ac3CouplingFieldsInput<'_>,
) -> Result<I0Ac3CouplingFieldsReceipt, I0Ac3CouplingFieldsError> {
    if cplinu.value() != CallerOwnedAc3Cplinu::Coupled {
        return Err(I0Ac3CouplingFieldsError::CouplingNotInUse);
    }
    let actual_cursor = writer.cursor();
    let expected_cursor = cplinu.next_field_cursor();
    if actual_cursor != expected_cursor {
        return Err(I0Ac3CouplingFieldsError::PrefixCursorMismatch {
            expected: expected_cursor,
            actual: actual_cursor,
        });
    }
    if input
        .channel_in_coupling
        .iter()
        .filter(|flag| **flag)
        .count()
        < 2
    {
        return Err(I0Ac3CouplingFieldsError::FewerThanTwoChannelsSelected);
    }
    for (field, value) in [
        (I0Ac3CouplingFrequencyField::Begin, input.cplbegf),
        (I0Ac3CouplingFrequencyField::End, input.cplendf),
    ] {
        if value > 15 {
            return Err(I0Ac3CouplingFieldsError::FrequencyCodeOutOfRange { field, value });
        }
    }
    let active_subband_count = 3_i16 + i16::from(input.cplendf) - i16::from(input.cplbegf);
    if !(1..=18).contains(&active_subband_count) {
        return Err(I0Ac3CouplingFieldsError::ActiveSubbandCountOutOfRange {
            value: active_subband_count,
        });
    }
    let active_subband_count = u8::try_from(active_subband_count).map_err(|_| {
        I0Ac3CouplingFieldsError::ActiveSubbandCountOutOfRange {
            value: active_subband_count,
        }
    })?;
    let expected_band_bits = active_subband_count - 1;
    if input.band_structure_bits.len() != usize::from(expected_band_bits) {
        return Err(I0Ac3CouplingFieldsError::BandStructureLengthMismatch {
            expected: expected_band_bits,
            actual: input.band_structure_bits.len(),
        });
    }
    let total_bits = 13_u32 + u32::from(expected_band_bits);
    writer.preflight_capacity(total_bits)?;

    let channel_start = writer.cursor();
    for included in input.channel_in_coupling {
        writer.write_bit(included)?;
    }
    let cplbegf_start = writer.cursor();
    writer.write_unsigned(u32::from(input.cplbegf), 4)?;
    let cplendf_start = writer.cursor();
    writer.write_unsigned(u32::from(input.cplendf), 4)?;
    let band_structure = if expected_band_bits == 0 {
        None
    } else {
        let start = writer.cursor();
        for value in input.band_structure_bits {
            writer.write_bit(*value)?;
        }
        Some(I0Ac3AudioBlockFieldExtent {
            start,
            bit_length: expected_band_bits,
        })
    };

    Ok(I0Ac3CouplingFieldsReceipt {
        channel_in_coupling: I0Ac3AudioBlockFieldExtent {
            start: channel_start,
            bit_length: 5,
        },
        cplbegf: I0Ac3AudioBlockFieldExtent {
            start: cplbegf_start,
            bit_length: 4,
        },
        cplendf: I0Ac3AudioBlockFieldExtent {
            start: cplendf_start,
            bit_length: 4,
        },
        band_structure,
        active_subband_count,
        next_field_cursor: writer.cursor(),
    })
}

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

/// Typed caller input for frame assembly with the approved BSI-tail field writer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xI0Ac3FrameInput<'a> {
    pub header_fields: Flat7xI0Ac3FrameHeaderFields,
    pub bsi_tail: Flat7xI0Ac3BsiTailInput<'a>,
    pub audio_blocks: [OpaqueI0Ac3BitSpan<'a>; I0_AUDIO_BLOCK_COUNT],
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

/// Receipt from typed BSI-field serialization followed by whole-frame assembly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xI0Ac3FrameAssemblyReceipt {
    frame: Flat7xI0FrameAssemblyReceipt,
    bsi_tail: Flat7xI0Ac3BsiTailReceipt,
}

impl Flat7xI0Ac3FrameAssemblyReceipt {
    pub const fn frame(self) -> Flat7xI0FrameAssemblyReceipt {
        self.frame
    }

    pub const fn bsi_tail(self) -> Flat7xI0Ac3BsiTailReceipt {
        self.bsi_tail
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
    BsiTail(Flat7xI0Ac3BsiTailError),
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

impl From<Flat7xI0Ac3BsiTailError> for Flat7xI0FrameAssemblyError {
    fn from(error: Flat7xI0Ac3BsiTailError) -> Self {
        Self::BsiTail(error)
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
/// Serializes typed caller-owned BSI values, then assembles the complete I0 frame.
///
/// The lower-level bit-span API remains available for callers that already have a
/// serialized BSI tail. This entry preflights the caller frame writer, writes the
/// BSI fields into private scratch, then delegates to the same whole-frame staging
/// and CRC path. The audio-block spans remain opaque caller inputs.
pub fn assemble_flat7x_i0_ac3_frame_from_fields(
    target: Flat7xI0D0TargetPlan,
    input: Flat7xI0Ac3FrameInput<'_>,
    writer: &mut FrameBitWriter<'_>,
) -> Result<Flat7xI0Ac3FrameAssemblyReceipt, Flat7xI0FrameAssemblyError> {
    validate_target(target)?;
    let actual_cursor = writer.cursor().bit_offset();
    if actual_cursor != 0 {
        return Err(Flat7xI0FrameAssemblyError::CursorMismatch {
            actual: actual_cursor,
        });
    }
    writer.preflight_capacity(I0_TARGET_FRAME_BITS)?;

    let header_end_bit = if input.header_fields.compr.is_some() {
        70
    } else {
        62
    };
    let mut bsi_scratch = [0_u8; I0_TARGET_FRAME_BYTES];
    let bsi_receipt = {
        let mut bsi_writer =
            FrameBitWriter::new(&mut bsi_scratch, FrameBitCursor::new(header_end_bit))?;
        write_flat7x_i0_ac3_bsi_tail(&mut bsi_writer, input.bsi_tail)?
    };
    let bsi_source = OpaqueI0Ac3BitSpan::new(
        &bsi_scratch,
        bsi_receipt.start_bit(),
        bsi_receipt.bit_length(),
    );
    let frame_receipt = assemble_flat7x_i0_ac3_frame(
        target,
        Flat7xI0FrameBitInput {
            header_fields: input.header_fields,
            bsi_tail: bsi_source,
            audio_blocks: input.audio_blocks,
            auxdata: input.auxdata,
            crcrsv_seed: input.crcrsv_seed,
        },
        writer,
    )?;
    Ok(Flat7xI0Ac3FrameAssemblyReceipt {
        frame: frame_receipt,
        bsi_tail: bsi_receipt,
    })
}

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
