// pattern: Functional Core
use jocforge_domain::codec::grouped_quantizer::{
    MantissaBlockError, MantissaBlockItemInput, MantissaChannel, pack_block_mantissas,
};
use jocforge_domain::codec::mantissa::MantissaQuantizerError;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const PACKAGE_DIRECTORY: &str =
    "outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-QUANTIZER-MANTISSA-PRODUCTION-CONTRACT-V1";
const PACKAGE_MANIFEST_SHA256: &str =
    "fd819c3ec27e7bade55efda5e648792c9c33892a9be7a36bbfce89231b9e961f";

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn package_root() -> PathBuf {
    repository_root().join(PACKAGE_DIRECTORY)
}

fn read_json(name: &str) -> (Vec<u8>, Value) {
    let bytes = std::fs::read(package_root().join(name)).unwrap();
    let value = serde_json::from_slice(&bytes).unwrap();
    (bytes, value)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn approved_vectors() -> Value {
    let (manifest_bytes, manifest) = read_json("MANIFEST.json");
    assert_eq!(sha256(&manifest_bytes), PACKAGE_MANIFEST_SHA256);
    assert_eq!(
        manifest["package_id"],
        "CODEC-QUANTIZER-MANTISSA-PRODUCTION-CONTRACT-V1"
    );

    let (vector_bytes, vectors) = read_json("INPUT-ONLY-VECTORS.json");
    let vector_entry = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["file"] == "INPUT-ONLY-VECTORS.json")
        .unwrap();
    assert_eq!(
        vector_entry["bytes"].as_u64().unwrap(),
        u64::try_from(vector_bytes.len()).unwrap()
    );
    assert_eq!(
        vector_entry["sha256"].as_str().unwrap(),
        sha256(&vector_bytes)
    );
    vectors
}

fn channel(value: &Value) -> MantissaChannel {
    match value.as_u64().unwrap() {
        0 => MantissaChannel::Left,
        1 => MantissaChannel::Center,
        2 => MantissaChannel::Right,
        3 => MantissaChannel::LeftSurround,
        4 => MantissaChannel::RightSurround,
        5 => MantissaChannel::Lfe,
        other => panic!("unsupported input-vector channel {other}"),
    }
}

fn item(value: &Value) -> MantissaBlockItemInput {
    MantissaBlockItemInput {
        channel: channel(&value["channel"]),
        bin: u16::try_from(value["bin"].as_u64().unwrap()).unwrap(),
        coefficient_word: u32::from_str_radix(value["coefficient_f32_bits"].as_str().unwrap(), 16)
            .unwrap(),
        decoded_exponent: i16::try_from(value["decoded_exponent"].as_i64().unwrap()).unwrap(),
        bap: u8::try_from(value["bap"].as_u64().unwrap()).unwrap(),
    }
}

#[test]
fn approved_input_package_is_hash_bound_and_excludes_expected_codes() {
    let vectors = approved_vectors();
    assert_eq!(vectors["schema"], "eac3-quantizer-mantissa-input-only.v1");
    assert_eq!(vectors["input_only"], true);
    assert_eq!(vectors["expected_quantizer_codes_included"], false);
    assert_eq!(vectors["expected_group_codes_included"], false);
    assert_eq!(vectors["expected_au_body_included"], false);
    assert_eq!(vectors["pcm_included"], false);
    assert_eq!(vectors["private_key_included"], false);
}

#[test]
fn same_block_groups_cross_channels_and_packs_first_item_msb_first() {
    let vectors = approved_vectors();
    let case = vectors["grouping_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "same_block_cross_channel")
        .unwrap();
    let mut inputs = case["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(item)
        .collect::<Vec<_>>();
    inputs.reverse();

    let output = pack_block_mantissas(&inputs).unwrap();
    assert_eq!(
        output
            .items()
            .iter()
            .map(|observation| (observation.channel(), observation.bin()))
            .collect::<Vec<_>>(),
        [
            (MantissaChannel::Left, 0),
            (MantissaChannel::Left, 1),
            (MantissaChannel::Center, 0),
            (MantissaChannel::Center, 1),
            (MantissaChannel::Right, 0),
            (MantissaChannel::Right, 1),
            (MantissaChannel::LeftSurround, 0),
            (MantissaChannel::RightSurround, 0),
            (MantissaChannel::Lfe, 0),
        ]
    );
    assert_eq!(output.bit_count(), 19);
    assert_eq!(
        output
            .grouped_packs()
            .iter()
            .map(|pack| (pack.bap(), pack.value(), pack.width(), pack.start_bit()))
            .collect::<Vec<_>>(),
        [(1, 5, 5, 0), (2, 19, 7, 5), (4, 10, 7, 12)]
    );
    assert_eq!(output.packed_bytes(), [0x29, 0x31, 0x40]);
}

#[test]
fn every_block_starts_with_empty_groups_and_partial_groups_use_fillers() {
    let vectors = approved_vectors();
    let case = vectors["grouping_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "reset_between_blocks")
        .unwrap();
    let blocks = case["blocks"].as_array().unwrap();
    let first = pack_block_mantissas(
        &blocks[0]["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(item)
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let second = pack_block_mantissas(
        &blocks[1]["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(item)
            .collect::<Vec<_>>(),
    )
    .unwrap();

    assert_eq!(first.bit_count(), 5);
    assert_eq!(first.grouped_packs().len(), 1);
    assert_eq!(first.grouped_packs()[0].value(), 13);
    assert_eq!(first.grouped_packs()[0].start_bit(), 0);

    assert_eq!(second.bit_count(), 19);
    assert_eq!(
        second
            .grouped_packs()
            .iter()
            .map(|pack| (pack.bap(), pack.value(), pack.start_bit()))
            .collect::<Vec<_>>(),
        [(1, 13, 0), (2, 87, 5), (4, 71, 12)]
    );
}

#[test]
fn exact_full_domain_mantissa_cost_is_124_bits() {
    let vectors = approved_vectors();
    let domain = vectors["domain_sweep"].as_array().unwrap();
    let channels = [
        MantissaChannel::Left,
        MantissaChannel::Center,
        MantissaChannel::Right,
        MantissaChannel::LeftSurround,
        MantissaChannel::RightSurround,
        MantissaChannel::Lfe,
    ];
    let inputs = domain
        .iter()
        .enumerate()
        .map(|(index, value)| MantissaBlockItemInput {
            channel: channels[index % channels.len()],
            bin: u16::try_from(index / channels.len()).unwrap(),
            coefficient_word: u32::from_str_radix(
                value["coefficient_f32_bits"].as_str().unwrap(),
                16,
            )
            .unwrap(),
            decoded_exponent: i16::try_from(value["decoded_exponent"].as_i64().unwrap()).unwrap(),
            bap: u8::try_from(value["bap"].as_u64().unwrap()).unwrap(),
        })
        .collect::<Vec<_>>();

    let output = pack_block_mantissas(&inputs).unwrap();
    assert_eq!(output.bit_count(), 124);
    assert_eq!(output.packed_bytes().len(), 16);
    assert_eq!(output.grouped_packs().len(), 3);
}

#[test]
fn malformed_positions_and_quantizer_failures_return_block_errors() {
    let duplicate = MantissaBlockItemInput {
        channel: MantissaChannel::Left,
        bin: 0,
        coefficient_word: 0x3e80_0000,
        decoded_exponent: 0,
        bap: 1,
    };
    assert_eq!(
        pack_block_mantissas(&[duplicate, duplicate]),
        Err(MantissaBlockError::DuplicateItemPosition {
            channel: MantissaChannel::Left,
            bin: 0,
        })
    );

    let vectors = approved_vectors();
    let sentinel = vectors["boundary_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "bap3_sentinel")
        .unwrap();
    let sentinel_input = MantissaBlockItemInput {
        channel: MantissaChannel::Left,
        bin: 0,
        coefficient_word: u32::from_str_radix(
            sentinel["coefficient_f32_bits"].as_str().unwrap(),
            16,
        )
        .unwrap(),
        decoded_exponent: i16::try_from(sentinel["decoded_exponent"].as_i64().unwrap()).unwrap(),
        bap: u8::try_from(sentinel["bap"].as_u64().unwrap()).unwrap(),
    };
    assert_eq!(
        pack_block_mantissas(&[sentinel_input]),
        Err(MantissaBlockError::Quantizer(
            MantissaQuantizerError::ReservedCodeSentinel
        ))
    );

    let unsupported = MantissaBlockItemInput {
        bap: 16,
        ..duplicate
    };
    assert_eq!(
        pack_block_mantissas(&[unsupported]),
        Err(MantissaBlockError::Quantizer(
            MantissaQuantizerError::UnsupportedBap
        ))
    );
}

#[test]
fn empty_block_has_no_bits_or_persistent_group_state() {
    let output = pack_block_mantissas(&[]).unwrap();
    assert!(output.items().is_empty());
    assert!(output.grouped_packs().is_empty());
    assert!(output.packed_bytes().is_empty());
    assert_eq!(output.bit_count(), 0);
}
