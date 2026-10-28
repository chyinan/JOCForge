// pattern: Functional Core
//! Selected-profile direct-LFE ring staging; F04 assembly and commit remain deferred.

use super::{LfeDelayRingState, SpatialAudioDigest, SpatialAudioJoinMetadata, SpatialAudioMachine};
use crate::{
    numeric::FiniteF32,
    profile::{EncodingProfile, resolve_selected_profile},
    spatial::{
        source_flags::{
            HostId, SourceFlagsAttempt, SourceFlagsProgrammeId, ValidatedSemanticRoute,
        },
        spatial_control::{
            SpatialControlDigest, SpatialControlJoinMetadata, SpatialControlLfeIngressAuthority,
            SpatialControlMachine, SpatialControlProviderLineage,
        },
    },
    timeline::FrontendCallbackIndex,
    transaction::CallbackLineageIdentity,
};
use std::sync::Arc;

const LFE_DELAY_SLOTS: i64 = 3;
const LFE_FRAME_SAMPLES: usize = 1536;
const LFE_ROW_BYTES: usize = LFE_FRAME_SAMPLES * 4;
const MODEL_SAMPLES_PER_CALLBACK: i64 = 1536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::spatial) enum DirectLfeError {
    State,
    PipelineRetired,
    CallbackOrder,
    Input,
    ProfileUnsupported,
}

impl DirectLfeError {
    #[allow(dead_code)]
    pub(in crate::spatial) const fn code(self) -> &'static str {
        match self {
            Self::State => "F04_STATE_ERROR",
            Self::PipelineRetired => "F04_PIPELINE_RETIRED",
            Self::CallbackOrder => "F04_CALLBACK_ORDER",
            Self::Input => "F04_INPUT_ERROR",
            Self::ProfileUnsupported => "F04_PROFILE_UNSUPPORTED",
        }
    }
}

/// Opaque input created only from a validated spatial-control route and callback snapshot.
#[must_use]
#[allow(dead_code)]
pub(in crate::spatial) struct DirectLfeInputView {
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: CallbackLineageIdentity,
    callback_index: FrontendCallbackIndex,
    phase: super::super::post_window_fanout::CallbackPhase,
    profile_identity: EncodingProfile,
    spatial_control_before_generation: u64,
    spatial_control_before_digest: SpatialControlDigest,
    spatial_control_provider_lineage: SpatialControlProviderLineage,
    lfe_host: HostId,
    row_bytes: Arc<[u8]>,
}

impl DirectLfeInputView {
    #[allow(clippy::too_many_arguments)]
    #[allow(dead_code)]
    pub(in crate::spatial) fn try_from_spatial_control(
        authority: SpatialControlLfeIngressAuthority,
        programme_id: SourceFlagsProgrammeId,
        attempt: SourceFlagsAttempt,
        transaction_id: CallbackLineageIdentity,
        callback_index: FrontendCallbackIndex,
        phase: super::super::post_window_fanout::CallbackPhase,
        profile_identity: EncodingProfile,
        spatial_control: &SpatialControlMachine,
        route: &ValidatedSemanticRoute,
        samples: Vec<f32>,
    ) -> Result<Self, DirectLfeError> {
        Self::try_from_spatial_control_host(
            authority,
            programme_id,
            attempt,
            transaction_id,
            callback_index,
            phase,
            profile_identity,
            spatial_control,
            route.lfe_host(),
            samples,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::spatial) fn try_from_spatial_control_host(
        _authority: SpatialControlLfeIngressAuthority,
        programme_id: SourceFlagsProgrammeId,
        attempt: SourceFlagsAttempt,
        transaction_id: CallbackLineageIdentity,
        callback_index: FrontendCallbackIndex,
        phase: super::super::post_window_fanout::CallbackPhase,
        profile_identity: EncodingProfile,
        spatial_control: &SpatialControlMachine,
        lfe_host: HostId,
        samples: Vec<f32>,
    ) -> Result<Self, DirectLfeError> {
        if resolve_selected_profile(&profile_identity).is_err()
            || profile_identity != spatial_control.profile()
        {
            return Err(DirectLfeError::ProfileUnsupported);
        }
        if samples.len() != LFE_FRAME_SAMPLES {
            return Err(DirectLfeError::Input);
        }

        let mut row_bytes = Vec::with_capacity(LFE_ROW_BYTES);
        for sample in samples {
            let finite = FiniteF32::new(sample).map_err(|_| DirectLfeError::Input)?;
            row_bytes.extend_from_slice(&finite.to_bits().to_le_bytes());
        }
        if row_bytes.len() != LFE_ROW_BYTES {
            return Err(DirectLfeError::Input);
        }

        Ok(Self {
            programme_id,
            attempt,
            transaction_id,
            callback_index,
            phase,
            profile_identity,
            spatial_control_before_generation: spatial_control.generation(),
            spatial_control_before_digest: spatial_control.digest(),
            spatial_control_provider_lineage: spatial_control.provider_lineage(),
            lfe_host,
            row_bytes: Arc::from(row_bytes.into_boxed_slice()),
        })
    }
}

/// Linear substage proposal; the future parent composer owns any whole-root replacement.
#[must_use]
#[allow(dead_code)]
pub(in crate::spatial) struct LfeStageProposal {
    after: LfeDelayRingState,
    type10_row0: Option<Type10LfeRow0Input>,
    lineage: DirectLfeStageLineage,
}

impl LfeStageProposal {
    #[allow(dead_code)]
    pub(super) fn into_parts(
        self,
    ) -> (
        LfeDelayRingState,
        Option<Type10LfeRow0Input>,
        DirectLfeStageLineage,
    ) {
        (self.after, self.type10_row0, self.lineage)
    }
}

#[must_use]
#[allow(dead_code)]
pub(super) struct Type10LfeRow0Input {
    callback_generation: FrontendCallbackIndex,
    origin_generation: FrontendCallbackIndex,
    row_bytes: Arc<[u8]>,
}

impl Type10LfeRow0Input {
    #[allow(dead_code)]
    pub(super) const fn callback_generation(&self) -> FrontendCallbackIndex {
        self.callback_generation
    }

    #[allow(dead_code)]
    pub(super) const fn origin_generation(&self) -> FrontendCallbackIndex {
        self.origin_generation
    }

    #[allow(dead_code)]
    pub(super) fn row_bytes(&self) -> &[u8] {
        &self.row_bytes
    }
}

#[allow(dead_code)]
pub(super) struct DirectLfeStageLineage {
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: CallbackLineageIdentity,
    callback_index: FrontendCallbackIndex,
    phase: super::super::post_window_fanout::CallbackPhase,
    profile_identity: EncodingProfile,
    spatial_audio_before_generation: u64,
    spatial_audio_before_digest: SpatialAudioDigest,
    spatial_control_before_generation: u64,
    spatial_control_before_digest: SpatialControlDigest,
    spatial_control_provider_lineage: SpatialControlProviderLineage,
    lfe_host: HostId,
}

impl DirectLfeStageLineage {
    #[allow(dead_code)]
    pub(super) fn programme_id(&self) -> &SourceFlagsProgrammeId {
        &self.programme_id
    }

    #[allow(dead_code)]
    pub(super) const fn attempt(&self) -> SourceFlagsAttempt {
        self.attempt
    }

    #[allow(dead_code)]
    pub(super) fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    #[allow(dead_code)]
    pub(super) const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    #[allow(dead_code)]
    pub(super) const fn phase(&self) -> super::super::post_window_fanout::CallbackPhase {
        self.phase
    }

    #[allow(dead_code)]
    pub(super) const fn profile_identity(&self) -> EncodingProfile {
        self.profile_identity
    }

    #[allow(dead_code)]
    pub(super) const fn spatial_audio_before_generation(&self) -> u64 {
        self.spatial_audio_before_generation
    }

    #[allow(dead_code)]
    pub(super) const fn spatial_audio_before_digest(&self) -> SpatialAudioDigest {
        self.spatial_audio_before_digest
    }

    #[allow(dead_code)]
    pub(super) const fn spatial_control_before_generation(&self) -> u64 {
        self.spatial_control_before_generation
    }

    #[allow(dead_code)]
    pub(super) const fn spatial_control_before_digest(&self) -> SpatialControlDigest {
        self.spatial_control_before_digest
    }

    #[allow(dead_code)]
    pub(super) const fn spatial_control_provider_lineage(&self) -> SpatialControlProviderLineage {
        self.spatial_control_provider_lineage
    }

    #[allow(dead_code)]
    pub(super) const fn lfe_host(&self) -> HostId {
        self.lfe_host
    }
}

#[allow(dead_code)]
pub(in crate::spatial) fn produce_direct_lfe_stage(
    spatial_audio: &SpatialAudioMachine,
    spatial_audio_join: &SpatialAudioJoinMetadata,
    spatial_control_join: &SpatialControlJoinMetadata,
    spatial_control: &SpatialControlMachine,
    expected_phase: super::super::post_window_fanout::CallbackPhase,
    input: DirectLfeInputView,
) -> Result<LfeStageProposal, DirectLfeError> {
    let before = &spatial_audio.committed.root;
    if before.retired {
        return Err(DirectLfeError::PipelineRetired);
    }

    let selected_profile = spatial_audio.profile_identity();
    for candidate in [
        selected_profile,
        spatial_audio_join.profile_identity(),
        spatial_control_join.profile_identity(),
        spatial_control.profile(),
        input.profile_identity,
    ] {
        if resolve_selected_profile(&candidate).is_err() || candidate != selected_profile {
            return Err(DirectLfeError::ProfileUnsupported);
        }
    }

    let callback_index = before.callback_index;
    if spatial_audio_join.callback_index() != callback_index
        || spatial_control_join.callback_index() != callback_index
        || input.callback_index != callback_index
        || input.phase != expected_phase
    {
        return Err(DirectLfeError::CallbackOrder);
    }

    if spatial_audio_join.before_generation() != spatial_audio.generation()
        || spatial_audio_join.before_digest() != spatial_audio.digest()
        || spatial_control_join.before_generation() != spatial_control.generation()
        || spatial_control_join.before_digest() != spatial_control.digest()
        || input.spatial_control_before_generation != spatial_control.generation()
        || input.spatial_control_before_digest != spatial_control.digest()
    {
        return Err(DirectLfeError::State);
    }

    validate_lfe_ring_state(&before.lfe_state, callback_index)?;

    if input.programme_id != *spatial_audio_join.programme_id()
        || input.attempt != spatial_audio_join.attempt()
        || input.transaction_id != *spatial_audio_join.transaction_id()
        || input.transaction_id != *spatial_control_join.transaction_id()
        || input.spatial_control_provider_lineage != spatial_control.provider_lineage()
        || spatial_control_join.provider_lineage() != spatial_control.provider_lineage()
        || !valid_row_bytes(&input.row_bytes)
    {
        return Err(DirectLfeError::Input);
    }

    stage_lfe_ring_transition(
        &before.lfe_state,
        callback_index,
        expected_phase,
        input,
        spatial_audio.generation(),
        spatial_audio.digest(),
    )
}

fn stage_lfe_ring_transition(
    before: &LfeDelayRingState,
    callback_index: FrontendCallbackIndex,
    expected_phase: super::super::post_window_fanout::CallbackPhase,
    input: DirectLfeInputView,
    spatial_audio_before_generation: u64,
    spatial_audio_before_digest: SpatialAudioDigest,
) -> Result<LfeStageProposal, DirectLfeError> {
    if input.callback_index != callback_index || input.phase != expected_phase {
        return Err(DirectLfeError::CallbackOrder);
    }
    validate_lfe_ring_state(before, callback_index)?;
    if !valid_row_bytes(&input.row_bytes) {
        return Err(DirectLfeError::Input);
    }

    let next_write = (before.write_index + 1) % LFE_DELAY_SLOTS;
    let next_read = (before.read_index + 1) % LFE_DELAY_SLOTS;
    let next_write_slot = usize::try_from(next_write).map_err(|_| DirectLfeError::State)?;
    let next_read_slot = usize::try_from(next_read).map_err(|_| DirectLfeError::State)?;

    // Read the old next_read slot before assigning the current callback into next_write.
    let output_row = before.rows[next_read_slot].as_ref().map(Arc::clone);
    let output_origin = before.input_frame_indexes[next_read_slot];
    let expected_origin = callback_index.value().checked_sub(2);
    match (expected_origin, output_row.as_ref(), output_origin) {
        (None, None, None) => {}
        (Some(expected), Some(_), Some(origin)) if origin.value() == expected => {}
        _ => return Err(DirectLfeError::State),
    }

    let mut after_rows = [
        before.rows[0].as_ref().map(Arc::clone),
        before.rows[1].as_ref().map(Arc::clone),
        before.rows[2].as_ref().map(Arc::clone),
    ];
    let mut after_indexes = before.input_frame_indexes;
    after_rows[next_write_slot] = Some(input.row_bytes);
    after_indexes[next_write_slot] = Some(callback_index);
    let model_samples_consumed = before
        .model_samples_consumed
        .checked_add(MODEL_SAMPLES_PER_CALLBACK)
        .ok_or(DirectLfeError::State)?;

    let type10_row0 = match (output_row, output_origin) {
        (Some(row_bytes), Some(origin_generation)) => Some(Type10LfeRow0Input {
            callback_generation: callback_index,
            origin_generation,
            row_bytes,
        }),
        (None, None) => None,
        _ => return Err(DirectLfeError::State),
    };

    let after = LfeDelayRingState {
        write_index: next_write,
        read_index: next_read,
        rows: after_rows,
        input_frame_indexes: after_indexes,
        model_samples_consumed,
    };
    let lineage = DirectLfeStageLineage {
        programme_id: input.programme_id,
        attempt: input.attempt,
        transaction_id: input.transaction_id,
        callback_index,
        phase: input.phase,
        profile_identity: input.profile_identity,
        spatial_audio_before_generation,
        spatial_audio_before_digest,
        spatial_control_before_generation: input.spatial_control_before_generation,
        spatial_control_before_digest: input.spatial_control_before_digest,
        spatial_control_provider_lineage: input.spatial_control_provider_lineage,
        lfe_host: input.lfe_host,
    };
    Ok(LfeStageProposal {
        after,
        type10_row0,
        lineage,
    })
}

pub(super) fn validate_lfe_ring_state(
    state: &LfeDelayRingState,
    callback_index: FrontendCallbackIndex,
) -> Result<(), DirectLfeError> {
    let expected_model_samples = i64::from(callback_index.value())
        .checked_mul(MODEL_SAMPLES_PER_CALLBACK)
        .ok_or(DirectLfeError::State)?;
    if !(0..LFE_DELAY_SLOTS).contains(&state.write_index)
        || !(0..LFE_DELAY_SLOTS).contains(&state.read_index)
        || state.read_index != (state.write_index + 1) % LFE_DELAY_SLOTS
        || state.model_samples_consumed != expected_model_samples
    {
        return Err(DirectLfeError::State);
    }
    for (row, index) in state.rows.iter().zip(state.input_frame_indexes.iter()) {
        match (row, index) {
            (None, None) => {}
            (Some(bytes), Some(index))
                if valid_row_bytes(bytes) && index.value() < callback_index.value() => {}
            _ => return Err(DirectLfeError::State),
        }
    }
    Ok(())
}

pub(super) fn valid_row_bytes(bytes: &[u8]) -> bool {
    bytes.len() == LFE_ROW_BYTES
        && bytes.chunks_exact(4).all(|word| {
            f32::from_le_bytes(word.try_into().expect("four-byte F32 word")).is_finite()
        })
}

#[cfg(test)]
mod tests;

#[cfg(test)]
impl Type10LfeRow0Input {
    pub(in crate::spatial::spatial_audio_state) fn fixture_for_type10(
        callback_generation: FrontendCallbackIndex,
        origin_generation: FrontendCallbackIndex,
        row_bytes: Vec<u8>,
    ) -> Self {
        Self {
            callback_generation,
            origin_generation,
            row_bytes: Arc::from(row_bytes.into_boxed_slice()),
        }
    }
}

#[cfg(test)]
impl LfeStageProposal {
    pub(in crate::spatial::spatial_audio_state) fn fixture_for_type10(
        spatial_audio_join: &SpatialAudioJoinMetadata,
        phase: super::super::post_window_fanout::CallbackPhase,
        spatial_control: &SpatialControlMachine,
        lfe_host: HostId,
        after: LfeDelayRingState,
        type10_row0: Option<Type10LfeRow0Input>,
        spatial_control_provider_lineage: Option<&SpatialControlProviderLineage>,
    ) -> Self {
        Self {
            after,
            type10_row0,
            lineage: DirectLfeStageLineage {
                programme_id: spatial_audio_join.programme_id().clone(),
                attempt: spatial_audio_join.attempt(),
                transaction_id: spatial_audio_join.transaction_id().clone(),
                callback_index: spatial_audio_join.callback_index(),
                phase,
                profile_identity: spatial_audio_join.profile_identity(),
                spatial_audio_before_generation: spatial_audio_join.before_generation(),
                spatial_audio_before_digest: spatial_audio_join.before_digest(),
                spatial_control_before_generation: spatial_control.generation(),
                spatial_control_before_digest: spatial_control.digest(),
                spatial_control_provider_lineage: spatial_control_provider_lineage
                    .copied()
                    .unwrap_or_else(|| spatial_control.provider_lineage()),
                lfe_host,
            },
        }
    }
}
