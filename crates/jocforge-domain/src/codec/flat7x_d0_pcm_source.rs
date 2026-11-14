// pattern: Functional Core
//! Builds a six-block D0 spectrum and D15 candidate from one source's four PCM rows.

use super::{
    bap::{FLAT7X_D0_BAP_BLOCK_COUNT, FLAT7X_D0_BAP_CHANNEL_COUNT, Flat7xD0BapParameters},
    flat7x_d0_audio_frame::{
        Flat7xD0AudioFrameCandidate, Flat7xD0AudioFrameError, Flat7xD0AudioFrameInput,
        produce_flat7x_d0_audio_frame,
    },
    flat7x_d0_exponents::{
        Flat7xD0ExponentError, Flat7xD0ExponentRow, prepare_flat7x_d0_d15_exponent_row,
    },
    flat7x_d0_spectrum::{
        D0_PCM_SAMPLES_PER_BLOCK, Flat7xD0PcmBlockInput, Flat7xD0PcmWindow, Flat7xD0SpectrumBlock,
        Flat7xD0SpectrumError, transform_flat7x_d0_pcm_block,
    },
    frame_builder::Flat7xD0BsiStructure,
};
use crate::profile::Flat7xI0D0TargetPlan;
use crate::timeline::{
    AdmSampleIndex, CodecSourceIndex, FrontendCallbackIndex, source_for_callback,
};
use crate::transform::math_generated_parameters::MathGeneratedTransformParameters;

const SOURCE_SAMPLES: usize = FLAT7X_D0_BAP_BLOCK_COUNT * D0_PCM_SAMPLES_PER_BLOCK;
const SOURCE_SAMPLES_U64: u64 = 1_536;

/// Reader-owned sample extent bound to one callback and codec-source index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0PcmSourceExtent {
    source_index: CodecSourceIndex,
    callback_index: FrontendCallbackIndex,
    first_sample: AdmSampleIndex,
    valid_samples: u16,
    padded_samples: u16,
    origin: Flat7xD0PcmSourceExtentOrigin,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0PcmSourceExtentOrigin {
    ReaderCallback,
    CachedDrainRepeatPreviousCallbackV1,
}

impl Flat7xD0PcmSourceExtent {
    pub fn new(
        source_index: CodecSourceIndex,
        callback_index: FrontendCallbackIndex,
        first_sample: AdmSampleIndex,
        valid_samples: u16,
        padded_samples: u16,
    ) -> Result<Self, Flat7xD0PcmSourceExtentError> {
        if source_for_callback(callback_index) != Some(source_index) {
            return Err(Flat7xD0PcmSourceExtentError::CallbackSourceMismatch);
        }
        let expected_first_sample = u64::from(callback_index.value()) * SOURCE_SAMPLES_U64;
        if first_sample.value() != expected_first_sample {
            return Err(Flat7xD0PcmSourceExtentError::FirstSampleMismatch);
        }
        if valid_samples == 0 {
            return Err(Flat7xD0PcmSourceExtentError::EmptySource);
        }
        if usize::from(valid_samples) + usize::from(padded_samples) != SOURCE_SAMPLES {
            return Err(Flat7xD0PcmSourceExtentError::SampleCountMismatch);
        }
        Ok(Self {
            source_index,
            callback_index,
            first_sample,
            valid_samples,
            padded_samples,
            origin: Flat7xD0PcmSourceExtentOrigin::ReaderCallback,
        })
    }

    /// Issues the current local cached-drain model: reuse the last cached PCM window for this
    /// logical source clock while retaining its guessed callback sample origin explicitly.
    pub fn repeat_previous_callback_for_cached_drain(
        source_index: CodecSourceIndex,
        callback_index: FrontendCallbackIndex,
    ) -> Result<Self, Flat7xD0PcmSourceExtentError> {
        let first_sample =
            AdmSampleIndex::new(u64::from(callback_index.value()) * SOURCE_SAMPLES_U64);
        let mut extent = Self::new(
            source_index,
            callback_index,
            first_sample,
            SOURCE_SAMPLES as u16,
            0,
        )?;
        extent.origin = Flat7xD0PcmSourceExtentOrigin::CachedDrainRepeatPreviousCallbackV1;
        Ok(extent)
    }

    pub const fn source_index(self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn callback_index(self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn first_sample(self) -> AdmSampleIndex {
        self.first_sample
    }

    pub const fn valid_samples(self) -> u16 {
        self.valid_samples
    }

    pub const fn padded_samples(self) -> u16 {
        self.padded_samples
    }

    pub const fn origin(self) -> Flat7xD0PcmSourceExtentOrigin {
        self.origin
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0PcmSourceExtentError {
    CallbackSourceMismatch,
    FirstSampleMismatch,
    EmptySource,
    SampleCountMismatch,
}

/// D0 overlap samples in coded-channel order `[Ls, Rs, Lrs, Rrs]`.
///
/// The parent owner retains the committed value and installs a staged replacement only after its
/// source transaction is accepted. Samples are stored as their original f32 bit words.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0OverlapHistory {
    samples_by_channel: [[u32; D0_PCM_SAMPLES_PER_BLOCK]; FLAT7X_D0_BAP_CHANNEL_COUNT],
}

impl Flat7xD0OverlapHistory {
    /// Creates the selected policy's zero-history cold value.
    pub const fn cold() -> Self {
        Self {
            samples_by_channel: [[0; D0_PCM_SAMPLES_PER_BLOCK]; FLAT7X_D0_BAP_CHANNEL_COUNT],
        }
    }

    pub const fn samples_by_channel(
        &self,
    ) -> &[[u32; D0_PCM_SAMPLES_PER_BLOCK]; FLAT7X_D0_BAP_CHANNEL_COUNT] {
        &self.samples_by_channel
    }
}

impl Default for Flat7xD0OverlapHistory {
    fn default() -> Self {
        Self::cold()
    }
}

/// Candidate values for one source's six D0 blocks and following overlap history.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xD0PcmBlockSeriesCandidate {
    spectra: Box<[Flat7xD0SpectrumBlock; FLAT7X_D0_BAP_BLOCK_COUNT]>,
    exponent_rows:
        Box<[[Flat7xD0ExponentRow; FLAT7X_D0_BAP_CHANNEL_COUNT]; FLAT7X_D0_BAP_BLOCK_COUNT]>,
    after_history: Flat7xD0OverlapHistory,
}

impl Flat7xD0PcmBlockSeriesCandidate {
    pub fn spectra(&self) -> &[Flat7xD0SpectrumBlock; FLAT7X_D0_BAP_BLOCK_COUNT] {
        self.spectra.as_ref()
    }

    pub fn exponent_rows(
        &self,
    ) -> &[[Flat7xD0ExponentRow; FLAT7X_D0_BAP_CHANNEL_COUNT]; FLAT7X_D0_BAP_BLOCK_COUNT] {
        self.exponent_rows.as_ref()
    }

    pub const fn after_history(&self) -> &Flat7xD0OverlapHistory {
        &self.after_history
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

/// Complete D0 frame candidate paired with the PCM block-series and history proposal that fed it.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xD0PcmFrameCandidate {
    block_series: Flat7xD0PcmBlockSeriesCandidate,
    frame: Flat7xD0AudioFrameCandidate,
    source_extent: Flat7xD0PcmSourceExtent,
}

impl Flat7xD0PcmFrameCandidate {
    pub const fn block_series(&self) -> &Flat7xD0PcmBlockSeriesCandidate {
        &self.block_series
    }

    pub const fn frame(&self) -> &Flat7xD0AudioFrameCandidate {
        &self.frame
    }

    pub const fn after_history(&self) -> &Flat7xD0OverlapHistory {
        self.block_series.after_history()
    }

    pub const fn source_extent(&self) -> Flat7xD0PcmSourceExtent {
        self.source_extent
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

/// Caller-owned frame controls for a source-PCM-to-D0-frame candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0PcmFrameSettings<'a> {
    pub source_extent: Flat7xD0PcmSourceExtent,
    pub before: &'a Flat7xD0OverlapHistory,
    pub transform_parameters: &'a MathGeneratedTransformParameters,
    pub target: Flat7xI0D0TargetPlan,
    pub bsi_structure: Flat7xD0BsiStructure<'a>,
    pub complete_emdf_container: &'a [u8],
    pub bap_parameters: &'a Flat7xD0BapParameters,
    pub dynamic_range_codes: [Option<u8>; FLAT7X_D0_BAP_BLOCK_COUNT],
}

/// Inputs to the source-PCM-to-D0-frame functional core.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0PcmFrameInput<'a> {
    pub pcm_rows: [&'a [u32]; FLAT7X_D0_BAP_CHANNEL_COUNT],
    pub settings: Flat7xD0PcmFrameSettings<'a>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0PcmSourceError {
    RowLengthMismatch {
        channel: u8,
        actual_samples: usize,
    },
    Spectrum {
        block: u8,
        source: Flat7xD0SpectrumError,
    },
    Exponent {
        block: u8,
        channel: u8,
        source: Flat7xD0ExponentError,
    },
    SourceExtent(Flat7xD0PcmSourceExtentError),
    PcmPaddingMismatch {
        channel: u8,
    },
    AudioFrame(Flat7xD0AudioFrameError),
    BlockCountMismatch,
}

impl Flat7xD0PcmSourceError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::RowLengthMismatch { .. } => "D0_PCM_SOURCE_ROW_LENGTH_MISMATCH",
            Self::Spectrum { source, .. } => source.code(),
            Self::Exponent { source, .. } => source.code(),
            Self::SourceExtent(_) => "D0_PCM_SOURCE_EXTENT_MISMATCH",
            Self::PcmPaddingMismatch { .. } => "D0_PCM_SOURCE_PADDING_MISMATCH",
            Self::AudioFrame(_) => "D0_PCM_SOURCE_FRAME_PRODUCTION_FAILED",
            Self::BlockCountMismatch => "D0_PCM_SOURCE_BLOCK_COUNT_MISMATCH",
        }
    }
}

/// Produces the complete D0 frame candidate from one source's PCM rows and explicit frame inputs.
///
/// BSI, EMDF, BAP parameters and dynamic-range codes remain caller-owned inputs. The result keeps
/// the PCM/history proposal beside its frame so a higher owner can bind and commit them together.
pub fn produce_flat7x_d0_frame_from_pcm_source(
    input: Flat7xD0PcmFrameInput<'_>,
) -> Result<Flat7xD0PcmFrameCandidate, Flat7xD0PcmSourceError> {
    let settings = input.settings;
    let valid_samples = usize::from(settings.source_extent.valid_samples());
    for (channel, row) in input.pcm_rows.iter().enumerate() {
        if row.len() == SOURCE_SAMPLES && row[valid_samples..].iter().any(|word| *word != 0) {
            return Err(Flat7xD0PcmSourceError::PcmPaddingMismatch {
                channel: channel as u8,
            });
        }
    }
    let block_series = stage_flat7x_d0_pcm_block_series(
        input.pcm_rows,
        settings.before,
        settings.transform_parameters,
    )?;
    let frame = produce_flat7x_d0_audio_frame(&Flat7xD0AudioFrameInput {
        target: settings.target,
        bsi_structure: settings.bsi_structure,
        complete_emdf_container: settings.complete_emdf_container,
        bap_parameters: settings.bap_parameters,
        spectrum_by_block: block_series.spectra(),
        exponent_rows_by_block: block_series.exponent_rows(),
        dynamic_range_codes: settings.dynamic_range_codes,
    })
    .map_err(Flat7xD0PcmSourceError::AudioFrame)?;

    Ok(Flat7xD0PcmFrameCandidate {
        block_series,
        frame,
        source_extent: settings.source_extent,
    })
}

/// Stages the target's six 256-sample blocks from four D0 PCM rows and committed overlap history.
///
/// The rows are ordered `[Ls, Rs, Lrs, Rrs]`, contain 1,536 f32 bit-words each, and include any
/// upstream zero padding. The function returns the next 256-sample history as a candidate; it
/// does not install state, bind programme/source identity, or publish a frame.
pub fn stage_flat7x_d0_pcm_block_series(
    pcm_rows: [&[u32]; FLAT7X_D0_BAP_CHANNEL_COUNT],
    before: &Flat7xD0OverlapHistory,
    parameters: &MathGeneratedTransformParameters,
) -> Result<Flat7xD0PcmBlockSeriesCandidate, Flat7xD0PcmSourceError> {
    for (channel, row) in pcm_rows.iter().enumerate() {
        if row.len() != SOURCE_SAMPLES {
            return Err(Flat7xD0PcmSourceError::RowLengthMismatch {
                channel: channel as u8,
                actual_samples: row.len(),
            });
        }
    }

    let mut spectra = Vec::with_capacity(FLAT7X_D0_BAP_BLOCK_COUNT);
    let mut exponent_rows_by_block = Vec::with_capacity(FLAT7X_D0_BAP_BLOCK_COUNT);
    for block in 0..FLAT7X_D0_BAP_BLOCK_COUNT {
        let block_start = block * D0_PCM_SAMPLES_PER_BLOCK;
        let mut channels = [Flat7xD0PcmWindow {
            previous_samples: [0.0; D0_PCM_SAMPLES_PER_BLOCK],
            new_samples: [0.0; D0_PCM_SAMPLES_PER_BLOCK],
        }; FLAT7X_D0_BAP_CHANNEL_COUNT];
        for channel in 0..FLAT7X_D0_BAP_CHANNEL_COUNT {
            for sample in 0..D0_PCM_SAMPLES_PER_BLOCK {
                let previous_word = if block == 0 {
                    before.samples_by_channel[channel][sample]
                } else {
                    pcm_rows[channel][block_start - D0_PCM_SAMPLES_PER_BLOCK + sample]
                };
                channels[channel].previous_samples[sample] = f32::from_bits(previous_word);
                channels[channel].new_samples[sample] =
                    f32::from_bits(pcm_rows[channel][block_start + sample]);
            }
        }

        let spectrum =
            transform_flat7x_d0_pcm_block(&Flat7xD0PcmBlockInput { channels }, parameters)
                .map_err(|source| Flat7xD0PcmSourceError::Spectrum {
                    block: block as u8,
                    source,
                })?;
        let coefficient_rows = spectrum.coefficient_bits_by_channel();
        let exponent_rows = [
            prepare_flat7x_d0_d15_exponent_row(&coefficient_rows[0]).map_err(|source| {
                Flat7xD0PcmSourceError::Exponent {
                    block: block as u8,
                    channel: 0,
                    source,
                }
            })?,
            prepare_flat7x_d0_d15_exponent_row(&coefficient_rows[1]).map_err(|source| {
                Flat7xD0PcmSourceError::Exponent {
                    block: block as u8,
                    channel: 1,
                    source,
                }
            })?,
            prepare_flat7x_d0_d15_exponent_row(&coefficient_rows[2]).map_err(|source| {
                Flat7xD0PcmSourceError::Exponent {
                    block: block as u8,
                    channel: 2,
                    source,
                }
            })?,
            prepare_flat7x_d0_d15_exponent_row(&coefficient_rows[3]).map_err(|source| {
                Flat7xD0PcmSourceError::Exponent {
                    block: block as u8,
                    channel: 3,
                    source,
                }
            })?,
        ];
        spectra.push(spectrum);
        exponent_rows_by_block.push(exponent_rows);
    }

    let spectra: Box<[Flat7xD0SpectrumBlock; FLAT7X_D0_BAP_BLOCK_COUNT]> = spectra
        .into_boxed_slice()
        .try_into()
        .map_err(|_| Flat7xD0PcmSourceError::BlockCountMismatch)?;
    let exponent_rows: Box<
        [[Flat7xD0ExponentRow; FLAT7X_D0_BAP_CHANNEL_COUNT]; FLAT7X_D0_BAP_BLOCK_COUNT],
    > = exponent_rows_by_block
        .into_boxed_slice()
        .try_into()
        .map_err(|_| Flat7xD0PcmSourceError::BlockCountMismatch)?;

    let mut after_history = Flat7xD0OverlapHistory::cold();
    for (history, row) in after_history.samples_by_channel.iter_mut().zip(pcm_rows) {
        history.copy_from_slice(&row[SOURCE_SAMPLES - D0_PCM_SAMPLES_PER_BLOCK..]);
    }

    Ok(Flat7xD0PcmBlockSeriesCandidate {
        spectra,
        exponent_rows,
        after_history,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        Flat7xD0OverlapHistory, Flat7xD0PcmSourceError, SOURCE_SAMPLES,
        stage_flat7x_d0_pcm_block_series,
    };
    use crate::transform::math_generated_parameters::MathGeneratedTransformParameters;

    fn pcm_rows(level: f32) -> [Vec<u32>; 4] {
        std::array::from_fn(|_| vec![level.to_bits(); SOURCE_SAMPLES])
    }

    fn row_refs(rows: &[Vec<u32>; 4]) -> [&[u32]; 4] {
        std::array::from_fn(|channel| rows[channel].as_slice())
    }

    fn parameters() -> MathGeneratedTransformParameters {
        MathGeneratedTransformParameters::selected_math_generated_v1().unwrap()
    }

    #[test]
    fn stages_six_ordered_blocks_and_the_source_tail_as_next_history() {
        let rows: [Vec<u32>; 4] = std::array::from_fn(|channel| {
            (0..SOURCE_SAMPLES)
                .map(|sample| ((channel as f32 + 1.0) * (sample as f32 / 8_000.0)).to_bits())
                .collect()
        });

        let candidate = stage_flat7x_d0_pcm_block_series(
            row_refs(&rows),
            &Flat7xD0OverlapHistory::cold(),
            &parameters(),
        )
        .unwrap();

        assert_eq!(candidate.spectra().len(), 6);
        assert_eq!(candidate.exponent_rows().len(), 6);
        for (history, row) in candidate
            .after_history()
            .samples_by_channel()
            .iter()
            .zip(&rows)
        {
            assert_eq!(history.as_slice(), &row[SOURCE_SAMPLES - 256..]);
        }
        assert!(!candidate.production_allowed());
    }

    #[test]
    fn first_block_uses_committed_overlap_from_the_prior_source() {
        let nonzero_rows = pcm_rows(0.5);
        let first = stage_flat7x_d0_pcm_block_series(
            row_refs(&nonzero_rows),
            &Flat7xD0OverlapHistory::cold(),
            &parameters(),
        )
        .unwrap();
        let silent_rows = pcm_rows(0.0);
        let next = stage_flat7x_d0_pcm_block_series(
            row_refs(&silent_rows),
            first.after_history(),
            &parameters(),
        )
        .unwrap();

        assert_eq!(next.spectra()[0].gain_range_codes()[0], 0);
        assert_eq!(next.spectra()[1].gain_range_codes()[0], 3);
    }

    #[test]
    fn rejects_a_source_row_without_the_complete_frame_extent() {
        let rows = pcm_rows(0.0);
        let mut row_refs = row_refs(&rows);
        row_refs[2] = &row_refs[2][..SOURCE_SAMPLES - 1];

        assert_eq!(
            stage_flat7x_d0_pcm_block_series(
                row_refs,
                &Flat7xD0OverlapHistory::cold(),
                &parameters(),
            ),
            Err(Flat7xD0PcmSourceError::RowLengthMismatch {
                channel: 2,
                actual_samples: SOURCE_SAMPLES - 1,
            })
        );
    }
}
