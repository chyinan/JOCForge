// pattern: Functional Core
//! Target-only F15 staging values for a whole two-frame Flat7x access unit.

use sha2::{Digest, Sha256};

use crate::{
    integrity::crc16::eac3_crc16,
    profile::{Flat7xAuByteSequenceCandidate, Flat7xI0D0TargetPlan, Flat7xSourceGeneration},
    timeline::PublishedAuIndex,
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
}

/// One in-memory target-only output owner for complete Flat7x I0+D0 access units.
///
/// This owner models one aggregate AU append and keeps a pair-aware ledger. Receipt values are
/// caller-supplied candidates; this module cannot prove a real write, rollback, or readback
/// provenance. It has no canonical GTC registration, filesystem adapter, product-plan admission,
/// source-callback identity, or connection to the selected 5.1 sink.
#[derive(Clone, Debug, Eq, PartialEq)]
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

    pub const fn is_complete(&self) -> bool {
        self.next_access_unit.value() == self.expected_access_units
            && self.durable_length == self.expected_product_bytes()
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }

    /// Stages the next complete AU as one append proposal containing I0 followed by D0.
    pub fn stage_next_access_unit(
        &self,
        sequence: Flat7xAuByteSequenceCandidate,
    ) -> Result<Flat7xAuAppendProposal, Flat7xAuOutputOwnerError> {
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
            offset: expected_offset,
            length: access_unit_bytes(),
            i0_frame_bytes: plan.i0().frame_bytes(),
            d0_frame_bytes: plan.d0().frame_bytes(),
            i0_sha256,
            d0_sha256,
            au_sha256,
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
        let mut after_owner = self.clone();
        after_owner.generation = next_generation;
        after_owner.next_access_unit = PublishedAuIndex::new(next_au);
        after_owner.durable_length = next_length;
        after_owner.ledger.push(ledger_entry.clone());

        Ok(Flat7xAuAppendProposal {
            attempt_id: self.attempt_id.clone(),
            before_generation: self.generation,
            ledger_entry,
            sequence,
            after_owner,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xAuOutputLedgerEntry {
    au_index: PublishedAuIndex,
    source_generation: Flat7xSourceGeneration,
    offset: u64,
    length: u32,
    i0_frame_bytes: u16,
    d0_frame_bytes: u16,
    i0_sha256: [u8; 32],
    d0_sha256: [u8; 32],
    au_sha256: [u8; 32],
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

    pub const fn frame_crc_residuals(&self) -> (u16, u16) {
        (self.i0_crc_residual, self.d0_crc_residual)
    }
}

#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct Flat7xAuAppendProposal {
    attempt_id: String,
    before_generation: u64,
    ledger_entry: Flat7xAuOutputLedgerEntry,
    sequence: Flat7xAuByteSequenceCandidate,
    after_owner: Flat7xAuOutputOwner,
}

impl Flat7xAuAppendProposal {
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
    pub fn complete(
        self,
        receipt: Flat7xAuAppendReceiptCandidate,
    ) -> Result<Flat7xAuOutputAfterProposal, Flat7xAuOutputOwnerError> {
        let write_total = receipt.write_calls.iter().try_fold(0_u64, |total, count| {
            if *count == 0 {
                None
            } else {
                total.checked_add(*count)
            }
        });
        let expected_after_generation = self
            .before_generation
            .checked_add(1)
            .ok_or(Flat7xAuOutputOwnerError::AccessUnitCountOverflow)?;
        if receipt.attempt_id != self.attempt_id
            || receipt.au_index != self.ledger_entry.au_index
            || receipt.offset != self.ledger_entry.offset
            || receipt.bytes_written != self.ledger_entry.length
            || write_total != Some(u64::from(self.ledger_entry.length))
            || receipt.readback_sha256 != self.ledger_entry.au_sha256
            || receipt.i0_crc_residual != self.ledger_entry.i0_crc_residual
            || receipt.d0_crc_residual != self.ledger_entry.d0_crc_residual
            || receipt.i0_crc_residual != 0
            || receipt.d0_crc_residual != 0
            || !receipt.file_fsync
            || !receipt.durable
            || self.after_owner.generation != expected_after_generation
        {
            return Err(Flat7xAuOutputOwnerError::AppendReceiptCandidateMismatch);
        }

        Ok(Flat7xAuOutputAfterProposal {
            ledger_entry: self.ledger_entry,
            receipt,
            after_owner: self.after_owner,
        })
    }
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
#[derive(Debug, Eq, PartialEq)]
pub struct Flat7xAuOutputAfterProposal {
    ledger_entry: Flat7xAuOutputLedgerEntry,
    receipt: Flat7xAuAppendReceiptCandidate,
    after_owner: Flat7xAuOutputOwner,
}

impl Flat7xAuOutputAfterProposal {
    pub const fn ledger_entry(&self) -> &Flat7xAuOutputLedgerEntry {
        &self.ledger_entry
    }

    pub const fn receipt(&self) -> &Flat7xAuAppendReceiptCandidate {
        &self.receipt
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

    fn matching_receipt(proposal: &Flat7xAuAppendProposal) -> Flat7xAuAppendReceiptCandidate {
        Flat7xAuAppendReceiptCandidate::new(
            proposal.attempt_id().to_owned(),
            proposal.ledger_entry().au_index(),
            proposal.ledger_entry().offset(),
            proposal.ledger_entry().length(),
            vec![2_000, 4_656],
            *proposal.ledger_entry().au_sha256(),
            0,
            0,
            true,
            true,
        )
    }

    #[test]
    fn owner_stages_one_whole_au_append_and_records_both_frame_checks() {
        let owner = Flat7xAuOutputOwner::cold("flat7x-attempt", 2).unwrap();
        let proposal = owner.stage_next_access_unit(candidate_sequence()).unwrap();

        assert_eq!(proposal.bytes().len(), 6_656);
        assert_eq!(proposal.ledger_entry().au_index().value(), 0);
        assert_eq!(proposal.ledger_entry().offset(), 0);
        assert_eq!(proposal.ledger_entry().frame_lengths(), (2_560, 4_096));
        assert_eq!(proposal.ledger_entry().frame_crc_residuals(), (0, 0));
        assert!(!proposal.production_allowed());

        let receipt = matching_receipt(&proposal);
        let after = proposal.complete(receipt).unwrap();
        assert_eq!(after.ledger_entry().length(), 6_656);
        assert_eq!(after.after_owner().generation(), 1);
        assert_eq!(after.after_owner().next_access_unit().value(), 1);
        assert_eq!(after.after_owner().durable_length(), 6_656);
        assert_eq!(after.after_owner().ledger().len(), 1);
        assert_eq!(after.after_owner().expected_product_bytes(), 13_312);
        assert!(!after.after_owner().is_complete());
        let next_owner = after.into_after_owner();
        let next_proposal = next_owner
            .stage_next_access_unit(candidate_sequence())
            .unwrap();
        assert_eq!(next_proposal.ledger_entry().au_index().value(), 1);
        assert_eq!(next_proposal.ledger_entry().offset(), 6_656);
    }

    #[test]
    fn incomplete_receipt_returns_no_after_owner_and_preserves_cold_root() {
        let owner = Flat7xAuOutputOwner::cold("flat7x-attempt", 1).unwrap();
        let proposal = owner.stage_next_access_unit(candidate_sequence()).unwrap();
        let mut receipt = matching_receipt(&proposal);
        receipt.bytes_written = 6_655;

        assert_eq!(
            proposal.complete(receipt),
            Err(Flat7xAuOutputOwnerError::AppendReceiptCandidateMismatch)
        );
        assert_eq!(owner.generation(), 0);
        assert_eq!(owner.next_access_unit().value(), 0);
        assert_eq!(owner.durable_length(), 0);
        assert!(owner.ledger().is_empty());
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
