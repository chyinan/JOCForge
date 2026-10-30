// pattern: Functional Core
//! Pure typed clock-domain conversions for the approved CLOCK-FOUNDATION slice.

mod clock;
mod index;
mod source_time_range;

pub use clock::{
    ClockError, ProgrammeCounts, PublicationDisposition, PublishedConsumption, programme_counts,
    publication_for_source, source_for_callback,
};
pub use index::{
    AdmSampleCount, AdmSampleIndex, ClockDomainTypeSeparation, CodecSourceIndex,
    FrontendCallbackIndex, JocDispatchIndex, JocGeneratedWriterIndex, JocPoppedWriterIndex,
    NormalCallbackCount, OrdinaryPayloadWriterIndex, PublishedAuIndex, ReceiverCallbackCount,
};
pub use source_time_range::{
    ActiveWriterSampleIndex, SelectedFilePositionRangeInput, SelectedFilePositionSampleRange,
    SelectedNumericFrameRate, SourceTimeRangeError, ValidatedSampleIndex,
    resolve_selected_file_position_sample_range,
};
