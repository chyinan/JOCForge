// pattern: Functional Core
//! Selected JOC owner root and noncommitting one-source after proposal.
//!
//! Use the parent-bound staging entry when the proposal will cross F13; the unbound entry is
//! limited to isolated JOC-stage validation.

use crate::{
    joc::{
        coefficient_fitting::CoefficientFitState,
        coefficient_representation::RepresentationRetryProvider,
        dispatch_state::JocDispatchState,
        fit_score::FitScoreProvider,
        id14_body::Id14BodyProposal,
        source_stage::{JocSourceStageError, stage_joc_source, stage_joc_source_with_transaction},
    },
    spatial::{QmfReceiverFunctionalProvider, spatial_audio_state::SelectedJocQmfProjection},
    transaction::{
        CallbackTransactionId, GtcOwnerId, GtcOwnerReceipt, GtcOwnerReceiptError,
        GtcOwnerStateSnapshot, OwnerStateRef,
    },
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
    OwnerBeforeMismatch,
    ProducerTransactionMismatch,
    GtcReceipt(GtcOwnerReceiptError),
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

    /// Builds an owned GTC snapshot projection from this complete JOCSEL01 state.
    ///
    /// The snapshot owns the canonical bytes so its borrowed `GtcOwnerStateSnapshot` can never
    /// outlive the preimage. It is a state view only; it carries no commit authority.
    pub fn gtc_owner_snapshot(&self) -> Result<SelectedJocGtcSnapshot, SelectedJocStateError> {
        let canonical_bytes = self.canonical_bytes()?;
        let cold_sha256 = Self::cold().state_sha256()?;
        Ok(SelectedJocGtcSnapshot {
            generation: self.generation(),
            cold_sha256,
            canonical_bytes,
        })
    }

    /// Creates the JOC owner's transition receipt for an already staged after-state.
    ///
    /// This does not install either state. The enclosing callback parent remains the only
    /// authority that can consume the receipt and replace owner roots.
    pub fn gtc_owner_receipt_to(
        &self,
        after: &SelectedJocState,
        transaction_id: CallbackTransactionId,
    ) -> Result<GtcOwnerReceipt, SelectedJocStateError> {
        let before_snapshot = self.gtc_owner_snapshot()?;
        let after_snapshot = after.gtc_owner_snapshot()?;
        GtcOwnerReceipt::from_transition(
            before_snapshot
                .as_gtc_snapshot()
                .map_err(SelectedJocStateError::GtcReceipt)?,
            after_snapshot
                .as_gtc_snapshot()
                .map_err(SelectedJocStateError::GtcReceipt)?,
            transaction_id,
        )
        .map_err(SelectedJocStateError::GtcReceipt)
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

/// Owned complete canonical bytes for one selected JOC GTC snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedJocGtcSnapshot {
    generation: u64,
    cold_sha256: [u8; 32],
    canonical_bytes: Vec<u8>,
}

impl SelectedJocGtcSnapshot {
    pub const fn owner_id(&self) -> GtcOwnerId {
        GtcOwnerId::Joc
    }

    pub const fn state_ref(&self) -> OwnerStateRef {
        OwnerStateRef::from_owner(GtcOwnerId::Joc)
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn cold_sha256(&self) -> &[u8; 32] {
        &self.cold_sha256
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn as_gtc_snapshot(&self) -> Result<GtcOwnerStateSnapshot<'_>, GtcOwnerReceiptError> {
        GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::Joc,
            self.generation,
            self.cold_sha256,
            &self.canonical_bytes,
        )
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
    id14_body: Id14BodyProposal,
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

    pub const fn id14_body(&self) -> &Id14BodyProposal {
        &self.id14_body
    }

    pub const fn lineage(&self) -> &crate::joc::coefficient_fitting::FitSourceLineage {
        &self.lineage
    }

    /// Binds the staged JOC after-state to the exact transaction carried by its ID14 producer.
    pub fn gtc_owner_receipt(
        &self,
        before: &SelectedJocState,
        transaction_id: CallbackTransactionId,
    ) -> Result<GtcOwnerReceipt, SelectedJocStateError> {
        if before.generation() != self.before_generation
            || before.state_sha256()? != self.before_sha256
        {
            return Err(SelectedJocStateError::OwnerBeforeMismatch);
        }
        if self.id14_body.provenance().producer_transaction_id() != Some(&transaction_id) {
            return Err(SelectedJocStateError::ProducerTransactionMismatch);
        }
        before.gtc_owner_receipt_to(&self.after, transaction_id)
    }

    pub fn into_parts(
        self,
    ) -> (
        SelectedJocState,
        Id14BodyProposal,
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
        || id14_body.body().writer_generation != projection.source_index()
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

/// Parent-bound variant that retains the actual source transaction token in its ID14 proposal.
pub fn stage_selected_joc_source_with_transaction(
    before: &SelectedJocState,
    projection: &SelectedJocQmfProjection,
    producer_transaction_id: CallbackTransactionId,
    qmf_provider: &QmfReceiverFunctionalProvider,
    score_provider: &FitScoreProvider,
    retry_provider: &RepresentationRetryProvider,
) -> Result<SelectedJocAfterProposal, SelectedJocStateError> {
    let before_sha256 = before.state_sha256()?;
    let source = stage_joc_source_with_transaction(
        before.fit(),
        before.dispatch(),
        projection,
        producer_transaction_id,
        qmf_provider,
        score_provider,
        retry_provider,
    )
    .map_err(|error| map_source_error(&error))?;
    let (fit_after, dispatch_after, id14_body, lineage) = source.into_parts();
    let after = SelectedJocState::from_staged(fit_after, dispatch_after)?;
    if after.generation() != projection.dispatch().value()
        || id14_body.body().writer_generation != projection.source_index()
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
    use crate::{
        joc::{
            coefficient_fitting::FitSourceLineage,
            emdf::metadata_owner::MetadataPayloadBody,
            id14_body::{Id14BodyProposal, Id14ObjectCount},
        },
        spatial::spatial_audio_state::SpatialAudioMachine,
        timeline::{CodecSourceIndex, FrontendCallbackIndex, JocDispatchIndex},
        transaction::{AttemptId, CallbackLineageIdentity, ProgrammeId},
    };

    fn state_with_generation(generation: u64) -> SelectedJocState {
        let sequence = sequence_after_successes(generation).unwrap();
        SelectedJocState::from_staged(
            CoefficientFitState::with_generation_for_test(generation),
            JocDispatchState::with_root_clock_for_test(generation, sequence),
        )
        .unwrap()
    }

    fn id14_body_with_transaction(
        transaction_id: CallbackTransactionId,
    ) -> (Id14BodyProposal, FitSourceLineage) {
        let programme_id = ProgrammeId::new("joc-gtc-test").unwrap();
        let attempt = AttemptId::new(0);
        let source_index = CodecSourceIndex::new(0);
        let dispatch = JocDispatchIndex::new(1);
        let callback_lineage = CallbackLineageIdentity::new(
            programme_id.clone(),
            attempt,
            FrontendCallbackIndex::new(2),
        );
        let spatial_digest =
            SpatialAudioMachine::new_cold(crate::profile::EncodingProfile::selected())
                .unwrap()
                .digest();
        let lineage = FitSourceLineage::for_test(
            programme_id,
            attempt,
            callback_lineage,
            source_index,
            dispatch,
            0,
            spatial_digest,
        );
        // This fixture exercises transaction-token transport only. It is not parsed as an ID14
        // body and makes no claim about metadata wire production.
        let body = MetadataPayloadBody {
            writer_generation: source_index,
            bytes: vec![0x5a],
        };
        let proposal = Id14BodyProposal::from_successful_writer(
            body,
            lineage.clone(),
            Some(transaction_id),
            dispatch,
            0,
            Id14ObjectCount::from_selected_slots(1).unwrap(),
        )
        .unwrap();
        (proposal, lineage)
    }

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

    #[test]
    fn gtc_snapshot_uses_the_complete_jocsel01_owner_state() {
        let state = SelectedJocState::cold();
        let reference = state.gtc_owner_snapshot().unwrap();
        let snapshot = reference.as_gtc_snapshot().unwrap();

        assert_eq!(reference.owner_id(), GtcOwnerId::Joc);
        assert_eq!(reference.state_ref().as_str(), "owners.joc");
        assert_eq!(reference.generation(), 0);
        assert_eq!(
            reference.canonical_bytes(),
            state.canonical_bytes().unwrap()
        );
        assert_eq!(snapshot.owner_id(), GtcOwnerId::Joc);
        assert_eq!(snapshot.state_ref().as_str(), "owners.joc");
        assert_eq!(snapshot.generation(), state.generation());
        assert_eq!(snapshot.canonical_bytes(), reference.canonical_bytes());
        assert_eq!(snapshot.cold_sha256(), reference.cold_sha256().to_owned());
        assert_eq!(
            hex(&snapshot.state_sha256()),
            "a90bc4b2e80fb1e226491e6e05ae35e1c2e50c661c622c19c9e9b40dcf0c1862"
        );
    }

    #[test]
    fn unchanged_gtc_transition_preserves_bytes_generation_and_zero_commit_count() {
        let state = SelectedJocState::cold();
        let transaction_id = CallbackTransactionId::new("joc-callback-g0").unwrap();

        let receipt = state
            .gtc_owner_receipt_to(&state, transaction_id.clone())
            .unwrap();

        assert_eq!(receipt.owner_id(), GtcOwnerId::Joc);
        assert_eq!(receipt.before_generation(), 0);
        assert_eq!(receipt.after_generation(), 0);
        assert_eq!(receipt.before_sha256(), receipt.after_sha256());
        assert!(!receipt.changed());
        assert_eq!(receipt.commit_count(), 0);
        assert_eq!(receipt.transaction_id(), &transaction_id);
    }

    #[test]
    fn staged_gtc_receipt_binds_actual_after_state_and_exact_id14_transaction() {
        let before = SelectedJocState::cold();
        let after = state_with_generation(1);
        let transaction_id = CallbackTransactionId::new("joc-source-s0").unwrap();
        let (id14_body, lineage) = id14_body_with_transaction(transaction_id.clone());
        let proposal = SelectedJocAfterProposal {
            before_generation: before.generation(),
            before_sha256: before.state_sha256().unwrap(),
            after_sha256: after.state_sha256().unwrap(),
            after,
            id14_body,
            lineage,
        };

        let receipt = proposal
            .gtc_owner_receipt(&before, transaction_id.clone())
            .unwrap();

        assert_eq!(receipt.owner_id(), GtcOwnerId::Joc);
        assert_eq!(receipt.before_generation(), 0);
        assert_eq!(receipt.after_generation(), 1);
        assert_ne!(receipt.before_sha256(), receipt.after_sha256());
        assert!(receipt.changed());
        assert_eq!(receipt.commit_count(), 1);
        assert_eq!(receipt.transaction_id(), &transaction_id);
    }

    #[test]
    fn staged_gtc_receipt_rejects_stale_before_state_and_wrong_transaction() {
        let before = SelectedJocState::cold();
        let after = state_with_generation(1);
        let transaction_id = CallbackTransactionId::new("joc-source-s0").unwrap();
        let (id14_body, lineage) = id14_body_with_transaction(transaction_id.clone());
        let proposal = SelectedJocAfterProposal {
            before_generation: before.generation(),
            before_sha256: before.state_sha256().unwrap(),
            after_sha256: after.state_sha256().unwrap(),
            after,
            id14_body,
            lineage,
        };

        assert_eq!(
            proposal.gtc_owner_receipt(&state_with_generation(1), transaction_id.clone()),
            Err(SelectedJocStateError::OwnerBeforeMismatch)
        );
        assert_eq!(
            proposal.gtc_owner_receipt(
                &before,
                CallbackTransactionId::new("wrong-callback").unwrap()
            ),
            Err(SelectedJocStateError::ProducerTransactionMismatch)
        );
    }
}
