// pattern: Functional Core
//! Stateless F05O reader for a single unprotected rowdata section body.

use super::rowdata::{OamdRowKind, OamdTiming};

/// Header values decoded from one selected OAMD body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OamdBodyHeader {
    /// Version field.
    pub version: u8,
    /// Five-bit count prefix from the wire.
    pub count_code: u8,
    /// Total LFE and object row count.
    pub row_count: usize,
    /// Shortcut flag.
    pub shortcut: bool,
    /// Whether row zero is the leading LFE row.
    pub leading_lfe: bool,
    /// Whether the body marks protected sections.
    pub protected_sections: bool,
    /// Number of sections in the body.
    pub section_count: u8,
}

/// One row decoded from the body rowdata section.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OamdDecodedRow {
    /// Row kind derived from the body header and position.
    pub kind: OamdRowKind,
    /// Mute bit.
    pub mute: bool,
    /// Decoded gain.
    pub gain: u32,
    /// Decoded spread.
    pub spread: u32,
    /// Decoded Q15 XYZ point.
    pub xyz_q15: [i32; 3],
    /// Decoded component value; wire value seven maps to zero.
    pub component: u8,
    /// Whether decoded Z is valid.
    pub z_valid: bool,
    /// Diffuse bit.
    pub diffuse: bool,
}

/// Stateless metadata returned by the selected F05O body reader.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OamdBodyMetadata {
    /// Header values recovered from the body bytes.
    pub header: OamdBodyHeader,
    /// Single-frame timing values recovered from the rowdata section.
    pub timing: OamdTiming,
    /// Rows in header-declared order.
    pub rows: Vec<OamdDecodedRow>,
}

/// Local rejection categories for the bounded F05O body reader.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OamdBodyReadError {
    /// The body is truncated or has malformed bit framing or padding.
    InvalidInput,
    /// The body requests a layout or field branch outside the direct reader scope.
    UnsupportedProfile,
    /// An escaped section length is malformed or exceeds its four-group form.
    InvalidEscapedInteger,
}

/// Detailed rowdata decode outcomes shared with the two-section assembler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OamdRowdataDecodeError {
    SectionControl,
    Truncated,
    UnconsumedFields,
    NonzeroPadding,
    TimingBase,
    TimingDomain,
    TimingDuration,
    TimingReserved,
    TimingMultiframe,
    RowDomain,
    RowXyz,
    RowGain,
    RowSpread,
    RowExtension,
    EscapedOverflow,
}

impl From<OamdBodyReadError> for OamdRowdataDecodeError {
    fn from(error: OamdBodyReadError) -> Self {
        match error {
            OamdBodyReadError::InvalidInput => Self::Truncated,
            OamdBodyReadError::UnsupportedProfile => Self::RowExtension,
            OamdBodyReadError::InvalidEscapedInteger => Self::EscapedOverflow,
        }
    }
}

const DURATION_TABLE: [u32; 16] = [
    32, 64, 128, 256, 320, 480, 1000, 1001, 1024, 1600, 1601, 1602, 1920, 2000, 2002, 2048,
];

/// Decodes a single unprotected rowdata-section OAMD body.
///
/// `row_count` and `leading_lfe` come only from the MSB-first body header.
pub fn decode_oamd_body(
    payload_bytes: &[u8],
    xy_table_63: &[u32; 63],
    z_table_16: &[u32; 16],
) -> Result<OamdBodyMetadata, OamdBodyReadError> {
    let mut reader = BitReader::new(payload_bytes, payload_bytes.len().saturating_mul(8));
    let version = u8::try_from(reader.read(2)?).map_err(|_| OamdBodyReadError::InvalidInput)?;
    let count_code = u8::try_from(reader.read(5)?).map_err(|_| OamdBodyReadError::InvalidInput)?;
    let row_count = if count_code == 31 {
        32 + usize::try_from(reader.read(7)?).map_err(|_| OamdBodyReadError::InvalidInput)?
    } else {
        usize::from(count_code) + 1
    };
    let shortcut = reader.read(1)? != 0;
    let leading_lfe = reader.read(1)? != 0;
    let protected_sections = reader.read(1)? != 0;
    let section_count =
        u8::try_from(reader.read(4)?).map_err(|_| OamdBodyReadError::InvalidInput)?;
    if version != 0 || !shortcut || protected_sections || section_count != 1 {
        return Err(OamdBodyReadError::UnsupportedProfile);
    }

    let section_kind = reader.read(4)?;
    if section_kind != 1 {
        return Err(OamdBodyReadError::UnsupportedProfile);
    }
    let payload_length = read_escaped_value(&mut reader)?
        .checked_add(1)
        .ok_or(OamdBodyReadError::InvalidEscapedInteger)?;
    let payload_length =
        usize::try_from(payload_length).map_err(|_| OamdBodyReadError::InvalidInput)?;
    let payload_bits = payload_length
        .checked_mul(8)
        .ok_or(OamdBodyReadError::InvalidInput)?;
    if payload_bits > reader.remaining_bits() {
        return Err(OamdBodyReadError::InvalidInput);
    }
    let mut rowdata_payload = Vec::with_capacity(payload_length);
    for _ in 0..payload_length {
        rowdata_payload
            .push(u8::try_from(reader.read(8)?).map_err(|_| OamdBodyReadError::InvalidInput)?);
    }
    reader.finish_zero_padding()?;

    let (timing, rows, _) = decode_rowdata_payload_with_bits(
        &rowdata_payload,
        row_count,
        leading_lfe,
        xy_table_63,
        z_table_16,
    )
    .map_err(map_rowdata_error_to_body_read_error)?;
    Ok(OamdBodyMetadata {
        header: OamdBodyHeader {
            version,
            count_code,
            row_count,
            shortcut,
            leading_lfe,
            protected_sections,
            section_count,
        },
        timing,
        rows,
    })
}

pub(crate) fn decode_rowdata_payload_with_bits(
    payload: &[u8],
    row_count: usize,
    leading_lfe: bool,
    xy_table_63: &[u32; 63],
    z_table_16: &[u32; 16],
) -> Result<(OamdTiming, Vec<OamdDecodedRow>, u32), OamdRowdataDecodeError> {
    let mut reader = BitReader::new(payload, payload.len().saturating_mul(8));
    if reader.read(1)? != 0 {
        return Err(OamdRowdataDecodeError::SectionControl);
    }
    let timing = decode_timing(&mut reader)?;
    let mut rows = Vec::with_capacity(row_count);
    let mut previous_gain = 0_u32;
    let mut previous_unmuted = false;
    for row_index in 0..row_count {
        let kind = if leading_lfe && row_index == 0 {
            OamdRowKind::Lfe
        } else {
            OamdRowKind::Object
        };
        let mute = reader.read(1)? != 0;
        let decoded = if mute {
            OamdDecodedRow {
                kind,
                mute: true,
                gain: 0,
                spread: 0,
                xyz_q15: [16_384, 16_384, 0],
                component: 0,
                z_valid: true,
                diffuse: false,
            }
        } else {
            let gain_mode = reader.read(2)?;
            let gain = match gain_mode {
                0 => 4096,
                3 if row_index == 0 => 4096,
                3 if previous_unmuted => previous_gain,
                1 | 3 => 0,
                _ => return Err(OamdRowdataDecodeError::RowGain),
            };
            if reader.read(1)? != 1 {
                return Err(OamdRowdataDecodeError::RowSpread);
            }
            let mut row = OamdDecodedRow {
                kind,
                mute: false,
                gain,
                spread: 32_767,
                xyz_q15: [16_384, 16_384, 0],
                component: 0,
                z_valid: true,
                diffuse: false,
            };
            if kind == OamdRowKind::Object {
                let x_code = reader.read(6)?.min(62);
                let y_code = reader.read(6)?.min(62);
                let positive_z = reader.read(1)? != 0;
                let z_code = reader.read(4)?;
                let position_extension = reader.read(1)?;
                let component_code = reader.read(3)?;
                let z_valid = reader.read(1)? != 0;
                let width_mode = reader.read(2)?;
                let ancillary_presence = reader.read(1)?;
                let diffuse = reader.read(1)? != 0;
                if position_extension != 0 || width_mode != 0 || ancillary_presence != 0 {
                    return Err(OamdRowdataDecodeError::RowExtension);
                }
                let z_magnitude = i32::try_from(z_table_16[z_code as usize])
                    .map_err(|_| OamdRowdataDecodeError::RowXyz)?;
                let z = if positive_z {
                    z_magnitude
                } else {
                    -z_magnitude
                };
                row.xyz_q15 = [
                    i32::try_from(xy_table_63[x_code as usize])
                        .map_err(|_| OamdRowdataDecodeError::RowXyz)?,
                    i32::try_from(xy_table_63[y_code as usize])
                        .map_err(|_| OamdRowdataDecodeError::RowXyz)?,
                    if z_valid { z } else { 0 },
                ];
                row.component = if component_code == 7 {
                    0
                } else {
                    u8::try_from(component_code).map_err(|_| OamdRowdataDecodeError::RowDomain)?
                };
                row.z_valid = z_valid;
                row.diffuse = diffuse;
            }
            previous_gain = gain;
            row
        };
        if reader.read(1)? != 0 {
            return Err(OamdRowdataDecodeError::RowExtension);
        }
        previous_unmuted = !mute;
        rows.push(decoded);
    }
    let valid_bits =
        u32::try_from(reader.position()).map_err(|_| OamdRowdataDecodeError::RowDomain)?;
    reader.finish_zero_padding_detailed()?;
    Ok((timing, rows, valid_bits))
}

fn map_rowdata_error_to_body_read_error(error: OamdRowdataDecodeError) -> OamdBodyReadError {
    match error {
        OamdRowdataDecodeError::Truncated
        | OamdRowdataDecodeError::UnconsumedFields
        | OamdRowdataDecodeError::NonzeroPadding
        | OamdRowdataDecodeError::RowXyz
        | OamdRowdataDecodeError::RowDomain
        | OamdRowdataDecodeError::TimingDomain => OamdBodyReadError::InvalidInput,
        OamdRowdataDecodeError::EscapedOverflow => OamdBodyReadError::InvalidEscapedInteger,
        OamdRowdataDecodeError::SectionControl
        | OamdRowdataDecodeError::TimingBase
        | OamdRowdataDecodeError::TimingDuration
        | OamdRowdataDecodeError::TimingReserved
        | OamdRowdataDecodeError::TimingMultiframe
        | OamdRowdataDecodeError::RowGain
        | OamdRowdataDecodeError::RowSpread
        | OamdRowdataDecodeError::RowExtension => OamdBodyReadError::UnsupportedProfile,
    }
}

fn decode_timing(reader: &mut BitReader<'_>) -> Result<OamdTiming, OamdRowdataDecodeError> {
    let base = match reader.read(2)? {
        0 => 0,
        1 => match reader.read(2)? {
            0 => 8,
            1 => 16,
            2 => 18,
            3 => 24,
            _ => unreachable!("two-bit base ordinal is in range"),
        },
        2 => {
            let base = reader.read(5)?;
            if matches!(base, 0 | 8 | 16 | 18 | 24) {
                return Err(OamdRowdataDecodeError::TimingBase);
            }
            base
        }
        _ => return Err(OamdRowdataDecodeError::TimingReserved),
    };
    if reader.read(3)? != 0 {
        return Err(OamdRowdataDecodeError::TimingMultiframe);
    }
    let position = reader.read(6)?;
    let duration = match reader.read(2)? {
        0 => 0,
        1 => 512,
        2 => 1536,
        3 => {
            if reader.read(1)? != 0 {
                DURATION_TABLE[reader.read(4)? as usize]
            } else {
                let duration = reader.read(11)?;
                if matches!(duration, 0 | 512 | 1536) || DURATION_TABLE.contains(&duration) {
                    return Err(OamdRowdataDecodeError::TimingDuration);
                }
                duration
            }
        }
        _ => unreachable!("two-bit duration mode is in range"),
    };
    let precision = if reader.read(1)? != 0 {
        32
    } else {
        reader.read(5)?
    };
    Ok(OamdTiming {
        base: u8::try_from(base).map_err(|_| OamdRowdataDecodeError::TimingBase)?,
        position: u8::try_from(position).map_err(|_| OamdRowdataDecodeError::TimingDomain)?,
        duration: u16::try_from(duration).map_err(|_| OamdRowdataDecodeError::TimingDuration)?,
        precision: u8::try_from(precision).map_err(|_| OamdRowdataDecodeError::TimingDomain)?,
    })
}

fn read_escaped_value(reader: &mut BitReader<'_>) -> Result<u32, OamdBodyReadError> {
    let mut value = 0_u32;
    for group in 0..4 {
        let digit = reader
            .read(4)
            .map_err(|_| OamdBodyReadError::InvalidEscapedInteger)?;
        let continuation = reader
            .read(1)
            .map_err(|_| OamdBodyReadError::InvalidEscapedInteger)?;
        value = if group == 0 {
            digit
        } else {
            value
                .checked_add(1)
                .and_then(|prior| prior.checked_mul(16))
                .and_then(|prior| prior.checked_add(digit))
                .ok_or(OamdBodyReadError::InvalidEscapedInteger)?
        };
        if continuation == 0 {
            return Ok(value);
        }
    }
    Err(OamdBodyReadError::InvalidEscapedInteger)
}

struct BitReader<'a> {
    bytes: &'a [u8],
    limit: usize,
    position: usize,
}

impl<'a> BitReader<'a> {
    const fn new(bytes: &'a [u8], limit: usize) -> Self {
        Self {
            bytes,
            limit,
            position: 0,
        }
    }

    fn read(&mut self, count: u8) -> Result<u32, OamdBodyReadError> {
        let count = usize::from(count);
        if count > 32 || self.position.saturating_add(count) > self.limit {
            return Err(OamdBodyReadError::InvalidInput);
        }
        let mut value = 0_u32;
        for _ in 0..count {
            let byte = self.bytes[self.position / 8];
            let bit = (byte >> (7 - self.position % 8)) & 1;
            value = (value << 1) | u32::from(bit);
            self.position += 1;
        }
        Ok(value)
    }

    fn remaining_bits(&self) -> usize {
        self.limit.saturating_sub(self.position)
    }

    const fn position(&self) -> usize {
        self.position
    }

    fn finish_zero_padding(&mut self) -> Result<(), OamdBodyReadError> {
        if self.remaining_bits() > 7 {
            return Err(OamdBodyReadError::InvalidInput);
        }
        while self.position < self.limit {
            if self.read(1)? != 0 {
                return Err(OamdBodyReadError::InvalidInput);
            }
        }
        Ok(())
    }

    fn finish_zero_padding_detailed(&mut self) -> Result<(), OamdRowdataDecodeError> {
        if self.remaining_bits() > 7 {
            return Err(OamdRowdataDecodeError::UnconsumedFields);
        }
        while self.position < self.limit {
            if self.read(1)? != 0 {
                return Err(OamdRowdataDecodeError::NonzeroPadding);
            }
        }
        Ok(())
    }
}
