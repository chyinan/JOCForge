// pattern: Functional Core

//! Public-spec JOCForge encoding primitives.

mod adm;
mod analysis;
mod profile;
mod writer;

pub use adm::{
    AdmError, AdmObject, AdmPosition, ParsedAdm, SyntheticFixture, generate_synthetic_fixture,
    parse_bw64_adm,
};
pub use analysis::{
    QuantizedCoefficient, RankFactorization, factorize_rank_controlled, quantize_coefficient,
};
pub use profile::{JocProfileSpec, profile_spec};
pub use writer::{package_ffmpeg_access_unit, write_one_au_proof};

/// MVP JOC downmix profiles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JocProfile {
    Idx0,
    Idx1,
    Idx2,
}
