// pattern: Functional Core
//! F07.1 one-lane, one-block binary32 input-history transition.

/// F07.1 persistent input-history samples per lane.
pub const INPUT_HISTORY_SAMPLES: usize = 1_280;
/// F07.1 block samples per lane.
pub const INPUT_HISTORY_BLOCK_SAMPLES: usize = 256;
const HISTORY_OUTPUT_START: usize = INPUT_HISTORY_BLOCK_SAMPLES;
const HISTORY_OUTPUT_END: usize = HISTORY_OUTPUT_START + INPUT_HISTORY_BLOCK_SAMPLES;
const RETAINED_HISTORY_SAMPLES: usize = INPUT_HISTORY_SAMPLES - INPUT_HISTORY_BLOCK_SAMPLES;

/// One lane's F07.1 persistent binary32 history, stored as exact U32 words.
#[derive(Debug, Eq, PartialEq)]
pub struct InputHistory([u32; INPUT_HISTORY_SAMPLES]);

/// One delayed F07.1 output block, stored as exact U32 words.
#[derive(Debug, Eq, PartialEq)]
pub struct DelayedInputBlock([u32; INPUT_HISTORY_BLOCK_SAMPLES]);

impl InputHistory {
    /// Creates the cold state from positive-zero binary32 words.
    pub const fn cold() -> Self {
        Self([0; INPUT_HISTORY_SAMPLES])
    }

    /// Returns one block from the old history and the next history state.
    ///
    /// The old window is history[256..512]. The next state drops the earliest
    /// 256 words and appends the current block, without interpreting F32 values.
    pub fn advance_block(
        self,
        current: &[u32; INPUT_HISTORY_BLOCK_SAMPLES],
    ) -> (DelayedInputBlock, Self) {
        let mut delayed = [0; INPUT_HISTORY_BLOCK_SAMPLES];
        delayed.copy_from_slice(&self.0[HISTORY_OUTPUT_START..HISTORY_OUTPUT_END]);

        let mut next = [0; INPUT_HISTORY_SAMPLES];
        next[..RETAINED_HISTORY_SAMPLES].copy_from_slice(&self.0[INPUT_HISTORY_BLOCK_SAMPLES..]);
        next[RETAINED_HISTORY_SAMPLES..].copy_from_slice(current);
        (DelayedInputBlock(delayed), Self(next))
    }

    pub const fn words(&self) -> &[u32; INPUT_HISTORY_SAMPLES] {
        &self.0
    }

    pub(crate) fn staged_copy(&self) -> Self {
        Self(self.0)
    }
}

impl DelayedInputBlock {
    pub const fn words(&self) -> &[u32; INPUT_HISTORY_BLOCK_SAMPLES] {
        &self.0
    }
}
