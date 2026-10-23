// pattern: Functional Core
use jocforge_domain::joc::huffman::{
    HuffmanBitCount, Id14HeaderInput, Id14ObjectDescriptor, Id14Presence, Id14WriterState,
    write_id14_header,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const SOURCE_MANIFEST_SHA256: &str =
    "60f778ba40c6bd6fe65e85d7ae500dc4d9ee6527d976f3a241b41b360743968e";
const SOURCE_CHECK_SHA256: &str =
    "1bac23f5bb459d750e1643798701c4cce4076e9d6164272e726e4783c67f3044";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_root() -> PathBuf {
    crate_root().join("tests/fixtures/id14_header_framing")
}

fn assert_source_asset(root: &Path, manifest: &Value, package_path: &str, local_path: &str) {
    let source = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == package_path)
        .unwrap();
    let bytes = std::fs::read(root.join(local_path)).unwrap();
    assert_eq!(
        source["bytes"].as_u64().unwrap(),
        u64::try_from(bytes.len()).unwrap()
    );
    assert_eq!(source["sha256"].as_str().unwrap(), sha256(&bytes));
}

#[test]
fn washed_package_identity_and_payload_hashes_are_bound() {
    let root = crate_root();
    let fixture_root = fixture_root();
    let manifest_bytes = std::fs::read(fixture_root.join("source-manifest.json")).unwrap();
    let check_bytes = std::fs::read(fixture_root.join("independent-check.json")).unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    let check: Value = serde_json::from_slice(&check_bytes).unwrap();

    assert_eq!(sha256(&manifest_bytes), SOURCE_MANIFEST_SHA256);
    assert_eq!(sha256(&check_bytes), SOURCE_CHECK_SHA256);
    assert_eq!(check["manifest_sha256"], SOURCE_MANIFEST_SHA256);
    assert!(check["all_payload_hashes_match"].as_bool().unwrap());
    assert!(check["sanitized_scan_pass"].as_bool().unwrap());
    assert_eq!(manifest["implementation_ready"], "NO");
    assert_source_asset(
        &root,
        &manifest,
        "ID14-HEADER-FRAMING-VECTORS.json",
        "tests/fixtures/id14_header_framing/input-vectors.json",
    );
    assert_source_asset(
        &root,
        &manifest,
        "BOUNDED-COVERAGE.md",
        "tests/fixtures/id14_header_framing/bounded-coverage.md",
    );
    assert_source_asset(
        &root,
        &manifest,
        "ID14-HEADER-FRAMING.md",
        "../../docs/implementation/id14-header-framing.md",
    );
    assert_source_asset(
        &root,
        &manifest,
        "PROVENANCE.json",
        "../../docs/implementation/id14-header-framing-provenance.json",
    );
}

#[test]
fn selected_writer_vectors_match_status_return_length_and_bytes() {
    let fixture_root = fixture_root();
    let vectors: Value =
        serde_json::from_slice(&std::fs::read(fixture_root.join("input-vectors.json")).unwrap())
            .unwrap();

    for case in vectors["vectors"].as_array().unwrap() {
        let recipe = &case["input_recipe"];
        let capacity = recipe["capacity_bytes"].as_u64().unwrap();
        let initial_buffer = if let Some(initial_hex) = recipe["initial_output_buffer_hex"].as_str()
        {
            decode_hex(initial_hex)
        } else {
            vec![0xaa; usize::try_from(capacity).unwrap()]
        };
        assert_eq!(u64::try_from(initial_buffer.len()).unwrap(), capacity);
        let (descriptors, null_descriptor_pointer) = descriptors_for(recipe);
        assert_eq!(
            null_descriptor_pointer,
            recipe["null_object_pointer"].as_bool().unwrap()
        );
        let body = decode_hex(recipe["body_hex"].as_str().unwrap());
        let input = Id14HeaderInput {
            downmix_index: u32::try_from(recipe["downmix_index"].as_u64().unwrap()).unwrap(),
            object_count: u32::try_from(recipe["object_count"].as_u64().unwrap()).unwrap(),
            extension_config: u32::try_from(recipe["extension_config"].as_u64().unwrap()).unwrap(),
            clip_gain_index: u32::try_from(recipe["clip_gain_index"].as_u64().unwrap()).unwrap(),
            sequence_count: u32::try_from(recipe["sequence_count"].as_u64().unwrap()).unwrap(),
            channel_limit: u32::try_from(recipe["channel_limit"].as_u64().unwrap()).unwrap(),
            descriptors,
            body,
            body_valid_bits: HuffmanBitCount::new(recipe["body_valid_bits"].as_u64().unwrap()),
        };
        let mut writer = Id14WriterState::new(initial_buffer);
        let outcome = write_id14_header(&input, &mut writer).unwrap();

        assert_eq!(
            outcome.return_code().value(),
            u8::try_from(case["return_code"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            writer.status_word(),
            u32::try_from(case["status_u32"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            writer.output_length().value(),
            case["bytes_written_or_length_after"]
        );
        if case["return_code"].as_u64().unwrap() == 0 {
            assert_eq!(
                outcome.valid_bit_count().value(),
                case["valid_bits_reconstructed"].as_u64().unwrap()
            );
            assert_eq!(
                outcome.padding_zero_bits(),
                u8::try_from(case["padding_zero_bits"].as_u64().unwrap()).unwrap()
            );
            let written =
                &writer.buffer()[..usize::try_from(writer.output_length().value()).unwrap()];
            assert_eq!(
                to_hex(written),
                case["independent_expected_hex"].as_str().unwrap()
            );
            assert_eq!(sha256(written), case["expected_sha256"]);
        } else {
            assert_eq!(
                to_hex(writer.buffer()),
                case["native_prefix_hex"].as_str().unwrap()
            );
        }
    }
}

fn descriptors_for(recipe: &Value) -> (Option<Vec<Id14ObjectDescriptor>>, bool) {
    if recipe["null_object_pointer"].as_bool().unwrap() {
        return (None, true);
    }
    let template = if recipe["descriptor_template"].is_object() {
        &recipe["descriptor_template"]
    } else {
        &recipe["object"]
    };
    let repeat_count = recipe["descriptor_template_repeat_count"]
        .as_u64()
        .unwrap_or(1);
    let descriptor = Id14ObjectDescriptor {
        present: match template["present"].as_u64().unwrap() {
            0 => Id14Presence::Absent,
            1 => Id14Presence::Present,
            2 => Id14Presence::ObservedNonBooleanTwo,
            value => panic!("unmapped observed presence value {value}"),
        },
        kind_code: u32::try_from(template["kind"].as_u64().unwrap()).unwrap(),
        mode_code: u32::try_from(template["mode"].as_u64().unwrap()).unwrap(),
        nested_mode: u32::try_from(template["nested_mode"].as_u64().unwrap()).unwrap(),
        nested_count: u32::try_from(template["nested_count"].as_u64().unwrap()).unwrap(),
        selected_index0: u32::try_from(template["index0"].as_u64().unwrap()).unwrap(),
        selected_index1: u32::try_from(template["index1"].as_u64().unwrap()).unwrap(),
    };
    (
        Some(vec![descriptor; usize::try_from(repeat_count).unwrap()]),
        false,
    )
}

fn decode_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = char::from(pair[0]).to_digit(16).unwrap();
            let low = char::from(pair[1]).to_digit(16).unwrap();
            u8::try_from((high << 4) | low).unwrap()
        })
        .collect()
}

fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    value
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
