// pattern: Functional Core

//! Stateless assembly and decoding of one selected two-section OAMD body.

use super::{
    body_reader::{OamdRowdataDecodeError, decode_rowdata_payload_with_bits},
    rowdata::{
        OamdRow, OamdRowKind, OamdRowdataError, OamdTiming, encode_escaped_integer,
        encode_rowdata_payload_with_bits, encode_xyz_codes,
    },
    trim_section::{Trim, TrimSectionError, decode_trim_section_payload, encode_trim_body},
};

const MAX_ROW_COUNT: usize = 159;
const MAX_BODY_BYTES: usize = 160;
const ROWDATA_SECTION_KIND: u8 = 1;
const TRIM_SECTION_KIND: u8 = 2;

/// Already-resolved metadata consumed by the single-frame body assembler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OamdMetadata {
    /// Optional leading LFE row followed by object rows.
    pub rows: Vec<OamdRow>,
    /// Timing for the single metadata frame.
    pub timing: OamdTiming,
    /// Already-parsed Type11 trim input.
    pub trim: Trim,
}

/// Framing details for one section payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OamdSectionDescriptor {
    /// Section kind: 1 for rowdata or 2 for trim.
    pub kind: u8,
    /// Payload size including its control bit and byte padding.
    pub payload_bytes: usize,
    /// Meaningful bits in the payload before its zero byte padding.
    pub payload_valid_bits: u32,
    /// Escaped length value, equal to `payload_bytes - 1`.
    pub escaped_value: u32,
}

/// Encoded body and its exact outer bit accounting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OamdBodyResult {
    /// Complete Type11 body bytes with zero final padding.
    pub bytes: Vec<u8>,
    /// Number of body bits before final byte padding.
    pub valid_bits: u32,
    /// Number of zero bits appended to align the body to a byte.
    pub padding_bits: u8,
    /// Number of metadata rows in the body.
    pub row_count: usize,
    /// Whether row zero is the leading LFE row.
    pub leading_lfe: bool,
    /// This direct slice always emits one rowdata and one trim section.
    pub section_count: u8,
    /// Descriptors in wire order: rowdata, then trim.
    pub sections: [OamdSectionDescriptor; 2],
}

/// Decoded metadata and framing facts recovered from one complete body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedOamdBody {
    /// Recovered rows, timing and trim.
    pub metadata: OamdMetadata,
    /// Number of rows declared by the body header.
    pub row_count: usize,
    /// Whether row zero is the leading LFE row.
    pub leading_lfe: bool,
    /// Number of sections declared by the body header.
    pub section_count: u8,
    /// Descriptors in wire order: rowdata, then trim.
    pub sections: [OamdSectionDescriptor; 2],
}

/// Rejection categories for the bounded single-frame OAMD assembler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OamdBodyAssemblyError {
    /// Row count is outside 1..=159.
    RowCount,
    /// An LFE row is not the first row.
    LeadingLfe,
    /// A row field is outside its selected domain.
    RowDomain,
    /// Object coordinates cannot be encoded or decoded.
    RowXyz,
    /// Gain has no selected encoding branch.
    RowGain,
    /// An unmuted row does not use full spread.
    RowSpread,
    /// A row uses a disallowed extension field.
    RowExtension,
    /// Timing base has no selected representation.
    TimingBase,
    /// A timing field is outside its wire width.
    TimingDomain,
    /// Duration has no selected representation.
    TimingDuration,
    /// A reserved timing code is present.
    TimingReserved,
    /// The body describes more than one metadata frame.
    TimingMultiframe,
    /// Trim shape is inconsistent with row count.
    TrimShape,
    /// A trim mode or flag is outside its domain.
    TrimDomain,
    /// A trim gain field cannot be represented as U4.
    TrimU4,
    /// A trim balance field is outside the signed range.
    TrimSignedBalance,
    /// Section count is not exactly two.
    SectionCount,
    /// A section kind is unknown or out of order.
    SectionKind,
    /// A section kind appears more than once.
    DuplicateSection,
    /// A declared section is missing.
    SectionMissing,
    /// A section payload control bit is nonzero.
    SectionControl,
    /// Required header or payload bits are truncated.
    Truncated,
    /// A section payload extends past the available body.
    SectionOverflow,
    /// Escaped length arithmetic or representation overflows.
    EscapedOverflow,
    /// Four escaped-length groups all request continuation.
    EscapedUnterminated,
    /// Section or body padding contains a nonzero bit.
    NonzeroPadding,
    /// Unexplained bits remain after the declared sections.
    UnconsumedFields,
    /// The complete body exceeds the 160-byte capacity.
    BodyCapacity,
}

/// Assembles exactly one rowdata section followed by one Type11 trim section.
pub fn assemble_body(metadata: &OamdMetadata) -> Result<OamdBodyResult, OamdBodyAssemblyError> {
    let row_count = metadata.rows.len();
    if !(1..=MAX_ROW_COUNT).contains(&row_count) {
        return Err(OamdBodyAssemblyError::RowCount);
    }

    let leading_lfe = metadata.rows[0].kind == OamdRowKind::Lfe;
    if metadata
        .rows
        .iter()
        .enumerate()
        .skip(1)
        .any(|(_, row)| row.kind == OamdRowKind::Lfe)
    {
        return Err(OamdBodyAssemblyError::LeadingLfe);
    }
    validate_timing(metadata.timing)?;
    validate_rows(&metadata.rows)?;

    let row_count_u32 = u32::try_from(row_count).map_err(|_| OamdBodyAssemblyError::RowCount)?;
    let rowdata = encode_rowdata_payload_with_bits(metadata.timing, &metadata.rows)
        .map_err(map_rowdata_error)?;
    let trim = encode_trim_body(&metadata.trim, row_count_u32).map_err(map_trim_error)?;
    let (trim_payload, trim_valid_bits) = encode_trim_section_payload(&trim)?;

    let rowdata_descriptor = section_descriptor(
        ROWDATA_SECTION_KIND,
        rowdata.bytes.len(),
        rowdata.valid_bits,
    )?;
    let trim_descriptor =
        section_descriptor(TRIM_SECTION_KIND, trim_payload.len(), trim_valid_bits)?;

    let mut writer = BodyBitWriter::default();
    writer.write(0, 2)?;
    if row_count < 32 {
        writer.write(
            u32::try_from(row_count - 1).map_err(|_| OamdBodyAssemblyError::RowCount)?,
            5,
        )?;
    } else {
        writer.write(31, 5)?;
        writer.write(
            u32::try_from(row_count - 32).map_err(|_| OamdBodyAssemblyError::RowCount)?,
            7,
        )?;
    }
    writer.write(1, 1)?;
    writer.write(u32::from(leading_lfe), 1)?;
    writer.write(0, 1)?;
    writer.write(2, 4)?;
    write_section(&mut writer, rowdata_descriptor, &rowdata.bytes)?;
    write_section(&mut writer, trim_descriptor, &trim_payload)?;

    let (bytes, valid_bits, padding_bits) = writer.finish()?;
    if bytes.len() > MAX_BODY_BYTES {
        return Err(OamdBodyAssemblyError::BodyCapacity);
    }

    Ok(OamdBodyResult {
        bytes,
        valid_bits,
        padding_bits,
        row_count,
        leading_lfe,
        section_count: 2,
        sections: [rowdata_descriptor, trim_descriptor],
    })
}

/// Decodes one complete two-section body using caller-supplied PG06 tables.
pub fn decode_body(
    bytes: &[u8],
    xy_table_63: &[u32; 63],
    z_table_16: &[u32; 16],
) -> Result<DecodedOamdBody, OamdBodyAssemblyError> {
    if bytes.len() > MAX_BODY_BYTES {
        return Err(OamdBodyAssemblyError::BodyCapacity);
    }

    let mut reader = BodyBitReader::new(bytes)?;
    let version = reader.read(2)?;
    let count_code = reader.read(5)?;
    let row_count = if count_code == 31 {
        32_usize
            .checked_add(
                usize::try_from(reader.read(7)?).map_err(|_| OamdBodyAssemblyError::RowCount)?,
            )
            .ok_or(OamdBodyAssemblyError::RowCount)?
    } else {
        usize::try_from(count_code + 1).map_err(|_| OamdBodyAssemblyError::RowCount)?
    };
    let shortcut = reader.read(1)?;
    let leading_lfe = reader.read(1)? != 0;
    let protected_sections = reader.read(1)?;
    let section_count =
        u8::try_from(reader.read(4)?).map_err(|_| OamdBodyAssemblyError::SectionCount)?;
    if version != 0 || shortcut != 1 || protected_sections != 0 {
        return Err(OamdBodyAssemblyError::RowDomain);
    }
    if section_count != 2 {
        return Err(OamdBodyAssemblyError::SectionCount);
    }

    let rowdata_section = read_section(&mut reader, ROWDATA_SECTION_KIND)?;
    let trim_section = read_section(&mut reader, TRIM_SECTION_KIND)?;
    reader.finish_zero_padding()?;

    let (timing, decoded_rows, rowdata_valid_bits) = decode_rowdata_payload_with_bits(
        &rowdata_section.payload,
        row_count,
        leading_lfe,
        xy_table_63,
        z_table_16,
    )
    .map_err(map_rowdata_decode_error)?;
    let row_flags_count = u32::try_from(row_count).map_err(|_| OamdBodyAssemblyError::RowCount)?;
    let trim_payload = trim_section.payload.as_slice();
    if trim_payload
        .first()
        .is_some_and(|first_byte| first_byte & 0x80 != 0)
    {
        return Err(OamdBodyAssemblyError::SectionControl);
    }
    let (trim, trim_valid_bits) = decode_trim_section_payload(trim_payload, row_flags_count)
        .map_err(map_trim_decode_error)?;

    let rows = decoded_rows.into_iter().map(row_from_decoded).collect();
    let rowdata_descriptor = section_descriptor(
        ROWDATA_SECTION_KIND,
        rowdata_section.payload.len(),
        rowdata_valid_bits,
    )?;
    let trim_descriptor = section_descriptor(
        TRIM_SECTION_KIND,
        trim_section.payload.len(),
        trim_valid_bits,
    )?;

    Ok(DecodedOamdBody {
        metadata: OamdMetadata { rows, timing, trim },
        row_count,
        leading_lfe,
        section_count,
        sections: [rowdata_descriptor, trim_descriptor],
    })
}

fn validate_timing(timing: OamdTiming) -> Result<(), OamdBodyAssemblyError> {
    if timing.base > 31 {
        return Err(OamdBodyAssemblyError::TimingBase);
    }
    if timing.position > 63 || timing.precision > 32 {
        return Err(OamdBodyAssemblyError::TimingDomain);
    }
    if timing.duration > 2048 {
        return Err(OamdBodyAssemblyError::TimingDuration);
    }
    Ok(())
}

fn validate_rows(rows: &[OamdRow]) -> Result<(), OamdBodyAssemblyError> {
    let mut previous_unmuted_gain = None;
    for row in rows {
        if row.mute {
            previous_unmuted_gain = None;
            continue;
        }

        let reuses_previous_gain = previous_unmuted_gain == Some(row.gain);
        if !reuses_previous_gain && !(3866..=4337).contains(&row.gain) && row.gain != 0 {
            return Err(OamdBodyAssemblyError::RowGain);
        }
        previous_unmuted_gain = Some(row.gain);

        if row.spread != 32_767 {
            return Err(OamdBodyAssemblyError::RowSpread);
        }
        if row.kind == OamdRowKind::Object {
            if row.component > 7 {
                return Err(OamdBodyAssemblyError::RowDomain);
            }
            encode_xyz_codes(row.xyz_q15).map_err(|_| OamdBodyAssemblyError::RowXyz)?;
        }
    }
    Ok(())
}

fn encode_trim_section_payload(
    trim: &super::trim_section::EncodedTrimBody,
) -> Result<(Vec<u8>, u32), OamdBodyAssemblyError> {
    let mut writer = BodyBitWriter::default();
    writer.write(0, 1)?;
    writer.write_bits(
        &trim.body_bytes,
        usize::try_from(trim.valid_bits).map_err(|_| OamdBodyAssemblyError::TrimShape)?,
    )?;
    let (bytes, valid_bits, _) = writer.finish()?;
    Ok((bytes, valid_bits))
}

fn section_descriptor(
    kind: u8,
    payload_bytes: usize,
    payload_valid_bits: u32,
) -> Result<OamdSectionDescriptor, OamdBodyAssemblyError> {
    let escaped_value = payload_bytes
        .checked_sub(1)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(OamdBodyAssemblyError::SectionOverflow)?;
    Ok(OamdSectionDescriptor {
        kind,
        payload_bytes,
        payload_valid_bits,
        escaped_value,
    })
}

fn write_section(
    writer: &mut BodyBitWriter,
    descriptor: OamdSectionDescriptor,
    payload: &[u8],
) -> Result<(), OamdBodyAssemblyError> {
    writer.write(u32::from(descriptor.kind), 4)?;
    let (encoded_length, valid_bits) =
        encode_escaped_integer(i64::from(descriptor.escaped_value)).map_err(map_rowdata_error)?;
    writer.write_bits(&encoded_length, usize::from(valid_bits))?;
    writer.write_bytes(payload)
}

fn read_section(
    reader: &mut BodyBitReader<'_>,
    expected_kind: u8,
) -> Result<SectionPayload, OamdBodyAssemblyError> {
    if reader.remaining_bits() == 0 {
        return Err(OamdBodyAssemblyError::SectionMissing);
    }
    if reader.remaining_bits() < 4 && reader.remaining_bits_are_zero() {
        return Err(OamdBodyAssemblyError::SectionMissing);
    }
    let kind = u8::try_from(reader.read(4)?).map_err(|_| OamdBodyAssemblyError::SectionKind)?;
    if !(ROWDATA_SECTION_KIND..=TRIM_SECTION_KIND).contains(&kind) {
        return Err(OamdBodyAssemblyError::SectionKind);
    }
    if kind != expected_kind {
        if expected_kind == TRIM_SECTION_KIND && kind == ROWDATA_SECTION_KIND {
            return Err(OamdBodyAssemblyError::DuplicateSection);
        }
        if kind == ROWDATA_SECTION_KIND || kind == TRIM_SECTION_KIND {
            return Err(OamdBodyAssemblyError::SectionKind);
        }
        return Err(OamdBodyAssemblyError::SectionKind);
    }

    let escaped_value = reader.read_escaped_value()?;
    let payload_length = escaped_value
        .checked_add(1)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(OamdBodyAssemblyError::EscapedOverflow)?;
    let payload = reader.read_bytes(payload_length)?;
    Ok(SectionPayload { payload })
}

fn row_from_decoded(row: super::body_reader::OamdDecodedRow) -> OamdRow {
    OamdRow {
        kind: row.kind,
        mute: row.mute,
        gain: row.gain,
        spread: row.spread,
        xyz_q15: row.xyz_q15,
        component: row.component,
        z_valid: row.z_valid,
        diffuse: row.diffuse,
    }
}

fn map_rowdata_error(error: OamdRowdataError) -> OamdBodyAssemblyError {
    match error {
        OamdRowdataError::InvalidInput | OamdRowdataError::UnsupportedProfile => {
            OamdBodyAssemblyError::RowDomain
        }
        OamdRowdataError::InvalidEscapedInteger => OamdBodyAssemblyError::EscapedOverflow,
    }
}

fn map_trim_error(error: TrimSectionError) -> OamdBodyAssemblyError {
    match error {
        TrimSectionError::TrimShape => OamdBodyAssemblyError::TrimShape,
        TrimSectionError::TrimDomain
        | TrimSectionError::TrimReserved
        | TrimSectionError::TrimGroupMode
        | TrimSectionError::DbmdHeader
        | TrimSectionError::DbmdTruncatedHeader
        | TrimSectionError::DbmdTruncatedPayload
        | TrimSectionError::DbmdChecksum
        | TrimSectionError::DbmdDuplicateSection
        | TrimSectionError::DbmdTerminatorTail
        | TrimSectionError::DbmdRequiredSections
        | TrimSectionError::DbmdSupplemental
        | TrimSectionError::DbmdWarmMode
        | TrimSectionError::DbmdNondefaultTrim
        | TrimSectionError::DbmdRowCount => OamdBodyAssemblyError::TrimDomain,
        TrimSectionError::TrimU4 => OamdBodyAssemblyError::TrimU4,
        TrimSectionError::TrimSignedBalance => OamdBodyAssemblyError::TrimSignedBalance,
        TrimSectionError::TrimTruncated => OamdBodyAssemblyError::Truncated,
        TrimSectionError::TrimNonzeroPadding => OamdBodyAssemblyError::NonzeroPadding,
    }
}

fn map_trim_decode_error(error: TrimSectionError) -> OamdBodyAssemblyError {
    match error {
        TrimSectionError::TrimReserved => OamdBodyAssemblyError::TrimDomain,
        other => map_trim_error(other),
    }
}

fn map_rowdata_decode_error(error: OamdRowdataDecodeError) -> OamdBodyAssemblyError {
    match error {
        OamdRowdataDecodeError::SectionControl => OamdBodyAssemblyError::SectionControl,
        OamdRowdataDecodeError::Truncated => OamdBodyAssemblyError::Truncated,
        OamdRowdataDecodeError::UnconsumedFields => OamdBodyAssemblyError::UnconsumedFields,
        OamdRowdataDecodeError::NonzeroPadding => OamdBodyAssemblyError::NonzeroPadding,
        OamdRowdataDecodeError::TimingBase => OamdBodyAssemblyError::TimingBase,
        OamdRowdataDecodeError::TimingDomain => OamdBodyAssemblyError::TimingDomain,
        OamdRowdataDecodeError::TimingDuration => OamdBodyAssemblyError::TimingDuration,
        OamdRowdataDecodeError::TimingReserved => OamdBodyAssemblyError::TimingReserved,
        OamdRowdataDecodeError::TimingMultiframe => OamdBodyAssemblyError::TimingMultiframe,
        OamdRowdataDecodeError::RowDomain => OamdBodyAssemblyError::RowDomain,
        OamdRowdataDecodeError::RowXyz => OamdBodyAssemblyError::RowXyz,
        OamdRowdataDecodeError::RowGain => OamdBodyAssemblyError::RowGain,
        OamdRowdataDecodeError::RowSpread => OamdBodyAssemblyError::RowSpread,
        OamdRowdataDecodeError::RowExtension => OamdBodyAssemblyError::RowExtension,
        OamdRowdataDecodeError::EscapedOverflow => OamdBodyAssemblyError::EscapedOverflow,
    }
}

struct SectionPayload {
    payload: Vec<u8>,
}

#[derive(Default)]
struct BodyBitWriter {
    bytes: Vec<u8>,
    bit_position: usize,
}

impl BodyBitWriter {
    fn write(&mut self, value: u32, width: u8) -> Result<(), OamdBodyAssemblyError> {
        if width > 32 || (width < 32 && value >= (1_u32 << width)) {
            return Err(OamdBodyAssemblyError::RowDomain);
        }
        for shift in (0..width).rev() {
            let byte_index = self.bit_position / 8;
            if byte_index == self.bytes.len() {
                self.bytes.push(0);
            }
            let bit = (value >> shift) & 1;
            self.bytes[byte_index] |= (bit as u8) << (7 - self.bit_position % 8);
            self.bit_position += 1;
        }
        Ok(())
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), OamdBodyAssemblyError> {
        for byte in bytes {
            self.write(u32::from(*byte), 8)?;
        }
        Ok(())
    }

    fn write_bits(&mut self, bytes: &[u8], bit_count: usize) -> Result<(), OamdBodyAssemblyError> {
        let available = bytes
            .len()
            .checked_mul(8)
            .ok_or(OamdBodyAssemblyError::BodyCapacity)?;
        if bit_count > available {
            return Err(OamdBodyAssemblyError::Truncated);
        }
        for position in 0..bit_count {
            let bit = (bytes[position / 8] >> (7 - position % 8)) & 1;
            self.write(u32::from(bit), 1)?;
        }
        Ok(())
    }

    fn finish(mut self) -> Result<(Vec<u8>, u32, u8), OamdBodyAssemblyError> {
        let valid_bits =
            u32::try_from(self.bit_position).map_err(|_| OamdBodyAssemblyError::BodyCapacity)?;
        let padding_bits = ((8 - self.bit_position % 8) % 8) as u8;
        for _ in 0..padding_bits {
            self.write(0, 1)?;
        }
        Ok((self.bytes, valid_bits, padding_bits))
    }
}

struct BodyBitReader<'a> {
    bytes: &'a [u8],
    bit_limit: usize,
    bit_position: usize,
}

impl<'a> BodyBitReader<'a> {
    fn new(bytes: &'a [u8]) -> Result<Self, OamdBodyAssemblyError> {
        let bit_limit = bytes
            .len()
            .checked_mul(8)
            .ok_or(OamdBodyAssemblyError::BodyCapacity)?;
        Ok(Self {
            bytes,
            bit_limit,
            bit_position: 0,
        })
    }

    fn read(&mut self, width: u8) -> Result<u32, OamdBodyAssemblyError> {
        let end = self
            .bit_position
            .checked_add(usize::from(width))
            .ok_or(OamdBodyAssemblyError::Truncated)?;
        if width > 32 || end > self.bit_limit {
            return Err(OamdBodyAssemblyError::Truncated);
        }
        let mut value = 0_u32;
        while self.bit_position < end {
            let bit = (self.bytes[self.bit_position / 8] >> (7 - self.bit_position % 8)) & 1;
            value = (value << 1) | u32::from(bit);
            self.bit_position += 1;
        }
        Ok(value)
    }

    fn remaining_bits(&self) -> usize {
        self.bit_limit - self.bit_position
    }

    fn remaining_bits_are_zero(&self) -> bool {
        (self.bit_position..self.bit_limit)
            .all(|position| (self.bytes[position / 8] >> (7 - position % 8)) & 1 == 0)
    }

    fn read_escaped_value(&mut self) -> Result<u32, OamdBodyAssemblyError> {
        let mut value = 0_u32;
        for group in 0..4 {
            let digit = self.read(4)?;
            let continuation = self.read(1)? != 0;
            value = if group == 0 {
                digit
            } else {
                value
                    .checked_add(1)
                    .and_then(|prior| prior.checked_mul(16))
                    .and_then(|prior| prior.checked_add(digit))
                    .ok_or(OamdBodyAssemblyError::EscapedOverflow)?
            };
            if !continuation {
                return Ok(value);
            }
        }
        Err(OamdBodyAssemblyError::EscapedUnterminated)
    }

    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>, OamdBodyAssemblyError> {
        let payload_bits = count
            .checked_mul(8)
            .ok_or(OamdBodyAssemblyError::SectionOverflow)?;
        if payload_bits > self.remaining_bits() {
            return Err(OamdBodyAssemblyError::SectionOverflow);
        }
        let mut payload = Vec::new();
        payload
            .try_reserve_exact(count)
            .map_err(|_| OamdBodyAssemblyError::SectionOverflow)?;
        for _ in 0..count {
            payload
                .push(u8::try_from(self.read(8)?).map_err(|_| OamdBodyAssemblyError::Truncated)?);
        }
        Ok(payload)
    }

    fn finish_zero_padding(&mut self) -> Result<u8, OamdBodyAssemblyError> {
        let remaining = self.remaining_bits();
        if remaining > 7 {
            return Err(OamdBodyAssemblyError::UnconsumedFields);
        }
        let padding_bits =
            u8::try_from(remaining).map_err(|_| OamdBodyAssemblyError::NonzeroPadding)?;
        while self.bit_position < self.bit_limit {
            if self.read(1)? != 0 {
                return Err(OamdBodyAssemblyError::NonzeroPadding);
            }
        }
        Ok(padding_bits)
    }
}
