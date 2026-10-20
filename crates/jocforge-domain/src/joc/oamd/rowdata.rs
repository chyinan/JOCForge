// pattern: Functional Core
//! Single-frame OAMD rowdata payload encoding from the isolated F05O rules.

/// The kind of one OAMD row supplied to the rowdata writer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OamdRowKind {
    /// A low-frequency-effects row. It is legal only at index zero.
    Lfe,
    /// An object row with Q15 XYZ coordinates.
    Object,
}

/// Timing fields for one OAMD metadata frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OamdTiming {
    /// Base mode value.
    pub base: u8,
    /// Position within the metadata frame.
    pub position: u8,
    /// Duration value in samples.
    pub duration: u16,
    /// Precision value.
    pub precision: u8,
}

/// Explicit input values for one row in the OAMD rowdata payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OamdRow {
    /// Row kind.
    pub kind: OamdRowKind,
    /// Whether the row is muted.
    pub mute: bool,
    /// Input gain value used by the row's gain coding mode.
    pub gain: u32,
    /// Input spread value. Unmuted rows require full spread.
    pub spread: u32,
    /// Object coordinates in the selected Q15 input domain. Ignored for LFE or muted rows.
    pub xyz_q15: [i32; 3],
    /// Component code written for object rows.
    pub component: u8,
    /// Whether object Z is valid.
    pub z_valid: bool,
    /// Diffuse flag for object rows.
    pub diffuse: bool,
}

/// Local rejection categories for the bounded OAMD rowdata writer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OamdRowdataError {
    /// Input shape or a value lies outside the written field's domain.
    InvalidInput,
    /// The input requests a branch excluded by the selected OAMD route.
    UnsupportedProfile,
    /// An escaped integer is negative, overlong, truncated, or malformed.
    InvalidEscapedInteger,
}

/// A rowdata payload and its meaningful bits before byte padding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EncodedRowdataPayload {
    /// Payload bytes, including the zero control bit and byte padding.
    pub bytes: Vec<u8>,
    /// Meaningful bits through the final row trailing-extension bit.
    pub valid_bits: u32,
}

/// Converts one integer XYZ point into the three signed wire-code values.
///
/// The first two outputs are XY codes in `0..=62`; the last is a signed Z code
/// in `-15..=15`. No floating-point rounding is involved.
pub fn encode_xyz_codes(xyz_q15: [i32; 3]) -> Result<[i16; 3], OamdRowdataError> {
    let x = encode_xy_code(xyz_q15[0])?;
    let y = encode_xy_code(xyz_q15[1])?;
    let z = encode_z_code(xyz_q15[2])?;
    Ok([x, y, z])
}

/// Encodes one nonnegative escaped integer as bytes and its exact bit length.
///
/// The maximum accepted value is the four-group bijective base-16 limit,
/// 69,903. Negative and larger values are rejected.
pub fn encode_escaped_integer(value: i64) -> Result<(Vec<u8>, u8), OamdRowdataError> {
    if value < 0 {
        return Err(OamdRowdataError::InvalidEscapedInteger);
    }
    let value = u32::try_from(value).map_err(|_| OamdRowdataError::InvalidEscapedInteger)?;
    let mut writer = BitWriter::default();
    write_escaped_value(&mut writer, value)?;
    let (bytes, valid_bits) = writer.finish();
    Ok((
        bytes,
        u8::try_from(valid_bits).map_err(|_| OamdRowdataError::InvalidEscapedInteger)?,
    ))
}

/// Decodes one escaped integer from its exact bit length, requiring zero byte padding.
pub fn decode_escaped_integer(bytes: &[u8], valid_bits: u8) -> Result<u64, OamdRowdataError> {
    let valid_bits = usize::from(valid_bits);
    if valid_bits == 0 || valid_bits > 20 || bytes.len() != valid_bits.div_ceil(8) {
        return Err(OamdRowdataError::InvalidEscapedInteger);
    }
    let used_bits_in_last_byte = valid_bits % 8;
    if used_bits_in_last_byte != 0 {
        let padding_mask = (1_u8 << (8 - used_bits_in_last_byte)) - 1;
        if bytes.last().is_some_and(|byte| byte & padding_mask != 0) {
            return Err(OamdRowdataError::InvalidEscapedInteger);
        }
    }

    let mut reader = BitReader::new(bytes, valid_bits);
    let mut value = 0_u64;
    for group_index in 0..4 {
        let digit = u64::from(reader.read(4)?);
        let continues = reader.read(1)? == 1;
        value = if group_index == 0 {
            digit
        } else {
            value
                .checked_add(1)
                .and_then(|value| value.checked_mul(16))
                .and_then(|value| value.checked_add(digit))
                .ok_or(OamdRowdataError::InvalidEscapedInteger)?
        };
        if !continues {
            if reader.position() != valid_bits {
                return Err(OamdRowdataError::InvalidEscapedInteger);
            }
            return Ok(value);
        }
    }
    Err(OamdRowdataError::InvalidEscapedInteger)
}

/// Encodes one rowdata section payload, including its zero control bit and byte padding.
pub fn encode_rowdata_payload(
    timing: &OamdTiming,
    rows: &[OamdRow],
) -> Result<Vec<u8>, OamdRowdataError> {
    encode_rowdata_payload_with_bits(*timing, rows).map(|payload| payload.bytes)
}

pub(crate) fn encode_rowdata_payload_with_bits(
    timing: OamdTiming,
    rows: &[OamdRow],
) -> Result<EncodedRowdataPayload, OamdRowdataError> {
    if !(1..=159).contains(&rows.len()) {
        return Err(OamdRowdataError::UnsupportedProfile);
    }

    let mut writer = BitWriter::default();
    writer.write(0, 1)?;
    encode_timing(&mut writer, timing)?;

    let mut previous_unmuted_gain = None;
    for (row_index, row) in rows.iter().enumerate() {
        if row.kind == OamdRowKind::Lfe && row_index != 0 {
            return Err(OamdRowdataError::UnsupportedProfile);
        }
        writer.write(u32::from(row.mute), 1)?;
        if row.mute {
            writer.write(0, 1)?;
            previous_unmuted_gain = None;
            continue;
        }

        if previous_unmuted_gain == Some(row.gain) {
            writer.write(3, 2)?;
        } else if (3866..=4337).contains(&row.gain) {
            writer.write(0, 2)?;
        } else if row.gain == 0 {
            writer.write(1, 2)?;
        } else {
            return Err(OamdRowdataError::UnsupportedProfile);
        }
        previous_unmuted_gain = Some(row.gain);

        if row.spread != 32767 {
            return Err(OamdRowdataError::UnsupportedProfile);
        }
        writer.write(1, 1)?;

        if row.kind == OamdRowKind::Object {
            if row.component > 7 {
                return Err(OamdRowdataError::UnsupportedProfile);
            }
            let [x, y, z] = encode_xyz_codes(row.xyz_q15)?;
            writer.write(
                u32::try_from(x).map_err(|_| OamdRowdataError::InvalidInput)?,
                6,
            )?;
            writer.write(
                u32::try_from(y).map_err(|_| OamdRowdataError::InvalidInput)?,
                6,
            )?;
            writer.write(u32::from(z >= 0), 1)?;
            writer.write(u32::from(z.unsigned_abs()), 4)?;
            writer.write(0, 1)?;
            writer.write(u32::from(row.component), 3)?;
            writer.write(u32::from(row.z_valid), 1)?;
            writer.write(0, 2)?;
            writer.write(0, 1)?;
            writer.write(u32::from(row.diffuse), 1)?;
        }
        writer.write(0, 1)?;
    }
    let (bytes, valid_bits) = writer.finish();
    Ok(EncodedRowdataPayload {
        bytes,
        valid_bits: u32::try_from(valid_bits).map_err(|_| OamdRowdataError::InvalidInput)?,
    })
}

/// Encodes a complete OAMD body containing one rowdata section and no trim section.
///
/// The input rows determine both `row_count` and whether the leading-LFE flag is set.
pub fn encode_oamd_body(
    timing: &OamdTiming,
    rows: &[OamdRow],
) -> Result<Vec<u8>, OamdRowdataError> {
    let rowdata_payload = encode_rowdata_payload(timing, rows)?;
    let row_count = rows.len();
    let leading_lfe = rows[0].kind == OamdRowKind::Lfe;

    let mut writer = BitWriter::default();
    writer.write(0, 2)?;
    if row_count < 32 {
        writer.write(
            u32::try_from(row_count - 1).map_err(|_| OamdRowdataError::InvalidInput)?,
            5,
        )?;
    } else {
        writer.write(31, 5)?;
        writer.write(
            u32::try_from(row_count - 32).map_err(|_| OamdRowdataError::InvalidInput)?,
            7,
        )?;
    }
    writer.write(1, 1)?;
    writer.write(u32::from(leading_lfe), 1)?;
    writer.write(0, 1)?;
    writer.write(1, 4)?;
    writer.write(1, 4)?;
    write_escaped_value(
        &mut writer,
        u32::try_from(rowdata_payload.len() - 1).map_err(|_| OamdRowdataError::InvalidInput)?,
    )?;
    for byte in rowdata_payload {
        writer.write(u32::from(byte), 8)?;
    }
    Ok(writer.finish().0)
}

fn encode_timing(writer: &mut BitWriter, timing: OamdTiming) -> Result<(), OamdRowdataError> {
    if timing.base > 31 || timing.position > 63 || timing.precision > 32 {
        return Err(OamdRowdataError::InvalidInput);
    }

    match timing.base {
        0 => writer.write(0, 2)?,
        8 => {
            writer.write(1, 2)?;
            writer.write(0, 2)?;
        }
        16 => {
            writer.write(1, 2)?;
            writer.write(1, 2)?;
        }
        18 => {
            writer.write(1, 2)?;
            writer.write(2, 2)?;
        }
        24 => {
            writer.write(1, 2)?;
            writer.write(3, 2)?;
        }
        base => {
            writer.write(2, 2)?;
            writer.write(u32::from(base), 5)?;
        }
    }

    writer.write(0, 3)?;
    writer.write(u32::from(timing.position), 6)?;
    match timing.duration {
        0 => writer.write(0, 2)?,
        512 => writer.write(1, 2)?,
        1536 => writer.write(2, 2)?,
        duration => {
            writer.write(3, 2)?;
            if let Some(ordinal) = duration_table_ordinal(duration) {
                writer.write(1, 1)?;
                writer.write(ordinal, 4)?;
            } else if duration <= 2047 {
                writer.write(0, 1)?;
                writer.write(u32::from(duration), 11)?;
            } else {
                return Err(OamdRowdataError::UnsupportedProfile);
            }
        }
    }
    if timing.precision == 32 {
        writer.write(1, 1)?;
    } else {
        writer.write(0, 1)?;
        writer.write(u32::from(timing.precision), 5)?;
    }
    Ok(())
}

fn duration_table_ordinal(duration: u16) -> Option<u32> {
    const TABLE: [u16; 16] = [
        32, 64, 128, 256, 320, 480, 1000, 1001, 1024, 1600, 1601, 1602, 1920, 2000, 2002, 2048,
    ];
    TABLE
        .iter()
        .position(|candidate| *candidate == duration)
        .and_then(|ordinal| u32::try_from(ordinal).ok())
}

fn write_escaped_value(writer: &mut BitWriter, value: u32) -> Result<(), OamdRowdataError> {
    let mut digits = [0_u8; 4];
    let mut digit_count = 0;
    let mut remaining = value;
    loop {
        digits[digit_count] = (remaining % 16) as u8;
        digit_count += 1;
        if remaining / 16 == 0 {
            break;
        }
        if digit_count == digits.len() {
            return Err(OamdRowdataError::InvalidEscapedInteger);
        }
        remaining = remaining / 16 - 1;
    }
    for index in (0..digit_count).rev() {
        writer.write(u32::from(digits[index]), 4)?;
        writer.write(u32::from(index != 0), 1)?;
    }
    Ok(())
}

fn encode_xy_code(value: i32) -> Result<i16, OamdRowdataError> {
    if !(0..=32767).contains(&value) {
        return Err(OamdRowdataError::InvalidInput);
    }
    let difference = i64::from(value) - 16384;
    let sign = if difference < 0 { -1 } else { 1 };
    let magnitude = difference.unsigned_abs();
    let rounded = (62 * magnitude + 16384) / 32768;
    let code = 31 + sign * i64::try_from(rounded).map_err(|_| OamdRowdataError::InvalidInput)?;
    i16::try_from(code).map_err(|_| OamdRowdataError::InvalidInput)
}

fn encode_z_code(value: i32) -> Result<i16, OamdRowdataError> {
    if !(-32767..=32767).contains(&value) {
        return Err(OamdRowdataError::InvalidInput);
    }
    let magnitude = u64::from(value.unsigned_abs());
    let rounded = (15 * magnitude + 16384) / 32768;
    let signed = i64::try_from(rounded).map_err(|_| OamdRowdataError::InvalidInput)?;
    i16::try_from(if value < 0 { -signed } else { signed })
        .map_err(|_| OamdRowdataError::InvalidInput)
}

#[derive(Default)]
struct BitWriter {
    bytes: Vec<u8>,
    position: usize,
}

impl BitWriter {
    fn write(&mut self, value: u32, count: u8) -> Result<(), OamdRowdataError> {
        if count > 32 || (count < 32 && value >= (1_u32 << count)) {
            return Err(OamdRowdataError::InvalidInput);
        }
        for offset in (0..count).rev() {
            let bit = (value >> offset) & 1;
            let byte_index = self.position / 8;
            let bit_index = 7 - self.position % 8;
            if byte_index == self.bytes.len() {
                self.bytes.push(0);
            }
            self.bytes[byte_index] |= (bit as u8) << bit_index;
            self.position += 1;
        }
        Ok(())
    }

    fn finish(self) -> (Vec<u8>, usize) {
        (self.bytes, self.position)
    }
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

    fn read(&mut self, count: u8) -> Result<u32, OamdRowdataError> {
        let count = usize::from(count);
        if count > 32 || self.position.saturating_add(count) > self.limit {
            return Err(OamdRowdataError::InvalidEscapedInteger);
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

    const fn position(&self) -> usize {
        self.position
    }
}
