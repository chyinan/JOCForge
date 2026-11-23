// pattern: Imperative Shell
//! Selected-profile encode pipeline shared by the native entry point and the browser bridge.
//!
//! This crate owns the encode orchestration that sits between the application owner roots and the
//! io adapters: input preparation, the meter prepass, the spatial callback loop, source-frame
//! staging and the F15 output publication. It is a library with no process entry, so a caller
//! supplies the input resource, the destination and the provider root instead of relying on an
//! implicit working directory.

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod cmaf;
pub mod encode_request;
pub mod independent_meter_callback;
pub(crate) mod loudness_cache;
pub mod meter_callback_stage;
pub mod meter_prepass;
pub mod meter_source_bridge;
pub mod output_publication;
pub(crate) mod programme_roots;
pub(crate) mod progress;
pub mod selected_meter_provider_identity;
pub mod selected_spatial_callback_providers;
pub mod source_execution;
