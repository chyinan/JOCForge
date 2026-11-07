// pattern: Functional Core
//! Dynamic selected-profile physical-row and F03W source-axis mapping.

use crate::frontend::dbmd_sections::DbmdTrackFields;
use crate::frontend::job_configuration::{
    PrimaryMetadataChunkHashes, SelectedInputKind, SelectedPrimaryControlBinding,
};
use crate::frontend::normal_control_snapshot::{
    NormalControlAdmHeldSourceView, NormalControlAdmInputOutput, NormalControlAdmInputSnapshot,
    NormalControlPhysicalTrack, NormalControlPreparedSnapshot, NormalControlRegularSourceBinding,
    NormalControlRow, NormalControlTrackKind,
};
use crate::frontend::object_control::ObjectControl52;
use crate::frontend::primary_metadata_control::{
    PrimaryControl38Fields, PrimaryMetadataControlRows,
};
use crate::frontend::selected_adm_graph::{
    PhysicalHostIndex, SelectedPhysicalTrack, SelectedTrackKind,
};
use crate::spatial::bed_source_points::selected_bed_source_points;
use crate::spatial::installed48_render::Installed48AudioControl;
use crate::spatial::prefix_membership::{
    PrefixMembershipError, PrefixMembershipProducer, PrefixMembershipResult,
};
use crate::spatial::selected_p11_geometry::{P11PrefixOrdinal, SelectedP11Geometry};
use crate::{
    source_axis::{
        F03W_PREFIX_SOURCE_COUNT, SELECTED_PROFILE_BED_ROW_COUNT, SelectedSourceRowDimensions,
        SelectedSourceRowShapeError,
    },
    timeline::CodecSourceIndex,
};

const PREFIX_COUNT: usize = F03W_PREFIX_SOURCE_COUNT as usize;
const MIN_PROFILE_PHYSICAL_HOST_COUNT: usize = SELECTED_PROFILE_BED_ROW_COUNT as usize;
const MAX_PROFILE_PHYSICAL_HOST_COUNT: usize = u8::MAX as usize + 1;
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
    SourceAxisCapacityExceeded,
    PhysicalHostOrder,
    DirectLfeIdentity,
    NormalControlIdentityMismatch,
    AudioControlDomain,
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
            Self::SourceAxisCapacityExceeded => "INSTALLED48_SOURCE_AXIS_CAPACITY",
            Self::PhysicalHostOrder => "INSTALLED48_PHYSICAL_HOST_ORDER",
            Self::DirectLfeIdentity => "INSTALLED48_DIRECT_LFE_IDENTITY",
            Self::NormalControlIdentityMismatch => "INSTALLED48_NORMAL_CONTROL_IDENTITY",
            Self::AudioControlDomain => "INSTALLED48_AUDIO_CONTROL_DOMAIN",
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
    point_response_source_fields: [u32; 4],
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

    /// `[feature_mode, source_skip, spatial_enum, coordinate_parameter]` from the installed row.
    pub const fn point_response_source_fields(&self) -> [u32; 4] {
        self.point_response_source_fields
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

/// Immutable N+10 source map built from bound primary metadata/control rows and P11 geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct Installed48SourceMap {
    source_binding: SelectedPrimaryControlBinding,
    p11_geometry: SelectedP11Geometry,
    direct_lfe_track: SelectedPhysicalTrack,
    physical_host_count: u16,
    rows: Vec<Installed48SourceRow>,
    physical_to_global_source: Vec<PhysicalSourceBinding>,
}

impl Installed48SourceMap {
    pub fn source_binding(&self) -> &SelectedPrimaryControlBinding {
        &self.source_binding
    }

    pub const fn p11_geometry(&self) -> &SelectedP11Geometry {
        &self.p11_geometry
    }

    pub const fn direct_lfe_host(&self) -> PhysicalHostIndex {
        self.direct_lfe_track.physical_host()
    }

    pub fn rows(&self) -> &[Installed48SourceRow] {
        &self.rows
    }

    pub const fn prefix_count(&self) -> u8 {
        11
    }

    pub fn regular_count(&self) -> u8 {
        u8::try_from(self.physical_to_global_source.len()).unwrap_or(u8::MAX)
    }

    pub const fn physical_host_count(&self) -> u16 {
        self.physical_host_count
    }

    pub fn source_count(&self) -> u16 {
        u16::try_from(self.rows.len()).unwrap_or(u16::MAX)
    }

    pub fn physical_to_global_source(&self) -> &[PhysicalSourceBinding] {
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
        quiet_regular_rows: &[u8],
        producer: &PrefixMembershipProducer,
    ) -> Result<PrimaryPrefixRoute, Installed48SourceMapError> {
        if producer.basis_f32le_sha256() != self.p11_geometry.f32le_sha256() {
            return Err(Installed48SourceMapError::Membership(
                PrefixMembershipError::ParameterDomain,
            ));
        }
        if quiet_regular_rows.len() != self.physical_to_global_source.len()
            || quiet_regular_rows.iter().any(|quiet| *quiet > 1)
        {
            return Err(Installed48SourceMapError::ControlFieldDomain);
        }
        let mut writers = PrimaryPrefixWriterSet::empty();
        let mut contributions = Vec::new();
        for row in &self.rows[PREFIX_COUNT..] {
            let source_value = usize::try_from(row.source_index().value())
                .map_err(|_| Installed48SourceMapError::SourceIndexDomain)?;
            let regular_ordinal = source_value
                .checked_sub(PREFIX_COUNT)
                .filter(|ordinal| *ordinal < self.physical_to_global_source.len())
                .ok_or(Installed48SourceMapError::SourceIndexDomain)?;
            if quiet_regular_rows[regular_ordinal] != 0 || row.kind() != "object" {
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

    /// Produces current-callback prefix gains from the exact F01E control snapshot.
    /// Static source-map rows provide identity and routing only; dynamic controls come from F01E.
    pub fn produce_primary_prefix_route_from_normal_controls(
        &self,
        quiet_regular_rows: &[u8],
        prepared: &NormalControlPreparedSnapshot,
        control_rows: &[NormalControlRow],
        producer: &PrefixMembershipProducer,
    ) -> Result<PrimaryPrefixRoute, Installed48SourceMapError> {
        let controls = self.regular_audio_controls_from_normal_rows(prepared, control_rows)?;
        self.produce_primary_prefix_route_from_rows(
            quiet_regular_rows,
            control_rows,
            &controls,
            producer,
        )
    }

    /// Produces the same prefix-membership route from the current dynamic ADM F01E view.
    pub fn produce_primary_prefix_route_from_adm_input_output(
        &self,
        quiet_regular_rows: &[u8],
        output: &NormalControlAdmInputOutput,
        producer: &PrefixMembershipProducer,
    ) -> Result<PrimaryPrefixRoute, Installed48SourceMapError> {
        let controls = self.regular_audio_controls_from_adm_input_output(output)?;
        self.produce_primary_prefix_route_from_rows(
            quiet_regular_rows,
            output.physical_rows(),
            &controls,
            producer,
        )
    }

    pub fn produce_primary_prefix_route_from_adm_held_view(
        &self,
        quiet_regular_rows: &[u8],
        held: &NormalControlAdmHeldSourceView,
        producer: &PrefixMembershipProducer,
    ) -> Result<PrimaryPrefixRoute, Installed48SourceMapError> {
        let controls = self.regular_audio_controls_from_adm_held_view(held)?;
        self.produce_primary_prefix_route_from_rows(
            quiet_regular_rows,
            held.physical_control_rows(),
            &controls,
            producer,
        )
    }

    fn produce_primary_prefix_route_from_rows(
        &self,
        quiet_regular_rows: &[u8],
        control_rows: &[NormalControlRow],
        controls: &[Installed48AudioControl],
        producer: &PrefixMembershipProducer,
    ) -> Result<PrimaryPrefixRoute, Installed48SourceMapError> {
        if producer.basis_f32le_sha256() != self.p11_geometry.f32le_sha256() {
            return Err(Installed48SourceMapError::Membership(
                PrefixMembershipError::ParameterDomain,
            ));
        }
        if quiet_regular_rows.len() != self.physical_to_global_source.len()
            || quiet_regular_rows.iter().any(|quiet| *quiet > 1)
            || control_rows.len() != usize::from(self.physical_host_count)
            || controls.len() != self.physical_to_global_source.len()
        {
            return Err(Installed48SourceMapError::ControlFieldDomain);
        }

        let mut writers = PrimaryPrefixWriterSet::empty();
        let mut contributions = Vec::new();
        for (tail, binding) in self.physical_to_global_source.iter().enumerate() {
            let source_value = usize::try_from(binding.source_index().value())
                .map_err(|_| Installed48SourceMapError::SourceIndexDomain)?;
            if source_value != PREFIX_COUNT + tail {
                return Err(Installed48SourceMapError::SourceIndexDomain);
            }
            let row = self
                .row_for_source(binding.source_index())
                .ok_or(Installed48SourceMapError::SourceIndexDomain)?;
            if quiet_regular_rows[tail] != 0
                || row.kind() != "object"
                || controls[tail].spread <= 0.0
            {
                continue;
            }
            if controls[tail].mute != 0 {
                return Err(Installed48SourceMapError::AudioControlDomain);
            }
            let NormalControlRow::Object13 { control, .. } =
                &control_rows[usize::from(binding.physical_host().value())]
            else {
                return Err(Installed48SourceMapError::NormalControlIdentityMismatch);
            };
            let result = producer
                .produce(&membership_record(control.words()))
                .map_err(Installed48SourceMapError::Membership)?;
            for member in result.positive_members() {
                let slot = writers
                    .written
                    .get_mut(usize::from(*member))
                    .ok_or(Installed48SourceMapError::SourceIndexDomain)?;
                *slot = true;
            }
            contributions.push(PrimaryPrefixMembershipContribution {
                source_index: binding.source_index(),
                result,
            });
        }
        Ok(PrimaryPrefixRoute {
            writers,
            contributions,
        })
    }

    /// Returns per-tail audio controls after proving the source map and F01E snapshot share the
    /// same admitted physical tracks and regular-source join.
    pub(crate) fn regular_audio_controls_from_normal_rows(
        &self,
        prepared: &NormalControlPreparedSnapshot,
        control_rows: &[NormalControlRow],
    ) -> Result<Vec<Installed48AudioControl>, Installed48SourceMapError> {
        self.validate_normal_control_identity(prepared, control_rows)?;
        let mut controls = Vec::with_capacity(self.physical_to_global_source.len());
        for binding in &self.physical_to_global_source {
            let row = control_rows
                .get(usize::from(binding.physical_host().value()))
                .ok_or(Installed48SourceMapError::NormalControlIdentityMismatch)?;
            controls.push(audio_control(row)?);
        }
        Ok(controls)
    }

    pub(crate) fn regular_audio_controls_from_adm_input_output(
        &self,
        output: &NormalControlAdmInputOutput,
    ) -> Result<Vec<Installed48AudioControl>, Installed48SourceMapError> {
        let prepared = output.prepared_identity();
        let control_rows = output.physical_rows();
        self.regular_audio_controls_from_adm_rows(prepared, control_rows)
    }

    pub(crate) fn regular_audio_controls_from_adm_held_view(
        &self,
        held: &NormalControlAdmHeldSourceView,
    ) -> Result<Vec<Installed48AudioControl>, Installed48SourceMapError> {
        let prepared = held.prepared_snapshot().as_ref();
        let control_rows = held.physical_control_rows();
        self.regular_audio_controls_from_adm_rows(prepared, control_rows)
    }

    fn regular_audio_controls_from_adm_rows(
        &self,
        prepared: &NormalControlAdmInputSnapshot,
        control_rows: &[NormalControlRow],
    ) -> Result<Vec<Installed48AudioControl>, Installed48SourceMapError> {
        self.validate_adm_input_identity(prepared)?;
        self.validate_normal_control_rows(
            prepared.source_samples(),
            prepared.lfe_host(),
            prepared.physical_tracks(),
            prepared.regular_source_bindings(),
            prepared.regular_control_hosts(),
            control_rows,
        )?;
        let mut controls = Vec::with_capacity(self.physical_to_global_source.len());
        for binding in &self.physical_to_global_source {
            let row = control_rows
                .get(usize::from(binding.physical_host().value()))
                .ok_or(Installed48SourceMapError::NormalControlIdentityMismatch)?;
            controls.push(audio_control(row)?);
        }
        Ok(controls)
    }

    fn validate_adm_input_identity(
        &self,
        prepared: &NormalControlAdmInputSnapshot,
    ) -> Result<(), Installed48SourceMapError> {
        let input_identity = prepared.prepared_input_identity();
        let resource_identity = input_identity.resource();
        let configuration_identity = input_identity.normalized_configuration();
        let graph_provenance = input_identity.graph_provenance();
        let metadata_hashes = self.source_binding.metadata_hashes();
        if resource_identity.input_sha256() != *self.source_binding.source_receipt_sha256()
            || resource_identity.resource_bytes() != self.source_binding.source_file_bytes()
            || configuration_identity.config_sha256()
                != *self.source_binding.job_projection_sha256()
            || graph_provenance.axml().payload_sha256() != metadata_hashes.axml()
            || graph_provenance.chna().payload_sha256() != metadata_hashes.chna()
            || graph_provenance.dbmd().payload_sha256() != metadata_hashes.dbmd()
        {
            return Err(Installed48SourceMapError::NormalControlIdentityMismatch);
        }
        Ok(())
    }

    fn validate_normal_control_identity(
        &self,
        prepared: &NormalControlPreparedSnapshot,
        control_rows: &[NormalControlRow],
    ) -> Result<(), Installed48SourceMapError> {
        self.validate_normal_control_rows(
            prepared.source_samples(),
            prepared.lfe_host(),
            prepared.physical_tracks(),
            prepared.regular_source_bindings(),
            prepared.regular37_hosts(),
            control_rows,
        )
    }

    fn validate_normal_control_rows(
        &self,
        source_samples: u64,
        lfe_host: u8,
        prepared_tracks: &[NormalControlPhysicalTrack],
        prepared_bindings: &[NormalControlRegularSourceBinding],
        regular_control_hosts: &[u8],
        control_rows: &[NormalControlRow],
    ) -> Result<(), Installed48SourceMapError> {
        let physical_count = usize::from(self.physical_host_count);
        if lfe_host != self.direct_lfe_host().value()
            || source_samples != self.source_binding.source_samples().value()
            || prepared_bindings.len() != self.physical_to_global_source.len()
            || regular_control_hosts.len() != self.physical_to_global_source.len()
            || prepared_tracks.len() != physical_count
            || control_rows.len() != physical_count
        {
            return Err(Installed48SourceMapError::NormalControlIdentityMismatch);
        }
        for (host, row) in control_rows.iter().enumerate() {
            let host_u8 = u8::try_from(host)
                .map_err(|_| Installed48SourceMapError::NormalControlIdentityMismatch)?;
            let prepared_track = &prepared_tracks[host];
            let map_track = if host_u8 == self.direct_lfe_host().value() {
                &self.direct_lfe_track
            } else {
                let binding = self
                    .physical_to_global_source
                    .iter()
                    .find(|binding| binding.physical_host().value() == host_u8)
                    .ok_or(Installed48SourceMapError::NormalControlIdentityMismatch)?;
                let map_row = self
                    .row_for_source(binding.source_index())
                    .ok_or(Installed48SourceMapError::NormalControlIdentityMismatch)?;
                let Installed48SourceOwner::PhysicalTrack { track, .. } = map_row.owner() else {
                    return Err(Installed48SourceMapError::NormalControlIdentityMismatch);
                };
                track
            };
            if row.physical_host() != host_u8
                || prepared_track.physical_host() != host_u8
                || row.track_uid() != prepared_track.track_uid()
                || row.channel_format_id() != prepared_track.channel_format_id()
                || row.dbmd().track_index() != u16::from(host_u8)
                || map_track.physical_host().value() != host_u8
                || map_track.uid() != prepared_track.track_uid()
                || map_track.channel_format() != prepared_track.channel_format_id()
                || !track_kinds_match(&map_track.kind(), prepared_track.kind(), row)
            {
                return Err(Installed48SourceMapError::NormalControlIdentityMismatch);
            }
        }
        for (map_binding, prepared_binding) in
            self.physical_to_global_source.iter().zip(prepared_bindings)
        {
            if map_binding.physical_host().value() != prepared_binding.physical_host()
                || map_binding.source_index().value() != prepared_binding.global_source()
            {
                return Err(Installed48SourceMapError::NormalControlIdentityMismatch);
            }
        }
        if self
            .physical_to_global_source
            .iter()
            .zip(regular_control_hosts)
            .any(|(binding, host)| binding.physical_host().value() != *host)
        {
            return Err(Installed48SourceMapError::NormalControlIdentityMismatch);
        }
        Ok(())
    }

    /// Applies the specified post-route field transitions to a computed quiet vector and
    /// prefix-writer set. This does not generate membership gains or publish state.
    pub fn postroute_rows(
        &self,
        quiet_regular_rows: &[u8],
        prefix_writers: PrimaryPrefixWriterSet,
    ) -> Result<Vec<Installed48PostRouteRow>, Installed48SourceMapError> {
        if quiet_regular_rows.len() != self.physical_to_global_source.len()
            || quiet_regular_rows.iter().any(|quiet| *quiet > 1)
        {
            return Err(Installed48SourceMapError::ControlFieldDomain);
        }
        let mut output = Vec::with_capacity(self.rows.len());
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
                        .filter(|ordinal| *ordinal < self.physical_to_global_source.len())
                        .ok_or(Installed48SourceMapError::SourceIndexDomain)?;
                    let quiet = quiet_regular_rows[regular_ordinal];
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
        if output.len() != self.rows.len() {
            return Err(Installed48SourceMapError::RowCount);
        }
        Ok(output)
    }
}

/// Encodes bit position = source ID. The row count is the active extent; unused high bits in the
/// final hexadecimal nibble are storage only and do not identify zero-filled source rows.
pub fn field_masks_hex(
    rows: &[Installed48PostRouteRow],
) -> Result<(String, String), Installed48SourceMapError> {
    let mut field0_mask = vec![0_u8; rows.len().div_ceil(8)];
    let mut field4_mask = vec![0_u8; rows.len().div_ceil(8)];
    for (expected_source_index, row) in rows.iter().enumerate() {
        let source_index = usize::try_from(row.source_index().value())
            .map_err(|_| Installed48SourceMapError::SourceIndexDomain)?;
        if source_index != expected_source_index
            || row.field0() > 1
            || row.field4() > 1
            || row.source_index().value() != u64::try_from(source_index).unwrap_or(u64::MAX)
        {
            return Err(Installed48SourceMapError::SourceIndexDomain);
        }
        let byte_index = source_index / 8;
        let bit_index = source_index % 8;
        field0_mask[byte_index] |= (row.field0() as u8) << bit_index;
        field4_mask[byte_index] |= (row.field4() as u8) << bit_index;
    }
    Ok((
        mask_bytes_to_hex(&field0_mask, rows.len()),
        mask_bytes_to_hex(&field4_mask, rows.len()),
    ))
}

fn mask_bytes_to_hex(bytes: &[u8], bit_count: usize) -> String {
    let nibble_count = bit_count.div_ceil(4);
    let mut output = String::with_capacity(nibble_count);
    for nibble_index in (0..nibble_count).rev() {
        let byte = bytes[nibble_index / 2];
        let nibble = if nibble_index % 2 == 0 {
            byte & 0x0f
        } else {
            byte >> 4
        };
        output.push(char::from_digit(u32::from(nibble), 16).unwrap_or('0'));
    }
    output
}

/// Builds the selected primary source map. The direct LFE remains outside regular sources.
pub fn build_primary_installed48_source_map(
    controls: &PrimaryMetadataControlRows,
    geometry: &SelectedP11Geometry,
) -> Result<Installed48SourceMap, Installed48SourceMapError> {
    validate_primary_binding(controls.source_binding())?;
    build_installed48_source_map(controls, geometry)
}

/// Builds the same dynamic source map from an admitted runtime-selected input.
/// Static P11/bed/object mapping is shared with the detached projection path.
pub fn build_meter_installed48_source_map(
    controls: &PrimaryMetadataControlRows,
    geometry: &SelectedP11Geometry,
) -> Result<Installed48SourceMap, Installed48SourceMapError> {
    build_selected_installed48_source_map(controls, geometry)
}

/// Builds the selected ADM source map shared by MAIN F03W rendering and the independent Meter.
///
/// The returned map contains immutable source identities only; it owns no callback state or audio.
pub fn build_selected_installed48_source_map(
    controls: &PrimaryMetadataControlRows,
    geometry: &SelectedP11Geometry,
) -> Result<Installed48SourceMap, Installed48SourceMapError> {
    validate_selected_source_binding(controls.source_binding())?;
    build_installed48_source_map(controls, geometry)
}

fn build_installed48_source_map(
    controls: &PrimaryMetadataControlRows,
    geometry: &SelectedP11Geometry,
) -> Result<Installed48SourceMap, Installed48SourceMapError> {
    let physical_host_count = controls.row_count();
    if controls.rows().len() != physical_host_count
        || usize::from(controls.source_binding().wave().channels()) != physical_host_count
    {
        return Err(Installed48SourceMapError::RowCount);
    }
    let dimensions = SelectedSourceRowDimensions::for_physical_rows(
        u16::try_from(physical_host_count).map_err(|_| Installed48SourceMapError::RowCount)?,
    )
    .map_err(|error| match error {
        SelectedSourceRowShapeError::PhysicalRowCountOutsideSelectedProfile => {
            Installed48SourceMapError::RowCount
        }
        SelectedSourceRowShapeError::SourceAxisCapacityExceeded { .. } => {
            Installed48SourceMapError::SourceAxisCapacityExceeded
        }
    })?;
    let regular_host_count = usize::from(dimensions.regular_row_count());
    let active_source_count = usize::from(dimensions.active_source_count());

    let mut rows = Vec::with_capacity(active_source_count);
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
            point_response_source_fields: installed_source_fields(&metadata, 0),
        });
    }

    let bed_points = selected_bed_source_points();
    let mut physical_rows = controls.rows().iter().collect::<Vec<_>>();
    physical_rows.sort_by_key(|row| row.track().physical_host().value());
    if physical_rows.len() != physical_host_count
        || physical_rows
            .iter()
            .enumerate()
            .any(|(expected, row)| usize::from(row.track().physical_host().value()) != expected)
    {
        return Err(Installed48SourceMapError::PhysicalHostOrder);
    }
    let mut lfe_count = 0;
    let mut direct_lfe_track = None;
    let mut regular_ordinal = 0_usize;
    let mut physical_bindings = Vec::with_capacity(regular_host_count);
    for row in physical_rows {
        let host = row.track().physical_host();
        let is_lfe = matches!(
            row.track().kind(),
            SelectedTrackKind::DirectSpeakersBed { bed_id: 3, .. }
        );
        if is_lfe {
            lfe_count += 1;
            direct_lfe_track = Some(row.track().clone());
            continue;
        }

        let source_value = PREFIX_COUNT
            .checked_add(regular_ordinal)
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
            point_response_source_fields: control52.map_or_else(
                || installed_source_fields(&metadata, 0),
                ObjectControl52::point_response_source_fields,
            ),
        });
        physical_bindings.push(PhysicalSourceBinding {
            physical_host: host,
            source_index,
        });
        regular_ordinal = regular_ordinal
            .checked_add(1)
            .ok_or(Installed48SourceMapError::SourceIndexDomain)?;
    }
    if lfe_count != 1 || regular_ordinal != regular_host_count {
        return Err(Installed48SourceMapError::DirectLfeIdentity);
    }
    if rows.len() != active_source_count {
        return Err(Installed48SourceMapError::RowCount);
    }
    let direct_lfe_track = direct_lfe_track.ok_or(Installed48SourceMapError::DirectLfeIdentity)?;
    Ok(Installed48SourceMap {
        source_binding: controls.source_binding().clone(),
        p11_geometry: geometry.clone(),
        direct_lfe_track,
        physical_host_count: u16::try_from(physical_host_count)
            .map_err(|_| Installed48SourceMapError::RowCount)?,
        rows,
        physical_to_global_source: physical_bindings,
    })
}

fn validate_selected_source_binding(
    binding: &SelectedPrimaryControlBinding,
) -> Result<(), Installed48SourceMapError> {
    if !binding.runtime_resource_bound() || binding.input_kind() != SelectedInputKind::Adm {
        return Err(Installed48SourceMapError::SourceBinding);
    }
    let channels = usize::from(binding.wave().channels());
    if !(MIN_PROFILE_PHYSICAL_HOST_COUNT..=MAX_PROFILE_PHYSICAL_HOST_COUNT).contains(&channels) {
        return Err(Installed48SourceMapError::SourceExtent);
    }
    let block_align = u64::try_from(
        channels
            .checked_mul(3)
            .ok_or(Installed48SourceMapError::SourceExtent)?,
    )
    .map_err(|_| Installed48SourceMapError::SourceExtent)?;
    let sample_count = binding.data_bytes() / block_align;
    if binding.data_bytes() % block_align != 0
        || binding.data_bytes() > binding.source_file_bytes()
        || sample_count != binding.source_samples().value()
        || binding.wave().sample_rate_hz() != 48_000
        || binding.wave().pcm_bits() != 24
    {
        return Err(Installed48SourceMapError::SourceExtent);
    }
    Ok(())
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

fn track_kinds_match(
    map_kind: &SelectedTrackKind,
    prepared_kind: &NormalControlTrackKind,
    row: &NormalControlRow,
) -> bool {
    match (map_kind, prepared_kind, row) {
        (
            SelectedTrackKind::DirectSpeakersBed { bed_id, .. },
            NormalControlTrackKind::Bed {
                bed_id: prepared_bed_id,
                semantic_role,
                ..
            },
            NormalControlRow::Bed6 { words, .. },
        ) => {
            *bed_id == *prepared_bed_id
                && words[..5] == [0, 0, 3, 2, 1]
                && words[5] == u32::from(*bed_id)
                && ((*bed_id == 3) == (semantic_role == "lfe"))
        }
        (
            SelectedTrackKind::Object { semantic_key },
            NormalControlTrackKind::Object {
                semantic_key: prepared_key,
                local_owner_ordinal: prepared_ordinal,
            },
            NormalControlRow::Object13 {
                semantic_key: row_key,
                local_owner_ordinal: row_ordinal,
                ..
            },
        ) => {
            semantic_key == prepared_key
                && semantic_key == row_key
                && prepared_ordinal == row_ordinal
        }
        _ => false,
    }
}

fn audio_control(
    row: &NormalControlRow,
) -> Result<Installed48AudioControl, Installed48SourceMapError> {
    match row {
        NormalControlRow::Bed6 { words, .. } => {
            if words[..5] != [0, 0, 3, 2, 1] {
                return Err(Installed48SourceMapError::AudioControlDomain);
            }
            Ok(Installed48AudioControl {
                mute: 0,
                spread: 0.0,
                diffuse: 0,
            })
        }
        NormalControlRow::Object13 { control, .. } => {
            let words = control.words();
            let xyz = [words[5], words[6], words[7]].map(f32::from_bits);
            let spread = f32::from_bits(words[9]);
            if words[1..5] != [0, 3, 2, 0]
                || words[8] != 0
                || words[11] != 0
                || words[12] != 1
                || words[0] > 1
                || words[10] > 1
                || xyz
                    .iter()
                    .any(|coordinate| !coordinate.is_finite() || !(0.0..=1.0).contains(coordinate))
                || !spread.is_finite()
                || !(0.0..=1.0).contains(&spread)
            {
                return Err(Installed48SourceMapError::AudioControlDomain);
            }
            Ok(Installed48AudioControl {
                mute: u8::try_from(words[0])
                    .map_err(|_| Installed48SourceMapError::AudioControlDomain)?,
                spread,
                diffuse: u8::try_from(words[10])
                    .map_err(|_| Installed48SourceMapError::AudioControlDomain)?,
            })
        }
    }
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

fn installed_source_fields(metadata_words_8_to_55: &[u32; 12], source_skip: u32) -> [u32; 4] {
    [
        metadata_words_8_to_55[7],
        source_skip,
        metadata_words_8_to_55[10],
        metadata_words_8_to_55[11],
    ]
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
