// pattern: Functional Core
//! Independent F06W publication-clock state, separate from the sample queue.

use super::sample_block_queue::{
    METER_CALLBACK_SAMPLES, MeterCallbackCount, MeterCallbackIndex, MeterCallbackPhase,
    MeterSampleCount, MeterSourceSampleCount, MeterUpstreamGeneration, SampleQueueCallbackReceipt,
};

const CALLBACK_SAMPLES_U64: u64 = METER_CALLBACK_SAMPLES as u64;
const STARTUP_SAMPLES: u64 = 3_104;
const ZERO_INPUT_DRAINS: u64 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterWindowClockError {
    CallbackSequenceComplete,
    CallbackClockMismatch,
    UpstreamGenerationMismatch,
    CallbackPhaseMismatch,
    ReceiptSpanMismatch,
    ArithmeticOverflow,
    StateInvalid,
}

/// F06W startup, scheduler-pending and processed counters for the independent meter.
///
/// `processed_samples` is a distinct committed leaf from the sample queue's
/// `published_samples`; both advance from the same validated callback receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeterWindowClockState {
    source_samples: MeterSourceSampleCount,
    normal_callback_count: MeterCallbackCount,
    next_callback_index: MeterCallbackIndex,
    startup_remaining: MeterSampleCount,
    scheduler_pending: MeterSampleCount,
    processed_samples: MeterSampleCount,
}

impl MeterWindowClockState {
    /// Reconstructs the selected F06W clock from its source extent.
    pub fn cold(source_samples: MeterSourceSampleCount) -> Result<Self, MeterWindowClockError> {
        let normal_callbacks = normal_callback_count(source_samples.value())?;
        let state = Self {
            source_samples,
            normal_callback_count: MeterCallbackCount::new(normal_callbacks),
            next_callback_index: MeterCallbackIndex::new(0),
            startup_remaining: MeterSampleCount::new(STARTUP_SAMPLES),
            scheduler_pending: MeterSampleCount::new(0),
            processed_samples: MeterSampleCount::new(0),
        };
        state.validate()?;
        Ok(state)
    }

    /// Stages the F06W clock transition for the same actual span admitted by the queue.
    pub fn stage_callback(
        &self,
        queue_receipt: &SampleQueueCallbackReceipt,
    ) -> Result<Self, MeterWindowClockError> {
        self.validate()?;
        let total_callbacks = self.total_callback_count()?;
        if self.next_callback_index.value() >= total_callbacks {
            return Err(MeterWindowClockError::CallbackSequenceComplete);
        }
        if queue_receipt.callback_index() != self.next_callback_index {
            return Err(MeterWindowClockError::CallbackClockMismatch);
        }

        let callback_index = self.next_callback_index.value();
        let expected_generation = callback_index
            .checked_add(1)
            .ok_or(MeterWindowClockError::ArithmeticOverflow)?;
        if queue_receipt.upstream_generation() != MeterUpstreamGeneration::new(expected_generation)
        {
            return Err(MeterWindowClockError::UpstreamGenerationMismatch);
        }

        let normal = callback_index < self.normal_callback_count.value();
        let expected_phase = if normal {
            MeterCallbackPhase::NormalSource
        } else {
            MeterCallbackPhase::ZeroInputDrain
        };
        if queue_receipt.phase() != expected_phase {
            return Err(MeterWindowClockError::CallbackPhaseMismatch);
        }

        let skipped = self.startup_remaining.value().min(CALLBACK_SAMPLES_U64);
        if queue_receipt.skipped_samples().value() != skipped {
            return Err(MeterWindowClockError::ReceiptSpanMismatch);
        }
        let pending_before_publish = if normal {
            self.scheduler_pending
                .value()
                .checked_add(CALLBACK_SAMPLES_U64)
                .ok_or(MeterWindowClockError::ArithmeticOverflow)?
        } else {
            self.scheduler_pending.value()
        };
        let available = CALLBACK_SAMPLES_U64 - skipped;
        let valid = pending_before_publish.min(available);
        let processed_end = self
            .processed_samples
            .value()
            .checked_add(valid)
            .ok_or(MeterWindowClockError::ArithmeticOverflow)?;
        if queue_receipt.published_begin().value() != self.processed_samples.value()
            || queue_receipt.published_samples().value() != valid
            || queue_receipt.published_end().value() != processed_end
        {
            return Err(MeterWindowClockError::ReceiptSpanMismatch);
        }

        let next_callback_index = callback_index
            .checked_add(1)
            .ok_or(MeterWindowClockError::ArithmeticOverflow)?;
        let next = Self {
            source_samples: self.source_samples,
            normal_callback_count: self.normal_callback_count,
            next_callback_index: MeterCallbackIndex::new(next_callback_index),
            startup_remaining: MeterSampleCount::new(self.startup_remaining.value() - skipped),
            scheduler_pending: MeterSampleCount::new(pending_before_publish - valid),
            processed_samples: MeterSampleCount::new(processed_end),
        };
        next.validate()?;
        Ok(next)
    }

    pub const fn source_samples(&self) -> MeterSourceSampleCount {
        self.source_samples
    }

    pub const fn normal_callback_count(&self) -> MeterCallbackCount {
        self.normal_callback_count
    }

    pub const fn next_callback_index(&self) -> MeterCallbackIndex {
        self.next_callback_index
    }

    pub const fn startup_remaining(&self) -> MeterSampleCount {
        self.startup_remaining
    }

    pub const fn scheduler_pending(&self) -> MeterSampleCount {
        self.scheduler_pending
    }

    pub const fn processed_samples(&self) -> MeterSampleCount {
        self.processed_samples
    }

    pub fn is_ready_to_finalize(&self) -> Result<bool, MeterWindowClockError> {
        self.validate()?;
        Ok(
            self.next_callback_index.value() == self.total_callback_count()?
                && self.startup_remaining.value() == 0
                && self.scheduler_pending.value() == 0
                && self.processed_samples.value()
                    == self
                        .normal_callback_count
                        .value()
                        .checked_mul(CALLBACK_SAMPLES_U64)
                        .ok_or(MeterWindowClockError::ArithmeticOverflow)?,
        )
    }

    fn total_callback_count(&self) -> Result<u64, MeterWindowClockError> {
        self.normal_callback_count
            .value()
            .checked_add(ZERO_INPUT_DRAINS)
            .ok_or(MeterWindowClockError::ArithmeticOverflow)
    }

    fn validate(&self) -> Result<(), MeterWindowClockError> {
        let normal_callbacks = normal_callback_count(self.source_samples.value())?;
        let callback_index = self.next_callback_index.value();
        let elapsed_samples = callback_index
            .checked_mul(CALLBACK_SAMPLES_U64)
            .ok_or(MeterWindowClockError::ArithmeticOverflow)?;
        let normal_callbacks_elapsed = callback_index.min(normal_callbacks);
        let source_samples_enqueued = normal_callbacks_elapsed
            .checked_mul(CALLBACK_SAMPLES_U64)
            .ok_or(MeterWindowClockError::ArithmeticOverflow)?;
        let available_samples = elapsed_samples.saturating_sub(STARTUP_SAMPLES);
        let expected_processed = source_samples_enqueued.min(available_samples);
        let expected_pending = source_samples_enqueued - expected_processed;
        let expected_startup = STARTUP_SAMPLES.saturating_sub(elapsed_samples);
        if self.normal_callback_count.value() != normal_callbacks
            || callback_index > self.total_callback_count()?
            || self.startup_remaining.value() != expected_startup
            || self.scheduler_pending.value() != expected_pending
            || self.processed_samples.value() != expected_processed
        {
            return Err(MeterWindowClockError::StateInvalid);
        }
        Ok(())
    }
}

fn normal_callback_count(source_samples: u64) -> Result<u64, MeterWindowClockError> {
    source_samples
        .checked_add(CALLBACK_SAMPLES_U64 - 1)
        .map(|rounded| rounded / CALLBACK_SAMPLES_U64)
        .ok_or(MeterWindowClockError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::{MeterWindowClockError, MeterWindowClockState};
    use crate::meter::{
        output_schedule::calculate_meter_output_schedule,
        sample_block_queue::{
            METER_CALLBACK_SAMPLES, MeterCallbackFrame, MeterCallbackIndex, MeterCallbackPhase,
            MeterSourceSampleCount, MeterUpstreamGeneration, SampleBlockQueueState,
            stage_sample_block_queue_callback,
        },
    };

    fn advance_both_states(source_samples: u64) -> (MeterWindowClockState, SampleBlockQueueState) {
        let source_samples = MeterSourceSampleCount::new(source_samples).unwrap();
        let mut window_clock = MeterWindowClockState::cold(source_samples).unwrap();
        let mut queue = SampleBlockQueueState::cold(source_samples).unwrap();
        let normal_callbacks = window_clock.normal_callback_count().value();
        let total_callbacks = normal_callbacks + 3;
        let samples: [Vec<u32>; 6] = std::array::from_fn(|_| vec![0; METER_CALLBACK_SAMPLES]);

        for callback_index in 0..total_callbacks {
            let phase = if callback_index < normal_callbacks {
                MeterCallbackPhase::NormalSource
            } else {
                MeterCallbackPhase::ZeroInputDrain
            };
            let sample_words = std::array::from_fn(|lane| samples[lane].as_slice());
            let frame = MeterCallbackFrame::new(
                MeterCallbackIndex::new(callback_index),
                MeterUpstreamGeneration::new(callback_index + 1),
                phase,
                sample_words,
            );
            let proposal = stage_sample_block_queue_callback(&queue, frame).unwrap();
            let (queue_after, _, receipt) = proposal.into_parts();
            let window_after = window_clock.stage_callback(&receipt).unwrap();

            assert_eq!(
                window_after.next_callback_index(),
                queue_after.next_callback_index()
            );
            assert_eq!(
                window_after.startup_remaining(),
                queue_after.startup_remaining()
            );
            assert_eq!(
                window_after.scheduler_pending(),
                queue_after.scheduler_pending()
            );
            assert_eq!(
                window_after.processed_samples().value(),
                receipt.published_end().value()
            );
            assert_eq!(
                window_after.processed_samples(),
                queue_after.published_samples()
            );
            window_clock = window_after;
            queue = queue_after;
        }

        (window_clock, queue)
    }

    #[test]
    fn one_sample_schedule_keeps_processed_separate_from_queue_publication() {
        let (window_clock, queue) = advance_both_states(1);
        assert_eq!(window_clock.normal_callback_count().value(), 1);
        assert_eq!(window_clock.next_callback_index().value(), 4);
        assert_eq!(window_clock.processed_samples().value(), 1_536);
        assert_eq!(queue.published_samples().value(), 1_536);
        assert_eq!(window_clock.is_ready_to_finalize(), Ok(true));

        let cold = MeterWindowClockState::cold(MeterSourceSampleCount::new(1).unwrap()).unwrap();
        assert_eq!(cold.next_callback_index().value(), 0);
        assert_eq!(cold.startup_remaining().value(), 3_104);
        assert_eq!(cold.scheduler_pending().value(), 0);
        assert_eq!(cold.processed_samples().value(), 0);
    }

    #[test]
    fn seven_normal_three_drain_schedule_retains_raw_and_energy_tails() {
        let (window_clock, queue) = advance_both_states(9_217);
        let schedule = calculate_meter_output_schedule(9_217).unwrap();
        assert_eq!(window_clock.normal_callback_count().value(), 7);
        assert_eq!(window_clock.next_callback_index().value(), 10);
        assert_eq!(window_clock.processed_samples().value(), 10_752);
        assert_eq!(
            window_clock.processed_samples().value(),
            schedule.published_samples()
        );
        assert_eq!(queue.published_samples().value(), 10_752);
        assert_eq!(queue.pcm_consumed_samples().value(), 10_240);
        assert_eq!(queue.complete_block_count().value(), 10);
        assert_eq!(queue.pending_lanes()[0].len(), 512);
        assert!(queue.pending_lanes().iter().all(|lane| lane.len() == 512));
        assert_eq!(window_clock.is_ready_to_finalize(), Ok(true));
    }

    #[test]
    fn window_clock_rejects_a_replayed_queue_receipt() {
        let source_samples = MeterSourceSampleCount::new(1).unwrap();
        let window_clock = MeterWindowClockState::cold(source_samples).unwrap();
        let queue = SampleBlockQueueState::cold(source_samples).unwrap();
        let samples: [Vec<u32>; 6] = std::array::from_fn(|_| vec![0; METER_CALLBACK_SAMPLES]);
        let frame = MeterCallbackFrame::new(
            MeterCallbackIndex::new(0),
            MeterUpstreamGeneration::new(1),
            MeterCallbackPhase::NormalSource,
            std::array::from_fn(|lane| samples[lane].as_slice()),
        );
        let proposal = stage_sample_block_queue_callback(&queue, frame).unwrap();
        let receipt = *proposal.receipt();
        let after = window_clock.stage_callback(&receipt).unwrap();
        assert_eq!(
            after.stage_callback(&receipt),
            Err(MeterWindowClockError::CallbackClockMismatch)
        );
    }
}
