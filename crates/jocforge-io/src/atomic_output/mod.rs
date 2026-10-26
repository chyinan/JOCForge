// pattern: Imperative Shell
//! Local-model F15P staging writes for already validated source proposals.

#[cfg(test)]
mod tests;

use jocforge_domain::{
    integrity::crc16::eac3_crc16,
    publication::{
        OutputAppendProposal, OutputConfiguration, OutputDurableAppendReceipt,
        OutputFinalExposureReceipt, OutputFinalizeProposal,
    },
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Component, Path, PathBuf},
};

const FRAME_BYTES: usize = 4_096;
const FRAME_BYTES_U32: u32 = FRAME_BYTES as u32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputIoPhase {
    Setup,
    AppendPreflight,
    AppendWrite,
    AppendReadback,
    AppendFlush,
    AppendFileSync,
    FinalizePreflight,
    FinalizeStagingSync,
    FinalizeIntentWrite,
    FinalizeIntentSync,
    FinalizeRename,
    FinalizeParentSync,
}

impl OutputIoPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Setup => "setup/staging",
            Self::AppendPreflight => "append/preflight",
            Self::AppendWrite => "append/write",
            Self::AppendReadback => "append/readback",
            Self::AppendFlush => "append/flush",
            Self::AppendFileSync => "append/file-sync",
            Self::FinalizePreflight => "finalize/preflight",
            Self::FinalizeStagingSync => "finalize/staging-fsync",
            Self::FinalizeIntentWrite => "finalize/intent-write",
            Self::FinalizeIntentSync => "finalize/intent-fsync",
            Self::FinalizeRename => "finalize/rename",
            Self::FinalizeParentSync => "finalize/parent-sync",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputIoFailure {
    InvalidDirectory,
    InvalidName,
    FinalPathExists,
    StagingPathExists,
    SessionAbandoned,
    SessionFinalized,
    ProposalIdentity,
    StagingLength,
    Io,
    ZeroProgress,
    WriteOverrun,
    ReadbackMismatch,
    CrcMismatch,
    DurableReceipt,
    FinalizeProposalIdentity,
    FinalizeLength,
    IntentPathExists,
    FinalPathExistsAtRename,
    Rename,
    ParentFsyncAfterRename,
}

#[derive(Debug)]
pub struct OutputIoError {
    phase: OutputIoPhase,
    failure: OutputIoFailure,
    io_kind: Option<io::ErrorKind>,
}

impl OutputIoError {
    const fn new(phase: OutputIoPhase, failure: OutputIoFailure) -> Self {
        Self {
            phase,
            failure,
            io_kind: None,
        }
    }

    fn from_io(phase: OutputIoPhase, error: &io::Error) -> Self {
        Self {
            phase,
            failure: OutputIoFailure::Io,
            io_kind: Some(error.kind()),
        }
    }

    pub const fn code(&self) -> &'static str {
        "DURABLE_OUTPUT_ERROR"
    }

    pub const fn phase(&self) -> OutputIoPhase {
        self.phase
    }

    pub const fn failure(&self) -> OutputIoFailure {
        self.failure
    }

    pub const fn io_kind(&self) -> Option<io::ErrorKind> {
        self.io_kind
    }
}

impl std::fmt::Display for OutputIoError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}:{}:{:?}",
            self.code(),
            self.phase.as_str(),
            self.failure
        )
    }
}

impl std::error::Error for OutputIoError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SessionState {
    Open,
    Abandoned,
    ExposureUncertain,
    Finalized,
}

/// Attempt-scoped staging file. It never commits F15P state and never deletes
/// an abandoned file; the parent source transaction owns RETIRED/restart policy.
#[derive(Debug)]
pub struct AtomicOutputStaging {
    directory: PathBuf,
    staging_name: String,
    intent_name: String,
    product_name: String,
    attempt_id: String,
    file: Option<File>,
    state: SessionState,
}

impl AtomicOutputStaging {
    /// Creates the selected attempt staging file with create-new semantics.
    pub fn create(
        directory: &Path,
        configuration: &OutputConfiguration,
    ) -> Result<Self, OutputIoError> {
        let metadata = fs::metadata(directory)
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::Setup, &error))?;
        if !metadata.is_dir()
            || !is_single_component(configuration.staging_name())
            || !is_single_component(configuration.intent_name())
            || !is_single_component(configuration.product_name())
        {
            let failure = if metadata.is_dir() {
                OutputIoFailure::InvalidName
            } else {
                OutputIoFailure::InvalidDirectory
            };
            return Err(OutputIoError::new(OutputIoPhase::Setup, failure));
        }

        let product_path = directory.join(configuration.product_name());
        if path_exists(&product_path)
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::Setup, &error))?
        {
            return Err(OutputIoError::new(
                OutputIoPhase::Setup,
                OutputIoFailure::FinalPathExists,
            ));
        }

        let staging_path = directory.join(configuration.staging_name());
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(staging_path)
            .map_err(|error| {
                let failure = if error.kind() == io::ErrorKind::AlreadyExists {
                    OutputIoFailure::StagingPathExists
                } else {
                    OutputIoFailure::Io
                };
                OutputIoError {
                    phase: OutputIoPhase::Setup,
                    failure,
                    io_kind: Some(error.kind()),
                }
            })?;

        Ok(Self {
            directory: directory.to_path_buf(),
            staging_name: configuration.staging_name().to_owned(),
            intent_name: configuration.intent_name().to_owned(),
            product_name: configuration.product_name().to_owned(),
            attempt_id: configuration.attempt_id().to_owned(),
            file: Some(file),
            state: SessionState::Open,
        })
    }

    /// Writes one exact source proposal. Success means complete readback and
    /// file fsync have finished; it does not install any parent owner state.
    pub fn append(
        &mut self,
        proposal: &OutputAppendProposal,
    ) -> Result<OutputDurableAppendReceipt, OutputIoError> {
        if self.state == SessionState::Abandoned {
            return Err(OutputIoError::new(
                OutputIoPhase::AppendPreflight,
                OutputIoFailure::SessionAbandoned,
            ));
        }
        if self.state == SessionState::Finalized {
            return Err(OutputIoError::new(
                OutputIoPhase::AppendPreflight,
                OutputIoFailure::SessionFinalized,
            ));
        }
        if self.state == SessionState::ExposureUncertain {
            return Err(OutputIoError::new(
                OutputIoPhase::AppendPreflight,
                OutputIoFailure::ParentFsyncAfterRename,
            ));
        }

        let result = self.append_inner(proposal);
        if result.is_err() {
            self.state = SessionState::Abandoned;
        }
        result
    }

    /// Finalizes a terminal F15P proposal. A parent-directory fsync failure
    /// after rename returns no receipt and leaves the visible path untouched.
    pub fn finalize(
        &mut self,
        proposal: &OutputFinalizeProposal,
    ) -> Result<OutputFinalExposureReceipt, OutputIoError> {
        self.finalize_with_parent_sync(proposal, sync_parent_directory)
    }

    fn finalize_with_parent_sync<F>(
        &mut self,
        proposal: &OutputFinalizeProposal,
        parent_sync: F,
    ) -> Result<OutputFinalExposureReceipt, OutputIoError>
    where
        F: FnOnce(&Path) -> io::Result<()>,
    {
        match self.state {
            SessionState::Abandoned => {
                return Err(OutputIoError::new(
                    OutputIoPhase::FinalizePreflight,
                    OutputIoFailure::SessionAbandoned,
                ));
            }
            SessionState::ExposureUncertain => {
                return Err(OutputIoError::new(
                    OutputIoPhase::FinalizePreflight,
                    OutputIoFailure::ParentFsyncAfterRename,
                ));
            }
            SessionState::Finalized => {
                return Err(OutputIoError::new(
                    OutputIoPhase::FinalizePreflight,
                    OutputIoFailure::SessionFinalized,
                ));
            }
            SessionState::Open => {}
        }

        let result = self.finalize_inner(proposal, parent_sync);
        match result {
            Ok(receipt) => {
                self.state = SessionState::Finalized;
                Ok(receipt)
            }
            Err(error) => {
                self.state = if error.failure() == OutputIoFailure::ParentFsyncAfterRename {
                    SessionState::ExposureUncertain
                } else {
                    SessionState::Abandoned
                };
                Err(error)
            }
        }
    }

    fn append_inner(
        &mut self,
        proposal: &OutputAppendProposal,
    ) -> Result<OutputDurableAppendReceipt, OutputIoError> {
        if proposal.attempt_id() != self.attempt_id || proposal.staging_name() != self.staging_name
        {
            return Err(OutputIoError::new(
                OutputIoPhase::AppendPreflight,
                OutputIoFailure::ProposalIdentity,
            ));
        }
        let file = self.file.as_mut().ok_or_else(|| {
            OutputIoError::new(
                OutputIoPhase::AppendPreflight,
                OutputIoFailure::SessionFinalized,
            )
        })?;
        let initial_length = file
            .metadata()
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::AppendPreflight, &error))?
            .len();
        if initial_length != proposal.expected_offset() {
            return Err(OutputIoError::new(
                OutputIoPhase::AppendPreflight,
                OutputIoFailure::StagingLength,
            ));
        }

        file.seek(SeekFrom::Start(proposal.expected_offset()))
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::AppendWrite, &error))?;
        let write_calls = match write_all_progress(file, proposal.frame_bytes()) {
            Ok(calls) => calls,
            Err(WriteProgressError::ZeroProgress) => {
                return Err(OutputIoError::new(
                    OutputIoPhase::AppendWrite,
                    OutputIoFailure::ZeroProgress,
                ));
            }
            Err(WriteProgressError::Overrun) => {
                return Err(OutputIoError::new(
                    OutputIoPhase::AppendWrite,
                    OutputIoFailure::WriteOverrun,
                ));
            }
            Err(WriteProgressError::Io(error)) => {
                return Err(OutputIoError::from_io(OutputIoPhase::AppendWrite, &error));
            }
        };

        let expected_end = proposal
            .expected_offset()
            .checked_add(u64::from(FRAME_BYTES_U32))
            .ok_or_else(|| {
                OutputIoError::new(OutputIoPhase::AppendWrite, OutputIoFailure::WriteOverrun)
            })?;
        let resulting_length = file
            .metadata()
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::AppendReadback, &error))?
            .len();
        if resulting_length != expected_end {
            return Err(OutputIoError::new(
                OutputIoPhase::AppendReadback,
                OutputIoFailure::StagingLength,
            ));
        }

        file.seek(SeekFrom::Start(proposal.expected_offset()))
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::AppendReadback, &error))?;
        let mut readback = [0_u8; FRAME_BYTES];
        file.read_exact(&mut readback)
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::AppendReadback, &error))?;
        if readback != *proposal.frame_bytes() || sha256(&readback) != *proposal.frame_sha256() {
            return Err(OutputIoError::new(
                OutputIoPhase::AppendReadback,
                OutputIoFailure::ReadbackMismatch,
            ));
        }
        let crc_residual = eac3_crc16(&readback[2..]);
        if crc_residual != 0 {
            return Err(OutputIoError::new(
                OutputIoPhase::AppendReadback,
                OutputIoFailure::CrcMismatch,
            ));
        }
        file.flush()
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::AppendFlush, &error))?;
        file.sync_all()
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::AppendFileSync, &error))?;

        proposal
            .record_durable_io_success(write_calls, sha256(&readback), crc_residual)
            .map_err(|_| {
                OutputIoError::new(
                    OutputIoPhase::AppendFileSync,
                    OutputIoFailure::DurableReceipt,
                )
            })
    }

    fn finalize_inner<F>(
        &mut self,
        proposal: &OutputFinalizeProposal,
        parent_sync: F,
    ) -> Result<OutputFinalExposureReceipt, OutputIoError>
    where
        F: FnOnce(&Path) -> io::Result<()>,
    {
        if proposal.attempt_id() != self.attempt_id
            || proposal.staging_name() != self.staging_name
            || proposal.product_name() != self.product_name
        {
            return Err(OutputIoError::new(
                OutputIoPhase::FinalizePreflight,
                OutputIoFailure::FinalizeProposalIdentity,
            ));
        }

        let staging_path = self.directory.join(&self.staging_name);
        let intent_path = self.directory.join(&self.intent_name);
        let product_path = self.directory.join(&self.product_name);
        if path_exists(&product_path)
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::FinalizePreflight, &error))?
        {
            return Err(OutputIoError::new(
                OutputIoPhase::FinalizePreflight,
                OutputIoFailure::FinalPathExists,
            ));
        }

        // Reopen the staging artifact and verify the exact terminal length and
        // complete product digest before any final-path namespace operation.
        drop(self.file.take());
        let mut staging_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&staging_path)
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::FinalizePreflight, &error))?;
        let staging_length = staging_file
            .metadata()
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::FinalizePreflight, &error))?
            .len();
        if staging_length != proposal.expected_length() {
            return Err(OutputIoError::new(
                OutputIoPhase::FinalizePreflight,
                OutputIoFailure::FinalizeLength,
            ));
        }
        let product_sha256 = hash_file_exact(&mut staging_file, proposal.expected_length())?;
        staging_file
            .flush()
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::FinalizeStagingSync, &error))?;
        staging_file
            .sync_all()
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::FinalizeStagingSync, &error))?;
        drop(staging_file);

        let intent = json!({
            "attempt_id": proposal.attempt_id(),
            "product_length": proposal.expected_length(),
            "product_sha256": hex_lower(&product_sha256),
            "state_sha256": hex_lower(proposal.state_sha256()),
        });
        // serde_json::Value uses sorted map keys, matching the F15P canonical
        // finalize-intent encoding. No BOM or trailing newline is written.
        let intent_bytes = serde_json::to_vec(&intent).map_err(|error| {
            OutputIoError::from_io(OutputIoPhase::FinalizeIntentWrite, &io::Error::other(error))
        })?;
        let mut intent_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&intent_path)
            .map_err(|error| {
                let failure = if error.kind() == io::ErrorKind::AlreadyExists {
                    OutputIoFailure::IntentPathExists
                } else {
                    OutputIoFailure::Io
                };
                OutputIoError {
                    phase: OutputIoPhase::FinalizeIntentWrite,
                    failure,
                    io_kind: Some(error.kind()),
                }
            })?;
        intent_file
            .write_all(&intent_bytes)
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::FinalizeIntentWrite, &error))?;
        intent_file
            .flush()
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::FinalizeIntentSync, &error))?;
        intent_file
            .sync_all()
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::FinalizeIntentSync, &error))?;
        drop(intent_file);

        // This is a selected no-overwrite precondition. The local model makes
        // no cross-process/filesystem atomicity claim for a concurrent creator.
        if path_exists(&product_path)
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::FinalizeRename, &error))?
        {
            return Err(OutputIoError::new(
                OutputIoPhase::FinalizeRename,
                OutputIoFailure::FinalPathExistsAtRename,
            ));
        }
        fs::rename(&staging_path, &product_path).map_err(|error| OutputIoError {
            phase: OutputIoPhase::FinalizeRename,
            failure: OutputIoFailure::Rename,
            io_kind: Some(error.kind()),
        })?;

        parent_sync(&self.directory).map_err(|error| OutputIoError {
            phase: OutputIoPhase::FinalizeParentSync,
            failure: OutputIoFailure::ParentFsyncAfterRename,
            io_kind: Some(error.kind()),
        })?;

        proposal
            .record_durable_io_success(product_sha256, staging_length)
            .map_err(|_| {
                OutputIoError::new(
                    OutputIoPhase::FinalizeParentSync,
                    OutputIoFailure::DurableReceipt,
                )
            })
    }

    pub fn staging_path(&self) -> PathBuf {
        self.directory.join(&self.staging_name)
    }

    pub fn product_path(&self) -> PathBuf {
        self.directory.join(&self.product_name)
    }

    pub fn is_abandoned(&self) -> bool {
        self.state == SessionState::Abandoned
    }

    pub fn is_exposure_uncertain(&self) -> bool {
        self.state == SessionState::ExposureUncertain
    }
}

#[derive(Debug)]
enum WriteProgressError {
    Io(io::Error),
    ZeroProgress,
    Overrun,
}

fn write_all_progress<W: Write>(
    writer: &mut W,
    bytes: &[u8],
) -> Result<Vec<u64>, WriteProgressError> {
    let mut written = 0_usize;
    let mut calls = Vec::new();
    while written < bytes.len() {
        let remaining = bytes.len() - written;
        let count = writer
            .write(&bytes[written..])
            .map_err(WriteProgressError::Io)?;
        calls.push(u64::try_from(count).map_err(|_| WriteProgressError::Overrun)?);
        if count == 0 {
            return Err(WriteProgressError::ZeroProgress);
        }
        if count > remaining {
            return Err(WriteProgressError::Overrun);
        }
        written = written
            .checked_add(count)
            .ok_or(WriteProgressError::Overrun)?;
    }
    Ok(calls)
}

fn is_single_component(value: &str) -> bool {
    let path = Path::new(value);
    let mut components = path.components();
    matches!(components.next(), Some(Component::Normal(name)) if name.to_str() == Some(value))
        && components.next().is_none()
}

fn path_exists(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn sync_parent_directory(directory: &Path) -> io::Result<()> {
    File::open(directory)?.sync_all()
}

fn hash_file_exact(file: &mut File, expected_length: u64) -> Result<[u8; 32], OutputIoError> {
    file.seek(SeekFrom::Start(0))
        .map_err(|error| OutputIoError::from_io(OutputIoPhase::FinalizePreflight, &error))?;
    let mut digest = Sha256::new();
    let mut total = 0_u64;
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| OutputIoError::from_io(OutputIoPhase::FinalizePreflight, &error))?;
        if count == 0 {
            break;
        }
        total = total
            .checked_add(u64::try_from(count).map_err(|_| {
                OutputIoError::new(
                    OutputIoPhase::FinalizePreflight,
                    OutputIoFailure::FinalizeLength,
                )
            })?)
            .ok_or_else(|| {
                OutputIoError::new(
                    OutputIoPhase::FinalizePreflight,
                    OutputIoFailure::FinalizeLength,
                )
            })?;
        if total > expected_length {
            return Err(OutputIoError::new(
                OutputIoPhase::FinalizePreflight,
                OutputIoFailure::FinalizeLength,
            ));
        }
        digest.update(&buffer[..count]);
    }
    if total != expected_length {
        return Err(OutputIoError::new(
            OutputIoPhase::FinalizePreflight,
            OutputIoFailure::FinalizeLength,
        ));
    }
    Ok(digest.finalize().into())
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

fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
