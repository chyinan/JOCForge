// pattern: Functional Core
//! Selected input value preparation and the currently unavailable encode boundary.

pub use jocforge_domain::frontend::job_configuration::SelectedWaveDescriptor;
use jocforge_domain::frontend::job_configuration::{
    JobConfigurationError, PrimaryControlBindingError, PrimaryMetadataChunkHashes,
    SelectedJobConfiguration, SelectedPrimaryControlBinding, SelectedPrimaryRuntimeIdentity,
    bind_admitted_selected_input, parse_selected_job_configuration,
};
use jocforge_domain::frontend::{
    F01EProviderError, FrontendControlError, FrontendControlMachine, FrontendControlState,
    ValidatedF01EColdProvider,
    adm_file_storage_map::{
        AdmFileStorageChannelMap, AdmFileStorageMapError, build_adm_file_storage_channel_map,
    },
    adm_object_events::AdmittedSelectedObjectEvents,
    dbmd_sections::{DbmdSectionError, SelectedDbmdSections, parse_selected_dbmd_sections},
    normal_control_snapshot::{
        NormalControlAdmFreshZeroReceipt, NormalControlAdmInputProposal,
        NormalControlAdmInputSnapshot, NormalControlPreparedSnapshot, NormalControlSnapshotError,
    },
    primary_metadata_control::{
        PrimaryMetadataControlError, PrimaryMetadataControlRows,
        produce_primary_metadata_control_rows,
    },
    selected_adm_graph::{SelectedAdmGraph, SelectedAdmGraphError, parse_selected_adm_graph},
};
use jocforge_domain::meter::prepared_job_identity::{
    NormalizedJobConfigurationIdentity, PreparedAdmChunkSpan,
    PreparedAdmGraphProvenance as DomainAdmGraphProvenance, PreparedInputResourceIdentity,
    PreparedJobInputIdentity, PreparedJobInputIdentityError,
};
pub use jocforge_domain::state::EncodeError;
use jocforge_domain::timeline::{AdmSampleCount, ClockError, FrontendCallbackIndex};
pub use jocforge_domain::transaction::ProgrammeId;
use jocforge_domain::transaction::{AttemptId, CallbackLineageIdentity};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::Arc;

pub mod codec_source_bridge;
pub mod drc_source_bridge;
pub mod independent_meter_callback;
pub mod integrity_source_bridge;
pub mod metadata_source_bridge;
pub mod meter_finalization_bridge;
pub mod meter_source_admission;
pub mod meter_source_bridge;
pub mod output_publication;
pub mod reader_eof_callback;
pub mod source_analysis_bridge;
pub mod source_frame_bridge;
pub mod spatial_callback;

/// LOCAL_TRANSLATION: immutable caller-resource binding after the input adapter
/// validates RIFF and supplies its descriptor and sample extent. This value is
/// input preparation only; it grants no metering, READY or encode authority.
#[derive(Debug)]
pub struct PreparedJobInput {
    programme_id: ProgrammeId,
    configuration: SelectedJobConfiguration,
    graph: PreparedAdmGraph,
    dbmd: SelectedDbmdSections,
    admitted_object_events: AdmittedSelectedObjectEvents,
    identity_envelope: Value,
    job_xml_sha256: [u8; 32],
    job_xml_bytes: u64,
    config_sha256: [u8; 32],
    resource: Arc<[u8]>,
    resource_identity: ResourceIdentity,
    domain_identity: Arc<PreparedJobInputIdentity>,
    primary_control_binding: Option<SelectedPrimaryControlBinding>,
    primary_metadata_controls: Option<PrimaryMetadataControlRows>,
}

/// Cold F01E owner paired with the exact immutable input admission that created it.
pub struct PreparedF01EColdOwner {
    machine: FrontendControlMachine,
    prepared_identity: Arc<PreparedJobInputIdentity>,
}

impl PreparedF01EColdOwner {
    pub const fn committed(&self) -> &FrontendControlState {
        self.machine.committed()
    }

    pub const fn generation(&self) -> u64 {
        self.machine.generation()
    }

    pub const fn attempt_id(&self) -> AttemptId {
        self.machine.committed().attempt_id()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceIdentity {
    input_sha256: [u8; 32],
    resource_bytes: u64,
}

impl ResourceIdentity {
    pub const fn input_sha256(self) -> [u8; 32] {
        self.input_sha256
    }

    pub const fn resource_bytes(self) -> u64 {
        self.resource_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceChunkSpan {
    chunk_id: [u8; 4],
    payload_offset: u64,
    payload_bytes: u64,
    payload_sha256: [u8; 32],
}

impl ResourceChunkSpan {
    pub const fn chunk_id(self) -> [u8; 4] {
        self.chunk_id
    }

    pub const fn payload_offset(self) -> u64 {
        self.payload_offset
    }

    pub const fn payload_bytes(self) -> u64 {
        self.payload_bytes
    }

    pub const fn payload_sha256(self) -> [u8; 32] {
        self.payload_sha256
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparedAdmGraphProvenance {
    parent_resource: ResourceIdentity,
    axml: ResourceChunkSpan,
    chna: ResourceChunkSpan,
    dbmd: ResourceChunkSpan,
}

impl PreparedAdmGraphProvenance {
    pub const fn parent_resource(self) -> ResourceIdentity {
        self.parent_resource
    }

    pub const fn axml(self) -> ResourceChunkSpan {
        self.axml
    }

    pub const fn chna(self) -> ResourceChunkSpan {
        self.chna
    }

    pub const fn dbmd(self) -> ResourceChunkSpan {
        self.dbmd
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreparedAdmGraph {
    graph: SelectedAdmGraph,
    provenance: PreparedAdmGraphProvenance,
}

impl PreparedAdmGraph {
    pub const fn graph(&self) -> &SelectedAdmGraph {
        &self.graph
    }

    pub const fn provenance(&self) -> PreparedAdmGraphProvenance {
        self.provenance
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreparedAdmFileStorageMap {
    resource_identity: ResourceIdentity,
    axml: ResourceChunkSpan,
    chna: ResourceChunkSpan,
    channels: AdmFileStorageChannelMap,
}

impl PreparedAdmFileStorageMap {
    pub const fn resource_identity(&self) -> ResourceIdentity {
        self.resource_identity
    }

    pub const fn axml(&self) -> ResourceChunkSpan {
        self.axml
    }

    pub const fn chna(&self) -> ResourceChunkSpan {
        self.chna
    }

    pub const fn channels(&self) -> &AdmFileStorageChannelMap {
        &self.channels
    }
}

/// Embedded ADM payloads borrowed from the same immutable resource allocation.
#[derive(Clone, Copy, Debug)]
pub struct SelectedAdmResourcePayloads<'a> {
    axml: &'a [u8],
    chna: &'a [u8],
    dbmd: &'a [u8],
}

impl<'a> SelectedAdmResourcePayloads<'a> {
    pub const fn new(axml: &'a [u8], chna: &'a [u8], dbmd: &'a [u8]) -> Self {
        Self { axml, chna, dbmd }
    }
}

impl PreparedJobInput {
    pub fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub const fn configuration(&self) -> &SelectedJobConfiguration {
        &self.configuration
    }

    pub const fn graph(&self) -> &PreparedAdmGraph {
        &self.graph
    }

    pub const fn dbmd(&self) -> &SelectedDbmdSections {
        &self.dbmd
    }

    pub const fn admitted_object_events(&self) -> &AdmittedSelectedObjectEvents {
        &self.admitted_object_events
    }

    pub fn resource(&self) -> &[u8] {
        &self.resource
    }

    pub const fn resource_identity(&self) -> ResourceIdentity {
        self.resource_identity
    }

    /// Returns the one domain-owned identity projection derived during admission.
    pub fn domain_identity(&self) -> &PreparedJobInputIdentity {
        self.domain_identity.as_ref()
    }

    /// Clones the one admission-created identity capability without recomputing either hash.
    pub fn domain_identity_capability(&self) -> Arc<PreparedJobInputIdentity> {
        Arc::clone(&self.domain_identity)
    }

    pub const fn primary_control_binding(&self) -> Option<&SelectedPrimaryControlBinding> {
        self.primary_control_binding.as_ref()
    }

    pub const fn primary_metadata_controls(&self) -> Option<&PrimaryMetadataControlRows> {
        self.primary_metadata_controls.as_ref()
    }

    /// Adds the F01E primary-source binding after an IO adapter supplies the actual data extent.
    pub fn bind_primary_control_rows(
        mut self,
        data_bytes: u64,
    ) -> Result<Self, PreparedJobInputError> {
        let graph = self.graph.graph();
        let binding = bind_admitted_selected_input(
            &self.configuration,
            SelectedPrimaryRuntimeIdentity {
                source_file_bytes: self.resource_identity.resource_bytes(),
                data_bytes,
                metadata_hashes: PrimaryMetadataChunkHashes::new(
                    *graph.receipt().axml_sha256(),
                    *graph.receipt().chna_sha256(),
                    *self.dbmd.payload_sha256(),
                ),
                resource_sha256: self.resource_identity.input_sha256(),
                config_sha256: self.config_sha256,
                job_xml_sha256: self.job_xml_sha256,
                job_xml_bytes: self.job_xml_bytes,
            },
        )
        .map_err(PreparedJobInputError::PrimaryControlBinding)?;
        let controls = produce_primary_metadata_control_rows(&binding, graph, &self.dbmd)
            .map_err(PreparedJobInputError::PrimaryMetadataControls)?;
        self.primary_control_binding = Some(binding);
        self.primary_metadata_controls = Some(controls);
        Ok(self)
    }

    /// Constructs the versioned F01E cold owner from this input's paired configuration and graph.
    ///
    /// This binds the cold projection to one immutable `PreparedJobInput` and its programme/attempt
    /// identity. It does not resize or stage the 38-row normal-control snapshot, execute a callback,
    /// or create a DDP frame-view descriptor.
    pub fn f01e_cold_owner(
        &self,
        provider: &ValidatedF01EColdProvider,
        attempt_id: AttemptId,
    ) -> Result<PreparedF01EColdOwner, PreparedJobInputError> {
        let projected = provider
            .for_admitted_adm_input(&self.configuration, self.graph.graph())
            .map_err(PreparedJobInputError::F01EProviderProjection)?;
        let machine =
            FrontendControlMachine::from_provider(projected, self.programme_id.clone(), attempt_id)
                .map_err(PreparedJobInputError::F01EColdOwner)?;
        Ok(PreparedF01EColdOwner {
            machine,
            prepared_identity: Arc::clone(&self.domain_identity),
        })
    }

    fn validate_f01e_cold_owner(
        &self,
        owner: &PreparedF01EColdOwner,
    ) -> Result<(), PreparedJobInputError> {
        if !Arc::ptr_eq(&self.domain_identity, &owner.prepared_identity)
            || owner.committed().programme_id() != &self.programme_id
            || owner.committed().schema() != "F01E_ADM_INPUT_COLD_PROJECTION_V1"
            || owner.committed().normalized_config().channels()
                != u32::from(self.configuration.wave().channels())
            || owner.committed().normalized_config().source_samples()
                != self.configuration.source_samples()
        {
            return Err(PreparedJobInputError::PreparedIdentityMismatch);
        }
        Ok(())
    }

    /// Binds one dynamic normal-control input view to this prepared input and its F01E owner.
    pub fn normal_control_adm_input_snapshot(
        &self,
        owner: &PreparedF01EColdOwner,
    ) -> Result<Arc<NormalControlAdmInputSnapshot>, PreparedJobInputError> {
        self.validate_f01e_cold_owner(owner)?;
        let binding = self
            .primary_control_binding
            .as_ref()
            .ok_or(PreparedJobInputError::PrimaryControlBindingUnavailable)?;
        NormalControlAdmInputSnapshot::from_admitted_adm_input(
            owner.committed(),
            owner.generation(),
            binding,
            self.graph.graph(),
            &self.admitted_object_events,
            &self.dbmd,
            Arc::clone(&self.domain_identity),
        )
        .map(Arc::new)
        .map_err(PreparedJobInputError::NormalControlSnapshot)
    }

    /// Proposes the first per-input NORMAL callback without installing owner after-state.
    pub fn propose_f01e_adm_normal_callback(
        &self,
        owner: &PreparedF01EColdOwner,
    ) -> Result<NormalControlAdmInputProposal, PreparedJobInputError> {
        let snapshot = self.normal_control_adm_input_snapshot(owner)?;
        let transaction_id = CallbackLineageIdentity::new(
            self.programme_id.clone(),
            owner.attempt_id(),
            owner.committed().callback_index(),
        );
        snapshot
            .propose_first_normal_callback(owner.committed(), owner.generation(), transaction_id)
            .map_err(PreparedJobInputError::NormalControlSnapshot)
    }

    /// Binds one Meter zero-input frame to this prepared input and its completed normal proposal.
    pub fn bind_f01e_adm_fresh_zero_rows(
        &self,
        owner: &PreparedF01EColdOwner,
        proposal: &NormalControlAdmInputProposal,
        source_view: &meter_source_bridge::MeterSourceCallbackView<'_>,
    ) -> Result<NormalControlAdmFreshZeroReceipt, PreparedJobInputError> {
        self.validate_f01e_cold_owner(owner)?;
        let after = proposal.after_state();
        if !Arc::ptr_eq(&self.domain_identity, after.prepared_input_identity())
            || !Arc::ptr_eq(&self.domain_identity, &owner.prepared_identity)
            || proposal.before_generation() != owner.generation()
            || proposal.before_digest() != owner.committed().digest()
            || proposal.transaction_id().programme_id() != self.programme_id()
            || proposal.transaction_id().attempt_id() != owner.attempt_id()
            || proposal.transaction_id().callback_index() != owner.committed().callback_index()
            || !std::ptr::eq(self.graph.graph(), source_view.selected_graph())
            || source_view.source_identity().sha256() != self.resource_identity.input_sha256()
            || source_view.source_samples().value() != self.configuration.source_samples()
            || source_view.normal_callbacks().value() != after.expected_normal_callback_count()
            || source_view.phase() != meter_source_bridge::MeterCallbackPhase::ZeroInputDrain
            || source_view.source_cursor_before().value() != self.configuration.source_samples()
            || source_view.source_cursor_after().value() != self.configuration.source_samples()
            || source_view.source_read_index_before().value()
                != after.expected_normal_callback_count()
            || source_view.source_read_index_after().value()
                != after.expected_normal_callback_count()
            || source_view.host_input_count().value() != 0
            || source_view.source_padding_count().value() != 1_536
        {
            return Err(PreparedJobInputError::PreparedIdentityMismatch);
        }
        let callback_index = u32::try_from(source_view.callback_index().value()).map_err(|_| {
            PreparedJobInputError::NormalControlSnapshot(
                NormalControlSnapshotError::FreshZeroWindowMismatch,
            )
        })?;
        let delivery = CallbackLineageIdentity::new(
            self.programme_id.clone(),
            owner.attempt_id(),
            FrontendCallbackIndex::new(callback_index),
        );
        after
            .fresh_zero_view(delivery)
            .and_then(|view| view.validate_zero_physical_input_rows(source_view.physical_rows()))
            .map_err(PreparedJobInputError::NormalControlSnapshot)
    }

    /// Builds the reusable F01E callback snapshot from this admitted input and one cold owner.
    pub fn normal_control_prepared_snapshot(
        &self,
        state: &FrontendControlState,
    ) -> Result<Arc<NormalControlPreparedSnapshot>, PreparedJobInputError> {
        if state.programme_id() != &self.programme_id {
            return Err(PreparedJobInputError::ProgrammeIdentityMismatch);
        }
        let binding = self
            .primary_control_binding
            .as_ref()
            .ok_or(PreparedJobInputError::PrimaryControlBindingUnavailable)?;
        NormalControlPreparedSnapshot::from_validated(
            state,
            binding,
            self.graph.graph(),
            &self.admitted_object_events,
            &self.dbmd,
        )
        .map_err(PreparedJobInputError::NormalControlSnapshot)
    }

    pub const fn job_xml_sha256(&self) -> [u8; 32] {
        self.job_xml_sha256
    }

    pub const fn config_sha256(&self) -> [u8; 32] {
        self.config_sha256
    }

    pub const fn identity_envelope(&self) -> &Value {
        &self.identity_envelope
    }

    /// Exposes the converter-local channel map with the metadata's enclosing resource identity.
    ///
    /// This binds the map to this prepared AXML/CHNA resource; it does not bind map values to
    /// physical PCM rows or the downstream meter frame descriptor.
    pub fn converter_storage_map(
        &self,
    ) -> Result<PreparedAdmFileStorageMap, PreparedJobInputError> {
        let channels = build_adm_file_storage_channel_map(&self.graph.graph)
            .map_err(PreparedJobInputError::ConverterStorageMap)?;
        Ok(PreparedAdmFileStorageMap {
            resource_identity: self.resource_identity,
            axml: self.graph.provenance.axml,
            chna: self.graph.provenance.chna,
            channels,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreparedJobInputError {
    Configuration(JobConfigurationError),
    SampleCount(ClockError),
    GraphAdmission(SelectedAdmGraphError),
    GraphParse(SelectedAdmGraphError),
    ConverterStorageMap(AdmFileStorageMapError),
    DbmdParse(DbmdSectionError),
    MetadataPayloadBinding,
    WaveTrackCountMismatch,
    IdentityEnvelope,
    IdentityProjection(PreparedJobInputIdentityError),
    PrimaryControlBinding(PrimaryControlBindingError),
    PrimaryMetadataControls(PrimaryMetadataControlError),
    PrimaryControlBindingUnavailable,
    ProgrammeIdentityMismatch,
    PreparedIdentityMismatch,
    F01EProviderProjection(F01EProviderError),
    F01EColdOwner(FrontendControlError),
    NormalControlSnapshot(NormalControlSnapshotError),
}

impl std::fmt::Display for PreparedJobInputError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Configuration(error) => write!(formatter, "job configuration: {error:?}"),
            Self::SampleCount(error) => write!(formatter, "source sample count: {error:?}"),
            Self::GraphAdmission(error) => write!(formatter, "ADM graph admission: {error:?}"),
            Self::GraphParse(error) => write!(formatter, "ADM graph parse: {error:?}"),
            Self::ConverterStorageMap(error) => {
                write!(formatter, "ADM converter storage map: {error:?}")
            }
            Self::DbmdParse(error) => write!(formatter, "DBMD parse: {error:?}"),
            Self::MetadataPayloadBinding => formatter.write_str(
                "ADM metadata payload is not a unique subslice of the selected resource",
            ),
            Self::WaveTrackCountMismatch => formatter.write_str(
                "RIFF PCM channel count does not match the admitted CHNA physical-track count",
            ),
            Self::IdentityEnvelope => {
                formatter.write_str("failed to serialize selected job identity envelope")
            }
            Self::IdentityProjection(error) => {
                write!(formatter, "prepared job identity projection: {error:?}")
            }
            Self::PrimaryControlBinding(error) => {
                write!(formatter, "primary control binding: {error:?}")
            }
            Self::PrimaryMetadataControls(error) => {
                write!(formatter, "primary metadata controls: {error:?}")
            }
            Self::PrimaryControlBindingUnavailable => {
                formatter.write_str("selected PCM extent is required before F01E callback staging")
            }
            Self::ProgrammeIdentityMismatch => {
                formatter.write_str("prepared input and F01E owner have different programme IDs")
            }
            Self::PreparedIdentityMismatch => formatter
                .write_str("F01E cold owner is bound to a different prepared input identity"),
            Self::F01EProviderProjection(error) => {
                write!(formatter, "F01E dynamic cold projection: {error:?}")
            }
            Self::F01EColdOwner(error) => {
                write!(formatter, "F01E cold owner construction: {error:?}")
            }
            Self::NormalControlSnapshot(error) => {
                write!(
                    formatter,
                    "F01E normal-control prepared snapshot: {error:?}"
                )
            }
        }
    }
}

impl std::error::Error for PreparedJobInputError {}

/// Normalizes a selected job and binds embedded AXML/CHNA/DBMD views to one
/// caller-owned immutable resource. The input adapter must parse and validate
/// RIFF first, derive `wave`, `source_samples`, and payload views from that same
/// allocation, then pass them here. The caller supplies the opaque programme
/// identity separately; it is not derived from resource/config hashes. File
/// references in XML remain intent.
pub fn prepare_selected_job_input(
    programme_id: ProgrammeId,
    xml: &[u8],
    wave: SelectedWaveDescriptor,
    source_samples: u64,
    resource: Arc<[u8]>,
    metadata_payloads: SelectedAdmResourcePayloads<'_>,
) -> Result<PreparedJobInput, PreparedJobInputError> {
    let configuration = parse_selected_job_configuration(xml, wave, source_samples)
        .map_err(PreparedJobInputError::Configuration)?;
    let resource_bytes =
        u64::try_from(resource.len()).map_err(|_| PreparedJobInputError::MetadataPayloadBinding)?;
    let input_sha256: [u8; 32] = Sha256::digest(resource.as_ref()).into();
    let resource_identity = ResourceIdentity {
        input_sha256,
        resource_bytes,
    };
    let axml_span = bind_resource_payload(&resource, *b"axml", metadata_payloads.axml)?;
    let chna_span = bind_resource_payload(&resource, *b"chna", metadata_payloads.chna)?;
    let dbmd_span = bind_resource_payload(&resource, *b"dbmd", metadata_payloads.dbmd)?;
    let graph = parse_selected_adm_graph(metadata_payloads.axml, metadata_payloads.chna)
        .map_err(PreparedJobInputError::GraphParse)?;
    if usize::from(wave.channels()) != graph.tracks().len() {
        return Err(PreparedJobInputError::WaveTrackCountMismatch);
    }
    if graph.receipt().axml_sha256() != &axml_span.payload_sha256()
        || graph.receipt().chna_sha256() != &chna_span.payload_sha256()
    {
        return Err(PreparedJobInputError::MetadataPayloadBinding);
    }
    let dbmd = parse_selected_dbmd_sections(metadata_payloads.dbmd)
        .map_err(PreparedJobInputError::DbmdParse)?;
    let admitted_sample_count = AdmSampleCount::new(configuration.source_samples())
        .map_err(PreparedJobInputError::SampleCount)?;
    let admitted_object_events = graph
        .admit_object_events(admitted_sample_count)
        .map_err(PreparedJobInputError::GraphAdmission)?;
    let job_xml_sha256: [u8; 32] = Sha256::digest(xml).into();
    let identity_envelope = canonicalize_identity(json!({
        "schema": "selected-job-config-identity.v1",
        "job_xml_sha256": hex_lower(&job_xml_sha256),
        "selected_input_label": configuration.input_kind().selected_label(),
        "normalized_f01e": configuration.normalized_identity_value()
    }));
    let identity_bytes = serde_json::to_vec(&identity_envelope)
        .map_err(|_| PreparedJobInputError::IdentityEnvelope)?;
    let config_sha256: [u8; 32] = Sha256::digest(identity_bytes).into();
    let job_xml_bytes = u64::try_from(xml.len()).map_err(|_| {
        PreparedJobInputError::PrimaryControlBinding(PrimaryControlBindingError::SourceExtent)
    })?;
    let graph = PreparedAdmGraph {
        graph,
        provenance: PreparedAdmGraphProvenance {
            parent_resource: resource_identity,
            axml: axml_span,
            chna: chna_span,
            dbmd: dbmd_span,
        },
    };
    let domain_resource = PreparedInputResourceIdentity::new(
        resource_identity.input_sha256(),
        resource_identity.resource_bytes(),
    );
    let graph_provenance = graph.provenance();
    let domain_graph = DomainAdmGraphProvenance::new(
        PreparedInputResourceIdentity::new(
            graph_provenance.parent_resource().input_sha256(),
            graph_provenance.parent_resource().resource_bytes(),
        ),
        PreparedAdmChunkSpan::new(
            graph_provenance.axml().chunk_id(),
            graph_provenance.axml().payload_offset(),
            graph_provenance.axml().payload_bytes(),
            graph_provenance.axml().payload_sha256(),
        ),
        PreparedAdmChunkSpan::new(
            graph_provenance.chna().chunk_id(),
            graph_provenance.chna().payload_offset(),
            graph_provenance.chna().payload_bytes(),
            graph_provenance.chna().payload_sha256(),
        ),
        PreparedAdmChunkSpan::new(
            graph_provenance.dbmd().chunk_id(),
            graph_provenance.dbmd().payload_offset(),
            graph_provenance.dbmd().payload_bytes(),
            graph_provenance.dbmd().payload_sha256(),
        ),
    );
    let domain_identity = PreparedJobInputIdentity::new(
        domain_resource,
        NormalizedJobConfigurationIdentity::new(
            config_sha256,
            job_xml_sha256,
            configuration.input_kind(),
        ),
        domain_graph,
    )
    .map_err(PreparedJobInputError::IdentityProjection)?;
    Ok(PreparedJobInput {
        programme_id,
        configuration,
        graph,
        dbmd,
        admitted_object_events,
        identity_envelope,
        job_xml_sha256,
        job_xml_bytes,
        config_sha256,
        resource,
        resource_identity,
        domain_identity: Arc::new(domain_identity),
        primary_control_binding: None,
        primary_metadata_controls: None,
    })
}

fn bind_resource_payload(
    resource: &[u8],
    chunk_id: [u8; 4],
    payload: &[u8],
) -> Result<ResourceChunkSpan, PreparedJobInputError> {
    if payload.is_empty() {
        return Err(PreparedJobInputError::MetadataPayloadBinding);
    }
    let base = resource.as_ptr() as usize;
    let start = payload.as_ptr() as usize;
    let offset = start
        .checked_sub(base)
        .ok_or(PreparedJobInputError::MetadataPayloadBinding)?;
    let end = offset
        .checked_add(payload.len())
        .ok_or(PreparedJobInputError::MetadataPayloadBinding)?;
    if end > resource.len() || resource.get(offset..end) != Some(payload) {
        return Err(PreparedJobInputError::MetadataPayloadBinding);
    }
    let header_start = offset
        .checked_sub(8)
        .ok_or(PreparedJobInputError::MetadataPayloadBinding)?;
    let header = resource
        .get(header_start..offset)
        .ok_or(PreparedJobInputError::MetadataPayloadBinding)?;
    let declared_payload_bytes = u32::from_le_bytes(
        header[4..8]
            .try_into()
            .map_err(|_| PreparedJobInputError::MetadataPayloadBinding)?,
    );
    if header[..4] != chunk_id
        || usize::try_from(declared_payload_bytes).ok() != Some(payload.len())
    {
        return Err(PreparedJobInputError::MetadataPayloadBinding);
    }
    Ok(ResourceChunkSpan {
        chunk_id,
        payload_offset: u64::try_from(offset)
            .map_err(|_| PreparedJobInputError::MetadataPayloadBinding)?,
        payload_bytes: u64::try_from(payload.len())
            .map_err(|_| PreparedJobInputError::MetadataPayloadBinding)?,
        payload_sha256: Sha256::digest(payload).into(),
    })
}

fn canonicalize_identity(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let sorted: BTreeMap<_, _> = object.into_iter().collect();
            let mut canonical = Map::new();
            for (key, value) in sorted {
                canonical.insert(key, canonicalize_identity(value));
            }
            Value::Object(canonical)
        }
        Value::Array(values) => {
            Value::Array(values.into_iter().map(canonicalize_identity).collect())
        }
        scalar => scalar,
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

/// Validates selected owner providers and destination authority before source execution.
///
/// Provider values and the output capability stay private to the application. Source callbacks
/// and AU publication still need their complete owner proposals before this call can write output.
pub fn encode(
    prepared: &PreparedJobInput,
    admitted_plan: &output_publication::AdmittedProductionPlan,
) -> Result<(), EncodeError> {
    if !admitted_plan.is_bound_to(prepared) {
        return Err(EncodeError::F15AdmissionRejected);
    }
    Err(EncodeError::SourcePipelineUnavailable)
}
