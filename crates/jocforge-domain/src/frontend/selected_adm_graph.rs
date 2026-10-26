// pattern: Functional Core
//! Selected real AXML/CHNA identity graph and physical-host route projection.

use std::collections::{BTreeMap, BTreeSet};

use roxmltree::{Document, Node};
use sha2::{Digest, Sha256};

use crate::frontend::adm_object_events::{
    AdmittedSelectedObjectEvents, RawAdmObjectEvent, SelectedAdmObjectEventError,
    admit_selected_object_events,
};
use crate::frontend::bed_channel_join::{
    BedChannelJoinError, PhysicalBedTrack, SelectedBedChannelJoin, join_selected_bed_channels,
};
use crate::frontend::dbmd_sections::SelectedDbmdSections;
use crate::frontend::object_control::{
    ObjectControlError, SelectedObjectControlSet, build_selected_object_control_set,
};
use crate::timeline::{AdmSampleCount, CodecSourceIndex};

const CHNA_HEADER_BYTES: usize = 4;
const CHNA_ENTRY_BYTES: usize = 40;
const CHNA_TRACK_INDEX_BYTES: usize = 2;
const CHNA_UID_BYTES: usize = 12;
const CHNA_TRACK_FORMAT_BYTES: usize = 14;
const CHNA_PACK_FORMAT_BYTES: usize = 11;
const SELECTED_TRACK_COUNT: usize = 38;
const SELECTED_BED_COUNT: usize = 10;
const SELECTED_OBJECT_COUNT: usize = 28;
const VIRTUAL_PREFIX_SOURCE_COUNT: u64 = 11;
const SELECTED_TOTAL_SOURCE_COUNT: u64 = 48;

/// One-based CHNA track index, distinct from its zero-based physical host.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ChnaTrackIndex(u16);

impl ChnaTrackIndex {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u16 {
        self.0
    }
}

/// Zero-based host channel slot, distinct from codec source and AU indices.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicalHostIndex(u8);

impl PhysicalHostIndex {
    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedTrackKind {
    DirectSpeakersBed {
        bed_id: u8,
        bed_pack_ordinal: u8,
        bed_input_id: u32,
        role: &'static str,
    },
    Object {
        semantic_key: u32,
    },
}

/// A clean local owner slot for an admitted object track.
///
/// This ordinal is allocated from TrackUID byte order. It does not assert a
/// native sampler-array order or replace the track's physical-host identity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ObjectSamplerOwnerOrdinal(u8);

impl ObjectSamplerOwnerOrdinal {
    pub const fn value(self) -> u8 {
        self.0
    }
}

/// Immutable association between a physical object track and its local sampler slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectSamplerOwnerBinding {
    track_uid: String,
    local_owner_ordinal: ObjectSamplerOwnerOrdinal,
    semantic_key: u32,
    physical_host: PhysicalHostIndex,
    channel_format_id: String,
}

impl ObjectSamplerOwnerBinding {
    pub fn track_uid(&self) -> &str {
        &self.track_uid
    }

    pub const fn local_owner_ordinal(&self) -> ObjectSamplerOwnerOrdinal {
        self.local_owner_ordinal
    }

    pub const fn semantic_key(&self) -> u32 {
        self.semantic_key
    }

    pub const fn physical_host(&self) -> PhysicalHostIndex {
        self.physical_host
    }

    pub fn channel_format_id(&self) -> &str {
        &self.channel_format_id
    }
}

/// Graph identity and descriptor identity for one CHNA physical track.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedPhysicalTrack {
    physical_host: PhysicalHostIndex,
    track_index: ChnaTrackIndex,
    uid: String,
    track_format: String,
    stream_format: String,
    channel_format: String,
    pack_format: String,
    object_id: String,
    descriptor_semantic_key: u32,
    kind: SelectedTrackKind,
    block_count: u16,
    chna_reserved: u8,
}

impl SelectedPhysicalTrack {
    pub const fn physical_host(&self) -> PhysicalHostIndex {
        self.physical_host
    }

    pub const fn track_index(&self) -> ChnaTrackIndex {
        self.track_index
    }

    pub fn uid(&self) -> &str {
        &self.uid
    }

    pub fn track_format(&self) -> &str {
        &self.track_format
    }

    pub fn stream_format(&self) -> &str {
        &self.stream_format
    }

    pub fn channel_format(&self) -> &str {
        &self.channel_format
    }

    pub fn pack_format(&self) -> &str {
        &self.pack_format
    }

    pub fn object_id(&self) -> &str {
        &self.object_id
    }

    pub const fn descriptor_semantic_key(&self) -> u32 {
        self.descriptor_semantic_key
    }

    pub const fn kind(&self) -> SelectedTrackKind {
        self.kind
    }

    pub const fn block_count(&self) -> u16 {
        self.block_count
    }

    pub const fn chna_reserved(&self) -> u8 {
        self.chna_reserved
    }
}

/// Physical host to global regular source join; no source+0/prefix transition is modeled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegularSourceBinding {
    physical_host: PhysicalHostIndex,
    global_source: CodecSourceIndex,
}

impl RegularSourceBinding {
    pub const fn physical_host(self) -> PhysicalHostIndex {
        self.physical_host
    }

    pub const fn global_source(self) -> CodecSourceIndex {
        self.global_source
    }
}

/// Hash-bound local receipt over the caller-provided metadata chunks only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedAdmGraphReceipt {
    axml_sha256: [u8; 32],
    chna_sha256: [u8; 32],
    physical_track_count: u8,
    bed_track_count: u8,
    object_track_count: u8,
    direct_lfe_host: PhysicalHostIndex,
    virtual_prefix_source_count: u8,
    total_source_count: u8,
}

impl SelectedAdmGraphReceipt {
    pub const fn axml_sha256(&self) -> &[u8; 32] {
        &self.axml_sha256
    }

    pub const fn chna_sha256(&self) -> &[u8; 32] {
        &self.chna_sha256
    }

    pub const fn physical_track_count(&self) -> u8 {
        self.physical_track_count
    }

    pub const fn bed_track_count(&self) -> u8 {
        self.bed_track_count
    }

    pub const fn object_track_count(&self) -> u8 {
        self.object_track_count
    }

    pub const fn direct_lfe_host(&self) -> PhysicalHostIndex {
        self.direct_lfe_host
    }

    pub const fn virtual_prefix_source_count(&self) -> u8 {
        self.virtual_prefix_source_count
    }

    pub const fn total_source_count(&self) -> u8 {
        self.total_source_count
    }
}

/// Selected valid-input graph projection. It does not parse event numerics or install control38.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedAdmGraph {
    tracks: [SelectedPhysicalTrack; SELECTED_TRACK_COUNT],
    object_events: [RawAdmObjectEvent; SELECTED_OBJECT_COUNT],
    object_sampler_owner_bindings: [ObjectSamplerOwnerBinding; SELECTED_OBJECT_COUNT],
    bed_pack_channel_format_ids: [String; SELECTED_BED_COUNT],
    bed_join: SelectedBedChannelJoin,
    regular_source_bindings: [RegularSourceBinding; SELECTED_TRACK_COUNT - 1],
    uid_to_track_index: BTreeMap<String, ChnaTrackIndex>,
    receipt: SelectedAdmGraphReceipt,
}

impl SelectedAdmGraph {
    pub fn tracks(&self) -> &[SelectedPhysicalTrack; SELECTED_TRACK_COUNT] {
        &self.tracks
    }

    pub fn bed_pack_channel_format_ids(&self) -> &[String; SELECTED_BED_COUNT] {
        &self.bed_pack_channel_format_ids
    }

    pub const fn bed_channel_join(&self) -> &SelectedBedChannelJoin {
        &self.bed_join
    }

    /// Local owner slots in unsigned ASCII TrackUID byte order.
    ///
    /// Runtime invocation remains a separate physical-host-ordered traversal;
    /// callers join each track to this table by TrackUID.
    pub const fn object_sampler_owner_bindings(
        &self,
    ) -> &[ObjectSamplerOwnerBinding; SELECTED_OBJECT_COUNT] {
        &self.object_sampler_owner_bindings
    }

    pub fn object_sampler_owner_binding_for_uid(
        &self,
        track_uid: &str,
    ) -> Option<&ObjectSamplerOwnerBinding> {
        find_object_sampler_owner_binding(&self.object_sampler_owner_bindings, track_uid)
    }

    pub const fn regular_source_bindings(
        &self,
    ) -> &[RegularSourceBinding; SELECTED_TRACK_COUNT - 1] {
        &self.regular_source_bindings
    }

    pub const fn receipt(&self) -> &SelectedAdmGraphReceipt {
        &self.receipt
    }

    /// Admits actual selected AXML object events against the caller's validated source sample count.
    pub fn admit_object_events(
        &self,
        source_samples: AdmSampleCount,
    ) -> Result<AdmittedSelectedObjectEvents, SelectedAdmGraphError> {
        admit_selected_object_events(self.object_events.clone(), source_samples)
            .map_err(SelectedAdmGraphError::ObjectEvent)
    }

    /// Model lookup is immutable and returns `None` for absent/empty UIDs.
    /// This is not a claim about native output-sentinel or input-string mutation behavior.
    pub fn track_index_for_uid(&self, uid: &str) -> Option<ChnaTrackIndex> {
        self.uid_to_track_index.get(uid).copied()
    }

    /// Builds candidate-source controls from this graph and caller-supplied parsed DBMD fields.
    /// This API has no binding to the F01E selected job or production encode path.
    pub fn build_candidate_object_controls(
        &self,
        source_samples: AdmSampleCount,
        dbmd: &SelectedDbmdSections,
    ) -> Result<SelectedObjectControlSet, SelectedAdmGraphError> {
        let admitted = self.admit_object_events(source_samples)?;
        let dbmd_tracks = admitted
            .events()
            .iter()
            .map(|event| {
                dbmd.fields_for_physical_host(u16::from(event.physical_host()))
                    .ok_or(SelectedAdmGraphError::DbmdTrackMissing)
            })
            .collect::<Result<Vec<_>, _>>()?;
        build_selected_object_control_set(&admitted.control_events(), &dbmd_tracks, &self.bed_join)
            .map_err(SelectedAdmGraphError::ObjectControl)
    }
}

fn build_object_sampler_owner_bindings(
    object_tracks: &[&SelectedPhysicalTrack],
) -> Result<[ObjectSamplerOwnerBinding; SELECTED_OBJECT_COUNT], SelectedAdmGraphError> {
    if object_tracks.len() != SELECTED_OBJECT_COUNT {
        return Err(SelectedAdmGraphError::SelectedTrackDomain);
    }

    let mut ordered_tracks = object_tracks.to_vec();
    ordered_tracks.sort_by(|left, right| left.uid.as_bytes().cmp(right.uid.as_bytes()));
    if ordered_tracks
        .windows(2)
        .any(|pair| pair[0].uid == pair[1].uid)
    {
        return Err(SelectedAdmGraphError::DuplicateUid);
    }

    let bindings = ordered_tracks
        .into_iter()
        .enumerate()
        .map(|(index, track)| {
            let SelectedTrackKind::Object { semantic_key } = track.kind else {
                return Err(SelectedAdmGraphError::SelectedTrackDomain);
            };
            let local_owner_ordinal = u8::try_from(index)
                .map(ObjectSamplerOwnerOrdinal)
                .map_err(|_| SelectedAdmGraphError::SelectedTrackDomain)?;
            Ok(ObjectSamplerOwnerBinding {
                track_uid: track.uid.clone(),
                local_owner_ordinal,
                semantic_key,
                physical_host: track.physical_host,
                channel_format_id: track.channel_format.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    bindings
        .try_into()
        .map_err(|_| SelectedAdmGraphError::SelectedTrackDomain)
}

fn find_object_sampler_owner_binding<'a>(
    bindings: &'a [ObjectSamplerOwnerBinding; SELECTED_OBJECT_COUNT],
    track_uid: &str,
) -> Option<&'a ObjectSamplerOwnerBinding> {
    let index = bindings
        .binary_search_by(|binding| binding.track_uid.as_bytes().cmp(track_uid.as_bytes()))
        .ok()?;
    bindings.get(index)
}

/// Parses the selected valid AXML/CHNA metadata chunks without reading a WAV/data chunk.
pub fn parse_selected_adm_graph(
    axml: &[u8],
    chna: &[u8],
) -> Result<SelectedAdmGraph, SelectedAdmGraphError> {
    if contains_forbidden_xml_declaration(axml) {
        return Err(SelectedAdmGraphError::ForbiddenXmlDeclaration);
    }
    let xml = std::str::from_utf8(axml).map_err(|_| SelectedAdmGraphError::XmlEncoding)?;
    let document = Document::parse(xml).map_err(|_| SelectedAdmGraphError::XmlParse)?;
    let root = document.root_element();
    if root.tag_name().name() != "ebuCoreMain" {
        return Err(SelectedAdmGraphError::XmlRoot);
    }
    let namespace = root.tag_name().namespace();
    let records = parse_chna(chna)?;
    let track_uid_nodes = index_nodes(&document, namespace, "audioTrackUID", "UID");
    let track_format_nodes = index_nodes(
        &document,
        namespace,
        "audioTrackFormat",
        "audioTrackFormatID",
    );
    let stream_format_nodes = index_nodes(
        &document,
        namespace,
        "audioStreamFormat",
        "audioStreamFormatID",
    );
    let channel_format_nodes = index_nodes(
        &document,
        namespace,
        "audioChannelFormat",
        "audioChannelFormatID",
    );
    let pack_format_nodes =
        index_nodes(&document, namespace, "audioPackFormat", "audioPackFormatID");

    if records.len() != SELECTED_TRACK_COUNT
        || records
            .iter()
            .enumerate()
            .any(|(slot, record)| usize::from(record.track_index.value()) != slot + 1)
    {
        return Err(SelectedAdmGraphError::SelectedTrackDomain);
    }

    let mut rows = Vec::with_capacity(SELECTED_TRACK_COUNT);
    let mut object_events = Vec::with_capacity(SELECTED_OBJECT_COUNT);
    for record in &records {
        let uid_node = lookup(&track_uid_nodes, &record.uid)?;
        let direct_track_format = first_child_text(uid_node, "audioTrackFormatIDRef", namespace)?;
        let direct_pack_format = first_child_text(uid_node, "audioPackFormatIDRef", namespace)?;
        if direct_track_format != record.track_format || direct_pack_format != record.pack_format {
            return Err(SelectedAdmGraphError::ChnaReferenceMismatch);
        }
        validate_optional_uid_profile(uid_node)?;

        let track_node = lookup(&track_format_nodes, &record.track_format)?;
        let stream_id = first_child_text(track_node, "audioStreamFormatIDRef", namespace)?;
        let stream_node = lookup(&stream_format_nodes, &stream_id)?;
        let channel_id = first_child_text(stream_node, "audioChannelFormatIDRef", namespace)?;
        let channel_node = lookup(&channel_format_nodes, &channel_id)?;
        let pack_node = lookup(&pack_format_nodes, &record.pack_format)?;
        let object_node = single_referencing_object(&document, namespace, &record.uid)?;
        let object_id = required_attribute(object_node, "audioObjectID")?;
        let object_semantic_key = parse_selected_object_semantic_key(&object_id)?;

        let channel_type = required_attribute(channel_node, "typeDefinition")?;
        let pack_type = required_attribute(pack_node, "typeDefinition")?;
        let physical_host = u8::try_from(record.track_index.value() - 1)
            .map_err(|_| SelectedAdmGraphError::SelectedTrackDomain)?;
        let (kind, descriptor_semantic_key) = match (channel_type.as_str(), pack_type.as_str()) {
            ("DirectSpeakers", "DirectSpeakers") => {
                let block = single_direct_block(channel_node, namespace)?;
                let speaker_label = single_speaker_label(block, namespace)?;
                let (role, bed_id) = route_bed_speaker_label(&speaker_label)?;
                let bed_pack_ordinal = u8::try_from(
                    rows.iter()
                        .filter(|row: &&SelectedPhysicalTrack| {
                            matches!(row.kind, SelectedTrackKind::DirectSpeakersBed { .. })
                        })
                        .count(),
                )
                .map_err(|_| SelectedAdmGraphError::BedDomain)?;
                let bed_input_id = object_semantic_key
                    .checked_add(u32::from(bed_pack_ordinal))
                    .ok_or(SelectedAdmGraphError::BedDomain)?;
                (
                    SelectedTrackKind::DirectSpeakersBed {
                        bed_id,
                        bed_pack_ordinal,
                        bed_input_id,
                        role,
                    },
                    bed_input_id,
                )
            }
            ("Objects", "Objects") => {
                let block = single_direct_block(channel_node, namespace)?;
                object_events.push(
                    RawAdmObjectEvent::from_xml(
                        physical_host,
                        record.uid.clone(),
                        object_id.clone(),
                        object_node,
                        block,
                        namespace,
                    )
                    .map_err(SelectedAdmGraphError::ObjectEvent)?,
                );
                (
                    SelectedTrackKind::Object {
                        semantic_key: object_semantic_key,
                    },
                    object_semantic_key,
                )
            }
            _ => return Err(SelectedAdmGraphError::SelectedTrackKind),
        };

        let references = direct_child_texts(object_node, "audioTrackUIDRef", namespace);
        if !references.iter().any(|reference| reference == &record.uid) {
            return Err(SelectedAdmGraphError::ObjectAssociation);
        }
        rows.push(SelectedPhysicalTrack {
            physical_host: PhysicalHostIndex(physical_host),
            track_index: record.track_index,
            uid: record.uid.clone(),
            track_format: record.track_format.clone(),
            stream_format: stream_id,
            channel_format: channel_id,
            pack_format: record.pack_format.clone(),
            object_id,
            descriptor_semantic_key,
            kind,
            block_count: 1,
            chna_reserved: record.reserved,
        });
    }

    let bed_rows = rows
        .iter()
        .filter_map(|row| match row.kind {
            SelectedTrackKind::DirectSpeakersBed { bed_id, role, .. } => Some((row, bed_id, role)),
            SelectedTrackKind::Object { .. } => None,
        })
        .collect::<Vec<_>>();
    let object_rows = rows
        .iter()
        .filter(|row| matches!(row.kind, SelectedTrackKind::Object { .. }))
        .collect::<Vec<_>>();
    if bed_rows.len() != SELECTED_BED_COUNT || object_rows.len() != SELECTED_OBJECT_COUNT {
        return Err(SelectedAdmGraphError::SelectedTrackDomain);
    }

    let master_object_id = bed_rows[0].0.object_id.as_str();
    if bed_rows
        .iter()
        .any(|(row, _, _)| row.object_id != master_object_id)
    {
        return Err(SelectedAdmGraphError::BedDomain);
    }
    let master_object = single_object_by_id(&document, namespace, master_object_id)?;
    let master_pack_id = first_child_text(master_object, "audioPackFormatIDRef", namespace)?;
    if bed_rows
        .iter()
        .any(|(row, _, _)| row.pack_format != master_pack_id)
    {
        return Err(SelectedAdmGraphError::BedDomain);
    }
    let master_pack = lookup(&pack_format_nodes, &master_pack_id)?;
    let pack_channel_ids = direct_child_texts(master_pack, "audioChannelFormatIDRef", namespace);
    if pack_channel_ids.len() != SELECTED_BED_COUNT {
        return Err(SelectedAdmGraphError::BedPackDomain);
    }
    let pack_channel_format_ids: [String; SELECTED_BED_COUNT] = pack_channel_ids
        .try_into()
        .map_err(|_| SelectedAdmGraphError::BedPackDomain)?;

    let physical_bed_tracks = bed_rows
        .iter()
        .map(|(row, bed_id, role)| {
            PhysicalBedTrack::new(
                row.channel_format.clone(),
                row.physical_host.value(),
                (*role).to_owned(),
                *bed_id,
            )
        })
        .collect::<Vec<_>>();
    let object_host_slots = object_rows
        .iter()
        .map(|row| row.physical_host.value())
        .collect::<Vec<_>>();
    let bed_join = join_selected_bed_channels(
        &pack_channel_format_ids,
        &physical_bed_tracks,
        &object_host_slots,
    )
    .map_err(SelectedAdmGraphError::BedChannelJoin)?;

    let direct_lfe_host = bed_join.lfe_host();
    let regular_hosts = bed_join.regular_control_order();
    if regular_hosts.len() != SELECTED_TRACK_COUNT - 1 {
        return Err(SelectedAdmGraphError::SourceBindingDomain);
    }
    let regular_source_bindings: [RegularSourceBinding; SELECTED_TRACK_COUNT - 1] =
        std::array::from_fn(|rank| RegularSourceBinding {
            physical_host: PhysicalHostIndex(regular_hosts[rank]),
            global_source: CodecSourceIndex::new(
                VIRTUAL_PREFIX_SOURCE_COUNT + u64::try_from(rank).unwrap_or(u64::MAX),
            ),
        });

    let uid_to_track_index = rows
        .iter()
        .map(|row| (row.uid.clone(), row.track_index))
        .collect::<BTreeMap<_, _>>();
    if uid_to_track_index.len() != SELECTED_TRACK_COUNT {
        return Err(SelectedAdmGraphError::DuplicateUid);
    }
    let object_sampler_owner_bindings = build_object_sampler_owner_bindings(&object_rows)?;
    let tracks: [SelectedPhysicalTrack; SELECTED_TRACK_COUNT] = rows
        .try_into()
        .map_err(|_| SelectedAdmGraphError::SelectedTrackDomain)?;
    let object_events: [RawAdmObjectEvent; SELECTED_OBJECT_COUNT] = object_events
        .try_into()
        .map_err(|_| SelectedAdmGraphError::SelectedTrackDomain)?;
    let receipt = SelectedAdmGraphReceipt {
        axml_sha256: Sha256::digest(axml).into(),
        chna_sha256: Sha256::digest(chna).into(),
        physical_track_count: u8::try_from(SELECTED_TRACK_COUNT).unwrap_or(u8::MAX),
        bed_track_count: u8::try_from(SELECTED_BED_COUNT).unwrap_or(u8::MAX),
        object_track_count: u8::try_from(SELECTED_OBJECT_COUNT).unwrap_or(u8::MAX),
        direct_lfe_host: PhysicalHostIndex(direct_lfe_host),
        virtual_prefix_source_count: u8::try_from(VIRTUAL_PREFIX_SOURCE_COUNT).unwrap_or(u8::MAX),
        total_source_count: u8::try_from(SELECTED_TOTAL_SOURCE_COUNT).unwrap_or(u8::MAX),
    };

    Ok(SelectedAdmGraph {
        tracks,
        object_events,
        object_sampler_owner_bindings,
        bed_pack_channel_format_ids: pack_channel_format_ids,
        bed_join,
        regular_source_bindings,
        uid_to_track_index,
        receipt,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedAdmGraphError {
    ForbiddenXmlDeclaration,
    XmlEncoding,
    XmlParse,
    XmlRoot,
    ChnaHeader,
    ChnaExtent,
    ChnaText,
    DuplicateUid,
    DuplicateTrackIndex,
    SelectedTrackDomain,
    ChnaReferenceMismatch,
    MissingReference,
    ObjectAssociation,
    ObjectIdentifier,
    SelectedTrackKind,
    BedDomain,
    BedPackDomain,
    BedLabel,
    LfeHost,
    SourceBindingDomain,
    ObjectEvent(SelectedAdmObjectEventError),
    DbmdTrackMissing,
    ObjectControl(ObjectControlError),
    BedChannelJoin(BedChannelJoinError),
}

impl SelectedAdmGraphError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ForbiddenXmlDeclaration => "ADM_XML_DECLARATION",
            Self::XmlEncoding => "ADM_XML_ENCODING",
            Self::XmlParse => "ADM_XML_PARSE",
            Self::XmlRoot => "ADM_XML_ROOT",
            Self::ChnaHeader => "ADM_CHNA_HEADER",
            Self::ChnaExtent => "ADM_CHNA_EXTENT",
            Self::ChnaText => "ADM_CHNA_TEXT",
            Self::DuplicateUid => "ADM_UID_DUPLICATE",
            Self::DuplicateTrackIndex => "ADM_TRACK_INDEX_DUPLICATE",
            Self::SelectedTrackDomain => "ADM_SELECTED_TRACK_DOMAIN",
            Self::ChnaReferenceMismatch => "ADM_CHNA_XML_REFERENCE",
            Self::MissingReference => "ADM_REFERENCE_MISSING",
            Self::ObjectAssociation => "ADM_OBJECT_ASSOCIATION",
            Self::ObjectIdentifier => "ADM_OBJECT_IDENTIFIER",
            Self::SelectedTrackKind => "ADM_SELECTED_TRACK_KIND",
            Self::BedDomain => "ADM_BED_DOMAIN",
            Self::BedPackDomain => "ADM_BED_PACK_DOMAIN",
            Self::BedLabel => "ADM_BED_LABEL",
            Self::LfeHost => "ADM_LFE_HOST",
            Self::SourceBindingDomain => "ADM_SOURCE_BINDING",
            Self::ObjectEvent(error) => error.code(),
            Self::DbmdTrackMissing => "ADM_DBMD_TRACK_MISSING",
            Self::ObjectControl(error) => error.code(),
            Self::BedChannelJoin(_) => "ADM_BED_JOIN",
        }
    }
}

struct ChnaRecord {
    track_index: ChnaTrackIndex,
    uid: String,
    track_format: String,
    pack_format: String,
    reserved: u8,
}

fn parse_chna(bytes: &[u8]) -> Result<Vec<ChnaRecord>, SelectedAdmGraphError> {
    if bytes.len() < CHNA_HEADER_BYTES {
        return Err(SelectedAdmGraphError::ChnaHeader);
    }
    let track_count = read_u16_le(bytes, 0)?;
    let uid_count = read_u16_le(bytes, 2)?;
    let expected_length = CHNA_HEADER_BYTES
        .checked_add(
            usize::from(uid_count)
                .checked_mul(CHNA_ENTRY_BYTES)
                .ok_or(SelectedAdmGraphError::ChnaExtent)?,
        )
        .ok_or(SelectedAdmGraphError::ChnaExtent)?;
    if bytes.len() != expected_length
        || usize::from(track_count) != SELECTED_TRACK_COUNT
        || usize::from(uid_count) != SELECTED_TRACK_COUNT
    {
        return Err(SelectedAdmGraphError::ChnaExtent);
    }
    let mut records = Vec::with_capacity(SELECTED_TRACK_COUNT);
    let mut seen_indices = BTreeSet::new();
    let mut seen_uids = BTreeSet::new();
    for ordinal in 0..usize::from(uid_count) {
        let start = CHNA_HEADER_BYTES + ordinal * CHNA_ENTRY_BYTES;
        let track_index = ChnaTrackIndex::new(read_u16_le(bytes, start)?);
        let uid_start = start + CHNA_TRACK_INDEX_BYTES;
        let track_format_start = uid_start + CHNA_UID_BYTES;
        let pack_format_start = track_format_start + CHNA_TRACK_FORMAT_BYTES;
        let reserved_offset = pack_format_start + CHNA_PACK_FORMAT_BYTES;
        let uid = fixed_ascii(&bytes[uid_start..track_format_start])?;
        let track_format = fixed_ascii(&bytes[track_format_start..pack_format_start])?;
        let pack_format = fixed_ascii(&bytes[pack_format_start..reserved_offset])?;
        if track_index.value() == 0
            || !seen_indices.insert(track_index.value())
            || uid.is_empty()
            || !seen_uids.insert(uid.clone())
            || track_format.is_empty()
            || pack_format.is_empty()
        {
            return Err(SelectedAdmGraphError::DuplicateTrackIndex);
        }
        records.push(ChnaRecord {
            track_index,
            uid,
            track_format,
            pack_format,
            reserved: bytes[reserved_offset],
        });
    }
    Ok(records)
}

fn read_u16_le(bytes: &[u8], offset: usize) -> Result<u16, SelectedAdmGraphError> {
    let end = offset
        .checked_add(2)
        .ok_or(SelectedAdmGraphError::ChnaHeader)?;
    let pair: [u8; 2] = bytes
        .get(offset..end)
        .ok_or(SelectedAdmGraphError::ChnaHeader)?
        .try_into()
        .map_err(|_| SelectedAdmGraphError::ChnaHeader)?;
    Ok(u16::from_le_bytes(pair))
}

fn fixed_ascii(bytes: &[u8]) -> Result<String, SelectedAdmGraphError> {
    let length = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    if bytes[..length].iter().any(|byte| !byte.is_ascii()) {
        return Err(SelectedAdmGraphError::ChnaText);
    }
    String::from_utf8(bytes[..length].to_vec()).map_err(|_| SelectedAdmGraphError::ChnaText)
}

fn contains_forbidden_xml_declaration(bytes: &[u8]) -> bool {
    let lowered = bytes.iter().map(u8::to_ascii_lowercase).collect::<Vec<_>>();
    [b"<!doctype".as_slice(), b"<!entity".as_slice()]
        .iter()
        .any(|needle| {
            lowered
                .windows(needle.len())
                .any(|window| window == *needle)
        })
}

fn index_nodes<'document, 'input>(
    document: &'document Document<'input>,
    namespace: Option<&str>,
    element_name: &str,
    id_attribute: &str,
) -> BTreeMap<String, Node<'document, 'input>> {
    let mut index = BTreeMap::new();
    for node in document.descendants().filter(|node| {
        node.is_element()
            && node.tag_name().namespace() == namespace
            && node.tag_name().name() == element_name
    }) {
        if let Some(identifier) = node
            .attribute(id_attribute)
            .filter(|value| !value.is_empty())
        {
            index.insert(identifier.to_owned(), node);
        }
    }
    index
}

fn lookup<'document, 'input>(
    index: &BTreeMap<String, Node<'document, 'input>>,
    identifier: &str,
) -> Result<Node<'document, 'input>, SelectedAdmGraphError> {
    index
        .get(identifier)
        .copied()
        .ok_or(SelectedAdmGraphError::MissingReference)
}

fn first_child_text(
    parent: Node<'_, '_>,
    name: &str,
    namespace: Option<&str>,
) -> Result<String, SelectedAdmGraphError> {
    parent
        .children()
        .find(|node| {
            node.is_element()
                && node.tag_name().namespace() == namespace
                && node.tag_name().name() == name
        })
        .and_then(|node| node.text())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
        .ok_or(SelectedAdmGraphError::MissingReference)
}

fn direct_child_texts(parent: Node<'_, '_>, name: &str, namespace: Option<&str>) -> Vec<String> {
    parent
        .children()
        .filter(|node| {
            node.is_element()
                && node.tag_name().namespace() == namespace
                && node.tag_name().name() == name
        })
        .filter_map(|node| node.text())
        .map(str::trim)
        .map(str::to_owned)
        .collect()
}

fn required_attribute(node: Node<'_, '_>, name: &str) -> Result<String, SelectedAdmGraphError> {
    node.attribute(name)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(SelectedAdmGraphError::MissingReference)
}

fn single_referencing_object<'document, 'input>(
    document: &'document Document<'input>,
    namespace: Option<&str>,
    uid: &str,
) -> Result<Node<'document, 'input>, SelectedAdmGraphError> {
    let matching = document
        .descendants()
        .filter(|node| {
            node.is_element()
                && node.tag_name().namespace() == namespace
                && node.tag_name().name() == "audioObject"
        })
        .filter(|object| {
            direct_child_texts(*object, "audioTrackUIDRef", namespace)
                .iter()
                .any(|reference| reference == uid)
        })
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return Err(SelectedAdmGraphError::ObjectAssociation);
    }
    matching
        .first()
        .copied()
        .ok_or(SelectedAdmGraphError::ObjectAssociation)
}

fn single_object_by_id<'document, 'input>(
    document: &'document Document<'input>,
    namespace: Option<&str>,
    object_id: &str,
) -> Result<Node<'document, 'input>, SelectedAdmGraphError> {
    let objects = document
        .descendants()
        .filter(|node| {
            node.is_element()
                && node.tag_name().namespace() == namespace
                && node.tag_name().name() == "audioObject"
                && node.attribute("audioObjectID") == Some(object_id)
        })
        .collect::<Vec<_>>();
    objects
        .last()
        .copied()
        .ok_or(SelectedAdmGraphError::ObjectAssociation)
}

fn validate_optional_uid_profile(uid: Node<'_, '_>) -> Result<(), SelectedAdmGraphError> {
    for (attribute, expected) in [("sampleRate", 48_000_u32), ("bitDepth", 24_u32)] {
        if let Some(value) = uid.attribute(attribute) {
            let parsed = value
                .parse::<u32>()
                .map_err(|_| SelectedAdmGraphError::SelectedTrackDomain)?;
            if parsed != expected {
                return Err(SelectedAdmGraphError::SelectedTrackDomain);
            }
        }
    }
    Ok(())
}

fn single_direct_block<'document, 'input>(
    channel: Node<'document, 'input>,
    namespace: Option<&str>,
) -> Result<Node<'document, 'input>, SelectedAdmGraphError> {
    let blocks = channel
        .children()
        .filter(|node| {
            node.is_element()
                && node.tag_name().namespace() == namespace
                && node.tag_name().name() == "audioBlockFormat"
        })
        .collect::<Vec<_>>();
    if blocks.len() != 1 {
        return Err(SelectedAdmGraphError::SelectedTrackDomain);
    }
    blocks
        .first()
        .copied()
        .ok_or(SelectedAdmGraphError::SelectedTrackDomain)
}

fn single_speaker_label(
    block: Node<'_, '_>,
    namespace: Option<&str>,
) -> Result<String, SelectedAdmGraphError> {
    let labels = block
        .descendants()
        .filter(|node| {
            node.is_element()
                && node.tag_name().namespace() == namespace
                && node.tag_name().name() == "speakerLabel"
        })
        .filter_map(|node| node.text())
        .map(str::trim)
        .collect::<Vec<_>>();
    if labels.len() != 1 || labels[0].is_empty() {
        return Err(SelectedAdmGraphError::BedLabel);
    }
    Ok(labels[0].to_owned())
}

fn route_bed_speaker_label(label: &str) -> Result<(&'static str, u8), SelectedAdmGraphError> {
    let suffix = label.chars().skip(3).collect::<String>();
    match suffix.as_str() {
        "L" => Ok(("front_left", 0)),
        "R" => Ok(("front_right", 1)),
        "C" => Ok(("front_center", 2)),
        "LFE" => Ok(("lfe", 3)),
        "Ls" | "Lss" => Ok(("side_left", 4)),
        "Rs" | "Rss" => Ok(("side_right", 5)),
        "Lrs" => Ok(("rear_left", 6)),
        "Rrs" => Ok(("rear_right", 7)),
        "Ltm" | "Lts" => Ok(("top_side_left", 8)),
        "Rtm" | "Rts" => Ok(("top_side_right", 9)),
        _ => Err(SelectedAdmGraphError::BedLabel),
    }
}

fn parse_selected_object_semantic_key(object_id: &str) -> Result<u32, SelectedAdmGraphError> {
    let digits = object_id
        .strip_prefix("AO_")
        .filter(|digits| digits.len() == 4 && digits.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or(SelectedAdmGraphError::ObjectIdentifier)?;
    let parsed =
        u32::from_str_radix(digits, 16).map_err(|_| SelectedAdmGraphError::ObjectIdentifier)?;
    parsed
        .checked_sub(0x1001)
        .filter(|_| parsed > 0x1000)
        .ok_or(SelectedAdmGraphError::ObjectIdentifier)
}

#[cfg(test)]
mod object_sampler_owner_binding_tests {
    use super::{
        ChnaTrackIndex, ObjectSamplerOwnerBinding, ObjectSamplerOwnerOrdinal, PhysicalHostIndex,
        SelectedAdmGraphError, SelectedPhysicalTrack, SelectedTrackKind,
        build_object_sampler_owner_bindings, find_object_sampler_owner_binding,
    };

    const INPUT_ONLY_VECTORS: &str = include_str!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/OBJECT-TRACK-SAMPLER-OWNER-BINDING-V1/INPUT-ONLY-VECTORS.json"
    );

    #[test]
    fn noncontinuous_keys_keep_their_identity_while_uid_sort_allocates_slots() {
        let tracks = base_object_tracks();
        let bindings = owner_bindings(&tracks);

        assert_associations(&tracks, &bindings);
        assert_eq!(bindings[0].track_uid(), "ATU_00000000");
        assert_eq!(bindings[0].local_owner_ordinal().value(), 0);
        assert_eq!(bindings[0].semantic_key(), 7);
        assert_eq!(bindings[0].physical_host().value(), 10);
        assert_eq!(bindings[0].channel_format_id(), "AC_OBJ_0000");
    }

    #[test]
    fn reversing_graph_order_does_not_change_uid_owner_bindings() {
        let tracks = base_object_tracks();
        let expected = owner_bindings(&tracks);
        let mut reversed = tracks.clone();
        reversed.reverse();

        let actual = owner_bindings(&reversed);

        assert_eq!(actual, expected);
        assert_associations(&reversed, &actual);
    }

    #[test]
    fn rotating_graph_order_does_not_change_uid_owner_bindings() {
        let tracks = base_object_tracks();
        let expected = owner_bindings(&tracks);
        let mut rotated = tracks.clone();
        rotated.rotate_left(11);

        let actual = owner_bindings(&rotated);

        assert_eq!(actual, expected);
        assert_associations(&rotated, &actual);
    }

    #[test]
    fn reversed_hosts_keep_owner_slots_and_host_invocation_uses_uid_joins() {
        let mut tracks = base_object_tracks();
        let original = owner_bindings(&tracks);
        for track in &mut tracks {
            let host = 47 - track.physical_host.value();
            track.physical_host = PhysicalHostIndex(host);
            track.track_index = ChnaTrackIndex(u16::from(host) + 1);
        }
        let bindings = owner_bindings(&tracks);
        assert_associations(&tracks, &bindings);

        for previous in &original {
            let current = find_object_sampler_owner_binding(&bindings, previous.track_uid())
                .expect("same TrackUID remains mapped");
            assert_eq!(
                current.local_owner_ordinal(),
                previous.local_owner_ordinal()
            );
            assert_eq!(current.semantic_key(), previous.semantic_key());
            assert_eq!(current.channel_format_id(), previous.channel_format_id());
            assert_eq!(
                current.physical_host().value(),
                47 - previous.physical_host().value()
            );
        }

        let mut host_order = tracks.iter().collect::<Vec<_>>();
        host_order.sort_by_key(|track| track.physical_host.value());
        assert!(
            host_order
                .windows(2)
                .all(|pair| { pair[0].physical_host.value() < pair[1].physical_host.value() })
        );
        for track in host_order {
            let binding = find_object_sampler_owner_binding(&bindings, &track.uid)
                .expect("host traversal joins the UID-sorted owner table");
            assert_eq!(binding.physical_host(), track.physical_host);
            assert_eq!(binding.channel_format_id(), track.channel_format);
        }
        assert_ne!(
            bindings
                .iter()
                .map(ObjectSamplerOwnerBinding::track_uid)
                .collect::<Vec<_>>(),
            tracks
                .iter()
                .map(|track| track.uid.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn shared_semantic_key_keeps_distinct_track_owner_slots() {
        let mut tracks = base_object_tracks();
        let shared_identity = tracks[0].clone();
        let SelectedTrackKind::Object { semantic_key } = shared_identity.kind else {
            panic!("test input contains only object tracks");
        };
        tracks[1].object_id = shared_identity.object_id.clone();
        tracks[1].descriptor_semantic_key = shared_identity.descriptor_semantic_key;
        tracks[1].kind = SelectedTrackKind::Object { semantic_key };
        let bindings = owner_bindings(&tracks);

        assert_associations(&tracks, &bindings);
        let first = find_object_sampler_owner_binding(&bindings, &tracks[0].uid).unwrap();
        let second = find_object_sampler_owner_binding(&bindings, &tracks[1].uid).unwrap();
        assert_ne!(first.track_uid(), second.track_uid());
        assert_eq!(first.semantic_key(), second.semantic_key());
        assert_ne!(first.local_owner_ordinal(), second.local_owner_ordinal());
        assert_ne!(first.channel_format_id(), second.channel_format_id());
    }

    #[test]
    fn existing_ascii_uid_values_use_unsigned_byte_lexical_order() {
        let mut tracks = base_object_tracks();
        for (track, uid) in tracks
            .iter_mut()
            .take(4)
            .zip(["UID_2", "UID_10", "UID_A", "UID_a"])
        {
            track.uid = uid.to_owned();
        }
        let bindings = owner_bindings(&tracks);
        assert_associations(&tracks, &bindings);

        let ordered_uids = bindings
            .iter()
            .map(ObjectSamplerOwnerBinding::track_uid)
            .collect::<Vec<_>>();
        assert_eq!(&ordered_uids[24..], &["UID_10", "UID_2", "UID_A", "UID_a"]);
    }

    #[test]
    fn uid_sort_uses_ordinary_prefix_length_order() {
        let mut tracks = base_object_tracks();
        tracks[0].uid = "UID_A".to_owned();
        tracks[1].uid = "UID_A0".to_owned();
        let bindings = owner_bindings(&tracks);

        let shorter = find_object_sampler_owner_binding(&bindings, "UID_A").unwrap();
        let longer = find_object_sampler_owner_binding(&bindings, "UID_A0").unwrap();
        assert!(shorter.local_owner_ordinal() < longer.local_owner_ordinal());
    }

    fn base_object_tracks() -> Vec<SelectedPhysicalTrack> {
        let input: serde_json::Value = serde_json::from_str(INPUT_ONLY_VECTORS).unwrap();
        input["base_object_tracks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                let track_uid = row["audio_track_uid"].as_str().unwrap().to_owned();
                let semantic_key = u32::try_from(row["semantic_key"].as_u64().unwrap()).unwrap();
                let host = u8::try_from(row["host_channel_index"].as_u64().unwrap()).unwrap();
                let channel_format = row["channel_format_id"].as_str().unwrap().to_owned();
                let object_id = row["audio_objects"][0]["id"].as_str().unwrap().to_owned();
                SelectedPhysicalTrack {
                    physical_host: PhysicalHostIndex(host),
                    track_index: ChnaTrackIndex(u16::from(host) + 1),
                    uid: track_uid,
                    track_format: "synthetic-track-format".to_owned(),
                    stream_format: "synthetic-stream-format".to_owned(),
                    channel_format,
                    pack_format: "synthetic-objects-pack".to_owned(),
                    object_id,
                    descriptor_semantic_key: semantic_key,
                    kind: SelectedTrackKind::Object { semantic_key },
                    block_count: 1,
                    chna_reserved: 0,
                }
            })
            .collect()
    }

    fn owner_bindings(tracks: &[SelectedPhysicalTrack]) -> [ObjectSamplerOwnerBinding; 28] {
        let object_tracks = tracks.iter().collect::<Vec<_>>();
        build_object_sampler_owner_bindings(&object_tracks).unwrap()
    }

    fn assert_associations(
        tracks: &[SelectedPhysicalTrack],
        bindings: &[ObjectSamplerOwnerBinding; 28],
    ) {
        assert_eq!(bindings.len(), 28);
        for (index, binding) in bindings.iter().enumerate() {
            assert_eq!(
                binding.local_owner_ordinal(),
                ObjectSamplerOwnerOrdinal(u8::try_from(index).unwrap())
            );
            let track = tracks
                .iter()
                .find(|track| track.uid == binding.track_uid())
                .unwrap();
            let SelectedTrackKind::Object { semantic_key } = track.kind else {
                panic!("test input contains only object tracks");
            };
            assert_eq!(binding.semantic_key(), semantic_key);
            assert_eq!(binding.physical_host(), track.physical_host);
            assert_eq!(binding.channel_format_id(), track.channel_format);
        }
    }

    #[test]
    fn owner_binding_rejects_duplicate_track_uids() {
        let mut tracks = base_object_tracks();
        let duplicate_uid = tracks[0].uid.clone();
        tracks[1].uid = duplicate_uid;
        let object_tracks = tracks.iter().collect::<Vec<_>>();

        assert_eq!(
            build_object_sampler_owner_bindings(&object_tracks),
            Err(SelectedAdmGraphError::DuplicateUid)
        );
    }
}
