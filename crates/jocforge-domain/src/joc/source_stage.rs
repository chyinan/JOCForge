// pattern: Functional Core
//! One-source JOC fit, dispatch, and immutable ID14 proposal aggregation.

use crate::{
    frontend::job_configuration::SelectedJobConfiguration,
    joc::{
        coefficient_fitting::{
            CoefficientFitProposal, CoefficientFitState, CoefficientFittingError, FitSourceLineage,
            Flat7xCoefficientRowsProposal, stage_coefficient_fit_from_qmf,
        },
        coefficient_representation::{RepresentationRetryError, RepresentationRetryProvider},
        dispatch_state::{
            JocDispatchError, JocDispatchState, JocPayloadInput, stage_joc_payload,
            stage_joc_payload_for_profile, stage_joc_payload_with_flat7x_rows,
        },
        emdf::metadata_bodies::{
            MetadataBodiesError, MetadataBodiesProposal, assemble_metadata_bodies,
        },
        fit_score::FitScoreProvider,
        id14_body::Id14BodyProposal,
        oamd::reader_target::ReaderTargetUpdateReceipt,
    },
    profile::{CarrierCodecProfile, ResolvedEncodingProfile, SubstreamTopology},
    spatial::{QmfReceiverFunctionalProvider, spatial_audio_state::SelectedJocQmfProjection},
    transaction::CallbackTransactionId,
};

#[derive(Debug)]
pub struct JocSourceProposal {
    fit_after: CoefficientFitState,
    dispatch_after: JocDispatchState,
    id14_body: Id14BodyProposal,
    lineage: FitSourceLineage,
}

impl JocSourceProposal {
    pub const fn fit_after(&self) -> &CoefficientFitState {
        &self.fit_after
    }

    pub const fn dispatch_after(&self) -> &JocDispatchState {
        &self.dispatch_after
    }

    pub const fn id14_body(&self) -> &Id14BodyProposal {
        &self.id14_body
    }

    pub const fn lineage(&self) -> &FitSourceLineage {
        &self.lineage
    }

    /// Joins this same-dispatch ID14 writer result to the F13 body barrier.
    ///
    /// Header fields are already bound by the F05JL source stage. F13 consumes
    /// the complete body and writer generation; it does not accept or recompute
    /// `clip_gain_index` or `sequence_count`.
    pub fn assemble_metadata_bodies(
        &self,
        configuration: &SelectedJobConfiguration,
        projection: &SelectedJocQmfProjection,
        reader_receipt: &ReaderTargetUpdateReceipt,
    ) -> Result<MetadataBodiesProposal, MetadataBodiesError> {
        let lineage = &self.lineage;
        if lineage.programme_id() != projection.programme_id()
            || lineage.attempt() != projection.attempt()
            || lineage.transaction_id() != projection.transaction_id()
            || lineage.source_index() != projection.source_index()
            || lineage.dispatch() != projection.dispatch()
            || lineage.spatial_before_generation() != projection.spatial_before_generation()
            || lineage.spatial_before_digest() != projection.spatial_before_digest()
            || self.dispatch_after.successful_dispatches() != self.fit_after.fit_generation()
        {
            return Err(MetadataBodiesError::Lineage);
        }
        if self.id14_body.body().writer_generation != lineage.source_index() {
            return Err(MetadataBodiesError::WriterGeneration);
        }
        assemble_metadata_bodies(
            configuration,
            projection,
            reader_receipt,
            self.id14_body.clone(),
        )
    }

    pub fn into_parts(
        self,
    ) -> (
        CoefficientFitState,
        JocDispatchState,
        Id14BodyProposal,
        FitSourceLineage,
    ) {
        (
            self.fit_after,
            self.dispatch_after,
            self.id14_body,
            self.lineage,
        )
    }
}

#[derive(Debug)]
pub enum JocSourceStageError {
    Fit(CoefficientFittingError),
    Flat7xCoefficientSourceUnavailable,
    Flat7xCoefficientProfileMismatch,
    Flat7xCoefficientLineageMismatch,
    MissingFitLineage,
    Payload(JocDispatchError),
    Retry(RepresentationRetryError),
    WriterLineage,
}

impl std::fmt::Display for JocSourceStageError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for JocSourceStageError {}

/// Runs the first JOC-owned stage through a local ID14 proposal without a parent transaction ID.
///
/// This unbound entry is useful for isolated F05JL validation. Its provenance has no
/// `CallbackTransactionId`, so it cannot cross the production F13 metadata body barrier.
pub fn stage_joc_source(
    fit_before: &CoefficientFitState,
    dispatch_before: &JocDispatchState,
    projection: &SelectedJocQmfProjection,
    qmf_provider: &QmfReceiverFunctionalProvider,
    score_provider: &FitScoreProvider,
    retry_provider: &RepresentationRetryProvider,
) -> Result<JocSourceProposal, JocSourceStageError> {
    stage_joc_source_inner(
        fit_before,
        dispatch_before,
        projection,
        qmf_provider,
        score_provider,
        retry_provider,
        None,
        None,
    )
}

/// Parent-bound entry which carries the actual GTC callback transaction token into ID14.
///
/// The token remains distinct from F05JL's `CallbackLineageIdentity`. The older
/// `stage_joc_source` entry can produce an isolated body proposal, but that proposal cannot
/// cross the F13 metadata body barrier without this parent-supplied token.
pub fn stage_joc_source_with_transaction(
    fit_before: &CoefficientFitState,
    dispatch_before: &JocDispatchState,
    projection: &SelectedJocQmfProjection,
    producer_transaction_id: CallbackTransactionId,
    qmf_provider: &QmfReceiverFunctionalProvider,
    score_provider: &FitScoreProvider,
    retry_provider: &RepresentationRetryProvider,
) -> Result<JocSourceProposal, JocSourceStageError> {
    stage_joc_source_inner(
        fit_before,
        dispatch_before,
        projection,
        qmf_provider,
        score_provider,
        retry_provider,
        Some(producer_transaction_id),
        None,
    )
}

/// Parent-bound profile-aware entry that carries the immutable layout selection through the
/// existing JOC header writer for one source proposal.
#[expect(
    clippy::too_many_arguments,
    reason = "The owner states, provider receipts, transaction, and profile stay separately typed"
)]
pub fn stage_joc_source_with_transaction_for_profile(
    fit_before: &CoefficientFitState,
    dispatch_before: &JocDispatchState,
    projection: &SelectedJocQmfProjection,
    producer_transaction_id: CallbackTransactionId,
    qmf_provider: &QmfReceiverFunctionalProvider,
    score_provider: &FitScoreProvider,
    retry_provider: &RepresentationRetryProvider,
    profile: ResolvedEncodingProfile,
) -> Result<JocSourceProposal, JocSourceStageError> {
    if source_route_for_profile(profile, false)? != JocSourceRowRoute::SelectedFivePointOne {
        return Err(JocSourceStageError::Flat7xCoefficientProfileMismatch);
    }
    stage_joc_source_inner(
        fit_before,
        dispatch_before,
        projection,
        qmf_provider,
        score_provider,
        retry_provider,
        Some(producer_transaction_id),
        Some(profile),
    )
}

/// Stages one Flat7x JOC source from caller-provided seven-row values and same-source fit state.
#[expect(
    clippy::needless_pass_by_value,
    reason = "The validated row proposal is accepted once for this source stage"
)]
pub fn stage_joc_source_with_flat7x_rows_for_transaction(
    fit_before: &CoefficientFitState,
    dispatch_before: &JocDispatchState,
    projection: &SelectedJocQmfProjection,
    producer_transaction_id: CallbackTransactionId,
    rows_proposal: Flat7xCoefficientRowsProposal<'_>,
    profile: ResolvedEncodingProfile,
) -> Result<JocSourceProposal, JocSourceStageError> {
    if source_route_for_profile(profile, true)? != JocSourceRowRoute::Flat7x {
        return Err(JocSourceStageError::Flat7xCoefficientProfileMismatch);
    }
    if !flat7x_rows_match_source(&rows_proposal, fit_before, projection) {
        return Err(JocSourceStageError::Flat7xCoefficientLineageMismatch);
    }

    let payload_input =
        JocPayloadInput::from_projection_with_transaction(projection, producer_transaction_id)
            .map_err(JocSourceStageError::Payload)?;
    let payload = stage_joc_payload_with_flat7x_rows(
        dispatch_before,
        &rows_proposal,
        &payload_input,
        profile,
    )
    .map_err(JocSourceStageError::Payload)?;
    let (dispatch_after, id14_body) = payload.into_parts().map_err(JocSourceStageError::Payload)?;
    let fit_after = rows_proposal.after().clone();
    let lineage = rows_proposal.source_lineage().clone();
    if id14_body.body().writer_generation != lineage.source_index()
        || dispatch_after.successful_dispatches() != fit_after.fit_generation()
    {
        return Err(JocSourceStageError::WriterLineage);
    }
    Ok(JocSourceProposal {
        fit_after,
        dispatch_after,
        id14_body,
        lineage,
    })
}

#[cfg(test)]
mod tests {
    use super::{JocSourceRowRoute, JocSourceStageError, source_route_for_profile};
    use crate::profile::{EncodingProfile, JocReconstructionLayout, resolve_selected_profile};

    #[test]
    fn row_source_route_keeps_five_point_one_and_fails_closed_without_flat7x_rows() {
        let five_point_one = resolve_selected_profile(&EncodingProfile::selected()).unwrap();
        let flat7x =
            resolve_selected_profile(&EncodingProfile::selected_with_joc_reconstruction_layout(
                JocReconstructionLayout::Flat7x,
            ))
            .unwrap();

        assert!(matches!(
            source_route_for_profile(five_point_one, false),
            Ok(JocSourceRowRoute::SelectedFivePointOne)
        ));
        assert!(matches!(
            source_route_for_profile(flat7x, false),
            Err(JocSourceStageError::Flat7xCoefficientSourceUnavailable)
        ));
        assert!(matches!(
            source_route_for_profile(flat7x, true),
            Ok(JocSourceRowRoute::Flat7x)
        ));
        assert!(matches!(
            source_route_for_profile(five_point_one, true),
            Err(JocSourceStageError::Flat7xCoefficientProfileMismatch)
        ));
    }
}

fn flat7x_rows_match_source(
    rows: &Flat7xCoefficientRowsProposal<'_>,
    fit_before: &CoefficientFitState,
    projection: &SelectedJocQmfProjection,
) -> bool {
    let lineage = rows.source_lineage();
    fit_before.fit_generation() == projection.source_index().value()
        && rows.after().fit_generation() == projection.dispatch().value()
        && lineage.programme_id() == projection.programme_id()
        && lineage.attempt() == projection.attempt()
        && lineage.transaction_id() == projection.transaction_id()
        && lineage.source_index() == projection.source_index()
        && lineage.dispatch() == projection.dispatch()
        && lineage.spatial_before_generation() == projection.spatial_before_generation()
        && lineage.spatial_before_digest() == projection.spatial_before_digest()
}

fn is_flat7x_profile(profile: ResolvedEncodingProfile) -> bool {
    profile.carrier_codec() == CarrierCodecProfile::Flat7xAc3I0DependentEac3D0
        && profile.substream_topology() == SubstreamTopology::Flat7xIndependentI0DependentD0
        && profile.joc_reconstruction_layout().uses_seven_carriers()
        && profile.joc_payload_downmix_index().uses_seven_carriers()
}

fn is_selected_five_point_one_profile(profile: ResolvedEncodingProfile) -> bool {
    profile.carrier_codec() == CarrierCodecProfile::IndependentEac3
        && profile.substream_topology() == SubstreamTopology::SelectedSingleIndependentZeroDependent
        && profile.joc_reconstruction_layout().uses_five_carriers()
        && !profile.joc_payload_downmix_index().uses_seven_carriers()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum JocSourceRowRoute {
    SelectedFivePointOne,
    Flat7x,
}

fn source_route_for_profile(
    profile: ResolvedEncodingProfile,
    has_flat7x_rows: bool,
) -> Result<JocSourceRowRoute, JocSourceStageError> {
    if is_flat7x_profile(profile) {
        return if has_flat7x_rows {
            Ok(JocSourceRowRoute::Flat7x)
        } else {
            Err(JocSourceStageError::Flat7xCoefficientSourceUnavailable)
        };
    }
    if is_selected_five_point_one_profile(profile) {
        return if has_flat7x_rows {
            Err(JocSourceStageError::Flat7xCoefficientProfileMismatch)
        } else {
            Ok(JocSourceRowRoute::SelectedFivePointOne)
        };
    }
    Err(JocSourceStageError::Flat7xCoefficientProfileMismatch)
}

#[expect(
    clippy::too_many_arguments,
    reason = "The common staging kernel keeps its owner and provider inputs explicit"
)]
fn stage_joc_source_inner(
    fit_before: &CoefficientFitState,
    dispatch_before: &JocDispatchState,
    projection: &SelectedJocQmfProjection,
    qmf_provider: &QmfReceiverFunctionalProvider,
    score_provider: &FitScoreProvider,
    retry_provider: &RepresentationRetryProvider,
    producer_transaction_id: Option<CallbackTransactionId>,
    profile: Option<ResolvedEncodingProfile>,
) -> Result<JocSourceProposal, JocSourceStageError> {
    let fit = stage_coefficient_fit_from_qmf(fit_before, projection, qmf_provider, score_provider)
        .map_err(JocSourceStageError::Fit)?;
    let payload_input = match producer_transaction_id {
        Some(transaction_id) => {
            JocPayloadInput::from_projection_with_transaction(projection, transaction_id)
        }
        None => JocPayloadInput::from_projection(projection),
    }
    .map_err(JocSourceStageError::Payload)?;
    assemble_staged_source(
        dispatch_before,
        fit,
        &payload_input,
        retry_provider,
        profile,
    )
}

pub(crate) fn assemble_staged_source(
    dispatch_before: &JocDispatchState,
    fit: CoefficientFitProposal,
    payload_input: &JocPayloadInput,
    retry_provider: &RepresentationRetryProvider,
    profile: Option<ResolvedEncodingProfile>,
) -> Result<JocSourceProposal, JocSourceStageError> {
    let lineage = fit
        .source_lineage()
        .cloned()
        .ok_or(JocSourceStageError::MissingFitLineage)?;
    let payload = match profile {
        Some(profile) => stage_joc_payload_for_profile(
            dispatch_before,
            &fit,
            payload_input,
            retry_provider,
            profile,
        ),
        None => stage_joc_payload(dispatch_before, &fit, payload_input, retry_provider),
    }
    .map_err(JocSourceStageError::Payload)?;
    let fit_after = fit.into_after();
    let (dispatch_after, id14_body) = payload.into_parts().map_err(JocSourceStageError::Payload)?;
    if id14_body.body().writer_generation != lineage.source_index()
        || dispatch_after.successful_dispatches() != fit_after.fit_generation()
    {
        return Err(JocSourceStageError::WriterLineage);
    }
    Ok(JocSourceProposal {
        fit_after,
        dispatch_after,
        id14_body,
        lineage,
    })
}
