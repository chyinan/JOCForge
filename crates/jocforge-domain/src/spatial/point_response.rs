// pattern: Functional Core
//! Validates source inputs and prepares generated-source views for F03P response columns.

use super::selected_p15_geometry::{SelectedP15Geometry, SelectedP15GeometryIdentity};
use crate::{
    numeric::{
        FiniteF32, R32Error, exact_trigonometry::exact_r64_to_r32, normalize_q15_coordinate,
        quantize_q15_coordinate, r32_add, r32_mul, r32_sub,
    },
    spatial::source_flags::SourceControlSnapshotClock,
    timeline::{CodecSourceIndex, FrontendCallbackIndex},
    transaction::CallbackLineageIdentity,
};

const SOURCE_COUNT: usize = 48;
const SOURCE_COUNT_U32: u32 = 48;
const SELECTED_BASIS_COUNT: usize = 15;
const SELECTED_LOWER_BASIS_COUNT: usize = 9;
const SELECTED_UPPER_BASIS_COUNT: usize = 6;
const SELECTED_FEATURE_MODE: u32 = 0;
const SELECTED_SPATIAL_ENUM: u32 = 0;
const POST_AUDIO_FEATURE_WIDTH_POSITIVE_ZERO: u32 = 0;
const LOWER_PLANE_Z_EXCLUSIVE_BITS: u32 = 0xbdcc_cccd;
const LOWER_PLANE_Z_INCLUSIVE_MAX_BITS: u32 = 0x3f00_0000;
const UPPER_PLANE_Z_MIN_BITS: u32 = 0x3f00_0000;
const GROUPING_Y_TOLERANCE_BITS: u32 = 0x3c23_d70a;
const RESPONSE_THRESHOLD_BITS: u32 = 0x3586_37bd;
const PI_OVER_FOUR_BITS: u32 = 0x3f49_0fdb;
const ONE_BITS: u32 = 0x3f80_0000;
const HALF_BITS: u32 = 0x3f00_0000;
const FOUR_BITS: u32 = 0x4080_0000;
const Q15_MAX: i32 = 32_767;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointResponseError {
    SourceCount,
    SourceIdentity,
    SourceGateIdentityMismatch,
    SourceMode,
    GeneratedSourceBranch,
    NonFiniteInput,
    NumericFailure,
    BasisDomain,
    BasisGrouping,
}

impl PointResponseError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::SourceCount => "POINT_RESPONSE_SOURCE_COUNT",
            Self::SourceIdentity => "POINT_RESPONSE_SOURCE_IDENTITY",
            Self::SourceGateIdentityMismatch => "POINT_RESPONSE_SOURCE_GATE_IDENTITY",
            Self::SourceMode => "POINT_RESPONSE_SOURCE_MODE",
            Self::GeneratedSourceBranch => "POINT_RESPONSE_GENERATED_SOURCE_BRANCH",
            Self::NonFiniteInput => "POINT_RESPONSE_INPUT_NONFINITE",
            Self::NumericFailure => "POINT_RESPONSE_NUMERIC_FAILURE",
            Self::BasisDomain => "POINT_RESPONSE_BASIS_DOMAIN",
            Self::BasisGrouping => "POINT_RESPONSE_BASIS_GROUPING",
        }
    }
}

/// One physical/global source row from the current callback's semantic source view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PointResponseSourceInput {
    source_index: CodecSourceIndex,
    xyz_f32_bits: [u32; 3],
    original_audio_width_f32_bits: u32,
    feature_mode: u32,
    source_skip: u32,
    spatial_enum: u32,
    coordinate_parameter: u32,
}

impl PointResponseSourceInput {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        source_index: CodecSourceIndex,
        xyz_f32_bits: [u32; 3],
        original_audio_width_f32_bits: u32,
        feature_mode: u32,
        source_skip: u32,
        spatial_enum: u32,
        coordinate_parameter: u32,
    ) -> Self {
        Self {
            source_index,
            xyz_f32_bits,
            original_audio_width_f32_bits,
            feature_mode,
            source_skip,
            spatial_enum,
            coordinate_parameter,
        }
    }

    pub const fn source_index(self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn xyz_f32_bits(self) -> [u32; 3] {
        self.xyz_f32_bits
    }

    pub const fn original_audio_width_f32_bits(self) -> u32 {
        self.original_audio_width_f32_bits
    }

    pub const fn feature_mode(self) -> u32 {
        self.feature_mode
    }

    pub const fn source_skip(self) -> u32 {
        self.source_skip
    }

    pub const fn spatial_enum(self) -> u32 {
        self.spatial_enum
    }

    pub const fn coordinate_parameter(self) -> u32 {
        self.coordinate_parameter
    }
}

/// Callback-bound F01E source view, including its normal or held-drain origin.
#[must_use]
#[derive(Debug)]
pub struct PointResponseSourceSnapshot {
    callback_index: FrontendCallbackIndex,
    transaction_id: CallbackLineageIdentity,
    source_control_clock: SourceControlSnapshotClock,
    rows: [PointResponseSourceInput; SOURCE_COUNT],
}

impl PointResponseSourceSnapshot {
    pub fn new(
        callback_index: FrontendCallbackIndex,
        transaction_id: CallbackLineageIdentity,
        source_control_clock: SourceControlSnapshotClock,
        source_rows: Vec<PointResponseSourceInput>,
    ) -> Result<Self, PointResponseError> {
        if transaction_id.callback_index() != callback_index
            || source_control_clock.callback_index() != callback_index
        {
            return Err(PointResponseError::SourceGateIdentityMismatch);
        }
        if source_rows.len() != SOURCE_COUNT {
            return Err(PointResponseError::SourceCount);
        }
        let mut by_source: [Option<PointResponseSourceInput>; SOURCE_COUNT] = [None; SOURCE_COUNT];
        for source in source_rows {
            let ordinal = usize::try_from(source.source_index().value())
                .map_err(|_| PointResponseError::SourceIdentity)?;
            let slot = by_source
                .get_mut(ordinal)
                .ok_or(PointResponseError::SourceIdentity)?;
            if slot.replace(source).is_some() {
                return Err(PointResponseError::SourceIdentity);
            }
        }
        let mut sorted_rows = Vec::with_capacity(SOURCE_COUNT);
        for source in by_source {
            sorted_rows.push(source.ok_or(PointResponseError::SourceIdentity)?);
        }
        let rows = sorted_rows
            .try_into()
            .map_err(|_| PointResponseError::SourceCount)?;
        Ok(Self {
            callback_index,
            transaction_id,
            source_control_clock,
            rows,
        })
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub const fn source_control_clock(&self) -> SourceControlSnapshotClock {
        self.source_control_clock
    }

    pub const fn rows(&self) -> &[PointResponseSourceInput; SOURCE_COUNT] {
        &self.rows
    }
}

/// Candidate and processing IDs produced by the same callback's feature gate.
#[must_use]
#[derive(Debug)]
pub struct PointResponseGateInput {
    callback_index: FrontendCallbackIndex,
    transaction_id: CallbackLineageIdentity,
    candidate_ids: Box<[u32]>,
    processing_ids: Box<[u32]>,
}

impl PointResponseGateInput {
    pub fn new(
        callback_index: FrontendCallbackIndex,
        transaction_id: CallbackLineageIdentity,
        candidate_ids: Vec<u32>,
        processing_ids: Vec<u32>,
    ) -> Result<Self, PointResponseError> {
        if transaction_id.callback_index() != callback_index {
            return Err(PointResponseError::SourceGateIdentityMismatch);
        }
        validate_source_ids(&candidate_ids)?;
        validate_source_ids(&processing_ids)?;
        Ok(Self {
            callback_index,
            transaction_id,
            candidate_ids: candidate_ids.into_boxed_slice(),
            processing_ids: processing_ids.into_boxed_slice(),
        })
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub fn candidate_ids(&self) -> &[u32] {
        &self.candidate_ids
    }

    pub fn processing_ids(&self) -> &[u32] {
        &self.processing_ids
    }
}

/// A source whose fields passed selected F03P branch checks and Q15 conversion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PointResponseSourceRequest {
    source_index: CodecSourceIndex,
    quantized_xyz_q15: [i32; 3],
    quantized_xyz_f32_bits: [u32; 3],
    original_audio_width_f32_bits: u32,
    post_audio_feature_width_f32_bits: u32,
}

impl PointResponseSourceRequest {
    pub const fn source_index(self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn quantized_xyz_f32_bits(self) -> [u32; 3] {
        self.quantized_xyz_f32_bits
    }

    pub const fn quantized_xyz_q15(self) -> [i32; 3] {
        self.quantized_xyz_q15
    }

    pub const fn original_audio_width_f32_bits(self) -> u32 {
        self.original_audio_width_f32_bits
    }

    pub const fn post_audio_feature_width_f32_bits(self) -> u32 {
        self.post_audio_feature_width_f32_bits
    }
}

/// Sorted, deduplicated callback-local request set for a later P15 response producer.
#[must_use]
#[derive(Debug)]
pub struct PointResponseSourceBatch {
    callback_index: FrontendCallbackIndex,
    transaction_id: CallbackLineageIdentity,
    source_control_clock: SourceControlSnapshotClock,
    candidate_ids: Box<[u32]>,
    processing_ids: Box<[u32]>,
    response_source_ids: Box<[CodecSourceIndex]>,
    generated_sources: Box<[PointResponseSourceRequest]>,
    candidate_feature_width_f32_bits: Box<[u32]>,
}

impl PointResponseSourceBatch {
    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub const fn source_control_clock(&self) -> SourceControlSnapshotClock {
        self.source_control_clock
    }

    pub fn candidate_ids(&self) -> &[u32] {
        &self.candidate_ids
    }

    pub fn processing_ids(&self) -> &[u32] {
        &self.processing_ids
    }

    pub fn response_source_ids(&self) -> &[CodecSourceIndex] {
        &self.response_source_ids
    }

    pub fn generated_sources(&self) -> &[PointResponseSourceRequest] {
        &self.generated_sources
    }

    pub fn candidate_feature_width_f32_bits(&self) -> &[u32] {
        &self.candidate_feature_width_f32_bits
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PointResponseColumn {
    source_index: CodecSourceIndex,
    response_f32_bits: [u32; SELECTED_BASIS_COUNT],
}

impl PointResponseColumn {
    pub const fn source_index(self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn response_f32_bits(self) -> [u32; SELECTED_BASIS_COUNT] {
        self.response_f32_bits
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointResponseAngleMode {
    MathematicalR64R32,
}

/// Immutable response columns keyed by global source ID and stable P15 basis order.
#[must_use]
pub struct PointResponseProduction {
    callback_index: FrontendCallbackIndex,
    transaction_id: CallbackLineageIdentity,
    source_control_clock: SourceControlSnapshotClock,
    basis_identity: SelectedP15GeometryIdentity,
    angle_mode: PointResponseAngleMode,
    candidate_ids: Box<[u32]>,
    processing_ids: Box<[u32]>,
    response_source_ids: Box<[CodecSourceIndex]>,
    columns: Box<[PointResponseColumn]>,
    candidate_feature_width_f32_bits: Box<[u32]>,
}

impl PointResponseProduction {
    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub const fn source_control_clock(&self) -> SourceControlSnapshotClock {
        self.source_control_clock
    }

    pub const fn basis_identity(&self) -> SelectedP15GeometryIdentity {
        self.basis_identity
    }

    pub const fn angle_mode(&self) -> PointResponseAngleMode {
        self.angle_mode
    }

    pub fn candidate_ids(&self) -> &[u32] {
        &self.candidate_ids
    }

    pub fn processing_ids(&self) -> &[u32] {
        &self.processing_ids
    }

    pub fn response_source_ids(&self) -> &[CodecSourceIndex] {
        &self.response_source_ids
    }

    pub fn columns(&self) -> &[PointResponseColumn] {
        &self.columns
    }

    pub fn candidate_feature_width_f32_bits(&self) -> &[u32] {
        &self.candidate_feature_width_f32_bits
    }
}

/// Admits exactly 48 source records, then processes only the candidate/processing union.
pub fn prepare_point_response_sources(
    source_snapshot: PointResponseSourceSnapshot,
    gate: PointResponseGateInput,
) -> Result<PointResponseSourceBatch, PointResponseError> {
    let PointResponseGateInput {
        callback_index: gate_callback_index,
        transaction_id: gate_transaction_id,
        candidate_ids,
        processing_ids,
    } = gate;
    if source_snapshot.callback_index != gate_callback_index
        || source_snapshot.transaction_id != gate_transaction_id
    {
        return Err(PointResponseError::SourceGateIdentityMismatch);
    }

    for source in &source_snapshot.rows {
        if source.feature_mode() != SELECTED_FEATURE_MODE {
            return Err(PointResponseError::SourceMode);
        }
        for word in source
            .xyz_f32_bits()
            .into_iter()
            .chain([source.original_audio_width_f32_bits()])
        {
            if !f32::from_bits(word).is_finite() {
                return Err(PointResponseError::NonFiniteInput);
            }
        }
    }

    let response_source_ids = merge_source_ids(&candidate_ids, &processing_ids)?;
    let candidate_count = candidate_ids.len();
    let mut generated_sources = Vec::with_capacity(response_source_ids.len());
    for source_index in response_source_ids.iter().copied() {
        let ordinal = usize::try_from(source_index.value())
            .map_err(|_| PointResponseError::SourceIdentity)?;
        let source = source_snapshot
            .rows
            .get(ordinal)
            .ok_or(PointResponseError::SourceIdentity)?;
        if source.source_skip() != 0
            || source.spatial_enum() != SELECTED_SPATIAL_ENUM
            || source.coordinate_parameter() > 1
        {
            return Err(PointResponseError::GeneratedSourceBranch);
        }

        let mut quantized_xyz_q15 = [0; 3];
        let mut quantized_xyz_f32_bits = [0; 3];
        for (axis, coordinate_bits) in source.xyz_f32_bits().into_iter().enumerate() {
            let coordinate = if axis == 2 && source.coordinate_parameter() == 0 {
                0.0
            } else {
                f32::from_bits(coordinate_bits)
            };
            let quantized = quantize_q15_coordinate(coordinate)
                .map_err(|_| PointResponseError::NumericFailure)?;
            quantized_xyz_q15[axis] = quantized.value();
            quantized_xyz_f32_bits[axis] = normalize_q15_coordinate(quantized).to_bits();
        }
        generated_sources.push(PointResponseSourceRequest {
            source_index,
            quantized_xyz_q15,
            quantized_xyz_f32_bits,
            original_audio_width_f32_bits: source.original_audio_width_f32_bits(),
            post_audio_feature_width_f32_bits: POST_AUDIO_FEATURE_WIDTH_POSITIVE_ZERO,
        });
    }

    Ok(PointResponseSourceBatch {
        callback_index: source_snapshot.callback_index,
        transaction_id: source_snapshot.transaction_id,
        source_control_clock: source_snapshot.source_control_clock,
        candidate_ids,
        processing_ids,
        response_source_ids,
        generated_sources: generated_sources.into_boxed_slice(),
        candidate_feature_width_f32_bits: vec![
            POST_AUDIO_FEATURE_WIDTH_POSITIVE_ZERO;
            candidate_count
        ]
        .into_boxed_slice(),
    })
}

/// Computes only the requested source columns against the selected stable P15 basis.
pub fn produce_point_response_columns(
    geometry: &SelectedP15Geometry,
    source_batch: PointResponseSourceBatch,
) -> Result<PointResponseProduction, PointResponseError> {
    let basis_identity = validate_selected_p15_geometry(geometry)?;

    let columns = if source_batch.generated_sources.is_empty() {
        Vec::new()
    } else {
        let planes = group_p15_basis(geometry)?;
        source_batch
            .response_source_ids
            .iter()
            .copied()
            .zip(source_batch.generated_sources.iter().copied())
            .map(|(source_index, source)| {
                if source_index != source.source_index() {
                    return Err(PointResponseError::SourceIdentity);
                }
                let response_f32_bits = produce_source_column(geometry, &planes, source)?;
                Ok(PointResponseColumn {
                    source_index,
                    response_f32_bits,
                })
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    Ok(PointResponseProduction {
        callback_index: source_batch.callback_index,
        transaction_id: source_batch.transaction_id,
        source_control_clock: source_batch.source_control_clock,
        basis_identity,
        angle_mode: PointResponseAngleMode::MathematicalR64R32,
        candidate_ids: source_batch.candidate_ids,
        processing_ids: source_batch.processing_ids,
        response_source_ids: source_batch.response_source_ids,
        columns: columns.into_boxed_slice(),
        candidate_feature_width_f32_bits: source_batch.candidate_feature_width_f32_bits,
    })
}

/// Computes one target-context column through the same selected P15 quantization and angle path.
///
/// This is crate-private because the public source-response API also requires source identities,
/// callback gates, and source-control lineage that target context does not own.
pub(crate) fn produce_target_context_response_column(
    geometry: &SelectedP15Geometry,
    xyz_f32_bits: [u32; 3],
) -> Result<[u32; SELECTED_BASIS_COUNT], PointResponseError> {
    validate_selected_p15_geometry(geometry)?;
    if xyz_f32_bits
        .iter()
        .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(PointResponseError::NonFiniteInput);
    }

    let mut quantized_xyz_q15 = [0; 3];
    let mut quantized_xyz_f32_bits = [0; 3];
    for (axis, coordinate_bits) in xyz_f32_bits.into_iter().enumerate() {
        let quantized = quantize_q15_coordinate(f32::from_bits(coordinate_bits))
            .map_err(|_| PointResponseError::NumericFailure)?;
        quantized_xyz_q15[axis] = quantized.value();
        quantized_xyz_f32_bits[axis] = normalize_q15_coordinate(quantized).to_bits();
    }

    let planes = group_p15_basis(geometry)?;
    produce_quantized_response_column(
        geometry,
        &planes,
        quantized_xyz_f32_bits,
        quantized_xyz_q15[2],
    )
}

fn validate_selected_p15_geometry(
    geometry: &SelectedP15Geometry,
) -> Result<SelectedP15GeometryIdentity, PointResponseError> {
    let basis_identity = geometry.identity();
    if basis_identity.layout() != 0x159b
        || !basis_identity.profile_enabled()
        || geometry.front_side_y_q15() != 5_285
        || geometry.points().len() != SELECTED_BASIS_COUNT
    {
        return Err(PointResponseError::BasisDomain);
    }
    Ok(basis_identity)
}

struct PlaneLayer {
    basis_indices: Vec<usize>,
    layer_y: FiniteF32,
}

struct P15PlaneGroups {
    lower: Vec<PlaneLayer>,
    upper: Vec<PlaneLayer>,
}

#[derive(Clone, Copy)]
struct WeightedLayerIndex {
    layer_index: usize,
    weight: FiniteF32,
}

#[derive(Clone, Copy)]
struct WeightedBasisIndex {
    basis_index: usize,
    weight: FiniteF32,
}

fn group_p15_basis(geometry: &SelectedP15Geometry) -> Result<P15PlaneGroups, PointResponseError> {
    let lower_min = finite_word(LOWER_PLANE_Z_EXCLUSIVE_BITS)?;
    let lower_max = finite_word(LOWER_PLANE_Z_INCLUSIVE_MAX_BITS)?;
    let upper_min = finite_word(UPPER_PLANE_Z_MIN_BITS)?;
    let mut lower = Vec::new();
    let mut upper = Vec::new();
    for (basis_index, point) in geometry.points().iter().enumerate() {
        let xyz = point.xyz_f32_bits();
        for word in xyz {
            finite_word(word)?;
        }
        let z = finite_word(xyz[2])?;
        if z.value() > upper_min.value() {
            upper.push(basis_index);
        } else if z.value() > lower_min.value() && z.value() <= lower_max.value() {
            lower.push(basis_index);
        } else {
            return Err(PointResponseError::BasisDomain);
        }
    }
    if lower.len() != SELECTED_LOWER_BASIS_COUNT || upper.len() != SELECTED_UPPER_BASIS_COUNT {
        return Err(PointResponseError::BasisDomain);
    }
    Ok(P15PlaneGroups {
        lower: group_plane_layers(geometry, &lower)?,
        upper: group_plane_layers(geometry, &upper)?,
    })
}

fn group_plane_layers(
    geometry: &SelectedP15Geometry,
    plane_indices: &[usize],
) -> Result<Vec<PlaneLayer>, PointResponseError> {
    let tolerance = finite_word(GROUPING_Y_TOLERANCE_BITS)?;
    let mut pending = plane_indices.to_vec();
    let mut layers = Vec::new();
    while let Some(anchor_index) = pending.first().copied() {
        let anchor_y = basis_coordinate(geometry, anchor_index, 1)?;
        let mut members = Vec::new();
        let mut remaining = Vec::new();
        for basis_index in pending.iter().copied() {
            let y = basis_coordinate(geometry, basis_index, 1)?;
            let difference = subtract(y, anchor_y)?;
            if difference.value().abs() < tolerance.value() {
                members.push(basis_index);
            } else {
                remaining.push(basis_index);
            }
        }
        if members.is_empty() {
            return Err(PointResponseError::BasisGrouping);
        }
        members.sort_by(|left, right| {
            let left_x =
                basis_coordinate(geometry, *left, 0).expect("admitted P15 X coordinate is finite");
            let right_x =
                basis_coordinate(geometry, *right, 0).expect("admitted P15 X coordinate is finite");
            left_x.value().total_cmp(&right_x.value())
        });
        let first_member = members[0];
        layers.push(PlaneLayer {
            basis_indices: members,
            layer_y: basis_coordinate(geometry, first_member, 1)?,
        });
        pending = remaining;
    }
    layers.sort_by(|left, right| left.layer_y.value().total_cmp(&right.layer_y.value()));
    if layers.is_empty() {
        return Err(PointResponseError::BasisGrouping);
    }
    Ok(layers)
}

fn produce_source_column(
    geometry: &SelectedP15Geometry,
    planes: &P15PlaneGroups,
    source: PointResponseSourceRequest,
) -> Result<[u32; SELECTED_BASIS_COUNT], PointResponseError> {
    produce_quantized_response_column(
        geometry,
        planes,
        source.quantized_xyz_f32_bits(),
        source.quantized_xyz_q15()[2],
    )
}

fn produce_quantized_response_column(
    geometry: &SelectedP15Geometry,
    planes: &P15PlaneGroups,
    xyz: [u32; 3],
    quantized_z_q15: i32,
) -> Result<[u32; SELECTED_BASIS_COUNT], PointResponseError> {
    let x = finite_word(xyz[0])?;
    let y = finite_word(xyz[1])?;
    let z = finite_word(xyz[2])?;
    let lower = plane_weights(geometry, planes.lower.as_slice(), x, y)?;
    let upper = plane_weights(geometry, planes.upper.as_slice(), x, y)?;
    let (lower_height, upper_height) = height_weights(quantized_z_q15, z)?;
    let threshold = finite_word(RESPONSE_THRESHOLD_BITS)?;
    let mut response = [0; SELECTED_BASIS_COUNT];
    for basis_index in 0..SELECTED_BASIS_COUNT {
        let lower_contribution = multiply(lower[basis_index], lower_height)?;
        let upper_contribution = multiply(upper[basis_index], upper_height)?;
        let value = add(lower_contribution, upper_contribution)?;
        if value.value() < 0.0 {
            return Err(PointResponseError::NumericFailure);
        }
        response[basis_index] = threshold_response(value, threshold);
    }
    Ok(response)
}

fn plane_weights(
    geometry: &SelectedP15Geometry,
    layers: &[PlaneLayer],
    source_x: FiniteF32,
    source_y: FiniteF32,
) -> Result<[FiniteF32; SELECTED_BASIS_COUNT], PointResponseError> {
    let zero = finite_word(POST_AUDIO_FEATURE_WIDTH_POSITIVE_ZERO)?;
    let mut weights = [zero; SELECTED_BASIS_COUNT];
    let layer_weights = vertical_weights(layers, source_y)?;
    for weighted_layer in layer_weights.iter().flatten() {
        let layer = layers
            .get(weighted_layer.layer_index)
            .ok_or(PointResponseError::BasisGrouping)?;
        let x_weights = horizontal_weights(geometry, &layer.basis_indices, source_x)?;
        for weighted_point in x_weights.iter().flatten() {
            weights[weighted_point.basis_index] =
                multiply(weighted_point.weight, weighted_layer.weight)?;
        }
    }
    Ok(weights)
}

fn vertical_weights(
    layers: &[PlaneLayer],
    source_y: FiniteF32,
) -> Result<[Option<WeightedLayerIndex>; 2], PointResponseError> {
    let first = layers.first().ok_or(PointResponseError::BasisGrouping)?;
    let last_index = layers
        .len()
        .checked_sub(1)
        .ok_or(PointResponseError::BasisGrouping)?;
    let last = &layers[last_index];
    if source_y.value() <= first.layer_y.value() {
        return Ok([Some(weighted_layer(0, one())), None]);
    }
    if source_y.value() > last.layer_y.value() {
        return Ok([Some(weighted_layer(last_index, one())), None]);
    }
    for (left_index, pair) in layers.windows(2).enumerate() {
        let left = pair[0].layer_y;
        let right = pair[1].layer_y;
        if left.value() <= source_y.value() && source_y.value() <= right.value() {
            let (left_weight, right_weight) =
                equal_power_weights(fraction(source_y, left, right)?)?;
            return Ok([
                Some(weighted_layer(left_index, left_weight)),
                Some(weighted_layer(left_index + 1, right_weight)),
            ]);
        }
    }
    Err(PointResponseError::BasisGrouping)
}

fn horizontal_weights(
    geometry: &SelectedP15Geometry,
    basis_indices: &[usize],
    source_x: FiniteF32,
) -> Result<[Option<WeightedBasisIndex>; 2], PointResponseError> {
    let first = *basis_indices
        .first()
        .ok_or(PointResponseError::BasisGrouping)?;
    let last = *basis_indices
        .last()
        .ok_or(PointResponseError::BasisGrouping)?;
    if source_x.value() <= basis_coordinate(geometry, first, 0)?.value() {
        return Ok([Some(weighted_point(first, one())), None]);
    }
    if source_x.value() >= basis_coordinate(geometry, last, 0)?.value() {
        return Ok([Some(weighted_point(last, one())), None]);
    }
    for pair in basis_indices.windows(2) {
        let left = basis_coordinate(geometry, pair[0], 0)?;
        let right = basis_coordinate(geometry, pair[1], 0)?;
        if left.value() < source_x.value() && source_x.value() <= right.value() {
            let (left_weight, right_weight) =
                equal_power_weights(fraction(source_x, left, right)?)?;
            return Ok([
                Some(weighted_point(pair[0], left_weight)),
                Some(weighted_point(pair[1], right_weight)),
            ]);
        }
    }
    Err(PointResponseError::BasisGrouping)
}

fn height_weights(
    q15_z: i32,
    normalized_z: FiniteF32,
) -> Result<(FiniteF32, FiniteF32), PointResponseError> {
    if q15_z <= 0 {
        Ok((one(), zero()))
    } else if q15_z >= Q15_MAX {
        Ok((zero(), one()))
    } else {
        equal_power_weights(normalized_z)
    }
}

fn fraction(
    value: FiniteF32,
    left: FiniteF32,
    right: FiniteF32,
) -> Result<FiniteF32, PointResponseError> {
    let numerator = subtract(value, left)?;
    let denominator = subtract(right, left)?;
    if denominator.value() <= 0.0 {
        return Err(PointResponseError::BasisGrouping);
    }
    FiniteF32::new(numerator.value() / denominator.value())
        .map_err(|_| PointResponseError::NumericFailure)
}

fn equal_power_weights(ratio: FiniteF32) -> Result<(FiniteF32, FiniteF32), PointResponseError> {
    if !(0.0..=1.0).contains(&ratio.value()) {
        return Err(PointResponseError::BasisDomain);
    }
    let (angle_ratio, swap) = if ratio.value() > 0.5 {
        (subtract(finite_word(ONE_BITS)?, ratio)?, true)
    } else {
        (ratio, false)
    };
    let angle = multiply(
        multiply(
            multiply(angle_ratio, finite_word(PI_OVER_FOUR_BITS)?)?,
            finite_word(HALF_BITS)?,
        )?,
        finite_word(FOUR_BITS)?,
    )?;
    let trig = exact_r64_to_r32(angle.value()).map_err(|_| PointResponseError::NumericFailure)?;
    let cosine = FiniteF32::new(trig.cosine).map_err(|_| PointResponseError::NumericFailure)?;
    let sine = FiniteF32::new(trig.sine).map_err(|_| PointResponseError::NumericFailure)?;
    Ok(if swap { (sine, cosine) } else { (cosine, sine) })
}

fn weighted_layer(layer_index: usize, weight: FiniteF32) -> WeightedLayerIndex {
    WeightedLayerIndex {
        layer_index,
        weight,
    }
}

fn weighted_point(basis_index: usize, weight: FiniteF32) -> WeightedBasisIndex {
    WeightedBasisIndex {
        basis_index,
        weight,
    }
}

fn basis_coordinate(
    geometry: &SelectedP15Geometry,
    basis_index: usize,
    axis: usize,
) -> Result<FiniteF32, PointResponseError> {
    let point = geometry
        .points()
        .get(basis_index)
        .ok_or(PointResponseError::BasisDomain)?;
    let coordinate = point
        .xyz_f32_bits()
        .get(axis)
        .copied()
        .ok_or(PointResponseError::BasisDomain)?;
    finite_word(coordinate)
}

fn finite_word(word: u32) -> Result<FiniteF32, PointResponseError> {
    FiniteF32::new(f32::from_bits(word)).map_err(|_| PointResponseError::NumericFailure)
}

fn zero() -> FiniteF32 {
    FiniteF32::new(0.0).expect("positive zero is finite")
}

fn one() -> FiniteF32 {
    FiniteF32::new(1.0).expect("one is finite")
}

fn threshold_response(value: FiniteF32, threshold: FiniteF32) -> u32 {
    if threshold.value() > value.value() {
        POST_AUDIO_FEATURE_WIDTH_POSITIVE_ZERO
    } else {
        value.to_bits()
    }
}

fn add(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, PointResponseError> {
    r32_add(left, right).map_err(map_numeric_error)
}

fn subtract(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, PointResponseError> {
    r32_sub(left, right).map_err(map_numeric_error)
}

fn multiply(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, PointResponseError> {
    r32_mul(left, right).map_err(map_numeric_error)
}

fn map_numeric_error(_: R32Error) -> PointResponseError {
    PointResponseError::NumericFailure
}

#[cfg(test)]
mod tests;

fn validate_source_ids(ids: &[u32]) -> Result<(), PointResponseError> {
    if ids.iter().any(|source| *source >= SOURCE_COUNT_U32)
        || ids.windows(2).any(|pair| pair[0] >= pair[1])
    {
        Err(PointResponseError::SourceIdentity)
    } else {
        Ok(())
    }
}

fn merge_source_ids(
    candidate_ids: &[u32],
    processing_ids: &[u32],
) -> Result<Box<[CodecSourceIndex]>, PointResponseError> {
    let capacity = candidate_ids
        .len()
        .checked_add(processing_ids.len())
        .ok_or(PointResponseError::SourceIdentity)?;
    let mut values = Vec::with_capacity(capacity);
    let (mut candidate, mut processing) = (0, 0);
    while candidate < candidate_ids.len() && processing < processing_ids.len() {
        match candidate_ids[candidate].cmp(&processing_ids[processing]) {
            std::cmp::Ordering::Less => {
                values.push(candidate_ids[candidate]);
                candidate += 1;
            }
            std::cmp::Ordering::Greater => {
                values.push(processing_ids[processing]);
                processing += 1;
            }
            std::cmp::Ordering::Equal => {
                values.push(candidate_ids[candidate]);
                candidate += 1;
                processing += 1;
            }
        }
    }
    values.extend_from_slice(&candidate_ids[candidate..]);
    values.extend_from_slice(&processing_ids[processing..]);
    values
        .into_iter()
        .map(|source| Ok(CodecSourceIndex::new(u64::from(source))))
        .collect::<Result<Vec<_>, PointResponseError>>()
        .map(Vec::into_boxed_slice)
}
