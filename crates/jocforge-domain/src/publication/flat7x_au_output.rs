// pattern: Functional Core
//! Target-only F15 staging values for a whole two-frame Flat7x access unit.

use sha2::{Digest, Sha256};

use crate::{
    integrity::crc16::eac3_crc16,
    integrity::programme_owner::{
        IntegrityF12SourceReceipts, IntegrityIdentity, IntegrityPayloadEvidence,
    },
    joc::emdf::metadata_owner::MetadataSourceProposal,
    profile::{
        Flat7xAuByteSequenceCandidate, Flat7xI0D0TargetPlan, Flat7xSourceGeneration,
        Flat7xSubstreamRole,
    },
    timeline::{CodecSourceIndex, JocDispatchIndex, PublishedAuIndex},
    transaction::CallbackTransactionId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xAuOutputOwnerError {
    EmptyAttemptId,
    EmptyAccessUnitCount,
    AccessUnitCountOverflow,
    AccessUnitCountExceeded,
    I0FrameExtentMismatch,
    D0FrameExtentMismatch,
    AccessUnitExtentMismatch,
    I0FrameCrc,
    D0FrameCrc,
    OutputOffsetMismatch,
    AppendReceiptCandidateMismatch,
    F13MetadataReceiptMissing,
    I0IntegrityReceiptMissing,
    D0IntegrityReceiptMissing,
    I0RoleReceiptMismatch,
    D0RoleReceiptMismatch,
    MetadataContainerMissing,
    MetadataSourceReceiptMismatch,
    MetadataCarrierBlockMismatch,
    MetadataContainerExtentMismatch,
    IntegritySourceIdentityMismatch,
    IntegrityFrameExtentMismatch,
    IntegrityFrameHashMismatch,
    IntegrityFrameCrcMismatch,
    IntegrityPayloadMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xAuAttemptRetirementReason {
    StageRejected(Flat7xAuOutputOwnerError),
    AppendReceiptCandidateMismatch,
    StagingIoFailure,
    FinalizeIncomplete,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xAuAttemptRetirement {
    attempt_id: String,
    owner_generation: u64,
    next_access_unit: PublishedAuIndex,
    durable_length: u64,
    reason: Flat7xAuAttemptRetirementReason,
}

impl Flat7xAuAttemptRetirement {
    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub const fn owner_generation(&self) -> u64 {
        self.owner_generation
    }

    pub const fn next_access_unit(&self) -> PublishedAuIndex {
        self.next_access_unit
    }

    pub const fn durable_length(&self) -> u64 {
        self.durable_length
    }

    pub const fn reason(&self) -> Flat7xAuAttemptRetirementReason {
        self.reason
    }

    pub const fn committed_owner_unchanged(&self) -> bool {
        true
    }

    pub const fn fresh_programme_cold_required(&self) -> bool {
        true
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xAuIntegrityFrameReceipt<'a> {
    role: Flat7xSubstreamRole,
    receipt: &'a IntegrityF12SourceReceipts,
}

impl<'a> Flat7xAuIntegrityFrameReceipt<'a> {
    pub const fn independent_i0(receipt: &'a IntegrityF12SourceReceipts) -> Self {
        Self {
            role: Flat7xSubstreamRole::IndependentI0,
            receipt,
        }
    }

    pub const fn dependent_d0(receipt: &'a IntegrityF12SourceReceipts) -> Self {
        Self {
            role: Flat7xSubstreamRole::DependentD0,
            receipt,
        }
    }
}

/// Optional receipt inputs are represented explicitly so missing F13 or frame-integrity
/// evidence fails closed before an append proposal exists.
#[derive(Clone, Copy)]
pub struct Flat7xAuSourceReceiptInput<'a> {
    f13_metadata: Option<&'a MetadataSourceProposal>,
    i0_integrity: Option<Flat7xAuIntegrityFrameReceipt<'a>>,
    d0_integrity: Option<Flat7xAuIntegrityFrameReceipt<'a>>,
}

impl<'a> Flat7xAuSourceReceiptInput<'a> {
    pub const fn new(
        f13_metadata: Option<&'a MetadataSourceProposal>,
        i0_integrity: Option<Flat7xAuIntegrityFrameReceipt<'a>>,
        d0_integrity: Option<Flat7xAuIntegrityFrameReceipt<'a>>,
    ) -> Self {
        Self {
            f13_metadata,
            i0_integrity,
            d0_integrity,
        }
    }
}

fn missing_source_receipt(
    f13_receipt_present: bool,
    independent_frame_receipt_present: bool,
    dependent_frame_receipt_present: bool,
) -> Option<Flat7xAuOutputOwnerError> {
    if !f13_receipt_present {
        Some(Flat7xAuOutputOwnerError::F13MetadataReceiptMissing)
    } else if !independent_frame_receipt_present {
        Some(Flat7xAuOutputOwnerError::I0IntegrityReceiptMissing)
    } else if !dependent_frame_receipt_present {
        Some(Flat7xAuOutputOwnerError::D0IntegrityReceiptMissing)
    } else {
        None
    }
}

#[derive(Clone, Copy)]
pub struct Flat7xAuSourceReceipts<'a> {
    f13_metadata: &'a MetadataSourceProposal,
    i0_integrity: &'a IntegrityF12SourceReceipts,
    d0_integrity: &'a IntegrityF12SourceReceipts,
}

/// One in-memory target-only output owner for complete Flat7x I0+D0 access units.
///
/// This owner models one aggregate AU append and keeps a pair-aware ledger. Receipt values are
/// caller-supplied candidates; this module cannot prove a real write, rollback, or readback
/// provenance. It has no canonical GTC registration, filesystem adapter, product-plan admission,
/// source-callback identity, or connection to the selected 5.1 sink.
#[derive(Debug, Eq, PartialEq)]
pub struct Flat7xAuOutputOwner {
    attempt_id: String,
    expected_access_units: u64,
    generation: u64,
    next_access_unit: PublishedAuIndex,
    durable_length: u64,
    ledger: Vec<Flat7xAuOutputLedgerEntry>,
}

impl Flat7xAuOutputOwner {
    pub fn cold(
        attempt_id: &str,
        expected_access_units: u64,
    ) -> Result<Self, Flat7xAuOutputOwnerError> {
        if attempt_id.is_empty() {
            return Err(Flat7xAuOutputOwnerError::EmptyAttemptId);
        }
        if expected_access_units == 0 {
            return Err(Flat7xAuOutputOwnerError::EmptyAccessUnitCount);
        }
        expected_access_units
            .checked_mul(u64::from(access_unit_bytes()))
            .ok_or(Flat7xAuOutputOwnerError::AccessUnitCountOverflow)?;

        Ok(Self {
            attempt_id: attempt_id.to_owned(),
            expected_access_units,
            generation: 0,
            next_access_unit: PublishedAuIndex::new(0),
            durable_length: 0,
            ledger: Vec::new(),
        })
    }

    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub const fn expected_access_units(&self) -> u64 {
        self.expected_access_units
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn next_access_unit(&self) -> PublishedAuIndex {
        self.next_access_unit
    }

    pub const fn durable_length(&self) -> u64 {
        self.durable_length
    }

    pub fn ledger(&self) -> &[Flat7xAuOutputLedgerEntry] {
        &self.ledger
    }

    pub const fn expected_product_bytes(&self) -> u64 {
        self.expected_access_units * access_unit_bytes() as u64
    }

    pub fn is_complete(&self) -> bool {
        self.next_access_unit.value() == self.expected_access_units
            && self.durable_length == self.expected_product_bytes()
            && u64::try_from(self.ledger.len()).ok() == Some(self.expected_access_units)
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }

    /// Consumes the attempt's active owner and stages the next complete I0+D0 AU.
    ///
    /// A rejected stage returns a retirement receipt; this attempt cannot be retried.
    pub fn stage_next_access_unit(
        self,
        sequence: Flat7xAuByteSequenceCandidate,
        receipt_input: Flat7xAuSourceReceiptInput<'_>,
    ) -> Result<Flat7xAuAppendProposal<'_>, Flat7xAuAttemptRetirement> {
        let result = self.stage_next_access_unit_inner(sequence, receipt_input);
        match result {
            Ok(proposal) => Ok(proposal),
            Err(reason) => {
                Err(self.retirement(Flat7xAuAttemptRetirementReason::StageRejected(reason)))
            }
        }
    }

    fn stage_next_access_unit_inner<'a>(
        &self,
        sequence: Flat7xAuByteSequenceCandidate,
        receipt_input: Flat7xAuSourceReceiptInput<'a>,
    ) -> Result<Flat7xAuAppendProposal<'a>, Flat7xAuOutputOwnerError> {
        let plan = Flat7xI0D0TargetPlan::approved_contract();
        let i0_frame_len = usize::from(plan.i0().frame_bytes());
        let d0_frame_extent = usize::from(plan.d0().frame_bytes());
        if sequence.i0_frame_bytes() != plan.i0().frame_bytes() {
            return Err(Flat7xAuOutputOwnerError::I0FrameExtentMismatch);
        }
        if sequence.d0_frame_bytes() != plan.d0().frame_bytes() {
            return Err(Flat7xAuOutputOwnerError::D0FrameExtentMismatch);
        }
        let bytes = sequence.bytes();
        if bytes.len() != i0_frame_len + d0_frame_extent {
            return Err(Flat7xAuOutputOwnerError::AccessUnitExtentMismatch);
        }
        let i0_bytes = &bytes[..i0_frame_len];
        let d0_bytes = &bytes[i0_frame_len..];
        let i0_crc_residual = eac3_crc16(&i0_bytes[2..]);
        if i0_crc_residual != 0 {
            return Err(Flat7xAuOutputOwnerError::I0FrameCrc);
        }
        let d0_crc_residual = eac3_crc16(&d0_bytes[2..]);
        if d0_crc_residual != 0 {
            return Err(Flat7xAuOutputOwnerError::D0FrameCrc);
        }

        let au_index = self.next_access_unit;
        if au_index.value() >= self.expected_access_units {
            return Err(Flat7xAuOutputOwnerError::AccessUnitCountExceeded);
        }
        if let Some(error) = missing_source_receipt(
            receipt_input.f13_metadata.is_some(),
            receipt_input.i0_integrity.is_some(),
            receipt_input.d0_integrity.is_some(),
        ) {
            return Err(error);
        }
        let f13_metadata = receipt_input
            .f13_metadata
            .ok_or(Flat7xAuOutputOwnerError::F13MetadataReceiptMissing)?;
        let i0_integrity = receipt_input
            .i0_integrity
            .ok_or(Flat7xAuOutputOwnerError::I0IntegrityReceiptMissing)?;
        let d0_integrity = receipt_input
            .d0_integrity
            .ok_or(Flat7xAuOutputOwnerError::D0IntegrityReceiptMissing)?;
        if i0_integrity.role != Flat7xSubstreamRole::IndependentI0 {
            return Err(Flat7xAuOutputOwnerError::I0RoleReceiptMismatch);
        }
        if d0_integrity.role != Flat7xSubstreamRole::DependentD0 {
            return Err(Flat7xAuOutputOwnerError::D0RoleReceiptMismatch);
        }
        let source_receipts = Flat7xAuSourceReceipts {
            f13_metadata,
            i0_integrity: i0_integrity.receipt,
            d0_integrity: d0_integrity.receipt,
        };
        let receipt_summary = validate_source_receipts(au_index, &sequence, source_receipts)?;
        let expected_offset = au_index
            .value()
            .checked_mul(u64::from(access_unit_bytes()))
            .ok_or(Flat7xAuOutputOwnerError::AccessUnitCountOverflow)?;
        if expected_offset != self.durable_length {
            return Err(Flat7xAuOutputOwnerError::OutputOffsetMismatch);
        }

        let i0_sha256 = Sha256::digest(i0_bytes).into();
        let d0_sha256 = Sha256::digest(d0_bytes).into();
        let au_sha256 = Sha256::digest(bytes).into();
        let ledger_entry = Flat7xAuOutputLedgerEntry {
            au_index,
            source_generation: sequence.source_generation(),
            metadata_source_index: receipt_summary.metadata_source_index,
            metadata_dispatch: receipt_summary.metadata_dispatch,
            metadata_au_index: receipt_summary.metadata_au_index,
            offset: expected_offset,
            length: access_unit_bytes(),
            i0_frame_bytes: plan.i0().frame_bytes(),
            d0_frame_bytes: plan.d0().frame_bytes(),
            i0_sha256,
            d0_sha256,
            au_sha256,
            f13_owner_generations: receipt_summary.f13_owner_generations,
            f13_zero_slot_sha256: receipt_summary.f13_zero_slot_sha256,
            d0_emdf_container_bytes: receipt_summary.d0_emdf_container_bytes,
            d0_emdf_container_sha256: receipt_summary.d0_emdf_container_sha256,
            i0_integrity_generations: receipt_summary.i0_integrity_generations,
            d0_integrity_generations: receipt_summary.d0_integrity_generations,
            i0_crc_residual,
            d0_crc_residual,
        };

        let next_generation = self
            .generation
            .checked_add(1)
            .ok_or(Flat7xAuOutputOwnerError::AccessUnitCountOverflow)?;
        let next_au = au_index
            .value()
            .checked_add(1)
            .ok_or(Flat7xAuOutputOwnerError::AccessUnitCountOverflow)?;
        let next_length = self
            .durable_length
            .checked_add(u64::from(access_unit_bytes()))
            .ok_or(Flat7xAuOutputOwnerError::AccessUnitCountOverflow)?;
        let mut after_owner = Self {
            attempt_id: self.attempt_id.clone(),
            expected_access_units: self.expected_access_units,
            generation: next_generation,
            next_access_unit: PublishedAuIndex::new(next_au),
            durable_length: next_length,
            ledger: self.ledger.clone(),
        };
        after_owner.ledger.push(ledger_entry.clone());

        Ok(Flat7xAuAppendProposal {
            attempt_id: self.attempt_id.clone(),
            before_generation: self.generation,
            ledger_entry,
            sequence,
            source_receipts,
            after_owner,
        })
    }

    fn retirement(&self, reason: Flat7xAuAttemptRetirementReason) -> Flat7xAuAttemptRetirement {
        Flat7xAuAttemptRetirement {
            attempt_id: self.attempt_id.clone(),
            owner_generation: self.generation,
            next_access_unit: self.next_access_unit,
            durable_length: self.durable_length,
            reason,
        }
    }

    /// Produces a model-only finalize proposal once every expected AU is present in the ledger.
    pub fn prepare_finalize(self) -> Result<Flat7xAuFinalizeProposal, Flat7xAuAttemptRetirement> {
        let mut ledger_valid = self.is_complete();
        for (index, entry) in self.ledger.iter().enumerate() {
            let Some(index) = u64::try_from(index).ok() else {
                ledger_valid = false;
                break;
            };
            let Some(expected_offset) = index.checked_mul(u64::from(access_unit_bytes())) else {
                ledger_valid = false;
                break;
            };
            if entry.au_index.value() != index
                || entry.offset != expected_offset
                || entry.length != access_unit_bytes()
                || entry.i0_crc_residual != 0
                || entry.d0_crc_residual != 0
                || entry.d0_emdf_container_bytes == 0
                || entry.d0_emdf_container_bytes > 511
                || entry.metadata_au_index != entry.au_index
            {
                ledger_valid = false;
                break;
            }
        }
        if !ledger_valid {
            return Err(self.retirement(Flat7xAuAttemptRetirementReason::FinalizeIncomplete));
        }
        Ok(Flat7xAuFinalizeProposal {
            attempt_id: self.attempt_id,
            owner_generation: self.generation,
            expected_access_units: self.expected_access_units,
            durable_length: self.durable_length,
            ledger: self.ledger,
        })
    }
}

struct Flat7xAuValidatedSourceReceiptSummary {
    metadata_source_index: CodecSourceIndex,
    metadata_dispatch: JocDispatchIndex,
    metadata_au_index: PublishedAuIndex,
    f13_owner_generations: (u64, u64),
    f13_zero_slot_sha256: [u8; 32],
    d0_emdf_container_bytes: u16,
    d0_emdf_container_sha256: [u8; 32],
    i0_integrity_generations: (u64, u64),
    d0_integrity_generations: (u64, u64),
}

fn validate_source_receipts(
    au_index: PublishedAuIndex,
    sequence: &Flat7xAuByteSequenceCandidate,
    receipts: Flat7xAuSourceReceipts<'_>,
) -> Result<Flat7xAuValidatedSourceReceiptSummary, Flat7xAuOutputOwnerError> {
    let metadata = receipts.f13_metadata;
    let f13 = metadata.receipt();
    let parsed = metadata
        .parsed_container()
        .ok_or(Flat7xAuOutputOwnerError::MetadataContainerMissing)?;
    let protected_container = parsed.bytes();
    let zero_slot_container = metadata
        .emdf()
        .ok_or(Flat7xAuOutputOwnerError::MetadataContainerMissing)?;
    if f13.au_index() != Some(au_index)
        || parsed.consumer_au_index() != au_index
        || parsed.consumer_source_index() != f13.source_index()
        || parsed.consumer_dispatch() != f13.dispatch()
        || parsed.consumer_programme_id() != f13.programme_id()
        || parsed.consumer_attempt() != f13.attempt()
        || metadata.consumer_transaction_id() != f13.consumer_transaction_id()
    {
        return Err(Flat7xAuOutputOwnerError::MetadataSourceReceiptMismatch);
    }

    if parsed.parsed_container().carrier_block().value() != 2 {
        return Err(Flat7xAuOutputOwnerError::MetadataCarrierBlockMismatch);
    }
    if protected_container.is_empty()
        || protected_container.len() > 511
        || protected_container.len() != usize::from(sequence.d0_emdf_container_bytes())
        || zero_slot_container.len() != protected_container.len()
    {
        return Err(Flat7xAuOutputOwnerError::MetadataContainerExtentMismatch);
    }

    let i0 = receipts.i0_integrity;
    let d0 = receipts.d0_integrity;
    let i0_identity = i0.source().identity();
    let d0_identity = d0.source().identity();
    let consumer_transaction = metadata
        .consumer_transaction_id()
        .ok_or(Flat7xAuOutputOwnerError::MetadataSourceReceiptMismatch)?;
    if i0_identity != d0_identity
        || i0_identity.programme_id != f13.programme_id().as_str()
        || i0_identity.attempt != f13.attempt().value()
        || i0_identity.transaction_id != consumer_transaction.as_str()
        || i0_identity.source_index != f13.source_index()
        || i0_identity.dispatch != f13.dispatch()
        || i0_identity.au_index != Some(au_index)
        || i0.dispatch().identity() != i0_identity
        || i0.au().identity() != i0_identity
        || d0.dispatch().identity() != d0_identity
        || d0.au().identity() != d0_identity
    {
        return Err(Flat7xAuOutputOwnerError::IntegritySourceIdentityMismatch);
    }

    let plan = Flat7xI0D0TargetPlan::approved_contract();
    let i0_end = usize::from(plan.i0().frame_bytes());
    let bytes = sequence.bytes();
    let i0_bytes = &bytes[..i0_end];
    let d0_bytes = &bytes[i0_end..];
    if i0.dispatch().frame_bytes() != i0_bytes.len()
        || d0.dispatch().frame_bytes() != d0_bytes.len()
    {
        return Err(Flat7xAuOutputOwnerError::IntegrityFrameExtentMismatch);
    }
    let i0_sha256: [u8; 32] = Sha256::digest(i0_bytes).into();
    let d0_sha256: [u8; 32] = Sha256::digest(d0_bytes).into();
    if i0.dispatch().frame_sha256().as_bytes() != &i0_sha256
        || i0.au().frame_sha256().as_bytes() != &i0_sha256
        || d0.dispatch().frame_sha256().as_bytes() != &d0_sha256
        || d0.au().frame_sha256().as_bytes() != &d0_sha256
    {
        return Err(Flat7xAuOutputOwnerError::IntegrityFrameHashMismatch);
    }
    if !i0.au().published()
        || !d0.au().published()
        || i0.au().crc_residual() != 0
        || d0.au().crc_residual() != 0
        || i0.dispatch().emdf_container_start().is_some()
        || i0.dispatch().source_payload_sha256().is_some()
        || d0.dispatch().emdf_container_start().is_none()
    {
        return Err(Flat7xAuOutputOwnerError::IntegrityFrameCrcMismatch);
    }

    let IntegrityPayloadEvidence::AbsentPublishedPayloadV1 = i0.source().payload() else {
        return Err(Flat7xAuOutputOwnerError::IntegrityPayloadMismatch);
    };
    let IntegrityPayloadEvidence::ProtectedEmdf {
        zero_slot_sha256,
        protected_sha256,
        container_bytes,
        ..
    } = d0.source().payload()
    else {
        return Err(Flat7xAuOutputOwnerError::IntegrityPayloadMismatch);
    };
    let zero_slot_sha: [u8; 32] = Sha256::digest(zero_slot_container).into();
    let protected_sha: [u8; 32] = Sha256::digest(protected_container).into();
    if zero_slot_sha256.as_bytes() != &zero_slot_sha
        || protected_sha256.as_bytes() != &protected_sha
        || container_bytes != protected_container.len()
        || d0
            .dispatch()
            .source_payload_sha256()
            .map(|sha| *sha.as_bytes())
            != Some(protected_sha)
    {
        return Err(Flat7xAuOutputOwnerError::IntegrityPayloadMismatch);
    }
    Ok(Flat7xAuValidatedSourceReceiptSummary {
        metadata_source_index: f13.source_index(),
        metadata_dispatch: f13.dispatch(),
        metadata_au_index: f13
            .au_index()
            .ok_or(Flat7xAuOutputOwnerError::MetadataSourceReceiptMismatch)?,
        f13_owner_generations: (f13.before_generation(), f13.proposed_after_generation()),
        f13_zero_slot_sha256: zero_slot_sha,
        d0_emdf_container_bytes: u16::try_from(protected_container.len())
            .map_err(|_| Flat7xAuOutputOwnerError::MetadataContainerExtentMismatch)?,
        d0_emdf_container_sha256: protected_sha,
        i0_integrity_generations: (
            i0.source().before_generation(),
            i0.source().proposed_after_generation(),
        ),
        d0_integrity_generations: (
            d0.source().before_generation(),
            d0.source().proposed_after_generation(),
        ),
    })
}

#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct Flat7xAuFinalizeProposal {
    attempt_id: String,
    owner_generation: u64,
    expected_access_units: u64,
    durable_length: u64,
    ledger: Vec<Flat7xAuOutputLedgerEntry>,
}

impl Flat7xAuFinalizeProposal {
    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub const fn owner_generation(&self) -> u64 {
        self.owner_generation
    }

    pub const fn expected_access_units(&self) -> u64 {
        self.expected_access_units
    }

    pub const fn durable_length(&self) -> u64 {
        self.durable_length
    }

    pub fn ledger(&self) -> &[Flat7xAuOutputLedgerEntry] {
        &self.ledger
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xAuOutputLedgerEntry {
    au_index: PublishedAuIndex,
    source_generation: Flat7xSourceGeneration,
    metadata_source_index: CodecSourceIndex,
    metadata_dispatch: JocDispatchIndex,
    metadata_au_index: PublishedAuIndex,
    offset: u64,
    length: u32,
    i0_frame_bytes: u16,
    d0_frame_bytes: u16,
    i0_sha256: [u8; 32],
    d0_sha256: [u8; 32],
    au_sha256: [u8; 32],
    f13_owner_generations: (u64, u64),
    f13_zero_slot_sha256: [u8; 32],
    d0_emdf_container_bytes: u16,
    d0_emdf_container_sha256: [u8; 32],
    i0_integrity_generations: (u64, u64),
    d0_integrity_generations: (u64, u64),
    i0_crc_residual: u16,
    d0_crc_residual: u16,
}

impl Flat7xAuOutputLedgerEntry {
    pub const fn au_index(&self) -> PublishedAuIndex {
        self.au_index
    }

    pub const fn source_generation(&self) -> Flat7xSourceGeneration {
        self.source_generation
    }

    pub const fn metadata_identity(
        &self,
    ) -> (CodecSourceIndex, JocDispatchIndex, PublishedAuIndex) {
        (
            self.metadata_source_index,
            self.metadata_dispatch,
            self.metadata_au_index,
        )
    }

    pub const fn offset(&self) -> u64 {
        self.offset
    }

    pub const fn length(&self) -> u32 {
        self.length
    }

    pub const fn frame_lengths(&self) -> (u16, u16) {
        (self.i0_frame_bytes, self.d0_frame_bytes)
    }

    pub const fn frame_sha256(&self) -> (&[u8; 32], &[u8; 32]) {
        (&self.i0_sha256, &self.d0_sha256)
    }

    pub const fn au_sha256(&self) -> &[u8; 32] {
        &self.au_sha256
    }

    pub const fn f13_emdf_receipt(&self) -> ((u64, u64), &[u8; 32], u16, &[u8; 32]) {
        (
            self.f13_owner_generations,
            &self.f13_zero_slot_sha256,
            self.d0_emdf_container_bytes,
            &self.d0_emdf_container_sha256,
        )
    }

    pub const fn integrity_owner_generations(&self) -> ((u64, u64), (u64, u64)) {
        (self.i0_integrity_generations, self.d0_integrity_generations)
    }

    pub const fn frame_crc_residuals(&self) -> (u16, u16) {
        (self.i0_crc_residual, self.d0_crc_residual)
    }
}

#[must_use]
pub struct Flat7xAuAppendProposal<'a> {
    attempt_id: String,
    before_generation: u64,
    ledger_entry: Flat7xAuOutputLedgerEntry,
    sequence: Flat7xAuByteSequenceCandidate,
    source_receipts: Flat7xAuSourceReceipts<'a>,
    after_owner: Flat7xAuOutputOwner,
}

impl std::fmt::Debug for Flat7xAuAppendProposal<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Flat7xAuAppendProposal")
            .field("attempt_id", &self.attempt_id)
            .field("before_generation", &self.before_generation)
            .field("ledger_entry", &self.ledger_entry)
            .field("bytes", &self.sequence.bytes().len())
            .field("source_receipts_present", &true)
            .finish()
    }
}

impl<'a> Flat7xAuAppendProposal<'a> {
    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub const fn before_generation(&self) -> u64 {
        self.before_generation
    }

    pub const fn ledger_entry(&self) -> &Flat7xAuOutputLedgerEntry {
        &self.ledger_entry
    }

    pub fn bytes(&self) -> &[u8] {
        self.sequence.bytes()
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }

    /// Returns an after-owner proposal when caller-supplied receipt fields match the whole AU.
    ///
    /// This comparison validates staged contract fields only. It does not attest that an IO
    /// adapter performed the write, rollback, fsync, or readback represented by the candidate.
    pub fn source_receipts(&self) -> Flat7xAuSourceReceipts<'_> {
        self.source_receipts
    }

    /// Returns the F12 source identities already validated against F13 when this proposal was
    /// created. The independent and dependent receipts remain distinct values.
    pub fn source_identities(&self) -> (&IntegrityIdentity, &IntegrityIdentity) {
        (
            self.source_receipts.i0_integrity.source().identity(),
            self.source_receipts.d0_integrity.source().identity(),
        )
    }

    /// Returns F13's consumer transaction used to validate the paired F12 source identities.
    pub fn metadata_consumer_transaction_id(&self) -> Option<&CallbackTransactionId> {
        self.source_receipts.f13_metadata.consumer_transaction_id()
    }

    pub fn complete(
        self,
        receipt: Flat7xAuAppendReceiptCandidate,
    ) -> Result<Flat7xAuOutputAfterProposal<'a>, Flat7xAuAttemptRetirement> {
        let Some(expected_after_generation) = self.before_generation.checked_add(1) else {
            return Err(
                self.retirement(Flat7xAuAttemptRetirementReason::StageRejected(
                    Flat7xAuOutputOwnerError::AccessUnitCountOverflow,
                )),
            );
        };
        if validate_append_receipt_candidate(&self.attempt_id, &self.ledger_entry, &receipt)
            .is_err()
            || !receipt.file_fsync
            || !receipt.durable
            || self.after_owner.generation != expected_after_generation
        {
            return Err(
                self.retirement(Flat7xAuAttemptRetirementReason::AppendReceiptCandidateMismatch)
            );
        }

        Ok(Flat7xAuOutputAfterProposal {
            ledger_entry: self.ledger_entry,
            receipt,
            source_receipts: self.source_receipts,
            after_owner: self.after_owner,
        })
    }

    /// Consumes this attempt proposal after an IO-stage failure.
    pub fn retire_staging_failure(self) -> Flat7xAuAttemptRetirement {
        self.retirement(Flat7xAuAttemptRetirementReason::StagingIoFailure)
    }

    fn retirement(&self, reason: Flat7xAuAttemptRetirementReason) -> Flat7xAuAttemptRetirement {
        Flat7xAuAttemptRetirement {
            attempt_id: self.attempt_id.clone(),
            owner_generation: self.before_generation,
            next_access_unit: self.ledger_entry.au_index,
            durable_length: self.ledger_entry.offset,
            reason,
        }
    }
}

fn validate_append_receipt_candidate(
    attempt_id: &str,
    ledger_entry: &Flat7xAuOutputLedgerEntry,
    receipt: &Flat7xAuAppendReceiptCandidate,
) -> Result<(), Flat7xAuOutputOwnerError> {
    let write_total = receipt.write_calls.iter().try_fold(0_u64, |total, count| {
        if *count == 0 {
            None
        } else {
            total.checked_add(*count)
        }
    });
    if receipt.attempt_id != attempt_id
        || receipt.au_index != ledger_entry.au_index
        || receipt.offset != ledger_entry.offset
        || receipt.bytes_written != ledger_entry.length
        || write_total != Some(u64::from(ledger_entry.length))
        || receipt.readback_sha256 != ledger_entry.au_sha256
        || receipt.i0_crc_residual != ledger_entry.i0_crc_residual
        || receipt.d0_crc_residual != ledger_entry.d0_crc_residual
        || receipt.i0_crc_residual != 0
        || receipt.d0_crc_residual != 0
    {
        return Err(Flat7xAuOutputOwnerError::AppendReceiptCandidateMismatch);
    }
    Ok(())
}

/// Caller-supplied receipt fields for a proposed aggregate append.
///
/// This value has no IO provenance until a future filesystem adapter constructs and binds it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xAuAppendReceiptCandidate {
    attempt_id: String,
    au_index: PublishedAuIndex,
    offset: u64,
    bytes_written: u32,
    write_calls: Vec<u64>,
    readback_sha256: [u8; 32],
    i0_crc_residual: u16,
    d0_crc_residual: u16,
    file_fsync: bool,
    durable: bool,
}

impl Flat7xAuAppendReceiptCandidate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        attempt_id: String,
        au_index: PublishedAuIndex,
        offset: u64,
        bytes_written: u32,
        write_calls: Vec<u64>,
        readback_sha256: [u8; 32],
        i0_crc_residual: u16,
        d0_crc_residual: u16,
        file_fsync: bool,
        durable: bool,
    ) -> Self {
        Self {
            attempt_id,
            au_index,
            offset,
            bytes_written,
            write_calls,
            readback_sha256,
            i0_crc_residual,
            d0_crc_residual,
            file_fsync,
            durable,
        }
    }

    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub const fn au_index(&self) -> PublishedAuIndex {
        self.au_index
    }

    pub const fn offset(&self) -> u64 {
        self.offset
    }

    pub const fn bytes_written(&self) -> u32 {
        self.bytes_written
    }

    pub fn write_calls(&self) -> &[u64] {
        &self.write_calls
    }

    pub const fn readback_sha256(&self) -> &[u8; 32] {
        &self.readback_sha256
    }

    pub const fn frame_crc_residuals(&self) -> (u16, u16) {
        (self.i0_crc_residual, self.d0_crc_residual)
    }

    pub const fn file_fsync(&self) -> bool {
        self.file_fsync
    }

    pub const fn durable(&self) -> bool {
        self.durable
    }
}

#[must_use]
pub struct Flat7xAuOutputAfterProposal<'a> {
    ledger_entry: Flat7xAuOutputLedgerEntry,
    receipt: Flat7xAuAppendReceiptCandidate,
    source_receipts: Flat7xAuSourceReceipts<'a>,
    after_owner: Flat7xAuOutputOwner,
}

impl Flat7xAuOutputAfterProposal<'_> {
    pub const fn ledger_entry(&self) -> &Flat7xAuOutputLedgerEntry {
        &self.ledger_entry
    }

    pub const fn receipt(&self) -> &Flat7xAuAppendReceiptCandidate {
        &self.receipt
    }

    pub fn source_receipts(&self) -> Flat7xAuSourceReceipts<'_> {
        self.source_receipts
    }

    pub fn after_owner(&self) -> &Flat7xAuOutputOwner {
        &self.after_owner
    }

    pub fn into_after_owner(self) -> Flat7xAuOutputOwner {
        self.after_owner
    }
}

const fn access_unit_bytes() -> u32 {
    Flat7xI0D0TargetPlan::approved_contract().aggregate_frame_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{
        D0EmdfSkipFieldExtent, D0FrameExtent, Flat7xAuCandidate, Flat7xSourceGeneration,
        I0FrameExtent, compose_flat7x_au_byte_sequence, validate_flat7x_au_candidate,
    };
    use crate::publication::{
        Flat7xElementaryStreamStageProposal, Flat7xElementaryStreamStageProposalError,
        Flat7xElementaryStreamStageReceiptCandidate,
    };

    fn crc_valid_frame(frame_bytes: usize, marker: u8) -> Vec<u8> {
        let size = crate::codec::frame_size::Eac3FrameSize::from_frame_bytes(frame_bytes).unwrap();
        let mut bytes = vec![0_u8; frame_bytes];
        bytes[0..2].copy_from_slice(&[0x0b, 0x77]);
        bytes[100] = marker;
        for offset in 0..11 {
            let value = (size.code().value() >> (10 - offset)) & 1;
            if value != 0 {
                let bit = 21 + offset;
                bytes[bit / 8] |= 1 << (7 - (bit % 8));
            }
        }
        let crc_offset = bytes.len() - 2;
        let crc = eac3_crc16(&bytes[2..crc_offset]);
        bytes[crc_offset..].copy_from_slice(&crc.to_be_bytes());
        assert_eq!(eac3_crc16(&bytes[2..]), 0);
        bytes
    }

    fn candidate_sequence() -> Flat7xAuByteSequenceCandidate {
        let plan = Flat7xI0D0TargetPlan::approved_contract();
        let generation = Flat7xSourceGeneration::new(7);
        let metadata = Flat7xAuCandidate::new(
            Some(I0FrameExtent::new(generation, 2_560)),
            Some(D0FrameExtent::new(generation, 4_096)),
            Some(D0EmdfSkipFieldExtent::new(
                generation,
                2,
                1,
                Some(16),
                false,
            )),
        );
        let validation = validate_flat7x_au_candidate(plan, metadata).unwrap();
        let i0 = crc_valid_frame(2_560, 0x31);
        let d0 = crc_valid_frame(4_096, 0x72);
        compose_flat7x_au_byte_sequence(&validation, &i0, &d0).unwrap()
    }

    fn ledger_entry() -> Flat7xAuOutputLedgerEntry {
        Flat7xAuOutputLedgerEntry {
            au_index: PublishedAuIndex::new(0),
            source_generation: Flat7xSourceGeneration::new(42),
            metadata_source_index: CodecSourceIndex::new(2),
            metadata_dispatch: JocDispatchIndex::new(3),
            metadata_au_index: PublishedAuIndex::new(0),
            offset: 0,
            length: 6_656,
            i0_frame_bytes: 2_560,
            d0_frame_bytes: 4_096,
            i0_sha256: [0x11; 32],
            d0_sha256: [0x22; 32],
            au_sha256: [0x33; 32],
            f13_owner_generations: (0, 1),
            f13_zero_slot_sha256: [0x44; 32],
            d0_emdf_container_bytes: 16,
            d0_emdf_container_sha256: [0x55; 32],
            i0_integrity_generations: (0, 1),
            d0_integrity_generations: (1, 2),
            i0_crc_residual: 0,
            d0_crc_residual: 0,
        }
    }

    fn stream_stage_ledger_entry(
        au_index: u64,
        source_generation: u64,
        offset: u64,
        bytes: &[u8],
    ) -> Flat7xAuOutputLedgerEntry {
        let plan = Flat7xI0D0TargetPlan::approved_contract();
        let i0_end = usize::from(plan.i0().frame_bytes());
        let i0_bytes = &bytes[..i0_end];
        let d0_bytes = &bytes[i0_end..];
        Flat7xAuOutputLedgerEntry {
            au_index: PublishedAuIndex::new(au_index),
            source_generation: Flat7xSourceGeneration::new(source_generation),
            metadata_source_index: CodecSourceIndex::new(2),
            metadata_dispatch: JocDispatchIndex::new(au_index + 3),
            metadata_au_index: PublishedAuIndex::new(au_index),
            offset,
            length: plan.aggregate_frame_bytes(),
            i0_frame_bytes: plan.i0().frame_bytes(),
            d0_frame_bytes: plan.d0().frame_bytes(),
            i0_sha256: Sha256::digest(i0_bytes).into(),
            d0_sha256: Sha256::digest(d0_bytes).into(),
            au_sha256: Sha256::digest(bytes).into(),
            f13_owner_generations: (0, 1),
            f13_zero_slot_sha256: [0x44; 32],
            d0_emdf_container_bytes: 16,
            d0_emdf_container_sha256: [0x55; 32],
            i0_integrity_generations: (0, 1),
            d0_integrity_generations: (1, 2),
            i0_crc_residual: 0,
            d0_crc_residual: 0,
        }
    }

    fn synthetic_finalize_proposal(
        attempt_id: &str,
        owner_generation: u64,
    ) -> Flat7xAuFinalizeProposal {
        let first = candidate_sequence().bytes().to_vec();
        let second = candidate_sequence().bytes().to_vec();
        let au_bytes = u64::from(access_unit_bytes());
        Flat7xAuFinalizeProposal {
            attempt_id: attempt_id.to_owned(),
            owner_generation,
            expected_access_units: 2,
            durable_length: au_bytes * 2,
            ledger: vec![
                stream_stage_ledger_entry(0, 7, 0, &first),
                stream_stage_ledger_entry(1, 8, au_bytes, &second),
            ],
        }
    }

    #[test]
    fn elementary_stream_stage_proposal_binds_finalized_f15_ledger_and_receipt_identity() {
        let target = Flat7xI0D0TargetPlan::approved_contract();
        let first = candidate_sequence().bytes().to_vec();
        let second = candidate_sequence().bytes().to_vec();
        let mut stream = first.clone();
        stream.extend_from_slice(&second);
        let finalize = synthetic_finalize_proposal("flat7x-stage-attempt", 29);

        let proposal =
            Flat7xElementaryStreamStageProposal::new(target, &stream, &finalize).unwrap();
        assert_eq!(proposal.attempt_id(), "flat7x-stage-attempt");
        assert_eq!(proposal.owner_generation(), 29);
        assert_eq!(proposal.extent().access_unit_count(), 2);
        assert_eq!(
            proposal.finalize_proposal().ledger()[0]
                .source_generation()
                .value(),
            7
        );
        assert_eq!(
            proposal.finalize_proposal().ledger()[1]
                .source_generation()
                .value(),
            8
        );
        assert!(!proposal.production_allowed());

        let stream_sha256: [u8; 32] = Sha256::digest(&stream).into();
        let candidate = Flat7xElementaryStreamStageReceiptCandidate::new(
            &proposal,
            access_unit_bytes() * 2,
            2,
            stream_sha256,
            true,
            true,
        );
        let staged_after = proposal.complete(candidate).unwrap();
        assert_eq!(
            staged_after.finalize_proposal().attempt_id(),
            "flat7x-stage-attempt"
        );
        assert_eq!(staged_after.finalize_proposal().owner_generation(), 29);
        assert_eq!(staged_after.receipt().readback_sha256(), &stream_sha256);
        assert!(!staged_after.production_allowed());
        assert!(!staged_after.final_exposure_allowed());

        let mut wrong_stream = stream.clone();
        wrong_stream[access_unit_bytes() as usize + 100] ^= 1;
        assert_eq!(
            Flat7xElementaryStreamStageProposal::new(target, &wrong_stream, &finalize).unwrap_err(),
            Flat7xElementaryStreamStageProposalError::AccessUnitLedgerHashMismatch { au_index: 1 }
        );

        let other_finalize = synthetic_finalize_proposal("other-attempt", 29);
        let proposal =
            Flat7xElementaryStreamStageProposal::new(target, &stream, &finalize).unwrap();
        let other_proposal =
            Flat7xElementaryStreamStageProposal::new(target, &stream, &other_finalize).unwrap();
        let other_receipt = Flat7xElementaryStreamStageReceiptCandidate::new(
            &other_proposal,
            access_unit_bytes() * 2,
            2,
            stream_sha256,
            true,
            true,
        );
        assert_eq!(
            proposal.complete(other_receipt).err(),
            Some(Flat7xElementaryStreamStageProposalError::StageReceiptAttemptMismatch)
        );
    }

    #[test]
    fn partial_aggregate_append_receipt_is_rejected_as_one_au() {
        let ledger = ledger_entry();
        let receipt = Flat7xAuAppendReceiptCandidate::new(
            "flat7x-attempt".to_owned(),
            PublishedAuIndex::new(0),
            0,
            6_655,
            vec![6_655],
            [0x33; 32],
            0,
            0,
            true,
            true,
        );

        assert_eq!(
            validate_append_receipt_candidate("flat7x-attempt", &ledger, &receipt),
            Err(Flat7xAuOutputOwnerError::AppendReceiptCandidateMismatch)
        );
    }

    #[test]
    fn owner_requires_f13_and_both_frame_integrity_receipts_before_staging() {
        let owner = Flat7xAuOutputOwner::cold("flat7x-attempt", 1).unwrap();

        let retirement = owner
            .stage_next_access_unit(
                candidate_sequence(),
                Flat7xAuSourceReceiptInput::new(None, None, None),
            )
            .unwrap_err();

        assert_eq!(
            retirement.reason(),
            Flat7xAuAttemptRetirementReason::StageRejected(
                Flat7xAuOutputOwnerError::F13MetadataReceiptMissing
            )
        );
        assert_eq!(retirement.attempt_id(), "flat7x-attempt");
        assert_eq!(retirement.owner_generation(), 0);
        assert_eq!(retirement.next_access_unit().value(), 0);
        assert_eq!(retirement.durable_length(), 0);
        assert!(retirement.committed_owner_unchanged());
        assert!(retirement.fresh_programme_cold_required());
    }

    #[test]
    fn finalize_proposal_retires_an_attempt_with_missing_access_units() {
        let owner = Flat7xAuOutputOwner::cold("flat7x-attempt", 1).unwrap();

        let retirement = owner.prepare_finalize().unwrap_err();

        assert_eq!(
            retirement.reason(),
            Flat7xAuAttemptRetirementReason::FinalizeIncomplete
        );
        assert_eq!(retirement.owner_generation(), 0);
        assert_eq!(retirement.next_access_unit().value(), 0);
        assert_eq!(retirement.durable_length(), 0);
    }

    #[test]
    fn source_generation_published_au_and_owner_generation_remain_distinct() {
        let owner = Flat7xAuOutputOwner::cold("flat7x-attempt", 1).unwrap();
        let sequence = candidate_sequence();

        assert_eq!(sequence.source_generation().value(), 7);
        assert_eq!(owner.next_access_unit().value(), 0);
        assert_eq!(owner.generation(), 0);
    }

    #[test]
    fn source_receipt_gate_rejects_each_missing_f13_or_frame_integrity_component() {
        assert_eq!(
            missing_source_receipt(false, false, false),
            Some(Flat7xAuOutputOwnerError::F13MetadataReceiptMissing)
        );
        assert_eq!(
            missing_source_receipt(true, false, true),
            Some(Flat7xAuOutputOwnerError::I0IntegrityReceiptMissing)
        );
        assert_eq!(
            missing_source_receipt(true, true, false),
            Some(Flat7xAuOutputOwnerError::D0IntegrityReceiptMissing)
        );
        assert_eq!(missing_source_receipt(true, true, true), None);
    }

    #[test]
    fn owner_rejects_empty_attempt_and_empty_access_unit_plan() {
        assert_eq!(
            Flat7xAuOutputOwner::cold("", 1),
            Err(Flat7xAuOutputOwnerError::EmptyAttemptId)
        );
        assert_eq!(
            Flat7xAuOutputOwner::cold("flat7x-attempt", 0),
            Err(Flat7xAuOutputOwnerError::EmptyAccessUnitCount)
        );
    }
}
