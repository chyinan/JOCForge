use jocforge_domain::codec::converter_header_cost::virtual_header_fixed_bit_cost;
use serde_json::Value;
use sha2::{Digest, Sha256};

const INPUT_SHA256: &str = "039fdea71f3aa976c65da0faf261b67d93746725a1636294ea65d9ddb519d75c";
const RESULT_SHA256: &str = "62a2c7500bf7dc2c489a45641ebd49238ed1e4577d798b328f3d4a9dcb2e7593";
const PROVENANCE_SHA256: &str = "4e46ef23700c4d0796b154df123f72026ca6271754d8cdeccee5e320c5b4aeff";
const INPUTS: &str = include_str!("fixtures/converter_header_cost/INPUT-ONLY-VECTORS.json");
const RESULT: &str = include_str!("fixtures/converter_header_cost/RESULT.json");
const PROVENANCE: &str = include_str!("fixtures/converter_header_cost/PROVENANCE.json");

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn f11_virtual_header_fixed_cost_matches_selected_unit_cases() {
    assert_eq!(sha256(INPUTS.as_bytes()), INPUT_SHA256);
    assert_eq!(sha256(RESULT.as_bytes()), RESULT_SHA256);
    assert_eq!(sha256(PROVENANCE.as_bytes()), PROVENANCE_SHA256);

    let inputs: Value = serde_json::from_str(INPUTS).unwrap();
    let expected: Value = serde_json::from_str(RESULT).unwrap();
    let provenance: Value = serde_json::from_str(PROVENANCE).unwrap();
    assert_eq!(inputs["input_only"], true);
    assert_eq!(
        inputs["source_package_manifest_sha256"],
        "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
    );
    assert_eq!(
        inputs["source_vector_sha256"],
        "a14dd6c3700fae1765399c72c816dd3146d57ab7dd026b375efb6aed7408cfa8"
    );
    assert_eq!(
        provenance["extraction_boundary"],
        "Copies only each unit input surround_ex_mode and expected header_bits field; no BAP parameter, provider table, audio, or product data is included."
    );

    let input_cases = inputs["cases"].as_array().unwrap();
    let expected_cases = expected["cases"].as_array().unwrap();
    assert_eq!(input_cases.len(), 2);
    assert_eq!(expected_cases.len(), input_cases.len());
    for (input, output) in input_cases.iter().zip(expected_cases) {
        assert_eq!(input["functional_id"], "F11.2");
        assert_eq!(input["id"], output["id"]);
        assert_eq!(
            virtual_header_fixed_bit_cost(
                u8::try_from(input["surround_ex_mode"].as_u64().unwrap()).unwrap()
            ),
            u16::try_from(output["header_bits"].as_u64().unwrap()).unwrap(),
            "{}",
            input["id"]
        );
    }
}
