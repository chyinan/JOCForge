// pattern: Imperative Shell
//! Staging-file adapter for one complete Flat7x I0+D0 append proposal.

use std::{
    error::Error,
    fmt,
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
};

use jocforge_domain::{
    integrity::crc16::eac3_crc16,
    publication::{
        Flat7xAuAppendProposal, Flat7xAuAppendReceiptCandidate, Flat7xAuAttemptRetirement,
        Flat7xAuOutputAfterProposal,
    },
};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub enum Flat7xAuAppendIoError {
    Io(io::Error),
    StagingLengthMismatch { expected: u64, actual: u64 },
    ProposalExtentMismatch,
    I0FrameCrc,
    D0FrameCrc,
    WriteZero,
    ReadbackMismatch,
    ReceiptCandidateRejected,
}

impl fmt::Display for Flat7xAuAppendIoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "Flat7x staging IO failed: {error}"),
            Self::StagingLengthMismatch { expected, actual } => write!(
                formatter,
                "Flat7x staging length mismatch: expected {expected}, found {actual}"
            ),
            Self::ProposalExtentMismatch => {
                formatter.write_str("Flat7x append proposal extent differs from its ledger")
            }
            Self::I0FrameCrc => formatter.write_str("Flat7x I0 frame CRC mismatch"),
            Self::D0FrameCrc => formatter.write_str("Flat7x D0 frame CRC mismatch"),
            Self::WriteZero => formatter.write_str("Flat7x staging write returned zero bytes"),
            Self::ReadbackMismatch => {
                formatter.write_str("Flat7x staging bytes differ from the append proposal")
            }
            Self::ReceiptCandidateRejected => {
                formatter.write_str("Flat7x output owner rejected the IO receipt candidate")
            }
        }
    }
}

impl Error for Flat7xAuAppendIoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for Flat7xAuAppendIoError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug)]
pub struct Flat7xAuAppendIoFailure {
    error: Flat7xAuAppendIoError,
    retirement: Flat7xAuAttemptRetirement,
}

impl Flat7xAuAppendIoFailure {
    pub const fn error(&self) -> &Flat7xAuAppendIoError {
        &self.error
    }

    pub const fn retirement(&self) -> &Flat7xAuAttemptRetirement {
        &self.retirement
    }
}

impl fmt::Display for Flat7xAuAppendIoFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}; Flat7x attempt retired", self.error)
    }
}

impl Error for Flat7xAuAppendIoFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.error.source()
    }
}

struct StagingWriteEvidence {
    write_calls: Vec<u64>,
    bytes_written: u32,
    readback_sha256: [u8; 32],
    i0_crc_residual: u16,
    d0_crc_residual: u16,
    file_fsync: bool,
}

/// Appends one proposal to a caller-owned staging file and consumes it on every outcome.
///
/// The adapter does not create or finalize a file, roll back a partial append, or expose a final
/// destination. Any IO error returns a retirement receipt; the caller must abandon the staging
/// attempt. Success returns an after-owner proposal carrying the readback receipt candidate.
pub fn append_flat7x_au_to_staging_file<'a>(
    file: &mut File,
    proposal: Flat7xAuAppendProposal<'a>,
) -> Result<Flat7xAuOutputAfterProposal<'a>, Flat7xAuAppendIoFailure> {
    let ledger = proposal.ledger_entry();
    let expected_offset = ledger.offset();
    let (i0_frame_bytes, d0_frame_bytes) = ledger.frame_lengths();
    let evidence = match write_and_readback_staging(
        file,
        expected_offset,
        proposal.bytes(),
        usize::from(i0_frame_bytes),
        usize::from(d0_frame_bytes),
    ) {
        Ok(evidence) => evidence,
        Err(error) => {
            return Err(Flat7xAuAppendIoFailure {
                error,
                retirement: proposal.retire_staging_failure(),
            });
        }
    };

    complete_staged_proposal(proposal, evidence)
}

fn write_and_readback_staging(
    file: &mut File,
    expected_offset: u64,
    bytes: &[u8],
    i0_frame_bytes: usize,
    d0_frame_bytes: usize,
) -> Result<StagingWriteEvidence, Flat7xAuAppendIoError> {
    write_and_readback_sink(
        file,
        expected_offset,
        bytes,
        i0_frame_bytes,
        d0_frame_bytes,
        |file| {
            file.sync_all()?;
            Ok(true)
        },
    )
}

fn write_and_readback_sink<S>(
    sink: &mut S,
    expected_offset: u64,
    bytes: &[u8],
    i0_frame_bytes: usize,
    d0_frame_bytes: usize,
    sync: impl FnOnce(&mut S) -> io::Result<bool>,
) -> Result<StagingWriteEvidence, Flat7xAuAppendIoError>
where
    S: Read + Seek + Write,
{
    let actual_length = sink.seek(SeekFrom::End(0))?;
    if actual_length != expected_offset {
        return Err(Flat7xAuAppendIoError::StagingLengthMismatch {
            expected: expected_offset,
            actual: actual_length,
        });
    }

    let i0_end = i0_frame_bytes;
    let expected_total = i0_frame_bytes
        .checked_add(d0_frame_bytes)
        .ok_or(Flat7xAuAppendIoError::ProposalExtentMismatch)?;
    if bytes.len() != expected_total {
        return Err(Flat7xAuAppendIoError::ProposalExtentMismatch);
    }
    let i0_bytes = &bytes[..i0_end];
    let d0_bytes = &bytes[i0_end..];
    if eac3_crc16(&i0_bytes[2..]) != 0 {
        return Err(Flat7xAuAppendIoError::I0FrameCrc);
    }
    if eac3_crc16(&d0_bytes[2..]) != 0 {
        return Err(Flat7xAuAppendIoError::D0FrameCrc);
    }

    let mut write_calls = Vec::new();
    write_span(sink, i0_bytes, &mut write_calls)?;
    write_span(sink, d0_bytes, &mut write_calls)?;
    sink.flush()?;
    let file_fsync = sync(sink)?;

    sink.seek(SeekFrom::Start(expected_offset))?;
    let mut readback = vec![0_u8; bytes.len()];
    sink.read_exact(&mut readback)?;
    if readback.as_slice() != bytes {
        return Err(Flat7xAuAppendIoError::ReadbackMismatch);
    }

    let first_frame_readback = &readback[..i0_end];
    let second_frame_readback = &readback[i0_end..];
    let i0_crc_residual = eac3_crc16(&first_frame_readback[2..]);
    if i0_crc_residual != 0 {
        return Err(Flat7xAuAppendIoError::I0FrameCrc);
    }
    let d0_crc_residual = eac3_crc16(&second_frame_readback[2..]);
    if d0_crc_residual != 0 {
        return Err(Flat7xAuAppendIoError::D0FrameCrc);
    }
    let bytes_written =
        u32::try_from(readback.len()).map_err(|_| Flat7xAuAppendIoError::ProposalExtentMismatch)?;
    Ok(StagingWriteEvidence {
        write_calls,
        bytes_written,
        readback_sha256: Sha256::digest(&readback).into(),
        i0_crc_residual,
        d0_crc_residual,
        file_fsync,
    })
}

fn write_span<S: Write>(
    sink: &mut S,
    span: &[u8],
    write_calls: &mut Vec<u64>,
) -> Result<(), Flat7xAuAppendIoError> {
    let mut remaining = span;
    while !remaining.is_empty() {
        let written = sink.write(remaining)?;
        if written == 0 {
            return Err(Flat7xAuAppendIoError::WriteZero);
        }
        write_calls.push(u64::try_from(written).map_err(|_| {
            Flat7xAuAppendIoError::Io(io::Error::other("write count conversion failed"))
        })?);
        remaining = &remaining[written..];
    }
    Ok(())
}

fn complete_staged_proposal(
    proposal: Flat7xAuAppendProposal<'_>,
    evidence: StagingWriteEvidence,
) -> Result<Flat7xAuOutputAfterProposal<'_>, Flat7xAuAppendIoFailure> {
    let ledger = proposal.ledger_entry();
    let receipt = Flat7xAuAppendReceiptCandidate::new(
        proposal.attempt_id().to_owned(),
        ledger.au_index(),
        ledger.offset(),
        evidence.bytes_written,
        evidence.write_calls,
        evidence.readback_sha256,
        evidence.i0_crc_residual,
        evidence.d0_crc_residual,
        evidence.file_fsync,
        evidence.file_fsync,
    );
    proposal
        .complete(receipt)
        .map_err(|retirement| Flat7xAuAppendIoFailure {
            error: Flat7xAuAppendIoError::ReceiptCandidateRejected,
            retirement,
        })
}

#[cfg(any(test, feature = "test-support"))]
pub fn append_flat7x_au_to_memory_for_test<'a>(
    sink: &mut std::io::Cursor<Vec<u8>>,
    proposal: Flat7xAuAppendProposal<'a>,
) -> Result<Flat7xAuOutputAfterProposal<'a>, Flat7xAuAppendIoFailure> {
    let ledger = proposal.ledger_entry();
    let (i0_frame_bytes, d0_frame_bytes) = ledger.frame_lengths();
    let evidence = match write_and_readback_sink(
        sink,
        ledger.offset(),
        proposal.bytes(),
        usize::from(i0_frame_bytes),
        usize::from(d0_frame_bytes),
        |_| Ok(false),
    ) {
        Ok(evidence) => evidence,
        Err(error) => {
            return Err(Flat7xAuAppendIoFailure {
                error,
                retirement: proposal.retire_staging_failure(),
            });
        }
    };
    complete_staged_proposal(proposal, evidence)
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File, OpenOptions},
        io::{Cursor, Read, Seek, Write},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        Flat7xAuAppendIoError, append_flat7x_au_to_memory_for_test, write_and_readback_staging,
    };
    use jocforge_domain::{
        codec::frame_size::Eac3FrameSize,
        joc::emdf::metadata_owner::test_support::Flat7xReceiptGateMetadataTestInputs,
        profile::{
            D0EmdfSkipFieldExtent, D0FrameExtent, Flat7xAuCandidate, Flat7xSourceGeneration,
            I0FrameExtent, compose_flat7x_au_byte_sequence, validate_flat7x_au_candidate,
        },
        publication::{Flat7xAuAttemptRetirementReason, test_support::Flat7xAuAppendTestFixture},
    };

    const F15_PAIR_VECTORS: &[u8] = include_bytes!(
        "../../../outputs/IMPLEMENTER_SUPPLEMENTS/FLAT7X-F15-RECEIPT-PAIR-INPUT-ONLY-V1/INPUT-ONLY-VECTORS.json"
    );
    const METADATA_OWNER_VECTORS: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/metadata_owner/METADATA-OWNER-VECTORS.json"
    );
    const METADATA_PUBLIC_PROVIDER: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/metadata_owner/metadata-public-provider.json"
    );
    const METADATA_WRAPPER_CONFIG: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/metadata_owner/metadata-wrapper-config.json"
    );
    const KEYLESS_INTEGRITY_PROVIDER: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/integrity_owner/integrity-public-provider.json"
    );

    fn source_receipted_pair_fixture() -> Flat7xAuAppendTestFixture {
        Flat7xAuAppendTestFixture::positive_pair(Flat7xReceiptGateMetadataTestInputs {
            f15_pair_vectors: F15_PAIR_VECTORS,
            metadata_owner_vectors: METADATA_OWNER_VECTORS,
            metadata_public_provider: METADATA_PUBLIC_PROVIDER,
            metadata_wrapper_config: METADATA_WRAPPER_CONFIG,
            keyless_integrity_provider: KEYLESS_INTEGRITY_PROVIDER,
        })
    }

    fn temp_staging_file() -> (PathBuf, File) {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("monotonic wall clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jocforge-flat7x-au-{}-{unique}.partial",
            std::process::id()
        ));
        let file = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(&path)
            .expect("create unique temporary staging file");
        (path, file)
    }

    fn crc_valid_frame(frame_bytes: usize, marker: u8) -> Vec<u8> {
        let size = Eac3FrameSize::from_frame_bytes(frame_bytes).unwrap();
        let mut bytes = vec![0_u8; frame_bytes];
        bytes[0..2].copy_from_slice(&[0x0b, 0x77]);
        bytes[100] = marker;
        for offset in 0..11 {
            let value = (size.code().value() >> (10 - offset)) & 1;
            if value != 0 {
                let bit = 21 + offset;
                bytes[bit / 8] |= 1 << (7 - (bit % 8));
            }
        }
        let crc_offset = bytes.len() - 2;
        let crc = jocforge_domain::integrity::crc16::eac3_crc16(&bytes[2..crc_offset]);
        bytes[crc_offset..].copy_from_slice(&crc.to_be_bytes());
        assert_eq!(
            jocforge_domain::integrity::crc16::eac3_crc16(&bytes[2..]),
            0
        );
        bytes
    }

    fn candidate_bytes() -> Vec<u8> {
        let generation = Flat7xSourceGeneration::new(7);
        let metadata = Flat7xAuCandidate::new(
            Some(I0FrameExtent::new(generation, 2_560)),
            Some(D0FrameExtent::new(generation, 4_096)),
            Some(D0EmdfSkipFieldExtent::new(
                generation,
                2,
                1,
                Some(16),
                false,
            )),
        );
        let validation = validate_flat7x_au_candidate(
            jocforge_domain::profile::Flat7xI0D0TargetPlan::approved_contract(),
            metadata,
        )
        .unwrap();
        let sequence = compose_flat7x_au_byte_sequence(
            &validation,
            &crc_valid_frame(2_560, 0x31),
            &crc_valid_frame(4_096, 0x72),
        )
        .unwrap();
        sequence.bytes().to_vec()
    }

    #[test]
    fn staging_io_helper_writes_syncs_and_reads_back_one_aggregate_au() {
        let (path, mut file) = temp_staging_file();
        let expected = candidate_bytes();

        let evidence = write_and_readback_staging(&mut file, 0, &expected, 2_560, 4_096).unwrap();

        assert_eq!(fs::metadata(&path).unwrap().len(), 6_656);
        assert_eq!(evidence.bytes_written, 6_656);
        assert_eq!((evidence.i0_crc_residual, evidence.d0_crc_residual), (0, 0));
        assert!(evidence.write_calls.iter().sum::<u64>() == 6_656);
        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        let mut actual = Vec::new();
        file.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, expected);
        drop(file);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn adapter_rejects_wrong_staging_offset_before_writing() {
        let (path, mut file) = temp_staging_file();
        file.write_all(b"existing").unwrap();
        file.sync_all().unwrap();
        let bytes = candidate_bytes();

        assert!(matches!(
            write_and_readback_staging(&mut file, 0, &bytes, 2_560, 4_096),
            Err(Flat7xAuAppendIoError::StagingLengthMismatch {
                expected: 0,
                actual: 8
            })
        ));
        assert_eq!(fs::metadata(&path).unwrap().len(), 8);
        drop(file);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn staging_io_helper_rejects_corrupt_frame_crc_before_writing() {
        let (path, mut file) = temp_staging_file();
        let mut bytes = candidate_bytes();
        bytes[100] ^= 1;

        assert!(matches!(
            write_and_readback_staging(&mut file, 0, &bytes, 2_560, 4_096),
            Err(Flat7xAuAppendIoError::I0FrameCrc)
        ));
        assert_eq!(fs::metadata(&path).unwrap().len(), 0);
        drop(file);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn in_memory_sink_rejects_non_durable_fully_receipted_pair_commit() {
        let fixture = source_receipted_pair_fixture();
        let expected_bytes = fixture.expected_bytes();
        let proposal = fixture.append_proposal(fixture.attempt_id()).unwrap();
        let mut sink = Cursor::new(Vec::new());

        let failure = match append_flat7x_au_to_memory_for_test(&mut sink, proposal) {
            Ok(_) => panic!("an in-memory write must not produce a durable after-owner"),
            Err(failure) => failure,
        };

        assert!(matches!(
            failure.error(),
            Flat7xAuAppendIoError::ReceiptCandidateRejected
        ));
        assert_eq!(
            failure.retirement().reason(),
            Flat7xAuAttemptRetirementReason::AppendReceiptCandidateMismatch
        );
        assert!(failure.retirement().committed_owner_unchanged());
        assert!(failure.retirement().fresh_programme_cold_required());
        assert_eq!(sink.into_inner(), expected_bytes);
    }
}
