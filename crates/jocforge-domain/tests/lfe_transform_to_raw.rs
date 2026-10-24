// pattern: Functional Core
use jocforge_domain::codec::lfe_coefficient_exponent::LFE_COEFFICIENT_BINS;
use jocforge_domain::transform::lfe_transform::{LfeTransformBranchOutput, lfe_transform_to_raw};
use jocforge_domain::transform::long_transform::{
    LONG_TRANSFORM_HISTORY_WORDS, LONG_TRANSFORM_WORDS, LongTransformHistory,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/lfe_transform_to_raw";
const PACKAGE_MANIFEST_SHA256: &str =
    "96b7b6b5b4d76f4967daa09d3817fcac789a22fba362c505fe915d1aee29f5ca";
const INDEPENDENT_CHECK_SHA256: &str =
    "60bcbc859b2b8e0cd487261915c20b38ad3ab3a63dc89851d8a5570da8e07167";
const PACKAGE_PAYLOADS: [(&str, usize, &str); 5] = [
    (
        "CONTRACT.md",
        4827,
        "576632dc3ec286a2e34fd4c4f0d3ce740faeb08a9b35480aec741977d9edd906",
    ),
    (
        "INPUTS.json",
        934,
        "a0482cc260a07901e1f1d72ff3dada518398fe90e654c78faeb558cd59c9c1b8",
    ),
    (
        "EXPECTED.json",
        199824,
        "30120cea975fe6184359173a44893e87e2f8383effa18fad8ab22d0567520baf",
    ),
    (
        "PARAMETERS.json",
        9572,
        "73ceb9195554577f5fde109cd97ca0b9ef4a1618be9497e6222bdd05963467e8",
    ),
    (
        "PROVENANCE.json",
        3736,
        "1ac870d38f6dd1e38e8d85bd6415e6dc2e74926cf489494648b0a4972ec9cbd7",
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

fn input_for(kind: &str, call: usize) -> [u32; LONG_TRANSFORM_WORDS] {
    let mut input = [0.0_f32; LONG_TRANSFORM_WORDS];
    match kind {
        "zero" => {}
        "impulse" => input[call] = 0.5,
        "periodic" => {
            let cycle = [0.0_f32, 0.125, -0.25, 0.5, -0.5, 0.25];
            for (word, value) in input.iter_mut().enumerate() {
                *value = cycle[(word + call) % cycle.len()];
            }
        }
        other => panic!("unsupported LFE long-transform recipe {other}"),
    }
    input.map(f32::to_bits)
}

fn expected_u32(expected: &Value) -> Vec<u32> {
    expected
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .collect()
}

fn compare_branch(
    actual: &LfeTransformBranchOutput,
    expected: &Value,
    branch: &str,
) -> (usize, usize, usize, usize, usize) {
    assert_eq!(expected["branch"], branch);
    let folded = expected_u32(&expected["folded"]);
    let pre = expected_u32(&expected["pre"]);
    let low = expected_u32(&expected["low"]);
    let spectrum = expected_u32(&expected["spectrum_bits"]);
    let wrapper = expected_u32(&expected["wrapper_bits"]);
    let histories = expected_u32(&expected["history_bits"]);
    assert_eq!(
        actual.folded_bits().as_slice(),
        folded.as_slice(),
        "{branch} fold"
    );
    assert_eq!(actual.pre_bits().as_slice(), pre.as_slice(), "{branch} pre");
    assert_eq!(
        actual.low_bits().as_slice(),
        low.as_slice(),
        "{branch} low FFT/recurrence"
    );
    assert_eq!(
        actual.spectrum_bits().as_slice(),
        spectrum.as_slice(),
        "{branch} spectrum"
    );
    assert_eq!(
        actual.wrapper_bits().as_slice(),
        wrapper.as_slice(),
        "{branch} wrapper"
    );
    assert_eq!(
        actual.history_after().words().as_slice(),
        histories.as_slice(),
        "{branch} history"
    );
    assert_eq!(expected["tail_canary"], 305419896_u64);
    (
        folded.len(),
        pre.len(),
        low.len(),
        spectrum.len(),
        histories.len(),
    )
}

#[test]
fn package_payloads_and_dependency_identity_are_hash_bound() {
    let check_bytes = fixture_bytes("INDEPENDENT-CHECK.json");
    let check: Value = serde_json::from_slice(&check_bytes).unwrap();
    let provenance = fixture_json("PROVENANCE.json");

    assert_eq!(sha256(&check_bytes), INDEPENDENT_CHECK_SHA256);
    assert_eq!(check["manifest_sha256"], PACKAGE_MANIFEST_SHA256);
    assert_eq!(check["payload_count"], 5);
    assert_eq!(check["payload_hashes_match"], true);
    assert_eq!(check["upstream_manifest_check_and_payloads_match"], true);
    assert_eq!(check["module_file_identity_runtime_bound"], true);
    assert_eq!(check["recipes"], 3);
    assert_eq!(check["calls"], 6);
    assert_eq!(check["independent_fold_values"], 3072);
    assert_eq!(check["independent_pre_values"], 3072);
    assert_eq!(check["independent_low_fft_values"], 192);
    assert_eq!(check["independent_coefficients"], 84);
    assert_eq!(check["independent_history_values"], 1536);
    assert_eq!(check["independent_raw_lfe_values"], 42);
    assert_eq!(check["formula_replay"], "PASS all values");
    assert_eq!(check["independent_product_audio_integration"], "NOT_RUN");
    assert_eq!(check["whole_f09_closed"], false);
    assert_eq!(
        provenance["verified_dependency"]["package"],
        "LFE-COEFFICIENT-EXPONENT"
    );
    assert_eq!(
        provenance["verified_dependency"]["manifest_sha256"],
        "63d6fa945445ea9545b3ac02ad9df86b832a4abcccf6beb5caaf515ed1a3540a"
    );
    assert_eq!(
        provenance["verified_dependency"]["check_sha256"],
        "9321c12d5414febcce241e9102628434dc7ab6b1c0e1c4a1168b2037ba6fc258"
    );

    for (name, expected_bytes, expected_sha256) in PACKAGE_PAYLOADS {
        let bytes = fixture_bytes(name);
        assert_eq!(bytes.len(), expected_bytes, "{name} size");
        assert_eq!(sha256(&bytes), expected_sha256, "{name} SHA-256");
    }
}

#[test]
fn three_two_call_recipes_replay_lfe_transform_history_and_raw_output() {
    let inputs = fixture_json("INPUTS.json");
    let expected = fixture_json("EXPECTED.json");
    assert_eq!(inputs["expected_used_as_seed"], false);
    assert!(
        inputs["construction"]
            .as_str()
            .unwrap()
            .contains("Short flag0")
    );
    let recipes = inputs["recipes"].as_array().unwrap();
    let expected_cases = expected["cases"].as_array().unwrap();
    assert_eq!(recipes.len(), 3);
    assert_eq!(expected_cases.len(), recipes.len());

    let mut fold_values = 0;
    let mut pre_values = 0;
    let mut low_values = 0;
    let mut coefficient_values = 0;
    let mut history_values = 0;
    let mut raw_values = 0;

    for (recipe_index, (recipe, expected_case)) in recipes.iter().zip(expected_cases).enumerate() {
        let kind = recipe["kind"].as_str().unwrap();
        assert_eq!(recipe["calls"], 2);
        assert_eq!(recipe["input_count"], LONG_TRANSFORM_WORDS);
        assert_eq!(recipe["initial_histories_zero"], true);
        assert_eq!(recipe["short_flag"], 0);
        assert_eq!(recipe["lfe_count"], LFE_COEFFICIENT_BINS);
        assert_eq!(expected_case["recipe_index"], recipe_index);

        let mut primary_history = LongTransformHistory::zeroed();
        let mut companion_history = LongTransformHistory::zeroed();
        let mut raw_output = [12345_i16; LFE_COEFFICIENT_BINS + 1];

        for call in 0..2 {
            let input = input_for(kind, call);
            let output = lfe_transform_to_raw(
                &input,
                &primary_history,
                &companion_history,
                &mut raw_output,
            );
            let step = &expected_case["steps"][call];
            assert_eq!(step["call"], call);
            assert_eq!(step["raw_lfe_return"], "0");
            let branches = step["branches"].as_array().unwrap();
            assert_eq!(branches.len(), 2);

            let primary_counts = compare_branch(output.primary(), &branches[0], "primary");
            let companion_counts = compare_branch(output.companion(), &branches[1], "companion");
            fold_values += primary_counts.0 + companion_counts.0;
            pre_values += primary_counts.1 + companion_counts.1;
            low_values += primary_counts.2 + companion_counts.2;
            coefficient_values += primary_counts.3 + companion_counts.3;
            history_values += primary_counts.4 + companion_counts.4;

            let raw_expected: Vec<i16> = step["raw_lfe"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| i16::try_from(value.as_i64().unwrap()).unwrap())
                .collect();
            assert_eq!(raw_expected.len(), LFE_COEFFICIENT_BINS);
            assert_eq!(&raw_output[..LFE_COEFFICIENT_BINS], raw_expected.as_slice());
            assert_eq!(raw_output[LFE_COEFFICIENT_BINS], 12345);
            assert_eq!(step["raw_canary"], 12345);
            raw_values += raw_expected.len();

            primary_history = output.primary().history_after().clone();
            companion_history = output.companion().history_after().clone();
        }
    }

    assert_eq!(fold_values, 3072);
    assert_eq!(pre_values, 3072);
    assert_eq!(low_values, 192);
    assert_eq!(coefficient_values, 84);
    assert_eq!(history_values, 1536);
    assert_eq!(raw_values, 42);
    assert_eq!(LONG_TRANSFORM_HISTORY_WORDS, 128);
}
