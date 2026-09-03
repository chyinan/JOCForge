// pattern: Functional Core

use std::fmt;

const JOC_COARSE_QUANTUM: f64 = 820.0 / 4096.0;
// Absolute lower bound for ridge regularization in normalized PCM units.
// It keeps an all-silent or very small basis away from a singular solve.
const SOURCE_DERIVED_RIDGE: f64 = 1.0e-8;
// Five percent of mean basis energy is the scale-responsive regularization;
// the lower bound above supplies a finite floor for near-silent windows.
const SOURCE_DERIVED_RIDGE_RATIO: f64 = 0.05;
// The same floor used for bounded zero-energy decisions; it prevents a
// normalized residual from magnifying numerical noise in silent windows.
const SOURCE_DERIVED_ENERGY_FLOOR: f64 = 1.0e-12;
// A compact deterministic view limits per-frame solve cost while the carrier
// itself continues to use the complete source window.
const SOURCE_DERIVED_ANALYSIS_SAMPLES: usize = 64;
/// Shared normalized PCM peak limit for all public carrier paths, including LFE.
pub const SOURCE_DERIVED_CARRIER_PEAK_LIMIT: f64 = 0.90;
// Gaussian elimination treats a pivot below this normalized magnitude as
// numerically singular; ridge normally prevents this path for source-derived
// inputs, while the guard keeps public callers fail-closed.
const PIVOT_EPSILON: f64 = 1.0e-12;

/// A low-rank factorization with an explicit carrier scaling identity.
#[derive(Clone, Debug, PartialEq)]
pub struct RankFactorization {
    pub carriers: Vec<Vec<f64>>,
    pub ideal_matrix: Vec<Vec<f64>>,
    pub scaled_matrix: Vec<Vec<f64>>,
    pub scale_factor: f64,
    pub carrier_peak_before_scaling: f64,
    pub carrier_peak_after_scaling: f64,
}

/// Controlled source-to-carrier perturbation evidence for one profile.
#[derive(Clone, Debug, PartialEq)]
pub struct CarrierCausalityMeasurement {
    pub carrier_index: usize,
    pub source_index: usize,
    pub maximum_change: f64,
    pub carrier_changed: bool,
}

/// Quantization evidence for one full-matrix coefficient.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuantizedCoefficient {
    pub ideal: f64,
    pub encoded_symbol: u16,
    pub decoded: f64,
    pub error: f64,
}

/// Errors from the bounded rank-controlled factorization path.
#[derive(Clone, Debug, PartialEq)]
pub enum AnalysisError {
    EmptyObjectSet,
    EmptyObject,
    DimensionMismatch,
    InvalidRank { rank: usize, object_count: usize },
    SingularBasis,
    NonFiniteInput,
    NonFiniteCoefficient,
    CoefficientOutOfRange { step: f64 },
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyObjectSet => formatter.write_str("object PCM set is empty"),
            Self::EmptyObject => formatter.write_str("object PCM track is empty"),
            Self::DimensionMismatch => formatter.write_str("object PCM dimensions do not match"),
            Self::InvalidRank { rank, object_count } => write!(
                formatter,
                "rank {rank} is invalid for {object_count} objects"
            ),
            Self::SingularBasis => formatter.write_str("selected carrier basis is singular"),
            Self::NonFiniteInput => formatter.write_str("object PCM contains a non-finite value"),
            Self::NonFiniteCoefficient => {
                formatter.write_str("factorization produced a non-finite coefficient")
            }
            Self::CoefficientOutOfRange { step } => write!(
                formatter,
                "factorization coefficient quantizer step {step:.6} is outside -48..=47"
            ),
        }
    }
}

impl std::error::Error for AnalysisError {}

/// Factorizes objects against their first `rank` tracks and applies the
/// explicit identity `X_scaled = X / s`, `M_scaled = M * s` with the coarse
/// JOC step `s`. This is intentionally a deterministic controlled-fixture
/// method, not a perceptual encoder.
pub fn factorize_rank_controlled(
    objects: &[Vec<f64>],
    rank: usize,
) -> Result<RankFactorization, AnalysisError> {
    let first = objects.first().ok_or(AnalysisError::EmptyObjectSet)?;
    if first.is_empty() {
        return Err(AnalysisError::EmptyObject);
    }
    if rank == 0 || rank > objects.len() {
        return Err(AnalysisError::InvalidRank {
            rank,
            object_count: objects.len(),
        });
    }
    if objects.iter().any(|object| object.len() != first.len()) {
        return Err(AnalysisError::DimensionMismatch);
    }
    if objects.iter().flatten().any(|value| !value.is_finite()) {
        return Err(AnalysisError::NonFiniteInput);
    }

    let basis = &objects[..rank];
    let mut gram = vec![vec![0.0; rank]; rank];
    for row in 0..rank {
        for column in 0..rank {
            gram[row][column] = dot(&basis[row], &basis[column]);
        }
    }
    let ideal_matrix = objects
        .iter()
        .map(|object| {
            solve_normal_equation(
                &gram,
                &basis.iter().map(|row| dot(row, object)).collect::<Vec<_>>(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    let carriers = basis
        .iter()
        .map(|carrier| {
            carrier
                .iter()
                .map(|value| value / JOC_COARSE_QUANTUM)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let scaled_matrix = ideal_matrix
        .iter()
        .map(|row| row.iter().map(|value| value * JOC_COARSE_QUANTUM).collect())
        .collect::<Vec<Vec<f64>>>();
    let peak = basis
        .iter()
        .flatten()
        .map(|value| value.abs())
        .fold(0.0, f64::max);
    let scaled_peak = carriers
        .iter()
        .flatten()
        .map(|value| value.abs())
        .fold(0.0, f64::max);
    Ok(RankFactorization {
        carriers,
        ideal_matrix,
        scaled_matrix,
        scale_factor: JOC_COARSE_QUANTUM,
        carrier_peak_before_scaling: peak,
        carrier_peak_after_scaling: scaled_peak,
    })
}

/// Factorizes a reduced coded-object scene for a public profile carrier count.
///
/// When a scene has fewer coded objects than the selected profile has carrier
/// channels, the additional carriers are deterministic source-derived copies
/// and the corresponding matrix columns are zero. This keeps the public
/// topology intact without fabricating non-source audio or requiring a
/// mathematically impossible rank.
pub fn factorize_source_derived(
    objects: &[Vec<f64>],
    carrier_count: usize,
) -> Result<RankFactorization, AnalysisError> {
    if carrier_count == 0 {
        return Err(AnalysisError::InvalidRank {
            rank: carrier_count,
            object_count: objects.len(),
        });
    }
    validate_factorization_inputs(objects)?;
    let rank = carrier_count.min(objects.len());
    let analysis_objects = objects
        .iter()
        .map(|object| analysis_view(object))
        .collect::<Vec<_>>();
    let ridge = regularization_strength(&analysis_objects, rank);
    let mut factorization = factorize_regularized(&analysis_objects, rank, ridge)?;
    factorization.carriers = objects[..rank]
        .iter()
        .map(|carrier| {
            carrier
                .iter()
                .map(|value| value / JOC_COARSE_QUANTUM)
                .collect::<Vec<_>>()
        })
        .collect();
    let carrier_peak_before_scaling = factorization
        .carriers
        .iter()
        .flatten()
        .map(|value| value.abs())
        .fold(0.0, f64::max);
    let carrier_scale = if carrier_peak_before_scaling > SOURCE_DERIVED_CARRIER_PEAK_LIMIT {
        SOURCE_DERIVED_CARRIER_PEAK_LIMIT / carrier_peak_before_scaling
    } else {
        1.0
    };
    for carrier in &mut factorization.carriers {
        for value in carrier {
            *value *= carrier_scale;
        }
    }
    for row in &mut factorization.ideal_matrix {
        for value in row {
            *value /= carrier_scale;
        }
    }
    factorization.scaled_matrix = factorization
        .ideal_matrix
        .iter()
        .map(|row| {
            row.iter()
                .map(|value| value * JOC_COARSE_QUANTUM)
                .collect::<Vec<_>>()
        })
        .collect();
    factorization.carrier_peak_before_scaling = carrier_peak_before_scaling;
    while factorization.carriers.len() < carrier_count {
        let source_index = 0;
        factorization
            .carriers
            .push(factorization.carriers[source_index].clone());
        for row in &mut factorization.ideal_matrix {
            row.push(0.0);
        }
        for row in &mut factorization.scaled_matrix {
            row.push(0.0);
        }
    }
    factorization.carrier_peak_after_scaling = factorization
        .carriers
        .iter()
        .flatten()
        .map(|value| value.abs())
        .fold(0.0, f64::max);
    Ok(factorization)
}

/// Perturbs each intended source basis independently and measures its matching
/// carrier response. This is a deterministic validation fixture; it does not
/// participate in production encoding.
pub fn source_derived_carrier_causality(
    carrier_count: usize,
) -> Result<Vec<CarrierCausalityMeasurement>, AnalysisError> {
    if carrier_count == 0 || carrier_count > 7 {
        return Err(AnalysisError::InvalidRank {
            rank: carrier_count,
            object_count: 8,
        });
    }
    let objects = (0..8)
        .map(|object| {
            (0..128)
                .map(|sample| {
                    let x = f64::from(sample) + 1.0;
                    (x * (f64::from(object) + 1.0) * 0.031).sin()
                        + 0.25 * (x * (f64::from(object) + 2.0) * 0.017).cos()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let baseline = factorize_source_derived(&objects, carrier_count)?;
    let mut measurements = Vec::with_capacity(carrier_count);
    for source_index in 0..carrier_count {
        let mut perturbed_objects = objects.clone();
        for (sample, value) in perturbed_objects[source_index].iter_mut().enumerate() {
            *value += 0.125 * ((sample as f64 + 1.0) * 0.071).sin();
        }
        let perturbed = factorize_source_derived(&perturbed_objects, carrier_count)?;
        let maximum_change = baseline.carriers[source_index]
            .iter()
            .zip(&perturbed.carriers[source_index])
            .map(|(left, right)| (left - right).abs())
            .fold(0.0, f64::max);
        measurements.push(CarrierCausalityMeasurement {
            carrier_index: source_index,
            source_index,
            maximum_change,
            carrier_changed: maximum_change > 1.0e-12,
        });
    }
    Ok(measurements)
}

/// Measures normalized RMS residual on the same deterministic analysis view
/// used by source-derived factorization. This is a source-domain oracle
/// independent of metadata labels or an acceptance script's expected zero.
pub fn reconstruction_error(
    objects: &[Vec<f64>],
    factorization: &RankFactorization,
) -> Result<f64, AnalysisError> {
    validate_factorization_inputs(objects)?;
    let first = objects.first().ok_or(AnalysisError::EmptyObjectSet)?;
    let carrier_count = factorization.carriers.len();
    if carrier_count == 0
        || factorization
            .carriers
            .iter()
            .any(|carrier| carrier.len() != first.len())
        || factorization.ideal_matrix.len() != objects.len()
        || factorization.scaled_matrix.len() != objects.len()
        || factorization
            .scaled_matrix
            .iter()
            .any(|row| row.len() != carrier_count)
    {
        return Err(AnalysisError::DimensionMismatch);
    }
    let sample_indices = if first.len() <= SOURCE_DERIVED_ANALYSIS_SAMPLES {
        (0..first.len()).collect::<Vec<_>>()
    } else {
        (0..first.len())
            .step_by(first.len().div_ceil(SOURCE_DERIVED_ANALYSIS_SAMPLES))
            .take(SOURCE_DERIVED_ANALYSIS_SAMPLES)
            .collect::<Vec<_>>()
    };
    let mut total_energy = 0.0;
    let mut total_error_energy = 0.0;
    for (object, coefficients) in objects.iter().zip(&factorization.scaled_matrix) {
        for sample in sample_indices.iter().copied() {
            let reconstructed = factorization
                .carriers
                .iter()
                .zip(coefficients)
                .map(|(carrier, coefficient)| carrier[sample] * coefficient)
                .sum::<f64>();
            let error = object[sample] - reconstructed;
            total_energy += object[sample] * object[sample];
            total_error_energy += error * error;
        }
    }
    if total_energy <= SOURCE_DERIVED_ENERGY_FLOOR {
        return Ok(0.0);
    }
    Ok((total_error_energy / total_energy).sqrt())
}

/// Applies the clean carrier peak policy to one finite PCM signal.
pub fn limit_signal_peak(signal: &[f64], peak_limit: f64) -> Result<Vec<f64>, AnalysisError> {
    if signal.is_empty() {
        return Err(AnalysisError::EmptyObject);
    }
    if !peak_limit.is_finite() || !(0.0..=1.0).contains(&peak_limit) {
        return Err(AnalysisError::NonFiniteCoefficient);
    }
    if signal.iter().any(|value| !value.is_finite()) {
        return Err(AnalysisError::NonFiniteInput);
    }
    let peak = signal.iter().map(|value| value.abs()).fold(0.0, f64::max);
    let scale = if peak > peak_limit && peak > SOURCE_DERIVED_ENERGY_FLOOR {
        peak_limit / peak
    } else {
        1.0
    };
    Ok(signal.iter().map(|value| value * scale).collect())
}

/// Returns the largest finite element-wise delta between two rectangular
/// coefficient matrices or carrier collections.
pub fn maximum_absolute_difference(
    left: &[Vec<f64>],
    right: &[Vec<f64>],
) -> Result<f64, AnalysisError> {
    if left.len() != right.len() {
        return Err(AnalysisError::DimensionMismatch);
    }
    let mut maximum = 0.0_f64;
    for (left_row, right_row) in left.iter().zip(right) {
        if left_row.len() != right_row.len() {
            return Err(AnalysisError::DimensionMismatch);
        }
        for (left, right) in left_row.iter().zip(right_row) {
            if !left.is_finite() || !right.is_finite() {
                return Err(AnalysisError::NonFiniteInput);
            }
            maximum = maximum.max((left - right).abs());
        }
    }
    Ok(maximum)
}

fn validate_factorization_inputs(objects: &[Vec<f64>]) -> Result<(), AnalysisError> {
    let first = objects.first().ok_or(AnalysisError::EmptyObjectSet)?;
    if first.is_empty() {
        return Err(AnalysisError::EmptyObject);
    }
    if objects.iter().any(|object| object.len() != first.len()) {
        return Err(AnalysisError::DimensionMismatch);
    }
    if objects.iter().flatten().any(|value| !value.is_finite()) {
        return Err(AnalysisError::NonFiniteInput);
    }
    Ok(())
}

fn analysis_view(signal: &[f64]) -> Vec<f64> {
    if signal.len() <= SOURCE_DERIVED_ANALYSIS_SAMPLES {
        return signal.to_vec();
    }
    let step = signal.len().div_ceil(SOURCE_DERIVED_ANALYSIS_SAMPLES);
    signal
        .iter()
        .step_by(step)
        .take(SOURCE_DERIVED_ANALYSIS_SAMPLES)
        .copied()
        .collect()
}

fn regularization_strength(objects: &[Vec<f64>], rank: usize) -> f64 {
    let average_basis_energy = objects[..rank]
        .iter()
        .map(|object| dot(object, object))
        .sum::<f64>()
        / rank as f64;
    (average_basis_energy * SOURCE_DERIVED_RIDGE_RATIO).max(SOURCE_DERIVED_RIDGE)
}

fn factorize_regularized(
    objects: &[Vec<f64>],
    rank: usize,
    ridge: f64,
) -> Result<RankFactorization, AnalysisError> {
    let first = objects.first().ok_or(AnalysisError::EmptyObjectSet)?;
    if first.is_empty() {
        return Err(AnalysisError::EmptyObject);
    }
    if rank == 0 || rank > objects.len() {
        return Err(AnalysisError::InvalidRank {
            rank,
            object_count: objects.len(),
        });
    }
    if objects.iter().any(|object| object.len() != first.len()) {
        return Err(AnalysisError::DimensionMismatch);
    }
    if objects.iter().flatten().any(|value| !value.is_finite()) {
        return Err(AnalysisError::NonFiniteInput);
    }
    let basis = &objects[..rank];
    let mut gram = vec![vec![0.0; rank]; rank];
    for row in 0..rank {
        for column in 0..rank {
            gram[row][column] = dot(&basis[row], &basis[column]);
        }
        gram[row][row] += ridge;
    }
    let ideal_matrix = objects
        .iter()
        .map(|object| {
            solve_normal_equation(
                &gram,
                &basis.iter().map(|row| dot(row, object)).collect::<Vec<_>>(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let carriers = basis
        .iter()
        .map(|carrier| {
            carrier
                .iter()
                .map(|value| value / JOC_COARSE_QUANTUM)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let scaled_matrix = ideal_matrix
        .iter()
        .map(|row| {
            row.iter()
                .map(|value| value * JOC_COARSE_QUANTUM)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let carrier_peak_before_scaling = basis
        .iter()
        .flatten()
        .map(|value| value.abs())
        .fold(0.0, f64::max);
    let carrier_peak_after_scaling = carriers
        .iter()
        .flatten()
        .map(|value| value.abs())
        .fold(0.0, f64::max);
    Ok(RankFactorization {
        carriers,
        ideal_matrix,
        scaled_matrix,
        scale_factor: JOC_COARSE_QUANTUM,
        carrier_peak_before_scaling,
        carrier_peak_after_scaling,
    })
}

/// Quantizes one coefficient in the full 96-step JOC domain without clipping.
pub fn quantize_coefficient(ideal: f64) -> Result<QuantizedCoefficient, AnalysisError> {
    if !ideal.is_finite() {
        return Err(AnalysisError::NonFiniteCoefficient);
    }
    let step = (ideal / JOC_COARSE_QUANTUM).round();
    if !step.is_finite() {
        return Err(AnalysisError::NonFiniteCoefficient);
    }
    if !(-48.0..=47.0).contains(&step) {
        return Err(AnalysisError::CoefficientOutOfRange { step });
    }
    let symbol = (step as i32).rem_euclid(96) as u16;
    let decoded = f64::from(step as i32) * JOC_COARSE_QUANTUM;
    Ok(QuantizedCoefficient {
        ideal,
        encoded_symbol: symbol,
        decoded,
        error: decoded - ideal,
    })
}

fn dot(left: &[f64], right: &[f64]) -> f64 {
    left.iter().zip(right).map(|(a, b)| a * b).sum()
}

fn solve_normal_equation(matrix: &[Vec<f64>], right: &[f64]) -> Result<Vec<f64>, AnalysisError> {
    let size = right.len();
    let mut augmented = matrix
        .iter()
        .zip(right)
        .map(|(row, value)| {
            let mut row = row.clone();
            row.push(*value);
            row
        })
        .collect::<Vec<_>>();
    for pivot in 0..size {
        let pivot_row = (pivot..size)
            .max_by(|&left, &right| {
                augmented[left][pivot]
                    .abs()
                    .total_cmp(&augmented[right][pivot].abs())
            })
            .ok_or(AnalysisError::SingularBasis)?;
        if augmented[pivot_row][pivot].abs() < PIVOT_EPSILON {
            return Err(AnalysisError::SingularBasis);
        }
        augmented.swap(pivot, pivot_row);
        let divisor = augmented[pivot][pivot];
        for value in augmented[pivot].iter_mut().take(size + 1).skip(pivot) {
            *value /= divisor;
        }
        let pivot_values = augmented[pivot].clone();
        for (row_index, row) in augmented.iter_mut().enumerate().take(size) {
            if row_index == pivot {
                continue;
            }
            let factor = row[pivot];
            for (value, pivot_value) in row
                .iter_mut()
                .skip(pivot)
                .zip(pivot_values.iter().skip(pivot))
            {
                *value -= factor * pivot_value;
            }
        }
    }
    let result = augmented
        .into_iter()
        .map(|row| row[size])
        .collect::<Vec<_>>();
    if result.iter().any(|value| !value.is_finite()) {
        return Err(AnalysisError::NonFiniteCoefficient);
    }
    Ok(result)
}
