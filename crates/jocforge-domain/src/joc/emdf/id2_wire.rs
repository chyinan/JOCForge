// pattern: Functional Core
//! Stateless F13D2 profile-code mapping, local Type11 LFE-header read, and ID2 wire body.

use crate::{joc::emdf::metadata_owner::MetadataPayloadBody, timeline::CodecSourceIndex};

const ID2_BODY_VALID_BITS: u8 = 21;
const ID2_BODY_BYTES: usize = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Id2WireError {
    UnsupportedProfile,
    InvalidType11Header,
    FieldOutOfRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Id2ProfileCode(u8);

impl Id2ProfileCode {
    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Id2Body {
    pub bytes: [u8; ID2_BODY_BYTES],
    pub valid_bits: u8,
}

impl Id2Body {
    pub fn into_metadata_body(self, writer_generation: CodecSourceIndex) -> MetadataPayloadBody {
        MetadataPayloadBody {
            writer_generation,
            bytes: self.bytes.to_vec(),
        }
    }
}

/// Maps one exact external profile name to its selected ID2 U3 code.
pub fn map_drc_profile(profile: &str) -> Result<Id2ProfileCode, Id2WireError> {
    let code = match profile {
        "film_light" => 2,
        "film_standard" => 1,
        "music_light" => 4,
        "music_standard" => 3,
        "speech" => 5,
        "none" => 0,
        _ => return Err(Id2WireError::UnsupportedProfile),
    };
    Ok(Id2ProfileCode(code))
}

/// Builds the selected ID2 body from external profile labels and one Type11 header prefix.
///
/// The labels are only translated to local wire codes here; this function does not
/// implement or admit a programme-level line/RF DRC producer.
pub fn build_id2_body(
    line_profile: &str,
    rf_profile: &str,
    type11_header: &[u8],
) -> Result<Id2Body, Id2WireError> {
    let line_code = map_drc_profile(line_profile)?.value();
    let rf_code = map_drc_profile(rf_profile)?.value();
    let lfe_flag = read_type11_lfe_flag(type11_header)?;
    serialize_id2_body(
        1,
        1,
        u32::from(lfe_flag),
        u32::from(line_code),
        u32::from(rf_code),
    )
}

/// Serializes the five explicit configuration values into the selected 21-bit ID2 body.
pub fn serialize_id2_body(
    phase_indicator: u32,
    format_mask: u32,
    lfe_mask: u32,
    line_code: u32,
    rf_code: u32,
) -> Result<Id2Body, Id2WireError> {
    if line_code > 7 || rf_code > 7 {
        return Err(Id2WireError::FieldOutOfRange);
    }

    let phase_bit = u32::from(phase_indicator != 0);
    let lfe_bit = u32::from((format_mask & lfe_mask) != 0);
    let fields = [
        (0, 2), // version/mode
        (0, 1), // optional A
        (0, 1), // optional B
        (0, 1), // optional C
        (1, 1), // phase block present
        (0, 1), // phase reserved
        (phase_bit, 1),
        (lfe_bit, 1),
        (0, 2), // reserved
        (1, 1), // DRC profile present
        (line_code, 3),
        (rf_code, 3),
        (0, 1), // trailing optional A
        (0, 1), // trailing optional B
        (0, 1), // trailing optional C
    ];

    let mut bytes = [0; ID2_BODY_BYTES];
    let mut bit_cursor = 0_usize;
    for (value, width) in fields {
        write_bits(&mut bytes, &mut bit_cursor, value, width);
    }

    debug_assert_eq!(bit_cursor, usize::from(ID2_BODY_VALID_BITS));
    Ok(Id2Body {
        bytes,
        valid_bits: ID2_BODY_VALID_BITS,
    })
}

fn read_type11_lfe_flag(header: &[u8]) -> Result<bool, Id2WireError> {
    let mut reader = BitReader::new(header);
    let version = reader.read(2).ok_or(Id2WireError::InvalidType11Header)?;
    if version != 0 {
        return Err(Id2WireError::InvalidType11Header);
    }

    let count_code = reader.read(5).ok_or(Id2WireError::InvalidType11Header)?;
    let count = if count_code < 31 {
        count_code + 1
    } else {
        32 + reader.read(7).ok_or(Id2WireError::InvalidType11Header)?
    };
    let shortcut = reader.read(1).ok_or(Id2WireError::InvalidType11Header)?;
    if shortcut != 1 {
        return Err(Id2WireError::InvalidType11Header);
    }
    let lfe_flag = reader.read(1).ok_or(Id2WireError::InvalidType11Header)?;
    if count < lfe_flag {
        return Err(Id2WireError::InvalidType11Header);
    }
    Ok(lfe_flag != 0)
}

fn write_bits(bytes: &mut [u8; ID2_BODY_BYTES], cursor: &mut usize, value: u32, width: u8) {
    for shift in (0..width).rev() {
        let bit = (value >> shift) & 1;
        let byte_index = *cursor / 8;
        let bit_index = 7 - (*cursor % 8);
        bytes[byte_index] |= u8::try_from(bit).expect("one-bit value fits u8") << bit_index;
        *cursor += 1;
    }
}

struct BitReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> BitReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn read(&mut self, width: u8) -> Option<u32> {
        let end = self.position.checked_add(usize::from(width))?;
        let available = self.bytes.len().checked_mul(8)?;
        if end > available {
            return None;
        }

        let mut value = 0_u32;
        while self.position < end {
            let byte_index = self.position / 8;
            let bit_index = 7 - (self.position % 8);
            value = (value << 1) | u32::from((self.bytes[byte_index] >> bit_index) & 1);
            self.position += 1;
        }
        Some(value)
    }
}
