// pattern: Functional Core
//! Joins one terminal Meter receipt to the exact prepared input before deriving F06/F13 candidates.

pub use jocforge_domain::meter::measurement_dependents::MeasurementDependentOwnerInputs;

use jocforge_domain::meter::{
    measurement_dependents::{
        MeasurementDependentOwnerError, prepare_measurement_dependent_owner_inputs,
    },
    standalone_root::{StandaloneMeterMeasurementProposal, StandaloneMeterRoot},
};

use crate::PreparedJobInput;

/// Failures while connecting a terminal Meter candidate to its dependent owner inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterFinalizationBridgeError {
    PreparedInputIdentityMismatch,
    MeterTerminalStateMismatch,
    MeasurementScheduleMismatch,
    DependentOwners(MeasurementDependentOwnerError),
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
