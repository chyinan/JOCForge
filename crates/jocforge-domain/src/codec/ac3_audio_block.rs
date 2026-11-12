//! Functional Core: production-disabled AC-3 3/2+LFE audio-block prototype.
//!
//! The prototype serializes one block-zero audblk from caller PCM windows and explicit coding
//! controls. It supports long transforms, independent channels, and the 48 kHz / 217-bin
//! full-band row. I0 frame assembly, six-block state, and publication remain separate.

use super::{
    bap::{BapError, BapProfileInput, BapRowInput, allocate_bap_row},
    bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
    exponent_packing::{ExponentPackingError, pack_exponent_difference_group},
    i0_frame_assembler::{
        CallerOwnedAc3Cplinu, I0Ac3AudioBlockPrefixError, I0Ac3AudioBlockPrefixInput,
        I0Ac3CplinuWriteError, OpaqueI0Ac3BitSpan, write_i0_ac3_first_audio_block_prefix,
        write_i0_ac3_first_block_cplinu,
    },
};
use crate::transform::math_generated_parameters::MathGeneratedTransformParameters;

const PCM_SAMPLES_PER_BLOCK: usize = 256;
const PCM_WINDOW_SAMPLES: usize = PCM_SAMPLES_PER_BLOCK * 2;
const FULLBAND_CHANNEL_COUNT: usize = 5;
const FULLBAND_ENDMANT: usize = 217;
const LFE_ENDMANT: usize = 7;
const AC3_SAMPLE_RATE_HZ: u32 = 48_000;
const AC3_FULLBAND_CHBW_CODE: u8 = 48;
const AC3_BLOCK_BUDGET_BYTES: usize = 2_560;

/// One AC-3 transform window: prior overlap samples followed by this block's 256 new samples.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ac3PcmBlockWindow {
    pub previous_samples: [f32; PCM_SAMPLES_PER_BLOCK],
    pub current_samples: [f32; PCM_SAMPLES_PER_BLOCK],
}

/// Explicit 3/2+LFE PCM input in L, C, R, Ls, Rs, LFE order.
///
/// The caller supplies LFE samples after its selected input preparation. This stage applies no
/// LFE filter or gain; ETSI TS 102 366 clause 7.2.1.3 leaves that encoder-side choice open.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ac3AudioBlockPcm {
    pub fullband: [Ac3PcmBlockWindow; FULLBAND_CHANNEL_COUNT],
    pub lfe: Ac3PcmBlockWindow,
}

/// AC-3 exponent coding strategy carried by one block-zero full-band channel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ac3ExponentStrategy {
    D15,
    D25,
    D45,
}

impl Ac3ExponentStrategy {
    const fn wire_code(self) -> u8 {
        match self {
            Self::D15 => 1,
            Self::D25 => 2,
            Self::D45 => 3,
        }
    }

    const fn group_span(self) -> usize {
        match self {
            Self::D15 => 1,
            Self::D25 => 2,
            Self::D45 => 4,
        }
    }
}

/// Caller-selected decoder exponents for one independent full-band channel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ac3FullbandExponentPlan {
    pub strategy: Ac3ExponentStrategy,
    pub decoded_exponents: [u8; FULLBAND_ENDMANT],
}

/// Caller-selected coarse/fine allocation controls for one block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ac3BlockAllocationControls {
    pub slow_decay_code: u8,
    pub fast_decay_code: u8,
    pub slow_gain_code: u8,
    pub db_per_bit_code: u8,
    pub floor_code: u8,
    pub coarse_snr_offset: u8,
    pub fullband_fine_snr_offsets: [u8; FULLBAND_CHANNEL_COUNT],
    pub fullband_fast_gain_codes: [u8; FULLBAND_CHANNEL_COUNT],
    pub lfe_fine_snr_offset: u8,
    pub lfe_fast_gain_code: u8,
}

/// Caller-owned choice for the AC-3 block-zero delta-allocation field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ac3DeltaBitAllocationInput {
    Absent,
    Present,
}

/// Explicit caller-selected coefficient quantizer policy for this encoder slice.
///
/// ETSI TS 102 366 defines decoder reconstruction values, while code selection between values
/// remains an encoder policy. The caller must supply this choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ac3MantissaQuantizerPolicy {
    NearestTiesEven,
}

/// Inputs for a complete, independent, long-transform AC-3 audblk.
///
/// The exponent arrays are the exact values that the decoder will reconstruct. Supplying them
/// explicitly keeps exponent grouping and normalization choices visible to the caller.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ac3AudioBlockInput<'a> {
    pub pcm: Ac3AudioBlockPcm,
    pub block_switch_flags: [bool; FULLBAND_CHANNEL_COUNT],
    pub dither_flags: [bool; FULLBAND_CHANNEL_COUNT],
    pub dynamic_range_code: Option<u8>,
    pub cplinu: CallerOwnedAc3Cplinu,
    pub delta_bit_allocation: Ac3DeltaBitAllocationInput,
    pub skip_field: Option<&'a [u8]>,
    pub fullband_exponent_plans: [Ac3FullbandExponentPlan; FULLBAND_CHANNEL_COUNT],
    pub lfe_decoded_exponents: [u8; LFE_ENDMANT],
    /// Caller-selected bandwidth code; this prototype admits code 48 (217 bins) only.
    pub channel_bandwidth_codes: [u8; FULLBAND_CHANNEL_COUNT],
    /// Caller-supplied block floating-point codes; this prototype accepts code zero only.
    pub gain_range_codes: [u8; FULLBAND_CHANNEL_COUNT],
    pub allocation: Ac3BlockAllocationControls,
    pub quantizer_policy: Ac3MantissaQuantizerPolicy,
}

/// Immutable output of one complete AC-3 block-zero producer stage.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ac3AudioBlockCandidate {
    bytes: Vec<u8>,
    bit_length: u32,
}

impl Ac3AudioBlockCandidate {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub const fn bit_length(&self) -> u32 {
        self.bit_length
    }

    /// Adapts the complete block to the existing I0 frame assembler's opaque-span boundary.
    pub fn as_i0_bit_span(&self) -> OpaqueI0Ac3BitSpan<'_> {
        OpaqueI0Ac3BitSpan::new(&self.bytes, 0, self.bit_length)
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ac3AudioBlockError {
    InvalidPcm,
    UnsupportedBlockSwitch,
    UnsupportedCoupling,
    UnsupportedDeltaBitAllocation,
    SkipFieldTooLarge,
    UnsupportedChannelBandwidth,
    UnsupportedGainRange,
    InvalidExponentPlan,
    InvalidGainRange,
    CoefficientOutsideNormalizedRange,
    BitCountOverflow,
    BlockExceedsTargetFrame,
    Bap(BapError),
    ExponentPacking(ExponentPackingError),
    Prefix(I0Ac3AudioBlockPrefixError),
    Cplinu(I0Ac3CplinuWriteError),
    BitWriter(FrameBitWriterError),
}

impl From<BapError> for Ac3AudioBlockError {
    fn from(error: BapError) -> Self {
        Self::Bap(error)
    }
}

impl From<ExponentPackingError> for Ac3AudioBlockError {
    fn from(error: ExponentPackingError) -> Self {
        Self::ExponentPacking(error)
    }
}

impl From<I0Ac3AudioBlockPrefixError> for Ac3AudioBlockError {
    fn from(error: I0Ac3AudioBlockPrefixError) -> Self {
        Self::Prefix(error)
    }
}

impl From<I0Ac3CplinuWriteError> for Ac3AudioBlockError {
    fn from(error: I0Ac3CplinuWriteError) -> Self {
        Self::Cplinu(error)
    }
}

impl From<FrameBitWriterError> for Ac3AudioBlockError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

#[derive(Clone, Copy)]
struct EncodedExponentRow {
    absolute: u8,
    groups: [u8; 72],
    group_count: usize,
}

#[derive(Clone, Copy)]
struct EncodedLfeExponentRow {
    absolute: u8,
    groups: [u8; 2],
}

#[derive(Clone, Copy)]
struct QuantizedMantissa {
    bap: u8,
    code: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MantissaField {
    value: u16,
    width: u8,
}

#[derive(Clone, Copy)]
struct PendingMantissaGroup {
    first_index: usize,
    count: usize,
    codes: [u16; 3],
}

impl PendingMantissaGroup {
    const EMPTY: Self = Self {
        first_index: 0,
        count: 0,
        codes: [0; 3],
    };
}

/// Produces one complete AC-3 `audblk` for `acmod=7`, `lfeon=1`.
///
/// ETSI TS 102 366 V1.4.1 defines the long analysis transform in clause 7.2.3, exponent syntax
/// in clauses 4.3.3 and 6.1, bit allocation in clause 6.2, and mantissa syntax in clauses 4.3.3
/// and 6.3. Coupling-enabled and short-transform inputs are explicitly outside this prototype.
/// The caller supplies the public 48 kHz BAP provider bytes and all sound-affecting controls.
pub fn produce_ac3_audio_block(
    input: Ac3AudioBlockInput<'_>,
    transform_parameters: &MathGeneratedTransformParameters,
    bap_provider_bytes: &[u8],
) -> Result<Ac3AudioBlockCandidate, Ac3AudioBlockError> {
    validate_input(&input)?;

    let fullband_spectra: [[f32; PCM_SAMPLES_PER_BLOCK]; FULLBAND_CHANNEL_COUNT] = (0
        ..FULLBAND_CHANNEL_COUNT)
        .map(|channel| {
            transform_long_block(
                &input.pcm.fullband[channel],
                transform_parameters.window_f32_bits(),
            )
        })
        .collect::<Result<Vec<[f32; PCM_SAMPLES_PER_BLOCK]>, Ac3AudioBlockError>>()?
        .try_into()
        .map_err(|_| Ac3AudioBlockError::BitCountOverflow)?;
    let mut fullband_coefficients = [[0.0_f32; FULLBAND_ENDMANT]; FULLBAND_CHANNEL_COUNT];
    for (destination, spectrum) in fullband_coefficients.iter_mut().zip(fullband_spectra) {
        destination.copy_from_slice(&spectrum[..FULLBAND_ENDMANT]);
    }
    let lfe_spectrum =
        transform_long_block(&input.pcm.lfe, transform_parameters.window_f32_bits())?;
    let mut lfe_coefficients = [0.0_f32; LFE_ENDMANT];
    lfe_coefficients.copy_from_slice(&lfe_spectrum[..LFE_ENDMANT]);

    let encoded_exponents = input
        .fullband_exponent_plans
        .iter()
        .map(|plan| encode_exponent_row(&plan.decoded_exponents, plan.strategy))
        .collect::<Result<Vec<EncodedExponentRow>, Ac3AudioBlockError>>()?
        .try_into()
        .map_err(|_| Ac3AudioBlockError::BitCountOverflow)?;
    let encoded_lfe_exponents = encode_lfe_exponent_row(input.lfe_decoded_exponents)?;

    let fullband_bap = (0..FULLBAND_CHANNEL_COUNT)
        .map(|channel| {
            allocate_bap_row(
                Some(bap_provider_bytes),
                &BapRowInput {
                    decoded_exponents: &input.fullband_exponent_plans[channel].decoded_exponents,
                    end_exclusive: FULLBAND_ENDMANT,
                    candidate: allocation_candidate(
                        input.allocation.coarse_snr_offset,
                        input.allocation.fullband_fine_snr_offsets[channel],
                    ),
                    slow_decay_code: input.allocation.slow_decay_code,
                    fast_decay_code: input.allocation.fast_decay_code,
                    slow_gain_code: input.allocation.slow_gain_code,
                    db_per_bit_code: input.allocation.db_per_bit_code,
                    floor_code: input.allocation.floor_code,
                    fast_gain_code: input.allocation.fullband_fast_gain_codes[channel],
                    caller_all_active_fine_zero: false,
                    profile: BapProfileInput {
                        sample_rate_hz: AC3_SAMPLE_RATE_HZ,
                        coupling: false,
                        delta: false,
                        aht: false,
                    },
                },
            )
            .and_then(|row| row.try_into().map_err(|_| BapError::InputError))
            .map_err(Ac3AudioBlockError::Bap)
        })
        .collect::<Result<Vec<[u8; FULLBAND_ENDMANT]>, Ac3AudioBlockError>>()?
        .try_into()
        .map_err(|_| Ac3AudioBlockError::BitCountOverflow)?;
    let lfe_bap: [u8; LFE_ENDMANT] = allocate_bap_row(
        Some(bap_provider_bytes),
        &BapRowInput {
            decoded_exponents: &input.lfe_decoded_exponents,
            end_exclusive: LFE_ENDMANT,
            candidate: allocation_candidate(
                input.allocation.coarse_snr_offset,
                input.allocation.lfe_fine_snr_offset,
            ),
            slow_decay_code: input.allocation.slow_decay_code,
            fast_decay_code: input.allocation.fast_decay_code,
            slow_gain_code: input.allocation.slow_gain_code,
            db_per_bit_code: input.allocation.db_per_bit_code,
            floor_code: input.allocation.floor_code,
            fast_gain_code: input.allocation.lfe_fast_gain_code,
            caller_all_active_fine_zero: false,
            profile: BapProfileInput {
                sample_rate_hz: AC3_SAMPLE_RATE_HZ,
                coupling: false,
                delta: false,
                aht: false,
            },
        },
    )?
    .try_into()
    .map_err(|_| Ac3AudioBlockError::Bap(BapError::InputError))?;

    let mantissa_fields = build_mantissa_fields(
        &fullband_coefficients,
        &input.fullband_exponent_plans,
        &fullband_bap,
        &lfe_coefficients,
        input.lfe_decoded_exponents,
        lfe_bap,
        input.quantizer_policy,
    )?;
    let total_bits = complete_audio_block_bit_count(
        &input,
        &encoded_exponents,
        encoded_lfe_exponents,
        &mantissa_fields,
    )?;
    let byte_count = usize::try_from(total_bits.div_ceil(8))
        .map_err(|_| Ac3AudioBlockError::BitCountOverflow)?;
    if byte_count > AC3_BLOCK_BUDGET_BYTES {
        return Err(Ac3AudioBlockError::BlockExceedsTargetFrame);
    }
    let mut bytes = vec![0_u8; byte_count];
    let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0))?;

    let prefix = write_i0_ac3_first_audio_block_prefix(
        &mut writer,
        I0Ac3AudioBlockPrefixInput {
            block_switch_flags: input.block_switch_flags,
            dither_flags: input.dither_flags,
            dynamic_range_code: input.dynamic_range_code,
        },
    )?;
    let _cplinu_receipt = write_i0_ac3_first_block_cplinu(&mut writer, prefix, input.cplinu)?;
    write_exponent_syntax(
        &mut writer,
        &input,
        &encoded_exponents,
        encoded_lfe_exponents,
    )?;
    write_bit_allocation_syntax(&mut writer, input.allocation)?;
    writer.write_bit(false)?; // deltbaie=0 is the caller-selected block-zero allocation mode.
    if let Some(skip_field) = input.skip_field {
        writer.write_bit(true)?;
        writer.write_unsigned(
            u32::try_from(skip_field.len()).map_err(|_| Ac3AudioBlockError::SkipFieldTooLarge)?,
            9,
        )?;
        writer.write_bytes(skip_field)?;
    } else {
        writer.write_bit(false)?;
    }
    for field in &mantissa_fields {
        writer.write_unsigned(u32::from(field.value), field.width)?;
    }

    let actual_bit_length = writer.cursor().bit_offset();
    if actual_bit_length != total_bits {
        return Err(Ac3AudioBlockError::BitCountOverflow);
    }
    Ok(Ac3AudioBlockCandidate {
        bytes,
        bit_length: actual_bit_length,
    })
}

fn validate_input(input: &Ac3AudioBlockInput<'_>) -> Result<(), Ac3AudioBlockError> {
    if input.cplinu != CallerOwnedAc3Cplinu::Independent {
        return Err(Ac3AudioBlockError::UnsupportedCoupling);
    }
    if input.block_switch_flags.iter().any(|flag| *flag) {
        return Err(Ac3AudioBlockError::UnsupportedBlockSwitch);
    }
    if input.delta_bit_allocation != Ac3DeltaBitAllocationInput::Absent {
        return Err(Ac3AudioBlockError::UnsupportedDeltaBitAllocation);
    }
    if input.skip_field.is_some_and(|bytes| bytes.len() > 511) {
        return Err(Ac3AudioBlockError::SkipFieldTooLarge);
    }
    if input
        .channel_bandwidth_codes
        .iter()
        .any(|code| *code != AC3_FULLBAND_CHBW_CODE)
    {
        return Err(Ac3AudioBlockError::UnsupportedChannelBandwidth);
    }
    if input.gain_range_codes.iter().any(|code| *code > 3) {
        return Err(Ac3AudioBlockError::InvalidGainRange);
    }
    if input.gain_range_codes.iter().any(|code| *code != 0) {
        return Err(Ac3AudioBlockError::UnsupportedGainRange);
    }
    for window in input
        .pcm
        .fullband
        .iter()
        .chain(std::iter::once(&input.pcm.lfe))
    {
        if window
            .previous_samples
            .iter()
            .chain(window.current_samples.iter())
            .any(|sample| !sample.is_finite() || !(-1.0..=1.0).contains(sample))
        {
            return Err(Ac3AudioBlockError::InvalidPcm);
        }
    }
    Ok(())
}

fn transform_long_block(
    input: &Ac3PcmBlockWindow,
    window_f32_bits: &[u32; PCM_SAMPLES_PER_BLOCK],
) -> Result<[f32; PCM_SAMPLES_PER_BLOCK], Ac3AudioBlockError> {
    let mut output = [0.0_f32; PCM_SAMPLES_PER_BLOCK];
    for (coefficient_index, output_value) in output.iter_mut().enumerate() {
        let mut sum = 0.0_f64;
        for sample_index in 0..PCM_WINDOW_SAMPLES {
            let sample = if sample_index < PCM_SAMPLES_PER_BLOCK {
                input.previous_samples[sample_index]
            } else {
                input.current_samples[sample_index - PCM_SAMPLES_PER_BLOCK]
            };
            let window_index = if sample_index < PCM_SAMPLES_PER_BLOCK {
                sample_index
            } else {
                PCM_WINDOW_SAMPLES - 1 - sample_index
            };
            let window_value = f64::from(f32::from_bits(window_f32_bits[window_index]));
            let phase = (std::f64::consts::PI / 1_024.0)
                * (2 * sample_index + 1) as f64
                * (2 * coefficient_index + 1) as f64
                + (std::f64::consts::PI / 4.0) * (2 * coefficient_index + 1) as f64;
            sum += f64::from(sample) * window_value * phase.cos();
        }
        let transformed = (-2.0 / PCM_WINDOW_SAMPLES as f64) * sum;
        #[allow(clippy::cast_possible_truncation)]
        let transformed = transformed as f32;
        if !transformed.is_finite() {
            return Err(Ac3AudioBlockError::InvalidPcm);
        }
        *output_value = transformed;
    }
    Ok(output)
}

fn encode_exponent_row(
    exponents: &[u8; FULLBAND_ENDMANT],
    strategy: Ac3ExponentStrategy,
) -> Result<EncodedExponentRow, Ac3AudioBlockError> {
    validate_exponent_values(exponents)?;
    let groups = exponent_group_count(FULLBAND_ENDMANT, strategy.group_span());
    let mut encoded_groups = [0_u8; 72];
    let mut previous = exponents[0];
    let group_extent = strategy.group_span() * 3;
    for (group_index, encoded_group) in encoded_groups.iter_mut().take(groups).enumerate() {
        let mut differences = [0_i8; 3];
        for (digit, difference_slot) in differences.iter_mut().enumerate() {
            let start = 1 + group_index * group_extent + digit * strategy.group_span();
            if start >= FULLBAND_ENDMANT {
                continue;
            }
            let current = exponents[start];
            for repeated in 1..strategy.group_span() {
                let repeated_index = start + repeated;
                if repeated_index < FULLBAND_ENDMANT && exponents[repeated_index] != current {
                    return Err(Ac3AudioBlockError::InvalidExponentPlan);
                }
            }
            let difference = i16::from(current) - i16::from(previous);
            *difference_slot =
                i8::try_from(difference).map_err(|_| Ac3AudioBlockError::InvalidExponentPlan)?;
            previous = current;
        }
        *encoded_group = pack_exponent_difference_group(&differences)?.value();
    }
    Ok(EncodedExponentRow {
        absolute: exponents[0],
        groups: encoded_groups,
        group_count: groups,
    })
}

fn encode_lfe_exponent_row(
    exponents: [u8; LFE_ENDMANT],
) -> Result<EncodedLfeExponentRow, Ac3AudioBlockError> {
    validate_exponent_values(&exponents)?;
    let mut groups = [0_u8; 2];
    let mut previous = exponents[0];
    for (group_index, group) in groups.iter_mut().enumerate() {
        let mut differences = [0_i8; 3];
        for (digit, difference) in differences.iter_mut().enumerate() {
            let current = exponents[1 + group_index * 3 + digit];
            *difference = i8::try_from(i16::from(current) - i16::from(previous))
                .map_err(|_| Ac3AudioBlockError::InvalidExponentPlan)?;
            previous = current;
        }
        *group = pack_exponent_difference_group(&differences)?.value();
    }
    Ok(EncodedLfeExponentRow {
        absolute: exponents[0],
        groups,
    })
}

fn validate_exponent_values(exponents: &[u8]) -> Result<(), Ac3AudioBlockError> {
    if exponents.is_empty() || exponents[0] > 15 || exponents.iter().any(|value| *value > 24) {
        return Err(Ac3AudioBlockError::InvalidExponentPlan);
    }
    Ok(())
}

fn exponent_group_count(end_mantissa: usize, group_span: usize) -> usize {
    (end_mantissa - 1).div_ceil(group_span * 3)
}

fn allocation_candidate(coarse: u8, fine: u8) -> u16 {
    (u16::from(coarse) << 4) | u16::from(fine)
}

fn build_mantissa_fields(
    fullband_coefficients: &[[f32; FULLBAND_ENDMANT]; FULLBAND_CHANNEL_COUNT],
    fullband_exponents: &[Ac3FullbandExponentPlan; FULLBAND_CHANNEL_COUNT],
    fullband_bap: &[[u8; FULLBAND_ENDMANT]; FULLBAND_CHANNEL_COUNT],
    lfe_coefficients: &[f32; LFE_ENDMANT],
    lfe_exponents: [u8; LFE_ENDMANT],
    lfe_bap: [u8; LFE_ENDMANT],
    policy: Ac3MantissaQuantizerPolicy,
) -> Result<Vec<MantissaField>, Ac3AudioBlockError> {
    let item_count = FULLBAND_CHANNEL_COUNT * FULLBAND_ENDMANT + LFE_ENDMANT;
    let mut items = Vec::with_capacity(item_count);
    for channel in 0..FULLBAND_CHANNEL_COUNT {
        for bin in 0..FULLBAND_ENDMANT {
            items.push(QuantizedMantissa {
                bap: fullband_bap[channel][bin],
                code: quantize_ac3_mantissa(
                    fullband_coefficients[channel][bin],
                    fullband_exponents[channel].decoded_exponents[bin],
                    fullband_bap[channel][bin],
                    policy,
                )?,
            });
        }
    }
    for bin in 0..LFE_ENDMANT {
        items.push(QuantizedMantissa {
            bap: lfe_bap[bin],
            code: quantize_ac3_mantissa(
                lfe_coefficients[bin],
                lfe_exponents[bin],
                lfe_bap[bin],
                policy,
            )?,
        });
    }
    pack_quantized_mantissas(&items)
}

fn quantize_ac3_mantissa(
    coefficient: f32,
    exponent: u8,
    bap: u8,
    policy: Ac3MantissaQuantizerPolicy,
) -> Result<u16, Ac3AudioBlockError> {
    if bap > 15 || !coefficient.is_finite() {
        return Err(Ac3AudioBlockError::CoefficientOutsideNormalizedRange);
    }
    if bap == 0 {
        return Ok(0);
    }
    let normalized = f64::from(coefficient) * 2.0_f64.powi(i32::from(exponent));
    if !normalized.is_finite() || !(-1.0..=1.0).contains(&normalized) {
        return Err(Ac3AudioBlockError::CoefficientOutsideNormalizedRange);
    }
    match policy {
        Ac3MantissaQuantizerPolicy::NearestTiesEven => {
            if let Some(levels) = symmetric_quantizer_levels(bap) {
                let mut code = 0_u16;
                for boundary in 1..levels {
                    let threshold = -1.0 + (2.0 * f64::from(boundary)) / f64::from(levels);
                    match normalized.total_cmp(&threshold) {
                        std::cmp::Ordering::Greater => code = boundary,
                        std::cmp::Ordering::Equal if boundary % 2 == 0 => code = boundary,
                        std::cmp::Ordering::Less | std::cmp::Ordering::Equal => break,
                    }
                }
                return Ok(code);
            }
            let width =
                mantissa_width(bap).ok_or(Ac3AudioBlockError::CoefficientOutsideNormalizedRange)?;
            let signed_levels = 1_i64 << (width - 1);
            let rounded = (normalized * signed_levels as f64).round_ties_even();
            let minimum = -(signed_levels as f64);
            let maximum = (signed_levels - 1) as f64;
            #[allow(clippy::cast_possible_truncation)]
            let bounded = rounded.clamp(minimum, maximum) as i64;
            let code = if bounded < 0 {
                (1_i64 << width) + bounded
            } else {
                bounded
            };
            u16::try_from(code).map_err(|_| Ac3AudioBlockError::CoefficientOutsideNormalizedRange)
        }
    }
}

fn symmetric_quantizer_levels(bap: u8) -> Option<u16> {
    match bap {
        1 => Some(3),
        2 => Some(5),
        3 => Some(7),
        4 => Some(11),
        5 => Some(15),
        _ => None,
    }
}

fn mantissa_width(bap: u8) -> Option<u8> {
    match bap {
        0 => Some(0),
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

fn pack_quantized_mantissas(
    items: &[QuantizedMantissa],
) -> Result<Vec<MantissaField>, Ac3AudioBlockError> {
    let mut fields = vec![None; items.len()];
    let mut groups = [PendingMantissaGroup::EMPTY; 3];
    for (item_index, item) in items.iter().enumerate() {
        if let Some(group_index) = group_index(item.bap) {
            let group = &mut groups[group_index];
            if group.count == 0 {
                group.first_index = item_index;
            }
            group.codes[group.count] = item.code;
            group.count += 1;
            if group.count == usize::from(group_size(item.bap)) {
                fields[group.first_index] = Some(MantissaField {
                    value: grouped_value(item.bap, group.codes),
                    width: grouped_width(item.bap),
                });
                *group = PendingMantissaGroup::EMPTY;
            }
        } else if item.bap != 0 {
            fields[item_index] = Some(MantissaField {
                value: item.code,
                width: mantissa_width(item.bap)
                    .ok_or(Ac3AudioBlockError::CoefficientOutsideNormalizedRange)?,
            });
        }
    }

    for (group_index, bap) in [1_u8, 2, 4].into_iter().enumerate() {
        let group = &mut groups[group_index];
        if group.count == 0 {
            continue;
        }
        let fill = match bap {
            1 => 1,
            2 => 2,
            4 => 5,
            _ => 0,
        };
        for code in group
            .codes
            .iter_mut()
            .skip(group.count)
            .take(3 - group.count)
        {
            *code = fill;
        }
        fields[group.first_index] = Some(MantissaField {
            value: grouped_value(bap, group.codes),
            width: grouped_width(bap),
        });
    }

    Ok(fields.into_iter().flatten().collect())
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

fn grouped_value(bap: u8, codes: [u16; 3]) -> u16 {
    match bap {
        1 => 9 * codes[0] + 3 * codes[1] + codes[2],
        2 => 25 * codes[0] + 5 * codes[1] + codes[2],
        4 => 11 * codes[0] + codes[1],
        _ => 0,
    }
}

fn complete_audio_block_bit_count(
    input: &Ac3AudioBlockInput<'_>,
    exponent_rows: &[EncodedExponentRow; FULLBAND_CHANNEL_COUNT],
    lfe_exponents: EncodedLfeExponentRow,
    mantissas: &[MantissaField],
) -> Result<u32, Ac3AudioBlockError> {
    let dynamic_range_bits = if input.dynamic_range_code.is_some() {
        8_u32
    } else {
        0
    };
    let mut total = 5_u32 + 5 + 1 + dynamic_range_bits + 1 + 1;
    total = checked_add(total, 5 * 2 + 1)?;
    for (channel, row) in exponent_rows.iter().enumerate() {
        total = checked_add(
            total,
            6 + 4
                + 2
                + u32::try_from(row.group_count * 7)
                    .map_err(|_| Ac3AudioBlockError::BitCountOverflow)?,
        )?;
        if input.channel_bandwidth_codes[channel] != AC3_FULLBAND_CHBW_CODE {
            return Err(Ac3AudioBlockError::UnsupportedChannelBandwidth);
        }
    }
    total = checked_add(
        total,
        4 + u32::try_from(lfe_exponents.groups.len() * 7)
            .map_err(|_| Ac3AudioBlockError::BitCountOverflow)?,
    )?;
    total = checked_add(total, 1 + 2 + 2 + 2 + 2 + 3)?;
    total = checked_add(total, 1 + 6 + 5 * (4 + 3) + 4 + 3)?;
    total = checked_add(total, 1)?; // deltbaie
    let skip_bits = match input.skip_field {
        Some(bytes) => {
            let payload_bits = u32::try_from(bytes.len())
                .map_err(|_| Ac3AudioBlockError::BitCountOverflow)?
                .checked_mul(8)
                .ok_or(Ac3AudioBlockError::BitCountOverflow)?;
            checked_add(10, payload_bits)?
        }
        None => 1,
    };
    total = checked_add(total, skip_bits)?;
    for field in mantissas {
        total = checked_add(total, u32::from(field.width))?;
    }
    Ok(total)
}

fn checked_add(left: u32, right: u32) -> Result<u32, Ac3AudioBlockError> {
    left.checked_add(right)
        .ok_or(Ac3AudioBlockError::BitCountOverflow)
}

fn write_exponent_syntax(
    writer: &mut FrameBitWriter<'_>,
    input: &Ac3AudioBlockInput<'_>,
    fullband: &[EncodedExponentRow; FULLBAND_CHANNEL_COUNT],
    lfe: EncodedLfeExponentRow,
) -> Result<(), Ac3AudioBlockError> {
    for plan in &input.fullband_exponent_plans {
        writer.write_unsigned(u32::from(plan.strategy.wire_code()), 2)?;
    }
    writer.write_bit(true)?; // lfeexpstr=1 is required for block zero.
    for code in input.channel_bandwidth_codes {
        writer.write_unsigned(u32::from(code), 6)?;
    }
    for (channel, row) in fullband.iter().enumerate() {
        writer.write_unsigned(u32::from(row.absolute), 4)?;
        for group in row.groups.iter().take(row.group_count) {
            writer.write_unsigned(u32::from(*group), 7)?;
        }
        writer.write_unsigned(u32::from(input.gain_range_codes[channel]), 2)?;
    }
    writer.write_unsigned(u32::from(lfe.absolute), 4)?;
    for group in lfe.groups {
        writer.write_unsigned(u32::from(group), 7)?;
    }
    Ok(())
}

fn write_bit_allocation_syntax(
    writer: &mut FrameBitWriter<'_>,
    input: Ac3BlockAllocationControls,
) -> Result<(), Ac3AudioBlockError> {
    writer.write_bit(true)?; // baie is mandatory in block zero.
    writer.write_unsigned(u32::from(input.slow_decay_code), 2)?;
    writer.write_unsigned(u32::from(input.fast_decay_code), 2)?;
    writer.write_unsigned(u32::from(input.slow_gain_code), 2)?;
    writer.write_unsigned(u32::from(input.db_per_bit_code), 2)?;
    writer.write_unsigned(u32::from(input.floor_code), 3)?;

    writer.write_bit(true)?; // snroffste is mandatory in block zero.
    writer.write_unsigned(u32::from(input.coarse_snr_offset), 6)?;
    for channel in 0..FULLBAND_CHANNEL_COUNT {
        writer.write_unsigned(u32::from(input.fullband_fine_snr_offsets[channel]), 4)?;
        writer.write_unsigned(u32::from(input.fullband_fast_gain_codes[channel]), 3)?;
    }
    writer.write_unsigned(u32::from(input.lfe_fine_snr_offset), 4)?;
    writer.write_unsigned(u32::from(input.lfe_fast_gain_code), 3)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        Ac3MantissaQuantizerPolicy, Ac3PcmBlockWindow, MantissaField, QuantizedMantissa,
        grouped_value, grouped_width, pack_quantized_mantissas, quantize_ac3_mantissa,
        transform_long_block,
    };
    use crate::transform::math_generated_parameters::MathGeneratedTransformParameters;

    #[test]
    fn long_transform_matches_the_etsi_scalar_formula_for_one_hot_input() {
        let parameters = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
        let mut input = Ac3PcmBlockWindow {
            previous_samples: [0.0; 256],
            current_samples: [0.0; 256],
        };
        input.current_samples[17] = 0.25;
        let output = transform_long_block(&input, parameters.window_f32_bits()).unwrap();
        let sample_index = 256 + 17;
        for coefficient_index in [0, 1, 63, 127, 255] {
            let window_index = 511 - sample_index;
            let window = f64::from(f32::from_bits(parameters.window_f32_bits()[window_index]));
            let phase = (std::f64::consts::PI / 1_024.0)
                * (2 * sample_index + 1) as f64
                * (2 * coefficient_index + 1) as f64
                + (std::f64::consts::PI / 4.0) * (2 * coefficient_index + 1) as f64;
            let expected = (-2.0 / 512.0) * 0.25_f64 * window * phase.cos();
            assert!((f64::from(output[coefficient_index]) - expected).abs() < 1.0e-7);
        }
    }

    #[test]
    fn long_transform_rejects_non_finite_pcm_before_transforming() {
        let parameters = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
        let mut input = Ac3PcmBlockWindow {
            previous_samples: [0.0; 256],
            current_samples: [0.0; 256],
        };
        input.current_samples[0] = f32::INFINITY;
        assert_eq!(
            transform_long_block(&input, parameters.window_f32_bits()),
            Err(super::Ac3AudioBlockError::InvalidPcm)
        );
    }

    #[test]
    fn mathematical_window_matches_published_etsi_table_landmarks() {
        let parameters = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
        for (index, expected) in [
            (0, 0.00014_f64),
            (1, 0.00024),
            (2, 0.00037),
            (100, 0.44030),
            (127, 0.70269),
            (128, 0.71150),
            (255, 1.00000),
        ] {
            let window = f64::from(f32::from_bits(parameters.window_f32_bits()[index]));
            assert_eq!((window * 100_000.0).round() / 100_000.0, expected);
        }
    }

    #[test]
    fn nearest_ties_even_policy_is_explicit_at_asymmetric_code_boundaries() {
        let policy = Ac3MantissaQuantizerPolicy::NearestTiesEven;
        assert_eq!(quantize_ac3_mantissa(1.0 / 32.0, 0, 6, policy), Ok(0));
        assert_eq!(quantize_ac3_mantissa(3.0 / 32.0, 0, 6, policy), Ok(2));
        assert_eq!(quantize_ac3_mantissa(1.0, 0, 3, policy), Ok(6));
        assert_eq!(quantize_ac3_mantissa(-1.0, 0, 3, policy), Ok(0));
    }

    #[test]
    fn grouped_mantissas_continue_across_each_fullband_and_lfe_boundary() {
        let block_item_count = 5 * 217 + 7;
        let channel_boundaries = [217, 434, 651, 868, 1_085];
        for boundary in channel_boundaries {
            for (bap, codes, expected_width, expected_value) in [
                (1_u8, [0_u16, 1, 2], 5_u8, 5_u16),
                (2_u8, [0_u16, 1, 2], 7_u8, 7_u16),
                (4_u8, [0_u16, 1, 2], 7_u8, 1_u16),
            ] {
                let group_size = if bap == 4 { 2 } else { 3 };
                let start = boundary - (group_size - 1);
                let mut items = vec![QuantizedMantissa { bap: 0, code: 0 }; block_item_count];
                for offset in 0..group_size {
                    items[start + offset] = QuantizedMantissa {
                        bap,
                        code: codes[offset],
                    };
                }
                let fields = pack_quantized_mantissas(&items).unwrap();
                assert_eq!(fields.len(), 1);
                assert_eq!(
                    fields[0],
                    MantissaField {
                        value: expected_value,
                        width: expected_width,
                    }
                );
                assert_eq!(expected_width, grouped_width(bap));
                assert_eq!(expected_value, grouped_value(bap, codes));
            }
        }
    }
}
