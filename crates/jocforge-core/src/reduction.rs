// pattern: Functional Core

use std::{collections::BTreeSet, fmt};

use crate::{AdmObjectKind, AdmPosition};

/// Clean-side maximum number of coded dynamic-object representations.
pub const MAX_CODED_OBJECTS: usize = 15;

const DEFAULT_SIGNATURE_BINS: usize = 16;
const DEFAULT_OUTPUT_PEAK_LIMIT: f64 = 0.90;
const DEFAULT_SIGNAL_WEIGHT: f64 = 0.55;
const DEFAULT_SPATIAL_WEIGHT: f64 = 0.35;
const DEFAULT_ENERGY_WEIGHT: f64 = 0.10;
const DEFAULT_ENERGY_FLOOR: f64 = 1.0e-12;
const DEFAULT_TEMPORAL_CONTINUITY_WEIGHT: f64 = 0.15;
const DISTANCE_TIE_EPSILON: f64 = 1.0e-12;

/// Project-owned controls for deterministic authored-object reduction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReductionConfig {
    /// Hard coded-object limit, bounded by the clean project maximum.
    pub max_coded_objects: usize,
    /// Number of equally sized signal-mean bins used by the assignment metric.
    pub signature_bins: usize,
    /// Relative weight of normalized signal-signature distance.
    pub signal_weight: f64,
    /// Relative weight of normalized spatial distance.
    pub spatial_weight: f64,
    /// Relative weight of normalized RMS-energy distance.
    pub energy_weight: f64,
    /// Maximum absolute coded-object sample before deterministic scaling.
    pub output_peak_limit: f64,
    /// Additive cost for moving a source to a different prior-frame cluster.
    pub temporal_continuity_weight: f64,
}

impl Default for ReductionConfig {
    fn default() -> Self {
        Self {
            max_coded_objects: MAX_CODED_OBJECTS,
            signature_bins: DEFAULT_SIGNATURE_BINS,
            signal_weight: DEFAULT_SIGNAL_WEIGHT,
            spatial_weight: DEFAULT_SPATIAL_WEIGHT,
            energy_weight: DEFAULT_ENERGY_WEIGHT,
            output_peak_limit: DEFAULT_OUTPUT_PEAK_LIMIT,
            temporal_continuity_weight: DEFAULT_TEMPORAL_CONTINUITY_WEIGHT,
        }
    }
}

/// One authored source prepared by the imperative ingest shell.
#[derive(Clone, Debug, PartialEq)]
pub struct ReductionSource {
    pub source_index: usize,
    pub object_id: String,
    pub kind: AdmObjectKind,
    pub position: AdmPosition,
    pub signal: Vec<f64>,
}

/// One coded representation and its explicit authored-source membership.
#[derive(Clone, Debug, PartialEq)]
pub struct ReducedObject {
    pub coded_index: usize,
    pub representative_source_index: usize,
    pub source_indices: Vec<usize>,
    pub contribution_gains: Vec<f64>,
    pub position: AdmPosition,
    pub signal: Vec<f64>,
}

/// Accounting record proving how one authored source reaches a coded object.
#[derive(Clone, Debug, PartialEq)]
pub struct SourceAccounting {
    pub source_index: usize,
    pub coded_object_index: usize,
    pub contribution_gain: f64,
    pub source_energy: f64,
    pub represented_energy: f64,
    pub relative_error: f64,
    pub spatial_error: f64,
}

/// Externally reportable quality and stability measurements for one reduction.
#[derive(Clone, Debug, PartialEq)]
pub struct ReductionDiagnostics {
    pub authored_source_count: usize,
    pub coded_object_count: usize,
    pub silent_object_drops: usize,
    pub source_reconstruction_error: f64,
    pub spatial_representation_error: f64,
    pub cross_object_leakage: f64,
    pub max_coded_peak: f64,
    pub headroom_margin: f64,
    pub assignment_changes: usize,
    pub coded_identity_changes: usize,
}

/// Pure result of the clean authored-to-coded reduction stage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReductionResult {
    pub coded_objects: Vec<ReducedObject>,
    pub source_accounting: Vec<SourceAccounting>,
    pub diagnostics: ReductionDiagnostics,
}

/// Errors raised at the reduction core boundary.
#[derive(Clone, Debug, PartialEq)]
pub enum ReductionError {
    EmptySourceSet,
    InvalidBudget(usize),
    InvalidConfiguration(&'static str),
    DuplicateSourceIndex(usize),
    EmptySignal(usize),
    SignalLengthMismatch {
        source_index: usize,
        expected: usize,
        actual: usize,
    },
    NonFiniteSignal(usize),
    NonFinitePosition(usize),
}

impl fmt::Display for ReductionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySourceSet => formatter.write_str("reduction source set is empty"),
            Self::InvalidBudget(value) => write!(
                formatter,
                "reduction budget {value} is outside 1..={MAX_CODED_OBJECTS}"
            ),
            Self::InvalidConfiguration(detail) => {
                write!(formatter, "invalid reduction configuration: {detail}")
            }
            Self::DuplicateSourceIndex(index) => {
                write!(formatter, "duplicate reduction source index {index}")
            }
            Self::EmptySignal(index) => {
                write!(formatter, "reduction source {index} has an empty signal")
            }
            Self::SignalLengthMismatch {
                source_index,
                expected,
                actual,
            } => write!(
                formatter,
                "reduction source {source_index} signal length differs: expected {expected}, got {actual}"
            ),
            Self::NonFiniteSignal(index) => write!(
                formatter,
                "reduction source {index} contains a non-finite signal value"
            ),
            Self::NonFinitePosition(index) => write!(
                formatter,
                "reduction source {index} has a non-finite position"
            ),
        }
    }
}

impl std::error::Error for ReductionError {}

/// Reduces authored source signals to a deterministic bounded representation.
pub fn reduce_scene(
    sources: &[ReductionSource],
    config: ReductionConfig,
) -> Result<ReductionResult, ReductionError> {
    reduce_scene_internal(sources, config, None, None)
}

/// Reduces a scene while reusing the previous coded representatives when the
/// authored source identities are still available. This is a deterministic
/// continuity policy; changed source membership still falls back to fresh
/// seed selection.
pub fn reduce_scene_with_previous(
    sources: &[ReductionSource],
    config: ReductionConfig,
    previous: Option<&ReductionResult>,
) -> Result<ReductionResult, ReductionError> {
    let preferred_seeds = previous.and_then(|result| {
        let indices = result
            .coded_objects
            .iter()
            .map(|object| {
                sources
                    .iter()
                    .position(|source| source.source_index == object.representative_source_index)
            })
            .collect::<Option<Vec<_>>>()?;
        if indices.len() == sources.len().min(config.max_coded_objects) {
            Some(indices)
        } else {
            None
        }
    });
    let previous_assignments = preferred_seeds.as_ref().map(|_| {
        sources
            .iter()
            .map(|source| {
                previous.and_then(|result| {
                    result
                        .source_accounting
                        .iter()
                        .find(|accounting| accounting.source_index == source.source_index)
                        .map(|accounting| accounting.coded_object_index)
                })
            })
            .collect::<Vec<_>>()
    });
    let mut result = reduce_scene_internal(
        sources,
        config,
        preferred_seeds.as_deref(),
        previous_assignments.as_deref(),
    )?;
    if let Some(previous) = previous {
        result.diagnostics.assignment_changes = reduction_assignment_changes(previous, &result);
        result.diagnostics.coded_identity_changes = reduction_identity_changes(previous, &result);
    }
    Ok(result)
}

fn reduce_scene_internal(
    sources: &[ReductionSource],
    config: ReductionConfig,
    preferred_seeds: Option<&[usize]>,
    previous_assignments: Option<&[Option<usize>]>,
) -> Result<ReductionResult, ReductionError> {
    validate_inputs(sources, config)?;
    let source_count = sources.len();
    let signatures = sources
        .iter()
        .map(|source| normalized_signature(&source.signal, config.signature_bins))
        .collect::<Vec<_>>();
    let energies = sources
        .iter()
        .map(|source| signal_energy(&source.signal))
        .collect::<Vec<_>>();
    let spatial_scale = spatial_scale(sources);
    let seeds = preferred_seeds.map_or_else(
        || {
            choose_seeds(
                sources,
                &signatures,
                &energies,
                spatial_scale,
                source_count.min(config.max_coded_objects),
                config,
            )
        },
        ToOwned::to_owned,
    );
    let coded_count = seeds.len();
    let assignments = assign_sources(
        sources,
        &signatures,
        &energies,
        spatial_scale,
        &seeds,
        config,
        previous_assignments,
    );

    let mut clusters = vec![Vec::<usize>::new(); seeds.len()];
    for (source_index, cluster_index) in assignments.iter().copied().enumerate() {
        clusters[cluster_index].push(source_index);
    }

    let mut coded_objects = Vec::with_capacity(clusters.len());
    let mut source_accounting = Vec::with_capacity(source_count);
    let mut total_source_energy = 0.0;
    let mut total_source_error_energy = 0.0;
    let mut total_spatial_error = 0.0;
    let mut total_cluster_energy = 0.0;
    let mut total_other_energy = 0.0;
    let mut silent_object_drops = 0;
    let mut max_coded_peak: f64 = 0.0;

    for (coded_index, members) in clusters.iter().enumerate() {
        let (signal, gains, scale) = combine_cluster(sources, members, config.output_peak_limit);
        let position = aggregate_position(sources, members, &energies);
        let cluster_energy = signal_energy(&signal);
        total_cluster_energy += cluster_energy;
        max_coded_peak = max_coded_peak.max(signal_peak(&signal));
        let source_indices = members
            .iter()
            .map(|index| sources[*index].source_index)
            .collect::<Vec<_>>();
        let representative_source_index = sources[seeds[coded_index]].source_index;
        for (member_offset, source_array_index) in members.iter().copied().enumerate() {
            let source = &sources[source_array_index];
            let source_energy = energies[source_array_index];
            let gain = gains[member_offset];
            let represented_energy = source_energy * gain * gain;
            let relative_error = contribution_error(&source.signal, gain);
            let spatial_error = position_distance(source.position, position) / spatial_scale;
            total_source_energy += source_energy;
            total_source_error_energy += source_energy * relative_error * relative_error;
            total_spatial_error += source_energy * spatial_error;
            if source_energy > DEFAULT_ENERGY_FLOOR && gain <= DEFAULT_ENERGY_FLOOR {
                silent_object_drops += 1;
            }
            let other_signal_energy =
                other_contribution_energy(sources, members, &gains, member_offset, scale);
            total_other_energy += other_signal_energy;
            source_accounting.push(SourceAccounting {
                source_index: source.source_index,
                coded_object_index: coded_index,
                contribution_gain: gain,
                source_energy,
                represented_energy,
                relative_error,
                spatial_error,
            });
        }
        coded_objects.push(ReducedObject {
            coded_index,
            representative_source_index,
            source_indices,
            contribution_gains: gains,
            position,
            signal,
        });
    }
    source_accounting.sort_by_key(|accounting| accounting.source_index);

    let source_reconstruction_error = if total_source_energy <= DEFAULT_ENERGY_FLOOR {
        0.0
    } else {
        (total_source_error_energy / total_source_energy).sqrt()
    };
    let spatial_representation_error = if total_source_energy <= DEFAULT_ENERGY_FLOOR {
        0.0
    } else {
        total_spatial_error / total_source_energy
    };
    let cross_object_leakage = if total_cluster_energy <= DEFAULT_ENERGY_FLOOR {
        0.0
    } else {
        (total_other_energy / total_cluster_energy).min(1.0)
    };
    Ok(ReductionResult {
        coded_objects,
        source_accounting,
        diagnostics: ReductionDiagnostics {
            authored_source_count: source_count,
            coded_object_count: coded_count,
            silent_object_drops,
            source_reconstruction_error,
            spatial_representation_error,
            cross_object_leakage,
            max_coded_peak,
            headroom_margin: (config.output_peak_limit - max_coded_peak).max(0.0),
            assignment_changes: 0,
            coded_identity_changes: 0,
        },
    })
}

/// Counts source-to-coded assignment changes between two pure reduction results.
#[must_use]
pub fn reduction_assignment_changes(left: &ReductionResult, right: &ReductionResult) -> usize {
    let mut right_assignments = right
        .source_accounting
        .iter()
        .map(|accounting| (accounting.source_index, accounting.coded_object_index))
        .collect::<Vec<_>>();
    right_assignments.sort_unstable_by_key(|(source_index, _)| *source_index);
    left.source_accounting
        .iter()
        .map(|accounting| (accounting.source_index, accounting.coded_object_index))
        .zip(right_assignments)
        .filter(|(left_assignment, right_assignment)| left_assignment != right_assignment)
        .count()
}

/// Counts coded representative identities that disappeared between reductions.
///
/// Coded ordinals are serialization positions, not identities. A stable
/// representative source remains the same identity even if a caller presents
/// the coded-object vector in a different order.
#[must_use]
pub fn reduction_identity_changes(left: &ReductionResult, right: &ReductionResult) -> usize {
    let left_identities = left
        .coded_objects
        .iter()
        .map(|object| object.representative_source_index)
        .collect::<BTreeSet<_>>();
    let right_identities = right
        .coded_objects
        .iter()
        .map(|object| object.representative_source_index)
        .collect::<BTreeSet<_>>();
    left_identities.difference(&right_identities).count()
}

fn validate_inputs(
    sources: &[ReductionSource],
    config: ReductionConfig,
) -> Result<(), ReductionError> {
    if sources.is_empty() {
        return Err(ReductionError::EmptySourceSet);
    }
    if !(1..=MAX_CODED_OBJECTS).contains(&config.max_coded_objects) {
        return Err(ReductionError::InvalidBudget(config.max_coded_objects));
    }
    if !(4..=64).contains(&config.signature_bins) {
        return Err(ReductionError::InvalidConfiguration(
            "signature bins must be 4..=64",
        ));
    }
    if !config.output_peak_limit.is_finite() || !(0.0..=1.0).contains(&config.output_peak_limit) {
        return Err(ReductionError::InvalidConfiguration(
            "output peak limit must be finite and in 0..=1",
        ));
    }
    if !config.temporal_continuity_weight.is_finite() || config.temporal_continuity_weight < 0.0 {
        return Err(ReductionError::InvalidConfiguration(
            "temporal continuity weight must be finite and non-negative",
        ));
    }
    if [
        config.signal_weight,
        config.spatial_weight,
        config.energy_weight,
    ]
    .iter()
    .any(|weight| !weight.is_finite() || *weight < 0.0)
    {
        return Err(ReductionError::InvalidConfiguration(
            "objective weights must be finite and non-negative",
        ));
    }
    if config.signal_weight + config.spatial_weight + config.energy_weight <= 0.0 {
        return Err(ReductionError::InvalidConfiguration(
            "objective weights must have a positive sum",
        ));
    }
    let mut source_indices = BTreeSet::new();
    let expected_length = sources.first().map_or(0, |source| source.signal.len());
    for source in sources {
        if !source_indices.insert(source.source_index) {
            return Err(ReductionError::DuplicateSourceIndex(source.source_index));
        }
        if source.signal.is_empty() {
            return Err(ReductionError::EmptySignal(source.source_index));
        }
        if source.signal.len() != expected_length {
            return Err(ReductionError::SignalLengthMismatch {
                source_index: source.source_index,
                expected: expected_length,
                actual: source.signal.len(),
            });
        }
        if source.signal.iter().any(|value| !value.is_finite()) {
            return Err(ReductionError::NonFiniteSignal(source.source_index));
        }
        if !source.position.is_finite() {
            return Err(ReductionError::NonFinitePosition(source.source_index));
        }
    }
    Ok(())
}

fn signal_energy(signal: &[f64]) -> f64 {
    signal.iter().map(|value| value * value).sum::<f64>() / signal.len() as f64
}

fn signal_peak(signal: &[f64]) -> f64 {
    signal.iter().map(|value| value.abs()).fold(0.0, f64::max)
}

fn normalized_signature(signal: &[f64], bins: usize) -> Vec<f64> {
    let mut signature = vec![0.0; bins];
    for (bin, value) in signature.iter_mut().enumerate() {
        let start = bin * signal.len() / bins;
        let end = ((bin + 1) * signal.len() / bins)
            .max(start + 1)
            .min(signal.len());
        *value = signal[start..end].iter().sum::<f64>() / (end - start) as f64;
    }
    let norm = signature
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    if norm <= DEFAULT_ENERGY_FLOOR {
        return signature;
    }
    for value in &mut signature {
        *value /= norm;
    }
    signature
}

fn spatial_scale(sources: &[ReductionSource]) -> f64 {
    let mut scale: f64 = 1.0;
    for left in sources {
        for right in sources {
            scale = scale.max(position_distance(left.position, right.position));
        }
    }
    scale
}

fn position_distance(left: AdmPosition, right: AdmPosition) -> f64 {
    let azimuth = left.azimuth - right.azimuth;
    let elevation = left.elevation - right.elevation;
    let distance = left.distance - right.distance;
    (azimuth * azimuth + elevation * elevation + distance * distance).sqrt()
}

fn source_distance(
    left: usize,
    right: usize,
    sources: &[ReductionSource],
    signatures: &[Vec<f64>],
    energies: &[f64],
    spatial_scale: f64,
    config: ReductionConfig,
) -> f64 {
    let signature_distance = signatures[left]
        .iter()
        .zip(&signatures[right])
        .map(|(left, right)| {
            let delta = left - right;
            delta * delta
        })
        .sum::<f64>()
        / signatures[left].len() as f64;
    let spatial_distance =
        position_distance(sources[left].position, sources[right].position) / spatial_scale;
    let spatial_distance = spatial_distance * spatial_distance;
    let left_rms = energies[left].sqrt();
    let right_rms = energies[right].sqrt();
    let energy_distance = (left_rms - right_rms) / (left_rms + right_rms + DEFAULT_ENERGY_FLOOR);
    let energy_distance = energy_distance * energy_distance;
    config.signal_weight * signature_distance
        + config.spatial_weight * spatial_distance
        + config.energy_weight * energy_distance
}

fn choose_seeds(
    sources: &[ReductionSource],
    signatures: &[Vec<f64>],
    energies: &[f64],
    spatial_scale: f64,
    count: usize,
    config: ReductionConfig,
) -> Vec<usize> {
    let first = (0..sources.len())
        .max_by(|&left, &right| compare_energy_then_index(left, right, energies, sources))
        .unwrap_or(0);
    let mut seeds = vec![first];
    while seeds.len() < count {
        let candidate = (0..sources.len())
            .filter(|index| !seeds.contains(index))
            .max_by(|&left, &right| {
                let left_distance = seeds
                    .iter()
                    .map(|seed| {
                        source_distance(
                            left,
                            *seed,
                            sources,
                            signatures,
                            energies,
                            spatial_scale,
                            config,
                        )
                    })
                    .fold(f64::INFINITY, f64::min);
                let right_distance = seeds
                    .iter()
                    .map(|seed| {
                        source_distance(
                            right,
                            *seed,
                            sources,
                            signatures,
                            energies,
                            spatial_scale,
                            config,
                        )
                    })
                    .fold(f64::INFINITY, f64::min);
                compare_max_score_then_index(left_distance, right_distance, left, right, sources)
            });
        let Some(candidate) = candidate else { break };
        seeds.push(candidate);
    }
    seeds.sort_unstable_by_key(|index| sources[*index].source_index);
    seeds
}

fn assign_sources(
    sources: &[ReductionSource],
    signatures: &[Vec<f64>],
    energies: &[f64],
    spatial_scale: f64,
    seeds: &[usize],
    config: ReductionConfig,
    previous_assignments: Option<&[Option<usize>]>,
) -> Vec<usize> {
    (0..sources.len())
        .map(|source_index| {
            if let Some(cluster_index) = seeds.iter().position(|seed| *seed == source_index) {
                return cluster_index;
            }
            seeds
                .iter()
                .enumerate()
                .min_by(|(left_cluster, left_seed), (right_cluster, right_seed)| {
                    let left_distance = source_distance(
                        source_index,
                        **left_seed,
                        sources,
                        signatures,
                        energies,
                        spatial_scale,
                        config,
                    );
                    let left_distance = continuity_cost(
                        left_distance,
                        previous_assignments,
                        source_index,
                        *left_cluster,
                        config,
                    );
                    let right_distance = source_distance(
                        source_index,
                        **right_seed,
                        sources,
                        signatures,
                        energies,
                        spatial_scale,
                        config,
                    );
                    let right_distance = continuity_cost(
                        right_distance,
                        previous_assignments,
                        source_index,
                        *right_cluster,
                        config,
                    );
                    compare_min_score_then_index(
                        left_distance,
                        right_distance,
                        **left_seed,
                        **right_seed,
                        sources,
                    )
                    .then_with(|| left_cluster.cmp(right_cluster))
                })
                .map_or(0, |(cluster_index, _)| cluster_index)
        })
        .collect()
}

fn continuity_cost(
    distance: f64,
    previous_assignments: Option<&[Option<usize>]>,
    source_index: usize,
    cluster_index: usize,
    config: ReductionConfig,
) -> f64 {
    let should_switch = previous_assignments
        .and_then(|assignments| assignments.get(source_index).copied().flatten())
        .is_some_and(|previous_cluster| previous_cluster != cluster_index);
    distance
        + if should_switch {
            config.temporal_continuity_weight
        } else {
            0.0
        }
}

fn combine_cluster(
    sources: &[ReductionSource],
    members: &[usize],
    peak_limit: f64,
) -> (Vec<f64>, Vec<f64>, f64) {
    let raw_weights = members
        .iter()
        .map(|index| {
            signal_energy(&sources[*index].signal)
                .sqrt()
                .max(DEFAULT_ENERGY_FLOOR)
        })
        .collect::<Vec<_>>();
    let norm = raw_weights
        .iter()
        .map(|weight| weight * weight)
        .sum::<f64>()
        .sqrt();
    let normalized_weights = raw_weights
        .iter()
        .map(|weight| weight / norm)
        .collect::<Vec<_>>();
    let mut signal = vec![0.0; sources[members[0]].signal.len()];
    for (member_offset, source_index) in members.iter().copied().enumerate() {
        for (output, input) in signal.iter_mut().zip(&sources[source_index].signal) {
            *output += normalized_weights[member_offset] * input;
        }
    }
    let peak = signal_peak(&signal);
    let scale = if peak > peak_limit && peak > DEFAULT_ENERGY_FLOOR {
        peak_limit / peak
    } else {
        1.0
    };
    for value in &mut signal {
        *value *= scale;
    }
    let gains = normalized_weights
        .iter()
        .map(|weight| weight * scale)
        .collect::<Vec<_>>();
    (signal, gains, scale)
}

fn aggregate_position(
    sources: &[ReductionSource],
    members: &[usize],
    energies: &[f64],
) -> AdmPosition {
    let weights = members
        .iter()
        .map(|index| energies[*index].sqrt().max(DEFAULT_ENERGY_FLOOR))
        .collect::<Vec<_>>();
    let total = weights.iter().sum::<f64>();
    let weighted = |select: fn(AdmPosition) -> f64| {
        members
            .iter()
            .zip(&weights)
            .map(|(index, weight)| select(sources[*index].position) * weight)
            .sum::<f64>()
            / total
    };
    AdmPosition {
        azimuth: weighted(|position| position.azimuth),
        elevation: weighted(|position| position.elevation),
        distance: weighted(|position| position.distance),
    }
}

fn contribution_error(signal: &[f64], gain: f64) -> f64 {
    let source_energy = signal_energy(signal);
    if source_energy <= DEFAULT_ENERGY_FLOOR {
        return 0.0;
    }
    let error_energy = signal
        .iter()
        .map(|value| {
            let error = *value - gain * *value;
            error * error
        })
        .sum::<f64>()
        / signal.len() as f64;
    (error_energy / source_energy).sqrt()
}

fn other_contribution_energy(
    sources: &[ReductionSource],
    members: &[usize],
    gains: &[f64],
    selected_offset: usize,
    scale: f64,
) -> f64 {
    if members.len() <= 1 {
        return 0.0;
    }
    let length = sources[members[0]].signal.len();
    let mut other = vec![0.0; length];
    for (member_offset, source_index) in members.iter().copied().enumerate() {
        if member_offset == selected_offset {
            continue;
        }
        let normalized_gain = if scale <= DEFAULT_ENERGY_FLOOR {
            0.0
        } else {
            gains[member_offset] / scale
        };
        for (output, input) in other.iter_mut().zip(&sources[source_index].signal) {
            *output += normalized_gain * input;
        }
    }
    signal_energy(&other)
}

fn compare_energy_then_index(
    left: usize,
    right: usize,
    energies: &[f64],
    sources: &[ReductionSource],
) -> std::cmp::Ordering {
    compare_max_score_then_index(energies[left], energies[right], left, right, sources)
}

fn compare_max_score_then_index(
    left_score: f64,
    right_score: f64,
    left: usize,
    right: usize,
    sources: &[ReductionSource],
) -> std::cmp::Ordering {
    if (left_score - right_score).abs() > DISTANCE_TIE_EPSILON {
        return left_score.total_cmp(&right_score);
    }
    sources[right].source_index.cmp(&sources[left].source_index)
}

fn compare_min_score_then_index(
    left_score: f64,
    right_score: f64,
    left: usize,
    right: usize,
    sources: &[ReductionSource],
) -> std::cmp::Ordering {
    if (left_score - right_score).abs() > DISTANCE_TIE_EPSILON {
        return left_score.total_cmp(&right_score);
    }
    sources[left].source_index.cmp(&sources[right].source_index)
}
