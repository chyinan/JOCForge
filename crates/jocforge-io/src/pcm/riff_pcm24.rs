// pattern: Functional Core
//! ADM RIFF/PCM24 byte reader and planar F32 hop conversion.

use jocforge_domain::timeline::AdmSampleIndex;
use sha2::Digest;

/// Selected PCM24 input channel count.
pub const SELECTED_CHANNELS: usize = 38;
/// Selected PCM24 samples per channel and frame.
pub const SELECTED_SAMPLE_RATE_HZ: u32 = 48_000;
/// Selected PCM24 sample width.
pub const SELECTED_BITS_PER_SAMPLE: u16 = 24;
/// Selected interleaved bytes per sample frame.
pub const SELECTED_BLOCK_ALIGN: usize = 114;
const PCM_SUBTYPE: [u8; 16] = [
    1, 0, 0, 0, 0, 0, 0x10, 0, 0x80, 0, 0, 0xaa, 0, 0x38, 0x9b, 0x71,
];
const RIFF_HEADER_BYTES: usize = 12;
const CHUNK_HEADER_BYTES: usize = 8;
const PCM24_SIGN_BIT: i32 = 0x0080_0000;
const PCM24_MODULUS: i32 = 0x0100_0000;
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

    pub fn into_planar_f32_bits(self) -> Vec<Vec<u32>> {
        self.planar_f32_bits
    }
}

/// One preserved selected metadata chunk, owned by the parsed resource.
///
/// Streaming parses cannot borrow metadata from a resident buffer, so the payload is copied. A
/// resident parse pays the same copy; the bytes handed to callers are identical either way.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiffMetadataPayload {
    chunk_id: String,
    payload: Vec<u8>,
    /// File offset of the first payload byte, so a consumer can re-read the same bytes from the
    /// resource without relying on the payload sharing one allocation with it.
    payload_offset: u64,
}

impl RiffMetadataPayload {
    pub fn chunk_id(&self) -> &str {
        &self.chunk_id
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn payload_offset(&self) -> u64 {
        self.payload_offset
    }
}

/// Unique ADM metadata payloads borrowed from one parsed RIFF resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedAdmRiffPayloads<'a> {
    axml: &'a [u8],
    axml_offset: u64,
    chna: &'a [u8],
    chna_offset: u64,
    dbmd: &'a [u8],
    dbmd_offset: u64,
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

    pub const fn axml_offset(&self) -> u64 {
        self.axml_offset
    }

    pub const fn chna_offset(&self) -> u64 {
        self.chna_offset
    }

    pub const fn dbmd_offset(&self) -> u64 {
        self.dbmd_offset
    }
}

/// Supplies exact byte ranges of one RIFF resource that is not fully resident.
///
/// `offset` is measured from the first byte of the **file**, so one reader serves the chunk walk,
/// the source hash and the PCM block reads. An implementation must return exactly `len` bytes;
/// returning a different length is an input error, because the resident path can never short-read
/// and a silent zero-fill would change the decoded blocks.
pub trait RiffDataReader: Send {
    fn read_at(&mut self, offset: u64, len: usize) -> Result<Vec<u8>, RiffPcm24Error>;

    /// Total resource length in bytes.
    fn len(&self) -> u64;

    /// SHA-256 of the whole resource, in file order.
    ///
    /// The default streams it in bounded chunks; a resident implementation overrides this so the
    /// hash costs one pass and no copies.
    fn sha256(&mut self) -> Result<[u8; 32], RiffPcm24Error> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        let total = self.len();
        let mut offset = 0_u64;
        while offset < total {
            let remaining = usize::try_from(total - offset).unwrap_or(usize::MAX);
            let take = remaining.min(SHA256_CHUNK_BYTES);
            let bytes = self.read_at(offset, take)?;
            if bytes.len() != take {
                return Err(RiffPcm24Error::InvalidInput);
            }
            hasher.update(&bytes);
            offset += take as u64;
        }
        Ok(hasher.finalize().into())
    }
}

/// Bound on one hash read from a streaming source.
const SHA256_CHUNK_BYTES: usize = 1 << 20;

/// Borrowed or supplied PCM byte window behind one parsed RIFF resource.
///
/// The resident variant is the original whole-buffer slice: reads stay zero-copy and every
/// current caller behaves exactly as before. The streaming variant exists so a window can be
/// served by a reader instead of a resident buffer; it keeps the same byte semantics, and the
/// block reader below only ever asks it for exact ranges.
pub enum RiffDataWindow<'a> {
    Resident(&'a [u8]),
    Streamed {
        reader: &'a std::sync::Mutex<Box<dyn RiffDataReader>>,
        /// File offset of this window's first byte.
        base: u64,
        len: usize,
    },
}

impl RiffDataWindow<'_> {
    fn len(&self) -> usize {
        match self {
            Self::Resident(bytes) => bytes.len(),
            Self::Streamed { len, .. } => *len,
        }
    }

    fn read(&self, start: usize, end: usize) -> Result<std::borrow::Cow<'_, [u8]>, RiffPcm24Error> {
        match self {
            Self::Resident(bytes) => bytes
                .get(start..end)
                .map(std::borrow::Cow::Borrowed)
                .ok_or(RiffPcm24Error::InvalidInput),
            Self::Streamed { reader, base, .. } => {
                let len = end.checked_sub(start).ok_or(RiffPcm24Error::InvalidInput)?;
                let offset = base
                    .checked_add(u64::try_from(start).map_err(|_| RiffPcm24Error::InvalidInput)?)
                    .ok_or(RiffPcm24Error::InvalidInput)?;
                let bytes = reader
                    .lock()
                    .map_err(|_| RiffPcm24Error::InvalidInput)?
                    .read_at(offset, len)?;
                if bytes.len() != len {
                    return Err(RiffPcm24Error::InvalidInput);
                }
                Ok(std::borrow::Cow::Owned(bytes))
            }
        }
    }
}

impl Clone for RiffDataWindow<'_> {
    /// Cloning a resident window is a pointer copy. A streamed window cannot be cloned: two
    /// clones would share one reader and interleave reads, so callers that need that must build
    /// a second window instead.
    fn clone(&self) -> Self {
        match self {
            Self::Resident(bytes) => Self::Resident(bytes),
            // A streamed window is a borrowed reader plus an extent: copying the reference keeps
            // both clones reading the same source, which is exactly what the resident variant does
            // when it copies a slice.
            Self::Streamed { reader, base, len } => Self::Streamed {
                reader,
                base: *base,
                len: *len,
            },
        }
    }
}

impl std::fmt::Debug for RiffDataWindow<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resident(bytes) => formatter
                .debug_tuple("Resident")
                .field(&format_args!("{} bytes", bytes.len()))
                .finish(),
            Self::Streamed { base, len, .. } => formatter
                .debug_tuple("Streamed")
                .field(&format_args!("{len} bytes at {base}"))
                .finish(),
        }
    }
}

impl PartialEq for RiffDataWindow<'_> {
    /// Two resident windows compare by content. A streamed window compares by extent only: reading
    /// it to compare would consume the reader and change what the caller observes next.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Resident(left), Self::Resident(right)) => left == right,
            (
                Self::Streamed {
                    base: left_base,
                    len: left_len,
                    ..
                },
                Self::Streamed {
                    base: right_base,
                    len: right_len,
                    ..
                },
            ) => left_base == right_base && left_len == right_len,
            _ => false,
        }
    }
}

impl Eq for RiffDataWindow<'_> {}

/// Parsed selected RIFF/PCM24 metadata and a borrowed data payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedRiffPcm24<'a> {
    sample_rate_hz: u32,
    channels: u16,
    block_align: u16,
    timeline: jocforge_domain::timeline::RenderTimeline,
    source_sha256: [u8; 32],
    resource_bytes: u64,
    data_offset: u64,
    data_payload: RiffDataWindow<'a>,
    chunk_ids: Vec<String>,
    metadata_payloads: Vec<RiffMetadataPayload>,
    selected_metadata_payload: Option<Vec<u8>>,
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
        self.timeline.output_samples()
    }

    pub fn with_render_timeline(
        mut self,
        timeline: jocforge_domain::timeline::RenderTimeline,
    ) -> Result<Self, RiffPcm24Error> {
        if self.timeline.file_samples() != timeline.file_samples() {
            return Err(RiffPcm24Error::InvalidInput);
        }
        self.timeline = timeline;
        Ok(self)
    }

    /// Returns the exact PCM data-chunk payload length observed by the RIFF parser.
    pub fn data_payload_bytes(&self) -> usize {
        self.data_payload.len()
    }

    /// Returns the SHA-256 identity of the complete immutable RIFF byte slice.
    pub const fn source_sha256(&self) -> [u8; 32] {
        self.source_sha256
    }

    /// Whole-resource length, so a caller can rebuild the resource identity without the bytes.
    pub const fn resource_bytes(&self) -> u64 {
        self.resource_bytes
    }

    pub const fn data_offset(&self) -> u64 {
        self.data_offset
    }

    pub fn chunk_ids(&self) -> &[String] {
        &self.chunk_ids
    }

    pub fn metadata_payloads(&self) -> &[RiffMetadataPayload] {
        &self.metadata_payloads
    }

    pub fn selected_metadata_payload(&self) -> Option<&[u8]> {
        self.selected_metadata_payload.as_deref()
    }

    pub const fn has_adm_chunks(&self) -> bool {
        self.has_adm_chunks
    }

    /// Selects one AXML, CHNA and DBMD payload, each borrowed from this parsed resource.
    ///
    /// The slices now borrow the parsed resource rather than the original input bytes, because a
    /// streaming parse owns its metadata.
    pub fn selected_adm_payloads(&self) -> Result<SelectedAdmRiffPayloads<'_>, RiffPcm24Error> {
        let (axml, axml_offset) = unique_metadata_payload(&self.metadata_payloads, *b"axml")?;
        let (chna, chna_offset) = unique_metadata_payload(&self.metadata_payloads, *b"chna")?;
        let (dbmd, dbmd_offset) = unique_metadata_payload(&self.metadata_payloads, *b"dbmd")?;
        Ok(SelectedAdmRiffPayloads {
            axml,
            axml_offset,
            chna,
            chna_offset,
            dbmd,
            dbmd_offset,
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
        if sample_count == 0 || end_sample > self.total_samples() {
            return Err(RiffPcm24Error::InvalidInput);
        }
        Ok(self
            .read_pcm24_chunk(first_sample, sample_count)?
            .into_planar_f32_bits())
    }

    /// Reads one sample chunk, capping its request at this RIFF's sample extent.
    ///
    /// The per-channel sample position is converted to a relative byte offset using
    /// the parsed block alignment. This does not perform an operating-system read.
    pub fn read_pcm24_chunk(
        &self,
        first_sample: AdmSampleIndex,
        requested_frames: u64,
    ) -> Result<DecodedPcm24ReadChunk, RiffPcm24Error> {
        let Some(remaining_frames) = self.total_samples().checked_sub(first_sample.value()) else {
            return decode_pcm24_read_chunk_with_layout(
                &[],
                0,
                usize::from(self.channels),
                usize::from(self.block_align),
            );
        };
        let capped_frames = requested_frames.min(remaining_frames);
        if capped_frames == 0 {
            return decode_pcm24_read_chunk_with_layout(
                &[],
                0,
                usize::from(self.channels),
                usize::from(self.block_align),
            );
        }

        let block_align = u64::from(self.block_align);
        if capped_frames > u64::from(u32::MAX) / block_align {
            return Err(RiffPcm24Error::InvalidInput);
        }
        let (leading, file_offset, file_count) =
            self.timeline.pcm_span(first_sample.value(), capped_frames);
        let byte_start = usize::try_from(
            file_offset
                .checked_mul(block_align)
                .ok_or(RiffPcm24Error::InvalidInput)?,
        )
        .map_err(|_| RiffPcm24Error::InvalidInput)?;
        let bytes = u32::try_from(
            file_count
                .checked_mul(block_align)
                .ok_or(RiffPcm24Error::InvalidInput)?,
        )
        .map_err(|_| RiffPcm24Error::InvalidInput)?;
        let byte_end = byte_start
            .checked_add(bytes as usize)
            .ok_or(RiffPcm24Error::InvalidInput)?;
        let window = self
            .data_payload
            .read(byte_start, byte_end)
            .map_err(|_| RiffPcm24Error::InvalidInput)?;
        let mut chunk = decode_pcm24_read_chunk_with_layout(
            &window,
            bytes,
            usize::from(self.channels),
            usize::from(self.block_align),
        )?;
        if file_count != capped_frames {
            let frames =
                usize::try_from(capped_frames).map_err(|_| RiffPcm24Error::InvalidInput)?;
            let leading = usize::try_from(leading).map_err(|_| RiffPcm24Error::InvalidInput)?;
            for row in &mut chunk.planar_f32_bits {
                let copied = row.len();
                row.resize(frames, 0);
                if leading != 0 {
                    row.copy_within(..copied, leading);
                    row[..leading].fill(0);
                }
            }
            chunk.actual_frames = Pcm24FrameCount(capped_frames);
        }
        Ok(chunk)
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
    decode_pcm24_read_chunk_with_layout(
        read_bytes,
        requested_bytes,
        SELECTED_CHANNELS,
        SELECTED_BLOCK_ALIGN,
    )
}

/// Decodes one interleaved PCM24 read using its declared channel count.
///
/// Rows retain physical interleave order. This describes RIFF PCM rows only;
/// it does not assign them to a DDP frame-view or core processing lane.
pub fn decode_pcm24_read_chunk(
    read_bytes: &[u8],
    requested_bytes: u32,
    channels: u16,
) -> Result<DecodedPcm24ReadChunk, RiffPcm24Error> {
    let channel_count = usize::from(channels);
    let block_align = channel_count
        .checked_mul(3)
        .ok_or(RiffPcm24Error::InvalidInput)?;
    decode_pcm24_read_chunk_with_layout(read_bytes, requested_bytes, channel_count, block_align)
}

fn decode_pcm24_read_chunk_with_layout(
    read_bytes: &[u8],
    requested_bytes: u32,
    channels: usize,
    block_align: usize,
) -> Result<DecodedPcm24ReadChunk, RiffPcm24Error> {
    let actual_bytes = u32::try_from(read_bytes.len()).map_err(|_| RiffPcm24Error::InvalidInput)?;
    if channels == 0 || block_align != channels.saturating_mul(3) || actual_bytes > requested_bytes
    {
        return Err(RiffPcm24Error::InvalidInput);
    }

    let actual_byte_count =
        usize::try_from(actual_bytes).map_err(|_| RiffPcm24Error::InvalidInput)?;
    let frame_count = actual_byte_count / block_align;
    let complete_bytes = frame_count
        .checked_mul(block_align)
        .ok_or(RiffPcm24Error::InvalidInput)?;
    let mut planar_f32_bits = (0..channels)
        .map(|_| Vec::with_capacity(frame_count))
        .collect::<Vec<_>>();

    for frame in 0..frame_count {
        let frame_offset = frame
            .checked_mul(block_align)
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
    metadata: &'a [RiffMetadataPayload],
    wanted_id: [u8; 4],
) -> Result<(&'a [u8], u64), RiffPcm24Error> {
    let mut matches = metadata
        .iter()
        .filter(|chunk| chunk.chunk_id().as_bytes() == wanted_id);
    let chunk = matches.next().ok_or(RiffPcm24Error::InvalidInput)?;
    if matches.next().is_some() {
        return Err(RiffPcm24Error::InvalidInput);
    }
    Ok((chunk.payload(), chunk.payload_offset()))
}

/// Parses a 48 kHz PCM24 RIFF resource while preserving its actual channel count.
///
/// This function accepts complete bytes and performs no filesystem I/O. It does not
/// validate ADM/XML link semantics, RF64/BW64, extensible format, or native malformed-
/// input behavior beyond the explicitly delivered model-reader cases.
pub fn parse_selected_riff_pcm24(
    bytes: &[u8],
    requested_metadata_id: Option<[u8; 4]>,
) -> Result<ParsedRiffPcm24<'_>, RiffPcm24Error> {
    let mut source = SliceRiffDataReader::new(bytes);
    walk_riff(&mut source, requested_metadata_id, |offset, len| {
        let start = usize::try_from(offset).map_err(|_| RiffPcm24Error::InvalidInput)?;
        let end = start.checked_add(len).ok_or(RiffPcm24Error::InvalidInput)?;
        bytes
            .get(start..end)
            .map(RiffDataWindow::Resident)
            .ok_or(RiffPcm24Error::InvalidInput)
    })
}

/// Parses one RIFF resource through a reader, without holding its PCM payload in memory.
///
/// The chunk walk reads only headers and metadata payloads; the `data` payload is skipped and
/// served later through the same reader. `source_sha256` still covers the whole resource, so the
/// parsed identity is identical to the resident parse of the same bytes.
pub fn parse_selected_riff_pcm24_streamed<'a>(
    reader: &'a std::sync::Mutex<Box<dyn RiffDataReader>>,
    requested_metadata_id: Option<[u8; 4]>,
) -> Result<ParsedRiffPcm24<'a>, RiffPcm24Error> {
    let mut locked = LockedReader { reader };
    walk_riff(&mut locked, requested_metadata_id, |offset, len| {
        Ok(RiffDataWindow::Streamed {
            reader,
            base: offset,
            len,
        })
    })
}

/// Adapter that walks a shared reader by locking per read.
///
/// It exists so the parser can take `&mut dyn RiffDataReader` for both a local source and a
/// caller-owned cell. The window builder passed to `walk_riff` runs only after the walk, so this
/// adapter never nests a lock inside another one.
struct LockedReader<'a> {
    reader: &'a std::sync::Mutex<Box<dyn RiffDataReader>>,
}

impl RiffDataReader for LockedReader<'_> {
    fn read_at(&mut self, offset: u64, len: usize) -> Result<Vec<u8>, RiffPcm24Error> {
        self.reader
            .lock()
            .map_err(|_| RiffPcm24Error::InvalidInput)?
            .read_at(offset, len)
    }

    fn len(&self) -> u64 {
        self.reader
            .lock()
            .map(|reader| reader.len())
            .unwrap_or_default()
    }

    fn sha256(&mut self) -> Result<[u8; 32], RiffPcm24Error> {
        self.reader
            .lock()
            .map_err(|_| RiffPcm24Error::InvalidInput)?
            .sha256()
    }
}

/// Walks RIFF chunks through a reader and builds the parsed resource.
///
/// `window` receives the data chunk's file offset and payload length; it decides whether those
/// bytes stay resident or are re-served later. Everything else in the parse is reader-driven, so
/// this is the single validation path for both entry points.
fn walk_riff<'a>(
    read: &mut dyn RiffDataReader,
    requested_metadata_id: Option<[u8; 4]>,
    window: impl FnOnce(u64, usize) -> Result<RiffDataWindow<'a>, RiffPcm24Error>,
) -> Result<ParsedRiffPcm24<'a>, RiffPcm24Error> {
    let resource_len = read.len();
    let header = read.read_at(0, RIFF_HEADER_BYTES)?;
    if header.len() != RIFF_HEADER_BYTES || &header[..4] != b"RIFF" || &header[8..12] != b"WAVE" {
        return Err(RiffPcm24Error::InvalidInput);
    }
    let riff_extent = u64::from(read_u32_le(&header[4..8])?)
        .checked_add(8)
        .ok_or(RiffPcm24Error::InvalidInput)?;
    if riff_extent != resource_len {
        return Err(RiffPcm24Error::InvalidInput);
    }

    let mut position = RIFF_HEADER_BYTES as u64;
    let mut chunk_ids = Vec::new();
    let mut metadata_payloads = Vec::new();
    let mut format: Option<SelectedFormat> = None;
    let mut data_extent: Option<(u64, usize)> = None;
    let mut axml_present = false;
    let mut chna_present = false;

    while position
        .checked_add(CHUNK_HEADER_BYTES as u64)
        .is_some_and(|header_end| header_end <= riff_extent)
    {
        let chunk_header = read.read_at(position, CHUNK_HEADER_BYTES)?;
        if chunk_header.len() != CHUNK_HEADER_BYTES {
            return Err(RiffPcm24Error::InvalidInput);
        }
        let chunk_id: [u8; 4] = chunk_header[..4]
            .try_into()
            .map_err(|_| RiffPcm24Error::InvalidInput)?;
        let payload_bytes = usize::try_from(read_u32_le(&chunk_header[4..8])?)
            .map_err(|_| RiffPcm24Error::InvalidInput)?;
        let payload_start = position + CHUNK_HEADER_BYTES as u64;
        let payload_end = payload_start
            .checked_add(u64::try_from(payload_bytes).map_err(|_| RiffPcm24Error::InvalidInput)?)
            .ok_or(RiffPcm24Error::InvalidInput)?;
        if payload_end > riff_extent {
            return Err(RiffPcm24Error::InvalidInput);
        }
        chunk_ids.push(decode_chunk_id(chunk_id));

        match &chunk_id {
            b"fmt " => {
                if format.is_some() {
                    return Err(RiffPcm24Error::InvalidInput);
                }
                let payload = read.read_at(payload_start, payload_bytes)?;
                if payload.len() != payload_bytes {
                    return Err(RiffPcm24Error::InvalidInput);
                }
                format = Some(parse_selected_format(&payload)?);
            }
            b"data" => {
                if data_extent.is_some() || payload_bytes == 0 {
                    return Err(RiffPcm24Error::InvalidInput);
                }
                data_extent = Some((payload_start, payload_bytes));
            }
            b"axml" | b"chna" | b"dbmd" | b"bext" | b"iXML" => {
                let chunk_name = decode_chunk_id(chunk_id);
                axml_present |= chunk_id == *b"axml";
                chna_present |= chunk_id == *b"chna";
                let payload = read.read_at(payload_start, payload_bytes)?;
                if payload.len() != payload_bytes {
                    return Err(RiffPcm24Error::InvalidInput);
                }
                metadata_payloads.push(RiffMetadataPayload {
                    chunk_id: chunk_name,
                    payload,
                    payload_offset: payload_start,
                });
            }
            _ => {}
        }

        position = payload_end
            .checked_add(u64::from(payload_bytes as u32 % 2))
            .ok_or(RiffPcm24Error::InvalidInput)?;
    }

    let format = format.ok_or(RiffPcm24Error::InvalidInput)?;
    let (data_offset, data_len) = data_extent.ok_or(RiffPcm24Error::InvalidInput)?;
    let block_align = usize::from(format.block_align);
    if block_align == 0 || data_len % block_align != 0 {
        return Err(RiffPcm24Error::InvalidInput);
    }
    let source_sha256 = read.sha256()?;
    let selected_metadata_payload = if let Some(requested_id) = requested_metadata_id {
        let mut matches = metadata_payloads
            .iter()
            .filter(|chunk| chunk.chunk_id.as_bytes() == requested_id);
        let first = matches.next().map(|chunk| chunk.payload.clone());
        if matches.next().is_some() {
            return Err(RiffPcm24Error::InvalidInput);
        }
        first
    } else {
        None
    };
    let data_payload = window(data_offset, data_len)?;

    Ok(ParsedRiffPcm24 {
        sample_rate_hz: format.sample_rate_hz,
        channels: format.channels,
        block_align: format.block_align,
        timeline: jocforge_domain::timeline::RenderTimeline::full(
            u64::try_from(data_len / block_align).map_err(|_| RiffPcm24Error::InvalidInput)?,
        ),
        source_sha256,
        resource_bytes: resource_len,
        data_offset,
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
    let expected_block_align = u32::from(channels)
        .checked_mul(u32::from(bits_per_sample / 8))
        .ok_or(RiffPcm24Error::InvalidInput)?;
    let expected_byte_rate = sample_rate_hz
        .checked_mul(expected_block_align)
        .ok_or(RiffPcm24Error::InvalidInput)?;
    // WAVE_FORMAT_EXTENSIBLE describes the same packed samples when its subtype is
    // KSDATAFORMAT_SUBTYPE_PCM (GUID fields in RIFF byte order). ADM CHNA/AXML, not the
    // optional speaker mask, provide the association between physical tracks and ADM roles.
    let packed_pcm = match format_tag {
        1 => true,
        0xfffe if payload.len() >= 40 => {
            let extension_len = usize::from(read_u16_le(&payload[16..18])?);
            extension_len >= 22
                && extension_len <= payload.len() - 18
                && read_u16_le(&payload[18..20])? == SELECTED_BITS_PER_SAMPLE
                && payload[24..40] == PCM_SUBTYPE
        }
        _ => false,
    };
    if !packed_pcm
        || channels == 0
        || sample_rate_hz != SELECTED_SAMPLE_RATE_HZ
        || bits_per_sample != SELECTED_BITS_PER_SAMPLE
        || expected_block_align != u32::from(block_align)
        || expected_byte_rate != byte_rate
    {
        return Err(RiffPcm24Error::InvalidInput);
    }
    Ok(SelectedFormat {
        channels,
        sample_rate_hz,
        block_align,
    })
}

/// Re-serves one parsed resource's PCM window from a reader instead of resident bytes.
///
/// The caller must supply the same data extent the parse observed. Nothing else about the parsed
/// resource changes: metadata payloads, timeline and block alignment stay as parsed, so every
/// block read below behaves exactly as it did against the resident slice.
pub fn streamed_window<'a>(
    parsed: &ParsedRiffPcm24<'a>,
    reader: &'a std::sync::Mutex<Box<dyn RiffDataReader>>,
) -> ParsedRiffPcm24<'a> {
    let mut streamed = parsed.clone();
    streamed.data_payload = RiffDataWindow::Streamed {
        reader,
        base: parsed.data_offset(),
        len: parsed.data_payload.len(),
    };
    streamed
}

/// Native reader over one already open RIFF file positioned at its data-chunk payload.
///
/// It exists to prove the streaming window is byte-equivalent to the resident one before the
/// browser target grows a reader; it performs ordinary seek+read_exact I/O.
#[cfg(not(target_arch = "wasm32"))]
pub struct FileRiffDataReader {
    file: std::fs::File,
    len: u64,
}

#[cfg(not(target_arch = "wasm32"))]
impl FileRiffDataReader {
    /// Opens `path`; every offset is a file offset.
    pub fn open(path: &std::path::Path) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let len = file.metadata()?.len();
        Ok(Self { file, len })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl RiffDataReader for FileRiffDataReader {
    fn read_at(&mut self, offset: u64, len: usize) -> Result<Vec<u8>, RiffPcm24Error> {
        use std::io::{Read, Seek, SeekFrom};
        self.file
            .seek(SeekFrom::Start(offset))
            .map_err(|_| RiffPcm24Error::InvalidInput)?;
        let mut bytes = vec![0_u8; len];
        self.file
            .read_exact(&mut bytes)
            .map_err(|_| RiffPcm24Error::InvalidInput)?;
        Ok(bytes)
    }

    fn len(&self) -> u64 {
        self.len
    }
}

/// A reader over bytes that are already resident, so the parser has exactly one code path.
pub struct SliceRiffDataReader<'a> {
    bytes: &'a [u8],
}

impl<'a> SliceRiffDataReader<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}

impl RiffDataReader for SliceRiffDataReader<'_> {
    fn sha256(&mut self) -> Result<[u8; 32], RiffPcm24Error> {
        use sha2::{Digest, Sha256};
        Ok(Sha256::digest(self.bytes).into())
    }

    fn read_at(&mut self, offset: u64, len: usize) -> Result<Vec<u8>, RiffPcm24Error> {
        let start = usize::try_from(offset).map_err(|_| RiffPcm24Error::InvalidInput)?;
        let end = start.checked_add(len).ok_or(RiffPcm24Error::InvalidInput)?;
        self.bytes
            .get(start..end)
            .map(<[u8]>::to_vec)
            .ok_or(RiffPcm24Error::InvalidInput)
    }

    fn len(&self) -> u64 {
        self.bytes.len() as u64
    }
}

/// Browser reader over a file the host worker keeps in OPFS.
///
/// The worker exposes two synchronous imports: `jocforge_host_len()` and
/// `jocforge_host_read(offset, len, dst)`. A `FileSystemSyncAccessHandle` answers the second one while
/// this module is on the stack, so the synchronous pipeline never needs shared memory or atomics.
/// Browser reader over a file the host worker keeps in OPFS.
///
/// The worker exposes two synchronous imports: `jocforge_host_len()` and
/// `jocforge_host_read(offset, len, dst)`. A `FileSystemSyncAccessHandle` answers the second one while
/// this module is on the stack, so the synchronous pipeline never needs shared memory or atomics.
/// One module-level `allow(unsafe_code)` covers the whole sanctioned FFI boundary.
#[cfg(target_arch = "wasm32")]
#[allow(unsafe_code)]
mod host_reader {
    use super::{RiffDataReader, RiffPcm24Error};

    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        fn jocforge_host_read(offset: u64, len: u32, destination: *mut u8) -> i32;
        fn jocforge_host_len() -> u64;
    }

    pub struct HostRiffDataReader;

    impl RiffDataReader for HostRiffDataReader {
        fn read_at(&mut self, offset: u64, len: usize) -> Result<Vec<u8>, RiffPcm24Error> {
            let len_u32 = u32::try_from(len).map_err(|_| RiffPcm24Error::InvalidInput)?;
            let mut buffer = vec![0_u8; len];
            // SAFETY: the host writes at most `len` bytes into a buffer this module owns and reports
            // failure through the return code.
            let status = unsafe { jocforge_host_read(offset, len_u32, buffer.as_mut_ptr()) };
            if status != 0 {
                return Err(RiffPcm24Error::InvalidInput);
            }
            Ok(buffer)
        }

        fn len(&self) -> u64 {
            // SAFETY: the host import only reads its own state.
            unsafe { jocforge_host_len() }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use host_reader::HostRiffDataReader;

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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod streamed_window_tests {
    use super::*;

    /// Builds one small but structurally valid PCM24 RIFF: JUNK, fmt, data, then a tail chunk.
    fn synthetic_riff(frames: usize) -> Vec<u8> {
        let block_align = 3 * 6; // six channels, 24-bit
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.extend_from_slice(b"WAVE");

        let junk = [0_u8; 16];
        bytes.extend_from_slice(b"JUNK");
        bytes.extend_from_slice(&(junk.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&junk);

        let mut fmt = Vec::new();
        fmt.extend_from_slice(&1_u16.to_le_bytes());
        fmt.extend_from_slice(&6_u16.to_le_bytes());
        fmt.extend_from_slice(&48_000_u32.to_le_bytes());
        fmt.extend_from_slice(&(48_000_u32 * block_align as u32).to_le_bytes());
        fmt.extend_from_slice(&(block_align as u16).to_le_bytes());
        fmt.extend_from_slice(&24_u16.to_le_bytes());
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&(fmt.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&fmt);

        let mut data = Vec::with_capacity(frames * block_align);
        for frame in 0..frames {
            for channel in 0..6 {
                let value = ((frame * 6 + channel) as u32 & 0x00ff_ffff).to_le_bytes();
                data.extend_from_slice(&value[..3]);
            }
        }
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&data);

        let tail = b"tail-metadata";
        bytes.extend_from_slice(b"bext");
        bytes.extend_from_slice(&(tail.len() as u32).to_le_bytes());
        bytes.extend_from_slice(tail);

        let riff_size = (bytes.len() - 8) as u32;
        bytes[4..8].copy_from_slice(&riff_size.to_le_bytes());
        bytes
    }

    fn digest_blocks(parsed: &ParsedRiffPcm24<'_>) -> Vec<u32> {
        let mut digest = Vec::new();
        let total = parsed.total_samples();
        let mut first = 0_u64;
        while first < total {
            let chunk = parsed
                .read_pcm24_chunk(AdmSampleIndex::new(first), 1_536)
                .expect("block read");
            for row in chunk.planar_f32_bits() {
                digest.extend_from_slice(row);
            }
            first += chunk.actual_frames().value().max(1);
        }
        digest
    }

    #[test]
    fn extensible_input_requires_packed_pcm24() {
        // fmt payload written by FFmpeg for 11 channels of 48 kHz packed s24le.
        let mut fmt = [
            0xfe, 0xff, 11, 0, 0x80, 0xbb, 0, 0, 0x80, 0x2b, 0x18, 0, 33, 0, 24, 0, 22, 0, 24, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0x10, 0, 0x80, 0, 0, 0xaa, 0, 0x38, 0x9b, 0x71,
        ];
        let parsed = parse_selected_format(&fmt).expect("ordinary extensible PCM24");
        assert_eq!(
            (parsed.channels, parsed.sample_rate_hz, parsed.block_align),
            (11, 48_000, 33)
        );

        fmt[18] = 20; // 20 valid bits in 24 is not the selected format.
        assert!(parse_selected_format(&fmt).is_err());
        fmt[18] = 24;
        // Ambisonic B-format PCM shares the first four bytes, not the full subtype GUID.
        fmt[24..40].copy_from_slice(&[
            1, 0, 0, 0, 0x21, 0x07, 0xd3, 0x11, 0x86, 0x44, 0xc8, 0xc1, 0xca, 0, 0, 0,
        ]);
        assert!(parse_selected_format(&fmt).is_err());
    }

    #[test]
    fn streamed_window_decodes_exactly_the_resident_blocks() {
        let riff = synthetic_riff(4_096);
        let resident = parse_selected_riff_pcm24(&riff, None).expect("resident parse");
        let wanted = digest_blocks(&resident);
        assert!(!wanted.is_empty());

        let path = std::env::temp_dir().join(format!(
            "riff-stream-{}-{}.wav",
            std::process::id(),
            resident.data_offset()
        ));
        std::fs::write(&path, &riff).expect("write fixture");
        let reader = std::sync::Mutex::new(Box::new(
            FileRiffDataReader::open(&path).expect("open fixture"),
        ) as Box<dyn RiffDataReader>);
        let streamed = streamed_window(&resident, &reader);

        let got = digest_blocks(&streamed);
        std::fs::remove_file(&path).ok();
        assert_eq!(got, wanted, "streamed blocks must match resident blocks");
    }

    /// A reader that always returns fewer bytes than asked for.
    struct ShortReader;

    impl RiffDataReader for ShortReader {
        fn read_at(&mut self, _offset: u64, _len: usize) -> Result<Vec<u8>, RiffPcm24Error> {
            Ok(Vec::new())
        }

        fn len(&self) -> u64 {
            0
        }
    }

    #[test]
    fn streamed_parse_observes_exactly_what_the_resident_parse_does() {
        let riff = synthetic_riff(2_048);
        let resident = parse_selected_riff_pcm24(&riff, Some(*b"bext")).expect("resident parse");

        let path = std::env::temp_dir().join(format!(
            "riff-streamed-parse-{}-{}.wav",
            std::process::id(),
            riff.len()
        ));
        std::fs::write(&path, &riff).expect("write fixture");
        let reader = std::sync::Mutex::new(Box::new(
            FileRiffDataReader::open(&path).expect("open fixture"),
        ) as Box<dyn RiffDataReader>);
        let streamed =
            parse_selected_riff_pcm24_streamed(&reader, Some(*b"bext")).expect("streamed parse");

        assert_eq!(streamed.source_sha256(), resident.source_sha256());
        assert_eq!(streamed.chunk_ids(), resident.chunk_ids());
        assert_eq!(streamed.metadata_payloads(), resident.metadata_payloads());
        assert_eq!(
            streamed.selected_metadata_payload(),
            resident.selected_metadata_payload()
        );
        assert_eq!(streamed.total_samples(), resident.total_samples());
        assert_eq!(streamed.block_align(), resident.block_align());
        assert_eq!(streamed.channels(), resident.channels());
        assert_eq!(streamed.data_offset(), resident.data_offset());
        assert_eq!(streamed.resource_bytes(), riff.len() as u64);
        assert_eq!(streamed.resource_bytes(), resident.resource_bytes());
        assert_eq!(streamed.has_adm_chunks(), resident.has_adm_chunks());
        assert_eq!(digest_blocks(&streamed), digest_blocks(&resident));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn streamed_parse_rejects_a_resource_whose_extent_disagrees() {
        let mut riff = synthetic_riff(128);
        // Declare one more byte than the resource holds.
        let declared = (riff.len() - 8 + 1) as u32;
        riff[4..8].copy_from_slice(&declared.to_le_bytes());

        let path = std::env::temp_dir().join(format!(
            "riff-bad-extent-{}-{}.wav",
            std::process::id(),
            riff.len()
        ));
        std::fs::write(&path, &riff).expect("write fixture");
        let reader = std::sync::Mutex::new(Box::new(
            FileRiffDataReader::open(&path).expect("open fixture"),
        ) as Box<dyn RiffDataReader>);
        let outcome = parse_selected_riff_pcm24_streamed(&reader, None);
        std::fs::remove_file(&path).ok();
        assert!(matches!(outcome, Err(RiffPcm24Error::InvalidInput)));
    }
    #[test]
    fn streamed_window_rejects_a_short_reader() {
        let riff = synthetic_riff(64);
        let resident = parse_selected_riff_pcm24(&riff, None).expect("resident parse");
        let reader = std::sync::Mutex::new(Box::new(ShortReader) as Box<dyn RiffDataReader>);
        let streamed = streamed_window(&resident, &reader);
        assert!(matches!(
            streamed.read_pcm24_chunk(AdmSampleIndex::new(0), 1_536),
            Err(RiffPcm24Error::InvalidInput)
        ));
    }
}
