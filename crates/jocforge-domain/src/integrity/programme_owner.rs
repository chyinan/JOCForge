// pattern: Functional Core
//! Programme-local keyless integrity owner with pure stage calculations.

use super::{
    crc16::eac3_crc16,
    keyless_checksum::{
        EmdfProtectionBitRange, IntegritySha256, KeylessIntegrityProvider, KeylessProviderError,
        ProtectedEmdfContainer, crc8_smbus, crc32_iso_hdlc, sha256, validate_keyless_provider,
    },
};
use crate::{
    codec::bitio::FrameBitCursor,
    codec::frame_size::Eac3FrameSize,
    timeline::{CodecSourceIndex, JocDispatchIndex, PublishedAuIndex, source_for_callback},
    transaction::{
        AttemptId, CallbackLineageIdentity, CallbackTransactionId, GtcOwnerId,
        GtcOwnerReceiptError, GtcOwnerStateSnapshot, ProgrammeId,
    },
};
use std::{error::Error, fmt};

const PROFILE_ID: &str = "keyless-crc32-crc8-crc16.v1";
const RECEIPT_SCHEMA: &str = "integrity-stage-receipt.v1";
const OWNER_ID: &str = "integrity";
const FRAME_CRC_INPUT_START: usize = 2;
const SUPPRESSED_SOURCE_COUNT: u64 = 2;
const GTC_STATE_REF: &str = "owners.integrity";

fn integrity_profile_id(codec_rate: crate::profile::CodecRateProfile) -> &'static str {
    match codec_rate {
        crate::profile::CodecRateProfile::Kbps384 => {
            "keyless-crc32-crc8-crc16.v1/48khz_384kbps_1536bytes"
        }
        crate::profile::CodecRateProfile::Kbps448 => {
            "keyless-crc32-crc8-crc16.v1/48khz_448kbps_1792bytes"
        }
        crate::profile::CodecRateProfile::Kbps576 => {
            "keyless-crc32-crc8-crc16.v1/48khz_576kbps_2304bytes"
        }
        crate::profile::CodecRateProfile::Kbps640 => {
            "keyless-crc32-crc8-crc16.v1/48khz_640kbps_2560bytes"
        }
        crate::profile::CodecRateProfile::Kbps768 => {
            "keyless-crc32-crc8-crc16.v1/48khz_768kbps_3072bytes"
        }
        crate::profile::CodecRateProfile::Kbps1024
        | crate::profile::CodecRateProfile::Unsupported => PROFILE_ID,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrityIdentity {
    pub programme_id: String,
    pub attempt: u64,
    pub transaction_id: String,
    pub source_index: CodecSourceIndex,
    pub dispatch: JocDispatchIndex,
    pub au_index: Option<PublishedAuIndex>,
}

pub struct IntegrityInput<'a> {
    pub identity: IntegrityIdentity,
    pub zero_slot_container: &'a [u8],
    pub primary_slot_offset: usize,
    pub secondary_slot_offset: usize,
    pub frame_template: &'a [u8],
    pub container_offset: usize,
}

/// F14K result bound to the source input before F12 owns the frame write.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum F14kPayloadInput<'a> {
    /// Sources 0/1 have no published EMDF payload.
    AbsentPublishedPayloadV1,
    /// F14K-protected bytes and their exact zero-slot F13 input.
    ProtectedEmdf {
        zero_slot_container: &'a [u8],
        protected_container: &'a ProtectedEmdfContainer,
    },
}

/// Complete F12 output offered to the Integrity owner as immutable evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct F12CompletedFrameInput<'a> {
    pub frame: &'a [u8],
    /// F12's live cursor immediately before EMDF carrier bytes, absent for source 0/1.
    pub emdf_container_start: Option<FrameBitCursor>,
}

/// One same-source F14K payload outcome and F12 completed frame.
pub struct IntegrityF12SourceInput<'a> {
    pub identity: IntegrityIdentity,
    pub payload: F14kPayloadInput<'a>,
    pub completed_frame: F12CompletedFrameInput<'a>,
}

/// Both completed frames and D0's protected F13 payload for one Flat7x access unit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xI0D0F12PairInput<'a> {
    pub identity: IntegrityIdentity,
    pub rate_plan: Flat7xIntegrityPairRatePlan,
    pub i0_frame: &'a [u8],
    pub d0_frame: &'a [u8],
    pub d0_emdf_container_start: FrameBitCursor,
    pub d0_zero_slot_container: &'a [u8],
    pub d0_protected_container: &'a ProtectedEmdfContainer,
}

/// Fixed role-specific codec rates for the current Flat7x I0/D0 transaction model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xIntegrityPairRatePlan {
    independent_i0_rate: crate::profile::CodecRateProfile,
    dependent_d0_rate: crate::profile::CodecRateProfile,
}

impl Flat7xIntegrityPairRatePlan {
    pub const fn flat7x_i0_640_d0_1024() -> Self {
        Self {
            independent_i0_rate: crate::profile::CodecRateProfile::Kbps640,
            dependent_d0_rate: crate::profile::CodecRateProfile::Kbps1024,
        }
    }

    pub const fn independent_i0_rate(self) -> crate::profile::CodecRateProfile {
        self.independent_i0_rate
    }

    pub const fn dependent_d0_rate(self) -> crate::profile::CodecRateProfile {
        self.dependent_d0_rate
    }

    pub const fn production_allowed(self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrityStage {
    Source,
    Dispatch,
    Au,
}

impl IntegrityStage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Source => "SOURCE",
            Self::Dispatch => "DISPATCH",
            Self::Au => "AU",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrityErrorCode {
    UnsupportedProfile,
    ProviderValues,
    ProgrammeIdentity,
    AttemptIdentity,
    TransactionIdentity,
    SourceOrder,
    DispatchOrder,
    AuOrder,
    ChecksumSlots,
    FrameShape,
    SourceFailed,
    DispatchFailed,
    AuFailed,
    AttemptRetired,
    AlreadyFinalized,
    IncompleteFinalize,
    GenerationOverflow,
    AttemptReuse,
}

impl IntegrityErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedProfile => "UNSUPPORTED_PROFILE",
            Self::ProviderValues => "INTEGRITY_PROVIDER_VALUES",
            Self::ProgrammeIdentity => "INTEGRITY_PROGRAMME_IDENTITY",
            Self::AttemptIdentity => "INTEGRITY_ATTEMPT_IDENTITY",
            Self::TransactionIdentity => "INTEGRITY_TRANSACTION_IDENTITY",
            Self::SourceOrder => "INTEGRITY_SOURCE_ORDER",
            Self::DispatchOrder => "INTEGRITY_DISPATCH_ORDER",
            Self::AuOrder => "INTEGRITY_AU_ORDER",
            Self::ChecksumSlots => "INTEGRITY_CHECKSUM_SLOTS",
            Self::FrameShape => "INTEGRITY_FRAME_SHAPE",
            Self::SourceFailed => "INTEGRITY_SOURCE_FAILED",
            Self::DispatchFailed => "INTEGRITY_DISPATCH_FAILED",
            Self::AuFailed => "INTEGRITY_AU_FAILED",
            Self::AttemptRetired => "INTEGRITY_ATTEMPT_RETIRED",
            Self::AlreadyFinalized => "INTEGRITY_ALREADY_FINALIZED",
            Self::IncompleteFinalize => "INTEGRITY_FINALIZE_INCOMPLETE",
            Self::GenerationOverflow => "INTEGRITY_GENERATION_OVERFLOW",
            Self::AttemptReuse => "INTEGRITY_ATTEMPT_REUSE",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegrityError {
    code: IntegrityErrorCode,
}

impl IntegrityError {
    const fn new(code: IntegrityErrorCode) -> Self {
        Self { code }
    }

    pub const fn code(self) -> IntegrityErrorCode {
        self.code
    }
}

impl fmt::Display for IntegrityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code.as_str())
    }
}

impl Error for IntegrityError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrityState {
    profile_id: &'static str,
    provider_sha256: IntegritySha256,
    generation: u64,
    next_source_index: CodecSourceIndex,
    published_au_count: u64,
    finalized: bool,
}

impl IntegrityState {
    pub const fn profile_id(&self) -> &'static str {
        self.profile_id
    }

    pub const fn provider_sha256(&self) -> IntegritySha256 {
        self.provider_sha256
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn next_source_index(&self) -> CodecSourceIndex {
        self.next_source_index
    }

    pub const fn published_au_count(&self) -> u64 {
        self.published_au_count
    }

    pub const fn finalized(&self) -> bool {
        self.finalized
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let profile = self.profile_id.as_bytes();
        let profile_bytes = profile.len() as u16;
        let mut bytes = Vec::with_capacity(17 + 2 + profile.len() + 32 + 26);
        bytes.extend_from_slice(b"IntegrityStateV1\0");
        bytes.extend_from_slice(&profile_bytes.to_le_bytes());
        bytes.extend_from_slice(profile);
        bytes.extend_from_slice(self.provider_sha256.as_bytes());
        bytes.extend_from_slice(&self.generation.to_le_bytes());
        bytes.extend_from_slice(&self.next_source_index.value().to_le_bytes());
        bytes.extend_from_slice(&self.published_au_count.to_le_bytes());
        bytes.push(u8::from(self.finalized));
        bytes.push(0);
        bytes
    }

    pub fn sha256(&self) -> IntegritySha256 {
        sha256(&self.canonical_bytes())
    }
}

/// Immutable owner-state value projected for the selected GTC ownerState envelope.
///
/// `state_ref` is the approved logical member name. The complete typed state and
/// canonical bytes travel with the snapshot; the reference does not replace either.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrityOwnerStateSnapshot {
    state: IntegrityState,
    canonical_bytes: Vec<u8>,
    state_sha256: IntegritySha256,
    cold_sha256: IntegritySha256,
}

impl IntegrityOwnerStateSnapshot {
    fn from_state(state: &IntegrityState) -> Self {
        let canonical_bytes = state.canonical_bytes();
        let state_sha256 = sha256(&canonical_bytes);
        let cold_state = IntegrityState {
            generation: 0,
            next_source_index: CodecSourceIndex::new(0),
            published_au_count: 0,
            finalized: false,
            ..state.clone()
        };
        let cold_sha256 = cold_state.sha256();
        Self {
            state: state.clone(),
            canonical_bytes,
            state_sha256,
            cold_sha256,
        }
    }

    pub const fn owner_id(&self) -> &'static str {
        OWNER_ID
    }

    pub const fn state_ref(&self) -> &'static str {
        GTC_STATE_REF
    }

    pub const fn state(&self) -> &IntegrityState {
        &self.state
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub const fn generation(&self) -> u64 {
        self.state.generation()
    }

    pub const fn state_sha256(&self) -> IntegritySha256 {
        self.state_sha256
    }

    pub const fn cold_sha256(&self) -> IntegritySha256 {
        self.cold_sha256
    }

    /// Adapts the complete owner snapshot to the registered GTC state member.
    pub fn gtc_snapshot(&self) -> Result<GtcOwnerStateSnapshot<'_>, GtcOwnerReceiptError> {
        GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::Integrity,
            self.state.generation(),
            *self.cold_sha256.as_bytes(),
            &self.canonical_bytes,
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum IntegrityStageDetails {
    Source {
        primary: [u8; 4],
        secondary: [u8; 1],
        message_sha256: IntegritySha256,
        container_bytes: usize,
    },
    Dispatch {
        container_offset: usize,
        frame_bytes: usize,
    },
    Au {
        crc16: u16,
        crc_residual: u16,
        published: bool,
    },
}

impl IntegrityStageDetails {
    #[cfg(test)]
    fn crc16(&self) -> Option<u16> {
        match self {
            Self::Au { crc16, .. } => Some(*crc16),
            _ => None,
        }
    }

    #[cfg(test)]
    fn crc_residual(&self) -> Option<u16> {
        match self {
            Self::Au { crc_residual, .. } => Some(*crc_residual),
            _ => None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct IntegrityStageReceipt {
    schema: &'static str,
    owner_id: &'static str,
    stage: IntegrityStage,
    identity: IntegrityIdentity,
    before_generation: u64,
    before_sha256: IntegritySha256,
    proposed_after_generation: u64,
    proposed_after_sha256: IntegritySha256,
    provider_id: &'static str,
    provider_sha256: IntegritySha256,
    input_sha256: IntegritySha256,
    output_sha256: IntegritySha256,
    details: IntegrityStageDetails,
}

impl IntegrityStageReceipt {
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    pub const fn owner_id(&self) -> &'static str {
        self.owner_id
    }

    pub const fn stage(&self) -> IntegrityStage {
        self.stage
    }

    pub const fn identity(&self) -> &IntegrityIdentity {
        &self.identity
    }

    pub const fn before_generation(&self) -> u64 {
        self.before_generation
    }

    pub const fn before_sha256(&self) -> IntegritySha256 {
        self.before_sha256
    }

    pub const fn proposed_after_generation(&self) -> u64 {
        self.proposed_after_generation
    }

    pub const fn proposed_after_sha256(&self) -> IntegritySha256 {
        self.proposed_after_sha256
    }

    pub const fn provider_id(&self) -> &'static str {
        self.provider_id
    }

    pub const fn provider_sha256(&self) -> IntegritySha256 {
        self.provider_sha256
    }

    pub const fn input_sha256(&self) -> IntegritySha256 {
        self.input_sha256
    }

    pub const fn output_sha256(&self) -> IntegritySha256 {
        self.output_sha256
    }

    pub const fn details(&self) -> &IntegrityStageDetails {
        &self.details
    }
}

/// F14K evidence for an absent or protected source payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrityPayloadEvidence {
    AbsentPublishedPayloadV1,
    ProtectedEmdf {
        zero_slot_sha256: IntegritySha256,
        protected_sha256: IntegritySha256,
        message_sha256: IntegritySha256,
        primary_crc32: u32,
        secondary_crc8: u8,
        primary_range: EmdfProtectionBitRange,
        secondary_range: EmdfProtectionBitRange,
        container_bytes: usize,
    },
}

/// F14 SOURCE-stage receipt paired with the Integrity owner transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrityF12SourceReceipt {
    identity: IntegrityIdentity,
    before_generation: u64,
    proposed_after_generation: u64,
    before_sha256: IntegritySha256,
    proposed_after_sha256: IntegritySha256,
    provider_sha256: IntegritySha256,
    payload: IntegrityPayloadEvidence,
}

impl IntegrityF12SourceReceipt {
    pub const fn identity(&self) -> &IntegrityIdentity {
        &self.identity
    }

    pub const fn before_generation(&self) -> u64 {
        self.before_generation
    }

    pub const fn proposed_after_generation(&self) -> u64 {
        self.proposed_after_generation
    }

    pub const fn before_sha256(&self) -> IntegritySha256 {
        self.before_sha256
    }

    pub const fn proposed_after_sha256(&self) -> IntegritySha256 {
        self.proposed_after_sha256
    }

    pub const fn provider_sha256(&self) -> IntegritySha256 {
        self.provider_sha256
    }

    pub const fn payload(&self) -> IntegrityPayloadEvidence {
        self.payload
    }
}

/// F12 DISPATCH-stage receipt for carrier placement and frame production.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrityF12DispatchReceipt {
    identity: IntegrityIdentity,
    frame_sha256: IntegritySha256,
    frame_bytes: usize,
    emdf_container_start: Option<FrameBitCursor>,
    source_payload_sha256: Option<IntegritySha256>,
}

impl IntegrityF12DispatchReceipt {
    pub const fn identity(&self) -> &IntegrityIdentity {
        &self.identity
    }

    pub const fn frame_sha256(&self) -> IntegritySha256 {
        self.frame_sha256
    }

    pub const fn frame_bytes(&self) -> usize {
        self.frame_bytes
    }

    pub const fn emdf_container_start(&self) -> Option<FrameBitCursor> {
        self.emdf_container_start
    }

    pub const fn source_payload_sha256(&self) -> Option<IntegritySha256> {
        self.source_payload_sha256
    }
}

/// F12 AU-stage receipt; `published` distinguishes priming CRCs from AU writes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrityF12AuReceipt {
    identity: IntegrityIdentity,
    frame_sha256: IntegritySha256,
    crc16: u16,
    crc_residual: u16,
    published: bool,
}

impl IntegrityF12AuReceipt {
    pub const fn identity(&self) -> &IntegrityIdentity {
        &self.identity
    }

    pub const fn frame_sha256(&self) -> IntegritySha256 {
        self.frame_sha256
    }

    pub const fn crc16(&self) -> u16 {
        self.crc16
    }

    pub const fn crc_residual(&self) -> u16 {
        self.crc_residual
    }

    pub const fn published(&self) -> bool {
        self.published
    }
}

/// Typed F14 SOURCE, F12 DISPATCH, and F12 AU evidence for one noncommitting transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrityF12SourceReceipts {
    source: IntegrityF12SourceReceipt,
    dispatch: IntegrityF12DispatchReceipt,
    au: IntegrityF12AuReceipt,
}

impl IntegrityF12SourceReceipts {
    pub const fn source(&self) -> &IntegrityF12SourceReceipt {
        &self.source
    }

    pub const fn dispatch(&self) -> &IntegrityF12DispatchReceipt {
        &self.dispatch
    }

    pub const fn au(&self) -> &IntegrityF12AuReceipt {
        &self.au
    }
}

/// Builds typed synthetic receipt values for receipt-gate unit tests only.
///
/// This fixture bypasses the Integrity owner stage; it does not establish F12
/// producer integration or source-generation ownership.
#[cfg(any(test, feature = "test-support"))]
pub(crate) fn synthetic_f12_receipts_for_flat7x_test(
    identity: IntegrityIdentity,
    generations: (u64, u64),
    frame: &[u8],
    payload: IntegrityPayloadEvidence,
    emdf_container_start: Option<FrameBitCursor>,
    source_payload_sha256: Option<IntegritySha256>,
) -> IntegrityF12SourceReceipts {
    let frame_sha256 = sha256(frame);
    let crc16_offset = frame
        .len()
        .checked_sub(2)
        .expect("synthetic receipt frame includes CRC16");
    let crc16 = u16::from_be_bytes([frame[crc16_offset], frame[crc16_offset + 1]]);

    IntegrityF12SourceReceipts {
        source: IntegrityF12SourceReceipt {
            identity: identity.clone(),
            before_generation: generations.0,
            proposed_after_generation: generations.1,
            before_sha256: sha256(&generations.0.to_le_bytes()),
            proposed_after_sha256: sha256(&generations.1.to_le_bytes()),
            provider_sha256: sha256(b"wash synthetic receipt-gate provider"),
            payload,
        },
        dispatch: IntegrityF12DispatchReceipt {
            identity: identity.clone(),
            frame_sha256,
            frame_bytes: frame.len(),
            emdf_container_start,
            source_payload_sha256,
        },
        au: IntegrityF12AuReceipt {
            identity,
            frame_sha256,
            crc16,
            crc_residual: eac3_crc16(&frame[FRAME_CRC_INPUT_START..]),
            published: true,
        },
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct IntegrityCallResult {
    receipts: [IntegrityStageReceipt; 3],
    frame: Vec<u8>,
    before_sha256: IntegritySha256,
    after_sha256: IntegritySha256,
    commit_count: u8,
}

impl IntegrityCallResult {
    pub const fn receipts(&self) -> &[IntegrityStageReceipt; 3] {
        &self.receipts
    }

    pub fn frame(&self) -> &[u8] {
        &self.frame
    }

    pub const fn before_sha256(&self) -> IntegritySha256 {
        self.before_sha256
    }

    pub const fn after_sha256(&self) -> IntegritySha256 {
        self.after_sha256
    }

    pub const fn commit_count(&self) -> u8 {
        self.commit_count
    }
}

#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct IntegritySourceProposal {
    after_owner: IntegrityProgrammeOwner,
    result: IntegrityCallResult,
}

impl IntegritySourceProposal {
    pub const fn after_state(&self) -> &IntegrityState {
        &self.after_owner.state
    }

    pub const fn result(&self) -> &IntegrityCallResult {
        &self.result
    }

    pub fn into_parts(self) -> (IntegrityProgrammeOwner, IntegrityCallResult) {
        (self.after_owner, self.result)
    }
}

/// Noncommitting projection of one source proposal into the GTC owner-receipt fields.
///
/// This value deliberately has no `commit_count`: only the enclosing GTC parent
/// can admit a callback and decide whether the owner commit count is zero or one.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct IntegrityOwnerReceiptProposal {
    programme_id: ProgrammeId,
    attempt: AttemptId,
    transaction_id: CallbackTransactionId,
    callback_identity: CallbackLineageIdentity,
    before: IntegrityOwnerStateSnapshot,
    after: IntegrityOwnerStateSnapshot,
    changed: bool,
}

impl IntegrityOwnerReceiptProposal {
    pub const fn owner_id(&self) -> &'static str {
        OWNER_ID
    }

    pub fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub const fn attempt(&self) -> AttemptId {
        self.attempt
    }

    pub fn transaction_id(&self) -> &CallbackTransactionId {
        &self.transaction_id
    }

    pub fn callback_identity(&self) -> &CallbackLineageIdentity {
        &self.callback_identity
    }

    pub const fn before(&self) -> &IntegrityOwnerStateSnapshot {
        &self.before
    }

    pub const fn after(&self) -> &IntegrityOwnerStateSnapshot {
        &self.after
    }

    pub const fn before_generation(&self) -> u64 {
        self.before.generation()
    }

    pub const fn after_generation(&self) -> u64 {
        self.after.generation()
    }

    pub const fn before_sha256(&self) -> IntegritySha256 {
        self.before.state_sha256()
    }

    pub const fn after_sha256(&self) -> IntegritySha256 {
        self.after.state_sha256()
    }

    pub const fn changed(&self) -> bool {
        self.changed
    }
}

/// Owner-local staged source plus its prospective GTC state/receipt projection.
///
/// Consuming this value yields the replacement owner but never installs it. The
/// programme parent decides whether to use the staged result or discard it.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct IntegrityGtcSourceProposal {
    source: IntegritySourceProposal,
    owner_receipt: IntegrityOwnerReceiptProposal,
}

impl IntegrityGtcSourceProposal {
    pub const fn source_proposal(&self) -> &IntegritySourceProposal {
        &self.source
    }

    pub const fn owner_receipt_proposal(&self) -> &IntegrityOwnerReceiptProposal {
        &self.owner_receipt
    }

    pub fn into_parts(
        self,
    ) -> (
        IntegrityProgrammeOwner,
        IntegrityCallResult,
        IntegrityOwnerReceiptProposal,
    ) {
        let (after_owner, result) = self.source.into_parts();
        (after_owner, result, self.owner_receipt)
    }
}

/// Noncommitting Integrity root and typed F12-owned receipt set for one source.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct IntegrityF12SourceProposal {
    after_owner: IntegrityProgrammeOwner,
    receipts: IntegrityF12SourceReceipts,
}

impl IntegrityF12SourceProposal {
    pub const fn after_owner(&self) -> &IntegrityProgrammeOwner {
        &self.after_owner
    }

    pub const fn receipts(&self) -> &IntegrityF12SourceReceipts {
        &self.receipts
    }

    pub fn into_parts(self) -> (IntegrityProgrammeOwner, IntegrityF12SourceReceipts) {
        (self.after_owner, self.receipts)
    }
}

/// F12 source proposal paired with the corresponding prospective GTC owner receipt.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct IntegrityGtcF12SourceProposal {
    source: IntegrityF12SourceProposal,
    owner_receipt: IntegrityOwnerReceiptProposal,
}

impl IntegrityGtcF12SourceProposal {
    pub const fn source_proposal(&self) -> &IntegrityF12SourceProposal {
        &self.source
    }

    pub const fn owner_receipt_proposal(&self) -> &IntegrityOwnerReceiptProposal {
        &self.owner_receipt
    }

    pub fn into_parts(
        self,
    ) -> (
        IntegrityProgrammeOwner,
        IntegrityF12SourceReceipts,
        IntegrityOwnerReceiptProposal,
    ) {
        let (after_owner, receipts) = self.source.into_parts();
        (after_owner, receipts, self.owner_receipt)
    }
}

/// Typed I0/D0 frame receipts that share one AU identity and one owner transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xI0D0F12PairReceipts {
    i0: IntegrityF12SourceReceipts,
    d0: IntegrityF12SourceReceipts,
}

impl Flat7xI0D0F12PairReceipts {
    pub const fn i0(&self) -> &IntegrityF12SourceReceipts {
        &self.i0
    }

    pub const fn d0(&self) -> &IntegrityF12SourceReceipts {
        &self.d0
    }
}

/// Noncommitting Flat7x pair F12 receipts with one shared Integrity owner transition.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct IntegrityGtcFlat7xF12PairProposal {
    after_owner: IntegrityProgrammeOwner,
    receipts: Flat7xI0D0F12PairReceipts,
    owner_receipt: IntegrityOwnerReceiptProposal,
}

impl IntegrityGtcFlat7xF12PairProposal {
    pub const fn after_owner(&self) -> &IntegrityProgrammeOwner {
        &self.after_owner
    }

    pub const fn receipts(&self) -> &Flat7xI0D0F12PairReceipts {
        &self.receipts
    }

    pub const fn owner_receipt_proposal(&self) -> &IntegrityOwnerReceiptProposal {
        &self.owner_receipt
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }

    pub fn into_parts(
        self,
    ) -> (
        IntegrityProgrammeOwner,
        Flat7xI0D0F12PairReceipts,
        IntegrityOwnerReceiptProposal,
    ) {
        (self.after_owner, self.receipts, self.owner_receipt)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct IntegrityProgrammeOwner {
    state: IntegrityState,
    programme_id: String,
    attempt: u64,
    codec_rate: crate::profile::CodecRateProfile,
    expected_source_count: u64,
    retired: bool,
}

impl IntegrityProgrammeOwner {
    /// Admits a cold owner only after validating the exact selected public keyless provider.
    pub fn cold(
        programme_id: &str,
        attempt: u64,
        expected_source_count: u64,
        provider_json: &[u8],
    ) -> Result<Self, IntegrityError> {
        let provider = validate_keyless_provider(provider_json).map_err(map_provider_error)?;
        Self::cold_from_provider(programme_id, attempt, expected_source_count, &provider)
    }

    /// Admits a cold owner from the keyless provider already retained by F15 admission.
    pub fn cold_from_provider(
        programme_id: &str,
        attempt: u64,
        expected_source_count: u64,
        provider: &KeylessIntegrityProvider,
    ) -> Result<Self, IntegrityError> {
        Self::cold_from_provider_for_rate(
            programme_id,
            attempt,
            expected_source_count,
            provider,
            crate::profile::CodecRateProfile::Kbps1024,
        )
    }

    /// Admits the checksum provider for one supported selected E-AC-3 carrier rate.
    pub fn cold_from_provider_for_rate(
        programme_id: &str,
        attempt: u64,
        expected_source_count: u64,
        provider: &KeylessIntegrityProvider,
        codec_rate: crate::profile::CodecRateProfile,
    ) -> Result<Self, IntegrityError> {
        if programme_id.is_empty() {
            return Err(IntegrityError::new(IntegrityErrorCode::ProgrammeIdentity));
        }
        if provider.profile_id() != PROFILE_ID
            || Eac3FrameSize::from_codec_rate(codec_rate).is_none()
        {
            return Err(IntegrityError::new(IntegrityErrorCode::UnsupportedProfile));
        }
        Ok(Self {
            state: IntegrityState {
                profile_id: integrity_profile_id(codec_rate),
                provider_sha256: provider.provider_sha256(),
                generation: 0,
                next_source_index: CodecSourceIndex::new(0),
                published_au_count: 0,
                finalized: false,
            },
            programme_id: programme_id.to_owned(),
            attempt,
            codec_rate,
            expected_source_count,
            retired: false,
        })
    }

    /// Runs SOURCE, DISPATCH and AU as provisional stages, then publishes one owner commit.
    pub fn process(
        &mut self,
        input: IntegrityInput<'_>,
        provider_json: &[u8],
    ) -> Result<IntegrityCallResult, IntegrityError> {
        if self.retired {
            return Err(IntegrityError::new(IntegrityErrorCode::AttemptRetired));
        }
        let result = self.stage_source(input, provider_json);
        match result {
            Ok(proposal) => {
                let (after_owner, result) = proposal.into_parts();
                *self = after_owner;
                Ok(result)
            }
            Err(error) => {
                self.retired = true;
                Err(error)
            }
        }
    }

    /// Stages SOURCE, DISPATCH and AU without replacing or retiring this owner.
    pub fn stage_source(
        &self,
        input: IntegrityInput<'_>,
        provider_json: &[u8],
    ) -> Result<IntegritySourceProposal, IntegrityError> {
        if self.retired {
            return Err(IntegrityError::new(IntegrityErrorCode::AttemptRetired));
        }
        if self.state.finalized {
            return Err(IntegrityError::new(IntegrityErrorCode::AlreadyFinalized));
        }
        let before_sha256 = self.state.sha256();
        let (result, next_state) =
            self.stage_call(input, provider_json, before_sha256, &mut |_| false)?;
        Ok(IntegritySourceProposal {
            after_owner: self.with_state(next_state),
            result,
        })
    }

    /// Stages one source and projects its owner snapshot against the typed GTC callback identity.
    ///
    /// This production-facing adapter uses `stage_source` and returns only a proposal. It does
    /// not replace the committed owner or assign a GTC commit count.
    pub fn stage_source_for_callback(
        &self,
        input: IntegrityInput<'_>,
        provider_json: &[u8],
        programme_id: &ProgrammeId,
        attempt: AttemptId,
        transaction_id: &CallbackTransactionId,
        callback_identity: &CallbackLineageIdentity,
    ) -> Result<IntegrityGtcSourceProposal, IntegrityError> {
        if self.retired {
            return Err(IntegrityError::new(IntegrityErrorCode::AttemptRetired));
        }
        if self.state.finalized {
            return Err(IntegrityError::new(IntegrityErrorCode::AlreadyFinalized));
        }
        if self.programme_id != programme_id.as_str()
            || input.identity.programme_id != programme_id.as_str()
            || callback_identity.programme_id() != programme_id
        {
            return Err(IntegrityError::new(IntegrityErrorCode::ProgrammeIdentity));
        }
        if self.attempt != attempt.value()
            || input.identity.attempt != attempt.value()
            || callback_identity.attempt_id() != attempt
        {
            return Err(IntegrityError::new(IntegrityErrorCode::AttemptIdentity));
        }
        if transaction_id.as_str().is_empty()
            || input.identity.transaction_id != transaction_id.as_str()
        {
            return Err(IntegrityError::new(IntegrityErrorCode::TransactionIdentity));
        }
        if source_for_callback(callback_identity.callback_index())
            != Some(input.identity.source_index)
        {
            return Err(IntegrityError::new(IntegrityErrorCode::SourceOrder));
        }

        let before = IntegrityOwnerStateSnapshot::from_state(&self.state);
        let source = self.stage_source(input, provider_json)?;
        let after = IntegrityOwnerStateSnapshot::from_state(source.after_state());
        let changed = before.generation() != after.generation()
            || before.state_sha256() != after.state_sha256();
        let owner_receipt = IntegrityOwnerReceiptProposal {
            programme_id: programme_id.clone(),
            attempt,
            transaction_id: transaction_id.clone(),
            callback_identity: callback_identity.clone(),
            before,
            after,
            changed,
        };
        Ok(IntegrityGtcSourceProposal {
            source,
            owner_receipt,
        })
    }

    /// Stages source/dispatch/AU receipts from F14K evidence and an already completed F12 frame.
    ///
    /// Unlike `stage_source_for_callback`, this adapter does not patch a carrier or write CRC16.
    /// F12 owns the complete frame; this owner validates its dynamic carrier cursor and CRC
    /// residual, then proposes the next Integrity root with typed receipts for F15.
    pub fn stage_f12_source_for_callback(
        &self,
        input: IntegrityF12SourceInput<'_>,
        provider: &KeylessIntegrityProvider,
        programme_id: &ProgrammeId,
        attempt: AttemptId,
        transaction_id: &CallbackTransactionId,
        callback_identity: &CallbackLineageIdentity,
    ) -> Result<IntegrityGtcF12SourceProposal, IntegrityError> {
        if self.retired {
            return Err(IntegrityError::new(IntegrityErrorCode::AttemptRetired));
        }
        if self.state.finalized {
            return Err(IntegrityError::new(IntegrityErrorCode::AlreadyFinalized));
        }
        if provider.profile_id() != PROFILE_ID
            || provider.provider_sha256() != self.state.provider_sha256
        {
            return Err(IntegrityError::new(IntegrityErrorCode::ProviderValues));
        }
        validate_identity(&input.identity, self)?;
        if input.identity.programme_id != programme_id.as_str()
            || callback_identity.programme_id() != programme_id
        {
            return Err(IntegrityError::new(IntegrityErrorCode::ProgrammeIdentity));
        }
        if input.identity.attempt != attempt.value() || callback_identity.attempt_id() != attempt {
            return Err(IntegrityError::new(IntegrityErrorCode::AttemptIdentity));
        }
        if transaction_id.as_str().is_empty()
            || input.identity.transaction_id != transaction_id.as_str()
        {
            return Err(IntegrityError::new(IntegrityErrorCode::TransactionIdentity));
        }
        if source_for_callback(callback_identity.callback_index())
            != Some(input.identity.source_index)
        {
            return Err(IntegrityError::new(IntegrityErrorCode::SourceOrder));
        }

        let source_payload_sha256;
        let payload_evidence = match input.payload {
            F14kPayloadInput::AbsentPublishedPayloadV1 => {
                if input.identity.source_index.value() >= SUPPRESSED_SOURCE_COUNT
                    || input.identity.au_index.is_some()
                    || input.completed_frame.emdf_container_start.is_some()
                {
                    return Err(IntegrityError::new(IntegrityErrorCode::AuOrder));
                }
                source_payload_sha256 = None;
                IntegrityPayloadEvidence::AbsentPublishedPayloadV1
            }
            F14kPayloadInput::ProtectedEmdf {
                zero_slot_container,
                protected_container,
            } => {
                if input.identity.source_index.value() < SUPPRESSED_SOURCE_COUNT
                    || input.identity.au_index.is_none()
                {
                    return Err(IntegrityError::new(IntegrityErrorCode::AuOrder));
                }
                let protected_bytes = protected_container.bytes();
                let primary_crc32 = crc32_iso_hdlc(
                    zero_slot_container
                        .get(4..)
                        .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::ChecksumSlots))?,
                );
                let secondary_crc8 = crc8_smbus(&primary_crc32.to_be_bytes());
                if protected_bytes.len() != zero_slot_container.len()
                    || protected_container.input_sha256() != sha256(zero_slot_container)
                    || protected_container.output_sha256() != sha256(protected_bytes)
                    || protected_container.primary_crc32() != primary_crc32
                    || protected_container.secondary_crc8() != secondary_crc8
                {
                    return Err(IntegrityError::new(IntegrityErrorCode::ChecksumSlots));
                }
                let cursor = input
                    .completed_frame
                    .emdf_container_start
                    .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::FrameShape))?;
                if !frame_contains_payload_at_cursor(
                    input.completed_frame.frame,
                    cursor,
                    protected_bytes,
                ) {
                    return Err(IntegrityError::new(IntegrityErrorCode::FrameShape));
                }
                source_payload_sha256 = Some(protected_container.output_sha256());
                IntegrityPayloadEvidence::ProtectedEmdf {
                    zero_slot_sha256: protected_container.input_sha256(),
                    protected_sha256: protected_container.output_sha256(),
                    message_sha256: sha256(&zero_slot_container[4..]),
                    primary_crc32,
                    secondary_crc8,
                    primary_range: protected_container.primary_range(),
                    secondary_range: protected_container.secondary_range(),
                    container_bytes: protected_bytes.len(),
                }
            }
        };

        let frame = input.completed_frame.frame;
        let (_crc_start, crc16) = validate_completed_f12_frame(frame, self.codec_rate)?;
        let frame_sha256 = sha256(frame);
        let before = IntegrityOwnerStateSnapshot::from_state(&self.state);
        let generation = self
            .state
            .generation
            .checked_add(1)
            .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::GenerationOverflow))?;
        let next_source = input
            .identity
            .source_index
            .value()
            .checked_add(1)
            .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::SourceOrder))?;
        let published_au_count = if input.identity.au_index.is_some() {
            self.state
                .published_au_count
                .checked_add(1)
                .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::GenerationOverflow))?
        } else {
            self.state.published_au_count
        };
        let after_state = IntegrityState {
            generation,
            next_source_index: CodecSourceIndex::new(next_source),
            published_au_count,
            ..self.state.clone()
        };
        let after_owner = self.with_state(after_state);
        let after = after_owner.owner_state_snapshot();
        let before_sha256 = before.state_sha256();
        let proposed_after_sha256 = after.state_sha256();
        let identity = input.identity;
        let receipts = IntegrityF12SourceReceipts {
            source: IntegrityF12SourceReceipt {
                identity: identity.clone(),
                before_generation: before.generation(),
                proposed_after_generation: after.generation(),
                before_sha256,
                proposed_after_sha256,
                provider_sha256: provider.provider_sha256(),
                payload: payload_evidence,
            },
            dispatch: IntegrityF12DispatchReceipt {
                identity: identity.clone(),
                frame_sha256,
                frame_bytes: frame.len(),
                emdf_container_start: input.completed_frame.emdf_container_start,
                source_payload_sha256,
            },
            au: IntegrityF12AuReceipt {
                identity: identity.clone(),
                frame_sha256,
                crc16,
                crc_residual: 0,
                published: identity.au_index.is_some(),
            },
        };
        let changed = before.generation() != after.generation()
            || before.state_sha256() != after.state_sha256();
        let owner_receipt = IntegrityOwnerReceiptProposal {
            programme_id: programme_id.clone(),
            attempt,
            transaction_id: transaction_id.clone(),
            callback_identity: callback_identity.clone(),
            before,
            after,
            changed,
        };
        Ok(IntegrityGtcF12SourceProposal {
            source: IntegrityF12SourceProposal {
                after_owner,
                receipts,
            },
            owner_receipt,
        })
    }

    /// Stages I0 and D0 frame receipts with role-specific rates and one shared AU-level transition.
    ///
    /// I0 carries no F13 payload; D0 carries the one protected block-two EMDF container. Both
    /// frame validations are proposals from the same committed owner root. The returned owner
    /// after-state advances once for the pair and remains uninstalled until the parent admits the
    /// durable F15 append receipt.
    pub fn stage_flat7x_i0_d0_f12_pair_for_callback(
        &self,
        input: Flat7xI0D0F12PairInput<'_>,
        provider: &KeylessIntegrityProvider,
        programme_id: &ProgrammeId,
        attempt: AttemptId,
        transaction_id: &CallbackTransactionId,
        callback_identity: &CallbackLineageIdentity,
    ) -> Result<IntegrityGtcFlat7xF12PairProposal, IntegrityError> {
        if self.retired {
            return Err(IntegrityError::new(IntegrityErrorCode::AttemptRetired));
        }
        if self.state.finalized {
            return Err(IntegrityError::new(IntegrityErrorCode::AlreadyFinalized));
        }
        if provider.profile_id() != PROFILE_ID
            || provider.provider_sha256() != self.state.provider_sha256
        {
            return Err(IntegrityError::new(IntegrityErrorCode::ProviderValues));
        }
        let target = crate::profile::Flat7xI0D0TargetPlan::approved_contract();
        let rate_plan = input.rate_plan;
        if rate_plan.independent_i0_rate() != target.i0().rate()
            || rate_plan.dependent_d0_rate() != target.d0().rate()
            || self.codec_rate != rate_plan.dependent_d0_rate()
        {
            return Err(IntegrityError::new(IntegrityErrorCode::UnsupportedProfile));
        }

        validate_identity(&input.identity, self)?;
        if input.identity.programme_id != programme_id.as_str()
            || callback_identity.programme_id() != programme_id
        {
            return Err(IntegrityError::new(IntegrityErrorCode::ProgrammeIdentity));
        }
        if input.identity.attempt != attempt.value() || callback_identity.attempt_id() != attempt {
            return Err(IntegrityError::new(IntegrityErrorCode::AttemptIdentity));
        }
        if transaction_id.as_str().is_empty()
            || input.identity.transaction_id != transaction_id.as_str()
        {
            return Err(IntegrityError::new(IntegrityErrorCode::TransactionIdentity));
        }
        if source_for_callback(callback_identity.callback_index())
            != Some(input.identity.source_index)
        {
            return Err(IntegrityError::new(IntegrityErrorCode::SourceOrder));
        }
        let expected_dispatch = input
            .identity
            .source_index
            .dispatch()
            .map_err(|_| IntegrityError::new(IntegrityErrorCode::DispatchOrder))?;
        let expected_au = input
            .identity
            .source_index
            .value()
            .checked_sub(SUPPRESSED_SOURCE_COUNT)
            .map(PublishedAuIndex::new);
        if input.identity.source_index.value() < SUPPRESSED_SOURCE_COUNT
            || input.identity.au_index.is_none()
            || input.identity.dispatch != expected_dispatch
            || input.identity.au_index != expected_au
        {
            return Err(IntegrityError::new(IntegrityErrorCode::AuOrder));
        }

        let (_, i0_crc16) =
            validate_completed_f12_frame(input.i0_frame, rate_plan.independent_i0_rate())?;
        let (_, d0_crc16) =
            validate_completed_f12_frame(input.d0_frame, rate_plan.dependent_d0_rate())?;

        let zero_slot_container = input.d0_zero_slot_container;
        let protected_container = input.d0_protected_container;
        if zero_slot_container.is_empty()
            || zero_slot_container.len() > 511
            || protected_container.bytes().len() != zero_slot_container.len()
        {
            return Err(IntegrityError::new(IntegrityErrorCode::ChecksumSlots));
        }
        let primary_crc32 = crc32_iso_hdlc(
            zero_slot_container
                .get(4..)
                .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::ChecksumSlots))?,
        );
        let secondary_crc8 = crc8_smbus(&primary_crc32.to_be_bytes());
        if protected_container.input_sha256() != sha256(zero_slot_container)
            || protected_container.output_sha256() != sha256(protected_container.bytes())
            || protected_container.primary_crc32() != primary_crc32
            || protected_container.secondary_crc8() != secondary_crc8
            || !frame_contains_payload_at_cursor(
                input.d0_frame,
                input.d0_emdf_container_start,
                protected_container.bytes(),
            )
        {
            return Err(IntegrityError::new(IntegrityErrorCode::ChecksumSlots));
        }

        let before = IntegrityOwnerStateSnapshot::from_state(&self.state);
        let after_generation = self
            .state
            .generation
            .checked_add(1)
            .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::GenerationOverflow))?;
        let next_source_index = input
            .identity
            .source_index
            .value()
            .checked_add(1)
            .map(CodecSourceIndex::new)
            .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::SourceOrder))?;
        let published_au_count = self
            .state
            .published_au_count
            .checked_add(1)
            .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::GenerationOverflow))?;
        let after_owner = self.with_state(IntegrityState {
            generation: after_generation,
            next_source_index,
            published_au_count,
            ..self.state.clone()
        });
        let after = after_owner.owner_state_snapshot();
        let before_sha256 = before.state_sha256();
        let after_sha256 = after.state_sha256();
        let provider_sha256 = provider.provider_sha256();
        let identity = input.identity;
        let i0_sha256 = sha256(input.i0_frame);
        let d0_sha256 = sha256(input.d0_frame);
        let protected_evidence = IntegrityPayloadEvidence::ProtectedEmdf {
            zero_slot_sha256: protected_container.input_sha256(),
            protected_sha256: protected_container.output_sha256(),
            message_sha256: sha256(&zero_slot_container[4..]),
            primary_crc32,
            secondary_crc8,
            primary_range: protected_container.primary_range(),
            secondary_range: protected_container.secondary_range(),
            container_bytes: protected_container.bytes().len(),
        };
        let receipts = Flat7xI0D0F12PairReceipts {
            i0: IntegrityF12SourceReceipts {
                source: IntegrityF12SourceReceipt {
                    identity: identity.clone(),
                    before_generation: before.generation(),
                    proposed_after_generation: after.generation(),
                    before_sha256,
                    proposed_after_sha256: after_sha256,
                    provider_sha256,
                    payload: IntegrityPayloadEvidence::AbsentPublishedPayloadV1,
                },
                dispatch: IntegrityF12DispatchReceipt {
                    identity: identity.clone(),
                    frame_sha256: i0_sha256,
                    frame_bytes: input.i0_frame.len(),
                    emdf_container_start: None,
                    source_payload_sha256: None,
                },
                au: IntegrityF12AuReceipt {
                    identity: identity.clone(),
                    frame_sha256: i0_sha256,
                    crc16: i0_crc16,
                    crc_residual: 0,
                    published: true,
                },
            },
            d0: IntegrityF12SourceReceipts {
                source: IntegrityF12SourceReceipt {
                    identity: identity.clone(),
                    before_generation: before.generation(),
                    proposed_after_generation: after.generation(),
                    before_sha256,
                    proposed_after_sha256: after_sha256,
                    provider_sha256,
                    payload: protected_evidence,
                },
                dispatch: IntegrityF12DispatchReceipt {
                    identity: identity.clone(),
                    frame_sha256: d0_sha256,
                    frame_bytes: input.d0_frame.len(),
                    emdf_container_start: Some(input.d0_emdf_container_start),
                    source_payload_sha256: Some(protected_container.output_sha256()),
                },
                au: IntegrityF12AuReceipt {
                    identity: identity.clone(),
                    frame_sha256: d0_sha256,
                    crc16: d0_crc16,
                    crc_residual: 0,
                    published: true,
                },
            },
        };
        let owner_receipt = IntegrityOwnerReceiptProposal {
            programme_id: programme_id.clone(),
            attempt,
            transaction_id: transaction_id.clone(),
            callback_identity: callback_identity.clone(),
            before,
            after,
            changed: true,
        };
        Ok(IntegrityGtcFlat7xF12PairProposal {
            after_owner,
            receipts,
            owner_receipt,
        })
    }

    fn with_state(&self, state: IntegrityState) -> Self {
        Self {
            state,
            programme_id: self.programme_id.clone(),
            attempt: self.attempt,
            codec_rate: self.codec_rate,
            expected_source_count: self.expected_source_count,
            retired: self.retired,
        }
    }

    /// EOF commits only the finalized flag and generation after the exact source count is reached.
    pub fn finalize(&mut self) -> Result<(), IntegrityError> {
        if self.retired {
            return Err(IntegrityError::new(IntegrityErrorCode::AttemptRetired));
        }
        if self.state.finalized {
            self.retired = true;
            return Err(IntegrityError::new(IntegrityErrorCode::AlreadyFinalized));
        }
        let expected_published = self
            .expected_source_count
            .saturating_sub(SUPPRESSED_SOURCE_COUNT);
        if self.state.next_source_index.value() != self.expected_source_count
            || self.state.published_au_count != expected_published
        {
            self.retired = true;
            return Err(IntegrityError::new(IntegrityErrorCode::IncompleteFinalize));
        }
        let Some(generation) = self.state.generation.checked_add(1) else {
            self.retired = true;
            return Err(IntegrityError::new(IntegrityErrorCode::GenerationOverflow));
        };
        let next_state = IntegrityState {
            generation,
            finalized: true,
            ..self.state.clone()
        };
        self.state = next_state;
        Ok(())
    }

    /// Consumes this attempt and creates a cold owner bound to a distinct attempt identity.
    pub fn reset(self, new_attempt: u64, provider_json: &[u8]) -> Result<Self, IntegrityError> {
        if new_attempt == self.attempt {
            return Err(IntegrityError::new(IntegrityErrorCode::AttemptReuse));
        }
        let provider = validate_keyless_provider(provider_json).map_err(map_provider_error)?;
        Self::cold_from_provider_for_rate(
            &self.programme_id,
            new_attempt,
            self.expected_source_count,
            &provider,
            self.codec_rate,
        )
    }

    pub const fn committed_state(&self) -> &IntegrityState {
        &self.state
    }

    pub const fn codec_rate(&self) -> crate::profile::CodecRateProfile {
        self.codec_rate
    }

    /// Returns the complete current state projection without changing owner state.
    pub fn owner_state_snapshot(&self) -> IntegrityOwnerStateSnapshot {
        IntegrityOwnerStateSnapshot::from_state(&self.state)
    }

    pub const fn is_retired(&self) -> bool {
        self.retired
    }

    #[cfg(test)]
    fn process_with_test_fault(
        &mut self,
        input: IntegrityInput<'_>,
        provider_json: &[u8],
        stage: IntegrityStage,
    ) -> Result<IntegrityCallResult, IntegrityError> {
        self.process_with_stage_observer(input, provider_json, |current| current == stage)
    }

    #[cfg(test)]
    fn process_with_stage_observer<F>(
        &mut self,
        input: IntegrityInput<'_>,
        provider_json: &[u8],
        mut fail_after_stage: F,
    ) -> Result<IntegrityCallResult, IntegrityError>
    where
        F: FnMut(IntegrityStage) -> bool,
    {
        if self.retired {
            return Err(IntegrityError::new(IntegrityErrorCode::AttemptRetired));
        }
        if self.state.finalized {
            self.retired = true;
            return Err(IntegrityError::new(IntegrityErrorCode::AlreadyFinalized));
        }

        let before_sha256 = self.state.sha256();
        let result = self.stage_call(input, provider_json, before_sha256, &mut fail_after_stage);
        match result {
            Ok((output, next_state)) => {
                self.state = next_state;
                Ok(output)
            }
            Err(error) => {
                self.retired = true;
                Err(error)
            }
        }
    }

    fn stage_call<F>(
        &self,
        input: IntegrityInput<'_>,
        provider_json: &[u8],
        before_sha256: IntegritySha256,
        fail_after_stage: &mut F,
    ) -> Result<(IntegrityCallResult, IntegrityState), IntegrityError>
    where
        F: FnMut(IntegrityStage) -> bool,
    {
        let provider = validate_keyless_provider(provider_json).map_err(map_provider_error)?;
        if provider.provider_sha256() != self.state.provider_sha256 {
            return Err(IntegrityError::new(IntegrityErrorCode::ProviderValues));
        }
        validate_identity(&input.identity, self)?;

        let primary_range = checked_range(
            input.primary_slot_offset,
            4,
            input.zero_slot_container.len(),
        )
        .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::ChecksumSlots))?;
        let secondary_range = checked_range(
            input.secondary_slot_offset,
            1,
            input.zero_slot_container.len(),
        )
        .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::ChecksumSlots))?;
        if input.zero_slot_container.len() < 9
            || ranges_overlap(&primary_range, &secondary_range)
            || input.zero_slot_container[primary_range.clone()]
                .iter()
                .chain(input.zero_slot_container[secondary_range.clone()].iter())
                .any(|byte| *byte != 0)
        {
            return Err(IntegrityError::new(IntegrityErrorCode::ChecksumSlots));
        }
        let message = input
            .zero_slot_container
            .get(4..)
            .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::ChecksumSlots))?;
        let message_sha256 = sha256(message);
        let source_input_sha256 = sha256(input.zero_slot_container);
        let primary = crc32_iso_hdlc(message).to_be_bytes();
        let secondary = [crc8_smbus(&primary)];
        let mut patched_container = input.zero_slot_container.to_vec();
        patched_container[primary_range].copy_from_slice(&primary);
        patched_container[secondary_range].copy_from_slice(&secondary);
        let source_output_sha256 = sha256(&patched_container);

        let next_source_index = input
            .identity
            .source_index
            .value()
            .checked_add(1)
            .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::SourceOrder))?;
        let published = input.identity.source_index.value() >= SUPPRESSED_SOURCE_COUNT;
        let published_au_count = if published {
            self.state
                .published_au_count
                .checked_add(1)
                .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::GenerationOverflow))?
        } else {
            self.state.published_au_count
        };
        let generation = self
            .state
            .generation
            .checked_add(1)
            .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::GenerationOverflow))?;
        let next_state = IntegrityState {
            generation,
            next_source_index: CodecSourceIndex::new(next_source_index),
            published_au_count,
            ..self.state.clone()
        };
        let proposed_after_sha256 = next_state.sha256();
        let receipt_common = ReceiptCommon::new(
            input.identity,
            self.state.generation,
            before_sha256,
            generation,
            proposed_after_sha256,
            provider.provider_sha256(),
        );
        let source_receipt = source_receipt(
            &receipt_common,
            source_input_sha256,
            source_output_sha256,
            message_sha256,
            primary,
            secondary,
            patched_container.len(),
        );
        if fail_after_stage(IntegrityStage::Source) {
            return Err(IntegrityError::new(IntegrityErrorCode::SourceFailed));
        }

        let frame_carrier_range = checked_range(
            input.container_offset,
            patched_container.len(),
            input.frame_template.len(),
        )
        .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::FrameShape))?;
        let frame_crc_start = input.frame_template.len().saturating_sub(2);
        if Eac3FrameSize::from_frame_bytes(input.frame_template.len())
            .map(Eac3FrameSize::codec_rate)
            != Some(self.codec_rate)
            || input.container_offset < FRAME_CRC_INPUT_START
            || frame_carrier_range.end > frame_crc_start
            || input.frame_template[frame_carrier_range.clone()]
                .iter()
                .any(|byte| *byte != 0)
        {
            return Err(IntegrityError::new(IntegrityErrorCode::FrameShape));
        }
        let mut frame = input.frame_template.to_vec();
        frame[frame_carrier_range].copy_from_slice(&patched_container);
        let dispatch_output_sha256 = sha256(&frame);
        let dispatch_receipt = dispatch_receipt(
            &receipt_common,
            source_output_sha256,
            dispatch_output_sha256,
            input.container_offset,
            frame.len(),
        );
        if fail_after_stage(IntegrityStage::Dispatch) {
            return Err(IntegrityError::new(IntegrityErrorCode::DispatchFailed));
        }

        let crc16 = eac3_crc16(&frame[FRAME_CRC_INPUT_START..frame_crc_start]);
        let au_input_sha256 = dispatch_output_sha256;
        frame[frame_crc_start..].copy_from_slice(&crc16.to_be_bytes());
        let crc_residual = eac3_crc16(&frame[FRAME_CRC_INPUT_START..]);
        if crc_residual != 0 {
            return Err(IntegrityError::new(IntegrityErrorCode::AuFailed));
        }
        let au_output_sha256 = sha256(&frame);
        let au_receipt = au_receipt(
            &receipt_common,
            au_input_sha256,
            au_output_sha256,
            crc16,
            crc_residual,
            published,
        );
        if fail_after_stage(IntegrityStage::Au) {
            return Err(IntegrityError::new(IntegrityErrorCode::AuFailed));
        }

        let output = IntegrityCallResult {
            receipts: [source_receipt, dispatch_receipt, au_receipt],
            frame,
            before_sha256,
            after_sha256: proposed_after_sha256,
            commit_count: 1,
        };
        Ok((output, next_state))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReceiptCommon {
    identity: IntegrityIdentity,
    before_generation: u64,
    before_sha256: IntegritySha256,
    proposed_after_generation: u64,
    proposed_after_sha256: IntegritySha256,
    provider_sha256: IntegritySha256,
}

impl ReceiptCommon {
    fn new(
        identity: IntegrityIdentity,
        before_generation: u64,
        before_sha256: IntegritySha256,
        proposed_after_generation: u64,
        proposed_after_sha256: IntegritySha256,
        provider_sha256: IntegritySha256,
    ) -> Self {
        Self {
            identity,
            before_generation,
            before_sha256,
            proposed_after_generation,
            proposed_after_sha256,
            provider_sha256,
        }
    }
}

fn source_receipt(
    common: &ReceiptCommon,
    input_sha256: IntegritySha256,
    output_sha256: IntegritySha256,
    message_sha256: IntegritySha256,
    primary: [u8; 4],
    secondary: [u8; 1],
    container_bytes: usize,
) -> IntegrityStageReceipt {
    IntegrityStageReceipt {
        schema: RECEIPT_SCHEMA,
        owner_id: OWNER_ID,
        stage: IntegrityStage::Source,
        identity: common.identity.clone(),
        before_generation: common.before_generation,
        before_sha256: common.before_sha256,
        proposed_after_generation: common.proposed_after_generation,
        proposed_after_sha256: common.proposed_after_sha256,
        provider_id: PROFILE_ID,
        provider_sha256: common.provider_sha256,
        input_sha256,
        output_sha256,
        details: IntegrityStageDetails::Source {
            primary,
            secondary,
            message_sha256,
            container_bytes,
        },
    }
}

fn dispatch_receipt(
    common: &ReceiptCommon,
    input_sha256: IntegritySha256,
    output_sha256: IntegritySha256,
    container_offset: usize,
    frame_bytes: usize,
) -> IntegrityStageReceipt {
    IntegrityStageReceipt {
        schema: RECEIPT_SCHEMA,
        owner_id: OWNER_ID,
        stage: IntegrityStage::Dispatch,
        identity: common.identity.clone(),
        before_generation: common.before_generation,
        before_sha256: common.before_sha256,
        proposed_after_generation: common.proposed_after_generation,
        proposed_after_sha256: common.proposed_after_sha256,
        provider_id: PROFILE_ID,
        provider_sha256: common.provider_sha256,
        input_sha256,
        output_sha256,
        details: IntegrityStageDetails::Dispatch {
            container_offset,
            frame_bytes,
        },
    }
}

fn au_receipt(
    common: &ReceiptCommon,
    input_sha256: IntegritySha256,
    output_sha256: IntegritySha256,
    crc16: u16,
    crc_residual: u16,
    published: bool,
) -> IntegrityStageReceipt {
    IntegrityStageReceipt {
        schema: RECEIPT_SCHEMA,
        owner_id: OWNER_ID,
        stage: IntegrityStage::Au,
        identity: common.identity.clone(),
        before_generation: common.before_generation,
        before_sha256: common.before_sha256,
        proposed_after_generation: common.proposed_after_generation,
        proposed_after_sha256: common.proposed_after_sha256,
        provider_id: PROFILE_ID,
        provider_sha256: common.provider_sha256,
        input_sha256,
        output_sha256,
        details: IntegrityStageDetails::Au {
            crc16,
            crc_residual,
            published,
        },
    }
}

fn validate_identity(
    identity: &IntegrityIdentity,
    owner: &IntegrityProgrammeOwner,
) -> Result<(), IntegrityError> {
    if identity.programme_id != owner.programme_id {
        return Err(IntegrityError::new(IntegrityErrorCode::ProgrammeIdentity));
    }
    if identity.attempt != owner.attempt {
        return Err(IntegrityError::new(IntegrityErrorCode::AttemptIdentity));
    }
    if identity.transaction_id.is_empty() {
        return Err(IntegrityError::new(IntegrityErrorCode::TransactionIdentity));
    }
    let source_index = identity.source_index.value();
    if source_index != owner.state.next_source_index.value()
        || source_index >= owner.expected_source_count
    {
        return Err(IntegrityError::new(IntegrityErrorCode::SourceOrder));
    }
    let expected_dispatch = source_index
        .checked_add(1)
        .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::DispatchOrder))?;
    if identity.dispatch.value() != expected_dispatch {
        return Err(IntegrityError::new(IntegrityErrorCode::DispatchOrder));
    }
    let expected_au = if source_index < SUPPRESSED_SOURCE_COUNT {
        None
    } else {
        Some(PublishedAuIndex::new(
            source_index - SUPPRESSED_SOURCE_COUNT,
        ))
    };
    if identity.au_index != expected_au {
        return Err(IntegrityError::new(IntegrityErrorCode::AuOrder));
    }
    Ok(())
}

fn checked_range(offset: usize, width: usize, bound: usize) -> Option<std::ops::Range<usize>> {
    let end = offset.checked_add(width)?;
    (end <= bound).then_some(offset..end)
}

fn validate_completed_f12_frame(
    frame: &[u8],
    expected_rate: crate::profile::CodecRateProfile,
) -> Result<(usize, u16), IntegrityError> {
    let frame_size = Eac3FrameSize::from_frame_bytes(frame.len())
        .filter(|size| size.codec_rate() == expected_rate)
        .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::FrameShape))?;
    let crc_start = frame_size.bytes() - 2;
    let crc_residual = eac3_crc16(
        frame
            .get(FRAME_CRC_INPUT_START..)
            .ok_or_else(|| IntegrityError::new(IntegrityErrorCode::FrameShape))?,
    );
    if crc_residual != 0 {
        return Err(IntegrityError::new(IntegrityErrorCode::AuFailed));
    }
    let crc16 = u16::from_be_bytes([frame[crc_start], frame[crc_start + 1]]);
    Ok((crc_start, crc16))
}

fn frame_contains_payload_at_cursor(frame: &[u8], cursor: FrameBitCursor, payload: &[u8]) -> bool {
    let Some(payload_bits) = u64::try_from(payload.len())
        .ok()
        .and_then(|length| length.checked_mul(8))
    else {
        return false;
    };
    let start_bit = u64::from(cursor.bit_offset());
    let Some(end_bit) = start_bit.checked_add(payload_bits) else {
        return false;
    };
    let range_start = (FRAME_CRC_INPUT_START as u64) * 8;
    let Some(frame_size) = Eac3FrameSize::from_frame_bytes(frame.len()) else {
        return false;
    };
    let range_end = u64::try_from(frame_size.bytes().saturating_sub(2)).unwrap_or(0) * 8;
    if start_bit < range_start || end_bit > range_end {
        return false;
    }

    for (byte_index, expected_byte) in payload.iter().copied().enumerate() {
        let Some(byte_start) = u64::try_from(byte_index)
            .ok()
            .and_then(|index| index.checked_mul(8))
            .and_then(|offset| start_bit.checked_add(offset))
        else {
            return false;
        };
        for bit_index in 0..8_u64 {
            let Some(bit_offset) = byte_start.checked_add(bit_index) else {
                return false;
            };
            let Ok(frame_byte_index) = usize::try_from(bit_offset / 8) else {
                return false;
            };
            let frame_shift = 7 - (bit_offset % 8) as u8;
            let payload_shift = 7 - bit_index as u8;
            let Some(frame_byte) = frame.get(frame_byte_index) else {
                return false;
            };
            if (frame_byte >> frame_shift) & 1 != (expected_byte >> payload_shift) & 1 {
                return false;
            }
        }
    }
    true
}

fn ranges_overlap(left: &std::ops::Range<usize>, right: &std::ops::Range<usize>) -> bool {
    left.start < right.end && right.start < left.end
}

fn map_provider_error(error: KeylessProviderError) -> IntegrityError {
    let code = match error {
        KeylessProviderError::UnsupportedProfile => IntegrityErrorCode::UnsupportedProfile,
        KeylessProviderError::ProviderValues => IntegrityErrorCode::ProviderValues,
    };
    IntegrityError::new(code)
}

#[cfg(test)]
mod tests;
