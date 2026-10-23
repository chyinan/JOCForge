// pattern: Functional Core
use jocforge_domain::codec::mantissa::{MantissaQuantizerError, quantize_mantissa};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const SOURCE_MANIFEST_SHA256: &str =
    "2f50ddb6f894d26d3a87390b0fb5d48021afe90e4fa5e7cff976e02ae83e0ae5";
const SOURCE_CHECK_SHA256: &str =
    "ed62b615af423b0ce98fbae9c7876c7ca7d60fa1853e7e95fd8ce68684492210";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_root() -> PathBuf {
    crate_root().join("tests/fixtures/mantissa_quantizer")
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn sanitized_package_identity_and_payloads_are_hash_bound() {
    let root = fixture_root();
    let manifest_bytes = std::fs::read(root.join("source-manifest.json")).unwrap();
    let check_bytes = std::fs::read(root.join("independent-check.json")).unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    let check: Value = serde_json::from_slice(&check_bytes).unwrap();
    let provenance: Value = serde_json::from_slice(
        &std::fs::read(
            crate_root().join("../../docs/implementation/mantissa-quantizer-provenance.json"),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(sha256(&manifest_bytes), SOURCE_MANIFEST_SHA256);
    assert_eq!(sha256(&check_bytes), SOURCE_CHECK_SHA256);
    assert_eq!(check["manifest_sha256"], SOURCE_MANIFEST_SHA256);
    assert_eq!(check["payload_count"], 7);
    assert_eq!(check["all_payload_hashes_match"], true);
    assert_eq!(check["scope_is_stateless_only"], true);
    assert_eq!(check["whole_status_not_ready"], true);
    assert_eq!(manifest["production_usable"], "SLICE_ONLY");
    assert_eq!(manifest["implementation_ready"], "NO");
    assert_eq!(provenance["source_manifest_sha256"], SOURCE_MANIFEST_SHA256);
    assert_eq!(
        provenance["source_independent_check_sha256"],
        SOURCE_CHECK_SHA256
    );
    assert_eq!(
        provenance["scope"]["whole_product_implementation_ready"],
        "NO"
    );

    let local_names = HashMap::from([
        ("SCOPE.md", "scope.md"),
        ("PROVENANCE.json", "provenance.json"),
        ("SPEC-SELECTED-CODEC-WIRE.md", "spec.md"),
        ("PARAMETERS-CODEC-WIRE-SELECTED.json", "parameters.json"),
        ("QUANTIZER-INPUT-VECTORS.json", "input-vectors.json"),
        ("QUANTIZER-VERIFICATION.json", "verification.json"),
        ("LOCAL-REVIEW-STATUS.json", "local-review-status.json"),
    ]);
    for entry in manifest["files"].as_array().unwrap() {
        let source_name = entry["path"].as_str().unwrap();
        let local_name = local_names.get(source_name).unwrap();
        assert_payload_matches(&root, entry, local_name);
    }
}

fn assert_payload_matches(root: &Path, entry: &Value, local_name: &str) {
    let bytes = std::fs::read(root.join(local_name)).unwrap();
    assert_eq!(entry["bytes"].as_u64().unwrap(), bytes.len() as u64);
    assert_eq!(entry["sha256"].as_str().unwrap(), sha256(&bytes));
}

#[test]
fn all_washed_quantizer_vectors_match_exact_codes_or_failure_status() {
    let root = fixture_root();
    let vectors: Value =
        serde_json::from_slice(&std::fs::read(root.join("input-vectors.json")).unwrap()).unwrap();
    let cases = vectors["quantizer_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 208);

    let mut success_count = 0;
    let mut error_count = 0;
    for case in cases {
        let word = u32::from_str_radix(case["f32_word"].as_str().unwrap(), 16).unwrap();
        let bap = u8::try_from(case["bap"].as_u64().unwrap()).unwrap();
        let actual = quantize_mantissa(word, bap);
        match case["status"].as_str().unwrap() {
            "OK" => {
                success_count += 1;
                assert_eq!(
                    actual.unwrap().value(),
                    u16::try_from(case["code"].as_u64().unwrap()).unwrap()
                );
            }
            "ERROR" => {
                error_count += 1;
                assert_eq!(actual, Err(MantissaQuantizerError::ReservedCodeSentinel));
            }
            status => panic!("unmapped quantizer status: {status}"),
        }
    }
    assert_eq!(success_count, 204);
    assert_eq!(error_count, 4);
}

#[test]
fn nonfinite_inputs_unsupported_bap_and_nonfinite_intermediates_fail_closed() {
    assert_eq!(
        quantize_mantissa(f32::INFINITY.to_bits(), 0),
        Err(MantissaQuantizerError::NonFiniteInput)
    );
    assert_eq!(
        quantize_mantissa(f32::NAN.to_bits(), 15),
        Err(MantissaQuantizerError::NonFiniteInput)
    );
    assert_eq!(
        quantize_mantissa(0.0_f32.to_bits(), 16),
        Err(MantissaQuantizerError::UnsupportedBap)
    );
    assert_eq!(
        quantize_mantissa(f32::MAX.to_bits(), 1),
        Err(MantissaQuantizerError::NonFiniteIntermediate)
    );
}
