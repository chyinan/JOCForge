use super::*;
use crate::spatial::target_slot_install::{
    SelectedTargetSlots, TargetSlotInstallResult, TargetSlotState, install_target_slots,
};
use crate::{
    timeline::FrontendCallbackIndex,
    transaction::{AttemptId, CallbackLineageIdentity, ProgrammeId},
};

fn geometry() -> SelectedP15Geometry {
    crate::spatial::selected_p15_geometry::generate_selected_p15_geometry(0x159b, true).unwrap()
}

fn state(geometry: &SelectedP15Geometry, active_flags: [u8; TARGET_SLOT_COUNT]) -> TargetSlotState {
    TargetSlotState {
        xyz_words: std::array::from_fn(|slot| geometry.points()[slot].xyz_f32_bits()),
        ids_u32: std::array::from_fn(|slot| 100 + u32::try_from(slot).unwrap()),
        attributes_u32: std::array::from_fn(|slot| u32::try_from(slot).unwrap() << 4),
        previous_xyz_words: std::array::from_fn(|slot| geometry.points()[slot].xyz_f32_bits()),
        previous_ids_u32: std::array::from_fn(|slot| 100 + u32::try_from(slot).unwrap()),
        previous_attributes_u32: std::array::from_fn(|slot| u32::try_from(slot).unwrap() << 4),
        active_flags,
    }
}

fn empty_selection() -> SelectedTargetSlots {
    SelectedTargetSlots {
        xyz_words: Vec::new(),
        ids_u32: Vec::new(),
        attributes_u32: Vec::new(),
    }
}

fn project(
    geometry: &SelectedP15Geometry,
    installation: &TargetSlotInstallResult,
) -> Result<TargetContextProjection, TargetContextError> {
    let callback_index = FrontendCallbackIndex::new(0);
    let transaction_id = CallbackLineageIdentity::new(
        ProgrammeId::new("target-context-test").unwrap(),
        AttemptId::new(0),
        callback_index,
    );
    build_target_context_projection(geometry, installation, transaction_id, callback_index)
}

fn assert_identity_context(context: &[[u32; TARGET_SLOT_COUNT]; TARGET_SLOT_COUNT]) {
    for (basis, row) in context.iter().enumerate() {
        for (slot, word) in row.iter().copied().enumerate() {
            assert_eq!(word, if basis == slot { 0x3f80_0000 } else { 0 });
        }
    }
}

#[test]
fn empty_selection_keeps_active_installed_targets_and_uses_slot_order_for_compaction() {
    let geometry = geometry();
    let mut active_flags = [0; TARGET_SLOT_COUNT];
    for slot in [2, 6, 11] {
        active_flags[slot] = 1;
    }
    let old = state(&geometry, active_flags);
    let installation = install_target_slots(&old, &empty_selection()).unwrap();
    assert!(installation.empty_branch());

    let projection = project(&geometry, &installation).unwrap();
    assert!(projection.selected_branch_empty());
    assert_eq!(projection.compact_target_count(), 3);
    assert_identity_context(projection.full_context_u32());
    assert_eq!(
        projection
            .destinations()
            .iter()
            .map(|entry| (entry.compact_ordinal().value(), entry.target_slot().value()))
            .collect::<Vec<_>>(),
        [(0, 2), (1, 6), (2, 11)]
    );
    assert_eq!(
        projection.compact_target_xyz_u32(),
        [old.xyz_words[2], old.xyz_words[6], old.xyz_words[11]]
    );
    for basis in 0..TARGET_SLOT_COUNT {
        assert_eq!(
            projection.compact_context_u32()[basis].as_ref(),
            [
                if basis == 2 { 0x3f80_0000 } else { 0 },
                if basis == 6 { 0x3f80_0000 } else { 0 },
                if basis == 11 { 0x3f80_0000 } else { 0 },
            ]
        );
    }
}

#[test]
fn zero_active_destinations_still_validate_and_compute_all_fifteen_contexts() {
    let geometry = geometry();
    let old = state(&geometry, [0; TARGET_SLOT_COUNT]);
    let installation = install_target_slots(&old, &empty_selection()).unwrap();
    assert!(installation.empty_branch());

    let projection = project(&geometry, &installation).unwrap();
    assert_eq!(projection.compact_target_count(), 0);
    assert!(projection.destinations().is_empty());
    assert!(projection.compact_target_xyz_u32().is_empty());
    assert_eq!(projection.compact_context_u32().len(), TARGET_SLOT_COUNT);
    assert!(
        projection
            .compact_context_u32()
            .iter()
            .all(|row| row.is_empty())
    );
    assert_identity_context(projection.full_context_u32());
}

#[test]
fn nonempty_installation_uses_the_matched_slot_and_packs_fit_words() {
    let geometry = geometry();
    let mut old = state(&geometry, [1; TARGET_SLOT_COUNT]);
    old.attributes_u32[8] = 0x0000_16a0;
    let selected = SelectedTargetSlots {
        xyz_words: vec![geometry.points()[8].xyz_f32_bits()],
        ids_u32: vec![old.ids_u32[8]],
        attributes_u32: vec![0x0000_16a0],
    };
    let installation = install_target_slots(&old, &selected).unwrap();
    assert!(!installation.empty_branch());
    assert_eq!(
        installation.next_state().active_flags,
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
    );

    let projection = project(&geometry, &installation).unwrap();
    assert!(!projection.selected_branch_empty());
    assert_eq!(projection.compact_target_count(), 1);
    assert_eq!(projection.destinations()[0].target_slot().value(), 8);
    assert_eq!(projection.compact_target_xyz_u32(), [old.xyz_words[8]]);
    assert_identity_context(projection.full_context_u32());

    let fit = projection.fit_records()[8];
    assert_eq!(fit.target_slot().value(), 8);
    let words = fit.words();
    assert_eq!(&words[0..2], &[0, 0]);
    assert_eq!((words[2], words[3], words[4]), (10, 2, 1));
    assert_eq!(words[5], 0);
    assert_eq!(&words[6..9], &old.xyz_words[8]);
    assert_eq!(
        (words[9], words[10], words[11], words[12], words[13]),
        (0, 0, 0, 0, 1)
    );
}

#[test]
fn inactive_fit_records_are_validated_for_mode_and_spatial_enum() {
    let geometry = geometry();
    let mut mode_state = state(&geometry, [0; TARGET_SLOT_COUNT]);
    mode_state.attributes_u32[9] = 1;
    let mode_installation = install_target_slots(&mode_state, &empty_selection()).unwrap();
    assert_eq!(
        project(&geometry, &mode_installation).err(),
        Some(TargetContextError::UnsupportedFitContext {
            target_slot: TargetSlotIndex(9),
            field: TargetFitContextField::Mode,
        })
    );

    let mut spatial_state = state(&geometry, [0; TARGET_SLOT_COUNT]);
    spatial_state.attributes_u32[3] = 0b100;
    let spatial_installation = install_target_slots(&spatial_state, &empty_selection()).unwrap();
    assert_eq!(
        project(&geometry, &spatial_installation).err(),
        Some(TargetContextError::UnsupportedFitContext {
            target_slot: TargetSlotIndex(3),
            field: TargetFitContextField::SpatialEnum,
        })
    );
}
