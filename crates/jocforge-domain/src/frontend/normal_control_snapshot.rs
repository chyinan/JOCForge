// pattern: Functional Core
//! Noncommitting F01E normal-callback control snapshot proposal.

use std::sync::Arc;

use serde_json::{Number, Value, json};

use crate::numeric::FiniteF32;
use crate::timeline::{AdmSampleCount, FrontendCallbackIndex, NormalCallbackCount};
use crate::transaction::{AttemptId, CallbackLineageIdentity, ProgrammeId};

use super::adm_object_events::{AdmittedSelectedObjectEvent, AdmittedSelectedObjectEvents};
use super::dbmd_sections::{DbmdTrackFields, SelectedDbmdSections};
use super::job_configuration::{
    PrimaryMetadataChunkHashes, SelectedInputKind, SelectedPrimaryControlBinding,
    SelectedWaveDescriptor,
};
use super::object_control::{
    ObjectControl52, ObjectControlError, SelectedObjectEvent, sample_event_from_previous,
    writer_coordinates, writer_uniform_width,
};
use super::selected_adm_graph::{
    SelectedAdmGraph, SelectedAdmGraphReceipt, SelectedPhysicalTrack, SelectedTrackKind,
};
use super::state::{
    FrontendControlDigest, FrontendControlPhase, FrontendControlState, FrontendNormalizedConfig,
    FrontendObjectState,
};

const PHYSICAL_TRACK_COUNT: usize = 38;
const BED_TRACK_COUNT: usize = 10;
const OBJECT_TRACK_COUNT: usize = 28;
const CONTROL_WORDS_PER_TRACK: usize = 13;
const CONTROL38_WORD_COUNT: usize = PHYSICAL_TRACK_COUNT * CONTROL_WORDS_PER_TRACK;
const REGULAR_TRACK_COUNT: usize = PHYSICAL_TRACK_COUNT - 1;
const REGULAR37_WORD_COUNT: usize = REGULAR_TRACK_COUNT * CONTROL_WORDS_PER_TRACK;
const NORMAL_CALLBACK_SAMPLES: u64 = 1_536;
const MAIN_CACHED_DRAIN_CALLBACKS: u32 = 4;
const SELECTED_SAMPLE_RATE_HZ: u32 = 48_000;
const SELECTED_CHANNELS: u32 = 38;
const SELECTED_PCM_BITS: u32 = 24;

fn optional_f64_json(value: Option<f64>) -> Result<Value, NormalControlSnapshotError> {
    match value {
        Some(value) => Number::from_f64(value)
            .map(Value::Number)
            .ok_or(NormalControlSnapshotError::CanonicalSerialization),
        None => Ok(Value::Null),
    }
}

fn dbmd_fields_json(fields: NormalControlDbmdFields) -> Value {
    json!({
        "track_index": fields.track_index(),
        "enum48": fields.enum48(),
        "enum52": fields.enum52(),
        "flag64": fields.flag64(),
        "raw_flag_low2": fields.raw_flag_low2(),
        "raw_packed": fields.raw_packed(),
    })
}

fn hex_sha256(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NormalControlSnapshotError {
    ColdStateRequired,
    SourceConfigurationMismatch,
    MetadataIdentityMismatch,
    GraphDomain,
    PhysicalHostDomain,
    PhysicalIdentityMismatch,
    ObjectOwnerBindingMismatch,
    ObjectEventMismatch,
    BedJoinMismatch,
    LfeIdentityMismatch,
    DbmdIdentityMismatch,
    CallbackIdentityMismatch,
    CallbackClockMismatch,
    GenerationMismatch,
    PreparedIdentityMismatch,
    StateShapeMismatch,
    StateOwnerMismatch,
    StateSamplerInvalid,
    SnapshotShapeMismatch,
    IndexOverflow,
    ObjectControl(ObjectControlError),
    CanonicalSerialization,
}

impl NormalControlSnapshotError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ColdStateRequired => "NORMAL_CONTROL_COLD_STATE",
            Self::SourceConfigurationMismatch => "NORMAL_CONTROL_SOURCE_CONFIGURATION",
            Self::MetadataIdentityMismatch => "NORMAL_CONTROL_METADATA_IDENTITY",
            Self::GraphDomain => "NORMAL_CONTROL_GRAPH_DOMAIN",
            Self::PhysicalHostDomain => "NORMAL_CONTROL_PHYSICAL_HOST",
            Self::PhysicalIdentityMismatch => "NORMAL_CONTROL_PHYSICAL_IDENTITY",
            Self::ObjectOwnerBindingMismatch => "NORMAL_CONTROL_OWNER_BINDING",
            Self::ObjectEventMismatch => "NORMAL_CONTROL_OBJECT_EVENT",
            Self::BedJoinMismatch => "NORMAL_CONTROL_BED_JOIN",
            Self::LfeIdentityMismatch => "NORMAL_CONTROL_LFE_IDENTITY",
            Self::DbmdIdentityMismatch => "NORMAL_CONTROL_DBMD_IDENTITY",
            Self::CallbackIdentityMismatch => "NORMAL_CONTROL_CALLBACK_IDENTITY",
            Self::CallbackClockMismatch => "NORMAL_CONTROL_CALLBACK_CLOCK",
            Self::GenerationMismatch => "NORMAL_CONTROL_GENERATION",
            Self::PreparedIdentityMismatch => "NORMAL_CONTROL_PREPARED_IDENTITY",
            Self::StateShapeMismatch => "NORMAL_CONTROL_STATE_SHAPE",
            Self::StateOwnerMismatch => "NORMAL_CONTROL_STATE_OWNER",
            Self::StateSamplerInvalid => "NORMAL_CONTROL_SAMPLER_STATE",
            Self::SnapshotShapeMismatch => "NORMAL_CONTROL_SNAPSHOT_SHAPE",
            Self::IndexOverflow => "NORMAL_CONTROL_INDEX_OVERFLOW",
            Self::ObjectControl(error) => error.code(),
            Self::CanonicalSerialization => "NORMAL_CONTROL_CANONICAL_SERIALIZATION",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalControlProfileIdentity {
    sample_rate_hz: u32,
    channels: u32,
    pcm_bits: u32,
    source_samples: u64,
    bit_rate_kbps: u32,
    bit_rate_enum: u32,
    layout_enum: u32,
    target_capacity: u32,
    frame_samples: u32,
    spatial_config_mode: u32,
    dispatch_mode: u32,
    virtual_prefix_count: u32,
    bed_count: u32,
    object_count: u32,
    event_start_samples: u64,
    line_drc: String,
    rf_drc: String,
    time_base: String,
}

impl NormalControlProfileIdentity {
    fn from_config(config: &FrontendNormalizedConfig) -> Self {
        Self {
            sample_rate_hz: config.sample_rate_hz(),
            channels: config.channels(),
            pcm_bits: config.pcm_bits(),
            source_samples: config.source_samples(),
            bit_rate_kbps: config.bit_rate_kbps(),
            bit_rate_enum: config.bit_rate_enum(),
            layout_enum: config.layout_enum(),
            target_capacity: config.target_capacity(),
            frame_samples: config.frame_samples(),
            spatial_config_mode: config.spatial_config_mode(),
            dispatch_mode: config.dispatch_mode(),
            virtual_prefix_count: config.virtual_prefix_count(),
            bed_count: config.bed_count(),
            object_count: config.object_count(),
            event_start_samples: config.event_start_samples(),
            line_drc: config.line_drc().to_owned(),
            rf_drc: config.rf_drc().to_owned(),
            time_base: config.time_base().to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalControlSourceIdentity {
    selected_profile: crate::profile::ResolvedEncodingProfile,
    input_kind: SelectedInputKind,
    source_file_name: String,
    source_file_bytes: u64,
    data_bytes: u64,
    wave: SelectedWaveDescriptor,
    source_samples: u64,
    metadata_hashes: PrimaryMetadataChunkHashes,
    job_projection_sha256: [u8; 32],
    source_receipt_sha256: [u8; 32],
    original_job_sha256_declaration: [u8; 32],
    original_job_bytes_declaration: u64,
    original_job_bytes_available: bool,
    full_pcm_identity_rehashed: bool,
    runtime_resource_bound: bool,
}

impl NormalControlSourceIdentity {
    fn from_binding(binding: &SelectedPrimaryControlBinding) -> Self {
        Self {
            selected_profile: binding.selected_profile(),
            input_kind: binding.input_kind(),
            source_file_name: binding.source_file_name().to_owned(),
            source_file_bytes: binding.source_file_bytes(),
            data_bytes: binding.data_bytes(),
            wave: binding.wave(),
            source_samples: binding.source_samples().value(),
            metadata_hashes: binding.metadata_hashes(),
            job_projection_sha256: *binding.job_projection_sha256(),
            source_receipt_sha256: *binding.source_receipt_sha256(),
            original_job_sha256_declaration: *binding.original_job_sha256_declaration(),
            original_job_bytes_declaration: binding.original_job_bytes_declaration(),
            original_job_bytes_available: binding.original_job_bytes_available(),
            full_pcm_identity_rehashed: binding.full_pcm_identity_rehashed(),
            runtime_resource_bound: binding.runtime_resource_bound(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalControlGraphIdentity {
    axml_sha256: [u8; 32],
    chna_sha256: [u8; 32],
    physical_track_count: u8,
    bed_track_count: u8,
    object_track_count: u8,
    direct_lfe_host: u8,
    virtual_prefix_source_count: u8,
    total_source_count: u8,
}

impl NormalControlGraphIdentity {
    fn from_receipt(receipt: &SelectedAdmGraphReceipt) -> Self {
        Self {
            axml_sha256: *receipt.axml_sha256(),
            chna_sha256: *receipt.chna_sha256(),
            physical_track_count: receipt.physical_track_count(),
            bed_track_count: receipt.bed_track_count(),
            object_track_count: receipt.object_track_count(),
            direct_lfe_host: receipt.direct_lfe_host().value(),
            virtual_prefix_source_count: receipt.virtual_prefix_source_count(),
            total_source_count: receipt.total_source_count(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NormalControlDbmdFields {
    track_index: u16,
    enum48: i8,
    enum52: i8,
    flag64: u8,
    raw_flag_low2: u8,
    raw_packed: u8,
}

impl NormalControlDbmdFields {
    fn from_fields(fields: DbmdTrackFields) -> Self {
        Self {
            track_index: fields.track_index(),
            enum48: fields.enum48(),
            enum52: fields.enum52(),
            flag64: fields.flag64(),
            raw_flag_low2: fields.raw_flag_low2(),
            raw_packed: fields.raw_packed(),
        }
    }

    pub const fn track_index(self) -> u16 {
        self.track_index
    }

    pub const fn enum48(self) -> i8 {
        self.enum48
    }

    pub const fn enum52(self) -> i8 {
        self.enum52
    }

    pub const fn flag64(self) -> u8 {
        self.flag64
    }

    pub const fn raw_flag_low2(self) -> u8 {
        self.raw_flag_low2
    }

    pub const fn raw_packed(self) -> u8 {
        self.raw_packed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NormalControlTrackKind {
    Bed {
        bed_id: u8,
        bed_pack_ordinal: u8,
        semantic_role: String,
        bed_input_id: u32,
    },
    Object {
        semantic_key: u32,
        local_owner_ordinal: u8,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalControlPhysicalTrack {
    track_uid: String,
    object_id: String,
    physical_host: u8,
    channel_format_id: String,
    kind: NormalControlTrackKind,
    dbmd: NormalControlDbmdFields,
}

impl NormalControlPhysicalTrack {
    pub fn track_uid(&self) -> &str {
        &self.track_uid
    }

    pub const fn physical_host(&self) -> u8 {
        self.physical_host
    }

    pub fn channel_format_id(&self) -> &str {
        &self.channel_format_id
    }

    pub fn kind(&self) -> &NormalControlTrackKind {
        &self.kind
    }

    pub const fn dbmd(&self) -> NormalControlDbmdFields {
        self.dbmd
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalControlObjectOwnerBinding {
    track_uid: String,
    local_owner_ordinal: u8,
    semantic_key: u32,
    physical_host: u8,
    channel_format_id: String,
}

impl NormalControlObjectOwnerBinding {
    pub fn track_uid(&self) -> &str {
        &self.track_uid
    }

    pub const fn local_owner_ordinal(&self) -> u8 {
        self.local_owner_ordinal
    }

    pub const fn semantic_key(&self) -> u32 {
        self.semantic_key
    }

    pub const fn physical_host(&self) -> u8 {
        self.physical_host
    }

    pub fn channel_format_id(&self) -> &str {
        &self.channel_format_id
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NormalControlRegularSourceBinding {
    physical_host: u8,
    global_source: u64,
}

impl NormalControlRegularSourceBinding {
    pub const fn physical_host(self) -> u8 {
        self.physical_host
    }

    pub const fn global_source(self) -> u64 {
        self.global_source
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NormalControlObjectEvent {
    track_uid: String,
    object_id: String,
    physical_host: u8,
    object_start: u64,
    block_rtime: u64,
    interpolation_length: u64,
    object_duration: u64,
    block_duration: u64,
    control_event: SelectedObjectEvent,
}

impl NormalControlObjectEvent {
    fn from_admitted(event: &AdmittedSelectedObjectEvent) -> Self {
        Self {
            track_uid: event.track_uid().to_owned(),
            object_id: event.object_id().to_owned(),
            physical_host: event.physical_host(),
            object_start: event.object_start().value(),
            block_rtime: event.block_rtime().value(),
            interpolation_length: event.interpolation_length().value(),
            object_duration: event.object_duration().value(),
            block_duration: event.block_duration().value(),
            control_event: *event.control_event(),
        }
    }

    pub fn track_uid(&self) -> &str {
        &self.track_uid
    }

    pub fn object_id(&self) -> &str {
        &self.object_id
    }

    pub const fn physical_host(&self) -> u8 {
        self.physical_host
    }

    pub const fn control_event(&self) -> &SelectedObjectEvent {
        &self.control_event
    }
}

/// Immutable prepared inputs and their complete physical/DBMD/owner association.
#[derive(Clone, Debug, PartialEq)]
pub struct NormalControlPreparedSnapshot {
    programme_id: ProgrammeId,
    attempt_id: AttemptId,
    profile: NormalControlProfileIdentity,
    source_identity: NormalControlSourceIdentity,
    graph_identity: NormalControlGraphIdentity,
    frontend_state_digest: Option<FrontendControlDigest>,
    dbmd_payload_sha256: [u8; 32],
    physical_tracks: [NormalControlPhysicalTrack; PHYSICAL_TRACK_COUNT],
    owner_bindings: [NormalControlObjectOwnerBinding; OBJECT_TRACK_COUNT],
    object_events: [NormalControlObjectEvent; OBJECT_TRACK_COUNT],
    bed_pack_channel_format_ids: [String; BED_TRACK_COUNT],
    bed_pack_hosts: [u8; BED_TRACK_COUNT],
    regular_source_bindings: [NormalControlRegularSourceBinding; REGULAR_TRACK_COUNT],
    lfe_host: u8,
    regular37_hosts: [u8; REGULAR_TRACK_COUNT],
}

impl NormalControlPreparedSnapshot {
    /// Binds already parsed, admitted inputs into the immutable F01E normal-call view.
    pub fn from_validated(
        frontend_state: &FrontendControlState,
        source_binding: &SelectedPrimaryControlBinding,
        graph: &SelectedAdmGraph,
        admitted_events: &AdmittedSelectedObjectEvents,
        dbmd: &SelectedDbmdSections,
    ) -> Result<Arc<Self>, NormalControlSnapshotError> {
        validate_frontend_owner_state(frontend_state)?;
        let profile = NormalControlProfileIdentity::from_config(frontend_state.normalized_config());
        let source_identity = NormalControlSourceIdentity::from_binding(source_binding);
        validate_selected_profile(&profile, &source_identity)?;

        let metadata_hashes = source_binding.metadata_hashes();
        if graph.receipt().axml_sha256() != &metadata_hashes.axml()
            || graph.receipt().chna_sha256() != &metadata_hashes.chna()
            || dbmd.payload_sha256() != &metadata_hashes.dbmd()
        {
            return Err(NormalControlSnapshotError::MetadataIdentityMismatch);
        }
        if dbmd.track_fields().len() != PHYSICAL_TRACK_COUNT {
            return Err(NormalControlSnapshotError::DbmdIdentityMismatch);
        }
        let source_samples = source_binding.source_samples();
        let reparsed_events = graph
            .admit_object_events(source_samples)
            .map_err(|_| NormalControlSnapshotError::ObjectEventMismatch)?;
        if &reparsed_events != admitted_events {
            return Err(NormalControlSnapshotError::ObjectEventMismatch);
        }

        let mut tracks_by_host = graph.tracks().to_vec();
        tracks_by_host.sort_by_key(|track| track.physical_host().value());
        let tracks_by_host: [SelectedPhysicalTrack; PHYSICAL_TRACK_COUNT] = tracks_by_host
            .try_into()
            .map_err(|_| NormalControlSnapshotError::GraphDomain)?;

        let mut physical_tracks = Vec::with_capacity(PHYSICAL_TRACK_COUNT);
        for track in &tracks_by_host {
            let host = track.physical_host().value();
            let dbmd_fields = dbmd
                .fields_for_physical_host(u16::from(host))
                .ok_or(NormalControlSnapshotError::DbmdIdentityMismatch)?;
            physical_tracks.push(normal_control_track(track, dbmd_fields, graph)?);
        }
        let physical_tracks: [NormalControlPhysicalTrack; PHYSICAL_TRACK_COUNT] = physical_tracks
            .try_into()
            .map_err(|_| NormalControlSnapshotError::GraphDomain)?;

        let owner_bindings = graph
            .object_sampler_owner_bindings()
            .iter()
            .map(|binding| NormalControlObjectOwnerBinding {
                track_uid: binding.track_uid().to_owned(),
                local_owner_ordinal: binding.local_owner_ordinal().value(),
                semantic_key: binding.semantic_key(),
                physical_host: binding.physical_host().value(),
                channel_format_id: binding.channel_format_id().to_owned(),
            })
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| NormalControlSnapshotError::ObjectOwnerBindingMismatch)?;

        let mut object_events = admitted_events
            .events()
            .iter()
            .map(NormalControlObjectEvent::from_admitted)
            .collect::<Vec<_>>();
        object_events
            .sort_by(|left, right| left.track_uid.as_bytes().cmp(right.track_uid.as_bytes()));
        let object_events = object_events
            .try_into()
            .map_err(|_| NormalControlSnapshotError::ObjectEventMismatch)?;

        let bed_pack_channel_format_ids: [String; BED_TRACK_COUNT] =
            graph.bed_pack_channel_format_ids().to_owned();
        let bed_pack_hosts = *graph.bed_channel_join().slots_by_pack();
        let regular_source_bindings: [NormalControlRegularSourceBinding; REGULAR_TRACK_COUNT] =
            graph
                .regular_source_bindings()
                .iter()
                .map(|binding| NormalControlRegularSourceBinding {
                    physical_host: binding.physical_host().value(),
                    global_source: binding.global_source().value(),
                })
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| NormalControlSnapshotError::BedJoinMismatch)?;
        let regular37_hosts = *graph.bed_channel_join().regular_control_order();
        let prepared = Self {
            programme_id: frontend_state.programme_id().clone(),
            attempt_id: frontend_state.attempt_id(),
            profile,
            source_identity,
            graph_identity: NormalControlGraphIdentity::from_receipt(graph.receipt()),
            frontend_state_digest: Some(frontend_state.digest()),
            dbmd_payload_sha256: *dbmd.payload_sha256(),
            physical_tracks,
            owner_bindings,
            object_events,
            bed_pack_channel_format_ids,
            bed_pack_hosts,
            regular_source_bindings,
            lfe_host: graph.bed_channel_join().lfe_host(),
            regular37_hosts,
        };
        validate_prepared_snapshot(&prepared)?;
        Ok(Arc::new(prepared))
    }

    pub fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub const fn attempt_id(&self) -> AttemptId {
        self.attempt_id
    }

    pub const fn source_samples(&self) -> u64 {
        self.profile.source_samples
    }

    /// Checks that this prepared F01E view was derived from the caller's exact
    /// admitted primary binding, ADM graph and DBMD payload.
    pub fn matches_selected_input(
        &self,
        binding: &SelectedPrimaryControlBinding,
        graph: &SelectedAdmGraph,
        dbmd: &SelectedDbmdSections,
    ) -> bool {
        self.source_identity == NormalControlSourceIdentity::from_binding(binding)
            && self.graph_identity == NormalControlGraphIdentity::from_receipt(graph.receipt())
            && self.dbmd_payload_sha256 == *dbmd.payload_sha256()
    }

    /// Compares immutable admitted inputs while allowing each callback preparation to bind its
    /// own actual frontend before-state digest.
    pub(crate) fn same_prepared_input_identity(&self, other: &Self) -> bool {
        self.programme_id == other.programme_id
            && self.attempt_id == other.attempt_id
            && self.profile == other.profile
            && self.source_identity == other.source_identity
            && self.graph_identity == other.graph_identity
            && self.dbmd_payload_sha256 == other.dbmd_payload_sha256
            && self.physical_tracks == other.physical_tracks
            && self.owner_bindings == other.owner_bindings
            && self.object_events == other.object_events
            && self.bed_pack_channel_format_ids == other.bed_pack_channel_format_ids
            && self.bed_pack_hosts == other.bed_pack_hosts
            && self.regular_source_bindings == other.regular_source_bindings
            && self.lfe_host == other.lfe_host
            && self.regular37_hosts == other.regular37_hosts
    }

    pub fn physical_tracks(&self) -> &[NormalControlPhysicalTrack; PHYSICAL_TRACK_COUNT] {
        &self.physical_tracks
    }

    pub fn object_owner_bindings(&self) -> &[NormalControlObjectOwnerBinding; OBJECT_TRACK_COUNT] {
        &self.owner_bindings
    }

    pub const fn regular_source_bindings(
        &self,
    ) -> &[NormalControlRegularSourceBinding; REGULAR_TRACK_COUNT] {
        &self.regular_source_bindings
    }

    pub const fn lfe_host(&self) -> u8 {
        self.lfe_host
    }

    pub const fn regular37_hosts(&self) -> &[u8; REGULAR_TRACK_COUNT] {
        &self.regular37_hosts
    }

    /// Returns every validated prepared-input identity used by the local after-state serializer.
    /// This is a value projection; it does not reparse the input or alter its admission.
    pub(crate) fn canonical_json_value(&self) -> Result<Value, NormalControlSnapshotError> {
        let source = &self.source_identity;
        let profile = &self.profile;
        let source_value = json!({
            "input_kind": source.input_kind.selected_label(),
            "source_file_name": source.source_file_name,
            "source_file_bytes": source.source_file_bytes,
            "data_bytes": source.data_bytes,
            "wave": {
                "sample_rate_hz": source.wave.sample_rate_hz(),
                "channels": source.wave.channels(),
                "pcm_bits": source.wave.pcm_bits(),
            },
            "source_samples": source.source_samples,
            "metadata_hashes": {
                "axml_sha256": hex_sha256(&source.metadata_hashes.axml()),
                "chna_sha256": hex_sha256(&source.metadata_hashes.chna()),
                "dbmd_sha256": hex_sha256(&source.metadata_hashes.dbmd()),
            },
            "job_projection_sha256": hex_sha256(&source.job_projection_sha256),
            "source_receipt_sha256": hex_sha256(&source.source_receipt_sha256),
            "original_job_sha256_declaration": hex_sha256(&source.original_job_sha256_declaration),
            "original_job_bytes_declaration": source.original_job_bytes_declaration,
            "original_job_bytes_available": source.original_job_bytes_available,
            "full_pcm_identity_rehashed": source.full_pcm_identity_rehashed,
            "runtime_resource_bound": source.runtime_resource_bound,
        });
        let graph_value = json!({
            "axml_sha256": hex_sha256(&self.graph_identity.axml_sha256),
            "chna_sha256": hex_sha256(&self.graph_identity.chna_sha256),
            "physical_track_count": self.graph_identity.physical_track_count,
            "bed_track_count": self.graph_identity.bed_track_count,
            "object_track_count": self.graph_identity.object_track_count,
            "direct_lfe_host": self.graph_identity.direct_lfe_host,
            "virtual_prefix_source_count": self.graph_identity.virtual_prefix_source_count,
            "total_source_count": self.graph_identity.total_source_count,
        });
        let profile_value = json!({
            "sample_rate_hz": profile.sample_rate_hz,
            "channels": profile.channels,
            "pcm_bits": profile.pcm_bits,
            "source_samples": profile.source_samples,
            "bit_rate_kbps": profile.bit_rate_kbps,
            "bit_rate_enum": profile.bit_rate_enum,
            "layout_enum": profile.layout_enum,
            "target_capacity": profile.target_capacity,
            "frame_samples": profile.frame_samples,
            "spatial_config_mode": profile.spatial_config_mode,
            "dispatch_mode": profile.dispatch_mode,
            "virtual_prefix_count": profile.virtual_prefix_count,
            "bed_count": profile.bed_count,
            "object_count": profile.object_count,
            "event_start_samples": profile.event_start_samples,
            "line_drc": profile.line_drc,
            "rf_drc": profile.rf_drc,
            "time_base": profile.time_base,
        });
        let physical_tracks = self
            .physical_tracks
            .iter()
            .map(|track| {
                let kind = match &track.kind {
                    NormalControlTrackKind::Bed {
                        bed_id,
                        bed_pack_ordinal,
                        semantic_role,
                        bed_input_id,
                    } => json!({
                        "kind": "bed",
                        "bed_id": bed_id,
                        "bed_pack_ordinal": bed_pack_ordinal,
                        "semantic_role": semantic_role,
                        "bed_input_id": bed_input_id,
                    }),
                    NormalControlTrackKind::Object {
                        semantic_key,
                        local_owner_ordinal,
                    } => json!({
                        "kind": "object",
                        "semantic_key": semantic_key,
                        "local_owner_ordinal": local_owner_ordinal,
                    }),
                };
                json!({
                    "track_uid": track.track_uid,
                    "object_id": track.object_id,
                    "physical_host": track.physical_host,
                    "channel_format_id": track.channel_format_id,
                    "kind": kind,
                    "dbmd": dbmd_fields_json(track.dbmd),
                })
            })
            .collect::<Vec<_>>();
        let object_owner_bindings = self
            .owner_bindings
            .iter()
            .map(|binding| {
                json!({
                    "track_uid": binding.track_uid,
                    "local_owner_ordinal": binding.local_owner_ordinal,
                    "semantic_key": binding.semantic_key,
                    "physical_host": binding.physical_host,
                    "channel_format_id": binding.channel_format_id,
                })
            })
            .collect::<Vec<_>>();
        let object_events = self
            .object_events
            .iter()
            .map(|event| {
                let coordinates = event
                    .control_event
                    .authored_coordinates()
                    .map(optional_f64_json)
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;
                let spread = event
                    .control_event
                    .authored_spread()
                    .map(optional_f64_json)
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(json!({
                    "track_uid": event.track_uid,
                    "object_id": event.object_id,
                    "physical_host": event.physical_host,
                    "object_start": event.object_start,
                    "block_rtime": event.block_rtime,
                    "interpolation_length": event.interpolation_length,
                    "object_duration": event.object_duration,
                    "block_duration": event.block_duration,
                    "control_event": {
                        "host_channel_index": event.control_event.host_channel_index(),
                        "start_sample": event.control_event.start_sample(),
                        "duration_samples": event.control_event.duration_samples(),
                        "programme_samples": event.control_event.programme_samples(),
                        "coordinates": coordinates,
                        "spread": spread,
                        "diffuse": event.control_event.diffuse(),
                    }
                }))
            })
            .collect::<Result<Vec<_>, NormalControlSnapshotError>>()?;
        let regular_source_bindings = self
            .regular_source_bindings
            .iter()
            .map(|binding| {
                json!({
                    "physical_host": binding.physical_host,
                    "global_source": binding.global_source,
                })
            })
            .collect::<Vec<_>>();
        Ok(json!({
            "schema": "frontend_control_prepared_identity_v1",
            "programme_id": self.programme_id.as_str(),
            "attempt_id": self.attempt_id.value(),
            "profile": profile_value,
            "source_identity": source_value,
            "graph_identity": graph_value,
            "frontend_state_digest": self.frontend_state_digest.map(FrontendControlDigest::to_hex),
            "dbmd_payload_sha256": hex_sha256(&self.dbmd_payload_sha256),
            "physical_tracks": physical_tracks,
            "object_owner_bindings": object_owner_bindings,
            "object_events": object_events,
            "bed_pack_channel_format_ids": self.bed_pack_channel_format_ids,
            "bed_pack_hosts": self.bed_pack_hosts,
            "regular_source_bindings": regular_source_bindings,
            "lfe_host": self.lfe_host,
            "regular37_hosts": self.regular37_hosts.to_vec(),
        }))
    }

    pub const fn profile_identity(&self) -> crate::profile::ResolvedEncodingProfile {
        self.source_identity.selected_profile
    }

    pub fn matches_graph(&self, graph: &SelectedAdmGraph) -> bool {
        if self.graph_identity != NormalControlGraphIdentity::from_receipt(graph.receipt())
            || self.lfe_host != graph.bed_channel_join().lfe_host()
            || self.bed_pack_channel_format_ids != *graph.bed_pack_channel_format_ids()
            || self.bed_pack_hosts != *graph.bed_channel_join().slots_by_pack()
            || self.regular37_hosts != *graph.bed_channel_join().regular_control_order()
            || self.source_identity.metadata_hashes.axml() != *graph.receipt().axml_sha256()
            || self.source_identity.metadata_hashes.chna() != *graph.receipt().chna_sha256()
        {
            return false;
        }
        for track in graph.tracks() {
            let host = usize::from(track.physical_host().value());
            let Some(prepared_track) = self.physical_tracks.get(host) else {
                return false;
            };
            if prepared_track.track_uid != track.uid()
                || prepared_track.object_id != track.object_id()
                || prepared_track.channel_format_id != track.channel_format()
            {
                return false;
            }
            match (track.kind(), &prepared_track.kind) {
                (
                    SelectedTrackKind::DirectSpeakersBed {
                        bed_id,
                        bed_pack_ordinal,
                        bed_input_id,
                        role,
                    },
                    NormalControlTrackKind::Bed {
                        bed_id: prepared_bed_id,
                        bed_pack_ordinal: prepared_pack_ordinal,
                        bed_input_id: prepared_bed_input_id,
                        semantic_role,
                    },
                ) if bed_id == *prepared_bed_id
                    && bed_pack_ordinal == *prepared_pack_ordinal
                    && bed_input_id == *prepared_bed_input_id
                    && role == semantic_role => {}
                (
                    SelectedTrackKind::Object { semantic_key },
                    NormalControlTrackKind::Object {
                        semantic_key: prepared_key,
                        local_owner_ordinal: prepared_ordinal,
                    },
                ) => {
                    let Some(binding) = graph.object_sampler_owner_binding_for_uid(track.uid())
                    else {
                        return false;
                    };
                    if semantic_key != *prepared_key
                        || binding.local_owner_ordinal().value() != *prepared_ordinal
                    {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        for (prepared, source) in self
            .regular_source_bindings
            .iter()
            .zip(graph.regular_source_bindings())
        {
            if prepared.physical_host != source.physical_host().value()
                || prepared.global_source != source.global_source().value()
            {
                return false;
            }
        }
        for prepared in &self.owner_bindings {
            let Some(source) = graph.object_sampler_owner_binding_for_uid(&prepared.track_uid)
            else {
                return false;
            };
            if prepared.local_owner_ordinal != source.local_owner_ordinal().value()
                || prepared.semantic_key != source.semantic_key()
                || prepared.physical_host != source.physical_host().value()
                || prepared.channel_format_id != source.channel_format_id()
            {
                return false;
            }
        }
        let Ok(samples) = AdmSampleCount::new(self.profile.source_samples) else {
            return false;
        };
        let Ok(admitted_events) = graph.admit_object_events(samples) else {
            return false;
        };
        let mut object_events = admitted_events
            .events()
            .iter()
            .map(NormalControlObjectEvent::from_admitted)
            .collect::<Vec<_>>();
        object_events
            .sort_by(|left, right| left.track_uid.as_bytes().cmp(right.track_uid.as_bytes()));
        self.object_events.as_slice() == object_events.as_slice()
    }

    pub(crate) fn binds_frontend_state(&self, state: &FrontendControlState) -> bool {
        let config = state.normalized_config();
        self.programme_id == *state.programme_id()
            && self.attempt_id == state.attempt_id()
            && self.frontend_state_digest == Some(state.digest())
            && self.profile.sample_rate_hz == config.sample_rate_hz()
            && self.profile.channels == config.channels()
            && self.profile.pcm_bits == config.pcm_bits()
            && self.profile.source_samples == config.source_samples()
            && self.profile.bit_rate_kbps == config.bit_rate_kbps()
            && self.profile.bit_rate_enum == config.bit_rate_enum()
            && self.profile.layout_enum == config.layout_enum()
            && self.profile.target_capacity == config.target_capacity()
            && self.profile.frame_samples == config.frame_samples()
            && self.profile.spatial_config_mode == config.spatial_config_mode()
            && self.profile.dispatch_mode == config.dispatch_mode()
            && self.profile.virtual_prefix_count == config.virtual_prefix_count()
            && self.profile.bed_count == config.bed_count()
            && self.profile.object_count == config.object_count()
            && self.profile.event_start_samples == config.event_start_samples()
            && self.profile.line_drc == config.line_drc()
            && self.profile.rf_drc == config.rf_drc()
            && self.profile.time_base == config.time_base()
            && self.source_identity.source_samples == config.source_samples()
    }

    pub(crate) fn binds_normal_objects(
        &self,
        objects: &[NormalControlObjectState; OBJECT_TRACK_COUNT],
    ) -> bool {
        let mut seen = [false; OBJECT_TRACK_COUNT];
        for object in objects {
            let Ok(ordinal) = usize::try_from(object.object_ordinal()) else {
                return false;
            };
            let Some(slot) = seen.get_mut(ordinal) else {
                return false;
            };
            if *slot {
                return false;
            }
            *slot = true;
            let Some(binding) = self
                .owner_bindings
                .iter()
                .find(|binding| usize::from(binding.local_owner_ordinal) == ordinal)
            else {
                return false;
            };
            if object.host_channel_index() != Some(u32::from(binding.physical_host)) {
                return false;
            }
            let Some(event) = self
                .object_events
                .iter()
                .find(|event| event.track_uid == binding.track_uid)
            else {
                return false;
            };
            if event.physical_host != binding.physical_host {
                return false;
            }
        }
        seen.into_iter().all(|value| value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalControlObjectState {
    object_ordinal: u32,
    event_cache: Option<Arc<[u8]>>,
    host_channel_index: Option<u32>,
    sampler_previous_present: bool,
    sampler_previous_xyz_u32: [u32; 3],
}

impl NormalControlObjectState {
    pub const fn object_ordinal(&self) -> u32 {
        self.object_ordinal
    }

    pub const fn host_channel_index(&self) -> Option<u32> {
        self.host_channel_index
    }

    pub const fn sampler_previous_present(&self) -> bool {
        self.sampler_previous_present
    }

    pub const fn sampler_previous_xyz_u32(&self) -> [u32; 3] {
        self.sampler_previous_xyz_u32
    }

    pub const fn event_cache(&self) -> Option<&Arc<[u8]>> {
        self.event_cache.as_ref()
    }

    pub(crate) fn into_frontend_parts(self) -> NormalControlObjectStateParts {
        NormalControlObjectStateParts {
            object_ordinal: self.object_ordinal,
            event_cache: self.event_cache,
            host_channel_index: self.host_channel_index,
            sampler_previous_present: self.sampler_previous_present,
            sampler_previous_xyz_u32: self.sampler_previous_xyz_u32,
        }
    }

    fn from_frontend_state(state: &FrontendObjectState) -> Self {
        Self {
            object_ordinal: state.object_ordinal(),
            event_cache: state.event_cache().cloned(),
            host_channel_index: state.host_channel_index(),
            sampler_previous_present: state.sampler_previous_present(),
            sampler_previous_xyz_u32: state.sampler_previous_xyz_u32(),
        }
    }

    fn after_normal_callback(&self, host_channel_index: u8, previous_xyz: [u32; 3]) -> Self {
        Self {
            object_ordinal: self.object_ordinal,
            event_cache: self.event_cache.clone(),
            host_channel_index: Some(u32::from(host_channel_index)),
            sampler_previous_present: true,
            sampler_previous_xyz_u32: previous_xyz,
        }
    }
}

pub(crate) struct NormalControlObjectStateParts {
    pub(crate) object_ordinal: u32,
    pub(crate) event_cache: Option<Arc<[u8]>>,
    pub(crate) host_channel_index: Option<u32>,
    pub(crate) sampler_previous_present: bool,
    pub(crate) sampler_previous_xyz_u32: [u32; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct NormalControlBeforeState {
    programme_id: ProgrammeId,
    attempt_id: AttemptId,
    callback_index: FrontendCallbackIndex,
    normal_callbacks: NormalCallbackCount,
    owner_generation: u64,
    remaining_cached_drains: u32,
    phase: FrontendControlPhase,
    profile: NormalControlProfileIdentity,
    state_digest: Option<FrontendControlDigest>,
    objects: [NormalControlObjectState; OBJECT_TRACK_COUNT],
    last_control38_snapshot: Option<Arc<[u32]>>,
    last_regular37_snapshot: Option<Arc<[u32]>>,
    physical_binding: Option<Arc<[u8]>>,
    dbmd_binding: Option<Arc<[u8]>>,
    previous_prepared_identity: Option<Arc<NormalControlPreparedSnapshot>>,
}

impl NormalControlBeforeState {
    /// Copies the current F01E owner view; it never changes the owner.
    pub fn from_frontend_state(
        state: &FrontendControlState,
        owner_generation: u64,
    ) -> Result<Self, NormalControlSnapshotError> {
        let objects: [NormalControlObjectState; OBJECT_TRACK_COUNT] = state
            .objects()
            .iter()
            .map(NormalControlObjectState::from_frontend_state)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| NormalControlSnapshotError::StateShapeMismatch)?;
        let previous_prepared_identity = if state.callback_index().value() == 0 {
            None
        } else {
            Some(Arc::new(
                state
                    .prepared_identity()
                    .ok_or(NormalControlSnapshotError::PreparedIdentityMismatch)?
                    .clone(),
            ))
        };
        Ok(Self {
            programme_id: state.programme_id().clone(),
            attempt_id: state.attempt_id(),
            callback_index: state.callback_index(),
            normal_callbacks: state.normal_callbacks(),
            owner_generation,
            remaining_cached_drains: state.remaining_cached_drains(),
            phase: state.phase(),
            profile: NormalControlProfileIdentity::from_config(state.normalized_config()),
            state_digest: Some(state.digest()),
            objects,
            last_control38_snapshot: state.last_control38_snapshot().cloned(),
            last_regular37_snapshot: state.last_regular37_snapshot().cloned(),
            physical_binding: state.physical_binding().cloned(),
            dbmd_binding: state.dbmd_binding().cloned(),
            previous_prepared_identity,
        })
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn normal_callbacks(&self) -> NormalCallbackCount {
        self.normal_callbacks
    }

    pub const fn owner_generation(&self) -> u64 {
        self.owner_generation
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NormalControlSamplerBefore {
    local_owner_ordinal: u8,
    physical_host: u8,
    sampler_previous_present: bool,
    sampler_previous_xyz_u32: [u32; 3],
}

impl NormalControlSamplerBefore {
    pub const fn local_owner_ordinal(self) -> u8 {
        self.local_owner_ordinal
    }

    pub const fn physical_host(self) -> u8 {
        self.physical_host
    }

    pub const fn sampler_previous_present(self) -> bool {
        self.sampler_previous_present
    }

    pub const fn sampler_previous_xyz_u32(self) -> [u32; 3] {
        self.sampler_previous_xyz_u32
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnproducedBedTail {
    offset_bytes: u8,
    byte_count: u8,
}

impl UnproducedBedTail {
    pub const fn offset_bytes(self) -> u8 {
        self.offset_bytes
    }

    pub const fn byte_count(self) -> u8 {
        self.byte_count
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NormalControlSourceFlagsView {
    mute_word: u32,
    spread_word: u32,
}

impl NormalControlSourceFlagsView {
    pub const fn mute_word(self) -> u32 {
        self.mute_word
    }

    pub const fn spread_word(self) -> u32 {
        self.spread_word
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NormalControlRow {
    Bed6 {
        physical_host: u8,
        track_uid: String,
        channel_format_id: String,
        dbmd: NormalControlDbmdFields,
        words: [u32; 6],
        unproduced_tail: UnproducedBedTail,
    },
    Object13 {
        physical_host: u8,
        track_uid: String,
        channel_format_id: String,
        semantic_key: u32,
        local_owner_ordinal: u8,
        dbmd: NormalControlDbmdFields,
        control: ObjectControl52,
    },
}

impl NormalControlRow {
    pub const fn physical_host(&self) -> u8 {
        match self {
            Self::Bed6 { physical_host, .. } | Self::Object13 { physical_host, .. } => {
                *physical_host
            }
        }
    }

    pub fn track_uid(&self) -> &str {
        match self {
            Self::Bed6 { track_uid, .. } | Self::Object13 { track_uid, .. } => track_uid,
        }
    }

    pub fn channel_format_id(&self) -> &str {
        match self {
            Self::Bed6 {
                channel_format_id, ..
            }
            | Self::Object13 {
                channel_format_id, ..
            } => channel_format_id,
        }
    }

    pub const fn dbmd(&self) -> NormalControlDbmdFields {
        match self {
            Self::Bed6 { dbmd, .. } | Self::Object13 { dbmd, .. } => *dbmd,
        }
    }

    /// Applies the approved Bed6/Object13 SourceFlags view without reading bed padding.
    pub const fn source_flags_view(&self) -> NormalControlSourceFlagsView {
        match self {
            Self::Bed6 { .. } => NormalControlSourceFlagsView {
                mute_word: 0,
                spread_word: 0.0_f32.to_bits(),
            },
            Self::Object13 { control, .. } => {
                let words = control.words();
                NormalControlSourceFlagsView {
                    mute_word: words[0],
                    spread_word: words[9],
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NormalControlAfterState {
    programme_id: ProgrammeId,
    attempt_id: AttemptId,
    next_callback_index: FrontendCallbackIndex,
    next_normal_callbacks: NormalCallbackCount,
    next_owner_generation: u64,
    remaining_cached_drains: u32,
    phase: FrontendControlPhase,
    objects: [NormalControlObjectState; OBJECT_TRACK_COUNT],
    control38_snapshot: Arc<[u32]>,
    regular37_snapshot: Arc<[u32]>,
    physical_binding_before: Option<Arc<[u8]>>,
    dbmd_binding_before: Option<Arc<[u8]>>,
    prepared_identity: Arc<NormalControlPreparedSnapshot>,
}

impl NormalControlAfterState {
    pub fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub const fn attempt_id(&self) -> AttemptId {
        self.attempt_id
    }

    pub const fn next_callback_index(&self) -> FrontendCallbackIndex {
        self.next_callback_index
    }

    pub const fn next_normal_callbacks(&self) -> NormalCallbackCount {
        self.next_normal_callbacks
    }

    pub const fn next_owner_generation(&self) -> u64 {
        self.next_owner_generation
    }

    pub const fn remaining_cached_drains(&self) -> u32 {
        self.remaining_cached_drains
    }

    pub const fn phase(&self) -> FrontendControlPhase {
        self.phase
    }

    pub fn objects(&self) -> &[NormalControlObjectState; OBJECT_TRACK_COUNT] {
        &self.objects
    }

    pub fn control38_snapshot(&self) -> &Arc<[u32]> {
        &self.control38_snapshot
    }

    pub fn regular37_snapshot(&self) -> &Arc<[u32]> {
        &self.regular37_snapshot
    }

    pub const fn physical_binding_before(&self) -> Option<&Arc<[u8]>> {
        self.physical_binding_before.as_ref()
    }

    pub const fn dbmd_binding_before(&self) -> Option<&Arc<[u8]>> {
        self.dbmd_binding_before.as_ref()
    }

    pub fn prepared_identity(&self) -> &Arc<NormalControlPreparedSnapshot> {
        &self.prepared_identity
    }

    pub(crate) fn into_frontend_owner_parts(self) -> NormalControlAfterStateParts {
        NormalControlAfterStateParts {
            programme_id: self.programme_id,
            attempt_id: self.attempt_id,
            next_callback_index: self.next_callback_index,
            next_normal_callbacks: self.next_normal_callbacks,
            next_owner_generation: self.next_owner_generation,
            remaining_cached_drains: self.remaining_cached_drains,
            phase: self.phase,
            objects: self.objects,
            control38_snapshot: self.control38_snapshot,
            regular37_snapshot: self.regular37_snapshot,
            physical_binding_before: self.physical_binding_before,
            dbmd_binding_before: self.dbmd_binding_before,
            prepared_identity: self.prepared_identity,
        }
    }

    /// Reuses this committed last-normal source snapshot for one main cached-audio delivery.
    pub fn held_source_view_for_cached_callback(
        &self,
        delivery_transaction_id: CallbackLineageIdentity,
        normal_callbacks: NormalCallbackCount,
    ) -> Result<NormalControlHeldSourceView, NormalControlSnapshotError> {
        let normal_count = normal_callbacks.value();
        let expected_source_callbacks = self
            .prepared_identity
            .source_samples()
            .div_ceil(NORMAL_CALLBACK_SAMPLES);
        let expected_delivery_index =
            u32::try_from(normal_count).map_err(|_| NormalControlSnapshotError::IndexOverflow)?;
        let delivery_callback = delivery_transaction_id.callback_index();
        let delivery_end = normal_count
            .checked_add(u64::from(MAIN_CACHED_DRAIN_CALLBACKS))
            .ok_or(NormalControlSnapshotError::IndexOverflow)?;
        if normal_count == 0
            || expected_source_callbacks != normal_count
            || self.next_normal_callbacks != normal_callbacks
            || self.next_callback_index.value() != expected_delivery_index
            || self.next_owner_generation != normal_count
            || self.remaining_cached_drains != MAIN_CACHED_DRAIN_CALLBACKS
            || delivery_transaction_id.programme_id() != &self.programme_id
            || delivery_transaction_id.attempt_id() != self.attempt_id
            || u64::from(delivery_callback.value()) < normal_count
            || u64::from(delivery_callback.value()) >= delivery_end
        {
            return Err(NormalControlSnapshotError::CallbackClockMismatch);
        }

        let control38_snapshot = self.control38_snapshot.clone();
        let regular37_snapshot = self.regular37_snapshot.clone();
        let prepared_identity = Arc::clone(&self.prepared_identity);
        let source_origin_generation = FrontendCallbackIndex::new(
            u32::try_from(normal_count - 1)
                .map_err(|_| NormalControlSnapshotError::IndexOverflow)?,
        );
        let source_origin_transaction_id = CallbackLineageIdentity::new(
            self.programme_id.clone(),
            self.attempt_id,
            source_origin_generation,
        );

        if control38_snapshot.len() != CONTROL38_WORD_COUNT
            || regular37_snapshot.len() != REGULAR37_WORD_COUNT
            || prepared_identity.physical_tracks().len() != PHYSICAL_TRACK_COUNT
            || prepared_identity.regular37_hosts().len() != REGULAR_TRACK_COUNT
        {
            return Err(NormalControlSnapshotError::SnapshotShapeMismatch);
        }

        let control38_rows = held_control38_rows(&control38_snapshot, &prepared_identity)?;
        validate_held_regular_snapshot(&regular37_snapshot, &prepared_identity, &control38_rows)?;
        let regular37_rows = prepared_identity
            .regular37_hosts()
            .iter()
            .map(|host| {
                control38_rows
                    .get(usize::from(*host))
                    .cloned()
                    .ok_or(NormalControlSnapshotError::PhysicalHostDomain)
            })
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| NormalControlSnapshotError::SnapshotShapeMismatch)?;

        Ok(NormalControlHeldSourceView {
            delivery_transaction_id,
            source_origin_transaction_id,
            normal_callbacks,
            source_origin_generation,
            prepared_identity,
            control38_rows,
            regular37_rows,
        })
    }

    /// Reuses this completed Meter-owned normal snapshot for one fresh-zero Meter drain.
    ///
    /// Unlike the main cached-audio view, this consumes no cached-drain budget and is limited to
    /// the Meter phase's three delivery callbacks after the Q normal source callbacks.
    pub fn held_source_view_for_meter_fresh_zero_drain(
        &self,
        delivery_transaction_id: CallbackLineageIdentity,
        normal_callbacks: NormalCallbackCount,
    ) -> Result<NormalControlFreshZeroMeterView, NormalControlSnapshotError> {
        let normal_count = normal_callbacks.value();
        let expected_source_callbacks = self
            .prepared_identity
            .source_samples()
            .div_ceil(NORMAL_CALLBACK_SAMPLES);
        let expected_delivery_index =
            u32::try_from(normal_count).map_err(|_| NormalControlSnapshotError::IndexOverflow)?;
        let delivery_callback = delivery_transaction_id.callback_index();
        let delivery_end = normal_count
            .checked_add(3)
            .ok_or(NormalControlSnapshotError::IndexOverflow)?;
        if normal_count == 0
            || expected_source_callbacks != normal_count
            || self.next_normal_callbacks != normal_callbacks
            || self.next_callback_index.value() != expected_delivery_index
            || self.next_owner_generation != normal_count
            || delivery_transaction_id.programme_id() != &self.programme_id
            || delivery_transaction_id.attempt_id() != self.attempt_id
            || u64::from(delivery_callback.value()) < normal_count
            || u64::from(delivery_callback.value()) >= delivery_end
        {
            return Err(NormalControlSnapshotError::CallbackClockMismatch);
        }

        let control38_snapshot = self.control38_snapshot.clone();
        let regular37_snapshot = self.regular37_snapshot.clone();
        let prepared_identity = Arc::clone(&self.prepared_identity);
        let source_origin_generation = FrontendCallbackIndex::new(
            u32::try_from(normal_count - 1)
                .map_err(|_| NormalControlSnapshotError::IndexOverflow)?,
        );
        let source_origin_transaction_id = CallbackLineageIdentity::new(
            self.programme_id.clone(),
            self.attempt_id,
            source_origin_generation,
        );

        if control38_snapshot.len() != CONTROL38_WORD_COUNT
            || regular37_snapshot.len() != REGULAR37_WORD_COUNT
            || prepared_identity.physical_tracks().len() != PHYSICAL_TRACK_COUNT
            || prepared_identity.regular37_hosts().len() != REGULAR_TRACK_COUNT
        {
            return Err(NormalControlSnapshotError::SnapshotShapeMismatch);
        }

        let control38_rows = held_control38_rows(&control38_snapshot, &prepared_identity)?;
        validate_held_regular_snapshot(&regular37_snapshot, &prepared_identity, &control38_rows)?;
        let regular37_rows = prepared_identity
            .regular37_hosts()
            .iter()
            .map(|host| {
                control38_rows
                    .get(usize::from(*host))
                    .cloned()
                    .ok_or(NormalControlSnapshotError::PhysicalHostDomain)
            })
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| NormalControlSnapshotError::SnapshotShapeMismatch)?;

        Ok(NormalControlFreshZeroMeterView {
            delivery_transaction_id,
            source_origin_transaction_id,
            normal_callbacks,
            source_origin_generation,
            prepared_identity,
            control38_rows,
            regular37_rows,
        })
    }

    /// Test-only clock fixture. It preserves source bytes and does not assert an F01E replay.
    #[cfg(test)]
    pub(crate) fn fixture_as_last_normal_clock(&self) -> Result<Self, NormalControlSnapshotError> {
        let normal_count = self
            .prepared_identity
            .source_samples()
            .div_ceil(NORMAL_CALLBACK_SAMPLES);
        let next_callback =
            u32::try_from(normal_count).map_err(|_| NormalControlSnapshotError::IndexOverflow)?;
        let mut fixture = self.clone();
        fixture.next_callback_index = FrontendCallbackIndex::new(next_callback);
        fixture.next_normal_callbacks = NormalCallbackCount::new(normal_count);
        fixture.next_owner_generation = normal_count;
        fixture.remaining_cached_drains = 4;
        Ok(fixture)
    }
}

pub(crate) struct NormalControlAfterStateParts {
    pub(crate) programme_id: ProgrammeId,
    pub(crate) attempt_id: AttemptId,
    pub(crate) next_callback_index: FrontendCallbackIndex,
    pub(crate) next_normal_callbacks: NormalCallbackCount,
    pub(crate) next_owner_generation: u64,
    pub(crate) remaining_cached_drains: u32,
    pub(crate) phase: FrontendControlPhase,
    pub(crate) objects: [NormalControlObjectState; OBJECT_TRACK_COUNT],
    pub(crate) control38_snapshot: Arc<[u32]>,
    pub(crate) regular37_snapshot: Arc<[u32]>,
    pub(crate) physical_binding_before: Option<Arc<[u8]>>,
    pub(crate) dbmd_binding_before: Option<Arc<[u8]>>,
    pub(crate) prepared_identity: Arc<NormalControlPreparedSnapshot>,
}

/// Immutable main cached-drain view of the last committed normal F01E source snapshot.
/// Delivery g advances while all source values retain origin Q-1.
#[derive(Debug, PartialEq)]
pub struct NormalControlHeldSourceView {
    delivery_transaction_id: CallbackLineageIdentity,
    source_origin_transaction_id: CallbackLineageIdentity,
    normal_callbacks: NormalCallbackCount,
    source_origin_generation: FrontendCallbackIndex,
    prepared_identity: Arc<NormalControlPreparedSnapshot>,
    control38_rows: [NormalControlRow; PHYSICAL_TRACK_COUNT],
    regular37_rows: [NormalControlRow; REGULAR_TRACK_COUNT],
}

/// Immutable Meter fresh-zero drain view of a completed normal F01E snapshot.
///
/// The delivery callback advances through the Meter drain window while all values keep their
/// source origin at Q-1. This type is intentionally distinct from the main cached-drain view.
#[derive(Debug, PartialEq)]
pub struct NormalControlFreshZeroMeterView {
    delivery_transaction_id: CallbackLineageIdentity,
    source_origin_transaction_id: CallbackLineageIdentity,
    normal_callbacks: NormalCallbackCount,
    source_origin_generation: FrontendCallbackIndex,
    prepared_identity: Arc<NormalControlPreparedSnapshot>,
    control38_rows: [NormalControlRow; PHYSICAL_TRACK_COUNT],
    regular37_rows: [NormalControlRow; REGULAR_TRACK_COUNT],
}

impl NormalControlFreshZeroMeterView {
    /// Rebinds these held normal rows to another delivery in the Meter fresh-zero window.
    ///
    /// This lets one last-normal source view carry across Q+1 and Q+2 without consulting or
    /// advancing the main cached-drain budget.
    pub fn for_delivery(
        &self,
        delivery_transaction_id: CallbackLineageIdentity,
    ) -> Result<Self, NormalControlSnapshotError> {
        let normal_count = self.normal_callbacks.value();
        let delivery_end = normal_count
            .checked_add(3)
            .ok_or(NormalControlSnapshotError::IndexOverflow)?;
        let expected_origin = normal_count
            .checked_sub(1)
            .and_then(|value| u32::try_from(value).ok())
            .map(FrontendCallbackIndex::new)
            .ok_or(NormalControlSnapshotError::CallbackClockMismatch)?;
        let prepared = self.prepared_identity.as_ref();
        if normal_count == 0
            || prepared.source_samples().div_ceil(NORMAL_CALLBACK_SAMPLES) != normal_count
            || self.delivery_transaction_id.programme_id() != prepared.programme_id()
            || self.delivery_transaction_id.attempt_id() != prepared.attempt_id()
            || u64::from(self.delivery_generation().value()) < normal_count
            || u64::from(self.delivery_generation().value()) >= delivery_end
            || self.source_origin_generation != expected_origin
            || self.source_origin_transaction_id.programme_id() != prepared.programme_id()
            || self.source_origin_transaction_id.attempt_id() != prepared.attempt_id()
            || self.source_origin_transaction_id.callback_index() != expected_origin
            || delivery_transaction_id.programme_id() != prepared.programme_id()
            || delivery_transaction_id.attempt_id() != prepared.attempt_id()
            || u64::from(delivery_transaction_id.callback_index().value()) < normal_count
            || u64::from(delivery_transaction_id.callback_index().value()) >= delivery_end
        {
            return Err(NormalControlSnapshotError::CallbackClockMismatch);
        }
        Ok(Self {
            delivery_transaction_id,
            source_origin_transaction_id: self.source_origin_transaction_id.clone(),
            normal_callbacks: self.normal_callbacks,
            source_origin_generation: self.source_origin_generation,
            prepared_identity: Arc::clone(&self.prepared_identity),
            control38_rows: self.control38_rows.clone(),
            regular37_rows: self.regular37_rows.clone(),
        })
    }

    pub fn delivery_transaction_id(&self) -> &CallbackLineageIdentity {
        &self.delivery_transaction_id
    }

    pub fn source_origin_transaction_id(&self) -> &CallbackLineageIdentity {
        &self.source_origin_transaction_id
    }

    pub const fn delivery_generation(&self) -> FrontendCallbackIndex {
        self.delivery_transaction_id.callback_index()
    }

    pub const fn source_origin_generation(&self) -> FrontendCallbackIndex {
        self.source_origin_generation
    }

    pub const fn normal_callbacks(&self) -> NormalCallbackCount {
        self.normal_callbacks
    }

    pub fn prepared_identity(&self) -> &NormalControlPreparedSnapshot {
        &self.prepared_identity
    }

    pub const fn control38_rows(&self) -> &[NormalControlRow; PHYSICAL_TRACK_COUNT] {
        &self.control38_rows
    }

    pub const fn regular37_rows(&self) -> &[NormalControlRow; REGULAR_TRACK_COUNT] {
        &self.regular37_rows
    }
}

impl NormalControlHeldSourceView {
    pub fn delivery_transaction_id(&self) -> &CallbackLineageIdentity {
        &self.delivery_transaction_id
    }

    pub fn source_origin_transaction_id(&self) -> &CallbackLineageIdentity {
        &self.source_origin_transaction_id
    }

    pub const fn delivery_generation(&self) -> FrontendCallbackIndex {
        self.delivery_transaction_id.callback_index()
    }

    pub const fn source_origin_generation(&self) -> FrontendCallbackIndex {
        self.source_origin_generation
    }

    pub const fn normal_callbacks(&self) -> NormalCallbackCount {
        self.normal_callbacks
    }

    pub fn prepared_identity(&self) -> &NormalControlPreparedSnapshot {
        &self.prepared_identity
    }

    pub const fn control38_rows(&self) -> &[NormalControlRow; PHYSICAL_TRACK_COUNT] {
        &self.control38_rows
    }

    pub const fn regular37_rows(&self) -> &[NormalControlRow; REGULAR_TRACK_COUNT] {
        &self.regular37_rows
    }
}

fn held_control38_rows(
    snapshot: &[u32],
    prepared: &NormalControlPreparedSnapshot,
) -> Result<[NormalControlRow; PHYSICAL_TRACK_COUNT], NormalControlSnapshotError> {
    let mut rows = Vec::with_capacity(PHYSICAL_TRACK_COUNT);
    for (host, track) in prepared.physical_tracks().iter().enumerate() {
        let host_u8 =
            u8::try_from(host).map_err(|_| NormalControlSnapshotError::PhysicalHostDomain)?;
        if track.physical_host() != host_u8 {
            return Err(NormalControlSnapshotError::PhysicalHostDomain);
        }
        let start = host
            .checked_mul(CONTROL_WORDS_PER_TRACK)
            .ok_or(NormalControlSnapshotError::SnapshotShapeMismatch)?;
        let row_words = snapshot
            .get(start..start + CONTROL_WORDS_PER_TRACK)
            .ok_or(NormalControlSnapshotError::SnapshotShapeMismatch)?;
        let row = match track.kind() {
            NormalControlTrackKind::Bed { bed_id, .. } => {
                let words: [u32; 6] = row_words[..6]
                    .try_into()
                    .map_err(|_| NormalControlSnapshotError::SnapshotShapeMismatch)?;
                if words != [0, 0, 3, 2, 1, u32::from(*bed_id)] {
                    return Err(NormalControlSnapshotError::SnapshotShapeMismatch);
                }
                NormalControlRow::Bed6 {
                    physical_host: host_u8,
                    track_uid: track.track_uid().to_owned(),
                    channel_format_id: track.channel_format_id().to_owned(),
                    dbmd: track.dbmd(),
                    words,
                    unproduced_tail: UnproducedBedTail {
                        offset_bytes: 24,
                        byte_count: 28,
                    },
                }
            }
            NormalControlTrackKind::Object {
                semantic_key,
                local_owner_ordinal,
            } => {
                let words: [u32; CONTROL_WORDS_PER_TRACK] = row_words
                    .try_into()
                    .map_err(|_| NormalControlSnapshotError::SnapshotShapeMismatch)?;
                NormalControlRow::Object13 {
                    physical_host: host_u8,
                    track_uid: track.track_uid().to_owned(),
                    channel_format_id: track.channel_format_id().to_owned(),
                    semantic_key: *semantic_key,
                    local_owner_ordinal: *local_owner_ordinal,
                    dbmd: track.dbmd(),
                    control: ObjectControl52::from_committed_snapshot_words(words),
                }
            }
        };
        rows.push(row);
    }
    rows.try_into()
        .map_err(|_| NormalControlSnapshotError::SnapshotShapeMismatch)
}

fn validate_held_regular_snapshot(
    snapshot: &[u32],
    prepared: &NormalControlPreparedSnapshot,
    control38_rows: &[NormalControlRow; PHYSICAL_TRACK_COUNT],
) -> Result<(), NormalControlSnapshotError> {
    for (ordinal, host) in prepared.regular37_hosts().iter().copied().enumerate() {
        let row = control38_rows
            .get(usize::from(host))
            .ok_or(NormalControlSnapshotError::PhysicalHostDomain)?;
        let start = ordinal
            .checked_mul(CONTROL_WORDS_PER_TRACK)
            .ok_or(NormalControlSnapshotError::SnapshotShapeMismatch)?;
        let carrier = snapshot
            .get(start..start + CONTROL_WORDS_PER_TRACK)
            .ok_or(NormalControlSnapshotError::SnapshotShapeMismatch)?;
        match row {
            NormalControlRow::Bed6 { words, .. } => {
                if carrier[..6] != *words {
                    return Err(NormalControlSnapshotError::SnapshotShapeMismatch);
                }
            }
            NormalControlRow::Object13 { control, .. } => {
                if carrier != control.words() {
                    return Err(NormalControlSnapshotError::SnapshotShapeMismatch);
                }
            }
        }
    }
    Ok(())
}

/// Complete immutable normal-callback output. It carries no commit operation.
#[derive(Debug)]
pub struct NormalControlSnapshotProposal {
    transaction_id: CallbackLineageIdentity,
    value_generation: FrontendCallbackIndex,
    normal_origin_generation: FrontendCallbackIndex,
    query_sample: u64,
    actual_samples: u32,
    invocation_hosts: [u8; OBJECT_TRACK_COUNT],
    sampler_before: [NormalControlSamplerBefore; OBJECT_TRACK_COUNT],
    control38_rows: [NormalControlRow; PHYSICAL_TRACK_COUNT],
    regular37_rows: [NormalControlRow; REGULAR_TRACK_COUNT],
    control38_carrier_words: Arc<[u32]>,
    regular37_carrier_words: Arc<[u32]>,
    after_state: NormalControlAfterState,
}

/// One-shot projection retained for F03W after ownership of the complete normal after-state is
/// transferred to the frontend-control owner stage.
#[derive(Debug)]
pub struct NormalControlSnapshotOutput {
    transaction_id: CallbackLineageIdentity,
    value_generation: FrontendCallbackIndex,
    normal_origin_generation: FrontendCallbackIndex,
    next_callback_index: FrontendCallbackIndex,
    next_owner_generation: u64,
    normal_callbacks: NormalCallbackCount,
    prepared_identity: Arc<NormalControlPreparedSnapshot>,
    query_sample: u64,
    actual_samples: u32,
    invocation_hosts: [u8; OBJECT_TRACK_COUNT],
    sampler_before: [NormalControlSamplerBefore; OBJECT_TRACK_COUNT],
    control38_rows: [NormalControlRow; PHYSICAL_TRACK_COUNT],
    regular37_rows: [NormalControlRow; REGULAR_TRACK_COUNT],
    control38_carrier_words: Arc<[u32]>,
    regular37_carrier_words: Arc<[u32]>,
}

impl NormalControlSnapshotOutput {
    pub const fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub const fn value_generation(&self) -> FrontendCallbackIndex {
        self.value_generation
    }

    pub const fn normal_origin_generation(&self) -> FrontendCallbackIndex {
        self.normal_origin_generation
    }

    pub const fn next_callback_index(&self) -> FrontendCallbackIndex {
        self.next_callback_index
    }

    pub const fn next_owner_generation(&self) -> u64 {
        self.next_owner_generation
    }

    pub const fn normal_callbacks(&self) -> NormalCallbackCount {
        self.normal_callbacks
    }

    pub fn prepared_identity(&self) -> &NormalControlPreparedSnapshot {
        &self.prepared_identity
    }

    pub const fn query_sample(&self) -> u64 {
        self.query_sample
    }

    pub const fn actual_samples(&self) -> u32 {
        self.actual_samples
    }

    pub const fn invocation_hosts(&self) -> &[u8; OBJECT_TRACK_COUNT] {
        &self.invocation_hosts
    }

    pub const fn sampler_before(&self) -> &[NormalControlSamplerBefore; OBJECT_TRACK_COUNT] {
        &self.sampler_before
    }

    pub const fn control38_rows(&self) -> &[NormalControlRow; PHYSICAL_TRACK_COUNT] {
        &self.control38_rows
    }

    pub const fn regular37_rows(&self) -> &[NormalControlRow; REGULAR_TRACK_COUNT] {
        &self.regular37_rows
    }

    pub fn control38_carrier_words(&self) -> &[u32] {
        &self.control38_carrier_words
    }

    pub fn regular37_carrier_words(&self) -> &[u32] {
        &self.regular37_carrier_words
    }
}

impl NormalControlSnapshotProposal {
    /// Splits the complete proposal without duplicating its mutable sampler after-state.
    pub fn into_frontend_parent_parts(
        self,
    ) -> (NormalControlAfterState, NormalControlSnapshotOutput) {
        let Self {
            transaction_id,
            value_generation,
            normal_origin_generation,
            query_sample,
            actual_samples,
            invocation_hosts,
            sampler_before,
            control38_rows,
            regular37_rows,
            control38_carrier_words,
            regular37_carrier_words,
            after_state,
        } = self;
        let next_callback_index = after_state.next_callback_index;
        let next_owner_generation = after_state.next_owner_generation;
        let normal_callbacks = after_state.next_normal_callbacks;
        let prepared_identity = Arc::clone(&after_state.prepared_identity);
        (
            after_state,
            NormalControlSnapshotOutput {
                transaction_id,
                value_generation,
                normal_origin_generation,
                next_callback_index,
                next_owner_generation,
                normal_callbacks,
                prepared_identity,
                query_sample,
                actual_samples,
                invocation_hosts,
                sampler_before,
                control38_rows,
                regular37_rows,
                control38_carrier_words,
                regular37_carrier_words,
            },
        )
    }

    pub const fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub const fn value_generation(&self) -> FrontendCallbackIndex {
        self.value_generation
    }

    pub const fn normal_origin_generation(&self) -> FrontendCallbackIndex {
        self.normal_origin_generation
    }

    pub const fn query_sample(&self) -> u64 {
        self.query_sample
    }

    pub const fn actual_samples(&self) -> u32 {
        self.actual_samples
    }

    pub const fn invocation_hosts(&self) -> &[u8; OBJECT_TRACK_COUNT] {
        &self.invocation_hosts
    }

    pub const fn sampler_before(&self) -> &[NormalControlSamplerBefore; OBJECT_TRACK_COUNT] {
        &self.sampler_before
    }

    pub const fn control38_rows(&self) -> &[NormalControlRow; PHYSICAL_TRACK_COUNT] {
        &self.control38_rows
    }

    pub const fn regular37_rows(&self) -> &[NormalControlRow; REGULAR_TRACK_COUNT] {
        &self.regular37_rows
    }

    pub const fn after_state(&self) -> &NormalControlAfterState {
        &self.after_state
    }

    /// Returns the owner's rectangular control38 storage. Bed tail positions
    /// are unproduced local zero padding; use the tagged rows for semantic data.
    pub fn control38_carrier_words(&self) -> &[u32] {
        &self.control38_carrier_words
    }

    /// Returns the owner's rectangular regular37 storage. Bed tail positions
    /// are unproduced local zero padding; use the tagged rows for semantic data.
    pub fn regular37_carrier_words(&self) -> &[u32] {
        &self.regular37_carrier_words
    }
}

fn validate_frontend_owner_state(
    state: &FrontendControlState,
) -> Result<(), NormalControlSnapshotError> {
    if state.objects().len() != OBJECT_TRACK_COUNT {
        return Err(NormalControlSnapshotError::StateShapeMismatch);
    }
    for (index, object) in state.objects().iter().enumerate() {
        if object.object_ordinal() != u32::try_from(index).unwrap_or(u32::MAX) {
            return Err(NormalControlSnapshotError::StateShapeMismatch);
        }
    }
    Ok(())
}

fn validate_selected_profile(
    profile: &NormalControlProfileIdentity,
    source: &NormalControlSourceIdentity,
) -> Result<(), NormalControlSnapshotError> {
    if source.input_kind != SelectedInputKind::Adm
        || source.wave.sample_rate_hz() != SELECTED_SAMPLE_RATE_HZ
        || u32::from(source.wave.channels()) != SELECTED_CHANNELS
        || u32::from(source.wave.pcm_bits()) != SELECTED_PCM_BITS
        || source.source_samples == 0
        || profile.sample_rate_hz != SELECTED_SAMPLE_RATE_HZ
        || profile.channels != SELECTED_CHANNELS
        || profile.pcm_bits != SELECTED_PCM_BITS
        || profile.source_samples != source.source_samples
        || profile.frame_samples != u32::try_from(NORMAL_CALLBACK_SAMPLES).unwrap_or(u32::MAX)
        || profile.bed_count != u32::try_from(BED_TRACK_COUNT).unwrap_or(u32::MAX)
        || profile.object_count != u32::try_from(OBJECT_TRACK_COUNT).unwrap_or(u32::MAX)
        || profile.event_start_samples != 0
    {
        return Err(NormalControlSnapshotError::SourceConfigurationMismatch);
    }
    Ok(())
}

fn normal_control_track(
    track: &SelectedPhysicalTrack,
    dbmd: DbmdTrackFields,
    graph: &SelectedAdmGraph,
) -> Result<NormalControlPhysicalTrack, NormalControlSnapshotError> {
    let physical_host = track.physical_host().value();
    let kind = match track.kind() {
        SelectedTrackKind::DirectSpeakersBed {
            bed_id,
            bed_pack_ordinal,
            bed_input_id,
            role,
        } => NormalControlTrackKind::Bed {
            bed_id,
            bed_pack_ordinal,
            semantic_role: role.to_owned(),
            bed_input_id,
        },
        SelectedTrackKind::Object { semantic_key } => {
            let binding = graph
                .object_sampler_owner_binding_for_uid(track.uid())
                .ok_or(NormalControlSnapshotError::ObjectOwnerBindingMismatch)?;
            if binding.physical_host().value() != physical_host
                || binding.semantic_key() != semantic_key
                || binding.channel_format_id() != track.channel_format()
            {
                return Err(NormalControlSnapshotError::ObjectOwnerBindingMismatch);
            }
            NormalControlTrackKind::Object {
                semantic_key,
                local_owner_ordinal: binding.local_owner_ordinal().value(),
            }
        }
    };
    Ok(NormalControlPhysicalTrack {
        track_uid: track.uid().to_owned(),
        object_id: track.object_id().to_owned(),
        physical_host,
        channel_format_id: track.channel_format().to_owned(),
        kind,
        dbmd: NormalControlDbmdFields::from_fields(dbmd),
    })
}

fn validate_prepared_snapshot(
    prepared: &NormalControlPreparedSnapshot,
) -> Result<(), NormalControlSnapshotError> {
    validate_selected_profile(&prepared.profile, &prepared.source_identity)?;
    if prepared.graph_identity.physical_track_count
        != u8::try_from(PHYSICAL_TRACK_COUNT).unwrap_or(u8::MAX)
        || prepared.graph_identity.bed_track_count
            != u8::try_from(BED_TRACK_COUNT).unwrap_or(u8::MAX)
        || prepared.graph_identity.object_track_count
            != u8::try_from(OBJECT_TRACK_COUNT).unwrap_or(u8::MAX)
        || prepared.graph_identity.direct_lfe_host != prepared.lfe_host
        || prepared.source_identity.metadata_hashes.axml() != prepared.graph_identity.axml_sha256
        || prepared.source_identity.metadata_hashes.chna() != prepared.graph_identity.chna_sha256
        || prepared.source_identity.metadata_hashes.dbmd() != prepared.dbmd_payload_sha256
    {
        return Err(NormalControlSnapshotError::MetadataIdentityMismatch);
    }

    let mut uid_seen = std::collections::BTreeSet::new();
    let mut object_count = 0_usize;
    let mut bed_count = 0_usize;
    let mut lfe_host = None;
    for (host_index, track) in prepared.physical_tracks.iter().enumerate() {
        if usize::from(track.physical_host) != host_index
            || track.physical_host >= u8::try_from(PHYSICAL_TRACK_COUNT).unwrap_or(u8::MAX)
            || track.track_uid.is_empty()
            || track.channel_format_id.is_empty()
            || !uid_seen.insert(track.track_uid.as_str())
            || track.dbmd.track_index != u16::from(track.physical_host)
        {
            return Err(NormalControlSnapshotError::PhysicalIdentityMismatch);
        }
        match &track.kind {
            NormalControlTrackKind::Bed {
                bed_id,
                bed_pack_ordinal,
                semantic_role,
                ..
            } => {
                bed_count += 1;
                if *bed_id >= u8::try_from(BED_TRACK_COUNT).unwrap_or(u8::MAX)
                    || *bed_pack_ordinal >= u8::try_from(BED_TRACK_COUNT).unwrap_or(u8::MAX)
                {
                    return Err(NormalControlSnapshotError::BedJoinMismatch);
                }
                if semantic_role == "lfe" {
                    if *bed_id != 3 || lfe_host.replace(track.physical_host).is_some() {
                        return Err(NormalControlSnapshotError::LfeIdentityMismatch);
                    }
                } else if *bed_id == 3 {
                    return Err(NormalControlSnapshotError::LfeIdentityMismatch);
                }
            }
            NormalControlTrackKind::Object { .. } => object_count += 1,
        }
    }
    if bed_count != BED_TRACK_COUNT || object_count != OBJECT_TRACK_COUNT {
        return Err(NormalControlSnapshotError::GraphDomain);
    }
    let lfe_host = lfe_host.ok_or(NormalControlSnapshotError::LfeIdentityMismatch)?;
    if lfe_host != prepared.lfe_host {
        return Err(NormalControlSnapshotError::LfeIdentityMismatch);
    }

    if prepared.owner_bindings.len() != OBJECT_TRACK_COUNT
        || prepared.object_events.len() != OBJECT_TRACK_COUNT
    {
        return Err(NormalControlSnapshotError::ObjectOwnerBindingMismatch);
    }
    for (index, binding) in prepared.owner_bindings.iter().enumerate() {
        if binding.local_owner_ordinal != u8::try_from(index).unwrap_or(u8::MAX)
            || (index > 0
                && prepared.owner_bindings[index - 1].track_uid.as_bytes()
                    >= binding.track_uid.as_bytes())
        {
            return Err(NormalControlSnapshotError::ObjectOwnerBindingMismatch);
        }
        let track = prepared
            .physical_tracks
            .iter()
            .find(|track| track.track_uid == binding.track_uid)
            .ok_or(NormalControlSnapshotError::ObjectOwnerBindingMismatch)?;
        let NormalControlTrackKind::Object {
            semantic_key,
            local_owner_ordinal,
        } = &track.kind
        else {
            return Err(NormalControlSnapshotError::ObjectOwnerBindingMismatch);
        };
        if *local_owner_ordinal != binding.local_owner_ordinal
            || *semantic_key != binding.semantic_key
            || track.physical_host != binding.physical_host
            || track.channel_format_id != binding.channel_format_id
        {
            return Err(NormalControlSnapshotError::ObjectOwnerBindingMismatch);
        }
    }

    for (index, event) in prepared.object_events.iter().enumerate() {
        if index > 0
            && prepared.object_events[index - 1].track_uid.as_bytes() >= event.track_uid.as_bytes()
        {
            return Err(NormalControlSnapshotError::ObjectEventMismatch);
        }
        let binding = find_owner_binding(&prepared.owner_bindings, &event.track_uid)
            .ok_or(NormalControlSnapshotError::ObjectEventMismatch)?;
        let track = prepared
            .physical_tracks
            .iter()
            .find(|track| track.track_uid == event.track_uid)
            .ok_or(NormalControlSnapshotError::ObjectEventMismatch)?;
        let control = &event.control_event;
        if binding.physical_host != event.physical_host
            || track.physical_host != event.physical_host
            || track.object_id != event.object_id
            || control.host_channel_index() != event.physical_host
            || control.start_sample() != 0
            || event.object_start != 0
            || event.block_rtime != 0
            || event.interpolation_length != 0
            || event.object_duration != event.block_duration
            || event.object_duration < prepared.profile.source_samples
            || control.duration_samples() != event.object_duration
            || control.programme_samples() != prepared.profile.source_samples
        {
            return Err(NormalControlSnapshotError::ObjectEventMismatch);
        }
        writer_coordinates(control).map_err(NormalControlSnapshotError::ObjectControl)?;
        writer_uniform_width(control).map_err(NormalControlSnapshotError::ObjectControl)?;
    }

    if prepared.bed_pack_channel_format_ids.len() != BED_TRACK_COUNT
        || prepared.bed_pack_hosts.len() != BED_TRACK_COUNT
        || prepared.regular37_hosts.len() != REGULAR_TRACK_COUNT
    {
        return Err(NormalControlSnapshotError::BedJoinMismatch);
    }
    let mut pack_ids = std::collections::BTreeSet::new();
    for (channel_format_id, host) in prepared
        .bed_pack_channel_format_ids
        .iter()
        .zip(prepared.bed_pack_hosts)
    {
        if !pack_ids.insert(channel_format_id.as_str()) || usize::from(host) >= PHYSICAL_TRACK_COUNT
        {
            return Err(NormalControlSnapshotError::BedJoinMismatch);
        }
        let track = &prepared.physical_tracks[usize::from(host)];
        let NormalControlTrackKind::Bed { .. } = &track.kind else {
            return Err(NormalControlSnapshotError::BedJoinMismatch);
        };
        if track.channel_format_id != *channel_format_id {
            return Err(NormalControlSnapshotError::BedJoinMismatch);
        }
    }
    if prepared.regular37_hosts
        != std::array::from_fn(|index| {
            u8::try_from(if index < usize::from(lfe_host) {
                index
            } else {
                index + 1
            })
            .unwrap_or(u8::MAX)
        })
    {
        return Err(NormalControlSnapshotError::BedJoinMismatch);
    }
    let mut route_hosts = std::collections::BTreeSet::new();
    let mut route_sources = std::collections::BTreeSet::new();
    for (rank, binding) in prepared.regular_source_bindings.iter().enumerate() {
        if binding.physical_host != prepared.regular37_hosts[rank]
            || binding.physical_host == prepared.lfe_host
            || usize::from(binding.physical_host) >= PHYSICAL_TRACK_COUNT
            || !(11..48).contains(&binding.global_source)
            || !route_hosts.insert(binding.physical_host)
            || !route_sources.insert(binding.global_source)
        {
            return Err(NormalControlSnapshotError::BedJoinMismatch);
        }
    }
    Ok(())
}

fn validate_before_state(
    before: &NormalControlBeforeState,
    prepared: &NormalControlPreparedSnapshot,
    transaction_id: &CallbackLineageIdentity,
) -> Result<(u64, u64, u32), NormalControlSnapshotError> {
    if before.programme_id != prepared.programme_id
        || before.attempt_id != prepared.attempt_id
        || transaction_id.programme_id() != &before.programme_id
        || transaction_id.attempt_id() != before.attempt_id
    {
        return Err(NormalControlSnapshotError::CallbackIdentityMismatch);
    }
    if before.profile != prepared.profile {
        return Err(NormalControlSnapshotError::SourceConfigurationMismatch);
    }
    let generation = u64::from(before.callback_index.value());
    if transaction_id.callback_index() != before.callback_index
        || before.normal_callbacks.value() != generation
    {
        return Err(NormalControlSnapshotError::CallbackClockMismatch);
    }
    if before.owner_generation != generation {
        return Err(NormalControlSnapshotError::GenerationMismatch);
    }
    if before.callback_index.value() == 0 {
        if before.previous_prepared_identity.is_some()
            || before.last_control38_snapshot.is_some()
            || before.last_regular37_snapshot.is_some()
            || before.physical_binding.is_some()
            || before.dbmd_binding.is_some()
            || before.state_digest != prepared.frontend_state_digest
        {
            return Err(NormalControlSnapshotError::ColdStateRequired);
        }
    } else {
        let previous = before
            .previous_prepared_identity
            .as_ref()
            .ok_or(NormalControlSnapshotError::PreparedIdentityMismatch)?;
        if !previous.same_prepared_input_identity(prepared) {
            return Err(NormalControlSnapshotError::PreparedIdentityMismatch);
        }
        if before
            .last_control38_snapshot
            .as_ref()
            .is_none_or(|snapshot| snapshot.len() != CONTROL38_WORD_COUNT)
            || before
                .last_regular37_snapshot
                .as_ref()
                .is_none_or(|snapshot| snapshot.len() != REGULAR37_WORD_COUNT)
        {
            return Err(NormalControlSnapshotError::SnapshotShapeMismatch);
        }
    }

    for (index, object) in before.objects.iter().enumerate() {
        if object.object_ordinal != u32::try_from(index).unwrap_or(u32::MAX) {
            return Err(NormalControlSnapshotError::StateShapeMismatch);
        }
    }
    let owners_by_ordinal = owner_bindings_by_ordinal(&prepared.owner_bindings)?;
    for (index, object) in before.objects.iter().enumerate() {
        let binding = owners_by_ordinal[index];
        let expected_host = Some(u32::from(binding.physical_host));
        if generation == 0 {
            if object.event_cache.is_some()
                || object.host_channel_index.is_some()
                || object.sampler_previous_present
                || object.sampler_previous_xyz_u32 != [0; 3]
            {
                return Err(NormalControlSnapshotError::ColdStateRequired);
            }
        } else if object.host_channel_index != expected_host
            || !object.sampler_previous_present
            || object
                .sampler_previous_xyz_u32
                .iter()
                .any(|word| FiniteF32::new(f32::from_bits(*word)).is_err())
        {
            return Err(NormalControlSnapshotError::StateOwnerMismatch);
        }
    }

    let query_sample = generation
        .checked_mul(NORMAL_CALLBACK_SAMPLES)
        .ok_or(NormalControlSnapshotError::IndexOverflow)?;
    if query_sample >= prepared.profile.source_samples {
        return Err(NormalControlSnapshotError::CallbackClockMismatch);
    }
    let remaining = prepared.profile.source_samples - query_sample;
    let actual_samples = u32::try_from(remaining.min(NORMAL_CALLBACK_SAMPLES))
        .map_err(|_| NormalControlSnapshotError::IndexOverflow)?;
    Ok((generation, query_sample, actual_samples))
}

/// Produces one complete NORMAL-callback proposal. It never changes the owner.
pub fn propose_normal_control_snapshot(
    before: &NormalControlBeforeState,
    prepared: &Arc<NormalControlPreparedSnapshot>,
    transaction_id: CallbackLineageIdentity,
) -> Result<NormalControlSnapshotProposal, NormalControlSnapshotError> {
    validate_prepared_snapshot(prepared)?;
    let (_generation, query_sample, actual_samples) =
        validate_before_state(before, prepared, &transaction_id)?;
    let owners_by_ordinal = owner_bindings_by_ordinal(&prepared.owner_bindings)?;

    let mut normalized_by_owner = vec![None; OBJECT_TRACK_COUNT];
    for (binding, event) in prepared.owner_bindings.iter().zip(&prepared.object_events) {
        if binding.track_uid != event.track_uid {
            return Err(NormalControlSnapshotError::ObjectEventMismatch);
        }
        let ordinal = usize::from(binding.local_owner_ordinal);
        let normalized = NormalizedObjectControl {
            writer_xyz: writer_coordinates(&event.control_event)
                .map_err(NormalControlSnapshotError::ObjectControl)?,
            uniform_width: writer_uniform_width(&event.control_event)
                .map_err(NormalControlSnapshotError::ObjectControl)?,
            diffuse: event.control_event.diffuse(),
        };
        if normalized_by_owner[ordinal].replace(normalized).is_some() {
            return Err(NormalControlSnapshotError::ObjectOwnerBindingMismatch);
        }
    }
    let normalized_by_owner: [Option<NormalizedObjectControl>; OBJECT_TRACK_COUNT] =
        normalized_by_owner
            .try_into()
            .map_err(|_| NormalControlSnapshotError::ObjectEventMismatch)?;
    let normalized_by_owner: [NormalizedObjectControl; OBJECT_TRACK_COUNT] = normalized_by_owner
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or(NormalControlSnapshotError::ObjectEventMismatch)?
        .try_into()
        .map_err(|_| NormalControlSnapshotError::ObjectEventMismatch)?;

    let mut sampler_before = Vec::with_capacity(OBJECT_TRACK_COUNT);
    for (ordinal, binding) in owners_by_ordinal.iter().enumerate() {
        let object = &before.objects[ordinal];
        sampler_before.push(NormalControlSamplerBefore {
            local_owner_ordinal: binding.local_owner_ordinal,
            physical_host: binding.physical_host,
            sampler_previous_present: object.sampler_previous_present,
            sampler_previous_xyz_u32: object.sampler_previous_xyz_u32,
        });
    }
    let sampler_before: [NormalControlSamplerBefore; OBJECT_TRACK_COUNT] = sampler_before
        .try_into()
        .map_err(|_| NormalControlSnapshotError::StateShapeMismatch)?;

    let mut next_objects = vec![None; OBJECT_TRACK_COUNT];
    let mut invocation_hosts = Vec::with_capacity(OBJECT_TRACK_COUNT);
    let mut control38_rows = Vec::with_capacity(PHYSICAL_TRACK_COUNT);
    let mut control38_words = vec![0_u32; CONTROL38_WORD_COUNT];
    for track in &prepared.physical_tracks {
        let host = track.physical_host;
        let row_offset = usize::from(host)
            .checked_mul(CONTROL_WORDS_PER_TRACK)
            .ok_or(NormalControlSnapshotError::IndexOverflow)?;
        match &track.kind {
            NormalControlTrackKind::Bed { bed_id, .. } => {
                let words = [0, 0, 3, 2, 1, u32::from(*bed_id)];
                control38_words[row_offset..row_offset + words.len()].copy_from_slice(&words);
                control38_rows.push(NormalControlRow::Bed6 {
                    physical_host: host,
                    track_uid: track.track_uid.clone(),
                    channel_format_id: track.channel_format_id.clone(),
                    dbmd: track.dbmd,
                    words,
                    unproduced_tail: UnproducedBedTail {
                        offset_bytes: 24,
                        byte_count: 28,
                    },
                });
            }
            NormalControlTrackKind::Object {
                semantic_key,
                local_owner_ordinal: track_ordinal,
            } => {
                invocation_hosts.push(host);
                let binding = find_owner_binding(&prepared.owner_bindings, &track.track_uid)
                    .ok_or(NormalControlSnapshotError::ObjectOwnerBindingMismatch)?;
                let ordinal = usize::from(binding.local_owner_ordinal);
                if binding.local_owner_ordinal != *track_ordinal
                    || binding.physical_host != host
                    || binding.semantic_key != *semantic_key
                    || binding.channel_format_id != track.channel_format_id
                {
                    return Err(NormalControlSnapshotError::ObjectOwnerBindingMismatch);
                }
                let event = find_object_event(&prepared.object_events, &track.track_uid)
                    .ok_or(NormalControlSnapshotError::ObjectEventMismatch)?;
                if event.physical_host != host || event.object_id != track.object_id {
                    return Err(NormalControlSnapshotError::ObjectEventMismatch);
                }
                let normalized = normalized_by_owner[ordinal];
                let before_object = &before.objects[ordinal];
                let previous_xyz = if before_object.sampler_previous_present {
                    before_object.sampler_previous_xyz_u32
                } else {
                    normalized.writer_xyz
                };
                let sampled_xyz = sample_event_from_previous(normalized.writer_xyz, previous_xyz)
                    .map_err(NormalControlSnapshotError::ObjectControl)?;
                let control = ObjectControl52::from_normal_sample(
                    sampled_xyz,
                    normalized.uniform_width,
                    normalized.diffuse,
                );
                let words = control.words();
                control38_words[row_offset..row_offset + CONTROL_WORDS_PER_TRACK]
                    .copy_from_slice(&words);
                if next_objects[ordinal]
                    .replace(before_object.after_normal_callback(host, sampled_xyz))
                    .is_some()
                {
                    return Err(NormalControlSnapshotError::ObjectOwnerBindingMismatch);
                }
                control38_rows.push(NormalControlRow::Object13 {
                    physical_host: host,
                    track_uid: track.track_uid.clone(),
                    channel_format_id: track.channel_format_id.clone(),
                    semantic_key: *semantic_key,
                    local_owner_ordinal: binding.local_owner_ordinal,
                    dbmd: track.dbmd,
                    control,
                });
            }
        }
    }

    if invocation_hosts.len() != OBJECT_TRACK_COUNT {
        return Err(NormalControlSnapshotError::GraphDomain);
    }
    let invocation_hosts: [u8; OBJECT_TRACK_COUNT] = invocation_hosts
        .try_into()
        .map_err(|_| NormalControlSnapshotError::GraphDomain)?;
    let control38_rows: [NormalControlRow; PHYSICAL_TRACK_COUNT] = control38_rows
        .try_into()
        .map_err(|_| NormalControlSnapshotError::GraphDomain)?;
    let next_objects: [NormalControlObjectState; OBJECT_TRACK_COUNT] = next_objects
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or(NormalControlSnapshotError::ObjectOwnerBindingMismatch)?
        .try_into()
        .map_err(|_| NormalControlSnapshotError::ObjectOwnerBindingMismatch)?;

    let regular37_rows: [NormalControlRow; REGULAR_TRACK_COUNT] = prepared
        .regular37_hosts
        .iter()
        .map(|host| control38_rows[usize::from(*host)].clone())
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| NormalControlSnapshotError::BedJoinMismatch)?;
    let mut regular37_words = Vec::with_capacity(REGULAR37_WORD_COUNT);
    for row in &regular37_rows {
        match row {
            NormalControlRow::Bed6 { words, .. } => {
                regular37_words.extend_from_slice(words);
                regular37_words.resize(regular37_words.len() + CONTROL_WORDS_PER_TRACK - 6, 0);
            }
            NormalControlRow::Object13 { control, .. } => {
                regular37_words.extend_from_slice(&control.words());
            }
        }
    }
    if control38_words.len() != CONTROL38_WORD_COUNT
        || regular37_words.len() != REGULAR37_WORD_COUNT
    {
        return Err(NormalControlSnapshotError::SnapshotShapeMismatch);
    }

    let next_callback_value = before
        .callback_index
        .value()
        .checked_add(1)
        .ok_or(NormalControlSnapshotError::IndexOverflow)?;
    let next_generation = before
        .owner_generation
        .checked_add(1)
        .ok_or(NormalControlSnapshotError::IndexOverflow)?;
    let next_normal_count = before
        .normal_callbacks
        .value()
        .checked_add(1)
        .ok_or(NormalControlSnapshotError::IndexOverflow)?;
    let control38_carrier_words: Arc<[u32]> = control38_words.into();
    let regular37_carrier_words: Arc<[u32]> = regular37_words.into();
    let after_state = NormalControlAfterState {
        programme_id: before.programme_id.clone(),
        attempt_id: before.attempt_id,
        next_callback_index: FrontendCallbackIndex::new(next_callback_value),
        next_normal_callbacks: NormalCallbackCount::new(next_normal_count),
        next_owner_generation: next_generation,
        remaining_cached_drains: before.remaining_cached_drains,
        phase: before.phase,
        objects: next_objects,
        control38_snapshot: Arc::clone(&control38_carrier_words),
        regular37_snapshot: Arc::clone(&regular37_carrier_words),
        physical_binding_before: before.physical_binding.clone(),
        dbmd_binding_before: before.dbmd_binding.clone(),
        prepared_identity: Arc::clone(prepared),
    };

    Ok(NormalControlSnapshotProposal {
        transaction_id,
        value_generation: before.callback_index,
        normal_origin_generation: before.callback_index,
        query_sample,
        actual_samples,
        invocation_hosts,
        sampler_before,
        control38_rows,
        regular37_rows,
        control38_carrier_words,
        regular37_carrier_words,
        after_state,
    })
}

#[derive(Clone, Copy)]
struct NormalizedObjectControl {
    writer_xyz: [u32; 3],
    uniform_width: u32,
    diffuse: Option<i32>,
}

fn owner_bindings_by_ordinal(
    bindings: &[NormalControlObjectOwnerBinding; OBJECT_TRACK_COUNT],
) -> Result<[&NormalControlObjectOwnerBinding; OBJECT_TRACK_COUNT], NormalControlSnapshotError> {
    let mut slots = vec![None; OBJECT_TRACK_COUNT];
    for binding in bindings {
        let ordinal = usize::from(binding.local_owner_ordinal);
        if ordinal >= OBJECT_TRACK_COUNT || slots[ordinal].replace(binding).is_some() {
            return Err(NormalControlSnapshotError::ObjectOwnerBindingMismatch);
        }
    }
    slots
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or(NormalControlSnapshotError::ObjectOwnerBindingMismatch)?
        .try_into()
        .map_err(|_| NormalControlSnapshotError::ObjectOwnerBindingMismatch)
}

fn find_owner_binding<'a>(
    bindings: &'a [NormalControlObjectOwnerBinding; OBJECT_TRACK_COUNT],
    track_uid: &str,
) -> Option<&'a NormalControlObjectOwnerBinding> {
    let index = bindings
        .binary_search_by(|binding| binding.track_uid.as_bytes().cmp(track_uid.as_bytes()))
        .ok()?;
    bindings.get(index)
}

fn find_object_event<'a>(
    events: &'a [NormalControlObjectEvent; OBJECT_TRACK_COUNT],
    track_uid: &str,
) -> Option<&'a NormalControlObjectEvent> {
    let index = events
        .binary_search_by(|event| event.track_uid.as_bytes().cmp(track_uid.as_bytes()))
        .ok()?;
    events.get(index)
}

#[cfg(test)]
mod tests;
