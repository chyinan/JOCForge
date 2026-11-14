//! Functional Core: production-disabled AC-3 3/2+LFE audio-block prototype.
//!
//! The prototype serializes one block-zero audblk from caller PCM windows and explicit coding
//! controls. It supports long transforms, independent channels, and the 48 kHz / 217-bin
//! full-band row. I0 frame assembly, six-block state, and publication remain separate.

use super::{
    bap::{
        BapError, BapProfileInput, BapRowInput, allocate_bap_row, allocate_bap_row_with_parameters,
    },
    bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
    exponent_packing::{ExponentPackingError, pack_exponent_difference_group},
    i0_frame_assembler::{
        CallerOwnedAc3Cplinu, I0Ac3AudioBlockPrefixError, I0Ac3AudioBlockPrefixInput,
        I0Ac3CplinuWriteError, OpaqueI0Ac3BitSpan, write_i0_ac3_audio_block_prefix,
        write_i0_ac3_first_audio_block_prefix, write_i0_ac3_first_block_cplinu,
    },
};
use crate::{
    analysis::source_frame::SharedAnalysisIdentity,
    codec::selected_frame_trial::{BlockExponentPlan, SelectedSourceNumericPlan},
    transform::{
        fullband_exponent_pipeline::NumericPlanReceipt, long_mdct::forward_long_mdct_512,
        math_generated_parameters::MathGeneratedTransformParameters,
    },
};

const PCM_SAMPLES_PER_BLOCK: usize = 256;
const FULLBAND_CHANNEL_COUNT: usize = 5;
const FULLBAND_ENDMANT: usize = 217;
const LFE_ENDMANT: usize = 7;
const AC3_SAMPLE_RATE_HZ: u32 = 48_000;
const AC3_FULLBAND_CHBW_CODE: u8 = 48;
const AC3_BLOCK_BUDGET_BYTES: usize = 2_560;
const I0_AUDIO_BLOCK_COUNT: usize = 6;

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

/// The zero-based position of one AC-3 I0 audio block in its syncframe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct I0AudioBlockIndex(u8);

impl I0AudioBlockIndex {
    pub const fn new(value: u8) -> Option<Self> {
        if value < I0_AUDIO_BLOCK_COUNT as u8 {
            Some(Self(value))
        } else {
            None
        }
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

/// One caller-selected ten-bit SNR/BAP candidate, split into AC-3 coarse and fine fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct I0BapCandidateQ {
    value: u16,
    coarse: u8,
    fine: u8,
}

impl I0BapCandidateQ {
    pub fn new(value: u16) -> Option<Self> {
        if value > 1_023 {
            return None;
        }
        let coarse = u8::try_from(value >> 4).ok()?;
        let fine = u8::try_from(value & 0x0f).ok()?;
        Some(Self {
            value,
            coarse,
            fine,
        })
    }

    pub const fn value(self) -> u16 {
        self.value
    }

    const fn coarse(self) -> u8 {
        self.coarse
    }

    const fn fine(self) -> u8 {
        self.fine
    }
}

/// Caller-owned aggregate bit budget for the six I0 audio-block spans.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct I0AudioBlockSeriesBudget {
    available_bits: u32,
}

impl I0AudioBlockSeriesBudget {
    pub const fn new(available_bits: u32) -> Self {
        Self { available_bits }
    }

    pub const fn available_bits(self) -> u32 {
        self.available_bits
    }
}

/// Caller-owned AC-3 block controls and BAP policy for one source block.
///
/// The `candidate_q` value is used for every channel row in this block. Its high six bits are
/// serialized as `csnroffst`; its low four bits become each channel's fine SNR offset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct I0AudioBlockControlPolicy {
    pub candidate_q: I0BapCandidateQ,
    pub slow_decay_code: u8,
    pub fast_decay_code: u8,
    pub slow_gain_code: u8,
    pub floor_code: u8,
    pub fast_gain_codes: [u8; FULLBAND_CHANNEL_COUNT + 1],
    pub caller_all_active_fine_zero: bool,
    pub dither_flags: [bool; FULLBAND_CHANNEL_COUNT],
    pub dynamic_range_code: Option<u8>,
}

/// Six explicit, ordered block policies supplied by the source-stage caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct I0AudioBlockSeriesPolicy {
    pub blocks: [I0AudioBlockControlPolicy; I0_AUDIO_BLOCK_COUNT],
    pub cplinu: CallerOwnedAc3Cplinu,
    pub delta_bit_allocation: Ac3DeltaBitAllocationInput,
    pub quantizer_policy: Ac3MantissaQuantizerPolicy,
}

/// Auditable assumptions used by this candidate-only projection from the selected F12 plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum I0AudioBlockAssumptionId {
    SelectedF12SpectrumAsI0Ac3LongBlockSyntaxV1,
    SelectedF09F10ExponentStrategyWithD15RecodeOnExponentChangeV1,
}

impl I0AudioBlockAssumptionId {
    pub const fn id(self) -> &'static str {
        match self {
            Self::SelectedF12SpectrumAsI0Ac3LongBlockSyntaxV1 => {
                "PROVISIONAL_I0_SELECTED_F12_51_SPECTRUM_LONG_BLOCKS_V1"
            }
            Self::SelectedF09F10ExponentStrategyWithD15RecodeOnExponentChangeV1 => {
                "PROVISIONAL_I0_F09_F10_EXPSTR_D15_RECODE_ON_EXPONENT_CHANGE_V1"
            }
        }
    }
}

/// Caller-selected allocation controls and the validated provider used for all six blocks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct I0BapPolicyReceipt {
    block_controls: [I0AudioBlockControlPolicy; I0_AUDIO_BLOCK_COUNT],
    db_per_bit_codes: [u8; I0_AUDIO_BLOCK_COUNT],
    bap_provider_sha256: [u8; 32],
}

impl I0BapPolicyReceipt {
    pub const fn block_controls(&self) -> &[I0AudioBlockControlPolicy; I0_AUDIO_BLOCK_COUNT] {
        &self.block_controls
    }

    pub const fn db_per_bit_codes(&self) -> &[u8; I0_AUDIO_BLOCK_COUNT] {
        &self.db_per_bit_codes
    }

    pub const fn bap_provider_sha256(&self) -> &[u8; 32] {
        &self.bap_provider_sha256
    }
}

/// Receipt for one source-bound, six-block I0 audio proposal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct I0AudioBlockSeriesReceipt {
    source_identity: SharedAnalysisIdentity,
    block_order: [I0AudioBlockIndex; I0_AUDIO_BLOCK_COUNT],
    block_bit_lengths: [u32; I0_AUDIO_BLOCK_COUNT],
    bap_policy: I0BapPolicyReceipt,
    cplinu: CallerOwnedAc3Cplinu,
    delta_bit_allocation: Ac3DeltaBitAllocationInput,
    quantizer_policy: Ac3MantissaQuantizerPolicy,
    assumptions: [I0AudioBlockAssumptionId; 2],
}

impl I0AudioBlockSeriesReceipt {
    pub const fn source_identity(&self) -> &SharedAnalysisIdentity {
        &self.source_identity
    }

    pub const fn block_order(&self) -> &[I0AudioBlockIndex; I0_AUDIO_BLOCK_COUNT] {
        &self.block_order
    }

    pub const fn block_bit_lengths(&self) -> &[u32; I0_AUDIO_BLOCK_COUNT] {
        &self.block_bit_lengths
    }

    pub const fn bap_policy(&self) -> &I0BapPolicyReceipt {
        &self.bap_policy
    }

    pub const fn cplinu(&self) -> CallerOwnedAc3Cplinu {
        self.cplinu
    }

    pub const fn delta_bit_allocation(&self) -> Ac3DeltaBitAllocationInput {
        self.delta_bit_allocation
    }

    pub const fn quantizer_policy(&self) -> Ac3MantissaQuantizerPolicy {
        self.quantizer_policy
    }

    pub const fn assumptions(&self) -> &[I0AudioBlockAssumptionId; 2] {
        &self.assumptions
    }
}

/// Noncommitting six-audblk candidate in AC-3 I0 block order.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct I0AudioBlockSeriesCandidate {
    blocks: [Ac3AudioBlockCandidate; I0_AUDIO_BLOCK_COUNT],
    receipt: I0AudioBlockSeriesReceipt,
}

impl I0AudioBlockSeriesCandidate {
    pub const fn receipt(&self) -> &I0AudioBlockSeriesReceipt {
        &self.receipt
    }

    pub fn block(&self, index: I0AudioBlockIndex) -> &Ac3AudioBlockCandidate {
        &self.blocks[usize::from(index.value())]
    }

    pub fn blocks(&self) -> &[Ac3AudioBlockCandidate; I0_AUDIO_BLOCK_COUNT] {
        &self.blocks
    }

    pub fn i0_bit_spans(&self) -> [OpaqueI0Ac3BitSpan<'_>; I0_AUDIO_BLOCK_COUNT] {
        std::array::from_fn(|index| self.blocks[index].as_i0_bit_span())
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum I0AudioBlockSeriesError {
    BapProviderIdentityMismatch,
    ShortTransformUnsupported {
        block: u8,
        channel: u8,
    },
    FullbandExponentStrategyUnsupported {
        block: u8,
        channel: u8,
        value: u8,
    },
    FullbandExponentReuseMismatch {
        block: u8,
        channel: u8,
    },
    BlockZeroFullbandExponentReuse {
        channel: u8,
    },
    LfeExponentStrategyUnsupported {
        block: u8,
        value: u8,
    },
    LfeExponentReuseMismatch {
        block: u8,
    },
    BlockZeroLfeExponentReuse,
    ChannelBandwidthUnsupported {
        block: u8,
        channel: u8,
        value: u8,
    },
    GainRangeUnsupported {
        block: u8,
        channel: u8,
        value: u8,
    },
    Bap(BapError),
    Block(Ac3AudioBlockError),
    BitWriter(FrameBitWriterError),
    BudgetExceedsI0Frame {
        available_bits: u32,
    },
    AudioBlockBudgetExceeded {
        required_bits: u32,
        available_bits: u32,
    },
    BitCountOverflow,
}

impl From<BapError> for I0AudioBlockSeriesError {
    fn from(error: BapError) -> Self {
        Self::Bap(error)
    }
}

impl From<Ac3AudioBlockError> for I0AudioBlockSeriesError {
    fn from(error: Ac3AudioBlockError) -> Self {
        Self::Block(error)
    }
}

impl From<FrameBitWriterError> for I0AudioBlockSeriesError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
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

/// Produces six candidate I0 `audblk` spans from one source-bound selected numeric plan.
///
/// The selected F12 5.1 coefficient/exponent rows are reused as the I0 spectrum with long-block
/// syntax under the explicit `SelectedF12SpectrumAsI0Ac3LongBlockSyntaxV1` assumption. F09/F10 block strategy values are
/// interpreted as AC-3 `chexpstr`/`lfeexpstr` values under the separate
/// `SelectedF09F10ExponentStrategyWithD15RecodeOnExponentChangeV1` assumption. Strategy zero does
/// not serialize the caller's packed groups; it reuses prior bandwidth and gain-range fields. If
/// decoded exponents differ, the current values are emitted as an explicit D15 row. Explicit
/// strategies serialize the caller's 2-bit gain-range code. Coefficients are unchanged.
pub fn produce_i0_audio_block_series(
    numeric_plan: &NumericPlanReceipt<'_, SelectedSourceNumericPlan>,
    policy: I0AudioBlockSeriesPolicy,
    budget: I0AudioBlockSeriesBudget,
) -> Result<I0AudioBlockSeriesCandidate, I0AudioBlockSeriesError> {
    let plan = numeric_plan.plan();
    if plan.bap_provider_sha256 != plan.bap_parameters.provider_sha256() {
        return Err(I0AudioBlockSeriesError::BapProviderIdentityMismatch);
    }
    if budget.available_bits() > (AC3_BLOCK_BUDGET_BYTES as u32) * 8 {
        return Err(I0AudioBlockSeriesError::BudgetExceedsI0Frame {
            available_bits: budget.available_bits(),
        });
    }
    if policy.cplinu != CallerOwnedAc3Cplinu::Independent {
        return Err(I0AudioBlockSeriesError::Block(
            Ac3AudioBlockError::UnsupportedCoupling,
        ));
    }
    if policy.delta_bit_allocation != Ac3DeltaBitAllocationInput::Absent {
        return Err(I0AudioBlockSeriesError::Block(
            Ac3AudioBlockError::UnsupportedDeltaBitAllocation,
        ));
    }

    let mut blocks = Vec::with_capacity(I0_AUDIO_BLOCK_COUNT);
    let mut block_bit_lengths = [0_u32; I0_AUDIO_BLOCK_COUNT];
    let mut previous_fullband_exponents: Option<[[u8; FULLBAND_ENDMANT]; FULLBAND_CHANNEL_COUNT]> =
        None;
    let mut previous_bandwidth_codes: Option<[u8; FULLBAND_CHANNEL_COUNT]> = None;
    let mut previous_gain_ranges: Option<[u8; FULLBAND_CHANNEL_COUNT]> = None;
    let mut previous_lfe_exponents: Option<[u8; LFE_ENDMANT]> = None;
    let mut total_audio_bits = 0_u32;

    for (block_index, block_bit_length) in block_bit_lengths.iter_mut().enumerate() {
        let block_number =
            u8::try_from(block_index).map_err(|_| I0AudioBlockSeriesError::BitCountOverflow)?;
        let exponent_plan = &plan.exponent_plan_by_block[block_index];
        let source_coefficients = &plan.coefficients_by_block[block_index];
        let block_policy = policy.blocks[block_index];

        let mut encoded_fullband: [Option<EncodedExponentRow>; FULLBAND_CHANNEL_COUNT] =
            [None; FULLBAND_CHANNEL_COUNT];
        let mut fullband_wire_strategy_codes = [0_u8; FULLBAND_CHANNEL_COUNT];
        let mut quantizer_exponent_plans = [Ac3FullbandExponentPlan {
            strategy: Ac3ExponentStrategy::D15,
            decoded_exponents: [0; FULLBAND_ENDMANT],
        }; FULLBAND_CHANNEL_COUNT];
        let mut current_fullband_exponents = [[0_u8; FULLBAND_ENDMANT]; FULLBAND_CHANNEL_COUNT];
        let mut current_bandwidth_codes = [0_u8; FULLBAND_CHANNEL_COUNT];
        let mut current_gain_ranges = [0_u8; FULLBAND_CHANNEL_COUNT];

        for channel in 0..FULLBAND_CHANNEL_COUNT {
            let lane = &exponent_plan.fullband[channel];
            current_fullband_exponents[channel] = lane.decoded_exponents;
            let quantizer_strategy = match lane.strategy {
                0 => {
                    if block_index == 0 {
                        return Err(I0AudioBlockSeriesError::BlockZeroFullbandExponentReuse {
                            channel: u8::try_from(channel)
                                .map_err(|_| I0AudioBlockSeriesError::BitCountOverflow)?,
                        });
                    }
                    let previous = previous_fullband_exponents.as_ref().ok_or(
                        I0AudioBlockSeriesError::BlockZeroFullbandExponentReuse {
                            channel: u8::try_from(channel)
                                .map_err(|_| I0AudioBlockSeriesError::BitCountOverflow)?,
                        },
                    )?;
                    let previous_bandwidth = previous_bandwidth_codes.as_ref().ok_or(
                        I0AudioBlockSeriesError::BlockZeroFullbandExponentReuse {
                            channel: u8::try_from(channel)
                                .map_err(|_| I0AudioBlockSeriesError::BitCountOverflow)?,
                        },
                    )?;
                    let previous_gain = previous_gain_ranges.as_ref().ok_or(
                        I0AudioBlockSeriesError::BlockZeroFullbandExponentReuse {
                            channel: u8::try_from(channel)
                                .map_err(|_| I0AudioBlockSeriesError::BitCountOverflow)?,
                        },
                    )?;
                    current_bandwidth_codes[channel] = previous_bandwidth[channel];
                    current_gain_ranges[channel] = previous_gain[channel];
                    if lane.decoded_exponents == previous[channel] {
                        (Ac3ExponentStrategy::D15, 0)
                    } else {
                        let strategy = Ac3ExponentStrategy::D15;
                        let encoded = encode_exponent_row(&lane.decoded_exponents, strategy)?;
                        if lane.base_exponent != encoded.absolute {
                            return Err(I0AudioBlockSeriesError::Block(
                                Ac3AudioBlockError::InvalidExponentPlan,
                            ));
                        }
                        encoded_fullband[channel] = Some(encoded);
                        (strategy, strategy.wire_code())
                    }
                }
                value @ 1..=3 => {
                    current_bandwidth_codes[channel] = lane.bandwidth_code;
                    current_gain_ranges[channel] = lane.gain_range;
                    if lane.bandwidth_code != AC3_FULLBAND_CHBW_CODE {
                        return Err(I0AudioBlockSeriesError::ChannelBandwidthUnsupported {
                            block: block_number,
                            channel: u8::try_from(channel)
                                .map_err(|_| I0AudioBlockSeriesError::BitCountOverflow)?,
                            value: lane.bandwidth_code,
                        });
                    }
                    if lane.gain_range > 3 {
                        return Err(I0AudioBlockSeriesError::GainRangeUnsupported {
                            block: block_number,
                            channel: u8::try_from(channel)
                                .map_err(|_| I0AudioBlockSeriesError::BitCountOverflow)?,
                            value: lane.gain_range,
                        });
                    }
                    let strategy = ac3_exponent_strategy(value).ok_or(
                        I0AudioBlockSeriesError::FullbandExponentStrategyUnsupported {
                            block: block_number,
                            channel: u8::try_from(channel)
                                .map_err(|_| I0AudioBlockSeriesError::BitCountOverflow)?,
                            value,
                        },
                    )?;
                    let encoded = encode_exponent_row(&lane.decoded_exponents, strategy)?;
                    if lane.base_exponent != encoded.absolute
                        || lane.packed_groups.len() != encoded.group_count
                        || lane
                            .packed_groups
                            .iter()
                            .zip(encoded.groups.iter())
                            .any(|(actual, expected)| actual.value() != *expected)
                    {
                        return Err(I0AudioBlockSeriesError::Block(
                            Ac3AudioBlockError::InvalidExponentPlan,
                        ));
                    }
                    encoded_fullband[channel] = Some(encoded);
                    (strategy, value)
                }
                value => {
                    return Err(
                        I0AudioBlockSeriesError::FullbandExponentStrategyUnsupported {
                            block: block_number,
                            channel: u8::try_from(channel)
                                .map_err(|_| I0AudioBlockSeriesError::BitCountOverflow)?,
                            value,
                        },
                    );
                }
            };
            let (quantizer_strategy, wire_strategy_code) = quantizer_strategy;
            fullband_wire_strategy_codes[channel] = wire_strategy_code;
            quantizer_exponent_plans[channel] = Ac3FullbandExponentPlan {
                strategy: quantizer_strategy,
                decoded_exponents: lane.decoded_exponents,
            };
        }

        let lfe_exponents = exponent_plan.lfe.decoded_exponents;
        let encoded_lfe = match exponent_plan.lfe.strategy {
            0 => {
                if block_index == 0 {
                    return Err(I0AudioBlockSeriesError::BlockZeroLfeExponentReuse);
                }
                if previous_lfe_exponents.as_ref() != Some(&lfe_exponents) {
                    return Err(I0AudioBlockSeriesError::LfeExponentReuseMismatch {
                        block: block_number,
                    });
                }
                None
            }
            1 => {
                let encoded = encode_lfe_exponent_row(lfe_exponents)?;
                if exponent_plan.lfe.base_exponent != encoded.absolute
                    || exponent_plan
                        .lfe
                        .packed_groups
                        .iter()
                        .zip(encoded.groups.iter())
                        .any(|(actual, expected)| actual.value() != *expected)
                {
                    return Err(I0AudioBlockSeriesError::Block(
                        Ac3AudioBlockError::InvalidExponentPlan,
                    ));
                }
                Some(encoded)
            }
            value => {
                return Err(I0AudioBlockSeriesError::LfeExponentStrategyUnsupported {
                    block: block_number,
                    value,
                });
            }
        };

        let mut fullband_bap_rows = Vec::with_capacity(FULLBAND_CHANNEL_COUNT);
        for channel in 0..FULLBAND_CHANNEL_COUNT {
            let q = block_policy.candidate_q.value();
            let row = allocate_bap_row_with_parameters(
                plan.bap_parameters.as_ref(),
                &BapRowInput {
                    decoded_exponents: &exponent_plan.fullband[channel].decoded_exponents,
                    end_exclusive: FULLBAND_ENDMANT,
                    candidate: q,
                    slow_decay_code: block_policy.slow_decay_code,
                    fast_decay_code: block_policy.fast_decay_code,
                    slow_gain_code: block_policy.slow_gain_code,
                    db_per_bit_code: plan.db_per_bit_codes[block_index],
                    floor_code: block_policy.floor_code,
                    fast_gain_code: block_policy.fast_gain_codes[channel],
                    caller_all_active_fine_zero: block_policy.caller_all_active_fine_zero,
                    profile: BapProfileInput {
                        sample_rate_hz: AC3_SAMPLE_RATE_HZ,
                        coupling: false,
                        delta: false,
                        aht: false,
                    },
                },
            )?
            .try_into()
            .map_err(|_| I0AudioBlockSeriesError::BitCountOverflow)?;
            fullband_bap_rows.push(row);
        }
        let fullband_bap_rows: [[u8; FULLBAND_ENDMANT]; FULLBAND_CHANNEL_COUNT] = fullband_bap_rows
            .try_into()
            .map_err(|_| I0AudioBlockSeriesError::BitCountOverflow)?;
        let lfe_bap: [u8; LFE_ENDMANT] = allocate_bap_row_with_parameters(
            plan.bap_parameters.as_ref(),
            &BapRowInput {
                decoded_exponents: &lfe_exponents,
                end_exclusive: LFE_ENDMANT,
                candidate: block_policy.candidate_q.value(),
                slow_decay_code: block_policy.slow_decay_code,
                fast_decay_code: block_policy.fast_decay_code,
                slow_gain_code: block_policy.slow_gain_code,
                db_per_bit_code: plan.db_per_bit_codes[block_index],
                floor_code: block_policy.floor_code,
                fast_gain_code: block_policy.fast_gain_codes[FULLBAND_CHANNEL_COUNT],
                caller_all_active_fine_zero: block_policy.caller_all_active_fine_zero,
                profile: BapProfileInput {
                    sample_rate_hz: AC3_SAMPLE_RATE_HZ,
                    coupling: false,
                    delta: false,
                    aht: false,
                },
            },
        )?
        .try_into()
        .map_err(|_| I0AudioBlockSeriesError::BitCountOverflow)?;

        let coefficients = &source_coefficients.fullband_words;
        let mut fullband_coefficients = [[0.0_f32; FULLBAND_ENDMANT]; FULLBAND_CHANNEL_COUNT];
        for channel in 0..FULLBAND_CHANNEL_COUNT {
            for bin in 0..FULLBAND_ENDMANT {
                fullband_coefficients[channel][bin] = f32::from_bits(coefficients[channel][bin]);
            }
        }
        let lfe_coefficients = source_coefficients.lfe_words.map(f32::from_bits);
        let mantissa_fields = build_mantissa_fields(
            &fullband_coefficients,
            &quantizer_exponent_plans,
            &fullband_bap_rows,
            &lfe_coefficients,
            lfe_exponents,
            lfe_bap,
            policy.quantizer_policy,
        )?;

        let allocation = Ac3BlockAllocationControls {
            slow_decay_code: block_policy.slow_decay_code,
            fast_decay_code: block_policy.fast_decay_code,
            slow_gain_code: block_policy.slow_gain_code,
            db_per_bit_code: plan.db_per_bit_codes[block_index],
            floor_code: block_policy.floor_code,
            coarse_snr_offset: block_policy.candidate_q.coarse(),
            fullband_fine_snr_offsets: [block_policy.candidate_q.fine(); FULLBAND_CHANNEL_COUNT],
            fullband_fast_gain_codes: std::array::from_fn(|channel| {
                block_policy.fast_gain_codes[channel]
            }),
            lfe_fine_snr_offset: block_policy.candidate_q.fine(),
            lfe_fast_gain_code: block_policy.fast_gain_codes[FULLBAND_CHANNEL_COUNT],
        };

        let mut frame_scratch = vec![0_u8; AC3_BLOCK_BUDGET_BYTES];
        let bit_length = {
            let mut writer = FrameBitWriter::new(&mut frame_scratch, FrameBitCursor::new(0))?;
            let prefix = write_i0_ac3_audio_block_prefix(
                &mut writer,
                I0Ac3AudioBlockPrefixInput {
                    block_switch_flags: [false; FULLBAND_CHANNEL_COUNT],
                    dither_flags: block_policy.dither_flags,
                    dynamic_range_code: block_policy.dynamic_range_code,
                },
            )
            .map_err(|error| I0AudioBlockSeriesError::Block(Ac3AudioBlockError::Prefix(error)))?;
            let _cplinu_receipt =
                write_i0_ac3_first_block_cplinu(&mut writer, prefix, policy.cplinu).map_err(
                    |error| I0AudioBlockSeriesError::Block(Ac3AudioBlockError::Cplinu(error)),
                )?;
            write_selected_plan_exponent_syntax(
                &mut writer,
                exponent_plan,
                &fullband_wire_strategy_codes,
                &current_bandwidth_codes,
                &current_gain_ranges,
                &encoded_fullband,
                encoded_lfe,
            )?;
            write_bit_allocation_syntax(&mut writer, allocation)?;
            writer.write_bit(false)?; // deltbaie=0 for this caller-selected block policy.
            writer.write_bit(false)?; // I0 uses no skip-field payload.
            for field in &mantissa_fields {
                writer.write_unsigned(u32::from(field.value), field.width)?;
            }
            writer.cursor().bit_offset()
        };

        total_audio_bits = total_audio_bits
            .checked_add(bit_length)
            .ok_or(I0AudioBlockSeriesError::BitCountOverflow)?;
        if total_audio_bits > budget.available_bits() {
            return Err(I0AudioBlockSeriesError::AudioBlockBudgetExceeded {
                required_bits: total_audio_bits,
                available_bits: budget.available_bits(),
            });
        }
        let byte_length = usize::try_from(bit_length.div_ceil(8))
            .map_err(|_| I0AudioBlockSeriesError::BitCountOverflow)?;
        frame_scratch.truncate(byte_length);
        *block_bit_length = bit_length;
        blocks.push(Ac3AudioBlockCandidate {
            bytes: frame_scratch,
            bit_length,
        });
        previous_fullband_exponents = Some(current_fullband_exponents);
        previous_bandwidth_codes = Some(current_bandwidth_codes);
        previous_gain_ranges = Some(current_gain_ranges);
        previous_lfe_exponents = Some(lfe_exponents);
    }

    let blocks: [Ac3AudioBlockCandidate; I0_AUDIO_BLOCK_COUNT] = blocks
        .try_into()
        .map_err(|_| I0AudioBlockSeriesError::BitCountOverflow)?;
    let receipt = I0AudioBlockSeriesReceipt {
        source_identity: numeric_plan.identity().clone(),
        block_order: [
            I0AudioBlockIndex(0),
            I0AudioBlockIndex(1),
            I0AudioBlockIndex(2),
            I0AudioBlockIndex(3),
            I0AudioBlockIndex(4),
            I0AudioBlockIndex(5),
        ],
        block_bit_lengths,
        bap_policy: I0BapPolicyReceipt {
            block_controls: policy.blocks,
            db_per_bit_codes: plan.db_per_bit_codes,
            bap_provider_sha256: plan.bap_provider_sha256,
        },
        cplinu: policy.cplinu,
        delta_bit_allocation: policy.delta_bit_allocation,
        quantizer_policy: policy.quantizer_policy,
        assumptions: [
            I0AudioBlockAssumptionId::SelectedF12SpectrumAsI0Ac3LongBlockSyntaxV1,
            I0AudioBlockAssumptionId::SelectedF09F10ExponentStrategyWithD15RecodeOnExponentChangeV1,
        ],
    };

    Ok(I0AudioBlockSeriesCandidate { blocks, receipt })
}

fn write_selected_plan_exponent_syntax(
    writer: &mut FrameBitWriter<'_>,
    plan: &BlockExponentPlan,
    fullband_strategy_codes: &[u8; FULLBAND_CHANNEL_COUNT],
    bandwidth_codes: &[u8; FULLBAND_CHANNEL_COUNT],
    gain_range_codes: &[u8; FULLBAND_CHANNEL_COUNT],
    fullband_rows: &[Option<EncodedExponentRow>; FULLBAND_CHANNEL_COUNT],
    lfe_row: Option<EncodedLfeExponentRow>,
) -> Result<(), I0AudioBlockSeriesError> {
    for strategy_code in fullband_strategy_codes {
        writer.write_unsigned(u32::from(*strategy_code), 2)?;
    }
    writer.write_bit(plan.lfe.strategy == 1)?;

    for (channel, strategy_code) in fullband_strategy_codes.iter().enumerate() {
        if *strategy_code != 0 {
            writer.write_unsigned(u32::from(bandwidth_codes[channel]), 6)?;
        }
    }
    for (channel, row) in fullband_rows.iter().enumerate() {
        if let Some(row) = row {
            writer.write_unsigned(u32::from(row.absolute), 4)?;
            for group in row.groups.iter().take(row.group_count) {
                writer.write_unsigned(u32::from(*group), 7)?;
            }
            writer.write_unsigned(u32::from(gain_range_codes[channel]), 2)?;
        }
    }
    if let Some(row) = lfe_row {
        writer.write_unsigned(u32::from(row.absolute), 4)?;
        for group in row.groups {
            writer.write_unsigned(u32::from(group), 7)?;
        }
    }
    Ok(())
}

fn ac3_exponent_strategy(value: u8) -> Option<Ac3ExponentStrategy> {
    match value {
        1 => Some(Ac3ExponentStrategy::D15),
        2 => Some(Ac3ExponentStrategy::D25),
        3 => Some(Ac3ExponentStrategy::D45),
        _ => None,
    }
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
    forward_long_mdct_512(
        &input.previous_samples,
        &input.current_samples,
        window_f32_bits,
    )
    .map_err(|_| Ac3AudioBlockError::InvalidPcm)
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

#[cfg(test)]
mod i0_audio_block_series_tests {
    use std::sync::Arc;

    use super::{
        Ac3DeltaBitAllocationInput, Ac3MantissaQuantizerPolicy, CallerOwnedAc3Cplinu,
        I0AudioBlockAssumptionId, I0AudioBlockControlPolicy, I0AudioBlockIndex,
        I0AudioBlockSeriesBudget, I0AudioBlockSeriesError, I0AudioBlockSeriesPolicy,
        I0BapCandidateQ, produce_i0_audio_block_series,
    };
    use crate::{
        analysis::{
            input_history::{INPUT_HISTORY_BLOCK_SAMPLES, InputHistory},
            source_frame::{
                AnalysisOwnerGeneration, F07SourceFrameBeforeState, InstalledAudioSourceFrame,
                SharedAnalysisIdentity, stage_f07a_source_frame,
            },
            transient_channel::TransientChannelState,
            transient_parameters::load_transient_parameters,
        },
        codec::{
            access_unit_composer::{
                FLAT7X_AU_FRAME_BYTES, Flat7xI0D0AuCandidateError,
                assemble_flat7x_i0_d0_au_candidate,
            },
            bap::{
                BapProfileInput, BapRowInput, allocate_bap_row, allocate_bap_row_with_parameters,
                load_bap_parameters,
            },
            d0_frame_assembler::{
                D0_CRC2_FIELD_BITS, D0_TARGET_FRAME_BITS, D0_TARGET_FRAME_BYTES,
                D0Block2SkipFieldInput, D0BlockTwoAudioSpans, D0FrameAudioBlockInput,
                Flat7xD0FrameAssemblyError, Flat7xD0FrameBitInput, OpaqueD0AudioBitSpan,
            },
            d0_frame_layout::{CallerOwnedUnboundD0Block2InsertionPoint, D0SixBlockLayoutError},
            emdf_skip_field::{D0EmdfSkipFieldWriteError, flat7x_d0_block2_skipfld_bit_length},
            exponent_packing::pack_exponent_difference_group,
            flat7x_d0_frame_controls::{
                FLAT7X_D0_AUDIO_FRAME_CONTROL_BITS, Flat7xD0AllocationCandidateQ,
                Flat7xD0AudioFrameControlPolicy,
            },
            frame_builder::{Flat7xD0BsiStructure, measure_flat7x_d0_bsi_structure},
            i0_frame_assembler::{
                CallerOwnedI0CrcrsvSeed, Flat7xI0Ac3AudioSeriesFrameCandidate,
                Flat7xI0FrameAssemblyError, I0_TARGET_FRAME_BYTES, I0AuxDataInput,
                OpaqueI0Ac3BitSpan, assemble_flat7x_i0_ac3_frame_from_audio_series,
            },
            selected_frame_trial::{
                BlockExponentPlan, CoefficientBlockInput, ConverterHeaderInput,
                FullbandExponentLanePlan, LfeExponentPlan, MainExponentSyntax,
                SelectedSourceNumericPlan,
            },
        },
        timeline::CodecSourceIndex,
        transaction::{AttemptId, CallbackTransactionId, SourceTransaction},
    };

    const I0_BLOCKS: usize = 6;
    const FULLBAND_CHANNELS: usize = 5;
    const FULLBAND_BINS: usize = 217;
    const LFE_BINS: usize = 7;
    const BAP_PROVIDER: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-PUBLIC-BAP-PROVIDER-V1/public-bap-48k-v1.json"
    );
    const TRANSIENT_PARAMETERS: &[u8] =
        include_bytes!("../../tests/fixtures/transient_channel/PARAMETERS.json");

    fn f07_proposal() -> crate::analysis::source_frame::F07SourceFrameProposal {
        let before = F07SourceFrameBeforeState::new(
            std::array::from_fn(|_| InputHistory::cold()),
            std::array::from_fn(|_| TransientChannelState::cold()),
            [false; FULLBAND_CHANNELS],
        );
        let installed_audio: Box<InstalledAudioSourceFrame> =
            Box::new([[[0_u32; INPUT_HISTORY_BLOCK_SAMPLES]; I0_BLOCKS]; 6]);
        let parameters = load_transient_parameters(Some(TRANSIENT_PARAMETERS)).unwrap();
        stage_f07a_source_frame(&before, &installed_audio, &parameters).unwrap()
    }

    fn source_transaction() -> SourceTransaction {
        let mut transaction = SourceTransaction::new(
            AttemptId::new(7),
            CallbackTransactionId::new("i0-series-source-4").unwrap(),
            CodecSourceIndex::new(4),
            AnalysisOwnerGeneration::from_parent_snapshot(12),
        );
        transaction
            .prepare_shared_analysis(
                AnalysisOwnerGeneration::from_parent_snapshot(13),
                f07_proposal(),
            )
            .unwrap();
        transaction
    }

    fn numeric_plan() -> SelectedSourceNumericPlan {
        let bap_parameters = Arc::new(load_bap_parameters(Some(BAP_PROVIDER)).unwrap());
        let zero_group = pack_exponent_difference_group(&[0, 0, 0]).unwrap();
        let exponent_plan_by_block = std::array::from_fn(|block| {
            let strategy = u8::from(block == 0);
            let fullband_groups = if strategy == 0 {
                Vec::new()
            } else {
                vec![zero_group; 72]
            };
            let fullband = std::array::from_fn(|_| FullbandExponentLanePlan {
                decoded_exponents: [15; FULLBAND_BINS],
                strategy,
                bandwidth_code: 48,
                base_exponent: 15,
                packed_groups: fullband_groups.clone(),
                gain_range: 0,
            });
            BlockExponentPlan {
                fullband,
                lfe: LfeExponentPlan {
                    decoded_exponents: [15; LFE_BINS],
                    strategy,
                    base_exponent: 15,
                    packed_groups: [zero_group; 2],
                },
            }
        });
        let coefficients_by_block = std::array::from_fn(|_| CoefficientBlockInput {
            fullband_words: std::array::from_fn(|_| [0.0_f32.to_bits(); FULLBAND_BINS]),
            lfe_words: [0.0_f32.to_bits(); LFE_BINS],
        });
        SelectedSourceNumericPlan {
            bap_provider_sha256: bap_parameters.provider_sha256(),
            bap_parameters,
            coefficients_by_block,
            exponent_plan_by_block,
            db_per_bit_codes: [2; I0_BLOCKS],
            short_flags: [[false; FULLBAND_CHANNELS]; I0_BLOCKS],
            previous_short: [false; FULLBAND_CHANNELS],
            main_exponent_syntax: MainExponentSyntax::BlockStrategies,
            converter: ConverterHeaderInput {
                frame_codes: [0; FULLBAND_CHANNELS],
                snr_offset: 0,
            },
        }
    }

    // Synthetic test controls exercise caller-selected q and block ordering only. They are not
    // a selected or recommended production q policy.
    fn synthetic_policy() -> I0AudioBlockSeriesPolicy {
        I0AudioBlockSeriesPolicy {
            blocks: std::array::from_fn(|block| I0AudioBlockControlPolicy {
                candidate_q: I0BapCandidateQ::new(if block == 0 { 16 } else { 0 }).unwrap(),
                slow_decay_code: 2,
                fast_decay_code: 1,
                slow_gain_code: 1,
                floor_code: 7,
                fast_gain_codes: [0; FULLBAND_CHANNELS + 1],
                caller_all_active_fine_zero: block != 0,
                dither_flags: [false; FULLBAND_CHANNELS],
                dynamic_range_code: Some(0x30 + u8::try_from(block).unwrap()),
            }),
            cplinu: CallerOwnedAc3Cplinu::Independent,
            delta_bit_allocation: Ac3DeltaBitAllocationInput::Absent,
            quantizer_policy: Ac3MantissaQuantizerPolicy::NearestTiesEven,
        }
    }

    fn source_bound_i0_frame_candidate()
    -> (SharedAnalysisIdentity, Flat7xI0Ac3AudioSeriesFrameCandidate) {
        let mut transaction = source_transaction();
        let numeric_receipt = transaction.prepare_numeric_plan(numeric_plan()).unwrap();
        let audio_candidate = produce_i0_audio_block_series(
            &numeric_receipt,
            synthetic_policy(),
            I0AudioBlockSeriesBudget::new(20_480),
        )
        .unwrap();
        let source_identity = numeric_receipt.identity().clone();
        let target = crate::profile::Flat7xI0D0TargetPlan::approved_contract();
        let assemble = |auxdata| {
            assemble_flat7x_i0_ac3_frame_from_audio_series(
                target,
                numeric_receipt.identity(),
                &audio_candidate,
                auxdata,
                CallerOwnedI0CrcrsvSeed { value: false },
            )
        };
        let frame_candidate = match assemble(I0AuxDataInput::Absent) {
            Ok(candidate) => candidate,
            Err(Flat7xI0FrameAssemblyError::FrameSpanMismatch {
                actual_bits,
                required_bits,
            }) if actual_bits < required_bits => {
                let aux_payload_bits = required_bits - actual_bits;
                let aux_payload_bytes = usize::try_from(aux_payload_bits.div_ceil(8)).unwrap();
                let aux_payload = vec![0_u8; aux_payload_bytes];
                assemble(I0AuxDataInput::Present(OpaqueI0Ac3BitSpan::new(
                    &aux_payload,
                    0,
                    aux_payload_bits,
                )))
                .unwrap()
            }
            other => panic!("unexpected I0 frame assembly result: {other:?}"),
        };
        (source_identity, frame_candidate)
    }

    fn synthetic_d0_bsi() -> Flat7xD0BsiStructure<'static> {
        // Caller-authored test structure; fields after the captured chanmap are synthetic.
        Flat7xD0BsiStructure {
            dialnorm: 31,
            compr: Some(1),
            mixing: None,
            block_mix: [None; 6],
            information: None,
            addbsi: None,
        }
    }

    #[derive(Clone, Debug)]
    struct SyntheticD0AudioSpans {
        audio_bytes: [Vec<u8>; 6],
        skipfield_prefix_bytes: Vec<u8>,
        skipfield_suffix_bytes: Vec<u8>,
        audio_block_bit_lengths: [u32; 6],
        skipfield_offset_bits: u32,
    }

    fn synthetic_d0_audio_spans(
        target: crate::profile::Flat7xI0D0TargetPlan,
        bsi: &Flat7xD0BsiStructure<'_>,
        container_bytes: u16,
    ) -> SyntheticD0AudioSpans {
        let bsi_bits = measure_flat7x_d0_bsi_structure(target, bsi)
            .unwrap()
            .value();
        let skipfield_bits = flat7x_d0_block2_skipfld_bit_length(container_bytes).unwrap();
        let frame_bits = u32::from(target.d0().frame_bytes()) * 8;
        let audio_bits = frame_bits
            - D0_CRC2_FIELD_BITS
            - bsi_bits
            - FLAT7X_D0_AUDIO_FRAME_CONTROL_BITS
            - skipfield_bits;
        let even_bits = audio_bits / 6;
        let remainder = audio_bits % 6;
        let block_lengths = std::array::from_fn(|index| {
            even_bits + u32::from(u32::try_from(index).unwrap() < remainder)
        });
        let block_bytes = std::array::from_fn(|index| {
            let byte_count = usize::try_from(block_lengths[index].div_ceil(8)).unwrap();
            vec![0x30 + u8::try_from(index).unwrap(); byte_count]
        });
        let block_two_before_bits = block_lengths[2] / 2;
        let block_two_after_bits = block_lengths[2] - block_two_before_bits;
        let block_two_before =
            vec![0x52; usize::try_from(block_two_before_bits.div_ceil(8)).unwrap()];
        let block_two_after =
            vec![0x73; usize::try_from(block_two_after_bits.div_ceil(8)).unwrap()];

        SyntheticD0AudioSpans {
            audio_bytes: block_bytes,
            skipfield_prefix_bytes: block_two_before,
            skipfield_suffix_bytes: block_two_after,
            audio_block_bit_lengths: block_lengths,
            skipfield_offset_bits: block_two_before_bits,
        }
    }

    fn d0_frame_bit_input<'a>(
        audio: &'a SyntheticD0AudioSpans,
        skip_fields: &'a [D0Block2SkipFieldInput<'a>],
        block_lengths: [u32; 6],
    ) -> Flat7xD0FrameBitInput<'a> {
        let audio_blocks = std::array::from_fn(|index| {
            if index == 2 {
                D0FrameAudioBlockInput::BlockTwo(D0BlockTwoAudioSpans {
                    before_skipfield: OpaqueD0AudioBitSpan::new(
                        &audio.skipfield_prefix_bytes,
                        0,
                        audio.skipfield_offset_bits,
                    ),
                    skip_fields,
                    after_skipfield: OpaqueD0AudioBitSpan::new(
                        &audio.skipfield_suffix_bytes,
                        0,
                        block_lengths[2] - audio.skipfield_offset_bits,
                    ),
                })
            } else {
                D0FrameAudioBlockInput::Opaque(OpaqueD0AudioBitSpan::new(
                    &audio.audio_bytes[index],
                    0,
                    block_lengths[index],
                ))
            }
        });
        Flat7xD0FrameBitInput {
            audio_frame_controls: Flat7xD0AudioFrameControlPolicy::new(
                Flat7xD0AllocationCandidateQ::new(1).unwrap(),
            ),
            audio_blocks,
        }
    }

    fn read_bits(bytes: &[u8], start: usize, width: usize) -> u32 {
        (start..start + width).fold(0_u32, |value, bit_index| {
            let byte = bytes[bit_index / 8];
            let shift = 7 - bit_index % 8;
            (value << 1) | u32::from((byte >> shift) & 1)
        })
    }

    #[test]
    fn validated_bap_parameter_row_matches_the_provider_file_entry() {
        let parameters = load_bap_parameters(Some(BAP_PROVIDER)).unwrap();
        let exponents = [15_u8; FULLBAND_BINS];
        let input = BapRowInput {
            decoded_exponents: &exponents,
            end_exclusive: FULLBAND_BINS,
            candidate: 16,
            slow_decay_code: 2,
            fast_decay_code: 1,
            slow_gain_code: 1,
            db_per_bit_code: 2,
            floor_code: 7,
            fast_gain_code: 0,
            caller_all_active_fine_zero: false,
            profile: BapProfileInput {
                sample_rate_hz: 48_000,
                coupling: false,
                delta: false,
                aht: false,
            },
        };

        let reparsed = allocate_bap_row(Some(BAP_PROVIDER), &input).unwrap();
        let reused = allocate_bap_row_with_parameters(&parameters, &input).unwrap();
        assert_eq!(reused, reparsed);
    }

    #[test]
    fn six_ordered_blocks_retain_source_identity_and_caller_q_controls() {
        let mut transaction = source_transaction();
        let numeric_receipt = transaction.prepare_numeric_plan(numeric_plan()).unwrap();
        let selected_policy = synthetic_policy();
        let candidate = produce_i0_audio_block_series(
            &numeric_receipt,
            selected_policy,
            I0AudioBlockSeriesBudget::new(20_480),
        )
        .unwrap();

        assert_eq!(
            candidate.receipt().source_identity(),
            numeric_receipt.identity()
        );
        assert_eq!(candidate.blocks().len(), I0_BLOCKS);
        assert_eq!(candidate.receipt().block_bit_lengths().len(), I0_BLOCKS);
        assert!(!candidate.production_allowed());
        assert_eq!(
            candidate.receipt().assumptions()[0].id(),
            "PROVISIONAL_I0_SELECTED_F12_51_SPECTRUM_LONG_BLOCKS_V1"
        );
        assert_eq!(
            candidate.receipt().assumptions()[1].id(),
            "PROVISIONAL_I0_F09_F10_EXPSTR_D15_RECODE_ON_EXPONENT_CHANGE_V1"
        );

        for block in 0..I0_BLOCKS {
            let index = I0AudioBlockIndex::new(u8::try_from(block).unwrap()).unwrap();
            assert_eq!(
                candidate.receipt().block_order()[block].value(),
                u8::try_from(block).unwrap()
            );
            assert_eq!(
                candidate.receipt().bap_policy().block_controls()[block].candidate_q,
                selected_policy.blocks[block].candidate_q
            );
            assert_eq!(
                read_bits(candidate.block(index).bytes(), 11, 8),
                u32::from(0x30 + u8::try_from(block).unwrap())
            );
            assert_eq!(
                candidate.block(index).bit_length(),
                candidate.receipt().block_bit_lengths()[block]
            );
        }

        let spans = candidate.i0_bit_spans();
        for block in 0..I0_BLOCKS {
            assert_eq!(
                spans[block].bit_length(),
                candidate.blocks()[block].bit_length()
            );
        }
    }

    #[test]
    fn block_zero_exponent_reuse_rejects_the_series_before_returning_a_candidate() {
        let mut plan = numeric_plan();
        plan.exponent_plan_by_block[0].fullband[0].strategy = 0;
        let mut transaction = source_transaction();
        let numeric_receipt = transaction.prepare_numeric_plan(plan).unwrap();

        let error = produce_i0_audio_block_series(
            &numeric_receipt,
            synthetic_policy(),
            I0AudioBlockSeriesBudget::new(20_480),
        )
        .unwrap_err();

        assert_eq!(
            error,
            I0AudioBlockSeriesError::BlockZeroFullbandExponentReuse { channel: 0 }
        );
    }

    #[test]
    fn i0_audio_block_series_preserves_gain_range_and_implicit_reuse() {
        let mut baseline_transaction = source_transaction();
        let baseline_receipt = baseline_transaction
            .prepare_numeric_plan(numeric_plan())
            .unwrap();
        let baseline = produce_i0_audio_block_series(
            &baseline_receipt,
            synthetic_policy(),
            I0AudioBlockSeriesBudget::new(20_480),
        )
        .unwrap();

        let mut plan = numeric_plan();
        plan.exponent_plan_by_block[0].fullband[0].gain_range = 3;
        let mut transaction = source_transaction();
        let numeric_receipt = transaction.prepare_numeric_plan(plan).unwrap();
        let candidate = produce_i0_audio_block_series(
            &numeric_receipt,
            synthetic_policy(),
            I0AudioBlockSeriesBudget::new(20_480),
        )
        .unwrap();

        let block_zero = I0AudioBlockIndex::new(0).unwrap();
        let block_one = I0AudioBlockIndex::new(1).unwrap();
        assert_ne!(
            candidate.block(block_zero).bytes(),
            baseline.block(block_zero).bytes()
        );
        assert_eq!(
            candidate.block(block_one).bytes(),
            baseline.block(block_one).bytes()
        );
        assert!(!candidate.production_allowed());
    }

    #[test]
    fn i0_audio_block_series_uses_long_block_syntax_for_short_marked_source() {
        let mut plan = numeric_plan();
        plan.short_flags[0][0] = true;
        let mut transaction = source_transaction();
        let numeric_receipt = transaction.prepare_numeric_plan(plan).unwrap();

        let candidate = produce_i0_audio_block_series(
            &numeric_receipt,
            synthetic_policy(),
            I0AudioBlockSeriesBudget::new(20_480),
        )
        .unwrap();

        assert_eq!(candidate.blocks().len(), I0_BLOCKS);
        assert_eq!(
            candidate.receipt().assumptions()[0].id(),
            "PROVISIONAL_I0_SELECTED_F12_51_SPECTRUM_LONG_BLOCKS_V1"
        );
    }

    #[test]
    fn i0_audio_block_series_explicitly_codes_exponents_changed_under_reuse_marker() {
        let mut baseline_transaction = source_transaction();
        let baseline_receipt = baseline_transaction
            .prepare_numeric_plan(numeric_plan())
            .unwrap();
        let baseline = produce_i0_audio_block_series(
            &baseline_receipt,
            synthetic_policy(),
            I0AudioBlockSeriesBudget::new(20_480),
        )
        .unwrap();

        let mut plan = numeric_plan();
        plan.exponent_plan_by_block[1].fullband[1].decoded_exponents = [14; FULLBAND_BINS];
        plan.exponent_plan_by_block[1].fullband[1].base_exponent = 14;
        plan.exponent_plan_by_block[1].fullband[1].packed_groups =
            vec![pack_exponent_difference_group(&[0, 0, 0]).unwrap(); 72];
        let mut transaction = source_transaction();
        let numeric_receipt = transaction.prepare_numeric_plan(plan).unwrap();
        let candidate = produce_i0_audio_block_series(
            &numeric_receipt,
            synthetic_policy(),
            I0AudioBlockSeriesBudget::new(20_480),
        )
        .unwrap();

        let changed_block = I0AudioBlockIndex::new(1).unwrap();
        assert!(
            candidate.block(changed_block).bit_length()
                > baseline.block(changed_block).bit_length()
        );
        assert_eq!(
            candidate.receipt().assumptions()[1].id(),
            "PROVISIONAL_I0_F09_F10_EXPSTR_D15_RECODE_ON_EXPONENT_CHANGE_V1"
        );
    }

    #[test]
    fn aggregate_budget_rejects_without_returning_a_partial_block_set() {
        let mut transaction = source_transaction();
        let numeric_receipt = transaction.prepare_numeric_plan(numeric_plan()).unwrap();

        let error = produce_i0_audio_block_series(
            &numeric_receipt,
            synthetic_policy(),
            I0AudioBlockSeriesBudget::new(1),
        )
        .unwrap_err();
        assert!(
            matches!(
                error,
                I0AudioBlockSeriesError::AudioBlockBudgetExceeded {
                    available_bits: 1,
                    ..
                }
            ),
            "unexpected series error: {error:?}"
        );
    }

    #[test]
    fn six_block_candidate_composes_official_bsi_crc_and_source_identity_into_i0_frame() {
        let mut transaction = source_transaction();
        let numeric_receipt = transaction.prepare_numeric_plan(numeric_plan()).unwrap();
        let audio_candidate = produce_i0_audio_block_series(
            &numeric_receipt,
            synthetic_policy(),
            I0AudioBlockSeriesBudget::new(20_480),
        )
        .unwrap();
        let target = crate::profile::Flat7xI0D0TargetPlan::approved_contract();

        let assemble = |auxdata| {
            assemble_flat7x_i0_ac3_frame_from_audio_series(
                target,
                numeric_receipt.identity(),
                &audio_candidate,
                auxdata,
                CallerOwnedI0CrcrsvSeed { value: false },
            )
        };
        // Synthetic caller-owned auxdata fills the fixed target extent for this structural vector.
        let frame_candidate = match assemble(I0AuxDataInput::Absent) {
            Ok(candidate) => candidate,
            Err(Flat7xI0FrameAssemblyError::FrameSpanMismatch {
                actual_bits,
                required_bits,
            }) if actual_bits < required_bits => {
                let aux_payload_bits = required_bits - actual_bits;
                let aux_payload_bytes = usize::try_from(aux_payload_bits.div_ceil(8)).unwrap();
                let aux_payload = vec![0_u8; aux_payload_bytes];
                assemble(I0AuxDataInput::Present(OpaqueI0Ac3BitSpan::new(
                    &aux_payload,
                    0,
                    aux_payload_bits,
                )))
                .unwrap()
            }
            other => panic!("unexpected I0 frame assembly result: {other:?}"),
        };

        assert_eq!(frame_candidate.target(), target);
        assert_eq!(
            frame_candidate.source_identity(),
            numeric_receipt.identity()
        );
        assert_eq!(frame_candidate.frame_bytes().len(), I0_TARGET_FRAME_BYTES);
        assert_eq!(frame_candidate.bsi_tail_receipt().start_bit(), 70);
        assert_eq!(frame_candidate.bsi_tail_receipt().end_bit(), 105);
        assert_eq!(frame_candidate.bsi_tail_receipt().values().dmixmod(), 1);
        assert_eq!(
            frame_candidate.audio_assumptions(),
            &[
                I0AudioBlockAssumptionId::SelectedF12SpectrumAsI0Ac3LongBlockSyntaxV1,
                I0AudioBlockAssumptionId::SelectedF09F10ExponentStrategyWithD15RecodeOnExponentChangeV1,
            ]
        );
        assert_eq!(frame_candidate.frame_receipt().audio_block_spans().len(), 6);
        assert_eq!(
            frame_candidate.frame_receipt().target_frame_bytes(),
            I0_TARGET_FRAME_BYTES as u16
        );
        assert_eq!(
            frame_candidate
                .frame_receipt()
                .integrity_receipts()
                .crc1()
                .residual(),
            0
        );
        assert_eq!(
            frame_candidate
                .frame_receipt()
                .integrity_receipts()
                .crc2()
                .residual(),
            0
        );
        assert!(!frame_candidate.production_allowed());

        let mut mismatched_transaction = SourceTransaction::new(
            AttemptId::new(8),
            CallbackTransactionId::new("i0-series-other-source").unwrap(),
            CodecSourceIndex::new(5),
            AnalysisOwnerGeneration::from_parent_snapshot(12),
        );
        mismatched_transaction
            .prepare_shared_analysis(
                AnalysisOwnerGeneration::from_parent_snapshot(13),
                f07_proposal(),
            )
            .unwrap();
        let mismatched_numeric_receipt = mismatched_transaction
            .prepare_numeric_plan(numeric_plan())
            .unwrap();
        assert_eq!(
            assemble_flat7x_i0_ac3_frame_from_audio_series(
                target,
                mismatched_numeric_receipt.identity(),
                &audio_candidate,
                I0AuxDataInput::Absent,
                CallerOwnedI0CrcrsvSeed { value: false },
            ),
            Err(Flat7xI0FrameAssemblyError::SourceIdentityMismatch)
        );
    }

    #[test]
    fn c511_pair_candidate_binds_index2_and_composes_i0_before_d0() {
        let (source_identity, i0_frame) = source_bound_i0_frame_candidate();
        let target = crate::profile::Flat7xI0D0TargetPlan::approved_contract();
        let d0_bsi = synthetic_d0_bsi();
        let d0_audio = synthetic_d0_audio_spans(target, &d0_bsi, 511);
        let emdf_container = vec![0xa5_u8; 511];
        let skip_fields = [D0Block2SkipFieldInput {
            insertion_point: CallerOwnedUnboundD0Block2InsertionPoint::new(
                d0_audio.skipfield_offset_bits,
            ),
            complete_emdf_container: &emdf_container,
        }];
        let d0_input =
            d0_frame_bit_input(&d0_audio, &skip_fields, d0_audio.audio_block_bit_lengths);

        let candidate = assemble_flat7x_i0_d0_au_candidate(
            target,
            &source_identity,
            i0_frame.clone(),
            d0_bsi,
            d0_input,
        )
        .unwrap();

        assert_eq!(candidate.source_identity(), &source_identity);
        assert_eq!(candidate.target().aggregate_rate_kbps(), Some(1_664));
        assert_eq!(candidate.target().joc_spatial_index().value(), 1);
        assert_eq!(candidate.target().downmix_index().value(), 1);
        assert_eq!(candidate.target().joc_object_count(), 15);
        assert_eq!(
            candidate
                .target()
                .d0()
                .channel_map()
                .map(|channel_map| channel_map.value()),
            Some(0x1a00)
        );
        assert_eq!(candidate.d0_frame_bytes().len(), D0_TARGET_FRAME_BYTES);
        assert_eq!(candidate.d0_frame_receipt().target_frame_bytes(), 4_096);
        assert_eq!(
            candidate.d0_frame_receipt().layout().audio_blocks()[2]
                .block_index()
                .zero_based_value(),
            2
        );
        assert_eq!(
            candidate
                .d0_frame_receipt()
                .layout()
                .block_two_skip_field()
                .payload_bytes(),
            511
        );
        assert_eq!(
            candidate
                .d0_frame_receipt()
                .crc2_integrity()
                .verified_residual(),
            0
        );

        let receipt = candidate.access_unit_receipt();
        assert_eq!(receipt.i0_span().start_byte(), 0);
        assert_eq!(receipt.i0_span().byte_length(), 2_560);
        assert_eq!(receipt.d0_span().start_byte(), 2_560);
        assert_eq!(receipt.d0_span().byte_length(), 4_096);
        assert_eq!(receipt.total_byte_length(), FLAT7X_AU_FRAME_BYTES as u32);
        assert_eq!(candidate.access_unit_bytes().len(), FLAT7X_AU_FRAME_BYTES);
        assert_eq!(
            &candidate.access_unit_bytes()[..2_560],
            i0_frame.frame_bytes()
        );
        assert_eq!(
            &candidate.access_unit_bytes()[2_560..],
            candidate.d0_frame_bytes()
        );
        assert!(!candidate.production_allowed());
    }

    #[test]
    fn pair_candidate_rejects_identity_mismatch_c512_and_d0_extent_error() {
        let (source_identity, i0_frame) = source_bound_i0_frame_candidate();
        let target = crate::profile::Flat7xI0D0TargetPlan::approved_contract();
        let d0_bsi = synthetic_d0_bsi();
        let d0_audio = synthetic_d0_audio_spans(target, &d0_bsi, 511);
        let emdf_container = vec![0x5a_u8; 511];
        let skip_fields = [D0Block2SkipFieldInput {
            insertion_point: CallerOwnedUnboundD0Block2InsertionPoint::new(
                d0_audio.skipfield_offset_bits,
            ),
            complete_emdf_container: &emdf_container,
        }];
        let d0_input =
            d0_frame_bit_input(&d0_audio, &skip_fields, d0_audio.audio_block_bit_lengths);

        let mut mismatched_transaction = SourceTransaction::new(
            AttemptId::new(8),
            CallbackTransactionId::new("pair-au-other-source").unwrap(),
            CodecSourceIndex::new(5),
            AnalysisOwnerGeneration::from_parent_snapshot(12),
        );
        mismatched_transaction
            .prepare_shared_analysis(
                AnalysisOwnerGeneration::from_parent_snapshot(13),
                f07_proposal(),
            )
            .unwrap();
        let mismatched_numeric_receipt = mismatched_transaction
            .prepare_numeric_plan(numeric_plan())
            .unwrap();
        assert_eq!(
            assemble_flat7x_i0_d0_au_candidate(
                target,
                mismatched_numeric_receipt.identity(),
                i0_frame.clone(),
                d0_bsi,
                d0_input,
            ),
            Err(Flat7xI0D0AuCandidateError::SourceIdentityMismatch)
        );

        let emdf_container_512 = vec![0x5a_u8; 512];
        let skip_fields_512 = [D0Block2SkipFieldInput {
            insertion_point: CallerOwnedUnboundD0Block2InsertionPoint::new(
                d0_audio.skipfield_offset_bits,
            ),
            complete_emdf_container: &emdf_container_512,
        }];
        let d0_input_512 = d0_frame_bit_input(
            &d0_audio,
            &skip_fields_512,
            d0_audio.audio_block_bit_lengths,
        );
        assert_eq!(
            assemble_flat7x_i0_d0_au_candidate(
                target,
                &source_identity,
                i0_frame.clone(),
                d0_bsi,
                d0_input_512,
            ),
            Err(Flat7xI0D0AuCandidateError::D0FrameAssembly(
                Flat7xD0FrameAssemblyError::Layout(D0SixBlockLayoutError::SkipField(
                    D0EmdfSkipFieldWriteError::ContainerTooLarge,
                )),
            ))
        );

        let mut short_block_lengths = d0_audio.audio_block_bit_lengths;
        short_block_lengths[0] -= 1;
        let short_d0_input = d0_frame_bit_input(&d0_audio, &skip_fields, short_block_lengths);
        assert_eq!(
            assemble_flat7x_i0_d0_au_candidate(
                target,
                &source_identity,
                i0_frame,
                d0_bsi,
                short_d0_input,
            ),
            Err(Flat7xI0D0AuCandidateError::D0FrameAssembly(
                Flat7xD0FrameAssemblyError::FrameSpanMismatch {
                    actual_bits: D0_TARGET_FRAME_BITS - 1,
                    required_bits: D0_TARGET_FRAME_BITS,
                },
            ))
        );
    }
}
