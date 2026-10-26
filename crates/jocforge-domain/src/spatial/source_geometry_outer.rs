// pattern: Functional Core
//! Same-callback global source geometry join and compact F03S outer matrix production.

use super::{
    installed48_source_map::{Installed48SourceMap, Installed48SourceOwner},
    outer_active_update::{OuterActiveUpdateError, active_energy_update},
    outer_energy_fallback::{OuterEnergyFallbackError, evaluate_outer_energy_fallback},
    outer_geometric_correction::{OuterGeometricCorrectionError, geometric_correction},
    outer_postprocess::{OuterPostprocessError, postprocess_coefficients},
    point_response::{
        PointResponseAngleMode, PointResponseError, PointResponseGateInput,
        PointResponseProduction, PointResponseSourceInput, PointResponseSourceSnapshot,
        prepare_point_response_sources, produce_point_response_columns,
    },
    rectangular_nnls::{NnlsError, NnlsStatus, solve_rectangular_nnls},
    selected_p11_geometry::P11PrefixOrdinal,
    selected_p15_geometry::SelectedP15Geometry,
    target_context::{CompactTargetOrdinal, TargetContextProjection, TargetDestination},
};
use crate::{
    frontend::{
        dbmd_sections::DbmdTrackFields,
        normal_control_snapshot::{
            NormalControlDbmdFields, NormalControlHeldSourceView, NormalControlPreparedSnapshot,
            NormalControlRow, NormalControlSnapshotProposal,
        },
        selected_adm_graph::{PhysicalHostIndex, SelectedAdmGraph, SelectedTrackKind},
    },
    spatial::source_flags::SourceControlSnapshotClock,
    timeline::{CodecSourceIndex, FrontendCallbackIndex},
    transaction::CallbackLineageIdentity,
};

const GLOBAL_SOURCE_COUNT: usize = 48;
const VIRTUAL_PREFIX_SOURCE_COUNT: usize = 11;
const REGULAR_SOURCE_COUNT: usize = GLOBAL_SOURCE_COUNT - VIRTUAL_PREFIX_SOURCE_COUNT;
const PHYSICAL_TRACK_COUNT: usize = 38;
const TOTAL_SOURCE_COUNT_U64: u64 = 48;
const TARGET_SLOT_COUNT: usize = 15;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OuterSourceResult {
    source_index: CodecSourceIndex,
    nnls_status: NnlsStatus,
    active_branch: bool,
    seed_u32: Box<[u32]>,
    final_u32: Box<[u32]>,
}

impl OuterSourceResult {
    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn nnls_status(&self) -> NnlsStatus {
        self.nnls_status
    }

    pub const fn active_branch(&self) -> bool {
        self.active_branch
    }

    pub fn seed_u32(&self) -> &[u32] {
        &self.seed_u32
    }

    pub fn final_u32(&self) -> &[u32] {
        &self.final_u32
    }
}

/// Compact F03S output matrix. Rows are compact target ordinals; columns are global sources0..47.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactOuterMatrix {
    transaction_id: CallbackLineageIdentity,
    value_generation: FrontendCallbackIndex,
    source_origin_generation: FrontendCallbackIndex,
    source_receipt_sha256: [u8; 32],
    target_context_identity: super::selected_p15_geometry::SelectedP15GeometryIdentity,
    destinations: Box<[TargetDestination]>,
    processing_source_order: Box<[CodecSourceIndex]>,
    c_rows_u32: Box<[[u32; GLOBAL_SOURCE_COUNT]]>,
    source_results: Box<[OuterSourceResult]>,
}

impl CompactOuterMatrix {
    pub fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub const fn value_generation(&self) -> FrontendCallbackIndex {
        self.value_generation
    }

    pub const fn source_origin_generation(&self) -> FrontendCallbackIndex {
        self.source_origin_generation
    }

    pub const fn source_receipt_sha256(&self) -> &[u8; 32] {
        &self.source_receipt_sha256
    }

    pub const fn target_context_identity(
        &self,
    ) -> super::selected_p15_geometry::SelectedP15GeometryIdentity {
        self.target_context_identity
    }

    pub fn destinations(&self) -> &[TargetDestination] {
        &self.destinations
    }

    pub fn processing_source_order(&self) -> &[CodecSourceIndex] {
        &self.processing_source_order
    }

    pub fn c_rows_u32(&self) -> &[[u32; GLOBAL_SOURCE_COUNT]] {
        &self.c_rows_u32
    }

    pub fn source_results(&self) -> &[OuterSourceResult] {
        &self.source_results
    }

    pub fn coefficient(
        &self,
        target: CompactTargetOrdinal,
        source: CodecSourceIndex,
    ) -> Option<u32> {
        let row = self.c_rows_u32.get(usize::from(target.value()))?;
        let column = usize::try_from(source.value()).ok()?;
        row.get(column).copied()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompactOuterMatrixError {
    CallbackIdentityMismatch,
    SourceGeometryIdentityMismatch,
    TargetContextShapeMismatch,
    ResponseSourceOrderInvalid,
    ResponseColumnMissing,
    SourceGeometryMissing,
    Nnls(NnlsError),
    EnergyFallback(OuterEnergyFallbackError),
    GeometricCorrection(OuterGeometricCorrectionError),
    ActiveUpdate(OuterActiveUpdateError),
    Postprocess(OuterPostprocessError),
    OutputShapeMismatch,
}

/// Origin of one raw, same-callback global source XYZ row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GlobalSourceGeometryOwner {
    VirtualPrefix {
        prefix_ordinal: P11PrefixOrdinal,
        source_layout_index: u8,
    },
    Bed {
        physical_host: PhysicalHostIndex,
        track_uid: String,
        bed_id: u8,
    },
    Object {
        physical_host: PhysicalHostIndex,
        track_uid: String,
        semantic_key: u32,
        local_owner_ordinal: u8,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalSourceGeometryRow {
    source_index: CodecSourceIndex,
    xyz_f32_bits: [u32; 3],
    original_audio_width_f32_bits: u32,
    feature_mode: u32,
    source_skip: u32,
    spatial_enum: u32,
    coordinate_parameter: u32,
    owner: GlobalSourceGeometryOwner,
}

impl GlobalSourceGeometryRow {
    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn xyz_f32_bits(&self) -> [u32; 3] {
        self.xyz_f32_bits
    }

    pub const fn original_audio_width_f32_bits(&self) -> u32 {
        self.original_audio_width_f32_bits
    }

    pub const fn feature_mode(&self) -> u32 {
        self.feature_mode
    }

    pub const fn source_skip(&self) -> u32 {
        self.source_skip
    }

    pub const fn spatial_enum(&self) -> u32 {
        self.spatial_enum
    }

    pub const fn coordinate_parameter(&self) -> u32 {
        self.coordinate_parameter
    }

    pub const fn owner(&self) -> &GlobalSourceGeometryOwner {
        &self.owner
    }
}

/// Raw global source geometry joined from F01B's installed48 map and one F01E proposal.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalSourceGeometrySnapshot {
    transaction_id: CallbackLineageIdentity,
    value_generation: FrontendCallbackIndex,
    source_origin_generation: FrontendCallbackIndex,
    source_receipt_sha256: [u8; 32],
    rows: [GlobalSourceGeometryRow; GLOBAL_SOURCE_COUNT],
}

impl GlobalSourceGeometrySnapshot {
    pub fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub const fn value_generation(&self) -> FrontendCallbackIndex {
        self.value_generation
    }

    pub const fn source_origin_generation(&self) -> FrontendCallbackIndex {
        self.source_origin_generation
    }

    pub const fn source_receipt_sha256(&self) -> &[u8; 32] {
        &self.source_receipt_sha256
    }

    pub const fn rows(&self) -> &[GlobalSourceGeometryRow; GLOBAL_SOURCE_COUNT] {
        &self.rows
    }

    pub fn row_for_source(
        &self,
        source_index: CodecSourceIndex,
    ) -> Option<&GlobalSourceGeometryRow> {
        let ordinal = usize::try_from(source_index.value()).ok()?;
        self.rows
            .get(ordinal)
            .filter(|row| row.source_index == source_index)
    }

    /// Binds the exact installed source geometry and semantic point fields to one source clock.
    pub fn point_response_snapshot(
        &self,
        source_control_clock: SourceControlSnapshotClock,
    ) -> Result<PointResponseSourceSnapshot, GlobalSourceGeometryError> {
        if source_control_clock.callback_index() != self.value_generation
            || source_control_clock.source_origin() != self.source_origin_generation
        {
            return Err(GlobalSourceGeometryError::CallbackIdentityMismatch);
        }
        let rows = self
            .rows
            .iter()
            .map(|row| {
                PointResponseSourceInput::new(
                    row.source_index,
                    row.xyz_f32_bits,
                    row.original_audio_width_f32_bits,
                    row.feature_mode,
                    row.source_skip,
                    row.spatial_enum,
                    row.coordinate_parameter,
                )
            })
            .collect();
        PointResponseSourceSnapshot::new(
            self.value_generation,
            self.transaction_id.clone(),
            source_control_clock,
            rows,
        )
        .map_err(GlobalSourceGeometryError::PointResponseInput)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlobalSourceGeometryError {
    PreparedGraphMismatch,
    CallbackIdentityMismatch,
    SourceMapIdentityMismatch,
    DirectLfeIdentityMismatch,
    PhysicalRowIdentityMismatch,
    RegularSourceBindingMismatch,
    PrefixIdentityMismatch,
    BedGeometryMismatch,
    ObjectOwnerBindingMismatch,
    NonFiniteGeometry,
    SourceIndexDomain,
    PointResponseInput(PointResponseError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallbackPointResponseError {
    SourceView(GlobalSourceGeometryError),
    Response(PointResponseError),
}

/// Builds source rows0..10 from the installed P11 prefix and rows11..47 from graph-bound F01E rows.
///
/// The direct semantic LFE is excluded by the graph's `regular_source_bindings`. Bed geometry
/// comes from the selected F01B source map. Object geometry comes from the same callback's
/// sampled Object13 row (control words5..7), never from P11/P15 geometry or marker history.
pub fn build_global_source_geometry_snapshot(
    graph: &SelectedAdmGraph,
    source_map: &Installed48SourceMap,
    normal: &NormalControlSnapshotProposal,
) -> Result<GlobalSourceGeometrySnapshot, GlobalSourceGeometryError> {
    let after = normal.after_state();
    let prepared = after.prepared_identity();
    let transaction_id = normal.transaction_id();
    if transaction_id.programme_id() != after.programme_id()
        || transaction_id.attempt_id() != after.attempt_id()
        || transaction_id.callback_index() != normal.value_generation()
        || normal.normal_origin_generation() != normal.value_generation()
        || after.next_callback_index().value().checked_sub(1)
            != Some(normal.value_generation().value())
    {
        return Err(GlobalSourceGeometryError::CallbackIdentityMismatch);
    }

    build_global_source_geometry_from_rows(
        graph,
        source_map,
        transaction_id,
        normal.value_generation(),
        normal.normal_origin_generation(),
        prepared,
        normal.control38_rows(),
    )
}

/// Reuses committed last-normal F01E rows at a later main cached-audio delivery callback.
/// The returned value generation is delivery g; its source geometry origin remains Q-1.
pub fn build_held_global_source_geometry_snapshot(
    graph: &SelectedAdmGraph,
    source_map: &Installed48SourceMap,
    held: &NormalControlHeldSourceView,
) -> Result<GlobalSourceGeometrySnapshot, GlobalSourceGeometryError> {
    let transaction_id = held.delivery_transaction_id();
    if transaction_id.callback_index() != held.delivery_generation()
        || held.source_origin_transaction_id().programme_id() != transaction_id.programme_id()
        || held.source_origin_transaction_id().attempt_id() != transaction_id.attempt_id()
        || held.source_origin_transaction_id().callback_index() != held.source_origin_generation()
    {
        return Err(GlobalSourceGeometryError::CallbackIdentityMismatch);
    }
    build_global_source_geometry_from_rows(
        graph,
        source_map,
        transaction_id,
        held.delivery_generation(),
        held.source_origin_generation(),
        held.prepared_identity(),
        held.control38_rows(),
    )
}

fn build_global_source_geometry_from_rows(
    graph: &SelectedAdmGraph,
    source_map: &Installed48SourceMap,
    transaction_id: &CallbackLineageIdentity,
    value_generation: FrontendCallbackIndex,
    source_origin_generation: FrontendCallbackIndex,
    prepared: &NormalControlPreparedSnapshot,
    control38_rows: &[NormalControlRow; PHYSICAL_TRACK_COUNT],
) -> Result<GlobalSourceGeometrySnapshot, GlobalSourceGeometryError> {
    if !prepared.matches_graph(graph) {
        return Err(GlobalSourceGeometryError::PreparedGraphMismatch);
    }
    if transaction_id.programme_id() != prepared.programme_id()
        || transaction_id.attempt_id() != prepared.attempt_id()
        || transaction_id.callback_index() != value_generation
        || source_origin_generation.value() > value_generation.value()
    {
        return Err(GlobalSourceGeometryError::CallbackIdentityMismatch);
    }

    let source_binding = source_map.source_binding();
    let metadata_hashes = source_binding.metadata_hashes();
    if source_binding.input_kind() != crate::frontend::job_configuration::SelectedInputKind::Adm
        || source_binding.selected_profile() != prepared.profile_identity()
        || source_binding.source_samples().value() != prepared.source_samples()
        || metadata_hashes.axml() != *graph.receipt().axml_sha256()
        || metadata_hashes.chna() != *graph.receipt().chna_sha256()
        || source_map.rows().len() != GLOBAL_SOURCE_COUNT
        || source_map.prefix_count() != u8::try_from(VIRTUAL_PREFIX_SOURCE_COUNT).unwrap_or(u8::MAX)
        || source_map.regular_count() != u8::try_from(REGULAR_SOURCE_COUNT).unwrap_or(u8::MAX)
    {
        return Err(GlobalSourceGeometryError::SourceMapIdentityMismatch);
    }
    let prepared_tracks = prepared.physical_tracks();
    if control38_rows.len() != PHYSICAL_TRACK_COUNT || prepared_tracks.len() != PHYSICAL_TRACK_COUNT
    {
        return Err(GlobalSourceGeometryError::PhysicalRowIdentityMismatch);
    }
    for host in 0..PHYSICAL_TRACK_COUNT {
        let host_index = u8::try_from(host)
            .map_err(|_| GlobalSourceGeometryError::PhysicalRowIdentityMismatch)?;
        let graph_track = graph
            .tracks()
            .iter()
            .find(|track| track.physical_host().value() == host_index)
            .ok_or(GlobalSourceGeometryError::PhysicalRowIdentityMismatch)?;
        let row = &control38_rows[host];
        let prepared_track = &prepared_tracks[host];
        if usize::from(row.physical_host()) != host
            || row.track_uid() != graph_track.uid()
            || row.channel_format_id() != graph_track.channel_format()
            || prepared_track.physical_host() != host_index
            || prepared_track.track_uid() != graph_track.uid()
            || prepared_track.channel_format_id() != graph_track.channel_format()
            || prepared_track.dbmd() != row.dbmd()
        {
            return Err(GlobalSourceGeometryError::PhysicalRowIdentityMismatch);
        }
    }
    if graph.regular_source_bindings().len() != REGULAR_SOURCE_COUNT
        || source_map.physical_to_global_source().len() != REGULAR_SOURCE_COUNT
        || prepared.regular_source_bindings().len() != REGULAR_SOURCE_COUNT
    {
        return Err(GlobalSourceGeometryError::RegularSourceBindingMismatch);
    }

    let lfe_host = graph.receipt().direct_lfe_host();
    if source_map.source_for_physical_host(lfe_host).is_some()
        || graph
            .regular_source_bindings()
            .iter()
            .any(|binding| binding.physical_host() == lfe_host)
    {
        return Err(GlobalSourceGeometryError::DirectLfeIdentityMismatch);
    }
    let lfe_track = graph
        .tracks()
        .iter()
        .find(|track| track.physical_host() == lfe_host)
        .ok_or(GlobalSourceGeometryError::DirectLfeIdentityMismatch)?;
    let lfe_row = control38_rows
        .get(usize::from(lfe_host.value()))
        .ok_or(GlobalSourceGeometryError::DirectLfeIdentityMismatch)?;
    if !matches!(
        lfe_track.kind(),
        SelectedTrackKind::DirectSpeakersBed { bed_id: 3, .. }
    ) || !matches!(lfe_row, NormalControlRow::Bed6 { words, .. } if *words == [0, 0, 3, 2, 1, 3])
    {
        return Err(GlobalSourceGeometryError::DirectLfeIdentityMismatch);
    }

    let prefix_rows = build_prefix_source_geometry_rows(source_map)?;
    let regular_rows =
        build_regular_source_geometry_rows(graph, source_map, prepared, control38_rows)?;
    let rows = prefix_rows
        .into_iter()
        .chain(regular_rows)
        .enumerate()
        .map(|(expected, row)| {
            if row.source_index.value() == u64::try_from(expected).unwrap_or(u64::MAX) {
                Ok(row)
            } else {
                Err(GlobalSourceGeometryError::SourceIndexDomain)
            }
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| GlobalSourceGeometryError::SourceIndexDomain)?;

    Ok(GlobalSourceGeometrySnapshot {
        transaction_id: transaction_id.clone(),
        value_generation,
        source_origin_generation,
        source_receipt_sha256: *source_binding.source_receipt_sha256(),
        rows,
    })
}

/// Joins the same-callback installed source rows and gate to the selected P15 response producer.
pub fn produce_callback_point_response(
    source_geometry: &GlobalSourceGeometrySnapshot,
    source_control_clock: SourceControlSnapshotClock,
    geometry: &SelectedP15Geometry,
    gate: PointResponseGateInput,
) -> Result<PointResponseProduction, CallbackPointResponseError> {
    let sources = source_geometry
        .point_response_snapshot(source_control_clock)
        .map_err(CallbackPointResponseError::SourceView)?;
    let batch = prepare_point_response_sources(sources, gate)
        .map_err(CallbackPointResponseError::Response)?;
    produce_point_response_columns(geometry, batch).map_err(CallbackPointResponseError::Response)
}

fn build_prefix_source_geometry_rows(
    source_map: &Installed48SourceMap,
) -> Result<Vec<GlobalSourceGeometryRow>, GlobalSourceGeometryError> {
    let points = source_map.p11_geometry().points();
    let mut rows = Vec::with_capacity(VIRTUAL_PREFIX_SOURCE_COUNT);
    for (prefix_index, point) in points.iter().copied().enumerate() {
        let source_index = CodecSourceIndex::new(
            u64::try_from(prefix_index)
                .map_err(|_| GlobalSourceGeometryError::SourceIndexDomain)?,
        );
        let installed = source_map
            .row_for_source(source_index)
            .ok_or(GlobalSourceGeometryError::PrefixIdentityMismatch)?;
        let Installed48SourceOwner::P11 {
            layout_ordinal,
            source_layout_index,
        } = installed.owner()
        else {
            return Err(GlobalSourceGeometryError::PrefixIdentityMismatch);
        };
        if installed.kind() != "virtual_prefix"
            || layout_ordinal.value() != point.ordinal().value()
            || *source_layout_index != point.source_layout_index()
            || installed.metadata_words_8_to_55()[4..7] != point.xyz_words()
        {
            return Err(GlobalSourceGeometryError::PrefixIdentityMismatch);
        }
        let xyz_f32_bits = point.xyz_words();
        if xyz_f32_bits
            .iter()
            .any(|word| !f32::from_bits(*word).is_finite())
        {
            return Err(GlobalSourceGeometryError::NonFiniteGeometry);
        }
        rows.push(GlobalSourceGeometryRow {
            source_index,
            xyz_f32_bits,
            original_audio_width_f32_bits: installed.metadata_words_8_to_55()[8],
            feature_mode: installed.point_response_source_fields()[0],
            source_skip: installed.point_response_source_fields()[1],
            spatial_enum: installed.point_response_source_fields()[2],
            coordinate_parameter: installed.point_response_source_fields()[3],
            owner: GlobalSourceGeometryOwner::VirtualPrefix {
                prefix_ordinal: point.ordinal(),
                source_layout_index: point.source_layout_index(),
            },
        });
    }
    Ok(rows)
}

fn build_regular_source_geometry_rows(
    graph: &SelectedAdmGraph,
    source_map: &Installed48SourceMap,
    prepared: &NormalControlPreparedSnapshot,
    control38_rows: &[NormalControlRow; PHYSICAL_TRACK_COUNT],
) -> Result<Vec<GlobalSourceGeometryRow>, GlobalSourceGeometryError> {
    let mut rows_by_regular_ordinal: Vec<Option<GlobalSourceGeometryRow>> =
        (0..REGULAR_SOURCE_COUNT).map(|_| None).collect();
    for graph_binding in graph.regular_source_bindings() {
        let host = graph_binding.physical_host();
        let source_index = graph_binding.global_source();
        let mut prepared_bindings = prepared
            .regular_source_bindings()
            .iter()
            .filter(|binding| binding.physical_host() == host.value());
        let prepared_binding = prepared_bindings
            .next()
            .ok_or(GlobalSourceGeometryError::RegularSourceBindingMismatch)?;
        if prepared_binding.physical_host() != host.value()
            || prepared_binding.global_source() != source_index.value()
            || prepared_bindings.next().is_some()
            || host == graph.receipt().direct_lfe_host()
            || source_map.source_for_physical_host(host) != Some(source_index)
        {
            return Err(GlobalSourceGeometryError::RegularSourceBindingMismatch);
        }
        let regular_ordinal = usize::try_from(
            source_index
                .value()
                .checked_sub(u64::try_from(VIRTUAL_PREFIX_SOURCE_COUNT).unwrap_or(u64::MAX))
                .ok_or(GlobalSourceGeometryError::SourceIndexDomain)?,
        )
        .map_err(|_| GlobalSourceGeometryError::SourceIndexDomain)?;
        let destination = rows_by_regular_ordinal
            .get_mut(regular_ordinal)
            .ok_or(GlobalSourceGeometryError::SourceIndexDomain)?;
        if destination.is_some() {
            return Err(GlobalSourceGeometryError::SourceIndexDomain);
        }

        let physical_host = usize::from(host.value());
        let graph_track = graph
            .tracks()
            .iter()
            .find(|track| track.physical_host() == host)
            .ok_or(GlobalSourceGeometryError::PhysicalRowIdentityMismatch)?;
        let control_row = control38_rows
            .get(physical_host)
            .ok_or(GlobalSourceGeometryError::PhysicalRowIdentityMismatch)?;
        let source_map_row = source_map
            .row_for_source(source_index)
            .ok_or(GlobalSourceGeometryError::RegularSourceBindingMismatch)?;
        let Installed48SourceOwner::PhysicalTrack {
            track: mapped_track,
            dbmd: mapped_dbmd,
        } = source_map_row.owner()
        else {
            return Err(GlobalSourceGeometryError::RegularSourceBindingMismatch);
        };
        if source_map_row.source_index() != source_index
            || mapped_track != graph_track
            || source_map_row.kind()
                != match graph_track.kind() {
                    SelectedTrackKind::DirectSpeakersBed { .. } => "bed",
                    SelectedTrackKind::Object { .. } => "object",
                }
            || control_row.physical_host() != host.value()
            || control_row.track_uid() != graph_track.uid()
            || control_row.channel_format_id() != graph_track.channel_format()
        {
            return Err(GlobalSourceGeometryError::PhysicalRowIdentityMismatch);
        }

        let (xyz_f32_bits, original_audio_width_f32_bits, source_fields, owner) =
            match (graph_track.kind(), control_row) {
                (
                    SelectedTrackKind::DirectSpeakersBed { bed_id, .. },
                    NormalControlRow::Bed6 { words, dbmd, .. },
                ) => {
                    if words != &[0, 0, 3, 2, 1, u32::from(bed_id)]
                        || !dbmd_fields_match(*dbmd, *mapped_dbmd)
                    {
                        return Err(GlobalSourceGeometryError::BedGeometryMismatch);
                    }
                    let metadata = source_map_row.metadata_words_8_to_55();
                    let xyz = [metadata[4], metadata[5], metadata[6]];
                    if xyz.iter().any(|word| !f32::from_bits(*word).is_finite()) {
                        return Err(GlobalSourceGeometryError::NonFiniteGeometry);
                    }
                    (
                        xyz,
                        metadata[8],
                        source_map_row.point_response_source_fields(),
                        GlobalSourceGeometryOwner::Bed {
                            physical_host: host,
                            track_uid: graph_track.uid().to_owned(),
                            bed_id,
                        },
                    )
                }
                (
                    SelectedTrackKind::Object { semantic_key },
                    NormalControlRow::Object13 {
                        semantic_key: row_semantic_key,
                        local_owner_ordinal,
                        dbmd,
                        control,
                        ..
                    },
                ) => {
                    let owner_binding = graph
                        .object_sampler_owner_binding_for_uid(graph_track.uid())
                        .ok_or(GlobalSourceGeometryError::ObjectOwnerBindingMismatch)?;
                    if *row_semantic_key != semantic_key
                        || owner_binding.semantic_key() != semantic_key
                        || owner_binding.physical_host() != host
                        || owner_binding.local_owner_ordinal().value() != *local_owner_ordinal
                        || !dbmd_fields_match(*dbmd, *mapped_dbmd)
                    {
                        return Err(GlobalSourceGeometryError::ObjectOwnerBindingMismatch);
                    }
                    let words = control.words();
                    let xyz = [words[5], words[6], words[7]];
                    if xyz
                        .iter()
                        .chain([&words[9]])
                        .any(|word| !f32::from_bits(*word).is_finite())
                    {
                        return Err(GlobalSourceGeometryError::NonFiniteGeometry);
                    }
                    (
                        xyz,
                        words[9],
                        control.point_response_source_fields(),
                        GlobalSourceGeometryOwner::Object {
                            physical_host: host,
                            track_uid: graph_track.uid().to_owned(),
                            semantic_key,
                            local_owner_ordinal: *local_owner_ordinal,
                        },
                    )
                }
                _ => return Err(GlobalSourceGeometryError::PhysicalRowIdentityMismatch),
            };
        *destination = Some(GlobalSourceGeometryRow {
            source_index,
            xyz_f32_bits,
            original_audio_width_f32_bits,
            feature_mode: source_fields[0],
            source_skip: source_fields[1],
            spatial_enum: source_fields[2],
            coordinate_parameter: source_fields[3],
            owner,
        });
    }

    rows_by_regular_ordinal
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or(GlobalSourceGeometryError::RegularSourceBindingMismatch)
}

fn dbmd_fields_match(normal: NormalControlDbmdFields, source_map: DbmdTrackFields) -> bool {
    normal.track_index() == source_map.track_index()
        && normal.enum48() == source_map.enum48()
        && normal.enum52() == source_map.enum52()
        && normal.flag64() == source_map.flag64()
        && normal.raw_flag_low2() == source_map.raw_flag_low2()
        && normal.raw_packed() == source_map.raw_packed()
}

/// Runs cold NNLS and the selected F03S branch for each processing source, then builds C[n][48].
///
/// Candidate-only response columns are retained as response inputs but are never passed to NNLS
/// or the outer solver. Every source call starts with a fresh zero index state and scalar flag0.
pub fn produce_compact_outer_matrix(
    target_context: &TargetContextProjection,
    source_geometry: &GlobalSourceGeometrySnapshot,
    response: &PointResponseProduction,
) -> Result<CompactOuterMatrix, CompactOuterMatrixError> {
    let clock = response.source_control_clock();
    if target_context.callback_index() != source_geometry.value_generation
        || target_context.callback_index() != response.callback_index()
        || target_context.transaction_id() != &source_geometry.transaction_id
        || target_context.transaction_id() != response.transaction_id()
    {
        return Err(CompactOuterMatrixError::CallbackIdentityMismatch);
    }
    if response.callback_index() != source_geometry.value_generation
        || response.callback_index() != clock.callback_index()
        || response.transaction_id() != &source_geometry.transaction_id
        || clock.source_origin() != source_geometry.source_origin_generation
        || response.basis_identity() != target_context.geometry_identity()
        || response.angle_mode() != PointResponseAngleMode::MathematicalR64R32
    {
        return Err(CompactOuterMatrixError::SourceGeometryIdentityMismatch);
    }
    validate_target_context_shape(target_context)?;
    validate_response_columns(response)?;

    let mut processing_source_order = Vec::with_capacity(response.processing_ids().len());
    let mut previous_source = None;
    for value in response.processing_ids().iter().copied() {
        if u64::from(value) >= TOTAL_SOURCE_COUNT_U64
            || previous_source.is_some_and(|previous| previous >= value)
        {
            return Err(CompactOuterMatrixError::ResponseSourceOrderInvalid);
        }
        let source_index = CodecSourceIndex::new(u64::from(value));
        if !response
            .response_source_ids()
            .iter()
            .any(|candidate| candidate.value() == source_index.value())
            || source_geometry.row_for_source(source_index).is_none()
        {
            return Err(CompactOuterMatrixError::SourceGeometryMissing);
        }
        processing_source_order.push(source_index);
        previous_source = Some(value);
    }

    let destination_count = usize::from(target_context.compact_target_count());
    let mut c_rows_u32 = vec![[0_u32; GLOBAL_SOURCE_COUNT]; destination_count];
    let mut source_results = Vec::with_capacity(processing_source_order.len());
    if destination_count > 0 && !processing_source_order.is_empty() {
        let context_rows = target_context
            .compact_context_u32()
            .iter()
            .map(|row| row.to_vec())
            .collect::<Vec<_>>();
        let target_xyz = target_context
            .compact_target_xyz_u32()
            .iter()
            .map(|xyz| xyz.to_vec())
            .collect::<Vec<_>>();

        for source_index in processing_source_order.iter().copied() {
            let column = response
                .columns()
                .iter()
                .find(|column| column.source_index() == source_index)
                .ok_or(CompactOuterMatrixError::ResponseColumnMissing)?;
            let source = source_geometry
                .row_for_source(source_index)
                .ok_or(CompactOuterMatrixError::SourceGeometryMissing)?;
            let response_words = column.response_f32_bits();
            let nnls = solve_rectangular_nnls(&context_rows, &response_words)
                .map_err(CompactOuterMatrixError::Nnls)?;
            let nnls_status = nnls.status;
            let seed_u32 = nnls.seed_u32;
            let branch = evaluate_outer_energy_fallback(&context_rows, &response_words, &seed_u32)
                .map_err(CompactOuterMatrixError::EnergyFallback)?;
            let active_branch = branch.active();
            let final_u32 = if active_branch {
                let source_xyz = source.xyz_f32_bits();
                let index_state = vec![0_u32; destination_count];
                let correction = geometric_correction(&target_xyz, &source_xyz, 0, &index_state)
                    .map_err(CompactOuterMatrixError::GeometricCorrection)?;
                let active = active_energy_update(
                    &context_rows,
                    &response_words,
                    &seed_u32,
                    correction.correction_u32(),
                )
                .map_err(CompactOuterMatrixError::ActiveUpdate)?;
                postprocess_coefficients(
                    active.output_u32(),
                    super::outer_energy_fallback::FLOOR_U32,
                    super::outer_energy_fallback::DOMINANCE_RATIO_U32,
                )
                .map_err(CompactOuterMatrixError::Postprocess)?
                .final_u32()
                .to_vec()
            } else {
                branch.final_u32().to_vec()
            };
            if final_u32.len() != destination_count
                || final_u32
                    .iter()
                    .any(|word| !f32::from_bits(*word).is_finite())
            {
                return Err(CompactOuterMatrixError::OutputShapeMismatch);
            }

            let source_column = usize::try_from(source_index.value())
                .map_err(|_| CompactOuterMatrixError::OutputShapeMismatch)?;
            for (destination_row, coefficient) in final_u32.iter().copied().enumerate() {
                c_rows_u32[destination_row][source_column] = coefficient;
            }
            source_results.push(OuterSourceResult {
                source_index,
                nnls_status,
                active_branch,
                seed_u32: seed_u32.into_boxed_slice(),
                final_u32: final_u32.into_boxed_slice(),
            });
        }
    }

    Ok(CompactOuterMatrix {
        transaction_id: source_geometry.transaction_id.clone(),
        value_generation: source_geometry.value_generation,
        source_origin_generation: source_geometry.source_origin_generation,
        source_receipt_sha256: source_geometry.source_receipt_sha256,
        target_context_identity: target_context.geometry_identity(),
        destinations: target_context.destinations().to_vec().into_boxed_slice(),
        processing_source_order: processing_source_order.into_boxed_slice(),
        c_rows_u32: c_rows_u32.into_boxed_slice(),
        source_results: source_results.into_boxed_slice(),
    })
}

fn validate_target_context_shape(
    target_context: &TargetContextProjection,
) -> Result<(), CompactOuterMatrixError> {
    let target_count = usize::from(target_context.compact_target_count());
    if target_count > TARGET_SLOT_COUNT
        || target_context.destinations().len() != target_count
        || target_context.compact_target_xyz_u32().len() != target_count
        || target_context.compact_context_u32().len() != TARGET_SLOT_COUNT
        || target_context
            .compact_context_u32()
            .iter()
            .any(|row| row.len() != target_count)
        || target_context
            .compact_target_xyz_u32()
            .iter()
            .flatten()
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(CompactOuterMatrixError::TargetContextShapeMismatch);
    }
    Ok(())
}

fn validate_response_columns(
    response: &PointResponseProduction,
) -> Result<(), CompactOuterMatrixError> {
    if response.response_source_ids().len() != response.columns().len()
        || response
            .response_source_ids()
            .windows(2)
            .any(|pair| pair[0].value() >= pair[1].value())
    {
        return Err(CompactOuterMatrixError::ResponseSourceOrderInvalid);
    }
    for (source_index, column) in response
        .response_source_ids()
        .iter()
        .copied()
        .zip(response.columns())
    {
        if source_index != column.source_index() || source_index.value() >= TOTAL_SOURCE_COUNT_U64 {
            return Err(CompactOuterMatrixError::ResponseColumnMissing);
        }
    }
    Ok(())
}

#[cfg(test)]
pub(in crate::spatial) mod tests;
