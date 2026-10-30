// pattern: Functional Core
//! Noncommitting codec-owner state for the selected source-frame transition.

use crate::{
    analysis::structure_refresh::{F07_BLOCKS_PER_FRAME, F07_FULLBAND_LANES},
    integrity::crc16::eac3_crc16,
    publication::{OutputSourceAfterProposal, OutputSourceCommitReceipt},
    timeline::{CodecSourceIndex, FrontendCallbackIndex, PublishedAuIndex, source_for_callback},
    transaction::{
        AttemptId, CallbackTransactionId, GtcOwnerId, GtcOwnerReceipt, GtcOwnerReceiptError,
        GtcOwnerStateSnapshot,
    },
};
use sha2::{Digest, Sha256};

use super::selected_frame_trial::SelectedMainFrameCandidate;

const FRAME_BYTES: usize = 4_096;
const SUPPRESSED_SOURCE_COUNT: u64 = 2;
const CODEC_OWNER_ID: &str = "codec";
const CODEC_STATE_REF: &str = "owners.codec";

/// Six current short-flag blocks in codec lane order `[L,C,R,Ls,Rs]`.
pub type CodecShortFlags = [[bool; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME];

/// Codec-owner generation, distinct from the source clock owned by GTC.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodecOwnerGeneration(u64);

impl CodecOwnerGeneration {
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Persistent selected codec state. Source order and transform histories belong to other owners.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodecOwnerRoot {
    generation: CodecOwnerGeneration,
    previous_short: [bool; F07_FULLBAND_LANES],
}

impl CodecOwnerRoot {
    /// Creates the selected codec cold state.
    pub const fn cold() -> Self {
        Self {
            generation: CodecOwnerGeneration(0),
            previous_short: [false; F07_FULLBAND_LANES],
        }
    }

    /// Reconstructs this owner from its cold schema during a programme-origin reset.
    pub fn reset(&mut self) {
        *self = Self::cold();
    }

    pub const fn generation(&self) -> CodecOwnerGeneration {
        self.generation
    }

    pub const fn previous_short(&self) -> &[bool; F07_FULLBAND_LANES] {
        &self.previous_short
    }

    /// Serializes this complete owner value as the selected canonical JSON preimage.
    ///
    /// Clock and attempt identities are owned by the enclosing GTC source transaction and are
    /// deliberately not part of the persistent Codec owner value.
    pub fn canonical_state_bytes(&self) -> Vec<u8> {
        let [left, center, right, left_surround, right_surround] =
            self.previous_short.map(u8::from);
        format!(
            "{{\"codec\":{{\"generation\":{}}},\"previous_short\":{{\"i1_words\":[{left},{center},{right},{left_surround},{right_surround}]}}}}",
            self.generation.value()
        )
        .into_bytes()
    }

    /// Returns immutable bytes and hashes for this root in the selected GTC owner slot.
    pub fn state_reference(&self) -> CodecOwnerStateReference {
        let canonical_bytes = self.canonical_state_bytes();
        let state_sha256 = Sha256::digest(&canonical_bytes).into();
        let cold_sha256 = Sha256::digest(Self::cold().canonical_state_bytes()).into();
        CodecOwnerStateReference {
            generation: self.generation,
            state_sha256,
            cold_sha256,
            canonical_bytes,
        }
    }

    /// Builds the unchanged codec owner receipt for a CALLBACK with no codec source stage.
    ///
    /// The caller supplies the shared parent transaction ID. Callback clocks 0 and 1 do not
    /// advance this source-owned root, so the snapshot is used for both sides of the receipt.
    pub fn gtc_unchanged_callback_receipt(
        &self,
        callback_index: FrontendCallbackIndex,
        transaction_id: CallbackTransactionId,
    ) -> Result<GtcOwnerReceipt, CodecOwnerStageError> {
        if source_for_callback(callback_index).is_some() {
            return Err(CodecOwnerStageError::SourceStageRequired);
        }
        let reference = self.state_reference();
        let snapshot = reference
            .gtc_snapshot()
            .map_err(CodecOwnerStageError::GtcReceipt)?;
        GtcOwnerReceipt::from_transition(snapshot, snapshot, transaction_id)
            .map_err(CodecOwnerStageError::GtcReceipt)
    }

    /// Stages the selected F12 state after a complete candidate has passed CRC and, for
    /// published sources, the matching F15 durable append receipt has been admitted.
    ///
    /// This returns a proposal only. The parent owns installation with all other source roots.
    pub fn stage_source(
        &self,
        accepted_frame: &AcceptedCodecFrame,
        current_short_flags: &CodecShortFlags,
        output_after: Option<&OutputSourceAfterProposal>,
    ) -> Result<CodecOwnerProposal, CodecOwnerStageError> {
        validate_publication(accepted_frame, output_after)?;
        let next_generation = self
            .generation
            .0
            .checked_add(1)
            .ok_or(CodecOwnerStageError::GenerationOverflow)?;
        let after_state = Self {
            generation: CodecOwnerGeneration(next_generation),
            previous_short: current_short_flags[F07_BLOCKS_PER_FRAME - 1],
        };
        let before_state_sha256 = self.state_reference().state_sha256;

        Ok(CodecOwnerProposal {
            source_index: accepted_frame.source_index,
            frame_sha256: accepted_frame.frame_sha256,
            before_generation: self.generation,
            before_state_sha256,
            after_state,
        })
    }
}

/// Complete immutable state-ref material for one selected GTC Codec owner snapshot.
///
/// This is an owner snapshot only; it is not a GTC OwnerReceipt and carries no transaction ID.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodecOwnerStateReference {
    generation: CodecOwnerGeneration,
    state_sha256: [u8; 32],
    cold_sha256: [u8; 32],
    canonical_bytes: Vec<u8>,
}

impl CodecOwnerStateReference {
    pub const fn owner_id(&self) -> &'static str {
        CODEC_OWNER_ID
    }

    pub const fn generation(&self) -> CodecOwnerGeneration {
        self.generation
    }

    pub const fn state_ref(&self) -> &'static str {
        CODEC_STATE_REF
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

    /// Adapts the existing canonical codec bytes to the registered GTC owner slot.
    pub fn gtc_snapshot(&self) -> Result<GtcOwnerStateSnapshot<'_>, GtcOwnerReceiptError> {
        GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::Codec,
            self.generation.value(),
            self.cold_sha256,
            &self.canonical_bytes,
        )
    }
}

/// Proof that the selected F12 builder produced one complete, CRC-valid frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcceptedCodecFrame {
    source_index: CodecSourceIndex,
    attempt_id: AttemptId,
    frame_sha256: [u8; 32],
}

impl AcceptedCodecFrame {
    /// Binds the accepted candidate to its F12 source and full-frame bytes.
    pub fn from_candidate(
        candidate: &SelectedMainFrameCandidate,
    ) -> Result<Self, CodecOwnerStageError> {
        let identity = candidate.source_generation_identity();
        Self::from_complete_frame(
            identity.source_index(),
            identity.source_stage_key().attempt_id(),
            candidate.frame_bytes(),
        )
    }

    fn from_complete_frame(
        source_index: CodecSourceIndex,
        attempt_id: AttemptId,
        frame: &[u8],
    ) -> Result<Self, CodecOwnerStageError> {
        if frame.len() != FRAME_BYTES {
            return Err(CodecOwnerStageError::IncompleteFrame);
        }
        if eac3_crc16(&frame[2..]) != 0 {
            return Err(CodecOwnerStageError::FrameCrc);
        }
        Ok(Self {
            source_index,
            attempt_id,
            frame_sha256: Sha256::digest(frame).into(),
        })
    }

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn attempt_id(&self) -> AttemptId {
        self.attempt_id
    }

    pub const fn frame_sha256(&self) -> &[u8; 32] {
        &self.frame_sha256
    }
}

/// One source's staged codec-root value. No owner-local commit operation is provided.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodecOwnerProposal {
    source_index: CodecSourceIndex,
    frame_sha256: [u8; 32],
    before_generation: CodecOwnerGeneration,
    before_state_sha256: [u8; 32],
    after_state: CodecOwnerRoot,
}

impl CodecOwnerProposal {
    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn frame_sha256(&self) -> &[u8; 32] {
        &self.frame_sha256
    }

    pub const fn before_generation(&self) -> CodecOwnerGeneration {
        self.before_generation
    }

    pub const fn after_state(&self) -> &CodecOwnerRoot {
        &self.after_state
    }

    /// Builds an owner-level GTC receipt from the exact staged after-state.
    ///
    /// `before` and `transaction_id` are parent-supplied. This validates the proposal's before
    /// generation and canonical digest before describing its after-state; it does not install
    /// either value or grant top-level commit authority.
    pub fn gtc_receipt(
        &self,
        before: &CodecOwnerRoot,
        transaction_id: CallbackTransactionId,
    ) -> Result<GtcOwnerReceipt, CodecOwnerStageError> {
        let before_reference = before.state_reference();
        if before.generation != self.before_generation
            || before_reference.state_sha256 != self.before_state_sha256
        {
            return Err(CodecOwnerStageError::BeforeStateMismatch);
        }
        let expected_after_generation = before
            .generation
            .0
            .checked_add(1)
            .ok_or(CodecOwnerStageError::GenerationOverflow)?;
        if self.after_state.generation.0 != expected_after_generation {
            return Err(CodecOwnerStageError::GenerationMismatch);
        }
        let after_reference = self.after_state.state_reference();
        let before_snapshot = before_reference
            .gtc_snapshot()
            .map_err(CodecOwnerStageError::GtcReceipt)?;
        let after_snapshot = after_reference
            .gtc_snapshot()
            .map_err(CodecOwnerStageError::GtcReceipt)?;
        GtcOwnerReceipt::from_transition(before_snapshot, after_snapshot, transaction_id)
            .map_err(CodecOwnerStageError::GtcReceipt)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecOwnerStageError {
    IncompleteFrame,
    FrameCrc,
    MissingDurableAppend,
    UnexpectedDurableAppend,
    DurableAppendMismatch,
    SourceStageRequired,
    BeforeStateMismatch,
    GenerationMismatch,
    GenerationOverflow,
    GtcReceipt(GtcOwnerReceiptError),
}

fn validate_publication(
    accepted_frame: &AcceptedCodecFrame,
    output_after: Option<&OutputSourceAfterProposal>,
) -> Result<(), CodecOwnerStageError> {
    let source = accepted_frame.source_index.value();
    if source < SUPPRESSED_SOURCE_COUNT {
        return if output_after.is_none() {
            Ok(())
        } else {
            Err(CodecOwnerStageError::UnexpectedDurableAppend)
        };
    }

    let output_after = output_after.ok_or(CodecOwnerStageError::MissingDurableAppend)?;
    let receipt = output_after
        .durable_append_receipt()
        .ok_or(CodecOwnerStageError::MissingDurableAppend)?;
    let au_index = source
        .checked_sub(SUPPRESSED_SOURCE_COUNT)
        .ok_or(CodecOwnerStageError::DurableAppendMismatch)?;
    let expected_offset = au_index
        .checked_mul(FRAME_BYTES as u64)
        .ok_or(CodecOwnerStageError::DurableAppendMismatch)?;
    let expected_au = PublishedAuIndex::new(au_index);
    let expected_next_source = source
        .checked_add(1)
        .ok_or(CodecOwnerStageError::DurableAppendMismatch)?;
    let expected_attempt = accepted_frame.attempt_id.value().to_string();
    let published_au_matches = matches!(
        output_after.receipt(),
        OutputSourceCommitReceipt::Published(commit) if commit.au_index() == expected_au
    );
    if !published_au_matches
        || output_after.after_owner().next_source_index().value() != expected_next_source
        || output_after.after_owner().configuration().attempt_id() != expected_attempt.as_str()
        || receipt.attempt_id() != expected_attempt.as_str()
        || receipt.au_index() != expected_au
        || receipt.offset() != expected_offset
        || receipt.bytes_written() != FRAME_BYTES as u32
        || receipt.readback_sha256() != &accepted_frame.frame_sha256
        || receipt.crc_residual() != 0
        || !receipt.file_fsync()
        || !receipt.durable()
    {
        return Err(CodecOwnerStageError::DurableAppendMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        AcceptedCodecFrame, CodecOwnerRoot, CodecOwnerStageError, CodecShortFlags,
        F07_BLOCKS_PER_FRAME, F07_FULLBAND_LANES, FRAME_BYTES,
    };
    use crate::{
        publication::{
            OutputConfiguration, OutputOwner, OutputSourceAfterProposal, OutputSourceIdentity,
            OutputSourceProposal,
        },
        timeline::{CodecSourceIndex, FrontendCallbackIndex, JocDispatchIndex, PublishedAuIndex},
        transaction::{AttemptId, CallbackTransactionId, GtcOwnerId},
    };
    use sha2::{Digest, Sha256};

    const OUTPUT_PROVIDER: &[u8] =
        include_bytes!("../../tests/fixtures/output_owner/synthetic-output-provider.json");
    const TEST_ATTEMPT: AttemptId = AttemptId::new(3);

    fn complete_frame(source: u64, bytes: &[u8]) -> AcceptedCodecFrame {
        AcceptedCodecFrame::from_complete_frame(CodecSourceIndex::new(source), TEST_ATTEMPT, bytes)
            .expect("self-authored complete frame with valid CRC")
    }

    fn complete_frame_for_attempt(
        source: u64,
        attempt: AttemptId,
        bytes: &[u8],
    ) -> AcceptedCodecFrame {
        AcceptedCodecFrame::from_complete_frame(CodecSourceIndex::new(source), attempt, bytes)
            .expect("self-authored complete frame with valid CRC")
    }

    fn output_identity(source: u64) -> OutputSourceIdentity {
        OutputSourceIdentity::new(
            CodecSourceIndex::new(source),
            JocDispatchIndex::new(source + 1),
            source.checked_sub(2).map(PublishedAuIndex::new),
        )
    }

    fn output_after_for_source_two(frame: &[u8]) -> OutputSourceAfterProposal {
        output_after_for_source_two_attempt(frame, TEST_ATTEMPT)
    }

    fn output_after_for_source_two_attempt(
        frame: &[u8],
        attempt: AttemptId,
    ) -> OutputSourceAfterProposal {
        let configuration = OutputConfiguration::from_provider_json(
            OUTPUT_PROVIDER,
            4_608,
            &attempt.value().to_string(),
            "selected.ec3",
        )
        .expect("synthetic output provider");
        let mut output = OutputOwner::cold(configuration);
        let frame_sha256: [u8; 32] = Sha256::digest(frame).into();

        for source in 0..2 {
            let proposal = output
                .prepare_source(output_identity(source), frame, &frame_sha256)
                .expect("suppressed sources still require the complete frame");
            let OutputSourceProposal::Suppressed(proposal) = proposal else {
                panic!("source {source} must be suppressed")
            };
            output = proposal.into_after().into_after_owner();
        }

        let proposal = output
            .prepare_source(output_identity(2), frame, &frame_sha256)
            .expect("published source append proposal");
        let OutputSourceProposal::Append(proposal) = proposal else {
            panic!("source 2 must append")
        };
        let receipt = proposal
            .synthetic_durable_append_receipt(vec![FRAME_BYTES as u64], frame_sha256, 0)
            .expect("synthetic test receipt for the exact frame");
        proposal
            .complete(receipt)
            .expect("matching output proposal")
    }

    fn all_long_flags() -> CodecShortFlags {
        [[false; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME]
    }

    #[test]
    fn cold_state_reference_matches_the_washed_canonical_owner_value() {
        let reference = CodecOwnerRoot::cold().state_reference();
        assert_eq!(reference.owner_id(), "codec");
        assert_eq!(reference.state_ref(), "owners.codec");
        assert_eq!(reference.generation().value(), 0);
        assert_eq!(
            reference.canonical_bytes(),
            br#"{"codec":{"generation":0},"previous_short":{"i1_words":[0,0,0,0,0]}}"#
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(reference.canonical_bytes())),
            "f039d91adab362b63a9913eea116dc5ce38c8b77bf7d94c32352184a24594a55"
        );
        assert_eq!(reference.state_sha256(), reference.cold_sha256());

        let snapshot = reference
            .gtc_snapshot()
            .expect("canonical codec bytes adapt to the registered GTC owner");
        assert_eq!(snapshot.owner_id(), GtcOwnerId::Codec);
        assert_eq!(snapshot.state_ref().as_str(), "owners.codec");
        assert_eq!(snapshot.generation(), reference.generation().value());
        assert_eq!(snapshot.state_sha256(), *reference.state_sha256());
        assert_eq!(snapshot.cold_sha256(), *reference.cold_sha256());
        assert_eq!(snapshot.canonical_bytes(), reference.canonical_bytes());
    }

    #[test]
    fn callbacks_zero_and_one_emit_unchanged_codec_owner_receipts() {
        let root = CodecOwnerRoot::cold();

        for callback_index in [0, 1] {
            let transaction_id =
                CallbackTransactionId::new(&format!("codec-callback-{callback_index}"))
                    .expect("callback transaction identity is nonempty");
            let receipt = root
                .gtc_unchanged_callback_receipt(
                    FrontendCallbackIndex::new(callback_index),
                    transaction_id.clone(),
                )
                .expect("the two priming callbacks have no codec source stage");

            assert_eq!(receipt.owner_id(), GtcOwnerId::Codec);
            assert_eq!(receipt.before_generation(), 0);
            assert_eq!(receipt.after_generation(), 0);
            assert_eq!(receipt.before_sha256(), receipt.after_sha256());
            assert!(!receipt.changed());
            assert_eq!(receipt.commit_count(), 0);
            assert_eq!(receipt.transaction_id(), &transaction_id);
        }

        assert_eq!(
            root.gtc_unchanged_callback_receipt(
                FrontendCallbackIndex::new(2),
                CallbackTransactionId::new("codec-callback-2").unwrap(),
            ),
            Err(CodecOwnerStageError::SourceStageRequired)
        );
    }

    #[test]
    fn staged_source_receipt_binds_parent_transaction_and_exact_codec_after_state() {
        let before = CodecOwnerRoot::cold();
        let mut flags = all_long_flags();
        flags[F07_BLOCKS_PER_FRAME - 1] = [true, false, true, false, true];
        let accepted = complete_frame(0, &[0; FRAME_BYTES]);
        let proposal = before
            .stage_source(&accepted, &flags, None)
            .expect("suppressed source stages a complete codec after-state");
        let transaction_id = CallbackTransactionId::new("codec-source-zero-transaction")
            .expect("callback transaction identity is nonempty");

        let receipt = proposal
            .gtc_receipt(&before, transaction_id.clone())
            .expect("codec proposal adapts to its exact GTC before/after snapshots");
        let after_reference = proposal.after_state().state_reference();

        assert_eq!(receipt.owner_id(), GtcOwnerId::Codec);
        assert_eq!(receipt.before_generation(), before.generation().value());
        assert_eq!(
            receipt.after_generation(),
            after_reference.generation().value()
        );
        assert_eq!(receipt.after_sha256(), *after_reference.state_sha256());
        assert!(receipt.changed());
        assert_eq!(receipt.commit_count(), 1);
        assert_eq!(receipt.transaction_id(), &transaction_id);
        assert_eq!(
            proposal.after_state().previous_short(),
            &[true, false, true, false, true]
        );
        assert_eq!(before, CodecOwnerRoot::cold());
    }

    #[test]
    fn source_proposal_from_another_codec_before_state_is_rejected() {
        let cold = CodecOwnerRoot::cold();
        let accepted = complete_frame(0, &[0; FRAME_BYTES]);
        let mut flags_a = all_long_flags();
        flags_a[F07_BLOCKS_PER_FRAME - 1] = [true, false, false, false, false];
        let mut flags_b = all_long_flags();
        flags_b[F07_BLOCKS_PER_FRAME - 1] = [false, true, false, false, false];
        let root_a = *cold
            .stage_source(&accepted, &flags_a, None)
            .unwrap()
            .after_state();
        let root_b = *cold
            .stage_source(&accepted, &flags_b, None)
            .unwrap()
            .after_state();
        assert_eq!(root_a.generation(), root_b.generation());
        assert_ne!(
            root_a.state_reference().state_sha256(),
            root_b.state_reference().state_sha256()
        );
        let source_one = complete_frame(1, &[0; FRAME_BYTES]);
        let proposal = root_a
            .stage_source(&source_one, &all_long_flags(), None)
            .expect("source proposal stages from root A");

        assert_eq!(
            proposal.gtc_receipt(
                &root_b,
                CallbackTransactionId::new("codec-source-one-transaction").unwrap(),
            ),
            Err(CodecOwnerStageError::BeforeStateMismatch)
        );
    }

    #[test]
    fn accepted_source_state_reference_contains_full_after_bytes_and_keeps_cold_hash() {
        let mut flags = all_long_flags();
        flags[F07_BLOCKS_PER_FRAME - 1] = [true, false, true, false, true];
        let frame = complete_frame(0, &[0; FRAME_BYTES]);
        let proposal = CodecOwnerRoot::cold()
            .stage_source(&frame, &flags, None)
            .expect("complete suppressed source stages its codec after-state");
        let reference = proposal.after_state().state_reference();

        assert_eq!(reference.generation().value(), 1);
        assert_eq!(
            reference.canonical_bytes(),
            br#"{"codec":{"generation":1},"previous_short":{"i1_words":[1,0,1,0,1]}}"#
        );
        let cold_reference = CodecOwnerRoot::cold().state_reference();
        assert_eq!(reference.cold_sha256(), cold_reference.state_sha256());
        assert_ne!(reference.state_sha256(), reference.cold_sha256());
        assert_eq!(proposal.after_state().generation().value(), 1);
    }

    #[test]
    fn cold_and_reset_contain_only_codec_generation_and_five_previous_short_flags() {
        let cold = CodecOwnerRoot::cold();
        assert_eq!(cold.generation().value(), 0);
        assert_eq!(cold.previous_short(), &[false; F07_FULLBAND_LANES]);

        let mut warm = cold;
        let flags = complete_frame(0, &[0; FRAME_BYTES]);
        let proposal = warm
            .stage_source(&flags, &all_long_flags(), None)
            .expect("suppressed frame stages without an AU append");
        warm = *proposal.after_state();
        assert_eq!(warm.generation().value(), 1);
        warm.reset();
        assert_eq!(warm, cold);
    }

    #[test]
    fn suppressed_sources_require_complete_crc_valid_frame_and_stage_last_block_flags() {
        let root = CodecOwnerRoot::cold();
        let frame0 = complete_frame(0, &[0; FRAME_BYTES]);
        let source0 = root
            .stage_source(&frame0, &all_long_flags(), None)
            .expect("source 0 builds a complete suppressed frame");
        assert_eq!(root, CodecOwnerRoot::cold());
        assert_eq!(source0.source_index(), CodecSourceIndex::new(0));
        assert_eq!(source0.after_state().generation().value(), 1);

        let mut flags = all_long_flags();
        flags[5] = [true, false, true, false, true];
        let frame1 = complete_frame(1, &[0; FRAME_BYTES]);
        let source1 = source0
            .after_state()
            .stage_source(&frame1, &flags, None)
            .expect("source 1 stages the complete six-block plan");
        assert_eq!(
            source1.after_state().previous_short(),
            &[true, false, true, false, true]
        );
        assert_eq!(source1.after_state().generation().value(), 2);
    }

    #[test]
    fn frame_acceptance_rejects_incomplete_or_crc_invalid_bytes() {
        assert_eq!(
            AcceptedCodecFrame::from_complete_frame(
                CodecSourceIndex::new(0),
                TEST_ATTEMPT,
                &[0; FRAME_BYTES - 1],
            ),
            Err(CodecOwnerStageError::IncompleteFrame)
        );
        let mut invalid = [0; FRAME_BYTES];
        invalid[2] = 1;
        assert_eq!(
            AcceptedCodecFrame::from_complete_frame(
                CodecSourceIndex::new(0),
                TEST_ATTEMPT,
                &invalid
            ),
            Err(CodecOwnerStageError::FrameCrc)
        );
    }

    #[test]
    fn published_source_requires_a_matching_f15_durable_append_receipt() {
        let frame = [0; FRAME_BYTES];
        let accepted = complete_frame(2, &frame);
        let root = CodecOwnerRoot::cold();
        assert_eq!(
            root.stage_source(&accepted, &all_long_flags(), None),
            Err(CodecOwnerStageError::MissingDurableAppend)
        );
        assert_eq!(root, CodecOwnerRoot::cold());

        let output_after = output_after_for_source_two(&frame);
        let proposal = root
            .stage_source(&accepted, &all_long_flags(), Some(&output_after))
            .expect("matching F15 append-after proposal admits the codec proposal");
        assert_eq!(proposal.after_state().generation().value(), 1);
        assert_eq!(
            proposal.after_state().previous_short(),
            &[false; F07_FULLBAND_LANES]
        );

        let wrong_source = complete_frame(3, &frame);
        assert_eq!(
            root.stage_source(&wrong_source, &all_long_flags(), Some(&output_after)),
            Err(CodecOwnerStageError::DurableAppendMismatch)
        );
    }

    #[test]
    fn published_source_requires_the_same_attempt_as_the_f15_after_owner() {
        let frame = [0; FRAME_BYTES];
        let accepted = complete_frame_for_attempt(2, AttemptId::new(3), &frame);
        let output_after = output_after_for_source_two_attempt(&frame, AttemptId::new(4));

        assert_eq!(
            CodecOwnerRoot::cold().stage_source(&accepted, &all_long_flags(), Some(&output_after)),
            Err(CodecOwnerStageError::DurableAppendMismatch)
        );
    }

    #[test]
    fn published_source_requires_the_same_frame_bytes_as_the_f15_after_owner() {
        let written_frame = [0; FRAME_BYTES];
        let different_frame = {
            let mut bytes = [0; FRAME_BYTES];
            bytes[0] = 1;
            bytes
        };
        let output_after = output_after_for_source_two(&written_frame);
        let accepted = complete_frame(2, &different_frame);

        assert_eq!(
            CodecOwnerRoot::cold().stage_source(&accepted, &all_long_flags(), Some(&output_after)),
            Err(CodecOwnerStageError::DurableAppendMismatch)
        );
    }

    #[test]
    fn suppressed_sources_reject_a_published_append_receipt() {
        let frame = [0; FRAME_BYTES];
        let output_after = output_after_for_source_two(&frame);
        let accepted = complete_frame(0, &frame);
        assert_eq!(
            CodecOwnerRoot::cold().stage_source(&accepted, &all_long_flags(), Some(&output_after)),
            Err(CodecOwnerStageError::UnexpectedDurableAppend)
        );
    }
}
