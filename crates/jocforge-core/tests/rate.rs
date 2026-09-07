// pattern: Functional Core

use jocforge_core::{JocCarriage, rate_budget, rate_budget_for_carriage};

#[test]
fn clean_rate_budget_maps_supported_total_rates_to_explicit_frame_sizes() {
    let budget = rate_budget(1024).expect("minimum paired JOC budget");

    assert_eq!(budget.total_bitrate_kbps, 1024);
    assert_eq!(budget.carrier_bitrate_kbps, 384);
    assert_eq!(budget.d0_bitrate_kbps, 640);
    assert_eq!(budget.carrier_frame_bytes, 1536);
    assert_eq!(budget.d0_frame_bytes, 2560);

    let reference = rate_budget(1664).expect("reference JOC budget");
    assert_eq!(reference.carrier_bitrate_kbps, 1024);
    assert_eq!(reference.d0_bitrate_kbps, 640);
    assert_eq!(reference.carrier_frame_bytes, 4096);
    assert_eq!(reference.d0_frame_bytes, 2560);
}

#[test]
fn clean_rate_budget_rejects_an_unrepresentable_total_budget() {
    let error = rate_budget(768).expect_err("source-derived D0 needs the established capacity");

    assert!(error.to_string().contains("unsupported total bitrate"));
}

#[test]
fn deployed_compat_budget_uses_a_padded_nominal_i0_frame() {
    let budget = rate_budget_for_carriage(JocCarriage::I0SkipField, 1024)
        .expect("deployed-compatible idx0 budget");
    assert_eq!(budget.carrier_bitrate_kbps, 1024);
    assert_eq!(budget.carrier_encode_bitrate_kbps, 896);
    assert_eq!(budget.carrier_frame_bytes, 4096);
    assert_eq!(budget.carrier_encoded_frame_bytes, 3584);
    assert_eq!(budget.d0_frame_bytes, 0);
}
