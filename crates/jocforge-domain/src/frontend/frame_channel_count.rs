// pattern: Functional Core
//! Static frame-channel count candidates for the inspected Atmos source-axis branch.

const STATIC_OAMDI_SOURCE_COUNT_MAX: u32 = 159;
const EXTENDED_BED_PREFIX_ROWS: u16 = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StaticOamdiSourceCount(u16);

impl StaticOamdiSourceCount {
    pub const fn value(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessFrameChannelCount(u16);

impl ProcessFrameChannelCount {
    pub const fn value(self) -> u16 {
        self.0
    }
}

/// Both statically described composite-mode counts, without selecting a runtime mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameChannelCountCandidates {
    ordinary_mode: ProcessFrameChannelCount,
    extended_mode: ProcessFrameChannelCount,
}

impl FrameChannelCountCandidates {
    pub const fn ordinary_mode(self) -> ProcessFrameChannelCount {
        self.ordinary_mode
    }

    pub const fn extended_mode(self) -> ProcessFrameChannelCount {
        self.extended_mode
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameChannelCountError {
    StaticOamdiCountOutOfRange,
    NoSourceAdapters,
    CountOverflow,
    ReferenceCountMismatch,
}

/// Applies the explicit static OAMDI source-count guard from the inspected branch.
///
/// This is a model bound only. It does not establish that a selected job reaches the branch or
/// that opaque PE/JOC internals accept every count in this range.
pub fn static_oamdi_source_count(
    physical_input_rows: u32,
) -> Result<StaticOamdiSourceCount, FrameChannelCountError> {
    if !(1..=STATIC_OAMDI_SOURCE_COUNT_MAX).contains(&physical_input_rows) {
        return Err(FrameChannelCountError::StaticOamdiCountOutOfRange);
    }
    let count = u16::try_from(physical_input_rows)
        .map_err(|_| FrameChannelCountError::StaticOamdiCountOutOfRange)?;
    Ok(StaticOamdiSourceCount(count))
}

/// Projects the two documented composite-mode candidates for the washed 36/38 vectors.
///
/// The ordinary candidate preserves the adapter's U16 count. The extended candidate uses the
/// maximum tag-255 row count across adapters plus ten bed-prefix rows. Both candidates must
/// match the supplied 36/38 physical count. This function does not choose the runtime mode or
/// equate this channel-row axis with profile code-row shapes.
pub fn project_frame_channel_count_candidates(
    physical_input_rows: u16,
    ordinary_adapter_count: u16,
    tag_255_object_rows_by_adapter: &[u16],
) -> Result<FrameChannelCountCandidates, FrameChannelCountError> {
    if !matches!(physical_input_rows, 36 | 38) || ordinary_adapter_count != physical_input_rows {
        return Err(FrameChannelCountError::ReferenceCountMismatch);
    }
    let maximum_object_rows = tag_255_object_rows_by_adapter
        .iter()
        .copied()
        .max()
        .ok_or(FrameChannelCountError::NoSourceAdapters)?;
    let extended_count = maximum_object_rows
        .checked_add(EXTENDED_BED_PREFIX_ROWS)
        .ok_or(FrameChannelCountError::CountOverflow)?;
    if extended_count != physical_input_rows {
        return Err(FrameChannelCountError::ReferenceCountMismatch);
    }
    Ok(FrameChannelCountCandidates {
        ordinary_mode: ProcessFrameChannelCount(ordinary_adapter_count),
        extended_mode: ProcessFrameChannelCount(extended_count),
    })
}
