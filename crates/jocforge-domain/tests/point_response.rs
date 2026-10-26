// pattern: Imperative Shell
use jocforge_domain::{
    spatial::{
        point_response::{
            PointResponseAngleMode, PointResponseGateInput, PointResponseSourceInput,
            PointResponseSourceSnapshot, prepare_point_response_sources,
            produce_point_response_columns,
        },
        selected_p15_geometry::generate_selected_p15_geometry,
        source_flags::SourceControlSnapshotClock,
    },
    timeline::{CodecSourceIndex, FrontendCallbackIndex, NormalCallbackCount},
    transaction::{AttemptId, CallbackLineageIdentity, ProgrammeId},
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const PACKAGE_MANIFEST: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/SPATIAL-POINT-RESPONSE-PRODUCER-CONTRACT-V1/MANIFEST.json"
);
const INPUT_ONLY_VECTORS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/SPATIAL-POINT-RESPONSE-PRODUCER-CONTRACT-V1/INPUT-ONLY-VECTORS.json"
);
const PACKAGE_MANIFEST_SHA256: &str =
    "255b5a1b6d3648099ec31abd61c490b02987a5f320fcea7a3cf51d9f524343d8";

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn vectors() -> Value {
    let manifest: Value = serde_json::from_slice(PACKAGE_MANIFEST).unwrap();
    assert_eq!(sha256(PACKAGE_MANIFEST), PACKAGE_MANIFEST_SHA256);
    let entry = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["file"] == "INPUT-ONLY-VECTORS.json")
        .unwrap();
    assert_eq!(
        entry["bytes"].as_u64().unwrap(),
        u64::try_from(INPUT_ONLY_VECTORS.len()).unwrap()
    );
    assert_eq!(
        entry["sha256"].as_str().unwrap(),
        sha256(INPUT_ONLY_VECTORS)
    );
    serde_json::from_slice(INPUT_ONLY_VECTORS).unwrap()
}

fn coordinate_word(half_steps: usize) -> u32 {
    match half_steps {
        0 => 0x0000_0000,
        1 => 0x3f00_0000,
        2 => 0x3f80_0000,
        _ => panic!("input recipe has only three half-step positions"),
    }
}

fn sources_from_index_recipe() -> Vec<PointResponseSourceInput> {
    let rows = (0..48)
        .map(|source| {
            PointResponseSourceInput::new(
                CodecSourceIndex::new(source as u64),
                [
                    coordinate_word(source % 3),
                    coordinate_word((source / 3) % 3),
                    coordinate_word((source / 9) % 3),
                ],
                [0x3e00_0000, 0x3e80_0000, 0x3ec0_0000, 0x3f00_0000][source % 4],
                0,
                0,
                0,
                1,
            )
        })
        .collect::<Vec<_>>();
    rows.into_iter().rev().collect()
}

fn selected_geometry() -> jocforge_domain::spatial::selected_p15_geometry::SelectedP15Geometry {
    generate_selected_p15_geometry(0x159b, true).unwrap()
}

fn transaction_id(callback_index: FrontendCallbackIndex) -> CallbackLineageIdentity {
    CallbackLineageIdentity::new(
        ProgrammeId::new("POINT_RESPONSE_TEST").unwrap(),
        AttemptId::new(0),
        callback_index,
    )
}

#[test]
fn response_source_union_uses_global_ids_and_keeps_audio_and_feature_widths_separate() {
    let vectors = vectors();
    assert!(vectors.get("expected_outputs_included").is_none());
    assert!(vectors["xyz_vectors"].as_array().is_some());
    let candidates = vectors["gate_inputs"]["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>();
    let processing = vectors["gate_inputs"]["processing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>();
    let callback_index = FrontendCallbackIndex::new(4);
    let transaction_id = transaction_id(callback_index);
    let source_snapshot = PointResponseSourceSnapshot::new(
        callback_index,
        transaction_id.clone(),
        SourceControlSnapshotClock::for_normal_callback(
            callback_index,
            NormalCallbackCount::new(5),
        )
        .unwrap(),
        sources_from_index_recipe(),
    )
    .unwrap();
    let gate = PointResponseGateInput::new(callback_index, transaction_id, candidates, processing)
        .unwrap();
    let batch = prepare_point_response_sources(source_snapshot, gate).unwrap();
    assert_eq!(batch.candidate_ids(), &[2, 47]);
    assert_eq!(batch.processing_ids(), &[2, 11]);
    assert_eq!(batch.source_control_clock().source_origin(), callback_index);
    assert_eq!(
        batch
            .response_source_ids()
            .iter()
            .map(|source| source.value())
            .collect::<Vec<_>>(),
        [2, 11, 47]
    );
    assert_eq!(batch.candidate_feature_width_f32_bits(), &[0, 0]);
    let source2 = batch.generated_sources()[0];
    let source11 = batch.generated_sources()[1];
    let source47 = batch.generated_sources()[2];
    assert_eq!(source2.source_index().value(), 2);
    assert_eq!(source11.source_index().value(), 11);
    assert_eq!(source47.source_index().value(), 47);
    assert_eq!(source2.original_audio_width_f32_bits(), 0x3ec0_0000);
    assert_eq!(source47.original_audio_width_f32_bits(), 0x3f00_0000);
    assert!(
        batch
            .generated_sources()
            .iter()
            .all(|source| { source.post_audio_feature_width_f32_bits() == 0 })
    );

    let production = produce_point_response_columns(&selected_geometry(), batch).unwrap();
    assert_eq!(production.columns().len(), 3);
    assert_eq!(
        production.angle_mode(),
        PointResponseAngleMode::MathematicalR64R32
    );
    assert_eq!(production.response_source_ids().len(), 3);
    assert!(production.columns().iter().all(|column| {
        column
            .response_f32_bits()
            .iter()
            .all(|word| f32::from_bits(*word).is_finite() && f32::from_bits(*word) >= 0.0)
    }));
}

#[test]
fn p15_basis_points_produce_identity_columns_at_exact_geometry_points() {
    let geometry = selected_geometry();
    let mut sources = (0..48)
        .map(|source| {
            let basis = geometry.points()[source.min(14)].xyz_f32_bits();
            PointResponseSourceInput::new(
                CodecSourceIndex::new(source as u64),
                basis,
                0x3f00_0000,
                0,
                0,
                0,
                1,
            )
        })
        .collect::<Vec<_>>();
    sources.reverse();
    let candidates = (0..15).collect::<Vec<u32>>();
    let callback_index = FrontendCallbackIndex::new(0);
    let transaction_id = transaction_id(callback_index);
    let source_snapshot = PointResponseSourceSnapshot::new(
        callback_index,
        transaction_id.clone(),
        SourceControlSnapshotClock::for_normal_callback(
            callback_index,
            NormalCallbackCount::new(1),
        )
        .unwrap(),
        sources,
    )
    .unwrap();
    let gate = PointResponseGateInput::new(callback_index, transaction_id, candidates, Vec::new())
        .unwrap();
    let batch = prepare_point_response_sources(source_snapshot, gate).unwrap();

    let production = produce_point_response_columns(&geometry, batch).unwrap();

    assert_eq!(production.columns().len(), 15);
    for (ordinal, column) in production.columns().iter().enumerate() {
        assert_eq!(column.source_index().value(), ordinal as u64);
        for (basis, word) in column.response_f32_bits().iter().copied().enumerate() {
            assert_eq!(word, if basis == ordinal { 0x3f80_0000 } else { 0 });
        }
    }
}

#[test]
fn empty_union_checks_global_mode_but_skips_unconsumed_source_branches() {
    let mut sources = sources_from_index_recipe();
    let row = sources
        .iter_mut()
        .find(|source| source.source_index().value() == 17)
        .unwrap();
    *row = PointResponseSourceInput::new(
        row.source_index(),
        row.xyz_f32_bits(),
        row.original_audio_width_f32_bits(),
        row.feature_mode(),
        1,
        8,
        9,
    );

    let callback_index = FrontendCallbackIndex::new(1);
    let transaction_id = transaction_id(callback_index);
    let drain_clock =
        SourceControlSnapshotClock::for_cached_drain(callback_index, NormalCallbackCount::new(1))
            .unwrap();
    let source_snapshot = PointResponseSourceSnapshot::new(
        callback_index,
        transaction_id.clone(),
        drain_clock,
        sources.clone(),
    )
    .unwrap();
    let gate = PointResponseGateInput::new(
        callback_index,
        transaction_id.clone(),
        Vec::new(),
        Vec::new(),
    )
    .unwrap();
    let empty = prepare_point_response_sources(source_snapshot, gate).unwrap();
    assert!(empty.source_control_clock().is_cached_drain());
    assert_eq!(
        empty.source_control_clock().source_origin(),
        FrontendCallbackIndex::new(0)
    );
    assert!(empty.generated_sources().is_empty());
    assert!(
        produce_point_response_columns(&selected_geometry(), empty)
            .unwrap()
            .columns()
            .is_empty()
    );

    let mut invalid_mode = sources;
    let row = invalid_mode
        .iter_mut()
        .find(|source| source.source_index().value() == 17)
        .unwrap();
    *row = PointResponseSourceInput::new(
        row.source_index(),
        row.xyz_f32_bits(),
        row.original_audio_width_f32_bits(),
        1,
        row.source_skip(),
        row.spatial_enum(),
        row.coordinate_parameter(),
    );
    let source_snapshot = PointResponseSourceSnapshot::new(
        callback_index,
        transaction_id.clone(),
        drain_clock,
        invalid_mode,
    )
    .unwrap();
    let gate = PointResponseGateInput::new(callback_index, transaction_id, Vec::new(), Vec::new())
        .unwrap();
    assert_eq!(
        prepare_point_response_sources(source_snapshot, gate)
            .unwrap_err()
            .code(),
        "POINT_RESPONSE_SOURCE_MODE"
    );
}

#[test]
fn response_source_and_gate_must_share_transaction_identity() {
    let callback_index = FrontendCallbackIndex::new(0);
    let source_transaction = transaction_id(callback_index);
    let gate_transaction = CallbackLineageIdentity::new(
        ProgrammeId::new("DIFFERENT_POINT_RESPONSE_TEST").unwrap(),
        AttemptId::new(0),
        callback_index,
    );
    let source_snapshot = PointResponseSourceSnapshot::new(
        callback_index,
        source_transaction,
        SourceControlSnapshotClock::for_normal_callback(
            callback_index,
            NormalCallbackCount::new(1),
        )
        .unwrap(),
        sources_from_index_recipe(),
    )
    .unwrap();
    let gate =
        PointResponseGateInput::new(callback_index, gate_transaction, vec![2], vec![11]).unwrap();

    assert_eq!(
        prepare_point_response_sources(source_snapshot, gate)
            .unwrap_err()
            .code(),
        "POINT_RESPONSE_SOURCE_GATE_IDENTITY"
    );
}
