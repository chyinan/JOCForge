// pattern: Imperative Shell
//! Composes one real MAIN source callback through F07-F15 as a noncommitting root proposal.

use jocforge_application::{
    PreparedJobInput,
    drc_source_bridge::{
        CallbackLineageIdentity, NoneDrcOwnerRoot, prepare_drc_source_from_analysis,
    },
    independent_meter_callback::{AttemptId, CallbackTransactionId, GtcOwnerReceipt},
    integrity_source_bridge::{
        F14kPayloadOutcome, F14kSourceCandidate, IntegrityF12SourceReceipts,
        IntegrityOwnerReceiptProposal, IntegrityProgrammeOwner, protect_metadata_source_for_frame,
        stage_integrity_after_f12,
    },
    metadata_source_bridge::{
        MainMetadataBodiesInput, MetadataOwner, SelectedJocState,
        prepare_main_metadata_bodies_for_callback, stage_main_metadata_source,
    },
    meter_finalization_bridge::{
        ProgrammeMeasurementReceipt, StandaloneMeterRoot, unchanged_meter_root_receipt,
    },
    meter_source_bridge::{Flat7xI0D0TargetPlan, JocReconstructionLayout, ResolvedEncodingProfile},
    output_publication::{
        AdmittedProductionPlan, OutputDurableAppendReceipt, OutputSourceAfterProposal,
        ProgrammeOutputBinding, prepare_output_source_from_candidate,
    },
    reader_eof_callback::{GtcOwnerId, ReaderEofMachine, ReaderEofReplacementCandidate},
    source_analysis_bridge::{
        AnalysisOwnerRoot, SelectedSourceAnalysisProviders, SpatialSourceAnalysisBridgeInput,
        prepare_source_analysis_from_spatial_callback,
    },
    source_frame_bridge::{
        CodecOwnerRoot, Flat7xSourcePcmProjection, build_flat7x_source_pcm_projection,
        prepare_admitted_source_frame,
    },
    spatial_callback::{MainSpatialCallbackCandidate, MainSpatialColdRoots},
};

use jocforge_application::flat7x_pair_source::{
    Flat7xD0AudioSpanInput, Flat7xD0AudioSpanProducerState, Flat7xPairSourceError,
    bind_flat7x_pair_source, require_flat7x_d0_audio_blocks,
};

use crate::{
    output_publication::{
        ProgrammeOutputStaging, SourceFrameOutputStageError, stage_source_frame_output,
    },
    programme_roots::source_for_callback_clock,
    selected_spatial_callback_providers::SelectedMainSpatialCallbackProviders,
};
use jocforge_io::pcm::frontend_pcm_source::{CodecSourceIndex, FrontendCallbackIndex};

/// Receipt summary for one admitted durable append.
pub(crate) struct DurableAppendSummary {
    pub au_index: u64,
    pub bytes_written: u32,
    pub crc_residual: u16,
    pub file_fsync: bool,
    pub durable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Flat7xPcmRowDelivery {
    NormalCallback,
    CachedDrain,
}

/// Physical RIFF PCM rows carried to the Flat7x source stage with their callback/source lineage.
pub(crate) struct Flat7xPcmSourceRows {
    input_sha256: [u8; 32],
    callback_index: FrontendCallbackIndex,
    source_index: CodecSourceIndex,
    delivery: Flat7xPcmRowDelivery,
    physical_pcm_rows: Vec<Vec<u32>>,
}

impl Flat7xPcmSourceRows {
    pub(crate) fn new(
        input_sha256: [u8; 32],
        callback_index: FrontendCallbackIndex,
        source_index: CodecSourceIndex,
        delivery: Flat7xPcmRowDelivery,
        physical_pcm_rows: Vec<Vec<u32>>,
    ) -> Self {
        Self {
            input_sha256,
            callback_index,
            source_index,
            delivery,
            physical_pcm_rows,
        }
    }

    pub const fn input_sha256(&self) -> &[u8; 32] {
        &self.input_sha256
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn delivery(&self) -> Flat7xPcmRowDelivery {
        self.delivery
    }

    pub fn physical_pcm_rows(&self) -> &[Vec<u32>] {
        &self.physical_pcm_rows
    }
}

/// Complete same-source after-values staged before a full programme parent replacement.
#[must_use]
pub(crate) struct SourceFrameAdvance<'pcm> {
    pub callback_index: FrontendCallbackIndex,
    pub source_transaction_id: CallbackTransactionId,
    pub main_roots: MainSpatialColdRoots,
    pub reader_eof_root: ReaderEofMachine,
    pub integrity_root: IntegrityProgrammeOwner,
    pub analysis_root: AnalysisOwnerRoot,
    pub drc_root: NoneDrcOwnerRoot,
    pub metadata_owner: MetadataOwner,
    pub joc_state: SelectedJocState,
    pub codec_root: CodecOwnerRoot,
    pub output_after: OutputSourceAfterProposal,
    pub flat7x_source_pcm_projection: Option<Flat7xSourcePcmProjection<'pcm>>,
    pub source_index: CodecSourceIndex,
    pub frame_sha256: [u8; 32],
    pub metadata_container_bytes: Option<usize>,
    pub f14k_source: F14kSourceCandidate,
    pub integrity_receipts: IntegrityF12SourceReceipts,
    pub integrity_owner_receipt: IntegrityOwnerReceiptProposal,
    pub append: Option<DurableAppendSummary>,
    pub owner_receipts: Vec<GtcOwnerReceipt>,
}

/// Stages F07-F15 for one actual source index without installing any owner root.
#[allow(clippy::too_many_arguments)]
pub(crate) fn stage_source_frame<'pcm>(
    prepared: &PreparedJobInput,
    joc_profile: ResolvedEncodingProfile,
    attempt: AttemptId,
    callback_index: FrontendCallbackIndex,
    source_index: CodecSourceIndex,
    main_callback_transaction: CallbackTransactionId,
    frontend_candidate: Option<
        jocforge_application::meter_source_bridge::FrontendControlReplacementCandidate,
    >,
    spatial_callback: MainSpatialCallbackCandidate,
    reader_eof_candidate: ReaderEofReplacementCandidate,
    main_roots_before: &MainSpatialColdRoots,
    reader_eof_before: &ReaderEofMachine,
    integrity_before: &IntegrityProgrammeOwner,
    main_providers: &SelectedMainSpatialCallbackProviders,
    meter_root: &StandaloneMeterRoot,
    measurement: &ProgrammeMeasurementReceipt,
    analysis_before: &AnalysisOwnerRoot,
    analysis_providers: &SelectedSourceAnalysisProviders,
    drc_before: &NoneDrcOwnerRoot,
    metadata_before: &MetadataOwner,
    joc_before: &SelectedJocState,
    codec_before: &CodecOwnerRoot,
    output_binding_before: &ProgrammeOutputBinding,
    admitted_plan: &AdmittedProductionPlan,
    staging: &mut ProgrammeOutputStaging,
    flat7x_physical_pcm_rows: Option<&'pcm Flat7xPcmSourceRows>,
) -> Result<SourceFrameAdvance<'pcm>, String> {
    if source_for_callback_clock(callback_index) != Some(source_index)
        || spatial_callback.callback_index() != callback_index
        || main_callback_transaction.as_str().is_empty()
    {
        return Err("callback/source lineage does not match the selected clock map".to_owned());
    }
    let source_transaction_id =
        CallbackTransactionId::new(&format!("jocforge-source-{}", source_index.value()))
            .map_err(|error| format!("source transaction identity: {error:?}"))?;
    let callback_identity =
        CallbackLineageIdentity::new(prepared.programme_id().clone(), attempt, callback_index);
    let meter_receipt = unchanged_meter_root_receipt(
        prepared,
        meter_root,
        measurement,
        main_callback_transaction.clone(),
    )
    .map_err(|error| format!("Meter unchanged callback receipt: {error:?}"))?;

    let analysis_box =
        prepare_source_analysis_from_spatial_callback(SpatialSourceAnalysisBridgeInput {
            prepared,
            analysis_before,
            spatial_callback: &spatial_callback,
            attempt_id: attempt,
            source_transaction_id: source_transaction_id.clone(),
            previous_short: *codec_before.previous_short(),
            providers: analysis_providers.providers(),
        })
        .map_err(|error| format!("F07-F10 source stage: {error:?}"))?;
    let mut analysis_stage = *analysis_box;
    let flat7x_source_pcm_projection =
        if joc_profile.joc_reconstruction_layout() == JocReconstructionLayout::Flat7x {
            let pcm_source = flat7x_physical_pcm_rows.ok_or_else(|| {
                "Flat7x source stage requires the callback's physical PCM rows".to_owned()
            })?;
            let expected_input_sha = prepared.resource_identity().input_sha256();
            let expected_delivery = if u64::from(callback_index.value())
                < prepared.configuration().source_samples().div_ceil(1_536)
            {
                Flat7xPcmRowDelivery::NormalCallback
            } else {
                Flat7xPcmRowDelivery::CachedDrain
            };
            if pcm_source.input_sha256() != &expected_input_sha
                || pcm_source.callback_index() != callback_index
                || pcm_source.source_index() != source_index
                || pcm_source.delivery() != expected_delivery
            {
                return Err(
                    "Flat7x PCM rows do not match the current callback/source lineage".to_owned(),
                );
            }
            let shared_analysis = analysis_stage.shared_analysis().ok_or_else(|| {
                "Flat7x role map requires the current shared source analysis".to_owned()
            })?;
            Some(
                build_flat7x_source_pcm_projection(
                    prepared,
                    Flat7xI0D0TargetPlan::approved_contract(),
                    &callback_identity,
                    source_index,
                    &source_transaction_id,
                    shared_analysis.identity(),
                    pcm_source.physical_pcm_rows(),
                )
                .map_err(|error| format!("Flat7x source PCM projection: {error:?}"))?,
            )
        } else {
            None
        };

    let drc_source =
        prepare_drc_source_from_analysis(prepared, drc_before, &analysis_stage, &callback_identity)
            .map_err(|error| format!("F06ND/F06NR source stage: {error}"))?;
    let metadata_bodies = prepare_main_metadata_bodies_for_callback(
        MainMetadataBodiesInput {
            prepared,
            joc_profile,
            spatial_callback: &spatial_callback,
            joc_before,
            source_transaction_id: source_transaction_id.clone(),
            qmf_functional: main_providers.spatial_audio().qmf_receiver_functional(),
            fit_score: admitted_plan.source_providers().fit_score(),
            retry: admitted_plan.source_providers().representation_retry(),
        },
        main_callback_transaction.clone(),
    )
    .map_err(|error| format!("F05/F13 body stage: {error:?}"))?;
    let metadata_candidate = stage_main_metadata_source(
        metadata_before,
        metadata_bodies,
        source_transaction_id.clone(),
    )
    .map_err(|error| format!("F13 source stage: {error:?}"))?;
    let (joc_after, joc_receipt, mut metadata_source) = metadata_candidate.into_parts();
    let f14k_source = protect_metadata_source_for_frame(
        &mut metadata_source,
        admitted_plan.source_providers().keyless_integrity(),
    )
    .map_err(|error| format!("F14/F14K source stage: {error:?}"))?;
    if f14k_source.source_index() != source_index
        || f14k_source.dispatch() != metadata_source.receipt().dispatch()
        || f14k_source.au_index() != metadata_source.receipt().au_index()
    {
        return Err("F14K receipt does not match the current source/AU lineage".to_owned());
    }
    let metadata_container_bytes = match f14k_source.outcome() {
        F14kPayloadOutcome::AbsentPublishedPayloadV1 => None,
        F14kPayloadOutcome::ProtectedEmdf {
            container_bytes, ..
        } => Some(container_bytes),
    };

    let frame_candidate = prepare_admitted_source_frame(
        prepared,
        admitted_plan,
        &mut analysis_stage,
        drc_source.after_root(),
        &metadata_source,
    )
    .map_err(|error| format!("F12 frame candidate: {error:?}"))?;
    let flat7x_pair_source_binding = match flat7x_source_pcm_projection.as_ref() {
        Some(projection) => Some(
            bind_flat7x_pair_source(
                joc_profile,
                projection,
                frame_candidate.numeric_plan_receipt(),
            )
            .map_err(|error: Flat7xPairSourceError| {
                format!("Flat7x I0/D0 source binding: {error:?}")
            })?,
        ),
        None => None,
    };
    if let Some(binding) = flat7x_pair_source_binding {
        require_flat7x_d0_audio_blocks(
            Flat7xD0AudioSpanInput::Disabled(
                Flat7xD0AudioSpanProducerState::NoApprovedAudioProductionRule,
            ),
            binding.d0_source_identity(),
        )
        .map_err(|error: Flat7xPairSourceError| {
            format!(
                "Flat7x paired source {} awaits D0 audio spans: {error:?}",
                binding.source_index().value()
            )
        })?;
        return Err("Flat7x pair candidate cannot enter single-frame output staging".to_owned());
    }
    let integrity_candidate = stage_integrity_after_f12(
        integrity_before,
        &metadata_source,
        &f14k_source,
        &frame_candidate,
        admitted_plan.source_providers().keyless_integrity(),
        prepared.programme_id(),
        attempt,
        &source_transaction_id,
    )
    .map_err(|error| format!("F14/F12 Integrity owner stage: {error:?}"))?;
    let (integrity_root, integrity_receipts, integrity_owner_receipt) =
        integrity_candidate.into_parts();
    let output_proposal = prepare_output_source_from_candidate(
        prepared,
        &output_binding_before,
        &metadata_source,
        &frame_candidate,
    )
    .map_err(|error| format!("F15 output proposal: {error}"))?;
    let staged_output = stage_source_frame_output(
        staging,
        output_binding_before,
        codec_before,
        &frame_candidate,
        output_proposal,
    )
    .map_err(|error: SourceFrameOutputStageError| format!("F12/F15 source output: {error}"))?;
    let (output_after, codec_proposal) = staged_output.into_parts();
    let frame_sha256 = *codec_proposal.frame_sha256();
    let codec_receipt = codec_proposal
        .gtc_receipt(codec_before, main_callback_transaction.clone())
        .map_err(|error| format!("Codec owner receipt: {error:?}"))?;
    let output_receipt = output_binding_before
        .gtc_receipt_after_source(&output_after, main_callback_transaction.clone())
        .map_err(|error| format!("Output owner receipt: {error}"))?;
    let append = output_after.durable_append_receipt().map(to_append_summary);
    drop(frame_candidate);

    let spatial_parent = match frontend_candidate {
        Some(frontend) => main_roots_before
            .prepare_spatial_after_callback(
                prepared,
                frontend,
                spatial_callback,
                main_callback_transaction.clone(),
            )
            .map_err(|error| format!("F01E/F03W/F04 parent proposal: {error:?}"))?,
        None => {
            let candidate_callback_index = spatial_callback.callback_index().value();
            let source_lineage_callback_index =
                spatial_callback.transaction_id().callback_index().value();
            let frontend_callback_index = main_roots_before
                .frontend_control()
                .committed()
                .callback_index()
                .value();
            let spatial_control_callback_index = main_roots_before
                .spatial_control()
                .committed()
                .callback_index()
                .value();
            let spatial_audio_callback_index = main_roots_before
                .spatial_audio()
                .committed()
                .callback_index()
                .value();
            main_roots_before
                .prepare_spatial_after_cached_callback(
                    prepared,
                    spatial_callback,
                    main_callback_transaction.clone(),
                )
                .map_err(|error| {
                    format!(
                        "cached F01E/F03W/F04 parent proposal: {error:?}; phase=cached_drain callback_index={} candidate_callback_index={} source_lineage_callback_index={} frontend_callback_index={} spatial_control_callback_index={} spatial_audio_callback_index={} source_index={}",
                        callback_index.value(),
                        candidate_callback_index,
                        source_lineage_callback_index,
                        frontend_callback_index,
                        spatial_control_callback_index,
                        spatial_audio_callback_index,
                        source_index.value(),
                    )
                })?
        }
    };
    let (main_roots, _, _, spatial_receipts) = spatial_parent.into_parts();

    if reader_eof_candidate.owner_receipt().transaction_id() != main_callback_transaction.as_str()
        || reader_eof_candidate.owner_receipt().before_generation()
            != reader_eof_before.generation()
        || reader_eof_candidate.gtc_receipt().owner_id() != GtcOwnerId::ReaderEof
    {
        return Err("ReaderEof receipt is not bound to the callback before-root".to_owned());
    }
    let (reader_eof_root, _, reader_eof_receipt) = reader_eof_candidate.into_parts();

    let analysis_shared = analysis_stage
        .shared_analysis()
        .ok_or_else(|| "Analysis parent proposal: SharedAnalysisMissing".to_owned())?;
    let analysis_source_key = analysis_shared.identity().source_stage_key();
    let actual_analysis_source_transaction = analysis_source_key.transaction_id().clone();
    let analysis_before_generation = analysis_source_key.analysis_before_generation().value();
    let analysis_after_generation = analysis_shared
        .identity()
        .analysis_after_generation()
        .value();
    let analysis_parent = analysis_stage
        .into_parent_proposal_for_parent_callback(
            analysis_before,
            &source_transaction_id,
            main_callback_transaction.clone(),
        )
        .map_err(|error| {
            format!(
                "Analysis parent proposal: {error:?}; owner=Analysis source_transaction_expected={} source_transaction_actual={} parent_callback_transaction={} callback_index={} source_index={} analysis_generation={}->{}",
                source_transaction_id.as_str(),
                actual_analysis_source_transaction.as_str(),
                main_callback_transaction.as_str(),
                callback_index.value(),
                source_index.value(),
                analysis_before_generation,
                analysis_after_generation,
            )
        })?;
    let (analysis_root, _shared_analysis, analysis_receipt) = analysis_parent.into_parts();

    let (_, _, _, _, drc_after) = drc_source.into_parts();
    let drc_source_key = drc_after.shared_analysis_identity().source_stage_key();
    let actual_drc_source_transaction = drc_source_key.transaction_id().clone();
    let drc_source_index = drc_after.source_index();
    let drc_callback_index = drc_after.callback_identity().callback_index();
    let drc_before_generation = drc_before
        .state()
        .source_generation()
        .map_err(|error| format!("DRC before generation: {error:?}"))?
        .value();
    let drc_after_generation = drc_after
        .state()
        .source_generation()
        .map_err(|error| format!("DRC after generation: {error:?}"))?
        .value();
    let drc_parent = drc_before
        .prepare_replacement_for_parent_callback(
            drc_after,
            &source_transaction_id,
            main_callback_transaction.clone(),
        )
        .map_err(|error| {
            format!(
                "DRC parent proposal: {error:?}; owner=DRC source_transaction_expected={} source_transaction_actual={} parent_callback_transaction={} callback_index={} source_index={} DRC_generation={}->{}",
                source_transaction_id.as_str(),
                actual_drc_source_transaction.as_str(),
                main_callback_transaction.as_str(),
                u64::from(drc_callback_index.value()),
                drc_source_index.value(),
                drc_before_generation,
                drc_after_generation,
            )
        })?;
    let (drc_root, drc_receipt) = drc_parent.into_parts();

    let metadata_receipt_before_handoff = metadata_source.receipt();
    let actual_metadata_consumer_transaction = metadata_source.consumer_transaction_id().cloned();
    let metadata_source_index = metadata_receipt_before_handoff.source_index();
    let metadata_dispatch = metadata_receipt_before_handoff.dispatch();
    let metadata_au_index = metadata_receipt_before_handoff.au_index();
    let metadata_before_generation = metadata_before.state_reference().generation();
    let metadata_after_generation = metadata_receipt_before_handoff.proposed_after_generation();
    let metadata_parent = metadata_source
        .into_parent_handoff_for_parent_callback(
            metadata_before,
            &source_transaction_id,
            main_callback_transaction.clone(),
        )
        .map_err(|error| {
            format!(
                "Metadata parent proposal: {error:?}; owner=Metadata consumer_transaction_expected={} consumer_transaction_actual={} parent_callback_transaction={} callback_index={} source_index={} dispatch={} au_index={:?} metadata_generation={}->{}",
                source_transaction_id.as_str(),
                actual_metadata_consumer_transaction
                    .as_ref()
                    .map_or("<none>", CallbackTransactionId::as_str),
                main_callback_transaction.as_str(),
                callback_index.value(),
                metadata_source_index.value(),
                metadata_dispatch.value(),
                metadata_au_index.map(|value| value.value()),
                metadata_before_generation,
                metadata_after_generation,
            )
        })?;
    let (metadata_owner, _, metadata_receipt, _, _, _, _, _) = metadata_parent.into_parts();
    let (joc_state, _, _) = joc_after.into_parts();

    let mut owner_receipts = spatial_receipts.to_vec();
    owner_receipts.extend([
        analysis_receipt,
        drc_receipt,
        metadata_receipt,
        joc_receipt,
        meter_receipt,
        codec_receipt,
        output_receipt,
        reader_eof_receipt,
    ]);

    Ok(SourceFrameAdvance {
        callback_index,
        source_transaction_id,
        main_roots,
        reader_eof_root,
        integrity_root,
        analysis_root,
        drc_root,
        metadata_owner,
        joc_state,
        codec_root: *codec_proposal.after_state(),
        output_after,
        flat7x_source_pcm_projection,
        source_index,
        frame_sha256,
        metadata_container_bytes,
        f14k_source,
        integrity_receipts,
        integrity_owner_receipt,
        append,
        owner_receipts,
    })
}

fn to_append_summary(receipt: &OutputDurableAppendReceipt) -> DurableAppendSummary {
    DurableAppendSummary {
        au_index: receipt.au_index().value(),
        bytes_written: receipt.bytes_written(),
        crc_residual: receipt.crc_residual(),
        file_fsync: receipt.file_fsync(),
        durable: receipt.durable(),
    }
}
