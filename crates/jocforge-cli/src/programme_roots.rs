// pattern: Imperative Shell
//! Owns the selected CLI programme roots and installs complete callback proposals once.

use jocforge_application::{
    PreparedJobInput,
    drc_source_bridge::NoneDrcOwnerRoot,
    independent_meter_callback::{AttemptId, CallbackTransactionId, GtcOwnerReceipt, ProgrammeId},
    integrity_source_bridge::IntegrityProgrammeOwner,
    metadata_source_bridge::{MetadataEofProposal, MetadataOwner, SelectedJocState},
    meter_finalization_bridge::{
        ProgrammeMeasurementReceipt, StandaloneMeterRoot, unchanged_meter_root_receipt,
    },
    output_publication::{
        OutputFinalExposureReceipt, OutputFinalizedProposal, ProgrammeOutputBinding,
        ProgrammeOutputFinalizeProposal,
    },
    reader_eof_callback::{GtcOwnerId, ReaderEofMachine},
    source_analysis_bridge::AnalysisOwnerRoot,
    source_frame_bridge::CodecOwnerRoot,
    spatial_callback::MainSpatialColdRoots,
};

use crate::source_execution::{DurableAppendSummary, SourceFrameAdvance};
use jocforge_io::pcm::frontend_pcm_source::{
    CodecSourceIndex, FrontendCallbackIndex, NormalCallbackCount,
};

const MAIN_FRAME_SAMPLES: u64 = 1_536;
const SUPPRESSED_SOURCES: u64 = 2;
const CACHED_DRAINS: u64 = 4;
const OUTPUT_FRAME_BYTES: u64 = 4_096;

const OWNER_ORDER: [GtcOwnerId; 12] = [
    GtcOwnerId::Analysis,
    GtcOwnerId::Codec,
    GtcOwnerId::Drc,
    GtcOwnerId::FrontendControl,
    GtcOwnerId::Integrity,
    GtcOwnerId::Joc,
    GtcOwnerId::Metadata,
    GtcOwnerId::Meter,
    GtcOwnerId::Output,
    GtcOwnerId::ReaderEof,
    GtcOwnerId::SpatialAudio,
    GtcOwnerId::SpatialControl,
];

const CALLBACK_OWNERS: [GtcOwnerId; 4] = [
    GtcOwnerId::ReaderEof,
    GtcOwnerId::FrontendControl,
    GtcOwnerId::SpatialControl,
    GtcOwnerId::SpatialAudio,
];

/// Selected clock projection s=g-2, kept typed at the CLI/application boundary.
pub(crate) fn source_for_callback_clock(
    callback_index: FrontendCallbackIndex,
) -> Option<CodecSourceIndex> {
    callback_index
        .value()
        .checked_sub(2)
        .map(|source| CodecSourceIndex::new(u64::from(source)))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProgrammeRootStatus {
    Ready,
    Active,
    Finalizing,
    Done,
}

/// One typed receipt batch after a successful in-memory programme-root replacement.
#[must_use]
pub(crate) struct ProgrammeCallbackCommit {
    pub callback_transaction_id: CallbackTransactionId,
    pub source_transaction_id: Option<CallbackTransactionId>,
    pub callback_index: FrontendCallbackIndex,
    pub source_index: Option<CodecSourceIndex>,
    pub au_index: Option<u64>,
    pub frame_sha256: Option<[u8; 32]>,
    pub metadata_container_bytes: Option<usize>,
    pub append: Option<DurableAppendSummary>,
    pub phase_generation_before: u64,
    pub phase_generation_after: u64,
    pub owner_receipts: Vec<GtcOwnerReceipt>,
}

/// The only mutable programme snapshot used by the selected CLI attempt.
///
/// It contains all twelve committed owner roots. Callers prepare proposals against `&self`, then
/// consume this value only after the complete receipt batch has passed validation.
#[must_use]
pub(crate) struct ProgrammeOwnerRoots<'a> {
    programme_id: ProgrammeId,
    attempt: AttemptId,
    status: ProgrammeRootStatus,
    phase_generation: u64,
    callback_index: FrontendCallbackIndex,
    source_index: CodecSourceIndex,
    normal_callbacks: NormalCallbackCount,
    expected_sources: u64,
    expected_published: u64,
    expected_product_bytes: u64,
    published_aus: u64,
    durable_bytes: u64,
    measurement: &'a ProgrammeMeasurementReceipt,
    meter: &'a StandaloneMeterRoot,
    main: MainSpatialColdRoots,
    reader_eof: ReaderEofMachine,
    integrity: IntegrityProgrammeOwner,
    analysis: AnalysisOwnerRoot,
    drc: NoneDrcOwnerRoot,
    joc: SelectedJocState,
    metadata: MetadataOwner,
    codec: CodecOwnerRoot,
    output: ProgrammeOutputBinding,
}

impl<'a> ProgrammeOwnerRoots<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn ready(
        prepared: &PreparedJobInput,
        attempt: AttemptId,
        measurement: &'a ProgrammeMeasurementReceipt,
        meter: &'a StandaloneMeterRoot,
        main: MainSpatialColdRoots,
        reader_eof: ReaderEofMachine,
        integrity: IntegrityProgrammeOwner,
        analysis: AnalysisOwnerRoot,
        drc: NoneDrcOwnerRoot,
        joc: SelectedJocState,
        metadata: MetadataOwner,
        codec: CodecOwnerRoot,
        output: ProgrammeOutputBinding,
    ) -> Result<Self, String> {
        let normal = prepared
            .configuration()
            .source_samples()
            .div_ceil(MAIN_FRAME_SAMPLES);
        let expected_sources = output.output_configuration().expected_sources();
        let expected_published = output.output_configuration().expected_published_aus();
        let expected_product_bytes = output.output_configuration().expected_product_bytes();
        if measurement.programme_id() != prepared.programme_id()
            || measurement.attempt() != attempt
            || measurement.input_sha256() != &prepared.resource_identity().input_sha256()
            || measurement.config_sha256() != &prepared.config_sha256()
            || meter.identity().programme_id() != prepared.programme_id()
            || meter.identity().attempt() != attempt
            || output.programme_id() != prepared.programme_id()
            || output.attempt_id() != attempt
            || reader_eof.generation() != 0
            || reader_eof.committed().callback_index().value() != 0
            || main.frontend_control().generation() != 0
            || main.frontend_control().committed().callback_index().value() != 0
            || main.spatial_control().generation() != 0
            || main.spatial_control().committed().callback_index().value() != 0
            || main.spatial_audio().generation() != 0
            || main.spatial_audio().committed().callback_index().value() != 0
            || integrity.committed_state().generation() != 0
            || analysis
                .state_reference()
                .map_err(|error| format!("Analysis cold state reference: {error:?}"))?
                .generation()
                != 0
            || drc
                .state()
                .source_generation()
                .map_err(|error| format!("DRC cold source generation: {error:?}"))?
                .value()
                != 0
            || joc.generation() != 0
            || metadata.state_reference().generation() != 0
            || codec.generation().value() != 0
            || output
                .gtc_state_reference()
                .map_err(|error| format!("Output cold state reference: {error}"))?
                .generation()
                != 0
        {
            return Err("READY root set failed input identity or cold-root checks".to_owned());
        }
        if expected_sources == 0
            || expected_sources != normal.saturating_add(SUPPRESSED_SOURCES)
            || expected_published != expected_sources.saturating_sub(SUPPRESSED_SOURCES)
            || expected_published.checked_mul(OUTPUT_FRAME_BYTES) != Some(expected_product_bytes)
        {
            return Err("READY clock plan does not match the admitted output plan".to_owned());
        }

        Ok(Self {
            programme_id: prepared.programme_id().clone(),
            attempt,
            status: ProgrammeRootStatus::Ready,
            phase_generation: 1,
            callback_index: FrontendCallbackIndex::new(0),
            source_index: CodecSourceIndex::new(0),
            normal_callbacks: NormalCallbackCount::new(normal),
            expected_sources,
            expected_published,
            expected_product_bytes,
            published_aus: 0,
            durable_bytes: 0,
            measurement,
            meter,
            main,
            reader_eof,
            integrity,
            analysis,
            drc,
            joc,
            metadata,
            codec,
            output,
        })
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn normal_callbacks(&self) -> NormalCallbackCount {
        self.normal_callbacks
    }

    pub const fn expected_sources(&self) -> u64 {
        self.expected_sources
    }

    pub const fn expected_published(&self) -> u64 {
        self.expected_published
    }

    pub const fn expected_product_bytes(&self) -> u64 {
        self.expected_product_bytes
    }

    pub const fn published_aus(&self) -> u64 {
        self.published_aus
    }

    pub const fn durable_bytes(&self) -> u64 {
        self.durable_bytes
    }

    pub const fn phase_generation(&self) -> u64 {
        self.phase_generation
    }

    pub fn is_done(&self) -> bool {
        self.status == ProgrammeRootStatus::Done
    }

    pub const fn main(&self) -> &MainSpatialColdRoots {
        &self.main
    }

    pub const fn reader_eof(&self) -> &ReaderEofMachine {
        &self.reader_eof
    }

    pub const fn integrity(&self) -> &IntegrityProgrammeOwner {
        &self.integrity
    }

    pub const fn analysis(&self) -> &AnalysisOwnerRoot {
        &self.analysis
    }

    pub const fn drc(&self) -> &NoneDrcOwnerRoot {
        &self.drc
    }

    pub const fn joc(&self) -> &SelectedJocState {
        &self.joc
    }

    pub const fn metadata(&self) -> &MetadataOwner {
        &self.metadata
    }

    pub const fn codec(&self) -> &CodecOwnerRoot {
        &self.codec
    }

    pub const fn output(&self) -> &ProgrammeOutputBinding {
        &self.output
    }

    pub const fn meter(&self) -> &StandaloneMeterRoot {
        &self.meter
    }

    pub const fn measurement(&self) -> &ProgrammeMeasurementReceipt {
        &self.measurement
    }

    pub fn commit_callback_only(
        self,
        prepared: &PreparedJobInput,
        callback_index: FrontendCallbackIndex,
        transaction_id: CallbackTransactionId,
        main_roots: MainSpatialColdRoots,
        reader_eof: ReaderEofMachine,
        changed_receipts: [GtcOwnerReceipt; 4],
    ) -> Result<(Self, ProgrammeCallbackCommit), String> {
        if callback_index != self.callback_index
            || source_for_callback_clock(callback_index).is_some()
            || !matches!(
                (self.status, callback_index.value()),
                (ProgrammeRootStatus::Ready, 0) | (ProgrammeRootStatus::Active, 1)
            )
        {
            return Err(
                "callback-only transaction clock does not match READY/ACTIVE root".to_owned(),
            );
        }
        let expected_transaction = callback_transaction_id(callback_index)?;
        if transaction_id != expected_transaction {
            return Err("callback-only transaction identity is not canonical for g".to_owned());
        }
        let next_callback = callback_index
            .value()
            .checked_add(1)
            .ok_or_else(|| "callback clock overflow".to_owned())?;
        if reader_eof.committed().callback_index().value() != next_callback
            || main_roots
                .spatial_control()
                .committed()
                .callback_index()
                .value()
                != next_callback
            || main_roots
                .spatial_audio()
                .committed()
                .callback_index()
                .value()
                != next_callback
            || main_roots
                .frontend_control()
                .committed()
                .callback_index()
                .value()
                != next_callback
        {
            return Err("callback-only after roots do not match g+1".to_owned());
        }

        let mut receipts = changed_receipts.into_iter().collect::<Vec<_>>();
        receipts.push(
            unchanged_meter_root_receipt(
                prepared,
                &self.meter,
                &self.measurement,
                transaction_id.clone(),
            )
            .map_err(|error| format!("Meter unchanged receipt: {error:?}"))?,
        );
        receipts.push(
            self.drc
                .gtc_unchanged_callback_receipt(callback_index, transaction_id.clone())
                .map_err(|error| format!("DRC unchanged receipt: {error:?}"))?,
        );
        receipts.push(
            self.analysis
                .gtc_unchanged_callback_receipt(callback_index, transaction_id.clone())
                .map_err(|error| format!("Analysis unchanged receipt: {error:?}"))?,
        );
        receipts.push(
            self.codec
                .gtc_unchanged_callback_receipt(callback_index, transaction_id.clone())
                .map_err(|error| format!("Codec unchanged receipt: {error:?}"))?,
        );
        receipts.push(
            self.metadata
                .gtc_unchanged_callback_receipt(callback_index, transaction_id.clone())
                .map_err(|error| format!("Metadata unchanged receipt: {error:?}"))?,
        );
        receipts.push(
            self.joc
                .gtc_owner_receipt_to(&self.joc, transaction_id.clone())
                .map_err(|error| format!("JOC unchanged receipt: {error:?}"))?,
        );
        let integrity_owner_state = self.integrity.owner_state_snapshot();
        let integrity_snapshot = integrity_owner_state
            .gtc_snapshot()
            .map_err(|error| format!("Integrity unchanged snapshot: {error:?}"))?;
        receipts.push(
            GtcOwnerReceipt::from_transition(
                integrity_snapshot,
                integrity_snapshot,
                transaction_id.clone(),
            )
            .map_err(|error| format!("Integrity unchanged receipt: {error:?}"))?,
        );
        receipts.push(
            self.output
                .gtc_unchanged_callback_receipt(callback_index, transaction_id.clone())
                .map_err(|error| format!("Output unchanged receipt: {error}"))?,
        );
        let expected_changed = |owner: GtcOwnerId| CALLBACK_OWNERS.contains(&owner);
        let owner_receipts = validate_receipts(receipts, &transaction_id, expected_changed)?;

        let phase_generation_after = self
            .phase_generation
            .checked_add(1)
            .ok_or_else(|| "phase generation overflow".to_owned())?;
        let after = Self {
            programme_id: self.programme_id,
            attempt: self.attempt,
            status: ProgrammeRootStatus::Active,
            phase_generation: phase_generation_after,
            callback_index: FrontendCallbackIndex::new(next_callback),
            source_index: self.source_index,
            normal_callbacks: self.normal_callbacks,
            expected_sources: self.expected_sources,
            expected_published: self.expected_published,
            expected_product_bytes: self.expected_product_bytes,
            published_aus: self.published_aus,
            durable_bytes: self.durable_bytes,
            measurement: self.measurement,
            meter: self.meter,
            main: main_roots,
            reader_eof,
            integrity: self.integrity,
            analysis: self.analysis,
            drc: self.drc,
            joc: self.joc,
            metadata: self.metadata,
            codec: self.codec,
            output: self.output,
        };
        let receipt = ProgrammeCallbackCommit {
            callback_transaction_id: transaction_id,
            source_transaction_id: None,
            callback_index,
            source_index: None,
            au_index: None,
            frame_sha256: None,
            metadata_container_bytes: None,
            append: None,
            phase_generation_before: phase_generation_after - 1,
            phase_generation_after,
            owner_receipts,
        };
        Ok((after, receipt))
    }

    pub fn commit_source_advance(
        self,
        gtc_callback_transaction_id: CallbackTransactionId,
        prepared: &PreparedJobInput,
        advance: SourceFrameAdvance,
    ) -> Result<(Self, ProgrammeCallbackCommit), String> {
        if prepared.programme_id() != &self.programme_id
            || self.measurement.programme_id() != prepared.programme_id()
            || self.measurement.attempt() != self.attempt
            || self.measurement.input_sha256() != &prepared.resource_identity().input_sha256()
            || self.measurement.config_sha256() != &prepared.config_sha256()
        {
            return Err("source callback input identity differs from the READY root".to_owned());
        }
        let callback_index = advance.callback_index;
        let source_index = advance.source_index;
        if self.status != ProgrammeRootStatus::Active
            || callback_index != self.callback_index
            || source_index != self.source_index
            || source_for_callback_clock(callback_index) != Some(source_index)
        {
            return Err("source callback/source index is stale or out of order".to_owned());
        }
        let expected_callback_transaction = callback_transaction_id(callback_index)?;
        let expected_source_transaction = source_transaction_id(source_index)?;
        if gtc_callback_transaction_id != expected_callback_transaction
            || advance.source_transaction_id != expected_source_transaction
        {
            return Err("nested callback/source transaction identity mismatch".to_owned());
        }
        if advance.source_transaction_id != expected_source_transaction
            || advance.integrity_owner_receipt.transaction_id() != &advance.source_transaction_id
            || advance.integrity_owner_receipt.programme_id() != &self.programme_id
            || advance.integrity_owner_receipt.attempt() != self.attempt
            || advance
                .integrity_owner_receipt
                .callback_identity()
                .programme_id()
                != &self.programme_id
            || advance
                .integrity_owner_receipt
                .callback_identity()
                .attempt_id()
                != self.attempt
            || advance
                .integrity_owner_receipt
                .callback_identity()
                .callback_index()
                != callback_index
            || advance.integrity_owner_receipt.before_generation()
                != self.integrity.committed_state().generation()
            || advance.integrity_owner_receipt.after_generation()
                != advance.integrity_root.committed_state().generation()
        {
            return Err("Integrity proposal is not bound to this source callback root".to_owned());
        }
        let next_callback = callback_index
            .value()
            .checked_add(1)
            .ok_or_else(|| "callback clock overflow".to_owned())?;
        let next_source = source_index
            .value()
            .checked_add(1)
            .ok_or_else(|| "source clock overflow".to_owned())?;
        let next_phase_generation = self
            .phase_generation
            .checked_add(1)
            .ok_or_else(|| "phase generation overflow".to_owned())?;

        let cached_drain = u64::from(callback_index.value()) >= self.normal_callbacks.value();
        let mut receipts = advance.owner_receipts.clone();
        let integrity_before = advance
            .integrity_owner_receipt
            .before()
            .gtc_snapshot()
            .map_err(|error| format!("Integrity before snapshot: {error:?}"))?;
        let integrity_after = advance
            .integrity_owner_receipt
            .after()
            .gtc_snapshot()
            .map_err(|error| format!("Integrity after snapshot: {error:?}"))?;
        receipts.push(
            GtcOwnerReceipt::from_transition(
                integrity_before,
                integrity_after,
                gtc_callback_transaction_id.clone(),
            )
            .map_err(|error| format!("Integrity GTC receipt: {error:?}"))?,
        );

        let output_before = self
            .output
            .gtc_state_reference()
            .map_err(|error| format!("Output before state reference: {error}"))?;
        let output_after_reference = advance
            .output_after
            .after_owner()
            .state_reference()
            .map_err(|error| format!("Output after-root: {error}"))?;
        let output_after_generation = output_after_reference.generation();
        let next_published = self
            .published_aus
            .checked_add(u64::from(advance.append.is_some()))
            .ok_or_else(|| "published AU clock overflow".to_owned())?;
        let next_durable_bytes = self
            .durable_bytes
            .checked_add(u64::from(
                advance
                    .append
                    .as_ref()
                    .map_or(0, |append| append.bytes_written),
            ))
            .ok_or_else(|| "durable output length overflow".to_owned())?;

        let expected_au = source_index.value().checked_sub(SUPPRESSED_SOURCES);
        match (expected_au, advance.append.as_ref()) {
            (None, None) => {
                if advance.metadata_container_bytes.is_some()
                    || !matches!(
                        advance.f14k_source.outcome(),
                        jocforge_application::integrity_source_bridge::F14kPayloadOutcome::AbsentPublishedPayloadV1
                    )
                    || advance.integrity_receipts.au().published()
                {
                    return Err("suppressed source unexpectedly carries a published payload".to_owned());
                }
            }
            (Some(expected_au_index), Some(append)) => {
                if append.au_index != expected_au_index
                    || append.bytes_written != 4_096
                    || append.crc_residual != 0
                    || !append.file_fsync
                    || !append.durable
                    || advance.metadata_container_bytes.is_none()
                    || !matches!(
                        advance.f14k_source.outcome(),
                        jocforge_application::integrity_source_bridge::F14kPayloadOutcome::ProtectedEmdf { .. }
                    )
                    || !advance.integrity_receipts.au().published()
                    || advance.integrity_receipts.au().crc_residual() != 0
                    || advance.f14k_source.au_index().map(|value| value.value())
                        != Some(expected_au_index)
                    || advance.integrity_receipts.au().identity().au_index
                        .map(|value| value.value())
                        != Some(expected_au_index)
                {
                    return Err("published source failed F12/F14/F15 durable receipt checks".to_owned());
                }
            }
            _ => return Err("source publication disposition and durable append disagree".to_owned()),
        }
        if output_after_generation != output_before.generation() + 1
            || advance.reader_eof_root.committed().callback_index().value() != next_callback
            || advance
                .main_roots
                .spatial_control()
                .committed()
                .callback_index()
                .value()
                != next_callback
            || advance
                .main_roots
                .spatial_audio()
                .committed()
                .callback_index()
                .value()
                != next_callback
            || advance.main_roots.frontend_control().generation()
                != self.main.frontend_control().generation() + u64::from(!cached_drain)
            || analysis_generation(&advance.analysis_root)?
                != analysis_generation(&self.analysis)? + 1
            || drc_generation(&advance.drc_root)? != drc_generation(&self.drc)? + 1
            || advance.metadata_owner.state_reference().generation()
                != self.metadata.state_reference().generation() + 1
            || advance.joc_state.generation() != self.joc.generation() + 1
            || advance.integrity_root.committed_state().generation()
                != self.integrity.committed_state().generation() + 1
            || advance.codec_root.generation().value() != self.codec.generation().value() + 1
        {
            return Err(
                "source after-roots do not advance the expected owner generations".to_owned(),
            );
        }
        let expected_changed = |owner: GtcOwnerId| {
            owner != GtcOwnerId::Meter && !(cached_drain && owner == GtcOwnerId::FrontendControl)
        };
        let owner_receipts =
            validate_receipts(receipts, &gtc_callback_transaction_id, expected_changed)?;
        validate_source_receipt_generations(
            &owner_receipts,
            &self,
            &advance,
            output_before.generation(),
            output_after_generation,
        )?;

        let frame_sha256 = advance.frame_sha256;
        let metadata_container_bytes = advance.metadata_container_bytes;
        let append = advance.append;
        let output_after = self
            .output
            .with_after_source(advance.output_after)
            .map_err(|error| format!("Output after-root: {error}"))?;
        let after = Self {
            programme_id: self.programme_id,
            attempt: self.attempt,
            status: ProgrammeRootStatus::Active,
            phase_generation: next_phase_generation,
            callback_index: FrontendCallbackIndex::new(next_callback),
            source_index: CodecSourceIndex::new(next_source),
            normal_callbacks: self.normal_callbacks,
            expected_sources: self.expected_sources,
            expected_published: self.expected_published,
            expected_product_bytes: self.expected_product_bytes,
            published_aus: next_published,
            durable_bytes: next_durable_bytes,
            measurement: self.measurement,
            meter: self.meter,
            main: advance.main_roots,
            reader_eof: advance.reader_eof_root,
            integrity: advance.integrity_root,
            analysis: advance.analysis_root,
            drc: advance.drc_root,
            joc: advance.joc_state,
            metadata: advance.metadata_owner,
            codec: advance.codec_root,
            output: output_after,
        };
        let au_index = append.as_ref().map(|receipt| receipt.au_index);
        let receipt = ProgrammeCallbackCommit {
            callback_transaction_id: gtc_callback_transaction_id,
            source_transaction_id: Some(advance.source_transaction_id),
            callback_index,
            source_index: Some(source_index),
            au_index,
            frame_sha256: Some(frame_sha256),
            metadata_container_bytes,
            append,
            phase_generation_before: next_phase_generation - 1,
            phase_generation_after: next_phase_generation,
            owner_receipts,
        };
        Ok((after, receipt))
    }

    pub fn is_complete_for_eof(&self) -> bool {
        self.status == ProgrammeRootStatus::Active
            && self.normal_callbacks.value().checked_add(CACHED_DRAINS)
                == Some(u64::from(self.callback_index.value()))
            && self.source_index.value() == self.expected_sources
            && self.published_aus == self.expected_published
            && self.durable_bytes == self.expected_product_bytes
            && self.output.output_configuration().expected_product_bytes()
                == self.expected_product_bytes
    }

    pub fn prepare_eof_proposal(&mut self) -> Result<MetadataEofProposal, String> {
        if !self.is_complete_for_eof() {
            return Err("EOF requires all selected source and durable AU roots".to_owned());
        }
        self.metadata
            .prepare_eof(self.expected_sources)
            .map_err(|error| format!("Metadata EOF proposal: {error:?}"))
    }

    pub fn begin_finalizing(
        mut self,
        eof: MetadataEofProposal,
        transaction_id: CallbackTransactionId,
    ) -> Result<(Self, GtcOwnerReceipt, ProgrammeOutputFinalizeProposal), String> {
        if !self.is_complete_for_eof()
            || self.source_index.value() != eof.expected_sources()
            || transaction_id.as_str().is_empty()
        {
            return Err("programme EOF does not match the committed source/AU roots".to_owned());
        }
        let finalize =
            jocforge_application::output_publication::prepare_output_finalize(&self.output, &eof)
                .map_err(|error| format!("F15 finalize proposal: {error}"))?;
        let before_generation = self.metadata.state_reference().generation();
        let handoff = eof
            .into_parent_handoff(&self.metadata, transaction_id)
            .map_err(|error| format!("Metadata EOF parent handoff: {error:?}"))?;
        let (metadata, receipt) = handoff.into_parts();
        if receipt.owner_id() != GtcOwnerId::Metadata
            || receipt.before_generation() != before_generation
            || receipt.after_generation() != metadata.state_reference().generation()
            || !receipt.changed()
            || receipt.commit_count() != 1
        {
            return Err("Metadata EOF receipt does not match its after-root".to_owned());
        }
        self.metadata = metadata;
        self.status = ProgrammeRootStatus::Finalizing;
        self.phase_generation = self
            .phase_generation
            .checked_add(1)
            .ok_or_else(|| "phase generation overflow".to_owned())?;
        Ok((self, receipt, finalize))
    }

    pub fn finish(
        mut self,
        finalized: OutputFinalizedProposal,
    ) -> Result<(Self, OutputFinalExposureReceipt), String> {
        if self.status != ProgrammeRootStatus::Finalizing || !self.is_complete_for_terminal_output()
        {
            return Err("DONE transition requires a committed FINALIZING root".to_owned());
        }
        let receipt = finalized.receipt().clone();
        if receipt.product_length() != self.expected_product_bytes {
            return Err(
                "FinalExposureReceipt does not match the selected product length".to_owned(),
            );
        }
        self.output = self
            .output
            .with_finalized(finalized)
            .map_err(|error| format!("F15 finalized output root: {error}"))?;
        self.status = ProgrammeRootStatus::Done;
        self.phase_generation = self
            .phase_generation
            .checked_add(1)
            .ok_or_else(|| "phase generation overflow".to_owned())?;
        Ok((self, receipt))
    }

    fn is_complete_for_terminal_output(&self) -> bool {
        self.source_index.value() == self.expected_sources
            && self.published_aus == self.expected_published
            && self.durable_bytes == self.expected_product_bytes
    }
}

fn callback_transaction_id(
    callback_index: FrontendCallbackIndex,
) -> Result<CallbackTransactionId, String> {
    CallbackTransactionId::new(&format!(
        "jocforge-main-callback-g{}",
        callback_index.value()
    ))
    .map_err(|error| format!("callback transaction identity: {error:?}"))
}

fn source_transaction_id(source_index: CodecSourceIndex) -> Result<CallbackTransactionId, String> {
    CallbackTransactionId::new(&format!("jocforge-source-{}", source_index.value()))
        .map_err(|error| format!("source transaction identity: {error:?}"))
}

fn validate_receipts(
    mut receipts: Vec<GtcOwnerReceipt>,
    callback_transaction_id: &CallbackTransactionId,
    expected_changed: impl Fn(GtcOwnerId) -> bool,
) -> Result<Vec<GtcOwnerReceipt>, String> {
    if receipts.len() != OWNER_ORDER.len() {
        return Err(format!(
            "programme callback requires exactly 12 owner receipts, got {}",
            receipts.len()
        ));
    }
    receipts.sort_by(|left, right| left.owner_id().as_str().cmp(right.owner_id().as_str()));
    for (receipt, expected_owner) in receipts.iter().zip(OWNER_ORDER) {
        let changed = expected_changed(expected_owner);
        if receipt.owner_id() != expected_owner
            || receipt.transaction_id() != callback_transaction_id
            || receipt.changed() != changed
            || receipt.commit_count() != u8::from(changed)
            || receipt.after_generation()
                != receipt
                    .before_generation()
                    .checked_add(u64::from(changed))
                    .unwrap_or(u64::MAX)
        {
            return Err(format!(
                "GTC owner receipt failed the parent batch check for {}",
                expected_owner.as_str()
            ));
        }
    }
    Ok(receipts)
}

fn validate_source_receipt_generations(
    receipts: &[GtcOwnerReceipt],
    before: &ProgrammeOwnerRoots<'_>,
    after: &SourceFrameAdvance,
    output_before_generation: u64,
    output_after_generation: u64,
) -> Result<(), String> {
    let expected = [
        (
            GtcOwnerId::ReaderEof,
            before.reader_eof.generation(),
            after.reader_eof_root.generation(),
        ),
        (
            GtcOwnerId::FrontendControl,
            before.main.frontend_control().generation(),
            after.main_roots.frontend_control().generation(),
        ),
        (
            GtcOwnerId::SpatialControl,
            before.main.spatial_control().generation(),
            after.main_roots.spatial_control().generation(),
        ),
        (
            GtcOwnerId::SpatialAudio,
            before.main.spatial_audio().generation(),
            after.main_roots.spatial_audio().generation(),
        ),
        (
            GtcOwnerId::Joc,
            before.joc.generation(),
            after.joc_state.generation(),
        ),
        (
            GtcOwnerId::Meter,
            before.meter.meter_callback_index().value(),
            before.meter.meter_callback_index().value(),
        ),
        (
            GtcOwnerId::Drc,
            drc_generation(&before.drc)?,
            drc_generation(&after.drc_root)?,
        ),
        (
            GtcOwnerId::Analysis,
            analysis_generation(&before.analysis)?,
            analysis_generation(&after.analysis_root)?,
        ),
        (
            GtcOwnerId::Codec,
            before.codec.generation().value(),
            after.codec_root.generation().value(),
        ),
        (
            GtcOwnerId::Metadata,
            before.metadata.state_reference().generation(),
            after.metadata_owner.state_reference().generation(),
        ),
        (
            GtcOwnerId::Integrity,
            before.integrity.committed_state().generation(),
            after.integrity_root.committed_state().generation(),
        ),
        (
            GtcOwnerId::Output,
            output_before_generation,
            output_after_generation,
        ),
    ];
    for (owner, before_generation, after_generation) in expected {
        let receipt = receipts
            .iter()
            .find(|receipt| receipt.owner_id() == owner)
            .ok_or_else(|| format!("GTC owner receipt missing {}", owner.as_str()))?;
        if receipt.before_generation() != before_generation
            || receipt.after_generation() != after_generation
        {
            return Err(format!(
                "GTC owner receipt generation mismatch for {}",
                owner.as_str()
            ));
        }
    }
    Ok(())
}

fn analysis_generation(root: &AnalysisOwnerRoot) -> Result<u64, String> {
    root.owner_generation()
        .ok_or_else(|| "Analysis leaves do not share one owner generation".to_owned())
}

fn drc_generation(root: &NoneDrcOwnerRoot) -> Result<u64, String> {
    root.state()
        .source_generation()
        .map(|generation| generation.value())
        .map_err(|error| format!("DRC owner generation: {error:?}"))
}

#[cfg(test)]
mod tests {
    use super::{CALLBACK_OWNERS, OWNER_ORDER, callback_transaction_id, validate_receipts};
    use jocforge_application::{
        independent_meter_callback::GtcOwnerReceipt, reader_eof_callback::GtcOwnerId,
    };
    use jocforge_domain::transaction::GtcOwnerStateSnapshot;
    use jocforge_io::pcm::frontend_pcm_source::FrontendCallbackIndex;

    fn receipt(
        owner: GtcOwnerId,
        changed: bool,
        transaction: &super::CallbackTransactionId,
    ) -> GtcOwnerReceipt {
        let before_bytes = [0_u8];
        let after_bytes = [u8::from(changed)];
        let before =
            GtcOwnerStateSnapshot::from_canonical_bytes(owner, 0, [0_u8; 32], &before_bytes)
                .unwrap();
        let after = GtcOwnerStateSnapshot::from_canonical_bytes(
            owner,
            u64::from(changed),
            [0_u8; 32],
            if changed { &after_bytes } else { &before_bytes },
        )
        .unwrap();
        GtcOwnerReceipt::from_transition(before, after, transaction.clone()).unwrap()
    }

    #[test]
    fn callback_parent_requires_all_twelve_unique_owners_and_emits_ascii_order() {
        let transaction = callback_transaction_id(FrontendCallbackIndex::new(0)).unwrap();
        let receipts = OWNER_ORDER
            .into_iter()
            .map(|owner| receipt(owner, CALLBACK_OWNERS.contains(&owner), &transaction))
            .rev()
            .collect();
        let canonical = validate_receipts(receipts, &transaction, |owner| {
            CALLBACK_OWNERS.contains(&owner)
        })
        .unwrap();
        assert_eq!(canonical.len(), 12);
        assert_eq!(
            canonical
                .iter()
                .map(|receipt| receipt.owner_id())
                .collect::<Vec<_>>(),
            OWNER_ORDER,
        );
    }

    #[test]
    fn callback_parent_rejects_missing_duplicate_and_wrong_transaction_receipts() {
        let transaction = callback_transaction_id(FrontendCallbackIndex::new(1)).unwrap();
        let full = OWNER_ORDER
            .into_iter()
            .map(|owner| receipt(owner, CALLBACK_OWNERS.contains(&owner), &transaction))
            .collect::<Vec<_>>();

        assert!(
            validate_receipts(
                full.iter().take(11).cloned().collect(),
                &transaction,
                |owner| CALLBACK_OWNERS.contains(&owner),
            )
            .is_err()
        );

        let mut duplicate = full.clone();
        duplicate[11] = duplicate[0].clone();
        assert!(
            validate_receipts(duplicate, &transaction, |owner| CALLBACK_OWNERS
                .contains(&owner),)
            .is_err()
        );

        let mut wrong_transaction = full;
        wrong_transaction[0] = receipt(
            OWNER_ORDER[0],
            CALLBACK_OWNERS.contains(&OWNER_ORDER[0]),
            &super::CallbackTransactionId::new("different-callback").unwrap(),
        );
        assert!(
            validate_receipts(wrong_transaction, &transaction, |owner| CALLBACK_OWNERS
                .contains(&owner),)
            .is_err()
        );
    }
}
