// pattern: Functional Core
//! Joins one terminal Meter receipt to the exact prepared input before deriving F06/F13 candidates.

pub use jocforge_domain::meter::{
    measurement_dependents::MeasurementDependentOwnerInputs,
    measurement_receipt::ProgrammeMeasurementReceipt, standalone_root::StandaloneMeterRoot,
};

use jocforge_domain::meter::{
    measurement_dependents::{
        MeasurementDependentOwnerError, prepare_measurement_dependent_owner_inputs,
    },
    standalone_root::StandaloneMeterMeasurementProposal,
};
use jocforge_domain::transaction::{
    GtcOwnerId, GtcOwnerReceipt, GtcOwnerReceiptError, GtcOwnerStateSnapshot,
};

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
    let identity = meter_root.identity();
    if identity.programme_id() != prepared.programme_id()
        || measurement.programme_id() != prepared.programme_id()
        || identity.attempt() != measurement.attempt()
        || identity.input_sha256() != &prepared.resource_identity().input_sha256()
        || identity.config_sha256() != &prepared.config_sha256()
        || measurement.input_sha256() != identity.input_sha256()
        || measurement.config_sha256() != identity.config_sha256()
        || measurement.meter_terminal_sha256()
            != &meter_root
                .canonical_sha256()
                .map_err(|_| MeterFinalizationBridgeError::CanonicalState)?
        || measurement
            .normal_callbacks()
            .checked_add(measurement.zero_drains())
            != Some(meter_root.meter_callback_index().value())
    {
        return Err(MeterFinalizationBridgeError::PreparedInputIdentityMismatch);
    }
    let canonical = meter_root
        .canonical_preimage()
        .map_err(|_| MeterFinalizationBridgeError::CanonicalState)?;
    let cold_sha256 = meter_root
        .cold_canonical_sha256()
        .map_err(|_| MeterFinalizationBridgeError::CanonicalState)?;
    let generation = meter_root.meter_callback_index().value();
    let before = GtcOwnerStateSnapshot::from_canonical_bytes(
        GtcOwnerId::Meter,
        generation,
        cold_sha256,
        &canonical,
    )
    .map_err(MeterFinalizationBridgeError::GtcReceipt)?;
    let after = GtcOwnerStateSnapshot::from_canonical_bytes(
        GtcOwnerId::Meter,
        generation,
        cold_sha256,
        &canonical,
    )
    .map_err(MeterFinalizationBridgeError::GtcReceipt)?;
    GtcOwnerReceipt::from_transition(before, after, transaction_id)
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
