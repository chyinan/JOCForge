// pattern: Functional Core
//! None-DRC scalar processing leaves.

pub mod none_drc_bsi_projection;
pub mod none_drc_callback;
pub mod none_drc_dialnorm_offset;
pub mod none_drc_frame;
pub mod none_drc_normalizer;
pub mod none_drc_override_fifo;
pub(crate) mod none_drc_owner;
pub use none_drc_owner::{
    NoneDrcOwnerAfterProposal, NoneDrcOwnerError, NoneDrcOwnerRoot, NoneDrcOwnerState,
};
pub mod none_drc_quantizer;
pub mod none_drc_scheduler;
pub mod none_drc_statistics;
pub mod secondary_source_owner;
pub use secondary_source_owner::{NoneDrcSourceFrameProposal, SecondarySourceError};
