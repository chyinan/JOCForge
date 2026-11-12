//! Independent validation-only parser for the selected AC-3 block-zero subset.

use jocforge_domain::codec::bap::{BapError, BapProfileInput, BapRowInput, allocate_bap_row};

const FULLBAND_CHANNELS: usize = 5;
const FULLBAND_ENDMANT: usize = 217;
const LFE_ENDMANT: usize = 7;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedAc3AudioBlock {
    pub block_switch_flags: [bool; FULLBAND_CHANNELS],
    pub dither_flags: [bool; FULLBAND_CHANNELS],
    pub dynamic_range_code: Option<u8>,
    pub cplinu: bool,
    pub lfe_exponents_present: bool,
    pub channel_bandwidth_codes: [u8; FULLBAND_CHANNELS],
    pub fullband_exponents: [[u8; FULLBAND_ENDMANT]; FULLBAND_CHANNELS],
    pub gain_range_codes: [u8; FULLBAND_CHANNELS],
    pub lfe_exponents: [u8; LFE_ENDMANT],
    pub delta_bit_allocation_present: bool,
    pub skip_field_present: bool,
    pub mantissa_field_count: usize,
    pub bit_length: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ac3AudioBlockParseError {
    InputExtent,
    Truncated,
    UnsupportedSyntax,
    InvalidField,
    InvalidMantissaCode,
    Bap(BapError),
}

impl From<BapError> for Ac3AudioBlockParseError {
    fn from(error: BapError) -> Self {
        Self::Bap(error)
    }
}

struct BitReader<'a> {
    bytes: &'a [u8],
    bit_length: u32,
    cursor: u32,
}

impl<'a> BitReader<'a> {
    fn new(bytes: &'a [u8], bit_length: u32) -> Result<Self, Ac3AudioBlockParseError> {
        let expected_bytes = usize::try_from(bit_length.div_ceil(8))
            .map_err(|_| Ac3AudioBlockParseError::InputExtent)?;
        if bytes.len() != expected_bytes {
            return Err(Ac3AudioBlockParseError::InputExtent);
        }
        if bit_length % 8 != 0 && !bytes.is_empty() {
            let unused = 8 - (bit_length % 8) as u8;
            let mask = (1_u8 << unused) - 1;
            if bytes[bytes.len() - 1] & mask != 0 {
                return Err(Ac3AudioBlockParseError::InputExtent);
            }
        }
        Ok(Self {
            bytes,
            bit_length,
            cursor: 0,
        })
    }

    fn read_bit(&mut self) -> Result<bool, Ac3AudioBlockParseError> {
        Ok(self.read_unsigned(1)? != 0)
    }

    fn read_unsigned(&mut self, width: u8) -> Result<u32, Ac3AudioBlockParseError> {
        let end = self
            .cursor
            .checked_add(u32::from(width))
            .ok_or(Ac3AudioBlockParseError::Truncated)?;
        if end > self.bit_length {
            return Err(Ac3AudioBlockParseError::Truncated);
        }
        let mut value = 0_u32;
        for _ in 0..width {
            let byte_index = usize::try_from(self.cursor / 8)
                .map_err(|_| Ac3AudioBlockParseError::InputExtent)?;
            let bit_index = 7 - (self.cursor % 8);
            let bit = (self.bytes[byte_index] >> bit_index) & 1;
            value = (value << 1) | u32::from(bit);
            self.cursor += 1;
        }
        Ok(value)
    }

    fn skip(&mut self, count: u32) -> Result<(), Ac3AudioBlockParseError> {
        let end = self
            .cursor
            .checked_add(count)
            .ok_or(Ac3AudioBlockParseError::Truncated)?;
        if end > self.bit_length {
            return Err(Ac3AudioBlockParseError::Truncated);
        }
        self.cursor = end;
        Ok(())
    }
}

/// Parses one AC-3 `acmod=7`, `lfeon=1`, block-zero audblk and verifies exact bit consumption.
///
/// The parser is validation-only. It derives the mantissa field extent from parsed exponents and
/// BAP controls, then checks each grouped or individual mantissa code against its legal code space.
pub fn parse_ac3_3_2_lfe_audio_block(
    bytes: &[u8],
    bit_length: u32,
    bap_provider_bytes: &[u8],
) -> Result<ParsedAc3AudioBlock, Ac3AudioBlockParseError> {
    let mut reader = BitReader::new(bytes, bit_length)?;
    let block_switch_flags = read_bool_row(&mut reader)?;
    if block_switch_flags.iter().any(|flag| *flag) {
        return Err(Ac3AudioBlockParseError::UnsupportedSyntax);
    }
    let dither_flags = read_bool_row(&mut reader)?;
    let dynamic_range_code = if reader.read_bit()? {
        Some(
            u8::try_from(reader.read_unsigned(8)?)
                .map_err(|_| Ac3AudioBlockParseError::InvalidField)?,
        )
    } else {
        None
    };
    if !reader.read_bit()? {
        return Err(Ac3AudioBlockParseError::InvalidField);
    }
    let cplinu = reader.read_bit()?;
    if cplinu {
        return Err(Ac3AudioBlockParseError::UnsupportedSyntax);
    }

    let mut strategies = [0_u8; FULLBAND_CHANNELS];
    for strategy in &mut strategies {
        *strategy = u8::try_from(reader.read_unsigned(2)?)
            .map_err(|_| Ac3AudioBlockParseError::InvalidField)?;
        if *strategy == 0 {
            return Err(Ac3AudioBlockParseError::InvalidField);
        }
    }
    let lfe_exponents_present = reader.read_bit()?;
    if !lfe_exponents_present {
        return Err(Ac3AudioBlockParseError::InvalidField);
    }

    let mut channel_bandwidth_codes = [0_u8; FULLBAND_CHANNELS];
    let mut end_mantissas = [0_usize; FULLBAND_CHANNELS];
    for (code, end) in channel_bandwidth_codes.iter_mut().zip(&mut end_mantissas) {
        *code = u8::try_from(reader.read_unsigned(6)?)
            .map_err(|_| Ac3AudioBlockParseError::InvalidField)?;
        if *code > 60 {
            return Err(Ac3AudioBlockParseError::InvalidField);
        }
        *end = (usize::from(*code + 12) * 3) + 37;
        if *end != FULLBAND_ENDMANT {
            return Err(Ac3AudioBlockParseError::UnsupportedSyntax);
        }
    }

    let mut fullband_exponents = [[0_u8; FULLBAND_ENDMANT]; FULLBAND_CHANNELS];
    let mut gain_range_codes = [0_u8; FULLBAND_CHANNELS];
    for channel in 0..FULLBAND_CHANNELS {
        fullband_exponents[channel] =
            read_exponent_row(&mut reader, end_mantissas[channel], strategies[channel])?;
        gain_range_codes[channel] = u8::try_from(reader.read_unsigned(2)?)
            .map_err(|_| Ac3AudioBlockParseError::InvalidField)?;
    }
    let lfe_exponents = read_lfe_exponents(&mut reader)?;

    if !reader.read_bit()? {
        return Err(Ac3AudioBlockParseError::InvalidField);
    }
    let slow_decay_code = read_u8(&mut reader, 2)?;
    let fast_decay_code = read_u8(&mut reader, 2)?;
    let slow_gain_code = read_u8(&mut reader, 2)?;
    let db_per_bit_code = read_u8(&mut reader, 2)?;
    let floor_code = read_u8(&mut reader, 3)?;
    if !reader.read_bit()? {
        return Err(Ac3AudioBlockParseError::InvalidField);
    }
    let coarse_snr_offset = read_u8(&mut reader, 6)?;
    let mut fine_snr_offsets = [0_u8; FULLBAND_CHANNELS];
    let mut fast_gain_codes = [0_u8; FULLBAND_CHANNELS];
    for channel in 0..FULLBAND_CHANNELS {
        fine_snr_offsets[channel] = read_u8(&mut reader, 4)?;
        fast_gain_codes[channel] = read_u8(&mut reader, 3)?;
    }
    let lfe_fine_snr_offset = read_u8(&mut reader, 4)?;
    let lfe_fast_gain_code = read_u8(&mut reader, 3)?;
    let delta_bit_allocation_present = reader.read_bit()?;
    if delta_bit_allocation_present {
        return Err(Ac3AudioBlockParseError::UnsupportedSyntax);
    }
    let skip_field_present = reader.read_bit()?;
    if skip_field_present {
        let skip_bytes = reader.read_unsigned(9)?;
        reader.skip(
            skip_bytes
                .checked_mul(8)
                .ok_or(Ac3AudioBlockParseError::InputExtent)?,
        )?;
    }

    let mut fullband_bap = [[0_u8; FULLBAND_ENDMANT]; FULLBAND_CHANNELS];
    for channel in 0..FULLBAND_CHANNELS {
        fullband_bap[channel] = allocate_bap_row(
            Some(bap_provider_bytes),
            &BapRowInput {
                decoded_exponents: &fullband_exponents[channel],
                end_exclusive: FULLBAND_ENDMANT,
                candidate: allocation_candidate(coarse_snr_offset, fine_snr_offsets[channel]),
                slow_decay_code,
                fast_decay_code,
                slow_gain_code,
                db_per_bit_code,
                floor_code,
                fast_gain_code: fast_gain_codes[channel],
                caller_all_active_fine_zero: false,
                profile: BapProfileInput {
                    sample_rate_hz: 48_000,
                    coupling: false,
                    delta: false,
                    aht: false,
                },
            },
        )?
        .try_into()
        .map_err(|_| Ac3AudioBlockParseError::InvalidField)?;
    }
    let lfe_bap: [u8; LFE_ENDMANT] = allocate_bap_row(
        Some(bap_provider_bytes),
        &BapRowInput {
            decoded_exponents: &lfe_exponents,
            end_exclusive: LFE_ENDMANT,
            candidate: allocation_candidate(coarse_snr_offset, lfe_fine_snr_offset),
            slow_decay_code,
            fast_decay_code,
            slow_gain_code,
            db_per_bit_code,
            floor_code,
            fast_gain_code: lfe_fast_gain_code,
            caller_all_active_fine_zero: false,
            profile: BapProfileInput {
                sample_rate_hz: 48_000,
                coupling: false,
                delta: false,
                aht: false,
            },
        },
    )?
    .try_into()
    .map_err(|_| Ac3AudioBlockParseError::InvalidField)?;

    let fields = mantissa_field_limits(&fullband_bap, &lfe_bap)?;
    for (width, max_code) in &fields {
        let code = reader.read_unsigned(*width)?;
        if code > u32::from(*max_code) {
            return Err(Ac3AudioBlockParseError::InvalidMantissaCode);
        }
    }
    if reader.cursor != bit_length {
        return Err(Ac3AudioBlockParseError::InputExtent);
    }
    Ok(ParsedAc3AudioBlock {
        block_switch_flags,
        dither_flags,
        dynamic_range_code,
        cplinu,
        lfe_exponents_present,
        channel_bandwidth_codes,
        fullband_exponents,
        gain_range_codes,
        lfe_exponents,
        delta_bit_allocation_present,
        skip_field_present,
        mantissa_field_count: fields.len(),
        bit_length,
    })
}

fn read_bool_row(
    reader: &mut BitReader<'_>,
) -> Result<[bool; FULLBAND_CHANNELS], Ac3AudioBlockParseError> {
    let mut row = [false; FULLBAND_CHANNELS];
    for value in &mut row {
        *value = reader.read_bit()?;
    }
    Ok(row)
}

fn read_exponent_row(
    reader: &mut BitReader<'_>,
    end_mantissa: usize,
    strategy: u8,
) -> Result<[u8; FULLBAND_ENDMANT], Ac3AudioBlockParseError> {
    let group_span = strategy_span(strategy)?;
    let group_count = exponent_group_count(end_mantissa, group_span);
    let absolute = read_u8(reader, 4)?;
    let mut exponents = [0_u8; FULLBAND_ENDMANT];
    exponents[0] = absolute;
    let mut previous = absolute;
    for group in 0..group_count {
        let packed = read_u8(reader, 7)?;
        let digits = [packed / 25, (packed % 25) / 5, packed % 5];
        if digits.iter().any(|digit| *digit > 4) {
            return Err(Ac3AudioBlockParseError::InvalidField);
        }
        for (digit, mapped) in digits.into_iter().enumerate() {
            let difference = i16::from(mapped) - 2;
            let value = i16::from(previous) + difference;
            if !(0..=24).contains(&value) {
                return Err(Ac3AudioBlockParseError::InvalidField);
            }
            let first = 1 + group * group_span * 3 + digit * group_span;
            if first >= end_mantissa {
                continue;
            }
            let value = u8::try_from(value).map_err(|_| Ac3AudioBlockParseError::InvalidField)?;
            for span in 0..group_span {
                let index = first + span;
                if index < end_mantissa {
                    exponents[index] = value;
                }
            }
            previous = value;
        }
    }
    Ok(exponents)
}

fn read_lfe_exponents(
    reader: &mut BitReader<'_>,
) -> Result<[u8; LFE_ENDMANT], Ac3AudioBlockParseError> {
    let absolute = read_u8(reader, 4)?;
    let mut exponents = [0_u8; LFE_ENDMANT];
    exponents[0] = absolute;
    let mut previous = absolute;
    for group in 0..2 {
        let packed = read_u8(reader, 7)?;
        let digits = [packed / 25, (packed % 25) / 5, packed % 5];
        for (digit, mapped) in digits.into_iter().enumerate() {
            if mapped > 4 {
                return Err(Ac3AudioBlockParseError::InvalidField);
            }
            let value = i16::from(previous) + i16::from(mapped) - 2;
            if !(0..=24).contains(&value) {
                return Err(Ac3AudioBlockParseError::InvalidField);
            }
            previous = u8::try_from(value).map_err(|_| Ac3AudioBlockParseError::InvalidField)?;
            exponents[1 + group * 3 + digit] = previous;
        }
    }
    Ok(exponents)
}

fn mantissa_field_limits(
    fullband: &[[u8; FULLBAND_ENDMANT]; FULLBAND_CHANNELS],
    lfe: &[u8; LFE_ENDMANT],
) -> Result<Vec<(u8, u16)>, Ac3AudioBlockParseError> {
    let baps = fullband
        .iter()
        .flat_map(|row| row.iter().copied())
        .chain(lfe.iter().copied())
        .collect::<Vec<_>>();
    let mut fields = vec![None; baps.len()];
    let mut pending = [PendingGroup::EMPTY; 3];
    for (index, bap) in baps.iter().copied().enumerate() {
        if let Some(group_index) = group_index(bap) {
            let group = &mut pending[group_index];
            if group.count == 0 {
                group.first = index;
            }
            group.count += 1;
            if group.count == usize::from(group_size(bap)) {
                fields[group.first] = Some((grouped_width(bap), grouped_max_code(bap)));
                *group = PendingGroup::EMPTY;
            }
        } else if bap != 0 {
            let width = mantissa_width(bap).ok_or(Ac3AudioBlockParseError::InvalidField)?;
            let max_code = if bap == 3 {
                6
            } else if bap == 5 {
                14
            } else {
                (1_u16 << width) - 1
            };
            fields[index] = Some((width, max_code));
        }
    }
    for (group_index, bap) in [1_u8, 2, 4].into_iter().enumerate() {
        let group = pending[group_index];
        if group.count != 0 {
            fields[group.first] = Some((grouped_width(bap), grouped_max_code(bap)));
        }
    }
    Ok(fields.into_iter().flatten().collect())
}

fn read_u8(reader: &mut BitReader<'_>, width: u8) -> Result<u8, Ac3AudioBlockParseError> {
    u8::try_from(reader.read_unsigned(width)?).map_err(|_| Ac3AudioBlockParseError::InvalidField)
}

fn strategy_span(strategy: u8) -> Result<usize, Ac3AudioBlockParseError> {
    match strategy {
        1 => Ok(1),
        2 => Ok(2),
        3 => Ok(4),
        _ => Err(Ac3AudioBlockParseError::InvalidField),
    }
}

fn exponent_group_count(end_mantissa: usize, group_span: usize) -> usize {
    (end_mantissa - 1).div_ceil(group_span * 3)
}

fn allocation_candidate(coarse: u8, fine: u8) -> u16 {
    (u16::from(coarse) << 4) | u16::from(fine)
}

fn group_index(bap: u8) -> Option<usize> {
    match bap {
        1 => Some(0),
        2 => Some(1),
        4 => Some(2),
        _ => None,
    }
}

fn group_size(bap: u8) -> u8 {
    match bap {
        1 | 2 => 3,
        4 => 2,
        _ => 0,
    }
}

fn grouped_width(bap: u8) -> u8 {
    match bap {
        1 => 5,
        2 | 4 => 7,
        _ => 0,
    }
}

fn grouped_max_code(bap: u8) -> u16 {
    match bap {
        1 => 26,
        2 => 124,
        4 => 120,
        _ => 0,
    }
}

fn mantissa_width(bap: u8) -> Option<u8> {
    match bap {
        1 | 6 => Some(5),
        2 | 4 | 8 => Some(7),
        3 => Some(3),
        5 => Some(4),
        7 => Some(6),
        9 => Some(8),
        10 => Some(9),
        11 => Some(10),
        12 => Some(11),
        13 => Some(12),
        14 => Some(14),
        15 => Some(16),
        _ => None,
    }
}

#[derive(Clone, Copy)]
struct PendingGroup {
    first: usize,
    count: usize,
}

impl PendingGroup {
    const EMPTY: Self = Self { first: 0, count: 0 };
}

#[cfg(test)]
mod tests {
    use super::parse_ac3_3_2_lfe_audio_block;
    use jocforge_domain::{
        codec::{
            ac3_audio_block::{
                Ac3AudioBlockError, Ac3AudioBlockInput, Ac3AudioBlockPcm,
                Ac3BlockAllocationControls, Ac3DeltaBitAllocationInput, Ac3ExponentStrategy,
                Ac3FullbandExponentPlan, Ac3MantissaQuantizerPolicy, Ac3PcmBlockWindow,
                produce_ac3_audio_block,
            },
            i0_frame_assembler::CallerOwnedAc3Cplinu,
        },
        transform::math_generated_parameters::MathGeneratedTransformParameters,
    };

    const BAP_PROVIDER: &[u8] = include_bytes!(
        "../../../outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-PUBLIC-BAP-PROVIDER-V1/public-bap-48k-v1.json"
    );

    fn synthetic_input() -> Ac3AudioBlockInput<'static> {
        let mut fullband = [Ac3PcmBlockWindow {
            previous_samples: [0.0; 256],
            current_samples: [0.0; 256],
        }; 5];
        for (channel, sample_index, value) in [
            (0, 7, 0.25),
            (1, 31, -0.125),
            (2, 63, 0.0625),
            (3, 127, -0.25),
            (4, 251, 0.125),
        ] {
            fullband[channel].current_samples[sample_index] = value;
        }
        let mut lfe = Ac3PcmBlockWindow {
            previous_samples: [0.0; 256],
            current_samples: [0.0; 256],
        };
        lfe.current_samples[19] = 0.125;

        Ac3AudioBlockInput {
            pcm: Ac3AudioBlockPcm { fullband, lfe },
            block_switch_flags: [false; 5],
            dither_flags: [false; 5],
            dynamic_range_code: None,
            cplinu: CallerOwnedAc3Cplinu::Independent,
            delta_bit_allocation: Ac3DeltaBitAllocationInput::Absent,
            skip_field: None,
            fullband_exponent_plans: [Ac3FullbandExponentPlan {
                strategy: Ac3ExponentStrategy::D15,
                decoded_exponents: [2; 217],
            }; 5],
            lfe_decoded_exponents: [2; 7],
            channel_bandwidth_codes: [48; 5],
            gain_range_codes: [0; 5],
            allocation: Ac3BlockAllocationControls {
                slow_decay_code: 2,
                fast_decay_code: 1,
                slow_gain_code: 1,
                db_per_bit_code: 2,
                floor_code: 4,
                coarse_snr_offset: 15,
                fullband_fine_snr_offsets: [0; 5],
                fullband_fast_gain_codes: [4; 5],
                lfe_fine_snr_offset: 0,
                lfe_fast_gain_code: 4,
            },
            quantizer_policy: Ac3MantissaQuantizerPolicy::NearestTiesEven,
        }
    }

    #[test]
    fn produced_block_has_complete_etsi_structure_and_exact_cursor() {
        let transform = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
        let input = synthetic_input();
        let candidate = produce_ac3_audio_block(input, &transform, BAP_PROVIDER).unwrap();
        assert!(!candidate.production_allowed());

        let parsed =
            parse_ac3_3_2_lfe_audio_block(candidate.bytes(), candidate.bit_length(), BAP_PROVIDER)
                .unwrap();
        assert_eq!(parsed.block_switch_flags, [false; 5]);
        assert_eq!(parsed.dither_flags, [false; 5]);
        assert_eq!(parsed.dynamic_range_code, None);
        assert!(!parsed.cplinu);
        assert!(parsed.lfe_exponents_present);
        assert_eq!(parsed.channel_bandwidth_codes, [48; 5]);
        assert_eq!(parsed.fullband_exponents, [[2; 217]; 5]);
        assert_eq!(parsed.gain_range_codes, [0; 5]);
        assert_eq!(parsed.lfe_exponents, [2; 7]);
        assert!(!parsed.delta_bit_allocation_present);
        assert!(!parsed.skip_field_present);
        assert!(parsed.mantissa_field_count > 0);
        assert_eq!(parsed.bit_length, candidate.bit_length());
        assert_eq!(
            candidate.as_i0_bit_span().bit_length(),
            candidate.bit_length()
        );
    }

    #[test]
    fn parser_rejects_trailing_bits() {
        let transform = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
        let candidate =
            produce_ac3_audio_block(synthetic_input(), &transform, BAP_PROVIDER).unwrap();
        assert_eq!(
            parse_ac3_3_2_lfe_audio_block(
                candidate.bytes(),
                candidate.bit_length() + 1,
                BAP_PROVIDER,
            ),
            Err(super::Ac3AudioBlockParseError::InputExtent)
        );
    }

    #[test]
    fn parser_accepts_the_caller_supplied_skip_field_branch() {
        let transform = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
        let skip_field = [0x52, 0xa9];
        let mut input = synthetic_input();
        input.skip_field = Some(&skip_field);
        let candidate = produce_ac3_audio_block(input, &transform, BAP_PROVIDER).unwrap();
        let parsed =
            parse_ac3_3_2_lfe_audio_block(candidate.bytes(), candidate.bit_length(), BAP_PROVIDER)
                .unwrap();
        assert!(parsed.skip_field_present);
        assert_eq!(parsed.bit_length, candidate.bit_length());
    }

    #[test]
    fn producer_requires_explicit_supported_coupling_and_delta_choices() {
        let transform = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
        let mut coupled = synthetic_input();
        coupled.cplinu = CallerOwnedAc3Cplinu::Coupled;
        assert_eq!(
            produce_ac3_audio_block(coupled, &transform, &[]),
            Err(Ac3AudioBlockError::UnsupportedCoupling)
        );

        let mut delta = synthetic_input();
        delta.delta_bit_allocation = Ac3DeltaBitAllocationInput::Present;
        assert_eq!(
            produce_ac3_audio_block(delta, &transform, &[]),
            Err(Ac3AudioBlockError::UnsupportedDeltaBitAllocation)
        );

        let mut gain_range = synthetic_input();
        gain_range.gain_range_codes[0] = 1;
        assert_eq!(
            produce_ac3_audio_block(gain_range, &transform, &[]),
            Err(Ac3AudioBlockError::UnsupportedGainRange)
        );
    }
}
