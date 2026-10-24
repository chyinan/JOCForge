// pattern: Functional Core
use jocforge_domain::codec::exponent_refresh_state::refresh_exponent_state;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/exponent_refresh_state";
const PACKAGE_MANIFEST_SHA256: &str =
    "f46a4c67bf019649d8f770a04d59b3809ed9eda7b3d5268aab4e3bddd03505e5";
const INDEPENDENT_CHECK_SHA256: &str =
    "0f42620d973b858f3a14d930e4c6c830be7af0e6a6528561edbe9c81cc4035a5";
const UPSTREAM_WASH_MANIFEST_SHA256: &str =
    "21ec09924b008e699f2261c990144bb8e2d23fecc49562e9a270e5846974807b";
const UPSTREAM_INDEPENDENT_CHECK_SHA256: &str =
    "f50c7c742d5a6c7a3a691aace7b592ec5f3bd759a6e9314575682a8ffa0211e6";
const PACKAGE_PAYLOADS: [(&str, usize, &str); 5] = [
    (
        "CONTRACT.md",
        5194,
        "d363f933a3871be31dc489c2a9163b3f988aa45f6ca881856f276db85fff8c93",
    ),
    (
        "INPUTS.json",
        3010,
        "5af60faa4ff3215b38bbe13f351ff2775d0d8662d5944955f6360ab29c0f602b",
    ),
    (
        "EXPECTED.json",
        3821,
        "bdb79c0f5df7be6a0aedf4d2ef9812b10718574cafcc44021abb2c776c52295b",
    ),
    (
        "PARAMETERS.json",
        1340,
        "e404bac6e584f2b9fa37aa63edf811095e3b7b199b5863120d6c55127e32c55f",
    ),
    (
        "PROVENANCE.json",
        3060,
        "0505c8a4352573b59946764c72a4f8268def48fda06b638ac26d9c26f2209115",
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

fn i16_values(values: &Value) -> Vec<i16> {
    values
        .as_array()
        .unwrap()
        .iter()
        .map(|value| i16::try_from(value.as_i64().unwrap()).unwrap())
        .collect()
}

#[test]
fn package_and_upstream_identities_are_hash_bound_without_a_manifest_fixture() {
    let check_bytes = fixture_bytes("INDEPENDENT-CHECK.json");
    let check: Value = serde_json::from_slice(&check_bytes).unwrap();
    let provenance = fixture_json("PROVENANCE.json");

    assert_eq!(sha256(&check_bytes), INDEPENDENT_CHECK_SHA256);
    assert_eq!(check["manifest_sha256"], PACKAGE_MANIFEST_SHA256);
    assert_eq!(check["payload_count"], 5);
    assert_eq!(check["payload_hashes_match"], true);
    assert_eq!(check["upstream_manifest_and_check_match"], true);
    assert_eq!(check["upstream_payload_hashes_match"], true);
    assert_eq!(check["module_file_identity_runtime_bound"], true);
    assert_eq!(check["valid_input_recipes"], 10);
    assert_eq!(check["state_transitions"], 12);
    assert_eq!(check["prototype_postcompare"], "12/12");
    assert_eq!(check["wash_formula_replay"], "PASS 12/12");
    assert_eq!(check["whole_f09_closed"], false);
    assert_eq!(check["whole_implementation_ready"], "NO");
    assert_eq!(
        provenance["upstream_manifest_sha256"],
        UPSTREAM_WASH_MANIFEST_SHA256
    );
    assert_eq!(
        provenance["upstream_independent_check_sha256"],
        UPSTREAM_INDEPENDENT_CHECK_SHA256
    );

    for (name, expected_bytes, expected_sha256) in PACKAGE_PAYLOADS {
        let bytes = fixture_bytes(name);
        assert_eq!(bytes.len(), expected_bytes, "{name} size");
        assert_eq!(sha256(&bytes), expected_sha256, "{name} hash");
    }
}

#[test]
fn bounded_refresh_recipes_match_each_ordered_state_transition() {
    let inputs = fixture_json("INPUTS.json");
    let expected = fixture_json("EXPECTED.json");
    let input_recipes = inputs["recipes"].as_array().unwrap();
    let expected_recipes = expected["cases"].as_array().unwrap();
    assert_eq!(input_recipes.len(), 10);
    assert_eq!(expected_recipes.len(), input_recipes.len());

    let mut transition_count = 0;
    for (recipe, expected_recipe) in input_recipes.iter().zip(expected_recipes) {
        let name = recipe["name"].as_str().unwrap();
        assert_eq!(name, expected_recipe["name"].as_str().unwrap());

        let mut current = i16_values(&recipe["current"]);
        let mut work = i16_values(&recipe["work"]);
        assert_eq!(current.len(), work.len(), "{name} equal row lengths");
        assert!(
            current.iter().all(|value| (0..=31).contains(value)),
            "{name} current domain"
        );
        assert!(
            work.iter().all(|value| (0..=31).contains(value)),
            "{name} work domain"
        );

        let mut marker = i16::try_from(recipe["marker"].as_i64().unwrap()).unwrap();
        let mode = i16::try_from(recipe["mode"].as_i64().unwrap()).unwrap();
        let special = i16::try_from(recipe["special"].as_i64().unwrap()).unwrap();
        assert!((0..=1).contains(&mode), "{name} mode precondition");
        assert!((0..=1).contains(&special), "{name} special precondition");

        let begin =
            i16::try_from(recipe.get("begin").and_then(Value::as_i64).unwrap_or(0)).unwrap();
        let end = i16::try_from(current.len()).unwrap();
        assert!(begin >= 0 && begin < end, "{name} begin/end precondition");
        assert!(i32::from(end) < 32_768, "{name} end precondition");
        assert!(
            usize::try_from(begin).unwrap() < current.len(),
            "{name} begin range"
        );

        let repeat = recipe.get("repeat").and_then(Value::as_u64).unwrap_or(1);
        let expected_steps = expected_recipe["steps"].as_array().unwrap();
        assert_eq!(
            expected_steps.len(),
            usize::try_from(repeat).unwrap(),
            "{name} steps"
        );

        for (step_index, expected_step) in expected_steps.iter().enumerate() {
            refresh_exponent_state(
                &mut current,
                &mut work,
                &mut marker,
                mode,
                special,
                begin,
                end,
            );
            assert_eq!(
                current,
                i16_values(&expected_step["current"]),
                "{name} step {step_index} current"
            );
            assert_eq!(
                work,
                i16_values(&expected_step["work"]),
                "{name} step {step_index} work"
            );
            assert_eq!(
                marker,
                i16::try_from(expected_step["marker"].as_i64().unwrap()).unwrap(),
                "{name} step {step_index} marker"
            );
            transition_count += 1;
        }
    }

    assert_eq!(transition_count, 12);
}
