// pattern: Imperative Shell
//! Atomic ownership boundary for the selected independent meter callback.

use super::{
    k_matrix::KMatrixProvider,
    lfe_delay_ring::{
        MeterFrontendCallbackInput, MeterLfeCallbackOrigin, MeterLfeDelayState, MeterLfeError,
    },
    post_render_meter::{
        IndependentMeterCallbackProposal, PostRenderMeterError, PostRenderMeterReceipt,
        PostRenderMeterState, stage_independent_meter_callback,
    },
    sample_block_queue::{MeterCallbackCount, MeterCallbackIndex, MeterSourceSampleCount},
    shared_gain_delay::SharedGainDelayParameters,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndependentMeterPrepassPhase {
    Meter,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndependentMeterFailureStage {
    MeterCallback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndependentMeterPrepassError {
    ColdFrontend(MeterLfeError),
    ColdPostRender(PostRenderMeterError),
    Callback(PostRenderMeterError),
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
struct IndependentMeterPrepassState {
    phase: IndependentMeterPrepassPhase,
    failure_stage: Option<IndependentMeterFailureStage>,
    lfe: MeterLfeDelayState,
    post_render: PostRenderMeterState,
}

/// Owns the local F06 prepass callback state. Each successful callback commits
/// its delayed-LFE and post-render after states together; the GTC parent root
/// and terminal measurement remain separate owners.
#[derive(Debug)]
pub struct IndependentMeterPrepass {
    source_samples: MeterSourceSampleCount,
    gain_parameters: SharedGainDelayParameters,
    k_matrix: KMatrixProvider,
    state: IndependentMeterPrepassState,
}

impl IndependentMeterPrepass {
    pub fn cold(
        source_samples: MeterSourceSampleCount,
        gain_parameters: SharedGainDelayParameters,
        k_matrix: KMatrixProvider,
    ) -> Result<Self, IndependentMeterPrepassError> {
        let state = Self::cold_state(source_samples, &k_matrix)?;
        Ok(Self {
            source_samples,
            gain_parameters,
            k_matrix,
            state,
        })
    }

    pub const fn phase(&self) -> IndependentMeterPrepassPhase {
        self.state.phase
    }

    pub const fn failure_stage(&self) -> Option<IndependentMeterFailureStage> {
        self.state.failure_stage
    }

    pub const fn next_callback_index(&self) -> MeterCallbackIndex {
        self.state.lfe.callback_index()
    }

    pub const fn source_samples(&self) -> MeterSourceSampleCount {
        self.source_samples
    }

    pub const fn normal_callback_count(&self) -> MeterCallbackCount {
        self.state.lfe.normal_callbacks()
    }

    pub const fn lfe_state(&self) -> &MeterLfeDelayState {
        &self.state.lfe
    }

    pub const fn post_render_state(&self) -> &PostRenderMeterState {
        &self.state.post_render
    }

    /// Indicates that all normal callbacks and the three specified zero-input
    /// drains have committed. This does not finalize or mint a measurement.
    pub fn ready_to_finalize(&self) -> bool {
        let Some(total_callbacks) = self.state.lfe.normal_callbacks().value().checked_add(3) else {
            return false;
        };
        self.state.phase == IndependentMeterPrepassPhase::Meter
            && self.state.lfe.callback_index().value() == total_callbacks
            && self
                .state
                .post_render
                .sample_queue()
                .finalization_view()
                .is_ok()
    }

    /// Stages one callback in owned leaf proposals. The returned non-Clone
    /// token exclusively borrows this owner until it is committed or dropped.
    /// A stage failure changes only lifecycle to FAILED.
    pub fn stage_callback(
        &mut self,
        input: MeterFrontendCallbackInput,
    ) -> Result<IndependentMeterCallbackStage<'_>, IndependentMeterPrepassError> {
        if self.state.phase == IndependentMeterPrepassPhase::Failed {
            return Err(IndependentMeterPrepassError::Failed);
        }

        match stage_independent_meter_callback(
            &self.state.lfe,
            &self.state.post_render,
            input,
            &self.gain_parameters,
            &self.k_matrix,
        ) {
            Ok(proposal) => Ok(IndependentMeterCallbackStage {
                owner: self,
                proposal,
            }),
            Err(error) => {
                self.state.phase = IndependentMeterPrepassPhase::Failed;
                self.state.failure_stage = Some(IndependentMeterFailureStage::MeterCallback);
                Err(IndependentMeterPrepassError::Callback(error))
            }
        }
    }

    /// Builds the complete next prepass root without replacing this root.
    ///
    /// The GTC parent may retain the returned after-root with its other
    /// staged owners and perform one root replacement at its own boundary.
    pub(crate) fn propose_callback(
        &self,
        input: MeterFrontendCallbackInput,
    ) -> Result<IndependentMeterPrepassProposal, IndependentMeterPrepassError> {
        if self.state.phase == IndependentMeterPrepassPhase::Failed {
            return Err(IndependentMeterPrepassError::Failed);
        }

        let proposal = stage_independent_meter_callback(
            &self.state.lfe,
            &self.state.post_render,
            input,
            &self.gain_parameters,
            &self.k_matrix,
        )
        .map_err(IndependentMeterPrepassError::Callback)?;
        let receipt = proposal.receipt();
        let lfe_output_origin = proposal.lfe_output_origin();
        let (lfe, post_render) = proposal.into_parts();
        let after = Self {
            source_samples: self.source_samples,
            gain_parameters: self.gain_parameters.clone(),
            k_matrix: self.k_matrix.clone(),
            state: IndependentMeterPrepassState {
                phase: IndependentMeterPrepassPhase::Meter,
                failure_stage: None,
                lfe,
                post_render,
            },
        };
        Ok(IndependentMeterPrepassProposal {
            after,
            receipt,
            lfe_output_origin,
        })
    }

    /// Reconstructs both callback owners from cold providers before replacing
    /// the current state; no partial reset is observable.
    pub fn reset(&mut self) -> Result<(), IndependentMeterPrepassError> {
        let cold = Self::cold_state(self.source_samples, &self.k_matrix)?;
        self.state = cold;
        Ok(())
    }

    fn cold_state(
        source_samples: MeterSourceSampleCount,
        k_matrix: &KMatrixProvider,
    ) -> Result<IndependentMeterPrepassState, IndependentMeterPrepassError> {
        let lfe = MeterLfeDelayState::cold(source_samples)
            .map_err(IndependentMeterPrepassError::ColdFrontend)?;
        let post_render = PostRenderMeterState::cold(source_samples.value(), k_matrix)
            .map_err(IndependentMeterPrepassError::ColdPostRender)?;
        Ok(IndependentMeterPrepassState {
            phase: IndependentMeterPrepassPhase::Meter,
            failure_stage: None,
            lfe,
            post_render,
        })
    }
}

/// Complete noncommitting meter-prepass after-state plus its leaf callback receipt.
#[must_use]
pub(crate) struct IndependentMeterPrepassProposal {
    after: IndependentMeterPrepass,
    receipt: PostRenderMeterReceipt,
    lfe_output_origin: Option<MeterLfeCallbackOrigin>,
}

impl IndependentMeterPrepassProposal {
    pub(crate) fn into_parts(
        self,
    ) -> (
        IndependentMeterPrepass,
        PostRenderMeterReceipt,
        Option<MeterLfeCallbackOrigin>,
    ) {
        (self.after, self.receipt, self.lfe_output_origin)
    }
}

#[must_use]
pub struct IndependentMeterCallbackStage<'a> {
    owner: &'a mut IndependentMeterPrepass,
    proposal: IndependentMeterCallbackProposal,
}

impl IndependentMeterCallbackStage<'_> {
    pub const fn receipt(&self) -> PostRenderMeterReceipt {
        self.proposal.receipt()
    }

    /// Installs both after states in one root replacement.
    pub fn commit(self) -> PostRenderMeterReceipt {
        let receipt = self.proposal.receipt();
        let (lfe, post_render) = self.proposal.into_parts();
        self.owner.state = IndependentMeterPrepassState {
            phase: IndependentMeterPrepassPhase::Meter,
            failure_stage: None,
            lfe,
            post_render,
        };
        receipt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meter::{
        k_matrix::{KMatrixMode, KMatrixProfile, generate_k_matrix},
        object_renderer::ObjectRenderCallbackGeneration,
        sample_block_queue::MeterCallbackPhase,
    };
    use serde_json::Value;

    const CALLBACK_SAMPLES: usize = 1_536;
    const SELECTED_KERNEL_Q15: [u16; 32] = [
        20, 177, 491, 958, 1_573, 2_331, 3_224, 4_244, 5_381, 6_624, 7_961, 9_379, 10_864, 12_403,
        13_980, 15_580, 17_188, 18_788, 20_365, 21_904, 23_389, 24_807, 26_144, 27_387, 28_524,
        29_544, 30_437, 31_195, 31_810, 32_277, 32_591, 32_748,
    ];
    const ZERO_PROGRAMME: &[u8] =
        include_bytes!("../../tests/fixtures/post_render_meter/zero_programme.json");

    fn controls(fixture: &Value) -> Vec<Vec<u32>> {
        let source_count =
            usize::try_from(fixture["oar_packet"]["source_count"].as_u64().unwrap()).unwrap();
        let direct = fixture["oar_packet"]["direct_control_words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>();
        let object = fixture["oar_packet"]["object_control_words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>();
        let mut rows = vec![object; source_count];
        rows[0] = direct;
        rows
    }

    fn input(
        callback: u64,
        phase: MeterCallbackPhase,
        nonfinite_regular_sample: bool,
    ) -> MeterFrontendCallbackInput {
        let fixture: Value = serde_json::from_slice(ZERO_PROGRAMME).unwrap();
        let controls = controls(&fixture);
        let regular_controls = controls.into_iter().skip(1).collect();
        let mut regular_pcm = vec![vec![0; CALLBACK_SAMPLES]; 15];
        if nonfinite_regular_sample {
            regular_pcm[0][0] = f32::INFINITY.to_bits();
        }
        MeterFrontendCallbackInput::new(
            MeterCallbackIndex::new(callback),
            ObjectRenderCallbackGeneration::try_new(callback + 1).unwrap(),
            phase,
            vec![0; CALLBACK_SAMPLES],
            regular_pcm,
            regular_controls,
        )
    }

    fn prepass() -> IndependentMeterPrepass {
        let k_matrix = generate_k_matrix(KMatrixProfile {
            sample_rate_hz: 48_000,
            mode: KMatrixMode::Modern,
            stage_count: 2,
        })
        .unwrap();
        let gain_parameters = SharedGainDelayParameters::selected(&SELECTED_KERNEL_Q15).unwrap();
        IndependentMeterPrepass::cold(
            MeterSourceSampleCount::new(1).unwrap(),
            gain_parameters,
            k_matrix,
        )
        .unwrap()
    }

    #[test]
    fn callback_after_states_install_together_and_exactly_three_drains_complete_the_schedule() {
        let mut machine = prepass();
        let cold_lfe = machine.lfe_state().clone();
        let cold_post_render = machine.post_render_state().clone();

        let abandoned = machine
            .stage_callback(input(0, MeterCallbackPhase::NormalSource, false))
            .unwrap();
        drop(abandoned);
        assert_eq!(machine.next_callback_index().value(), 0);
        assert_eq!(machine.lfe_state(), &cold_lfe);
        assert_eq!(machine.post_render_state(), &cold_post_render);

        for callback in 0..4 {
            let phase = if callback == 0 {
                MeterCallbackPhase::NormalSource
            } else {
                MeterCallbackPhase::ZeroInputDrain
            };
            let receipt = machine
                .stage_callback(input(callback, phase, false))
                .unwrap()
                .commit();
            assert_eq!(receipt.callback_index().value(), callback);
            assert_eq!(machine.next_callback_index().value(), callback + 1);
        }
        assert!(machine.ready_to_finalize());
    }

    #[test]
    fn drain_failure_is_fail_stop_without_numeric_owner_commit_and_full_reset_restarts() {
        let mut machine = prepass();
        let cold_lfe = machine.lfe_state().clone();
        let cold_post_render = machine.post_render_state().clone();
        machine
            .stage_callback(input(0, MeterCallbackPhase::NormalSource, false))
            .unwrap()
            .commit();
        let before_lfe = machine.lfe_state().clone();
        let before_post_render = machine.post_render_state().clone();

        let Err(error) = machine.stage_callback(input(1, MeterCallbackPhase::ZeroInputDrain, true))
        else {
            panic!("nonfinite regular input must fail before owner commit");
        };
        assert!(matches!(
            error,
            IndependentMeterPrepassError::Callback(PostRenderMeterError::FrontendLfe(
                MeterLfeError::InputNonFinite
            ))
        ));
        assert_eq!(machine.phase(), IndependentMeterPrepassPhase::Failed);
        assert_eq!(
            machine.failure_stage(),
            Some(IndependentMeterFailureStage::MeterCallback)
        );
        assert_eq!(machine.lfe_state(), &before_lfe);
        assert_eq!(machine.post_render_state(), &before_post_render);
        assert!(matches!(
            machine.stage_callback(input(1, MeterCallbackPhase::ZeroInputDrain, false)),
            Err(IndependentMeterPrepassError::Failed)
        ));

        machine.reset().unwrap();
        assert_eq!(machine.phase(), IndependentMeterPrepassPhase::Meter);
        assert_eq!(machine.failure_stage(), None);
        assert_eq!(machine.next_callback_index().value(), 0);
        assert_eq!(machine.lfe_state(), &cold_lfe);
        assert_eq!(machine.post_render_state(), &cold_post_render);
        assert!(!machine.ready_to_finalize());
    }

    #[test]
    fn proposal_callback_preserves_before_root_and_returns_owned_after_root() {
        let machine = prepass();
        let before_lfe = machine.lfe_state().clone();
        let before_post_render = machine.post_render_state().clone();
        let before_callback = machine.next_callback_index();
        let proposal = machine
            .propose_callback(input(0, MeterCallbackPhase::NormalSource, false))
            .unwrap();
        let (after, receipt, _) = proposal.into_parts();

        assert_eq!(machine.next_callback_index(), before_callback);
        assert_eq!(machine.lfe_state(), &before_lfe);
        assert_eq!(machine.post_render_state(), &before_post_render);
        assert_eq!(receipt.callback_index().value(), 0);
        assert_eq!(after.next_callback_index().value(), 1);
        assert_ne!(after.lfe_state(), &before_lfe);
        assert_ne!(after.post_render_state(), &before_post_render);
    }
}
