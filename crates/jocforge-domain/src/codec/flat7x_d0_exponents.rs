// pattern: Functional Core
//! Prepares the decoder-visible D0 exponent envelope and packs its differentials.

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
pub const FLAT7X_D0_D15_STRATEGY_CODE: u8 = 1;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum D0ExponentStrategy {
    D15,
    D25,
    D45,
}
impl D0ExponentStrategy {
    pub const fn for_rate(rate: crate::profile::CodecRateProfile) -> Self {
        use crate::profile::CodecRateProfile as Rate;
        match rate {
            Rate::Kbps384 | Rate::Kbps448 => Self::D45,
            Rate::Kbps576 | Rate::Kbps640 | Rate::Kbps768 => Self::D25,
            _ => Self::D15,
        }
    }
    pub const fn code(self) -> u8 {
        match self {
            Self::D15 => 1,
            Self::D25 => 2,
            Self::D45 => 3,
        }
    }
    const fn bins_per_exponent(self) -> usize {
        1 << (self.code() - 1)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xD0ExponentRow {
    raw_exponents: [u8; FLAT7X_D0_D15_EXPONENTS_PER_ROW],
    decoded_exponents: [u8; FLAT7X_D0_D15_EXPONENTS_PER_ROW],
    seed: Eac3ExponentSeed,
    groups: Vec<PackedExponentGroup>,
    strategy: D0ExponentStrategy,
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

    pub fn groups(&self) -> &[PackedExponentGroup] {
        &self.groups
    }

    pub const fn strategy(&self) -> D0ExponentStrategy {
        self.strategy
    }

    pub const fn preprocessor_id(&self) -> &'static str {
        match self.strategy {
            D0ExponentStrategy::D15 => FLAT7X_D0_D15_PREPROCESSOR_ID,
            D0ExponentStrategy::D25 => "D25",
            D0ExponentStrategy::D45 => "D45",
        }
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
    prepare_d0_exponent_row(coefficient_bits, D0ExponentStrategy::D15)
}

pub fn prepare_d0_exponent_row(
    coefficient_bits: &[u32; FLAT7X_D0_D15_EXPONENTS_PER_ROW],
    strategy: D0ExponentStrategy,
) -> Result<Flat7xD0ExponentRow, Flat7xD0ExponentError> {
    if coefficient_bits
        .iter()
        .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(Flat7xD0ExponentError::NonFiniteCoefficient);
    }

    let raw_exponents = coefficient_bits.map(raw_exponent_from_f32_word);
    // Each D25/D45 differential covers two/four spectral bins. The minimum
    // exponent keeps every coefficient in that group representable.
    let width = strategy.bins_per_exponent();
    let mut envelope = Vec::with_capacity(1 + FLAT7X_D0_D15_DIFFERENCE_COUNT / width);
    envelope.push(raw_exponents[0].min(MAX_ABSOLUTE_EXPONENT));
    for bins in raw_exponents[1..].chunks_exact(width) {
        let smallest = bins.iter().copied().min().unwrap_or(MAX_DECODED_EXPONENT);
        envelope.push(
            smallest
                .min(envelope[envelope.len() - 1].saturating_add(MAX_D15_DIFFERENCE))
                .min(MAX_DECODED_EXPONENT),
        );
    }
    for i in (0..envelope.len() - 1).rev() {
        envelope[i] = envelope[i].min(envelope[i + 1].saturating_add(MAX_D15_DIFFERENCE));
    }
    let seed = Eac3ExponentSeed::new(envelope[0]).ok_or(Flat7xD0ExponentError::ExponentPacking(
        ExponentPackingError::ExponentInputError,
    ))?;
    let mut groups = Vec::with_capacity(FLAT7X_D0_D15_GROUP_COUNT / width);
    for i in (0..envelope.len() - 1).step_by(3) {
        let differences = [
            exponent_difference(envelope[i], envelope[i + 1])?,
            exponent_difference(envelope[i + 1], envelope[i + 2])?,
            exponent_difference(envelope[i + 2], envelope[i + 3])?,
        ];
        groups.push(
            pack_exponent_difference_group(&differences)
                .map_err(Flat7xD0ExponentError::ExponentPacking)?,
        );
    }
    let mut decoded_exponents = [0; FLAT7X_D0_D15_EXPONENTS_PER_ROW];
    decoded_exponents[0] = envelope[0];
    for (bins, &value) in decoded_exponents[1..]
        .chunks_exact_mut(width)
        .zip(&envelope[1..])
    {
        bins.fill(value);
    }
    Ok(Flat7xD0ExponentRow {
        raw_exponents,
        decoded_exponents,
        seed,
        groups,
        strategy,
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
