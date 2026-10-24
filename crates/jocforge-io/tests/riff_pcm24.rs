// pattern: Functional Core
use jocforge_domain::timeline::AdmSampleIndex;
use jocforge_io::pcm::riff_pcm24::{ParsedRiffPcm24, RiffPcm24Error, parse_selected_riff_pcm24};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/riff_pcm24";
const SOURCE_PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const SOURCE_SPEC_SHA256: &str = "6bd78ebe23536c3a55ec54f2e13c2e8b98c2ee245ae5a886e47bc9580520b93a";
const UNIT_VECTORS_SHA256: &str =
    "a14dd6c3700fae1765399c72c816dd3146d57ab7dd026b375efb6aed7408cfa8";
const ADAPTER_CONTRACT_SHA256: &str =
    "9bce2e35ce47147c76c13e58ee976d44c5109938f0b19e1f3a81f1d8a61fa1ae";
const UNIT_README_SHA256: &str = "a455c6b33e582b0d359face09667e91622404fd203d376e446d615f9a52bbcce";

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

fn parse_hex(hex: &str) -> Result<Vec<u8>, RiffPcm24Error> {
    if hex.len() % 2 != 0 {
        return Err(RiffPcm24Error::InvalidInput);
    }
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = char::from(pair[0])
                .to_digit(16)
                .ok_or(RiffPcm24Error::InvalidInput)?;
            let low = char::from(pair[1])
                .to_digit(16)
                .ok_or(RiffPcm24Error::InvalidInput)?;
            u8::try_from(high * 16 + low).map_err(|_| RiffPcm24Error::InvalidInput)
        })
        .collect()
}

fn request_metadata_id(value: &Value) -> Result<Option<[u8; 4]>, RiffPcm24Error> {
    if value.is_null() {
        return Ok(None);
    }
    let text = value.as_str().ok_or(RiffPcm24Error::InvalidInput)?;
    let id: [u8; 4] = text
        .as_bytes()
        .try_into()
        .map_err(|_| RiffPcm24Error::InvalidInput)?;
    Ok(Some(id))
}

fn input_index(value: &Value) -> Result<u64, RiffPcm24Error> {
    value.as_u64().ok_or(RiffPcm24Error::InvalidInput)
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}

#[derive(Debug)]
struct AcceptedProjection {
    channels: u16,
    sample_rate_hz: u32,
    samples: u64,
    data_offset: u64,
    chunk_ids: Vec<String>,
    has_adm_chunks: bool,
    selected_metadata_hex: Option<String>,
    planar_u32: Vec<Vec<u32>>,
    metadata_payload_extents: Vec<(String, usize)>,
}

enum ActualProjection {
    Accepted(AcceptedProjection),
    Rejected(RiffPcm24Error),
}

fn read_hop(
    file: &ParsedRiffPcm24<'_>,
    input: &Value,
) -> Result<AcceptedProjection, RiffPcm24Error> {
    let first_sample = AdmSampleIndex::new(input_index(&input["first_sample"])?);
    let sample_count = input_index(&input["sample_count"])?;
    let planar_u32 = file.read_pcm24_hop(first_sample, sample_count)?;
    let metadata_payload_extents = file
        .metadata_payloads()
        .iter()
        .map(|metadata| (metadata.chunk_id().to_owned(), metadata.payload().len()))
        .collect();
    Ok(AcceptedProjection {
        channels: file.channels(),
        sample_rate_hz: file.sample_rate_hz(),
        samples: file.total_samples(),
        data_offset: file.data_offset(),
        chunk_ids: file.chunk_ids().to_vec(),
        has_adm_chunks: file.has_adm_chunks(),
        selected_metadata_hex: file.selected_metadata_payload().map(hex),
        planar_u32,
        metadata_payload_extents,
    })
}

fn calculate(input: &Value) -> ActualProjection {
    let riff_bytes = match parse_hex(input["riff_hex"].as_str().unwrap_or_default()) {
        Ok(bytes) => bytes,
        Err(error) => return ActualProjection::Rejected(error),
    };
    let requested_metadata = match request_metadata_id(&input["request_metadata"]) {
        Ok(value) => value,
        Err(error) => return ActualProjection::Rejected(error),
    };
    match parse_selected_riff_pcm24(&riff_bytes, requested_metadata) {
        Ok(file) => match read_hop(&file, input) {
            Ok(projection) => ActualProjection::Accepted(projection),
            Err(error) => ActualProjection::Rejected(error),
        },
        Err(error) => ActualProjection::Rejected(error),
    }
}

#[test]
fn riff_pcm24_cases_match_the_ten_selected_unit_vectors() {
    let vectors_bytes = fixture_bytes("unit-vectors.json");
    let vectors: Value = serde_json::from_slice(&vectors_bytes).unwrap();
    let provenance = fixture_json("PROVENANCE.json");
    assert_eq!(sha256(&vectors_bytes), UNIT_VECTORS_SHA256);
    assert_eq!(vectors["runtime_role"], "validation_only");
    assert_eq!(
        provenance["source_package_manifest_sha256"],
        SOURCE_PACKAGE_MANIFEST_SHA256
    );
    assert_eq!(provenance["source_spec_sha256"], SOURCE_SPEC_SHA256);
    assert_eq!(
        provenance["source_unit_vectors_sha256"],
        UNIT_VECTORS_SHA256
    );
    assert_eq!(
        provenance["source_unit_adapter_contract_sha256"],
        ADAPTER_CONTRACT_SHA256
    );
    assert_eq!(provenance["source_unit_readme_sha256"], UNIT_README_SHA256);

    let cases: Vec<_> = vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["entry_id"] == "riff-pcm24-read")
        .collect();
    assert_eq!(cases.len(), 10);

    // Construct every result from input fields before reading any expected projection.
    let actual: Vec<_> = cases.iter().map(|case| calculate(&case["input"])).collect();

    for (case, actual) in cases.iter().zip(actual) {
        let expected = &case["expected"];
        match actual {
            ActualProjection::Rejected(error) => {
                assert_eq!(
                    expected["status"], "REJECTED_MODEL_READER",
                    "{}",
                    case["id"]
                );
                assert_eq!(expected["error_class"], error.code(), "{}", case["id"]);
            }
            ActualProjection::Accepted(actual) => {
                assert_eq!(
                    expected["status"], "ACCEPTED_MODEL_READER",
                    "{}",
                    case["id"]
                );
                assert_eq!(
                    actual.channels,
                    expected["channels"].as_u64().unwrap() as u16
                );
                assert_eq!(
                    actual.sample_rate_hz,
                    expected["sample_rate"].as_u64().unwrap() as u32
                );
                assert_eq!(actual.samples, expected["samples"].as_u64().unwrap());
                assert_eq!(
                    actual.data_offset,
                    expected["data_offset"].as_u64().unwrap()
                );
                let chunk_ids: Vec<String> = expected["chunk_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|id| id.as_str().unwrap().to_owned())
                    .collect();
                assert_eq!(actual.chunk_ids, chunk_ids);
                assert_eq!(
                    actual.has_adm_chunks,
                    expected["has_adm_chunks"].as_bool().unwrap()
                );
                assert_eq!(
                    actual.selected_metadata_hex,
                    expected["selected_metadata_hex"]
                        .as_str()
                        .map(str::to_owned)
                );
                let planar: Vec<Vec<u32>> = expected["planar_u32"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|row| {
                        row.as_array()
                            .unwrap()
                            .iter()
                            .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
                            .collect()
                    })
                    .collect();
                assert_eq!(actual.planar_u32, planar, "{}", case["id"]);
                assert!(
                    !actual.metadata_payload_extents.is_empty(),
                    "{}",
                    case["id"]
                );
            }
        }
    }
}
