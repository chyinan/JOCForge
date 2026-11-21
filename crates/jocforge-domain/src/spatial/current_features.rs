// pattern: Functional Core
//! Source-axis ordered CurrentFeatures consumer and linear staged proposal boundary.

use super::{
    SourceFlagsPacket,
    post_window_fanout::{
        AudioPayloadSha256, CallbackPhase, CurrentFeaturesAudioView, PostWindowBoundary,
        RenderStageIdentity,
    },
    spatial_control::{
        SpatialControlDigest, SpatialControlJoinMetadata, SpatialControlMachine,
        SpatialControlProviderLineage, SpatialControlRenderIdentity,
    },
};
use crate::{
    frontend::{
        FeatureFft4096MathProviderReceipt, FeatureFft4096MathRoute,
        FeatureProcessorMathProviderReceipt, ValidatedFeatureFft4096MathProvider,
        ValidatedFeatureProcessorMathProvider,
    },
    numeric::{FiniteF32, R32Error, r32_add, r32_mul, r32_sub},
    profile::{EncodingProfile, resolve_selected_profile},
    source_axis::{SelectedSourceRowDimensions, SourceAxisBinding},
    spatial::source_flags::{
        SourceFlagsAttempt, SourceFlagsError, SourceFlagsParentLineage, SourceFlagsProgrammeId,
    },
    timeline::{CodecSourceIndex, FrontendCallbackIndex},
    transaction::CallbackLineageIdentity,
};
use std::sync::Arc;

const AUDIO_SAMPLES: usize = 3072;
const FFT_LENGTH: usize = 4096;
const FREQUENCY_BIN_COUNT: usize = 2048;
const BAND_COUNT: usize = 24;
const FFT_STAGE_COUNT: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrentFeaturesError {
    UnsupportedProfile,
    ProviderMismatch,
    ParentLineageMismatch,
    RootIdentityMismatch,
    RenderBoundaryMismatch,
    RenderPhaseMismatch,
    ReadAudioMissing,
    AudioShape,
    SourceShape,
    FlagDomain,
    BeforeVectorShape,
    BeforeScalarShape,
    NonFiniteBefore,
    StaleCallback,
    NonFiniteIntermediate,
}

/// Opaque linear band-major output; payload and callback binding have no public constructor.
#[must_use]
pub struct CurrentFeaturesPacket {
    _callback_index: FrontendCallbackIndex,
    _parent_transaction_id: CallbackLineageIdentity,
    source_axis: Arc<SourceAxisBinding>,
    features_f32_bits_by_band: Box<[Box<[u32]>]>,
}

impl CurrentFeaturesPacket {
    fn from_producer(
        callback_index: FrontendCallbackIndex,
        parent_transaction_id: CallbackLineageIdentity,
        source_axis: Arc<SourceAxisBinding>,
        features_f32_bits_by_band: Box<[Box<[u32]>]>,
    ) -> Self {
        Self {
            _callback_index: callback_index,
            _parent_transaction_id: parent_transaction_id,
            source_axis,
            features_f32_bits_by_band,
        }
    }

    pub(super) fn source_axis(&self) -> &SourceAxisBinding {
        &self.source_axis
    }

    pub(super) fn band_major_features(&self) -> &[Box<[u32]>] {
        &self.features_f32_bits_by_band
    }
}

pub(super) struct FeatureStateProposal {
    pub(super) source_vector_bits: Box<[u32]>,
    pub(super) source_scalar_bits: Box<[u32]>,
    pub(super) source_axis: Arc<SourceAxisBinding>,
    pub(super) before_generation: u64,
    pub(super) before_digest: SpatialControlDigest,
}

/// Linear output capability; no persistent packet, local commit, or parent decision is made here.
#[must_use]
pub(super) struct CurrentFeaturesProduced {
    pub(super) feature_state_proposal: FeatureStateProposal,
    pub(super) packet: CurrentFeaturesPacket,
    pub(super) source_flags: SourceFlagsPacket,
    pub(super) lineage: CurrentFeaturesLineage,
}

pub(super) struct CurrentFeaturesLineage {
    pub(super) programme_id: SourceFlagsProgrammeId,
    pub(super) attempt: SourceFlagsAttempt,
    pub(super) transaction_id: CallbackLineageIdentity,
    pub(super) callback_index: FrontendCallbackIndex,
    pub(super) profile: EncodingProfile,
    pub(super) provider_lineage: SpatialControlProviderLineage,
    pub(super) before_generation: u64,
    pub(super) before_digest: SpatialControlDigest,
    pub(super) callback_phase: CallbackPhase,
    pub(super) source_axis: Arc<SourceAxisBinding>,
    pub(super) render_stage_identity: Option<Arc<RenderStageIdentity>>,
    pub(super) spatial_control_render_identity: Option<Arc<SpatialControlRenderIdentity>>,
    pub(super) spatial_control_after_proposal_identity: Option<Arc<SpatialControlJoinMetadata>>,
    pub(super) audio_payload_sha256: Option<AudioPayloadSha256>,
    pub(super) audio_payload_byte_count: Option<u64>,
    pub(super) processor_receipt: FeatureProcessorMathProviderReceipt,
    pub(super) fft_receipt: FeatureFft4096MathProviderReceipt,
}

pub(super) fn validate_source_count(source_count: usize) -> Result<(), CurrentFeaturesError> {
    let supported = u16::try_from(source_count)
        .ok()
        .and_then(|count| SelectedSourceRowDimensions::for_active_source_count(count).ok())
        .is_some();
    if supported {
        Ok(())
    } else {
        Err(CurrentFeaturesError::SourceShape)
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn produce_current_features(
    owner: &SpatialControlMachine,
    programme_id: &SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: &CallbackLineageIdentity,
    callback_index: FrontendCallbackIndex,
    callback_phase: CallbackPhase,
    source_flags: SourceFlagsPacket,
    post_window_audio: Option<CurrentFeaturesAudioView>,
    processor: &ValidatedFeatureProcessorMathProvider,
    fft_provider: &ValidatedFeatureFft4096MathProvider,
) -> Result<CurrentFeaturesProduced, CurrentFeaturesError> {
    validate_provider_shape(processor, fft_provider)?;
    if transaction_id.programme_id() != programme_id
        || transaction_id.attempt_id() != attempt
        || transaction_id.callback_index() != callback_index
    {
        return Err(CurrentFeaturesError::ParentLineageMismatch);
    }
    validate_parent_identity(
        owner,
        programme_id,
        attempt,
        transaction_id,
        callback_index,
        callback_phase,
        &source_flags,
        post_window_audio.as_ref(),
    )?;

    let requires_audio =
        source_flags
            .payload()
            .iter()
            .any(|[processing_inactive, selection_suppressed]| {
                *processing_inactive == 0 && *selection_suppressed == 0
            });
    if requires_audio && post_window_audio.is_none() {
        return Err(CurrentFeaturesError::ReadAudioMissing);
    }
    if let Some(audio_view) = post_window_audio.as_ref() {
        validate_audio_shape(audio_view)?;
    }

    let source_axis = Arc::new(source_flags.source_axis().clone());
    if post_window_audio
        .as_ref()
        .is_some_and(|audio| source_axis.validate_handoff(audio.source_axis()).is_err())
    {
        return Err(CurrentFeaturesError::ParentLineageMismatch);
    }

    let before_generation = owner.generation();
    let before_digest = owner.digest();
    let before_state = owner.committed();
    let before_vector_bits = before_state.source_vector_bits();
    let before_scalar_bits = before_state.source_scalar_bits();
    let source_count = source_axis.source_count();
    validate_source_count(source_count)?;
    if source_flags.payload().len() != source_count
        || source_axis.callback_index() != callback_index
        || source_axis.transaction_id() != transaction_id
    {
        return Err(CurrentFeaturesError::SourceShape);
    }
    if before_vector_bits.len() != source_count * BAND_COUNT {
        return Err(CurrentFeaturesError::BeforeVectorShape);
    }
    if before_scalar_bits.len() != source_count {
        return Err(CurrentFeaturesError::BeforeScalarShape);
    }

    let mut before_vector = vec![[zero(); BAND_COUNT]; source_count];
    for source in 0..source_count {
        for band in 0..BAND_COUNT {
            before_vector[source][band] = FiniteF32::new(f32::from_bits(
                before_vector_bits[source * BAND_COUNT + band],
            ))
            .map_err(|_| CurrentFeaturesError::NonFiniteBefore)?;
        }
    }
    let mut before_scalar = vec![zero(); source_count];
    for source in 0..source_count {
        before_scalar[source] = FiniteF32::new(f32::from_bits(before_scalar_bits[source]))
            .map_err(|_| CurrentFeaturesError::NonFiniteBefore)?;
    }

    let fast_decay = finite_provider_word(processor.fast_decay_u32())?;
    let slow_retain = finite_provider_word(processor.slow_retain_u32())?;
    let accept = r32_sub(one(), slow_retain).map_err(nonfinite)?;
    // Histories advance once per source. Each FFT uses the original slot order.
    let compute = |(source, &source_id): (usize, &CodecSourceIndex)| {
        let mut vector = [0_u32; BAND_COUNT];
        let scalar;
        let [processing_inactive, selection_suppressed] = source_flags.payload()[source];
        if selection_suppressed == 1 {
            vector = [0; BAND_COUNT];
            scalar = 0;
        } else if processing_inactive == 1 {
            let mut next_scalar = zero();
            for band in 0..BAND_COUNT {
                let decayed =
                    r32_mul(before_vector[source][band], fast_decay).map_err(nonfinite)?;
                vector[band] = decayed.to_bits();
                next_scalar = r32_add(next_scalar, decayed).map_err(nonfinite)?;
            }
            scalar = next_scalar.to_bits();
        } else {
            let audio_view = post_window_audio
                .as_ref()
                .ok_or(CurrentFeaturesError::ReadAudioMissing)?;
            let observed = observed_features(audio_view, source_id, processor, fft_provider)?;
            let mut current_sum = zero();
            for observed_band in observed {
                current_sum = r32_add(current_sum, observed_band).map_err(nonfinite)?;
            }
            let current_threshold = threshold(current_sum)?;
            let before_threshold = threshold(before_scalar[source])?;
            let jump = current_sum.value() > before_threshold.value()
                || before_scalar[source].value() > current_threshold.value();

            let mut next_scalar = zero();
            for band in 0..BAND_COUNT {
                let next = if jump {
                    let decayed =
                        r32_mul(before_vector[source][band], fast_decay).map_err(nonfinite)?;
                    if observed[band].value() >= decayed.value() {
                        observed[band]
                    } else {
                        decayed
                    }
                } else {
                    let observed_part = r32_mul(observed[band], accept).map_err(nonfinite)?;
                    let retained_part =
                        r32_mul(before_vector[source][band], slow_retain).map_err(nonfinite)?;
                    r32_add(observed_part, retained_part).map_err(nonfinite)?
                };
                vector[band] = next.to_bits();
                next_scalar = r32_add(next_scalar, next).map_err(nonfinite)?;
            }
            scalar = next_scalar.to_bits();
        }
        Ok::<_, CurrentFeaturesError>((vector, scalar))
    };
    let sources = source_axis.source_ids();
    let active = source_flags
        .payload()
        .iter()
        .filter(|flags| **flags == [0, 0])
        .count();
    let rows: Vec<_> = if let Some(workers) = crate::dsp_workers::pool().filter(|_| active >= 3) {
        use rayon::prelude::*;
        workers.install(|| sources.par_iter().enumerate().map(compute).collect())
    } else {
        sources.iter().enumerate().map(compute).collect()
    };
    let (after_vector, after_scalar): (Vec<_>, Vec<_>) = rows
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .unzip();

    let features = (0..BAND_COUNT)
        .map(|band| {
            after_vector
                .iter()
                .map(|source| source[band])
                .collect::<Vec<_>>()
                .into_boxed_slice()
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();

    let render_stage_identity = post_window_audio
        .as_ref()
        .map(|view| Arc::clone(view.render_stage_identity()));
    let spatial_control_after_proposal_identity = post_window_audio
        .as_ref()
        .and_then(|view| view.spatial_control_after_proposal_identity().cloned());
    let spatial_control_render_identity = post_window_audio
        .as_ref()
        .map(|view| Arc::clone(view.spatial_control_render_identity()));
    let audio_payload_sha256 = post_window_audio
        .as_ref()
        .map(CurrentFeaturesAudioView::payload_sha256);
    let audio_payload_byte_count = post_window_audio
        .as_ref()
        .map(CurrentFeaturesAudioView::payload_byte_count);
    let lineage = CurrentFeaturesLineage {
        programme_id: programme_id.clone(),
        attempt,
        transaction_id: transaction_id.clone(),
        callback_index,
        profile: owner.profile(),
        provider_lineage: owner.provider_lineage(),
        before_generation,
        before_digest,
        callback_phase,
        source_axis: Arc::clone(&source_axis),
        render_stage_identity,
        spatial_control_render_identity,
        spatial_control_after_proposal_identity,
        audio_payload_sha256,
        audio_payload_byte_count,
        processor_receipt: *processor.receipt(),
        fft_receipt: *fft_provider.receipt(),
    };
    let feature_state_proposal = FeatureStateProposal {
        source_vector_bits: after_vector
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        source_scalar_bits: after_scalar
            .into_iter()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        source_axis: Arc::clone(&source_axis),
        before_generation,
        before_digest,
    };
    let packet = CurrentFeaturesPacket::from_producer(
        callback_index,
        transaction_id.clone(),
        Arc::clone(&source_axis),
        features,
    );
    drop(post_window_audio);
    Ok(CurrentFeaturesProduced {
        feature_state_proposal,
        packet,
        source_flags,
        lineage,
    })
}

fn validate_provider_shape(
    processor: &ValidatedFeatureProcessorMathProvider,
    fft_provider: &ValidatedFeatureFft4096MathProvider,
) -> Result<(), CurrentFeaturesError> {
    let stage_lengths = [2_u32, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];
    if processor.window_length() != AUDIO_SAMPLES as u32
        || processor.sample_rate() != 48_000
        || processor.fft_length() != FFT_LENGTH as u32
        || processor.component_count() != FREQUENCY_BIN_COUNT as u32
        || processor.vector_count() != BAND_COUNT as u32
        || processor.window_counts().iter().sum::<u32>() != FREQUENCY_BIN_COUNT as u32
        || fft_provider.fft_length() != FFT_LENGTH as u32
        || fft_provider.route() != FeatureFft4096MathRoute::RealFft4096Radix2MathV1
        || fft_provider.stages().len() != FFT_STAGE_COUNT
        || fft_provider
            .stages()
            .iter()
            .zip(stage_lengths)
            .any(|(stage, length)| {
                stage.length() != length || stage.roots_u32().len() != (length / 2) as usize
            })
    {
        return Err(CurrentFeaturesError::ProviderMismatch);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_parent_identity(
    owner: &SpatialControlMachine,
    programme_id: &SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: &CallbackLineageIdentity,
    callback_index: FrontendCallbackIndex,
    callback_phase: CallbackPhase,
    source_flags: &SourceFlagsPacket,
    audio: Option<&CurrentFeaturesAudioView>,
) -> Result<(), CurrentFeaturesError> {
    if callback_index != owner.committed().callback_index() {
        return Err(CurrentFeaturesError::StaleCallback);
    }
    let profile = owner.profile();
    resolve_selected_profile(&profile).map_err(|_| CurrentFeaturesError::UnsupportedProfile)?;
    owner
        .validate_profile_identity(&profile)
        .map_err(|_| CurrentFeaturesError::UnsupportedProfile)?;
    if let Some(audio) = audio {
        if !matches!(
            audio.boundary(),
            PostWindowBoundary::Installed48PostWindow48x3072
                | PostWindowBoundary::ActiveSourceAxisPostWindow3072
        ) {
            return Err(CurrentFeaturesError::RenderBoundaryMismatch);
        }
        if audio.callback_phase() != callback_phase {
            return Err(CurrentFeaturesError::RenderPhaseMismatch);
        }
        if audio.callback_index() != callback_index {
            return Err(CurrentFeaturesError::StaleCallback);
        }
        if audio.programme_id() != programme_id
            || audio.attempt() != attempt
            || audio.transaction_id() != transaction_id
        {
            return Err(CurrentFeaturesError::ParentLineageMismatch);
        }
        if audio.profile_identity() != profile {
            return Err(CurrentFeaturesError::UnsupportedProfile);
        }
        if audio.provider_lineage() != owner.provider_lineage() {
            return Err(CurrentFeaturesError::ProviderMismatch);
        }
        if audio.before_generation() != owner.generation()
            || audio.before_digest() != owner.digest()
        {
            return Err(CurrentFeaturesError::RootIdentityMismatch);
        }
    }
    let parent = SourceFlagsParentLineage::new(transaction_id.clone(), owner)
        .map_err(map_source_flags_error)?;
    source_flags
        .validate_parent_binding(&parent)
        .map_err(map_source_flags_error)?;
    let source_axis = source_flags.source_axis();
    if source_axis.transaction_id() != transaction_id
        || source_axis.callback_index() != callback_index
        || source_axis.source_count() != source_flags.payload().len()
    {
        return Err(CurrentFeaturesError::SourceShape);
    }
    let flags = source_flags.payload();
    if flags.is_empty() {
        return Err(CurrentFeaturesError::SourceShape);
    }
    if flags.iter().any(|pair| pair.iter().any(|flag| *flag > 1)) {
        return Err(CurrentFeaturesError::FlagDomain);
    }
    Ok(())
}

fn validate_audio_shape(audio: &CurrentFeaturesAudioView) -> Result<(), CurrentFeaturesError> {
    let source_count = audio.source_axis().source_count();
    if source_count == 0 || audio.sample_count() != source_count * AUDIO_SAMPLES {
        return Err(CurrentFeaturesError::AudioShape);
    }
    for source in audio.source_axis().source_ids() {
        if audio.source_samples(*source).map(<[_]>::len) != Some(AUDIO_SAMPLES) {
            return Err(CurrentFeaturesError::AudioShape);
        }
    }
    Ok(())
}

fn observed_features(
    audio: &CurrentFeaturesAudioView,
    source: CodecSourceIndex,
    processor: &ValidatedFeatureProcessorMathProvider,
    fft_provider: &ValidatedFeatureFft4096MathProvider,
) -> Result<[FiniteF32; BAND_COUNT], CurrentFeaturesError> {
    let samples = audio
        .source_samples(source)
        .ok_or(CurrentFeaturesError::AudioShape)?;
    // Assigned virtual sources can still carry digital silence. Squaring either
    // sign of zero and accumulating the bands from +0 yields +0 in every band.
    if samples.iter().all(|sample| sample.value() == 0.0) {
        return Ok([zero(); BAND_COUNT]);
    }
    let mut fft_values = vec![[0.0_f32; 2]; FFT_LENGTH];
    for (index, sample) in samples.iter().enumerate() {
        let reversed = index.reverse_bits() >> (usize::BITS - FFT_LENGTH.ilog2());
        fft_values[reversed][0] = sample.value();
    }
    fft4096(&mut fft_values, fft_provider)?;

    let mut weighted = [0.0_f32; FREQUENCY_BIN_COUNT];
    for ((weight, value), coefficient) in weighted[..FFT_LENGTH / 2]
        .iter_mut()
        .zip(&fft_values)
        .zip(processor.coefficient_u32())
    {
        let power = value[0] * value[0] + value[1] * value[1];
        *weight = power * f32::from_bits(*coefficient);
    }

    let mut grouped = [zero(); BAND_COUNT];
    let mut first_bin = 0;
    for (band_index, count) in processor.window_counts().iter().copied().enumerate() {
        let mut grouped_value = 0.0_f32;
        let group_end = first_bin + count as usize;
        for weighted_value in weighted[first_bin..group_end].iter().copied() {
            grouped_value += weighted_value;
        }
        grouped[band_index] = FiniteF32::new(grouped_value)
            .map_err(|_| CurrentFeaturesError::NonFiniteIntermediate)?;
        first_bin = group_end;
    }
    if first_bin != FREQUENCY_BIN_COUNT {
        return Err(CurrentFeaturesError::ProviderMismatch);
    }

    let alpha = finite_literal(0.0001);
    let forward = forward_smooth(&grouped, alpha)?;
    backward_smooth(&forward, &grouped, alpha)
}

fn fft4096(
    values: &mut [[f32; 2]],
    fft_provider: &ValidatedFeatureFft4096MathProvider,
) -> Result<(), CurrentFeaturesError> {
    for stage in fft_provider.stages() {
        super::fft_butterflies::apply_stage(values.as_flattened_mut(), stage.roots_u32());
    }
    if values.as_flattened().iter().any(|x| !x.is_finite()) {
        return Err(CurrentFeaturesError::NonFiniteIntermediate);
    }
    Ok(())
}

fn forward_smooth(
    grouped: &[FiniteF32; BAND_COUNT],
    alpha: FiniteF32,
) -> Result<[FiniteF32; BAND_COUNT], CurrentFeaturesError> {
    let mut forward = [zero(); BAND_COUNT];
    let mut previous = zero();
    for band_index in 0..BAND_COUNT {
        previous = r32_add(
            r32_mul(previous, alpha).map_err(nonfinite)?,
            grouped[band_index],
        )
        .map_err(nonfinite)?;
        forward[band_index] = previous;
    }
    Ok(forward)
}

fn backward_smooth(
    forward: &[FiniteF32; BAND_COUNT],
    grouped: &[FiniteF32; BAND_COUNT],
    alpha: FiniteF32,
) -> Result<[FiniteF32; BAND_COUNT], CurrentFeaturesError> {
    let mut observed = [zero(); BAND_COUNT];
    let mut previous = zero();
    for band_index in (0..BAND_COUNT).rev() {
        observed[band_index] = r32_add(
            r32_mul(previous, alpha).map_err(nonfinite)?,
            forward[band_index],
        )
        .map_err(nonfinite)?;
        previous = r32_add(
            r32_mul(previous, alpha).map_err(nonfinite)?,
            grouped[band_index],
        )
        .map_err(nonfinite)?;
    }
    Ok(observed)
}

fn threshold(value: FiniteF32) -> Result<FiniteF32, CurrentFeaturesError> {
    let scaled = r32_mul(value, finite_literal(100.0)).map_err(nonfinite)?;
    let bias = r32_mul(scaled, finite_literal(f32::from_bits(0x3400_0000))).map_err(nonfinite)?;
    r32_add(bias, scaled).map_err(nonfinite)
}

fn finite_provider_word(word: u32) -> Result<FiniteF32, CurrentFeaturesError> {
    FiniteF32::new(f32::from_bits(word)).map_err(|_| CurrentFeaturesError::ProviderMismatch)
}

fn finite_literal(value: f32) -> FiniteF32 {
    FiniteF32::new(value).expect("fixed CurrentFeatures binary32 literal is finite")
}

fn zero() -> FiniteF32 {
    finite_literal(0.0)
}

fn one() -> FiniteF32 {
    finite_literal(1.0)
}

fn nonfinite(_: R32Error) -> CurrentFeaturesError {
    CurrentFeaturesError::NonFiniteIntermediate
}

fn map_source_flags_error(error: SourceFlagsError) -> CurrentFeaturesError {
    match error {
        SourceFlagsError::StaleCallbackGeneration => CurrentFeaturesError::StaleCallback,
        SourceFlagsError::UnsupportedProfile => CurrentFeaturesError::UnsupportedProfile,
        SourceFlagsError::ProviderLineageMismatch => CurrentFeaturesError::ProviderMismatch,
        _ => CurrentFeaturesError::ParentLineageMismatch,
    }
}

#[cfg(test)]
#[allow(clippy::used_underscore_binding)]
impl CurrentFeaturesPacket {
    fn test_callback_index(&self) -> FrontendCallbackIndex {
        self._callback_index
    }

    fn test_payload_bits(&self) -> &[Box<[u32]>] {
        &self.features_f32_bits_by_band
    }
}

#[cfg(test)]
mod tests;
