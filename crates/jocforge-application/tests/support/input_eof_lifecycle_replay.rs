// pattern: Functional Core
//! Test-only replay of the washed selected external-input, flush, and EOF trace.
//!
//! This is not a production application API, PCM renderer, file reader, or body
//! producer. Fixture extents/spans are observation inputs only. API control
//! positions are not absolute ADM/WAV offsets.

use jocforge_domain::frontend::output_publication_gate::{
    CarrierPublicationGate, FrontendCallOrdinal, PublicationGateReceipt, PublishedAuOrdinal,
};
use jocforge_domain::joc::emdf_carrier::{CarrierPlanReceipt, CarrierPlannerState};

const SOURCE_CHANNELS: usize = 38;
const INITIAL_FLUSH_REMAINING: u32 = 4;
const MAX_EXTERNAL_PROCESS_CALLS: u32 = 4;
const MAX_FLUSH_CALLS: u32 = 5;
const RENDERER_THRESHOLD_SAMPLES: u32 = 3_072;
const RENDERER_HOP_SAMPLES: u32 = 1_536;
const RENDERER_PROGRESS_LIMIT: u32 = RENDERER_THRESHOLD_SAMPLES + RENDERER_HOP_SAMPLES;
const FRONTEND_INTERNAL_OUTPUT_BYTES: u32 = 4_096;
const POST_FLUSH_PROCESS_RETURN_I32: i32 = -3;
const POST_FLUSH_PROCESS_RETURN_RAW_U32: u32 = 4_294_967_293;

/// Selected-segment-relative input API position carried by control records.
///
/// It is deliberately distinct from reader cursor and selected-item source
/// sample coordinates. It is not an absolute file/sample offset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExternalControlPosition(u64);

impl ExternalControlPosition {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Source-sample coordinate passed to the selected CompositeMaster item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedItemSampleCoordinate(u64);

impl SelectedItemSampleCoordinate {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Reader cursor measured relative to the selected item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelativeReaderCursorSamples(u64);

impl RelativeReaderCursorSamples {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// The reader and external input ordinals from one capture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderCallIdentity {
    input_api_call_ordinal: u32,
    native_reader_frame_invocation: u32,
}

impl ReaderCallIdentity {
    pub const fn new(input_api_call_ordinal: u32, native_reader_frame_invocation: u32) -> Self {
        Self {
            input_api_call_ordinal,
            native_reader_frame_invocation,
        }
    }

    pub const fn input_api_call_ordinal(self) -> u32 {
        self.input_api_call_ordinal
    }

    pub const fn native_reader_frame_invocation(self) -> u32 {
        self.native_reader_frame_invocation
    }
}

/// Source-coordinate range for the one selected CompositeMaster item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedItemSampleRange {
    selection_index: u16,
    start: SelectedItemSampleCoordinate,
    end: SelectedItemSampleCoordinate,
}

impl SelectedItemSampleRange {
    pub const fn new(
        selected_item_index: u16,
        selected_item_start_sample: SelectedItemSampleCoordinate,
        selected_item_end_sample: SelectedItemSampleCoordinate,
    ) -> Self {
        Self {
            selection_index: selected_item_index,
            start: selected_item_start_sample,
            end: selected_item_end_sample,
        }
    }

    pub const fn selected_item_index(self) -> u16 {
        self.selection_index
    }

    pub const fn start_sample(self) -> SelectedItemSampleCoordinate {
        self.start
    }

    pub const fn end_sample(self) -> SelectedItemSampleCoordinate {
        self.end
    }
}

/// Relative reader cursor and matching per-record control position.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderCursorObservation {
    control_position: ExternalControlPosition,
    reader_cursor_before: RelativeReaderCursorSamples,
    reader_cursor_after: RelativeReaderCursorSamples,
}

impl ReaderCursorObservation {
    pub const fn new(
        control_position: ExternalControlPosition,
        reader_cursor_before: RelativeReaderCursorSamples,
        reader_cursor_after: RelativeReaderCursorSamples,
    ) -> Self {
        Self {
            control_position,
            reader_cursor_before,
            reader_cursor_after,
        }
    }

    pub const fn control_position(self) -> ExternalControlPosition {
        self.control_position
    }

    pub const fn before(self) -> RelativeReaderCursorSamples {
        self.reader_cursor_before
    }

    pub const fn after(self) -> RelativeReaderCursorSamples {
        self.reader_cursor_after
    }
}

/// Read request/actual lengths and the returned packet's reported sample count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderSampleCounts {
    reader_requested: u32,
    read_float_requested: u32,
    read_float_actual: u32,
    returned_packet: u32,
}

impl ReaderSampleCounts {
    pub const fn new(
        reader_requested_samples: u32,
        read_float_requested_samples: u32,
        read_float_actual_samples: u32,
        returned_packet_samples: u32,
    ) -> Self {
        Self {
            reader_requested: reader_requested_samples,
            read_float_requested: read_float_requested_samples,
            read_float_actual: read_float_actual_samples,
            returned_packet: returned_packet_samples,
        }
    }

    pub const fn reader_requested_samples(self) -> u32 {
        self.reader_requested
    }

    pub const fn read_float_requested_samples(self) -> u32 {
        self.read_float_requested
    }

    pub const fn read_float_actual_samples(self) -> u32 {
        self.read_float_actual
    }

    pub const fn returned_packet_samples(self) -> u32 {
        self.returned_packet
    }
}

/// Bounded reader/source join observation for one input API call.
///
/// The selected sample coordinate is derived from item start plus relative
/// cursor. This test-only record does not define a general seek or file-offset
/// rule and contains no PCM samples.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderSourcePositionObservation {
    identity: ReaderCallIdentity,
    selected_item: SelectedItemSampleRange,
    cursor: ReaderCursorObservation,
    sample_counts: ReaderSampleCounts,
}

impl ReaderSourcePositionObservation {
    pub const fn new(
        identity: ReaderCallIdentity,
        selected_item: SelectedItemSampleRange,
        cursor: ReaderCursorObservation,
        sample_counts: ReaderSampleCounts,
    ) -> Self {
        Self {
            identity,
            selected_item,
            cursor,
            sample_counts,
        }
    }

    pub const fn identity(self) -> ReaderCallIdentity {
        self.identity
    }

    pub const fn selected_item(self) -> SelectedItemSampleRange {
        self.selected_item
    }

    pub const fn cursor(self) -> ReaderCursorObservation {
        self.cursor
    }

    pub const fn sample_counts(self) -> ReaderSampleCounts {
        self.sample_counts
    }

    pub fn derived_selected_source_sample(self) -> Option<SelectedItemSampleCoordinate> {
        self.selected_item
            .start_sample()
            .0
            .checked_add(self.cursor.before().0)
            .map(SelectedItemSampleCoordinate)
    }
}

/// Zero-based source/channel index in the external input control records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExternalSourceIndex(u16);

impl ExternalSourceIndex {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u16 {
        self.0
    }
}

/// Renderer child output span for one work invocation, not an absolute clock.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RendererSpanSamples(u32);

impl RendererSpanSamples {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

/// One source-index/control-position pair supplied to an external process call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceControlRecord {
    source_index: ExternalSourceIndex,
    control_position: ExternalControlPosition,
}

impl SourceControlRecord {
    pub const fn new(
        source_index: ExternalSourceIndex,
        control_position: ExternalControlPosition,
    ) -> Self {
        Self {
            source_index,
            control_position,
        }
    }
}

/// Diagnostic source-stage inputs supplied by the selected washed boundary.
///
/// `pending_id14_payload_bytes` is a scalar extent only. This type carries no
/// descriptor, body bytes, parser result, or producer/authentication behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrontendSourceInput {
    planner_request_body_bytes: i32,
    pending_id14_payload_bytes: u16,
}

impl FrontendSourceInput {
    pub const fn new(planner_request_body_bytes: i32, pending_id14_payload_bytes: u16) -> Self {
        Self {
            planner_request_body_bytes,
            pending_id14_payload_bytes,
        }
    }
}

/// One of the four selected external process calls.
#[derive(Clone, Copy, Debug)]
pub struct ExternalProcessInput<'a> {
    source_records: &'a [SourceControlRecord; SOURCE_CHANNELS],
    reader_source: ReaderSourcePositionObservation,
    renderer_span: RendererSpanSamples,
    frontend_source: Option<FrontendSourceInput>,
}

impl<'a> ExternalProcessInput<'a> {
    pub const fn new(
        source_records: &'a [SourceControlRecord; SOURCE_CHANNELS],
        reader_source: ReaderSourcePositionObservation,
        renderer_span: RendererSpanSamples,
        frontend_source: Option<FrontendSourceInput>,
    ) -> Self {
        Self {
            source_records,
            reader_source,
            renderer_span,
            frontend_source,
        }
    }
}

/// One active flush work result and its already-produced diagnostic source step.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActiveFlushInput {
    renderer_span: RendererSpanSamples,
    frontend_source: FrontendSourceInput,
}

impl ActiveFlushInput {
    pub const fn new(
        renderer_span: RendererSpanSamples,
        frontend_source: FrontendSourceInput,
    ) -> Self {
        Self {
            renderer_span,
            frontend_source,
        }
    }
}

/// Local input-shape failures; these are not native status-code claims.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputLifecycleInputError {
    SourceIndexOrderMismatch,
    ControlPositionMismatch,
    ReaderCursorMismatch,
    InvalidSelectedItemRange,
    SelectedProcessCallLimit,
    SelectedFlushCallLimit,
    UnexpectedFrontendOrigin,
    MissingActiveFlushInput,
    InputProvidedForEmptyFlush,
    FlushHasNotStarted,
}

/// Test-only selected lifecycle replay state. It is not a production application
/// owner/API. Same-owner reset/recreate is intentionally absent.
#[derive(Debug, Eq, PartialEq)]
pub struct InputEofLifecycle {
    external_process_calls: u32,
    flush_calls: u32,
    renderer_work_calls: u32,
    renderer_progress_samples: u32,
    frontend_source_calls: u32,
    flush_remaining: u32,
    flush_started: bool,
    planner: CarrierPlannerState,
    publication_gate: CarrierPublicationGate,
    pending_id14_payload_bytes: Option<u16>,
    last_prepared_reservation_bytes: i32,
    runtime_present: bool,
}

impl InputEofLifecycle {
    /// Constructs the selected observed cold projection: four active flushes,
    /// no pending ID14 extent, and a live runtime/workspace owner.
    pub const fn cold() -> Self {
        Self {
            external_process_calls: 0,
            flush_calls: 0,
            renderer_work_calls: 0,
            renderer_progress_samples: 0,
            frontend_source_calls: 0,
            flush_remaining: INITIAL_FLUSH_REMAINING,
            flush_started: false,
            planner: CarrierPlannerState::cold(),
            publication_gate: CarrierPublicationGate::cold(),
            pending_id14_payload_bytes: None,
            last_prepared_reservation_bytes: 0,
            runtime_present: true,
        }
    }

    /// Applies one observed process call. Calls 1-2 do not invoke frontend;
    /// calls 3-4 carry the two observed frontend source steps.
    pub fn process(
        &mut self,
        input: ExternalProcessInput<'_>,
    ) -> Result<ExternalProcessReceipt, InputLifecycleInputError> {
        if self.external_process_calls >= MAX_EXTERNAL_PROCESS_CALLS {
            return Err(InputLifecycleInputError::SelectedProcessCallLimit);
        }
        validate_source_records(input.source_records)?;
        validate_reader_source_position(input.source_records, input.reader_source)?;

        let call_number = self.external_process_calls + 1;
        let expects_frontend = call_number >= 3;
        if input.frontend_source.is_some() != expects_frontend {
            return Err(InputLifecycleInputError::UnexpectedFrontendOrigin);
        }

        self.external_process_calls = call_number;
        let renderer_work = self.run_renderer_work(input.renderer_span);
        let frontend_source = input
            .frontend_source
            .map(|source| self.run_frontend_source(source));

        Ok(ExternalProcessReceipt {
            call_number,
            control_position: input.source_records[0].control_position,
            reader_source: input.reader_source,
            consumed_flag: 1,
            external_output_length: 0,
            return_i32: 0,
            renderer_work,
            frontend_source,
            after: self.snapshot(),
        })
    }

    /// Applies one flush call. Active flushes set `flush_started`, execute one
    /// renderer/frontend step, and decrement only after that selected step
    /// returns. Empty flush performs no work and preserves the started flag.
    pub fn flush(
        &mut self,
        input: Option<ActiveFlushInput>,
    ) -> Result<FlushReceipt, InputLifecycleInputError> {
        if self.flush_calls >= MAX_FLUSH_CALLS {
            return Err(InputLifecycleInputError::SelectedFlushCallLimit);
        }
        if self.flush_remaining == 0 && input.is_some() {
            return Err(InputLifecycleInputError::InputProvidedForEmptyFlush);
        }
        if self.flush_remaining > 0 && input.is_none() {
            return Err(InputLifecycleInputError::MissingActiveFlushInput);
        }

        self.flush_calls += 1;
        let call_number = self.flush_calls;
        if let Some(active) = input {
            self.flush_started = true;
            let renderer_work = self.run_renderer_work(active.renderer_span);
            let frontend_source = self.run_frontend_source(active.frontend_source);
            self.flush_remaining -= 1;
            Ok(FlushReceipt {
                call_number,
                return_i32: 0,
                valid_output_bytes: frontend_source.publication.valid_output_bytes(),
                renderer_work: Some(renderer_work),
                frontend_source: Some(frontend_source),
                after: self.snapshot(),
            })
        } else {
            Ok(FlushReceipt {
                call_number,
                return_i32: 0,
                valid_output_bytes: 0,
                renderer_work: None,
                frontend_source: None,
                after: self.snapshot(),
            })
        }
    }

    /// Captures the observed post-flush early return without changing any
    /// lifecycle state. Output length and consumed flag are set before return.
    pub fn process_after_flush_started(
        &self,
    ) -> Result<PostFlushProcessReceipt, InputLifecycleInputError> {
        if !self.flush_started {
            return Err(InputLifecycleInputError::FlushHasNotStarted);
        }
        Ok(PostFlushProcessReceipt {
            return_i32: POST_FLUSH_PROCESS_RETURN_I32,
            return_raw_u32: POST_FLUSH_PROCESS_RETURN_RAW_U32,
            consumed_flag: 1,
            output_length: 0,
            state_after: self.snapshot(),
        })
    }

    /// Consumes the selected owner and returns only its pre-retirement scalar
    /// projection. It does not retain a descriptor/body or read memory after
    /// allocation retirement.
    pub fn close(mut self) -> CloseReceipt {
        self.runtime_present = false;
        CloseReceipt {
            return_i32: 0,
            scalar_state_at_close: self.snapshot(),
            owner_close_order: OwnerCloseOrder::ClosedBeforeFree,
            allocation_retirement: AllocationRetirement::Retired,
            post_retirement_read: PostRetirementRead::NoRead,
            semantic_queue_status: SemanticQueueStatus::PendingStateRetained,
        }
    }

    fn run_renderer_work(&mut self, span: RendererSpanSamples) -> RendererWorkReceipt {
        self.renderer_work_calls += 1;
        let progress_before = self.renderer_progress_samples;
        if self.renderer_progress_samples < RENDERER_PROGRESS_LIMIT {
            self.renderer_progress_samples += RENDERER_HOP_SAMPLES;
        }
        RendererWorkReceipt {
            invocation: self.renderer_work_calls,
            span,
            progress_before,
            progress_after: self.renderer_progress_samples,
        }
    }

    fn run_frontend_source(&mut self, input: FrontendSourceInput) -> FrontendSourceReceipt {
        self.frontend_source_calls += 1;
        let planner = self.planner.plan(input.planner_request_body_bytes, false);
        self.last_prepared_reservation_bytes = input.planner_request_body_bytes;
        self.pending_id14_payload_bytes = Some(input.pending_id14_payload_bytes);
        let publication = self
            .publication_gate
            .advance(FRONTEND_INTERNAL_OUTPUT_BYTES);
        FrontendSourceReceipt {
            invocation: self.frontend_source_calls,
            planner,
            pending_id14_payload_bytes: input.pending_id14_payload_bytes,
            publication,
        }
    }

    pub fn snapshot(&self) -> InputEofLifecycleSnapshot {
        InputEofLifecycleSnapshot {
            external_process_calls: self.external_process_calls,
            flush_calls: self.flush_calls,
            renderer_work_calls: self.renderer_work_calls,
            renderer_progress_samples: self.renderer_progress_samples,
            frontend_source_calls: self.frontend_source_calls,
            gate_progress_samples: self.publication_gate.progress_samples(),
            published_au_count: self.publication_gate.published_au_count(),
            flush_remaining: self.flush_remaining,
            flush_started: self.flush_started,
            planner_counter: self.planner.counter(),
            planner_enabled: self.planner.enabled(),
            planner_first_block: self.planner.first_block().value(),
            planner_segments: *self.planner.segment_lengths(),
            last_prepared_reservation_bytes: self.last_prepared_reservation_bytes,
            pending_id14_payload_bytes: self.pending_id14_payload_bytes,
            runtime_present: self.runtime_present,
        }
    }
}

impl Default for InputEofLifecycle {
    fn default() -> Self {
        Self::cold()
    }
}

fn validate_source_records(
    records: &[SourceControlRecord; SOURCE_CHANNELS],
) -> Result<(), InputLifecycleInputError> {
    let control_position = records[0].control_position;
    for (expected_index, record) in records.iter().enumerate() {
        if record.control_position != control_position {
            return Err(InputLifecycleInputError::ControlPositionMismatch);
        }
        if u16::try_from(expected_index).unwrap_or(u16::MAX) != record.source_index.value() {
            return Err(InputLifecycleInputError::SourceIndexOrderMismatch);
        }
    }
    Ok(())
}

fn validate_reader_source_position(
    records: &[SourceControlRecord; SOURCE_CHANNELS],
    reader: ReaderSourcePositionObservation,
) -> Result<(), InputLifecycleInputError> {
    if reader.selected_item.end_sample().value() < reader.selected_item.start_sample().value() {
        return Err(InputLifecycleInputError::InvalidSelectedItemRange);
    }
    if reader.cursor.control_position != records[0].control_position
        || reader.cursor.before().value() != reader.cursor.control_position.value()
    {
        return Err(InputLifecycleInputError::ReaderCursorMismatch);
    }
    if reader.derived_selected_source_sample().is_none() {
        return Err(InputLifecycleInputError::ReaderCursorMismatch);
    }
    if reader
        .derived_selected_source_sample()
        .is_some_and(|source_sample| {
            source_sample.value() >= reader.selected_item.end_sample().value()
        })
    {
        return Err(InputLifecycleInputError::InvalidSelectedItemRange);
    }
    Ok(())
}

/// Result from one renderer child work call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RendererWorkReceipt {
    invocation: u32,
    span: RendererSpanSamples,
    progress_before: u32,
    progress_after: u32,
}

impl RendererWorkReceipt {
    pub const fn invocation(self) -> u32 {
        self.invocation
    }

    pub const fn span(self) -> RendererSpanSamples {
        self.span
    }

    pub const fn progress_before(self) -> u32 {
        self.progress_before
    }

    pub const fn progress_after(self) -> u32 {
        self.progress_after
    }
}

/// Result from one observed frontend source call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrontendSourceReceipt {
    invocation: u32,
    planner: CarrierPlanReceipt,
    pending_id14_payload_bytes: u16,
    publication: PublicationGateReceipt,
}

impl FrontendSourceReceipt {
    pub const fn invocation(self) -> u32 {
        self.invocation
    }

    pub const fn planner(self) -> CarrierPlanReceipt {
        self.planner
    }

    pub const fn pending_id14_payload_bytes(self) -> u16 {
        self.pending_id14_payload_bytes
    }

    pub const fn publication(self) -> PublicationGateReceipt {
        self.publication
    }

    pub const fn frontend_ordinal(self) -> FrontendCallOrdinal {
        self.publication.frontend_call()
    }

    pub const fn published_au_ordinal(self) -> Option<PublishedAuOrdinal> {
        self.publication.published_au_ordinal()
    }
}

/// Result of one external input process call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExternalProcessReceipt {
    call_number: u32,
    control_position: ExternalControlPosition,
    reader_source: ReaderSourcePositionObservation,
    consumed_flag: u32,
    external_output_length: u32,
    return_i32: i32,
    renderer_work: RendererWorkReceipt,
    frontend_source: Option<FrontendSourceReceipt>,
    after: InputEofLifecycleSnapshot,
}

impl ExternalProcessReceipt {
    pub const fn call_number(self) -> u32 {
        self.call_number
    }

    pub const fn control_position(self) -> ExternalControlPosition {
        self.control_position
    }

    pub const fn reader_source(self) -> ReaderSourcePositionObservation {
        self.reader_source
    }

    pub const fn consumed_flag(self) -> u32 {
        self.consumed_flag
    }

    pub const fn external_output_length(self) -> u32 {
        self.external_output_length
    }

    pub const fn return_i32(self) -> i32 {
        self.return_i32
    }

    pub const fn renderer_work(self) -> RendererWorkReceipt {
        self.renderer_work
    }

    pub const fn frontend_source(self) -> Option<FrontendSourceReceipt> {
        self.frontend_source
    }

    pub const fn after(self) -> InputEofLifecycleSnapshot {
        self.after
    }
}

/// Result of one active or empty flush call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FlushReceipt {
    call_number: u32,
    return_i32: i32,
    valid_output_bytes: u32,
    renderer_work: Option<RendererWorkReceipt>,
    frontend_source: Option<FrontendSourceReceipt>,
    after: InputEofLifecycleSnapshot,
}

impl FlushReceipt {
    pub const fn call_number(self) -> u32 {
        self.call_number
    }

    pub const fn return_i32(self) -> i32 {
        self.return_i32
    }

    pub const fn valid_output_bytes(self) -> u32 {
        self.valid_output_bytes
    }

    pub const fn renderer_work(self) -> Option<RendererWorkReceipt> {
        self.renderer_work
    }

    pub const fn frontend_source(self) -> Option<FrontendSourceReceipt> {
        self.frontend_source
    }

    pub const fn after(self) -> InputEofLifecycleSnapshot {
        self.after
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerCloseOrder {
    ClosedBeforeFree,
    NotClosedBeforeFree,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllocationRetirement {
    Retired,
    StillAllocated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostRetirementRead {
    NoRead,
    Read,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticQueueStatus {
    PendingStateRetained,
}

/// Process-after-EOF early return projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PostFlushProcessReceipt {
    return_i32: i32,
    return_raw_u32: u32,
    consumed_flag: u32,
    output_length: u32,
    state_after: InputEofLifecycleSnapshot,
}

impl PostFlushProcessReceipt {
    pub const fn return_i32(self) -> i32 {
        self.return_i32
    }

    pub const fn return_raw_u32(self) -> u32 {
        self.return_raw_u32
    }

    pub const fn consumed_flag(self) -> u32 {
        self.consumed_flag
    }

    pub const fn output_length(self) -> u32 {
        self.output_length
    }

    pub const fn state_after(self) -> InputEofLifecycleSnapshot {
        self.state_after
    }
}

/// Scalar state copied by close before workspace retirement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InputEofLifecycleSnapshot {
    external_process_calls: u32,
    flush_calls: u32,
    renderer_work_calls: u32,
    renderer_progress_samples: u32,
    frontend_source_calls: u32,
    gate_progress_samples: u32,
    published_au_count: u32,
    flush_remaining: u32,
    flush_started: bool,
    planner_counter: u32,
    planner_enabled: bool,
    planner_first_block: u8,
    planner_segments: [u16; 6],
    last_prepared_reservation_bytes: i32,
    pending_id14_payload_bytes: Option<u16>,
    runtime_present: bool,
}

impl InputEofLifecycleSnapshot {
    pub const fn external_process_calls(self) -> u32 {
        self.external_process_calls
    }

    pub const fn flush_calls(self) -> u32 {
        self.flush_calls
    }

    pub const fn renderer_work_calls(self) -> u32 {
        self.renderer_work_calls
    }

    pub const fn renderer_progress_samples(self) -> u32 {
        self.renderer_progress_samples
    }

    pub const fn frontend_source_calls(self) -> u32 {
        self.frontend_source_calls
    }

    pub const fn gate_progress_samples(self) -> u32 {
        self.gate_progress_samples
    }

    pub const fn published_au_count(self) -> u32 {
        self.published_au_count
    }

    pub const fn flush_remaining(self) -> u32 {
        self.flush_remaining
    }

    pub const fn flush_started(self) -> bool {
        self.flush_started
    }

    pub const fn planner_counter(self) -> u32 {
        self.planner_counter
    }

    pub const fn planner_enabled(self) -> bool {
        self.planner_enabled
    }

    pub const fn planner_first_block(self) -> u8 {
        self.planner_first_block
    }

    pub const fn planner_segments(&self) -> &[u16; 6] {
        &self.planner_segments
    }

    pub const fn last_prepared_reservation_bytes(self) -> i32 {
        self.last_prepared_reservation_bytes
    }

    pub const fn pending_id14_payload_bytes(self) -> Option<u16> {
        self.pending_id14_payload_bytes
    }

    pub const fn queue_occupied_count(self) -> u32 {
        if self.pending_id14_payload_bytes.is_some() {
            1
        } else {
            0
        }
    }

    pub const fn runtime_present(self) -> bool {
        self.runtime_present
    }
}

/// Close/free evidence projection. The pending byte count is a scalar captured
/// before retirement; the descriptor/body allocation itself is not retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloseReceipt {
    return_i32: i32,
    scalar_state_at_close: InputEofLifecycleSnapshot,
    owner_close_order: OwnerCloseOrder,
    allocation_retirement: AllocationRetirement,
    post_retirement_read: PostRetirementRead,
    semantic_queue_status: SemanticQueueStatus,
}

impl CloseReceipt {
    pub const fn return_i32(self) -> i32 {
        self.return_i32
    }

    pub const fn scalar_state_at_close(self) -> InputEofLifecycleSnapshot {
        self.scalar_state_at_close
    }

    pub const fn owner_close_order(self) -> OwnerCloseOrder {
        self.owner_close_order
    }

    pub const fn allocation_retirement(self) -> AllocationRetirement {
        self.allocation_retirement
    }

    pub const fn post_retirement_read(self) -> PostRetirementRead {
        self.post_retirement_read
    }

    pub const fn semantic_queue_status(self) -> SemanticQueueStatus {
        self.semantic_queue_status
    }
}

#[cfg(test)]
mod tests {
    use super::InputEofLifecycle;
    use serde_json::Value;
    use std::path::PathBuf;

    const FIXTURE_DIRECTORY: &str = "tests/fixtures/input_eof_lifecycle";

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(FIXTURE_DIRECTORY)
            .join(name)
    }

    fn fixture_json(name: &str) -> Value {
        let bytes = std::fs::read(fixture_path(name)).unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[test]
    fn empty_flush_with_zero_started_flag_preserves_the_flag_and_performs_no_work() {
        let expected = fixture_json("EXPECTED.json");
        let empty_observation = expected["early_returns"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["kind"] == "empty_flush" && row["input_flush_started"] == 0)
            .unwrap();
        let mut lifecycle = InputEofLifecycle::cold();
        lifecycle.flush_remaining = 0;
        lifecycle.flush_started = false;
        let before = lifecycle.snapshot();

        let empty = lifecycle.flush(None).unwrap();

        assert_eq!(
            empty.return_i32(),
            empty_observation["return_i32"].as_i64().unwrap() as i32
        );
        assert_eq!(
            empty.valid_output_bytes(),
            empty_observation["output_length_after"]
                .as_str()
                .unwrap()
                .parse::<u32>()
                .unwrap()
        );
        assert_eq!(empty.renderer_work(), None);
        assert_eq!(empty.frontend_source(), None);
        assert_eq!(
            empty.after().flush_started(),
            empty_observation["flush_started_after"].as_u64().unwrap() == 1
        );
        assert_eq!(
            empty.after().renderer_work_calls(),
            before.renderer_work_calls()
        );
        assert_eq!(
            empty.after().frontend_source_calls(),
            before.frontend_source_calls()
        );
        assert_eq!(empty.after().planner_counter(), before.planner_counter());
    }

    #[test]
    fn lifecycle_contract_keeps_control_positions_and_reset_scope_explicit() {
        let contract = fixture_json("CONTRACT.json");
        let overlay = fixture_json("CURRENT-OVERLAY.json");
        assert!(
            contract["not_closed"]
                .as_array()
                .unwrap()
                .contains(&Value::String(
                    "Same-owner reset/recreate native event".to_owned()
                ))
        );
        assert_eq!(overlay["absolute_ADM_WAV_offset"], "NOT_BOUND");
        assert_eq!(overlay["same_owner_reset"], "NOT_RUN");
        assert_eq!(overlay["whole_F13C_ready"], "NO");
    }
}
