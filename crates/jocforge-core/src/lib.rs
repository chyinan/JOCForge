// pattern: Functional Core

//! Public-spec JOCForge encoding primitives.

mod adm;
mod analysis;
mod profile;
mod rate;
mod reduction;
mod spectral;
mod validation;
mod writer;

pub use adm::{
    AdmChannel, AdmError, AdmObject, AdmObjectKind, AdmPosition, AdmProgramme, ParsedAdm,
    ParsedAdmMetadata, SyntheticFixture, decode_pcm_frame,
    generate_synthetic_direct_speaker_fixture, generate_synthetic_fixture,
    generate_synthetic_scene_fixture, generate_synthetic_unsupported_fixture, parse_bw64_adm,
    parse_bw64_adm_metadata, validate_source_derived_adm_metadata,
};
pub use analysis::{
    QuantizedCoefficient, RankFactorization, SOURCE_DERIVED_CARRIER_PEAK_LIMIT,
    factorize_rank_controlled, factorize_source_derived, limit_signal_peak,
    maximum_absolute_difference, quantize_coefficient, reconstruction_error,
};
pub use profile::{JocProfileSpec, profile_spec};
pub use rate::{RateBudget, RateBudgetError, rate_budget};
pub use reduction::{
    MAX_CODED_OBJECTS, ReducedObject, ReductionConfig, ReductionDiagnostics, ReductionError,
    ReductionResult, ReductionSource, SourceAccounting, reduce_scene, reduce_scene_with_previous,
    reduction_assignment_changes, reduction_identity_changes, speaker_render_error,
    unexplained_cluster_leakage,
};
pub use spectral::{
    GateScope, GateStatus, SpectralPhaseMetric, SpectralPhaseReport, spectral_phase_report,
};
pub use validation::{
    TemporalMetrics, TemporalScene, adversarial_temporal_corpus, analyze_temporal_scene,
    analyze_temporal_scene_with_config, temporal_corpus,
};
pub use writer::{
    package_ffmpeg_access_unit, package_source_derived_access_unit,
    package_source_derived_access_unit_with_frame_sizes, write_one_au_proof,
};

/// MVP JOC downmix profiles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JocProfile {
    Idx0,
    Idx1,
    Idx2,
}
