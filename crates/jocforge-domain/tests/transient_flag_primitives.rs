// pattern: Functional Core
use jocforge_domain::analysis::dither::plan_selected_dither_flags;
use jocforge_domain::analysis::structure_refresh::{
    F07_BLOCKS_PER_FRAME, F07_FULLBAND_LANES, selected_structure_refresh_flags,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/transient_flag_primitives";
const INPUTS_SHA256: &str = "48298f5254fa5d2b5efaa1d83e263abbcb8fcc630db56a1812b220a2f9ecb387";
const EXPECTED_SHA256: &str = "69dfc823dc97a1f455054fd1ecdf9c5a4f2013b62c1b719a070660948456d210";
const PROVENANCE_SHA256: &str = "78ea36cde5f298ccf2d49ae1d86de24702dddcc732de24e8131c5b40f3e390d8";
const SOURCE_PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const SOURCE_SPEC_SHA256: &str = "61a957410a05e1e44c15d7a3f6622d2ff4c2a1135020f35261c0dee1979f1eac";
const SOURCE_VERIFICATION_SHA256: &str =
    "7aa92b4c0dee80910c8b9c52100bc2ca933903e33efde081e568aafc8ed854d7";
const SOURCE_REVIEW_SHA256: &str =
    "68e25c9805e03b39c22e76b3b39c84327721d3f8282418b3e818385838a751f2";
const STRUCTURE_OUTPUT_SHA256: &str =
    "724105a06037f6d9fc3c91f621469845fc28f867d0971b46d0ac3102509fff53";
const DITHER_OUTPUT_SHA256: &str =
    "0a94502a427bb52fa66757fe26f5afbe6ed29bf55b0d1a6cd96f186ce6886563";

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

fn ascii_flag(flag: bool) -> u8 {
    if flag { b'1' } else { b'0' }
}

#[test]
fn selected_profile_flag_inputs_are_bound_to_the_washed_spec_and_review() {
    let inputs_bytes = fixture_bytes("INPUTS.json");
    let expected_bytes = fixture_bytes("EXPECTED.json");
    let provenance_bytes = fixture_bytes("PROVENANCE.json");
    let inputs: Value = serde_json::from_slice(&inputs_bytes).unwrap();
    let expected: Value = serde_json::from_slice(&expected_bytes).unwrap();
    let provenance: Value = serde_json::from_slice(&provenance_bytes).unwrap();

    assert_eq!(sha256(&inputs_bytes), INPUTS_SHA256);
    assert_eq!(sha256(&expected_bytes), EXPECTED_SHA256);
    assert_eq!(sha256(&provenance_bytes), PROVENANCE_SHA256);
    assert_eq!(inputs["input_only"], true);
    assert_eq!(
        provenance["source_package_manifest_sha256"],
        SOURCE_PACKAGE_MANIFEST_SHA256
    );
    assert_eq!(provenance["source_spec_sha256"], SOURCE_SPEC_SHA256);
    assert_eq!(
        provenance["source_verification_sha256"],
        SOURCE_VERIFICATION_SHA256
    );
    assert_eq!(provenance["source_review_sha256"], SOURCE_REVIEW_SHA256);
    assert_eq!(
        provenance["current_spec_review_binding"],
        "R1_REVIEW_HASH_PRECEDES_CURRENT_SPEC_HASH"
    );
    assert_eq!(provenance["mismatch_count_in_source_review"], 0);
    assert_eq!(
        expected["structure_refresh"]["ascii_output_sha256"],
        STRUCTURE_OUTPUT_SHA256
    );
    assert_eq!(
        expected["dither"]["ascii_output_sha256"],
        DITHER_OUTPUT_SHA256
    );
}

#[test]
fn fixed_profile_structure_refresh_matches_all_4096_short_other_combinations() {
    let inputs = fixture_json("INPUTS.json");
    let expected = fixture_json("EXPECTED.json");
    let definition = &inputs["structure_refresh"];
    assert_eq!(definition["profile_spx"], 0);
    assert_eq!(definition["profile_end_bin"], 217);
    assert_eq!(definition["blocks"], F07_BLOCKS_PER_FRAME);
    assert_eq!(definition["bit_order"], "bit b supplies block b");

    let short_min = u32::try_from(definition["short_mask_min"].as_u64().unwrap()).unwrap();
    let short_max = u32::try_from(definition["short_mask_max"].as_u64().unwrap()).unwrap();
    let other_min = u32::try_from(definition["other_mask_min"].as_u64().unwrap()).unwrap();
    let other_max = u32::try_from(definition["other_mask_max"].as_u64().unwrap()).unwrap();
    let mut serialized = Vec::new();

    for short_mask in short_min..=short_max {
        for other_mask in other_min..=other_max {
            let short = std::array::from_fn(|block| short_mask & (1 << block) != 0);
            let other = std::array::from_fn(|block| other_mask & (1 << block) != 0);
            let refresh = selected_structure_refresh_flags(&short, &other);
            serialized.extend(refresh.as_array().iter().copied().map(ascii_flag));
        }
    }

    assert_eq!(serialized.len(), 24_576);
    assert_eq!(
        serialized.len(),
        expected["structure_refresh"]["serialized_flag_count"]
    );
    assert_eq!(sha256(&serialized), STRUCTURE_OUTPUT_SHA256);
}

#[test]
fn selected_profile_dither_matches_2048_ordered_synthetic_plans() {
    let inputs = fixture_json("INPUTS.json");
    let expected = fixture_json("EXPECTED.json");
    let definition = &inputs["dither"];
    assert_eq!(definition["blocks"], F07_BLOCKS_PER_FRAME);
    assert_eq!(definition["fullband_lanes"], F07_FULLBAND_LANES);
    assert_eq!(definition["selected_profile"]["selector"], 7);
    assert_eq!(definition["selected_profile"]["acmod"], 7);
    assert_eq!(definition["selected_profile"]["coupling"], false);
    assert_eq!(definition["selected_profile"]["gate"], true);
    assert_eq!(
        definition["current_short_recipe"],
        "short[b][lane]=(pattern>>((b+lane)%6))&1"
    );

    let previous_min = u32::try_from(definition["previous_mask_min"].as_u64().unwrap()).unwrap();
    let previous_max = u32::try_from(definition["previous_mask_max"].as_u64().unwrap()).unwrap();
    let pattern_min = u32::try_from(definition["pattern_min"].as_u64().unwrap()).unwrap();
    let pattern_max = u32::try_from(definition["pattern_max"].as_u64().unwrap()).unwrap();
    let mut serialized = Vec::new();

    for previous_mask in previous_min..=previous_max {
        for pattern in pattern_min..=pattern_max {
            let previous = std::array::from_fn(|lane| previous_mask & (1 << lane) != 0);
            let current = std::array::from_fn(|block| {
                std::array::from_fn(|lane| pattern & (1 << ((block + lane) % 6)) != 0)
            });
            let plan = plan_selected_dither_flags(previous, &current);
            for block in plan.flags().as_array() {
                serialized.extend(block.iter().copied().map(ascii_flag));
            }
            serialized.push(ascii_flag(plan.syntax_present()));
            serialized.extend(plan.next_previous_short().iter().copied().map(ascii_flag));
        }
    }

    assert_eq!(serialized.len(), 73_728);
    assert_eq!(
        serialized.len(),
        expected["dither"]["serialized_flag_count"]
    );
    assert_eq!(sha256(&serialized), DITHER_OUTPUT_SHA256);
}
