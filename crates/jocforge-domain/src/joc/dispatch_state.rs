// pattern: Functional Core
//! Selected JOC dispatch, sequence, and completed ID14 body staging.

use crate::{
    joc::{
        coefficient_fitting::{CoefficientFitProposal, FitSourceLineage},
        coefficient_representation::{
            RepresentationRetryError, RepresentationRetryProvider, RepresentationWriteResult,
        },
        emdf::metadata_owner::MetadataPayloadBody,
    },
    spatial::{
        source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
        spatial_audio_state::SelectedJocQmfProjection,
    },
    timeline::{CodecSourceIndex, JocDispatchIndex},
    transaction::CallbackLineageIdentity,
};

const SELECTED_CAPACITY_BYTES: u16 = 372;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JocDispatchState {
    next_dispatch: JocDispatchIndex,
    sequence_counter: i64,
    successful_dispatches: u64,
    committed_writer_count: u64,
    retired: bool,
}

impl JocDispatchState {
    pub const fn cold() -> Self {
        Self {
            next_dispatch: JocDispatchIndex::new(1),
            sequence_counter: -1,
            successful_dispatches: 0,
            committed_writer_count: 0,
            retired: false,
        }
    }

    pub const fn next_dispatch(&self) -> JocDispatchIndex {
        self.next_dispatch
    }

    pub const fn sequence_counter(&self) -> i64 {
        self.sequence_counter
    }

    pub const fn successful_dispatches(&self) -> u64 {
        self.successful_dispatches
    }

    pub const fn committed_writer_count(&self) -> u64 {
        self.committed_writer_count
    }

    pub const fn is_retired(&self) -> bool {
        self.retired
    }

    #[cfg(test)]
    pub(crate) const fn with_clock_for_test(
        next_dispatch: JocDispatchIndex,
        sequence_counter: i64,
    ) -> Self {
        Self {
            next_dispatch,
            sequence_counter,
            successful_dispatches: 0,
            committed_writer_count: 0,
            retired: false,
        }
    }

    #[cfg(test)]
    pub(crate) const fn with_root_clock_for_test(
        successful_dispatches: u64,
        sequence_counter: i64,
    ) -> Self {
        Self {
            next_dispatch: JocDispatchIndex::new(successful_dispatches + 1),
            sequence_counter,
            successful_dispatches,
            committed_writer_count: successful_dispatches,
            retired: false,
        }
    }
}

impl Default for JocDispatchState {
    fn default() -> Self {
        Self::cold()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JocPayloadInput {
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: CallbackLineageIdentity,
    source_index: CodecSourceIndex,
    dispatch: JocDispatchIndex,
    clip_gain_index: u8,
    descriptor_target_gain_word: u32,
}

impl JocPayloadInput {
    pub fn from_projection(
        projection: &SelectedJocQmfProjection,
    ) -> Result<Self, JocDispatchError> {
        let expected_dispatch = projection
            .source_index()
            .dispatch()
            .map_err(|_| JocDispatchError::Clock)?;
        let expected_callback = projection
            .source_index()
            .value()
            .checked_add(2)
            .ok_or(JocDispatchError::Clock)?;
        if projection.dispatch() != expected_dispatch
            || u64::from(projection.callback_index().value()) != expected_callback
            || !(128..=255).contains(&projection.descriptor_selected_index())
            || !f32::from_bits(projection.descriptor_target_gain_word()).is_finite()
        {
            return Err(JocDispatchError::Lineage);
        }
        Ok(Self {
            programme_id: projection.programme_id().clone(),
            attempt: projection.attempt(),
            transaction_id: projection.transaction_id().clone(),
            source_index: projection.source_index(),
            dispatch: projection.dispatch(),
            clip_gain_index: projection.descriptor_selected_index(),
            descriptor_target_gain_word: projection.descriptor_target_gain_word(),
        })
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        programme_id: SourceFlagsProgrammeId,
        attempt: SourceFlagsAttempt,
        transaction_id: CallbackLineageIdentity,
        source_index: CodecSourceIndex,
        clip_gain_index: u8,
        descriptor_target_gain_word: u32,
    ) -> Self {
        Self {
            programme_id,
            attempt,
            transaction_id,
            source_index,
            dispatch: source_index.dispatch().expect("bounded test source"),
            clip_gain_index,
            descriptor_target_gain_word,
        }
    }
}

#[derive(Debug)]
pub struct JocPayloadProposal {
    after: JocDispatchState,
    dispatch: JocDispatchIndex,
    writer_generation: CodecSourceIndex,
    sequence_count: u16,
    clip_gain_index: u8,
    descriptor_target_gain_word: u32,
    representation: RepresentationWriteResult,
}

impl JocPayloadProposal {
    pub const fn after(&self) -> &JocDispatchState {
        &self.after
    }

    pub const fn dispatch(&self) -> JocDispatchIndex {
        self.dispatch
    }

    pub const fn writer_generation(&self) -> CodecSourceIndex {
        self.writer_generation
    }

    pub const fn sequence_count(&self) -> u16 {
        self.sequence_count
    }

    pub const fn clip_gain_index(&self) -> u8 {
        self.clip_gain_index
    }

    pub const fn descriptor_target_gain_word(&self) -> u32 {
        self.descriptor_target_gain_word
    }

    pub const fn representation(&self) -> &RepresentationWriteResult {
        &self.representation
    }

    pub fn into_parts(self) -> Result<(JocDispatchState, MetadataPayloadBody), JocDispatchError> {
        let body = self
            .representation
            .into_metadata_body(self.writer_generation)
            .ok_or(JocDispatchError::PayloadAbsent)?;
        Ok((self.after, body))
    }
}

#[derive(Debug)]
pub enum JocDispatchError {
    Retired,
    Clock,
    Lineage,
    Sequence,
    Retry(RepresentationRetryError),
    PayloadAbsent,
}

impl std::fmt::Display for JocDispatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for JocDispatchError {}

/// Writes one complete ID14 proposal with one shared sequence across every
/// representation attempt. The returned after state remains uncommitted.
pub fn stage_joc_payload(
    before: &JocDispatchState,
    fit: &CoefficientFitProposal,
    input: &JocPayloadInput,
    retry_provider: &RepresentationRetryProvider,
) -> Result<JocPayloadProposal, JocDispatchError> {
    if before.retired {
        return Err(JocDispatchError::Retired);
    }
    if before.next_dispatch != input.dispatch
        || input
            .source_index
            .dispatch()
            .map_err(|_| JocDispatchError::Clock)?
            != input.dispatch
    {
        return Err(JocDispatchError::Clock);
    }
    validate_fit_lineage(fit.source_lineage(), input)?;
    let sequence_count = if before.sequence_counter < 0 {
        0
    } else {
        u16::try_from(before.sequence_counter).map_err(|_| JocDispatchError::Sequence)?
    };
    if sequence_count > 1_023 {
        return Err(JocDispatchError::Sequence);
    }
    let representation = fit
        .write_payload_with_retry(
            retry_provider,
            input.clip_gain_index,
            sequence_count,
            SELECTED_CAPACITY_BYTES,
        )
        .map_err(JocDispatchError::Retry)?;
    let mut after = before.clone();
    after.next_dispatch = JocDispatchIndex::new(
        input
            .dispatch
            .value()
            .checked_add(1)
            .ok_or(JocDispatchError::Clock)?,
    );
    after.sequence_counter = if before.sequence_counter == 1_023 {
        1
    } else {
        before
            .sequence_counter
            .checked_add(1)
            .ok_or(JocDispatchError::Sequence)?
    };
    after.successful_dispatches = before
        .successful_dispatches
        .checked_add(1)
        .ok_or(JocDispatchError::Clock)?;
    after.committed_writer_count = before
        .committed_writer_count
        .checked_add(1)
        .ok_or(JocDispatchError::Clock)?;
    Ok(JocPayloadProposal {
        after,
        dispatch: input.dispatch,
        writer_generation: input.source_index,
        sequence_count,
        clip_gain_index: input.clip_gain_index,
        descriptor_target_gain_word: input.descriptor_target_gain_word,
        representation,
    })
}

fn validate_fit_lineage(
    lineage: Option<&FitSourceLineage>,
    input: &JocPayloadInput,
) -> Result<(), JocDispatchError> {
    let lineage = lineage.ok_or(JocDispatchError::Lineage)?;
    if lineage.programme_id() != &input.programme_id
        || lineage.attempt() != input.attempt
        || lineage.transaction_id() != &input.transaction_id
        || lineage.source_index() != input.source_index
        || lineage.dispatch() != input.dispatch
    {
        return Err(JocDispatchError::Lineage);
    }
    Ok(())
}
