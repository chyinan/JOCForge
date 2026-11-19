// pattern: Functional Core
//! Joins the selected Flat7x source projection to the existing I0/D0 audio-frame seams.

use jocforge_domain::{
    codec::{
        ac3_audio_block::{
            I0AudioBlockSeriesCandidate, I0AudioBlockSeriesError, produce_i0_audio_block_series,
        },
        access_unit_composer::{
            Flat7xI0D0AuCandidate, Flat7xI0D0AuCandidateError,
            assemble_flat7x_i0_d0_au_candidate_with_tail_policy,
        },
        bitio::FrameBitCursor,
        d0_frame_assembler::{Flat7xD0FrameBitInput, Flat7xD0FrameTailPolicy},
        d0_frame_layout::{D0AudioBlockIndex, D0FrameBitSpan},
        flat7x_d0_pcm_source::{
            Flat7xD0PcmFrameInput, Flat7xD0PcmSourceError, produce_flat7x_d0_frame_from_pcm_source,
            stage_flat7x_d0_pcm_block_series,
        },
        i0_frame_assembler::{
            Flat7xI0Ac3AudioSeriesFrameCandidate, Flat7xI0Ac3FrameHeaderFields,
            assemble_flat7x_i0_audio_with_header,
        },
        selected_frame_trial::SelectedSourceNumericPlan,
    },
    joc::emdf::protected_container::CurrentEmdfPayloadSequence,
    profile::{
        CodecRateProfile, Flat7xD0PcmRows, Flat7xI0D0TargetPlan, Flat7xI0PcmRows,
        JocReconstructionLayout, ResolvedEncodingProfile,
    },
    timeline::{CodecSourceIndex, FrontendCallbackIndex, PublishedAuIndex},
    transaction::{AttemptId, CallbackTransactionId, ProgrammeId},
    transform::fullband_exponent_pipeline::NumericPlanReceipt,
};

pub use jocforge_domain::codec::{
    d0_frame_assembler::{
        D0Block2F14kSourceReceiptInput, D0Block2SourceReceipt, D0Block2SourceReceiptError,
    },
    owner::{Flat7xD0SourceGeneration, Flat7xD0SourceGenerationReceipt},
};

pub use jocforge_domain::codec::ac3_audio_block::{
    Ac3DeltaBitAllocationInput, Ac3MantissaQuantizerPolicy, I0AudioBlockControlPolicy,
    I0AudioBlockSeriesBudget, I0AudioBlockSeriesError as Flat7xI0AudioBlockSeriesError,
    I0AudioBlockSeriesPolicy, I0BapCandidateQ,
};
pub use jocforge_domain::codec::bap::Flat7xD0BapParameters;
pub use jocforge_domain::codec::flat7x_d0_pcm_source::{
    Flat7xD0OverlapHistory, Flat7xD0PcmBlockSeriesCandidate, Flat7xD0PcmFrameCandidate,
    Flat7xD0PcmFrameSettings, Flat7xD0PcmSourceExtent, Flat7xD0PcmSourceExtentOrigin,
};
pub use jocforge_domain::codec::frame_builder::{Flat7xD0BsiStructure, Flat7xD0Ci16Addbsi};
pub use jocforge_domain::codec::i0_frame_assembler::{
    CallerOwnedAc3Cplinu, CallerOwnedI0CrcrsvSeed, Flat7xI0FrameAssemblyError, I0AuxDataInput,
    OpaqueI0Ac3BitSpan,
};

pub const FLAT7X_I0_AUDIO_BLOCK_POLICY_ID: &str = "JOCFORGE_FLAT7X_I0_AUDIO_POLICY_V1";

use crate::source_frame_bridge::{Flat7xSourcePcmProjection, Flat7xSourceRoleMapReceipt};
use jocforge_domain::analysis::source_frame::SharedAnalysisIdentity;

/// Current D0 source status at the replaceable audio-span producer boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0AudioSpanProducerState {
    /// Current CLI provider slot; a producer supplies a `Flat7xD0AudioSpanReceipt` when available.
    NoApprovedAudioProductionRule,
}

/// Analysis generation included in the D0 producer receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xAnalysisGeneration(u64);

impl Flat7xAnalysisGeneration {
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Callback source and analysis generation that produced one D0 span set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xD0SourceIdentity {
    programme_id: ProgrammeId,
    attempt_id: AttemptId,
    callback_index: FrontendCallbackIndex,
    source_transaction_id: CallbackTransactionId,
    source_index: CodecSourceIndex,
    analysis_generation: Flat7xAnalysisGeneration,
}

impl Flat7xD0SourceIdentity {
    fn from_analysis(
        programme_id: &ProgrammeId,
        callback_index: FrontendCallbackIndex,
        identity: &SharedAnalysisIdentity,
    ) -> Self {
        let source_stage = identity.source_stage_key();
        Self {
            programme_id: programme_id.clone(),
            attempt_id: source_stage.attempt_id(),
            callback_index,
            source_transaction_id: source_stage.transaction_id().clone(),
            source_index: identity.source_index(),
            analysis_generation: Flat7xAnalysisGeneration(
                identity.analysis_after_generation().value(),
            ),
        }
    }

    pub fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub const fn attempt_id(&self) -> AttemptId {
        self.attempt_id
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub fn source_transaction_id(&self) -> &CallbackTransactionId {
        &self.source_transaction_id
    }

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn analysis_generation(&self) -> Flat7xAnalysisGeneration {
        self.analysis_generation
    }

    fn matches_pair_source(
        &self,
        programme_id: &ProgrammeId,
        attempt_id: AttemptId,
        callback_index: FrontendCallbackIndex,
        source_transaction_id: &CallbackTransactionId,
        source_index: CodecSourceIndex,
        analysis_generation: Flat7xAnalysisGeneration,
    ) -> bool {
        &self.programme_id == programme_id
            && self.attempt_id == attempt_id
            && self.callback_index == callback_index
            && &self.source_transaction_id == source_transaction_id
            && self.source_index == source_index
            && self.analysis_generation == analysis_generation
    }
}

/// AU-bound proof of the complete skip-field span in the assembled D0 block two.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct D0Block2PlacementReceipt {
    block_index: D0AudioBlockIndex,
    bit_span: D0FrameBitSpan,
    container_bytes: u16,
    payload_sequence: CurrentEmdfPayloadSequence,
    source_generation_receipt: Flat7xD0SourceGenerationReceipt,
    au_index: PublishedAuIndex,
}

impl D0Block2PlacementReceipt {
    pub const fn block_index(&self) -> D0AudioBlockIndex {
        self.block_index
    }

    pub const fn bit_span(&self) -> D0FrameBitSpan {
        self.bit_span
    }

    pub const fn container_bytes(&self) -> u16 {
        self.container_bytes
    }

    pub const fn payload_sequence(&self) -> CurrentEmdfPayloadSequence {
        self.payload_sequence
    }

    pub const fn codec_source_generation(&self) -> Flat7xD0SourceGeneration {
        self.source_generation_receipt.source_generation()
    }

    pub const fn source_generation_receipt(&self) -> &Flat7xD0SourceGenerationReceipt {
        &self.source_generation_receipt
    }

    pub const fn au_index(&self) -> PublishedAuIndex {
        self.au_index
    }
}

/// Immutable source-bound D0 audio spans issued by a caller-owned D0 producer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xD0AudioSpanReceipt<'a> {
    source_identity: Flat7xD0SourceIdentity,
    audio_blocks: Flat7xD0FrameBitInput<'a>,
    frame_tail_policy: Flat7xD0FrameTailPolicy,
}

impl<'a> Flat7xD0AudioSpanReceipt<'a> {
    /// Binds D0 block spans and a typed frame-tail policy to the admitted source.
    pub fn from_source_bound_producer(
        binding: &Flat7xPairSourceBinding<'_>,
        audio_blocks: Flat7xD0FrameBitInput<'a>,
        frame_tail_policy: Flat7xD0FrameTailPolicy,
    ) -> Self {
        Self {
            source_identity: binding.d0_source_identity.clone(),
            audio_blocks,
            frame_tail_policy,
        }
    }
}

impl Flat7xD0AudioSpanReceipt<'_> {
    pub const fn source_identity(&self) -> &Flat7xD0SourceIdentity {
        &self.source_identity
    }
}

/// Audio spans required by D0 frame assembly, supplied by a future approved producer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Flat7xD0AudioSpanInput<'a> {
    Disabled(Flat7xD0AudioSpanProducerState),
    Provided(Box<Flat7xD0AudioSpanReceipt<'a>>),
}

/// D0 audio-block spans and the selected frame-tail policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0FrameInput<'a> {
    audio_blocks: Flat7xD0FrameBitInput<'a>,
    frame_tail_policy: Flat7xD0FrameTailPolicy,
}

impl<'a> Flat7xD0FrameInput<'a> {
    pub const fn audio_blocks(self) -> Flat7xD0FrameBitInput<'a> {
        self.audio_blocks
    }

    pub const fn frame_tail_policy(self) -> Flat7xD0FrameTailPolicy {
        self.frame_tail_policy
    }
}

/// Errors while binding the profile and current source projection to the I0/D0 frame inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xPairSourceError {
    UnsupportedProfile,
    ProfileTargetMismatch,
    SourceIdentityMismatch,
    SourceExtentMismatch,
    D0AudioSpanProducerDisabled(Flat7xD0AudioSpanProducerState),
    D0AudioSpanSourceIdentityMismatch,
    D0Block2SourceIdentityMismatch,
    D0Block2SourceReceipt(D0Block2SourceReceiptError),
    D0Block2MetadataReceiptMissing,
    D0Block2Id14ProvenanceMissing,
    D0Block2ProducerTransactionMissing,
    D0Block2AccessUnitMissing,
    D0Block2F14kGenerationReceiptMissing,
    D0Block2ProtectedContainerMissing,
    D0Block2PlacementMismatch,
    D0PcmFrame(Flat7xD0PcmSourceError),
    I0AudioBlockSeries(I0AudioBlockSeriesError),
    I0FrameAssembly(Flat7xI0FrameAssemblyError),
    AccessUnitAssembly(Flat7xI0D0AuCandidateError),
}

/// Binds the exact protected F14K container to its F13 transaction, producer generations, and
/// codec-owner-issued source-generation receipt before D0 frame assembly.
pub fn bind_flat7x_d0_block2_source_receipt<'a>(
    metadata_source: &jocforge_domain::joc::emdf::metadata_owner::MetadataSourceProposal,
    f14k_source: &'a crate::integrity_source_bridge::F14kSourceCandidate,
    expected_source_generation_receipt: &'a Flat7xD0SourceGenerationReceipt,
) -> Result<D0Block2SourceReceipt<'a>, Flat7xPairSourceError> {
    let f14k_f13_source_lineage = f14k_source
        .f13_source_lineage()
        .ok_or(Flat7xPairSourceError::D0Block2MetadataReceiptMissing)?;
    let f14k_au_index = f14k_source
        .au_index()
        .ok_or(Flat7xPairSourceError::D0Block2AccessUnitMissing)?;
    let f14k_source_generation_receipt = f14k_source
        .source_generation_receipt()
        .ok_or(Flat7xPairSourceError::D0Block2F14kGenerationReceiptMissing)?;
    let protected_container = f14k_source
        .protected_container()
        .ok_or(Flat7xPairSourceError::D0Block2ProtectedContainerMissing)?;

    D0Block2SourceReceipt::from_f13_f14k_source_receipts(
        metadata_source,
        D0Block2F14kSourceReceiptInput {
            f13_lineage: f14k_f13_source_lineage,
            source_index: f14k_source.source_index(),
            dispatch: f14k_source.dispatch(),
            au_index: f14k_au_index,
            expected_source_generation_receipt,
            f14k_source_generation_receipt,
            protected_container,
        },
    )
    .map_err(Flat7xPairSourceError::D0Block2SourceReceipt)
}

/// Resolves the typed D0 producer boundary before a pair candidate can be created.
pub fn require_flat7x_d0_audio_blocks<'a>(
    input: Flat7xD0AudioSpanInput<'a>,
    expected_source_identity: &Flat7xD0SourceIdentity,
) -> Result<Flat7xD0FrameBitInput<'a>, Flat7xPairSourceError> {
    require_flat7x_d0_frame_input(input, expected_source_identity)
        .map(Flat7xD0FrameInput::audio_blocks)
}

/// Resolves the full source-bound D0 frame input and its tail policy.
pub fn require_flat7x_d0_frame_input<'a>(
    input: Flat7xD0AudioSpanInput<'a>,
    expected_source_identity: &Flat7xD0SourceIdentity,
) -> Result<Flat7xD0FrameInput<'a>, Flat7xPairSourceError> {
    match input {
        Flat7xD0AudioSpanInput::Disabled(reason) => {
            Err(Flat7xPairSourceError::D0AudioSpanProducerDisabled(reason))
        }
        Flat7xD0AudioSpanInput::Provided(receipt) => {
            let receipt = *receipt;
            if receipt.source_identity != *expected_source_identity {
                return Err(Flat7xPairSourceError::D0AudioSpanSourceIdentityMismatch);
            }
            Ok(Flat7xD0FrameInput {
                audio_blocks: receipt.audio_blocks,
                frame_tail_policy: receipt.frame_tail_policy,
            })
        }
    }
}

/// Source-bound audio-block inputs ready for the existing I0/D0 frame assemblers.
///
/// The role-map receipt retains the replaceable ADM-row projection assumption. This candidate
/// authorizes no frame publication, AU append, or product output.
#[must_use]
#[derive(Clone)]
pub struct Flat7xPairSourceBinding<'pcm> {
    i0_header: Flat7xI0Ac3FrameHeaderFields,
    resolved_profile: ResolvedEncodingProfile,
    target: Flat7xI0D0TargetPlan,
    source_index: CodecSourceIndex,
    role_map_receipt: Flat7xSourceRoleMapReceipt,
    source_extent: Flat7xD0PcmSourceExtent,
    d0_source_identity: Flat7xD0SourceIdentity,
    i0_rows: Flat7xI0PcmRows<'pcm>,
    d0_rows: Flat7xD0PcmRows<'pcm>,
}

impl Flat7xPairSourceBinding<'_> {
    pub fn with_programme_levels(mut self, dialnorm: u8, compr: Option<u8>) -> Self {
        self.i0_header.dialnorm = dialnorm;
        self.i0_header.compr = compr;
        self
    }
    pub const fn resolved_profile(&self) -> ResolvedEncodingProfile {
        self.resolved_profile
    }

    pub const fn target(&self) -> Flat7xI0D0TargetPlan {
        self.target
    }

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn role_map_receipt(&self) -> &Flat7xSourceRoleMapReceipt {
        &self.role_map_receipt
    }

    pub const fn source_extent(&self) -> Flat7xD0PcmSourceExtent {
        self.source_extent
    }

    pub const fn d0_source_identity(&self) -> &Flat7xD0SourceIdentity {
        &self.d0_source_identity
    }

    pub const fn i0_rows(&self) -> Flat7xI0PcmRows<'_> {
        self.i0_rows
    }

    pub const fn d0_rows(&self) -> Flat7xD0PcmRows<'_> {
        self.d0_rows
    }

    pub fn is_bound_to_projection(&self, projection: &Flat7xSourcePcmProjection<'_>) -> bool {
        self.role_map_receipt == *projection.role_map_receipt()
            && self.source_extent == projection.source_extent()
            && self.i0_rows == projection.i0_rows()
            && self.d0_rows == projection.d0_rows()
    }
}

/// Full D0 frame candidate bound to the pair source's D0 role-map and analysis lineage.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xPairD0PcmFrameCandidate {
    source_identity: Flat7xD0SourceIdentity,
    source_role_map_receipt: Flat7xSourceRoleMapReceipt,
    frame_candidate: Flat7xD0PcmFrameCandidate,
    block2_placement: D0Block2PlacementReceipt,
}

/// Six-block D0 transform and next overlap history for a bound pair source.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xPairD0PcmBlockSeriesCandidate {
    source_identity: Flat7xD0SourceIdentity,
    source_role_map_receipt: Flat7xSourceRoleMapReceipt,
    source_extent: Flat7xD0PcmSourceExtent,
    block_series: Flat7xD0PcmBlockSeriesCandidate,
}

impl Flat7xPairD0PcmBlockSeriesCandidate {
    pub const fn source_identity(&self) -> &Flat7xD0SourceIdentity {
        &self.source_identity
    }

    pub const fn source_role_map_receipt(&self) -> &Flat7xSourceRoleMapReceipt {
        &self.source_role_map_receipt
    }

    pub const fn source_extent(&self) -> Flat7xD0PcmSourceExtent {
        self.source_extent
    }

    pub const fn after_history(&self) -> &Flat7xD0OverlapHistory {
        self.block_series.after_history()
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

/// Stages D0 PCM history for a source that advances codec overlap without publishing an AU.
pub fn stage_flat7x_pair_d0_pcm_block_series(
    binding: &Flat7xPairSourceBinding<'_>,
    before: &Flat7xD0OverlapHistory,
    transform_parameters: &jocforge_domain::transform::math_generated_parameters::MathGeneratedTransformParameters,
) -> Result<Flat7xPairD0PcmBlockSeriesCandidate, Flat7xPairSourceError> {
    let source_extent = binding.source_extent();
    let valid_samples = usize::from(source_extent.valid_samples());
    let d0_rows = binding.d0_rows.rows();
    for (channel, row) in d0_rows.iter().enumerate() {
        if row.len() == 1_536 && row[valid_samples..].iter().any(|word| *word != 0) {
            return Err(Flat7xPairSourceError::D0PcmFrame(
                Flat7xD0PcmSourceError::PcmPaddingMismatch {
                    channel: channel as u8,
                },
            ));
        }
    }
    let block_series = stage_flat7x_d0_pcm_block_series(d0_rows, before, transform_parameters)
        .map_err(Flat7xPairSourceError::D0PcmFrame)?;
    Ok(Flat7xPairD0PcmBlockSeriesCandidate {
        source_identity: binding.d0_source_identity.clone(),
        source_role_map_receipt: binding.role_map_receipt.clone(),
        source_extent,
        block_series,
    })
}

impl Flat7xPairD0PcmFrameCandidate {
    pub const fn d0_source_identity(&self) -> &Flat7xD0SourceIdentity {
        &self.source_identity
    }

    pub const fn source_role_map_receipt(&self) -> &Flat7xSourceRoleMapReceipt {
        &self.source_role_map_receipt
    }

    pub const fn frame_candidate(&self) -> &Flat7xD0PcmFrameCandidate {
        &self.frame_candidate
    }

    pub const fn source_extent(&self) -> Flat7xD0PcmSourceExtent {
        self.frame_candidate.source_extent()
    }

    pub const fn block2_placement_receipt(&self) -> &D0Block2PlacementReceipt {
        &self.block2_placement
    }

    pub const fn after_history(
        &self,
    ) -> &jocforge_domain::codec::flat7x_d0_pcm_source::Flat7xD0OverlapHistory {
        self.frame_candidate.after_history()
    }

    /// Returns the first payload bit after the D0 block-two `skipfld` header.
    pub fn emdf_container_start(&self) -> Option<FrameBitCursor> {
        let skip = self
            .frame_candidate
            .frame()
            .receipt()
            .frame_assembly()
            .layout()
            .block_two_skip_field();
        let payload_bits = u32::from(skip.payload_bytes()).checked_mul(8)?;
        let payload_start = skip.bit_span().end_bit().checked_sub(payload_bits)?;
        Some(FrameBitCursor::new(payload_start))
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

/// Runs the PCM-to-frame candidate against the exact D0 rows carried by a bound pair source.
///
/// The returned proposal keeps the D0 source identity and replaceable role-map receipt beside the
/// frame/history candidate. It does not create F12/F13/F15 receipts or install an owner root.
pub fn produce_flat7x_pair_d0_pcm_frame_candidate(
    binding: &Flat7xPairSourceBinding<'_>,
    settings: Flat7xD0PcmFrameSettings<'_>,
) -> Result<Flat7xPairD0PcmFrameCandidate, Flat7xPairSourceError> {
    if settings.target != binding.target {
        return Err(Flat7xPairSourceError::ProfileTargetMismatch);
    }
    if settings.source_extent != binding.source_extent {
        return Err(Flat7xPairSourceError::SourceExtentMismatch);
    }
    let d0_identity = binding.d0_source_identity();
    let shared_identity = binding.role_map_receipt().shared_analysis_identity();
    let payload_sequence = settings.block2_source_receipt.payload_sequence();
    if settings.source_generation_receipt
        != settings.block2_source_receipt.source_generation_receipt()
        || !settings.block2_source_receipt.matches_source_identity(
            d0_identity.programme_id(),
            d0_identity.attempt_id(),
            d0_identity.callback_index(),
            d0_identity.source_transaction_id(),
            d0_identity.source_index(),
            shared_identity.analysis_after_generation(),
        )
    {
        return Err(Flat7xPairSourceError::D0Block2SourceIdentityMismatch);
    }
    let au_index = settings.block2_source_receipt.au_index();
    let frame_candidate = produce_flat7x_d0_frame_from_pcm_source(Flat7xD0PcmFrameInput {
        pcm_rows: binding.d0_rows.rows(),
        settings,
    })
    .map_err(Flat7xPairSourceError::D0PcmFrame)?;

    let layout = frame_candidate.frame().receipt().frame_assembly().layout();
    let block_index = layout.audio_blocks()[2].block_index();
    let skip_field = layout.block_two_skip_field();
    let bit_span = skip_field.bit_span();
    let container_bytes = skip_field.payload_bytes();
    let expected_bit_length = 10 + u32::from(container_bytes) * 8;
    if block_index.zero_based_value() != 2
        || container_bytes == 0
        || container_bytes > 511
        || bit_span.bit_length() != expected_bit_length
    {
        return Err(Flat7xPairSourceError::D0Block2PlacementMismatch);
    }

    Ok(Flat7xPairD0PcmFrameCandidate {
        source_identity: binding.d0_source_identity.clone(),
        source_role_map_receipt: binding.role_map_receipt.clone(),
        frame_candidate,
        block2_placement: D0Block2PlacementReceipt {
            block_index,
            bit_span,
            container_bytes,
            payload_sequence,
            source_generation_receipt: settings.source_generation_receipt.clone(),
            au_index,
        },
    })
}

/// Source-bound audio-block inputs ready for the existing I0/D0 frame assemblers.
///
/// The role-map receipt retains the replaceable ADM-row projection assumption. This candidate
/// authorizes no frame publication, AU append, or product output.
#[must_use]
pub struct Flat7xPairAudioBlockCandidate<'pcm, 'd0> {
    binding: Flat7xPairSourceBinding<'pcm>,
    i0_audio_blocks: I0AudioBlockSeriesCandidate,
    d0_audio_blocks: Flat7xD0FrameBitInput<'d0>,
    d0_frame_tail_policy: Flat7xD0FrameTailPolicy,
}

impl<'pcm, 'd0> Flat7xPairAudioBlockCandidate<'pcm, 'd0> {
    pub const fn target(&self) -> Flat7xI0D0TargetPlan {
        self.binding.target
    }

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.binding.source_index
    }

    pub const fn role_map_receipt(&self) -> &Flat7xSourceRoleMapReceipt {
        &self.binding.role_map_receipt
    }

    pub const fn i0_rows(&self) -> Flat7xI0PcmRows<'pcm> {
        self.binding.i0_rows
    }

    pub const fn d0_rows(&self) -> Flat7xD0PcmRows<'pcm> {
        self.binding.d0_rows
    }

    pub const fn source_binding(&self) -> &Flat7xPairSourceBinding<'pcm> {
        &self.binding
    }

    pub const fn i0_audio_blocks(&self) -> &I0AudioBlockSeriesCandidate {
        &self.i0_audio_blocks
    }

    pub const fn d0_audio_blocks(&self) -> Flat7xD0FrameBitInput<'d0> {
        self.d0_audio_blocks
    }

    pub const fn d0_frame_tail_policy(&self) -> Flat7xD0FrameTailPolicy {
        self.d0_frame_tail_policy
    }
}

/// Source-bound I0 frame candidate paired with the selected projection receipt.
///
/// The frame remains in memory with its provisional audio assumptions and has no AU or
/// publication authority.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xPairI0FrameCandidate {
    resolved_profile: ResolvedEncodingProfile,
    source_role_map_receipt: Flat7xSourceRoleMapReceipt,
    source_extent: Flat7xD0PcmSourceExtent,
    source_generation_receipt: Flat7xD0SourceGenerationReceipt,
    i0_frame: Flat7xI0Ac3AudioSeriesFrameCandidate,
}

impl Flat7xPairI0FrameCandidate {
    pub const fn resolved_profile(&self) -> ResolvedEncodingProfile {
        self.resolved_profile
    }

    pub fn source_identity(&self) -> &SharedAnalysisIdentity {
        self.i0_frame.source_identity()
    }

    pub const fn target(&self) -> Flat7xI0D0TargetPlan {
        self.i0_frame.target()
    }

    pub const fn source_role_map_receipt(&self) -> &Flat7xSourceRoleMapReceipt {
        &self.source_role_map_receipt
    }

    pub const fn source_extent(&self) -> Flat7xD0PcmSourceExtent {
        self.source_extent
    }

    pub const fn source_generation_receipt(&self) -> &Flat7xD0SourceGenerationReceipt {
        &self.source_generation_receipt
    }

    pub const fn i0_frame(&self) -> &Flat7xI0Ac3AudioSeriesFrameCandidate {
        &self.i0_frame
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

/// Joins the selected pair profile/source receipt and the six-block I0 candidate to frame assembly.
///
/// D0 spans must already be present on `pair_audio`; the returned value is an I0 frame candidate,
/// not an access unit and not a publication proposal. Auxdata and the initial `crcrsv` bit are
/// caller-supplied inputs.
pub fn assemble_flat7x_pair_i0_frame_candidate(
    pair_audio: &Flat7xPairAudioBlockCandidate<'_, '_>,
    source_generation_receipt: &Flat7xD0SourceGenerationReceipt,
    auxdata: I0AuxDataInput<'_>,
    crcrsv_seed: CallerOwnedI0CrcrsvSeed,
) -> Result<Flat7xPairI0FrameCandidate, Flat7xPairSourceError> {
    assemble_flat7x_i0_frame_candidate_from_series(
        &pair_audio.binding,
        &pair_audio.i0_audio_blocks,
        source_generation_receipt,
        auxdata,
        crcrsv_seed,
    )
}

/// Builds the Flat7x I0 audio policy from the same-source selected core frame's accepted q.
pub fn flat7x_i0_audio_block_policy_v1(
    selected_core_q: I0BapCandidateQ,
) -> Option<I0AudioBlockSeriesPolicy> {
    Some(I0AudioBlockSeriesPolicy {
        blocks: [I0AudioBlockControlPolicy {
            candidate_q: selected_core_q,
            slow_decay_code: 2,
            fast_decay_code: 1,
            slow_gain_code: 1,
            floor_code: 7,
            fast_gain_codes: [0; 6],
            caller_all_active_fine_zero: false,
            dither_flags: [false; 5],
            dynamic_range_code: None,
        }; 6],
        cplinu: CallerOwnedAc3Cplinu::Independent,
        delta_bit_allocation: Ac3DeltaBitAllocationInput::Absent,
        quantizer_policy: Ac3MantissaQuantizerPolicy::NearestTiesEven,
    })
}

/// Builds the existing source-bound I0 frame candidate without consuming D0 block data.
///
/// It prepares I0 from the exact selected numeric-plan receipt and explicit caller-owned controls;
/// later pair construction still requires a caller-owned opaque D0 frame span and F13/I0/D0
/// receipts. The returned candidate grants no publication authority.
pub fn assemble_flat7x_i0_frame_candidate_from_binding(
    binding: &Flat7xPairSourceBinding<'_>,
    numeric_plan: &NumericPlanReceipt<'_, SelectedSourceNumericPlan>,
    source_generation_receipt: &Flat7xD0SourceGenerationReceipt,
    i0_policy: I0AudioBlockSeriesPolicy,
    i0_budget: I0AudioBlockSeriesBudget,
    auxdata: I0AuxDataInput<'_>,
    crcrsv_seed: CallerOwnedI0CrcrsvSeed,
) -> Result<Flat7xPairI0FrameCandidate, Flat7xPairSourceError> {
    if binding.role_map_receipt.shared_analysis_identity() != numeric_plan.identity()
        || binding.source_index != numeric_plan.identity().source_index()
    {
        return Err(Flat7xPairSourceError::SourceIdentityMismatch);
    }
    let i0_audio_blocks = produce_i0_audio_block_series(numeric_plan, i0_policy, i0_budget)
        .map_err(Flat7xPairSourceError::I0AudioBlockSeries)?;
    assemble_flat7x_i0_frame_candidate_from_series(
        binding,
        &i0_audio_blocks,
        source_generation_receipt,
        auxdata,
        crcrsv_seed,
    )
}

fn assemble_flat7x_i0_frame_candidate_from_series(
    binding: &Flat7xPairSourceBinding<'_>,
    i0_audio_blocks: &I0AudioBlockSeriesCandidate,
    source_generation_receipt: &Flat7xD0SourceGenerationReceipt,
    auxdata: I0AuxDataInput<'_>,
    crcrsv_seed: CallerOwnedI0CrcrsvSeed,
) -> Result<Flat7xPairI0FrameCandidate, Flat7xPairSourceError> {
    if binding.target != binding.role_map_receipt.target()
        || binding.resolved_profile.joc_reconstruction_layout() != JocReconstructionLayout::Flat7x
        || binding.resolved_profile.codec_rate() != CodecRateProfile::Kbps1024
        || binding.source_index != binding.role_map_receipt.source_index()
        || binding.source_index != i0_audio_blocks.receipt().source_identity().source_index()
    {
        return Err(Flat7xPairSourceError::SourceIdentityMismatch);
    }
    if binding.source_extent.source_index() != binding.source_index
        || binding.source_extent.callback_index() != binding.role_map_receipt.callback_index()
    {
        return Err(Flat7xPairSourceError::SourceExtentMismatch);
    }
    if !d0_source_generation_matches_binding(binding, source_generation_receipt) {
        return Err(Flat7xPairSourceError::D0Block2SourceIdentityMismatch);
    }

    let i0_frame = assemble_flat7x_i0_audio_with_header(
        binding.target,
        binding.role_map_receipt.shared_analysis_identity(),
        i0_audio_blocks,
        auxdata,
        crcrsv_seed,
        binding.i0_header,
        2, // Lo/Ro for the horizontal programme mix.
    )
    .map_err(Flat7xPairSourceError::I0FrameAssembly)?;

    Ok(Flat7xPairI0FrameCandidate {
        resolved_profile: binding.resolved_profile,
        source_role_map_receipt: binding.role_map_receipt.clone(),
        source_extent: binding.source_extent,
        source_generation_receipt: source_generation_receipt.clone(),
        i0_frame,
    })
}

fn d0_source_generation_matches_binding(
    binding: &Flat7xPairSourceBinding<'_>,
    receipt: &Flat7xD0SourceGenerationReceipt,
) -> bool {
    let identity = binding.d0_source_identity();
    let lineage = receipt.callback_lineage();
    receipt.source_transaction_id() == identity.source_transaction_id()
        && receipt.source_index() == identity.source_index()
        && lineage.programme_id() == identity.programme_id()
        && lineage.attempt_id() == identity.attempt_id()
        && lineage.callback_index() == identity.callback_index()
        && receipt.analysis_generation().value() == identity.analysis_generation().value()
}

/// Complete in-memory I0 and D0 AU candidate with the selected profile and both source receipts.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xPairAccessUnitCandidate {
    resolved_profile: ResolvedEncodingProfile,
    source_role_map_receipt: Flat7xSourceRoleMapReceipt,
    source_extent: Flat7xD0PcmSourceExtent,
    d0_source_identity: Flat7xD0SourceIdentity,
    access_unit: Flat7xI0D0AuCandidate,
}

impl Flat7xPairAccessUnitCandidate {
    pub const fn resolved_profile(&self) -> ResolvedEncodingProfile {
        self.resolved_profile
    }

    pub const fn target(&self) -> Flat7xI0D0TargetPlan {
        self.access_unit.target()
    }

    pub fn source_identity(&self) -> &SharedAnalysisIdentity {
        self.access_unit.source_identity()
    }

    pub const fn source_role_map_receipt(&self) -> &Flat7xSourceRoleMapReceipt {
        &self.source_role_map_receipt
    }

    pub const fn source_extent(&self) -> Flat7xD0PcmSourceExtent {
        self.source_extent
    }

    pub const fn d0_source_identity(&self) -> &Flat7xD0SourceIdentity {
        &self.d0_source_identity
    }

    pub const fn access_unit(&self) -> &Flat7xI0D0AuCandidate {
        &self.access_unit
    }

    pub const fn access_unit_bytes(
        &self,
    ) -> &[u8; jocforge_domain::codec::access_unit_composer::FLAT7X_AU_FRAME_BYTES] {
        self.access_unit.access_unit_bytes()
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

/// Builds the paired I0 and D0 byte candidate from the same profile, source receipt and D0 input.
///
/// The D0 BSI and six opaque audio spans remain caller-owned. The existing D0 assembler validates
/// its target, six block positions and single block-index-2 skip-field before the I0 and D0 composer
/// emits one in-memory AU candidate. This function performs no CLI, AU-owner, file or publication
/// call.
pub fn assemble_flat7x_pair_access_unit_candidate(
    pair_audio: &Flat7xPairAudioBlockCandidate<'_, '_>,
    i0_frame: &Flat7xPairI0FrameCandidate,
    d0_bsi: Flat7xD0BsiStructure,
) -> Result<Flat7xPairAccessUnitCandidate, Flat7xPairSourceError> {
    let binding = &pair_audio.binding;
    let role_map = &binding.role_map_receipt;
    let shared_identity = role_map.shared_analysis_identity();
    if binding.resolved_profile != i0_frame.resolved_profile
        || binding.target != i0_frame.target()
        || role_map != i0_frame.source_role_map_receipt()
        || i0_frame.source_identity() != shared_identity
        || pair_audio.i0_audio_blocks.receipt().source_identity() != shared_identity
        || binding.source_index != shared_identity.source_index()
        || !d0_identity_matches_role_map(&binding.d0_source_identity, role_map)
    {
        return Err(Flat7xPairSourceError::SourceIdentityMismatch);
    }
    if binding.source_extent != i0_frame.source_extent {
        return Err(Flat7xPairSourceError::SourceExtentMismatch);
    }

    let access_unit = assemble_flat7x_i0_d0_au_candidate_with_tail_policy(
        binding.target,
        shared_identity,
        i0_frame.i0_frame.clone(),
        d0_bsi,
        pair_audio.d0_audio_blocks,
        pair_audio.d0_frame_tail_policy,
    )
    .map_err(Flat7xPairSourceError::AccessUnitAssembly)?;

    Ok(Flat7xPairAccessUnitCandidate {
        resolved_profile: binding.resolved_profile,
        source_role_map_receipt: role_map.clone(),
        source_extent: i0_frame.source_extent,
        d0_source_identity: binding.d0_source_identity.clone(),
        access_unit,
    })
}

pub(crate) fn d0_identity_matches_role_map(
    d0_identity: &Flat7xD0SourceIdentity,
    role_map: &Flat7xSourceRoleMapReceipt,
) -> bool {
    let shared_identity = role_map.shared_analysis_identity();
    let source_stage = shared_identity.source_stage_key();
    d0_identity.matches_pair_source(
        role_map.programme_id(),
        source_stage.attempt_id(),
        role_map.callback_index(),
        source_stage.transaction_id(),
        role_map.source_index(),
        Flat7xAnalysisGeneration(shared_identity.analysis_after_generation().value()),
    )
}

/// Binds one selected Flat7x profile, source projection, and numeric plan to one source identity.
pub fn bind_flat7x_pair_source<'pcm>(
    profile: ResolvedEncodingProfile,
    projection: &Flat7xSourcePcmProjection<'pcm>,
    numeric_plan: &NumericPlanReceipt<'_, SelectedSourceNumericPlan>,
) -> Result<Flat7xPairSourceBinding<'pcm>, Flat7xPairSourceError> {
    if profile.joc_reconstruction_layout() != JocReconstructionLayout::Flat7x
        || profile.codec_rate() != CodecRateProfile::Kbps1024
    {
        return Err(Flat7xPairSourceError::UnsupportedProfile);
    }

    let target = Flat7xI0D0TargetPlan::approved_contract();
    let role_map_receipt = projection.role_map_receipt();
    if role_map_receipt.target() != target || target.d0().rate() != profile.codec_rate() {
        return Err(Flat7xPairSourceError::ProfileTargetMismatch);
    }
    if role_map_receipt.shared_analysis_identity() != numeric_plan.identity()
        || role_map_receipt.source_index() != numeric_plan.identity().source_index()
    {
        return Err(Flat7xPairSourceError::SourceIdentityMismatch);
    }
    Ok(Flat7xPairSourceBinding {
        i0_header: Flat7xI0Ac3FrameHeaderFields::official_demo_sample_non_crc_fields(),
        resolved_profile: profile,
        target,
        source_index: numeric_plan.identity().source_index(),
        role_map_receipt: role_map_receipt.clone(),
        source_extent: projection.source_extent(),
        d0_source_identity: Flat7xD0SourceIdentity::from_analysis(
            role_map_receipt.programme_id(),
            role_map_receipt.callback_index(),
            numeric_plan.identity(),
        ),
        i0_rows: projection.i0_rows(),
        d0_rows: projection.d0_rows(),
    })
}

/// Prepares the existing I0 audio-block series and pairs it with one typed D0 span input.
///
/// I0 policy values remain explicit caller inputs. The current D0 no-rule state returns before
/// creating any I0-only candidate.
pub fn prepare_flat7x_pair_audio_blocks<'pcm, 'd0>(
    binding: Flat7xPairSourceBinding<'pcm>,
    numeric_plan: &NumericPlanReceipt<'_, SelectedSourceNumericPlan>,
    i0_policy: I0AudioBlockSeriesPolicy,
    i0_budget: I0AudioBlockSeriesBudget,
    d0_audio: Flat7xD0AudioSpanInput<'d0>,
) -> Result<Flat7xPairAudioBlockCandidate<'pcm, 'd0>, Flat7xPairSourceError> {
    if binding.role_map_receipt.shared_analysis_identity() != numeric_plan.identity()
        || binding.source_index != numeric_plan.identity().source_index()
    {
        return Err(Flat7xPairSourceError::SourceIdentityMismatch);
    }
    let d0_frame_input = require_flat7x_d0_frame_input(d0_audio, binding.d0_source_identity())?;
    let i0_audio_blocks = produce_i0_audio_block_series(numeric_plan, i0_policy, i0_budget)
        .map_err(Flat7xPairSourceError::I0AudioBlockSeries)?;

    Ok(Flat7xPairAudioBlockCandidate {
        binding,
        i0_audio_blocks,
        d0_audio_blocks: d0_frame_input.audio_blocks(),
        d0_frame_tail_policy: d0_frame_input.frame_tail_policy(),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        Flat7xAnalysisGeneration, Flat7xD0AudioSpanInput, Flat7xD0AudioSpanProducerState,
        Flat7xD0AudioSpanReceipt, Flat7xD0FrameTailPolicy, Flat7xD0SourceIdentity,
        Flat7xPairSourceError, require_flat7x_d0_audio_blocks, require_flat7x_d0_frame_input,
    };
    use jocforge_domain::codec::d0_frame_assembler::{
        D0FrameAudioBlockInput, Flat7xD0FrameBitInput, OpaqueD0AudioBitSpan,
    };
    use jocforge_domain::codec::flat7x_d0_frame_controls::{
        Flat7xD0AllocationCandidateQ, Flat7xD0AudioFrameControlPolicy,
    };
    use jocforge_domain::{
        timeline::{CodecSourceIndex, FrontendCallbackIndex},
        transaction::{AttemptId, CallbackTransactionId, ProgrammeId},
    };

    fn fixture_source_identity(generation: u64) -> Flat7xD0SourceIdentity {
        Flat7xD0SourceIdentity {
            programme_id: ProgrammeId::new("flat7x-programme").unwrap(),
            attempt_id: AttemptId::new(2),
            callback_index: FrontendCallbackIndex::new(9),
            source_transaction_id: CallbackTransactionId::new("source-7").unwrap(),
            source_index: CodecSourceIndex::new(7),
            analysis_generation: Flat7xAnalysisGeneration(generation),
        }
    }

    fn fixture_span_receipt(
        source_identity: Flat7xD0SourceIdentity,
        byte: &[u8],
    ) -> Flat7xD0AudioSpanReceipt<'_> {
        let block = D0FrameAudioBlockInput::Opaque(OpaqueD0AudioBitSpan::new(byte, 0, 1));
        Flat7xD0AudioSpanReceipt {
            source_identity,
            audio_blocks: Flat7xD0FrameBitInput {
                audio_frame_controls: Flat7xD0AudioFrameControlPolicy::new(
                    Flat7xD0AllocationCandidateQ::new(1).unwrap(),
                ),
                audio_blocks: [block; 6],
            },
            frame_tail_policy: Flat7xD0FrameTailPolicy::ZeroFillUnusedAuxBits,
        }
    }

    #[test]
    fn d0_source_identity_matches_the_pair_source_generation_and_transaction() {
        let identity = fixture_source_identity(9);
        let programme_id = ProgrammeId::new("flat7x-programme").unwrap();
        let source_transaction_id = CallbackTransactionId::new("source-7").unwrap();

        assert!(identity.matches_pair_source(
            &programme_id,
            AttemptId::new(2),
            FrontendCallbackIndex::new(9),
            &source_transaction_id,
            CodecSourceIndex::new(7),
            Flat7xAnalysisGeneration(9),
        ));
        assert!(!identity.matches_pair_source(
            &programme_id,
            AttemptId::new(2),
            FrontendCallbackIndex::new(9),
            &source_transaction_id,
            CodecSourceIndex::new(7),
            Flat7xAnalysisGeneration(8),
        ));
    }

    #[test]
    fn d0_audio_span_input_keeps_disabled_and_provided_states_distinct() {
        let disabled = Flat7xD0AudioSpanInput::Disabled(
            Flat7xD0AudioSpanProducerState::NoApprovedAudioProductionRule,
        );
        assert_eq!(
            require_flat7x_d0_audio_blocks(disabled, &fixture_source_identity(9)),
            Err(Flat7xPairSourceError::D0AudioSpanProducerDisabled(
                Flat7xD0AudioSpanProducerState::NoApprovedAudioProductionRule,
            ))
        );

        let byte = [0_u8];
        let expected = fixture_source_identity(9);
        let provided = Flat7xD0AudioSpanInput::Provided(Box::new(fixture_span_receipt(
            expected.clone(),
            &byte,
        )));
        assert!(require_flat7x_d0_audio_blocks(provided, &expected).is_ok());
    }

    #[test]
    fn d0_audio_span_receipt_rejects_source_generation_mismatch() {
        let byte = [0_u8];
        let expected = fixture_source_identity(9);
        let receipt = fixture_span_receipt(fixture_source_identity(8), &byte);

        assert_eq!(
            require_flat7x_d0_audio_blocks(
                Flat7xD0AudioSpanInput::Provided(Box::new(receipt)),
                &expected,
            ),
            Err(Flat7xPairSourceError::D0AudioSpanSourceIdentityMismatch)
        );
    }

    #[test]
    fn source_bound_d0_frame_input_carries_tail_policy_with_the_receipt() {
        let byte = [0xa5_u8];
        let expected = fixture_source_identity(9);
        let receipt = fixture_span_receipt(expected.clone(), &byte);

        let input = require_flat7x_d0_frame_input(
            Flat7xD0AudioSpanInput::Provided(Box::new(receipt)),
            &expected,
        )
        .unwrap();

        assert_eq!(input.audio_blocks().audio_blocks.len(), 6);
        assert_eq!(
            input.frame_tail_policy(),
            Flat7xD0FrameTailPolicy::ZeroFillUnusedAuxBits
        );
    }
}

#[cfg(all(test, feature = "test-support"))]
mod pair_access_unit_integration_tests {
    use std::{ops::Range, sync::Arc};

    use super::{
        Flat7xD0AudioSpanInput, Flat7xD0AudioSpanReceipt, Flat7xD0Ci16Addbsi,
        Flat7xD0SourceIdentity, Flat7xPairSourceError,
        assemble_flat7x_i0_frame_candidate_from_binding,
        assemble_flat7x_pair_access_unit_candidate, assemble_flat7x_pair_i0_frame_candidate,
        bind_flat7x_pair_source, prepare_flat7x_pair_audio_blocks,
        produce_flat7x_pair_d0_pcm_frame_candidate,
    };
    use crate::{
        PreparedJobInput, SelectedAdmResourcePayloads, SelectedWaveDescriptor,
        flat7x_pair_integrity::{Flat7xPairF12StageInput, stage_flat7x_pair_f12_integrity},
        flat7x_pair_output::{
            Flat7xOpaqueD0FrameInput, Flat7xPairAppendProposalError, Flat7xPairIoStageDisposition,
            Flat7xPairIoStageError, preflight_flat7x_pair_io_stage, stage_flat7x_pcm_pair_for_f15,
            stage_receipted_i0_with_opaque_d0,
        },
        prepare_selected_job_input,
        source_frame_bridge::build_flat7x_source_pcm_projection,
    };
    use jocforge_domain::{
        analysis::{
            input_history::INPUT_HISTORY_BLOCK_SAMPLES,
            owner::{AnalysisNumericPlanStage, AnalysisOwnerRoot},
            source_frame::InstalledAudioSourceFrame,
            structure_refresh::{F07_BLOCKS_PER_FRAME, F07_FULLBAND_LANES},
            transient_parameters::load_transient_parameters,
        },
        codec::{
            ac3_audio_block::{
                Ac3DeltaBitAllocationInput, Ac3MantissaQuantizerPolicy, I0AudioBlockControlPolicy,
                I0AudioBlockSeriesBudget, I0AudioBlockSeriesPolicy, I0BapCandidateQ,
            },
            bap::{BapParameters, Flat7xD0BapParameters, load_bap_parameters},
            d0_frame_assembler::{
                D0_CRC2_FIELD_BITS, D0Block2F13SourceLineage, D0Block2F14kSourceReceiptInput,
                D0Block2SkipFieldInput, D0Block2SourceReceipt, D0Block2SourceReceiptError,
                D0BlockTwoAudioSpans, D0FrameAudioBlockInput, Flat7xD0FrameBitInput,
                Flat7xD0FrameTailPolicy, OpaqueD0AudioBitSpan,
            },
            d0_frame_layout::CallerOwnedUnboundD0Block2InsertionPoint,
            emdf_skip_field::flat7x_d0_block2_skipfld_bit_length,
            exponent_packing::pack_exponent_difference_group,
            flat7x_d0_frame_controls::{
                Flat7xD0AllocationCandidateQ, Flat7xD0AudioFrameControlPolicy,
            },
            flat7x_d0_pcm_source::{
                Flat7xD0OverlapHistory, Flat7xD0PcmFrameSettings, Flat7xD0PcmSourceExtent,
            },
            frame_builder::{Flat7xD0BsiStructure, measure_flat7x_d0_bsi_structure},
            i0_frame_assembler::{
                CallerOwnedAc3Cplinu, CallerOwnedI0CrcrsvSeed, Flat7xI0FrameAssemblyError,
                I0AuxDataInput, OpaqueI0Ac3BitSpan,
            },
            owner::Flat7xD0SourceGenerationReceipt,
            policy_parameters::{PolicyProfile, load_policy_parameters},
            selected_frame_trial::{
                BlockExponentPlan, CoefficientBlockInput, ConverterHeaderInput,
                FullbandExponentLanePlan, LfeExponentPlan, MainExponentSyntax,
                SelectedSourceNumericPlan,
            },
        },
        integrity::{
            keyless_checksum::validate_keyless_provider,
            programme_owner::{
                F12CompletedFrameInput, F14kPayloadInput, IntegrityF12SourceInput,
                IntegrityIdentity, IntegrityProgrammeOwner,
            },
        },
        joc::emdf::metadata_owner::test_support::{
            Flat7xReceiptGateMetadataTestInputs, flat7x_receipt_gate_metadata_for_test,
            flat7x_receipt_gate_metadata_for_test_with_payload_sequence,
        },
        profile::{
            D0EmdfSkipFieldExtent, EncodingProfile, Flat7xI0D0TargetPlan, Flat7xSourceGeneration,
            JocReconstructionLayout, resolve_selected_profile,
        },
        publication::{
            Flat7xAuAttemptRetirementReason, Flat7xAuIntegrityFrameReceipt, Flat7xAuOutputOwner,
            Flat7xAuOutputOwnerError, Flat7xAuSourceReceiptInput,
            test_support::Flat7xAuAppendTestFixture,
        },
        timeline::{
            AdmSampleIndex, CodecSourceIndex, FrontendCallbackIndex, PublicationDisposition,
            PublishedAuIndex, publication_for_source,
        },
        transaction::{AttemptId, CallbackLineageIdentity, CallbackTransactionId, ProgrammeId},
        transform::{
            fullband_exponent_pipeline::NumericPlanReceipt,
            math_generated_parameters::MathGeneratedTransformParameters,
        },
    };
    use serde_json::Value;

    const INPUT_XML_VECTORS: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    );
    const AXML: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.xml"
    );
    const CHNA: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_track_associations.bin"
    );
    const DBMD: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.dbmd"
    );
    const TRANSIENT_PARAMETERS: &[u8] =
        include_bytes!("../../jocforge-domain/tests/fixtures/transient_channel/PARAMETERS.json");
    const F08_PARAMETERS: &[u8] = include_bytes!(
        "../../jocforge-domain/src/transform/ordinary_transform_math_parameters.json"
    );
    const POLICY_PARAMETERS: &[u8] =
        include_bytes!("../../jocforge-domain/tests/fixtures/policy_authorized/PARAMETERS.json");
    const BAP_PROVIDER: &[u8] = include_bytes!(
        "../../../outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-PUBLIC-BAP-PROVIDER-V1/public-bap-48k-v1.json"
    );
    const F15_PAIR_VECTORS: &[u8] = include_bytes!(
        "../../../outputs/IMPLEMENTER_SUPPLEMENTS/FLAT7X-F15-RECEIPT-PAIR-INPUT-ONLY-V1/INPUT-ONLY-VECTORS.json"
    );
    const METADATA_OWNER_VECTORS: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/metadata_owner/METADATA-OWNER-VECTORS.json"
    );
    const METADATA_PUBLIC_PROVIDER: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/metadata_owner/metadata-public-provider.json"
    );
    const METADATA_WRAPPER_CONFIG: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/metadata_owner/metadata-wrapper-config.json"
    );
    const KEYLESS_INTEGRITY_PROVIDER: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/integrity_owner/integrity-public-provider.json"
    );
    const PHYSICAL_CHANNELS: u16 = 38;
    const PHYSICAL_PCM_SAMPLES: usize = 1_536;
    const FULLBAND_CHANNELS: usize = 5;
    const FULLBAND_BINS: usize = 217;
    const LFE_BINS: usize = 7;

    fn d0_block2_source_receipt_for_test<'a>(
        identity: &Flat7xD0SourceIdentity,
        container: &'a [u8],
    ) -> (Flat7xD0SourceGenerationReceipt, D0Block2SourceReceipt<'a>) {
        let source_index = identity.source_index();
        let source_transaction_id = identity.source_transaction_id().clone();
        let callback_lineage = CallbackLineageIdentity::new(
            identity.programme_id().clone(),
            identity.attempt_id(),
            identity.callback_index(),
        );
        let source_generation_receipt = Flat7xD0SourceGenerationReceipt::synthetic_for_test(
            source_transaction_id.clone(),
            41,
            callback_lineage.clone(),
            source_index,
            identity.analysis_generation().value(),
        );
        let block2_source_receipt = D0Block2SourceReceipt::synthetic_for_source_for_test(
            &source_generation_receipt,
            &source_generation_receipt,
            (12, 13),
            (
                source_index,
                jocforge_domain::timeline::JocGeneratedWriterIndex::new(source_index.value()),
            ),
            container,
        )
        .unwrap();
        (source_generation_receipt, block2_source_receipt)
    }

    fn d0_block2_source_receipt_with_generation_for_test<'a>(
        identity: &Flat7xD0SourceIdentity,
        source_generation_receipt: &Flat7xD0SourceGenerationReceipt,
        container: &'a [u8],
    ) -> D0Block2SourceReceipt<'a> {
        let source_index = identity.source_index();
        D0Block2SourceReceipt::synthetic_for_source_for_test(
            source_generation_receipt,
            source_generation_receipt,
            (12, 13),
            (
                source_index,
                jocforge_domain::timeline::JocGeneratedWriterIndex::new(source_index.value()),
            ),
            container,
        )
        .unwrap()
    }

    fn bind_block2_source_receipt_for_test<'a>(
        metadata: &'a jocforge_domain::joc::emdf::metadata_owner::MetadataSourceProposal,
        carried_f13_lineage: &'a D0Block2F13SourceLineage,
        expected_source_generation_receipt: &'a Flat7xD0SourceGenerationReceipt,
        f14k_source_generation_receipt: &'a Flat7xD0SourceGenerationReceipt,
        protected_container: &'a jocforge_domain::integrity::keyless_checksum::ProtectedEmdfContainer,
    ) -> Result<D0Block2SourceReceipt<'a>, D0Block2SourceReceiptError> {
        let f13_receipt = metadata.receipt();
        let au_index = f13_receipt
            .au_index()
            .ok_or(D0Block2SourceReceiptError::AccessUnitMismatch)?;
        D0Block2SourceReceipt::from_f13_f14k_source_receipts(
            metadata,
            D0Block2F14kSourceReceiptInput {
                f13_lineage: carried_f13_lineage,
                source_index: f13_receipt.source_index(),
                dispatch: f13_receipt.dispatch(),
                au_index,
                expected_source_generation_receipt,
                f14k_source_generation_receipt,
                protected_container,
            },
        )
    }

    struct SyntheticD0AudioSpans {
        audio_bytes: [Vec<u8>; 6],
        prefix_bytes: Vec<u8>,
        suffix_bytes: Vec<u8>,
        bit_lengths: [u32; 6],
        prefix_bits: u32,
    }

    impl SyntheticD0AudioSpans {
        fn new(
            target: Flat7xI0D0TargetPlan,
            bsi: &Flat7xD0BsiStructure,
            container_bytes: u16,
            frame_tail_bits: u32,
        ) -> Self {
            let bsi_bits = measure_flat7x_d0_bsi_structure(target, bsi)
                .unwrap()
                .value();
            let skipfield_bits = flat7x_d0_block2_skipfld_bit_length(container_bytes).unwrap();
            let frame_bits = u32::from(target.d0().frame_bytes()) * 8;
            let audio_bits =
                frame_bits - D0_CRC2_FIELD_BITS - bsi_bits - skipfield_bits - frame_tail_bits;
            let even_bits = audio_bits / 6;
            let remainder = audio_bits % 6;
            let bit_lengths = std::array::from_fn(|index| {
                even_bits + u32::from(u32::try_from(index).unwrap() < remainder)
            });
            let audio_bytes = std::array::from_fn(|index| {
                vec![
                    0x30 + u8::try_from(index).unwrap();
                    usize::try_from(bit_lengths[index].div_ceil(8)).unwrap()
                ]
            });
            let prefix_bits = bit_lengths[2] / 2;
            let suffix_bits = bit_lengths[2] - prefix_bits;
            let prefix_bytes = vec![0x52; usize::try_from(prefix_bits.div_ceil(8)).unwrap()];
            let suffix_bytes = vec![0x73; usize::try_from(suffix_bits.div_ceil(8)).unwrap()];

            Self {
                audio_bytes,
                prefix_bytes,
                suffix_bytes,
                bit_lengths,
                prefix_bits,
            }
        }

        fn frame_input<'a>(
            &'a self,
            skip_fields: &'a [D0Block2SkipFieldInput<'a>],
        ) -> Flat7xD0FrameBitInput<'a> {
            let audio_blocks = std::array::from_fn(|index| {
                if index == 2 {
                    D0FrameAudioBlockInput::BlockTwo(D0BlockTwoAudioSpans {
                        before_skipfield: OpaqueD0AudioBitSpan::new(
                            &self.prefix_bytes,
                            0,
                            self.prefix_bits,
                        ),
                        skip_fields,
                        after_skipfield: OpaqueD0AudioBitSpan::new(
                            &self.suffix_bytes,
                            0,
                            self.bit_lengths[2] - self.prefix_bits,
                        ),
                    })
                } else {
                    D0FrameAudioBlockInput::Opaque(OpaqueD0AudioBitSpan::new(
                        &self.audio_bytes[index],
                        0,
                        self.bit_lengths[index],
                    ))
                }
            });
            Flat7xD0FrameBitInput {
                audio_frame_controls: Flat7xD0AudioFrameControlPolicy::new(
                    Flat7xD0AllocationCandidateQ::new(1).unwrap(),
                ),
                audio_blocks,
            }
        }
    }

    fn selected_prepared_input() -> PreparedJobInput {
        let vectors: Value = serde_json::from_slice(INPUT_XML_VECTORS).unwrap();
        let case = &vectors["config_cases"][0];
        let input_xml = case["input_xml"].as_str().unwrap();
        let descriptor = &case["pcm_descriptor"];
        selected_prepared_input_with_samples(
            input_xml,
            descriptor["source_samples"].as_u64().unwrap(),
        )
    }

    fn selected_prepared_input_with_samples(
        input_xml: &str,
        source_samples: u64,
    ) -> PreparedJobInput {
        let vectors: Value = serde_json::from_slice(INPUT_XML_VECTORS).unwrap();
        let descriptor = &vectors["config_cases"][0]["pcm_descriptor"];
        let (resource, spans) = selected_wave_resource(source_samples);
        let metadata = SelectedAdmResourcePayloads::new(
            &resource[spans[0].clone()],
            &resource[spans[1].clone()],
            &resource[spans[2].clone()],
        );
        prepare_selected_job_input(
            ProgrammeId::new("flat7x-pair-integration-test").unwrap(),
            input_xml.as_bytes(),
            SelectedWaveDescriptor::new(
                u32::try_from(descriptor["sample_rate"].as_u64().unwrap()).unwrap(),
                PHYSICAL_CHANNELS,
                u16::try_from(descriptor["pcm_bits"].as_u64().unwrap()).unwrap(),
            ),
            source_samples,
            Arc::clone(&resource),
            metadata,
        )
        .unwrap()
    }

    fn selected_wave_resource(source_samples: u64) -> (Arc<[u8]>, [Range<usize>; 3]) {
        let mut bytes = b"RIFF\0\0\0\0WAVE".to_vec();
        append_wave_chunk(&mut bytes, *b"fmt ", &selected_format_payload());
        append_wave_chunk(
            &mut bytes,
            *b"data",
            &vec![
                0_u8;
                usize::try_from(source_samples).unwrap() * usize::from(PHYSICAL_CHANNELS) * 3
            ],
        );
        let axml = append_wave_chunk(&mut bytes, *b"axml", AXML);
        let chna = append_wave_chunk(&mut bytes, *b"chna", CHNA);
        let dbmd = append_wave_chunk(&mut bytes, *b"dbmd", DBMD);
        let riff_size = u32::try_from(bytes.len() - 8).unwrap();
        bytes[4..8].copy_from_slice(&riff_size.to_le_bytes());
        (Arc::from(bytes.into_boxed_slice()), [axml, chna, dbmd])
    }

    fn selected_format_payload() -> Vec<u8> {
        let block_align = PHYSICAL_CHANNELS * 3;
        let mut format = Vec::with_capacity(16);
        format.extend_from_slice(&1_u16.to_le_bytes());
        format.extend_from_slice(&PHYSICAL_CHANNELS.to_le_bytes());
        format.extend_from_slice(&48_000_u32.to_le_bytes());
        format.extend_from_slice(&(48_000_u32 * u32::from(block_align)).to_le_bytes());
        format.extend_from_slice(&block_align.to_le_bytes());
        format.extend_from_slice(&24_u16.to_le_bytes());
        format
    }

    fn append_wave_chunk(bytes: &mut Vec<u8>, chunk_id: [u8; 4], payload: &[u8]) -> Range<usize> {
        bytes.extend_from_slice(&chunk_id);
        bytes.extend_from_slice(&u32::try_from(payload.len()).unwrap().to_le_bytes());
        let start = bytes.len();
        bytes.extend_from_slice(payload);
        let end = bytes.len();
        if payload.len() % 2 != 0 {
            bytes.push(0);
        }
        start..end
    }

    fn analysis_numeric_stage(
        prepared: &PreparedJobInput,
        transaction_id: &str,
    ) -> AnalysisNumericPlanStage {
        analysis_numeric_stage_for_source(prepared, transaction_id, CodecSourceIndex::new(0))
    }

    fn analysis_numeric_stage_for_source(
        prepared: &PreparedJobInput,
        transaction_id: &str,
        source_index: CodecSourceIndex,
    ) -> AnalysisNumericPlanStage {
        let root =
            AnalysisOwnerRoot::cold_for_prepared_input(prepared.domain_identity_capability());
        let installed_audio: Box<InstalledAudioSourceFrame> =
            vec![[[0_u32; INPUT_HISTORY_BLOCK_SAMPLES]; F07_BLOCKS_PER_FRAME]; 6]
                .into_boxed_slice()
                .try_into()
                .unwrap();
        let transient = load_transient_parameters(Some(TRANSIENT_PARAMETERS)).unwrap();
        let transform =
            MathGeneratedTransformParameters::from_provider_bytes(F08_PARAMETERS).unwrap();
        // Git checks this text fixture out as CRLF on Windows; provider identity is pinned to
        // the canonical LF bytes stored in the repository blob.
        let policy_parameters = POLICY_PARAMETERS
            .iter()
            .copied()
            .filter(|byte| *byte != b'\r')
            .collect::<Vec<_>>();
        let policy =
            load_policy_parameters(PolicyProfile::selected(), Some(&policy_parameters)).unwrap();
        root.prepare_f07_source(
            AttemptId::new(2),
            CallbackTransactionId::new(transaction_id).unwrap(),
            source_index,
            &installed_audio,
            [false; F07_FULLBAND_LANES],
            &transient,
        )
        .unwrap()
        .complete_f08(&transform)
        .unwrap()
        .complete_gain_db(&policy)
        .unwrap()
        .complete_f09_f10(&policy)
        .unwrap()
    }

    fn synthetic_selected_numeric_plan(
        bap_parameters: Arc<BapParameters>,
    ) -> SelectedSourceNumericPlan {
        let zero_group = pack_exponent_difference_group(&[0, 0, 0]).unwrap();
        let exponent_plan_by_block = std::array::from_fn(|block| {
            let strategy = u8::from(block == 0);
            let fullband_groups = if strategy == 0 {
                Vec::new()
            } else {
                vec![zero_group; 72]
            };
            let fullband = std::array::from_fn(|_| FullbandExponentLanePlan {
                decoded_exponents: [15; FULLBAND_BINS],
                strategy,
                bandwidth_code: 48,
                base_exponent: 15,
                packed_groups: fullband_groups.clone(),
                gain_range: 0,
            });
            BlockExponentPlan {
                fullband,
                lfe: LfeExponentPlan {
                    decoded_exponents: [15; LFE_BINS],
                    strategy,
                    base_exponent: 15,
                    packed_groups: [zero_group; 2],
                },
            }
        });
        let coefficients_by_block = std::array::from_fn(|_| CoefficientBlockInput {
            fullband_words: std::array::from_fn(|_| [0.0_f32.to_bits(); FULLBAND_BINS]),
            lfe_words: [0.0_f32.to_bits(); LFE_BINS],
        });

        SelectedSourceNumericPlan {
            bap_provider_sha256: bap_parameters.provider_sha256(),
            bap_parameters,
            coefficients_by_block,
            exponent_plan_by_block,
            db_per_bit_codes: [2; F07_BLOCKS_PER_FRAME],
            short_flags: [[false; FULLBAND_CHANNELS]; F07_BLOCKS_PER_FRAME],
            previous_short: [false; FULLBAND_CHANNELS],
            main_exponent_syntax: MainExponentSyntax::BlockStrategies,
            converter: ConverterHeaderInput {
                frame_codes: [0; FULLBAND_CHANNELS],
                snr_offset: 0,
            },
        }
    }

    fn synthetic_i0_policy() -> I0AudioBlockSeriesPolicy {
        I0AudioBlockSeriesPolicy {
            blocks: std::array::from_fn(|block| I0AudioBlockControlPolicy {
                candidate_q: I0BapCandidateQ::new(352).unwrap(),
                slow_decay_code: 2,
                fast_decay_code: 1,
                slow_gain_code: 1,
                floor_code: 7,
                fast_gain_codes: [0; 6],
                caller_all_active_fine_zero: false,
                dither_flags: [false; 5],
                dynamic_range_code: Some(0x30 + u8::try_from(block).unwrap()),
            }),
            cplinu: CallerOwnedAc3Cplinu::Independent,
            delta_bit_allocation: Ac3DeltaBitAllocationInput::Absent,
            quantizer_policy: Ac3MantissaQuantizerPolicy::NearestTiesEven,
        }
    }

    fn synthetic_d0_bsi() -> Flat7xD0BsiStructure {
        Flat7xD0BsiStructure {
            dialnorm: 31,
            compr: Some(1),
            mixing: None,
            block_mix: [None; 6],
            information: None,
            addbsi: Flat7xD0Ci16Addbsi::new([0x01, 0x10]),
        }
    }

    fn d0_extent_for_source(
        source_index: CodecSourceIndex,
        callback_index: FrontendCallbackIndex,
    ) -> Flat7xD0PcmSourceExtent {
        let first_sample = u64::from(callback_index.value()) * 1_536;
        Flat7xD0PcmSourceExtent::new(
            source_index,
            callback_index,
            AdmSampleIndex::new(first_sample),
            1_536,
            0,
        )
        .unwrap()
    }

    #[test]
    fn pair_binding_produces_a_source_bound_d0_frame_from_its_projected_pcm_rows() {
        let prepared = selected_prepared_input();
        let analysis = analysis_numeric_stage_for_source(
            &prepared,
            "flat7x-d0-pcm-binding-source",
            CodecSourceIndex::new(2),
        );
        let bap_parameters: Arc<BapParameters> =
            Arc::new(load_bap_parameters(Some(BAP_PROVIDER)).unwrap());
        let numeric_plan = NumericPlanReceipt::from_parent_for_test(
            analysis.shared_analysis().unwrap(),
            synthetic_selected_numeric_plan(Arc::clone(&bap_parameters)),
        );
        let target = Flat7xI0D0TargetPlan::approved_contract();
        let source_identity = numeric_plan.identity();
        let callback_index = FrontendCallbackIndex::new(4);
        let callback_lineage = CallbackLineageIdentity::new(
            prepared.programme_id().clone(),
            source_identity.source_stage_key().attempt_id(),
            callback_index,
        );
        let physical_pcm_rows: Vec<Vec<u32>> = (0..usize::from(PHYSICAL_CHANNELS))
            .map(|row_index| {
                let mut row = vec![0.0_f32.to_bits(); PHYSICAL_PCM_SAMPLES];
                for block in 0..6 {
                    row[block * 256 + 13 + row_index % 23] = 0.5_f32.to_bits();
                    row[block * 256 + 129 + row_index % 17] = (-0.25_f32).to_bits();
                }
                row
            })
            .collect();
        let projection = build_flat7x_source_pcm_projection(
            &prepared,
            target,
            &callback_lineage,
            source_identity.source_index(),
            source_identity.source_stage_key().transaction_id(),
            source_identity,
            d0_extent_for_source(source_identity.source_index(), callback_index),
            &physical_pcm_rows,
        )
        .unwrap();
        let resolved_profile =
            resolve_selected_profile(&EncodingProfile::selected_with_joc_reconstruction_layout(
                JocReconstructionLayout::Flat7x,
            ))
            .unwrap();
        let binding =
            bind_flat7x_pair_source(resolved_profile, &projection, &numeric_plan).unwrap();
        let transform_parameters =
            MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
        let d0_bap_parameters = Flat7xD0BapParameters::etsi_ts_102_366_v1_4_1();
        let emdf_container = [0xa5_u8];
        let (source_generation_receipt, block2_source_receipt) =
            d0_block2_source_receipt_for_test(binding.d0_source_identity(), &emdf_container);
        let frame = produce_flat7x_pair_d0_pcm_frame_candidate(
            &binding,
            Flat7xD0PcmFrameSettings {
                source_extent: d0_extent_for_source(binding.source_index(), callback_index),
                before: &Flat7xD0OverlapHistory::cold(),
                transform_parameters: &transform_parameters,
                target,
                bsi_structure: Flat7xD0BsiStructure {
                    dialnorm: 16,
                    compr: None,
                    mixing: None,
                    block_mix: [None; 6],
                    information: None,
                    addbsi: Flat7xD0Ci16Addbsi::new([0x01, 0x10]),
                },
                source_generation_receipt: &source_generation_receipt,
                block2_source_receipt: &block2_source_receipt,
                bap_parameters: &d0_bap_parameters,
                dynamic_range_codes: [None; 6],
            },
        )
        .unwrap();

        assert_eq!(frame.d0_source_identity(), binding.d0_source_identity());
        assert_eq!(frame.source_role_map_receipt(), binding.role_map_receipt());
        assert_eq!(frame.frame_candidate().frame().frame_bytes().len(), 4_096);
        assert_eq!(
            frame.after_history().samples_by_channel()[0],
            binding.d0_rows().rows()[0][1_280..]
        );
        assert!(!frame.production_allowed());
    }

    #[test]
    fn d0_pcm_pair_stages_role_receipts_and_history_with_one_f15_candidate() {
        let prepared = selected_prepared_input();
        let source_transaction = "jocforge-source-2";
        let analysis = analysis_numeric_stage_for_source(
            &prepared,
            source_transaction,
            CodecSourceIndex::new(2),
        );
        let source_identity = analysis.shared_analysis().unwrap().identity();
        let attempt = source_identity.source_stage_key().attempt_id();
        let source_index = source_identity.source_index();
        let dispatch = source_index.dispatch().unwrap();
        let callback_index = FrontendCallbackIndex::new(4);
        let callback_lineage =
            CallbackLineageIdentity::new(prepared.programme_id().clone(), attempt, callback_index);
        let source_generation_owner =
            jocforge_domain::codec::owner::CodecOwnerRoot::cold_for_flat7x(
                jocforge_domain::profile::CodecRateProfile::Kbps1024,
            )
            .unwrap();
        let owner_issued_source_generation_receipt = source_generation_owner
            .issue_flat7x_d0_source_generation_receipt(
                source_identity.source_stage_key().transaction_id().clone(),
                callback_lineage.clone(),
                source_identity,
            )
            .unwrap();
        assert_eq!(
            owner_issued_source_generation_receipt
                .source_generation()
                .value(),
            1
        );
        assert_ne!(
            owner_issued_source_generation_receipt
                .source_generation()
                .value(),
            source_index.value()
        );
        // Model the two completed priming callbacks while keeping the independent F15 label at s=2.
        let source_generation_receipt =
            owner_issued_source_generation_receipt.with_generation_for_test(3);
        assert_eq!(source_generation_receipt.source_generation().value(), 3);
        let bap_parameters: Arc<BapParameters> =
            Arc::new(load_bap_parameters(Some(BAP_PROVIDER)).unwrap());
        let numeric_plan = NumericPlanReceipt::from_parent_for_test(
            analysis.shared_analysis().unwrap(),
            synthetic_selected_numeric_plan(Arc::clone(&bap_parameters)),
        );
        let target = Flat7xI0D0TargetPlan::approved_contract();
        let physical_pcm_rows: Vec<Vec<u32>> = (0..usize::from(PHYSICAL_CHANNELS))
            .map(|row_index| {
                let mut row = vec![0.0_f32.to_bits(); PHYSICAL_PCM_SAMPLES];
                for block in 0..6 {
                    row[block * 256 + 13 + row_index % 23] = 0.5_f32.to_bits();
                    row[block * 256 + 129 + row_index % 17] = (-0.25_f32).to_bits();
                }
                row
            })
            .collect();
        let projection = build_flat7x_source_pcm_projection(
            &prepared,
            target,
            &callback_lineage,
            source_index,
            source_identity.source_stage_key().transaction_id(),
            source_identity,
            d0_extent_for_source(source_index, callback_index),
            &physical_pcm_rows,
        )
        .unwrap();
        let resolved_profile =
            resolve_selected_profile(&EncodingProfile::selected_with_joc_reconstruction_layout(
                JocReconstructionLayout::Flat7x,
            ))
            .unwrap();
        let binding =
            bind_flat7x_pair_source(resolved_profile, &projection, &numeric_plan).unwrap();

        let mut vectors: Value = serde_json::from_slice(F15_PAIR_VECTORS).unwrap();
        let metadata_identity = &mut vectors["positive_pair"]["integrity_identity"];
        metadata_identity["programme_id"] =
            Value::String(prepared.programme_id().as_str().to_owned());
        metadata_identity["attempt"] = Value::from(attempt.value());
        metadata_identity["transaction_id"] = Value::String(source_transaction.to_owned());
        metadata_identity["source_index"] = Value::from(source_index.value());
        metadata_identity["dispatch"] = Value::from(dispatch.value());
        metadata_identity["published_au_index"] = Value::from(0_u64);
        let f15_vectors = serde_json::to_vec(&vectors).unwrap();
        let metadata_inputs = Flat7xReceiptGateMetadataTestInputs {
            f15_pair_vectors: &f15_vectors,
            metadata_owner_vectors: METADATA_OWNER_VECTORS,
            metadata_public_provider: METADATA_PUBLIC_PROVIDER,
            metadata_wrapper_config: METADATA_WRAPPER_CONFIG,
            keyless_integrity_provider: KEYLESS_INTEGRITY_PROVIDER,
        };
        let (metadata, zero_slot_container, protected_container) =
            flat7x_receipt_gate_metadata_for_test_with_payload_sequence(
                metadata_inputs,
                jocforge_domain::joc::emdf::protected_container::
                    CurrentEmdfPayloadSequence::Flat7xExperimentalJocPair,
            );
        let f13_lineage =
            jocforge_domain::codec::d0_frame_assembler::D0Block2F13SourceLineage::from_metadata_source(
                &metadata,
            )
            .unwrap();
        let payload_sequence = jocforge_domain::joc::emdf::protected_container::
            CurrentEmdfPayloadSequence::Flat7xExperimentalJocPair;
        assert_eq!(metadata.receipt().payload_sequence(), payload_sequence);
        assert_eq!(f13_lineage.payload_sequence(), payload_sequence);
        assert_eq!(
            metadata
                .parsed_container()
                .unwrap()
                .parsed_container()
                .payload_sequence(),
            payload_sequence
        );
        assert_ne!(
            f13_lineage.consumer_transaction_id(),
            f13_lineage.id14_producer_transaction_id()
        );
        let block2_receipt = bind_block2_source_receipt_for_test(
            &metadata,
            &f13_lineage,
            &source_generation_receipt,
            &source_generation_receipt,
            &protected_container,
        )
        .unwrap();
        assert_eq!(
            block2_receipt.complete_emdf_container(),
            protected_container.bytes()
        );
        assert_eq!(block2_receipt.payload_sequence(), payload_sequence);

        let mismatched_owner_generation = f13_lineage.with_owner_generations_for_test(
            f13_lineage
                .metadata_owner_before_generation()
                .wrapping_add(1),
            f13_lineage.metadata_owner_after_generation(),
        );
        assert!(matches!(
            bind_block2_source_receipt_for_test(
                &metadata,
                &mismatched_owner_generation,
                &source_generation_receipt,
                &source_generation_receipt,
                &protected_container,
            ),
            Err(D0Block2SourceReceiptError::F13F14kReceiptMismatch)
        ));
        let mismatched_id14_producer = f13_lineage.with_id14_producer_generation_for_test(
            CodecSourceIndex::new(f13_lineage.id14_producer_generation().value() + 1),
        );
        assert!(matches!(
            bind_block2_source_receipt_for_test(
                &metadata,
                &mismatched_id14_producer,
                &source_generation_receipt,
                &source_generation_receipt,
                &protected_container,
            ),
            Err(D0Block2SourceReceiptError::F13F14kReceiptMismatch)
        ));
        let mismatched_id14_writer = f13_lineage.with_id14_writer_generation_for_test(
            jocforge_domain::timeline::JocGeneratedWriterIndex::new(
                f13_lineage.id14_writer_generation().value() + 1,
            ),
        );
        assert!(matches!(
            bind_block2_source_receipt_for_test(
                &metadata,
                &mismatched_id14_writer,
                &source_generation_receipt,
                &source_generation_receipt,
                &protected_container,
            ),
            Err(D0Block2SourceReceiptError::F13F14kReceiptMismatch)
        ));
        let mismatched_analysis_generation = source_generation_receipt
            .with_analysis_generation_for_test(
                source_generation_receipt.analysis_generation().value() + 1,
            );
        assert!(matches!(
            bind_block2_source_receipt_for_test(
                &metadata,
                &f13_lineage,
                &source_generation_receipt,
                &mismatched_analysis_generation,
                &protected_container,
            ),
            Err(D0Block2SourceReceiptError::SourceGenerationMismatch)
        ));

        let i0_frame = match assemble_flat7x_i0_frame_candidate_from_binding(
            &binding,
            &numeric_plan,
            &source_generation_receipt,
            synthetic_i0_policy(),
            I0AudioBlockSeriesBudget::new(20_480),
            I0AuxDataInput::Absent,
            CallerOwnedI0CrcrsvSeed { value: false },
        ) {
            Ok(frame) => frame,
            Err(Flat7xPairSourceError::I0FrameAssembly(
                Flat7xI0FrameAssemblyError::FrameSpanMismatch {
                    actual_bits,
                    required_bits,
                },
            )) if actual_bits < required_bits => {
                let aux_bits = required_bits - actual_bits;
                let auxdata = vec![0_u8; usize::try_from(aux_bits.div_ceil(8)).unwrap()];
                assemble_flat7x_i0_frame_candidate_from_binding(
                    &binding,
                    &numeric_plan,
                    &source_generation_receipt,
                    synthetic_i0_policy(),
                    I0AudioBlockSeriesBudget::new(20_480),
                    I0AuxDataInput::Present(OpaqueI0Ac3BitSpan::new(&auxdata, 0, aux_bits)),
                    CallerOwnedI0CrcrsvSeed { value: false },
                )
                .unwrap()
            }
            other => panic!("unexpected I0 candidate result: {other:?}"),
        };

        let transform_parameters =
            MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
        let d0_bap_parameters = Flat7xD0BapParameters::etsi_ts_102_366_v1_4_1();
        let block2_source_receipt = block2_receipt;
        let d0_frame = produce_flat7x_pair_d0_pcm_frame_candidate(
            &binding,
            Flat7xD0PcmFrameSettings {
                source_extent: d0_extent_for_source(source_index, callback_index),
                before: &Flat7xD0OverlapHistory::cold(),
                transform_parameters: &transform_parameters,
                target,
                bsi_structure: synthetic_d0_bsi(),
                source_generation_receipt: &source_generation_receipt,
                block2_source_receipt: &block2_source_receipt,
                bap_parameters: &d0_bap_parameters,
                dynamic_range_codes: [None; 6],
            },
        )
        .unwrap();
        assert_eq!(
            d0_frame.block2_placement_receipt().payload_sequence(),
            payload_sequence
        );

        // SYNTHETIC/TEST_ONLY F14K receipt-gate bytes use the same protected-container/cursor
        // path as production. The F12 pair stage below also verifies those exact bytes at the
        // reported payload cursor in the assembled D0 frame.
        let d0_audio_frame = d0_frame.frame_candidate().frame();
        let d0_audio_receipt = d0_audio_frame.receipt();
        let d0_assembly = d0_audio_receipt.frame_assembly();
        let d0_layout = d0_assembly.layout();
        let block_two_receipt = d0_audio_receipt.block_receipts()[2];
        let block_two_layout = d0_layout.audio_blocks()[2];
        let block_two_prefix = block_two_layout.leading_audio_span();
        let block_two_mantissas = block_two_layout.trailing_audio_span().unwrap();
        let skip = d0_layout.block_two_skip_field();
        let skip_span = skip.bit_span();
        assert_eq!(block_two_receipt.block_index().value(), 2);
        assert_eq!(
            block_two_prefix.bit_length(),
            block_two_receipt.prefix_before_skipfield_bits()
        );
        assert_eq!(
            skip.insertion_point().offset_within_audio_span_bits(),
            block_two_prefix.bit_length()
        );
        assert_eq!(skip_span.start_bit(), block_two_prefix.end_bit());
        assert_eq!(block_two_mantissas.start_bit(), skip_span.end_bit());
        assert_eq!(
            block_two_mantissas.bit_length(),
            block_two_receipt.mantissa_bits()
        );
        assert_eq!(
            usize::from(skip.payload_bytes()),
            protected_container.bytes().len()
        );
        assert_eq!(
            d0_frame.emdf_container_start().unwrap().bit_offset(),
            skip_span.start_bit() + 10
        );

        let keyless_provider = validate_keyless_provider(KEYLESS_INTEGRITY_PROVIDER).unwrap();
        let mut integrity_owner = IntegrityProgrammeOwner::cold_from_provider_for_rate(
            prepared.programme_id().as_str(),
            attempt.value(),
            5,
            &keyless_provider,
            jocforge_domain::profile::CodecRateProfile::Kbps1024,
        )
        .unwrap();
        for suppressed_source in 0..2_u64 {
            let suppressed_index = CodecSourceIndex::new(suppressed_source);
            let suppressed_identity = IntegrityIdentity {
                programme_id: prepared.programme_id().as_str().to_owned(),
                attempt: attempt.value(),
                transaction_id: format!("jocforge-source-{suppressed_source}"),
                source_index: suppressed_index,
                dispatch: suppressed_index.dispatch().unwrap(),
                au_index: None,
            };
            let suppressed_callback = CallbackLineageIdentity::new(
                prepared.programme_id().clone(),
                attempt,
                FrontendCallbackIndex::new(u32::try_from(suppressed_source + 2).unwrap()),
            );
            let suppressed_frame = vec![0_u8; 4_096];
            integrity_owner = integrity_owner
                .stage_f12_source_for_callback(
                    IntegrityF12SourceInput {
                        identity: suppressed_identity,
                        payload: F14kPayloadInput::AbsentPublishedPayloadV1,
                        completed_frame: F12CompletedFrameInput {
                            frame: &suppressed_frame,
                            emdf_container_start: None,
                        },
                    },
                    &keyless_provider,
                    prepared.programme_id(),
                    attempt,
                    &CallbackTransactionId::new(&format!("jocforge-source-{suppressed_source}"))
                        .unwrap(),
                    &suppressed_callback,
                )
                .unwrap()
                .into_parts()
                .0;
        }

        let f14k_source = crate::integrity_source_bridge::
            published_payload_candidate_with_source_generation_for_test(
            source_index,
            dispatch,
            Some(PublishedAuIndex::new(0)),
            &keyless_provider,
            &zero_slot_container,
            &serde_json::json!({
                "bit_order": "MSB-first",
                "terminal_id": { "value": 0, "start_bit": 32, "width_bits": 5 },
                "primary_length_code": { "value": 2, "start_bit": 37, "width_bits": 2, "byte_count": 4 },
                "secondary_length_code": { "value": 1, "start_bit": 39, "width_bits": 2, "byte_count": 1 },
                "primary_protection_range_bits": [41, 73],
                "secondary_protection_range_bits": [73, 81],
                "terminal_padding_range_bits": [81, 128],
                "terminal_padding_bits": 47,
                "wire_bytes": 16,
                "body_bytes": 12
            }),
            &source_generation_receipt,
            f13_lineage.clone(),
        )
        .unwrap();
        assert_eq!(
            f14k_source.protected_container(),
            Some(&protected_container)
        );
        assert_eq!(f14k_source.payload_sequence(), Some(payload_sequence));
        let mut mismatched_i0_extent = i0_frame.clone();
        let d0_extent = d0_frame.source_extent();
        mismatched_i0_extent.source_extent = d0_extent_for_source(
            CodecSourceIndex::new(d0_extent.source_index().value() + 1),
            FrontendCallbackIndex::new(d0_extent.callback_index().value() + 1),
        );
        assert_eq!(
            stage_flat7x_pair_f12_integrity(Flat7xPairF12StageInput {
                integrity_before: &integrity_owner,
                i0_frame: &mismatched_i0_extent,
                d0_frame: &d0_frame,
                f13_metadata: &metadata,
                f14k_source: &f14k_source,
                keyless_integrity: &keyless_provider,
            })
            .unwrap_err(),
            crate::flat7x_pair_integrity::Flat7xPairF12StageError::SourceExtentMismatch
        );

        let mut mismatched_i0_frame = i0_frame.clone();
        mismatched_i0_frame.source_generation_receipt = source_generation_receipt
            .with_generation_for_test(source_generation_receipt.source_generation().value() + 1);
        assert_eq!(
            stage_flat7x_pair_f12_integrity(Flat7xPairF12StageInput {
                integrity_before: &integrity_owner,
                i0_frame: &mismatched_i0_frame,
                d0_frame: &d0_frame,
                f13_metadata: &metadata,
                f14k_source: &f14k_source,
                keyless_integrity: &keyless_provider,
            })
            .unwrap_err(),
            crate::flat7x_pair_integrity::Flat7xPairF12StageError::SourceGenerationReceiptMismatch
        );
        let f12_pair = stage_flat7x_pair_f12_integrity(Flat7xPairF12StageInput {
            integrity_before: &integrity_owner,
            i0_frame: &i0_frame,
            d0_frame: &d0_frame,
            f13_metadata: &metadata,
            f14k_source: &f14k_source,
            keyless_integrity: &keyless_provider,
        })
        .unwrap();

        let mut wrong_container_extent_d0_frame = d0_frame.clone();
        wrong_container_extent_d0_frame
            .block2_placement
            .container_bytes += 1;
        assert_eq!(
            stage_flat7x_pair_f12_integrity(Flat7xPairF12StageInput {
                integrity_before: &integrity_owner,
                i0_frame: &i0_frame,
                d0_frame: &wrong_container_extent_d0_frame,
                f13_metadata: &metadata,
                f14k_source: &f14k_source,
                keyless_integrity: &keyless_provider,
            })
            .unwrap_err(),
            crate::flat7x_pair_integrity::Flat7xPairF12StageError::D0Block2PlacementMismatch
        );

        let mut wrong_span_d0_frame = d0_frame.clone();
        wrong_span_d0_frame.block2_placement.bit_span = d0_frame
            .frame_candidate()
            .frame()
            .receipt()
            .frame_assembly()
            .layout()
            .audio_blocks()[3]
            .leading_audio_span();
        assert!(wrong_span_d0_frame.block2_placement.bit_span.bit_length() > 0);
        assert_eq!(
            stage_flat7x_pair_f12_integrity(Flat7xPairF12StageInput {
                integrity_before: &integrity_owner,
                i0_frame: &i0_frame,
                d0_frame: &wrong_span_d0_frame,
                f13_metadata: &metadata,
                f14k_source: &f14k_source,
                keyless_integrity: &keyless_provider,
            })
            .unwrap_err(),
            crate::flat7x_pair_integrity::Flat7xPairF12StageError::D0Block2PlacementMismatch
        );

        assert!(matches!(
            stage_flat7x_pcm_pair_for_f15(
                &mismatched_i0_frame,
                &d0_frame,
                Flat7xSourceGeneration::new(source_index.value()),
                &metadata,
                &f12_pair,
                Flat7xAuOutputOwner::cold(&attempt.value().to_string(), 1).unwrap(),
            ),
            Err(
                crate::flat7x_pair_output::Flat7xPairAppendProposalError::D0Block2PlacementMismatch
            )
        ));
        assert!(matches!(
            stage_flat7x_pcm_pair_for_f15(
                &i0_frame,
                &wrong_container_extent_d0_frame,
                Flat7xSourceGeneration::new(source_index.value()),
                &metadata,
                &f12_pair,
                Flat7xAuOutputOwner::cold(&attempt.value().to_string(), 1).unwrap(),
            ),
            Err(
                crate::flat7x_pair_output::Flat7xPairAppendProposalError::D0Block2PlacementMismatch
            )
        ));
        assert!(matches!(
            stage_flat7x_pcm_pair_for_f15(
                &i0_frame,
                &wrong_span_d0_frame,
                Flat7xSourceGeneration::new(source_index.value()),
                &metadata,
                &f12_pair,
                Flat7xAuOutputOwner::cold(&attempt.value().to_string(), 1).unwrap(),
            ),
            Err(
                crate::flat7x_pair_output::Flat7xPairAppendProposalError::D0Block2PlacementMismatch
            )
        ));

        let output_candidate = stage_flat7x_pcm_pair_for_f15(
            &i0_frame,
            &d0_frame,
            Flat7xSourceGeneration::new(source_index.value()),
            &metadata,
            &f12_pair,
            Flat7xAuOutputOwner::cold(&attempt.value().to_string(), 1).unwrap(),
        )
        .unwrap();

        assert_eq!(
            output_candidate
                .d0_block2_placement_receipt()
                .payload_sequence(),
            payload_sequence
        );
        assert_eq!(output_candidate.append_proposal().bytes().len(), 6_656);
        assert_eq!(
            output_candidate.source_generation().value(),
            source_index.value()
        );
        assert_eq!(
            output_candidate
                .d0_block2_placement_receipt()
                .codec_source_generation()
                .value(),
            source_generation_receipt.source_generation().value()
        );
        assert_eq!(
            output_candidate.append_proposal().source_identities().0,
            &IntegrityIdentity {
                programme_id: prepared.programme_id().as_str().to_owned(),
                attempt: attempt.value(),
                transaction_id: source_transaction.to_owned(),
                source_index,
                dispatch,
                au_index: Some(PublishedAuIndex::new(0)),
            }
        );
        assert_eq!(output_candidate.after_history(), d0_frame.after_history());
        assert_eq!(f12_pair.owner_receipt_proposal().after_generation(), 3);
        assert_eq!(
            output_candidate
                .integrity_after_owner()
                .committed_state()
                .generation(),
            3
        );
        assert_eq!(
            output_candidate
                .integrity_owner_receipt()
                .after_generation(),
            3
        );
        assert!(!output_candidate.production_allowed());
    }

    #[test]
    fn application_pair_candidate_composes_matching_i0_d0_frames_in_one_au() {
        let prepared = selected_prepared_input();
        let analysis = analysis_numeric_stage_for_source(
            &prepared,
            "flat7x-pair-integration-source",
            CodecSourceIndex::new(2),
        );
        let bap_parameters: Arc<BapParameters> =
            Arc::new(load_bap_parameters(Some(BAP_PROVIDER)).unwrap());
        // Test support binds synthetic codec values to the exact prepared-source analysis receipt.
        let numeric_plan = NumericPlanReceipt::from_parent_for_test(
            analysis.shared_analysis().unwrap(),
            synthetic_selected_numeric_plan(Arc::clone(&bap_parameters)),
        );

        let target = Flat7xI0D0TargetPlan::approved_contract();
        let source_identity = numeric_plan.identity();
        let callback_index = FrontendCallbackIndex::new(4);
        let callback_lineage = CallbackLineageIdentity::new(
            prepared.programme_id().clone(),
            source_identity.source_stage_key().attempt_id(),
            callback_index,
        );
        let physical_pcm_rows =
            vec![vec![0_u32; PHYSICAL_PCM_SAMPLES]; usize::from(PHYSICAL_CHANNELS)];
        let projection = build_flat7x_source_pcm_projection(
            &prepared,
            target,
            &callback_lineage,
            source_identity.source_index(),
            source_identity.source_stage_key().transaction_id(),
            source_identity,
            d0_extent_for_source(source_identity.source_index(), callback_index),
            &physical_pcm_rows,
        )
        .unwrap();
        let resolved_profile =
            resolve_selected_profile(&EncodingProfile::selected_with_joc_reconstruction_layout(
                JocReconstructionLayout::Flat7x,
            ))
            .unwrap();
        let binding =
            bind_flat7x_pair_source(resolved_profile, &projection, &numeric_plan).unwrap();

        let d0_bsi = synthetic_d0_bsi();
        let emdf_container = [0xa5_u8];
        let d0_audio = SyntheticD0AudioSpans::new(target, &d0_bsi, 1, 85);
        let d0_skip_fields = [D0Block2SkipFieldInput::synthetic_for_test(
            CallerOwnedUnboundD0Block2InsertionPoint::new(d0_audio.prefix_bits),
            &emdf_container,
        )
        .unwrap()];
        let d0_audio_blocks = d0_audio.frame_input(&d0_skip_fields);
        let d0_receipt = Flat7xD0AudioSpanReceipt::from_source_bound_producer(
            &binding,
            d0_audio_blocks,
            Flat7xD0FrameTailPolicy::ZeroFillUnusedAuxBits,
        );
        let pair_audio = prepare_flat7x_pair_audio_blocks(
            binding,
            &numeric_plan,
            synthetic_i0_policy(),
            I0AudioBlockSeriesBudget::new(20_480),
            Flat7xD0AudioSpanInput::Provided(Box::new(d0_receipt)),
        )
        .unwrap();
        let pair_identity = pair_audio.source_binding().d0_source_identity();
        let i0_source_generation_receipt = Flat7xD0SourceGenerationReceipt::synthetic_for_test(
            pair_identity.source_transaction_id().clone(),
            3,
            callback_lineage.clone(),
            source_identity.source_index(),
            pair_identity.analysis_generation().value(),
        );
        let i0_frame = match assemble_flat7x_pair_i0_frame_candidate(
            &pair_audio,
            &i0_source_generation_receipt,
            I0AuxDataInput::Absent,
            CallerOwnedI0CrcrsvSeed { value: false },
        ) {
            Ok(candidate) => candidate,
            Err(Flat7xPairSourceError::I0FrameAssembly(
                Flat7xI0FrameAssemblyError::FrameSpanMismatch {
                    actual_bits,
                    required_bits,
                },
            )) if actual_bits < required_bits => {
                let auxdata_bits = required_bits - actual_bits;
                let auxdata = vec![0_u8; usize::try_from(auxdata_bits.div_ceil(8)).unwrap()];
                assemble_flat7x_pair_i0_frame_candidate(
                    &pair_audio,
                    &i0_source_generation_receipt,
                    I0AuxDataInput::Present(OpaqueI0Ac3BitSpan::new(&auxdata, 0, auxdata_bits)),
                    CallerOwnedI0CrcrsvSeed { value: false },
                )
                .unwrap()
            }
            other => panic!("unexpected I0 frame candidate result: {other:?}"),
        };

        let candidate =
            assemble_flat7x_pair_access_unit_candidate(&pair_audio, &i0_frame, d0_bsi).unwrap();

        assert_eq!(candidate.source_identity(), numeric_plan.identity());
        assert_eq!(
            candidate.source_role_map_receipt(),
            pair_audio.role_map_receipt()
        );
        assert_eq!(
            candidate.d0_source_identity(),
            pair_audio.source_binding().d0_source_identity()
        );
        assert_eq!(
            i0_frame.source_extent(),
            pair_audio.source_binding().source_extent()
        );
        assert_eq!(
            candidate.source_extent(),
            pair_audio.source_binding().source_extent()
        );
        let mut mismatched_i0_extent = i0_frame.clone();
        let source_extent = i0_frame.source_extent();
        mismatched_i0_extent.source_extent = d0_extent_for_source(
            CodecSourceIndex::new(source_extent.source_index().value() + 1),
            FrontendCallbackIndex::new(source_extent.callback_index().value() + 1),
        );
        assert_eq!(
            assemble_flat7x_pair_access_unit_candidate(
                &pair_audio,
                &mismatched_i0_extent,
                synthetic_d0_bsi(),
            )
            .unwrap_err(),
            Flat7xPairSourceError::SourceExtentMismatch
        );
        assert_eq!(candidate.access_unit_bytes().len(), 6_656);
        assert_eq!(candidate.access_unit().d0_frame_bytes().len(), 4_096);
        let d0_receipt = candidate.access_unit().d0_frame_receipt();
        assert_eq!(d0_receipt.target_frame_bytes(), 4_096);
        assert_eq!(d0_receipt.frame_auxbits_span().bit_length(), 6);
        assert_eq!(
            d0_receipt.frame_tail_policy(),
            Some(Flat7xD0FrameTailPolicy::ZeroFillUnusedAuxBits)
        );
        assert_eq!(d0_receipt.auxdatae_value(), Some(false));
        assert_eq!(d0_receipt.zero_filled_auxbits(), 6);
        assert_eq!(
            d0_receipt.crc2_span().start_bit(),
            d0_receipt.frame_auxbits_span().end_bit() + 2
        );
        assert_eq!(d0_receipt.frame_auxbits_span().start_bit() % 8, 0);
        let tail_start = usize::try_from(d0_receipt.frame_auxbits_span().start_bit() / 8).unwrap();
        assert_eq!(
            &candidate.access_unit_bytes()[2_560 + tail_start..2_560 + tail_start + 1],
            &[0]
        );
        assert_eq!(
            d0_receipt.layout().audio_blocks()[2]
                .block_index()
                .zero_based_value(),
            2
        );
        assert_eq!(
            d0_receipt.layout().block_two_skip_field().payload_bytes(),
            1
        );
        let au_receipt = candidate.access_unit().access_unit_receipt();
        assert_eq!(au_receipt.i0_span().start_byte(), 0);
        assert_eq!(au_receipt.i0_span().byte_length(), 2_560);
        assert_eq!(au_receipt.d0_span().start_byte(), 2_560);
        assert_eq!(au_receipt.d0_span().byte_length(), 4_096);
        assert_eq!(au_receipt.total_byte_length(), 6_656);
        assert_eq!(
            &candidate.access_unit_bytes()[..2_560],
            i0_frame.i0_frame().frame_bytes()
        );
        assert!(!candidate.production_allowed());

        let output_owner = Flat7xAuOutputOwner::cold("flat7x-pair-integration", 1).unwrap();
        let source_generation = Flat7xSourceGeneration::new(0)
            .checked_next()
            .expect("the local source-generation counter advances");
        let stage_receipt = preflight_flat7x_pair_io_stage(
            &candidate,
            numeric_plan.identity(),
            source_generation,
            output_owner,
        )
        .unwrap();
        assert_eq!(stage_receipt.source_identity(), numeric_plan.identity());
        assert_eq!(stage_receipt.i0_frame_bytes(), 2_560);
        assert_eq!(stage_receipt.d0_frame_bytes(), 4_096);
        assert_eq!(stage_receipt.access_unit_bytes(), 6_656);
        assert_eq!(stage_receipt.d0_skipfield_block(), 2);
        assert_eq!(stage_receipt.d0_skipfield_bytes(), 1);
        assert_eq!(
            stage_receipt.disposition(),
            Flat7xPairIoStageDisposition::RejectedProductionDisabled
        );
        assert!(!stage_receipt.production_allowed());
        assert!(!stage_receipt.append_proposal_created());
        assert!(!stage_receipt.io_attempted());
        assert_eq!(
            stage_receipt.retirement().reason(),
            Flat7xAuAttemptRetirementReason::StageRejected(
                Flat7xAuOutputOwnerError::F13MetadataReceiptMissing
            )
        );
        assert!(stage_receipt.retirement().committed_owner_unchanged());
        assert!(stage_receipt.retirement().fresh_programme_cold_required());
        assert_eq!(stage_receipt.retirement().next_access_unit().value(), 0);
        assert_eq!(stage_receipt.retirement().durable_length(), 0);

        let mismatched_analysis = analysis_numeric_stage(&prepared, "different-source-stage");
        assert_eq!(
            preflight_flat7x_pair_io_stage(
                &candidate,
                mismatched_analysis.shared_analysis().unwrap().identity(),
                source_generation,
                Flat7xAuOutputOwner::cold("mismatched-source", 1).unwrap(),
            ),
            Err(Flat7xPairIoStageError::SourceIdentityMismatch)
        );
    }

    #[test]
    fn receipted_i0_and_opaque_d0_frame_stage_one_same_source_f15_pair_proposal() {
        let config_vectors: Value = serde_json::from_slice(INPUT_XML_VECTORS).unwrap();
        let input_xml = config_vectors["config_cases"][0]["input_xml"]
            .as_str()
            .unwrap();
        let prepared = selected_prepared_input_with_samples(input_xml, 4_608);
        let source_index = CodecSourceIndex::new(2);
        let analysis = analysis_numeric_stage_for_source(
            &prepared,
            "flat7x-pair-integration-source",
            source_index,
        );
        let bap_parameters: Arc<BapParameters> =
            Arc::new(load_bap_parameters(Some(BAP_PROVIDER)).unwrap());
        let numeric_plan = NumericPlanReceipt::from_parent_for_test(
            analysis.shared_analysis().unwrap(),
            synthetic_selected_numeric_plan(Arc::clone(&bap_parameters)),
        );

        let target = Flat7xI0D0TargetPlan::approved_contract();
        let shared_identity = numeric_plan.identity();
        let callback_index = FrontendCallbackIndex::new(4);
        let callback_lineage = CallbackLineageIdentity::new(
            prepared.programme_id().clone(),
            shared_identity.source_stage_key().attempt_id(),
            callback_index,
        );
        let physical_pcm_rows =
            vec![vec![0_u32; PHYSICAL_PCM_SAMPLES]; usize::from(PHYSICAL_CHANNELS)];
        let projection = build_flat7x_source_pcm_projection(
            &prepared,
            target,
            &callback_lineage,
            source_index,
            shared_identity.source_stage_key().transaction_id(),
            shared_identity,
            d0_extent_for_source(source_index, callback_index),
            &physical_pcm_rows,
        )
        .unwrap();
        let resolved_profile =
            resolve_selected_profile(&EncodingProfile::selected_with_joc_reconstruction_layout(
                JocReconstructionLayout::Flat7x,
            ))
            .unwrap();
        let binding =
            bind_flat7x_pair_source(resolved_profile, &projection, &numeric_plan).unwrap();
        let d0_source_identity = binding.d0_source_identity().clone();
        assert_eq!(binding.d0_rows().rows().len(), 4);
        let i0_source_generation_receipt = Flat7xD0SourceGenerationReceipt::synthetic_for_test(
            d0_source_identity.source_transaction_id().clone(),
            3,
            callback_lineage.clone(),
            source_index,
            d0_source_identity.analysis_generation().value(),
        );
        let i0_frame = match assemble_flat7x_i0_frame_candidate_from_binding(
            &binding,
            &numeric_plan,
            &i0_source_generation_receipt,
            synthetic_i0_policy(),
            I0AudioBlockSeriesBudget::new(20_480),
            I0AuxDataInput::Absent,
            CallerOwnedI0CrcrsvSeed { value: false },
        ) {
            Ok(candidate) => candidate,
            Err(Flat7xPairSourceError::I0FrameAssembly(
                Flat7xI0FrameAssemblyError::FrameSpanMismatch {
                    actual_bits,
                    required_bits,
                },
            )) if actual_bits < required_bits => {
                let auxdata_bits = required_bits - actual_bits;
                let auxdata = vec![0_u8; usize::try_from(auxdata_bits.div_ceil(8)).unwrap()];
                let binding =
                    bind_flat7x_pair_source(resolved_profile, &projection, &numeric_plan).unwrap();
                assemble_flat7x_i0_frame_candidate_from_binding(
                    &binding,
                    &numeric_plan,
                    &i0_source_generation_receipt,
                    synthetic_i0_policy(),
                    I0AudioBlockSeriesBudget::new(20_480),
                    I0AuxDataInput::Present(OpaqueI0Ac3BitSpan::new(&auxdata, 0, auxdata_bits)),
                    CallerOwnedI0CrcrsvSeed { value: false },
                )
                .unwrap()
            }
            other => panic!("unexpected source-bound I0 frame result: {other:?}"),
        };

        let receipt_inputs = Flat7xReceiptGateMetadataTestInputs {
            f15_pair_vectors: F15_PAIR_VECTORS,
            metadata_owner_vectors: METADATA_OWNER_VECTORS,
            metadata_public_provider: METADATA_PUBLIC_PROVIDER,
            metadata_wrapper_config: METADATA_WRAPPER_CONFIG,
            keyless_integrity_provider: KEYLESS_INTEGRITY_PROVIDER,
        };
        let existing_pair = Flat7xAuAppendTestFixture::positive_pair(receipt_inputs);
        let existing_pair_bytes = existing_pair.expected_bytes();
        let opaque_d0_frame = &existing_pair_bytes[2_560..];
        let expected_au = match publication_for_source(source_index).unwrap() {
            PublicationDisposition::Published(consumption) => consumption.au,
            PublicationDisposition::Suppressed { .. } => panic!("source 2 publishes AU 0"),
        };
        let source_stage = i0_frame.source_identity().source_stage_key();
        // The pair-source owner supplies this generation independently of analysis identity,
        // callback order, or any DEE observation.
        let source_generation = Flat7xSourceGeneration::new(source_index.value());
        let integrity_identity = IntegrityIdentity {
            programme_id: prepared.programme_id().as_str().to_owned(),
            attempt: source_stage.attempt_id().value(),
            transaction_id: source_stage.transaction_id().as_str().to_owned(),
            source_index,
            dispatch: source_index.dispatch().unwrap(),
            au_index: Some(expected_au),
        };
        let receipts = Flat7xAuAppendTestFixture::with_opaque_frames(
            receipt_inputs,
            integrity_identity,
            source_generation,
            i0_frame.i0_frame().frame_bytes(),
            opaque_d0_frame,
        );
        let skip_target = target.d0_emdf_skip_field();
        let d0_input = Flat7xOpaqueD0FrameInput::new(
            &d0_source_identity,
            source_generation,
            opaque_d0_frame,
            D0EmdfSkipFieldExtent::new(
                source_generation,
                skip_target.zero_based_audio_block(),
                skip_target.field_count(),
                Some(receipts.d0_emdf_container_bytes()),
                skip_target.fragmentation_allowed(),
            ),
        );
        let receipt_input = Flat7xAuSourceReceiptInput::new(
            Some(receipts.metadata_source_proposal()),
            Some(Flat7xAuIntegrityFrameReceipt::independent_i0(
                receipts.i0_integrity_receipts(),
            )),
            Some(Flat7xAuIntegrityFrameReceipt::dependent_d0(
                receipts.d0_integrity_receipts(),
            )),
            Some(i0_frame.source_generation_receipt()),
        );

        let wrong_generation = Flat7xOpaqueD0FrameInput::new(
            &d0_source_identity,
            Flat7xSourceGeneration::new(source_generation.value() + 1),
            opaque_d0_frame,
            D0EmdfSkipFieldExtent::new(
                Flat7xSourceGeneration::new(source_generation.value() + 1),
                skip_target.zero_based_audio_block(),
                skip_target.field_count(),
                Some(receipts.d0_emdf_container_bytes()),
                skip_target.fragmentation_allowed(),
            ),
        );
        assert!(matches!(
            stage_receipted_i0_with_opaque_d0(
                &i0_frame,
                source_generation,
                wrong_generation,
                Flat7xAuOutputOwner::cold(receipts.attempt_id(), 1).unwrap(),
                receipt_input,
            ),
            Err(Flat7xPairAppendProposalError::SourceGenerationMismatch)
        ));

        let mismatched_codec_source_generation_receipt = i0_frame
            .source_generation_receipt()
            .with_generation_for_test(99);
        let mismatched_receipt_input = Flat7xAuSourceReceiptInput::new(
            Some(receipts.metadata_source_proposal()),
            Some(Flat7xAuIntegrityFrameReceipt::independent_i0(
                receipts.i0_integrity_receipts(),
            )),
            Some(Flat7xAuIntegrityFrameReceipt::dependent_d0(
                receipts.d0_integrity_receipts(),
            )),
            Some(&mismatched_codec_source_generation_receipt),
        );
        assert!(matches!(
            stage_receipted_i0_with_opaque_d0(
                &i0_frame,
                source_generation,
                d0_input,
                Flat7xAuOutputOwner::cold(receipts.attempt_id(), 1).unwrap(),
                mismatched_receipt_input,
            ),
            Err(Flat7xPairAppendProposalError::CodecSourceGenerationReceiptMismatch)
        ));

        let proposal = stage_receipted_i0_with_opaque_d0(
            &i0_frame,
            source_generation,
            d0_input,
            Flat7xAuOutputOwner::cold(receipts.attempt_id(), 1).unwrap(),
            receipt_input,
        )
        .unwrap();

        assert_eq!(proposal.bytes().len(), 6_656);
        assert_eq!(
            &proposal.bytes()[..2_560],
            i0_frame.i0_frame().frame_bytes()
        );
        assert_eq!(&proposal.bytes()[2_560..], opaque_d0_frame);
        assert_eq!(proposal.ledger_entry().frame_lengths(), (2_560, 4_096));
        assert_eq!(
            proposal.ledger_entry().source_generation(),
            source_generation
        );
        assert_eq!(
            proposal.ledger_entry().codec_source_generation_receipt(),
            i0_frame.source_generation_receipt()
        );
        assert_eq!(
            proposal
                .ledger_entry()
                .codec_source_generation_receipt()
                .source_index(),
            source_index
        );
        assert_eq!(
            proposal
                .ledger_entry()
                .codec_source_generation_receipt()
                .codec_owner_after_generation()
                .value(),
            3
        );
        assert_eq!(
            proposal
                .ledger_entry()
                .codec_source_generation_receipt()
                .source_generation()
                .value(),
            3
        );
        assert_eq!(
            proposal.ledger_entry().metadata_identity(),
            (source_index, source_index.dispatch().unwrap(), expected_au)
        );
        assert!(!proposal.production_allowed());
    }
}

#[cfg(test)]
mod i0_audio_policy_tests {
    use super::{I0BapCandidateQ, flat7x_i0_audio_block_policy_v1};

    #[test]
    fn flat7x_i0_policy_uses_the_selected_q_for_all_live_audio_blocks() {
        let q = I0BapCandidateQ::new(400).unwrap();
        let policy = flat7x_i0_audio_block_policy_v1(q).unwrap();

        assert!(policy.blocks.iter().all(|block| block.candidate_q == q));
        assert!(
            policy
                .blocks
                .iter()
                .all(|block| !block.caller_all_active_fine_zero)
        );
    }
}
