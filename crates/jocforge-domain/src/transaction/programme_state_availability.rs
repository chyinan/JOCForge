// pattern: Functional Core
//! Typed v3 ProgrammeState owner-availability projection and measurement transition proposal.

use crate::meter::measurement_receipt::ProgrammeMeasurementReceipt;
use serde_json::{Value, json};

use super::{
    AttemptId, CallbackTransactionId, GtcOwnerId, GtcOwnerReceipt, GtcOwnerReceiptError,
    GtcOwnerStateSnapshot, ProgrammeId,
};

const MAIN_OWNER_CONSTRUCTION_ORDER: [GtcOwnerId; 11] = [
    GtcOwnerId::Analysis,
    GtcOwnerId::Codec,
    GtcOwnerId::Drc,
    GtcOwnerId::FrontendControl,
    GtcOwnerId::Integrity,
    GtcOwnerId::Joc,
    GtcOwnerId::Metadata,
    GtcOwnerId::Output,
    GtcOwnerId::ReaderEof,
    GtcOwnerId::SpatialAudio,
    GtcOwnerId::SpatialControl,
];

/// Lifecycle status from the approved GTC ProgrammeState contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgrammeStatus {
    Metering,
    Ready,
    Active,
    Retired,
    Finalizing,
    Done,
}

impl ProgrammeStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Metering => "METERING",
            Self::Ready => "READY",
            Self::Active => "ACTIVE",
            Self::Retired => "RETIRED",
            Self::Finalizing => "FINALIZING",
            Self::Done => "DONE",
        }
    }
}

/// Programme and input identity retained by one owner-availability projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgrammeTransactionIdentity {
    programme_id: ProgrammeId,
    attempt: AttemptId,
    input_sha256: [u8; 32],
    config_sha256: [u8; 32],
}

impl ProgrammeTransactionIdentity {
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

/// One typed owner slot in the approved local ProgrammeState v3 schema.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProgrammeOwnerAvailability {
    Uninitialized { owner_id: GtcOwnerId },
    Initialized(ProgrammeOwnerStateReference),
}

impl ProgrammeOwnerAvailability {
    pub const fn owner_id(&self) -> GtcOwnerId {
        match self {
            Self::Uninitialized { owner_id } => *owner_id,
            Self::Initialized(state) => state.owner_id,
        }
    }

    pub const fn is_initialized(&self) -> bool {
        matches!(self, Self::Initialized(_))
    }

    pub fn as_json(&self) -> Value {
        match self {
            Self::Uninitialized { owner_id } => json!({
                "owner_id": owner_id.as_str(),
                "availability": "UNINITIALIZED"
            }),
            Self::Initialized(state) => state.as_json(),
        }
    }
}

/// Canonical owner reference derived from the owner's actual state bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgrammeOwnerStateReference {
    owner_id: GtcOwnerId,
    generation: u64,
    state_sha256: [u8; 32],
    cold_sha256: [u8; 32],
    state_ref: &'static str,
}

impl ProgrammeOwnerStateReference {
    fn from_snapshot(snapshot: GtcOwnerStateSnapshot<'_>) -> Self {
        Self {
            owner_id: snapshot.owner_id(),
            generation: snapshot.generation(),
            state_sha256: snapshot.state_sha256(),
            cold_sha256: snapshot.cold_sha256(),
            state_ref: snapshot.state_ref().as_str(),
        }
    }

    pub const fn owner_id(&self) -> GtcOwnerId {
        self.owner_id
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn state_sha256(&self) -> &[u8; 32] {
        &self.state_sha256
    }

    pub const fn cold_sha256(&self) -> &[u8; 32] {
        &self.cold_sha256
    }

    pub const fn state_ref(&self) -> &'static str {
        self.state_ref
    }

    fn as_json(&self) -> Value {
        json!({
            "owner_id": self.owner_id.as_str(),
            "availability": "INITIALIZED",
            "generation": self.generation,
            "state_sha256": hex(&self.state_sha256),
            "cold_sha256": hex(&self.cold_sha256),
            "state_ref": self.state_ref,
        })
    }
}

/// Fixed owner slots. METERING initializes only `meter`; READY requires all twelve slots.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgrammeOwnersAvailability {
    reader_eof: ProgrammeOwnerAvailability,
    frontend_control: ProgrammeOwnerAvailability,
    spatial_control: ProgrammeOwnerAvailability,
    spatial_audio: ProgrammeOwnerAvailability,
    joc: ProgrammeOwnerAvailability,
    meter: ProgrammeOwnerAvailability,
    drc: ProgrammeOwnerAvailability,
    analysis: ProgrammeOwnerAvailability,
    codec: ProgrammeOwnerAvailability,
    metadata: ProgrammeOwnerAvailability,
    integrity: ProgrammeOwnerAvailability,
    output: ProgrammeOwnerAvailability,
}

impl ProgrammeOwnersAvailability {
    fn metering(meter: GtcOwnerStateSnapshot<'_>) -> Result<Self, ProgrammeStateV3Error> {
        let meter = initialized_owner(meter, GtcOwnerId::Meter)?;
        Ok(Self {
            reader_eof: uninitialized_owner(GtcOwnerId::ReaderEof),
            frontend_control: uninitialized_owner(GtcOwnerId::FrontendControl),
            spatial_control: uninitialized_owner(GtcOwnerId::SpatialControl),
            spatial_audio: uninitialized_owner(GtcOwnerId::SpatialAudio),
            joc: uninitialized_owner(GtcOwnerId::Joc),
            meter: ProgrammeOwnerAvailability::Initialized(meter),
            drc: uninitialized_owner(GtcOwnerId::Drc),
            analysis: uninitialized_owner(GtcOwnerId::Analysis),
            codec: uninitialized_owner(GtcOwnerId::Codec),
            metadata: uninitialized_owner(GtcOwnerId::Metadata),
            integrity: uninitialized_owner(GtcOwnerId::Integrity),
            output: uninitialized_owner(GtcOwnerId::Output),
        })
    }

    fn with_meter(&self, meter: GtcOwnerStateSnapshot<'_>) -> Result<Self, ProgrammeStateV3Error> {
        let before = initialized_state(&self.meter, GtcOwnerId::Meter)?;
        let after = ProgrammeOwnerStateReference::from_snapshot(meter);
        if after.owner_id != GtcOwnerId::Meter || after.cold_sha256 != before.cold_sha256 {
            return Err(ProgrammeStateV3Error::MeterOwnerMismatch);
        }
        let mut next = self.clone();
        next.meter = ProgrammeOwnerAvailability::Initialized(after);
        Ok(next)
    }

    fn ready(
        meter: ProgrammeOwnerStateReference,
        main: &ProgrammeMainColdOwnerSnapshots<'_>,
    ) -> Result<Self, ProgrammeStateV3Error> {
        if meter.owner_id != GtcOwnerId::Meter {
            return Err(ProgrammeStateV3Error::MeterOwnerMismatch);
        }
        Ok(Self {
            reader_eof: ProgrammeOwnerAvailability::Initialized(initialized_cold_owner(
                main.reader_eof,
                GtcOwnerId::ReaderEof,
            )?),
            frontend_control: ProgrammeOwnerAvailability::Initialized(initialized_cold_owner(
                main.frontend_control,
                GtcOwnerId::FrontendControl,
            )?),
            spatial_control: ProgrammeOwnerAvailability::Initialized(initialized_cold_owner(
                main.spatial_control,
                GtcOwnerId::SpatialControl,
            )?),
            spatial_audio: ProgrammeOwnerAvailability::Initialized(initialized_cold_owner(
                main.spatial_audio,
                GtcOwnerId::SpatialAudio,
            )?),
            joc: ProgrammeOwnerAvailability::Initialized(initialized_cold_owner(
                main.joc,
                GtcOwnerId::Joc,
            )?),
            meter: ProgrammeOwnerAvailability::Initialized(meter),
            drc: ProgrammeOwnerAvailability::Initialized(initialized_cold_owner(
                main.drc,
                GtcOwnerId::Drc,
            )?),
            analysis: ProgrammeOwnerAvailability::Initialized(initialized_cold_owner(
                main.analysis,
                GtcOwnerId::Analysis,
            )?),
            codec: ProgrammeOwnerAvailability::Initialized(initialized_cold_owner(
                main.codec,
                GtcOwnerId::Codec,
            )?),
            metadata: ProgrammeOwnerAvailability::Initialized(initialized_cold_owner(
                main.metadata,
                GtcOwnerId::Metadata,
            )?),
            integrity: ProgrammeOwnerAvailability::Initialized(initialized_cold_owner(
                main.integrity,
                GtcOwnerId::Integrity,
            )?),
            output: ProgrammeOwnerAvailability::Initialized(initialized_cold_owner(
                main.output,
                GtcOwnerId::Output,
            )?),
        })
    }

    pub fn owner(&self, owner_id: GtcOwnerId) -> &ProgrammeOwnerAvailability {
        match owner_id {
            GtcOwnerId::ReaderEof => &self.reader_eof,
            GtcOwnerId::FrontendControl => &self.frontend_control,
            GtcOwnerId::SpatialControl => &self.spatial_control,
            GtcOwnerId::SpatialAudio => &self.spatial_audio,
            GtcOwnerId::Joc => &self.joc,
            GtcOwnerId::Meter => &self.meter,
            GtcOwnerId::Drc => &self.drc,
            GtcOwnerId::Analysis => &self.analysis,
            GtcOwnerId::Codec => &self.codec,
            GtcOwnerId::Metadata => &self.metadata,
            GtcOwnerId::Integrity => &self.integrity,
            GtcOwnerId::Output => &self.output,
        }
    }

    pub fn all_initialized(&self) -> bool {
        GTC_OWNER_ORDER
            .into_iter()
            .all(|owner| self.owner(owner).is_initialized())
    }

    pub fn as_json(&self) -> Value {
        json!({
            "reader_eof": self.reader_eof.as_json(),
            "frontend_control": self.frontend_control.as_json(),
            "spatial_control": self.spatial_control.as_json(),
            "spatial_audio": self.spatial_audio.as_json(),
            "joc": self.joc.as_json(),
            "meter": self.meter.as_json(),
            "drc": self.drc.as_json(),
            "analysis": self.analysis.as_json(),
            "codec": self.codec.as_json(),
            "metadata": self.metadata.as_json(),
            "integrity": self.integrity.as_json(),
            "output": self.output.as_json(),
        })
    }
}

const GTC_OWNER_ORDER: [GtcOwnerId; 12] = [
    GtcOwnerId::ReaderEof,
    GtcOwnerId::FrontendControl,
    GtcOwnerId::SpatialControl,
    GtcOwnerId::SpatialAudio,
    GtcOwnerId::Joc,
    GtcOwnerId::Meter,
    GtcOwnerId::Drc,
    GtcOwnerId::Analysis,
    GtcOwnerId::Codec,
    GtcOwnerId::Metadata,
    GtcOwnerId::Integrity,
    GtcOwnerId::Output,
];

/// Actual cold snapshots for the eleven main owners created during the READY transition.
#[derive(Clone, Copy, Debug)]
pub struct ProgrammeMainColdOwnerSnapshots<'a> {
    pub reader_eof: GtcOwnerStateSnapshot<'a>,
    pub frontend_control: GtcOwnerStateSnapshot<'a>,
    pub spatial_control: GtcOwnerStateSnapshot<'a>,
    pub spatial_audio: GtcOwnerStateSnapshot<'a>,
    pub joc: GtcOwnerStateSnapshot<'a>,
    pub drc: GtcOwnerStateSnapshot<'a>,
    pub analysis: GtcOwnerStateSnapshot<'a>,
    pub codec: GtcOwnerStateSnapshot<'a>,
    pub metadata: GtcOwnerStateSnapshot<'a>,
    pub integrity: GtcOwnerStateSnapshot<'a>,
    pub output: GtcOwnerStateSnapshot<'a>,
}

impl<'a> ProgrammeMainColdOwnerSnapshots<'a> {
    fn snapshots_in_owner_order(&self) -> [(GtcOwnerId, GtcOwnerStateSnapshot<'a>); 11] {
        [
            (GtcOwnerId::Analysis, self.analysis),
            (GtcOwnerId::Codec, self.codec),
            (GtcOwnerId::Drc, self.drc),
            (GtcOwnerId::FrontendControl, self.frontend_control),
            (GtcOwnerId::Integrity, self.integrity),
            (GtcOwnerId::Joc, self.joc),
            (GtcOwnerId::Metadata, self.metadata),
            (GtcOwnerId::Output, self.output),
            (GtcOwnerId::ReaderEof, self.reader_eof),
            (GtcOwnerId::SpatialAudio, self.spatial_audio),
            (GtcOwnerId::SpatialControl, self.spatial_control),
        ]
    }
}

/// Typed ProgrammeState owner-availability projection for the adopted local v3 schema.
///
/// This projection validates owner references and transition receipts. It does not own all twelve
/// concrete module roots or grant installation authority to a programme parent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgrammeStateAvailabilityProjection {
    identity: ProgrammeTransactionIdentity,
    status: ProgrammeStatus,
    phase_generation: u64,
    measurement_receipt: Option<ProgrammeMeasurementReceipt>,
    owners: ProgrammeOwnersAvailability,
}

impl ProgrammeStateAvailabilityProjection {
    pub fn metering(
        identity: ProgrammeTransactionIdentity,
        meter_cold: GtcOwnerStateSnapshot<'_>,
    ) -> Result<Self, ProgrammeStateV3Error> {
        if meter_cold.generation() != 0 || meter_cold.state_sha256() != meter_cold.cold_sha256() {
            return Err(ProgrammeStateV3Error::MeterNotCold);
        }
        let owners = ProgrammeOwnersAvailability::metering(meter_cold)?;
        let projection = Self {
            identity,
            status: ProgrammeStatus::Metering,
            phase_generation: 0,
            measurement_receipt: None,
            owners,
        };
        projection.validate()?;
        Ok(projection)
    }

    /// Projects a terminal Meter owner snapshot into METERING before the single READY proposal.
    ///
    /// This binds the terminal owner state and measurement receipt to the exact programme
    /// attempt/input/configuration. It does not commit the Meter root or install READY.
    pub fn metering_at_terminal(
        identity: ProgrammeTransactionIdentity,
        terminal_meter: GtcOwnerStateSnapshot<'_>,
        receipt: &ProgrammeMeasurementReceipt,
    ) -> Result<Self, ProgrammeStateV3Error> {
        if terminal_meter.owner_id() != GtcOwnerId::Meter
            || terminal_meter.state_sha256() != *receipt.meter_terminal_sha256()
            || receipt
                .normal_callbacks()
                .checked_add(receipt.zero_drains())
                != Some(terminal_meter.generation())
        {
            return Err(ProgrammeStateV3Error::MeterTerminalMismatch);
        }
        if receipt.programme_id() != identity.programme_id()
            || receipt.attempt() != identity.attempt()
            || receipt.input_sha256() != &identity.input_sha256
            || receipt.config_sha256() != &identity.config_sha256
        {
            return Err(ProgrammeStateV3Error::MeasurementIdentityMismatch);
        }
        let phase_generation = terminal_meter.generation();
        let projection = Self {
            identity,
            status: ProgrammeStatus::Metering,
            phase_generation,
            measurement_receipt: None,
            owners: ProgrammeOwnersAvailability::metering(terminal_meter)?,
        };
        projection.validate()?;
        Ok(projection)
    }

    pub const fn status(&self) -> ProgrammeStatus {
        self.status
    }

    pub const fn phase_generation(&self) -> u64 {
        self.phase_generation
    }

    pub const fn measurement_receipt(&self) -> Option<&ProgrammeMeasurementReceipt> {
        self.measurement_receipt.as_ref()
    }

    pub const fn owners(&self) -> &ProgrammeOwnersAvailability {
        &self.owners
    }

    /// Proposes one successful Meter owner transition while all main owners remain uninitialized.
    pub fn prepare_meter_callback(
        &self,
        after_meter: GtcOwnerStateSnapshot<'_>,
        receipt: &GtcOwnerReceipt,
    ) -> Result<Self, ProgrammeStateV3Error> {
        if self.status != ProgrammeStatus::Metering || self.measurement_receipt.is_some() {
            return Err(ProgrammeStateV3Error::WrongStatus);
        }
        let before = initialized_state(&self.owners.meter, GtcOwnerId::Meter)?;
        let after = ProgrammeOwnerStateReference::from_snapshot(after_meter);
        if receipt.owner_id() != GtcOwnerId::Meter
            || receipt.before_generation() != before.generation
            || receipt.before_sha256() != before.state_sha256
            || receipt.after_generation() != after.generation
            || receipt.after_sha256() != after.state_sha256
            || !receipt.changed()
            || receipt.commit_count() != 1
            || after.cold_sha256 != before.cold_sha256
        {
            return Err(ProgrammeStateV3Error::MeterTransitionMismatch);
        }

        let phase_generation = self
            .phase_generation
            .checked_add(1)
            .ok_or(ProgrammeStateV3Error::PhaseGenerationOverflow)?;
        let projection = Self {
            identity: self.identity.clone(),
            status: ProgrammeStatus::Metering,
            phase_generation,
            measurement_receipt: None,
            owners: self.owners.with_meter(after_meter)?,
        };
        projection.validate()?;
        Ok(projection)
    }

    /// Prepares the single METERING-to-READY parent transition from the actual terminal Meter
    /// snapshot, same-identity receipt, and eleven actual cold owner snapshots.
    pub fn prepare_measurement_ready(
        &self,
        terminal_meter: GtcOwnerStateSnapshot<'_>,
        receipt: ProgrammeMeasurementReceipt,
        main_cold_owners: &ProgrammeMainColdOwnerSnapshots<'_>,
        transaction_id: CallbackTransactionId,
    ) -> Result<MeasurementReadyAvailabilityProposal, ProgrammeStateV3Error> {
        if self.status != ProgrammeStatus::Metering || self.measurement_receipt.is_some() {
            return Err(ProgrammeStateV3Error::WrongStatus);
        }
        let meter_before = initialized_state(&self.owners.meter, GtcOwnerId::Meter)?;
        let meter_after = ProgrammeOwnerStateReference::from_snapshot(terminal_meter);
        if *meter_before != meter_after
            || receipt.meter_terminal_sha256() != &meter_after.state_sha256
            || receipt
                .normal_callbacks()
                .checked_add(receipt.zero_drains())
                != Some(meter_after.generation)
        {
            return Err(ProgrammeStateV3Error::MeterTerminalMismatch);
        }
        if receipt.programme_id() != self.identity.programme_id()
            || receipt.attempt() != self.identity.attempt()
            || receipt.input_sha256() != &self.identity.input_sha256
            || receipt.config_sha256() != &self.identity.config_sha256
        {
            return Err(ProgrammeStateV3Error::MeasurementIdentityMismatch);
        }

        let main_snapshots = main_cold_owners.snapshots_in_owner_order();
        let mut construction_receipts = Vec::with_capacity(MAIN_OWNER_CONSTRUCTION_ORDER.len());
        for (expected_owner, (actual_owner, snapshot)) in MAIN_OWNER_CONSTRUCTION_ORDER
            .into_iter()
            .zip(main_snapshots)
        {
            if actual_owner != expected_owner {
                return Err(ProgrammeStateV3Error::MainOwnerIdentityMismatch {
                    expected: expected_owner,
                    actual: actual_owner,
                });
            }
            let state = initialized_cold_owner(snapshot, expected_owner)?;
            construction_receipts.push(ProgrammeOwnerConstructionReceipt::new(
                &state,
                transaction_id.clone(),
            ));
        }
        let construction_receipts: [ProgrammeOwnerConstructionReceipt; 11] = construction_receipts
            .try_into()
            .map_err(|_| ProgrammeStateV3Error::MainOwnerSetMismatch)?;

        let meter_receipt = GtcOwnerReceipt::from_transition(
            terminal_meter,
            terminal_meter,
            transaction_id.clone(),
        )
        .map_err(ProgrammeStateV3Error::OwnerReceipt)?;
        let phase_generation = self
            .phase_generation
            .checked_add(1)
            .ok_or(ProgrammeStateV3Error::PhaseGenerationOverflow)?;
        let owners = ProgrammeOwnersAvailability::ready(meter_after, main_cold_owners)?;
        let after = Self {
            identity: self.identity.clone(),
            status: ProgrammeStatus::Ready,
            phase_generation,
            measurement_receipt: Some(receipt),
            owners,
        };
        after.validate()?;
        Ok(MeasurementReadyAvailabilityProposal {
            transaction_id,
            meter_receipt,
            owner_construction_receipts: construction_receipts,
            after,
        })
    }

    pub fn validate(&self) -> Result<(), ProgrammeStateV3Error> {
        for expected_owner in GTC_OWNER_ORDER {
            if self.owners.owner(expected_owner).owner_id() != expected_owner {
                return Err(ProgrammeStateV3Error::OwnerAvailabilityMismatch(
                    expected_owner,
                ));
            }
        }
        match self.status {
            ProgrammeStatus::Metering => {
                let meter = initialized_state(&self.owners.meter, GtcOwnerId::Meter)?;
                if self.measurement_receipt.is_some()
                    || meter.generation != self.phase_generation
                    || GTC_OWNER_ORDER
                        .into_iter()
                        .filter(|owner| *owner != GtcOwnerId::Meter)
                        .any(|owner| self.owners.owner(owner).is_initialized())
                {
                    return Err(ProgrammeStateV3Error::MeteringAvailabilityMismatch);
                }
            }
            ProgrammeStatus::Ready
            | ProgrammeStatus::Active
            | ProgrammeStatus::Finalizing
            | ProgrammeStatus::Done => {
                let receipt = self
                    .measurement_receipt
                    .as_ref()
                    .ok_or(ProgrammeStateV3Error::ReadyAvailabilityMismatch)?;
                let meter = initialized_state(&self.owners.meter, GtcOwnerId::Meter)?;
                if !self.owners.all_initialized() {
                    return Err(ProgrammeStateV3Error::ReadyAvailabilityMismatch);
                }
                if receipt.programme_id() != self.identity.programme_id()
                    || receipt.attempt() != self.identity.attempt()
                    || receipt.input_sha256() != &self.identity.input_sha256
                    || receipt.config_sha256() != &self.identity.config_sha256
                {
                    return Err(ProgrammeStateV3Error::MeasurementIdentityMismatch);
                }
                if receipt.meter_terminal_sha256() != &meter.state_sha256
                    || receipt
                        .normal_callbacks()
                        .checked_add(receipt.zero_drains())
                        != Some(meter.generation)
                {
                    return Err(ProgrammeStateV3Error::MeterTerminalMismatch);
                }
            }
            ProgrammeStatus::Retired => {}
        }
        Ok(())
    }
}

/// One owner construction receipt from UNINITIALIZED to an actual cold state reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgrammeOwnerConstructionReceipt {
    owner_id: GtcOwnerId,
    after_generation: u64,
    after_sha256: [u8; 32],
    cold_sha256: [u8; 32],
    state_ref: &'static str,
    transaction_id: CallbackTransactionId,
}

impl ProgrammeOwnerConstructionReceipt {
    fn new(state: &ProgrammeOwnerStateReference, transaction_id: CallbackTransactionId) -> Self {
        Self {
            owner_id: state.owner_id,
            after_generation: state.generation,
            after_sha256: state.state_sha256,
            cold_sha256: state.cold_sha256,
            state_ref: state.state_ref,
            transaction_id,
        }
    }

    pub const fn owner_id(&self) -> GtcOwnerId {
        self.owner_id
    }

    pub const fn after_generation(&self) -> u64 {
        self.after_generation
    }

    pub const fn after_sha256(&self) -> &[u8; 32] {
        &self.after_sha256
    }

    pub const fn cold_sha256(&self) -> &[u8; 32] {
        &self.cold_sha256
    }

    pub const fn state_ref(&self) -> &'static str {
        self.state_ref
    }

    pub fn transaction_id(&self) -> &CallbackTransactionId {
        &self.transaction_id
    }

    pub fn as_json(&self) -> Value {
        json!({
            "owner_id": self.owner_id.as_str(),
            "before_availability": "UNINITIALIZED",
            "after_generation": self.after_generation,
            "after_sha256": hex(&self.after_sha256),
            "cold_sha256": hex(&self.cold_sha256),
            "state_ref": self.state_ref,
            "changed": true,
            "commit_count": 1,
            "transaction_id": self.transaction_id.as_str(),
        })
    }
}

/// Noncommitting MEASUREMENT parent candidate. It carries owner references/receipts, not live
/// module roots; installation remains the responsibility of a full ProgrammeState owner.
#[must_use]
pub struct MeasurementReadyAvailabilityProposal {
    transaction_id: CallbackTransactionId,
    meter_receipt: GtcOwnerReceipt,
    owner_construction_receipts: [ProgrammeOwnerConstructionReceipt; 11],
    after: ProgrammeStateAvailabilityProjection,
}

impl MeasurementReadyAvailabilityProposal {
    pub fn transaction_id(&self) -> &CallbackTransactionId {
        &self.transaction_id
    }

    pub const fn meter_receipt(&self) -> &GtcOwnerReceipt {
        &self.meter_receipt
    }

    pub const fn owner_construction_receipts(&self) -> &[ProgrammeOwnerConstructionReceipt; 11] {
        &self.owner_construction_receipts
    }

    pub const fn after(&self) -> &ProgrammeStateAvailabilityProjection {
        &self.after
    }

    /// Returns staged owner receipts in canonical ASCII owner order.
    ///
    /// This proposal is not a committed ProgrammeState transaction and therefore does not
    /// serialize a top-level transaction result or commit count.
    pub fn owner_receipts_json(&self) -> Vec<Value> {
        let mut receipts = Vec::with_capacity(GTC_OWNER_ORDER.len());
        receipts.push((
            self.meter_receipt.owner_id().as_str(),
            owner_receipt_json(&self.meter_receipt),
        ));
        receipts.extend(
            self.owner_construction_receipts
                .iter()
                .map(|receipt| (receipt.owner_id.as_str(), receipt.as_json())),
        );
        receipts.sort_by_key(|(owner_id, _)| *owner_id);
        receipts.into_iter().map(|(_, receipt)| receipt).collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgrammeStateV3Error {
    WrongStatus,
    MeterOwnerMismatch,
    MeterNotCold,
    MeterPhaseGenerationMismatch,
    MeterTransitionMismatch,
    MeterTerminalMismatch,
    MeasurementIdentityMismatch,
    MeterReceiptScheduleMismatch,
    MainOwnerIdentityMismatch {
        expected: GtcOwnerId,
        actual: GtcOwnerId,
    },
    MainOwnerNotCold(GtcOwnerId),
    MainOwnerSetMismatch,
    OwnerAvailabilityMismatch(GtcOwnerId),
    MeteringAvailabilityMismatch,
    ReadyAvailabilityMismatch,
    PhaseGenerationOverflow,
    OwnerReceipt(GtcOwnerReceiptError),
}

fn initialized_owner(
    snapshot: GtcOwnerStateSnapshot<'_>,
    expected: GtcOwnerId,
) -> Result<ProgrammeOwnerStateReference, ProgrammeStateV3Error> {
    let state = ProgrammeOwnerStateReference::from_snapshot(snapshot);
    if state.owner_id != expected {
        return Err(ProgrammeStateV3Error::MainOwnerIdentityMismatch {
            expected,
            actual: state.owner_id,
        });
    }
    Ok(state)
}

fn initialized_cold_owner(
    snapshot: GtcOwnerStateSnapshot<'_>,
    expected: GtcOwnerId,
) -> Result<ProgrammeOwnerStateReference, ProgrammeStateV3Error> {
    let state = initialized_owner(snapshot, expected)?;
    if state.generation != 0 || state.state_sha256 != state.cold_sha256 {
        return Err(ProgrammeStateV3Error::MainOwnerNotCold(expected));
    }
    Ok(state)
}

fn initialized_state(
    availability: &ProgrammeOwnerAvailability,
    expected: GtcOwnerId,
) -> Result<&ProgrammeOwnerStateReference, ProgrammeStateV3Error> {
    match availability {
        ProgrammeOwnerAvailability::Initialized(state) if state.owner_id == expected => Ok(state),
        ProgrammeOwnerAvailability::Initialized(state) => {
            Err(ProgrammeStateV3Error::MainOwnerIdentityMismatch {
                expected,
                actual: state.owner_id,
            })
        }
        ProgrammeOwnerAvailability::Uninitialized { owner_id } => {
            Err(ProgrammeStateV3Error::OwnerAvailabilityMismatch(*owner_id))
        }
    }
}

fn uninitialized_owner(owner_id: GtcOwnerId) -> ProgrammeOwnerAvailability {
    ProgrammeOwnerAvailability::Uninitialized { owner_id }
}

fn owner_receipt_json(receipt: &GtcOwnerReceipt) -> Value {
    json!({
        "owner_id": receipt.owner_id().as_str(),
        "before_generation": receipt.before_generation(),
        "before_sha256": hex(&receipt.before_sha256()),
        "after_generation": receipt.after_generation(),
        "after_sha256": hex(&receipt.after_sha256()),
        "changed": receipt.changed(),
        "commit_count": receipt.commit_count(),
        "transaction_id": receipt.transaction_id().as_str(),
    })
}

fn hex(bytes: &[u8; 32]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{
        GTC_OWNER_ORDER, MAIN_OWNER_CONSTRUCTION_ORDER, ProgrammeMainColdOwnerSnapshots,
        ProgrammeOwnerAvailability, ProgrammeOwnerConstructionReceipt,
        ProgrammeStateAvailabilityProjection, ProgrammeStateV3Error, ProgrammeStatus,
        ProgrammeTransactionIdentity,
    };
    use crate::{
        meter::measurement_receipt::{
            ProgrammeMeasurementReceipt, ProgrammeMeasurementReceiptInput,
        },
        transaction::{
            AttemptId, CallbackTransactionId, GtcOwnerId, GtcOwnerReceipt, GtcOwnerStateSnapshot,
            ProgrammeId,
        },
    };
    use sha2::{Digest, Sha256};

    fn snapshot(
        owner_id: GtcOwnerId,
        generation: u64,
        cold_sha256: [u8; 32],
        bytes: &[u8],
    ) -> GtcOwnerStateSnapshot<'_> {
        GtcOwnerStateSnapshot::from_canonical_bytes(owner_id, generation, cold_sha256, bytes)
            .unwrap()
    }

    fn cold_snapshot(owner_id: GtcOwnerId, bytes: &[u8]) -> GtcOwnerStateSnapshot<'_> {
        let cold_sha256 = Sha256::digest(bytes).into();
        snapshot(owner_id, 0, cold_sha256, bytes)
    }

    fn identity() -> ProgrammeTransactionIdentity {
        ProgrammeTransactionIdentity::new(
            ProgrammeId::new("programme-v3-test").unwrap(),
            AttemptId::new(2),
            [0x11; 32],
            [0x22; 32],
        )
    }

    fn main_cold_snapshots() -> ProgrammeMainColdOwnerSnapshots<'static> {
        let bytes: [&'static [u8]; 11] = [
            b"reader-eof-cold",
            b"frontend-control-cold",
            b"spatial-control-cold",
            b"spatial-audio-cold",
            b"joc-cold",
            b"drc-cold",
            b"analysis-cold",
            b"codec-cold",
            b"metadata-cold",
            b"integrity-cold",
            b"output-cold",
        ];
        ProgrammeMainColdOwnerSnapshots {
            reader_eof: cold_snapshot(GtcOwnerId::ReaderEof, bytes[0]),
            frontend_control: cold_snapshot(GtcOwnerId::FrontendControl, bytes[1]),
            spatial_control: cold_snapshot(GtcOwnerId::SpatialControl, bytes[2]),
            spatial_audio: cold_snapshot(GtcOwnerId::SpatialAudio, bytes[3]),
            joc: cold_snapshot(GtcOwnerId::Joc, bytes[4]),
            drc: cold_snapshot(GtcOwnerId::Drc, bytes[5]),
            analysis: cold_snapshot(GtcOwnerId::Analysis, bytes[6]),
            codec: cold_snapshot(GtcOwnerId::Codec, bytes[7]),
            metadata: cold_snapshot(GtcOwnerId::Metadata, bytes[8]),
            integrity: cold_snapshot(GtcOwnerId::Integrity, bytes[9]),
            output: cold_snapshot(GtcOwnerId::Output, bytes[10]),
        }
    }

    fn terminal_meter_snapshot() -> GtcOwnerStateSnapshot<'static> {
        let cold_bytes: &'static [u8] = b"meter-cold-state";
        let cold_sha256 = Sha256::digest(cold_bytes).into();
        snapshot(
            GtcOwnerId::Meter,
            4_250,
            cold_sha256,
            b"meter-terminal-state",
        )
    }

    fn metering_at_terminal() -> (
        ProgrammeStateAvailabilityProjection,
        GtcOwnerStateSnapshot<'static>,
        ProgrammeMeasurementReceipt,
    ) {
        let terminal = terminal_meter_snapshot();
        let receipt = receipt(terminal.state_sha256(), [0x11; 32]);
        let projection = ProgrammeStateAvailabilityProjection::metering_at_terminal(
            identity(),
            terminal,
            &receipt,
        )
        .unwrap();
        (projection, terminal, receipt)
    }

    fn receipt(terminal_sha256: [u8; 32], input_sha256: [u8; 32]) -> ProgrammeMeasurementReceipt {
        ProgrammeMeasurementReceipt::from_verified_prepass(ProgrammeMeasurementReceiptInput {
            programme_id: ProgrammeId::new("programme-v3-test").unwrap(),
            attempt: AttemptId::new(2),
            input_sha256,
            config_sha256: [0x22; 32],
            dialogue_word: (-23.0_f32).to_bits(),
            itu_word: (-23.0_f32).to_bits(),
            true_peak_word: (-1.0_f32).to_bits(),
            gating_kind: "LEVEL",
            selected_lufs_word: (-23.0_f32).to_bits(),
            meter_terminal_sha256: terminal_sha256,
            normal_callbacks: 4_247,
            zero_drains: 3,
            energy_windows: 6_794,
        })
        .unwrap()
    }

    fn cold_metering_projection() -> ProgrammeStateAvailabilityProjection {
        let bytes: &'static [u8] = b"meter-cold-state";
        ProgrammeStateAvailabilityProjection::metering(
            identity(),
            cold_snapshot(GtcOwnerId::Meter, bytes),
        )
        .unwrap()
    }

    #[test]
    fn metering_serializes_one_initialized_meter_and_eleven_hashless_slots() {
        let projection = cold_metering_projection();
        assert_eq!(projection.status(), ProgrammeStatus::Metering);
        assert_eq!(projection.phase_generation(), 0);
        assert!(projection.measurement_receipt().is_none());
        assert!(
            projection
                .owners()
                .owner(GtcOwnerId::Meter)
                .is_initialized()
        );
        for owner_id in GTC_OWNER_ORDER {
            let availability = projection.owners().owner(owner_id);
            let value = availability.as_json();
            assert_eq!(value["owner_id"], owner_id.as_str());
            if owner_id == GtcOwnerId::Meter {
                assert_eq!(value["availability"], "INITIALIZED");
                assert!(value.get("state_sha256").is_some());
            } else {
                assert!(matches!(
                    availability,
                    ProgrammeOwnerAvailability::Uninitialized { .. }
                ));
                assert_eq!(value["availability"], "UNINITIALIZED");
                assert_eq!(value.as_object().unwrap().len(), 2);
                assert!(value.get("state_sha256").is_none());
                assert!(value.get("state_ref").is_none());
                assert!(value.get("generation").is_none());
            }
        }
    }

    #[test]
    fn one_meter_commit_advances_only_meter_and_programme_phase_generation() {
        let bytes: &'static [u8] = b"meter-cold-state";
        let before = cold_snapshot(GtcOwnerId::Meter, bytes);
        let projection =
            ProgrammeStateAvailabilityProjection::metering(identity(), before).unwrap();
        let after_bytes: &'static [u8] = b"meter-after-callback-1";
        let after = snapshot(GtcOwnerId::Meter, 1, before.cold_sha256(), after_bytes);
        let receipt = GtcOwnerReceipt::from_transition(
            before,
            after,
            CallbackTransactionId::new("meter-callback-0").unwrap(),
        )
        .unwrap();

        let next = projection.prepare_meter_callback(after, &receipt).unwrap();

        assert_eq!(next.status(), ProgrammeStatus::Metering);
        assert_eq!(next.phase_generation(), 1);
        assert!(next.measurement_receipt().is_none());
        assert_eq!(
            match next.owners().owner(GtcOwnerId::Meter) {
                ProgrammeOwnerAvailability::Initialized(state) => state.generation(),
                ProgrammeOwnerAvailability::Uninitialized { .. } =>
                    panic!("Meter stays initialized"),
            },
            1
        );
        assert!(
            GTC_OWNER_ORDER
                .into_iter()
                .filter(|owner| *owner != GtcOwnerId::Meter)
                .all(|owner| !next.owners().owner(owner).is_initialized())
        );
    }

    #[test]
    fn measurement_transition_installs_receipt_and_eleven_actual_cold_references_once() {
        let (metering, terminal_meter, receipt) = metering_at_terminal();
        let proposal = metering
            .prepare_measurement_ready(
                terminal_meter,
                receipt.clone(),
                &main_cold_snapshots(),
                CallbackTransactionId::new("measurement-ready-test").unwrap(),
            )
            .unwrap();

        assert_eq!(proposal.after().status(), ProgrammeStatus::Ready);
        assert_eq!(proposal.after().phase_generation(), 4_251);
        assert_eq!(proposal.after().measurement_receipt(), Some(&receipt));
        assert!(proposal.after().owners().all_initialized());
        assert_eq!(proposal.owner_construction_receipts().len(), 11);
        assert_eq!(
            proposal
                .owner_construction_receipts()
                .iter()
                .map(ProgrammeOwnerConstructionReceipt::owner_id)
                .collect::<Vec<_>>(),
            MAIN_OWNER_CONSTRUCTION_ORDER
        );
        assert!(!proposal.meter_receipt().changed());
        assert_eq!(proposal.meter_receipt().commit_count(), 0);

        let owner_receipts = proposal.owner_receipts_json();
        assert_eq!(owner_receipts.len(), 12);
        let owner_names = owner_receipts
            .iter()
            .map(|receipt| receipt["owner_id"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert!(owner_names.windows(2).all(|pair| pair[0] <= pair[1]));
        assert_eq!(owner_receipts[0]["before_availability"], "UNINITIALIZED");
        assert_eq!(owner_receipts[0]["owner_id"], "analysis");
        assert_eq!(owner_receipts[11]["owner_id"], "spatial_control");
        assert!(
            owner_receipts
                .iter()
                .all(|receipt| receipt.get("result").is_none())
        );
    }

    #[test]
    fn terminal_metering_projection_rejects_schedule_and_identity_mismatches() {
        let terminal = terminal_meter_snapshot();
        let valid_receipt = receipt(terminal.state_sha256(), [0x11; 32]);
        let cold_sha256: [u8; 32] = Sha256::digest(b"meter-cold-state").into();
        let wrong_generation = snapshot(
            GtcOwnerId::Meter,
            4_249,
            cold_sha256,
            b"meter-terminal-state",
        );
        assert_eq!(
            ProgrammeStateAvailabilityProjection::metering_at_terminal(
                identity(),
                wrong_generation,
                &valid_receipt,
            )
            .unwrap_err(),
            ProgrammeStateV3Error::MeterTerminalMismatch
        );

        let wrong_identity = ProgrammeTransactionIdentity::new(
            ProgrammeId::new("programme-v3-test").unwrap(),
            AttemptId::new(2),
            [0x99; 32],
            [0x22; 32],
        );
        assert_eq!(
            ProgrammeStateAvailabilityProjection::metering_at_terminal(
                wrong_identity,
                terminal,
                &valid_receipt,
            )
            .unwrap_err(),
            ProgrammeStateV3Error::MeasurementIdentityMismatch
        );
    }

    #[test]
    fn stale_measurement_identity_or_non_cold_main_owner_produces_no_ready_proposal() {
        let (metering, terminal_meter, _) = metering_at_terminal();
        let bad_receipt = receipt(terminal_meter.state_sha256(), [0x99; 32]);
        assert_eq!(
            metering
                .prepare_measurement_ready(
                    terminal_meter,
                    bad_receipt,
                    &main_cold_snapshots(),
                    CallbackTransactionId::new("measurement-ready-bad-identity").unwrap(),
                )
                .err(),
            Some(ProgrammeStateV3Error::MeasurementIdentityMismatch)
        );

        let bad_output_bytes: &'static [u8] = b"not-cold-output";
        let bad_main = ProgrammeMainColdOwnerSnapshots {
            output: snapshot(
                GtcOwnerId::Output,
                1,
                Sha256::digest(b"cold-output").into(),
                bad_output_bytes,
            ),
            ..main_cold_snapshots()
        };
        let good_receipt = receipt(terminal_meter.state_sha256(), [0x11; 32]);
        assert!(matches!(
            metering.prepare_measurement_ready(
                terminal_meter,
                good_receipt,
                &bad_main,
                CallbackTransactionId::new("measurement-ready-bad-cold-root").unwrap(),
            ),
            Err(ProgrammeStateV3Error::MainOwnerNotCold(GtcOwnerId::Output))
        ));
        assert_eq!(metering.status(), ProgrammeStatus::Metering);
        assert!(metering.measurement_receipt().is_none());
    }

    #[test]
    fn uninitialized_slots_serialize_without_owner_state_identity_fields() {
        let value = ProgrammeOwnerAvailability::Uninitialized {
            owner_id: GtcOwnerId::Metadata,
        }
        .as_json();
        assert_eq!(value.as_object().unwrap().len(), 2);
        assert_eq!(value["availability"], "UNINITIALIZED");
        assert!(value.get("generation").is_none());
        assert!(value.get("state_sha256").is_none());
        assert!(value.get("cold_sha256").is_none());
        assert!(value.get("state_ref").is_none());
    }
}
