// pattern: Functional Core
//! Consumer-bound numeric primitives; no generic math toolkit.

mod finite_f32;
mod q15_coordinate;

pub use finite_f32::{FiniteF32, FiniteF32Error, R32Error, r32_add, r32_mul};
pub use q15_coordinate::{Q15Coordinate, Q15CoordinateError, quantize_q15_coordinate};
