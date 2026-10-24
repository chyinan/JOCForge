// pattern: Functional Core
//! Bounded F09 exponent-refresh state transition helper.

use crate::codec::refresh_difference::map_refresh_difference;
use crate::numeric::{FiniteF32, r32_mul};

const POSITION_WEIGHTS_I32: [i32; 4] = [3, 6, 10, 10];
const NORMAL_THRESHOLD_BITS: u32 = 0x3ebd_a123;
const SPECIAL_THRESHOLD_BITS: u32 = 0x3ed7_0a3d;
const I16_SCALE_BITS: u32 = 0x3800_0000;

/// Applies one bounded in-place exponent-refresh transition to caller-owned rows.
///
/// Preconditions: `current` and `work` have equal lengths and do not overlap;
/// `0 <= begin < end <= current.len()`; `end < 32768`; `mode` and `special` are
/// each 0 or 1; every initial row value is in `0..=31`; and every selected
/// difference-table index is in `0..=31`. The caller must establish these facts.
/// Behavior outside this contract is unspecified. `mode` and `special` use I16
/// because the Wash interface records a 16-bit operand; this does not claim the
/// native code used a Rust bool or enum.
///
/// # Panics
///
/// Panics if the caller violates an index or row-length precondition. This is a
/// local Rust boundary, not a recovered native error outcome.
pub fn refresh_exponent_state(
    current: &mut [i16],
    work: &mut [i16],
    marker: &mut i16,
    mode: i16,
    special: i16,
    begin: i16,
    end: i16,
) {
    let begin_index = usize::try_from(begin).expect("begin is nonnegative by precondition");
    let end_index = usize::try_from(end).expect("end is nonnegative by precondition");

    if *marker != 0 {
        work[begin_index] = current[begin_index];
        for index in begin_index + 1..end_index {
            let limit = i32::from(work[index - 1]) + 2;
            work[index] = clamp_i16_to_i32(current[index], limit);
        }
        return;
    }

    if mode != 0 {
        let mut difference_sum = 0_i16;
        for index in begin_index..end_index {
            if index > begin_index {
                let limit = i32::from(current[index - 1]) + 2;
                if i32::from(current[index]) >= limit {
                    current[index] = i16::try_from(limit).expect("bounded refresh limit fits I16");
                }
            }

            let difference = (i32::from(current[index]) - i32::from(work[index])).abs();
            let weight_index = index.min(POSITION_WEIGHTS_I32.len() - 1);
            let table_index = difference * POSITION_WEIGHTS_I32[weight_index] / 10;
            difference_sum = difference_sum.wrapping_add(difference_table_value(table_index));
        }

        let threshold_bits = if special > 0 {
            SPECIAL_THRESHOLD_BITS
        } else {
            NORMAL_THRESHOLD_BITS
        };
        if exceeds_threshold(difference_sum, end - begin, threshold_bits) {
            *marker = 1;
            work[begin_index..end_index].copy_from_slice(&current[begin_index..end_index]);
        }
        return;
    }

    let first_difference = (i32::from(current[begin_index]) - i32::from(work[begin_index])).abs();
    let mut difference_sum = difference_table_value(first_difference);
    work[begin_index] = current[begin_index];

    for index in begin_index + 1..end_index {
        let limit = i32::from(work[index - 1]) + 2;
        let value = clamp_i16_to_i32(current[index], limit);
        let difference = (i32::from(value) - i32::from(work[index])).abs();
        difference_sum = difference_sum.wrapping_add(difference_table_value(difference));
        work[index] = value;
    }

    if exceeds_threshold(difference_sum, end - begin, NORMAL_THRESHOLD_BITS) {
        *marker = 1;
    }
}

fn clamp_i16_to_i32(value: i16, limit: i32) -> i16 {
    i16::try_from(i32::from(value).min(limit)).expect("bounded refresh limit fits I16")
}

fn difference_table_value(index: i32) -> i16 {
    let index = i16::try_from(index).expect("difference-table index fits I16 by precondition");
    map_refresh_difference(index).expect("difference-table index is valid by precondition")
}

fn exceeds_threshold(sum: i16, span: i16, threshold_bits: u32) -> bool {
    let scale = finite_f32(I16_SCALE_BITS);
    let sum_scaled = r32_mul(finite_f32(f32::from(sum).to_bits()), scale)
        .expect("bounded I16 scaled by 2^-15 remains finite");
    let span_scaled = r32_mul(finite_f32(f32::from(span).to_bits()), scale)
        .expect("bounded I16 span scaled by 2^-15 remains finite");
    let thresholded_span = r32_mul(span_scaled, finite_f32(threshold_bits))
        .expect("bounded refresh threshold product remains finite");
    sum_scaled.value() > thresholded_span.value()
}

fn finite_f32(bits: u32) -> FiniteF32 {
    FiniteF32::new(f32::from_bits(bits)).expect("fixed refresh parameter is finite")
}
