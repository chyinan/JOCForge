// pattern: Functional Core
//! Parent-issued same-source projection into the selected F12 source plan.

use std::sync::Arc;

use crate::{
    analysis::{
        converter_source_plan::{F11ConverterSourceResult, MainFrameCodeSelection},
        exponent_plan::{AnalysisBlockIndex, FullbandLaneIndex},
        owner::{AnalysisNumericPlanStage, AnalysisOwnerStageError},
        structure_refresh::{F07_BLOCKS_PER_FRAME, F07_FULLBAND_LANES},
    },
    codec::{
        bap::BapParameters,
        selected_frame_trial::{
            BlockExponentPlan, CoefficientBlockInput, ConverterHeaderInput, EmdfFrameInput,
            FullbandExponentLanePlan, LfeExponentPlan, MainExponentSyntax, SelectedBsiInput,
            SelectedSourceNumericPlan,
        },
    },
    transform::fullband_exponent_pipeline::NumericPlanReceipt,
};

const SELECTED_FBW_BANDWIDTH_CODE: u8 = 48;
const SELECTED_CODEC_BLOCK_COUNT: usize = F07_BLOCKS_PER_FRAME;

/// Errors while binding the completed main analysis plan, F11 result, and same-source payloads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedSourcePlanError {
    AnalysisPlanUnavailable,
    SharedAnalysisUnavailable,
    IncompleteF08Plan,
    IncompleteExponentPlan,
    SharedAnalysisAdvancementMismatch,
    SourceIdentityMismatch,
    PublishedPayloadRequired,
    PublishedPayloadNotAllowed,
    BapProviderIdentityMismatch,
    BlockIndexOverflow,
    LaneIndexOverflow,
    FullbandExponentPlanMissing { block: u8, lane: u8 },
    LfeExponentPlanMissing { block: u8 },
    OwnerTransaction(AnalysisOwnerStageError),
}

/// Projects the already completed same-source F08/F09/F10 plan and F11 result into one
/// `NumericPlanReceipt<SelectedSourceNumericPlan>` through the existing SourceTransaction.
///
/// BSI and EMDF remain separately typed inputs to F12. This function checks their source
/// identities before issuing the one numeric receipt; it neither constructs their values nor
/// advances F07/F08/F09/F10 again.
pub fn prepare_selected_source_numeric_plan<'a>(
    analysis_stage: &'a mut AnalysisNumericPlanStage,
    f11: &F11ConverterSourceResult,
    bsi: &SelectedBsiInput,
    emdf: &EmdfFrameInput<'_>,
    bap_parameters: Arc<BapParameters>,
) -> Result<NumericPlanReceipt<'a, SelectedSourceNumericPlan>, SelectedSourcePlanError> {
    let selected_plan = {
        let shared = analysis_stage
            .shared_analysis()
            .ok_or(SelectedSourcePlanError::SharedAnalysisUnavailable)?;
        let source_identity = shared.identity();
        if shared.advancement_count() != 1 {
            return Err(SelectedSourcePlanError::SharedAnalysisAdvancementMismatch);
        }
        if f11.source_identity() != source_identity || bsi.source_identity() != source_identity {
            return Err(SelectedSourcePlanError::SourceIdentityMismatch);
        }
        match emdf {
            EmdfFrameInput::AbsentPublishedPayloadV1
                if source_identity.source_index().value() < 2 => {}
            EmdfFrameInput::AbsentPublishedPayloadV1 => {
                return Err(SelectedSourcePlanError::PublishedPayloadRequired);
            }
            EmdfFrameInput::Published(payload)
                if payload.source_identity() == source_identity
                    && source_identity.source_index().value() >= 2 => {}
            EmdfFrameInput::Published(payload) if payload.source_identity() != source_identity => {
                return Err(SelectedSourcePlanError::SourceIdentityMismatch);
            }
            EmdfFrameInput::Published(_) => {
                return Err(SelectedSourcePlanError::PublishedPayloadNotAllowed);
            }
        }
        if bap_parameters.provider_sha256() != f11.bap_provider_sha256() {
            return Err(SelectedSourcePlanError::BapProviderIdentityMismatch);
        }

        let analysis_plan = analysis_stage
            .plan()
            .ok_or(SelectedSourcePlanError::AnalysisPlanUnavailable)?;
        let f08 = analysis_plan.f08();
        let exponents = analysis_plan.exponents();
        if !f08.is_six_block_complete() {
            return Err(SelectedSourcePlanError::IncompleteF08Plan);
        }
        if !exponents.is_six_block_complete() {
            return Err(SelectedSourcePlanError::IncompleteExponentPlan);
        }

        let coefficients_by_block = std::array::from_fn(|block| {
            let fullband_words = std::array::from_fn(|lane| {
                *f08.fullband_exponent_inputs()[block * F07_FULLBAND_LANES + lane]
                    .primary_clipped_f32_bits()
            });
            CoefficientBlockInput {
                fullband_words,
                lfe_words: *f08.lfe_exponent_inputs()[block].primary_clipped_f32_bits(),
            }
        });

        let mut exponent_blocks = Vec::with_capacity(SELECTED_CODEC_BLOCK_COUNT);
        for block in 0..SELECTED_CODEC_BLOCK_COUNT {
            let block_index = u8::try_from(block)
                .map(AnalysisBlockIndex::new)
                .map_err(|_| SelectedSourcePlanError::BlockIndexOverflow)?;
            let mut fullband_lanes = Vec::with_capacity(F07_FULLBAND_LANES);
            for lane in 0..F07_FULLBAND_LANES {
                let lane_index = u8::try_from(lane)
                    .map(FullbandLaneIndex::new)
                    .map_err(|_| SelectedSourcePlanError::LaneIndexOverflow)?;
                let exponent_plan = exponents.fullband_block(block_index, lane_index).ok_or(
                    SelectedSourcePlanError::FullbandExponentPlanMissing {
                        block: u8::try_from(block).unwrap_or(u8::MAX),
                        lane: u8::try_from(lane).unwrap_or(u8::MAX),
                    },
                )?;
                let strategy = exponent_plan.strategy();
                fullband_lanes.push(FullbandExponentLanePlan {
                    decoded_exponents: *exponent_plan.decoded_exponents(),
                    strategy,
                    // The selected wire profile uses code 48 for an anchor; strategy 0
                    // emits no bandwidth field, so its stored local placeholder is inert.
                    bandwidth_code: if strategy == 0 {
                        0
                    } else {
                        SELECTED_FBW_BANDWIDTH_CODE
                    },
                    base_exponent: exponent_plan.base_exponent(),
                    packed_groups: exponent_plan.packed_groups().to_vec(),
                    gain_range: exponent_plan.gain_range(),
                });
            }
            let fullband: [FullbandExponentLanePlan; F07_FULLBAND_LANES] = fullband_lanes
                .try_into()
                .map_err(|_| SelectedSourcePlanError::IncompleteExponentPlan)?;
            let lfe_plan = exponents.lfe_block(block_index).ok_or(
                SelectedSourcePlanError::LfeExponentPlanMissing {
                    block: u8::try_from(block).unwrap_or(u8::MAX),
                },
            )?;
            exponent_blocks.push(BlockExponentPlan {
                fullband,
                lfe: LfeExponentPlan {
                    decoded_exponents: lfe_plan.decoded_exponents(),
                    strategy: lfe_plan.strategy(),
                    base_exponent: lfe_plan.base_exponent(),
                    packed_groups: lfe_plan.packed_groups(),
                },
            });
        }
        let exponent_plan_by_block: [BlockExponentPlan; SELECTED_CODEC_BLOCK_COUNT] =
            exponent_blocks
                .try_into()
                .map_err(|_| SelectedSourcePlanError::IncompleteExponentPlan)?;

        let main_exponent_syntax = match f11.main_frame_code_selection() {
            MainFrameCodeSelection::FrameCodes(codes) => {
                MainExponentSyntax::FrameCodes(std::array::from_fn(|index| codes[index].value()))
            }
            MainFrameCodeSelection::BlockStrategies => MainExponentSyntax::BlockStrategies,
        };
        let converter = ConverterHeaderInput {
            frame_codes: std::array::from_fn(|index| f11.converter_frame_codes()[index].value()),
            snr_offset: f11.converter_snr_offset(),
        };

        let bap_provider_sha256 = bap_parameters.provider_sha256();
        SelectedSourceNumericPlan {
            bap_provider_sha256,
            bap_parameters,
            coefficients_by_block,
            exponent_plan_by_block,
            db_per_bit_codes: *analysis_plan.gain_db().db_per_bit_codes_by_block(),
            short_flags: *analysis_plan.short_flags(),
            previous_short: *analysis_plan.previous_short_before(),
            main_exponent_syntax,
            converter,
        }
    };

    analysis_stage
        .prepare_selected_source_numeric_plan(selected_plan)
        .map_err(SelectedSourcePlanError::OwnerTransaction)
}

#[cfg(test)]
mod tests {
    use super::{SelectedSourcePlanError, prepare_selected_source_numeric_plan};
    use crate::{
        analysis::converter_source_plan::prepare_f11_converter_source_result,
        analysis::{
            input_history::INPUT_HISTORY_BLOCK_SAMPLES,
            owner::{AnalysisNumericPlanStage, AnalysisOwnerRoot, AnalysisOwnerStageError},
            source_frame::InstalledAudioSourceFrame,
            structure_refresh::{F07_BLOCKS_PER_FRAME, F07_FULLBAND_LANES},
            transient_parameters::load_transient_parameters,
        },
        codec::{
            bap::{BapParameters, load_bap_parameters},
            policy_parameters::{PolicyProfile, load_policy_parameters},
            selected_frame_trial::{
                EmdfFrameInput, SelectedBsiInformationInput, SelectedBsiInput,
                SelectedBsiMixingInput, SelectedFrameTrialPlan, SelectedFrameTrialPlanInput,
            },
        },
        timeline::CodecSourceIndex,
        transaction::{AttemptId, CallbackTransactionId},
        transform::math_generated_parameters::MathGeneratedTransformParameters,
    };
    use std::sync::Arc;

    const TRANSIENT_PARAMETERS: &[u8] =
        include_bytes!("../../tests/fixtures/transient_channel/PARAMETERS.json");
    const F08_PARAMETERS: &[u8] =
        include_bytes!("../transform/ordinary_transform_math_parameters.json");
    const POLICY_PARAMETERS: &[u8] =
        include_bytes!("../../tests/fixtures/policy_authorized/PARAMETERS.json");
    const BAP_PROVIDER: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-PUBLIC-BAP-PROVIDER-V1/public-bap-48k-v1.json"
    );

    fn analysis_numeric_stage() -> AnalysisNumericPlanStage {
        let root = AnalysisOwnerRoot::cold();
        let installed: Box<InstalledAudioSourceFrame> =
            vec![[[0_u32; INPUT_HISTORY_BLOCK_SAMPLES]; F07_BLOCKS_PER_FRAME]; 6]
                .into_boxed_slice()
                .try_into()
                .unwrap();
        let transient = load_transient_parameters(Some(TRANSIENT_PARAMETERS)).unwrap();
        let transform =
            MathGeneratedTransformParameters::from_provider_bytes(F08_PARAMETERS).unwrap();
        let policy =
            load_policy_parameters(PolicyProfile::selected(), Some(POLICY_PARAMETERS)).unwrap();
        root.prepare_f07_source(
            AttemptId::new(0),
            CallbackTransactionId::new("selected-source-plan-adapter").unwrap(),
            CodecSourceIndex::new(0),
            &installed,
            [false; F07_FULLBAND_LANES],
            &transient,
        )
        .unwrap()
        .complete_f08(&transform)
        .unwrap()
        .complete_gain_db(&policy)
        .unwrap()
        .complete_f09_f10(&policy)
        .unwrap()
    }

    fn selected_bsi(
        shared: &crate::analysis::source_frame::SharedAnalysisReceipt,
    ) -> SelectedBsiInput {
        SelectedBsiInput::from_parent_values(
            shared,
            1,
            Some(0),
            [Some(0); F07_BLOCKS_PER_FRAME],
            SelectedBsiMixingInput {
                preferred_downmix_mode: 0,
                ltrt_center_mix_level: 0,
                loro_center_mix_level: 0,
                ltrt_surround_mix_level: 0,
                loro_surround_mix_level: 0,
            },
            SelectedBsiInformationInput {
                bitstream_mode: 0,
                copyright: false,
                original_bitstream: false,
                surround_ex_mode: 0,
                source_sample_rate_code: 0,
            },
        )
        .unwrap()
    }

    #[test]
    fn selected_plan_receipt_uses_one_same_source_transaction_and_validated_bap_provider() {
        let mut analysis = analysis_numeric_stage();
        let bsi = selected_bsi(analysis.shared_analysis().unwrap());
        let bap_parameters: Arc<BapParameters> =
            Arc::new(load_bap_parameters(Some(BAP_PROVIDER)).unwrap());
        let f11 =
            prepare_f11_converter_source_result(&analysis, &bsi, bap_parameters.as_ref()).unwrap();
        let emdf = EmdfFrameInput::AbsentPublishedPayloadV1;

        let receipt = prepare_selected_source_numeric_plan(
            &mut analysis,
            &f11,
            &bsi,
            &emdf,
            Arc::clone(&bap_parameters),
        )
        .unwrap();
        let shared = receipt.shared_analysis();
        assert_eq!(receipt.identity().source_index(), CodecSourceIndex::new(0));
        assert_eq!(shared.advancement_count(), 1);
        assert_eq!(
            receipt.plan().bap_provider_sha256,
            bap_parameters.provider_sha256()
        );
        assert_eq!(
            receipt.plan().converter.snr_offset,
            f11.converter_snr_offset()
        );
        assert_eq!(
            receipt.plan().converter.frame_codes,
            std::array::from_fn(|index| f11.converter_frame_codes()[index].value())
        );

        let frame_plan = SelectedFrameTrialPlan::new(SelectedFrameTrialPlanInput {
            numeric_plan: &receipt,
            bsi: bsi.clone(),
            emdf: EmdfFrameInput::AbsentPublishedPayloadV1,
        })
        .unwrap();
        drop(frame_plan);
        drop(receipt);

        assert_eq!(
            prepare_selected_source_numeric_plan(
                &mut analysis,
                &f11,
                &bsi,
                &emdf,
                Arc::clone(&bap_parameters),
            ),
            Err(SelectedSourcePlanError::OwnerTransaction(
                AnalysisOwnerStageError::SourceTransactionRejected
            ))
        );
    }
}
