// pattern: Functional Core
//! D0 coefficient exponent extraction and the selected legal D15 row envelope.

use super::{
    bitio::FrameBitWriter,
    exponent_packing::{
        Eac3ExponentRowReceipt, Eac3ExponentRowWriteError, Eac3ExponentSeed, ExponentPackingError,
        PackedExponentGroup, pack_exponent_difference_group, write_eac3_exponent_row,
    },
    mantissa::raw_exponent_from_f32_word,
};

pub const FLAT7X_D0_D15_EXPONENTS_PER_ROW: usize = 253;
pub const FLAT7X_D0_D15_DIFFERENCE_COUNT: usize = FLAT7X_D0_D15_EXPONENTS_PER_ROW - 1;
pub const FLAT7X_D0_D15_GROUP_COUNT: usize = FLAT7X_D0_D15_DIFFERENCE_COUNT.div_ceil(3);
pub const FLAT7X_D0_D15_PREPROCESSOR_ID: &str = "JOCFORGE_D15_LOWER_ONLY_SEED15_SLEW2_V1";

const MAX_DECODED_EXPONENT: u8 = 24;
const MAX_ABSOLUTE_EXPONENT: u8 = 15;
const MAX_D15_DIFFERENCE: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0ExponentError {
    NonFiniteCoefficient,
    ExponentPacking(ExponentPackingError),
}

impl Flat7xD0ExponentError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::NonFiniteCoefficient => "D0_EXPONENT_INPUT_ERROR",
            Self::ExponentPacking(error) => error.code(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xD0ExponentRow {
    raw_exponents: [u8; FLAT7X_D0_D15_EXPONENTS_PER_ROW],
    decoded_exponents: [u8; FLAT7X_D0_D15_EXPONENTS_PER_ROW],
    seed: Eac3ExponentSeed,
    groups: [PackedExponentGroup; FLAT7X_D0_D15_GROUP_COUNT],
}

impl Flat7xD0ExponentRow {
    pub const fn raw_exponents(&self) -> &[u8; FLAT7X_D0_D15_EXPONENTS_PER_ROW] {
        &self.raw_exponents
    }

    pub const fn decoded_exponents(&self) -> &[u8; FLAT7X_D0_D15_EXPONENTS_PER_ROW] {
        &self.decoded_exponents
    }

    pub const fn seed(&self) -> Eac3ExponentSeed {
        self.seed
    }

    pub const fn groups(&self) -> &[PackedExponentGroup; FLAT7X_D0_D15_GROUP_COUNT] {
        &self.groups
    }

    pub const fn preprocessor_id(&self) -> &'static str {
        FLAT7X_D0_D15_PREPROCESSOR_ID
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

/// Extracts raw exponents and builds one legal D15 U4/U7 row from a D0 coefficient vector.
///
/// The selected JOCForge V1 policy caps the absolute seed at 15, forward-limits each rising
/// decoded exponent to the prior decoded exponent +2, then backward-lowers earlier exponents as
/// needed to bound downward differences. Every decoded exponent stays at or below its raw value
/// and in 0..=24. The policy makes all D15 differentials representable while retaining the exact
/// decoder-visible exponent row for downstream BAP and mantissa stages.
pub fn prepare_flat7x_d0_d15_exponent_row(
    coefficient_bits: &[u32; FLAT7X_D0_D15_EXPONENTS_PER_ROW],
) -> Result<Flat7xD0ExponentRow, Flat7xD0ExponentError> {
    if coefficient_bits
        .iter()
        .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(Flat7xD0ExponentError::NonFiniteCoefficient);
    }

    let raw_exponents = coefficient_bits.map(raw_exponent_from_f32_word);
    let mut decoded_exponents = [0_u8; FLAT7X_D0_D15_EXPONENTS_PER_ROW];
    decoded_exponents[0] = raw_exponents[0].min(MAX_ABSOLUTE_EXPONENT);
    for index in 1..FLAT7X_D0_D15_EXPONENTS_PER_ROW {
        let maximum = decoded_exponents[index - 1]
            .saturating_add(MAX_D15_DIFFERENCE)
            .min(MAX_DECODED_EXPONENT);
        decoded_exponents[index] = raw_exponents[index].min(maximum);
    }
    for index in (0..FLAT7X_D0_D15_EXPONENTS_PER_ROW - 1).rev() {
        let maximum = decoded_exponents[index + 1]
            .saturating_add(MAX_D15_DIFFERENCE)
            .min(MAX_DECODED_EXPONENT);
        decoded_exponents[index] = decoded_exponents[index].min(maximum);
    }

    let seed = Eac3ExponentSeed::new(decoded_exponents[0]).ok_or(
        Flat7xD0ExponentError::ExponentPacking(ExponentPackingError::ExponentInputError),
    )?;
    let mut groups = Vec::with_capacity(FLAT7X_D0_D15_GROUP_COUNT);
    for group_index in 0..FLAT7X_D0_D15_GROUP_COUNT {
        let start = group_index * 3;
        let differences = [
            exponent_difference(decoded_exponents[start], decoded_exponents[start + 1])?,
            exponent_difference(decoded_exponents[start + 1], decoded_exponents[start + 2])?,
            exponent_difference(decoded_exponents[start + 2], decoded_exponents[start + 3])?,
        ];
        groups.push(
            pack_exponent_difference_group(&differences)
                .map_err(Flat7xD0ExponentError::ExponentPacking)?,
        );
    }
    let groups = groups.try_into().map_err(|_| {
        Flat7xD0ExponentError::ExponentPacking(ExponentPackingError::ExponentInputError)
    })?;

    Ok(Flat7xD0ExponentRow {
        raw_exponents,
        decoded_exponents,
        seed,
        groups,
    })
}

fn exponent_difference(previous: u8, current: u8) -> Result<i8, Flat7xD0ExponentError> {
    i8::try_from(i16::from(current) - i16::from(previous)).map_err(|_| {
        Flat7xD0ExponentError::ExponentPacking(ExponentPackingError::ExponentInputError)
    })
}

/// Writes one prepared D0 D15 row through the existing typed U4/U7 serializer.
pub fn write_flat7x_d0_d15_exponent_row(
    writer: &mut FrameBitWriter<'_>,
    row: &Flat7xD0ExponentRow,
) -> Result<Eac3ExponentRowReceipt, Eac3ExponentRowWriteError> {
    write_eac3_exponent_row(writer, row.seed.value(), &row.groups)
}
