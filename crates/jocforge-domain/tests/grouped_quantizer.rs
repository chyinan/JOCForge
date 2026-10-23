// pattern: Functional Core
use jocforge_domain::codec::grouped_quantizer::{
    GroupedBap, GroupedItemInput, GroupedItemObservation, GroupedPackObservation,
    GroupedQuantizerState,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const PACKAGE_DIRECTORY: &str = "tests/fixtures/grouped_quantizer";
const PACKAGE_MANIFEST_SHA256: &str =
    "59d070c4678fcf3a4d63d6e9d746c454a59050ba0dc8fdb14fa1c20cab642a96";
const INDEPENDENT_CHECK_SHA256: &str =
    "e0244cb95bbd12823ab790eb5643a79bc2c00e03626f7fd2d9dabb2b349d90d7";
const TRANSITION_CONTRACT_SHA256: &str =
    "e54e02a8f37f11e4d6911ce9bdf312e385143715d354a26f683da3ddf03afb18";
const PACKAGE_PAYLOADS: [(&str, usize, &str); 5] = [
    (
        "SCOPE.md",
        1324,
        "4c43b1102934f37f74a1e4a6895885f00989c91fc5bd99cfbdbec9b7c850ddbc",
    ),
    (
        "TRANSITION-CONTRACT.md",
        2252,
        "e54e02a8f37f11e4d6911ce9bdf312e385143715d354a26f683da3ddf03afb18",
    ),
    (
        "PROVENANCE.json",
        1090,
        "83de507f41a80baca2c9731dddbbdea6e38e0389bd68e464dd0f1ededf0a3cb1",
    ),
    (
        "R18-VECTORS.json",
        22316,
        "2227c98e31699ae3a6759a32385e665d8e8aaf487c5629495b92707ad1667388",
    ),
    (
        "R15-CAPACITY-VECTORS.json",
        17902,
        "2c1f9c645866d7bb6f413ae2b86faa2b9400203acef2b06fddc8cbda8bf6eef3",
    ),
];

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json(name: &str) -> (Vec<u8>, Value) {
    let bytes = read_bytes(name);
    let json = serde_json::from_slice(&bytes).unwrap();
    (bytes, json)
}

fn read_bytes(name: &str) -> Vec<u8> {
    std::fs::read(crate_root().join(PACKAGE_DIRECTORY).join(name)).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn bap(value: &Value) -> GroupedBap {
    match value.as_u64().unwrap() {
        1 => GroupedBap::Bap1,
        2 => GroupedBap::Bap2,
        4 => GroupedBap::Bap4,
        other => panic!("out-of-scope BAP {other}"),
    }
}

fn coefficient_word(value: &Value) -> u32 {
    if let Some(text) = value.as_str() {
        match text {
            "NaN" => f32::NAN.to_bits(),
            "Infinity" => f32::INFINITY.to_bits(),
            "-Infinity" => f32::NEG_INFINITY.to_bits(),
            other => panic!("unexpected coefficient token {other}"),
        }
    } else {
        (value.as_f64().unwrap() as f32).to_bits()
    }
}

#[test]
fn package_manifest_independent_check_and_all_payload_hashes_match() {
    let (check_bytes, check) = read_json("INDEPENDENT-CHECK.json");
    assert_eq!(sha256(&check_bytes), INDEPENDENT_CHECK_SHA256);
    assert_eq!(check["manifest_sha256"], PACKAGE_MANIFEST_SHA256);
    assert_eq!(check["schema"], "f12-grouped-quantizer-local-check.v2");
    assert_eq!(check["payload_count"], 5);
    assert_eq!(check["all_payload_hashes_match"], true);
    assert_eq!(check["transition_contract_bound"], true);
    assert_eq!(check["out_of_domain_bap_not_claimed"], true);
    assert_eq!(check["caller_capacity_not_closed"], true);
    assert_eq!(check["whole_status_not_ready"], true);
    for (name, expected_bytes, expected_hash) in PACKAGE_PAYLOADS {
        let bytes = read_bytes(name);
        assert_eq!(bytes.len(), expected_bytes, "{name}");
        assert_eq!(sha256(&bytes), expected_hash, "{name}");
    }
}

#[test]
fn r18_and_r15_grouped_cases_match_observed_items_packs_and_writer_state() {
    for file in ["R18-VECTORS.json", "R15-CAPACITY-VECTORS.json"] {
        let (_, vectors) = read_json(file);
        for vector in vectors["vectors"].as_array().unwrap() {
            assert_grouped_vector(vector);
        }
    }
}

fn assert_grouped_vector(vector: &Value) {
    let name = vector["name"].as_str().unwrap();
    let input = &vector["input"];
    let baps = input["baps"].as_array().unwrap();
    let coefficients = input["coefficients_f32"].as_array().unwrap();
    assert_eq!(baps.len(), coefficients.len(), "{name}");
    let inputs = baps
        .iter()
        .zip(coefficients)
        .map(|(bap_value, coefficient)| GroupedItemInput {
            bap: bap(bap_value),
            coefficient_word: coefficient_word(coefficient),
        })
        .collect::<Vec<_>>();

    let mut state = GroupedQuantizerState::cold();
    let quantizer_call = if inputs.is_empty() {
        None
    } else {
        Some(
            state
                .quantize(&inputs)
                .unwrap_or_else(|error| panic!("{name} quantizer: {error:?}")),
        )
    };
    if vector["quantizer_return"].is_null() {
        assert!(quantizer_call.is_none(), "{name} quantizer call");
    } else {
        assert_eq!(
            quantizer_call.as_ref().unwrap().return_code(),
            vector["quantizer_return"].as_i64().unwrap() as i32,
            "{name} quantizer return"
        );
    }

    let after_quantizer_counts = state.group_counts();
    let after_quantizer_bytes = state.emitted_word_memory_bytes().len();
    if let Some(expected_counts) = vector.get("state_counts_after_quantizer") {
        assert_u32s(&after_quantizer_counts, expected_counts, name);
        assert_eq!(
            state.pending_bit_count(),
            vector["pending_bits_after_quantizer"].as_u64().unwrap() as u8,
            "{name} pending bits after quantizer"
        );
        assert_eq!(
            state.pending_accumulator_u16(),
            vector["pending_accumulator_after_quantizer"]
                .as_u64()
                .unwrap() as u16,
            "{name} pending accumulator after quantizer"
        );
        assert_eq!(
            after_quantizer_bytes,
            vector["emitted_bytes_after_quantizer"].as_u64().unwrap() as usize,
            "{name} emitted bytes after quantizer"
        );
    }

    let mut observed_items = quantizer_call
        .as_ref()
        .map(|call| call.items().to_vec())
        .unwrap_or_default();
    let mut observed_packs = quantizer_call
        .as_ref()
        .map(|call| call.packs().to_vec())
        .unwrap_or_default();
    if !vector["flush_return"].is_null() {
        let flush = state
            .flush()
            .unwrap_or_else(|error| panic!("{name} flush: {error:?}"));
        assert_eq!(
            flush.return_code(),
            vector["flush_return"].as_i64().unwrap() as i32,
            "{name} flush return"
        );
        observed_items.extend_from_slice(flush.items());
        observed_packs.extend_from_slice(flush.packs());
    }

    assert_items(&observed_items, &vector["observed_items"], name);
    assert_packs(&observed_packs, &vector["observed_packs"], name);
    assert_u32s(&state.group_counts(), &vector["state_counts"], name);
    assert_eq!(
        state.pending_bit_count(),
        vector["pending_bit_count"].as_u64().unwrap() as u8,
        "{name} final pending bits"
    );
    assert_eq!(
        state.pending_accumulator_u16(),
        vector["pending_accumulator_u16"].as_u64().unwrap() as u16,
        "{name} final accumulator"
    );
    assert_eq!(
        state.effective_bits(),
        vector["effective_bits"].as_u64().unwrap(),
        "{name} effective bits"
    );
    assert_eq!(
        state.emitted_word_memory_bytes(),
        vector["emitted_word_memory_bytes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|byte| u8::try_from(byte.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>(),
        "{name} output bytes"
    );
    assert_eq!(
        sha256(state.emitted_word_memory_bytes()),
        vector["emitted_word_memory_sha256"].as_str().unwrap(),
        "{name} output digest"
    );

    let logical_capacity = input["logical_capacity_bytes"].as_u64();
    let capacity_exceeded = logical_capacity.is_some_and(|capacity| {
        u64::try_from(state.emitted_word_memory_bytes().len()).unwrap() > capacity
    });
    assert_eq!(
        capacity_exceeded,
        vector["capacity_exceeded_without_error"].as_bool().unwrap(),
        "{name} logical capacity observation"
    );
}

fn assert_u32s(actual: &[u32], expected: &Value, context: &str) {
    let expected = expected
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "{context}");
}

fn assert_items(actual: &[GroupedItemObservation], expected: &Value, context: &str) {
    let expected = expected.as_array().unwrap();
    assert_eq!(actual.len(), expected.len(), "{context} item count");
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(
            actual.bap().value(),
            expected["bap"].as_u64().unwrap() as u8,
            "{context} item {index} BAP"
        );
        assert_eq!(
            actual.code(),
            expected["code"].as_u64().unwrap() as u16,
            "{context} item {index} code"
        );
        assert_eq!(
            actual.width(),
            expected["width"].as_u64().unwrap() as u8,
            "{context} item {index} width"
        );
    }
    assert_eq!(
        sha256(&read_bytes("TRANSITION-CONTRACT.md")),
        TRANSITION_CONTRACT_SHA256
    );
}

fn assert_packs(actual: &[GroupedPackObservation], expected: &Value, context: &str) {
    let expected = expected.as_array().unwrap();
    assert_eq!(actual.len(), expected.len(), "{context} pack count");
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(
            actual.value(),
            expected["value"].as_u64().unwrap() as u16,
            "{context} pack {index} value"
        );
        assert_eq!(
            actual.width(),
            expected["width"].as_u64().unwrap() as u8,
            "{context} pack {index} width"
        );
        assert_eq!(
            actual.start_bits(),
            expected["start_bits"].as_u64().unwrap(),
            "{context} pack {index} start"
        );
    }
}
