use super::*;
use jocforge_domain::publication::{
    OutputOwner, OutputProvider, OutputSourceIdentity, OutputSourceProposal,
};
use jocforge_domain::timeline::{CodecSourceIndex, JocDispatchIndex, PublishedAuIndex};
use serde_json::Value;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    io,
    time::{SystemTime, UNIX_EPOCH},
};

const SYNTHETIC_PROVIDER: &[u8] = include_bytes!(
    "../../../jocforge-domain/tests/fixtures/output_owner/synthetic-output-provider.json"
);

struct ScratchDirectory(PathBuf);

impl ScratchDirectory {
    fn new() -> io::Result<Self> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("jocforge-output-{}-{nonce}", std::process::id()));
        fs::create_dir(&path)?;
        Ok(Self(path))
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

struct LimitedWriter {
    maximum: usize,
    bytes: Vec<u8>,
}

impl Write for LimitedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let count = bytes.len().min(self.maximum);
        self.bytes.extend_from_slice(&bytes[..count]);
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct ZeroWriter;

impl Write for ZeroWriter {
    fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn frame_sha256(frame: &[u8]) -> [u8; 32] {
    Sha256::digest(frame).into()
}

fn output_owner() -> OutputOwner {
    let provider: Value = serde_json::from_slice(SYNTHETIC_PROVIDER).unwrap();
    let canonical = serde_json::to_vec(&provider).unwrap();
    let attempt_id = "synthetic-0";
    OutputOwner::cold(
        OutputConfiguration::new(
            OutputProvider {
                profile_id: provider["profile_id"].as_str().unwrap().to_owned(),
                provider_sha256: Sha256::digest(&canonical).into(),
                frame_bytes: provider["frame_bytes"].as_u64().unwrap() as u32,
                suppressed_sources: provider["suppressed_sources"].as_u64().unwrap(),
                staging_name_template: provider["staging_name_template"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
                intent_name_template: provider["intent_name_template"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
            },
            attempt_id,
            "product.ec3",
            4_608,
        )
        .unwrap(),
    )
}

fn advance_suppressed_sources(mut owner: OutputOwner) -> OutputOwner {
    let frame = [0_u8; FRAME_BYTES];
    let frame_sha = frame_sha256(&frame);
    for source in 0..2 {
        let proposal = owner
            .prepare_source(
                OutputSourceIdentity::new(
                    CodecSourceIndex::new(source),
                    JocDispatchIndex::new(source + 1),
                    None,
                ),
                &frame,
                &frame_sha,
            )
            .unwrap();
        let OutputSourceProposal::Suppressed(proposal) = proposal else {
            panic!("source {source} must not append")
        };
        owner = proposal.into_after().into_after_owner();
    }
    owner
}

fn terminal_output(scratch: &ScratchDirectory) -> (OutputOwner, AtomicOutputStaging) {
    let mut owner = output_owner();
    let mut staging = AtomicOutputStaging::create(scratch.path(), owner.configuration()).unwrap();
    let frame = [0_u8; FRAME_BYTES];
    let frame_sha = frame_sha256(&frame);

    for source in 0..2 {
        let proposal = owner
            .prepare_source(
                OutputSourceIdentity::new(
                    CodecSourceIndex::new(source),
                    JocDispatchIndex::new(source + 1),
                    None,
                ),
                &frame,
                &frame_sha,
            )
            .unwrap();
        let OutputSourceProposal::Suppressed(proposal) = proposal else {
            panic!("source {source} must be suppressed")
        };
        owner = proposal.into_after().into_after_owner();
    }

    for source in 2..5 {
        let proposal = owner
            .prepare_source(
                OutputSourceIdentity::new(
                    CodecSourceIndex::new(source),
                    JocDispatchIndex::new(source + 1),
                    Some(PublishedAuIndex::new(source - 2)),
                ),
                &frame,
                &frame_sha,
            )
            .unwrap();
        let OutputSourceProposal::Append(proposal) = proposal else {
            panic!("source {source} must publish")
        };
        let receipt = staging.append(&proposal).unwrap();
        owner = proposal.complete(receipt).unwrap().into_after_owner();
    }

    (owner, staging)
}

#[test]
fn selected_append_returns_durable_receipt_only_after_readback_and_fsync() {
    let scratch = ScratchDirectory::new().unwrap();
    let owner = advance_suppressed_sources(output_owner());
    let mut staging = AtomicOutputStaging::create(scratch.path(), owner.configuration()).unwrap();
    let frame = [0_u8; FRAME_BYTES];
    let proposal = owner
        .prepare_source(
            OutputSourceIdentity::new(
                CodecSourceIndex::new(2),
                JocDispatchIndex::new(3),
                Some(PublishedAuIndex::new(0)),
            ),
            &frame,
            &frame_sha256(&frame),
        )
        .unwrap();
    let OutputSourceProposal::Append(proposal) = proposal else {
        panic!("source2 must append AU0")
    };

    let receipt = staging.append(&proposal).unwrap();
    assert_eq!(receipt.bytes_written(), FRAME_BYTES_U32);
    assert_eq!(receipt.offset(), 0);
    assert_eq!(receipt.crc_residual(), 0);
    assert!(receipt.file_fsync());
    assert!(receipt.durable());
    assert_eq!(
        receipt.write_calls().iter().sum::<u64>(),
        u64::from(FRAME_BYTES_U32)
    );
    assert_eq!(fs::read(staging.staging_path()).unwrap(), frame);

    let after = proposal.complete(receipt).unwrap().into_after_owner();
    assert_eq!(after.state_json()["published_au_count"], 1);
    assert_eq!(
        after.state_json()["durable_length"],
        u64::from(FRAME_BYTES_U32)
    );
    assert!(!staging.is_abandoned());
}

#[test]
fn output_path_collision_is_rejected_without_creating_staging() {
    let scratch = ScratchDirectory::new().unwrap();
    let owner = output_owner();
    fs::write(
        scratch.path().join(owner.configuration().product_name()),
        b"existing",
    )
    .unwrap();

    let error = AtomicOutputStaging::create(scratch.path(), owner.configuration()).unwrap_err();
    assert_eq!(error.failure(), OutputIoFailure::FinalPathExists);
    assert!(
        !scratch
            .path()
            .join(owner.configuration().staging_name())
            .exists()
    );
}

#[test]
fn explicit_progress_loop_continues_short_writes_and_rejects_zero_progress() {
    let bytes = [0x5a; FRAME_BYTES];
    let mut writer = LimitedWriter {
        maximum: 257,
        bytes: Vec::new(),
    };
    let calls = write_all_progress(&mut writer, &bytes).unwrap();
    assert_eq!(
        calls,
        vec![257_u64; 15]
            .into_iter()
            .chain([241])
            .collect::<Vec<_>>()
    );
    assert_eq!(writer.bytes, bytes);

    assert!(matches!(
        write_all_progress(&mut ZeroWriter, &bytes),
        Err(WriteProgressError::ZeroProgress)
    ));
}

#[test]
fn terminal_finalize_writes_canonical_intent_then_exposes_final_path() {
    let scratch = ScratchDirectory::new().unwrap();
    let (owner, mut staging) = terminal_output(&scratch);
    let proposal = owner.prepare_finalize().unwrap();
    let expected_product = vec![0_u8; 12_288];
    let expected_intent = json!({
        "attempt_id": "synthetic-0",
        "product_length": expected_product.len(),
        "product_sha256": hex_lower(&frame_sha256(&expected_product)),
        "state_sha256": hex_lower(proposal.state_sha256()),
    });

    let receipt = staging
        .finalize_with_parent_sync(&proposal, |_| Ok(()))
        .unwrap();
    assert!(staging.product_path().is_file());
    assert!(!staging.staging_path().exists());
    assert_eq!(fs::read(staging.product_path()).unwrap(), expected_product);
    assert_eq!(
        fs::read(scratch.path().join(owner.configuration().intent_name())).unwrap(),
        serde_json::to_vec(&expected_intent).unwrap()
    );

    let finalized = proposal.complete(receipt).unwrap().into_after_owner();
    assert_eq!(finalized.state_json()["finalized"], true);
    assert_eq!(
        finalized.state_json()["final_product_sha256"],
        hex_lower(&frame_sha256(&expected_product))
    );
}

#[test]
fn parent_directory_fsync_failure_after_rename_returns_no_receipt_or_done_state() {
    let scratch = ScratchDirectory::new().unwrap();
    let (owner, mut staging) = terminal_output(&scratch);
    let proposal = owner.prepare_finalize().unwrap();
    let expected_product = vec![0_u8; 12_288];
    let error = staging
        .finalize_with_parent_sync(&proposal, |_| {
            Err(io::Error::other("injected parent-directory fsync failure"))
        })
        .unwrap_err();

    assert_eq!(error.code(), "DURABLE_OUTPUT_ERROR");
    assert_eq!(error.phase(), OutputIoPhase::FinalizeParentSync);
    assert_eq!(error.failure(), OutputIoFailure::ParentFsyncAfterRename);
    assert!(staging.is_exposure_uncertain());
    assert!(!staging.staging_path().exists());
    assert_eq!(fs::read(staging.product_path()).unwrap(), expected_product);
    assert!(
        scratch
            .path()
            .join(owner.configuration().intent_name())
            .is_file()
    );
}

#[test]
fn public_finalize_reports_the_local_parent_sync_outcome_without_false_done() {
    let scratch = ScratchDirectory::new().unwrap();
    let (owner, mut staging) = terminal_output(&scratch);
    let proposal = owner.prepare_finalize().unwrap();
    let result = staging.finalize(&proposal);

    match result {
        Ok(receipt) => {
            let finalized = proposal.complete(receipt).unwrap().into_after_owner();
            assert_eq!(finalized.state_json()["finalized"], true);
            assert!(staging.product_path().is_file());
            assert!(!staging.staging_path().exists());
        }
        Err(error) => {
            assert_eq!(error.code(), "DURABLE_OUTPUT_ERROR");
            assert_eq!(error.phase(), OutputIoPhase::FinalizeParentSync);
            assert_eq!(error.failure(), OutputIoFailure::ParentFsyncAfterRename);
            assert!(staging.is_exposure_uncertain());
            assert!(staging.product_path().is_file());
            assert!(!staging.staging_path().exists());
        }
    }
}

#[test]
fn finalize_preflight_never_overwrites_an_existing_product() {
    let scratch = ScratchDirectory::new().unwrap();
    let (owner, mut staging) = terminal_output(&scratch);
    let proposal = owner.prepare_finalize().unwrap();
    let product_path = staging.product_path();
    fs::write(&product_path, b"existing product").unwrap();

    let error = staging
        .finalize_with_parent_sync(&proposal, |_| Ok(()))
        .unwrap_err();
    assert_eq!(error.phase(), OutputIoPhase::FinalizePreflight);
    assert_eq!(error.failure(), OutputIoFailure::FinalPathExists);
    assert_eq!(fs::read(product_path).unwrap(), b"existing product");
    assert!(staging.staging_path().is_file());
    assert!(
        !scratch
            .path()
            .join(owner.configuration().intent_name())
            .exists()
    );
}

fn hex_lower(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
