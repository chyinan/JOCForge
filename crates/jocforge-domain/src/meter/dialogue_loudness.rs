//! Speech-gated programme measurement using 16 ms Earshot decisions.
use std::sync::Arc;

const FRAME: usize = 256;
const FRAMES_PER_BATCH: usize = 64;
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

#[derive(Clone, Debug, PartialEq)]
struct SpeechFrame {
    mono: [f32; FRAME],
    energy: f64,
    samples: usize,
}
impl Default for SpeechFrame {
    fn default() -> Self {
        Self {
            mono: [0.0; FRAME],
            energy: 0.0,
            samples: 0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct DialogueLoudness {
    filters: [[[f64; 2]; 2]; 5],
    current: SpeechFrame,
    frames: Vec<Arc<SpeechFrame>>,
    context: Vec<Arc<SpeechFrame>>,
    speech_energy: f64,
    speech_samples: u64,
    total: u64,
}

impl DialogueLoudness {
    pub(super) fn accumulate(
        &mut self,
        lanes: [&[u32]; 6],
        a_weighted: bool,
        a_meter: &mut super::a_weighted_loudness::AWeightedLoudness,
    ) {
        for i in 0..lanes[0].len() {
            let mut mono = 0.0;
            for (lane, (filters, ch)) in self.filters.iter_mut().zip([0, 1, 2, 4, 5]).enumerate() {
                let mut x = f64::from(f32::from_bits(lanes[ch][i]));
                mono += x * [0.22, 0.22, 0.4, 0.08, 0.08][lane];
                if a_weighted {
                    x = a_meter.filter_channel(lane, x);
                } else {
                    for (delay, [b0, b1, b2, a1, a2]) in filters.iter_mut().zip(K_FILTER) {
                        let y = b0 * x + delay[0];
                        delay[0] = b1 * x - a1 * y + delay[1];
                        delay[1] = b2 * x - a2 * y;
                        x = y;
                    }
                }
                self.current.energy += x * x * if !a_weighted && lane >= 3 { 1.41 } else { 1.0 };
            }
            // Three 48 kHz samples contribute to each 16 kHz detector input.
            self.current.mono[self.current.samples / 3] += (mono / 3.0) as f32;
            self.current.samples += 1;
            if self.current.samples == 3 * FRAME {
                self.frames
                    .push(Arc::new(std::mem::take(&mut self.current)));
                if self.frames.len() == FRAMES_PER_BATCH {
                    self.score_batch();
                }
            }
        }
    }

    fn score_batch(&mut self) {
        let mut detector = earshot::Detector::default_boxed();
        // Recreate recurrent context at batch boundaries so tentative meter
        // callbacks can be cloned or discarded without sharing mutable state.
        for frame in &self.context {
            detector.predict_f32(&frame.mono);
        }
        for frame in &self.frames {
            if detector.predict_f32(&frame.mono) >= 0.5 {
                self.speech_energy += frame.energy;
                self.speech_samples += frame.samples as u64;
            }
            self.total += frame.samples as u64;
        }
        self.context = std::mem::take(&mut self.frames);
    }

    pub(super) fn result(&self, a_weighted: bool) -> (f32, f32) {
        let mut end = self.clone();
        if end.current.samples > 0 {
            end.frames.push(Arc::new(std::mem::take(&mut end.current)));
        }
        end.score_batch();
        let percent = 100.0 * end.speech_samples as f64 / end.total.max(1) as f64;
        let energy = end.speech_energy / end.speech_samples.max(1) as f64;
        let level = 10.0 * energy.log10() - if a_weighted { 0.0 } else { 0.691 };
        (percent as f32, level.max(-120.0) as f32)
    }
}
