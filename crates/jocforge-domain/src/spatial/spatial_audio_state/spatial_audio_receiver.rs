// pattern: Functional Core
#![allow(dead_code)] // The F04R proposal is not yet wired into the deferred F04C producer.

use super::{
    JocGeometryRowState, ModelState, RawBytes, RawWord, SpatialAudioJoinMetadata, TemporalState,
    descriptor_control::{
        DescriptorControlInputView, DescriptorGainInputView, DescriptorParameterInput,
        apply_descriptor_gain, stage_descriptor_control,
    },
    receiver_mode1::{
        ReceiverAnalysis, ReceiverSurroundOutput, analyze_receiver_lanes, process_surround_lanes,
    },
};
use crate::{
    joc::oamd::reader_target::{HistoryProducerGeneration, ReaderInvocationIndex},
    numeric::{FiniteF32, r32_add, r32_mul, r32_sub},
    profile::resolve_selected_profile,
    spatial::{
        DescriptorMathematicalProvider, QmfMathematicalFftProvider, QmfReceiverFunctionalProvider,
        ValidatedWindowQ15Provider,
        post_window_fanout::CallbackPhase,
        source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
    },
    timeline::{
        CodecSourceIndex, FrontendCallbackIndex, JocDispatchIndex, ReceiverCallbackCount,
        source_for_callback,
    },
    transaction::CallbackLineageIdentity,
};

const FRAME_SAMPLES: usize = 1_536;
const INPUT_ROWS: usize = 16;
const LANE_COUNT: usize = 6;
const TARGET_VALUES: usize = INPUT_ROWS * LANE_COUNT;
const BLOCK_SAMPLES: usize = 32;
const BLOCK_COUNT: usize = FRAME_SAMPLES / BLOCK_SAMPLES;
const RECEIVER_GENERATION_OFFSET: u32 = 1;
const RING_SAMPLES: usize = 225;
const RING_BYTES: usize = RING_SAMPLES * 4;
const TEMPORAL_LAYOUTS: [(i64, usize); 5] = [(1_217, 3), (63, 5), (900, 1), (256, 6), (896, 15)];

const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const FUNCTIONAL_PROVIDER_PATH: &str = "parameters/qmf-receiver-functional-v1.json";
const FUNCTIONAL_PROVIDER_BYTES: u64 = 20_325;
const FUNCTIONAL_PROVIDER_SHA256: &str =
    "30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c";
const WINDOW_PROVIDER_PATH: &str = "parameters/window-q15.json";
const WINDOW_PROVIDER_BYTES: u64 = 2_704;
const WINDOW_PROVIDER_SHA256: &str =
    "a12f9cbf91f1cf540c2eae77e4d236970defa70409f00ddf62ba87bfeafbb130";
const FFT_PROVIDER_PATH: &str = "parameters/qmf-mathematical-fft-v1.json";
const FFT_PROVIDER_BYTES: u64 = 6_983;
const FFT_PROVIDER_SHA256: &str =
    "e8d3fb1514a5ffd857740222eacc8995208a29c3447e48c1b63083af368c0431";
const DESCRIPTOR_PROVIDER_PATH: &str = "parameters/descriptor-mathematical.json";
const DESCRIPTOR_PROVIDER_BYTES: u64 = 4_861;
const DESCRIPTOR_PROVIDER_SHA256: &str =
    "a493fbdaf2dab30e940cae9976ed3305f3af083e5614ab0cbd6889c71a02b197";

const BLEND_SCALE_BITS: u32 = 0x3b80_0000; // R32(2^-8)
const TRANSFORM_SCALE_BITS: u32 = 0x4380_0000; // R32(256)
const PRECONDITION_SCALE_BITS: u32 = 0x3e80_0000; // R32(0.25)
const POST_S09_SCALE_BITS: u32 = 0x3f00_0000; // R32(0.5)

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::spatial) enum SpatialAudioReceiverError {
    ProviderMissing,
    ProviderIdentity,
    LineageTargetUpdates,
    StateShape,
    TemporalCursor,
    Type10Shape,
    TargetShape,
    InputNonfinite,
    NumericNonfinite,
}

impl SpatialAudioReceiverError {
    pub(in crate::spatial) const fn code(self) -> &'static str {
        match self {
            Self::ProviderMissing => "F04R_PROVIDER_MISSING",
            Self::ProviderIdentity => "F04R_PROVIDER_IDENTITY",
            Self::LineageTargetUpdates => "F04R_LINEAGE_TARGET_UPDATES",
            Self::StateShape => "F04R_STATE_SHAPE",
            Self::TemporalCursor => "F04R_TEMPORAL_CURSOR",
            Self::Type10Shape => "F04R_TYPE10_SHAPE",
            Self::TargetShape => "F04R_TARGET_SHAPE",
            Self::InputNonfinite => "F04R_INPUT_NONFINITE",
            Self::NumericNonfinite => "F04R_NUMERIC_NONFINITE",
        }
    }
}

pub(super) struct SpatialAudioReceiverProviderInput<'a> {
    pub(super) functional: Option<&'a QmfReceiverFunctionalProvider>,
    pub(super) blend: Option<&'a ValidatedWindowQ15Provider>,
    pub(super) fft: Option<&'a QmfMathematicalFftProvider>,
    pub(super) descriptor: Option<&'a DescriptorMathematicalProvider>,
    pub(super) validated_manifest_sha256: Option<&'a str>,
    pub(super) selected_manifest_sha256: Option<&'a str>,
}

pub(super) struct Type10ReceiverInput<'a> {
    parent: &'a SpatialAudioJoinMetadata,
    phase: CallbackPhase,
    callback_index: FrontendCallbackIndex,
    receiver_generation: ReceiverCallbackCount,
    component_origin_generation: FrontendCallbackIndex,
    lfe_origin_generation: FrontendCallbackIndex,
    samples: &'a [f32],
}

impl Type10ReceiverInput<'_> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn from_assembly<'a>(
        parent: &'a SpatialAudioJoinMetadata,
        phase: CallbackPhase,
        callback_index: FrontendCallbackIndex,
        receiver_generation: ReceiverCallbackCount,
        component_origin_generation: FrontendCallbackIndex,
        lfe_origin_generation: FrontendCallbackIndex,
        samples: &'a [f32],
    ) -> Type10ReceiverInput<'a> {
        Type10ReceiverInput {
            parent,
            phase,
            callback_index,
            receiver_generation,
            component_origin_generation,
            lfe_origin_generation,
            samples,
        }
    }

    #[cfg(test)]
    pub(super) fn from_direct_vector_for_test<'a>(
        parent: &'a SpatialAudioJoinMetadata,
        phase: CallbackPhase,
        callback_index: FrontendCallbackIndex,
        receiver_generation: ReceiverCallbackCount,
        samples: &'a [f32],
    ) -> Type10ReceiverInput<'a> {
        Type10ReceiverInput::from_assembly(
            parent,
            phase,
            callback_index,
            receiver_generation,
            callback_index,
            FrontendCallbackIndex::new(callback_index.value().saturating_sub(2)),
            samples,
        )
    }
}

#[derive(Clone, Copy)]
pub(super) struct TargetUpdatesLineage {
    pub(super) reader_invocation: ReaderInvocationIndex,
    pub(super) history_producer_generations: [HistoryProducerGeneration; INPUT_ROWS],
    pub(super) state_before_sha256: [u8; 32],
    pub(super) state_after_sha256: [u8; 32],
    pub(super) target_sha256: [u8; 32],
    pub(super) semantic_sha256: [u8; 32],
    pub(super) target_words: [u32; TARGET_VALUES],
}

pub(super) struct TargetUpdatesReceipt<'a> {
    parent: &'a SpatialAudioJoinMetadata,
    phase: CallbackPhase,
    callback_index: FrontendCallbackIndex,
    receiver_generation: ReceiverCallbackCount,
    lineage: TargetUpdatesLineage,
    values: &'a [f32],
}

impl TargetUpdatesReceipt<'_> {
    pub(super) fn from_reader_receipt<'a>(
        parent: &'a SpatialAudioJoinMetadata,
        phase: CallbackPhase,
        callback_index: FrontendCallbackIndex,
        receiver_generation: ReceiverCallbackCount,
        lineage: &TargetUpdatesLineage,
        values: &'a [f32],
    ) -> TargetUpdatesReceipt<'a> {
        TargetUpdatesReceipt {
            parent,
            phase,
            callback_index,
            receiver_generation,
            lineage: *lineage,
            values,
        }
    }

    #[cfg(test)]
    pub(super) fn from_direct_vector_for_test<'a>(
        parent: &'a SpatialAudioJoinMetadata,
        phase: CallbackPhase,
        callback_index: FrontendCallbackIndex,
        receiver_generation: ReceiverCallbackCount,
        values: &'a [f32],
    ) -> TargetUpdatesReceipt<'a> {
        let target_words =
            std::array::from_fn(|index| values.get(index).copied().unwrap_or_default().to_bits());
        TargetUpdatesReceipt::from_reader_receipt(
            parent,
            phase,
            callback_index,
            receiver_generation,
            &TargetUpdatesLineage {
                reader_invocation: ReaderInvocationIndex::new(u64::from(
                    callback_index.value().saturating_sub(2),
                )),
                history_producer_generations: [HistoryProducerGeneration::Callback(callback_index);
                    INPUT_ROWS],
                state_before_sha256: [0; 32],
                state_after_sha256: [0; 32],
                target_sha256: [0; 32],
                semantic_sha256: [0; 32],
                target_words,
            },
            values,
        )
    }
}

pub(super) struct SpatialAudioReceiverStageInput<'a> {
    pub(super) parent: &'a SpatialAudioJoinMetadata,
    pub(super) phase: CallbackPhase,
    pub(super) receiver_callback_count_before: ReceiverCallbackCount,
    pub(super) before: ModelState,
    pub(super) type10: Type10ReceiverInput<'a>,
    pub(super) target_updates: Option<TargetUpdatesReceipt<'a>>,
    pub(super) providers: SpatialAudioReceiverProviderInput<'a>,
}

#[cfg(test)]
#[derive(Debug)]
pub(super) struct ReceiverStageTrace {
    pub(super) preconditioned: Vec<f32>,
    pub(super) lane3_before_postfilter: Vec<f32>,
    pub(super) lane3_after_postfilter: Vec<f32>,
    pub(super) s09_input: Vec<f32>,
    pub(super) descriptor_input: Vec<f32>,
}

#[must_use]
#[derive(Debug)]
pub(super) struct SpatialAudioReceiverProposal {
    pub(super) programme_id: SourceFlagsProgrammeId,
    pub(super) attempt: SourceFlagsAttempt,
    pub(super) transaction_id: CallbackLineageIdentity,
    pub(super) spatial_before_generation: u64,
    pub(super) spatial_before_digest: super::SpatialAudioDigest,
    pub(super) phase: CallbackPhase,
    pub(super) callback_index: FrontendCallbackIndex,
    pub(super) source_index: CodecSourceIndex,
    pub(super) dispatch: JocDispatchIndex,
    pub(super) component_origin_generation: FrontendCallbackIndex,
    pub(super) lfe_origin_generation: FrontendCallbackIndex,
    pub(super) target_reader_invocation: ReaderInvocationIndex,
    pub(super) target_history_producer_generations: [HistoryProducerGeneration; INPUT_ROWS],
    pub(super) target_state_before_sha256: [u8; 32],
    pub(super) target_state_after_sha256: [u8; 32],
    pub(super) target_sha256: [u8; 32],
    pub(super) target_semantic_sha256: [u8; 32],
    pub(super) target_update_words: [u32; TARGET_VALUES],
    /// Same-execution descriptor result for downstream J05 admission. Consumers
    /// must use this selection rather than run descriptor/temporal3 again.
    pub(super) descriptor_selected_index: u8,
    pub(super) descriptor_target_gain_word: u32,
    pub(super) s09_return: [Box<[FiniteF32]>; 6],
    /// Preconditioned source rows 1..15 retained for the same-generation
    /// target QMF stage. These are audio values, not the 16x6 target gains.
    pub(super) target_qmf_input: [Box<[f32]>; 15],
    pub(super) geometry_rows_before: [JocGeometryRowState; 16],
    pub(super) geometry_rows_after: [JocGeometryRowState; 16],
    pub(super) s09_input: [Box<[FiniteF32]>; 6],
    pub(super) final_six: [Box<[FiniteF32]>; 6],
    pub(super) after: ModelState,
    #[cfg(test)]
    pub(super) trace: ReceiverStageTrace,
}

#[derive(Clone, Copy)]
struct ValidatedProviders<'a> {
    functional: &'a QmfReceiverFunctionalProvider,
    blend: &'a ValidatedWindowQ15Provider,
    fft: &'a QmfMathematicalFftProvider,
    descriptor: &'a DescriptorMathematicalProvider,
}

#[allow(dead_code)]
pub(super) fn stage_spatial_audio_receiver(
    mut input: SpatialAudioReceiverStageInput<'_>,
) -> Result<SpatialAudioReceiverProposal, SpatialAudioReceiverError> {
    let providers = validate_providers(&input.providers)?;
    let target_updates = input
        .target_updates
        .as_ref()
        .ok_or(SpatialAudioReceiverError::LineageTargetUpdates)?;
    validate_lineage(&input, target_updates)?;
    let callback_index = input.parent.callback_index();
    let source_index = source_for_callback(callback_index)
        .ok_or(SpatialAudioReceiverError::LineageTargetUpdates)?;
    let dispatch = source_index
        .dispatch()
        .map_err(|_| SpatialAudioReceiverError::LineageTargetUpdates)?;
    validate_state_shape(&input.before)?;
    validate_temporal_states(&input.before)?;
    if input.type10.samples.len() != INPUT_ROWS * FRAME_SAMPLES {
        return Err(SpatialAudioReceiverError::Type10Shape);
    }
    if target_updates.values.len() != TARGET_VALUES {
        return Err(SpatialAudioReceiverError::TargetShape);
    }
    if input.type10.samples.iter().any(|value| !value.is_finite())
        || target_updates.values.iter().any(|value| !value.is_finite())
    {
        return Err(SpatialAudioReceiverError::InputNonfinite);
    }
    if target_updates
        .values
        .iter()
        .zip(target_updates.lineage.target_words)
        .any(|(value, word)| value.to_bits() != word)
    {
        return Err(SpatialAudioReceiverError::LineageTargetUpdates);
    }

    let geometry_rows_before = input
        .before
        .row_states
        .each_ref()
        .map(JocGeometryRowState::from_owner);
    let preconditioned = precondition_rows(&mut input.before, input.type10.samples, providers)?;
    let target_qmf_input = target_qmf_rows(&preconditioned)?;
    install_target_updates(&mut input.before, target_updates.values);
    let accumulator = accumulate_rows(&mut input.before, &preconditioned, providers)?;
    let geometry_rows_after = input
        .before
        .row_states
        .each_ref()
        .map(JocGeometryRowState::from_owner);
    let mut receiver_lanes = transform_and_first_ring(&mut input.before, &accumulator, providers)?;
    #[cfg(test)]
    let lane3_before_postfilter = receiver_lanes[3].clone();
    filter_lfe_lane(&mut input.before, &mut receiver_lanes[3], providers)?;
    let s09_input = clone_lanes(&receiver_lanes);

    let finite_s09 = finite_lanes(&s09_input)?;
    let finite_s09_refs: [&[FiniteF32]; LANE_COUNT] =
        std::array::from_fn(|lane| finite_s09[lane].as_ref());
    let analysis = analyze_receiver_lanes(
        &input.before,
        &finite_s09_refs,
        providers.functional,
        providers.fft,
    )
    .map_err(|_| SpatialAudioReceiverError::NumericNonfinite)?;
    let surround = process_surround_lanes(
        &input.before,
        &analysis.spectra,
        providers.functional,
        providers.fft,
    )
    .map_err(|_| SpatialAudioReceiverError::NumericNonfinite)?;
    let surround_pcm = apply_receiver_after(&mut input.before, analysis, surround);
    receiver_lanes[4] = finite_values_to_f32(&surround_pcm[0]);
    receiver_lanes[5] = finite_values_to_f32(&surround_pcm[1]);

    exchange_temporal(
        &mut input.before.temporal_states[0],
        &mut receiver_lanes,
        &[0, 1, 2],
    )?;
    exchange_temporal(
        &mut input.before.temporal_states[1],
        &mut receiver_lanes,
        &[0, 1, 2, 4, 5],
    )?;
    exchange_temporal(
        &mut input.before.temporal_states[2],
        &mut receiver_lanes,
        &[3],
    )?;
    #[cfg(test)]
    let descriptor_input = flatten_lanes(&receiver_lanes);

    let control_snapshot = clone_lanes(&receiver_lanes);
    let control_lanes: [&[f32]; LANE_COUNT] =
        std::array::from_fn(|lane| control_snapshot[lane].as_slice());
    let descriptor_parameters = DescriptorParameterInput::from_validated_providers(
        Some(providers.functional),
        Some(providers.descriptor),
    );
    let control_input =
        DescriptorControlInputView::from_f04r_temporal2(input.parent, input.phase, &control_lanes);
    let selection = stage_descriptor_control(
        &input.before.descriptor_state,
        &control_input,
        &descriptor_parameters,
    )
    .map_err(map_descriptor_error)?;

    exchange_temporal(
        &mut input.before.temporal_states[3],
        &mut receiver_lanes,
        &[0, 1, 2, 3, 4, 5],
    )?;
    let gain_lanes: [&[f32]; LANE_COUNT] =
        std::array::from_fn(|lane| receiver_lanes[lane].as_slice());
    let gain_input = DescriptorGainInputView::from_f04r_temporal3(selection, &gain_lanes);
    let descriptor_proposal = apply_descriptor_gain(gain_input).map_err(map_descriptor_error)?;
    let (_, _, descriptor_selected_index, gained, descriptor_after) =
        descriptor_proposal.into_parts();
    let descriptor_target_gain_word = u32::from_le_bytes(descriptor_after.gain.0);
    input.before.descriptor_state = descriptor_after;
    for lane in 0..LANE_COUNT {
        receiver_lanes[lane] = finite_values_to_f32(&gained[lane]);
    }

    let delayed_and_scaled = post_s09_delay_and_scale(&mut input.before, &receiver_lanes)?;
    let final_six = second_ring(&mut input.before, &delayed_and_scaled)?;
    let s09_input = finite_lanes(&s09_input)?;
    let final_six = finite_lanes(&final_six)?;
    #[cfg(test)]
    let trace = ReceiverStageTrace {
        preconditioned,
        lane3_before_postfilter,
        lane3_after_postfilter: s09_input[3].iter().map(|sample| sample.value()).collect(),
        s09_input: flatten_finite_lanes(&s09_input),
        descriptor_input,
    };
    Ok(SpatialAudioReceiverProposal {
        programme_id: input.parent.programme_id().clone(),
        attempt: input.parent.attempt(),
        transaction_id: input.parent.transaction_id().clone(),
        spatial_before_generation: input.parent.before_generation(),
        spatial_before_digest: input.parent.before_digest(),
        phase: input.phase,
        callback_index,
        source_index,
        dispatch,
        component_origin_generation: input.type10.component_origin_generation,
        lfe_origin_generation: input.type10.lfe_origin_generation,
        target_reader_invocation: target_updates.lineage.reader_invocation,
        target_history_producer_generations: target_updates.lineage.history_producer_generations,
        target_state_before_sha256: target_updates.lineage.state_before_sha256,
        target_state_after_sha256: target_updates.lineage.state_after_sha256,
        target_sha256: target_updates.lineage.target_sha256,
        target_semantic_sha256: target_updates.lineage.semantic_sha256,
        target_update_words: target_updates.lineage.target_words,
        descriptor_selected_index,
        descriptor_target_gain_word,
        s09_return: gained,
        target_qmf_input,
        geometry_rows_before,
        geometry_rows_after,
        s09_input,
        final_six,
        after: input.before,
        #[cfg(test)]
        trace,
    })
}

fn target_qmf_rows(preconditioned: &[f32]) -> Result<[Box<[f32]>; 15], SpatialAudioReceiverError> {
    if preconditioned.len() != INPUT_ROWS * FRAME_SAMPLES {
        return Err(SpatialAudioReceiverError::Type10Shape);
    }
    Ok(std::array::from_fn(|object| {
        let row = object + 1;
        preconditioned[row * FRAME_SAMPLES..(row + 1) * FRAME_SAMPLES]
            .to_vec()
            .into_boxed_slice()
    }))
}

fn validate_providers<'a>(
    input: &SpatialAudioReceiverProviderInput<'a>,
) -> Result<ValidatedProviders<'a>, SpatialAudioReceiverError> {
    let (
        Some(functional),
        Some(blend),
        Some(fft),
        Some(descriptor),
        Some(validated_manifest_sha),
        Some(approved_source_manifest_sha),
    ) = (
        input.functional,
        input.blend,
        input.fft,
        input.descriptor,
        input.validated_manifest_sha256,
        input.selected_manifest_sha256,
    )
    else {
        return Err(SpatialAudioReceiverError::ProviderMissing);
    };
    if approved_source_manifest_sha != PACKAGE_MANIFEST_SHA256
        || !lineage_matches(
            functional.receipt().validated_manifest_lineage(),
            validated_manifest_sha,
            approved_source_manifest_sha,
            FUNCTIONAL_PROVIDER_PATH,
            FUNCTIONAL_PROVIDER_BYTES,
            FUNCTIONAL_PROVIDER_SHA256,
        )
        || functional.receipt().provider_file_bytes() != FUNCTIONAL_PROVIDER_BYTES
        || functional.receipt().provider_file_sha256().to_hex() != FUNCTIONAL_PROVIDER_SHA256
        || !lineage_matches(
            blend.receipt().validated_manifest_lineage(),
            validated_manifest_sha,
            approved_source_manifest_sha,
            WINDOW_PROVIDER_PATH,
            WINDOW_PROVIDER_BYTES,
            WINDOW_PROVIDER_SHA256,
        )
        || blend.receipt().provider_file_bytes() != WINDOW_PROVIDER_BYTES
        || blend.receipt().provider_file_sha256().to_hex() != WINDOW_PROVIDER_SHA256
        || !lineage_matches(
            fft.receipt().validated_manifest_lineage(),
            validated_manifest_sha,
            approved_source_manifest_sha,
            FFT_PROVIDER_PATH,
            FFT_PROVIDER_BYTES,
            FFT_PROVIDER_SHA256,
        )
        || fft.receipt().provider_file_bytes() != FFT_PROVIDER_BYTES
        || fft.receipt().provider_file_sha256().to_hex() != FFT_PROVIDER_SHA256
        || fft.receipt().route() != "QMF_FFT_RADIX2_MATH_V1"
        || !lineage_matches(
            descriptor.receipt().validated_manifest_lineage(),
            validated_manifest_sha,
            approved_source_manifest_sha,
            DESCRIPTOR_PROVIDER_PATH,
            DESCRIPTOR_PROVIDER_BYTES,
            DESCRIPTOR_PROVIDER_SHA256,
        )
        || descriptor.receipt().provider_file_bytes() != DESCRIPTOR_PROVIDER_BYTES
        || descriptor.receipt().provider_file_sha256().to_hex() != DESCRIPTOR_PROVIDER_SHA256
        || descriptor.receipt().private_vendor_key_included()
        || functional.qwin_synthesis_words().len() != 640
        || functional.band0_real_mode1_words().len() != 21
        || functional.band0_imag_mode1_words().len() != 21
        || functional.lfe_post_filter_words().len() != 20
        || functional.descriptor_threshold_words().len() != 127
        || blend.receiver_blend32().len() != 32
        || descriptor.gain_words().len() != 128
        || !functional
            .scalar_words()
            .iter()
            .all(|word| f32::from_bits(*word).is_finite())
        || !functional
            .qwin_synthesis_words()
            .iter()
            .chain(functional.band0_real_mode1_words())
            .chain(functional.band0_imag_mode1_words())
            .chain(functional.lfe_post_filter_words())
            .chain(functional.descriptor_threshold_words())
            .all(|word| f32::from_bits(*word).is_finite())
        || !descriptor
            .gain_words()
            .iter()
            .all(|word| f32::from_bits(*word).is_finite() && f32::from_bits(*word) > 0.0)
        || !fft
            .rotation_words()
            .iter()
            .all(|word| f32::from_bits(*word).is_finite())
        || fft.stages().len() != 6
        || !fft
            .stages()
            .iter()
            .zip([2_u8, 4, 8, 16, 32, 64])
            .all(|(stage, length)| {
                stage.length() == length
                    && stage.roots().len() == usize::from(length) / 2
                    && stage
                        .roots()
                        .iter()
                        .flatten()
                        .all(|word| f32::from_bits(*word).is_finite())
            })
    {
        return Err(SpatialAudioReceiverError::ProviderIdentity);
    }
    Ok(ValidatedProviders {
        functional,
        blend,
        fft,
        descriptor,
    })
}

fn lineage_matches(
    lineage: &crate::spatial::F04ParameterProviderLineage,
    validated_manifest_sha256: &str,
    approved_source_manifest_sha256: &str,
    path: &str,
    bytes: u64,
    sha256: &str,
) -> bool {
    lineage.manifest_sha256().to_hex() == validated_manifest_sha256
        && lineage.approved_source_manifest_sha256() == Some(approved_source_manifest_sha256)
        && lineage.provider_path() == path
        && lineage.manifest_entry_bytes() == bytes
        && lineage.manifest_entry_sha256().to_hex() == sha256
}

fn validate_lineage(
    input: &SpatialAudioReceiverStageInput<'_>,
    target_updates: &TargetUpdatesReceipt<'_>,
) -> Result<(), SpatialAudioReceiverError> {
    let callback = input.parent.callback_index().value();
    let receiver_before = callback
        .checked_sub(2)
        .ok_or(SpatialAudioReceiverError::LineageTargetUpdates)?;
    let receiver_after = callback
        .checked_sub(RECEIVER_GENERATION_OFFSET)
        .ok_or(SpatialAudioReceiverError::LineageTargetUpdates)?;
    if resolve_selected_profile(&input.parent.profile_identity()).is_err()
        || input.receiver_callback_count_before.value() != u64::from(receiver_before)
        || input.type10.callback_index != input.parent.callback_index()
        || target_updates.callback_index != input.parent.callback_index()
        || input.type10.receiver_generation.value() != u64::from(receiver_after)
        || target_updates.receiver_generation.value() != u64::from(receiver_after)
        || input.type10.component_origin_generation != input.parent.callback_index()
        || input.type10.lfe_origin_generation.value() != receiver_before
        || target_updates.lineage.reader_invocation.value() != u64::from(receiver_before)
        || input.type10.phase != input.phase
        || target_updates.phase != input.phase
        || !same_parent(input.parent, input.type10.parent)
        || !same_parent(input.parent, target_updates.parent)
    {
        return Err(SpatialAudioReceiverError::LineageTargetUpdates);
    }
    Ok(())
}

fn same_parent(left: &SpatialAudioJoinMetadata, right: &SpatialAudioJoinMetadata) -> bool {
    left.programme_id() == right.programme_id()
        && left.attempt() == right.attempt()
        && left.transaction_id() == right.transaction_id()
        && left.callback_index() == right.callback_index()
        && left.before_generation() == right.before_generation()
        && left.before_digest() == right.before_digest()
        && left.profile_identity() == right.profile_identity()
}

fn validate_state_shape(state: &ModelState) -> Result<(), SpatialAudioReceiverError> {
    let valid_rows = state.row_states.iter().all(|row| {
        row.current
            .iter()
            .chain(&row.target)
            .chain(&row.slope)
            .all(|word| raw_word_is_finite(*word))
    });
    let valid_temporal_owner_count = state.temporal_states.len() == TEMPORAL_LAYOUTS.len();
    let valid_descriptor = raw_word_is_finite(state.descriptor_state.tail_peak)
        && raw_word_is_finite(state.descriptor_state.smoothed)
        && raw_word_is_finite(state.descriptor_state.reference)
        && raw_word_is_finite(state.descriptor_state.gain)
        && state.descriptor_state.hold_count >= 0
        && u32::try_from(state.descriptor_state.hold_count).is_ok()
        && i32::try_from(state.descriptor_state.exponent).is_ok();
    if state
        .filter_state
        .iter()
        .any(|word| !raw_word_is_finite(*word))
        || !valid_rows
        || state
            .transform_delay
            .iter()
            .any(|bytes| !valid_f32_bytes(bytes, BLOCK_SAMPLES * 4))
        || state.first_ring.iter().any(|ring| !valid_ring(ring))
        || !valid_f32_bytes(&state.post_filter_state, 8 * 4)
        || state
            .analysis_history
            .iter()
            .flatten()
            .any(|bytes| !valid_f32_bytes(bytes, 64 * 4))
        || state
            .qmf_delay_history
            .iter()
            .flatten()
            .any(|bytes| !valid_f32_bytes(bytes, 128 * 4))
        || state
            .qmf_fir_history
            .iter()
            .any(|bytes| !valid_f32_bytes(bytes, 40 * 4))
        || state
            .synthesis_history
            .iter()
            .any(|bytes| !valid_f32_bytes(bytes, 576 * 4))
        || !valid_temporal_owner_count
        || !valid_descriptor
        || state
            .post_s09_delay
            .iter()
            .any(|bytes| !valid_f32_bytes(bytes, BLOCK_SAMPLES * 4))
        || state.second_ring.iter().any(|ring| !valid_ring(ring))
    {
        return Err(SpatialAudioReceiverError::StateShape);
    }
    Ok(())
}

fn validate_temporal_states(state: &ModelState) -> Result<(), SpatialAudioReceiverError> {
    for (temporal, (expected_span, expected_lanes)) in
        state.temporal_states.iter().zip(TEMPORAL_LAYOUTS)
    {
        if temporal.span != expected_span
            || temporal.stride_bytes != 4
            || temporal.buffers.len() != expected_lanes
            || temporal.buffers.iter().any(|buffer| {
                !valid_f32_bytes(
                    buffer,
                    usize::try_from(expected_span).unwrap_or_default() * 8,
                )
            })
            || !cursor_in_range(temporal.index_28, expected_span)
            || !cursor_in_range(temporal.index_32, expected_span)
        {
            return Err(SpatialAudioReceiverError::TemporalCursor);
        }
    }
    Ok(())
}

fn cursor_in_range(cursor: i64, span: i64) -> bool {
    (0..=2 * span).contains(&cursor)
}

fn valid_ring(ring: &super::RingState) -> bool {
    let Ok(ring_sample_count) = i64::try_from(RING_SAMPLES) else {
        return false;
    };
    valid_f32_bytes(&ring.words, RING_BYTES)
        && (0..ring_sample_count).contains(&ring.write_index)
        && (0..ring_sample_count).contains(&ring.read_index)
}

fn valid_f32_bytes(bytes: &RawBytes, expected_len: usize) -> bool {
    bytes.bytes.len() == expected_len
        && bytes.bytes.chunks_exact(4).all(|word| {
            f32::from_bits(u32::from_le_bytes([word[0], word[1], word[2], word[3]])).is_finite()
        })
}

fn raw_word_is_finite(word: RawWord) -> bool {
    f32::from_bits(u32::from_le_bytes(word.0)).is_finite()
}

fn precondition_rows(
    state: &mut ModelState,
    input: &[f32],
    providers: ValidatedProviders<'_>,
) -> Result<Vec<f32>, SpatialAudioReceiverError> {
    let scalars = providers.functional.scalars();
    let alpha = finite_word(f32::from_bits(scalars.preconditioning_alpha()))?;
    let beta = finite_word(f32::from_bits(scalars.preconditioning_beta()))?;
    let tiny = f32::from_bits(scalars.preconditioning_tiny_strict());
    let scale = finite_word(f32::from_bits(PRECONDITION_SCALE_BITS))?;
    let mut output = vec![0.0_f32; INPUT_ROWS * FRAME_SAMPLES];
    for row in 0..INPUT_ROWS {
        let mut scalar = raw_word_value(state.filter_state[row])?;
        for sample_index in 0..FRAME_SAMPLES {
            let input_value = finite_word(input[row * FRAME_SAMPLES + sample_index])?;
            let scaled_input = r32_mul(alpha, input_value).map_err(map_numeric_error)?;
            let residual = r32_sub(scaled_input, scalar).map_err(map_numeric_error)?;
            let clipped = finite_word(residual.value().clamp(-1.0, 1.0))?;
            let emitted = r32_mul(clipped, scale).map_err(map_numeric_error)?;
            let beta_residual = r32_mul(residual, beta).map_err(map_numeric_error)?;
            scalar = r32_add(beta_residual, scalar).map_err(map_numeric_error)?;
            output[row * FRAME_SAMPLES + sample_index] = emitted.value();
            if (sample_index + 1) % 256 == 0 && scalar.value() > -tiny && scalar.value() < tiny {
                scalar = finite_word(0.0)?;
            }
        }
        state.filter_state[row] = raw_word(scalar.value());
    }
    Ok(output)
}

fn install_target_updates(state: &mut ModelState, target_updates: &[f32]) {
    for row in 0..INPUT_ROWS {
        for lane in 0..LANE_COUNT {
            state.row_states[row].target[lane] = raw_word(target_updates[row * LANE_COUNT + lane]);
        }
    }
}

fn accumulate_rows(
    state: &mut ModelState,
    preconditioned: &[f32],
    providers: ValidatedProviders<'_>,
) -> Result<[Vec<f32>; LANE_COUNT], SpatialAudioReceiverError> {
    let scalars = providers.functional.scalars();
    let magnitude_threshold =
        finite_word(f32::from_bits(scalars.accumulator_magnitude_threshold()))?;
    let delta_threshold = finite_word(f32::from_bits(scalars.accumulator_delta_threshold()))?;
    let smoothing = finite_word(f32::from_bits(scalars.accumulator_smoothing()))?;
    let mut accumulator: [Vec<f32>; LANE_COUNT] =
        std::array::from_fn(|_| vec![0.0_f32; FRAME_SAMPLES]);

    for block in 0..BLOCK_COUNT {
        let active = u32::from(block == 0);
        let duration = u32::try_from(BLOCK_COUNT - block)
            .map_err(|_| SpatialAudioReceiverError::NumericNonfinite)?;
        for row_index in 0..INPUT_ROWS {
            let row = &mut state.row_states[row_index];
            for (lane, lane_accumulator) in accumulator.iter_mut().enumerate() {
                let current = raw_word_value(row.current[lane])?;
                let target = raw_word_value(row.target[lane])?;
                let mut slope = raw_word_value(row.slope[lane])?;
                let mut remaining = u32::from_le_bytes(row.remaining[lane].0);
                if target.value().abs().max(current.value().abs()) < magnitude_threshold.value() {
                    continue;
                }

                let delta = r32_sub(target, current).map_err(map_numeric_error)?;
                let mut next_current = current;
                if delta.value().abs() >= delta_threshold.value() {
                    if active != 0 {
                        remaining = duration;
                        if duration >= 1 {
                            slope = r32_mul(delta, smoothing).map_err(map_numeric_error)?;
                        }
                    }
                } else {
                    remaining = 0;
                    next_current = target;
                }

                let start = block * BLOCK_SAMPLES;
                if remaining != 0 {
                    for sample_in_block in 0..BLOCK_SAMPLES {
                        let window =
                            finite_word((sample_in_block + 1) as f32 / BLOCK_SAMPLES as f32)?;
                        let weighted_slope = r32_mul(window, slope).map_err(map_numeric_error)?;
                        let coefficient =
                            r32_add(weighted_slope, next_current).map_err(map_numeric_error)?;
                        let sample_index = start + sample_in_block;
                        let input =
                            finite_word(preconditioned[row_index * FRAME_SAMPLES + sample_index])?;
                        let product = r32_mul(coefficient, input).map_err(map_numeric_error)?;
                        let old_output = finite_word(lane_accumulator[sample_index])?;
                        lane_accumulator[sample_index] = r32_add(product, old_output)
                            .map_err(map_numeric_error)?
                            .value();
                    }
                    next_current = r32_add(slope, next_current).map_err(map_numeric_error)?;
                    remaining -= 1;
                } else {
                    for sample_in_block in 0..BLOCK_SAMPLES {
                        let sample_index = start + sample_in_block;
                        let input =
                            finite_word(preconditioned[row_index * FRAME_SAMPLES + sample_index])?;
                        let product = r32_mul(target, input).map_err(map_numeric_error)?;
                        let old_output = finite_word(lane_accumulator[sample_index])?;
                        lane_accumulator[sample_index] = r32_add(product, old_output)
                            .map_err(map_numeric_error)?
                            .value();
                    }
                    next_current = target;
                }
                row.current[lane] = raw_word(next_current.value());
                row.slope[lane] = raw_word(slope.value());
                row.remaining[lane] = RawWord(remaining.to_le_bytes());
            }
        }
    }
    Ok(accumulator)
}

fn transform_and_first_ring(
    state: &mut ModelState,
    accumulator: &[Vec<f32>; LANE_COUNT],
    providers: ValidatedProviders<'_>,
) -> Result<[Vec<f32>; LANE_COUNT], SpatialAudioReceiverError> {
    let blend = providers.blend.receiver_blend32();
    let blend_scale = finite_word(f32::from_bits(BLEND_SCALE_BITS))?;
    let transform_scale = finite_word(f32::from_bits(TRANSFORM_SCALE_BITS))?;
    let mut blend_words = Vec::with_capacity(BLOCK_SAMPLES);
    for word in blend {
        blend_words.push(finite_word(f32::from(*word) / 32_768.0)?);
    }
    let mut transformed: [Vec<f32>; LANE_COUNT] =
        std::array::from_fn(|_| vec![0.0_f32; FRAME_SAMPLES]);
    for lane in 0..LANE_COUNT {
        for block in 0..BLOCK_COUNT {
            let start = block * BLOCK_SAMPLES;
            let mut old_delay = [finite_word(0.0)?; BLOCK_SAMPLES];
            for (index, sample) in old_delay.iter_mut().enumerate() {
                *sample = finite_word(read_f32(&state.transform_delay[lane].bytes, index * 4))?;
            }
            for sample in 0..BLOCK_SAMPLES {
                write_f32(
                    &mut state.transform_delay[lane].bytes,
                    sample * 4,
                    accumulator[lane][start + sample],
                );
                let left_window = blend_words[BLOCK_SAMPLES - 1 - sample];
                let right_window = blend_words[sample];
                let left = r32_mul(
                    r32_mul(old_delay[sample], blend_scale).map_err(map_numeric_error)?,
                    left_window,
                )
                .map_err(map_numeric_error)?;
                let right = r32_mul(
                    r32_mul(old_delay[sample], blend_scale).map_err(map_numeric_error)?,
                    right_window,
                )
                .map_err(map_numeric_error)?;
                transformed[lane][start + sample] = r32_mul(
                    r32_add(left, right).map_err(map_numeric_error)?,
                    transform_scale,
                )
                .map_err(map_numeric_error)?
                .value();
            }
        }
    }
    apply_ring_states(&mut state.first_ring, &mut transformed)?;
    Ok(transformed)
}

fn apply_ring_states(
    rings: &mut [super::RingState; LANE_COUNT],
    lanes: &mut [Vec<f32>; LANE_COUNT],
) -> Result<(), SpatialAudioReceiverError> {
    let ring_sample_count =
        i64::try_from(RING_SAMPLES).map_err(|_| SpatialAudioReceiverError::StateShape)?;
    for lane in 0..LANE_COUNT {
        let ring = &mut rings[lane];
        for sample in &mut lanes[lane] {
            let clamped = sample.clamp(-1.0, 1.0);
            let write_index = usize::try_from(ring.write_index)
                .map_err(|_| SpatialAudioReceiverError::StateShape)?;
            let read_index = usize::try_from(ring.read_index)
                .map_err(|_| SpatialAudioReceiverError::StateShape)?;
            write_f32(&mut ring.words.bytes, write_index * 4, clamped);
            *sample = read_f32(&ring.words.bytes, read_index * 4);
            ring.write_index = (ring.write_index + 1) % ring_sample_count;
            ring.read_index = (ring.read_index + 1) % ring_sample_count;
        }
    }
    Ok(())
}

fn filter_lfe_lane(
    state: &mut ModelState,
    lane: &mut [f32],
    providers: ValidatedProviders<'_>,
) -> Result<(), SpatialAudioReceiverError> {
    if lane.len() != FRAME_SAMPLES {
        return Err(SpatialAudioReceiverError::NumericNonfinite);
    }
    let coefficient_words = providers.functional.lfe_post_filter_words();
    let mut section_state: [[f32; 2]; 4] = std::array::from_fn(|section| {
        let offset = section * 8;
        [
            read_f32(&state.post_filter_state.bytes, offset),
            read_f32(&state.post_filter_state.bytes, offset + 4),
        ]
    });

    for chunk in 0..6 {
        let start = chunk * 256;
        let mut section_input = lane[start..start + 256].to_vec();
        for (section, saved_section_state) in section_state.iter_mut().enumerate() {
            let coefficient_offset = section * 5;
            let a0 = finite_word(f32::from_bits(coefficient_words[coefficient_offset]))?;
            let a1 = finite_word(f32::from_bits(coefficient_words[coefficient_offset + 1]))?;
            let b0 = finite_word(f32::from_bits(coefficient_words[coefficient_offset + 2]))?;
            let b1 = finite_word(f32::from_bits(coefficient_words[coefficient_offset + 3]))?;
            let b2 = finite_word(f32::from_bits(coefficient_words[coefficient_offset + 4]))?;
            let numerator_first = r32_add(r32_mul(b0, a0).map_err(map_numeric_error)?, b1)
                .map_err(map_numeric_error)?;
            let numerator_second = r32_add(r32_mul(b0, a1).map_err(map_numeric_error)?, b2)
                .map_err(map_numeric_error)?;
            let a0_squared = r32_mul(a0, a0).map_err(map_numeric_error)?;
            let denominator_numerator = [
                1.0_f32,
                a0.value(),
                r32_add(a0_squared, a1).map_err(map_numeric_error)?.value(),
                r32_mul(a1, a0).map_err(map_numeric_error)?.value(),
            ];
            let denominator_feedback = [0.0_f32, 1.0_f32, a0.value(), a1.value()];
            let numerator_first_times_denominator_leading =
                r32_mul(numerator_first, a0).map_err(map_numeric_error)?;
            let numerator_output_coefficients = [
                b0.value(),
                numerator_first.value(),
                r32_add(numerator_first_times_denominator_leading, numerator_second)
                    .map_err(map_numeric_error)?
                    .value(),
                r32_mul(numerator_first, a1)
                    .map_err(map_numeric_error)?
                    .value(),
            ];
            let numerator_input_coefficients = [
                0.0_f32,
                b0.value(),
                numerator_first.value(),
                numerator_second.value(),
            ];
            let [mut h0, mut h1] = *saved_section_state;
            let mut section_output = vec![0.0_f32; 256];
            for pair in 0..(256 / 2) {
                let x0 = finite_word(section_input[pair * 2])?;
                let x1 = finite_word(section_input[pair * 2 + 1])?;
                let h0_word = finite_word(h0)?;
                let h1_word = finite_word(h1)?;
                let mut value = [0.0_f32; 4];
                for index in 0..4 {
                    let x1_q = r32_mul(x1, finite_word(numerator_input_coefficients[index])?)
                        .map_err(map_numeric_error)?;
                    let x0_p = r32_mul(x0, finite_word(numerator_output_coefficients[index])?)
                        .map_err(map_numeric_error)?;
                    let first = r32_add(x1_q, x0_p).map_err(map_numeric_error)?;
                    let h0_u = r32_mul(h0_word, finite_word(denominator_numerator[index])?)
                        .map_err(map_numeric_error)?;
                    let second = r32_add(first, h0_u).map_err(map_numeric_error)?;
                    let h1_v = r32_mul(h1_word, finite_word(denominator_feedback[index])?)
                        .map_err(map_numeric_error)?;
                    value[index] = r32_add(second, h1_v).map_err(map_numeric_error)?.value();
                }
                section_output[pair * 2] = value[0];
                section_output[pair * 2 + 1] = value[1];
                h0 = value[2];
                h1 = value[3];
            }
            *saved_section_state = [h0, h1];
            section_input = section_output;
        }
        lane[start..start + 256].copy_from_slice(&section_input);
    }

    for (section, [h0, h1]) in section_state.into_iter().enumerate() {
        write_f32(&mut state.post_filter_state.bytes, section * 8, h0);
        write_f32(&mut state.post_filter_state.bytes, section * 8 + 4, h1);
    }
    Ok(())
}

fn apply_receiver_after(
    state: &mut ModelState,
    analysis: ReceiverAnalysis,
    surround: ReceiverSurroundOutput,
) -> [Box<[FiniteF32]>; 2] {
    state.analysis_history = analysis.history_after;
    state.qmf_delay_history = surround.delay_after;
    state.qmf_fir_history = surround.fir_after;
    state.synthesis_history = surround.synthesis_after;
    surround.pcm
}

fn exchange_temporal(
    temporal: &mut TemporalState,
    lanes: &mut [Vec<f32>; LANE_COUNT],
    lane_indexes: &[usize],
) -> Result<(), SpatialAudioReceiverError> {
    let span =
        usize::try_from(temporal.span).map_err(|_| SpatialAudioReceiverError::TemporalCursor)?;
    let doubled_span = span
        .checked_mul(2)
        .ok_or(SpatialAudioReceiverError::TemporalCursor)?;
    let count = span.min(FRAME_SAMPLES);
    let read = usize::try_from(temporal.index_28)
        .map_err(|_| SpatialAudioReceiverError::TemporalCursor)?;
    let write = usize::try_from(temporal.index_32)
        .map_err(|_| SpatialAudioReceiverError::TemporalCursor)?;
    let write_first = count.min(doubled_span - write);
    let read_first = count.min(doubled_span - read);

    for (temporal_lane, lane_index) in lane_indexes.iter().copied().enumerate() {
        let original_input = lanes[lane_index].clone();
        let buffer = &mut temporal.buffers[temporal_lane].bytes;
        for offset in 0..count {
            let buffer_index = if offset < write_first {
                write + offset
            } else {
                offset - write_first
            };
            let sample_index = FRAME_SAMPLES - count + offset;
            write_f32(buffer, buffer_index * 4, original_input[sample_index]);
        }
        let mut output = vec![0.0_f32; FRAME_SAMPLES];
        for (offset, output_sample) in output.iter_mut().take(count).enumerate() {
            let buffer_index = if offset < read_first {
                read + offset
            } else {
                offset - read_first
            };
            *output_sample = read_f32(buffer, buffer_index * 4);
        }
        output[count..].copy_from_slice(&original_input[..FRAME_SAMPLES - count]);
        lanes[lane_index] = output;
    }

    let next_write = if count == write_first {
        write + count
    } else {
        count - write_first
    };
    let next_read = if count == read_first {
        read + count
    } else {
        count - read_first
    };
    temporal.index_32 =
        i64::try_from(next_write).map_err(|_| SpatialAudioReceiverError::TemporalCursor)?;
    temporal.index_28 =
        i64::try_from(next_read).map_err(|_| SpatialAudioReceiverError::TemporalCursor)?;
    Ok(())
}

fn post_s09_delay_and_scale(
    state: &mut ModelState,
    gained_lanes: &[Vec<f32>; LANE_COUNT],
) -> Result<[Vec<f32>; LANE_COUNT], SpatialAudioReceiverError> {
    let half = finite_word(f32::from_bits(POST_S09_SCALE_BITS))?;
    let mut output: [Vec<f32>; LANE_COUNT] = std::array::from_fn(|_| vec![0.0_f32; FRAME_SAMPLES]);
    for lane in 0..LANE_COUNT {
        for index in 0..BLOCK_SAMPLES {
            output[lane][index] = read_f32(&state.post_s09_delay[lane].bytes, index * 4);
            write_f32(
                &mut state.post_s09_delay[lane].bytes,
                index * 4,
                gained_lanes[lane][FRAME_SAMPLES - BLOCK_SAMPLES + index],
            );
        }
        output[lane][BLOCK_SAMPLES..]
            .copy_from_slice(&gained_lanes[lane][..FRAME_SAMPLES - BLOCK_SAMPLES]);
        for sample in &mut output[lane] {
            *sample = r32_mul(finite_word(*sample)?, half)
                .map_err(map_numeric_error)?
                .value();
        }
    }
    Ok(output)
}

fn second_ring(
    state: &mut ModelState,
    input: &[Vec<f32>; LANE_COUNT],
) -> Result<[Vec<f32>; LANE_COUNT], SpatialAudioReceiverError> {
    let mut output = clone_lanes(input);
    apply_ring_states(&mut state.second_ring, &mut output)?;
    Ok(output)
}

fn finite_lanes(
    lanes: &[Vec<f32>; LANE_COUNT],
) -> Result<[Box<[FiniteF32]>; LANE_COUNT], SpatialAudioReceiverError> {
    let mut converted: [Vec<FiniteF32>; LANE_COUNT] =
        std::array::from_fn(|_| Vec::with_capacity(FRAME_SAMPLES));
    for lane in 0..LANE_COUNT {
        for value in &lanes[lane] {
            converted[lane].push(finite_word(*value)?);
        }
    }
    Ok(std::array::from_fn(|lane| {
        std::mem::take(&mut converted[lane]).into_boxed_slice()
    }))
}

fn finite_values_to_f32(values: &[FiniteF32]) -> Vec<f32> {
    values.iter().map(|value| value.value()).collect()
}

fn clone_lanes(lanes: &[Vec<f32>; LANE_COUNT]) -> [Vec<f32>; LANE_COUNT] {
    std::array::from_fn(|lane| lanes[lane].clone())
}

#[cfg(test)]
fn flatten_lanes(lanes: &[Vec<f32>; LANE_COUNT]) -> Vec<f32> {
    lanes.iter().flat_map(|lane| lane.iter().copied()).collect()
}

fn flatten_finite_lanes(lanes: &[Box<[FiniteF32]>; LANE_COUNT]) -> Vec<f32> {
    lanes
        .iter()
        .flat_map(|lane| lane.iter().map(|sample| sample.value()))
        .collect()
}

fn finite_word(value: f32) -> Result<FiniteF32, SpatialAudioReceiverError> {
    FiniteF32::new(value).map_err(|_| SpatialAudioReceiverError::NumericNonfinite)
}

fn raw_word_value(word: RawWord) -> Result<FiniteF32, SpatialAudioReceiverError> {
    finite_word(f32::from_bits(u32::from_le_bytes(word.0)))
}

fn raw_word(value: f32) -> RawWord {
    RawWord(value.to_bits().to_le_bytes())
}

fn read_f32(bytes: &[u8], offset: usize) -> f32 {
    f32::from_bits(u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]))
}

fn write_f32(bytes: &mut [u8], offset: usize, value: f32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_bits().to_le_bytes());
}

fn map_numeric_error(_: crate::numeric::R32Error) -> SpatialAudioReceiverError {
    SpatialAudioReceiverError::NumericNonfinite
}

fn map_descriptor_error(
    error: super::descriptor_control::DescriptorError,
) -> SpatialAudioReceiverError {
    match error {
        super::descriptor_control::DescriptorError::ParameterUnavailable => {
            SpatialAudioReceiverError::ProviderMissing
        }
        super::descriptor_control::DescriptorError::ParameterInvalid => {
            SpatialAudioReceiverError::ProviderIdentity
        }
        super::descriptor_control::DescriptorError::HoldRange
        | super::descriptor_control::DescriptorError::InputShape => {
            SpatialAudioReceiverError::StateShape
        }
        super::descriptor_control::DescriptorError::InputNonFinite
        | super::descriptor_control::DescriptorError::NonFiniteIntermediate => {
            SpatialAudioReceiverError::NumericNonfinite
        }
    }
}

#[cfg(test)]
mod tests;
