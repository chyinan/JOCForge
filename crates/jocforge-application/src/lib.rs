// pattern: Functional Core
//! Selected input value preparation and the currently unavailable encode boundary.

use jocforge_domain::frontend::job_configuration::{
    JobConfigurationError, SelectedJobConfiguration, SelectedWaveDescriptor,
    parse_selected_job_configuration,
};
use jocforge_domain::frontend::{
    adm_object_events::AdmittedSelectedObjectEvents,
    selected_adm_graph::{SelectedAdmGraph, SelectedAdmGraphError},
};
use jocforge_domain::state::EncodeError;
use jocforge_domain::timeline::{AdmSampleCount, ClockError};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// LOCAL_TRANSLATION: immutable caller-resource binding after the input adapter
/// validates RIFF and supplies its descriptor and sample extent. This value is
/// input preparation only; it grants no metering, READY or encode authority.
#[derive(Debug)]
pub struct PreparedJobInput {
    configuration: SelectedJobConfiguration,
    graph: SelectedAdmGraph,
    admitted_object_events: AdmittedSelectedObjectEvents,
    resource: Arc<[u8]>,
    resource_sha256: [u8; 32],
}

impl PreparedJobInput {
    pub const fn configuration(&self) -> &SelectedJobConfiguration {
        &self.configuration
    }

    pub const fn graph(&self) -> &SelectedAdmGraph {
        &self.graph
    }

    pub const fn admitted_object_events(&self) -> &AdmittedSelectedObjectEvents {
        &self.admitted_object_events
    }

    pub fn resource(&self) -> &[u8] {
        &self.resource
    }

    pub const fn resource_sha256(&self) -> [u8; 32] {
        self.resource_sha256
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreparedJobInputError {
    Configuration(JobConfigurationError),
    SampleCount(ClockError),
    GraphAdmission(SelectedAdmGraphError),
}

impl std::fmt::Display for PreparedJobInputError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Configuration(error) => write!(formatter, "job configuration: {error:?}"),
            Self::SampleCount(error) => write!(formatter, "source sample count: {error:?}"),
            Self::GraphAdmission(error) => write!(formatter, "ADM graph admission: {error:?}"),
        }
    }
}

impl std::error::Error for PreparedJobInputError {}

/// Normalizes the selected job and binds the caller's immutable resource by its
/// actual bytes. The input adapter must validate RIFF and derive `wave` and
/// `source_samples` from this same resource before calling. File references in
/// XML remain intent, never an instruction to open another resource.
pub fn prepare_selected_job_input(
    xml: &[u8],
    wave: SelectedWaveDescriptor,
    source_samples: u64,
    resource: Arc<[u8]>,
    graph: SelectedAdmGraph,
) -> Result<PreparedJobInput, PreparedJobInputError> {
    let configuration = parse_selected_job_configuration(xml, wave, source_samples)
        .map_err(PreparedJobInputError::Configuration)?;
    let admitted_sample_count = AdmSampleCount::new(configuration.source_samples())
        .map_err(PreparedJobInputError::SampleCount)?;
    let admitted_object_events = graph
        .admit_object_events(admitted_sample_count)
        .map_err(PreparedJobInputError::GraphAdmission)?;
    let resource_sha256 = Sha256::digest(resource.as_ref()).into();
    Ok(PreparedJobInput {
        configuration,
        graph,
        admitted_object_events,
        resource,
        resource_sha256,
    })
}

/// Rejects encoding until the architecture and implementation have been reviewed.
pub const fn encode() -> Result<(), EncodeError> {
    Err(EncodeError::BackendUnavailable)
}
