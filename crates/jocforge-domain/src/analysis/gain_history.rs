// pattern: Functional Core
//! Selected pass-0/gate-1/format-7 peak, gain, and Q15 history slice.

use crate::numeric::{FiniteF32, r32_add, r32_mul};

/// Samples in each caller-supplied channel row.
pub const GAIN_BLOCK_SAMPLES: usize = 256;
/// Native-order channels used by the selected statistics pass.
pub const GAIN_CHANNELS: usize = 6;
/// Full-band channels that receive gain codes; channel five is the LFE boundary.
pub const FULLBAND_GAIN_CHANNELS: usize = 5;
/// Blocks in one selected frame.
pub const GAIN_BLOCKS_PER_FRAME: usize = 6;

const CENTER_F32_BITS: u32 = 1_060_437_492;
const SIDE_MONO_F32_BITS: u32 = 1_056_959_542;

/// Explicit native reduction profile; never inferred from Rust allocation alignment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeakReductionProfile {
    AlignedFourLane,
    UnalignedScalar,
}

/// Static-rule peak trace. Aligned lane-register values are not claimed as native captures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PeakReductionTrace {
    profile: PeakReductionProfile,
    horizontal_minimum_pair_bits: Option<[u32; 2]>,
    horizontal_maximum_pair_bits: Option<[u32; 2]>,
    minimum_bits: u32,
    maximum_bits: u32,
    negated_minimum_bits: u32,
    peak_bits: u32,
}

impl PeakReductionTrace {
    pub const fn profile(self) -> PeakReductionProfile {
        self.profile
    }
    pub const fn horizontal_minimum_pair_bits(self) -> Option<[u32; 2]> {
        self.horizontal_minimum_pair_bits
    }
    pub const fn horizontal_maximum_pair_bits(self) -> Option<[u32; 2]> {
        self.horizontal_maximum_pair_bits
    }
    pub const fn minimum_bits(self) -> u32 {
        self.minimum_bits
    }
    pub const fn maximum_bits(self) -> u32 {
        self.maximum_bits
    }
    pub const fn negated_minimum_bits(self) -> u32 {
        self.negated_minimum_bits
    }
    pub const fn peak_bits(self) -> u32 {
        self.peak_bits
    }
}

/// Computed transition for one native-order channel in a selected block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GainChannelTransition {
    peak: PeakReductionTrace,
    current_peak_bits: u32,
    summary_bits: u32,
    q15_max: i16,
}

impl GainChannelTransition {
    pub const fn peak(self) -> PeakReductionTrace {
        self.peak
    }
    pub const fn current_peak_bits(self) -> u32 {
        self.current_peak_bits
    }
    pub const fn summary_bits(self) -> u32 {
        self.summary_bits
    }
    pub const fn q15_max(self) -> i16 {
        self.q15_max
    }
}

/// Gain formula diagnostics for one emitted full-band channel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GainFormulaTrace {
    weighted_peak_bits: u32,
    clipped_peak_bits: u32,
    doubled_word: u32,
    binade: i32,
    headroom: i32,
    gain: i16,
}

impl GainFormulaTrace {
    pub const fn weighted_peak_bits(self) -> u32 {
        self.weighted_peak_bits
    }
    pub const fn clipped_peak_bits(self) -> u32 {
        self.clipped_peak_bits
    }
    pub const fn doubled_word(self) -> u32 {
        self.doubled_word
    }
    pub const fn binade(self) -> i32 {
        self.binade
    }
    pub const fn headroom(self) -> i32 {
        self.headroom
    }
    pub const fn gain(self) -> i16 {
        self.gain
    }
}

/// Caller-owned selected statistics fields only; filter auxiliary state is excluded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GainHistoryState {
    previous_peak_bits: [u32; GAIN_CHANNELS],
    summary_bits: [[u32; GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME],
    q15_max: [i16; GAIN_CHANNELS],
    gain_output: [[i16; FULLBAND_GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME],
}

impl GainHistoryState {
    pub const fn cold_zero() -> Self {
        Self {
            previous_peak_bits: [0; GAIN_CHANNELS],
            summary_bits: [[0; GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME],
            q15_max: [0; GAIN_CHANNELS],
            gain_output: [[0; FULLBAND_GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME],
        }
    }

    pub const fn from_parts(
        previous_peak_bits: [u32; GAIN_CHANNELS],
        summary_bits: [[u32; GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME],
        q15_max: [i16; GAIN_CHANNELS],
        gain_output: [[i16; FULLBAND_GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME],
    ) -> Self {
        Self {
            previous_peak_bits,
            summary_bits,
            q15_max,
            gain_output,
        }
    }

    pub const fn previous_peak_bits(&self) -> &[u32; GAIN_CHANNELS] {
        &self.previous_peak_bits
    }
    pub const fn summary_bits(&self) -> &[[u32; GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME] {
        &self.summary_bits
    }
    pub const fn q15_max(&self) -> &[i16; GAIN_CHANNELS] {
        &self.q15_max
    }
    pub const fn gain_output(&self) -> &[[i16; FULLBAND_GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME] {
        &self.gain_output
    }
}

/// Output and state snapshots for one caller-specified block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GainBlockOutcome {
    summary_six_bits: [u32; GAIN_CHANNELS],
    gains_before: [i16; FULLBAND_GAIN_CHANNELS],
    gains_after: [i16; FULLBAND_GAIN_CHANNELS],
    channels: [GainChannelTransition; GAIN_CHANNELS],
    gain_traces: [GainFormulaTrace; FULLBAND_GAIN_CHANNELS],
}

impl GainBlockOutcome {
    pub const fn summary_six_bits(self) -> [u32; GAIN_CHANNELS] {
        self.summary_six_bits
    }
    pub const fn gains_before(self) -> [i16; FULLBAND_GAIN_CHANNELS] {
        self.gains_before
    }
    pub const fn gains_after(self) -> [i16; FULLBAND_GAIN_CHANNELS] {
        self.gains_after
    }
    pub const fn channels(self) -> [GainChannelTransition; GAIN_CHANNELS] {
        self.channels
    }
    pub const fn gain_traces(self) -> [GainFormulaTrace; FULLBAND_GAIN_CHANNELS] {
        self.gain_traces
    }
}

/// Processes one caller-supplied six-channel 256-sample block.
///
/// The fixed slice is pass 0, summary gate 1, format 7, five full-band outputs plus one LFE
/// statistics channel. `block_index` is the caller's selected summary/gain row. The explicit
/// `profile` chooses aligned four-lane or unaligned scalar peak reduction; it is not inferred
/// from the Rust buffer address. The sixth output slot is caller-owned and is not written.
///
/// This function does not create/advance an input clock, filter state, reset lifecycle, or
/// transaction. Samples must satisfy the finite bounded package input domain, and block_index
/// must be in `0..6`; behavior outside these preconditions has no washed native status.
///
/// # Panics
///
/// Panics if block_index is outside `0..6` or if an R32 intermediate is non-finite, which is
/// outside the selected finite sample profile.
pub fn process_gain_block(
    state: &mut GainHistoryState,
    block_index: u8,
    mask: u16,
    profile: PeakReductionProfile,
    samples: &[[f32; GAIN_BLOCK_SAMPLES]; GAIN_CHANNELS],
    gain_output_buffer: &mut [i16; FULLBAND_GAIN_CHANNELS + 1],
) -> GainBlockOutcome {
    let block = usize::from(block_index);
    let gains_before = state.gain_output[block];
    let channels = std::array::from_fn(|channel| {
        let peak = peak_reduce(&samples[channel], profile);
        let current_peak = multiply(2.0, f32::from_bits(peak.peak_bits));
        let previous_peak = f32::from_bits(state.previous_peak_bits[channel]);
        let summary = max_second(previous_peak, current_peak);
        let scaled_summary = multiply(summary, 32_768.0);
        let rounded_q15 =
            lrint_rne_i32(scaled_summary).clamp(i32::from(i16::MIN), i32::from(i16::MAX));
        let q15 = i16::try_from(rounded_q15).expect("clamped Q15 value fits I16");
        state.previous_peak_bits[channel] = current_peak.to_bits();
        state.summary_bits[block][channel] = summary.to_bits();
        state.q15_max[channel] = state.q15_max[channel].max(q15);

        GainChannelTransition {
            peak,
            current_peak_bits: current_peak.to_bits(),
            summary_bits: summary.to_bits(),
            q15_max: state.q15_max[channel],
        }
    });

    let summary_six_bits = state.summary_bits[block];
    let summary = summary_six_bits.map(f32::from_bits);
    let mut gains_after = [0_i16; FULLBAND_GAIN_CHANNELS];
    let mut gain_traces = [GainFormulaTrace {
        weighted_peak_bits: 0,
        clipped_peak_bits: 0,
        doubled_word: 0,
        binade: 0,
        headroom: 0,
        gain: 0,
    }; FULLBAND_GAIN_CHANNELS];

    for channel in 0..FULLBAND_GAIN_CHANNELS {
        let trace = gain_from_summary(&summary, channel, mask);
        gains_after[channel] = trace.gain;
        gain_traces[channel] = trace;
    }

    state.gain_output[block] = gains_after;
    gain_output_buffer[..FULLBAND_GAIN_CHANNELS].copy_from_slice(&gains_after);

    GainBlockOutcome {
        summary_six_bits,
        gains_before,
        gains_after,
        channels,
        gain_traces,
    }
}

/// Caller-owned reuse projection output with a trailing canary snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GainReuseOutput {
    rows: [[i16; FULLBAND_GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME],
    tail_canary: u32,
}

impl GainReuseOutput {
    pub const fn new(
        rows: [[i16; FULLBAND_GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME],
        tail_canary: u32,
    ) -> Self {
        Self { rows, tail_canary }
    }

    pub const fn rows(&self) -> &[[i16; FULLBAND_GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME] {
        &self.rows
    }

    pub const fn tail_canary(&self) -> u32 {
        self.tail_canary
    }
}

/// Applies the selected five-channel reuse-min projection into caller-owned rows.
/// `strategy_by_block` is the washed six-element block pattern shared by each full-band
/// channel. The sixth output element per row remains caller-owned and untouched.
pub fn apply_reuse_gain_minimum(
    gains: &[[i16; FULLBAND_GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME],
    strategy_by_block: &[i16; GAIN_BLOCKS_PER_FRAME],
    output: &mut GainReuseOutput,
) {
    for (channel, _) in gains[0].iter().enumerate() {
        let mut projected =
            std::array::from_fn::<_, GAIN_BLOCKS_PER_FRAME, _>(|block| gains[block][channel]);
        let original = projected;
        let mut anchor = None;
        for (block, gain) in original.into_iter().enumerate() {
            if strategy_by_block[block] != 0 {
                anchor = Some(block);
            } else if let Some(anchor_block) = anchor {
                if gain < projected[anchor_block] {
                    projected[anchor_block] = gain;
                }
            }
        }

        for (block, strategy) in strategy_by_block.iter().copied().enumerate() {
            if strategy != 0 {
                let segment_gain = projected[block];
                let end = strategy_by_block
                    .iter()
                    .enumerate()
                    .skip(block + 1)
                    .find_map(|(next, next_strategy)| (*next_strategy != 0).then_some(next))
                    .unwrap_or(GAIN_BLOCKS_PER_FRAME);
                projected[block..end].fill(segment_gain);
            }
        }

        for (output_row, gain) in output.rows.iter_mut().zip(projected) {
            output_row[channel] = gain;
        }
    }
}

fn peak_reduce(
    samples: &[f32; GAIN_BLOCK_SAMPLES],
    profile: PeakReductionProfile,
) -> PeakReductionTrace {
    let (minimum, maximum, min_pairs, max_pairs) = match profile {
        PeakReductionProfile::AlignedFourLane => {
            let mut minimum_lanes = [samples[0], samples[1], samples[2], samples[3]];
            let mut maximum_lanes = minimum_lanes;
            for start in (4..GAIN_BLOCK_SAMPLES).step_by(4) {
                for lane in 0..4 {
                    minimum_lanes[lane] = min_second(minimum_lanes[lane], samples[start + lane]);
                    maximum_lanes[lane] = max_second(maximum_lanes[lane], samples[start + lane]);
                }
            }
            let min_pair0 = min_second(minimum_lanes[0], minimum_lanes[2]);
            let min_pair1 = min_second(minimum_lanes[1], minimum_lanes[3]);
            let minimum = min_second(min_pair0, min_pair1);
            let max_pair0 = max_second(maximum_lanes[0], maximum_lanes[2]);
            let max_pair1 = max_second(maximum_lanes[1], maximum_lanes[3]);
            let maximum = max_second(max_pair0, max_pair1);
            (
                minimum,
                maximum,
                Some([min_pair0.to_bits(), min_pair1.to_bits()]),
                Some([max_pair0.to_bits(), max_pair1.to_bits()]),
            )
        }
        PeakReductionProfile::UnalignedScalar => {
            let mut minimum = samples[0];
            let mut maximum = samples[0];
            for sample in samples.iter().skip(1) {
                minimum = min_second(minimum, *sample);
                maximum = max_second(*sample, maximum);
            }
            (minimum, maximum, None, None)
        }
    };

    let negated_minimum = f32::from_bits(minimum.to_bits() ^ 0x8000_0000);
    let peak = max_second(negated_minimum, maximum);
    PeakReductionTrace {
        profile,
        horizontal_minimum_pair_bits: min_pairs,
        horizontal_maximum_pair_bits: max_pairs,
        minimum_bits: minimum.to_bits(),
        maximum_bits: maximum.to_bits(),
        negated_minimum_bits: negated_minimum.to_bits(),
        peak_bits: peak.to_bits(),
    }
}

fn gain_from_summary(
    summary: &[f32; GAIN_CHANNELS],
    channel: usize,
    mask: u16,
) -> GainFormulaTrace {
    let center = f32::from_bits(CENTER_F32_BITS);
    let side = f32::from_bits(SIDE_MONO_F32_BITS);
    let mut weighted = summary[channel];

    if (channel == 0 || channel == 2) && (mask & 1) == 0 {
        weighted = add(multiply(summary[1], center), weighted);
    }
    if channel == 2 && (mask & 4) == 0 {
        weighted = add(multiply(summary[4], center), weighted);
    } else if channel == 1 && mask == 1 {
        weighted = add(multiply(summary[3], side), weighted);
        weighted = add(weighted, multiply(summary[4], side));
        weighted = add(weighted, multiply(summary[0], center));
        weighted = add(weighted, multiply(summary[2], center));
    } else if channel == 0 && (mask & 4) == 0 {
        weighted = add(multiply(summary[3], center), weighted);
    }

    let clipped = min_second(max_second(-1.0, weighted), 1.0);
    let doubled_word = clipped.to_bits().wrapping_mul(2);
    let binade = if doubled_word == 0 {
        -127
    } else {
        i32::try_from(doubled_word >> 24).expect("upper exponent byte fits I32") - 126
    };
    let headroom = (-binade).clamp(0, 60);
    let gain = (headroom.min(4) - 1).max(0);

    GainFormulaTrace {
        weighted_peak_bits: weighted.to_bits(),
        clipped_peak_bits: clipped.to_bits(),
        doubled_word,
        binade,
        headroom,
        gain: i16::try_from(gain).expect("selected gain code fits I16"),
    }
}

fn min_second(first: f32, second: f32) -> f32 {
    if first < second { first } else { second }
}

fn max_second(first: f32, second: f32) -> f32 {
    if first > second { first } else { second }
}

fn finite(value: f32) -> FiniteF32 {
    FiniteF32::new(value).expect("gain history arithmetic is finite within washed domain")
}

fn multiply(first: f32, second: f32) -> f32 {
    r32_mul(finite(first), finite(second))
        .expect("gain history product remains finite within washed domain")
        .value()
}

fn add(first: f32, second: f32) -> f32 {
    r32_add(finite(first), finite(second))
        .expect("gain history sum remains finite within washed domain")
        .value()
}

fn lrint_rne_i32(value: f32) -> i32 {
    let bits = value.to_bits();
    let sign_negative = bits >> 31 != 0;
    let exponent_field = (bits >> 23) & 0xff;
    let fraction = bits & 0x007f_ffff;
    let (significand, exponent) = if exponent_field == 0 {
        (u64::from(fraction), -126_i32)
    } else {
        (
            u64::from(fraction | 0x0080_0000),
            i32::try_from(exponent_field).expect("F32 exponent field fits I32") - 127,
        )
    };
    let magnitude = if exponent < -1 {
        0_u64
    } else if exponent == -1 {
        u64::from(significand > 0x0080_0000)
    } else if exponent >= 23 {
        significand << usize::try_from(exponent - 23).expect("nonnegative exponent shift")
    } else {
        let shift = u32::try_from(23 - exponent).expect("positive significand shift");
        let integer = significand >> shift;
        let remainder = significand & ((1_u64 << shift) - 1);
        let halfway = 1_u64 << (shift - 1);
        if remainder > halfway || (remainder == halfway && integer % 2 == 1) {
            integer + 1
        } else {
            integer
        }
    };
    let signed = if sign_negative {
        -i64::try_from(magnitude).expect("Q15 rounded magnitude fits I64")
    } else {
        i64::try_from(magnitude).expect("Q15 rounded magnitude fits I64")
    };
    i32::try_from(signed).expect("selected Q15 rounded value fits I32")
}
