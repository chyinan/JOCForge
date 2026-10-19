// pattern: Functional Core
//! Callback-local receiver QMF lane routing and mode=1 state staging.

use super::{
    LfeDelayRingState, ModelState, RawBytes, SpatialAudioJoinMetadata,
    component_stage::ComponentAfterState,
    type10_assembler::{Type10AssemblyOutcome, Type10AssemblyProposal, Type10AssemblyReceipt},
};
use crate::{
    numeric::FiniteF32,
    profile::resolve_selected_profile,
    spatial::{
        QmfMathematicalFftProvider, QmfReceiverFunctionalProvider,
        post_window_fanout::CallbackPhase,
        qmf_analysis_synthesis::{
            QmfAnalysisHistory, QmfAnalysisSynthesisError, QmfReceiverMode1Histories,
            analyze_qmf_slot, process_qmf_receiver_mode1_callback,
        },
    },
    timeline::{FrontendCallbackIndex, ReceiverCallbackCount},
};

const FRAME_SAMPLES: usize = 1_536;
const SLOT_SAMPLES: usize = 64;
const SLOT_COUNT: usize = 24;
const CARRIER_LANES: [usize; 5] = [0, 1, 2, 4, 5];
const SURROUND_CARRIER_ROWS: [usize; 2] = [3, 4];
const SPECTRUM_WORDS: usize = 128;
pub(super) type CarrierAnalysisSpectra = [Box<[[u32; SPECTRUM_WORDS]; SLOT_COUNT]>; 5];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(super) enum ReceiverMode1Error {
    ProfileUnsupported,
    CallbackOrder,
    State,
    Input,
    NumericFailure,
}

impl ReceiverMode1Error {
    #[allow(dead_code)]
    pub(super) const fn code(self) -> &'static str {
        match self {
            Self::ProfileUnsupported => "F04_PROFILE_UNSUPPORTED",
            Self::CallbackOrder => "F04_CALLBACK_ORDER",
            Self::State => "F04_STATE_ERROR",
            Self::Input => "F04_INPUT_ERROR",
            Self::NumericFailure => "F04_NUMERIC_FAILURE",
        }
    }
}

#[must_use]
#[allow(dead_code)]
pub(super) struct ReceiverMode1StageProposal {
    component_after: ComponentAfterState,
    lfe_after: LfeDelayRingState,
    type10_outcome: Type10AssemblyOutcome,
    receiver_outcome: ReceiverMode1CallbackOutput,
}

impl ReceiverMode1StageProposal {
    #[allow(dead_code)]
    pub(super) fn into_parts(
        self,
    ) -> (
        ComponentAfterState,
        LfeDelayRingState,
        Type10AssemblyOutcome,
        ReceiverMode1CallbackOutput,
    ) {
        (
            self.component_after,
            self.lfe_after,
            self.type10_outcome,
            self.receiver_outcome,
        )
    }
}

#[must_use]
pub(super) struct ReceiverS09InputView<'a> {
    parent: &'a SpatialAudioJoinMetadata,
    phase: CallbackPhase,
    lanes: [&'a [FiniteF32]; 6],
}

impl<'a> ReceiverS09InputView<'a> {
    #[allow(dead_code)]
    pub(super) fn from_f04r_stage3(
        parent: &'a SpatialAudioJoinMetadata,
        phase: CallbackPhase,
        lanes: [&'a [FiniteF32]; 6],
    ) -> Result<Self, ReceiverMode1Error> {
        if lanes.iter().any(|lane| lane.len() != FRAME_SAMPLES) {
            return Err(ReceiverMode1Error::Input);
        }
        Ok(Self {
            parent,
            phase,
            lanes,
        })
    }
}

#[allow(dead_code)]
pub(super) enum ReceiverMode1CallbackOutput {
    Priming {
        receiver_callback_count_after: ReceiverCallbackCount,
    },
    Ready(Box<ReceiverMode1ReadyOutput>),
}

#[allow(dead_code)]
pub(super) struct ReceiverMode1ReadyOutput {
    pub(super) analysis_spectra: CarrierAnalysisSpectra,
    pub(super) surround_pcm: [Box<[FiniteF32]>; 2],
    pub(super) after: ReceiverMode1AfterState,
}

#[allow(dead_code)]
pub(super) struct ReceiverMode1AfterState {
    pub(super) analysis_history: [[RawBytes; 9]; 5],
    pub(super) qmf_delay_history: [[RawBytes; 10]; 2],
    pub(super) qmf_fir_history: [RawBytes; 2],
    pub(super) synthesis_history: [RawBytes; 2],
    pub(super) receiver_callback_count: ReceiverCallbackCount,
}

/// Route one assembled callback through receiver analysis and surround-only mode=1 QMF.
/// The caller supplies the F04R-produced S09 lanes and owns every returned after slice.
#[allow(clippy::too_many_arguments, dead_code)]
pub(super) fn stage_receiver_mode1(
    expected_parent: &SpatialAudioJoinMetadata,
    expected_phase: CallbackPhase,
    receiver_before: &ModelState,
    receiver_callback_count_before: ReceiverCallbackCount,
    type10_proposal: Type10AssemblyProposal,
    s09_input: Option<&ReceiverS09InputView<'_>>,
    functional_provider: &QmfReceiverFunctionalProvider,
    fft_provider: &QmfMathematicalFftProvider,
) -> Result<ReceiverMode1StageProposal, ReceiverMode1Error> {
    let (component_after, lfe_after, type10_outcome) = type10_proposal.into_parts();
    {
        let receipt = match &type10_outcome {
            Type10AssemblyOutcome::Priming { receipt }
            | Type10AssemblyOutcome::Ready { receipt, .. } => receipt,
        };
        validate_parent_lineage(expected_parent, expected_phase, receipt)?;
    }
    let callback = expected_parent.callback_index().value();
    match (callback.checked_sub(2), type10_outcome, s09_input) {
        (None, Type10AssemblyOutcome::Priming { receipt }, None) => {
            if receipt.shape() != [16, FRAME_SAMPLES]
                || receipt.component_origin_generation() != expected_parent.callback_index()
                || receipt.lfe_origin_generation().is_some()
            {
                return Err(ReceiverMode1Error::State);
            }
            if receiver_callback_count_before.value() != 0 {
                return Err(ReceiverMode1Error::State);
            }
            Ok(ReceiverMode1StageProposal {
                component_after,
                lfe_after,
                type10_outcome: Type10AssemblyOutcome::Priming { receipt },
                receiver_outcome: ReceiverMode1CallbackOutput::Priming {
                    receiver_callback_count_after: receiver_callback_count_before,
                },
            })
        }
        (Some(receiver_index), Type10AssemblyOutcome::Ready { packet, receipt }, Some(s09)) => {
            validate_s09_lineage(expected_parent, expected_phase, s09)?;
            if receiver_callback_count_before.value() != receiver_index {
                return Err(ReceiverMode1Error::State);
            }
            if packet.callback_index() != expected_parent.callback_index()
                || packet.shape() != [16, FRAME_SAMPLES]
                || receipt.shape() != [16, FRAME_SAMPLES]
                || receipt.component_origin_generation() != expected_parent.callback_index()
                || receipt.lfe_origin_generation()
                    != Some(FrontendCallbackIndex::new(receiver_index))
            {
                return Err(ReceiverMode1Error::State);
            }
            if s09.lanes.iter().any(|lane| lane.len() != FRAME_SAMPLES) {
                return Err(ReceiverMode1Error::Input);
            }

            let receiver_callback_count_after = receiver_index
                .checked_add(1)
                .map(ReceiverCallbackCount::new)
                .ok_or(ReceiverMode1Error::State)?;
            let analysis = analyze_receiver_lanes(
                receiver_before,
                &s09.lanes,
                functional_provider,
                fft_provider,
            )?;
            let surround = process_surround_lanes(
                receiver_before,
                &analysis.spectra,
                functional_provider,
                fft_provider,
            )?;
            let after = ReceiverMode1AfterState {
                analysis_history: analysis.history_after,
                qmf_delay_history: surround.delay_after,
                qmf_fir_history: surround.fir_after,
                synthesis_history: surround.synthesis_after,
                receiver_callback_count: receiver_callback_count_after,
            };
            Ok(ReceiverMode1StageProposal {
                component_after,
                lfe_after,
                type10_outcome: Type10AssemblyOutcome::Ready { packet, receipt },
                receiver_outcome: ReceiverMode1CallbackOutput::Ready(Box::new(
                    ReceiverMode1ReadyOutput {
                        analysis_spectra: analysis.spectra,
                        surround_pcm: surround.pcm,
                        after,
                    },
                )),
            })
        }
        (None, Type10AssemblyOutcome::Priming { .. }, Some(_))
        | (Some(_), Type10AssemblyOutcome::Ready { .. }, None) => Err(ReceiverMode1Error::Input),
        _ => Err(ReceiverMode1Error::CallbackOrder),
    }
}

fn validate_parent_lineage(
    expected_parent: &SpatialAudioJoinMetadata,
    expected_phase: CallbackPhase,
    receipt: &Type10AssemblyReceipt,
) -> Result<(), ReceiverMode1Error> {
    if resolve_selected_profile(&expected_parent.profile_identity()).is_err()
        || receipt.profile_identity() != expected_parent.profile_identity()
    {
        return Err(ReceiverMode1Error::ProfileUnsupported);
    }
    if receipt.callback_index() != expected_parent.callback_index()
        || receipt.phase() != expected_phase
    {
        return Err(ReceiverMode1Error::CallbackOrder);
    }
    if receipt.spatial_audio_before_generation() != expected_parent.before_generation()
        || receipt.spatial_audio_before_digest() != expected_parent.before_digest()
    {
        return Err(ReceiverMode1Error::State);
    }
    if receipt.programme_id().as_str() != expected_parent.programme_id().as_str()
        || receipt.attempt() != expected_parent.attempt()
        || receipt.transaction_id().as_str() != expected_parent.transaction_id().as_str()
    {
        return Err(ReceiverMode1Error::Input);
    }
    Ok(())
}

fn validate_s09_lineage(
    expected_parent: &SpatialAudioJoinMetadata,
    expected_phase: CallbackPhase,
    input: &ReceiverS09InputView<'_>,
) -> Result<(), ReceiverMode1Error> {
    if input.phase != expected_phase
        || input.parent.callback_index() != expected_parent.callback_index()
    {
        return Err(ReceiverMode1Error::CallbackOrder);
    }
    if input.parent.profile_identity() != expected_parent.profile_identity() {
        return Err(ReceiverMode1Error::ProfileUnsupported);
    }
    if input.parent.before_generation() != expected_parent.before_generation()
        || input.parent.before_digest() != expected_parent.before_digest()
    {
        return Err(ReceiverMode1Error::State);
    }
    if input.parent.programme_id().as_str() != expected_parent.programme_id().as_str()
        || input.parent.attempt() != expected_parent.attempt()
        || input.parent.transaction_id().as_str() != expected_parent.transaction_id().as_str()
    {
        return Err(ReceiverMode1Error::Input);
    }
    Ok(())
}

struct ReceiverAnalysis {
    spectra: CarrierAnalysisSpectra,
    history_after: [[RawBytes; 9]; 5],
}

struct ReceiverSurroundOutput {
    pcm: [Box<[FiniteF32]>; 2],
    delay_after: [[RawBytes; 10]; 2],
    fir_after: [RawBytes; 2],
    synthesis_after: [RawBytes; 2],
}

fn analyze_receiver_lanes(
    receiver_before: &ModelState,
    s09_input: &[&[FiniteF32]; 6],
    functional_provider: &QmfReceiverFunctionalProvider,
    fft_provider: &QmfMathematicalFftProvider,
) -> Result<ReceiverAnalysis, ReceiverMode1Error> {
    let mut spectra: CarrierAnalysisSpectra =
        std::array::from_fn(|_| Box::new([[0_u32; SPECTRUM_WORDS]; SLOT_COUNT]));
    let mut history_after_by_carrier = Vec::with_capacity(CARRIER_LANES.len());
    for (carrier, receiver_lane) in CARRIER_LANES.into_iter().enumerate() {
        let history_words = receiver_before.analysis_history[carrier]
            .iter()
            .map(raw_words::<64>)
            .collect::<Result<Vec<_>, _>>()?;
        let history_rows: [[u32; 64]; 9] = history_words
            .try_into()
            .map_err(|_| ReceiverMode1Error::State)?;
        let mut history = QmfAnalysisHistory::from_words(history_rows).map_err(map_qmf_error)?;
        for (slot, spectrum_slot) in spectra[carrier].iter_mut().enumerate() {
            let start = slot * SLOT_SAMPLES;
            let current_samples =
                std::array::from_fn(|sample| s09_input[receiver_lane][start + sample].value());
            let result = analyze_qmf_slot(
                &current_samples,
                &history,
                functional_provider,
                fft_provider,
            )
            .map_err(map_qmf_error)?;
            *spectrum_slot = *result.spectrum_words();
            history = QmfAnalysisHistory::from_words(*result.history_after().rows())
                .map_err(map_qmf_error)?;
        }
        history_after_by_carrier.push(std::array::from_fn(|row| {
            raw_bytes_from_words(&history.rows()[row])
        }));
    }
    let analysis_history_after = history_after_by_carrier
        .try_into()
        .map_err(|_| ReceiverMode1Error::State)?;
    Ok(ReceiverAnalysis {
        spectra,
        history_after: analysis_history_after,
    })
}

fn process_surround_lanes(
    receiver_before: &ModelState,
    analysis_spectra: &CarrierAnalysisSpectra,
    functional_provider: &QmfReceiverFunctionalProvider,
    fft_provider: &QmfMathematicalFftProvider,
) -> Result<ReceiverSurroundOutput, ReceiverMode1Error> {
    let mut pcm_by_surround = Vec::with_capacity(SURROUND_CARRIER_ROWS.len());
    let mut delay_after_by_surround = Vec::with_capacity(SURROUND_CARRIER_ROWS.len());
    let mut fir_after_by_surround = Vec::with_capacity(SURROUND_CARRIER_ROWS.len());
    let mut synthesis_after_by_surround = Vec::with_capacity(SURROUND_CARRIER_ROWS.len());

    for (surround, carrier) in SURROUND_CARRIER_ROWS.into_iter().enumerate() {
        let history_before = receiver_mode1_history_before(receiver_before, surround)?;
        let result = process_qmf_receiver_mode1_callback(
            &analysis_spectra[carrier],
            &history_before,
            functional_provider,
            fft_provider,
        )
        .map_err(map_qmf_error)?;
        let pcm = result
            .pcm_slots()
            .iter()
            .flatten()
            .map(|sample| FiniteF32::new(*sample).map_err(|_| ReceiverMode1Error::NumericFailure))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        let history_after = result.history_after();
        delay_after_by_surround.push(std::array::from_fn(|slot| {
            raw_bytes_from_words(&history_after.delay_history()[slot])
        }));
        let fir_words: [u32; 40] =
            std::array::from_fn(|word| history_after.band0_fir_history()[word / 2][word % 2]);
        fir_after_by_surround.push(raw_bytes_from_words(&fir_words));
        synthesis_after_by_surround.push(raw_bytes_from_words(history_after.synthesis_history()));
        pcm_by_surround.push(pcm);
    }

    Ok(ReceiverSurroundOutput {
        pcm: pcm_by_surround
            .try_into()
            .map_err(|_| ReceiverMode1Error::State)?,
        delay_after: delay_after_by_surround
            .try_into()
            .map_err(|_| ReceiverMode1Error::State)?,
        fir_after: fir_after_by_surround
            .try_into()
            .map_err(|_| ReceiverMode1Error::State)?,
        synthesis_after: synthesis_after_by_surround
            .try_into()
            .map_err(|_| ReceiverMode1Error::State)?,
    })
}

fn receiver_mode1_history_before(
    receiver_before: &ModelState,
    surround: usize,
) -> Result<QmfReceiverMode1Histories, ReceiverMode1Error> {
    let delay_words = receiver_before.qmf_delay_history[surround]
        .iter()
        .map(raw_words::<SPECTRUM_WORDS>)
        .collect::<Result<Vec<_>, _>>()?;
    let delay_history = delay_words
        .try_into()
        .map_err(|_| ReceiverMode1Error::State)?;
    let fir_words = raw_words::<40>(&receiver_before.qmf_fir_history[surround])?;
    let fir_history = std::array::from_fn(|index| [fir_words[index * 2], fir_words[index * 2 + 1]]);
    let synthesis_history = raw_words::<576>(&receiver_before.synthesis_history[surround])?;
    QmfReceiverMode1Histories::from_words(delay_history, fir_history, synthesis_history)
        .map_err(map_qmf_error)
}

fn raw_words<const COUNT: usize>(raw: &RawBytes) -> Result<[u32; COUNT], ReceiverMode1Error> {
    if raw.bytes.len() != COUNT * 4 {
        return Err(ReceiverMode1Error::State);
    }
    let mut words = [0_u32; COUNT];
    for (index, chunk) in raw.bytes.chunks_exact(4).enumerate() {
        let bytes: [u8; 4] = chunk.try_into().map_err(|_| ReceiverMode1Error::State)?;
        words[index] = u32::from_le_bytes(bytes);
    }
    Ok(words)
}

fn raw_bytes_from_words<const COUNT: usize>(words: &[u32; COUNT]) -> RawBytes {
    let mut bytes = Vec::with_capacity(COUNT * 4);
    for word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    RawBytes {
        bytes: bytes.into_boxed_slice(),
    }
}

fn map_qmf_error(error: QmfAnalysisSynthesisError) -> ReceiverMode1Error {
    match error {
        QmfAnalysisSynthesisError::InvalidInput => ReceiverMode1Error::Input,
        QmfAnalysisSynthesisError::NumericFailure => ReceiverMode1Error::NumericFailure,
    }
}
