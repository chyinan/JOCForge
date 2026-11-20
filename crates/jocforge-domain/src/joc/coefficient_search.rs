// pattern: Functional Core
//! Selected multi-stage coefficient candidate enumeration and strict score search.

use crate::joc::{
    coefficient_normal_equations::CoefficientNormalEquation,
    coefficient_quantization::{
        CoefficientPrecision, CoefficientQuantizationError, quantize_selected_coefficient,
        restore_trial_coefficient,
    },
    coefficient_solver::{
        CoefficientSolution, CoefficientSolverError, normalize_solution_energy,
        normalize_solution_for_carriers, solve_support_for_carriers,
    },
    fit_score::{
        FitScoreError, FitScoreEvaluation, FitScoreProvider, evaluate_fit_energies,
        score_trial_coefficients_for_carriers,
    },
};

const CHANNEL_COUNT: usize = 5;
const SLOT_COUNT: usize = 48;
const BIN_COUNT: usize = 64;
const ZERO_BITS: u32 = 0x0000_0000;
const PENALTY_F32_BITS: u32 = 0x3f8f_5c29; // R32(28/25)

#[derive(Clone, Debug, PartialEq)]
pub struct CoefficientFitStage<const CARRIER_COUNT: usize = CHANNEL_COUNT> {
    iteration: u8,
    candidate_count: u8,
    winner_index: u8,
    support: [bool; CARRIER_COUNT],
    stored_words: [u32; CARRIER_COUNT],
    trial_words: [u32; CARRIER_COUNT],
    score: FitScoreEvaluation,
    penalized_score: f64,
}

impl<const CARRIER_COUNT: usize> CoefficientFitStage<CARRIER_COUNT> {
    pub const fn iteration(&self) -> u8 {
        self.iteration
    }

    pub const fn candidate_count(&self) -> u8 {
        self.candidate_count
    }

    pub const fn winner_index(&self) -> u8 {
        self.winner_index
    }

    pub const fn support(&self) -> &[bool; CARRIER_COUNT] {
        &self.support
    }

    pub const fn stored_words(&self) -> &[u32; CARRIER_COUNT] {
        &self.stored_words
    }

    pub const fn trial_words(&self) -> &[u32; CARRIER_COUNT] {
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
pub struct CoefficientFitResult<const CARRIER_COUNT: usize = CHANNEL_COUNT> {
    support: [bool; CARRIER_COUNT],
    stored_words: [u32; CARRIER_COUNT],
    sparse_selected_channel: u8,
    sparse_stored_word: u32,
    winning_stage: u8,
    stages: Vec<CoefficientFitStage<CARRIER_COUNT>>,
}

impl<const CARRIER_COUNT: usize> CoefficientFitResult<CARRIER_COUNT> {
    pub const fn support(&self) -> &[bool; CARRIER_COUNT] {
        &self.support
    }

    pub const fn stored_words(&self) -> &[u32; CARRIER_COUNT] {
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

    pub fn stages(&self) -> &[CoefficientFitStage<CARRIER_COUNT>] {
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
    search_coefficient_cell_for_carriers::<CHANNEL_COUNT>(
        provider,
        equation,
        target_words,
        carrier_words,
        weight_words,
        initial_eligibility,
    )
}

/// Runs the existing support-growth search for a fixed carrier count.
/// For C=5, candidate and floating-point accumulation order remain unchanged. C=7 extends the
/// same experimental policy in target-role order and remains a non-production strategy.
pub fn search_coefficient_cell_for_carriers<const CARRIER_COUNT: usize>(
    provider: &FitScoreProvider,
    equation: &CoefficientNormalEquation<CARRIER_COUNT>,
    target_words: &[u32],
    carrier_words: &[u32],
    weight_words: &[u32; SLOT_COUNT],
    initial_eligibility: [bool; CARRIER_COUNT],
) -> Result<CoefficientFitResult<CARRIER_COUNT>, CoefficientSearchError> {
    let iteration_limit =
        u8::try_from(CARRIER_COUNT).map_err(|_| CoefficientSearchError::InputShape)?;
    if CARRIER_COUNT == 0 {
        return Err(CoefficientSearchError::EmptyInitialEligibility);
    }
    if !initial_eligibility.iter().any(|enabled| *enabled) {
        return Err(CoefficientSearchError::EmptyInitialEligibility);
    }
    let high_band = equation.begin_bin() >= 18;
    let wire_mode = if high_band { 3 } else { 1 };
    let precision = CoefficientPrecision::from_wire_mode(wire_mode)
        .map_err(CoefficientSearchError::Quantization)?;
    let mut candidate_supports = single_channel_supports(initial_eligibility);
    let mut stages = Vec::with_capacity(if high_band {
        2
    } else {
        usize::from(iteration_limit)
    });
    let mut overall_stage = 0_u8;
    let mut overall_penalized = f64::INFINITY;
    let mut overall_support = [false; CARRIER_COUNT];
    let mut overall_stored = [ZERO_BITS; CARRIER_COUNT];
    let mut sparse_selected_channel = 0_u8;
    let mut sparse_stored_word = ZERO_BITS;

    // Both high-band stages visit single-channel supports for the same equation.
    let mut singletons = [None; CARRIER_COUNT];
    let mut cell = FitCell {
        band: WeightedFitBand::from_windows(equation, target_words, carrier_words, weight_words),
        provider,
        equation,
        target_words,
        carrier_words,
        weight_words,
        precision,
    };
    let mut scores = Vec::new();
    for iteration in 0..iteration_limit {
        if candidate_supports.is_empty() {
            break;
        }
        let mut winner: Option<(usize, Candidate<CARRIER_COUNT>)> = None;
        let mut last_scaled_range_error = None;
        for (candidate_index, support) in candidate_supports.iter().copied().enumerate() {
            let mut evaluate = || cell.evaluate(support, &mut scores);
            let result = if high_band {
                let channel = support
                    .iter()
                    .position(|active| *active)
                    .ok_or(CoefficientSearchError::EmptyInitialEligibility)?;
                *singletons[channel].get_or_insert_with(evaluate)
            } else {
                evaluate()
            };
            let candidate = match result {
                Ok(candidate) => candidate,
                Err(
                    error @ CoefficientSearchError::Quantization(
                        CoefficientQuantizationError::ScaledI32Range,
                    ),
                ) if CARRIER_COUNT == 7 => {
                    last_scaled_range_error = Some(error);
                    continue;
                }
                Err(error) => return Err(error),
            };
            if winner
                .as_ref()
                .is_none_or(|(_, current)| candidate.score.score() < current.score.score())
            {
                winner = Some((candidate_index, candidate));
            }
        }
        let Some((winner_index, winner)) = winner else {
            return Err(last_scaled_range_error.unwrap_or(CoefficientSearchError::InputShape));
        };
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
                candidate_supports = single_channel_supports([true; CARRIER_COUNT]);
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

#[derive(Clone, Copy, Debug)]
struct Candidate<const CARRIER_COUNT: usize> {
    support: [bool; CARRIER_COUNT],
    stored_words: [u32; CARRIER_COUNT],
    trial_words: [u32; CARRIER_COUNT],
    score: FitScoreEvaluation,
}

struct FitCell<'a, const CARRIER_COUNT: usize> {
    provider: &'a FitScoreProvider,
    equation: &'a CoefficientNormalEquation<CARRIER_COUNT>,
    target_words: &'a [u32],
    carrier_words: &'a [u32],
    weight_words: &'a [u32; SLOT_COUNT],
    precision: CoefficientPrecision,
    band: Option<WeightedFitBand>,
}

impl<const CARRIER_COUNT: usize> FitCell<'_, CARRIER_COUNT> {
    fn evaluate(
        &mut self,
        support: [bool; CARRIER_COUNT],
        scores: &mut Vec<([u32; CARRIER_COUNT], FitScoreEvaluation)>,
    ) -> Result<Candidate<CARRIER_COUNT>, CoefficientSearchError> {
        let solved = solve_support_for_carriers(self.equation, support)
            .map_err(CoefficientSearchError::Solver)?;
        let normalized = if let Some(band) = &mut self.band {
            let model_energy = band.model_energy(&solved);
            normalize_solution_energy(&solved, band.target_energy, model_energy)
        } else {
            normalize_solution_for_carriers(
                self.equation,
                &solved,
                self.target_words,
                self.carrier_words,
                self.weight_words,
            )
        }
        .map_err(CoefficientSearchError::Solver)?;
        let mut trial_words = [ZERO_BITS; CARRIER_COUNT];
        for (trial, stored) in trial_words
            .iter_mut()
            .zip(normalized.stored_words().iter().copied())
        {
            let quantized = quantize_selected_coefficient(f32::from_bits(stored), self.precision)
                .map_err(CoefficientSearchError::Quantization)?;
            *trial = restore_trial_coefficient(quantized, self.precision)
                .map_err(CoefficientSearchError::Quantization)?
                .to_bits();
        }
        // Distinct supports can quantize to identical coefficients. Only the score
        // is shared; the candidate's support and stored words still participate in selection.
        let score = if let Some((_, score)) = scores.iter().find(|(words, _)| *words == trial_words)
        {
            *score
        } else {
            let score = if let Some(band) = &mut self.band {
                band.score(self.provider, &trial_words)
            } else {
                score_trial_coefficients_for_carriers(
                    self.provider,
                    self.equation,
                    self.target_words,
                    self.carrier_words,
                    self.weight_words,
                    &trial_words,
                )
            }
            .map_err(CoefficientSearchError::Score)?;
            scores.push((trial_words, score));
            score
        };
        Ok(Candidate {
            support,
            stored_words: *normalized.stored_words(),
            trial_words,
            score,
        })
    }
}

/// Slot-major complex samples packed once per cell. All support candidates reuse
/// the weighted inputs, both target energies, and the scratch for the predicted row.
struct WeightedFitBand {
    target: Vec<f32>,
    carriers: Vec<f32>,
    slot_width: usize,
    target_energy: f32,
    score_target_energy: f64,
    model: Vec<f32>,
    residual: Vec<f64>,
}

impl WeightedFitBand {
    fn from_windows<const C: usize>(
        equation: &CoefficientNormalEquation<C>,
        target_words: &[u32],
        carrier_words: &[u32],
        weight_words: &[u32; SLOT_COUNT],
    ) -> Option<Self> {
        let row_stride = SLOT_COUNT * BIN_COUNT * 2;
        if target_words.len() != 15 * row_stride || carrier_words.len() != C * row_stride {
            return None;
        }
        let begin = usize::from(equation.begin_bin()) * 2;
        let end = usize::from(equation.end_bin()) * 2;
        let slot_width = end - begin;
        let samples = SLOT_COUNT * slot_width;
        let pack = |row: &[u32], result: &mut Vec<f32>| {
            for (slot, &weight) in row.chunks_exact(BIN_COUNT * 2).zip(weight_words) {
                let weight = f32::from_bits(weight);
                result.extend(slot[begin..end].iter().map(|&w| f32::from_bits(w) * weight));
            }
        };
        let object = usize::from(equation.object());
        let mut target = Vec::with_capacity(samples);
        pack(
            &target_words[object * row_stride..(object + 1) * row_stride],
            &mut target,
        );
        let mut carriers = Vec::with_capacity(samples * C);
        for row in carrier_words.chunks_exact(row_stride) {
            pack(row, &mut carriers);
        }
        // Standalone callers may supply different windows from those used to construct
        // the equation. Keep the original per-support validation for invalid inputs.
        if target.iter().chain(&carriers).any(|x| !x.is_finite()) {
            return None;
        }
        let target_energy = normalization_energy(&target, slot_width);
        let score_target_energy = target.iter().fold(0., |sum, &x| {
            let x = f64::from(x);
            sum + x * x
        });
        Some(Self {
            target,
            carriers,
            slot_width,
            target_energy,
            score_target_energy,
            model: vec![0.; samples],
            residual: vec![0.; samples],
        })
    }

    fn model_energy<const C: usize>(&mut self, solved: &CoefficientSolution<C>) -> f32 {
        let mut empty = true;
        for (&bits, carrier) in solved
            .coefficient_words()
            .iter()
            .zip(self.carriers.chunks_exact(self.target.len()))
        {
            let a = f32::from_bits(bits);
            if a == 0. {
                continue;
            }
            if empty {
                for (y, &x) in self.model.iter_mut().zip(carrier) {
                    *y = 0. + a * x;
                }
                empty = false;
            } else {
                for (y, &x) in self.model.iter_mut().zip(carrier) {
                    *y += a * x;
                }
            }
        }
        if empty {
            0.
        } else {
            normalization_energy(&self.model, self.slot_width)
        }
    }

    fn score<const C: usize>(
        &mut self,
        provider: &FitScoreProvider,
        trial: &[u32; C],
    ) -> Result<FitScoreEvaluation, FitScoreError> {
        let mut empty = true;
        for (&bits, carrier) in trial
            .iter()
            .zip(self.carriers.chunks_exact(self.target.len()))
        {
            let a = f64::from(f32::from_bits(bits));
            if a == 0. {
                continue;
            }
            if empty {
                for ((y, &t), &x) in self.residual.iter_mut().zip(&self.target).zip(carrier) {
                    *y = -f64::from(t) + f64::from(x) * a;
                }
                empty = false;
            } else {
                for (y, &x) in self.residual.iter_mut().zip(carrier) {
                    *y += f64::from(x) * a;
                }
            }
        }
        if empty {
            return evaluate_fit_energies(
                provider,
                self.score_target_energy,
                0.,
                self.score_target_energy,
            );
        }
        let mut error = 0.;
        let mut model_energy = 0.;
        for (&r, &t) in self.residual.iter().zip(&self.target) {
            let model = r - (-f64::from(t));
            error += r * r;
            model_energy += model * model;
        }
        evaluate_fit_energies(provider, error, model_energy, self.score_target_energy)
    }
}

fn normalization_energy(samples: &[f32], slot_width: usize) -> f32 {
    samples.chunks_exact(slot_width).fold(0., |total, slot| {
        total + slot.iter().fold(0., |sum, &x| sum + x * x)
    })
}

fn single_channel_supports<const CARRIER_COUNT: usize>(
    eligibility: [bool; CARRIER_COUNT],
) -> Vec<[bool; CARRIER_COUNT]> {
    eligibility
        .into_iter()
        .enumerate()
        .filter(|(_, enabled)| *enabled)
        .map(|(channel, _)| {
            let mut support = [false; CARRIER_COUNT];
            support[channel] = true;
            support
        })
        .collect()
}

fn add_one_channel_supports<const CARRIER_COUNT: usize>(
    current: [bool; CARRIER_COUNT],
) -> Vec<[bool; CARRIER_COUNT]> {
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
    fn seven_carrier_support_growth_uses_target_role_order() {
        let candidates = single_channel_supports([true; 7]);
        assert_eq!(candidates.len(), 7);
        assert_eq!(
            candidates[0],
            [true, false, false, false, false, false, false]
        );
        assert_eq!(
            candidates[6],
            [false, false, false, false, false, false, true]
        );

        let expanded = add_one_channel_supports(candidates[0]);
        assert_eq!(expanded.len(), 6);
        assert_eq!(expanded[0], [true, true, false, false, false, false, false]);
    }

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
