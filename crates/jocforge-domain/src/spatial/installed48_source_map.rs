// pattern: Functional Core
//! Primary-source 11-prefix and 37-physical-host installed48 source map.

use crate::frontend::dbmd_sections::DbmdTrackFields;
use crate::frontend::job_configuration::{
    PrimaryMetadataChunkHashes, SelectedPrimaryControlBinding,
};
use crate::frontend::object_control::ObjectControl52;
use crate::frontend::primary_metadata_control::{
    PrimaryControl38Fields, PrimaryMetadataControlRows,
};
use crate::frontend::selected_adm_graph::{
    PhysicalHostIndex, SelectedPhysicalTrack, SelectedTrackKind,
};
use crate::spatial::bed_source_points::selected_bed_source_points;
use crate::spatial::prefix_membership::{
    PrefixMembershipError, PrefixMembershipProducer, PrefixMembershipResult,
};
use crate::spatial::selected_p11_geometry::{P11PrefixOrdinal, SelectedP11Geometry};
use crate::timeline::CodecSourceIndex;

const SOURCE_COUNT: usize = 48;
const PREFIX_COUNT: usize = 11;
const REGULAR_COUNT: usize = 37;
const PHYSICAL_HOST_COUNT: usize = 38;
const DIRECT_LFE_HOST: u8 = 3;
const SELECTED_SAMPLE_COUNT: u64 = 6_522_906;
const SELECTED_SOURCE_BYTES: u64 = 743_696_526;
const SELECTED_DATA_BYTES: u64 = 743_611_284;
const SELECTED_BLOCK_ALIGN: u64 = 114;
const SELECTED_JOB_PROJECTION_SHA256: &str =
    "b710150f1b8357ca417d3fe705c88334b9c9bbd418656b458e49f8ffa3faba82";
const SELECTED_SOURCE_RECEIPT_SHA256: &str =
    "c60c45033ee40592b9fc82c9c68edf94c4ee1676f27d9df34634291b2cb0f7ad";
const SELECTED_AXML_SHA256: &str =
    "7e4e61d4e2bec581ff2b05133281198a220602290597950e301a6079701c1659";
const SELECTED_CHNA_SHA256: &str =
    "f82b14e4c1b5b3b1750ed4104aa53f1d595e3ff2d4dbaef76a76c0e35f31b7a6";
const SELECTED_DBMD_SHA256: &str =
    "ea961b79063bc68f64b1210dba002a5729d1003427196123f93b0890939b75d9";
const SELECTED_WIDE_OBJECT_THRESHOLD: f32 = f32::from_bits(0x3e4c_cccd);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Installed48SourceMapError {
    SourceBinding,
    SourceExtent,
    RowCount,
    PhysicalHostOrder,
    DirectLfeIdentity,
    BedIdentity,
    ControlRowBoundary,
    ControlFieldDomain,
    SourceIndexDomain,
    Membership(PrefixMembershipError),
}

impl Installed48SourceMapError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::SourceBinding => "INSTALLED48_SOURCE_BINDING",
            Self::SourceExtent => "INSTALLED48_SOURCE_EXTENT",
            Self::RowCount => "INSTALLED48_ROW_COUNT",
            Self::PhysicalHostOrder => "INSTALLED48_PHYSICAL_HOST_ORDER",
            Self::DirectLfeIdentity => "INSTALLED48_DIRECT_LFE_IDENTITY",
            Self::BedIdentity => "INSTALLED48_BED_IDENTITY",
            Self::ControlRowBoundary => "INSTALLED48_CONTROL38_BOUNDARY",
            Self::ControlFieldDomain => "INSTALLED48_CONTROL_FIELD_DOMAIN",
            Self::SourceIndexDomain => "INSTALLED48_SOURCE_INDEX_DOMAIN",
            Self::Membership(error) => error.code(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Installed48SourceOwner {
    P11 {
        layout_ordinal: P11PrefixOrdinal,
        source_layout_index: u8,
    },
    PhysicalTrack {
        track: SelectedPhysicalTrack,
        dbmd: DbmdTrackFields,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Installed48SourceRow {
    source_index: CodecSourceIndex,
    kind: &'static str,
    owner: Installed48SourceOwner,
    metadata_words_8_to_55: [u32; 12],
    input_mute: Option<u8>,
    control52: Option<ObjectControl52>,
    packed_attribute: u32,
}

impl Installed48SourceRow {
    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn kind(&self) -> &'static str {
        self.kind
    }

    pub const fn owner(&self) -> &Installed48SourceOwner {
        &self.owner
    }

    pub const fn metadata_words_8_to_55(&self) -> &[u32; 12] {
        &self.metadata_words_8_to_55
    }

    pub const fn input_mute(&self) -> Option<u8> {
        self.input_mute
    }

    pub const fn control52(&self) -> Option<ObjectControl52> {
        self.control52
    }

    pub const fn packed_attribute(&self) -> u32 {
        self.packed_attribute
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalSourceBinding {
    physical_host: PhysicalHostIndex,
    source_index: CodecSourceIndex,
}

#[derive(Debug, PartialEq)]
pub struct PrimaryPrefixMembershipContribution {
    source_index: CodecSourceIndex,
    result: PrefixMembershipResult,
}

impl PrimaryPrefixMembershipContribution {
    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub fn result(&self) -> &PrefixMembershipResult {
        &self.result
    }
}

#[derive(Debug, PartialEq)]
pub struct PrimaryPrefixRoute {
    writers: PrimaryPrefixWriterSet,
    contributions: Vec<PrimaryPrefixMembershipContribution>,
}

impl PrimaryPrefixRoute {
    pub const fn writers(&self) -> PrimaryPrefixWriterSet {
        self.writers
    }

    pub fn writer_indices(&self) -> Vec<u8> {
        self.writers.written_ordinals()
    }

    pub fn contributions(&self) -> &[PrimaryPrefixMembershipContribution] {
        &self.contributions
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrimaryPrefixWriterSet {
    written: [bool; PREFIX_COUNT],
}

impl PrimaryPrefixWriterSet {
    pub const fn empty() -> Self {
        Self {
            written: [false; PREFIX_COUNT],
        }
    }

    pub fn from_positive_members(members: &[u8]) -> Result<Self, Installed48SourceMapError> {
        let mut written = [false; PREFIX_COUNT];
        for member in members {
            let slot = usize::from(*member);
            let Some(destination) = written.get_mut(slot) else {
                return Err(Installed48SourceMapError::SourceIndexDomain);
            };
            if std::mem::replace(destination, true) {
                return Err(Installed48SourceMapError::SourceIndexDomain);
            }
        }
        Ok(Self { written })
    }

    pub const fn contains(self, ordinal: P11PrefixOrdinal) -> bool {
        self.written[ordinal.value() as usize]
    }

    pub fn written_ordinals(self) -> Vec<u8> {
        self.written
            .iter()
            .enumerate()
            .filter_map(|(index, written)| written.then(|| u8::try_from(index).ok()).flatten())
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Installed48PostRouteRow {
    source_index: CodecSourceIndex,
    field0: u32,
    field4: u32,
    metadata_words_8_to_55: [u32; 12],
    packed_attribute: u32,
}

impl Installed48PostRouteRow {
    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn field0(&self) -> u32 {
        self.field0
    }

    pub const fn field4(&self) -> u32 {
        self.field4
    }

    pub const fn metadata_words_8_to_55(&self) -> &[u32; 12] {
        &self.metadata_words_8_to_55
    }

    pub const fn packed_attribute(&self) -> u32 {
        self.packed_attribute
    }

    pub fn defined_bytes(&self) -> [u8; 56] {
        let mut bytes = [0_u8; 56];
        bytes[..4].copy_from_slice(&self.field0.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.field4.to_le_bytes());
        for (index, word) in self.metadata_words_8_to_55.iter().enumerate() {
            let start = 8 + index * 4;
            bytes[start..start + 4].copy_from_slice(&word.to_le_bytes());
        }
        bytes
    }
}

impl PhysicalSourceBinding {
    pub const fn physical_host(self) -> PhysicalHostIndex {
        self.physical_host
    }

    pub const fn source_index(self) -> CodecSourceIndex {
        self.source_index
    }
}

/// Immutable source map built from the bound primary metadata/control rows and P11 geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct Installed48SourceMap {
    source_binding: SelectedPrimaryControlBinding,
    p11_geometry: SelectedP11Geometry,
    rows: [Installed48SourceRow; SOURCE_COUNT],
    physical_to_global_source: [PhysicalSourceBinding; REGULAR_COUNT],
}

impl Installed48SourceMap {
    pub fn source_binding(&self) -> &SelectedPrimaryControlBinding {
        &self.source_binding
    }

    pub const fn p11_geometry(&self) -> &SelectedP11Geometry {
        &self.p11_geometry
    }

    pub fn rows(&self) -> &[Installed48SourceRow; SOURCE_COUNT] {
        &self.rows
    }

    pub const fn prefix_count(&self) -> u8 {
        11
    }

    pub const fn regular_count(&self) -> u8 {
        37
    }

    pub fn physical_to_global_source(&self) -> &[PhysicalSourceBinding; REGULAR_COUNT] {
        &self.physical_to_global_source
    }

    pub fn row_for_source(&self, source_index: CodecSourceIndex) -> Option<&Installed48SourceRow> {
        self.rows
            .iter()
            .find(|row| row.source_index == source_index)
    }

    pub fn source_for_physical_host(
        &self,
        physical_host: PhysicalHostIndex,
    ) -> Option<CodecSourceIndex> {
        self.physical_to_global_source
            .iter()
            .find(|binding| binding.physical_host == physical_host)
            .map(|binding| binding.source_index)
    }

    /// Recomputes full gains for each active positive-width object in physical-host order.
    pub fn produce_primary_prefix_route(
        &self,
        quiet37: &[u8; REGULAR_COUNT],
        producer: &PrefixMembershipProducer,
    ) -> Result<PrimaryPrefixRoute, Installed48SourceMapError> {
        if producer.basis_f32le_sha256() != self.p11_geometry.f32le_sha256() {
            return Err(Installed48SourceMapError::Membership(
                PrefixMembershipError::ParameterDomain,
            ));
        }
        if quiet37.iter().any(|quiet| *quiet > 1) {
            return Err(Installed48SourceMapError::ControlFieldDomain);
        }
        let mut writers = PrimaryPrefixWriterSet::empty();
        let mut contributions = Vec::new();
        for row in &self.rows[PREFIX_COUNT..] {
            let source_value = usize::try_from(row.source_index().value())
                .map_err(|_| Installed48SourceMapError::SourceIndexDomain)?;
            let regular_ordinal = source_value
                .checked_sub(PREFIX_COUNT)
                .filter(|ordinal| *ordinal < REGULAR_COUNT)
                .ok_or(Installed48SourceMapError::SourceIndexDomain)?;
            if quiet37[regular_ordinal] != 0 || row.kind() != "object" {
                continue;
            }
            if row.input_mute() != Some(0) {
                return Err(Installed48SourceMapError::ControlFieldDomain);
            }
            let control = row
                .control52()
                .ok_or(Installed48SourceMapError::ControlFieldDomain)?;
            let words = control.words();
            let raw_width = f32::from_bits(words[9]);
            if !raw_width.is_finite() || raw_width <= 0.0 {
                continue;
            }
            let record = membership_record(words);
            let result = producer
                .produce(&record)
                .map_err(Installed48SourceMapError::Membership)?;
            for member in result.positive_members() {
                let slot = writers
                    .written
                    .get_mut(usize::from(*member))
                    .ok_or(Installed48SourceMapError::SourceIndexDomain)?;
                *slot = true;
            }
            contributions.push(PrimaryPrefixMembershipContribution {
                source_index: row.source_index(),
                result,
            });
        }
        Ok(PrimaryPrefixRoute {
            writers,
            contributions,
        })
    }

    /// Applies the specified post-route field transitions to a computed quiet vector and
    /// prefix-writer set. This does not generate membership gains or publish state.
    pub fn postroute_rows(
        &self,
        quiet37: &[u8; REGULAR_COUNT],
        prefix_writers: PrimaryPrefixWriterSet,
    ) -> Result<[Installed48PostRouteRow; SOURCE_COUNT], Installed48SourceMapError> {
        if quiet37.iter().any(|quiet| *quiet > 1) {
            return Err(Installed48SourceMapError::ControlFieldDomain);
        }
        let mut output = Vec::with_capacity(SOURCE_COUNT);
        for row in &self.rows {
            match row.owner() {
                Installed48SourceOwner::P11 { layout_ordinal, .. } => {
                    let written = prefix_writers.contains(*layout_ordinal);
                    output.push(Installed48PostRouteRow {
                        source_index: row.source_index(),
                        field0: u32::from(!written),
                        field4: u32::from(!written),
                        metadata_words_8_to_55: *row.metadata_words_8_to_55(),
                        packed_attribute: row.packed_attribute(),
                    });
                }
                Installed48SourceOwner::PhysicalTrack { .. } => {
                    let source = usize::try_from(row.source_index().value())
                        .map_err(|_| Installed48SourceMapError::SourceIndexDomain)?;
                    let regular_ordinal = source
                        .checked_sub(PREFIX_COUNT)
                        .filter(|ordinal| *ordinal < REGULAR_COUNT)
                        .ok_or(Installed48SourceMapError::SourceIndexDomain)?;
                    let quiet = quiet37[regular_ordinal];
                    let mute = row
                        .input_mute()
                        .ok_or(Installed48SourceMapError::ControlFieldDomain)?;
                    let mut field0 = u32::from(quiet);
                    let mut field4 = u32::from(mute);
                    let mut metadata = *row.metadata_words_8_to_55();
                    let pre_route_width = f32::from_bits(metadata[8]);
                    if !pre_route_width.is_finite() || mute > 1 {
                        return Err(Installed48SourceMapError::ControlFieldDomain);
                    }
                    if quiet == 0 && pre_route_width > 0.0 {
                        metadata[7] = 0;
                        metadata[8] = 0;
                        metadata[9] = 0;
                        if pre_route_width >= SELECTED_WIDE_OBJECT_THRESHOLD {
                            field0 = 1;
                            field4 = 1;
                            metadata[6] = 0;
                            metadata[10] = 0;
                            metadata[11] = 0;
                        }
                    } else {
                        metadata[8] = 0;
                        if metadata[10] == 0 && metadata[11] == 0 {
                            metadata[6] = 0;
                        }
                    }
                    output.push(Installed48PostRouteRow {
                        source_index: row.source_index(),
                        field0,
                        field4,
                        packed_attribute: packed_attribute(&metadata)?,
                        metadata_words_8_to_55: metadata,
                    });
                }
            }
        }
        output
            .try_into()
            .map_err(|_| Installed48SourceMapError::RowCount)
    }
}

pub fn field_masks_hex(
    rows: &[Installed48PostRouteRow; SOURCE_COUNT],
) -> Result<(String, String), Installed48SourceMapError> {
    let mut field0_mask = 0_u64;
    let mut field4_mask = 0_u64;
    for row in rows {
        let source_index = row.source_index().value();
        if source_index >= SOURCE_COUNT as u64 || row.field0() > 1 || row.field4() > 1 {
            return Err(Installed48SourceMapError::SourceIndexDomain);
        }
        let bit = 1_u64 << source_index;
        field0_mask |= bit * u64::from(row.field0());
        field4_mask |= bit * u64::from(row.field4());
    }
    Ok((format!("{field0_mask:012x}"), format!("{field4_mask:012x}")))
}

/// Builds the selected 48-source map. The direct LFE remains outside regular sources.
pub fn build_primary_installed48_source_map(
    controls: &PrimaryMetadataControlRows,
    geometry: &SelectedP11Geometry,
) -> Result<Installed48SourceMap, Installed48SourceMapError> {
    validate_primary_binding(controls.source_binding())?;
    if controls.row_count() != PHYSICAL_HOST_COUNT {
        return Err(Installed48SourceMapError::RowCount);
    }

    let mut rows = Vec::with_capacity(SOURCE_COUNT);
    for point in geometry.points() {
        let metadata = prefix_metadata(point.xyz_words());
        rows.push(Installed48SourceRow {
            source_index: CodecSourceIndex::new(u64::from(point.ordinal().value())),
            kind: "virtual_prefix",
            owner: Installed48SourceOwner::P11 {
                layout_ordinal: point.ordinal(),
                source_layout_index: point.source_layout_index(),
            },
            packed_attribute: packed_attribute(&metadata)?,
            metadata_words_8_to_55: metadata,
            input_mute: None,
            control52: None,
        });
    }

    let bed_points = selected_bed_source_points();
    let mut physical_rows = controls.rows().iter().collect::<Vec<_>>();
    physical_rows.sort_by_key(|row| row.track().physical_host().value());
    if physical_rows.len() != PHYSICAL_HOST_COUNT
        || physical_rows
            .iter()
            .enumerate()
            .any(|(expected, row)| usize::from(row.track().physical_host().value()) != expected)
    {
        return Err(Installed48SourceMapError::PhysicalHostOrder);
    }
    let mut lfe_count = 0;
    let mut regular_ordinal = 0_u8;
    let mut physical_bindings = Vec::with_capacity(REGULAR_COUNT);
    for row in physical_rows {
        let host = row.track().physical_host();
        let is_lfe = matches!(
            row.track().kind(),
            SelectedTrackKind::DirectSpeakersBed { bed_id: 3, .. }
        );
        if is_lfe {
            lfe_count += 1;
            if host.value() != DIRECT_LFE_HOST {
                return Err(Installed48SourceMapError::DirectLfeIdentity);
            }
            continue;
        }

        let source_value = PREFIX_COUNT
            .checked_add(usize::from(regular_ordinal))
            .ok_or(Installed48SourceMapError::SourceIndexDomain)?;
        let source_index = CodecSourceIndex::new(
            u64::try_from(source_value)
                .map_err(|_| Installed48SourceMapError::SourceIndexDomain)?,
        );
        let (kind, metadata, control52) = match (row.track().kind(), row.fields()) {
            (
                SelectedTrackKind::DirectSpeakersBed { bed_id, .. },
                PrimaryControl38Fields::Bed {
                    unproduced_tail, ..
                },
            ) => {
                if unproduced_tail.offset() != 24 || unproduced_tail.byte_count() != 28 {
                    return Err(Installed48SourceMapError::ControlRowBoundary);
                }
                let point = bed_points
                    .iter()
                    .find(|point| point.bed_id() == bed_id)
                    .ok_or(Installed48SourceMapError::BedIdentity)?;
                ("bed", bed_metadata(point.xyz_words()), None)
            }
            (SelectedTrackKind::Object { .. }, PrimaryControl38Fields::Object { control52 }) => {
                ("object", object_metadata(control52)?, Some(control52))
            }
            _ => return Err(Installed48SourceMapError::ControlFieldDomain),
        };
        let packed = packed_attribute(&metadata)?;
        rows.push(Installed48SourceRow {
            source_index,
            kind,
            owner: Installed48SourceOwner::PhysicalTrack {
                track: row.track().clone(),
                dbmd: row.dbmd(),
            },
            metadata_words_8_to_55: metadata,
            input_mute: Some(0),
            control52,
            packed_attribute: packed,
        });
        physical_bindings.push(PhysicalSourceBinding {
            physical_host: host,
            source_index,
        });
        regular_ordinal = regular_ordinal
            .checked_add(1)
            .ok_or(Installed48SourceMapError::SourceIndexDomain)?;
    }
    if lfe_count != 1 || usize::from(regular_ordinal) != REGULAR_COUNT {
        return Err(Installed48SourceMapError::DirectLfeIdentity);
    }
    if rows.len() != SOURCE_COUNT {
        return Err(Installed48SourceMapError::RowCount);
    }
    let rows = rows
        .try_into()
        .map_err(|_| Installed48SourceMapError::RowCount)?;
    let physical_to_global_source = physical_bindings
        .try_into()
        .map_err(|_| Installed48SourceMapError::RowCount)?;
    Ok(Installed48SourceMap {
        source_binding: controls.source_binding().clone(),
        p11_geometry: geometry.clone(),
        rows,
        physical_to_global_source,
    })
}

fn membership_record(control_words: [u32; 13]) -> [u8; 32] {
    let semantic_words = [
        control_words[5],
        control_words[6],
        control_words[7],
        control_words[4],
        control_words[9],
        control_words[8],
        control_words[11],
        control_words[12],
    ];
    let mut bytes = [0_u8; 32];
    for (index, word) in semantic_words.iter().enumerate() {
        let offset = index * 4;
        bytes[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
    }
    bytes
}

fn validate_primary_binding(
    binding: &SelectedPrimaryControlBinding,
) -> Result<(), Installed48SourceMapError> {
    let hashes: PrimaryMetadataChunkHashes = binding.metadata_hashes();
    let sample_count = binding.data_bytes() / SELECTED_BLOCK_ALIGN;
    if binding.job_projection_sha256() != &decode_hash(SELECTED_JOB_PROJECTION_SHA256)?
        || binding.source_receipt_sha256() != &decode_hash(SELECTED_SOURCE_RECEIPT_SHA256)?
        || hashes.axml() != decode_hash(SELECTED_AXML_SHA256)?
        || hashes.chna() != decode_hash(SELECTED_CHNA_SHA256)?
        || hashes.dbmd() != decode_hash(SELECTED_DBMD_SHA256)?
    {
        return Err(Installed48SourceMapError::SourceBinding);
    }
    if binding.data_bytes() != SELECTED_DATA_BYTES
        || binding.source_file_bytes() != SELECTED_SOURCE_BYTES
        || sample_count != SELECTED_SAMPLE_COUNT
        || binding.source_samples().value() != SELECTED_SAMPLE_COUNT
        || binding.wave().channels() != 38
        || binding.wave().sample_rate_hz() != 48_000
        || binding.wave().pcm_bits() != 24
    {
        return Err(Installed48SourceMapError::SourceExtent);
    }
    Ok(())
}

fn object_metadata(control52: ObjectControl52) -> Result<[u32; 12], Installed48SourceMapError> {
    let words = control52.words();
    if words[1] != 0
        || words[2] != 3
        || words[3] != 2
        || words[4] != 0
        || words[8] != 0
        || words[11] != 0
        || words[12] != 1
    {
        return Err(Installed48SourceMapError::ControlFieldDomain);
    }
    words[1..]
        .try_into()
        .map_err(|_| Installed48SourceMapError::ControlFieldDomain)
}

fn prefix_metadata(xyz_words: [u32; 3]) -> [u32; 12] {
    [
        0,
        3,
        2,
        0,
        xyz_words[0],
        xyz_words[1],
        xyz_words[2],
        0,
        0,
        0,
        0,
        1,
    ]
}

fn bed_metadata(xyz_words: [u32; 3]) -> [u32; 12] {
    prefix_metadata(xyz_words)
}

fn packed_attribute(metadata: &[u32; 12]) -> Result<u32, Installed48SourceMapError> {
    if metadata[0] != 0 || metadata[1] != 3 || metadata[2] != 2 {
        return Err(Installed48SourceMapError::ControlFieldDomain);
    }
    Ok(metadata[7]
        | (metadata[10] << 1)
        | (metadata[0] << 4)
        | (metadata[1] << 8)
        | (metadata[2] << 10))
}

fn decode_hash(value: &str) -> Result<[u8; 32], Installed48SourceMapError> {
    if value.len() != 64 {
        return Err(Installed48SourceMapError::SourceBinding);
    }
    let mut digest = [0_u8; 32];
    for (index, byte) in digest.iter_mut().enumerate() {
        let offset = index * 2;
        *byte = u8::from_str_radix(&value[offset..offset + 2], 16)
            .map_err(|_| Installed48SourceMapError::SourceBinding)?;
    }
    Ok(digest)
}
