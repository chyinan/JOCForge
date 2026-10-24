// pattern: Functional Core
//! Independent-meter sample publication and complete-block queue.
//!
//! This module owns the post-front-end publication clock and the single
//! synchronized six-lane pending sample buffer. It stages complete 1024-sample
//! blocks without committing them. OAR, gain-delay, peak, K-filter, energy,
//! integrated-meter, final measurement, and programme commit authority remain
//! outside this boundary.

pub const METER_SAMPLE_LANES: usize = 6;
pub const METER_CALLBACK_SAMPLES: usize = 1_536;
pub const METER_COMPLETE_BLOCK_SAMPLES: usize = 1_024;

const METER_CALLBACK_SAMPLES_U64: u64 = 1_536;
const METER_COMPLETE_BLOCK_SAMPLES_U64: u64 = 1_024;
const METER_STARTUP_SAMPLES: u64 = 3_104;
const METER_ZERO_DRAIN_CALLBACKS: u64 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampleBlockQueueError {
    InvalidSourceSampleCount,
    ArithmeticOverflow,
    CallbackClockMismatch,
    UpstreamGenerationMismatch,
    CallbackPhaseMismatch,
    CallbackSequenceComplete,
    InputShape,
    InputNonFinite,
    CapacityUnavailable,
    StateInvalid,
    FinalizationNotReady,
}

macro_rules! clock_type {
    ($name:ident, $word:ty) => {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name($word);

        impl $name {
            pub const fn new(value: $word) -> Self {
                Self(value)
            }

            pub const fn value(self) -> $word {
                self.0
            }
        }
    };
}

clock_type!(MeterCallbackIndex, u64);
clock_type!(MeterCallbackCount, u64);
clock_type!(MeterUpstreamGeneration, u64);
clock_type!(MeterSampleCount, u64);
clock_type!(MeterSamplePosition, u64);
clock_type!(MeterBlockIndex, u64);
clock_type!(MeterBlockCount, u64);
clock_type!(SampleBlockFillCursor, u16);

/// Positive source extent used only to determine the normal-callback count.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MeterSourceSampleCount(u64);

impl MeterSourceSampleCount {
    pub fn new(value: u64) -> Result<Self, SampleBlockQueueError> {
        if value == 0 || value > i64::MAX as u64 {
            Err(SampleBlockQueueError::InvalidSourceSampleCount)
        } else {
            Ok(Self(value))
        }
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterCallbackPhase {
    NormalSource,
    ZeroInputDrain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterSampleLane {
    Left,
    Right,
    Centre,
    LowFrequencyEffects,
    LeftSurround,
    RightSurround,
}

impl MeterSampleLane {
    #[cfg(test)]
    const ALL: [Self; METER_SAMPLE_LANES] = [
        Self::Left,
        Self::Right,
        Self::Centre,
        Self::LowFrequencyEffects,
        Self::LeftSurround,
        Self::RightSurround,
    ];

    const fn offset(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
            Self::Centre => 2,
            Self::LowFrequencyEffects => 3,
            Self::LeftSurround => 4,
            Self::RightSurround => 5,
        }
    }
}

/// One actual six-lane post-front-end callback.
///
/// Each word is the exact bit pattern of one finite `f32` sample. Drain frames
/// are real upstream outputs and are not assumed to contain zero samples.
#[derive(Clone, Copy, Debug)]
pub struct MeterCallbackFrame<'a> {
    callback_index: MeterCallbackIndex,
    upstream_generation: MeterUpstreamGeneration,
    phase: MeterCallbackPhase,
    sample_words: [&'a [u32]; METER_SAMPLE_LANES],
}

impl<'a> MeterCallbackFrame<'a> {
    pub const fn new(
        callback_index: MeterCallbackIndex,
        upstream_generation: MeterUpstreamGeneration,
        phase: MeterCallbackPhase,
        sample_words: [&'a [u32]; METER_SAMPLE_LANES],
    ) -> Self {
        Self {
            callback_index,
            upstream_generation,
            phase,
            sample_words,
        }
    }
}

/// Actual samples for one complete synchronized six-lane block.
#[derive(Debug, Eq, PartialEq)]
pub struct CompleteSampleBlock {
    block_index: MeterBlockIndex,
    logical_begin: MeterSamplePosition,
    logical_end: MeterSamplePosition,
    sample_words: [Vec<u32>; METER_SAMPLE_LANES],
}

impl CompleteSampleBlock {
    pub const fn block_index(&self) -> MeterBlockIndex {
        self.block_index
    }

    pub const fn logical_begin(&self) -> MeterSamplePosition {
        self.logical_begin
    }

    pub const fn logical_end(&self) -> MeterSamplePosition {
        self.logical_end
    }

    pub fn lane(&self, lane: MeterSampleLane) -> &[u32] {
        &self.sample_words[lane.offset()]
    }

    pub fn lanes(&self) -> [&[u32]; METER_SAMPLE_LANES] {
        std::array::from_fn(|lane| self.sample_words[lane].as_slice())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampleQueueCallbackReceipt {
    callback_index: MeterCallbackIndex,
    upstream_generation: MeterUpstreamGeneration,
    phase: MeterCallbackPhase,
    skipped_samples: MeterSampleCount,
    published_begin: MeterSamplePosition,
    published_end: MeterSamplePosition,
    completed_blocks: MeterBlockCount,
}

impl SampleQueueCallbackReceipt {
    pub const fn callback_index(&self) -> MeterCallbackIndex {
        self.callback_index
    }

    pub const fn upstream_generation(&self) -> MeterUpstreamGeneration {
        self.upstream_generation
    }

    pub const fn phase(&self) -> MeterCallbackPhase {
        self.phase
    }

    pub const fn skipped_samples(&self) -> MeterSampleCount {
        self.skipped_samples
    }

    pub const fn published_begin(&self) -> MeterSamplePosition {
        self.published_begin
    }

    pub const fn published_end(&self) -> MeterSamplePosition {
        self.published_end
    }

    pub const fn completed_blocks(&self) -> MeterBlockCount {
        self.completed_blocks
    }

    pub const fn published_samples(&self) -> MeterSampleCount {
        MeterSampleCount(self.published_end.0 - self.published_begin.0)
    }
}

/// Persistent queue and publication-clock state for the independent meter.
///
/// `pending_sample_words` is the only raw PCM tail owned by this boundary. All
/// lanes always share `block_fill_cursor`; no per-lane cursor exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SampleBlockQueueState {
    source_samples: MeterSourceSampleCount,
    normal_callback_count: MeterCallbackCount,
    expected_published_samples: MeterSampleCount,
    next_callback_index: MeterCallbackIndex,
    last_upstream_generation: Option<MeterUpstreamGeneration>,
    startup_remaining: MeterSampleCount,
    scheduler_pending: MeterSampleCount,
    published_samples: MeterSampleCount,
    pcm_enqueued_samples: MeterSampleCount,
    pcm_consumed_samples: MeterSampleCount,
    complete_block_count: MeterBlockCount,
    block_fill_cursor: SampleBlockFillCursor,
    pending_sample_words: [Vec<u32>; METER_SAMPLE_LANES],
}

impl SampleBlockQueueState {
    pub fn cold(source_samples: MeterSourceSampleCount) -> Result<Self, SampleBlockQueueError> {
        let normal_callbacks = source_samples
            .value()
            .checked_sub(1)
            .and_then(|samples| samples.checked_div(METER_CALLBACK_SAMPLES_U64))
            .and_then(|callbacks| callbacks.checked_add(1))
            .ok_or(SampleBlockQueueError::ArithmeticOverflow)?;
        let expected_published_samples = normal_callbacks
            .checked_mul(METER_CALLBACK_SAMPLES_U64)
            .ok_or(SampleBlockQueueError::ArithmeticOverflow)?;
        normal_callbacks
            .checked_add(METER_ZERO_DRAIN_CALLBACKS)
            .and_then(|callbacks| callbacks.checked_mul(METER_CALLBACK_SAMPLES_U64))
            .ok_or(SampleBlockQueueError::ArithmeticOverflow)?;

        let state = Self {
            source_samples,
            normal_callback_count: MeterCallbackCount(normal_callbacks),
            expected_published_samples: MeterSampleCount(expected_published_samples),
            next_callback_index: MeterCallbackIndex(0),
            last_upstream_generation: None,
            startup_remaining: MeterSampleCount(METER_STARTUP_SAMPLES),
            scheduler_pending: MeterSampleCount(0),
            published_samples: MeterSampleCount(0),
            pcm_enqueued_samples: MeterSampleCount(0),
            pcm_consumed_samples: MeterSampleCount(0),
            complete_block_count: MeterBlockCount(0),
            block_fill_cursor: SampleBlockFillCursor(0),
            pending_sample_words: std::array::from_fn(|_| Vec::new()),
        };
        state.validate()?;
        Ok(state)
    }

    pub fn reset(&mut self) {
        self.next_callback_index = MeterCallbackIndex(0);
        self.last_upstream_generation = None;
        self.startup_remaining = MeterSampleCount(METER_STARTUP_SAMPLES);
        self.scheduler_pending = MeterSampleCount(0);
        self.published_samples = MeterSampleCount(0);
        self.pcm_enqueued_samples = MeterSampleCount(0);
        self.pcm_consumed_samples = MeterSampleCount(0);
        self.complete_block_count = MeterBlockCount(0);
        self.block_fill_cursor = SampleBlockFillCursor(0);
        for lane in &mut self.pending_sample_words {
            lane.clear();
        }
    }

    pub const fn source_samples(&self) -> MeterSourceSampleCount {
        self.source_samples
    }

    pub const fn normal_callback_count(&self) -> MeterCallbackCount {
        self.normal_callback_count
    }

    pub const fn expected_published_samples(&self) -> MeterSampleCount {
        self.expected_published_samples
    }

    pub const fn source_padding_samples(&self) -> MeterSampleCount {
        MeterSampleCount(self.expected_published_samples.0 - self.source_samples.0)
    }

    pub const fn next_callback_index(&self) -> MeterCallbackIndex {
        self.next_callback_index
    }

    pub const fn last_upstream_generation(&self) -> Option<MeterUpstreamGeneration> {
        self.last_upstream_generation
    }

    pub const fn startup_remaining(&self) -> MeterSampleCount {
        self.startup_remaining
    }

    pub const fn scheduler_pending(&self) -> MeterSampleCount {
        self.scheduler_pending
    }

    pub const fn published_samples(&self) -> MeterSampleCount {
        self.published_samples
    }

    pub const fn pcm_enqueued_samples(&self) -> MeterSampleCount {
        self.pcm_enqueued_samples
    }

    pub const fn pcm_consumed_samples(&self) -> MeterSampleCount {
        self.pcm_consumed_samples
    }

    pub const fn complete_block_count(&self) -> MeterBlockCount {
        self.complete_block_count
    }

    pub const fn block_fill_cursor(&self) -> SampleBlockFillCursor {
        self.block_fill_cursor
    }

    pub fn pending_lanes(&self) -> [&[u32]; METER_SAMPLE_LANES] {
        std::array::from_fn(|lane| self.pending_sample_words[lane].as_slice())
    }

    /// Returns the terminal queue view without adding a block or clearing a
    /// raw tail. Numeric leaf snapshots are finalized by the meter parent.
    pub fn finalization_view(
        &self,
    ) -> Result<SampleQueueFinalizationView<'_>, SampleBlockQueueError> {
        self.validate()?;
        if self.next_callback_index.0 != self.total_callback_count()? {
            return Err(SampleBlockQueueError::FinalizationNotReady);
        }
        Ok(SampleQueueFinalizationView { state: self })
    }

    fn total_callback_count(&self) -> Result<u64, SampleBlockQueueError> {
        self.normal_callback_count
            .0
            .checked_add(METER_ZERO_DRAIN_CALLBACKS)
            .ok_or(SampleBlockQueueError::ArithmeticOverflow)
    }

    fn expected_phase(&self) -> Result<MeterCallbackPhase, SampleBlockQueueError> {
        if self.next_callback_index.0 < self.normal_callback_count.0 {
            Ok(MeterCallbackPhase::NormalSource)
        } else if self.next_callback_index.0 < self.total_callback_count()? {
            Ok(MeterCallbackPhase::ZeroInputDrain)
        } else {
            Err(SampleBlockQueueError::CallbackSequenceComplete)
        }
    }

    fn validate(&self) -> Result<(), SampleBlockQueueError> {
        let total_callbacks = self.total_callback_count()?;
        if self.source_samples.0 == 0
            || self.normal_callback_count.0 == 0
            || self.next_callback_index.0 > total_callbacks
            || self.startup_remaining.0 > METER_STARTUP_SAMPLES
            || u64::from(self.block_fill_cursor.0) >= METER_COMPLETE_BLOCK_SAMPLES_U64
        {
            return Err(SampleBlockQueueError::StateInvalid);
        }

        let expected_last_generation = if self.next_callback_index.0 == 0 {
            None
        } else {
            Some(MeterUpstreamGeneration(self.next_callback_index.0))
        };
        if self.last_upstream_generation != expected_last_generation {
            return Err(SampleBlockQueueError::StateInvalid);
        }

        let elapsed_samples = self
            .next_callback_index
            .0
            .checked_mul(METER_CALLBACK_SAMPLES_U64)
            .ok_or(SampleBlockQueueError::StateInvalid)?;
        let expected_startup = METER_STARTUP_SAMPLES.saturating_sub(elapsed_samples);
        if self.startup_remaining.0 != expected_startup {
            return Err(SampleBlockQueueError::StateInvalid);
        }

        let normal_callbacks_seen = self.next_callback_index.0.min(self.normal_callback_count.0);
        let scheduled_samples = normal_callbacks_seen
            .checked_mul(METER_CALLBACK_SAMPLES_U64)
            .ok_or(SampleBlockQueueError::StateInvalid)?;
        if self
            .published_samples
            .0
            .checked_add(self.scheduler_pending.0)
            != Some(scheduled_samples)
            || self.pcm_enqueued_samples != self.published_samples
        {
            return Err(SampleBlockQueueError::StateInvalid);
        }

        let consumed_from_blocks = self
            .complete_block_count
            .0
            .checked_mul(METER_COMPLETE_BLOCK_SAMPLES_U64)
            .ok_or(SampleBlockQueueError::StateInvalid)?;
        if consumed_from_blocks != self.pcm_consumed_samples.0
            || self
                .pcm_consumed_samples
                .0
                .checked_add(u64::from(self.block_fill_cursor.0))
                != Some(self.pcm_enqueued_samples.0)
        {
            return Err(SampleBlockQueueError::StateInvalid);
        }

        let fill = usize::from(self.block_fill_cursor.0);
        if self.pending_sample_words.iter().any(|lane| {
            lane.len() != fill || lane.iter().any(|word| !f32::from_bits(*word).is_finite())
        }) {
            return Err(SampleBlockQueueError::StateInvalid);
        }

        if self.next_callback_index.0 == total_callbacks
            && (self.scheduler_pending.0 != 0
                || self.published_samples != self.expected_published_samples)
        {
            return Err(SampleBlockQueueError::StateInvalid);
        }
        Ok(())
    }
}

/// Borrowed terminal view of the still-owned raw sample tail.
#[derive(Clone, Copy, Debug)]
pub struct SampleQueueFinalizationView<'a> {
    state: &'a SampleBlockQueueState,
}

impl<'a> SampleQueueFinalizationView<'a> {
    pub const fn callback_count(&self) -> MeterCallbackCount {
        MeterCallbackCount(self.state.next_callback_index.0)
    }

    pub const fn last_upstream_generation(&self) -> Option<MeterUpstreamGeneration> {
        self.state.last_upstream_generation
    }

    pub const fn published_samples(&self) -> MeterSampleCount {
        self.state.published_samples
    }

    pub const fn consumed_samples(&self) -> MeterSampleCount {
        self.state.pcm_consumed_samples
    }

    pub const fn complete_blocks(&self) -> MeterBlockCount {
        self.state.complete_block_count
    }

    pub fn raw_tail_samples(&self) -> MeterSampleCount {
        MeterSampleCount(u64::from(self.state.block_fill_cursor.0))
    }

    pub fn raw_tail_lanes(&self) -> [&'a [u32]; METER_SAMPLE_LANES] {
        std::array::from_fn(|lane| self.state.pending_sample_words[lane].as_slice())
    }
}

/// Linear, non-committing result for one meter callback.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct SampleBlockQueueProposal {
    after: SampleBlockQueueState,
    completed_blocks: Vec<CompleteSampleBlock>,
    receipt: SampleQueueCallbackReceipt,
}

impl SampleBlockQueueProposal {
    pub const fn after(&self) -> &SampleBlockQueueState {
        &self.after
    }

    pub fn completed_blocks(&self) -> &[CompleteSampleBlock] {
        &self.completed_blocks
    }

    pub const fn receipt(&self) -> &SampleQueueCallbackReceipt {
        &self.receipt
    }

    pub fn into_parts(
        self,
    ) -> (
        SampleBlockQueueState,
        Vec<CompleteSampleBlock>,
        SampleQueueCallbackReceipt,
    ) {
        (self.after, self.completed_blocks, self.receipt)
    }
}

/// Stages one normal or drain callback without mutating the committed state.
pub fn stage_sample_block_queue_callback(
    before: &SampleBlockQueueState,
    frame: MeterCallbackFrame<'_>,
) -> Result<SampleBlockQueueProposal, SampleBlockQueueError> {
    before.validate()?;
    if frame.callback_index != before.next_callback_index {
        return Err(SampleBlockQueueError::CallbackClockMismatch);
    }
    let expected_generation = before
        .next_callback_index
        .0
        .checked_add(1)
        .map(MeterUpstreamGeneration)
        .ok_or(SampleBlockQueueError::ArithmeticOverflow)?;
    if frame.upstream_generation != expected_generation {
        return Err(SampleBlockQueueError::UpstreamGenerationMismatch);
    }
    if frame.phase != before.expected_phase()? {
        return Err(SampleBlockQueueError::CallbackPhaseMismatch);
    }
    validate_frame(&frame)?;

    let mut scheduler_pending = before.scheduler_pending.0;
    if frame.phase == MeterCallbackPhase::NormalSource {
        scheduler_pending = scheduler_pending
            .checked_add(METER_CALLBACK_SAMPLES_U64)
            .ok_or(SampleBlockQueueError::ArithmeticOverflow)?;
    }
    let skipped = before.startup_remaining.0.min(METER_CALLBACK_SAMPLES_U64);
    let next_startup = before.startup_remaining.0 - skipped;
    let available = METER_CALLBACK_SAMPLES_U64 - skipped;
    let valid = scheduler_pending.min(available);
    let next_scheduler_pending = scheduler_pending - valid;

    let published_end = before
        .published_samples
        .0
        .checked_add(valid)
        .ok_or(SampleBlockQueueError::ArithmeticOverflow)?;
    let fill_with_new_samples = u64::from(before.block_fill_cursor.0)
        .checked_add(valid)
        .ok_or(SampleBlockQueueError::ArithmeticOverflow)?;
    let completed_block_count = fill_with_new_samples / METER_COMPLETE_BLOCK_SAMPLES_U64;
    let next_fill = fill_with_new_samples % METER_COMPLETE_BLOCK_SAMPLES_U64;
    let next_complete_block_count = before
        .complete_block_count
        .0
        .checked_add(completed_block_count)
        .ok_or(SampleBlockQueueError::ArithmeticOverflow)?;
    let consumed_after = next_complete_block_count
        .checked_mul(METER_COMPLETE_BLOCK_SAMPLES_U64)
        .ok_or(SampleBlockQueueError::ArithmeticOverflow)?;
    let next_callback = before
        .next_callback_index
        .0
        .checked_add(1)
        .ok_or(SampleBlockQueueError::ArithmeticOverflow)?;

    let skipped_offset =
        usize::try_from(skipped).map_err(|_| SampleBlockQueueError::ArithmeticOverflow)?;
    let valid_length =
        usize::try_from(valid).map_err(|_| SampleBlockQueueError::ArithmeticOverflow)?;
    let published_end_offset = skipped_offset
        .checked_add(valid_length)
        .ok_or(SampleBlockQueueError::ArithmeticOverflow)?;
    let completed_capacity = usize::try_from(completed_block_count)
        .map_err(|_| SampleBlockQueueError::CapacityUnavailable)?;

    let mut after = before.clone();
    for lane in &mut after.pending_sample_words {
        lane.try_reserve(valid_length)
            .map_err(|_| SampleBlockQueueError::CapacityUnavailable)?;
    }
    for (destination, source) in after
        .pending_sample_words
        .iter_mut()
        .zip(frame.sample_words)
    {
        destination.extend_from_slice(&source[skipped_offset..published_end_offset]);
    }

    let mut completed_blocks = Vec::new();
    completed_blocks
        .try_reserve_exact(completed_capacity)
        .map_err(|_| SampleBlockQueueError::CapacityUnavailable)?;
    for block_offset in 0..completed_block_count {
        let block_index_value = before
            .complete_block_count
            .0
            .checked_add(block_offset)
            .ok_or(SampleBlockQueueError::ArithmeticOverflow)?;
        let logical_begin = block_index_value
            .checked_mul(METER_COMPLETE_BLOCK_SAMPLES_U64)
            .ok_or(SampleBlockQueueError::ArithmeticOverflow)?;
        let logical_end = logical_begin
            .checked_add(METER_COMPLETE_BLOCK_SAMPLES_U64)
            .ok_or(SampleBlockQueueError::ArithmeticOverflow)?;
        let sample_words = std::array::from_fn(|lane| {
            after.pending_sample_words[lane]
                .drain(..METER_COMPLETE_BLOCK_SAMPLES)
                .collect()
        });
        completed_blocks.push(CompleteSampleBlock {
            block_index: MeterBlockIndex(block_index_value),
            logical_begin: MeterSamplePosition(logical_begin),
            logical_end: MeterSamplePosition(logical_end),
            sample_words,
        });
    }

    after.next_callback_index = MeterCallbackIndex(next_callback);
    after.last_upstream_generation = Some(frame.upstream_generation);
    after.startup_remaining = MeterSampleCount(next_startup);
    after.scheduler_pending = MeterSampleCount(next_scheduler_pending);
    after.published_samples = MeterSampleCount(published_end);
    after.pcm_enqueued_samples = MeterSampleCount(published_end);
    after.pcm_consumed_samples = MeterSampleCount(consumed_after);
    after.complete_block_count = MeterBlockCount(next_complete_block_count);
    after.block_fill_cursor = SampleBlockFillCursor(
        u16::try_from(next_fill).map_err(|_| SampleBlockQueueError::ArithmeticOverflow)?,
    );
    after.validate()?;

    Ok(SampleBlockQueueProposal {
        after,
        completed_blocks,
        receipt: SampleQueueCallbackReceipt {
            callback_index: frame.callback_index,
            upstream_generation: frame.upstream_generation,
            phase: frame.phase,
            skipped_samples: MeterSampleCount(skipped),
            published_begin: MeterSamplePosition(before.published_samples.0),
            published_end: MeterSamplePosition(published_end),
            completed_blocks: MeterBlockCount(completed_block_count),
        },
    })
}

fn validate_frame(frame: &MeterCallbackFrame<'_>) -> Result<(), SampleBlockQueueError> {
    if frame
        .sample_words
        .iter()
        .any(|lane| lane.len() != METER_CALLBACK_SAMPLES)
    {
        return Err(SampleBlockQueueError::InputShape);
    }
    if frame
        .sample_words
        .iter()
        .flat_map(|lane| lane.iter())
        .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(SampleBlockQueueError::InputNonFinite);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED_N9217: [(u64, u64, u64, u64, u64, u64, u64); 10] = [
        (1, 1_568, 1_536, 0, 0, 0, 0),
        (2, 32, 3_072, 0, 0, 0, 0),
        (3, 0, 3_104, 1_504, 1_024, 480, 1),
        (4, 0, 3_104, 3_040, 2_048, 992, 2),
        (5, 0, 3_104, 4_576, 4_096, 480, 4),
        (6, 0, 3_104, 6_112, 5_120, 992, 5),
        (7, 0, 3_104, 7_648, 7_168, 480, 7),
        (8, 0, 1_568, 9_184, 8_192, 992, 8),
        (9, 0, 32, 10_720, 10_240, 480, 10),
        (10, 0, 0, 10_752, 10_240, 512, 10),
    ];

    fn callback_words(generation: u64) -> [Vec<u32>; METER_SAMPLE_LANES] {
        std::array::from_fn(|lane| {
            (0..METER_CALLBACK_SAMPLES)
                .map(|sample| {
                    let seed = (generation * 97
                        + u64::try_from(lane).unwrap() * 31
                        + u64::try_from(sample).unwrap())
                        % 2_047;
                    let centered = i32::try_from(seed).unwrap() - 1_023;
                    (centered as f32 / 2_048.0).to_bits()
                })
                .collect()
        })
    }

    fn views(rows: &[Vec<u32>; METER_SAMPLE_LANES]) -> [&[u32]; METER_SAMPLE_LANES] {
        std::array::from_fn(|lane| rows[lane].as_slice())
    }

    fn expected_phase(callback: u64, normal_callbacks: u64) -> MeterCallbackPhase {
        if callback < normal_callbacks {
            MeterCallbackPhase::NormalSource
        } else {
            MeterCallbackPhase::ZeroInputDrain
        }
    }

    #[test]
    fn approved_n9217_lifecycle_preserves_actual_values_and_terminal_tails() {
        let source_samples = MeterSourceSampleCount::new(9_217).unwrap();
        let mut state = SampleBlockQueueState::cold(source_samples).unwrap();
        let mut published: [Vec<u32>; METER_SAMPLE_LANES] = std::array::from_fn(|_| Vec::new());
        let mut consumed_samples = 0_u64;
        let mut energy_partial_samples = 0_u64;

        for (callback, expected) in EXPECTED_N9217.into_iter().enumerate() {
            let callback = u64::try_from(callback).unwrap();
            let generation = callback + 1;
            let phase = expected_phase(callback, 7);
            let rows = callback_words(generation);
            let proposal = stage_sample_block_queue_callback(
                &state,
                MeterCallbackFrame::new(
                    MeterCallbackIndex::new(callback),
                    MeterUpstreamGeneration::new(generation),
                    phase,
                    views(&rows),
                ),
            )
            .unwrap();

            let skip = match callback.cmp(&2) {
                std::cmp::Ordering::Less => METER_CALLBACK_SAMPLES,
                std::cmp::Ordering::Equal => 32,
                std::cmp::Ordering::Greater => 0,
            };
            let valid = usize::try_from(proposal.receipt().published_samples().value()).unwrap();
            for lane in 0..METER_SAMPLE_LANES {
                published[lane].extend_from_slice(&rows[lane][skip..skip + valid]);
            }

            assert_eq!(proposal.receipt().callback_index().value(), callback);
            assert_eq!(proposal.receipt().upstream_generation().value(), generation);
            assert_eq!(proposal.receipt().phase(), phase);
            assert_eq!(
                proposal.receipt().completed_blocks().value(),
                u64::try_from(proposal.completed_blocks().len()).unwrap()
            );

            for block in proposal.completed_blocks() {
                assert_eq!(block.block_index().value(), consumed_samples / 1_024);
                assert_eq!(block.logical_begin().value(), consumed_samples);
                assert_eq!(block.logical_end().value(), consumed_samples + 1_024);
                for lane in MeterSampleLane::ALL {
                    let offset = lane.offset();
                    let begin = usize::try_from(consumed_samples).unwrap();
                    assert_eq!(
                        block.lane(lane),
                        &published[offset][begin..begin + METER_COMPLETE_BLOCK_SAMPLES]
                    );
                }
                consumed_samples += 1_024;
                energy_partial_samples = (energy_partial_samples + 1_024) % 960;
            }

            state = proposal.into_parts().0;
            assert_eq!(state.next_callback_index().value(), expected.0);
            assert_eq!(state.startup_remaining().value(), expected.1);
            assert_eq!(state.scheduler_pending().value(), expected.2);
            assert_eq!(state.published_samples().value(), expected.3);
            assert_eq!(state.pcm_enqueued_samples().value(), expected.3);
            assert_eq!(state.pcm_consumed_samples().value(), expected.4);
            assert_eq!(u64::from(state.block_fill_cursor().value()), expected.5);
            assert_eq!(state.complete_block_count().value(), expected.6);
        }

        assert_eq!(state.normal_callback_count().value(), 7);
        assert_eq!(state.expected_published_samples().value(), 10_752);
        assert_eq!(state.source_padding_samples().value(), 1_535);
        assert_eq!(consumed_samples, 10_240);
        assert_eq!(energy_partial_samples, 640);

        for lane in MeterSampleLane::ALL {
            let offset = lane.offset();
            assert_eq!(
                state.pending_lanes()[offset],
                &published[offset][10_240..10_752]
            );
        }

        let before_finalize = state.clone();
        let finalization = state.finalization_view().unwrap();
        assert_eq!(finalization.callback_count().value(), 10);
        assert_eq!(finalization.last_upstream_generation().unwrap().value(), 10);
        assert_eq!(finalization.published_samples().value(), 10_752);
        assert_eq!(finalization.consumed_samples().value(), 10_240);
        assert_eq!(finalization.complete_blocks().value(), 10);
        assert_eq!(finalization.raw_tail_samples().value(), 512);
        assert_eq!(finalization.raw_tail_lanes(), state.pending_lanes());
        assert_eq!(state, before_finalize);
    }

    #[test]
    fn small_source_boundaries_include_padding_and_exactly_three_drains() {
        for (source_samples, expected_published) in
            [(1_u64, 1_536_u64), (1_536, 1_536), (1_537, 3_072)]
        {
            let mut state =
                SampleBlockQueueState::cold(MeterSourceSampleCount::new(source_samples).unwrap())
                    .unwrap();
            let total_callbacks = state.normal_callback_count().value() + 3;
            for callback in 0..total_callbacks {
                let rows = callback_words(callback + 1);
                let proposal = stage_sample_block_queue_callback(
                    &state,
                    MeterCallbackFrame::new(
                        MeterCallbackIndex::new(callback),
                        MeterUpstreamGeneration::new(callback + 1),
                        expected_phase(callback, state.normal_callback_count().value()),
                        views(&rows),
                    ),
                )
                .unwrap();
                state = proposal.into_parts().0;
            }
            assert_eq!(state.published_samples().value(), expected_published);
            assert_eq!(
                state.pcm_consumed_samples().value(),
                expected_published / 1_024 * 1_024
            );
            assert_eq!(
                u64::from(state.block_fill_cursor().value()),
                expected_published % 1_024
            );
            assert!(state.finalization_view().is_ok());
        }
    }

    #[test]
    fn invalid_clock_phase_shape_and_sample_fail_before_state_change() {
        let state =
            SampleBlockQueueState::cold(MeterSourceSampleCount::new(9_217).unwrap()).unwrap();
        let snapshot = state.clone();
        let rows = callback_words(1);

        assert!(matches!(
            stage_sample_block_queue_callback(
                &state,
                MeterCallbackFrame::new(
                    MeterCallbackIndex::new(1),
                    MeterUpstreamGeneration::new(1),
                    MeterCallbackPhase::NormalSource,
                    views(&rows),
                ),
            ),
            Err(SampleBlockQueueError::CallbackClockMismatch)
        ));
        assert!(matches!(
            stage_sample_block_queue_callback(
                &state,
                MeterCallbackFrame::new(
                    MeterCallbackIndex::new(0),
                    MeterUpstreamGeneration::new(2),
                    MeterCallbackPhase::NormalSource,
                    views(&rows),
                ),
            ),
            Err(SampleBlockQueueError::UpstreamGenerationMismatch)
        ));
        assert!(matches!(
            stage_sample_block_queue_callback(
                &state,
                MeterCallbackFrame::new(
                    MeterCallbackIndex::new(0),
                    MeterUpstreamGeneration::new(1),
                    MeterCallbackPhase::ZeroInputDrain,
                    views(&rows),
                ),
            ),
            Err(SampleBlockQueueError::CallbackPhaseMismatch)
        ));

        let mut short = callback_words(1);
        short[2].pop();
        assert!(matches!(
            stage_sample_block_queue_callback(
                &state,
                MeterCallbackFrame::new(
                    MeterCallbackIndex::new(0),
                    MeterUpstreamGeneration::new(1),
                    MeterCallbackPhase::NormalSource,
                    views(&short),
                ),
            ),
            Err(SampleBlockQueueError::InputShape)
        ));

        let mut nonfinite = callback_words(1);
        nonfinite[4][100] = f32::NAN.to_bits();
        assert!(matches!(
            stage_sample_block_queue_callback(
                &state,
                MeterCallbackFrame::new(
                    MeterCallbackIndex::new(0),
                    MeterUpstreamGeneration::new(1),
                    MeterCallbackPhase::NormalSource,
                    views(&nonfinite),
                ),
            ),
            Err(SampleBlockQueueError::InputNonFinite)
        ));
        assert!(matches!(
            state.finalization_view(),
            Err(SampleBlockQueueError::FinalizationNotReady)
        ));
        assert_eq!(state, snapshot);
    }

    #[test]
    fn reset_discards_all_prior_queue_values_and_restores_the_cold_clock() {
        let source_samples = MeterSourceSampleCount::new(9_217).unwrap();
        let cold = SampleBlockQueueState::cold(source_samples).unwrap();
        let rows = callback_words(1);
        let mut advanced = stage_sample_block_queue_callback(
            &cold,
            MeterCallbackFrame::new(
                MeterCallbackIndex::new(0),
                MeterUpstreamGeneration::new(1),
                MeterCallbackPhase::NormalSource,
                views(&rows),
            ),
        )
        .unwrap()
        .into_parts()
        .0;
        advanced.reset();
        assert_eq!(advanced, cold);
    }

    #[test]
    fn impossible_source_extent_is_rejected_before_state_construction() {
        assert!(matches!(
            MeterSourceSampleCount::new(0),
            Err(SampleBlockQueueError::InvalidSourceSampleCount)
        ));
        assert!(matches!(
            MeterSourceSampleCount::new(u64::MAX),
            Err(SampleBlockQueueError::InvalidSourceSampleCount)
        ));
    }
}
