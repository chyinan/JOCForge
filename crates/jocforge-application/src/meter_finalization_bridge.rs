// pattern: Functional Core
//! Joins one terminal Meter receipt to the exact prepared input before deriving F06/F13 candidates.

pub use jocforge_domain::meter::{
    measurement_dependents::MeasurementDependentOwnerInputs,
    measurement_receipt::ProgrammeMeasurementReceipt, standalone_root::StandaloneMeterRoot,
};
pub use jocforge_domain::meter::{
    owner::MeterOwnerStateReference, programme_measurement::ProgrammeMeasurementValue,
};

use jocforge_domain::meter::{
    measurement_dependents::{
        MeasurementDependentOwnerError, prepare_measurement_dependent_owner_inputs,
    },
    standalone_root::StandaloneMeterMeasurementProposal,
};
use jocforge_domain::transaction::{GtcOwnerReceipt, GtcOwnerReceiptError};

use crate::PreparedJobInput;

/// Failures while connecting a terminal Meter candidate to its dependent owner inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterFinalizationBridgeError {
    PreparedInputIdentityMismatch,
    MeterTerminalStateMismatch,
    MeasurementScheduleMismatch,
    DependentOwners(MeasurementDependentOwnerError),
    CanonicalState,
    GtcReceipt(GtcOwnerReceiptError),
}

/// Projects an unchanged terminal Meter root receipt for one MAIN callback transaction.
pub fn unchanged_meter_root_receipt(
    prepared: &PreparedJobInput,
    meter_root: &StandaloneMeterRoot,
    measurement: &ProgrammeMeasurementReceipt,
    transaction_id: jocforge_domain::transaction::CallbackTransactionId,
) -> Result<GtcOwnerReceipt, MeterFinalizationBridgeError> {
    let reference = MeterOwnerStateReference::from_root(meter_root)
        .map_err(|_| MeterFinalizationBridgeError::CanonicalState)?;
    let identity = meter_root.identity();
    if identity.programme_id() != prepared.programme_id()
        || measurement.programme_id() != prepared.programme_id()
        || identity.attempt() != measurement.attempt()
        || identity.input_sha256() != &prepared.resource_identity().input_sha256()
        || identity.config_sha256() != &prepared.config_sha256()
        || measurement.input_sha256() != identity.input_sha256()
        || measurement.config_sha256() != identity.config_sha256()
    {
        return Err(MeterFinalizationBridgeError::PreparedInputIdentityMismatch);
    }
    unchanged_meter_reference_receipt(prepared, &reference, measurement, transaction_id)
}

/// MAIN keeps the accepted meter result immutable for the whole encode.
pub fn unchanged_meter_reference_receipt(
    prepared: &PreparedJobInput,
    reference: &MeterOwnerStateReference,
    measurement: &ProgrammeMeasurementReceipt,
    transaction_id: jocforge_domain::transaction::CallbackTransactionId,
) -> Result<GtcOwnerReceipt, MeterFinalizationBridgeError> {
    if measurement.programme_id() != prepared.programme_id()
        || measurement.input_sha256() != &prepared.resource_identity().input_sha256()
        || measurement.config_sha256() != &prepared.config_sha256()
        || measurement.meter_terminal_sha256() != reference.state_sha256()
        || measurement
            .normal_callbacks()
            .checked_add(measurement.zero_drains())
            != Some(reference.generation().value())
    {
        return Err(MeterFinalizationBridgeError::PreparedInputIdentityMismatch);
    }
    let snapshot = reference
        .gtc_snapshot()
        .map_err(MeterFinalizationBridgeError::GtcReceipt)?;
    GtcOwnerReceipt::from_transition(snapshot, snapshot, transaction_id)
        .map_err(MeterFinalizationBridgeError::GtcReceipt)
}

/// Derives same-measurement DRC and F13 ID1 inputs without committing Meter or programme roots.
pub fn prepare_meter_measurement_dependents(
    prepared: &PreparedJobInput,
    meter_after: &StandaloneMeterRoot,
    terminal_measurement: &StandaloneMeterMeasurementProposal,
) -> Result<MeasurementDependentOwnerInputs, MeterFinalizationBridgeError> {
    let receipt = terminal_measurement.receipt();
    let meter_identity = meter_after.identity();
    if receipt.programme_id() != prepared.programme_id()
        || meter_identity.programme_id() != prepared.programme_id()
        || receipt.programme_id() != meter_identity.programme_id()
        || receipt.attempt() != meter_identity.attempt()
        || *receipt.input_sha256() != prepared.domain_identity().resource().input_sha256()
        || receipt.config_sha256()
            != &prepared
                .domain_identity()
                .normalized_configuration()
                .config_sha256()
        || receipt.input_sha256() != meter_identity.input_sha256()
        || receipt.config_sha256() != meter_identity.config_sha256()
    {
        return Err(MeterFinalizationBridgeError::PreparedInputIdentityMismatch);
    }

    let terminal_meter_sha256 = meter_after
        .canonical_sha256()
        .map_err(|_| MeterFinalizationBridgeError::MeterTerminalStateMismatch)?;
    if terminal_measurement.terminal_meter_sha256() != &terminal_meter_sha256
        || receipt.meter_terminal_sha256() != terminal_measurement.terminal_meter_sha256()
    {
        return Err(MeterFinalizationBridgeError::MeterTerminalStateMismatch);
    }

    let schedule = terminal_measurement.schedule();
    if receipt.normal_callbacks() != schedule.normal_callbacks().value()
        || receipt.zero_drains() != schedule.zero_drains().value()
        || receipt.energy_windows() != schedule.energy_windows()
    {
        return Err(MeterFinalizationBridgeError::MeasurementScheduleMismatch);
    }

    let prepared_identity = prepared.domain_identity_capability();
    prepare_measurement_dependent_owner_inputs(receipt, &prepared_identity)
        .map_err(MeterFinalizationBridgeError::DependentOwners)
}

/// Rebind a cached numeric measurement to the current input and encode settings.
pub fn reuse_programme_measurement(
    prepared: &PreparedJobInput,
    attempt: jocforge_domain::transaction::AttemptId,
    key: [u8; 32],
    original_terminal: [u8; 32],
    original_cold: [u8; 32],
    measured: &ProgrammeMeasurementValue,
) -> Result<(MeterOwnerStateReference, MeasurementDependentOwnerInputs), MeterFinalizationBridgeError>
{
    use jocforge_domain::meter::{
        sample_block_queue::MeterSourceSampleCount, standalone_root::StandaloneMeterSchedule,
    };
    let samples = MeterSourceSampleCount::new(prepared.configuration().source_samples())
        .map_err(|_| MeterFinalizationBridgeError::MeasurementScheduleMismatch)?;
    let schedule = StandaloneMeterSchedule::for_source_samples(samples)
        .map_err(|_| MeterFinalizationBridgeError::MeasurementScheduleMismatch)?;
    let identity = prepared.domain_identity_capability();
    let (reference, receipt) = MeterOwnerStateReference::bind_cached_measurement(
        measured,
        key,
        original_terminal,
        original_cold,
        prepared.programme_id().clone(),
        attempt,
        &identity,
        schedule,
    )
    .map_err(|_| MeterFinalizationBridgeError::CanonicalState)?;
    let inputs = prepare_measurement_dependent_owner_inputs(&receipt, &identity)
        .map_err(MeterFinalizationBridgeError::DependentOwners)?;
    Ok((reference, inputs))
}
