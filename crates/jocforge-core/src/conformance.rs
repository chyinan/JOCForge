// pattern: Functional Core

use crate::{EmdfProfile, JocProfile, profile_spec};
use std::fmt;

const AC3_48_KBPS: [usize; 19] = [
    32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384, 448, 512, 576, 640,
];

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

/// Classification of a complete six-block raw JOC programme partition.
///
/// The E-AC-3 syntax defines the legal count for each syncframe. The normal
/// JOCForge writer deliberately closes only the canonical pure families;
/// other six-block compositions remain available to the lower-level
/// conformance planner but are not silently enabled for production output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockPartitionStatus {
    PublicLegal,
    PublicAmbiguous,
    Invalid,
}

impl fmt::Display for BlockPartitionStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::PublicLegal => "PUBLIC_LEGAL",
            Self::PublicAmbiguous => "PUBLIC_AMBIGUOUS",
            Self::Invalid => "INVALID",
        })
    }
}

/// One source-sample range represented by one short syncframe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShortBlockFrameRange {
    pub syncframe_index: usize,
    pub start_sample: usize,
    pub end_sample: usize,
    pub audio_blocks: usize,
}

/// Independent structural report for one grouped short-block JOC interval.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShortBlockValidationReport {
    pub syncframes: usize,
    pub audio_blocks: usize,
    pub samples: usize,
    pub metadata_owner_frame: usize,
    pub lfe_present: bool,
    pub complexity_index: u8,
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

    #[must_use]
    pub fn is_default(&self) -> bool {
        self.partitions.as_slice() == [BlockPartition::Six]
    }

    #[must_use]
    pub fn frame_ranges(&self, start_sample: usize) -> Vec<ShortBlockFrameRange> {
        let mut cursor = start_sample;
        self.partitions
            .iter()
            .copied()
            .enumerate()
            .map(|(syncframe_index, partition)| {
                let start = cursor;
                let end = start.saturating_add(partition.audio_blocks() * 256);
                cursor = end;
                ShortBlockFrameRange {
                    syncframe_index,
                    start_sample: start,
                    end_sample: end,
                    audio_blocks: partition.audio_blocks(),
                }
            })
            .collect()
    }
}

/// Classifies the public closure boundary for one complete six-block group.
#[must_use]
pub fn classify_block_partition(partitions: &[BlockPartition]) -> BlockPartitionStatus {
    let total_blocks = partitions
        .iter()
        .map(|partition| partition.audio_blocks())
        .sum::<usize>();
    if partitions.is_empty() || total_blocks != 6 {
        return BlockPartitionStatus::Invalid;
    }
    match partitions {
        [BlockPartition::Six]
        | [BlockPartition::Three, BlockPartition::Three]
        | [
            BlockPartition::Two,
            BlockPartition::Two,
            BlockPartition::Two,
        ]
        | [
            BlockPartition::One,
            BlockPartition::One,
            BlockPartition::One,
            BlockPartition::One,
            BlockPartition::One,
            BlockPartition::One,
        ] => BlockPartitionStatus::PublicLegal,
        _ => BlockPartitionStatus::PublicAmbiguous,
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

/// First-class LFE ownership policy for a dependent plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DependentLfePolicy {
    Absent,
    Supplement,
    ReplaceIndependent,
}

/// One dependent substream's public channel ownership declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependentPlan {
    pub id: u8,
    pub channel_map: u16,
    pub lfe_policy: DependentLfePolicy,
}

impl DependentPlan {
    #[must_use]
    pub const fn new(id: u8, channel_map: u16, lfe_policy: DependentLfePolicy) -> Self {
        Self {
            id,
            channel_map,
            lfe_policy,
        }
    }

    #[must_use]
    pub fn full_band_channel_count(&self) -> usize {
        channel_map_tokens(self.channel_map).map_or(0, |tokens| {
            tokens.into_iter().filter(|token| *token < 100).count()
        })
    }

    #[must_use]
    pub fn has_lfe(&self) -> bool {
        channel_map_tokens(self.channel_map).is_ok_and(|tokens| tokens.contains(&101))
    }
}

/// Programme-level owner used by the ownership report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgrammeOwner {
    Independent,
    Dependent(u8),
}

/// Immutable ownership plan consumed by the raw multi-dependent writer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnershipPlan {
    pub profile: JocProfile,
    pub independent_lfe: bool,
    pub dependents: Vec<DependentPlan>,
    pub final_joc_owner: u8,
}

/// Capability state for one public profile/topology cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityVerdict {
    SupportedPass,
    SemanticCeiling,
    SpecLegalNotImplemented,
    RejectedCapacity,
    PublicAmbiguous,
    NotApplicable,
    Invalid,
    Reserved,
}

impl fmt::Display for CapabilityVerdict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::SupportedPass => "SUPPORTED_PASS",
            Self::SemanticCeiling => "SEMANTIC_CEILING",
            Self::SpecLegalNotImplemented => "SPEC_LEGAL_NOT_IMPLEMENTED",
            Self::RejectedCapacity => "REJECTED_CAPACITY",
            Self::PublicAmbiguous => "PUBLIC_AMBIGUOUS",
            Self::NotApplicable => "NOT_APPLICABLE",
            Self::Invalid => "INVALID",
            Self::Reserved => "RESERVED",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnershipPlanError {
    EmptyPlan,
    TooManyDependents { actual: usize },
    SemanticCeiling { actual: usize, maximum: usize },
    NonSequentialId { expected: u8, actual: u8 },
    EmptyChannelMap { id: u8 },
    NoFullBandChannel { id: u8 },
    UnsupportedLfe2 { id: u8 },
    LfePolicyConflict,
    MultipleDependentLfe,
    DuplicateDependentChannel { id: u8 },
    UnexpectedChannel { id: u8, token: u8 },
    FinalJocOwnerMismatch { expected: u8, actual: u8 },
}

impl fmt::Display for OwnershipPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPlan => {
                formatter.write_str("ownership plan requires at least one dependent")
            }
            Self::TooManyDependents { actual } => {
                write!(
                    formatter,
                    "ownership plan has {actual} dependents; maximum is 8"
                )
            }
            Self::SemanticCeiling { actual, maximum } => {
                write!(
                    formatter,
                    "ownership plan has {actual} dependents; canonical semantic maximum is {maximum}"
                )
            }
            Self::NonSequentialId { expected, actual } => {
                write!(
                    formatter,
                    "ownership plan expected D{expected}, got D{actual}"
                )
            }
            Self::EmptyChannelMap { id } => {
                write!(formatter, "dependent D{id} has an empty channel map")
            }
            Self::NoFullBandChannel { id } => {
                write!(
                    formatter,
                    "dependent D{id} must carry at least one full-band channel"
                )
            }
            Self::UnsupportedLfe2 { id } => {
                write!(
                    formatter,
                    "dependent D{id} claims unsupported LFE2 ownership"
                )
            }
            Self::LfePolicyConflict => {
                formatter.write_str("dependent LFE policy conflicts with independent LFE ownership")
            }
            Self::MultipleDependentLfe => {
                formatter.write_str("multiple dependents claim LFE ownership")
            }
            Self::DuplicateDependentChannel { id } => {
                write!(
                    formatter,
                    "dependent D{id} duplicates dependent full-band ownership"
                )
            }
            Self::UnexpectedChannel { id, token } => {
                write!(
                    formatter,
                    "dependent D{id} claims channel token {token} outside the selected JOC profile"
                )
            }
            Self::FinalJocOwnerMismatch { expected, actual } => {
                write!(
                    formatter,
                    "final JOC owner must be D{expected}, got D{actual}"
                )
            }
        }
    }
}

impl std::error::Error for OwnershipPlanError {}

/// Machine-readable result of validating an ownership plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnershipValidationReport {
    pub lfe_owner: Option<ProgrammeOwner>,
    pub full_band_channel_roles: Vec<u8>,
    pub semantic_maximum: usize,
    pub final_joc_owner: u8,
}

/// Complete public Table E.1.4 dependent channel-role census.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DependentChannelRoleSpec {
    pub public_role: &'static str,
    pub speaker_channel_semantic: &'static str,
    pub chanmap_syntax: &'static str,
    pub replacement_supplement_meaning: &'static str,
    pub lfe: bool,
    pub applicable_base_acmod: &'static str,
    pub conflict_rules: &'static str,
    pub public_clause_table: &'static str,
}

pub const DEPENDENT_CHANNEL_ROLE_CENSUS: &[DependentChannelRoleSpec] = &[
    DependentChannelRoleSpec {
        public_role: "Left",
        speaker_channel_semantic: "L",
        chanmap_syntax: "bit 0 / 0x8000",
        replacement_supplement_meaning: "replacement if present in I0; supplement otherwise",
        lfe: false,
        applicable_base_acmod: "dependent acmod/lfeon cardinality",
        conflict_rules: "duplicate dependent role rejected by canonical plan",
        public_clause_table: "TS 102 366 E.1.3.1.7-E.1.3.1.8, Table E.1.4, E.2.8.2",
    },
    DependentChannelRoleSpec {
        public_role: "Centre",
        speaker_channel_semantic: "C",
        chanmap_syntax: "bit 1 / 0x4000",
        replacement_supplement_meaning: "replacement if present in I0; supplement otherwise",
        lfe: false,
        applicable_base_acmod: "dependent acmod/lfeon cardinality",
        conflict_rules: "duplicate dependent role rejected by canonical plan",
        public_clause_table: "TS 102 366 E.1.3.1.7-E.1.3.1.8, Table E.1.4, E.2.8.2",
    },
    DependentChannelRoleSpec {
        public_role: "Right",
        speaker_channel_semantic: "R",
        chanmap_syntax: "bit 2 / 0x2000",
        replacement_supplement_meaning: "replacement if present in I0; supplement otherwise",
        lfe: false,
        applicable_base_acmod: "dependent acmod/lfeon cardinality",
        conflict_rules: "duplicate dependent role rejected by canonical plan",
        public_clause_table: "TS 102 366 E.1.3.1.7-E.1.3.1.8, Table E.1.4, E.2.8.2",
    },
    DependentChannelRoleSpec {
        public_role: "Left Surround",
        speaker_channel_semantic: "Ls",
        chanmap_syntax: "bit 3 / 0x1000",
        replacement_supplement_meaning: "replacement if present in I0; supplement otherwise",
        lfe: false,
        applicable_base_acmod: "dependent acmod/lfeon cardinality",
        conflict_rules: "duplicate dependent role rejected by canonical plan",
        public_clause_table: "TS 102 366 E.1.3.1.7-E.1.3.1.8, Table E.1.4, E.2.8.2",
    },
    DependentChannelRoleSpec {
        public_role: "Right Surround",
        speaker_channel_semantic: "Rs",
        chanmap_syntax: "bit 4 / 0x0800",
        replacement_supplement_meaning: "replacement if present in I0; supplement otherwise",
        lfe: false,
        applicable_base_acmod: "dependent acmod/lfeon cardinality",
        conflict_rules: "duplicate dependent role rejected by canonical plan",
        public_clause_table: "TS 102 366 E.1.3.1.7-E.1.3.1.8, Table E.1.4, E.2.8.2",
    },
    DependentChannelRoleSpec {
        public_role: "Lc/Rc pair",
        speaker_channel_semantic: "left/right centre pair",
        chanmap_syntax: "bit 5 / 0x0400",
        replacement_supplement_meaning: "replacement if present in I0; supplement otherwise",
        lfe: false,
        applicable_base_acmod: "dependent acmod/lfeon cardinality",
        conflict_rules: "duplicate pair or member role rejected by canonical plan",
        public_clause_table: "TS 102 366 E.1.3.1.7-E.1.3.1.8, Table E.1.4, E.2.8.2",
    },
    DependentChannelRoleSpec {
        public_role: "Lrs/Rrs pair",
        speaker_channel_semantic: "left/right rear surround pair",
        chanmap_syntax: "bit 6 / 0x0200",
        replacement_supplement_meaning: "replacement if present in I0; supplement otherwise",
        lfe: false,
        applicable_base_acmod: "dependent acmod/lfeon cardinality",
        conflict_rules: "duplicate pair or member role rejected by canonical plan",
        public_clause_table: "TS 102 366 E.1.3.1.7-E.1.3.1.8, Table E.1.4, E.2.8.2",
    },
    DependentChannelRoleSpec {
        public_role: "Cs",
        speaker_channel_semantic: "centre surround",
        chanmap_syntax: "bit 7 / 0x0100",
        replacement_supplement_meaning: "replacement if present in I0; supplement otherwise",
        lfe: false,
        applicable_base_acmod: "dependent acmod/lfeon cardinality",
        conflict_rules: "duplicate dependent role rejected by canonical plan",
        public_clause_table: "TS 102 366 E.1.3.1.7-E.1.3.1.8, Table E.1.4, E.2.8.2",
    },
    DependentChannelRoleSpec {
        public_role: "Ts",
        speaker_channel_semantic: "top surround",
        chanmap_syntax: "bit 8 / 0x0080",
        replacement_supplement_meaning: "replacement if present in I0; supplement otherwise",
        lfe: false,
        applicable_base_acmod: "dependent acmod/lfeon cardinality",
        conflict_rules: "duplicate dependent role rejected by canonical plan",
        public_clause_table: "TS 102 366 E.1.3.1.7-E.1.3.1.8, Table E.1.4, E.2.8.2",
    },
    DependentChannelRoleSpec {
        public_role: "Lsd/Rsd pair",
        speaker_channel_semantic: "left/right side surround pair",
        chanmap_syntax: "bit 9 / 0x0040",
        replacement_supplement_meaning: "replacement if present in I0; supplement otherwise",
        lfe: false,
        applicable_base_acmod: "dependent acmod/lfeon cardinality",
        conflict_rules: "duplicate pair or member role rejected by canonical plan",
        public_clause_table: "TS 102 366 E.1.3.1.7-E.1.3.1.8, Table E.1.4, E.2.8.2",
    },
    DependentChannelRoleSpec {
        public_role: "Lw/Rw pair",
        speaker_channel_semantic: "left/right wide pair",
        chanmap_syntax: "bit 10 / 0x0020",
        replacement_supplement_meaning: "replacement if present in I0; supplement otherwise",
        lfe: false,
        applicable_base_acmod: "dependent acmod/lfeon cardinality",
        conflict_rules: "duplicate pair or member role rejected by canonical plan",
        public_clause_table: "TS 102 366 E.1.3.1.7-E.1.3.1.8, Table E.1.4, E.2.8.2",
    },
    DependentChannelRoleSpec {
        public_role: "Vhl/Vhr pair",
        speaker_channel_semantic: "vertical height left/right pair",
        chanmap_syntax: "bit 11 / 0x0010",
        replacement_supplement_meaning: "replacement if present in I0; supplement otherwise",
        lfe: false,
        applicable_base_acmod: "dependent acmod/lfeon cardinality",
        conflict_rules: "duplicate pair or member role rejected by canonical plan",
        public_clause_table: "TS 102 366 E.1.3.1.7-E.1.3.1.8, Table E.1.4, E.2.8.2",
    },
    DependentChannelRoleSpec {
        public_role: "Vhc",
        speaker_channel_semantic: "vertical height centre",
        chanmap_syntax: "bit 12 / 0x0008",
        replacement_supplement_meaning: "replacement if present in I0; supplement otherwise",
        lfe: false,
        applicable_base_acmod: "dependent acmod/lfeon cardinality",
        conflict_rules: "duplicate dependent role rejected by canonical plan",
        public_clause_table: "TS 102 366 E.1.3.1.7-E.1.3.1.8, Table E.1.4, E.2.8.2",
    },
    DependentChannelRoleSpec {
        public_role: "Lts/Rts pair",
        speaker_channel_semantic: "left/right top side pair",
        chanmap_syntax: "bit 13 / 0x0004",
        replacement_supplement_meaning: "replacement if present in I0; supplement otherwise",
        lfe: false,
        applicable_base_acmod: "dependent acmod/lfeon cardinality",
        conflict_rules: "duplicate pair or member role rejected by canonical plan",
        public_clause_table: "TS 102 366 E.1.3.1.7-E.1.3.1.8, Table E.1.4, E.2.8.2",
    },
    DependentChannelRoleSpec {
        public_role: "LFE2",
        speaker_channel_semantic: "second low-frequency effects channel",
        chanmap_syntax: "bit 14 / 0x0002",
        replacement_supplement_meaning: "public E-AC-3 location; not admitted by canonical single-LFE JOC plan",
        lfe: true,
        applicable_base_acmod: "dependent lfeon/map cardinality",
        conflict_rules: "LFE2 is not Table 47 LFE(0); fail closed",
        public_clause_table: "TS 102 366 E.1.3.1.7-E.1.3.1.8, Table E.1.4",
    },
    DependentChannelRoleSpec {
        public_role: "LFE",
        speaker_channel_semantic: "low-frequency effects channel",
        chanmap_syntax: "bit 15 / 0x0001 or lfeon=1",
        replacement_supplement_meaning: "replacement of I0 LFE if present; supplement if I0 has no LFE",
        lfe: true,
        applicable_base_acmod: "dependent lfeon/map cardinality",
        conflict_rules: "exactly one final LFE owner in canonical plan",
        public_clause_table: "TS 102 366 E.1.3.1.7-E.1.3.1.8, Table E.1.4, E.2.8.2",
    },
];

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
    pub lfe_owner: Option<ProgrammeOwner>,
    pub semantic_maximum: usize,
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

/// Validates the independently serialised two-substream form used by the
/// source-derived short writer. Metadata is owned by the final D0 of the
/// grouped interval, while each preceding frame remains audio-only.
pub fn validate_short_block_access_unit(
    stream: &[u8],
    profile: JocProfile,
    partitions: &[BlockPartition],
) -> Result<ShortBlockValidationReport, String> {
    let plan = resolve_short_block_plan(partitions)
        .map_err(|error| format!("invalid short-block validation plan: {error}"))?;
    if classify_block_partition(partitions) != BlockPartitionStatus::PublicLegal {
        return Err("short-block validator requires a publicly closed partition".to_owned());
    }
    if partitions.contains(&BlockPartition::Six) {
        return Err("short-block validator requires only 1/2/3-block frames".to_owned());
    }
    let expected_frames = partitions.len() * 2;
    let mut frames = Vec::with_capacity(expected_frames);
    let mut offset = 0_usize;
    while offset < stream.len() {
        let frame = parse_structural_frame(stream, offset)?;
        let frame_end = offset
            .checked_add(frame.frame_size)
            .ok_or_else(|| "short-block frame offset overflow".to_owned())?;
        crate::eac3::validate_crc2(
            stream
                .get(offset..frame_end)
                .ok_or_else(|| "short-block frame exceeds stream bounds".to_owned())?,
        )?;
        offset = frame_end;
        frames.push(frame);
    }
    if offset != stream.len() || frames.len() != expected_frames {
        return Err(format!(
            "short-block access unit contains {} frames; expected {expected_frames}",
            frames.len()
        ));
    }
    let expected_channel_map = profile_spec(profile).channel_map;
    let lfe_present = frames.first().is_some_and(|frame| frame.lfe_on);
    let mut complexity_index = 0;
    for (index, partition) in partitions.iter().copied().enumerate() {
        let independent = &frames[index * 2];
        let dependent = &frames[index * 2 + 1];
        if independent.stream_type != 0
            || independent.substream_id != 0
            || independent.sample_rate != 48_000
            || independent.audio_blocks != partition.audio_blocks()
            || independent.audio_coding_mode != 7
            || independent.channel_map.is_some()
            || independent.lfe_on != lfe_present
            || independent.addbsi.is_some()
            || independent.emdf.is_some()
            || independent.convsync != Some(plan.convsync()[index])
        {
            return Err(format!("invalid short-block I0 at syncframe group {index}"));
        }
        if dependent.stream_type != 1
            || dependent.substream_id != 0
            || dependent.sample_rate != 48_000
            || dependent.audio_blocks != partition.audio_blocks()
            || dependent.channel_map != Some(expected_channel_map)
            || dependent.lfe_on
            || dependent.convsync.is_some()
        {
            return Err(format!("invalid short-block D0 at syncframe group {index}"));
        }
        let final_frame = index + 1 == partitions.len();
        if dependent.addbsi.is_some() != final_frame || dependent.emdf.is_some() != final_frame {
            return Err(format!(
                "short-block JOC metadata must be owned by final D0; group {index}"
            ));
        }
        if let Some(addbsi) = &dependent.addbsi {
            if !addbsi.type_a || addbsi.complexity_index == 0 || dependent.emdf.is_none() {
                return Err("short-block final D0 has invalid JOC addbsi".to_owned());
            }
            complexity_index = addbsi.complexity_index;
        }
        if let Some(emdf) = &dependent.emdf {
            if emdf.payload_ids != vec![11, 14]
                || emdf.joc_downmix_index != profile_spec(profile).downmix_index
            {
                return Err("short-block final D0 has invalid JOC EMDF profile".to_owned());
            }
        }
    }
    Ok(ShortBlockValidationReport {
        syncframes: frames.len(),
        audio_blocks: plan.audio_blocks(),
        samples: plan.samples(),
        metadata_owner_frame: expected_frames - 1,
        lfe_present,
        complexity_index,
    })
}

/// Validates a grouped short I0-only skip-field programme. The final I0 owns
/// the exact supplied JOC/OAMD EMDF body in its first audio-block skip field;
/// no dependent frame is allowed in this carriage.
pub fn validate_short_skip_field_access_unit(
    stream: &[u8],
    profile: JocProfile,
    emdf_profile: EmdfProfile,
    expected_emdf: &[u8],
    partitions: &[BlockPartition],
    lfe_present: bool,
) -> Result<ShortBlockValidationReport, String> {
    let plan = resolve_short_block_plan(partitions)
        .map_err(|error| format!("invalid short skip-field validation plan: {error}"))?;
    if classify_block_partition(partitions) != BlockPartitionStatus::PublicLegal {
        return Err("short skip-field validator requires a publicly closed partition".to_owned());
    }
    let mut frames = Vec::with_capacity(partitions.len());
    let mut offset = 0_usize;
    while offset < stream.len() {
        let frame = parse_structural_frame(stream, offset)?;
        let frame_end = offset
            .checked_add(frame.frame_size)
            .ok_or_else(|| "short skip-field frame offset overflow".to_owned())?;
        let bytes = stream
            .get(offset..frame_end)
            .ok_or_else(|| "short skip-field frame exceeds stream bounds".to_owned())?;
        crate::eac3::validate_crc2(bytes)?;
        offset = frame_end;
        frames.push((frame, bytes));
    }
    if offset != stream.len() || frames.len() != partitions.len() {
        return Err(format!(
            "short skip-field access unit contains {} frames; expected {}",
            frames.len(),
            partitions.len()
        ));
    }
    let expected_downmix_index = profile_spec(profile).downmix_index;
    let mut complexity_index = 0;
    for (index, (frame, bytes)) in frames.iter().enumerate() {
        if frame.stream_type != 0
            || frame.substream_id != 0
            || frame.sample_rate != 48_000
            || frame.audio_blocks != partitions[index].audio_blocks()
            || frame.audio_coding_mode != 7
            || frame.channel_map.is_some()
            || frame.lfe_on != lfe_present
            || frame.emdf.is_some()
            || frame.convsync
                != if partitions[index] == BlockPartition::Six {
                    None
                } else {
                    Some(plan.convsync()[index])
                }
        {
            return Err(format!("invalid short skip-field I0 at syncframe {index}"));
        }
        let skip_enabled = short_skip_field_enabled(frame, bytes)?;
        let embedded = embedded_emdf(bytes, expected_emdf)?;
        let final_frame = index + 1 == frames.len();
        if skip_enabled != final_frame || embedded.is_some() != final_frame {
            return Err(format!(
                "short skip-field EMDF must be present only on final I0; frame {index}, skip_enabled={skip_enabled}, embedded={}",
                embedded.is_some()
            ));
        }
        if let Some((summary, payload_start, _body)) = embedded {
            let payload_end = payload_start
                .checked_add(expected_emdf.len().saturating_mul(8))
                .ok_or_else(|| "short skip-field EMDF payload offset overflows".to_owned())?;
            if payload_start < 10 || payload_end > bytes.len().saturating_mul(8) {
                return Err(
                    "short skip-field EMDF payload exceeds its physical skip-field range"
                        .to_owned(),
                );
            }
            if summary.payload_ids != vec![11, 14]
                || summary.joc_downmix_index != expected_downmix_index
                || summary.codec_data_present != vec![emdf_profile == EmdfProfile::EtsiStrict; 2]
            {
                return Err("short skip-field EMDF has an invalid profile or signaling".to_owned());
            }
            if frame
                .addbsi
                .as_ref()
                .is_none_or(|addbsi| !addbsi.type_a || addbsi.complexity_index == 0)
            {
                return Err("short skip-field final I0 has invalid JOC addbsi".to_owned());
            }
            complexity_index = frame
                .addbsi
                .as_ref()
                .map_or(0, |addbsi| addbsi.complexity_index);
        } else if frame.addbsi.is_some() {
            return Err("short skip-field non-final I0 unexpectedly owns JOC addbsi".to_owned());
        }
    }
    Ok(ShortBlockValidationReport {
        syncframes: frames.len(),
        audio_blocks: plan.audio_blocks(),
        samples: plan.samples(),
        metadata_owner_frame: frames.len() - 1,
        lfe_present,
        complexity_index,
    })
}

/// Validates the public legacy original-syntax AC-3 I0 plus one E-AC-3 D0
/// form. The AC-3 header is checked independently from the E-AC-3 structural
/// parser so the legacy frame is not mistaken for an E-AC-3 stream header.
pub fn validate_legacy_ac3_source_derived_access_unit(
    stream: &[u8],
    profile: JocProfile,
    ownership_plan: &OwnershipPlan,
    emdf_profile: EmdfProfile,
    expected_i0_frame_bytes: usize,
) -> Result<(), String> {
    if ownership_plan.profile != profile || ownership_plan.dependents.len() != 1 {
        return Err("legacy validator requires one matching dependent ownership plan".to_owned());
    }
    let i0 = stream
        .get(..expected_i0_frame_bytes)
        .ok_or_else(|| "legacy AC-3 I0 is truncated".to_owned())?;
    let mut reader = StructuralBitReader::new(i0);
    if reader.read(16)? != 0x0b77 {
        return Err("legacy AC-3 I0 has an invalid syncword".to_owned());
    }
    reader.read(16)?; // original-syntax CRC1
    if reader.read(2)? != 0 {
        return Err("legacy AC-3 I0 is not 48 kHz".to_owned());
    }
    let frame_size_code = reader.read(6)? as usize;
    let declared_i0_bytes = AC3_48_KBPS
        .get(frame_size_code / 2)
        .and_then(|bitrate| bitrate.checked_mul(4))
        .ok_or_else(|| "legacy AC-3 I0 has an invalid frmsizecod".to_owned())?;
    if declared_i0_bytes != expected_i0_frame_bytes {
        return Err(format!(
            "legacy AC-3 I0 frmsizecod declares {declared_i0_bytes} bytes; expected {expected_i0_frame_bytes}"
        ));
    }
    let bitstream_id = reader.read(5)?;
    if !matches!(bitstream_id, 6 | 8) {
        return Err("legacy AC-3 I0 requires bsid 6 or 8".to_owned());
    }
    reader.read(3)?; // bsmod
    let audio_coding_mode = reader.read(3)? as u8;
    if audio_coding_mode != 7 {
        return Err("legacy AC-3 I0 requires the public 5.1 acmod".to_owned());
    }
    if audio_coding_mode & 1 != 0 && audio_coding_mode != 1 {
        reader.read(2)?; // cmixlev
    }
    if audio_coding_mode & 4 != 0 {
        reader.read(2)?; // surmixlev
    }
    if audio_coding_mode == 2 {
        reader.read(2)?; // dsurmod
    }
    let i0_lfe = reader.read(1)? != 0;
    if i0_lfe != ownership_plan.independent_lfe {
        return Err("legacy AC-3 I0 LFE state disagrees with ownership plan".to_owned());
    }
    crate::eac3::validate_legacy_ac3_crc(i0)?;
    let d0 = parse_structural_frame(stream, expected_i0_frame_bytes)?;
    let d0_end = expected_i0_frame_bytes
        .checked_add(d0.frame_size)
        .ok_or_else(|| "legacy D0 frame offset overflows".to_owned())?;
    if d0_end != stream.len()
        || d0.stream_type != 1
        || d0.substream_id != ownership_plan.dependents[0].id
        || d0.sample_rate != 48_000
        || d0.audio_blocks != 6
        || d0.channel_map != Some(ownership_plan.dependents[0].channel_map)
        || d0.lfe_on != ownership_plan.dependents[0].has_lfe()
    {
        return Err("legacy E-AC-3 D0 does not match the ownership plan".to_owned());
    }
    crate::eac3::validate_crc2(
        stream
            .get(expected_i0_frame_bytes..d0_end)
            .ok_or_else(|| "legacy D0 frame is truncated".to_owned())?,
    )?;
    let Some(addbsi) = d0.addbsi else {
        return Err("legacy D0 is missing final JOC addbsi".to_owned());
    };
    if !addbsi.type_a || addbsi.complexity_index == 0 {
        return Err("legacy D0 has invalid JOC addbsi".to_owned());
    }
    let Some(emdf) = d0.emdf else {
        return Err("legacy D0 is missing final JOC EMDF".to_owned());
    };
    if emdf.payload_ids != vec![11, 14]
        || emdf.joc_downmix_index != profile_spec(profile).downmix_index
        || emdf.codec_data_present != vec![emdf_profile == EmdfProfile::EtsiStrict; 2]
    {
        return Err("legacy D0 EMDF has an invalid profile or signaling".to_owned());
    }
    Ok(())
}

/// Validates a grouped short-block programme with one I0 and a shared
/// partition across every associated D0..D7 dependent.
pub fn validate_short_multi_dependent_access_unit(
    stream: &[u8],
    ownership_plan: &OwnershipPlan,
    partitions: &[BlockPartition],
) -> Result<ShortBlockValidationReport, String> {
    let plan = resolve_short_block_plan(partitions)
        .map_err(|error| format!("invalid short multi-dependent plan: {error}"))?;
    if classify_block_partition(partitions) != BlockPartitionStatus::PublicLegal {
        return Err(
            "short multi-dependent validator requires a publicly closed partition".to_owned(),
        );
    }
    if partitions.contains(&BlockPartition::Six) {
        return Err("short multi-dependent validator requires only 1/2/3-block frames".to_owned());
    }
    validate_ownership_plan(ownership_plan)
        .map_err(|error| format!("invalid short multi-dependent ownership plan: {error}"))?;
    let component_count = ownership_plan.dependents.len().saturating_add(1);
    let expected_frames = partitions
        .len()
        .checked_mul(component_count)
        .ok_or_else(|| "short multi-dependent frame count overflow".to_owned())?;
    let mut frames = Vec::with_capacity(expected_frames);
    let mut offset = 0_usize;
    while offset < stream.len() {
        let frame = parse_structural_frame(stream, offset)?;
        let frame_end = offset
            .checked_add(frame.frame_size)
            .ok_or_else(|| "short multi-dependent frame offset overflow".to_owned())?;
        crate::eac3::validate_crc2(
            stream
                .get(offset..frame_end)
                .ok_or_else(|| "short multi-dependent frame exceeds stream bounds".to_owned())?,
        )?;
        offset = frame_end;
        frames.push(frame);
    }
    if offset != stream.len() || frames.len() != expected_frames {
        return Err(format!(
            "short multi-dependent access unit contains {} frames; expected {expected_frames}",
            frames.len()
        ));
    }
    let mut frame_index = 0_usize;
    let mut lfe_present = false;
    let mut complexity_index = 0;
    for (partition_index, partition) in partitions.iter().copied().enumerate() {
        let independent = frames
            .get(frame_index)
            .ok_or_else(|| "short multi-dependent I0 frame is missing".to_owned())?;
        frame_index += 1;
        if independent.stream_type != 0
            || independent.substream_id != 0
            || independent.sample_rate != 48_000
            || independent.numblkscod != partition.numblkscod()
            || independent.audio_blocks != partition.audio_blocks()
            || independent.audio_coding_mode != 7
            || independent.channel_map.is_some()
            || independent.lfe_on != ownership_plan.independent_lfe
            || independent.addbsi.is_some()
            || independent.emdf.is_some()
            || independent.convsync != Some(plan.convsync()[partition_index])
        {
            return Err(format!(
                "invalid short multi-dependent I0 at syncframe group {partition_index}"
            ));
        }
        lfe_present |= independent.lfe_on;
        for (dependent_index, dependent_plan) in ownership_plan.dependents.iter().enumerate() {
            let dependent = frames.get(frame_index).ok_or_else(|| {
                format!(
                    "short multi-dependent D{} frame is missing at group {partition_index}",
                    dependent_plan.id
                )
            })?;
            frame_index += 1;
            let final_dependent = partition_index + 1 == partitions.len()
                && dependent_index + 1 == ownership_plan.dependents.len();
            let expected_full_band_channels = dependent_plan.full_band_channel_count();
            let expected_acmod = match expected_full_band_channels {
                1 => 1,
                2 => 2,
                _ => 7,
            };
            if dependent.stream_type != 1
                || dependent.substream_id != dependent_plan.id
                || dependent.sample_rate != 48_000
                || dependent.numblkscod != partition.numblkscod()
                || dependent.audio_blocks != partition.audio_blocks()
                || dependent.audio_coding_mode != expected_acmod
                || dependent.channel_map != Some(dependent_plan.channel_map)
                || dependent.full_bandwidth_channels != expected_full_band_channels
                || dependent.lfe_on != dependent_plan.has_lfe()
                || dependent.convsync.is_some()
                || dependent.addbsi.is_some() != final_dependent
                || dependent.emdf.is_some() != final_dependent
            {
                return Err(format!(
                    "invalid short multi-dependent D{} at syncframe group {partition_index}",
                    dependent_plan.id
                ));
            }
            if final_dependent {
                if dependent
                    .addbsi
                    .as_ref()
                    .is_none_or(|addbsi| !addbsi.type_a || addbsi.complexity_index == 0)
                {
                    return Err(
                        "short multi-dependent final D0..D7 has invalid JOC addbsi".to_owned()
                    );
                }
                complexity_index = dependent
                    .addbsi
                    .as_ref()
                    .map_or(0, |addbsi| addbsi.complexity_index);
                if dependent.emdf.as_ref().is_none_or(|emdf| {
                    emdf.payload_ids != vec![11, 14]
                        || emdf.joc_downmix_index
                            != profile_spec(ownership_plan.profile).downmix_index
                }) {
                    return Err(
                        "short multi-dependent final D0..D7 has invalid JOC EMDF profile"
                            .to_owned(),
                    );
                }
            }
            lfe_present |= dependent.lfe_on;
        }
    }
    Ok(ShortBlockValidationReport {
        syncframes: frames.len(),
        audio_blocks: plan.audio_blocks(),
        samples: plan.samples(),
        metadata_owner_frame: expected_frames - 1,
        lfe_present,
        complexity_index,
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
    if dependent_count > semantic_maximum(profile) {
        return Err(format!(
            "multi-dependent access unit exceeds the {} canonical semantic maximum of {}",
            profile.semantic_name(),
            semantic_maximum(profile)
        ));
    }
    let mut dependent_ids = Vec::with_capacity(dependent_count);
    let mut dependent_channel_maps = Vec::with_capacity(dependent_count);
    let mut owned = Vec::new();
    let mut dependent_lfe_seen = false;
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
        if tokens.len() != frame.full_bandwidth_channels + usize::from(frame.lfe_on) {
            return Err(format!(
                "dependent D{} chanmap carries {} channels but acmod/lfeon declare {}",
                frame.substream_id,
                tokens.len(),
                frame.full_bandwidth_channels + usize::from(frame.lfe_on)
            ));
        }
        if tokens.contains(&100) {
            return Err(format!(
                "dependent D{} claims unsupported LFE2 ownership",
                frame.substream_id
            ));
        }
        if frame.lfe_on != tokens.contains(&101) {
            return Err(format!(
                "dependent D{} lfeon does not match its custom channel map",
                frame.substream_id
            ));
        }
        if frame.lfe_on {
            if dependent_lfe_seen {
                return Err("multiple dependents claim LFE ownership".to_owned());
            }
            dependent_lfe_seen = true;
        }
        if tokens
            .iter()
            .filter(|token| **token < 100)
            .any(|token| owned.contains(token))
        {
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
        owned.extend(tokens.iter().filter(|token| **token < 100).copied());
        for token in tokens.into_iter().filter(|token| *token < 100) {
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

    let lfe_owner = if dependent_lfe_seen {
        dependent_ids
            .iter()
            .zip(frames.iter().skip(1))
            .find(|(_, frame)| frame.lfe_on)
            .map(|(id, _)| ProgrammeOwner::Dependent(*id))
    } else if independent.lfe_on {
        Some(ProgrammeOwner::Independent)
    } else {
        None
    };
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
        lfe_present: independent.lfe_on || dependent_lfe_seen,
        lfe_owner,
        semantic_maximum: semantic_maximum(profile),
        unique_dependent_channel_ownership: true,
        ownership_census,
    })
}

pub(crate) fn validate_source_dependent_ownership_plan(
    plan: &OwnershipPlan,
    dependent_full_band_counts: &[usize],
    dependent_lfe_present: &[bool],
) -> Result<(), String> {
    let report = validate_ownership_plan(plan).map_err(|error| error.to_string())?;
    if plan.dependents.len() != dependent_full_band_counts.len() {
        return Err("multi-dependent channel maps must match carrier count".to_owned());
    }
    if !dependent_lfe_present.is_empty() && plan.dependents.len() != dependent_lfe_present.len() {
        return Err("dependent LFE carriers must match dependent count".to_owned());
    }
    for (index, dependent) in plan.dependents.iter().enumerate() {
        let full_band_count = dependent_full_band_counts[index];
        if !(1..=2).contains(&full_band_count) {
            return Err(format!(
                "dependent D{index} requires one or two source-derived full-band channels"
            ));
        }
        if dependent.full_band_channel_count() != full_band_count {
            return Err(format!(
                "dependent D{index} chanmap carries {} full-band channels but the source plan carries {full_band_count}",
                dependent.full_band_channel_count()
            ));
        }
        if !dependent_lfe_present.is_empty() && dependent.has_lfe() != dependent_lfe_present[index]
        {
            return Err(format!(
                "dependent D{index} LFE ownership does not match its source carrier"
            ));
        }
    }
    if report.final_joc_owner != plan.final_joc_owner {
        return Err("ownership plan final JOC owner changed during validation".to_owned());
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

/// Returns the canonical maximum topology with one source-derived dependent
/// LFE supplement. The supplement replaces no I0 LFE because this plan's I0
/// intentionally has no LFE; it remains one and only one programme LFE.
pub fn canonical_dependent_lfe_plan(profile: JocProfile) -> Result<OwnershipPlan, String> {
    canonical_dependent_lfe_plan_for_count(profile, semantic_maximum(profile))
}

/// Returns a canonical no-overlap topology with one dependent LFE for any
/// useful count at or below the profile semantic maximum.
pub fn canonical_dependent_lfe_plan_for_count(
    profile: JocProfile,
    dependent_count: usize,
) -> Result<OwnershipPlan, String> {
    let mut maps = canonical_dependent_channel_maps(profile, dependent_count)?;
    let first = maps
        .first_mut()
        .ok_or_else(|| "canonical dependent LFE plan has no dependent".to_owned())?;
    *first |= 0x0001;
    let dependents = maps
        .into_iter()
        .enumerate()
        .map(|(index, channel_map)| {
            DependentPlan::new(
                u8::try_from(index).unwrap_or(7),
                channel_map,
                if index == 0 {
                    DependentLfePolicy::Supplement
                } else {
                    DependentLfePolicy::Absent
                },
            )
        })
        .collect::<Vec<_>>();
    Ok(OwnershipPlan {
        profile,
        independent_lfe: false,
        final_joc_owner: u8::try_from(dependents.len().saturating_sub(1)).unwrap_or(7),
        dependents,
    })
}

#[derive(Clone, Debug)]
struct StructuralFrame {
    frame_size: usize,
    stream_type: u8,
    substream_id: u8,
    sample_rate: u32,
    numblkscod: u8,
    audio_blocks: usize,
    convsync: Option<bool>,
    audio_coding_mode: u8,
    lfe_on: bool,
    full_bandwidth_channels: usize,
    channel_map: Option<u16>,
    audio_syntax_start_bit: usize,
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
    codec_data_present: Vec<bool>,
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
    let convsync = if stream_type == 0 && numblkscod != 3 {
        Some(reader.read(1)? != 0)
    } else {
        None
    };
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
    let audio_syntax_start_bit = reader.position;
    let emdf = parse_emdf(frame)
        .map_err(|error| format!("frame at byte offset {offset} EMDF: {error}"))?;
    Ok(StructuralFrame {
        frame_size,
        stream_type,
        substream_id,
        sample_rate,
        numblkscod,
        audio_blocks,
        convsync,
        audio_coding_mode,
        lfe_on,
        full_bandwidth_channels,
        channel_map,
        audio_syntax_start_bit,
        addbsi,
        emdf,
    })
}

fn short_skip_field_enabled(frame: &StructuralFrame, bytes: &[u8]) -> Result<bool, String> {
    let mut reader = StructuralBitReader {
        bytes,
        position: frame.audio_syntax_start_bit,
    };
    if frame.numblkscod == 3 {
        reader.read(2)?; // exponent strategy exists and AHT exists
    }
    reader.read(2)?;
    reader.read(1)?;
    reader.read(5)?;
    Ok(reader.read(1)? != 0)
}

fn embedded_emdf(
    frame: &[u8],
    expected: &[u8],
) -> Result<Option<(EmdfSummary, usize, Vec<u8>)>, String> {
    let mut found = None;
    let frame_bits = frame
        .len()
        .checked_mul(8)
        .ok_or_else(|| "short skip-field frame bit length overflows".to_owned())?;
    for index in 0..frame_bits.saturating_sub(32) {
        if read_bits_at(frame, index, 16)? != 0x5838 {
            continue;
        }
        let body_len = usize::try_from(read_bits_at(frame, index + 16, 16)?)
            .map_err(|_| "short skip-field EMDF length overflows usize".to_owned())?;
        let end = index
            .checked_add(32)
            .and_then(|value| value.checked_add(body_len.saturating_mul(8)))
            .ok_or_else(|| "short skip-field EMDF length overflows frame".to_owned())?;
        if end > frame_bits {
            continue;
        }
        let body = read_bytes_at(frame, index + 32, body_len)?;
        if let Ok(summary) = parse_emdf_body(&body) {
            if body != expected {
                continue;
            }
            let skip_start = index.saturating_sub(10);
            if index < 10
                || read_bits_at(frame, skip_start, 1)? != 1
                || read_bits_at(frame, skip_start + 1, 9)?
                    != u32::try_from(body_len + 4).unwrap_or(u32::MAX)
            {
                continue;
            }
            found.get_or_insert((summary, index, body));
        }
    }
    Ok(found)
}

fn read_bits_at(bytes: &[u8], position: usize, width: usize) -> Result<u32, String> {
    let end = position
        .checked_add(width)
        .ok_or_else(|| "short skip-field bit position overflows".to_owned())?;
    if end > bytes.len().saturating_mul(8) {
        return Err("short skip-field bit read exceeds frame".to_owned());
    }
    let mut value = 0_u32;
    for bit in position..end {
        value = (value << 1) | u32::from((bytes[bit / 8] >> (7 - bit % 8)) & 1);
    }
    Ok(value)
}

fn read_bytes_at(bytes: &[u8], position: usize, count: usize) -> Result<Vec<u8>, String> {
    (0..count)
        .map(|index| {
            u8::try_from(read_bits_at(bytes, position + index * 8, 8)?)
                .map_err(|_| "short skip-field byte value overflows".to_owned())
        })
        .collect()
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
    Ok(Some(parse_emdf_body(body)?))
}

fn parse_emdf_body(body: &[u8]) -> Result<EmdfSummary, String> {
    let mut reader = StructuralBitReader::new(body);
    let _version = reader.read(2)?;
    let _key_identifier = reader.read(3)?;
    let mut payloads = Vec::new();
    let mut codec_data_present = Vec::new();
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
        let codec_data = reader.read(1)? != 0;
        if codec_data {
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
        codec_data_present.push(codec_data);
    }
    Ok(EmdfSummary {
        payload_ids: payloads,
        joc_downmix_index: joc_downmix_index
            .ok_or_else(|| "EMDF is missing JOC payload 14".to_owned())?,
        codec_data_present,
    })
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

/// Returns the public Table E.1.4 role names in coded-channel order.
pub fn channel_map_role_labels(channel_map: u16) -> Result<Vec<&'static str>, String> {
    channel_map_tokens(channel_map).map(|tokens| {
        tokens
            .into_iter()
            .map(|token| match token {
                0 => "Left",
                1 => "Centre",
                2 => "Right",
                3 => "Left Surround",
                4 => "Right Surround",
                5 => "Lrs",
                6 => "Rrs",
                7 => "Vhl",
                8 => "Vhr",
                9 => "Lc",
                10 => "Rc",
                11 => "Cs",
                12 => "Ts",
                13 => "Lsd",
                14 => "Rsd",
                15 => "Lw",
                16 => "Rw",
                17 => "Vhc",
                18 => "Lts",
                19 => "Rts",
                100 => "LFE2",
                101 => "LFE",
                _ => "unknown",
            })
            .collect()
    })
}

const CANONICAL_SINGLE_ROLE_MAPS: &[u16] = &[0x8000, 0x4000, 0x2000, 0x1000, 0x0800];
const IDX1_CANONICAL_ROLE_GROUPS: &[u16] = &[0x8000, 0x4000, 0x2000, 0x1000, 0x0800, 0x0200];
const IDX2_CANONICAL_ROLE_GROUPS: &[u16] = &[0x8000, 0x4000, 0x2000, 0x1000, 0x0800, 0x0010];

fn canonical_role_group_masks(profile: JocProfile) -> &'static [u16] {
    match profile {
        JocProfile::Idx0 | JocProfile::Idx3 => CANONICAL_SINGLE_ROLE_MAPS,
        JocProfile::Idx1 => IDX1_CANONICAL_ROLE_GROUPS,
        JocProfile::Idx2 | JocProfile::Idx4 => IDX2_CANONICAL_ROLE_GROUPS,
    }
}

/// Returns the useful dependent maximum derived from the canonical public
/// Table E.1.4 role groups for the selected Table-47 profile.
#[must_use]
pub fn semantic_maximum(profile: JocProfile) -> usize {
    canonical_role_group_masks(profile).len()
}

/// Classifies a requested dependent count without conflating syntax and
/// canonical programme ownership.
#[must_use]
pub fn dependent_topology_verdict(
    profile: JocProfile,
    dependent_count: usize,
) -> CapabilityVerdict {
    if !(1..=8).contains(&dependent_count) {
        CapabilityVerdict::Invalid
    } else if dependent_count > semantic_maximum(profile) {
        CapabilityVerdict::SemanticCeiling
    } else {
        CapabilityVerdict::SupportedPass
    }
}

/// Validates one immutable programme ownership plan before serialization.
pub fn validate_ownership_plan(
    plan: &OwnershipPlan,
) -> Result<OwnershipValidationReport, OwnershipPlanError> {
    if plan.dependents.is_empty() {
        return Err(OwnershipPlanError::EmptyPlan);
    }
    if plan.dependents.len() > 8 {
        return Err(OwnershipPlanError::TooManyDependents {
            actual: plan.dependents.len(),
        });
    }
    let maximum = semantic_maximum(plan.profile);
    // Dependent maps intentionally cover a selected replacement/supplement
    // subset of the profile roles; the validator rejects unknown or duplicate
    // roles but does not require every profile channel to appear in every plan.
    let expected_full_band = match plan.profile {
        JocProfile::Idx0 | JocProfile::Idx3 => vec![0, 1, 2, 3, 4],
        JocProfile::Idx1 => vec![0, 1, 2, 3, 4, 5, 6],
        JocProfile::Idx2 | JocProfile::Idx4 => vec![0, 1, 2, 3, 4, 7, 8],
    };
    let mut dependent_full_band = Vec::new();
    let mut lfe_owner = plan.independent_lfe.then_some(ProgrammeOwner::Independent);
    for (index, dependent) in plan.dependents.iter().enumerate() {
        let expected_id = u8::try_from(index).unwrap_or(7);
        if dependent.id != expected_id {
            return Err(OwnershipPlanError::NonSequentialId {
                expected: expected_id,
                actual: dependent.id,
            });
        }
        let tokens = channel_map_tokens(dependent.channel_map)
            .map_err(|_| OwnershipPlanError::EmptyChannelMap { id: dependent.id })?;
        if !tokens.iter().any(|token| *token < 100) {
            return Err(OwnershipPlanError::NoFullBandChannel { id: dependent.id });
        }
        let has_lfe = tokens.contains(&101);
        if tokens.contains(&100) {
            return Err(OwnershipPlanError::UnsupportedLfe2 { id: dependent.id });
        }
        let expected_policy = match dependent.lfe_policy {
            DependentLfePolicy::Absent => false,
            DependentLfePolicy::Supplement | DependentLfePolicy::ReplaceIndependent => true,
        };
        if has_lfe != expected_policy {
            return Err(OwnershipPlanError::LfePolicyConflict);
        }
        if has_lfe {
            if lfe_owner.is_some() && !plan.independent_lfe {
                return Err(OwnershipPlanError::MultipleDependentLfe);
            }
            if plan.independent_lfe
                && dependent.lfe_policy != DependentLfePolicy::ReplaceIndependent
            {
                return Err(OwnershipPlanError::LfePolicyConflict);
            }
            if !plan.independent_lfe && dependent.lfe_policy != DependentLfePolicy::Supplement {
                return Err(OwnershipPlanError::LfePolicyConflict);
            }
            if lfe_owner.is_some_and(|owner| matches!(owner, ProgrammeOwner::Dependent(_))) {
                return Err(OwnershipPlanError::MultipleDependentLfe);
            }
            lfe_owner = Some(ProgrammeOwner::Dependent(dependent.id));
        }
        for token in tokens.into_iter().filter(|token| *token < 100) {
            if !expected_full_band.contains(&token) {
                return Err(OwnershipPlanError::UnexpectedChannel {
                    id: dependent.id,
                    token,
                });
            }
            if dependent_full_band.contains(&token) {
                return Err(OwnershipPlanError::DuplicateDependentChannel { id: dependent.id });
            }
            dependent_full_band.push(token);
        }
    }
    let expected_final_owner = plan.dependents.last().map_or(0, |dependent| dependent.id);
    if plan.final_joc_owner != expected_final_owner {
        return Err(OwnershipPlanError::FinalJocOwnerMismatch {
            expected: expected_final_owner,
            actual: plan.final_joc_owner,
        });
    }
    if plan.dependents.len() > maximum {
        return Err(OwnershipPlanError::SemanticCeiling {
            actual: plan.dependents.len(),
            maximum,
        });
    }
    Ok(OwnershipValidationReport {
        lfe_owner,
        full_band_channel_roles: expected_full_band,
        semantic_maximum: maximum,
        final_joc_owner: expected_final_owner,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        CapabilityVerdict, DependentLfePolicy, DependentPlan, JocProfile, OwnershipPlan,
        OwnershipPlanError, canonical_dependent_channel_maps, dependent_topology_verdict,
        semantic_maximum, validate_ownership_plan,
    };

    fn idx0_full_plan(lfe_policy: DependentLfePolicy) -> OwnershipPlan {
        let first_map = if lfe_policy == DependentLfePolicy::Absent {
            0x4000
        } else {
            0x4001
        };
        OwnershipPlan {
            profile: JocProfile::Idx0,
            independent_lfe: matches!(lfe_policy, DependentLfePolicy::ReplaceIndependent),
            dependents: vec![
                DependentPlan::new(0, first_map, lfe_policy),
                DependentPlan::new(1, 0x8000, DependentLfePolicy::Absent),
                DependentPlan::new(2, 0x2000, DependentLfePolicy::Absent),
                DependentPlan::new(3, 0x1000, DependentLfePolicy::Absent),
                DependentPlan::new(4, 0x0800, DependentLfePolicy::Absent),
            ],
            final_joc_owner: 4,
        }
    }

    #[test]
    fn dependent_lfe_replacement_is_one_explicit_programme_owner() {
        let report =
            validate_ownership_plan(&idx0_full_plan(DependentLfePolicy::ReplaceIndependent))
                .expect("dependent LFE replacement plan");

        assert_eq!(report.lfe_owner, Some(super::ProgrammeOwner::Dependent(0)));
        assert_eq!(report.semantic_maximum, 5);
    }

    #[test]
    fn dependent_lfe_supplement_is_valid_when_i0_has_no_lfe() {
        let report = validate_ownership_plan(&idx0_full_plan(DependentLfePolicy::Supplement))
            .expect("dependent LFE supplement plan");

        assert_eq!(report.lfe_owner, Some(super::ProgrammeOwner::Dependent(0)));
        assert!(report.full_band_channel_roles.contains(&2));
    }

    #[test]
    fn second_dependent_lfe_is_rejected() {
        let mut plan = idx0_full_plan(DependentLfePolicy::Supplement);
        plan.dependents[1] = DependentPlan::new(1, 0x2001, DependentLfePolicy::Supplement);

        assert_eq!(
            validate_ownership_plan(&plan),
            Err(OwnershipPlanError::MultipleDependentLfe)
        );
    }

    #[test]
    fn lfe_policy_must_match_independent_lfe_state() {
        let mut plan = idx0_full_plan(DependentLfePolicy::Supplement);
        plan.independent_lfe = true;

        assert_eq!(
            validate_ownership_plan(&plan),
            Err(OwnershipPlanError::LfePolicyConflict)
        );
    }

    #[test]
    fn d6_and_d7_are_first_class_semantic_ceiling_verdicts() {
        for profile in [
            JocProfile::Idx0,
            JocProfile::Idx1,
            JocProfile::Idx2,
            JocProfile::Idx3,
            JocProfile::Idx4,
        ] {
            let maximum = super::semantic_maximum(profile);
            assert_eq!(
                dependent_topology_verdict(profile, maximum + 1),
                CapabilityVerdict::SemanticCeiling
            );
            assert_eq!(
                dependent_topology_verdict(profile, maximum + 2),
                CapabilityVerdict::SemanticCeiling
            );
        }
    }

    #[test]
    fn empty_dependents_are_not_a_semantic_plan() {
        let mut plan = idx0_full_plan(DependentLfePolicy::Absent);
        plan.dependents[0] = DependentPlan::new(0, 0, DependentLfePolicy::Absent);

        assert_eq!(
            validate_ownership_plan(&plan),
            Err(OwnershipPlanError::EmptyChannelMap { id: 0 })
        );
    }

    #[test]
    fn duplicate_channel_and_id_gap_plans_fail_before_serialization() {
        let mut duplicate = idx0_full_plan(DependentLfePolicy::Absent);
        duplicate.dependents[1] = DependentPlan::new(1, 0x4000, DependentLfePolicy::Absent);
        assert_eq!(
            validate_ownership_plan(&duplicate),
            Err(OwnershipPlanError::DuplicateDependentChannel { id: 1 })
        );

        let mut gap = idx0_full_plan(DependentLfePolicy::Absent);
        gap.dependents[2].id = 3;
        assert_eq!(
            validate_ownership_plan(&gap),
            Err(OwnershipPlanError::NonSequentialId {
                expected: 2,
                actual: 3,
            })
        );
    }

    #[test]
    fn lfe2_and_semantic_ceiling_are_fail_closed() {
        let mut lfe2 = idx0_full_plan(DependentLfePolicy::Absent);
        lfe2.dependents[0] = DependentPlan::new(0, 0x4002, DependentLfePolicy::Absent);
        assert_eq!(
            validate_ownership_plan(&lfe2),
            Err(OwnershipPlanError::UnsupportedLfe2 { id: 0 })
        );

        assert_eq!(
            dependent_topology_verdict(JocProfile::Idx0, 6),
            CapabilityVerdict::SemanticCeiling
        );

        let mut lfe_only = idx0_full_plan(DependentLfePolicy::Absent);
        lfe_only.dependents[0] = DependentPlan::new(0, 0x0001, DependentLfePolicy::Supplement);
        assert_eq!(
            validate_ownership_plan(&lfe_only),
            Err(OwnershipPlanError::NoFullBandChannel { id: 0 })
        );

        let mut wrong_owner = idx0_full_plan(DependentLfePolicy::Absent);
        wrong_owner.final_joc_owner = 3;
        assert_eq!(
            validate_ownership_plan(&wrong_owner),
            Err(OwnershipPlanError::FinalJocOwnerMismatch {
                expected: 4,
                actual: 3,
            })
        );
    }

    #[test]
    fn d6_and_d7_malformed_candidates_are_rejected_independently_of_the_ceiling() {
        for profile in [
            JocProfile::Idx0,
            JocProfile::Idx1,
            JocProfile::Idx2,
            JocProfile::Idx3,
            JocProfile::Idx4,
        ] {
            let maximum = semantic_maximum(profile);
            let base = canonical_dependent_channel_maps(profile, maximum)
                .expect("canonical maximum maps")
                .into_iter()
                .enumerate()
                .map(|(id, channel_map)| {
                    DependentPlan::new(
                        u8::try_from(id).unwrap_or(7),
                        channel_map,
                        DependentLfePolicy::Absent,
                    )
                })
                .collect::<Vec<_>>();
            for extra_count in 1..=2 {
                let extra_id = u8::try_from(maximum).unwrap_or(7);
                let final_extra_id = u8::try_from(maximum + extra_count - 1).unwrap_or(7);
                let mut empty = OwnershipPlan {
                    profile,
                    independent_lfe: false,
                    dependents: base.clone(),
                    final_joc_owner: final_extra_id,
                };
                for id in extra_id..=final_extra_id {
                    empty
                        .dependents
                        .push(DependentPlan::new(id, 0, DependentLfePolicy::Absent));
                }
                assert_eq!(
                    validate_ownership_plan(&empty),
                    Err(OwnershipPlanError::EmptyChannelMap { id: extra_id })
                );

                let mut duplicate = OwnershipPlan {
                    profile,
                    independent_lfe: false,
                    dependents: base.clone(),
                    final_joc_owner: final_extra_id,
                };
                for id in extra_id..=final_extra_id {
                    duplicate.dependents.push(DependentPlan::new(
                        id,
                        base[0].channel_map,
                        DependentLfePolicy::Absent,
                    ));
                }
                assert_eq!(
                    validate_ownership_plan(&duplicate),
                    Err(OwnershipPlanError::DuplicateDependentChannel { id: extra_id })
                );

                let mut invalid_map = OwnershipPlan {
                    profile,
                    independent_lfe: false,
                    dependents: base.clone(),
                    final_joc_owner: final_extra_id,
                };
                for id in extra_id..=final_extra_id {
                    invalid_map.dependents.push(DependentPlan::new(
                        id,
                        0x4002,
                        DependentLfePolicy::Absent,
                    ));
                }
                assert_eq!(
                    validate_ownership_plan(&invalid_map),
                    Err(OwnershipPlanError::UnsupportedLfe2 { id: extra_id })
                );
            }
        }
    }
}
