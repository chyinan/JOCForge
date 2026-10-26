// pattern: Functional Core
//! Boundary for bit-allocation proposals. It cannot read files or mutate frame state.
//! Implements the selected ordinary F12A branch from ETSI TS 102 366 V1.4.1, clause 6.2.3.

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

const SELECTED_SAMPLE_RATE_HZ: u32 = 48_000;
const SELECTED_PROVIDER_BYTES: usize = 8_509;
const SELECTED_PROVIDER_SHA256: &str =
    "eb913038a89d853db453831d64df34c19f8ea8a89d04a72c0ef43733619545a3";
const SELECTED_PROVIDER_SCHEMA: &str = "public-bap-parameters.v1";
const SELECTED_PROVIDER_PROFILE: &str = "48000_HZ_ORDINARY_FBW217_LFE7_NO_COUPLING_DELTA_AHT";

/// Selected F12A main-path fast-gain code.
pub const MAIN_FAST_GAIN_CODE: u8 = 1;

/// Selected F12A converter-path fast-gain code.
pub const CONVERTER_FAST_GAIN_CODE: u8 = 4;

/// Number of full-band rows in one selected 5.1 block.
pub const FULLBAND_CHANNEL_COUNT: usize = 5;

/// Number of decoded exponent bins in each full-band row.
pub const FULLBAND_BIN_COUNT: usize = 217;

/// Number of decoded exponent bins in the LFE row.
pub const LFE_BIN_COUNT: usize = 7;

/// Errors returned by the selected 48 kHz ordinary F12A provider and producer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BapError {
    InputError,
    ParameterUnavailable,
    ParameterError,
    UnsupportedProfile,
}

impl BapError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputError => "BAP_INPUT_ERROR",
            Self::ParameterUnavailable => "PARAMETER_UNAVAILABLE",
            Self::ParameterError => "BAP_PARAMETER_ERROR",
            Self::UnsupportedProfile => "UNSUPPORTED_PROFILE",
        }
    }
}

/// Validated exact public tables for the selected F12A profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BapParameters {
    slow_decay: [i16; 4],
    fast_decay: [i16; 4],
    slow_gain: [i16; 4],
    db_knee: [i16; 4],
    floor: [i16; 8],
    fast_gain: [i16; 8],
    log_addition: [i16; 256],
    bands: [BapBand; 50],
    hearing_48k: [i16; 50],
    bap_pointer: [u8; 64],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BapBand {
    start: u16,
    size: u8,
}

/// Typed input for one F12A element. Its fourteen fields correspond to the
/// approved F12A input object; Rust types rule out JSON booleans as integers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BapRowInput<'a> {
    pub decoded_exponents: &'a [u8],
    pub end_exclusive: usize,
    pub candidate: u16,
    pub slow_decay_code: u8,
    pub fast_decay_code: u8,
    pub slow_gain_code: u8,
    pub db_per_bit_code: u8,
    pub floor_code: u8,
    pub fast_gain_code: u8,
    pub caller_all_active_fine_zero: bool,
    pub profile: BapProfileInput,
}

/// Sample-rate and feature flags supplied by one typed F12A input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BapProfileInput {
    pub sample_rate_hz: u32,
    pub coupling: bool,
    pub delta: bool,
    pub aht: bool,
}

impl BapProfileInput {
    pub const fn selected() -> Self {
        Self {
            sample_rate_hz: SELECTED_SAMPLE_RATE_HZ,
            coupling: false,
            delta: false,
            aht: false,
        }
    }
}

/// Typed six-block F12A input in internal `[L, C, R, Ls, Rs, LFE]` order.
/// Each full-band row has 217 exponents and each LFE row has 7.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BapFrameInput<'a> {
    pub decoded_exponents_by_block: &'a [[&'a [u8]; 6]; 6],
    pub end_exclusive_by_channel: [usize; 6],
    pub candidate: u16,
    pub slow_decay_code: u8,
    pub fast_decay_code: u8,
    pub slow_gain_code: u8,
    pub db_per_bit_codes: [u8; 6],
    pub floor_code: u8,
    pub fast_gain_code: u8,
    pub caller_all_active_fine_zero_by_block: [bool; 6],
    pub profile: BapProfileInput,
}

/// One selected frame's immutable BAP output, preserving block and channel axes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BapFrame {
    candidate_q: u16,
    blocks: [BapBlock; 6],
}

impl BapFrame {
    pub const fn candidate_q(&self) -> u16 {
        self.candidate_q
    }

    pub const fn blocks(&self) -> &[BapBlock; 6] {
        &self.blocks
    }
}

/// Axis-preserving BAP output for one selected 5.1 block.
///
/// F12A produces full-band rows in internal `[L, C, R, Ls, Rs]` order and a
/// separate LFE row. Converter costs consume this shape, but reset their
/// grouped counters at each block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BapBlock {
    fullband_rows: [[u8; FULLBAND_BIN_COUNT]; FULLBAND_CHANNEL_COUNT],
    lfe_row: [u8; LFE_BIN_COUNT],
}

impl BapBlock {
    /// Admits six rows of BAP codes without changing their channel or bin axes.
    pub fn new(
        fullband_rows: [[u8; FULLBAND_BIN_COUNT]; FULLBAND_CHANNEL_COUNT],
        lfe_row: [u8; LFE_BIN_COUNT],
    ) -> Result<Self, BapOutputError> {
        for (channel, row) in fullband_rows.iter().enumerate() {
            for (bin, code) in row.iter().copied().enumerate() {
                if code > 15 {
                    return Err(BapOutputError::CodeOutOfRange { channel, bin, code });
                }
            }
        }
        for (bin, code) in lfe_row.iter().copied().enumerate() {
            if code > 15 {
                return Err(BapOutputError::CodeOutOfRange {
                    channel: FULLBAND_CHANNEL_COUNT,
                    bin,
                    code,
                });
            }
        }
        Ok(Self {
            fullband_rows,
            lfe_row,
        })
    }

    pub const fn fullband_rows(&self) -> &[[u8; FULLBAND_BIN_COUNT]; FULLBAND_CHANNEL_COUNT] {
        &self.fullband_rows
    }

    pub const fn lfe_row(&self) -> &[u8; LFE_BIN_COUNT] {
        &self.lfe_row
    }
}

/// Rejects an externally supplied BAP row containing a code outside 0 through 15.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BapOutputError {
    CodeOutOfRange {
        channel: usize,
        bin: usize,
        code: u8,
    },
}

impl BapOutputError {
    pub const fn code(self) -> &'static str {
        "BAP_INPUT_ERROR"
    }
}

/// Loads the exact selected P01 public BAP provider from caller-supplied bytes.
///
/// Missing bytes map to `PARAMETER_UNAVAILABLE`; bytes with a different identity
/// or malformed tables map to `BAP_PARAMETER_ERROR`.
pub fn load_bap_parameters(provider_file: Option<&[u8]>) -> Result<BapParameters, BapError> {
    let provider_file = provider_file.ok_or(BapError::ParameterUnavailable)?;
    if provider_file.len() != SELECTED_PROVIDER_BYTES
        || sha256_hex(provider_file) != SELECTED_PROVIDER_SHA256
    {
        return Err(BapError::ParameterError);
    }
    parse_bap_parameters(provider_file)
}

/// Allocates one selected F12A row without retaining state between calls.
pub fn allocate_bap_row(
    provider_file: Option<&[u8]>,
    input: &BapRowInput<'_>,
) -> Result<Vec<u8>, BapError> {
    validate_sample_and_profile(input.profile)?;
    let parameters = load_bap_parameters(provider_file)?;
    validate_row_extent(input.end_exclusive)?;
    validate_exponent_row(input.decoded_exponents, input.end_exclusive)?;
    validate_candidate_codes(
        input.candidate,
        input.slow_decay_code,
        input.fast_decay_code,
        input.slow_gain_code,
        input.db_per_bit_code,
        input.floor_code,
        input.fast_gain_code,
    )?;
    validate_caller_context(input.candidate, input.caller_all_active_fine_zero)?;
    compute_bap_row(input, &parameters)
}

/// Allocates BAP for one selected q over six blocks in internal
/// `[L, C, R, Ls, Rs, LFE]` channel order.
///
/// The function validates the complete input and provider before calculating
/// any output row. It returns no partial frame on a provider, shape, input, or
/// profile error.
pub fn allocate_bap_frame(
    provider_file: Option<&[u8]>,
    input: &BapFrameInput<'_>,
) -> Result<BapFrame, BapError> {
    validate_sample_and_profile(input.profile)?;
    let parameters = load_bap_parameters(provider_file)?;

    for end_exclusive in input.end_exclusive_by_channel {
        validate_row_extent(end_exclusive)?;
    }
    for (channel, end_exclusive) in input.end_exclusive_by_channel.iter().copied().enumerate() {
        let expected_end = if channel < FULLBAND_CHANNEL_COUNT {
            FULLBAND_BIN_COUNT
        } else {
            LFE_BIN_COUNT
        };
        if end_exclusive != expected_end {
            return Err(BapError::InputError);
        }
    }

    for block_rows in input.decoded_exponents_by_block {
        for (channel, row) in block_rows.iter().enumerate() {
            validate_exponent_row(row, input.end_exclusive_by_channel[channel])?;
        }
    }

    validate_candidate_codes(
        input.candidate,
        input.slow_decay_code,
        input.fast_decay_code,
        input.slow_gain_code,
        input.db_per_bit_codes[0],
        input.floor_code,
        input.fast_gain_code,
    )?;
    if input.db_per_bit_codes.iter().any(|code| *code > 3) {
        return Err(BapError::InputError);
    }
    let expected_active_fine_zero = input.candidate.trailing_zeros() >= 4;
    for context in input.caller_all_active_fine_zero_by_block {
        if context != expected_active_fine_zero {
            return Err(BapError::InputError);
        }
        validate_caller_context(input.candidate, context)?;
    }

    let mut blocks = [BapBlock {
        fullband_rows: [[0_u8; FULLBAND_BIN_COUNT]; FULLBAND_CHANNEL_COUNT],
        lfe_row: [0_u8; LFE_BIN_COUNT],
    }; 6];
    for (block_index, block) in blocks.iter_mut().enumerate() {
        for (channel, decoded_exponents) in input.decoded_exponents_by_block[block_index]
            .iter()
            .enumerate()
        {
            let row_input = BapRowInput {
                decoded_exponents,
                end_exclusive: input.end_exclusive_by_channel[channel],
                candidate: input.candidate,
                slow_decay_code: input.slow_decay_code,
                fast_decay_code: input.fast_decay_code,
                slow_gain_code: input.slow_gain_code,
                db_per_bit_code: input.db_per_bit_codes[block_index],
                floor_code: input.floor_code,
                fast_gain_code: input.fast_gain_code,
                caller_all_active_fine_zero: input.caller_all_active_fine_zero_by_block
                    [block_index],
                profile: input.profile,
            };
            let codes = compute_bap_row(&row_input, &parameters)?;
            if channel < FULLBAND_CHANNEL_COUNT {
                block.fullband_rows[channel].copy_from_slice(&codes);
            } else {
                block.lfe_row.copy_from_slice(&codes);
            }
        }
    }
    Ok(BapFrame {
        candidate_q: input.candidate,
        blocks,
    })
}

fn validate_sample_and_profile(profile: BapProfileInput) -> Result<(), BapError> {
    if profile.sample_rate_hz == 0 {
        return Err(BapError::InputError);
    }
    if profile.sample_rate_hz != SELECTED_SAMPLE_RATE_HZ
        || profile.coupling
        || profile.delta
        || profile.aht
    {
        return Err(BapError::UnsupportedProfile);
    }
    Ok(())
}

fn validate_row_extent(end_exclusive: usize) -> Result<(), BapError> {
    if end_exclusive != FULLBAND_BIN_COUNT && end_exclusive != LFE_BIN_COUNT {
        return Err(BapError::UnsupportedProfile);
    }
    Ok(())
}

fn validate_exponent_row(row: &[u8], end_exclusive: usize) -> Result<(), BapError> {
    if row.len() != end_exclusive || row.iter().any(|exponent| *exponent > 24) {
        return Err(BapError::InputError);
    }
    Ok(())
}

fn validate_candidate_codes(
    candidate: u16,
    slow_decay_code: u8,
    fast_decay_code: u8,
    slow_gain_code: u8,
    db_per_bit_code: u8,
    floor_code: u8,
    fast_gain_code: u8,
) -> Result<(), BapError> {
    if candidate > 1_023
        || slow_decay_code > 3
        || fast_decay_code > 3
        || slow_gain_code > 3
        || db_per_bit_code > 3
        || floor_code > 7
        || fast_gain_code > 7
    {
        return Err(BapError::InputError);
    }
    Ok(())
}

fn validate_caller_context(
    candidate: u16,
    caller_all_active_fine_zero: bool,
) -> Result<(), BapError> {
    if caller_all_active_fine_zero && candidate & 15 != 0 {
        return Err(BapError::InputError);
    }
    Ok(())
}

fn parse_bap_parameters(provider_file: &[u8]) -> Result<BapParameters, BapError> {
    let document: Value =
        serde_json::from_slice(provider_file).map_err(|_| BapError::ParameterError)?;
    if document.get("schema").and_then(Value::as_str) != Some(SELECTED_PROVIDER_SCHEMA) {
        return Err(BapError::ParameterError);
    }
    if document.get("selected_profile").and_then(Value::as_str) != Some(SELECTED_PROVIDER_PROFILE) {
        return Err(BapError::UnsupportedProfile);
    }
    let parameter_value = document
        .get("parameters")
        .ok_or(BapError::ParameterUnavailable)?;
    let parameter_object = parameter_value
        .as_object()
        .ok_or(BapError::ParameterError)?;
    let parameter_keys = [
        "slow_decay",
        "fast_decay",
        "slow_gain",
        "db_knee",
        "floor",
        "fast_gain",
        "log_addition",
        "bands",
        "hearing_48k",
        "bap_pointer",
    ];
    if parameter_keys
        .iter()
        .any(|key| !parameter_object.contains_key(*key))
    {
        return Err(BapError::ParameterUnavailable);
    }
    if parameter_object.len() != parameter_keys.len() {
        return Err(BapError::ParameterError);
    }

    let slow_decay = i16_array::<4>(parameter_object, "slow_decay")?;
    let fast_decay = i16_array::<4>(parameter_object, "fast_decay")?;
    let slow_gain = i16_array::<4>(parameter_object, "slow_gain")?;
    let db_knee = i16_array::<4>(parameter_object, "db_knee")?;
    let floor = i16_array::<8>(parameter_object, "floor")?;
    let fast_gain = i16_array::<8>(parameter_object, "fast_gain")?;
    let log_addition = i16_array::<256>(parameter_object, "log_addition")?;
    let bands = bands_array(parameter_object)?;
    let hearing_48k = i16_array::<50>(parameter_object, "hearing_48k")?;
    let bap_pointer = u8_array::<64>(parameter_object, "bap_pointer")?;
    if bap_pointer.iter().any(|value| *value > 15) {
        return Err(BapError::ParameterError);
    }
    validate_band_coverage(&bands)?;

    Ok(BapParameters {
        slow_decay,
        fast_decay,
        slow_gain,
        db_knee,
        floor,
        fast_gain,
        log_addition,
        bands,
        hearing_48k,
        bap_pointer,
    })
}

fn i16_array<const N: usize>(
    parameters: &Map<String, Value>,
    key: &str,
) -> Result<[i16; N], BapError> {
    let values = parameters
        .get(key)
        .ok_or(BapError::ParameterUnavailable)?
        .as_array()
        .ok_or(BapError::ParameterError)?;
    if values.len() != N {
        return Err(BapError::ParameterError);
    }
    let mut output = [0_i16; N];
    for (destination, value) in output.iter_mut().zip(values) {
        let signed = value.as_i64().ok_or(BapError::ParameterError)?;
        *destination = i16::try_from(signed).map_err(|_| BapError::ParameterError)?;
    }
    Ok(output)
}

fn u8_array<const N: usize>(
    parameters: &Map<String, Value>,
    key: &str,
) -> Result<[u8; N], BapError> {
    let values = parameters
        .get(key)
        .ok_or(BapError::ParameterUnavailable)?
        .as_array()
        .ok_or(BapError::ParameterError)?;
    if values.len() != N {
        return Err(BapError::ParameterError);
    }
    let mut output = [0_u8; N];
    for (destination, value) in output.iter_mut().zip(values) {
        let unsigned = value.as_u64().ok_or(BapError::ParameterError)?;
        *destination = u8::try_from(unsigned).map_err(|_| BapError::ParameterError)?;
    }
    Ok(output)
}

fn bands_array(parameters: &Map<String, Value>) -> Result<[BapBand; 50], BapError> {
    let values = parameters
        .get("bands")
        .ok_or(BapError::ParameterUnavailable)?
        .as_array()
        .ok_or(BapError::ParameterError)?;
    if values.len() != 50 {
        return Err(BapError::ParameterError);
    }
    let mut output = [BapBand { start: 0, size: 1 }; 50];
    for (destination, value) in output.iter_mut().zip(values) {
        let band = value.as_object().ok_or(BapError::ParameterError)?;
        if band.len() != 2 || !band.contains_key("start") || !band.contains_key("size") {
            return Err(BapError::ParameterError);
        }
        let start = band
            .get("start")
            .and_then(Value::as_u64)
            .ok_or(BapError::ParameterError)?;
        let size = band
            .get("size")
            .and_then(Value::as_u64)
            .ok_or(BapError::ParameterError)?;
        *destination = BapBand {
            start: u16::try_from(start).map_err(|_| BapError::ParameterError)?,
            size: u8::try_from(size).map_err(|_| BapError::ParameterError)?,
        };
        if destination.size == 0 {
            return Err(BapError::ParameterError);
        }
    }
    Ok(output)
}

fn validate_band_coverage(bands: &[BapBand; 50]) -> Result<(), BapError> {
    let mut next_start = 0_u32;
    for band in bands {
        if u32::from(band.start) != next_start || band.size == 0 {
            return Err(BapError::ParameterError);
        }
        next_start += u32::from(band.size);
    }
    if next_start != 253 {
        return Err(BapError::ParameterError);
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn compute_bap_row(
    input: &BapRowInput<'_>,
    parameters: &BapParameters,
) -> Result<Vec<u8>, BapError> {
    let coarse = input.candidate >> 4;
    let fine = input.candidate & 15;
    if coarse == 0 && input.caller_all_active_fine_zero {
        return Ok(vec![0; input.end_exclusive]);
    }

    let mut psd = Vec::with_capacity(input.end_exclusive);
    for exponent in input.decoded_exponents.iter().copied() {
        let value = 3_072_i32 - 128_i32 * i32::from(exponent);
        psd.push(i16::try_from(value).map_err(|_| BapError::InputError)?);
    }

    let active_band_count = band_count_for_end(input.end_exclusive, &parameters.bands)?;
    let mut band_psd = [0_i16; 50];
    for (band_index, band) in parameters.bands.iter().take(active_band_count).enumerate() {
        let start = usize::from(band.start);
        let band_end = start + usize::from(band.size);
        let active_end = band_end.min(input.end_exclusive);
        if start >= active_end {
            return Err(BapError::ParameterError);
        }
        let mut combined = psd[start];
        for power_spectral_density in psd.iter().take(active_end).skip(start + 1) {
            combined = log_add(combined, *power_spectral_density, &parameters.log_addition);
        }
        band_psd[band_index] = combined;
    }

    let mut excitation = [0_i16; 50];
    let fast_gain = parameters.fast_gain[usize::from(input.fast_gain_code)];
    let slow_gain = parameters.slow_gain[usize::from(input.slow_gain_code)];
    let fast_decay = parameters.fast_decay[usize::from(input.fast_decay_code)];
    let slow_decay = parameters.slow_decay[usize::from(input.slow_decay_code)];

    let mut lowcomp = update_lowcompensation(0, band_psd[0], band_psd[1], 0);
    excitation[0] = s16_sub(s16_sub(band_psd[0], fast_gain), lowcomp);
    lowcomp = update_lowcompensation(lowcomp, band_psd[1], band_psd[2], 1);
    excitation[1] = s16_sub(s16_sub(band_psd[1], fast_gain), lowcomp);

    let mut begin = 7_usize;
    let mut fast_leak = 0_i16;
    let mut slow_leak = 0_i16;
    for band_index in 2..=6 {
        let lfe_last_band = active_band_count == 7 && band_index == 6;
        if !lfe_last_band {
            lowcomp = update_lowcompensation(
                lowcomp,
                band_psd[band_index],
                band_psd[band_index + 1],
                band_index,
            );
        }
        fast_leak = s16_sub(band_psd[band_index], fast_gain);
        slow_leak = s16_sub(band_psd[band_index], slow_gain);
        excitation[band_index] = s16_sub(fast_leak, lowcomp);
        if !lfe_last_band && band_psd[band_index] <= band_psd[band_index + 1] {
            begin = band_index + 1;
            break;
        }
    }

    for band_index in begin..active_band_count.min(22) {
        let lfe_last_band = active_band_count == 7 && band_index == 6;
        if !lfe_last_band {
            lowcomp = update_lowcompensation(
                lowcomp,
                band_psd[band_index],
                band_psd[band_index + 1],
                band_index,
            );
        }
        fast_leak = s16_sub(fast_leak, fast_decay).max(s16_sub(band_psd[band_index], fast_gain));
        slow_leak = s16_sub(slow_leak, slow_decay).max(s16_sub(band_psd[band_index], slow_gain));
        excitation[band_index] = s16_sub(fast_leak, lowcomp).max(slow_leak);
    }

    for band_index in 22..active_band_count {
        fast_leak = s16_sub(fast_leak, fast_decay).max(s16_sub(band_psd[band_index], fast_gain));
        slow_leak = s16_sub(slow_leak, slow_decay).max(s16_sub(band_psd[band_index], slow_gain));
        excitation[band_index] = fast_leak.max(slow_leak);
    }

    let snr = (i32::from(coarse) - 15) * 64 + i32::from(fine) * 4;
    let floor = parameters.floor[usize::from(input.floor_code)];
    let db_knee = parameters.db_knee[usize::from(input.db_per_bit_code)];
    let mut bap = vec![0_u8; input.end_exclusive];
    for (band_index, band) in parameters.bands.iter().take(active_band_count).enumerate() {
        let band_power = band_psd[band_index];
        let knee = if band_power < db_knee {
            (i32::from(db_knee) - i32::from(band_power)) / 4
        } else {
            0
        };
        let masked_excitation = i32::from(excitation[band_index]) + knee;
        let mask = masked_excitation
            .max(i32::from(parameters.hearing_48k[band_index]))
            .min(i32::from(i16::MAX));
        let adjusted = (mask - snr - i32::from(floor)).max(0);
        let adjusted = (adjusted & 0x1fe0) + i32::from(floor);
        let start = usize::from(band.start);
        let band_end = (start + usize::from(band.size)).min(input.end_exclusive);
        for bin in start..band_end {
            let address = ((i32::from(psd[bin]) - adjusted) >> 5).clamp(0, 63);
            let address = usize::try_from(address).map_err(|_| BapError::ParameterError)?;
            bap[bin] = parameters.bap_pointer[address];
        }
    }
    Ok(bap)
}

fn band_count_for_end(end_exclusive: usize, bands: &[BapBand; 50]) -> Result<usize, BapError> {
    let last_bin = end_exclusive.checked_sub(1).ok_or(BapError::InputError)?;
    bands
        .iter()
        .position(|band| {
            let start = usize::from(band.start);
            start <= last_bin && last_bin < start + usize::from(band.size)
        })
        .map(|index| index + 1)
        .ok_or(BapError::ParameterError)
}

fn log_add(a: i16, b: i16, table: &[i16; 256]) -> i16 {
    let difference = (i32::from(a) - i32::from(b)).abs();
    let address = usize::try_from((difference / 2).min(255))
        .expect("a log-add table address is clamped to 255");
    let maximum = i32::from(a.max(b));
    saturate_i16(maximum + i32::from(table[address]))
}

fn update_lowcompensation(a: i16, b0: i16, b1: i16, band_index: usize) -> i16 {
    let a = i32::from(a);
    let b0 = i32::from(b0);
    let b1 = i32::from(b1);
    let next = if band_index < 7 {
        if b0 + 256 == b1 {
            384
        } else if b0 > b1 {
            (a - 64).max(0)
        } else {
            a
        }
    } else if band_index < 20 {
        if b0 + 256 == b1 {
            320
        } else if b0 > b1 {
            (a - 64).max(0)
        } else {
            a
        }
    } else {
        (a - 128).max(0)
    };
    saturate_i16(next.min(i32::from(i16::MAX)))
}

fn s16_sub(left: i16, right: i16) -> i16 {
    saturate_i16(i32::from(left) - i32::from(right))
}

fn saturate_i16(value: i32) -> i16 {
    let clamped = value.clamp(i32::from(i16::MIN), i32::from(i16::MAX));
    i16::try_from(clamped).expect("the value is clamped to the I16 range")
}
