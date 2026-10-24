// pattern: Functional Core
use jocforge_domain::transform::history_owner::{TransformHistoryOwner, TransformSourceSlot};
use jocforge_domain::transform::long_transform::{
    LongTransformHistory, TransformBranch, long_transform,
};
use jocforge_domain::transform::workspace::{
    TRANSFORM_ARENA_BYTES, TRANSFORM_DESCRIPTOR_BYTES, WorkspaceTailCanaries,
    initialize_transform_workspace,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/transform_history_owner";
const PACKAGE_MANIFEST_SHA256: &str =
    "d21eef82d1b135c0219c7e6cb9b40da8052016f907e27813eb2de823af275327";
const INDEPENDENT_CHECK_SHA256: &str =
    "19d81e5342e46910d83b358e60de513c3a7de084417ec8434c95b6da6833544b";
const PACKAGE_PAYLOADS: [(&str, usize, &str); 5] = [
    (
        "CONTRACT.md",
        3608,
        "51ace5f4634a3c6dd6d1fbc0af0cce19dd7c4d95fef49b4474843cdfca0eab54",
    ),
    (
        "INPUTS.json",
        1990,
        "d473035fd6f3ca0f69dddd4ff99e1160b749f01e2fa6421697f553868a467fa7",
    ),
    (
        "EXPECTED.json",
        218868,
        "7841824f2339e3beb36c37471befb50ed89f9a0c91b4c8ba66eba79a98173af2",
    ),
    (
        "PARAMETERS.json",
        4662,
        "a247fb0e0c595edf0b53cf4a943bf701223f13c663924814f549ec6da7ba4a18",
    ),
    (
        "PROVENANCE.json",
        3684,
        "002995ea62ef2d7ba464daac179a7152e132edccb520026bdd7c28a4ffb9a1b7",
    ),
];

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(crate_root().join(FIXTURE_DIRECTORY).join(name)).unwrap()
}

fn fixture_json(name: &str) -> Value {
    serde_json::from_slice(&fixture_bytes(name)).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn assert_words(actual: &[u32], expected: &Value, label: &str) {
    let expected: Vec<u32> = expected
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .collect();
    assert_eq!(actual, expected.as_slice(), "{label}");
}

fn history_input(kind: &str, call: usize) -> [u32; 256] {
    let mut input = [0.0_f32; 256];
    match kind {
        "zero" => {}
        "impulse" => input[call] = 0.5,
        "periodic" => {
            let cycle = [0.0_f32, 0.125, -0.25, 0.5, -0.5, 0.25];
            for (index, value) in input.iter_mut().enumerate() {
                *value = cycle[(index + call) % cycle.len()];
            }
        }
        other => panic!("unexpected history-owner recipe {other}"),
    }
    input.map(f32::to_bits)
}

fn branch_from_name(name: &str) -> TransformBranch {
    match name {
        "primary" => TransformBranch::Primary,
        "companion" => TransformBranch::Companion,
        other => panic!("unexpected history branch {other}"),
    }
}

#[test]
fn package_and_dependency_identities_are_hash_bound() {
    let check_bytes = fixture_bytes("INDEPENDENT-CHECK.json");
    let check: Value = serde_json::from_slice(&check_bytes).unwrap();
    let provenance = fixture_json("PROVENANCE.json");
    assert_eq!(sha256(&check_bytes), INDEPENDENT_CHECK_SHA256);
    assert_eq!(check["manifest_sha256"], PACKAGE_MANIFEST_SHA256);
    assert_eq!(check["payload_count"], 5);
    assert_eq!(check["payload_hashes_match"], true);
    assert_eq!(check["source_manifest_check_and_payloads_match"], true);
    assert_eq!(check["independent_history_words"], 1536);
    assert_eq!(check["history_replay"], "PASS all words");
    assert_eq!(
        check["workspace_initializer"],
        "PASS zero fields/null pointers/0xA5 tail canaries"
    );
    assert_eq!(check["native_fold_calls"], 432);
    assert_eq!(check["cold_slots"], 12);
    assert_eq!(check["carry_checks"], 420);
    assert_eq!(check["carry_mismatches"], 0);
    assert_eq!(check["full_reset_eof"], "NOT_RUN");
    assert_eq!(check["whole_f09_closed"], false);
    assert_eq!(
        provenance["verified_dependency"]["manifest_sha256"],
        "4eaa2690e30ed0705404d7a6cd835e8637c2f933319d60be8812d900cb6521c1"
    );
    assert_eq!(
        provenance["verified_dependency"]["check_sha256"],
        "0574d3f4aff6460340709216fd1d0c141c47baa68367f2c43abbc446ba2157b5"
    );

    for (name, expected_bytes, expected_hash) in PACKAGE_PAYLOADS {
        let bytes = fixture_bytes(name);
        assert_eq!(bytes.len(), expected_bytes, "{name} bytes");
        assert_eq!(sha256(&bytes), expected_hash, "{name} SHA-256");
    }
}

#[test]
fn six_source_slots_keep_primary_and_companion_history_banks_independent() {
    let inputs = fixture_json("INPUTS.json");
    let expected = fixture_json("EXPECTED.json");
    assert_eq!(inputs["expected_used_as_seed"], false);
    assert_eq!(inputs["history_sequences"].as_array().unwrap().len(), 6);
    assert_eq!(expected["history_sequences"].as_array().unwrap().len(), 6);
    assert_eq!(
        TransformSourceSlot::ALL.map(TransformSourceSlot::source_index),
        [0, 1, 2, 3, 4, 5]
    );

    let mut primary_final_states = Vec::new();
    let mut companion_final_states = Vec::new();
    let mut replayed_words = 0;

    for (sequence_index, (sequence, expected_sequence)) in inputs["history_sequences"]
        .as_array()
        .unwrap()
        .iter()
        .zip(expected["history_sequences"].as_array().unwrap())
        .enumerate()
    {
        let kind = sequence["kind"].as_str().unwrap();
        let branch = branch_from_name(sequence["branch"].as_str().unwrap());
        assert_eq!(sequence["calls"], 2);
        assert_eq!(sequence["input_count"], 256);
        assert_eq!(sequence["history_count"], 128);
        assert_eq!(sequence["initial_history_zero"], true);
        let mut history = LongTransformHistory::zeroed();

        for call in 0..2 {
            let output = long_transform(&history_input(kind, call), &history, branch);
            let expected_step = &expected_sequence["steps"][call];
            assert_eq!(expected_step["call"], call);
            assert_words(
                output.history_after().words(),
                &expected_step["history_bits"],
                "history continuity transition",
            );
            replayed_words += output.history_after().words().len();
            history = output.history_after().clone();
        }

        match branch {
            TransformBranch::Primary => primary_final_states.push((sequence_index, history)),
            TransformBranch::Companion => companion_final_states.push((sequence_index, history)),
        }
    }
    assert_eq!(replayed_words, 1536);
    assert_eq!(primary_final_states.len(), 3);
    assert_eq!(companion_final_states.len(), 3);

    let primary_template = primary_final_states
        .iter()
        .find(|(recipe, _)| *recipe == 2)
        .unwrap()
        .1
        .clone();
    let companion_template = companion_final_states
        .iter()
        .find(|(recipe, _)| *recipe == 3)
        .unwrap()
        .1
        .clone();
    let zero_primary = primary_final_states
        .iter()
        .find(|(recipe, _)| *recipe == 0)
        .unwrap()
        .1
        .clone();

    let mut owner = TransformHistoryOwner::zeroed();
    for source in TransformSourceSlot::ALL {
        assert!(
            owner
                .history(source, TransformBranch::Primary)
                .words()
                .iter()
                .all(|word| *word == 0)
        );
        assert!(
            owner
                .history(source, TransformBranch::Companion)
                .words()
                .iter()
                .all(|word| *word == 0)
        );
        owner.store_history_after(source, TransformBranch::Primary, &primary_template);
        owner.store_history_after(source, TransformBranch::Companion, &companion_template);
    }

    owner.store_history_after(
        TransformSourceSlot::FullBand0,
        TransformBranch::Primary,
        &zero_primary,
    );
    assert_eq!(
        owner.history(TransformSourceSlot::FullBand0, TransformBranch::Primary),
        &zero_primary
    );
    assert_eq!(
        owner.history(TransformSourceSlot::FullBand0, TransformBranch::Companion),
        &companion_template
    );
    for source in TransformSourceSlot::ALL.into_iter().skip(1) {
        assert_eq!(
            owner.history(source, TransformBranch::Primary),
            &primary_template
        );
        assert_eq!(
            owner.history(source, TransformBranch::Companion),
            &companion_template
        );
    }
}

#[test]
fn bounded_workspace_initializer_sets_typed_fields_and_preserves_tail_canaries() {
    let inputs = fixture_json("INPUTS.json");
    let expected = fixture_json("EXPECTED.json");
    let initializer = &inputs["workspace_initializer"];
    let expected = &expected["workspace_initializer"];
    assert_eq!(initializer["descriptor_bytes"], TRANSFORM_DESCRIPTOR_BYTES);
    assert_eq!(initializer["arena_bytes"], TRANSFORM_ARENA_BYTES);
    assert_eq!(initializer["fill_byte"], 165);
    assert_eq!(initializer["blocks"], 6);
    assert_eq!(initializer["channels"], 5);

    let workspace =
        initialize_transform_workspace(WorkspaceTailCanaries::new([0xa5; 24], [0xa5; 16]));
    assert_eq!(workspace.return_value(), 0);

    let actual_power = workspace
        .power_rows()
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    let expected_power = expected["power_rows"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|row| row.as_array().unwrap())
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(actual_power, expected_power);

    let actual_bias = workspace
        .bias_rows()
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    let expected_bias = expected["bias_rows"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|row| row.as_array().unwrap())
        .map(|value| i16::try_from(value.as_i64().unwrap()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(actual_bias, expected_bias);

    let actual_scratch = workspace
        .scratch_exponent_rows()
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    let expected_scratch = expected["scratch_exponent_rows"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|row| row.as_array().unwrap())
        .map(|value| i16::try_from(value.as_i64().unwrap()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(actual_scratch, expected_scratch);
    assert!(
        workspace
            .full_band_current_row_bound()
            .iter()
            .all(|bound| !bound)
    );
    assert_eq!(
        expected["current_row_nulls"].as_array().unwrap(),
        &vec![Value::Bool(true); 30]
    );

    let actual_lfe = workspace
        .lfe_scratch_rows()
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    let expected_lfe = expected["lfe_scratch_rows"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|row| row.as_array().unwrap())
        .map(|value| i16::try_from(value.as_i64().unwrap()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(actual_lfe, expected_lfe);
    assert!(workspace.lfe_current_row_bound().iter().all(|bound| !bound));
    assert_eq!(
        expected["lfe_current_nulls"].as_array().unwrap(),
        &vec![Value::Bool(true); 6]
    );
    let expected_work = expected["work_row"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| i16::try_from(value.as_i64().unwrap()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        workspace.shared_work_row().as_slice(),
        expected_work.as_slice()
    );
    assert_eq!(
        workspace.tail_canaries().arena_tail().as_slice(),
        expected["arena_tail_bytes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| u8::try_from(value.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        workspace.tail_canaries().descriptor_tail().as_slice(),
        expected["descriptor_tail_bytes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| u8::try_from(value.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>()
    );
}
