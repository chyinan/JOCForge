// pattern: Functional Core
//! Typed selected callback clock and staged F06ND frame transition.

use crate::drc::none_drc_frame::{
    NoneDrcFrameError, NoneDrcFrameInput, NoneDrcFrameTransition, transition_none_drc_frame,
};
use crate::drc::none_drc_override_fifo::{
    NoneDrcOverrideFifoError, NoneDrcOverrideFifoState, NoneDrcOverrideFifoTransition,
    transition_none_drc_override_fifo,
};
use crate::drc::none_drc_statistics::FiniteStatisticsPlanes;
use crate::timeline::{
    ClockError, CodecSourceIndex, FrontendCallbackIndex, JocDispatchIndex, JocGeneratedWriterIndex,
    PublicationDisposition, publication_for_source, source_for_callback,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoneDrcCallbackClock {
    pub callback_index: FrontendCallbackIndex,
    pub source_index: CodecSourceIndex,
    pub dispatch_index: JocDispatchIndex,
    pub generated_writer_index: JocGeneratedWriterIndex,
    pub publication: PublicationDisposition,
}

pub struct NoneDrcCallbackInput<'a> {
    pub callback_index: FrontendCallbackIndex,
    pub primary_samples: &'a FiniteStatisticsPlanes,
    pub secondary_samples: &'a FiniteStatisticsPlanes,
    pub dialnorm: i32,
}

#[derive(Debug, PartialEq)]
pub struct NoneDrcCallbackTransition {
    clock: NoneDrcCallbackClock,
    frame: NoneDrcFrameTransition,
    fifo_transition: NoneDrcOverrideFifoTransition,
    dialnorm: i32,
}

impl NoneDrcCallbackTransition {
    pub const fn clock(&self) -> NoneDrcCallbackClock {
        self.clock
    }

    pub const fn frame(&self) -> &NoneDrcFrameTransition {
        &self.frame
    }

    pub const fn fifo_transition(&self) -> &NoneDrcOverrideFifoTransition {
        &self.fifo_transition
    }

    pub const fn dialnorm(&self) -> i32 {
        self.dialnorm
    }

    pub fn into_parts(
        self,
    ) -> (
        NoneDrcCallbackClock,
        NoneDrcFrameTransition,
        NoneDrcOverrideFifoTransition,
        i32,
    ) {
        (self.clock, self.frame, self.fifo_transition, self.dialnorm)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoneDrcCallbackError {
    Clock(ClockError),
    Fifo(NoneDrcOverrideFifoError),
    Frame(NoneDrcFrameError),
}

impl NoneDrcCallbackError {
    pub const fn outcome_code(self) -> Option<&'static str> {
        match self {
            Self::Clock(_) => None,
            Self::Fifo(error) => Some(error.outcome_code()),
            Self::Frame(error) => error.outcome_code(),
        }
    }
}

/// Projects one callback into its selected source, dispatch, writer and AU domains.
///
/// Priming callbacks 0 and 1 have no codec source and return `None`.
pub fn project_none_drc_callback_clock(
    callback_index: FrontendCallbackIndex,
) -> Result<Option<NoneDrcCallbackClock>, ClockError> {
    let Some(source_index) = source_for_callback(callback_index) else {
        return Ok(None);
    };
    let dispatch_index = source_index.dispatch()?;
    let generated_writer_index = dispatch_index.generated_writer()?;
    let publication = publication_for_source(source_index)?;
    Ok(Some(NoneDrcCallbackClock {
        callback_index,
        source_index,
        dispatch_index,
        generated_writer_index,
        publication,
    }))
}

/// Derives the F06ND source generation from the callback before staging the frame.
///
/// The returned frame and zero-record FIFO proposals share one callback lineage.
/// This function owns no commit, parent reset, frame write or EOF flush.
pub fn transition_none_drc_callback(
    statistics_state: &crate::drc::none_drc_statistics::NoneDrcStatisticsState,
    scheduler_state: &crate::drc::none_drc_scheduler::NoneDrcSchedulerState,
    fifo_state: &NoneDrcOverrideFifoState,
    input: &NoneDrcCallbackInput<'_>,
) -> Result<Option<NoneDrcCallbackTransition>, NoneDrcCallbackError> {
    let Some(clock) = project_none_drc_callback_clock(input.callback_index)
        .map_err(NoneDrcCallbackError::Clock)?
    else {
        return Ok(None);
    };
    let fifo_transition = transition_none_drc_override_fifo(fifo_state, clock.source_index)
        .map_err(NoneDrcCallbackError::Fifo)?;
    let frame_input = NoneDrcFrameInput {
        primary_samples: input.primary_samples,
        secondary_samples: input.secondary_samples,
        source_generation: clock.source_index,
        dialnorm: input.dialnorm,
    };
    let frame = transition_none_drc_frame(statistics_state, scheduler_state, &frame_input)
        .map_err(NoneDrcCallbackError::Frame)?;
    Ok(Some(NoneDrcCallbackTransition {
        clock,
        frame,
        fifo_transition,
        dialnorm: input.dialnorm,
    }))
}
