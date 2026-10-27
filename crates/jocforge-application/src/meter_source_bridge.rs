// pattern: Imperative Shell
//! Typed application boundary between physical PCM frames and the independent meter/OAR owner.

pub use jocforge_domain::{
    frontend::{
        adm_file_storage_map::{
            AdmFileStorageChannelEntry, AdmFileStorageChannelMap, AdmFileStorageMapError,
            AdmFileStorageTrackSlot, ConverterChannelInputId,
        },
        dbmd_sections::{DbmdTrackFields, SelectedDbmdSections},
        job_configuration::{SelectedInputKind, SelectedWaveDescriptor},
        selected_adm_graph::{
            ChnaTrackIndex, PhysicalHostIndex, SelectedAdmGraph, SelectedPhysicalTrack,
            SelectedTrackKind,
        },
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

use crate::{PreparedJobInput, PreparedJobInputError, ResourceIdentity};

const PHYSICAL_CHANNEL_COUNT: usize = 38;
const CALLBACK_SAMPLES: u64 = 1_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterSourceBridgeError {
    ResourceIdentityMismatch,
    PreparedMetadataProvenanceMismatch,
    ConverterStorageMap(AdmFileStorageMapError),
    PhysicalRowBindingMismatch,
    DbmdRowBindingMismatch,
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

/// Zero-based interleaved/planar PCM channel row, distinct from the CHNA track
/// index, physical-host control slot, and converter semantic input identifier.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PcmPhysicalRowIndex(u8);

impl PcmPhysicalRowIndex {
    const fn from_validated_track_slot(value: u8) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

/// DBMD per-track array ordinal, bound to the same physical-host row.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DbmdPhysicalTrackIndex(u16);

impl DbmdPhysicalTrackIndex {
    const fn from_validated_host(value: u16) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u16 {
        self.0
    }
}

/// Immutable selected graph/DBMD route identity joined to one PCM planar row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeterSourcePhysicalRowBinding {
    pcm_row: PcmPhysicalRowIndex,
    track_index: ChnaTrackIndex,
    physical_host: PhysicalHostIndex,
    channel_input_id: ConverterChannelInputId,
    dbmd_track_index: DbmdPhysicalTrackIndex,
}

impl MeterSourcePhysicalRowBinding {
    pub const fn pcm_row(self) -> PcmPhysicalRowIndex {
        self.pcm_row
    }

    pub const fn track_index(self) -> ChnaTrackIndex {
        self.track_index
    }

    pub const fn physical_host(self) -> PhysicalHostIndex {
        self.physical_host
    }

    /// Semantic route value from the slot-keyed map; it is not a PCM row key.
    pub const fn channel_input_id(self) -> ConverterChannelInputId {
        self.channel_input_id
    }

    pub const fn dbmd_track_index(self) -> DbmdPhysicalTrackIndex {
        self.dbmd_track_index
    }
}

/// Prepared resource and exact per-track route bindings for the cursor's
/// physical PCM rows. No source, callback, meter, or programme state is owned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterSourcePhysicalRowMap {
    resource_identity: ResourceIdentity,
    axml_sha256: [u8; 32],
    chna_sha256: [u8; 32],
    dbmd_sha256: [u8; 32],
    entries_by_pcm_row: Box<[MeterSourcePhysicalRowBinding; PHYSICAL_CHANNEL_COUNT]>,
}

impl MeterSourcePhysicalRowMap {
    pub const fn resource_identity(&self) -> ResourceIdentity {
        self.resource_identity
    }

    pub const fn axml_sha256(&self) -> &[u8; 32] {
        &self.axml_sha256
    }

    pub const fn chna_sha256(&self) -> &[u8; 32] {
        &self.chna_sha256
    }

    pub const fn dbmd_sha256(&self) -> &[u8; 32] {
        &self.dbmd_sha256
    }

    pub const fn entries_by_pcm_row(
        &self,
    ) -> &[MeterSourcePhysicalRowBinding; PHYSICAL_CHANNEL_COUNT] {
        &self.entries_by_pcm_row
    }

    pub fn binding_for_pcm_row(
        &self,
        pcm_row: PcmPhysicalRowIndex,
    ) -> &MeterSourcePhysicalRowBinding {
        &self.entries_by_pcm_row[usize::from(pcm_row.value())]
    }

    pub fn binding_for_physical_host(
        &self,
        physical_host: PhysicalHostIndex,
    ) -> Option<&MeterSourcePhysicalRowBinding> {
        self.entries_by_pcm_row
            .get(usize::from(physical_host.value()))
            .filter(|binding| binding.physical_host == physical_host)
    }
}

fn build_meter_source_physical_row_map(
    prepared: &PreparedJobInput,
) -> Result<MeterSourcePhysicalRowMap, MeterSourceBridgeError> {
    let graph = prepared.graph().graph();
    let provenance = prepared.graph().provenance();
    let resource_identity = prepared.resource_identity();
    let graph_receipt = graph.receipt();
    if provenance.parent_resource() != resource_identity
        || provenance.axml().payload_sha256() != *graph_receipt.axml_sha256()
        || provenance.chna().payload_sha256() != *graph_receipt.chna_sha256()
        || provenance.dbmd().payload_sha256() != *prepared.dbmd().payload_sha256()
    {
        return Err(MeterSourceBridgeError::PreparedMetadataProvenanceMismatch);
    }

    let storage_map = prepared
        .converter_storage_map()
        .map_err(|error| match error {
            PreparedJobInputError::ConverterStorageMap(error) => {
                MeterSourceBridgeError::ConverterStorageMap(error)
            }
            _ => MeterSourceBridgeError::PreparedMetadataProvenanceMismatch,
        })?;
    if storage_map.resource_identity() != resource_identity
        || storage_map.axml() != provenance.axml()
        || storage_map.chna() != provenance.chna()
    {
        return Err(MeterSourceBridgeError::PreparedMetadataProvenanceMismatch);
    }

    let entries_by_pcm_row = build_meter_source_physical_row_bindings(
        graph.tracks(),
        storage_map.channels(),
        prepared.dbmd(),
    )?;

    Ok(MeterSourcePhysicalRowMap {
        resource_identity,
        axml_sha256: *graph_receipt.axml_sha256(),
        chna_sha256: *graph_receipt.chna_sha256(),
        dbmd_sha256: *prepared.dbmd().payload_sha256(),
        entries_by_pcm_row,
    })
}

fn build_meter_source_physical_row_bindings(
    tracks: &[SelectedPhysicalTrack],
    channels: &AdmFileStorageChannelMap,
    dbmd: &SelectedDbmdSections,
) -> Result<Box<[MeterSourcePhysicalRowBinding; PHYSICAL_CHANNEL_COUNT]>, MeterSourceBridgeError> {
    if tracks.len() != PHYSICAL_CHANNEL_COUNT || dbmd.track_fields().len() != PHYSICAL_CHANNEL_COUNT
    {
        return Err(MeterSourceBridgeError::PhysicalRowBindingMismatch);
    }

    let mut entries_by_pcm_row: [Option<MeterSourcePhysicalRowBinding>; PHYSICAL_CHANNEL_COUNT] =
        [None; PHYSICAL_CHANNEL_COUNT];
    for track in tracks {
        let pcm_row = pcm_row_for_track_index(track.track_index())?;
        let pcm_row_value = pcm_row.value();
        let slot = AdmFileStorageTrackSlot::new(pcm_row_value)
            .map_err(MeterSourceBridgeError::ConverterStorageMap)?;
        let entry = channels.entry_for_slot(slot);
        let dbmd_track = dbmd
            .fields_for_physical_host(u16::from(track.physical_host().value()))
            .ok_or(MeterSourceBridgeError::DbmdRowBindingMismatch)?;
        let binding = bind_selected_physical_track(track, entry, dbmd_track)?;
        let row_index = usize::from(binding.pcm_row.value());
        if entries_by_pcm_row[row_index].replace(binding).is_some() {
            return Err(MeterSourceBridgeError::PhysicalRowBindingMismatch);
        }
    }

    let entries = entries_by_pcm_row
        .into_iter()
        .map(|entry| entry.ok_or(MeterSourceBridgeError::PhysicalRowBindingMismatch))
        .collect::<Result<Vec<_>, _>>()?;
    let entries_by_pcm_row: Box<[MeterSourcePhysicalRowBinding; PHYSICAL_CHANNEL_COUNT]> = entries
        .into_boxed_slice()
        .try_into()
        .map_err(|_| MeterSourceBridgeError::PhysicalRowBindingMismatch)?;

    for dbmd_track in dbmd.track_fields() {
        let physical_host = u8::try_from(dbmd_track.track_index())
            .map_err(|_| MeterSourceBridgeError::DbmdRowBindingMismatch)?;
        if usize::from(physical_host) >= PHYSICAL_CHANNEL_COUNT
            || entries_by_pcm_row[usize::from(physical_host)]
                .dbmd_track_index()
                .value()
                != dbmd_track.track_index()
            || entries_by_pcm_row[usize::from(physical_host)]
                .physical_host()
                .value()
                != physical_host
        {
            return Err(MeterSourceBridgeError::DbmdRowBindingMismatch);
        }
    }
    Ok(entries_by_pcm_row)
}

fn bind_selected_physical_track(
    track: &SelectedPhysicalTrack,
    entry: AdmFileStorageChannelEntry,
    dbmd_track: DbmdTrackFields,
) -> Result<MeterSourcePhysicalRowBinding, MeterSourceBridgeError> {
    let pcm_row = pcm_row_for_track_index(track.track_index())?;
    let pcm_row_value = pcm_row.value();
    let physical_host = track.physical_host();
    let slot = AdmFileStorageTrackSlot::new(pcm_row_value)
        .map_err(MeterSourceBridgeError::ConverterStorageMap)?;
    let expected_channel_input_id = match track.kind() {
        SelectedTrackKind::DirectSpeakersBed { bed_input_id, .. } => bed_input_id,
        SelectedTrackKind::Object { semantic_key } => semantic_key,
    };
    if physical_host.value() != pcm_row_value
        || entry.track_slot() != slot
        || entry.channel_input_id().value() != expected_channel_input_id
    {
        return Err(MeterSourceBridgeError::PhysicalRowBindingMismatch);
    }
    if dbmd_track.track_index() != u16::from(physical_host.value()) {
        return Err(MeterSourceBridgeError::DbmdRowBindingMismatch);
    }
    Ok(MeterSourcePhysicalRowBinding {
        pcm_row,
        track_index: track.track_index(),
        physical_host,
        channel_input_id: entry.channel_input_id(),
        dbmd_track_index: DbmdPhysicalTrackIndex::from_validated_host(dbmd_track.track_index()),
    })
}

fn pcm_row_for_track_index(
    track_index: ChnaTrackIndex,
) -> Result<PcmPhysicalRowIndex, MeterSourceBridgeError> {
    let row = track_index
        .value()
        .checked_sub(1)
        .and_then(|value| u8::try_from(value).ok())
        .filter(|value| usize::from(*value) < PHYSICAL_CHANNEL_COUNT)
        .ok_or(MeterSourceBridgeError::PhysicalRowBindingMismatch)?;
    Ok(PcmPhysicalRowIndex::from_validated_track_slot(row))
}

/// Read-only typed source view passed to the existing spatial frontend producer.
#[must_use]
pub struct MeterSourceCallbackView<'a> {
    frame: MeterSourceCallbackFrame<'a>,
    selected_graph: &'a SelectedAdmGraph,
    direct_lfe_host: PhysicalHostIndex,
    direct_lfe_pcm_row: PcmPhysicalRowIndex,
    physical_row_map: MeterSourcePhysicalRowMap,
}

impl<'a> MeterSourceCallbackView<'a> {
    pub fn new(
        prepared: &'a PreparedJobInput,
        frame: MeterSourceCallbackFrame<'a>,
    ) -> Result<Self, MeterSourceBridgeError> {
        let selected_graph = prepared.graph().graph();
        let direct_lfe_host = selected_graph.receipt().direct_lfe_host();
        let physical_row_map = build_meter_source_physical_row_map(prepared)?;
        let direct_lfe_pcm_row = physical_row_map
            .binding_for_physical_host(direct_lfe_host)
            .map(|binding| binding.pcm_row())
            .ok_or(MeterSourceBridgeError::PhysicalRowBindingMismatch)?;
        let expected_identity =
            MeterSourceIdentity::from_sha256(physical_row_map.resource_identity().input_sha256());
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
            || frame.rows().len() != physical_row_map.entries_by_pcm_row().len()
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
            direct_lfe_pcm_row,
            physical_row_map,
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

    pub const fn physical_row_map(&self) -> &MeterSourcePhysicalRowMap {
        &self.physical_row_map
    }

    pub fn physical_rows(&self) -> &[Vec<u32>] {
        self.frame.rows()
    }

    pub fn direct_lfe_words(&self) -> &[u32] {
        &self.physical_rows()[usize::from(self.direct_lfe_pcm_row.value())]
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

#[cfg(test)]
mod tests {
    use super::*;
    use jocforge_domain::frontend::{
        adm_file_storage_map::build_adm_file_storage_channel_map,
        dbmd_sections::parse_selected_dbmd_sections, selected_adm_graph::parse_selected_adm_graph,
    };

    const AXML: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.xml"
    );
    const CHNA: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_track_associations.bin"
    );
    const DBMD: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.dbmd"
    );

    fn graph_and_dbmd() -> (SelectedAdmGraph, SelectedDbmdSections) {
        (
            parse_selected_adm_graph(AXML, CHNA).unwrap(),
            parse_selected_dbmd_sections(DBMD).unwrap(),
        )
    }

    #[test]
    fn physical_row_map_is_independent_of_graph_iteration_order() {
        let (graph, dbmd) = graph_and_dbmd();
        let channels = build_adm_file_storage_channel_map(&graph).unwrap();
        let forward =
            build_meter_source_physical_row_bindings(graph.tracks(), &channels, &dbmd).unwrap();
        let reversed_tracks = graph.tracks().iter().rev().cloned().collect::<Vec<_>>();
        let reversed =
            build_meter_source_physical_row_bindings(&reversed_tracks, &channels, &dbmd).unwrap();

        assert_eq!(forward.as_ref(), reversed.as_ref());
        for (row, binding) in forward.iter().enumerate() {
            let row = u8::try_from(row).unwrap();
            let slot = AdmFileStorageTrackSlot::new(row).unwrap();
            let channel_entry = channels.entry_for_slot(slot);
            assert_eq!(binding.pcm_row().value(), row);
            assert_eq!(binding.track_index().value(), u16::from(row) + 1);
            assert_eq!(binding.physical_host().value(), row);
            assert_eq!(binding.dbmd_track_index().value(), u16::from(row));
            assert_eq!(binding.channel_input_id(), channel_entry.channel_input_id());
            assert_eq!(channel_entry.track_slot(), slot);
        }
    }

    #[test]
    fn physical_row_binding_rejects_converter_slot_and_dbmd_crosswires() {
        let (graph, dbmd) = graph_and_dbmd();
        let channels = build_adm_file_storage_channel_map(&graph).unwrap();
        let first_track = &graph.tracks()[0];
        let row_zero = AdmFileStorageTrackSlot::new(0).unwrap();
        let row_one = AdmFileStorageTrackSlot::new(1).unwrap();
        let dbmd_zero = dbmd.fields_for_physical_host(0).unwrap();
        let dbmd_one = dbmd.fields_for_physical_host(1).unwrap();

        assert_eq!(
            bind_selected_physical_track(first_track, channels.entry_for_slot(row_one), dbmd_zero)
                .err(),
            Some(MeterSourceBridgeError::PhysicalRowBindingMismatch)
        );
        assert_eq!(
            bind_selected_physical_track(first_track, channels.entry_for_slot(row_zero), dbmd_one)
                .err(),
            Some(MeterSourceBridgeError::DbmdRowBindingMismatch)
        );
    }

    #[test]
    fn physical_row_binding_rejects_invalid_track_indices_and_duplicate_slots() {
        assert_eq!(
            pcm_row_for_track_index(ChnaTrackIndex::new(0)).err(),
            Some(MeterSourceBridgeError::PhysicalRowBindingMismatch)
        );
        assert_eq!(
            pcm_row_for_track_index(ChnaTrackIndex::new(39)).err(),
            Some(MeterSourceBridgeError::PhysicalRowBindingMismatch)
        );

        let (graph, dbmd) = graph_and_dbmd();
        let channels = build_adm_file_storage_channel_map(&graph).unwrap();
        let mut duplicate_tracks = graph.tracks().to_vec();
        duplicate_tracks[1] = duplicate_tracks[0].clone();
        assert_eq!(
            build_meter_source_physical_row_bindings(&duplicate_tracks, &channels, &dbmd).err(),
            Some(MeterSourceBridgeError::PhysicalRowBindingMismatch)
        );
        assert_eq!(
            build_meter_source_physical_row_bindings(&graph.tracks()[..37], &channels, &dbmd).err(),
            Some(MeterSourceBridgeError::PhysicalRowBindingMismatch)
        );
    }

    #[test]
    fn physical_rows_follow_track_index_and_slot_map_not_semantic_id_order() {
        let (graph, dbmd) = graph_and_dbmd();
        let channel_map = build_adm_file_storage_channel_map(&graph).unwrap();
        let forward =
            build_meter_source_physical_row_bindings(graph.tracks(), &channel_map, &dbmd).unwrap();
        let reversed_tracks = graph.tracks().iter().rev().cloned().collect::<Vec<_>>();
        let reversed =
            build_meter_source_physical_row_bindings(&reversed_tracks, &channel_map, &dbmd)
                .unwrap();

        assert_eq!(forward.as_ref(), reversed.as_ref());
        for (physical_row, binding) in forward.iter().enumerate() {
            let row_value = u8::try_from(physical_row).unwrap();
            let slot = AdmFileStorageTrackSlot::new(row_value).unwrap();
            let entry = channel_map.entry_for_slot(slot);

            assert_eq!(binding.pcm_row().value(), row_value);
            assert_eq!(binding.track_index().value(), u16::from(row_value) + 1);
            assert_eq!(binding.physical_host().value(), row_value);
            assert_eq!(binding.dbmd_track_index().value(), u16::from(row_value));
            assert_eq!(entry.track_slot(), slot);
            assert_eq!(binding.channel_input_id(), entry.channel_input_id());
        }
    }

    #[test]
    fn physical_row_binding_rejects_a_converter_entry_from_a_different_slot() {
        let (graph, dbmd) = graph_and_dbmd();
        let channel_map = build_adm_file_storage_channel_map(&graph).unwrap();
        let first_track = &graph.tracks()[0];
        let wrong_slot = AdmFileStorageTrackSlot::new(1).unwrap();
        let wrong_entry = channel_map.entry_for_slot(wrong_slot);
        let dbmd_track = dbmd.fields_for_physical_host(0).unwrap();

        assert_eq!(
            bind_selected_physical_track(first_track, wrong_entry, dbmd_track).err(),
            Some(MeterSourceBridgeError::PhysicalRowBindingMismatch)
        );
    }

    #[test]
    fn physical_row_binding_rejects_dbmd_from_a_different_host() {
        let (graph, dbmd) = graph_and_dbmd();
        let channel_map = build_adm_file_storage_channel_map(&graph).unwrap();
        let first_track = &graph.tracks()[0];
        let slot = AdmFileStorageTrackSlot::new(0).unwrap();
        let entry = channel_map.entry_for_slot(slot);
        let wrong_dbmd_track = dbmd.fields_for_physical_host(1).unwrap();

        assert_eq!(
            bind_selected_physical_track(first_track, entry, wrong_dbmd_track).err(),
            Some(MeterSourceBridgeError::DbmdRowBindingMismatch)
        );
    }

    #[test]
    fn physical_row_binding_rejects_missing_track_coverage() {
        let (graph, dbmd) = graph_and_dbmd();
        let channel_map = build_adm_file_storage_channel_map(&graph).unwrap();

        assert_eq!(
            build_meter_source_physical_row_bindings(&graph.tracks()[..37], &channel_map, &dbmd)
                .err(),
            Some(MeterSourceBridgeError::PhysicalRowBindingMismatch)
        );
    }
}
