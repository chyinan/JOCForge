// pattern: Functional Core
//! F05H target fit records, F03P full context, and destination-ordered compact A projection.

use super::{
    point_response::{PointResponseError, produce_target_context_response_column},
    selected_p15_geometry::{SelectedP15Geometry, SelectedP15GeometryIdentity},
    target_slot_install::TargetSlotInstallResult,
};
use crate::{timeline::FrontendCallbackIndex, transaction::CallbackLineageIdentity};

const TARGET_SLOT_COUNT: usize = 15;
const FIT_RECORD_WORD_COUNT: usize = 14;
const SELECTED_LAYOUT: u32 = 0x159b;
const FRONT_SIDE_Y_Q15: i32 = 5_285;
const POSITIVE_ZERO_BITS: u32 = 0;

/// Installed target-slot identity, distinct from compact solver ordinal.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TargetSlotIndex(u8);

impl TargetSlotIndex {
    /// Creates an index after the caller has checked the fixed target-slot domain.
    pub(crate) const fn from_validated(value: u8) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

/// Compact active-target column ordinal, distinct from the installed target slot.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CompactTargetOrdinal(u8);

impl CompactTargetOrdinal {
    pub const fn value(self) -> u8 {
        self.0
    }
}

/// One item in the installed-slot to compact-target mapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetDestination {
    compact_ordinal: CompactTargetOrdinal,
    target_slot: TargetSlotIndex,
}

impl TargetDestination {
    pub const fn compact_ordinal(self) -> CompactTargetOrdinal {
        self.compact_ordinal
    }

    pub const fn target_slot(self) -> TargetSlotIndex {
        self.target_slot
    }
}

/// Context field checked for every F05H fit record, including inactive slots.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetFitContextField {
    Mode,
    Width,
    Extra,
    SpatialEnum,
    CoordinateParameter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetContextError {
    UnsupportedGeometry,
    InvalidInstallation,
    CallbackIdentityMismatch,
    UnsupportedFitContext {
        target_slot: TargetSlotIndex,
        field: TargetFitContextField,
    },
    PointResponse(PointResponseError),
}

/// The complete fourteen-word F05H.1 record for one installed target slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetContextFitRecord {
    target_slot: TargetSlotIndex,
    words: [u32; FIT_RECORD_WORD_COUNT],
}

impl TargetContextFitRecord {
    pub const fn target_slot(self) -> TargetSlotIndex {
        self.target_slot
    }

    pub const fn words(self) -> [u32; FIT_RECORD_WORD_COUNT] {
        self.words
    }

    pub const fn xyz_f32_bits(self) -> [u32; 3] {
        [self.words[6], self.words[7], self.words[8]]
    }
}

/// Full fifteen-slot context plus the compact active-slot projection for one installation result.
///
/// `selected_branch_empty` preserves F03G's empty-selection branch independently from `n`.
/// An empty selected branch can retain old active slots and therefore have `n > 0`.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetContextProjection {
    transaction_id: CallbackLineageIdentity,
    callback_index: FrontendCallbackIndex,
    geometry_identity: SelectedP15GeometryIdentity,
    selected_branch_empty: bool,
    fit_records: [TargetContextFitRecord; TARGET_SLOT_COUNT],
    full_context_u32: [[u32; TARGET_SLOT_COUNT]; TARGET_SLOT_COUNT],
    compact_target_count: u8,
    destinations: Box<[TargetDestination]>,
    compact_context_u32: Box<[Box<[u32]>]>,
    compact_target_xyz_u32: Box<[[u32; 3]]>,
}

impl TargetContextProjection {
    pub fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn geometry_identity(&self) -> SelectedP15GeometryIdentity {
        self.geometry_identity
    }

    pub const fn selected_branch_empty(&self) -> bool {
        self.selected_branch_empty
    }

    pub fn fit_records(&self) -> &[TargetContextFitRecord; TARGET_SLOT_COUNT] {
        &self.fit_records
    }

    /// Full context is basis-major and retains every installed target slot.
    pub const fn full_context_u32(&self) -> &[[u32; TARGET_SLOT_COUNT]; TARGET_SLOT_COUNT] {
        &self.full_context_u32
    }

    /// Destinations are in ascending installed target-slot order.
    pub fn destinations(&self) -> &[TargetDestination] {
        &self.destinations
    }

    /// Compact A is basis-major with one column per destination, in the same order.
    pub fn compact_context_u32(&self) -> &[Box<[u32]>] {
        &self.compact_context_u32
    }

    /// Current target XYZ rows use the same compact destination order as A.
    pub fn compact_target_xyz_u32(&self) -> &[[u32; 3]] {
        &self.compact_target_xyz_u32
    }

    pub fn compact_target_count(&self) -> u8 {
        self.compact_target_count
    }
}

/// Builds every target fit context, then gathers active installed slots into compact A.
///
/// The input must be a completed `install_target_slots` result. The caller supplies the existing
/// F03W callback identity because F03G installation itself has no callback receipt; this value is
/// the parent call's local binding, not independent evidence about the installation's origin.
/// The function always validates and computes all fifteen fit-record contexts before it projects
/// the active destination list. This includes `n == 0` and the empty-selection branch, whose
/// retained target state is separate from the selected count.
pub fn build_target_context_projection(
    geometry: &SelectedP15Geometry,
    installation: &TargetSlotInstallResult,
    transaction_id: CallbackLineageIdentity,
    callback_index: FrontendCallbackIndex,
) -> Result<TargetContextProjection, TargetContextError> {
    if transaction_id.callback_index() != callback_index {
        return Err(TargetContextError::CallbackIdentityMismatch);
    }
    let geometry_identity = geometry.identity();
    if geometry.layout() != SELECTED_LAYOUT
        || !geometry_identity.profile_enabled()
        || geometry.front_side_y_q15() != FRONT_SIDE_Y_Q15
        || geometry.points().len() != TARGET_SLOT_COUNT
    {
        return Err(TargetContextError::UnsupportedGeometry);
    }

    let installed = installation.next_state();
    if installed.active_flags.iter().any(|active| *active > 1)
        || installed
            .xyz_words
            .iter()
            .flatten()
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(TargetContextError::InvalidInstallation);
    }

    let fit_records: [TargetContextFitRecord; TARGET_SLOT_COUNT] = (0..TARGET_SLOT_COUNT)
        .map(|slot| make_fit_record(installed, slot))
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| TargetContextError::InvalidInstallation)?;

    let mut full_context_u32 = [[POSITIVE_ZERO_BITS; TARGET_SLOT_COUNT]; TARGET_SLOT_COUNT];
    for (slot, record) in fit_records.iter().copied().enumerate() {
        let column = produce_target_context_response_column(geometry, record.xyz_f32_bits())
            .map_err(TargetContextError::PointResponse)?;
        for (basis, value) in column.into_iter().enumerate() {
            full_context_u32[basis][slot] = value;
        }
    }

    let mut destinations = Vec::new();
    for (slot, active) in installed.active_flags.iter().copied().enumerate() {
        if active == 1 {
            let compact_ordinal = u8::try_from(destinations.len())
                .map_err(|_| TargetContextError::InvalidInstallation)?;
            destinations.push(TargetDestination {
                compact_ordinal: CompactTargetOrdinal(compact_ordinal),
                target_slot: target_slot_index(slot)?,
            });
        }
    }

    let compact_context_u32 = (0..TARGET_SLOT_COUNT)
        .map(|basis| {
            destinations
                .iter()
                .map(|destination| {
                    full_context_u32[basis][usize::from(destination.target_slot.value())]
                })
                .collect::<Vec<_>>()
                .into_boxed_slice()
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let compact_target_count =
        u8::try_from(destinations.len()).map_err(|_| TargetContextError::InvalidInstallation)?;
    let compact_target_xyz_u32 = destinations
        .iter()
        .map(|destination| installed.xyz_words[usize::from(destination.target_slot.value())])
        .collect::<Vec<_>>()
        .into_boxed_slice();

    Ok(TargetContextProjection {
        transaction_id,
        callback_index,
        geometry_identity,
        selected_branch_empty: installation.empty_branch(),
        fit_records,
        full_context_u32,
        compact_target_count,
        destinations: destinations.into_boxed_slice(),
        compact_context_u32,
        compact_target_xyz_u32,
    })
}

fn make_fit_record(
    installed: &super::target_slot_install::TargetSlotState,
    slot: usize,
) -> Result<TargetContextFitRecord, TargetContextError> {
    let target_slot = target_slot_index(slot)?;
    let attributes = installed.attributes_u32[slot];
    let xyz = installed.xyz_words[slot];
    if xyz.iter().any(|word| !f32::from_bits(*word).is_finite()) {
        return Err(TargetContextError::InvalidInstallation);
    }

    let mut words = [POSITIVE_ZERO_BITS; FIT_RECORD_WORD_COUNT];
    words[2] = (attributes >> 4) & 15;
    words[3] = (attributes >> 8) & 3;
    words[4] = (attributes >> 10) & 3;
    words[6..9].copy_from_slice(&xyz);
    words[9] = attributes & 1;
    words[12] = (attributes >> 1) & 7;
    words[13] = 1;

    let checks = [
        (TargetFitContextField::Mode, words[9] == 0),
        (
            TargetFitContextField::Width,
            words[10] == POSITIVE_ZERO_BITS,
        ),
        (
            TargetFitContextField::Extra,
            words[11] == POSITIVE_ZERO_BITS,
        ),
        (TargetFitContextField::SpatialEnum, words[12] == 0),
        (TargetFitContextField::CoordinateParameter, words[13] == 1),
    ];
    for (field, supported) in checks {
        if !supported {
            return Err(TargetContextError::UnsupportedFitContext { target_slot, field });
        }
    }

    Ok(TargetContextFitRecord { target_slot, words })
}

fn target_slot_index(slot: usize) -> Result<TargetSlotIndex, TargetContextError> {
    u8::try_from(slot)
        .map(TargetSlotIndex)
        .map_err(|_| TargetContextError::InvalidInstallation)
}

#[cfg(test)]
mod tests;
