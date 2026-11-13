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

#[cfg(all(test, feature = "test-support"))]
mod pair_access_unit_integration_tests {
    use std::{ops::Range, sync::Arc};

    use super::{
        Flat7xD0AudioSpanInput, Flat7xD0AudioSpanReceipt, Flat7xPairSourceError,
        assemble_flat7x_pair_access_unit_candidate, assemble_flat7x_pair_i0_frame_candidate,
        bind_flat7x_pair_source, prepare_flat7x_pair_audio_blocks,
    };
    use crate::{
        PreparedJobInput, SelectedAdmResourcePayloads, SelectedWaveDescriptor,
        flat7x_pair_output::{
            Flat7xPairIoStageDisposition, Flat7xPairIoStageError, preflight_flat7x_pair_io_stage,
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
            bap::{BapParameters, load_bap_parameters},
            d0_frame_assembler::{
                D0_CRC2_FIELD_BITS, D0Block2SkipFieldInput, D0BlockTwoAudioSpans,
                D0FrameAudioBlockInput, Flat7xD0FrameBitInput, OpaqueD0AudioBitSpan,
            },
            d0_frame_layout::CallerOwnedUnboundD0Block2InsertionPoint,
            emdf_skip_field::flat7x_d0_block2_skipfld_bit_length,
            exponent_packing::pack_exponent_difference_group,
            frame_builder::{Flat7xD0BsiStructure, measure_flat7x_d0_bsi_structure},
            i0_frame_assembler::{
                CallerOwnedAc3Cplinu, CallerOwnedI0CrcrsvSeed, Flat7xI0FrameAssemblyError,
                I0AuxDataInput, OpaqueI0Ac3BitSpan,
            },
            policy_parameters::{PolicyProfile, load_policy_parameters},
            selected_frame_trial::{
                BlockExponentPlan, CoefficientBlockInput, ConverterHeaderInput,
                FullbandExponentLanePlan, LfeExponentPlan, MainExponentSyntax,
                SelectedSourceNumericPlan,
            },
        },
        profile::{
            EncodingProfile, Flat7xI0D0TargetPlan, JocReconstructionLayout,
            resolve_selected_profile,
        },
        publication::{
            Flat7xAuAttemptRetirementReason, Flat7xAuOutputOwner, Flat7xAuOutputOwnerError,
        },
        timeline::{CodecSourceIndex, FrontendCallbackIndex},
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
    const PHYSICAL_CHANNELS: u16 = 38;
    const PHYSICAL_PCM_SAMPLES: usize = 1_536;
    const FULLBAND_CHANNELS: usize = 5;
    const FULLBAND_BINS: usize = 217;
    const LFE_BINS: usize = 7;

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
            bsi: &Flat7xD0BsiStructure<'_>,
            container_bytes: u16,
        ) -> Self {
            let bsi_bits = measure_flat7x_d0_bsi_structure(target, bsi)
                .unwrap()
                .value();
            let skipfield_bits = flat7x_d0_block2_skipfld_bit_length(container_bytes).unwrap();
            let frame_bits = u32::from(target.d0().frame_bytes()) * 8;
            let audio_bits = frame_bits - D0_CRC2_FIELD_BITS - bsi_bits - skipfield_bits;
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
            Flat7xD0FrameBitInput { audio_blocks }
        }
    }

    fn selected_prepared_input() -> PreparedJobInput {
        let vectors: Value = serde_json::from_slice(INPUT_XML_VECTORS).unwrap();
        let case = &vectors["config_cases"][0];
        let input_xml = case["input_xml"].as_str().unwrap();
        let descriptor = &case["pcm_descriptor"];
        let (resource, spans) = selected_wave_resource();
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
            1,
            Arc::clone(&resource),
            metadata,
        )
        .unwrap()
    }

    fn selected_wave_resource() -> (Arc<[u8]>, [Range<usize>; 3]) {
        let mut bytes = b"RIFF\0\0\0\0WAVE".to_vec();
        append_wave_chunk(&mut bytes, *b"fmt ", &selected_format_payload());
        append_wave_chunk(
            &mut bytes,
            *b"data",
            &vec![0_u8; usize::from(PHYSICAL_CHANNELS) * 3],
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
        let policy =
            load_policy_parameters(PolicyProfile::selected(), Some(POLICY_PARAMETERS)).unwrap();
        root.prepare_f07_source(
            AttemptId::new(2),
            CallbackTransactionId::new(transaction_id).unwrap(),
            CodecSourceIndex::new(0),
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
                candidate_q: I0BapCandidateQ::new(if block == 0 { 16 } else { 0 }).unwrap(),
                slow_decay_code: 2,
                fast_decay_code: 1,
                slow_gain_code: 1,
                floor_code: 7,
                fast_gain_codes: [0; 6],
                caller_all_active_fine_zero: block != 0,
                dither_flags: [false; 5],
                dynamic_range_code: Some(0x30 + u8::try_from(block).unwrap()),
            }),
            cplinu: CallerOwnedAc3Cplinu::Independent,
            delta_bit_allocation: Ac3DeltaBitAllocationInput::Absent,
            quantizer_policy: Ac3MantissaQuantizerPolicy::NearestTiesEven,
        }
    }

    fn synthetic_d0_bsi() -> Flat7xD0BsiStructure<'static> {
        Flat7xD0BsiStructure {
            dialnorm: 31,
            compr: Some(1),
            mixing: None,
            block_mix: [None; 6],
            information: None,
            addbsi: None,
        }
    }

    #[test]
    fn application_pair_candidate_composes_matching_i0_d0_frames_in_one_au() {
        let prepared = selected_prepared_input();
        let analysis = analysis_numeric_stage(&prepared, "flat7x-pair-integration-source");
        let bap_parameters: Arc<BapParameters> =
            Arc::new(load_bap_parameters(Some(BAP_PROVIDER)).unwrap());
        // Test support binds synthetic codec values to the exact prepared-source analysis receipt.
        let numeric_plan = NumericPlanReceipt::from_parent_for_test(
            analysis.shared_analysis().unwrap(),
            synthetic_selected_numeric_plan(Arc::clone(&bap_parameters)),
        );

        let target = Flat7xI0D0TargetPlan::approved_contract();
        let source_identity = numeric_plan.identity();
        let callback_index = FrontendCallbackIndex::new(2);
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
        let d0_audio = SyntheticD0AudioSpans::new(target, &d0_bsi, 1);
        let d0_skip_fields = [D0Block2SkipFieldInput {
            insertion_point: CallerOwnedUnboundD0Block2InsertionPoint::new(d0_audio.prefix_bits),
            complete_emdf_container: &emdf_container,
        }];
        let d0_audio_blocks = d0_audio.frame_input(&d0_skip_fields);
        let d0_receipt = Flat7xD0AudioSpanReceipt {
            source_identity: binding.d0_source_identity().clone(),
            audio_blocks: d0_audio_blocks,
        };
        let pair_audio = prepare_flat7x_pair_audio_blocks(
            binding,
            &numeric_plan,
            synthetic_i0_policy(),
            I0AudioBlockSeriesBudget::new(20_480),
            Flat7xD0AudioSpanInput::Provided(d0_receipt),
        )
        .unwrap();
        let i0_frame = match assemble_flat7x_pair_i0_frame_candidate(
            &pair_audio,
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
        assert_eq!(candidate.access_unit_bytes().len(), 6_656);
        assert_eq!(candidate.access_unit().d0_frame_bytes().len(), 4_096);
        let d0_receipt = candidate.access_unit().d0_frame_receipt();
        assert_eq!(d0_receipt.target_frame_bytes(), 4_096);
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
        let stage_receipt =
            preflight_flat7x_pair_io_stage(&candidate, numeric_plan.identity(), output_owner)
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
                Flat7xAuOutputOwner::cold("mismatched-source", 1).unwrap(),
            ),
            Err(Flat7xPairIoStageError::SourceIdentityMismatch)
        );
    }
}
