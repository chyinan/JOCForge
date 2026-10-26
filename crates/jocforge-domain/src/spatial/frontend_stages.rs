// pattern: Functional Core
//! Stages source flags and current features from one selected callback input.

use super::{
    current_features::{
        CurrentFeaturesError, CurrentFeaturesPacket, CurrentFeaturesProduced,
        produce_current_features,
    },
    post_window_fanout::{CallbackPhase, CurrentFeaturesAudioView},
    source_flags::{
        RawHistoryProposal, SourceFlagsError, SourceFlagsInput, SourceFlagsPacket,
        SourceFlagsProducer,
    },
    spatial_control::SpatialControlMachine,
};
use crate::frontend::{ValidatedFeatureFft4096MathProvider, ValidatedFeatureProcessorMathProvider};
use crate::numeric::{FiniteF32, r32_add};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FrontendFeatureStageError {
    SourceFlags(SourceFlagsError),
    CurrentFeatures(CurrentFeaturesError),
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
