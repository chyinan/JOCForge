// pattern: Functional Core
//! Same-frame composition of the selected F06ND statistics and scheduler leaves.

use crate::drc::none_drc_bsi_projection::{
    NoneDrcBsiProjection, project_scheduler_controls_to_bsi,
};
use crate::drc::none_drc_scheduler::{
    NoneDrcSchedulerError, NoneDrcSchedulerFrameInput, NoneDrcSchedulerFrameOutput,
    NoneDrcSchedulerState, transition_none_drc_scheduler_frame,
};
use crate::drc::none_drc_statistics::{
    FiniteStatisticsPlanes, NoneDrcStatisticsError, NoneDrcStatisticsFrameInput,
    NoneDrcStatisticsFrameOutput, NoneDrcStatisticsState, transition_none_drc_statistics_frame,
};
use crate::numeric::{FiniteF32, R32Error};
use crate::timeline::CodecSourceIndex;

/// Complete input for one selected F06ND source frame.
pub struct NoneDrcFrameInput<'a> {
    pub primary_samples: &'a FiniteStatisticsPlanes,
    pub secondary_samples: &'a FiniteStatisticsPlanes,
    pub source_generation: CodecSourceIndex,
    pub dialnorm: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoneDrcFrameOutput {
    pub statistics: NoneDrcStatisticsFrameOutput,
    pub scheduler: NoneDrcSchedulerFrameOutput,
    pub bsi_projection: NoneDrcBsiProjection,
}

#[derive(Debug, PartialEq)]
pub struct NoneDrcFrameTransition {
    pub output: NoneDrcFrameOutput,
    pub next_statistics_state: NoneDrcStatisticsState,
    pub next_scheduler_state: NoneDrcSchedulerState,
}

/// Preserves the two stage errors without introducing a wrapper outcome code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoneDrcFrameError {
    GenerationMismatch,
    Statistics(NoneDrcStatisticsError),
    Scheduler(NoneDrcSchedulerError),
}

impl NoneDrcFrameError {
    pub const fn outcome_code(self) -> Option<&'static str> {
        match self {
            Self::GenerationMismatch => Some("DRC_INPUT_ERROR"),
            Self::Statistics(error) => error.outcome_code(),
            Self::Scheduler(error) => error.outcome_code(),
        }
    }
}

/// Runs statistics first and passes that invocation's outputs to the scheduler.
///
/// This function stages both transitions locally. It returns neither next state
/// unless both stages succeed, and it performs no commit or outer generation.
pub fn transition_none_drc_frame(
    statistics_state: &NoneDrcStatisticsState,
    scheduler_state: &NoneDrcSchedulerState,
    input: &NoneDrcFrameInput<'_>,
) -> Result<NoneDrcFrameTransition, NoneDrcFrameError> {
    if scheduler_state.generation() != input.source_generation.value() {
        return Err(NoneDrcFrameError::GenerationMismatch);
    }

    let statistics_input = NoneDrcStatisticsFrameInput {
        primary_samples: input.primary_samples,
        secondary_samples: input.secondary_samples,
        dialnorm: input.dialnorm,
    };
    let statistics = transition_none_drc_statistics_frame(statistics_state, &statistics_input)
        .map_err(NoneDrcFrameError::Statistics)?;

    let scheduler_input = NoneDrcSchedulerFrameInput {
        dialnorm: input.dialnorm,
        rms_levels_six: finite_block(statistics.output.rms_levels_six)?,
        peak_inputs_six: finite_block(statistics.output.peak_inputs_six)?,
    };
    let scheduler = transition_none_drc_scheduler_frame(scheduler_state, &scheduler_input)
        .map_err(NoneDrcFrameError::Scheduler)?;
    let scheduler_output = scheduler.output;
    let bsi_projection = project_scheduler_controls_to_bsi(&scheduler_output);

    Ok(NoneDrcFrameTransition {
        output: NoneDrcFrameOutput {
            statistics: statistics.output,
            scheduler: scheduler_output,
            bsi_projection,
        },
        next_statistics_state: statistics.next_state,
        next_scheduler_state: scheduler.next_state,
    })
}

fn finite_block(values: [f32; 6]) -> Result<[FiniteF32; 6], NoneDrcFrameError> {
    Ok([
        finite_value(values[0])?,
        finite_value(values[1])?,
        finite_value(values[2])?,
        finite_value(values[3])?,
        finite_value(values[4])?,
        finite_value(values[5])?,
    ])
}

fn finite_value(value: f32) -> Result<FiniteF32, NoneDrcFrameError> {
    FiniteF32::new(value).map_err(|_| {
        NoneDrcFrameError::Statistics(NoneDrcStatisticsError::Numeric(R32Error::NonFiniteResult))
    })
}
