use std::sync::Arc;

use crate::frontend::{PackageManifestEvidence, ProviderFileEvidence};
use crate::spatial::diffusion_filter::{
    DiffusionFilterParameters, preview_filter_block, validate_diffusion_filter_package_manifest,
    validate_diffusion_filter_parameters,
};

use super::{DiffusionTailState, DiffusionTailTransition, preview_diffusion_tail};

const PACKAGE_MANIFEST: &[u8] =
    include_bytes!("../../../tests/fixtures/f01e_provider/MANIFEST.json");
const DIFFUSION_PARAMETERS: &[u8] =
    include_bytes!("../../../tests/fixtures/diffusion_filter/diffusion-mathematical-v1.json");
const CONSTRUCTOR_PARAMETERS: &[u8] = include_bytes!(
    "../../../tests/fixtures/diffusion_filter/diffusion-constructor-mathematical-v1.json"
);

fn selected_parameters() -> DiffusionFilterParameters {
    let manifest_evidence = PackageManifestEvidence::new(Arc::from(PACKAGE_MANIFEST), None);
    let manifest = validate_diffusion_filter_package_manifest(&manifest_evidence).unwrap();
    let coefficient_evidence = ProviderFileEvidence::new(Arc::from(DIFFUSION_PARAMETERS), None);
    let constructor_evidence = ProviderFileEvidence::new(Arc::from(CONSTRUCTOR_PARAMETERS), None);
    validate_diffusion_filter_parameters(&coefficient_evidence, &constructor_evidence, &manifest)
        .unwrap()
}

fn frame(offset: usize) -> [f32; 1536] {
    std::array::from_fn(|index| {
        let residue = (index * 17 + offset * 23) % 251;
        f32::from(i16::try_from(residue).unwrap() - 125) / 1024.0
    })
}

#[test]
fn cold_tail_filters_previous_then_current_and_caches_current_outputs() {
    let parameters = selected_parameters();
    let previous = frame(0);
    let current = frame(1);
    let mut concatenated = Vec::with_capacity(3072);
    concatenated.extend_from_slice(&previous);
    concatenated.extend_from_slice(&current);

    let expected = preview_filter_block(
        &parameters,
        &parameters.cold_state().unwrap(),
        &concatenated,
    )
    .unwrap();
    let (expected_lanes, expected_filter_state) = expected.into_parts();
    let transition = preview_diffusion_tail(&parameters, None, true, &previous, &current).unwrap();

    let DiffusionTailTransition::Active(proposal) = transition else {
        panic!("active cold tail must produce a proposal");
    };
    let (previous_lanes, current_lanes, next_state) = (*proposal).into_parts();
    for lane in 0..4 {
        assert_eq!(previous_lanes[lane], expected_lanes[lane][..1536]);
        assert_eq!(current_lanes[lane], expected_lanes[lane][1536..]);
        assert_eq!(
            &next_state.previous_filter_outputs().unwrap()[lane],
            &current_lanes[lane]
        );
    }
    assert_eq!(
        next_state.filter_state().sha256_hex(),
        expected_filter_state.sha256_hex()
    );
}

#[test]
fn warm_tail_uses_cached_previous_output_and_processes_current_pcm_only() {
    let parameters = selected_parameters();
    let previous = frame(2);
    let current = frame(3);
    let cold = preview_diffusion_tail(&parameters, None, true, &previous, &current).unwrap();
    let DiffusionTailTransition::Active(cold_proposal) = cold else {
        panic!("active cold tail must produce a proposal");
    };
    let (_, _, warm_before) = (*cold_proposal).into_parts();
    let warm_input = frame(4);
    let expected =
        preview_filter_block(&parameters, warm_before.filter_state(), &warm_input).unwrap();
    let (expected_current, expected_filter_state) = expected.into_parts();
    let transition = preview_diffusion_tail(
        &parameters,
        Some(&warm_before),
        true,
        &frame(99),
        &warm_input,
    )
    .unwrap();

    let DiffusionTailTransition::Active(proposal) = transition else {
        panic!("active warm tail must produce a proposal");
    };
    let (previous_outputs, current_outputs, next_state) = (*proposal).into_parts();
    for lane in 0..4 {
        assert_eq!(
            previous_outputs[lane],
            warm_before.previous_filter_outputs().unwrap()[lane]
        );
        assert_eq!(current_outputs[lane], expected_current[lane]);
        assert_eq!(
            &next_state.previous_filter_outputs().unwrap()[lane],
            &current_outputs[lane]
        );
    }
    assert_eq!(
        next_state.filter_state().sha256_hex(),
        expected_filter_state.sha256_hex()
    );
}

#[test]
fn inactive_removes_the_tail_and_reactivation_restarts_from_cold() {
    let parameters = selected_parameters();
    let previous = frame(5);
    let current = frame(6);
    let cold = preview_diffusion_tail(&parameters, None, true, &previous, &current).unwrap();
    let DiffusionTailTransition::Active(proposal) = cold else {
        panic!("active cold tail must produce a proposal");
    };
    let (_, _, before_inactive) = (*proposal).into_parts();

    assert!(matches!(
        preview_diffusion_tail(
            &parameters,
            Some(&before_inactive),
            false,
            &previous,
            &current
        )
        .unwrap(),
        DiffusionTailTransition::Removed
    ));

    let reactivated = preview_diffusion_tail(&parameters, None, true, &previous, &current).unwrap();
    let DiffusionTailTransition::Active(reactivated) = reactivated else {
        panic!("reactivated tail must start with a cold proposal");
    };
    let (_, _, reactivated_state) = (*reactivated).into_parts();
    let independent_cold =
        preview_diffusion_tail(&parameters, None, true, &previous, &current).unwrap();
    let DiffusionTailTransition::Active(independent_cold) = independent_cold else {
        panic!("independent cold tail must produce a proposal");
    };
    let (_, _, independent_state) = (*independent_cold).into_parts();
    assert_eq!(
        reactivated_state.filter_state().sha256_hex(),
        independent_state.filter_state().sha256_hex()
    );
}

#[test]
fn warm_tail_without_the_same_instance_output_cache_fails_closed() {
    let parameters = selected_parameters();
    let filter_state = parameters.cold_state().unwrap();
    let before = DiffusionTailState {
        filter_state,
        previous_filter_outputs: None,
    };
    assert_eq!(
        preview_diffusion_tail(&parameters, Some(&before), true, &frame(7), &frame(8))
            .unwrap_err()
            .code(),
        "STATE_INCOMPLETE"
    );
}
