// pattern: Functional Core
//! JOC exposes scoped pure algorithms and immutable boundaries; production-owner composition remains separate.

pub mod additional_bsi;
pub mod clip_producer;
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
pub mod metadata_carrier;
pub mod model_complexity;
pub mod oamd;
pub mod programme_state;
pub mod source_stage;
pub mod temporal_weights;

#[cfg(test)]
mod synthetic_coordinator;
