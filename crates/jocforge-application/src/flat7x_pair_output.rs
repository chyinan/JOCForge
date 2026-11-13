// pattern: Functional Core
//! Application gate from a completed in-memory pair candidate to Flat7x output staging.

use jocforge_domain::{
    analysis::source_frame::SharedAnalysisIdentity,
    profile::{
        D0EmdfSkipFieldExtent, D0FrameExtent, Flat7xAuByteSequenceError, Flat7xAuCandidate,
        Flat7xAuPlanValidationError, Flat7xI0D0TargetPlan, Flat7xSourceGeneration, I0FrameExtent,
        compose_flat7x_au_byte_sequence, validate_flat7x_au_candidate,
    },
    publication::{
        Flat7xAuAttemptRetirementReason, Flat7xAuOutputOwner, Flat7xAuOutputOwnerError,
        Flat7xAuSourceReceiptInput,
    },
};

pub use jocforge_domain::publication::{
    Flat7xAuAppendProposal, Flat7xAuAttemptRetirement, Flat7xAuOutputAfterProposal,
};

use crate::flat7x_pair_source::Flat7xPairAccessUnitCandidate;

#[cfg(feature = "test-support")]
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Flat7xPairIoStageError {
    SourceIdentityMismatch,
    SourceRoleMapMismatch,
    UnsupportedTarget,
    I0FrameExtentMismatch,
    D0FrameExtentMismatch,
    AccessUnitExtentMismatch,
    AccessUnitSpanOrderMismatch,
    D0BlockIndexMismatch,
    D0SkipFieldMismatch,
    ProductionAuthorizationUnexpected,
    ByteSequenceValidation(Flat7xAuPlanValidationError),
    ByteSequenceComposition(Flat7xAuByteSequenceError),
    OutputOwnerRetirement(Flat7xAuAttemptRetirement),
    AppendProposalUnexpected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xPairIoStageDisposition {
    RejectedProductionDisabled,
}

/// Receipt for the in-memory application/I/O boundary while publication remains disabled.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xPairIoStageReceipt {
    source_identity: SharedAnalysisIdentity,
    i0_frame_bytes: u16,
    d0_frame_bytes: u16,
    access_unit_bytes: u32,
    d0_skipfield_block: u8,
    d0_skipfield_bytes: u16,
    retirement: Flat7xAuAttemptRetirement,
    disposition: Flat7xPairIoStageDisposition,
}

impl Flat7xPairIoStageReceipt {
    pub fn source_identity(&self) -> &SharedAnalysisIdentity {
        &self.source_identity
    }

    pub const fn i0_frame_bytes(&self) -> u16 {
        self.i0_frame_bytes
    }

    pub const fn d0_frame_bytes(&self) -> u16 {
        self.d0_frame_bytes
    }

    pub const fn access_unit_bytes(&self) -> u32 {
        self.access_unit_bytes
    }

    pub const fn d0_skipfield_block(&self) -> u8 {
        self.d0_skipfield_block
    }

    pub const fn d0_skipfield_bytes(&self) -> u16 {
        self.d0_skipfield_bytes
    }

    pub const fn retirement(&self) -> &Flat7xAuAttemptRetirement {
        &self.retirement
    }

    pub const fn disposition(&self) -> Flat7xPairIoStageDisposition {
        self.disposition
    }

    pub const fn append_proposal_created(&self) -> bool {
        false
    }

    pub const fn io_attempted(&self) -> bool {
        false
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

/// Validates one completed pair candidate against the current F15 owner and returns its disabled
/// staging receipt. The owner consumes the in-memory sequence and retires it when the required
/// F13/I0/D0 receipts are absent; no append proposal reaches an I/O adapter.
pub fn preflight_flat7x_pair_io_stage(
    candidate: &Flat7xPairAccessUnitCandidate,
    expected_source_identity: &SharedAnalysisIdentity,
    output_owner: Flat7xAuOutputOwner,
) -> Result<Flat7xPairIoStageReceipt, Flat7xPairIoStageError> {
    if candidate.source_identity() != expected_source_identity {
        return Err(Flat7xPairIoStageError::SourceIdentityMismatch);
    }

    let role_map = candidate.source_role_map_receipt();
    let shared_identity = role_map.shared_analysis_identity();
    let source_stage = shared_identity.source_stage_key();
    let d0_identity = candidate.d0_source_identity();
    if shared_identity != expected_source_identity
        || d0_identity.programme_id() != role_map.programme_id()
        || d0_identity.attempt_id() != source_stage.attempt_id()
        || d0_identity.callback_index() != role_map.callback_index()
        || d0_identity.source_transaction_id() != source_stage.transaction_id()
        || d0_identity.source_index() != role_map.source_index()
        || d0_identity.analysis_generation().value()
            != shared_identity.analysis_after_generation().value()
    {
        return Err(Flat7xPairIoStageError::SourceRoleMapMismatch);
    }

    let target = Flat7xI0D0TargetPlan::approved_contract();
    let access_unit = candidate.access_unit();
    if candidate.target() != target || role_map.target() != target || access_unit.target() != target
    {
        return Err(Flat7xPairIoStageError::UnsupportedTarget);
    }

    let i0_frame_bytes = target.i0().frame_bytes();
    let d0_frame_bytes = target.d0().frame_bytes();
    if access_unit.i0_frame().frame_bytes().len() != usize::from(i0_frame_bytes) {
        return Err(Flat7xPairIoStageError::I0FrameExtentMismatch);
    }
    if access_unit.d0_frame_bytes().len() != usize::from(d0_frame_bytes)
        || access_unit.d0_frame_receipt().target_frame_bytes() != d0_frame_bytes
    {
        return Err(Flat7xPairIoStageError::D0FrameExtentMismatch);
    }

    let expected_au_bytes = target.aggregate_frame_bytes();
    let composition = access_unit.access_unit_receipt();
    if u32::try_from(candidate.access_unit_bytes().len()).ok() != Some(expected_au_bytes)
        || composition.total_byte_length() != expected_au_bytes
    {
        return Err(Flat7xPairIoStageError::AccessUnitExtentMismatch);
    }
    if composition.i0_span().start_byte() != 0
        || composition.i0_span().byte_length() != i0_frame_bytes
        || composition.d0_span().start_byte() != i0_frame_bytes
        || composition.d0_span().byte_length() != d0_frame_bytes
    {
        return Err(Flat7xPairIoStageError::AccessUnitSpanOrderMismatch);
    }

    let skip_target = target.d0_emdf_skip_field();
    let d0_layout = access_unit.d0_frame_receipt().layout();
    let d0_skipfield_bytes = d0_layout.block_two_skip_field().payload_bytes();
    if d0_layout.audio_blocks()[2].block_index().zero_based_value()
        != skip_target.zero_based_audio_block()
        || skip_target.zero_based_audio_block() != 2
    {
        return Err(Flat7xPairIoStageError::D0BlockIndexMismatch);
    }
    if skip_target.field_count() != 1
        || d0_skipfield_bytes == 0
        || d0_skipfield_bytes > skip_target.maximum_complete_container_bytes()
        || skip_target.fragmentation_allowed()
    {
        return Err(Flat7xPairIoStageError::D0SkipFieldMismatch);
    }

    if candidate.production_allowed() || output_owner.production_allowed() {
        return Err(Flat7xPairIoStageError::ProductionAuthorizationUnexpected);
    }

    let generation =
        Flat7xSourceGeneration::new(expected_source_identity.analysis_after_generation().value());
    let candidate_metadata = Flat7xAuCandidate::new(
        Some(I0FrameExtent::new(generation, i0_frame_bytes)),
        Some(D0FrameExtent::new(generation, d0_frame_bytes)),
        Some(D0EmdfSkipFieldExtent::new(
            generation,
            skip_target.zero_based_audio_block(),
            skip_target.field_count(),
            Some(d0_skipfield_bytes),
            skip_target.fragmentation_allowed(),
        )),
    );
    let validation = validate_flat7x_au_candidate(target, candidate_metadata)
        .map_err(Flat7xPairIoStageError::ByteSequenceValidation)?;
    let sequence = compose_flat7x_au_byte_sequence(
        &validation,
        access_unit.i0_frame().frame_bytes(),
        access_unit.d0_frame_bytes(),
    )
    .map_err(Flat7xPairIoStageError::ByteSequenceComposition)?;

    let retirement = match output_owner
        .stage_next_access_unit(sequence, Flat7xAuSourceReceiptInput::new(None, None, None))
    {
        Err(retirement) => retirement,
        Ok(_proposal) => return Err(Flat7xPairIoStageError::AppendProposalUnexpected),
    };
    if retirement.reason()
        != Flat7xAuAttemptRetirementReason::StageRejected(
            Flat7xAuOutputOwnerError::F13MetadataReceiptMissing,
        )
        || !retirement.committed_owner_unchanged()
    {
        return Err(Flat7xPairIoStageError::OutputOwnerRetirement(retirement));
    }

    Ok(Flat7xPairIoStageReceipt {
        source_identity: expected_source_identity.clone(),
        i0_frame_bytes,
        d0_frame_bytes,
        access_unit_bytes: expected_au_bytes,
        d0_skipfield_block: skip_target.zero_based_audio_block(),
        d0_skipfield_bytes,
        retirement,
        disposition: Flat7xPairIoStageDisposition::RejectedProductionDisabled,
    })
}

/// Expected F15 output identity and extents supplied by the output-stage caller.
///
/// Source generation and owner generation remain separate typed values; neither is derived from
/// the other or from an AU index.
#[cfg(feature = "test-support")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xPairIoBinding {
    attempt_id: String,
    output_owner_generation: u64,
    source_generation: jocforge_domain::profile::Flat7xSourceGeneration,
    f13_owner_generations: (u64, u64),
    i0_integrity_generations: (u64, u64),
    d0_integrity_generations: (u64, u64),
    source_index: jocforge_domain::timeline::CodecSourceIndex,
    dispatch_index: jocforge_domain::timeline::JocDispatchIndex,
    published_au_index: jocforge_domain::timeline::PublishedAuIndex,
    byte_offset: u64,
    i0_frame_bytes: u16,
    d0_frame_bytes: u16,
}

#[cfg(feature = "test-support")]
impl Flat7xPairIoBinding {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        attempt_id: String,
        output_owner_generation: u64,
        source_generation: jocforge_domain::profile::Flat7xSourceGeneration,
        f13_owner_generations: (u64, u64),
        i0_integrity_generations: (u64, u64),
        d0_integrity_generations: (u64, u64),
        source_index: jocforge_domain::timeline::CodecSourceIndex,
        dispatch_index: jocforge_domain::timeline::JocDispatchIndex,
        published_au_index: jocforge_domain::timeline::PublishedAuIndex,
        byte_offset: u64,
        i0_frame_bytes: u16,
        d0_frame_bytes: u16,
    ) -> Self {
        Self {
            attempt_id,
            output_owner_generation,
            source_generation,
            f13_owner_generations,
            i0_integrity_generations,
            d0_integrity_generations,
            source_index,
            dispatch_index,
            published_au_index,
            byte_offset,
            i0_frame_bytes,
            d0_frame_bytes,
        }
    }

    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub const fn output_owner_generation(&self) -> u64 {
        self.output_owner_generation
    }

    pub const fn source_generation(&self) -> jocforge_domain::profile::Flat7xSourceGeneration {
        self.source_generation
    }

    pub const fn f13_owner_generations(&self) -> (u64, u64) {
        self.f13_owner_generations
    }

    pub const fn i0_integrity_generations(&self) -> (u64, u64) {
        self.i0_integrity_generations
    }

    pub const fn d0_integrity_generations(&self) -> (u64, u64) {
        self.d0_integrity_generations
    }

    pub const fn source_index(&self) -> jocforge_domain::timeline::CodecSourceIndex {
        self.source_index
    }

    pub const fn dispatch_index(&self) -> jocforge_domain::timeline::JocDispatchIndex {
        self.dispatch_index
    }

    pub const fn published_au_index(&self) -> jocforge_domain::timeline::PublishedAuIndex {
        self.published_au_index
    }

    pub const fn byte_offset(&self) -> u64 {
        self.byte_offset
    }

    pub const fn frame_lengths(&self) -> (u16, u16) {
        (self.i0_frame_bytes, self.d0_frame_bytes)
    }
}

#[cfg(feature = "test-support")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xPairIoHandoffError {
    AttemptMismatch,
    OutputOwnerGenerationMismatch,
    SourceGenerationMismatch,
    F13GenerationMismatch,
    IntegrityGenerationMismatch,
    SourceIdentityMismatch,
    OutputOffsetMismatch,
    FrameExtentMismatch,
    AccessUnitExtentMismatch,
    FrameIntegrityMismatch,
    AccessUnitHashMismatch,
    ProductionAuthorizationUnexpected,
}

/// One-shot application handoff to the existing I/O adapter for a fully F15-staged pair.
///
/// The handoff preserves the F15 receipt-bearing proposal. It carries no production or final-file
/// authority; the current opt-in path is available only through the test-support feature.
#[cfg(feature = "test-support")]
#[must_use]
pub struct Flat7xPairIoHandoff<'a> {
    binding: Flat7xPairIoBinding,
    proposal: jocforge_domain::publication::Flat7xAuAppendProposal<'a>,
}

#[cfg(feature = "test-support")]
impl<'a> Flat7xPairIoHandoff<'a> {
    pub const fn binding(&self) -> &Flat7xPairIoBinding {
        &self.binding
    }

    pub fn bytes(&self) -> &[u8] {
        self.proposal.bytes()
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }

    pub fn into_append_proposal(self) -> Flat7xAuAppendProposal<'a> {
        self.proposal
    }
}

/// Validates a fully source-receipted F15 pair proposal before passing it to the I/O adapter.
///
/// The expectation keeps output-owner generation and Flat7x source-generation labels distinct.
/// I0/D0/AU extents, ordering, source identity and receipt-derived digests are checked before the
/// one-shot proposal is returned. The resulting handoff remains production-disabled.
#[cfg(feature = "test-support")]
pub fn handoff_source_receipted_pair_for_io<'a>(
    proposal: jocforge_domain::publication::Flat7xAuAppendProposal<'a>,
    binding: Flat7xPairIoBinding,
) -> Result<Flat7xPairIoHandoff<'a>, Flat7xPairIoHandoffError> {
    if proposal.attempt_id() != binding.attempt_id() {
        return Err(Flat7xPairIoHandoffError::AttemptMismatch);
    }
    if proposal.before_generation() != binding.output_owner_generation() {
        return Err(Flat7xPairIoHandoffError::OutputOwnerGenerationMismatch);
    }
    if proposal.production_allowed() {
        return Err(Flat7xPairIoHandoffError::ProductionAuthorizationUnexpected);
    }

    let ledger = proposal.ledger_entry();
    if ledger.source_generation() != binding.source_generation() {
        return Err(Flat7xPairIoHandoffError::SourceGenerationMismatch);
    }
    if ledger.f13_emdf_receipt().0 != binding.f13_owner_generations() {
        return Err(Flat7xPairIoHandoffError::F13GenerationMismatch);
    }
    if ledger.integrity_owner_generations()
        != (
            binding.i0_integrity_generations(),
            binding.d0_integrity_generations(),
        )
    {
        return Err(Flat7xPairIoHandoffError::IntegrityGenerationMismatch);
    }
    let metadata_identity = ledger.metadata_identity();
    if ledger.au_index() != binding.published_au_index()
        || metadata_identity
            != (
                binding.source_index(),
                binding.dispatch_index(),
                binding.published_au_index(),
            )
    {
        return Err(Flat7xPairIoHandoffError::SourceIdentityMismatch);
    }
    let expected_offset = u64::from(binding.published_au_index().value())
        .checked_mul(u64::from(
            Flat7xI0D0TargetPlan::approved_contract().aggregate_frame_bytes(),
        ))
        .ok_or(Flat7xPairIoHandoffError::OutputOffsetMismatch)?;
    if ledger.offset() != binding.byte_offset() || binding.byte_offset() != expected_offset {
        return Err(Flat7xPairIoHandoffError::OutputOffsetMismatch);
    }

    let target = Flat7xI0D0TargetPlan::approved_contract();
    let expected_frames = (target.i0().frame_bytes(), target.d0().frame_bytes());
    let frame_lengths = ledger.frame_lengths();
    let expected_au_bytes = target.aggregate_frame_bytes();
    let bytes = proposal.bytes();
    if binding.frame_lengths() != expected_frames || frame_lengths != expected_frames {
        return Err(Flat7xPairIoHandoffError::FrameExtentMismatch);
    }
    if ledger.length() != expected_au_bytes
        || u32::try_from(bytes.len()).ok() != Some(expected_au_bytes)
        || usize::from(frame_lengths.0) + usize::from(frame_lengths.1) != bytes.len()
    {
        return Err(Flat7xPairIoHandoffError::AccessUnitExtentMismatch);
    }

    let i0_end = usize::from(frame_lengths.0);
    let i0_bytes = &bytes[..i0_end];
    let d0_bytes = &bytes[i0_end..];
    let (i0_crc_residual, d0_crc_residual) = ledger.frame_crc_residuals();
    let (i0_digest, d0_digest) = ledger.frame_sha256();
    let actual_i0_digest: [u8; 32] = Sha256::digest(i0_bytes).into();
    let actual_d0_digest: [u8; 32] = Sha256::digest(d0_bytes).into();
    if i0_crc_residual != 0
        || d0_crc_residual != 0
        || &actual_i0_digest != i0_digest
        || &actual_d0_digest != d0_digest
    {
        return Err(Flat7xPairIoHandoffError::FrameIntegrityMismatch);
    }
    let actual_au_digest: [u8; 32] = Sha256::digest(bytes).into();
    if &actual_au_digest != ledger.au_sha256() {
        return Err(Flat7xPairIoHandoffError::AccessUnitHashMismatch);
    }

    // The proposal type can only be issued by the F15 owner after all three source receipts pass.
    let _source_receipts = proposal.source_receipts();
    Ok(Flat7xPairIoHandoff { binding, proposal })
}
