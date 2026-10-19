// pattern: Functional Core
//! Selected 16-row secondary-eight panner and back-to-side fold.

use super::{READER_ROWS, RECORD_BYTES, ReaderTargetError};

const QUARTER_PI: f32 = f32::from_bits(0x3f49_0fdb);
const HALF: f32 = f32::from_bits(0x3f00_0000);
const GEOMETRY: [[i32; 4]; 8] = [
    [0, 0, 0, 35],
    [32_767, 0, 0, 35],
    [16_384, 0, 0, 35],
    [-1, -1, -1, 35],
    [0, 16_384, 0, 6],
    [32_767, 16_384, 0, 7],
    [0, 32_767, 0, 35],
    [32_767, 32_767, 0, 35],
];
const SLOT_BY_GEOMETRY_ROW: [usize; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

#[cfg(test)]
pub(super) const fn selected_geometry() -> [[i32; 4]; 8] {
    GEOMETRY
}

#[cfg(test)]
pub(super) const fn selected_equal_power_words() -> [u32; 2] {
    [QUARTER_PI.to_bits(), HALF.to_bits()]
}

pub(super) fn generate_targets(
    reader_records: &[[u8; RECORD_BYTES]; READER_ROWS],
) -> Result<[[u32; 6]; READER_ROWS], ReaderTargetError> {
    let mut targets = [[0_u32; 6]; READER_ROWS];
    for (row, record) in reader_records.iter().enumerate() {
        targets[row] = generate_target_row(row, record)?;
    }
    Ok(targets)
}

#[cfg(test)]
pub(super) fn generate_target_for_test(
    row: usize,
    record: &[u8; RECORD_BYTES],
) -> Result<[u32; 6], ReaderTargetError> {
    generate_target_row(row, record)
}

fn generate_target_row(
    row: usize,
    record: &[u8; RECORD_BYTES],
) -> Result<[u32; 6], ReaderTargetError> {
    let kind = word(record, 0);
    if row == 0 {
        if kind != 1
            || word(record, 1) != 4
            || word(record, 12) != 4096
            || word(record, 13) != 0
            || word(record, 11) < 1
        {
            return Err(ReaderTargetError::ProfileUnsupported);
        }
        let mut direct = [0_u32; 6];
        direct[3] = 1.0_f32.to_bits();
        return Ok(direct);
    }
    if kind != 2 || word(record, 12) != 4096 || word(record, 13) != 0 {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    let x = coordinate(record, 1)?;
    let y = coordinate(record, 2)?;
    let _z = coordinate(record, 3)?;
    let mut eight = [0.0_f32; 8];
    let y_weights = y_weights(y)?;
    for (layer, layer_weight) in y_weights.into_iter().enumerate() {
        for (slot, weight) in x_weights(x, layer)? {
            eight[slot] = checked_mul(weight, layer_weight)?;
        }
    }
    let mut six = [0.0_f32; 6];
    six.copy_from_slice(&eight[..6]);
    six[4] = checked_add(six[4], eight[6])?;
    six[5] = checked_add(six[5], eight[7])?;
    Ok(six.map(f32::to_bits))
}

fn coordinate(record: &[u8; RECORD_BYTES], word_index: usize) -> Result<f32, ReaderTargetError> {
    let signed = i32::from_le_bytes(word(record, word_index).to_le_bytes());
    if !(0..=32_767).contains(&signed) {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    Ok((signed as f32) / 32_768.0_f32)
}

fn x_weights(x: f32, layer: usize) -> Result<Vec<(usize, f32)>, ReaderTargetError> {
    let rows: &[usize] = match layer {
        0 => &[0, 2, 1],
        1 => &[4, 5],
        2 => &[6, 7],
        _ => return Err(ReaderTargetError::StateError),
    };
    let first = point_x(rows[0]);
    let last = point_x(*rows.last().expect("selected row is nonempty"));
    if x <= first {
        return Ok(vec![(SLOT_BY_GEOMETRY_ROW[rows[0]], 1.0)]);
    }
    if x >= last {
        let row = *rows.last().expect("selected row is nonempty");
        return Ok(vec![(SLOT_BY_GEOMETRY_ROW[row], 1.0)]);
    }
    for pair in rows.windows(2) {
        let left_x = point_x(pair[0]);
        let right_x = point_x(pair[1]);
        if left_x < x && x <= right_x {
            let fraction = checked_div(checked_sub(x, left_x)?, checked_sub(right_x, left_x)?)?;
            let (left_weight, right_weight) = equal_power(fraction)?;
            return Ok(vec![
                (SLOT_BY_GEOMETRY_ROW[pair[0]], left_weight),
                (SLOT_BY_GEOMETRY_ROW[pair[1]], right_weight),
            ]);
        }
    }
    Err(ReaderTargetError::ProfileUnsupported)
}

fn y_weights(y: f32) -> Result<[f32; 3], ReaderTargetError> {
    let levels = [point_y(0), point_y(4), point_y(6)];
    if y <= levels[0] {
        return Ok([1.0, 0.0, 0.0]);
    }
    if y >= levels[2] {
        return Ok([0.0, 0.0, 1.0]);
    }
    for layer in 0..2 {
        if levels[layer] < y && y <= levels[layer + 1] {
            let fraction = checked_div(
                checked_sub(y, levels[layer])?,
                checked_sub(levels[layer + 1], levels[layer])?,
            )?;
            let (low, high) = equal_power(fraction)?;
            let mut weights = [0.0; 3];
            weights[layer] = low;
            weights[layer + 1] = high;
            return Ok(weights);
        }
    }
    Err(ReaderTargetError::ProfileUnsupported)
}

fn equal_power(fraction: f32) -> Result<(f32, f32), ReaderTargetError> {
    if !(0.0..=1.0).contains(&fraction) {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    let (angle_fraction, swap) = if fraction <= HALF {
        (fraction, false)
    } else {
        (checked_sub(1.0, fraction)?, true)
    };
    let scaled = checked_mul(angle_fraction, QUARTER_PI)?;
    let halved = checked_mul(scaled, HALF)?;
    let angle = checked_mul(halved, 4.0)?;
    let cosine = (f64::from(angle).cos()) as f32;
    let sine = (f64::from(angle).sin()) as f32;
    if !cosine.is_finite() || !sine.is_finite() {
        return Err(ReaderTargetError::NumericFailure);
    }
    if swap {
        Ok((sine, cosine))
    } else {
        Ok((cosine, sine))
    }
}

fn point_x(row: usize) -> f32 {
    (GEOMETRY[row][0] as f32) / 32_768.0_f32
}

fn point_y(row: usize) -> f32 {
    (GEOMETRY[row][1] as f32) / 32_768.0_f32
}

fn word(record: &[u8; RECORD_BYTES], index: usize) -> u32 {
    let start = index * 4;
    u32::from_le_bytes(
        record[start..start + 4]
            .try_into()
            .expect("fixed word size"),
    )
}

fn checked_add(left: f32, right: f32) -> Result<f32, ReaderTargetError> {
    finite(left + right)
}

fn checked_sub(left: f32, right: f32) -> Result<f32, ReaderTargetError> {
    finite(left - right)
}

fn checked_mul(left: f32, right: f32) -> Result<f32, ReaderTargetError> {
    finite(left * right)
}

fn checked_div(left: f32, right: f32) -> Result<f32, ReaderTargetError> {
    if right == 0.0 {
        return Err(ReaderTargetError::NumericFailure);
    }
    finite(left / right)
}

fn finite(value: f32) -> Result<f32, ReaderTargetError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(ReaderTargetError::NumericFailure)
    }
}
