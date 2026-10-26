// pattern: Functional Core
//! Selected P15 basis generation for layout 0x159B.

use sha2::{Digest, Sha256};

use crate::numeric::{FiniteF32, r32_mul};

const SELECTED_P15_LAYOUT: u32 = 0x159b;
const SELECTED_PROFILE_ENABLED: bool = true;
const Q15_U: i32 = 32_767;
const Q15_H: i32 = 16_384;
const FRONT_SIDE_Y_Q15: i32 = 5_285;
const P15_GENERATION_SPEC_SHA256: &str =
    "8ea5ff070ebfb77ecfe86a7e87ce846e05de2a1b3575bf834b5ead7414a51830";
const FRONT_SIDE_Y_PARAMETER_SHA256: &str =
    "5d2346dd39671aae7f7d3b56396fb1dea05da363a218a0efbd926dbab046c185";
const APPROVED_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const Q15_SCALE_F32: f32 = f32::from_bits(0x3800_0000);
const SOURCE_LAYOUT_INDICES: [u8; 15] = [0, 1, 2, 4, 5, 6, 7, 11, 12, 13, 14, 17, 18, 21, 22];
const P15_Q15_POINTS: [[i32; 3]; 15] = [
    [0, 0, 0],
    [Q15_U, 0, 0],
    [Q15_H, 0, 0],
    [0, Q15_H, 0],
    [Q15_U, Q15_H, 0],
    [0, Q15_U, 0],
    [Q15_U, Q15_U, 0],
    [0, FRONT_SIDE_Y_Q15, 0],
    [Q15_U, FRONT_SIDE_Y_Q15, 0],
    [0, 0, Q15_U],
    [Q15_U, 0, Q15_U],
    [0, Q15_H, Q15_U],
    [Q15_U, Q15_H, Q15_U],
    [0, Q15_U, Q15_U],
    [Q15_U, Q15_U, Q15_U],
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct P15BasisIndex(u8);

impl P15BasisIndex {
    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedP15Point {
    ordinal: P15BasisIndex,
    source_layout_index: u8,
    q15_xyz: [i32; 3],
    xyz_f32_bits: [u32; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedP15GeometryIdentity {
    layout: u32,
    profile_enabled: bool,
    generation_spec_sha256: &'static str,
    front_side_y_parameter_sha256: &'static str,
    approved_manifest_sha256: &'static str,
    f32le_sha256: [u8; 32],
}

impl SelectedP15GeometryIdentity {
    pub const fn layout(self) -> u32 {
        self.layout
    }

    pub const fn profile_enabled(self) -> bool {
        self.profile_enabled
    }

    pub const fn generation_spec_sha256(self) -> &'static str {
        self.generation_spec_sha256
    }

    pub const fn front_side_y_parameter_sha256(self) -> &'static str {
        self.front_side_y_parameter_sha256
    }

    pub const fn approved_manifest_sha256(self) -> &'static str {
        self.approved_manifest_sha256
    }

    pub const fn f32le_sha256(self) -> [u8; 32] {
        self.f32le_sha256
    }
}

impl SelectedP15Point {
    pub const fn ordinal(self) -> P15BasisIndex {
        self.ordinal
    }

    pub const fn source_layout_index(self) -> u8 {
        self.source_layout_index
    }

    pub const fn q15_xyz(self) -> [i32; 3] {
        self.q15_xyz
    }

    pub const fn xyz_f32_bits(self) -> [u32; 3] {
        self.xyz_f32_bits
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedP15GeometryError {
    UnsupportedProfile,
    NumericDomain,
}

impl SelectedP15GeometryError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnsupportedProfile => "UNSUPPORTED_PROFILE",
            Self::NumericDomain => "SELECTED_P15_GEOMETRY_NUMERIC",
        }
    }
}

/// Stable 15-point selected geometry with the authorized front-side Y parameter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedP15Geometry {
    points: [SelectedP15Point; 15],
    layout: u32,
    front_side_y_q15: i32,
    f32le_sha256: [u8; 32],
}

impl SelectedP15Geometry {
    pub fn points(&self) -> &[SelectedP15Point; 15] {
        &self.points
    }

    pub const fn layout(&self) -> u32 {
        self.layout
    }

    pub const fn front_side_y_q15(&self) -> i32 {
        self.front_side_y_q15
    }

    pub const fn f32le_sha256(&self) -> &[u8; 32] {
        &self.f32le_sha256
    }

    pub const fn identity(&self) -> SelectedP15GeometryIdentity {
        SelectedP15GeometryIdentity {
            layout: self.layout,
            profile_enabled: SELECTED_PROFILE_ENABLED,
            generation_spec_sha256: P15_GENERATION_SPEC_SHA256,
            front_side_y_parameter_sha256: FRONT_SIDE_Y_PARAMETER_SHA256,
            approved_manifest_sha256: APPROVED_MANIFEST_SHA256,
            f32le_sha256: self.f32le_sha256,
        }
    }
}

/// Generates only the selected P15 layout; row order is fixed by the approved recipe.
pub fn generate_selected_p15_geometry(
    layout: u32,
    profile_enabled: bool,
) -> Result<SelectedP15Geometry, SelectedP15GeometryError> {
    if layout != SELECTED_P15_LAYOUT || profile_enabled != SELECTED_PROFILE_ENABLED {
        return Err(SelectedP15GeometryError::UnsupportedProfile);
    }
    if !(0..Q15_H).contains(&FRONT_SIDE_Y_Q15) {
        return Err(SelectedP15GeometryError::NumericDomain);
    }

    let mut canonical = Vec::with_capacity(15 * 3 * 4);
    let point_results: [Result<SelectedP15Point, SelectedP15GeometryError>; 15] =
        std::array::from_fn(|ordinal| {
            let q15_xyz = P15_Q15_POINTS[ordinal];
            let xyz_f32_bits = q15_xyz.map(|coordinate| {
                let coordinate = FiniteF32::new(coordinate as f32)
                    .map_err(|_| SelectedP15GeometryError::NumericDomain)?;
                let scale = FiniteF32::new(Q15_SCALE_F32)
                    .map_err(|_| SelectedP15GeometryError::NumericDomain)?;
                let word = r32_mul(coordinate, scale)
                    .map_err(|_| SelectedP15GeometryError::NumericDomain)?
                    .to_bits();
                canonical.extend_from_slice(&word.to_le_bytes());
                Ok(word)
            });
            let xyz_f32_bits = xyz_f32_bits
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
                .map_err(|_| SelectedP15GeometryError::NumericDomain)?;
            Ok(SelectedP15Point {
                ordinal: P15BasisIndex(
                    u8::try_from(ordinal).map_err(|_| SelectedP15GeometryError::NumericDomain)?,
                ),
                source_layout_index: SOURCE_LAYOUT_INDICES[ordinal],
                q15_xyz,
                xyz_f32_bits,
            })
        });
    let points = point_results
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| SelectedP15GeometryError::NumericDomain)?;
    let mut f32le_sha256 = [0; 32];
    f32le_sha256.copy_from_slice(&Sha256::digest(canonical));
    Ok(SelectedP15Geometry {
        points,
        layout,
        front_side_y_q15: FRONT_SIDE_Y_Q15,
        f32le_sha256,
    })
}
