// pattern: Functional Core
//! Selected 11-point F04G prefix membership and gain production.
//!
//! The producer consumes one explicit 32-byte selected control record and a
//! supplied basis. It owns no callback state, I/O, or commit authority.

use sha2::{Digest, Sha256};

use crate::numeric::{
    FiniteF32, r32_add as numeric_r32_add, r32_mul as numeric_r32_mul, r32_sub as numeric_r32_sub,
    trunc_f32_to_i32,
};
use crate::spatial::selected_p11_geometry::SelectedP11Geometry;

const POINT_COUNT: usize = 11;
const INPUT_BYTES: usize = 32;
const Q15_SCALE: f32 = 32_768.0;
const Q15_MAX: i32 = 32_767;
const Q15_HALF_BIAS: f32 = 0.5;
const MAPPED_WIDTH_THRESHOLD: f32 = 0.000_025;
const MEMBERSHIP_GAIN_THRESHOLD: f32 = 0.000_001;
const MIN_NORMAL_BITS: u32 = 0x0080_0000;
const SQRT_HALF_BITS: u32 = 0x3f35_04f3;
const QUARTER_BITS: u32 = 0x3e80_0000;
const HALF_BITS: u32 = 0x3f00_0000;
const FOUR_BITS: u32 = 0x4080_0000;
const FIVE_BITS: u32 = 0x40a0_0000;
const PI_BITS: u32 = 0x4049_0fdb;
const PI_OVER_FOUR_BITS: u32 = 0x3f49_0fdb;
const PI_OVER_EIGHT_BITS: u32 = 0x3ec9_0fdb;
const SIN2_FULL_WIDTH_BITS: u32 = 0x3e4c_cccd;
const SIN2_EXPONENT_SCALE_BITS: u32 = 0xbf99_999a;
const SIN2_EXPONENT_DIVISOR_BITS: u32 = 0x41a0_0000;
const BASIC_NORM_FLOOR_BITS: u32 = 0x3727_c5ac;
const SPREAD_PARAMETER_GROUP_HASHES: [&str; 4] = [
    "046f0a6a56eb4df5f13840a9df11396576bfd1484f6ff5158acdf97523de1963",
    "39b06e6c918fa26a476f40383ef38eb1047d15439ee37e2aa5c21c7bab607ed0",
    "65a0e821be6b2e5be26a0c16d61f389da7bf5e703fde75a829ea470760c27f0d",
    "58f8111480a233047de740a50ab09c83ede9d79d90f3dea616b026807154d592",
];

const D10_W: [u32; 10] = [
    0x0000_0000,
    0x0000_0000,
    0x3e4c_cccd,
    0x3d99_999a,
    0x3f00_0000,
    0x3e80_0000,
    0x3f40_0000,
    0x3ee6_6666,
    0x3f80_0000,
    0x3f33_3333,
];
const D10_WK: [u32; 2] = [0x3f22_0000, 0x41d4_9a78];
const D10_PC: [u32; 7] = [
    0x40c0_afdb,
    0x4048_907a,
    0x4083_c679,
    0x3f9d_9b95,
    0x3f2f_b000,
    0x3e82_7800,
    0x3d71_4000,
];
const D10_SC: [u32; 5] = [
    0x40c0_0000,
    0x3e00_0000,
    0x40db_d37a,
    0x40de_9bd4,
    0x3eae_d623,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrefixMembershipError {
    Shape,
    ControlDomain,
    InputNonfinite,
    ParameterUnavailable,
    ParameterDomain,
    MappedWidthBelowThreshold,
}

impl PrefixMembershipError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Shape => "SHAPE",
            Self::ControlDomain => "CONTROL_DOMAIN",
            Self::InputNonfinite => "INPUT_NONFINITE",
            Self::ParameterUnavailable => "PARAMETER_UNAVAILABLE",
            Self::ParameterDomain => "PARAMETER_DOMAIN",
            Self::MappedWidthBelowThreshold => "MAPPED_WIDTH_BELOW_THRESHOLD",
        }
    }
}

/// The four authorized parameter families, represented by their frozen F32 words.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PrefixMembershipParameters {
    width_pairs: [u32; 10],
    kernel_coefficients: [u32; 2],
    log_exp_coefficients: [u32; 7],
    power_z_parameters: [u32; 5],
}

impl PrefixMembershipParameters {
    const fn selected_authorized() -> Self {
        Self {
            width_pairs: D10_W,
            kernel_coefficients: D10_WK,
            log_exp_coefficients: D10_PC,
            power_z_parameters: D10_SC,
        }
    }

    fn verify_authorized_words(&self) -> Result<(), PrefixMembershipError> {
        let groups: [&[u32]; 4] = [
            &self.width_pairs,
            &self.kernel_coefficients,
            &self.log_exp_coefficients,
            &self.power_z_parameters,
        ];
        for (words, expected_hash) in groups.into_iter().zip(SPREAD_PARAMETER_GROUP_HASHES) {
            let bytes = words
                .iter()
                .flat_map(|word| word.to_le_bytes())
                .collect::<Vec<_>>();
            if hex_lower(&Sha256::digest(bytes)) != expected_hash {
                return Err(PrefixMembershipError::ParameterDomain);
            }
            if words.iter().any(|word| !f32::from_bits(*word).is_finite()) {
                return Err(PrefixMembershipError::ParameterDomain);
            }
        }
        Ok(())
    }

    fn width_pair(&self, index: usize) -> (f32, f32) {
        (
            f32::from_bits(self.width_pairs[index * 2]),
            f32::from_bits(self.width_pairs[index * 2 + 1]),
        )
    }

    fn kernel_coefficients(&self) -> (f32, f32) {
        (
            f32::from_bits(self.kernel_coefficients[0]),
            f32::from_bits(self.kernel_coefficients[1]),
        )
    }

    fn log_coefficients(&self) -> [f32; 4] {
        [
            f32::from_bits(self.log_exp_coefficients[0]),
            f32::from_bits(self.log_exp_coefficients[1]),
            f32::from_bits(self.log_exp_coefficients[2]),
            f32::from_bits(self.log_exp_coefficients[3]),
        ]
    }

    fn exp_coefficients(&self) -> [f32; 3] {
        [
            f32::from_bits(self.log_exp_coefficients[4]),
            f32::from_bits(self.log_exp_coefficients[5]),
            f32::from_bits(self.log_exp_coefficients[6]),
        ]
    }

    fn power_and_z(&self) -> [f32; 5] {
        self.power_z_parameters.map(f32::from_bits)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrefixMembershipCellOutput {
    axis: u8,
    plane: u8,
    position_index: u8,
    width_index: u8,
    words: Vec<u32>,
}

impl PrefixMembershipCellOutput {
    pub const fn axis(&self) -> u8 {
        self.axis
    }

    pub const fn plane(&self) -> u8 {
        self.plane
    }

    pub const fn position_index(&self) -> u8 {
        self.position_index
    }

    pub const fn width_index(&self) -> u8 {
        self.width_index
    }

    pub fn words(&self) -> &[u32] {
        &self.words
    }
}

/// Complete pure membership result. Words retain the exact F32 bit patterns.
#[derive(Clone, Debug, PartialEq)]
pub struct PrefixMembershipResult {
    q15_xyz: [i32; 3],
    q15_width: i32,
    mapped_width_word: u32,
    axis_words: [[u32; POINT_COUNT]; 3],
    boundary_words: [u32; POINT_COUNT],
    interior_words: [u32; POINT_COUNT],
    spread_words: [u32; POINT_COUNT],
    column_words: [u32; POINT_COUNT],
    sin_squared_word: u32,
    unadjusted_words: [u32; POINT_COUNT],
    gain_words: [u32; POINT_COUNT],
    positive_members: Vec<u8>,
    cells: Vec<PrefixMembershipCellOutput>,
}

impl PrefixMembershipResult {
    pub const fn q15_xyz(&self) -> &[i32; 3] {
        &self.q15_xyz
    }

    pub const fn q15_width(&self) -> i32 {
        self.q15_width
    }

    pub const fn mapped_width_word(&self) -> u32 {
        self.mapped_width_word
    }

    pub const fn axis_words(&self) -> &[[u32; POINT_COUNT]; 3] {
        &self.axis_words
    }

    pub const fn boundary_words(&self) -> &[u32; POINT_COUNT] {
        &self.boundary_words
    }

    pub const fn interior_words(&self) -> &[u32; POINT_COUNT] {
        &self.interior_words
    }

    pub const fn spread_words(&self) -> &[u32; POINT_COUNT] {
        &self.spread_words
    }

    pub const fn column_words(&self) -> &[u32; POINT_COUNT] {
        &self.column_words
    }

    pub const fn sin_squared_word(&self) -> u32 {
        self.sin_squared_word
    }

    pub const fn unadjusted_words(&self) -> &[u32; POINT_COUNT] {
        &self.unadjusted_words
    }

    pub const fn gain_words(&self) -> &[u32; POINT_COUNT] {
        &self.gain_words
    }

    pub fn positive_members(&self) -> &[u8] {
        &self.positive_members
    }

    pub fn cells(&self) -> &[PrefixMembershipCellOutput] {
        &self.cells
    }
}

/// Stateless selected-profile evaluator for synthetic KATs and the primary P11 basis.
#[derive(Debug)]
pub struct PrefixMembershipProducer {
    parameters: PrefixMembershipParameters,
    basis_words: [[u32; 3]; POINT_COUNT],
    basis_f32le_sha256: [u8; 32],
}

impl PrefixMembershipProducer {
    fn new(
        parameters: Option<PrefixMembershipParameters>,
        basis_points: Option<[[f32; 3]; POINT_COUNT]>,
    ) -> Result<Self, PrefixMembershipError> {
        let parameters = parameters.ok_or(PrefixMembershipError::ParameterUnavailable)?;
        let basis_points = basis_points.ok_or(PrefixMembershipError::ParameterUnavailable)?;
        parameters.verify_authorized_words()?;
        let mut basis_words = [[0_u32; 3]; POINT_COUNT];
        for (row, point) in basis_words.iter_mut().zip(basis_points) {
            for (word, coordinate) in row.iter_mut().zip(point) {
                if !coordinate.is_finite() || !(0.0..=1.0).contains(&coordinate) {
                    return Err(PrefixMembershipError::ParameterDomain);
                }
                *word = coordinate.to_bits();
            }
        }
        validate_width_pairs(&parameters)?;
        let mut canonical_basis = Vec::with_capacity(POINT_COUNT * 3 * 4);
        for point in &basis_words {
            for word in point {
                canonical_basis.extend_from_slice(&word.to_le_bytes());
            }
        }
        let basis_f32le_sha256 = Sha256::digest(canonical_basis).into();
        Ok(Self {
            parameters,
            basis_words,
            basis_f32le_sha256,
        })
    }

    /// Constructs the producer only with the approved selected P11 basis.
    pub fn for_selected_primary_p11(
        geometry: &SelectedP11Geometry,
    ) -> Result<Self, PrefixMembershipError> {
        let basis = geometry
            .points()
            .map(|point| point.xyz_words().map(f32::from_bits));
        let producer = Self::new(
            Some(PrefixMembershipParameters::selected_authorized()),
            Some(basis),
        )?;
        if &producer.basis_f32le_sha256 != geometry.f32le_sha256() {
            return Err(PrefixMembershipError::ParameterDomain);
        }
        Ok(producer)
    }

    pub(crate) const fn basis_f32le_sha256(&self) -> &[u8; 32] {
        &self.basis_f32le_sha256
    }

    /// Produces gains from the F04G 32-byte input-only record.
    pub fn produce(&self, record: &[u8]) -> Result<PrefixMembershipResult, PrefixMembershipError> {
        if record.len() != INPUT_BYTES {
            return Err(PrefixMembershipError::Shape);
        }
        let words = record
            .chunks_exact(4)
            .map(|bytes| {
                bytes
                    .try_into()
                    .map(u32::from_le_bytes)
                    .map_err(|_| PrefixMembershipError::Shape)
            })
            .collect::<Result<Vec<_>, PrefixMembershipError>>()?;
        let xyz_words: [u32; 3] = words[..3]
            .try_into()
            .map_err(|_| PrefixMembershipError::Shape)?;
        let mode = words[3];
        let raw_width = f32::from_bits(words[4]);
        let component_enum = words[6];
        let enabled = words[7];
        if mode != 0
            || component_enum != 0
            || enabled != 1
            || !raw_width.is_finite()
            || raw_width <= 0.0
            || raw_width > 1.0
        {
            return Err(PrefixMembershipError::ControlDomain);
        }
        let xyz = xyz_words.map(f32::from_bits);
        if xyz.iter().any(|coordinate| !coordinate.is_finite()) {
            return Err(PrefixMembershipError::InputNonfinite);
        }
        if xyz
            .iter()
            .any(|coordinate| !(0.0..=1.0).contains(coordinate))
        {
            return Err(PrefixMembershipError::ControlDomain);
        }

        let q15_xyz: [i32; 3] = xyz
            .into_iter()
            .map(quantize_q15)
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| PrefixMembershipError::ParameterDomain)?;
        let q15_width = quantize_q15(raw_width)?;
        let quantized_xyz = q15_xyz
            .map(q15_to_f32)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        let quantized_xyz: [f32; 3] = quantized_xyz
            .try_into()
            .map_err(|_| PrefixMembershipError::ParameterDomain)?;
        let mapped_width = map_width(&self.parameters, q15_to_f32(q15_width)?)?;
        if mapped_width < MAPPED_WIDTH_THRESHOLD {
            return Err(PrefixMembershipError::MappedWidthBelowThreshold);
        }
        compute_gain(
            &self.parameters,
            self.basis_words,
            q15_xyz,
            q15_width,
            quantized_xyz,
            raw_width,
            mapped_width,
        )
    }
}

fn validate_width_pairs(
    parameters: &PrefixMembershipParameters,
) -> Result<(), PrefixMembershipError> {
    let mut previous = f32::NEG_INFINITY;
    for pair in 0..5 {
        let (input, mapped) = parameters.width_pair(pair);
        if !input.is_finite() || !mapped.is_finite() || !(0.0..=1.0).contains(&input) {
            return Err(PrefixMembershipError::ParameterDomain);
        }
        if input <= previous && (pair != 0 || input.to_bits() != 0.0_f32.to_bits()) {
            return Err(PrefixMembershipError::ParameterDomain);
        }
        previous = input;
    }
    if parameters.width_pair(0).0.to_bits() != 0.0_f32.to_bits()
        || parameters.width_pair(4).0.to_bits() != 1.0_f32.to_bits()
    {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    Ok(())
}

fn quantize_q15(value: f32) -> Result<i32, PrefixMembershipError> {
    let scaled = r32_mul(value, Q15_SCALE)?;
    let biased = r32_add(scaled, Q15_HALF_BIAS)?;
    Ok((biased.trunc() as i32).min(Q15_MAX))
}

fn q15_to_f32(value: i32) -> Result<f32, PrefixMembershipError> {
    r32_div(value as f32, Q15_SCALE)
}

fn map_width(
    parameters: &PrefixMembershipParameters,
    width: f32,
) -> Result<f32, PrefixMembershipError> {
    if width <= 0.0 {
        return Ok(0.0);
    }
    if width >= 1.0 {
        return Ok(1.0);
    }
    for pair in 0..4 {
        let (left_input, left_mapped) = parameters.width_pair(pair);
        let (right_input, right_mapped) = parameters.width_pair(pair + 1);
        if width <= right_input {
            let numerator = r32_sub(width, left_input)?;
            let denominator = r32_sub(right_input, left_input)?;
            let fraction = r32_div(numerator, denominator)?;
            let range = r32_sub(right_mapped, left_mapped)?;
            return r32_add(r32_mul(fraction, range)?, left_mapped);
        }
    }
    Err(PrefixMembershipError::ParameterDomain)
}

fn power(
    parameters: &PrefixMembershipParameters,
    mapped_width: f32,
) -> Result<f32, PrefixMembershipError> {
    let [power_low, breakpoint, intercept, slope, _] = parameters.power_and_z();
    if mapped_width <= breakpoint {
        Ok(power_low)
    } else {
        r32_sub(intercept, r32_mul(mapped_width, slope)?)
    }
}

fn pow_value(
    parameters: &PrefixMembershipParameters,
    value: f32,
    exponent: f32,
) -> Result<f32, PrefixMembershipError> {
    if value == 0.0 || value.abs() < f32::from_bits(MIN_NORMAL_BITS) {
        return Ok(0.0);
    }
    if value <= 0.0 || !value.is_finite() || !exponent.is_finite() {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    let logarithm = log2_approx(parameters, value)?;
    let exponential = exp2_approx(parameters, r32_mul(logarithm, exponent)?)?;
    if exponential.is_finite() {
        Ok(exponential)
    } else {
        Err(PrefixMembershipError::ParameterDomain)
    }
}

fn compute_gain(
    parameters: &PrefixMembershipParameters,
    basis_words: [[u32; 3]; POINT_COUNT],
    q15_xyz: [i32; 3],
    q15_width: i32,
    quantized_xyz: [f32; 3],
    raw_width: f32,
    mapped_width: f32,
) -> Result<PrefixMembershipResult, PrefixMembershipError> {
    let basis_points = basis_words.map(|point| point.map(f32::from_bits));
    let planes = build_planes(&basis_points)?;
    let sample_grids = [
        repeated_grid(20, r32_div(1.0, 19.0)?)?,
        repeated_grid(20, r32_div(1.0, 19.0)?)?,
        repeated_grid(8, r32_div(1.0, 7.0)?)?,
    ];
    let position_grids = [
        repeated_grid(35, r32_div(1.0, 34.0)?)?,
        repeated_grid(35, r32_div(1.0, 34.0)?)?,
        repeated_grid(4, r32_div(1.0, 3.0)?)?,
    ];
    let width_grid = repeated_grid(20, r32_div(0.7, 19.0)?)?;
    let position_descriptors = [
        position_descriptor(quantized_xyz[0], 35, r32_div(1.0, 34.0)?)?,
        position_descriptor(quantized_xyz[1], 35, r32_div(1.0, 34.0)?)?,
        position_descriptor(quantized_xyz[2], 4, r32_div(1.0, 3.0)?)?,
    ];
    let width_descriptor = width_descriptor(mapped_width)?;
    let p = power(parameters, mapped_width)?;
    let mut cells = Vec::with_capacity(24);
    let mut cell_words = Vec::with_capacity(24);

    for axis in 0..3 {
        let descriptor = position_descriptors[axis];
        let position_indexes = [descriptor.left, descriptor.right];
        let width_indexes = [width_descriptor.left, width_descriptor.right];
        for (plane_index, plane) in planes.iter().enumerate() {
            for width_index in width_indexes {
                for position_index in position_indexes {
                    let center = position_grids[axis][position_index];
                    let cell_width = width_grid[width_index];
                    let cell_power = power(parameters, cell_width)?;
                    let context = CellContext {
                        parameters,
                        axis,
                        planes: &planes,
                        basis: &basis_points,
                        samples: &sample_grids[axis],
                        center,
                        width: cell_width,
                        exponent: cell_power,
                    };
                    let words = compute_cell(&context, plane)?;
                    cell_words.push(words.clone());
                    cells.push(PrefixMembershipCellOutput {
                        axis: u8::try_from(axis)
                            .map_err(|_| PrefixMembershipError::ParameterDomain)?,
                        plane: u8::try_from(plane_index)
                            .map_err(|_| PrefixMembershipError::ParameterDomain)?,
                        position_index: u8::try_from(position_index)
                            .map_err(|_| PrefixMembershipError::ParameterDomain)?,
                        width_index: u8::try_from(width_index)
                            .map_err(|_| PrefixMembershipError::ParameterDomain)?,
                        words: words.into_iter().map(f32::to_bits).collect(),
                    });
                }
            }
        }
    }

    let mut axis_values = [[0.0_f32; POINT_COUNT]; 3];
    for axis in 0..3 {
        let position_fraction = position_descriptors[axis].fraction;
        let mix = interpolation_weights(position_fraction, width_descriptor.fraction)?;
        for (plane_index, plane) in planes.iter().enumerate() {
            let base = (axis * 2 + plane_index) * 4;
            for (point_offset, point_index) in plane.points.iter().copied().enumerate() {
                let values = if axis == 2 {
                    [
                        cell_words[base][0],
                        cell_words[base + 1][0],
                        cell_words[base + 2][0],
                        cell_words[base + 3][0],
                    ]
                } else {
                    [
                        cell_words[base][point_offset],
                        cell_words[base + 1][point_offset],
                        cell_words[base + 2][point_offset],
                        cell_words[base + 3][point_offset],
                    ]
                };
                axis_values[axis][point_index] = mix_four(values, mix)?;
            }
        }
    }
    let axis_words = axis_values.map(|axis| axis.map(f32::to_bits));

    let boundary_words = normalize_boundary(
        parameters,
        &basis_points,
        &planes,
        quantized_xyz,
        mapped_width,
        p,
        &axis_values,
    )?;
    let interior_products: [Result<f32, PrefixMembershipError>; POINT_COUNT] =
        std::array::from_fn(|index| {
            r32_mul(
                r32_mul(axis_values[1][index], axis_values[0][index])?,
                axis_values[2][index],
            )
        });
    let interior_values = normalize_basic(
        interior_products
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?,
    )?;
    let interior_words = interior_values.map(f32::to_bits);
    let boundary_values = boundary_words.map(f32::from_bits);
    let interior_values = interior_words.map(f32::from_bits);
    let distance = min6(
        quantized_xyz[0],
        r32_sub(1.0, quantized_xyz[0])?,
        quantized_xyz[1],
        r32_sub(1.0, quantized_xyz[1])?,
        quantized_xyz[2],
        r32_sub(1.0, quantized_xyz[2])?,
    );
    let quarter = r32_mul(distance, f32::from_bits(QUARTER_BITS))?;
    let edge_weight = if quarter > mapped_width && quarter > 0.05 {
        1.0
    } else {
        let width4 = r32_mul(mapped_width, f32::from_bits(FOUR_BITS))?;
        let distance_over_width = r32_div(distance, width4)?;
        let distance5 = r32_mul(distance, f32::from_bits(FIVE_BITS))?;
        r32_mul(distance_over_width, r32_mul(distance5, distance5)?)?
    };
    let mut spread_input = Vec::with_capacity(POINT_COUNT);
    let reciprocal_power = m32(1.0_f64 / f64::from(p))?;
    for index in 0..POINT_COUNT {
        let mixed = r32_add(
            r32_mul(edge_weight, interior_values[index])?,
            boundary_values[index],
        )?;
        let half = r32_mul(mixed, f32::from_bits(HALF_BITS))?;
        let value = r32_mul(half, 0.0625)?;
        let powered = pow_value(parameters, value, reciprocal_power)?;
        spread_input.push(powered);
    }
    let spread_values = normalize_scaled(spread_input)?;
    let spread_words = spread_values.map(f32::to_bits);
    let column_values: [Result<f32, PrefixMembershipError>; POINT_COUNT] =
        std::array::from_fn(|index| r32_mul(r32_mul(spread_values[index], 0.125)?, 8.0));
    let column_values = column_values.into_iter().collect::<Result<Vec<_>, _>>()?;
    let column_words: [u32; POINT_COUNT] = column_values
        .iter()
        .map(|value| value.to_bits())
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| PrefixMembershipError::ParameterDomain)?;
    let sin_squared = sin_squared(raw_width)?;
    let unadjusted = column_values
        .iter()
        .map(|value| r32_mul(*value, sin_squared))
        .collect::<Result<Vec<_>, _>>()?;
    let unadjusted_words: [u32; POINT_COUNT] = unadjusted
        .iter()
        .map(|value| value.to_bits())
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| PrefixMembershipError::ParameterDomain)?;
    let final_gain = adjust_front_gain(raw_width, &unadjusted)?;
    let gain_words: [u32; POINT_COUNT] = final_gain
        .iter()
        .map(|value| value.to_bits())
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| PrefixMembershipError::ParameterDomain)?;
    let positive_members = final_gain
        .iter()
        .enumerate()
        .filter_map(|(index, gain)| {
            // The selected threshold is positive, so this also enforces gain > 0.
            (*gain >= MEMBERSHIP_GAIN_THRESHOLD)
                .then(|| u8::try_from(index).ok())
                .flatten()
        })
        .collect();

    Ok(PrefixMembershipResult {
        q15_xyz,
        q15_width,
        mapped_width_word: mapped_width.to_bits(),
        axis_words,
        boundary_words,
        interior_words,
        spread_words,
        column_words,
        sin_squared_word: sin_squared.to_bits(),
        unadjusted_words,
        gain_words,
        positive_members,
        cells,
    })
}

#[derive(Clone, Debug)]
struct MembershipLayer {
    points_by_x: Vec<usize>,
    layer_y: f32,
}

#[derive(Clone, Debug)]
struct MembershipPlane {
    points: Vec<usize>,
    layers: Vec<MembershipLayer>,
}

#[derive(Clone, Copy, Debug)]
struct GridDescriptor {
    left: usize,
    right: usize,
    fraction: f32,
}

fn build_planes(
    basis: &[[f32; 3]; POINT_COUNT],
) -> Result<[MembershipPlane; 2], PrefixMembershipError> {
    let mut lower = Vec::new();
    let mut upper = Vec::new();
    for (index, point) in basis.iter().enumerate() {
        if point[2] > -0.1 && point[2] <= 0.5 {
            lower.push(index);
        } else if point[2] > 0.5 {
            upper.push(index);
        } else {
            return Err(PrefixMembershipError::ParameterDomain);
        }
    }
    if lower.is_empty() || upper.is_empty() || lower.len() + upper.len() != POINT_COUNT {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    Ok([build_plane(lower, basis)?, build_plane(upper, basis)?])
}

fn build_plane(
    points: Vec<usize>,
    basis: &[[f32; 3]; POINT_COUNT],
) -> Result<MembershipPlane, PrefixMembershipError> {
    let mut pending = points.clone();
    let mut layers = Vec::new();
    while let Some(anchor) = pending.first().copied() {
        let anchor_y = basis[anchor][1];
        let mut members = Vec::new();
        let mut remaining = Vec::new();
        for point in pending {
            if (basis[point][1] - anchor_y).abs() < 0.01 {
                members.push(point);
            } else {
                remaining.push(point);
            }
        }
        members.sort_by(|left, right| {
            basis[*left][0]
                .total_cmp(&basis[*right][0])
                .then_with(|| left.cmp(right))
        });
        let layer_y = basis[*members
            .first()
            .ok_or(PrefixMembershipError::ParameterDomain)?][1];
        layers.push(MembershipLayer {
            points_by_x: members,
            layer_y,
        });
        pending = remaining;
    }
    if layers.is_empty()
        || layers
            .windows(2)
            .any(|pair| pair[0].layer_y >= pair[1].layer_y)
    {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    Ok(MembershipPlane { points, layers })
}

fn repeated_grid(count: usize, step: f32) -> Result<Vec<f32>, PrefixMembershipError> {
    if count == 0 || !step.is_finite() || step <= 0.0 {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    let mut values = Vec::with_capacity(count);
    values.push(0.0);
    for index in 1..count {
        values.push(r32_add(values[index - 1], step)?);
    }
    Ok(values)
}

fn position_descriptor(
    position: f32,
    count: usize,
    step: f32,
) -> Result<GridDescriptor, PrefixMembershipError> {
    if count == 0 || !(0.0..=1.0).contains(&position) {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    let scale = r32_mul((count - 1) as f32, position)?;
    let floor = scale.floor();
    let floor = trunc_f32_to_i32(finite(floor)?)
        .map_err(|_| PrefixMembershipError::ParameterDomain)?
        .value();
    let index = usize::try_from(floor)
        .map_err(|_| PrefixMembershipError::ParameterDomain)?
        .min(count - 1);
    let left = index.min(count - 1);
    let right = (index + 1).min(count - 1);
    let fraction = if left == right {
        0.0
    } else {
        let left_position = r32_mul(left as f32, step)?;
        r32_div(r32_sub(position, left_position)?, step)?.max(0.0)
    };
    Ok(GridDescriptor {
        left,
        right,
        fraction,
    })
}

fn width_descriptor(width: f32) -> Result<GridDescriptor, PrefixMembershipError> {
    if !(0.0..=1.0).contains(&width) {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    let five_sevenths = r32_div(5.0, 7.0)?;
    let scaled = r32_mul(r32_mul(r32_mul(width, five_sevenths)?, 2.0)?, 19.0)?;
    let floor = trunc_f32_to_i32(finite(scaled.floor())?)
        .map_err(|_| PrefixMembershipError::ParameterDomain)?
        .value();
    let index = usize::try_from(floor)
        .map_err(|_| PrefixMembershipError::ParameterDomain)?
        .min(19);
    let left = index.min(19);
    let right = (index + 1).min(19);
    let step = r32_div(0.7, 19.0)?;
    let fraction = if left == right {
        0.0
    } else {
        let left_width = r32_mul(left as f32, step)?;
        r32_div(r32_sub(width, left_width)?, step)?.max(0.0)
    };
    Ok(GridDescriptor {
        left,
        right,
        fraction,
    })
}

fn interpolation_weights(
    position_fraction: f32,
    width_fraction: f32,
) -> Result<[f32; 4], PrefixMembershipError> {
    let one_minus_width = r32_sub(1.0, width_fraction)?;
    let one_minus_position = r32_sub(1.0, position_fraction)?;
    let a = r32_sub(1.0, width_fraction.max(position_fraction))?;
    let b = r32_sub(1.0, width_fraction.max(one_minus_position))?;
    let c = r32_sub(1.0, one_minus_width.max(position_fraction))?;
    let d = r32_sub(1.0, one_minus_width.max(one_minus_position))?;
    let total = r32_add(r32_add(r32_add(b, a)?, c)?, d)?;
    if total == 0.0 || !total.is_finite() {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    [a, b, c, d]
        .map(|weight| m32(f64::from(weight) / f64::from(total)))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| PrefixMembershipError::ParameterDomain)
}

fn mix_four(values: [f32; 4], weights: [f32; 4]) -> Result<f32, PrefixMembershipError> {
    let a = r32_mul(values[0], weights[0])?;
    let b = r32_mul(values[1], weights[1])?;
    let c = r32_mul(values[2], weights[2])?;
    let d = r32_mul(values[3], weights[3])?;
    r32_add(r32_add(r32_add(a, b)?, c)?, d)
}

fn axis_weights_at(
    axis: usize,
    position: f32,
    basis: &[[f32; 3]; POINT_COUNT],
    planes: &[MembershipPlane; 2],
) -> Result<[f32; POINT_COUNT], PrefixMembershipError> {
    let mut weights = [0.0_f32; POINT_COUNT];
    match axis {
        0 => {
            for plane in planes {
                for layer in &plane.layers {
                    let points = &layer.points_by_x;
                    let first = points[0];
                    let last = points[points.len() - 1];
                    if position <= basis[first][0] {
                        weights[first] = 1.0;
                    } else if position >= basis[last][0] {
                        weights[last] = 1.0;
                    } else {
                        for pair in points.windows(2) {
                            let left = pair[0];
                            let right = pair[1];
                            if basis[left][0] < position && position <= basis[right][0] {
                                let fraction = r32_div(
                                    r32_sub(position, basis[left][0])?,
                                    r32_sub(basis[right][0], basis[left][0])?,
                                )?;
                                let (left_weight, right_weight) = equal_power(fraction)?;
                                weights[left] = left_weight;
                                weights[right] = right_weight;
                                break;
                            }
                        }
                    }
                }
            }
        }
        1 => {
            for plane in planes {
                let layers = &plane.layers;
                let mut layer_weights = vec![0.0_f32; layers.len()];
                if position <= layers[0].layer_y {
                    layer_weights[0] = 1.0;
                } else if position > layers[layers.len() - 1].layer_y {
                    layer_weights[layers.len() - 1] = 1.0;
                } else {
                    for index in 0..layers.len() - 1 {
                        let left = layers[index].layer_y;
                        let right = layers[index + 1].layer_y;
                        if left <= position && position <= right {
                            let fraction =
                                r32_div(r32_sub(position, left)?, r32_sub(right, left)?)?;
                            let (left_weight, right_weight) = equal_power(fraction)?;
                            layer_weights[index] = left_weight;
                            layer_weights[index + 1] = right_weight;
                            break;
                        }
                    }
                }
                for (layer, layer_weight) in layers.iter().zip(layer_weights) {
                    for point in &layer.points_by_x {
                        weights[*point] = layer_weight;
                    }
                }
            }
        }
        2 => {
            let (low, high) = if position <= 0.0 {
                (1.0, 0.0)
            } else if position >= r32_div(Q15_MAX as f32, Q15_SCALE)? {
                (0.0, 1.0)
            } else {
                let angle = r32_mul(
                    r32_mul(position, f32::from_bits(PI_OVER_EIGHT_BITS))?,
                    f32::from_bits(FOUR_BITS),
                )?;
                (m32(f64::from(angle).cos())?, m32(f64::from(angle).sin())?)
            };
            for (index, point) in basis.iter().enumerate() {
                weights[index] = if point[2] >= 0.5 { high } else { low };
            }
        }
        _ => return Err(PrefixMembershipError::ParameterDomain),
    }
    Ok(weights)
}

fn equal_power(fraction: f32) -> Result<(f32, f32), PrefixMembershipError> {
    if !(0.0..=1.0).contains(&fraction) {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    let swap = fraction > 0.5;
    let reduced = if swap {
        r32_sub(1.0, fraction)?
    } else {
        fraction
    };
    let angle = r32_mul(
        r32_mul(r32_mul(reduced, f32::from_bits(PI_OVER_FOUR_BITS))?, 0.5)?,
        4.0,
    )?;
    let cosine = m32(f64::from(angle).cos())?;
    let sine = m32(f64::from(angle).sin())?;
    if swap {
        Ok((sine, cosine))
    } else {
        Ok((cosine, sine))
    }
}

struct CellContext<'a> {
    parameters: &'a PrefixMembershipParameters,
    axis: usize,
    planes: &'a [MembershipPlane; 2],
    basis: &'a [[f32; 3]; POINT_COUNT],
    samples: &'a [f32],
    center: f32,
    width: f32,
    exponent: f32,
}

fn compute_cell(
    context: &CellContext<'_>,
    plane: &MembershipPlane,
) -> Result<Vec<f32>, PrefixMembershipError> {
    let trace_len = if context.axis == 2 {
        1
    } else {
        plane.points.len()
    };
    let mut result = vec![0.0_f32; trace_len];
    for sample in context.samples {
        let weights = axis_weights_at(context.axis, *sample, context.basis, context.planes)?;
        for (offset, point) in plane.points.iter().copied().enumerate() {
            if context.axis == 2 && offset != 0 {
                continue;
            }
            let kernel = kernel_value(
                context.parameters,
                context.axis,
                *sample,
                context.center,
                context.width,
            )?;
            let weighted_kernel = r32_mul(kernel, weights[point])?;
            let contribution = pow_value(context.parameters, weighted_kernel, context.exponent)?;
            let result_index = if context.axis == 2 { 0 } else { offset };
            result[result_index] = r32_add(result[result_index], contribution)?;
        }
    }
    Ok(result)
}

fn kernel_value(
    parameters: &PrefixMembershipParameters,
    axis: usize,
    sample: f32,
    center: f32,
    width: f32,
) -> Result<f32, PrefixMembershipError> {
    if width < MAPPED_WIDTH_THRESHOLD {
        return Ok(0.0);
    }
    let delta = r32_mul(r32_sub(sample, center)?, 0.25)?;
    if delta.abs() >= width {
        return Ok(0.0);
    }
    let ratio = m32(f64::from(delta) / f64::from(width))?;
    let square = r32_mul(ratio, ratio)?;
    let fourth = r32_mul(square, square)?;
    let (kernel_c0, kernel_c1) = parameters.kernel_coefficients();
    let negative_fourth = -fourth;
    let scaled = r32_mul(r32_mul(negative_fourth, kernel_c0)?, kernel_c1)?;
    let mut value = exp2_approx(parameters, scaled)?;
    let (_, exponent) = if value == 0.0 {
        return Ok(0.0);
    } else {
        frexp_f32(value)?
    };
    if (-31..=1).contains(&exponent) {
        let (mantissa, _) = frexp_f32(value)?;
        value = r32_mul(power_of_two(exponent)?, mantissa)?;
    } else {
        value = 0.0;
    }
    if axis == 2 {
        let angle = r32_mul(r32_mul(sample, parameters.power_and_z()[4])?, 4.0)?;
        value = r32_mul(value, m32(f64::from(angle).cos())?)?;
    }
    Ok(value)
}

fn normalize_boundary(
    parameters: &PrefixMembershipParameters,
    basis: &[[f32; 3]; POINT_COUNT],
    planes: &[MembershipPlane; 2],
    source_xyz: [f32; 3],
    mapped_width: f32,
    exponent: f32,
    axis_values: &[[f32; POINT_COUNT]; 3],
) -> Result<[u32; POINT_COUNT], PrefixMembershipError> {
    let edges = [(0_usize, 0.0_f32), (0, 1.0), (1, 0.0), (1, 1.0), (2, 1.0)];
    let mut edge_vectors = [[0.0_f32; POINT_COUNT]; 5];
    for (edge_index, (axis, edge_position)) in edges.into_iter().enumerate() {
        let weights = axis_weights_at(axis, edge_position, basis, planes)?;
        for point in 0..POINT_COUNT {
            let kernel = kernel_value(
                parameters,
                axis,
                edge_position,
                source_xyz[axis],
                mapped_width,
            )?;
            edge_vectors[edge_index][point] =
                pow_value(parameters, r32_mul(kernel, weights[point])?, exponent)?;
        }
    }

    let mut raw_boundary = Vec::with_capacity(POINT_COUNT);
    for point in 0..POINT_COUNT {
        let yz = r32_mul(axis_values[1][point], axis_values[2][point])?;
        let yx = r32_mul(axis_values[1][point], axis_values[0][point])?;
        let xz = r32_mul(axis_values[0][point], axis_values[2][point])?;
        let first = r32_mul(yz, edge_vectors[0][point])?;
        let second = r32_mul(yx, edge_vectors[4][point])?;
        let third = r32_mul(yz, edge_vectors[1][point])?;
        let fourth = r32_mul(xz, edge_vectors[2][point])?;
        let fifth = r32_mul(xz, edge_vectors[3][point])?;
        raw_boundary.push(r32_add(
            r32_add(r32_add(r32_add(first, second)?, third)?, fourth)?,
            fifth,
        )?);
    }
    normalize_basic(raw_boundary).map(|values| values.map(f32::to_bits))
}

fn normalize_basic(values: Vec<f32>) -> Result<[f32; POINT_COUNT], PrefixMembershipError> {
    if values.len() != POINT_COUNT {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    let mut sum = 0.0_f32;
    for value in &values {
        sum = r32_add(sum, r32_mul(*value, *value)?)?;
    }
    if sum.abs() < f32::from_bits(MIN_NORMAL_BITS) {
        sum = 0.0;
    }
    let norm = m32(f64::from(sum).sqrt())?.max(f32::from_bits(BASIC_NORM_FLOOR_BITS));
    let mut normalized = Vec::with_capacity(POINT_COUNT);
    for value in values {
        let result = r32_div(value, norm)?;
        normalized.push(if result.abs() < f32::from_bits(MIN_NORMAL_BITS) {
            0.0
        } else {
            result
        });
    }
    normalized
        .try_into()
        .map_err(|_| PrefixMembershipError::ParameterDomain)
}

fn normalize_scaled(values: Vec<f32>) -> Result<[f32; POINT_COUNT], PrefixMembershipError> {
    if values.len() != POINT_COUNT {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    let mut first_scale = 0_i32;
    let mut additional_scale = 0_i32;
    let mut accumulator = 0.0_f32;
    let mut next_factor = 0.5_f32;
    for value in &values {
        let squared = r32_mul(*value, *value)?;
        let shift = (-binary_exponent(squared)?).clamp(0, 60);
        let term;
        if shift == 0 && first_scale == 0 {
            first_scale = 1;
            term = r32_mul(0.5, accumulator)?;
            accumulator = r32_mul(next_factor, squared)?;
        } else {
            let scale = power_of_two(-first_scale - additional_scale)?;
            term = r32_mul(scale, squared)?;
        }
        accumulator = r32_add(accumulator, term)?;
        if (-binary_exponent(accumulator)?).clamp(0, 60) == 0 {
            additional_scale += 1;
            accumulator = r32_mul(accumulator, 0.5)?;
            next_factor = f32::from_bits(
                next_factor
                    .to_bits()
                    .checked_sub(0x0080_0000)
                    .ok_or(PrefixMembershipError::ParameterDomain)?,
            );
        }
    }

    let scale_count = first_scale + additional_scale;
    let mut norm = m32(f64::from(accumulator).sqrt())?;
    if scale_count == 0 && norm < f32::from_bits(BASIC_NORM_FLOOR_BITS) {
        norm = f32::from_bits(BASIC_NORM_FLOOR_BITS);
    }
    let k = (-binary_exponent(norm)?).clamp(0, 60);
    let adjusted_norm = r32_mul(norm, power_of_two(k)?)?;
    let exponent = scale_count - 2 * k;
    let half_exponent = exponent.div_euclid(2);
    let odd = exponent - 2 * half_exponent;
    let multiplier = power_of_two(-half_exponent)?;
    let mut normalized = Vec::with_capacity(POINT_COUNT);
    for value in values {
        let adjusted_value = if odd == 1 {
            r32_mul(value, f32::from_bits(SQRT_HALF_BITS))?
        } else {
            value
        };
        let numerator = r32_mul(multiplier, adjusted_value)?;
        normalized.push(if adjusted_norm < numerator {
            1.0
        } else {
            r32_div(numerator, adjusted_norm)?
        });
    }
    normalized
        .try_into()
        .map_err(|_| PrefixMembershipError::ParameterDomain)
}

fn binary_exponent(value: f32) -> Result<i32, PrefixMembershipError> {
    let doubled = value.to_bits().wrapping_mul(2);
    if doubled == 0 {
        Ok(-127)
    } else {
        let high_byte =
            u8::try_from(doubled >> 24).map_err(|_| PrefixMembershipError::ParameterDomain)?;
        Ok(i32::from(high_byte) - 126)
    }
}

fn power_of_two(exponent: i32) -> Result<f32, PrefixMembershipError> {
    if !(-126..=127).contains(&exponent) {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    let encoded =
        u32::try_from(exponent + 127).map_err(|_| PrefixMembershipError::ParameterDomain)? << 23;
    Ok(f32::from_bits(encoded))
}

fn min6(
    candidate_zero: f32,
    candidate_one: f32,
    candidate_two: f32,
    candidate_three: f32,
    candidate_four: f32,
    candidate_five: f32,
) -> f32 {
    candidate_zero
        .min(candidate_one)
        .min(candidate_two)
        .min(candidate_three)
        .min(candidate_four)
        .min(candidate_five)
}

fn sin_squared(width: f32) -> Result<f32, PrefixMembershipError> {
    if width <= 0.0 {
        return Ok(0.0);
    }
    if width >= f32::from_bits(SIN2_FULL_WIDTH_BITS) {
        return Ok(1.0);
    }
    let bounded = width.min(f32::from_bits(SIN2_FULL_WIDTH_BITS));
    let theta = r32_div(r32_mul(bounded, f32::from_bits(PI_BITS))?, 0.4)?;
    let sine = m32(f64::from(theta).sin())?;
    r32_mul(sine, sine)
}

fn adjust_front_gain(width: f32, unadjusted: &[f32]) -> Result<Vec<f32>, PrefixMembershipError> {
    if unadjusted.len() != POINT_COUNT {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    let mut front = 0.0_f32;
    for value in &unadjusted[..3] {
        front = r32_add(front, r32_mul(*value, *value)?)?;
    }
    let mut rest = 0.0_f32;
    for value in &unadjusted[3..] {
        rest = r32_add(rest, r32_mul(*value, *value)?)?;
    }
    let total = r32_add(rest, front)?;
    if total <= f32::from_bits(0x3586_37bd) {
        return Ok(unadjusted.to_vec());
    }
    let exponent = r32_div(
        r32_mul(width, f32::from_bits(SIN2_EXPONENT_SCALE_BITS))?,
        f32::from_bits(SIN2_EXPONENT_DIVISOR_BITS),
    )?;
    let k = m32(10.0_f64.powf(f64::from(exponent)))?;
    let k2 = r32_mul(k, k)?;
    let numerator = r32_mul(k2, total)?;
    let denominator = r32_add(r32_mul(k2, rest)?, front)?;
    let ratio = r32_div(numerator, denominator)?;
    let norm = m32(f64::from(ratio).sqrt())?;
    let mut result = Vec::with_capacity(POINT_COUNT);
    for (index, value) in unadjusted.iter().copied().enumerate() {
        let adjusted = if index < 3 {
            r32_mul(norm, value)?
        } else {
            r32_mul(r32_mul(norm, k)?, value)?
        };
        result.push(adjusted);
    }
    Ok(result)
}

fn m32(value: f64) -> Result<f32, PrefixMembershipError> {
    let rounded = value as f32;
    if rounded.is_finite() {
        Ok(rounded)
    } else {
        Err(PrefixMembershipError::ParameterDomain)
    }
}

fn log2_approx(
    parameters: &PrefixMembershipParameters,
    value: f32,
) -> Result<f32, PrefixMembershipError> {
    if value <= 0.0 || !value.is_finite() {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    let (mantissa, exponent) = frexp_f32(value)?;
    let [a, b, c, d] = parameters.log_coefficients();
    let squared = r32_mul(mantissa, mantissa)?;
    let first = r32_sub(r32_mul(mantissa, a)?, b)?;
    let second = r32_mul(squared, c)?;
    let third = r32_mul(r32_mul(mantissa, squared)?, d)?;
    let polynomial = r32_add(r32_add(r32_sub(first, second)?, third)?, exponent as f32)?;
    Ok(polynomial)
}

fn exp2_approx(
    parameters: &PrefixMembershipParameters,
    value: f32,
) -> Result<f32, PrefixMembershipError> {
    if !value.is_finite() {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    let integer = value.floor();
    let exponent = integer as i32;
    let fraction = r32_sub(value, integer)?;
    let square = r32_mul(fraction, fraction)?;
    let [u, v, w] = parameters.exp_coefficients();
    let first = r32_add(r32_mul(fraction, u)?, 1.0)?;
    let second = r32_add(first, r32_mul(square, v)?)?;
    let third = r32_mul(r32_mul(fraction, square)?, w)?;
    let mantissa = r32_add(second, third)?;
    scale_by_power_of_two(mantissa, exponent)
}

fn scale_by_power_of_two(value: f32, exponent: i32) -> Result<f32, PrefixMembershipError> {
    if !(-1_074..=1_023).contains(&exponent) {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    let scaled = f64::from(value) * 2.0_f64.powi(exponent);
    let rounded = scaled as f32;
    if rounded.is_finite() {
        Ok(rounded)
    } else {
        Err(PrefixMembershipError::ParameterDomain)
    }
}

fn frexp_f32(value: f32) -> Result<(f32, i32), PrefixMembershipError> {
    if value <= 0.0 || !value.is_finite() || value < f32::from_bits(MIN_NORMAL_BITS) {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    let bits = value.to_bits();
    let exponent_bits = (bits >> 23) & 0xff;
    let exponent =
        i32::try_from(exponent_bits).map_err(|_| PrefixMembershipError::ParameterDomain)? - 126;
    let mantissa = f32::from_bits((bits & 0x807f_ffff) | (126_u32 << 23));
    Ok((mantissa, exponent))
}

fn finite(value: f32) -> Result<FiniteF32, PrefixMembershipError> {
    FiniteF32::new(value).map_err(|_| PrefixMembershipError::ParameterDomain)
}

fn r32_add(left: f32, right: f32) -> Result<f32, PrefixMembershipError> {
    numeric_r32_add(finite(left)?, finite(right)?)
        .map(FiniteF32::value)
        .map_err(|_| PrefixMembershipError::ParameterDomain)
}

fn r32_sub(left: f32, right: f32) -> Result<f32, PrefixMembershipError> {
    numeric_r32_sub(finite(left)?, finite(right)?)
        .map(FiniteF32::value)
        .map_err(|_| PrefixMembershipError::ParameterDomain)
}

fn r32_mul(left: f32, right: f32) -> Result<f32, PrefixMembershipError> {
    numeric_r32_mul(finite(left)?, finite(right)?)
        .map(FiniteF32::value)
        .map_err(|_| PrefixMembershipError::ParameterDomain)
}

fn r32_div(left: f32, right: f32) -> Result<f32, PrefixMembershipError> {
    if right == 0.0 || !left.is_finite() || !right.is_finite() {
        return Err(PrefixMembershipError::ParameterDomain);
    }
    let quotient = left / right;
    if quotient.is_finite() {
        Ok(quotient)
    } else {
        Err(PrefixMembershipError::ParameterDomain)
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut text, "{byte:02x}").expect("writing to a String cannot fail");
    }
    text
}

#[cfg(test)]
mod prefix_membership_tests;
