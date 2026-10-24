// pattern: Functional Core
//! Selected multi-stage coefficient candidate enumeration and strict score search.

use crate::joc::{
    coefficient_normal_equations::CoefficientNormalEquation,
    coefficient_quantization::{
        CoefficientPrecision, CoefficientQuantizationError, quantize_selected_coefficient,
        restore_trial_coefficient,
    },
    coefficient_solver::{
        CoefficientSolverError, normalize_selected_solution, solve_selected_support,
    },
    fit_score::{FitScoreError, FitScoreEvaluation, FitScoreProvider, score_trial_coefficients},
};

const CHANNEL_COUNT: usize = 5;
const SLOT_COUNT: usize = 48;
#[cfg(test)]
const BIN_COUNT: usize = 64;
const ZERO_BITS: u32 = 0x0000_0000;
const PENALTY_F32_BITS: u32 = 0x3f8f_5c29; // R32(28/25)

#[derive(Clone, Debug, PartialEq)]
pub struct CoefficientFitStage {
    iteration: u8,
    candidate_count: u8,
    winner_index: u8,
    support: [bool; CHANNEL_COUNT],
    stored_words: [u32; CHANNEL_COUNT],
    trial_words: [u32; CHANNEL_COUNT],
    score: FitScoreEvaluation,
    penalized_score: f64,
}

impl CoefficientFitStage {
    pub const fn iteration(&self) -> u8 {
        self.iteration
    }

    pub const fn candidate_count(&self) -> u8 {
        self.candidate_count
    }

    pub const fn winner_index(&self) -> u8 {
        self.winner_index
    }

    pub const fn support(&self) -> &[bool; CHANNEL_COUNT] {
        &self.support
    }

    pub const fn stored_words(&self) -> &[u32; CHANNEL_COUNT] {
        &self.stored_words
    }

    pub const fn trial_words(&self) -> &[u32; CHANNEL_COUNT] {
        &self.trial_words
    }

    pub const fn score(&self) -> FitScoreEvaluation {
        self.score
    }

    pub const fn penalized_score(&self) -> f64 {
        self.penalized_score
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CoefficientFitResult {
    support: [bool; CHANNEL_COUNT],
    stored_words: [u32; CHANNEL_COUNT],
    sparse_selected_channel: u8,
    sparse_stored_word: u32,
    winning_stage: u8,
    stages: Vec<CoefficientFitStage>,
}

impl CoefficientFitResult {
    pub const fn support(&self) -> &[bool; CHANNEL_COUNT] {
        &self.support
    }

    pub const fn stored_words(&self) -> &[u32; CHANNEL_COUNT] {
        &self.stored_words
    }

    pub const fn sparse_selected_channel(&self) -> u8 {
        self.sparse_selected_channel
    }

    pub const fn sparse_stored_word(&self) -> u32 {
        self.sparse_stored_word
    }

    pub const fn winning_stage(&self) -> u8 {
        self.winning_stage
    }

    pub fn stages(&self) -> &[CoefficientFitStage] {
        &self.stages
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoefficientSearchError {
    EmptyInitialEligibility,
    InputShape,
    Solver(CoefficientSolverError),
    Quantization(CoefficientQuantizationError),
    Score(FitScoreError),
}

impl std::fmt::Display for CoefficientSearchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for CoefficientSearchError {}

/// Searches one selected object/band cell. Equal scores retain the first
/// enumerated candidate; the stage penalty is applied once per iteration.
pub fn search_selected_coefficient_cell(
    provider: &FitScoreProvider,
    equation: &CoefficientNormalEquation,
    target_words: &[u32],
    carrier_words: &[u32],
    weight_words: &[u32; SLOT_COUNT],
    initial_eligibility: [bool; CHANNEL_COUNT],
) -> Result<CoefficientFitResult, CoefficientSearchError> {
    if !initial_eligibility.iter().any(|enabled| *enabled) {
        return Err(CoefficientSearchError::EmptyInitialEligibility);
    }
    let high_band = equation.begin_bin() >= 18;
    let wire_mode = if high_band { 3 } else { 1 };
    let precision = CoefficientPrecision::from_wire_mode(wire_mode)
        .map_err(CoefficientSearchError::Quantization)?;
    let mut candidate_supports = single_channel_supports(initial_eligibility);
    let mut stages = Vec::with_capacity(if high_band { 2 } else { 5 });
    let mut overall_stage = 0_u8;
    let mut overall_penalized = f64::INFINITY;
    let mut overall_support = [false; CHANNEL_COUNT];
    let mut overall_stored = [ZERO_BITS; CHANNEL_COUNT];
    let mut sparse_selected_channel = 0_u8;
    let mut sparse_stored_word = ZERO_BITS;

    for iteration in 0..5_u8 {
        if candidate_supports.is_empty() {
            break;
        }
        let mut winner: Option<(usize, Candidate)> = None;
        for (candidate_index, support) in candidate_supports.iter().copied().enumerate() {
            let candidate = evaluate_candidate(
                provider,
                equation,
                target_words,
                carrier_words,
                weight_words,
                support,
                precision,
            )?;
            if winner
                .as_ref()
                .is_none_or(|(_, current)| candidate.score.score() < current.score.score())
            {
                winner = Some((candidate_index, candidate));
            }
        }
        let (winner_index, winner) = winner.ok_or(CoefficientSearchError::InputShape)?;
        if iteration == 0 {
            let selected = winner
                .support
                .iter()
                .position(|enabled| *enabled)
                .ok_or(CoefficientSearchError::InputShape)?;
            sparse_selected_channel =
                u8::try_from(selected).map_err(|_| CoefficientSearchError::InputShape)?;
            sparse_stored_word = winner.stored_words[selected];
        }
        let mut penalized_score = winner.score.score();
        let penalty = f64::from(f32::from_bits(PENALTY_F32_BITS));
        for _ in 0..iteration {
            penalized_score *= penalty;
        }
        if penalized_score < overall_penalized {
            overall_penalized = penalized_score;
            overall_stage = iteration;
            overall_support = winner.support;
            overall_stored = winner.stored_words;
        }
        stages.push(CoefficientFitStage {
            iteration,
            candidate_count: u8::try_from(candidate_supports.len())
                .map_err(|_| CoefficientSearchError::InputShape)?,
            winner_index: u8::try_from(winner_index)
                .map_err(|_| CoefficientSearchError::InputShape)?,
            support: winner.support,
            stored_words: winner.stored_words,
            trial_words: winner.trial_words,
            score: winner.score,
            penalized_score,
        });

        if high_band {
            if iteration == 0 {
                candidate_supports = single_channel_supports([true; CHANNEL_COUNT]);
                continue;
            }
            break;
        }
        if winner.support.iter().all(|enabled| *enabled) {
            break;
        }
        candidate_supports = add_one_channel_supports(winner.support);
    }

    Ok(CoefficientFitResult {
        support: overall_support,
        stored_words: overall_stored,
        sparse_selected_channel,
        sparse_stored_word,
        winning_stage: overall_stage,
        stages,
    })
}

#[derive(Clone, Debug)]
struct Candidate {
    support: [bool; CHANNEL_COUNT],
    stored_words: [u32; CHANNEL_COUNT],
    trial_words: [u32; CHANNEL_COUNT],
    score: FitScoreEvaluation,
}

fn evaluate_candidate(
    provider: &FitScoreProvider,
    equation: &CoefficientNormalEquation,
    target_words: &[u32],
    carrier_words: &[u32],
    weight_words: &[u32; SLOT_COUNT],
    support: [bool; CHANNEL_COUNT],
    precision: CoefficientPrecision,
) -> Result<Candidate, CoefficientSearchError> {
    let solved =
        solve_selected_support(equation, support).map_err(CoefficientSearchError::Solver)?;
    let normalized =
        normalize_selected_solution(equation, &solved, target_words, carrier_words, weight_words)
            .map_err(CoefficientSearchError::Solver)?;
    let mut trial_words = [ZERO_BITS; CHANNEL_COUNT];
    for (trial, stored) in trial_words
        .iter_mut()
        .zip(normalized.stored_words().iter().copied())
    {
        let quantized = quantize_selected_coefficient(f32::from_bits(stored), precision)
            .map_err(CoefficientSearchError::Quantization)?;
        *trial = restore_trial_coefficient(quantized, precision)
            .map_err(CoefficientSearchError::Quantization)?
            .to_bits();
    }
    let score = score_trial_coefficients(
        provider,
        equation,
        target_words,
        carrier_words,
        weight_words,
        &trial_words,
    )
    .map_err(CoefficientSearchError::Score)?;
    Ok(Candidate {
        support,
        stored_words: *normalized.stored_words(),
        trial_words,
        score,
    })
}

fn single_channel_supports(eligibility: [bool; CHANNEL_COUNT]) -> Vec<[bool; CHANNEL_COUNT]> {
    eligibility
        .into_iter()
        .enumerate()
        .filter(|(_, enabled)| *enabled)
        .map(|(channel, _)| {
            let mut support = [false; CHANNEL_COUNT];
            support[channel] = true;
            support
        })
        .collect()
}

fn add_one_channel_supports(current: [bool; CHANNEL_COUNT]) -> Vec<[bool; CHANNEL_COUNT]> {
    current
        .iter()
        .enumerate()
        .filter(|(_, enabled)| !**enabled)
        .map(|(channel, _)| {
            let mut support = current;
            support[channel] = true;
            support
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joc::coefficient_normal_equations::build_selected_normal_equations;
    use serde_json::Value;

    const INPUT: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_fit_cell_boundaries/INPUT-ONLY.json");
    const EXPECTED: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_fit_cell_boundaries/EXPECTED.json");
    const FUNCTIONAL: &[u8] = include_bytes!(
        "../../tests/fixtures/synthetic_qmf_fit_checkpoints/approved-joc-functional.json"
    );
    const MATH: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_qmf_fit_checkpoints/approved-joc-math.json");

    #[test]
    fn six_boundary_cases_match_all_stage_winners_penalties_and_final_outputs() {
        let input: Value = serde_json::from_slice(INPUT).unwrap();
        let expected: Value = serde_json::from_slice(EXPECTED).unwrap();
        let provider = FitScoreProvider::from_approved_bytes(FUNCTIONAL, MATH).unwrap();
        let mut score_calls = 0_u64;
        for (case, expected) in input["cases"]
            .as_array()
            .unwrap()
            .iter()
            .zip(expected["results"].as_array().unwrap())
        {
            let eligibility = bool_array(&case["eligibility"]);
            let (equation, targets, carriers) = synthetic_cell(case);
            let result = search_selected_coefficient_cell(
                &provider,
                &equation,
                &targets,
                &carriers,
                &[0x3f80_0000; SLOT_COUNT],
                eligibility,
            );
            if !eligibility.iter().any(|enabled| *enabled) {
                assert_eq!(result, Err(CoefficientSearchError::EmptyInitialEligibility));
                continue;
            }
            let actual = result.unwrap();
            let fit = &expected["fit"];
            assert_eq!(actual.support(), &bool_array(&fit["support"]));
            assert_eq!(
                u64::from(actual.sparse_selected_channel()),
                fit["sparse_backup"]["selected"].as_u64().unwrap()
            );
            assert_eq!(
                actual.sparse_stored_word(),
                f64_to_f32_word(&fit["sparse_backup"]["coefficient"])
            );
            assert_eq!(
                u64::from(actual.winning_stage()),
                fit["winning_stage"].as_u64().unwrap()
            );
            for channel in 0..CHANNEL_COUNT {
                assert_eq!(
                    actual.stored_words()[channel],
                    f64_to_f32_word(&expected["stored_full"][channel])
                );
            }
            let expected_stages = fit["stages"].as_array().unwrap();
            assert_eq!(actual.stages().len(), expected_stages.len());
            for (stage, expected_stage) in actual.stages().iter().zip(expected_stages) {
                score_calls += u64::from(stage.candidate_count());
                assert_eq!(u64::from(stage.iteration()), expected_stage["iteration"]);
                assert_eq!(
                    u64::from(stage.candidate_count()),
                    expected_stage["candidate_count"]
                );
                assert_eq!(u64::from(stage.winner_index()), expected_stage["winner"]);
                assert_eq!(
                    stage.support(),
                    &bool_array(&expected_stage["chosen"]["support"])
                );
                assert_eq!(
                    stage.score().score().to_bits(),
                    f64_word(&expected_stage["chosen"]["score"])
                );
                assert_eq!(
                    stage.penalized_score().to_bits(),
                    f64_word(&expected_stage["penalized_score"])
                );
            }
        }
        let diagnostic_initial_calls = expected["results"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|result| result.get("initial_single_candidates"))
            .map(|candidates| candidates.as_array().unwrap().len() as u64)
            .sum::<u64>();
        assert_eq!(
            score_calls + diagnostic_initial_calls,
            expected["score_calls"].as_u64().unwrap()
        );
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
        let mut carriers = vec![ZERO_BITS; CHANNEL_COUNT * SLOT_COUNT * BIN_COUNT * 2];
        let mut targets = vec![ZERO_BITS; 15 * SLOT_COUNT * BIN_COUNT * 2];
        for slot in 0..SLOT_COUNT {
            let mut values = [0.0_f32; CHANNEL_COUNT];
            for channel in 0..CHANNEL_COUNT {
                let value = if channel == 0 || (channel == 1 && duplicate) {
                    amplitude
                } else {
                    let bit = (slot >> (channel - 1)) & 1;
                    amplitude * if bit == 0 { 1.0 } else { -1.0 }
                };
                values[channel] = value;
                carriers[((channel * SLOT_COUNT + slot) * BIN_COUNT + selected_bin) * 2] =
                    value.to_bits();
            }
            let mut target = 0.0_f32;
            for channel in 0..CHANNEL_COUNT {
                target += values[channel] * mix[channel];
            }
            targets[(slot * BIN_COUNT + selected_bin) * 2] = target.to_bits();
        }
        let equation =
            build_selected_normal_equations(&targets, &carriers, &[0x3f80_0000; SLOT_COUNT])
                .unwrap()
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

    fn f64_to_f32_word(value: &Value) -> u32 {
        if value.is_number() {
            return ZERO_BITS;
        }
        (f64::from_bits(f64_word(value)) as f32).to_bits()
    }

    fn f64_word(value: &Value) -> u64 {
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
