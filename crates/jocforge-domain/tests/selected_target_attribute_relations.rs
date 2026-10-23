// pattern: Functional Core
use jocforge_domain::frontend::{
    PackageManifestEvidence, ProviderFileEvidence, validate_f03_package_manifest,
    validate_f03_target_cold_provider,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const WASHED_MANIFEST: &str =
    include_str!("fixtures/selected_target_attribute_lineage/washed-package-manifest.json");
const INDEPENDENT_CHECK: &str =
    include_str!("fixtures/selected_target_attribute_lineage/independent-check.json");
const WASHED_VECTORS: &[u8] =
    include_bytes!("fixtures/selected_target_attribute_lineage/washed-vectors.json");
const SOURCE_MANIFEST_SHA256: &str =
    "207c9f33d2ec921c7a13c90f015133aba543edde8f88a557b71acb659073568f";
const SOURCE_VECTORS_SHA256: &str =
    "dde8e489895978087eca37c070acc30220b805984fa3924fa17f521bbf868408";
const LOCAL_MANIFEST_SHA256: &str =
    "c46394dac37cd30fb02f2d8d4de63b7e4c4a01d966288e72a32ccbed9cde7f85";
const LOCAL_CHECK_SHA256: &str = "3da7d636f76629ae1511141d858eaa69f59922c2d4ba1b2ec14c458c066c5f1d";

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn validated_target_provider() -> jocforge_domain::frontend::ValidatedF03TargetColdProvider {
    let root = fixture_root();
    let manifest_path = root.join("f01e_provider/MANIFEST.json");
    let provider_path = root.join("target_cold/f03-selected-target-cold-v1.json");
    let manifest_evidence =
        PackageManifestEvidence::new(std::fs::read(manifest_path).unwrap().into(), None);
    let manifest = validate_f03_package_manifest(&manifest_evidence).unwrap();
    let provider_evidence =
        ProviderFileEvidence::new(std::fs::read(provider_path).unwrap().into(), None);
    validate_f03_target_cold_provider(&provider_evidence, &manifest).unwrap()
}

#[test]
fn washed_fixture_identity_and_non_ready_status_are_preserved() {
    let manifest: Value = serde_json::from_str(WASHED_MANIFEST).unwrap();
    let independent_check: Value = serde_json::from_str(INDEPENDENT_CHECK).unwrap();
    let vectors: Value = serde_json::from_slice(WASHED_VECTORS).unwrap();
    let actual_vectors_sha256 = format!("{:x}", Sha256::digest(WASHED_VECTORS));
    let actual_manifest_sha256 = format!("{:x}", Sha256::digest(WASHED_MANIFEST.as_bytes()));
    let actual_check_sha256 = format!("{:x}", Sha256::digest(INDEPENDENT_CHECK.as_bytes()));
    let vector_entry = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == "WASHED-VECTORS.json")
        .unwrap();

    assert_eq!(
        manifest["package"],
        "selected-target-attribute-lineage-washed-v1"
    );
    assert_eq!(actual_manifest_sha256, LOCAL_MANIFEST_SHA256);
    assert_eq!(actual_check_sha256, LOCAL_CHECK_SHA256);
    assert_eq!(vector_entry["bytes"], WASHED_VECTORS.len());
    assert_eq!(vector_entry["sha256"], SOURCE_VECTORS_SHA256);
    assert!(!manifest["production_usable"].as_bool().unwrap());
    assert_eq!(manifest["implementation_ready"], "NO");
    assert_eq!(independent_check["manifest_sha256"], SOURCE_MANIFEST_SHA256);
    assert!(
        independent_check["checks"]["all_hashes_match"]
            .as_bool()
            .unwrap()
    );
    assert!(
        independent_check["checks"]["no_research_addresses_in_washed_spec"]
            .as_bool()
            .unwrap()
    );
    assert!(
        independent_check["checks"]["no_private_key_material"]
            .as_bool()
            .unwrap()
    );
    assert!(!vectors["production_usable"].as_bool().unwrap());
    assert_eq!(actual_vectors_sha256, SOURCE_VECTORS_SHA256);

    for case_id in [
        "shortage_does_not_fail",
        "multiplicity_mask",
        "greedy_tie_order",
    ] {
        let case = vectors["vectors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"] == case_id)
            .unwrap();
        assert_eq!(case["runtime_status"], "STATIC_ONLY_NOT_RUN");
    }
}

#[test]
fn selected_cold_provider_matches_washed_target_relations() {
    let vectors: Value = serde_json::from_slice(WASHED_VECTORS).unwrap();
    let cold_case = vectors["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "cold_initialization")
        .unwrap();
    let provider = validated_target_provider();

    assert_eq!(cold_case["input"]["capacity"], 15);
    assert_eq!(provider.current_id_u32(), &[u32::MAX; 15]);
    assert_eq!(provider.previous_id_u32(), &[u32::MAX; 15]);
    assert_eq!(provider.current_attribute_u32(), &[0x0000_0b00; 15]);
    assert_eq!(provider.previous_attribute_u32(), &[0x0000_0b00; 15]);
    assert_eq!(provider.active_u8(), &[0; 15]);
    assert_eq!(provider.previous_marker_active_u8(), &[0; 15]);
    assert_eq!(cold_case["runtime_status"], "CONFIRMED_SELECTED_PATH");
}
