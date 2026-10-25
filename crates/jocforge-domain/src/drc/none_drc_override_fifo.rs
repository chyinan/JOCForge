// pattern: Functional Core
//! Selected zero-record F06ND metadata override FIFO transition.

use crate::timeline::CodecSourceIndex;

const LOGICAL_RECORD_COUNT: usize = 4;
const METADATA_DELAY_SAMPLES: u32 = 3_072;
const APPEND_SLOT: u8 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZeroLogicalRecord;

#[derive(Debug, Eq, PartialEq)]
pub struct NoneDrcOverrideFifoState {
    records: [ZeroLogicalRecord; LOGICAL_RECORD_COUNT],
    frames_processed: u64,
}

impl NoneDrcOverrideFifoState {
    pub const fn cold() -> Self {
        Self {
            records: [ZeroLogicalRecord; LOGICAL_RECORD_COUNT],
            frames_processed: 0,
        }
    }

    pub const fn records(&self) -> &[ZeroLogicalRecord; LOGICAL_RECORD_COUNT] {
        &self.records
    }

    pub const fn frames_processed(&self) -> u64 {
        self.frames_processed
    }

    pub const fn metadata_delay_samples(&self) -> u32 {
        METADATA_DELAY_SAMPLES
    }

    pub const fn append_slot(&self) -> u8 {
        APPEND_SLOT
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoneDrcOverrideFifoError {
    SourceGenerationMismatch,
    FramesProcessedOverflow,
}

impl NoneDrcOverrideFifoError {
    pub const fn outcome_code(self) -> &'static str {
        match self {
            Self::SourceGenerationMismatch => "DRC_INPUT_ERROR",
            Self::FramesProcessedOverflow => "NUMERIC_FAILURE",
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct NoneDrcOverrideFifoTransition {
    consumed_record: ZeroLogicalRecord,
    next_state: NoneDrcOverrideFifoState,
}

impl NoneDrcOverrideFifoTransition {
    pub const fn consumed_record(&self) -> ZeroLogicalRecord {
        self.consumed_record
    }

    pub const fn next_state(&self) -> &NoneDrcOverrideFifoState {
        &self.next_state
    }

    pub fn into_parts(self) -> (ZeroLogicalRecord, NoneDrcOverrideFifoState) {
        (self.consumed_record, self.next_state)
    }
}

/// Stages the selected zero-record FIFO rule for one source generation.
///
/// The selected profile performs no DRC-control FIFO writes. This type models
/// only the four logical zero records and frame counter; record fields, byte
/// serialization, and nonzero category-5 payload behavior remain unspecified.
pub fn transition_none_drc_override_fifo(
    before: &NoneDrcOverrideFifoState,
    source_index: CodecSourceIndex,
) -> Result<NoneDrcOverrideFifoTransition, NoneDrcOverrideFifoError> {
    if before.frames_processed != source_index.value() {
        return Err(NoneDrcOverrideFifoError::SourceGenerationMismatch);
    }

    let shifted = std::array::from_fn(|slot| {
        if slot + 1 < LOGICAL_RECORD_COUNT {
            before.records[slot + 1]
        } else {
            ZeroLogicalRecord
        }
    });
    let consumed_record = shifted[0];
    let mut next_records = shifted;
    next_records[0] = ZeroLogicalRecord;
    let frames_processed = before
        .frames_processed
        .checked_add(1)
        .ok_or(NoneDrcOverrideFifoError::FramesProcessedOverflow)?;

    Ok(NoneDrcOverrideFifoTransition {
        consumed_record,
        next_state: NoneDrcOverrideFifoState {
            records: next_records,
            frames_processed,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_processed_overflow_returns_numeric_failure_without_a_transition() {
        let before = NoneDrcOverrideFifoState {
            records: [ZeroLogicalRecord; LOGICAL_RECORD_COUNT],
            frames_processed: u64::MAX,
        };
        assert_eq!(
            transition_none_drc_override_fifo(&before, CodecSourceIndex::new(u64::MAX)),
            Err(NoneDrcOverrideFifoError::FramesProcessedOverflow),
        );
        assert_eq!(before.frames_processed(), u64::MAX);
        assert_eq!(
            NoneDrcOverrideFifoError::FramesProcessedOverflow.outcome_code(),
            "NUMERIC_FAILURE",
        );
    }
}
