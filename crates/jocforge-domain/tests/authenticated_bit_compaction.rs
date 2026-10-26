use jocforge_domain::codec::authenticated_bit_compaction::{
    BitCompactionError, ClosedBitExclusion, compact_authenticated_bits,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const INPUT_SHA256: &str = "3d53b26ee249cde0a6531f3ffd96cd352cad5877c64a3f154ba57c24073b4b87";
const RESULT_SHA256: &str = "a90344b2a749e6f94744fce50e847a424661dbecc3a29437935fdf4f36174626";
const PROVENANCE_SHA256: &str = "e38b9184b9a6fccd5e63d24d58a3a7658910407bcf2466aec1f00498f16514bd";
const INPUTS: &str = include_str!("fixtures/authenticated_bit_compaction/INPUT-ONLY-VECTORS.json");
const RESULT: &str = include_str!("fixtures/authenticated_bit_compaction/RESULT.json");
const PROVENANCE: &str = include_str!("fixtures/authenticated_bit_compaction/PROVENANCE.json");

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = char::from(pair[0]).to_digit(16).unwrap();
            let low = char::from(pair[1]).to_digit(16).unwrap();
            u8::try_from(high * 16 + low).unwrap()
        })
        .collect()
}

fn exclusions(value: &Value) -> Vec<ClosedBitExclusion> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|pair| {
            let pair = pair.as_array().unwrap();
            ClosedBitExclusion::new(pair[0].as_u64().unwrap(), pair[1].as_u64().unwrap())
        })
        .collect()
}

#[test]
fn f14_compacts_selected_closed_bit_ranges_and_commits_only_full_words() {
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
            .contains("no key")
    );

    let input_cases = inputs["cases"].as_array().unwrap();
    let expected_cases = expected["cases"].as_array().unwrap();
    assert_eq!(input_cases.len(), 4);
    assert_eq!(expected_cases.len(), input_cases.len());
    for (input, expected) in input_cases.iter().zip(expected_cases) {
        let frame = decode_hex(input["frame_hex"].as_str().unwrap());
        let exclusions = exclusions(&input["closed_exclusions"]);
        let actual = compact_authenticated_bits(&frame, &exclusions).unwrap();
        assert_eq!(input["id"], expected["id"]);
        assert_eq!(
            actual.bytes(),
            decode_hex(expected["basis_hex"].as_str().unwrap())
        );
        assert_eq!(actual.kept_bits(), expected["kept_bits"].as_u64().unwrap());
        assert_eq!(
            actual.word_count(),
            expected["word_count"].as_u64().unwrap()
        );
        assert_eq!(
            actual.uncommitted_tail_bits(),
            u8::try_from(expected["uncommitted_bits"].as_u64().unwrap()).unwrap()
        );
    }
}

#[test]
fn f14_rejects_invalid_lengths_and_closed_ranges() {
    assert_eq!(
        compact_authenticated_bits(&[0], &[]),
        Err(BitCompactionError::OddInputLength)
    );
    assert_eq!(
        compact_authenticated_bits(&[0, 0], &[ClosedBitExclusion::new(4, 3)]),
        Err(BitCompactionError::ReversedRange)
    );
    assert_eq!(
        compact_authenticated_bits(&[0, 0], &[ClosedBitExclusion::new(0, 16)]),
        Err(BitCompactionError::RangeOutOfBounds)
    );
    assert_eq!(
        compact_authenticated_bits(
            &[0, 0],
            &[ClosedBitExclusion::new(0, 3), ClosedBitExclusion::new(3, 5)]
        ),
        Err(BitCompactionError::UnsortedOrOverlappingRanges)
    );
    assert_eq!(
        compact_authenticated_bits(
            &[0, 0],
            &[ClosedBitExclusion::new(4, 5), ClosedBitExclusion::new(1, 2)]
        ),
        Err(BitCompactionError::UnsortedOrOverlappingRanges)
    );
}
