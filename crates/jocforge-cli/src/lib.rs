// pattern: Imperative Shell
//! Process entry for the selected-profile encode pipeline.
//!
//! The encode pipeline itself lives in the `jocforge-pipeline` library. This crate owns argument
//! parsing, process exit codes and the binary entry only; it re-exports the pipeline modules under
//! the names the CLI and its integration tests already use.

pub use jocforge_pipeline::{
    encode_request, independent_meter_callback, meter_callback_stage, meter_prepass,
    meter_source_bridge, output_publication, selected_meter_provider_identity,
    selected_spatial_callback_providers, source_execution,
};
