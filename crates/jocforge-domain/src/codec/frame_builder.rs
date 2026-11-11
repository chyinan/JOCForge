// pattern: Functional Core
//! Boundary that assembles validated typed receipts. It cannot calculate upstream values.

use crate::codec::bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError};
use crate::codec::eac3_channel_map::write_flat7x_d0_chanmap_fields;
use crate::codec::frame_size::Eac3FrameSizeCode;
use crate::codec::frame_syntax::{
    CodecBlockIndex, SelectedBsiDrcFields, SelectedBsiMixingInformationValues,
    SelectedComplexityIndexTypeA,
};
use crate::profile::{DependentChannelMap, Flat7xI0D0TargetPlan, Flat7xSubstreamRole};
use crate::timeline::CodecSourceIndex;

const DIALNORM_BIT_OFFSET: u32 = 45;
const SELECTED_FIXED_PREFIX_BITS: u32 = 45;
const COMPLEXITY_INDEX_TYPE_A_BIT_OFFSET: u32 = 105;
const COMPLEXITY_INDEX_TYPE_A_WIDTH: u8 = 8;
const LINE_BLOCK_COUNT: u8 = 6;
const SELECTED_DRC_HEADER_BITS_WITHOUT_COMPR: u32 = 6;
const SELECTED_DRC_HEADER_BITS_WITH_COMPR: u32 = 14;
const SELECTED_MIXING_INFORMATION_BITS: u32 = 31;
const SELECTED_FIXED_PREFIX_FIELDS: [(u32, u8); 9] = [
    (0x0B77, 16),
    (0, 2),
    (0, 3),
    (0x07FF, 11),
    (0, 2),
    (3, 2),
    (7, 3),
    (1, 1),
    (16, 5),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedEac3PrefixError {
    CursorMismatch { actual: u32 },
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for SelectedEac3PrefixError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Writes the selected 48 kHz / 5.1 fixed E-AC-3 prefix with its admitted frame-size code.
///
/// The prefix ends at bit 44; the next caller-owned field begins at bit 45.
/// Three display-pad bits are not emitted.
pub fn write_selected_eac3_fixed_prefix(
    writer: &mut FrameBitWriter<'_>,
    frame_size_code: Eac3FrameSizeCode,
) -> Result<FrameBitCursor, SelectedEac3PrefixError> {
    let actual = writer.cursor().bit_offset();
    if actual != 0 {
        return Err(SelectedEac3PrefixError::CursorMismatch { actual });
    }
    writer.preflight_capacity(SELECTED_FIXED_PREFIX_BITS)?;
    for (index, (value, width)) in SELECTED_FIXED_PREFIX_FIELDS.into_iter().enumerate() {
        let value = if index == 3 {
            u32::from(frame_size_code.value())
        } else {
            value
        };
        writer.write_unsigned(value, width)?;
    }
    Ok(writer.cursor())
}

const FLAT7X_I0_AC3_SEGMENT_BITS_WITHOUT_COMPR: u32 = 62;
const FLAT7X_I0_AC3_SEGMENT_BITS_WITH_COMPR: u32 = 70;
const FLAT7X_I0_AC3_DIALNORM_WIDTH_BITS: u8 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xI0Ac3HeaderSegmentError {
    CursorMismatch { actual: u32 },
    InvalidFrameSizeCode { value: u8 },
    InvalidBsmod { value: u8 },
    InvalidCmixlev { value: u8 },
    InvalidSurmixlev { value: u8 },
    InvalidDialnorm { value: u8 },
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for Flat7xI0Ac3HeaderSegmentError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xI0Ac3HeaderValues {
    pub crc1: u16,
    pub frmsizecod: u8,
    pub bsmod: u8,
    pub cmixlev: u8,
    pub surmixlev: u8,
    pub dialnorm: u8,
    pub compr: Option<u8>,
}

/// Writes the selected Flat7x I0 AC-3 header segment through optional `compr`.
///
/// All non-fixed syntax values are caller supplied. This helper validates their
/// field domains, preflights the complete 62- or 70-bit segment, and returns at
/// the caller-owned `langcode` boundary. It derives no DRC mapping and computes
/// no CRC or audio payload.
pub fn write_flat7x_i0_ac3_header_through_compression(
    writer: &mut FrameBitWriter<'_>,
    values: Flat7xI0Ac3HeaderValues,
) -> Result<FrameBitCursor, Flat7xI0Ac3HeaderSegmentError> {
    let Flat7xI0Ac3HeaderValues {
        crc1,
        frmsizecod,
        bsmod,
        cmixlev,
        surmixlev,
        dialnorm,
        compr,
    } = values;
    let actual = writer.cursor().bit_offset();
    if actual != 0 {
        return Err(Flat7xI0Ac3HeaderSegmentError::CursorMismatch { actual });
    }
    if !matches!(frmsizecod, 36 | 37) {
        return Err(Flat7xI0Ac3HeaderSegmentError::InvalidFrameSizeCode { value: frmsizecod });
    }
    if bsmod > 7 {
        return Err(Flat7xI0Ac3HeaderSegmentError::InvalidBsmod { value: bsmod });
    }
    if cmixlev > 3 {
        return Err(Flat7xI0Ac3HeaderSegmentError::InvalidCmixlev { value: cmixlev });
    }
    if surmixlev > 3 {
        return Err(Flat7xI0Ac3HeaderSegmentError::InvalidSurmixlev { value: surmixlev });
    }
    if !(1..=31).contains(&dialnorm) {
        return Err(Flat7xI0Ac3HeaderSegmentError::InvalidDialnorm { value: dialnorm });
    }

    let required_bits = if compr.is_some() {
        FLAT7X_I0_AC3_SEGMENT_BITS_WITH_COMPR
    } else {
        FLAT7X_I0_AC3_SEGMENT_BITS_WITHOUT_COMPR
    };
    writer.preflight_capacity(required_bits)?;

    writer.write_unsigned(0x0b77, 16)?;
    writer.write_unsigned(u32::from(crc1), 16)?;
    writer.write_unsigned(0, 2)?;
    writer.write_unsigned(u32::from(frmsizecod), 6)?;
    writer.write_unsigned(6, 5)?;
    writer.write_unsigned(u32::from(bsmod), 3)?;
    writer.write_unsigned(7, 3)?;
    writer.write_unsigned(u32::from(cmixlev), 2)?;
    writer.write_unsigned(u32::from(surmixlev), 2)?;
    writer.write_bit(true)?;
    writer.write_unsigned(u32::from(dialnorm), FLAT7X_I0_AC3_DIALNORM_WIDTH_BITS)?;
    writer.write_bit(compr.is_some())?;
    if let Some(compr) = compr {
        writer.write_unsigned(u32::from(compr), 8)?;
    }
    Ok(writer.cursor())
}

const FLAT7X_D0_FIXED_PREFIX_BITS: u32 = 45;
const FLAT7X_D0_FIXED_PREFIX_FIELDS: [(u32, u8); 9] = [
    (0x0b77, 16),
    (1, 2),
    (0, 3),
    (2_047, 11),
    (0, 2),
    (3, 2),
    (5, 3),
    (0, 1),
    (16, 5),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0FixedPrefixError {
    CursorMismatch { actual: u32 },
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for Flat7xD0FixedPrefixError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Writes the 45-bit Flat7x dependent E-AC-3 D0 syncframe prefix.
///
/// The writer must be positioned at bit zero. The complete prefix is
/// preflighted before mutation; the caller owns `dialnorm` and subsequent BSI.
/// The later dependent `chanmape`/`chanmap` field pair is a separate BSI group.
pub fn write_flat7x_d0_fixed_prefix(
    writer: &mut FrameBitWriter<'_>,
) -> Result<FrameBitCursor, Flat7xD0FixedPrefixError> {
    let actual = writer.cursor().bit_offset();
    if actual != 0 {
        return Err(Flat7xD0FixedPrefixError::CursorMismatch { actual });
    }
    writer.preflight_capacity(FLAT7X_D0_FIXED_PREFIX_BITS)?;
    for (value, width) in FLAT7X_D0_FIXED_PREFIX_FIELDS {
        writer.write_unsigned(value, width)?;
    }
    Ok(writer.cursor())
}

const D0_DIALNORM_WIDTH_BITS: u8 = 5;
const D0_COMPRE_WIDTH_BITS: u8 = 1;
const D0_COMPR_WIDTH_BITS: u8 = 8;
const D0_BSI_THROUGH_CHANMAP_BITS_WITHOUT_COMPR: u32 = 68;
const D0_BSI_THROUGH_CHANMAP_BITS_WITH_COMPR: u32 = 76;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0BsiSegmentError {
    CursorMismatch { actual: u32 },
    DialnormOutOfRange { value: u8 },
    TargetChannelMapMissing,
    FixedPrefix(Flat7xD0FixedPrefixError),
    BitWriter(FrameBitWriterError),
}

impl From<Flat7xD0FixedPrefixError> for Flat7xD0BsiSegmentError {
    fn from(error: Flat7xD0FixedPrefixError) -> Self {
        Self::FixedPrefix(error)
    }
}

impl From<FrameBitWriterError> for Flat7xD0BsiSegmentError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Writes the selected Flat7x D0 BSI segment through the dependent chanmap pair.
///
/// `dialnorm` and optional `compr` are explicit caller inputs. This function
/// serializes those fields only; it does not map XML DRC profiles or derive a
/// compression value. It stops before caller-owned `mixmdate` and later fields.
pub fn write_flat7x_d0_bsi_through_chanmap(
    writer: &mut FrameBitWriter<'_>,
    dialnorm: u8,
    compr: Option<u8>,
) -> Result<FrameBitCursor, Flat7xD0BsiSegmentError> {
    let actual = writer.cursor().bit_offset();
    if actual != 0 {
        return Err(Flat7xD0BsiSegmentError::CursorMismatch { actual });
    }
    if !(1..=31).contains(&dialnorm) {
        return Err(Flat7xD0BsiSegmentError::DialnormOutOfRange { value: dialnorm });
    }
    let channel_map = Flat7xI0D0TargetPlan::approved_contract()
        .d0()
        .channel_map()
        .ok_or(Flat7xD0BsiSegmentError::TargetChannelMapMissing)?;
    let required_bits = if compr.is_some() {
        D0_BSI_THROUGH_CHANMAP_BITS_WITH_COMPR
    } else {
        D0_BSI_THROUGH_CHANMAP_BITS_WITHOUT_COMPR
    };
    writer.preflight_capacity(required_bits)?;

    write_flat7x_d0_fixed_prefix(writer)?;
    writer.write_unsigned(u32::from(dialnorm), D0_DIALNORM_WIDTH_BITS)?;
    writer.write_unsigned(u32::from(compr.is_some()), D0_COMPRE_WIDTH_BITS)?;
    if let Some(compr) = compr {
        writer.write_unsigned(u32::from(compr), D0_COMPR_WIDTH_BITS)?;
    }
    write_flat7x_d0_chanmap_fields(writer, channel_map)?;
    Ok(writer.cursor())
}

const D0_MIXMDO_WIDTH_BITS: u8 = 2;
const D0_MIX_LEVEL_WIDTH_BITS: u8 = 3;
const D0_MIX_LEVEL_VALUE_COUNT: u8 = 4;
const D0_BLOCK_MIX_COUNT: u8 = 6;
const D0_BLOCK_MIX_VALUE_WIDTH_BITS: u8 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0MixingMetadataValues {
    pub mixmdo: u8,
    pub ltrtcmixlev: u8,
    pub lorocmixlev: u8,
    pub ltrtsurmixlev: u8,
    pub lorosurmixlev: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0MixingMetadataError {
    ParentSegmentCursorMismatch { actual: u32 },
    UnsupportedTargetContext,
    MixmdoOutOfRange { value: u8 },
    MixLevelOutOfRange { value: u8 },
    BlockMixValueOutOfRange { block: usize, value: u8 },
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for Flat7xD0MixingMetadataError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Appends target D0 mixing metadata and `infomdate` after the parent BSI segment.
///
/// Mixing values, six optional block values, and `infomdate` are caller supplied.
/// Presence flags derive only from their corresponding `Option`s. The complete
/// variable-length append is validated and preflighted before any bit is written.
pub fn append_flat7x_d0_mixing_metadata_and_infomdate(
    writer: &mut FrameBitWriter<'_>,
    target: Flat7xI0D0TargetPlan,
    mixing: Option<Flat7xD0MixingMetadataValues>,
    block_mix: [Option<u8>; 6],
    infomdate: bool,
) -> Result<FrameBitCursor, Flat7xD0MixingMetadataError> {
    let actual = writer.cursor().bit_offset();
    if actual != D0_BSI_THROUGH_CHANMAP_BITS_WITHOUT_COMPR
        && actual != D0_BSI_THROUGH_CHANMAP_BITS_WITH_COMPR
    {
        return Err(Flat7xD0MixingMetadataError::ParentSegmentCursorMismatch { actual });
    }

    let d0 = target.d0();
    if d0.role() != Flat7xSubstreamRole::DependentD0
        || d0.acmod().value() != 5
        || d0.lfeon()
        || target.audio_blocks_per_frame() != D0_BLOCK_MIX_COUNT
    {
        return Err(Flat7xD0MixingMetadataError::UnsupportedTargetContext);
    }

    if let Some(mixing) = mixing {
        if mixing.mixmdo > 3 {
            return Err(Flat7xD0MixingMetadataError::MixmdoOutOfRange {
                value: mixing.mixmdo,
            });
        }
        for value in [
            mixing.ltrtcmixlev,
            mixing.lorocmixlev,
            mixing.ltrtsurmixlev,
            mixing.lorosurmixlev,
        ] {
            if value > 7 {
                return Err(Flat7xD0MixingMetadataError::MixLevelOutOfRange { value });
            }
        }
    }
    let mut present_block_values = 0_u32;
    for (block, value) in block_mix.iter().enumerate() {
        if let Some(value) = value {
            if *value > 31 {
                return Err(Flat7xD0MixingMetadataError::BlockMixValueOutOfRange {
                    block,
                    value: *value,
                });
            }
            present_block_values += 1;
        }
    }

    let mixing_value_bits = if mixing.is_some() {
        u32::from(D0_MIXMDO_WIDTH_BITS)
            + u32::from(D0_MIX_LEVEL_VALUE_COUNT) * u32::from(D0_MIX_LEVEL_WIDTH_BITS)
    } else {
        0
    };
    let block_info_present = present_block_values != 0;
    let block_value_bits = if block_info_present {
        u32::from(D0_BLOCK_MIX_COUNT)
            + present_block_values * u32::from(D0_BLOCK_MIX_VALUE_WIDTH_BITS)
    } else {
        0
    };
    let total_bits = 3_u32
        .checked_add(mixing_value_bits)
        .and_then(|bits| bits.checked_add(block_value_bits))
        .ok_or(FrameBitWriterError::CursorOverflow)?;
    writer.preflight_capacity(total_bits)?;

    writer.write_bit(mixing.is_some())?;
    if let Some(mixing) = mixing {
        writer.write_unsigned(u32::from(mixing.mixmdo), D0_MIXMDO_WIDTH_BITS)?;
        writer.write_unsigned(u32::from(mixing.ltrtcmixlev), D0_MIX_LEVEL_WIDTH_BITS)?;
        writer.write_unsigned(u32::from(mixing.lorocmixlev), D0_MIX_LEVEL_WIDTH_BITS)?;
        writer.write_unsigned(u32::from(mixing.ltrtsurmixlev), D0_MIX_LEVEL_WIDTH_BITS)?;
        writer.write_unsigned(u32::from(mixing.lorosurmixlev), D0_MIX_LEVEL_WIDTH_BITS)?;
    }

    writer.write_bit(block_info_present)?;
    if block_info_present {
        for value in block_mix {
            writer.write_bit(value.is_some())?;
            if let Some(value) = value {
                writer.write_unsigned(u32::from(value), D0_BLOCK_MIX_VALUE_WIDTH_BITS)?;
            }
        }
    }
    writer.write_bit(infomdate)?;
    Ok(writer.cursor())
}

const D0_BSMOD_WIDTH_BITS: u8 = 3;
const D0_COPYRIGHTB_WIDTH_BITS: u8 = 1;
const D0_ORIGBS_WIDTH_BITS: u8 = 1;
const D0_AUDPRODIE_WIDTH_BITS: u8 = 1;
const D0_MIXLEVEL_WIDTH_BITS: u8 = 5;
const D0_ROOMTYP_WIDTH_BITS: u8 = 2;
const D0_CONVTYP_WIDTH_BITS: u8 = 1;
const D0_SOURCEFSCOD_WIDTH_BITS: u8 = 1;
const D0_ADDBSIE_WIDTH_BITS: u8 = 1;
const D0_ADDBSIL_WIDTH_BITS: u8 = 6;
const D0_MAX_ADDBSI_BYTES: usize = 64;
const D0_BSI_STRUCTURE_STAGING_BYTES: usize = 83;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0AudioProductionMetadata {
    pub mixlevel: u8,
    pub roomtyp: u8,
    pub convtyp: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0InformationMetadata {
    pub bsmod: u8,
    pub copyrightb: bool,
    pub origbs: bool,
    pub audio_production: Option<Flat7xD0AudioProductionMetadata>,
    pub sourcefscod: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0BsiStructure<'a> {
    pub dialnorm: u8,
    pub compr: Option<u8>,
    pub mixing: Option<Flat7xD0MixingMetadataValues>,
    pub block_mix: [Option<u8>; 6],
    pub information: Option<Flat7xD0InformationMetadata>,
    pub addbsi: Option<&'a [u8]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0BsiStructureError {
    CursorMismatch { actual: u32 },
    UnsupportedTargetContext,
    BsiSegment(Flat7xD0BsiSegmentError),
    MixingMetadata(Flat7xD0MixingMetadataError),
    InfoAddbsi(Flat7xD0InfoAddbsiError),
    BitWriter(FrameBitWriterError),
}

impl From<Flat7xD0BsiSegmentError> for Flat7xD0BsiStructureError {
    fn from(error: Flat7xD0BsiSegmentError) -> Self {
        Self::BsiSegment(error)
    }
}

impl From<Flat7xD0MixingMetadataError> for Flat7xD0BsiStructureError {
    fn from(error: Flat7xD0MixingMetadataError) -> Self {
        Self::MixingMetadata(error)
    }
}

impl From<Flat7xD0InfoAddbsiError> for Flat7xD0BsiStructureError {
    fn from(error: Flat7xD0InfoAddbsiError) -> Self {
        Self::InfoAddbsi(error)
    }
}

impl From<FrameBitWriterError> for Flat7xD0BsiStructureError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0InfoAddbsiError {
    InfomdateInputMismatch {
        infomdate: bool,
        information_present: bool,
    },
    UnsupportedTargetContext,
    FieldOutOfRange {
        field: &'static str,
        value: u8,
    },
    AddbsiLengthOutOfRange {
        bytes: usize,
    },
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for Flat7xD0InfoAddbsiError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Appends caller-supplied D0 informational BSI and opaque addbsi bytes.
///
/// The parent mixing/infomdate writer has already emitted `infomdate`; this
/// helper verifies its matching value, writes the conditional suffix, and
/// preflights the complete append before changing the frame scratch. It does
/// not interpret addbsi content or assemble audio, a complete frame, or an AU.
pub fn append_flat7x_d0_info_and_addbsi(
    writer: &mut FrameBitWriter<'_>,
    target: Flat7xI0D0TargetPlan,
    infomdate: bool,
    information: Option<Flat7xD0InformationMetadata>,
    addbsi: Option<&[u8]>,
) -> Result<FrameBitCursor, Flat7xD0InfoAddbsiError> {
    if infomdate != information.is_some() {
        return Err(Flat7xD0InfoAddbsiError::InfomdateInputMismatch {
            infomdate,
            information_present: information.is_some(),
        });
    }

    let d0 = target.d0();
    if d0.role() != Flat7xSubstreamRole::DependentD0
        || d0.acmod().value() != 5
        || d0.lfeon()
        || d0.channel_map() != Some(DependentChannelMap::from_numeric_value(0x1a00))
        || target.sample_rate_hz() != 48_000
        || target.audio_blocks_per_frame() != D0_BLOCK_MIX_COUNT
    {
        return Err(Flat7xD0InfoAddbsiError::UnsupportedTargetContext);
    }

    if let Some(information) = information {
        for (field, value, maximum) in [
            ("bsmod", information.bsmod, 7),
            ("sourcefscod", information.sourcefscod, 1),
        ] {
            if value > maximum {
                return Err(Flat7xD0InfoAddbsiError::FieldOutOfRange { field, value });
            }
        }
        if let Some(audio_production) = information.audio_production {
            for (field, value, maximum) in [
                ("mixlevel", audio_production.mixlevel, 31),
                ("roomtyp", audio_production.roomtyp, 3),
                ("convtyp", audio_production.convtyp, 1),
            ] {
                if value > maximum {
                    return Err(Flat7xD0InfoAddbsiError::FieldOutOfRange { field, value });
                }
            }
        }
    }

    if let Some(bytes) = addbsi {
        if !(1..=D0_MAX_ADDBSI_BYTES).contains(&bytes.len()) {
            return Err(Flat7xD0InfoAddbsiError::AddbsiLengthOutOfRange { bytes: bytes.len() });
        }
    }

    let information_bits = if let Some(information) = information {
        let base_bits = u32::from(D0_BSMOD_WIDTH_BITS)
            + u32::from(D0_COPYRIGHTB_WIDTH_BITS)
            + u32::from(D0_ORIGBS_WIDTH_BITS)
            + u32::from(D0_AUDPRODIE_WIDTH_BITS)
            + u32::from(D0_SOURCEFSCOD_WIDTH_BITS);
        let audio_production_bits = if information.audio_production.is_some() {
            u32::from(D0_MIXLEVEL_WIDTH_BITS)
                + u32::from(D0_ROOMTYP_WIDTH_BITS)
                + u32::from(D0_CONVTYP_WIDTH_BITS)
        } else {
            0
        };
        base_bits + audio_production_bits
    } else {
        0
    };
    let addbsi_bits = if let Some(bytes) = addbsi {
        let byte_bits = u32::try_from(bytes.len())
            .ok()
            .and_then(|length| length.checked_mul(8))
            .ok_or(FrameBitWriterError::CursorOverflow)?;
        u32::from(D0_ADDBSIE_WIDTH_BITS)
            .checked_add(u32::from(D0_ADDBSIL_WIDTH_BITS))
            .and_then(|bits| bits.checked_add(byte_bits))
            .ok_or(FrameBitWriterError::CursorOverflow)?
    } else {
        u32::from(D0_ADDBSIE_WIDTH_BITS)
    };
    let total_bits = information_bits
        .checked_add(addbsi_bits)
        .ok_or(FrameBitWriterError::CursorOverflow)?;
    writer.preflight_capacity(total_bits)?;

    if let Some(information) = information {
        writer.write_unsigned(u32::from(information.bsmod), D0_BSMOD_WIDTH_BITS)?;
        writer.write_bit(information.copyrightb)?;
        writer.write_bit(information.origbs)?;
        writer.write_bit(information.audio_production.is_some())?;
        if let Some(audio_production) = information.audio_production {
            writer.write_unsigned(u32::from(audio_production.mixlevel), D0_MIXLEVEL_WIDTH_BITS)?;
            writer.write_unsigned(u32::from(audio_production.roomtyp), D0_ROOMTYP_WIDTH_BITS)?;
            writer.write_unsigned(u32::from(audio_production.convtyp), D0_CONVTYP_WIDTH_BITS)?;
        }
        writer.write_unsigned(
            u32::from(information.sourcefscod),
            D0_SOURCEFSCOD_WIDTH_BITS,
        )?;
    }

    writer.write_bit(addbsi.is_some())?;
    if let Some(bytes) = addbsi {
        writer.write_unsigned(
            u32::try_from(bytes.len() - 1).map_err(|_| FrameBitWriterError::CursorOverflow)?,
            D0_ADDBSIL_WIDTH_BITS,
        )?;
        for byte in bytes {
            writer.write_unsigned(u32::from(*byte), 8)?;
        }
    }

    Ok(writer.cursor())
}

/// Writes the complete selected D0 BSI structure from the fixed prefix through opaque addbsi.
///
/// All component writers first validate and serialize into bounded private scratch. The caller's
/// writer is preflighted for the resulting complete bit extent before those staged bits are
/// copied, so any invalid component or insufficient capacity leaves caller bytes and cursor intact.
/// The only accepted plan is the approved Flat7x D0 row; this is a structural BSI entry, not a
/// frame/AU, audio, CRC, JOC interpretation, or production encoder.
pub fn write_flat7x_d0_bsi_structure(
    writer: &mut FrameBitWriter<'_>,
    target: Flat7xI0D0TargetPlan,
    structure: Flat7xD0BsiStructure<'_>,
) -> Result<FrameBitCursor, Flat7xD0BsiStructureError> {
    let actual = writer.cursor().bit_offset();
    if actual != 0 {
        return Err(Flat7xD0BsiStructureError::CursorMismatch { actual });
    }
    if target != Flat7xI0D0TargetPlan::approved_contract() {
        return Err(Flat7xD0BsiStructureError::UnsupportedTargetContext);
    }

    // Maximum extent: 76 BSI bits + 54 mixing bits + 534 information/addbsi bits.
    let mut staging = [0_u8; D0_BSI_STRUCTURE_STAGING_BYTES];
    let end;
    {
        let mut staging_writer = FrameBitWriter::new(&mut staging, FrameBitCursor::new(0))?;
        write_flat7x_d0_bsi_through_chanmap(
            &mut staging_writer,
            structure.dialnorm,
            structure.compr,
        )?;
        append_flat7x_d0_mixing_metadata_and_infomdate(
            &mut staging_writer,
            target,
            structure.mixing,
            structure.block_mix,
            structure.information.is_some(),
        )?;
        append_flat7x_d0_info_and_addbsi(
            &mut staging_writer,
            target,
            structure.information.is_some(),
            structure.information,
            structure.addbsi,
        )?;
        end = staging_writer.cursor().bit_offset();
    }

    writer.preflight_capacity(end)?;
    for bit_offset in 0..end {
        let byte = staging[(bit_offset / 8) as usize];
        let value = byte & (1_u8 << (7 - bit_offset % 8)) != 0;
        writer.write_bit(value)?;
    }
    Ok(writer.cursor())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedComplexityFieldWriteError {
    CursorMismatch { actual: u32 },
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for SelectedComplexityFieldWriteError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Writes only the selected F12 additional-BSI complexity U8 at bits 105..112.
///
/// The caller owns presence/length/identifier fields and positions the cursor.
/// This field writer does not derive the model value, select a carrier block,
/// identify the native BSI producer, or assemble a complete frame.
pub fn write_selected_complexity_index_type_a(
    writer: &mut FrameBitWriter<'_>,
    complexity: SelectedComplexityIndexTypeA,
) -> Result<FrameBitCursor, SelectedComplexityFieldWriteError> {
    let actual = writer.cursor().bit_offset();
    if actual != COMPLEXITY_INDEX_TYPE_A_BIT_OFFSET {
        return Err(SelectedComplexityFieldWriteError::CursorMismatch { actual });
    }
    writer.preflight_capacity(u32::from(COMPLEXITY_INDEX_TYPE_A_WIDTH))?;
    writer.write_unsigned(u32::from(complexity.value()), COMPLEXITY_INDEX_TYPE_A_WIDTH)?;
    Ok(writer.cursor())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedBsiDrcWriterError {
    HeaderAlreadyWritten,
    HeaderCursorMismatch { actual: u32 },
    HeaderNotWritten,
    BlockOutOfOrder { expected: u8, actual: u8 },
    IncompleteBlockFields { written: u8 },
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for SelectedBsiDrcWriterError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedBsiDrcFieldsWritten {
    source_index: CodecSourceIndex,
}

impl SelectedBsiDrcFieldsWritten {
    pub const fn source_index(self) -> CodecSourceIndex {
        self.source_index
    }
}

/// Writes the selected F06ND DRC fields into caller-owned F12 frame scratch.
///
/// The header segment starts at bit 45. The caller interleaves each line DRC
/// field at the current cursor after that block's optional short and dither
/// syntax, and must call blocks in order 0 through 5. This stage owns no frame
/// state, CRC, candidate retry or parent commit.
pub struct SelectedBsiDrcWriter {
    fields: SelectedBsiDrcFields,
    header_written: bool,
    next_block: u8,
}

impl SelectedBsiDrcWriter {
    pub const fn new(fields: SelectedBsiDrcFields) -> Self {
        Self {
            fields,
            header_written: false,
            next_block: 0,
        }
    }

    /// Writes U5 dialnorm, `compre`, then optional U8 `compr` at bit 45.
    pub fn write_header_drc(
        &mut self,
        writer: &mut FrameBitWriter<'_>,
    ) -> Result<(), SelectedBsiDrcWriterError> {
        if self.header_written {
            return Err(SelectedBsiDrcWriterError::HeaderAlreadyWritten);
        }
        let cursor = writer.cursor().bit_offset();
        if cursor != DIALNORM_BIT_OFFSET {
            return Err(SelectedBsiDrcWriterError::HeaderCursorMismatch { actual: cursor });
        }

        let dialnorm = u32::from(self.fields.dialnorm());
        let (value, width) = match self.fields.rf_code() {
            Some(code) => ((dialnorm << 9) | (1 << 8) | u32::from(code), 14),
            None => (dialnorm << 1, 6),
        };
        writer.write_unsigned(value, width)?;
        self.header_written = true;
        Ok(())
    }

    /// Writes one line-presence bit and its optional U8 code at the live cursor.
    pub fn write_line_drc(
        &mut self,
        writer: &mut FrameBitWriter<'_>,
        block: CodecBlockIndex,
    ) -> Result<(), SelectedBsiDrcWriterError> {
        if !self.header_written {
            return Err(SelectedBsiDrcWriterError::HeaderNotWritten);
        }
        let actual = block.value();
        if actual != self.next_block {
            return Err(SelectedBsiDrcWriterError::BlockOutOfOrder {
                expected: self.next_block,
                actual,
            });
        }

        let (value, width) = match self.fields.line_code(block) {
            Some(code) => ((1 << 8) | u32::from(code), 9),
            None => (0, 1),
        };
        writer.write_unsigned(value, width)?;
        self.next_block += 1;
        Ok(())
    }

    pub const fn is_complete(&self) -> bool {
        self.header_written && self.next_block == LINE_BLOCK_COUNT
    }

    pub fn finish(self) -> Result<SelectedBsiDrcFieldsWritten, SelectedBsiDrcWriterError> {
        if !self.header_written {
            return Err(SelectedBsiDrcWriterError::HeaderNotWritten);
        }
        if self.next_block != LINE_BLOCK_COUNT {
            return Err(SelectedBsiDrcWriterError::IncompleteBlockFields {
                written: self.next_block,
            });
        }
        Ok(SelectedBsiDrcFieldsWritten {
            source_index: self.fields.source_index(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedBsiMixingInformationField {
    Dmixmod,
    Ltrtcmixlev,
    Lorocmixlev,
    Ltrtsurmixlev,
    Lorosurmixlev,
    Bsmod,
    Dsurexmod,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedBsiMixingInformationWriteError {
    HeaderCursorMismatch {
        actual: u32,
    },
    FieldOutOfRange {
        field: SelectedBsiMixingInformationField,
        value: u8,
    },
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for SelectedBsiMixingInformationWriteError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Writes selected-profile `mixmdate` and `infomdate` syntax after the DRC header.
///
/// The fixed acmod=7/lfeon=1/strmtyp=0 profile emits the selected conditional
/// fields and fixed zero presence flags from the approved frame-assembly
/// contract. The caller supplies normalized profile values; this writer does
/// not produce or bind their source owner/generation.
pub fn write_selected_bsi_mixing_information(
    writer: &mut FrameBitWriter<'_>,
    fields: &SelectedBsiMixingInformationValues,
) -> Result<FrameBitCursor, SelectedBsiMixingInformationWriteError> {
    let actual = writer.cursor().bit_offset();
    let cursor_without_compr = DIALNORM_BIT_OFFSET + SELECTED_DRC_HEADER_BITS_WITHOUT_COMPR;
    let cursor_with_compr = DIALNORM_BIT_OFFSET + SELECTED_DRC_HEADER_BITS_WITH_COMPR;
    if actual != cursor_without_compr && actual != cursor_with_compr {
        return Err(SelectedBsiMixingInformationWriteError::HeaderCursorMismatch { actual });
    }

    for (field, value, maximum) in [
        (
            SelectedBsiMixingInformationField::Dmixmod,
            fields.dmixmod,
            3,
        ),
        (
            SelectedBsiMixingInformationField::Ltrtcmixlev,
            fields.ltrtcmixlev,
            7,
        ),
        (
            SelectedBsiMixingInformationField::Lorocmixlev,
            fields.lorocmixlev,
            7,
        ),
        (
            SelectedBsiMixingInformationField::Ltrtsurmixlev,
            fields.ltrtsurmixlev,
            7,
        ),
        (
            SelectedBsiMixingInformationField::Lorosurmixlev,
            fields.lorosurmixlev,
            7,
        ),
        (SelectedBsiMixingInformationField::Bsmod, fields.bsmod, 7),
        (
            SelectedBsiMixingInformationField::Dsurexmod,
            fields.dsurexmod,
            3,
        ),
    ] {
        if value > maximum {
            return Err(SelectedBsiMixingInformationWriteError::FieldOutOfRange { field, value });
        }
    }

    writer.preflight_capacity(SELECTED_MIXING_INFORMATION_BITS)?;

    // The selected profile has mixmdate=1; conditional mix levels follow MSB-first.
    writer.write_bit(true)?;
    writer.write_unsigned(u32::from(fields.dmixmod), 2)?;
    writer.write_unsigned(u32::from(fields.ltrtcmixlev), 3)?;
    writer.write_unsigned(u32::from(fields.lorocmixlev), 3)?;
    writer.write_unsigned(u32::from(fields.ltrtsurmixlev), 3)?;
    writer.write_unsigned(u32::from(fields.lorosurmixlev), 3)?;

    // Selected zero presence flags suppress their optional payloads.
    writer.write_bit(false)?; // lfemixlevcode
    writer.write_bit(false)?; // pgmscle
    writer.write_bit(false)?; // extpgmscle
    writer.write_unsigned(0, 2)?; // mixdef=0, no additional payload
    writer.write_bit(false)?; // frmmixcfginfoe

    // The selected profile has infomdate=1; acmod=7 omits dsurmod/dheadphonmod.
    writer.write_bit(true)?;
    writer.write_unsigned(u32::from(fields.bsmod), 3)?;
    writer.write_bit(fields.copyrightb)?;
    writer.write_bit(fields.origbs)?;
    writer.write_unsigned(u32::from(fields.dsurexmod), 2)?;
    writer.write_bit(false)?; // audprodie
    writer.write_bit(fields.sourcefscod)?;

    Ok(writer.cursor())
}

#[cfg(test)]
mod selected_bsi_mixing_information_tests {
    use super::*;

    fn append_bits(bits: &mut Vec<bool>, value: u32, width: u8) {
        for shift in (0..width).rev() {
            bits.push((value >> shift) & 1 != 0);
        }
    }

    fn bit_at(bytes: &[u8], bit_offset: u32) -> bool {
        (bytes[(bit_offset / 8) as usize] >> (7 - bit_offset % 8)) & 1 != 0
    }

    fn fixture_fields() -> SelectedBsiMixingInformationValues {
        SelectedBsiMixingInformationValues {
            dmixmod: 2,
            ltrtcmixlev: 1,
            lorocmixlev: 2,
            ltrtsurmixlev: 4,
            lorosurmixlev: 7,
            bsmod: 5,
            copyrightb: true,
            origbs: false,
            dsurexmod: 3,
            sourcefscod: true,
        }
    }

    #[test]
    fn writes_selected_mixing_and_information_fields_msb_first() {
        let start = DIALNORM_BIT_OFFSET + SELECTED_DRC_HEADER_BITS_WITHOUT_COMPR;
        let before = [0xa5_u8; 16];
        let mut bytes = before;
        let cursor = {
            let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
            write_selected_bsi_mixing_information(&mut writer, &fixture_fields()).unwrap()
        };

        let mut expected = Vec::with_capacity(SELECTED_MIXING_INFORMATION_BITS as usize);
        append_bits(&mut expected, 1, 1); // mixmdate
        append_bits(&mut expected, 2, 2); // dmixmod
        append_bits(&mut expected, 1, 3); // ltrtcmixlev
        append_bits(&mut expected, 2, 3); // lorocmixlev
        append_bits(&mut expected, 4, 3); // ltrtsurmixlev
        append_bits(&mut expected, 7, 3); // lorosurmixlev
        append_bits(&mut expected, 0, 1); // lfemixlevcode
        append_bits(&mut expected, 0, 1); // pgmscle
        append_bits(&mut expected, 0, 1); // extpgmscle
        append_bits(&mut expected, 0, 2); // mixdef
        append_bits(&mut expected, 0, 1); // frmmixcfginfoe
        append_bits(&mut expected, 1, 1); // infomdate
        append_bits(&mut expected, 5, 3); // bsmod
        append_bits(&mut expected, 1, 1); // copyrightb
        append_bits(&mut expected, 0, 1); // origbs
        append_bits(&mut expected, 3, 2); // dsurexmod
        append_bits(&mut expected, 0, 1); // audprodie
        append_bits(&mut expected, 1, 1); // sourcefscod

        assert_eq!(expected.len(), SELECTED_MIXING_INFORMATION_BITS as usize);
        assert_eq!(
            cursor.bit_offset(),
            start + SELECTED_MIXING_INFORMATION_BITS
        );
        for (offset, bit) in expected.into_iter().enumerate() {
            assert_eq!(bit_at(&bytes, start + offset as u32), bit);
        }
        for bit in 0..start {
            assert_eq!(bit_at(&bytes, bit), bit_at(&before, bit));
        }
        for bit in (start + SELECTED_MIXING_INFORMATION_BITS)..128 {
            assert_eq!(bit_at(&bytes, bit), bit_at(&before, bit));
        }
    }

    #[test]
    fn accepts_the_header_cursor_after_either_drc_compre_branch() {
        let start = DIALNORM_BIT_OFFSET + SELECTED_DRC_HEADER_BITS_WITH_COMPR;
        let mut bytes = [0_u8; 16];
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
        let cursor = write_selected_bsi_mixing_information(&mut writer, &fixture_fields()).unwrap();
        assert_eq!(
            cursor.bit_offset(),
            start + SELECTED_MIXING_INFORMATION_BITS
        );
    }

    #[test]
    fn rejects_invalid_fields_cursor_and_capacity_before_mutation() {
        let start = DIALNORM_BIT_OFFSET + SELECTED_DRC_HEADER_BITS_WITHOUT_COMPR;
        let mut bytes = [0xa5_u8; 16];
        let before = bytes;
        let mut invalid = fixture_fields();
        invalid.dmixmod = 4;
        let error = {
            let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
            write_selected_bsi_mixing_information(&mut writer, &invalid).unwrap_err()
        };
        assert_eq!(
            error,
            SelectedBsiMixingInformationWriteError::FieldOutOfRange {
                field: SelectedBsiMixingInformationField::Dmixmod,
                value: 4,
            }
        );
        assert_eq!(bytes, before);

        let mut bytes = before;
        let error = {
            let mut writer =
                FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start + 1)).unwrap();
            write_selected_bsi_mixing_information(&mut writer, &fixture_fields()).unwrap_err()
        };
        assert_eq!(
            error,
            SelectedBsiMixingInformationWriteError::HeaderCursorMismatch { actual: start + 1 }
        );
        assert_eq!(bytes, before);

        let mut short = [0xa5_u8; 10];
        let before = short;
        let error = {
            let mut writer = FrameBitWriter::new(&mut short, FrameBitCursor::new(start)).unwrap();
            write_selected_bsi_mixing_information(&mut writer, &fixture_fields()).unwrap_err()
        };
        assert_eq!(
            error,
            SelectedBsiMixingInformationWriteError::BitWriter(FrameBitWriterError::BufferTooSmall)
        );
        assert_eq!(short, before);
    }
}
