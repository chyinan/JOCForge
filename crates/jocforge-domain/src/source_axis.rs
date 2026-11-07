// pattern: Functional Core
//! Immutable ordered source-axis IDs bound to their source owner, input and callback lineage.

use crate::{
    timeline::CodecSourceIndex, timeline::FrontendCallbackIndex,
    transaction::CallbackLineageIdentity,
};
use std::sync::Arc;

pub const SELECTED_PROFILE_BED_ROW_COUNT: u16 = 10;
pub const F03W_PREFIX_SOURCE_COUNT: u16 = 11;
pub const F03W_SOURCE_AXIS_CAPACITY: u16 = 48;
const MAX_PHYSICAL_ROW_COUNT: u16 = u8::MAX as u16 + 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedSourceRowShapeError {
    PhysicalRowCountOutsideSelectedProfile,
    SourceAxisCapacityExceeded { required: u16, capacity: u16 },
}

/// Selected MAIN row dimensions. `N`, `R`, and `S` remain separate; target `T=15` and DEE
/// internal counts `C/M` are not derived from this value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedSourceRowDimensions {
    physical: u16,
    regular: u16,
    active_sources: u16,
}

impl SelectedSourceRowDimensions {
    pub fn for_physical_rows(physical_row_count: u16) -> Result<Self, SelectedSourceRowShapeError> {
        if !(SELECTED_PROFILE_BED_ROW_COUNT..=MAX_PHYSICAL_ROW_COUNT).contains(&physical_row_count)
        {
            return Err(SelectedSourceRowShapeError::PhysicalRowCountOutsideSelectedProfile);
        }
        let regular_row_count = physical_row_count
            .checked_sub(1)
            .ok_or(SelectedSourceRowShapeError::PhysicalRowCountOutsideSelectedProfile)?;
        let active_source_count = F03W_PREFIX_SOURCE_COUNT
            .checked_add(regular_row_count)
            .ok_or(SelectedSourceRowShapeError::PhysicalRowCountOutsideSelectedProfile)?;
        if active_source_count > F03W_SOURCE_AXIS_CAPACITY {
            return Err(SelectedSourceRowShapeError::SourceAxisCapacityExceeded {
                required: active_source_count,
                capacity: F03W_SOURCE_AXIS_CAPACITY,
            });
        }
        Ok(Self {
            physical: physical_row_count,
            regular: regular_row_count,
            active_sources: active_source_count,
        })
    }

    pub fn for_active_source_count(
        active_source_count: u16,
    ) -> Result<Self, SelectedSourceRowShapeError> {
        let regular_row_count = active_source_count
            .checked_sub(F03W_PREFIX_SOURCE_COUNT)
            .ok_or(SelectedSourceRowShapeError::PhysicalRowCountOutsideSelectedProfile)?;
        let physical_row_count = regular_row_count
            .checked_add(1)
            .ok_or(SelectedSourceRowShapeError::PhysicalRowCountOutsideSelectedProfile)?;
        let dimensions = Self::for_physical_rows(physical_row_count)?;
        if dimensions.active_sources != active_source_count {
            return Err(SelectedSourceRowShapeError::PhysicalRowCountOutsideSelectedProfile);
        }
        Ok(dimensions)
    }

    pub const fn physical_row_count(self) -> u16 {
        self.physical
    }

    pub const fn regular_row_count(self) -> u16 {
        self.regular
    }

    pub const fn active_source_count(self) -> u16 {
        self.active_sources
    }

    pub const fn maximum_mappable_physical_rows() -> u16 {
        F03W_SOURCE_AXIS_CAPACITY - F03W_PREFIX_SOURCE_COUNT + 1
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SourceAxisOwnerGeneration(u64);

impl SourceAxisOwnerGeneration {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceAxisError {
    Empty,
    SourceIdsNotStrictlyIncreasing,
    SourceOriginAfterDelivery,
    CallbackLineageMismatch,
}

/// The compact S axis. Absent IDs have no entry in this value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceAxisBinding {
    source_ids: Arc<[CodecSourceIndex]>,
    owner_generation: SourceAxisOwnerGeneration,
    input_sha256: [u8; 32],
    transaction_id: CallbackLineageIdentity,
    source_origin_generation: FrontendCallbackIndex,
}

impl SourceAxisBinding {
    pub fn new(
        source_ids: impl Into<Arc<[CodecSourceIndex]>>,
        owner_generation: SourceAxisOwnerGeneration,
        input_sha256: [u8; 32],
        transaction_id: CallbackLineageIdentity,
        source_origin_generation: FrontendCallbackIndex,
    ) -> Result<Self, SourceAxisError> {
        let source_ids = source_ids.into();
        if source_ids.is_empty() {
            return Err(SourceAxisError::Empty);
        }
        if source_ids
            .windows(2)
            .any(|pair| pair[0].value() >= pair[1].value())
        {
            return Err(SourceAxisError::SourceIdsNotStrictlyIncreasing);
        }
        if source_origin_generation.value() > transaction_id.callback_index().value() {
            return Err(SourceAxisError::SourceOriginAfterDelivery);
        }
        Ok(Self {
            source_ids,
            owner_generation,
            input_sha256,
            transaction_id,
            source_origin_generation,
        })
    }

    pub fn source_ids(&self) -> &[CodecSourceIndex] {
        &self.source_ids
    }

    pub fn source_count(&self) -> usize {
        self.source_ids.len()
    }

    pub const fn owner_generation(&self) -> SourceAxisOwnerGeneration {
        self.owner_generation
    }

    pub const fn input_sha256(&self) -> &[u8; 32] {
        &self.input_sha256
    }

    pub const fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.transaction_id.callback_index()
    }

    pub const fn source_origin_generation(&self) -> FrontendCallbackIndex {
        self.source_origin_generation
    }

    pub fn validate_handoff(&self, candidate: &Self) -> Result<(), SourceAxisError> {
        if self.source_ids() != candidate.source_ids()
            || self.owner_generation != candidate.owner_generation
            || self.input_sha256 != candidate.input_sha256
        {
            return Err(SourceAxisError::CallbackLineageMismatch);
        }
        if self.transaction_id.programme_id() != candidate.transaction_id.programme_id()
            || self.transaction_id.attempt_id() != candidate.transaction_id.attempt_id()
            || self.transaction_id.callback_index() != candidate.transaction_id.callback_index()
            || self.source_origin_generation != candidate.source_origin_generation
        {
            return Err(SourceAxisError::CallbackLineageMismatch);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SelectedSourceRowDimensions, SelectedSourceRowShapeError, SourceAxisBinding,
        SourceAxisError, SourceAxisOwnerGeneration,
    };
    use crate::{
        timeline::{CodecSourceIndex, FrontendCallbackIndex},
        transaction::{AttemptId, CallbackLineageIdentity, ProgrammeId},
    };

    fn binding(
        source_ids: Vec<CodecSourceIndex>,
        callback: u32,
        origin: u32,
        input_sha256: [u8; 32],
    ) -> Result<SourceAxisBinding, SourceAxisError> {
        let programme_id = ProgrammeId::new("source-axis-test").unwrap();
        let attempt = AttemptId::new(7);
        let transaction_id = CallbackLineageIdentity::new(
            programme_id,
            attempt,
            FrontendCallbackIndex::new(callback),
        );
        SourceAxisBinding::new(
            source_ids,
            SourceAxisOwnerGeneration::new(3),
            input_sha256,
            transaction_id,
            FrontendCallbackIndex::new(origin),
        )
    }

    #[test]
    fn source_axis_preserves_only_present_ids_and_callback_lineage() {
        let ids = (0_u64..46).map(CodecSourceIndex::new).collect::<Vec<_>>();
        let axis = binding(ids.clone(), 4, 3, [0x36; 32]).unwrap();
        assert_eq!(axis.source_count(), 46);
        assert_eq!(axis.source_ids(), ids);
        assert_eq!(axis.owner_generation(), SourceAxisOwnerGeneration::new(3));
        assert_eq!(axis.callback_index(), FrontendCallbackIndex::new(4));
        assert_eq!(
            axis.source_origin_generation(),
            FrontendCallbackIndex::new(3)
        );

        let next_callback = binding(ids, 5, 3, [0x36; 32]).unwrap();
        assert_eq!(
            axis.validate_handoff(&next_callback),
            Err(SourceAxisError::CallbackLineageMismatch)
        );
    }

    #[test]
    fn source_axis_rejects_duplicate_or_unordered_ids_and_future_origin() {
        let duplicate = vec![CodecSourceIndex::new(0), CodecSourceIndex::new(0)];
        assert_eq!(
            binding(duplicate, 0, 0, [0; 32]).unwrap_err(),
            SourceAxisError::SourceIdsNotStrictlyIncreasing
        );

        let unordered = vec![CodecSourceIndex::new(1), CodecSourceIndex::new(0)];
        assert_eq!(
            binding(unordered, 0, 0, [0; 32]).unwrap_err(),
            SourceAxisError::SourceIdsNotStrictlyIncreasing
        );

        assert_eq!(
            binding(vec![CodecSourceIndex::new(0)], 0, 1, [0; 32]).unwrap_err(),
            SourceAxisError::SourceOriginAfterDelivery
        );
    }

    #[test]
    fn selected_source_row_dimensions_keep_n_r_s_separate_and_check_capacity() {
        for (physical, regular, sources) in [
            (10, 9, 20),
            (20, 19, 30),
            (35, 34, 45),
            (36, 35, 46),
            (37, 36, 47),
            (38, 37, 48),
        ] {
            let shape = SelectedSourceRowDimensions::for_physical_rows(physical).unwrap();
            assert_eq!(shape.physical_row_count(), physical);
            assert_eq!(shape.regular_row_count(), regular);
            assert_eq!(shape.active_source_count(), sources);
        }

        assert_eq!(
            SelectedSourceRowDimensions::for_physical_rows(39),
            Err(SelectedSourceRowShapeError::SourceAxisCapacityExceeded {
                required: 49,
                capacity: 48,
            })
        );
        assert_eq!(
            SelectedSourceRowDimensions::for_physical_rows(9),
            Err(SelectedSourceRowShapeError::PhysicalRowCountOutsideSelectedProfile)
        );
        assert_eq!(
            SelectedSourceRowDimensions::for_physical_rows(257),
            Err(SelectedSourceRowShapeError::PhysicalRowCountOutsideSelectedProfile)
        );
        assert_eq!(
            SelectedSourceRowDimensions::for_active_source_count(20)
                .unwrap()
                .physical_row_count(),
            10
        );
        assert_eq!(
            SelectedSourceRowDimensions::for_active_source_count(49),
            Err(SelectedSourceRowShapeError::SourceAxisCapacityExceeded {
                required: 49,
                capacity: 48,
            })
        );
    }
}
