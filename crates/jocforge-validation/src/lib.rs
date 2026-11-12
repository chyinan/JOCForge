// pattern: Imperative Shell
//! Independent validation adapters are reserved, with no historical gates or fixtures.
//! External tools may decode or compare only; they cannot serve production encoding.

pub mod ac3_audio_block_parser;
pub mod channel_map_validator;
pub mod eac3_parser;
pub mod frame_report;
pub mod topology_validator;
pub mod w04;
