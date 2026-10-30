// pattern: Functional Core
//! Canonical GTC state references and noncommitting Meter callback receipts.

use sha2::{Digest, Sha256};

use crate::{
    meter::{
        lfe_delay_ring::MeterFrontendCallbackInput,
        sample_block_queue::{METER_CALLBACK_SAMPLES, MeterCallbackIndex},
        standalone_root::{
            MeterRootCallbackProgress, StandaloneMeterCallbackReceipt, StandaloneMeterRoot,
            StandaloneMeterRootError,
        },
    },
    transaction::{
        AttemptId, CallbackTransactionId, GtcOwnerId, GtcOwnerReceipt, GtcOwnerReceiptError,
        GtcOwnerStateSnapshot, ProgrammeId,
    },
};

/// The GTC member for the standalone Meter owner.
const METER_OWNER_ID: &str = "meter";
const METER_STATE_REF: &str = "owners.meter";

/// Errors while deriving a canonical state reference from the complete Meter root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterOwnerStateReferenceError {
    Root(StandaloneMeterRootError),
    CanonicalDigestMismatch,
}

/// Complete immutable canonical bytes and hashes for one Meter root.
///
/// The GTC generation is the number of committed Meter callbacks, read from the
/// root's callback clock. The state reference never includes a receipt or any
/// terminal measurement value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterOwnerStateReference {
    generation: MeterCallbackIndex,
    state_sha256: [u8; 32],
    cold_sha256: [u8; 32],
    canonical_bytes: Vec<u8>,
}

impl MeterOwnerStateReference {
    /// Serializes the root's complete V4 canonical member and rehashes those exact bytes.
    pub fn from_root(root: &StandaloneMeterRoot) -> Result<Self, MeterOwnerStateReferenceError> {
        let canonical_bytes = root
            .canonical_preimage()
            .map_err(MeterOwnerStateReferenceError::Root)?;
        let state_sha256: [u8; 32] = Sha256::digest(&canonical_bytes).into();
        let root_sha256 = root
            .canonical_sha256()
            .map_err(MeterOwnerStateReferenceError::Root)?;
        if state_sha256 != root_sha256 {
            return Err(MeterOwnerStateReferenceError::CanonicalDigestMismatch);
        }
        let cold_sha256 = root
            .cold_canonical_sha256()
            .map_err(MeterOwnerStateReferenceError::Root)?;

        Ok(Self {
            generation: root.meter_callback_index(),
            state_sha256,
            cold_sha256,
            canonical_bytes,
        })
    }

    pub const fn owner_id(&self) -> &'static str {
        METER_OWNER_ID
    }

    pub const fn state_ref(&self) -> &'static str {
        METER_STATE_REF
    }

    pub const fn generation(&self) -> MeterCallbackIndex {
        self.generation
    }

    pub const fn state_sha256(&self) -> &[u8; 32] {
        &self.state_sha256
    }

    pub const fn cold_sha256(&self) -> &[u8; 32] {
        &self.cold_sha256
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Adapts the full owner bytes to the registered GTC `owners.meter` slot.
    pub fn gtc_snapshot(&self) -> Result<GtcOwnerStateSnapshot<'_>, GtcOwnerReceiptError> {
        GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::Meter,
            self.generation.value(),
            self.cold_sha256,
            &self.canonical_bytes,
        )
    }
}

/// Parent callback context captured before staging a Meter callback.
///
/// The lower-level Meter proposal intentionally has no transaction token. The
/// parent supplies it here before staging; this wrapper retains that exact token
/// and the programme, attempt, and Meter callback coordinates through receipt
/// projection. The parent remains responsible for using the same token for every
/// owner in its callback transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterCallbackGtcBinding {
    transaction_id: CallbackTransactionId,
    programme_id: ProgrammeId,
    attempt: AttemptId,
    callback_index: MeterCallbackIndex,
}

impl MeterCallbackGtcBinding {
    pub const fn new(
        transaction_id: CallbackTransactionId,
        programme_id: ProgrammeId,
        attempt: AttemptId,
        callback_index: MeterCallbackIndex,
    ) -> Self {
        Self {
            transaction_id,
            programme_id,
            attempt,
            callback_index,
        }
    }

    pub fn transaction_id(&self) -> &CallbackTransactionId {
        &self.transaction_id
    }

    pub fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub const fn attempt(&self) -> AttemptId {
        self.attempt
    }

    pub const fn callback_index(&self) -> MeterCallbackIndex {
        self.callback_index
    }
}

/// Errors while binding a staged callback proposal to a GTC owner receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterOwnerGtcError {
    StateReference(MeterOwnerStateReferenceError),
    CallbackStage(StandaloneMeterRootError),
    GtcReceipt(GtcOwnerReceiptError),
    ParentContextMismatch,
    CallbackReceiptMismatch,
    CallbackClockMismatch,
    SourceCursorMismatch,
    SourceReadIndexMismatch,
}

/// A Meter callback proposal with its exact before-state reference and parent context.
///
/// Creating this value stages from the supplied committed root and retains that
/// root's complete canonical reference. It exposes no installation or commit API.
#[must_use]
pub struct MeterCallbackGtcProposal {
    before_reference: MeterOwnerStateReference,
    after_root: StandaloneMeterRoot,
    callback_receipt: StandaloneMeterCallbackReceipt,
    binding: MeterCallbackGtcBinding,
}

/// Validated noncommitting handoff from one Meter callback to its programme parent.
///
/// The after-root and its owner receipt travel together. This value grants no
/// installation or programme commit authority.
#[must_use]
pub struct MeterCallbackGtcHandoff {
    after_root: StandaloneMeterRoot,
    owner_receipt: GtcOwnerReceipt,
}

impl MeterCallbackGtcHandoff {
    pub const fn after_root(&self) -> &StandaloneMeterRoot {
        &self.after_root
    }

    pub const fn owner_receipt(&self) -> &GtcOwnerReceipt {
        &self.owner_receipt
    }

    pub fn into_parts(self) -> (StandaloneMeterRoot, GtcOwnerReceipt) {
        (self.after_root, self.owner_receipt)
    }
}

impl MeterCallbackGtcProposal {
    /// Stages one Meter callback under a parent-issued binding and retains the exact before root.
    pub fn stage(
        before: &StandaloneMeterRoot,
        input: MeterFrontendCallbackInput,
        progress: MeterRootCallbackProgress,
        binding: MeterCallbackGtcBinding,
    ) -> Result<Self, MeterOwnerGtcError> {
        validate_parent_context(before, &binding)?;
        let before_reference = MeterOwnerStateReference::from_root(before)
            .map_err(MeterOwnerGtcError::StateReference)?;
        let proposal = before
            .prepare_callback(input, progress)
            .map_err(MeterOwnerGtcError::CallbackStage)?;
        let (after_root, callback_receipt) = proposal.into_parts();

        validate_callback_roots(before, &after_root, &callback_receipt, &binding)?;

        Ok(Self {
            before_reference,
            after_root,
            callback_receipt,
            binding,
        })
    }

    pub const fn after_root(&self) -> &StandaloneMeterRoot {
        &self.after_root
    }

    pub const fn callback_receipt(&self) -> &StandaloneMeterCallbackReceipt {
        &self.callback_receipt
    }

    pub const fn before_reference(&self) -> &MeterOwnerStateReference {
        &self.before_reference
    }

    pub const fn binding(&self) -> &MeterCallbackGtcBinding {
        &self.binding
    }

    /// Projects the exact staged root transition into one noncommitting GTC owner receipt.
    pub fn gtc_receipt(&self) -> Result<GtcOwnerReceipt, MeterOwnerGtcError> {
        let after_reference = MeterOwnerStateReference::from_root(&self.after_root)
            .map_err(MeterOwnerGtcError::StateReference)?;
        if after_reference.cold_sha256 != self.before_reference.cold_sha256 {
            return Err(MeterOwnerGtcError::GtcReceipt(
                GtcOwnerReceiptError::ColdRootMismatch,
            ));
        }
        let before_snapshot = self
            .before_reference
            .gtc_snapshot()
            .map_err(MeterOwnerGtcError::GtcReceipt)?;
        let after_snapshot = after_reference
            .gtc_snapshot()
            .map_err(MeterOwnerGtcError::GtcReceipt)?;
        GtcOwnerReceipt::from_transition(
            before_snapshot,
            after_snapshot,
            self.binding.transaction_id.clone(),
        )
        .map_err(MeterOwnerGtcError::GtcReceipt)
    }

    /// Validates the GTC owner receipt before transferring the after-root to the parent.
    ///
    /// Consuming the proposal prevents a caller from separately extracting an unchecked
    /// after-root from this typed handoff. The returned after-root remains only a candidate.
    pub fn into_gtc_handoff(self) -> Result<MeterCallbackGtcHandoff, MeterOwnerGtcError> {
        let owner_receipt = self.gtc_receipt()?;
        Ok(MeterCallbackGtcHandoff {
            after_root: self.after_root,
            owner_receipt,
        })
    }
}

fn validate_parent_context(
    root: &StandaloneMeterRoot,
    binding: &MeterCallbackGtcBinding,
) -> Result<(), MeterOwnerGtcError> {
    if binding.programme_id != *root.identity().programme_id()
        || binding.attempt != root.identity().attempt()
        || binding.callback_index != root.meter_callback_index()
    {
        return Err(MeterOwnerGtcError::ParentContextMismatch);
    }
    Ok(())
}

fn validate_callback_roots(
    before: &StandaloneMeterRoot,
    after: &StandaloneMeterRoot,
    receipt: &StandaloneMeterCallbackReceipt,
    binding: &MeterCallbackGtcBinding,
) -> Result<(), MeterOwnerGtcError> {
    if before.identity() != after.identity()
        || receipt.identity() != before.identity()
        || binding.programme_id != *after.identity().programme_id()
        || binding.attempt != after.identity().attempt()
    {
        return Err(MeterOwnerGtcError::ParentContextMismatch);
    }

    let before_callback = before.meter_callback_index();
    let expected_after_callback = before_callback
        .value()
        .checked_add(1)
        .map(MeterCallbackIndex::new)
        .ok_or(MeterOwnerGtcError::CallbackClockMismatch)?;
    if receipt.callback_index() != before_callback
        || receipt.next_callback_index() != expected_after_callback
        || after.meter_callback_index() != expected_after_callback
        || binding.callback_index != before_callback
    {
        return Err(MeterOwnerGtcError::CallbackReceiptMismatch);
    }

    validate_source_progress(before, after, before_callback)
}

fn validate_source_progress(
    before: &StandaloneMeterRoot,
    after: &StandaloneMeterRoot,
    callback_index: MeterCallbackIndex,
) -> Result<(), MeterOwnerGtcError> {
    let callback = callback_index.value();
    let normal_callbacks = before.normal_callback_count().value();
    let source_samples = before.source_samples().value();
    let callback_samples = u64::try_from(METER_CALLBACK_SAMPLES)
        .map_err(|_| MeterOwnerGtcError::SourceCursorMismatch)?;
    let expected_cursor_before = if callback < normal_callbacks {
        callback
            .checked_mul(callback_samples)
            .map(|samples| samples.min(source_samples))
            .ok_or(MeterOwnerGtcError::SourceCursorMismatch)?
    } else {
        source_samples
    };
    let expected_read_before = callback.min(normal_callbacks);
    if before.source_cursor().value() != expected_cursor_before {
        return Err(MeterOwnerGtcError::SourceCursorMismatch);
    }
    if before.source_read_index().value() != expected_read_before {
        return Err(MeterOwnerGtcError::SourceReadIndexMismatch);
    }

    let (expected_cursor_after, expected_read_after) = if callback < normal_callbacks {
        let next_normal = callback
            .checked_add(1)
            .ok_or(MeterOwnerGtcError::CallbackClockMismatch)?;
        let cursor = next_normal
            .checked_mul(callback_samples)
            .map(|samples| samples.min(source_samples))
            .ok_or(MeterOwnerGtcError::SourceCursorMismatch)?;
        (cursor, next_normal)
    } else {
        (source_samples, normal_callbacks)
    };
    if after.source_cursor().value() != expected_cursor_after {
        return Err(MeterOwnerGtcError::SourceCursorMismatch);
    }
    if after.source_read_index().value() != expected_read_after {
        return Err(MeterOwnerGtcError::SourceReadIndexMismatch);
    }
    if after.source_samples() != before.source_samples()
        || after.normal_callback_count() != before.normal_callback_count()
    {
        return Err(MeterOwnerGtcError::SourceCursorMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meter::{
        k_matrix::{KMatrixMode, KMatrixProfile, generate_k_matrix},
        object_renderer::ObjectRenderCallbackGeneration,
        sample_block_queue::{MeterCallbackPhase, MeterSampleCount, MeterSourceSampleCount},
        shared_gain_delay::SharedGainDelayParameters,
        source_frame::MeterSourcePosition,
        standalone_root::{
            MeterKFilterExponentState, MeterProviderIdentity, MeterProviderSet, MeterSourceOwnerId,
            MeterSourceReadIndex,
        },
    };

    const SOURCE_SAMPLES: u64 = 6_522_906;
    const KERNEL_Q15: [u16; 32] = [
        20, 177, 491, 958, 1_573, 2_331, 3_224, 4_244, 5_381, 6_624, 7_961, 9_379, 10_864, 12_403,
        13_980, 15_580, 17_188, 18_788, 20_365, 21_904, 23_389, 24_807, 26_144, 27_387, 28_524,
        29_544, 30_437, 31_195, 31_810, 32_277, 32_591, 32_748,
    ];

    fn selected_regular_controls() -> Vec<Vec<u32>> {
        const ZERO_PROGRAMME: &[u8] =
            include_bytes!("../../tests/fixtures/post_render_meter/zero_programme.json");
        let fixture: serde_json::Value = serde_json::from_slice(ZERO_PROGRAMME).unwrap();
        let source_count =
            usize::try_from(fixture["oar_packet"]["source_count"].as_u64().unwrap()).unwrap();
        let direct = fixture["oar_packet"]["direct_control_words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>();
        let object = fixture["oar_packet"]["object_control_words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>();
        let mut rows = vec![object; source_count];
        rows[0] = direct;
        rows.into_iter().skip(1).collect()
    }

    fn cold_root(owner_key: &str) -> StandaloneMeterRoot {
        let identity = crate::meter::standalone_root::StandaloneMeterIdentity::new(
            ProgrammeId::new("meter-owner-gtc-test").unwrap(),
            AttemptId::new(0),
            [0x11; 32],
            [0x22; 32],
        );
        let first = MeterProviderIdentity::new("selected-provider", [0x33; 32]).unwrap();
        let provider_set = MeterProviderSet::new([0x44; 32], vec![first]).unwrap();
        let gain = SharedGainDelayParameters::selected(&KERNEL_Q15).unwrap();
        let matrix = generate_k_matrix(KMatrixProfile {
            sample_rate_hz: 48_000,
            mode: KMatrixMode::Modern,
            stage_count: 2,
        })
        .unwrap();
        StandaloneMeterRoot::cold_selected(
            identity,
            MeterSourceSampleCount::new(SOURCE_SAMPLES).unwrap(),
            MeterSourceOwnerId::new(owner_key).unwrap(),
            provider_set,
            MeterKFilterExponentState::new([0; 6]),
            gain,
            matrix,
        )
        .unwrap()
    }

    fn callback_inputs(
        root: &StandaloneMeterRoot,
        transaction_id: &str,
    ) -> (
        MeterFrontendCallbackInput,
        MeterRootCallbackProgress,
        MeterCallbackGtcBinding,
    ) {
        callback_inputs_with_k_exponents(
            root,
            transaction_id,
            *root.k_filter_exponents_i32_by_channel(),
        )
    }

    fn callback_inputs_with_k_exponents(
        root: &StandaloneMeterRoot,
        transaction_id: &str,
        k_filter_exponents_after: [i32; 6],
    ) -> (
        MeterFrontendCallbackInput,
        MeterRootCallbackProgress,
        MeterCallbackGtcBinding,
    ) {
        let callback_index = root.meter_callback_index();
        let source_cursor_before = root.source_cursor();
        let read_index_before = root.source_read_index();
        let input = MeterFrontendCallbackInput::new(
            callback_index,
            ObjectRenderCallbackGeneration::try_new(callback_index.value() + 1).unwrap(),
            MeterCallbackPhase::NormalSource,
            vec![0; METER_CALLBACK_SAMPLES],
            vec![vec![0; METER_CALLBACK_SAMPLES]; 15],
            selected_regular_controls(),
        );
        let progress = MeterRootCallbackProgress::new(
            callback_index,
            source_cursor_before,
            MeterSourcePosition::new(source_cursor_before.value() + 1_536),
            read_index_before,
            MeterSourceReadIndex::new(read_index_before.value() + 1),
            MeterSampleCount::new(1_536),
            MeterKFilterExponentState::new(k_filter_exponents_after),
        );
        let binding = MeterCallbackGtcBinding::new(
            CallbackTransactionId::new(transaction_id).unwrap(),
            root.identity().programme_id().clone(),
            root.identity().attempt(),
            callback_index,
        );
        (input, progress, binding)
    }

    #[test]
    fn meter_state_reference_uses_full_canonical_v4_bytes_and_recomputed_hashes() {
        let root = cold_root("meter-state-reference-test");
        let reference = MeterOwnerStateReference::from_root(&root).unwrap();
        let canonical_bytes = root.canonical_preimage().unwrap();
        let state_sha256: [u8; 32] = Sha256::digest(&canonical_bytes).into();

        assert_eq!(reference.owner_id(), "meter");
        assert_eq!(reference.state_ref(), "owners.meter");
        assert_eq!(reference.generation(), MeterCallbackIndex::new(0));
        assert_eq!(reference.canonical_bytes(), canonical_bytes);
        assert_eq!(reference.state_sha256(), &state_sha256);
        assert_eq!(
            reference.cold_sha256(),
            &root.cold_canonical_sha256().unwrap()
        );

        let snapshot = reference.gtc_snapshot().unwrap();
        assert_eq!(snapshot.owner_id(), GtcOwnerId::Meter);
        assert_eq!(snapshot.state_ref().as_str(), "owners.meter");
        assert_eq!(snapshot.generation(), 0);
        assert_eq!(snapshot.canonical_bytes(), reference.canonical_bytes());
        assert_eq!(snapshot.state_sha256(), state_sha256);
    }

    #[test]
    fn meter_callback_gtc_proposal_binds_exact_before_after_roots_and_parent_context() {
        let root = cold_root("meter-callback-source-test");
        let before = MeterOwnerStateReference::from_root(&root).unwrap();
        let (input, progress, binding) = callback_inputs(&root, "meter-parent-callback-0");
        let proposal =
            MeterCallbackGtcProposal::stage(&root, input, progress, binding.clone()).unwrap();
        let after = MeterOwnerStateReference::from_root(proposal.after_root()).unwrap();
        let receipt = proposal.gtc_receipt().unwrap();

        assert_eq!(receipt.owner_id(), GtcOwnerId::Meter);
        assert_eq!(receipt.before_generation(), 0);
        assert_eq!(receipt.before_sha256(), *before.state_sha256());
        assert_eq!(receipt.after_generation(), 1);
        assert_eq!(receipt.after_sha256(), *after.state_sha256());
        assert!(receipt.changed());
        assert_eq!(receipt.commit_count(), 1);
        assert_eq!(receipt.transaction_id(), binding.transaction_id());
        assert_eq!(root.meter_callback_index(), MeterCallbackIndex::new(0));
        assert_eq!(
            proposal.after_root().meter_callback_index(),
            MeterCallbackIndex::new(1)
        );
    }

    #[test]
    fn consuming_handoff_validates_receipt_and_preserves_the_before_root() {
        let before_root = cold_root("meter-ownership-handoff-test");
        let before_reference = MeterOwnerStateReference::from_root(&before_root).unwrap();
        let before_bytes = before_reference.canonical_bytes().to_vec();
        let (input, progress, binding) = callback_inputs(&before_root, "meter-parent-handoff-0");
        let proposal =
            MeterCallbackGtcProposal::stage(&before_root, input, progress, binding).unwrap();

        let handoff = proposal.into_gtc_handoff().unwrap();
        let (after_root, owner_receipt) = handoff.into_parts();
        let after_reference = MeterOwnerStateReference::from_root(&after_root).unwrap();

        assert_eq!(
            before_root.meter_callback_index(),
            MeterCallbackIndex::new(0)
        );
        assert_eq!(before_root.canonical_preimage().unwrap(), before_bytes);
        assert_eq!(
            after_root.meter_callback_index(),
            MeterCallbackIndex::new(1)
        );
        assert_eq!(owner_receipt.owner_id(), GtcOwnerId::Meter);
        assert_eq!(
            owner_receipt.before_sha256(),
            *before_reference.state_sha256()
        );
        assert_eq!(
            owner_receipt.after_sha256(),
            *after_reference.state_sha256()
        );
        assert_eq!(
            owner_receipt.transaction_id().as_str(),
            "meter-parent-handoff-0"
        );
        assert!(owner_receipt.changed());
        assert_eq!(owner_receipt.commit_count(), 1);
    }

    #[test]
    fn parent_k_exponent_progress_does_not_change_the_cold_root_identity() {
        let root = cold_root("meter-stable-cold-digest-test");
        let cold_sha256 = root.cold_canonical_sha256().unwrap();
        let (input, progress, binding) =
            callback_inputs_with_k_exponents(&root, "meter-parent-callback-k-update", [1; 6]);
        let proposal = MeterCallbackGtcProposal::stage(&root, input, progress, binding).unwrap();
        let after_reference = MeterOwnerStateReference::from_root(proposal.after_root()).unwrap();
        let receipt = proposal.gtc_receipt().unwrap();

        assert_eq!(after_reference.cold_sha256(), &cold_sha256);
        assert_ne!(
            after_reference.state_sha256(),
            MeterOwnerStateReference::from_root(&root)
                .unwrap()
                .state_sha256()
        );
        assert!(receipt.changed());
        assert_eq!(receipt.commit_count(), 1);
    }

    #[test]
    fn parent_programme_attempt_and_meter_callback_coordinates_must_match_before_staging() {
        let root = cold_root("meter-parent-context-test");
        let (input, progress, _) = callback_inputs(&root, "meter-parent-callback-0");
        let mismatched_binding = MeterCallbackGtcBinding::new(
            CallbackTransactionId::new("meter-parent-callback-foreign").unwrap(),
            ProgrammeId::new("other-programme").unwrap(),
            AttemptId::new(1),
            MeterCallbackIndex::new(1),
        );

        assert!(matches!(
            MeterCallbackGtcProposal::stage(&root, input, progress, mismatched_binding),
            Err(MeterOwnerGtcError::ParentContextMismatch)
        ));
        assert_eq!(root.meter_callback_index(), MeterCallbackIndex::new(0));
    }

    #[test]
    fn source_read_progress_must_match_the_meter_callback_clock() {
        let root = cold_root("meter-source-progress-test");
        let (input, _, binding) = callback_inputs(&root, "meter-parent-callback-0");
        let bad_progress = MeterRootCallbackProgress::new(
            MeterCallbackIndex::new(0),
            MeterSourcePosition::new(0),
            MeterSourcePosition::new(1_536),
            MeterSourceReadIndex::new(0),
            MeterSourceReadIndex::new(2),
            MeterSampleCount::new(1_536),
            MeterKFilterExponentState::new([0; 6]),
        );

        assert!(matches!(
            MeterCallbackGtcProposal::stage(&root, input, bad_progress, binding),
            Err(MeterOwnerGtcError::CallbackStage(
                StandaloneMeterRootError::SourceCursorProgress
            ))
        ));
        assert_eq!(root.meter_callback_index(), MeterCallbackIndex::new(0));
    }

    #[test]
    fn a_parent_binding_cannot_be_retargeted_after_the_callback_is_staged() {
        let root = cold_root("meter-parent-token-test");
        let (input, progress, binding) = callback_inputs(&root, "meter-parent-callback-0");
        let proposal = MeterCallbackGtcProposal::stage(&root, input, progress, binding).unwrap();
        let receipt = proposal.gtc_receipt().unwrap();

        assert_eq!(receipt.transaction_id().as_str(), "meter-parent-callback-0");
    }
}
