// pattern: Functional Core
use jocforge_domain::timeline::AdmSampleIndex;
use jocforge_io::pcm::riff_pcm24::{
    ParsedRiffPcm24, RiffPcm24Error, decode_selected_pcm24_read_chunk, parse_selected_riff_pcm24,
};
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
const F06OF_INPUT_ONLY_VECTORS_PATH: &str = "../../outputs/IMPLEMENTER_SUPPLEMENTS/F06OF-SELECTED-PCM-INPUT-READER-STATIC-CONTRACT-V1/INPUT-ONLY-VECTORS.json";
const F06OF_INPUT_ONLY_VECTORS_SHA256: &str =
    "16958ad2622e17d6667e825d430c2c6e5edd39dc61e148913ff1de252e39c976";

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

#[test]
fn selected_adm_payloads_are_unique_borrowed_spans_from_one_riff() {
    let mut bytes = b"RIFF\0\0\0\0WAVE".to_vec();
    append_chunk(&mut bytes, *b"fmt ", &selected_format_payload());
    append_chunk(&mut bytes, *b"data", &[0_u8; 114]);
    append_chunk(&mut bytes, *b"axml", b"<a/>");
    append_chunk(&mut bytes, *b"chna", &[1, 2, 3, 4]);
    append_chunk(&mut bytes, *b"dbmd", &[5, 6, 7]);
    let riff_size = u32::try_from(bytes.len() - 8).unwrap();
    bytes[4..8].copy_from_slice(&riff_size.to_le_bytes());

    let parsed = parse_selected_riff_pcm24(&bytes, None).unwrap();
    let payloads = parsed.selected_adm_payloads().unwrap();
    assert_eq!(payloads.axml(), b"<a/>");
    assert_eq!(payloads.chna(), &[1, 2, 3, 4]);
    assert_eq!(payloads.dbmd(), &[5, 6, 7]);
    assert!(is_subslice(&bytes, payloads.axml()));
    assert!(is_subslice(&bytes, payloads.chna()));
    assert!(is_subslice(&bytes, payloads.dbmd()));
}

#[test]
fn pcm24_read_chunks_return_complete_frames_from_washed_input_vectors() {
    let vector_bytes = std::fs::read(crate_root().join(F06OF_INPUT_ONLY_VECTORS_PATH)).unwrap();
    assert_eq!(sha256(&vector_bytes), F06OF_INPUT_ONLY_VECTORS_SHA256);
    let vectors: Value = serde_json::from_slice(&vector_bytes).unwrap();
    assert_eq!(
        vectors["grade"],
        "MODEL_ARITHMETIC_ONLY; NO_PCM_ACCESS_OR_NATIVE_CALLBACK_OBSERVATIONS"
    );

    let mut checked_cases = 0;
    for case in vectors["vectors"].as_array().unwrap() {
        let input = &case["input"];
        if input["channels"].as_u64() != Some(38) {
            continue;
        }
        let expected = &case["expected"];
        let block_align = expected["block_align_bytes"].as_u64().unwrap();
        let requested_bytes =
            u32::try_from(input["hop_frames"].as_u64().unwrap() * block_align).unwrap();
        let read_bytes =
            vec![0; usize::try_from(expected["last_actual_bytes"].as_u64().unwrap()).unwrap()];
        let decoded = decode_selected_pcm24_read_chunk(&read_bytes, requested_bytes).unwrap();

        assert_eq!(
            decoded.actual_frames().value(),
            expected["last_actual_frames"].as_u64().unwrap(),
            "{}",
            case["id"]
        );
        assert_eq!(decoded.planar_f32_bits().len(), 38);
        assert!(
            decoded
                .planar_f32_bits()
                .iter()
                .all(|channel| channel.len() == decoded.actual_frames().value() as usize)
        );
        checked_cases += 1;
    }
    assert_eq!(checked_cases, 3);
}

#[test]
fn pcm24_read_chunk_ignores_incomplete_tail_and_preserves_channel_order() {
    let requested_bytes = 2 * 114;
    let mut read_bytes = vec![0xff; 114 + 113];
    read_bytes[..3].copy_from_slice(&[0xff, 0xff, 0x7f]);
    read_bytes[3..6].copy_from_slice(&[0x00, 0x00, 0x80]);
    read_bytes[6..9].copy_from_slice(&[0xff, 0xff, 0xff]);
    read_bytes[9..12].copy_from_slice(&[0x00, 0x00, 0x00]);

    let decoded = decode_selected_pcm24_read_chunk(&read_bytes, requested_bytes).unwrap();

    assert_eq!(decoded.actual_frames().value(), 1);
    assert_eq!(decoded.planar_f32_bits().len(), 38);
    assert!(decoded.planar_f32_bits().iter().all(|row| row.len() == 1));
    assert_eq!(decoded.planar_f32_bits()[0][0], 0x3f7f_fffe);
    assert_eq!(decoded.planar_f32_bits()[1][0], 0xbf80_0000);
    assert_eq!(decoded.planar_f32_bits()[2][0], 0xb400_0000);
    assert_eq!(decoded.planar_f32_bits()[3][0], 0x0000_0000);
}

#[test]
fn selected_adm_payloads_reject_missing_or_duplicate_metadata_chunks() {
    let missing_dbmd = selected_riff_with_metadata(false, false);
    assert_eq!(
        parse_selected_riff_pcm24(&missing_dbmd, None)
            .unwrap()
            .selected_adm_payloads()
            .unwrap_err(),
        RiffPcm24Error::InvalidInput
    );

    let duplicate_axml = selected_riff_with_metadata(true, true);
    assert_eq!(
        parse_selected_riff_pcm24(&duplicate_axml, None)
            .unwrap()
            .selected_adm_payloads()
            .unwrap_err(),
        RiffPcm24Error::InvalidInput
    );
}

fn selected_riff_with_metadata(include_dbmd: bool, duplicate_axml: bool) -> Vec<u8> {
    let mut bytes = b"RIFF\0\0\0\0WAVE".to_vec();
    append_chunk(&mut bytes, *b"fmt ", &selected_format_payload());
    append_chunk(&mut bytes, *b"data", &[0_u8; 114]);
    append_chunk(&mut bytes, *b"axml", b"<a/>");
    if duplicate_axml {
        append_chunk(&mut bytes, *b"axml", b"<b/>");
    }
    append_chunk(&mut bytes, *b"chna", &[1, 2, 3, 4]);
    if include_dbmd {
        append_chunk(&mut bytes, *b"dbmd", &[5, 6, 7]);
    }
    let riff_size = u32::try_from(bytes.len() - 8).unwrap();
    bytes[4..8].copy_from_slice(&riff_size.to_le_bytes());
    bytes
}

fn selected_format_payload() -> Vec<u8> {
    let mut format = Vec::with_capacity(16);
    format.extend_from_slice(&1_u16.to_le_bytes());
    format.extend_from_slice(&38_u16.to_le_bytes());
    format.extend_from_slice(&48_000_u32.to_le_bytes());
    format.extend_from_slice(&5_472_000_u32.to_le_bytes());
    format.extend_from_slice(&114_u16.to_le_bytes());
    format.extend_from_slice(&24_u16.to_le_bytes());
    format
}

fn append_chunk(bytes: &mut Vec<u8>, chunk_id: [u8; 4], payload: &[u8]) {
    bytes.extend_from_slice(&chunk_id);
    bytes.extend_from_slice(&u32::try_from(payload.len()).unwrap().to_le_bytes());
    bytes.extend_from_slice(payload);
    if payload.len() % 2 != 0 {
        bytes.push(0);
    }
}

fn is_subslice(resource: &[u8], payload: &[u8]) -> bool {
    let start = (payload.as_ptr() as usize).checked_sub(resource.as_ptr() as usize);
    start
        .and_then(|start| start.checked_add(payload.len()).map(|end| (start, end)))
        .is_some_and(|(start, end)| resource.get(start..end) == Some(payload))
}
