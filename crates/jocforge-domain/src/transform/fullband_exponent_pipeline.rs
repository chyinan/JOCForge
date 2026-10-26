// pattern: Functional Core
//! Stages selected full-band F09 refresh and F10 normalization over F08 outputs.

use super::exponent_inputs::{FULL_BAND_EXPONENT_INPUT_BINS, FullBandTransformExponentInputs};
use crate::analysis::source_frame::{SharedAnalysisIdentity, SharedAnalysisReceipt};
use crate::codec::exponent_normalization::{
    ExponentNormalizationError, NormalizedExponentRow, normalize_fullband_exponents,
};
use crate::codec::exponent_refresh_state::refresh_exponent_state;
use crate::codec::policy_parameters::PolicyParameters;

const REFRESH_BEGIN_I16: i16 = 0;
const REFRESH_END_I16: i16 = 128;

/// Parent-issued source receipt plus the complete, fixed F09/F10 source plan.
///
/// The selected codec plan uses fixed six-block arrays, so its type provides the completion
/// boundary. The parent constructs this receipt only after staging that plan from one shared
/// analysis result. It carries no digest and does not advance any owner state.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct NumericPlanReceipt<'a, T> {
    shared_analysis: &'a SharedAnalysisReceipt,
    plan: T,
}

impl<'a, T> NumericPlanReceipt<'a, T> {
    /// Parent-only constructor for a completed source plan derived from the shared analysis.
    #[allow(dead_code)] // Reserved for the parent after its six-block source plan is staged.
    pub(crate) fn from_parent(shared_analysis: &'a SharedAnalysisReceipt, plan: T) -> Self {
        Self {
            shared_analysis,
            plan,
        }
    }

    pub const fn shared_analysis(&self) -> &'a SharedAnalysisReceipt {
        self.shared_analysis
    }

    pub const fn identity(&self) -> &SharedAnalysisIdentity {
        self.shared_analysis.identity()
    }

    pub const fn plan(&self) -> &T {
        &self.plan
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FullBandExponentStageControls {
    pub refresh_mode_i16: i16,
    pub refresh_special_i16: i16,
    pub normalization_span_u8: u8,
    pub normalization_option_u8: u8,
    pub normalization_gate_i16: i16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FullBandExponentStageError {
    ExponentInputError,
    Normalization(ExponentNormalizationError),
}

impl FullBandExponentStageError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ExponentInputError => "EXPONENT_INPUT_ERROR",
            Self::Normalization(error) => error.code(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct FullBandExponentStage {
    exponent_inputs: FullBandTransformExponentInputs,
    current_after_i16: [i16; FULL_BAND_EXPONENT_INPUT_BINS],
    work_after_i16: [i16; FULL_BAND_EXPONENT_INPUT_BINS],
    refresh_marker_after_i16: i16,
    normalized: NormalizedExponentRow,
}

impl FullBandExponentStage {
    pub const fn exponent_inputs(&self) -> &FullBandTransformExponentInputs {
        &self.exponent_inputs
    }

    pub const fn current_after_i16(&self) -> &[i16; FULL_BAND_EXPONENT_INPUT_BINS] {
        &self.current_after_i16
    }

    pub const fn work_after_i16(&self) -> &[i16; FULL_BAND_EXPONENT_INPUT_BINS] {
        &self.work_after_i16
    }

    pub const fn refresh_marker_after_i16(&self) -> i16 {
        self.refresh_marker_after_i16
    }

    pub const fn normalized(&self) -> &NormalizedExponentRow {
        &self.normalized
    }
}

/// Functional Core: returns F09 refresh and F10 normalization proposals for one FBW block.
///
/// The caller supplies the actual refresh state and all route controls. This function does not
/// choose a source clock, derive mode/marker/span/gate, or install the returned after-state.
pub fn stage_fullband_f09_f10(
    exponent_inputs: FullBandTransformExponentInputs,
    work_before_i16: &[i16; FULL_BAND_EXPONENT_INPUT_BINS],
    refresh_marker_before_i16: i16,
    controls: FullBandExponentStageControls,
    policy_parameters: Option<&PolicyParameters>,
) -> Result<FullBandExponentStage, FullBandExponentStageError> {
    let current_before = *exponent_inputs.power_raw_exponents_i16();
    if !matches!(refresh_marker_before_i16, 0 | 1)
        || !matches!(controls.refresh_mode_i16, 0 | 1)
        || !matches!(controls.refresh_special_i16, 0 | 1)
        || !matches!(controls.normalization_span_u8, 1 | 2 | 4)
        || controls.normalization_option_u8 > 1
        || current_before.iter().any(|value| !(0..=24).contains(value))
        || work_before_i16
            .iter()
            .any(|value| !(0..=31).contains(value))
    {
        return Err(FullBandExponentStageError::ExponentInputError);
    }

    let mut current_after_i16 = current_before;
    let mut work_after_i16 = *work_before_i16;
    let mut refresh_marker_after_i16 = refresh_marker_before_i16;
    refresh_exponent_state(
        &mut current_after_i16,
        &mut work_after_i16,
        &mut refresh_marker_after_i16,
        controls.refresh_mode_i16,
        controls.refresh_special_i16,
        REFRESH_BEGIN_I16,
        REFRESH_END_I16,
    );

    let normalized = normalize_fullband_exponents(
        &current_after_i16,
        controls.normalization_span_u8,
        controls.normalization_option_u8,
        controls.normalization_gate_i16,
        policy_parameters,
    )
    .map_err(FullBandExponentStageError::Normalization)?;

    Ok(FullBandExponentStage {
        exponent_inputs,
        current_after_i16,
        work_after_i16,
        refresh_marker_after_i16,
        normalized,
    })
}
