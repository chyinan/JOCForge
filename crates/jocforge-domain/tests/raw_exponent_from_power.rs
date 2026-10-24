// pattern: Functional Core
use jocforge_domain::codec::raw_exponent_from_power::raw_exponent_from_power_bits;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/raw_exponent";
const PACKAGE_MANIFEST_SHA256: &str =
    "f1acb6d4321e3b7fdb746bf7fd0e3b1ce7db179a5ecb5fe37201b429587158fb";
const INDEPENDENT_CHECK_SHA256: &str =
    "c184b804d2f7a607c24ee4ca86f9db2664cc9227c9e411a66ad21573d0537ed3";
const SOURCE_MANIFEST_SHA256: &str =
    "92624e1fe4319ffde8c11cfc10fb43d9c3cccd45687f913c97cd1f5d8cfacfac";
const SOURCE_INDEPENDENT_CHECK_SHA256: &str =
    "54d25872273474f1b26d769538182f7c14a24ad72791876bdcc3905259bd8d0d";
const PACKAGE_PAYLOADS: [(&str, usize, &str); 5] = [
    (
        "CONTRACT.md",
        2663,
        "32bf9d55b5e63aeb78c9f35fe1e4ffe08909f2f501cf5058d1bfa23f1176d784",
    ),
    (
        "INPUTS.json",
        1630,
        "12acc79d95ee6736a732c838ba4dea9a20ec65d3101719487e06a7ccdf9c493b",
    ),
    (
        "EXPECTED.json",
        1068,
        "e999e9479ee88e1ae0b7011cfaca1c50fc792522eeb9c3a04b5ffd4efb3ed0f6",
    ),
    (
        "PARAMETERS.json",
        739,
        "28ba7ad6cf9d3ea5d856b9b1ec81e78f431466d87f9e7ac0bb5a2a398ef8ee35",
    ),
    (
        "PROVENANCE.json",
        2761,
        "8a7cf7d026a242c30fa71428b2f2d7645c90a5ca53e7772b31d7c0876071d0e6",
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

#[test]
fn package_and_upstream_identities_are_bound_without_a_manifest_fixture() {
    let check_bytes = fixture_bytes("INDEPENDENT-CHECK.json");
    let check: Value = serde_json::from_slice(&check_bytes).unwrap();
    let provenance = fixture_json("PROVENANCE.json");

    assert_eq!(sha256(&check_bytes), INDEPENDENT_CHECK_SHA256);
    assert_eq!(check["manifest_sha256"], PACKAGE_MANIFEST_SHA256);
    assert_eq!(check["payload_count"], 5);
    assert_eq!(check["payload_hashes_match"], true);
    assert_eq!(check["upstream_manifest_check_and_payloads_match"], true);
    assert_eq!(check["module_file_identity_runtime_bound"], true);
    assert_eq!(check["valid_recipes"], 5);
    assert_eq!(check["values_compared"], 50);
    assert_eq!(check["formula_replay"], "PASS 50/50");
    assert_eq!(check["expected_used_as_seed"], false);
    assert_eq!(
        check["count_pointer_error_behavior"],
        "OUT_OF_SCOPE_AND_NOT_RUN"
    );
    assert_eq!(check["independent_product_implementation"], "NOT_RUN");
    assert_eq!(check["whole_f09_closed"], false);
    assert_eq!(check["whole_implementation_ready"], "NO");
    assert_eq!(provenance["source_manifest_sha256"], SOURCE_MANIFEST_SHA256);
    assert_eq!(
        provenance["source_independent_check_sha256"],
        SOURCE_INDEPENDENT_CHECK_SHA256
    );

    for (name, expected_bytes, expected_sha256) in PACKAGE_PAYLOADS {
        let bytes = fixture_bytes(name);
        assert_eq!(bytes.len(), expected_bytes, "{name} size");
        assert_eq!(sha256(&bytes), expected_sha256, "{name} hash");
    }
}

#[test]
fn all_50_power_bit_kernel_outputs_match_the_selected_integer_formula_vectors() {
    let inputs = fixture_json("INPUTS.json");
    let expected = fixture_json("EXPECTED.json");
    let input_cases = inputs["cases"].as_array().unwrap();
    let expected_cases = expected["cases"].as_array().unwrap();
    assert_eq!(input_cases.len(), 5);
    assert_eq!(expected_cases.len(), input_cases.len());
    assert_eq!(inputs["expected_used_as_seed"], false);

    let parameters = fixture_json("PARAMETERS.json");
    assert_eq!(parameters["word_transform"], "(2 * input_word) modulo 2^32");
    assert_eq!(
        parameters["bias"],
        "signed I16 scalar; addition to difference wraps modulo 65536 and is interpreted as signed I16"
    );
    assert_eq!(
        parameters["cap"],
        "signed I16 scalar; if wrapped biased value > cap, replace with cap using signed comparison"
    );
    assert_eq!(parameters["wrapper_cap_for_tested_route"], 48);
    assert_eq!(
        parameters["output"],
        "arithmetic right shift of signed I16 capped value by one"
    );

    let mut output_count = 0;
    for (input_case, expected_case) in input_cases.iter().zip(expected_cases) {
        let name = input_case["name"].as_str().unwrap();
        assert_eq!(name, expected_case["name"].as_str().unwrap());
        let power_words = input_case["power_f32_bits"].as_array().unwrap();
        let expected_values = expected_case["output_i16"].as_array().unwrap();
        assert_eq!(power_words.len(), 10, "{name} input count");
        assert_eq!(
            expected_values.len(),
            power_words.len(),
            "{name} output count"
        );
        let bias = i16::try_from(input_case["bias_i16"].as_i64().unwrap()).unwrap();
        let cap = i16::try_from(input_case["cap_i16"].as_i64().unwrap()).unwrap();

        for (index, (power_word, expected_value)) in
            power_words.iter().zip(expected_values).enumerate()
        {
            let power_word = u32::try_from(power_word.as_u64().unwrap()).unwrap();
            let expected_value = i16::try_from(expected_value.as_i64().unwrap()).unwrap();
            assert_eq!(
                raw_exponent_from_power_bits(power_word, bias, cap),
                expected_value,
                "{name} power word {index}"
            );
            output_count += 1;
        }
    }

    assert_eq!(output_count, 50);
}
