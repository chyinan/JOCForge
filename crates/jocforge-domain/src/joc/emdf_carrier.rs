// pattern: Functional Core
//! Selected EMDF carrier planning, fragment writing, and publication gating.
//!
//! This module transports caller-supplied body bytes. It does not produce or
//! authenticate a protected body, ordinary codec frame, or complete programme.

const BLOCK_COUNT: usize = 6;
const FIRST_BLOCK_CAPACITY_BYTES: u16 = 507;
const LATER_BLOCK_CAPACITY_BYTES: u16 = 511;
const PERIODIC_PREFIX_BYTES: u16 = 8;
const COUNTER_PERIOD: u32 = 62;
const CARRIER_MARKER: [u8; 2] = [0x58, 0x38];
const PRESENCE_FIELD_BITS: u32 = 1;
const SKIP_LENGTH_FIELD_BITS: u32 = 9;

/// Product length admission only; payload syntax and frame feasibility must
/// still be validated by their owning stages before publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedCarrierAdmissionError {
    EmptyOrTruncatedContainer,
    CompleteContainerTooLarge,
}

/// A zero-based carrier block index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CarrierBlockIndex(u8);

impl CarrierBlockIndex {
    pub const fn value(self) -> u8 {
        self.0
    }
}

/// A bit position in an explicitly supplied frame or block buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CarrierBitCursor(u32);

impl CarrierBitCursor {
    pub const fn new(bit_offset: u32) -> Self {
        Self(bit_offset)
    }

    pub const fn bit_offset(self) -> u32 {
        self.0
    }
}

/// Mutable state owned only by the selected carrier planner.
#[derive(Debug, Eq, PartialEq)]
pub struct CarrierPlannerState {
    counter: u32,
    enabled: bool,
    first_block: CarrierBlockIndex,
    segment_lengths: [u16; BLOCK_COUNT],
}

impl CarrierPlannerState {
    pub(crate) const fn staged_copy(&self) -> Self {
        Self {
            counter: self.counter,
            enabled: self.enabled,
            first_block: self.first_block,
            segment_lengths: self.segment_lengths,
        }
    }

    /// Admits the selected optional complete container C, including its four
    /// prefix bytes. Accepted bodies B=C-4 fit one segment at fixed block 2.
    /// Rejection leaves planner state untouched. None means no optional payload;
    /// Some(0) does not represent absence or a valid empty EMDF container.
    pub fn plan_selected_container(
        &mut self,
        complete_container_bytes: Option<u16>,
    ) -> Result<CarrierPlanReceipt, SelectedCarrierAdmissionError> {
        let body_length = match complete_container_bytes {
            None => 0,
            Some(0..=4) => {
                return Err(SelectedCarrierAdmissionError::EmptyOrTruncatedContainer);
            }
            Some(length @ 5..=511) => i32::from(length - 4),
            Some(_) => return Err(SelectedCarrierAdmissionError::CompleteContainerTooLarge),
        };
        Ok(self.plan(body_length, false))
    }

    /// Constructs the observed selected cold fields; the remaining native
    /// 192-byte storage is intentionally not represented by this projection.
    pub const fn cold() -> Self {
        Self {
            counter: 0,
            enabled: false,
            first_block: CarrierBlockIndex(0),
            segment_lengths: [0; BLOCK_COUNT],
        }
    }

    pub const fn counter(&self) -> u32 {
        self.counter
    }

    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    pub const fn first_block(&self) -> CarrierBlockIndex {
        self.first_block
    }

    pub const fn segment_lengths(&self) -> &[u16; BLOCK_COUNT] {
        &self.segment_lengths
    }

    /// Applies one selected planner request. A residual failure keeps the
    /// already-updated fields, matching the observed non-rollback mutation.
    pub fn plan(
        &mut self,
        requested_body_bytes: i32,
        periodic_prefix_enabled: bool,
    ) -> CarrierPlanReceipt {
        self.segment_lengths = [0; BLOCK_COUNT];
        self.counter += 1;
        if self.counter == COUNTER_PERIOD {
            self.counter = 0;
        }
        self.enabled = requested_body_bytes > 0;

        if requested_body_bytes <= 0 {
            return self.receipt(i32::from(requested_body_bytes < 0), requested_body_bytes);
        }

        let first = match requested_body_bytes {
            ..=2_040 => 2,
            2_041..=2_551 => 1,
            _ => 0,
        };
        self.first_block = CarrierBlockIndex(first);

        let mut remaining = u32::try_from(requested_body_bytes).unwrap_or_default();
        for block in usize::from(first)..BLOCK_COUNT {
            let mut capacity = if block == usize::from(first) {
                FIRST_BLOCK_CAPACITY_BYTES
            } else {
                LATER_BLOCK_CAPACITY_BYTES
            };
            if block == 0 && periodic_prefix_enabled && self.enabled && self.counter == 0 {
                capacity -= PERIODIC_PREFIX_BYTES;
            }
            let assigned = remaining.min(u32::from(capacity));
            self.segment_lengths[block] = u16::try_from(assigned).unwrap_or_default();
            remaining -= assigned;
        }

        self.receipt(i32::from(remaining != 0), requested_body_bytes)
    }

    fn receipt(&self, return_i32: i32, requested_body_bytes: i32) -> CarrierPlanReceipt {
        CarrierPlanReceipt {
            return_i32,
            requested_body_bytes,
            enabled: self.enabled,
            first_block: self.first_block,
            segment_lengths: self.segment_lengths,
            counter_after: self.counter,
        }
    }
}

impl Default for CarrierPlannerState {
    fn default() -> Self {
        Self::cold()
    }
}

/// Snapshot of the selected planner fields after one request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CarrierPlanReceipt {
    return_i32: i32,
    requested_body_bytes: i32,
    enabled: bool,
    first_block: CarrierBlockIndex,
    segment_lengths: [u16; BLOCK_COUNT],
    counter_after: u32,
}

impl CarrierPlanReceipt {
    pub const fn return_i32(self) -> i32 {
        self.return_i32
    }

    pub const fn requested_body_bytes(self) -> i32 {
        self.requested_body_bytes
    }

    pub const fn enabled(self) -> bool {
        self.enabled
    }

    pub const fn first_block(self) -> CarrierBlockIndex {
        self.first_block
    }

    pub const fn segment_lengths(&self) -> &[u16; BLOCK_COUNT] {
        &self.segment_lengths
    }

    pub const fn counter_after(self) -> u32 {
        self.counter_after
    }
}

/// Local caller/buffer errors; no corresponding native error status is claimed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CarrierWriteError {
    SkipLengthExceedsNineBits,
    BufferTooSmall,
    CursorOverflow,
}

/// Typed view of caller-supplied serialized body bytes for carrier transport.
///
/// This wrapper does not attest provenance, produce/authenticate bytes, or
/// replace a future model-owned body/provider receipt at product integration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SerializedEmdfBodyFragment<'a> {
    bytes: &'a [u8],
    full_body_length: u16,
    first_fragment: bool,
}

impl<'a> SerializedEmdfBodyFragment<'a> {
    pub const fn new(bytes: &'a [u8], full_body_length: u16, first_fragment: bool) -> Self {
        Self {
            bytes,
            full_body_length,
            first_fragment,
        }
    }
}

/// Writes one reservation field and returns the saved payload cursor.
///
/// A present fragment uses a one-bit presence flag followed by a high-bit-first
/// nine-bit byte length. An absent fragment writes only the zero presence bit.
pub fn write_fragment_reservation(
    storage: &mut [u8],
    cursor: CarrierBitCursor,
    present: bool,
    skip_length_bytes: u16,
) -> Result<CarrierBitCursor, CarrierWriteError> {
    if !present {
        ensure_bit_capacity(storage, cursor, PRESENCE_FIELD_BITS)?;
        write_field(storage, cursor, 0, PRESENCE_FIELD_BITS)?;
        return advance_cursor(cursor, PRESENCE_FIELD_BITS);
    }
    if skip_length_bytes > 511 {
        return Err(CarrierWriteError::SkipLengthExceedsNineBits);
    }
    let field_bits = PRESENCE_FIELD_BITS + SKIP_LENGTH_FIELD_BITS;
    ensure_bit_capacity(storage, cursor, field_bits)?;

    write_field(storage, cursor, 1, PRESENCE_FIELD_BITS)?;
    let length_cursor = advance_cursor(cursor, PRESENCE_FIELD_BITS)?;
    write_field(
        storage,
        length_cursor,
        u32::from(skip_length_bytes),
        SKIP_LENGTH_FIELD_BITS,
    )?;
    advance_cursor(cursor, field_bits)
}

/// Writes one body fragment into the supplied output buffer without changing
/// bits outside the selected range. The first fragment receives the selected
/// marker and the full body length; later fragments contain raw body bytes.
pub fn write_body_fragment(
    storage: &mut [u8],
    cursor: CarrierBitCursor,
    fragment: SerializedEmdfBodyFragment<'_>,
) -> Result<CarrierBitCursor, CarrierWriteError> {
    let prefix_length = if fragment.first_fragment { 4_u32 } else { 0 };
    let fragment_length =
        u32::try_from(fragment.bytes.len()).map_err(|_| CarrierWriteError::CursorOverflow)?;
    let output_length = fragment_length
        .checked_add(prefix_length)
        .ok_or(CarrierWriteError::CursorOverflow)?;
    let output_bits = output_length
        .checked_mul(8)
        .ok_or(CarrierWriteError::CursorOverflow)?;
    ensure_bit_capacity(storage, cursor, output_bits)?;

    let mut write_cursor = cursor;
    if fragment.first_fragment {
        write_bytes(storage, &mut write_cursor, &CARRIER_MARKER)?;
        let length = fragment.full_body_length.to_be_bytes();
        write_bytes(storage, &mut write_cursor, &length)?;
    }
    write_bytes(storage, &mut write_cursor, fragment.bytes)?;
    Ok(write_cursor)
}

fn ensure_bit_capacity(
    storage: &[u8],
    cursor: CarrierBitCursor,
    bit_count: u32,
) -> Result<(), CarrierWriteError> {
    let storage_bits = u64::try_from(storage.len())
        .map_err(|_| CarrierWriteError::CursorOverflow)?
        .checked_mul(8)
        .ok_or(CarrierWriteError::CursorOverflow)?;
    let end_bit = u64::from(cursor.0)
        .checked_add(u64::from(bit_count))
        .ok_or(CarrierWriteError::CursorOverflow)?;
    if end_bit > storage_bits {
        return Err(CarrierWriteError::BufferTooSmall);
    }
    Ok(())
}

fn advance_cursor(
    cursor: CarrierBitCursor,
    bit_count: u32,
) -> Result<CarrierBitCursor, CarrierWriteError> {
    cursor
        .0
        .checked_add(bit_count)
        .map(CarrierBitCursor)
        .ok_or(CarrierWriteError::CursorOverflow)
}

fn write_field(
    storage: &mut [u8],
    cursor: CarrierBitCursor,
    value: u32,
    bit_count: u32,
) -> Result<(), CarrierWriteError> {
    for index in 0..bit_count {
        let source_shift = bit_count - index - 1;
        let destination_bit = cursor
            .0
            .checked_add(index)
            .ok_or(CarrierWriteError::CursorOverflow)?;
        let byte_index =
            usize::try_from(destination_bit / 8).map_err(|_| CarrierWriteError::CursorOverflow)?;
        let bit_in_byte = destination_bit % 8;
        let mask = 1_u8 << (7 - bit_in_byte);
        let bit = (value >> source_shift) & 1;
        let destination = storage
            .get_mut(byte_index)
            .ok_or(CarrierWriteError::BufferTooSmall)?;
        *destination = (*destination & !mask) | ((bit as u8) * mask);
    }
    Ok(())
}

fn write_bytes(
    storage: &mut [u8],
    cursor: &mut CarrierBitCursor,
    bytes: &[u8],
) -> Result<(), CarrierWriteError> {
    let length = u32::try_from(bytes.len()).map_err(|_| CarrierWriteError::CursorOverflow)?;
    let bit_count = length
        .checked_mul(8)
        .ok_or(CarrierWriteError::CursorOverflow)?;
    ensure_bit_capacity(storage, *cursor, bit_count)?;
    for (byte_index, byte) in bytes.iter().copied().enumerate() {
        let offset = u32::try_from(byte_index)
            .map_err(|_| CarrierWriteError::CursorOverflow)?
            .checked_mul(8)
            .ok_or(CarrierWriteError::CursorOverflow)?;
        write_field(
            storage,
            advance_cursor(*cursor, offset)?,
            u32::from(byte),
            8,
        )?;
    }
    *cursor = advance_cursor(*cursor, bit_count)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn selected_container_policy_preserves_units_placement_and_rejection_state() {
        use super::SelectedCarrierAdmissionError;
        for complete in 5..=511 {
            let mut planner = CarrierPlannerState::cold();
            let receipt = planner.plan_selected_container(Some(complete)).unwrap();
            assert_eq!(receipt.first_block().value(), 2);
            assert_eq!(receipt.return_i32(), 0);
            assert_eq!(receipt.segment_lengths(), &[0, 0, complete - 4, 0, 0, 0]);
        }
        let mut planner = CarrierPlannerState::cold();
        planner.plan_selected_container(Some(511)).unwrap();
        for complete in [0, 1, 4, 512, u16::MAX] {
            let before = (
                planner.counter(),
                *planner.segment_lengths(),
                planner.first_block(),
            );
            let expected = if complete <= 4 {
                SelectedCarrierAdmissionError::EmptyOrTruncatedContainer
            } else {
                SelectedCarrierAdmissionError::CompleteContainerTooLarge
            };
            assert_eq!(
                planner.plan_selected_container(Some(complete)),
                Err(expected)
            );
            assert_eq!(
                before,
                (
                    planner.counter(),
                    *planner.segment_lengths(),
                    planner.first_block()
                )
            );
        }
        let absent = planner.plan_selected_container(None).unwrap();
        assert!(!absent.enabled());
        assert_eq!(absent.segment_lengths(), &[0; 6]);
    }

    use super::{
        CarrierBitCursor, CarrierBlockIndex, CarrierPlannerState, SerializedEmdfBodyFragment,
        write_body_fragment, write_fragment_reservation,
    };
    use crate::frontend::output_publication_gate::CarrierPublicationGate;
    use serde_json::Value;
    use sha2::{Digest, Sha256};
    use std::path::PathBuf;

    const FIXTURE_DIRECTORY: &str = "tests/fixtures/f13c_carrier_pipeline";
    const HANDOFF_MANIFEST_SHA256: &str =
        "9c136d43219ccd8fc5623066864d27959f079c9cfe9dfcd10a5726e7fb20f049";
    const HANDOFF_CHECK_SHA256: &str =
        "8a0343f13fe811e5de8a64cb20ddb7fe72f7190905ef1686590240ac32cf6ff9";
    const SOURCE_MANIFEST_SHA256: &str =
        "8edaa695dc53fd9f7de96693b16259b08b7399141f266bded7e4115029f9794e";
    const BASE_MANIFEST_SHA256: &str =
        "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(FIXTURE_DIRECTORY)
            .join(name)
    }

    fn fixture_bytes(name: &str) -> Vec<u8> {
        std::fs::read(fixture_path(name)).unwrap()
    }

    fn fixture_json(name: &str) -> Value {
        serde_json::from_slice(&fixture_bytes(name)).unwrap()
    }

    fn sha256(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    fn decode_hex(value: &str) -> Vec<u8> {
        value
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }

    fn read_field(storage: &[u8], cursor: CarrierBitCursor, bit_count: u32) -> u32 {
        (0..bit_count).fold(0, |value, index| {
            let position = cursor.bit_offset() + index;
            let byte_index = usize::try_from(position / 8).unwrap();
            let bit_index = position % 8;
            (value << 1) | u32::from((storage[byte_index] >> (7 - bit_index)) & 1)
        })
    }

    fn case_bytes(pattern_length: usize) -> Vec<u8> {
        (0..pattern_length)
            .map(|index| ((37 * index + 11) % 256) as u8)
            .collect()
    }

    fn assert_handoff_payload(manifest: &Value, name: &str, local_bytes: &[u8]) {
        let entry = manifest["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["path"] == name)
            .unwrap();
        assert_eq!(entry["bytes"].as_u64().unwrap(), local_bytes.len() as u64);
        assert_eq!(entry["sha256"], sha256(local_bytes), "{name}");
    }

    #[test]
    fn washed_carrier_payloads_are_hash_bound_and_keep_the_whole_parent_open() {
        let handoff_manifest_bytes = fixture_bytes("HANDOFF-MANIFEST.json");
        let handoff_check_bytes = fixture_bytes("HANDOFF-CHECK.json");
        let handoff_manifest: Value = serde_json::from_slice(&handoff_manifest_bytes).unwrap();
        let handoff_check: Value = serde_json::from_slice(&handoff_check_bytes).unwrap();

        assert_eq!(sha256(&handoff_manifest_bytes), HANDOFF_MANIFEST_SHA256);
        assert_eq!(sha256(&handoff_check_bytes), HANDOFF_CHECK_SHA256);
        assert_eq!(handoff_manifest["files"].as_array().unwrap().len(), 16);
        assert_eq!(
            handoff_manifest["source_package_manifest_sha256"],
            SOURCE_MANIFEST_SHA256
        );
        assert_eq!(
            handoff_manifest["base_package_manifest_sha256"],
            BASE_MANIFEST_SHA256
        );
        assert_eq!(
            handoff_check["handoff_manifest_sha256"],
            HANDOFF_MANIFEST_SHA256
        );
        assert_eq!(handoff_check["payload_hashes_match"], true);
        assert_eq!(handoff_check["whole_F13C_ready"], false);
        assert_eq!(handoff_check["implementation_ready"], false);

        for name in [
            "CARRIER-ADDENDUM.md",
            "CONTRACT.json",
            "EXPECTED.json",
            "INDEPENDENT-CHECK.json",
            "INPUTS.json",
            "NATIVE-AU-JOINS.json",
            "WASH-INDEPENDENT-CHECK.json",
        ] {
            assert_handoff_payload(&handoff_manifest, name, &fixture_bytes(name));
        }

        let independent = fixture_json("INDEPENDENT-CHECK.json");
        assert_eq!(independent["payload_hashes_match"], true);
        assert_eq!(independent["replay"]["boundary_cases"], 18);
        assert_eq!(independent["replay"]["natural_published_aus"], 4);
        assert_eq!(independent["replay"]["mismatch_count"], 0);
        assert_eq!(
            independent["replay"]["ordinary_codec_audio_frame_reproduction"],
            false
        );
        assert_eq!(independent["whole_F13C_ready"], false);

        let wash_check = fixture_json("WASH-INDEPENDENT-CHECK.json");
        assert_eq!(
            wash_check["result"],
            "PASS_FOR_SELECTED_CARRIER_AND_PUBLICATION_SLICES_ONLY"
        );
        assert_eq!(wash_check["protected_body_generation_authenticated"], false);
        assert_eq!(wash_check["whole_F13C_ready"], false);
        assert_eq!(wash_check["implementation_ready"], false);
    }

    #[test]
    fn planner_and_fragment_writer_match_all_eighteen_controlled_cases() {
        let inputs = fixture_json("INPUTS.json");
        let expected = fixture_json("EXPECTED.json");
        let input_cases = inputs["boundary_cases"].as_array().unwrap();
        let expected_cases = expected["boundary_cases"].as_array().unwrap();
        assert_eq!(input_cases.len(), 18);
        assert_eq!(expected_cases.len(), input_cases.len());

        for (input, expected_case) in input_cases.iter().zip(expected_cases) {
            let name = input["name"].as_str().unwrap();
            let requested = input["n"].as_i64().unwrap() as i32;
            let counter_before = input["counter"].as_u64().unwrap() as u32;
            let prior_first = input
                .get("prior_first")
                .and_then(Value::as_u64)
                .unwrap_or(0) as u8;
            let mut planner = CarrierPlannerState::cold();
            planner.counter = counter_before;
            planner.first_block = CarrierBlockIndex(prior_first);
            let plan = planner.plan(requested, input["flag"].as_u64().unwrap() != 0);

            assert_eq!(
                plan.return_i32(),
                expected_case["plan_return"].as_i64().unwrap() as i32,
                "{name}"
            );
            assert_eq!(
                i32::from(plan.enabled()),
                expected_case["enabled"].as_i64().unwrap() as i32,
                "{name}"
            );
            assert_eq!(
                i64::from(plan.first_block().value()),
                expected_case["first_block"].as_i64().unwrap(),
                "{name}"
            );
            let expected_segments = expected_case["segments"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_u64().unwrap() as u16)
                .collect::<Vec<_>>();
            assert_eq!(
                plan.segment_lengths().as_slice(),
                expected_segments,
                "{name}"
            );
            assert_eq!(
                plan.counter_after(),
                expected_case["counter_after"].as_u64().unwrap() as u32,
                "{name}"
            );
            assert_eq!(planner.enabled(), plan.enabled(), "{name}");
            assert_eq!(planner.first_block(), plan.first_block(), "{name}");
            assert_eq!(planner.segment_lengths(), plan.segment_lengths(), "{name}");

            let writer_executed = expected_case["writer_executed"].as_bool().unwrap();
            let expected_outputs = expected_case["outputs"].as_array().unwrap();
            assert_eq!(writer_executed, !expected_outputs.is_empty(), "{name}");
            if !writer_executed {
                continue;
            }

            let body = case_bytes(usize::try_from(requested).unwrap());
            let offset = u32::try_from(input["offset"].as_u64().unwrap()).unwrap();
            let mut body_offset = 0_usize;
            let mut first_fragment = true;
            let mut output_index = 0_usize;
            for block in 0..6 {
                let segment_length = usize::from(plan.segment_lengths()[block]);
                if segment_length == 0 {
                    continue;
                }
                let segment_end = body_offset + segment_length;
                let segment = &body[body_offset..segment_end];
                let mut storage = vec![0xA5; 544];
                let cursor = CarrierBitCursor::new(offset);
                let fragment_byte_length = segment_length + if first_fragment { 4 } else { 0 };

                let mut reservation_storage = vec![0xA5; 544];
                let reserved_cursor = write_fragment_reservation(
                    &mut reservation_storage,
                    cursor,
                    true,
                    fragment_byte_length as u16,
                )
                .unwrap();
                assert_eq!(
                    reserved_cursor.bit_offset(),
                    offset + 10,
                    "{name} block {block}"
                );
                assert_eq!(
                    read_field(&reservation_storage, cursor, 1),
                    1,
                    "{name} block {block}"
                );
                assert_eq!(
                    read_field(&reservation_storage, CarrierBitCursor::new(offset + 1), 9),
                    fragment_byte_length as u32,
                    "{name} block {block}"
                );

                let end = write_body_fragment(
                    &mut storage,
                    cursor,
                    SerializedEmdfBodyFragment::new(segment, body.len() as u16, first_fragment),
                )
                .unwrap();
                let expected_output = &expected_outputs[output_index];
                assert_eq!(
                    expected_output["block"].as_u64().unwrap() as usize,
                    block,
                    "{name}"
                );
                assert_eq!(
                    end.bit_offset(),
                    expected_output["cursor_bits_after"].as_u64().unwrap() as u32,
                    "{name}"
                );
                assert_eq!(
                    storage,
                    decode_hex(expected_output["wire_bytes_hex"].as_str().unwrap()),
                    "{name} block {block}"
                );

                body_offset = segment_end;
                first_fragment = false;
                output_index += 1;
            }
            assert_eq!(body_offset, body.len(), "{name}");
            assert_eq!(output_index, expected_outputs.len(), "{name}");
        }
    }

    #[test]
    fn one_owner_sequence_preserves_mutation_on_zero_and_failure() {
        let inputs = fixture_json("INPUTS.json");
        let expected = fixture_json("EXPECTED.json");
        let requested = inputs["sequence_requested_bytes"].as_array().unwrap();
        let expected_rows = expected["sequence"].as_array().unwrap();
        assert_eq!(requested.len(), 4);
        let mut planner = CarrierPlannerState::cold();

        for (requested, expected_row) in requested.iter().zip(expected_rows) {
            let outcome = planner.plan(requested.as_i64().unwrap() as i32, false);
            assert_eq!(
                outcome.return_i32(),
                expected_row["return_i32"].as_i64().unwrap() as i32
            );
            assert_eq!(
                outcome.enabled(),
                expected_row["enabled"].as_i64().unwrap() == 1
            );
            assert_eq!(
                outcome.first_block().value(),
                expected_row["first_block"].as_u64().unwrap() as u8
            );
            assert_eq!(
                outcome.segment_lengths().as_slice(),
                expected_row["segments"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap() as u16)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                outcome.counter_after(),
                expected_row["counter_after"].as_u64().unwrap() as u32
            );
        }
        assert_eq!(planner.counter(), 4);
        assert_eq!(planner.first_block().value(), 2);
        assert_eq!(planner.segment_lengths(), &[0, 0, 316, 0, 0, 0]);
    }

    #[test]
    fn natural_published_aus_match_native_reservations_and_fragments() {
        let inputs = fixture_json("INPUTS.json");
        let expected = fixture_json("EXPECTED.json");
        let joins = fixture_json("NATIVE-AU-JOINS.json");
        let natural_inputs = inputs["natural_carriers"].as_array().unwrap();
        let natural_expected = expected["natural_carriers"].as_array().unwrap();
        let native_aus = joins["published_aus"].as_array().unwrap();
        assert_eq!(natural_inputs.len(), 4);
        assert_eq!(natural_expected.len(), natural_inputs.len());
        assert_eq!(native_aus.len(), natural_inputs.len());
        assert_eq!(joins["absolute_ADM_sample_position"], "NOT_BOUND");
        assert_eq!(joins["gate_progress_is_source_clock"], false);

        for ((input, expected_row), native) in
            natural_inputs.iter().zip(natural_expected).zip(native_aus)
        {
            assert_eq!(
                input["body_origin"],
                "NATIVE_DIAGNOSTIC_BOUNDARY_INPUT_NOT_MODEL_PRODUCTION"
            );
            let body = decode_hex(input["serialized_body_hex"].as_str().unwrap());
            let body_length = body.len() as u16;
            let planner_input = &input["planner_input"];
            let mut planner = CarrierPlannerState::cold();
            planner.counter = planner_input["counter"].as_u64().unwrap() as u32;
            planner.first_block =
                CarrierBlockIndex(planner_input["prior_first"].as_u64().unwrap() as u8);
            let plan = planner.plan(
                i32::from(body_length),
                planner_input["flag"].as_u64().unwrap() != 0,
            );

            assert_eq!(plan.return_i32(), 0);
            assert_eq!(
                plan.first_block().value(),
                native["block"].as_u64().unwrap() as u8
            );
            assert_eq!(
                plan.segment_lengths().as_slice(),
                expected_row["segments"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap() as u16)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                native["published_au_ordinal"],
                input["published_au_ordinal"]
            );
            assert_eq!(native["substream_index"], 0);
            assert_eq!(native["fragment_mismatch_bytes"], 0);
            assert_eq!(sha256(&body), expected_row["body_sha256"].as_str().unwrap());

            let bit_cursor = u32::try_from(input["frame_bit_offset"].as_u64().unwrap()).unwrap();
            assert_eq!(
                native["saved_payload_bit"].as_u64().unwrap() as u32,
                bit_cursor
            );
            let field_start = u32::try_from(native["field_start_bit"].as_u64().unwrap()).unwrap();
            let fragment_length = body_length + 4;
            assert_eq!(
                native["wire_fragment_bytes"].as_u64().unwrap(),
                u64::from(fragment_length)
            );

            let mut modeled_frame = vec![0_u8; 4_096];
            let saved_payload_cursor = write_fragment_reservation(
                &mut modeled_frame,
                CarrierBitCursor::new(field_start),
                true,
                fragment_length,
            )
            .unwrap();
            assert_eq!(saved_payload_cursor.bit_offset(), bit_cursor);
            assert_eq!(
                native["field_end_bit"].as_u64().unwrap() as u32,
                bit_cursor + u32::from(fragment_length) * 8
            );

            let fragment_end = write_body_fragment(
                &mut modeled_frame,
                saved_payload_cursor,
                SerializedEmdfBodyFragment::new(&body, body_length, true),
            )
            .unwrap();
            assert_eq!(
                fragment_end.bit_offset(),
                native["field_end_bit"].as_u64().unwrap() as u32
            );

            let fragment = [0x58, 0x38]
                .into_iter()
                .chain(body_length.to_be_bytes())
                .chain(body.iter().copied())
                .collect::<Vec<_>>();
            assert_eq!(
                sha256(&fragment),
                expected_row["actual_fragment_sha256"].as_str().unwrap()
            );
            let native_frame =
                decode_hex(expected_row["published_frame_wire_hex"].as_str().unwrap());
            assert_eq!(native_frame.len(), 4_096);
            assert_eq!(
                sha256(&native_frame),
                expected_row["published_frame_sha256"].as_str().unwrap()
            );
            assert_eq!(
                read_field(&modeled_frame, CarrierBitCursor::new(field_start), 10),
                read_field(&native_frame, CarrierBitCursor::new(field_start), 10)
            );
            assert_eq!(
                read_bits(&modeled_frame, bit_cursor, fragment.len() as u32),
                read_bits(&native_frame, bit_cursor, fragment.len() as u32)
            );
            assert_eq!(
                read_bits(&modeled_frame, bit_cursor, fragment.len() as u32),
                fragment
            );
        }
    }

    #[test]
    fn publication_gate_numbers_only_the_four_caller_visible_aus() {
        let inputs = fixture_json("INPUTS.json");
        let expected = fixture_json("EXPECTED.json");
        let joins = fixture_json("NATIVE-AU-JOINS.json");
        let internal_calls = inputs["publication_gate"]["calls"].as_array().unwrap();
        let expected_calls = expected["publication_gate"].as_array().unwrap();
        let native_gate = joins["gate_observations"].as_array().unwrap();
        let native_aus = joins["published_aus"].as_array().unwrap();
        assert_eq!(internal_calls.len(), 6);

        let mut gate = CarrierPublicationGate::cold();
        let mut published_index = 0_usize;
        for ((input, expected_row), native_row) in
            internal_calls.iter().zip(expected_calls).zip(native_gate)
        {
            let receipt = gate.advance(input["internal_output_bytes"].as_u64().unwrap() as u32);
            assert_eq!(
                receipt.frontend_call().value(),
                expected_row["frontend_invocation"].as_u64().unwrap() as u32
            );
            assert_eq!(
                receipt.progress_before(),
                expected_row["gate_progress_before"].as_u64().unwrap() as u32
            );
            assert_eq!(
                receipt.progress_after(),
                expected_row["gate_progress_after"].as_u64().unwrap() as u32
            );
            assert_eq!(
                receipt.valid_output_bytes(),
                expected_row["valid_output_bytes"].as_u64().unwrap() as u32
            );
            assert_eq!(
                receipt.progress_after(),
                native_row["gate_progress_after"].as_u64().unwrap() as u32
            );
            assert_eq!(
                receipt.valid_output_bytes(),
                native_row["valid_output_bytes"].as_u64().unwrap() as u32
            );

            if let Some(ordinal) = receipt.published_au_ordinal() {
                let native_au = &native_aus[published_index];
                assert_eq!(
                    ordinal.value(),
                    expected_row["published_au_ordinal"].as_u64().unwrap() as u32
                );
                assert_eq!(
                    ordinal.value(),
                    native_au["published_au_ordinal"].as_u64().unwrap() as u32
                );
                assert_eq!(
                    receipt.frontend_call().value(),
                    native_au["frontend_source_call"].as_u64().unwrap() as u32
                );
                assert_eq!(
                    native_au["joc_producer_invocation"].as_u64().unwrap() + 1,
                    u64::from(receipt.frontend_call().value())
                );
                published_index += 1;
            } else {
                assert!(expected_row.get("published_au_ordinal").is_none());
                assert_eq!(receipt.valid_output_bytes(), 0);
            }
        }
        assert_eq!(published_index, 4);
        assert_eq!(gate.published_au_count(), 4);
        assert_eq!(gate.progress_samples(), 4_608);
    }

    #[test]
    fn absent_reservation_writes_only_the_presence_bit() {
        let mut storage = vec![0xA5; 8];
        let cursor = CarrierBitCursor::new(3);
        let end = write_fragment_reservation(&mut storage, cursor, false, 0).unwrap();
        assert_eq!(end.bit_offset(), 4);
        assert_eq!(read_field(&storage, cursor, 1), 0);
        assert_eq!(storage[0] & 0x1F, 0x05);
    }

    fn read_bits(storage: &[u8], start: u32, byte_count: u32) -> Vec<u8> {
        (0..byte_count)
            .map(|byte| {
                let mut result = 0_u8;
                for bit in 0..8 {
                    let position = start + byte * 8 + bit;
                    let byte_index = usize::try_from(position / 8).unwrap();
                    let bit_index = position % 8;
                    result = (result << 1) | ((storage[byte_index] >> (7 - bit_index)) & 1);
                }
                result
            })
            .collect()
    }
}
