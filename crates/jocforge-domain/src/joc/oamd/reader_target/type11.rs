// pattern: Functional Core
//! Selected single-frame Type11 writer/decoder seam used by the reader feedback stage.

use super::ReaderTargetError;

const ROW_COUNT: usize = 16;
const SELECTED_TRIM_FLAGS: [u8; 9] = [1, 0, 0, 1, 0, 1, 1, 0, 1];
const DEFAULT_TRIM_GROUP: [i32; 5] = [16, 16, 16, 0, 0];

#[cfg(test)]
pub(super) const fn selected_trim_profile() -> (u8, [u8; 9], [i32; 5]) {
    (3, SELECTED_TRIM_FLAGS, DEFAULT_TRIM_GROUP)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct DecodedOamdRow {
    pub kind: u8,
    pub mute: bool,
    pub x: u32,
    pub y: u32,
    pub z: i32,
    pub component: u32,
    pub diffuse: u32,
    pub z_valid: u32,
    pub extra: i32,
    pub gain: u32,
    pub row_flag: u32,
}

pub(super) fn roundtrip(
    controls: &[[u32; 13]; 15],
) -> Result<([DecodedOamdRow; ROW_COUNT], Vec<u8>), ReaderTargetError> {
    let body = encode_body(controls)?;
    let decoded = decode_body(&body)?;
    Ok((decoded, body))
}

fn encode_body(controls: &[[u32; 13]; 15]) -> Result<Vec<u8>, ReaderTargetError> {
    let mut rowdata = BitWriter::default();
    rowdata.write(0, 1)?;
    rowdata.write(0, 2)?;
    rowdata.write(0, 3)?;
    rowdata.write(0, 6)?;
    rowdata.write(2, 2)?;
    rowdata.write(1, 1)?;
    let mut previous_gain = 0_u32;
    let mut previous_unmuted = false;
    for row in 0..ROW_COUNT {
        let (kind, gain, coordinate, component, z_valid) = if row == 0 {
            (0_u8, 4096_u32, [0_i32; 3], 0_u32, false)
        } else {
            let control = &controls[row - 1];
            validate_control(control)?;
            let inactive_pair = control[0] != 0;
            let mut coordinate = [0_i32; 3];
            if !inactive_pair {
                for axis in 0..3 {
                    coordinate[axis] = quantize_coordinate(f32::from_bits(control[5 + axis]))?;
                }
            }
            (2_u8, 4096_u32, coordinate, control[11], control[12] != 0)
        };
        rowdata.write(0, 1)?;
        if row > 0 && previous_unmuted && previous_gain == gain {
            rowdata.write(3, 2)?;
        } else if (3866..=4337).contains(&gain) {
            rowdata.write(0, 2)?;
        } else if gain == 0 {
            rowdata.write(1, 2)?;
        } else {
            return Err(ReaderTargetError::ProfileUnsupported);
        }
        rowdata.write(1, 1)?;
        if kind == 2 {
            rowdata.write(xy_code(coordinate[0])?, 6)?;
            rowdata.write(xy_code(coordinate[1])?, 6)?;
            let z_code = z_code(coordinate[2])?;
            rowdata.write(u32::from(z_code >= 0), 1)?;
            rowdata.write(z_code.unsigned_abs(), 4)?;
            rowdata.write(0, 1)?;
            rowdata.write(component, 3)?;
            rowdata.write(u32::from(z_valid), 1)?;
            rowdata.write(0, 2)?;
            rowdata.write(0, 1)?;
            rowdata.write(0, 1)?;
        }
        rowdata.write(0, 1)?;
        previous_gain = gain;
        previous_unmuted = true;
    }
    let rowdata = rowdata.finish();
    let trim = encode_trim();
    let mut body = BitWriter::default();
    body.write(0, 2)?;
    body.write(15, 5)?;
    body.write(1, 1)?;
    body.write(1, 1)?;
    body.write(0, 1)?;
    body.write(2, 4)?;
    write_section(&mut body, 1, &rowdata)?;
    write_section(&mut body, 2, &trim)?;
    let bytes = body.finish();
    if bytes.len() != 69 || bytes.len() > 160 {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    Ok(bytes)
}

fn validate_control(control: &[u32; 13]) -> Result<(), ReaderTargetError> {
    if control[4] != 0 || control[8] != 0 || control[9] != 0 || control[11] > 5 {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    for bits in &control[5..8] {
        if !f32::from_bits(*bits).is_finite() {
            return Err(ReaderTargetError::InputError);
        }
    }
    Ok(())
}

fn quantize_coordinate(value: f32) -> Result<i32, ReaderTargetError> {
    let product = value * 32_767.0_f32;
    if !product.is_finite() {
        return Err(ReaderTargetError::NumericFailure);
    }
    let truncated = product.trunc();
    if !(-32_767.0..=32_767.0).contains(&truncated) {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    Ok(truncated as i32)
}

fn xy_code(value: i32) -> Result<u32, ReaderTargetError> {
    if !(0..=32_767).contains(&value) {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    let difference = value - 16_384;
    let sign = if difference < 0 { -1 } else { 1 };
    let magnitude = difference.unsigned_abs();
    let scaled = 62_u32
        .checked_mul(magnitude)
        .and_then(|product| product.checked_add(16_384))
        .ok_or(ReaderTargetError::NumericFailure)?;
    let rounded = scaled / 32_768;
    let code =
        31_i32 + sign * i32::try_from(rounded).map_err(|_| ReaderTargetError::NumericFailure)?;
    u32::try_from(code).map_err(|_| ReaderTargetError::ProfileUnsupported)
}

fn z_code(value: i32) -> Result<i32, ReaderTargetError> {
    if !(-32_767..=32_767).contains(&value) {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    let magnitude = value.unsigned_abs();
    let scaled = 15_u32
        .checked_mul(magnitude)
        .and_then(|product| product.checked_add(16_384))
        .ok_or(ReaderTargetError::NumericFailure)?
        / 32_768;
    let signed_magnitude = i32::try_from(scaled).map_err(|_| ReaderTargetError::NumericFailure)?;
    Ok(if value < 0 {
        -signed_magnitude
    } else {
        signed_magnitude
    })
}

fn encode_trim() -> Vec<u8> {
    let mut trim = BitWriter::default();
    trim.write(0, 1).expect("constant control fits");
    trim.write(3, 2).expect("constant mode fits");
    trim.write(0, 2).expect("constant reserved field fits");
    trim.write(2, 2).expect("mixed selected flags use mode two");
    for flag in SELECTED_TRIM_FLAGS {
        trim.write(u32::from(flag), 1).expect("constant flag fits");
        if flag == 0 {
            trim.write(1, 1).expect("default tuple marker fits");
        }
    }
    trim.write(0, 1).expect("selected row flags are all zero");
    let _ = DEFAULT_TRIM_GROUP;
    trim.finish()
}

fn write_section(
    output: &mut BitWriter,
    kind: u32,
    payload: &[u8],
) -> Result<(), ReaderTargetError> {
    if payload.is_empty() {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    output.write(kind, 4)?;
    write_escaped(
        output,
        u32::try_from(payload.len() - 1).map_err(|_| ReaderTargetError::InputError)?,
    )?;
    for byte in payload {
        output.write(u32::from(*byte), 8)?;
    }
    Ok(())
}

fn write_escaped(output: &mut BitWriter, value: u32) -> Result<(), ReaderTargetError> {
    let mut digits = [0_u8; 4];
    let mut length = 0;
    let mut current = value;
    loop {
        digits[length] = (current % 16) as u8;
        length += 1;
        if current / 16 == 0 {
            break;
        }
        current = current / 16 - 1;
        if length == digits.len() {
            return Err(ReaderTargetError::ProfileUnsupported);
        }
    }
    for index in (0..length).rev() {
        output.write(u32::from(digits[index]), 4)?;
        output.write(u32::from(index != 0), 1)?;
    }
    Ok(())
}

#[derive(Default)]
struct BitWriter {
    bytes: Vec<u8>,
    bits: u64,
}

impl BitWriter {
    fn write(&mut self, value: u32, count: u8) -> Result<(), ReaderTargetError> {
        if count > 32 || (count < 32 && value >= (1_u32 << count)) {
            return Err(ReaderTargetError::InputError);
        }
        for offset in (0..count).rev() {
            let bit = (value >> offset) & 1;
            let byte_index =
                usize::try_from(self.bits / 8).map_err(|_| ReaderTargetError::NumericFailure)?;
            let bit_index = 7 - (self.bits % 8) as u8;
            if byte_index == self.bytes.len() {
                self.bytes.push(0);
            }
            self.bytes[byte_index] |= (bit as u8) << bit_index;
            self.bits += 1;
        }
        Ok(())
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

fn decode_body(bytes: &[u8]) -> Result<[DecodedOamdRow; ROW_COUNT], ReaderTargetError> {
    if bytes.len() != 69 {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    let mut reader = BitReader::new(bytes);
    if reader.read(2)? != 0 {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    let count_prefix = reader.read(5)?;
    let row_count = if count_prefix == 31 {
        usize::try_from(32 + reader.read(7)?).map_err(|_| ReaderTargetError::InputError)?
    } else {
        usize::try_from(count_prefix + 1).map_err(|_| ReaderTargetError::InputError)?
    };
    let shortcut = reader.read(1)?;
    let leading_lfe = reader.read(1)?;
    let protected = reader.read(1)?;
    let section_count = reader.read(4)?;
    if row_count != ROW_COUNT
        || shortcut != 1
        || leading_lfe != 1
        || protected != 0
        || section_count != 2
    {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    let mut rows = [DecodedOamdRow::default(); ROW_COUNT];
    let mut rowdata_seen = false;
    let mut trim_seen = false;
    for _ in 0..section_count {
        let kind = reader.read(4)?;
        let payload_bytes = read_escaped(&mut reader)?
            .checked_add(1)
            .ok_or(ReaderTargetError::NumericFailure)?;
        let payload_bytes =
            usize::try_from(payload_bytes).map_err(|_| ReaderTargetError::InputError)?;
        let mut payload = Vec::with_capacity(payload_bytes);
        for _ in 0..payload_bytes {
            payload.push(reader.read(8)? as u8);
        }
        match kind {
            1 if !rowdata_seen => {
                rows = decode_rowdata(&payload, row_count)?;
                rowdata_seen = true;
            }
            2 if !trim_seen => {
                let row_flags = decode_trim(&payload, row_count)?;
                for (row, flag) in rows.iter_mut().zip(row_flags) {
                    row.row_flag = u32::from(flag);
                }
                trim_seen = true;
            }
            _ => return Err(ReaderTargetError::ProfileUnsupported),
        }
    }
    if !rowdata_seen || !trim_seen {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    reader.finish_zero_padding()?;
    Ok(rows)
}

fn decode_rowdata(
    payload: &[u8],
    row_count: usize,
) -> Result<[DecodedOamdRow; ROW_COUNT], ReaderTargetError> {
    let mut reader = BitReader::new(payload);
    if reader.read(1)? != 0 {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    let base_mode = reader.read(2)?;
    if base_mode != 0 || reader.read(3)? != 0 || reader.read(6)? != 0 {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    let duration_mode = reader.read(2)?;
    if duration_mode != 2 || reader.read(1)? != 1 {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    let mut rows = [DecodedOamdRow::default(); ROW_COUNT];
    let mut previous_gain = 0_u32;
    let mut previous_unmuted = false;
    for (row_index, output_row) in rows.iter_mut().enumerate().take(row_count) {
        let kind = if row_index == 0 { 0 } else { 2 };
        let mute = reader.read(1)? != 0;
        let gain_mode = reader.read(2)?;
        let gain = match gain_mode {
            0 => 4096,
            3 if row_index == 0 => 4096,
            3 if previous_unmuted => previous_gain,
            1 | 3 => 0,
            _ => return Err(ReaderTargetError::ProfileUnsupported),
        };
        let mut decoded = if mute {
            if kind == 0 {
                return Err(ReaderTargetError::ProfileUnsupported);
            }
            DecodedOamdRow {
                kind,
                mute: true,
                x: 16_384,
                y: 16_384,
                z: 0,
                component: 0,
                diffuse: 0,
                z_valid: 1,
                extra: 0,
                gain: 0,
                row_flag: 0,
            }
        } else {
            if reader.read(1)? != 1 {
                return Err(ReaderTargetError::ProfileUnsupported);
            }
            let mut decoded = DecodedOamdRow {
                kind,
                mute: false,
                gain,
                ..DecodedOamdRow::default()
            };
            if kind == 2 {
                decoded.x = decode_xy(reader.read(6)?)?;
                decoded.y = decode_xy(reader.read(6)?)?;
                let positive = reader.read(1)? != 0;
                let magnitude = decode_z(reader.read(4)?)?;
                decoded.z = if positive { magnitude } else { -magnitude };
                if reader.read(1)? != 0 {
                    return Err(ReaderTargetError::ProfileUnsupported);
                }
                decoded.component = reader.read(3)?;
                if decoded.component > 5 {
                    return Err(ReaderTargetError::ProfileUnsupported);
                }
                decoded.z_valid = reader.read(1)?;
                if reader.read(2)? != 0 || reader.read(1)? != 0 {
                    return Err(ReaderTargetError::ProfileUnsupported);
                }
                decoded.diffuse = reader.read(1)?;
                if decoded.z_valid == 0 {
                    decoded.z = 0;
                }
            }
            decoded
        };
        if reader.read(1)? != 0 {
            return Err(ReaderTargetError::ProfileUnsupported);
        }
        decoded.gain = gain;
        *output_row = decoded;
        previous_unmuted = !mute;
        previous_gain = gain;
    }
    reader.finish_zero_padding()?;
    Ok(rows)
}

fn decode_trim(payload: &[u8], row_count: usize) -> Result<[u8; ROW_COUNT], ReaderTargetError> {
    let mut reader = BitReader::new(payload);
    if reader.read(1)? != 0 {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    if reader.read(2)? != 3 || reader.read(2)? != 0 || reader.read(2)? != 2 {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    for expected in SELECTED_TRIM_FLAGS {
        let flag = reader.read(1)? as u8;
        if flag != expected {
            return Err(ReaderTargetError::ProfileUnsupported);
        }
        if flag == 0 && reader.read(1)? != 1 {
            return Err(ReaderTargetError::ProfileUnsupported);
        }
    }
    let has_rows = reader.read(1)?;
    let mut row_flags = [0_u8; ROW_COUNT];
    if has_rows != 0 {
        for flag in &mut row_flags[..row_count] {
            *flag = reader.read(1)? as u8;
        }
    }
    reader.finish_zero_padding()?;
    Ok(row_flags)
}

fn decode_xy(code: u32) -> Result<u32, ReaderTargetError> {
    if code > 62 {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    Ok(((32_768_u64 * u64::from(code) + 31) / 62).min(32_767) as u32)
}

#[cfg(test)]
pub(super) fn decode_xy_for_test(code: u32) -> Result<u32, ReaderTargetError> {
    decode_xy(code)
}

fn decode_z(code: u32) -> Result<i32, ReaderTargetError> {
    if code > 15 {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    Ok(((32_768_u64 * u64::from(code) + 7) / 15).min(32_767) as i32)
}

#[cfg(test)]
pub(super) fn decode_z_for_test(code: u32) -> Result<i32, ReaderTargetError> {
    decode_z(code)
}

fn read_escaped(reader: &mut BitReader<'_>) -> Result<u32, ReaderTargetError> {
    let mut value = 0_u32;
    for group in 0..4 {
        let digit = reader.read(4)?;
        let continuation = reader.read(1)?;
        if group == 0 {
            value = digit;
        } else {
            value = value
                .checked_add(1)
                .and_then(|previous| previous.checked_mul(16))
                .and_then(|previous| previous.checked_add(digit))
                .ok_or(ReaderTargetError::NumericFailure)?;
        }
        if continuation == 0 {
            return Ok(value);
        }
    }
    Err(ReaderTargetError::ProfileUnsupported)
}

struct BitReader<'a> {
    bytes: &'a [u8],
    position: u64,
}

impl<'a> BitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn read(&mut self, count: u8) -> Result<u32, ReaderTargetError> {
        if count > 32 {
            return Err(ReaderTargetError::InputError);
        }
        let mut value = 0_u32;
        for _ in 0..count {
            let byte_index = usize::try_from(self.position / 8)
                .map_err(|_| ReaderTargetError::NumericFailure)?;
            let bit_index = 7 - (self.position % 8) as u8;
            let byte = *self
                .bytes
                .get(byte_index)
                .ok_or(ReaderTargetError::InputError)?;
            value = (value << 1) | u32::from((byte >> bit_index) & 1);
            self.position += 1;
        }
        Ok(value)
    }

    fn finish_zero_padding(&mut self) -> Result<(), ReaderTargetError> {
        let remaining = (self.bytes.len() as u64)
            .checked_mul(8)
            .and_then(|length| length.checked_sub(self.position))
            .ok_or(ReaderTargetError::InputError)?;
        if remaining > 7 {
            return Err(ReaderTargetError::ProfileUnsupported);
        }
        while self.position < (self.bytes.len() as u64) * 8 {
            if self.read(1)? != 0 {
                return Err(ReaderTargetError::ProfileUnsupported);
            }
        }
        Ok(())
    }
}
