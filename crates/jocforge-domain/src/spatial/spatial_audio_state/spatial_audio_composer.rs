// pattern: Functional Core
#![allow(dead_code)] // The application callback coordinator remains a separately gated caller.
//! Whole spatial-audio callback composition and JOC projection staging.

use super::{
    SpatialAudioAfterProposal, SpatialAudioError, SpatialAudioJoinMetadata, SpatialAudioMachine,
    SpatialAudioState,
    fitting_mode0::{
        Mode0FittingError, Mode0FittingProviderInput, SelectedJocQmfProjection,
        SelectedJocQmfProposal, assemble_selected_joc_qmf, stage_mode0_fitting_from_receiver,
    },
    spatial_audio_receiver::{
        SpatialAudioReceiverError, SpatialAudioReceiverProviderInput,
        SpatialAudioReceiverStageInput, TargetUpdatesLineage, TargetUpdatesReceipt,
        Type10ReceiverInput, stage_spatial_audio_receiver,
    },
    type10_assembler::{
        Type10AssemblyOutcome, Type10AssemblyProposal, Type10AssemblyReceipt, Type10Packet,
    },
};
use crate::{
    joc::oamd::reader_target::{
        HistoryProducerGeneration, ReaderTargetUpdateReceipt, SELECTED_PROFILE_ID,
    },
    spatial::{
        DescriptorMathematicalProvider, QmfMathematicalFftProvider, QmfReceiverFunctionalProvider,
        ValidatedWindowQ15Provider,
    },
    spatial::{post_window_fanout::CallbackPhase, source_flags::SourceFlagsAttempt},
    timeline::{FrontendCallbackIndex, ReceiverCallbackCount},
};
use sha2::{Digest, Sha256};

const FRAME_SAMPLES: usize = 1_536;
const TYPE10_ROWS: usize = 16;
const TARGET_VALUES: usize = 16 * 6;

/// Explicit local binding between spatial-control and reader attempt values.
#[derive(Clone, Copy)]
pub(in crate::spatial) struct ReaderAttemptBinding {
    source_attempt: SourceFlagsAttempt,
    reader_attempt_id: SourceFlagsAttempt,
}

impl ReaderAttemptBinding {
    pub(in crate::spatial) fn new(
        source_attempt: SourceFlagsAttempt,
        reader_attempt_id: SourceFlagsAttempt,
    ) -> Result<Self, SpatialAudioCompositionError> {
        if reader_attempt_id != source_attempt {
            return Err(SpatialAudioCompositionError::TargetLineage);
        }
        Ok(Self {
            source_attempt,
            reader_attempt_id,
        })
    }
}

pub struct SpatialAudioJocProviders<'a> {
    functional: &'a QmfReceiverFunctionalProvider,
    blend: &'a ValidatedWindowQ15Provider,
    fft: &'a QmfMathematicalFftProvider,
    descriptor: &'a DescriptorMathematicalProvider,
}

impl<'a> SpatialAudioJocProviders<'a> {
    pub fn new(
        functional: &'a QmfReceiverFunctionalProvider,
        blend: &'a ValidatedWindowQ15Provider,
        fft: &'a QmfMathematicalFftProvider,
        descriptor: &'a DescriptorMathematicalProvider,
    ) -> Self {
        Self {
            functional,
            blend,
            fft,
            descriptor,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::spatial) enum SpatialAudioCompositionError {
    State,
    CallbackOrder,
    Type10Lineage,
    TargetLineage,
    ProviderMissing,
    Receiver(SpatialAudioReceiverError),
    Fitting(Mode0FittingError),
    DirectLfe(super::direct_lfe::DirectLfeError),
    Type10(super::type10_assembler::Type10AssemblyError),
}

impl SpatialAudioCompositionError {
    pub(in crate::spatial) const fn code(self) -> &'static str {
        match self {
            Self::State => "F04_STATE_ERROR",
            Self::CallbackOrder => "F04_CALLBACK_ORDER",
            Self::Type10Lineage => "F04_TYPE10_LINEAGE",
            Self::TargetLineage => "F04_TARGET_LINEAGE",
            Self::ProviderMissing => "F04_PROVIDER_MISSING",
            Self::Receiver(error) => error.code(),
            Self::Fitting(error) => error.code(),
            Self::DirectLfe(error) => error.code(),
            Self::Type10(error) => error.code(),
        }
    }
}

impl From<SpatialAudioError> for SpatialAudioCompositionError {
    fn from(_: SpatialAudioError) -> Self {
        Self::State
    }
}

#[must_use]
pub enum SpatialAudioCallbackOutcome {
    Priming,
    Joc(Box<SelectedJocQmfProjection>),
}

#[must_use]
pub(in crate::spatial) struct SpatialAudioCallbackProposal {
    after: SpatialAudioAfterProposal,
    outcome: SpatialAudioCallbackOutcome,
}

impl SpatialAudioCallbackProposal {
    pub(in crate::spatial) fn into_parts(
        self,
    ) -> (SpatialAudioAfterProposal, SpatialAudioCallbackOutcome) {
        (self.after, self.outcome)
    }
}

/// Validates the existing component/LFE and target receipts, executes the
/// receiver and fitting stages at most once, then materializes one full-root
/// after proposal. This function has no commit authority.
#[allow(clippy::too_many_arguments)]
pub(in crate::spatial) fn stage_spatial_audio_callback(
    machine: &SpatialAudioMachine,
    parent: SpatialAudioJoinMetadata,
    phase: CallbackPhase,
    type10: Type10AssemblyProposal,
    target_receipt: Option<&ReaderTargetUpdateReceipt>,
    attempt_binding: Option<ReaderAttemptBinding>,
    providers: Option<SpatialAudioJocProviders<'_>>,
) -> Result<SpatialAudioCallbackProposal, SpatialAudioCompositionError> {
    machine.validate_join(&parent)?;
    if machine.committed.root.retired
        || machine.committed.callback_index() != parent.callback_index()
        || machine.generation() != u64::from(parent.callback_index().value())
    {
        return Err(SpatialAudioCompositionError::State);
    }

    let (component_after, lfe_after, type10_outcome) = type10.into_parts();
    let receipt = type10_receipt(&type10_outcome);
    validate_type10_receipt(&parent, phase, receipt)?;

    let callback = parent.callback_index().value();
    match (callback.checked_sub(2), type10_outcome) {
        (None, Type10AssemblyOutcome::Priming { .. }) => {
            if target_receipt.is_some() || attempt_binding.is_some() {
                return Err(SpatialAudioCompositionError::TargetLineage);
            }
            let after = materialize_after(machine, parent, component_after, lfe_after, None)?;
            Ok(SpatialAudioCallbackProposal {
                after,
                outcome: SpatialAudioCallbackOutcome::Priming,
            })
        }
        (Some(receiver_before_generation), Type10AssemblyOutcome::Ready { packet, receipt }) => {
            let target_receipt =
                target_receipt.ok_or(SpatialAudioCompositionError::TargetLineage)?;
            let attempt_binding =
                attempt_binding.ok_or(SpatialAudioCompositionError::TargetLineage)?;
            let providers = providers.ok_or(SpatialAudioCompositionError::ProviderMissing)?;
            let selected_manifest_sha256 = providers
                .functional
                .receipt()
                .validated_manifest_lineage()
                .manifest_sha256()
                .to_hex();
            validate_ready_type10(&parent, &packet, &receipt, receiver_before_generation)?;
            validate_target_receipt(
                &parent,
                target_receipt,
                attempt_binding,
                u64::from(receiver_before_generation),
            )?;

            let type10_values: Vec<f32> = packet
                .payload()
                .iter()
                .map(|sample| sample.value())
                .collect();
            let target_words = target_receipt.target_words();
            let target_values: [f32; TARGET_VALUES] = target_words.map(f32::from_bits);
            let receiver_generation_after = ReceiverCallbackCount::new(
                u64::from(receiver_before_generation)
                    .checked_add(1)
                    .ok_or(SpatialAudioCompositionError::State)?,
            );
            let receiver_before = machine.committed.root.receiver_state.clone();
            let receiver = stage_spatial_audio_receiver(SpatialAudioReceiverStageInput {
                parent: &parent,
                phase,
                receiver_callback_count_before: machine.committed.root.receiver_callback_count,
                before: receiver_before.clone(),
                type10: Type10ReceiverInput::from_assembly(
                    &parent,
                    phase,
                    packet.callback_index(),
                    receiver_generation_after,
                    receipt.component_origin_generation(),
                    receipt
                        .lfe_origin_generation()
                        .ok_or(SpatialAudioCompositionError::Type10Lineage)?,
                    &type10_values,
                ),
                target_updates: Some(TargetUpdatesReceipt::from_reader_receipt(
                    &parent,
                    phase,
                    target_receipt.callback_generation(),
                    receiver_generation_after,
                    &TargetUpdatesLineage {
                        reader_invocation: target_receipt.reader_invocation(),
                        history_producer_generations: *target_receipt
                            .history_producer_generations(),
                        state_before_sha256: *target_receipt.state_before_sha256(),
                        state_after_sha256: *target_receipt.state_after_sha256(),
                        target_sha256: *target_receipt.target_sha256(),
                        semantic_sha256: *target_receipt.semantic_sha256(),
                        target_words,
                    },
                    &target_values,
                )),
                providers: SpatialAudioReceiverProviderInput {
                    functional: Some(providers.functional),
                    blend: Some(providers.blend),
                    fft: Some(providers.fft),
                    descriptor: Some(providers.descriptor),
                    selected_manifest_sha256: Some(&selected_manifest_sha256),
                },
            })
            .map_err(SpatialAudioCompositionError::Receiver)?;
            let fitting = stage_mode0_fitting_from_receiver(
                &parent,
                &machine.committed.root.fitting_qmf_state,
                &receiver_before,
                &receiver,
                Mode0FittingProviderInput::Ready {
                    functional: providers.functional,
                    fft: providers.fft,
                    descriptor: providers.descriptor,
                },
            )
            .map_err(SpatialAudioCompositionError::Fitting)?;
            let selected = assemble_selected_joc_qmf(receiver, fitting)
                .map_err(SpatialAudioCompositionError::Fitting)?;
            let after =
                materialize_after(machine, parent, component_after, lfe_after, Some(&selected))?;
            Ok(SpatialAudioCallbackProposal {
                after,
                outcome: SpatialAudioCallbackOutcome::Joc(Box::new(selected.projection)),
            })
        }
        _ => Err(SpatialAudioCompositionError::CallbackOrder),
    }
}

fn type10_receipt(outcome: &Type10AssemblyOutcome) -> &Type10AssemblyReceipt {
    match outcome {
        Type10AssemblyOutcome::Priming { receipt }
        | Type10AssemblyOutcome::Ready { receipt, .. } => receipt,
    }
}

fn validate_type10_receipt(
    parent: &SpatialAudioJoinMetadata,
    phase: CallbackPhase,
    receipt: &Type10AssemblyReceipt,
) -> Result<(), SpatialAudioCompositionError> {
    if receipt.callback_index() != parent.callback_index() || receipt.phase() != phase {
        return Err(SpatialAudioCompositionError::CallbackOrder);
    }
    if receipt.programme_id() != parent.programme_id()
        || receipt.attempt() != parent.attempt()
        || receipt.transaction_id() != parent.transaction_id()
        || receipt.profile_identity() != parent.profile_identity()
        || receipt.spatial_audio_before_generation() != parent.before_generation()
        || receipt.spatial_audio_before_digest() != parent.before_digest()
        || receipt.component_origin_generation() != parent.callback_index()
        || receipt.shape() != [TYPE10_ROWS, FRAME_SAMPLES]
    {
        return Err(SpatialAudioCompositionError::Type10Lineage);
    }
    Ok(())
}

fn validate_ready_type10(
    parent: &SpatialAudioJoinMetadata,
    packet: &Type10Packet,
    receipt: &Type10AssemblyReceipt,
    receiver_before_generation: u32,
) -> Result<(), SpatialAudioCompositionError> {
    if packet.callback_index() != parent.callback_index()
        || packet.shape() != [TYPE10_ROWS, FRAME_SAMPLES]
        || receipt.lfe_origin_generation()
            != Some(FrontendCallbackIndex::new(receiver_before_generation))
    {
        return Err(SpatialAudioCompositionError::Type10Lineage);
    }
    Ok(())
}

fn validate_target_receipt(
    parent: &SpatialAudioJoinMetadata,
    receipt: &ReaderTargetUpdateReceipt,
    attempt_binding: ReaderAttemptBinding,
    reader_invocation: u64,
) -> Result<(), SpatialAudioCompositionError> {
    let target_words = receipt.target_words();
    let target_sha256: [u8; 32] = Sha256::digest(receipt.target_payload_f32le()).into();
    let producer_generations_valid =
        receipt
            .history_producer_generations()
            .iter()
            .all(|generation| match generation {
                HistoryProducerGeneration::Cold => true,
                HistoryProducerGeneration::Callback(index) => {
                    index.value() <= parent.callback_index().value()
                }
            });
    if attempt_binding.source_attempt != parent.attempt()
        || receipt.attempt_id() != attempt_binding.reader_attempt_id
        || receipt.programme_id() != parent.programme_id()
        || receipt.transaction_id() != parent.transaction_id()
        || receipt.callback_generation() != parent.callback_index()
        || receipt.reader_invocation().value() != reader_invocation
        || receipt.profile_id() != SELECTED_PROFILE_ID
        || receipt.target_sha256() != &target_sha256
        || !producer_generations_valid
        || target_words
            .iter()
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(SpatialAudioCompositionError::TargetLineage);
    }
    Ok(())
}

fn materialize_after(
    machine: &SpatialAudioMachine,
    parent: SpatialAudioJoinMetadata,
    component_after: super::component_stage::ComponentAfterState,
    lfe_after: super::LfeDelayRingState,
    selected: Option<&SelectedJocQmfProposal>,
) -> Result<SpatialAudioAfterProposal, SpatialAudioCompositionError> {
    let mut state = machine.committed.staged_copy();
    state.root.callback_index = FrontendCallbackIndex::new(
        parent
            .callback_index()
            .value()
            .checked_add(1)
            .ok_or(SpatialAudioCompositionError::State)?,
    );
    state.root.component_state = component_after.audio;
    state.root.component_parameter_state = component_after.parameters;
    state.root.component_filter_history = component_after.filter_history;
    state.root.lfe_state = lfe_after;
    if let Some(selected) = selected {
        state.root.receiver_state = selected.receiver_after.clone();
        state.root.fitting_qmf_state = selected.fitting_after.clone();
        state.root.receiver_callback_count = selected.projection.receiver_generation_after();
    }
    let state = SpatialAudioState::from_root(state.root)?;
    Ok(SpatialAudioAfterProposal::new(state, parent))
}
