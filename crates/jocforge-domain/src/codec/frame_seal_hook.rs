// pattern: Functional Core
//! Optional external frame-seal hook.
//!
//! The published frame is complete before CRC2 is computed, so a downstream
//! consumer can patch the access unit (for example to fill a protection field)
//! while the CRC2 still covers the patched bytes. This module only carries
//! layout facts; it holds no policy and no keys.
//!
//! The hook is process-global and installed at most once. A build that never
//! installs one behaves exactly as before.

use std::sync::{Arc, OnceLock};

/// Half-open bit range inside one access unit, measured from the sync word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitRange {
    /// First bit of the range.
    pub start_bit: u32,
    /// Bit just past the range.
    pub end_bit: u32,
}

impl BitRange {
    /// Builds a range from its bounds.
    pub const fn new(start_bit: u32, end_bit: u32) -> Self {
        Self { start_bit, end_bit }
    }

    /// Returns the range width in bits.
    pub const fn len_bits(&self) -> u32 {
        self.end_bit.saturating_sub(self.start_bit)
    }
}

/// Layout facts of one freshly assembled access unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameLayout {
    /// Bit just past the bit-stream information block.
    pub bsi_end_bit: u32,
    /// Bit just past the mixing and informational metadata block.
    pub mixing_end_bit: u32,
    /// First bit of the seven-bit frame-control syntax mask.
    pub controls_mask_start_bit: u32,
    /// Per-block skip-field spans, in block order.
    pub skip_ranges: [BitRange; 6],
    /// First bit of the published EMDF container, when present.
    pub container_start_bit: Option<u32>,
    /// Published container length in octets, including its four-octet header.
    pub container_len_bytes: u16,
    /// Bit just past the last audio block, where tail padding begins.
    pub body_end_bit: u32,
}

/// External consumer that may patch one completed access unit.
pub trait FrameSealHook: Send + Sync {
    /// Patch `frame` in place using `layout`. Runs before CRC2 is computed.
    fn apply(&self, frame: &mut [u8], layout: &FrameLayout);
}

static HOOK: OnceLock<Arc<dyn FrameSealHook>> = OnceLock::new();

/// Installs the process-wide hook. Returns `false` when one is already installed.
pub fn install(hook: Arc<dyn FrameSealHook>) -> bool {
    HOOK.set(hook).is_ok()
}

/// Returns whether a hook is installed.
pub fn installed() -> bool {
    HOOK.get().is_some()
}

/// Applies the installed hook, if any.
pub(crate) fn apply(frame: &mut [u8], layout: &FrameLayout) {
    if let Some(hook) = HOOK.get() {
        hook.apply(frame, layout);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_len_is_saturating() {
        assert_eq!(BitRange::new(10, 14).len_bits(), 4);
        assert_eq!(BitRange::new(14, 10).len_bits(), 0);
    }

    #[test]
    fn apply_without_a_hook_is_inert() {
        let mut frame = [0_u8; 8];
        let layout = FrameLayout {
            bsi_end_bit: 113,
            mixing_end_bit: 90,
            controls_mask_start_bit: 118,
            skip_ranges: [BitRange::new(0, 0); 6],
            container_start_bit: None,
            container_len_bytes: 0,
            body_end_bit: 32_000,
        };
        apply(&mut frame, &layout);
        assert_eq!(frame, [0_u8; 8]);
    }
}
