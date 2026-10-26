// pattern: Functional Core
//! Pure local measurement-result aggregation and opaque I32 projections.
//!
//! The raw 44-byte record and its I32 member remain separate because their byte
//! layout and identity with the F06 model result are unbound. A distinct typed
//! path converts the model's public result kind to the ID1 boolean. This module
//! does not select a native runtime provider or own committed programme state.

use crate::{meter::measurement_receipt::PublicMeasurementResultKind, numeric::FiniteF32};

const MEASUREMENT_RECORD_BYTES: usize = 44;
const INITIAL_LOUDNESS_COMPARATOR_BITS: u32 = 0xc2f0_0000;

/// Canonical I32 boolean for the ID1 preferred field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Id1PreferredValue(i32);

impl Id1PreferredValue {
    /// Converts the public F06 measurement result kind, not the internal kind code.
    pub const fn from_public_result_kind(kind: PublicMeasurementResultKind) -> Self {
        if kind.value() == 1 { Self(1) } else { Self(0) }
    }

    /// Converts an opaque measurement-result I32 using the strict equality rule.
    pub const fn from_measurement_i32(source_i32: i32) -> Self {
        Self(id1_preferred_from_measurement_i32(source_i32))
    }

    /// Returns the canonical zero or one I32 value.
    pub const fn value(self) -> i32 {
        self.0
    }
}

/// One local measurement candidate without an inferred I32 field layout.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeasurementRecordCandidate {
    record_bytes: [u8; MEASUREMENT_RECORD_BYTES],
    comparison_loudness: FiniteF32,
    opaque_i32_member: i32,
}

impl MeasurementRecordCandidate {
    /// Carries a whole 44-byte record, its comparison loudness, and its opaque I32 member.
    pub const fn new(
        record_bytes: [u8; MEASUREMENT_RECORD_BYTES],
        comparison_loudness: FiniteF32,
        opaque_i32_member: i32,
    ) -> Self {
        Self {
            record_bytes,
            comparison_loudness,
            opaque_i32_member,
        }
    }
}

/// A pure aggregate value initialized from a zero record and an F32 -120 comparator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeasurementAggregate {
    record_bytes: [u8; MEASUREMENT_RECORD_BYTES],
    opaque_i32_member: i32,
    loudness_comparator_bits: u32,
    has_winning_record: bool,
}

impl MeasurementAggregate {
    /// Creates the local zero-record aggregate; this is not a selected-job default.
    pub const fn cold() -> Self {
        Self {
            record_bytes: [0; MEASUREMENT_RECORD_BYTES],
            opaque_i32_member: 0,
            loudness_comparator_bits: INITIAL_LOUDNESS_COMPARATOR_BITS,
            has_winning_record: false,
        }
    }

    /// Replaces the complete candidate record only when its F32 loudness is strictly greater.
    ///
    /// Equal loudness retains the current record and its opaque I32 member.
    #[must_use]
    pub fn consider(self, candidate: MeasurementRecordCandidate) -> (Self, bool) {
        if candidate.comparison_loudness.value() > f32::from_bits(self.loudness_comparator_bits) {
            (
                Self {
                    record_bytes: candidate.record_bytes,
                    opaque_i32_member: candidate.opaque_i32_member,
                    loudness_comparator_bits: candidate.comparison_loudness.to_bits(),
                    has_winning_record: true,
                },
                true,
            )
        } else {
            (self, false)
        }
    }

    /// Returns the selected whole record, or the zero record if no candidate won.
    pub const fn record_bytes(&self) -> &[u8; MEASUREMENT_RECORD_BYTES] {
        &self.record_bytes
    }

    /// Returns the separately carried I32 member without assigning it a tag name or offset.
    pub const fn opaque_i32_member(&self) -> i32 {
        self.opaque_i32_member
    }

    /// Returns the exact F32 comparator bits used by this aggregate.
    pub const fn loudness_comparator_bits(&self) -> u32 {
        self.loudness_comparator_bits
    }

    /// Reports whether a candidate strictly exceeded the initial/current comparator.
    pub const fn has_winning_record(&self) -> bool {
        self.has_winning_record
    }
}

/// Maps an opaque measurement-result I32 to the canonical ID1 I32 boolean.
pub const fn id1_preferred_from_measurement_i32(source_i32: i32) -> i32 {
    if source_i32 == 1 { 1 } else { 0 }
}

/// Applies the processor-backed provider's unnamed raw-I32 remap.
pub const fn processor_value_to_measurement_i32(raw_i32: i32) -> i32 {
    match raw_i32 {
        3 => 2,
        2 => 1,
        1 => 0,
        _ => -1,
    }
}
