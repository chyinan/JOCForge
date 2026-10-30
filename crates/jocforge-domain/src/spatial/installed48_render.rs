// pattern: Functional Core
//! Shared selected-profile F03W/F04A audio rendering from explicit owner snapshots.

use crate::{
    numeric::{FiniteF32, r32_add, r32_mul},
    spatial::{
        diffusion_filter::DiffusionFilterParameters,
        diffusion_tail::{
            DiffusionTailError, DiffusionTailState, DiffusionTailTransition, preview_diffusion_tail,
        },
        installed48_source_map::{Installed48SourceMap, PrimaryPrefixRoute},
        source_flags::triangle_window,
    },
};

const SOURCE_COUNT: usize = 48;
const HOST_COUNT: usize = 38;
const FRAME_SAMPLES: usize = 1_536;
const RENDERED_SAMPLES: usize = FRAME_SAMPLES * 2;
const PREFIX_ROWS: usize = 11;
const REGULAR_ROWS: usize = 37;
const MEMBERSHIP_GAIN_THRESHOLD: f32 = 0.000_001;
const SELECTED_WIDE_OBJECT_THRESHOLD: f32 = 0.2;
const SELECTED_SPREAD_DENOMINATOR: f32 = 0.4;
const PREFIX_DIRECT_WEIGHT: f64 = 0.85;
const PREFIX_FILTER_WEIGHT: f64 = 0.15;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Installed48AudioControl {
    pub(crate) mute: u8,
    pub(crate) spread: f32,
    pub(crate) diffuse: u8,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Installed48Activity {
    pub(crate) quiet: [u8; REGULAR_ROWS],
    pub(crate) mute_reset: [bool; REGULAR_ROWS],
}

#[derive(Clone, Copy)]
pub(crate) struct Installed48RenderInput<'a> {
    pub(crate) current_pcm_rows: &'a [Vec<u32>],
    /// Physical-host-major bits for the complete 38×1536 before frame.
    pub(crate) previous_pcm_bits: &'a [u32],
    pub(crate) controls: &'a [Installed48AudioControl; REGULAR_ROWS],
    pub(crate) activity: &'a Installed48Activity,
    pub(crate) membership_route: &'a PrimaryPrefixRoute,
    pub(crate) source_map: &'a Installed48SourceMap,
    /// Diffusion state by regular tail ordinal, not by host or source ordinal.
    pub(crate) diffusion_before: &'a [Option<&'a DiffusionTailState>; REGULAR_ROWS],
    pub(crate) diffusion_parameters: Option<&'a DiffusionFilterParameters>,
}

pub(crate) struct Installed48RenderOutput {
    /// Source-major F03W `installed48` after the shared triangular window.
    pub(crate) postwindow_rows: Vec<Vec<u32>>,
    /// Source-major current F04A rows before the window, retained by SpatialControl.
    pub(crate) current_rows_bits: Box<[u32]>,
    /// Raw regular source history in source-map regular-tail order.
    pub(crate) regular_raw_history_bits: Box<[u32]>,
    /// Proposed F04F states in regular-tail order; inactive tails are absent.
    pub(crate) diffusion_after: [Option<DiffusionTailState>; REGULAR_ROWS],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Installed48RenderError {
    InputShape,
    SourceMapBinding,
    ControlDomain,
    MembershipOrder,
    DiffusionParametersUnavailable,
    Diffusion(DiffusionTailError),
    NumericFailure,
}

impl Installed48RenderError {
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::InputShape => "INSTALLED48_RENDER_INPUT_SHAPE",
            Self::SourceMapBinding => "INSTALLED48_RENDER_SOURCE_MAP",
            Self::ControlDomain => "INSTALLED48_RENDER_CONTROL_DOMAIN",
            Self::MembershipOrder => "INSTALLED48_RENDER_MEMBERSHIP_ORDER",
            Self::DiffusionParametersUnavailable => "METER_INSTALLED48_DIFFUSION_PARAMETERS",
            Self::Diffusion(error) => error.code(),
            Self::NumericFailure => "INSTALLED48_RENDER_NUMERIC",
        }
    }
}

struct PrefixRows {
    previous: Vec<Vec<f32>>,
    current: Vec<Vec<f32>>,
    diffusion_after: [Option<DiffusionTailState>; REGULAR_ROWS],
}

/// Renders one callback without owning or mutating any callback history.
pub(crate) fn render_installed48(
    input: Installed48RenderInput<'_>,
) -> Result<Installed48RenderOutput, Installed48RenderError> {
    validate_render_input(&input)?;
    let PrefixRows {
        previous: previous_prefix,
        current: current_prefix,
        diffusion_after,
    } = produce_prefix_rows(&input)?;
    let window = triangle_window().map_err(|_| Installed48RenderError::NumericFailure)?;
    if window.len() != RENDERED_SAMPLES {
        return Err(Installed48RenderError::InputShape);
    }

    let mut previous_current = Vec::with_capacity(SOURCE_COUNT);
    previous_current.extend(previous_prefix);
    let mut current_current = Vec::with_capacity(SOURCE_COUNT);
    current_current.extend(current_prefix);
    let mut regular_raw_history_bits = Vec::with_capacity(REGULAR_ROWS * FRAME_SAMPLES);

    for (tail, binding) in input
        .source_map
        .physical_to_global_source()
        .iter()
        .enumerate()
    {
        let host = usize::from(binding.physical_host().value());
        let current = input
            .current_pcm_rows
            .get(host)
            .ok_or(Installed48RenderError::SourceMapBinding)?;
        let previous_start = host
            .checked_mul(FRAME_SAMPLES)
            .ok_or(Installed48RenderError::InputShape)?;
        let previous_end = previous_start
            .checked_add(FRAME_SAMPLES)
            .ok_or(Installed48RenderError::InputShape)?;
        let previous = input
            .previous_pcm_bits
            .get(previous_start..previous_end)
            .ok_or(Installed48RenderError::InputShape)?;
        let control = input.controls[tail];
        let quiet = input.activity.quiet[tail];
        let factor = spread_factor(control.spread, quiet)?;
        let previous_regular =
            scale_regular_half(previous, factor, input.activity.mute_reset[tail])?;
        let current_regular = scale_regular_half(current, factor, input.activity.mute_reset[tail])?;
        previous_current.push(previous_regular);
        current_current.push(current_regular);
        regular_raw_history_bits.extend(current.iter().copied());
    }

    if previous_current.len() != SOURCE_COUNT || current_current.len() != SOURCE_COUNT {
        return Err(Installed48RenderError::InputShape);
    }

    let mut postwindow_rows = Vec::with_capacity(SOURCE_COUNT);
    for (previous, current) in previous_current.iter().zip(&current_current) {
        if previous.len() != FRAME_SAMPLES || current.len() != FRAME_SAMPLES {
            return Err(Installed48RenderError::InputShape);
        }
        let mut row = Vec::with_capacity(RENDERED_SAMPLES);
        for sample in 0..FRAME_SAMPLES {
            row.push(multiply(previous[sample], window[sample].value())?);
        }
        for sample in 0..FRAME_SAMPLES {
            row.push(multiply(
                current[sample],
                window[FRAME_SAMPLES + sample].value(),
            )?);
        }
        postwindow_rows.push(row.into_iter().map(f32::to_bits).collect());
    }

    Ok(Installed48RenderOutput {
        postwindow_rows,
        current_rows_bits: current_current
            .into_iter()
            .flatten()
            .map(f32::to_bits)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        regular_raw_history_bits: regular_raw_history_bits.into_boxed_slice(),
        diffusion_after,
    })
}

fn validate_render_input(input: &Installed48RenderInput<'_>) -> Result<(), Installed48RenderError> {
    if input.current_pcm_rows.len() != HOST_COUNT
        || input
            .current_pcm_rows
            .iter()
            .any(|row| row.len() != FRAME_SAMPLES)
        || input.previous_pcm_bits.len() != HOST_COUNT * FRAME_SAMPLES
        || input
            .current_pcm_rows
            .iter()
            .flatten()
            .chain(input.previous_pcm_bits)
            .any(|word| !f32::from_bits(*word).is_finite())
        || input
            .controls
            .iter()
            .any(|control| control.mute > 1 || control.diffuse > 1 || !control.spread.is_finite())
        || input.activity.quiet.iter().any(|quiet| *quiet > 1)
        || input
            .activity
            .mute_reset
            .iter()
            .zip(input.controls)
            .zip(input.activity.quiet)
            .any(|((reset, control), quiet)| *reset && (control.mute == 0 || quiet == 0))
        || input.source_map.physical_to_global_source().len() != REGULAR_ROWS
    {
        return Err(Installed48RenderError::InputShape);
    }
    for (tail, binding) in input
        .source_map
        .physical_to_global_source()
        .iter()
        .enumerate()
    {
        if binding.source_index().value() != (PREFIX_ROWS + tail) as u64
            || usize::from(binding.physical_host().value()) >= HOST_COUNT
            || input.activity.mute_reset[tail]
                && (input.controls[tail].mute == 0 || input.activity.quiet[tail] == 0)
        {
            return Err(Installed48RenderError::SourceMapBinding);
        }
    }
    if input
        .membership_route
        .contributions()
        .windows(2)
        .any(|pair| pair[0].source_index().value() >= pair[1].source_index().value())
    {
        return Err(Installed48RenderError::MembershipOrder);
    }
    Ok(())
}

fn produce_prefix_rows(
    input: &Installed48RenderInput<'_>,
) -> Result<PrefixRows, Installed48RenderError> {
    let mut previous_prefix = zero_rows(PREFIX_ROWS);
    let mut current_prefix = zero_rows(PREFIX_ROWS);
    let mut diffusion_after: [Option<DiffusionTailState>; REGULAR_ROWS] =
        std::array::from_fn(|_| None);
    let mut diffusion_outputs = std::array::from_fn::<_, REGULAR_ROWS, _>(|_| None);

    for (tail, binding) in input
        .source_map
        .physical_to_global_source()
        .iter()
        .enumerate()
    {
        let control = input.controls[tail];
        let active =
            control.diffuse != 0 && control.spread > 0.0 && input.activity.quiet[tail] == 0;
        if !active {
            continue;
        }
        let parameters = input
            .diffusion_parameters
            .ok_or(Installed48RenderError::DiffusionParametersUnavailable)?;
        let host = usize::from(binding.physical_host().value());
        let current: [f32; FRAME_SAMPLES] = input.current_pcm_rows[host]
            .iter()
            .copied()
            .map(f32::from_bits)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| Installed48RenderError::InputShape)?;
        let previous_start = host
            .checked_mul(FRAME_SAMPLES)
            .ok_or(Installed48RenderError::InputShape)?;
        let previous_end = previous_start
            .checked_add(FRAME_SAMPLES)
            .ok_or(Installed48RenderError::InputShape)?;
        let previous: [f32; FRAME_SAMPLES] = input.previous_pcm_bits[previous_start..previous_end]
            .iter()
            .copied()
            .map(f32::from_bits)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| Installed48RenderError::InputShape)?;
        let transition = preview_diffusion_tail(
            parameters,
            input.diffusion_before[tail],
            true,
            &previous,
            &current,
        )
        .map_err(Installed48RenderError::Diffusion)?;
        let DiffusionTailTransition::Active(proposal) = transition else {
            return Err(Installed48RenderError::DiffusionParametersUnavailable);
        };
        let (previous_lanes, current_lanes, next_state) = (*proposal).into_parts();
        diffusion_outputs[tail] = Some((previous_lanes, current_lanes));
        diffusion_after[tail] = Some(next_state);
    }

    let mut previous_source = None;
    for contribution in input.membership_route.contributions() {
        let source_index = contribution.source_index().value();
        if previous_source.is_some_and(|previous| previous >= source_index)
            || !(PREFIX_ROWS as u64..(PREFIX_ROWS + REGULAR_ROWS) as u64).contains(&source_index)
        {
            return Err(Installed48RenderError::MembershipOrder);
        }
        previous_source = Some(source_index);
        let tail = usize::try_from(source_index - PREFIX_ROWS as u64)
            .map_err(|_| Installed48RenderError::MembershipOrder)?;
        let control = input.controls[tail];
        let gains = contribution.result().gain_words();
        for prefix in contribution.result().positive_members() {
            let prefix = usize::from(*prefix);
            if prefix >= PREFIX_ROWS {
                return Err(Installed48RenderError::MembershipOrder);
            }
            let gain = f32::from_bits(gains[prefix]);
            if !gain.is_finite() || gain < MEMBERSHIP_GAIN_THRESHOLD {
                return Err(Installed48RenderError::ControlDomain);
            }
            let filtered =
                control.diffuse != 0 && control.spread > 0.0 && input.activity.quiet[tail] == 0;
            let (mode, lane) = if let Some(parameters) = input.diffusion_parameters {
                (
                    parameters.modes()[prefix],
                    parameters.filter_lanes()[prefix],
                )
            } else if filtered {
                return Err(Installed48RenderError::DiffusionParametersUnavailable);
            } else {
                (0, -1)
            };
            if !(-1..=1).contains(&mode) || !(-1..=3).contains(&lane) {
                return Err(Installed48RenderError::ControlDomain);
            }
            let use_filter = filtered && lane >= 0;
            let (direct_scale, filter_scale) = filtered_prefix_scales(gain, mode)?;
            let output_lane = usize::try_from(lane).ok();
            let lanes = diffusion_outputs[tail].as_ref();
            if use_filter && (lanes.is_none() || output_lane.is_none()) {
                return Err(Installed48RenderError::DiffusionParametersUnavailable);
            }
            let host = usize::from(
                input.source_map.physical_to_global_source()[tail]
                    .physical_host()
                    .value(),
            );
            for sample in 0..FRAME_SAMPLES {
                let previous_input =
                    f32::from_bits(input.previous_pcm_bits[host * FRAME_SAMPLES + sample]);
                let current_input = f32::from_bits(input.current_pcm_rows[host][sample]);
                if use_filter {
                    let lane = output_lane.ok_or(Installed48RenderError::ControlDomain)?;
                    let (previous_lanes, current_lanes) =
                        lanes.ok_or(Installed48RenderError::DiffusionParametersUnavailable)?;
                    previous_prefix[prefix][sample] = mix_filtered_prefix_sample(
                        previous_prefix[prefix][sample],
                        previous_input,
                        previous_lanes[lane][sample],
                        direct_scale,
                        filter_scale,
                    )?;
                    current_prefix[prefix][sample] = mix_filtered_prefix_sample(
                        current_prefix[prefix][sample],
                        current_input,
                        current_lanes[lane][sample],
                        direct_scale,
                        filter_scale,
                    )?;
                } else {
                    previous_prefix[prefix][sample] = add(
                        multiply(previous_input, gain)?,
                        previous_prefix[prefix][sample],
                    )?;
                    current_prefix[prefix][sample] = add(
                        multiply(current_input, gain)?,
                        current_prefix[prefix][sample],
                    )?;
                }
            }
        }
    }
    Ok(PrefixRows {
        previous: previous_prefix,
        current: current_prefix,
        diffusion_after,
    })
}

fn scale_regular_half(
    row: &[u32],
    factor: f32,
    mute_reset: bool,
) -> Result<Vec<f32>, Installed48RenderError> {
    if row.len() != FRAME_SAMPLES {
        return Err(Installed48RenderError::InputShape);
    }
    row.iter()
        .map(|word| {
            let sample = if mute_reset {
                0.0
            } else {
                f32::from_bits(*word)
            };
            multiply(sample, factor)
        })
        .collect()
}

fn spread_factor(spread: f32, quiet: u8) -> Result<f32, Installed48RenderError> {
    if !spread.is_finite() || quiet > 1 {
        return Err(Installed48RenderError::ControlDomain);
    }
    if spread <= 0.0 || quiet != 0 {
        return Ok(1.0);
    }
    if spread >= SELECTED_WIDE_OBJECT_THRESHOLD {
        return Ok(0.0);
    }
    let theta = divide(
        multiply(
            spread.min(SELECTED_WIDE_OBJECT_THRESHOLD),
            std::f64::consts::PI as f32,
        )?,
        SELECTED_SPREAD_DENOMINATOR,
    )?;
    let cosine = (f64::from(theta)).cos() as f32;
    multiply(cosine, cosine)
}

pub(crate) fn filtered_prefix_scales(
    gain: f32,
    mode: i32,
) -> Result<(f32, f32), Installed48RenderError> {
    let direct_scale = multiply(sqrt64_to_f32(PREFIX_DIRECT_WEIGHT), gain)?;
    let filter_scale = multiply(
        multiply(sqrt64_to_f32(PREFIX_FILTER_WEIGHT), mode as f32)?,
        gain,
    )?;
    Ok((direct_scale, filter_scale))
}

pub(crate) fn mix_filtered_prefix_sample(
    previous_output: f32,
    source: f32,
    filtered: f32,
    direct_scale: f32,
    filter_scale: f32,
) -> Result<f32, Installed48RenderError> {
    let direct_product = multiply(source, direct_scale)?;
    let filter_product = multiply(filtered, filter_scale)?;
    let first_sum = add(direct_product, previous_output)?;
    add(first_sum, filter_product)
}

fn multiply(left: f32, right: f32) -> Result<f32, Installed48RenderError> {
    r32_mul(
        FiniteF32::new(left).map_err(|_| Installed48RenderError::NumericFailure)?,
        FiniteF32::new(right).map_err(|_| Installed48RenderError::NumericFailure)?,
    )
    .map(FiniteF32::value)
    .map_err(|_| Installed48RenderError::NumericFailure)
}

fn add(left: f32, right: f32) -> Result<f32, Installed48RenderError> {
    r32_add(
        FiniteF32::new(left).map_err(|_| Installed48RenderError::NumericFailure)?,
        FiniteF32::new(right).map_err(|_| Installed48RenderError::NumericFailure)?,
    )
    .map(FiniteF32::value)
    .map_err(|_| Installed48RenderError::NumericFailure)
}

fn divide(left: f32, right: f32) -> Result<f32, Installed48RenderError> {
    FiniteF32::new(left / right)
        .map(FiniteF32::value)
        .map_err(|_| Installed48RenderError::NumericFailure)
}

fn sqrt64_to_f32(value: f64) -> f32 {
    value.sqrt() as f32
}

fn zero_rows(row_count: usize) -> Vec<Vec<f32>> {
    (0..row_count).map(|_| vec![0.0; FRAME_SAMPLES]).collect()
}
