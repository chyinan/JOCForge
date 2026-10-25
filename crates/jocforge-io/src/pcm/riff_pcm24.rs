// pattern: Functional Core
//! Selected-profile RIFF/PCM24 byte reader and planar F32 hop conversion.

use jocforge_domain::timeline::AdmSampleIndex;
use sha2::{Digest, Sha256};

/// Selected PCM24 input channel count.
pub const SELECTED_CHANNELS: usize = 38;
/// Selected PCM24 samples per channel and frame.
pub const SELECTED_SAMPLE_RATE_HZ: u32 = 48_000;
/// Selected PCM24 sample width.
pub const SELECTED_BITS_PER_SAMPLE: u16 = 24;
/// Selected interleaved bytes per sample frame.
pub const SELECTED_BLOCK_ALIGN: usize = 114;
const SELECTED_BYTE_RATE: u32 = 5_472_000;
const RIFF_HEADER_BYTES: usize = 12;
const CHUNK_HEADER_BYTES: usize = 8;
const PCM24_SIGN_BIT: i32 = 0x0080_0000;
const PCM24_MODULUS: i32 = 0x0100_0000;
const PCM24_SCALE: f32 = 8_388_608.0;
const PCM24_INT32_SCALE: f32 = 1.0 / 2_147_483_648.0;

/// The selected model-reader failure projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiffPcm24Error {
    InvalidInput,
}

impl RiffPcm24Error {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidInput => "INVALID_INPUT",
        }
    }
}

/// Number of complete interleaved frames returned by one PCM24 read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pcm24FrameCount(u64);

impl Pcm24FrameCount {
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Planar F32 bit rows decoded from the complete frames in one successful read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedPcm24ReadChunk {
    actual_frames: Pcm24FrameCount,
    planar_f32_bits: Vec<Vec<u32>>,
}

impl DecodedPcm24ReadChunk {
    pub const fn actual_frames(&self) -> Pcm24FrameCount {
        self.actual_frames
    }

    pub fn planar_f32_bits(&self) -> &[Vec<u32>] {
        &self.planar_f32_bits
    }
}

/// One preserved selected metadata chunk, borrowed from the input bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiffMetadataPayload<'a> {
    chunk_id: String,
    payload: &'a [u8],
}

impl<'a> RiffMetadataPayload<'a> {
    pub fn chunk_id(&self) -> &str {
        &self.chunk_id
    }

    pub const fn payload(&self) -> &'a [u8] {
        self.payload
    }
}

/// Unique ADM metadata payloads borrowed from one parsed RIFF resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedAdmRiffPayloads<'a> {
    axml: &'a [u8],
    chna: &'a [u8],
    dbmd: &'a [u8],
}

impl<'a> SelectedAdmRiffPayloads<'a> {
    pub const fn axml(&self) -> &'a [u8] {
        self.axml
    }

    pub const fn chna(&self) -> &'a [u8] {
        self.chna
    }

    pub const fn dbmd(&self) -> &'a [u8] {
        self.dbmd
    }
}

/// Parsed selected RIFF/PCM24 metadata and a borrowed data payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedRiffPcm24<'a> {
    sample_rate_hz: u32,
    channels: u16,
    block_align: u16,
    total_samples: u64,
    source_sha256: [u8; 32],
    data_offset: u64,
    data_payload: &'a [u8],
    chunk_ids: Vec<String>,
    metadata_payloads: Vec<RiffMetadataPayload<'a>>,
    selected_metadata_payload: Option<&'a [u8]>,
    has_adm_chunks: bool,
}

impl<'a> ParsedRiffPcm24<'a> {
    pub const fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    pub const fn channels(&self) -> u16 {
        self.channels
    }

    pub const fn block_align(&self) -> u16 {
        self.block_align
    }

    pub const fn total_samples(&self) -> u64 {
        self.total_samples
    }

    /// Returns the SHA-256 identity of the complete immutable RIFF byte slice.
    pub const fn source_sha256(&self) -> [u8; 32] {
        self.source_sha256
    }

    pub const fn data_offset(&self) -> u64 {
        self.data_offset
    }

    pub fn chunk_ids(&self) -> &[String] {
        &self.chunk_ids
    }

    pub fn metadata_payloads(&self) -> &[RiffMetadataPayload<'_>] {
        &self.metadata_payloads
    }

    pub const fn selected_metadata_payload(&self) -> Option<&[u8]> {
        self.selected_metadata_payload
    }

    pub const fn has_adm_chunks(&self) -> bool {
        self.has_adm_chunks
    }

    /// Selects one AXML, CHNA and DBMD payload, each borrowed from this RIFF resource.
    pub fn selected_adm_payloads(&self) -> Result<SelectedAdmRiffPayloads<'a>, RiffPcm24Error> {
        Ok(SelectedAdmRiffPayloads {
            axml: unique_metadata_payload(&self.metadata_payloads, *b"axml")?,
            chna: unique_metadata_payload(&self.metadata_payloads, *b"chna")?,
            dbmd: unique_metadata_payload(&self.metadata_payloads, *b"dbmd")?,
        })
    }

    /// Converts one validated sample hop to 38 planar binary32 bit rows.
    pub fn read_pcm24_hop(
        &self,
        first_sample: AdmSampleIndex,
        sample_count: u64,
    ) -> Result<Vec<Vec<u32>>, RiffPcm24Error> {
        let end_sample = first_sample
            .value()
            .checked_add(sample_count)
            .ok_or(RiffPcm24Error::InvalidInput)?;
        if sample_count == 0 || end_sample > self.total_samples {
            return Err(RiffPcm24Error::InvalidInput);
        }
        let first_sample =
            usize::try_from(first_sample.value()).map_err(|_| RiffPcm24Error::InvalidInput)?;
        let end_sample = usize::try_from(end_sample).map_err(|_| RiffPcm24Error::InvalidInput)?;
        let sample_count =
            usize::try_from(sample_count).map_err(|_| RiffPcm24Error::InvalidInput)?;

        let mut planar = (0..SELECTED_CHANNELS)
            .map(|_| Vec::with_capacity(sample_count))
            .collect::<Vec<_>>();
        for sample in first_sample..end_sample {
            let frame_offset = sample
                .checked_mul(usize::from(self.block_align))
                .ok_or(RiffPcm24Error::InvalidInput)?;
            for (channel, row) in planar.iter_mut().enumerate() {
                let offset = frame_offset
                    .checked_add(channel * 3)
                    .ok_or(RiffPcm24Error::InvalidInput)?;
                let sample_end = offset.checked_add(3).ok_or(RiffPcm24Error::InvalidInput)?;
                let packed = self
                    .data_payload
                    .get(offset..sample_end)
                    .ok_or(RiffPcm24Error::InvalidInput)?;
                let unsigned = i32::from(packed[0])
                    | (i32::from(packed[1]) << 8)
                    | (i32::from(packed[2]) << 16);
                let signed = if unsigned & PCM24_SIGN_BIT != 0 {
                    unsigned - PCM24_MODULUS
                } else {
                    unsigned
                };
                let normalized = (signed as f32) / PCM24_SCALE;
                if !normalized.is_finite() {
                    return Err(RiffPcm24Error::InvalidInput);
                }
                row.push(normalized.to_bits());
            }
        }
        Ok(planar)
    }
}

/// Functional Core: decode the complete PCM24 frames from one successful byte read.
///
/// The caller reports the number of bytes it requested. A short read is accepted;
/// trailing bytes that do not complete an interleaved frame are not decoded. This
/// function performs no file access and does not update a source cursor or row table.
pub fn decode_selected_pcm24_read_chunk(
    read_bytes: &[u8],
    requested_bytes: u32,
) -> Result<DecodedPcm24ReadChunk, RiffPcm24Error> {
    let actual_bytes = u32::try_from(read_bytes.len()).map_err(|_| RiffPcm24Error::InvalidInput)?;
    if actual_bytes > requested_bytes {
        return Err(RiffPcm24Error::InvalidInput);
    }

    let actual_byte_count =
        usize::try_from(actual_bytes).map_err(|_| RiffPcm24Error::InvalidInput)?;
    let frame_count = actual_byte_count / SELECTED_BLOCK_ALIGN;
    let complete_bytes = frame_count
        .checked_mul(SELECTED_BLOCK_ALIGN)
        .ok_or(RiffPcm24Error::InvalidInput)?;
    let mut planar_f32_bits = (0..SELECTED_CHANNELS)
        .map(|_| Vec::with_capacity(frame_count))
        .collect::<Vec<_>>();

    for frame in 0..frame_count {
        let frame_offset = frame
            .checked_mul(SELECTED_BLOCK_ALIGN)
            .ok_or(RiffPcm24Error::InvalidInput)?;
        for (channel, row) in planar_f32_bits.iter_mut().enumerate() {
            let offset = frame_offset
                .checked_add(channel * 3)
                .ok_or(RiffPcm24Error::InvalidInput)?;
            let sample_end = offset.checked_add(3).ok_or(RiffPcm24Error::InvalidInput)?;
            if sample_end > complete_bytes {
                return Err(RiffPcm24Error::InvalidInput);
            }
            let packed = &read_bytes[offset..sample_end];
            let unsigned =
                i32::from(packed[0]) | (i32::from(packed[1]) << 8) | (i32::from(packed[2]) << 16);
            let signed = if unsigned & PCM24_SIGN_BIT != 0 {
                unsigned - PCM24_MODULUS
            } else {
                unsigned
            };
            let shifted = signed << 8;
            let normalized = (shifted as f32) * PCM24_INT32_SCALE;
            if !normalized.is_finite() {
                return Err(RiffPcm24Error::InvalidInput);
            }
            row.push(normalized.to_bits());
        }
    }

    Ok(DecodedPcm24ReadChunk {
        actual_frames: Pcm24FrameCount(
            u64::try_from(frame_count).map_err(|_| RiffPcm24Error::InvalidInput)?,
        ),
        planar_f32_bits,
    })
}

fn unique_metadata_payload<'a>(
    metadata: &[RiffMetadataPayload<'a>],
    wanted_id: [u8; 4],
) -> Result<&'a [u8], RiffPcm24Error> {
    let mut matches = metadata
        .iter()
        .filter(|chunk| chunk.chunk_id().as_bytes() == wanted_id);
    let payload = matches
        .next()
        .map(RiffMetadataPayload::payload)
        .ok_or(RiffPcm24Error::InvalidInput)?;
    if matches.next().is_some() {
        return Err(RiffPcm24Error::InvalidInput);
    }
    Ok(payload)
}

/// Parses the selected 48 kHz, 38-channel PCM24 RIFF model-reader contract.
///
/// This function accepts complete bytes and performs no filesystem I/O. It does not
/// validate ADM/XML link semantics, RF64/BW64, extensible format, or native malformed-
/// input behavior beyond the explicitly delivered model-reader cases.
pub fn parse_selected_riff_pcm24(
    bytes: &[u8],
    requested_metadata_id: Option<[u8; 4]>,
) -> Result<ParsedRiffPcm24<'_>, RiffPcm24Error> {
    if bytes.len() < RIFF_HEADER_BYTES || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(RiffPcm24Error::InvalidInput);
    }
    let riff_extent = usize::try_from(read_u32_le(&bytes[4..8])?)
        .map_err(|_| RiffPcm24Error::InvalidInput)?
        .checked_add(8)
        .ok_or(RiffPcm24Error::InvalidInput)?;
    if riff_extent != bytes.len() {
        return Err(RiffPcm24Error::InvalidInput);
    }

    let mut position = RIFF_HEADER_BYTES;
    let mut chunk_ids = Vec::new();
    let mut metadata_payloads = Vec::new();
    let mut format: Option<SelectedFormat> = None;
    let mut data_payload: Option<&[u8]> = None;
    let mut data_offset = 0;
    let mut axml_present = false;
    let mut chna_present = false;

    while position
        .checked_add(CHUNK_HEADER_BYTES)
        .is_some_and(|header_end| header_end <= riff_extent)
    {
        let header_end = position + CHUNK_HEADER_BYTES;
        let chunk_id: [u8; 4] = bytes[position..position + 4]
            .try_into()
            .map_err(|_| RiffPcm24Error::InvalidInput)?;
        let payload_bytes = usize::try_from(read_u32_le(&bytes[position + 4..header_end])?)
            .map_err(|_| RiffPcm24Error::InvalidInput)?;
        let payload_start = header_end;
        let payload_end = payload_start
            .checked_add(payload_bytes)
            .ok_or(RiffPcm24Error::InvalidInput)?;
        if payload_end > riff_extent {
            return Err(RiffPcm24Error::InvalidInput);
        }
        let payload = &bytes[payload_start..payload_end];
        chunk_ids.push(decode_chunk_id(chunk_id));

        match &chunk_id {
            b"fmt " => {
                if format.is_some() {
                    return Err(RiffPcm24Error::InvalidInput);
                }
                format = Some(parse_selected_format(payload)?);
            }
            b"data" => {
                if data_payload.is_some() || payload.is_empty() {
                    return Err(RiffPcm24Error::InvalidInput);
                }
                if payload.len() % SELECTED_BLOCK_ALIGN != 0 {
                    return Err(RiffPcm24Error::InvalidInput);
                }
                data_offset = payload_start;
                data_payload = Some(payload);
            }
            b"axml" | b"chna" | b"dbmd" | b"bext" | b"iXML" => {
                let chunk_name = decode_chunk_id(chunk_id);
                axml_present |= chunk_id == *b"axml";
                chna_present |= chunk_id == *b"chna";
                metadata_payloads.push(RiffMetadataPayload {
                    chunk_id: chunk_name,
                    payload,
                });
            }
            _ => {}
        }

        position = payload_end
            .checked_add(payload_bytes % 2)
            .ok_or(RiffPcm24Error::InvalidInput)?;
    }

    let format = format.ok_or(RiffPcm24Error::InvalidInput)?;
    let data_payload = data_payload.ok_or(RiffPcm24Error::InvalidInput)?;
    let selected_metadata_payload = if let Some(requested_id) = requested_metadata_id {
        let mut matches = metadata_payloads
            .iter()
            .filter(|chunk| chunk.chunk_id.as_bytes() == requested_id);
        let first = matches.next().map(|chunk| chunk.payload);
        if matches.next().is_some() {
            return Err(RiffPcm24Error::InvalidInput);
        }
        first
    } else {
        None
    };

    Ok(ParsedRiffPcm24 {
        sample_rate_hz: format.sample_rate_hz,
        channels: format.channels,
        block_align: format.block_align,
        total_samples: u64::try_from(data_payload.len() / SELECTED_BLOCK_ALIGN)
            .map_err(|_| RiffPcm24Error::InvalidInput)?,
        source_sha256: Sha256::digest(bytes).into(),
        data_offset: u64::try_from(data_offset).map_err(|_| RiffPcm24Error::InvalidInput)?,
        data_payload,
        chunk_ids,
        metadata_payloads,
        selected_metadata_payload,
        has_adm_chunks: axml_present && chna_present,
    })
}

struct SelectedFormat {
    channels: u16,
    sample_rate_hz: u32,
    block_align: u16,
}

fn parse_selected_format(payload: &[u8]) -> Result<SelectedFormat, RiffPcm24Error> {
    if payload.len() < 16 {
        return Err(RiffPcm24Error::InvalidInput);
    }
    let format_tag = read_u16_le(&payload[0..2])?;
    let channels = read_u16_le(&payload[2..4])?;
    let sample_rate_hz = read_u32_le(&payload[4..8])?;
    let byte_rate = read_u32_le(&payload[8..12])?;
    let block_align = read_u16_le(&payload[12..14])?;
    let bits_per_sample = read_u16_le(&payload[14..16])?;
    if format_tag != 1
        || usize::from(channels) != SELECTED_CHANNELS
        || sample_rate_hz != SELECTED_SAMPLE_RATE_HZ
        || byte_rate != SELECTED_BYTE_RATE
        || usize::from(block_align) != SELECTED_BLOCK_ALIGN
        || bits_per_sample != SELECTED_BITS_PER_SAMPLE
    {
        return Err(RiffPcm24Error::InvalidInput);
    }
    Ok(SelectedFormat {
        channels,
        sample_rate_hz,
        block_align,
    })
}

fn decode_chunk_id(chunk_id: [u8; 4]) -> String {
    String::from_utf8_lossy(&chunk_id).into_owned()
}

fn read_u16_le(bytes: &[u8]) -> Result<u16, RiffPcm24Error> {
    let word: [u8; 2] = bytes.try_into().map_err(|_| RiffPcm24Error::InvalidInput)?;
    Ok(u16::from_le_bytes(word))
}

fn read_u32_le(bytes: &[u8]) -> Result<u32, RiffPcm24Error> {
    let word: [u8; 4] = bytes.try_into().map_err(|_| RiffPcm24Error::InvalidInput)?;
    Ok(u32::from_le_bytes(word))
}
