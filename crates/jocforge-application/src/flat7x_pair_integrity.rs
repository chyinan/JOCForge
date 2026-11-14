// pattern: Functional Core
//! Joins a source-bound I0/D0 frame pair to F13/F14K and one role-aware F12 transition.

use jocforge_domain::{
    integrity::{
        keyless_checksum::KeylessIntegrityProvider,
        programme_owner::{
            Flat7xI0D0F12PairInput, Flat7xIntegrityPairRatePlan, IntegrityError,
            IntegrityGtcFlat7xF12PairProposal, IntegrityIdentity, IntegrityProgrammeOwner,
        },
    },
    joc::emdf::metadata_owner::MetadataSourceProposal,
    profile::Flat7xI0D0TargetPlan,
    timeline::{PublicationDisposition, publication_for_source},
    transaction::CallbackLineageIdentity,
};

pub use jocforge_domain::integrity::programme_owner::Flat7xI0D0F12PairReceipts;

use crate::{
    flat7x_pair_source::{
        Flat7xPairD0PcmFrameCandidate, Flat7xPairI0FrameCandidate, d0_identity_matches_role_map,
    },
    integrity_source_bridge::{F14kPayloadOutcome, F14kSourceCandidate},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xPairF12StageError {
    PairSourceIdentityMismatch,
    SourceClockMismatch,
    F13ReceiptMismatch,
    F13ContainerMissing,
    F14kReceiptMismatch,
    F14kContainerMissing,
    D0ContainerCursorMissing,
    Integrity(IntegrityError),
}

/// Inputs to one noncommitting I0/D0 F12 pair receipt stage.
pub struct Flat7xPairF12StageInput<'a> {
    pub integrity_before: &'a IntegrityProgrammeOwner,
    pub i0_frame: &'a Flat7xPairI0FrameCandidate,
    pub d0_frame: &'a Flat7xPairD0PcmFrameCandidate,
    pub f13_metadata: &'a MetadataSourceProposal,
    pub f14k_source: &'a F14kSourceCandidate,
    pub keyless_integrity: &'a KeylessIntegrityProvider,
}

/// Stages the paired F12 receipts from matching source-bound frames and their actual F13/F14K
/// proposals. I0 carries absent F13 payload evidence; D0 carries the one protected EMDF container.
pub fn stage_flat7x_pair_f12_integrity(
    input: Flat7xPairF12StageInput<'_>,
) -> Result<IntegrityGtcFlat7xF12PairProposal, Flat7xPairF12StageError> {
    let target = Flat7xI0D0TargetPlan::approved_contract();
    let i0_role_map = input.i0_frame.source_role_map_receipt();
    let d0_role_map = input.d0_frame.source_role_map_receipt();
    let d0_identity = input.d0_frame.d0_source_identity();
    if input.i0_frame.target() != target
        || i0_role_map != d0_role_map
        || i0_role_map.target() != target
        || input.i0_frame.source_identity() != i0_role_map.shared_analysis_identity()
        || !d0_identity_matches_role_map(d0_identity, d0_role_map)
    {
        return Err(Flat7xPairF12StageError::PairSourceIdentityMismatch);
    }

    let source_index = d0_identity.source_index();
    let dispatch = source_index
        .dispatch()
        .map_err(|_| Flat7xPairF12StageError::SourceClockMismatch)?;
    let au_index = match publication_for_source(source_index)
        .map_err(|_| Flat7xPairF12StageError::SourceClockMismatch)?
    {
        PublicationDisposition::Published(consumption) => consumption.au,
        PublicationDisposition::Suppressed { .. } => {
            return Err(Flat7xPairF12StageError::SourceClockMismatch);
        }
    };
    let expected_lineage = CallbackLineageIdentity::new(
        d0_identity.programme_id().clone(),
        d0_identity.attempt_id(),
        d0_identity.callback_index(),
    );
    let metadata = input.f13_metadata;
    let f13 = metadata.receipt();
    if f13.programme_id() != d0_identity.programme_id()
        || f13.attempt() != d0_identity.attempt_id()
        || f13.transaction_id() != &expected_lineage
        || f13.consumer_transaction_id() != Some(d0_identity.source_transaction_id())
        || metadata.consumer_transaction_id() != Some(d0_identity.source_transaction_id())
        || f13.source_index() != source_index
        || f13.dispatch() != dispatch
        || f13.au_index() != Some(au_index)
    {
        return Err(Flat7xPairF12StageError::F13ReceiptMismatch);
    }
    let zero_slot_container = metadata
        .emdf()
        .ok_or(Flat7xPairF12StageError::F13ContainerMissing)?;
    let protected = input
        .f14k_source
        .protected_container()
        .ok_or(Flat7xPairF12StageError::F14kContainerMissing)?;
    let parsed_container = metadata
        .parsed_container()
        .ok_or(Flat7xPairF12StageError::F14kContainerMissing)?;
    let protected_length = match input.f14k_source.outcome() {
        F14kPayloadOutcome::ProtectedEmdf {
            container_bytes, ..
        } => container_bytes,
        F14kPayloadOutcome::AbsentPublishedPayloadV1 => {
            return Err(Flat7xPairF12StageError::F14kContainerMissing);
        }
    };
    if input.f14k_source.source_index() != source_index
        || input.f14k_source.dispatch() != dispatch
        || input.f14k_source.au_index() != Some(au_index)
        || protected_length != protected.bytes().len()
        || parsed_container.bytes() != protected.bytes()
        || !(1..=511).contains(&protected_length)
    {
        return Err(Flat7xPairF12StageError::F14kReceiptMismatch);
    }
    let d0_emdf_container_start = input
        .d0_frame
        .emdf_container_start()
        .ok_or(Flat7xPairF12StageError::D0ContainerCursorMissing)?;
    let integrity_identity = IntegrityIdentity {
        programme_id: d0_identity.programme_id().as_str().to_owned(),
        attempt: d0_identity.attempt_id().value(),
        transaction_id: d0_identity.source_transaction_id().as_str().to_owned(),
        source_index,
        dispatch,
        au_index: Some(au_index),
    };
    let callback_identity = expected_lineage;
    input
        .integrity_before
        .stage_flat7x_i0_d0_f12_pair_for_callback(
            Flat7xI0D0F12PairInput {
                identity: integrity_identity,
                rate_plan: Flat7xIntegrityPairRatePlan::flat7x_i0_640_d0_1024(),
                i0_frame: input.i0_frame.i0_frame().frame_bytes(),
                d0_frame: input.d0_frame.frame_candidate().frame().frame_bytes(),
                d0_emdf_container_start,
                d0_zero_slot_container: zero_slot_container,
                d0_protected_container: protected,
            },
            input.keyless_integrity,
            d0_identity.programme_id(),
            d0_identity.attempt_id(),
            d0_identity.source_transaction_id(),
            &callback_identity,
        )
        .map_err(Flat7xPairF12StageError::Integrity)
}
