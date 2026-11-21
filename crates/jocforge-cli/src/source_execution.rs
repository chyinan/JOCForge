// pattern: Imperative Shell
//! Composes one real MAIN source callback through F07-F15 as a noncommitting root proposal.

use std::sync::Arc;

use crate::{
    output_publication::{
        ProgrammeOutputStaging, SourceFrameOutputStageError, stage_source_frame_output,
        stage_source_frame_output_with_d0_history,
    },
    programme_roots::source_for_callback_clock,
    selected_spatial_callback_providers::SelectedMainSpatialCallbackProviders,
};
use jocforge_application::flat7x_pair_source::{
    CallerOwnedI0CrcrsvSeed, D0Block2PlacementReceipt, Flat7xD0BapParameters, Flat7xD0BsiStructure,
    Flat7xD0PcmFrameSettings, Flat7xI0AudioBlockSeriesError, Flat7xI0FrameAssemblyError,
    Flat7xPairSourceError, I0AudioBlockBudgetProbe, I0AudioBlockSeriesBudget, I0AuxDataInput,
    I0BapCandidateQ, assemble_flat7x_i0_frame_candidate_from_binding,
    bind_flat7x_d0_block2_source_receipt, bind_flat7x_pair_source, flat7x_i0_audio_block_policy_v1,
    produce_flat7x_pair_d0_pcm_frame_candidate, stage_flat7x_pair_d0_pcm_block_series,
};
use jocforge_application::{
    PreparedJobInput,
    codec_source_bridge::bind_flat7x_d0_bsi_source_inputs,
    drc_source_bridge::{
        CallbackLineageIdentity, NoneDrcOwnerRoot, prepare_drc_source_from_analysis,
    },
    flat7x_pair_integrity::{Flat7xPairF12StageInput, stage_flat7x_pair_f12_integrity},
    flat7x_pair_output::{Flat7xSourceGeneration, stage_flat7x_pcm_pair_for_f15},
    independent_meter_callback::{AttemptId, CallbackTransactionId, GtcOwnerReceipt},
    integrity_source_bridge::{
        F14kPayloadOutcome, F14kSourceCandidate, IntegrityF12SourceReceipts,
        IntegrityOwnerReceiptProposal, IntegrityProgrammeOwner,
        protect_metadata_source_for_frame_with_source_generation, stage_integrity_after_f12,
    },
    metadata_source_bridge::{
        MainMetadataBodiesInput, MetadataOwner, SelectedJocState,
        prepare_main_metadata_bodies_for_callback,
        prepare_main_metadata_bodies_for_callback_with_seven_carrier_fit,
        stage_main_metadata_source,
    },
    meter_finalization_bridge::{
        MeterOwnerStateReference, ProgrammeMeasurementReceipt, unchanged_meter_reference_receipt,
    },
    meter_source_bridge::{Flat7xI0D0TargetPlan, ResolvedEncodingProfile},
    output_publication::{
        AdmittedProductionPlan, OutputDurableAppendReceipt, OutputSourceAfterProposal,
        ProgrammeOutputBinding, prepare_output_source_for_flat7x_suppressed,
        prepare_output_source_from_candidate, prepare_output_source_from_flat7x_pair,
    },
    reader_eof_callback::ReaderEofMachine,
    source_analysis_bridge::{
        AnalysisOwnerRoot, SelectedSourceAnalysisProviders, SpatialSourceAnalysisBridgeInput,
        prepare_source_analysis_on_current_thread,
    },
    source_frame_bridge::{
        CodecOwnerRoot, Flat7xD0PcmSourceExtent, Flat7xSceneMix, Flat7xSourcePcmProjection,
        SevenCarrierQmfState, bind_flat7x_source_role_map, installed_audio_from_flat7x_i0_rows,
        prepare_admitted_source_frame, stage_codec_owner_after_flat7x_pair,
        stage_flat7x_seven_carrier_coefficient_fit, stage_flat7x_seven_carrier_windows,
    },
    spatial_callback::MainSpatialColdRoots,
};
use jocforge_io::pcm::frontend_pcm_source::{CodecSourceIndex, FrontendCallbackIndex};

const FLAT7X_D0_OPTIONAL_FRAME_FIELDS_POLICY_ID: &str =
    "JOCFORGE_FLAT7X_D0_OPTIONAL_FRAME_FIELDS_V1";
const FLAT7X_SOURCE_GENERATION_MODEL_ID: &str = "JOCFORGE_FLAT7X_SOURCE_GENERATION_SOURCE_INDEX_V1";

/// Receipt summary for one admitted durable append.
pub(crate) struct DurableAppendSummary {
    pub au_index: u64,
    pub bytes_written: u32,
    pub crc_residual: u16,
    pub file_fsync: bool,
    pub durable: bool,
}

/// Structural bit spans retained from the first source-bound D0 frame receipt for diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Flat7xD0FrameSyntaxSummary {
    pub audfrm_start_bit: u32,
    pub audfrm_end_bit: u32,
    pub skipflde_bit_offset: u32,
    pub block_two_skiple_bit_offset: u32,
    pub block_two_skipl_bit_offset: u32,
    pub block_two_container_bytes: u16,
    pub block_two_container_start_bit: u32,
    pub block_two_container_end_bit: u32,
    pub nauxbits_start_bit: u32,
    pub nauxbits_end_bit: u32,
    pub auxdatae_bit_offset: u32,
    pub crcrsv_bit_offset: u32,
    pub crc2_start_bit: u32,
    pub crc2_end_bit: u32,
}

pub(crate) struct Flat7xCarrierFrame {
    pub input_sha256: [u8; 32],
    pub source_extent: Flat7xD0PcmSourceExtent,
    pub mix: Flat7xSceneMix,
}

/// Complete same-source after-values staged before a full programme parent replacement.
#[must_use]
pub(crate) struct SourceFrameAdvance<'pcm> {
    pub callback_index: FrontendCallbackIndex,
    pub source_transaction_id: CallbackTransactionId,
    pub main_roots: Arc<MainSpatialColdRoots>,
    pub reader_eof_root: Arc<ReaderEofMachine>,
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
    pub flat7x_d0_frame_syntax: Option<Flat7xD0FrameSyntaxSummary>,
    pub flat7x_d0_block2_placement_receipt: Option<D0Block2PlacementReceipt>,
    pub f14k_source: F14kSourceCandidate,
    pub integrity_receipts: IntegrityF12SourceReceipts,
    pub flat7x_integrity_pair_receipts:
        Option<jocforge_application::flat7x_pair_integrity::Flat7xI0D0F12PairReceipts>,
    pub integrity_owner_receipt: IntegrityOwnerReceiptProposal,
    pub append: Option<DurableAppendSummary>,
    pub owner_receipts: Vec<GtcOwnerReceipt>,
}

#[derive(Debug)]
pub(crate) enum SourceFrameStageError {
    Pipeline(String),
}

impl From<String> for SourceFrameStageError {
    fn from(message: String) -> Self {
        Self::Pipeline(message)
    }
}

/// Stages F07-F15 for one actual source index without installing any owner root.
#[allow(clippy::too_many_arguments)]
#[expect(
    clippy::needless_pass_by_value,
    reason = "The parent callback identity is transferred through this one source stage"
)]
#[expect(
    clippy::redundant_closure_for_method_calls,
    reason = "The CLI does not depend directly on the domain crate that owns this AU index"
)]
pub(crate) fn stage_source_frame<'pcm>(
    prepared: &PreparedJobInput,
    joc_profile: ResolvedEncodingProfile,
    attempt: AttemptId,
    callback_index: FrontendCallbackIndex,
    source_index: CodecSourceIndex,
    main_callback_transaction: CallbackTransactionId,
    callback: crate::encode_request::SpatialFrameAdvance,
    integrity_before: &IntegrityProgrammeOwner,
    main_providers: &SelectedMainSpatialCallbackProviders,
    meter_root: &MeterOwnerStateReference,
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
    flat7x_carriers: Option<&'pcm Flat7xCarrierFrame>,
) -> Result<SourceFrameAdvance<'pcm>, SourceFrameStageError> {
    let spatial_callback = &callback.spatial;
    if source_for_callback_clock(callback_index) != Some(source_index)
        || spatial_callback.callback_index() != callback_index
        || main_callback_transaction.as_str().is_empty()
    {
        return Err(SourceFrameStageError::Pipeline(
            "callback/source lineage does not match the selected clock map".to_owned(),
        ));
    }
    let source_transaction_id =
        CallbackTransactionId::new(&format!("jocforge-source-{}", source_index.value()))
            .map_err(|error| format!("source transaction identity: {error:?}"))?;
    let callback_identity =
        CallbackLineageIdentity::new(prepared.programme_id().clone(), attempt, callback_index);
    let meter_receipt = unchanged_meter_reference_receipt(
        prepared,
        meter_root,
        measurement,
        main_callback_transaction.clone(),
    )
    .map_err(|error| format!("Meter unchanged callback receipt: {error:?}"))?;

    let flat7x_callback_pcm = if joc_profile
        .joc_reconstruction_layout()
        .uses_seven_carriers()
    {
        let pcm_source = flat7x_carriers.ok_or_else(|| {
            "Flat7x source stage requires the callback's rendered carriers".to_owned()
        })?;
        let expected_input_sha = prepared.resource_identity().input_sha256();
        let source_extent = pcm_source.source_extent;
        if pcm_source.input_sha256 != expected_input_sha
            || source_extent.callback_index() != callback_index
            || source_extent.source_index() != source_index
        {
            return Err(SourceFrameStageError::Pipeline(
                "Flat7x programme window identity mismatch".to_owned(),
            ));
        }
        let i0_analysis_input = installed_audio_from_flat7x_i0_rows(
            source_index,
            pcm_source.mix.pcm_view().i0().rows(),
        )
        .map_err(|error| format!("Flat7x I0 scene mix: {error:?}"))?;
        Some((pcm_source, source_extent, i0_analysis_input))
    } else {
        None
    };

    let analysis_input = SpatialSourceAnalysisBridgeInput {
        prepared,
        analysis_before,
        spatial_callback,
        attempt_id: attempt,
        source_transaction_id: source_transaction_id.clone(),
        previous_short: *codec_before.previous_short(),
        providers: analysis_providers.providers(),
    };
    let installed_audio = flat7x_callback_pcm.as_ref().map(|(_, _, audio)| audio);
    let analysis_box = prepare_source_analysis_on_current_thread(analysis_input, installed_audio)
        .map_err(|error| format!("F07-F10 source stage: {error:?}"))?;
    let mut analysis_stage = *analysis_box;
    let flat7x_source_pcm_projection =
        if let Some((pcm_source, source_extent, _)) = flat7x_callback_pcm.as_ref() {
            let shared_analysis = analysis_stage.shared_analysis().ok_or_else(|| {
                "Flat7x role map requires the current shared source analysis".to_owned()
            })?;
            let receipt = bind_flat7x_source_role_map(
                prepared,
                Flat7xI0D0TargetPlan::for_layout(
                    joc_profile.codec_rate(),
                    joc_profile.joc_reconstruction_layout(),
                )
                .ok_or_else(|| "unsupported Flat7x rate".to_owned())?,
                &callback_identity,
                source_index,
                &source_transaction_id,
                shared_analysis.identity(),
            )
            .map_err(|error| format!("Flat7x carrier role map: {error:?}"))?;
            let projection =
                Flat7xSourcePcmProjection::from_scene_mix(receipt, *source_extent, &pcm_source.mix)
                    .map_err(|error| format!("Flat7x scene mix projection: {error:?}"))?;
            Some(projection)
        } else {
            None
        };

    let make_dynamics = || {
        let drc_source = prepare_drc_source_from_analysis(
            prepared,
            drc_before,
            &analysis_stage,
            &callback_identity,
        )
        .map_err(|error| format!("F06ND/F06NR source stage: {error}"))?;
        let flat7x_d0_bsi_inputs = if flat7x_source_pcm_projection.is_some() {
            let shared_analysis = analysis_stage.shared_analysis().ok_or_else(|| {
                SourceFrameStageError::Pipeline(
                    "Flat7x D0 BSI binding requires same-source shared analysis".to_owned(),
                )
            })?;
            Some(
                bind_flat7x_d0_bsi_source_inputs(
                    prepared,
                    shared_analysis,
                    drc_source.after_root(),
                    Flat7xI0D0TargetPlan::for_layout(
                        joc_profile.codec_rate(),
                        joc_profile.joc_reconstruction_layout(),
                    )
                    .ok_or_else(|| "unsupported Flat7x rate".to_owned())?,
                )
                .map_err(|error| {
                    SourceFrameStageError::Pipeline(format!(
                        "Flat7x same-source D0 BSI controls: {error:?}"
                    ))
                })?,
            )
        } else {
            None
        };
        Ok::<_, SourceFrameStageError>((drc_source, flat7x_d0_bsi_inputs))
    };
    let make_metadata = || {
        let flat7x_d0_source_generation_receipt =
            if let Some(projection) = flat7x_source_pcm_projection.as_ref() {
                Some(
                    codec_before
                        .issue_flat7x_d0_source_generation_receipt(
                            source_transaction_id.clone(),
                            callback_identity.clone(),
                            projection.role_map_receipt().shared_analysis_identity(),
                        )
                        .map_err(|error| format!("Flat7x source-generation receipt: {error:?}"))?,
                )
            } else {
                None
            };
        let flat7x_seven_carrier_fit_candidate =
            if let Some(source_projection) = flat7x_source_pcm_projection.as_ref() {
                let qmf_before = joc_before
                    .fit()
                    .seven_carrier_qmf_state()
                    .cloned()
                    .unwrap_or_else(SevenCarrierQmfState::cold);
                let carrier_windows = stage_flat7x_seven_carrier_windows(
                    source_projection,
                    &qmf_before,
                    main_providers.spatial_audio().qmf_receiver_functional(),
                    main_providers.spatial_audio().qmf_mathematical_fft(),
                )
                .map_err(|error| {
                    SourceFrameStageError::Pipeline(format!(
                        "Flat7x seven-carrier QMF windows: {error:?}"
                    ))
                })?;
                let target_projection = spatial_callback.joc_qmf_projection().ok_or_else(|| {
                    SourceFrameStageError::Pipeline(
                        "Flat7x seven-carrier fit requires a ready spatial target projection"
                            .to_owned(),
                    )
                })?;
                Some(
                    stage_flat7x_seven_carrier_coefficient_fit(
                        joc_profile,
                        joc_before.fit(),
                        target_projection,
                        &carrier_windows,
                        main_providers.spatial_audio().qmf_receiver_functional(),
                        admitted_plan.source_providers().fit_score(),
                    )
                    .map_err(|error| {
                        SourceFrameStageError::Pipeline(format!(
                            "Flat7x seven-carrier coefficient fit: {error:?}"
                        ))
                    })?,
                )
            } else {
                None
            };
        let metadata_input = MainMetadataBodiesInput {
            prepared,
            joc_profile,
            spatial_callback,
            joc_before,
            source_transaction_id: source_transaction_id.clone(),
            qmf_functional: main_providers.spatial_audio().qmf_receiver_functional(),
            fit_score: admitted_plan.source_providers().fit_score(),
            retry: admitted_plan.source_providers().representation_retry(),
        };
        let metadata_bodies = if let Some(fit_candidate) = flat7x_seven_carrier_fit_candidate {
            let source_generation_receipt = flat7x_d0_source_generation_receipt
                .as_ref()
                .ok_or_else(|| {
                    SourceFrameStageError::Pipeline(
                        "Flat7x source-generation receipt is required for the C=7 fit".to_owned(),
                    )
                })?;
            prepare_main_metadata_bodies_for_callback_with_seven_carrier_fit(
                metadata_input,
                main_callback_transaction.clone(),
                fit_candidate,
                source_generation_receipt,
            )
        } else {
            prepare_main_metadata_bodies_for_callback(
                metadata_input,
                main_callback_transaction.clone(),
            )
        }
        .map_err(|error| format!("F05/F13 body stage: {error:?}"))?;
        Ok::<_, SourceFrameStageError>((metadata_bodies, flat7x_d0_source_generation_receipt))
    };
    // Both read this frame's analysis, but the programme DRC and object metadata have independent histories.
    let (dynamics, metadata) = jocforge_application::join_dsp_tasks(make_dynamics, make_metadata);
    let (drc_source, flat7x_d0_bsi_inputs) = dynamics?;
    let (metadata_bodies, flat7x_d0_source_generation_receipt) = metadata?;
    let metadata_candidate = stage_main_metadata_source(
        metadata_before,
        metadata_bodies,
        source_transaction_id.clone(),
    )
    .map_err(|error| format!("F13 source stage: {error:?}"))?;
    let (joc_after, joc_receipt, mut metadata_source) = metadata_candidate.into_parts();
    let f14k_source = protect_metadata_source_for_frame_with_source_generation(
        &mut metadata_source,
        admitted_plan.source_providers().keyless_integrity(),
        flat7x_d0_source_generation_receipt.clone(),
    )
    .map_err(|error| format!("F14/F14K source stage: {error:?}"))?;
    if f14k_source.source_index() != source_index
        || f14k_source.dispatch() != metadata_source.receipt().dispatch()
        || f14k_source.au_index() != metadata_source.receipt().au_index()
    {
        return Err(SourceFrameStageError::Pipeline(
            "F14K receipt does not match the current source/AU lineage".to_owned(),
        ));
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
    let mut flat7x_d0_history_after = None;
    let mut flat7x_pair_after = None;
    let mut flat7x_d0_frame_syntax = None;
    let mut flat7x_d0_block2_placement_receipt = None;
    if let Some(binding) = flat7x_pair_source_binding {
        let history_before = codec_before.d0_overlap_history().ok_or_else(|| {
            SourceFrameStageError::Pipeline(
                "Flat7x Codec cold root has no D0 overlap history".to_owned(),
            )
        })?;
        if source_index.value() < 2 {
            let block_series = stage_flat7x_pair_d0_pcm_block_series(
                &binding,
                history_before,
                analysis_providers.providers().transform,
            )
            .map_err(|error| {
                SourceFrameStageError::Pipeline(format!(
                    "Flat7x suppressed-source PCM history: {error:?}"
                ))
            })?;
            flat7x_d0_history_after = Some(*block_series.after_history());
        } else {
            let protected_container = f14k_source.protected_container().ok_or_else(|| {
                SourceFrameStageError::Pipeline(
                    "Flat7x published D0 source requires the F14K protected container".to_owned(),
                )
            })?;
            let d0_bap_parameters = Flat7xD0BapParameters::etsi_ts_102_366_v1_4_1();
            let d0_bsi_inputs = flat7x_d0_bsi_inputs.as_ref().ok_or_else(|| {
                SourceFrameStageError::Pipeline(
                    "Flat7x D0 BSI source inputs are missing".to_owned(),
                )
            })?;
            if d0_bsi_inputs.source_identity()
                != binding.role_map_receipt().shared_analysis_identity()
            {
                return Err(SourceFrameStageError::Pipeline(
                    "Flat7x D0 BSI controls do not match the pair source role receipt".to_owned(),
                ));
            }
            let binding =
                binding.with_programme_levels(d0_bsi_inputs.dialnorm(), d0_bsi_inputs.compr());
            let expected_source_generation_receipt = flat7x_d0_source_generation_receipt
                .as_ref()
                .ok_or_else(|| {
                SourceFrameStageError::Pipeline(
                    "Flat7x D0 source-generation receipt is missing".to_owned(),
                )
            })?;
            let d0_block2_source_receipt = bind_flat7x_d0_block2_source_receipt(
                &metadata_source,
                &f14k_source,
                expected_source_generation_receipt,
            )
            .map_err(|error| {
                SourceFrameStageError::Pipeline(format!(
                    "Flat7x D0 block-two source receipt: {error:?}"
                ))
            })?;
            let make_dependent = || {
                produce_flat7x_pair_d0_pcm_frame_candidate(
                &binding,
                Flat7xD0PcmFrameSettings {
                    source_extent: binding.source_extent(),
                    before: history_before,
                    transform_parameters: analysis_providers.providers().transform,
                    target: binding.target(),
                    bsi_structure: Flat7xD0BsiStructure {
                        dialnorm: d0_bsi_inputs.dialnorm(),
                        compr: d0_bsi_inputs.compr(),
                        // The selected official-demo D0 profile observes mixmdate=0.
                        mixing: None,
                        block_mix: [None; 6],
                        information: None,
                        addbsi: d0_bsi_inputs.ci16_addbsi(),
                    },
                    source_generation_receipt: expected_source_generation_receipt,
                    block2_source_receipt: &d0_block2_source_receipt,
                    bap_parameters: &d0_bap_parameters,
                    dynamic_range_codes: d0_bsi_inputs.dynamic_range_codes(),
                },
            )
            .map_err(|error| {
                SourceFrameStageError::Pipeline(format!(
                    "Flat7x D0 PCM frame candidate ({}, {FLAT7X_D0_OPTIONAL_FRAME_FIELDS_POLICY_ID}): {error:?}",
                    jocforge_application::codec_source_bridge::FLAT7X_D0_BSI_SOURCE_POLICY_ID,
                ))
            })
            };
            let make_core = || {
                let i0_budget = I0AudioBlockSeriesBudget::new(20_480);
                // The 5.1 search can skip the entire interval between its seed and its
                // first midpoint. Try each coarse SNR offset against the smaller I0 frame.
                let selected_core_q = frame_candidate.frame().q();
                let mut q_candidates = frame_candidate
                    .attempted_qs()
                    .iter()
                    .copied()
                    .filter(|q| *q <= selected_core_q)
                    .collect::<Vec<_>>();
                q_candidates.extend((16..=selected_core_q).step_by(16));
                q_candidates.sort_unstable_by(|left, right| right.cmp(left));
                q_candidates.dedup();
                let all_q_candidates = q_candidates.clone();
                let mut rejected_q = Vec::new();
                let mut selected_i0_frame = None;
                let mut template = flat7x_i0_audio_block_policy_v1(
                    I0BapCandidateQ::new(0).expect("zero is an I0 SNR code"),
                )
                .ok_or_else(|| {
                    SourceFrameStageError::Pipeline(
                        "Flat7x I0 local block policy could not be constructed".to_owned(),
                    )
                })?;
                for (block, code) in template
                    .blocks
                    .iter_mut()
                    .zip(d0_bsi_inputs.dynamic_range_codes())
                {
                    block.dynamic_range_code = code;
                }
                let budget_probe = I0AudioBlockBudgetProbe::prepare(
                    frame_candidate.numeric_plan_receipt(),
                    template,
                    i0_budget,
                );
                for q_value in q_candidates {
                    let Some(candidate_q) = I0BapCandidateQ::new(q_value) else {
                        continue;
                    };
                    if budget_probe
                        .as_ref()
                        .is_some_and(|probe| probe.exceeds_budget(candidate_q))
                    {
                        rejected_q.push(q_value);
                        continue;
                    }
                    let mut i0_policy = template;
                    for block in &mut i0_policy.blocks {
                        block.candidate_q = candidate_q;
                    }
                    let candidate = match assemble_flat7x_i0_frame_candidate_from_binding(
                        &binding,
                        frame_candidate.numeric_plan_receipt(),
                        expected_source_generation_receipt,
                        i0_policy,
                        i0_budget,
                        I0AuxDataInput::PadToFrame,
                        CallerOwnedI0CrcrsvSeed { value: false },
                    ) {
                        Ok(candidate) => candidate,
                        Err(Flat7xPairSourceError::I0AudioBlockSeries(
                            Flat7xI0AudioBlockSeriesError::AudioBlockBudgetExceeded { .. },
                        )) => {
                            rejected_q.push(q_value);
                            continue;
                        }
                        Err(Flat7xPairSourceError::I0FrameAssembly(
                            Flat7xI0FrameAssemblyError::FrameSpanMismatch {
                                actual_bits,
                                required_bits,
                            },
                        )) if actual_bits > required_bits => {
                            rejected_q.push(q_value);
                            continue;
                        }
                        Err(error) => {
                            return Err(SourceFrameStageError::Pipeline(format!(
                            "Flat7x I0 frame candidate q={q_value} ({}): {error:?}",
                            jocforge_application::flat7x_pair_source::FLAT7X_I0_AUDIO_BLOCK_POLICY_ID,
                        )));
                        }
                    };
                    selected_i0_frame = Some((q_value, candidate));
                    break;
                }
                let Some((_selected_i0_q, i0_frame)) = selected_i0_frame else {
                    return Err(SourceFrameStageError::Pipeline(format!(
                        "Flat7x I0 bitrate-fit search found no candidate; attempted_qs={all_q_candidates:?} rejected_qs={rejected_q:?}"
                    )));
                };
                Ok::<_, SourceFrameStageError>(i0_frame)
            };
            // The AC-3 core and dependent frame read the same validated binding. Their bytes are still published I0 first.
            let (dependent, core) = jocforge_application::join_dsp_tasks(make_dependent, make_core);
            let d0_frame_candidate = dependent?;
            let assembly = d0_frame_candidate
                .frame_candidate()
                .frame()
                .receipt()
                .frame_assembly();
            let layout = assembly.layout();
            let skip = layout.block_two_skip_field();
            let skip_span = skip.bit_span();
            let container_bytes = skip.payload_bytes();
            let d0_frame = d0_frame_candidate.frame_candidate().frame();
            let d0_audio_receipt = d0_frame.receipt();
            let block_two_receipt = d0_audio_receipt.block_receipts()[2];
            let block_two_layout = layout.audio_blocks()[2];
            let block_two_prefix = block_two_layout.leading_audio_span();
            let block_two_mantissas = block_two_layout.trailing_audio_span().ok_or_else(|| {
                SourceFrameStageError::Pipeline(
                    "Flat7x D0 block-2 has no post-skipfield mantissa span".to_owned(),
                )
            })?;
            let d0_emdf_container_start =
                d0_frame_candidate.emdf_container_start().ok_or_else(|| {
                    SourceFrameStageError::Pipeline(
                        "Flat7x D0 protected EMDF payload cursor is missing".to_owned(),
                    )
                })?;
            let protected_container_length = u16::try_from(protected_container.bytes().len()).ok();
            if block_two_receipt.block_index().value() != 2
                || block_two_prefix.bit_length() != block_two_receipt.prefix_before_skipfield_bits()
                || skip.insertion_point().offset_within_audio_span_bits()
                    != block_two_prefix.bit_length()
                || skip_span.start_bit() != block_two_prefix.end_bit()
                || block_two_mantissas.start_bit() != skip_span.end_bit()
                || block_two_mantissas.bit_length() != block_two_receipt.mantissa_bits()
                || !(1..=511).contains(&container_bytes)
                || protected_container_length != Some(container_bytes)
                || d0_emdf_container_start.bit_offset() != skip_span.start_bit() + 10
            {
                return Err(SourceFrameStageError::Pipeline(
                    "Flat7x D0 protected EMDF cursor must place the complete block-2 skipfield after the block prefix and before its mantissas".to_owned(),
                ));
            }
            let container_start_bit = d0_emdf_container_start.bit_offset();
            let nauxbits = assembly.frame_auxbits_span();
            let crc2 = assembly.crc2_span();
            flat7x_d0_frame_syntax = Some(Flat7xD0FrameSyntaxSummary {
                audfrm_start_bit: layout.audio_frame_control_span().start_bit(),
                audfrm_end_bit: layout.audio_frame_control_span().end_bit(),
                skipflde_bit_offset: assembly.audio_frame_controls().skipflde_bit_offset(),
                block_two_skiple_bit_offset: skip_span.start_bit(),
                block_two_skipl_bit_offset: skip_span.start_bit() + 1,
                block_two_container_bytes: container_bytes,
                block_two_container_start_bit: container_start_bit,
                block_two_container_end_bit: skip_span.end_bit(),
                nauxbits_start_bit: nauxbits.start_bit(),
                nauxbits_end_bit: nauxbits.end_bit(),
                auxdatae_bit_offset: nauxbits.end_bit(),
                crcrsv_bit_offset: nauxbits.end_bit() + 1,
                crc2_start_bit: crc2.start_bit(),
                crc2_end_bit: crc2.end_bit(),
            });
            let i0_frame = core?;
            let f12_pair = stage_flat7x_pair_f12_integrity(Flat7xPairF12StageInput {
                integrity_before,
                i0_frame: &i0_frame,
                d0_frame: &d0_frame_candidate,
                f13_metadata: &metadata_source,
                f14k_source: &f14k_source,
                keyless_integrity: admitted_plan.source_providers().keyless_integrity(),
            })
            .map_err(|error| {
                SourceFrameStageError::Pipeline(format!(
                    "Flat7x paired F12 candidate failed: {error:?}"
                ))
            })?;
            let flat7x_output_owner = staging
                .take_flat7x_pair_output_owner(output_binding_before)
                .map_err(|error| {
                    SourceFrameStageError::Pipeline(format!("Flat7x pair output owner: {error}"))
                })?;
            let source_generation = Flat7xSourceGeneration::new(source_index.value());
            let f15_pair = stage_flat7x_pcm_pair_for_f15(
                &i0_frame,
                &d0_frame_candidate,
                source_generation,
                &metadata_source,
                &f12_pair,
                flat7x_output_owner,
            )
            .map_err(|error| {
                SourceFrameStageError::Pipeline(format!(
                    "Flat7x paired F15 candidate ({FLAT7X_SOURCE_GENERATION_MODEL_ID}): {error:?}"
                ))
            })?;
            let block2_placement = f15_pair.d0_block2_placement_receipt();
            flat7x_d0_block2_placement_receipt = Some(block2_placement.clone());
            if block2_placement.au_index().value() == 0 {
                let bit_span = block2_placement.bit_span();
                eprintln!(
                    "Flat7x D0 block-two placement: au={} payload_sequence={} block={} bits={}..{} C={} codec_source_generation={} f15_source_generation={}",
                    block2_placement.au_index().value(),
                    block2_placement.payload_sequence().label(),
                    block2_placement.block_index().zero_based_value(),
                    bit_span.start_bit(),
                    bit_span.end_bit(),
                    block2_placement.container_bytes(),
                    block2_placement.codec_source_generation().value(),
                    f15_pair.source_generation().value(),
                );
            }
            let output_owner_proposal =
                prepare_output_source_from_flat7x_pair(output_binding_before, &f15_pair).map_err(
                    |error| {
                        SourceFrameStageError::Pipeline(format!(
                            "Flat7x aggregate OutputOwner proposal failed: {error}"
                        ))
                    },
                )?;
            let (pair_proposal, after_history) = {
                let (pair_proposal, after_history, _) = f15_pair.into_parts();
                (pair_proposal, after_history)
            };
            let (_, output_after) = staging
                .append_flat7x_pair_with_output_proposal(
                    output_binding_before,
                    pair_proposal,
                    output_owner_proposal,
                )
                .map_err(|error| {
                    SourceFrameStageError::Pipeline(format!(
                        "Flat7x I0+D0 durable append: {error:?}"
                    ))
                })?;
            let codec_proposal = stage_codec_owner_after_flat7x_pair(
                codec_before,
                &frame_candidate,
                &i0_frame,
                &d0_frame_candidate,
                &output_after,
                &after_history,
            )
            .map_err(|error| {
                SourceFrameStageError::Pipeline(format!("Flat7x pair Codec after-root: {error:?}"))
            })?;
            let (integrity_root, pair_receipts, integrity_owner_receipt) = f12_pair.into_parts();
            let integrity_receipts = pair_receipts.d0().clone();
            flat7x_d0_history_after = Some(after_history);
            flat7x_pair_after = Some((
                output_after,
                codec_proposal,
                integrity_root,
                integrity_receipts,
                pair_receipts,
                integrity_owner_receipt,
            ));
        }
    }
    let (
        integrity_root,
        integrity_receipts,
        integrity_owner_receipt,
        flat7x_integrity_pair_receipts,
        output_after,
        codec_proposal,
    ) = if let Some((
        output_after,
        codec_proposal,
        integrity_root,
        integrity_receipts,
        pair_receipts,
        integrity_owner_receipt,
    )) = flat7x_pair_after
    {
        (
            integrity_root,
            integrity_receipts,
            integrity_owner_receipt,
            Some(pair_receipts),
            output_after,
            codec_proposal,
        )
    } else {
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
        let output_proposal = if joc_profile
            .joc_reconstruction_layout()
            .uses_seven_carriers()
        {
            if source_index.value() >= 2 || flat7x_d0_history_after.is_none() {
                return Err(SourceFrameStageError::Pipeline(
                    "Flat7x published source did not produce a paired F15 proposal".to_owned(),
                ));
            }
            prepare_output_source_for_flat7x_suppressed(output_binding_before, source_index)
                .map_err(|error| format!("Flat7x suppressed F15 output proposal: {error}"))?
        } else {
            prepare_output_source_from_candidate(
                prepared,
                output_binding_before,
                &metadata_source,
                &frame_candidate,
            )
            .map_err(|error| format!("F15 output proposal: {error}"))?
        };
        let staged_output = match flat7x_d0_history_after.as_ref() {
            Some(history_after) => stage_source_frame_output_with_d0_history(
                staging,
                output_binding_before,
                codec_before,
                &frame_candidate,
                output_proposal,
                history_after,
            ),
            None => stage_source_frame_output(
                staging,
                output_binding_before,
                codec_before,
                &frame_candidate,
                output_proposal,
            ),
        }
        .map_err(|error: SourceFrameOutputStageError| format!("F12/F15 source output: {error}"))?;
        let (output_after, codec_proposal) = staged_output.into_parts();
        (
            integrity_root,
            integrity_receipts,
            integrity_owner_receipt,
            None,
            output_after,
            codec_proposal,
        )
    };
    let frame_sha256 = *codec_proposal.frame_sha256();
    let codec_receipt = codec_proposal
        .gtc_receipt(codec_before, main_callback_transaction.clone())
        .map_err(|error| format!("Codec owner receipt: {error:?}"))?;
    let output_receipt = output_binding_before
        .gtc_receipt_after_source(&output_after, main_callback_transaction.clone())
        .map_err(|error| format!("Output owner receipt: {error}"))?;
    let append = output_after.durable_append_receipt().map(to_append_summary);
    drop(frame_candidate);

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

    let [frontend, control, audio, reader_eof_receipt] = callback.receipts;
    let mut owner_receipts = vec![frontend, control, audio];
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
        main_roots: callback.main_roots,
        reader_eof_root: callback.reader_eof_root,
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
        flat7x_d0_frame_syntax,
        flat7x_d0_block2_placement_receipt,
        f14k_source,
        integrity_receipts,
        flat7x_integrity_pair_receipts,
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
