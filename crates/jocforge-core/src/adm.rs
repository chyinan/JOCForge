// pattern: Functional Core

use std::{collections::HashMap, f64::consts::PI, fmt, fmt::Write as _};

const CHANNEL_COUNT: usize = 15;
const SAMPLE_RATE: u32 = 48_000;
const RANK: usize = 7;
const QUANTUM: f64 = 820.0 / 4096.0;

/// One supported dynamic ADM object after BW64/chna resolution.
#[derive(Clone, Debug, PartialEq)]
pub struct AdmObject {
    pub ordinal: usize,
    pub object_id: String,
    pub track_uid: String,
    pub track_index: usize,
    pub position: AdmPosition,
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
    pub objects: Vec<AdmObject>,
    pub pcm: Vec<Vec<f64>>,
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
    let sample_rate = read_u32_le(&fmt[4..8])?;
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
    let sample_count = data_range.len() / block_align;
    let pcm = decode_pcm(&bytes[data_range.clone()], channel_count, bits);
    let bindings = parse_chna(&chna)?;
    let objects = parse_adm_objects(
        std::str::from_utf8(&axml)
            .map_err(|_| AdmError::InvalidMetadata("axml is not UTF-8".to_owned()))?,
        &bindings,
        channel_count,
    )?;
    if sample_rate != SAMPLE_RATE {
        return Err(AdmError::InvalidMetadata(format!(
            "MVP requires 48000 Hz, got {sample_rate}"
        )));
    }
    Ok(ParsedAdm {
        sample_rate,
        channel_count,
        sample_count,
        objects,
        pcm,
    })
}

/// Generates a four-second, 15-object, rank-7 synthetic ADM/BW64 fixture.
pub fn generate_synthetic_fixture(duration_seconds: u32) -> Result<SyntheticFixture, String> {
    if duration_seconds == 0 || duration_seconds > 30 {
        return Err("synthetic fixture duration must be 1..=30 seconds".to_owned());
    }
    let sample_count = usize::try_from(duration_seconds)
        .map_err(|_| "fixture duration overflow".to_owned())?
        .checked_mul(SAMPLE_RATE as usize)
        .ok_or_else(|| "fixture sample count overflow".to_owned())?;
    let ground_truth = fixture_matrix();
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
    let xml = fixture_axml(duration_seconds);
    let chna = fixture_chna();
    let pcm_bytes = encode_pcm_24(&pcm);
    let bw64 = build_bw64(xml.as_bytes(), &chna, &pcm_bytes, sample_count);
    Ok(SyntheticFixture {
        bw64_header_and_pcm: bw64,
        matrix: ground_truth
            .iter()
            .map(|row| row.iter().map(|value| value * QUANTUM).collect())
            .collect(),
    })
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

fn parse_chna(bytes: &[u8]) -> Result<HashMap<String, usize>, AdmError> {
    if bytes.len() < 4 {
        return Err(AdmError::Truncated("chna chunk"));
    }
    let tracks = usize::from(u16::from_le_bytes(bytes[0..2].try_into().unwrap()));
    let uids = usize::from(u16::from_le_bytes(bytes[2..4].try_into().unwrap()));
    if tracks != uids || bytes.len() < 4 + tracks * 40 {
        return Err(AdmError::InvalidContainer("chna entry count"));
    }
    let mut result = HashMap::with_capacity(tracks);
    for index in 0..tracks {
        let start = 4 + index * 40;
        let uid = read_padded_string(&bytes[start + 2..start + 14]);
        let track_index = usize::from(read_u16_le(&bytes[start..start + 2])?);
        result.insert(uid, track_index);
    }
    Ok(result)
}

fn parse_adm_objects(
    xml: &str,
    bindings: &HashMap<String, usize>,
    channel_count: usize,
) -> Result<Vec<AdmObject>, AdmError> {
    let mut packs = HashMap::new();
    for block in tag_blocks(xml, "audioPackFormat") {
        let open = opening_tag(block, "audioPackFormat")?;
        let id = attribute(open, "audioPackFormatID")?;
        let channel = child_text(block, "audioChannelFormatIDRef")?;
        packs.insert(id, channel);
    }
    let mut positions = HashMap::new();
    for block in tag_blocks(xml, "audioChannelFormat") {
        let open = opening_tag(block, "audioChannelFormat")?;
        let id = attribute(open, "audioChannelFormatID")?;
        let block_format = tag_blocks(block, "audioBlockFormat")
            .into_iter()
            .next()
            .ok_or_else(|| {
                AdmError::InvalidMetadata(format!("channel {id} has no block format"))
            })?;
        positions.insert(id, parse_position(block_format)?);
    }
    let mut objects = Vec::new();
    for (ordinal, block) in tag_blocks(xml, "audioObject").into_iter().enumerate() {
        let open = opening_tag(block, "audioObject")?;
        let object_id = attribute(open, "audioObjectID")?;
        let track_uid = child_text(block, "audioTrackUIDRef")?;
        let pack_id = child_text(block, "audioPackFormatIDRef")?;
        let channel_id = packs.get(&pack_id).ok_or_else(|| {
            AdmError::InvalidMetadata(format!(
                "object {object_id} references unknown pack {pack_id}"
            ))
        })?;
        let position = positions
            .get(channel_id)
            .copied()
            .ok_or_else(|| AdmError::MissingPosition(object_id.clone()))?;
        let track_index = *bindings
            .get(&track_uid)
            .ok_or_else(|| AdmError::MissingObjectBinding(object_id.clone()))?;
        if track_index == 0 || track_index > channel_count {
            return Err(AdmError::InvalidMetadata(format!(
                "object {object_id} maps to PCM track {track_index}"
            )));
        }
        objects.push(AdmObject {
            ordinal,
            object_id,
            track_uid,
            track_index,
            position,
        });
    }
    if objects.is_empty() {
        return Err(AdmError::InvalidMetadata(
            "no audioObject elements".to_owned(),
        ));
    }
    Ok(objects)
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

fn fixture_matrix() -> Vec<Vec<f64>> {
    (0..CHANNEL_COUNT)
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

fn fixture_axml(duration_seconds: u32) -> String {
    let duration = format!("00:00:{duration_seconds:02}.00000");
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><ebuCoreMain xmlns=\"urn:ebu:metadata-schema:ebuCore_2016\"><coreMetadata><format><audioFormatExtended>",
    );
    let _ = write!(
        xml,
        "<audioProgramme audioProgrammeID=\"APR_1001\" audioProgrammeName=\"JOCForge Synthetic Programme\" start=\"00:00:00.00000\" end=\"{duration}\"><audioContentIDRef>ACO_1001</audioContentIDRef></audioProgramme><audioContent audioContentID=\"ACO_1001\" audioContentName=\"JOCForge Synthetic Content\">"
    );
    for object in 0..CHANNEL_COUNT {
        let _ = write!(
            xml,
            "<audioObjectIDRef>AO_{:04X}</audioObjectIDRef>",
            0x100B + object
        );
    }
    xml.push_str("<dialogue mixedContentKind=\"0\">2</dialogue></audioContent>");
    for object in 0..CHANNEL_COUNT {
        let id = 0x0003_1001 + object;
        let x = -0.8 + f64::from((object as u32) % 8) * 0.2;
        let y = 0.2 + f64::from((object as u32) / 8) * 0.2;
        let z = if object % 3 == 0 { 0.3 } else { 0.0 };
        let _ = write!(
            xml,
            "<audioPackFormat audioPackFormatID=\"AP_{id:08X}\" audioPackFormatName=\"Object {object:02}\" typeLabel=\"0003\" typeDefinition=\"Objects\"><audioChannelFormatIDRef>AC_{id:08X}</audioChannelFormatIDRef></audioPackFormat><audioChannelFormat audioChannelFormatID=\"AC_{id:08X}\" audioChannelFormatName=\"Object {object:02}\" typeLabel=\"0003\" typeDefinition=\"Objects\"><audioBlockFormat audioBlockFormatID=\"AB_{id:08X}_00000001\" rtime=\"00:00:00.00000\" duration=\"{duration}\"><cartesian>1</cartesian><position coordinate=\"X\">{x:.6}</position><position coordinate=\"Y\">{y:.6}</position><position coordinate=\"Z\">{z:.6}</position><jumpPosition interpolationLength=\"0\">1</jumpPosition></audioBlockFormat></audioChannelFormat>"
        );
    }
    for object in 0..CHANNEL_COUNT {
        let format_id = 0x0003_1001 + object;
        let object_id = 0x100B + object;
        let _ = write!(
            xml,
            "<audioObject audioObjectID=\"AO_{object_id:04X}\" audioObjectName=\"Object {object:02}\" start=\"00:00:00.00000\" duration=\"{duration}\"><audioPackFormatIDRef>AP_{format_id:08X}</audioPackFormatIDRef><audioTrackUIDRef>ATU_{:08X}</audioTrackUIDRef></audioObject>",
            object + 1
        );
    }
    for object in 0..CHANNEL_COUNT {
        let format_id = 0x0003_1001 + object;
        let _ = write!(
            xml,
            "<audioStreamFormat audioStreamFormatID=\"AS_{format_id:08X}\" audioStreamFormatName=\"PCM_Object {object:02}\" formatLabel=\"0001\" formatDefinition=\"PCM\"><audioChannelFormatIDRef>AC_{format_id:08X}</audioChannelFormatIDRef><audioPackFormatIDRef>AP_{format_id:08X}</audioPackFormatIDRef><audioTrackFormatIDRef>AT_{format_id:08X}_01</audioTrackFormatIDRef></audioStreamFormat><audioTrackFormat audioTrackFormatID=\"AT_{format_id:08X}_01\" audioTrackFormatName=\"PCM_Object {object:02}\" formatLabel=\"0001\" formatDefinition=\"PCM\"><audioStreamFormatIDRef>AS_{format_id:08X}</audioStreamFormatIDRef></audioTrackFormat><audioTrackUID UID=\"ATU_{:08X}\" sampleRate=\"48000\" bitDepth=\"24\"><audioTrackFormatIDRef>AT_{format_id:08X}_01</audioTrackFormatIDRef><audioPackFormatIDRef>AP_{format_id:08X}</audioPackFormatIDRef></audioTrackUID>",
            object + 1
        );
    }
    xml.push_str("</audioFormatExtended></format></coreMetadata></ebuCoreMain>");
    xml
}

fn fixture_chna() -> Vec<u8> {
    let mut bytes = Vec::with_capacity(4 + CHANNEL_COUNT * 40);
    bytes.extend_from_slice(&(CHANNEL_COUNT as u16).to_le_bytes());
    bytes.extend_from_slice(&(CHANNEL_COUNT as u16).to_le_bytes());
    for object in 0..CHANNEL_COUNT {
        let id = object + 1;
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
    let mut bytes = Vec::with_capacity(sample_count * CHANNEL_COUNT * 3);
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

fn build_bw64(xml: &[u8], chna: &[u8], pcm: &[u8], samples: usize) -> Vec<u8> {
    let mut output = Vec::new();
    output.extend_from_slice(b"BW64\xff\xff\xff\xffWAVE");
    let mut ds64 = Vec::with_capacity(28);
    ds64.extend_from_slice(&0_u64.to_le_bytes());
    ds64.extend_from_slice(&(pcm.len() as u64).to_le_bytes());
    ds64.extend_from_slice(&(samples as u64).to_le_bytes());
    ds64.extend_from_slice(&0_u32.to_le_bytes());
    append_chunk(&mut output, *b"ds64", &ds64);
    let channels = CHANNEL_COUNT as u16;
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
