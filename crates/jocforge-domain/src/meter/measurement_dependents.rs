// pattern: Functional Core
//! Derives the selected DRC and F13 ID1 inputs from one Meter receipt and admitted input identity.

use crate::{
    drc::{NoneDrcOwnerError, NoneDrcOwnerRoot},
    joc::emdf::id1_configuration::{Id1ConfigurationError, Id1MeasurementConfiguration},
    meter::{
        measurement_receipt::ProgrammeMeasurementReceipt,
        prepared_job_identity::PreparedJobInputIdentity,
    },
};
use std::sync::Arc;

/// Failures while binding one finalized measurement to its dependent owner inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeasurementDependentOwnerError {
    PreparedJobInputIdentityMismatch,
    Drc(NoneDrcOwnerError),
    Id1(Id1ConfigurationError),
}

/// Immutable DRC and ID1 inputs derived from one terminal measurement receipt.
///
/// These values grant no owner installation or programme-root commit authority.
#[must_use]
pub struct MeasurementDependentOwnerInputs {
    receipt: ProgrammeMeasurementReceipt,
    drc_root: NoneDrcOwnerRoot,
    id1_configuration: Id1MeasurementConfiguration,
}

impl MeasurementDependentOwnerInputs {
    pub const fn receipt(&self) -> &ProgrammeMeasurementReceipt {
        &self.receipt
    }

    pub const fn drc_root(&self) -> &NoneDrcOwnerRoot {
        &self.drc_root
    }

    pub const fn id1_configuration(&self) -> &Id1MeasurementConfiguration {
        &self.id1_configuration
    }

    pub fn into_parts(
        self,
    ) -> (
        ProgrammeMeasurementReceipt,
        NoneDrcOwnerRoot,
        Id1MeasurementConfiguration,
    ) {
        (self.receipt, self.drc_root, self.id1_configuration)
    }
}

/// Prepares dependent owner inputs from the same terminal receipt and admitted input capability.
///
/// The caller retains parent authority. The DRC root and ID1 configuration are values to stage
/// into the enclosing measurement transaction; this function does not install either value.
pub fn prepare_measurement_dependent_owner_inputs(
    receipt: &ProgrammeMeasurementReceipt,
    prepared_identity: &Arc<PreparedJobInputIdentity>,
) -> Result<MeasurementDependentOwnerInputs, MeasurementDependentOwnerError> {
    if prepared_identity.resource().input_sha256() != *receipt.input_sha256()
        || prepared_identity.normalized_configuration().config_sha256() != *receipt.config_sha256()
    {
        return Err(MeasurementDependentOwnerError::PreparedJobInputIdentityMismatch);
    }

    let drc_root = NoneDrcOwnerRoot::cold_after_meter_finalize_for_prepared_input(
        receipt,
        Arc::clone(prepared_identity),
    )
    .map_err(MeasurementDependentOwnerError::Drc)?;
    let id1_configuration = Id1MeasurementConfiguration::from_final_programme_measurement(receipt)
        .map_err(MeasurementDependentOwnerError::Id1)?;

    Ok(MeasurementDependentOwnerInputs {
        receipt: receipt.clone(),
        drc_root,
        id1_configuration,
    })
}

#[cfg(test)]
mod tests {
    use super::{MeasurementDependentOwnerError, prepare_measurement_dependent_owner_inputs};
    use crate::{
        frontend::job_configuration::SelectedInputKind,
        meter::{
            measurement_receipt::{ProgrammeMeasurementReceipt, ProgrammeMeasurementReceiptInput},
            prepared_job_identity::{
                NormalizedJobConfigurationIdentity, PreparedAdmChunkSpan,
                PreparedAdmGraphProvenance, PreparedInputResourceIdentity,
                PreparedJobInputIdentity,
            },
        },
        transaction::{AttemptId, ProgrammeId},
    };
    use std::sync::Arc;

    fn receipt() -> ProgrammeMeasurementReceipt {
        ProgrammeMeasurementReceipt::from_verified_prepass(ProgrammeMeasurementReceiptInput {
            programme_id: ProgrammeId::new("measurement-dependent-owner-test").unwrap(),
            attempt: AttemptId::new(3),
            input_sha256: [0x11; 32],
            config_sha256: [0x22; 32],
            dialogue_word: (-23.0_f32).to_bits(),
            itu_word: (-23.0_f32).to_bits(),
            true_peak_word: (-1.0_f32).to_bits(),
            gating_kind: "LEVEL",
            selected_lufs_word: (-23.0_f32).to_bits(),
            meter_terminal_sha256: [0x33; 32],
            normal_callbacks: 4_247,
            zero_drains: 3,
            energy_windows: 6_794,
        })
        .unwrap()
    }

    fn prepared_identity(
        input_sha256: [u8; 32],
        config_sha256: [u8; 32],
    ) -> Arc<PreparedJobInputIdentity> {
        let resource = PreparedInputResourceIdentity::new(input_sha256, 1_024);
        let graph = PreparedAdmGraphProvenance::new(
            resource,
            PreparedAdmChunkSpan::new(*b"axml", 10, 10, [0x41; 32]),
            PreparedAdmChunkSpan::new(*b"chna", 20, 10, [0x42; 32]),
            PreparedAdmChunkSpan::new(*b"dbmd", 30, 10, [0x43; 32]),
        );
        Arc::new(
            PreparedJobInputIdentity::new(
                resource,
                NormalizedJobConfigurationIdentity::new(
                    config_sha256,
                    [0x44; 32],
                    SelectedInputKind::Adm,
                ),
                graph,
            )
            .unwrap(),
        )
    }

    #[test]
    fn one_receipt_derives_drc_and_id1_inputs_bound_to_the_admitted_identity() {
        let receipt = receipt();
        let identity = prepared_identity(*receipt.input_sha256(), *receipt.config_sha256());

        let inputs = prepare_measurement_dependent_owner_inputs(&receipt, &identity).unwrap();

        assert_eq!(inputs.receipt(), &receipt);
        assert_eq!(
            inputs.drc_root().state().programme_dialnorm(),
            Some(
                inputs
                    .drc_root()
                    .measurement()
                    .unwrap()
                    .programme_dialnorm()
            )
        );
        assert!(Arc::ptr_eq(
            inputs
                .drc_root()
                .measurement()
                .unwrap()
                .prepared_job_input_identity()
                .unwrap(),
            &identity
        ));
        assert_eq!(
            inputs.id1_configuration().programme_id(),
            receipt.programme_id()
        );
        assert_eq!(inputs.id1_configuration().attempt(), receipt.attempt());
        assert_eq!(
            inputs.id1_configuration().measurement_input_sha256(),
            receipt.input_sha256()
        );
        assert_eq!(
            inputs.id1_configuration().measurement_config_sha256(),
            receipt.config_sha256()
        );
        assert_eq!(
            inputs.id1_configuration().meter_terminal_sha256(),
            receipt.meter_terminal_sha256()
        );
    }

    #[test]
    fn receipt_rejects_a_different_admitted_resource_or_configuration() {
        let receipt = receipt();
        for identity in [
            prepared_identity([0x55; 32], *receipt.config_sha256()),
            prepared_identity(*receipt.input_sha256(), [0x66; 32]),
        ] {
            assert_eq!(
                prepare_measurement_dependent_owner_inputs(&receipt, &identity).err(),
                Some(MeasurementDependentOwnerError::PreparedJobInputIdentityMismatch)
            );
        }
    }
}
