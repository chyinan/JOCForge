// pattern: Functional Core
//! Noncommitting codec-owner state for the selected source-frame transition.

use crate::{
    analysis::structure_refresh::{F07_BLOCKS_PER_FRAME, F07_FULLBAND_LANES},
    integrity::crc16::eac3_crc16,
    publication::OutputDurableAppendReceipt,
    timeline::{CodecSourceIndex, PublishedAuIndex},
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

    /// Stages the selected F12 state after a complete candidate has passed CRC and, for
    /// published sources, the matching F15 durable append receipt has been admitted.
    ///
    /// This returns a proposal only. The parent owns installation with all other source roots.
    pub fn stage_source(
        &self,
        accepted_frame: &AcceptedCodecFrame,
        current_short_flags: &CodecShortFlags,
        durable_append: Option<&OutputDurableAppendReceipt>,
    ) -> Result<CodecOwnerProposal, CodecOwnerStageError> {
        validate_publication(accepted_frame, durable_append)?;
        let next_generation = self
            .generation
            .0
            .checked_add(1)
            .ok_or(CodecOwnerStageError::GenerationOverflow)?;
        let after_state = Self {
            generation: CodecOwnerGeneration(next_generation),
            previous_short: current_short_flags[F07_BLOCKS_PER_FRAME - 1],
        };

        Ok(CodecOwnerProposal {
            source_index: accepted_frame.source_index,
            frame_sha256: accepted_frame.frame_sha256,
            before_generation: self.generation,
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
}

/// Proof that the selected F12 builder produced one complete, CRC-valid frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcceptedCodecFrame {
    source_index: CodecSourceIndex,
    frame_sha256: [u8; 32],
}

impl AcceptedCodecFrame {
    /// Binds the accepted candidate to its F12 source and full-frame bytes.
    pub fn from_candidate(
        candidate: &SelectedMainFrameCandidate,
    ) -> Result<Self, CodecOwnerStageError> {
        Self::from_complete_frame(candidate.source_index(), candidate.frame_bytes())
    }

    fn from_complete_frame(
        source_index: CodecSourceIndex,
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
            frame_sha256: Sha256::digest(frame).into(),
        })
    }

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecOwnerStageError {
    IncompleteFrame,
    FrameCrc,
    MissingDurableAppend,
    UnexpectedDurableAppend,
    DurableAppendMismatch,
    GenerationOverflow,
}

fn validate_publication(
    accepted_frame: &AcceptedCodecFrame,
    durable_append: Option<&OutputDurableAppendReceipt>,
) -> Result<(), CodecOwnerStageError> {
    let source = accepted_frame.source_index.value();
    if source < SUPPRESSED_SOURCE_COUNT {
        return if durable_append.is_none() {
            Ok(())
        } else {
            Err(CodecOwnerStageError::UnexpectedDurableAppend)
        };
    }

    let receipt = durable_append.ok_or(CodecOwnerStageError::MissingDurableAppend)?;
    let au_index = source
        .checked_sub(SUPPRESSED_SOURCE_COUNT)
        .ok_or(CodecOwnerStageError::DurableAppendMismatch)?;
    let expected_offset = au_index
        .checked_mul(FRAME_BYTES as u64)
        .ok_or(CodecOwnerStageError::DurableAppendMismatch)?;
    if receipt.au_index() != PublishedAuIndex::new(au_index)
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
            OutputConfiguration, OutputOwner, OutputSourceIdentity, OutputSourceProposal,
        },
        timeline::{CodecSourceIndex, JocDispatchIndex, PublishedAuIndex},
    };
    use sha2::{Digest, Sha256};

    const OUTPUT_PROVIDER: &[u8] =
        include_bytes!("../../tests/fixtures/output_owner/synthetic-output-provider.json");

    fn complete_frame(source: u64, bytes: &[u8]) -> AcceptedCodecFrame {
        AcceptedCodecFrame::from_complete_frame(CodecSourceIndex::new(source), bytes)
            .expect("self-authored complete frame with valid CRC")
    }

    fn output_identity(source: u64) -> OutputSourceIdentity {
        OutputSourceIdentity::new(
            CodecSourceIndex::new(source),
            JocDispatchIndex::new(source + 1),
            source.checked_sub(2).map(PublishedAuIndex::new),
        )
    }

    fn durable_receipt_for_source_two(
        frame: &[u8],
    ) -> crate::publication::OutputDurableAppendReceipt {
        let configuration = OutputConfiguration::from_provider_json(
            OUTPUT_PROVIDER,
            4_608,
            "codec-owner-test",
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
        proposal
            .record_durable_io_success(vec![FRAME_BYTES as u64], frame_sha256, 0)
            .expect("typed durable append receipt")
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
                &[0; FRAME_BYTES - 1]
            ),
            Err(CodecOwnerStageError::IncompleteFrame)
        );
        let mut invalid = [0; FRAME_BYTES];
        invalid[2] = 1;
        assert_eq!(
            AcceptedCodecFrame::from_complete_frame(CodecSourceIndex::new(0), &invalid),
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

        let receipt = durable_receipt_for_source_two(&frame);
        let proposal = root
            .stage_source(&accepted, &all_long_flags(), Some(&receipt))
            .expect("matching durable F15 append receipt admits the proposal");
        assert_eq!(proposal.after_state().generation().value(), 1);
        assert_eq!(
            proposal.after_state().previous_short(),
            &[false; F07_FULLBAND_LANES]
        );

        let wrong_source = complete_frame(3, &frame);
        assert_eq!(
            root.stage_source(&wrong_source, &all_long_flags(), Some(&receipt)),
            Err(CodecOwnerStageError::DurableAppendMismatch)
        );
    }

    #[test]
    fn suppressed_sources_reject_a_published_append_receipt() {
        let frame = [0; FRAME_BYTES];
        let receipt = durable_receipt_for_source_two(&frame);
        let accepted = complete_frame(0, &frame);
        assert_eq!(
            CodecOwnerRoot::cold().stage_source(&accepted, &all_long_flags(), Some(&receipt)),
            Err(CodecOwnerStageError::UnexpectedDurableAppend)
        );
    }
}
