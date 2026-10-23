// pattern: Functional Core
//! Pure F12S selected-profile mantissa quantization and its future receipt boundary.

use crate::numeric::{FiniteF32, r32_add, r32_mul, trunc_f32_to_i32};

const MANTISSA_WIDTH_BY_BAP: [u8; 16] = [0, 5, 7, 3, 7, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MantissaBoundaryError {
    Input,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LfeGateError {
    UnsupportedProfile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LfeGateProfile {
    pub sample_rate_hz: u32,
    pub bit_rate_kbps: u16,
    pub acmod: u8,
    pub blocks: u8,
    pub aht_enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClippedMantissaRow {
    clipped_words: Vec<u32>,
    raw_exponents: Vec<u8>,
    clipped_flags: Vec<bool>,
}

impl ClippedMantissaRow {
    pub fn clipped_words(&self) -> &[u32] {
        &self.clipped_words
    }

    pub fn raw_exponents(&self) -> &[u8] {
        &self.raw_exponents
    }

    pub fn clipped_flags(&self) -> &[bool] {
        &self.clipped_flags
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum LfeMantissaError {
    Input,
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MantissaDownstreamOutcome {
    Accepted,
    Failed,
}

#[derive(Debug, Eq, PartialEq)]
pub struct StagedLfeMantissaRow {
    row: ClippedMantissaRow,
}

impl StagedLfeMantissaRow {
    pub fn row(&self) -> &ClippedMantissaRow {
        &self.row
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct LfeMantissaFinishReceipt {
    committed: bool,
}

impl LfeMantissaFinishReceipt {
    pub const fn committed(&self) -> bool {
        self.committed
    }
}

/// The selected LFE coefficient/exponent owner for this bounded staged slice.
#[derive(Debug, Eq, PartialEq)]
pub struct LfeMantissaMachine {
    gate: i16,
    committed_row: Option<ClippedMantissaRow>,
    retired: bool,
}

impl LfeMantissaMachine {
    pub fn new(profile: LfeGateProfile) -> Result<Self, LfeGateError> {
        Ok(Self {
            gate: selected_lfe_gate(profile)?,
            committed_row: None,
            retired: false,
        })
    }

    pub fn stage_lfe_coefficients(
        &self,
        words: &[u32],
    ) -> Result<StagedLfeMantissaRow, LfeMantissaError> {
        if self.retired {
            return Err(LfeMantissaError::Retired);
        }
        let row = clip_lfe_main_coefficients(words).map_err(|_| LfeMantissaError::Input)?;
        Ok(StagedLfeMantissaRow { row })
    }

    /// Resolves one staged row after the selected downstream stage reports its outcome.
    pub fn finish_source(
        &mut self,
        staged: StagedLfeMantissaRow,
        outcome: MantissaDownstreamOutcome,
    ) -> LfeMantissaFinishReceipt {
        if self.retired || outcome == MantissaDownstreamOutcome::Failed {
            self.retired = true;
            return LfeMantissaFinishReceipt { committed: false };
        }
        self.committed_row = Some(staged.row);
        LfeMantissaFinishReceipt { committed: true }
    }

    pub const fn gate(&self) -> i16 {
        self.gate
    }

    pub const fn retired(&self) -> bool {
        self.retired
    }

    pub fn committed_row(&self) -> Option<&ClippedMantissaRow> {
        self.committed_row.as_ref()
    }

    pub fn reset(&mut self) {
        self.gate = 0;
        self.committed_row = None;
        self.retired = false;
    }
}

/// Returns the selected profile's cold and produced LFE normalization gate.
pub fn selected_lfe_gate(profile: LfeGateProfile) -> Result<i16, LfeGateError> {
    if profile.sample_rate_hz != 48_000
        || profile.bit_rate_kbps != 1_024
        || profile.acmod != 7
        || profile.blocks != 6
        || profile.aht_enabled
    {
        return Err(LfeGateError::UnsupportedProfile);
    }
    Ok(0)
}

/// Applies the shared strict PRIMARY/LFE clip and raw-exponent scan to a PRIMARY row.
pub fn clip_primary_coefficients(
    words: &[u32],
) -> Result<ClippedMantissaRow, MantissaBoundaryError> {
    if words.len() > 253 || words.iter().any(|word| !f32::from_bits(*word).is_finite()) {
        return Err(MantissaBoundaryError::Input);
    }
    Ok(scan_finite_coefficients(words))
}

/// Applies the shared strict PRIMARY/LFE clip and raw-exponent scan to the seven-value LFE row.
pub fn clip_lfe_main_coefficients(
    words: &[u32],
) -> Result<ClippedMantissaRow, MantissaBoundaryError> {
    if words.len() != 7 || words.iter().any(|word| !f32::from_bits(*word).is_finite()) {
        return Err(MantissaBoundaryError::Input);
    }
    Ok(scan_finite_coefficients(words))
}

fn scan_finite_coefficients(words: &[u32]) -> ClippedMantissaRow {
    let mut clipped_words = Vec::with_capacity(words.len());
    let mut raw_exponents = Vec::with_capacity(words.len());
    let mut clipped_flags = Vec::with_capacity(words.len());
    for &word in words {
        let value = f32::from_bits(word);
        let clipped_word = if value < -1.0 {
            0xbf80_0000
        } else if value > 1.0 {
            0x3f80_0000
        } else {
            word
        };
        clipped_words.push(clipped_word);
        raw_exponents.push(raw_primary_exponent(clipped_word));
        clipped_flags.push(clipped_word != word);
    }
    ClippedMantissaRow {
        clipped_words,
        raw_exponents,
        clipped_flags,
    }
}

fn raw_primary_exponent(clipped_word: u32) -> u8 {
    let doubled = clipped_word.wrapping_mul(2);
    let exponent = if doubled == 0 {
        -127_i16
    } else {
        i16::try_from(doubled >> 24).expect("top byte fits i16") - 126
    };
    let mut h = (-exponent).clamp(0, 60);
    if h == 60 {
        h = 24;
    }
    u8::try_from(h.min(24)).expect("raw exponent is in 0..=24")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MantissaQuantizerError {
    NonFiniteInput,
    UnsupportedBap,
    NonFiniteIntermediate,
    IntegerOutOfRange,
    ReservedCodeSentinel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MantissaCode(u16);

impl MantissaCode {
    pub const fn value(self) -> u16 {
        self.0
    }
}

/// Quantizes one already-scaled finite binary32 word for an admitted BAP value.
///
/// This is only the stateless F12S.5 code-value slice. It does not scale an input
/// coefficient by an exponent, group codes, write bits, or participate in a trial.
pub fn quantize_mantissa(
    finite_f32_word: u32,
    bap: u8,
) -> Result<MantissaCode, MantissaQuantizerError> {
    if bap > 15 {
        return Err(MantissaQuantizerError::UnsupportedBap);
    }
    let coefficient = FiniteF32::new(f32::from_bits(finite_f32_word))
        .map_err(|_| MantissaQuantizerError::NonFiniteInput)?;
    if bap == 0 {
        return Ok(MantissaCode(0));
    }

    match bap {
        1 => quantize_clamped(coefficient, 1.5, 3),
        2 => quantize_clamped(coefficient, 2.5, 5),
        4 => quantize_clamped(coefficient, 5.5, 11),
        3 => quantize_sentinel(coefficient, 3.5, 7),
        5 => quantize_sentinel(coefficient, 7.5, 15),
        6..=15 => quantize_fixed_point(coefficient, bap),
        _ => Err(MantissaQuantizerError::UnsupportedBap),
    }
}

fn quantize_clamped(
    coefficient: FiniteF32,
    scale: f32,
    levels: u16,
) -> Result<MantissaCode, MantissaQuantizerError> {
    let scaled = scaled_offset(coefficient, scale)?;
    let truncated = trunc_f32_to_i32(scaled)
        .map_err(|_| MantissaQuantizerError::IntegerOutOfRange)?
        .value();
    let bounded = truncated.clamp(0, i32::from(levels - 1));
    Ok(MantissaCode(
        u16::try_from(bounded).expect("clamped nonnegative code"),
    ))
}

fn quantize_sentinel(
    coefficient: FiniteF32,
    scale: f32,
    sentinel: u16,
) -> Result<MantissaCode, MantissaQuantizerError> {
    let scaled = scaled_offset(coefficient, scale)?;
    let truncated = trunc_f32_to_i32(scaled)
        .map_err(|_| MantissaQuantizerError::IntegerOutOfRange)?
        .value();
    let modulus = i32::from(sentinel) + 1;
    let raw = u32::try_from(truncated.rem_euclid(modulus)).expect("remainder is nonnegative");
    if raw == u32::from(sentinel) {
        return Err(MantissaQuantizerError::ReservedCodeSentinel);
    }
    Ok(MantissaCode(
        u16::try_from(raw).expect("masked code fits u16"),
    ))
}

fn scaled_offset(coefficient: FiniteF32, scale: f32) -> Result<FiniteF32, MantissaQuantizerError> {
    let offset = FiniteF32::new(1.0).expect("one is finite");
    let biased =
        r32_add(coefficient, offset).map_err(|_| MantissaQuantizerError::NonFiniteIntermediate)?;
    r32_mul(
        biased,
        FiniteF32::new(scale).expect("selected scale is finite"),
    )
    .map_err(|_| MantissaQuantizerError::NonFiniteIntermediate)
}

fn quantize_fixed_point(
    coefficient: FiniteF32,
    bap: u8,
) -> Result<MantissaCode, MantissaQuantizerError> {
    let width = MANTISSA_WIDTH_BY_BAP[usize::from(bap)];
    let offset = 2.0_f32.powi(-i32::from(width));
    let biased = r32_add(
        coefficient,
        FiniteF32::new(offset).expect("selected binary offset is finite"),
    )
    .map_err(|_| MantissaQuantizerError::NonFiniteIntermediate)?;
    let scaled = r32_mul(
        biased,
        FiniteF32::new(65_536.0).expect("selected fixed-point scale is finite"),
    )
    .map_err(|_| MantissaQuantizerError::NonFiniteIntermediate)?;
    let shifted = crate::numeric::r32_sub(
        scaled,
        FiniteF32::new(0.5).expect("selected half offset is finite"),
    )
    .map_err(|_| MantissaQuantizerError::NonFiniteIntermediate)?;

    let rounded = round_ties_even(f64::from(shifted.value()));
    let fixed = if rounded >= 65_534.0 {
        i16::MAX
    } else if rounded <= -65_536.0 {
        i16::MIN
    } else {
        i16::try_from((rounded as i64) >> 1).expect("bounded shifted code fits i16")
    };
    let fixed_bits = u16::from_ne_bytes(fixed.to_ne_bytes());
    let code = if width == 16 {
        fixed_bits
    } else {
        fixed_bits >> (16 - width)
    };
    Ok(MantissaCode(code))
}

fn round_ties_even(value: f64) -> f64 {
    value.round_ties_even()
}
