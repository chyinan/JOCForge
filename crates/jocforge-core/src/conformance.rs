// pattern: Functional Core

use crate::{JocProfile, profile_spec};
use std::fmt;

/// Public E-AC-3 `numblkscod` partitions for one syncframe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockPartition {
    One,
    Two,
    Three,
    Six,
}

impl BlockPartition {
    #[must_use]
    pub const fn audio_blocks(self) -> usize {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Six => 6,
        }
    }

    #[must_use]
    pub const fn numblkscod(self) -> u8 {
        match self {
            Self::One => 0,
            Self::Two => 1,
            Self::Three => 2,
            Self::Six => 3,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShortBlockPlan {
    partitions: Vec<BlockPartition>,
    convsync: Vec<bool>,
}

impl ShortBlockPlan {
    #[must_use]
    pub fn partitions(&self) -> &[BlockPartition] {
        &self.partitions
    }

    #[must_use]
    pub fn audio_blocks(&self) -> usize {
        self.partitions
            .iter()
            .map(|partition| partition.audio_blocks())
            .sum()
    }

    #[must_use]
    pub fn samples(&self) -> usize {
        self.audio_blocks() * 256
    }

    #[must_use]
    pub fn numblkscod(&self) -> Vec<u8> {
        self.partitions
            .iter()
            .map(|partition| partition.numblkscod())
            .collect()
    }

    #[must_use]
    pub fn convsync(&self) -> &[bool] {
        &self.convsync
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependentTopology {
    ids: Vec<u8>,
}

impl DependentTopology {
    #[must_use]
    pub fn ids(&self) -> &[u8] {
        &self.ids
    }
}

/// Machine-readable evidence returned by the independent raw-carriage
/// validator. The validator deliberately parses only public E-AC-3/JOC
/// structure; audio reconstruction remains the responsibility of OpenJOC.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiDependentValidationReport {
    pub component_count: usize,
    pub dependent_ids: Vec<u8>,
    pub dependent_channel_maps: Vec<u16>,
    pub joc_owner: Option<u8>,
    pub joc_downmix_index: u8,
    pub complexity_index: u8,
    pub payload_ids: Vec<u8>,
    pub sample_rate: u32,
    pub numblkscod: u8,
    pub total_samples: usize,
    pub lfe_present: bool,
    pub unique_dependent_channel_ownership: bool,
    pub ownership_census: Vec<ChannelOwnershipCensus>,
}

/// Machine-readable channel ownership for one serialized component.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelOwnershipCensus {
    /// `0` is independent; `1` is dependent.
    pub stream_type: u8,
    pub substream_id: u8,
    /// Stable Table E.1.4 token order. Base tokens are 0..4, rear pair is
    /// 5..6, and height pair is 7..8.
    pub channel_tokens: Vec<u8>,
}

/// External FFmpeg evidence is kept separate from normative JOC support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FfmpegInteroperabilityStatus {
    Pass,
    Partial,
    UnsupportedDecoderFeature,
    FailStreamError,
}

impl fmt::Display for FfmpegInteroperabilityStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Pass => "PASS",
            Self::Partial => "PARTIAL",
            Self::UnsupportedDecoderFeature => "UNSUPPORTED_DECODER_FEATURE",
            Self::FailStreamError => "FAIL_STREAM_ERROR",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConformancePlanError {
    EmptyBlockPlan,
    TotalBlocks { actual: usize },
    DependentCount { actual: usize },
}

impl fmt::Display for ConformancePlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBlockPlan => formatter.write_str("short-block plan is empty"),
            Self::TotalBlocks { actual } => {
                write!(
                    formatter,
                    "short-block plan totals {actual} audio blocks; expected 6"
                )
            }
            Self::DependentCount { actual } => {
                write!(formatter, "dependent count {actual} is outside D0..D7")
            }
        }
    }
}

impl std::error::Error for ConformancePlanError {}

/// Resolves one public General E-AC-3 programme interval.
pub fn resolve_short_block_plan(
    partitions: &[BlockPartition],
) -> Result<ShortBlockPlan, ConformancePlanError> {
    if partitions.is_empty() {
        return Err(ConformancePlanError::EmptyBlockPlan);
    }
    let audio_blocks = partitions
        .iter()
        .map(|partition| partition.audio_blocks())
        .sum::<usize>();
    if audio_blocks != 6 {
        return Err(ConformancePlanError::TotalBlocks {
            actual: audio_blocks,
        });
    }
    Ok(ShortBlockPlan {
        partitions: partitions.to_vec(),
        convsync: std::iter::once(true)
            .chain(std::iter::repeat_n(
                false,
                partitions.len().saturating_sub(1),
            ))
            .collect(),
    })
}

/// Resolves sequential dependent IDs permitted by the public three-bit field.
pub fn resolve_dependent_topology(
    dependent_count: usize,
) -> Result<DependentTopology, ConformancePlanError> {
    if !(1..=8).contains(&dependent_count) {
        return Err(ConformancePlanError::DependentCount {
            actual: dependent_count,
        });
    }
    Ok(DependentTopology {
        ids: (0..dependent_count)
            .map(|id| u8::try_from(id).unwrap_or(7))
            .collect(),
    })
}

/// Independently validates one project-owned raw E-AC-3/JOC access unit.
///
/// This is intentionally not an OpenJOC pass-through. It checks the public
/// syncframe header, dependent ordering, timing, channel-map cardinality and
/// ownership, final-dependent JOC/EMDF placement, payload IDs, complexity
/// signalling, and the JOC profile index carried inside payload 14.
pub fn validate_multi_dependent_access_unit(
    stream: &[u8],
    profile: JocProfile,
) -> Result<MultiDependentValidationReport, String> {
    let mut frames = Vec::new();
    let mut offset = 0_usize;
    while offset < stream.len() {
        let frame = parse_structural_frame(stream, offset)?;
        offset = offset
            .checked_add(frame.frame_size)
            .ok_or_else(|| "multi-dependent frame offset overflow".to_owned())?;
        frames.push(frame);
    }
    if offset != stream.len() || frames.len() < 2 {
        return Err(
            "multi-dependent access unit requires I0 and at least one dependent".to_owned(),
        );
    }

    let independent = &frames[0];
    if independent.stream_type != 0 || independent.substream_id != 0 {
        return Err(
            "multi-dependent access unit must start with independent substream I0".to_owned(),
        );
    }
    if independent.audio_coding_mode != 7 || independent.channel_map.is_some() {
        return Err(
            "multi-dependent I0 must use the canonical five-channel acmod 7 shape".to_owned(),
        );
    }
    if independent.addbsi.is_some() || independent.emdf.is_some() {
        return Err("JOC metadata must be owned by the final dependent".to_owned());
    }

    let dependent_count = frames.len() - 1;
    if dependent_count > 8 {
        return Err("multi-dependent access unit exceeds public D0..D7 bound".to_owned());
    }
    let mut dependent_ids = Vec::with_capacity(dependent_count);
    let mut dependent_channel_maps = Vec::with_capacity(dependent_count);
    let mut owned = Vec::new();
    let mut assembled = vec![0_u8, 1, 2, 3, 4];
    let mut ownership_census = vec![ChannelOwnershipCensus {
        stream_type: independent.stream_type,
        substream_id: independent.substream_id,
        channel_tokens: vec![0, 1, 2, 3, 4],
    }];
    for (index, frame) in frames.iter().skip(1).enumerate() {
        let expected_id = u8::try_from(index).map_err(|_| "dependent ID overflow".to_owned())?;
        if frame.stream_type != 1 || frame.substream_id != expected_id {
            return Err(format!(
                "dependent substreams must be sequential: expected D{expected_id}, got type {} id D{}",
                frame.stream_type, frame.substream_id
            ));
        }
        if frame.sample_rate != independent.sample_rate
            || frame.numblkscod != independent.numblkscod
            || frame.audio_blocks != independent.audio_blocks
        {
            return Err(format!(
                "dependent D{} does not match I0 sample rate/block timing",
                frame.substream_id
            ));
        }
        if index + 1 != dependent_count && frame.emdf.is_some() {
            return Err(format!(
                "dependent D{} unexpectedly carries JOC EMDF",
                frame.substream_id
            ));
        }
        if index + 1 != dependent_count && frame.addbsi.is_some() {
            return Err(format!(
                "dependent D{} unexpectedly carries addbsi",
                frame.substream_id
            ));
        }
        let channel_map = frame.channel_map.ok_or_else(|| {
            format!(
                "dependent D{} requires a custom chanmap",
                frame.substream_id
            )
        })?;
        let tokens = channel_map_tokens(channel_map)?;
        if tokens.len() != frame.full_bandwidth_channels {
            return Err(format!(
                "dependent D{} chanmap carries {} channels but acmod declares {}",
                frame.substream_id,
                tokens.len(),
                frame.full_bandwidth_channels
            ));
        }
        if tokens.iter().any(|token| *token >= 100) {
            return Err(
                "canonical JOC multi-dependent topology does not permit a dependent LFE".to_owned(),
            );
        }
        if tokens.iter().any(|token| owned.contains(token)) {
            return Err(format!(
                "dependent D{} duplicates physical channel ownership",
                frame.substream_id
            ));
        }
        ownership_census.push(ChannelOwnershipCensus {
            stream_type: frame.stream_type,
            substream_id: frame.substream_id,
            channel_tokens: tokens.clone(),
        });
        owned.extend(tokens.iter().copied());
        for token in tokens {
            if !assembled.contains(&token) {
                assembled.push(token);
            }
        }
        dependent_ids.push(frame.substream_id);
        dependent_channel_maps.push(channel_map);
    }

    let expected_channels = match profile {
        JocProfile::Idx0 | JocProfile::Idx3 => vec![0, 1, 2, 3, 4],
        JocProfile::Idx1 => vec![0, 1, 2, 3, 4, 5, 6],
        JocProfile::Idx2 | JocProfile::Idx4 => vec![0, 1, 2, 3, 4, 7, 8],
    };
    assembled.sort_unstable();
    if assembled != expected_channels {
        return Err(format!(
            "assembled channel ownership {:?} does not match {}",
            assembled,
            profile.semantic_name()
        ));
    }

    let final_frame = frames
        .last()
        .ok_or_else(|| "missing final dependent".to_owned())?;
    let addbsi = final_frame
        .addbsi
        .as_ref()
        .ok_or_else(|| "final dependent is missing type-A addbsi".to_owned())?;
    if !addbsi.type_a {
        return Err("final dependent addbsi does not signal JOC type A".to_owned());
    }
    if addbsi.complexity_index == 0 || addbsi.complexity_index > 16 {
        return Err(format!(
            "JOC type-A complexity index {} is outside 1..=16",
            addbsi.complexity_index
        ));
    }
    let emdf = final_frame
        .emdf
        .as_ref()
        .ok_or_else(|| "final dependent is missing JOC/OAMD EMDF".to_owned())?;
    if emdf.payload_ids != [11, 14] {
        return Err(format!(
            "JOC EMDF must contain payload IDs 11 and 14 exactly once, got {:?}",
            emdf.payload_ids
        ));
    }
    if emdf.joc_downmix_index != profile_spec(profile).downmix_index {
        return Err(format!(
            "JOC payload profile index {} does not match {}",
            emdf.joc_downmix_index,
            profile.semantic_name()
        ));
    }

    Ok(MultiDependentValidationReport {
        component_count: frames.len(),
        dependent_ids,
        dependent_channel_maps,
        joc_owner: Some(final_frame.substream_id),
        joc_downmix_index: emdf.joc_downmix_index,
        complexity_index: addbsi.complexity_index,
        payload_ids: emdf.payload_ids.clone(),
        sample_rate: independent.sample_rate,
        numblkscod: independent.numblkscod,
        total_samples: independent.audio_blocks * 256,
        lfe_present: independent.lfe_on,
        unique_dependent_channel_ownership: true,
        ownership_census,
    })
}

/// Validates a source-derived dependent plan before any frame bytes are
/// emitted. This keeps channel ownership and carrier cardinality fail-closed
/// at the writer boundary as well as at the serialized-byte boundary.
pub(crate) fn validate_multi_dependent_plan(
    profile: JocProfile,
    dependent_channel_maps: &[u16],
    dependent_channel_counts: &[usize],
) -> Result<(), String> {
    if dependent_channel_maps.is_empty() || dependent_channel_maps.len() > 8 {
        return Err("multi-dependent JOC requires 1..=8 dependents".to_owned());
    }
    if dependent_channel_maps.len() != dependent_channel_counts.len() {
        return Err("multi-dependent channel maps must match carrier count".to_owned());
    }
    let mut owned = Vec::new();
    let mut assembled = vec![0_u8, 1, 2, 3, 4];
    for (index, (&channel_map, &channel_count)) in dependent_channel_maps
        .iter()
        .zip(dependent_channel_counts)
        .enumerate()
    {
        if !(1..=2).contains(&channel_count) {
            return Err(format!(
                "dependent D{index} requires one or two source-derived channels"
            ));
        }
        let tokens = channel_map_tokens(channel_map)?;
        if tokens.len() != channel_count {
            return Err(format!(
                "dependent D{index} chanmap carries {} channels but the source plan carries {channel_count}",
                tokens.len()
            ));
        }
        if tokens.iter().any(|token| *token >= 100) {
            return Err("source-derived JOC dependents cannot claim an LFE channel".to_owned());
        }
        if tokens.iter().any(|token| owned.contains(token)) {
            return Err(format!("dependent D{index} duplicates channel ownership"));
        }
        owned.extend(tokens.iter().copied());
        for token in tokens {
            if !assembled.contains(&token) {
                assembled.push(token);
            }
        }
    }
    let expected = match profile {
        JocProfile::Idx0 | JocProfile::Idx3 => vec![0, 1, 2, 3, 4],
        JocProfile::Idx1 => vec![0, 1, 2, 3, 4, 5, 6],
        JocProfile::Idx2 | JocProfile::Idx4 => vec![0, 1, 2, 3, 4, 7, 8],
    };
    assembled.sort_unstable();
    if assembled != expected {
        return Err(format!(
            "dependent channel ownership {:?} does not close {}",
            assembled,
            profile.semantic_name()
        ));
    }
    Ok(())
}

/// Returns a canonical no-overlap channel-map plan for a JOC profile.
///
/// The public E-AC-3 syntax permits up to D7, but a Table-47 JOC downmix has
/// only five or seven full-band inputs. This canonical plan therefore closes
/// the useful no-LFE profile space through six dependents; D7 remains a legal
/// raw E-AC-3 count but is outside this profile-specific ownership plan.
pub fn canonical_dependent_channel_maps(
    profile: JocProfile,
    dependent_count: usize,
) -> Result<Vec<u16>, String> {
    let extra = match profile {
        JocProfile::Idx0 | JocProfile::Idx3 => None,
        JocProfile::Idx1 => Some(0x0200),
        JocProfile::Idx2 | JocProfile::Idx4 => Some(0x0010),
    };
    let maps = match (extra, dependent_count) {
        (None, 1) => vec![0xc000],
        (None, 2) => vec![0xc000, 0x3000],
        (None, count @ 3..=5) => [0x8000, 0x4000, 0x2000, 0x1000, 0x0800][..count].to_vec(),
        (Some(extra), 1) => vec![extra],
        (Some(extra), 2) => vec![0xc000, extra],
        (Some(extra), 3) => vec![0xc000, 0x3000, extra],
        (Some(extra), count @ 4..=6) => {
            let mut maps = [0x8000, 0x4000, 0x2000, 0x1000, 0x0800][..count - 1].to_vec();
            maps.push(extra);
            maps
        }
        _ => {
            return Err(format!(
                "{} canonical no-overlap JOC plan supports dependent counts 1..={}",
                profile.semantic_name(),
                if extra.is_some() { 6 } else { 5 }
            ));
        }
    };
    Ok(maps)
}

#[derive(Clone, Debug)]
struct StructuralFrame {
    frame_size: usize,
    stream_type: u8,
    substream_id: u8,
    sample_rate: u32,
    numblkscod: u8,
    audio_blocks: usize,
    audio_coding_mode: u8,
    lfe_on: bool,
    full_bandwidth_channels: usize,
    channel_map: Option<u16>,
    addbsi: Option<Addbsi>,
    emdf: Option<EmdfSummary>,
}

#[derive(Clone, Debug)]
struct Addbsi {
    type_a: bool,
    complexity_index: u8,
}

#[derive(Clone, Debug)]
struct EmdfSummary {
    payload_ids: Vec<u8>,
    joc_downmix_index: u8,
}

struct StructuralBitReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> StructuralBitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn read(&mut self, width: usize) -> Result<u32, String> {
        let end = self
            .position
            .checked_add(width)
            .ok_or_else(|| "E-AC-3 structural bit position overflow".to_owned())?;
        if end > self.bytes.len() * 8 {
            return Err(format!(
                "E-AC-3 structural read exceeded frame bounds at bit {} (width {}, {} bytes)",
                self.position,
                width,
                self.bytes.len()
            ));
        }
        let mut value = 0_u32;
        for _ in 0..width {
            value = (value << 1)
                | u32::from((self.bytes[self.position / 8] >> (7 - self.position % 8)) & 1);
            self.position += 1;
        }
        Ok(value)
    }

    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>, String> {
        (0..count)
            .map(|_| self.read(8).map(|value| value as u8))
            .collect()
    }
}

fn parse_structural_frame(stream: &[u8], offset: usize) -> Result<StructuralFrame, String> {
    parse_structural_frame_inner(stream, offset)
        .map_err(|error| format!("frame at byte offset {offset}: {error}"))
}

fn parse_structural_frame_inner(stream: &[u8], offset: usize) -> Result<StructuralFrame, String> {
    let header = stream
        .get(offset..)
        .ok_or_else(|| "E-AC-3 frame offset is outside stream".to_owned())?;
    let mut reader = StructuralBitReader::new(header);
    if reader.read(16)? != 0x0b77 {
        return Err(format!("invalid E-AC-3 syncword at byte offset {offset}"));
    }
    let stream_type = reader.read(2)? as u8;
    let substream_id = reader.read(3)? as u8;
    let frame_size = ((reader.read(11)? as usize) + 1) * 2;
    if frame_size < 8 {
        return Err(format!("invalid structural frame size {frame_size}"));
    }
    let frame = stream
        .get(offset..offset.saturating_add(frame_size))
        .filter(|frame| frame.len() == frame_size)
        .ok_or_else(|| format!("truncated E-AC-3 frame at byte offset {offset}"))?;
    let mut reader = StructuralBitReader::new(frame);
    let _ = reader.read(16)?;
    let _ = reader.read(2)?;
    let _ = reader.read(3)?;
    let _ = reader.read(11)?;
    let sample_rate = match reader.read(2)? as u8 {
        0 => 48_000,
        1 => 44_100,
        2 => 32_000,
        _ => return Err("reserved E-AC-3 sample-rate code".to_owned()),
    };
    let numblkscod = reader.read(2)? as u8;
    let audio_blocks = match numblkscod {
        0 => 1,
        1 => 2,
        2 => 3,
        3 => 6,
        _ => unreachable!(),
    };
    let audio_coding_mode = reader.read(3)? as u8;
    let lfe_on = reader.read(1)? != 0;
    let full_bandwidth_channels = standard_full_bandwidth_channels(audio_coding_mode)?;
    let bitstream_id = reader.read(5)? as u8;
    if bitstream_id != 16 {
        return Err(format!(
            "unsupported E-AC-3 bsid {bitstream_id}; expected 16"
        ));
    }
    let _dialnorm = reader.read(5)?;
    if reader.read(1)? != 0 {
        let _ = reader.read(8)?;
        if audio_coding_mode == 0 {
            let _ = reader.read(1)?;
            let _ = reader.read(8)?;
        }
    }
    let channel_map = if stream_type == 1 && reader.read(1)? != 0 {
        Some(reader.read(16)? as u16)
    } else {
        None
    };
    if reader.read(1)? != 0 {
        return Err("mixing metadata is outside the canonical structural fixture".to_owned());
    }
    if reader.read(1)? != 0 {
        return Err(
            "informational metadata is outside the canonical structural fixture".to_owned(),
        );
    }
    let addbsi = if reader.read(1)? != 0 {
        let byte_count = reader.read(6)? as usize + 1;
        let bytes = reader.read_bytes(byte_count)?;
        if bytes.len() < 2 {
            return Err("JOC addbsi is shorter than the public type-A fields".to_owned());
        }
        Some(Addbsi {
            type_a: bytes[0] & 1 != 0,
            complexity_index: bytes[1],
        })
    } else {
        None
    };
    let emdf = parse_emdf(frame)
        .map_err(|error| format!("frame at byte offset {offset} EMDF: {error}"))?;
    Ok(StructuralFrame {
        frame_size,
        stream_type,
        substream_id,
        sample_rate,
        numblkscod,
        audio_blocks,
        audio_coding_mode,
        lfe_on,
        full_bandwidth_channels,
        channel_map,
        addbsi,
        emdf,
    })
}

fn parse_emdf(frame: &[u8]) -> Result<Option<EmdfSummary>, String> {
    let marker = frame
        .windows(2)
        .enumerate()
        .rev()
        .find_map(|(index, window)| {
            if window != [0x58, 0x38] {
                return None;
            }
            let length = frame.get(index + 2..index + 4)?.try_into().ok()?;
            let body_len = usize::from(u16::from_be_bytes(length));
            (index.checked_add(4 + body_len) == Some(frame.len().saturating_sub(4)))
                .then_some((index, body_len))
        });
    let Some((marker, body_len)) = marker else {
        return Ok(None);
    };
    if frame.len() < 4 {
        return Err("EMDF frame is shorter than its trailer".to_owned());
    }
    let mut trailer = StructuralBitReader::new(&frame[frame.len() - 4..]);
    let declared_bits = trailer.read(14)? as usize;
    let auxdatae = trailer.read(1)?;
    let expected_bits = (body_len + 4)
        .checked_mul(8)
        .ok_or_else(|| "EMDF length overflow".to_owned())?;
    if auxdatae == 0 || declared_bits != expected_bits {
        return Err("EMDF trailer does not match its frame-end auxdata".to_owned());
    }
    let body = frame
        .get(marker + 4..marker + 4 + body_len)
        .ok_or_else(|| "truncated EMDF body".to_owned())?;
    let mut reader = StructuralBitReader::new(body);
    let _version = reader.read(2)?;
    let _key_identifier = reader.read(3)?;
    let mut payloads = Vec::new();
    let mut joc_downmix_index = None;
    loop {
        let payload_id = reader.read(5)? as u8;
        if payload_id == 0 {
            break;
        }
        let _sample_offset_present = reader.read(1)?;
        let _duration_present = reader.read(1)?;
        if reader.read(1)? != 0 {
            let _group_id = reader.read(2)?;
            let _group_id_terminates = reader.read(1)?;
        }
        if reader.read(1)? != 0 {
            let _codec_data = reader.read(8)?;
        }
        let _discard_unknown = reader.read(1)?;
        let _frame_aligned = reader.read(1)?;
        let _create_duplicate = reader.read(1)?;
        let _remove_duplicate = reader.read(1)?;
        let _priority = reader.read(5)?;
        let _processing_allowed = reader.read(2)?;
        let payload_len = reader.read(8)? as usize;
        let _length_terminates = reader.read(1)?;
        let payload = reader.read_bytes(payload_len)?;
        if payload_id == 14 {
            let mut payload_reader = StructuralBitReader::new(&payload);
            joc_downmix_index = Some(payload_reader.read(3)? as u8);
        }
        payloads.push(payload_id);
    }
    Ok(Some(EmdfSummary {
        payload_ids: payloads,
        joc_downmix_index: joc_downmix_index
            .ok_or_else(|| "EMDF is missing JOC payload 14".to_owned())?,
    }))
}

fn standard_full_bandwidth_channels(audio_coding_mode: u8) -> Result<usize, String> {
    match audio_coding_mode {
        0 | 2 => Ok(2),
        1 => Ok(1),
        3 | 4 => Ok(3),
        5 | 6 => Ok(4),
        7 => Ok(5),
        _ => Err(format!("unsupported E-AC-3 acmod {audio_coding_mode}")),
    }
}

pub(crate) fn channel_map_tokens(channel_map: u16) -> Result<Vec<u8>, String> {
    if channel_map == 0 {
        return Err("dependent chanmap must contain at least one channel".to_owned());
    }
    let mut tokens = Vec::new();
    for bit in 0..16_u8 {
        if channel_map & (1_u16 << (15 - bit)) == 0 {
            continue;
        }
        match bit {
            0..=4 => tokens.push(bit),
            5 => tokens.extend([9, 10]),
            6 => tokens.extend([5, 6]),
            7..=8 => tokens.push(bit + 4),
            9 => tokens.extend([13, 14]),
            10 => tokens.extend([15, 16]),
            11 => tokens.extend([7, 8]),
            12 => tokens.push(17),
            13 => tokens.extend([18, 19]),
            14 => tokens.push(100),
            15 => tokens.push(101),
            _ => unreachable!(),
        }
    }
    Ok(tokens)
}

/// Returns the number of coded channels represented by a custom chanmap.
#[must_use]
pub fn channel_map_channel_count(channel_map: u16) -> usize {
    channel_map_tokens(channel_map).map_or(0, |tokens| tokens.len())
}
