// pattern: Functional Core

use std::fmt;

const JOC_COARSE_QUANTUM: f64 = 820.0 / 4096.0;

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

/// Quantization evidence for one full-matrix coefficient.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuantizedCoefficient {
    pub ideal: f64,
    pub encoded_symbol: u16,
    pub decoded: f64,
    pub error: f64,
}

/// Errors from the bounded rank-controlled factorization path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnalysisError {
    EmptyObjectSet,
    EmptyObject,
    DimensionMismatch,
    InvalidRank { rank: usize, object_count: usize },
    SingularBasis,
    NonFiniteInput,
    NonFiniteCoefficient,
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

/// Quantizes one coefficient in the full 96-step JOC domain without clipping.
pub fn quantize_coefficient(ideal: f64) -> Result<QuantizedCoefficient, AnalysisError> {
    if !ideal.is_finite() {
        return Err(AnalysisError::NonFiniteCoefficient);
    }
    let step = (ideal / JOC_COARSE_QUANTUM).round();
    if !step.is_finite() || !(-48.0..=47.0).contains(&step) {
        return Err(AnalysisError::NonFiniteCoefficient);
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
        if augmented[pivot_row][pivot].abs() < 1.0e-12 {
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
