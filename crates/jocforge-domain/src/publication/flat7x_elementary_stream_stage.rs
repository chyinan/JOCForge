// pattern: Functional Core
//! Immutable Flat7x elementary-stream staging proposal; final exposure is outside this contract.

use crate::{
    codec::elementary_stream_composer::{
        Flat7xElementaryStreamExtent, measure_flat7x_elementary_stream,
    },
    profile::Flat7xI0D0TargetPlan,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xElementaryStreamStageProposalError {
    UnsupportedTargetContext,
    ExtentMismatch,
    ByteLengthMismatch { expected: u32, actual: usize },
}

#[must_use]
pub struct Flat7xElementaryStreamStageProposal<'a> {
    target: Flat7xI0D0TargetPlan,
    stream_bytes: &'a [u8],
    extent: Flat7xElementaryStreamExtent,
}

impl std::fmt::Debug for Flat7xElementaryStreamStageProposal<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Flat7xElementaryStreamStageProposal")
            .field("extent", &self.extent)
            .field("stream_byte_length", &self.stream_bytes.len())
            .field("production_allowed", &false)
            .finish()
    }
}

impl<'a> Flat7xElementaryStreamStageProposal<'a> {
    pub fn new(
        target: Flat7xI0D0TargetPlan,
        stream_bytes: &'a [u8],
        extent: Flat7xElementaryStreamExtent,
    ) -> Result<Self, Flat7xElementaryStreamStageProposalError> {
        if target != Flat7xI0D0TargetPlan::approved_contract() {
            return Err(Flat7xElementaryStreamStageProposalError::UnsupportedTargetContext);
        }
        let access_unit_count = usize::try_from(extent.access_unit_count())
            .map_err(|_| Flat7xElementaryStreamStageProposalError::ExtentMismatch)?;
        let measured = measure_flat7x_elementary_stream(target, access_unit_count)
            .map_err(|_| Flat7xElementaryStreamStageProposalError::ExtentMismatch)?;
        if measured != extent {
            return Err(Flat7xElementaryStreamStageProposalError::ExtentMismatch);
        }
        let expected_byte_length = usize::try_from(extent.byte_length())
            .map_err(|_| Flat7xElementaryStreamStageProposalError::ExtentMismatch)?;
        if stream_bytes.len() != expected_byte_length {
            return Err(
                Flat7xElementaryStreamStageProposalError::ByteLengthMismatch {
                    expected: extent.byte_length(),
                    actual: stream_bytes.len(),
                },
            );
        }

        Ok(Self {
            target,
            stream_bytes,
            extent,
        })
    }

    pub const fn target(&self) -> Flat7xI0D0TargetPlan {
        self.target
    }

    pub const fn extent(&self) -> Flat7xElementaryStreamExtent {
        self.extent
    }

    pub const fn stream_bytes(&self) -> &'a [u8] {
        self.stream_bytes
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}
