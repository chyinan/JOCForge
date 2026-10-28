// pattern: Functional Core
//! Stages source flags and current features from one selected callback input.

use super::{
    current_features::{
        CurrentFeaturesError, CurrentFeaturesPacket, CurrentFeaturesProduced,
        produce_current_features,
    },
    point_response::{PointResponseError, PointResponseGateInput},
    post_window_fanout::{
        CallbackPhase, CurrentFeaturesAudioView, PostWindowRenderProposal,
        SpatialAudioPostWindowView,
    },
    source_flags::{
        RawHistoryProposal, SourceFlagsError, SourceFlagsInput, SourceFlagsPacket,
        SourceFlagsProducer,
    },
    spatial_control::SpatialControlMachine,
};
use crate::frontend::{ValidatedFeatureFft4096MathProvider, ValidatedFeatureProcessorMathProvider};
use crate::meter::installed48_renderer::{
    MeterInstalled48Frame, MeterInstalled48Proposal, MeterInstalled48State,
};
use crate::numeric::{FiniteF32, r32_add};
use crate::spatial::installed48_activity::{PrimaryActivityPhase, PrimaryActivityProposal};
use crate::transaction::CallbackLineageIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontendFeatureStageError {
    SourceFlags(SourceFlagsError),
    CurrentFeatures(CurrentFeaturesError),
    PrimaryActivityIdentity,
    MeterFrameIdentity,
    PostWindow(super::post_window_fanout::PostWindowFanoutError),
}

/// Source-ascending gates and the unmodified candidate feature snapshot.
#[must_use]
#[allow(dead_code)]
pub(super) struct FeatureGateOutput {
    pub(super) candidate_ids: Box<[u32]>,
    pub(super) processing_ids: Box<[u32]>,
    pub(super) response_source_ids: Box<[u32]>,
    pub(super) candidate_features: Box<[Box<[u32]>]>,
}

/// Linear proposals from the source-flags and current-features stages.
#[must_use]
#[allow(dead_code)]
pub(super) struct FrontendFeatureStageProposal {
    pub(super) current_features: CurrentFeaturesProduced,
    pub(super) raw_history: RawHistoryProposal,
    pub(super) gate: FeatureGateOutput,
}

/// Callback-local frontend stages plus the single-use audio view reserved for component execution.
#[must_use]
pub struct RenderedFrontendFeatureStageProposal {
    pub(super) primary_activity: PrimaryActivityProposal,
    pub(super) installed48: MeterInstalled48Frame,
    pub(super) installed48_after: Option<MeterInstalled48State>,
    pub(super) frontend: FrontendFeatureStageProposal,
    pub(super) spatial_audio: SpatialAudioPostWindowView,
}

impl RenderedFrontendFeatureStageProposal {
    pub const fn primary_activity(&self) -> &PrimaryActivityProposal {
        &self.primary_activity
    }

    pub const fn installed48(&self) -> &MeterInstalled48Frame {
        &self.installed48
    }

    pub(super) fn take_installed48_after(&mut self) -> Option<MeterInstalled48State> {
        self.installed48_after.take()
    }
}

impl FrontendFeatureStageProposal {
    /// Transfers this callback's gate IDs with the same transaction lineage to F03P.
    #[allow(dead_code)] // The parent callback composer will consume this with the F01E source snapshot.
    pub(super) fn point_response_gate(&self) -> Result<PointResponseGateInput, PointResponseError> {
        PointResponseGateInput::new(
            self.current_features.lineage.callback_index,
            self.current_features.lineage.transaction_id.clone(),
            self.gate.candidate_ids.to_vec(),
            self.gate.processing_ids.to_vec(),
        )
    }
}

/// Runs each producer once and keeps both state proposals pending for the parent owner.
#[allow(dead_code)]
pub(super) fn stage_frontend_features(
    owner: &SpatialControlMachine,
    input: SourceFlagsInput,
    callback_phase: CallbackPhase,
    post_window_audio: Option<CurrentFeaturesAudioView>,
    processor: &ValidatedFeatureProcessorMathProvider,
    fft_provider: &ValidatedFeatureFft4096MathProvider,
) -> Result<FrontendFeatureStageProposal, FrontendFeatureStageError> {
    if input.source_control_clock().is_some_and(|clock| {
        clock.is_cached_drain() != (callback_phase == CallbackPhase::CachedDrain)
    }) {
        return Err(FrontendFeatureStageError::SourceFlags(
            SourceFlagsError::SourceControlClockMismatch,
        ));
    }
    let produced =
        SourceFlagsProducer::produce(input).map_err(FrontendFeatureStageError::SourceFlags)?;
    let (source_flags, raw_history) = produced
        .into_parts()
        .map_err(FrontendFeatureStageError::SourceFlags)?;
    let lineage = source_flags.lineage();
    let programme_id = lineage.programme_id().clone();
    let attempt = lineage.attempt();
    let transaction_id = lineage.transaction_id().clone();
    let callback_index = lineage.callback_index();

    let current_features = produce_current_features(
        owner,
        &programme_id,
        attempt,
        &transaction_id,
        callback_index,
        callback_phase,
        source_flags,
        post_window_audio,
        processor,
        fft_provider,
    )
    .map_err(FrontendFeatureStageError::CurrentFeatures)?;

    let gate = gate_current_features(&current_features.packet, &current_features.source_flags)?;

    Ok(FrontendFeatureStageProposal {
        current_features,
        raw_history,
        gate,
    })
}

/// Fans one actual installed48 frame to the feature owner and preserves the second view for F04A.
#[allow(clippy::too_many_arguments)]
pub fn stage_frontend_features_from_installed48(
    owner: &SpatialControlMachine,
    transaction_id: CallbackLineageIdentity,
    primary_activity: PrimaryActivityProposal,
    installed48: MeterInstalled48Proposal,
    input: SourceFlagsInput,
    callback_phase: CallbackPhase,
    processor: &ValidatedFeatureProcessorMathProvider,
    fft_provider: &ValidatedFeatureFft4096MathProvider,
) -> Result<RenderedFrontendFeatureStageProposal, FrontendFeatureStageError> {
    let callback = transaction_id.callback_index();
    let activity_output = primary_activity.output();
    if callback_phase != CallbackPhase::Normal
        || activity_output.callback_index() != callback
        || activity_output.phase() != PrimaryActivityPhase::NormalRead
        || activity_output.host_frame() != Some(callback)
    {
        return Err(FrontendFeatureStageError::PrimaryActivityIdentity);
    }
    let (installed48_after, installed48_frame) = installed48.into_parts();
    let expected_upstream_generation = u64::from(callback.value())
        .checked_add(1)
        .ok_or(FrontendFeatureStageError::MeterFrameIdentity)?;
    if installed48_frame.callback_index().value() != u64::from(callback.value())
        || installed48_frame.upstream_generation() != expected_upstream_generation
        || installed48_frame.rows().len() != 48
        || installed48_frame
            .rows()
            .iter()
            .any(|row| row.len() != 3_072)
        || owner.committed().callback_index() != callback
    {
        return Err(FrontendFeatureStageError::MeterFrameIdentity);
    }

    let installed48_payload = installed48_frame
        .rows()
        .iter()
        .flat_map(|row| row.iter().copied().map(f32::from_bits))
        .collect::<Vec<_>>();
    let render = PostWindowRenderProposal::try_from_spatial_control_callback(
        SpatialControlMachine::render_authority(),
        owner,
        transaction_id,
        callback_phase,
        installed48_payload,
    )
    .map_err(FrontendFeatureStageError::PostWindow)?;
    let (current_features_audio, spatial_audio) = render.fanout();
    let frontend = stage_frontend_features(
        owner,
        input,
        callback_phase,
        Some(current_features_audio),
        processor,
        fft_provider,
    )?;
    Ok(RenderedFrontendFeatureStageProposal {
        primary_activity,
        installed48: installed48_frame,
        installed48_after: Some(installed48_after),
        frontend,
        spatial_audio,
    })
}

/// Applies the selected plus4 threshold and plus0 processing gates to band-major features.
pub(super) fn gate_current_features(
    features: &CurrentFeaturesPacket,
    source_flags: &SourceFlagsPacket,
) -> Result<FeatureGateOutput, FrontendFeatureStageError> {
    source_flags
        .validate_payload()
        .map_err(FrontendFeatureStageError::SourceFlags)?;
    let bands = features.band_major_features();
    let source_count = bands.first().map_or(0, |band| band.len());
    let half = FiniteF32::new(0.5).map_err(|_| numeric_error())?;
    let mut candidate_ids = Vec::new();
    let mut processing_ids = Vec::new();

    for source in 0..source_count {
        let mut total = FiniteF32::new(0.0).map_err(|_| numeric_error())?;
        for band in bands {
            let value =
                FiniteF32::new(f32::from_bits(band[source])).map_err(|_| numeric_error())?;
            total = r32_add(total, value).map_err(|_| numeric_error())?;
        }

        let [plus0, plus4] = source_flags.payload()[source];
        let source_id = u32::try_from(source).map_err(|_| {
            FrontendFeatureStageError::CurrentFeatures(CurrentFeaturesError::SourceShape)
        })?;
        if plus4 == 0 && total.value() >= half.value() {
            candidate_ids.push(source_id);
        }
        if plus0 == 0 {
            processing_ids.push(source_id);
        }
    }

    let response_source_ids =
        merge_response_source_ids(&candidate_ids, &processing_ids, source_count)?;
    let candidate_features = bands
        .iter()
        .map(|band| {
            candidate_ids
                .iter()
                .map(|source| {
                    let source = usize::try_from(*source)
                        .expect("candidate source IDs originate from the feature extent");
                    band[source]
                })
                .collect::<Vec<_>>()
                .into_boxed_slice()
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();

    Ok(FeatureGateOutput {
        candidate_ids: candidate_ids.into_boxed_slice(),
        processing_ids: processing_ids.into_boxed_slice(),
        response_source_ids,
        candidate_features,
    })
}

fn merge_response_source_ids(
    candidate_ids: &[u32],
    processing_ids: &[u32],
    source_count: usize,
) -> Result<Box<[u32]>, FrontendFeatureStageError> {
    let source_count = u32::try_from(source_count).map_err(|_| {
        FrontendFeatureStageError::CurrentFeatures(CurrentFeaturesError::SourceShape)
    })?;
    if candidate_ids
        .iter()
        .chain(processing_ids)
        .any(|source| *source >= source_count)
        || candidate_ids.windows(2).any(|pair| pair[0] >= pair[1])
        || processing_ids.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(FrontendFeatureStageError::CurrentFeatures(
            CurrentFeaturesError::SourceShape,
        ));
    }

    let capacity = candidate_ids
        .len()
        .checked_add(processing_ids.len())
        .ok_or(FrontendFeatureStageError::CurrentFeatures(
            CurrentFeaturesError::SourceShape,
        ))?;
    let mut union = Vec::with_capacity(capacity);
    let (mut candidate, mut processing) = (0, 0);
    while candidate < candidate_ids.len() && processing < processing_ids.len() {
        match candidate_ids[candidate].cmp(&processing_ids[processing]) {
            std::cmp::Ordering::Less => {
                union.push(candidate_ids[candidate]);
                candidate += 1;
            }
            std::cmp::Ordering::Greater => {
                union.push(processing_ids[processing]);
                processing += 1;
            }
            std::cmp::Ordering::Equal => {
                union.push(candidate_ids[candidate]);
                candidate += 1;
                processing += 1;
            }
        }
    }
    union.extend_from_slice(&candidate_ids[candidate..]);
    union.extend_from_slice(&processing_ids[processing..]);
    Ok(union.into_boxed_slice())
}

fn numeric_error() -> FrontendFeatureStageError {
    FrontendFeatureStageError::CurrentFeatures(CurrentFeaturesError::NonFiniteIntermediate)
}
