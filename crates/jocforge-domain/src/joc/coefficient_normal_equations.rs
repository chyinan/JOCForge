// pattern: Functional Core
//! Weighted Gram and cross equations for the selected JOC coefficient fit.

const OBJECT_COUNT: usize = 15;
const SELECTED_CARRIER_COUNT: usize = 5;
const SLOT_COUNT: usize = 48;
const BIN_COUNT: usize = 64;
const COMPONENT_COUNT: usize = 2;
const BAND_COUNT: usize = 12;
const BAND_BOUNDARIES: [usize; BAND_COUNT + 1] = [0, 1, 2, 3, 4, 6, 8, 11, 14, 18, 23, 35, 64];
const ZERO_BITS: u32 = 0x0000_0000;
const GRAM_DIAGONAL_TINY_BITS: u32 = 0x1e3c_e508;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoefficientNormalEquation<const CARRIER_COUNT: usize = SELECTED_CARRIER_COUNT> {
    object: u8,
    band: u8,
    begin_bin: u8,
    end_bin: u8,
    gram_words: [[u32; CARRIER_COUNT]; CARRIER_COUNT],
    cross_words: [u32; CARRIER_COUNT],
}

impl<const CARRIER_COUNT: usize> CoefficientNormalEquation<CARRIER_COUNT> {
    pub const fn object(&self) -> u8 {
        self.object
    }

    pub const fn band(&self) -> u8 {
        self.band
    }

    pub const fn begin_bin(&self) -> u8 {
        self.begin_bin
    }

    pub const fn end_bin(&self) -> u8 {
        self.end_bin
    }

    pub const fn gram_words(&self) -> &[[u32; CARRIER_COUNT]; CARRIER_COUNT] {
        &self.gram_words
    }

    pub const fn cross_words(&self) -> &[u32; CARRIER_COUNT] {
        &self.cross_words
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoefficientNormalEquationError {
    InputShape,
    InputNonFinite,
    NumericNonFinite,
}

impl CoefficientNormalEquationError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputShape => "JOC_NORMAL_EQUATION_INPUT_SHAPE",
            Self::InputNonFinite => "JOC_NORMAL_EQUATION_INPUT_NONFINITE",
            Self::NumericNonFinite => "JOC_NORMAL_EQUATION_NUMERIC_NONFINITE",
        }
    }
}

impl std::fmt::Display for CoefficientNormalEquationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for CoefficientNormalEquationError {}

/// Builds all 15x12 selected equations. Target and carrier inputs are
/// object/carrier-major 48x64 interleaved complex-F32 windows. Each component
/// is independently weighted before any Gram or cross product.
#[allow(clippy::needless_range_loop)] // Explicit DSP index order is part of the contract.
pub fn build_selected_normal_equations(
    target_words: &[u32],
    carrier_words: &[u32],
    weight_words: &[u32; SLOT_COUNT],
) -> Result<Vec<CoefficientNormalEquation<SELECTED_CARRIER_COUNT>>, CoefficientNormalEquationError>
{
    build_normal_equations_for_carriers::<SELECTED_CARRIER_COUNT>(
        target_words,
        carrier_words,
        weight_words,
    )
}

/// Builds the selected object/band equations for a fixed number of carrier rows.
///
/// The `C=5` wrapper keeps the existing accumulation and loop order unchanged.
#[allow(clippy::needless_range_loop)] // Explicit DSP index order is part of the contract.
pub fn build_normal_equations_for_carriers<const CARRIER_COUNT: usize>(
    target_words: &[u32],
    carrier_words: &[u32],
    weight_words: &[u32; SLOT_COUNT],
) -> Result<Vec<CoefficientNormalEquation<CARRIER_COUNT>>, CoefficientNormalEquationError> {
    if target_words.len() != OBJECT_COUNT * SLOT_COUNT * BIN_COUNT * COMPONENT_COUNT
        || CARRIER_COUNT == 0
        || carrier_words.len() != CARRIER_COUNT * SLOT_COUNT * BIN_COUNT * COMPONENT_COUNT
    {
        return Err(CoefficientNormalEquationError::InputShape);
    }
    let weights = weight_words.map(f32::from_bits);
    if weights.iter().any(|weight| !weight.is_finite()) {
        return Err(CoefficientNormalEquationError::InputNonFinite);
    }
    let targets = weighted_slots(target_words, &weights)?;
    let carriers = weighted_slots(carrier_words, &weights)?;
    let mut gram = [[[ZERO_BITS; CARRIER_COUNT]; CARRIER_COUNT]; BAND_COUNT];
    for row in 0..CARRIER_COUNT {
        for column in row..CARRIER_COUNT {
            let initial = if row == column {
                GRAM_DIAGONAL_TINY_BITS
            } else {
                ZERO_BITS
            };
            let dots = band_dots(&carriers, row, &carriers, column, initial)?;
            for (matrix, dot) in gram.iter_mut().zip(dots) {
                matrix[row][column] = dot;
                matrix[column][row] = dot;
            }
        }
    }

    let mut equations = Vec::with_capacity(OBJECT_COUNT * BAND_COUNT);
    for object in 0..OBJECT_COUNT {
        let mut cross = [[ZERO_BITS; CARRIER_COUNT]; BAND_COUNT];
        for carrier in 0..CARRIER_COUNT {
            let dots = band_dots(&targets, object, &carriers, carrier, ZERO_BITS)?;
            for (row, dot) in cross.iter_mut().zip(dots) {
                row[carrier] = dot;
            }
        }
        for band in 0..BAND_COUNT {
            equations.push(CoefficientNormalEquation {
                object: object as u8,
                band: band as u8,
                begin_bin: BAND_BOUNDARIES[band] as u8,
                end_bin: BAND_BOUNDARIES[band + 1] as u8,
                gram_words: gram[band],
                cross_words: cross[band],
            });
        }
    }
    Ok(equations)
}

struct WeightedSlot {
    real: [f32; BIN_COUNT],
    imaginary: [f32; BIN_COUNT],
}

fn weighted_slots(
    words: &[u32],
    weights: &[f32; SLOT_COUNT],
) -> Result<Vec<WeightedSlot>, CoefficientNormalEquationError> {
    let mut slots = Vec::with_capacity(words.len() / (BIN_COUNT * COMPONENT_COUNT));
    for row in words.chunks_exact(SLOT_COUNT * BIN_COUNT * COMPONENT_COUNT) {
        let mut finite = true;
        for (slot, &weight) in row.chunks_exact(BIN_COUNT * COMPONENT_COUNT).zip(weights) {
            let mut result = WeightedSlot {
                real: [0.0; BIN_COUNT],
                imaginary: [0.0; BIN_COUNT],
            };
            for (bin, pair) in slot.chunks_exact(COMPONENT_COUNT).enumerate() {
                let real = f32::from_bits(pair[0]) * weight;
                let imaginary = f32::from_bits(pair[1]) * weight;
                finite &= real.is_finite() & imaginary.is_finite();
                result.real[bin] = real;
                result.imaginary[bin] = imaginary;
            }
            slots.push(result);
        }
        if !finite {
            // Match the original bin/slot/component error precedence on invalid input.
            for bin in 0..BIN_COUNT {
                for (slot, &weight) in weights.iter().enumerate() {
                    let offset = (slot * BIN_COUNT + bin) * COMPONENT_COUNT;
                    for &word in &row[offset..offset + COMPONENT_COUNT] {
                        let value = f32::from_bits(word);
                        if !value.is_finite() {
                            return Err(CoefficientNormalEquationError::InputNonFinite);
                        }
                        if !(value * weight).is_finite() {
                            return Err(CoefficientNormalEquationError::NumericNonFinite);
                        }
                    }
                }
            }
        }
    }
    Ok(slots)
}

fn band_dots(
    left: &[WeightedSlot],
    left_row: usize,
    right: &[WeightedSlot],
    right_row: usize,
    initial: u32,
) -> Result<[u32; BAND_COUNT], CoefficientNormalEquationError> {
    let left = &left[left_row * SLOT_COUNT..(left_row + 1) * SLOT_COUNT];
    let right = &right[right_row * SLOT_COUNT..(right_row + 1) * SLOT_COUNT];
    let mut dots = [0.0; BIN_COUNT];
    for (a, b) in left.iter().zip(right) {
        // Bins are independent SIMD lanes. Within a bin every slot contributes
        // two separately rounded products, in real then imaginary order.
        for (bin, dot) in dots.iter_mut().enumerate() {
            *dot = (a.real[bin] * b.real[bin] + *dot) + a.imaginary[bin] * b.imaginary[bin];
        }
    }
    let mut result = [ZERO_BITS; BAND_COUNT];
    for (word, bounds) in result.iter_mut().zip(BAND_BOUNDARIES.windows(2)) {
        let mut sum = f32::from_bits(initial);
        for &dot in &dots[bounds[0]..bounds[1]] {
            sum += dot;
        }
        // Only additions follow each product. Infinity and NaN cannot return to a finite sum.
        if !sum.is_finite() {
            return Err(CoefficientNormalEquationError::NumericNonFinite);
        }
        *word = sum.to_bits();
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joc::temporal_weights::TemporalWeightState;
    use serde_json::Value;

    const INPUT: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_qmf_fit_checkpoints/INPUT-ONLY.json");
    const EXPECTED: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_qmf_fit_checkpoints/EXPECTED.json");

    #[test]
    fn weighting_errors_follow_bin_then_slot_order() {
        let mut targets = vec![ZERO_BITS; OBJECT_COUNT * SLOT_COUNT * BIN_COUNT * COMPONENT_COUNT];
        let carriers =
            vec![ZERO_BITS; SELECTED_CARRIER_COUNT * SLOT_COUNT * BIN_COUNT * COMPONENT_COUNT];
        let weights = [2.0_f32.to_bits(); SLOT_COUNT];
        // Slot one, bin zero is admitted before slot zero, bin one.
        targets[BIN_COUNT * COMPONENT_COUNT] = f32::MAX.to_bits();
        targets[COMPONENT_COUNT] = f32::NAN.to_bits();
        assert_eq!(
            build_selected_normal_equations(&targets, &carriers, &weights),
            Err(CoefficientNormalEquationError::NumericNonFinite)
        );
        targets.swap(BIN_COUNT * COMPONENT_COUNT, COMPONENT_COUNT);
        assert_eq!(
            build_selected_normal_equations(&targets, &carriers, &weights),
            Err(CoefficientNormalEquationError::InputNonFinite)
        );
    }

    #[test]
    fn seven_carrier_normal_equations_have_seven_rows_and_columns() {
        let target_words = vec![ZERO_BITS; OBJECT_COUNT * SLOT_COUNT * BIN_COUNT * COMPONENT_COUNT];
        let carrier_words = vec![ZERO_BITS; 7 * SLOT_COUNT * BIN_COUNT * COMPONENT_COUNT];
        let weight_words = [1.0_f32.to_bits(); SLOT_COUNT];
        let equations =
            build_normal_equations_for_carriers::<7>(&target_words, &carrier_words, &weight_words)
                .unwrap();

        assert_eq!(equations.len(), OBJECT_COUNT * BAND_COUNT);
        assert_eq!(equations[0].gram_words().len(), 7);
        assert_eq!(equations[0].gram_words()[0].len(), 7);
        assert_eq!(equations[0].cross_words().len(), 7);
    }

    #[test]
    fn three_generations_match_all_five_hundred_forty_gram_and_cross_checkpoints() {
        let input: Value = serde_json::from_slice(INPUT).unwrap();
        let expected: Value = serde_json::from_slice(EXPECTED).unwrap();
        let mut temporal = TemporalWeightState::cold();
        let half_words = OBJECT_COUNT * 24 * BIN_COUNT * 2;
        let carrier_half_words = SELECTED_CARRIER_COUNT * 24 * BIN_COUNT * 2;
        let mut previous_targets = vec![ZERO_BITS; half_words];
        let mut previous_carriers = vec![ZERO_BITS; carrier_half_words];

        for generation in 0..3 {
            let current_carriers = synthetic_carrier_current24(&input, generation);
            let current_targets = synthetic_target_current24(&input, generation, &current_carriers);
            let target_windows =
                concatenate_windows(&previous_targets, &current_targets, OBJECT_COUNT);
            let carrier_windows = concatenate_windows(
                &previous_carriers,
                &current_carriers,
                SELECTED_CARRIER_COUNT,
            );
            let temporal_proposal = temporal.stage_target_windows48(&target_windows).unwrap();
            let equations = build_selected_normal_equations(
                &target_windows,
                &carrier_windows,
                temporal_proposal.after().weight_words(),
            )
            .unwrap();
            let expected_cells = expected["checkpoints"][generation]["cells"]
                .as_array()
                .unwrap();
            assert_eq!(equations.len(), 180);
            assert_eq!(expected_cells.len(), 180);
            for (actual, expected) in equations.iter().zip(expected_cells) {
                assert_eq!(
                    u64::from(actual.object),
                    expected["object"].as_u64().unwrap()
                );
                assert_eq!(u64::from(actual.band), expected["band"].as_u64().unwrap());
                assert_eq!(
                    u64::from(actual.begin_bin),
                    expected["begin"].as_u64().unwrap()
                );
                assert_eq!(u64::from(actual.end_bin), expected["end"].as_u64().unwrap());
                for row in 0..SELECTED_CARRIER_COUNT {
                    assert_eq!(
                        actual.cross_words[row],
                        f64_wrapper_to_f32_word(&expected["cross"][row]),
                        "generation {generation} object {} band {} cross {row}",
                        actual.object,
                        actual.band
                    );
                    for column in 0..SELECTED_CARRIER_COUNT {
                        assert_eq!(
                            actual.gram_words[row][column],
                            f64_wrapper_to_f32_word(&expected["gram"][row][column]),
                            "generation {generation} object {} band {} gram {row},{column}",
                            actual.object,
                            actual.band
                        );
                    }
                }
            }
            temporal = temporal_proposal.into_after();
            previous_targets = current_targets;
            previous_carriers = current_carriers;
        }
    }

    fn synthetic_carrier_current24(input: &Value, generation: usize) -> Vec<u32> {
        let amplitude = input["generations"][generation]["amplitude"]
            .as_f64()
            .unwrap() as f32;
        let mut output = Vec::with_capacity(SELECTED_CARRIER_COUNT * 24 * BIN_COUNT * 2);
        for carrier in 0..SELECTED_CARRIER_COUNT {
            for slot in 0..24 {
                for bin in 0..BIN_COUNT {
                    let value = synthetic_carrier(carrier, slot, bin, amplitude);
                    output.extend(value.map(f32::to_bits));
                }
            }
        }
        output
    }

    fn synthetic_target_current24(input: &Value, generation: usize, carriers: &[u32]) -> Vec<u32> {
        let transient_slot = input["generations"][generation]["transient_slot"]
            .as_u64()
            .unwrap() as usize;
        let multiplier = input["generations"][generation]["transient_multiplier"]
            .as_f64()
            .unwrap() as f32;
        let mut output = Vec::with_capacity(OBJECT_COUNT * 24 * BIN_COUNT * 2);
        for object in 0..OBJECT_COUNT {
            let (alpha, beta) = match object % 3 {
                0 => (0.75_f32, 0.0_f32),
                1 => (0.75_f32, 0.5_f32),
                _ => (0.5_f32, -0.25_f32),
            };
            for slot in 0..24 {
                for bin in 0..BIN_COUNT {
                    let carrier_a = current_complex(carriers, object % 5, slot, bin);
                    let carrier_z = current_complex(carriers, (object + 2) % 5, slot, bin);
                    let scale = if generation == 1 && object == 0 && slot >= transient_slot {
                        multiplier
                    } else {
                        1.0
                    };
                    for component in 0..2 {
                        let first = alpha * carrier_a[component];
                        let second = beta * carrier_z[component];
                        output.push(((first + second) * scale).to_bits());
                    }
                }
            }
        }
        output
    }

    fn current_complex(words: &[u32], row: usize, slot: usize, bin: usize) -> [f32; 2] {
        let offset = ((row * 24 + slot) * BIN_COUNT + bin) * 2;
        [
            f32::from_bits(words[offset]),
            f32::from_bits(words[offset + 1]),
        ]
    }

    fn synthetic_carrier(carrier: usize, slot: usize, bin: usize, amplitude: f32) -> [f32; 2] {
        let bit_index = if carrier == 1 && bin == 0 { 0 } else { carrier };
        let factor = 1.0_f32 + (bin % 3) as f32 / 4.0_f32;
        let real_sign = if (slot >> bit_index) & 1 == 0 {
            1.0
        } else {
            -1.0
        };
        let imaginary_bit = (bit_index + 1) % 5;
        let imaginary_sign = if ((slot + 1) >> imaginary_bit) & 1 == 0 {
            1.0
        } else {
            -1.0
        };
        [
            (amplitude * factor) * real_sign,
            (amplitude / 2.0_f32) * imaginary_sign,
        ]
    }

    fn concatenate_windows(previous: &[u32], current: &[u32], rows: usize) -> Vec<u32> {
        let words_per_half = 24 * BIN_COUNT * 2;
        let mut output = Vec::with_capacity(rows * words_per_half * 2);
        for row in 0..rows {
            let start = row * words_per_half;
            let end = start + words_per_half;
            output.extend_from_slice(&previous[start..end]);
            output.extend_from_slice(&current[start..end]);
        }
        output
    }

    fn f64_wrapper_to_f32_word(value: &Value) -> u32 {
        let bytes = decode_hex(value["f64le"].as_str().unwrap());
        (f64::from_le_bytes(bytes.try_into().unwrap()) as f32).to_bits()
    }

    fn decode_hex(hex: &str) -> Vec<u8> {
        hex.as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }
}
