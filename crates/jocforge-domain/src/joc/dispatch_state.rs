// pattern: Functional Core
//! Selected JOC dispatch, sequence, and completed ID14 body staging.

use crate::{
    joc::{
        coefficient_fitting::{
            CoefficientFitProposal, FitSourceLineage, Flat7xCoefficientRowsProposal,
            SevenCarrierCoefficientFitProposal,
        },
        coefficient_representation::{
            RepresentationRetryError, RepresentationRetryProvider, RepresentationWriteResult,
        },
        id14_body::Id14BodyProposal,
    },
    profile::{
        CarrierCodecProfile, JocPayloadDownmixIndex, JocReconstructionLayout,
        ResolvedEncodingProfile, SubstreamTopology,
    },
    spatial::{
        source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
        spatial_audio_state::SelectedJocQmfProjection,
    },
    timeline::{CodecSourceIndex, JocDispatchIndex},
    transaction::{CallbackLineageIdentity, CallbackTransactionId},
};

const SELECTED_CAPACITY_BYTES: u16 = 372;
const FLAT7X_CAPACITY_BYTES: u16 = 511;

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

    pub fn next_sequence_count(&self) -> Result<u16, JocDispatchError> {
        match self.sequence_counter {
            -1 => Ok(0),
            0..=1_023 => {
                u16::try_from(self.sequence_counter).map_err(|_| JocDispatchError::Sequence)
            }
            _ => Err(JocDispatchError::Sequence),
        }
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
    producer_transaction_id: Option<CallbackTransactionId>,
    source_index: CodecSourceIndex,
    dispatch: JocDispatchIndex,
    clip_gain_index: u8,
    descriptor_target_gain_word: u32,
}

impl JocPayloadInput {
    pub fn from_projection(
        projection: &SelectedJocQmfProjection,
    ) -> Result<Self, JocDispatchError> {
        Self::from_projection_with_optional_transaction(projection, None)
    }

    pub fn from_projection_with_transaction(
        projection: &SelectedJocQmfProjection,
        producer_transaction_id: CallbackTransactionId,
    ) -> Result<Self, JocDispatchError> {
        Self::from_projection_with_optional_transaction(projection, Some(producer_transaction_id))
    }

    pub const fn clip_gain_index(&self) -> u8 {
        self.clip_gain_index
    }

    fn from_projection_with_optional_transaction(
        projection: &SelectedJocQmfProjection,
        producer_transaction_id: Option<CallbackTransactionId>,
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
            producer_transaction_id,
            source_index: projection.source_index(),
            dispatch: projection.dispatch(),
            clip_gain_index: projection.descriptor_selected_index(),
            descriptor_target_gain_word: projection.descriptor_target_gain_word(),
        })
    }

    #[cfg(test)]
    pub(crate) const fn clip_gain_index_for_test(&self) -> u8 {
        self.clip_gain_index
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
            producer_transaction_id: None,
            source_index,
            dispatch: source_index.dispatch().expect("bounded test source"),
            clip_gain_index,
            descriptor_target_gain_word,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test_with_transaction(
        programme_id: SourceFlagsProgrammeId,
        attempt: SourceFlagsAttempt,
        transaction_id: CallbackLineageIdentity,
        producer_transaction_id: CallbackTransactionId,
        source_index: CodecSourceIndex,
        clip_gain_index: u8,
        descriptor_target_gain_word: u32,
    ) -> Self {
        let mut input = Self::for_test(
            programme_id,
            attempt,
            transaction_id,
            source_index,
            clip_gain_index,
            descriptor_target_gain_word,
        );
        input.producer_transaction_id = Some(producer_transaction_id);
        input
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
    lineage: FitSourceLineage,
    producer_transaction_id: Option<CallbackTransactionId>,
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

    pub fn into_parts(self) -> Result<(JocDispatchState, Id14BodyProposal), JocDispatchError> {
        let body = self
            .representation
            .into_id14_body_proposal(
                self.writer_generation,
                self.dispatch,
                self.sequence_count,
                self.producer_transaction_id,
                self.lineage,
            )
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
    Flat7xCoefficientSourceUnavailable,
    Flat7xProfileMismatch,
    CoefficientRowSourceMismatch,
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
    stage_joc_payload_inner(before, Some(fit), None, input, Some(retry_provider), None)
}

pub fn stage_joc_payload_for_profile(
    before: &JocDispatchState,
    fit: &CoefficientFitProposal,
    input: &JocPayloadInput,
    retry_provider: &RepresentationRetryProvider,
    profile: ResolvedEncodingProfile,
) -> Result<JocPayloadProposal, JocDispatchError> {
    if is_flat7x_profile(profile) {
        return Err(JocDispatchError::Flat7xCoefficientSourceUnavailable);
    }
    if !is_selected_five_point_one_profile(profile) {
        return Err(JocDispatchError::Flat7xProfileMismatch);
    }
    stage_joc_payload_inner(
        before,
        Some(fit),
        None,
        input,
        Some(retry_provider),
        Some(profile),
    )
}

pub fn stage_joc_payload_with_flat7x_rows(
    before: &JocDispatchState,
    rows: &Flat7xCoefficientRowsProposal<'_>,
    input: &JocPayloadInput,
    profile: ResolvedEncodingProfile,
) -> Result<JocPayloadProposal, JocDispatchError> {
    if !is_flat7x_profile(profile) {
        return Err(JocDispatchError::Flat7xProfileMismatch);
    }
    stage_joc_payload_inner(before, None, Some(rows), input, None, Some(profile))
}

/// Writes one C=7 fit proposal through the selected Flat7x ID14 representation path.
pub fn stage_joc_payload_with_seven_carrier_fit(
    before: &JocDispatchState,
    fit: &SevenCarrierCoefficientFitProposal,
    input: &JocPayloadInput,
    retry_provider: &RepresentationRetryProvider,
    profile: ResolvedEncodingProfile,
) -> Result<JocPayloadProposal, JocDispatchError> {
    if !is_flat7x_profile(profile) {
        return Err(JocDispatchError::Flat7xProfileMismatch);
    }
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
    let producer_transaction_id = input
        .producer_transaction_id
        .as_ref()
        .ok_or(JocDispatchError::Lineage)?;
    validate_fit_lineage(Some(fit.target_lineage()), input)?;
    let source_identity = fit.qmf_source_identity();
    if fit.after().fit_generation() != input.dispatch.value()
        || source_identity.callback_lineage() != &input.transaction_id
        || source_identity.source_index() != input.source_index
        || source_identity.source_transaction_id() != producer_transaction_id
        || fit.rows().payload_downmix_index() != JocPayloadDownmixIndex::SevenChannel
        || fit.rows().row_layout()
            != crate::joc::coefficient_representation::JocCoefficientRowLayout::SevenChannel
    {
        return Err(JocDispatchError::Lineage);
    }

    let sequence_count = before.next_sequence_count()?;
    let representation = fit
        .write_payload_with_resolved_profile(
            profile,
            retry_provider,
            input.clip_gain_index,
            sequence_count,
            FLAT7X_CAPACITY_BYTES,
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
        lineage: fit.target_lineage().clone(),
        producer_transaction_id: Some(producer_transaction_id.clone()),
        representation,
    })
}

fn stage_joc_payload_inner(
    before: &JocDispatchState,
    fit: Option<&CoefficientFitProposal>,
    flat7x_rows: Option<&Flat7xCoefficientRowsProposal<'_>>,
    input: &JocPayloadInput,
    retry_provider: Option<&RepresentationRetryProvider>,
    profile: Option<ResolvedEncodingProfile>,
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
    if fit.is_some() == flat7x_rows.is_some() {
        return Err(JocDispatchError::CoefficientRowSourceMismatch);
    }
    let source_lineage = match (fit, flat7x_rows) {
        (Some(fit), None) => fit.source_lineage().ok_or(JocDispatchError::Lineage)?,
        (None, Some(rows)) => rows.source_lineage(),
        _ => return Err(JocDispatchError::CoefficientRowSourceMismatch),
    };
    validate_fit_lineage(Some(source_lineage), input)?;
    let lineage = source_lineage.clone();
    let sequence_count = before.next_sequence_count()?;
    let representation = if let Some(rows) = flat7x_rows {
        let profile = profile.ok_or(JocDispatchError::Flat7xProfileMismatch)?;
        if !is_flat7x_profile(profile) {
            return Err(JocDispatchError::Flat7xProfileMismatch);
        }
        let outcome = rows
            .rows()
            .write_payload(profile)
            .map_err(|error| JocDispatchError::Retry(RepresentationRetryError::Writer(error)))?;
        RepresentationWriteResult::from_single_outcome(outcome)
    } else {
        let fit = fit.ok_or(JocDispatchError::CoefficientRowSourceMismatch)?;
        let retry_provider =
            retry_provider.ok_or(JocDispatchError::CoefficientRowSourceMismatch)?;
        let result = match profile {
            Some(profile) if is_selected_five_point_one_profile(profile) => fit
                .write_payload_with_resolved_profile(
                    profile,
                    retry_provider,
                    input.clip_gain_index,
                    sequence_count,
                    SELECTED_CAPACITY_BYTES,
                ),
            Some(profile) if is_flat7x_profile(profile) => {
                return Err(JocDispatchError::Flat7xCoefficientSourceUnavailable);
            }
            Some(_) => return Err(JocDispatchError::Flat7xProfileMismatch),
            None => fit.write_payload_with_retry(
                retry_provider,
                input.clip_gain_index,
                sequence_count,
                SELECTED_CAPACITY_BYTES,
            ),
        };
        result.map_err(JocDispatchError::Retry)?
    };
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
        lineage,
        producer_transaction_id: input.producer_transaction_id.clone(),
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

fn is_flat7x_profile(profile: ResolvedEncodingProfile) -> bool {
    profile.carrier_codec() == CarrierCodecProfile::Flat7xAc3I0DependentEac3D0
        && profile.substream_topology() == SubstreamTopology::Flat7xIndependentI0DependentD0
        && profile.joc_reconstruction_layout() == JocReconstructionLayout::Flat7x
        && profile.joc_payload_downmix_index() == JocPayloadDownmixIndex::SevenChannel
}

fn is_selected_five_point_one_profile(profile: ResolvedEncodingProfile) -> bool {
    profile.carrier_codec() == CarrierCodecProfile::IndependentEac3
        && profile.substream_topology() == SubstreamTopology::SelectedSingleIndependentZeroDependent
        && profile.joc_reconstruction_layout()
            == JocReconstructionLayout::ExistingSelectedFivePointOne
        && profile.joc_payload_downmix_index() == JocPayloadDownmixIndex::FiveChannel
}

#[cfg(test)]
mod tests {
    use super::{
        JocDispatchError, JocDispatchState, JocPayloadInput, stage_joc_payload_with_flat7x_rows,
        validate_fit_lineage,
    };
    use crate::{
        joc::{
            coefficient_fitting::{
                CoefficientFitState, FitSourceLineage, Flat7xCoefficientRowsProposal,
                Flat7xCoefficientRowsProviderOutput,
            },
            coefficient_payload::{
                CoefficientPayloadInput, CoefficientPayloadObject, CoefficientPayloadPoint,
                Flat7xCoefficientRows,
            },
        },
        profile::{EncodingProfile, JocReconstructionLayout, resolve_selected_profile},
        spatial::SpatialAudioMachine,
        timeline::{CodecSourceIndex, FrontendCallbackIndex, JocDispatchIndex},
        transaction::{AttemptId, CallbackLineageIdentity, ProgrammeId},
    };

    #[test]
    fn shared_dispatch_provenance_accepts_same_source_and_rejects_a_different_generation() {
        let programme_id = ProgrammeId::new("flat7x-dispatch-lineage-test").unwrap();
        let attempt = AttemptId::new(1);
        let source_index = CodecSourceIndex::new(0);
        let transaction_id = CallbackLineageIdentity::new(
            programme_id.clone(),
            attempt,
            FrontendCallbackIndex::new(2),
        );
        let lineage = FitSourceLineage::for_test(
            programme_id.clone(),
            attempt,
            transaction_id.clone(),
            source_index,
            source_index.dispatch().unwrap(),
            0,
            SpatialAudioMachine::new_cold(EncodingProfile::selected())
                .unwrap()
                .digest(),
        );
        let input = JocPayloadInput::for_test(
            programme_id.clone(),
            attempt,
            transaction_id.clone(),
            source_index,
            128,
            0x3f80_0000,
        );

        assert!(validate_fit_lineage(Some(&lineage), &input).is_ok());
        let next_source = CodecSourceIndex::new(1);
        let mismatched_input = JocPayloadInput::for_test(
            programme_id,
            attempt,
            transaction_id,
            next_source,
            128,
            0x3f80_0000,
        );
        assert!(matches!(
            validate_fit_lineage(Some(&lineage), &mismatched_input),
            Err(JocDispatchError::Lineage)
        ));
    }

    #[test]
    fn flat7x_rows_cross_the_shared_dispatch_boundary_with_same_source_provenance() {
        // SYNTHETIC/TEST_ONLY coefficient rows exercise routing and provenance, not audio fitting.
        let point = CoefficientPayloadPoint {
            channel_rows: (0..7).map(|_| vec![0.25]).collect(),
            selected_channels: Vec::new(),
        };
        let payload_input = CoefficientPayloadInput {
            downmix_index: 1,
            extension_config: 0,
            clip_gain_index: 128,
            sequence_count: 0,
            time_slot_limit: 24,
            capacity_bytes: 372,
            objects: (0..15)
                .map(|_| CoefficientPayloadObject {
                    present: true,
                    band_enum: 0,
                    mode: 1,
                    points: vec![point.clone()],
                    offsets: None,
                })
                .collect(),
        };
        let rows = Flat7xCoefficientRows::new(&payload_input).unwrap();
        let programme_id = ProgrammeId::new("flat7x-dispatch-provenance-test").unwrap();
        let attempt = AttemptId::new(2);
        let source_index = CodecSourceIndex::new(0);
        let dispatch = JocDispatchIndex::new(1);
        let transaction_id = CallbackLineageIdentity::new(
            programme_id.clone(),
            attempt,
            FrontendCallbackIndex::new(2),
        );
        let lineage = FitSourceLineage::for_test(
            programme_id.clone(),
            attempt,
            transaction_id.clone(),
            source_index,
            dispatch,
            0,
            SpatialAudioMachine::new_cold(EncodingProfile::selected())
                .unwrap()
                .digest(),
        );
        let after = CoefficientFitState::with_generation_for_test(dispatch.value());
        let rows_proposal = Flat7xCoefficientRowsProposal::from_source_lineage(
            &Flat7xCoefficientRowsProviderOutput::synthetic_for_test(rows, lineage.clone()),
            lineage.clone(),
            after,
        )
        .unwrap();
        let joc_input = JocPayloadInput::for_test(
            programme_id,
            attempt,
            transaction_id,
            source_index,
            128,
            0x3f80_0000,
        );
        let profile =
            resolve_selected_profile(&EncodingProfile::selected_with_joc_reconstruction_layout(
                JocReconstructionLayout::Flat7x,
            ))
            .unwrap();

        let proposal = stage_joc_payload_with_flat7x_rows(
            &JocDispatchState::cold(),
            &rows_proposal,
            &joc_input,
            profile,
        )
        .unwrap();
        let (after_dispatch, id14_body) = proposal.into_parts().unwrap();
        assert_eq!(after_dispatch.successful_dispatches(), 1);
        assert_eq!(id14_body.provenance().lineage(), &lineage);
        assert_eq!(id14_body.provenance().producer_generation(), source_index);
        assert_eq!(id14_body.provenance().dispatch(), dispatch);
        assert_eq!(id14_body.provenance().object_count().value(), 15);
    }
}
