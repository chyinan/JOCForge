// pattern: Functional Core
use jocforge_domain::codec::coefficient_power::{
    CoefficientRow, POWER_ROW_BINS, PowerRow, coefficient_pair_to_power_row,
    produce_single_power_row, produce_two_by_two_power_rows,
};
use jocforge_domain::codec::raw_exponent_from_power::raw_exponent_from_power_bits;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/coefficient_power";
const PACKAGE_MANIFEST_SHA256: &str =
    "4af64c87e7a40c3990075a3aa07d052ea0178c99ef1f1264847dfeaa7265c2d8";
const INDEPENDENT_CHECK_SHA256: &str =
    "ddcf281cb1edc3d805bd65aec282403fe962412a49eaf0311c1eac19d06ecd66";
const PACKAGE_PAYLOADS: [(&str, usize, &str); 5] = [
    (
        "CONTRACT.md",
        3771,
        "09d881feb584fc061009933374f61d9a38271970846680f524cfe110b85ac976",
    ),
    (
        "INPUTS.json",
        2873,
        "76bfff2673c035a5a2c360fb0b2de146f7a0141d6cf7d44c5313c92696c03bdc",
    ),
    (
        "EXPECTED.json",
        103124,
        "0c0516b60fb3b1cab23856ca8c3d5fdf16d93c1f492b1c0c618008288796345c",
    ),
    (
        "PARAMETERS.json",
        731,
        "8593cc226ea381634e14f16a26e536274a16a744ad6d8dc1c7f0b32586c5ad1f",
    ),
    (
        "PROVENANCE.json",
        3164,
        "027eafa3606a322352fb92488bdf89d68dab01a5729bfa9037a5df0b5d7790ac",
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

fn cycle_f32(values: &Value) -> Vec<f32> {
    values
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_f64().unwrap() as f32)
        .collect()
}

fn cycle_u32(values: &Value) -> Vec<f32> {
    values
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .map(f32::from_bits)
        .collect()
}

fn fill_single_recipe(recipe: &Value) -> (CoefficientRow, CoefficientRow) {
    let kind = recipe["kind"].as_str().unwrap();
    let mut real = [0.0; POWER_ROW_BINS];
    let mut imaginary = [0.0; POWER_ROW_BINS];

    match kind {
        "zero" => {}
        "periodic" => {
            let real_cycle = [0.0_f32, 0.25, -0.25, 0.5, 1.0, 2.0];
            let imaginary_cycle = [0.0_f32, 0.0, 0.25, 0.5, 0.0, 0.0];
            for bin in 0..POWER_ROW_BINS {
                real[bin] = real_cycle[bin % real_cycle.len()];
                imaginary[bin] = imaginary_cycle[bin % imaginary_cycle.len()];
            }
        }
        "tail_impulse" => real[POWER_ROW_BINS - 1] = 1.0,
        "rounding_edge_cycle" => {
            let real_cycle = cycle_u32(&recipe["real_bits_cycle"]);
            let imaginary_cycle = cycle_u32(&recipe["imaginary_bits_cycle"]);
            for bin in 0..POWER_ROW_BINS {
                real[bin] = real_cycle[bin % real_cycle.len()];
                imaginary[bin] = imaginary_cycle[bin % imaginary_cycle.len()];
            }
        }
        other => panic!("unexpected single-row recipe {other}"),
    }

    (real, imaginary)
}

fn fill_multi_recipe(
    recipe: &Value,
    call: usize,
) -> ([[CoefficientRow; 2]; 2], [[CoefficientRow; 2]; 2]) {
    let real_cycle = cycle_f32(&recipe["real_cycle"]);
    let imaginary_cycle = cycle_f32(&recipe["imag_cycle"]);
    let mut real = [[[0.0; POWER_ROW_BINS]; 2]; 2];
    let mut imaginary = [[[0.0; POWER_ROW_BINS]; 2]; 2];

    for block in 0..2 {
        for channel in 0..2 {
            for bin in 0..POWER_ROW_BINS {
                let phase = (bin + block + 2 * channel + call) % real_cycle.len();
                real[block][channel][bin] = real_cycle[phase];
                imaginary[block][channel][bin] = imaginary_cycle[phase];
            }
        }
    }

    (real, imaginary)
}

fn assert_power_and_raw(
    power: &PowerRow,
    expected_power: &Value,
    expected_raw: &Value,
) -> (usize, usize) {
    let expected_power: Vec<u32> = expected_power
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .collect();
    assert_eq!(expected_power.len(), POWER_ROW_BINS);
    assert_eq!(power.as_slice(), expected_power.as_slice());

    let raw_values = expected_raw.as_array().unwrap();
    assert_eq!(raw_values.len(), 217);
    for (power_word, expected_value) in power.iter().zip(raw_values) {
        let expected_value = i16::try_from(expected_value.as_i64().unwrap()).unwrap();
        assert_eq!(
            raw_exponent_from_power_bits(*power_word, 0, 48),
            expected_value
        );
    }

    (expected_power.len(), raw_values.len())
}

#[test]
fn package_and_upstream_identities_are_bound_to_the_input_and_expected_recipes() {
    let check_bytes = fixture_bytes("INDEPENDENT-CHECK.json");
    let check: Value = serde_json::from_slice(&check_bytes).unwrap();
    let provenance = fixture_json("PROVENANCE.json");

    assert_eq!(sha256(&check_bytes), INDEPENDENT_CHECK_SHA256);
    assert_eq!(check["manifest_sha256"], PACKAGE_MANIFEST_SHA256);
    assert_eq!(check["payload_count"], 5);
    assert_eq!(check["payload_hashes_match"], true);
    assert_eq!(check["upstream_manifest_check_and_payloads_match"], true);
    assert_eq!(check["module_file_identity_runtime_bound"], true);
    assert_eq!(check["controlled_invocations"], 6);
    assert_eq!(check["independent_power_replay_u32"], 3036);
    assert_eq!(check["independent_raw_exponent_replay_i16"], 2387);
    assert_eq!(check["formula_replay"], "PASS all values");
    assert_eq!(check["independent_product_audio_integration"], "NOT_RUN");
    assert_eq!(check["whole_f09_closed"], false);
    assert_eq!(check["whole_implementation_ready"], "NO");
    assert_eq!(
        provenance["source_wash_manifest_sha256"],
        check["source_wash_manifest_sha256"]
    );

    for (name, expected_bytes, expected_sha256) in PACKAGE_PAYLOADS {
        let bytes = fixture_bytes(name);
        assert_eq!(bytes.len(), expected_bytes, "{name} size");
        assert_eq!(sha256(&bytes), expected_sha256, "{name} hash");
    }
}

#[test]
fn six_input_recipes_replay_all_power_rows_and_downstream_exponents() {
    let inputs = fixture_json("INPUTS.json");
    let expected = fixture_json("EXPECTED.json");
    let recipes = inputs["recipes"].as_array().unwrap();
    let invocations = expected["invocations"].as_array().unwrap();
    assert_eq!(inputs["expected_used_as_seed"], false);
    assert_eq!(recipes.len(), 6);
    assert_eq!(invocations.len(), recipes.len());
    assert_eq!(
        inputs["reconstruction"]["tail_impulse"],
        "All zero except real[252]=1"
    );

    let mut power_values = 0;
    let mut raw_values = 0;
    let mut single_power = [0xa5a5_5a5a; POWER_ROW_BINS];
    let mut multi_power = [[[0xa5a5_5a5a; POWER_ROW_BINS]; 2]; 2];

    for (recipe, invocation) in recipes.iter().zip(invocations) {
        match recipe["kind"].as_str().unwrap() {
            "zero" | "periodic" | "tail_impulse" => {
                let (real, imaginary) = fill_single_recipe(recipe);
                let mut bias = i16::try_from(recipe["initial_bias"].as_i64().unwrap()).unwrap();
                produce_single_power_row(&real, &imaginary, &mut bias, &mut single_power);
                assert_eq!(bias, 0);
                assert_eq!(invocation["producer_return"], "0");
                assert_eq!(invocation["bias_after"], 0);
                assert_eq!(invocation["power_tail_canary"], 305419896_u64);
                assert_eq!(invocation["raw_tail_canary"], 12345);
                let (power_count, raw_count) = assert_power_and_raw(
                    &single_power,
                    &invocation["power_bits"],
                    &invocation["raw_exponents"],
                );
                power_values += power_count;
                raw_values += raw_count;
            }
            "periodic_multi" => {
                let call = usize::try_from(recipe["call"].as_u64().unwrap()).unwrap();
                let (real, imaginary) = fill_multi_recipe(recipe, call);
                let mut bias = [[0; 2]; 2];
                let initial_bias = i16::try_from(recipe["initial_bias"].as_i64().unwrap()).unwrap();
                for channel_bias in &mut bias {
                    channel_bias.fill(initial_bias);
                }
                produce_two_by_two_power_rows(&real, &imaginary, &mut bias, &mut multi_power);
                assert!(bias.iter().flatten().all(|value| *value == 0));
                assert_eq!(invocation["producer_return"], "0");
                assert_eq!(invocation["raw_return"], "0");
                for expected_row in invocation["rows"].as_array().unwrap() {
                    let block = usize::try_from(expected_row["block"].as_u64().unwrap()).unwrap();
                    let channel =
                        usize::try_from(expected_row["channel"].as_u64().unwrap()).unwrap();
                    assert_eq!(expected_row["bias"], 0);
                    assert_eq!(expected_row["power_tail_canary"], 305419896_u64);
                    assert_eq!(expected_row["raw_tail_canary"], 12345);
                    let (power_count, raw_count) = assert_power_and_raw(
                        &multi_power[channel][block],
                        &expected_row["power_bits"],
                        &expected_row["raw_exponents"],
                    );
                    power_values += power_count;
                    raw_values += raw_count;
                }
            }
            "rounding_edge_cycle" => {
                let (real, imaginary) = fill_single_recipe(recipe);
                let power = coefficient_pair_to_power_row(&real, &imaginary);
                let expected_power: Vec<u32> = invocation["power_bits"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
                    .collect();
                assert_eq!(expected_power.len(), POWER_ROW_BINS);
                assert_eq!(power.as_slice(), expected_power.as_slice());
                assert_eq!(invocation["producer_return"], "0");
                assert_eq!(invocation["bias_after"], 0);
                assert_eq!(invocation["tail_canary"], 305419896_u64);
                power_values += expected_power.len();
            }
            kind => panic!("unexpected coefficient recipe {kind}"),
        }
    }

    assert_eq!(power_values, 3036);
    assert_eq!(raw_values, 2387);
}
