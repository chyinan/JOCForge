// pattern: Functional Core
//! JOC exposes scoped pure algorithms and immutable boundaries; production-owner composition remains separate.

/// Number of coded dynamic objects, excluding the bypass LFE.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum JocObjectCount {
    #[serde(rename = "8")]
    Eight = 8,
    #[serde(rename = "12")]
    Twelve = 12,
    #[default]
    #[serde(rename = "15")]
    Fifteen = 15,
}

impl JocObjectCount {
    pub(crate) fn source_slots(self, object: usize) -> std::ops::Range<usize> {
        let count = self as usize;
        (object * 15).div_ceil(count)..((object + 1) * 15).div_ceil(count)
    }
}

pub mod additional_bsi;
pub mod clip_producer;
pub mod coefficient_delta;
pub mod coefficient_fitting;
pub mod coefficient_geometry;
pub mod coefficient_normal_equations;
pub mod coefficient_payload;
pub mod coefficient_quantization;
pub mod coefficient_representation;
pub mod coefficient_search;
pub mod coefficient_solver;
pub mod dispatch_state;
pub mod downmix_profile;
pub mod emdf;
pub mod emdf_carrier;
pub mod fit_score;
pub mod huffman;
pub mod id14_body;
pub mod id14_row_normalizer;
pub mod metadata_carrier;
pub mod model_complexity;
pub mod oamd;
pub mod programme_state;
pub mod seven_carrier_windows;
pub mod source_stage;
pub mod temporal_weights;

#[cfg(test)]
mod synthetic_coordinator;
