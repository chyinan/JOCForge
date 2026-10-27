// pattern: Functional Core
//! Selected input value preparation and the currently unavailable encode boundary.

pub use jocforge_domain::frontend::job_configuration::SelectedWaveDescriptor;
use jocforge_domain::frontend::job_configuration::{
    JobConfigurationError, SelectedJobConfiguration, parse_selected_job_configuration,
};
use jocforge_domain::frontend::{
    adm_file_storage_map::{
        AdmFileStorageChannelMap, AdmFileStorageMapError, build_adm_file_storage_channel_map,
    },
    adm_object_events::AdmittedSelectedObjectEvents,
    dbmd_sections::{DbmdSectionError, SelectedDbmdSections, parse_selected_dbmd_sections},
    selected_adm_graph::{SelectedAdmGraph, SelectedAdmGraphError, parse_selected_adm_graph},
};
use jocforge_domain::meter::prepared_job_identity::{
    NormalizedJobConfigurationIdentity, PreparedAdmChunkSpan,
    PreparedAdmGraphProvenance as DomainAdmGraphProvenance, PreparedInputResourceIdentity,
    PreparedJobInputIdentity, PreparedJobInputIdentityError,
};
use jocforge_domain::state::EncodeError;
use jocforge_domain::timeline::{AdmSampleCount, ClockError};
pub use jocforge_domain::transaction::ProgrammeId;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::Arc;

pub mod meter_source_bridge;
pub mod output_publication;

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
    config_sha256: [u8; 32],
    resource: Arc<[u8]>,
    resource_identity: ResourceIdentity,
    domain_identity: PreparedJobInputIdentity,
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
    pub const fn domain_identity(&self) -> &PreparedJobInputIdentity {
        &self.domain_identity
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
    IdentityEnvelope,
    IdentityProjection(PreparedJobInputIdentityError),
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
            Self::IdentityEnvelope => {
                formatter.write_str("failed to serialize selected job identity envelope")
            }
            Self::IdentityProjection(error) => {
                write!(formatter, "prepared job identity projection: {error:?}")
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
        config_sha256,
        resource,
        resource_identity,
        domain_identity,
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

/// Rejects encoding until the architecture and implementation have been reviewed.
pub const fn encode() -> Result<(), EncodeError> {
    Err(EncodeError::BackendUnavailable)
}
