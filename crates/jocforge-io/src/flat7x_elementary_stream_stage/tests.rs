// pattern: Imperative Shell
//! Synthetic staging-file tests for the Flat7x elementary-stream IO boundary.

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use jocforge_domain::{
    codec::elementary_stream_composer::{
        Flat7xElementaryStreamExtent, measure_flat7x_elementary_stream,
    },
    profile::Flat7xI0D0TargetPlan,
    publication::{Flat7xElementaryStreamStageProposal, Flat7xElementaryStreamStageProposalError},
};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{
    Flat7xElementaryStreamStageIoFailure, Flat7xElementaryStreamStageIoPhase,
    stage_flat7x_elementary_stream_to_file, write_and_readback_staging,
};

const INPUT_ONLY_VECTORS: &str =
    include_str!("../../tests/fixtures/flat7x_elementary_stream_stage/INPUT-ONLY-VECTORS.json");

struct TemporaryOutputDirectory(PathBuf);

impl TemporaryOutputDirectory {
    fn create() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock follows UNIX epoch")
            .as_nanos();
        for suffix in 0..100_u32 {
            let path = std::env::temp_dir().join(format!(
                "jocforge-flat7x-stream-stage-{}-{unique}-{suffix}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("create isolated staging test directory: {error}"),
            }
        }
        panic!("could not allocate a unique staging test directory")
    }

    fn final_path(&self) -> PathBuf {
        self.0.join("output.ec3")
    }

    fn staging_path(&self) -> PathBuf {
        self.0.join("output.ec3.partial")
    }

    fn create_staging_file(&self) -> File {
        OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(self.staging_path())
            .expect("caller creates a unique temporary staging destination")
    }
}

impl Drop for TemporaryOutputDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vectors() -> Value {
    let parsed: Value = serde_json::from_str(INPUT_ONLY_VECTORS).unwrap();
    assert_eq!(parsed["status"], "SYNTHETIC_NOT_REAL_OUTPUT");
    assert!(!parsed["contains_audio_or_frame_bytes"].as_bool().unwrap());
    parsed
}

fn case<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["id"] == id)
        .unwrap()
}

fn synthetic_stream(vector: &Value) -> Vec<u8> {
    let count = vector["access_unit_count"].as_u64().unwrap() as usize;
    let bytes_per_au = vector["bytes_per_access_unit"].as_u64().unwrap() as usize;
    let fill = u8::from_str_radix(vector["stream_test_fill_hex"].as_str().unwrap(), 16).unwrap();
    vec![fill; count * bytes_per_au]
}

fn extent_for(count: usize) -> Flat7xElementaryStreamExtent {
    measure_flat7x_elementary_stream(Flat7xI0D0TargetPlan::approved_contract(), count).unwrap()
}

fn proposal(bytes: &[u8], count: usize) -> Flat7xElementaryStreamStageProposal<'_> {
    Flat7xElementaryStreamStageProposal::new(
        Flat7xI0D0TargetPlan::approved_contract(),
        bytes,
        extent_for(count),
    )
    .unwrap()
}

#[test]
fn stages_exact_stream_to_unique_temp_file_and_preserves_existing_final_destination() {
    let vectors = vectors();
    let vector = case(&vectors, "stage-two-access-unit-stream");
    let bytes = synthetic_stream(vector);
    let sentinel = hex_bytes(vector["final_sentinel_hex"].as_str().unwrap());
    let temporary = TemporaryOutputDirectory::create();
    fs::write(temporary.final_path(), &sentinel).unwrap();
    let mut staging = temporary.create_staging_file();

    let receipt =
        stage_flat7x_elementary_stream_to_file(&mut staging, proposal(&bytes, 2)).unwrap();
    drop(staging);

    assert_eq!(
        receipt.bytes_written(),
        vector["expected_stream_bytes"].as_u64().unwrap() as u32
    );
    assert_eq!(receipt.extent().access_unit_count(), 2);
    assert!(receipt.write_call_count() > 0);
    assert!(receipt.file_flush_completed());
    assert!(receipt.file_sync_completed());
    assert!(!receipt.production_allowed());
    assert!(!receipt.final_exposure_allowed());
    assert_eq!(&fs::read(temporary.staging_path()).unwrap(), &bytes);
    assert_eq!(fs::read(temporary.final_path()).unwrap(), sentinel);
    assert_eq!(
        receipt.readback_sha256(),
        &<[u8; 32]>::from(Sha256::digest(&bytes))
    );
}

#[test]
fn proposal_extent_and_nonempty_temp_destination_reject_without_changing_final_file() {
    let vectors = vectors();
    let success_vector = case(&vectors, "stage-two-access-unit-stream");
    let bytes = synthetic_stream(success_vector);
    let temporary = TemporaryOutputDirectory::create();
    let sentinel = hex_bytes(success_vector["final_sentinel_hex"].as_str().unwrap());
    fs::write(temporary.final_path(), &sentinel).unwrap();
    let mismatch = case(&vectors, "reject-stream-byte-extent-mismatch");
    let short = &bytes[..mismatch["declared_stream_bytes"].as_u64().unwrap() as usize];
    assert_eq!(
        Flat7xElementaryStreamStageProposal::new(
            Flat7xI0D0TargetPlan::approved_contract(),
            short,
            extent_for(2),
        )
        .unwrap_err(),
        Flat7xElementaryStreamStageProposalError::ByteLengthMismatch {
            expected: 13_312,
            actual: 13_311,
        }
    );
    assert_eq!(fs::read(temporary.final_path()).unwrap(), sentinel);

    let mut staging = temporary.create_staging_file();
    let existing = hex_bytes(
        case(&vectors, "reject-nonempty-temporary-destination")["existing_temporary_bytes_hex"]
            .as_str()
            .unwrap(),
    );
    staging.write_all(&existing).unwrap();
    staging.sync_all().unwrap();

    let error =
        stage_flat7x_elementary_stream_to_file(&mut staging, proposal(&bytes, 2)).unwrap_err();
    assert_eq!(error.phase(), Flat7xElementaryStreamStageIoPhase::Preflight);
    assert_eq!(
        error.failure(),
        Flat7xElementaryStreamStageIoFailure::StagingLengthMismatch
    );
    drop(staging);
    assert_eq!(fs::read(temporary.staging_path()).unwrap(), existing);
    assert_eq!(fs::read(temporary.final_path()).unwrap(), sentinel);
}

#[test]
fn helper_completes_short_writes_and_returns_exact_readback_receipt() {
    let vectors = vectors();
    let vector = case(&vectors, "stage-two-access-unit-stream");
    let bytes = synthetic_stream(vector);
    let write_case = case(&vectors, "complete-short-writes");
    let mut file = ScriptedStagingFile::new(
        usize::try_from(write_case["maximum_bytes_per_write_call"].as_u64().unwrap()).unwrap(),
    );
    let evidence =
        write_and_readback_staging(&mut file, &bytes, extent_for(2), |_| Ok(())).unwrap();

    assert!(evidence.write_call_count > 1);
    assert_eq!(evidence.bytes_written, bytes.len() as u32);
    let expected_sha256: [u8; 32] = Sha256::digest(&bytes).into();
    assert_eq!(evidence.readback_sha256, expected_sha256);
    assert_eq!(file.bytes, bytes);
}

#[test]
fn zero_progress_flush_sync_and_readback_failures_return_no_receipt() {
    let vectors = vectors();
    let vector = case(&vectors, "stage-two-access-unit-stream");
    let bytes = synthetic_stream(vector);
    let temporary = TemporaryOutputDirectory::create();
    let sentinel = hex_bytes(vector["final_sentinel_hex"].as_str().unwrap());
    fs::write(temporary.final_path(), &sentinel).unwrap();
    let zero_case = case(&vectors, "reject-zero-progress-after-partial-stage");
    let mut short_then_zero = ScriptedStagingFile::with_zero_after(
        usize::try_from(zero_case["maximum_bytes_per_write_call"].as_u64().unwrap()).unwrap(),
        usize::try_from(zero_case["zero_progress_after_bytes"].as_u64().unwrap()).unwrap(),
    );
    let error = write_and_readback_staging(&mut short_then_zero, &bytes, extent_for(2), |_| Ok(()))
        .unwrap_err();
    assert_eq!(error.phase(), Flat7xElementaryStreamStageIoPhase::Write);
    assert_eq!(
        error.failure(),
        Flat7xElementaryStreamStageIoFailure::WriteZero
    );
    assert_eq!(short_then_zero.bytes.len(), 4_096);

    let mut flush_failure = ScriptedStagingFile::with_flush_failure(usize::MAX);
    let flush_error =
        write_and_readback_staging(&mut flush_failure, &bytes, extent_for(2), |_| Ok(()))
            .unwrap_err();
    assert_eq!(
        flush_error.phase(),
        Flat7xElementaryStreamStageIoPhase::Flush
    );
    assert_eq!(
        flush_error.failure(),
        Flat7xElementaryStreamStageIoFailure::Io
    );

    let mut sync_failure = ScriptedStagingFile::new(usize::MAX);
    let sync_error = write_and_readback_staging(&mut sync_failure, &bytes, extent_for(2), |_| {
        Err(io::Error::other("synthetic file sync failure"))
    })
    .unwrap_err();
    assert_eq!(
        sync_error.phase(),
        Flat7xElementaryStreamStageIoPhase::FileSync
    );
    assert_eq!(
        sync_error.failure(),
        Flat7xElementaryStreamStageIoFailure::Io
    );

    let mut readback_failure = ScriptedStagingFile::with_corrupt_readback(usize::MAX);
    let readback_error =
        write_and_readback_staging(&mut readback_failure, &bytes, extent_for(2), |_| Ok(()))
            .unwrap_err();
    assert_eq!(
        readback_error.phase(),
        Flat7xElementaryStreamStageIoPhase::Readback
    );
    assert_eq!(
        readback_error.failure(),
        Flat7xElementaryStreamStageIoFailure::ReadbackMismatch
    );
    assert_eq!(fs::read(temporary.final_path()).unwrap(), sentinel);
}

fn hex_bytes(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(text, 16).unwrap()
        })
        .collect()
}

struct ScriptedStagingFile {
    bytes: Vec<u8>,
    cursor: u64,
    maximum_write: usize,
    zero_after: Option<usize>,
    bytes_written: usize,
    fail_flush: bool,
    corrupt_readback: bool,
    readback_started: bool,
}

impl ScriptedStagingFile {
    fn new(maximum_write: usize) -> Self {
        Self {
            bytes: Vec::new(),
            cursor: 0,
            maximum_write,
            zero_after: None,
            bytes_written: 0,
            fail_flush: false,
            corrupt_readback: false,
            readback_started: false,
        }
    }

    fn with_zero_after(maximum_write: usize, zero_after: usize) -> Self {
        Self {
            zero_after: Some(zero_after),
            ..Self::new(maximum_write)
        }
    }

    fn with_flush_failure(maximum_write: usize) -> Self {
        Self {
            fail_flush: true,
            ..Self::new(maximum_write)
        }
    }

    fn with_corrupt_readback(maximum_write: usize) -> Self {
        Self {
            corrupt_readback: true,
            ..Self::new(maximum_write)
        }
    }
}

impl Read for ScriptedStagingFile {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let start = usize::try_from(self.cursor)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "cursor overflow"))?;
        let length = buffer.len().min(self.bytes.len().saturating_sub(start));
        buffer[..length].copy_from_slice(&self.bytes[start..start + length]);
        if self.corrupt_readback && self.readback_started && length > 0 {
            buffer[0] ^= 1;
            self.readback_started = false;
        }
        self.cursor += u64::try_from(length).unwrap();
        Ok(length)
    }
}

impl Write for ScriptedStagingFile {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self
            .zero_after
            .is_some_and(|limit| self.bytes_written >= limit)
        {
            return Ok(0);
        }
        let permitted_before_zero = self
            .zero_after
            .map_or(usize::MAX, |limit| limit.saturating_sub(self.bytes_written));
        let length = bytes
            .len()
            .min(self.maximum_write)
            .min(permitted_before_zero);
        if length == 0 {
            return Ok(0);
        }
        let start = usize::try_from(self.cursor)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "cursor overflow"))?;
        let end = start
            .checked_add(length)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "extent overflow"))?;
        if end > self.bytes.len() {
            self.bytes.resize(end, 0);
        }
        self.bytes[start..end].copy_from_slice(&bytes[..length]);
        self.cursor = u64::try_from(end).unwrap();
        self.bytes_written += length;
        Ok(length)
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.fail_flush {
            return Err(io::Error::other("synthetic flush failure"));
        }
        Ok(())
    }
}

impl Seek for ScriptedStagingFile {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        let next = match position {
            SeekFrom::Start(offset) => i128::from(offset),
            SeekFrom::End(offset) => i128::try_from(self.bytes.len()).unwrap() + i128::from(offset),
            SeekFrom::Current(offset) => i128::from(self.cursor) + i128::from(offset),
        };
        if next < 0 || next > i128::from(u64::MAX) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "seek out of range",
            ));
        }
        self.cursor = u64::try_from(next).unwrap();
        if matches!(position, SeekFrom::Start(0)) && self.bytes_written > 0 {
            self.readback_started = true;
        }
        Ok(self.cursor)
    }
}
