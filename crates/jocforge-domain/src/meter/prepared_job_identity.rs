// pattern: Functional Core
//! Pure identity projection for one admitted prepared job input.

use crate::frontend::job_configuration::SelectedInputKind;

/// Identity of the complete validated resource snapshot shared by meter and main.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparedInputResourceIdentity {
    input_sha256: [u8; 32],
    resource_bytes: u64,
}

impl PreparedInputResourceIdentity {
    pub const fn new(input_sha256: [u8; 32], resource_bytes: u64) -> Self {
        Self {
            input_sha256,
            resource_bytes,
        }
    }

    pub const fn input_sha256(self) -> [u8; 32] {
        self.input_sha256
    }

    pub const fn resource_bytes(self) -> u64 {
        self.resource_bytes
    }
}

/// Immutable metadata-chunk provenance into the same validated resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparedAdmChunkSpan {
    chunk_id: [u8; 4],
    payload_offset: u64,
    payload_bytes: u64,
    payload_sha256: [u8; 32],
}

impl PreparedAdmChunkSpan {
    pub const fn new(
        chunk_id: [u8; 4],
        payload_offset: u64,
        payload_bytes: u64,
        payload_sha256: [u8; 32],
    ) -> Self {
        Self {
            chunk_id,
            payload_offset,
            payload_bytes,
            payload_sha256,
        }
    }

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

/// Parsed ADM provenance, anchored to its parent resource and exact chunk spans.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparedAdmGraphProvenance {
    parent_resource: PreparedInputResourceIdentity,
    axml: PreparedAdmChunkSpan,
    chna: PreparedAdmChunkSpan,
    dbmd: PreparedAdmChunkSpan,
}

impl PreparedAdmGraphProvenance {
    pub const fn new(
        parent_resource: PreparedInputResourceIdentity,
        axml: PreparedAdmChunkSpan,
        chna: PreparedAdmChunkSpan,
        dbmd: PreparedAdmChunkSpan,
    ) -> Self {
        Self {
            parent_resource,
            axml,
            chna,
            dbmd,
        }
    }

    pub const fn parent_resource(self) -> PreparedInputResourceIdentity {
        self.parent_resource
    }

    pub const fn axml(self) -> PreparedAdmChunkSpan {
        self.axml
    }

    pub const fn chna(self) -> PreparedAdmChunkSpan {
        self.chna
    }

    pub const fn dbmd(self) -> PreparedAdmChunkSpan {
        self.dbmd
    }
}

/// Hash-bound normalized F01E identity plus the raw XML and selected input kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NormalizedJobConfigurationIdentity {
    config_sha256: [u8; 32],
    job_xml_sha256: [u8; 32],
    selected_input_kind: SelectedInputKind,
}

impl NormalizedJobConfigurationIdentity {
    pub const fn new(
        config_sha256: [u8; 32],
        job_xml_sha256: [u8; 32],
        selected_input_kind: SelectedInputKind,
    ) -> Self {
        Self {
            config_sha256,
            job_xml_sha256,
            selected_input_kind,
        }
    }

    pub const fn config_sha256(self) -> [u8; 32] {
        self.config_sha256
    }

    pub const fn job_xml_sha256(self) -> [u8; 32] {
        self.job_xml_sha256
    }

    pub const fn selected_input_kind(self) -> SelectedInputKind {
        self.selected_input_kind
    }

    pub const fn selected_input_label(self) -> &'static str {
        self.selected_input_kind.selected_label()
    }
}

/// Structural identity failures in the projection derived from one admitted input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreparedJobInputIdentityError {
    GraphResourceMismatch,
    ChunkIdentityMismatch,
    ChunkSpanOverflow,
    ChunkOutsideResource,
}

/// One immutable, I/O-free identity projection derived from a single admitted job input.
///
/// This carries no path or file handle. It is provenance for a future meter-finalization
/// proposal; it does not itself mint a measurement receipt or authorize a parent commit.
#[derive(Debug, Eq, PartialEq)]
pub struct PreparedJobInputIdentity {
    resource: PreparedInputResourceIdentity,
    normalized_configuration: NormalizedJobConfigurationIdentity,
    graph_provenance: PreparedAdmGraphProvenance,
}

impl PreparedJobInputIdentity {
    pub fn new(
        resource: PreparedInputResourceIdentity,
        normalized_configuration: NormalizedJobConfigurationIdentity,
        graph_provenance: PreparedAdmGraphProvenance,
    ) -> Result<Self, PreparedJobInputIdentityError> {
        if graph_provenance.parent_resource() != resource {
            return Err(PreparedJobInputIdentityError::GraphResourceMismatch);
        }

        validate_chunk_span(resource, graph_provenance.axml(), *b"axml")?;
        validate_chunk_span(resource, graph_provenance.chna(), *b"chna")?;
        validate_chunk_span(resource, graph_provenance.dbmd(), *b"dbmd")?;

        Ok(Self {
            resource,
            normalized_configuration,
            graph_provenance,
        })
    }

    pub const fn resource(&self) -> PreparedInputResourceIdentity {
        self.resource
    }

    pub const fn normalized_configuration(&self) -> NormalizedJobConfigurationIdentity {
        self.normalized_configuration
    }

    pub const fn graph_provenance(&self) -> &PreparedAdmGraphProvenance {
        &self.graph_provenance
    }
}

fn validate_chunk_span(
    resource: PreparedInputResourceIdentity,
    span: PreparedAdmChunkSpan,
    expected_chunk_id: [u8; 4],
) -> Result<(), PreparedJobInputIdentityError> {
    if span.chunk_id() != expected_chunk_id {
        return Err(PreparedJobInputIdentityError::ChunkIdentityMismatch);
    }
    let end = span
        .payload_offset()
        .checked_add(span.payload_bytes())
        .ok_or(PreparedJobInputIdentityError::ChunkSpanOverflow)?;
    if end > resource.resource_bytes() {
        return Err(PreparedJobInputIdentityError::ChunkOutsideResource);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_resource(sha256: [u8; 32]) -> PreparedInputResourceIdentity {
        PreparedInputResourceIdentity::new(sha256, 1_000)
    }

    fn configuration() -> NormalizedJobConfigurationIdentity {
        NormalizedJobConfigurationIdentity::new([0x22; 32], [0x33; 32], SelectedInputKind::Adm)
    }

    fn graph(parent_resource: PreparedInputResourceIdentity) -> PreparedAdmGraphProvenance {
        PreparedAdmGraphProvenance::new(
            parent_resource,
            PreparedAdmChunkSpan::new(*b"axml", 100, 50, [0x41; 32]),
            PreparedAdmChunkSpan::new(*b"chna", 200, 30, [0x42; 32]),
            PreparedAdmChunkSpan::new(*b"dbmd", 300, 40, [0x43; 32]),
        )
    }

    #[test]
    fn identity_projection_keeps_resource_config_and_graph_nested() {
        let resource = make_resource([0x11; 32]);
        let config = configuration();
        let graph = graph(resource);
        let identity = PreparedJobInputIdentity::new(resource, config, graph).unwrap();

        assert_eq!(identity.resource(), resource);
        assert_eq!(identity.normalized_configuration(), config);
        assert_eq!(
            identity.normalized_configuration().selected_input_label(),
            "adm"
        );
        assert_eq!(identity.graph_provenance().parent_resource(), resource);
        assert_eq!(identity.graph_provenance().axml().payload_offset(), 100);
    }

    #[test]
    fn a_graph_from_another_resource_is_rejected() {
        let resource = make_resource([0x11; 32]);
        let foreign_resource = make_resource([0x77; 32]);
        assert_eq!(
            PreparedJobInputIdentity::new(resource, configuration(), graph(foreign_resource)),
            Err(PreparedJobInputIdentityError::GraphResourceMismatch)
        );
    }

    #[test]
    fn chunk_kinds_and_resource_bounds_are_preflighted() {
        let resource = make_resource([0x11; 32]);
        let bad_kind = PreparedAdmGraphProvenance::new(
            resource,
            PreparedAdmChunkSpan::new(*b"chna", 100, 50, [0x41; 32]),
            graph(resource).chna(),
            graph(resource).dbmd(),
        );
        assert_eq!(
            PreparedJobInputIdentity::new(resource, configuration(), bad_kind),
            Err(PreparedJobInputIdentityError::ChunkIdentityMismatch)
        );

        let out_of_bounds = PreparedAdmGraphProvenance::new(
            resource,
            PreparedAdmChunkSpan::new(*b"axml", 990, 20, [0x41; 32]),
            graph(resource).chna(),
            graph(resource).dbmd(),
        );
        assert_eq!(
            PreparedJobInputIdentity::new(resource, configuration(), out_of_bounds),
            Err(PreparedJobInputIdentityError::ChunkOutsideResource)
        );

        let overflow = PreparedAdmGraphProvenance::new(
            resource,
            PreparedAdmChunkSpan::new(*b"axml", u64::MAX, 1, [0x41; 32]),
            graph(resource).chna(),
            graph(resource).dbmd(),
        );
        assert_eq!(
            PreparedJobInputIdentity::new(resource, configuration(), overflow),
            Err(PreparedJobInputIdentityError::ChunkSpanOverflow)
        );
    }
}
