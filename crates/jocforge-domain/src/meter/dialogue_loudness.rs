// pattern: Functional Core
//! Dialogue measurement on independently classified front channels of the 48 kHz render.
use super::{physical_speech::PhysicalSpeech, speech_score::speech_score};
use std::sync::Arc;

const FRONT_CHANNELS: usize = 3;
const ACTIVITY_WINDOW: usize = 960;
const REPORT_TICK: usize = 1024;
// -70 LUFS including the BS.1770 calibration offset of -0.691.
const ACTIVITY_MEAN_SQUARE: f64 = 1.172465304582298e-7;
const K_FILTER: [[f64; 5]; 2] = [
    [
        1.53512485958697,
        -2.69169618940638,
        1.19839281085285,
        -1.69065929318241,
        0.73248077421585,
    ],
    [1.0, -2.0, 1.0, -1.99004745483398, 0.99007225036621],
];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct ActivityWindow {
    front_energy: [f64; FRONT_CHANNELS],
    programme_energy: f64,
    samples: usize,
}

impl ActivityWindow {
    fn active(&self) -> bool {
        self.samples > 0 && self.programme_energy > self.samples as f64 * ACTIVITY_MEAN_SQUARE
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct SpeechChannel {
    features: PhysicalSpeech,
    history: Vec<Arc<[[f64; 7]]>>,
    pending: Vec<[f64; 7]>,
}

impl SpeechChannel {
    fn push(&mut self, words: &[u32]) {
        self.features.push(words, |row| {
            self.pending.push(row);
            if self.pending.len() == 32 {
                self.history.push(std::mem::take(&mut self.pending).into());
            }
        });
    }

    fn rows(&self) -> impl Iterator<Item = [f64; 7]> {
        self.history
            .iter()
            .flat_map(|chunk| chunk.iter())
            .chain(&self.pending)
            .copied()
    }

    fn decisions(&self) -> Vec<bool> {
        four_frame_decisions(self.rows().map(speech_score))
    }
}

fn four_frame_decisions(scores: impl Iterator<Item = f64>) -> Vec<bool> {
    let mut history = [0.0; 4];
    scores
        .map(|score| {
            history.rotate_left(1);
            history[3] = score;
            history.iter().sum::<f64>() >= 0.0
        })
        .collect()
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct DialogueLoudness {
    filters: [[[f64; 2]; 2]; 5],
    pending_activity: ActivityWindow,
    windows: Vec<ActivityWindow>,
    history: Vec<Arc<[ActivityWindow]>>,
    channels: [SpeechChannel; FRONT_CHANNELS],
}

impl DialogueLoudness {
    pub(super) fn accumulate(
        &mut self,
        lanes: [&[u32]; 6],
        a_weighted: bool,
        a_meter: &mut super::a_weighted_loudness::AWeightedLoudness,
    ) {
        for (channel, words) in self.channels.iter_mut().zip(lanes) {
            channel.push(words);
        }
        for (i, &left) in lanes[0].iter().enumerate() {
            let window = &mut self.pending_activity;
            let full_band = [left, lanes[1][i], lanes[2][i], lanes[4][i], lanes[5][i]];
            for (lane, word) in full_band.into_iter().enumerate() {
                let sample = f64::from(f32::from_bits(word));
                let mut weighted = sample;
                for (delay, [b0, b1, b2, a1, a2]) in self.filters[lane].iter_mut().zip(K_FILTER) {
                    let y = b0 * weighted + delay[0];
                    delay[0] = b1 * weighted - a1 * y + delay[1];
                    delay[1] = b2 * weighted - a2 * y;
                    weighted = y;
                }
                let k_power = weighted * weighted;
                window.programme_energy += k_power * if lane < FRONT_CHANNELS { 1.0 } else { 1.41 };
                if lane < FRONT_CHANNELS {
                    window.front_energy[lane] += if a_weighted {
                        a_meter.filter_channel(lane, sample).powi(2)
                    } else {
                        k_power
                    };
                }
            }
            window.samples += 1;
            if window.samples == ACTIVITY_WINDOW {
                self.windows.push(std::mem::take(window));
                if self.windows.len() == 128 {
                    self.history.push(std::mem::take(&mut self.windows).into());
                }
            }
        }
    }

    pub(super) fn result(self, a_weighted: bool) -> (f64, Option<f64>) {
        if self.channels[0].rows().next().is_none() {
            return (0.0, None);
        }
        let windows: Vec<_> = self
            .history
            .iter()
            .flat_map(|chunk| chunk.iter())
            .chain(&self.windows)
            .collect();
        let samples = windows.len() * ACTIVITY_WINDOW + self.pending_activity.samples;
        let decisions = self.channels.each_ref().map(SpeechChannel::decisions);
        integrate_dialogue(&windows, &decisions, samples / REPORT_TICK, a_weighted)
    }
}

// A report unit samples the latest classifier decision. Its energy comes from
// the 20 ms blocks leaving the 102-slot history during that unit.
fn integrate_dialogue(
    windows: &[&ActivityWindow],
    decisions: &[Vec<bool>; FRONT_CHANNELS],
    normal_ticks: usize,
    a_weighted: bool,
) -> (f64, Option<f64>) {
    const HISTORY_BLOCKS: usize = 102;
    const FIRST_DECISION: usize = 98_304 / REPORT_TICK;
    const DECISION_HOP: usize = 24_576 / REPORT_TICK;
    let residue = normal_ticks % 15 * 64; // (1024 * ticks) modulo 960
    let drain_stride = REPORT_TICK.div_ceil(ACTIVITY_WINDOW - residue);
    let drain_ticks = HISTORY_BLOCKS / drain_stride + 1;
    let report_ticks = normal_ticks + drain_ticks;
    let normal_blocks = normal_ticks + normal_ticks / 15;
    let mut consumed = 0;
    let mut selected = [false; FRONT_CHANNELS];
    let mut active = true;
    let mut active_ticks = 0;
    let mut speech_blocks = 0_usize;
    let mut speech_energy = 0.0;
    for tick in 1..=report_ticks {
        let target = if tick <= normal_ticks {
            if tick >= FIRST_DECISION && (tick - FIRST_DECISION) % DECISION_HOP == 0 {
                let decision = (tick - FIRST_DECISION) / DECISION_HOP;
                selected = std::array::from_fn(|lane| decisions[lane][decision]);
            }
            tick + tick / 15
        } else {
            (consumed + drain_stride).min(normal_blocks + HISTORY_BLOCKS)
        };
        let has_speech = selected.contains(&true);
        for slot in consumed..target {
            if let Some(index) = slot.checked_sub(HISTORY_BLOCKS) {
                let window = windows[index];
                active = window.active();
                if active && has_speech {
                    speech_blocks += 1;
                    for (lane, &accepted) in selected.iter().enumerate() {
                        if accepted {
                            speech_energy += window.front_energy[lane];
                        }
                    }
                }
            }
        }
        consumed = target;
        active_ticks += usize::from(active && has_speech);
    }
    let level = (speech_energy > 0.0 && speech_blocks > 0).then(|| {
        let mean = speech_energy / (speech_blocks as f64 * ACTIVITY_WINDOW as f64);
        10.0 * mean.log10() - if a_weighted { 0.0 } else { 0.691 }
    });
    (100.0 * active_ticks as f64 / report_ticks as f64, level)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meter::a_weighted_loudness::AWeightedLoudness;

    const RENDER_FRAME: usize = 1536;
    const DECIMATION: usize = 6;

    // Flite slt, "Three people are speaking today.", unsigned mono PCM at 8 kHz.
    const VOICE: &[u8] = include_bytes!("../../tests/fixtures/dialogue/synthetic_speech.u8");

    #[test]
    fn score_history_starts_at_zero_and_releases_each_old_frame() {
        assert_eq!(four_frame_decisions([0.0].into_iter()), [true]);
        assert_eq!(
            four_frame_decisions([-4.0, 3.0, 1.0, -2.0, 0.0, 0.0, 1.0, -1.0].into_iter()),
            [false, false, true, false, true, false, false, true]
        );
    }

    #[test]
    fn weak_background_opens_the_activity_gate_only_in_the_surrounds() {
        let tone: Vec<_> = (0..RENDER_FRAME * 10)
            .map(|i| {
                let phase = std::f64::consts::TAU * 770.0 * i as f64 / 48000.0;
                ((0.0004472 * phase.sin()) as f32).to_bits()
            })
            .collect();
        let silence = vec![0; tone.len()];
        for (physical, active) in [
            (0, false),
            (1, false),
            (2, false),
            (3, false),
            (4, true),
            (5, true),
        ] {
            let mut meter = DialogueLoudness::default();
            meter.accumulate(
                std::array::from_fn(|lane| {
                    if lane == physical {
                        &tone[..]
                    } else {
                        &silence[..]
                    }
                }),
                false,
                &mut AWeightedLoudness::default(),
            );
            let last = meter.windows.last().unwrap();
            assert_eq!(last.active(), active, "physical channel {physical}");
        }
    }

    #[test]
    fn report_clock_counts_complete_units_and_each_drain_phase() {
        let active = ActivityWindow {
            front_energy: [0.0, 0.0, 0.001 * ACTIVITY_WINDOW as f64],
            programme_energy: 0.001 * ACTIVITY_WINDOW as f64,
            samples: ACTIVITY_WINDOW,
        };
        for (aus, votes, total) in [
            (63, 0, 146),
            (64, 53, 148),
            (65, 54, 149),
            (66, 39, 134),
            (67, 31, 126),
            (68, 25, 120),
            (69, 21, 116),
            (70, 62, 157),
            (71, 63, 158),
            (72, 65, 160),
            (73, 66, 161),
            (77, 46, 141),
            (81, 78, 173),
        ] {
            let ticks = aus * RENDER_FRAME / REPORT_TICK;
            let windows = vec![&active; ticks * REPORT_TICK / ACTIVITY_WINDOW];
            let decisions = std::array::from_fn(|lane| vec![lane == 2; aus]);
            let (percent, level) = integrate_dialogue(&windows, &decisions, ticks, false);
            assert_eq!(
                percent.to_bits(),
                (100.0 * votes as f64 / total as f64).to_bits(),
                "{aus} AUs"
            );
            assert_eq!(
                level.map(|value| value as f32),
                (votes > 0).then_some(-30.691),
                "{aus} AUs"
            );
        }
    }

    fn measure(voice_gain: [f32; 6], tone_lane: Option<usize>) -> (f64, Option<f64>) {
        measure_signal(voice_gain, tone_lane, &[997.0])
    }

    fn measure_signal(
        voice_gain: [f32; 6],
        tone_lane: Option<usize>,
        frequencies: &[f32],
    ) -> (f64, Option<f64>) {
        let mut meter = DialogueLoudness::default();
        let mut a_meter = AWeightedLoudness::default();
        for start in (0..VOICE.len() * 6).step_by(512) {
            let length = 512.min(VOICE.len() * 6 - start);
            let words: [[u32; 512]; 6] = std::array::from_fn(|lane| {
                std::array::from_fn(|i| {
                    let time = (start + i).min(VOICE.len() * 6 - 1);
                    let voice = (f32::from(VOICE[time / 6]) - 128.0) / 128.0;
                    let tone = if tone_lane == Some(lane) {
                        let t = std::f64::consts::TAU * time as f64 / 48000.0;
                        let tones: f64 = frequencies
                            .iter()
                            .map(|&hz| (t * f64::from(hz)).sin())
                            .sum();
                        (0.6 * tones / frequencies.len() as f64) as f32
                    } else {
                        0.0
                    };
                    (voice_gain[lane] * voice + tone).to_bits()
                })
            });
            meter.accumulate(
                std::array::from_fn(|lane| &words[lane][..length]),
                false,
                &mut a_meter,
            );
        }
        meter.result(false)
    }

    #[test]
    fn final_report_is_independent_of_input_chunk_size() {
        let voice: Vec<_> = VOICE
            .iter()
            .flat_map(|&sample| {
                let word = ((f32::from(sample) - 128.0) / 128.0).to_bits();
                [word; DECIMATION]
            })
            .collect();
        let silence = vec![0; voice.len()];
        let report = |size| {
            let mut meter = DialogueLoudness::default();
            let mut a_meter = AWeightedLoudness::default();
            for (speech, quiet) in voice.chunks(size).zip(silence.chunks(size)) {
                meter.accumulate(
                    [quiet, quiet, speech, quiet, quiet, quiet],
                    false,
                    &mut a_meter,
                );
            }
            meter.result(false)
        };
        let whole = report(voice.len());
        assert!(whole.0 > 0.0 && whole.1.is_some());
        assert_eq!(report(1997), whole);
    }

    #[test]
    fn voice_activity_and_energy_follow_independent_front_channels() {
        let (percent, level) = measure([0.0, 0.0, 1.0, 0.0, 0.0, 0.0], None);
        assert!(
            percent > 15.0,
            "synthetic centre speech detected for {percent}%"
        );
        let level = level.expect("centre speech has measurable energy");
        for lane in [0, 1] {
            let mut solo = [0.0; 6];
            solo[lane] = 1.0;
            assert_eq!(measure(solo, None), (percent, Some(level)));
        }
        let gain = std::f32::consts::FRAC_1_SQRT_2;
        for polarity in [-1.0, 1.0] {
            let (split_percent, split_level) =
                measure([gain, polarity * gain, 0.0, 0.0, 0.0, 0.0], None);
            assert!(split_percent > 15.0);
            assert!((split_level.unwrap() - level).abs() < 1.0);
        }
        let mixed = measure([0.0, 0.0, 1.0, 0.0, 0.0, 0.0], Some(0));
        let (duplicated_percent, duplicated_level) =
            measure([0.0, 1.0, 1.0, 0.0, 0.0, 0.0], Some(0));
        assert_eq!(duplicated_percent.to_bits(), mixed.0.to_bits());
        assert!((duplicated_level.unwrap() - mixed.1.unwrap() - 3.01).abs() < 0.1);
        assert!((mixed.1.unwrap() - level).abs() < 0.1);
        let (half_percent, half_level) = measure([0.0, 0.0, 0.5, 0.0, 0.0, 0.0], Some(0));
        assert!(half_percent > 15.0);
        assert!((half_level.unwrap() - level + 6.02).abs() < 1.0);
        for frequencies in [&[997.0][..], &[431.0, 1061.0][..]] {
            assert_eq!(measure_signal([0.0; 6], Some(0), frequencies), (0.0, None));
            let (cochannel_percent, _) =
                measure_signal([0.0, 0.0, 1.0, 0.0, 0.0, 0.0], Some(2), frequencies);
            assert!(
                cochannel_percent < 15.0,
                "masked speech {cochannel_percent}% for {frequencies:?}"
            );
        }
        assert_eq!(measure([0.0, 0.0, 0.0, 0.0, 1.0, 0.0], None), (0.0, None));
    }
}
