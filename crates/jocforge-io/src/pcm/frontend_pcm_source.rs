// pattern: Functional Core
//! Reads one approved MAIN frontend callback window from an already parsed ADM BWF.

use jocforge_domain::timeline::source_for_callback;
pub use jocforge_domain::timeline::{
    AdmSampleIndex, CodecSourceIndex, FrontendCallbackIndex, NormalCallbackCount,
};

use super::riff_pcm24::{
    ParsedRiffPcm24, Pcm24FrameCount, RiffPcm24Error, SELECTED_SAMPLE_RATE_HZ,
};

const FRAME_SAMPLES: u64 = 1_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontendPcmSourceError {
    InputIdentityMismatch,
    SelectedProfileMismatch,
    CallbackOutOfRange,
    SampleRangeOverflow,
    Reader(RiffPcm24Error),
}

/// PCM and clock values for one normal MAIN callback.
#[must_use]
pub struct FrontendPcmCallbackFrame {
    input_sha256: [u8; 32],
    callback_index: FrontendCallbackIndex,
    normal_callbacks: NormalCallbackCount,
    first_sample: AdmSampleIndex,
    actual_samples: Pcm24FrameCount,
    padding_samples: u64,
    source_index: Option<CodecSourceIndex>,
    physical_pcm_rows: Vec<Vec<u32>>,
}

impl FrontendPcmCallbackFrame {
    pub const fn input_sha256(&self) -> &[u8; 32] {
        &self.input_sha256
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn normal_callbacks(&self) -> NormalCallbackCount {
        self.normal_callbacks
    }

    pub const fn first_sample(&self) -> AdmSampleIndex {
        self.first_sample
    }

    pub const fn actual_samples(&self) -> Pcm24FrameCount {
        self.actual_samples
    }

    pub const fn padding_samples(&self) -> u64 {
        self.padding_samples
    }

    pub const fn source_index(&self) -> Option<CodecSourceIndex> {
        self.source_index
    }

    pub fn physical_pcm_rows(&self) -> &[Vec<u32>] {
        &self.physical_pcm_rows
    }
}

/// Decodes the physical PCM rows for one MAIN callback using the approved F01.3 RIFF model.
///
/// Callback indices 0 and 1 carry PCM windows but have no codec source index. The accepted
/// callback-to-source mapping comes from the selected clock contract; row order remains the
/// parsed physical slot order for the ADM source-map stage.
pub fn read_frontend_pcm_callback(
    parsed_source: &ParsedRiffPcm24<'_>,
    selected_input_sha256: [u8; 32],
    callback_index: FrontendCallbackIndex,
) -> Result<FrontendPcmCallbackFrame, FrontendPcmSourceError> {
    if parsed_source.source_sha256() != selected_input_sha256 {
        return Err(FrontendPcmSourceError::InputIdentityMismatch);
    }
    let physical_channel_count = usize::from(parsed_source.channels());
    if parsed_source.sample_rate_hz() != SELECTED_SAMPLE_RATE_HZ
        || physical_channel_count == 0
        || usize::from(parsed_source.block_align()) != physical_channel_count * 3
    {
        return Err(FrontendPcmSourceError::SelectedProfileMismatch);
    }

    let source_samples = parsed_source.total_samples();
    let normal_callbacks = source_samples.div_ceil(FRAME_SAMPLES);
    if u64::from(callback_index.value()) >= normal_callbacks {
        return Err(FrontendPcmSourceError::CallbackOutOfRange);
    }
    let first_sample_value = u64::from(callback_index.value())
        .checked_mul(FRAME_SAMPLES)
        .ok_or(FrontendPcmSourceError::SampleRangeOverflow)?;
    let remaining = source_samples
        .checked_sub(first_sample_value)
        .ok_or(FrontendPcmSourceError::SampleRangeOverflow)?;
    let actual_samples = remaining.min(FRAME_SAMPLES);
    let first_sample = AdmSampleIndex::new(first_sample_value);
    let decoded = parsed_source
        .read_pcm24_chunk(first_sample, actual_samples)
        .map_err(FrontendPcmSourceError::Reader)?;
    if decoded.actual_frames().value() != actual_samples {
        return Err(FrontendPcmSourceError::Reader(RiffPcm24Error::InvalidInput));
    }

    let actual_frame_count = decoded.actual_frames();
    let mut physical_pcm_rows = decoded.into_planar_f32_bits();
    for row in &mut physical_pcm_rows {
        row.resize(FRAME_SAMPLES as usize, 0);
    }
    if physical_pcm_rows.len() != physical_channel_count
        || physical_pcm_rows
            .iter()
            .any(|row| row.len() != FRAME_SAMPLES as usize)
    {
        return Err(FrontendPcmSourceError::Reader(RiffPcm24Error::InvalidInput));
    }

    Ok(FrontendPcmCallbackFrame {
        input_sha256: selected_input_sha256,
        callback_index,
        normal_callbacks: NormalCallbackCount::new(normal_callbacks),
        first_sample,
        actual_samples: actual_frame_count,
        padding_samples: FRAME_SAMPLES - actual_samples,
        source_index: source_for_callback(callback_index),
        physical_pcm_rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pcm::riff_pcm24::{
        SELECTED_BLOCK_ALIGN, SELECTED_CHANNELS, parse_selected_riff_pcm24,
    };

    fn put_u16(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn put_u32(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn pcm24_wave(sample_count: usize) -> Vec<u8> {
        let mut data = Vec::with_capacity(sample_count * SELECTED_BLOCK_ALIGN);
        for sample in 0..sample_count {
            for channel in 0..SELECTED_CHANNELS {
                let value = i32::try_from(sample % 8 + channel).unwrap();
                let packed = value & 0x00ff_ffff;
                data.extend_from_slice(&packed.to_le_bytes()[..3]);
            }
        }

        let mut fmt = Vec::with_capacity(16);
        put_u16(&mut fmt, 1);
        put_u16(&mut fmt, SELECTED_CHANNELS as u16);
        put_u32(&mut fmt, SELECTED_SAMPLE_RATE_HZ);
        put_u32(
            &mut fmt,
            SELECTED_SAMPLE_RATE_HZ * SELECTED_BLOCK_ALIGN as u32,
        );
        put_u16(&mut fmt, SELECTED_BLOCK_ALIGN as u16);
        put_u16(&mut fmt, 24);

        let mut wave = Vec::new();
        wave.extend_from_slice(b"RIFF");
        let riff_size = 4 + 8 + fmt.len() + 8 + data.len();
        put_u32(&mut wave, u32::try_from(riff_size).unwrap());
        wave.extend_from_slice(b"WAVEfmt ");
        put_u32(&mut wave, u32::try_from(fmt.len()).unwrap());
        wave.extend_from_slice(&fmt);
        wave.extend_from_slice(b"data");
        put_u32(&mut wave, u32::try_from(data.len()).unwrap());
        wave.extend_from_slice(&data);
        wave
    }

    #[test]
    fn frontend_clock_reads_g0_g1_windows_and_maps_g2_to_source_zero() {
        let bytes = pcm24_wave(4 * FRAME_SAMPLES as usize + 1);
        let parsed = parse_selected_riff_pcm24(&bytes, None).unwrap();
        let identity = parsed.source_sha256();

        let g0 =
            read_frontend_pcm_callback(&parsed, identity, FrontendCallbackIndex::new(0)).unwrap();
        let g1 =
            read_frontend_pcm_callback(&parsed, identity, FrontendCallbackIndex::new(1)).unwrap();
        let g2 =
            read_frontend_pcm_callback(&parsed, identity, FrontendCallbackIndex::new(2)).unwrap();
        let g3 =
            read_frontend_pcm_callback(&parsed, identity, FrontendCallbackIndex::new(3)).unwrap();
        let g4 =
            read_frontend_pcm_callback(&parsed, identity, FrontendCallbackIndex::new(4)).unwrap();

        assert_eq!(g0.first_sample().value(), 0);
        assert_eq!(g0.actual_samples().value(), FRAME_SAMPLES);
        assert_eq!(g0.source_index(), None);
        assert_eq!(g1.first_sample().value(), FRAME_SAMPLES);
        assert_eq!(g1.actual_samples().value(), FRAME_SAMPLES);
        assert_eq!(g1.source_index(), None);
        assert_eq!(g2.first_sample().value(), 2 * FRAME_SAMPLES);
        assert_eq!(g2.source_index(), Some(CodecSourceIndex::new(0)));
        assert_eq!(g3.first_sample().value(), 3 * FRAME_SAMPLES);
        assert_eq!(g3.source_index(), Some(CodecSourceIndex::new(1)));
        assert_eq!(g3.actual_samples().value(), FRAME_SAMPLES);
        assert_eq!(g4.first_sample().value(), 4 * FRAME_SAMPLES);
        assert_eq!(g4.actual_samples().value(), 1);
        assert_eq!(g4.source_index(), Some(CodecSourceIndex::new(2)));
        assert_eq!(g4.padding_samples(), FRAME_SAMPLES - 1);
        assert_eq!(g4.physical_pcm_rows().len(), SELECTED_CHANNELS);
        assert!(
            g4.physical_pcm_rows()
                .iter()
                .all(|row| row[1..].iter().all(|word| *word == 0))
        );
    }
}
