// pattern: Functional Core
//! Complete selected JOC owner root and noncommitting one-source after proposal.

use crate::{
    joc::{
        coefficient_fitting::CoefficientFitState,
        coefficient_representation::RepresentationRetryProvider,
        dispatch_state::JocDispatchState,
        emdf::metadata_owner::MetadataPayloadBody,
        fit_score::FitScoreProvider,
        source_stage::{JocSourceStageError, stage_joc_source},
    },
    spatial::{QmfReceiverFunctionalProvider, spatial_audio_state::SelectedJocQmfProjection},
};
use sha2::{Digest, Sha256};

const HEADER: &[u8; 8] = b"JOCSEL01";
const CANONICAL_BYTES: usize = 22_790;
const TEMPORAL_BYTES: usize = 1_229;
const GEOMETRY_BYTES: usize = 21_512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedJocStateError {
    Clock,
    Sequence,
    Encoding,
    Source(JocSourceStageErrorKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JocSourceStageErrorKind {
    Fit,
    Lineage,
    Payload,
    Writer,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedJocState {
    fit: CoefficientFitState,
    dispatch: JocDispatchState,
}

impl SelectedJocState {
    pub const fn cold() -> Self {
        Self {
            fit: CoefficientFitState::cold(),
            dispatch: JocDispatchState::cold(),
        }
    }

    pub fn from_staged(
        fit: CoefficientFitState,
        dispatch: JocDispatchState,
    ) -> Result<Self, SelectedJocStateError> {
        let state = Self { fit, dispatch };
        state.validate()?;
        Ok(state)
    }

    pub const fn fit(&self) -> &CoefficientFitState {
        &self.fit
    }

    pub const fn dispatch(&self) -> &JocDispatchState {
        &self.dispatch
    }

    pub const fn generation(&self) -> u64 {
        self.dispatch.successful_dispatches()
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, SelectedJocStateError> {
        self.validate()?;
        let temporal = self.fit.temporal().canonical_payload_bytes();
        let geometry = self.fit.geometry().canonical_payload_bytes();
        if temporal.len() != TEMPORAL_BYTES || geometry.len() != GEOMETRY_BYTES {
            return Err(SelectedJocStateError::Encoding);
        }
        let mut bytes = Vec::with_capacity(CANONICAL_BYTES);
        bytes.extend_from_slice(HEADER);
        bytes.extend_from_slice(&self.dispatch.next_dispatch().value().to_le_bytes());
        bytes.extend_from_slice(&self.dispatch.sequence_counter().to_le_bytes());
        bytes.extend_from_slice(&self.dispatch.successful_dispatches().to_le_bytes());
        bytes.extend_from_slice(&self.dispatch.committed_writer_count().to_le_bytes());
        bytes.extend_from_slice(&self.fit.fit_generation().to_le_bytes());
        bytes.push(u8::from(self.dispatch.is_retired()));
        bytes.extend_from_slice(&temporal);
        bytes.extend_from_slice(&geometry);
        if bytes.len() != CANONICAL_BYTES {
            return Err(SelectedJocStateError::Encoding);
        }
        Ok(bytes)
    }

    pub fn state_sha256(&self) -> Result<[u8; 32], SelectedJocStateError> {
        Ok(Sha256::digest(self.canonical_bytes()?).into())
    }

    pub fn state_sha256_hex(&self) -> Result<String, SelectedJocStateError> {
        Ok(hex(&self.state_sha256()?))
    }

    fn validate(&self) -> Result<(), SelectedJocStateError> {
        let completed = self.dispatch.successful_dispatches();
        if self.dispatch.committed_writer_count() != completed
            || self.fit.fit_generation() != completed
            || self.fit.geometry().generation() != completed
            || self.dispatch.next_dispatch().value()
                != completed
                    .checked_add(1)
                    .ok_or(SelectedJocStateError::Clock)?
        {
            return Err(SelectedJocStateError::Clock);
        }
        if self.dispatch.sequence_counter() != sequence_after_successes(completed)? {
            return Err(SelectedJocStateError::Sequence);
        }
        Ok(())
    }
}

impl Default for SelectedJocState {
    fn default() -> Self {
        Self::cold()
    }
}

#[must_use]
#[derive(Debug)]
pub struct SelectedJocAfterProposal {
    before_generation: u64,
    before_sha256: [u8; 32],
    after: SelectedJocState,
    after_sha256: [u8; 32],
    id14_body: MetadataPayloadBody,
    lineage: crate::joc::coefficient_fitting::FitSourceLineage,
}

impl SelectedJocAfterProposal {
    pub const fn before_generation(&self) -> u64 {
        self.before_generation
    }

    pub const fn before_sha256(&self) -> [u8; 32] {
        self.before_sha256
    }

    pub const fn after(&self) -> &SelectedJocState {
        &self.after
    }

    pub const fn after_sha256(&self) -> [u8; 32] {
        self.after_sha256
    }

    pub const fn id14_body(&self) -> &MetadataPayloadBody {
        &self.id14_body
    }

    pub const fn lineage(&self) -> &crate::joc::coefficient_fitting::FitSourceLineage {
        &self.lineage
    }

    pub fn into_parts(
        self,
    ) -> (
        SelectedJocState,
        MetadataPayloadBody,
        crate::joc::coefficient_fitting::FitSourceLineage,
    ) {
        (self.after, self.id14_body, self.lineage)
    }
}

pub fn stage_selected_joc_source(
    before: &SelectedJocState,
    projection: &SelectedJocQmfProjection,
    qmf_provider: &QmfReceiverFunctionalProvider,
    score_provider: &FitScoreProvider,
    retry_provider: &RepresentationRetryProvider,
) -> Result<SelectedJocAfterProposal, SelectedJocStateError> {
    let before_sha256 = before.state_sha256()?;
    let source = stage_joc_source(
        before.fit(),
        before.dispatch(),
        projection,
        qmf_provider,
        score_provider,
        retry_provider,
    )
    .map_err(|error| map_source_error(&error))?;
    let (fit_after, dispatch_after, id14_body, lineage) = source.into_parts();
    let after = SelectedJocState::from_staged(fit_after, dispatch_after)?;
    if after.generation() != projection.dispatch().value()
        || id14_body.writer_generation != projection.source_index()
    {
        return Err(SelectedJocStateError::Clock);
    }
    let after_sha256 = after.state_sha256()?;
    Ok(SelectedJocAfterProposal {
        before_generation: before.generation(),
        before_sha256,
        after,
        after_sha256,
        id14_body,
        lineage,
    })
}

fn sequence_after_successes(successful_dispatches: u64) -> Result<i64, SelectedJocStateError> {
    match successful_dispatches {
        0 => Ok(-1),
        1..=1_024 => {
            i64::try_from(successful_dispatches - 1).map_err(|_| SelectedJocStateError::Sequence)
        }
        value => {
            i64::try_from((value - 1_025) % 1_023 + 1).map_err(|_| SelectedJocStateError::Sequence)
        }
    }
}

fn map_source_error(error: &JocSourceStageError) -> SelectedJocStateError {
    let kind = match error {
        JocSourceStageError::Fit(_) => JocSourceStageErrorKind::Fit,
        JocSourceStageError::MissingFitLineage => JocSourceStageErrorKind::Lineage,
        JocSourceStageError::Payload(_) | JocSourceStageError::Retry(_) => {
            JocSourceStageErrorKind::Payload
        }
        JocSourceStageError::WriterLineage => JocSourceStageErrorKind::Writer,
    };
    SelectedJocStateError::Source(kind)
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cold_and_nonzero_structural_vectors_match_the_selected_schema() {
        let cases = [
            (
                0,
                -1,
                "a90bc4b2e80fb1e226491e6e05ae35e1c2e50c661c622c19c9e9b40dcf0c1862",
            ),
            (
                1,
                0,
                "6327a22d45a2f82e019a1778b2d8ee7aeffae9bc82eb2009d39ab93d6eb07dc3",
            ),
            (
                2,
                1,
                "c8e6e035662d0a0f202e86ee16b4b1de124a56695b66fb0a8d3f543f5ca63f43",
            ),
            (
                1_024,
                1_023,
                "ec2190538804469d42e2d17cc9a79a58487913ed5bc5710b86e816b5b2032b5c",
            ),
            (
                1_025,
                1,
                "e6b1af9a8fdd64dfb44d6bf5957b5569431def1883170e091dfbffc454af719f",
            ),
            (
                1_026,
                2,
                "ec7ac6121698026688fae26e758cf2ee33074894a3a5d97fcc229c9e2d4974ca",
            ),
        ];
        for (completed, sequence, expected) in cases {
            let state = SelectedJocState::from_staged(
                CoefficientFitState::with_generation_for_test(completed),
                JocDispatchState::with_root_clock_for_test(completed, sequence),
            )
            .unwrap();
            assert_eq!(state.canonical_bytes().unwrap().len(), CANONICAL_BYTES);
            assert_eq!(state.state_sha256_hex().unwrap(), expected);
        }
    }

    #[test]
    fn inconsistent_component_generations_are_rejected() {
        assert_eq!(
            SelectedJocState::from_staged(
                CoefficientFitState::with_generation_for_test(1),
                JocDispatchState::with_root_clock_for_test(2, 1),
            ),
            Err(SelectedJocStateError::Clock)
        );
    }
}
