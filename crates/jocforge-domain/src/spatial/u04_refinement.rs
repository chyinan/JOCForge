// pattern: Functional Core
//! One callback-local F03U geometric refinement and projection proposal.

use crate::numeric::{FiniteF32, r32_add};
use crate::spatial::{
    candidate_selector::{CandidateSelectorInput, POWER_EXPONENT_WORD, SelectorError, pow32},
    target_slot_install::SelectedTargetSlots,
    u04_projection::{U04ProjectionError, project_targets},
};

const FEATURE_BANDS: usize = 24;
const MAX_SOURCE_COUNT: usize = 48;
const MAX_TARGET_COUNT: usize = 15;
const ZERO_F64: f64 = 0.0;
const ONE_F64: f64 = 1.0;
const TWO_F64: f64 = 2.0;
const ATTRIBUTE_PENALTY: f64 = 0.1;
const DISTANCE_SCALE: f64 = 0.01;
const DUPLICATE_TAU_WORD: u32 = duplicate_tau_word();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum U04RefinementError {
    InvalidInput,
    TargetDomainChanged,
    NumericFailure,
}

impl U04RefinementError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidInput => "U04_REFINEMENT_INPUT",
            Self::TargetDomainChanged => "U04_TARGET_DOMAIN_CHANGED",
            Self::NumericFailure => "U04_REFINEMENT_NUMERIC",
        }
    }
}

/// Noncommitting result for the selected order and original candidate order.
#[must_use]
#[derive(Clone, Debug, PartialEq)]
pub struct U04RefinementProposal {
    source_ids: Box<[u32]>,
    source_xyz_u32: Box<[[u32; 3]]>,
    source_attributes_u32: Box<[u32]>,
    selected_target_ids: Box<[u32]>,
    initial_target_xyz_u32: Box<[[u32; 3]]>,
    selected_target_attributes: Box<[u32]>,
    source_weight_u32: Box<[u32]>,
    signed_solution_f64: Box<[Box<[f64]>]>,
    membership_gain_u32: Box<[Box<[u32]>]>,
    normalized_m_u32: Box<[Box<[u32]>]>,
    projected_xyz_u32: Box<[[u32; 3]]>,
}

impl U04RefinementProposal {
    pub fn source_ids(&self) -> &[u32] {
        &self.source_ids
    }

    pub fn source_xyz_u32(&self) -> &[[u32; 3]] {
        &self.source_xyz_u32
    }

    pub fn source_attributes_u32(&self) -> &[u32] {
        &self.source_attributes_u32
    }

    pub fn selected_target_ids(&self) -> &[u32] {
        &self.selected_target_ids
    }

    pub fn initial_target_xyz_u32(&self) -> &[[u32; 3]] {
        &self.initial_target_xyz_u32
    }

    pub fn selected_target_attributes(&self) -> &[u32] {
        &self.selected_target_attributes
    }

    pub fn source_weight_u32(&self) -> &[u32] {
        &self.source_weight_u32
    }

    pub fn signed_solution_f64(&self) -> &[Box<[f64]>] {
        &self.signed_solution_f64
    }

    /// Source-major `[N][T]` geometric gains, in original candidate order.
    pub fn membership_gain_u32(&self) -> &[Box<[u32]>] {
        &self.membership_gain_u32
    }

    /// Target-major `[T][N]` normalized projection matrix.
    pub fn normalized_m_u32(&self) -> &[Box<[u32]>] {
        &self.normalized_m_u32
    }

    pub fn projected_xyz_u32(&self) -> &[[u32; 3]] {
        &self.projected_xyz_u32
    }
}

/// Runs only after the caller has admitted the selector's CAPACITY_REACHED exit.
///
/// The source rows and features remain in original candidate order. Targets remain in selector
/// order and are never updated while the per-source systems are solved.
pub(crate) fn produce_u04_refinement(
    candidates: &CandidateSelectorInput,
    selected_targets: &SelectedTargetSlots,
) -> Result<U04RefinementProposal, U04RefinementError> {
    validate_inputs(candidates, selected_targets)?;
    reject_target_domain_change(selected_targets)?;

    let source_count = candidates.candidate_ids.len();
    let source_xyz = candidates
        .positions_u32
        .iter()
        .map(|xyz| xyz.map(|word| f64::from(f32::from_bits(word))))
        .collect::<Vec<_>>();
    let target_xyz = selected_targets
        .xyz_words
        .iter()
        .map(|xyz| xyz.map(|word| f64::from(f32::from_bits(word))))
        .collect::<Vec<_>>();

    let mut source_weight_u32 = Vec::with_capacity(source_count);
    for source in 0..source_count {
        let mut total = 0_u32;
        for band in 0..FEATURE_BANDS {
            let powered = pow32(candidates.feature_u32[band][source], POWER_EXPONENT_WORD)
                .map_err(map_selector_error)?;
            total = add32(total, powered)?;
        }
        source_weight_u32.push(total);
    }

    let mut signed_solution_f64 = Vec::with_capacity(source_count);
    let mut membership_gain_u32 = Vec::with_capacity(source_count);
    for (source, source_xyz_row) in source_xyz.iter().copied().enumerate() {
        let (solution, gains) = solve_source(
            source_xyz_row,
            candidates.attributes_u32[source],
            &target_xyz,
            &selected_targets.attributes_u32,
        )?;
        signed_solution_f64.push(solution.into_boxed_slice());
        membership_gain_u32.push(gains.into_boxed_slice());
    }

    // Keep source-major gains distinct from the target-major projection matrix.
    let projection_gain = membership_gain_u32
        .iter()
        .map(|row| row.to_vec())
        .collect::<Vec<_>>();
    let projection = project_targets(
        &candidates.positions_u32,
        &selected_targets.xyz_words,
        &selected_targets.attributes_u32,
        &source_weight_u32,
        &projection_gain,
    )
    .map_err(map_projection_error)?;

    Ok(U04RefinementProposal {
        source_ids: candidates.candidate_ids.clone().into_boxed_slice(),
        source_xyz_u32: candidates.positions_u32.clone().into_boxed_slice(),
        source_attributes_u32: candidates.attributes_u32.clone().into_boxed_slice(),
        selected_target_ids: selected_targets.ids_u32.clone().into_boxed_slice(),
        initial_target_xyz_u32: selected_targets.xyz_words.clone().into_boxed_slice(),
        selected_target_attributes: selected_targets.attributes_u32.clone().into_boxed_slice(),
        source_weight_u32: source_weight_u32.into_boxed_slice(),
        signed_solution_f64: signed_solution_f64.into_boxed_slice(),
        membership_gain_u32: membership_gain_u32.into_boxed_slice(),
        normalized_m_u32: projection
            .normalized_m_u32()
            .iter()
            .map(|row| row.clone().into_boxed_slice())
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        projected_xyz_u32: projection.projected_xyz_u32().to_vec().into_boxed_slice(),
    })
}

fn validate_inputs(
    candidates: &CandidateSelectorInput,
    selected_targets: &SelectedTargetSlots,
) -> Result<(), U04RefinementError> {
    let source_count = candidates.candidate_ids.len();
    let target_count = selected_targets.xyz_words.len();
    if !(1..=MAX_SOURCE_COUNT).contains(&source_count)
        || !(1..=MAX_TARGET_COUNT).contains(&target_count)
        || candidates.positions_u32.len() != source_count
        || candidates.attributes_u32.len() != source_count
        || candidates.feature_u32.len() != FEATURE_BANDS
        || candidates
            .feature_u32
            .iter()
            .any(|band| band.len() != source_count)
        || selected_targets.ids_u32.len() != target_count
        || selected_targets.attributes_u32.len() != target_count
        || candidates.candidate_ids.iter().any(|id| *id >= 48)
        || candidates
            .candidate_ids
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        || selected_targets
            .ids_u32
            .iter()
            .enumerate()
            .any(|(index, id)| {
                candidates.candidate_ids.binary_search(id).is_err()
                    || selected_targets.ids_u32[..index].contains(id)
            })
        || selected_targets
            .ids_u32
            .iter()
            .zip(&selected_targets.attributes_u32)
            .zip(&selected_targets.xyz_words)
            .any(|((id, attribute), target_xyz)| {
                candidates
                    .candidate_ids
                    .binary_search(id)
                    .ok()
                    .is_none_or(|ordinal| {
                        candidates.attributes_u32[ordinal] != *attribute
                            || candidates.positions_u32[ordinal] != *target_xyz
                    })
            })
        || candidates
            .positions_u32
            .iter()
            .flatten()
            .chain(selected_targets.xyz_words.iter().flatten())
            .chain(candidates.feature_u32.iter().flatten())
            .any(|word| !f32::from_bits(*word).is_finite())
        || candidates
            .feature_u32
            .iter()
            .flatten()
            .any(|word| f32::from_bits(*word) < 0.0)
    {
        return Err(U04RefinementError::InvalidInput);
    }
    Ok(())
}

fn reject_target_domain_change(
    selected_targets: &SelectedTargetSlots,
) -> Result<(), U04RefinementError> {
    let tau = f64::from(f32::from_bits(DUPLICATE_TAU_WORD));
    for current in 0..selected_targets.xyz_words.len() {
        for previous in 0..current {
            if selected_targets.attributes_u32[current] == selected_targets.attributes_u32[previous]
                && squared_distance(
                    selected_targets.xyz_words[current],
                    selected_targets.xyz_words[previous],
                )? <= tau
            {
                return Err(U04RefinementError::TargetDomainChanged);
            }
        }
    }
    Ok(())
}

fn squared_distance(left: [u32; 3], right: [u32; 3]) -> Result<f64, U04RefinementError> {
    let mut total = ZERO_F64;
    for axis in 0..3 {
        let difference = checked_f64(
            f64::from(f32::from_bits(left[axis])) - f64::from(f32::from_bits(right[axis])),
        )?;
        let square = checked_f64(difference * difference)?;
        total = checked_f64(total + square)?;
    }
    Ok(total)
}

fn solve_source(
    source_xyz: [f64; 3],
    source_attribute: u32,
    target_xyz: &[[f64; 3]],
    target_attributes: &[u32],
) -> Result<(Vec<f64>, Vec<u32>), U04RefinementError> {
    let target_count = target_xyz.len();
    let mut q = ZERO_F64;
    for coordinate in source_xyz {
        q = checked_f64(q + checked_f64(coordinate * coordinate)?)?;
    }

    let mut p = vec![ZERO_F64; target_count];
    let mut g = vec![vec![ZERO_F64; target_count]; target_count];
    for target in 0..target_count {
        p[target] = dot(target_xyz[target], source_xyz)?;
        for other in 0..target_count {
            g[target][other] = dot(target_xyz[target], target_xyz[other])?;
        }
    }

    let mut b = vec![vec![ZERO_F64; target_count]; target_count];
    for row in 0..target_count {
        for column in 0..target_count {
            let q_plus_one = checked_f64(q + ONE_F64)?;
            let plus_geometry = checked_f64(q_plus_one + g[row][column])?;
            let twice_projection = checked_f64(TWO_F64 * p[column])?;
            b[row][column] = checked_f64(plus_geometry - twice_projection)?;
        }
    }

    let mut lower_system = vec![vec![ZERO_F64; target_count]; target_count];
    for row in 0..target_count {
        let distance = squared_distance_f64(source_xyz, target_xyz[row])?;
        let attribute_penalty = if target_attributes[row] == source_attribute {
            ZERO_F64
        } else {
            ATTRIBUTE_PENALTY
        };
        let distance_penalty = checked_f64(distance * DISTANCE_SCALE)?;
        let first_diagonal = checked_f64(attribute_penalty + distance_penalty)?;
        lower_system[row][row] = checked_f64(first_diagonal + b[row][row])?;
        for column in 0..target_count {
            if row != column {
                lower_system[row][column] = b[column][row];
            }
        }
    }

    let mut symmetric = vec![vec![ZERO_F64; target_count]; target_count];
    for row in 0..target_count {
        for column in 0..target_count {
            symmetric[row][column] =
                checked_f64(lower_system[row][column] + lower_system[column][row])?;
        }
    }

    let mut cholesky = vec![vec![ZERO_F64; target_count]; target_count];
    for row in 0..target_count {
        for column in 0..=row {
            let mut sum = ZERO_F64;
            for (&row_value, &column_value) in cholesky[row][..column]
                .iter()
                .zip(&cholesky[column][..column])
            {
                let product = checked_f64(row_value * column_value)?;
                sum = checked_f64(sum + product)?;
            }
            if row == column {
                let residual = checked_f64(symmetric[row][row] - sum)?;
                cholesky[row][column] = if residual > ZERO_F64 {
                    checked_f64(residual.sqrt())?
                } else {
                    ZERO_F64
                };
            } else {
                let numerator = checked_f64(symmetric[row][column] - sum)?;
                let diagonal = cholesky[column][column];
                if diagonal == ZERO_F64 {
                    return Err(U04RefinementError::NumericFailure);
                }
                cholesky[row][column] = checked_f64(numerator / diagonal)?;
            }
        }
    }

    let mut forward = vec![ZERO_F64; target_count];
    for row in 0..target_count {
        let mut value = -TWO_F64;
        for column in 0..row {
            value = checked_f64(value - checked_f64(cholesky[row][column] * forward[column])?)?;
        }
        let diagonal = cholesky[row][row];
        if diagonal == ZERO_F64 {
            return Err(U04RefinementError::NumericFailure);
        }
        forward[row] = checked_f64(value / diagonal)?;
    }

    let mut solution = vec![ZERO_F64; target_count];
    for row in (0..target_count).rev() {
        let mut value = forward[row];
        for column in (row + 1)..target_count {
            value = checked_f64(value - checked_f64(cholesky[column][row] * solution[column])?)?;
        }
        let diagonal = cholesky[row][row];
        if diagonal == ZERO_F64 {
            return Err(U04RefinementError::NumericFailure);
        }
        solution[row] = checked_f64(value / diagonal)?;
    }

    let gains = solution
        .iter()
        .map(|value| {
            let nonnegative = checked_f64((-*value).max(ZERO_F64))?;
            FiniteF32::new(nonnegative as f32)
                .map(FiniteF32::to_bits)
                .map_err(|_| U04RefinementError::NumericFailure)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((solution, gains))
}

fn dot(left: [f64; 3], right: [f64; 3]) -> Result<f64, U04RefinementError> {
    let mut total = ZERO_F64;
    for axis in 0..3 {
        total = checked_f64(total + checked_f64(left[axis] * right[axis])?)?;
    }
    Ok(total)
}

fn squared_distance_f64(left: [f64; 3], right: [f64; 3]) -> Result<f64, U04RefinementError> {
    let mut total = ZERO_F64;
    for axis in 0..3 {
        let difference = checked_f64(left[axis] - right[axis])?;
        total = checked_f64(total + checked_f64(difference * difference)?)?;
    }
    Ok(total)
}

fn add32(left: u32, right: u32) -> Result<u32, U04RefinementError> {
    let left =
        FiniteF32::new(f32::from_bits(left)).map_err(|_| U04RefinementError::NumericFailure)?;
    let right =
        FiniteF32::new(f32::from_bits(right)).map_err(|_| U04RefinementError::NumericFailure)?;
    r32_add(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| U04RefinementError::NumericFailure)
}

fn checked_f64(value: f64) -> Result<f64, U04RefinementError> {
    value
        .is_finite()
        .then_some(value)
        .ok_or(U04RefinementError::NumericFailure)
}

fn map_selector_error(_error: SelectorError) -> U04RefinementError {
    U04RefinementError::NumericFailure
}

fn map_projection_error(error: U04ProjectionError) -> U04RefinementError {
    match error {
        U04ProjectionError::InvalidInput => U04RefinementError::InvalidInput,
        U04ProjectionError::NumericFailure => U04RefinementError::NumericFailure,
    }
}

const fn duplicate_tau_word() -> u32 {
    let reciprocal = 1.0_f32 / 62.0_f32;
    (reciprocal * reciprocal).to_bits()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::candidate_selector::SelectorStatus;

    fn candidate_input(
        ids: &[u32],
        positions: &[[f32; 3]],
        attributes: &[u32],
    ) -> CandidateSelectorInput {
        let count = ids.len();
        let response = vec![vec![0.5_f32.to_bits(); count]; 15];
        CandidateSelectorInput {
            candidate_ids: ids.to_vec(),
            feature_u32: vec![vec![1.0_f32.to_bits(); count]; FEATURE_BANDS],
            positions_u32: positions.iter().map(|xyz| xyz.map(f32::to_bits)).collect(),
            width_u32: vec![1.0_f32.to_bits(); count],
            attributes_u32: attributes.to_vec(),
            response_u32: response,
            previous_ids: Vec::new(),
        }
    }

    fn selected_targets(
        ids: &[u32],
        positions: &[[f32; 3]],
        attributes: &[u32],
    ) -> SelectedTargetSlots {
        SelectedTargetSlots {
            xyz_words: positions.iter().map(|xyz| xyz.map(f32::to_bits)).collect(),
            ids_u32: ids.to_vec(),
            attributes_u32: attributes.to_vec(),
        }
    }

    fn separated_positions(count: usize) -> Vec<[f32; 3]> {
        (0..count)
            .map(|index| [index as f32 / 16.0, 0.0, 0.0])
            .collect()
    }

    #[test]
    fn candidate16_weights_and_gain_rows_keep_candidate_order_with_selected15_order() {
        let ids = [0, 2, 5, 6, 8, 9, 11, 14, 17, 20, 23, 27, 32, 36, 41, 47];
        let positions = separated_positions(ids.len());
        let attrs = vec![0x0b00; ids.len()];
        let mut candidates = candidate_input(&ids, &positions, &attrs);
        for (ordinal, id) in ids.iter().copied().enumerate() {
            for band in 0..FEATURE_BANDS {
                candidates.feature_u32[band][ordinal] = if band <= ordinal {
                    1.0_f32.to_bits()
                } else {
                    0
                };
            }
            assert_eq!(candidates.candidate_ids[ordinal], id);
        }
        let selected_order = [41, 0, 36, 2, 32, 5, 27, 6, 23, 8, 20, 9, 17, 11, 14];
        let selected_positions = selected_order
            .iter()
            .map(|id| positions[ids.iter().position(|candidate| candidate == id).unwrap()])
            .collect::<Vec<_>>();
        let selected_attrs = vec![0x0b00; selected_order.len()];
        let proposal = produce_u04_refinement(
            &candidates,
            &selected_targets(&selected_order, &selected_positions, &selected_attrs),
        )
        .unwrap();

        assert_eq!(proposal.source_ids(), ids);
        assert_eq!(proposal.source_weight_u32().len(), 16);
        assert_eq!(proposal.membership_gain_u32().len(), 16);
        assert_eq!(proposal.membership_gain_u32()[15].len(), 15);
        assert_eq!(proposal.selected_target_ids(), selected_order);
        assert_eq!(proposal.projected_xyz_u32().len(), 15);
        assert_ne!(
            proposal.source_weight_u32()[0],
            proposal.source_weight_u32()[15]
        );
    }

    #[test]
    fn zero_weight_and_zero_gain_follow_projection_epsilon_path() {
        let ids = [0];
        let positions = [[0.0, 0.0, 0.0]];
        let attrs = [0x0b00];
        let mut candidates = candidate_input(&ids, &positions, &attrs);
        candidates.feature_u32 = vec![vec![0]; FEATURE_BANDS];
        let selected = selected_targets(&ids, &positions, &attrs);
        let proposal = produce_u04_refinement(&candidates, &selected).unwrap();
        assert_eq!(proposal.source_weight_u32(), &[0]);
        assert!(proposal.normalized_m_u32()[0].iter().all(|word| *word == 0));

        let zero = project_targets(&[[0; 3]], &[[0; 3]], &[0], &[0], &[vec![0]]).unwrap();
        assert_eq!(zero.normalized_m_u32(), &[vec![0]]);
        assert_eq!(zero.projected_xyz_u32(), &[[0; 3]]);
    }

    #[test]
    fn duplicate_target_domain_is_rejected_before_projection() {
        let ids = [0, 1];
        let positions = [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let attrs = [0x0b00, 0x0b00];
        let candidates = candidate_input(&ids, &positions, &attrs);
        let selected =
            selected_targets(&ids, &[[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]], &[0x0b00, 0x0b00]);
        assert_eq!(
            produce_u04_refinement(&candidates, &selected),
            Err(U04RefinementError::TargetDomainChanged)
        );
    }

    #[test]
    fn protected_anchor_keeps_original_target_bits() {
        let ids = [0];
        let source_positions = [[-0.0, 0.125, 0.5]];
        let target_positions = source_positions;
        let source_attrs = [0x0b01];
        let target_attrs = [0x0b01];
        let candidates = candidate_input(&ids, &source_positions, &source_attrs);
        let selected = selected_targets(&ids, &target_positions, &target_attrs);
        let proposal = produce_u04_refinement(&candidates, &selected).unwrap();
        assert_eq!(
            proposal.projected_xyz_u32(),
            &target_positions.map(|xyz| xyz.map(f32::to_bits))
        );
        assert_eq!(proposal.selected_target_attributes(), &target_attrs);
    }

    #[test]
    fn selected_target_xyz_must_match_its_candidate_identity() {
        let ids = [0];
        let source_positions = [[0.0, 0.0, 0.0]];
        let source_attrs = [0x0b00];
        let candidates = candidate_input(&ids, &source_positions, &source_attrs);
        let stale_target = selected_targets(&ids, &[[0.25, 0.0, 0.0]], &source_attrs);
        assert_eq!(
            produce_u04_refinement(&candidates, &stale_target),
            Err(U04RefinementError::InvalidInput)
        );
    }

    #[test]
    fn invalid_numeric_inputs_fail_without_a_partial_proposal() {
        let ids = [0];
        let positions = [[0.0, 0.0, 0.0]];
        let attrs = [0x0b00];
        let mut candidates = candidate_input(&ids, &positions, &attrs);
        candidates.positions_u32[0][1] = f32::INFINITY.to_bits();
        assert_eq!(
            produce_u04_refinement(&candidates, &selected_targets(&ids, &positions, &attrs)),
            Err(U04RefinementError::InvalidInput)
        );

        let mut candidates = candidate_input(&ids, &positions, &attrs);
        candidates.feature_u32[0][0] = f32::NAN.to_bits();
        assert_eq!(
            produce_u04_refinement(&candidates, &selected_targets(&ids, &positions, &attrs)),
            Err(U04RefinementError::InvalidInput)
        );
    }

    #[test]
    fn projection_rejects_gain_sum_overflow() {
        assert_eq!(
            project_targets(
                &[[0; 3]],
                &[[0; 3]; 2],
                &[0, 0],
                &[1.0_f32.to_bits()],
                &[vec![f32::MAX.to_bits(); 2]],
            ),
            Err(U04ProjectionError::NumericFailure)
        );
    }

    #[test]
    fn non_capacity_status_is_not_an_input_to_the_producer() {
        let capacity = crate::spatial::u04_projection::u04_refinement_admission(
            SelectorStatus::CapacityReached,
            15,
        )
        .unwrap();
        let exhausted = crate::spatial::u04_projection::u04_refinement_admission(
            SelectorStatus::CandidatesExhausted,
            15,
        )
        .unwrap();
        assert!(capacity.refinement_called());
        assert!(!exhausted.refinement_called());
    }
}
