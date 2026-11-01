// pattern: Functional Core
//! Stages one normal ReaderEof owner transition from accepted input-clock values.

use jocforge_domain::{
    frontend::normal_control_snapshot::NormalControlSnapshotOutput,
    state::{Control38Snapshot, Host38Frame, ReaderEofError, ReaderEofPhase, ReaderEofState},
    timeline::{AdmSampleIndex, FrontendCallbackIndex, NormalCallbackCount},
    transaction::CallbackTransactionId,
};
pub use jocforge_domain::{
    state::{ReaderEofMachine, ReaderEofReplacementCandidate},
    transaction::GtcOwnerId,
};

const SELECTED_FRAME_SAMPLES: u32 = 1_536;
const SELECTED_HOST_CHANNELS: usize = 38;
const MAIN_CACHED_DRAIN_COUNT: u32 = 4;

/// Actual input samples accepted for one normal frontend callback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderEofActualSampleCount(u32);

impl ReaderEofActualSampleCount {
    pub fn new(value: u32) -> Result<Self, ReaderEofCallbackError> {
        if value == 0 || value > SELECTED_FRAME_SAMPLES {
            return Err(ReaderEofCallbackError::SampleCountMismatch);
        }
        Ok(Self(value))
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

/// Positive-zero padding accepted for one normal frontend callback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderEofPaddingSampleCount(u32);

impl ReaderEofPaddingSampleCount {
    pub fn new(value: u32) -> Result<Self, ReaderEofCallbackError> {
        if value >= SELECTED_FRAME_SAMPLES {
            return Err(ReaderEofCallbackError::SampleCountMismatch);
        }
        Ok(Self(value))
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

/// Accepted callback, source-window, and padding values for one ReaderEof transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderEofNormalCallbackClock {
    callback_index: FrontendCallbackIndex,
    first_sample: AdmSampleIndex,
    actual_samples: ReaderEofActualSampleCount,
    padding_samples: ReaderEofPaddingSampleCount,
    normal_callbacks: NormalCallbackCount,
}

impl ReaderEofNormalCallbackClock {
    pub fn new(
        callback_index: FrontendCallbackIndex,
        first_sample: AdmSampleIndex,
        actual_samples: ReaderEofActualSampleCount,
        padding_samples: ReaderEofPaddingSampleCount,
        normal_callbacks: NormalCallbackCount,
    ) -> Result<Self, ReaderEofCallbackError> {
        let after_callback = callback_index
            .value()
            .checked_add(1)
            .ok_or(ReaderEofCallbackError::CallbackOrder)?;
        if u64::from(actual_samples.value()) + u64::from(padding_samples.value())
            != u64::from(SELECTED_FRAME_SAMPLES)
            || normal_callbacks.value() != u64::from(after_callback)
        {
            return Err(ReaderEofCallbackError::SampleCountMismatch);
        }
        Ok(Self {
            callback_index,
            first_sample,
            actual_samples,
            padding_samples,
            normal_callbacks,
        })
    }

    pub const fn callback_index(self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn first_sample(self) -> AdmSampleIndex {
        self.first_sample
    }

    pub const fn actual_samples(self) -> ReaderEofActualSampleCount {
        self.actual_samples
    }

    pub const fn padding_samples(self) -> ReaderEofPaddingSampleCount {
        self.padding_samples
    }

    pub const fn normal_callbacks(self) -> NormalCallbackCount {
        self.normal_callbacks
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReaderEofCallbackError {
    CallbackOrder,
    SourceCursorMismatch,
    ControlMismatch,
    PcmShape,
    PcmPaddingMismatch,
    PcmNonFinite,
    SampleCountMismatch,
    SampleCursorOverflow,
    CachedDrainNormalCountMismatch,
    CachedDrainCallbackOrder,
    CachedDrainOutsideWindow,
    CachedDrainBudgetMismatch,
    CachedDrainPhaseMismatch,
    CachedDrainCacheMissing,
    CachedDrainClockOverflow,
    ReaderEof(ReaderEofError),
}

/// Stages ReaderEof with the exact normal F01E controls and physical PCM rows for this callback.
///
/// The bridge validates the accepted clock and row padding but does not decode, reorder, fill, or
/// otherwise change PCM. It returns a parent candidate and never installs the new owner root.
pub fn stage_reader_eof_normal_callback(
    before: &ReaderEofMachine,
    clock: ReaderEofNormalCallbackClock,
    physical_pcm_rows: &[Vec<u32>],
    controls: &NormalControlSnapshotOutput,
    parent_transaction_id: CallbackTransactionId,
) -> Result<ReaderEofReplacementCandidate, ReaderEofCallbackError> {
    let current = before.committed();
    if current.callback_index() != clock.callback_index()
        || u64::from(current.callback_index().value()) != current.normal_callbacks().value()
        || current.source_cursor_samples() != clock.first_sample().value()
    {
        return Err(ReaderEofCallbackError::CallbackOrder);
    }
    if controls.value_generation() != clock.callback_index()
        || controls.normal_origin_generation() != clock.callback_index()
        || controls.transaction_id().callback_index() != clock.callback_index()
        || controls.query_sample() != clock.first_sample().value()
        || controls.actual_samples() != clock.actual_samples().value()
        || controls.normal_callbacks() != clock.normal_callbacks()
        || controls.next_callback_index().value()
            != clock
                .callback_index()
                .value()
                .checked_add(1)
                .ok_or(ReaderEofCallbackError::CallbackOrder)?
        || controls.next_owner_generation()
            != u64::from(
                clock
                    .callback_index()
                    .value()
                    .checked_add(1)
                    .ok_or(ReaderEofCallbackError::CallbackOrder)?,
            )
    {
        return Err(ReaderEofCallbackError::ControlMismatch);
    }
    if physical_pcm_rows.len() != SELECTED_HOST_CHANNELS
        || physical_pcm_rows
            .iter()
            .any(|row| row.len() != SELECTED_FRAME_SAMPLES as usize)
    {
        return Err(ReaderEofCallbackError::PcmShape);
    }
    let actual_end = clock.actual_samples().value() as usize;
    if physical_pcm_rows
        .iter()
        .any(|row| row[actual_end..].iter().any(|word| *word != 0))
    {
        return Err(ReaderEofCallbackError::PcmPaddingMismatch);
    }
    if physical_pcm_rows.iter().any(|row| {
        row[..actual_end]
            .iter()
            .any(|word| !f32::from_bits(*word).is_finite())
    }) {
        return Err(ReaderEofCallbackError::PcmNonFinite);
    }
    let cursor_after = clock
        .first_sample()
        .value()
        .checked_add(u64::from(clock.actual_samples().value()))
        .ok_or(ReaderEofCallbackError::SampleCursorOverflow)?;
    let next_callback = clock
        .callback_index()
        .value()
        .checked_add(1)
        .ok_or(ReaderEofCallbackError::CallbackOrder)?;
    let host = Host38Frame::new(
        physical_pcm_rows
            .iter()
            .flat_map(|row| row.iter().copied().map(f32::from_bits))
            .collect(),
    )
    .map_err(ReaderEofCallbackError::ReaderEof)?;
    let controls = Control38Snapshot::new(controls.control38_carrier_words().to_vec())
        .map_err(ReaderEofCallbackError::ReaderEof)?;
    let after = ReaderEofState::from_parts(
        FrontendCallbackIndex::new(next_callback),
        cursor_after,
        clock.normal_callbacks(),
        4,
        Some(host),
        Some(controls),
        ReaderEofPhase::Normal,
    )
    .map_err(ReaderEofCallbackError::ReaderEof)?;
    let staged = before
        .stage(
            before
                .prepare(parent_transaction_id.as_str(), after)
                .map_err(ReaderEofCallbackError::ReaderEof)?,
        )
        .map_err(ReaderEofCallbackError::ReaderEof)?;
    before
        .replacement_candidate(staged, parent_transaction_id)
        .map_err(ReaderEofCallbackError::ReaderEof)
}

/// Stages one main cached-drain transition after all planned normal callbacks have committed.
///
/// The expected normal callback count must come from the admitted input extent. It distinguishes
/// the last normal callback from an arbitrary intermediate callback, since the committed reader
/// state records the number processed but not the planned total. Cached drains retain the last
/// normal PCM/control cache and source cursor while advancing only the callback/drain state.
pub fn stage_reader_eof_cached_drain(
    before: &ReaderEofMachine,
    callback_index: FrontendCallbackIndex,
    expected_normal_callbacks: NormalCallbackCount,
    transaction_id: CallbackTransactionId,
) -> Result<ReaderEofReplacementCandidate, ReaderEofCallbackError> {
    let committed = before.committed();
    let expected_count = expected_normal_callbacks.value();
    if expected_count == 0 || committed.normal_callbacks().value() != expected_count {
        return Err(ReaderEofCallbackError::CachedDrainNormalCountMismatch);
    }

    let drain_end = expected_count
        .checked_add(u64::from(MAIN_CACHED_DRAIN_COUNT))
        .ok_or(ReaderEofCallbackError::CachedDrainClockOverflow)?;
    if drain_end > u64::from(u32::MAX) {
        return Err(ReaderEofCallbackError::CachedDrainClockOverflow);
    }

    if committed.callback_index() != callback_index {
        return Err(ReaderEofCallbackError::CachedDrainCallbackOrder);
    }
    let callback_value = u64::from(callback_index.value());
    if callback_value < expected_count || callback_value >= drain_end {
        return Err(ReaderEofCallbackError::CachedDrainOutsideWindow);
    }

    let expected_remaining = u32::try_from(drain_end - callback_value)
        .map_err(|_| ReaderEofCallbackError::CachedDrainClockOverflow)?;
    if committed.remaining_cached_drains() != expected_remaining {
        return Err(ReaderEofCallbackError::CachedDrainBudgetMismatch);
    }

    let expected_phase = if callback_value == expected_count {
        ReaderEofPhase::Normal
    } else {
        ReaderEofPhase::CachedDrain
    };
    if committed.phase() != expected_phase {
        return Err(ReaderEofCallbackError::CachedDrainPhaseMismatch);
    }

    let last_normal_host38 = committed
        .last_normal_host38()
        .cloned()
        .ok_or(ReaderEofCallbackError::CachedDrainCacheMissing)?;
    let last_control_snapshot = committed
        .last_control_snapshot()
        .cloned()
        .ok_or(ReaderEofCallbackError::CachedDrainCacheMissing)?;
    let next_callback = callback_index
        .value()
        .checked_add(1)
        .ok_or(ReaderEofCallbackError::CachedDrainClockOverflow)?;
    let next_remaining = expected_remaining
        .checked_sub(1)
        .ok_or(ReaderEofCallbackError::CachedDrainBudgetMismatch)?;
    let after = ReaderEofState::from_parts(
        FrontendCallbackIndex::new(next_callback),
        committed.source_cursor_samples(),
        committed.normal_callbacks(),
        next_remaining,
        Some(last_normal_host38),
        Some(last_control_snapshot),
        ReaderEofPhase::CachedDrain,
    )
    .map_err(ReaderEofCallbackError::ReaderEof)?;
    let staged = before
        .stage(
            before
                .prepare(transaction_id.as_str(), after)
                .map_err(ReaderEofCallbackError::ReaderEof)?,
        )
        .map_err(ReaderEofCallbackError::ReaderEof)?;
    before
        .replacement_candidate(staged, transaction_id)
        .map_err(ReaderEofCallbackError::ReaderEof)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CACHED_HOST_WORDS: usize = 38 * SELECTED_FRAME_SAMPLES as usize;
    const CACHED_CONTROL_WORDS: usize = 38 * 13;

    fn reader_after_normal_callback(
        remaining_cached_drains: u32,
        phase: ReaderEofPhase,
        include_cached_inputs: bool,
    ) -> ReaderEofMachine {
        let before = ReaderEofMachine::cold();
        let last_normal_host38 =
            include_cached_inputs.then(|| Host38Frame::new(vec![0.0; CACHED_HOST_WORDS]).unwrap());
        let last_control_snapshot = include_cached_inputs
            .then(|| Control38Snapshot::new(vec![0; CACHED_CONTROL_WORDS]).unwrap());
        let after = ReaderEofState::from_parts(
            FrontendCallbackIndex::new(1),
            u64::from(SELECTED_FRAME_SAMPLES),
            NormalCallbackCount::new(1),
            remaining_cached_drains,
            last_normal_host38,
            last_control_snapshot,
            phase,
        )
        .unwrap();
        let staged = before
            .stage(before.prepare("normal-g0", after).unwrap())
            .unwrap();
        before
            .replacement_candidate(staged, CallbackTransactionId::new("gtc-normal-g0").unwrap())
            .unwrap()
            .into_parts()
            .0
    }

    #[test]
    fn accepted_normal_clock_keeps_frontend_and_sample_domains_typed() {
        let g0 = ReaderEofNormalCallbackClock::new(
            FrontendCallbackIndex::new(0),
            AdmSampleIndex::new(0),
            ReaderEofActualSampleCount::new(1_536).unwrap(),
            ReaderEofPaddingSampleCount::new(0).unwrap(),
            NormalCallbackCount::new(1),
        )
        .unwrap();
        let g1 = ReaderEofNormalCallbackClock::new(
            FrontendCallbackIndex::new(1),
            AdmSampleIndex::new(1_536),
            ReaderEofActualSampleCount::new(1_536).unwrap(),
            ReaderEofPaddingSampleCount::new(0).unwrap(),
            NormalCallbackCount::new(2),
        )
        .unwrap();

        assert_eq!(g0.callback_index(), FrontendCallbackIndex::new(0));
        assert_eq!(g0.first_sample().value(), 0);
        assert_eq!(g0.actual_samples().value(), 1_536);
        assert_eq!(g0.normal_callbacks().value(), 1);
        assert_eq!(g1.callback_index(), FrontendCallbackIndex::new(1));
        assert_eq!(g1.first_sample().value(), 1_536);
        assert_eq!(g1.normal_callbacks().value(), 2);
    }

    #[test]
    fn clock_extent_and_identity_errors_do_not_change_reader_eof_before_state() {
        let before = ReaderEofMachine::cold();
        let before_bytes = before.state_reference().canonical_bytes().to_vec();
        let invalid = ReaderEofNormalCallbackClock::new(
            FrontendCallbackIndex::new(0),
            AdmSampleIndex::new(0),
            ReaderEofActualSampleCount::new(1_535).unwrap(),
            ReaderEofPaddingSampleCount::new(0).unwrap(),
            NormalCallbackCount::new(1),
        );
        assert_eq!(invalid, Err(ReaderEofCallbackError::SampleCountMismatch));
        assert_eq!(
            before.state_reference().canonical_bytes(),
            before_bytes.as_slice()
        );

        let stale_after = ReaderEofState::cold();
        assert_eq!(
            before.prepare("stale-callback", stale_after),
            Err(ReaderEofError::GenerationMismatch)
        );
        assert_eq!(
            before.state_reference().canonical_bytes(),
            before_bytes.as_slice()
        );
    }

    #[test]
    fn cached_reader_eof_drains_preserve_input_and_control_cache_across_exact_window() {
        let mut machine = reader_after_normal_callback(4, ReaderEofPhase::Normal, true);
        let initial_cursor = machine.committed().source_cursor_samples();
        let initial_normal_callbacks = machine.committed().normal_callbacks();
        let initial_host = machine.committed().last_normal_host38().cloned().unwrap();
        let initial_controls = machine
            .committed()
            .last_control_snapshot()
            .cloned()
            .unwrap();

        for drain_offset in 0..4_u32 {
            let callback_index = FrontendCallbackIndex::new(1 + drain_offset);
            let before_bytes = machine.state_reference().canonical_bytes().to_vec();
            let candidate = stage_reader_eof_cached_drain(
                &machine,
                callback_index,
                NormalCallbackCount::new(1),
                CallbackTransactionId::new(&format!("gtc-cached-g{}", callback_index.value()))
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(
                machine.state_reference().canonical_bytes(),
                before_bytes.as_slice(),
                "the before owner is unchanged until the parent installs the candidate"
            );
            assert_eq!(candidate.gtc_receipt().owner_id(), GtcOwnerId::ReaderEof);
            assert!(candidate.gtc_receipt().changed());
            assert_eq!(candidate.gtc_receipt().commit_count(), 1);
            let (next, owner_receipt, _) = candidate.into_parts();
            assert!(owner_receipt.changed());
            assert_eq!(owner_receipt.commit_count(), 1);
            assert_eq!(
                next.committed().callback_index(),
                FrontendCallbackIndex::new(2 + drain_offset)
            );
            assert_eq!(next.committed().source_cursor_samples(), initial_cursor);
            assert_eq!(
                next.committed().normal_callbacks(),
                initial_normal_callbacks
            );
            assert_eq!(next.committed().remaining_cached_drains(), 3 - drain_offset);
            assert_eq!(next.committed().phase(), ReaderEofPhase::CachedDrain);
            assert_eq!(
                next.committed().last_normal_host38().unwrap(),
                &initial_host
            );
            assert_eq!(
                next.committed().last_control_snapshot().unwrap(),
                &initial_controls
            );
            machine = next;
        }

        assert_eq!(machine.committed().remaining_cached_drains(), 0);
        assert!(matches!(
            stage_reader_eof_cached_drain(
                &machine,
                machine.committed().callback_index(),
                NormalCallbackCount::new(1),
                CallbackTransactionId::new("gtc-cached-out-of-window").unwrap(),
            ),
            Err(ReaderEofCallbackError::CachedDrainOutsideWindow)
        ));
    }

    #[test]
    fn cached_reader_eof_rejects_wrong_count_order_budget_phase_and_missing_cache() {
        let valid = reader_after_normal_callback(4, ReaderEofPhase::Normal, true);
        assert!(matches!(
            stage_reader_eof_cached_drain(
                &valid,
                FrontendCallbackIndex::new(1),
                NormalCallbackCount::new(2),
                CallbackTransactionId::new("gtc-cached-wrong-count").unwrap(),
            ),
            Err(ReaderEofCallbackError::CachedDrainNormalCountMismatch)
        ));
        assert!(matches!(
            stage_reader_eof_cached_drain(
                &valid,
                FrontendCallbackIndex::new(2),
                NormalCallbackCount::new(1),
                CallbackTransactionId::new("gtc-cached-wrong-order").unwrap(),
            ),
            Err(ReaderEofCallbackError::CachedDrainCallbackOrder)
        ));

        let wrong_budget = reader_after_normal_callback(3, ReaderEofPhase::Normal, true);
        assert!(matches!(
            stage_reader_eof_cached_drain(
                &wrong_budget,
                FrontendCallbackIndex::new(1),
                NormalCallbackCount::new(1),
                CallbackTransactionId::new("gtc-cached-wrong-budget").unwrap(),
            ),
            Err(ReaderEofCallbackError::CachedDrainBudgetMismatch)
        ));

        let wrong_phase = reader_after_normal_callback(4, ReaderEofPhase::CachedDrain, true);
        assert!(matches!(
            stage_reader_eof_cached_drain(
                &wrong_phase,
                FrontendCallbackIndex::new(1),
                NormalCallbackCount::new(1),
                CallbackTransactionId::new("gtc-cached-wrong-phase").unwrap(),
            ),
            Err(ReaderEofCallbackError::CachedDrainPhaseMismatch)
        ));

        let missing_cache = reader_after_normal_callback(4, ReaderEofPhase::Normal, false);
        assert!(matches!(
            stage_reader_eof_cached_drain(
                &missing_cache,
                FrontendCallbackIndex::new(1),
                NormalCallbackCount::new(1),
                CallbackTransactionId::new("gtc-cached-missing-cache").unwrap(),
            ),
            Err(ReaderEofCallbackError::CachedDrainCacheMissing)
        ));
    }
}
