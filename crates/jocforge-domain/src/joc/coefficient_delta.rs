// pattern: Functional Core
//! Explicit-selector JOC coefficient delta candidate from the washed DEE writer boundary.

const VALUES_PER_SLOT: usize = 3;
const CODEWORD_WIDTH: u8 = 7;
const OUTPUT_WORD_WIDTH: u64 = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockDeltaSelector {
    Disabled,
    EveryValue,
    EverySecondValue,
    EveryFourthValue,
}

impl TryFrom<u8> for BlockDeltaSelector {
    type Error = CoefficientDeltaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Disabled),
            1 => Ok(Self::EveryValue),
            2 => Ok(Self::EverySecondValue),
            3 => Ok(Self::EveryFourthValue),
            _ => Err(CoefficientDeltaError::UnsupportedSelector),
        }
    }
}

impl BlockDeltaSelector {
    const fn stride(self) -> Option<usize> {
        match self {
            Self::Disabled => None,
            Self::EveryValue => Some(1),
            Self::EverySecondValue => Some(2),
            Self::EveryFourthValue => Some(4),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoefficientDeltaError {
    UnsupportedSelector,
    InsufficientSlots,
    CapacityOverflow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoefficientDeltaPayloadCandidate {
    values: Vec<i16>,
    bytes: Vec<u8>,
    valid_bit_count: u64,
    word_padding_bits: u8,
}

impl CoefficientDeltaPayloadCandidate {
    pub fn values(&self) -> &[i16] {
        &self.values
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub const fn valid_bit_count(&self) -> u64 {
        self.valid_bit_count
    }

    pub const fn word_padding_bits(&self) -> u8 {
        self.word_padding_bits
    }
}

/// Builds the washed DEE JOC delta candidate for one explicit block selector.
///
/// Selector zero skips the difference branch. Nonzero selectors map to strides
/// 1, 2, and 4; differences are `current - previous + 2`, missing values are
/// filled with 2, and each value is written MSB-first at seven bits. The final
/// partial word is zero-padded to a 16-bit boundary. This candidate is not yet
/// connected to the selected production coefficient-payload path because its
/// selected-profile selector source has not been delivered. The current model
/// uses signed-16 wrapping for the short difference; native overflow behavior
/// is not established by the washed evidence.
pub fn build_coefficient_delta_candidate(
    short_coefficients: &[i16],
    total_slots: u16,
    selector_value: u8,
) -> Result<Option<CoefficientDeltaPayloadCandidate>, CoefficientDeltaError> {
    let selector = BlockDeltaSelector::try_from(selector_value)?;
    let Some(stride) = selector.stride() else {
        return Ok(None);
    };

    let output_count = usize::from(total_slots)
        .checked_mul(VALUES_PER_SLOT)
        .ok_or(CoefficientDeltaError::CapacityOverflow)?;
    let difference_count = short_coefficients.len().saturating_sub(1) / stride;
    if difference_count > output_count {
        return Err(CoefficientDeltaError::InsufficientSlots);
    }

    let mut values = Vec::with_capacity(output_count);
    for current_index in (stride..short_coefficients.len()).step_by(stride) {
        let previous_index = current_index - stride;
        values.push(
            short_coefficients[current_index]
                .wrapping_sub(short_coefficients[previous_index])
                .wrapping_add(2),
        );
    }
    values.resize(output_count, 2);

    let valid_bit_count = u64::try_from(values.len())
        .unwrap_or(u64::MAX)
        .saturating_mul(u64::from(CODEWORD_WIDTH));
    let word_padding_bits =
        ((OUTPUT_WORD_WIDTH - (valid_bit_count % OUTPUT_WORD_WIDTH)) % OUTPUT_WORD_WIDTH) as u8;
    let padded_bit_count = valid_bit_count.saturating_add(u64::from(word_padding_bits));
    let byte_count = usize::try_from(padded_bit_count / 8)
        .map_err(|_| CoefficientDeltaError::CapacityOverflow)?;
    let mut bytes = Vec::with_capacity(byte_count);
    let mut bit_cursor = 0_usize;
    for value in &values {
        let code = u16::from_be_bytes(value.to_be_bytes()) & 0x7f;
        for shift in (0..CODEWORD_WIDTH).rev() {
            if bit_cursor % 8 == 0 {
                bytes.push(0);
            }
            if ((code >> shift) & 1) != 0 {
                let byte_index = bit_cursor / 8;
                let bit_index = 7 - (bit_cursor % 8);
                bytes[byte_index] |= 1 << bit_index;
            }
            bit_cursor += 1;
        }
    }
    bytes.resize(byte_count, 0);

    Ok(Some(CoefficientDeltaPayloadCandidate {
        values,
        bytes,
        valid_bit_count,
        word_padding_bits,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_controls_stride_and_tail_fill() {
        let coefficients = [10, 11, 14, 12, 20];

        let every_value = build_coefficient_delta_candidate(&coefficients, 2, 1)
            .unwrap()
            .unwrap();
        assert_eq!(every_value.values(), &[3, 5, 0, 10, 2, 2]);

        let every_second = build_coefficient_delta_candidate(&coefficients, 1, 2)
            .unwrap()
            .unwrap();
        assert_eq!(every_second.values(), &[6, 8, 2]);

        let every_fourth = build_coefficient_delta_candidate(&coefficients, 1, 3)
            .unwrap()
            .unwrap();
        assert_eq!(every_fourth.values(), &[12, 2, 2]);
    }

    #[test]
    fn disabled_selector_skips_the_delta_branch() {
        assert_eq!(
            build_coefficient_delta_candidate(&[1, 2, 3], 1, 0),
            Ok(None)
        );
    }

    #[test]
    fn selector_rejects_values_outside_the_washed_table() {
        assert_eq!(
            build_coefficient_delta_candidate(&[1, 2, 3], 1, 4),
            Err(CoefficientDeltaError::UnsupportedSelector)
        );
    }

    #[test]
    fn seven_bit_values_are_msb_first_and_final_word_is_padded_to_u16() {
        let candidate = build_coefficient_delta_candidate(&[10, 8, 7, 7], 1, 1)
            .unwrap()
            .unwrap();

        assert_eq!(candidate.values(), &[0, 1, 2]);
        assert_eq!(candidate.valid_bit_count(), 21);
        assert_eq!(candidate.word_padding_bits(), 11);
        assert_eq!(candidate.bytes(), &[0x00, 0x04, 0x10, 0x00]);
    }
}
