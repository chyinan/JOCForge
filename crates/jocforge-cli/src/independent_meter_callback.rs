// pattern: Imperative Shell
//! IO-to-application entry for one independent Meter g=0 callback candidate.

use super::meter_source_bridge::MeterSourceCoordinatorError;
use jocforge_application::{
    PreparedJobInput,
    independent_meter_callback::{
        AttemptId, CallbackTransactionId, GtcOwnerReceipt,
        IndependentMeterG0Candidate as ApplicationCandidate,
        IndependentMeterG0Error as ApplicationError, IndependentMeterG0Providers,
        MeterCallbackGtcBinding, MeterKFilterExponentState, ProgrammeId,
        StandaloneMeterMeasurementProposal, StandaloneMeterRoot,
        stage_independent_meter_fresh_zero_drain_callback as stage_application_fresh_zero_drain_callback,
        stage_independent_meter_g0_callback as stage_application_callback,
        stage_independent_meter_normal_callback as stage_application_normal_callback,
    },
    meter_finalization_bridge::{
        MeasurementDependentOwnerInputs, MeterFinalizationBridgeError,
        prepare_meter_measurement_dependents,
    },
    meter_source_bridge::MeterCallbackIndex,
};
use jocforge_io::pcm::{
    meter_source_cursor::{
        MeterPcmSourceCursor, MeterSourceCursorError, stage_meter_pcm_source_frame,
    },
    riff_pcm24::ParsedRiffPcm24,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndependentMeterG0StageError {
    SourceCursor(MeterSourceCursorError),
    SourceCoordinator(MeterSourceCoordinatorError),
    Application(ApplicationError),
    PreparedIdentityMismatch,
    ScheduleClock,
    TransactionIdentity,
    TerminalReceiptNotReady,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndependentMeterPrepassError {
    Input(IndependentMeterG0StageError),
    Callback {
        callback_index: MeterCallbackIndex,
        error: IndependentMeterG0StageError,
    },
    TerminalReceiptNotReady,
    WorkerUnavailable,
    WorkerPanicked,
}

/// Complete noncommitting source-cursor and producer candidate for independent Meter g=0.
#[must_use]
pub struct IndependentMeterG0Candidate {
    source_cursor_after: MeterPcmSourceCursor,
    application: ApplicationCandidate,
}

/// Complete staged independent Meter pass and its terminal measurement receipt proposal.
///
/// This is a noncommitting candidate. The parent still owns installation, READY admission and
/// the programme-wide root swap.
#[must_use]
pub struct IndependentMeterPrepassCandidate {
    final_callback: IndependentMeterG0Candidate,
    meter_owner_receipts: Vec<GtcOwnerReceipt>,
    terminal_measurement: StandaloneMeterMeasurementProposal,
}

impl IndependentMeterPrepassCandidate {
    pub const fn final_callback(&self) -> &IndependentMeterG0Candidate {
        &self.final_callback
    }

    pub fn meter_owner_receipts(&self) -> &[GtcOwnerReceipt] {
        &self.meter_owner_receipts
    }

    pub const fn terminal_measurement(&self) -> &StandaloneMeterMeasurementProposal {
        &self.terminal_measurement
    }

    /// Derives same-input DRC and ID1 candidates from this prepass's terminal Meter root.
    ///
    /// The enclosing programme transaction still owns READY admission and installation.
    pub fn prepare_measurement_dependents(
        &self,
        prepared: &PreparedJobInput,
    ) -> Result<MeasurementDependentOwnerInputs, MeterFinalizationBridgeError> {
        prepare_meter_measurement_dependents(
            prepared,
            self.final_callback
                .application_candidate()
                .meter_proposal()
                .after_root(),
            &self.terminal_measurement,
        )
    }

    pub fn into_parts(
        self,
    ) -> (
        IndependentMeterG0Candidate,
        Vec<GtcOwnerReceipt>,
        StandaloneMeterMeasurementProposal,
    ) {
        (
            self.final_callback,
            self.meter_owner_receipts,
            self.terminal_measurement,
        )
    }
}

impl IndependentMeterG0Candidate {
    pub const fn source_cursor_after(&self) -> &MeterPcmSourceCursor {
        &self.source_cursor_after
    }

    pub const fn application_candidate(&self) -> &ApplicationCandidate {
        &self.application
    }

    pub fn into_parts(self) -> (MeterPcmSourceCursor, ApplicationCandidate) {
        (self.source_cursor_after, self.application)
    }
}

/// Reads one selected normal source frame and delegates to the fresh application-owned
/// Meter producer chain. The IO cursor remains unchanged until a future parent commit.
pub fn stage_independent_meter_g0_callback(
    prepared: &PreparedJobInput,
    source_cursor_before: &MeterPcmSourceCursor,
    parsed_source: &ParsedRiffPcm24<'_>,
    meter_before: &StandaloneMeterRoot,
    meter_binding: MeterCallbackGtcBinding,
    k_filter_exponents_after: MeterKFilterExponentState,
    providers: &IndependentMeterG0Providers,
) -> Result<IndependentMeterG0Candidate, IndependentMeterG0StageError> {
    if source_cursor_before.source_identity().sha256()
        != prepared.resource_identity().input_sha256()
        || parsed_source.source_sha256() != prepared.resource_identity().input_sha256()
        || source_cursor_before.source_samples().value()
            != prepared.configuration().source_samples()
        || source_cursor_before.callback_index() != MeterCallbackIndex::new(0)
        || source_cursor_before.source_cursor().value() != 0
        || source_cursor_before.read_index().value() != 0
        || prepared.programme_id() != meter_before.identity().programme_id()
        || meter_binding.programme_id() != prepared.programme_id()
        || meter_binding.attempt() != meter_before.identity().attempt()
    {
        return Err(IndependentMeterG0StageError::PreparedIdentityMismatch);
    }

    let source_proposal = stage_meter_pcm_source_frame(
        source_cursor_before,
        MeterCallbackIndex::new(0),
        parsed_source,
    )
    .map_err(IndependentMeterG0StageError::SourceCursor)?;
    let source_view =
        super::meter_source_bridge::view_for_source_proposal(prepared, &source_proposal)
            .map_err(IndependentMeterG0StageError::SourceCoordinator)?;
    let application = stage_application_callback(
        prepared,
        source_view,
        meter_before,
        meter_binding,
        k_filter_exponents_after,
        providers,
    )
    .map_err(IndependentMeterG0StageError::Application)?;
    let (source_cursor_after, _frame) = source_proposal.into_parts();
    Ok(IndependentMeterG0Candidate {
        source_cursor_after,
        application,
    })
}

/// Reads and stages the next normal source callback from a prior Meter candidate.
///
/// The prior candidate is consumed and supplies the source cursor and every owner after-state;
/// the new cursor remains uncommitted until a future programme-level transaction accepts it.
pub fn stage_independent_meter_normal_callback(
    prepared: &PreparedJobInput,
    previous: IndependentMeterG0Candidate,
    parsed_source: &ParsedRiffPcm24<'_>,
    meter_binding: MeterCallbackGtcBinding,
    k_filter_exponents_after: MeterKFilterExponentState,
    providers: &IndependentMeterG0Providers,
) -> Result<IndependentMeterG0Candidate, IndependentMeterG0StageError> {
    let (source_cursor_before, application_before) = previous.into_parts();
    if source_cursor_before.source_identity().sha256()
        != prepared.resource_identity().input_sha256()
        || parsed_source.source_sha256() != prepared.resource_identity().input_sha256()
        || source_cursor_before.source_samples().value()
            != prepared.configuration().source_samples()
        || source_cursor_before.callback_index() != meter_binding.callback_index()
        || prepared.programme_id() != meter_binding.programme_id()
    {
        return Err(IndependentMeterG0StageError::PreparedIdentityMismatch);
    }

    let source_proposal = stage_meter_pcm_source_frame(
        &source_cursor_before,
        meter_binding.callback_index(),
        parsed_source,
    )
    .map_err(IndependentMeterG0StageError::SourceCursor)?;
    let source_view =
        super::meter_source_bridge::view_for_source_proposal(prepared, &source_proposal)
            .map_err(IndependentMeterG0StageError::SourceCoordinator)?;
    let application = stage_application_normal_callback(
        prepared,
        source_view,
        application_before,
        meter_binding,
        k_filter_exponents_after,
        providers,
    )
    .map_err(IndependentMeterG0StageError::Application)?;
    let (source_cursor_after, _frame) = source_proposal.into_parts();
    Ok(IndependentMeterG0Candidate {
        source_cursor_after,
        application,
    })
}

/// Stages the next fresh-zero Meter drain from a prior normal or drain candidate.
///
/// The IO cursor verifies the drain clock and keeps source position/read index unchanged. The
/// application advances its Meter-specific downstream roots from the previous candidate; no
/// owner is installed by this wrapper.
pub fn stage_independent_meter_fresh_zero_drain_callback(
    prepared: &PreparedJobInput,
    previous: IndependentMeterG0Candidate,
    parsed_source: &ParsedRiffPcm24<'_>,
    meter_binding: MeterCallbackGtcBinding,
    k_filter_exponents_after: MeterKFilterExponentState,
    providers: &IndependentMeterG0Providers,
) -> Result<IndependentMeterG0Candidate, IndependentMeterG0StageError> {
    let (source_cursor_before, application_before) = previous.into_parts();
    if source_cursor_before.source_identity().sha256()
        != prepared.resource_identity().input_sha256()
        || parsed_source.source_sha256() != prepared.resource_identity().input_sha256()
        || source_cursor_before.source_samples().value()
            != prepared.configuration().source_samples()
        || source_cursor_before.callback_index() != meter_binding.callback_index()
        || prepared.programme_id() != meter_binding.programme_id()
    {
        return Err(IndependentMeterG0StageError::PreparedIdentityMismatch);
    }

    let source_proposal = stage_meter_pcm_source_frame(
        &source_cursor_before,
        meter_binding.callback_index(),
        parsed_source,
    )
    .map_err(IndependentMeterG0StageError::SourceCursor)?;
    let source_view =
        super::meter_source_bridge::view_for_source_proposal(prepared, &source_proposal)
            .map_err(IndependentMeterG0StageError::SourceCoordinator)?;
    let application = stage_application_fresh_zero_drain_callback(
        prepared,
        source_view,
        application_before,
        meter_binding,
        k_filter_exponents_after,
        providers,
    )
    .map_err(IndependentMeterG0StageError::Application)?;
    let (source_cursor_after, _frame) = source_proposal.into_parts();
    Ok(IndependentMeterG0Candidate {
        source_cursor_after,
        application,
    })
}

/// Runs the selected independent Meter schedule from a cold root through Q normal callbacks and
/// three fresh-zero drains. All owner and cursor changes remain staged until a parent accepts them.
pub fn stage_independent_meter_prepass(
    prepared: &PreparedJobInput,
    source_cursor_before: &MeterPcmSourceCursor,
    parsed_source: &ParsedRiffPcm24<'_>,
    meter_before: &StandaloneMeterRoot,
    providers: &IndependentMeterG0Providers,
) -> Result<IndependentMeterPrepassCandidate, IndependentMeterPrepassError> {
    let source_samples = prepared.configuration().source_samples();
    let normal_callbacks = source_samples.div_ceil(1_536);
    if normal_callbacks == 0
        || source_cursor_before.source_identity().sha256()
            != prepared.resource_identity().input_sha256()
        || source_cursor_before.source_samples().value() != source_samples
        || source_cursor_before.callback_index() != MeterCallbackIndex::new(0)
        || source_cursor_before.source_cursor().value() != 0
        || source_cursor_before.read_index().value() != 0
        || parsed_source.source_sha256() != prepared.resource_identity().input_sha256()
        || parsed_source.total_samples() != source_samples
        || meter_before.source_samples().value() != source_samples
        || meter_before.normal_callback_count().value() != normal_callbacks
        || meter_before.meter_callback_index() != MeterCallbackIndex::new(0)
        || meter_before.source_cursor().value() != 0
        || meter_before.source_read_index().value() != 0
        || meter_before.identity().programme_id() != prepared.programme_id()
        || meter_before.identity().input_sha256() != &prepared.resource_identity().input_sha256()
        || meter_before.identity().config_sha256() != &prepared.config_sha256()
    {
        return Err(IndependentMeterPrepassError::Input(
            IndependentMeterG0StageError::PreparedIdentityMismatch,
        ));
    }
    let capacity = normal_callbacks
        .checked_add(3)
        .and_then(|count| usize::try_from(count).ok())
        .ok_or(IndependentMeterPrepassError::Input(
            IndependentMeterG0StageError::ScheduleClock,
        ))?;
    let attempt = meter_before.identity().attempt();
    let mut candidate = stage_independent_meter_g0_callback(
        prepared,
        source_cursor_before,
        parsed_source,
        meter_before,
        meter_binding(prepared.programme_id(), attempt, MeterCallbackIndex::new(0))
            .map_err(IndependentMeterPrepassError::Input)?,
        MeterKFilterExponentState::new(*meter_before.k_filter_exponents_i32_by_channel()),
        providers,
    )
    .map_err(IndependentMeterPrepassError::Input)?;
    let mut meter_owner_receipts = Vec::with_capacity(capacity);
    meter_owner_receipts.push(meter_receipt(&candidate).map_err(|error| {
        IndependentMeterPrepassError::Callback {
            callback_index: MeterCallbackIndex::new(0),
            error,
        }
    })?);

    for callback in 1..normal_callbacks {
        let before = candidate
            .application_candidate()
            .meter_proposal()
            .after_root();
        let k_filter_exponents =
            MeterKFilterExponentState::new(*before.k_filter_exponents_i32_by_channel());
        candidate = stage_independent_meter_normal_callback(
            prepared,
            candidate,
            parsed_source,
            meter_binding(
                prepared.programme_id(),
                attempt,
                MeterCallbackIndex::new(callback),
            )
            .map_err(IndependentMeterPrepassError::Input)?,
            k_filter_exponents,
            providers,
        )
        .map_err(|error| IndependentMeterPrepassError::Callback {
            callback_index: MeterCallbackIndex::new(callback),
            error,
        })?;
        meter_owner_receipts.push(meter_receipt(&candidate).map_err(|error| {
            IndependentMeterPrepassError::Callback {
                callback_index: MeterCallbackIndex::new(callback),
                error,
            }
        })?);
    }

    let drain_end = normal_callbacks
        .checked_add(3)
        .ok_or(IndependentMeterPrepassError::Input(
            IndependentMeterG0StageError::ScheduleClock,
        ))?;
    for callback in normal_callbacks..drain_end {
        let before = candidate
            .application_candidate()
            .meter_proposal()
            .after_root();
        let k_filter_exponents =
            MeterKFilterExponentState::new(*before.k_filter_exponents_i32_by_channel());
        candidate = stage_independent_meter_fresh_zero_drain_callback(
            prepared,
            candidate,
            parsed_source,
            meter_binding(
                prepared.programme_id(),
                attempt,
                MeterCallbackIndex::new(callback),
            )
            .map_err(IndependentMeterPrepassError::Input)?,
            k_filter_exponents,
            providers,
        )
        .map_err(|error| IndependentMeterPrepassError::Callback {
            callback_index: MeterCallbackIndex::new(callback),
            error,
        })?;
        meter_owner_receipts.push(meter_receipt(&candidate).map_err(|error| {
            IndependentMeterPrepassError::Callback {
                callback_index: MeterCallbackIndex::new(callback),
                error,
            }
        })?);
    }

    let terminal_measurement = candidate
        .application_candidate()
        .meter_proposal()
        .after_root()
        .prepare_measurement_receipt_proposal()
        .map_err(|_| IndependentMeterPrepassError::TerminalReceiptNotReady)?;
    Ok(IndependentMeterPrepassCandidate {
        final_callback: candidate,
        meter_owner_receipts,
        terminal_measurement,
    })
}

fn meter_binding(
    programme_id: &ProgrammeId,
    attempt: AttemptId,
    callback: MeterCallbackIndex,
) -> Result<MeterCallbackGtcBinding, IndependentMeterG0StageError> {
    let transaction_id = format!(
        "meter-prepass-{}-attempt-{}-callback-{}",
        programme_id.as_str(),
        attempt.value(),
        callback.value()
    );
    let transaction_id = CallbackTransactionId::new(&transaction_id)
        .map_err(|_| IndependentMeterG0StageError::TransactionIdentity)?;
    Ok(MeterCallbackGtcBinding::new(
        transaction_id,
        programme_id.clone(),
        attempt,
        callback,
    ))
}

fn meter_receipt(
    candidate: &IndependentMeterG0Candidate,
) -> Result<GtcOwnerReceipt, IndependentMeterG0StageError> {
    candidate
        .application_candidate()
        .meter_proposal()
        .gtc_receipt()
        .map_err(|error| {
            IndependentMeterG0StageError::Application(ApplicationError::MeterGtc(error))
        })
}
