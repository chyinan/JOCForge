// pattern: Functional Core
//! Selected short-read padding at the PCM reader/provider input boundary.
//!
//! This helper constructs a fresh, positive-zero 38-by-1536 provider row table,
//! then copies an opaque 38-by-1392 short-read prefix into it. It does not call
//! `read_float`, inspect sample values, or apply padding to renderer/DSP output.

/// Selected reader input channel count.
pub const SELECTED_READER_CHANNELS: usize = 38;
/// Selected reader input frame size per channel.
pub const SELECTED_READER_FRAME_SAMPLES: usize = 1_536;
/// Selected final short-read length per channel.
pub const SELECTED_READER_ACTUAL_SAMPLES: usize = 1_392;
/// Selected input tail size per channel.
pub const SELECTED_READER_TAIL_SAMPLES: usize =
    SELECTED_READER_FRAME_SAMPLES - SELECTED_READER_ACTUAL_SAMPLES;
/// F32 word width used for the selected row table.
pub const SELECTED_READER_WORD_BYTES: usize = 4;

/// Local row-shape rejection; no native status code is inferred.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedReaderPaddingError {
    ChannelCountMismatch,
    ActualReadLengthMismatch,
}

/// Opaque per-channel bit words returned by the selected short read.
///
/// The wrapper distinguishes reader input from renderer/DSP output. The words
/// are copied bit-for-bit; this module does not decode or numerically validate them.
#[derive(Clone, Copy, Debug)]
pub struct SelectedReaderReadPrefix<'a> {
    rows: &'a [Vec<u32>],
}

impl<'a> SelectedReaderReadPrefix<'a> {
    pub const fn new(rows: &'a [Vec<u32>]) -> Self {
        Self { rows }
    }
}

/// Fresh fixed-size input rows prepared for one selected reader frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedReaderInputFrameRows {
    rows: Vec<Vec<u32>>,
}

impl SelectedReaderInputFrameRows {
    pub fn rows(&self) -> &[Vec<u32>] {
        &self.rows
    }

    pub const fn channels(&self) -> usize {
        SELECTED_READER_CHANNELS
    }

    pub const fn samples_per_channel(&self) -> usize {
        SELECTED_READER_FRAME_SAMPLES
    }

    pub const fn tail_samples_per_channel(&self) -> usize {
        SELECTED_READER_TAIL_SAMPLES
    }
}

/// Builds the selected provider frame from an already-returned 1392-sample read.
/// The fresh full rows start as positive zero before the valid reader prefix is
/// copied, preserving every prefix bit and leaving the tail zero. This helper
/// does not execute or simulate the underlying reader call.
pub fn prepare_selected_reader_input_frame(
    prefix: SelectedReaderReadPrefix<'_>,
) -> Result<SelectedReaderInputFrameRows, SelectedReaderPaddingError> {
    if prefix.rows.len() != SELECTED_READER_CHANNELS {
        return Err(SelectedReaderPaddingError::ChannelCountMismatch);
    }
    if prefix
        .rows
        .iter()
        .any(|row| row.len() != SELECTED_READER_ACTUAL_SAMPLES)
    {
        return Err(SelectedReaderPaddingError::ActualReadLengthMismatch);
    }

    let mut rows = vec![vec![0_u32; SELECTED_READER_FRAME_SAMPLES]; SELECTED_READER_CHANNELS];
    for (output_row, input_row) in rows.iter_mut().zip(prefix.rows) {
        output_row[..SELECTED_READER_ACTUAL_SAMPLES].copy_from_slice(input_row);
    }
    Ok(SelectedReaderInputFrameRows { rows })
}
