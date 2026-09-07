// pattern: Functional Core

//! Public-spec JOCForge encoding primitives.

mod adm;
mod analysis;
mod conformance;
mod eac3;
mod plan;
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
    CarrierCausalityMeasurement, QuantizedCoefficient, RankFactorization,
    SOURCE_DERIVED_CARRIER_PEAK_LIMIT, factorize_rank_controlled, factorize_source_derived,
    limit_signal_peak, maximum_absolute_difference, quantize_coefficient, reconstruction_error,
    source_derived_carrier_causality,
};
pub use conformance::{
    BlockPartition, ConformancePlanError, DependentTopology, ShortBlockPlan,
    resolve_dependent_topology, resolve_short_block_plan,
};
pub use eac3::inject_joc_skipfld;
pub use plan::{JocEncodingPlan, JocPlanError, resolve_encoding_plan};
pub use profile::{JocProfileSpec, profile_spec};
pub use rate::{RateBudget, RateBudgetError, rate_budget, rate_budget_for_carriage};
pub use reduction::{
    MAX_CODED_OBJECTS, ReducedObject, ReductionConfig, ReductionDiagnostics, ReductionError,
    ReductionResult, ReductionSource, SourceAccounting, authored_source_assignment_changes,
    reduce_scene, reduce_scene_with_previous, representative_source_changes, speaker_render_error,
    unexplained_cluster_leakage,
};
pub use spectral::{
    FinalRenderGate, FinalRenderMeasurement, GateScope, GateStatus, SpectralPhaseMetric,
    SpectralPhaseReport, final_render_gate, spectral_phase_report,
};
pub use validation::{
    TemporalIdentityMetrics, TemporalMetrics, TemporalScene, adversarial_temporal_corpus,
    analyze_temporal_scene, analyze_temporal_scene_with_config, measure_temporal_identity_metrics,
    temporal_corpus,
};
pub use writer::{
    D0CausalityMeasurement, d0_carrier_causality, package_ffmpeg_access_unit,
    package_i0_skip_field_access_unit, package_legacy_ac3_source_derived_access_unit,
    package_multi_dependent_source_derived_access_unit, package_source_derived_access_unit,
    package_source_derived_access_unit_with_frame_sizes, write_one_au_proof,
};

/// MVP JOC downmix profiles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JocProfile {
    Idx0,
    Idx1,
    Idx2,
    Idx3,
    Idx4,
}

impl JocProfile {
    /// Returns the user-facing semantic name from ETSI TS 103 420 Table 47.
    #[must_use]
    pub const fn semantic_name(self) -> &'static str {
        match self {
            Self::Idx0 => "5.X",
            Self::Idx1 => "Flat-7.X",
            Self::Idx2 => "5.X+2",
            Self::Idx3 => "5.X Phase",
            Self::Idx4 => "5.X+2 Phase",
        }
    }

    /// Returns the numeric JOC downmix configuration index.
    #[must_use]
    pub const fn downmix_index(self) -> u8 {
        match self {
            Self::Idx0 => 0,
            Self::Idx1 => 1,
            Self::Idx2 => 2,
            Self::Idx3 => 3,
            Self::Idx4 => 4,
        }
    }

    /// Returns whether this profile signals the public phase variant.
    #[must_use]
    pub const fn is_phase_signaled(self) -> bool {
        matches!(self, Self::Idx3 | Self::Idx4)
    }
}

/// Publicly selected E-AC-3/JOC carriage topology.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum JocCarriage {
    /// Existing JOCForge I0 plus dependent-substream carriage with EMDF in D0.
    #[default]
    EtsiAuxDependent,
    /// Public E-AC-3 I0-only carriage with EMDF in audio-block skip fields.
    I0SkipField,
    /// Advanced public original-syntax AC-3 I0 plus E-AC-3 D0 carriage.
    LegacyAc3Dependent,
}

/// EMDF signaling convention selected independently from carriage topology.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EmdfProfile {
    /// Published ETSI TS 103 420 configuration, including `codecdatae=1`.
    #[default]
    EtsiStrict,
    /// Observed deployed interoperability deviation: `codecdatae=0` for IDs
    /// 11 and 14, with all other JOCForge-owned configuration unchanged.
    DeployedCompat,
}
