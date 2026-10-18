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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// `InvalidInput` maps to F04_INPUT_ERROR; `NumericFailure` maps to F04_NUMERIC_FAILURE.
pub enum QmfAnalysisSynthesisError {
    InvalidInput,
    NumericFailure,
}

#[derive(Debug, Eq, PartialEq)]
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

fn analyze_qmf_slot_trace(
    current_samples: &[f32; ANALYSIS_SAMPLE_COUNT],
    history_before: &QmfAnalysisHistory,
    functional_provider: &QmfReceiverFunctionalProvider,
    fft_provider: &QmfMathematicalFftProvider,
) -> Result<AnalysisTrace, QmfAnalysisSynthesisError> {
    let mut samples = [positive_zero(); ANALYSIS_SAMPLE_COUNT];
    for (index, sample) in current_samples.iter().copied().enumerate() {
        samples[index] = finite_input(sample)?;
    }

    let mut fold128 = [0_u32; SPECTRUM_WORD_COUNT];
    let qwin_synthesis = functional_provider.qwin_synthesis_words();
    for k in 0..ANALYSIS_SAMPLE_COUNT {
        let h6 = finite_word(history_before.rows[6][k])?;
        let h8 = finite_word(history_before.rows[8][k])?;
        let h4 = finite_word(history_before.rows[4][k])?;
        let h2 = finite_word(history_before.rows[2][k])?;
        let h0 = finite_word(history_before.rows[0][k])?;
        let h5 = finite_word(history_before.rows[5][k])?;
        let h7 = finite_word(history_before.rows[7][k])?;
        let h3 = finite_word(history_before.rows[3][k])?;
        let h1 = finite_word(history_before.rows[1][k])?;
        let x = samples[k];

        let p128 = analysis_prototype_word(qwin_synthesis, 128 + k)?;
        let p0 = analysis_prototype_word(qwin_synthesis, k)?;
        let mut even = r32_add(r32_mul(h6, p128)?, r32_mul(h8, p0)?)?;
        even = r32_add(
            even,
            r32_mul(h4, analysis_prototype_word(qwin_synthesis, 256 + k)?)?,
        )?;
        even = r32_add(
            even,
            r32_mul(h2, analysis_prototype_word(qwin_synthesis, 384 + k)?)?,
        )?;
        even = r32_add(
            even,
            r32_mul(h0, analysis_prototype_word(qwin_synthesis, 512 + k)?)?,
        )?;

        let mut odd = r32_add(
            r32_mul(h5, analysis_prototype_word(qwin_synthesis, 192 + k)?)?,
            r32_mul(h7, analysis_prototype_word(qwin_synthesis, 64 + k)?)?,
        )?;
        odd = r32_add(
            odd,
            r32_mul(h3, analysis_prototype_word(qwin_synthesis, 320 + k)?)?,
        )?;
        odd = r32_add(
            odd,
            r32_mul(h1, analysis_prototype_word(qwin_synthesis, 448 + k)?)?,
        )?;
        odd = r32_add(
            odd,
            r32_mul(x, analysis_prototype_word(qwin_synthesis, 576 + k)?)?,
        )?;

        fold128[126 - 2 * k] = odd.to_bits();
        fold128[127 - 2 * k] = even.to_bits();
    }

    let rotation = fft_provider.rotation_words();
    let mut rotated128 = [0_u32; SPECTRUM_WORD_COUNT];
    for complex_index in 0..ANALYSIS_SAMPLE_COUNT {
        let r = finite_word(fold128[complex_index * 2])?;
        let i = finite_word(fold128[complex_index * 2 + 1])?;
        let a = finite_word(rotation[complex_index * 2])?;
        let b = finite_word(rotation[complex_index * 2 + 1])?;

        let real = r32_sub(r32_mul(i, a)?, r32_mul(r, b)?)?;
        let imaginary = r32_add(r32_mul(i, b)?, r32_mul(r, a)?)?;
        rotated128[complex_index * 2] = real.to_bits();
        rotated128[complex_index * 2 + 1] = imaginary.to_bits();
    }

    #[cfg(test)]
    let mut fft_provider_input128 = [0_u32; SPECTRUM_WORD_COUNT];
    let mut scaled_fft_input128 = [0.0_f32; SPECTRUM_WORD_COUNT];
    for index in 0..SPECTRUM_WORD_COUNT {
        let rotated = finite_word(rotated128[index])?;
        let scaled = FiniteF32::new(rotated.value() / 64.0)
            .map_err(|_| QmfAnalysisSynthesisError::NumericFailure)?;
        #[cfg(test)]
        {
            fft_provider_input128[index] = rotated.to_bits();
        }
        scaled_fft_input128[index] = scaled.value();
    }

    #[cfg(test)]
    let mut bitrev_scaled_fft_input128 = [0_u32; SPECTRUM_WORD_COUNT];
    #[cfg(test)]
    for destination in 0..ANALYSIS_SAMPLE_COUNT {
        let source = reverse_six_bits(destination);
        bitrev_scaled_fft_input128[destination * 2] = scaled_fft_input128[source * 2].to_bits();
        bitrev_scaled_fft_input128[destination * 2 + 1] =
            scaled_fft_input128[source * 2 + 1].to_bits();
    }

    let fft_input = QmfFftInput::new(&scaled_fft_input128).map_err(map_fft_error)?;
    let fft_output = execute_qmf_fft64(&fft_input, fft_provider).map_err(map_fft_error)?;
    let fft_output128 = fft_output.interleaved().map(f32::to_bits);
    let final_reorder_output128 = analysis_reorder(&fft_output128);

    let mut history_after_9x64 = [[0_u32; ANALYSIS_SAMPLE_COUNT]; ANALYSIS_HISTORY_ROWS];
    for sample in 0..ANALYSIS_SAMPLE_COUNT {
        history_after_9x64[0][sample] = samples[sample].to_bits();
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

fn analysis_prototype_word(
    qwin_synthesis: &[u32; 640],
    index: usize,
) -> Result<FiniteF32, QmfAnalysisSynthesisError> {
    finite_word(qwin_synthesis[639 - index])
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
