// pattern: Functional Core
//! Primary-source F01E defined metadata/control row production.

use std::collections::BTreeMap;

use crate::frontend::dbmd_sections::{DbmdTrackFields, SelectedDbmdSections};
use crate::frontend::job_configuration::{
    PrimaryMetadataChunkHashes, SelectedPrimaryControlBinding,
};
use crate::frontend::object_control::{
    ObjectControl52, ObjectControlError, SelectedObjectControlSet,
};
use crate::frontend::selected_adm_graph::{
    SelectedAdmGraph, SelectedAdmGraphError, SelectedPhysicalTrack, SelectedTrackKind,
};

const CONTROL38_ROW_COUNT: usize = 38;
const OBJECT_ROW_COUNT: usize = 28;
const BED_ROW_COUNT: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnproducedControlBytes {
    offset: u8,
    byte_count: u8,
}

impl UnproducedControlBytes {
    pub const fn offset(self) -> u8 {
        self.offset
    }

    pub const fn byte_count(self) -> u8 {
        self.byte_count
    }
}

/// Defined control fields only. Bed bytes 24..51 remain explicitly unproduced.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimaryControl38Fields {
    Bed {
        words: [u32; 6],
        unproduced_tail: UnproducedControlBytes,
    },
    Object {
        control52: ObjectControl52,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrimaryMetadataControlRow {
    track: SelectedPhysicalTrack,
    dbmd: DbmdTrackFields,
    fields: PrimaryControl38Fields,
}

impl PrimaryMetadataControlRow {
    pub fn track(&self) -> &SelectedPhysicalTrack {
        &self.track
    }

    pub const fn dbmd(&self) -> DbmdTrackFields {
        self.dbmd
    }

    pub const fn fields(&self) -> PrimaryControl38Fields {
        self.fields
    }
}

/// Primary metadata/control values in physical host order; no file publication or commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrimaryMetadataControlRows {
    source_binding: SelectedPrimaryControlBinding,
    rows: [PrimaryMetadataControlRow; CONTROL38_ROW_COUNT],
}

impl PrimaryMetadataControlRows {
    pub fn source_binding(&self) -> &SelectedPrimaryControlBinding {
        &self.source_binding
    }

    pub fn rows(&self) -> &[PrimaryMetadataControlRow; CONTROL38_ROW_COUNT] {
        &self.rows
    }

    pub const fn row_count(&self) -> usize {
        CONTROL38_ROW_COUNT
    }

    pub const fn object_control_word_count(&self) -> usize {
        OBJECT_ROW_COUNT * 13
    }

    pub const fn defined_control_word_count(&self) -> usize {
        OBJECT_ROW_COUNT * 13 + BED_ROW_COUNT * 6
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimaryMetadataControlError {
    SourceMetadataIdentity,
    DbmdTrackMissing,
    BedIdentity,
    ObjectIdentity,
    Graph(SelectedAdmGraphError),
    ObjectControl(ObjectControlError),
}

impl PrimaryMetadataControlError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::SourceMetadataIdentity => "PRIMARY_CONTROL_SOURCE_IDENTITY",
            Self::DbmdTrackMissing => "PRIMARY_CONTROL_DBMD_TRACK_MISSING",
            Self::BedIdentity => "PRIMARY_CONTROL_BED_IDENTITY",
            Self::ObjectIdentity => "PRIMARY_CONTROL_OBJECT_IDENTITY",
            Self::Graph(error) => error.code(),
            Self::ObjectControl(error) => error.code(),
        }
    }
}

/// Recomputes all F01E-defined control rows from the bound primary job projection,
/// actual caller-supplied AXML/CHNA graph bytes, and parser-produced DBMD fields.
/// It does not produce unknown bed padding or connect to `application::encode`.
pub fn produce_primary_metadata_control_rows(
    source_binding: &SelectedPrimaryControlBinding,
    graph: &SelectedAdmGraph,
    dbmd: &SelectedDbmdSections,
) -> Result<PrimaryMetadataControlRows, PrimaryMetadataControlError> {
    validate_source_metadata_identity(source_binding.metadata_hashes(), graph, dbmd)?;
    let object_controls = graph
        .build_candidate_object_controls(source_binding.source_samples(), dbmd)
        .map_err(PrimaryMetadataControlError::Graph)?;
    let object_by_host = object_rows_by_host(graph, &object_controls)?;

    let mut rows = Vec::with_capacity(CONTROL38_ROW_COUNT);
    for track in graph.tracks() {
        let host = track.physical_host().value();
        let dbmd_track = dbmd
            .fields_for_physical_host(u16::from(host))
            .ok_or(PrimaryMetadataControlError::DbmdTrackMissing)?;
        let fields = match track.kind() {
            SelectedTrackKind::DirectSpeakersBed { bed_input_id, .. } => {
                PrimaryControl38Fields::Bed {
                    words: [0, 0, 3, 2, 1, bed_input_id],
                    unproduced_tail: UnproducedControlBytes {
                        offset: 24,
                        byte_count: 28,
                    },
                }
            }
            SelectedTrackKind::Object { .. } => PrimaryControl38Fields::Object {
                control52: object_by_host
                    .get(&host)
                    .copied()
                    .ok_or(PrimaryMetadataControlError::ObjectIdentity)?,
            },
        };
        rows.push(PrimaryMetadataControlRow {
            track: track.clone(),
            dbmd: dbmd_track,
            fields,
        });
    }
    let rows = rows
        .try_into()
        .map_err(|_| PrimaryMetadataControlError::ObjectIdentity)?;
    Ok(PrimaryMetadataControlRows {
        source_binding: source_binding.clone(),
        rows,
    })
}

fn validate_source_metadata_identity(
    expected: PrimaryMetadataChunkHashes,
    graph: &SelectedAdmGraph,
    dbmd: &SelectedDbmdSections,
) -> Result<(), PrimaryMetadataControlError> {
    if graph.receipt().axml_sha256() != &expected.axml()
        || graph.receipt().chna_sha256() != &expected.chna()
        || dbmd.payload_sha256() != &expected.dbmd()
    {
        return Err(PrimaryMetadataControlError::SourceMetadataIdentity);
    }
    Ok(())
}

fn object_rows_by_host(
    graph: &SelectedAdmGraph,
    controls: &SelectedObjectControlSet,
) -> Result<BTreeMap<u8, ObjectControl52>, PrimaryMetadataControlError> {
    let hosts = graph.bed_channel_join().object_hosts();
    if hosts.len() != OBJECT_ROW_COUNT || controls.rows().len() != OBJECT_ROW_COUNT {
        return Err(PrimaryMetadataControlError::ObjectIdentity);
    }
    Ok(hosts
        .iter()
        .copied()
        .zip(controls.rows().iter().copied())
        .collect())
}
