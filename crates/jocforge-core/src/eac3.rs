// pattern: Functional Core

//! Public E-AC-3 audio-block carriage primitives.
//!
//! This module is intentionally a narrow, fail-closed walker for the FFmpeg
//! core shape used by the deployed-compatible MVP.  It follows the public
//! E-AC-3 syntax in ETSI TS 102 366, clauses 4.4.3, E.1.2.3, and 6.2.2.  It
//! does not decode mantissa values; it only counts their public codeword width
//! so that `skipfld` can be inserted before the mantissas without changing the
//! decoded audio.

#![allow(
    clippy::assigning_clones,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::map_unwrap_or,
    clippy::needless_range_loop,
    clippy::too_many_arguments,
    clippy::unnecessary_map_or,
    clippy::useless_vec
)]

const FULL_CHANNELS: usize = 5;
const AUDIO_BLOCKS: usize = 6;
const MAX_BIN: usize = 256;

const BAND_ENDS: [u8; 50] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 31, 34, 37, 40, 43, 46, 49, 55, 61, 67, 73, 79, 85, 97, 109, 121, 133, 157, 181, 205,
    229, 253,
];

#[allow(dead_code)]
const BIN_BANDS: &[u8] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 28, 28, 29, 29, 29, 30, 30, 30, 31, 31, 31, 32, 32, 32, 33, 33, 33, 34, 34, 34, 35,
    35, 35, 35, 35, 36, 36, 36, 36, 36, 37, 37, 37, 37, 37, 38, 38, 38, 38, 38, 39, 39, 39, 39, 39,
    40, 40, 40, 40, 40, 40, 41, 41, 41, 41, 41, 41, 42, 42, 42, 42, 42, 42, 43, 43, 43, 43, 43, 43,
    43, 43, 44, 44, 44, 44, 44, 44, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 46, 46, 46, 46, 46,
    46, 46, 46, 46, 46, 46, 46, 46, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47,
    47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 47, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
    48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 49, 49, 49, 49, 49, 49, 49, 49, 49,
    49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49,
    49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49,
    49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49,
    49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49,
    49, 49, 49, 49, 49,
];

const LOG_ADD: &[u8] = &[
    0x40, 0x3f, 0x3e, 0x3d, 0x3c, 0x3b, 0x3a, 0x39, 0x38, 0x37, 0x36, 0x35, 0x34, 0x34, 0x33, 0x32,
    0x31, 0x30, 0x2f, 0x2f, 0x2e, 0x2d, 0x2c, 0x2c, 0x2b, 0x2a, 0x29, 0x29, 0x28, 0x27, 0x26, 0x26,
    0x25, 0x24, 0x24, 0x23, 0x23, 0x22, 0x21, 0x21, 0x20, 0x20, 0x1f, 0x1e, 0x1e, 0x1d, 0x1d, 0x1c,
    0x1c, 0x1b, 0x1b, 0x1a, 0x1a, 0x19, 0x19, 0x18, 0x18, 0x17, 0x17, 0x16, 0x16, 0x15, 0x15, 0x15,
    0x14, 0x14, 0x13, 0x13, 0x13, 0x12, 0x12, 0x12, 0x11, 0x11, 0x11, 0x10, 0x10, 0x10, 0x0f, 0x0f,
    0x0f, 0x0e, 0x0e, 0x0e, 0x0d, 0x0d, 0x0d, 0x0d, 0x0c, 0x0c, 0x0c, 0x0c, 0x0b, 0x0b, 0x0b, 0x0b,
    0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x09, 0x09, 0x09, 0x09, 0x09, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
    0x07, 0x07, 0x07, 0x07, 0x07, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x05, 0x05, 0x05,
    0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04,
    0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03,
    0x03, 0x03, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
    0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

const HEARING_THRESHOLD_48: [i32; 50] = [
    0x04d0, 0x04d0, 0x0440, 0x0400, 0x03e0, 0x03c0, 0x03b0, 0x03b0, 0x03a0, 0x03a0, 0x03a0, 0x03a0,
    0x03a0, 0x0390, 0x0390, 0x0390, 0x0380, 0x0380, 0x0370, 0x0370, 0x0360, 0x0360, 0x0350, 0x0350,
    0x0340, 0x0340, 0x0330, 0x0320, 0x0310, 0x0300, 0x02f0, 0x02f0, 0x02f0, 0x02f0, 0x0300, 0x0310,
    0x0340, 0x0390, 0x03e0, 0x0420, 0x0460, 0x0490, 0x04a0, 0x0460, 0x0440, 0x0440, 0x0520, 0x0800,
    0x0840, 0x0840,
];

const BAP: [u8; 64] = [
    0, 1, 1, 1, 1, 1, 2, 2, 3, 3, 3, 4, 4, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 9, 9, 9, 9,
    10, 10, 10, 10, 11, 11, 11, 11, 12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14, 14, 14, 14, 14,
    15, 15, 15, 15, 15, 15, 15, 15, 15,
];
const BITS_TO_READ: [u32; 16] = [0, 5, 7, 3, 7, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16];
const EXP_STRATEGIES: [[u8; 6]; 32] = [
    [1, 0, 0, 0, 0, 0],
    [1, 0, 0, 0, 0, 3],
    [1, 0, 0, 0, 2, 0],
    [1, 0, 0, 0, 3, 3],
    [2, 0, 0, 2, 0, 0],
    [2, 0, 0, 2, 0, 3],
    [2, 0, 0, 3, 2, 0],
    [2, 0, 0, 3, 3, 3],
    [2, 0, 1, 0, 0, 0],
    [2, 0, 2, 0, 0, 3],
    [2, 0, 2, 0, 2, 0],
    [2, 0, 2, 0, 3, 3],
    [2, 0, 3, 2, 0, 0],
    [2, 0, 3, 2, 0, 3],
    [2, 0, 3, 3, 2, 0],
    [2, 0, 3, 3, 3, 3],
    [3, 1, 0, 0, 0, 0],
    [3, 1, 0, 0, 0, 3],
    [3, 2, 0, 0, 2, 0],
    [3, 2, 0, 0, 3, 3],
    [3, 2, 0, 2, 0, 0],
    [3, 2, 0, 2, 0, 3],
    [3, 2, 0, 3, 2, 0],
    [3, 2, 0, 3, 3, 3],
    [3, 3, 1, 0, 0, 0],
    [3, 3, 2, 0, 0, 3],
    [3, 3, 2, 0, 2, 0],
    [3, 3, 2, 0, 3, 3],
    [3, 3, 3, 2, 0, 0],
    [3, 3, 3, 2, 0, 3],
    [3, 3, 3, 3, 2, 0],
    [3, 3, 3, 3, 3, 3],
];

#[derive(Clone)]
struct BitReader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8], position: usize) -> Self {
        Self { data, position }
    }

    fn read(&mut self, width: usize) -> Result<u32, String> {
        if self
            .position
            .checked_add(width)
            .is_none_or(|end| end > self.data.len() * 8)
        {
            return Err("E-AC-3 syntax read exceeded frame bounds".to_owned());
        }
        let mut value = 0_u32;
        for _ in 0..width {
            let byte = self.data[self.position / 8];
            let shift = 7 - self.position % 8;
            value = (value << 1) | u32::from((byte >> shift) & 1);
            self.position += 1;
        }
        Ok(value)
    }
}

#[derive(Default)]
struct BitWriter {
    bits: Vec<bool>,
}

impl BitWriter {
    fn bit(&mut self, value: bool) {
        self.bits.push(value);
    }

    fn write(&mut self, value: u64, width: usize) {
        for shift in (0..width).rev() {
            self.bit((value & (1_u64 << shift)) != 0);
        }
    }

    fn copy_range(&mut self, data: &[u8], start: usize, end: usize) {
        for position in start..end {
            self.bit((data[position / 8] & (0x80 >> (position % 8))) != 0);
        }
    }

    fn finish(self) -> Result<Vec<u8>, String> {
        if self.bits.len() % 8 != 0 {
            return Err("E-AC-3 rewritten frame is not byte aligned".to_owned());
        }
        let mut output = vec![0_u8; self.bits.len() / 8];
        for (index, bit) in self.bits.into_iter().enumerate() {
            if bit {
                output[index / 8] |= 0x80 >> (index % 8);
            }
        }
        Ok(output)
    }
}

struct Allocation {
    exponents: [i32; MAX_BIN],
    psd: [i32; MAX_BIN],
    band_psd: [i32; MAX_BIN],
    excitation: [i32; MAX_BIN],
    mask: [i32; MAX_BIN],
    bap: [u8; MAX_BIN],
}

impl Default for Allocation {
    fn default() -> Self {
        Self {
            exponents: [0; MAX_BIN],
            psd: [0; MAX_BIN],
            band_psd: [0; MAX_BIN],
            excitation: [0; MAX_BIN],
            mask: [0; MAX_BIN],
            bap: [0; MAX_BIN],
        }
    }
}

struct AudioFrameSyntax {
    skip_enable_bit: usize,
    block_start_bit: usize,
    block_switch: bool,
    dither: bool,
    fast_gain: bool,
    coarse_snr: u32,
    fine_snr: u32,
    exponent_strategy: [[u8; FULL_CHANNELS]; AUDIO_BLOCKS],
    lfe_exponent: [bool; AUDIO_BLOCKS],
}

#[derive(Clone)]
struct SpxInfo {
    channel_in_use: [bool; FULL_CHANNELS],
    begin_subband: usize,
    end_subband: usize,
    band_structure: [bool; 17],
}

fn log_add(left: i32, right: i32) -> i32 {
    let difference = left - right;
    let index = usize::try_from((difference.abs() >> 1).min(255)).unwrap_or(255);
    let correction = match index {
        117 => 7,
        125 => 6,
        144 => 4,
        159..=161 => 2,
        178..=191 => 1,
        210..=235 => 0,
        _ => i32::from(LOG_ADD[index]),
    };
    if difference >= 0 {
        left + correction
    } else {
        right + correction
    }
}

fn band_index(bin: usize) -> usize {
    BAND_ENDS
        .iter()
        .position(|end| bin < usize::from(*end))
        .unwrap_or(BAND_ENDS.len() - 1)
}

fn spx_subband_range(begin_code: usize, end_code: usize) -> Result<(usize, usize), String> {
    if begin_code > 7 || end_code > 7 {
        return Err("E-AC-3 SPX frequency code is outside its three-bit field".to_owned());
    }
    let begin = if begin_code < 6 {
        begin_code + 2
    } else {
        begin_code * 2 - 3
    };
    let end = if end_code < 3 {
        end_code + 5
    } else {
        end_code * 2 + 3
    };
    if begin >= end || end > 17 {
        return Err(format!(
            "E-AC-3 SPX subband range is invalid: begin_code={begin_code} end_code={end_code} derived={begin}..{end}"
        ));
    }
    Ok((begin, end))
}

fn spx_band_count(structure: &[bool; 17], begin: usize, end: usize) -> usize {
    structure[begin..end]
        .iter()
        .filter(|merged| !**merged)
        .count()
}

fn read_spx_coordinates(reader: &mut BitReader<'_>, band_count: usize) -> Result<(), String> {
    reader.read(5)?;
    reader.read(2)?;
    for _ in 0..band_count {
        reader.read(4)?;
        reader.read(2)?;
    }
    Ok(())
}

fn read_spx(
    reader: &mut BitReader<'_>,
    block: usize,
    previous: Option<&SpxInfo>,
    first_coordinates: &mut [bool; FULL_CHANNELS],
) -> Result<Option<SpxInfo>, String> {
    let spx_strategy = if block == 0 {
        true
    } else {
        reader.read(1)? != 0
    };
    if !spx_strategy {
        return Ok(previous.cloned());
    }
    if reader.read(1)? == 0 {
        *first_coordinates = [true; FULL_CHANNELS];
        return Ok(None);
    }
    let mut channel_in_use = [false; FULL_CHANNELS];
    for value in &mut channel_in_use {
        *value = reader.read(1)? != 0;
    }
    reader.read(2)?; // start_copy_frequency_code
    let begin_code = usize::try_from(reader.read(3)?).unwrap_or(0);
    let end_code = usize::try_from(reader.read(3)?).unwrap_or(0);
    let (begin_subband, end_subband) = spx_subband_range(begin_code, end_code)?;
    let mut band_structure = previous
        .filter(|info| info.begin_subband == begin_subband && info.end_subband == end_subband)
        .map_or(
            [
                false, false, false, false, false, false, false, false, true, false, true, false,
                true, false, true, false, true,
            ],
            |info| info.band_structure,
        );
    if reader.read(1)? != 0 {
        for subband in begin_subband + 1..end_subband {
            band_structure[subband] = reader.read(1)? != 0;
        }
    }
    let band_count = if block == 0 {
        (begin_subband + 1..end_subband)
            .filter(|subband| !band_structure[*subband])
            .count()
            + 1
    } else {
        spx_band_count(&band_structure, begin_subband, end_subband)
    };
    for channel in 0..FULL_CHANNELS {
        if channel_in_use[channel] {
            let exists = if block == 0 || first_coordinates[channel] {
                true
            } else {
                reader.read(1)? != 0
            };
            if exists {
                read_spx_coordinates(reader, band_count)?;
            } else if previous
                .and_then(|info| info.channel_in_use.get(channel))
                .copied()
                != Some(true)
            {
                return Err("E-AC-3 SPX coordinates were reused without a prior value".to_owned());
            }
            first_coordinates[channel] = false;
        } else {
            first_coordinates[channel] = true;
        }
    }
    Ok(Some(SpxInfo {
        channel_in_use,
        begin_subband,
        end_subband,
        band_structure,
    }))
}

fn low_compensation(value: i32, first: i32, second: i32, bin: usize) -> i32 {
    if bin < 7 {
        if first + 256 == second {
            384
        } else if first > second {
            (value - 64).max(0)
        } else {
            value
        }
    } else if bin < 20 {
        if first + 256 == second {
            320
        } else if first > second {
            (value - 64).max(0)
        } else {
            value
        }
    } else {
        (value - 128).max(0)
    }
}

fn ungroup_exponents(allocation: &mut Allocation, initial: i32, groups: &[i32], strategy: u8) {
    let group_size = if strategy == 3 {
        4
    } else {
        usize::from(strategy)
    };
    let mut exponent = initial;
    let mut index = 1_usize;
    allocation.exponents[0] = exponent;
    for group in groups {
        for delta in [group / 25 - 2, group % 25 / 5 - 2, group % 5 - 2] {
            exponent += delta;
            for _ in 0..group_size {
                if index < MAX_BIN {
                    allocation.exponents[index] = exponent;
                }
                index += 1;
            }
        }
    }
    for bin in 0..index.min(MAX_BIN) {
        allocation.psd[bin] = 3072 - (allocation.exponents[bin] << 7);
    }
    let mut bin = 0_usize;
    let mut band = band_index(0);
    while bin < index.min(MAX_BIN) {
        let last = usize::from(BAND_ENDS[band]).min(index);
        allocation.band_psd[band] = allocation.psd[bin];
        bin += 1;
        while bin < last {
            allocation.band_psd[band] = log_add(allocation.band_psd[band], allocation.psd[bin]);
            bin += 1;
        }
        band += 1;
        if index <= last {
            break;
        }
    }
}

fn allocate(allocation: &mut Allocation, end: usize, snr: i32) {
    let band_start = band_index(0);
    let band_end = band_index(end.saturating_sub(1)) + 1;
    let mut low_comp = low_compensation(0, allocation.band_psd[0], allocation.band_psd[1], 0);
    allocation.excitation[0] = allocation.band_psd[0] - 0x280 - low_comp;
    low_comp = low_compensation(low_comp, allocation.band_psd[1], allocation.band_psd[2], 1);
    allocation.excitation[1] = allocation.band_psd[1] - 0x280 - low_comp;
    let mut fast_leak = 0_i32;
    let mut slow_leak = 0_i32;
    let mut begin = 7_usize;
    for bin in 2..7 {
        if band_end != 7 || bin != 6 {
            low_comp = low_compensation(
                low_comp,
                allocation.band_psd[bin],
                allocation.band_psd[bin + 1],
                bin,
            );
        }
        fast_leak = allocation.band_psd[bin] - 0x280;
        slow_leak = allocation.band_psd[bin] - 0x4d8;
        allocation.excitation[bin] = fast_leak - low_comp;
        if (band_end != 7 || bin != 6) && allocation.band_psd[bin] <= allocation.band_psd[bin + 1] {
            begin = bin + 1;
            break;
        }
    }
    let middle_end = band_end.min(22);
    for bin in begin..middle_end {
        if band_end != 7 || bin != 6 {
            low_comp = low_compensation(
                low_comp,
                allocation.band_psd[bin],
                allocation.band_psd[bin + 1],
                bin,
            );
        }
        fast_leak = (fast_leak - 0x53).max(allocation.band_psd[bin] - 0x280);
        slow_leak = (slow_leak - 0x13).max(allocation.band_psd[bin] - 0x4d8);
        allocation.excitation[bin] = (fast_leak - low_comp).max(slow_leak);
    }
    for bin in 22..band_end {
        fast_leak = (fast_leak - 0x53).max(allocation.band_psd[bin] - 0x280);
        slow_leak = (slow_leak - 0x13).max(allocation.band_psd[bin] - 0x4d8);
        allocation.excitation[bin] = fast_leak.max(slow_leak);
    }
    for band in band_start..band_end {
        if allocation.band_psd[band] < 0x900 {
            allocation.excitation[band] += (0x900 - allocation.band_psd[band]) >> 2;
        }
        allocation.mask[band] = allocation.excitation[band].max(HEARING_THRESHOLD_48[band]);
    }
    let mut bin = 0_usize;
    let mut band = 0_usize;
    while bin < end {
        let last = usize::from(BAND_ENDS[band]).min(end);
        let mut masked = allocation.mask[band] - snr - (-2048);
        if masked < 0 {
            masked = 0;
        }
        masked = (masked & 0x1fe0) - 2048;
        while bin < last {
            let address = ((allocation.psd[bin] - masked) >> 5).clamp(0, 63) as usize;
            allocation.bap[bin] = BAP[address];
            bin += 1;
        }
        band += 1;
    }
    for value in allocation.bap.iter_mut().skip(bin) {
        *value = 0;
    }
}

fn mantissa_bits(
    allocation: &Allocation,
    end: usize,
    first: &mut i64,
    second: &mut i64,
    fourth: &mut i64,
) -> u32 {
    let mut bits = 0_u32;
    let mut counts = [0_i64; 4];
    for &bap in allocation.bap.iter().take(end) {
        if (1..5).contains(&bap) {
            counts[usize::from(bap - 1)] += 1;
        } else {
            bits += BITS_TO_READ[usize::from(bap)];
        }
    }
    bits += (((*first + counts[0]) / 3) as u32) * 5;
    bits += (((*second + counts[1]) / 3) as u32) * 7;
    bits += (counts[2] as u32) * 3;
    bits += (((*fourth + counts[3]) / 2) as u32) * 7;
    *first = (*first + counts[0]) % 3;
    *second = (*second + counts[1]) % 3;
    *fourth = (*fourth + counts[3]) % 2;
    bits
}

fn parse_bsi_end(frame: &[u8]) -> Result<(usize, usize), String> {
    let mut reader = BitReader::new(frame, 45);
    let acmod = ((frame[4] >> 1) & 0x07) as usize;
    let strmtyp = (frame[2] >> 6) & 0x03;
    if acmod != 7 || strmtyp != 0 {
        return Err("deployed-compatible carriage requires independent 5.1 I0".to_owned());
    }
    reader.read(5)?;
    if reader.read(1)? != 0 {
        reader.read(8)?;
    }
    if reader.read(1)? != 0 {
        return Err("FFmpeg I0 carries unsupported mixing metadata".to_owned());
    }
    if reader.read(1)? != 0 {
        return Err("FFmpeg I0 carries unsupported informational metadata".to_owned());
    }
    // fscod=0 and six blocks mean convsync is not present.
    let addbsi_bit = reader.position;
    let addbsi = reader.read(1)? as usize;
    Ok((addbsi_bit, addbsi))
}

fn parse_audio_syntax(frame: &[u8], bsi_end_bit: usize) -> Result<AudioFrameSyntax, String> {
    let mut reader = BitReader::new(frame, bsi_end_bit);
    if reader.read(1)? != 0 || reader.read(1)? != 0 {
        return Err("FFmpeg I0 uses unsupported audio-frame exponent/AHT syntax".to_owned());
    }
    let snr_strategy = reader.read(2)?;
    if snr_strategy != 0 {
        return Err("FFmpeg I0 uses unsupported frame SNR strategy".to_owned());
    }
    let transproce = reader.read(1)? != 0;
    let block_switch = reader.read(1)? != 0;
    let dither = reader.read(1)? != 0;
    let bamode = reader.read(1)? != 0;
    let fast_gain = reader.read(1)? != 0;
    let delta = reader.read(1)? != 0;
    let skip_enable_bit = reader.position;
    reader.read(1)?;
    let spx = reader.read(1)? != 0;
    if transproce || bamode || delta || spx {
        return Err("FFmpeg I0 uses unsupported audio-frame coding tools".to_owned());
    }
    if reader.read(1)? != 0 {
        return Err("FFmpeg I0 uses coupling".to_owned());
    }
    let mut coupling_strategy = [false; AUDIO_BLOCKS];
    for block in 1..AUDIO_BLOCKS {
        coupling_strategy[block] = reader.read(1)? != 0;
        if coupling_strategy[block] && reader.read(1)? != 0 {
            return Err("FFmpeg I0 uses coupling".to_owned());
        }
    }
    let mut exponent_strategy = [[0_u8; FULL_CHANNELS]; AUDIO_BLOCKS];
    for channel in 0..FULL_CHANNELS {
        let code = usize::try_from(reader.read(5)?).unwrap_or(31);
        exponent_strategy
            .iter_mut()
            .zip(EXP_STRATEGIES[code])
            .for_each(|(row, value)| row[channel] = value);
    }
    let mut lfe_exponent = [false; AUDIO_BLOCKS];
    for value in &mut lfe_exponent {
        *value = reader.read(1)? != 0;
    }
    for _ in 0..FULL_CHANNELS {
        reader.read(5)?;
    }
    let coarse_snr = reader.read(6)?;
    let fine_snr = reader.read(4)?;
    if reader.read(1)? != 0 {
        let words = frame.len() / 2;
        let log2 = usize::BITS as usize - (words.saturating_sub(1)).leading_zeros() as usize;
        let bits = (AUDIO_BLOCKS - 1) * (4 + log2);
        reader.read(bits)?;
    }
    Ok(AudioFrameSyntax {
        skip_enable_bit,
        block_start_bit: reader.position,
        block_switch,
        dither,
        fast_gain,
        coarse_snr,
        fine_snr,
        exponent_strategy,
        lfe_exponent,
    })
}

fn walk_block(
    frame: &[u8],
    syntax: &AudioFrameSyntax,
    block: usize,
    position: usize,
    allocations: &mut [Allocation; FULL_CHANNELS],
    lfe_allocation: &mut Allocation,
    end_mantissas: &mut [usize; FULL_CHANNELS],
    lfe_has_exponents: &mut bool,
    previous_spx: &mut Option<SpxInfo>,
    first_spx_coordinates: &mut [bool; FULL_CHANNELS],
) -> Result<(usize, usize), String> {
    let mut reader = BitReader::new(frame, position);
    if syntax.block_switch {
        reader.read(FULL_CHANNELS)?;
    }
    if syntax.dither {
        reader.read(FULL_CHANNELS)?;
    }
    if reader.read(1)? != 0 {
        reader.read(8)?;
    }
    let spx = read_spx(
        &mut reader,
        block,
        previous_spx.as_ref(),
        first_spx_coordinates,
    )?;
    *previous_spx = spx.clone();
    for channel in 0..FULL_CHANNELS {
        let strategy = syntax.exponent_strategy[block][channel];
        let spx_active = spx
            .as_ref()
            .map(|info| info.channel_in_use[channel])
            .unwrap_or(false);
        if spx_active {
            end_mantissas[channel] = [
                25, 37, 49, 61, 73, 85, 97, 109, 121, 133, 145, 157, 169, 181, 193, 205, 217, 229,
            ]
            .get(spx.as_ref().map(|info| info.begin_subband).unwrap_or(0))
            .copied()
            .ok_or_else(|| "E-AC-3 SPX mantissa range is invalid".to_owned())?;
        } else if strategy != 0 {
            let code = usize::try_from(reader.read(6)?).unwrap_or(63);
            end_mantissas[channel] = (code + 12) * 3 + 37;
        }
    }
    for channel in 0..FULL_CHANNELS {
        let strategy = syntax.exponent_strategy[block][channel];
        let should_read = block == 0 || strategy != 0;
        if should_read {
            if strategy == 0 {
                return Err("E-AC-3 block-zero exponent strategy reuse is unsupported".to_owned());
            }
            let group_count = (end_mantissas[channel] as i32
                + [-1, 2, 8][usize::from(strategy - 1)])
                / [3, 6, 12][usize::from(strategy - 1)];
            let initial = i32::try_from(reader.read(4)?).unwrap_or(0);
            let mut groups = Vec::with_capacity(usize::try_from(group_count).unwrap_or(0));
            for _ in 0..group_count {
                groups.push(i32::try_from(reader.read(7)?).unwrap_or(0));
            }
            reader.read(2)?;
            ungroup_exponents(&mut allocations[channel], initial, &groups, strategy);
        }
    }
    if syntax.lfe_exponent[block] {
        let initial = i32::try_from(reader.read(4)?).unwrap_or(0);
        let groups = [
            i32::try_from(reader.read(7)?).unwrap_or(0),
            i32::try_from(reader.read(7)?).unwrap_or(0),
        ];
        ungroup_exponents(lfe_allocation, initial, &groups, 1);
        *lfe_has_exponents = true;
    }
    if syntax.fast_gain && reader.read(1)? != 0 {
        return Err("FFmpeg I0 uses per-block fast gain codes".to_owned());
    }
    if reader.read(1)? != 0 {
        reader.read(10)?;
    }
    let skip_point = reader.position;
    let snr = ((syntax.coarse_snr as i32 - 15) << 6) + (syntax.fine_snr as i32 * 4);
    let mut first = 2_i64;
    let mut second = 2_i64;
    let mut fourth = 1_i64;
    let mut total_mantissa_bits = 0_u32;
    for (channel, allocation) in allocations.iter_mut().enumerate() {
        if syntax.coarse_snr != 0 || syntax.fine_snr != 0 {
            allocate(allocation, end_mantissas[channel], snr);
            total_mantissa_bits += mantissa_bits(
                allocation,
                end_mantissas[channel],
                &mut first,
                &mut second,
                &mut fourth,
            );
        }
    }
    if syntax.coarse_snr != 0 || syntax.fine_snr != 0 {
        allocate(lfe_allocation, 7, snr);
        total_mantissa_bits +=
            mantissa_bits(lfe_allocation, 7, &mut first, &mut second, &mut fourth);
    }
    let next = skip_point
        .checked_add(usize::try_from(total_mantissa_bits).unwrap_or(usize::MAX))
        .ok_or_else(|| "E-AC-3 mantissa length overflow".to_owned())?;
    if next > frame.len() * 8 {
        return Err("E-AC-3 mantissa length exceeds frame".to_owned());
    }
    Ok((skip_point, next))
}

fn locate_skip_points(
    frame: &[u8],
    bsi_end_bit: usize,
) -> Result<(usize, Vec<usize>, usize), String> {
    let syntax = parse_audio_syntax(frame, bsi_end_bit)?;
    let mut allocations = std::array::from_fn(|_| Allocation::default());
    let mut lfe_allocation = Allocation::default();
    let mut end_mantissas = [253_usize; FULL_CHANNELS];
    let mut lfe_has_exponents = false;
    let mut previous_spx = None;
    let mut first_spx_coordinates = [true; FULL_CHANNELS];
    let mut position = syntax.block_start_bit;
    let mut points = Vec::with_capacity(AUDIO_BLOCKS);
    let mut ends = Vec::with_capacity(AUDIO_BLOCKS);
    for block in 0..AUDIO_BLOCKS {
        let (skip_point, next) = walk_block(
            frame,
            &syntax,
            block,
            position,
            &mut allocations,
            &mut lfe_allocation,
            &mut end_mantissas,
            &mut lfe_has_exponents,
            &mut previous_spx,
            &mut first_spx_coordinates,
        )
        .map_err(|error| {
            format!(
                "block {block} at bit {position}: {error}; previous skip points {points:?}; ends {ends:?}"
            )
        })?;
        points.push(skip_point);
        ends.push(next);
        position = next;
    }
    let _ = lfe_has_exponents;
    Ok((syntax.skip_enable_bit, points, position))
}

fn crc16(bytes: &[u8]) -> u16 {
    let mut state = 0_u16;
    for byte in bytes {
        for shift in (0..8).rev() {
            let feedback = ((state >> 15) & 1) ^ u16::from((byte >> shift) & 1);
            state <<= 1;
            if feedback != 0 {
                state ^= 0x8005;
            }
        }
    }
    state
}

fn write_frame_size(frame: &mut [u8], size: usize) -> Result<(), String> {
    if frame.len() != size || size % 2 != 0 || size / 2 - 1 > 0x7ff {
        return Err("E-AC-3 frame size is not representable".to_owned());
    }
    let words = u16::try_from(size / 2 - 1).map_err(|_| "E-AC-3 frame size overflow".to_owned())?;
    frame[2] = (frame[2] & 0xf8) | u8::try_from(words >> 8).unwrap_or(0);
    frame[3] = u8::try_from(words & 0xff).unwrap_or(0);
    Ok(())
}

fn rewrite_crc2(frame: &mut [u8]) -> Result<(), String> {
    if frame.len() < 4 {
        return Err("E-AC-3 frame is too short for crc2".to_owned());
    }
    let crc = crc16(&frame[2..frame.len() - 2]);
    let end = frame.len();
    frame[end - 2..].copy_from_slice(&crc.to_be_bytes());
    Ok(())
}

/// Extends a public E-AC-3 frame with zero-valued auxiliary padding while
/// changing only its declared frame size and CRC2.  Audio syntax is copied
/// bit-for-bit; the final zero bit remains `auxdatae=0`.
pub(crate) fn inflate_eac3_frame(frame: &[u8], target_bytes: usize) -> Result<Vec<u8>, String> {
    if frame.len() < 8 || frame.get(0..2) != Some(&[0x0b, 0x77]) {
        return Err("carrier encoder did not return one E-AC-3 frame".to_owned());
    }
    let declared = ((usize::from(frame[2] & 0x07) << 8) | usize::from(frame[3])) + 1;
    if declared * 2 != frame.len() {
        return Err("carrier encoder returned an E-AC-3 frame with a mismatched size".to_owned());
    }
    if target_bytes <= frame.len() || target_bytes % 2 != 0 || target_bytes / 2 - 1 > 0x7ff {
        return Err("target E-AC-3 frame size is not a larger even public frame length".to_owned());
    }
    let old_body_bits = frame.len() * 8 - 17;
    let new_body_bits = target_bytes * 8 - 17;
    let padding_bits = new_body_bits - old_body_bits;
    let mut writer = BitWriter::default();
    writer.copy_range(frame, 0, old_body_bits);
    for _ in 0..padding_bits {
        writer.bit(false);
    }
    writer.write(0, 1); // crcrsv
    writer.write(0, 16); // crc2 placeholder
    let mut output = writer.finish()?;
    write_frame_size(&mut output, target_bytes)?;
    let auxdatae_bit = output.len() * 8 - 17;
    output[auxdatae_bit / 8] &= !(0x80 >> (auxdatae_bit % 8));
    rewrite_crc2(&mut output)?;
    Ok(output)
}

fn splice_addbsi(frame: &[u8], addbsi_bit: usize, complexity: u8) -> Result<Vec<u8>, String> {
    let total_bits = frame.len() * 8;
    let body_bits = total_bits - 17;
    let old_addbsi = BitReader::new(frame, addbsi_bit).read(1)?;
    if old_addbsi != 0 {
        return Err("deployed-compatible carriage requires an FFmpeg I0 without addbsi".to_owned());
    }
    let added_bits = 22_usize;
    let trim_at = body_bits
        .checked_sub(added_bits)
        .ok_or_else(|| "I0 frame has no room for public JOC addbsi".to_owned())?;
    if trim_at <= addbsi_bit {
        return Err("I0 frame has no room for public JOC addbsi".to_owned());
    }
    let mut writer = BitWriter::default();
    writer.copy_range(frame, 0, addbsi_bit);
    writer.write(1, 1);
    writer.write(1, 6); // addbsil + 1 = 2 bytes
    writer.write(0x01, 8); // public extension type A marker used by JOCForge
    writer.write(u64::from(complexity), 8);
    writer.copy_range(frame, addbsi_bit + 1, trim_at);
    writer.write(0, 1); // crcrsv
    writer.write(0, 16); // replaced below
    let mut output = writer.finish()?;
    let auxdatae_bit = output.len() * 8 - 17;
    output[auxdatae_bit / 8] &= !(0x80 >> (auxdatae_bit % 8));
    write_frame_size(&mut output, frame.len())?;
    rewrite_crc2(&mut output)?;
    Ok(output)
}

/// Inserts one strict/public JOC EMDF container into the I0 audio-block skip
/// field while preserving the encoded audio syntax and fixed frame size.
pub fn inject_joc_skipfld(frame: &[u8], emdf: &[u8], complexity: u8) -> Result<Vec<u8>, String> {
    if frame.len() != 4096 || frame.get(0..2) != Some(&[0x0b, 0x77]) {
        return Err(
            "deployed-compatible carriage requires one 4096-byte E-AC-3 I0 frame".to_owned(),
        );
    }
    if emdf.len() > 0x1ff {
        return Err("JOC EMDF exceeds the public 9-bit skip-field byte capacity".to_owned());
    }
    if BitReader::new(frame, frame.len() * 8 - 18).read(1)? != 0 {
        return Err(
            "deployed-compatible carriage does not rewrite existing auxiliary data".to_owned(),
        );
    }
    let (addbsi_bit, addbsi) = parse_bsi_end(frame)?;
    if addbsi != 0 {
        return Err(
            "deployed-compatible carriage requires one FFmpeg I0 without addbsi".to_owned(),
        );
    }
    let (_, _original_points, original_end) = locate_skip_points(frame, addbsi_bit + 1)?;
    let original_body_bits = frame.len() * 8 - 17;
    if original_body_bits.saturating_sub(original_end) < 22 {
        return Err("I0 frame leaves insufficient public padding for JOC addbsi".to_owned());
    }
    let with_addbsi = splice_addbsi(frame, addbsi_bit, complexity)?;
    let bsi_end = addbsi_bit + 23;
    let (skip_enable_bit, points, last_audio_bit) = locate_skip_points(&with_addbsi, bsi_end)?;
    let total_bits = with_addbsi.len() * 8;
    let body_bits = total_bits - 17;
    let inserted_bits = 15_usize
        .checked_add(emdf.len() * 8)
        .ok_or_else(|| "JOC skip-field insertion size overflow".to_owned())?;
    let trim_at = body_bits
        .checked_sub(inserted_bits)
        .ok_or_else(|| "I0 frame leaves no room for JOC EMDF skip-field carriage".to_owned())?;
    if last_audio_bit > trim_at || points.iter().any(|point| *point > trim_at) {
        return Err(format!(
            "I0 frame leaves insufficient public skip-field capacity: emdf={}B last_audio_bit={} trim_at={} body_bits={} inserted_bits={}",
            emdf.len(),
            last_audio_bit,
            trim_at,
            body_bits,
            inserted_bits
        ));
    }
    let mut writer = BitWriter::default();
    let mut cursor = 0_usize;
    if skip_enable_bit >= trim_at {
        return Err("skip-field enable bit is outside the preserved I0 syntax".to_owned());
    }
    writer.copy_range(&with_addbsi, cursor, skip_enable_bit);
    writer.write(1, 1);
    cursor = skip_enable_bit + 1;
    for (index, point) in points.iter().copied().enumerate() {
        if point < cursor || point > trim_at {
            return Err("E-AC-3 skip-field points are not monotone".to_owned());
        }
        writer.copy_range(&with_addbsi, cursor, point);
        if index == 0 {
            writer.write(1, 1);
            writer.write(u64::try_from(emdf.len()).unwrap_or(u64::MAX), 9);
            for byte in emdf {
                writer.write(u64::from(*byte), 8);
            }
        } else {
            writer.write(0, 1);
        }
        cursor = point;
    }
    writer.copy_range(&with_addbsi, cursor, trim_at);
    writer.write(0, 1); // crcrsv
    writer.write(0, 16); // crc2 placeholder
    let mut output = writer.finish()?;
    let auxdatae_bit = output.len() * 8 - 17;
    output[auxdatae_bit / 8] &= !(0x80 >> (auxdatae_bit % 8));
    write_frame_size(&mut output, frame.len())?;
    rewrite_crc2(&mut output)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::crc16;

    #[test]
    fn crc_zeroes_when_crc2_is_appended() {
        let mut bytes = vec![0x0b, 0x77, 0x12, 0x34, 0x56, 0x78, 0, 0];
        let crc = crc16(&bytes[2..bytes.len() - 2]);
        let end = bytes.len();
        bytes[end - 2..].copy_from_slice(&crc.to_be_bytes());
        assert_eq!(crc16(&bytes[2..]), 0);
    }
}
