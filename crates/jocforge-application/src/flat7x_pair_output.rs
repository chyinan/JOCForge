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
        Flat7xAuAttemptRetirement, Flat7xAuAttemptRetirementReason, Flat7xAuOutputOwner,
        Flat7xAuOutputOwnerError, Flat7xAuSourceReceiptInput,
    },
};

use crate::flat7x_pair_source::Flat7xPairAccessUnitCandidate;

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
