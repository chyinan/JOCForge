// pattern: Functional Core
//! Typed selected-profile owner for six source slots and independent transform histories.

use super::long_transform::{LongTransformHistory, TransformBranch};

/// Number of selected construction source slots.
pub const TRANSFORM_SOURCE_SLOT_COUNT: usize = 6;

/// Number of F32 words in each primary or companion history bank.
pub const TRANSFORM_HISTORY_WORDS_PER_BANK: usize = 128;

/// Logical selected source-slot order. Full-band indices are opaque positions here;
/// slot 5 is the LFE boundary. This does not map external speaker lanes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransformSourceSlot {
    FullBand0,
    FullBand1,
    FullBand2,
    FullBand3,
    FullBand4,
    LfeBoundary,
}

impl TransformSourceSlot {
    pub const ALL: [Self; TRANSFORM_SOURCE_SLOT_COUNT] = [
        Self::FullBand0,
        Self::FullBand1,
        Self::FullBand2,
        Self::FullBand3,
        Self::FullBand4,
        Self::LfeBoundary,
    ];

    pub const fn source_index(self) -> u8 {
        match self {
            Self::FullBand0 => 0,
            Self::FullBand1 => 1,
            Self::FullBand2 => 2,
            Self::FullBand3 => 3,
            Self::FullBand4 => 4,
            Self::LfeBoundary => 5,
        }
    }

    const fn array_index(self) -> usize {
        self.source_index() as usize
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceTransformHistories {
    primary: LongTransformHistory,
    companion: LongTransformHistory,
}

/// Model-owned history values arranged by source slot and transform branch.
///
/// This type carries no native ABI, pointer, reset, EOF, fatal, or absolute sample-clock
/// behavior. Its zeroed constructor is the observed selected-profile initial history value,
/// not a claim about a product owner lifecycle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransformHistoryOwner {
    slots: [SourceTransformHistories; TRANSFORM_SOURCE_SLOT_COUNT],
}

impl TransformHistoryOwner {
    pub fn zeroed() -> Self {
        Self {
            slots: std::array::from_fn(|_| SourceTransformHistories {
                primary: LongTransformHistory::zeroed(),
                companion: LongTransformHistory::zeroed(),
            }),
        }
    }

    pub fn history(
        &self,
        source: TransformSourceSlot,
        branch: TransformBranch,
    ) -> &LongTransformHistory {
        match branch {
            TransformBranch::Primary => &self.slots[source.array_index()].primary,
            TransformBranch::Companion => &self.slots[source.array_index()].companion,
        }
    }

    /// Stores exactly one returned branch history in exactly one source slot.
    pub fn store_history_after(
        &mut self,
        source: TransformSourceSlot,
        branch: TransformBranch,
        history_after: &LongTransformHistory,
    ) {
        match branch {
            TransformBranch::Primary => {
                self.slots[source.array_index()].primary = history_after.clone();
            }
            TransformBranch::Companion => {
                self.slots[source.array_index()].companion = history_after.clone();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TransformSourceSlot;
    use crate::transform::parameters::WINDOW_F32_BITS;
    use serde_json::Value;
    use sha2::{Digest, Sha256};
    use std::path::PathBuf;

    #[test]
    fn history_parameter_window_matches_shared_hash_bound_transform_table() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/transform_history_owner/PARAMETERS.json");
        let parameters: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let words = parameters["window_f32_bits"].as_array().unwrap();
        assert_eq!(words.len(), WINDOW_F32_BITS.len());
        for (actual, expected) in WINDOW_F32_BITS.iter().zip(words) {
            assert_eq!(*actual, u32::try_from(expected.as_u64().unwrap()).unwrap());
        }

        let bytes = WINDOW_F32_BITS
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect::<Vec<_>>();
        let digest = format!("{:x}", Sha256::digest(bytes));
        assert_eq!(
            parameters["window_sha256"].as_str().unwrap(),
            "097ea95140024fd143ed18a78299fa72d69bce1c8e0d356aed2efff2a6a099a2"
        );
        assert_eq!(parameters["window_sha256"].as_str().unwrap(), digest);
        assert_eq!(
            TransformSourceSlot::ALL.map(TransformSourceSlot::source_index),
            [0, 1, 2, 3, 4, 5]
        );
    }
}
