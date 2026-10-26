use jocforge_domain::codec::converter_search::{ConverterSearchInput, search_converter_offset};
use serde_json::Value;
use sha2::{Digest, Sha256};

const INPUT_SHA256: &str = "e4a50d95de741d4a2fbc39694d2461c26dd1928f26191fc7c9a1b1f1f1ebec6b";
const RESULT_SHA256: &str = "5e12861e59549de9cbcdfbd1f01495586c2581af8b754503dcd019fe386e8530";
const PROVENANCE_SHA256: &str = "abf7bb5e8f23a3ce0a90c25e8c859660ad16c0d8fdefd0cb54cb8a6a0766a7f7";
const INPUTS: &str = include_str!("fixtures/converter_search/INPUT-ONLY-VECTORS.json");
const RESULT: &str = include_str!("fixtures/converter_search/RESULT.json");
const PROVENANCE: &str = include_str!("fixtures/converter_search/PROVENANCE.json");

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn u16_array<const N: usize>(value: &Value) -> [u16; N] {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| u16::try_from(item.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

#[test]
fn f11_search_preserves_candidate_order_for_the_zero_bap_unit_callback() {
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
    assert!(
        provenance["extraction_boundary"]
            .as_str()
            .unwrap()
            .contains("BAP provider table")
    );

    let input = &inputs["cases"][0];
    let expected = &expected["cases"][0];
    assert_eq!(input["id"], "U-SEARCH-01");
    assert_eq!(input["functional_id"], "F11.3-4");
    assert_eq!(input["bap_all_zero_for_every_candidate"], true);
    let input = ConverterSearchInput {
        header_cost: i16::try_from(input["header_bits"].as_i64().unwrap()).unwrap(),
        fixed_costs: u16_array::<6>(&input["fixed_bits"])
            .map(|value| i16::try_from(value).unwrap()),
        auxiliary_cost: i16::try_from(input["auxiliary_bits"].as_i64().unwrap()).unwrap(),
    };
    let result = search_converter_offset(&input, |_| [0_i16; 6]);
    assert_eq!(result.header_bits(), input.header_cost);
    assert_eq!(
        result.best_offset(),
        Some(u16::try_from(expected["offset"].as_u64().unwrap()).unwrap())
    );
    let expected_order = u16_array::<10>(&expected["candidate_order"]).map(i32::from);
    assert_eq!(result.candidate_order(), expected_order.as_slice());
}
