// pattern: Functional Core
//! Joins the selected Flat7x source projection to the existing I0/D0 audio-frame seams.

use jocforge_domain::{
    codec::{
        ac3_audio_block::{
            I0AudioBlockSeriesBudget, I0AudioBlockSeriesCandidate, I0AudioBlockSeriesError,
            I0AudioBlockSeriesPolicy, produce_i0_audio_block_series,
        },
        d0_frame_assembler::Flat7xD0FrameBitInput,
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
    target: Flat7xI0D0TargetPlan,
    source_index: CodecSourceIndex,
    role_map_receipt: Flat7xSourceRoleMapReceipt,
    d0_source_identity: Flat7xD0SourceIdentity,
    i0_rows: Flat7xI0PcmRows<'pcm>,
    d0_rows: Flat7xD0PcmRows<'pcm>,
}

impl Flat7xPairSourceBinding<'_> {
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

    fn fixture_span_receipt<'a>(
        source_identity: Flat7xD0SourceIdentity,
        byte: &'a [u8],
    ) -> Flat7xD0AudioSpanReceipt<'a> {
        let block = D0FrameAudioBlockInput::Opaque(OpaqueD0AudioBitSpan::new(byte, 0, 1));
        Flat7xD0AudioSpanReceipt {
            source_identity,
            audio_blocks: Flat7xD0FrameBitInput {
                audio_blocks: [block; 6],
            },
        }
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
