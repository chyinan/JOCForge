// pattern: Functional Core
use jocforge_domain::codec::lfe_coefficient_exponent::{
    LFE_COEFFICIENT_BINS, LfeCoefficientBlock, lfe_coefficients_to_exponents,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/lfe_coefficient_exponent";
const PACKAGE_MANIFEST_SHA256: &str =
    "63d6fa945445ea9545b3ac02ad9df86b832a4abcccf6beb5caaf515ed1a3540a";
const INDEPENDENT_CHECK_SHA256: &str =
    "9321c12d5414febcce241e9102628434dc7ab6b1c0e1c4a1168b2037ba6fc258";
const PACKAGE_PAYLOADS: [(&str, usize, &str); 5] = [
    (
        "CONTRACT.md",
        3205,
        "33e9a41f11a6bf6f8aae6c3a6568cd5958d60f2deec022a20529325844c65df1",
    ),
    (
        "INPUTS.json",
        3025,
        "fbd9317bcb5036b6d706f9f136df46b69520047dde52a126b65401fdf2b48fff",
    ),
    (
        "EXPECTED.json",
        17992,
        "80952b734a0c87fe523395a16a3c3e6172766b857ff944ebb0c8d4d78e085c2d",
    ),
    (
        "PARAMETERS.json",
        604,
        "eae2870ebcf5e5164f85a3b5e8cc16db5da39de550fed48bc9175be06b69472a",
    ),
    (
        "PROVENANCE.json",
        3136,
        "779c36e3d088fb736bf7cc17e17c7a6fa4b7ce41d65176bcef4ae7dcb9a1090e",
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

fn f32_cycle(values: &Value) -> Vec<f32> {
    values
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_f64().unwrap() as f32)
        .collect()
}

#[test]
fn package_and_upstream_identities_are_hash_bound() {
    let check_bytes = fixture_bytes("INDEPENDENT-CHECK.json");
    let check: Value = serde_json::from_slice(&check_bytes).unwrap();
    let provenance = fixture_json("PROVENANCE.json");

    assert_eq!(sha256(&check_bytes), INDEPENDENT_CHECK_SHA256);
    assert_eq!(check["manifest_sha256"], PACKAGE_MANIFEST_SHA256);
    assert_eq!(check["payload_count"], 5);
    assert_eq!(check["payload_hashes_match"], true);
    assert_eq!(check["upstream_manifest_check_and_payloads_match"], true);
    assert_eq!(check["module_file_identity_runtime_bound"], true);
    assert_eq!(check["recipes"], 5);
    assert_eq!(check["continuous_calls"], 10);
    assert_eq!(check["lfe_rows"], 20);
    assert_eq!(check["independent_power_probe_values"], 140);
    assert_eq!(check["independent_lfe_raw_values"], 140);
    assert_eq!(check["independent_combined_block0_values"], 70);
    assert_eq!(check["formula_replay"], "PASS all values");
    assert_eq!(
        check["bias_independence_values"],
        serde_json::json!([0, 2, 4])
    );
    assert_eq!(check["separate_power_probe_not_internal_capture"], true);
    assert_eq!(check["independent_product_audio_integration"], "NOT_RUN");
    assert_eq!(check["whole_f09_closed"], false);
    assert_eq!(
        provenance["source_manifest_sha256"],
        "f2e8ab625d17809a5e3b119fb923e8123818f3fc8639b2d8ba22f8da33efe72f"
    );
    assert_eq!(
        provenance["source_independent_check_sha256"],
        "b256e4cd23b48dc9391e4b955516d9085468b35603b5b1180c169f6726d48d37"
    );

    for (name, expected_bytes, expected_sha256) in PACKAGE_PAYLOADS {
        let bytes = fixture_bytes(name);
        assert_eq!(bytes.len(), expected_bytes, "{name} size");
        assert_eq!(sha256(&bytes), expected_sha256, "{name} hash");
    }
}

#[test]
fn lfe_coefficient_pair_route_replays_two_blocks_and_preserves_canaries() {
    let inputs = fixture_json("INPUTS.json");
    let expected = fixture_json("EXPECTED.json");
    let recipes = inputs["recipes"].as_array().unwrap();
    let cases = expected["cases"].as_array().unwrap();
    assert_eq!(recipes.len(), 5);
    assert_eq!(cases.len(), recipes.len());

    let mut outputs = [[12345; LFE_COEFFICIENT_BINS + 1]; 2];
    let mut baseline: Option<[[Vec<i16>; 2]; 2]> = None;
    let mut lfe_output_count = 0;
    let mut combined_output_count = 0;
    let mut separate_power_probe_count = 0;

    for (recipe, case) in recipes.iter().zip(cases) {
        assert_eq!(recipe["lfe_bins"], 7);
        assert_eq!(recipe["lfe_blocks"], 2);
        assert_eq!(recipe["phase_rule"], "(bin+block+call)%7");
        assert_eq!(recipe["reuse_buffers"], true);

        let real_cycle = f32_cycle(&recipe["real_cycle"]);
        let imaginary_cycle = f32_cycle(&recipe["imag_cycle"]);
        assert_eq!(real_cycle.len(), LFE_COEFFICIENT_BINS);
        assert_eq!(imaginary_cycle.len(), LFE_COEFFICIENT_BINS);
        let mut recipe_outputs = [[Vec::new(), Vec::new()], [Vec::new(), Vec::new()]];

        for call in 0..2 {
            let mut real: [LfeCoefficientBlock; 2] = [[0.0; LFE_COEFFICIENT_BINS]; 2];
            let mut imaginary: [LfeCoefficientBlock; 2] = [[0.0; LFE_COEFFICIENT_BINS]; 2];
            for block in 0..2 {
                for bin in 0..LFE_COEFFICIENT_BINS {
                    let phase = (bin + block + call) % LFE_COEFFICIENT_BINS;
                    real[block][bin] = real_cycle[phase];
                    imaginary[block][bin] = imaginary_cycle[phase];
                }
            }

            lfe_coefficients_to_exponents(&real, &imaginary, &mut outputs);
            let step = &case["steps"][call];
            assert_eq!(step["lfe_only_return"], "0");
            assert_eq!(step["fullband_return"], "0");
            assert_eq!(step["combined_lfe_block0"].as_array().unwrap().len(), 7);

            for expected_row in step["lfe_rows"].as_array().unwrap() {
                let block = usize::try_from(expected_row["block"].as_u64().unwrap()).unwrap();
                let expected_values: Vec<i16> = expected_row["output"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| i16::try_from(value.as_i64().unwrap()).unwrap())
                    .collect();
                assert_eq!(expected_values.len(), LFE_COEFFICIENT_BINS);
                assert_eq!(&outputs[block][..LFE_COEFFICIENT_BINS], expected_values);
                assert_eq!(outputs[block][LFE_COEFFICIENT_BINS], 12345);
                recipe_outputs[call][block] = expected_values.clone();
                lfe_output_count += expected_values.len();

                assert_eq!(expected_row["tail_canary"], 12345);
                assert_eq!(expected_row["power_probe_bias"], 0);
                assert_eq!(
                    expected_row["power_probe_role"],
                    "SEPARATE_NATIVE_HELPER_NOT_INTERNAL_INTERMEDIATE_CAPTURE"
                );
                assert_eq!(
                    expected_row["power_probe_bits"].as_array().unwrap().len(),
                    7
                );
                separate_power_probe_count += 7;
            }

            let combined = step["combined_lfe_block0"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| i16::try_from(value.as_i64().unwrap()).unwrap())
                .collect::<Vec<_>>();
            assert_eq!(combined, recipe_outputs[call][0]);
            combined_output_count += combined.len();
        }

        if let Some(expected_baseline) = &baseline {
            assert_eq!(&recipe_outputs, expected_baseline);
        } else {
            baseline = Some(recipe_outputs);
        }
    }

    assert_eq!(lfe_output_count, 140);
    assert_eq!(combined_output_count, 70);
    assert_eq!(separate_power_probe_count, 140);
}
