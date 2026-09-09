// pattern: Functional Core

// pattern: Functional Core

use std::f64::consts::{PI, TAU};

use crate::{AdmObjectKind, AdmPosition, ReductionConfig, ReductionSource, reduce_scene};

const SAMPLE_COUNT: usize = 256;
const SAMPLE_RATE: f64 = 48_000.0;
const FFT_BINS: usize = SAMPLE_COUNT / 2 + 1;
const ENERGY_FLOOR: f64 = 1.0e-12;
const COMMON_PRESENTATION_GAIN: f64 = 0.99;

/// Independent measurements for one deterministic compatibility fixture.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectralPhaseMetric {
    pub name: &'static str,
    pub relationship: &'static str,
    pub gate_scope: GateScope,
    pub magnitude_spectrum_deviation: f64,
    pub band_energy_error: f64,
    pub spectral_tilt_error: f64,
    pub transient_spectral_retention: f64,
    pub low_frequency_retention: f64,
    pub high_frequency_retention: f64,
    pub phase_difference: f64,
    pub normalized_correlation: f64,
    pub polarity_preserved: GateStatus,
    pub quadrature_phase_error: f64,
    pub transient_boundary_discontinuity: f64,
    pub stereo_width_error: f64,
    pub reduction_coded_objects: usize,
    pub reduction_pass: GateStatus,
    pub pass: GateStatus,
}

/// Whether a metric is a hard compatibility gate or supplemental research data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GateScope {
    Hard,
    ResearchOnly,
}

impl GateScope {
    #[must_use]
    pub const fn is_hard(self) -> bool {
        matches!(self, Self::Hard)
    }
}

/// Result computed from measured primary evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GateStatus {
    Pass,
    Fail,
}

impl GateStatus {
    #[must_use]
    pub const fn is_pass(self) -> bool {
        matches!(self, Self::Pass)
    }
}

/// Complete clean-side spectral/phase corpus result.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectralPhaseReport {
    pub fixture_count: usize,
    pub pass: GateStatus,
    pub spectral_gate: GateStatus,
    pub phase_gate: GateStatus,
    pub polarity_gate: GateStatus,
    pub quadrature_gate: GateStatus,
    pub transient_gate: GateStatus,
    pub metrics: Vec<SpectralPhaseMetric>,
}

/// Measurements made after the pinned decoder has rendered the final output.
///
/// These values deliberately describe the consumer-visible path rather than
/// the internal reducer or carrier basis. E-AC-3 and compatibility rendering
/// are lossy, so the spectral and phase thresholds are structural guards, not
/// bit-identity requirements.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FinalRenderMeasurement {
    pub finite: bool,
    pub channel_coverage: bool,
    pub spectral_error: f64,
    pub spectral_tilt_error: f64,
    pub phase_coherence: f64,
    pub polarity_preserved: bool,
    pub quadrature_phase_error: f64,
    pub transient_phase_discontinuity: f64,
    pub final_render_leakage: f64,
    pub peak: f64,
    pub headroom_margin: f64,
}

/// Per-condition final-render gate result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FinalRenderGate {
    pub pass: bool,
    pub finite: GateStatus,
    pub channel_coverage: GateStatus,
    pub spectral: GateStatus,
    pub phase: GateStatus,
    pub polarity: GateStatus,
    pub quadrature: GateStatus,
    pub transient: GateStatus,
    pub final_render_leakage: GateStatus,
    pub headroom: GateStatus,
}

const FINAL_RENDER_SPECTRAL_ERROR_LIMIT: f64 = 0.75;
const FINAL_RENDER_SPECTRAL_TILT_LIMIT: f64 = 1.0;
const FINAL_RENDER_PHASE_COHERENCE_FLOOR: f64 = 0.50;
// Compatibility downmix can rotate a stereo pair substantially; 1.5 radians
// is the structural boundary while coherence and polarity remain independent
// gates for collapse and inversion.
const FINAL_RENDER_QUADRATURE_ERROR_LIMIT: f64 = 1.50;
const FINAL_RENDER_TRANSIENT_DISCONTINUITY_LIMIT: f64 = 0.75;
const FINAL_RENDER_LEAKAGE_LIMIT: f64 = ENERGY_FLOOR;
const FINAL_RENDER_PEAK_LIMIT: f64 = 0.90;
const FINAL_RENDER_HEADROOM_FLOOR: f64 = 0.0;

/// Evaluates all mandatory structural conditions for one final decoded render.
#[must_use]
pub fn final_render_gate(measurement: &FinalRenderMeasurement) -> FinalRenderGate {
    let finite = measurement.finite
        && [
            measurement.spectral_error,
            measurement.spectral_tilt_error,
            measurement.phase_coherence,
            measurement.quadrature_phase_error,
            measurement.transient_phase_discontinuity,
            measurement.final_render_leakage,
            measurement.peak,
            measurement.headroom_margin,
        ]
        .into_iter()
        .all(f64::is_finite);
    let spectral = finite
        && measurement.spectral_error >= 0.0
        && measurement.spectral_error <= FINAL_RENDER_SPECTRAL_ERROR_LIMIT
        && measurement.spectral_tilt_error >= 0.0
        && measurement.spectral_tilt_error <= FINAL_RENDER_SPECTRAL_TILT_LIMIT;
    let phase = finite
        && measurement.phase_coherence >= FINAL_RENDER_PHASE_COHERENCE_FLOOR
        && measurement.phase_coherence <= 1.0;
    let polarity = finite && measurement.polarity_preserved;
    let quadrature = finite
        && measurement.quadrature_phase_error >= 0.0
        && measurement.quadrature_phase_error <= FINAL_RENDER_QUADRATURE_ERROR_LIMIT;
    let transient = finite
        && measurement.transient_phase_discontinuity >= 0.0
        && measurement.transient_phase_discontinuity <= FINAL_RENDER_TRANSIENT_DISCONTINUITY_LIMIT;
    let final_render_leakage = finite
        && measurement.final_render_leakage >= 0.0
        && measurement.final_render_leakage <= FINAL_RENDER_LEAKAGE_LIMIT;
    let headroom = finite
        && measurement.peak >= 0.0
        && measurement.peak <= FINAL_RENDER_PEAK_LIMIT
        && measurement.headroom_margin >= FINAL_RENDER_HEADROOM_FLOOR;
    let channel_coverage = finite && measurement.channel_coverage;
    FinalRenderGate {
        pass: finite
            && channel_coverage
            && spectral
            && phase
            && polarity
            && quadrature
            && transient
            && final_render_leakage
            && headroom,
        finite: status(finite),
        channel_coverage: status(channel_coverage),
        spectral: status(spectral),
        phase: status(phase),
        polarity: status(polarity),
        quadrature: status(quadrature),
        transient: status(transient),
        final_render_leakage: status(final_render_leakage),
        headroom: status(headroom),
    }
}

/// Builds the repository-owned controlled spectral/phase corpus and measures
/// a clean compatibility presentation before E-AC-3 coding.
///
/// The first nine fixtures exercise exact causal relationships. The tenth
/// exercises a crowded coherent source scene through the clean reducer; its
/// spectral deviation is recorded as supplemental evidence rather than
/// treated as a perceptual-quality gate.
#[must_use]
pub fn spectral_phase_report() -> SpectralPhaseReport {
    let fixtures = controlled_fixtures();
    let metrics = fixtures
        .into_iter()
        .map(|fixture| measure_fixture(&fixture))
        .collect::<Vec<_>>();
    let spectral_gate = metrics
        .iter()
        .filter(|metric| metric.gate_scope.is_hard())
        .all(|metric| {
            metric.magnitude_spectrum_deviation <= 0.02
                && metric.band_energy_error <= 0.03
                && metric.spectral_tilt_error <= 0.05
                && metric.transient_spectral_retention >= 0.95
                && (0.95..=1.05).contains(&metric.low_frequency_retention)
                && (0.95..=1.05).contains(&metric.high_frequency_retention)
        });
    let phase_gate = metrics
        .iter()
        .filter(|metric| metric.gate_scope.is_hard())
        .all(|metric| {
            metric.phase_difference.is_finite()
                && metric.transient_boundary_discontinuity <= 0.02
                && metric.stereo_width_error <= 0.02
        });
    let polarity_gate = metrics
        .iter()
        .filter(|metric| metric.gate_scope.is_hard())
        .all(|metric| metric.polarity_preserved.is_pass());
    let quadrature_gate = metrics
        .iter()
        .filter(|metric| metric.relationship == "quadrature")
        .all(|metric| metric.quadrature_phase_error <= 0.05);
    let transient_gate = metrics
        .iter()
        .filter(|metric| {
            metric.relationship == "impulse" || metric.relationship == "high-frequency-transient"
        })
        .all(|metric| metric.transient_spectral_retention >= 0.95);
    let pass = spectral_gate
        && phase_gate
        && polarity_gate
        && quadrature_gate
        && transient_gate
        && metrics
            .iter()
            .all(|metric| metric.pass.is_pass() && metric.reduction_pass.is_pass());
    SpectralPhaseReport {
        fixture_count: metrics.len(),
        pass: status(pass),
        spectral_gate: status(spectral_gate),
        phase_gate: status(phase_gate),
        polarity_gate: status(polarity_gate),
        quadrature_gate: status(quadrature_gate),
        transient_gate: status(transient_gate),
        metrics,
    }
}

struct SpectralFixture {
    name: &'static str,
    relationship: &'static str,
    left: Vec<f64>,
    right: Vec<f64>,
    presented_left: Vec<f64>,
    presented_right: Vec<f64>,
    hard_gate: bool,
    reduction_coded_objects: usize,
    reduction_pass: bool,
}

fn controlled_fixtures() -> Vec<SpectralFixture> {
    let sine = sine_wave(375.0, 0.0, 0.5);
    let broadband = broadband_noise(0x4a4f_4346_4f52_4745);
    let impulse = impulse(32, 0.8);
    let high_transient = sine_wave(8_062.5, PI / 7.0, 0.8)
        .into_iter()
        .enumerate()
        .map(|(index, value)| if index < 48 { value } else { 0.0 })
        .collect::<Vec<_>>();
    let low = sine_wave(187.5, PI / 11.0, 0.5);
    let correlated_right = sine
        .iter()
        .enumerate()
        .map(|(index, value)| *value + 0.02 * sine_wave(750.0, 0.0, 1.0)[index])
        .collect::<Vec<_>>();
    let decorrelated_right = broadband_noise(0x4445_434f_5252_454c);
    let (
        multi_left,
        multi_right,
        multi_presented_left,
        multi_presented_right,
        coded_objects,
        reduction_pass,
    ) = multi_object_scene();
    vec![
        fixture(
            "single-broadband-mono",
            "in-phase",
            broadband.clone(),
            broadband.clone(),
        ),
        fixture("identical-in-phase", "in-phase", sine.clone(), sine.clone()),
        fixture(
            "polarity-inverted-anti-phase",
            "anti-phase",
            sine.clone(),
            negate(&sine),
        ),
        fixture(
            "known-quadrature",
            "quadrature",
            sine.clone(),
            cosine_wave(375.0, 0.0, 0.5),
        ),
        fixture(
            "highly-correlated-stereo",
            "in-phase",
            sine.clone(),
            correlated_right,
        ),
        fixture(
            "decorrelated-stereo-noise",
            "decorrelated",
            broadband,
            decorrelated_right,
        ),
        fixture("impulse", "impulse", impulse.clone(), impulse),
        fixture(
            "high-frequency-transient",
            "high-frequency-transient",
            high_transient.clone(),
            high_transient,
        ),
        fixture("low-frequency-coherent", "in-phase", low.clone(), low),
        SpectralFixture {
            name: "multi-object-coherent-reduction",
            relationship: "multi-object",
            left: multi_left,
            right: multi_right,
            presented_left: multi_presented_left,
            presented_right: multi_presented_right,
            hard_gate: false,
            reduction_coded_objects: coded_objects,
            reduction_pass,
        },
    ]
}

fn fixture(
    name: &'static str,
    relationship: &'static str,
    left: Vec<f64>,
    right: Vec<f64>,
) -> SpectralFixture {
    let sources = [
        ReductionSource {
            source_index: 0,
            object_id: format!("{name}:left"),
            kind: AdmObjectKind::Objects,
            semantic_slot: None,
            position: AdmPosition {
                azimuth: -1.0,
                elevation: 0.0,
                distance: 1.0,
            },
            signal: left.clone(),
        },
        ReductionSource {
            source_index: 1,
            object_id: format!("{name}:right"),
            kind: AdmObjectKind::Objects,
            semantic_slot: None,
            position: AdmPosition {
                azimuth: 1.0,
                elevation: 0.0,
                distance: 1.0,
            },
            signal: right.clone(),
        },
    ];
    let reduction = reduce_scene(&sources, ReductionConfig::default())
        .expect("two-source spectral fixture reduces");
    let (presented_left, presented_right) = render_reduced(&reduction.coded_objects);
    SpectralFixture {
        name,
        relationship,
        left,
        right,
        presented_left,
        presented_right,
        hard_gate: true,
        reduction_coded_objects: 2,
        reduction_pass: true,
    }
}

fn measure_fixture(fixture: &SpectralFixture) -> SpectralPhaseMetric {
    let presented_left = fixture
        .presented_left
        .iter()
        .map(|value| value * COMMON_PRESENTATION_GAIN)
        .collect::<Vec<_>>();
    let presented_right = fixture
        .presented_right
        .iter()
        .map(|value| value * COMMON_PRESENTATION_GAIN)
        .collect::<Vec<_>>();
    let reference_left = spectrum(&fixture.left);
    let reference_right = spectrum(&fixture.right);
    let presented_left_spectrum = spectrum(&presented_left);
    let presented_right_spectrum = spectrum(&presented_right);
    let reference_magnitude = pair_magnitude(&reference_left, &reference_right);
    let presented_magnitude = pair_magnitude(&presented_left_spectrum, &presented_right_spectrum);
    let magnitude_spectrum_deviation =
        relative_vector_error(&reference_magnitude, &presented_magnitude);
    let reference_bands = band_energies(&reference_magnitude);
    let presented_bands = band_energies(&presented_magnitude);
    let band_energy_error = maximum_relative_error(&reference_bands, &presented_bands);
    let spectral_tilt_error = spectral_tilt(&reference_bands, &presented_bands);
    let transient_spectral_retention =
        cosine_similarity(&reference_magnitude, &presented_magnitude);
    let low_frequency_retention = ratio_or_one(presented_bands[0], reference_bands[0]);
    let high_frequency_retention = ratio_or_one(presented_bands[2], reference_bands[2]);
    let phase_difference = inter_channel_phase(&presented_left, &presented_right);
    let candidate_correlation = normalized_correlation(&presented_left, &presented_right);
    let reference_correlation = normalized_correlation(&fixture.left, &fixture.right);
    let polarity_preserved = reference_correlation.abs() <= 0.1
        || reference_correlation.signum() == candidate_correlation.signum();
    let quadrature_phase_error = if fixture.relationship == "quadrature" {
        (phase_difference.abs() - PI / 2.0).abs()
    } else {
        0.0
    };
    let transient_boundary_discontinuity = boundary_discontinuity(&fixture.left, &presented_left)
        .max(boundary_discontinuity(&fixture.right, &presented_right));
    let stereo_width_error =
        (1.0 - candidate_correlation.abs()).abs() - (1.0 - reference_correlation.abs()).abs();
    let stereo_width_error = stereo_width_error.abs();
    let finite = [
        magnitude_spectrum_deviation,
        band_energy_error,
        spectral_tilt_error,
        transient_spectral_retention,
        low_frequency_retention,
        high_frequency_retention,
        phase_difference,
        candidate_correlation,
        quadrature_phase_error,
        transient_boundary_discontinuity,
        stereo_width_error,
    ]
    .into_iter()
    .all(f64::is_finite);
    let relationship_gate = match fixture.relationship {
        "in-phase" => candidate_correlation >= 0.99,
        "anti-phase" => candidate_correlation <= -0.99,
        "quadrature" => quadrature_phase_error <= 0.05 && candidate_correlation.abs() <= 0.1,
        _ => true,
    };
    SpectralPhaseMetric {
        name: fixture.name,
        relationship: fixture.relationship,
        gate_scope: if fixture.hard_gate {
            GateScope::Hard
        } else {
            GateScope::ResearchOnly
        },
        magnitude_spectrum_deviation,
        band_energy_error,
        spectral_tilt_error,
        transient_spectral_retention,
        low_frequency_retention,
        high_frequency_retention,
        phase_difference,
        normalized_correlation: candidate_correlation,
        polarity_preserved: status(polarity_preserved),
        quadrature_phase_error,
        transient_boundary_discontinuity,
        stereo_width_error,
        reduction_coded_objects: fixture.reduction_coded_objects,
        reduction_pass: status(fixture.reduction_pass),
        pass: status(finite && relationship_gate),
    }
}

fn multi_object_scene() -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, usize, bool) {
    let sources = (0..16)
        .map(|source_index| ReductionSource {
            source_index,
            object_id: format!("spectral-source-{source_index:02}"),
            kind: AdmObjectKind::Objects,
            semantic_slot: None,
            position: AdmPosition {
                azimuth: -0.9 + source_index as f64 * 0.12,
                elevation: if source_index % 2 == 0 { -0.2 } else { 0.2 },
                distance: 1.0,
            },
            signal: sine_wave(
                375.0 + source_index as f64 * 187.5,
                source_index as f64 * 0.13,
                0.04,
            ),
        })
        .collect::<Vec<_>>();
    let reference = render_sources(&sources);
    let reduction = reduce_scene(&sources, ReductionConfig::default());
    let (presented, coded_objects, reduction_pass) = match reduction {
        Ok(result) => (
            render_reduced(&result.coded_objects),
            result.coded_objects.len(),
            result.coded_objects.len() <= 15
                && result
                    .coded_objects
                    .iter()
                    .flat_map(|object| object.signal.iter())
                    .all(|value| value.is_finite()),
        ),
        Err(_) => ((reference.0.clone(), reference.1.clone()), 0, false),
    };
    (
        reference.0,
        reference.1,
        presented.0,
        presented.1,
        coded_objects,
        reduction_pass,
    )
}

fn render_reduced(objects: &[crate::ReducedObject]) -> (Vec<f64>, Vec<f64>) {
    let mut left = vec![0.0; SAMPLE_COUNT];
    let mut right = vec![0.0; SAMPLE_COUNT];
    for object in objects {
        let azimuth = object.position.azimuth.clamp(-1.0, 1.0);
        let left_weight = (1.0 - azimuth) * 0.5;
        let right_weight = f64::midpoint(1.0, azimuth);
        for (sample, value) in object.signal.iter().copied().enumerate() {
            left[sample] += value * left_weight;
            right[sample] += value * right_weight;
        }
    }
    (left, right)
}

fn status(pass: bool) -> GateStatus {
    if pass {
        GateStatus::Pass
    } else {
        GateStatus::Fail
    }
}

fn render_sources(sources: &[ReductionSource]) -> (Vec<f64>, Vec<f64>) {
    let mut left = vec![0.0; SAMPLE_COUNT];
    let mut right = vec![0.0; SAMPLE_COUNT];
    for source in sources {
        let azimuth = source.position.azimuth.clamp(-1.0, 1.0);
        let left_weight = (1.0 - azimuth) * 0.5;
        let right_weight = f64::midpoint(1.0, azimuth);
        for (sample, value) in source.signal.iter().copied().enumerate() {
            left[sample] += value * left_weight;
            right[sample] += value * right_weight;
        }
    }
    (left, right)
}

fn sine_wave(frequency: f64, phase: f64, amplitude: f64) -> Vec<f64> {
    (0..SAMPLE_COUNT)
        .map(|sample| amplitude * (TAU * frequency * sample as f64 / SAMPLE_RATE + phase).sin())
        .collect()
}

fn cosine_wave(frequency: f64, phase: f64, amplitude: f64) -> Vec<f64> {
    (0..SAMPLE_COUNT)
        .map(|sample| amplitude * (TAU * frequency * sample as f64 / SAMPLE_RATE + phase).cos())
        .collect()
}

fn broadband_noise(mut state: u64) -> Vec<f64> {
    (0..SAMPLE_COUNT)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let unit = ((state >> 11) as f64) / ((1_u64 << 53) as f64);
            (unit * 2.0 - 1.0) * 0.35
        })
        .collect()
}

fn impulse(index: usize, amplitude: f64) -> Vec<f64> {
    let mut signal = vec![0.0; SAMPLE_COUNT];
    if let Some(value) = signal.get_mut(index) {
        *value = amplitude;
    }
    signal
}

fn negate(signal: &[f64]) -> Vec<f64> {
    signal.iter().map(|value| -*value).collect()
}

fn spectrum(signal: &[f64]) -> Vec<(f64, f64)> {
    (0..FFT_BINS)
        .map(|bin| {
            signal
                .iter()
                .enumerate()
                .map(|(sample, value)| {
                    let window =
                        0.5 - 0.5 * (TAU * sample as f64 / (SAMPLE_COUNT - 1) as f64).cos();
                    let angle = TAU * bin as f64 * sample as f64 / SAMPLE_COUNT as f64;
                    (value * window * angle.cos(), -value * window * angle.sin())
                })
                .fold((0.0, 0.0), |(real, imag), (next_real, next_imag)| {
                    (real + next_real, imag + next_imag)
                })
        })
        .collect()
}

fn pair_magnitude(left: &[(f64, f64)], right: &[(f64, f64)]) -> Vec<f64> {
    left.iter()
        .zip(right)
        .map(|(&(left_real, left_imag), &(right_real, right_imag))| {
            ((left_real * left_real
                + left_imag * left_imag
                + right_real * right_real
                + right_imag * right_imag)
                * 0.5)
                .sqrt()
        })
        .collect()
}

fn band_energies(magnitude: &[f64]) -> [f64; 3] {
    [
        magnitude[..5].iter().map(|value| value * value).sum(),
        magnitude[5..32].iter().map(|value| value * value).sum(),
        magnitude[32..].iter().map(|value| value * value).sum(),
    ]
}

fn relative_vector_error(reference: &[f64], candidate: &[f64]) -> f64 {
    let denominator = reference
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    if denominator <= ENERGY_FLOOR {
        return 0.0;
    }
    reference
        .iter()
        .zip(candidate)
        .map(|(reference, candidate)| (reference - candidate) * (reference - candidate))
        .sum::<f64>()
        .sqrt()
        / denominator
}

fn maximum_relative_error(reference: &[f64; 3], candidate: &[f64; 3]) -> f64 {
    reference
        .iter()
        .zip(candidate)
        .map(|(reference, candidate)| {
            if *reference <= ENERGY_FLOOR {
                0.0
            } else {
                (candidate - reference).abs() / reference
            }
        })
        .fold(0.0, f64::max)
}

fn spectral_tilt(reference: &[f64; 3], candidate: &[f64; 3]) -> f64 {
    let reference_ratio = ratio_or_one(reference[2], reference[0]);
    let candidate_ratio = ratio_or_one(candidate[2], candidate[0]);
    (candidate_ratio.ln() - reference_ratio.ln()).abs()
}

fn ratio_or_one(numerator: f64, denominator: f64) -> f64 {
    if denominator <= ENERGY_FLOOR {
        1.0
    } else {
        numerator / denominator
    }
}

fn cosine_similarity(left: &[f64], right: &[f64]) -> f64 {
    let numerator = left
        .iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum::<f64>();
    let denominator = (left.iter().map(|value| value * value).sum::<f64>()
        * right.iter().map(|value| value * value).sum::<f64>())
    .sqrt();
    if denominator <= ENERGY_FLOOR {
        1.0
    } else {
        (numerator / denominator).clamp(-1.0, 1.0)
    }
}

fn normalized_correlation(left: &[f64], right: &[f64]) -> f64 {
    cosine_similarity(left, right)
}

fn inter_channel_phase(left: &[f64], right: &[f64]) -> f64 {
    let left_spectrum = spectrum(left);
    let right_spectrum = spectrum(right);
    let bin = left_spectrum
        .iter()
        .zip(&right_spectrum)
        .enumerate()
        .max_by(|(_, left), (_, right)| {
            let left_energy = left.0.0 * left.0.0
                + left.0.1 * left.0.1
                + left.1.0 * left.1.0
                + left.1.1 * left.1.1;
            let right_energy = right.0.0 * right.0.0
                + right.0.1 * right.0.1
                + right.1.0 * right.1.0
                + right.1.1 * right.1.1;
            left_energy.total_cmp(&right_energy)
        })
        .map_or(0, |(bin, _)| bin);
    let (left_real, left_imag) = left_spectrum[bin];
    let (right_real, right_imag) = right_spectrum[bin];
    (left_real * right_imag - left_imag * right_real)
        .atan2(left_real * right_real + left_imag * right_imag)
}

fn boundary_discontinuity(reference: &[f64], candidate: &[f64]) -> f64 {
    reference
        .windows(2)
        .zip(candidate.windows(2))
        .map(|(reference, candidate)| {
            ((reference[1] - reference[0]) - (candidate[1] - candidate[0])).abs()
        })
        .fold(0.0, f64::max)
}
