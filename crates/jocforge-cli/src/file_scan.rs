// pattern: Imperative Shell

// pattern: Imperative Shell

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use jocforge_core::{MAX_PCM_CHANNELS, decode_pcm_frame};

const JOC_FRAME_SAMPLES: usize = 1536;
// Metadata is bounded independently of the PCM payload so a malformed chunk
// cannot force an unbounded allocation in the file-reading shell.
const MAX_METADATA_BODY_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScannedChunk {
    pub(crate) id: [u8; 4],
    pub(crate) offset: u64,
    pub(crate) declared_size: u64,
    pub(crate) body_offset: u64,
}

#[derive(Debug)]
pub(crate) struct ScannedBw64 {
    pub(crate) form: [u8; 4],
    pub(crate) chunks: Vec<ScannedChunk>,
    pub(crate) fmt: Vec<u8>,
    pub(crate) axml: Vec<u8>,
    pub(crate) chna: Vec<u8>,
    pub(crate) dbmd: Vec<u8>,
    pub(crate) data_offset: u64,
    pub(crate) data_size: u64,
}

pub(crate) fn scan_bw64_file(path: &Path) -> Result<ScannedBw64, String> {
    let mut file = File::open(path)
        .map_err(|error| format!("failed to open ADM input '{}': {error}", path.display()))?;
    let physical_len = file
        .metadata()
        .map_err(|error| format!("failed to stat ADM input '{}': {error}", path.display()))?
        .len();
    if physical_len < 12 {
        return Err("truncated BW64 header".to_owned());
    }
    let mut header = [0_u8; 12];
    file.read_exact(&mut header)
        .map_err(|error| format!("failed to read BW64 header: {error}"))?;
    let form = [header[0], header[1], header[2], header[3]];
    if !matches!(&form, b"BW64" | b"RF64" | b"RIFF") || &header[8..12] != b"WAVE" {
        return Err("invalid BW64 RIFF form".to_owned());
    }

    let mut cursor = 12_u64;
    let mut ds64_data_size = None;
    let mut chunks = Vec::new();
    let mut fmt = None;
    let mut axml = None;
    let mut chna = None;
    let mut dbmd = None;
    let mut data = None;
    while cursor < physical_len {
        let remaining = physical_len - cursor;
        if remaining < 8 {
            return Err(format!("truncated BW64 chunk header at offset {cursor}"));
        }
        file.seek(SeekFrom::Start(cursor))
            .map_err(|error| format!("failed to seek BW64 chunk at offset {cursor}: {error}"))?;
        let mut chunk_header = [0_u8; 8];
        file.read_exact(&mut chunk_header).map_err(|error| {
            format!("failed to read BW64 chunk header at offset {cursor}: {error}")
        })?;
        let id = [
            chunk_header[0],
            chunk_header[1],
            chunk_header[2],
            chunk_header[3],
        ];
        let declared = u64::from(u32::from_le_bytes([
            chunk_header[4],
            chunk_header[5],
            chunk_header[6],
            chunk_header[7],
        ]));
        let size = if id == *b"data" && declared == u64::from(u32::MAX) {
            ds64_data_size.ok_or_else(|| {
                "BW64 data chunk uses 0xffffffff but no preceding ds64 chunk exists".to_owned()
            })?
        } else {
            declared
        };
        let body_offset = cursor + 8;
        let body_end = body_offset
            .checked_add(size)
            .ok_or_else(|| format!("BW64 chunk at offset {cursor} overflows file bounds"))?;
        let padded_end = body_end
            .checked_add(size % 2)
            .ok_or_else(|| format!("BW64 chunk at offset {cursor} overflows padding bounds"))?;
        if padded_end > physical_len {
            return Err(format!(
                "truncated BW64 chunk '{}' at offset {cursor}: declared {size} bytes",
                String::from_utf8_lossy(&id)
            ));
        }
        chunks.push(ScannedChunk {
            id,
            offset: cursor,
            declared_size: size,
            body_offset,
        });
        match id {
            id if id == *b"ds64" => {
                let body = read_body(&mut file, body_offset, size, "ds64")?;
                if body.len() < 16 {
                    return Err("truncated BW64 ds64 chunk".to_owned());
                }
                ds64_data_size = Some(u64::from_le_bytes(body[8..16].try_into().unwrap()));
            }
            id if id == *b"fmt " => fmt = Some(read_body(&mut file, body_offset, size, "fmt ")?),
            id if id == *b"axml" => axml = Some(read_body(&mut file, body_offset, size, "axml")?),
            id if id == *b"chna" => chna = Some(read_body(&mut file, body_offset, size, "chna")?),
            id if id == *b"dbmd" => dbmd = Some(read_body(&mut file, body_offset, size, "dbmd")?),
            id if id == *b"data" => data = Some((body_offset, size)),
            _ => {}
        }
        cursor = padded_end;
    }

    let (data_offset, data_size) =
        data.ok_or_else(|| "missing required ADM chunk 'data'".to_owned())?;
    Ok(ScannedBw64 {
        form,
        chunks,
        fmt: fmt.ok_or_else(|| "missing required ADM chunk 'fmt '".to_owned())?,
        axml: axml.ok_or_else(|| "missing required ADM chunk 'axml'".to_owned())?,
        chna: chna.ok_or_else(|| "missing required ADM chunk 'chna'".to_owned())?,
        dbmd: dbmd.unwrap_or_default(),
        data_offset,
        data_size,
    })
}

pub(crate) fn read_pcm_frame(
    path: &Path,
    scanned: &ScannedBw64,
    frame_index: usize,
    track_indices: &[usize],
) -> Result<Vec<Vec<f64>>, String> {
    if scanned.fmt.len() < 16 {
        return Err("BW64 fmt chunk is truncated".to_owned());
    }
    let format = u16::from_le_bytes([scanned.fmt[0], scanned.fmt[1]]);
    let channel_count = usize::from(u16::from_le_bytes([scanned.fmt[2], scanned.fmt[3]]));
    let bits = u16::from_le_bytes([scanned.fmt[14], scanned.fmt[15]]);
    let bytes_per_sample = usize::from(bits / 8);
    let block_align = channel_count
        .checked_mul(bytes_per_sample)
        .ok_or_else(|| "BW64 PCM block alignment overflows".to_owned())?;
    if channel_count == 0 || channel_count > MAX_PCM_CHANNELS {
        return Err(format!(
            "unsupported BW64 PCM channel count {channel_count}: maximum is {MAX_PCM_CHANNELS}"
        ));
    }
    if format != 1 || !matches!(bits, 16 | 24 | 32) {
        return Err(format!(
            "unsupported BW64 PCM format {format} with {bits} bits"
        ));
    }
    let total_samples = usize::try_from(scanned.data_size)
        .ok()
        .and_then(|bytes| bytes.checked_div(block_align))
        .ok_or_else(|| "BW64 PCM sample count overflows".to_owned())?;
    let start_sample = frame_index
        .checked_mul(JOC_FRAME_SAMPLES)
        .ok_or_else(|| "BW64 PCM frame offset overflows".to_owned())?;
    if start_sample >= total_samples {
        return Err(format!(
            "BW64 PCM frame {frame_index} is outside the data chunk"
        ));
    }
    let available_samples = JOC_FRAME_SAMPLES.min(total_samples - start_sample);
    let byte_count = available_samples
        .checked_mul(block_align)
        .ok_or_else(|| "BW64 PCM frame byte count overflows".to_owned())?;
    let byte_offset = scanned
        .data_offset
        .checked_add(
            u64::try_from(start_sample)
                .ok()
                .and_then(|sample| sample.checked_mul(u64::try_from(block_align).ok()?))
                .ok_or_else(|| "BW64 PCM frame byte offset overflows".to_owned())?,
        )
        .ok_or_else(|| "BW64 PCM frame byte offset overflows".to_owned())?;
    let mut file = File::open(path)
        .map_err(|error| format!("failed to open PCM input '{}': {error}", path.display()))?;
    file.seek(SeekFrom::Start(byte_offset))
        .map_err(|error| format!("failed to seek PCM frame {frame_index}: {error}"))?;
    let mut bytes = vec![0_u8; byte_count];
    file.read_exact(&mut bytes)
        .map_err(|error| format!("failed to read PCM frame {frame_index}: {error}"))?;
    let mut decoded = decode_pcm_frame(&bytes, channel_count, bits, track_indices)
        .map_err(|error| format!("failed to decode PCM frame {frame_index}: {error}"))?;
    for channel in &mut decoded {
        channel.resize(JOC_FRAME_SAMPLES, 0.0);
    }
    Ok(decoded)
}

fn read_body(file: &mut File, offset: u64, size: u64, label: &str) -> Result<Vec<u8>, String> {
    let length =
        usize::try_from(size).map_err(|_| format!("BW64 {label} chunk is too large to inspect"))?;
    if length > MAX_METADATA_BODY_BYTES {
        return Err(format!(
            "BW64 {label} chunk exceeds the {MAX_METADATA_BODY_BYTES} byte metadata bound"
        ));
    }
    let mut body = vec![0_u8; length];
    file.seek(SeekFrom::Start(offset))
        .map_err(|error| format!("failed to seek BW64 {label} chunk: {error}"))?;
    file.read_exact(&mut body)
        .map_err(|error| format!("failed to read BW64 {label} chunk: {error}"))?;
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::{ScannedChunk, read_pcm_frame, scan_bw64_file};
    use std::{
        fs,
        io::{Seek, SeekFrom, Write},
        path::PathBuf,
    };

    fn temp_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "jocforge-scan-{label}-{}-{}.bw64",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ))
    }

    fn append_chunk(file: &mut fs::File, id: [u8; 4], body: &[u8]) {
        file.write_all(&id).expect("chunk id");
        file.write_all(&(body.len() as u32).to_le_bytes())
            .expect("chunk length");
        file.write_all(body).expect("chunk body");
        if body.len() % 2 != 0 {
            file.write_all(&[0]).expect("odd padding");
        }
    }

    fn fmt_16_mono() -> [u8; 16] {
        [1, 0, 1, 0, 0x80, 0xbb, 0, 0, 0x80, 0xbb, 0, 0, 1, 0, 8, 0]
    }

    fn write_metadata_tail(file: &mut fs::File) {
        append_chunk(file, *b"axml", b"<audioFormatExtended/>");
        append_chunk(file, *b"chna", &[1, 0, 1, 0]);
    }

    #[test]
    fn scanner_finds_metadata_after_data_and_skips_odd_padding() {
        let path = temp_path("tail");
        let mut file = fs::File::create(&path).expect("temporary BW64");
        file.write_all(b"RIFF\x00\x00\x00\x00WAVE")
            .expect("RIFF header");
        append_chunk(&mut file, *b"JUNK", b"odd");
        append_chunk(&mut file, *b"fmt ", &fmt_16_mono());
        append_chunk(&mut file, *b"data", &[1, 2, 3, 4]);
        write_metadata_tail(&mut file);
        file.flush().expect("flush fixture");

        let scanned = scan_bw64_file(&path).expect("scan metadata tail");
        assert_eq!(scanned.data_size, 4);
        assert_eq!(&scanned.axml, b"<audioFormatExtended/>");
        assert_eq!(scanned.chna, vec![1, 0, 1, 0]);
        assert_eq!(
            scanned
                .chunks
                .iter()
                .map(|chunk| chunk.id)
                .collect::<Vec<_>>(),
            vec![*b"JUNK", *b"fmt ", *b"data", *b"axml", *b"chna"]
        );
        assert_eq!(
            scanned.chunks[1],
            ScannedChunk {
                id: *b"fmt ",
                offset: 24,
                declared_size: 16,
                body_offset: 32,
            }
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn scanner_seeks_over_large_data_without_reading_it() {
        let path = temp_path("large");
        let large_data_size = 128_u64 * 1024 * 1024;
        let mut file = fs::File::create(&path).expect("temporary BW64");
        file.write_all(b"RIFF\x00\x00\x00\x00WAVE")
            .expect("RIFF header");
        append_chunk(&mut file, *b"fmt ", &fmt_16_mono());
        file.write_all(b"data")
            .and_then(|()| file.write_all(&(large_data_size as u32).to_le_bytes()))
            .expect("data header");
        let data_end = file.stream_position().expect("data start") + large_data_size;
        file.set_len(data_end).expect("sparse data extent");
        file.seek(SeekFrom::End(0)).expect("metadata tail position");
        write_metadata_tail(&mut file);
        file.flush().expect("flush fixture");

        let scanned = scan_bw64_file(&path).expect("seek over large data");
        assert_eq!(scanned.data_size, large_data_size);
        assert_eq!(scanned.data_offset, 44);
        assert_eq!(scanned.chunks.len(), 4);
        assert_eq!(scanned.chunks[1].id, *b"data");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn scanner_stops_at_clean_eof_without_phantom_chunk() {
        let path = temp_path("eof");
        let mut file = fs::File::create(&path).expect("temporary BW64");
        file.write_all(b"RIFF\x00\x00\x00\x00WAVE")
            .expect("RIFF header");
        append_chunk(&mut file, *b"fmt ", &fmt_16_mono());
        append_chunk(&mut file, *b"data", &[0, 0]);
        write_metadata_tail(&mut file);
        file.flush().expect("flush fixture");

        let scanned = scan_bw64_file(&path).expect("clean EOF");
        assert_eq!(scanned.chunks.last().expect("last chunk").id, *b"chna");
        assert_eq!(scanned.chunks.len(), 4);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn scanner_rejects_metadata_above_the_named_memory_bound() {
        let path = temp_path("metadata-bound");
        let mut file = fs::File::create(&path).expect("temporary BW64");
        file.write_all(b"RIFF\x00\x00\x00\x00WAVE")
            .expect("RIFF header");
        append_chunk(&mut file, *b"fmt ", &fmt_16_mono());
        append_chunk(&mut file, *b"data", &[0, 0]);
        let oversized = vec![0_u8; super::MAX_METADATA_BODY_BYTES + 1];
        append_chunk(&mut file, *b"axml", &oversized);
        file.flush().expect("flush fixture");

        let error = scan_bw64_file(&path).expect_err("oversized metadata must be rejected");

        assert!(error.contains("metadata bound"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn pcm_frame_reader_rejects_excessive_channel_count_before_allocating() {
        let path = temp_path("channel-bound");
        let mut file = fs::File::create(&path).expect("temporary BW64");
        file.write_all(b"RIFF\x00\x00\x00\x00WAVE")
            .expect("RIFF header");
        let mut fmt = fmt_16_mono();
        fmt[2..4].copy_from_slice(&257_u16.to_le_bytes());
        fmt[12..14].copy_from_slice(&514_u16.to_le_bytes());
        append_chunk(&mut file, *b"fmt ", &fmt);
        append_chunk(&mut file, *b"data", &vec![0_u8; 514]);
        write_metadata_tail(&mut file);
        file.flush().expect("flush fixture");

        let scanned = scan_bw64_file(&path).expect("scan channel-bound fixture");
        let error = read_pcm_frame(&path, &scanned, 0, &[1]).expect_err("channel bound");

        assert!(error.contains("PCM channel count"));
        let _ = fs::remove_file(path);
    }
}
