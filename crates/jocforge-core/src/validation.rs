// pattern: Functional Core

use crate::{
    AdmObjectKind, AdmPosition, ReductionConfig, ReductionResult, ReductionSource,
    reduce_scene_with_previous,
};

const TEMPORAL_FRAME_COUNT: usize = 120;
const TEMPORAL_SIGNAL_SAMPLES: usize = 96;
const SLIDING_WINDOW_FRAMES: usize = 8;
const FRAME_SECONDS: f64 = 1536.0 / 48_000.0;

/// A deterministic source scene sampled at codec-frame boundaries.
#[derive(Clone, Debug, PartialEq)]
pub struct TemporalScene {
    pub name: &'static str,
    pub frames: Vec<Vec<ReductionSource>>,
}

/// Measured temporal stability and transition behaviour for one scene.
#[derive(Clone, Debug, PartialEq)]
pub struct TemporalMetrics {
    pub scene: &'static str,
    pub frame_count: usize,
    pub authored_source_count: usize,
    pub maximum_coded_object_count: usize,
    pub silent_object_drops: usize,
    pub maximum_source_reconstruction_error: f64,
    pub maximum_spatial_representation_error: f64,
    pub maximum_cross_object_leakage: f64,
    pub minimum_headroom_margin: f64,
    pub total_assignment_changes: usize,
    pub total_coded_identity_changes: usize,
    pub assignment_changes_per_minute: f64,
    pub max_sliding_window_changes: usize,
    pub minimum_coded_identity_lifetime_frames: usize,
    pub median_coded_identity_lifetime_frames: f64,
    pub reassignment_burstiness: f64,
    pub maximum_position_jump: f64,
    pub maximum_trajectory_error: f64,
    pub maximum_gain_jump: f64,
    pub maximum_pcm_boundary_discontinuity: f64,
    pub maximum_reconstruction_error_change: f64,
    pub maximum_speaker_render_discontinuity: f64,
    pub stationary_assignment_churn_after_initialization: usize,
}

/// Builds the six clean, repository-owned temporal validation scenes.
#[must_use]
pub fn temporal_corpus() -> Vec<TemporalScene> {
    [
        ("smooth-left-right", 3),
        ("front-rear", 3),
        ("floor-height", 3),
        ("crossing", 4),
        ("crowded-motion", 20),
        ("stationary-control", 20),
    ]
    .into_iter()
    .map(|(name, source_count)| build_scene(name, source_count))
    .collect()
}

/// Builds adversarial scenes for continuity decisions near assignment
/// boundaries and source activity transitions.
#[must_use]
pub fn adversarial_temporal_corpus() -> Vec<TemporalScene> {
    [
        "almost-equal-cost",
        "boundary-crossing",
        "alternating-levels",
        "temporary-silence",
        "correlated-diverge",
        "separated-correlated",
    ]
    .into_iter()
    .map(|name| build_scene(name, 20))
    .collect()
}

/// Runs the existing deterministic reducer over a temporal scene and reports
/// transition metrics without touching files or external processes.
pub fn analyze_temporal_scene(scene: &TemporalScene) -> Result<TemporalMetrics, String> {
    analyze_temporal_scene_with_config(scene, ReductionConfig::default())
}

/// Runs temporal analysis with explicit clean-side reduction controls.
pub fn analyze_temporal_scene_with_config(
    scene: &TemporalScene,
    config: ReductionConfig,
) -> Result<TemporalMetrics, String> {
    if scene.frames.len() < 2 {
        return Err("temporal scene requires at least two frames".to_owned());
    }
    let source_count = scene.frames.first().map_or(0, Vec::len);
    if source_count == 0 || scene.frames.iter().any(|frame| frame.len() != source_count) {
        return Err("temporal scene source counts must stay constant and non-empty".to_owned());
    }

    let mut reductions = Vec::with_capacity(scene.frames.len());
    let mut previous = None;
    for frame in &scene.frames {
        let result = reduce_scene_with_previous(frame, config, previous.as_ref())
            .map_err(|error| format!("failed to reduce temporal scene: {error}"))?;
        previous = Some(result.clone());
        reductions.push(result);
    }

    let transition_changes = reductions
        .iter()
        .skip(1)
        .map(|result| result.diagnostics.assignment_changes)
        .collect::<Vec<_>>();
    let total_assignment_changes = transition_changes.iter().sum::<usize>();
    let total_coded_identity_changes = reductions
        .iter()
        .skip(1)
        .map(|result| result.diagnostics.coded_identity_changes)
        .sum::<usize>();
    let max_sliding_window_changes = transition_changes
        .windows(SLIDING_WINDOW_FRAMES.min(transition_changes.len()).max(1))
        .map(|window| window.iter().sum())
        .max()
        .unwrap_or(0);
    let duration_minutes = scene.frames.len() as f64 * FRAME_SECONDS / 60.0;

    let mut lifetimes = Vec::new();
    for source_index in 0..source_count {
        let mut current_identity = coded_identity(&reductions[0], source_index)?;
        let mut current_lifetime = 1_usize;
        for reduction in reductions.iter().skip(1) {
            let identity = coded_identity(reduction, source_index)?;
            if identity == current_identity {
                current_lifetime += 1;
            } else {
                lifetimes.push(current_lifetime);
                current_identity = identity;
                current_lifetime = 1;
            }
        }
        lifetimes.push(current_lifetime);
    }
    lifetimes.sort_unstable();

    let mut maximum_position_jump: f64 = 0.0;
    let mut maximum_trajectory_error: f64 = 0.0;
    let mut maximum_gain_jump: f64 = 0.0;
    let mut maximum_pcm_boundary_discontinuity: f64 = 0.0;
    let mut maximum_reconstruction_error_change: f64 = 0.0;
    let mut maximum_speaker_render_discontinuity: f64 = 0.0;
    for pair in reductions.windows(2) {
        let [previous, current] = pair else {
            continue;
        };
        maximum_reconstruction_error_change = maximum_reconstruction_error_change.max(
            (current.diagnostics.source_reconstruction_error
                - previous.diagnostics.source_reconstruction_error)
                .abs(),
        );
        for source_index in 0..source_count {
            let previous_accounting = accounting(previous, source_index)?;
            let current_accounting = accounting(current, source_index)?;
            let previous_object = &previous.coded_objects[previous_accounting.coded_object_index];
            let current_object = &current.coded_objects[current_accounting.coded_object_index];
            maximum_trajectory_error =
                maximum_trajectory_error.max(current_accounting.spatial_error);
            if previous_accounting.coded_object_index != current_accounting.coded_object_index {
                maximum_position_jump = maximum_position_jump.max(position_distance(
                    previous_object.position,
                    current_object.position,
                ));
                maximum_gain_jump = maximum_gain_jump.max(
                    (previous_accounting.contribution_gain - current_accounting.contribution_gain)
                        .abs(),
                );
                let previous_last = previous_object.signal.last().copied().unwrap_or(0.0);
                let current_first = current_object.signal.first().copied().unwrap_or(0.0);
                maximum_pcm_boundary_discontinuity =
                    maximum_pcm_boundary_discontinuity.max((previous_last - current_first).abs());
                maximum_speaker_render_discontinuity = maximum_speaker_render_discontinuity.max(
                    stereo_boundary_discontinuity(previous_object, current_object),
                );
            }
        }
    }

    let silent_object_drops = reductions
        .iter()
        .map(|result| result.diagnostics.silent_object_drops)
        .sum();
    let maximum_source_reconstruction_error = reductions
        .iter()
        .map(|result| result.diagnostics.source_reconstruction_error)
        .fold(0.0, f64::max);
    let maximum_spatial_representation_error = reductions
        .iter()
        .map(|result| result.diagnostics.spatial_representation_error)
        .fold(0.0, f64::max);
    let maximum_cross_object_leakage = reductions
        .iter()
        .map(|result| result.diagnostics.cross_object_leakage)
        .fold(0.0, f64::max);
    let minimum_headroom_margin = reductions
        .iter()
        .map(|result| result.diagnostics.headroom_margin)
        .fold(f64::INFINITY, f64::min);
    let maximum_coded_object_count = reductions
        .iter()
        .map(|result| result.coded_objects.len())
        .max()
        .unwrap_or(0);
    let median_coded_identity_lifetime_frames = median(&lifetimes);
    Ok(TemporalMetrics {
        scene: scene.name,
        frame_count: scene.frames.len(),
        authored_source_count: source_count,
        maximum_coded_object_count,
        silent_object_drops,
        maximum_source_reconstruction_error,
        maximum_spatial_representation_error,
        maximum_cross_object_leakage,
        minimum_headroom_margin,
        total_assignment_changes,
        total_coded_identity_changes,
        assignment_changes_per_minute: total_assignment_changes as f64
            / duration_minutes.max(FRAME_SECONDS / 60.0),
        max_sliding_window_changes,
        minimum_coded_identity_lifetime_frames: lifetimes.first().copied().unwrap_or(0),
        median_coded_identity_lifetime_frames,
        reassignment_burstiness: if total_assignment_changes == 0 {
            0.0
        } else {
            max_sliding_window_changes as f64 / total_assignment_changes as f64
        },
        maximum_position_jump,
        maximum_trajectory_error,
        maximum_gain_jump,
        maximum_pcm_boundary_discontinuity,
        maximum_reconstruction_error_change,
        maximum_speaker_render_discontinuity,
        stationary_assignment_churn_after_initialization: if scene.name == "stationary-control" {
            total_assignment_changes
        } else {
            0
        },
    })
}

fn build_scene(name: &'static str, source_count: usize) -> TemporalScene {
    let frames = (0..TEMPORAL_FRAME_COUNT)
        .map(|frame_index| {
            (0..source_count)
                .map(|source_index| ReductionSource {
                    source_index,
                    object_id: format!("{name}:source-{source_index:02}"),
                    kind: AdmObjectKind::Objects,
                    position: scene_position(name, source_index, frame_index, source_count),
                    signal: scene_signal(name, source_index, frame_index),
                })
                .collect()
        })
        .collect();
    TemporalScene { name, frames }
}

fn scene_position(
    scene: &str,
    source_index: usize,
    frame_index: usize,
    source_count: usize,
) -> AdmPosition {
    let progress = frame_index as f64 / (TEMPORAL_FRAME_COUNT - 1) as f64;
    let base = -0.9 + (source_index % 10) as f64 * 0.2;
    match scene {
        "smooth-left-right" if source_index == 0 => AdmPosition {
            azimuth: -0.9 + progress * 1.8,
            elevation: 0.0,
            distance: 1.0,
        },
        "front-rear" if source_index == 0 => AdmPosition {
            azimuth: -0.15 + progress * 0.3,
            elevation: 0.0,
            distance: 0.25 + progress * 0.75,
        },
        "floor-height" if source_index == 0 => AdmPosition {
            azimuth: 0.0,
            elevation: -0.8 + progress * 1.6,
            distance: 1.0,
        },
        "crossing" if source_index == 0 => AdmPosition {
            azimuth: -0.8 + progress * 1.6,
            elevation: 0.0,
            distance: 1.0,
        },
        "crossing" if source_index == 1 => AdmPosition {
            azimuth: 0.8 - progress * 1.6,
            elevation: 0.0,
            distance: 1.0,
        },
        "almost-equal-cost" if source_index == 0 => AdmPosition {
            azimuth: -0.12 + 0.015 * (frame_index % 2) as f64,
            elevation: 0.0,
            distance: 1.0,
        },
        "almost-equal-cost" if source_index == 1 => AdmPosition {
            azimuth: 0.12 - 0.015 * (frame_index % 2) as f64,
            elevation: 0.0,
            distance: 1.0,
        },
        "boundary-crossing" if source_index == 0 => AdmPosition {
            azimuth: -0.08 + 0.16 * progress,
            elevation: 0.0,
            distance: 1.0,
        },
        "boundary-crossing" if source_index == 1 => AdmPosition {
            azimuth: 0.08 - 0.16 * progress,
            elevation: 0.0,
            distance: 1.0,
        },
        "almost-equal-cost"
        | "alternating-levels"
        | "temporary-silence"
        | "correlated-diverge"
        | "separated-correlated"
        | "stationary-control" => AdmPosition {
            azimuth: base,
            elevation: -0.4 + (source_index / 10) as f64 * 0.8,
            distance: 0.5 + (source_index % 5) as f64 * 0.15,
        },
        "crowded-motion" => AdmPosition {
            azimuth: base + 0.08 * (progress * 12.0 + source_index as f64).sin(),
            elevation: -0.4 + (source_index / 10) as f64 * 0.8,
            distance: 0.5 + (source_index % 5) as f64 * 0.15,
        },
        _ => AdmPosition {
            azimuth: if source_index == 0 { 0.0 } else { base },
            elevation: (source_index as f64 / source_count.max(1) as f64) * 0.5,
            distance: 1.0,
        },
    }
}

fn scene_signal(scene: &str, source_index: usize, frame_index: usize) -> Vec<f64> {
    let phase = if scene == "stationary-control" {
        0.0
    } else {
        frame_index as f64 * 0.03125
    };
    let mut frequency = 0.11 + source_index as f64 * 0.013;
    let mut amplitude = 0.01 + (source_index % 3) as f64 * 0.002;
    if scene == "alternating-levels" && source_index == 0 {
        amplitude = if frame_index % 2 == 0 { 0.02 } else { 0.0001 };
    }
    if scene == "temporary-silence" && source_index == 0 && frame_index % 20 < 4 {
        amplitude = 0.0;
    }
    if (scene == "correlated-diverge" || scene == "separated-correlated") && source_index == 1 {
        let correlated = if scene == "correlated-diverge" {
            frame_index < TEMPORAL_FRAME_COUNT / 2
        } else {
            frame_index >= TEMPORAL_FRAME_COUNT / 2
        };
        frequency = if correlated { 0.11 } else { 0.23 };
    }
    (0..TEMPORAL_SIGNAL_SAMPLES)
        .map(|sample| amplitude * (sample as f64 * frequency + phase).sin())
        .collect()
}

fn coded_identity(result: &ReductionResult, source_index: usize) -> Result<usize, String> {
    Ok(accounting(result, source_index)?.coded_object_index)
}

fn accounting(
    result: &ReductionResult,
    source_index: usize,
) -> Result<&crate::SourceAccounting, String> {
    result
        .source_accounting
        .iter()
        .find(|accounting| accounting.source_index == source_index)
        .ok_or_else(|| format!("temporal result is missing source {source_index}"))
}

fn position_distance(left: AdmPosition, right: AdmPosition) -> f64 {
    let azimuth = left.azimuth - right.azimuth;
    let elevation = left.elevation - right.elevation;
    let distance = left.distance - right.distance;
    (azimuth * azimuth + elevation * elevation + distance * distance).sqrt()
}

fn stereo_boundary_discontinuity(
    previous: &crate::ReducedObject,
    current: &crate::ReducedObject,
) -> f64 {
    let previous_value = previous.signal.last().copied().unwrap_or(0.0);
    let current_value = current.signal.first().copied().unwrap_or(0.0);
    let previous_render = [
        previous_value * (1.0 - previous.position.azimuth.clamp(-1.0, 1.0)) * 0.5,
        previous_value * (1.0 + previous.position.azimuth.clamp(-1.0, 1.0)) * 0.5,
    ];
    let current_render = [
        current_value * (1.0 - current.position.azimuth.clamp(-1.0, 1.0)) * 0.5,
        current_value * (1.0 + current.position.azimuth.clamp(-1.0, 1.0)) * 0.5,
    ];
    previous_render
        .into_iter()
        .zip(current_render)
        .map(|(left, right)| (left - right).abs())
        .fold(0.0, f64::max)
}

fn median(values: &[usize]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let middle = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[middle - 1] + values[middle]) as f64 / 2.0
    } else {
        values[middle] as f64
    }
}
