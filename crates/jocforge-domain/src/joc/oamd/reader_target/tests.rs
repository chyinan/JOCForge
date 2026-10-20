// pattern: Functional Core
use super::ReaderTargetState;
use super::{
    READER_ROWS, RECORD_BYTES, ReaderInvocationIndex, ReaderTargetCallbackInputs,
    ReaderTargetError, ReaderTargetUpdateReceipt, SELECTED_PROFILE_ID, hex, make_fit_record,
    read_record_word, stage_reader_target,
};
use crate::timeline::FrontendCallbackIndex;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const CONTINUOUS_VECTORS: &str = include_str!(
    "../../../../tests/fixtures/reader_target/READER56-TARGET-UPDATES-VERIFICATION.json"
);
const RECEIPT_VECTORS: &str =
    include_str!("../../../../tests/fixtures/reader_target/READER-TARGET-RECEIPT-VECTORS.json");
const MODE2_API_SCHEMA: &[u8] = include_bytes!(
    "../../../../tests/fixtures/reader_target/mode2_current_previous/API-SCHEMA.json"
);
const MODE2_CONTRACT: &[u8] =
    include_bytes!("../../../../tests/fixtures/reader_target/mode2_current_previous/CONTRACT.md");
const MODE2_OBSERVED_VECTORS: &[u8] = include_bytes!(
    "../../../../tests/fixtures/reader_target/mode2_current_previous/OBSERVED-VECTORS.json"
);
const PACKAGE_MANIFEST: &str =
    include_str!("../../../../tests/fixtures/f01e_provider/MANIFEST.json");
const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";

#[test]
fn cold_state_digest_matches_the_complete_schema_preimage() {
    let state = ReaderTargetState::cold();
    let actual = hex(&state.canonical_sha256());
    assert_eq!(
        actual,
        "309a342e6fcb4d6dee13b8da144c7e61880ce1fbd4a5e555291bd3c62470a588"
    );
}

#[test]
fn mode2_supplement_assets_match_the_washed_manifest_entries() {
    for (name, bytes, expected_size, expected_sha256) in [
        (
            "API-SCHEMA.json",
            MODE2_API_SCHEMA,
            810,
            "f9d5e6762de238dc64531a55bc087384306ac72e61cc0dcc92a50ab8160ea45e",
        ),
        (
            "CONTRACT.md",
            MODE2_CONTRACT,
            2038,
            "5d137597e61970255097bd3140c45cc3af40e2aba2216175bdb9e73d5f5bff36",
        ),
        (
            "OBSERVED-VECTORS.json",
            MODE2_OBSERVED_VECTORS,
            2125,
            "39c3e212eed8c636a52cbb01f304c9b460f5bef19123d7d2f4cb4bb8f9f6a84d",
        ),
    ] {
        assert_eq!(bytes.len(), expected_size, "{name}");
        assert_eq!(hex(&Sha256::digest(bytes)), expected_sha256, "{name}");
    }
}

#[test]
fn direct_fit_record_projection_copies_current_xyz_from_an_independent_recipe() {
    let observed: Value = serde_json::from_slice(MODE2_OBSERVED_VECTORS).unwrap();
    let run = &observed["run"];
    assert_eq!(run["frame_samples"].as_u64(), Some(3072));
    assert_eq!(run["source_count"].as_u64(), Some(48));
    assert_eq!(run["target_count"].as_u64(), Some(15));
    assert_eq!(run["selected_mode"].as_u64(), Some(2));
    assert_eq!(run["active_indices"], serde_json::json!([0, 1, 2, 3, 4, 5]));

    // Use an independent local input recipe; observed output words are assertions only.
    let recipe = recipe_data(2);
    let input = recipe.inputs(2);
    for row in 0..15 {
        let record = make_fit_record(&input, row, false, input.assignment_active[row], 0);
        assert_eq!(
            &record[6..9],
            &input.current_xyz_words[row * 3..row * 3 + 3],
            "fit record row {row} copies the current XYZ words"
        );
    }
}

#[test]
fn successful_direct_stage_carries_current_xyz_into_the_next_owner_state() {
    let recipe = recipe_data(2);
    let committed = state_after_priming();
    let input = recipe.inputs(2);
    assert_ne!(input.current_xyz_words, &committed.current_xyz_words);
    let proposal = stage_reader_target(&committed, &input).unwrap();
    assert_eq!(
        proposal.next_state.current_xyz_words,
        *input.current_xyz_words
    );
}

#[test]
fn selected_parameter_bytes_match_manifest_and_implemented_values() {
    let manifest = serde_json::from_str::<Value>(PACKAGE_MANIFEST).unwrap();
    assert_eq!(
        hex(&Sha256::digest(PACKAGE_MANIFEST.as_bytes())),
        PACKAGE_MANIFEST_SHA256
    );
    let assets: [(&str, &[u8]); 4] = [
        (
            "parameters/reader56-target-profile-v1.json",
            include_bytes!(
                "../../../../tests/fixtures/reader_target/reader56-target-profile-v1.json"
            ),
        ),
        (
            "parameters/reader56-main-panner-v1.json",
            include_bytes!("../../../../tests/fixtures/reader_target/reader56-main-panner-v1.json"),
        ),
        (
            "parameters/oamd-coordinate-grid-q15.json",
            include_bytes!(
                "../../../../tests/fixtures/reader_target/oamd-coordinate-grid-q15.json"
            ),
        ),
        (
            "parameters/meter-oar-secondary8-math-v1.json",
            include_bytes!(
                "../../../../tests/fixtures/reader_target/meter-oar-secondary8-math-v1.json"
            ),
        ),
    ];
    for (path, bytes) in assets {
        let matches = manifest["files"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entry| entry["file"].as_str() == Some(path))
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "manifest entry for {path}");
        assert_eq!(matches[0]["bytes"].as_u64().unwrap(), bytes.len() as u64);
        assert_eq!(
            hex(&Sha256::digest(bytes)),
            matches[0]["sha256"].as_str().unwrap(),
            "manifest hash for {path}"
        );
    }

    let profile: Value = serde_json::from_slice(assets[0].1).unwrap();
    let (mode, trim_flags, default_group) = super::type11::selected_trim_profile();
    assert_eq!(profile["timing"]["base"].as_u64(), Some(0));
    assert_eq!(profile["timing"]["position"].as_u64(), Some(0));
    assert_eq!(profile["timing"]["duration"].as_u64(), Some(1536));
    assert_eq!(profile["timing"]["precision"].as_u64(), Some(32));
    assert_eq!(profile["trim"]["mode"].as_u64(), Some(u64::from(mode)));
    assert_eq!(json_u8_array(&profile["trim"]["group_flags"]), trim_flags);
    assert_eq!(profile["trim"]["groups"].as_array().unwrap().len(), 9);
    for group in profile["trim"]["groups"].as_array().unwrap() {
        assert_eq!(json_i32_array::<5>(group), default_group);
    }
    assert_eq!(profile["trim"]["row_flags"].as_array().unwrap().len(), 16);

    let panner: Value = serde_json::from_slice(assets[1].1).unwrap();
    assert_eq!(
        panner["geometry_i32_rows"],
        serde_json::to_value(super::target_panner::selected_geometry()).unwrap()
    );
    assert_eq!(
        panner["quarter_pi_f32_u32"].as_u64().unwrap() as u32,
        super::target_panner::selected_equal_power_words()[0]
    );
    assert_eq!(
        panner["half_f32_u32"].as_u64().unwrap() as u32,
        super::target_panner::selected_equal_power_words()[1]
    );

    let secondary: Value = serde_json::from_slice(assets[3].1).unwrap();
    assert_eq!(panner["geometry_i32_rows"], secondary["generated_i32_rows"]);
    let grid: Value = serde_json::from_slice(assets[2].1).unwrap();
    let xy = grid["xy_q15"].as_array().unwrap();
    let z = grid["z_magnitude_q15"].as_array().unwrap();
    assert_eq!(xy.len(), 63);
    assert_eq!(z.len(), 16);
    for index in 0..63_u32 {
        assert_eq!(
            super::type11::decode_xy_for_test(index).unwrap(),
            xy[index as usize].as_u64().unwrap() as u32
        );
    }
    for index in 0..16_u32 {
        assert_eq!(
            super::type11::decode_z_for_test(index).unwrap(),
            z[index as usize].as_u64().unwrap() as i32
        );
    }
}

#[test]
fn reader_materializer_leaf_vectors_include_mute_and_previous_extra_edges() {
    let vectors: Value = serde_json::from_str(CONTINUOUS_VECTORS).unwrap();
    for case in vectors["reader_materializer_cases"].as_array().unwrap() {
        let input = &case["input"];
        let decoded_rows: [super::type11::DecodedOamdRow; READER_ROWS] =
            std::array::from_fn(|row| {
                let decoded = &input["decoded_rows"][row];
                let xyz = decoded["xyz_q15"].as_array().unwrap();
                super::type11::DecodedOamdRow {
                    kind: decoded["kind"].as_u64().unwrap() as u8,
                    mute: decoded["mute"].as_u64().unwrap() != 0,
                    x: xyz[0].as_u64().unwrap() as u32,
                    y: xyz[1].as_u64().unwrap() as u32,
                    z: xyz[2].as_i64().unwrap() as i32,
                    component: decoded["component"].as_u64().unwrap() as u32,
                    diffuse: decoded["diffuse"].as_u64().unwrap() as u32,
                    z_valid: decoded["z_valid"].as_u64().unwrap() as u32,
                    extra: decoded["extra28"].as_i64().unwrap() as i32,
                    gain: decoded["gain"].as_u64().unwrap() as u32,
                    row_flag: input["row_flags"][row].as_u64().unwrap() as u32,
                }
            });
        let previous: [[u8; RECORD_BYTES]; READER_ROWS] = std::array::from_fn(|row| {
            decode_hex(input["previous_reader56_hex"][row].as_str().unwrap())
                .try_into()
                .unwrap()
        });
        let actual = super::reader_records::materialize(&previous, &decoded_rows).unwrap();
        for row in 0..READER_ROWS {
            let expected = decode_hex(case["expected_reader56_hex"][row].as_str().unwrap());
            assert_eq!(
                actual[row].as_slice(),
                expected.as_slice(),
                "{} row {row}",
                case["id"]
            );
        }
        assert_eq!(
            hex(&Sha256::digest(
                actual.iter().flatten().copied().collect::<Vec<_>>()
            )),
            case["expected_reader56_sha256"]
        );
    }
}

#[test]
fn target_panner_leaf_vectors_and_rejections_match_the_selected_contract() {
    let vectors: Value = serde_json::from_str(CONTINUOUS_VECTORS).unwrap();
    for case in vectors["target_leaf_cases"].as_array().unwrap() {
        let input = &case["input"];
        assert_eq!(
            input["panner_parameter_id"].as_str().unwrap(),
            SELECTED_PROFILE_ID
        );
        let row = case["id"]
            .as_str()
            .unwrap()
            .rsplit('-')
            .next()
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let record: [u8; RECORD_BYTES] = decode_hex(input["reader56_hex"].as_str().unwrap())
            .try_into()
            .unwrap();
        let actual = super::target_panner::generate_target_for_test(row, &record).unwrap();
        let expected: [u32; 6] = case["expected_target_u32"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_u64().unwrap() as u32)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        assert_eq!(actual, expected, "{}", case["id"]);
    }
    for case in vectors["target_leaf_rejection_cases"].as_array().unwrap() {
        let record: [u8; RECORD_BYTES] = decode_hex(case["input_reader56_hex"].as_str().unwrap())
            .try_into()
            .unwrap();
        let actual = super::target_panner::generate_target_for_test(1, &record)
            .err()
            .unwrap();
        assert_eq!(actual.code(), case["expected_error"].as_str().unwrap());
        assert_eq!(
            case["expected_error"].as_str().unwrap(),
            case["actual_error"].as_str().unwrap()
        );
    }
}

#[test]
fn selected_callback_chain_matches_postcomputation_state_and_receipt_assertions() {
    let vectors: Value = serde_json::from_str(CONTINUOUS_VECTORS).unwrap();
    let receipt_vectors: Value = serde_json::from_str(RECEIPT_VECTORS).unwrap();
    let mut state = ReaderTargetState::cold();
    let mut receipt_count = 0;
    for callback in 0..8_u64 {
        let recipe = recipe_data(callback);
        let input = recipe.inputs(callback);
        let expected_case = &vectors["cases"][callback as usize];
        assert_fixture_input_hashes(&recipe, &expected_case["input"]);
        let before_sha = hex(&state.canonical_sha256());
        assert_eq!(before_sha, expected_case["expected_before_state_sha256"]);
        let proposal = stage_reader_target(&state, &input).unwrap();
        if callback < 2 {
            let mut marker_bytes = Vec::new();
            let mut control_bytes = Vec::new();
            for row in 0..15 {
                marker_bytes.extend_from_slice(&proposal.next_state.history.slots[0][row].payload);
                for word in 1..14 {
                    control_bytes.extend_from_slice(
                        &proposal.next_state.history.slots[0][row].payload[word * 4..word * 4 + 4],
                    );
                }
            }
            assert_eq!(
                hex(&Sha256::digest(&marker_bytes)),
                expected_case["expected"]["marker_records56_sha256"]
            );
            assert_eq!(
                hex(&Sha256::digest(&control_bytes)),
                expected_case["expected"]["delivered_control52_sha256"]
            );
        }
        assert_eq!(
            hex(&proposal.next_state.canonical_sha256()),
            expected_case["expected_after_state_sha256"]
        );
        match (callback >= 2, proposal.pending_receipt.as_ref()) {
            (false, None) => assert!(
                !expected_case["expected"]["reader_enabled"]
                    .as_bool()
                    .unwrap()
            ),
            (true, Some(receipt)) => {
                receipt_count += 1;
                let expected =
                    &receipt_vectors["positive_cases"][callback as usize]["expected_receipt"];
                assert_receipt(receipt, expected);

                let controls = std::array::from_fn(|row| {
                    std::array::from_fn(|word| {
                        read_record_word(
                            &proposal.next_state.history.slots[0][row].payload,
                            word + 1,
                        )
                    })
                });
                let (decoded_rows, wire) = super::type11::roundtrip(&controls).unwrap();
                assert_eq!(decoded_rows.len(), READER_ROWS);
                assert_eq!(hex(&wire), expected_case["expected"]["type11_wire_hex"]);
                assert_eq!(wire.len(), expected_case["expected"]["type11_wire_bytes"]);
                assert_eq!(
                    hex(&Sha256::digest(&wire)),
                    expected_case["expected"]["type11_wire_sha256"]
                );

                let actual_targets = payload_words(&receipt.target_payload_f32le);
                let expected_targets = expected_case["expected"]["targets_16x6_u32"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_u64().unwrap() as u32)
                    .collect::<Vec<_>>();
                assert_eq!(actual_targets, expected_targets);
                assert_eq!(
                    hex(&receipt.reader_records_sha256),
                    expected_case["expected"]["reader56_16x56_sha256"]
                );
            }
            (true, None) => panic!("reader callback {callback} omitted its pending receipt"),
            (false, Some(_)) => panic!("priming callback {callback} emitted a receipt"),
        }
        state = proposal.next_state;
    }
    assert_eq!(receipt_count, 6);
}

#[test]
fn producer_rejections_return_no_proposal_and_leave_committed_state_unchanged() {
    let committed = ReaderTargetState::cold();
    let before_sha = committed.canonical_sha256();
    let cold_recipe = recipe_data(0);

    let mut bad_callback = cold_recipe.inputs(0);
    bad_callback.callback_index = FrontendCallbackIndex::new(1);
    assert_eq!(
        stage_reader_target(&committed, &bad_callback)
            .err()
            .unwrap(),
        ReaderTargetError::CallbackOrder
    );
    assert_eq!(committed.canonical_sha256(), before_sha);

    let mut invalid = cold_recipe.inputs(0);
    invalid.reader_enabled = true;
    assert_eq!(
        stage_reader_target(&committed, &invalid).err().unwrap(),
        ReaderTargetError::CallbackOrder
    );
    assert_eq!(committed.canonical_sha256(), before_sha);

    let empty_row: [f32; 0] = [];
    let mut old_rows_missing = cold_recipe.inputs(0);
    old_rows_missing.previous_overlap_rows[14] = &empty_row;
    assert_eq!(
        stage_reader_target(&committed, &old_rows_missing)
            .err()
            .unwrap(),
        ReaderTargetError::InputError
    );

    let short_row = [0.0_f32; 1535];
    let mut mixer_row_short = cold_recipe.inputs(0);
    mixer_row_short.current_mixer_rows[0] = &short_row;
    assert_eq!(
        stage_reader_target(&committed, &mixer_row_short)
            .err()
            .unwrap(),
        ReaderTargetError::InputError
    );

    let mut snapshot_missing = cold_recipe.inputs(0);
    snapshot_missing.snapshot_keys_complete = false;
    assert_eq!(
        stage_reader_target(&committed, &snapshot_missing)
            .err()
            .unwrap(),
        ReaderTargetError::InputError
    );

    let mut invalid_profile = cold_recipe.inputs(0);
    invalid_profile.profile_id = "unregistered-profile";
    assert_eq!(
        stage_reader_target(&committed, &invalid_profile)
            .err()
            .unwrap(),
        ReaderTargetError::ProfileUnsupported
    );
    assert_eq!(committed.canonical_sha256(), before_sha);

    let mut retired = ReaderTargetState::cold();
    retired.retired = true;
    assert_eq!(
        stage_reader_target(&retired, &cold_recipe.inputs(0))
            .err()
            .unwrap(),
        ReaderTargetError::PipelineRetired
    );

    let mut warm = state_after_priming();
    let warm_before = warm.canonical_sha256();
    warm.reader_invocations = ReaderInvocationIndex::new(1);
    let warm_mutated_before = warm.canonical_sha256();
    let reader_state_error = stage_reader_target(&warm, &recipe_data(2).inputs(2))
        .err()
        .unwrap();
    assert_eq!(reader_state_error, ReaderTargetError::StateError);
    assert_eq!(warm.canonical_sha256(), warm_mutated_before);
    assert_ne!(warm_mutated_before, warm_before);

    let mut component_six_recipe = recipe_data(2);
    component_six_recipe.current_attributes[1] = 12;
    let component_six_inputs = component_six_recipe.inputs(2);
    let component_six = stage_reader_target(&state_after_priming(), &component_six_inputs)
        .err()
        .unwrap();
    assert_eq!(component_six, ReaderTargetError::ProfileUnsupported);

    let mut changed_recipe = recipe_data(1);
    changed_recipe.current_ids[0] += 1;
    changed_recipe.previous_overlap[0][0] = f32::MAX;
    changed_recipe.current_mixer[0][0] = f32::MAX;
    let numeric_inputs = changed_recipe.inputs(1);
    let primed = state_after_callback_zero();
    let primed_before = primed.canonical_sha256();
    assert_eq!(
        stage_reader_target(&primed, &numeric_inputs).err().unwrap(),
        ReaderTargetError::NumericFailure
    );
    assert_eq!(primed.canonical_sha256(), primed_before);
}

fn state_after_callback_zero() -> ReaderTargetState {
    let state = ReaderTargetState::cold();
    let recipe = recipe_data(0);
    stage_reader_target(&state, &recipe.inputs(0))
        .unwrap()
        .next_state
}

fn state_after_priming() -> ReaderTargetState {
    let after_zero = state_after_callback_zero();
    let recipe = recipe_data(1);
    stage_reader_target(&after_zero, &recipe.inputs(1))
        .unwrap()
        .next_state
}

struct RecipeData {
    current_xyz_words: [u32; 45],
    current_ids: [u32; 15],
    current_attributes: [u32; 15],
    assignment_active: [u8; 15],
    active_target_indices: Vec<u8>,
    matrix: Vec<f32>,
    source_flags: [u8; 48],
    previous_overlap: Vec<[f32; 1536]>,
    current_mixer: Vec<[f32; 3072]>,
    transaction_id: String,
}

impl RecipeData {
    fn inputs(&self, callback: u64) -> ReaderTargetCallbackInputs<'_> {
        ReaderTargetCallbackInputs {
            callback_index: FrontendCallbackIndex::new(callback),
            reader_enabled: callback >= 2,
            snapshot_keys_complete: true,
            programme_id: "VECTOR-PROGRAMME-0",
            attempt_id: "VECTOR-ATTEMPT-0",
            transaction_id: &self.transaction_id,
            profile_id: SELECTED_PROFILE_ID,
            current_xyz_words: &self.current_xyz_words,
            current_ids: &self.current_ids,
            current_attributes: &self.current_attributes,
            assignment_active: &self.assignment_active,
            active_target_indices: &self.active_target_indices,
            matrix: &self.matrix,
            source_flags: &self.source_flags,
            previous_overlap_rows: std::array::from_fn(|row| self.previous_overlap[row].as_slice()),
            current_mixer_rows: std::array::from_fn(|row| self.current_mixer[row].as_slice()),
        }
    }
}

fn recipe_data(callback: u64) -> RecipeData {
    let current_ids = std::array::from_fn(|row| {
        if callback < 2 {
            100 + row as u32
        } else {
            1000 + row as u32
        }
    });
    let current_attributes = std::array::from_fn(|row| 2 * (row as u32 % 6));
    let mut current_xyz_words = [0_u32; 45];
    let coordinate_grid = [0.0_f32, 0.5_f32, 1.0_f32];
    for row in 0..15 {
        let x_grid = row % 3;
        let x_index = if callback >= 2 {
            (x_grid + 1) % 3
        } else {
            x_grid
        };
        let y_index = (row / 3) % 3;
        current_xyz_words[row * 3] = coordinate_grid[x_index].to_bits();
        current_xyz_words[row * 3 + 1] = coordinate_grid[y_index].to_bits();
        current_xyz_words[row * 3 + 2] = 0.0_f32.to_bits();
    }
    let mut assignment_active = [1_u8; 15];
    if (4..=6).contains(&callback) {
        assignment_active[..8].fill(0);
    }
    let active_target_indices = (0..15_u8)
        .filter(|row| assignment_active[usize::from(*row)] == 1)
        .collect::<Vec<_>>();
    let mut matrix = vec![0.0_f32; active_target_indices.len() * 48];
    for (ordinal, row) in active_target_indices.iter().enumerate() {
        let source = (usize::from(*row) + callback as usize) % 48;
        matrix[ordinal * 48 + source] = 1.0;
    }
    let source_flags = [0_u8; 48];
    let mut previous_overlap = vec![[0.0_f32; 1536]; 15];
    let mut current_mixer = vec![[0.0_f32; 3072]; 15];
    for row in 0..15 {
        previous_overlap[row][row] = 1.0;
        current_mixer[row][row] = 1.0;
        if callback == 2 && row % 2 == 0 {
            current_mixer[row][row] = 1.0 / 1024.0;
        }
        if callback == 2 && row % 2 == 1 {
            previous_overlap[row][row] = 1.0 / 1024.0;
        }
    }
    RecipeData {
        current_xyz_words,
        current_ids,
        current_attributes,
        assignment_active,
        active_target_indices,
        matrix,
        source_flags,
        previous_overlap,
        current_mixer,
        transaction_id: format!("VECTOR-TX-{callback}"),
    }
}

fn assert_fixture_input_hashes(recipe: &RecipeData, expected: &Value) {
    let snapshot_hash = sha256_u32le(
        recipe
            .current_xyz_words
            .iter()
            .copied()
            .chain(recipe.current_ids.iter().copied())
            .chain(recipe.current_attributes.iter().copied())
            .chain(recipe.assignment_active.iter().copied().map(u32::from)),
    );
    assert_eq!(hex(&snapshot_hash), expected["snapshot_sha256"]);

    let matrix_hash = sha256_u32le(recipe.matrix.iter().map(|value| value.to_bits()));
    assert_eq!(hex(&matrix_hash), expected["matrix_u32le_sha256"]);

    let old_overlap_hash = sha256_u32le(
        recipe
            .previous_overlap
            .iter()
            .flat_map(|row| row.iter().map(|value| value.to_bits())),
    );
    assert_eq!(hex(&old_overlap_hash), expected["old_overlap_f32le_sha256"]);

    let new_mixer_hash = sha256_u32le(
        recipe
            .current_mixer
            .iter()
            .flat_map(|row| row.iter().map(|value| value.to_bits())),
    );
    assert_eq!(hex(&new_mixer_hash), expected["new_mixer_f32le_sha256"]);
}

fn sha256_u32le(words: impl IntoIterator<Item = u32>) -> [u8; 32] {
    let mut bytes = Vec::new();
    for word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    Sha256::digest(bytes).into()
}

fn assert_receipt(actual: &ReaderTargetUpdateReceipt, expected: &Value) {
    let mut expected_fields = expected.as_object().unwrap().clone();
    let expected_semantic = expected_fields
        .remove("receipt_semantic_sha256")
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned();
    let expected_fields = expected_fields.into_iter().collect::<BTreeMap<_, _>>();
    let mut actual_fields = BTreeMap::<String, Value>::new();
    actual_fields.insert(
        "schema".to_owned(),
        Value::String("reader-target-update-receipt.v1".to_owned()),
    );
    actual_fields.insert(
        "programme_id".to_owned(),
        Value::String(actual.programme_id.clone()),
    );
    actual_fields.insert(
        "attempt_id".to_owned(),
        Value::String(actual.attempt_id.clone()),
    );
    actual_fields.insert(
        "transaction_id".to_owned(),
        Value::String(actual.transaction_id.clone()),
    );
    actual_fields.insert(
        "callback_generation_g".to_owned(),
        Value::from(actual.callback_generation.value()),
    );
    actual_fields.insert(
        "reader_invocation_q".to_owned(),
        Value::from(actual.reader_invocation.value()),
    );
    actual_fields.insert(
        "profile_id".to_owned(),
        Value::String(actual.profile_id.to_owned()),
    );
    actual_fields.insert(
        "history_producer_generations".to_owned(),
        Value::Array(
            actual
                .history_producer_generations
                .iter()
                .map(|generation| Value::from(generation.canonical_value()))
                .collect(),
        ),
    );
    actual_fields.insert(
        "state_before_sha256".to_owned(),
        Value::String(hex(&actual.state_before_sha256)),
    );
    actual_fields.insert(
        "state_after_sha256".to_owned(),
        Value::String(hex(&actual.state_after_sha256)),
    );
    actual_fields.insert(
        "reader_records_shape".to_owned(),
        serde_json::json!([16, 56]),
    );
    actual_fields.insert("reader_records_bytes".to_owned(), Value::from(896));
    actual_fields.insert(
        "reader_records_sha256".to_owned(),
        Value::String(hex(&actual.reader_records_sha256)),
    );
    actual_fields.insert("target_shape".to_owned(), serde_json::json!([16, 6]));
    actual_fields.insert("target_dtype".to_owned(), Value::String("F32LE".to_owned()));
    actual_fields.insert("target_bytes".to_owned(), Value::from(384));
    actual_fields.insert(
        "target_sha256".to_owned(),
        Value::String(hex(&actual.target_sha256)),
    );
    actual_fields.insert(
        "target_payload_hex".to_owned(),
        Value::String(hex(&actual.target_payload_f32le)),
    );
    actual_fields.insert("persistent_target_state_bytes".to_owned(), Value::from(0));
    actual_fields.insert("persistent_target_state_tag".to_owned(), Value::Null);
    assert_eq!(actual_fields, expected_fields);
    assert_eq!(
        actual.programme_id,
        expected["programme_id"].as_str().unwrap()
    );
    assert_eq!(actual.attempt_id, expected["attempt_id"].as_str().unwrap());
    assert_eq!(
        actual.transaction_id,
        expected["transaction_id"].as_str().unwrap()
    );
    assert_eq!(
        actual.callback_generation.value(),
        expected["callback_generation_g"].as_u64().unwrap()
    );
    assert_eq!(
        actual.reader_invocation.value(),
        expected["reader_invocation_q"].as_u64().unwrap()
    );
    assert_eq!(actual.profile_id, expected["profile_id"].as_str().unwrap());
    assert_eq!(
        hex(&actual.state_before_sha256),
        expected["state_before_sha256"].as_str().unwrap()
    );
    assert_eq!(
        hex(&actual.state_after_sha256),
        expected["state_after_sha256"].as_str().unwrap()
    );
    assert_eq!(
        hex(&actual.reader_records_sha256),
        expected["reader_records_sha256"].as_str().unwrap()
    );
    assert_eq!(
        hex(&actual.target_sha256),
        expected["target_sha256"].as_str().unwrap()
    );
    assert_eq!(
        hex(&actual.target_payload_f32le),
        expected["target_payload_hex"].as_str().unwrap()
    );
    assert_eq!(hex(&actual.semantic_sha256), expected_semantic);
    let generations = actual
        .history_producer_generations
        .iter()
        .map(|generation| generation.canonical_value())
        .collect::<Vec<_>>();
    let expected_generations = expected["history_producer_generations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_i64().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(generations, expected_generations);
}

fn payload_words(payload: &[u8; 384]) -> Vec<u32> {
    payload
        .chunks_exact(4)
        .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
        .collect()
}

fn json_u8_array<const N: usize>(value: &Value) -> [u8; N] {
    let entries = value.as_array().unwrap();
    assert_eq!(entries.len(), N);
    std::array::from_fn(|index| entries[index].as_u64().unwrap() as u8)
}

fn json_i32_array<const N: usize>(value: &Value) -> [i32; N] {
    let entries = value.as_array().unwrap();
    assert_eq!(entries.len(), N);
    std::array::from_fn(|index| entries[index].as_i64().unwrap() as i32)
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|digits| {
            let digits = std::str::from_utf8(digits).unwrap();
            u8::from_str_radix(digits, 16).unwrap()
        })
        .collect()
}
