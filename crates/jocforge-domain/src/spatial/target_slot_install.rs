// pattern: Functional Core
//! F03G.2-F03G.4 target-slot cost, greedy mapping, and value projection.

use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};

const SLOT_COUNT: usize = 15;
const MAX_FINITE_WORD: u32 = 0x7f7f_ffff;
const ZERO_WORD: u32 = 0;
const ID_MISMATCH_PENALTY_WORD: u32 = 0x41a0_0000;
const ATTRIBUTE_MISMATCH_PENALTY_WORD: u32 = 0x42c8_0000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetSlotState {
    pub xyz_words: [[u32; 3]; SLOT_COUNT],
    pub ids_u32: [u32; SLOT_COUNT],
    pub attributes_u32: [u32; SLOT_COUNT],
    pub previous_xyz_words: [[u32; 3]; SLOT_COUNT],
    pub previous_ids_u32: [u32; SLOT_COUNT],
    pub previous_attributes_u32: [u32; SLOT_COUNT],
    pub active_flags: [u8; SLOT_COUNT],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedTargetSlots {
    pub xyz_words: Vec<[u32; 3]>,
    pub ids_u32: Vec<u32>,
    pub attributes_u32: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetSlotInstallError {
    InvalidInput,
    NumericFailure,
    UnsupportedAttributeMultiplicity,
    NoFiniteAssignment,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetSlotInstallResult {
    mapping: Vec<i16>,
    cost_u32: Vec<u32>,
    empty_branch: bool,
    next_state: TargetSlotState,
}

impl TargetSlotInstallResult {
    pub fn mapping(&self) -> &[i16] {
        &self.mapping
    }

    pub fn cost_u32(&self) -> &[u32] {
        &self.cost_u32
    }

    pub fn empty_branch(&self) -> bool {
        self.empty_branch
    }

    pub fn next_state(&self) -> &TargetSlotState {
        &self.next_state
    }
}

/// Installs a selected target proposal into an explicit prior 15-slot state.
///
/// This is only the F03G target mapping/value transition. It does not construct
/// a cold state, run selection/projection, update marker/history state, or
/// commit a programme owner.
pub fn install_target_slots(
    old: &TargetSlotState,
    selected: &SelectedTargetSlots,
) -> Result<TargetSlotInstallResult, TargetSlotInstallError> {
    let count = selected.xyz_words.len();
    if count > SLOT_COUNT
        || selected.ids_u32.len() != count
        || selected.attributes_u32.len() != count
        || old.active_flags.iter().any(|flag| *flag > 1)
        || old
            .xyz_words
            .iter()
            .flatten()
            .chain(selected.xyz_words.iter().flatten())
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(TargetSlotInstallError::InvalidInput);
    }

    if count == 0 {
        return Ok(TargetSlotInstallResult {
            mapping: vec![-1; SLOT_COUNT],
            cost_u32: Vec::new(),
            empty_branch: true,
            next_state: old.clone(),
        });
    }

    for attribute in &selected.attributes_u32 {
        let available = old
            .attributes_u32
            .iter()
            .filter(|old_attribute| *old_attribute == attribute)
            .count();
        let required = selected
            .attributes_u32
            .iter()
            .filter(|new_attribute| *new_attribute == attribute)
            .count();
        if required > available {
            return Err(TargetSlotInstallError::UnsupportedAttributeMultiplicity);
        }
    }

    let mut cost_matrix = vec![vec![ZERO_WORD; count]; SLOT_COUNT];
    for (row, row_costs) in cost_matrix.iter_mut().enumerate() {
        for (column, cost_word) in row_costs.iter_mut().enumerate() {
            *cost_word = assignment_cost(
                &old.xyz_words[row],
                &selected.xyz_words[column],
                old.ids_u32[row] != selected.ids_u32[column],
                old.attributes_u32[row] != selected.attributes_u32[column],
            )?;
        }
    }

    let mut mapping = vec![-1; SLOT_COUNT];
    let mut work_cost = cost_matrix.clone();
    for _ in 0..count {
        let mut best = f32::from_bits(MAX_FINITE_WORD);
        let mut selected_pair = None;
        for (row, row_costs) in work_cost.iter().enumerate() {
            for (column, word) in row_costs.iter().enumerate() {
                let value = f32::from_bits(*word);
                if best > value {
                    best = value;
                    selected_pair = Some((row, column));
                }
            }
        }

        let (row, column) = selected_pair.ok_or(TargetSlotInstallError::NoFiniteAssignment)?;
        mapping[row] = i16::try_from(column).map_err(|_| TargetSlotInstallError::InvalidInput)?;
        work_cost[row].fill(MAX_FINITE_WORD);
        for row_costs in &mut work_cost {
            row_costs[column] = MAX_FINITE_WORD;
        }
    }

    let mut next_state = old.clone();
    for (row, mapped_column) in mapping.iter().copied().enumerate() {
        if mapped_column < 0 {
            next_state.active_flags[row] = 0;
            continue;
        }

        let column =
            usize::try_from(mapped_column).map_err(|_| TargetSlotInstallError::InvalidInput)?;
        next_state.previous_xyz_words[row] = old.xyz_words[row];
        next_state.previous_ids_u32[row] = old.ids_u32[row];
        next_state.previous_attributes_u32[row] = old.attributes_u32[row];
        next_state.xyz_words[row] = selected.xyz_words[column];
        next_state.ids_u32[row] = selected.ids_u32[column];
        next_state.attributes_u32[row] = selected.attributes_u32[column];
        next_state.active_flags[row] = 1;
    }

    Ok(TargetSlotInstallResult {
        mapping,
        cost_u32: cost_matrix.into_iter().flatten().collect(),
        empty_branch: false,
        next_state,
    })
}

fn assignment_cost(
    old_xyz: &[u32; 3],
    new_xyz: &[u32; 3],
    id_mismatch: bool,
    attribute_mismatch: bool,
) -> Result<u32, TargetSlotInstallError> {
    let mut distance = ZERO_WORD;
    for axis in 0..3 {
        let delta = subtract(old_xyz[axis], new_xyz[axis])?;
        let square = multiply(delta, delta)?;
        distance = add(distance, square)?;
    }

    let id_penalty = if id_mismatch {
        ID_MISMATCH_PENALTY_WORD
    } else {
        ZERO_WORD
    };
    let attribute_penalty = if attribute_mismatch {
        ATTRIBUTE_MISMATCH_PENALTY_WORD
    } else {
        ZERO_WORD
    };
    let with_id = add(distance, id_penalty)?;
    let with_attribute = add(with_id, attribute_penalty)?;
    let cost = add(with_attribute, ZERO_WORD)?;
    if cost >= MAX_FINITE_WORD {
        return Err(TargetSlotInstallError::NumericFailure);
    }
    Ok(cost)
}

fn finite(word: u32) -> Result<FiniteF32, TargetSlotInstallError> {
    FiniteF32::new(f32::from_bits(word)).map_err(|_| TargetSlotInstallError::NumericFailure)
}

fn add(left: u32, right: u32) -> Result<u32, TargetSlotInstallError> {
    r32_add(finite(left)?, finite(right)?)
        .map(FiniteF32::to_bits)
        .map_err(|_| TargetSlotInstallError::NumericFailure)
}

fn subtract(left: u32, right: u32) -> Result<u32, TargetSlotInstallError> {
    r32_sub(finite(left)?, finite(right)?)
        .map(FiniteF32::to_bits)
        .map_err(|_| TargetSlotInstallError::NumericFailure)
}

fn multiply(left: u32, right: u32) -> Result<u32, TargetSlotInstallError> {
    r32_mul(finite(left)?, finite(right)?)
        .map(FiniteF32::to_bits)
        .map_err(|_| TargetSlotInstallError::NumericFailure)
}
