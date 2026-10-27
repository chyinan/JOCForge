// pattern: Imperative Shell
//! Standalone selected Meter owner around the existing independent prepass leaves.

use super::{
    k_matrix::KMatrixProvider,
    lfe_delay_ring::{MeterFrontendCallbackInput, MeterLfeCallbackOrigin},
    meter_prepass::{IndependentMeterPrepass, IndependentMeterPrepassError},
    post_render_meter::PostRenderMeterReceipt,
    programme_measurement::{
        ProgrammeMeasurementError, ProgrammeMeasurementValue, compute_programme_measurement_values,
    },
    sample_block_queue::{
        MeterBlockCount, MeterCallbackCount, MeterCallbackIndex, MeterSampleCount,
        MeterSourceSampleCount,
    },
    shared_gain_delay::SharedGainDelayParameters,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandaloneMeterRootError {
    UnsupportedSourceExtent,
    AttemptOverflow,
    AttemptSequence,
    CallbackClock,
    FinalizationNotReady,
    SelectedScheduleMismatch,
    Prepass(IndependentMeterPrepassError),
    Measurement(ProgrammeMeasurementError),
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
    gain_parameters: SharedGainDelayParameters,
    k_matrix: KMatrixProvider,
    prepass: IndependentMeterPrepass,
}

impl StandaloneMeterRoot {
    /// Constructs the fixed selected standalone meter from admitted immutable values.
    pub fn cold_selected(
        identity: StandaloneMeterIdentity,
        source_samples: MeterSourceSampleCount,
        gain_parameters: SharedGainDelayParameters,
        k_matrix: KMatrixProvider,
    ) -> Result<Self, StandaloneMeterRootError> {
        if source_samples.value() != SELECTED_SOURCE_SAMPLES {
            return Err(StandaloneMeterRootError::UnsupportedSourceExtent);
        }
        let prepass = IndependentMeterPrepass::cold(
            source_samples,
            gain_parameters.clone(),
            k_matrix.clone(),
        )
        .map_err(StandaloneMeterRootError::Prepass)?;
        Ok(Self {
            identity,
            source_samples,
            gain_parameters,
            k_matrix,
            prepass,
        })
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

    /// Stages one normal or fresh-zero callback and returns an owned after-root.
    /// The before-root is unchanged; root installation belongs to its parent.
    pub fn prepare_callback(
        &self,
        input: MeterFrontendCallbackInput,
    ) -> Result<StandaloneMeterCallbackProposal, StandaloneMeterRootError> {
        let before_callback = self.prepass.next_callback_index();
        if before_callback.value() >= SELECTED_METER_CALLBACKS {
            return Err(StandaloneMeterRootError::CallbackClock);
        }

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
            gain_parameters: self.gain_parameters.clone(),
            k_matrix: self.k_matrix.clone(),
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
    pub fn full_reset(
        self,
        identity: StandaloneMeterIdentity,
        source_samples: MeterSourceSampleCount,
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
        Self::cold_selected(identity, source_samples, gain_parameters, k_matrix)
    }

    /// Computes final scalar values without advancing clocks or minting a receipt.
    ///
    /// The returned object borrows the actual terminal meter root. It is not a
    /// GTC OwnerReceipt: meter_terminal_sha256 remains unavailable until the
    /// standalone canonical-state reference is implemented.
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

    fn finalization_schedule(&self) -> Result<StandaloneMeterSchedule, StandaloneMeterRootError> {
        let expected = StandaloneMeterSchedule::selected();
        if !self.prepass.ready_to_finalize()
            || self.prepass.normal_callback_count() != expected.normal_callbacks()
            || self.prepass.next_callback_index().value() != expected.meter_callbacks().value()
        {
            return Err(StandaloneMeterRootError::FinalizationNotReady);
        }

        let post_render = self.prepass.post_render_state();
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
            energy_windows: energy_samples / METER_ENERGY_WINDOW_SAMPLES,
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

#[cfg(test)]
mod tests;
