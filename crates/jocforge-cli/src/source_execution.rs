// pattern: Imperative Shell
//! Composes one real MAIN source callback through F07-F15 as a noncommitting root proposal.

use crate::{
    output_publication::{
        ProgrammeOutputStaging, SourceFrameOutputStageError, stage_source_frame_output,
        stage_source_frame_output_with_d0_history,
    },
    programme_roots::source_for_callback_clock,
    selected_spatial_callback_providers::SelectedMainSpatialCallbackProviders,
};
use jocforge_application::flat7x_pair_source::{
    CallerOwnedI0CrcrsvSeed, Flat7xD0BapParameters, Flat7xD0BsiStructure, Flat7xD0PcmFrameSettings,
    Flat7xI0FrameAssemblyError, Flat7xPairSourceError, I0AudioBlockSeriesBudget, I0AuxDataInput,
    OpaqueI0Ac3BitSpan, assemble_flat7x_i0_frame_candidate_from_binding, bind_flat7x_pair_source,
    flat7x_i0_audio_block_policy_v1, produce_flat7x_pair_d0_pcm_frame_candidate,
    stage_flat7x_pair_d0_pcm_block_series,
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
        ProgrammeOutputBinding, prepare_output_source_for_flat7x_suppressed,
        prepare_output_source_from_candidate, prepare_output_source_from_flat7x_pair,
    },
    reader_eof_callback::{GtcOwnerId, ReaderEofMachine, ReaderEofReplacementCandidate},
    source_analysis_bridge::{
        AnalysisOwnerRoot, SelectedSourceAnalysisProviders, SpatialSourceAnalysisBridgeInput,
        prepare_source_analysis_from_spatial_callback,
    },
    source_frame_bridge::{
        CodecOwnerRoot, Flat7xD0PcmSourceExtent, Flat7xD0PcmSourceExtentOrigin,
        Flat7xSourcePcmProjection, build_flat7x_source_pcm_projection,
        prepare_admitted_source_frame, stage_codec_owner_after_flat7x_pair,
    },
    spatial_callback::{MainSpatialCallbackCandidate, MainSpatialColdRoots},
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
    source_extent: Option<Flat7xD0PcmSourceExtent>,
    physical_pcm_rows: Vec<Vec<u32>>,
}

impl Flat7xPcmSourceRows {
    pub(crate) fn new(
        input_sha256: [u8; 32],
        callback_index: FrontendCallbackIndex,
        source_index: CodecSourceIndex,
        delivery: Flat7xPcmRowDelivery,
        source_extent: Option<Flat7xD0PcmSourceExtent>,
        physical_pcm_rows: Vec<Vec<u32>>,
    ) -> Self {
        Self {
            input_sha256,
            callback_index,
            source_index,
            delivery,
            source_extent,
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

    pub const fn source_extent(&self) -> Option<Flat7xD0PcmSourceExtent> {
        self.source_extent
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
    pub flat7x_d0_frame_syntax: Option<Flat7xD0FrameSyntaxSummary>,
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
) -> Result<SourceFrameAdvance<'pcm>, SourceFrameStageError> {
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
                return Err(SourceFrameStageError::Pipeline(
                    "Flat7x PCM rows do not match the current callback/source lineage".to_owned(),
                ));
            }
            let source_extent = match (expected_delivery, pcm_source.source_extent()) {
                (Flat7xPcmRowDelivery::NormalCallback, Some(extent))
                    if extent.callback_index() == callback_index
                        && extent.source_index() == source_index
                        && extent.origin() == Flat7xD0PcmSourceExtentOrigin::ReaderCallback =>
                {
                    extent
                }
                (Flat7xPcmRowDelivery::CachedDrain, Some(extent))
                    if extent.callback_index() == callback_index
                        && extent.source_index() == source_index
                        && extent.origin()
                            == Flat7xD0PcmSourceExtentOrigin::CachedDrainRepeatPreviousCallbackV1 =>
                {
                    extent
                }
                _ => {
                    return Err(SourceFrameStageError::Pipeline(
                        "Flat7x PCM extent does not match the current callback/source lineage"
                            .to_owned(),
                    ));
                }
            };
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
                    source_extent,
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
    let flat7x_d0_bsi_inputs = if flat7x_source_pcm_projection.is_some() {
        let shared_analysis = analysis_stage.shared_analysis().ok_or_else(|| {
            SourceFrameStageError::Pipeline(
                "Flat7x D0 BSI binding requires same-source shared analysis".to_owned(),
            )
        })?;
        Some(
            bind_flat7x_d0_bsi_source_inputs(prepared, shared_analysis, drc_source.after_root())
                .map_err(|error| {
                    SourceFrameStageError::Pipeline(format!(
                        "Flat7x same-source D0 BSI controls: {error:?}"
                    ))
                })?,
        )
    } else {
        None
    };
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
            let d0_frame_candidate = produce_flat7x_pair_d0_pcm_frame_candidate(
                &binding,
                Flat7xD0PcmFrameSettings {
                    source_extent: binding.source_extent(),
                    before: history_before,
                    transform_parameters: analysis_providers.providers().transform,
                    target: binding.target(),
                    bsi_structure: Flat7xD0BsiStructure {
                        dialnorm: d0_bsi_inputs.dialnorm(),
                        compr: d0_bsi_inputs.compr(),
                        mixing: Some(d0_bsi_inputs.mixing()),
                        block_mix: [None; 6],
                        information: None,
                        addbsi: None,
                    },
                    complete_emdf_container: protected_container.bytes(),
                    bap_parameters: &d0_bap_parameters,
                    dynamic_range_codes: d0_bsi_inputs.dynamic_range_codes(),
                },
            )
            .map_err(|error| {
                SourceFrameStageError::Pipeline(format!(
                    "Flat7x D0 PCM frame candidate ({}, {FLAT7X_D0_OPTIONAL_FRAME_FIELDS_POLICY_ID}): {error:?}",
                    jocforge_application::codec_source_bridge::FLAT7X_D0_BSI_SOURCE_POLICY_ID,
                ))
            })?;
            let assembly = d0_frame_candidate
                .frame_candidate()
                .frame()
                .receipt()
                .frame_assembly();
            let layout = assembly.layout();
            let skip = layout.block_two_skip_field();
            let skip_span = skip.bit_span();
            let container_bytes = skip.payload_bytes();
            let container_bits = u32::from(container_bytes) * 8;
            let container_start_bit = skip_span.end_bit() - container_bits;
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
            let i0_policy = flat7x_i0_audio_block_policy_v1().ok_or_else(|| {
                SourceFrameStageError::Pipeline(
                    "Flat7x I0 local block policy could not be constructed".to_owned(),
                )
            })?;
            let i0_budget = I0AudioBlockSeriesBudget::new(20_480);
            let i0_frame = match assemble_flat7x_i0_frame_candidate_from_binding(
                &binding,
                frame_candidate.numeric_plan_receipt(),
                i0_policy,
                i0_budget,
                I0AuxDataInput::Absent,
                CallerOwnedI0CrcrsvSeed { value: false },
            ) {
                Ok(candidate) => candidate,
                Err(Flat7xPairSourceError::I0FrameAssembly(
                    Flat7xI0FrameAssemblyError::FrameSpanMismatch {
                        actual_bits,
                        required_bits,
                    },
                )) if actual_bits < required_bits => {
                    let auxdata_bits = required_bits - actual_bits;
                    let auxdata_length =
                        usize::try_from(auxdata_bits.div_ceil(8)).map_err(|_| {
                            SourceFrameStageError::Pipeline(
                                "Flat7x I0 auxdata extent overflow".to_owned(),
                            )
                        })?;
                    let auxdata = vec![0_u8; auxdata_length];
                    assemble_flat7x_i0_frame_candidate_from_binding(
                        &binding,
                        frame_candidate.numeric_plan_receipt(),
                        i0_policy,
                        i0_budget,
                        I0AuxDataInput::Present(OpaqueI0Ac3BitSpan::new(
                            &auxdata,
                            0,
                            auxdata_bits,
                        )),
                        CallerOwnedI0CrcrsvSeed { value: false },
                    )
                    .map_err(|error| {
                        SourceFrameStageError::Pipeline(format!(
                            "Flat7x I0 frame candidate ({}): {error:?}",
                            jocforge_application::flat7x_pair_source::FLAT7X_I0_AUDIO_BLOCK_POLICY_ID,
                        ))
                    })?
                }
                Err(error) => {
                    return Err(SourceFrameStageError::Pipeline(format!(
                        "Flat7x I0 frame candidate ({}): {error:?}",
                        jocforge_application::flat7x_pair_source::FLAT7X_I0_AUDIO_BLOCK_POLICY_ID,
                    )));
                }
            };
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
        let output_proposal =
            if joc_profile.joc_reconstruction_layout() == JocReconstructionLayout::Flat7x {
                if source_index.value() >= 2 || flat7x_d0_history_after.is_none() {
                    return Err(SourceFrameStageError::Pipeline(
                        "Flat7x published source did not produce a paired F15 proposal".to_owned(),
                    ));
                }
                prepare_output_source_for_flat7x_suppressed(&output_binding_before, source_index)
                    .map_err(|error| format!("Flat7x suppressed F15 output proposal: {error}"))?
            } else {
                prepare_output_source_from_candidate(
                    prepared,
                    &output_binding_before,
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
        return Err(SourceFrameStageError::Pipeline(
            "ReaderEof receipt is not bound to the callback before-root".to_owned(),
        ));
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
        flat7x_d0_frame_syntax,
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
