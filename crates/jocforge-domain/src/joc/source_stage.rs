// pattern: Functional Core
//! One-source JOC fit, dispatch, and immutable ID14 proposal aggregation.

use crate::{
    joc::{
        coefficient_fitting::{
            CoefficientFitProposal, CoefficientFitState, CoefficientFittingError, FitSourceLineage,
            stage_coefficient_fit_from_qmf,
        },
        coefficient_representation::{RepresentationRetryError, RepresentationRetryProvider},
        dispatch_state::{JocDispatchError, JocDispatchState, JocPayloadInput, stage_joc_payload},
        emdf::metadata_owner::MetadataPayloadBody,
        fit_score::FitScoreProvider,
    },
    spatial::{QmfReceiverFunctionalProvider, spatial_audio_state::SelectedJocQmfProjection},
};

#[derive(Debug)]
pub struct JocSourceProposal {
    fit_after: CoefficientFitState,
    dispatch_after: JocDispatchState,
    id14_body: MetadataPayloadBody,
    lineage: FitSourceLineage,
}

impl JocSourceProposal {
    pub const fn fit_after(&self) -> &CoefficientFitState {
        &self.fit_after
    }

    pub const fn dispatch_after(&self) -> &JocDispatchState {
        &self.dispatch_after
    }

    pub const fn id14_body(&self) -> &MetadataPayloadBody {
        &self.id14_body
    }

    pub const fn lineage(&self) -> &FitSourceLineage {
        &self.lineage
    }

    pub fn into_parts(
        self,
    ) -> (
        CoefficientFitState,
        JocDispatchState,
        MetadataPayloadBody,
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

/// Runs the first JOC-owned stage through a complete ID14 proposal. All
/// returned state remains staged for the enclosing programme source parent.
pub fn stage_joc_source(
    fit_before: &CoefficientFitState,
    dispatch_before: &JocDispatchState,
    projection: &SelectedJocQmfProjection,
    qmf_provider: &QmfReceiverFunctionalProvider,
    score_provider: &FitScoreProvider,
    retry_provider: &RepresentationRetryProvider,
) -> Result<JocSourceProposal, JocSourceStageError> {
    let fit = stage_coefficient_fit_from_qmf(fit_before, projection, qmf_provider, score_provider)
        .map_err(JocSourceStageError::Fit)?;
    let payload_input =
        JocPayloadInput::from_projection(projection).map_err(JocSourceStageError::Payload)?;
    assemble_staged_source(dispatch_before, fit, &payload_input, retry_provider)
}

pub(crate) fn assemble_staged_source(
    dispatch_before: &JocDispatchState,
    fit: CoefficientFitProposal,
    payload_input: &JocPayloadInput,
    retry_provider: &RepresentationRetryProvider,
) -> Result<JocSourceProposal, JocSourceStageError> {
    let lineage = fit
        .source_lineage()
        .cloned()
        .ok_or(JocSourceStageError::MissingFitLineage)?;
    let payload = stage_joc_payload(dispatch_before, &fit, payload_input, retry_provider)
        .map_err(JocSourceStageError::Payload)?;
    let fit_after = fit.into_after();
    let (dispatch_after, id14_body) = payload.into_parts().map_err(JocSourceStageError::Payload)?;
    if id14_body.writer_generation != lineage.source_index()
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
