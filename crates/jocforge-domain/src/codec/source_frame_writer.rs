// pattern: Functional Core
//! Same-source F11/F12 orchestration that returns a complete frame candidate without commit.

use std::{convert::Infallible, sync::Arc};

use crate::{
    analysis::{
        converter_source_plan::{ConverterSourcePlanError, prepare_f11_converter_source_result},
        owner::AnalysisNumericPlanStage,
    },
    codec::{
        bap::BapParameters,
        owner::{AcceptedCodecFrame, CodecOwnerStageError},
        selected_frame_trial::{
            EmdfFrameInput, SelectedBsiInput, SelectedFramePlanError, SelectedFrameTrialFatalError,
            SelectedFrameTrialPlan, SelectedFrameTrialPlanInput, SelectedMainFrameCandidate,
            SelectedSourceNumericPlan,
        },
        selected_main_q_search::{
            SelectedMainQOutcome, SelectedMainQSearchError, search_selected_main_q,
        },
        selected_source_plan::{SelectedSourcePlanError, prepare_selected_source_numeric_plan},
    },
    transform::fullband_exponent_pipeline::NumericPlanReceipt,
};

/// One complete source-bound F12 frame candidate and its bounded search trace.
///
/// This value is still a candidate: it is not a codec-owner proposal, durable append receipt,
/// programme receipt, or permission to publish. The caller retains all owner commit authority.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct SelectedSourceFrameCandidate<'a> {
    numeric_plan: NumericPlanReceipt<'a, SelectedSourceNumericPlan>,
    bsi: SelectedBsiInput,
    emdf: EmdfFrameInput<'a>,
    frame: Box<SelectedMainFrameCandidate>,
    accepted_codec_frame: AcceptedCodecFrame,
    attempted_qs: Vec<u16>,
    midpoint_trials: u8,
}

impl<'a> SelectedSourceFrameCandidate<'a> {
    pub const fn numeric_plan_receipt(&self) -> &NumericPlanReceipt<'a, SelectedSourceNumericPlan> {
        &self.numeric_plan
    }

    pub const fn bsi_input(&self) -> &SelectedBsiInput {
        &self.bsi
    }

    pub const fn emdf_input(&self) -> &EmdfFrameInput<'a> {
        &self.emdf
    }

    pub const fn frame(&self) -> &SelectedMainFrameCandidate {
        &self.frame
    }

    pub const fn accepted_codec_frame(&self) -> AcceptedCodecFrame {
        self.accepted_codec_frame
    }

    pub fn attempted_qs(&self) -> &[u16] {
        &self.attempted_qs
    }

    pub const fn midpoint_trials(&self) -> u8 {
        self.midpoint_trials
    }

    pub fn into_frame(self) -> Box<SelectedMainFrameCandidate> {
        self.frame
    }
}

/// Errors before or during the same-source selected frame construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedSourceFrameWriterError {
    Converter(ConverterSourcePlanError),
    NumericPlan(SelectedSourcePlanError),
    FramePlan(SelectedFramePlanError),
    Trial(SelectedFrameTrialFatalError),
    CodecOwner(CodecOwnerStageError),
    NoFeasibleCandidate,
}

/// Builds one complete, CRC-checked source frame using the approved F11 and F12 producers.
///
/// The analysis stage remains a noncommitting proposal. This function does not install its
/// after-state, stage the codec owner, append bytes, or advance programme publication.
pub fn prepare_selected_source_frame_candidate<'a>(
    analysis_stage: &'a mut AnalysisNumericPlanStage,
    bsi: &SelectedBsiInput,
    emdf: &EmdfFrameInput<'a>,
    bap_parameters: Arc<BapParameters>,
) -> Result<SelectedSourceFrameCandidate<'a>, SelectedSourceFrameWriterError> {
    let expected_identity = analysis_stage
        .shared_analysis()
        .ok_or(SelectedSourceFrameWriterError::Converter(
            ConverterSourcePlanError::SharedAnalysisMissing,
        ))?
        .identity()
        .clone();

    let f11 = prepare_f11_converter_source_result(analysis_stage, bsi, bap_parameters.as_ref())
        .map_err(SelectedSourceFrameWriterError::Converter)?;
    let numeric_plan =
        prepare_selected_source_numeric_plan(analysis_stage, &f11, bsi, emdf, bap_parameters)
            .map_err(SelectedSourceFrameWriterError::NumericPlan)?;

    let (frame, attempted_qs, midpoint_trials) = {
        let frame_plan = SelectedFrameTrialPlan::new(SelectedFrameTrialPlanInput {
            numeric_plan: &numeric_plan,
            bsi: bsi.clone(),
            emdf: emdf.clone(),
        })
        .map_err(SelectedSourceFrameWriterError::FramePlan)?;
        let search = match search_selected_main_q::<Infallible>(Ok(&frame_plan)) {
            Ok(search) => search,
            Err(SelectedMainQSearchError::Trial(error)) => {
                return Err(SelectedSourceFrameWriterError::Trial(error));
            }
            Err(SelectedMainQSearchError::Upstream(never)) => match never {},
        };
        let SelectedMainQOutcome::Best(frame) = search.outcome() else {
            return Err(SelectedSourceFrameWriterError::NoFeasibleCandidate);
        };
        (
            frame.clone(),
            search.attempted_qs().to_vec(),
            search.midpoint_trials(),
        )
    };

    if frame.source_generation_identity() != &expected_identity {
        return Err(SelectedSourceFrameWriterError::FramePlan(
            SelectedFramePlanError::SourceGenerationMismatch,
        ));
    }

    let accepted_codec_frame = AcceptedCodecFrame::from_candidate(&frame)
        .map_err(SelectedSourceFrameWriterError::CodecOwner)?;

    Ok(SelectedSourceFrameCandidate {
        numeric_plan,
        bsi: bsi.clone(),
        emdf: emdf.clone(),
        frame,
        accepted_codec_frame,
        attempted_qs,
        midpoint_trials,
    })
}

#[cfg(test)]
mod tests {
    use super::{SelectedSourceFrameWriterError, prepare_selected_source_frame_candidate};
    use crate::{
        analysis::{
            input_history::INPUT_HISTORY_BLOCK_SAMPLES,
            owner::{AnalysisNumericPlanStage, AnalysisOwnerRoot},
            source_frame::InstalledAudioSourceFrame,
            structure_refresh::{F07_BLOCKS_PER_FRAME, F07_FULLBAND_LANES},
            transient_parameters::load_transient_parameters,
        },
        codec::{
            bap::{BapParameters, load_bap_parameters},
            policy_parameters::{PolicyProfile, load_policy_parameters},
            selected_frame_trial::{
                EmdfFrameInput, SelectedBsiInformationInput, SelectedBsiInput,
                SelectedBsiMixingInput,
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

    fn analysis_numeric_stage(source_index: u64) -> AnalysisNumericPlanStage {
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
            CallbackTransactionId::new("selected-source-frame-writer").unwrap(),
            CodecSourceIndex::new(source_index),
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

    fn source_bsi(analysis: &AnalysisNumericPlanStage) -> SelectedBsiInput {
        SelectedBsiInput::from_parent_values(
            analysis.shared_analysis().unwrap(),
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
    fn same_source_f11_f12_path_builds_complete_noncommitting_frame_candidate() {
        let mut analysis = analysis_numeric_stage(0);
        let bsi = source_bsi(&analysis);
        let bap_parameters: Arc<BapParameters> =
            Arc::new(load_bap_parameters(Some(BAP_PROVIDER)).unwrap());
        let expected_identity = analysis.shared_analysis().unwrap().identity().clone();

        let candidate = prepare_selected_source_frame_candidate(
            &mut analysis,
            &bsi,
            &EmdfFrameInput::AbsentPublishedPayloadV1,
            bap_parameters,
        )
        .unwrap();

        assert_eq!(
            candidate.frame().source_generation_identity(),
            &expected_identity
        );
        assert_eq!(
            candidate.numeric_plan_receipt().identity(),
            &expected_identity
        );
        assert_eq!(candidate.bsi_input().source_identity(), &expected_identity);
        assert_eq!(
            candidate.emdf_input(),
            &EmdfFrameInput::AbsentPublishedPayloadV1
        );
        assert_eq!(
            candidate.accepted_codec_frame().source_index(),
            CodecSourceIndex::new(0)
        );
        assert_eq!(candidate.frame().source_index(), CodecSourceIndex::new(0));
        assert_eq!(candidate.frame().frame_bytes().len(), 4_096);
        assert_eq!(candidate.frame().frame_bytes()[0..2], [0x0b, 0x77]);
        assert!(candidate.attempted_qs().contains(&candidate.frame().q()));
        assert!(candidate.midpoint_trials() <= 10);
        assert_eq!(
            crate::integrity::crc16::eac3_crc16(&candidate.frame().frame_bytes()[2..]),
            0
        );
    }

    #[test]
    fn later_source_cannot_use_suppressed_absent_payload() {
        let mut analysis = analysis_numeric_stage(2);
        let bsi = source_bsi(&analysis);
        assert!(matches!(
            prepare_selected_source_frame_candidate(
                &mut analysis,
                &bsi,
                &EmdfFrameInput::AbsentPublishedPayloadV1,
                Arc::new(load_bap_parameters(Some(BAP_PROVIDER)).unwrap()),
            ),
            Err(SelectedSourceFrameWriterError::NumericPlan(
                crate::codec::selected_source_plan::SelectedSourcePlanError::PublishedPayloadRequired
            ))
        ));
    }
}
