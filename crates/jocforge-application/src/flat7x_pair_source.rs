// pattern: Functional Core
//! Joins the selected Flat7x source projection to the existing I0/D0 audio-frame seams.

use jocforge_domain::{
    codec::{
        ac3_audio_block::{
            I0AudioBlockSeriesBudget, I0AudioBlockSeriesCandidate, I0AudioBlockSeriesError,
            I0AudioBlockSeriesPolicy, produce_i0_audio_block_series,
        },
        access_unit_composer::{
            Flat7xI0D0AuCandidate, Flat7xI0D0AuCandidateError, assemble_flat7x_i0_d0_au_candidate,
        },
        d0_frame_assembler::Flat7xD0FrameBitInput,
        frame_builder::Flat7xD0BsiStructure,
        i0_frame_assembler::{
            CallerOwnedI0CrcrsvSeed, Flat7xI0Ac3AudioSeriesFrameCandidate,
            Flat7xI0FrameAssemblyError, I0AuxDataInput,
            assemble_flat7x_i0_ac3_frame_from_audio_series,
        },
        selected_frame_trial::SelectedSourceNumericPlan,
    },
    profile::{
        CodecRateProfile, Flat7xD0PcmRows, Flat7xI0D0TargetPlan, Flat7xI0PcmRows,
        JocReconstructionLayout, ResolvedEncodingProfile,
    },
    timeline::{CodecSourceIndex, FrontendCallbackIndex},
    transaction::{AttemptId, CallbackTransactionId, ProgrammeId},
    transform::fullband_exponent_pipeline::NumericPlanReceipt,
};

use crate::source_frame_bridge::{Flat7xSourcePcmProjection, Flat7xSourceRoleMapReceipt};
use jocforge_domain::analysis::source_frame::SharedAnalysisIdentity;

/// Current D0 source status at the replaceable audio-span producer boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0AudioSpanProducerState {
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

/// Immutable source-bound D0 audio spans. Its constructor stays inside this module until a
/// Wash-approved D0 producer can issue the receipt; tests use explicit fixture construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xD0AudioSpanReceipt<'a> {
    source_identity: Flat7xD0SourceIdentity,
    audio_blocks: Flat7xD0FrameBitInput<'a>,
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
    Provided(Flat7xD0AudioSpanReceipt<'a>),
}

/// Errors while binding the profile and current source projection to the I0/D0 frame inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xPairSourceError {
    UnsupportedProfile,
    ProfileTargetMismatch,
    SourceIdentityMismatch,
    D0AudioSpanProducerDisabled(Flat7xD0AudioSpanProducerState),
    D0AudioSpanSourceIdentityMismatch,
    I0AudioBlockSeries(I0AudioBlockSeriesError),
    I0FrameAssembly(Flat7xI0FrameAssemblyError),
    AccessUnitAssembly(Flat7xI0D0AuCandidateError),
}

/// Resolves the typed D0 producer boundary before a pair candidate can be created.
pub fn require_flat7x_d0_audio_blocks<'a>(
    input: Flat7xD0AudioSpanInput<'a>,
    expected_source_identity: &Flat7xD0SourceIdentity,
) -> Result<Flat7xD0FrameBitInput<'a>, Flat7xPairSourceError> {
    match input {
        Flat7xD0AudioSpanInput::Disabled(reason) => {
            Err(Flat7xPairSourceError::D0AudioSpanProducerDisabled(reason))
        }
        Flat7xD0AudioSpanInput::Provided(receipt) => {
            if receipt.source_identity != *expected_source_identity {
                return Err(Flat7xPairSourceError::D0AudioSpanSourceIdentityMismatch);
            }
            Ok(receipt.audio_blocks)
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
    resolved_profile: ResolvedEncodingProfile,
    target: Flat7xI0D0TargetPlan,
    source_index: CodecSourceIndex,
    role_map_receipt: Flat7xSourceRoleMapReceipt,
    d0_source_identity: Flat7xD0SourceIdentity,
    i0_rows: Flat7xI0PcmRows<'pcm>,
    d0_rows: Flat7xD0PcmRows<'pcm>,
}

impl Flat7xPairSourceBinding<'_> {
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
            && self.i0_rows == projection.i0_rows()
            && self.d0_rows == projection.d0_rows()
    }
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
    auxdata: I0AuxDataInput<'_>,
    crcrsv_seed: CallerOwnedI0CrcrsvSeed,
) -> Result<Flat7xPairI0FrameCandidate, Flat7xPairSourceError> {
    let binding = &pair_audio.binding;
    if binding.target != binding.role_map_receipt.target()
        || binding.resolved_profile.joc_reconstruction_layout() != JocReconstructionLayout::Flat7x
        || binding.resolved_profile.codec_rate() != CodecRateProfile::Kbps1024
        || binding.source_index != binding.role_map_receipt.source_index()
        || binding.source_index
            != pair_audio
                .i0_audio_blocks
                .receipt()
                .source_identity()
                .source_index()
    {
        return Err(Flat7xPairSourceError::SourceIdentityMismatch);
    }

    let i0_frame = assemble_flat7x_i0_ac3_frame_from_audio_series(
        binding.target,
        binding.role_map_receipt.shared_analysis_identity(),
        &pair_audio.i0_audio_blocks,
        auxdata,
        crcrsv_seed,
    )
    .map_err(Flat7xPairSourceError::I0FrameAssembly)?;

    Ok(Flat7xPairI0FrameCandidate {
        resolved_profile: binding.resolved_profile,
        source_role_map_receipt: binding.role_map_receipt.clone(),
        i0_frame,
    })
}

/// Complete in-memory I0→D0 AU candidate with the selected profile and both source receipts.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xPairAccessUnitCandidate {
    resolved_profile: ResolvedEncodingProfile,
    source_role_map_receipt: Flat7xSourceRoleMapReceipt,
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

/// Builds the paired I0→D0 byte candidate from the same profile, source receipt and D0 input.
///
/// The D0 BSI and six opaque audio spans remain caller-owned. The existing D0 assembler validates
/// its target, six block positions and single block-index-2 skip-field before the I0→D0 composer
/// emits one in-memory AU candidate. This function performs no CLI, AU-owner, file or publication
/// call.
pub fn assemble_flat7x_pair_access_unit_candidate(
    pair_audio: &Flat7xPairAudioBlockCandidate<'_, '_>,
    i0_frame: &Flat7xPairI0FrameCandidate,
    d0_bsi: Flat7xD0BsiStructure<'_>,
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

    let access_unit = assemble_flat7x_i0_d0_au_candidate(
        binding.target,
        shared_identity,
        i0_frame.i0_frame.clone(),
        d0_bsi,
        pair_audio.d0_audio_blocks,
    )
    .map_err(Flat7xPairSourceError::AccessUnitAssembly)?;

    Ok(Flat7xPairAccessUnitCandidate {
        resolved_profile: binding.resolved_profile,
        source_role_map_receipt: role_map.clone(),
        d0_source_identity: binding.d0_source_identity.clone(),
        access_unit,
    })
}

fn d0_identity_matches_role_map(
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
        resolved_profile: profile,
        target,
        source_index: numeric_plan.identity().source_index(),
        role_map_receipt: role_map_receipt.clone(),
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
    let d0_audio_blocks = require_flat7x_d0_audio_blocks(d0_audio, binding.d0_source_identity())?;
    let i0_audio_blocks = produce_i0_audio_block_series(numeric_plan, i0_policy, i0_budget)
        .map_err(Flat7xPairSourceError::I0AudioBlockSeries)?;

    Ok(Flat7xPairAudioBlockCandidate {
        binding,
        i0_audio_blocks,
        d0_audio_blocks,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        Flat7xAnalysisGeneration, Flat7xD0AudioSpanInput, Flat7xD0AudioSpanProducerState,
        Flat7xD0AudioSpanReceipt, Flat7xD0SourceIdentity, Flat7xPairSourceError,
        require_flat7x_d0_audio_blocks,
    };
    use jocforge_domain::codec::d0_frame_assembler::{
        D0FrameAudioBlockInput, Flat7xD0FrameBitInput, OpaqueD0AudioBitSpan,
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
                audio_blocks: [block; 6],
            },
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
        let provided =
            Flat7xD0AudioSpanInput::Provided(fixture_span_receipt(expected.clone(), &byte));
        assert!(require_flat7x_d0_audio_blocks(provided, &expected).is_ok());
    }

    #[test]
    fn d0_audio_span_receipt_rejects_source_generation_mismatch() {
        let byte = [0_u8];
        let expected = fixture_source_identity(9);
        let receipt = fixture_span_receipt(fixture_source_identity(8), &byte);

        assert_eq!(
            require_flat7x_d0_audio_blocks(Flat7xD0AudioSpanInput::Provided(receipt), &expected,),
            Err(Flat7xPairSourceError::D0AudioSpanSourceIdentityMismatch)
        );
    }
}
