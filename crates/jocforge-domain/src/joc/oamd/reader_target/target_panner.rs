// pattern: Functional Core
//! Selected 16-row secondary-eight panner and back-to-side fold.

use super::{READER_ROWS, RECORD_BYTES, ReaderTargetError};
use crate::frontend::job_configuration::{MainTargetTrim, SurroundHeightTrim};

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
    trim: MainTargetTrim,
    active_points: &[bool; READER_ROWS - 1],
) -> Result<[[u32; 6]; READER_ROWS], ReaderTargetError> {
    let mut targets = [[0_u32; 6]; READER_ROWS];
    for (row, record) in reader_records.iter().enumerate() {
        let active = row != 0 && active_points[row - 1];
        targets[row] = generate_target_row(row, record, trim, active)?;
    }
    Ok(targets)
}

#[cfg(test)]
pub(super) fn generate_target_for_test(
    row: usize,
    record: &[u8; RECORD_BYTES],
) -> Result<[u32; 6], ReaderTargetError> {
    generate_target_row(row, record, MainTargetTrim::UNCONFIGURED, false)
}

fn generate_target_row(
    row: usize,
    record: &[u8; RECORD_BYTES],
    trim: MainTargetTrim,
    active: bool,
) -> Result<[u32; 6], ReaderTargetError> {
    render_target_row(
        row,
        record,
        trim,
        active,
        #[cfg(test)]
        &mut |_, _, _| {},
    )
}

fn render_target_row(
    row: usize,
    record: &[u8; RECORD_BYTES],
    trim: MainTargetTrim,
    active: bool,
    #[cfg(test)] observe_scale: &mut impl FnMut([f32; 8], f32, [f32; 8]),
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
    let z = coordinate(record, 3)?;
    let mut eight = point_geometry(x, y)?;
    if active && trim == MainTargetTrim::DefaultAutoPosition {
        #[cfg(test)]
        let geometry = eight;
        let factor = default_position_amplitude(y, z);
        scale_point_slots(&mut eight, factor);
        #[cfg(test)]
        observe_scale(geometry, factor, eight);
    }
    let mut six = fold_point(eight)?;
    if let MainTargetTrim::AfterFold(manual) = trim {
        if manual != SurroundHeightTrim::UNITY {
            let gain = manual.gain(y, z);
            six = six.map(|value| value * gain);
        }
    }
    Ok(six.map(f32::to_bits))
}

fn point_geometry(x: f32, y: f32) -> Result<[f32; 8], ReaderTargetError> {
    let mut eight = [0.0_f32; 8];
    let y_weights = y_weights(y)?;
    for (layer, layer_weight) in y_weights.into_iter().enumerate() {
        for (slot, weight) in x_weights(x, layer)? {
            eight[slot] = checked_mul(weight, layer_weight)?;
        }
    }
    Ok(eight)
}

/// JOC51-DEFAULT-AUTO-POSITION-GAIN-V1：四个等距幅度节点的牛顿前向插值。
fn default_position_amplitude(y: f32, z: f32) -> f32 {
    const A: [f64; 4] = [1.0, 0.89182374, 0.79474273, 0.70794702];
    let depth = (f64::from(y) / 0.6).clamp(0.0, 1.0);
    let height = ((f64::from(z) - 0.2) / 0.8).clamp(0.0, 1.0);
    let u = 3.0 * (depth + height).min(1.0);
    let first = A[1] - A[0];
    let second = (A[2] - A[1]) - first;
    let third = ((A[3] - A[2]) - (A[2] - A[1])) - second;
    (A[0] + u * (first + (u - 1.0) * (second / 2.0 + (u - 2.0) * third / 6.0))) as f32
}

fn scale_point_slots(eight: &mut [f32; 8], factor: f32) {
    for slot in [0, 1, 2, 4, 5, 6, 7] {
        eight[slot] *= factor;
    }
}

fn fold_point(eight: [f32; 8]) -> Result<[f32; 6], ReaderTargetError> {
    let mut six = [0.0_f32; 6];
    six.copy_from_slice(&eight[..6]);
    six[4] = checked_add(six[4], eight[6])?;
    six[5] = checked_add(six[5], eight[7])?;
    Ok(six)
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
    let (angle, swap) = equal_power_angle(fraction)?;
    let rounded = super::exact_trigonometry::mathematical_r64_r32(angle)?;
    if swap {
        Ok((rounded.sine, rounded.cosine))
    } else {
        Ok((rounded.cosine, rounded.sine))
    }
}

fn equal_power_angle(fraction: f32) -> Result<(f32, bool), ReaderTargetError> {
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
    Ok((angle, swap))
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

#[cfg(test)]
mod rate_scope_tests;

#[cfg(test)]
mod tests {
    use super::{equal_power, equal_power_angle};
    use serde_json::Value;

    const TRIGONOMETRY_KAT: &str =
        include_str!("../../../../tests/fixtures/reader_target/panner-trigonometry-kat.json");

    #[test]
    fn default_auto_position_matches_all_sealed_geometry_and_decoded_rows() {
        use super::{
            MainTargetTrim, default_position_amplitude, fold_point, generate_target_row,
            point_geometry, scale_point_slots,
        };
        let inputs: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/default_auto_position/inputs.json"
        ))
        .unwrap();
        let expected: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/default_auto_position/expected.json"
        ))
        .unwrap();
        let slots = [0, 1, 2, 4, 5, 6, 7];
        let words = |value: &Value| {
            value
                .as_array()
                .unwrap()
                .iter()
                .map(|v| (v.as_f64().unwrap() as f32).to_bits())
                .collect::<Vec<_>>()
        };
        for family in ["seam", "decoded_grid"] {
            for case in inputs[family].as_array().unwrap() {
                let id = case["id"].as_str().unwrap();
                let q: [u32; 3] = serde_json::from_value(case["q"].clone()).unwrap();
                let [x, y, z] = q.map(|v| v as f32 / 32768.0);
                let mut eight = if let Some(g) = case.get("geometry7") {
                    let mut eight = [0.0; 8];
                    for (slot, word) in slots.into_iter().zip(words(g)) {
                        eight[slot] = f32::from_bits(word);
                    }
                    eight
                } else {
                    point_geometry(x, y).unwrap()
                };
                let geometry = slots.map(|slot| eight[slot].to_bits());
                let factor = default_position_amplitude(y, z);
                scale_point_slots(&mut eight, factor);
                let six = fold_point(eight).unwrap().map(f32::to_bits);
                let answer = &expected[id];
                assert_eq!(geometry.as_slice(), words(&answer["geometry7"]), "{id}");
                assert_eq!(
                    factor.to_bits(),
                    (answer["factor"].as_f64().unwrap() as f32).to_bits(),
                    "{id}"
                );
                assert_eq!(
                    eight.map(f32::to_bits).as_slice(),
                    words(&answer["pre8"]),
                    "{id}"
                );
                assert_eq!(six.as_slice(), words(&answer["post6"]), "{id}");
                if family == "decoded_grid" {
                    let mut record = [0; super::RECORD_BYTES];
                    for (index, v) in [(0, 2), (1, q[0]), (2, q[1]), (3, q[2]), (12, 4096)] {
                        super::super::write_record_word(&mut record, index, v);
                    }
                    assert_eq!(
                        generate_target_row(1, &record, MainTargetTrim::DefaultAutoPosition, true)
                            .unwrap(),
                        six,
                        "{id}"
                    );
                }
            }
        }
    }

    #[test]
    fn default_auto_position_preserves_inactive_rows_direct_lfe_and_selected_rejections() {
        use super::{MainTargetTrim, ReaderTargetError, generate_target_row};
        let mut point = [0; super::RECORD_BYTES];
        for (index, v) in [(0, 2), (1, 10042), (2, 17970), (3, 10923), (12, 4096)] {
            super::super::write_record_word(&mut point, index, v);
        }
        let plain = MainTargetTrim::UNCONFIGURED;
        let auto = MainTargetTrim::DefaultAutoPosition;
        assert_eq!(
            generate_target_row(1, &point, auto, false),
            generate_target_row(1, &point, plain, true)
        );
        for (index, value) in [
            (12, 0),
            (13, 1),
            (12, 2048),
            (0, 3),
            (3, u32::MAX),
            (2, 32768),
            (1, u32::MAX),
        ] {
            let mut other = point;
            super::super::write_record_word(&mut other, index, value);
            for trim in [auto, plain] {
                assert_eq!(
                    generate_target_row(1, &other, trim, true),
                    Err(ReaderTargetError::ProfileUnsupported)
                );
            }
        }
        for (index, value) in [(0, 1), (1, 4), (2, u32::MAX), (11, 1)] {
            super::super::write_record_word(&mut point, index, value);
        }
        assert_eq!(
            generate_target_row(0, &point, auto, true).unwrap(),
            [0, 0, 0, 1.0_f32.to_bits(), 0, 0]
        );
    }

    #[test]
    fn selected_equal_power_pairs_match_the_rational_enclosure_kat() {
        let vectors: Value = serde_json::from_str(TRIGONOMETRY_KAT).unwrap();
        for case in vectors["panner_fractions"].as_array().unwrap() {
            let fraction_bits =
                u32::from_str_radix(case["fraction_f32_hex"].as_str().unwrap(), 16).unwrap();
            let expected_angle_bits =
                u32::from_str_radix(case["angle_f32_hex"].as_str().unwrap(), 16).unwrap();
            let (angle, swap) = equal_power_angle(f32::from_bits(fraction_bits)).unwrap();
            assert_eq!(
                angle.to_bits(),
                expected_angle_bits,
                "{}",
                case["fraction_f32_hex"]
            );
            let rounded = super::super::exact_trigonometry::mathematical_r64_r32(angle).unwrap();
            let expected = case["pair_f32_hex"]
                .as_array()
                .unwrap()
                .iter()
                .map(|word| u32::from_str_radix(word.as_str().unwrap(), 16).unwrap())
                .collect::<Vec<_>>();
            let pair = if swap {
                [rounded.sine.to_bits(), rounded.cosine.to_bits()]
            } else {
                [rounded.cosine.to_bits(), rounded.sine.to_bits()]
            };
            assert_eq!(pair, expected.as_slice(), "{}", case["fraction_f32_hex"]);
            assert_eq!(
                rounded.cosine_series_steps,
                case["cosine_series_steps"].as_u64().unwrap(),
                "{}",
                case["fraction_f32_hex"]
            );
            assert_eq!(
                rounded.sine_series_steps,
                case["sine_series_steps"].as_u64().unwrap(),
                "{}",
                case["fraction_f32_hex"]
            );
            assert_eq!(
                {
                    let (left, right) = equal_power(f32::from_bits(fraction_bits)).unwrap();
                    [left.to_bits(), right.to_bits()]
                },
                pair,
                "{}",
                case["fraction_f32_hex"]
            );
        }
    }
}
