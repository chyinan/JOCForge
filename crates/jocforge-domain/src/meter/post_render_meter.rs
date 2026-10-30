// pattern: Functional Core
//! Noncommitting post-render meter chain from an explicit object-render packet.

use std::sync::Arc;

use super::{
    energy_window::{EnergyWindowError, EnergyWindowState, stage_energy_window_block},
    integrated_meter::{
        IntegratedMeterError, IntegratedMeterInput, IntegratedMeterParameters,
        IntegratedMeterState, process_integrated_meter,
    },
    k_matrix::KMatrixProvider,
    lfe_delay_ring::{
        MeterFrontendCallbackInput, MeterLfeCallbackOrigin, MeterLfeDelayState, MeterLfeError,
        stage_meter_frontend_oar_packet,
    },
    object_render_metadata::SecondaryGeometry,
    object_renderer::{
        ObjectRenderCallbackPacket, ObjectRenderCallbackPhase, ObjectRendererError,
        ObjectRendererState, RenderSourceSampleCount, stage_object_render_callback_with_geometry,
    },
    sample_block_queue::{
        CompleteSampleBlock, MeterCallbackFrame, MeterCallbackIndex, MeterCallbackPhase,
        MeterSourceSampleCount, MeterUpstreamGeneration, SampleBlockQueueError,
        SampleBlockQueueState, SampleQueueCallbackReceipt, stage_sample_block_queue_callback,
    },
    shared_gain_delay::{
        SharedGainDelayError, SharedGainDelayParameters, SharedGainDelayState,
        stage_shared_gain_delay_callback,
    },
    true_peak::{
        TruePeakError, TruePeakParameters, TruePeakState, process_true_peak_block_with_parameters,
    },
    ungated_mean::{
        UngatedMeanError, UngatedMeanInput, UngatedMeanParameters, UngatedMeanState,
        process_ungated_mean,
    },
    window_clock::{MeterWindowClockError, MeterWindowClockState},
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EnergyRecordCount(u64);

impl EnergyRecordCount {
    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostRenderMeterError {
    ClockMismatch,
    ObjectRenderer(ObjectRendererError),
    GainDelay(SharedGainDelayError),
    SampleQueue(SampleBlockQueueError),
    WindowClock(MeterWindowClockError),
    TruePeak(TruePeakError),
    EnergyWindow(EnergyWindowError),
    IntegratedMeter(IntegratedMeterError),
    UngatedMean(UngatedMeanError),
    FrontendLfe(MeterLfeError),
    ArithmeticOverflow,
}

/// Owner-validated immutable parameters consumed by post-render meter leaves.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostRenderMeterParameters {
    true_peak: TruePeakParameters,
    integrated: IntegratedMeterParameters,
    ungated: UngatedMeanParameters,
    oar_geometry: SecondaryGeometry,
}

impl PostRenderMeterParameters {
    pub const fn new(
        true_peak: TruePeakParameters,
        integrated: IntegratedMeterParameters,
        ungated: UngatedMeanParameters,
        oar_geometry: SecondaryGeometry,
    ) -> Self {
        Self {
            true_peak,
            integrated,
            ungated,
            oar_geometry,
        }
    }

    /// Loads the selected embedded reference values for existing test/model callers.
    pub fn selected() -> Result<Self, PostRenderMeterError> {
        let true_peak =
            super::true_peak::selected_parameters().map_err(PostRenderMeterError::TruePeak)?;
        let integrated =
            IntegratedMeterParameters::selected().map_err(PostRenderMeterError::IntegratedMeter)?;
        let ungated =
            UngatedMeanParameters::selected().map_err(PostRenderMeterError::UngatedMean)?;
        Ok(Self::new(
            true_peak,
            integrated,
            ungated,
            SecondaryGeometry::selected(),
        ))
    }

    pub const fn true_peak(&self) -> &TruePeakParameters {
        &self.true_peak
    }

    pub const fn integrated(&self) -> &IntegratedMeterParameters {
        &self.integrated
    }

    pub const fn ungated(&self) -> &UngatedMeanParameters {
        &self.ungated
    }

    pub const fn oar_geometry(&self) -> &SecondaryGeometry {
        &self.oar_geometry
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PostRenderMeterState {
    object_renderer: ObjectRendererState,
    shared_gain_delay: SharedGainDelayState,
    sample_queue: SampleBlockQueueState,
    window_clock: MeterWindowClockState,
    true_peak: TruePeakState,
    energy_window: EnergyWindowState,
    integrated_meter: Arc<IntegratedMeterState>,
    ungated_mean: UngatedMeanState,
}

impl PostRenderMeterState {
    pub fn cold(
        source_samples: u64,
        k_matrix: &KMatrixProvider,
    ) -> Result<Self, PostRenderMeterError> {
        let render_source_samples = RenderSourceSampleCount::try_new(source_samples)
            .map_err(PostRenderMeterError::ObjectRenderer)?;
        let queue_source_samples = MeterSourceSampleCount::new(source_samples)
            .map_err(PostRenderMeterError::SampleQueue)?;
        Ok(Self {
            object_renderer: ObjectRendererState::cold(render_source_samples),
            shared_gain_delay: SharedGainDelayState::cold(),
            sample_queue: SampleBlockQueueState::cold(queue_source_samples)
                .map_err(PostRenderMeterError::SampleQueue)?,
            window_clock: MeterWindowClockState::cold(queue_source_samples)
                .map_err(PostRenderMeterError::WindowClock)?,
            true_peak: TruePeakState::cold(),
            energy_window: EnergyWindowState::cold(k_matrix)
                .map_err(PostRenderMeterError::EnergyWindow)?,
            integrated_meter: Arc::new(IntegratedMeterState::cold()),
            ungated_mean: UngatedMeanState::cold(),
        })
    }

    pub const fn object_renderer(&self) -> &ObjectRendererState {
        &self.object_renderer
    }

    pub const fn shared_gain_delay(&self) -> &SharedGainDelayState {
        &self.shared_gain_delay
    }

    pub const fn sample_queue(&self) -> &SampleBlockQueueState {
        &self.sample_queue
    }

    pub const fn window_clock(&self) -> &MeterWindowClockState {
        &self.window_clock
    }

    pub const fn true_peak(&self) -> &TruePeakState {
        &self.true_peak
    }

    pub const fn energy_window(&self) -> &EnergyWindowState {
        &self.energy_window
    }

    pub fn integrated_meter(&self) -> &IntegratedMeterState {
        self.integrated_meter.as_ref()
    }

    pub const fn ungated_mean(&self) -> &UngatedMeanState {
        &self.ungated_mean
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PostRenderMeterReceipt {
    callback_index: MeterCallbackIndex,
    upstream_generation: MeterUpstreamGeneration,
    completed_blocks: super::sample_block_queue::MeterBlockCount,
    energy_records: EnergyRecordCount,
}

impl PostRenderMeterReceipt {
    pub const fn callback_index(self) -> MeterCallbackIndex {
        self.callback_index
    }

    pub const fn upstream_generation(self) -> MeterUpstreamGeneration {
        self.upstream_generation
    }

    pub const fn completed_blocks(self) -> super::sample_block_queue::MeterBlockCount {
        self.completed_blocks
    }

    pub const fn energy_records(self) -> EnergyRecordCount {
        self.energy_records
    }
}

#[must_use]
pub struct PostRenderMeterProposal {
    after: PostRenderMeterState,
    receipt: PostRenderMeterReceipt,
}

struct PostRenderScheduleAfter {
    object_renderer: ObjectRendererState,
    shared_gain_delay: SharedGainDelayState,
    sample_queue: SampleBlockQueueState,
    window_clock: MeterWindowClockState,
    complete_blocks: Vec<CompleteSampleBlock>,
    queue_receipt: SampleQueueCallbackReceipt,
}

struct PostRenderNumericAfter {
    true_peak: TruePeakState,
    energy_window: EnergyWindowState,
    integrated_meter: Arc<IntegratedMeterState>,
    ungated_mean: UngatedMeanState,
    energy_records: EnergyRecordCount,
}

impl PostRenderMeterProposal {
    pub const fn after(&self) -> &PostRenderMeterState {
        &self.after
    }

    pub const fn receipt(&self) -> PostRenderMeterReceipt {
        self.receipt
    }

    pub fn into_after(self) -> PostRenderMeterState {
        self.after
    }

    fn into_parts(self) -> (PostRenderMeterState, PostRenderMeterReceipt) {
        (self.after, self.receipt)
    }
}

#[must_use]
pub struct IndependentMeterCallbackProposal {
    after_lfe: MeterLfeDelayState,
    after_post_render: PostRenderMeterState,
    lfe_output_origin: Option<MeterLfeCallbackOrigin>,
    receipt: PostRenderMeterReceipt,
}

impl IndependentMeterCallbackProposal {
    pub const fn after_lfe(&self) -> &MeterLfeDelayState {
        &self.after_lfe
    }

    pub const fn after_post_render(&self) -> &PostRenderMeterState {
        &self.after_post_render
    }

    pub const fn lfe_output_origin(&self) -> Option<MeterLfeCallbackOrigin> {
        self.lfe_output_origin
    }

    pub const fn receipt(&self) -> PostRenderMeterReceipt {
        self.receipt
    }

    pub fn into_parts(self) -> (MeterLfeDelayState, PostRenderMeterState) {
        (self.after_lfe, self.after_post_render)
    }
}

pub fn stage_independent_meter_callback(
    before_lfe: &MeterLfeDelayState,
    before_post_render: &PostRenderMeterState,
    input: MeterFrontendCallbackInput,
    gain_parameters: &SharedGainDelayParameters,
    k_matrix: &KMatrixProvider,
) -> Result<IndependentMeterCallbackProposal, PostRenderMeterError> {
    let parameters = PostRenderMeterParameters::selected()?;
    stage_independent_meter_callback_with_parameters(
        before_lfe,
        before_post_render,
        input,
        gain_parameters,
        k_matrix,
        &parameters,
    )
}

/// Stages one callback using the exact owner-validated post-render parameter values.
pub fn stage_independent_meter_callback_with_parameters(
    before_lfe: &MeterLfeDelayState,
    before_post_render: &PostRenderMeterState,
    input: MeterFrontendCallbackInput,
    gain_parameters: &SharedGainDelayParameters,
    k_matrix: &KMatrixProvider,
    parameters: &PostRenderMeterParameters,
) -> Result<IndependentMeterCallbackProposal, PostRenderMeterError> {
    if before_lfe.callback_index().value()
        != before_post_render.object_renderer.callback_index().value()
    {
        return Err(PostRenderMeterError::ClockMismatch);
    }
    let packet_proposal = stage_meter_frontend_oar_packet(before_lfe, input)
        .map_err(PostRenderMeterError::FrontendLfe)?;
    let (after_lfe, packet, lfe_output_origin) = packet_proposal.into_parts();
    let scheduled = stage_post_render_schedule(
        before_post_render,
        &packet,
        gain_parameters,
        parameters.oar_geometry(),
    )?;
    let post_render =
        finish_post_render_schedule(before_post_render, scheduled, k_matrix, parameters)?;
    let (after_post_render, receipt) = post_render.into_parts();
    Ok(IndependentMeterCallbackProposal {
        after_lfe,
        after_post_render,
        lfe_output_origin,
        receipt,
    })
}

pub fn stage_post_render_meter_callback(
    before: &PostRenderMeterState,
    packet: &ObjectRenderCallbackPacket,
    gain_parameters: &SharedGainDelayParameters,
    k_matrix: &KMatrixProvider,
) -> Result<PostRenderMeterProposal, PostRenderMeterError> {
    let parameters = PostRenderMeterParameters::selected()?;
    stage_post_render_meter_callback_with_parameters(
        before,
        packet,
        gain_parameters,
        k_matrix,
        &parameters,
    )
}

/// Stages one rendered callback with explicit validated true-peak/log values.
pub fn stage_post_render_meter_callback_with_parameters(
    before: &PostRenderMeterState,
    packet: &ObjectRenderCallbackPacket,
    gain_parameters: &SharedGainDelayParameters,
    k_matrix: &KMatrixProvider,
    parameters: &PostRenderMeterParameters,
) -> Result<PostRenderMeterProposal, PostRenderMeterError> {
    let schedule =
        stage_post_render_schedule(before, packet, gain_parameters, parameters.oar_geometry())?;
    finish_post_render_schedule(before, schedule, k_matrix, parameters)
}

fn finish_post_render_schedule(
    before: &PostRenderMeterState,
    schedule: PostRenderScheduleAfter,
    k_matrix: &KMatrixProvider,
    parameters: &PostRenderMeterParameters,
) -> Result<PostRenderMeterProposal, PostRenderMeterError> {
    let PostRenderScheduleAfter {
        object_renderer,
        shared_gain_delay,
        sample_queue,
        window_clock,
        complete_blocks,
        queue_receipt,
    } = schedule;
    let numeric = stage_post_render_numeric(before, &complete_blocks, k_matrix, parameters)?;
    Ok(PostRenderMeterProposal {
        after: PostRenderMeterState {
            object_renderer,
            shared_gain_delay,
            sample_queue,
            window_clock,
            true_peak: numeric.true_peak,
            energy_window: numeric.energy_window,
            integrated_meter: numeric.integrated_meter,
            ungated_mean: numeric.ungated_mean,
        },
        receipt: PostRenderMeterReceipt {
            callback_index: queue_receipt.callback_index(),
            upstream_generation: queue_receipt.upstream_generation(),
            completed_blocks: queue_receipt.completed_blocks(),
            energy_records: numeric.energy_records,
        },
    })
}

fn stage_post_render_schedule(
    before: &PostRenderMeterState,
    packet: &ObjectRenderCallbackPacket,
    gain_parameters: &SharedGainDelayParameters,
    oar_geometry: &SecondaryGeometry,
) -> Result<PostRenderScheduleAfter, PostRenderMeterError> {
    validate_shared_clock(before, packet)?;
    let object_proposal =
        stage_object_render_callback_with_geometry(&before.object_renderer, packet, oar_geometry)
            .map_err(PostRenderMeterError::ObjectRenderer)?;
    let (object_after, object_output) = object_proposal.into_parts();
    let gain_proposal = stage_shared_gain_delay_callback(
        &before.shared_gain_delay,
        &object_output,
        gain_parameters,
    )
    .map_err(PostRenderMeterError::GainDelay)?;
    let (gain_output, gain_after) = gain_proposal.into_parts();
    let sample_views = std::array::from_fn(|lane| gain_output[lane].as_slice());
    let callback_index = before.sample_queue.next_callback_index();
    let upstream_generation = MeterUpstreamGeneration::new(packet.generation().value());
    let phase = match packet.phase() {
        ObjectRenderCallbackPhase::Normal => MeterCallbackPhase::NormalSource,
        ObjectRenderCallbackPhase::Drain => MeterCallbackPhase::ZeroInputDrain,
    };
    let queue_proposal = stage_sample_block_queue_callback(
        &before.sample_queue,
        MeterCallbackFrame::new(callback_index, upstream_generation, phase, sample_views),
    )
    .map_err(PostRenderMeterError::SampleQueue)?;
    let queue_receipt = *queue_proposal.receipt();
    let window_clock = before
        .window_clock
        .stage_callback(&queue_receipt)
        .map_err(PostRenderMeterError::WindowClock)?;
    let (queue_after, complete_blocks, queue_receipt) = queue_proposal.into_parts();
    Ok(PostRenderScheduleAfter {
        object_renderer: object_after,
        shared_gain_delay: gain_after,
        sample_queue: queue_after,
        window_clock,
        complete_blocks,
        queue_receipt,
    })
}

fn stage_post_render_numeric(
    before: &PostRenderMeterState,
    complete_blocks: &[CompleteSampleBlock],
    k_matrix: &KMatrixProvider,
    parameters: &PostRenderMeterParameters,
) -> Result<PostRenderNumericAfter, PostRenderMeterError> {
    let mut true_peak_after = before.true_peak.clone();
    let mut energy_after = before.energy_window.clone();
    let mut integrated_after = Arc::clone(&before.integrated_meter);
    let mut ungated_after = before.ungated_mean.clone();
    let mut energy_record_count = 0_u64;
    for block in complete_blocks {
        let block_rows = block.lanes().map(<[u32]>::to_vec);
        true_peak_after = process_true_peak_block_with_parameters(
            &true_peak_after,
            &block_rows,
            parameters.true_peak(),
        )
        .map_err(PostRenderMeterError::TruePeak)?
        .next_state;
        let block_views = std::array::from_fn(|lane| block_rows[lane].as_slice());
        let energy_proposal = stage_energy_window_block(&energy_after, &block_views, k_matrix)
            .map_err(PostRenderMeterError::EnergyWindow)?;
        let (next_energy, records) = energy_proposal.into_parts();
        energy_after = next_energy;
        for record in records {
            integrated_after = Arc::new(
                process_integrated_meter(
                    &integrated_after,
                    IntegratedMeterInput {
                        mantissa_word: record.mantissa_word(),
                        exponent: record.exponent(),
                    },
                    parameters.integrated(),
                )
                .map_err(PostRenderMeterError::IntegratedMeter)?
                .into_next_state(),
            );
            ungated_after = process_ungated_mean(
                &ungated_after,
                UngatedMeanInput {
                    mantissa_word: record.mantissa_word(),
                    exponent: record.exponent(),
                    ungated_enabled: true,
                },
                parameters.ungated(),
            )
            .map_err(PostRenderMeterError::UngatedMean)?
            .into_next_state();
            energy_record_count = energy_record_count
                .checked_add(1)
                .ok_or(PostRenderMeterError::ArithmeticOverflow)?;
        }
    }
    Ok(PostRenderNumericAfter {
        true_peak: true_peak_after,
        energy_window: energy_after,
        integrated_meter: integrated_after,
        ungated_mean: ungated_after,
        energy_records: EnergyRecordCount(energy_record_count),
    })
}

fn validate_shared_clock(
    state: &PostRenderMeterState,
    packet: &ObjectRenderCallbackPacket,
) -> Result<(), PostRenderMeterError> {
    let callback = state.object_renderer.callback_index().value();
    if state.sample_queue.next_callback_index().value() != callback
        || u64::from(state.shared_gain_delay.callback_index().value()) != callback
        || packet.generation().value() != callback + 1
        || state.object_renderer.source_samples().value()
            != state.sample_queue.source_samples().value()
    {
        return Err(PostRenderMeterError::ClockMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meter::{
        k_matrix::{KMatrixMode, KMatrixProfile, generate_k_matrix},
        lfe_delay_ring::MeterFrontendCallbackInput,
        object_renderer::{ObjectRenderCallbackGeneration, ObjectRenderCallbackPhase},
        sample_block_queue::{
            MeterCallbackIndex, MeterCallbackPhase, MeterSampleLane, MeterSourceSampleCount,
        },
    };
    use serde_json::Value;

    const TEST_CALLBACK_SAMPLES: usize = 1_536;
    const TEST_REGULAR_SOURCE_COUNT: usize = 15;

    const SELECTED_KERNEL_Q15: [u16; 32] = [
        20, 177, 491, 958, 1_573, 2_331, 3_224, 4_244, 5_381, 6_624, 7_961, 9_379, 10_864, 12_403,
        13_980, 15_580, 17_188, 18_788, 20_365, 21_904, 23_389, 24_807, 26_144, 27_387, 28_524,
        29_544, 30_437, 31_195, 31_810, 32_277, 32_591, 32_748,
    ];

    fn k_matrix() -> KMatrixProvider {
        generate_k_matrix(KMatrixProfile {
            sample_rate_hz: 48_000,
            mode: KMatrixMode::Modern,
            stage_count: 2,
        })
        .unwrap()
    }

    fn controls(fixture: &Value) -> Vec<Vec<u32>> {
        let source_count =
            usize::try_from(fixture["oar_packet"]["source_count"].as_u64().unwrap()).unwrap();
        let direct: Vec<u32> = fixture["oar_packet"]["direct_control_words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
            .collect();
        let object: Vec<u32> = fixture["oar_packet"]["object_control_words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
            .collect();
        let mut controls = vec![object; source_count];
        controls[0] = direct;
        controls
    }

    fn packet(
        fixture: &Value,
        generation: u64,
        phase: ObjectRenderCallbackPhase,
    ) -> ObjectRenderCallbackPacket {
        let source_count =
            usize::try_from(fixture["oar_packet"]["source_count"].as_u64().unwrap()).unwrap();
        let samples_per_source = usize::try_from(
            fixture["oar_packet"]["samples_per_source"]
                .as_u64()
                .unwrap(),
        )
        .unwrap();
        let pcm_word =
            u32::try_from(fixture["oar_packet"]["pcm_f32_word"].as_u64().unwrap()).unwrap();
        ObjectRenderCallbackPacket::new(
            ObjectRenderCallbackGeneration::try_new(generation).unwrap(),
            phase,
            controls(fixture),
            vec![vec![pcm_word; samples_per_source]; source_count],
        )
    }

    fn frontend_input(
        fixture: &Value,
        callback_index: u64,
        phase: MeterCallbackPhase,
    ) -> MeterFrontendCallbackInput {
        let controls = controls(fixture);
        let regular_controls = controls.into_iter().skip(1).collect();
        MeterFrontendCallbackInput::new(
            MeterCallbackIndex::new(callback_index),
            ObjectRenderCallbackGeneration::try_new(callback_index + 1).unwrap(),
            phase,
            vec![0; TEST_CALLBACK_SAMPLES],
            vec![vec![0; TEST_CALLBACK_SAMPLES]; TEST_REGULAR_SOURCE_COUNT],
            regular_controls,
        )
    }

    #[test]
    fn zero_programme_runs_one_normal_three_drains_through_every_post_render_leaf() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/post_render_meter/zero_programme.json"
        ))
        .unwrap();
        let k_matrix = k_matrix();
        let gain_parameters = SharedGainDelayParameters::selected(&SELECTED_KERNEL_Q15).unwrap();
        let source_samples = fixture["source_samples"].as_u64().unwrap();
        let normal_callbacks = fixture["normal_callbacks"].as_u64().unwrap();
        let total_callbacks = normal_callbacks + fixture["zero_input_drains"].as_u64().unwrap();
        let expected = &fixture["expected_lifecycle"];
        let mut state = PostRenderMeterState::cold(source_samples, &k_matrix).unwrap();
        let mut lfe_state =
            MeterLfeDelayState::cold(MeterSourceSampleCount::new(source_samples).unwrap()).unwrap();
        let cold = state.clone();
        let cold_lfe = lfe_state.clone();
        let mut complete_blocks = 0_u64;
        let mut energy_records = 0_u64;
        for callback in 0..total_callbacks {
            let (object_phase, meter_phase) = if callback < normal_callbacks {
                (
                    ObjectRenderCallbackPhase::Normal,
                    MeterCallbackPhase::NormalSource,
                )
            } else {
                (
                    ObjectRenderCallbackPhase::Drain,
                    MeterCallbackPhase::ZeroInputDrain,
                )
            };
            let proposal = stage_independent_meter_callback(
                &lfe_state,
                &state,
                frontend_input(&fixture, callback, meter_phase),
                &gain_parameters,
                &k_matrix,
            )
            .unwrap();
            assert_eq!(
                proposal
                    .after_post_render()
                    .object_renderer()
                    .callback_index()
                    .value(),
                callback + 1
            );
            assert_eq!(proposal.after_lfe().callback_index().value(), callback + 1);
            assert_eq!(
                proposal
                    .lfe_output_origin()
                    .map(|origin| origin.callback_index().value()),
                if callback >= 2 {
                    Some(callback - 2)
                } else {
                    None
                }
            );
            assert_eq!(
                object_phase,
                if callback < normal_callbacks {
                    ObjectRenderCallbackPhase::Normal
                } else {
                    ObjectRenderCallbackPhase::Drain
                }
            );
            complete_blocks += proposal.receipt().completed_blocks().value();
            energy_records += proposal.receipt().energy_records().value();
            let (next_lfe, next_post_render) = proposal.into_parts();
            lfe_state = next_lfe;
            state = next_post_render;
        }

        assert_eq!(cold.object_renderer().callback_index().value(), 0);
        assert_eq!(cold_lfe.callback_index().value(), 0);
        assert_eq!(lfe_state.callback_index().value(), total_callbacks);
        assert_eq!(
            state.object_renderer().callback_index().value(),
            total_callbacks
        );
        assert_eq!(
            state.shared_gain_delay().callback_index().value(),
            u32::try_from(total_callbacks).unwrap()
        );
        assert_eq!(
            state.sample_queue().next_callback_index().value(),
            total_callbacks
        );
        assert_eq!(
            state.sample_queue().published_samples().value(),
            expected["published_samples_per_lane"].as_u64().unwrap()
        );
        assert_eq!(
            complete_blocks,
            expected["complete_1024_blocks"].as_u64().unwrap()
        );
        assert_eq!(energy_records, expected["energy_records"].as_u64().unwrap());
        assert_eq!(
            state.sample_queue().complete_block_count().value(),
            expected["complete_1024_blocks"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(state.sample_queue().block_fill_cursor().value()),
            expected["raw_pending_samples_per_lane"].as_u64().unwrap()
        );
        assert_eq!(
            state.energy_window().samples_in_window(),
            u16::try_from(expected["energy_window_partial_samples"].as_u64().unwrap()).unwrap()
        );
        assert!(
            state
                .sample_queue()
                .pending_lanes()
                .iter()
                .all(|lane| lane.len() == 512 && lane.iter().all(|word| *word == 0))
        );
        assert_eq!(
            state
                .sample_queue()
                .finalization_view()
                .unwrap()
                .raw_tail_lanes()[MeterSampleLane::Left as usize]
                .len(),
            usize::try_from(expected["raw_pending_samples_per_lane"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            u64::from(state.integrated_meter().counter()),
            expected["integrated_energy_pair_count"].as_u64().unwrap()
        );
    }

    #[test]
    fn clock_failure_returns_no_after_state() {
        let k_matrix = k_matrix();
        let gain_parameters = SharedGainDelayParameters::selected(&SELECTED_KERNEL_Q15).unwrap();
        let state = PostRenderMeterState::cold(1, &k_matrix).unwrap();
        assert_eq!(
            stage_post_render_meter_callback(
                &state,
                &packet(
                    &serde_json::from_str(include_str!(
                        "../../tests/fixtures/post_render_meter/zero_programme.json"
                    ))
                    .unwrap(),
                    2,
                    ObjectRenderCallbackPhase::Normal,
                ),
                &gain_parameters,
                &k_matrix,
            )
            .err(),
            Some(PostRenderMeterError::ClockMismatch)
        );
        assert_eq!(state.object_renderer().callback_index().value(), 0);
    }
}
