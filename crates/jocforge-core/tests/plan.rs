// pattern: Functional Core

use jocforge_core::{EmdfProfile, JocCarriage, JocPlanError, JocProfile, resolve_encoding_plan};

#[test]
fn resolver_returns_a_complete_profile_carriage_rate_plan() {
    let plan = resolve_encoding_plan(
        JocProfile::Idx4,
        JocCarriage::EtsiAuxDependent,
        EmdfProfile::EtsiStrict,
        1664,
    )
    .expect("idx4 strict plan");

    assert_eq!(plan.profile, JocProfile::Idx4);
    assert_eq!(plan.carriage, JocCarriage::EtsiAuxDependent);
    assert_eq!(plan.emdf_profile, EmdfProfile::EtsiStrict);
    assert_eq!(plan.rate.total_bitrate_kbps, 1664);
    assert_eq!(plan.spec.downmix_index, 4);
    assert_eq!(plan.spec.input_labels.len(), 7);
    assert!(plan.spec.phase_signaled);
}

#[test]
fn resolver_rejects_skip_field_for_profiles_without_a_closed_topology() {
    let error = resolve_encoding_plan(
        JocProfile::Idx1,
        JocCarriage::I0SkipField,
        EmdfProfile::DeployedCompat,
        1024,
    )
    .expect_err("idx1 skip-field carriage must remain fail-closed");

    assert_eq!(
        error,
        JocPlanError::UnsupportedCarriage {
            profile: JocProfile::Idx1,
            carriage: JocCarriage::I0SkipField,
        }
    );
}

#[test]
fn resolver_keeps_the_actionable_total_bitrate_error_category() {
    let error = resolve_encoding_plan(
        JocProfile::Idx4,
        JocCarriage::EtsiAuxDependent,
        EmdfProfile::EtsiStrict,
        896,
    )
    .expect_err("paired I0/D0 capacity is below the clean minimum");

    assert!(error.to_string().contains("unsupported total bitrate"));
}
