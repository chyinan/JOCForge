// pattern: Functional Core
//! Selected-profile F07.5 source-frame dither flag and previous-short projection.

use super::structure_refresh::{F07_BLOCKS_PER_FRAME, F07_FULLBAND_LANES};

/// Six blocks of five full-band dither flags.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DitherFrameFlags([[bool; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME]);

impl DitherFrameFlags {
    pub const fn as_array(&self) -> &[[bool; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME] {
        &self.0
    }
}

/// Selected-profile F07.5 results for one ordered six-block input frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DitherPlan {
    flags: DitherFrameFlags,
    syntax_present: bool,
    next_previous_short: [bool; F07_FULLBAND_LANES],
}

impl DitherPlan {
    pub const fn flags(&self) -> &DitherFrameFlags {
        &self.flags
    }

    pub const fn syntax_present(&self) -> bool {
        self.syntax_present
    }

    pub const fn next_previous_short(&self) -> &[bool; F07_FULLBAND_LANES] {
        &self.next_previous_short
    }
}

/// Plans per-block dither flags from consumed short flags and prior-block state.
///
/// The caller supplies current short decisions in block-major order. This helper
/// owns no filter, queue, source clock, header writer, or commit transaction.
pub fn plan_selected_dither_flags(
    previous_short: [bool; F07_FULLBAND_LANES],
    current_short: &[[bool; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
) -> DitherPlan {
    let mut previous = previous_short;
    let mut flags = [[false; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME];
    let mut syntax_present = false;

    for block in 0..F07_BLOCKS_PER_FRAME {
        for lane in 0..F07_FULLBAND_LANES {
            let is_dithered = !current_short[block][lane] && !previous[lane];
            flags[block][lane] = is_dithered;
            syntax_present |= !is_dithered;
            previous[lane] = current_short[block][lane];
        }
    }

    DitherPlan {
        flags: DitherFrameFlags(flags),
        syntax_present,
        next_previous_short: previous,
    }
}
