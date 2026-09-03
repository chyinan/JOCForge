// pattern: Functional Core

use jocforge_core::{
    AdmObjectKind, AdmPosition, ReductionConfig, ReductionSource,
    authored_source_assignment_changes, reduce_scene, representative_source_changes,
    speaker_render_error, unexplained_cluster_leakage,
};

fn source(ordinal: usize) -> ReductionSource {
    let signal = (0..96)
        .map(|sample| {
            let phase = (sample + ordinal * 7) as f64;
            0.02 * (phase * 0.17).sin() + 0.001 * (ordinal as f64 + 1.0)
        })
        .collect();
    ReductionSource {
        source_index: ordinal,
        object_id: format!("AO_{ordinal:04X}"),
        kind: AdmObjectKind::Objects,
        position: AdmPosition {
            azimuth: -1.0 + ordinal as f64 * 0.07,
            elevation: -0.4 + ordinal as f64 * 0.02,
            distance: 1.0 + ordinal as f64 * 0.01,
        },
        signal,
    }
}

#[test]
fn twenty_eight_sources_reduce_to_fifteen_without_silent_drops() {
    let sources = (0..28).map(source).collect::<Vec<_>>();

    let result = reduce_scene(&sources, ReductionConfig::default()).expect("scene reduction");

    assert_eq!(result.coded_objects.len(), 15);
    assert_eq!(result.source_accounting.len(), 28);
    assert_eq!(result.diagnostics.authored_source_count, 28);
    assert_eq!(result.diagnostics.silent_object_drops, 0);
    assert!(
        result
            .source_accounting
            .iter()
            .all(|accounting| accounting.contribution_gain > 0.0)
    );
    assert!(
        result
            .coded_objects
            .iter()
            .flat_map(|object| object.signal.iter())
            .all(|value| value.is_finite())
    );
    assert!(result.diagnostics.max_coded_peak <= 0.9);
}

#[test]
fn small_scene_keeps_one_coded_representation_per_source() {
    let sources = (0..3).map(source).collect::<Vec<_>>();

    let result = reduce_scene(&sources, ReductionConfig::default()).expect("scene reduction");

    assert_eq!(result.coded_objects.len(), 3);
    assert_eq!(
        result
            .coded_objects
            .iter()
            .map(|object| object.source_indices.clone())
            .collect::<Vec<_>>(),
        vec![vec![0], vec![1], vec![2]]
    );
    for (source, coded) in sources.iter().zip(&result.coded_objects) {
        assert_eq!(source.signal, coded.signal);
        assert_eq!(source.position, coded.position);
    }
}

#[test]
fn identical_windows_are_deterministic_and_have_zero_assignment_churn() {
    let sources = (0..28).map(source).collect::<Vec<_>>();
    let first = reduce_scene(&sources, ReductionConfig::default()).expect("first reduction");
    let second = reduce_scene(&sources, ReductionConfig::default()).expect("second reduction");

    assert_eq!(first, second);
    assert_eq!(authored_source_assignment_changes(&first, &second), 0);
}

#[test]
fn non_finite_source_input_is_rejected_at_the_core_boundary() {
    let mut invalid = source(0);
    invalid.signal[11] = f64::NAN;

    let error = reduce_scene(&[invalid], ReductionConfig::default())
        .expect_err("non-finite source must be rejected");

    assert!(error.to_string().contains("non-finite"));
}

#[test]
fn identical_sources_keep_each_selected_cluster_non_empty() {
    let template = source(0);
    let sources = (0..20)
        .map(|ordinal| ReductionSource {
            source_index: ordinal,
            object_id: format!("AO_{ordinal:04X}"),
            kind: template.kind,
            position: template.position,
            signal: template.signal.clone(),
        })
        .collect::<Vec<_>>();

    let result = reduce_scene(&sources, ReductionConfig::default()).expect("scene reduction");

    assert_eq!(result.coded_objects.len(), 15);
    assert!(
        result
            .coded_objects
            .iter()
            .all(|object| !object.source_indices.is_empty())
    );
}

#[test]
fn positive_contribution_gain_is_not_counted_as_a_silent_drop() {
    let sources = (0..20)
        .map(|ordinal| {
            let mut item = source(ordinal);
            item.signal = item
                .signal
                .into_iter()
                .map(|value| value * 0.0001)
                .collect();
            item
        })
        .collect::<Vec<_>>();

    let result = reduce_scene(&sources, ReductionConfig::default()).expect("scene reduction");

    assert_eq!(result.diagnostics.silent_object_drops, 0);
    assert!(
        result
            .source_accounting
            .iter()
            .all(|accounting| accounting.contribution_gain > 0.0)
    );
}

#[test]
fn representative_source_metric_ignores_member_reordering_when_representatives_persist() {
    let sources = (0..20).map(source).collect::<Vec<_>>();
    let first = reduce_scene(&sources, ReductionConfig::default()).expect("first reduction");
    let mut reordered = first.clone();
    reordered.coded_objects.swap(0, 1);

    assert_eq!(representative_source_changes(&first, &reordered), 0);
    assert_eq!(representative_source_changes(&first, &first), 0);
    assert_eq!(authored_source_assignment_changes(&first, &reordered), 0);
}

#[test]
fn representative_source_metric_counts_a_replacement_once() {
    let sources = (0..20).map(source).collect::<Vec<_>>();
    let first = reduce_scene(&sources, ReductionConfig::default()).expect("first reduction");
    let mut changed = first.clone();
    changed.coded_objects[0].representative_source_index = usize::MAX;

    assert_eq!(representative_source_changes(&first, &changed), 1);
}

#[test]
fn sparse_source_identity_keeps_its_prior_continuity_assignment() {
    let make_source = |source_index: usize, azimuth: f64| ReductionSource {
        source_index,
        object_id: format!("AO_{source_index:04X}"),
        kind: AdmObjectKind::Objects,
        position: AdmPosition {
            azimuth,
            elevation: 0.0,
            distance: 1.0,
        },
        signal: vec![if source_index == 5 { 0.5 } else { 1.0 }; 32],
    };
    let config = ReductionConfig {
        max_coded_objects: 2,
        temporal_continuity_weight: 0.15,
        ..ReductionConfig::default()
    };
    let previous_sources = vec![
        make_source(0, -1.0),
        make_source(2, 1.0),
        make_source(5, -0.1),
    ];
    let current_sources = vec![
        make_source(0, -1.0),
        make_source(5, 0.1),
        make_source(2, 1.0),
    ];

    let previous = reduce_scene(&previous_sources, config).expect("previous reduction");
    let previous_assignment = previous
        .source_accounting
        .iter()
        .find(|accounting| accounting.source_index == 5)
        .expect("sparse source accounting")
        .coded_object_index;
    let without_continuity = jocforge_core::reduce_scene_with_previous(
        &current_sources,
        ReductionConfig {
            temporal_continuity_weight: 0.0,
            ..config
        },
        Some(&previous),
    )
    .expect("unregularized current reduction");
    assert_ne!(
        without_continuity
            .source_accounting
            .iter()
            .find(|accounting| accounting.source_index == 5)
            .expect("unregularized sparse source accounting")
            .coded_object_index,
        previous_assignment
    );
    let current =
        jocforge_core::reduce_scene_with_previous(&current_sources, config, Some(&previous))
            .expect("current reduction");

    assert_eq!(
        current
            .source_accounting
            .iter()
            .find(|accounting| accounting.source_index == 5)
            .expect("current sparse source accounting")
            .coded_object_index,
        previous_assignment
    );
}

#[test]
fn clean_reduction_reports_no_unexplained_cluster_residual() {
    let sources = (0..20).map(source).collect::<Vec<_>>();
    let result = reduce_scene(&sources, ReductionConfig::default()).expect("scene reduction");

    let leakage = unexplained_cluster_leakage(&sources, &result).expect("leakage oracle");

    assert!(leakage.is_finite());
    assert!(leakage < 1.0e-24);
}

#[test]
fn clean_reduction_reports_a_quantitative_stereo_render_error() {
    let sources = vec![source(0)];
    let result = reduce_scene(&sources, ReductionConfig::default()).expect("scene reduction");

    let error = speaker_render_error(&sources, &result).expect("speaker render oracle");

    assert!(error.is_finite());
    assert!(error < 1.0e-12);
}
