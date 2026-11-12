use super::*;
use jocforge_domain::joc::emdf::metadata_owner::test_support::Flat7xReceiptGateMetadataTestInputs;
use jocforge_domain::publication::test_support::Flat7xAuAppendTestFixture;
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

#[cfg(windows)]
use std::process::Command;

const SYNTHETIC_PROVIDER: &[u8] = include_bytes!(
    "../../../jocforge-domain/tests/fixtures/output_owner/synthetic-output-provider.json"
);
const FLAT7X_RECEIPT_PAIR_VECTORS: &[u8] = include_bytes!(
    "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/FLAT7X-F15-RECEIPT-PAIR-INPUT-ONLY-V1/INPUT-ONLY-VECTORS.json"
);
const METADATA_OWNER_VECTORS: &[u8] = include_bytes!(
    "../../../../crates/jocforge-domain/tests/fixtures/metadata_owner/METADATA-OWNER-VECTORS.json"
);
const METADATA_PUBLIC_PROVIDER: &[u8] = include_bytes!(
    "../../../../crates/jocforge-domain/tests/fixtures/metadata_owner/metadata-public-provider.json"
);
const METADATA_WRAPPER_CONFIG: &[u8] = include_bytes!(
    "../../../../crates/jocforge-domain/tests/fixtures/metadata_owner/metadata-wrapper-config.json"
);
const KEYLESS_INTEGRITY_PROVIDER: &[u8] = include_bytes!(
    "../../../../crates/jocforge-domain/tests/fixtures/integrity_owner/integrity-public-provider.json"
);
const FRAME_BYTES: usize = 4_096;
const FRAME_BYTES_U32: u32 = FRAME_BYTES as u32;

struct ScratchDirectory(PathBuf);

fn flat7x_test_fixture() -> Flat7xAuAppendTestFixture {
    Flat7xAuAppendTestFixture::positive_pair(Flat7xReceiptGateMetadataTestInputs {
        f15_pair_vectors: FLAT7X_RECEIPT_PAIR_VECTORS,
        metadata_owner_vectors: METADATA_OWNER_VECTORS,
        metadata_public_provider: METADATA_PUBLIC_PROVIDER,
        metadata_wrapper_config: METADATA_WRAPPER_CONFIG,
        keyless_integrity_provider: KEYLESS_INTEGRITY_PROVIDER,
    })
}

impl ScratchDirectory {
    fn new() -> io::Result<Self> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        #[cfg(windows)]
        // Keep Win32 rename tests on the workspace volume; the process temp
        // directory is sandbox-backed and does not exercise the host NTFS path.
        let base_directory = std::env::current_exe()?
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| io::Error::other("test executable has no parent directory"))?;
        #[cfg(not(windows))]
        let base_directory = std::env::temp_dir();
        let path = base_directory.join(format!("jocforge-output-{}-{nonce}", std::process::id()));
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
    output_owner_for_rate(4_096, "synthetic-0", 4_608)
}

fn output_owner_for_rate(frame_bytes: usize, attempt_id: &str, source_samples: u64) -> OutputOwner {
    let provider: Value = serde_json::from_slice(SYNTHETIC_PROVIDER).unwrap();
    let provider_hash = Sha256::digest(
        format!("{}:{frame_bytes}", provider["profile_id"].as_str().unwrap()).as_bytes(),
    )
    .into();
    OutputOwner::cold(
        OutputConfiguration::new(
            OutputProvider {
                profile_id: provider["profile_id"].as_str().unwrap().to_owned(),
                provider_sha256: provider_hash,
                frame_bytes: frame_bytes as u32,
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
            source_samples,
        )
        .unwrap(),
    )
}

fn flat7x_output_owner(attempt_id: &str) -> OutputOwner {
    output_owner_for_rate(FRAME_BYTES, attempt_id, 1_536)
}

fn advance_suppressed_sources(owner: OutputOwner) -> OutputOwner {
    let frame = [0_u8; FRAME_BYTES];
    advance_suppressed_sources_for_frame(owner, &frame)
}

fn advance_suppressed_sources_for_frame(mut owner: OutputOwner, frame: &[u8]) -> OutputOwner {
    let frame_sha = frame_sha256(frame);
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
fn flat7x_pair_append_writes_and_reads_back_one_complete_6656_byte_au() {
    let scratch = ScratchDirectory::new().unwrap();
    let fixture = flat7x_test_fixture();
    let output = flat7x_output_owner(fixture.attempt_id());
    let mut staging = AtomicOutputStaging::create(scratch.path(), output.configuration()).unwrap();
    let proposal = fixture.append_proposal(fixture.attempt_id()).unwrap();

    let after = staging.append_flat7x_au(proposal).unwrap();

    assert_eq!(after.receipt().bytes_written(), 6_656);
    assert_eq!(after.receipt().offset(), 0);
    assert_eq!(after.receipt().frame_crc_residuals(), (0, 0));
    assert!(after.receipt().file_fsync());
    assert!(after.receipt().durable());
    assert_eq!(
        fs::read(staging.staging_path()).unwrap(),
        fixture.expected_bytes()
    );
    assert!(!staging.is_abandoned());
}

#[test]
fn flat7x_pair_attempt_mismatch_retires_pair_and_abandons_atomic_session() {
    let scratch = ScratchDirectory::new().unwrap();
    let fixture = flat7x_test_fixture();
    let output = flat7x_output_owner("atomic-good-attempt");
    let mut staging = AtomicOutputStaging::create(scratch.path(), output.configuration()).unwrap();

    let failure = staging
        .append_flat7x_au(
            fixture
                .append_proposal("pair-wrong-attempt")
                .expect("synthetic F15 pair proposal"),
        )
        .err()
        .unwrap_or_else(|| panic!("wrong attempt must be rejected"));
    assert!(matches!(
        failure,
        Flat7xAtomicAppendFailure::AttemptMismatch(_)
    ));
    assert!(staging.is_abandoned());
    assert_eq!(fs::metadata(staging.staging_path()).unwrap().len(), 0);

    let second = fixture
        .append_proposal("atomic-good-attempt")
        .expect("synthetic F15 pair proposal");
    let failure = staging
        .append_flat7x_au(second)
        .err()
        .unwrap_or_else(|| panic!("abandoned session must reject another pair"));
    assert!(matches!(
        failure,
        Flat7xAtomicAppendFailure::SessionAbandoned(_)
    ));
    assert!(staging.is_abandoned());
}

#[test]
fn flat7x_pair_wrong_staging_offset_retires_and_rejects_following_append() {
    let scratch = ScratchDirectory::new().unwrap();
    let fixture = flat7x_test_fixture();
    let attempt_id = fixture.attempt_id();
    let mut output = flat7x_output_owner(attempt_id);
    let mut staging = AtomicOutputStaging::create(scratch.path(), output.configuration()).unwrap();

    let frame = [0_u8; FRAME_BYTES];
    let frame_hash = frame_sha256(&frame);
    output = advance_suppressed_sources_for_frame(output, &frame);
    let proposal = output
        .prepare_source(
            OutputSourceIdentity::new(
                CodecSourceIndex::new(2),
                JocDispatchIndex::new(3),
                Some(PublishedAuIndex::new(0)),
            ),
            &frame,
            &frame_hash,
        )
        .unwrap();
    let OutputSourceProposal::Append(proposal) = proposal else {
        panic!("source two must append the ordinary test frame")
    };
    let receipt = staging.append(&proposal).unwrap();
    output = proposal.complete(receipt).unwrap().into_after_owner();
    assert_eq!(output.state_json()["durable_length"], FRAME_BYTES_U32);

    let failure = staging
        .append_flat7x_au(fixture.append_proposal(attempt_id).unwrap())
        .err()
        .unwrap_or_else(|| panic!("wrong staging offset must retire the pair"));
    assert!(matches!(failure, Flat7xAtomicAppendFailure::Append(_)));
    assert!(staging.is_abandoned());
    assert_eq!(
        fs::metadata(staging.staging_path()).unwrap().len(),
        FRAME_BYTES as u64
    );

    let second = fixture.append_proposal(attempt_id).unwrap();
    let failure = staging
        .append_flat7x_au(second)
        .err()
        .unwrap_or_else(|| panic!("abandoned session must reject another pair"));
    assert!(matches!(
        failure,
        Flat7xAtomicAppendFailure::SessionAbandoned(_)
    ));
}

#[test]
fn selected_append_reads_back_and_finalizes_each_supported_frame_extent() {
    let rates = [
        (384, 1_536),
        (448, 1_792),
        (576, 2_304),
        (640, 2_560),
        (768, 3_072),
        (1_024, 4_096),
    ];

    for (rate, frame_bytes) in rates {
        let scratch = ScratchDirectory::new().unwrap();
        let frame = vec![0_u8; frame_bytes];
        let owner = advance_suppressed_sources_for_frame(
            output_owner_for_rate(frame_bytes, &format!("rate-{rate}"), 1_536),
            &frame,
        );
        let mut staging =
            AtomicOutputStaging::create(scratch.path(), owner.configuration()).unwrap();
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
            panic!("source two publishes AU zero");
        };
        let receipt = staging.append(&proposal).unwrap();
        assert_eq!(receipt.bytes_written(), frame_bytes as u32);
        assert_eq!(
            receipt.write_calls().iter().sum::<u64>(),
            frame_bytes as u64
        );
        let owner = proposal.complete(receipt).unwrap().into_after_owner();
        let finalize = owner.prepare_finalize().unwrap();
        staging
            .finalize_with_parent_sync(&finalize, |_| Ok(()))
            .unwrap();
        assert_eq!(fs::read(scratch.path().join("product.ec3")).unwrap(), frame);
    }
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

#[cfg(windows)]
const RENAME_COLLISION_DESTINATION_ENV: &str = "JOCFORGE_RENAME_COLLISION_DESTINATION";

#[cfg(windows)]
#[test]
fn windows_create_collision_destination_child() -> io::Result<()> {
    let Some(destination) = std::env::var_os(RENAME_COLLISION_DESTINATION_ENV) else {
        return Ok(());
    };
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    file.write_all(b"created by competing process")?;
    file.sync_all()
}

#[cfg(windows)]
#[test]
fn windows_finalize_uses_atomic_no_replace_when_destination_appears_after_preflight()
-> io::Result<()> {
    let scratch = ScratchDirectory::new()?;
    let (owner, mut staging) = terminal_output(&scratch);
    let proposal = owner.prepare_finalize().unwrap();
    let destination = staging.product_path();
    let staging_path = staging.staging_path();
    let expected_staging = vec![0_u8; 12_288];

    let error = staging
        .finalize_with_parent_sync_and_before_rename(
            &proposal,
            || {
                let status = Command::new(std::env::current_exe()?)
                    .arg("--exact")
                    .arg("atomic_output::tests::windows_create_collision_destination_child")
                    .arg("--nocapture")
                    .env(RENAME_COLLISION_DESTINATION_ENV, &destination)
                    .status()?;
                if status.success() {
                    Ok(())
                } else {
                    Err(io::Error::other(
                        "competing process could not create the final destination",
                    ))
                }
            },
            |_| Ok(()),
        )
        .unwrap_err();

    assert_eq!(error.phase(), OutputIoPhase::FinalizeRename);
    assert_eq!(error.failure(), OutputIoFailure::FinalPathExistsAtRename);
    assert_eq!(error.io_kind(), Some(io::ErrorKind::AlreadyExists));
    assert!(staging.is_abandoned());
    assert_eq!(fs::read(&staging_path)?, expected_staging);
    assert_eq!(
        fs::read(&destination)?,
        b"created by competing process".to_vec()
    );
    Ok(())
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
