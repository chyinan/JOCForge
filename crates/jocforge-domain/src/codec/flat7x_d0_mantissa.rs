// pattern: Functional Core
//! Flat7x D0 four-channel mantissa quantization and grouped packing.

use super::{
    bap::{
        FLAT7X_D0_BAP_BIN_COUNT, FLAT7X_D0_BAP_BLOCK_COUNT, FLAT7X_D0_BAP_CHANNEL_COUNT,
        Flat7xD0BapBlock, Flat7xD0BapFrame,
    },
    flat7x_d0_exponents::Flat7xD0ExponentRow,
    flat7x_d0_spectrum::Flat7xD0SpectrumBlock,
    grouped_quantizer::{
        GroupedPackObservation, MantissaBlockError, MantissaBlockItemInput, MantissaBlockOutput,
        MantissaChannel, MantissaItemObservation, pack_block_mantissas,
    },
};

pub const FLAT7X_D0_MANTISSA_QUANTIZER_POLICY: &str = "JOCFORGE_EXISTING_EAC3_QUANTIZER_V1";

const D0_CHANNEL_ORDER: [MantissaChannel; FLAT7X_D0_BAP_CHANNEL_COUNT] = [
    MantissaChannel::LeftSurround,
    MantissaChannel::RightSurround,
    MantissaChannel::LeftRearSurround,
    MantissaChannel::RightRearSurround,
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Flat7xD0AudioBlockIndex(u8);

impl Flat7xD0AudioBlockIndex {
    pub const fn new(value: u8) -> Option<Self> {
        if value < FLAT7X_D0_BAP_BLOCK_COUNT as u8 {
            Some(Self(value))
        } else {
            None
        }
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0MantissaInput<'a> {
    pub spectrum: &'a Flat7xD0SpectrumBlock,
    pub exponent_rows: &'a [Flat7xD0ExponentRow; FLAT7X_D0_BAP_CHANNEL_COUNT],
    pub bap_frame: &'a Flat7xD0BapFrame,
    pub block_index: Flat7xD0AudioBlockIndex,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xD0MantissaBlock {
    packed: MantissaBlockOutput,
}

impl Flat7xD0MantissaBlock {
    pub fn items(&self) -> &[MantissaItemObservation] {
        self.packed.items()
    }

    pub fn grouped_packs(&self) -> &[GroupedPackObservation] {
        self.packed.grouped_packs()
    }

    pub fn packed_bytes(&self) -> &[u8] {
        self.packed.packed_bytes()
    }

    pub const fn bit_count(&self) -> u64 {
        self.packed.bit_count()
    }

    pub const fn quantizer_policy(&self) -> &'static str {
        FLAT7X_D0_MANTISSA_QUANTIZER_POLICY
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

/// Packs one D0 block in `[Ls, Rs, Lrs, Rrs]` order from matching spectrum, D15, and BAP rows.
///
/// BAP 1/2/4 groups follow the shared E-AC-3 grouped quantizer and may cross an exponent-set
/// boundary. This stage returns only mantissa bits; it does not serialize D0 `audblk` controls.
pub fn pack_flat7x_d0_mantissa_block(
    input: &Flat7xD0MantissaInput<'_>,
) -> Result<Flat7xD0MantissaBlock, MantissaBlockError> {
    let block_index = usize::from(input.block_index.value());
    let bap: &Flat7xD0BapBlock = &input.bap_frame.blocks()[block_index];
    let coefficients = input.spectrum.coefficient_bits_by_channel();
    let bap_rows = bap.rows();
    let mut items = Vec::with_capacity(FLAT7X_D0_BAP_CHANNEL_COUNT * FLAT7X_D0_BAP_BIN_COUNT);

    for channel in 0..FLAT7X_D0_BAP_CHANNEL_COUNT {
        let exponents = input.exponent_rows[channel].decoded_exponents();
        for bin in 0..FLAT7X_D0_BAP_BIN_COUNT {
            items.push(MantissaBlockItemInput {
                channel: D0_CHANNEL_ORDER[channel],
                bin: u16::try_from(bin).map_err(|_| MantissaBlockError::BitCountOverflow)?,
                coefficient_word: coefficients[channel][bin],
                decoded_exponent: i16::from(exponents[bin]),
                bap: bap_rows[channel][bin],
            });
        }
    }

    let packed = pack_block_mantissas(&items)?;
    Ok(Flat7xD0MantissaBlock { packed })
}
