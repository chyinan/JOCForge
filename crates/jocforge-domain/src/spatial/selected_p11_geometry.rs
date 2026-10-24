// pattern: Functional Core
//! Direct selected-profile P11 geometry generation from the approved layout rule.

use sha2::{Digest, Sha256};

use crate::numeric::{FiniteF32, r32_mul};

const SELECTED_P11_LAYOUT: u32 = 0x111b;
const SELECTED_PROFILE_ENABLED: bool = true;
const Q15_U: i32 = 32_767;
const Q15_H: i32 = 16_384;
const Q15_SCALE_F32: f32 = f32::from_bits(0x3800_0000);
const SOURCE_LAYOUT_INDICES: [u8; 11] = [0, 1, 2, 4, 5, 6, 7, 13, 14, 21, 22];
const Q15_POINTS: [[i32; 3]; 11] = [
    [0, 0, 0],
    [Q15_U, 0, 0],
    [Q15_H, 0, 0],
    [0, Q15_H, 0],
    [Q15_U, Q15_H, 0],
    [0, Q15_U, 0],
    [Q15_U, Q15_U, 0],
    [0, 0, Q15_U],
    [Q15_U, 0, Q15_U],
    [0, Q15_U, Q15_U],
    [Q15_U, Q15_U, Q15_U],
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct P11PrefixOrdinal(u8);

impl P11PrefixOrdinal {
    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedP11Point {
    ordinal: P11PrefixOrdinal,
    source_layout_index: u8,
    q15_xyz: [i32; 3],
    xyz_words: [u32; 3],
}

impl SelectedP11Point {
    pub const fn ordinal(self) -> P11PrefixOrdinal {
        self.ordinal
    }

    pub const fn source_layout_index(self) -> u8 {
        self.source_layout_index
    }

    pub const fn q15_xyz(self) -> [i32; 3] {
        self.q15_xyz
    }

    pub const fn xyz_words(self) -> [u32; 3] {
        self.xyz_words
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedP11GeometryError {
    UnsupportedProfile,
    NumericDomain,
}

impl SelectedP11GeometryError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedProfile => "UNSUPPORTED_PROFILE",
            Self::NumericDomain => "SELECTED_GEOMETRY_NUMERIC",
        }
    }
}

/// Selected P11 basis in prefix order; its receipt covers only these 11 points.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedP11Geometry {
    points: [SelectedP11Point; 11],
    f32le_sha256: [u8; 32],
}

impl SelectedP11Geometry {
    pub fn points(&self) -> &[SelectedP11Point; 11] {
        &self.points
    }

    pub const fn f32le_sha256(&self) -> &[u8; 32] {
        &self.f32le_sha256
    }
}

/// Generates selected layout 0x111B with profile P11 enabled. P15 and other
/// layout rows are intentionally outside this producer.
pub fn generate_selected_p11_geometry(
    layout: u32,
    profile_enabled: bool,
) -> Result<SelectedP11Geometry, SelectedP11GeometryError> {
    if layout != SELECTED_P11_LAYOUT || profile_enabled != SELECTED_PROFILE_ENABLED {
        return Err(SelectedP11GeometryError::UnsupportedProfile);
    }
    let mut canonical = Vec::with_capacity(11 * 3 * 4);
    let points: [Result<SelectedP11Point, SelectedP11GeometryError>; 11] =
        std::array::from_fn(|ordinal| {
            let q15_xyz = Q15_POINTS[ordinal];
            let xyz_words = q15_xyz.map(|coordinate| {
                let coordinate = FiniteF32::new(coordinate as f32)
                    .map_err(|_| SelectedP11GeometryError::NumericDomain)?;
                let scale = FiniteF32::new(Q15_SCALE_F32)
                    .map_err(|_| SelectedP11GeometryError::NumericDomain)?;
                let value = r32_mul(coordinate, scale)
                    .map_err(|_| SelectedP11GeometryError::NumericDomain)?
                    .value();
                canonical.extend_from_slice(&value.to_bits().to_le_bytes());
                Ok(value.to_bits())
            });
            let xyz_words = xyz_words
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
                .and_then(|words| {
                    words
                        .try_into()
                        .map_err(|_| SelectedP11GeometryError::NumericDomain)
                })?;
            Ok(SelectedP11Point {
                ordinal: P11PrefixOrdinal(
                    u8::try_from(ordinal).map_err(|_| SelectedP11GeometryError::NumericDomain)?,
                ),
                source_layout_index: SOURCE_LAYOUT_INDICES[ordinal],
                q15_xyz,
                xyz_words,
            })
        });
    let points = points
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| SelectedP11GeometryError::NumericDomain)?;
    let digest = Sha256::digest(canonical);
    let mut f32le_sha256 = [0_u8; 32];
    f32le_sha256.copy_from_slice(&digest);
    Ok(SelectedP11Geometry {
        points,
        f32le_sha256,
    })
}
