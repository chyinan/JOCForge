// pattern: Functional Core

use jocforge_core::{
    AdmObjectKind, AdmPosition, ReductionConfig, ReductionSource, reduce_scene,
    reduction_assignment_changes,
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
    assert_eq!(reduction_assignment_changes(&first, &second), 0);
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
