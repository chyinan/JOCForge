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
const SELECTED_LFE_HOST: u8 = 3;

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
    if direct_lfe_host != SELECTED_LFE_HOST {
        return Err(SelectedAdmGraphError::LfeHost);
    }
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
