// pattern: Functional Core
//! N00 finite binary32 wrapper and independently rounded add/subtract/multiply primitives.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FiniteF32Error {
    NonFinite,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FiniteF32(f32);

impl FiniteF32 {
    pub fn new(value: f32) -> Result<Self, FiniteF32Error> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(FiniteF32Error::NonFinite)
        }
    }
    pub const fn value(self) -> f32 {
        self.0
    }
    pub const fn to_bits(self) -> u32 {
        self.0.to_bits()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum R32Error {
    NonFiniteResult,
}

pub fn r32_mul(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, R32Error> {
    FiniteF32::new(left.value() * right.value()).map_err(|_| R32Error::NonFiniteResult)
}

pub fn r32_add(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, R32Error> {
    FiniteF32::new(left.value() + right.value()).map_err(|_| R32Error::NonFiniteResult)
}

pub fn r32_sub(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, R32Error> {
    r32_add(left, FiniteF32(-right.value()))
}
