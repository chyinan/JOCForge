// pattern: Functional Core

use std::{
    collections::{BTreeSet, HashMap},
    f64::consts::PI,
    fmt,
    fmt::Write as _,
};

const CHANNEL_COUNT: usize = 15;
const SAMPLE_RATE: u32 = 48_000;
const RANK: usize = 7;
const QUANTUM: f64 = 820.0 / 4096.0;
// ADM fixture/export timestamps carry five fractional decimal places; this
// is one timestamp unit and is smaller than a 48 kHz sample period.
const ADM_TIME_EPSILON: f64 = 1.0e-5;

/// One supported dynamic ADM object after BW64/chna resolution.
#[derive(Clone, Debug, PartialEq)]
pub struct AdmObject {
    pub ordinal: usize,
    pub object_id: String,
    pub object_name: String,
    pub start: Option<String>,
    pub duration: Option<String>,
    pub pack_format_id: String,
    pub pack_format_name: String,
    pub track_uid: String,
    pub track_index: usize,
    pub position: AdmPosition,
    pub kind: AdmObjectKind,
    pub channels: Vec<AdmChannel>,
}

/// One channel binding inside an ADM object, including its explicit PCM track.
#[derive(Clone, Debug, PartialEq)]
pub struct AdmChannel {
    pub channel_id: String,
    pub channel_name: String,
    /// Public ADM DirectSpeakers label, when supplied by the channel block.
    pub speaker_label: Option<String>,
    /// Public ADM frequency low-pass declaration, when supplied by the channel.
    pub low_pass: Option<f64>,
    pub track_uid: String,
    pub track_index: usize,
    pub position: AdmPosition,
    pub is_lfe: bool,
    pub block_rtime: Option<String>,
    pub block_duration: Option<String>,
    pub gain: Option<String>,
}

/// ADM object category retained for the source-file census.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmObjectKind {
    Objects,
    DirectSpeakers,
    Other,
}

impl AdmObjectKind {
    #[must_use]
    pub fn is_dynamic(self) -> bool {
        self == Self::Objects
    }
}

/// Cartesian-free ADM position values retained for OAMD conversion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdmPosition {
    pub azimuth: f64,
    pub elevation: f64,
    pub distance: f64,
}

impl AdmPosition {
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.azimuth.is_finite() && self.elevation.is_finite() && self.distance.is_finite()
    }
}

/// Narrow parsed ADM/BW64 input used by the MVP encoder shell.
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedAdm {
    pub sample_rate: u32,
    pub channel_count: usize,
    pub sample_count: usize,
    pub programmes: Vec<AdmProgramme>,
    pub objects: Vec<AdmObject>,
    pub pcm: Vec<Vec<f64>>,
}

/// ADM programme-to-content relationship retained for source inventory.
#[derive(Clone, Debug, PartialEq)]
pub struct AdmProgramme {
    pub programme_id: String,
    pub content_id_refs: Vec<String>,
    pub start: Option<String>,
    pub end: Option<String>,
}

/// ADM/BW64 metadata parsed without loading the PCM payload.
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedAdmMetadata {
    pub sample_rate: u32,
    pub channel_count: usize,
    pub sample_count: usize,
    pub data_bytes: u64,
    pub track_count: usize,
    pub track_uid_count: usize,
    pub programmes: Vec<AdmProgramme>,
    pub objects: Vec<AdmObject>,
}

impl ParsedAdmMetadata {
    #[must_use]
    pub fn dynamic_object_count(&self) -> usize {
        self.objects
            .iter()
            .filter(|object| object.kind.is_dynamic())
            .count()
    }

    #[must_use]
    pub fn direct_speaker_count(&self) -> usize {
        self.objects
            .iter()
            .filter(|object| object.kind == AdmObjectKind::DirectSpeakers)
            .count()
    }

    #[must_use]
    pub fn direct_speaker_channel_count(&self) -> usize {
        self.objects
            .iter()
            .filter(|object| object.kind == AdmObjectKind::DirectSpeakers)
            .map(|object| object.channels.len())
            .sum()
    }
}

/// Repository-owned fixture bytes and the ground-truth rank-7 matrix.
#[derive(Clone, Debug, PartialEq)]
pub struct SyntheticFixture {
    pub bw64_header_and_pcm: Vec<u8>,
    pub matrix: Vec<Vec<f64>>,
}

/// Errors from the deliberately narrow ADM/BW64 subset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmError {
    Truncated(&'static str),
    InvalidContainer(&'static str),
    MissingChunk(&'static str),
    UnsupportedPcm { format: u16, bits: u16 },
    InvalidMetadata(String),
    MissingObjectBinding(String),
    MissingPosition(String),
    InvalidNumber(String),
}

impl fmt::Display for AdmError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated(part) => write!(formatter, "truncated BW64 {part}"),
            Self::InvalidContainer(part) => write!(formatter, "invalid BW64 {part}"),
            Self::MissingChunk(chunk) => write!(formatter, "missing required ADM chunk '{chunk}'"),
            Self::UnsupportedPcm { format, bits } => {
                write!(
                    formatter,
                    "unsupported PCM format {format} with {bits} bits"
                )
            }
            Self::InvalidMetadata(error) => write!(formatter, "invalid ADM metadata: {error}"),
            Self::MissingObjectBinding(object) => {
                write!(
                    formatter,
                    "missing TrackUID binding for ADM object {object}"
                )
            }
            Self::MissingPosition(object) => {
                write!(formatter, "missing position for ADM object {object}")
            }
            Self::InvalidNumber(value) => write!(formatter, "invalid ADM numeric value '{value}'"),
        }
    }
}

impl std::error::Error for AdmError {}

/// Parses the narrow BW64/RF64/RIFF PCM + `axml` + `chna` subset.
///
/// # Panics
///
/// This parser validates chunk and sample bounds before indexing and does not
/// panic for arbitrary input bytes.
pub fn parse_bw64_adm(bytes: &[u8]) -> Result<ParsedAdm, AdmError> {
    if bytes.len() < 12 {
        return Err(AdmError::Truncated("header"));
    }
    let form = &bytes[0..4];
    if !matches!(form, b"BW64" | b"RF64" | b"RIFF") || &bytes[8..12] != b"WAVE" {
        return Err(AdmError::InvalidContainer("RIFF form"));
    }
    let mut cursor = 12_usize;
    let mut ds64_data_size = None;
    let mut fmt = None;
    let mut axml = None;
    let mut chna = None;
    let mut data_range = None;
    while cursor + 8 <= bytes.len() {
        let id = &bytes[cursor..cursor + 4];
        let declared = read_u32_le(&bytes[cursor + 4..cursor + 8])?;
        let body_start = cursor + 8;
        let body_size = if id == b"data" && declared == u32::MAX {
            usize::try_from(ds64_data_size.ok_or(AdmError::MissingChunk("ds64"))?)
                .map_err(|_| AdmError::InvalidContainer("data size"))?
        } else {
            usize::try_from(declared).map_err(|_| AdmError::InvalidContainer("chunk size"))?
        };
        let body_end = body_start
            .checked_add(body_size)
            .ok_or(AdmError::InvalidContainer("chunk bounds"))?;
        if body_end > bytes.len() {
            if id == b"data" && axml.is_none() {
                return Err(AdmError::MissingChunk("axml"));
            }
            if id == b"data" && chna.is_none() {
                return Err(AdmError::MissingChunk("chna"));
            }
            return Err(AdmError::Truncated("chunk body"));
        }
        match id {
            b"ds64" => {
                if body_size < 24 {
                    return Err(AdmError::Truncated("ds64"));
                }
                ds64_data_size = Some(read_u64_le(&bytes[body_start + 8..body_start + 16])?);
            }
            b"fmt " => fmt = Some(bytes[body_start..body_end].to_vec()),
            b"axml" => axml = Some(bytes[body_start..body_end].to_vec()),
            b"chna" => chna = Some(bytes[body_start..body_end].to_vec()),
            b"data" => data_range = Some(body_start..body_end),
            _ => {}
        }
        cursor = body_end + (body_size % 2);
    }
    let axml = axml.ok_or(AdmError::MissingChunk("axml"))?;
    let chna = chna.ok_or(AdmError::MissingChunk("chna"))?;
    let fmt = fmt.ok_or(AdmError::MissingChunk("fmt "))?;
    let data_range = data_range.ok_or(AdmError::MissingChunk("data"))?;
    if fmt.len() < 16 {
        return Err(AdmError::Truncated("fmt chunk"));
    }
    let format = read_u16_le(&fmt[0..2])?;
    let channel_count = usize::from(read_u16_le(&fmt[2..4])?);
    let block_align = usize::from(read_u16_le(&fmt[12..14])?);
    let bits = read_u16_le(&fmt[14..16])?;
    if format != 1 || !matches!(bits, 16 | 24 | 32) {
        return Err(AdmError::UnsupportedPcm { format, bits });
    }
    let bytes_per_sample = usize::from(bits / 8);
    if block_align != channel_count * bytes_per_sample || channel_count == 0 {
        return Err(AdmError::InvalidContainer("PCM block alignment"));
    }
    if data_range.len() % block_align != 0 {
        return Err(AdmError::InvalidContainer("PCM data alignment"));
    }
    let metadata = parse_bw64_adm_metadata(&fmt, &axml, &chna, data_range.len() as u64)?;
    let pcm = decode_pcm(&bytes[data_range], channel_count, bits);
    Ok(ParsedAdm {
        sample_rate: metadata.sample_rate,
        channel_count: metadata.channel_count,
        sample_count: metadata.sample_count,
        programmes: metadata.programmes,
        objects: metadata.objects,
        pcm,
    })
}

/// Parses ADM/BW64 metadata from already extracted chunks without reading PCM.
pub fn parse_bw64_adm_metadata(
    fmt: &[u8],
    axml: &[u8],
    chna: &[u8],
    data_bytes: u64,
) -> Result<ParsedAdmMetadata, AdmError> {
    if fmt.len() < 16 {
        return Err(AdmError::Truncated("fmt chunk"));
    }
    let format = read_u16_le(&fmt[0..2])?;
    let channel_count = usize::from(read_u16_le(&fmt[2..4])?);
    let sample_rate = read_u32_le(&fmt[4..8])?;
    let block_align = usize::from(read_u16_le(&fmt[12..14])?);
    let bits = read_u16_le(&fmt[14..16])?;
    if format != 1 || !matches!(bits, 16 | 24 | 32) {
        return Err(AdmError::UnsupportedPcm { format, bits });
    }
    let bytes_per_sample = usize::from(bits / 8);
    if channel_count == 0 || block_align != channel_count * bytes_per_sample {
        return Err(AdmError::InvalidContainer("PCM block alignment"));
    }
    let block_align_u64 = u64::try_from(block_align)
        .map_err(|_| AdmError::InvalidContainer("PCM block alignment"))?;
    if data_bytes % block_align_u64 != 0 {
        return Err(AdmError::InvalidContainer("PCM data alignment"));
    }
    let sample_count = usize::try_from(data_bytes / block_align_u64)
        .map_err(|_| AdmError::InvalidContainer("PCM sample count"))?;
    let chna = parse_chna(chna)?;
    let xml = std::str::from_utf8(axml)
        .map_err(|_| AdmError::InvalidMetadata("axml is not UTF-8".to_owned()))?;
    let programmes = parse_adm_programmes(xml)?;
    validate_programme_timing(&programmes, sample_count)?;
    let programme_object_ids = parse_programme_object_ids(xml, &programmes)?;
    let objects = parse_adm_objects(xml, &chna.bindings, channel_count, sample_count)?;
    if !programme_object_ids.is_empty() {
        for object in &objects {
            if !programme_object_ids.contains(&object.object_id) {
                return Err(AdmError::InvalidMetadata(format!(
                    "object {} is not referenced by the programme",
                    object.object_id
                )));
            }
        }
        for object_id in programme_object_ids {
            if !objects.iter().any(|object| object.object_id == object_id) {
                return Err(AdmError::InvalidMetadata(format!(
                    "programme references unknown audioObject {object_id}"
                )));
            }
        }
    }
    if sample_rate != SAMPLE_RATE {
        return Err(AdmError::InvalidMetadata(format!(
            "MVP requires 48000 Hz, got {sample_rate}"
        )));
    }
    Ok(ParsedAdmMetadata {
        sample_rate,
        channel_count,
        sample_count,
        data_bytes,
        track_count: chna.track_count,
        track_uid_count: chna.track_uid_count,
        programmes,
        objects,
    })
}

fn parse_adm_programmes(xml: &str) -> Result<Vec<AdmProgramme>, AdmError> {
    tag_blocks(xml, "audioProgramme")
        .into_iter()
        .map(|block| {
            let open = opening_tag(block, "audioProgramme")?;
            Ok(AdmProgramme {
                programme_id: attribute(open, "audioProgrammeID")?,
                content_id_refs: child_texts(block, "audioContentIDRef")?,
                start: attribute_optional(open, "start"),
                end: attribute_optional(open, "end"),
            })
        })
        .collect()
}

fn validate_programme_timing(
    programmes: &[AdmProgramme],
    sample_count: usize,
) -> Result<(), AdmError> {
    if programmes.len() > 1 {
        return Err(AdmError::InvalidMetadata(
            "multiple audioProgramme elements are unsupported by the narrow encoder".to_owned(),
        ));
    }
    let full_duration = sample_count as f64 / f64::from(SAMPLE_RATE);
    for programme in programmes {
        let start = programme
            .start
            .as_deref()
            .map(|value| parse_adm_time(value).map_err(|()| value))
            .transpose()
            .map_err(|value| {
                AdmError::InvalidMetadata(format!(
                    "{} programme timing uses unsupported ADM time '{value}'",
                    programme.programme_id
                ))
            })?;
        let end = programme
            .end
            .as_deref()
            .map(|value| parse_adm_time(value).map_err(|()| value))
            .transpose()
            .map_err(|value| {
                AdmError::InvalidMetadata(format!(
                    "{} programme timing uses unsupported ADM time '{value}'",
                    programme.programme_id
                ))
            })?;
        match (start, end) {
            (Some(start), Some(end))
                if end >= start
                    && (end - start - full_duration).abs()
                        <= ADM_TIME_EPSILON * full_duration.max(1.0) => {}
            (None, None) => {}
            _ => {
                return Err(AdmError::InvalidMetadata(format!(
                    "{} programme timing does not cover the full PCM duration",
                    programme.programme_id
                )));
            }
        }
    }
    Ok(())
}

fn parse_programme_object_ids(
    xml: &str,
    programmes: &[AdmProgramme],
) -> Result<BTreeSet<String>, AdmError> {
    if programmes.is_empty() {
        return Ok(BTreeSet::new());
    }
    let mut content_objects = HashMap::new();
    for block in tag_blocks(xml, "audioContent") {
        let open = opening_tag(block, "audioContent")?;
        let content_id = attribute(open, "audioContentID")?;
        content_objects.insert(content_id, child_texts(block, "audioObjectIDRef")?);
    }
    let mut object_ids = BTreeSet::new();
    for programme in programmes {
        if programme.content_id_refs.is_empty() {
            return Err(AdmError::InvalidMetadata(format!(
                "programme {} has no content references",
                programme.programme_id
            )));
        }
        for content_id in &programme.content_id_refs {
            let objects = content_objects.get(content_id).ok_or_else(|| {
                AdmError::InvalidMetadata(format!(
                    "programme {} references unknown audioContent {content_id}",
                    programme.programme_id
                ))
            })?;
            object_ids.extend(objects.iter().cloned());
        }
    }
    if object_ids.is_empty() {
        return Err(AdmError::InvalidMetadata(
            "programme has no audioObject references".to_owned(),
        ));
    }
    Ok(object_ids)
}

/// Decodes selected interleaved integer PCM tracks from one bounded frame.
///
/// The file shell supplies only the bytes for the requested frame; this pure
/// conversion keeps large-programme ingest from requiring a whole-file PCM
/// allocation.
pub fn decode_pcm_frame(
    bytes: &[u8],
    channel_count: usize,
    bits: u16,
    track_indices: &[usize],
) -> Result<Vec<Vec<f64>>, AdmError> {
    if channel_count == 0 || !matches!(bits, 16 | 24 | 32) {
        return Err(AdmError::UnsupportedPcm { format: 1, bits });
    }
    let bytes_per_sample = usize::from(bits / 8);
    let block_align = channel_count
        .checked_mul(bytes_per_sample)
        .ok_or(AdmError::InvalidContainer("PCM block alignment"))?;
    if bytes.len() % block_align != 0 {
        return Err(AdmError::InvalidContainer("PCM frame alignment"));
    }
    if track_indices
        .iter()
        .any(|track_index| *track_index == 0 || *track_index > channel_count)
    {
        return Err(AdmError::InvalidContainer("PCM track index"));
    }
    let sample_count = bytes.len() / block_align;
    let mut output = vec![Vec::with_capacity(sample_count); track_indices.len()];
    for sample in 0..sample_count {
        for (output_index, track_index) in track_indices.iter().copied().enumerate() {
            let offset = sample * block_align + (track_index - 1) * bytes_per_sample;
            let value = match bits {
                16 => f64::from(read_i16_le(bytes, offset)) / 32768.0,
                24 => {
                    let raw = i32::from(*bytes.get(offset).unwrap_or(&0))
                        | (i32::from(*bytes.get(offset + 1).unwrap_or(&0)) << 8)
                        | (i32::from(*bytes.get(offset + 2).unwrap_or(&0)) << 16);
                    let signed = if raw & 0x80_0000 != 0 {
                        raw | !0xFF_FFFF
                    } else {
                        raw
                    };
                    f64::from(signed) / 8_388_608.0
                }
                32 => f64::from(read_i32_le(bytes, offset)) / 2_147_483_648.0,
                _ => unreachable!("PCM bit depth validated above"),
            };
            output[output_index].push(value);
        }
    }
    Ok(output)
}

/// Validates the ADM metadata boundary required by source-derived encoding.
pub fn validate_source_derived_adm_metadata(metadata: &ParsedAdmMetadata) -> Result<(), AdmError> {
    if metadata.programmes.is_empty() {
        return Err(AdmError::InvalidMetadata(
            "source-derived encode requires an audioProgramme for object membership".to_owned(),
        ));
    }
    Ok(())
}

/// Generates a four-second, 15-object, rank-7 synthetic ADM/BW64 fixture.
pub fn generate_synthetic_fixture(duration_seconds: u32) -> Result<SyntheticFixture, String> {
    generate_synthetic_scene_fixture(duration_seconds, CHANNEL_COUNT)
}

/// Generates a repository-owned rank-7 synthetic fixture with a configurable
/// authored-object count for clean reduction and acceptance tests.
pub fn generate_synthetic_scene_fixture(
    duration_seconds: u32,
    object_count: usize,
) -> Result<SyntheticFixture, String> {
    generate_fixture_with_layout(duration_seconds, object_count, &[])
}

/// Generates a small mixed ADM fixture for DirectSpeakers causal validation.
/// The layout contains two non-LFE speakers, one LFE speaker, and two objects.
pub fn generate_synthetic_direct_speaker_fixture(
    duration_seconds: u32,
) -> Result<SyntheticFixture, String> {
    let direct_channels = [
        FixtureDirectChannel {
            channel_id: "AC_00011001",
            channel_name: "RoomCentricLeft",
            speaker_label: "M-030",
            position: AdmPosition {
                azimuth: -1.0,
                elevation: 0.0,
                distance: 1.0,
            },
        },
        FixtureDirectChannel {
            channel_id: "AC_00011002",
            channel_name: "RoomCentricRight",
            speaker_label: "M+030",
            position: AdmPosition {
                azimuth: 1.0,
                elevation: 0.0,
                distance: 1.0,
            },
        },
        FixtureDirectChannel {
            channel_id: "AC_00011003",
            channel_name: "RoomCentricLFE",
            speaker_label: "LFE1",
            position: AdmPosition {
                azimuth: 0.0,
                elevation: 0.0,
                distance: -1.0,
            },
        },
    ];
    generate_fixture_with_layout(duration_seconds, 2, &direct_channels)
}

/// Generates a deterministic ADM fixture whose pack type is intentionally
/// outside the supported Objects/DirectSpeakers subset.
pub fn generate_synthetic_unsupported_fixture(
    duration_seconds: u32,
) -> Result<SyntheticFixture, String> {
    let mut fixture = generate_synthetic_scene_fixture(duration_seconds, 1)?;
    let marker = b"typeDefinition=\"Objects\"";
    let replacement = b"typeDefinition=\"Other__\"";
    let start = fixture
        .bw64_header_and_pcm
        .windows(marker.len())
        .position(|window| window == marker)
        .ok_or_else(|| "synthetic fixture pack type marker is missing".to_owned())?;
    fixture.bw64_header_and_pcm[start..start + replacement.len()].copy_from_slice(replacement);
    let marker = b"typeLabel=\"0003\"";
    let replacement = b"typeLabel=\"0004\"";
    let start = fixture
        .bw64_header_and_pcm
        .windows(marker.len())
        .position(|window| window == marker)
        .ok_or_else(|| "synthetic fixture pack type label is missing".to_owned())?;
    fixture.bw64_header_and_pcm[start..start + replacement.len()].copy_from_slice(replacement);
    Ok(fixture)
}

fn generate_fixture_with_layout(
    duration_seconds: u32,
    object_count: usize,
    direct_channels: &[FixtureDirectChannel],
) -> Result<SyntheticFixture, String> {
    if duration_seconds == 0 || duration_seconds > 30 {
        return Err("synthetic fixture duration must be 1..=30 seconds".to_owned());
    }
    if !(1..=64).contains(&object_count) {
        return Err("synthetic fixture object count must be 1..=64".to_owned());
    }
    let sample_count = usize::try_from(duration_seconds)
        .map_err(|_| "fixture duration overflow".to_owned())?
        .checked_mul(SAMPLE_RATE as usize)
        .ok_or_else(|| "fixture sample count overflow".to_owned())?;
    let channel_count = object_count
        .checked_add(direct_channels.len())
        .ok_or_else(|| "fixture channel count overflow".to_owned())?;
    let ground_truth = fixture_matrix(channel_count);
    let basis = (0..RANK)
        .map(|index| {
            let frequency = 220.0 + f64::from((index as u32) * 37);
            (0..sample_count)
                .map(|sample| {
                    let time = sample as f64 / f64::from(SAMPLE_RATE);
                    0.035 * (2.0 * PI * frequency * time).sin()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let pcm = ground_truth
        .iter()
        .map(|row| {
            (0..sample_count)
                .map(|sample| {
                    row.iter()
                        .zip(&basis)
                        .map(|(coefficient, track)| coefficient * track[sample])
                        .sum::<f64>()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let xml = fixture_axml(duration_seconds, object_count, direct_channels);
    let chna = fixture_chna(object_count, direct_channels);
    let pcm_bytes = encode_pcm_24(&pcm);
    let bw64 = build_bw64(
        xml.as_bytes(),
        &chna,
        &pcm_bytes,
        sample_count,
        channel_count,
    );
    Ok(SyntheticFixture {
        bw64_header_and_pcm: bw64,
        matrix: ground_truth
            .iter()
            .map(|row| row.iter().map(|value| value * QUANTUM).collect())
            .collect(),
    })
}

#[derive(Clone, Copy)]
struct FixtureDirectChannel {
    channel_id: &'static str,
    channel_name: &'static str,
    speaker_label: &'static str,
    position: AdmPosition,
}

fn read_u16_le(bytes: &[u8]) -> Result<u16, AdmError> {
    let value = bytes
        .try_into()
        .map_err(|_| AdmError::Truncated("little-endian u16"))?;
    Ok(u16::from_le_bytes(value))
}

fn read_u32_le(bytes: &[u8]) -> Result<u32, AdmError> {
    let value = bytes
        .try_into()
        .map_err(|_| AdmError::Truncated("little-endian u32"))?;
    Ok(u32::from_le_bytes(value))
}

fn read_u64_le(bytes: &[u8]) -> Result<u64, AdmError> {
    let value = bytes
        .try_into()
        .map_err(|_| AdmError::Truncated("little-endian u64"))?;
    Ok(u64::from_le_bytes(value))
}

struct ChnaBindings {
    bindings: HashMap<String, ChnaBinding>,
    track_count: usize,
    track_uid_count: usize,
}

struct ChnaBinding {
    track_index: usize,
    pack_ref: String,
}

fn parse_chna(bytes: &[u8]) -> Result<ChnaBindings, AdmError> {
    if bytes.len() < 4 {
        return Err(AdmError::Truncated("chna chunk"));
    }
    let tracks = usize::from(u16::from_le_bytes(bytes[0..2].try_into().unwrap()));
    let uids = usize::from(u16::from_le_bytes(bytes[2..4].try_into().unwrap()));
    let expected_bytes = tracks
        .checked_mul(40)
        .and_then(|size| size.checked_add(4))
        .ok_or(AdmError::InvalidContainer("chna entry count"))?;
    if tracks != uids || bytes.len() < expected_bytes {
        return Err(AdmError::InvalidContainer("chna entry count"));
    }
    let mut result = HashMap::with_capacity(tracks);
    let mut indices = HashMap::with_capacity(tracks);
    for index in 0..tracks {
        let start = 4 + index * 40;
        let uid = read_padded_string(&bytes[start + 2..start + 14]);
        let track_index = usize::from(read_u16_le(&bytes[start..start + 2])?);
        let track_ref = read_padded_string(&bytes[start + 14..start + 28]);
        let pack_ref = read_padded_string(&bytes[start + 28..start + 39]);
        if track_index == 0 || track_index > tracks {
            return Err(AdmError::InvalidMetadata(format!(
                "CHNA track index {track_index} is outside 1..={tracks}"
            )));
        }
        if uid.is_empty() || track_ref.is_empty() || pack_ref.is_empty() {
            return Err(AdmError::InvalidMetadata(format!(
                "CHNA row {index} has an empty binding field"
            )));
        }
        if result.contains_key(&uid) {
            return Err(AdmError::InvalidMetadata(format!(
                "duplicate CHNA TrackUID {uid}"
            )));
        }
        if let Some(previous_uid) = indices.insert(track_index, uid.clone()) {
            return Err(AdmError::InvalidMetadata(format!(
                "duplicate CHNA track index {track_index} for {previous_uid} and {uid}"
            )));
        }
        result.insert(
            uid,
            ChnaBinding {
                track_index,
                pack_ref,
            },
        );
    }
    Ok(ChnaBindings {
        bindings: result,
        track_count: tracks,
        track_uid_count: uids,
    })
}

fn parse_adm_objects(
    xml: &str,
    bindings: &HashMap<String, ChnaBinding>,
    channel_count: usize,
    sample_count: usize,
) -> Result<Vec<AdmObject>, AdmError> {
    let mut packs = HashMap::new();
    for block in tag_blocks(xml, "audioPackFormat") {
        let open = opening_tag(block, "audioPackFormat")?;
        let id = attribute(open, "audioPackFormatID")?;
        let channels = child_texts(block, "audioChannelFormatIDRef")?;
        if channels.is_empty() {
            return Err(AdmError::InvalidMetadata(format!(
                "pack {id} has no channel format references"
            )));
        }
        let pack_name =
            attribute_optional(open, "audioPackFormatName").unwrap_or_else(|| id.clone());
        let type_definition = attribute_optional(open, "typeDefinition");
        let type_label = attribute_optional(open, "typeLabel");
        packs.insert(
            id,
            (
                channels,
                adm_object_kind(type_definition.as_deref(), type_label.as_deref()),
                pack_name,
            ),
        );
    }
    let mut positions = HashMap::new();
    for block in tag_blocks(xml, "audioChannelFormat") {
        let open = opening_tag(block, "audioChannelFormat")?;
        let id = attribute(open, "audioChannelFormatID")?;
        let channel_name =
            attribute_optional(open, "audioChannelFormatName").unwrap_or_else(|| id.clone());
        let block_formats = tag_blocks(block, "audioBlockFormat");
        if block_formats.is_empty() {
            return Err(AdmError::InvalidMetadata(format!(
                "channel {id} has no block format"
            )));
        }
        if block_formats.len() != 1 {
            return Err(AdmError::InvalidMetadata(format!(
                "channel {id} has multiple audioBlockFormat elements; trajectories are unsupported"
            )));
        }
        let block_format = block_formats[0];
        let block_open = opening_tag(block_format, "audioBlockFormat")?;
        validate_static_time(
            attribute_optional(block_open, "rtime").as_deref(),
            0.0,
            "block timing",
            &id,
        )?;
        validate_static_time(
            attribute_optional(block_open, "duration").as_deref(),
            sample_count as f64 / f64::from(SAMPLE_RATE),
            "block timing",
            &id,
        )?;
        validate_static_jump_position(block_format, &id)?;
        let (speaker_label, low_pass, lfe_semantics) = channel_semantics(block, &id)?;
        positions.insert(
            id,
            ChannelMetadata {
                channel_name: channel_name.clone(),
                position: parse_position(block_format)?,
                speaker_label,
                low_pass,
                lfe_semantics,
                block_rtime: attribute_optional(block_open, "rtime"),
                block_duration: attribute_optional(block_open, "duration"),
                gain: attribute_optional(block_open, "gain").or_else(|| {
                    tag_blocks(block_format, "gain")
                        .into_iter()
                        .next()
                        .and_then(|block| inner_text(block, "gain").ok())
                        .map(ToOwned::to_owned)
                }),
            },
        );
    }
    let mut objects = Vec::new();
    let mut referenced_track_uids = BTreeSet::new();
    for (ordinal, block) in tag_blocks(xml, "audioObject").into_iter().enumerate() {
        let open = opening_tag(block, "audioObject")?;
        let object_id = attribute(open, "audioObjectID")?;
        let pack_id = child_text(block, "audioPackFormatIDRef")?;
        let track_uids = child_texts(block, "audioTrackUIDRef")?;
        let (channel_ids, kind, pack_name) = packs.get(&pack_id).ok_or_else(|| {
            AdmError::InvalidMetadata(format!(
                "object {object_id} references unknown pack {pack_id}"
            ))
        })?;
        if channel_ids.len() != track_uids.len() {
            return Err(AdmError::InvalidMetadata(format!(
                "object {object_id} has {} channel references but {} TrackUID references",
                channel_ids.len(),
                track_uids.len()
            )));
        }
        let object_start = attribute_optional(open, "start");
        let object_duration = attribute_optional(open, "duration");
        validate_static_time(object_start.as_deref(), 0.0, "object timing", &object_id)?;
        validate_static_time(
            object_duration.as_deref(),
            sample_count as f64 / f64::from(SAMPLE_RATE),
            "object timing",
            &object_id,
        )?;
        let mut channels = Vec::with_capacity(channel_ids.len());
        for (channel_id, track_uid) in channel_ids.iter().zip(track_uids) {
            if !referenced_track_uids.insert(track_uid.clone()) {
                return Err(AdmError::InvalidMetadata(format!(
                    "duplicate ADM TrackUID reference {track_uid}"
                )));
            }
            let channel = positions
                .get(channel_id)
                .ok_or_else(|| AdmError::MissingPosition(object_id.clone()))?;
            let binding = bindings
                .get(&track_uid)
                .ok_or_else(|| AdmError::MissingObjectBinding(object_id.clone()))?;
            let track_index = binding.track_index;
            if track_index == 0 || track_index > channel_count {
                return Err(AdmError::InvalidMetadata(format!(
                    "object {object_id} maps to PCM track {track_index}"
                )));
            }
            if binding.pack_ref != pack_id.as_str() {
                return Err(AdmError::InvalidMetadata(format!(
                    "object {object_id} CHNA pack relation conflicts with {}",
                    binding.pack_ref
                )));
            }
            let is_lfe = if *kind == AdmObjectKind::DirectSpeakers {
                channel.lfe_semantics.ok_or_else(|| {
                    AdmError::InvalidMetadata(format!(
                        "channel {channel_id} is missing LFE semantic metadata"
                    ))
                })?
            } else {
                false
            };
            channels.push(AdmChannel {
                channel_id: channel_id.clone(),
                channel_name: channel.channel_name.clone(),
                speaker_label: channel.speaker_label.clone(),
                low_pass: channel.low_pass,
                track_uid,
                track_index,
                position: channel.position,
                is_lfe,
                block_rtime: channel.block_rtime.clone(),
                block_duration: channel.block_duration.clone(),
                gain: channel.gain.clone(),
            });
        }
        let first_channel = channels.first().ok_or_else(|| {
            AdmError::InvalidMetadata(format!("object {object_id} has no channels"))
        })?;
        objects.push(AdmObject {
            ordinal,
            object_id,
            object_name: attribute_optional(open, "audioObjectName")
                .unwrap_or_else(|| "unnamed".to_owned()),
            start: object_start,
            duration: object_duration,
            pack_format_id: pack_id,
            pack_format_name: pack_name.clone(),
            track_uid: first_channel.track_uid.clone(),
            track_index: first_channel.track_index,
            position: first_channel.position,
            kind: *kind,
            channels,
        });
    }
    if objects.is_empty() {
        return Err(AdmError::InvalidMetadata(
            "no audioObject elements".to_owned(),
        ));
    }
    Ok(objects)
}

fn validate_static_jump_position(block: &str, channel_id: &str) -> Result<(), AdmError> {
    let jump_positions = tag_blocks(block, "jumpPosition");
    if jump_positions.len() > 1 {
        return Err(AdmError::InvalidMetadata(format!(
            "channel {channel_id} has multiple jumpPosition elements; trajectories are unsupported"
        )));
    }
    if let Some(jump_position) = jump_positions.first() {
        let open = opening_tag(jump_position, "jumpPosition")?;
        let value = inner_text(jump_position, "jumpPosition")?;
        let interpolation = attribute_optional(open, "interpolationLength");
        if value != "1" || interpolation.as_deref().is_some_and(|value| value != "0") {
            return Err(AdmError::InvalidMetadata(format!(
                "channel {channel_id} has unsupported jumpPosition trajectory"
            )));
        }
    }
    Ok(())
}

fn validate_static_time(
    value: Option<&str>,
    expected_seconds: f64,
    field: &str,
    identity: &str,
) -> Result<(), AdmError> {
    let Some(value) = value else {
        return Ok(());
    };
    let actual_seconds = parse_adm_time(value).map_err(|()| {
        AdmError::InvalidMetadata(format!(
            "{identity} {field} uses unsupported ADM time '{value}'"
        ))
    })?;
    let tolerance = ADM_TIME_EPSILON * expected_seconds.abs().max(1.0);
    if (actual_seconds - expected_seconds).abs() > tolerance {
        return Err(AdmError::InvalidMetadata(format!(
            "{identity} {field} is not supported by the full-track static subset"
        )));
    }
    Ok(())
}

fn parse_adm_time(value: &str) -> Result<f64, ()> {
    let (hours_minutes, seconds) = value.rsplit_once(':').ok_or(())?;
    let (hours, minutes) = hours_minutes.split_once(':').ok_or(())?;
    let hours = hours.parse::<u64>().map_err(|_| ())?;
    let minutes = minutes.parse::<u64>().map_err(|_| ())?;
    let seconds = seconds.parse::<f64>().map_err(|_| ())?;
    if minutes >= 60 || !(0.0..60.0).contains(&seconds) || !seconds.is_finite() {
        return Err(());
    }
    Ok(hours as f64 * 3600.0 + minutes as f64 * 60.0 + seconds)
}

#[derive(Clone, Debug)]
struct ChannelMetadata {
    channel_name: String,
    position: AdmPosition,
    speaker_label: Option<String>,
    low_pass: Option<f64>,
    lfe_semantics: Option<bool>,
    block_rtime: Option<String>,
    block_duration: Option<String>,
    gain: Option<String>,
}

fn adm_object_kind(type_definition: Option<&str>, type_label: Option<&str>) -> AdmObjectKind {
    if type_definition.is_some_and(|value| value.eq_ignore_ascii_case("objects"))
        || type_label == Some("0003")
    {
        return AdmObjectKind::Objects;
    }
    if type_definition.is_some_and(|value| value.eq_ignore_ascii_case("directspeakers"))
        || type_label == Some("0002")
    {
        return AdmObjectKind::DirectSpeakers;
    }
    AdmObjectKind::Other
}

fn parse_position(block: &str) -> Result<AdmPosition, AdmError> {
    let mut values = HashMap::new();
    for position in tag_blocks(block, "position") {
        let open = opening_tag(position, "position")?;
        let coordinate = attribute(open, "coordinate")?;
        let value = if let Some(value) = attribute_optional(open, "value") {
            value
        } else {
            inner_text(position, "position")?.to_owned()
        };
        let number = value
            .parse::<f64>()
            .map_err(|_| AdmError::InvalidNumber(value.clone()))?;
        values.insert(coordinate.to_ascii_lowercase(), number);
    }
    let position = AdmPosition {
        azimuth: *values
            .get("azimuth")
            .or_else(|| values.get("x"))
            .ok_or_else(|| AdmError::InvalidMetadata("position missing azimuth".to_owned()))?,
        elevation: *values
            .get("elevation")
            .or_else(|| values.get("y"))
            .ok_or_else(|| AdmError::InvalidMetadata("position missing elevation".to_owned()))?,
        distance: *values
            .get("distance")
            .or_else(|| values.get("z"))
            .unwrap_or(&1.0),
    };
    if !position.is_finite() {
        return Err(AdmError::InvalidMetadata(
            "position is not finite".to_owned(),
        ));
    }
    Ok(position)
}

type ChannelSemantics = (Option<String>, Option<f64>, Option<bool>);

fn channel_semantics(channel_block: &str, channel_id: &str) -> Result<ChannelSemantics, AdmError> {
    let speaker_labels = child_texts(channel_block, "speakerLabel")?;
    if speaker_labels.len() > 1 {
        return Err(AdmError::InvalidMetadata(format!(
            "channel {channel_id} has multiple speakerLabel elements"
        )));
    }
    let speaker_label = speaker_labels
        .into_iter()
        .next()
        .filter(|value| !value.trim().is_empty());
    let frequencies = tag_blocks(channel_block, "frequency");
    if frequencies.len() > 1 {
        return Err(AdmError::InvalidMetadata(format!(
            "channel {channel_id} has multiple frequency elements"
        )));
    }
    let low_pass = frequencies
        .first()
        .map(|frequency| {
            let low_passes = tag_blocks(frequency, "lowPass");
            if low_passes.len() > 1 {
                return Err(AdmError::InvalidMetadata(format!(
                    "channel {channel_id} has multiple lowPass elements"
                )));
            }
            let Some(low_pass) = low_passes.first() else {
                return Ok(None);
            };
            let raw = inner_text(low_pass, "lowPass")?;
            let value = raw
                .parse::<f64>()
                .map_err(|_| AdmError::InvalidNumber(raw.to_owned()))?;
            if !value.is_finite() || value <= 0.0 {
                return Err(AdmError::InvalidMetadata(format!(
                    "channel {channel_id} has an invalid lowPass frequency"
                )));
            }
            Ok(Some(value))
        })
        .transpose()?
        .flatten();
    let label_is_lfe = speaker_label.as_deref().is_some_and(is_lfe_speaker_label);
    if low_pass.is_some() && speaker_label.is_some() && !label_is_lfe {
        return Err(AdmError::InvalidMetadata(format!(
            "channel {channel_id} has conflicting LFE semantic metadata"
        )));
    }
    let semantics = if speaker_label.is_some() || low_pass.is_some() {
        Some(label_is_lfe || low_pass.is_some())
    } else {
        None
    };
    Ok((speaker_label, low_pass, semantics))
}

fn is_lfe_speaker_label(label: &str) -> bool {
    let canonical = label
        .trim()
        .rsplit(':')
        .next()
        .unwrap_or(label)
        .to_ascii_uppercase();
    matches!(
        canonical.as_str(),
        "LFE" | "LFE1" | "LFE2" | "LFEL" | "LFER" | "RC_LFE" | "RC_LFE2"
    )
}

fn tag_blocks<'a>(xml: &'a str, name: &str) -> Vec<&'a str> {
    let open_marker = format!("<{name}");
    let close_marker = format!("</{name}>");
    let mut result = Vec::new();
    let mut offset = 0;
    while let Some(relative) = xml[offset..].find(&open_marker) {
        let start = offset + relative;
        let boundary = xml
            .get(start + open_marker.len()..)
            .and_then(|suffix| suffix.chars().next());
        if !matches!(boundary, Some(' ' | '>' | '/')) {
            offset = start + open_marker.len();
            continue;
        }
        let Some(end_open) = xml[start..].find('>') else {
            break;
        };
        let end_open = start + end_open + 1;
        if xml[start..end_open].ends_with("/>") {
            result.push(&xml[start..end_open]);
            offset = end_open;
            continue;
        }
        let Some(relative_close) = xml[end_open..].find(&close_marker) else {
            break;
        };
        let end = end_open + relative_close + close_marker.len();
        result.push(&xml[start..end]);
        offset = end;
    }
    result
}

fn opening_tag<'a>(block: &'a str, name: &str) -> Result<&'a str, AdmError> {
    let marker = format!("<{name}");
    let start = block
        .find(&marker)
        .ok_or_else(|| AdmError::InvalidMetadata(format!("missing <{name}>")))?;
    let end = block[start..]
        .find('>')
        .map(|offset| start + offset + 1)
        .ok_or_else(|| AdmError::InvalidMetadata(format!("unterminated <{name}>")))?;
    Ok(&block[start..end])
}

fn attribute(tag: &str, name: &str) -> Result<String, AdmError> {
    attribute_optional(tag, name)
        .ok_or_else(|| AdmError::InvalidMetadata(format!("missing {name}")))
}

fn attribute_optional(tag: &str, name: &str) -> Option<String> {
    let marker = format!("{name}=\"");
    let start = tag.find(&marker)? + marker.len();
    let end = tag[start..].find('"')? + start;
    Some(tag[start..end].to_owned())
}

fn child_text(block: &str, name: &str) -> Result<String, AdmError> {
    inner_text(block, name).map(ToOwned::to_owned)
}

fn child_texts(block: &str, name: &str) -> Result<Vec<String>, AdmError> {
    tag_blocks(block, name)
        .into_iter()
        .map(|child| child_text(child, name))
        .collect()
}

fn inner_text<'a>(block: &'a str, name: &str) -> Result<&'a str, AdmError> {
    let open_marker = format!("<{name}");
    let start = block
        .find(&open_marker)
        .ok_or_else(|| AdmError::InvalidMetadata(format!("missing <{name}>")))?;
    let open_end = block[start..]
        .find('>')
        .map(|offset| start + offset + 1)
        .ok_or_else(|| AdmError::InvalidMetadata(format!("unterminated <{name}>")))?;
    let close_marker = format!("</{name}>");
    let close_start = block[open_end..]
        .find(&close_marker)
        .map(|offset| open_end + offset)
        .ok_or_else(|| AdmError::InvalidMetadata(format!("missing </{name}>")))?;
    Ok(block[open_end..close_start].trim())
}

fn read_padded_string(bytes: &[u8]) -> String {
    let length = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..length]).to_string()
}

fn decode_pcm(bytes: &[u8], channels: usize, bits: u16) -> Vec<Vec<f64>> {
    let bytes_per_sample = usize::from(bits / 8);
    let sample_count = bytes.len() / (channels * bytes_per_sample);
    let mut output = vec![Vec::with_capacity(sample_count); channels];
    for sample in 0..sample_count {
        for (channel, output_channel) in output.iter_mut().enumerate().take(channels) {
            let offset = (sample * channels + channel) * bytes_per_sample;
            let value = match bits {
                16 => f64::from(read_i16_le(bytes, offset)) / 32768.0,
                24 => {
                    let raw = i32::from(*bytes.get(offset).unwrap_or(&0))
                        | (i32::from(*bytes.get(offset + 1).unwrap_or(&0)) << 8)
                        | (i32::from(*bytes.get(offset + 2).unwrap_or(&0)) << 16);
                    let signed = if raw & 0x80_0000 != 0 {
                        raw | !0xFF_FFFF
                    } else {
                        raw
                    };
                    f64::from(signed) / 8_388_608.0
                }
                32 => f64::from(read_i32_le(bytes, offset)) / 2_147_483_648.0,
                _ => 0.0,
            };
            output_channel.push(value);
        }
    }
    output
}

fn read_i16_le(bytes: &[u8], offset: usize) -> i16 {
    let value = [
        *bytes.get(offset).unwrap_or(&0),
        *bytes.get(offset + 1).unwrap_or(&0),
    ];
    i16::from_le_bytes(value)
}

fn read_i32_le(bytes: &[u8], offset: usize) -> i32 {
    let value = [
        *bytes.get(offset).unwrap_or(&0),
        *bytes.get(offset + 1).unwrap_or(&0),
        *bytes.get(offset + 2).unwrap_or(&0),
        *bytes.get(offset + 3).unwrap_or(&0),
    ];
    i32::from_le_bytes(value)
}

fn fixture_matrix(object_count: usize) -> Vec<Vec<f64>> {
    (0..object_count)
        .map(|object| {
            (0..RANK)
                .map(|basis| {
                    if object < RANK {
                        f64::from(u8::from(object == basis))
                    } else {
                        f64::from(u8::from((object + basis * 2) % 5 == 0))
                    }
                })
                .collect()
        })
        .collect()
}

fn fixture_axml(
    duration_seconds: u32,
    object_count: usize,
    direct_channels: &[FixtureDirectChannel],
) -> String {
    let duration = format!("00:00:{duration_seconds:02}.00000");
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><ebuCoreMain xmlns=\"urn:ebu:metadata-schema:ebuCore_2016\"><coreMetadata><format><audioFormatExtended>",
    );
    let _ = write!(
        xml,
        "<audioProgramme audioProgrammeID=\"APR_1001\" audioProgrammeName=\"JOCForge Synthetic Programme\" start=\"00:00:00.00000\" end=\"{duration}\"><audioContentIDRef>ACO_1001</audioContentIDRef></audioProgramme><audioContent audioContentID=\"ACO_1001\" audioContentName=\"JOCForge Synthetic Content\">"
    );
    if !direct_channels.is_empty() {
        xml.push_str("<audioObjectIDRef>AO_1001</audioObjectIDRef>");
    }
    for object in 0..object_count {
        let _ = write!(
            xml,
            "<audioObjectIDRef>AO_{:04X}</audioObjectIDRef>",
            0x100B + object
        );
    }
    xml.push_str("<dialogue mixedContentKind=\"0\">2</dialogue></audioContent>");
    if !direct_channels.is_empty() {
        xml.push_str("<audioPackFormat audioPackFormatID=\"AP_00011001\" audioPackFormatName=\"Synthetic Bed\" typeLabel=\"0001\" typeDefinition=\"DirectSpeakers\">");
        for channel in direct_channels {
            let _ = write!(
                xml,
                "<audioChannelFormatIDRef>{}</audioChannelFormatIDRef>",
                channel.channel_id
            );
        }
        xml.push_str("</audioPackFormat>");
        for channel in direct_channels {
            let _ = write!(
                xml,
                "<audioChannelFormat audioChannelFormatID=\"{}\" audioChannelFormatName=\"{}\" typeLabel=\"0001\" typeDefinition=\"DirectSpeakers\"><audioBlockFormat audioBlockFormatID=\"AB_{}\"><cartesian>1</cartesian><speakerLabel>{}</speakerLabel><position coordinate=\"X\">{:.6}</position><position coordinate=\"Y\">{:.6}</position><position coordinate=\"Z\">{:.6}</position></audioBlockFormat></audioChannelFormat>",
                channel.channel_id,
                channel.channel_name,
                channel.channel_id,
                channel.speaker_label,
                channel.position.azimuth,
                channel.position.elevation,
                channel.position.distance,
            );
        }
        let mut track_uids = String::new();
        for index in 0..direct_channels.len() {
            let _ = write!(
                track_uids,
                "<audioTrackUIDRef>ATU_{:08X}</audioTrackUIDRef>",
                index + 1
            );
        }
        let _ = write!(
            xml,
            "<audioObject audioObjectID=\"AO_1001\" audioObjectName=\"Synthetic Bed\" start=\"00:00:00.00000\" duration=\"{duration}\"><audioPackFormatIDRef>AP_00011001</audioPackFormatIDRef>{track_uids}</audioObject>"
        );
    }
    for object in 0..object_count {
        let id = 0x0003_1001 + object;
        let x = -0.8 + f64::from((object as u32) % 8) * 0.2;
        let y = 0.2 + f64::from((object as u32) / 8) * 0.2;
        let z = if object % 3 == 0 { 0.3 } else { 0.0 };
        let _ = write!(
            xml,
            "<audioPackFormat audioPackFormatID=\"AP_{id:08X}\" audioPackFormatName=\"Object {object:02}\" typeLabel=\"0003\" typeDefinition=\"Objects\"><audioChannelFormatIDRef>AC_{id:08X}</audioChannelFormatIDRef></audioPackFormat><audioChannelFormat audioChannelFormatID=\"AC_{id:08X}\" audioChannelFormatName=\"Object {object:02}\" typeLabel=\"0003\" typeDefinition=\"Objects\"><audioBlockFormat audioBlockFormatID=\"AB_{id:08X}_00000001\" rtime=\"00:00:00.00000\" duration=\"{duration}\"><cartesian>1</cartesian><position coordinate=\"X\">{x:.6}</position><position coordinate=\"Y\">{y:.6}</position><position coordinate=\"Z\">{z:.6}</position><jumpPosition interpolationLength=\"0\">1</jumpPosition></audioBlockFormat></audioChannelFormat>"
        );
    }
    for object in 0..object_count {
        let format_id = 0x0003_1001 + object;
        let object_id = 0x100B + object;
        let _ = write!(
            xml,
            "<audioObject audioObjectID=\"AO_{object_id:04X}\" audioObjectName=\"Object {object:02}\" start=\"00:00:00.00000\" duration=\"{duration}\"><audioPackFormatIDRef>AP_{format_id:08X}</audioPackFormatIDRef><audioTrackUIDRef>ATU_{:08X}</audioTrackUIDRef></audioObject>",
            object + direct_channels.len() + 1
        );
    }
    for (index, channel) in direct_channels.iter().enumerate() {
        let stream_id = channel.channel_id.trim_start_matches("AC_");
        let track_uid = index + 1;
        let _ = write!(
            xml,
            "<audioStreamFormat audioStreamFormatID=\"AS_{stream_id}\" audioStreamFormatName=\"PCM_{stream_id}\" formatLabel=\"0001\" formatDefinition=\"PCM\"><audioChannelFormatIDRef>{}</audioChannelFormatIDRef><audioPackFormatIDRef>AP_00011001</audioPackFormatIDRef><audioTrackFormatIDRef>AT_{stream_id}_01</audioTrackFormatIDRef></audioStreamFormat><audioTrackFormat audioTrackFormatID=\"AT_{stream_id}_01\" audioTrackFormatName=\"PCM_{stream_id}\" formatLabel=\"0001\" formatDefinition=\"PCM\"><audioStreamFormatIDRef>AS_{stream_id}</audioStreamFormatIDRef></audioTrackFormat><audioTrackUID UID=\"ATU_{track_uid:08X}\" sampleRate=\"48000\" bitDepth=\"24\"><audioTrackFormatIDRef>AT_{stream_id}_01</audioTrackFormatIDRef><audioPackFormatIDRef>AP_00011001</audioPackFormatIDRef></audioTrackUID>",
            channel.channel_id,
        );
    }
    for object in 0..object_count {
        let format_id = 0x0003_1001 + object;
        let track_uid = object + direct_channels.len() + 1;
        let _ = write!(
            xml,
            "<audioStreamFormat audioStreamFormatID=\"AS_{format_id:08X}\" audioStreamFormatName=\"PCM_Object {object:02}\" formatLabel=\"0001\" formatDefinition=\"PCM\"><audioChannelFormatIDRef>AC_{format_id:08X}</audioChannelFormatIDRef><audioPackFormatIDRef>AP_{format_id:08X}</audioPackFormatIDRef><audioTrackFormatIDRef>AT_{format_id:08X}_01</audioTrackFormatIDRef></audioStreamFormat><audioTrackFormat audioTrackFormatID=\"AT_{format_id:08X}_01\" audioTrackFormatName=\"PCM_Object {object:02}\" formatLabel=\"0001\" formatDefinition=\"PCM\"><audioStreamFormatIDRef>AS_{format_id:08X}</audioStreamFormatIDRef></audioTrackFormat><audioTrackUID UID=\"ATU_{track_uid:08X}\" sampleRate=\"48000\" bitDepth=\"24\"><audioTrackFormatIDRef>AT_{format_id:08X}_01</audioTrackFormatIDRef><audioPackFormatIDRef>AP_{format_id:08X}</audioPackFormatIDRef></audioTrackUID>"
        );
    }
    xml.push_str("</audioFormatExtended></format></coreMetadata></ebuCoreMain>");
    xml
}

fn fixture_chna(object_count: usize, direct_channels: &[FixtureDirectChannel]) -> Vec<u8> {
    let channel_count = object_count + direct_channels.len();
    let mut bytes = Vec::with_capacity(4 + channel_count * 40);
    bytes.extend_from_slice(&(channel_count as u16).to_le_bytes());
    bytes.extend_from_slice(&(channel_count as u16).to_le_bytes());
    for (index, channel) in direct_channels.iter().enumerate() {
        let track_index = index + 1;
        let stream_id = channel.channel_id.trim_start_matches("AC_");
        bytes.extend_from_slice(&(track_index as u16).to_le_bytes());
        append_padded_ascii(&mut bytes, &format!("ATU_{track_index:08X}"), 12);
        append_padded_ascii(&mut bytes, &format!("AT_{stream_id}_01"), 14);
        append_padded_ascii(&mut bytes, "AP_00011001", 11);
        bytes.push(0);
    }
    for object in 0..object_count {
        let id = object + direct_channels.len() + 1;
        bytes.extend_from_slice(&(id as u16).to_le_bytes());
        append_padded_ascii(&mut bytes, &format!("ATU_{id:08X}"), 12);
        append_padded_ascii(
            &mut bytes,
            &format!("AT_{:08X}_01", 0x0003_1001 + object),
            14,
        );
        append_padded_ascii(&mut bytes, &format!("AP_{:08X}", 0x0003_1001 + object), 11);
        bytes.push(0);
    }
    bytes
}

fn append_padded_ascii(output: &mut Vec<u8>, value: &str, width: usize) {
    let mut bytes = vec![0_u8; width];
    let value_bytes = value.as_bytes();
    let copy = value_bytes.len().min(width);
    bytes[..copy].copy_from_slice(&value_bytes[..copy]);
    output.extend_from_slice(&bytes);
}

fn encode_pcm_24(pcm: &[Vec<f64>]) -> Vec<u8> {
    let sample_count = pcm.first().map_or(0, Vec::len);
    let mut bytes = Vec::with_capacity(sample_count * pcm.len() * 3);
    for sample in 0..sample_count {
        for channel in pcm {
            let value = (channel[sample] * 8_388_607.0)
                .round()
                .clamp(-8_388_608.0, 8_388_607.0) as i32;
            let encoded = value.to_le_bytes();
            bytes.extend_from_slice(&encoded[..3]);
        }
    }
    bytes
}

fn append_chunk(output: &mut Vec<u8>, id: [u8; 4], body: &[u8]) {
    output.extend_from_slice(&id);
    output.extend_from_slice(&(u32::try_from(body.len()).unwrap_or(u32::MAX)).to_le_bytes());
    output.extend_from_slice(body);
    if body.len() % 2 != 0 {
        output.push(0);
    }
}

fn build_bw64(
    xml: &[u8],
    chna: &[u8],
    pcm: &[u8],
    samples: usize,
    channel_count: usize,
) -> Vec<u8> {
    let mut output = Vec::new();
    output.extend_from_slice(b"BW64\xff\xff\xff\xffWAVE");
    let mut ds64 = Vec::with_capacity(28);
    ds64.extend_from_slice(&0_u64.to_le_bytes());
    ds64.extend_from_slice(&(pcm.len() as u64).to_le_bytes());
    ds64.extend_from_slice(&(samples as u64).to_le_bytes());
    ds64.extend_from_slice(&0_u32.to_le_bytes());
    append_chunk(&mut output, *b"ds64", &ds64);
    let channels = channel_count as u16;
    let block_align = channels * 3;
    let byte_rate = SAMPLE_RATE * u32::from(block_align);
    let mut fmt = Vec::with_capacity(16);
    fmt.extend_from_slice(&1_u16.to_le_bytes());
    fmt.extend_from_slice(&channels.to_le_bytes());
    fmt.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    fmt.extend_from_slice(&byte_rate.to_le_bytes());
    fmt.extend_from_slice(&block_align.to_le_bytes());
    fmt.extend_from_slice(&24_u16.to_le_bytes());
    append_chunk(&mut output, *b"fmt ", &fmt);
    output.extend_from_slice(b"data\xff\xff\xff\xff");
    output.extend_from_slice(pcm);
    append_chunk(&mut output, *b"axml", xml);
    append_chunk(&mut output, *b"chna", chna);
    append_chunk(&mut output, *b"dbmd", &[0, 0, 0, 1, 0, 0]);
    let riff_size = output.len() as u64 - 8;
    output[20..28].copy_from_slice(&riff_size.to_le_bytes());
    output
}
