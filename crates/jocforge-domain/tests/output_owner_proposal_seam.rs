// pattern: Functional Core + Imperative Shell; test-only, no production API
use jocforge_domain::integrity::crc16::eac3_crc16;
use jocforge_domain::timeline::{CodecSourceIndex, JocDispatchIndex, PublishedAuIndex};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const FIXTURE_MANIFEST_SHA256: &str =
    "d8d97f07d82d07dfa05772bb60bc90c6cab46e908f04366f24cc1398a16d169f";
const PROVIDER_FILE: &str = "synthetic-output-provider.json";
const VECTORS_FILE: &str = "OUTPUT-OWNER-VECTORS.json";
const OUTPUT_FRAME_BYTES: usize = 4096;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/output_owner")
}

fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn digest_hex(bytes: &[u8]) -> String {
    hex(&digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push(char::from(DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    text
}

fn decode_hex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0, "hex strings have an even byte count");
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).expect("hex high nibble");
            let low = (pair[1] as char).to_digit(16).expect("hex low nibble");
            ((high << 4) | low) as u8
        })
        .collect()
}

fn canonical_json_sha256(value: &Value) -> String {
    digest_hex(&serde_json::to_vec(value).expect("JSON value serializes"))
}

fn assert_fixture_manifest() {
    let root = fixture_root();
    let manifest_bytes =
        fs::read(root.join("OUTPUT-OWNER-MANIFEST.json")).expect("fixture manifest exists");
    assert_eq!(digest_hex(&manifest_bytes), FIXTURE_MANIFEST_SHA256);
    let manifest: Value = serde_json::from_slice(&manifest_bytes).expect("manifest JSON");
    let entries = manifest["files"].as_array().expect("manifest files array");
    assert_eq!(entries.len(), 7);

    for entry in entries {
        let name = entry["file"].as_str().expect("manifest file name");
        let bytes = fs::read(root.join(name)).expect("manifested fixture exists");
        assert_eq!(
            bytes.len() as u64,
            entry["bytes"].as_u64().expect("manifest byte count")
        );
        assert_eq!(
            digest_hex(&bytes),
            entry["sha256"].as_str().expect("manifest SHA-256")
        );
    }

    let independent_check: Value = serde_json::from_slice(
        &fs::read(root.join("INDEPENDENT-CHECK.json")).expect("independent check exists"),
    )
    .expect("independent check JSON");
    let checks = independent_check["checks"]
        .as_array()
        .expect("independent checks array");
    assert_eq!(checks.len(), 26);
    assert!(checks.iter().all(|check| check["pass"] == true));
    assert_eq!(independent_check["all_pass"], true);
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LedgerEntry {
    au_index: PublishedAuIndex,
    offset: u64,
    length: u32,
    frame_sha256: [u8; 32],
    crc_residual: u16,
}

impl LedgerEntry {
    fn to_json(&self) -> Value {
        json!({
            "au_index": self.au_index.value(),
            "offset": self.offset,
            "length": self.length,
            "frame_sha256": hex(&self.frame_sha256),
            "crc_residual": self.crc_residual,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OutputState {
    profile_id: String,
    provider_sha256: [u8; 32],
    attempt_id: String,
    staging_name: String,
    generation: u64,
    next_source_index: CodecSourceIndex,
    published_au_count: u64,
    durable_length: u64,
    ledger: Vec<LedgerEntry>,
    finalized: bool,
    product_name: String,
    final_product_sha256: Option<[u8; 32]>,
}

impl OutputState {
    fn from_input(value: &Value) -> Self {
        let ledger = value["ledger"]
            .as_array()
            .expect("state ledger array")
            .iter()
            .map(|entry| LedgerEntry {
                au_index: PublishedAuIndex::new(
                    entry["au_index"].as_u64().expect("ledger AU index"),
                ),
                offset: entry["offset"].as_u64().expect("ledger offset"),
                length: entry["length"].as_u64().expect("ledger length") as u32,
                frame_sha256: decode_sha256(
                    entry["frame_sha256"]
                        .as_str()
                        .expect("ledger frame SHA-256"),
                ),
                crc_residual: entry["crc_residual"].as_u64().expect("ledger CRC") as u16,
            })
            .collect();
        let final_product_sha256 = value["final_product_sha256"].as_str().map(decode_sha256);

        Self {
            profile_id: value["profile_id"].as_str().expect("profile ID").to_owned(),
            provider_sha256: decode_sha256(
                value["provider_sha256"].as_str().expect("provider SHA-256"),
            ),
            attempt_id: value["attempt_id"].as_str().expect("attempt ID").to_owned(),
            staging_name: value["staging_name"]
                .as_str()
                .expect("staging name")
                .to_owned(),
            generation: value["generation"].as_u64().expect("generation"),
            next_source_index: CodecSourceIndex::new(
                value["next_source_index"]
                    .as_u64()
                    .expect("next source index"),
            ),
            published_au_count: value["published_au_count"]
                .as_u64()
                .expect("published AU count"),
            durable_length: value["durable_length"].as_u64().expect("durable length"),
            ledger,
            finalized: value["finalized"].as_bool().expect("finalized flag"),
            product_name: value["product_name"]
                .as_str()
                .expect("product name")
                .to_owned(),
            final_product_sha256,
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "profile_id": self.profile_id,
            "provider_sha256": hex(&self.provider_sha256),
            "attempt_id": self.attempt_id,
            "staging_name": self.staging_name,
            "generation": self.generation,
            "next_source_index": self.next_source_index.value(),
            "published_au_count": self.published_au_count,
            "durable_length": self.durable_length,
            "ledger": self.ledger.iter().map(LedgerEntry::to_json).collect::<Vec<_>>(),
            "finalized": self.finalized,
            "product_name": self.product_name,
            "final_product_sha256": self.final_product_sha256.map(|sha| hex(&sha)),
        })
    }

    fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = b"OutputStateV1\0".to_vec();
        push_string_u16(&mut bytes, &self.profile_id);
        push_string_u16(&mut bytes, &self.attempt_id);
        push_string_u16(&mut bytes, &self.staging_name);
        bytes.extend_from_slice(&self.provider_sha256);
        bytes.extend_from_slice(&self.generation.to_le_bytes());
        bytes.extend_from_slice(&self.next_source_index.value().to_le_bytes());
        bytes.extend_from_slice(&self.published_au_count.to_le_bytes());
        bytes.extend_from_slice(&self.durable_length.to_le_bytes());
        bytes.extend_from_slice(
            &u32::try_from(self.ledger.len())
                .expect("ledger count fits U32")
                .to_le_bytes(),
        );
        for entry in &self.ledger {
            bytes.extend_from_slice(&entry.au_index.value().to_le_bytes());
            bytes.extend_from_slice(&entry.offset.to_le_bytes());
            bytes.extend_from_slice(&entry.length.to_le_bytes());
            bytes.extend_from_slice(&entry.frame_sha256);
            bytes.extend_from_slice(&entry.crc_residual.to_le_bytes());
        }
        bytes.push(u8::from(self.finalized));
        push_string_u16(&mut bytes, &self.product_name);
        match self.final_product_sha256 {
            Some(sha256) => {
                bytes.push(1);
                bytes.extend_from_slice(&sha256);
            }
            None => bytes.push(0),
        }
        bytes
    }

    fn sha256(&self) -> String {
        digest_hex(&self.canonical_bytes())
    }
}

fn push_string_u16(bytes: &mut Vec<u8>, value: &str) {
    let encoded = value.as_bytes();
    bytes.extend_from_slice(
        &u16::try_from(encoded.len())
            .expect("contract strings fit U16")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(encoded);
}

fn decode_sha256(text: &str) -> [u8; 32] {
    let bytes = decode_hex(text);
    bytes.try_into().expect("SHA-256 has 32 bytes")
}

#[derive(Clone, Debug)]
struct OutputProvider {
    profile_id: String,
    provider_sha256: [u8; 32],
    frame_bytes: usize,
    suppressed_sources: u64,
    expected_sources: u64,
    expected_published_aus: u64,
    expected_product_bytes: u64,
    intent_name_template: String,
}

impl OutputProvider {
    fn from_input(value: &Value) -> Self {
        Self {
            profile_id: value["profile_id"].as_str().expect("profile ID").to_owned(),
            provider_sha256: digest(&serde_json::to_vec(value).expect("provider JSON")),
            frame_bytes: value["frame_bytes"].as_u64().expect("frame byte count") as usize,
            suppressed_sources: value["suppressed_sources"]
                .as_u64()
                .expect("suppressed source count"),
            expected_sources: value["expected_sources"].as_u64().expect("source count"),
            expected_published_aus: value["expected_published_aus"]
                .as_u64()
                .expect("published AU count"),
            expected_product_bytes: value["expected_product_bytes"]
                .as_u64()
                .expect("product byte count"),
            intent_name_template: value["intent_name_template"]
                .as_str()
                .expect("intent name template")
                .to_owned(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SourceIdentity {
    source_index: CodecSourceIndex,
    dispatch: JocDispatchIndex,
    au_index: Option<PublishedAuIndex>,
}

impl SourceIdentity {
    fn from_input(value: &Value) -> Self {
        Self {
            source_index: CodecSourceIndex::new(
                value["source_index"].as_u64().expect("source index"),
            ),
            dispatch: JocDispatchIndex::new(value["dispatch"].as_u64().expect("dispatch")),
            au_index: value["au_index"].as_u64().map(PublishedAuIndex::new),
        }
    }
}

#[derive(Clone, Debug)]
struct AppendProposal {
    value: Value,
    after: OutputState,
    frame: Vec<u8>,
}

#[derive(Clone, Debug)]
enum PreparedSource {
    Suppressed { receipt: Value, after: OutputState },
    Append(AppendProposal),
}

impl PreparedSource {
    fn kind(&self) -> &'static str {
        match self {
            Self::Suppressed { .. } => "SUPPRESSED",
            Self::Append(_) => "APPEND",
        }
    }

    fn prepared_json(&self) -> Value {
        match self {
            Self::Suppressed { receipt, .. } => json!({
                "status": "PASS",
                "kind": "SUPPRESSED",
                "proposal": null,
                "receipt": receipt,
                "proposed_state": null,
            }),
            Self::Append(proposal) => json!({
                "status": "PASS",
                "kind": "APPEND",
                "proposal": proposal.value,
                "receipt": null,
                "proposed_state": null,
            }),
        }
    }
}

fn prepare_source(
    state: &OutputState,
    provider: &OutputProvider,
    identity: SourceIdentity,
    frame: &[u8],
    declared_sha256: &str,
) -> Result<PreparedSource, &'static str> {
    if state.profile_id != provider.profile_id
        || state.provider_sha256 != provider.provider_sha256
        || state.finalized
    {
        return Err("OUTPUT_IDENTITY");
    }

    let source = identity.source_index.value();
    let expected_dispatch = source.checked_add(1).ok_or("OUTPUT_SOURCE_CLOCK")?;
    let expected_au = if source < provider.suppressed_sources {
        None
    } else {
        Some(PublishedAuIndex::new(
            source
                .checked_sub(provider.suppressed_sources)
                .ok_or("OUTPUT_AU_CLOCK")?,
        ))
    };
    if source != state.next_source_index.value()
        || identity.dispatch.value() != expected_dispatch
        || identity.au_index != expected_au
        || source >= provider.expected_sources
    {
        return Err("OUTPUT_SOURCE_CLOCK");
    }

    let mut after = state.clone();
    after.generation = after.generation.checked_add(1).ok_or("OUTPUT_GENERATION")?;
    after.next_source_index =
        CodecSourceIndex::new(source.checked_add(1).ok_or("OUTPUT_SOURCE_CLOCK")?);

    if source < provider.suppressed_sources {
        let receipt = json!({
            "schema": "output-suppressed-source-receipt.v1",
            "owner_id": "output",
            "source_index": source,
            "dispatch": identity.dispatch.value(),
            "au_index": null,
            "before_sha256": state.sha256(),
            "after_sha256": after.sha256(),
            "append_required": false,
            "commit_count": 1,
        });
        return Ok(PreparedSource::Suppressed { receipt, after });
    }

    if frame.len() != provider.frame_bytes || frame.len() != OUTPUT_FRAME_BYTES {
        return Err("OUTPUT_FRAME_LENGTH");
    }
    let actual_sha256 = digest_hex(frame);
    if declared_sha256 != actual_sha256 {
        return Err("OUTPUT_FRAME_HASH");
    }
    let crc_residual = eac3_crc16(&frame[2..]);
    if crc_residual != 0 {
        return Err("OUTPUT_FRAME_CRC");
    }

    let au_index = identity.au_index.expect("published source has an AU index");
    let offset = au_index
        .value()
        .checked_mul(u64::try_from(provider.frame_bytes).expect("frame bytes fit U64"))
        .ok_or("OUTPUT_OFFSET")?;
    let entry = LedgerEntry {
        au_index,
        offset,
        length: u32::try_from(provider.frame_bytes).expect("frame bytes fit U32"),
        frame_sha256: digest(frame),
        crc_residual,
    };
    after.published_au_count = after
        .published_au_count
        .checked_add(1)
        .ok_or("OUTPUT_AU_COUNT")?;
    after.durable_length = after
        .durable_length
        .checked_add(u64::try_from(provider.frame_bytes).expect("frame bytes fit U64"))
        .ok_or("OUTPUT_LENGTH")?;
    after.ledger.push(entry.clone());

    let proposal = json!({
        "schema": "output-append-proposal.v1",
        "attempt_id": state.attempt_id,
        "staging_name": state.staging_name,
        "source_index": source,
        "dispatch": identity.dispatch.value(),
        "au_index": au_index.value(),
        "expected_offset": offset,
        "frame_bytes": provider.frame_bytes,
        "frame_sha256": actual_sha256,
        "frame_hex": hex(frame),
        "before_state_sha256": state.sha256(),
        "proposed_after_sha256": after.sha256(),
        "ledger_entry": entry.to_json(),
    });
    Ok(PreparedSource::Append(AppendProposal {
        value: proposal,
        after,
        frame: frame.to_vec(),
    }))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppendFault {
    None,
    ZeroWriteAfter(usize),
    CorruptReadback {
        frame_relative_byte_offset: u64,
        before: u8,
        xor: u8,
    },
    FlushFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ShellFailure {
    code: &'static str,
    write_calls: Vec<u64>,
    physical_length: u64,
    disposition: &'static str,
}

fn append_to_staging(
    staging_path: &Path,
    proposal: &AppendProposal,
    max_write_bytes: usize,
    fault: AppendFault,
) -> Result<Value, ShellFailure> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(staging_path)
        .expect("staging file opens");
    let expected_offset = proposal.value["expected_offset"]
        .as_u64()
        .expect("proposal offset");
    let initial_length = file.metadata().expect("staging metadata").len();
    assert_eq!(initial_length, expected_offset);
    file.seek(SeekFrom::Start(expected_offset))
        .expect("seek to proposal offset");

    let mut write_calls = Vec::new();
    let mut written = 0_usize;
    while written < proposal.frame.len() {
        if fault == AppendFault::ZeroWriteAfter(write_calls.len()) {
            write_calls.push(0);
            return Err(ShellFailure {
                code: "ZERO_WRITE",
                write_calls,
                physical_length: file.metadata().expect("staging metadata").len(),
                disposition: "ABANDON",
            });
        }
        let requested = max_write_bytes.min(proposal.frame.len() - written);
        let count = file
            .write(&proposal.frame[written..written + requested])
            .expect("synthetic staging write");
        write_calls.push(count as u64);
        if count == 0 {
            return Err(ShellFailure {
                code: "ZERO_WRITE",
                write_calls,
                physical_length: file.metadata().expect("staging metadata").len(),
                disposition: "ABANDON",
            });
        }
        written += count;
    }

    let expected_end = expected_offset + proposal.frame.len() as u64;
    assert_eq!(
        file.metadata().expect("staging metadata").len(),
        expected_end
    );

    if let AppendFault::CorruptReadback {
        frame_relative_byte_offset,
        before,
        xor,
    } = fault
    {
        let absolute_offset = expected_offset + frame_relative_byte_offset;
        file.seek(SeekFrom::Start(absolute_offset))
            .expect("seek to synthetic corruption byte");
        let mut byte = [0_u8; 1];
        file.read_exact(&mut byte)
            .expect("read synthetic corruption byte");
        assert_eq!(byte[0], before);
        byte[0] ^= xor;
        file.seek(SeekFrom::Start(absolute_offset))
            .expect("seek back to synthetic corruption byte");
        file.write_all(&byte)
            .expect("write synthetic corruption byte");
    }

    file.seek(SeekFrom::Start(expected_offset))
        .expect("seek for append readback");
    let mut readback = vec![0_u8; proposal.frame.len()];
    file.read_exact(&mut readback)
        .expect("read back appended bytes");
    if readback != proposal.frame || digest_hex(&readback) != proposal.value["frame_sha256"] {
        return Err(ShellFailure {
            code: "READBACK_HASH",
            write_calls,
            physical_length: file.metadata().expect("staging metadata").len(),
            disposition: "ABANDON",
        });
    }
    let crc_residual = eac3_crc16(&readback[2..]);
    if crc_residual != 0 {
        return Err(ShellFailure {
            code: "READBACK_CRC",
            write_calls,
            physical_length: file.metadata().expect("staging metadata").len(),
            disposition: "ABANDON",
        });
    }
    if fault == AppendFault::FlushFailure {
        return Err(ShellFailure {
            code: "FLUSH_FAILURE",
            write_calls,
            physical_length: file.metadata().expect("staging metadata").len(),
            disposition: "ABANDON",
        });
    }

    file.flush().expect("flush staging file");
    file.sync_all().expect("sync staging file");
    Ok(json!({
        "schema": "durable-append-receipt.v1",
        "proposal_sha256": canonical_json_sha256(&proposal.value),
        "attempt_id": proposal.value["attempt_id"],
        "au_index": proposal.value["au_index"],
        "offset": expected_offset,
        "bytes_written": written,
        "write_calls": write_calls,
        "readback_sha256": digest_hex(&readback),
        "crc_residual": crc_residual,
        "file_fsync": true,
        "durable": true,
    }))
}

fn commit_append(
    before: &OutputState,
    proposal: AppendProposal,
    durable_receipt: &Value,
) -> Result<(OutputState, Value), &'static str> {
    if before.sha256() != proposal.value["before_state_sha256"]
        || durable_receipt["schema"] != "durable-append-receipt.v1"
        || durable_receipt["proposal_sha256"] != canonical_json_sha256(&proposal.value)
        || durable_receipt["attempt_id"] != proposal.value["attempt_id"]
        || durable_receipt["au_index"] != proposal.value["au_index"]
        || durable_receipt["offset"] != proposal.value["expected_offset"]
        || durable_receipt["bytes_written"] != OUTPUT_FRAME_BYTES
        || durable_receipt["readback_sha256"] != proposal.value["frame_sha256"]
        || durable_receipt["crc_residual"] != 0
        || durable_receipt["file_fsync"] != true
        || durable_receipt["durable"] != true
    {
        return Err("OUTPUT_DURABLE_RECEIPT");
    }

    let after = proposal.after;
    if after.sha256() != proposal.value["proposed_after_sha256"] {
        return Err("OUTPUT_PROPOSAL_STATE");
    }
    let receipt = json!({
        "schema": "output-owner-commit-receipt.v1",
        "owner_id": "output",
        "before_sha256": before.sha256(),
        "after_sha256": after.sha256(),
        "au_index": proposal.value["au_index"],
        "commit_count": 1,
    });
    Ok((after, receipt))
}

fn durable_ledger_preimage(ledger: &[LedgerEntry]) -> Vec<u8> {
    let mut bytes = b"OutputDurableLedgerV1\0".to_vec();
    bytes.extend_from_slice(
        &u32::try_from(ledger.len())
            .expect("ledger count fits U32")
            .to_le_bytes(),
    );
    for entry in ledger {
        bytes.extend_from_slice(&entry.au_index.value().to_le_bytes());
        bytes.extend_from_slice(&entry.offset.to_le_bytes());
        bytes.extend_from_slice(&entry.length.to_le_bytes());
        bytes.extend_from_slice(&entry.frame_sha256);
        bytes.extend_from_slice(&entry.crc_residual.to_le_bytes());
    }
    bytes
}

fn prepare_finalize(state: &OutputState, provider: &OutputProvider) -> Result<Value, &'static str> {
    if state.profile_id != provider.profile_id
        || state.provider_sha256 != provider.provider_sha256
        || state.finalized
        || state.next_source_index.value() != provider.expected_sources
        || state.published_au_count != provider.expected_published_aus
        || state.ledger.len() as u64 != provider.expected_published_aus
        || state.durable_length != provider.expected_product_bytes
    {
        return Err("OUTPUT_FINALIZE_COUNTS");
    }
    for (index, entry) in state.ledger.iter().enumerate() {
        let au_index = u64::try_from(index).expect("ledger index fits U64");
        let expected_offset = au_index
            .checked_mul(u64::try_from(provider.frame_bytes).expect("frame bytes fit U64"))
            .ok_or("OUTPUT_FINALIZE_LEDGER")?;
        if entry.au_index.value() != au_index
            || entry.offset != expected_offset
            || u64::from(entry.length)
                != u64::try_from(provider.frame_bytes).expect("frame bytes fit U64")
            || entry.crc_residual != 0
        {
            return Err("OUTPUT_FINALIZE_LEDGER");
        }
    }
    Ok(json!({
        "schema": "output-finalize-proposal.v1",
        "attempt_id": state.attempt_id,
        "staging_name": state.staging_name,
        "product_name": state.product_name,
        "state_sha256": state.sha256(),
        "expected_length": state.durable_length,
        "ledger_sha256": digest_hex(&durable_ledger_preimage(&state.ledger)),
    }))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FinalizeFault {
    RenameFailure,
    ParentFsyncFailure,
    ParentFsyncUnsupported,
}

#[derive(Debug)]
struct FinalizeOutcome {
    result: Value,
    steps: Vec<&'static str>,
    intent_path: PathBuf,
}

fn run_unqualified_finalize_shell(
    state: &OutputState,
    provider: &OutputProvider,
    proposal: &Value,
    directory: &Path,
    fault: FinalizeFault,
) -> FinalizeOutcome {
    let staging_path = directory.join(
        proposal["staging_name"]
            .as_str()
            .expect("proposal staging name"),
    );
    let product_path = directory.join(
        proposal["product_name"]
            .as_str()
            .expect("proposal product name"),
    );
    let intent_name = provider
        .intent_name_template
        .replace("{attempt_id}", &state.attempt_id);
    let intent_path = directory.join(intent_name);
    let mut steps = Vec::new();

    assert_eq!(state.sha256(), proposal["state_sha256"]);
    assert_eq!(
        fs::metadata(&staging_path)
            .expect("staging exists before finalization")
            .len(),
        proposal["expected_length"]
            .as_u64()
            .expect("expected length")
    );
    let product_bytes = fs::read(&staging_path).expect("read complete staged product");
    assert_eq!(product_bytes.len() as u64, provider.expected_product_bytes);
    let product_sha256 = digest_hex(&product_bytes);
    steps.push("staging_verify");

    OpenOptions::new()
        .read(true)
        .write(true)
        .open(&staging_path)
        .expect("reopen staging for fsync")
        .sync_all()
        .expect("fsync staging file");
    steps.push("staging_fsync");

    let intent = json!({
        "attempt_id": state.attempt_id,
        "product_length": product_bytes.len(),
        "product_sha256": product_sha256,
        "state_sha256": state.sha256(),
    });
    let intent_bytes = serde_json::to_vec(&intent).expect("canonical finalize intent JSON");
    let mut intent_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&intent_path)
        .expect("create finalize intent");
    intent_file
        .write_all(&intent_bytes)
        .expect("write finalize intent");
    intent_file.flush().expect("flush finalize intent");
    intent_file.sync_all().expect("fsync finalize intent");
    steps.push("intent_write_fsync");

    if fault == FinalizeFault::RenameFailure {
        steps.push("atomic_replace");
        return FinalizeOutcome {
            result: json!({
                "status": "ERROR",
                "error": "RENAME_FAILURE",
                "receipt": null,
                "staging_exists": staging_path.exists(),
                "product_exists": product_path.exists(),
                "intent_exists": intent_path.exists(),
                "attempt_disposition": "RETIRED_RECOVERY_REQUIRED",
            }),
            steps,
            intent_path,
        };
    }

    fs::rename(&staging_path, &product_path).expect("replace staging with product");
    steps.push("atomic_replace");
    steps.push("parent_directory_fsync");
    let error = match fault {
        FinalizeFault::ParentFsyncFailure => "PARENT_FSYNC_FAILURE",
        FinalizeFault::ParentFsyncUnsupported => "PARENT_FSYNC_UNSUPPORTED",
        FinalizeFault::RenameFailure => unreachable!("rename failure returned above"),
    };
    FinalizeOutcome {
        result: json!({
            "status": "ERROR",
            "error": error,
            "receipt": null,
            "staging_exists": staging_path.exists(),
            "product_exists": product_path.exists(),
            "intent_exists": intent_path.exists(),
            "attempt_disposition": "RETIRED_RECOVERY_REQUIRED",
        }),
        steps,
        intent_path,
    }
}

#[derive(Debug)]
struct ScratchDirectory(PathBuf);

impl ScratchDirectory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir();
        let path = root.join(format!(
            "jocforge-output-owner-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("create isolated test directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScratchDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn read_fixture(name: &str) -> Value {
    let bytes = fs::read(fixture_root().join(name)).expect("fixture exists");
    serde_json::from_slice(&bytes).expect("fixture JSON")
}

fn input_frame(input: &Value) -> Vec<u8> {
    decode_hex(input["frame_hex"].as_str().expect("input frame hex"))
}

fn prepare_from_call(
    state: &OutputState,
    provider: &OutputProvider,
    call: &Value,
) -> Result<PreparedSource, &'static str> {
    let input = &call["input"];
    prepare_source(
        state,
        provider,
        SourceIdentity::from_input(&input["identity"]),
        &input_frame(input),
        input["declared_sha256"]
            .as_str()
            .expect("declared frame SHA"),
    )
}

fn first_append_proposal(
    provider: &OutputProvider,
    vectors: &Value,
) -> (OutputState, AppendProposal) {
    let prestate_vector = vectors["failure_vectors"]
        .as_array()
        .expect("failure vectors")
        .iter()
        .find(|entry| entry["id"] == "crc_failure")
        .expect("explicit source-2 prestate");
    let state = OutputState::from_input(&prestate_vector["prestate"]);
    let input = &vectors["continuous_calls"][2]["input"];
    let proposal = prepare_source(
        &state,
        provider,
        SourceIdentity::from_input(&input["identity"]),
        &input_frame(input),
        input["declared_sha256"]
            .as_str()
            .expect("declared frame SHA"),
    )
    .expect("valid source-2 input");
    match proposal {
        PreparedSource::Append(proposal) => (state, proposal),
        PreparedSource::Suppressed { .. } => panic!("source 2 must yield an append proposal"),
    }
}

fn replay_successful_attempt(
    provider: &OutputProvider,
    vectors: &Value,
) -> (OutputState, ScratchDirectory, PathBuf) {
    let mut state = OutputState::from_input(&vectors["cold"]["state"]);
    let scratch = ScratchDirectory::new();
    let staging_path = scratch.path().join(&state.staging_name);
    File::create(&staging_path).expect("create synthetic staging stream");

    for (index, call) in vectors["continuous_calls"]
        .as_array()
        .expect("continuous call vector")
        .iter()
        .enumerate()
    {
        let prepared =
            prepare_from_call(&state, provider, call).expect("input-only source prepares");
        assert_eq!(prepared.kind(), call["prepared"]["kind"]);
        assert_eq!(prepared.prepared_json(), call["prepared"]);
        match prepared {
            PreparedSource::Suppressed { receipt, after } => {
                assert_eq!(receipt, call["prepared"]["receipt"]);
                assert_eq!(after.to_json(), call["committed_state"]);
                state = after;
            }
            PreparedSource::Append(proposal) => {
                assert_eq!(proposal.value, call["prepared"]["proposal"]);
                let max_write_bytes = if index == 2 { 257 } else { OUTPUT_FRAME_BYTES };
                let durable_receipt =
                    append_to_staging(&staging_path, &proposal, max_write_bytes, AppendFault::None)
                        .expect("synthetic append is durable");
                assert_eq!(durable_receipt, call["durable_receipt"]);
                let (after, commit_receipt) = commit_append(&state, proposal, &durable_receipt)
                    .expect("durable append commits");
                assert_eq!(commit_receipt, call["commit_receipt"]);
                assert_eq!(after.to_json(), call["committed_state"]);
                state = after;
            }
        }
    }
    (state, scratch, staging_path)
}

#[test]
fn input_only_vectors_recompute_cold_and_five_source_proposals() {
    assert_fixture_manifest();
    let provider_value = read_fixture(PROVIDER_FILE);
    let provider = OutputProvider::from_input(&provider_value);
    let publication_provider = read_fixture("output-publication-provider.json");
    let vectors = read_fixture(VECTORS_FILE);
    assert_eq!(vectors["expected_used_as_seed"], false);
    assert_eq!(provider_value["production_allowed"], false);
    assert_eq!(publication_provider["production_allowed"], false);
    assert_eq!(publication_provider["expected_sources"], 4249);
    assert_eq!(publication_provider["expected_published_aus"], 4247);
    assert_eq!(provider.expected_sources, 5);
    assert_eq!(provider.expected_published_aus, 3);
    assert_eq!(provider.expected_product_bytes, 12_288);
    assert_eq!(provider.frame_bytes, OUTPUT_FRAME_BYTES);

    let cold_state = OutputState::from_input(&vectors["cold"]["state"]);
    assert_eq!(cold_state.sha256(), vectors["cold"]["sha256"]);
    let (state, _scratch, staging_path) = replay_successful_attempt(&provider, &vectors);
    assert_eq!(provider.profile_id, state.profile_id);
    assert_eq!(provider.provider_sha256, state.provider_sha256);
    assert_eq!(
        cold_state.canonical_bytes(),
        decode_hex(vectors["cold"]["canonical_hex"].as_str().unwrap())
    );
    assert_eq!(cold_state.sha256(), vectors["cold"]["sha256"]);

    assert_eq!(state.to_json(), vectors["terminal_state"]["state"]);
    assert_eq!(
        state.canonical_bytes(),
        decode_hex(vectors["terminal_state"]["canonical_hex"].as_str().unwrap())
    );
    assert_eq!(state.sha256(), vectors["terminal_state"]["sha256"]);
    assert_eq!(
        fs::metadata(&staging_path)
            .expect("staging stream remains open")
            .len(),
        provider.expected_product_bytes
    );
}

#[test]
fn pre_append_crc_and_declared_hash_failures_leave_state_and_files_unchanged() {
    assert_fixture_manifest();
    let provider = OutputProvider::from_input(&read_fixture(PROVIDER_FILE));
    let vectors = read_fixture(VECTORS_FILE);
    for failure in vectors["failure_vectors"]
        .as_array()
        .expect("failure vectors")
    {
        let id = failure["id"].as_str().expect("failure ID");
        if id != "crc_failure" && id != "declared_hash_mismatch" {
            continue;
        }
        let state = OutputState::from_input(&failure["prestate"]);
        let before_sha256 = state.sha256();
        assert_eq!(before_sha256, failure["result"]["before_sha256"]);

        let scratch = ScratchDirectory::new();
        let staging_path = scratch.path().join(format!("{id}.partial"));
        fs::write(&staging_path, b"unchanged synthetic staging").expect("seed staging bytes");
        let before_file = fs::read(&staging_path).expect("read initial staging bytes");
        let input = &failure["input"];
        let result = prepare_source(
            &state,
            &provider,
            SourceIdentity::from_input(&input["identity"]),
            &input_frame(input),
            input["declared_sha256"].as_str().expect("declared SHA"),
        );
        let error = result.expect_err("frozen pre-append failure rejects");
        assert_eq!(error, failure["result"]["error"]);
        assert_eq!(state.sha256(), failure["result"]["after_sha256"]);
        assert_eq!(state.sha256(), before_sha256);
        assert_eq!(
            fs::read(&staging_path).expect("staging bytes unchanged"),
            before_file
        );
        assert_eq!(failure["result"]["proposal"], Value::Null);
        assert_eq!(failure["result"]["receipt"], Value::Null);
        assert_eq!(failure["result"]["proposed_state"], Value::Null);
        assert_eq!(failure["result"]["commit_count"], 0);
    }
}

#[test]
fn physical_append_failures_never_commit_the_proposed_state() {
    assert_fixture_manifest();
    let provider = OutputProvider::from_input(&read_fixture(PROVIDER_FILE));
    let vectors = read_fixture(VECTORS_FILE);
    let failure_vectors = vectors["failure_vectors"]
        .as_array()
        .expect("failure vectors");
    let probe = read_fixture("LOCAL-FILESYSTEM-PROBE.json");
    for (id, fault, max_write_bytes) in [
        ("zero_write", AppendFault::ZeroWriteAfter(2), 257),
        (
            "readback_corruption",
            AppendFault::CorruptReadback {
                frame_relative_byte_offset: 17,
                before: 0x5f,
                xor: 0x01,
            },
            257,
        ),
        ("flush_failure", AppendFault::FlushFailure, 257),
    ] {
        let failure = failure_vectors
            .iter()
            .find(|entry| entry["id"] == id)
            .expect("fault vector exists");
        let (state, proposal) = first_append_proposal(&provider, &vectors);
        assert_eq!(proposal.value, failure["proposal"]);
        let before_sha256 = state.sha256();
        assert_eq!(before_sha256, failure["committed_before_sha256"]);

        let scratch = ScratchDirectory::new();
        let staging_path = scratch.path().join(format!("{id}.partial"));
        File::create(&staging_path).expect("create empty staging stream");
        let shell_failure = append_to_staging(&staging_path, &proposal, max_write_bytes, fault)
            .expect_err("injected filesystem fault is fatal");
        assert_eq!(shell_failure.code, failure["result"]["error"]);
        assert_eq!(
            shell_failure.write_calls,
            numbers(&failure["result"]["writes"])
        );
        assert_eq!(
            shell_failure.physical_length,
            failure["result"]["physical_length"]
        );
        assert_eq!(
            shell_failure.disposition,
            failure["result"]["attempt_disposition"]
        );
        assert_eq!(state.sha256(), failure["committed_after_sha256"]);
        assert_eq!(state.sha256(), before_sha256);
        assert_eq!(failure["result"]["receipt"], Value::Null);

        let physical_bytes = fs::read(&staging_path).expect("abandoned bytes remain on disk");
        assert_eq!(physical_bytes.len() as u64, shell_failure.physical_length);
        match id {
            "zero_write" | "flush_failure" => {
                let expected_name = format!("{id}.partial");
                let expected_hash = probe["work_files"]
                    .as_array()
                    .expect("probe work files")
                    .iter()
                    .find(|file| file["name"] == expected_name)
                    .expect("matching physical file")["sha256"]
                    .as_str()
                    .expect("physical file SHA-256");
                assert_eq!(digest_hex(&physical_bytes), expected_hash);
            }
            "readback_corruption" => {
                assert_eq!(
                    digest_hex(&physical_bytes),
                    failure["result"]["physical_sha256"]
                );
                assert_eq!(physical_bytes[17], 0x5e);
            }
            _ => unreachable!("only append shell faults are listed"),
        }
    }
}

#[test]
fn finalize_proposal_ledger_digest_and_intent_are_recomputed_from_committed_inputs() {
    assert_fixture_manifest();
    let provider = OutputProvider::from_input(&read_fixture(PROVIDER_FILE));
    let vectors = read_fixture(VECTORS_FILE);
    let (state, _scratch, staging_path) = replay_successful_attempt(&provider, &vectors);

    let ledger_vector = &vectors["ledger_digest_vector"];
    let ledger_preimage = durable_ledger_preimage(&state.ledger);
    assert_eq!(
        ledger_preimage,
        decode_hex(
            ledger_vector["preimage_hex"]
                .as_str()
                .expect("ledger preimage")
        )
    );
    assert_eq!(
        ledger_preimage.len() as u64,
        ledger_vector["preimage_bytes"]
    );
    assert_eq!(digest_hex(&ledger_preimage), ledger_vector["sha256"]);

    let proposal = prepare_finalize(&state, &provider).expect("exact terminal counts finalize");
    assert_eq!(proposal, vectors["finalize_proposal"]);
    assert_eq!(proposal["ledger_sha256"], digest_hex(&ledger_preimage));

    let product_bytes = fs::read(&staging_path).expect("read synthetic product input");
    let intent = json!({
        "attempt_id": state.attempt_id,
        "product_length": product_bytes.len(),
        "product_sha256": digest_hex(&product_bytes),
        "state_sha256": state.sha256(),
    });
    let intent_vector = &vectors["finalize_intent_vector"];
    let intent_bytes = serde_json::to_vec(&intent).expect("canonical finalize intent");
    assert_eq!(intent, intent_vector["input"]);
    assert_eq!(
        intent_bytes,
        decode_hex(
            intent_vector["canonical_utf8_hex"]
                .as_str()
                .expect("canonical intent bytes")
        )
    );
    assert_eq!(intent_bytes.len() as u64, intent_vector["bytes"]);
    assert_eq!(digest_hex(&intent_bytes), intent_vector["sha256"]);
}

#[test]
fn local_finalize_and_failure_vectors_never_claim_final_exposure() {
    assert_fixture_manifest();
    let provider = OutputProvider::from_input(&read_fixture(PROVIDER_FILE));
    let vectors = read_fixture(VECTORS_FILE);
    let probe = read_fixture("LOCAL-FILESYSTEM-PROBE.json");
    let failure_vectors = vectors["failure_vectors"]
        .as_array()
        .expect("failure vectors");

    for (id, fault) in [
        ("rename_failure", FinalizeFault::RenameFailure),
        ("parent_fsync_failure", FinalizeFault::ParentFsyncFailure),
    ] {
        let failure = failure_vectors
            .iter()
            .find(|entry| entry["id"] == id)
            .expect("finalize failure vector");
        let (state, scratch, staging_path) = replay_successful_attempt(&provider, &vectors);
        let proposal = prepare_finalize(&state, &provider).expect("terminal state finalizes");
        assert_eq!(proposal, vectors["finalize_proposal"]);
        assert_eq!(state.sha256(), failure["committed_before_sha256"]);

        let outcome =
            run_unqualified_finalize_shell(&state, &provider, &proposal, scratch.path(), fault);
        assert_eq!(outcome.result, failure["result"]);
        assert_eq!(state.sha256(), failure["committed_after_sha256"]);
        assert_eq!(state.sha256(), proposal["state_sha256"]);
        assert_eq!(outcome.result["receipt"], Value::Null);
        assert_eq!(
            outcome.steps,
            if fault == FinalizeFault::RenameFailure {
                vec![
                    "staging_verify",
                    "staging_fsync",
                    "intent_write_fsync",
                    "atomic_replace",
                ]
            } else {
                vec![
                    "staging_verify",
                    "staging_fsync",
                    "intent_write_fsync",
                    "atomic_replace",
                    "parent_directory_fsync",
                ]
            }
        );

        let intent_vector = &vectors["finalize_intent_vector"];
        let intent_bytes = fs::read(&outcome.intent_path).expect("intent persists on failure");
        assert_eq!(
            intent_bytes,
            decode_hex(intent_vector["canonical_utf8_hex"].as_str().unwrap())
        );
        assert_eq!(digest_hex(&intent_bytes), intent_vector["sha256"]);
        assert!(staging_path.exists() == (fault == FinalizeFault::RenameFailure));
        let product_path = scratch.path().join(&state.product_name);
        assert_eq!(product_path.exists(), fault != FinalizeFault::RenameFailure);
        if product_path.exists() {
            let product_bytes = fs::read(&product_path).expect("exposed product after replace");
            assert_eq!(
                digest_hex(&product_bytes),
                intent_vector["input"]["product_sha256"]
            );
        }
    }

    let (state, scratch, staging_path) = replay_successful_attempt(&provider, &vectors);
    let proposal = prepare_finalize(&state, &provider).expect("terminal state finalizes");
    let outcome = run_unqualified_finalize_shell(
        &state,
        &provider,
        &proposal,
        scratch.path(),
        FinalizeFault::ParentFsyncUnsupported,
    );
    assert_eq!(outcome.result, vectors["local_finalize_result"]);
    assert_eq!(outcome.result, probe["finalize_result"]);
    assert!(!staging_path.exists());
    let product_path = scratch.path().join(&state.product_name);
    let product_bytes = fs::read(&product_path).expect("product remains after unsupported fsync");
    assert_eq!(
        digest_hex(&product_bytes),
        probe["work_files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|file| file["name"] == state.product_name)
            .unwrap()["sha256"]
    );
    let intent_bytes = fs::read(&outcome.intent_path)
        .expect("intent exists after parent directory fsync is unqualified");
    let intent_vector = &vectors["finalize_intent_vector"];
    assert_eq!(
        intent_bytes,
        decode_hex(
            intent_vector["canonical_utf8_hex"]
                .as_str()
                .expect("canonical intent bytes")
        )
    );
    assert_eq!(digest_hex(&intent_bytes), intent_vector["sha256"]);
    let observed_intent = probe["work_files"]
        .as_array()
        .expect("probe work files")
        .iter()
        .find(|file| file["name"] == "attempt-synthetic-0.finalize-intent.json")
        .expect("observed synthetic intent");
    assert_eq!(intent_bytes.len() as u64, observed_intent["bytes"]);
    assert_eq!(digest_hex(&intent_bytes), observed_intent["sha256"]);
    assert!(!state.finalized);
    assert_eq!(state.sha256(), proposal["state_sha256"]);
    assert_eq!(outcome.result["receipt"], Value::Null);
    assert_eq!(probe["parent_fsync_qualified"], false);
    assert_eq!(probe["production_qualification"], false);
    assert_eq!(vectors["real_4249_publication"], "NOT_RUN");
    assert_eq!(vectors["power_loss_recovery"], "NOT_RUN");
    assert_eq!(vectors["product_execution"], "NOT_RUN");
    assert_eq!(
        outcome.steps,
        vec![
            "staging_verify",
            "staging_fsync",
            "intent_write_fsync",
            "atomic_replace",
            "parent_directory_fsync",
        ]
    );
}

#[test]
fn recovery_vectors_are_classifications_not_automatic_recovery_authority() {
    assert_fixture_manifest();
    let vectors = read_fixture(VECTORS_FILE);
    assert_eq!(vectors["expected_used_as_seed"], false);
    assert_eq!(
        vectors["recovery_vectors"],
        json!([
            {
                "id":"partial_staging",
                "physical_length":100,
                "committed_length":0,
                "classification":"ABANDON_PARTIAL_RESTART"
            },
            {
                "id":"durable_without_state_commit",
                "physical_length":4096,
                "committed_length":0,
                "classification":"ABANDON_UNCOMMITTED_DURABLE_RESTART"
            },
            {
                "id":"replace_without_parent_fsync",
                "classification":"RETIRED_RECOVERY_REQUIRED_NO_AUTOMATIC_DECISION"
            }
        ])
    );
}

fn numbers(value: &Value) -> Vec<u64> {
    value
        .as_array()
        .expect("number array")
        .iter()
        .map(|item| item.as_u64().expect("unsigned integer"))
        .collect()
}
