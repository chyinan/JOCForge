// pattern: Functional Core
//! F11's independent converter POWER refresh and frame-pattern projection.

use crate::{
    analysis::{
        exponent_plan::{AnalysisBlockIndex, FullbandLaneIndex},
        owner::AnalysisNumericPlanStage,
        source_frame::SharedAnalysisIdentity,
    },
    codec::exponent_normalization::{
        ExponentNormalizationError, normalize_fullband_exponents, normalize_lfe_exponents_option0,
    },
    codec::{
        bap::{
            BapError, BapFrameInput, BapParameters, BapProfileInput, FULLBAND_BIN_COUNT,
            LFE_BIN_COUNT, allocate_bap_frame_with_parameters,
        },
        converter_header_cost::virtual_header_fixed_bit_cost,
        converter_search::{
            ConverterFrameSearchError, ConverterSearchInput, search_converter_offset_from_bap_frame,
        },
        exponent_refresh_state::refresh_exponent_state,
        selected_frame_trial::SelectedBsiInput,
    },
};

const BLOCK_COUNT: usize = 6;
const FULLBAND_LANE_COUNT: usize = 5;
const FULLBAND_EXPONENT_BINS: usize = 217;
const LFE_EXPONENT_BINS: usize = 7;
const CONVERTER_REFRESH_MODE_I16: i16 = 1;
const CONVERTER_REFRESH_SPECIAL_I16: i16 = 0;
const CONVERTER_REFRESH_BEGIN_I16: i16 = 0;
const CONVERTER_REFRESH_END_I16: i16 = 128;
const CONVERTER_SLOW_DECAY_CODE: u8 = 2;
const CONVERTER_FAST_DECAY_CODE: u8 = 1;
const CONVERTER_SLOW_GAIN_CODE: u8 = 1;
const CONVERTER_FLOOR_CODE: u8 = 7;
const CONVERTER_FAST_GAIN_CODE: u8 = 4;
const CONVERTER_AUXILIARY_ABSENT_COST: i16 = 1;

/// A selected five-bit frame-pattern code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FramePatternCode(u8);

impl FramePatternCode {
    pub const fn new(value: u8) -> Option<Self> {
        if value <= 31 { Some(Self(value)) } else { None }
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

/// F11 main exponent serialization selected from the six per-lane strategies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MainFrameCodeSelection {
    FrameCodes([FramePatternCode; FULLBAND_LANE_COUNT]),
    BlockStrategies,
}

/// Source-local F11 converter refresh, exponent projection and frame-pattern selection.
///
/// This source-local value includes F11's option-0 reuse expansion and converter
/// exponent/code patterns. It is scratch for the per-source budget search, not a
/// persistent F11 owner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConverterSourcePlan {
    source_identity: SharedAnalysisIdentity,
    main_frame_code_selection: MainFrameCodeSelection,
    converter_current_after_i16:
        [[[i16; FULLBAND_EXPONENT_BINS]; FULLBAND_LANE_COUNT]; BLOCK_COUNT],
    converter_work_after_i16: [[[i16; FULLBAND_EXPONENT_BINS]; FULLBAND_LANE_COUNT]; BLOCK_COUNT],
    converter_refresh_marker_after: [[i16; FULLBAND_LANE_COUNT]; BLOCK_COUNT],
    converter_strategy: [[u8; FULLBAND_LANE_COUNT]; BLOCK_COUNT],
    converter_fullband_decoded_exponents:
        [[[u8; FULLBAND_EXPONENT_BINS]; FULLBAND_LANE_COUNT]; BLOCK_COUNT],
    converter_lfe_decoded_exponents: [[u8; LFE_EXPONENT_BINS]; BLOCK_COUNT],
    converter_frame_codes: [FramePatternCode; FULLBAND_LANE_COUNT],
}

/// Complete same-source F11 result before the parent emits its numeric-plan receipt.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct F11ConverterSourceResult {
    source_identity: SharedAnalysisIdentity,
    main_frame_code_selection: MainFrameCodeSelection,
    converter_frame_codes: [FramePatternCode; FULLBAND_LANE_COUNT],
    converter_snr_offset: u16,
    bap_provider_sha256: [u8; 32],
}

impl F11ConverterSourceResult {
    pub const fn source_identity(&self) -> &SharedAnalysisIdentity {
        &self.source_identity
    }

    pub const fn main_frame_code_selection(&self) -> MainFrameCodeSelection {
        self.main_frame_code_selection
    }

    pub const fn converter_frame_codes(&self) -> &[FramePatternCode; FULLBAND_LANE_COUNT] {
        &self.converter_frame_codes
    }

    pub const fn converter_snr_offset(&self) -> u16 {
        self.converter_snr_offset
    }

    pub const fn bap_provider_sha256(&self) -> [u8; 32] {
        self.bap_provider_sha256
    }
}

impl ConverterSourcePlan {
    pub const fn source_identity(&self) -> &SharedAnalysisIdentity {
        &self.source_identity
    }

    pub const fn main_frame_code_selection(&self) -> MainFrameCodeSelection {
        self.main_frame_code_selection
    }

    pub const fn converter_refresh_marker_after(
        &self,
    ) -> &[[i16; FULLBAND_LANE_COUNT]; BLOCK_COUNT] {
        &self.converter_refresh_marker_after
    }

    pub const fn converter_current_after_i16(
        &self,
    ) -> &[[[i16; FULLBAND_EXPONENT_BINS]; FULLBAND_LANE_COUNT]; BLOCK_COUNT] {
        &self.converter_current_after_i16
    }

    pub const fn converter_work_after_i16(
        &self,
    ) -> &[[[i16; FULLBAND_EXPONENT_BINS]; FULLBAND_LANE_COUNT]; BLOCK_COUNT] {
        &self.converter_work_after_i16
    }

    pub const fn converter_strategy(&self) -> &[[u8; FULLBAND_LANE_COUNT]; BLOCK_COUNT] {
        &self.converter_strategy
    }

    pub const fn converter_fullband_decoded_exponents(
        &self,
    ) -> &[[[u8; FULLBAND_EXPONENT_BINS]; FULLBAND_LANE_COUNT]; BLOCK_COUNT] {
        &self.converter_fullband_decoded_exponents
    }

    pub const fn converter_lfe_decoded_exponents(&self) -> &[[u8; LFE_EXPONENT_BINS]; BLOCK_COUNT] {
        &self.converter_lfe_decoded_exponents
    }

    pub const fn converter_frame_codes(&self) -> &[FramePatternCode; FULLBAND_LANE_COUNT] {
        &self.converter_frame_codes
    }
}

/// Errors while projecting one same-source F11 converter refresh and frame pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConverterSourcePlanError {
    SharedAnalysisMissing,
    NumericPlanMissing,
    SharedAnalysisAdvancement,
    IncompleteSourcePlan,
    F07ShortFlagMismatch { block: u8, lane: u8 },
    MissingMainPowerRow { block: u8, lane: u8 },
    InvalidMainPowerRow { block: u8, lane: u8 },
    InvalidConverterRefresh { block: u8, lane: u8 },
    ConverterFramePatternMismatch { lane: u8 },
    MissingF10Anchor { block: u8, lane: u8 },
    InvalidConverterStrategy { block: u8, lane: u8 },
    ExponentNormalization(ExponentNormalizationError),
    DecodedExponentOutOfRange { block: u8, lane: u8 },
    MissingLfeAnchor { block: u8 },
    InvalidLfeStrategy { block: u8 },
    LfeExponentNormalization(ExponentNormalizationError),
    LfeDecodedExponentOutOfRange { block: u8 },
    BsiSourceIdentityMismatch,
    UnsupportedSelectedBsiCosts,
    InvalidSurroundExMode,
    InvalidDbPerBitCode { block: u8 },
    BapProvider(BapError),
    ConverterSearch(ConverterFrameSearchError<BapError>),
    NoFeasibleConverterOffset,
    ConverterOffsetOutOfRange,
    ConverterCostOverflow,
}

/// Produces F11's independent converter refresh decisions and frame-pattern codes.
///
/// The staged source plan binds F07 flags and the main F08-F10 plan to one SharedAnalysisReceipt.
/// Converter F09 uses the saved current row immediately after the first main POWER
/// refresh as `S`, starts independent zero work per lane, and consumes the same F07
/// structure-refresh markers. It never reruns F08/POWER over PRIMARY data.
pub fn prepare_converter_source_plan(
    numeric_plan_stage: &AnalysisNumericPlanStage,
) -> Result<ConverterSourcePlan, ConverterSourcePlanError> {
    let shared_analysis = numeric_plan_stage
        .shared_analysis()
        .ok_or(ConverterSourcePlanError::SharedAnalysisMissing)?;
    if shared_analysis.advancement_count() != 1 {
        return Err(ConverterSourcePlanError::SharedAnalysisAdvancement);
    }

    let plan = numeric_plan_stage
        .plan()
        .ok_or(ConverterSourcePlanError::NumericPlanMissing)?;
    let exponent_plan = plan.exponents();
    if !plan.f08().is_six_block_complete() || !exponent_plan.is_six_block_complete() {
        return Err(ConverterSourcePlanError::IncompleteSourcePlan);
    }

    let f07_output = shared_analysis.f07_proposal().output();
    let mut converter_current_after_i16 =
        [[[0_i16; FULLBAND_EXPONENT_BINS]; FULLBAND_LANE_COUNT]; BLOCK_COUNT];
    let mut converter_work_after_i16 =
        [[[0_i16; FULLBAND_EXPONENT_BINS]; FULLBAND_LANE_COUNT]; BLOCK_COUNT];
    let mut converter_refresh_marker_after = [[0_i16; FULLBAND_LANE_COUNT]; BLOCK_COUNT];
    let mut converter_work_by_lane = [[0_i16; FULLBAND_EXPONENT_BINS]; FULLBAND_LANE_COUNT];

    for block in 0..BLOCK_COUNT {
        for lane in 0..FULLBAND_LANE_COUNT {
            let block_index = AnalysisBlockIndex::new(u8::try_from(block).unwrap_or(u8::MAX));
            let lane_index = FullbandLaneIndex::new(u8::try_from(lane).unwrap_or(u8::MAX));
            let f07_short = f07_output.short_consumed()[block][lane];
            if f07_short > 1 || plan.short_flags()[block][lane] != (f07_short == 1) {
                return Err(ConverterSourcePlanError::F07ShortFlagMismatch {
                    block: block_index.value(),
                    lane: lane_index.value(),
                });
            }

            let saved_s = exponent_plan
                .first_power_refresh_current_after_i16(block_index, lane_index)
                .ok_or(ConverterSourcePlanError::MissingMainPowerRow {
                    block: block_index.value(),
                    lane: lane_index.value(),
                })?;
            if saved_s.iter().any(|value| !(0..=31).contains(value)) {
                return Err(ConverterSourcePlanError::InvalidMainPowerRow {
                    block: block_index.value(),
                    lane: lane_index.value(),
                });
            }

            let mut current = *saved_s;
            let work = &mut converter_work_by_lane[lane];
            let mut marker = i16::from(f07_output.structure_refresh()[block][lane]);
            refresh_exponent_state(
                &mut current,
                work,
                &mut marker,
                CONVERTER_REFRESH_MODE_I16,
                CONVERTER_REFRESH_SPECIAL_I16,
                CONVERTER_REFRESH_BEGIN_I16,
                CONVERTER_REFRESH_END_I16,
            );
            if !matches!(marker, 0 | 1)
                || current.iter().any(|value| !(0..=31).contains(value))
                || work.iter().any(|value| !(0..=31).contains(value))
            {
                return Err(ConverterSourcePlanError::InvalidConverterRefresh {
                    block: block_index.value(),
                    lane: lane_index.value(),
                });
            }
            converter_current_after_i16[block][lane] = current;
            converter_work_after_i16[block][lane] = *work;
            converter_refresh_marker_after[block][lane] = marker;
        }
    }

    let converter_strategy =
        derive_converter_strategies(&converter_refresh_marker_after, plan.short_flags())?;
    let converter_fullband_decoded_exponents =
        expand_fullband_converter_exponents(exponent_plan, &converter_strategy)?;
    let converter_lfe_decoded_exponents = expand_lfe_converter_exponents(exponent_plan)?;

    let mut converter_frame_codes = [FramePatternCode(0); FULLBAND_LANE_COUNT];
    for (lane, frame_code) in converter_frame_codes.iter_mut().enumerate() {
        let strategy_row = std::array::from_fn(|block| converter_strategy[block][lane]);
        *frame_code = frame_code_for_strategy_row(strategy_row).ok_or(
            ConverterSourcePlanError::ConverterFramePatternMismatch {
                lane: u8::try_from(lane).unwrap_or(u8::MAX),
            },
        )?;
    }

    let main_frame_code_selection = select_main_frame_codes(exponent_plan.fullband_strategy());

    Ok(ConverterSourcePlan {
        source_identity: shared_analysis.identity().clone(),
        main_frame_code_selection,
        converter_current_after_i16,
        converter_work_after_i16,
        converter_refresh_marker_after,
        converter_strategy,
        converter_fullband_decoded_exponents,
        converter_lfe_decoded_exponents,
        converter_frame_codes,
    })
}

/// Completes the source-local F11 offset search using same-source BSI and BAP inputs.
///
/// F11 runs once for the source. Only the callback passed to the existing converter
/// search allocates fresh BAP for each candidate; it never reruns F07/F08/F09/F10.
pub fn prepare_f11_converter_source_result(
    numeric_plan_stage: &AnalysisNumericPlanStage,
    bsi: &SelectedBsiInput,
    bap_provider: &BapParameters,
) -> Result<F11ConverterSourceResult, ConverterSourcePlanError> {
    let shared_analysis = numeric_plan_stage
        .shared_analysis()
        .ok_or(ConverterSourcePlanError::SharedAnalysisMissing)?;
    let bsi_costs = bsi.converter_cost_projection();
    if bsi_costs.source_identity() != shared_analysis.identity() {
        return Err(ConverterSourcePlanError::BsiSourceIdentityMismatch);
    }
    if !bsi_costs.rf_compression_present() || bsi_costs.line_drc_presence().iter().any(|v| !v) {
        return Err(ConverterSourcePlanError::UnsupportedSelectedBsiCosts);
    }
    let surround_ex_mode = bsi_costs.surround_ex_mode();
    if !matches!(surround_ex_mode, 0 | 2) {
        return Err(ConverterSourcePlanError::InvalidSurroundExMode);
    }

    let numeric_plan = numeric_plan_stage
        .plan()
        .ok_or(ConverterSourcePlanError::NumericPlanMissing)?;
    let converter_source_plan = prepare_converter_source_plan(numeric_plan_stage)?;
    let lfe_strategy_by_block = std::array::from_fn(|block| {
        let block_index = AnalysisBlockIndex::new(u8::try_from(block).unwrap_or(u8::MAX));
        numeric_plan.exponents().lfe_block(block_index).map_or(
            u8::MAX,
            crate::analysis::exponent_plan::LfeExponentPlanBlock::strategy,
        )
    });
    let fixed_costs = converter_fixed_costs(
        &converter_source_plan.converter_strategy,
        lfe_strategy_by_block,
        *numeric_plan.gain_db().db_per_bit_codes_by_block(),
    )?;
    let header_cost = i16::try_from(virtual_header_fixed_bit_cost(surround_ex_mode))
        .map_err(|_| ConverterSourcePlanError::ConverterCostOverflow)?;
    let search_input = ConverterSearchInput {
        header_cost,
        fixed_costs,
        auxiliary_cost: CONVERTER_AUXILIARY_ABSENT_COST,
    };
    let decoded_exponents_by_block: [[&[u8]; 6]; BLOCK_COUNT] = std::array::from_fn(|block| {
        [
            &converter_source_plan.converter_fullband_decoded_exponents[block][0][..],
            &converter_source_plan.converter_fullband_decoded_exponents[block][1][..],
            &converter_source_plan.converter_fullband_decoded_exponents[block][2][..],
            &converter_source_plan.converter_fullband_decoded_exponents[block][3][..],
            &converter_source_plan.converter_fullband_decoded_exponents[block][4][..],
            &converter_source_plan.converter_lfe_decoded_exponents[block][..],
        ]
    });
    let db_per_bit_codes = *numeric_plan.gain_db().db_per_bit_codes_by_block();
    if let Some(block) = db_per_bit_codes.iter().position(|code| *code > 3) {
        return Err(ConverterSourcePlanError::InvalidDbPerBitCode {
            block: u8::try_from(block).unwrap_or(u8::MAX),
        });
    }
    let search_result = search_converter_offset_from_bap_frame(&search_input, |candidate| {
        let candidate = u16::try_from(candidate).map_err(|_| BapError::InputError)?;
        let input = BapFrameInput {
            decoded_exponents_by_block: &decoded_exponents_by_block,
            end_exclusive_by_channel: [
                FULLBAND_BIN_COUNT,
                FULLBAND_BIN_COUNT,
                FULLBAND_BIN_COUNT,
                FULLBAND_BIN_COUNT,
                FULLBAND_BIN_COUNT,
                LFE_BIN_COUNT,
            ],
            candidate,
            slow_decay_code: CONVERTER_SLOW_DECAY_CODE,
            fast_decay_code: CONVERTER_FAST_DECAY_CODE,
            slow_gain_code: CONVERTER_SLOW_GAIN_CODE,
            db_per_bit_codes,
            floor_code: CONVERTER_FLOOR_CODE,
            fast_gain_code: CONVERTER_FAST_GAIN_CODE,
            caller_all_active_fine_zero_by_block: [candidate.trailing_zeros() >= 4; BLOCK_COUNT],
            profile: BapProfileInput::selected(),
        };
        allocate_bap_frame_with_parameters(bap_provider, &input)
    })
    .map_err(ConverterSourcePlanError::ConverterSearch)?;
    let converter_snr_offset = search_result
        .best_offset()
        .ok_or(ConverterSourcePlanError::NoFeasibleConverterOffset)?;
    if converter_snr_offset > 671 {
        return Err(ConverterSourcePlanError::ConverterOffsetOutOfRange);
    }

    Ok(F11ConverterSourceResult {
        source_identity: converter_source_plan.source_identity.clone(),
        main_frame_code_selection: converter_source_plan.main_frame_code_selection,
        converter_frame_codes: converter_source_plan.converter_frame_codes,
        converter_snr_offset,
        bap_provider_sha256: bap_provider.provider_sha256(),
    })
}

fn converter_fixed_costs(
    converter_strategy: &[[u8; FULLBAND_LANE_COUNT]; BLOCK_COUNT],
    lfe_strategy_by_block: [u8; BLOCK_COUNT],
    db_codes: [u8; BLOCK_COUNT],
) -> Result<[i16; BLOCK_COUNT], ConverterSourcePlanError> {
    if db_codes.iter().any(|code| *code > 3) {
        let block = db_codes.iter().position(|code| *code > 3).unwrap_or(0);
        return Err(ConverterSourcePlanError::InvalidDbPerBitCode {
            block: u8::try_from(block).unwrap_or(u8::MAX),
        });
    }
    let mut costs = [0_i16; BLOCK_COUNT];
    for block in 0..BLOCK_COUNT {
        let mut cost = 10_i16
            .checked_add(9)
            .and_then(|value| value.checked_add(if block == 0 { 2 } else { 1 }))
            .and_then(|value| value.checked_add(11))
            .and_then(|value| value.checked_add(4))
            .ok_or(ConverterSourcePlanError::ConverterCostOverflow)?;
        for (lane, strategy) in converter_strategy[block].iter().copied().enumerate() {
            if strategy == 0 {
                continue;
            }
            let groups = converter_group_count(strategy).ok_or(
                ConverterSourcePlanError::InvalidConverterStrategy {
                    block: u8::try_from(block).unwrap_or(u8::MAX),
                    lane: u8::try_from(lane).unwrap_or(u8::MAX),
                },
            )?;
            let strategy_cost = 12_i16
                .checked_add(
                    groups
                        .checked_mul(7)
                        .ok_or(ConverterSourcePlanError::ConverterCostOverflow)?,
                )
                .ok_or(ConverterSourcePlanError::ConverterCostOverflow)?;
            cost = cost
                .checked_add(strategy_cost)
                .ok_or(ConverterSourcePlanError::ConverterCostOverflow)?;
        }
        let lfe_strategy = lfe_strategy_by_block[block];
        if lfe_strategy > 1 {
            return Err(ConverterSourcePlanError::InvalidLfeStrategy {
                block: u8::try_from(block).unwrap_or(u8::MAX),
            });
        }
        if lfe_strategy == 1 {
            cost = cost
                .checked_add(18)
                .ok_or(ConverterSourcePlanError::ConverterCostOverflow)?;
        }
        let ba_record_update = block == 0 || db_codes[block] != db_codes[block - 1];
        if ba_record_update {
            cost = cost
                .checked_add(11)
                .ok_or(ConverterSourcePlanError::ConverterCostOverflow)?;
        }
        if block == 0 {
            cost = cost
                .checked_add(48)
                .ok_or(ConverterSourcePlanError::ConverterCostOverflow)?;
        }
        costs[block] = cost;
    }
    Ok(costs)
}

const fn converter_group_count(strategy: u8) -> Option<i16> {
    match strategy {
        1 => Some(72),
        2 => Some(36),
        3 => Some(18),
        _ => None,
    }
}

fn expand_fullband_converter_exponents(
    main_plan: &crate::analysis::exponent_plan::AnalysisExponentPlan,
    converter_strategy: &[[u8; FULLBAND_LANE_COUNT]; BLOCK_COUNT],
) -> Result<
    [[[u8; FULLBAND_EXPONENT_BINS]; FULLBAND_LANE_COUNT]; BLOCK_COUNT],
    ConverterSourcePlanError,
> {
    let mut output = [[[0_u8; FULLBAND_EXPONENT_BINS]; FULLBAND_LANE_COUNT]; BLOCK_COUNT];
    for lane in 0..FULLBAND_LANE_COUNT {
        let mut block = 0;
        let mut anchor_seen = false;
        while block < BLOCK_COUNT {
            let strategy = converter_strategy[block][lane];
            if strategy == 0 {
                if !anchor_seen {
                    return Err(ConverterSourcePlanError::MissingF10Anchor {
                        block: u8::try_from(block).unwrap_or(u8::MAX),
                        lane: u8::try_from(lane).unwrap_or(u8::MAX),
                    });
                }
                block += 1;
                continue;
            }
            let span = frequency_span_for_strategy(strategy).ok_or(
                ConverterSourcePlanError::InvalidConverterStrategy {
                    block: u8::try_from(block).unwrap_or(u8::MAX),
                    lane: u8::try_from(lane).unwrap_or(u8::MAX),
                },
            )?;
            let end = (block + 1..BLOCK_COUNT)
                .find(|candidate| converter_strategy[*candidate][lane] != 0)
                .unwrap_or(BLOCK_COUNT);
            let mut merged = [i16::MAX; FULLBAND_EXPONENT_BINS];
            for source_block in block..end {
                let block_index =
                    AnalysisBlockIndex::new(u8::try_from(source_block).unwrap_or(u8::MAX));
                let lane_index = FullbandLaneIndex::new(u8::try_from(lane).unwrap_or(u8::MAX));
                let main = main_plan.fullband_block(block_index, lane_index).ok_or(
                    ConverterSourcePlanError::MissingF10Anchor {
                        block: block_index.value(),
                        lane: lane_index.value(),
                    },
                )?;
                for (destination, exponent) in merged
                    .iter_mut()
                    .zip(main.decoded_exponents().iter().copied())
                {
                    if exponent > 24 {
                        return Err(ConverterSourcePlanError::DecodedExponentOutOfRange {
                            block: block_index.value(),
                            lane: lane_index.value(),
                        });
                    }
                    *destination = (*destination).min(i16::from(exponent));
                }
            }
            let normalized = normalize_fullband_exponents(&merged, span, 0, 0, None)
                .map_err(ConverterSourcePlanError::ExponentNormalization)?;
            let mut decoded = [0_u8; FULLBAND_EXPONENT_BINS];
            for (destination, exponent) in decoded
                .iter_mut()
                .zip(normalized.expanded_exponents().iter().copied())
            {
                *destination = u8::try_from(exponent).map_err(|_| {
                    ConverterSourcePlanError::DecodedExponentOutOfRange {
                        block: u8::try_from(block).unwrap_or(u8::MAX),
                        lane: u8::try_from(lane).unwrap_or(u8::MAX),
                    }
                })?;
                if *destination > 24 {
                    return Err(ConverterSourcePlanError::DecodedExponentOutOfRange {
                        block: u8::try_from(block).unwrap_or(u8::MAX),
                        lane: u8::try_from(lane).unwrap_or(u8::MAX),
                    });
                }
            }
            for destination in output.iter_mut().take(end).skip(block) {
                destination[lane] = decoded;
            }
            anchor_seen = true;
            block = end;
        }
    }
    Ok(output)
}

fn expand_lfe_converter_exponents(
    main_plan: &crate::analysis::exponent_plan::AnalysisExponentPlan,
) -> Result<[[u8; LFE_EXPONENT_BINS]; BLOCK_COUNT], ConverterSourcePlanError> {
    let mut strategies = [0_u8; BLOCK_COUNT];
    let mut main_rows = [[0_u8; LFE_EXPONENT_BINS]; BLOCK_COUNT];
    for block in 0..BLOCK_COUNT {
        let block_index = AnalysisBlockIndex::new(u8::try_from(block).unwrap_or(u8::MAX));
        let main =
            main_plan
                .lfe_block(block_index)
                .ok_or(ConverterSourcePlanError::MissingLfeAnchor {
                    block: block_index.value(),
                })?;
        strategies[block] = main.strategy();
        main_rows[block] = main.decoded_exponents();
        if strategies[block] > 1 {
            return Err(ConverterSourcePlanError::InvalidLfeStrategy {
                block: block_index.value(),
            });
        }
        if main_rows[block].iter().any(|exponent| *exponent > 24) {
            return Err(ConverterSourcePlanError::LfeDecodedExponentOutOfRange {
                block: block_index.value(),
            });
        }
    }
    if strategies[0] != 1 {
        return Err(ConverterSourcePlanError::MissingLfeAnchor { block: 0 });
    }

    let mut output = [[0_u8; LFE_EXPONENT_BINS]; BLOCK_COUNT];
    let mut block = 0;
    let mut anchor_seen = false;
    while block < BLOCK_COUNT {
        let strategy = strategies[block];
        if strategy == 0 {
            if !anchor_seen {
                return Err(ConverterSourcePlanError::MissingLfeAnchor {
                    block: u8::try_from(block).unwrap_or(u8::MAX),
                });
            }
            block += 1;
            continue;
        }
        let end = (block + 1..BLOCK_COUNT)
            .find(|candidate| strategies[*candidate] != 0)
            .unwrap_or(BLOCK_COUNT);
        let mut merged = [i16::MAX; LFE_EXPONENT_BINS];
        for row in main_rows.iter().take(end).skip(block) {
            for (destination, exponent) in merged.iter_mut().zip(row.iter().copied()) {
                *destination = (*destination).min(i16::from(exponent));
            }
        }
        let normalized = normalize_lfe_exponents_option0(&merged)
            .map_err(ConverterSourcePlanError::LfeExponentNormalization)?;
        let mut decoded = [0_u8; LFE_EXPONENT_BINS];
        for (destination, exponent) in decoded
            .iter_mut()
            .zip(normalized.expanded_exponents().iter().copied())
        {
            *destination = u8::try_from(exponent).map_err(|_| {
                ConverterSourcePlanError::LfeDecodedExponentOutOfRange {
                    block: u8::try_from(block).unwrap_or(u8::MAX),
                }
            })?;
            if *destination > 24 {
                return Err(ConverterSourcePlanError::LfeDecodedExponentOutOfRange {
                    block: u8::try_from(block).unwrap_or(u8::MAX),
                });
            }
        }
        for destination in output.iter_mut().take(end).skip(block) {
            *destination = decoded;
        }
        anchor_seen = true;
        block = end;
    }
    Ok(output)
}

const fn frequency_span_for_strategy(strategy: u8) -> Option<u8> {
    match strategy {
        1 => Some(1),
        2 => Some(2),
        3 => Some(4),
        _ => None,
    }
}

fn derive_converter_strategies(
    markers_after: &[[i16; FULLBAND_LANE_COUNT]; BLOCK_COUNT],
    short_flags: &[[bool; FULLBAND_LANE_COUNT]; BLOCK_COUNT],
) -> Result<[[u8; FULLBAND_LANE_COUNT]; BLOCK_COUNT], ConverterSourcePlanError> {
    let mut strategies = [[0_u8; FULLBAND_LANE_COUNT]; BLOCK_COUNT];
    for lane in 0..FULLBAND_LANE_COUNT {
        let mut distance = 0_usize;
        for block in (0..BLOCK_COUNT).rev() {
            let marker = markers_after[block][lane];
            if !matches!(marker, 0 | 1) {
                return Err(ConverterSourcePlanError::InvalidConverterRefresh {
                    block: u8::try_from(block).unwrap_or(u8::MAX),
                    lane: u8::try_from(lane).unwrap_or(u8::MAX),
                });
            }
            let mut strategy = if marker == 0 {
                distance += 1;
                0
            } else {
                let selected = if distance < 2 {
                    3
                } else if distance < 4 {
                    2
                } else {
                    1
                };
                distance = 0;
                selected
            };
            if short_flags[block][lane] {
                strategy = 3;
            }
            strategies[block][lane] = strategy;
        }
    }
    Ok(strategies)
}

/// Maps a complete six-block U2 strategy row to the first matching U5 code.
pub fn frame_code_for_strategy_row(strategy_row: [u8; BLOCK_COUNT]) -> Option<FramePatternCode> {
    if strategy_row.iter().any(|strategy| *strategy > 3) {
        return None;
    }
    (0_u8..32).find_map(|value| {
        let code = FramePatternCode::new(value)?;
        (strategy_row_for_code(code) == strategy_row).then_some(code)
    })
}

/// Reconstructs the approved equivalent Table E.1.9 row for one U5 code.
pub fn strategy_row_for_code(code: FramePatternCode) -> [u8; BLOCK_COUNT] {
    let mut row = [0_u8; BLOCK_COUNT];
    let mut segment_start = 0_usize;
    for boundary in 0..BLOCK_COUNT - 1 {
        let bit = u32::try_from(4 - boundary).unwrap_or(0);
        if code.value() & (1_u8 << bit) != 0 {
            write_segment_strategy(&mut row, segment_start, boundary + 1);
            segment_start = boundary + 1;
        }
    }
    write_segment_strategy(&mut row, segment_start, BLOCK_COUNT);
    row
}

fn write_segment_strategy(row: &mut [u8; BLOCK_COUNT], start: usize, end: usize) {
    let strategy = match end - start {
        1 => 3,
        2 | 3 => 2,
        4..=6 => 1,
        _ => 0,
    };
    row[start] = strategy;
}

fn select_main_frame_codes(
    strategy_by_block: &[[u8; FULLBAND_LANE_COUNT]; BLOCK_COUNT],
) -> MainFrameCodeSelection {
    let mut codes = [FramePatternCode(0); FULLBAND_LANE_COUNT];
    for lane in 0..FULLBAND_LANE_COUNT {
        let strategy_row = std::array::from_fn(|block| strategy_by_block[block][lane]);
        let Some(code) = frame_code_for_strategy_row(strategy_row) else {
            return MainFrameCodeSelection::BlockStrategies;
        };
        codes[lane] = code;
    }
    MainFrameCodeSelection::FrameCodes(codes)
}

#[cfg(test)]
mod tests {
    use super::{
        FramePatternCode, MainFrameCodeSelection, converter_fixed_costs,
        frame_code_for_strategy_row, select_main_frame_codes, strategy_row_for_code,
    };
    use crate::codec::exponent_normalization::normalize_lfe_exponents_option0;
    use sha2::{Digest, Sha256};
    use std::collections::BTreeSet;

    #[test]
    fn derived_frame_pattern_rows_match_the_approved_projection_identity() {
        let mut projection = Vec::with_capacity(32 * 6);
        let mut unique_rows = BTreeSet::new();
        for value in 0_u8..32 {
            let code = FramePatternCode::new(value).unwrap();
            let row = strategy_row_for_code(code);
            assert!(unique_rows.insert(row));
            assert_eq!(frame_code_for_strategy_row(row), Some(code));
            projection.extend(row);
        }
        let digest = Sha256::digest(&projection);
        assert_eq!(
            hex_lower(&digest),
            "acfcaf98c12fa9688ef75943537a2f2f0a97a41ab75aa342468bd649b783d6bc"
        );
    }

    #[test]
    fn valid_nonmatching_strategy_row_has_no_frame_code() {
        assert_eq!(frame_code_for_strategy_row([1, 1, 0, 0, 0, 0]), None);
    }

    #[test]
    fn frame_pattern_code_rejects_values_outside_u5() {
        assert_eq!(FramePatternCode::new(31).unwrap().value(), 31);
        assert_eq!(FramePatternCode::new(32), None);
    }

    #[test]
    fn converter_lfe_option0_applies_only_the_first_difference_pass() {
        let raw = [0_i16, 3, 3, 3, 3, 3, 3];
        let normalized = normalize_lfe_exponents_option0(&raw).unwrap();
        assert_eq!(normalized.base_exponent(), 0);
        assert_eq!(normalized.expanded_exponents(), &[0, 2, 3, 3, 3, 3, 3]);
    }

    #[test]
    fn main_exponent_syntax_uses_block_strategy_for_a_nonmatching_row() {
        let lanes = [
            [1_u8; 5], [1_u8; 5], [0_u8; 5], [0_u8; 5], [0_u8; 5], [0_u8; 5],
        ];
        assert_eq!(
            select_main_frame_codes(&lanes),
            MainFrameCodeSelection::BlockStrategies
        );
    }

    #[test]
    fn converter_fixed_costs_follow_the_selected_f11_fields() {
        let mut strategy = [[0_u8; 5]; 6];
        strategy[1] = [1; 5];
        strategy[2][0] = 2;
        let lfe_strategy = [1, 0, 0, 0, 0, 0];
        let db_codes = [0, 1, 1, 1, 1, 1];
        assert_eq!(
            converter_fixed_costs(&strategy, lfe_strategy, db_codes).unwrap(),
            [113, 2_626, 299, 35, 35, 35]
        );
    }

    fn hex_lower(bytes: &[u8]) -> String {
        let mut output = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            use std::fmt::Write as _;
            write!(&mut output, "{byte:02x}").unwrap();
        }
        output
    }
}
