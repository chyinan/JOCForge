// pattern: Functional Core

use jocforge_core::{
    ReductionConfig, adversarial_temporal_corpus, analyze_temporal_scene,
    analyze_temporal_scene_with_config, temporal_corpus,
};

#[test]
fn temporal_corpus_contains_the_required_six_controlled_scenes() {
    let scenes = temporal_corpus();
    let names = scenes.iter().map(|scene| scene.name).collect::<Vec<_>>();

    assert_eq!(names.len(), 6);
    for required in [
        "smooth-left-right",
        "front-rear",
        "floor-height",
        "crossing",
        "crowded-motion",
        "stationary-control",
    ] {
        assert!(names.contains(&required));
    }
}

#[test]
fn stationary_temporal_control_has_no_steady_state_assignment_churn() {
    let scene = temporal_corpus()
        .into_iter()
        .find(|scene| scene.name == "stationary-control")
        .expect("stationary control");
    let metrics = analyze_temporal_scene(&scene).expect("temporal analysis");

    assert_eq!(metrics.stationary_assignment_churn_after_initialization, 0);
    assert_eq!(metrics.max_sliding_window_changes, 0);
}

#[test]
fn crowded_temporal_control_reduces_all_sources_without_silent_drops() {
    let scene = temporal_corpus()
        .into_iter()
        .find(|scene| scene.name == "crowded-motion")
        .expect("crowded motion");
    let metrics = analyze_temporal_scene(&scene).expect("temporal analysis");

    assert!(metrics.authored_source_count > 15);
    assert!(metrics.maximum_coded_object_count <= 15);
    assert_eq!(metrics.silent_object_drops, 0);
    assert!(metrics.assignment_changes_per_minute.is_finite());
}

#[test]
fn continuity_weight_reduces_membership_thrashing_without_changing_identity_metric() {
    let scene = temporal_corpus()
        .into_iter()
        .find(|scene| scene.name == "crowded-motion")
        .expect("crowded motion");
    let unstable = analyze_temporal_scene_with_config(
        &scene,
        ReductionConfig {
            temporal_continuity_weight: 0.0,
            ..ReductionConfig::default()
        },
    )
    .expect("unregularized temporal analysis");
    let stable = analyze_temporal_scene(&scene).expect("regularized temporal analysis");

    assert!(unstable.total_assignment_changes > stable.total_assignment_changes);
    assert_eq!(unstable.total_coded_identity_changes, 0);
    assert_eq!(stable.total_coded_identity_changes, 0);
}

#[test]
fn adversarial_temporal_corpus_contains_the_required_six_cases() {
    let scenes = adversarial_temporal_corpus();
    let names = scenes.iter().map(|scene| scene.name).collect::<Vec<_>>();

    assert_eq!(names.len(), 6);
    for required in [
        "almost-equal-cost",
        "boundary-crossing",
        "alternating-levels",
        "temporary-silence",
        "correlated-diverge",
        "separated-correlated",
    ] {
        assert!(names.contains(&required));
    }
}

#[test]
fn adversarial_temporal_corpus_is_finite_and_reducer_safe() {
    for scene in adversarial_temporal_corpus() {
        let metrics = analyze_temporal_scene(&scene).expect("adversarial temporal analysis");
        assert!(metrics.assignment_changes_per_minute.is_finite());
        assert!(metrics.maximum_position_jump.is_finite());
        assert!(metrics.maximum_pcm_boundary_discontinuity.is_finite());
        assert!(metrics.maximum_gain_jump.is_finite());
    }
}

#[test]
fn adversarial_temporal_corpus_rejects_unregularized_thrashing() {
    let unstable_config = ReductionConfig {
        temporal_continuity_weight: 0.0,
        ..ReductionConfig::default()
    };
    let unstable = adversarial_temporal_corpus()
        .iter()
        .map(|scene| analyze_temporal_scene_with_config(scene, unstable_config))
        .collect::<Result<Vec<_>, _>>()
        .expect("unregularized adversarial analysis");
    let stable = adversarial_temporal_corpus()
        .iter()
        .map(analyze_temporal_scene)
        .collect::<Result<Vec<_>, _>>()
        .expect("regularized adversarial analysis");

    assert!(
        unstable
            .iter()
            .any(|metrics| metrics.total_assignment_changes > 0)
    );
    assert!(
        stable
            .iter()
            .all(|metrics| metrics.total_assignment_changes == 0)
    );
}
