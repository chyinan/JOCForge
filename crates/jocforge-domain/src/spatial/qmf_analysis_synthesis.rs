// pattern: Functional Core

use super::qmf_fft64::{QmfFftError, QmfFftInput, execute_qmf_fft64};
use super::qmf_mathematical_fft_provider::QmfMathematicalFftProvider;
use super::qmf_receiver_functional_provider::QmfReceiverFunctionalProvider;
use crate::numeric::{FiniteF32, R32Error, r32_add, r32_mul, r32_sub};

const ANALYSIS_SAMPLE_COUNT: usize = 64;
const SPECTRUM_WORD_COUNT: usize = 128;
const ANALYSIS_HISTORY_ROWS: usize = 9;
const RECEIVER_SLOT_COUNT: usize = 24;
const MODE1_DELAY_SLOT_COUNT: usize = 10;
const BAND0_FIR_HISTORY_COMPLEX_COUNT: usize = 20;
const SYNTHESIS_HISTORY_WORD_COUNT: usize = 576;
const SIGN_BIT: u32 = 0x8000_0000;
pub(crate) const QMF_ANALYSIS_SAMPLES_PER_SLOT: usize = ANALYSIS_SAMPLE_COUNT;
pub(crate) const QMF_ANALYSIS_SPECTRUM_WORDS: usize = SPECTRUM_WORD_COUNT;
pub(crate) const QMF_ANALYSIS_SLOTS_PER_SOURCE: usize = 24;
pub(crate) const QMF_ANALYSIS_SAMPLES_PER_SOURCE: usize =
    QMF_ANALYSIS_SAMPLES_PER_SLOT * QMF_ANALYSIS_SLOTS_PER_SOURCE;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// `InvalidInput` maps to F04_INPUT_ERROR; `NumericFailure` maps to F04_NUMERIC_FAILURE.
pub enum QmfAnalysisSynthesisError {
    InvalidInput,
    NumericFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// One lane's nine analysis-history rows, stored as F32 U32 words in `H0..H8` order.
pub struct QmfAnalysisHistory {
    rows: [[u32; ANALYSIS_SAMPLE_COUNT]; ANALYSIS_HISTORY_ROWS],
}

impl QmfAnalysisHistory {
    pub const fn cold() -> Self {
        Self {
            rows: [[0; ANALYSIS_SAMPLE_COUNT]; ANALYSIS_HISTORY_ROWS],
        }
    }

    pub fn from_words(
        rows: [[u32; ANALYSIS_SAMPLE_COUNT]; ANALYSIS_HISTORY_ROWS],
    ) -> Result<Self, QmfAnalysisSynthesisError> {
        if rows
            .iter()
            .flatten()
            .any(|word| !f32::from_bits(*word).is_finite())
        {
            return Err(QmfAnalysisSynthesisError::InvalidInput);
        }
        Ok(Self { rows })
    }

    pub const fn rows(&self) -> &[[u32; ANALYSIS_SAMPLE_COUNT]; ANALYSIS_HISTORY_ROWS] {
        &self.rows
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct QmfAnalysisSlotResult {
    spectrum_words: [u32; SPECTRUM_WORD_COUNT],
    history_after: QmfAnalysisHistory,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct QmfAnalysisRowResult {
    spectrum_by_slot: Box<[[u32; SPECTRUM_WORD_COUNT]; QMF_ANALYSIS_SLOTS_PER_SOURCE]>,
    history_after: QmfAnalysisHistory,
}

impl QmfAnalysisRowResult {
    pub(crate) const fn spectrum_by_slot(
        &self,
    ) -> &[[u32; SPECTRUM_WORD_COUNT]; QMF_ANALYSIS_SLOTS_PER_SOURCE] {
        &self.spectrum_by_slot
    }

    pub(crate) const fn history_after(&self) -> &QmfAnalysisHistory {
        &self.history_after
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Box<[[u32; SPECTRUM_WORD_COUNT]; QMF_ANALYSIS_SLOTS_PER_SOURCE]>,
        QmfAnalysisHistory,
    ) {
        (self.spectrum_by_slot, self.history_after)
    }
}

impl QmfAnalysisSlotResult {
    pub const fn spectrum_words(&self) -> &[u32; SPECTRUM_WORD_COUNT] {
        &self.spectrum_words
    }

    pub const fn history_after(&self) -> &QmfAnalysisHistory {
        &self.history_after
    }
}

#[derive(Debug, Eq, PartialEq)]
/// The ten delay slots, twenty complex band0 history values, and 576 synthesis words for one mode=1 lane.
pub struct QmfReceiverMode1Histories {
    delay: [[u32; SPECTRUM_WORD_COUNT]; MODE1_DELAY_SLOT_COUNT],
    band0_fir: [[u32; 2]; BAND0_FIR_HISTORY_COMPLEX_COUNT],
    synthesis: [u32; SYNTHESIS_HISTORY_WORD_COUNT],
}

impl QmfReceiverMode1Histories {
    pub const fn cold() -> Self {
        Self {
            delay: [[0; SPECTRUM_WORD_COUNT]; MODE1_DELAY_SLOT_COUNT],
            band0_fir: [[0; 2]; BAND0_FIR_HISTORY_COMPLEX_COUNT],
            synthesis: [0; SYNTHESIS_HISTORY_WORD_COUNT],
        }
    }

    pub fn from_words(
        delay_history: [[u32; SPECTRUM_WORD_COUNT]; MODE1_DELAY_SLOT_COUNT],
        band0_fir_history: [[u32; 2]; BAND0_FIR_HISTORY_COMPLEX_COUNT],
        synthesis_history: [u32; SYNTHESIS_HISTORY_WORD_COUNT],
    ) -> Result<Self, QmfAnalysisSynthesisError> {
        if delay_history
            .iter()
            .flatten()
            .chain(band0_fir_history.iter().flatten())
            .chain(synthesis_history.iter())
            .any(|word| !f32::from_bits(*word).is_finite())
        {
            return Err(QmfAnalysisSynthesisError::InvalidInput);
        }
        Ok(Self {
            delay: delay_history,
            band0_fir: band0_fir_history,
            synthesis: synthesis_history,
        })
    }

    pub const fn delay_history(&self) -> &[[u32; SPECTRUM_WORD_COUNT]; MODE1_DELAY_SLOT_COUNT] {
        &self.delay
    }

    pub const fn band0_fir_history(&self) -> &[[u32; 2]; BAND0_FIR_HISTORY_COMPLEX_COUNT] {
        &self.band0_fir
    }

    pub const fn synthesis_history(&self) -> &[u32; SYNTHESIS_HISTORY_WORD_COUNT] {
        &self.synthesis
    }
}

#[derive(Debug, PartialEq)]
pub struct QmfReceiverMode1CallbackResult {
    pcm_slots: [[f32; ANALYSIS_SAMPLE_COUNT]; RECEIVER_SLOT_COUNT],
    history_after: QmfReceiverMode1Histories,
}

impl QmfReceiverMode1CallbackResult {
    pub const fn pcm_slots(&self) -> &[[f32; ANALYSIS_SAMPLE_COUNT]; RECEIVER_SLOT_COUNT] {
        &self.pcm_slots
    }

    pub const fn history_after(&self) -> &QmfReceiverMode1Histories {
        &self.history_after
    }
}

struct AnalysisTrace {
    #[cfg(test)]
    fold128: [u32; SPECTRUM_WORD_COUNT],
    #[cfg(test)]
    rotated128: [u32; SPECTRUM_WORD_COUNT],
    #[cfg(test)]
    fft_provider_input128: [u32; SPECTRUM_WORD_COUNT],
    #[cfg(test)]
    bitrev_scaled_fft_input128: [u32; SPECTRUM_WORD_COUNT],
    #[cfg(test)]
    fft_output128: [u32; SPECTRUM_WORD_COUNT],
    final_reorder_output128: [u32; SPECTRUM_WORD_COUNT],
    history_after_9x64: [[u32; ANALYSIS_SAMPLE_COUNT]; ANALYSIS_HISTORY_ROWS],
}

struct SynthesisTrace {
    #[cfg(test)]
    synthesis_preformat128: [u32; SPECTRUM_WORD_COUNT],
    #[cfg(test)]
    inverse_fft_input128: [u32; SPECTRUM_WORD_COUNT],
    #[cfg(test)]
    inverse_fft_output128: [u32; SPECTRUM_WORD_COUNT],
    #[cfg(test)]
    postmodulation128: [u32; SPECTRUM_WORD_COUNT],
    polyphase_output64: [u32; ANALYSIS_SAMPLE_COUNT],
    next_synthesis_history576: [u32; SYNTHESIS_HISTORY_WORD_COUNT],
}

struct Mode1PhaseTrace {
    phase_output24x128: [[u32; SPECTRUM_WORD_COUNT]; RECEIVER_SLOT_COUNT],
    next_delay_history10x128: [[u32; SPECTRUM_WORD_COUNT]; MODE1_DELAY_SLOT_COUNT],
    next_fir_history20x2: [[u32; 2]; BAND0_FIR_HISTORY_COMPLEX_COUNT],
}

/// Analyze one lane's 64-sample slot and return, but do not commit, its next history.
/// The caller owns lane routing and decides whether to retain the returned history.
pub fn analyze_qmf_slot(
    current_samples: &[f32; ANALYSIS_SAMPLE_COUNT],
    history_before: &QmfAnalysisHistory,
    functional_provider: &QmfReceiverFunctionalProvider,
    fft_provider: &QmfMathematicalFftProvider,
) -> Result<QmfAnalysisSlotResult, QmfAnalysisSynthesisError> {
    let trace = analyze_qmf_slot_trace(
        current_samples,
        history_before,
        functional_provider,
        fft_provider,
    )?;
    Ok(QmfAnalysisSlotResult {
        spectrum_words: trace.final_reorder_output128,
        history_after: QmfAnalysisHistory {
            rows: trace.history_after_9x64,
        },
    })
}

/// Analyzes one complete source row as consecutive 64-sample QMF slots.
///
/// Each slot uses the preceding slot's history, and the returned history belongs to this row
/// only. The caller owns role selection, cross-source state, and whether to retain the proposal.
pub(crate) fn analyze_qmf_row(
    source_samples: &[f32],
    history_before: &QmfAnalysisHistory,
    functional_provider: &QmfReceiverFunctionalProvider,
    fft_provider: &QmfMathematicalFftProvider,
) -> Result<QmfAnalysisRowResult, QmfAnalysisSynthesisError> {
    if source_samples.len() != QMF_ANALYSIS_SAMPLES_PER_SOURCE {
        return Err(QmfAnalysisSynthesisError::InvalidInput);
    }
    let mut history = QmfAnalysisHistory {
        rows: *history_before.rows(),
    };
    let mut spectrum_by_slot = [[0_u32; SPECTRUM_WORD_COUNT]; QMF_ANALYSIS_SLOTS_PER_SOURCE];
    for slot in 0..QMF_ANALYSIS_SLOTS_PER_SOURCE {
        let slot_samples = std::array::from_fn(|sample| {
            source_samples[slot * QMF_ANALYSIS_SAMPLES_PER_SLOT + sample]
        });
        let result = analyze_qmf_slot(&slot_samples, &history, functional_provider, fft_provider)?;
        spectrum_by_slot[slot] = *result.spectrum_words();
        history = result.history_after;
    }
    Ok(QmfAnalysisRowResult {
        spectrum_by_slot: Box::new(spectrum_by_slot),
        history_after: history,
    })
}

fn analyze_qmf_slot_trace(
    current_samples: &[f32; ANALYSIS_SAMPLE_COUNT],
    history_before: &QmfAnalysisHistory,
    functional_provider: &QmfReceiverFunctionalProvider,
    fft_provider: &QmfMathematicalFftProvider,
) -> Result<AnalysisTrace, QmfAnalysisSynthesisError> {
    if current_samples.iter().any(|sample| !sample.is_finite()) {
        return Err(QmfAnalysisSynthesisError::InvalidInput);
    }
    let mut fold128 = [0_u32; SPECTRUM_WORD_COUNT];
    let window = functional_provider.qwin_synthesis_words();
    for k in 0..ANALYSIS_SAMPLE_COUNT {
        let h = |row: usize| f32::from_bits(history_before.rows[row][k]);
        let w = |offset: usize| f32::from_bits(window[639 - offset - k]);
        let mut even = h(6) * w(128) + h(8) * w(0);
        even += h(4) * w(256);
        even += h(2) * w(384);
        even += h(0) * w(512);
        let mut odd = h(5) * w(192) + h(7) * w(64);
        odd += h(3) * w(320);
        odd += h(1) * w(448);
        odd += current_samples[k] * w(576);
        fold128[126 - 2 * k] = odd.to_bits();
        fold128[127 - 2 * k] = even.to_bits();
    }
    let rotation = fft_provider.rotation_words();
    let mut rotated128 = [0_u32; SPECTRUM_WORD_COUNT];
    for ((input, output), coefficients) in fold128
        .chunks_exact(2)
        .zip(rotated128.chunks_exact_mut(2))
        .zip(rotation.chunks_exact(2))
    {
        let (r, i) = (f32::from_bits(input[0]), f32::from_bits(input[1]));
        let (a, b) = (
            f32::from_bits(coefficients[0]),
            f32::from_bits(coefficients[1]),
        );
        output[0] = (i * a - r * b).to_bits();
        output[1] = (i * b + r * a).to_bits();
    }
    #[cfg(test)]
    let fft_provider_input128 = rotated128;
    let scaled_fft_input128 = rotated128.map(|word| f32::from_bits(word) / 64.0);
    #[cfg(test)]
    let mut bitrev_scaled_fft_input128 = [0_u32; SPECTRUM_WORD_COUNT];
    #[cfg(test)]
    for destination in 0..ANALYSIS_SAMPLE_COUNT {
        let source = reverse_six_bits(destination);
        bitrev_scaled_fft_input128[destination * 2] = scaled_fft_input128[source * 2].to_bits();
        bitrev_scaled_fft_input128[destination * 2 + 1] =
            scaled_fft_input128[source * 2 + 1].to_bits();
    }

    // Source samples, history and the immutable provider are finite. A nonfinite FFT input here can only be arithmetic overflow.
    let fft_input = QmfFftInput::new(&scaled_fft_input128)
        .map_err(|_| QmfAnalysisSynthesisError::NumericFailure)?;
    let fft_output = execute_qmf_fft64(&fft_input, fft_provider).map_err(map_fft_error)?;
    let fft_output128 = fft_output.interleaved().map(f32::to_bits);
    let final_reorder_output128 = analysis_reorder(&fft_output128);

    let mut history_after_9x64 = [[0_u32; ANALYSIS_SAMPLE_COUNT]; ANALYSIS_HISTORY_ROWS];
    for sample in 0..ANALYSIS_SAMPLE_COUNT {
        history_after_9x64[0][sample] = current_samples[sample].to_bits();
    }
    history_after_9x64[1..].copy_from_slice(&history_before.rows[..ANALYSIS_HISTORY_ROWS - 1]);

    Ok(AnalysisTrace {
        #[cfg(test)]
        fold128,
        #[cfg(test)]
        rotated128,
        #[cfg(test)]
        fft_provider_input128,
        #[cfg(test)]
        bitrev_scaled_fft_input128,
        #[cfg(test)]
        fft_output128,
        final_reorder_output128,
        history_after_9x64,
    })
}

/// Process one surround lane's fixed 24-slot receiver mode=1 callback without committing state.
/// `analysis_slots` must be the selected lane's already-computed QMF analysis spectra.
/// This function does not select a lane, accept a mode, or mutate the before histories.
pub fn process_qmf_receiver_mode1_callback(
    analysis_slots: &[[u32; SPECTRUM_WORD_COUNT]; RECEIVER_SLOT_COUNT],
    history_before: &QmfReceiverMode1Histories,
    functional_provider: &QmfReceiverFunctionalProvider,
    fft_provider: &QmfMathematicalFftProvider,
) -> Result<QmfReceiverMode1CallbackResult, QmfAnalysisSynthesisError> {
    let phase = qmf_mode1_delay_fir_trace(analysis_slots, history_before, functional_provider)?;
    let mut synthesis_history = history_before.synthesis;
    let mut pcm_slots = [[0.0_f32; ANALYSIS_SAMPLE_COUNT]; RECEIVER_SLOT_COUNT];
    for (slot, pcm_slot) in pcm_slots.iter_mut().enumerate() {
        let trace = synthesize_qmf_slot_trace(
            &phase.phase_output24x128[slot],
            &synthesis_history,
            functional_provider,
            fft_provider,
        )?;
        for (sample, output_sample) in pcm_slot.iter_mut().enumerate() {
            *output_sample = f32::from_bits(trace.polyphase_output64[sample]);
        }
        synthesis_history = trace.next_synthesis_history576;
    }

    Ok(QmfReceiverMode1CallbackResult {
        pcm_slots,
        history_after: QmfReceiverMode1Histories {
            delay: phase.next_delay_history10x128,
            band0_fir: phase.next_fir_history20x2,
            synthesis: synthesis_history,
        },
    })
}

fn qmf_mode1_delay_fir_trace(
    analysis_slots: &[[u32; SPECTRUM_WORD_COUNT]; RECEIVER_SLOT_COUNT],
    history_before: &QmfReceiverMode1Histories,
    functional_provider: &QmfReceiverFunctionalProvider,
) -> Result<Mode1PhaseTrace, QmfAnalysisSynthesisError> {
    if analysis_slots
        .iter()
        .flatten()
        .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(QmfAnalysisSynthesisError::InvalidInput);
    }

    let mut phase_output24x128 = [[0_u32; SPECTRUM_WORD_COUNT]; RECEIVER_SLOT_COUNT];
    for slot in 0..RECEIVER_SLOT_COUNT {
        let source = if slot < MODE1_DELAY_SLOT_COUNT {
            &history_before.delay[slot]
        } else {
            &analysis_slots[slot - MODE1_DELAY_SLOT_COUNT]
        };
        for band in 0..ANALYSIS_SAMPLE_COUNT {
            phase_output24x128[slot][band * 2] = source[band * 2 + 1] ^ SIGN_BIT;
            phase_output24x128[slot][band * 2 + 1] = source[band * 2];
        }
    }

    let mut fir_input = [[0_u32; 2]; BAND0_FIR_HISTORY_COMPLEX_COUNT + RECEIVER_SLOT_COUNT];
    fir_input[..BAND0_FIR_HISTORY_COMPLEX_COUNT].copy_from_slice(&history_before.band0_fir);
    for slot in 0..RECEIVER_SLOT_COUNT {
        fir_input[BAND0_FIR_HISTORY_COMPLEX_COUNT + slot] =
            [analysis_slots[slot][0], analysis_slots[slot][1]];
    }

    let two = FiniteF32::new(2.0).expect("two is a finite F32 constant");
    let real_taps = functional_provider.band0_real_mode1_words();
    let imag_taps = functional_provider.band0_imag_mode1_words();
    for slot in 0..RECEIVER_SLOT_COUNT {
        let mut accumulated_real = positive_zero();
        let mut accumulated_imag = positive_zero();
        for tap in 0..21 {
            let input_real = finite_word(fir_input[slot + tap][0])?;
            let input_imag = finite_word(fir_input[slot + tap][1])?;
            let real_tap = finite_word(real_taps[tap])?;
            let imag_tap = finite_word(imag_taps[tap])?;

            let real_product = r32_mul(real_tap, input_real)?;
            let real_sum = r32_add(accumulated_real, real_product)?;
            let imag_product = r32_mul(imag_tap, input_imag)?;
            accumulated_real = r32_sub(real_sum, imag_product)?;

            let imag_from_real = r32_mul(real_tap, input_imag)?;
            let imag_sum = r32_add(accumulated_imag, imag_from_real)?;
            let real_from_imag = r32_mul(imag_tap, input_real)?;
            accumulated_imag = r32_add(imag_sum, real_from_imag)?;
        }

        phase_output24x128[slot][0] = r32_mul(two, accumulated_real)?.to_bits();
        phase_output24x128[slot][1] = r32_mul(two, accumulated_imag)?.to_bits();
    }

    let mut next_delay_history10x128 = [[0_u32; SPECTRUM_WORD_COUNT]; MODE1_DELAY_SLOT_COUNT];
    next_delay_history10x128
        .copy_from_slice(&analysis_slots[RECEIVER_SLOT_COUNT - MODE1_DELAY_SLOT_COUNT..]);
    let mut next_fir_history20x2 = [[0_u32; 2]; BAND0_FIR_HISTORY_COMPLEX_COUNT];
    next_fir_history20x2.copy_from_slice(&fir_input[RECEIVER_SLOT_COUNT..]);

    Ok(Mode1PhaseTrace {
        phase_output24x128,
        next_delay_history10x128,
        next_fir_history20x2,
    })
}

fn synthesize_qmf_slot_trace(
    bands128: &[u32; SPECTRUM_WORD_COUNT],
    history_before576: &[u32; SYNTHESIS_HISTORY_WORD_COUNT],
    functional_provider: &QmfReceiverFunctionalProvider,
    fft_provider: &QmfMathematicalFftProvider,
) -> Result<SynthesisTrace, QmfAnalysisSynthesisError> {
    if bands128
        .iter()
        .chain(history_before576.iter())
        .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(QmfAnalysisSynthesisError::InvalidInput);
    }

    let synthesis_preformat128 = synthesis_preformat(bands128);
    let inverse_fft_input128 = synthesis_preformat128;
    let inverse_fft_input = inverse_fft_input128.map(f32::from_bits);
    let fft_input = QmfFftInput::new(&inverse_fft_input).map_err(map_fft_error)?;
    let fft_output = execute_qmf_fft64(&fft_input, fft_provider).map_err(map_fft_error)?;
    let inverse_fft_output128 = fft_output.interleaved().map(f32::to_bits);

    let rotation = fft_provider.rotation_words();
    let two = FiniteF32::new(2.0).expect("two is a finite F32 constant");
    let mut postmodulation128 = [0_u32; SPECTRUM_WORD_COUNT];
    for complex_index in 0..ANALYSIS_SAMPLE_COUNT {
        let r = finite_word(inverse_fft_output128[complex_index * 2])?;
        let i = finite_word(inverse_fft_output128[complex_index * 2 + 1])?;
        let a = finite_word(rotation[complex_index * 2])?;
        let b = finite_word(rotation[complex_index * 2 + 1])?;

        let real_from_r = r32_mul(r, b)?;
        let real_from_i = r32_mul(i, a)?;
        let real_sum = r32_add(real_from_r, real_from_i)?;
        let real = r32_mul(real_sum, two)?;

        let imag_from_i = r32_mul(i, b)?;
        let imag_from_r = r32_mul(r, a)?;
        let imag_difference = r32_sub(imag_from_i, imag_from_r)?;
        let imaginary = r32_mul(imag_difference, two)?;

        postmodulation128[complex_index * 2] = real.to_bits();
        postmodulation128[complex_index * 2 + 1] = imaginary.to_bits();
    }

    let prototype = functional_provider.qwin_synthesis_words();
    let mut polyphase_output64 = [0_u32; ANALYSIS_SAMPLE_COUNT];
    let mut next_synthesis_history576 = [0_u32; SYNTHESIS_HISTORY_WORD_COUNT];
    for k in 0..ANALYSIS_SAMPLE_COUNT {
        let group = k / 4;
        let lane = k % 4;
        let history_index = 36 * group + lane;
        let real = finite_word(postmodulation128[k * 2])?;
        let imaginary = finite_word(postmodulation128[k * 2 + 1])?;
        let old_v = finite_word(history_before576[history_index])?;
        let v = r32_add(r32_mul(finite_word(prototype[k])?, real)?, old_v)?;
        polyphase_output64[k] = r32_add(v, v)?.to_bits();

        for j in 0..8 {
            let coefficient = if (j + 1) % 2 == 1 { imaginary } else { real };
            let tap_index = 64 * (j + 1) + k;
            let old_history_index = history_index + 4 * (j + 1);
            next_synthesis_history576[history_index + 4 * j] = r32_add(
                r32_mul(finite_word(prototype[tap_index])?, coefficient)?,
                finite_word(history_before576[old_history_index])?,
            )?
            .to_bits();
        }
        next_synthesis_history576[history_index + 32] =
            r32_mul(finite_word(prototype[576 + k])?, imaginary)?.to_bits();
    }

    Ok(SynthesisTrace {
        #[cfg(test)]
        synthesis_preformat128,
        #[cfg(test)]
        inverse_fft_input128,
        #[cfg(test)]
        inverse_fft_output128,
        #[cfg(test)]
        postmodulation128,
        polyphase_output64,
        next_synthesis_history576,
    })
}

fn analysis_reorder(input128: &[u32; SPECTRUM_WORD_COUNT]) -> [u32; SPECTRUM_WORD_COUNT] {
    let mut output = [0_u32; SPECTRUM_WORD_COUNT];
    for p in 0..32 {
        output[4 * p] = input128[2 * p];
        output[4 * p + 1] = input128[2 * p + 1] ^ SIGN_BIT;
        output[4 * p + 2] = input128[126 - 2 * p];
        output[4 * p + 3] = input128[127 - 2 * p];
    }
    output
}

fn synthesis_preformat(input128: &[u32; SPECTRUM_WORD_COUNT]) -> [u32; SPECTRUM_WORD_COUNT] {
    let mut output = [0_u32; SPECTRUM_WORD_COUNT];
    for p in 0..32 {
        output[2 * p] = input128[4 * p];
        output[2 * p + 1] = input128[4 * p + 1] ^ SIGN_BIT;
        output[126 - 2 * p] = input128[4 * p + 2];
        output[127 - 2 * p] = input128[4 * p + 3];
    }
    output
}

fn finite_input(value: f32) -> Result<FiniteF32, QmfAnalysisSynthesisError> {
    FiniteF32::new(value).map_err(|_| QmfAnalysisSynthesisError::InvalidInput)
}

fn finite_word(word: u32) -> Result<FiniteF32, QmfAnalysisSynthesisError> {
    finite_input(f32::from_bits(word))
}

fn positive_zero() -> FiniteF32 {
    FiniteF32::new(0.0).expect("positive zero is finite")
}

#[cfg(test)]
fn reverse_six_bits(value: usize) -> usize {
    let mut remaining = value;
    let mut reversed = 0;
    for _ in 0..6 {
        reversed = (reversed << 1) | (remaining & 1);
        remaining >>= 1;
    }
    reversed
}

fn map_fft_error(error: QmfFftError) -> QmfAnalysisSynthesisError {
    match error {
        QmfFftError::WrongScalarCount { .. } | QmfFftError::NonFiniteInput { .. } => {
            QmfAnalysisSynthesisError::InvalidInput
        }
        QmfFftError::NumericFailure => QmfAnalysisSynthesisError::NumericFailure,
    }
}

impl From<R32Error> for QmfAnalysisSynthesisError {
    fn from(_: R32Error) -> Self {
        Self::NumericFailure
    }
}

#[cfg(test)]
mod tests;
