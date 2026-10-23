// pattern: Functional Core
use jocforge_domain::spatial::target_slot_install::{
    SelectedTargetSlots, TargetSlotInstallError, TargetSlotState, install_target_slots,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const SOURCE_SPEC_SHA256: &str = "aeb6791967413e71518cfb1c5853c52cd8e6e7e7174f60afa611cc0e0fa8d9dc";
const SOURCE_ADAPTER_SHA256: &str =
    "9bce2e35ce47147c76c13e58ee976d44c5109938f0b19e1f3a81f1d8a61fa1ae";
const SOURCE_VECTORS_SHA256: &str =
    "a14dd6c3700fae1765399c72c816dd3146d57ab7dd026b375efb6aed7408cfa8";
const SOURCE_REVIEW_SHA256: &str =
    "b462c5614d26bded5abc66134606c34d0a85c1ab21802057150e17b229e79fca";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn vectors() -> Value {
    serde_json::from_slice(
        &std::fs::read(crate_root().join("tests/fixtures/f04_component/unit-vectors.json"))
            .unwrap(),
    )
    .unwrap()
}

fn words(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .unwrap_or_else(|| panic!("expected array, got {value}"))
        .iter()
        .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
        .collect()
}

fn points(value: &Value) -> Vec<[u32; 3]> {
    words(value)
        .chunks_exact(3)
        .map(|point| [point[0], point[1], point[2]])
        .collect()
}

fn state(value: &Value) -> TargetSlotState {
    let active = value["active_flags_u32"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u8::try_from(value.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>();
    TargetSlotState {
        xyz_words: points(&value["XYZ_words"]).try_into().unwrap(),
        ids_u32: words(&value["IDs_u32"]).try_into().unwrap(),
        attributes_u32: words(&value["attributes_u32"]).try_into().unwrap(),
        previous_xyz_words: points(&value["previous_XYZ_words"]).try_into().unwrap(),
        previous_ids_u32: words(&value["previous_IDs_u32"]).try_into().unwrap(),
        previous_attributes_u32: words(&value["previous_attributes_u32"]).try_into().unwrap(),
        active_flags: active.try_into().unwrap(),
    }
}

#[test]
fn package_assets_and_four_target_install_entries_are_hash_bound() {
    let manifest_bytes =
        std::fs::read(crate_root().join("tests/fixtures/f01e_provider/MANIFEST.json")).unwrap();
    let vector_bytes =
        std::fs::read(crate_root().join("tests/fixtures/f04_component/unit-vectors.json")).unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    assert_eq!(sha256(&manifest_bytes), PACKAGE_MANIFEST_SHA256);
    assert_eq!(sha256(&vector_bytes), SOURCE_VECTORS_SHA256);
    for (path, bytes, hash) in [
        (
            "specs/03-gate-and-target-installation.md",
            5_475,
            SOURCE_SPEC_SHA256,
        ),
        (
            "test-vectors/unit/ADAPTER-CONTRACT.md",
            22_205,
            SOURCE_ADAPTER_SHA256,
        ),
        (
            "test-vectors/unit/vectors.json",
            1_503_142,
            SOURCE_VECTORS_SHA256,
        ),
        (
            "test-vectors/OUTER-LOCAL-REVIEW-STATUS.json",
            21_337,
            SOURCE_REVIEW_SHA256,
        ),
    ] {
        let entry = manifest["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["file"] == path)
            .unwrap();
        assert_eq!(entry["bytes"], bytes);
        assert_eq!(entry["sha256"], hash);
    }
    let all_vectors = vectors();
    let ids = all_vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["entry_id"] == "target-slot-install")
        .map(|case| case["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 4);
    assert_eq!(
        ids,
        [
            "U-FRONT-INSTALL-EMPTY",
            "U-FRONT-INSTALL-RECT",
            "U-FRONT-INSTALL-TIES",
            "U-FRONT-INSTALL-ATTR"
        ]
    );
}

#[test]
fn empty_rectangular_tie_and_attribute_install_vectors_match_full_state() {
    let vectors = vectors();
    for id in [
        "U-FRONT-INSTALL-EMPTY",
        "U-FRONT-INSTALL-RECT",
        "U-FRONT-INSTALL-TIES",
    ] {
        let case = vectors["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"] == id)
            .unwrap();
        let input = &case["input"];
        let old = state(&input["old_state"]);
        let new = SelectedTargetSlots {
            xyz_words: points(&input["new_xyz_u32"]),
            ids_u32: words(&input["new_ids"]),
            attributes_u32: words(&input["new_attributes"]),
        };
        let result =
            install_target_slots(&old, &new).unwrap_or_else(|error| panic!("{id}: {error:?}"));
        assert_eq!(
            result.mapping(),
            parse_i16(&case["expected"]["mapping"]),
            "{id}"
        );
        assert_eq!(
            result.cost_u32(),
            words(&case["expected"]["cost_u32"]),
            "{id}"
        );
        assert_eq!(
            result.empty_branch(),
            case["expected"]["empty_branch"].as_bool().unwrap()
        );
        let next = result.next_state();
        assert_eq!(
            flatten_points(&next.xyz_words),
            words(&case["expected"]["XYZ_u32"]),
            "{id}"
        );
        assert_eq!(
            next.ids_u32.as_slice(),
            words(&case["expected"]["IDs_u32"]),
            "{id}"
        );
        assert_eq!(
            next.attributes_u32.as_slice(),
            words(&case["expected"]["attributes_u32"]),
            "{id}"
        );
        assert_eq!(
            flatten_points(&next.previous_xyz_words),
            words(&case["expected"]["previous_XYZ_u32"]),
            "{id}"
        );
        assert_eq!(
            next.previous_ids_u32.as_slice(),
            words(&case["expected"]["previous_IDs_u32"]),
            "{id}"
        );
        assert_eq!(
            next.previous_attributes_u32.as_slice(),
            words(&case["expected"]["previous_attributes_u32"]),
            "{id}"
        );
        assert_eq!(
            next.active_flags,
            parse_u8(&case["expected"]["active_flags"]),
            "{id}"
        );
    }
}

#[test]
fn attribute_multiplicity_rejects_before_state_is_returned() {
    let vectors = vectors();
    let case = vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "U-FRONT-INSTALL-ATTR")
        .unwrap();
    let input = &case["input"];
    let old = state(&input["old_state"]);
    let new = SelectedTargetSlots {
        xyz_words: points(&input["new_xyz_u32"]),
        ids_u32: words(&input["new_ids"]),
        attributes_u32: words(&input["new_attributes"]),
    };
    assert_eq!(
        install_target_slots(&old, &new),
        Err(TargetSlotInstallError::UnsupportedAttributeMultiplicity)
    );
}

#[test]
fn malformed_shape_active_flag_and_nonfinite_cost_are_rejected() {
    let vectors = vectors();
    let case = vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "U-FRONT-INSTALL-EMPTY")
        .unwrap();
    let mut old = state(&case["input"]["old_state"]);

    let mismatched = SelectedTargetSlots {
        xyz_words: Vec::new(),
        ids_u32: vec![0],
        attributes_u32: Vec::new(),
    };
    assert_eq!(
        install_target_slots(&old, &mismatched),
        Err(TargetSlotInstallError::InvalidInput)
    );

    old.active_flags[0] = 2;
    assert_eq!(
        install_target_slots(
            &old,
            &SelectedTargetSlots {
                xyz_words: Vec::new(),
                ids_u32: Vec::new(),
                attributes_u32: Vec::new(),
            }
        ),
        Err(TargetSlotInstallError::InvalidInput)
    );

    old.active_flags[0] = 1;
    let overflow = SelectedTargetSlots {
        xyz_words: vec![[0xff7f_ffff, 0, 0]],
        ids_u32: vec![old.ids_u32[0]],
        attributes_u32: vec![old.attributes_u32[0]],
    };
    old.xyz_words[0][0] = 0x7f7f_ffff;
    assert_eq!(
        install_target_slots(&old, &overflow),
        Err(TargetSlotInstallError::NumericFailure)
    );
}

fn parse_i16(value: &Value) -> Vec<i16> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| i16::try_from(value.as_i64().unwrap()).unwrap())
        .collect()
}

fn parse_u8(value: &Value) -> [u8; 15] {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u8::try_from(value.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn flatten_points(points: &[[u32; 3]; 15]) -> Vec<u32> {
    points.iter().flatten().copied().collect()
}
