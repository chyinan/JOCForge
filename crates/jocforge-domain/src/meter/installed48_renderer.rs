// pattern: Functional Core
//! Fresh F04A installed48/post-window production for an independent meter input.

use std::collections::BTreeMap;

use crate::{
    frontend::primary_metadata_control::PrimaryMetadataControlRows,
    meter::sample_block_queue::{
        MeterCallbackIndex, MeterSourceSampleCount, SampleBlockQueueError,
    },
    numeric::{FiniteF32, r32_add, r32_mul},
    spatial::{
        diffusion_filter::DiffusionFilterParameters,
        diffusion_tail::{
            DiffusionTailError, DiffusionTailState, DiffusionTailTransition, preview_diffusion_tail,
        },
        installed48_activity::{PrimaryActivityOutput, PrimaryActivityPhase},
        installed48_source_map::{
            Installed48SourceMap, Installed48SourceOwner, PrimaryPrefixRoute,
        },
        source_flags::triangle_window,
    },
    timeline::{AdmSampleIndex, FrontendCallbackIndex},
};

pub const METER_INSTALLED48_ROWS: usize = 48;
pub const METER_INSTALLED48_SAMPLES: usize = 3_072;
pub const METER_INSTALLED48_FRAME_SAMPLES: usize = 1_536;
pub const METER_INSTALLED48_PREFIX_ROWS: usize = 11;
pub const METER_INSTALLED48_REGULAR_ROWS: usize = 37;
const METER_INSTALLED48_HOST_ROWS: usize = 38;
const MEMBERSHIP_GAIN_THRESHOLD: f32 = 0.000_001;
const SELECTED_WIDE_OBJECT_THRESHOLD: f32 = 0.2;
const SELECTED_SPREAD_DENOMINATOR: f32 = 0.4;
const PREFIX_DIRECT_WEIGHT: f64 = 0.85;
const PREFIX_FILTER_WEIGHT: f64 = 0.15;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterInstalled48Error {
    SourceIdentityMismatch,
    SourceExtentMismatch,
    CallbackOrder,
    CallbackPhase,
    InputShape,
    SourceMapBinding,
    ControlDomain,
    MembershipOrder,
    DiffusionParametersUnavailable,
    Diffusion(DiffusionTailError),
    NumericFailure,
    ArithmeticOverflow,
}

impl MeterInstalled48Error {
    pub const fn code(self) -> &'static str {
        match self {
            Self::SourceIdentityMismatch => "METER_INSTALLED48_SOURCE_IDENTITY",
            Self::SourceExtentMismatch => "METER_INSTALLED48_SOURCE_EXTENT",
            Self::CallbackOrder => "METER_INSTALLED48_CALLBACK_ORDER",
            Self::CallbackPhase => "METER_INSTALLED48_CALLBACK_PHASE",
            Self::InputShape => "METER_INSTALLED48_INPUT_SHAPE",
            Self::SourceMapBinding => "METER_INSTALLED48_SOURCE_MAP_BINDING",
            Self::ControlDomain => "METER_INSTALLED48_CONTROL_DOMAIN",
            Self::MembershipOrder => "METER_INSTALLED48_MEMBERSHIP_ORDER",
            Self::DiffusionParametersUnavailable => "METER_INSTALLED48_DIFFUSION_PARAMETERS",
            Self::Diffusion(error) => error.code(),
            Self::NumericFailure => "METER_INSTALLED48_NUMERIC",
            Self::ArithmeticOverflow => "METER_INSTALLED48_ARITHMETIC",
        }
    }
}

impl From<SampleBlockQueueError> for MeterInstalled48Error {
    fn from(_: SampleBlockQueueError) -> Self {
        Self::SourceExtentMismatch
    }
}

#[derive(Debug)]
pub struct MeterInstalled48State {
    source_sha256: [u8; 32],
    source_samples: MeterSourceSampleCount,
    callback_index: MeterCallbackIndex,
    source_cursor: AdmSampleIndex,
    previous_pcm_rows: Vec<Vec<u32>>,
    previous_current_rows: Vec<Vec<u32>>,
    diffusion_states: BTreeMap<u8, DiffusionTailState>,
}

impl MeterInstalled48State {
    pub fn cold(source_sha256: [u8; 32], source_samples: MeterSourceSampleCount) -> Self {
        Self {
            source_sha256,
            source_samples,
            callback_index: MeterCallbackIndex::new(0),
            source_cursor: AdmSampleIndex::new(0),
            previous_pcm_rows: zero_rows(
                METER_INSTALLED48_HOST_ROWS,
                METER_INSTALLED48_FRAME_SAMPLES,
            ),
            previous_current_rows: zero_rows(
                METER_INSTALLED48_ROWS,
                METER_INSTALLED48_FRAME_SAMPLES,
            ),
            diffusion_states: BTreeMap::new(),
        }
    }

    pub const fn callback_index(&self) -> MeterCallbackIndex {
        self.callback_index
    }

    pub const fn source_cursor(&self) -> AdmSampleIndex {
        self.source_cursor
    }
}

#[derive(Clone, Copy)]
pub struct MeterInstalled48Input<'a> {
    pub source_sha256: [u8; 32],
    pub callback_index: MeterCallbackIndex,
    pub source_cursor_before: AdmSampleIndex,
    pub host_input_samples: u16,
    pub source_padding_samples: u16,
    pub raw_pcm_rows: &'a [Vec<u32>],
    pub activity: &'a PrimaryActivityOutput,
    pub membership_route: &'a PrimaryPrefixRoute,
    pub source_map: &'a Installed48SourceMap,
    pub controls: &'a PrimaryMetadataControlRows,
    pub diffusion_parameters: Option<&'a DiffusionFilterParameters>,
}

#[derive(Debug, PartialEq)]
pub struct MeterInstalled48Frame {
    callback_index: MeterCallbackIndex,
    upstream_generation: u64,
    rows: Vec<Vec<u32>>,
}

impl MeterInstalled48Frame {
    pub const fn callback_index(&self) -> MeterCallbackIndex {
        self.callback_index
    }

    pub const fn upstream_generation(&self) -> u64 {
        self.upstream_generation
    }

    pub fn rows(&self) -> &[Vec<u32>] {
        &self.rows
    }
}

#[must_use]
pub struct MeterInstalled48Proposal {
    after: MeterInstalled48State,
    frame: MeterInstalled48Frame,
}

impl MeterInstalled48Proposal {
    pub const fn after(&self) -> &MeterInstalled48State {
        &self.after
    }

    pub const fn frame(&self) -> &MeterInstalled48Frame {
        &self.frame
    }

    pub fn into_parts(self) -> (MeterInstalled48State, MeterInstalled48Frame) {
        (self.after, self.frame)
    }
}

/// Stages one selected normal callback without changing the main spatial owner.
pub fn stage_meter_installed48(
    before: &MeterInstalled48State,
    input: MeterInstalled48Input<'_>,
) -> Result<MeterInstalled48Proposal, MeterInstalled48Error> {
    validate_input(before, &input)?;
    let regular_rows = regular_rows(input.source_map)?;
    let controls = regular_controls(input.source_map, &regular_rows)?;
    let mut previous_regular = Vec::with_capacity(METER_INSTALLED48_REGULAR_ROWS);
    let mut current_regular = Vec::with_capacity(METER_INSTALLED48_REGULAR_ROWS);
    for (tail, (host, control)) in regular_rows.iter().zip(&controls).enumerate() {
        let current = &input.raw_pcm_rows[usize::from(*host)];
        let previous = &before.previous_pcm_rows[usize::from(*host)];
        let quiet = input.activity.quiet37()[tail];
        if control.mute != 0 {
            return Err(MeterInstalled48Error::ControlDomain);
        }
        let factor = spread_factor(control.spread, quiet)?;
        previous_regular.push(scale_row(previous, factor)?);
        current_regular.push(scale_row(current, factor)?);
    }

    let prefix_rows = produce_prefix_rows(before, &input, &regular_rows, &controls)?;
    let mut previous_current = Vec::with_capacity(METER_INSTALLED48_ROWS);
    previous_current.extend(prefix_rows.previous);
    previous_current.extend(previous_regular);
    let mut current_current = Vec::with_capacity(METER_INSTALLED48_ROWS);
    current_current.extend(prefix_rows.current);
    current_current.extend(current_regular);

    let window = triangle_window().map_err(|_| MeterInstalled48Error::NumericFailure)?;
    if window.len() != METER_INSTALLED48_SAMPLES {
        return Err(MeterInstalled48Error::InputShape);
    }
    let mut rendered_rows = Vec::with_capacity(METER_INSTALLED48_ROWS);
    for (previous, current) in previous_current.iter().zip(&current_current) {
        let mut row = Vec::with_capacity(METER_INSTALLED48_SAMPLES);
        for sample in 0..METER_INSTALLED48_FRAME_SAMPLES {
            row.push(multiply(previous[sample], window[sample].value())?);
        }
        for sample in 0..METER_INSTALLED48_FRAME_SAMPLES {
            row.push(multiply(
                current[sample],
                window[METER_INSTALLED48_FRAME_SAMPLES + sample].value(),
            )?);
        }
        rendered_rows.push(row.into_iter().map(f32::to_bits).collect());
    }

    let next_callback = before
        .callback_index
        .value()
        .checked_add(1)
        .ok_or(MeterInstalled48Error::ArithmeticOverflow)?;
    let next_cursor = before
        .source_cursor
        .value()
        .checked_add(u64::from(input.host_input_samples))
        .ok_or(MeterInstalled48Error::ArithmeticOverflow)?;
    if next_cursor > before.source_samples.value() {
        return Err(MeterInstalled48Error::SourceExtentMismatch);
    }
    let after = MeterInstalled48State {
        source_sha256: before.source_sha256,
        source_samples: before.source_samples,
        callback_index: MeterCallbackIndex::new(next_callback),
        source_cursor: AdmSampleIndex::new(next_cursor),
        previous_pcm_rows: input.raw_pcm_rows.to_vec(),
        previous_current_rows: current_current
            .iter()
            .map(|row| row.iter().copied().map(f32::to_bits).collect())
            .collect(),
        diffusion_states: prefix_rows.diffusion_states,
    };
    Ok(MeterInstalled48Proposal {
        after,
        frame: MeterInstalled48Frame {
            callback_index: input.callback_index,
            upstream_generation: next_callback,
            rows: rendered_rows,
        },
    })
}

fn validate_input(
    before: &MeterInstalled48State,
    input: &MeterInstalled48Input<'_>,
) -> Result<(), MeterInstalled48Error> {
    if before.source_sha256 != input.source_sha256
        || input.source_map.source_binding().source_receipt_sha256() != &input.source_sha256
        || input.controls.source_binding() != input.source_map.source_binding()
    {
        return Err(MeterInstalled48Error::SourceIdentityMismatch);
    }
    if input.callback_index != before.callback_index
        || input.source_cursor_before != before.source_cursor
        || input.activity.callback_index().value() != input.callback_index.value()
        || input
            .activity
            .host_frame()
            .map(FrontendCallbackIndex::value)
            != Some(input.callback_index.value())
        || input.activity.actual_source_samples() != input.host_input_samples
        || input.membership_route.writer_indices() != input.activity.prefix_writer_indices()
        || input
            .membership_route
            .contributions()
            .iter()
            .any(|contribution| {
                contribution.source_index().value() < METER_INSTALLED48_PREFIX_ROWS as u64
                    || contribution.source_index().value()
                        >= (METER_INSTALLED48_PREFIX_ROWS + METER_INSTALLED48_REGULAR_ROWS) as u64
            })
    {
        return Err(MeterInstalled48Error::CallbackOrder);
    }
    if input.activity.phase() != PrimaryActivityPhase::NormalRead {
        return Err(MeterInstalled48Error::CallbackPhase);
    }
    let expected_cursor = input
        .callback_index
        .value()
        .checked_mul(METER_INSTALLED48_FRAME_SAMPLES as u64)
        .ok_or(MeterInstalled48Error::ArithmeticOverflow)?;
    let remaining_samples = before
        .source_samples
        .value()
        .checked_sub(before.source_cursor.value())
        .ok_or(MeterInstalled48Error::SourceExtentMismatch)?;
    let expected_host_samples = remaining_samples.min(METER_INSTALLED48_FRAME_SAMPLES as u64);
    let expected_padding_samples = METER_INSTALLED48_FRAME_SAMPLES as u64 - expected_host_samples;
    let normal_callback_count = before
        .source_samples
        .value()
        .div_ceil(METER_INSTALLED48_FRAME_SAMPLES as u64);
    if expected_cursor != input.source_cursor_before.value()
        || input.callback_index.value() >= normal_callback_count
        || u64::from(input.host_input_samples) != expected_host_samples
        || u64::from(input.source_padding_samples) != expected_padding_samples
        || input.raw_pcm_rows.len() != METER_INSTALLED48_HOST_ROWS
        || input.raw_pcm_rows.iter().any(|row| {
            row.len() != METER_INSTALLED48_FRAME_SAMPLES
                || row.iter().any(|word| !f32::from_bits(*word).is_finite())
        })
        || input.activity.quiet37().len() != METER_INSTALLED48_REGULAR_ROWS
        || before.previous_pcm_rows.len() != METER_INSTALLED48_HOST_ROWS
        || before.previous_current_rows.len() != METER_INSTALLED48_ROWS
    {
        return Err(MeterInstalled48Error::InputShape);
    }
    if before.source_samples.value() != input.source_map.source_binding().source_samples().value()
        || input.source_map.source_binding().data_bytes() % 114 != 0
        || input.source_map.source_binding().data_bytes() / 114 != before.source_samples.value()
    {
        return Err(MeterInstalled48Error::SourceExtentMismatch);
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct TailControl {
    mute: u8,
    spread: f32,
    diffuse: u8,
}

struct PrefixRows {
    previous: Vec<Vec<f32>>,
    current: Vec<Vec<f32>>,
    diffusion_states: BTreeMap<u8, DiffusionTailState>,
}

fn regular_rows(source_map: &Installed48SourceMap) -> Result<Vec<u8>, MeterInstalled48Error> {
    let mut hosts = Vec::with_capacity(METER_INSTALLED48_REGULAR_ROWS);
    for binding in source_map.physical_to_global_source() {
        let row = source_map
            .row_for_source(binding.source_index())
            .ok_or(MeterInstalled48Error::SourceMapBinding)?;
        let Installed48SourceOwner::PhysicalTrack { track, .. } = row.owner() else {
            return Err(MeterInstalled48Error::SourceMapBinding);
        };
        if track.physical_host() != binding.physical_host() {
            return Err(MeterInstalled48Error::SourceMapBinding);
        }
        hosts.push(binding.physical_host().value());
    }
    if hosts.len() != METER_INSTALLED48_REGULAR_ROWS
        || hosts.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(MeterInstalled48Error::SourceMapBinding);
    }
    Ok(hosts)
}

fn regular_controls(
    source_map: &Installed48SourceMap,
    hosts: &[u8],
) -> Result<Vec<TailControl>, MeterInstalled48Error> {
    hosts
        .iter()
        .map(|host| {
            let binding = source_map
                .physical_to_global_source()
                .iter()
                .find(|binding| binding.physical_host().value() == *host)
                .ok_or(MeterInstalled48Error::SourceMapBinding)?;
            let row = source_map
                .row_for_source(binding.source_index())
                .ok_or(MeterInstalled48Error::SourceMapBinding)?;
            let mute = row
                .input_mute()
                .ok_or(MeterInstalled48Error::ControlDomain)?;
            let words = row.metadata_words_8_to_55();
            let spread = f32::from_bits(words[8]);
            let diffuse =
                u8::try_from(words[9]).map_err(|_| MeterInstalled48Error::ControlDomain)?;
            let component =
                u8::try_from(words[10]).map_err(|_| MeterInstalled48Error::ControlDomain)?;
            if mute > 1 || !spread.is_finite() || diffuse > 1 || component != 0 {
                return Err(MeterInstalled48Error::ControlDomain);
            }
            Ok(TailControl {
                mute,
                spread,
                diffuse,
            })
        })
        .collect()
}

fn produce_prefix_rows(
    before: &MeterInstalled48State,
    input: &MeterInstalled48Input<'_>,
    hosts: &[u8],
    controls: &[TailControl],
) -> Result<PrefixRows, MeterInstalled48Error> {
    let mut previous_prefix = zero_float_rows(
        METER_INSTALLED48_PREFIX_ROWS,
        METER_INSTALLED48_FRAME_SAMPLES,
    );
    let mut current_prefix = zero_float_rows(
        METER_INSTALLED48_PREFIX_ROWS,
        METER_INSTALLED48_FRAME_SAMPLES,
    );
    let mut next_diffusion_states = BTreeMap::new();
    let mut diffusion_outputs = BTreeMap::new();
    for (tail, host) in hosts.iter().copied().enumerate() {
        let control = controls[tail];
        let quiet = input.activity.quiet37()[tail];
        let active_diffusion = control.diffuse != 0 && control.spread > 0.0 && quiet == 0;
        if active_diffusion {
            let parameters = input
                .diffusion_parameters
                .ok_or(MeterInstalled48Error::DiffusionParametersUnavailable)?;
            let previous: [f32; METER_INSTALLED48_FRAME_SAMPLES] = before.previous_pcm_rows
                [usize::from(host)]
            .iter()
            .copied()
            .map(f32::from_bits)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| MeterInstalled48Error::InputShape)?;
            let current: [f32; METER_INSTALLED48_FRAME_SAMPLES] = input.raw_pcm_rows
                [usize::from(host)]
            .iter()
            .copied()
            .map(f32::from_bits)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| MeterInstalled48Error::InputShape)?;
            let transition = preview_diffusion_tail(
                parameters,
                before.diffusion_states.get(&host),
                true,
                &previous,
                &current,
            )
            .map_err(MeterInstalled48Error::Diffusion)?;
            let DiffusionTailTransition::Active(proposal) = transition else {
                return Err(MeterInstalled48Error::DiffusionParametersUnavailable);
            };
            let (previous_lanes, current_lanes, next_state) = (*proposal).into_parts();
            diffusion_outputs.insert(host, (previous_lanes, current_lanes));
            next_diffusion_states.insert(host, next_state);
        }
    }

    let mut previous_source = None;
    for contribution in input.membership_route.contributions() {
        let source_index = contribution.source_index().value();
        if previous_source.is_some_and(|previous| previous >= source_index) {
            return Err(MeterInstalled48Error::MembershipOrder);
        }
        previous_source = Some(source_index);
        let tail = usize::try_from(source_index - METER_INSTALLED48_PREFIX_ROWS as u64)
            .map_err(|_| MeterInstalled48Error::MembershipOrder)?;
        if tail >= hosts.len() {
            return Err(MeterInstalled48Error::MembershipOrder);
        }
        let host = hosts[tail];
        let control = controls[tail];
        let gains = contribution.result().gain_words();
        for prefix in contribution.result().positive_members() {
            let prefix = usize::from(*prefix);
            if prefix >= METER_INSTALLED48_PREFIX_ROWS {
                return Err(MeterInstalled48Error::MembershipOrder);
            }
            let gain = f32::from_bits(gains[prefix]);
            if !gain.is_finite() || gain < MEMBERSHIP_GAIN_THRESHOLD {
                return Err(MeterInstalled48Error::ControlDomain);
            }
            let filtered =
                control.diffuse != 0 && control.spread > 0.0 && input.activity.quiet37()[tail] == 0;
            let (mode, lane) = if let Some(parameters) = input.diffusion_parameters {
                (
                    parameters.modes()[prefix],
                    parameters.filter_lanes()[prefix],
                )
            } else if filtered {
                return Err(MeterInstalled48Error::DiffusionParametersUnavailable);
            } else {
                (0, -1)
            };
            if !(-1..=1).contains(&mode) || !(-1..=3).contains(&lane) {
                return Err(MeterInstalled48Error::ControlDomain);
            }
            let use_filter = filtered && lane >= 0;
            let direct_scale = multiply(sqrt64_to_f32(PREFIX_DIRECT_WEIGHT), gain)?;
            let filter_scale = multiply(
                multiply(sqrt64_to_f32(PREFIX_FILTER_WEIGHT), mode as f32)?,
                gain,
            )?;
            let output_lane = usize::try_from(lane).ok();
            let lanes = diffusion_outputs.get(&host);
            if use_filter && (lanes.is_none() || output_lane.is_none()) {
                return Err(MeterInstalled48Error::DiffusionParametersUnavailable);
            }
            for sample in 0..METER_INSTALLED48_FRAME_SAMPLES {
                let previous_input =
                    f32::from_bits(before.previous_pcm_rows[usize::from(host)][sample]);
                let current_input = f32::from_bits(input.raw_pcm_rows[usize::from(host)][sample]);
                if use_filter {
                    let lane = output_lane.ok_or(MeterInstalled48Error::ControlDomain)?;
                    let (previous_lanes, current_lanes) =
                        lanes.ok_or(MeterInstalled48Error::DiffusionParametersUnavailable)?;
                    let previous_direct = multiply(previous_input, direct_scale)?;
                    let previous_filtered = multiply(previous_lanes[lane][sample], filter_scale)?;
                    previous_prefix[prefix][sample] = add(
                        add(previous_direct, previous_prefix[prefix][sample])?,
                        previous_filtered,
                    )?;
                    let current_direct = multiply(current_input, direct_scale)?;
                    let current_filtered = multiply(current_lanes[lane][sample], filter_scale)?;
                    current_prefix[prefix][sample] = add(
                        add(current_direct, current_prefix[prefix][sample])?,
                        current_filtered,
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
        diffusion_states: next_diffusion_states,
    })
}

fn spread_factor(spread: f32, quiet: u8) -> Result<f32, MeterInstalled48Error> {
    if !spread.is_finite() || quiet > 1 {
        return Err(MeterInstalled48Error::ControlDomain);
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

fn scale_row(row: &[u32], factor: f32) -> Result<Vec<f32>, MeterInstalled48Error> {
    row.iter()
        .map(|word| multiply(f32::from_bits(*word), factor))
        .collect()
}

fn multiply(left: f32, right: f32) -> Result<f32, MeterInstalled48Error> {
    r32_mul(
        FiniteF32::new(left).map_err(|_| MeterInstalled48Error::NumericFailure)?,
        FiniteF32::new(right).map_err(|_| MeterInstalled48Error::NumericFailure)?,
    )
    .map(FiniteF32::value)
    .map_err(|_| MeterInstalled48Error::NumericFailure)
}

fn add(left: f32, right: f32) -> Result<f32, MeterInstalled48Error> {
    r32_add(
        FiniteF32::new(left).map_err(|_| MeterInstalled48Error::NumericFailure)?,
        FiniteF32::new(right).map_err(|_| MeterInstalled48Error::NumericFailure)?,
    )
    .map(FiniteF32::value)
    .map_err(|_| MeterInstalled48Error::NumericFailure)
}

fn divide(left: f32, right: f32) -> Result<f32, MeterInstalled48Error> {
    FiniteF32::new(left / right)
        .map(FiniteF32::value)
        .map_err(|_| MeterInstalled48Error::NumericFailure)
}

fn sqrt64_to_f32(value: f64) -> f32 {
    value.sqrt() as f32
}

fn zero_rows(row_count: usize, sample_count: usize) -> Vec<Vec<u32>> {
    (0..row_count).map(|_| vec![0; sample_count]).collect()
}

fn zero_float_rows(row_count: usize, sample_count: usize) -> Vec<Vec<f32>> {
    (0..row_count).map(|_| vec![0.0; sample_count]).collect()
}
