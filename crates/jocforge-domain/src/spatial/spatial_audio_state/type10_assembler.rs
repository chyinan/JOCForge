// pattern: Functional Core
//! Callback-local Type10 assembly boundary.

use super::{LfeDelayRingState, SpatialAudioDigest, SpatialAudioJoinMetadata};
use crate::{
    numeric::FiniteF32,
    profile::{EncodingProfile, resolve_selected_profile},
    spatial::{
        post_window_fanout::CallbackPhase,
        spatial_audio_state::component_stage::{
            ComponentAfterState, ComponentCallbackPhase, ComponentStageProposal,
            ComponentStageReceipt,
        },
        spatial_audio_state::direct_lfe::{
            DirectLfeStageLineage, LfeStageProposal, Type10LfeRow0Input, valid_row_bytes,
            validate_lfe_ring_state,
        },
    },
    timeline::FrontendCallbackIndex,
};
use std::sync::Arc;

const COMPONENT_ROW_COUNT: usize = 15;
const TYPE10_ROW_COUNT: usize = 16;
const FRAME_SAMPLES: usize = 1536;
const TYPE10_SAMPLE_COUNT: usize = TYPE10_ROW_COUNT * FRAME_SAMPLES;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(super) enum Type10AssemblyError {
    State,
    CallbackOrder,
    Input,
    ProfileUnsupported,
}

impl Type10AssemblyError {
    #[allow(dead_code)]
    pub(super) const fn code(self) -> &'static str {
        match self {
            Self::State => "F04_STATE_ERROR",
            Self::CallbackOrder => "F04_CALLBACK_ORDER",
            Self::Input => "F04_INPUT_ERROR",
            Self::ProfileUnsupported => "F04_PROFILE_UNSUPPORTED",
        }
    }
}

#[must_use]
#[allow(dead_code)]
pub(super) struct Type10Packet {
    callback_index: FrontendCallbackIndex,
    payload: Arc<[FiniteF32]>,
}

impl Type10Packet {
    #[allow(dead_code)]
    pub(super) const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    #[allow(dead_code)]
    pub(super) fn payload(&self) -> &[FiniteF32] {
        &self.payload
    }

    #[allow(dead_code)]
    pub(super) fn shape(&self) -> [usize; 2] {
        [self.payload.len() / FRAME_SAMPLES, FRAME_SAMPLES]
    }
}

#[allow(dead_code)]
pub(super) struct Type10AssemblyReceipt {
    component_lineage: ComponentStageReceipt,
    lfe_lineage: DirectLfeStageLineage,
    component_origin_generation: FrontendCallbackIndex,
    lfe_origin_generation: Option<FrontendCallbackIndex>,
    shape: [usize; 2],
}

impl Type10AssemblyReceipt {
    #[allow(dead_code)]
    pub(super) const fn callback_index(&self) -> FrontendCallbackIndex {
        self.component_lineage.callback_index()
    }

    #[allow(dead_code)]
    pub(super) const fn component_origin_generation(&self) -> FrontendCallbackIndex {
        self.component_origin_generation
    }

    #[allow(dead_code)]
    pub(super) const fn lfe_origin_generation(&self) -> Option<FrontendCallbackIndex> {
        self.lfe_origin_generation
    }

    #[allow(dead_code)]
    pub(super) const fn shape(&self) -> [usize; 2] {
        self.shape
    }

    #[allow(dead_code)]
    pub(super) fn programme_id(&self) -> &crate::spatial::source_flags::SourceFlagsProgrammeId {
        self.component_lineage.programme_id()
    }

    #[allow(dead_code)]
    pub(super) const fn attempt(&self) -> crate::spatial::source_flags::SourceFlagsAttempt {
        self.component_lineage.attempt()
    }

    #[allow(dead_code)]
    pub(super) fn transaction_id(&self) -> &crate::transaction::CallbackLineageIdentity {
        self.component_lineage.transaction_id()
    }

    #[allow(dead_code)]
    pub(super) const fn phase(&self) -> CallbackPhase {
        self.lfe_lineage.phase()
    }

    #[allow(dead_code)]
    pub(super) const fn profile_identity(&self) -> EncodingProfile {
        self.component_lineage.profile_identity()
    }

    #[allow(dead_code)]
    pub(super) const fn spatial_audio_before_generation(&self) -> u64 {
        self.component_lineage.spatial_audio_before_generation()
    }

    #[allow(dead_code)]
    pub(super) const fn spatial_audio_before_digest(&self) -> SpatialAudioDigest {
        self.component_lineage.spatial_audio_before_digest()
    }

    #[allow(dead_code)]
    pub(super) const fn spatial_control_provider_lineage(
        &self,
    ) -> crate::spatial::spatial_control::SpatialControlProviderLineage {
        self.component_lineage.spatial_control_provider_lineage()
    }

    #[allow(dead_code)]
    pub(super) const fn spatial_control_before_generation(&self) -> u64 {
        self.component_lineage.spatial_control_before_generation()
    }

    #[allow(dead_code)]
    pub(super) const fn spatial_control_before_digest(
        &self,
    ) -> crate::spatial::spatial_control::SpatialControlDigest {
        self.component_lineage.spatial_control_before_digest()
    }

    #[allow(dead_code)]
    pub(super) const fn f04_provider_lineage(
        &self,
    ) -> [crate::spatial::F04ParameterProviderLineage; 2] {
        self.component_lineage.provider_lineage()
    }

    #[allow(dead_code)]
    pub(super) const fn lfe_host(&self) -> crate::spatial::source_flags::HostId {
        self.lfe_lineage.lfe_host()
    }
}

#[allow(dead_code)]
pub(super) enum Type10AssemblyOutcome {
    Priming {
        receipt: Type10AssemblyReceipt,
    },
    Ready {
        packet: Type10Packet,
        receipt: Type10AssemblyReceipt,
    },
}

#[must_use]
#[allow(dead_code)]
pub(super) struct Type10AssemblyProposal {
    component_after: ComponentAfterState,
    lfe_after: LfeDelayRingState,
    outcome: Type10AssemblyOutcome,
}

impl Type10AssemblyProposal {
    #[allow(dead_code)]
    pub(super) fn into_parts(
        self,
    ) -> (
        ComponentAfterState,
        LfeDelayRingState,
        Type10AssemblyOutcome,
    ) {
        (self.component_after, self.lfe_after, self.outcome)
    }
}

#[allow(dead_code)]
pub(super) fn assemble_type10(
    expected_parent: &SpatialAudioJoinMetadata,
    expected_phase: CallbackPhase,
    component: ComponentStageProposal,
    lfe: LfeStageProposal,
) -> Result<Type10AssemblyProposal, Type10AssemblyError> {
    let (component_after, component_output, component_lineage) = component.into_parts();
    let (lfe_after, type10_row0, lfe_lineage) = lfe.into_parts();

    if resolve_selected_profile(&expected_parent.profile_identity()).is_err()
        || component_lineage.profile_identity() != expected_parent.profile_identity()
        || lfe_lineage.profile_identity() != expected_parent.profile_identity()
    {
        return Err(Type10AssemblyError::ProfileUnsupported);
    }

    if component_lineage.callback_index() != expected_parent.callback_index()
        || lfe_lineage.callback_index() != expected_parent.callback_index()
        || !component_phase_matches(component_lineage.phase(), expected_phase)
        || lfe_lineage.phase() != expected_phase
    {
        return Err(Type10AssemblyError::CallbackOrder);
    }

    if component_lineage.spatial_audio_before_generation() != expected_parent.before_generation()
        || component_lineage.spatial_audio_before_digest() != expected_parent.before_digest()
        || lfe_lineage.spatial_audio_before_generation() != expected_parent.before_generation()
        || lfe_lineage.spatial_audio_before_digest() != expected_parent.before_digest()
    {
        return Err(Type10AssemblyError::State);
    }

    if !component_lfe_same_parent(&component_lineage, &lfe_lineage, expected_parent) {
        return Err(Type10AssemblyError::Input);
    }

    let callback_index = expected_parent.callback_index();
    let after_callback_count = callback_index
        .value()
        .checked_add(1)
        .map(FrontendCallbackIndex::new)
        .ok_or(Type10AssemblyError::State)?;
    validate_lfe_ring_state(&lfe_after, after_callback_count)
        .map_err(|_| Type10AssemblyError::State)?;

    let expected_lfe_origin = callback_index.value().checked_sub(2);
    let lfe_origin_generation = validate_row0_ring_origin(
        &lfe_after,
        type10_row0.as_ref(),
        callback_index,
        expected_lfe_origin,
    )?;

    if component_output.len() != COMPONENT_ROW_COUNT * FRAME_SAMPLES
        || component_output
            .iter()
            .any(|sample| !sample.value().is_finite())
    {
        return Err(Type10AssemblyError::State);
    }

    let receipt = Type10AssemblyReceipt {
        component_lineage,
        lfe_lineage,
        component_origin_generation: callback_index,
        lfe_origin_generation,
        shape: [TYPE10_ROW_COUNT, FRAME_SAMPLES],
    };
    let outcome = match (expected_lfe_origin, type10_row0) {
        (None, None) => Type10AssemblyOutcome::Priming { receipt },
        (Some(_), Some(row0)) => {
            let mut payload = Vec::with_capacity(TYPE10_SAMPLE_COUNT);
            for word in row0.row_bytes().chunks_exact(4) {
                let bits =
                    u32::from_le_bytes(word.try_into().map_err(|_| Type10AssemblyError::State)?);
                payload.push(
                    FiniteF32::new(f32::from_bits(bits)).map_err(|_| Type10AssemblyError::State)?,
                );
            }
            payload.extend(component_output.into_vec());
            if payload.len() != TYPE10_SAMPLE_COUNT {
                return Err(Type10AssemblyError::State);
            }
            let packet = Type10Packet {
                callback_index,
                payload: Arc::from(payload.into_boxed_slice()),
            };
            Type10AssemblyOutcome::Ready { packet, receipt }
        }
        _ => return Err(Type10AssemblyError::State),
    };

    Ok(Type10AssemblyProposal {
        component_after,
        lfe_after,
        outcome,
    })
}

fn component_phase_matches(phase: ComponentCallbackPhase, expected: CallbackPhase) -> bool {
    matches!(
        (phase, expected),
        (ComponentCallbackPhase::Normal, CallbackPhase::Normal)
            | (
                ComponentCallbackPhase::CachedDrain,
                CallbackPhase::CachedDrain
            )
    )
}

fn component_lfe_same_parent(
    component: &ComponentStageReceipt,
    lfe: &DirectLfeStageLineage,
    expected_parent: &SpatialAudioJoinMetadata,
) -> bool {
    component.programme_id() == expected_parent.programme_id()
        && lfe.programme_id() == expected_parent.programme_id()
        && component.attempt() == expected_parent.attempt()
        && lfe.attempt() == expected_parent.attempt()
        && component.transaction_id() == expected_parent.transaction_id()
        && lfe.transaction_id() == expected_parent.transaction_id()
        && component.profile_identity() == expected_parent.profile_identity()
        && lfe.profile_identity() == expected_parent.profile_identity()
        && component.spatial_control_before_generation() == lfe.spatial_control_before_generation()
        && component.spatial_control_before_digest() == lfe.spatial_control_before_digest()
        && component.spatial_control_provider_lineage() == lfe.spatial_control_provider_lineage()
}

fn validate_row0_ring_origin(
    after: &LfeDelayRingState,
    row0: Option<&Type10LfeRow0Input>,
    callback_index: FrontendCallbackIndex,
    expected_origin: Option<u32>,
) -> Result<Option<FrontendCallbackIndex>, Type10AssemblyError> {
    match (expected_origin, row0) {
        (None, None) => Ok(None),
        (None, Some(_)) | (Some(_), None) => Err(Type10AssemblyError::State),
        (Some(expected), Some(row0)) => {
            if row0.callback_generation() != callback_index
                || row0.origin_generation().value() != expected
                || !valid_row_bytes(row0.row_bytes())
            {
                return Err(Type10AssemblyError::State);
            }
            let matches = after
                .input_frame_indexes
                .iter()
                .zip(after.rows.iter())
                .filter(|(index, row)| {
                    **index == Some(row0.origin_generation())
                        && row.as_deref() == Some(row0.row_bytes())
                })
                .count();
            if matches != 1 {
                return Err(Type10AssemblyError::State);
            }
            Ok(Some(row0.origin_generation()))
        }
    }
}

#[cfg(test)]
mod tests;
