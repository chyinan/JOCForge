// pattern: Functional Core
//! Codec boundaries consume approved immutable receipts; local F12 helper slices are explicitly scoped.

pub mod authenticated_bit_compaction;
pub mod bap;
pub mod bitio;
pub mod coefficient_power;
pub mod configuration_import;
pub mod converter_header_cost;
pub mod converter_search;
pub mod core_layout;
pub mod db_selector;
pub mod exponent_normalization;
pub mod exponent_packing;
pub mod exponent_refresh_state;
pub mod frame_builder;
pub mod frame_size;
pub mod frame_syntax;
pub mod grouped_quantizer;
pub mod lfe_coefficient_exponent;
pub mod mantissa;
pub mod owner;
pub mod policy_parameters;
pub mod published_emdf_binding;
pub mod rate_profile;
pub mod raw_exponent_from_power;
pub mod refresh_difference;
pub mod selected_frame_trial;
pub mod selected_main_q_search;
pub mod selected_source_plan;
pub mod source_frame_writer;
pub mod substream_topology;
