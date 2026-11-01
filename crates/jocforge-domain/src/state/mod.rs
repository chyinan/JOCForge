// pattern: Functional Core
//! Typed state contracts.

mod reader_eof;

pub use reader_eof::{
    Control38Snapshot, Host38Frame, ReaderEofDigest, ReaderEofError, ReaderEofMachine,
    ReaderEofOwnerId, ReaderEofOwnerStateReference, ReaderEofPhase, ReaderEofPrepared,
    ReaderEofReceipt, ReaderEofReplacementCandidate, ReaderEofStaged, ReaderEofState,
};

/// Production encoding cannot begin before private F15 role and destination admission exist.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// The prepared input is valid, but production provider admissions and destination authority are absent.
    ProductionPlanUnavailable,
    /// An embedded selected owner provider or destination binding failed admission.
    F15AdmissionRejected,
    /// Provider admission succeeded, but the selected source and AU execution path is not connected.
    SourcePipelineUnavailable,
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProductionPlanUnavailable => formatter.write_str(
                "production plan unavailable: F15 owner admissions and parent destination capability are not wired",
            ),
            Self::F15AdmissionRejected => formatter.write_str(
                "production plan rejected: selected F15 owner values or destination identity failed validation",
            ),
            Self::SourcePipelineUnavailable => formatter.write_str(
                "source execution unavailable: selected callback and AU publication path are not wired",
            ),
        }
    }
}

impl std::error::Error for EncodeError {}
