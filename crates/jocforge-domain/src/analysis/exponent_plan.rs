// pattern: Functional Core
//! Selected same-source F09 refresh, F09.4 strategy, and F10 anchor/reuse planning.

use super::owner::{F08AnalysisPlan, F09GainDbPlan};
use super::source_frame::F07SourceFrameOutput;
use super::structure_refresh::{F07_BLOCKS_PER_FRAME, F07_FULLBAND_LANES};
use crate::codec::exponent_normalization::{
    ExponentNormalizationError, FULLBAND_EXPONENT_BINS, LFE_EXPONENT_BINS, NormalizedExponentRow,
    NormalizedLfeExponentRow, normalize_fullband_exponents, normalize_lfe_exponents,
};
use crate::codec::exponent_packing::{
    ExponentPackingError, PackedExponentGroup, pack_exponent_difference_group,
};
use crate::codec::exponent_refresh_state::refresh_exponent_state;
use crate::codec::policy_parameters::{PolicyParameters, PolicyProfile};
use crate::transform::exponent_inputs::{
    FullBandTransformExponentInputs, LfeTransformExponentInputs,
};

const LFE_MARKER_BEFORE: [i16; F07_BLOCKS_PER_FRAME] = [1, 0, 0, 0, 0, 0];
const FBW_ACTIVE_MODE_I16: i16 = 1;
const LFE_ACTIVE_MODE_I16: i16 = 0;
const REFRESH_SPECIAL_I16: i16 = 0;
const REFRESH_BEGIN_I16: i16 = 0;
const FBW_REFRESH_END_I16: i16 = 128;
const LFE_REFRESH_END_I16: i16 = 7;
const FBW_GROUP_COUNT: usize = FULLBAND_EXPONENT_BINS - 1;
const FBW_PACKED_GROUP_COUNT: usize = FBW_GROUP_COUNT / 3;
const LFE_PACKED_GROUP_COUNT: usize = (LFE_EXPONENT_BINS - 1) / 3;

/// One block position within the selected six-block analysis frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AnalysisBlockIndex(u8);

impl AnalysisBlockIndex {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

/// One full-band lane in the selected internal `[L,C,R,Ls,Rs]` order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FullbandLaneIndex(u8);

impl FullbandLaneIndex {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

/// F10 values for one FBW block after anchor normalization or reuse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FullbandExponentPlanBlock {
    strategy: u8,
    base_exponent: u8,
    decoded_exponents: [u8; FULLBAND_EXPONENT_BINS],
    packed_groups: Vec<PackedExponentGroup>,
    gain_range: u8,
}

impl FullbandExponentPlanBlock {
    pub const fn strategy(&self) -> u8 {
        self.strategy
    }

    pub const fn base_exponent(&self) -> u8 {
        self.base_exponent
    }

    pub const fn decoded_exponents(&self) -> &[u8; FULLBAND_EXPONENT_BINS] {
        &self.decoded_exponents
    }

    pub fn packed_groups(&self) -> &[PackedExponentGroup] {
        &self.packed_groups
    }

    pub const fn gain_range(&self) -> u8 {
        self.gain_range
    }
}

/// F10 values for one LFE block after independent anchor normalization or reuse.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LfeExponentPlanBlock {
    strategy: u8,
    base_exponent: u8,
    decoded_exponents: [u8; LFE_EXPONENT_BINS],
    packed_groups: [PackedExponentGroup; LFE_PACKED_GROUP_COUNT],
}

impl LfeExponentPlanBlock {
    pub const fn strategy(self) -> u8 {
        self.strategy
    }

    pub const fn base_exponent(self) -> u8 {
        self.base_exponent
    }

    pub const fn decoded_exponents(self) -> [u8; LFE_EXPONENT_BINS] {
        self.decoded_exponents
    }

    pub const fn packed_groups(self) -> [PackedExponentGroup; LFE_PACKED_GROUP_COUNT] {
        self.packed_groups
    }
}

/// Six-block source-local POWER refresh and PRIMARY F10 anchor/reuse plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisExponentPlan {
    fullband: Vec<FullbandExponentPlanBlock>,
    lfe: Vec<LfeExponentPlanBlock>,
    first_power_refresh_current_after_i16:
        [[[i16; FULLBAND_EXPONENT_BINS]; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
    fullband_refresh_marker_after: [[i16; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
    fullband_strategy: [[u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
    lfe_refresh_marker_after: [i16; F07_BLOCKS_PER_FRAME],
}

impl AnalysisExponentPlan {
    pub fn is_six_block_complete(&self) -> bool {
        self.fullband.len() == F07_BLOCKS_PER_FRAME * F07_FULLBAND_LANES
            && self.lfe.len() == F07_BLOCKS_PER_FRAME
    }

    pub fn fullband_block(
        &self,
        block: AnalysisBlockIndex,
        lane: FullbandLaneIndex,
    ) -> Option<&FullbandExponentPlanBlock> {
        let block = usize::from(block.value());
        let lane = usize::from(lane.value());
        if block >= F07_BLOCKS_PER_FRAME || lane >= F07_FULLBAND_LANES {
            return None;
        }
        self.fullband.get(block * F07_FULLBAND_LANES + lane)
    }

    pub fn lfe_block(&self, block: AnalysisBlockIndex) -> Option<LfeExponentPlanBlock> {
        let block = usize::from(block.value());
        self.lfe.get(block).copied()
    }

    /// Returns S: the current row immediately after the first main F09 POWER refresh.
    /// This preserves current-limiting effects and is distinct from raw input, work-after,
    /// and PRIMARY rows for the same source block/lane.
    pub fn first_power_refresh_current_after_i16(
        &self,
        block: AnalysisBlockIndex,
        lane: FullbandLaneIndex,
    ) -> Option<&[i16; FULLBAND_EXPONENT_BINS]> {
        let block = usize::from(block.value());
        let lane = usize::from(lane.value());
        if block >= F07_BLOCKS_PER_FRAME || lane >= F07_FULLBAND_LANES {
            return None;
        }
        Some(&self.first_power_refresh_current_after_i16[block][lane])
    }

    pub const fn fullband_refresh_marker_after(
        &self,
    ) -> &[[i16; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME] {
        &self.fullband_refresh_marker_after
    }

    pub const fn fullband_strategy(&self) -> &[[u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME] {
        &self.fullband_strategy
    }

    pub const fn lfe_refresh_marker_after(&self) -> &[i16; F07_BLOCKS_PER_FRAME] {
        &self.lfe_refresh_marker_after
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisExponentPlanError {
    IncompleteF08Plan,
    InvalidPowerRow,
    InvalidPrimaryRow,
    InvalidRefreshOutput,
    MissingF10Anchor,
    GainRangeOutOfRange,
    Normalization(ExponentNormalizationError),
    Packing(ExponentPackingError),
}

/// Runs the selected six-block POWER refresh, reverse strategy scan, and PRIMARY anchor plan.
///
/// F09 work rows are source-local: each FBW lane starts with zero and carries its returned
/// work row across blocks 0..5. `marker_before` comes from the matching F07 structure flag.
/// The LFE route has its own fixed marker-before sequence and does not read F07 flags.
pub fn stage_f09_f10_source_plan(
    f08: &F08AnalysisPlan,
    f07_output: &F07SourceFrameOutput,
    gain_db: &F09GainDbPlan,
    parameters: &PolicyParameters,
) -> Result<AnalysisExponentPlan, AnalysisExponentPlanError> {
    if !f08.is_six_block_complete() {
        return Err(AnalysisExponentPlanError::IncompleteF08Plan);
    }

    let mut fullband_refresh_marker_after = [[0_i16; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME];
    let mut first_power_refresh_current_after_i16 =
        [[[0_i16; FULLBAND_EXPONENT_BINS]; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME];
    let mut work_by_lane = [[0_i16; FULLBAND_EXPONENT_BINS]; F07_FULLBAND_LANES];
    for (block, marker_row) in fullband_refresh_marker_after.iter_mut().enumerate() {
        for (lane, (work, marker_after)) in work_by_lane
            .iter_mut()
            .zip(marker_row.iter_mut())
            .enumerate()
        {
            let input_index = block * F07_FULLBAND_LANES + lane;
            let exponent_input = f08
                .fullband_exponent_inputs()
                .get(input_index)
                .ok_or(AnalysisExponentPlanError::IncompleteF08Plan)?;
            let mut current = *exponent_input.power_raw_exponents_i16();
            if current.iter().any(|value| !(0..=24).contains(value))
                || work.iter().any(|value| !(0..=31).contains(value))
            {
                return Err(AnalysisExponentPlanError::InvalidPowerRow);
            }
            let mut marker = i16::from(f07_output.structure_refresh()[block][lane]);
            refresh_exponent_state(
                &mut current,
                work,
                &mut marker,
                FBW_ACTIVE_MODE_I16,
                REFRESH_SPECIAL_I16,
                REFRESH_BEGIN_I16,
                FBW_REFRESH_END_I16,
            );
            first_power_refresh_current_after_i16[block][lane] = current;
            if !matches!(marker, 0 | 1)
                || current.iter().any(|value| !(0..=31).contains(value))
                || work.iter().any(|value| !(0..=31).contains(value))
            {
                return Err(AnalysisExponentPlanError::InvalidRefreshOutput);
            }
            *marker_after = marker;
        }
    }

    let fullband_strategy =
        fullband_strategies(&fullband_refresh_marker_after, f07_output.short_consumed())?;
    let anchor_plans = stage_f10_anchor_plans(f08, gain_db, &fullband_strategy, parameters)?;

    Ok(AnalysisExponentPlan {
        fullband: anchor_plans.fullband,
        lfe: anchor_plans.lfe,
        first_power_refresh_current_after_i16,
        fullband_refresh_marker_after,
        fullband_strategy,
        lfe_refresh_marker_after: anchor_plans.lfe_refresh_marker_after,
    })
}

fn fullband_strategies(
    markers_after: &[[i16; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
    short_consumed: &[[u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
) -> Result<[[u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME], AnalysisExponentPlanError> {
    let mut strategies = [[0_u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME];
    for lane in 0..F07_FULLBAND_LANES {
        let mut distance = 0_usize;
        for block in (0..F07_BLOCKS_PER_FRAME).rev() {
            let marker = markers_after[block][lane];
            if !matches!(marker, 0 | 1) || short_consumed[block][lane] > 1 {
                return Err(AnalysisExponentPlanError::InvalidRefreshOutput);
            }
            let mut strategy = if marker == 0 {
                distance += 1;
                0
            } else {
                let strategy = if distance < 2 {
                    3
                } else if distance < 4 {
                    2
                } else {
                    1
                };
                distance = 0;
                strategy
            };
            if short_consumed[block][lane] == 1 {
                strategy = 3;
            }
            strategies[block][lane] = strategy;
        }
    }
    Ok(strategies)
}

fn stage_f10_anchor_plans(
    f08: &F08AnalysisPlan,
    gain_db: &F09GainDbPlan,
    fullband_strategy: &[[u8; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
    parameters: &PolicyParameters,
) -> Result<F10AnchorPlans, AnalysisExponentPlanError> {
    let profile = PolicyProfile::selected();
    let normalization_gate = i16::from(profile.normalization_gate);
    let normalization_option = profile.normalization_option;
    let mut fullband_slots = (0..F07_BLOCKS_PER_FRAME * F07_FULLBAND_LANES)
        .map(|_| None)
        .collect::<Vec<Option<FullbandExponentPlanBlock>>>();

    for lane in 0..F07_FULLBAND_LANES {
        let strategies: [u8; F07_BLOCKS_PER_FRAME] =
            std::array::from_fn(|block| fullband_strategy[block][lane]);
        let mut anchor_block = None;
        for block in 0..F07_BLOCKS_PER_FRAME {
            let strategy = strategies[block];
            if strategy == 0 {
                if anchor_block.is_none() {
                    return Err(AnalysisExponentPlanError::MissingF10Anchor);
                }
                continue;
            }
            let end = strategies
                .iter()
                .enumerate()
                .skip(block + 1)
                .find_map(|(next, next_strategy)| (*next_strategy != 0).then_some(next))
                .unwrap_or(F07_BLOCKS_PER_FRAME);
            let raw_rows = (block..end)
                .map(|source_block| {
                    f08.fullband_exponent_inputs()
                        .get(source_block * F07_FULLBAND_LANES + lane)
                        .map(FullBandTransformExponentInputs::primary_raw_exponents_i16)
                        .ok_or(AnalysisExponentPlanError::IncompleteF08Plan)
                })
                .collect::<Result<Vec<_>, _>>()?;
            if raw_rows
                .iter()
                .any(|row| row.iter().any(|value| !(0..=24).contains(value)))
            {
                return Err(AnalysisExponentPlanError::InvalidPrimaryRow);
            }
            let merged_primary =
                std::array::from_fn(|bin| raw_rows.iter().map(|row| row[bin]).min().unwrap_or(0));
            let gain_range = gain_db
                .gain_codes_by_block()
                .get(block..end)
                .ok_or(AnalysisExponentPlanError::GainRangeOutOfRange)?
                .iter()
                .map(|row| row[lane])
                .min()
                .ok_or(AnalysisExponentPlanError::GainRangeOutOfRange)?;
            if !(0..=3).contains(&gain_range) {
                return Err(AnalysisExponentPlanError::GainRangeOutOfRange);
            }
            let span = span_for_strategy(strategy)?;
            let normalized = normalize_fullband_exponents(
                &merged_primary,
                span,
                normalization_option,
                normalization_gate,
                Some(parameters),
            )
            .map_err(AnalysisExponentPlanError::Normalization)?;
            let plan = fullband_plan_block(
                strategy,
                u8::try_from(gain_range)
                    .map_err(|_| AnalysisExponentPlanError::GainRangeOutOfRange)?,
                &normalized,
            )?;
            for source_block in block..end {
                let mut reuse = plan.clone();
                reuse.strategy = strategies[source_block];
                fullband_slots[source_block * F07_FULLBAND_LANES + lane] = Some(reuse);
            }
            anchor_block = Some(block);
        }
    }
    let fullband = fullband_slots
        .into_iter()
        .map(|slot| slot.ok_or(AnalysisExponentPlanError::MissingF10Anchor))
        .collect::<Result<Vec<_>, _>>()?;

    let (lfe_refresh_marker_after, lfe) = stage_lfe_f09_f10(f08, parameters)?;

    Ok(F10AnchorPlans {
        fullband,
        lfe_refresh_marker_after,
        lfe,
    })
}

struct F10AnchorPlans {
    fullband: Vec<FullbandExponentPlanBlock>,
    lfe_refresh_marker_after: [i16; F07_BLOCKS_PER_FRAME],
    lfe: Vec<LfeExponentPlanBlock>,
}

fn stage_lfe_f09_f10(
    f08: &F08AnalysisPlan,
    parameters: &PolicyParameters,
) -> Result<([i16; F07_BLOCKS_PER_FRAME], Vec<LfeExponentPlanBlock>), AnalysisExponentPlanError> {
    let mut work = [0_i16; LFE_EXPONENT_BINS];
    let mut markers_after = [0_i16; F07_BLOCKS_PER_FRAME];
    for block in 0..F07_BLOCKS_PER_FRAME {
        let exponent_input = f08
            .lfe_exponent_inputs()
            .get(block)
            .ok_or(AnalysisExponentPlanError::IncompleteF08Plan)?;
        let mut current = *exponent_input.power_raw_exponents_i16();
        if current.iter().any(|value| !(0..=24).contains(value))
            || work.iter().any(|value| !(0..=31).contains(value))
        {
            return Err(AnalysisExponentPlanError::InvalidPowerRow);
        }
        let mut marker = LFE_MARKER_BEFORE[block];
        refresh_exponent_state(
            &mut current,
            &mut work,
            &mut marker,
            LFE_ACTIVE_MODE_I16,
            REFRESH_SPECIAL_I16,
            REFRESH_BEGIN_I16,
            LFE_REFRESH_END_I16,
        );
        if !matches!(marker, 0 | 1)
            || current.iter().any(|value| !(0..=31).contains(value))
            || work.iter().any(|value| !(0..=31).contains(value))
        {
            return Err(AnalysisExponentPlanError::InvalidRefreshOutput);
        }
        markers_after[block] = marker;
    }

    let mut lfe_slots = [None; F07_BLOCKS_PER_FRAME];
    let mut anchor_block = None;
    for block in 0..F07_BLOCKS_PER_FRAME {
        let strategy = u8::try_from(markers_after[block])
            .map_err(|_| AnalysisExponentPlanError::InvalidRefreshOutput)?;
        if strategy == 0 {
            if anchor_block.is_none() {
                return Err(AnalysisExponentPlanError::MissingF10Anchor);
            }
            continue;
        }
        let end = markers_after
            .iter()
            .enumerate()
            .skip(block + 1)
            .find_map(|(next, marker)| (*marker != 0).then_some(next))
            .unwrap_or(F07_BLOCKS_PER_FRAME);
        let raw_rows = (block..end)
            .map(|source_block| {
                f08.lfe_exponent_inputs()
                    .get(source_block)
                    .map(LfeTransformExponentInputs::primary_raw_exponents_i16)
                    .ok_or(AnalysisExponentPlanError::IncompleteF08Plan)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if raw_rows
            .iter()
            .any(|row| row.iter().any(|value| !(0..=24).contains(value)))
        {
            return Err(AnalysisExponentPlanError::InvalidPrimaryRow);
        }
        let merged_primary =
            std::array::from_fn(|bin| raw_rows.iter().map(|row| row[bin]).min().unwrap_or(0));
        let normalized = normalize_lfe_exponents(&merged_primary, Some(parameters))
            .map_err(AnalysisExponentPlanError::Normalization)?;
        let plan = lfe_plan_block(strategy, &normalized)?;
        for source_block in block..end {
            let mut reuse = plan;
            reuse.strategy = u8::try_from(markers_after[source_block])
                .map_err(|_| AnalysisExponentPlanError::InvalidRefreshOutput)?;
            lfe_slots[source_block] = Some(reuse);
        }
        anchor_block = Some(block);
    }
    let lfe = lfe_slots
        .into_iter()
        .map(|slot| slot.ok_or(AnalysisExponentPlanError::MissingF10Anchor))
        .collect::<Result<Vec<_>, _>>()?;

    Ok((markers_after, lfe))
}

fn fullband_plan_block(
    strategy: u8,
    gain_range: u8,
    normalized: &NormalizedExponentRow,
) -> Result<FullbandExponentPlanBlock, AnalysisExponentPlanError> {
    let base_exponent = u8::try_from(normalized.base_exponent())
        .map_err(|_| AnalysisExponentPlanError::InvalidPrimaryRow)?;
    let decoded_exponents = std::array::from_fn(|index| {
        u8::try_from(normalized.expanded_exponents()[index]).unwrap_or(u8::MAX)
    });
    if decoded_exponents.iter().any(|value| *value > 24) {
        return Err(AnalysisExponentPlanError::InvalidPrimaryRow);
    }
    let packed_groups = pack_exponent_row(normalized.expanded_exponents())?;
    if packed_groups.len() != FBW_PACKED_GROUP_COUNT {
        return Err(AnalysisExponentPlanError::Packing(
            ExponentPackingError::ExponentInputError,
        ));
    }
    Ok(FullbandExponentPlanBlock {
        strategy,
        base_exponent,
        decoded_exponents,
        packed_groups,
        gain_range,
    })
}

fn lfe_plan_block(
    strategy: u8,
    normalized: &NormalizedLfeExponentRow,
) -> Result<LfeExponentPlanBlock, AnalysisExponentPlanError> {
    let base_exponent = u8::try_from(normalized.base_exponent())
        .map_err(|_| AnalysisExponentPlanError::InvalidPrimaryRow)?;
    let decoded_exponents = std::array::from_fn(|index| {
        u8::try_from(normalized.expanded_exponents()[index]).unwrap_or(u8::MAX)
    });
    if decoded_exponents.iter().any(|value| *value > 24) {
        return Err(AnalysisExponentPlanError::InvalidPrimaryRow);
    }
    let packed_groups = pack_exponent_row(normalized.expanded_exponents())?
        .try_into()
        .map_err(|_| {
            AnalysisExponentPlanError::Packing(ExponentPackingError::ExponentInputError)
        })?;
    Ok(LfeExponentPlanBlock {
        strategy,
        base_exponent,
        decoded_exponents,
        packed_groups,
    })
}

fn pack_exponent_row<const BINS: usize>(
    row: &[i16; BINS],
) -> Result<Vec<PackedExponentGroup>, AnalysisExponentPlanError> {
    if BINS < 2 {
        return Err(AnalysisExponentPlanError::InvalidPrimaryRow);
    }
    let mut differences = Vec::with_capacity(BINS - 1);
    let mut previous = row[0];
    for exponent in row.iter().copied().skip(1) {
        let difference = exponent.wrapping_sub(previous);
        differences.push(
            i8::try_from(difference).map_err(|_| AnalysisExponentPlanError::InvalidPrimaryRow)?,
        );
        previous = exponent;
    }
    differences
        .chunks(3)
        .map(|group| {
            pack_exponent_difference_group(group).map_err(AnalysisExponentPlanError::Packing)
        })
        .collect()
}

fn span_for_strategy(strategy: u8) -> Result<u8, AnalysisExponentPlanError> {
    match strategy {
        1 => Ok(1),
        2 => Ok(2),
        3 => Ok(4),
        _ => Err(AnalysisExponentPlanError::InvalidRefreshOutput),
    }
}

#[cfg(test)]
mod tests {
    use super::{AnalysisExponentPlanError, fullband_strategies, span_for_strategy};

    #[test]
    fn reverse_refresh_scan_applies_short_override_after_marker_strategy() {
        let markers = [
            [1, 1, 1, 1, 1],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [1, 1, 1, 1, 1],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ];
        let short_consumed = [
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 1, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ];

        let actual = fullband_strategies(&markers, &short_consumed).unwrap();
        assert_eq!(actual[0], [2, 2, 2, 2, 2]);
        assert_eq!(actual[1], [0, 0, 0, 0, 0]);
        assert_eq!(actual[2], [0, 0, 3, 0, 0]);
        assert_eq!(actual[3], [2, 2, 2, 2, 2]);
        assert_eq!(actual[4], [0, 0, 0, 0, 0]);
        assert_eq!(actual[5], [0, 0, 0, 0, 0]);
    }

    #[test]
    fn f10_anchor_strategies_map_to_exact_spans_and_reuse_has_no_span() {
        assert_eq!(span_for_strategy(1), Ok(1));
        assert_eq!(span_for_strategy(2), Ok(2));
        assert_eq!(span_for_strategy(3), Ok(4));
        assert_eq!(
            span_for_strategy(0),
            Err(AnalysisExponentPlanError::InvalidRefreshOutput)
        );
    }
}
