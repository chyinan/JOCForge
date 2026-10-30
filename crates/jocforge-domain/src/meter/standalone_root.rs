// pattern: Imperative Shell
//! Standalone selected Meter owner around the existing independent prepass leaves.

use std::collections::BTreeMap;

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::measurement_receipt::{
    MeasurementReceiptError, ProgrammeMeasurementReceipt, ProgrammeMeasurementReceiptInput,
};
pub use super::source_frame::MeterSourceReadIndex;
use super::{
    k_matrix::KMatrixProvider,
    lfe_delay_ring::{MeterFrontendCallbackInput, MeterLfeCallbackOrigin},
    meter_prepass::{IndependentMeterPrepass, IndependentMeterPrepassError},
    post_render_meter::PostRenderMeterParameters,
    post_render_meter::PostRenderMeterReceipt,
    programme_measurement::{
        ProgrammeMeasurementError, ProgrammeMeasurementValue, compute_programme_measurement_values,
    },
    sample_block_queue::{
        MeterBlockCount, MeterCallbackCount, MeterCallbackIndex, MeterCallbackPhase,
        MeterSampleCount, MeterSourceSampleCount, MeterUpstreamGeneration,
    },
    shared_gain_delay::SharedGainDelayParameters,
    source_frame::MeterSourcePosition,
};
use crate::transaction::{AttemptId, ProgrammeId};

const SELECTED_SOURCE_SAMPLES: u64 = 6_522_906;
const SELECTED_NORMAL_CALLBACKS: u64 = 4_247;
const SELECTED_ZERO_DRAINS: u64 = 3;
const SELECTED_METER_CALLBACKS: u64 = 4_250;
const SELECTED_PUBLISHED_SAMPLES: u64 = 6_523_392;
const SELECTED_COMPLETE_BLOCKS: u64 = 6_370;
const SELECTED_COMPLETE_PREFIX_SAMPLES: u64 = 6_522_880;
const SELECTED_UNUSED_BLOCK_TAIL: u64 = 512;
const SELECTED_ENERGY_WINDOWS: u64 = 6_794;
const SELECTED_ENERGY_SAMPLES: u64 = 6_522_240;
const SELECTED_ENERGY_WINDOW_REMAINDER: u64 = 640;
const METER_BLOCK_SAMPLES: u64 = 1_024;
const METER_ENERGY_WINDOW_SAMPLES: u64 = 960;
const SELECTED_CHANNELS: u64 = 38;
const SELECTED_PCM_BITS: u64 = 24;
const SELECTED_SAMPLE_RATE_HZ: u64 = 48_000;
const SELECTED_PCM_FRAME_BYTES: u64 = SELECTED_CHANNELS * (SELECTED_PCM_BITS / 8);
const SELECTED_CALLBACK_SAMPLES: u64 = 1_536;
const ROOT_SCHEMA: &str = "standalone-meter-root-materialization.v3";
const METER_CHANNEL_COUNT: usize = 6;

const OAR_STATE_LAYOUT: &str = "OAR_MODEL_CALLBACK_STATE_V1";
const SHARED_DV32_STATE_LAYOUT: &str = "SHARED_DV32_STATE_V1";
const INTEGRATED_STATE_LAYOUT: &str = "METER_INTEGRATED_STATE_V1";
const UNGATED_STATE_LAYOUT: &str = "MEAN_STATE_V1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandaloneMeterRootError {
    UnsupportedSourceExtent,
    AttemptOverflow,
    AttemptSequence,
    CallbackClock,
    SourceOwnerId,
    ProviderSet,
    SourceCursorProgress,
    FinalizationNotReady,
    SelectedScheduleMismatch,
    CanonicalState,
    Prepass(IndependentMeterPrepassError),
    Measurement(ProgrammeMeasurementError),
    MeasurementReceipt(MeasurementReceiptError),
}

/// Parent-issued opaque identity for the local Meter PCM source cursor.
/// It is never synthesized from audio, a path, or another provider's identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterSourceOwnerId(String);

impl MeterSourceOwnerId {
    pub fn new(value: impl Into<String>) -> Result<Self, StandaloneMeterRootError> {
        let value = value.into();
        if value.is_empty() {
            return Err(StandaloneMeterRootError::SourceOwnerId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One provider identity supplied by the parent with the approved provider set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterProviderIdentity {
    id: String,
    sha256: [u8; 32],
}

impl MeterProviderIdentity {
    pub fn new(id: impl Into<String>, sha256: [u8; 32]) -> Result<Self, StandaloneMeterRootError> {
        let id = id.into();
        if id.is_empty() {
            return Err(StandaloneMeterRootError::ProviderSet);
        }
        Ok(Self { id, sha256 })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn sha256(&self) -> &[u8; 32] {
        &self.sha256
    }
}

/// Parent-supplied digest and sorted member identities for this exact provider set.
/// The approved package does not define a digest recipe, so this type preserves
/// the supplied aggregate and never recomputes it from the member rows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterProviderSet {
    sha256: [u8; 32],
    providers: Vec<MeterProviderIdentity>,
}

impl MeterProviderSet {
    pub fn new(
        sha256: [u8; 32],
        providers: Vec<MeterProviderIdentity>,
    ) -> Result<Self, StandaloneMeterRootError> {
        if providers.is_empty()
            || providers
                .windows(2)
                .any(|pair| pair[0].id.as_bytes() >= pair[1].id.as_bytes())
        {
            return Err(StandaloneMeterRootError::ProviderSet);
        }
        Ok(Self { sha256, providers })
    }

    pub const fn sha256(&self) -> &[u8; 32] {
        &self.sha256
    }

    pub fn providers(&self) -> &[MeterProviderIdentity] {
        &self.providers
    }
}

/// Distinct typed state for the K-filter exponents in the canonical root.
/// This is supplied by the current owner; it is not copied from the energy exponents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeterKFilterExponentState([i32; METER_CHANNEL_COUNT]);

impl MeterKFilterExponentState {
    pub const fn new(values: [i32; METER_CHANNEL_COUNT]) -> Self {
        Self(values)
    }

    pub const fn values(self) -> [i32; METER_CHANNEL_COUNT] {
        self.0
    }
}

/// Typed parent evidence for the source-cursor and K-exponent after-state of one callback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeterRootCallbackProgress {
    callback_index: MeterCallbackIndex,
    source_cursor_before: MeterSourcePosition,
    source_cursor_after: MeterSourcePosition,
    read_index_before: MeterSourceReadIndex,
    read_index_after: MeterSourceReadIndex,
    host_input_count: MeterSampleCount,
    k_filter_exponents_after: MeterKFilterExponentState,
}

impl MeterRootCallbackProgress {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        callback_index: MeterCallbackIndex,
        source_cursor_before: MeterSourcePosition,
        source_cursor_after: MeterSourcePosition,
        read_index_before: MeterSourceReadIndex,
        read_index_after: MeterSourceReadIndex,
        host_input_count: MeterSampleCount,
        k_filter_exponents_after: MeterKFilterExponentState,
    ) -> Self {
        Self {
            callback_index,
            source_cursor_before,
            source_cursor_after,
            read_index_before,
            read_index_after,
            host_input_count,
            k_filter_exponents_after,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MeterSourceCursorState {
    source_cursor: MeterSourcePosition,
    read_index: MeterSourceReadIndex,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct MeterEnergyWindowCount(u64);

impl MeterEnergyWindowCount {
    const fn new(value: u64) -> Self {
        Self(value)
    }

    const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandaloneMeterIdentity {
    programme_id: ProgrammeId,
    attempt: AttemptId,
    input_sha256: [u8; 32],
    config_sha256: [u8; 32],
}

impl StandaloneMeterIdentity {
    pub const fn new(
        programme_id: ProgrammeId,
        attempt: AttemptId,
        input_sha256: [u8; 32],
        config_sha256: [u8; 32],
    ) -> Self {
        Self {
            programme_id,
            attempt,
            input_sha256,
            config_sha256,
        }
    }

    pub fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub const fn attempt(&self) -> AttemptId {
        self.attempt
    }

    pub const fn input_sha256(&self) -> &[u8; 32] {
        &self.input_sha256
    }

    pub const fn config_sha256(&self) -> &[u8; 32] {
        &self.config_sha256
    }
}

/// Fixed selected-profile meter schedule derived from the admitted source extent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StandaloneMeterSchedule {
    normal_callbacks: MeterCallbackCount,
    zero_drains: MeterCallbackCount,
    meter_callbacks: MeterCallbackCount,
    published_samples: MeterSampleCount,
    source_padding_samples: MeterSampleCount,
    complete_blocks: MeterBlockCount,
    complete_prefix_samples: MeterSampleCount,
    unused_block_tail_samples: MeterSampleCount,
    energy_windows: u64,
    energy_samples: MeterSampleCount,
    energy_window_remainder: MeterSampleCount,
}

impl StandaloneMeterSchedule {
    pub const fn selected() -> Self {
        Self {
            normal_callbacks: MeterCallbackCount::new(SELECTED_NORMAL_CALLBACKS),
            zero_drains: MeterCallbackCount::new(SELECTED_ZERO_DRAINS),
            meter_callbacks: MeterCallbackCount::new(SELECTED_METER_CALLBACKS),
            published_samples: MeterSampleCount::new(SELECTED_PUBLISHED_SAMPLES),
            source_padding_samples: MeterSampleCount::new(
                SELECTED_PUBLISHED_SAMPLES - SELECTED_SOURCE_SAMPLES,
            ),
            complete_blocks: MeterBlockCount::new(SELECTED_COMPLETE_BLOCKS),
            complete_prefix_samples: MeterSampleCount::new(SELECTED_COMPLETE_PREFIX_SAMPLES),
            unused_block_tail_samples: MeterSampleCount::new(SELECTED_UNUSED_BLOCK_TAIL),
            energy_windows: SELECTED_ENERGY_WINDOWS,
            energy_samples: MeterSampleCount::new(SELECTED_ENERGY_SAMPLES),
            energy_window_remainder: MeterSampleCount::new(SELECTED_ENERGY_WINDOW_REMAINDER),
        }
    }

    pub const fn normal_callbacks(self) -> MeterCallbackCount {
        self.normal_callbacks
    }

    pub const fn zero_drains(self) -> MeterCallbackCount {
        self.zero_drains
    }

    pub const fn meter_callbacks(self) -> MeterCallbackCount {
        self.meter_callbacks
    }

    pub const fn published_samples(self) -> MeterSampleCount {
        self.published_samples
    }

    pub const fn source_padding_samples(self) -> MeterSampleCount {
        self.source_padding_samples
    }

    pub const fn complete_blocks(self) -> MeterBlockCount {
        self.complete_blocks
    }

    pub const fn complete_prefix_samples(self) -> MeterSampleCount {
        self.complete_prefix_samples
    }

    pub const fn unused_block_tail_samples(self) -> MeterSampleCount {
        self.unused_block_tail_samples
    }

    pub const fn energy_windows(self) -> u64 {
        self.energy_windows
    }

    pub const fn energy_samples(self) -> MeterSampleCount {
        self.energy_samples
    }

    pub const fn energy_window_remainder(self) -> MeterSampleCount {
        self.energy_window_remainder
    }
}

/// The standalone METER-phase numeric owner. MAIN owns a separate fresh state.
#[derive(Debug)]
pub struct StandaloneMeterRoot {
    identity: StandaloneMeterIdentity,
    source_samples: MeterSourceSampleCount,
    /// Stable digest of the complete canonical root as supplied to `cold_selected`.
    /// This identity is deliberately excluded from canonical state bytes.
    cold_state_sha256: [u8; 32],
    source_owner_id: MeterSourceOwnerId,
    provider_set: MeterProviderSet,
    source_cursor: MeterSourceCursorState,
    completed_energy_windows: MeterEnergyWindowCount,
    gain_parameters: SharedGainDelayParameters,
    k_matrix: KMatrixProvider,
    k_filter_exponents: MeterKFilterExponentState,
    prepass: IndependentMeterPrepass,
}

impl StandaloneMeterRoot {
    /// Constructs the fixed selected standalone meter from admitted immutable values.
    pub fn cold_selected(
        identity: StandaloneMeterIdentity,
        source_samples: MeterSourceSampleCount,
        source_owner_id: MeterSourceOwnerId,
        provider_set: MeterProviderSet,
        k_filter_exponents: MeterKFilterExponentState,
        gain_parameters: SharedGainDelayParameters,
        k_matrix: KMatrixProvider,
    ) -> Result<Self, StandaloneMeterRootError> {
        let prepass = IndependentMeterPrepass::cold(
            source_samples,
            gain_parameters.clone(),
            k_matrix.clone(),
        )
        .map_err(StandaloneMeterRootError::Prepass)?;
        Self::cold_selected_from_prepass(
            identity,
            source_samples,
            source_owner_id,
            provider_set,
            k_filter_exponents,
            gain_parameters,
            k_matrix,
            prepass,
        )
    }

    /// Constructs the selected standalone meter with approved direct post-render values.
    #[allow(clippy::too_many_arguments)]
    pub fn cold_selected_with_post_render_parameters(
        identity: StandaloneMeterIdentity,
        source_samples: MeterSourceSampleCount,
        source_owner_id: MeterSourceOwnerId,
        provider_set: MeterProviderSet,
        k_filter_exponents: MeterKFilterExponentState,
        gain_parameters: SharedGainDelayParameters,
        k_matrix: KMatrixProvider,
        post_render_parameters: PostRenderMeterParameters,
    ) -> Result<Self, StandaloneMeterRootError> {
        let prepass = IndependentMeterPrepass::cold_with_post_render_parameters(
            source_samples,
            gain_parameters.clone(),
            k_matrix.clone(),
            post_render_parameters,
        )
        .map_err(StandaloneMeterRootError::Prepass)?;
        Self::cold_selected_from_prepass(
            identity,
            source_samples,
            source_owner_id,
            provider_set,
            k_filter_exponents,
            gain_parameters,
            k_matrix,
            prepass,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn cold_selected_from_prepass(
        identity: StandaloneMeterIdentity,
        source_samples: MeterSourceSampleCount,
        source_owner_id: MeterSourceOwnerId,
        provider_set: MeterProviderSet,
        k_filter_exponents: MeterKFilterExponentState,
        gain_parameters: SharedGainDelayParameters,
        k_matrix: KMatrixProvider,
        prepass: IndependentMeterPrepass,
    ) -> Result<Self, StandaloneMeterRootError> {
        if source_samples.value() != SELECTED_SOURCE_SAMPLES {
            return Err(StandaloneMeterRootError::UnsupportedSourceExtent);
        }
        let mut root = Self {
            identity,
            source_samples,
            cold_state_sha256: [0; 32],
            source_owner_id,
            provider_set,
            source_cursor: MeterSourceCursorState {
                source_cursor: MeterSourcePosition::new(0),
                read_index: MeterSourceReadIndex::new(0),
            },
            completed_energy_windows: MeterEnergyWindowCount::new(0),
            gain_parameters,
            k_matrix,
            k_filter_exponents,
            prepass,
        };
        root.cold_state_sha256 = root.canonical_sha256()?;
        Ok(root)
    }

    pub fn identity(&self) -> &StandaloneMeterIdentity {
        &self.identity
    }

    pub const fn source_samples(&self) -> MeterSourceSampleCount {
        self.source_samples
    }

    pub const fn meter_callback_index(&self) -> MeterCallbackIndex {
        self.prepass.next_callback_index()
    }

    pub const fn normal_callback_count(&self) -> MeterCallbackCount {
        self.prepass.normal_callback_count()
    }

    pub const fn prepass(&self) -> &IndependentMeterPrepass {
        &self.prepass
    }

    pub const fn k_filter_exponents_i32_by_channel(&self) -> &[i32; METER_CHANNEL_COUNT] {
        &self.k_filter_exponents.0
    }

    pub const fn source_cursor(&self) -> MeterSourcePosition {
        self.source_cursor.source_cursor
    }

    pub const fn source_read_index(&self) -> MeterSourceReadIndex {
        self.source_cursor.read_index
    }

    /// Returns the complete local canonical root preimage as compact sorted-key
    /// UTF-8 JSON. This projects a value only; it does not commit the root or
    /// mint a GTC owner receipt or `MeasurementReceipt`.
    pub fn canonical_preimage(&self) -> Result<Vec<u8>, StandaloneMeterRootError> {
        let data_length_bytes = self
            .source_samples
            .value()
            .checked_mul(SELECTED_PCM_FRAME_BYTES)
            .ok_or(StandaloneMeterRootError::CanonicalState)?;
        let expected_published_samples = SELECTED_NORMAL_CALLBACKS
            .checked_mul(SELECTED_CALLBACK_SAMPLES)
            .ok_or(StandaloneMeterRootError::CanonicalState)?;
        let total_callbacks = SELECTED_NORMAL_CALLBACKS
            .checked_add(SELECTED_ZERO_DRAINS)
            .ok_or(StandaloneMeterRootError::CanonicalState)?;
        let callback_index = self.meter_callback_index().value();
        if self.source_cursor.source_cursor.value() > self.source_samples.value()
            || self.source_cursor.read_index.value() > SELECTED_NORMAL_CALLBACKS
            || callback_index > total_callbacks
            || self.completed_energy_windows.value() > SELECTED_ENERGY_WINDOWS
        {
            return Err(StandaloneMeterRootError::CanonicalState);
        }

        let post_render = self.prepass.post_render_state();
        let frontend_oar = post_render
            .object_renderer()
            .canonical_bytes()
            .map_err(|_| StandaloneMeterRootError::CanonicalState)?;
        let shared_dv32 = post_render.shared_gain_delay().canonical_bytes();
        let integrated = post_render.integrated_meter().canonical_bytes();
        let ungated = post_render.ungated_mean().canonical_bytes();
        if shared_dv32.len() != 812 || integrated.len() != 24_512 || ungated.len() != 40 {
            return Err(StandaloneMeterRootError::CanonicalState);
        }

        let queue = post_render.sample_queue();
        let pending_lanes = queue.pending_lanes();
        let window_clock = post_render.window_clock();
        let lfe_state = self.prepass.lfe_state();
        let queued_pcm = queue
            .pcm_enqueued_samples()
            .value()
            .checked_sub(queue.pcm_consumed_samples().value())
            .ok_or(StandaloneMeterRootError::CanonicalState)?;
        if pending_lanes
            .iter()
            .any(|lane| u64::try_from(lane.len()).ok() != Some(queued_pcm))
            || window_clock.next_callback_index().value() != callback_index
            || queue.next_callback_index().value() != callback_index
            || lfe_state.callback_index().value() != callback_index
            || queue.source_samples() != self.source_samples
            || window_clock.source_samples() != self.source_samples
            || queue.normal_callback_count().value() != SELECTED_NORMAL_CALLBACKS
            || queue.expected_published_samples().value() != expected_published_samples
            || window_clock.processed_samples().value() != queue.published_samples().value()
        {
            return Err(StandaloneMeterRootError::CanonicalState);
        }

        let energy = post_render.energy_window();
        let energy_samples = self
            .completed_energy_windows
            .value()
            .checked_mul(METER_ENERGY_WINDOW_SAMPLES)
            .and_then(|samples| samples.checked_add(u64::from(energy.samples_in_window())))
            .ok_or(StandaloneMeterRootError::CanonicalState)?;
        if energy_samples != queue.pcm_consumed_samples().value() {
            return Err(StandaloneMeterRootError::CanonicalState);
        }
        let true_peak = post_render.true_peak();
        let fir_state_bytes = true_peak.history_f32le_bytes();
        let expected_fir_bytes = METER_CHANNEL_COUNT * 4_628 * 4;
        if fir_state_bytes.len() != expected_fir_bytes {
            return Err(StandaloneMeterRootError::CanonicalState);
        }
        let mut lfe_frames = Vec::with_capacity(3);
        for slot in 0..3 {
            let frame = match (
                self.prepass.lfe_state().slot_origin(slot),
                self.prepass.lfe_state().slot_samples(slot),
            ) {
                (None, None) => Value::Null,
                (Some(origin), Some(sample_words)) if sample_words.len() == 1_536 => {
                    serde_json::json!({
                        "origin": origin.callback_index().value(),
                        "sample_words": sample_words,
                    })
                }
                _ => return Err(StandaloneMeterRootError::CanonicalState),
            };
            lfe_frames.push(frame);
        }

        let provider_rows = self
            .provider_set
            .providers()
            .iter()
            .map(|provider| {
                serde_json::json!({
                    "id": provider.id(),
                    "sha256": lowercase_hex(provider.sha256()),
                })
            })
            .collect::<Vec<_>>();

        let root_value = serde_json::json!({
            "schema": ROOT_SCHEMA,
            "binding": {
                "programme_id": self.identity.programme_id().as_str(),
                "attempt": self.identity.attempt().value(),
                "input_sha256": lowercase_hex(self.identity.input_sha256()),
                "config_sha256": lowercase_hex(self.identity.config_sha256()),
                "source_samples": self.source_samples.value(),
                "data_length_bytes": data_length_bytes,
                "channels": SELECTED_CHANNELS,
                "pcm_bits": SELECTED_PCM_BITS,
                "sample_rate_hz": SELECTED_SAMPLE_RATE_HZ,
                "callback_samples": SELECTED_CALLBACK_SAMPLES,
                "normal_callbacks": SELECTED_NORMAL_CALLBACKS,
                "expected_published_samples": expected_published_samples,
                "zero_input_drains": SELECTED_ZERO_DRAINS,
                "provider_set_sha256": lowercase_hex(self.provider_set.sha256()),
                "provider_ids_hash_sorted": provider_rows,
            },
            "input_cursor": {
                "model_owner_key": self.source_owner_id.as_str(),
                "duration_frames": self.source_samples.value(),
                "source_cursor": self.source_cursor.source_cursor.value(),
                "read_index": self.source_cursor.read_index.value(),
                "data_length_bytes": data_length_bytes,
                "channels": SELECTED_CHANNELS,
                "pcm_bits": SELECTED_PCM_BITS,
                "source_identity_sha256": lowercase_hex(self.identity.input_sha256()),
            },
            "callback_clock": {
                "committed_meter_callbacks": callback_index,
                "next_meter_callback_index": callback_index,
                "normal_callbacks": SELECTED_NORMAL_CALLBACKS,
                "zero_input_drains": SELECTED_ZERO_DRAINS,
                "total_meter_callbacks": total_callbacks,
            },
            "leaves": {
                "frontend_oar": {
                    "layout": OAR_STATE_LAYOUT,
                    "bytes_le_hex": lowercase_hex(&frontend_oar),
                },
                "shared_dv32": {
                    "layout": SHARED_DV32_STATE_LAYOUT,
                    "bytes_le_hex": lowercase_hex(&shared_dv32),
                },
                "window_k_energy": {
                    "schedule": {
                        "source_samples": self.source_samples.value(),
                        "normal_callbacks": SELECTED_NORMAL_CALLBACKS,
                        "drain_callbacks": SELECTED_ZERO_DRAINS,
                        "total_callbacks": total_callbacks,
                    },
                    "window_clock": {
                        "callback_index": window_clock.next_callback_index().value(),
                        "startup_remaining": window_clock.startup_remaining().value(),
                        "scheduler_pending": window_clock.scheduler_pending().value(),
                        "processed": window_clock.processed_samples().value(),
                    },
                    "sample_block_queue": {
                        "callback_index": queue.next_callback_index().value(),
                        "startup_remaining": queue.startup_remaining().value(),
                        "scheduler_pending": queue.scheduler_pending().value(),
                        "published_samples": queue.published_samples().value(),
                        "last_upstream_generation": queue
                            .last_upstream_generation()
                            .map(MeterUpstreamGeneration::value),
                        "pcm_block_fill_cursor": u32::from(queue.block_fill_cursor().value()),
                        "pcm_enqueued_samples": queue.pcm_enqueued_samples().value(),
                        "pcm_consumed_samples": queue.pcm_consumed_samples().value(),
                        "complete_block_count": queue.complete_block_count().value(),
                        "pending_sample_words": pending_lanes,
                    },
                    "k_filter": {
                        "state_f32_u32_by_channel_stage_lane": energy.k_state_words(),
                        "exponents_i32_by_channel": self.k_filter_exponents.values(),
                    },
                    "energy_window_accumulator": {
                        "completed_windows": self.completed_energy_windows.value(),
                        "mantissa_f32_u32_by_channel": energy.accumulator_mantissa_words(),
                        "exponent_i32_by_channel": energy.accumulator_exponents(),
                        "samples_in_window": energy.samples_in_window(),
                    },
                },
                "true_peak": {
                    "fir_state_f32le_hex": lowercase_hex(&fir_state_bytes),
                    "programme_true_max_f32_u32": true_peak.true_peak_internal_words(),
                    "programme_sample_max_f32_u32": true_peak.sample_peak_internal_words(),
                },
                "integrated": {
                    "layout": INTEGRATED_STATE_LAYOUT,
                    "bytes_le_hex": lowercase_hex(&integrated),
                },
                "ungated": {
                    "layout": UNGATED_STATE_LAYOUT,
                    "bytes_le_hex": lowercase_hex(&ungated),
                },
                "meter_lfe_delay_ring": {
                    "source_samples": lfe_state.source_samples().value(),
                    "normal_callbacks": lfe_state.normal_callbacks().value(),
                    "callback_index": lfe_state.callback_index().value(),
                    "samples_consumed": lfe_state.samples_consumed().value(),
                    "write_index": lfe_state.write_index(),
                    "read_index": lfe_state.read_index(),
                    "frames": lfe_frames,
                },
            },
        });

        serde_json::to_vec(&sort_canonical_json(root_value))
            .map_err(|_| StandaloneMeterRootError::CanonicalState)
    }

    pub fn canonical_sha256(&self) -> Result<[u8; 32], StandaloneMeterRootError> {
        Ok(Sha256::digest(self.canonical_preimage()?).into())
    }

    /// Returns the stable complete-root digest captured by the cold constructor.
    /// Parent-supplied mutable progress (including K exponents) cannot redefine it.
    pub fn cold_canonical_sha256(&self) -> Result<[u8; 32], StandaloneMeterRootError> {
        Ok(self.cold_state_sha256)
    }

    /// Stages one normal or fresh-zero callback and returns an owned after-root.
    /// The before-root is unchanged; root installation belongs to its parent.
    pub fn prepare_callback(
        &self,
        input: MeterFrontendCallbackInput,
        progress: MeterRootCallbackProgress,
    ) -> Result<StandaloneMeterCallbackProposal, StandaloneMeterRootError> {
        let before_callback = self.prepass.next_callback_index();
        if before_callback.value() >= SELECTED_METER_CALLBACKS {
            return Err(StandaloneMeterRootError::CallbackClock);
        }
        let source_cursor = self.validate_callback_progress(&input, progress)?;

        let leaf_proposal = self
            .prepass
            .propose_callback(input)
            .map_err(StandaloneMeterRootError::Prepass)?;
        let (after_prepass, leaf_receipt, lfe_output_origin) = leaf_proposal.into_parts();
        let expected_after = before_callback
            .value()
            .checked_add(1)
            .ok_or(StandaloneMeterRootError::CallbackClock)?;
        if leaf_receipt.callback_index() != before_callback
            || after_prepass.next_callback_index().value() != expected_after
        {
            return Err(StandaloneMeterRootError::CallbackClock);
        }

        let after = Self {
            identity: self.identity.clone(),
            source_samples: self.source_samples,
            cold_state_sha256: self.cold_state_sha256,
            source_owner_id: self.source_owner_id.clone(),
            provider_set: self.provider_set.clone(),
            source_cursor,
            completed_energy_windows: MeterEnergyWindowCount::new(
                self.completed_energy_windows
                    .value()
                    .checked_add(leaf_receipt.energy_records().value())
                    .ok_or(StandaloneMeterRootError::SelectedScheduleMismatch)?,
            ),
            gain_parameters: self.gain_parameters.clone(),
            k_matrix: self.k_matrix.clone(),
            k_filter_exponents: progress.k_filter_exponents_after,
            prepass: after_prepass,
        };
        Ok(StandaloneMeterCallbackProposal {
            receipt: StandaloneMeterCallbackReceipt {
                identity: self.identity.clone(),
                callback_index: before_callback,
                next_callback_index: MeterCallbackIndex::new(expected_after),
                lfe_output_origin,
                post_render: leaf_receipt,
            },
            after,
        })
    }

    /// Rebuilds every meter leaf from cold and advances exactly one programme attempt.
    #[allow(clippy::too_many_arguments)]
    pub fn full_reset(
        self,
        identity: StandaloneMeterIdentity,
        source_samples: MeterSourceSampleCount,
        source_owner_id: MeterSourceOwnerId,
        provider_set: MeterProviderSet,
        k_filter_exponents: MeterKFilterExponentState,
        gain_parameters: SharedGainDelayParameters,
        k_matrix: KMatrixProvider,
    ) -> Result<Self, StandaloneMeterRootError> {
        let expected_attempt = self
            .identity
            .attempt
            .value()
            .checked_add(1)
            .ok_or(StandaloneMeterRootError::AttemptOverflow)?;
        if identity.programme_id != self.identity.programme_id
            || identity.attempt.value() != expected_attempt
        {
            return Err(StandaloneMeterRootError::AttemptSequence);
        }
        let post_render_parameters = self.prepass.post_render_parameters().clone();
        Self::cold_selected_with_post_render_parameters(
            identity,
            source_samples,
            source_owner_id,
            provider_set,
            k_filter_exponents,
            gain_parameters,
            k_matrix,
            post_render_parameters,
        )
    }

    fn validate_callback_progress(
        &self,
        input: &MeterFrontendCallbackInput,
        progress: MeterRootCallbackProgress,
    ) -> Result<MeterSourceCursorState, StandaloneMeterRootError> {
        let callback = self.prepass.next_callback_index().value();
        if input.callback_index() != progress.callback_index
            || progress.callback_index.value() != callback
            || progress.source_cursor_before != self.source_cursor.source_cursor
            || progress.read_index_before != self.source_cursor.read_index
        {
            return Err(StandaloneMeterRootError::SourceCursorProgress);
        }

        let expected_phase = if callback < SELECTED_NORMAL_CALLBACKS {
            MeterCallbackPhase::NormalSource
        } else {
            MeterCallbackPhase::ZeroInputDrain
        };
        if input.phase() != expected_phase {
            return Err(StandaloneMeterRootError::SourceCursorProgress);
        }

        let expected_cursor_after;
        let expected_read_index_after;
        let expected_host_input_count;
        if expected_phase == MeterCallbackPhase::NormalSource {
            let remaining = self
                .source_samples
                .value()
                .checked_sub(self.source_cursor.source_cursor.value())
                .ok_or(StandaloneMeterRootError::SourceCursorProgress)?;
            let sample_count = remaining.min(SELECTED_CALLBACK_SAMPLES);
            expected_cursor_after = self
                .source_cursor
                .source_cursor
                .value()
                .checked_add(sample_count)
                .ok_or(StandaloneMeterRootError::SourceCursorProgress)?;
            expected_read_index_after = self
                .source_cursor
                .read_index
                .value()
                .checked_add(1)
                .ok_or(StandaloneMeterRootError::SourceCursorProgress)?;
            expected_host_input_count = sample_count;
        } else {
            expected_cursor_after = self.source_samples.value();
            expected_read_index_after = SELECTED_NORMAL_CALLBACKS;
            expected_host_input_count = 0;
        }

        if progress.source_cursor_after.value() != expected_cursor_after
            || progress.read_index_after.value() != expected_read_index_after
            || progress.host_input_count.value() != expected_host_input_count
        {
            return Err(StandaloneMeterRootError::SourceCursorProgress);
        }

        Ok(MeterSourceCursorState {
            source_cursor: progress.source_cursor_after,
            read_index: progress.read_index_after,
        })
    }

    /// Computes final scalar values without advancing clocks, hashing the root,
    /// or minting a receipt. Use the receipt proposal path when terminal identity
    /// and all receipt fields are required.
    pub fn prepare_finalization_values(
        &self,
    ) -> Result<StandaloneMeterFinalizationValuesProposal<'_>, StandaloneMeterRootError> {
        let schedule = self.finalization_schedule()?;
        let post_render = self.prepass.post_render_state();
        let measurement = compute_programme_measurement_values(
            post_render.true_peak(),
            post_render.integrated_meter(),
            post_render.ungated_mean(),
        )
        .map_err(StandaloneMeterRootError::Measurement)?;
        Ok(StandaloneMeterFinalizationValuesProposal {
            identity: &self.identity,
            meter_state: self,
            schedule,
            measurement,
        })
    }

    /// Hashes the complete current root before calling the selected read-only
    /// getters, then returns a typed receipt proposal without changing this root.
    pub fn prepare_measurement_receipt_proposal(
        &self,
    ) -> Result<StandaloneMeterMeasurementProposal, StandaloneMeterRootError> {
        let schedule = self.finalization_schedule()?;
        let terminal_meter_sha256 = Sha256::digest(self.canonical_preimage()?).into();
        let post_render = self.prepass.post_render_state();
        let measurement = compute_programme_measurement_values(
            post_render.true_peak(),
            post_render.integrated_meter(),
            post_render.ungated_mean(),
        )
        .map_err(StandaloneMeterRootError::Measurement)?;
        let receipt =
            ProgrammeMeasurementReceipt::from_verified_prepass(ProgrammeMeasurementReceiptInput {
                programme_id: self.identity.programme_id().clone(),
                attempt: self.identity.attempt(),
                input_sha256: *self.identity.input_sha256(),
                config_sha256: *self.identity.config_sha256(),
                dialogue_word: measurement.dialogue_word(),
                itu_word: measurement.itu_word(),
                true_peak_word: measurement.true_peak_word(),
                gating_kind: measurement.gating_kind().as_str(),
                selected_lufs_word: measurement.selected_word(),
                meter_terminal_sha256: terminal_meter_sha256,
                normal_callbacks: schedule.normal_callbacks().value(),
                zero_drains: schedule.zero_drains().value(),
                energy_windows: schedule.energy_windows(),
            })
            .map_err(StandaloneMeterRootError::MeasurementReceipt)?;

        Ok(StandaloneMeterMeasurementProposal {
            receipt,
            measurement,
            schedule,
            terminal_meter_sha256,
        })
    }

    fn finalization_schedule(&self) -> Result<StandaloneMeterSchedule, StandaloneMeterRootError> {
        let expected = StandaloneMeterSchedule::selected();
        if !self.prepass.ready_to_finalize()
            || self.prepass.normal_callback_count() != expected.normal_callbacks()
            || self.prepass.next_callback_index().value() != expected.meter_callbacks().value()
            || self.source_cursor.source_cursor.value() != self.source_samples.value()
            || self.source_cursor.read_index.value() != expected.normal_callbacks().value()
        {
            return Err(StandaloneMeterRootError::FinalizationNotReady);
        }

        let post_render = self.prepass.post_render_state();
        if !post_render
            .window_clock()
            .is_ready_to_finalize()
            .unwrap_or(false)
        {
            return Err(StandaloneMeterRootError::FinalizationNotReady);
        }
        let queue = post_render
            .sample_queue()
            .finalization_view()
            .map_err(|_| StandaloneMeterRootError::FinalizationNotReady)?;
        let complete_prefix = queue
            .complete_blocks()
            .value()
            .checked_mul(METER_BLOCK_SAMPLES)
            .ok_or(StandaloneMeterRootError::SelectedScheduleMismatch)?;
        let energy_remainder = u64::from(post_render.energy_window().samples_in_window());
        let energy_samples = complete_prefix
            .checked_sub(energy_remainder)
            .ok_or(StandaloneMeterRootError::SelectedScheduleMismatch)?;
        if energy_samples % METER_ENERGY_WINDOW_SAMPLES != 0 {
            return Err(StandaloneMeterRootError::SelectedScheduleMismatch);
        }
        let energy_windows = energy_samples / METER_ENERGY_WINDOW_SAMPLES;
        if energy_windows != self.completed_energy_windows.value() {
            return Err(StandaloneMeterRootError::SelectedScheduleMismatch);
        }
        let actual = StandaloneMeterSchedule {
            normal_callbacks: self.prepass.normal_callback_count(),
            zero_drains: MeterCallbackCount::new(
                self.prepass
                    .next_callback_index()
                    .value()
                    .checked_sub(self.prepass.normal_callback_count().value())
                    .ok_or(StandaloneMeterRootError::SelectedScheduleMismatch)?,
            ),
            meter_callbacks: MeterCallbackCount::new(self.prepass.next_callback_index().value()),
            published_samples: queue.published_samples(),
            source_padding_samples: MeterSampleCount::new(
                queue
                    .published_samples()
                    .value()
                    .checked_sub(self.source_samples.value())
                    .ok_or(StandaloneMeterRootError::SelectedScheduleMismatch)?,
            ),
            complete_blocks: queue.complete_blocks(),
            complete_prefix_samples: MeterSampleCount::new(complete_prefix),
            unused_block_tail_samples: queue.raw_tail_samples(),
            energy_windows,
            energy_samples: MeterSampleCount::new(energy_samples),
            energy_window_remainder: MeterSampleCount::new(energy_remainder),
        };
        if actual != expected || queue.consumed_samples() != expected.complete_prefix_samples() {
            return Err(StandaloneMeterRootError::SelectedScheduleMismatch);
        }
        Ok(actual)
    }
}

/// Noncommitting callback proposal. The parent owns any replacement decision.
#[must_use]
pub struct StandaloneMeterCallbackProposal {
    after: StandaloneMeterRoot,
    receipt: StandaloneMeterCallbackReceipt,
}

impl StandaloneMeterCallbackProposal {
    pub const fn after(&self) -> &StandaloneMeterRoot {
        &self.after
    }

    pub const fn receipt(&self) -> &StandaloneMeterCallbackReceipt {
        &self.receipt
    }

    pub fn into_parts(self) -> (StandaloneMeterRoot, StandaloneMeterCallbackReceipt) {
        (self.after, self.receipt)
    }
}

/// Local meter callback receipt; no GTC owner digest or commit authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandaloneMeterCallbackReceipt {
    identity: StandaloneMeterIdentity,
    callback_index: MeterCallbackIndex,
    next_callback_index: MeterCallbackIndex,
    lfe_output_origin: Option<MeterLfeCallbackOrigin>,
    post_render: PostRenderMeterReceipt,
}

impl StandaloneMeterCallbackReceipt {
    pub fn identity(&self) -> &StandaloneMeterIdentity {
        &self.identity
    }

    pub const fn callback_index(&self) -> MeterCallbackIndex {
        self.callback_index
    }

    pub const fn next_callback_index(&self) -> MeterCallbackIndex {
        self.next_callback_index
    }

    pub const fn lfe_output_origin(&self) -> Option<MeterLfeCallbackOrigin> {
        self.lfe_output_origin
    }

    pub const fn post_render(&self) -> PostRenderMeterReceipt {
        self.post_render
    }
}

/// Final scalar projection plus a borrow of the actual terminal meter state.
///
/// No ProgrammeMeasurementReceipt, READY transition, or terminal hash is
/// emitted until the standalone canonical-state reference is bound.
#[must_use]
pub struct StandaloneMeterFinalizationValuesProposal<'a> {
    identity: &'a StandaloneMeterIdentity,
    meter_state: &'a StandaloneMeterRoot,
    schedule: StandaloneMeterSchedule,
    measurement: ProgrammeMeasurementValue,
}

impl StandaloneMeterFinalizationValuesProposal<'_> {
    pub fn identity(&self) -> &StandaloneMeterIdentity {
        self.identity
    }

    /// Actual in-memory Meter root, not a serialized GTC state_ref.
    pub const fn meter_state(&self) -> &StandaloneMeterRoot {
        self.meter_state
    }

    pub const fn schedule(&self) -> StandaloneMeterSchedule {
        self.schedule
    }

    pub fn measurement(&self) -> &ProgrammeMeasurementValue {
        &self.measurement
    }
}

/// Terminal measurement receipt candidate. The parent still owns installation,
/// GTC generation changes, READY, and the programme root swap.
#[must_use]
pub struct StandaloneMeterMeasurementProposal {
    receipt: ProgrammeMeasurementReceipt,
    measurement: ProgrammeMeasurementValue,
    schedule: StandaloneMeterSchedule,
    terminal_meter_sha256: [u8; 32],
}

impl StandaloneMeterMeasurementProposal {
    pub const fn receipt(&self) -> &ProgrammeMeasurementReceipt {
        &self.receipt
    }

    pub const fn measurement(&self) -> &ProgrammeMeasurementValue {
        &self.measurement
    }

    pub const fn schedule(&self) -> StandaloneMeterSchedule {
        self.schedule
    }

    pub const fn terminal_meter_sha256(&self) -> &[u8; 32] {
        &self.terminal_meter_sha256
    }

    pub fn into_parts(self) -> (ProgrammeMeasurementReceipt, ProgrammeMeasurementValue) {
        (self.receipt, self.measurement)
    }
}

fn lowercase_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn sort_canonical_json(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(sort_canonical_json).collect()),
        Value::Object(values) => {
            let sorted: BTreeMap<_, _> = values
                .into_iter()
                .map(|(key, value)| (key, sort_canonical_json(value)))
                .collect();
            Value::Object(sorted.into_iter().collect())
        }
        scalar => scalar,
    }
}

#[cfg(test)]
mod tests;
