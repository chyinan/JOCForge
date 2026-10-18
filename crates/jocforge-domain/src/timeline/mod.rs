// pattern: Functional Core
//! Pure typed clock-domain conversions for the approved CLOCK-FOUNDATION slice.

mod clock;
mod index;

pub use clock::{
    ClockError, ProgrammeCounts, PublicationDisposition, PublishedConsumption, programme_counts,
    publication_for_source, source_for_callback,
};
pub use index::{
    AdmSampleCount, AdmSampleIndex, ClockDomainTypeSeparation, CodecSourceIndex,
    FrontendCallbackIndex, JocDispatchIndex, JocGeneratedWriterIndex, JocPoppedWriterIndex,
    NormalCallbackCount, OrdinaryPayloadWriterIndex, PublishedAuIndex, ReceiverCallbackCount,
};
