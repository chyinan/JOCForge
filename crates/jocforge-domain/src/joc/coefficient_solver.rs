// pattern: Functional Core
//! Selected signed regularized LDL coefficient solve for one fit cell.

use crate::{
    joc::coefficient_normal_equations::CoefficientNormalEquation,
    numeric::{FiniteF32, r32_add, r32_mul, r32_sub},
};

const CHANNEL_COUNT: usize = 5;
const ZERO_BITS: u32 = 0x0000_0000;
const ONE_BITS: u32 = 0x3f80_0000;
const REGULARIZER_SCALE_BITS: u32 = 0x3c23_d70a;
const SOLVER_TINY_BITS: u32 = 0x2720_0000;
const STORAGE_SCALE_BITS: u32 = 0x3d80_0000;
const SLOT_COUNT: usize = 48;
const BIN_COUNT: usize = 64;
const COMPONENT_COUNT: usize = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoefficientSolution {
    support: [bool; CHANNEL_COUNT],
    coefficient_words: [u32; CHANNEL_COUNT],
}

impl CoefficientSolution {
    pub const fn support(&self) -> &[bool; CHANNEL_COUNT] {
        &self.support
    }

    pub const fn coefficient_words(&self) -> &[u32; CHANNEL_COUNT] {
        &self.coefficient_words
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NormalizedCoefficientSolution {
    support: [bool; CHANNEL_COUNT],
    normalized_words: [u32; CHANNEL_COUNT],
    stored_words: [u32; CHANNEL_COUNT],
}

impl NormalizedCoefficientSolution {
    pub const fn support(&self) -> &[bool; CHANNEL_COUNT] {
        &self.support
    }

    pub const fn normalized_words(&self) -> &[u32; CHANNEL_COUNT] {
        &self.normalized_words
    }

    pub const fn stored_words(&self) -> &[u32; CHANNEL_COUNT] {
        &self.stored_words
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoefficientSolverError {
    EmptySupport,
    InputShape,
    InputNonFinite,
    NumericNonFinite,
}

impl CoefficientSolverError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmptySupport => "JOC_FIT_EMPTY_INITIAL_ELIGIBILITY",
            Self::InputShape => "JOC_FIT_INPUT_SHAPE",
            Self::InputNonFinite => "JOC_FIT_EQUATION_NONFINITE",
            Self::NumericNonFinite => "JOC_FIT_SOLVER_NUMERIC_NONFINITE",
        }
    }
}

impl std::fmt::Display for CoefficientSolverError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for CoefficientSolverError {}

/// Solves one support mask in ascending channel order. The source equation is
/// immutable and unsuccessful candidates expose no partial coefficient vector.
#[allow(clippy::needless_range_loop)] // The selected LDL operation order is index-defined.
pub fn solve_selected_support(
    equation: &CoefficientNormalEquation,
    support: [bool; CHANNEL_COUNT],
) -> Result<CoefficientSolution, CoefficientSolverError> {
    let active = support
        .iter()
        .enumerate()
        .filter_map(|(channel, enabled)| enabled.then_some(channel))
        .collect::<Vec<_>>();
    if active.is_empty() {
        return Err(CoefficientSolverError::EmptySupport);
    }
    let zero = finite_word(ZERO_BITS)?;
    let one = finite_word(ONE_BITS)?;
    let tiny = finite_word(SOLVER_TINY_BITS)?;
    let mut matrix = [[zero; CHANNEL_COUNT]; CHANNEL_COUNT];
    let mut cross = [zero; CHANNEL_COUNT];
    let mut trace = zero;
    for (row, channel_row) in active.iter().copied().enumerate() {
        cross[row] = finite_word(equation.cross_words()[channel_row])?;
        trace = add(
            trace,
            finite_word(equation.gram_words()[channel_row][channel_row])?,
        )?;
        for (column, channel_column) in active.iter().copied().enumerate() {
            matrix[row][column] = finite_word(equation.gram_words()[channel_row][channel_column])?;
        }
    }
    let regularizer = multiply(trace, finite_word(REGULARIZER_SCALE_BITS)?)?;
    for index in 0..active.len() {
        matrix[index][index] = add(add(matrix[index][index], regularizer)?, tiny)?;
    }

    let mut lower = [[zero; CHANNEL_COUNT]; CHANNEL_COUNT];
    let mut diagonal = [zero; CHANNEL_COUNT];
    for index in 0..active.len() {
        lower[index][index] = one;
    }
    for column in 0..active.len() {
        let mut diagonal_value = matrix[column][column];
        for prior in 0..column {
            let square = multiply(lower[column][prior], lower[column][prior])?;
            diagonal_value = subtract(diagonal_value, multiply(square, diagonal[prior])?)?;
        }
        diagonal[column] = if diagonal_value.value() < tiny.value() {
            tiny
        } else {
            diagonal_value
        };
        for row in column + 1..active.len() {
            let mut value = matrix[row][column];
            for prior in 0..column {
                let product = multiply(lower[column][prior], lower[row][prior])?;
                value = subtract(value, multiply(product, diagonal[prior])?)?;
            }
            lower[row][column] = divide(value, diagonal[column])?;
        }
    }

    let mut forward = [zero; CHANNEL_COUNT];
    for row in 0..active.len() {
        let mut value = cross[row];
        for prior in 0..row {
            value = subtract(value, multiply(lower[row][prior], forward[prior])?)?;
        }
        forward[row] = value;
    }
    let mut divided = [zero; CHANNEL_COUNT];
    for index in 0..active.len() {
        divided[index] = divide(forward[index], diagonal[index])?;
    }
    let mut solved = [zero; CHANNEL_COUNT];
    for row in (0..active.len()).rev() {
        let mut value = divided[row];
        for later in row + 1..active.len() {
            value = subtract(value, multiply(lower[later][row], solved[later])?)?;
        }
        solved[row] = value;
    }

    let mut coefficient_words = [ZERO_BITS; CHANNEL_COUNT];
    for (active_index, channel) in active.into_iter().enumerate() {
        coefficient_words[channel] = solved[active_index].to_bits();
    }
    Ok(CoefficientSolution {
        support,
        coefficient_words,
    })
}

/// Normalizes one solved cell using the selected slot/bin accumulation order,
/// then stores supported coefficients in the 1/16 domain exactly once.
#[allow(clippy::needless_range_loop)] // The selected slot/channel order is index-defined.
pub fn normalize_selected_solution(
    equation: &CoefficientNormalEquation,
    solution: &CoefficientSolution,
    target_words: &[u32],
    carrier_words: &[u32],
    weight_words: &[u32; SLOT_COUNT],
) -> Result<NormalizedCoefficientSolution, CoefficientSolverError> {
    if target_words.len() != 15 * SLOT_COUNT * BIN_COUNT * COMPONENT_COUNT
        || carrier_words.len() != CHANNEL_COUNT * SLOT_COUNT * BIN_COUNT * COMPONENT_COUNT
    {
        return Err(CoefficientSolverError::InputShape);
    }
    let object = usize::from(equation.object());
    let begin = usize::from(equation.begin_bin());
    let end = usize::from(equation.end_bin());
    let mut target_energy = finite_word(ZERO_BITS)?;
    let mut model_energy = finite_word(ZERO_BITS)?;

    for slot in 0..SLOT_COUNT {
        let weight = finite_word(weight_words[slot])?;
        let mut slot_target = finite_word(ZERO_BITS)?;
        let mut slot_model = finite_word(ZERO_BITS)?;
        for bin in begin..end {
            let target = complex(target_words, object, 15, slot, bin)?;
            let target_real = multiply(target[0], weight)?;
            let target_imaginary = multiply(target[1], weight)?;
            slot_target = add(slot_target, multiply(target_real, target_real)?)?;
            slot_target = add(slot_target, multiply(target_imaginary, target_imaginary)?)?;

            let mut model_real = finite_word(ZERO_BITS)?;
            let mut model_imaginary = finite_word(ZERO_BITS)?;
            for channel in 0..CHANNEL_COUNT {
                let coefficient = finite_word(solution.coefficient_words[channel])?;
                if !solution.support[channel] || coefficient.value() == 0.0 {
                    continue;
                }
                let carrier = complex(carrier_words, channel, CHANNEL_COUNT, slot, bin)?;
                let carrier_real = multiply(carrier[0], weight)?;
                let carrier_imaginary = multiply(carrier[1], weight)?;
                model_real = add(model_real, multiply(coefficient, carrier_real)?)?;
                model_imaginary = add(model_imaginary, multiply(coefficient, carrier_imaginary)?)?;
            }
            slot_model = add(slot_model, multiply(model_real, model_real)?)?;
            slot_model = add(slot_model, multiply(model_imaginary, model_imaginary)?)?;
        }
        target_energy = add(target_energy, slot_target)?;
        model_energy = add(model_energy, slot_model)?;
    }

    let denominator = add(model_energy, finite_word(SOLVER_TINY_BITS)?)?;
    let ratio = divide(target_energy, denominator)?;
    let scale = square_root(ratio)?;
    let storage_scale = finite_word(STORAGE_SCALE_BITS)?;
    let mut normalized_words = [ZERO_BITS; CHANNEL_COUNT];
    let mut stored_words = [ZERO_BITS; CHANNEL_COUNT];
    for channel in 0..CHANNEL_COUNT {
        if !solution.support[channel] {
            continue;
        }
        let normalized = multiply(finite_word(solution.coefficient_words[channel])?, scale)?;
        normalized_words[channel] = normalized.to_bits();
        stored_words[channel] = multiply(normalized, storage_scale)?.to_bits();
    }
    Ok(NormalizedCoefficientSolution {
        support: solution.support,
        normalized_words,
        stored_words,
    })
}

fn finite_word(word: u32) -> Result<FiniteF32, CoefficientSolverError> {
    FiniteF32::new(f32::from_bits(word)).map_err(|_| CoefficientSolverError::InputNonFinite)
}

fn multiply(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, CoefficientSolverError> {
    r32_mul(left, right).map_err(|_| CoefficientSolverError::NumericNonFinite)
}

fn add(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, CoefficientSolverError> {
    r32_add(left, right).map_err(|_| CoefficientSolverError::NumericNonFinite)
}

fn subtract(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, CoefficientSolverError> {
    r32_sub(left, right).map_err(|_| CoefficientSolverError::NumericNonFinite)
}

fn divide(
    numerator: FiniteF32,
    denominator: FiniteF32,
) -> Result<FiniteF32, CoefficientSolverError> {
    FiniteF32::new(numerator.value() / denominator.value())
        .map_err(|_| CoefficientSolverError::NumericNonFinite)
}

fn square_root(value: FiniteF32) -> Result<FiniteF32, CoefficientSolverError> {
    FiniteF32::new(value.value().sqrt()).map_err(|_| CoefficientSolverError::NumericNonFinite)
}

fn complex(
    words: &[u32],
    row: usize,
    row_count: usize,
    slot: usize,
    bin: usize,
) -> Result<[FiniteF32; 2], CoefficientSolverError> {
    if row >= row_count || slot >= SLOT_COUNT || bin >= BIN_COUNT {
        return Err(CoefficientSolverError::InputShape);
    }
    let offset = ((row * SLOT_COUNT + slot) * BIN_COUNT + bin) * COMPONENT_COUNT;
    Ok([finite_word(words[offset])?, finite_word(words[offset + 1])?])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joc::{
        coefficient_normal_equations::build_selected_normal_equations,
        coefficient_quantization::{
            CoefficientPrecision, quantize_selected_coefficient, restore_trial_coefficient,
        },
        fit_score::{FitScoreProvider, score_trial_coefficients},
    };
    use serde_json::Value;
    use sha2::{Digest, Sha256};

    const INPUT: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_fit_cell_boundaries/INPUT-ONLY.json");
    const EXPECTED: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_fit_cell_boundaries/EXPECTED.json");
    const MANIFEST: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_fit_cell_boundaries/WASH-MANIFEST.json");
    const SCORE_FUNCTIONAL: &[u8] = include_bytes!(
        "../../tests/fixtures/synthetic_qmf_fit_checkpoints/approved-joc-functional.json"
    );
    const SCORE_MATH: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_qmf_fit_checkpoints/approved-joc-math.json");
    const PAYLOADS: &[(&str, &[u8])] = &[
        (
            "approved-joc-math.json",
            include_bytes!(
                "../../tests/fixtures/synthetic_fit_cell_boundaries/approved-joc-math.json"
            ),
        ),
        (
            "CONTRACT.md",
            include_bytes!("../../tests/fixtures/synthetic_fit_cell_boundaries/CONTRACT.md"),
        ),
        ("EXPECTED.json", EXPECTED),
        ("INPUT-ONLY.json", INPUT),
    ];

    #[test]
    fn washed_fit_boundary_payloads_match_their_manifest() {
        let manifest: Value = serde_json::from_slice(MANIFEST).unwrap();
        for entry in manifest["payloads"].as_array().unwrap() {
            let name = entry["file"].as_str().unwrap();
            let bytes = PAYLOADS
                .iter()
                .find_map(|(candidate, bytes)| (*candidate == name).then_some(*bytes))
                .unwrap();
            assert_eq!(bytes.len() as u64, entry["bytes"].as_u64().unwrap());
            assert_eq!(
                format!("{:x}", Sha256::digest(bytes)),
                entry["sha256"].as_str().unwrap()
            );
        }
    }

    #[test]
    fn initial_single_channel_candidates_match_all_nonempty_boundary_cases() {
        let input: Value = serde_json::from_slice(INPUT).unwrap();
        let expected: Value = serde_json::from_slice(EXPECTED).unwrap();
        let score_provider =
            FitScoreProvider::from_approved_bytes(SCORE_FUNCTIONAL, SCORE_MATH).unwrap();
        for (case, expected) in input["cases"]
            .as_array()
            .unwrap()
            .iter()
            .zip(expected["results"].as_array().unwrap())
        {
            let eligibility = bool_array(&case["eligibility"]);
            if !eligibility.iter().any(|value| *value) {
                assert_eq!(
                    expected["error"],
                    "FIXTURE_REJECT_EMPTY_INITIAL_ELIGIBILITY"
                );
                let (equation, _targets, _carriers) = synthetic_cell(case);
                assert_eq!(
                    solve_selected_support(&equation, eligibility),
                    Err(CoefficientSolverError::EmptySupport)
                );
                continue;
            }
            let (equation, targets, carriers) = synthetic_cell(case);
            let candidates = expected["initial_single_candidates"].as_array().unwrap();
            assert_eq!(
                candidates.len(),
                eligibility.iter().filter(|value| **value).count()
            );
            for candidate in candidates {
                let channel = candidate["channel"].as_u64().unwrap() as usize;
                let mut support = [false; CHANNEL_COUNT];
                support[channel] = true;
                let solution = solve_selected_support(&equation, support).unwrap();
                let normalized = normalize_selected_solution(
                    &equation,
                    &solution,
                    &targets,
                    &carriers,
                    &[ONE_BITS; SLOT_COUNT],
                )
                .unwrap();
                let precision = CoefficientPrecision::from_wire_mode(
                    u8::try_from(case["trial_mode"].as_u64().unwrap()).unwrap(),
                )
                .unwrap();
                assert_eq!(solution.support(), &support);
                let mut trial_words = [ZERO_BITS; CHANNEL_COUNT];
                for index in 0..CHANNEL_COUNT {
                    assert_eq!(
                        solution.coefficient_words()[index],
                        f64_or_zero_to_f32_word(&candidate["solved"][index]),
                        "{} channel {channel} coefficient {index}",
                        case["name"].as_str().unwrap()
                    );
                    assert_eq!(
                        normalized.normalized_words()[index],
                        f64_or_zero_to_f32_word(&candidate["normalized"][index]),
                        "{} channel {channel} normalized {index}",
                        case["name"].as_str().unwrap()
                    );
                    assert_eq!(
                        normalized.stored_words()[index],
                        f64_or_zero_to_f32_word(&candidate["stored"][index]),
                        "{} channel {channel} stored {index}",
                        case["name"].as_str().unwrap()
                    );
                    let quantized = quantize_selected_coefficient(
                        f32::from_bits(normalized.stored_words()[index]),
                        precision,
                    )
                    .unwrap();
                    trial_words[index] = restore_trial_coefficient(quantized, precision)
                        .unwrap()
                        .to_bits();
                    assert_eq!(
                        trial_words[index],
                        f64_or_zero_to_f32_word(&candidate["trial"][index]),
                        "{} channel {channel} trial {index}",
                        case["name"].as_str().unwrap()
                    );
                }
                let score = score_trial_coefficients(
                    &score_provider,
                    &equation,
                    &targets,
                    &carriers,
                    &[ONE_BITS; SLOT_COUNT],
                    &trial_words,
                )
                .unwrap();
                assert_eq!(
                    score.error().to_bits(),
                    f64_wrapper_word(&candidate["terms"]["error"])
                );
                assert_eq!(
                    score.model_energy().to_bits(),
                    f64_wrapper_word(&candidate["terms"]["model_energy"])
                );
                assert_eq!(
                    score.target_energy().to_bits(),
                    f64_wrapper_word(&candidate["terms"]["target_energy"])
                );
                assert_eq!(
                    score.ratio().to_bits(),
                    f64_wrapper_word(&candidate["terms"]["ratio"])
                );
                assert_eq!(
                    score.score().to_bits(),
                    f64_wrapper_word(&candidate["score"])
                );
            }
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn synthetic_cell(case: &Value) -> (CoefficientNormalEquation, Vec<u32>, Vec<u32>) {
        let selected_bin = case["band"].as_u64().unwrap() as usize;
        let amplitude = case["amplitude"].as_f64().unwrap() as f32;
        let duplicate = case["duplicate01"].as_bool().unwrap();
        let mix = case["target_mix"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_f64().unwrap() as f32)
            .collect::<Vec<_>>();
        let mut carriers = vec![ZERO_BITS; CHANNEL_COUNT * 48 * 64 * 2];
        let mut targets = vec![ZERO_BITS; 15 * 48 * 64 * 2];
        for slot in 0..48 {
            let mut carrier_values = [0.0_f32; CHANNEL_COUNT];
            for channel in 0..CHANNEL_COUNT {
                let value = if channel == 0 || (channel == 1 && duplicate) {
                    amplitude
                } else {
                    let bit = (slot >> (channel - 1)) & 1;
                    amplitude * if bit == 0 { 1.0 } else { -1.0 }
                };
                carrier_values[channel] = value;
                let offset = ((channel * 48 + slot) * 64 + selected_bin) * 2;
                carriers[offset] = value.to_bits();
            }
            let mut target = 0.0_f32;
            for channel in 0..CHANNEL_COUNT {
                let product = carrier_values[channel] * mix[channel];
                target += product;
            }
            let offset = (slot * 64 + selected_bin) * 2;
            targets[offset] = target.to_bits();
        }
        let equations =
            build_selected_normal_equations(&targets, &carriers, &[ONE_BITS; 48]).unwrap();
        let equation = equations
            .into_iter()
            .find(|equation| {
                usize::from(equation.begin_bin()) <= selected_bin
                    && selected_bin < usize::from(equation.end_bin())
            })
            .unwrap();
        (equation, targets, carriers)
    }

    fn bool_array(value: &Value) -> [bool; CHANNEL_COUNT] {
        std::array::from_fn(|index| value[index].as_u64().unwrap() != 0)
    }

    fn f64_or_zero_to_f32_word(value: &Value) -> u32 {
        if value.is_number() {
            return ZERO_BITS;
        }
        let bytes = decode_hex(value["f64le"].as_str().unwrap());
        (f64::from_le_bytes(bytes.try_into().unwrap()) as f32).to_bits()
    }

    fn f64_wrapper_word(value: &Value) -> u64 {
        let bytes = decode_hex(value["f64le"].as_str().unwrap());
        u64::from_le_bytes(bytes.try_into().unwrap())
    }

    fn decode_hex(hex: &str) -> Vec<u8> {
        hex.as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }
}
