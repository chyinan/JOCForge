// pattern: Imperative Shell
use jocforge::selected_spatial_callback_providers::{
    load_selected_independent_meter_callback_providers,
    load_selected_main_spatial_callback_providers,
};
use jocforge_application::independent_meter_callback::generate_selected_spatial_math_geometry;
use jocforge_domain::meter::{
    meter_prepass::IndependentMeterPrepass, sample_block_queue::MeterSourceSampleCount,
    shared_gain_delay::SharedGainDelayParameters,
};
use jocforge_io::provider::load_meter_direct_provider_inventory;
use std::path::PathBuf;

const SPATIAL_PROVIDER_BUNDLE: &str = "ADM-F03W-APPROVED-PROVIDER-BUNDLE-V1";
const COLD_STATE_BUNDLE: &str = "F01B-F03W-COLD-STATE-ASSETS-V1";
const CURRENT_FEATURES_BUNDLE: &str = "METER-APPROVED-PARAMETER-ASSETS-V1";
const F04_RECEIVER_BUNDLE: &str = "F04-RECEIVER-APPROVED-PARAMETERS-V1";

fn approved_bundle(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../outputs/IMPLEMENTER_SUPPLEMENTS")
        .join(name)
}

#[test]
fn combines_approved_math_and_cold_values_without_claiming_runtime_layout_or_callback() {
    let geometry = generate_selected_spatial_math_geometry()
        .expect("selected P11/P15 mathematical geometry is available");

    let inputs = load_selected_independent_meter_callback_providers(
        &approved_bundle(SPATIAL_PROVIDER_BUNDLE),
        &approved_bundle(COLD_STATE_BUNDLE),
        &approved_bundle(CURRENT_FEATURES_BUNDLE),
        geometry,
    )
    .expect("approved values bind to the existing callback provider shape");

    assert_eq!(
        inputs
            .spatial_cold_lineage()
            .f03w_manifest_sha256()
            .to_hex(),
        "9b81b736e9dd110441d3b59333afac9cbb98886f8bf4ac9854b5d74bdf57783f"
    );
    assert_eq!(
        inputs
            .spatial_cold_lineage()
            .cold_state_manifest_sha256()
            .to_hex(),
        "d15672b723f8091b576c2228ed7685744f522706d89078050b6486a72a068372"
    );
    assert_eq!(inputs.consumed_spatial_provider_entries().len(), 3);
    assert_eq!(inputs.declared_spatial_provider_entries(), 48);
    assert_eq!(inputs.open_spatial_provider_entries(), 45);
    assert_eq!(inputs.declared_current_features_assets(), 18);
    assert_eq!(inputs.validated_current_features_assets(), 2);
    assert_eq!(inputs.open_current_features_assets(), 16);
    assert!(inputs.transitive_provider_dependencies_are_open());
    assert!(!inputs.runtime_layout_binding_is_observed());
    assert!(!inputs.callback_lineage_is_bound());
    assert!(!inputs.is_runtime_provider_set());

    let providers = inputs.providers();
    assert_eq!(providers.f01e.object_count(), 28);
    assert_eq!(providers.f03_target.current_id_u32(), &[u32::MAX; 15]);
    assert_eq!(providers.window.receiver_blend32().len(), 32);
    assert_eq!(
        providers
            .component_parameters
            .receipt()
            .provider_file_bytes(),
        1_077
    );
    assert_eq!(providers.diffusion.ring_length(), 96);
    assert_eq!(providers.p11_geometry.points().len(), 11);
    assert_eq!(providers.p15_geometry.layout(), 0x159b);
    assert_eq!(providers.p15_geometry.points().len(), 15);

    let meter_direct =
        load_meter_direct_provider_inventory(approved_bundle(CURRENT_FEATURES_BUNDLE))
            .expect("the directly inventoried Meter assets load and pass owner validators");
    let profile_id = jocforge_application::meter_source_admission::selected_meter_profile_identity(
        jocforge_application::meter_source_bridge::EmdfCarriageProfile::DeployedCompat,
    )
    .unwrap();
    let meter_identity =
        jocforge::selected_meter_provider_identity::load_selected_meter_provider_identity(
            &approved_bundle("METER-PROVIDER-SET-IDENTITY-APPLICATION-V1"),
            &inputs,
            &meter_direct,
            profile_id,
        )
        .expect("the selected static provider set identity is assembled from validated assets");
    assert_eq!(meter_identity.members().len(), 19);
    let meter_provider_set = meter_identity
        .clone()
        .into_meter_provider_set()
        .expect("the member receipts project to a local MeterProviderSet");
    assert_eq!(meter_provider_set.providers().len(), 19);
    assert!(
        meter_provider_set
            .providers()
            .windows(2)
            .all(|pair| pair[0].id().as_bytes() < pair[1].id().as_bytes())
    );
    let composed = SharedGainDelayParameters::from_approved_direct_parameters(
        meter_direct.shared_dv32_parameters(),
        &providers.window,
    )
    .expect("the approved DV32 asset and validated receiver blend compose");
    let selected = SharedGainDelayParameters::selected(providers.window.receiver_blend32())
        .expect("the selected callback parameter reference remains available");
    assert_eq!(composed, selected);
    assert!(!meter_direct.transitive_provider_set_complete());

    let post_render_parameters = meter_direct.post_render_parameters();
    assert_eq!(
        post_render_parameters.oar_geometry(),
        meter_direct.oar_geometry().render_geometry()
    );
    let mut prepass = IndependentMeterPrepass::cold_with_post_render_parameters(
        MeterSourceSampleCount::new(6_522_906).unwrap(),
        composed,
        meter_direct.k_matrix().clone(),
        post_render_parameters,
    )
    .expect("validated direct true-peak and log parameters build a cold prepass owner");
    assert_eq!(prepass.next_callback_index().value(), 0);
    assert_eq!(prepass.source_samples().value(), 6_522_906);
    assert_eq!(
        prepass.post_render_parameters().true_peak(),
        meter_direct.true_peak_parameters()
    );
    assert_eq!(
        prepass.post_render_parameters().integrated(),
        meter_direct.integrated_meter_parameters()
    );
    assert_eq!(
        prepass.post_render_parameters().ungated(),
        meter_direct.ungated_mean_parameters()
    );
    assert_eq!(
        prepass.post_render_parameters().oar_geometry(),
        meter_direct.oar_geometry().render_geometry()
    );
    prepass.reset().unwrap();
    assert_eq!(prepass.next_callback_index().value(), 0);
    assert_eq!(
        prepass.post_render_parameters().true_peak(),
        meter_direct.true_peak_parameters()
    );
    assert_eq!(
        prepass.post_render_parameters().integrated(),
        meter_direct.integrated_meter_parameters()
    );
    assert_eq!(
        prepass.post_render_parameters().ungated(),
        meter_direct.ungated_mean_parameters()
    );
    assert_eq!(
        prepass.post_render_parameters().oar_geometry(),
        meter_direct.oar_geometry().render_geometry()
    );
}

#[test]
fn main_callback_composition_loads_the_hash_bound_f04_receiver_bundle() {
    let geometry = generate_selected_spatial_math_geometry()
        .expect("selected P11/P15 mathematical geometry is available");
    let providers = load_selected_main_spatial_callback_providers(
        &approved_bundle(SPATIAL_PROVIDER_BUNDLE),
        &approved_bundle(COLD_STATE_BUNDLE),
        &approved_bundle(CURRENT_FEATURES_BUNDLE),
        &approved_bundle(F04_RECEIVER_BUNDLE),
        geometry,
    )
    .expect("selected callback assets and the Wash-approved F04 bundle validate");

    assert_eq!(
        providers.spatial_audio().manifest_sha256().to_hex(),
        "161147c7959083738f5426b2280457454b08b73ee58a075322c33880cecc818b"
    );
    assert_eq!(
        providers
            .callback_values()
            .declared_spatial_provider_entries(),
        48
    );
    assert_eq!(
        providers.callback_values().open_spatial_provider_entries(),
        45
    );
    assert_eq!(
        providers
            .callback_values()
            .declared_current_features_assets(),
        18
    );
    assert_eq!(
        providers.callback_values().open_current_features_assets(),
        16
    );
}
