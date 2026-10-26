// pattern: Imperative Shell
//! Typed application boundary between physical PCM frames and the independent meter/OAR owner.

pub use jocforge_domain::{
    frontend::{
        job_configuration::{SelectedInputKind, SelectedWaveDescriptor},
        selected_adm_graph::{PhysicalHostIndex, SelectedAdmGraph},
    },
    joc::oamd::reader_target::{ReaderTargetControlLineage, ReaderTargetControlRows},
    meter::{
        k_matrix::{KMatrixMode, KMatrixProfile, generate_k_matrix},
        lfe_delay_ring::{MeterFrontendCallbackInput, MeterLfeError},
        meter_prepass::{
            IndependentMeterCallbackStage, IndependentMeterPrepass, IndependentMeterPrepassError,
        },
        object_renderer::ObjectRenderCallbackGeneration,
        post_render_meter::PostRenderMeterReceipt,
        sample_block_queue::{
            MeterCallbackCount, MeterCallbackIndex, MeterCallbackPhase, MeterSampleCount,
            MeterSourceSampleCount, MeterUpstreamGeneration,
        },
        shared_gain_delay::SharedGainDelayParameters,
        source_frame::{
            MeterSourceCallbackFrame, MeterSourceCallbackFrameParts, MeterSourceIdentity,
            MeterSourcePosition,
        },
    },
    spatial::spatial_audio_state::MeterComponentOutputRows,
    timeline::FrontendCallbackIndex,
    transaction::ProgrammeId,
};

use crate::PreparedJobInput;

const PHYSICAL_CHANNEL_COUNT: usize = 38;
const CALLBACK_SAMPLES: u64 = 1_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterSourceBridgeError {
    ResourceIdentityMismatch,
    SelectedInputKindMismatch,
    SourceExtentMismatch,
    SourceShape,
    SourceClock,
    FrontendClock,
    DirectLfeMismatch,
    SpatialStage(MeterLfeError),
    PrepassClockMismatch,
    Prepass(IndependentMeterPrepassError),
}

/// Read-only typed source view passed to the existing spatial frontend producer.
#[must_use]
pub struct MeterSourceCallbackView<'a> {
    frame: MeterSourceCallbackFrame<'a>,
    selected_graph: &'a SelectedAdmGraph,
    direct_lfe_host: PhysicalHostIndex,
}

impl<'a> MeterSourceCallbackView<'a> {
    pub fn new(
        prepared: &'a PreparedJobInput,
        frame: MeterSourceCallbackFrame<'a>,
    ) -> Result<Self, MeterSourceBridgeError> {
        let expected_identity =
            MeterSourceIdentity::from_sha256(prepared.resource_identity().input_sha256());
        let selected_graph = prepared.graph().graph();
        let direct_lfe_host = selected_graph.receipt().direct_lfe_host();
        if frame.source_identity() != expected_identity {
            return Err(MeterSourceBridgeError::ResourceIdentityMismatch);
        }
        if prepared.configuration().input_kind() != SelectedInputKind::Adm {
            return Err(MeterSourceBridgeError::SelectedInputKindMismatch);
        }
        if frame.source_samples().value() == 0
            || frame.source_samples().value() != prepared.configuration().source_samples()
        {
            return Err(MeterSourceBridgeError::SourceExtentMismatch);
        }
        if frame.rows().len() != PHYSICAL_CHANNEL_COUNT
            || frame
                .rows()
                .iter()
                .any(|row| row.len() != CALLBACK_SAMPLES as usize)
            || usize::from(direct_lfe_host.value()) >= PHYSICAL_CHANNEL_COUNT
        {
            return Err(MeterSourceBridgeError::SourceShape);
        }

        let callback = frame.callback_index().value();
        let generation = callback
            .checked_add(1)
            .ok_or(MeterSourceBridgeError::SourceClock)?;
        let source_samples = frame.source_samples().value();
        let normal_callbacks = source_samples.div_ceil(CALLBACK_SAMPLES);
        let total_callbacks = normal_callbacks
            .checked_add(3)
            .ok_or(MeterSourceBridgeError::SourceClock)?;
        let source_before = frame.source_cursor_before().value();
        let source_after = frame.source_cursor_after().value();
        let host_count = frame.host_input_count().value();
        let padding_count = frame.source_padding_count().value();
        if frame.upstream_generation().value() != generation
            || frame.normal_callbacks().value() != normal_callbacks
            || callback >= total_callbacks
            || source_before > source_samples
            || source_after > source_samples
            || host_count + padding_count != CALLBACK_SAMPLES
            || source_before.checked_add(host_count) != Some(source_after)
        {
            return Err(MeterSourceBridgeError::SourceClock);
        }

        let normal = callback < normal_callbacks;
        let expected_phase = if normal {
            MeterCallbackPhase::NormalSource
        } else {
            MeterCallbackPhase::ZeroInputDrain
        };
        let expected_host_count = if normal {
            source_samples
                .checked_sub(source_before)
                .ok_or(MeterSourceBridgeError::SourceClock)?
                .min(CALLBACK_SAMPLES)
        } else {
            0
        };
        if frame.phase() != expected_phase
            || host_count != expected_host_count
            || padding_count != CALLBACK_SAMPLES - expected_host_count
            || (!normal && (source_before != source_samples || source_after != source_samples))
        {
            return Err(MeterSourceBridgeError::SourceClock);
        }

        for row in frame.rows() {
            if row.iter().any(|word| !f32::from_bits(*word).is_finite()) {
                return Err(MeterSourceBridgeError::SourceShape);
            }
            if row
                [usize::try_from(host_count).map_err(|_| MeterSourceBridgeError::SourceShape)?..]
                .iter()
                .any(|word| *word != 0)
            {
                return Err(MeterSourceBridgeError::SourceShape);
            }
        }

        Ok(Self {
            frame,
            selected_graph,
            direct_lfe_host,
        })
    }

    pub const fn callback_index(&self) -> MeterCallbackIndex {
        self.frame.callback_index()
    }

    pub const fn upstream_generation(
        &self,
    ) -> jocforge_domain::meter::sample_block_queue::MeterUpstreamGeneration {
        self.frame.upstream_generation()
    }

    pub const fn phase(&self) -> MeterCallbackPhase {
        self.frame.phase()
    }

    pub const fn source_cursor_before(&self) -> MeterSourcePosition {
        self.frame.source_cursor_before()
    }

    pub const fn source_cursor_after(&self) -> MeterSourcePosition {
        self.frame.source_cursor_after()
    }

    pub const fn source_samples(&self) -> MeterSourceSampleCount {
        self.frame.source_samples()
    }

    pub const fn normal_callbacks(&self) -> MeterCallbackCount {
        self.frame.normal_callbacks()
    }

    pub const fn host_input_count(&self) -> MeterSampleCount {
        self.frame.host_input_count()
    }

    pub const fn source_padding_count(&self) -> MeterSampleCount {
        self.frame.source_padding_count()
    }

    pub const fn direct_lfe_host(&self) -> PhysicalHostIndex {
        self.direct_lfe_host
    }

    pub fn physical_rows(&self) -> &[Vec<u32>] {
        self.frame.rows()
    }

    pub fn direct_lfe_words(&self) -> &[u32] {
        &self.physical_rows()[usize::from(self.direct_lfe_host.value())]
    }

    pub const fn selected_graph(&self) -> &'a SelectedAdmGraph {
        self.selected_graph
    }

    pub const fn source_identity(&self) -> MeterSourceIdentity {
        self.frame.source_identity()
    }

    /// Binds frontend output to this exact callback and selected direct-LFE row.
    pub fn bind_frontend_input(
        self,
        input: MeterFrontendCallbackInput,
    ) -> Result<MeterSourceOarHandoff, MeterSourceBridgeError> {
        if input.callback_index() != self.frame.callback_index()
            || input.upstream_generation().value() != self.frame.upstream_generation().value()
            || input.phase() != self.frame.phase()
        {
            return Err(MeterSourceBridgeError::FrontendClock);
        }
        if input.current_lfe_words() != self.direct_lfe_words() {
            return Err(MeterSourceBridgeError::DirectLfeMismatch);
        }

        let spatial_callback_index = input.spatial_callback_index();
        let target_control_lineages = input.target_control_lineages().copied();

        Ok(MeterSourceOarHandoff {
            input,
            source_receipt: MeterSourceOarReceipt {
                callback_index: self.frame.callback_index(),
                upstream_generation: self.frame.upstream_generation(),
                phase: self.frame.phase(),
                source_cursor_before: self.frame.source_cursor_before(),
                source_cursor_after: self.frame.source_cursor_after(),
                source_samples: self.frame.source_samples(),
                normal_callbacks: self.frame.normal_callbacks(),
                host_input_count: self.frame.host_input_count(),
                source_padding_count: self.frame.source_padding_count(),
                direct_lfe_host: self.direct_lfe_host,
                resource_identity: self.frame.source_identity(),
                spatial_callback_index,
                target_control_lineages,
            },
        })
    }

    /// Builds and binds the F06OF input from canonical F04A rows and admitted F05R56 controls.
    /// Row labels and history lineage remain target-slot based through the resulting handoff.
    pub fn bind_spatial_outputs(
        self,
        component_rows: &MeterComponentOutputRows,
        target_controls: &ReaderTargetControlRows,
    ) -> Result<MeterSourceOarHandoff, MeterSourceBridgeError> {
        let object_render_generation =
            ObjectRenderCallbackGeneration::try_new(self.frame.upstream_generation().value())
                .map_err(|_| MeterSourceBridgeError::FrontendClock)?;
        let input = MeterFrontendCallbackInput::from_spatial_outputs(
            self.frame.callback_index(),
            object_render_generation,
            self.frame.phase(),
            self.direct_lfe_words().to_vec(),
            component_rows,
            target_controls,
        )
        .map_err(MeterSourceBridgeError::SpatialStage)?;
        self.bind_frontend_input(input)
    }
}

#[must_use]
pub struct MeterSourceOarHandoff {
    input: MeterFrontendCallbackInput,
    source_receipt: MeterSourceOarReceipt,
}

impl MeterSourceOarHandoff {
    pub const fn source_receipt(&self) -> MeterSourceOarReceipt {
        self.source_receipt
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeterSourceOarReceipt {
    callback_index: MeterCallbackIndex,
    upstream_generation: jocforge_domain::meter::sample_block_queue::MeterUpstreamGeneration,
    phase: MeterCallbackPhase,
    source_cursor_before: MeterSourcePosition,
    source_cursor_after: MeterSourcePosition,
    source_samples: MeterSourceSampleCount,
    normal_callbacks: MeterCallbackCount,
    host_input_count: MeterSampleCount,
    source_padding_count: MeterSampleCount,
    direct_lfe_host: PhysicalHostIndex,
    resource_identity: MeterSourceIdentity,
    spatial_callback_index: Option<FrontendCallbackIndex>,
    target_control_lineages: Option<[ReaderTargetControlLineage; 15]>,
}

impl MeterSourceOarReceipt {
    pub const fn callback_index(self) -> MeterCallbackIndex {
        self.callback_index
    }

    pub const fn upstream_generation(
        self,
    ) -> jocforge_domain::meter::sample_block_queue::MeterUpstreamGeneration {
        self.upstream_generation
    }

    pub const fn phase(self) -> MeterCallbackPhase {
        self.phase
    }

    pub const fn source_cursor_before(self) -> MeterSourcePosition {
        self.source_cursor_before
    }

    pub const fn source_cursor_after(self) -> MeterSourcePosition {
        self.source_cursor_after
    }

    pub const fn source_samples(self) -> MeterSourceSampleCount {
        self.source_samples
    }

    pub const fn normal_callbacks(self) -> MeterCallbackCount {
        self.normal_callbacks
    }

    pub const fn host_input_count(self) -> MeterSampleCount {
        self.host_input_count
    }

    pub const fn source_padding_count(self) -> MeterSampleCount {
        self.source_padding_count
    }

    pub const fn direct_lfe_host(self) -> PhysicalHostIndex {
        self.direct_lfe_host
    }

    pub const fn resource_identity(self) -> MeterSourceIdentity {
        self.resource_identity
    }

    pub const fn spatial_callback_index(self) -> Option<FrontendCallbackIndex> {
        self.spatial_callback_index
    }

    pub const fn target_control_lineages(self) -> Option<[ReaderTargetControlLineage; 15]> {
        self.target_control_lineages
    }
}

#[must_use]
pub struct MeterSourceOarStage<'a> {
    prepass_stage: IndependentMeterCallbackStage<'a>,
    source_receipt: MeterSourceOarReceipt,
}

impl MeterSourceOarStage<'_> {
    /// Commits the application-owned meter proposal. The CLI composition root
    /// installs the IO cursor after this infallible state transition.
    pub fn commit(self) -> (PostRenderMeterReceipt, MeterSourceOarReceipt) {
        (self.prepass_stage.commit(), self.source_receipt)
    }
}

pub fn stage_meter_source_oar_callback(
    prepass: &mut IndependentMeterPrepass,
    handoff: MeterSourceOarHandoff,
) -> Result<MeterSourceOarStage<'_>, MeterSourceBridgeError> {
    let receipt = handoff.source_receipt;
    if prepass.source_samples() != receipt.source_samples
        || prepass.normal_callback_count() != receipt.normal_callbacks
        || prepass.next_callback_index() != receipt.callback_index
    {
        return Err(MeterSourceBridgeError::PrepassClockMismatch);
    }
    let expected_phase = if receipt.callback_index.value() < receipt.normal_callbacks.value() {
        MeterCallbackPhase::NormalSource
    } else {
        MeterCallbackPhase::ZeroInputDrain
    };
    if receipt.phase != expected_phase {
        return Err(MeterSourceBridgeError::PrepassClockMismatch);
    }

    let prepass_stage = prepass
        .stage_callback(handoff.input)
        .map_err(MeterSourceBridgeError::Prepass)?;
    Ok(MeterSourceOarStage {
        prepass_stage,
        source_receipt: receipt,
    })
}

/// The CLI composition root supplies the IO cursor's typed snapshot here.
pub fn ready_to_finalize(
    prepass: &IndependentMeterPrepass,
    source_callback_index: MeterCallbackIndex,
    source_cursor_after: MeterSourcePosition,
    source_samples: MeterSourceSampleCount,
    normal_callbacks: MeterCallbackCount,
) -> bool {
    let Some(total_callbacks) = normal_callbacks.value().checked_add(3) else {
        return false;
    };
    prepass.source_samples() == source_samples
        && prepass.normal_callback_count() == normal_callbacks
        && prepass.next_callback_index().value() == total_callbacks
        && source_callback_index.value() == total_callbacks
        && source_cursor_after.value() == source_samples.value()
        && prepass.ready_to_finalize()
}
