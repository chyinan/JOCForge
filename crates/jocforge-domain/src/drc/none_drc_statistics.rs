// pattern: Functional Core
//! Selected F06ND per-frame filtering, energy, peak and state transition.

use crate::drc::none_drc_dialnorm_offset::{
    NoneDrcDialnormOffsetError, none_drc_dialnorm_offset_word,
};
use crate::drc::none_drc_normalizer::{NoneDrcNormalizationError, normalize_none_drc_metric};
use crate::numeric::{FiniteF32, R32Error, r32_add, r32_mul, r32_sub};

const CHANNEL_COUNT: usize = 6;
const FULLBAND_CHANNEL_COUNT: usize = 5;
const SAMPLE_COUNT: usize = 1_536;
const BLOCK_COUNT: usize = 6;
const SAMPLES_PER_BLOCK: usize = 256;
const PAIRS_PER_BLOCK: usize = SAMPLES_PER_BLOCK / 2;
const INTERNAL_TO_EXTERNAL_CHANNEL: [usize; CHANNEL_COUNT] = [0, 2, 1, 4, 5, 3];

const FILTER_A0: f32 = f32::from_bits(0x3F79_999A);
const FILTER_A1: f32 = f32::from_bits(0x0000_0000);
const FILTER_B0: f32 = f32::from_bits(0x3F13_33DB);
const FILTER_B1: f32 = f32::from_bits(0xBDEB_862B);
const FILTER_B2: f32 = f32::from_bits(0xBEEB_862B);

const AUXILIARY_N: f32 = f32::from_bits(0xBEB5_04F3);
const AUXILIARY_V: f32 = f32::from_bits(0x3F80_0000);
const AUXILIARY_S: f32 = f32::from_bits(0xBF00_0000);
const ROTATION_P0: f32 = f32::from_bits(0x3F5E_F778);
const ROTATION_P1: f32 = f32::from_bits(0x3EFA_C44A);

const POSITIVE_ZERO: f32 = f32::from_bits(0x0000_0000);
const ONE: f32 = 1.0;
const TWO: f32 = 2.0;
const ONE_OVER_32: f32 = f32::from_bits(0x3D00_0000);
const ONE_OVER_128: f32 = f32::from_bits(0x3C00_0000);
const TERMINAL_PEAK_SCALE: f32 = ONE;

pub type FiniteStatisticsPlanes = [[FiniteF32; SAMPLE_COUNT]; CHANNEL_COUNT];

/// Inputs for one six-block F06ND statistics frame.
pub struct NoneDrcStatisticsFrameInput<'a> {
    /// Planes are in external L, R, C, LFE, Ls, Rs order.
    pub primary_samples: &'a FiniteStatisticsPlanes,
    /// Planes use the same external order as `primary_samples`.
    pub secondary_samples: &'a FiniteStatisticsPlanes,
    pub dialnorm: i32,
}

/// Committed per-programme state used only by this statistics transition.
#[derive(Debug, PartialEq)]
pub struct NoneDrcStatisticsState {
    filter_pairs: [[f32; 2]; FULLBAND_CHANNEL_COUNT],
    previous_primary_peak: [f32; CHANNEL_COUNT],
    previous_secondary_peak: [f32; CHANNEL_COUNT],
    previous_downmix_peak: f32,
}

impl NoneDrcStatisticsState {
    pub const fn cold() -> Self {
        Self {
            filter_pairs: [[POSITIVE_ZERO; 2]; FULLBAND_CHANNEL_COUNT],
            previous_primary_peak: [POSITIVE_ZERO; CHANNEL_COUNT],
            previous_secondary_peak: [POSITIVE_ZERO; CHANNEL_COUNT],
            previous_downmix_peak: POSITIVE_ZERO,
        }
    }

    pub const fn filter_pairs(&self) -> &[[f32; 2]; FULLBAND_CHANNEL_COUNT] {
        &self.filter_pairs
    }

    pub const fn previous_primary_peak(&self) -> &[f32; CHANNEL_COUNT] {
        &self.previous_primary_peak
    }

    pub const fn previous_secondary_peak(&self) -> &[f32; CHANNEL_COUNT] {
        &self.previous_secondary_peak
    }

    pub const fn previous_downmix_peak(&self) -> f32 {
        self.previous_downmix_peak
    }

    fn staged_next_state(&self) -> Self {
        Self {
            filter_pairs: self.filter_pairs,
            previous_primary_peak: self.previous_primary_peak,
            previous_secondary_peak: self.previous_secondary_peak,
            previous_downmix_peak: self.previous_downmix_peak,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoneDrcEnergyBlock {
    /// Mean squares in internal L, C, R, Ls, Rs order.
    pub mean_square_five: [f32; FULLBAND_CHANNEL_COUNT],
    pub summed_power: f32,
    pub normalized_level: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoneDrcStatisticsFrameOutput {
    pub rms_levels_six: [f32; BLOCK_COUNT],
    pub peak_inputs_six: [f32; BLOCK_COUNT],
    pub energy_blocks: [NoneDrcEnergyBlock; BLOCK_COUNT],
    /// Auxiliary peak after the block's previous-downmix carry is applied.
    pub auxiliary_raw_peaks_six: [f32; BLOCK_COUNT],
}

#[derive(Debug, PartialEq)]
pub struct NoneDrcStatisticsFrameTransition {
    pub output: NoneDrcStatisticsFrameOutput,
    pub next_state: NoneDrcStatisticsState,
}

/// Numeric or selected dialnorm failure propagated without assigning a new
/// F06ND error code. The numeric I03 outcome mapping remains outside this leaf.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoneDrcStatisticsError {
    Numeric(R32Error),
    Normalization(NoneDrcNormalizationError),
    Dialnorm(NoneDrcDialnormOffsetError),
}

impl From<R32Error> for NoneDrcStatisticsError {
    fn from(error: R32Error) -> Self {
        Self::Numeric(error)
    }
}

impl From<NoneDrcNormalizationError> for NoneDrcStatisticsError {
    fn from(error: NoneDrcNormalizationError) -> Self {
        Self::Normalization(error)
    }
}

impl From<NoneDrcDialnormOffsetError> for NoneDrcStatisticsError {
    fn from(error: NoneDrcDialnormOffsetError) -> Self {
        Self::Dialnorm(error)
    }
}

/// Advances the selected F06ND statistics state by one six-block frame.
///
/// Primary and secondary input planes are validated finite F32 values. The
/// returned state is an actual transition result and is the only valid seed
/// for the following frame in a continuous programme.
pub fn transition_none_drc_statistics_frame(
    state: &NoneDrcStatisticsState,
    input: &NoneDrcStatisticsFrameInput<'_>,
) -> Result<NoneDrcStatisticsFrameTransition, NoneDrcStatisticsError> {
    let dialnorm_offset = f32::from_bits(none_drc_dialnorm_offset_word(input.dialnorm)?);
    let filter_vectors = paired_filter_vectors()?;
    let auxiliary_coefficients = auxiliary_mix_coefficients()?;
    let mut next_state = state.staged_next_state();
    let mut rms_levels_six = [POSITIVE_ZERO; BLOCK_COUNT];
    let mut peak_inputs_six = [POSITIVE_ZERO; BLOCK_COUNT];
    let mut auxiliary_raw_peaks_six = [POSITIVE_ZERO; BLOCK_COUNT];
    let mut energy_blocks = [NoneDrcEnergyBlock {
        mean_square_five: [POSITIVE_ZERO; FULLBAND_CHANNEL_COUNT],
        summed_power: POSITIVE_ZERO,
        normalized_level: ONE,
    }; BLOCK_COUNT];

    for block in 0..BLOCK_COUNT {
        let sample_start = block * SAMPLES_PER_BLOCK;
        let sample_end = sample_start + SAMPLES_PER_BLOCK;
        let mut mean_square_five = [POSITIVE_ZERO; FULLBAND_CHANNEL_COUNT];

        for internal_channel in 0..FULLBAND_CHANNEL_COUNT {
            let external_channel = INTERNAL_TO_EXTERNAL_CHANNEL[internal_channel];
            let primary = &input.primary_samples[external_channel][sample_start..sample_end];
            let mut filtered = [POSITIVE_ZERO; SAMPLES_PER_BLOCK];
            apply_paired_filter(
                &mut next_state.filter_pairs[internal_channel],
                primary,
                &filter_vectors,
                &mut filtered,
            )?;
            mean_square_five[internal_channel] = mean_square(&filtered)?;
        }

        let summed_power = sum_channel_energy(&mean_square_five)?;
        let bounded_power = clamp(summed_power);
        let normalized = normalize_none_drc_metric(bounded_power)?;
        let normalized_level = clamp(r32_add_values(normalized, dialnorm_offset)?);
        rms_levels_six[block] = normalized_level;
        energy_blocks[block] = NoneDrcEnergyBlock {
            mean_square_five,
            summed_power,
            normalized_level,
        };

        let mut combined_peaks = [POSITIVE_ZERO; CHANNEL_COUNT];
        for internal_channel in 0..CHANNEL_COUNT {
            let external_channel = INTERNAL_TO_EXTERNAL_CHANNEL[internal_channel];
            let primary = &input.primary_samples[external_channel][sample_start..sample_end];
            let current = peak_scale(current_peak(primary))?;
            combined_peaks[internal_channel] =
                next_state.previous_primary_peak[internal_channel].max(current);
            next_state.previous_primary_peak[internal_channel] = current;
        }

        if block < BLOCK_COUNT - 1 {
            for internal_channel in 0..FULLBAND_CHANNEL_COUNT {
                let external_channel = INTERNAL_TO_EXTERNAL_CHANNEL[internal_channel];
                let secondary =
                    &input.secondary_samples[external_channel][sample_start..sample_end];
                let current = peak_scale(current_peak(secondary))?;
                let combined_secondary =
                    next_state.previous_secondary_peak[internal_channel].max(current);
                next_state.previous_secondary_peak[internal_channel] = current;
                combined_peaks[internal_channel] =
                    combined_peaks[internal_channel].max(combined_secondary);
            }
        }

        let channel_peak = r32_mul_values(
            combined_peaks.into_iter().fold(POSITIVE_ZERO, f32::max),
            ONE_OVER_128,
        )?;
        let auxiliary_source = if block < BLOCK_COUNT - 1 {
            input.secondary_samples
        } else {
            input.primary_samples
        };
        let auxiliary_current =
            auxiliary_current_peak(auxiliary_source, sample_start, &auxiliary_coefficients)?;
        let (auxiliary_peak, next_downmix_peak) = if block < BLOCK_COUNT - 1 {
            (
                next_state.previous_downmix_peak.max(auxiliary_current),
                auxiliary_current,
            )
        } else {
            let scaled = r32_mul_values(TERMINAL_PEAK_SCALE, auxiliary_current)?;
            (next_state.previous_downmix_peak.max(scaled), scaled)
        };
        next_state.previous_downmix_peak = next_downmix_peak;
        auxiliary_raw_peaks_six[block] = auxiliary_peak;
        peak_inputs_six[block] = channel_peak.max(auxiliary_peak);
    }

    Ok(NoneDrcStatisticsFrameTransition {
        output: NoneDrcStatisticsFrameOutput {
            rms_levels_six,
            peak_inputs_six,
            energy_blocks,
            auxiliary_raw_peaks_six,
        },
        next_state,
    })
}

#[derive(Clone, Copy)]
struct PairedFilterVectors {
    q25: [f32; 4],
    q26: [f32; 4],
    q27: [f32; 4],
    q28: [f32; 4],
}

fn paired_filter_vectors() -> Result<PairedFilterVectors, R32Error> {
    let q1 = r32_add_values(r32_mul_values(FILTER_B0, FILTER_A0)?, FILTER_B1)?;
    let q2 = r32_add_values(r32_mul_values(FILTER_B0, FILTER_A1)?, FILTER_B2)?;
    let a0_squared = r32_mul_values(FILTER_A0, FILTER_A0)?;
    let q25 = [
        ONE,
        FILTER_A0,
        r32_add_values(a0_squared, FILTER_A1)?,
        r32_mul_values(FILTER_A1, FILTER_A0)?,
    ];
    let q26 = [POSITIVE_ZERO, ONE, FILTER_A0, FILTER_A1];
    let q27 = [
        FILTER_B0,
        q1,
        r32_add_values(r32_mul_values(q1, FILTER_A0)?, q2)?,
        r32_mul_values(q1, FILTER_A1)?,
    ];
    let q28 = [POSITIVE_ZERO, FILTER_B0, q1, q2];
    Ok(PairedFilterVectors { q25, q26, q27, q28 })
}

fn apply_paired_filter(
    state: &mut [f32; 2],
    input: &[FiniteF32],
    vectors: &PairedFilterVectors,
    output: &mut [f32; SAMPLES_PER_BLOCK],
) -> Result<(), R32Error> {
    for pair in 0..PAIRS_PER_BLOCK {
        let x0 = input[pair * 2].value();
        let x1 = input[pair * 2 + 1].value();
        let mut lane_values = [POSITIVE_ZERO; 4];

        for (lane, lane_value) in lane_values.iter_mut().enumerate() {
            let x1_term = r32_mul_values(x1, vectors.q28[lane])?;
            let x0_term = r32_mul_values(x0, vectors.q27[lane])?;
            let paired_inputs = r32_add_values(x1_term, x0_term)?;
            let prior_first = r32_mul_values(state[0], vectors.q25[lane])?;
            let with_prior_first = r32_add_values(paired_inputs, prior_first)?;
            let prior_second = r32_mul_values(state[1], vectors.q26[lane])?;
            *lane_value = r32_add_values(with_prior_first, prior_second)?;
        }

        output[pair * 2] = clamp(lane_values[0]);
        output[pair * 2 + 1] = clamp(lane_values[1]);
        *state = [lane_values[2], lane_values[3]];
    }
    Ok(())
}

fn mean_square(samples: &[f32; SAMPLES_PER_BLOCK]) -> Result<f32, R32Error> {
    let mut accumulators = [POSITIVE_ZERO; 4];
    for base in (0..SAMPLES_PER_BLOCK).step_by(4) {
        for lane in 0..4 {
            let value = samples[base + lane];
            accumulators[lane] = r32_add_values(accumulators[lane], r32_mul_values(value, value)?)?;
        }
    }

    let mut total = r32_add_values(accumulators[0], POSITIVE_ZERO)?;
    for accumulator in &accumulators[1..] {
        total = r32_add_values(total, *accumulator)?;
    }
    r32_div_values(total, SAMPLES_PER_BLOCK as f32)
}

fn sum_channel_energy(mean_square_five: &[f32; FULLBAND_CHANNEL_COUNT]) -> Result<f32, R32Error> {
    let mut total = mean_square_five[0];
    for energy in &mean_square_five[1..] {
        total = r32_add_values(total, *energy)?;
    }
    Ok(total)
}

fn current_peak(samples: &[FiniteF32]) -> f32 {
    samples
        .iter()
        .map(|sample| sample.value().abs())
        .fold(POSITIVE_ZERO, f32::max)
}

fn peak_scale(peak: f32) -> Result<f32, R32Error> {
    r32_mul_values(peak, TWO)
}

#[derive(Clone, Copy)]
struct AuxiliaryMixCoefficients {
    scale: f32,
    center: f32,
    side: f32,
    pl_side: f32,
}

fn auxiliary_mix_coefficients() -> Result<AuxiliaryMixCoefficients, R32Error> {
    let scale = r32_div_values(AUXILIARY_V, 64.0)?;
    let center_product = r32_mul_values(r32_mul_values(AUXILIARY_N, AUXILIARY_V)?, ONE_OVER_32)?;
    let center = -center_product;
    let pl_side_product = r32_mul_values(r32_mul_values(AUXILIARY_S, AUXILIARY_V)?, ONE_OVER_32)?;
    let pl_side = -pl_side_product;
    Ok(AuxiliaryMixCoefficients {
        scale,
        center,
        side: center,
        pl_side,
    })
}

fn auxiliary_current_peak(
    planes: &FiniteStatisticsPlanes,
    sample_start: usize,
    coefficients: &AuxiliaryMixCoefficients,
) -> Result<f32, R32Error> {
    let mut row_peaks = [POSITIVE_ZERO; 6];
    for offset in 0..SAMPLES_PER_BLOCK {
        let sample = sample_start + offset;
        let left = planes[0][sample].value();
        let right = planes[1][sample].value();
        let center = planes[2][sample].value();
        let left_surround = planes[4][sample].value();
        let right_surround = planes[5][sample].value();

        let left_base = r32_add_values(
            r32_mul_values(left, coefficients.scale)?,
            r32_mul_values(center, coefficients.center)?,
        )?;
        let right_center =
            r32_add_values(POSITIVE_ZERO, r32_mul_values(center, coefficients.center)?)?;
        let right_base = r32_add_values(right_center, r32_mul_values(right, coefficients.scale)?)?;

        let left_downmix_output =
            r32_add_values(left_base, r32_mul_values(left_surround, coefficients.side)?)?;
        let right_downmix_output = r32_add_values(
            right_base,
            r32_mul_values(right_surround, coefficients.side)?,
        )?;
        row_peaks[0] = row_peaks[0].max(left_downmix_output.abs());
        row_peaks[1] = row_peaks[1].max(right_downmix_output.abs());

        let left_side = r32_mul_values(left_surround, coefficients.side)?;
        let left_surround_difference_output = r32_sub_values(
            r32_sub_values(left_base, left_side)?,
            r32_mul_values(right_surround, coefficients.side)?,
        )?;
        let right_surround_difference_output = r32_add_values(
            r32_add_values(right_base, left_side)?,
            r32_mul_values(right_surround, coefficients.side)?,
        )?;
        row_peaks[2] = row_peaks[2].max(left_surround_difference_output.abs());
        row_peaks[3] = row_peaks[3].max(right_surround_difference_output.abs());

        let left_rotation = r32_sub_values(
            r32_sub_values(
                left_base,
                r32_mul_values(
                    r32_mul_values(left_surround, ROTATION_P0)?,
                    coefficients.pl_side,
                )?,
            )?,
            r32_mul_values(
                r32_mul_values(right_surround, ROTATION_P1)?,
                coefficients.pl_side,
            )?,
        )?;
        let right_rotation = r32_add_values(
            r32_add_values(
                right_base,
                r32_mul_values(
                    r32_mul_values(left_surround, ROTATION_P1)?,
                    coefficients.pl_side,
                )?,
            )?,
            r32_mul_values(
                r32_mul_values(right_surround, ROTATION_P0)?,
                coefficients.pl_side,
            )?,
        )?;
        row_peaks[4] = row_peaks[4].max(left_rotation.abs());
        row_peaks[5] = row_peaks[5].max(right_rotation.abs());
    }
    Ok(row_peaks.into_iter().fold(POSITIVE_ZERO, f32::max))
}

fn clamp(value: f32) -> f32 {
    (-ONE).max(value).min(ONE)
}

fn r32_mul_values(left: f32, right: f32) -> Result<f32, R32Error> {
    let left = FiniteF32::new(left).map_err(|_| R32Error::NonFiniteResult)?;
    let right = FiniteF32::new(right).map_err(|_| R32Error::NonFiniteResult)?;
    r32_mul(left, right)
        .map(FiniteF32::value)
        .map_err(|_| R32Error::NonFiniteResult)
}

fn r32_add_values(left: f32, right: f32) -> Result<f32, R32Error> {
    let left = FiniteF32::new(left).map_err(|_| R32Error::NonFiniteResult)?;
    let right = FiniteF32::new(right).map_err(|_| R32Error::NonFiniteResult)?;
    r32_add(left, right)
        .map(FiniteF32::value)
        .map_err(|_| R32Error::NonFiniteResult)
}

fn r32_sub_values(left: f32, right: f32) -> Result<f32, R32Error> {
    let left = FiniteF32::new(left).map_err(|_| R32Error::NonFiniteResult)?;
    let right = FiniteF32::new(right).map_err(|_| R32Error::NonFiniteResult)?;
    r32_sub(left, right)
        .map(FiniteF32::value)
        .map_err(|_| R32Error::NonFiniteResult)
}

fn r32_div_values(left: f32, right: f32) -> Result<f32, R32Error> {
    let result = left / right;
    FiniteF32::new(result)
        .map(FiniteF32::value)
        .map_err(|_| R32Error::NonFiniteResult)
}
