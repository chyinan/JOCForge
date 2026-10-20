// pattern: Functional Core
//! Pure Type11 trim-body and selected DBMD default-trim projection.

use std::collections::BTreeMap;

const GROUP_COUNT: usize = 9;
const GROUP_WIDTH: usize = 5;
const DEFAULT_GROUP: [i32; GROUP_WIDTH] = [16, 16, 16, 0, 0];
const GROUP_KEY_ORDER: [usize; GROUP_COUNT] = [0, 3, 6, 1, 4, 7, 2, 5, 8];
const DBMD_SECTION9_LENGTH: usize = 248;
const DBMD_SECTION10_MIN_LENGTH: usize = 142;
const DBMD_TRIM_RECORD_START: usize = 7;
const DBMD_TRIM_RECORD_LENGTH: usize = 15;
const MAX_TYPE11_ROW_COUNT: u32 = 159;
const DBMD_SUPPLEMENTAL_MAGIC: u32 = 0xf872_6fbd;

/// One value-only Type11 trim configuration for a supplied row count.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trim {
    /// Two-bit trim mode.
    pub trim_mode: u8,
    /// One-bit flags for the nine groups.
    pub group_flags: [u8; GROUP_COUNT],
    /// Five integer trim fields for each group.
    pub groups: [[i32; GROUP_WIDTH]; GROUP_COUNT],
    /// One-bit flags for the caller-supplied row count.
    pub row_flags: Vec<u8>,
}

/// Encoded trim body plus its semantic bit length and selected group mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedTrimBody {
    /// Body bytes with zero padding to the next byte boundary.
    pub body_bytes: Vec<u8>,
    /// Number of meaningful bits before byte padding.
    pub valid_bits: u32,
    /// Number of zero bits appended to reach the byte boundary.
    pub padding_bits: u8,
    /// Derived two-bit group mode.
    pub group_mode: u8,
}

/// Rejections defined by the selected trim-section codec contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrimSectionError {
    /// Row count or value container has an unsupported shape.
    TrimShape,
    /// A trim mode or one-bit flag is outside its declared domain.
    TrimDomain,
    /// A changed gain field cannot be represented as U4.
    TrimU4,
    /// A changed balance field is outside the signed range.
    TrimSignedBalance,
    /// A trim body ends before all required semantic bits are present.
    TrimTruncated,
    /// A trim body has a nonzero reserved field.
    TrimReserved,
    /// A trim body declares the unsupported group mode three.
    TrimGroupMode,
    /// A trim body has nonzero final byte-padding bits.
    TrimNonzeroPadding,
    /// A DBMD header has the selected rejected length relation.
    DbmdHeader,
    /// A DBMD section header is truncated.
    DbmdTruncatedHeader,
    /// A DBMD section body or checksum is truncated.
    DbmdTruncatedPayload,
    /// A DBMD section checksum is invalid.
    DbmdChecksum,
    /// A DBMD section kind appears more than once.
    DbmdDuplicateSection,
    /// A DBMD terminator is followed by nonzero bytes.
    DbmdTerminatorTail,
    /// Required DBMD sections 9 and 10 are not both present.
    DbmdRequiredSections,
    /// A required DBMD section has an unsupported size or magic value.
    DbmdSupplemental,
    /// The selected section 9 warm mode is unsupported.
    DbmdWarmMode,
    /// A default-trim record contains nonzero gain or balance data.
    DbmdNondefaultTrim,
    /// A selected DBMD-to-Type11 projection count is outside 1..=159.
    DbmdRowCount,
}

/// Encodes one standalone trim body using the selected MSB-first Type11 rules.
pub fn encode_trim_body(trim: &Trim, row_count: u32) -> Result<EncodedTrimBody, TrimSectionError> {
    let row_count = checked_row_count(row_count)?;
    validate_trim_shape_and_flags(trim, row_count)?;

    let all_groups_skipped = trim.group_flags.iter().all(|flag| *flag == 1);
    let all_default_groups = trim.group_flags.iter().all(|flag| *flag == 0)
        && trim.groups.iter().all(|group| *group == DEFAULT_GROUP);
    let group_mode = if all_groups_skipped {
        0
    } else if all_default_groups {
        1
    } else {
        2
    };

    let mut writer = TrimBitWriter::default();
    writer.write(u32::from(trim.trim_mode), 2);
    writer.write(0, 2);
    writer.write(u32::from(group_mode), 2);

    if group_mode == 2 {
        for group_index in 0..GROUP_COUNT {
            let group_flag = trim.group_flags[group_index];
            writer.write(u32::from(group_flag), 1);
            if group_flag == 1 {
                continue;
            }

            let group = trim.groups[group_index];
            if group == DEFAULT_GROUP {
                writer.write(1, 1);
                continue;
            }

            writer.write(0, 1);
            let mut mask = 0_u32;
            for (field_index, value) in group.iter().enumerate() {
                if *value != DEFAULT_GROUP[field_index] {
                    mask |= 1 << field_index;
                }
            }
            writer.write(mask, 5);

            for (field_index, value) in group.iter().copied().enumerate() {
                if mask & (1 << field_index) == 0 {
                    continue;
                }
                if field_index < 3 {
                    if !(0..=15).contains(&value) {
                        return Err(TrimSectionError::TrimU4);
                    }
                    writer.write(
                        u32::try_from(value).map_err(|_| TrimSectionError::TrimU4)?,
                        4,
                    );
                } else {
                    if !(-16..=16).contains(&value) || value == 0 {
                        return Err(TrimSectionError::TrimSignedBalance);
                    }
                    writer.write(u32::from(value > 0), 1);
                    writer.write(value.unsigned_abs() - 1, 4);
                }
            }
        }
    }

    let row_presence = trim.row_flags.iter().any(|flag| *flag != 0);
    writer.write(u32::from(row_presence), 1);
    if row_presence {
        for flag in &trim.row_flags {
            writer.write(u32::from(*flag), 1);
        }
    }

    let (body_bytes, valid_bits, padding_bits) = writer.finish()?;
    Ok(EncodedTrimBody {
        body_bytes,
        valid_bits,
        padding_bits,
        group_mode,
    })
}

/// Decodes one standalone trim body and requires zero byte-alignment padding.
pub fn decode_trim_body(body_bytes: &[u8], row_count: u32) -> Result<Trim, TrimSectionError> {
    let row_count = checked_row_count(row_count)?;
    let mut reader = TrimBitReader::new(body_bytes)?;
    let trim_mode = u8::try_from(reader.read(2)?).map_err(|_| TrimSectionError::TrimDomain)?;
    if reader.read(2)? != 0 {
        return Err(TrimSectionError::TrimReserved);
    }

    let group_mode = reader.read(2)?;
    if group_mode == 3 {
        return Err(TrimSectionError::TrimGroupMode);
    }

    let (group_flags, groups) = match group_mode {
        0 => ([1; GROUP_COUNT], [DEFAULT_GROUP; GROUP_COUNT]),
        1 => ([0; GROUP_COUNT], [DEFAULT_GROUP; GROUP_COUNT]),
        2 => decode_explicit_groups(&mut reader)?,
        _ => return Err(TrimSectionError::TrimGroupMode),
    };

    let row_presence = reader.read(1)? != 0;
    if row_presence && reader.remaining_bits() < row_count {
        return Err(TrimSectionError::TrimTruncated);
    }
    let mut row_flags = zero_row_flags(row_count)?;
    if row_presence {
        for flag in &mut row_flags {
            *flag = u8::try_from(reader.read(1)?).map_err(|_| TrimSectionError::TrimDomain)?;
        }
    }
    reader.finish_zero_padding()?;

    Ok(Trim {
        trim_mode,
        group_flags,
        groups,
        row_flags,
    })
}

/// Parses DBMD section framing after its four-byte little-endian header.
pub fn parse_dbmd_sections(chunk_bytes: &[u8]) -> Result<BTreeMap<u8, Vec<u8>>, TrimSectionError> {
    if chunk_bytes.len() < 4 {
        return Err(TrimSectionError::DbmdTruncatedHeader);
    }

    let declared_header = u32::from_le_bytes(
        chunk_bytes[..4]
            .try_into()
            .map_err(|_| TrimSectionError::DbmdTruncatedHeader)?,
    );
    let content_length =
        u64::try_from(chunk_bytes.len() - 4).map_err(|_| TrimSectionError::DbmdHeader)?;
    if u64::from(declared_header) == content_length {
        return Err(TrimSectionError::DbmdHeader);
    }

    let mut sections = BTreeMap::new();
    let mut position = 4;
    while position < chunk_bytes.len() {
        let section_kind = chunk_bytes[position];
        position += 1;
        if section_kind == 0 {
            if chunk_bytes[position..].iter().any(|byte| *byte != 0) {
                return Err(TrimSectionError::DbmdTerminatorTail);
            }
            return Ok(sections);
        }

        if chunk_bytes.len() - position < 2 {
            return Err(TrimSectionError::DbmdTruncatedHeader);
        }
        let body_length = usize::from(u16::from_le_bytes([
            chunk_bytes[position],
            chunk_bytes[position + 1],
        ]));
        position += 2;
        let body_end = position
            .checked_add(body_length)
            .ok_or(TrimSectionError::DbmdTruncatedPayload)?;
        let checksum_end = body_end
            .checked_add(1)
            .ok_or(TrimSectionError::DbmdTruncatedPayload)?;
        if checksum_end > chunk_bytes.len() {
            return Err(TrimSectionError::DbmdTruncatedPayload);
        }

        let body = &chunk_bytes[position..body_end];
        let body_sum: u32 = body.iter().map(|byte| u32::from(*byte)).sum();
        let checksum = (0_u32.wrapping_sub(body_length as u32 + body_sum) & 0xff) as u8;
        if chunk_bytes[body_end] != checksum {
            return Err(TrimSectionError::DbmdChecksum);
        }
        if sections.contains_key(&section_kind) {
            return Err(TrimSectionError::DbmdDuplicateSection);
        }

        sections.insert(section_kind, body.to_vec());
        position = checksum_end;
    }
    Ok(sections)
}

/// Projects the selected default trim from already framed DBMD sections 9 and 10.
pub fn selected_trim_from_dbmd_sections(
    sections: &BTreeMap<u8, Vec<u8>>,
    row_count: u32,
) -> Result<Trim, TrimSectionError> {
    let row_count = checked_dbmd_row_count(row_count)?;
    let section_nine = sections
        .get(&9)
        .ok_or(TrimSectionError::DbmdRequiredSections)?;
    let section_ten = sections
        .get(&10)
        .ok_or(TrimSectionError::DbmdRequiredSections)?;
    if section_nine.len() != DBMD_SECTION9_LENGTH
        || section_ten.len() < DBMD_SECTION10_MIN_LENGTH
        || u32::from_le_bytes(
            section_ten[..4]
                .try_into()
                .map_err(|_| TrimSectionError::DbmdSupplemental)?,
        ) != DBMD_SUPPLEMENTAL_MAGIC
    {
        return Err(TrimSectionError::DbmdSupplemental);
    }

    let trim_mode = section_nine[152] & 0x07;
    if trim_mode > 3 {
        return Err(TrimSectionError::DbmdWarmMode);
    }

    let mut record_flags = [0_u8; GROUP_COUNT];
    for (record_ordinal, record_flag) in record_flags.iter_mut().enumerate() {
        let record_start = DBMD_TRIM_RECORD_START + record_ordinal * DBMD_TRIM_RECORD_LENGTH;
        *record_flag = section_ten[record_start] & 1;
        if *record_flag == 1 {
            continue;
        }

        for gain_index in 0..3 {
            let gain_start = record_start + 1 + gain_index * 4;
            let gain_bits = u32::from_le_bytes(
                section_ten[gain_start..gain_start + 4]
                    .try_into()
                    .map_err(|_| TrimSectionError::DbmdSupplemental)?,
            );
            if f32::from_bits(gain_bits) != 0.0 {
                return Err(TrimSectionError::DbmdNondefaultTrim);
            }
        }
        if section_ten[record_start + 13] & 0x7f != 0 || section_ten[record_start + 14] & 0x7f != 0
        {
            return Err(TrimSectionError::DbmdNondefaultTrim);
        }
    }

    let mut intermediate_flags = [0_u8; GROUP_COUNT];
    for (record_ordinal, group_key) in GROUP_KEY_ORDER.iter().copied().enumerate() {
        intermediate_flags[group_key] = record_flags[record_ordinal];
    }
    let mut group_flags = [0_u8; GROUP_COUNT];
    for (intermediate_key, final_key) in GROUP_KEY_ORDER.iter().copied().enumerate() {
        group_flags[final_key] = intermediate_flags[intermediate_key];
    }

    Ok(Trim {
        trim_mode,
        group_flags,
        groups: [DEFAULT_GROUP; GROUP_COUNT],
        row_flags: vec![0; row_count],
    })
}

fn checked_row_count(row_count: u32) -> Result<usize, TrimSectionError> {
    usize::try_from(row_count).map_err(|_| TrimSectionError::TrimShape)
}

fn checked_dbmd_row_count(row_count: u32) -> Result<usize, TrimSectionError> {
    if row_count == 0 || row_count > MAX_TYPE11_ROW_COUNT {
        return Err(TrimSectionError::DbmdRowCount);
    }
    checked_row_count(row_count)
}

fn zero_row_flags(row_count: usize) -> Result<Vec<u8>, TrimSectionError> {
    let mut row_flags = Vec::new();
    row_flags
        .try_reserve_exact(row_count)
        .map_err(|_| TrimSectionError::TrimShape)?;
    row_flags.resize(row_count, 0);
    Ok(row_flags)
}

fn validate_trim_shape_and_flags(trim: &Trim, row_count: usize) -> Result<(), TrimSectionError> {
    if trim.row_flags.len() != row_count {
        return Err(TrimSectionError::TrimShape);
    }
    if trim.trim_mode > 3
        || trim.group_flags.iter().any(|flag| *flag > 1)
        || trim.row_flags.iter().any(|flag| *flag > 1)
    {
        return Err(TrimSectionError::TrimDomain);
    }
    Ok(())
}

fn decode_explicit_groups(
    reader: &mut TrimBitReader<'_>,
) -> Result<([u8; GROUP_COUNT], [[i32; GROUP_WIDTH]; GROUP_COUNT]), TrimSectionError> {
    let mut group_flags = [0_u8; GROUP_COUNT];
    let mut groups = [DEFAULT_GROUP; GROUP_COUNT];
    for group_index in 0..GROUP_COUNT {
        let group_flag = u8::try_from(reader.read(1)?).map_err(|_| TrimSectionError::TrimDomain)?;
        group_flags[group_index] = group_flag;
        if group_flag == 1 || reader.read(1)? == 1 {
            continue;
        }

        let mask = reader.read(5)?;
        for (field_index, field) in groups[group_index].iter_mut().enumerate() {
            if mask & (1 << field_index) == 0 {
                continue;
            }
            *field = if field_index < 3 {
                i32::try_from(reader.read(4)?).map_err(|_| TrimSectionError::TrimDomain)?
            } else {
                let positive = reader.read(1)? != 0;
                let magnitude =
                    i32::try_from(reader.read(4)? + 1).map_err(|_| TrimSectionError::TrimDomain)?;
                if positive { magnitude } else { -magnitude }
            };
        }
    }
    Ok((group_flags, groups))
}

#[derive(Default)]
struct TrimBitWriter {
    bytes: Vec<u8>,
    bit_position: usize,
}

impl TrimBitWriter {
    fn write(&mut self, value: u32, width: u8) {
        for shift in (0..width).rev() {
            let byte_index = self.bit_position / 8;
            if byte_index == self.bytes.len() {
                self.bytes.push(0);
            }
            let bit = (value >> shift) & 1;
            self.bytes[byte_index] |= (bit as u8) << (7 - self.bit_position % 8);
            self.bit_position += 1;
        }
    }

    fn finish(mut self) -> Result<(Vec<u8>, u32, u8), TrimSectionError> {
        let padding_bits = ((8 - self.bit_position % 8) % 8) as u8;
        for _ in 0..padding_bits {
            self.write(0, 1);
        }
        let valid_bits = u32::try_from(self.bit_position - usize::from(padding_bits))
            .map_err(|_| TrimSectionError::TrimShape)?;
        Ok((self.bytes, valid_bits, padding_bits))
    }
}

struct TrimBitReader<'a> {
    bytes: &'a [u8],
    bit_position: usize,
    bit_limit: usize,
}

impl<'a> TrimBitReader<'a> {
    fn new(bytes: &'a [u8]) -> Result<Self, TrimSectionError> {
        let bit_limit = bytes
            .len()
            .checked_mul(8)
            .ok_or(TrimSectionError::TrimShape)?;
        Ok(Self {
            bytes,
            bit_position: 0,
            bit_limit,
        })
    }

    fn read(&mut self, width: u8) -> Result<u32, TrimSectionError> {
        let end = self
            .bit_position
            .checked_add(usize::from(width))
            .ok_or(TrimSectionError::TrimTruncated)?;
        if end > self.bit_limit {
            return Err(TrimSectionError::TrimTruncated);
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

    fn finish_zero_padding(&mut self) -> Result<(), TrimSectionError> {
        if self.bit_limit - self.bit_position > 7 {
            return Err(TrimSectionError::TrimShape);
        }
        while self.bit_position < self.bit_limit {
            if self.read(1)? != 0 {
                return Err(TrimSectionError::TrimNonzeroPadding);
            }
        }
        Ok(())
    }
}
