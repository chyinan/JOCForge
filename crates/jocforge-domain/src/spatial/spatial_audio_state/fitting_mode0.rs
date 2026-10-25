// pattern: Functional Core
#![allow(dead_code)] // F04RB remains a standalone proposal until parent transaction integration is authorized.
//! Selected mode=0 fitting window and history proposal. This stage cannot commit state.

use super::{
    JocGeometryRowState, ModelState, QmfWindowState, RawBytes, SpatialAudioJoinMetadata,
    TemporalState, spatial_audio_receiver::SpatialAudioReceiverProposal, zero_raw_matrix,
};
use crate::{
    joc::oamd::reader_target::{HistoryProducerGeneration, ReaderInvocationIndex},
    numeric::FiniteF32,
    profile::resolve_selected_profile,
    spatial::{
        DescriptorMathematicalProvider, QmfMathematicalFftProvider, QmfReceiverFunctionalProvider,
        post_window_fanout::CallbackPhase,
        qmf_analysis_synthesis::{QmfAnalysisHistory, QmfAnalysisSynthesisError, analyze_qmf_slot},
        source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
    },
    timeline::{
        CodecSourceIndex, FrontendCallbackIndex, JocDispatchIndex, ReceiverCallbackCount,
        source_for_callback,
    },
    transaction::CallbackLineageIdentity,
};

const CARRIER_LANES: [usize; 5] = [0, 1, 2, 4, 5];
const SLOT_COUNT: usize = 24;
const SLOT_SAMPLES: usize = 64;
const SPECTRUM_WORDS: usize = 128;
const TARGET_COUNT: usize = 15;
const ANALYSIS_HISTORY_ROWS: usize = 9;
const TEMPORAL_SPAN: usize = 896;
const TEMPORAL_SPAN_I64: i64 = 896;
const TEMPORAL_BUFFER_SAMPLES: usize = 1_792;
const TEMPORAL_BUFFER_SAMPLES_I64: i64 = 1_792;
const TEMPORAL_BUFFER_BYTES: usize = TEMPORAL_BUFFER_SAMPLES * 4;
const ANALYSIS_HISTORY_BYTES: usize = SLOT_SAMPLES * 4;
const WINDOW_SAMPLES: usize = SLOT_COUNT * 2;
const DELAY_SLOTS: usize = 10;
const FIR_COMPLEX_VALUES: usize = 20;
const FIR_WORDS: usize = FIR_COMPLEX_VALUES * 2;

const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const FUNCTIONAL_PROVIDER_PATH: &str = "parameters/qmf-receiver-functional-v1.json";
const FUNCTIONAL_PROVIDER_BYTES: u64 = 20_325;
const FUNCTIONAL_PROVIDER_SHA256: &str =
    "30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c";
const FFT_PROVIDER_PATH: &str = "parameters/qmf-mathematical-fft-v1.json";
const FFT_PROVIDER_BYTES: u64 = 6_983;
const FFT_PROVIDER_SHA256: &str =
    "e8d3fb1514a5ffd857740222eacc8995208a29c3447e48c1b63083af368c0431";
const DESCRIPTOR_PROVIDER_PATH: &str = "parameters/descriptor-mathematical.json";
const DESCRIPTOR_PROVIDER_BYTES: u64 = 4_861;
const DESCRIPTOR_PROVIDER_SHA256: &str =
    "a493fbdaf2dab30e940cae9976ed3305f3af083e5614ab0cbd6889c71a02b197";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Mode0FittingError {
    ReceiverParameterUnavailable,
    ReceiverParameterInvalid,
    Lineage,
    DescriptorProjection,
    StateShape,
    InputShape,
    TargetTemporalShape,
    InputNonfinite,
    NumericNonfinite,
}

impl Mode0FittingError {
    pub(super) const fn code(self) -> &'static str {
        match self {
            Self::ReceiverParameterUnavailable => "RECEIVER_PARAMETER_UNAVAILABLE",
            Self::ReceiverParameterInvalid => "RECEIVER_PARAMETER_INVALID",
            Self::Lineage => "F04RB_LINEAGE_ERROR",
            Self::DescriptorProjection => "F04RB_DESCRIPTOR_PROJECTION",
            Self::StateShape => "F04RB_MODE0_STATE_SHAPE",
            Self::InputShape => "F04RB_INPUT_SHAPE",
            Self::TargetTemporalShape => "F04RB_TARGET_TEMPORAL_SHAPE",
            Self::InputNonfinite => "F04RB_INPUT_NONFINITE",
            Self::NumericNonfinite => "F04RB_NUMERIC_NONFINITE",
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct S09InputView<'a> {
    parent: &'a SpatialAudioJoinMetadata,
    phase: CallbackPhase,
    lanes: &'a [&'a [f32]],
}

#[derive(Clone, Copy)]
pub(super) struct TargetInputView<'a> {
    parent: &'a SpatialAudioJoinMetadata,
    phase: CallbackPhase,
    rows: &'a [&'a [f32]],
}

#[derive(Clone, Copy)]
pub(super) struct ReceiverAnalysisAfterView<'a> {
    parent: &'a SpatialAudioJoinMetadata,
    phase: CallbackPhase,
    generation: ReceiverCallbackCount,
    histories: &'a [[RawBytes; ANALYSIS_HISTORY_ROWS]; 5],
}

#[derive(Clone, Copy)]
pub(super) struct DescriptorAfterView<'a> {
    parent: &'a SpatialAudioJoinMetadata,
    phase: CallbackPhase,
    generation: ReceiverCallbackCount,
    selected_index: u8,
    target_gain_u32: u32,
}

#[derive(Clone, Copy)]
pub(super) struct Mode0FittingInput<'a> {
    expected_parent: &'a SpatialAudioJoinMetadata,
    expected_phase: CallbackPhase,
    fitting_before: &'a QmfWindowState,
    fitting_before_generation: ReceiverCallbackCount,
    receiver_analysis_before: &'a [[RawBytes; ANALYSIS_HISTORY_ROWS]; 5],
    receiver_analysis_before_generation: ReceiverCallbackCount,
    receiver_analysis_after: ReceiverAnalysisAfterView<'a>,
    s09: S09InputView<'a>,
    targets: TargetInputView<'a>,
    descriptor_after: DescriptorAfterView<'a>,
}

#[derive(Clone, Copy)]
pub(super) enum Mode0FittingProviderInput<'a> {
    Missing,
    Invalid,
    Ready {
        functional: &'a QmfReceiverFunctionalProvider,
        fft: &'a QmfMathematicalFftProvider,
        descriptor: &'a DescriptorMathematicalProvider,
    },
}

#[derive(Debug)]
pub(super) struct Mode0FittingProposal {
    pub(super) receiver_index: ReceiverCallbackCount,
    pub(super) receiver_generation_after: ReceiverCallbackCount,
    pub(super) dispatch: JocDispatchIndex,
    pub(super) clip_index: u8,
    pub(super) target_temporal_output: Vec<u32>,
    pub(super) target_raw_analysis: Vec<u32>,
    pub(super) target_windows48: Vec<u32>,
    pub(super) carrier_raw_analysis: Vec<u32>,
    pub(super) carrier_mode0_current24: Vec<u32>,
    pub(super) carrier_windows48: Vec<u32>,
    pub(super) receiver_analysis_after_join: [[RawBytes; ANALYSIS_HISTORY_ROWS]; 5],
    pub(super) after_state: QmfWindowState,
}

/// Immutable same-generation QMF values prepared for the JOC coefficient stage.
/// This projection carries actual buffers and cannot advance receiver/QMF state.
#[derive(Debug)]
pub struct SelectedJocQmfProjection {
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: CallbackLineageIdentity,
    spatial_before_generation: u64,
    spatial_before_digest: super::SpatialAudioDigest,
    callback_index: FrontendCallbackIndex,
    source_index: CodecSourceIndex,
    receiver_generation_after: ReceiverCallbackCount,
    dispatch: JocDispatchIndex,
    component_origin_generation: FrontendCallbackIndex,
    lfe_origin_generation: FrontendCallbackIndex,
    target_reader_invocation: ReaderInvocationIndex,
    target_history_producer_generations: [HistoryProducerGeneration; 16],
    target_state_before_sha256: [u8; 32],
    target_state_after_sha256: [u8; 32],
    target_sha256: [u8; 32],
    target_semantic_sha256: [u8; 32],
    target_update_words: [u32; 96],
    descriptor_selected_index: u8,
    descriptor_target_gain_word: u32,
    s09_return: [Box<[FiniteF32]>; 6],
    final_six: [Box<[FiniteF32]>; 6],
    target_object_windows48: Box<[u32]>,
    carrier_windows48: Box<[u32]>,
    geometry_rows_before: [JocGeometryRowState; 16],
    geometry_rows_after: [JocGeometryRowState; 16],
}

impl SelectedJocQmfProjection {
    pub(crate) fn programme_id(&self) -> &SourceFlagsProgrammeId {
        &self.programme_id
    }

    pub(crate) const fn attempt(&self) -> SourceFlagsAttempt {
        self.attempt
    }

    pub(crate) const fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub(crate) const fn spatial_before_generation(&self) -> u64 {
        self.spatial_before_generation
    }

    pub(crate) const fn spatial_before_digest(&self) -> super::SpatialAudioDigest {
        self.spatial_before_digest
    }

    pub(crate) const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub(crate) const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub(crate) const fn receiver_generation_after(&self) -> ReceiverCallbackCount {
        self.receiver_generation_after
    }

    pub(crate) const fn dispatch(&self) -> JocDispatchIndex {
        self.dispatch
    }

    pub(crate) const fn component_origin_generation(&self) -> FrontendCallbackIndex {
        self.component_origin_generation
    }

    pub(crate) const fn lfe_origin_generation(&self) -> FrontendCallbackIndex {
        self.lfe_origin_generation
    }

    pub(crate) const fn target_reader_invocation(&self) -> ReaderInvocationIndex {
        self.target_reader_invocation
    }

    pub(crate) const fn target_history_producer_generations(
        &self,
    ) -> &[HistoryProducerGeneration; 16] {
        &self.target_history_producer_generations
    }

    pub(crate) const fn target_state_before_sha256(&self) -> &[u8; 32] {
        &self.target_state_before_sha256
    }

    pub(crate) const fn target_state_after_sha256(&self) -> &[u8; 32] {
        &self.target_state_after_sha256
    }

    pub(crate) const fn target_sha256(&self) -> &[u8; 32] {
        &self.target_sha256
    }

    pub(crate) const fn target_semantic_sha256(&self) -> &[u8; 32] {
        &self.target_semantic_sha256
    }

    pub(crate) const fn target_update_words(&self) -> &[u32; 96] {
        &self.target_update_words
    }

    pub(crate) const fn descriptor_selected_index(&self) -> u8 {
        self.descriptor_selected_index
    }

    pub(crate) const fn descriptor_target_gain_word(&self) -> u32 {
        self.descriptor_target_gain_word
    }

    pub(crate) const fn s09_return(&self) -> &[Box<[FiniteF32]>; 6] {
        &self.s09_return
    }

    pub(crate) const fn final_six(&self) -> &[Box<[FiniteF32]>; 6] {
        &self.final_six
    }

    pub(crate) fn target_object_complex(
        &self,
        object: usize,
        slot: usize,
        bin: usize,
    ) -> Option<[u32; 2]> {
        complex_pair(&self.target_object_windows48, object, 15, slot, bin)
    }

    pub(crate) fn target_object_window_words(&self) -> &[u32] {
        &self.target_object_windows48
    }

    pub(crate) fn carrier_window_words(&self) -> &[u32] {
        &self.carrier_windows48
    }

    /// Reads the approved packed 5x48x192 view without copying or transposing.
    pub(crate) fn target_packed_complex(
        &self,
        group: usize,
        slot: usize,
        packed_bin: usize,
    ) -> Option<[u32; 2]> {
        if group >= 5 || packed_bin >= 192 {
            return None;
        }
        let object = group * 3 + packed_bin / 64;
        self.target_object_complex(object, slot, packed_bin % 64)
    }

    pub(crate) fn carrier_complex(
        &self,
        carrier: usize,
        slot: usize,
        bin: usize,
    ) -> Option<[u32; 2]> {
        complex_pair(&self.carrier_windows48, carrier, 5, slot, bin)
    }

    pub(crate) const fn geometry_rows_before(&self) -> &[JocGeometryRowState; 16] {
        &self.geometry_rows_before
    }

    pub(crate) const fn geometry_rows_after(&self) -> &[JocGeometryRowState; 16] {
        &self.geometry_rows_after
    }
}

fn complex_pair(
    words: &[u32],
    row: usize,
    row_count: usize,
    slot: usize,
    bin: usize,
) -> Option<[u32; 2]> {
    if row >= row_count || slot >= 48 || bin >= 64 {
        return None;
    }
    let offset = ((row * 48 + slot) * 64 + bin) * 2;
    Some([*words.get(offset)?, *words.get(offset + 1)?])
}

/// Owner-private staged states plus the immutable downstream QMF projection.
#[derive(Debug)]
pub(super) struct SelectedJocQmfProposal {
    pub(super) receiver_after: ModelState,
    pub(super) fitting_after: QmfWindowState,
    pub(super) projection: SelectedJocQmfProjection,
}

/// Executes the fitting QMF stage from one already-executed receiver proposal.
/// It converts validated finite values to callback-local F32 slices but does not
/// rerun the receiver or descriptor stages.
pub(super) fn stage_mode0_fitting_from_receiver(
    expected_parent: &SpatialAudioJoinMetadata,
    fitting_before: &QmfWindowState,
    receiver_before: &ModelState,
    receiver: &SpatialAudioReceiverProposal,
    provider_input: Mode0FittingProviderInput<'_>,
) -> Result<Mode0FittingProposal, Mode0FittingError> {
    let Some(expected_source) = source_for_callback(expected_parent.callback_index()) else {
        return Err(Mode0FittingError::Lineage);
    };
    let expected_dispatch = expected_source
        .dispatch()
        .map_err(|_| Mode0FittingError::Lineage)?;
    if receiver.callback_index != expected_parent.callback_index()
        || receiver.source_index != expected_source
        || receiver.dispatch != expected_dispatch
    {
        return Err(Mode0FittingError::Lineage);
    }

    let s09_values: [Vec<f32>; 6] = std::array::from_fn(|lane| {
        receiver.s09_input[lane]
            .iter()
            .map(|sample| sample.value())
            .collect()
    });
    let s09_lanes: [&[f32]; 6] = std::array::from_fn(|lane| s09_values[lane].as_slice());
    let target_rows: [&[f32]; 15] =
        std::array::from_fn(|object| receiver.target_qmf_input[object].as_ref());
    let receiver_index = ReceiverCallbackCount::new(expected_source.value());
    let generation_after = ReceiverCallbackCount::new(
        expected_source
            .value()
            .checked_add(1)
            .ok_or(Mode0FittingError::Lineage)?,
    );
    stage_mode0_fitting(
        Mode0FittingInput {
            expected_parent,
            expected_phase: receiver.phase,
            fitting_before,
            fitting_before_generation: receiver_index,
            receiver_analysis_before: &receiver_before.analysis_history,
            receiver_analysis_before_generation: receiver_index,
            receiver_analysis_after: ReceiverAnalysisAfterView {
                parent: expected_parent,
                phase: receiver.phase,
                generation: generation_after,
                histories: &receiver.after.analysis_history,
            },
            s09: S09InputView {
                parent: expected_parent,
                phase: receiver.phase,
                lanes: &s09_lanes,
            },
            targets: TargetInputView {
                parent: expected_parent,
                phase: receiver.phase,
                rows: &target_rows,
            },
            descriptor_after: DescriptorAfterView {
                parent: expected_parent,
                phase: receiver.phase,
                generation: generation_after,
                selected_index: receiver.descriptor_selected_index,
                target_gain_u32: receiver.descriptor_target_gain_word,
            },
        },
        provider_input,
    )
}

pub(super) fn assemble_selected_joc_qmf(
    receiver: SpatialAudioReceiverProposal,
    fitting: Mode0FittingProposal,
) -> Result<SelectedJocQmfProposal, Mode0FittingError> {
    let expected_generation_after = receiver
        .source_index
        .value()
        .checked_add(1)
        .ok_or(Mode0FittingError::Lineage)?;
    if fitting.receiver_index.value() != receiver.source_index.value()
        || fitting.receiver_generation_after.value() != expected_generation_after
        || fitting.dispatch != receiver.dispatch
        || fitting.clip_index != receiver.descriptor_selected_index
        || fitting.target_windows48.len() != 15 * 48 * 64 * 2
        || fitting.carrier_windows48.len() != 5 * 48 * 64 * 2
    {
        return Err(Mode0FittingError::Lineage);
    }
    Ok(SelectedJocQmfProposal {
        receiver_after: receiver.after,
        fitting_after: fitting.after_state,
        projection: SelectedJocQmfProjection {
            programme_id: receiver.programme_id,
            attempt: receiver.attempt,
            transaction_id: receiver.transaction_id,
            spatial_before_generation: receiver.spatial_before_generation,
            spatial_before_digest: receiver.spatial_before_digest,
            callback_index: receiver.callback_index,
            source_index: receiver.source_index,
            receiver_generation_after: fitting.receiver_generation_after,
            dispatch: receiver.dispatch,
            component_origin_generation: receiver.component_origin_generation,
            lfe_origin_generation: receiver.lfe_origin_generation,
            target_reader_invocation: receiver.target_reader_invocation,
            target_history_producer_generations: receiver.target_history_producer_generations,
            target_state_before_sha256: receiver.target_state_before_sha256,
            target_state_after_sha256: receiver.target_state_after_sha256,
            target_sha256: receiver.target_sha256,
            target_semantic_sha256: receiver.target_semantic_sha256,
            target_update_words: receiver.target_update_words,
            descriptor_selected_index: receiver.descriptor_selected_index,
            descriptor_target_gain_word: receiver.descriptor_target_gain_word,
            s09_return: receiver.s09_return,
            final_six: receiver.final_six,
            target_object_windows48: fitting.target_windows48.into_boxed_slice(),
            carrier_windows48: fitting.carrier_windows48.into_boxed_slice(),
            geometry_rows_before: receiver.geometry_rows_before,
            geometry_rows_after: receiver.geometry_rows_after,
        },
    })
}

/// Stages one same-generation mode=0 fitting callback and returns its immutable after proposal.
/// This function never changes the committed owner or executes mode=1/QMF synthesis work.
pub(super) fn stage_mode0_fitting(
    input: Mode0FittingInput<'_>,
    provider_input: Mode0FittingProviderInput<'_>,
) -> Result<Mode0FittingProposal, Mode0FittingError> {
    let (functional, fft, descriptor) = match provider_input {
        Mode0FittingProviderInput::Missing => {
            return Err(Mode0FittingError::ReceiverParameterUnavailable);
        }
        Mode0FittingProviderInput::Invalid => {
            return Err(Mode0FittingError::ReceiverParameterInvalid);
        }
        Mode0FittingProviderInput::Ready {
            functional,
            fft,
            descriptor,
        } => (functional, fft, descriptor),
    };
    if !providers_match_contract(functional, fft, descriptor) {
        return Err(Mode0FittingError::ReceiverParameterInvalid);
    }

    let (receiver_index, receiver_generation_after, dispatch) = validate_input_lineage(&input)?;
    if input.descriptor_after.selected_index < 128
        || input.descriptor_after.generation != receiver_generation_after
        || descriptor.gain_words()[(input.descriptor_after.selected_index - 128) as usize]
            != input.descriptor_after.target_gain_u32
    {
        return Err(Mode0FittingError::DescriptorProjection);
    }

    if !valid_fitting_state(input.fitting_before)
        || !valid_analysis_histories(input.receiver_analysis_before)
    {
        return Err(Mode0FittingError::StateShape);
    }

    if input.s09.lanes.len() != 6
        || input
            .s09
            .lanes
            .iter()
            .any(|lane| lane.len() != SLOT_COUNT * SLOT_SAMPLES)
    {
        return Err(Mode0FittingError::InputShape);
    }
    if input.targets.rows.len() != TARGET_COUNT
        || input
            .targets
            .rows
            .iter()
            .any(|row| row.len() != SLOT_COUNT * SLOT_SAMPLES)
    {
        return Err(Mode0FittingError::TargetTemporalShape);
    }
    if input
        .s09
        .lanes
        .iter()
        .any(|lane| lane.iter().any(|sample| !sample.is_finite()))
        || input
            .targets
            .rows
            .iter()
            .any(|row| row.iter().any(|sample| !sample.is_finite()))
    {
        return Err(Mode0FittingError::InputNonfinite);
    }

    let target_temporal_output =
        exchange_target_temporal(input.fitting_before, input.targets.rows)?;
    let (target_raw_analysis, target_history_after) = analyze_rows(
        &target_temporal_output,
        TARGET_COUNT,
        &input.fitting_before.target_history,
        functional,
        fft,
    )?;
    let (carrier_raw_analysis, receiver_analysis_after_join) = analyze_carriers(
        input.s09.lanes,
        input.receiver_analysis_before,
        functional,
        fft,
    )?;
    if !equal_history_rows(
        &receiver_analysis_after_join,
        input.receiver_analysis_after.histories,
    ) {
        return Err(Mode0FittingError::Lineage);
    }

    let Mode0CarrierFit {
        current24: carrier_mode0_current24,
        delay_after,
        fir_after,
    } = fit_mode0_carriers(
        &carrier_raw_analysis,
        &input.fitting_before.fitting_delay,
        &input.fitting_before.fitting_fir,
    );
    let target_windows48 = concatenate_windows(
        &input.fitting_before.previous_targets,
        &target_raw_analysis,
        TARGET_COUNT,
    );
    let carrier_windows48 = concatenate_windows(
        &input.fitting_before.previous_carriers,
        &carrier_mode0_current24,
        5,
    );

    let after_state = make_after_state(
        input.fitting_before,
        input.targets.rows,
        &target_history_after,
        &target_raw_analysis,
        &carrier_mode0_current24,
        &delay_after,
        &fir_after,
    );
    Ok(Mode0FittingProposal {
        receiver_index,
        receiver_generation_after,
        dispatch,
        clip_index: input.descriptor_after.selected_index,
        target_temporal_output,
        target_raw_analysis,
        target_windows48,
        carrier_raw_analysis,
        carrier_mode0_current24,
        carrier_windows48,
        receiver_analysis_after_join,
        after_state,
    })
}

fn validate_input_lineage(
    input: &Mode0FittingInput<'_>,
) -> Result<
    (
        ReceiverCallbackCount,
        ReceiverCallbackCount,
        JocDispatchIndex,
    ),
    Mode0FittingError,
> {
    let parent = input.expected_parent;
    if resolve_selected_profile(&parent.profile_identity()).is_err()
        || input.expected_phase != input.s09.phase
        || input.expected_phase != input.targets.phase
        || input.expected_phase != input.receiver_analysis_after.phase
        || input.expected_phase != input.descriptor_after.phase
        || !same_parent(parent, input.s09.parent)
        || !same_parent(parent, input.targets.parent)
        || !same_parent(parent, input.receiver_analysis_after.parent)
        || !same_parent(parent, input.descriptor_after.parent)
    {
        return Err(Mode0FittingError::Lineage);
    }
    let callback = parent.callback_index().value();
    let receiver = callback.checked_sub(2).ok_or(Mode0FittingError::Lineage)?;
    let generation_after = receiver.checked_add(1).ok_or(Mode0FittingError::Lineage)?;
    let dispatch = callback
        .checked_sub(1)
        .map(|value| JocDispatchIndex::new(u64::from(value)))
        .ok_or(Mode0FittingError::Lineage)?;
    let receiver_index = ReceiverCallbackCount::new(u64::from(receiver));
    let generation_after = ReceiverCallbackCount::new(u64::from(generation_after));
    if input.fitting_before_generation != receiver_index
        || input.receiver_analysis_before_generation != receiver_index
        || input.receiver_analysis_after.generation != generation_after
        || input.descriptor_after.generation != generation_after
    {
        return Err(Mode0FittingError::Lineage);
    }
    Ok((receiver_index, generation_after, dispatch))
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

fn providers_match_contract(
    functional: &QmfReceiverFunctionalProvider,
    fft: &QmfMathematicalFftProvider,
    descriptor: &DescriptorMathematicalProvider,
) -> bool {
    let functional_receipt = functional.receipt();
    let functional_lineage = functional_receipt.validated_manifest_lineage();
    let fft_receipt = fft.receipt();
    let fft_lineage = fft_receipt.validated_manifest_lineage();
    let descriptor_receipt = descriptor.receipt();
    let descriptor_lineage = descriptor_receipt.validated_manifest_lineage();
    let common_manifest = |lineage: &crate::spatial::F04ParameterProviderLineage| {
        lineage.manifest_sha256().to_hex() == PACKAGE_MANIFEST_SHA256
    };
    common_manifest(functional_lineage)
        && common_manifest(fft_lineage)
        && common_manifest(descriptor_lineage)
        && functional_lineage.provider_path() == FUNCTIONAL_PROVIDER_PATH
        && functional_lineage.manifest_entry_bytes() == FUNCTIONAL_PROVIDER_BYTES
        && functional_lineage.manifest_entry_sha256().to_hex() == FUNCTIONAL_PROVIDER_SHA256
        && functional_receipt.provider_file_bytes() == FUNCTIONAL_PROVIDER_BYTES
        && functional_receipt.provider_file_sha256().to_hex() == FUNCTIONAL_PROVIDER_SHA256
        && functional.qwin_synthesis_words().len() == 640
        && functional
            .qwin_synthesis_words()
            .iter()
            .all(|word| f32::from_bits(*word).is_finite())
        && fft_lineage.provider_path() == FFT_PROVIDER_PATH
        && fft_lineage.manifest_entry_bytes() == FFT_PROVIDER_BYTES
        && fft_lineage.manifest_entry_sha256().to_hex() == FFT_PROVIDER_SHA256
        && fft_receipt.provider_file_bytes() == FFT_PROVIDER_BYTES
        && fft_receipt.provider_file_sha256().to_hex() == FFT_PROVIDER_SHA256
        && fft_receipt.route() == "QMF_FFT_RADIX2_MATH_V1"
        && fft.rotation_words().len() == SPECTRUM_WORDS
        && fft
            .rotation_words()
            .iter()
            .all(|word| f32::from_bits(*word).is_finite())
        && fft.stages().len() == 6
        && fft
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
        && descriptor_lineage.provider_path() == DESCRIPTOR_PROVIDER_PATH
        && descriptor_lineage.manifest_entry_bytes() == DESCRIPTOR_PROVIDER_BYTES
        && descriptor_lineage.manifest_entry_sha256().to_hex() == DESCRIPTOR_PROVIDER_SHA256
        && descriptor_receipt.provider_file_bytes() == DESCRIPTOR_PROVIDER_BYTES
        && descriptor_receipt.provider_file_sha256().to_hex() == DESCRIPTOR_PROVIDER_SHA256
        && !descriptor_receipt.private_vendor_key_included()
        && descriptor.gain_words().len() == 128
        && descriptor
            .gain_words()
            .iter()
            .all(|word| f32::from_bits(*word).is_finite() && f32::from_bits(*word) > 0.0)
}

fn valid_f32_bytes(bytes: &RawBytes, expected_bytes: usize) -> bool {
    bytes.bytes.len() == expected_bytes
        && bytes.bytes.chunks_exact(4).all(|word| {
            f32::from_bits(u32::from_le_bytes([word[0], word[1], word[2], word[3]])).is_finite()
        })
}

fn valid_analysis_histories(histories: &[[RawBytes; ANALYSIS_HISTORY_ROWS]; 5]) -> bool {
    histories
        .iter()
        .flatten()
        .all(|history| valid_f32_bytes(history, ANALYSIS_HISTORY_BYTES))
}

fn valid_fitting_state(state: &QmfWindowState) -> bool {
    let temporal = &state.target_temporal;
    if temporal.span != TEMPORAL_SPAN_I64
        || temporal.stride_bytes != 4
        || !valid_temporal_index(temporal.index_28)
        || !valid_temporal_index(temporal.index_32)
        || temporal.buffers.len() != TARGET_COUNT
        || temporal
            .buffers
            .iter()
            .any(|buffer| !valid_f32_bytes(buffer, TEMPORAL_BUFFER_BYTES))
    {
        return false;
    }
    state
        .target_history
        .iter()
        .flatten()
        .all(|history| valid_f32_bytes(history, ANALYSIS_HISTORY_BYTES))
        && state
            .previous_targets
            .iter()
            .flatten()
            .all(|slot| valid_f32_bytes(slot, SPECTRUM_WORDS * 4))
        && state
            .previous_carriers
            .iter()
            .flatten()
            .all(|slot| valid_f32_bytes(slot, SPECTRUM_WORDS * 4))
        && state
            .fitting_delay
            .iter()
            .flatten()
            .all(|slot| valid_f32_bytes(slot, SPECTRUM_WORDS * 4))
        && state
            .fitting_fir
            .iter()
            .all(|history| valid_f32_bytes(history, FIR_WORDS * 4))
}

fn valid_temporal_index(index: i64) -> bool {
    (0..=TEMPORAL_BUFFER_SAMPLES_I64).contains(&index) && index % TEMPORAL_SPAN_I64 == 0
}

fn word_at(bytes: &[u8], index: usize) -> u32 {
    let start = index * 4;
    u32::from_le_bytes([
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
    ])
}

fn store_word(bytes: &mut [u8], index: usize, word: u32) {
    let start = index * 4;
    bytes[start..start + 4].copy_from_slice(&word.to_le_bytes());
}

fn exchange_target_temporal(
    before: &QmfWindowState,
    input_rows: &[&[f32]],
) -> Result<Vec<u32>, Mode0FittingError> {
    let mut output = Vec::with_capacity(TARGET_COUNT * SLOT_COUNT * SLOT_SAMPLES);
    let span = TEMPORAL_SPAN;
    let buffer_size = TEMPORAL_BUFFER_SAMPLES;
    let read_before = usize::try_from(before.target_temporal.index_28)
        .map_err(|_| Mode0FittingError::StateShape)?;
    for (row, input) in input_rows.iter().enumerate() {
        let buffer = &before.target_temporal.buffers[row].bytes;
        for offset in 0..span {
            let source = (read_before + offset) % buffer_size;
            output.push(word_at(buffer, source));
        }
        for sample in input.iter().take(SLOT_SAMPLES * SLOT_COUNT - span) {
            output.push(sample.to_bits());
        }
    }
    Ok(output)
}

fn advance_temporal_index(index: i64) -> i64 {
    let next = index + TEMPORAL_SPAN_I64;
    if next > TEMPORAL_BUFFER_SAMPLES_I64 {
        next - TEMPORAL_BUFFER_SAMPLES_I64
    } else {
        next
    }
}

fn analyze_one_row(
    input_words_as_f32: &[f32],
    history_before: &[RawBytes; ANALYSIS_HISTORY_ROWS],
    functional: &QmfReceiverFunctionalProvider,
    fft: &QmfMathematicalFftProvider,
) -> Result<(Vec<u32>, [RawBytes; ANALYSIS_HISTORY_ROWS]), Mode0FittingError> {
    let mut history = history_from_words(history_before)?;
    let mut spectra = Vec::with_capacity(SLOT_COUNT * SPECTRUM_WORDS);
    for slot in 0..SLOT_COUNT {
        let mut samples = [0.0_f32; SLOT_SAMPLES];
        for (sample, value) in samples.iter_mut().enumerate() {
            *value = input_words_as_f32[slot * SLOT_SAMPLES + sample];
        }
        let result =
            analyze_qmf_slot(&samples, &history, functional, fft).map_err(map_analysis_error)?;
        spectra.extend_from_slice(result.spectrum_words());
        history = QmfAnalysisHistory::from_words(*result.history_after().rows())
            .map_err(map_analysis_error)?;
    }
    Ok((spectra, raw_history(history.rows())))
}

fn history_from_words(
    history: &[RawBytes; ANALYSIS_HISTORY_ROWS],
) -> Result<QmfAnalysisHistory, Mode0FittingError> {
    let mut words = [[0_u32; SLOT_SAMPLES]; ANALYSIS_HISTORY_ROWS];
    for (row, raw) in history.iter().enumerate() {
        for (sample, bytes) in raw.bytes.chunks_exact(4).enumerate() {
            words[row][sample] = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        }
    }
    QmfAnalysisHistory::from_words(words).map_err(map_analysis_error)
}

fn raw_history(
    words: &[[u32; SLOT_SAMPLES]; ANALYSIS_HISTORY_ROWS],
) -> [RawBytes; ANALYSIS_HISTORY_ROWS] {
    std::array::from_fn(|row| {
        let mut bytes = vec![0_u8; ANALYSIS_HISTORY_BYTES].into_boxed_slice();
        for (sample, word) in words[row].iter().copied().enumerate() {
            store_word(&mut bytes, sample, word);
        }
        RawBytes { bytes }
    })
}

fn map_analysis_error(error: QmfAnalysisSynthesisError) -> Mode0FittingError {
    match error {
        QmfAnalysisSynthesisError::InvalidInput | QmfAnalysisSynthesisError::NumericFailure => {
            Mode0FittingError::NumericNonfinite
        }
    }
}

fn analyze_rows(
    input_words: &[u32],
    rows: usize,
    histories_before: &[[RawBytes; ANALYSIS_HISTORY_ROWS]],
    functional: &QmfReceiverFunctionalProvider,
    fft: &QmfMathematicalFftProvider,
) -> Result<(Vec<u32>, Vec<[RawBytes; ANALYSIS_HISTORY_ROWS]>), Mode0FittingError> {
    let samples_per_row = SLOT_COUNT * SLOT_SAMPLES;
    let mut spectra = Vec::with_capacity(rows * SLOT_COUNT * SPECTRUM_WORDS);
    let mut histories_after = Vec::with_capacity(rows);
    for row in 0..rows {
        let samples = input_words[row * samples_per_row..(row + 1) * samples_per_row]
            .iter()
            .map(|word| f32::from_bits(*word))
            .collect::<Vec<_>>();
        let (row_spectra, row_history_after) =
            analyze_one_row(&samples, &histories_before[row], functional, fft)?;
        spectra.extend(row_spectra);
        histories_after.push(row_history_after);
    }
    Ok((spectra, histories_after))
}

fn analyze_carriers(
    s09_lanes: &[&[f32]],
    histories_before: &[[RawBytes; ANALYSIS_HISTORY_ROWS]; 5],
    functional: &QmfReceiverFunctionalProvider,
    fft: &QmfMathematicalFftProvider,
) -> Result<(Vec<u32>, [[RawBytes; ANALYSIS_HISTORY_ROWS]; 5]), Mode0FittingError> {
    let mut spectra = Vec::with_capacity(5 * SLOT_COUNT * SPECTRUM_WORDS);
    let mut histories_after = zero_raw_matrix::<5, ANALYSIS_HISTORY_ROWS>(ANALYSIS_HISTORY_BYTES);
    for lane_row in 0..5 {
        let lane = s09_lanes[CARRIER_LANES[lane_row]];
        let (lane_spectra, lane_history_after) =
            analyze_one_row(lane, &histories_before[lane_row], functional, fft)?;
        spectra.extend(lane_spectra);
        histories_after[lane_row] = lane_history_after;
    }
    Ok((spectra, histories_after))
}

fn equal_history_rows(
    left: &[[RawBytes; ANALYSIS_HISTORY_ROWS]; 5],
    right: &[[RawBytes; ANALYSIS_HISTORY_ROWS]; 5],
) -> bool {
    left.iter()
        .flatten()
        .zip(right.iter().flatten())
        .all(|(a, b)| a.bytes.as_ref() == b.bytes.as_ref())
}

fn words_from_bytes(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
        .collect()
}

fn copy_raw(raw: &RawBytes) -> RawBytes {
    RawBytes {
        bytes: raw.bytes.clone(),
    }
}

fn copy_raw_matrix<const OUTER: usize, const INNER: usize>(
    source: &[[RawBytes; INNER]; OUTER],
) -> [[RawBytes; INNER]; OUTER] {
    std::array::from_fn(|outer| std::array::from_fn(|inner| copy_raw(&source[outer][inner])))
}

fn copy_raw_tuple<const COUNT: usize>(source: &[RawBytes; COUNT]) -> [RawBytes; COUNT] {
    std::array::from_fn(|index| copy_raw(&source[index]))
}

fn copy_raw_boxed(source: &[RawBytes]) -> Box<[RawBytes]> {
    source
        .iter()
        .map(copy_raw)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn concatenate_windows(
    previous: &[[RawBytes; SLOT_COUNT]],
    current_words: &[u32],
    rows: usize,
) -> Vec<u32> {
    let mut output = Vec::with_capacity(rows * WINDOW_SAMPLES * SPECTRUM_WORDS);
    for (row, previous_row) in previous.iter().take(rows).enumerate() {
        for slot in previous_row.iter().take(SLOT_COUNT) {
            output.extend(words_from_bytes(&slot.bytes));
        }
        let start = row * SLOT_COUNT * SPECTRUM_WORDS;
        output.extend_from_slice(&current_words[start..start + SLOT_COUNT * SPECTRUM_WORDS]);
    }
    output
}

fn raw_slots_from_words<const SLOTS: usize>(words: &[u32]) -> [RawBytes; SLOTS] {
    std::array::from_fn(|slot| {
        let start = slot * SPECTRUM_WORDS;
        let mut bytes = vec![0_u8; SPECTRUM_WORDS * 4].into_boxed_slice();
        for (index, word) in words[start..start + SPECTRUM_WORDS]
            .iter()
            .copied()
            .enumerate()
        {
            store_word(&mut bytes, index, word);
        }
        RawBytes { bytes }
    })
}

type Mode0FittingDelayAfter = [[RawBytes; DELAY_SLOTS]; 5];
type Mode0FittingFirAfter = [RawBytes; 5];

struct Mode0CarrierFit {
    current24: Vec<u32>,
    delay_after: Mode0FittingDelayAfter,
    fir_after: Mode0FittingFirAfter,
}

fn fit_mode0_carriers(
    raw_analysis: &[u32],
    delay_before: &[[RawBytes; DELAY_SLOTS]; 5],
    fir_before: &[RawBytes; 5],
) -> Mode0CarrierFit {
    let raw_lane_words = SLOT_COUNT * SPECTRUM_WORDS;
    let mut current24 = Vec::with_capacity(5 * SLOT_COUNT * SPECTRUM_WORDS);
    let mut delay_after = zero_raw_matrix::<5, DELAY_SLOTS>(SPECTRUM_WORDS * 4);
    let mut fir_after = std::array::from_fn(|_| RawBytes::zeroed(FIR_WORDS * 4));
    for lane in 0..5 {
        let mut previous_delay = Vec::with_capacity(DELAY_SLOTS * SPECTRUM_WORDS);
        for slot in &delay_before[lane] {
            previous_delay.extend(words_from_bytes(&slot.bytes));
        }
        let start = lane * raw_lane_words;
        let lane_raw = &raw_analysis[start..start + raw_lane_words];
        current24.extend_from_slice(&previous_delay);
        current24.extend_from_slice(&lane_raw[..(SLOT_COUNT - DELAY_SLOTS) * SPECTRUM_WORDS]);

        let after_start = (SLOT_COUNT - DELAY_SLOTS) * SPECTRUM_WORDS;
        delay_after[lane] = raw_slots_from_words::<DELAY_SLOTS>(&lane_raw[after_start..]);

        let mut fir_sequence = Vec::with_capacity((FIR_COMPLEX_VALUES + SLOT_COUNT) * 2);
        fir_sequence.extend(words_from_bytes(&fir_before[lane].bytes));
        for slot in 0..SLOT_COUNT {
            let band0 = start + slot * SPECTRUM_WORDS;
            fir_sequence.extend_from_slice(&lane_raw[band0 - start..band0 - start + 2]);
        }
        let mut fir_bytes = vec![0_u8; FIR_WORDS * 4].into_boxed_slice();
        for (word_index, word) in fir_sequence[fir_sequence.len() - FIR_WORDS..]
            .iter()
            .copied()
            .enumerate()
        {
            store_word(&mut fir_bytes, word_index, word);
        }
        fir_after[lane] = RawBytes { bytes: fir_bytes };
    }
    Mode0CarrierFit {
        current24,
        delay_after,
        fir_after,
    }
}

fn make_after_state(
    before: &QmfWindowState,
    target_input_rows: &[&[f32]],
    target_history_after: &[[RawBytes; ANALYSIS_HISTORY_ROWS]],
    target_current: &[u32],
    carrier_current: &[u32],
    delay_after: &[[RawBytes; DELAY_SLOTS]; 5],
    fir_after: &[RawBytes; 5],
) -> QmfWindowState {
    let mut temporal = TemporalState {
        span: before.target_temporal.span,
        stride_bytes: before.target_temporal.stride_bytes,
        index_28: advance_temporal_index(before.target_temporal.index_28),
        index_32: advance_temporal_index(before.target_temporal.index_32),
        buffers: copy_raw_boxed(&before.target_temporal.buffers),
    };
    let write_start = usize::try_from(before.target_temporal.index_32)
        .expect("preflight validates target temporal write index");
    for (row, input_row) in target_input_rows.iter().enumerate() {
        let input_tail = &input_row[SLOT_COUNT * SLOT_SAMPLES - TEMPORAL_SPAN..];
        let buffer = &mut temporal.buffers[row].bytes;
        for (offset, sample) in input_tail.iter().copied().enumerate() {
            let destination = (write_start + offset) % TEMPORAL_BUFFER_SAMPLES;
            store_word(buffer, destination, sample.to_bits());
        }
    }
    let target_history = std::array::from_fn(|row| {
        std::array::from_fn(|history_row| copy_raw(&target_history_after[row][history_row]))
    });
    let previous_targets = std::array::from_fn(|row| {
        let start = row * SLOT_COUNT * SPECTRUM_WORDS;
        raw_slots_from_words::<SLOT_COUNT>(
            &target_current[start..start + SLOT_COUNT * SPECTRUM_WORDS],
        )
    });
    let previous_carriers = std::array::from_fn(|row| {
        let start = row * SLOT_COUNT * SPECTRUM_WORDS;
        raw_slots_from_words::<SLOT_COUNT>(
            &carrier_current[start..start + SLOT_COUNT * SPECTRUM_WORDS],
        )
    });
    QmfWindowState {
        target_temporal: temporal,
        target_history,
        previous_targets,
        previous_carriers,
        fitting_delay: copy_raw_matrix(delay_after),
        fitting_fir: copy_raw_tuple(fir_after),
    }
}

#[cfg(test)]
mod tests;
