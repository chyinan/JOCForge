// pattern: Functional Core
//! Routes five secondary block streams into the selected six-row carrier shape.

const LOGICAL_LANE_COUNT: usize = 5;
const OUTPUT_ROW_COUNT: usize = 6;
const BLOCK_COUNT: usize = 6;
const BLOCK_SAMPLES: usize = 256;
const CARRIER_SAMPLES: usize = 1_536;
const OUTPUT_ROW_FOR_LOGICAL_LANE: [usize; LOGICAL_LANE_COUNT] = [0, 2, 1, 4, 5];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecondaryCarrierError {
    InputShape,
    InputNonFinite,
}

impl SecondaryCarrierError {
    pub const fn code(self) -> &'static str {
        "SECONDARY_INPUT_ERROR"
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecondaryCarrier {
    rows: [Vec<u32>; OUTPUT_ROW_COUNT],
}

impl SecondaryCarrier {
    pub const fn rows(&self) -> &[Vec<u32>; OUTPUT_ROW_COUNT] {
        &self.rows
    }
}

/// Drops the per-source warmup block, routes five logical lanes, and leaves
/// LFE plus every incomplete carrier tail as explicit positive zero.
pub fn assemble_secondary_carrier(
    lane_blocks: &[Vec<Vec<u32>>],
) -> Result<SecondaryCarrier, SecondaryCarrierError> {
    if lane_blocks.len() != LOGICAL_LANE_COUNT
        || lane_blocks.iter().any(|lane| {
            lane.len() != BLOCK_COUNT || lane.iter().any(|block| block.len() != BLOCK_SAMPLES)
        })
    {
        return Err(SecondaryCarrierError::InputShape);
    }
    if lane_blocks
        .iter()
        .flatten()
        .flatten()
        .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(SecondaryCarrierError::InputNonFinite);
    }

    let mut rows: [Vec<u32>; OUTPUT_ROW_COUNT] =
        std::array::from_fn(|_| vec![0_u32; CARRIER_SAMPLES]);
    for (logical_lane, output_row) in OUTPUT_ROW_FOR_LOGICAL_LANE.into_iter().enumerate() {
        for (input_block, block_words) in lane_blocks[logical_lane].iter().enumerate().skip(1) {
            let output_block = input_block - 1;
            let destination = output_block * BLOCK_SAMPLES;
            rows[output_row][destination..destination + BLOCK_SAMPLES].copy_from_slice(block_words);
        }
    }

    Ok(SecondaryCarrier { rows })
}
