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
    publication::{Flat7xAuAppendProposal, Flat7xAuAppendReceiptCandidate},
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

/// Writes one whole-AU proposal to the caller-owned staging file and verifies the same bytes.
///
/// The file must be exclusively owned by the caller and positioned by its current length. The
/// adapter does not create or finalize a file, roll back a partial append, or expose a final
/// destination. Any error after writing begins requires the caller to abandon the staging attempt.
pub fn append_flat7x_au_to_staging_file(
    file: &mut File,
    proposal: &Flat7xAuAppendProposal,
) -> Result<Flat7xAuAppendReceiptCandidate, Flat7xAuAppendIoError> {
    let ledger = proposal.ledger_entry();
    let expected_offset = ledger.offset();
    let actual_length = file.seek(SeekFrom::End(0))?;
    if actual_length != expected_offset {
        return Err(Flat7xAuAppendIoError::StagingLengthMismatch {
            expected: expected_offset,
            actual: actual_length,
        });
    }

    let bytes = proposal.bytes();
    let (i0_frame_bytes, d0_frame_bytes) = ledger.frame_lengths();
    let i0_end = usize::from(i0_frame_bytes);
    let expected_total = u32::from(i0_frame_bytes) + u32::from(d0_frame_bytes);
    let expected_total = usize::try_from(expected_total)
        .map_err(|_| Flat7xAuAppendIoError::ProposalExtentMismatch)?;
    if bytes.len() != expected_total || u32::try_from(expected_total).ok() != Some(ledger.length())
    {
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

    let mut remaining = bytes;
    let mut write_calls = Vec::new();
    while !remaining.is_empty() {
        let written = file.write(remaining)?;
        if written == 0 {
            return Err(Flat7xAuAppendIoError::WriteZero);
        }
        write_calls.push(u64::try_from(written).map_err(|_| {
            Flat7xAuAppendIoError::Io(io::Error::other("write count conversion failed"))
        })?);
        remaining = &remaining[written..];
    }
    file.flush()?;
    file.sync_all()?;

    file.seek(SeekFrom::Start(expected_offset))?;
    let mut readback = vec![0_u8; bytes.len()];
    file.read_exact(&mut readback)?;
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

    Ok(Flat7xAuAppendReceiptCandidate::new(
        proposal.attempt_id().to_owned(),
        ledger.au_index(),
        expected_offset,
        bytes_written,
        write_calls,
        Sha256::digest(&readback).into(),
        i0_crc_residual,
        d0_crc_residual,
        true,
        true,
    ))
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File, OpenOptions},
        io::{Read, Seek, Write},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{Flat7xAuAppendIoError, append_flat7x_au_to_staging_file};
    use jocforge_domain::{
        codec::frame_size::Eac3FrameSize,
        profile::{
            D0EmdfSkipFieldExtent, D0FrameExtent, Flat7xAuCandidate, Flat7xSourceGeneration,
            I0FrameExtent, compose_flat7x_au_byte_sequence, validate_flat7x_au_candidate,
        },
        publication::{Flat7xAuOutputOwner, Flat7xAuOutputOwnerError},
    };

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

    fn proposal() -> jocforge_domain::publication::Flat7xAuAppendProposal {
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
        Flat7xAuOutputOwner::cold("io-test-attempt", 1)
            .unwrap()
            .stage_next_access_unit(sequence)
            .unwrap()
    }

    #[test]
    fn adapter_writes_syncs_and_reads_back_one_aggregate_au() {
        let (path, mut file) = temp_staging_file();
        let proposal = proposal();
        let expected = proposal.bytes().to_vec();

        let receipt = append_flat7x_au_to_staging_file(&mut file, &proposal).unwrap();
        let after = proposal.complete(receipt).unwrap();

        assert_eq!(fs::metadata(&path).unwrap().len(), 6_656);
        assert_eq!(after.after_owner().durable_length(), 6_656);
        assert_eq!(after.ledger_entry().frame_lengths(), (2_560, 4_096));
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
        let proposal = proposal();

        assert!(matches!(
            append_flat7x_au_to_staging_file(&mut file, &proposal),
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
    fn receipt_candidate_is_bound_to_proposal_and_requires_both_crc_checks() {
        let (path, mut file) = temp_staging_file();
        let proposal = proposal();
        let receipt = append_flat7x_au_to_staging_file(&mut file, &proposal).unwrap();
        let (i0_crc, d0_crc) = receipt.frame_crc_residuals();
        let changed = jocforge_domain::publication::Flat7xAuAppendReceiptCandidate::new(
            receipt.attempt_id().to_owned(),
            receipt.au_index(),
            receipt.offset(),
            receipt.bytes_written(),
            receipt.write_calls().to_vec(),
            *receipt.readback_sha256(),
            i0_crc.wrapping_add(1),
            d0_crc,
            receipt.file_fsync(),
            receipt.durable(),
        );

        assert_eq!(
            proposal.complete(changed),
            Err(Flat7xAuOutputOwnerError::AppendReceiptCandidateMismatch)
        );
        drop(file);
        fs::remove_file(path).unwrap();
    }
}
