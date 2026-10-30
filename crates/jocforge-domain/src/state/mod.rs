// pattern: Functional Core
//! Typed state contracts.

mod reader_eof;

pub use reader_eof::{
    Control38Snapshot, Host38Frame, ReaderEofDigest, ReaderEofError, ReaderEofMachine,
    ReaderEofOwnerId, ReaderEofOwnerStateReference, ReaderEofPhase, ReaderEofPrepared,
    ReaderEofReceipt, ReaderEofReplacementCandidate, ReaderEofStaged, ReaderEofState,
};

/// An encoding request cannot run in the migration baseline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// The reviewed implementation has not been installed.
    BackendUnavailable,
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(
            "encoding backend unavailable: architecture review and implementation required",
        )
    }
}

impl std::error::Error for EncodeError {}
