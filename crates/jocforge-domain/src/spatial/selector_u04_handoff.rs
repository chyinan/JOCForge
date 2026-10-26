// pattern: Functional Core
//! Binds selector ordinals to original target values and gates U04 refinement.

use crate::spatial::{
    candidate_selector::{CandidateSelectorInput, CandidateSelectorResult, SelectorStatus},
    target_slot_install::SelectedTargetSlots,
    u04_projection::{
        U04AdmissionError, U04AdmissionResult, U04ProjectionResult, u04_refinement_admission,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectorU04HandoffError {
    SelectedDomainMismatch,
    Admission(U04AdmissionError),
    RefinementMissing,
    RefinementUnexpected,
    RefinementShape,
}

impl SelectorU04HandoffError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::SelectedDomainMismatch => "SELECTOR_TARGET_DOMAIN_MISMATCH",
            Self::Admission(_) => "SELECTOR_U04_ADMISSION",
            Self::RefinementMissing => "U04_REFINEMENT_MISSING",
            Self::RefinementUnexpected => "U04_REFINEMENT_UNEXPECTED",
            Self::RefinementShape => "U04_REFINEMENT_SHAPE",
        }
    }
}

/// Callback-local selector output plus the original feature snapshot needed by U04.
#[must_use]
pub struct SelectorU04Handoff {
    selector_status: SelectorStatus,
    admission: U04AdmissionResult,
    selected_targets: SelectedTargetSlots,
    pre_decay_features: Vec<Vec<u32>>,
}

impl SelectorU04Handoff {
    pub const fn selector_status(&self) -> SelectorStatus {
        self.selector_status
    }

    pub const fn admission(&self) -> U04AdmissionResult {
        self.admission
    }

    pub const fn selected_targets(&self) -> &SelectedTargetSlots {
        &self.selected_targets
    }

    /// Original band-major candidate features gathered in the selector's chosen order.
    pub fn pre_decay_features(&self) -> &[Vec<u32>] {
        &self.pre_decay_features
    }
}

/// Joins selector ordinals back to the original candidate rows without rerunning selection.
pub fn prepare_selector_u04_handoff(
    input: &CandidateSelectorInput,
    result: &CandidateSelectorResult,
) -> Result<SelectorU04Handoff, SelectorU04HandoffError> {
    let selected_count = usize::from(result.selected_count());
    let selected_ids = result.selected_ids();
    let selected_ordinals = result.selected_initial_ordinals();
    let selected_attributes = result.selected_attributes();
    if selected_ids.len() != selected_count
        || selected_ordinals.len() != selected_count
        || selected_attributes.len() != selected_count
        || input.positions_u32.len() != input.candidate_ids.len()
        || input.attributes_u32.len() != input.candidate_ids.len()
        || input
            .feature_u32
            .iter()
            .any(|band| band.len() != input.candidate_ids.len())
    {
        return Err(SelectorU04HandoffError::SelectedDomainMismatch);
    }

    let mut selected_positions = Vec::with_capacity(selected_count);
    let mut selected_seen = vec![false; input.candidate_ids.len()];
    for ((selected_id, ordinal), selected_attribute) in selected_ids
        .iter()
        .zip(selected_ordinals)
        .zip(selected_attributes)
    {
        let ordinal = usize::try_from(*ordinal)
            .map_err(|_| SelectorU04HandoffError::SelectedDomainMismatch)?;
        if selected_seen.get(ordinal).is_none_or(|seen| *seen)
            || input.candidate_ids.get(ordinal) != Some(selected_id)
            || input.attributes_u32.get(ordinal) != Some(selected_attribute)
        {
            return Err(SelectorU04HandoffError::SelectedDomainMismatch);
        }
        selected_seen[ordinal] = true;
        selected_positions.push(input.positions_u32[ordinal]);
    }

    let mut pre_decay_features = Vec::with_capacity(input.feature_u32.len());
    for band in &input.feature_u32 {
        let selected_band = selected_ordinals
            .iter()
            .map(|ordinal| {
                usize::try_from(*ordinal)
                    .ok()
                    .and_then(|ordinal| band.get(ordinal).copied())
                    .ok_or(SelectorU04HandoffError::SelectedDomainMismatch)
            })
            .collect::<Result<Vec<_>, _>>()?;
        pre_decay_features.push(selected_band);
    }

    let admission = u04_refinement_admission(result.status(), result.selected_count())
        .map_err(SelectorU04HandoffError::Admission)?;
    Ok(SelectorU04Handoff {
        selector_status: result.status(),
        admission,
        selected_targets: SelectedTargetSlots {
            xyz_words: selected_positions,
            ids_u32: selected_ids.to_vec(),
            attributes_u32: selected_attributes.to_vec(),
        },
        pre_decay_features,
    })
}

/// Applies a U04 position result only when status explicitly admitted refinement.
pub fn finish_selector_u04_handoff(
    handoff: SelectorU04Handoff,
    refinement: Option<&U04ProjectionResult>,
) -> Result<SelectedTargetSlots, SelectorU04HandoffError> {
    finish_selector_u04_handoff_with_xyz(
        handoff,
        refinement.map(U04ProjectionResult::projected_xyz_u32),
    )
}

/// Applies an already-produced U04 XYZ projection only when CAPACITY_REACHED admitted it.
pub fn finish_selector_u04_handoff_with_xyz(
    handoff: SelectorU04Handoff,
    projected_xyz_u32: Option<&[[u32; 3]]>,
) -> Result<SelectedTargetSlots, SelectorU04HandoffError> {
    let mut selected = handoff.selected_targets;
    if handoff.admission.refinement_called() {
        let projected_xyz_u32 =
            projected_xyz_u32.ok_or(SelectorU04HandoffError::RefinementMissing)?;
        if projected_xyz_u32.len() != usize::from(handoff.admission.selected_count()) {
            return Err(SelectorU04HandoffError::RefinementShape);
        }
        selected.xyz_words = projected_xyz_u32.to_vec();
    } else if projected_xyz_u32.is_some() {
        return Err(SelectorU04HandoffError::RefinementUnexpected);
    }
    Ok(selected)
}
