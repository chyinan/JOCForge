// pattern: Functional Core
//! F05R56 reader/target transition boundary. The parent spatial_control transaction owns commit.

mod exact_trigonometry;
mod reader_records;
mod target_panner;
#[cfg(test)]
mod tests;
mod type11;

use crate::{
    joc::emdf::metadata_owner::MetadataPayloadBody,
    spatial::target_context::TargetSlotIndex,
    timeline::{CodecSourceIndex, FrontendCallbackIndex},
    transaction::{AttemptId, CallbackLineageIdentity, ProgrammeId},
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const TARGET_ROWS: usize = 15;
const READER_ROWS: usize = 16;
const RECORD_BYTES: usize = 56;
const TARGET_ROW_IDENTITIES: [i64; 15] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
const SAMPLES_PER_ENERGY_ROW: usize = 1536;
const SOURCE_COUNT: usize = 48;
pub(crate) const SELECTED_PROFILE_ID: &str = "reader56-main-target-panner.v1";
const READER_PRIMING_CALLBACKS: u32 = 2;

/// Reader callback count; distinct from the frontend callback index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderInvocationIndex(u64);

impl ReaderInvocationIndex {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

fn reader_invocation_for_callback(
    callback: FrontendCallbackIndex,
) -> Option<ReaderInvocationIndex> {
    callback
        .value()
        .checked_sub(READER_PRIMING_CALLBACKS)
        .map(|invocation| ReaderInvocationIndex::new(u64::from(invocation)))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HistoryProducerGeneration {
    Cold,
    Callback(FrontendCallbackIndex),
}

impl HistoryProducerGeneration {
    fn canonical_value(self) -> i64 {
        match self {
            Self::Cold => -1,
            Self::Callback(index) => i64::from(index.value()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CleanHistoryRecord {
    payload: [u8; RECORD_BYTES],
    producer_generation: HistoryProducerGeneration,
    producer_row: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CleanHistory {
    slots: [[CleanHistoryRecord; TARGET_ROWS]; 3],
    initial_markers: [i64; TARGET_ROWS],
}

/// The exact nine-field persistent F05R56 state. Target payloads are callback-local receipt data.
#[derive(Debug, Eq, PartialEq)]
pub struct ReaderTargetState {
    callback_index: FrontendCallbackIndex,
    retired: bool,
    history: CleanHistory,
    previous_active: [i64; TARGET_ROWS],
    current_xyz_words: [u32; TARGET_ROWS * 3],
    current_ids: [u32; TARGET_ROWS],
    current_attributes: [u32; TARGET_ROWS],
    reader_records: [[u8; RECORD_BYTES]; READER_ROWS],
    reader_invocations: ReaderInvocationIndex,
}

/// Immutable callback inputs. Rows and clock domains retain their distinct selected dimensions.
pub struct ReaderTargetCallbackInputs<'a> {
    pub callback_index: FrontendCallbackIndex,
    pub reader_enabled: bool,
    pub snapshot_keys_complete: bool,
    pub programme_id: ProgrammeId,
    pub attempt_id: AttemptId,
    pub transaction_id: CallbackLineageIdentity,
    pub profile_id: &'a str,
    pub current_xyz_words: &'a [u32; TARGET_ROWS * 3],
    pub current_ids: &'a [u32; TARGET_ROWS],
    pub current_attributes: &'a [u32; TARGET_ROWS],
    pub assignment_active: &'a [u8; TARGET_ROWS],
    pub active_target_indices: &'a [u8],
    pub matrix: &'a [f32],
    pub source_flags: &'a [u8; SOURCE_COUNT],
    pub previous_overlap_rows: [&'a [f32]; TARGET_ROWS],
    pub current_mixer_rows: [&'a [f32]; TARGET_ROWS],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReaderTargetError {
    StateError,
    PipelineRetired,
    CallbackOrder,
    CallbackIdentityMismatch,
    InputError,
    ProfileUnsupported,
    NumericFailure,
}

impl ReaderTargetError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::StateError => "R56T_STATE_ERROR",
            Self::PipelineRetired => "R56T_PIPELINE_RETIRED",
            Self::CallbackOrder => "R56T_CALLBACK_ORDER",
            Self::CallbackIdentityMismatch => "R56T_CALLBACK_IDENTITY_MISMATCH",
            Self::InputError => "R56T_INPUT_ERROR",
            Self::ProfileUnsupported => "R56T_PROFILE_UNSUPPORTED",
            Self::NumericFailure => "R56T_NUMERIC_FAILURE",
        }
    }
}

/// Non-clone proposal. Only the enclosing spatial_control transaction can admit its receipt.
#[allow(dead_code)] // Parent admission is intentionally outside this selected producer slice.
pub struct ReaderTargetStageProposal {
    next_state: ReaderTargetState,
    pending_receipt: Option<ReaderTargetUpdateReceipt>,
    delivered_controls: ReaderTargetControlRows,
}

/// Callback-local typed receipt candidate; never part of ReaderTargetState serialization.
#[allow(dead_code)] // The parent transaction is the first authorized consumer of these fields.
pub struct ReaderTargetUpdateReceipt {
    programme_id: ProgrammeId,
    attempt_id: AttemptId,
    transaction_id: CallbackLineageIdentity,
    callback_generation: FrontendCallbackIndex,
    reader_invocation: ReaderInvocationIndex,
    profile_id: &'static str,
    history_producer_generations: [HistoryProducerGeneration; READER_ROWS],
    state_before_sha256: [u8; 32],
    state_after_sha256: [u8; 32],
    reader_records_sha256: [u8; 32],
    target_payload_f32le: [u8; 384],
    target_sha256: [u8; 32],
    semantic_sha256: [u8; 32],
    id11_body: Vec<u8>,
}

/// One delivered F05R56 Control52 row and its original history owner lineage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderTargetControlLineage {
    target_slot: TargetSlotIndex,
    producer_row: u8,
    producer_generation: HistoryProducerGeneration,
}

impl ReaderTargetControlLineage {
    pub const fn target_slot(self) -> TargetSlotIndex {
        self.target_slot
    }

    pub const fn producer_row(self) -> u8 {
        self.producer_row
    }

    pub const fn producer_generation(self) -> HistoryProducerGeneration {
        self.producer_generation
    }
}

/// A target-slot Control52 value paired with its delivered S0 history lineage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderTargetControlRow {
    lineage: ReaderTargetControlLineage,
    control52_words: [u32; 13],
}

impl ReaderTargetControlRow {
    pub const fn target_slot(&self) -> TargetSlotIndex {
        self.lineage.target_slot()
    }

    pub const fn producer_row(&self) -> u8 {
        self.lineage.producer_row()
    }

    pub const fn producer_generation(&self) -> HistoryProducerGeneration {
        self.lineage.producer_generation()
    }

    pub const fn lineage(&self) -> ReaderTargetControlLineage {
        self.lineage
    }

    pub const fn control52_words(&self) -> &[u32; 13] {
        &self.control52_words
    }
}

/// Immutable target-slot ordered Control52 values from one staged F05R56 callback.
/// The enclosing spatial_control parent exposes these to consumers only after admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReaderTargetControlRows {
    callback_index: FrontendCallbackIndex,
    transaction_id: CallbackLineageIdentity,
    active_target_slots: Box<[TargetSlotIndex]>,
    reader_invocation: Option<ReaderInvocationIndex>,
    rows: [ReaderTargetControlRow; TARGET_ROWS],
}

impl ReaderTargetControlRows {
    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub fn active_target_slots(&self) -> &[TargetSlotIndex] {
        &self.active_target_slots
    }

    pub const fn reader_invocation(&self) -> Option<ReaderInvocationIndex> {
        self.reader_invocation
    }

    pub const fn rows(&self) -> &[ReaderTargetControlRow; TARGET_ROWS] {
        &self.rows
    }

    pub fn lineages(&self) -> [ReaderTargetControlLineage; TARGET_ROWS] {
        std::array::from_fn(|row| self.rows[row].lineage())
    }

    fn control_words(&self) -> [[u32; 13]; TARGET_ROWS] {
        std::array::from_fn(|row| self.rows[row].control52_words)
    }

    #[cfg(test)]
    pub(crate) fn fixture(
        callback_index: FrontendCallbackIndex,
        transaction_id: CallbackLineageIdentity,
        active_target_slots: Vec<TargetSlotIndex>,
        reader_invocation: Option<ReaderInvocationIndex>,
        producer_generations: [HistoryProducerGeneration; TARGET_ROWS],
    ) -> Self {
        let rows = std::array::from_fn(|row| {
            let target_slot = TargetSlotIndex::from_validated(row as u8);
            ReaderTargetControlRow {
                lineage: ReaderTargetControlLineage {
                    target_slot,
                    producer_row: row as u8,
                    producer_generation: producer_generations[row],
                },
                control52_words: [u32::try_from(row + 1).unwrap(); 13],
            }
        });
        Self {
            callback_index,
            transaction_id,
            active_target_slots: active_target_slots.into_boxed_slice(),
            reader_invocation,
            rows,
        }
    }
}

struct MarkerHistoryPreview {
    next_state: ReaderTargetState,
    delivered_controls: ReaderTargetControlRows,
    active_after: [u8; TARGET_ROWS],
}

struct ReaderTargetComputedRows {
    reader_records: [[u8; RECORD_BYTES]; READER_ROWS],
    targets: [[u32; 6]; READER_ROWS],
    history_producer_generations: [HistoryProducerGeneration; READER_ROWS],
    id11_body: Vec<u8>,
}

impl ReaderTargetState {
    /// Builds the complete schema cold state, including constructor-owned history row identities.
    pub fn cold() -> Self {
        let cold_history = CleanHistoryRecord {
            payload: fit_history_cold_payload(),
            producer_generation: HistoryProducerGeneration::Cold,
            producer_row: 0,
        };
        let mut slots = [[cold_history; TARGET_ROWS]; 3];
        for row in 0..TARGET_ROWS {
            for slot in &mut slots {
                slot[row].producer_row = TARGET_ROW_IDENTITIES[row];
            }
        }

        let mut reader_records = [[0_u8; RECORD_BYTES]; READER_ROWS];
        for record in &mut reader_records {
            write_record_word(record, 11, 3);
        }

        Self {
            callback_index: FrontendCallbackIndex::new(0),
            retired: false,
            history: CleanHistory {
                slots,
                initial_markers: [0; TARGET_ROWS],
            },
            previous_active: [0; TARGET_ROWS],
            current_xyz_words: [0; TARGET_ROWS * 3],
            current_ids: [u32::MAX; TARGET_ROWS],
            current_attributes: [0; TARGET_ROWS],
            reader_records,
            reader_invocations: ReaderInvocationIndex::new(0),
        }
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn reader_invocations(&self) -> ReaderInvocationIndex {
        self.reader_invocations
    }

    pub const fn is_retired(&self) -> bool {
        self.retired
    }

    fn staged_copy(&self) -> Self {
        Self {
            callback_index: self.callback_index,
            retired: self.retired,
            history: self.history,
            previous_active: self.previous_active,
            current_xyz_words: self.current_xyz_words,
            current_ids: self.current_ids,
            current_attributes: self.current_attributes,
            reader_records: self.reader_records,
            reader_invocations: self.reader_invocations,
        }
    }

    /// Returns SHA-256 over the schema's complete encode_node-v1 canonical byte preimage.
    pub fn canonical_sha256(&self) -> [u8; 32] {
        let mut bytes = Vec::with_capacity(9_047);
        encode_state(self, &mut bytes);
        Sha256::digest(bytes).into()
    }
}

/// Previews one callback without mutating committed state or admitting its receipt.
pub fn stage_reader_target(
    committed: &ReaderTargetState,
    input: &ReaderTargetCallbackInputs<'_>,
) -> Result<ReaderTargetStageProposal, ReaderTargetError> {
    if committed.retired {
        return Err(ReaderTargetError::PipelineRetired);
    }
    validate_callback_order(committed, input)?;
    validate_selected_profile(input)?;
    validate_callback_inputs(input)?;

    let marker_preview = preview_marker_and_history(committed, input)?;
    let mut staged = marker_preview.next_state;
    staged.callback_index = FrontendCallbackIndex::new(
        input
            .callback_index
            .value()
            .checked_add(1)
            .ok_or(ReaderTargetError::StateError)?,
    );
    staged.current_xyz_words = *input.current_xyz_words;
    staged.current_ids = *input.current_ids;
    staged.current_attributes = *input.current_attributes;
    staged.previous_active = marker_preview.active_after.map(i64::from);

    let mut delivered_controls = marker_preview.delivered_controls;
    if !input.reader_enabled {
        return Ok(ReaderTargetStageProposal {
            next_state: staged,
            pending_receipt: None,
            delivered_controls,
        });
    }

    let reader_invocation = reader_invocation_for_callback(input.callback_index)
        .ok_or(ReaderTargetError::CallbackOrder)?;
    if committed.reader_invocations != reader_invocation {
        return Err(ReaderTargetError::StateError);
    }

    let control_words = delivered_controls.control_words();
    delivered_controls.reader_invocation = Some(reader_invocation);
    let computed = preview_reader_and_targets(committed, &staged.history, input, &control_words)?;
    staged.reader_records = computed.reader_records;
    staged.reader_invocations = ReaderInvocationIndex::new(
        reader_invocation
            .value()
            .checked_add(1)
            .filter(|value| i64::try_from(*value).is_ok())
            .ok_or(ReaderTargetError::StateError)?,
    );

    let receipt = build_receipt(
        input,
        reader_invocation,
        computed.history_producer_generations,
        committed,
        &staged,
        &computed.reader_records,
        &computed.targets,
        &computed.id11_body,
    )?;
    Ok(ReaderTargetStageProposal {
        next_state: staged,
        pending_receipt: Some(receipt),
        delivered_controls,
    })
}

#[allow(dead_code)] // Reserved for the separately scoped parent admission consumer.
impl ReaderTargetStageProposal {
    pub(crate) const fn next_state(&self) -> &ReaderTargetState {
        &self.next_state
    }

    /// Proposed Control52 rows remain pending until the enclosing spatial_control parent admits them.
    pub(crate) const fn delivered_controls(&self) -> &ReaderTargetControlRows {
        &self.delivered_controls
    }

    pub(crate) const fn pending_receipt(&self) -> Option<&ReaderTargetUpdateReceipt> {
        self.pending_receipt.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn into_parts_for_test(
        self,
    ) -> (ReaderTargetState, Option<ReaderTargetUpdateReceipt>) {
        (self.next_state, self.pending_receipt)
    }
}

#[allow(dead_code)] // Receipt admission and downstream use are outside this producer slice.
impl ReaderTargetUpdateReceipt {
    pub(crate) fn programme_id(&self) -> &ProgrammeId {
        &self.programme_id
    }

    pub(crate) const fn attempt_id(&self) -> AttemptId {
        self.attempt_id
    }

    pub(crate) const fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub(crate) const fn callback_generation(&self) -> FrontendCallbackIndex {
        self.callback_generation
    }

    pub(crate) const fn reader_invocation(&self) -> ReaderInvocationIndex {
        self.reader_invocation
    }

    pub(crate) const fn profile_id(&self) -> &str {
        self.profile_id
    }

    pub(crate) const fn history_producer_generations(
        &self,
    ) -> &[HistoryProducerGeneration; READER_ROWS] {
        &self.history_producer_generations
    }

    pub(crate) const fn target_payload_f32le(&self) -> &[u8; 384] {
        &self.target_payload_f32le
    }

    pub(crate) fn target_words(&self) -> [u32; READER_ROWS * 6] {
        std::array::from_fn(|index| {
            let offset = index * 4;
            u32::from_le_bytes(
                self.target_payload_f32le[offset..offset + 4]
                    .try_into()
                    .expect("fixed target payload words are four bytes"),
            )
        })
    }

    pub(crate) const fn target_sha256(&self) -> &[u8; 32] {
        &self.target_sha256
    }

    pub(crate) const fn semantic_sha256(&self) -> &[u8; 32] {
        &self.semantic_sha256
    }

    pub(crate) const fn state_before_sha256(&self) -> &[u8; 32] {
        &self.state_before_sha256
    }

    pub(crate) const fn state_after_sha256(&self) -> &[u8; 32] {
        &self.state_after_sha256
    }

    pub(crate) fn id11_body(&self) -> &[u8] {
        &self.id11_body
    }

    pub(crate) fn id11_metadata_body(
        &self,
        writer_generation: CodecSourceIndex,
    ) -> MetadataPayloadBody {
        MetadataPayloadBody {
            writer_generation,
            bytes: self.id11_body.clone(),
        }
    }
}

fn validate_callback_order(
    committed: &ReaderTargetState,
    input: &ReaderTargetCallbackInputs<'_>,
) -> Result<(), ReaderTargetError> {
    if input.callback_index != committed.callback_index
        || input.reader_enabled != (input.callback_index.value() >= 2)
    {
        return Err(ReaderTargetError::CallbackOrder);
    }
    Ok(())
}

fn validate_selected_profile(
    input: &ReaderTargetCallbackInputs<'_>,
) -> Result<(), ReaderTargetError> {
    if input.profile_id != SELECTED_PROFILE_ID {
        return Err(ReaderTargetError::ProfileUnsupported);
    }
    Ok(())
}

fn validate_callback_inputs(
    input: &ReaderTargetCallbackInputs<'_>,
) -> Result<(), ReaderTargetError> {
    if input.transaction_id.programme_id() != &input.programme_id
        || input.transaction_id.attempt_id() != input.attempt_id
        || input.transaction_id.callback_index() != input.callback_index
    {
        return Err(ReaderTargetError::CallbackIdentityMismatch);
    }
    if !input.snapshot_keys_complete
        || input.active_target_indices.len() > TARGET_ROWS
        || input.matrix.len() != input.active_target_indices.len() * SOURCE_COUNT
    {
        return Err(ReaderTargetError::InputError);
    }

    let mut seen = [false; TARGET_ROWS];
    for target in input.active_target_indices {
        let row = usize::from(*target);
        if row >= TARGET_ROWS || seen[row] || input.assignment_active[row] != 1 {
            return Err(ReaderTargetError::InputError);
        }
        seen[row] = true;
    }
    for (row, active) in input.assignment_active.iter().enumerate() {
        if *active > 1 || (seen[row] != (*active == 1)) {
            return Err(ReaderTargetError::InputError);
        }
    }

    if input
        .current_xyz_words
        .iter()
        .any(|bits| !f32::from_bits(*bits).is_finite())
        || input
            .matrix
            .iter()
            .any(|coefficient| !coefficient.is_finite())
    {
        return Err(ReaderTargetError::InputError);
    }
    for row in 0..TARGET_ROWS {
        if input.previous_overlap_rows[row].len() < SAMPLES_PER_ENERGY_ROW
            || input.current_mixer_rows[row].len() < SAMPLES_PER_ENERGY_ROW
            || input.previous_overlap_rows[row][..SAMPLES_PER_ENERGY_ROW]
                .iter()
                .chain(&input.current_mixer_rows[row][..SAMPLES_PER_ENERGY_ROW])
                .any(|sample| !sample.is_finite())
        {
            return Err(ReaderTargetError::InputError);
        }
    }
    Ok(())
}

fn preview_marker_and_history(
    committed: &ReaderTargetState,
    input: &ReaderTargetCallbackInputs<'_>,
) -> Result<MarkerHistoryPreview, ReaderTargetError> {
    let mut staged = committed.staged_copy();
    let mut control_words = [[0_u32; 13]; TARGET_ROWS];
    let mut producer_generations = [HistoryProducerGeneration::Cold; TARGET_ROWS];
    let mut producer_rows = [0_u8; TARGET_ROWS];
    let mut active_after = [0_u8; TARGET_ROWS];
    for row in 0..TARGET_ROWS {
        let has_witness = has_quiet_witness(input, row);
        let quiet = !has_witness;
        let current_active = u8::from(input.assignment_active[row] == 1 && has_witness);
        let previous_active = u8::try_from(committed.previous_active[row])
            .map_err(|_| ReaderTargetError::StateError)?;
        let marker = choose_marker(committed, input, row, current_active, previous_active)?;
        let fit_record = make_fit_record(input, row, quiet, current_active, previous_active);
        let generated = CleanHistoryRecord {
            payload: words_to_bytes(&fit_record),
            producer_generation: HistoryProducerGeneration::Callback(input.callback_index),
            producer_row: TARGET_ROW_IDENTITIES[row],
        };
        let after = update_history_row(&committed.history, row, marker, generated);
        staged.history.slots[0][row] = after[0];
        staged.history.slots[1][row] = after[1];
        staged.history.slots[2][row] = after[2];
        for (word, delivered_word) in control_words[row].iter_mut().enumerate() {
            *delivered_word = read_record_word(&after[0].payload, word + 1);
        }
        producer_generations[row] = after[0].producer_generation;
        producer_rows[row] =
            u8::try_from(after[0].producer_row).map_err(|_| ReaderTargetError::StateError)?;
        active_after[row] = current_active;
    }
    let mut rows = Vec::with_capacity(TARGET_ROWS);
    for row in 0..TARGET_ROWS {
        let target_slot =
            u8::try_from(TARGET_ROW_IDENTITIES[row]).map_err(|_| ReaderTargetError::StateError)?;
        rows.push(ReaderTargetControlRow {
            lineage: ReaderTargetControlLineage {
                target_slot: TargetSlotIndex::from_validated(target_slot),
                producer_row: producer_rows[row],
                producer_generation: producer_generations[row],
            },
            control52_words: control_words[row],
        });
    }
    let rows: [ReaderTargetControlRow; TARGET_ROWS] =
        rows.try_into().map_err(|_| ReaderTargetError::StateError)?;
    let active_target_slots = input
        .active_target_indices
        .iter()
        .copied()
        .map(TargetSlotIndex::from_validated)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    Ok(MarkerHistoryPreview {
        next_state: staged,
        delivered_controls: ReaderTargetControlRows {
            callback_index: input.callback_index,
            transaction_id: input.transaction_id.clone(),
            active_target_slots,
            reader_invocation: None,
            rows,
        },
        active_after,
    })
}

fn has_quiet_witness(input: &ReaderTargetCallbackInputs<'_>, target_row: usize) -> bool {
    if input.assignment_active[target_row] == 0 {
        return false;
    }
    let Some(ordinal) = input
        .active_target_indices
        .iter()
        .position(|target| usize::from(*target) == target_row)
    else {
        return false;
    };
    let row = &input.matrix[ordinal * SOURCE_COUNT..(ordinal + 1) * SOURCE_COUNT];
    row.iter()
        .enumerate()
        .any(|(source, coefficient)| input.source_flags[source] == 0 && *coefficient > 0.0)
}

fn choose_marker(
    committed: &ReaderTargetState,
    input: &ReaderTargetCallbackInputs<'_>,
    row: usize,
    current_active: u8,
    previous_active: u8,
) -> Result<i64, ReaderTargetError> {
    if current_active == 1 && previous_active == 0 {
        return Ok(-1);
    }
    if !target_changed(committed, input, row)? {
        return Ok(0);
    }
    let old_energy = energy(input.previous_overlap_rows[row])?;
    let new_energy = energy(input.current_mixer_rows[row])?;
    let hundred = 100.0_f32;
    let old_limit = checked_mul(new_energy, hundred)?;
    if old_energy > old_limit {
        return Ok(1);
    }
    let new_limit = checked_mul(old_energy, hundred)?;
    if new_energy > new_limit {
        return Ok(-1);
    }
    Ok(0)
}

fn target_changed(
    committed: &ReaderTargetState,
    input: &ReaderTargetCallbackInputs<'_>,
    row: usize,
) -> Result<bool, ReaderTargetError> {
    if committed.current_ids[row] != input.current_ids[row]
        || committed.current_attributes[row] != input.current_attributes[row]
    {
        return Ok(true);
    }
    let coordinate_start = row * 3;
    let mut distance = 0.0_f32;
    for axis in 0..3 {
        let previous = f32::from_bits(committed.current_xyz_words[coordinate_start + axis]);
        let current = f32::from_bits(input.current_xyz_words[coordinate_start + axis]);
        let difference = checked_sub(previous, current)?;
        distance = checked_add(distance, checked_mul(difference, difference)?)?;
    }
    Ok(distance > 0.1_f32)
}

fn energy(samples: &[f32]) -> Result<f32, ReaderTargetError> {
    let mut total = 0.0_f32;
    for sample in &samples[..SAMPLES_PER_ENERGY_ROW] {
        total = checked_add(total, checked_mul(*sample, *sample)?)?;
    }
    Ok(total)
}

fn checked_add(left: f32, right: f32) -> Result<f32, ReaderTargetError> {
    finite_result(left + right)
}

fn checked_sub(left: f32, right: f32) -> Result<f32, ReaderTargetError> {
    finite_result(left - right)
}

fn checked_mul(left: f32, right: f32) -> Result<f32, ReaderTargetError> {
    finite_result(left * right)
}

fn finite_result(value: f32) -> Result<f32, ReaderTargetError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(ReaderTargetError::NumericFailure)
    }
}

fn make_fit_record(
    input: &ReaderTargetCallbackInputs<'_>,
    row: usize,
    quiet: bool,
    current_active: u8,
    previous_active: u8,
) -> [u32; 14] {
    let attribute = input.current_attributes[row];
    let coordinate_start = row * 3;
    [
        u32::from(quiet),
        u32::from(current_active == 0 && previous_active == 0),
        (attribute >> 4) & 15,
        (attribute >> 8) & 3,
        (attribute >> 10) & 3,
        0,
        input.current_xyz_words[coordinate_start],
        input.current_xyz_words[coordinate_start + 1],
        input.current_xyz_words[coordinate_start + 2],
        attribute & 1,
        0,
        0,
        (attribute >> 1) & 7,
        1,
    ]
}

fn update_history_row(
    history: &CleanHistory,
    row: usize,
    marker: i64,
    generated: CleanHistoryRecord,
) -> [CleanHistoryRecord; 3] {
    let old = [
        history.slots[0][row],
        history.slots[1][row],
        history.slots[2][row],
    ];
    let mut next = [old[1], old[1], old[2]];
    match marker {
        -1 => next = [generated; 3],
        0 => {
            next[1] = generated;
            next[2] = generated;
        }
        1 => next[2] = generated,
        _ => {}
    }
    if read_record_word(&next[0].payload, 1) != 0 && read_record_word(&next[1].payload, 1) != 0 {
        next[1] = next[0];
    }
    next
}

fn words_to_bytes(words: &[u32; 14]) -> [u8; RECORD_BYTES] {
    let mut bytes = [0_u8; RECORD_BYTES];
    for (index, word) in words.iter().enumerate() {
        let start = index * 4;
        bytes[start..start + 4].copy_from_slice(&word.to_le_bytes());
    }
    bytes
}

fn read_record_word(record: &[u8; RECORD_BYTES], index: usize) -> u32 {
    let start = index * 4;
    u32::from_le_bytes(
        record[start..start + 4]
            .try_into()
            .expect("fixed word size"),
    )
}

fn preview_reader_and_targets(
    committed: &ReaderTargetState,
    history_after: &CleanHistory,
    input: &ReaderTargetCallbackInputs<'_>,
    controls: &[[u32; 13]; TARGET_ROWS],
) -> Result<ReaderTargetComputedRows, ReaderTargetError> {
    let (decoded_rows, id11_body) = type11::roundtrip(controls)?;
    let reader_records = reader_records::materialize(&committed.reader_records, &decoded_rows)?;
    let targets = target_panner::generate_targets(&reader_records)?;
    let mut row_generations = [HistoryProducerGeneration::Cold; READER_ROWS];
    row_generations[0] = HistoryProducerGeneration::Callback(input.callback_index);
    for row in 0..TARGET_ROWS {
        row_generations[row + 1] = history_after.slots[0][row].producer_generation;
    }
    Ok(ReaderTargetComputedRows {
        reader_records,
        targets,
        history_producer_generations: row_generations,
        id11_body,
    })
}

#[allow(clippy::too_many_arguments)] // Receipt binds all independently owned producer values.
fn build_receipt(
    input: &ReaderTargetCallbackInputs<'_>,
    reader_invocation: ReaderInvocationIndex,
    history_producer_generations: [HistoryProducerGeneration; READER_ROWS],
    before_state: &ReaderTargetState,
    after_state: &ReaderTargetState,
    reader_records: &[[u8; RECORD_BYTES]; READER_ROWS],
    targets: &[[u32; 6]; READER_ROWS],
    id11_body: &[u8],
) -> Result<ReaderTargetUpdateReceipt, ReaderTargetError> {
    if id11_body.len() > 160 {
        return Err(ReaderTargetError::StateError);
    }
    let state_before_sha256 = before_state.canonical_sha256();
    let state_after_sha256 = after_state.canonical_sha256();
    let mut reader_bytes = Vec::with_capacity(READER_ROWS * RECORD_BYTES);
    for record in reader_records {
        reader_bytes.extend_from_slice(record);
    }
    let reader_records_sha256: [u8; 32] = Sha256::digest(&reader_bytes).into();
    let mut target_payload_f32le = [0_u8; 384];
    let mut target_offset = 0;
    for row in targets {
        for bits in row {
            if !f32::from_bits(*bits).is_finite() {
                return Err(ReaderTargetError::NumericFailure);
            }
            target_payload_f32le[target_offset..target_offset + 4]
                .copy_from_slice(&bits.to_le_bytes());
            target_offset += 4;
        }
    }
    let target_sha256: [u8; 32] = Sha256::digest(target_payload_f32le).into();
    let mut fields = BTreeMap::<String, Value>::new();
    fields.insert(
        "schema".to_owned(),
        Value::String("reader-target-update-receipt.v1".to_owned()),
    );
    fields.insert(
        "callback_identity".to_owned(),
        serde_json::json!({
            "programme_id": input.programme_id.as_str(),
            "attempt_id_u64": input.attempt_id.value(),
            "frontend_callback_index_u32": input.transaction_id.callback_index().value()
        }),
    );
    fields.insert(
        "callback_generation_g".to_owned(),
        Value::from(input.callback_index.value()),
    );
    fields.insert(
        "reader_invocation_q".to_owned(),
        Value::from(reader_invocation.value()),
    );
    fields.insert(
        "profile_id".to_owned(),
        Value::String(SELECTED_PROFILE_ID.to_owned()),
    );
    fields.insert(
        "history_producer_generations".to_owned(),
        Value::Array(
            history_producer_generations
                .iter()
                .map(|generation| Value::from(generation.canonical_value()))
                .collect(),
        ),
    );
    fields.insert(
        "state_before_sha256".to_owned(),
        Value::String(hex(&state_before_sha256)),
    );
    fields.insert(
        "state_after_sha256".to_owned(),
        Value::String(hex(&state_after_sha256)),
    );
    fields.insert(
        "reader_records_shape".to_owned(),
        serde_json::json!([16, 56]),
    );
    fields.insert("reader_records_bytes".to_owned(), Value::from(896));
    fields.insert(
        "reader_records_sha256".to_owned(),
        Value::String(hex(&reader_records_sha256)),
    );
    fields.insert("target_shape".to_owned(), serde_json::json!([16, 6]));
    fields.insert("target_dtype".to_owned(), Value::String("F32LE".to_owned()));
    fields.insert("target_bytes".to_owned(), Value::from(384));
    fields.insert(
        "target_sha256".to_owned(),
        Value::String(hex(&target_sha256)),
    );
    fields.insert(
        "target_payload_hex".to_owned(),
        Value::String(hex(&target_payload_f32le)),
    );
    fields.insert("persistent_target_state_bytes".to_owned(), Value::from(0));
    fields.insert("persistent_target_state_tag".to_owned(), Value::Null);
    let canonical = serde_json::to_vec(&fields).map_err(|_| ReaderTargetError::NumericFailure)?;
    let semantic_sha256 = Sha256::digest(canonical).into();
    Ok(ReaderTargetUpdateReceipt {
        programme_id: input.programme_id.clone(),
        attempt_id: input.attempt_id,
        transaction_id: input.transaction_id.clone(),
        callback_generation: input.callback_index,
        reader_invocation,
        profile_id: SELECTED_PROFILE_ID,
        history_producer_generations,
        state_before_sha256,
        state_after_sha256,
        reader_records_sha256,
        target_payload_f32le,
        target_sha256,
        semantic_sha256,
        id11_body: id11_body.to_vec(),
    })
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn fit_history_cold_payload() -> [u8; RECORD_BYTES] {
    let words = [0_u32, 1, 0, 3, 2, 0, 0, 0, 0, 0, 0, 0, 0, 1];
    let mut payload = [0_u8; RECORD_BYTES];
    for (index, word) in words.into_iter().enumerate() {
        let start = index * 4;
        payload[start..start + 4].copy_from_slice(&word.to_le_bytes());
    }
    payload
}

fn write_record_word(record: &mut [u8; RECORD_BYTES], index: usize, word: u32) {
    let start = index * 4;
    record[start..start + 4].copy_from_slice(&word.to_le_bytes());
}

fn encode_state(state: &ReaderTargetState, output: &mut Vec<u8>) {
    encode_dataclass_start(output, "ReaderTargetState", 9);
    encode_field(output, "callback_index");
    encode_i64(output, i64::from(state.callback_index.value()));
    encode_field(output, "retired");
    encode_bool(output, state.retired);
    encode_field(output, "history");
    encode_dataclass_start(output, "CleanHistory", 2);
    encode_field(output, "slots");
    encode_tuple_start(output, 3);
    for slot in &state.history.slots {
        encode_tuple_start(output, TARGET_ROWS as u64);
        for record in slot {
            encode_history_record(output, record);
        }
    }
    encode_field(output, "initial_markers");
    encode_i64_tuple(output, &state.history.initial_markers);
    encode_field(output, "previous_active");
    encode_i64_tuple(output, &state.previous_active);
    encode_field(output, "current_xyz_words");
    encode_i64_tuple_u32(output, &state.current_xyz_words);
    encode_field(output, "current_ids");
    encode_i64_tuple_u32(output, &state.current_ids);
    encode_field(output, "current_attributes");
    encode_i64_tuple_u32(output, &state.current_attributes);
    encode_field(output, "reader_records");
    encode_tuple_start(output, READER_ROWS as u64);
    for record in &state.reader_records {
        encode_bytes(output, record);
    }
    encode_field(output, "reader_invocations");
    encode_i64(
        output,
        i64::try_from(state.reader_invocations.value()).expect("state range checked"),
    );
}

fn encode_history_record(output: &mut Vec<u8>, record: &CleanHistoryRecord) {
    encode_dataclass_start(output, "CleanHistoryRecord", 3);
    encode_field(output, "payload");
    encode_bytes(output, &record.payload);
    encode_field(output, "producer_generation");
    encode_i64(output, record.producer_generation.canonical_value());
    encode_field(output, "producer_row");
    encode_i64(output, record.producer_row);
}

fn encode_dataclass_start(output: &mut Vec<u8>, name: &str, fields: u16) {
    output.push(b'D');
    output.extend_from_slice(&(name.len() as u16).to_le_bytes());
    output.extend_from_slice(name.as_bytes());
    output.extend_from_slice(&fields.to_le_bytes());
}

fn encode_field(output: &mut Vec<u8>, name: &str) {
    output.extend_from_slice(&(name.len() as u16).to_le_bytes());
    output.extend_from_slice(name.as_bytes());
}

fn encode_tuple_start(output: &mut Vec<u8>, count: u64) {
    output.push(b'T');
    output.extend_from_slice(&count.to_le_bytes());
}

fn encode_bytes(output: &mut Vec<u8>, bytes: &[u8]) {
    output.push(b'B');
    output.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    output.extend_from_slice(bytes);
}

fn encode_i64(output: &mut Vec<u8>, value: i64) {
    output.push(b'I');
    output.extend_from_slice(&value.to_le_bytes());
}

fn encode_bool(output: &mut Vec<u8>, value: bool) {
    output.push(b'Z');
    output.push(u8::from(value));
}

fn encode_i64_tuple<const N: usize>(output: &mut Vec<u8>, values: &[i64; N]) {
    encode_tuple_start(output, N as u64);
    for value in values {
        encode_i64(output, *value);
    }
}

fn encode_i64_tuple_u32<const N: usize>(output: &mut Vec<u8>, values: &[u32; N]) {
    encode_tuple_start(output, N as u64);
    for value in values {
        encode_i64(output, i64::from(*value));
    }
}
