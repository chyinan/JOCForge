// pattern: Imperative Shell
//! CLI composition helpers for the selected input adapters.

pub mod independent_meter_callback;
pub mod meter_callback_stage;
pub mod meter_source_bridge;
#[cfg(any(test, feature = "test-support"))]
pub mod output_publication;
pub mod selected_meter_provider_identity;
pub mod selected_spatial_callback_providers;
