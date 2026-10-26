// pattern: Functional Core
//! F11.3/F11.4 candidate search over caller-provided per-candidate costs.

const FRAME_BITS_I16: i16 = 20_480;
const FIRST_PARTITION_BITS_I16: i16 = 12_800;
const TAIL_PARTITION_BITS_I16: i16 = 7_680;
const PER_BLOCK_QUOTA_I16: i16 = 3_413;
const GLOBAL_OVERHEAD_I16: i16 = 17;
const MAX_OFFSET: i32 = 671;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConverterSearchInput {
    pub header_cost: i16,
    pub fixed_costs: [i16; 6],
    pub auxiliary_cost: i16,
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

/// Searches the selected F11.4 candidate order with caller-provided per-candidate mantissa costs.
///
/// The callback is invoked for every visited candidate, including candidates above the selected
/// cap. It supplies six mantissa costs; the caller remains responsible for producing them from
/// the candidate's BAP result. Cost inputs satisfy the nonnegative I16 F11.2/F11.3 contract.
/// This function owns no BAP provider, policy state, or product converter transaction.
pub fn search_converter_offset<F>(
    input: &ConverterSearchInput,
    mantissa_costs_for: F,
) -> ConverterSearchResult
where
    F: Fn(i32) -> [i16; 6],
{
    let mut low = 0_i32;
    let mut high = 1_024_i32;
    let mut best = None;
    let mut candidate_order = Vec::with_capacity(10);

    while high - low >= 2 {
        let candidate = (low + high) / 2;
        candidate_order.push(candidate);
        let candidate_mantissa_costs = mantissa_costs_for(candidate);
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

    ConverterSearchResult {
        header_bits: input.header_cost,
        candidate_order,
        best_offset: best.and_then(|value| u16::try_from(value).ok()),
    }
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
