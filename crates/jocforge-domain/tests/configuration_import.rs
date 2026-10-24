// pattern: Functional Core
use jocforge_domain::codec::configuration_import::{
    ENCODER_OWNER_BYTES, METADATA_BUFFER_BYTES, SOURCE_CONFIGURATION_BYTES,
    import_selected_configuration,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/selected_configuration_import";
const PACKAGE_MANIFEST_SHA256: &str =
    "8ad92e6a0372414c4f2319f996399a5167379922ff3ebaa3d29467f339bd61b1";
const SOURCE_MANIFEST_SHA256: &str =
    "51931af77f7264d8b38037b90c52fa874323a60493214aed584233e332ce3d3e";
const SOURCE_INDEPENDENT_CHECK_SHA256: &str =
    "408583cb04f4b0ca76e4565bef20fd61c2e8d005d440bf86d9caa9bdd0caf766";
const INDEPENDENT_CHECK_SHA256: &str =
    "ffed91e451eb8eb58a41a46d2412e32a64551f80d149dd2b526d5fc225576ff8";
const PACKAGE_PAYLOADS: [(&str, usize, &str); 8] = [
    (
        "CONTRACT.md",
        2734,
        "35781fb548beef5e54409d2f1458cfc759121ad33f8fbc968d2e4da9330f37c1",
    ),
    (
        "CONTRACT.json",
        10413,
        "4f9bbed8c90fbac996d82cf4b0ed61f177fc396644c866ae2e1b9a57aa2b1906",
    ),
    (
        "INPUTS.json",
        42574,
        "ea2b7ab2bede4924e60f069b8632b4e7c1102dd63bb30c5f7e0dd03ff3cd8c44",
    ),
    (
        "EXPECTED.json",
        449521,
        "fee6e9d8d10bed3eaa54c86f77084ec0cfe569dd3e6ce6a8bfa3eeb775084a71",
    ),
    (
        "PARAMETERS.json",
        1924,
        "7155f238fe62097490addbaf6893d6386bf7a6cdbac928c74ebf252e5553998f",
    ),
    (
        "RUNTIME-RECEIPT.json",
        993,
        "a95bdcf94ad79551fc7fe7ed82eb58f791bdef6a8a8fc1d579dd52e3410abaad",
    ),
    (
        "NATIVE-LINEAGE.json",
        5240,
        "72342c45d45cd77b01336ead87aa1150de3f13db89abe1a2ae3b6215b2559f53",
    ),
    (
        "PROVENANCE.json",
        3705,
        "6513c5acd36c2c42c8a04a5fde1380b0594a12ed7fea86b6a27e839d67af4bd9",
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

fn expected_bytes(value: &Value) -> Vec<u8> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|byte| u8::try_from(byte.as_u64().unwrap()).unwrap())
        .collect()
}

#[test]
fn package_identity_and_all_eight_manifested_payloads_are_bound() {
    let check_bytes = fixture_bytes("INDEPENDENT-CHECK.json");
    let check: Value = serde_json::from_slice(&check_bytes).unwrap();
    let provenance = fixture_json("PROVENANCE.json");

    assert_eq!(sha256(&check_bytes), INDEPENDENT_CHECK_SHA256);
    assert_eq!(check["manifest_sha256"], PACKAGE_MANIFEST_SHA256);
    assert_eq!(check["payload_count"], 8);
    assert_eq!(check["payload_hashes_match"], true);
    assert_eq!(check["source_manifest_check_and_payloads_match"], true);
    assert_eq!(check["controlled_cases"], 7);
    assert_eq!(check["owner_bytes_replayed"], 28672);
    assert_eq!(check["metadata_bytes_replayed"], 448);
    assert_eq!(
        check["independent_copy_and_numeric_replay"],
        "PASS all owner and metadata bytes"
    );
    assert_eq!(check["partial_mutation_on_return_59"], true);
    assert_eq!(
        check["channel_map_return_161"],
        "STATIC_ONLY; no rejecting runtime vector"
    );
    assert_eq!(check["whole_f09_closed"], false);
    assert_eq!(check["whole_implementation_ready"], "NO");
    assert_eq!(provenance["source_manifest_sha256"], SOURCE_MANIFEST_SHA256);
    assert_eq!(
        provenance["source_independent_check_sha256"],
        SOURCE_INDEPENDENT_CHECK_SHA256
    );

    for (name, expected_size, expected_hash) in PACKAGE_PAYLOADS {
        let bytes = fixture_bytes(name);
        assert_eq!(bytes.len(), expected_size, "{name} bytes");
        assert_eq!(sha256(&bytes), expected_hash, "{name} SHA-256");
    }
}

#[test]
fn seven_input_only_recipes_match_ordered_owner_and_metadata_mutations() {
    let inputs = fixture_json("INPUTS.json");
    let expected = fixture_json("EXPECTED.json");
    assert_eq!(inputs["expected_used_as_seed"], false);
    assert_eq!(inputs["recipes"].as_array().unwrap().len(), 7);
    assert_eq!(expected["cases"].as_array().unwrap().len(), 7);

    let mut return_values = Vec::new();
    for (recipe, expected_case) in inputs["recipes"]
        .as_array()
        .unwrap()
        .iter()
        .zip(expected["cases"].as_array().unwrap())
    {
        assert_eq!(recipe["source_bytes"], SOURCE_CONFIGURATION_BYTES);
        assert_eq!(recipe["target_bytes"], ENCODER_OWNER_BYTES);
        assert_eq!(recipe["metadata_bytes"], METADATA_BUFFER_BYTES);
        let mut source = [u8::try_from(recipe["source_initial_fill"].as_u64().unwrap()).unwrap();
            SOURCE_CONFIGURATION_BYTES];
        for patch in recipe["input_byte_patches"].as_array().unwrap() {
            let offset = usize::try_from(patch[0].as_u64().unwrap()).unwrap();
            let value = u8::try_from(patch[1].as_u64().unwrap()).unwrap();
            source[offset] = value;
        }
        assert_eq!(sha256(&source), expected_case["input_sha256"]);

        let fill = u8::try_from(recipe["target_initial_fill"].as_u64().unwrap()).unwrap();
        let mut owner = [fill; ENCODER_OWNER_BYTES];
        let mut metadata = [fill; METADATA_BUFFER_BYTES];
        let owner_before_sha = sha256(&owner);
        let metadata_before_sha = sha256(&metadata);
        assert_eq!(expected_case["before"]["target_sha256"], owner_before_sha);
        assert_eq!(
            expected_case["before"]["metadata_sha256"],
            metadata_before_sha
        );

        let returned = import_selected_configuration(&source, &mut owner, &mut metadata);
        let expected_return = expected_case["return_value"]
            .as_str()
            .unwrap()
            .parse::<i32>()
            .unwrap();
        assert_eq!(returned, expected_return, "{}", recipe["name"]);
        assert_eq!(
            owner.as_slice(),
            expected_bytes(&expected_case["after"]["owner_bytes"]).as_slice(),
            "{} owner bytes",
            recipe["name"]
        );
        assert_eq!(
            metadata.as_slice(),
            expected_bytes(&expected_case["after"]["metadata_bytes"]).as_slice(),
            "{} metadata bytes",
            recipe["name"]
        );
        assert_eq!(
            sha256(&owner),
            expected_case["after"]["target_sha256"].as_str().unwrap()
        );
        assert_eq!(
            sha256(&metadata),
            expected_case["after"]["metadata_sha256"].as_str().unwrap()
        );
        assert_eq!(
            &owner[ENCODER_OWNER_BYTES - 16..],
            expected_bytes(&expected_case["target_tail"]).as_slice()
        );
        assert_eq!(
            &metadata[METADATA_BUFFER_BYTES - 16..],
            expected_bytes(&expected_case["metadata_tail"]).as_slice()
        );

        if returned == 140 {
            assert_eq!(
                sha256(&owner),
                owner_before_sha,
                "prefix failure owner is unchanged"
            );
            assert_eq!(
                sha256(&metadata),
                metadata_before_sha,
                "prefix failure metadata is unchanged"
            );
        }
        if recipe["name"] == "invalid_rate_after_copy" {
            assert_eq!(returned, 59);
            assert_ne!(sha256(&owner), owner_before_sha);
            assert_ne!(sha256(&metadata), metadata_before_sha);
        }
        return_values.push(returned);
    }

    assert_eq!(return_values, [0, 0, 0, 0, 140, 140, 59]);
}
