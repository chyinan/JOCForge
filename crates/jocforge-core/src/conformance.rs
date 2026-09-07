// pattern: Functional Core

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
