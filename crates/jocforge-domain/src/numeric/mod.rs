// pattern: Functional Core
//! Consumer-bound numeric primitives; no generic math toolkit.

pub(crate) mod exact_trigonometry;
mod finite_f32;
mod q15_coordinate;
mod trunc_i32;

pub use finite_f32::{FiniteF32, FiniteF32Error, R32Error, r32_add, r32_mul, r32_sub};
pub use q15_coordinate::{
    NormalizedQ15Coordinate, Q15Coordinate, Q15CoordinateError, normalize_q15_coordinate,
    quantize_q15_coordinate,
};
pub use trunc_i32::{TruncI32Error, TruncatedI32, trunc_f32_to_i32};
