// pattern: Functional Core
//! Public keyless integrity calculations and a bounded programme-local owner.
//! The owner tracks clocks and completion, but keeps no checksum accumulator.

pub mod crc16;
pub mod hmac_sha256;
pub mod keyless_checksum;
pub mod programme_owner;
