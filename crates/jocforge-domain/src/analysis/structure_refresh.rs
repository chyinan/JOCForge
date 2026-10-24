// pattern: Functional Core
//! Selected-profile F07.4 structure-refresh flag projection.

/// F07 structure decisions are ordered by six blocks and five full-band lanes.
pub const F07_BLOCKS_PER_FRAME: usize = 6;

/// F07 full-band lane count in the selected six-channel profile.
pub const F07_FULLBAND_LANES: usize = 5;

/// Six block refresh flags for one full-band lane.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StructureRefreshFlags([bool; F07_BLOCKS_PER_FRAME]);

impl StructureRefreshFlags {
    pub const fn as_array(&self) -> &[bool; F07_BLOCKS_PER_FRAME] {
        &self.0
    }
}

/// Derives selected-profile structure-refresh flags from consumed short/other flags.
///
/// The selected profile fixes SPX to zero and end-bin count to 217 for every block,
/// so those two comparison terms are always false. Block zero always refreshes.
pub fn selected_structure_refresh_flags(
    short_consumed: &[bool; F07_BLOCKS_PER_FRAME],
    other_consumed: &[bool; F07_BLOCKS_PER_FRAME],
) -> StructureRefreshFlags {
    let mut refresh = [false; F07_BLOCKS_PER_FRAME];
    refresh[0] = true;
    for block in 1..F07_BLOCKS_PER_FRAME {
        refresh[block] =
            other_consumed[block] || short_consumed[block] != short_consumed[block - 1];
    }
    StructureRefreshFlags(refresh)
}
