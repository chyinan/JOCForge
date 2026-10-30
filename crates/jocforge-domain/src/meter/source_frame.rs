// pattern: Functional Core
//! Immutable typed physical PCM callback values crossing from IO to application.

use super::sample_block_queue::{
    MeterCallbackCount, MeterCallbackIndex, MeterCallbackPhase, MeterSampleCount,
    MeterSourceSampleCount, MeterUpstreamGeneration,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MeterSourceIdentity([u8; 32]);

impl MeterSourceIdentity {
    pub const fn from_sha256(value: [u8; 32]) -> Self {
        Self(value)
    }

    pub const fn sha256(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MeterSourcePosition(u64);

impl MeterSourcePosition {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Count of successful normal Meter source reads. Zero-input drains do not advance it.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MeterSourceReadIndex(u64);

impl MeterSourceReadIndex {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Typed fields copied from one in-memory source-cursor proposal at the CLI boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeterSourceCallbackFrameParts<'a> {
    pub callback_index: MeterCallbackIndex,
    pub upstream_generation: MeterUpstreamGeneration,
    pub phase: MeterCallbackPhase,
    pub source_identity: MeterSourceIdentity,
    pub source_samples: MeterSourceSampleCount,
    pub normal_callbacks: MeterCallbackCount,
    pub source_cursor_before: MeterSourcePosition,
    pub source_cursor_after: MeterSourcePosition,
    pub source_read_index_before: MeterSourceReadIndex,
    pub source_read_index_after: MeterSourceReadIndex,
    pub host_input_count: MeterSampleCount,
    pub source_padding_count: MeterSampleCount,
    pub rows: &'a [Vec<u32>],
}

/// Borrowed physical-row view. The source bytes and cursor remain owned by IO.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeterSourceCallbackFrame<'a> {
    parts: MeterSourceCallbackFrameParts<'a>,
}

impl<'a> MeterSourceCallbackFrame<'a> {
    pub const fn new(parts: MeterSourceCallbackFrameParts<'a>) -> Self {
        Self { parts }
    }

    pub const fn callback_index(self) -> MeterCallbackIndex {
        self.parts.callback_index
    }

    pub const fn upstream_generation(self) -> MeterUpstreamGeneration {
        self.parts.upstream_generation
    }

    pub const fn phase(self) -> MeterCallbackPhase {
        self.parts.phase
    }

    pub const fn source_identity(self) -> MeterSourceIdentity {
        self.parts.source_identity
    }

    pub const fn source_samples(self) -> MeterSourceSampleCount {
        self.parts.source_samples
    }

    pub const fn normal_callbacks(self) -> MeterCallbackCount {
        self.parts.normal_callbacks
    }

    pub const fn source_cursor_before(self) -> MeterSourcePosition {
        self.parts.source_cursor_before
    }

    pub const fn source_cursor_after(self) -> MeterSourcePosition {
        self.parts.source_cursor_after
    }

    pub const fn source_read_index_before(self) -> MeterSourceReadIndex {
        self.parts.source_read_index_before
    }

    pub const fn source_read_index_after(self) -> MeterSourceReadIndex {
        self.parts.source_read_index_after
    }

    pub const fn host_input_count(self) -> MeterSampleCount {
        self.parts.host_input_count
    }

    pub const fn source_padding_count(self) -> MeterSampleCount {
        self.parts.source_padding_count
    }

    pub fn rows(self) -> &'a [Vec<u32>] {
        self.parts.rows
    }
}
