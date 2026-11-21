//! Programme Leq(A) from the five full-band channels of the 48 kHz render.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct AWeightedLoudness {
    delays: [[[f64; 2]; 3]; 5],
    energy: f64,
    samples: u64,
}

// Bilinear transform of the IEC A-weighting prototype, normalized at 1 kHz.
const SECTIONS: [[f64; 5]; 3] = [
    [
        0.23418304260356074,
        0.4683660852071215,
        0.23418304260356074,
        -0.22455845805977914,
        0.012606625271546396,
    ],
    [1., -2., 1., -1.8938704947230707, 0.8951597690946617],
    [1., -2., 1., -1.9946144559930215, 0.9946217070140843],
];

impl AWeightedLoudness {
    pub(super) fn accumulate(&mut self, lanes: [&[u32]; 6]) {
        for (lane, ch) in [0, 1, 2, 4, 5].into_iter().enumerate() {
            for &word in lanes[ch] {
                let x = self.filter_channel(lane, f64::from(f32::from_bits(word)));
                self.energy += x * x;
            }
        }
        self.samples += lanes[0].len() as u64;
    }

    pub(super) fn filter_channel(&mut self, lane: usize, mut x: f64) -> f64 {
        let filters = &mut self.delays[lane];
        for (delay, [b0, b1, b2, a1, a2]) in filters.iter_mut().zip(SECTIONS) {
            let y = b0 * x + delay[0];
            delay[0] = b1 * x - a1 * y + delay[1];
            delay[1] = b2 * x - a2 * y;
            x = y;
        }
        x
    }
    pub(super) fn level(&self) -> Option<f32> {
        (self.samples != 0)
            .then(|| (10.0 * (self.energy / self.samples as f64).log10()).max(-120.0) as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_khz_half_amplitude_and_lfe_exclusion() {
        let tone: Vec<_> = (0..48000)
            .map(|n| (0.5 * (std::f32::consts::TAU * n as f32 / 48.0).sin()).to_bits())
            .collect();
        let silence = vec![0; tone.len()];
        let mut m = AWeightedLoudness::default();
        m.accumulate([&tone, &silence, &silence, &tone, &silence, &silence]);
        assert!((m.level().unwrap() + 9.031).abs() < 0.02);
    }
}
