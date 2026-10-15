// pattern: Functional Core
//! Consumer-bound numeric primitives; no generic math toolkit.

mod q15_coordinate;

pub use q15_coordinate::{Q15Coordinate, Q15CoordinateError, quantize_q15_coordinate};
