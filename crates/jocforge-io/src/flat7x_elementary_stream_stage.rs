// pattern: Imperative Shell
//! Exact-byte writer for caller-owned Flat7x elementary-stream staging files.

use std::{
    error::Error,
    fmt,
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
};

use jocforge_domain::{
    codec::elementary_stream_composer::Flat7xElementaryStreamExtent,
    publication::{
        Flat7xElementaryStreamStageAfterProposal, Flat7xElementaryStreamStageProposal,
        Flat7xElementaryStreamStageReceiptCandidate,
    },
};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xElementaryStreamStageIoPhase {
    Preflight,
    Write,
    Flush,
    FileSync,
    Readback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xElementaryStreamStageIoFailure {
    Io,
    StagingLengthMismatch,
    ProposalExtentMismatch,
    WriteZero,
    WriteCallCountOverflow,
    ReadbackMismatch,
    StageReceiptRejected,
}

#[derive(Debug)]
pub struct Flat7xElementaryStreamStageIoError {
    phase: Flat7xElementaryStreamStageIoPhase,
    failure: Flat7xElementaryStreamStageIoFailure,
    io_kind: Option<io::ErrorKind>,
}

impl Flat7xElementaryStreamStageIoError {
    fn from_io(phase: Flat7xElementaryStreamStageIoPhase, error: &io::Error) -> Self {
        Self {
            phase,
            failure: Flat7xElementaryStreamStageIoFailure::Io,
            io_kind: Some(error.kind()),
        }
    }

    fn new(
        phase: Flat7xElementaryStreamStageIoPhase,
        failure: Flat7xElementaryStreamStageIoFailure,
    ) -> Self {
        Self {
            phase,
            failure,
            io_kind: None,
        }
    }

    pub const fn phase(&self) -> Flat7xElementaryStreamStageIoPhase {
        self.phase
    }

    pub const fn failure(&self) -> Flat7xElementaryStreamStageIoFailure {
        self.failure
    }

    pub const fn io_kind(&self) -> Option<io::ErrorKind> {
        self.io_kind
    }
}

impl fmt::Display for Flat7xElementaryStreamStageIoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Flat7x elementary-stream staging failed at {:?}: {:?}",
            self.phase, self.failure
        )?;
        if let Some(kind) = self.io_kind {
            write!(formatter, " ({kind:?})")?;
        }
        Ok(())
    }
}

impl Error for Flat7xElementaryStreamStageIoError {}

#[derive(Debug)]
struct StagingWriteEvidence {
    bytes_written: u32,
    write_call_count: u32,
    readback_sha256: [u8; 32],
}

/// Writes, flushes, syncs, and reads back one complete proposal on a caller-created unique temp file.
///
/// The caller must create the temporary destination with create-new semantics and abandon it after
/// every error. This adapter accepts no final path and performs no rename or exposure.
pub fn stage_flat7x_elementary_stream_to_file<'finalize>(
    file: &mut File,
    proposal: Flat7xElementaryStreamStageProposal<'_, 'finalize>,
) -> Result<Flat7xElementaryStreamStageAfterProposal<'finalize>, Flat7xElementaryStreamStageIoError>
{
    let extent = proposal.extent();
    let evidence = write_and_readback_staging(file, proposal.stream_bytes(), extent, |file| {
        file.sync_all()
    })?;

    let receipt = Flat7xElementaryStreamStageReceiptCandidate::new(
        &proposal,
        evidence.bytes_written,
        evidence.write_call_count,
        evidence.readback_sha256,
        true,
        true,
    );
    proposal.complete(receipt).map_err(|_| {
        Flat7xElementaryStreamStageIoError::new(
            Flat7xElementaryStreamStageIoPhase::Readback,
            Flat7xElementaryStreamStageIoFailure::StageReceiptRejected,
        )
    })
}

fn write_and_readback_staging<S, Sync>(
    file: &mut S,
    bytes: &[u8],
    extent: Flat7xElementaryStreamExtent,
    sync_all: Sync,
) -> Result<StagingWriteEvidence, Flat7xElementaryStreamStageIoError>
where
    S: Read + Write + Seek,
    Sync: FnOnce(&mut S) -> io::Result<()>,
{
    let expected_bytes = extent.byte_length();
    let actual_start = file.seek(SeekFrom::End(0)).map_err(|error| {
        Flat7xElementaryStreamStageIoError::from_io(
            Flat7xElementaryStreamStageIoPhase::Preflight,
            &error,
        )
    })?;
    if actual_start != 0 {
        return Err(Flat7xElementaryStreamStageIoError::new(
            Flat7xElementaryStreamStageIoPhase::Preflight,
            Flat7xElementaryStreamStageIoFailure::StagingLengthMismatch,
        ));
    }
    let expected_bits = expected_bytes.checked_mul(8).ok_or_else(|| {
        Flat7xElementaryStreamStageIoError::new(
            Flat7xElementaryStreamStageIoPhase::Preflight,
            Flat7xElementaryStreamStageIoFailure::ProposalExtentMismatch,
        )
    })?;
    if u32::try_from(bytes.len()).ok() != Some(expected_bytes)
        || extent.bit_length() != expected_bits
    {
        return Err(Flat7xElementaryStreamStageIoError::new(
            Flat7xElementaryStreamStageIoPhase::Preflight,
            Flat7xElementaryStreamStageIoFailure::ProposalExtentMismatch,
        ));
    }
    let expected_end = actual_start
        .checked_add(u64::from(expected_bytes))
        .ok_or_else(|| {
            Flat7xElementaryStreamStageIoError::new(
                Flat7xElementaryStreamStageIoPhase::Preflight,
                Flat7xElementaryStreamStageIoFailure::ProposalExtentMismatch,
            )
        })?;

    let mut remaining = bytes;
    let mut write_call_count = 0_u32;
    while !remaining.is_empty() {
        let written = file.write(remaining).map_err(|error| {
            Flat7xElementaryStreamStageIoError::from_io(
                Flat7xElementaryStreamStageIoPhase::Write,
                &error,
            )
        })?;
        if written == 0 {
            return Err(Flat7xElementaryStreamStageIoError::new(
                Flat7xElementaryStreamStageIoPhase::Write,
                Flat7xElementaryStreamStageIoFailure::WriteZero,
            ));
        }
        write_call_count = write_call_count.checked_add(1).ok_or_else(|| {
            Flat7xElementaryStreamStageIoError::new(
                Flat7xElementaryStreamStageIoPhase::Write,
                Flat7xElementaryStreamStageIoFailure::WriteCallCountOverflow,
            )
        })?;
        remaining = &remaining[written..];
    }

    file.flush().map_err(|error| {
        Flat7xElementaryStreamStageIoError::from_io(
            Flat7xElementaryStreamStageIoPhase::Flush,
            &error,
        )
    })?;
    sync_all(file).map_err(|error| {
        Flat7xElementaryStreamStageIoError::from_io(
            Flat7xElementaryStreamStageIoPhase::FileSync,
            &error,
        )
    })?;

    let actual_end = file.seek(SeekFrom::End(0)).map_err(|error| {
        Flat7xElementaryStreamStageIoError::from_io(
            Flat7xElementaryStreamStageIoPhase::Readback,
            &error,
        )
    })?;
    if actual_end != expected_end {
        return Err(Flat7xElementaryStreamStageIoError::new(
            Flat7xElementaryStreamStageIoPhase::Readback,
            Flat7xElementaryStreamStageIoFailure::StagingLengthMismatch,
        ));
    }
    file.seek(SeekFrom::Start(0)).map_err(|error| {
        Flat7xElementaryStreamStageIoError::from_io(
            Flat7xElementaryStreamStageIoPhase::Readback,
            &error,
        )
    })?;

    let mut readback_hash = Sha256::new();
    let mut offset = 0;
    let mut read_buffer = [0_u8; 8_192];
    while offset < bytes.len() {
        let chunk_length = (bytes.len() - offset).min(read_buffer.len());
        file.read_exact(&mut read_buffer[..chunk_length])
            .map_err(|error| {
                Flat7xElementaryStreamStageIoError::from_io(
                    Flat7xElementaryStreamStageIoPhase::Readback,
                    &error,
                )
            })?;
        if read_buffer[..chunk_length] != bytes[offset..offset + chunk_length] {
            return Err(Flat7xElementaryStreamStageIoError::new(
                Flat7xElementaryStreamStageIoPhase::Readback,
                Flat7xElementaryStreamStageIoFailure::ReadbackMismatch,
            ));
        }
        readback_hash.update(&read_buffer[..chunk_length]);
        offset += chunk_length;
    }

    Ok(StagingWriteEvidence {
        bytes_written: expected_bytes,
        write_call_count,
        readback_sha256: readback_hash.finalize().into(),
    })
}

#[cfg(test)]
mod tests;
