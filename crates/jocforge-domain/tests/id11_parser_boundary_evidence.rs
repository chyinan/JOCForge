// pattern: Imperative Shell
use std::collections::BTreeMap;

use serde_json::Value;
use sha2::{Digest, Sha256};

const INPUTS: &str = include_str!("fixtures/id11_parser_boundary/ID11-PARSER-INPUTS.json");
const OBSERVATIONS: &str =
    include_str!("fixtures/id11_parser_boundary/ID11-PARSER-OBSERVATIONS.json");
const SCOPE: &str = include_str!("fixtures/id11_parser_boundary/F13C-BOUNDARY-SCOPE.json");
const PROVENANCE: &str = include_str!("fixtures/id11_parser_boundary/PROVENANCE.json");
const WASH_VALIDATION: &str = include_str!("fixtures/id11_parser_boundary/WASH-VALIDATION.json");

#[test]
fn washed_id11_boundary_payloads_keep_the_sealed_identities() {
    assert_eq!(
        sha256(INPUTS.as_bytes()),
        "b2075d6f15fe5966908516e8396a8749e708ca6b5ecff3c0f65d9096b745c31c"
    );
    assert_eq!(
        sha256(OBSERVATIONS.as_bytes()),
        "61ec995678833171c064e79e068a46ece69dea442bd54336ac3312e3ade06acf"
    );
    assert_eq!(
        sha256(SCOPE.as_bytes()),
        "c6269768961ba8b2462f2b74558492bbfe4249e2f786667d5356587f8eca7bb1"
    );
    assert_eq!(
        sha256(PROVENANCE.as_bytes()),
        "bb5fec412e2af0e2a5fb13dea61225ebfbe5c60152c147012c0c20ef4bfb4582"
    );
    assert_eq!(
        sha256(WASH_VALIDATION.as_bytes()),
        "be7e0274caed8f3428a64f207bf948dcad1939ccaa261ebab55734c021f7cfbb"
    );

    let provenance: Value = serde_json::from_str(PROVENANCE).unwrap();
    let wash: Value = serde_json::from_str(WASH_VALIDATION).unwrap();
    assert_eq!(
        provenance["source_sealed_manifest_sha256"],
        "46b6a7418f759a134fd722a50979440b9fcc75e41552b557388ffddd85ffcfbe"
    );
    assert_eq!(provenance["pcm_payload_or_hash_included"], false);
    assert_eq!(provenance["private_source_included"], false);
    assert_eq!(provenance["private_key_included"], false);
    assert_eq!(wash["limits"]["whole_F13C"], "NOT_READY");
    assert_eq!(
        wash["limits"]["ID14_semantic_parser"],
        "NOT_OBSERVED_IN_BOUNDED_SCOPE"
    );
}

#[test]
fn selected_id11_prefix_and_trailing_byte_observations_are_self_consistent() {
    let input_document: Value = serde_json::from_str(INPUTS).unwrap();
    let observations_document: Value = serde_json::from_str(OBSERVATIONS).unwrap();
    let scope: Value = serde_json::from_str(SCOPE).unwrap();
    let inputs = input_document["cases"].as_array().unwrap();
    let observations = observations_document["cases"].as_array().unwrap();
    assert_eq!(
        input_document["scope"],
        "CONTROLLED_PREFIX_AND_TRAILING_INPUTS"
    );
    assert_eq!(
        observations_document["scope"],
        "PRIVATE_COLD_OWNER_NATIVE_NOT_MODEL_REPLAY"
    );
    assert_eq!(inputs.len(), 72);
    assert_eq!(observations.len(), 72);

    let observations_by_name: BTreeMap<&str, &Value> = observations
        .iter()
        .map(|observation| (observation["name"].as_str().unwrap(), observation))
        .collect();
    assert_eq!(observations_by_name.len(), 72);
    let mut failure_owner_equal_cold = 0_u8;
    let mut failure_owner_changed = 0_u8;

    for input in inputs {
        let name = input["name"].as_str().unwrap();
        let bytes = decode_hex(input["input_hex"].as_str().unwrap());
        let supplied_bytes = input["supplied_bytes"].as_u64().unwrap();
        assert_eq!(bytes.len() as u64, supplied_bytes, "{name}");
        assert_eq!(
            sha256(&bytes),
            input["input_sha256"].as_str().unwrap(),
            "{name}"
        );
        assert_eq!(input["constructor_objects"], 16, "{name}");
        assert_eq!(input["constructor_frames"], 8, "{name}");

        let observation = observations_by_name[name];
        assert_eq!(observation["supplied_bytes"], supplied_bytes, "{name}");
        assert_eq!(observation["constructor_objects"], 16, "{name}");
        assert_eq!(observation["constructor_frames"], 8, "{name}");
        assert_eq!(observation["owner_extent_bytes"], 8192, "{name}");
        assert_eq!(observation["guard_unchanged"], true, "{name}");
        assert_eq!(observation["input_unchanged"], true, "{name}");
        assert_eq!(observation["cursor_before"]["consumed_bits"], 0, "{name}");
        assert_eq!(
            observation["cursor_before"]["remaining_bits"],
            supplied_bytes * 8,
            "{name}"
        );

        let return_status = observation["return_status"].as_u64().unwrap();
        if name.starts_with("prefix_") {
            let prefix_bytes = name
                .strip_prefix("prefix_")
                .unwrap()
                .parse::<u64>()
                .unwrap();
            assert_eq!(prefix_bytes, supplied_bytes, "{name}");
            if prefix_bytes < 69 {
                assert_eq!(return_status, 1, "{name}");
                assert_eq!(observation["nested_status"], 1, "{name}");
                assert_eq!(observation["reset_calls"], 2, "{name}");
                if observation["owner_equals_cold"] == true {
                    failure_owner_equal_cold += 1;
                } else {
                    failure_owner_changed += 1;
                }
            } else {
                assert_eq!(name, "prefix_69");
                assert_eq!(return_status, 0);
                assert_eq!(observation["nested_status"], 0);
                assert_eq!(observation["alignment_before"]["consumed_bits"], 546);
                assert_eq!(observation["alignment_before"]["remaining_bits"], 6);
                assert_eq!(observation["alignment_after"]["consumed_bits"], 552);
                assert_eq!(observation["alignment_after"]["remaining_bits"], 0);
                assert_eq!(observation["cursor_after"]["consumed_bits"], 552);
                assert_eq!(observation["cursor_after"]["remaining_bits"], 0);
                assert_eq!(observation["reset_calls"], 1);
            }
        } else {
            assert!(matches!(name, "trailing_zero" | "trailing_ff"));
            assert_eq!(supplied_bytes, 70);
            assert_eq!(bytes.len(), 70);
            assert_eq!(
                &bytes[..69],
                &decode_hex(
                    inputs
                        .iter()
                        .find(|case| case["name"] == "prefix_69")
                        .unwrap()["input_hex"]
                        .as_str()
                        .unwrap()
                )
            );
            assert_eq!(bytes[69], if name == "trailing_zero" { 0 } else { 255 });
            assert_eq!(return_status, 0, "{name}");
            assert_eq!(observation["nested_status"], 0, "{name}");
            assert_eq!(observation["cursor_after"]["consumed_bits"], 552, "{name}");
            assert_eq!(observation["cursor_after"]["remaining_bits"], 8, "{name}");
            assert_eq!(observation["reset_calls"], 1, "{name}");
        }
    }

    assert_eq!(failure_owner_equal_cold, 2);
    assert_eq!(failure_owner_changed, 67);
    assert_eq!(scope["implementation_ready"], false);
    assert_eq!(scope["id11"]["semantic_parse_bits_selected_seed"], 546);
    assert_eq!(scope["id11"]["final_alignment_bits"], 6);
    assert_eq!(scope["id11"]["final_consumed_bits"], 552);
    assert_eq!(scope["id11"]["trailing_zero_and_ff_accepted"], true);
    assert_eq!(scope["id11"]["trailing_remaining_bits"], 8);
    assert_eq!(
        scope["id11"]["no_commit"],
        "NOT_PROVEN; two local resets do not restore every byte in the owned allocation"
    );
    assert_eq!(scope["collector"]["ID14_semantic_parser"], "NOT_RUN");
}

fn decode_hex(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(text, 16).unwrap()
        })
        .collect()
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut output, byte| {
            use std::fmt::Write as _;
            write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
            output
        })
}
