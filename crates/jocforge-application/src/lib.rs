// pattern: Functional Core
//! Application boundary. It currently rejects every encoding request without effects.

use jocforge_domain::state::EncodeError;

/// Rejects encoding until the architecture and implementation have been reviewed.
pub const fn encode() -> Result<(), EncodeError> {
    Err(EncodeError::BackendUnavailable)
}
