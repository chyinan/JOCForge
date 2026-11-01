// pattern: Imperative Shell
//! Reads one caller-selected RIFF resource into the allocation used by input admission.

use std::{io, path::Path, sync::Arc};

#[derive(Debug)]
pub struct RiffPcm24FileReadError {
    source: io::Error,
}

impl std::fmt::Display for RiffPcm24FileReadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "RIFF_PCM24_INPUT_READ: {}", self.source)
    }
}

impl std::error::Error for RiffPcm24FileReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Loads the selected BWF bytes once. The PCM parser and application validate the shared bytes.
pub fn read_resource(path: &Path) -> Result<Arc<[u8]>, RiffPcm24FileReadError> {
    let bytes = std::fs::read(path).map_err(|source| RiffPcm24FileReadError { source })?;
    Ok(Arc::from(bytes.into_boxed_slice()))
}
