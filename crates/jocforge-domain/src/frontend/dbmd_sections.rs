// pattern: Functional Core
//! Selected F01E DBMD section framing, authored mode, and per-track fields.
//!
//! This parser consumes an already supplied `dbmd` payload. It performs no file
//! I/O, ADM graph resolution, event sampling, or control-row installation.

use sha2::{Digest, Sha256};

const DBMD_PREFIX_BYTES: usize = 4;
const SECTION9_ID: u8 = 9;
const SECTION10_ID: u8 = 10;
const SECTION9_LENGTH: usize = 248;
const SECTION10_MINIMUM_LENGTH: usize = 142;
const SECTION10_MAGIC: u32 = 0xF872_6FBD;
const MAX_TRACK_COUNT: usize = 2_048;

/// F01E-local parsing errors. They are not native error status values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DbmdSectionError {
    HeaderLength,
    SectionHeaderTruncated,
    SectionExtentTruncated,
    ChecksumMismatch,
    DuplicateSection,
    NonzeroTerminatorTail,
    RequiredSectionMissing,
    Section9Length,
    AuthoredModeUnsupported,
    Section10Length,
    Section10Magic,
    TrackTableInvalid,
}

impl DbmdSectionError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::HeaderLength => "DBMD_HEADER",
            Self::SectionHeaderTruncated => "DBMD_SECTION_HEADER",
            Self::SectionExtentTruncated => "DBMD_SECTION_EXTENT",
            Self::ChecksumMismatch => "DBMD_CHECKSUM",
            Self::DuplicateSection => "DBMD_DUPLICATE_SECTION",
            Self::NonzeroTerminatorTail => "DBMD_TERMINATOR",
            Self::RequiredSectionMissing => "DBMD_SECTION_REQUIRED",
            Self::Section9Length | Self::AuthoredModeUnsupported => "DBMD_AUTHORED_MODE",
            Self::Section10Length | Self::TrackTableInvalid => "DBMD_TRACK_TABLE",
            Self::Section10Magic => "DBMD_SECTION10_MAGIC",
        }
    }
}

/// Per-physical-track fields decoded from the two section-10 arrays.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DbmdTrackFields {
    track_index: u16,
    enum48: i8,
    enum52: i8,
    flag64: u8,
    raw_flag_low2: u8,
    raw_packed: u8,
}

impl DbmdTrackFields {
    pub const fn track_index(self) -> u16 {
        self.track_index
    }

    pub const fn enum48(self) -> i8 {
        self.enum48
    }

    pub const fn enum52(self) -> i8 {
        self.enum52
    }

    pub const fn flag64(self) -> u8 {
        self.flag64
    }

    pub const fn raw_flag_low2(self) -> u8 {
        self.raw_flag_low2
    }

    pub const fn raw_packed(self) -> u8 {
        self.raw_packed
    }
}

/// Selected decoded DBMD values. Track order is physical track index order;
/// downstream graph installation must join by the physical host identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedDbmdSections {
    warm_trim_mode: u8,
    authored_source_mode_byte: u8,
    surround_ex_mode: u8,
    track_fields: Vec<DbmdTrackFields>,
    payload_sha256: [u8; 32],
}

impl SelectedDbmdSections {
    pub const fn warm_trim_mode(&self) -> u8 {
        self.warm_trim_mode
    }

    pub const fn authored_source_mode_byte(&self) -> u8 {
        self.authored_source_mode_byte
    }

    pub const fn surround_ex_mode(&self) -> u8 {
        self.surround_ex_mode
    }

    pub fn track_fields(&self) -> &[DbmdTrackFields] {
        &self.track_fields
    }

    pub const fn payload_sha256(&self) -> &[u8; 32] {
        &self.payload_sha256
    }

    /// Looks up decoded fields by physical track index, not pack or object ordinal.
    pub fn fields_for_physical_host(&self, host_channel_index: u16) -> Option<DbmdTrackFields> {
        self.track_fields
            .iter()
            .copied()
            .find(|fields| fields.track_index() == host_channel_index)
    }
}

/// Parses the selected DBMD payload and returns authored mode plus physical
/// track fields. Input begins with the four-byte DBMD prefix; section parsing
/// starts at byte 4 as specified.
pub fn parse_selected_dbmd_sections(
    payload: &[u8],
) -> Result<SelectedDbmdSections, DbmdSectionError> {
    if payload.len() < DBMD_PREFIX_BYTES {
        return Err(DbmdSectionError::HeaderLength);
    }
    let first_word = u32::from_le_bytes(
        payload[..DBMD_PREFIX_BYTES]
            .try_into()
            .map_err(|_| DbmdSectionError::HeaderLength)?,
    );
    let payload_length_minus_prefix = payload
        .len()
        .checked_sub(DBMD_PREFIX_BYTES)
        .ok_or(DbmdSectionError::HeaderLength)?;
    if u64::from(first_word) == payload_length_minus_prefix as u64 {
        return Err(DbmdSectionError::HeaderLength);
    }

    let mut offset = DBMD_PREFIX_BYTES;
    let mut seen_sections = [false; 256];
    let mut section9: Option<&[u8]> = None;
    let mut section10: Option<&[u8]> = None;
    while offset < payload.len() {
        let id = *payload
            .get(offset)
            .ok_or(DbmdSectionError::SectionHeaderTruncated)?;
        if id == 0 {
            if payload[offset + 1..].iter().any(|byte| *byte != 0) {
                return Err(DbmdSectionError::NonzeroTerminatorTail);
            }
            break;
        }

        let header_end = offset
            .checked_add(3)
            .ok_or(DbmdSectionError::SectionHeaderTruncated)?;
        let length_bytes = payload
            .get(offset + 1..header_end)
            .ok_or(DbmdSectionError::SectionHeaderTruncated)?;
        let length = usize::from(u16::from_le_bytes(
            length_bytes
                .try_into()
                .map_err(|_| DbmdSectionError::SectionHeaderTruncated)?,
        ));
        let body_end = header_end
            .checked_add(length)
            .ok_or(DbmdSectionError::SectionExtentTruncated)?;
        let section_end = body_end
            .checked_add(1)
            .ok_or(DbmdSectionError::SectionExtentTruncated)?;
        let body = payload
            .get(header_end..body_end)
            .ok_or(DbmdSectionError::SectionExtentTruncated)?;
        let checksum = *payload
            .get(body_end)
            .ok_or(DbmdSectionError::SectionExtentTruncated)?;
        if section_checksum(length as u16, body) != checksum {
            return Err(DbmdSectionError::ChecksumMismatch);
        }

        let section_index = usize::from(id);
        if seen_sections[section_index] {
            return Err(DbmdSectionError::DuplicateSection);
        }
        seen_sections[section_index] = true;

        match id {
            SECTION9_ID => section9 = Some(body),
            SECTION10_ID => section10 = Some(body),
            _ => {}
        }
        offset = section_end;
    }

    let section9 = section9.ok_or(DbmdSectionError::RequiredSectionMissing)?;
    let section10 = section10.ok_or(DbmdSectionError::RequiredSectionMissing)?;
    if section9.len() != SECTION9_LENGTH {
        return Err(DbmdSectionError::Section9Length);
    }
    if section10.len() < SECTION10_MINIMUM_LENGTH {
        return Err(DbmdSectionError::Section10Length);
    }

    let warm_trim_mode = section9[152] & 7;
    if warm_trim_mode > 3 {
        return Err(DbmdSectionError::AuthoredModeUnsupported);
    }
    let authored_source_mode_byte = warm_trim_mode + 1;
    let surround_ex_mode = if authored_source_mode_byte == 3 { 2 } else { 0 };

    if read_u32_le(section10, 0)? != SECTION10_MAGIC {
        return Err(DbmdSectionError::Section10Magic);
    }
    let track_count = usize::from(read_u16_le(section10, 4)?);
    if track_count > MAX_TRACK_COUNT {
        return Err(DbmdSectionError::TrackTableInvalid);
    }
    let second_array_start = SECTION10_MINIMUM_LENGTH
        .checked_add(track_count)
        .ok_or(DbmdSectionError::TrackTableInvalid)?;
    let track_table_end = second_array_start
        .checked_add(track_count)
        .ok_or(DbmdSectionError::TrackTableInvalid)?;
    if section10.len() < track_table_end {
        return Err(DbmdSectionError::TrackTableInvalid);
    }

    let first_array = &section10[SECTION10_MINIMUM_LENGTH..second_array_start];
    let second_array = &section10[second_array_start..track_table_end];
    let track_fields = first_array
        .iter()
        .copied()
        .zip(second_array.iter().copied())
        .enumerate()
        .map(|(track_index, (first, packed))| {
            let raw_flag_low2 = first & 3;
            let code48 = (packed >> 6) & 3;
            let code52 = packed & 7;
            DbmdTrackFields {
                track_index: u16::try_from(track_index).unwrap_or(u16::MAX),
                enum48: if code48 <= 1 {
                    i8::try_from(code48).unwrap_or_default()
                } else {
                    -1
                },
                enum52: if code52 <= 2 {
                    i8::try_from(code52).unwrap_or_default()
                } else {
                    -1
                },
                flag64: u8::from(raw_flag_low2 == 1),
                raw_flag_low2,
                raw_packed: packed,
            }
        })
        .collect();

    Ok(SelectedDbmdSections {
        warm_trim_mode,
        authored_source_mode_byte,
        surround_ex_mode,
        track_fields,
        payload_sha256: Sha256::digest(payload).into(),
    })
}

fn section_checksum(length: u16, body: &[u8]) -> u8 {
    let sum = body.iter().copied().fold(length as u8, u8::wrapping_add);
    sum.wrapping_neg()
}

fn read_u16_le(bytes: &[u8], offset: usize) -> Result<u16, DbmdSectionError> {
    let end = offset
        .checked_add(2)
        .ok_or(DbmdSectionError::Section10Length)?;
    let value = bytes
        .get(offset..end)
        .ok_or(DbmdSectionError::Section10Length)?;
    let array: [u8; 2] = value
        .try_into()
        .map_err(|_| DbmdSectionError::Section10Length)?;
    Ok(u16::from_le_bytes(array))
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32, DbmdSectionError> {
    let end = offset
        .checked_add(4)
        .ok_or(DbmdSectionError::Section10Length)?;
    let value = bytes
        .get(offset..end)
        .ok_or(DbmdSectionError::Section10Length)?;
    let array: [u8; 4] = value
        .try_into()
        .map_err(|_| DbmdSectionError::Section10Length)?;
    Ok(u32::from_le_bytes(array))
}
