// pattern: Functional Core
//! Stateless selected F03N 15-by-n Lawson-Hanson initial NNLS solve.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NnlsError {
    InvalidInput,
    NumericFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NnlsStatus {
    Solved,
    IterationLimit,
}

impl NnlsStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Solved => "SOLVED",
            Self::IterationLimit => "ITERATION_LIMIT",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NnlsRejectionReason {
    RelativeRank,
    NonpositiveTrial,
}

impl NnlsRejectionReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RelativeRank => "RELATIVE_RANK",
            Self::NonpositiveTrial => "NONPOSITIVE_TRIAL",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NnlsRejectedCandidate {
    pub variable: usize,
    pub reason: NnlsRejectionReason,
    pub passive_count: usize,
    pub rank_increment: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NnlsRemoval {
    pub variable: usize,
    pub passive_position: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NnlsResult {
    pub seed_u32: Vec<u32>,
    pub solution_f64: Vec<f64>,
    pub status: NnlsStatus,
    pub passive: Vec<usize>,
    pub inactive: Vec<usize>,
    pub permutation: Vec<usize>,
    pub dual: Vec<f64>,
    pub residual_norm: f64,
    pub accepted: Vec<usize>,
    pub rejected: Vec<NnlsRejectedCandidate>,
    pub removals: Vec<NnlsRemoval>,
    pub iterations: usize,
    pub iteration_limit: usize,
}

const OBSERVATION_COUNT: usize = 15;
const R32_ONE_PERCENT_BITS: u32 = 0x3c23_d70a;
const POSITIVE_DUAL_STOP: f64 = 1.0e-12;

/// Solves `min ||A x - b||²`, `x >= 0`, for one 15-by-n F32 input.
///
/// Input words are already laid out as fifteen rows of `n` F32 bit patterns.
/// The solve starts cold on every call; it retains no warm seed or other state.
pub fn solve_rectangular_nnls(
    context_u32: &[Vec<u32>],
    response_u32: &[u32],
) -> Result<NnlsResult, NnlsError> {
    if context_u32.len() != OBSERVATION_COUNT || response_u32.len() != OBSERVATION_COUNT {
        return Err(NnlsError::InvalidInput);
    }
    let column_count = context_u32.first().ok_or(NnlsError::InvalidInput)?.len();
    if !(1..=OBSERVATION_COUNT).contains(&column_count)
        || context_u32.iter().any(|row| {
            row.len() != column_count || row.iter().any(|word| !f32::from_bits(*word).is_finite())
        })
        || response_u32
            .iter()
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(NnlsError::InvalidInput);
    }

    let mut columns = (0..column_count)
        .map(|column| {
            context_u32
                .iter()
                .map(|row| f64::from(f32::from_bits(row[column])))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut rhs = response_u32
        .iter()
        .map(|word| f64::from(f32::from_bits(*word)))
        .collect::<Vec<_>>();
    let mut solution = vec![0.0; column_count];
    let mut dual = vec![0.0; column_count];
    let mut permutation = (0..column_count).collect::<Vec<_>>();
    let mut passive_count = 0;
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    let mut removals = Vec::new();
    let mut secondary_iterations = 0;
    let iteration_limit = 3 * column_count;
    let rank_factor = f64::from(f32::from_bits(R32_ONE_PERCENT_BITS));
    let mut status = NnlsStatus::Solved;

    'active_set: while passive_count < column_count && passive_count < OBSERVATION_COUNT {
        for variable in permutation.iter().copied().skip(passive_count) {
            let mut value = 0.0;
            for row in passive_count..OBSERVATION_COUNT {
                value = add(value, multiply(columns[variable][row], rhs[row])?)?;
            }
            dual[variable] = value;
        }

        loop {
            let Some((candidate_position, candidate)) =
                highest_positive_inactive(&permutation, passive_count, &dual)
            else {
                break 'active_set;
            };
            if dual[candidate] <= POSITIVE_DUAL_STOP {
                break 'active_set;
            }

            let pivot = passive_count;
            let old_diagonal = columns[candidate][pivot];
            let (reflector_u, diagonal) = construct_householder(&mut columns[candidate], pivot)?;
            let unorm = norm_prefix(&columns[candidate], pivot)?;
            let rank_increment =
                subtract(add(unorm, multiply(diagonal.abs(), rank_factor)?)?, unorm)?;

            let rejection_reason = if rank_increment <= 0.0 {
                Some(NnlsRejectionReason::RelativeRank)
            } else {
                let mut trial_rhs = rhs.clone();
                apply_householder(&mut trial_rhs, &columns[candidate], reflector_u, pivot)?;
                if diagonal == 0.0 {
                    return Err(NnlsError::NumericFailure);
                }
                if divide(trial_rhs[pivot], diagonal)? <= 0.0 {
                    Some(NnlsRejectionReason::NonpositiveTrial)
                } else {
                    rhs = trial_rhs;
                    permutation.swap(candidate_position, passive_count);
                    let accepted_variable = permutation[passive_count];
                    accepted.push(accepted_variable);
                    passive_count += 1;
                    dual[accepted_variable] = 0.0;

                    let reflector_column = columns[accepted_variable].clone();
                    for variable in permutation.iter().copied().skip(passive_count) {
                        apply_householder(
                            &mut columns[variable],
                            &reflector_column,
                            reflector_u,
                            pivot,
                        )?;
                    }
                    columns[accepted_variable][passive_count..OBSERVATION_COUNT].fill(0.0);

                    let mut secondary_finished = false;
                    while !secondary_finished {
                        secondary_iterations += 1;
                        if secondary_iterations > iteration_limit {
                            status = NnlsStatus::IterationLimit;
                            break 'active_set;
                        }
                        let trial =
                            triangular_solution(&columns, &rhs, &permutation, passive_count)?;
                        if trial[..passive_count].iter().all(|value| *value > 0.0) {
                            for (position, value) in trial.iter().take(passive_count).enumerate() {
                                solution[permutation[position]] = *value;
                            }
                            secondary_finished = true;
                        } else {
                            let (remove_position, alpha) = minimum_feasible_step(
                                &trial,
                                &solution,
                                &permutation,
                                passive_count,
                            )?;
                            update_solution_to_boundary(
                                &trial,
                                &mut solution,
                                &permutation,
                                passive_count,
                                alpha,
                            )?;
                            let removed = permutation[remove_position];
                            solution[removed] = 0.0;
                            remove_passive(
                                &mut columns,
                                &mut rhs,
                                &mut permutation,
                                &mut passive_count,
                                remove_position,
                                &mut solution,
                                &mut removals,
                            )?;

                            while passive_count > 0
                                && permutation[..passive_count]
                                    .iter()
                                    .any(|variable| solution[*variable] <= 0.0)
                            {
                                let last_position = passive_count - 1;
                                let last_variable = permutation[last_position];
                                solution[last_variable] = 0.0;
                                remove_passive(
                                    &mut columns,
                                    &mut rhs,
                                    &mut permutation,
                                    &mut passive_count,
                                    last_position,
                                    &mut solution,
                                    &mut removals,
                                )?;
                            }
                        }
                    }
                    None
                }
            };

            if let Some(reason) = rejection_reason {
                columns[candidate][pivot] = old_diagonal;
                dual[candidate] = 0.0;
                rejected.push(NnlsRejectedCandidate {
                    variable: candidate,
                    reason,
                    passive_count,
                    rank_increment,
                });
                continue;
            }
            break;
        }
    }

    if passive_count >= OBSERVATION_COUNT {
        dual.fill(0.0);
    }
    let mut residual_square = 0.0;
    for value in &rhs[passive_count..] {
        residual_square = add(residual_square, multiply(*value, *value)?)?;
    }
    if residual_square < 0.0 {
        return Err(NnlsError::NumericFailure);
    }
    let residual_norm = residual_square.sqrt();
    if !residual_norm.is_finite() {
        return Err(NnlsError::NumericFailure);
    }
    let seed_u32 = solution
        .iter()
        .map(|value| f64_to_f32_word(*value))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(NnlsResult {
        seed_u32,
        solution_f64: solution,
        status,
        passive: permutation[..passive_count].to_vec(),
        inactive: permutation[passive_count..].to_vec(),
        permutation,
        dual,
        residual_norm,
        accepted,
        rejected,
        removals,
        iterations: secondary_iterations,
        iteration_limit,
    })
}

fn highest_positive_inactive(
    permutation: &[usize],
    passive_count: usize,
    dual: &[f64],
) -> Option<(usize, usize)> {
    let mut best_value = 0.0;
    let mut best = None;
    for (position, variable) in permutation.iter().copied().enumerate().skip(passive_count) {
        if dual[variable] > best_value {
            best_value = dual[variable];
            best = Some((position, variable));
        }
    }
    best
}

fn construct_householder(column: &mut [f64], pivot: usize) -> Result<(f64, f64), NnlsError> {
    if pivot + 1 >= OBSERVATION_COUNT {
        return Ok((0.0, column[pivot]));
    }
    let diagonal = column[pivot];
    let scale = column[pivot..]
        .iter()
        .map(|value| value.abs())
        .fold(0.0, f64::max);
    if scale <= 0.0 {
        return Ok((0.0, diagonal));
    }
    let inverse = divide(1.0, scale)?;
    let mut norm_square = 0.0;
    for value in &column[pivot..] {
        let normalized = multiply(*value, inverse)?;
        norm_square = add(norm_square, multiply(normalized, normalized)?)?;
    }
    let norm = norm_square.sqrt();
    if !norm.is_finite() {
        return Err(NnlsError::NumericFailure);
    }
    let mut d = multiply(scale, norm)?;
    if diagonal > 0.0 {
        d = -d;
    }
    let u = subtract(diagonal, d)?;
    column[pivot] = d;
    Ok((u, d))
}

fn apply_householder(
    vector: &mut [f64],
    reflector_column: &[f64],
    reflector_u: f64,
    pivot: usize,
) -> Result<(), NnlsError> {
    if pivot + 1 >= OBSERVATION_COUNT || reflector_column[pivot].abs() <= 0.0 || reflector_u == 0.0
    {
        return Ok(());
    }
    let denominator = multiply(reflector_column[pivot], reflector_u)?;
    if denominator >= 0.0 {
        return Ok(());
    }
    let mut t = multiply(reflector_u, vector[pivot])?;
    for row in pivot + 1..OBSERVATION_COUNT {
        t = add(t, multiply(vector[row], reflector_column[row])?)?;
    }
    if t == 0.0 {
        return Ok(());
    }
    let reciprocal = divide(1.0, denominator)?;
    let step = multiply(t, reciprocal)?;
    let old_pivot = vector[pivot];
    vector[pivot] = add(multiply(reflector_u, step)?, old_pivot)?;
    for row in pivot + 1..OBSERVATION_COUNT {
        let old = vector[row];
        vector[row] = add(multiply(step, reflector_column[row])?, old)?;
    }
    Ok(())
}

fn norm_prefix(column: &[f64], end: usize) -> Result<f64, NnlsError> {
    let mut sum = 0.0;
    for value in column.iter().take(end) {
        sum = add(sum, multiply(*value, *value)?)?;
    }
    let norm = sum.sqrt();
    if !norm.is_finite() {
        return Err(NnlsError::NumericFailure);
    }
    Ok(norm)
}

fn triangular_solution(
    columns: &[Vec<f64>],
    rhs: &[f64],
    permutation: &[usize],
    passive_count: usize,
) -> Result<Vec<f64>, NnlsError> {
    let mut work = rhs.to_vec();
    let mut solution = vec![0.0; columns.len()];
    for pivot in (0..passive_count).rev() {
        let variable = permutation[pivot];
        let diagonal = columns[variable][pivot];
        if diagonal == 0.0 {
            return Err(NnlsError::NumericFailure);
        }
        solution[pivot] = divide(work[pivot], diagonal)?;
        for row in 0..pivot {
            work[row] = subtract(
                work[row],
                multiply(solution[pivot], columns[variable][row])?,
            )?;
        }
    }
    Ok(solution)
}

fn minimum_feasible_step(
    trial: &[f64],
    current: &[f64],
    permutation: &[usize],
    passive_count: usize,
) -> Result<(usize, f64), NnlsError> {
    let mut best_position = None;
    let mut best_alpha = f64::INFINITY;
    for position in 0..passive_count {
        let variable = permutation[position];
        if trial[position] <= 0.0 {
            let denominator = subtract(trial[position], current[variable])?;
            if denominator == 0.0 {
                return Err(NnlsError::NumericFailure);
            }
            let alpha = divide(-current[variable], denominator)?;
            if alpha < best_alpha {
                best_alpha = alpha;
                best_position = Some(position);
            }
        }
    }
    best_position
        .map(|position| (position, best_alpha))
        .ok_or(NnlsError::NumericFailure)
}

fn update_solution_to_boundary(
    trial: &[f64],
    solution: &mut [f64],
    permutation: &[usize],
    passive_count: usize,
    alpha: f64,
) -> Result<(), NnlsError> {
    for position in 0..passive_count {
        let variable = permutation[position];
        let delta = subtract(trial[position], solution[variable])?;
        solution[variable] = add(multiply(delta, alpha)?, solution[variable])?;
    }
    Ok(())
}

fn remove_passive(
    columns: &mut [Vec<f64>],
    rhs: &mut [f64],
    permutation: &mut [usize],
    passive_count: &mut usize,
    remove_position: usize,
    solution: &mut [f64],
    removals: &mut Vec<NnlsRemoval>,
) -> Result<(), NnlsError> {
    let old_count = *passive_count;
    let removed = permutation[remove_position];
    for position in remove_position + 1..old_count {
        let moved = permutation[position];
        permutation[position - 1] = moved;
        let top_row = position - 1;
        let bottom_row = position;
        let (cosine, sine, sigma) = givens(columns[moved][top_row], columns[moved][bottom_row])?;
        columns[moved][top_row] = sigma;
        columns[moved][bottom_row] = 0.0;

        for (variable, column) in columns.iter_mut().enumerate() {
            if variable == moved {
                continue;
            }
            rotate_column(column, top_row, bottom_row, cosine, sine)?;
        }
        rotate_rhs(rhs, top_row, bottom_row, cosine, sine)?;
    }
    *passive_count = old_count - 1;
    permutation[*passive_count] = removed;
    solution[removed] = 0.0;
    removals.push(NnlsRemoval {
        variable: removed,
        passive_position: remove_position,
    });
    Ok(())
}

fn givens(a: f64, b: f64) -> Result<(f64, f64, f64), NnlsError> {
    if a.abs() > b.abs() {
        let ratio = divide(b, a)?;
        let h = add(multiply(ratio, ratio)?, 1.0)?.sqrt();
        let mut cosine = divide(1.0, h)?.abs();
        if a < 0.0 {
            cosine = -cosine;
        }
        let sine = multiply(cosine, ratio)?;
        let sigma = multiply(a.abs(), h)?;
        Ok((cosine, sine, sigma))
    } else if b != 0.0 {
        let ratio = divide(a, b)?;
        let h = add(multiply(ratio, ratio)?, 1.0)?.sqrt();
        let mut sine = divide(1.0, h)?.abs();
        if b < 0.0 {
            sine = -sine;
        }
        let cosine = multiply(sine, ratio)?;
        let sigma = multiply(b.abs(), h)?;
        Ok((cosine, sine, sigma))
    } else {
        Ok((0.0, 1.0, 0.0))
    }
}

fn rotate_column(
    column: &mut [f64],
    top_row: usize,
    bottom_row: usize,
    cosine: f64,
    sine: f64,
) -> Result<(), NnlsError> {
    let upper = column[top_row];
    let lower = column[bottom_row];
    column[top_row] = add(multiply(lower, sine)?, multiply(upper, cosine)?)?;
    column[bottom_row] = subtract(multiply(lower, cosine)?, multiply(upper, sine)?)?;
    Ok(())
}

fn rotate_rhs(
    rhs: &mut [f64],
    top_row: usize,
    bottom_row: usize,
    cosine: f64,
    sine: f64,
) -> Result<(), NnlsError> {
    let upper = rhs[top_row];
    let lower = rhs[bottom_row];
    rhs[top_row] = add(multiply(upper, cosine)?, multiply(sine, lower)?)?;
    rhs[bottom_row] = subtract(multiply(cosine, lower)?, multiply(upper, sine)?)?;
    Ok(())
}

fn add(left: f64, right: f64) -> Result<f64, NnlsError> {
    finite(left + right)
}

fn subtract(left: f64, right: f64) -> Result<f64, NnlsError> {
    finite(left - right)
}

fn multiply(left: f64, right: f64) -> Result<f64, NnlsError> {
    finite(left * right)
}

fn divide(left: f64, right: f64) -> Result<f64, NnlsError> {
    if right == 0.0 {
        return Err(NnlsError::NumericFailure);
    }
    finite(left / right)
}

fn finite(value: f64) -> Result<f64, NnlsError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(NnlsError::NumericFailure)
    }
}

fn f64_to_f32_word(value: f64) -> Result<u32, NnlsError> {
    if !value.is_finite() {
        return Err(NnlsError::NumericFailure);
    }
    let source = value.to_bits();
    let sign = u32::try_from(source >> 32).map_err(|_| NnlsError::NumericFailure)? & 0x8000_0000;
    let exponent_bits =
        i32::try_from((source >> 52) & 0x7ff).map_err(|_| NnlsError::NumericFailure)?;
    let fraction = source & ((1_u64 << 52) - 1);
    if exponent_bits == 0 {
        return Ok(sign);
    }
    let mut exponent = exponent_bits - 1023;
    let significand = (1_u64 << 52) | fraction;
    if exponent < -126 {
        let shift = u32::try_from(-(exponent + 97)).map_err(|_| NnlsError::NumericFailure)?;
        let rounded = round_shift_right_even(significand, shift);
        if rounded == 0 {
            return Ok(sign);
        }
        let rounded = u32::try_from(rounded).map_err(|_| NnlsError::NumericFailure)?;
        if rounded >= (1 << 23) {
            return Ok(sign | (1 << 23));
        }
        return Ok(sign | rounded);
    }

    let mut rounded = round_shift_right_even(significand, 29);
    if rounded == (1 << 24) {
        rounded >>= 1;
        exponent += 1;
    }
    if exponent > 127 {
        return Err(NnlsError::NumericFailure);
    }
    let rounded = u32::try_from(rounded).map_err(|_| NnlsError::NumericFailure)?;
    let fraction_bits = rounded
        .checked_sub(1 << 23)
        .filter(|value| *value < (1 << 23))
        .ok_or(NnlsError::NumericFailure)?;
    let exponent_bits = u32::try_from(exponent + 127).map_err(|_| NnlsError::NumericFailure)?;
    Ok(sign | (exponent_bits << 23) | fraction_bits)
}

fn round_shift_right_even(value: u64, shift: u32) -> u64 {
    if shift == 0 {
        return value;
    }
    if shift > 64 {
        return 0;
    }
    let (quotient, remainder) = if shift == 64 {
        (0, value)
    } else {
        (value >> shift, value & ((1_u64 << shift) - 1))
    };
    let halfway = 1_u64 << (shift - 1);
    if remainder > halfway || (remainder == halfway && quotient % 2 == 1) {
        quotient + 1
    } else {
        quotient
    }
}

#[cfg(test)]
mod tests;
