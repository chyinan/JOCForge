// pattern: Functional Core
//! One selected F04F tail's active/inactive and cold/warm callback transition.

use crate::spatial::diffusion_filter::{
    DiffusionFilterError, DiffusionFilterParameters, DiffusionFilterState, preview_filter_block,
};

const FRAME_SAMPLES: usize = 1_536;
const COLD_TAIL_SAMPLES: usize = FRAME_SAMPLES * 2;
const LANE_COUNT: usize = 4;

pub type DiffusionPcmFrame = [f32; FRAME_SAMPLES];
pub type DiffusionOutputLanes = [Vec<f32>; LANE_COUNT];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiffusionTailError {
    StateIncomplete,
    Filter(DiffusionFilterError),
}

impl DiffusionTailError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::StateIncomplete => "STATE_INCOMPLETE",
            Self::Filter(error) => error.code(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DiffusionTailState {
    filter_state: DiffusionFilterState,
    previous_filter_outputs: Option<DiffusionOutputLanes>,
}

impl DiffusionTailState {
    pub const fn filter_state(&self) -> &DiffusionFilterState {
        &self.filter_state
    }

    pub const fn previous_filter_outputs(&self) -> Option<&DiffusionOutputLanes> {
        self.previous_filter_outputs.as_ref()
    }
}

#[derive(Debug)]
pub struct DiffusionTailProposal {
    previous_outputs: DiffusionOutputLanes,
    current_outputs: DiffusionOutputLanes,
    next_state: DiffusionTailState,
}

impl DiffusionTailProposal {
    pub fn into_parts(
        self,
    ) -> (
        DiffusionOutputLanes,
        DiffusionOutputLanes,
        DiffusionTailState,
    ) {
        (self.previous_outputs, self.current_outputs, self.next_state)
    }
}

#[derive(Debug)]
pub enum DiffusionTailTransition {
    Removed,
    Active(Box<DiffusionTailProposal>),
}

/// Previews one tail transition. The caller owns host/source mapping and the raw-history state.
///
/// `previous_pcm` is used only when creating a cold tail; a warm tail obtains its previous
/// outputs from the cache bound to that same tail state.
pub fn preview_diffusion_tail(
    parameters: &DiffusionFilterParameters,
    before: Option<&DiffusionTailState>,
    active: bool,
    previous_pcm: &DiffusionPcmFrame,
    current_pcm: &DiffusionPcmFrame,
) -> Result<DiffusionTailTransition, DiffusionTailError> {
    if !active {
        return Ok(DiffusionTailTransition::Removed);
    }

    let (previous_outputs, current_outputs, next_filter_state) = if let Some(before) = before {
        before
            .filter_state
            .validate(parameters)
            .map_err(DiffusionTailError::Filter)?;
        if current_pcm.iter().any(|sample| !sample.is_finite()) {
            return Err(DiffusionTailError::Filter(
                DiffusionFilterError::InputNonfinite,
            ));
        }
        let cached = before
            .previous_filter_outputs
            .as_ref()
            .ok_or(DiffusionTailError::StateIncomplete)?;
        validate_output_cache(cached)?;
        let (current_outputs, next_state) =
            preview_filter_block(parameters, &before.filter_state, current_pcm)
                .map_err(DiffusionTailError::Filter)?
                .into_parts();
        (copy_lanes(cached), current_outputs, next_state)
    } else {
        let mut input = Vec::with_capacity(COLD_TAIL_SAMPLES);
        input.extend_from_slice(previous_pcm);
        input.extend_from_slice(current_pcm);
        let cold_state = parameters
            .cold_state()
            .map_err(DiffusionTailError::Filter)?;
        let (all_outputs, next_state) = preview_filter_block(parameters, &cold_state, &input)
            .map_err(DiffusionTailError::Filter)?
            .into_parts();
        let (previous_outputs, current_outputs) = split_cold_outputs(all_outputs)?;
        (previous_outputs, current_outputs, next_state)
    };
    let next_state = DiffusionTailState {
        filter_state: next_filter_state,
        previous_filter_outputs: Some(copy_lanes(&current_outputs)),
    };
    Ok(DiffusionTailTransition::Active(Box::new(
        DiffusionTailProposal {
            previous_outputs,
            current_outputs,
            next_state,
        },
    )))
}

fn validate_output_cache(cache: &DiffusionOutputLanes) -> Result<(), DiffusionTailError> {
    if cache.iter().any(|lane| lane.len() != FRAME_SAMPLES) {
        return Err(DiffusionTailError::StateIncomplete);
    }
    if cache.iter().flatten().any(|sample| !sample.is_finite()) {
        return Err(DiffusionTailError::Filter(DiffusionFilterError::StateShape));
    }
    Ok(())
}

fn split_cold_outputs(
    mut lanes: DiffusionOutputLanes,
) -> Result<(DiffusionOutputLanes, DiffusionOutputLanes), DiffusionTailError> {
    if lanes.iter().any(|lane| lane.len() != COLD_TAIL_SAMPLES) {
        return Err(DiffusionTailError::Filter(DiffusionFilterError::StateShape));
    }
    let mut previous = std::array::from_fn(|_| Vec::with_capacity(FRAME_SAMPLES));
    let mut current = std::array::from_fn(|_| Vec::with_capacity(FRAME_SAMPLES));
    for lane in 0..LANE_COUNT {
        current[lane] = lanes[lane].split_off(FRAME_SAMPLES);
        previous[lane] = std::mem::take(&mut lanes[lane]);
    }
    Ok((previous, current))
}

fn copy_lanes(lanes: &DiffusionOutputLanes) -> DiffusionOutputLanes {
    std::array::from_fn(|index| lanes[index].clone())
}

#[cfg(test)]
mod tests;
