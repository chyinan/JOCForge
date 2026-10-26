// pattern: Functional Core
//! F11.3/F11.4 candidate search over caller-provided per-candidate costs.

use std::convert::Infallible;

use super::bap::{BapBlock, BapFrame};

const FRAME_BITS_I16: i16 = 20_480;
const FIRST_PARTITION_BITS_I16: i16 = 12_800;
const TAIL_PARTITION_BITS_I16: i16 = 7_680;
const PER_BLOCK_QUOTA_I16: i16 = 3_413;
const GLOBAL_OVERHEAD_I16: i16 = 17;
const MAX_OFFSET: i32 = 671;
const WIDTH_BY_BAP: [i16; 16] = [0, 5, 7, 3, 7, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConverterSearchInput {
    pub header_cost: i16,
    pub fixed_costs: [i16; 6],
    pub auxiliary_cost: i16,
}

/// Errors while evaluating a converter search from actual per-candidate BAP rows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConverterBapSearchError<E> {
    InvalidCostInput,
    CandidateBap(E),
}

/// Errors while joining a candidate-tagged F12A frame to converter search.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConverterFrameSearchError<E> {
    InvalidCostInput,
    CandidateBap(E),
    CandidateMismatch { requested: i32, received: u16 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConverterSearchResult {
    header_bits: i16,
    candidate_order: Vec<i32>,
    best_offset: Option<u16>,
}

impl ConverterSearchResult {
    pub const fn header_bits(&self) -> i16 {
        self.header_bits
    }

    pub fn candidate_order(&self) -> &[i32] {
        &self.candidate_order
    }

    pub const fn best_offset(&self) -> Option<u16> {
        self.best_offset
    }
}

/// Computes the six virtual mantissa costs from actual BAP rows.
///
/// BAP1 and BAP2 items are grouped in threes and BAP4 items in pairs across
/// the five full-band channels and LFE within each block. Each block starts
/// with fresh group counts. Other BAP codes use the selected F12S width vector.
/// The accumulator is W16; no coefficient quantization or sentinel judgment
/// is performed here.
pub fn converter_mantissa_costs(blocks: &[BapBlock; 6]) -> [i16; 6] {
    std::array::from_fn(|block_index| block_mantissa_cost(&blocks[block_index]))
}

/// Searches the selected F11.4 candidate order with caller-provided per-candidate mantissa costs.
///
/// The callback is invoked for every visited candidate, including candidates above the selected
/// cap. It supplies six mantissa costs; the caller remains responsible for producing them from
/// the candidate's BAP result. Cost inputs satisfy the nonnegative I16 F11.2/F11.3 contract.
/// This function owns no BAP provider, policy state, or product converter transaction.
pub fn search_converter_offset<F>(
    input: &ConverterSearchInput,
    mut mantissa_costs_for: F,
) -> ConverterSearchResult
where
    F: FnMut(i32) -> [i16; 6],
{
    match search_candidates(input, |candidate| {
        Ok::<[i16; 6], Infallible>(mantissa_costs_for(candidate))
    }) {
        Ok(result) => result,
        Err(error) => match error {},
    }
}

/// Searches offsets by producing fresh BAP for each candidate and deriving its costs.
///
/// The callback runs for every visited candidate, including candidates above
/// the cap. Its BAP results are consumed immediately and are not reused for a
/// different candidate, block, or channel. A callback failure is fatal to this
/// search call. The no-best case checks the specified zero-mantissa cost
/// case without invoking the BAP callback at offset zero.
pub fn search_converter_offset_from_bap<F, E>(
    input: &ConverterSearchInput,
    mut bap_blocks_for: F,
) -> Result<ConverterSearchResult, ConverterBapSearchError<E>>
where
    F: FnMut(i32) -> Result<[BapBlock; 6], E>,
{
    if input.header_cost < 0
        || input.auxiliary_cost < 0
        || input.fixed_costs.iter().any(|cost| *cost < 0)
    {
        return Err(ConverterBapSearchError::InvalidCostInput);
    }

    search_candidates(input, |candidate| {
        let blocks = bap_blocks_for(candidate).map_err(ConverterBapSearchError::CandidateBap)?;
        Ok(converter_mantissa_costs(&blocks))
    })
}

/// Searches using candidate-tagged F12A frames and rejects a mismatched q join.
///
/// This adapter keeps the lower-level block-array callback API available while
/// preserving the F12A candidate key at the frame boundary. Source generation
/// identity remains caller-owned.
pub fn search_converter_offset_from_bap_frame<F, E>(
    input: &ConverterSearchInput,
    mut bap_frame_for: F,
) -> Result<ConverterSearchResult, ConverterFrameSearchError<E>>
where
    F: FnMut(i32) -> Result<BapFrame, E>,
{
    if input.header_cost < 0
        || input.auxiliary_cost < 0
        || input.fixed_costs.iter().any(|cost| *cost < 0)
    {
        return Err(ConverterFrameSearchError::InvalidCostInput);
    }

    search_candidates(input, |candidate| {
        let frame = bap_frame_for(candidate).map_err(ConverterFrameSearchError::CandidateBap)?;
        if i32::from(frame.candidate_q()) != candidate {
            return Err(ConverterFrameSearchError::CandidateMismatch {
                requested: candidate,
                received: frame.candidate_q(),
            });
        }
        Ok(converter_mantissa_costs(frame.blocks()))
    })
}

fn search_candidates<F, E>(
    input: &ConverterSearchInput,
    mut mantissa_costs_for: F,
) -> Result<ConverterSearchResult, E>
where
    F: FnMut(i32) -> Result<[i16; 6], E>,
{
    let mut low = 0_i32;
    let mut high = 1_024_i32;
    let mut best = None;
    let mut candidate_order = Vec::with_capacity(10);

    while high - low >= 2 {
        let candidate = (low + high) / 2;
        candidate_order.push(candidate);
        let candidate_mantissa_costs = mantissa_costs_for(candidate)?;
        let feasible = converter_frame_feasible(
            input.header_cost,
            &input.fixed_costs,
            input.auxiliary_cost,
            &candidate_mantissa_costs,
        );
        if candidate <= MAX_OFFSET && feasible {
            best = Some(candidate);
            low = candidate;
        } else {
            high = candidate;
        }
    }

    if best.is_none()
        && converter_frame_feasible(
            input.header_cost,
            &input.fixed_costs,
            input.auxiliary_cost,
            &[0_i16; 6],
        )
    {
        best = Some(0);
    }

    Ok(ConverterSearchResult {
        header_bits: input.header_cost,
        candidate_order,
        best_offset: best.and_then(|value| u16::try_from(value).ok()),
    })
}

fn block_mantissa_cost(block: &BapBlock) -> i16 {
    let mut counts = [0_u16; WIDTH_BY_BAP.len()];
    for code in block
        .fullband_rows()
        .iter()
        .flatten()
        .chain(block.lfe_row().iter())
    {
        let count = &mut counts[usize::from(*code)];
        *count = count.wrapping_add(1);
    }

    let mut cost = 0_i16;
    for (code, count) in counts.into_iter().enumerate() {
        let contribution = match code {
            1 => grouped_width(count, 3, 5),
            2 => grouped_width(count, 3, 7),
            4 => grouped_width(count, 2, 7),
            _ => i16::try_from(count)
                .expect("a selected block has at most 1,092 bins")
                .wrapping_mul(WIDTH_BY_BAP[code]),
        };
        cost = cost.wrapping_add(contribution);
    }
    debug_assert!(cost >= 0, "a six-row BAP cost fits in signed W16");
    cost
}

fn grouped_width(count: u16, group_size: u16, field_width: i16) -> i16 {
    let group_count = count.wrapping_add(group_size - 1) / group_size;
    i16::try_from(group_count)
        .expect("a selected block has at most 1,092 bins")
        .wrapping_mul(field_width)
}

fn converter_frame_feasible(
    header_bits: i16,
    fixed_costs: &[i16; 6],
    auxiliary_bits: i16,
    mantissa_costs: &[i16; 6],
) -> bool {
    let fixed_sum = fixed_costs.iter().copied().fold(0_i16, i16::wrapping_add);
    let mantissa_sum = mantissa_costs
        .iter()
        .copied()
        .fold(0_i16, i16::wrapping_add);
    let initial_remaining = FRAME_BITS_I16
        .wrapping_sub(header_bits)
        .wrapping_sub(fixed_sum)
        .wrapping_sub(mantissa_sum)
        .wrapping_sub(auxiliary_bits)
        .wrapping_sub(GLOBAL_OVERHEAD_I16);
    if initial_remaining < 0 {
        return false;
    }

    let mut cumulative_capacity = 0_i16;
    let mut spent = header_bits;
    let mut remaining = initial_remaining;
    let mut stuffing_cost = [0_i16; 6];
    for block in 0..6 {
        cumulative_capacity = cumulative_capacity.wrapping_add(PER_BLOCK_QUOTA_I16);
        spent = spent
            .wrapping_add(fixed_costs[block])
            .wrapping_add(mantissa_costs[block]);
        let available = cumulative_capacity.wrapping_sub(spent).min(remaining);
        let cost = if available >= 9 {
            let groups = ((available - 9) / 8).min(511);
            groups.wrapping_mul(8).wrapping_add(9)
        } else {
            0_i16
        };
        stuffing_cost[block] = cost;
        spent = spent.wrapping_add(cost);
        remaining = remaining.wrapping_sub(cost);
    }

    let first_partition_remaining = FIRST_PARTITION_BITS_I16
        .wrapping_sub(stuffing_cost[0])
        .wrapping_sub(stuffing_cost[1])
        .wrapping_sub(mantissa_costs[0])
        .wrapping_sub(mantissa_costs[1])
        .wrapping_sub(header_bits)
        .wrapping_sub(fixed_costs[0])
        .wrapping_sub(fixed_costs[1]);
    let tail_partition_remaining = TAIL_PARTITION_BITS_I16
        .wrapping_sub(remaining)
        .wrapping_sub(auxiliary_bits)
        .wrapping_sub(mantissa_costs[5])
        .wrapping_sub(GLOBAL_OVERHEAD_I16);
    initial_remaining
        .min(first_partition_remaining)
        .min(tail_partition_remaining)
        >= 0
}
