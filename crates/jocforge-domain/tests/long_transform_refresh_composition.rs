// pattern: Functional Core
use jocforge_domain::codec::coefficient_power::{CoefficientRow, coefficient_pair_to_power_row};
use jocforge_domain::codec::exponent_refresh_state::refresh_exponent_state;
use jocforge_domain::codec::raw_exponent_from_power::raw_exponent_from_power_bits;
use jocforge_domain::transform::long_transform::{
    LONG_TRANSFORM_HISTORY_WORDS, LONG_TRANSFORM_WORDS, LongTransformHistory, TransformBranch,
    long_transform,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/long_transform_refresh";
const PACKAGE_MANIFEST_SHA256: &str =
    "4eaa2690e30ed0705404d7a6cd835e8637c2f933319d60be8812d900cb6521c1";
const INDEPENDENT_CHECK_SHA256: &str =
    "0574d3f4aff6460340709216fd1d0c141c47baa68367f2c43abbc446ba2157b5";
const PACKAGE_PAYLOADS: [(&str, usize, &str); 8] = [
    (
        "CONTRACT.md",
        5609,
        "9d44569e61f49eed238f6a8dbb65136fa2d9307c755fb1158a397ee29b92570f",
    ),
    (
        "INPUTS.json",
        3093,
        "75a4a7ac0b10a1ab7821db78f460fa9999d42ec9993d5d142012de12dfeea840",
    ),
    (
        "EXPECTED.json",
        695029,
        "54dbd50e02582f8e6fba46cb689965b2c82a87d2d86bc53d8036d4c604e6b181",
    ),
    (
        "PARAMETERS.json",
        13694,
        "d227af94962d12229927f7090d96ffb265bdccc963a967b6e8e3bb1e3ecb7608",
    ),
    (
        "PROVENANCE.json",
        5553,
        "7fa05f0c482aa3321edd927a73376b343eb593d87652976a4c0a0d1f20da101c",
    ),
    (
        "FFT-CANARY-INPUT.json",
        391,
        "f8043b255bc1bba291584fba156b51a74ca3911ce84e095662aa11c44f2f92ae",
    ),
    (
        "FFT-CANARY-EXPECTED.json",
        675,
        "ad457c9cc79bdd11522cdc2ca69529fba8442abf690a811789890575d9b906b2",
    ),
    (
        "FFT-CANARY-CHECK.json",
        319,
        "5c4d27ee11f41bfa71ff713f0b56605c2937cbd85ac0c4d8f562346ed4655242",
    ),
];
const DEPENDENCY_IDENTITIES: [(&str, &str, &str); 3] = [
    (
        "COEFFICIENT-TO-POWER-ROW",
        "4af64c87e7a40c3990075a3aa07d052ea0178c99ef1f1264847dfeaa7265c2d8",
        "ddcf281cb1edc3d805bd65aec282403fe962412a49eaf0311c1eac19d06ecd66",
    ),
    (
        "RAW-EXPONENT-FROM-POWER-BITS",
        "f1acb6d4321e3b7fdb746bf7fd0e3b1ce7db179a5ecb5fe37201b429587158fb",
        "c184b804d2f7a607c24ee4ca86f9db2664cc9227c9e411a66ad21573d0537ed3",
    ),
    (
        "EXPONENT-REFRESH-STATE-MAP",
        "f46a4c67bf019649d8f770a04d59b3809ed9eda7b3d5268aab4e3bddd03505e5",
        "0f42620d973b858f3a14d930e4c6c830be7af0e6a6528561edbe9c81cc4035a5",
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

fn assert_u32_words(actual: &[u32], expected: &Value, label: &str) -> usize {
    let expected_words: Vec<u32> = expected
        .as_array()
        .unwrap()
        .iter()
        .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
        .collect();
    assert_eq!(actual, expected_words.as_slice(), "{label}");
    actual.len()
}

fn assert_i16_values(actual: &[i16], expected: &Value, label: &str) -> usize {
    let expected_values: Vec<i16> = expected
        .as_array()
        .unwrap()
        .iter()
        .map(|value| i16::try_from(value.as_i64().unwrap()).unwrap())
        .collect();
    assert_eq!(actual, expected_values.as_slice(), "{label}");
    actual.len()
}

fn input_for(kind: &str, call: usize) -> [f32; LONG_TRANSFORM_WORDS] {
    let mut input = [0.0; LONG_TRANSFORM_WORDS];
    match kind {
        "zero" => {}
        "impulse" => input[call] = 0.5,
        "periodic" => {
            let cycle = [0.0_f32, 0.125, -0.25, 0.5, -0.5, 0.25];
            for (index, value) in input.iter_mut().enumerate() {
                *value = cycle[(index + call) % cycle.len()];
            }
        }
        other => panic!("unexpected long-transform input recipe {other}"),
    }
    input
}

fn input_bits(input: &[f32; LONG_TRANSFORM_WORDS]) -> [u32; LONG_TRANSFORM_WORDS] {
    input.map(f32::to_bits)
}

#[test]
fn package_payloads_and_three_direct_dependency_identities_are_hash_bound() {
    let check_bytes = fixture_bytes("INDEPENDENT-CHECK.json");
    let check: Value = serde_json::from_slice(&check_bytes).unwrap();
    let provenance = fixture_json("PROVENANCE.json");

    assert_eq!(sha256(&check_bytes), INDEPENDENT_CHECK_SHA256);
    assert_eq!(check["manifest_sha256"], PACKAGE_MANIFEST_SHA256);
    assert_eq!(check["payload_count"], 8);
    assert_eq!(check["payload_hashes_match"], true);
    assert_eq!(check["source_manifest_check_and_payloads_match"], true);
    assert_eq!(check["module_file_identity_runtime_bound"], true);
    assert!(
        check["standalone_stage_replay"]
            .as_str()
            .unwrap()
            .contains("PASS")
    );
    assert_eq!(check["fft_group_index_canary"], "PASS 32/32");
    assert!(
        check["composition_replay"]
            .as_str()
            .unwrap()
            .contains("PASS")
    );
    assert_eq!(check["native_inner_fft_stage_buffers"], "NOT_RUN");
    assert_eq!(check["independent_product_audio_integration"], "NOT_RUN");
    assert_eq!(check["whole_f09_closed"], false);
    assert_eq!(check["whole_implementation_ready"], "NO");

    for (name, expected_bytes, expected_sha256) in PACKAGE_PAYLOADS {
        let bytes = fixture_bytes(name);
        assert_eq!(bytes.len(), expected_bytes, "{name} bytes");
        assert_eq!(sha256(&bytes), expected_sha256, "{name} SHA-256");
    }

    let dependencies = provenance["verified_dependencies"].as_array().unwrap();
    assert_eq!(dependencies.len(), DEPENDENCY_IDENTITIES.len());
    for ((name, manifest_sha256, check_sha256), dependency) in
        DEPENDENCY_IDENTITIES.iter().zip(dependencies)
    {
        assert_eq!(dependency["package"], *name);
        assert_eq!(dependency["manifest_sha256"], *manifest_sha256);
        assert_eq!(dependency["check_sha256"], *check_sha256);
    }
}

#[test]
fn six_standalone_recipes_replay_each_transform_stage_and_history_transition() {
    let inputs = fixture_json("INPUTS.json");
    let expected = fixture_json("EXPECTED.json");
    assert_eq!(inputs["expected_used_as_seed"], false);
    let recipes = inputs["standalone"].as_array().unwrap();
    let expected_recipes = expected["standalone"].as_array().unwrap();
    assert_eq!(recipes.len(), 6);
    assert_eq!(expected_recipes.len(), recipes.len());

    let mut fold_values = 0;
    let mut pre_values = 0;
    let mut fft_values = 0;
    let mut post_values = 0;
    let mut wrapper_values = 0;
    let mut history_values = 0;
    let mut calls = 0;

    for (recipe_index, (recipe, expected_recipe)) in
        recipes.iter().zip(expected_recipes).enumerate()
    {
        assert_eq!(recipe["calls"], 2);
        assert_eq!(recipe["input_count"], LONG_TRANSFORM_WORDS);
        assert_eq!(recipe["history_count"], LONG_TRANSFORM_HISTORY_WORDS);
        assert_eq!(recipe["initial_history_zero"], true);
        let branch = match recipe["branch"].as_str().unwrap() {
            "primary" => TransformBranch::Primary,
            "companion" => TransformBranch::Companion,
            other => panic!("unexpected transform branch {other}"),
        };
        let kind = recipe["kind"].as_str().unwrap();
        let mut history = LongTransformHistory::zeroed();

        for call in 0..2 {
            let input = input_for(kind, call);
            let output = long_transform(&input_bits(&input), &history, branch);
            let step = &expected_recipe["steps"][call];
            let stages = &step["stages"];
            assert_eq!(step["call"], call);
            assert_eq!(step["fold_return"], "0");
            assert_eq!(step["transform_return"], "0");
            fold_values += assert_u32_words(output.folded_bits(), &stages["folded"], "fold");
            pre_values += assert_u32_words(
                output.pre_rotated_bits(),
                &stages["pre_rotated"],
                "pre-rotation",
            );
            fft_values += assert_u32_words(output.fft_bits(), &stages["fft"], "FFT");
            post_values += assert_u32_words(
                output.post_rotated_bits(),
                &stages["post_rotated"],
                "post-rotation",
            );
            wrapper_values += assert_u32_words(
                output.wrapper_spectrum_bits(),
                &step["spectrum_bits"],
                "wrapper spectrum",
            );
            history_values += assert_u32_words(
                output.history_after().words(),
                &step["history_bits"],
                "history after",
            );
            assert_eq!(step["tail_canary"], 305419896_u64);
            history = output.history_after().clone();
            calls += 1;
        }
        assert_eq!(expected_recipe["recipe_index"], recipe_index);
    }

    assert_eq!(calls, 12);
    assert_eq!(fold_values, 3072);
    assert_eq!(pre_values, 3072);
    assert_eq!(fft_values, 3072);
    assert_eq!(post_values, 3072);
    assert_eq!(wrapper_values, 3072);
    assert_eq!(history_values, 1536);
}

#[test]
fn three_two_call_recipes_replay_transform_power_raw_and_refresh_composition() {
    let inputs = fixture_json("INPUTS.json");
    let expected = fixture_json("EXPECTED.json");
    let recipes = inputs["composition"].as_array().unwrap();
    let expected_recipes = expected["composition"].as_array().unwrap();
    assert_eq!(recipes.len(), 3);
    assert_eq!(expected_recipes.len(), recipes.len());

    let mut spectrum_values = 0;
    let mut history_values = 0;
    let mut power_values = 0;
    let mut raw_values = 0;
    let mut refresh_values = 0;
    let mut marker_values = 0;

    for (recipe_index, (recipe, expected_recipe)) in
        recipes.iter().zip(expected_recipes).enumerate()
    {
        assert_eq!(recipe["calls"], 2);
        assert_eq!(recipe["block_words"], LONG_TRANSFORM_WORDS);
        assert_eq!(recipe["initial_histories_zero"], true);
        assert_eq!(recipe["initial_work_zero"], true);
        let kind = recipe["kind"].as_str().unwrap();
        let marker_schedule = recipe["marker_schedule"].as_array().unwrap();
        assert_eq!(marker_schedule.len(), 2);
        let mut primary_history = LongTransformHistory::zeroed();
        let mut companion_history = LongTransformHistory::zeroed();
        let mut work = [0_i16; 217];

        for call in 0..2 {
            let input = input_for(kind, call);
            let input_bits = input_bits(&input);
            let primary = long_transform(&input_bits, &primary_history, TransformBranch::Primary);
            let companion =
                long_transform(&input_bits, &companion_history, TransformBranch::Companion);
            let step = &expected_recipe["steps"][call];
            assert_eq!(step["call"], call);
            let real_words = primary.wrapper_spectrum_bits();
            let imaginary_words = companion.wrapper_spectrum_bits();
            spectrum_values += assert_u32_words(real_words, &step["real_bits"], "primary spectrum");
            spectrum_values +=
                assert_u32_words(imaginary_words, &step["imag_bits"], "companion spectrum");

            let expected_histories = step["history_bits"].as_array().unwrap();
            assert_eq!(expected_histories.len(), 2);
            history_values += assert_u32_words(
                primary.history_after().words(),
                &expected_histories[0],
                "primary history",
            );
            history_values += assert_u32_words(
                companion.history_after().words(),
                &expected_histories[1],
                "companion history",
            );

            let real: CoefficientRow = std::array::from_fn(|bin| f32::from_bits(real_words[bin]));
            let imaginary: CoefficientRow =
                std::array::from_fn(|bin| f32::from_bits(imaginary_words[bin]));
            let power = coefficient_pair_to_power_row(&real, &imaginary);
            power_values += assert_u32_words(&power, &step["power_bits"], "POWER row");
            assert_eq!(step["bias_after"], 0);
            assert_eq!(step["power_canary"], 305419896_u64);

            let raw: [i16; 217] =
                std::array::from_fn(|index| raw_exponent_from_power_bits(power[index], 0, 48));
            raw_values += assert_i16_values(&raw, &step["raw_before_refresh"], "raw exponents");
            assert_eq!(step["raw_canary"], 12345);
            assert_i16_values(&work, &step["work_before"], "work before refresh");

            let mut current = raw;
            let mut marker = i16::try_from(marker_schedule[call].as_i64().unwrap()).unwrap();
            refresh_exponent_state(&mut current, &mut work, &mut marker, 1, 0, 0, 128);
            refresh_values +=
                assert_i16_values(&current, &step["current_after"], "current after refresh");
            refresh_values += assert_i16_values(&work, &step["work_after"], "work after refresh");
            assert_eq!(step["marker_after"], i64::from(marker));
            marker_values += 1;

            primary_history = primary.history_after().clone();
            companion_history = companion.history_after().clone();
        }
        assert_eq!(expected_recipe["recipe_index"], recipe_index);
    }

    assert_eq!(spectrum_values, 3072);
    assert_eq!(history_values, 1536);
    assert_eq!(power_values, 1518);
    assert_eq!(raw_values, 1302);
    assert_eq!(refresh_values, 2604);
    assert_eq!(marker_values, 6);
}
