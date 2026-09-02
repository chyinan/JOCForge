// pattern: Functional Core

use std::fmt;

/// Explicit clean-side allocation for one 48 kHz JOC access unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RateBudget {
    pub total_bitrate_kbps: u32,
    pub carrier_bitrate_kbps: u32,
    pub d0_bitrate_kbps: u32,
    pub carrier_frame_bytes: usize,
    pub d0_frame_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RateBudgetError {
    UnsupportedTotalBitrate(u32),
}

impl fmt::Display for RateBudgetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedTotalBitrate(value) => {
                write!(formatter, "unsupported total bitrate {value} kbps")
            }
        }
    }
}

impl std::error::Error for RateBudgetError {}

/// Returns the deterministic clean allocation for a supported total rate.
///
/// The two component rates are chosen independently of any reference
/// encoder: both components use legal 48 kHz / 1536-sample frame sizes, and
/// their sum is the requested total rate. The source-derived D0 currently
/// requires its established 640 kbps frame capacity for a valid render.
pub fn rate_budget(total_bitrate_kbps: u32) -> Result<RateBudget, RateBudgetError> {
    let (carrier_bitrate_kbps, d0_bitrate_kbps) = match total_bitrate_kbps {
        1024 => (384, 640),
        1280 => (640, 640),
        1408 => (768, 640),
        1536 => (896, 640),
        1664 => (1024, 640),
        other => return Err(RateBudgetError::UnsupportedTotalBitrate(other)),
    };
    Ok(RateBudget {
        total_bitrate_kbps,
        carrier_bitrate_kbps,
        d0_bitrate_kbps,
        carrier_frame_bytes: frame_bytes(carrier_bitrate_kbps),
        d0_frame_bytes: frame_bytes(d0_bitrate_kbps),
    })
}

fn frame_bytes(bitrate_kbps: u32) -> usize {
    (bitrate_kbps as usize) * 4
}
