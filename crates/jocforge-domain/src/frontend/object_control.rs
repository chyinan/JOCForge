// pattern: Functional Core
//! Selected F01E object-event to control52 value production.

use crate::frontend::bed_channel_join::SelectedBedChannelJoin;
use crate::frontend::dbmd_sections::DbmdTrackFields;
use crate::numeric::{FiniteF32, r32_add, r32_mul};

const HOST_CHANNEL_COUNT: u8 = 38;
const SAMPLE_RATE: f32 = 48_000.0;
const SAMPLER_PERIOD: f32 = 512.0;
const SAMPLER_TAU: f32 = f32::from_bits(0x3ccc_cccd);
const THIRD: f32 = f32::from_bits(0x3eaa_aaab);

/// One selected-profile object event value. DBMD fields remain a separate parsed
/// input to the ordered set join and do not populate control52 slots.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectedObjectEvent {
    host_channel_index: u8,
    start_sample: u64,
    duration_samples: u64,
    programme_samples: u64,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
    width: Option<f64>,
    depth: Option<f64>,
    height: Option<f64>,
    diffuse: Option<i32>,
}

impl SelectedObjectEvent {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        host_channel_index: u8,
        start_sample: u64,
        duration_samples: u64,
        programme_samples: u64,
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        width: Option<f64>,
        depth: Option<f64>,
        height: Option<f64>,
        diffuse: Option<i32>,
    ) -> Self {
        Self {
            host_channel_index,
            start_sample,
            duration_samples,
            programme_samples,
            x,
            y,
            z,
            width,
            depth,
            height,
            diffuse,
        }
    }

    pub const fn host_channel_index(&self) -> u8 {
        self.host_channel_index
    }

    pub const fn start_sample(&self) -> u64 {
        self.start_sample
    }

    pub const fn duration_samples(&self) -> u64 {
        self.duration_samples
    }

    pub const fn programme_samples(&self) -> u64 {
        self.programme_samples
    }

    /// Returns the authored optional XYZ fields before writer conversion.
    /// Omitted coordinates and the exact floating-point values are preserved.
    pub const fn authored_coordinates(&self) -> [Option<f64>; 3] {
        [self.x, self.y, self.z]
    }

    /// Returns the authored optional width, depth, and height fields before
    /// spread validation or uniform-width conversion.
    pub const fn authored_spread(&self) -> [Option<f64>; 3] {
        [self.width, self.depth, self.height]
    }

    /// Returns the admitted discrete diffuse value, preserving field presence.
    pub const fn diffuse(&self) -> Option<i32> {
        self.diffuse
    }
}

/// Errors local to selected event value production; not native status values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectControlError {
    HostDomain,
    DbmdHostMismatch,
    EventCount,
    EventOrder,
    EventStart,
    EventDuration,
    NonFiniteGeometry,
    SpreadMismatch,
    NumericFailure,
}

impl ObjectControlError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::HostDomain => "EVENT_HOST",
            Self::DbmdHostMismatch => "EVENT_DBMD_JOIN",
            Self::EventCount => "EVENT_COUNT",
            Self::EventOrder => "EVENT_ORDER",
            Self::EventStart => "EVENT_START",
            Self::EventDuration => "EVENT_DURATION",
            Self::NonFiniteGeometry => "EVENT_GEOMETRY",
            Self::SpreadMismatch => "EVENT_SPREAD",
            Self::NumericFailure => "EVENT_NUMERIC",
        }
    }
}

/// The 13 control52 slots represented as their exact little-endian words.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectControl52([u32; 13]);

impl ObjectControl52 {
    pub const fn words(self) -> [u32; 13] {
        self.0
    }

    /// Wraps a row already committed by the F01E owner without re-sampling or rewriting it.
    pub(crate) const fn from_committed_snapshot_words(words: [u32; 13]) -> Self {
        Self(words)
    }

    /// Selected F03P source fields copied from their approved Object13 slots.
    pub const fn point_response_source_fields(self) -> [u32; 4] {
        [self.0[8], self.0[0], self.0[11], self.0[12]]
    }

    #[cfg(test)]
    pub(crate) const fn fixture_from_words(words: [u32; 13]) -> Self {
        Self(words)
    }

    pub(crate) const fn from_normal_sample(
        sampled_xyz: [u32; 3],
        uniform_width: u32,
        diffuse: Option<i32>,
    ) -> Self {
        let mut words = [0_u32; 13];
        words[0] = 0;
        words[1] = 0;
        words[2] = 3;
        words[3] = 2;
        words[4] = 0;
        words[5] = sampled_xyz[0];
        words[6] = sampled_xyz[1];
        words[7] = sampled_xyz[2];
        words[8] = 0;
        words[9] = uniform_width;
        words[10] = match diffuse {
            Some(value) if value != 0 => 1,
            Some(_) | None => 0,
        };
        words[11] = 0;
        words[12] = 1;
        Self(words)
    }
}

/// Ordered object rows derived from the exact non-bed complement of a validated
/// selected bed/host join. No row is returned when any event fails validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedObjectControlSet {
    rows: [ObjectControl52; 28],
    dbmd_tracks: [DbmdTrackFields; 28],
}

impl SelectedObjectControlSet {
    pub const fn rows(&self) -> &[ObjectControl52; 28] {
        &self.rows
    }

    pub const fn dbmd_tracks(&self) -> &[DbmdTrackFields; 28] {
        &self.dbmd_tracks
    }
}

/// Produces one selected control52 row from a single validated object event.
/// This is a value builder only: it owns no callback, hold, reset, or commit state.
pub fn build_selected_object_control52(
    event: SelectedObjectEvent,
) -> Result<ObjectControl52, ObjectControlError> {
    if event.host_channel_index >= HOST_CHANNEL_COUNT {
        return Err(ObjectControlError::HostDomain);
    }
    if event.start_sample != 0 {
        return Err(ObjectControlError::EventStart);
    }
    if event.duration_samples < event.programme_samples || event.programme_samples == 0 {
        return Err(ObjectControlError::EventDuration);
    }

    let xyz = writer_coordinates(&event)?;
    let sampler_xyz = sample_event_from_previous(xyz, xyz)?;
    let uniform_width = writer_uniform_width(&event)?;
    Ok(ObjectControl52::from_normal_sample(
        sampler_xyz,
        uniform_width,
        event.diffuse,
    ))
}

/// Builds the complete selected 28-object control set in ascending physical-host
/// order. The caller must supply the previously validated bed/host identity join.
pub fn build_selected_object_control_set(
    events: &[SelectedObjectEvent],
    dbmd_tracks: &[DbmdTrackFields],
    bed_join: &SelectedBedChannelJoin,
) -> Result<SelectedObjectControlSet, ObjectControlError> {
    if events.len() != 28 || dbmd_tracks.len() != 28 {
        return Err(ObjectControlError::EventCount);
    }
    let mut rows = Vec::with_capacity(events.len());
    for ((event, dbmd_track), expected_host) in
        events.iter().zip(dbmd_tracks).zip(bed_join.object_hosts())
    {
        if event.host_channel_index != *expected_host {
            return Err(ObjectControlError::EventOrder);
        }
        if dbmd_track.track_index() != u16::from(*expected_host) {
            return Err(ObjectControlError::DbmdHostMismatch);
        }
        rows.push(build_selected_object_control52(*event)?);
    }
    let rows = rows
        .try_into()
        .map_err(|_| ObjectControlError::EventCount)?;
    let dbmd_tracks: [DbmdTrackFields; 28] = dbmd_tracks
        .to_vec()
        .try_into()
        .map_err(|_| ObjectControlError::EventCount)?;
    Ok(SelectedObjectControlSet { rows, dbmd_tracks })
}

pub(crate) fn writer_coordinates(
    event: &SelectedObjectEvent,
) -> Result<[u32; 3], ObjectControlError> {
    event_coordinates(event.x, event.y, event.z)
}

pub(crate) fn writer_uniform_width(event: &SelectedObjectEvent) -> Result<u32, ObjectControlError> {
    uniform_width(event.width, event.depth, event.height)
}

fn event_coordinates(
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<[u32; 3], ObjectControlError> {
    let x = coordinate(x, 0.5, false)?;
    let y = coordinate(y, 0.5, true)?;
    let z = z_coordinate(z)?;
    Ok([x, y, z])
}

fn coordinate(value: Option<f64>, default: f64, invert: bool) -> Result<u32, ObjectControlError> {
    let Some(value) = value else {
        return Ok((default as f32).to_bits());
    };
    if !value.is_finite() {
        return Err(ObjectControlError::NonFiniteGeometry);
    }
    let authored = if value.to_bits() == (-1000.0_f64).to_bits() {
        value
    } else {
        value.clamp(-1.0, 1.0)
    };
    let transformed = if invert {
        0.5_f64 - (authored * 0.5_f64)
    } else {
        (authored * 0.5_f64) + 0.5_f64
    };
    let writer_value = (transformed as f32).clamp(0.0, 1.0);
    Ok(writer_value.to_bits())
}

fn z_coordinate(value: Option<f64>) -> Result<u32, ObjectControlError> {
    let Some(value) = value else {
        return Ok(0.0_f32.to_bits());
    };
    if !value.is_finite() {
        return Err(ObjectControlError::NonFiniteGeometry);
    }
    let authored = if value.to_bits() == (-1000.0_f64).to_bits() {
        value
    } else {
        value.clamp(-1.0, 1.0)
    };
    let writer_value = (authored as f32).clamp(-1.0, 1.0);
    Ok(writer_value.to_bits())
}

pub(crate) fn sample_event_from_previous(
    writer_xyz: [u32; 3],
    previous_xyz: [u32; 3],
) -> Result<[u32; 3], ObjectControlError> {
    let denominator = r32_mul(finite(SAMPLE_RATE)?, finite(SAMPLER_TAU)?)
        .map_err(|_| ObjectControlError::NumericFailure)?;
    let ratio = finite(SAMPLER_PERIOD / denominator.value())?;
    let history = finite((-ratio.value()).exp())?;
    let incoming = r32_sub_one(history)?;
    let writer = writer_xyz.map(f32::from_bits);
    let mut previous = previous_xyz.map(f32::from_bits);
    for _slot in 0..2 {
        for (coordinate, previous_value) in writer.iter().zip(previous.iter_mut()) {
            let coordinate = finite(*coordinate)?;
            let input_component =
                r32_mul(coordinate, incoming).map_err(|_| ObjectControlError::NumericFailure)?;
            let previous_component = r32_mul(finite(*previous_value)?, history)
                .map_err(|_| ObjectControlError::NumericFailure)?;
            *previous_value = r32_add(input_component, previous_component)
                .map_err(|_| ObjectControlError::NumericFailure)?
                .value();
        }
    }
    Ok(previous.map(f32::to_bits))
}

fn uniform_width(
    width: Option<f64>,
    depth: Option<f64>,
    height: Option<f64>,
) -> Result<u32, ObjectControlError> {
    if !same_present_values([width, depth, height]) {
        return Err(ObjectControlError::SpreadMismatch);
    }
    let width = match width {
        None => 0.0_f32,
        Some(value) if !value.is_finite() => return Err(ObjectControlError::NonFiniteGeometry),
        Some(-1000.0) => -1000.0_f32,
        Some(value) => value.clamp(0.0, 1.0) as f32,
    };
    let first =
        r32_add(finite(width)?, finite(width)?).map_err(|_| ObjectControlError::NumericFailure)?;
    let sum = r32_add(first, finite(width)?).map_err(|_| ObjectControlError::NumericFailure)?;
    r32_mul(sum, finite(THIRD)?)
        .map(FiniteF32::to_bits)
        .map_err(|_| ObjectControlError::NumericFailure)
}

#[allow(clippy::float_cmp)]
fn same_present_values(values: [Option<f64>; 3]) -> bool {
    let mut present = values.into_iter().flatten();
    let Some(first) = present.next() else {
        return true;
    };
    present.all(|value| value == first)
}

fn finite(value: f32) -> Result<FiniteF32, ObjectControlError> {
    FiniteF32::new(value).map_err(|_| ObjectControlError::NumericFailure)
}

fn r32_sub_one(value: FiniteF32) -> Result<FiniteF32, ObjectControlError> {
    let one = finite(1.0)?;
    crate::numeric::r32_sub(one, value).map_err(|_| ObjectControlError::NumericFailure)
}
