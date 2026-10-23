// pattern: Functional Core
use jocforge_domain::joc::huffman::{
    HuffmanBitCount, HuffmanByteCapacity, HuffmanCodebookId, HuffmanCodebooks, HuffmanSymbolCount,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const SOURCE_MANIFEST_SHA256: &str =
    "d0a96531cdd3029fbed633aa9b6df83e26519014ab145365dd0fa41237495355";
const SOURCE_CHECK_SHA256: &str =
    "42f2384934451929f19e48cf9cd8b7ead4c69a89ad38d90e2f5adbed017d3c55";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn assert_source_asset(root: &Path, manifest: &Value, package_path: &str, repository_path: &str) {
    let source = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == package_path)
        .unwrap();
    let bytes = std::fs::read(root.join(repository_path)).unwrap();
    assert_eq!(source["bytes"].as_u64().unwrap(), bytes.len() as u64);
    assert_eq!(source["sha256"].as_str().unwrap(), sha256(&bytes));
}

#[test]
fn authorized_package_identity_and_materialized_assets_are_hash_bound() {
    let root = crate_root();
    let fixture_root = root.join("tests/fixtures/huffman");
    let manifest_bytes = std::fs::read(fixture_root.join("source-manifest.json")).unwrap();
    let independent_check: Value = serde_json::from_slice(
        &std::fs::read(fixture_root.join("independent-check.json")).unwrap(),
    )
    .unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();

    assert_eq!(sha256(&manifest_bytes), SOURCE_MANIFEST_SHA256);
    assert_eq!(
        sha256(&std::fs::read(fixture_root.join("independent-check.json")).unwrap()),
        SOURCE_CHECK_SHA256
    );
    assert_eq!(independent_check["manifest_sha256"], SOURCE_MANIFEST_SHA256);
    assert_eq!(independent_check["payload_count"], 8);
    assert!(
        independent_check["all_payload_hashes_match"]
            .as_bool()
            .unwrap()
    );
    assert!(
        independent_check["user_authorization_recorded"]
            .as_bool()
            .unwrap()
    );
    assert!(
        independent_check["license_clearance_unknown"]
            .as_bool()
            .unwrap()
    );
    assert!(
        independent_check["native_runtime_not_run_preserved"]
            .as_bool()
            .unwrap()
    );
    assert!(
        manifest["user_authorized_public_repo_use"]
            .as_bool()
            .unwrap()
    );
    assert_eq!(manifest["license_clearance"], "UNKNOWN");
    assert_eq!(manifest["production_usable"], "F05QH_TABLE_SCOPE_ONLY");
    assert_eq!(manifest["implementation_ready"], "NO");

    assert_source_asset(
        &root,
        &manifest,
        "parameters/joc-huffman-v1.json",
        "src/joc/huffman/codebooks.json",
    );
    assert_source_asset(
        &root,
        &manifest,
        "vectors/JOC-HUFFMAN-INPUT-VECTORS.json",
        "tests/fixtures/huffman/input-vectors.json",
    );
    assert_source_asset(
        &root,
        &manifest,
        "vectors/JOC-HUFFMAN-VERIFICATION-RESULT.json",
        "tests/fixtures/huffman/verification-result.json",
    );
    assert_source_asset(
        &root,
        &manifest,
        "vectors/JOC-HUFFMAN-CONSUMER-CONNECTION-VERIFICATION.json",
        "tests/fixtures/huffman/consumer-connection-vectors.json",
    );
    assert_source_asset(
        &root,
        &manifest,
        "spec/05-joc-huffman-codebooks.md",
        "../../docs/implementation/huffman-codebooks.md",
    );
    assert_source_asset(
        &root,
        &manifest,
        "PROVENANCE.json",
        "../../docs/implementation/huffman-codebooks-provenance.json",
    );
    assert_source_asset(
        &root,
        &manifest,
        "README.md",
        "tests/fixtures/huffman/README.md",
    );
}

#[test]
fn washed_stream_vectors_match_valid_bits_and_padded_byte_identity() {
    let codebooks = HuffmanCodebooks::selected().unwrap();
    let fixture_root = crate_root().join("tests/fixtures/huffman");
    let inputs: Value =
        serde_json::from_slice(&std::fs::read(fixture_root.join("input-vectors.json")).unwrap())
            .unwrap();
    let results: Value = serde_json::from_slice(
        &std::fs::read(fixture_root.join("verification-result.json")).unwrap(),
    )
    .unwrap();

    for case in inputs["stream_cases"].as_array().unwrap() {
        let case_id = case["case_id"].as_str().unwrap();
        let table_id = codebook_id(case["codebook"].as_str().unwrap());
        let symbols = case["symbols"]
            .as_array()
            .unwrap()
            .iter()
            .map(|symbol| symbol.as_i64().unwrap())
            .collect::<Vec<_>>();
        let expected = results["stream_results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|result| result["case_id"] == case_id)
            .unwrap();
        let encoded = codebooks.table(table_id).encode_symbols(&symbols).unwrap();

        assert_eq!(
            encoded.valid_bit_count().value(),
            expected["valid_bit_count"].as_u64().unwrap()
        );
        assert_eq!(encoded.padding_zero_bits(), expected["padding_zero_bits"]);
        assert_eq!(encoded.bytes().len(), expected["padded_byte_count"]);
        assert_eq!(sha256(encoded.bytes()), expected["msb_padded_bytes_sha256"]);
        assert_eq!(
            codebooks
                .table(table_id)
                .decode_symbols(
                    encoded.bytes(),
                    encoded.valid_bit_count(),
                    HuffmanSymbolCount::new(u16::try_from(symbols.len()).unwrap()),
                )
                .unwrap(),
            symbols
                .iter()
                .map(|symbol| u16::try_from(*symbol).unwrap())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn invalid_symbols_and_truncated_words_use_the_washed_error_outcomes() {
    let codebooks = HuffmanCodebooks::selected().unwrap();
    let fixture_root = crate_root().join("tests/fixtures/huffman");
    let inputs: Value =
        serde_json::from_slice(&std::fs::read(fixture_root.join("input-vectors.json")).unwrap())
            .unwrap();

    for case in inputs["invalid_symbol_cases"].as_array().unwrap() {
        let table = codebooks.table(codebook_id(case["codebook"].as_str().unwrap()));
        let symbol = case["symbol"].as_i64();
        if let Some(symbol) = symbol {
            let error = table.encode_symbols(&[symbol]).unwrap_err();
            assert_eq!(error.code(), "HUFFMAN_SYMBOL_OUT_OF_RANGE");
        } else {
            assert!(case["symbol"].is_boolean());
            // Boolean is not representable by the typed integer API.
            assert!(case["symbol"].as_i64().is_none());
        }
    }

    for case in inputs["truncation_cases"].as_array().unwrap() {
        let table = codebooks.table(codebook_id(case["codebook"].as_str().unwrap()));
        let symbol = case["symbol"].as_i64().unwrap();
        let drop_tail_bits = case["drop_tail_bits"].as_u64().unwrap() as usize;
        let encoded = table.encode_symbols(&[symbol]).unwrap();
        let available_bits =
            encoded.valid_bit_count().value() - u64::try_from(drop_tail_bits).unwrap();
        let mut reader = jocforge_domain::joc::huffman::HuffmanBitReader::new(
            encoded.bytes(),
            HuffmanBitCount::new(available_bits),
        );
        let cursor_before = reader.bit_cursor();
        let error = reader.read_symbol(table).unwrap_err();
        assert_eq!(error.code(), "HUFFMAN_TRUNCATED_CODEWORD");
        assert_eq!(reader.bit_cursor(), cursor_before);
    }
}

#[test]
fn f05q_capacity_vectors_check_scratch_before_final_bytes() {
    let codebooks = HuffmanCodebooks::selected().unwrap();
    let fixture_root = crate_root().join("tests/fixtures/huffman");
    let inputs: Value =
        serde_json::from_slice(&std::fs::read(fixture_root.join("input-vectors.json")).unwrap())
            .unwrap();
    let results: Value = serde_json::from_slice(
        &std::fs::read(fixture_root.join("verification-result.json")).unwrap(),
    )
    .unwrap();

    for case in inputs["capacity_cases"].as_array().unwrap() {
        let table_id = codebook_id(case["codebook"].as_str().unwrap());
        let symbols = case["symbols"]
            .as_array()
            .unwrap()
            .iter()
            .map(|symbol| symbol.as_i64().unwrap())
            .collect::<Vec<_>>();
        let encoded = codebooks.table(table_id).encode_symbols(&symbols).unwrap();
        let expected = results["capacity_results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|result| result["case_id"] == case["case_id"])
            .unwrap();
        let header_bits = case["header_bits"].as_u64().unwrap() as usize;
        let scratch_limit = case["scratch_limit_bits"].as_u64().unwrap() as usize;
        let capacity_bytes = case["capacity_bytes"].as_u64().unwrap() as usize;

        assert_eq!(
            encoded.valid_bit_count().value(),
            expected["coefficient_bits"].as_u64().unwrap()
        );
        let result = jocforge_domain::joc::huffman::check_f05q_capacity(
            encoded.valid_bit_count(),
            HuffmanBitCount::new(u64::try_from(header_bits).unwrap()),
            HuffmanBitCount::new(u64::try_from(scratch_limit).unwrap()),
            HuffmanByteCapacity::new(u64::try_from(capacity_bytes).unwrap()),
        );
        if expected["fits_both"].as_bool().unwrap() {
            assert!(result.is_ok());
        } else if expected["fits_coefficient_scratch"].as_bool().unwrap() {
            assert_eq!(result.unwrap_err().code(), "JOC_PAYLOAD_CAPACITY_EXCEEDED");
        } else {
            assert_eq!(
                result.unwrap_err().code(),
                "JOC_COEFFICIENT_SCRATCH_EXCEEDED"
            );
        }
    }
}

#[test]
fn writer_commits_a_symbol_batch_only_after_validation_and_capacity_checks() {
    let codebooks = HuffmanCodebooks::selected().unwrap();
    let table = codebooks.table(HuffmanCodebookId::FullCoarse);
    let mut writer = jocforge_domain::joc::huffman::HuffmanBitWriter::default();
    writer.write_symbols(table, &[0]).unwrap();
    let before = writer.clone();

    let error = writer.write_symbols(table, &[1, 96]).unwrap_err();
    assert_eq!(error.code(), "HUFFMAN_SYMBOL_OUT_OF_RANGE");
    assert_eq!(writer, before);

    let fixture_root = crate_root().join("tests/fixtures/huffman");
    let inputs: Value =
        serde_json::from_slice(&std::fs::read(fixture_root.join("input-vectors.json")).unwrap())
            .unwrap();
    let results: Value = serde_json::from_slice(
        &std::fs::read(fixture_root.join("verification-result.json")).unwrap(),
    )
    .unwrap();

    for case in inputs["capacity_cases"].as_array().unwrap() {
        let expected = results["capacity_results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|result| result["case_id"] == case["case_id"])
            .unwrap();
        if expected["fits_both"].as_bool().unwrap() {
            continue;
        }
        let table = codebooks.table(codebook_id(case["codebook"].as_str().unwrap()));
        let symbols = case["symbols"]
            .as_array()
            .unwrap()
            .iter()
            .map(|symbol| symbol.as_i64().unwrap())
            .collect::<Vec<_>>();
        let header_bits = case["header_bits"].as_u64().unwrap() as usize;
        let scratch_limit = case["scratch_limit_bits"].as_u64().unwrap() as usize;
        let capacity_bytes = case["capacity_bytes"].as_u64().unwrap() as usize;
        let before = writer.clone();
        let error = writer
            .write_symbols_with_capacity(
                table,
                &symbols,
                HuffmanBitCount::new(u64::try_from(header_bits).unwrap()),
                HuffmanBitCount::new(u64::try_from(scratch_limit).unwrap()),
                HuffmanByteCapacity::new(u64::try_from(capacity_bytes).unwrap()),
            )
            .unwrap_err();

        if expected["fits_coefficient_scratch"].as_bool().unwrap() {
            assert_eq!(error.code(), "JOC_PAYLOAD_CAPACITY_EXCEEDED");
        } else {
            assert_eq!(error.code(), "JOC_COEFFICIENT_SCRATCH_EXCEEDED");
        }
        assert_eq!(writer, before);
    }
}

#[test]
fn f05q_consumer_vectors_select_tables_reset_previous_and_write_exact_bits() {
    let codebooks = HuffmanCodebooks::selected().unwrap();
    let fixture_root = crate_root().join("tests/fixtures/huffman");
    let vectors: Value = serde_json::from_slice(
        &std::fs::read(fixture_root.join("consumer-connection-vectors.json")).unwrap(),
    )
    .unwrap();

    for case in vectors["success_cases"].as_array().unwrap() {
        let input = f05q_input(&case["input"]);
        let expected = &case["expected"];
        let output = codebooks.encode_f05q_consumer(&input).unwrap();

        assert_eq!(
            output.coefficient_codebook().name(),
            expected["selected_coefficient_codebook"].as_str().unwrap()
        );
        assert_eq!(
            output.position_codebook().map(HuffmanCodebookId::name),
            expected["selected_position_codebook"].as_str()
        );
        assert_eq!(
            output.steps().value(),
            u16::try_from(expected["steps"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(output.initial_previous(), expected["initial_previous"]);
        assert_eq!(
            output.reset_count(),
            u64::try_from(expected["reset_events"].as_array().unwrap().len()).unwrap()
        );
        assert_eq!(
            output.write_count(),
            u64::try_from(expected["write_events"].as_array().unwrap().len()).unwrap()
        );
        let mut coefficient_bits = 0;
        let mut position_bits = 0;
        for event in expected["write_events"].as_array().unwrap() {
            let bit_length = event["bit_length"].as_u64().unwrap() as usize;
            if event["event_kind"] == "COEFFICIENT_HUFFMAN" {
                coefficient_bits += bit_length;
            } else {
                position_bits += bit_length;
            }
        }
        assert_eq!(
            output.coefficient_bit_count().value(),
            u64::try_from(coefficient_bits).unwrap()
        );
        assert_eq!(
            output.position_bit_count().value(),
            u64::try_from(position_bits).unwrap()
        );
        assert_eq!(
            output.bitstream().valid_bit_count().value(),
            expected["valid_bit_count"].as_u64().unwrap()
        );
        assert_eq!(
            output.bitstream().padding_zero_bits(),
            expected["padding_zero_bits"]
        );
        assert_eq!(
            to_hex(output.bitstream().bytes()),
            expected["padded_bytes_hex"].as_str().unwrap()
        );
        assert_eq!(
            sha256(output.bitstream().bytes()),
            expected["padded_bytes_sha256"]
        );
    }
}

#[test]
fn every_authorized_codeword_roundtrips_without_reordering() {
    let codebooks = HuffmanCodebooks::selected().unwrap();
    let tables = [
        (HuffmanCodebookId::FullCoarse, 96),
        (HuffmanCodebookId::FullFine, 192),
        (HuffmanCodebookId::SparseCoarse, 96),
        (HuffmanCodebookId::SparseFine, 192),
        (HuffmanCodebookId::Position5, 5),
        (HuffmanCodebookId::Position7, 7),
    ];

    for (table_id, symbol_count) in tables {
        let table = codebooks.table(table_id);
        assert_eq!(
            table.symbol_count().value(),
            u16::try_from(symbol_count).unwrap()
        );

        for symbol in 0..symbol_count {
            let encoded = table.encode_symbols(&[i64::from(symbol)]).unwrap();
            let decoded = table
                .decode_symbols(
                    encoded.bytes(),
                    encoded.valid_bit_count(),
                    HuffmanSymbolCount::new(1),
                )
                .unwrap();
            assert_eq!(decoded, [u16::try_from(symbol).unwrap()]);
        }
    }
}

fn codebook_id(name: &str) -> HuffmanCodebookId {
    match name {
        "full_coarse" => HuffmanCodebookId::FullCoarse,
        "full_fine" => HuffmanCodebookId::FullFine,
        "sparse_coarse" => HuffmanCodebookId::SparseCoarse,
        "sparse_fine" => HuffmanCodebookId::SparseFine,
        "position_5" => HuffmanCodebookId::Position5,
        "position_7" => HuffmanCodebookId::Position7,
        _ => panic!("unexpected codebook in hash-bound vector"),
    }
}

fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    value
}

fn f05q_input(input: &Value) -> jocforge_domain::joc::huffman::F05QConsumerInput {
    use jocforge_domain::joc::huffman::{F05QConsumerInput, F05QConsumerPointInput};

    let mode = input["mode"].as_u64().unwrap() as u8;
    let channel_count = input["channel_count"].as_u64().unwrap() as u8;
    let points = input["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|point| {
            if let Some(rows) = point["quantized_rows"].as_array() {
                F05QConsumerPointInput::Full {
                    quantized_rows: rows
                        .iter()
                        .map(|row| {
                            row.as_array()
                                .unwrap()
                                .iter()
                                .map(|value| value.as_i64().unwrap())
                                .collect()
                        })
                        .collect(),
                }
            } else {
                F05QConsumerPointInput::Sparse {
                    selected_channels: point["selected_channels"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|channel| channel.as_i64().unwrap())
                        .collect(),
                    quantized_coefficients: point["quantized_coefficients"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|value| value.as_i64().unwrap())
                        .collect(),
                }
            }
        })
        .collect();

    F05QConsumerInput {
        mode,
        channel_count,
        points,
    }
}
