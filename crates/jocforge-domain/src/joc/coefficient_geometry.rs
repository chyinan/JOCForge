// pattern: Functional Core
//! Selected control geometry, delayed cube and five-carrier eligibility proposal.

use crate::{
    numeric::{FiniteF32, r32_add, r32_mul, r32_sub},
    spatial::{
        QmfReceiverFunctionalProvider,
        spatial_audio_state::{JocGeometryRowState, SelectedJocQmfProjection},
    },
};

const SOURCE_ROWS: usize = 16;
const OBJECT_COUNT: usize = 15;
const LANE_COUNT: usize = 6;
const CARRIER_LANES: [usize; 5] = [0, 1, 2, 4, 5];
const BLOCK_COUNT: usize = 48;
const SLOT_COUNT: usize = 24;
const CUBE_WORDS: usize = 128;
const CARRY_SLOTS: usize = 18;
const ZERO_BITS: u32 = 0x0000_0000;
const ONE_HALF_BITS: u32 = 0x3f00_0000;
const ONE_OVER_16_BITS: u32 = 0x3d80_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::struct_field_names)]
pub(crate) struct GeometryAccumulatorRow {
    current_words: [u32; LANE_COUNT],
    target_words: [u32; LANE_COUNT],
    slope_words: [u32; LANE_COUNT],
    remaining_words: [u32; LANE_COUNT],
}

impl GeometryAccumulatorRow {
    #[cfg(test)]
    pub(crate) const fn cold() -> Self {
        Self {
            current_words: [ZERO_BITS; LANE_COUNT],
            target_words: [ZERO_BITS; LANE_COUNT],
            slope_words: [ZERO_BITS; LANE_COUNT],
            remaining_words: [0; LANE_COUNT],
        }
    }

    fn from_projection(row: &JocGeometryRowState) -> Self {
        Self {
            current_words: *row.current_words(),
            target_words: *row.target_words(),
            slope_words: *row.slope_words(),
            remaining_words: *row.remaining_words(),
        }
    }

    fn matches_projection(&self, row: &JocGeometryRowState) -> bool {
        self.current_words == *row.current_words()
            && self.target_words == *row.target_words()
            && self.slope_words == *row.slope_words()
            && self.remaining_words == *row.remaining_words()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoefficientGeometryState {
    carry18_words: [[u32; CUBE_WORDS]; CARRY_SLOTS],
    previous24_words: [[u32; CUBE_WORDS]; SLOT_COUNT],
    generation: u64,
}

impl CoefficientGeometryState {
    pub const fn cold() -> Self {
        Self {
            carry18_words: [[ZERO_BITS; CUBE_WORDS]; CARRY_SLOTS],
            previous24_words: [[ZERO_BITS; CUBE_WORDS]; SLOT_COUNT],
            generation: 0,
        }
    }

    pub const fn carry18_words(&self) -> &[[u32; CUBE_WORDS]; CARRY_SLOTS] {
        &self.carry18_words
    }

    pub const fn previous24_words(&self) -> &[[u32; CUBE_WORDS]; SLOT_COUNT] {
        &self.previous24_words
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn canonical_payload_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(21_512);
        for word in self.carry18_words.iter().flatten() {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        for word in self.previous24_words.iter().flatten() {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes.extend_from_slice(&self.generation.to_le_bytes());
        bytes
    }

    #[cfg(test)]
    pub(crate) const fn with_generation_for_test(generation: u64) -> Self {
        let mut state = Self::cold();
        state.generation = generation;
        state
    }
}

impl Default for CoefficientGeometryState {
    fn default() -> Self {
        Self::cold()
    }
}

#[derive(Debug)]
pub struct CoefficientGeometryProposal {
    after: CoefficientGeometryState,
    row_states_after: [GeometryAccumulatorRow; SOURCE_ROWS],
    #[cfg(test)]
    cube_words: [[u32; CUBE_WORDS]; SLOT_COUNT],
    #[cfg(test)]
    lane_sum_words: [[u32; LANE_COUNT]; OBJECT_COUNT],
    eligibility: [[bool; 5]; OBJECT_COUNT],
}

impl CoefficientGeometryProposal {
    pub const fn after(&self) -> &CoefficientGeometryState {
        &self.after
    }

    pub const fn eligibility(&self) -> &[[bool; 5]; OBJECT_COUNT] {
        &self.eligibility
    }

    #[cfg(test)]
    pub(crate) const fn row_states_after(&self) -> &[GeometryAccumulatorRow; SOURCE_ROWS] {
        &self.row_states_after
    }

    pub fn into_after(self) -> CoefficientGeometryState {
        self.after
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoefficientGeometryError {
    InputNonFinite,
    NumericNonFinite,
    ReceiverLineage,
}

impl std::fmt::Display for CoefficientGeometryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for CoefficientGeometryError {}

pub(crate) fn stage_geometry_from_qmf(
    before: &CoefficientGeometryState,
    projection: &SelectedJocQmfProjection,
    weight_words: &[u32; 48],
    provider: &QmfReceiverFunctionalProvider,
) -> Result<CoefficientGeometryProposal, CoefficientGeometryError> {
    let rows_before = projection
        .geometry_rows_before()
        .each_ref()
        .map(GeometryAccumulatorRow::from_projection);
    let target_words = projection
        .geometry_rows_after()
        .each_ref()
        .map(|row| *row.target_words());
    let proposal =
        stage_selected_geometry(before, &rows_before, &target_words, weight_words, provider)?;
    if proposal.after.generation != projection.dispatch().value() {
        return Err(CoefficientGeometryError::ReceiverLineage);
    }
    if !proposal
        .row_states_after
        .iter()
        .zip(projection.geometry_rows_after())
        .all(|(actual, expected)| actual.matches_projection(expected))
    {
        return Err(CoefficientGeometryError::ReceiverLineage);
    }
    Ok(proposal)
}

#[allow(clippy::needless_range_loop)] // Block/row/lane order is part of the contract.
pub(crate) fn stage_selected_geometry(
    before: &CoefficientGeometryState,
    rows_before: &[GeometryAccumulatorRow; SOURCE_ROWS],
    target_words: &[[u32; LANE_COUNT]; SOURCE_ROWS],
    weight_words: &[u32; 48],
    provider: &QmfReceiverFunctionalProvider,
) -> Result<CoefficientGeometryProposal, CoefficientGeometryError> {
    let mut rows = *rows_before;
    let scalars = provider.scalars();
    let magnitude_threshold = finite_word(scalars.accumulator_magnitude_threshold())?;
    let delta_threshold = finite_word(scalars.accumulator_delta_threshold())?;
    let smoothing = finite_word(scalars.accumulator_smoothing())?;
    let half = finite_word(ONE_HALF_BITS)?;
    let mut cube_words = [[ZERO_BITS; CUBE_WORDS]; SLOT_COUNT];
    for (row, target) in rows.iter_mut().zip(target_words) {
        for word in target {
            finite_word(*word)?;
        }
        row.target_words = *target;
    }

    for block in 0..BLOCK_COUNT {
        let active = block == 0;
        let duration = u32::try_from(BLOCK_COUNT - block)
            .map_err(|_| CoefficientGeometryError::NumericNonFinite)?;
        for row in &mut rows {
            for lane in 0..LANE_COUNT {
                let current = finite_word(row.current_words[lane])?;
                let target = finite_word(row.target_words[lane])?;
                let mut slope = finite_word(row.slope_words[lane])?;
                let mut remaining = row.remaining_words[lane];
                let mut next_current = current;
                if target.value().abs().max(current.value().abs()) >= magnitude_threshold.value() {
                    let delta = subtract(target, current)?;
                    if delta.value().abs() >= delta_threshold.value() {
                        if active {
                            remaining = duration;
                            slope = multiply(delta, smoothing)?;
                        }
                    } else {
                        remaining = 0;
                        next_current = target;
                    }
                    if remaining != 0 {
                        next_current = add(slope, next_current)?;
                        remaining -= 1;
                    } else {
                        next_current = target;
                    }
                    row.current_words[lane] = next_current.to_bits();
                    row.slope_words[lane] = slope.to_bits();
                    row.remaining_words[lane] = remaining;
                }
            }
        }

        let slot = block / 2;
        for object in 0..OBJECT_COUNT {
            for lane in 0..LANE_COUNT {
                let current = finite_word(rows[object + 1].current_words[lane])?;
                let half_current = multiply(current, half)?;
                let index = object * 8 + lane;
                cube_words[slot][index] = if block % 2 == 0 {
                    half_current.to_bits()
                } else {
                    add(finite_word(cube_words[slot][index])?, half_current)?.to_bits()
                };
            }
        }
    }

    let mut current24 = [[ZERO_BITS; CUBE_WORDS]; SLOT_COUNT];
    current24[..CARRY_SLOTS].copy_from_slice(&before.carry18_words);
    current24[CARRY_SLOTS..].copy_from_slice(&cube_words[..SLOT_COUNT - CARRY_SLOTS]);
    let carry18_words: [[u32; CUBE_WORDS]; CARRY_SLOTS] = cube_words[SLOT_COUNT - CARRY_SLOTS..]
        .try_into()
        .map_err(|_| CoefficientGeometryError::NumericNonFinite)?;

    let weights = weight_words
        .iter()
        .map(|word| finite_word(*word))
        .collect::<Result<Vec<_>, _>>()?;
    let mut lane_sum_words = [[ZERO_BITS; LANE_COUNT]; OBJECT_COUNT];
    let mut eligibility = [[false; 5]; OBJECT_COUNT];
    for object in 0..OBJECT_COUNT {
        for lane in 0..LANE_COUNT {
            let index = object * 8 + lane;
            let mut sum = finite_word(ZERO_BITS)?;
            for slot in 0..SLOT_COUNT {
                let previous = finite_word(before.previous24_words[slot][index])?;
                sum = add(multiply(previous, weights[slot])?, sum)?;
            }
            for slot in 0..SLOT_COUNT {
                let current = finite_word(current24[slot][index])?;
                sum = add(sum, multiply(current, weights[SLOT_COUNT + slot])?)?;
            }
            lane_sum_words[object][lane] = sum.to_bits();
        }
        let maximum = lane_sum_words[object]
            .iter()
            .map(|word| f32::from_bits(*word))
            .fold(0.0_f32, f32::max);
        let threshold = multiply(finite_value(maximum)?, finite_word(ONE_OVER_16_BITS)?)?;
        eligibility[object] = CARRIER_LANES
            .map(|lane| f32::from_bits(lane_sum_words[object][lane]) >= threshold.value());
    }

    Ok(CoefficientGeometryProposal {
        after: CoefficientGeometryState {
            carry18_words,
            previous24_words: current24,
            generation: before
                .generation
                .checked_add(1)
                .ok_or(CoefficientGeometryError::NumericNonFinite)?,
        },
        row_states_after: rows,
        #[cfg(test)]
        cube_words,
        #[cfg(test)]
        lane_sum_words,
        eligibility,
    })
}

fn finite_word(word: u32) -> Result<FiniteF32, CoefficientGeometryError> {
    finite_value(f32::from_bits(word))
}

fn finite_value(value: f32) -> Result<FiniteF32, CoefficientGeometryError> {
    FiniteF32::new(value).map_err(|_| CoefficientGeometryError::InputNonFinite)
}

fn multiply(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, CoefficientGeometryError> {
    r32_mul(left, right).map_err(|_| CoefficientGeometryError::NumericNonFinite)
}

fn add(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, CoefficientGeometryError> {
    r32_add(left, right).map_err(|_| CoefficientGeometryError::NumericNonFinite)
}

fn subtract(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, CoefficientGeometryError> {
    r32_sub(left, right).map_err(|_| CoefficientGeometryError::NumericNonFinite)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn cold_geometry_payload_matches_the_approved_family_identity() {
        let bytes = CoefficientGeometryState::cold().canonical_payload_bytes();
        assert_eq!(bytes.len(), 21_512);
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            "e575bf3985b08db4829cfe7ae78928a448b3eaab0e06996be7cdd9a814fc6ac3"
        );
    }
    use crate::{
        frontend::{PackageManifestEvidence, ProviderFileEvidence},
        joc::temporal_weights::TemporalWeightState,
        spatial::{
            validate_f04_parameter_package_manifest, validate_qmf_receiver_functional_provider,
        },
    };
    use serde_json::Value;
    use std::sync::Arc;

    const INPUT: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_qmf_fit_checkpoints/INPUT-ONLY.json");
    const EXPECTED: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_qmf_fit_checkpoints/EXPECTED.json");
    const MANIFEST: &[u8] = include_bytes!("../../tests/fixtures/f01e_provider/MANIFEST.json");
    const FUNCTIONAL: &[u8] =
        include_bytes!("../../tests/fixtures/f04_parameters/qmf-receiver-functional-v1.json");

    #[test]
    fn three_generations_match_cube_rows_delays_sums_and_eligibility() {
        let input: Value = serde_json::from_slice(INPUT).unwrap();
        let expected: Value = serde_json::from_slice(EXPECTED).unwrap();
        let manifest = validate_f04_parameter_package_manifest(&PackageManifestEvidence::new(
            Arc::from(MANIFEST),
            Some("geometry test manifest".to_owned()),
        ))
        .unwrap();
        let provider = validate_qmf_receiver_functional_provider(
            &ProviderFileEvidence::new(
                Arc::from(FUNCTIONAL),
                Some("geometry functional provider".to_owned()),
            ),
            &manifest,
        )
        .unwrap();
        let targets = synthetic_targets(&input);
        let mut rows = [GeometryAccumulatorRow::cold(); SOURCE_ROWS];
        let mut state = CoefficientGeometryState::cold();
        let mut temporal = TemporalWeightState::cold();
        let mut previous_targets = vec![ZERO_BITS; OBJECT_COUNT * 24 * 64 * 2];

        for generation in 0..3 {
            let current_targets = synthetic_target_current24(&input, generation);
            let windows = concatenate_target_windows(&previous_targets, &current_targets);
            let temporal_proposal = temporal.stage_target_windows48(&windows).unwrap();
            let proposal = stage_selected_geometry(
                &state,
                &rows,
                &targets,
                temporal_proposal.after().weight_words(),
                &provider,
            )
            .unwrap();
            let checkpoint = &expected["checkpoints"][generation]["geometry"];
            assert_geometry_state(&state, &checkpoint["before"]);
            assert_geometry_state(proposal.after(), &checkpoint["after"]);
            for slot in 0..SLOT_COUNT {
                assert_eq!(
                    proposal.cube_words[slot].as_slice(),
                    decode_u32_words(checkpoint["cube"][slot]["hex"].as_str().unwrap()).as_slice()
                );
            }
            for row in 0..SOURCE_ROWS {
                assert_row(
                    &proposal.row_states_after[row],
                    &checkpoint["row_states_after"][row],
                );
            }
            for object in 0..OBJECT_COUNT {
                for lane in 0..LANE_COUNT {
                    assert_eq!(
                        proposal.lane_sum_words[object][lane],
                        f64_to_f32_word(&checkpoint["lane_sums"][object][lane])
                    );
                }
                assert_eq!(
                    proposal.eligibility[object],
                    std::array::from_fn(|carrier| {
                        checkpoint["eligibility"][object][carrier].as_u64().unwrap() != 0
                    })
                );
            }
            rows = proposal.row_states_after;
            state = proposal.into_after();
            temporal = temporal_proposal.into_after();
            previous_targets = current_targets;
        }
    }

    fn synthetic_targets(input: &Value) -> [[u32; LANE_COUNT]; SOURCE_ROWS] {
        let mut targets = [[ZERO_BITS; LANE_COUNT]; SOURCE_ROWS];
        targets[0][3] = 0x3f80_0000;
        for object in 0..OBJECT_COUNT {
            for carrier in input["target_control_carriers"][object].as_array().unwrap() {
                targets[object + 1][CARRIER_LANES[carrier.as_u64().unwrap() as usize]] =
                    0x3f80_0000;
            }
        }
        targets
    }

    fn assert_geometry_state(actual: &CoefficientGeometryState, expected: &Value) {
        for slot in 0..CARRY_SLOTS {
            assert_eq!(
                actual.carry18_words[slot].as_slice(),
                decode_u32_words(expected["carry18"][slot]["hex"].as_str().unwrap()).as_slice()
            );
        }
        for slot in 0..SLOT_COUNT {
            assert_eq!(
                actual.previous24_words[slot].as_slice(),
                decode_u32_words(expected["previous24"][slot]["hex"].as_str().unwrap()).as_slice()
            );
        }
    }

    fn assert_row(actual: &GeometryAccumulatorRow, expected: &Value) {
        for (name, words) in [
            ("current", &actual.current_words),
            ("target", &actual.target_words),
            ("slope", &actual.slope_words),
            ("remaining", &actual.remaining_words),
        ] {
            for lane in 0..LANE_COUNT {
                assert_eq!(
                    words[lane],
                    u32::from_le_bytes(
                        decode_hex(expected[name][lane]["hex"].as_str().unwrap())
                            .try_into()
                            .unwrap()
                    ),
                    "{name} lane {lane}"
                );
            }
        }
    }

    fn synthetic_target_current24(input: &Value, generation: usize) -> Vec<u32> {
        let amplitude = input["generations"][generation]["amplitude"]
            .as_f64()
            .unwrap() as f32;
        let transient_slot = input["generations"][generation]["transient_slot"]
            .as_u64()
            .unwrap() as usize;
        let multiplier = input["generations"][generation]["transient_multiplier"]
            .as_f64()
            .unwrap() as f32;
        let mut output = Vec::with_capacity(OBJECT_COUNT * 24 * 64 * 2);
        for object in 0..OBJECT_COUNT {
            let (alpha, beta) = match object % 3 {
                0 => (0.75_f32, 0.0_f32),
                1 => (0.75_f32, 0.5_f32),
                _ => (0.5_f32, -0.25_f32),
            };
            for slot in 0..24 {
                for bin in 0..64 {
                    let first = synthetic_carrier(object % 5, slot, bin, amplitude);
                    let second = synthetic_carrier((object + 2) % 5, slot, bin, amplitude);
                    let scale = if generation == 1 && object == 0 && slot >= transient_slot {
                        multiplier
                    } else {
                        1.0
                    };
                    for component in 0..2 {
                        output.push(
                            ((alpha * first[component] + beta * second[component]) * scale)
                                .to_bits(),
                        );
                    }
                }
            }
        }
        output
    }

    fn synthetic_carrier(carrier: usize, slot: usize, bin: usize, amplitude: f32) -> [f32; 2] {
        let bit_index = if carrier == 1 && bin == 0 { 0 } else { carrier };
        let real = amplitude
            * (1.0 + (bin % 3) as f32 / 4.0)
            * if (slot >> bit_index) & 1 == 0 {
                1.0
            } else {
                -1.0
            };
        let imaginary = amplitude / 2.0
            * if ((slot + 1) >> ((bit_index + 1) % 5)) & 1 == 0 {
                1.0
            } else {
                -1.0
            };
        [real, imaginary]
    }

    fn concatenate_target_windows(previous: &[u32], current: &[u32]) -> Vec<u32> {
        let words_per_half = 24 * 64 * 2;
        let mut output = Vec::with_capacity(OBJECT_COUNT * words_per_half * 2);
        for object in 0..OBJECT_COUNT {
            let start = object * words_per_half;
            let end = start + words_per_half;
            output.extend_from_slice(&previous[start..end]);
            output.extend_from_slice(&current[start..end]);
        }
        output
    }

    fn f64_to_f32_word(value: &Value) -> u32 {
        let bytes = decode_hex(value["f64le"].as_str().unwrap());
        (f64::from_le_bytes(bytes.try_into().unwrap()) as f32).to_bits()
    }

    fn decode_u32_words(hex: &str) -> Vec<u32> {
        decode_hex(hex)
            .chunks_exact(4)
            .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
            .collect()
    }

    fn decode_hex(hex: &str) -> Vec<u8> {
        hex.as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }
}
