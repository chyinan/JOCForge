// pattern: Functional Core
//! Selected transform helpers; parent audio clocks and lifecycle remain outside this module.

pub mod history_owner;
pub mod lfe_transform;
pub mod long_transform;
pub(super) mod parameters;
pub mod secondary_carrier;
pub mod secondary_fft;
pub mod secondary_inverse;
pub mod secondary_overlap;
pub mod workspace;
