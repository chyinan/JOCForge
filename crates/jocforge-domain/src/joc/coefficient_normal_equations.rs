// pattern: Functional Core
//! Weighted Gram and cross equations for the selected JOC coefficient fit.

use crate::numeric::{FiniteF32, r32_add, r32_mul};

const OBJECT_COUNT: usize = 15;
const CARRIER_COUNT: usize = 5;
const SLOT_COUNT: usize = 48;
const BIN_COUNT: usize = 64;
const COMPONENT_COUNT: usize = 2;
const BAND_COUNT: usize = 12;
const BAND_BOUNDARIES: [usize; BAND_COUNT + 1] = [0, 1, 2, 3, 4, 6, 8, 11, 14, 18, 23, 35, 64];
const ZERO_BITS: u32 = 0x0000_0000;
const GRAM_DIAGONAL_TINY_BITS: u32 = 0x1e3c_e508;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoefficientNormalEquation {
    object: u8,
    band: u8,
    begin_bin: u8,
    end_bin: u8,
    gram_words: [[u32; CARRIER_COUNT]; CARRIER_COUNT],
    cross_words: [u32; CARRIER_COUNT],
}

impl CoefficientNormalEquation {
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
) -> Result<Vec<CoefficientNormalEquation>, CoefficientNormalEquationError> {
    if target_words.len() != OBJECT_COUNT * SLOT_COUNT * BIN_COUNT * COMPONENT_COUNT
        || carrier_words.len() != CARRIER_COUNT * SLOT_COUNT * BIN_COUNT * COMPONENT_COUNT
    {
        return Err(CoefficientNormalEquationError::InputShape);
    }
    let weights = weight_words
        .iter()
        .map(|word| finite_word(*word))
        .collect::<Result<Vec<_>, _>>()?;
    let mut equations = Vec::with_capacity(OBJECT_COUNT * BAND_COUNT);

    for object in 0..OBJECT_COUNT {
        for band in 0..BAND_COUNT {
            let begin = BAND_BOUNDARIES[band];
            let end = BAND_BOUNDARIES[band + 1];
            let mut gram_words = [[ZERO_BITS; CARRIER_COUNT]; CARRIER_COUNT];
            for row in 0..CARRIER_COUNT {
                for column in row..CARRIER_COUNT {
                    let mut gram = finite_word(if row == column {
                        GRAM_DIAGONAL_TINY_BITS
                    } else {
                        ZERO_BITS
                    })?;
                    for bin in begin..end {
                        let mut sub = finite_word(ZERO_BITS)?;
                        for slot in 0..SLOT_COUNT {
                            let left = carrier_complex(carrier_words, row, slot, bin)?;
                            let right = carrier_complex(carrier_words, column, slot, bin)?;
                            let left_real = multiply(left[0], weights[slot])?;
                            let right_real = multiply(right[0], weights[slot])?;
                            let left_imaginary = multiply(left[1], weights[slot])?;
                            let right_imaginary = multiply(right[1], weights[slot])?;
                            sub = add(multiply(left_real, right_real)?, sub)?;
                            sub = add(sub, multiply(left_imaginary, right_imaginary)?)?;
                        }
                        gram = add(gram, sub)?;
                    }
                    gram_words[row][column] = gram.to_bits();
                    gram_words[column][row] = gram.to_bits();
                }
            }

            let mut cross_words = [ZERO_BITS; CARRIER_COUNT];
            for carrier in 0..CARRIER_COUNT {
                let mut cross = finite_word(ZERO_BITS)?;
                for bin in begin..end {
                    let mut sub = finite_word(ZERO_BITS)?;
                    for slot in 0..SLOT_COUNT {
                        let target = target_complex(target_words, object, slot, bin)?;
                        let carrier_value = carrier_complex(carrier_words, carrier, slot, bin)?;
                        let target_real = multiply(target[0], weights[slot])?;
                        let carrier_real = multiply(carrier_value[0], weights[slot])?;
                        let target_imaginary = multiply(target[1], weights[slot])?;
                        let carrier_imaginary = multiply(carrier_value[1], weights[slot])?;
                        sub = add(multiply(target_real, carrier_real)?, sub)?;
                        sub = add(sub, multiply(target_imaginary, carrier_imaginary)?)?;
                    }
                    cross = add(cross, sub)?;
                }
                cross_words[carrier] = cross.to_bits();
            }

            equations.push(CoefficientNormalEquation {
                object: u8::try_from(object)
                    .map_err(|_| CoefficientNormalEquationError::InputShape)?,
                band: u8::try_from(band).map_err(|_| CoefficientNormalEquationError::InputShape)?,
                begin_bin: u8::try_from(begin)
                    .map_err(|_| CoefficientNormalEquationError::InputShape)?,
                end_bin: u8::try_from(end)
                    .map_err(|_| CoefficientNormalEquationError::InputShape)?,
                gram_words,
                cross_words,
            });
        }
    }
    Ok(equations)
}

fn target_complex(
    words: &[u32],
    object: usize,
    slot: usize,
    bin: usize,
) -> Result<[FiniteF32; 2], CoefficientNormalEquationError> {
    complex(words, object, slot, bin, OBJECT_COUNT)
}

fn carrier_complex(
    words: &[u32],
    carrier: usize,
    slot: usize,
    bin: usize,
) -> Result<[FiniteF32; 2], CoefficientNormalEquationError> {
    complex(words, carrier, slot, bin, CARRIER_COUNT)
}

fn complex(
    words: &[u32],
    row: usize,
    slot: usize,
    bin: usize,
    row_count: usize,
) -> Result<[FiniteF32; 2], CoefficientNormalEquationError> {
    if row >= row_count || slot >= SLOT_COUNT || bin >= BIN_COUNT {
        return Err(CoefficientNormalEquationError::InputShape);
    }
    let offset = ((row * SLOT_COUNT + slot) * BIN_COUNT + bin) * COMPONENT_COUNT;
    Ok([finite_word(words[offset])?, finite_word(words[offset + 1])?])
}

fn finite_word(word: u32) -> Result<FiniteF32, CoefficientNormalEquationError> {
    FiniteF32::new(f32::from_bits(word)).map_err(|_| CoefficientNormalEquationError::InputNonFinite)
}

fn multiply(
    left: FiniteF32,
    right: FiniteF32,
) -> Result<FiniteF32, CoefficientNormalEquationError> {
    r32_mul(left, right).map_err(|_| CoefficientNormalEquationError::NumericNonFinite)
}

fn add(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, CoefficientNormalEquationError> {
    r32_add(left, right).map_err(|_| CoefficientNormalEquationError::NumericNonFinite)
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
    fn three_generations_match_all_five_hundred_forty_gram_and_cross_checkpoints() {
        let input: Value = serde_json::from_slice(INPUT).unwrap();
        let expected: Value = serde_json::from_slice(EXPECTED).unwrap();
        let mut temporal = TemporalWeightState::cold();
        let half_words = OBJECT_COUNT * 24 * BIN_COUNT * 2;
        let carrier_half_words = CARRIER_COUNT * 24 * BIN_COUNT * 2;
        let mut previous_targets = vec![ZERO_BITS; half_words];
        let mut previous_carriers = vec![ZERO_BITS; carrier_half_words];

        for generation in 0..3 {
            let current_carriers = synthetic_carrier_current24(&input, generation);
            let current_targets = synthetic_target_current24(&input, generation, &current_carriers);
            let target_windows =
                concatenate_windows(&previous_targets, &current_targets, OBJECT_COUNT);
            let carrier_windows =
                concatenate_windows(&previous_carriers, &current_carriers, CARRIER_COUNT);
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
                for row in 0..CARRIER_COUNT {
                    assert_eq!(
                        actual.cross_words[row],
                        f64_wrapper_to_f32_word(&expected["cross"][row]),
                        "generation {generation} object {} band {} cross {row}",
                        actual.object,
                        actual.band
                    );
                    for column in 0..CARRIER_COUNT {
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
        let mut output = Vec::with_capacity(CARRIER_COUNT * 24 * BIN_COUNT * 2);
        for carrier in 0..CARRIER_COUNT {
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
