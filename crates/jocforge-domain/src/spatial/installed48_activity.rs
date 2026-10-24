// pattern: Functional Core
//! Selected primary-source per-callback installed48 activity production.
//!
//! The producer returns a proposed after-history, cursor, cache and callback
//! state. It has no file I/O, cross-owner commit, or encode-path authority.

use crate::numeric::FiniteF32;
use crate::spatial::installed48_source_map::{
    Installed48PostRouteRow, Installed48SourceMap, PrimaryPrefixRoute, field_masks_hex,
};
use crate::spatial::prefix_membership::PrefixMembershipProducer;
use crate::spatial::source_flags::{triangle_activity_witness, triangle_window};
use crate::timeline::{AdmSampleIndex, FrontendCallbackIndex};

const FRAME_SAMPLES: usize = 1_536;
const CACHED_DRAIN_CALLBACKS: u64 = 4;
const SELECTED_SOURCE_SAMPLES: u64 = 6_522_906;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimaryActivityPhase {
    NormalRead,
    CachedDrain,
}

impl PrimaryActivityPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NormalRead => "normal",
            Self::CachedDrain => "cached_flush",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimaryActivityError {
    SourceBinding,
    SourceExtent,
    CallbackSequenceComplete,
    CallbackPhase,
    ReaderPosition,
    ReaderFrameShape,
    NonfinitePcm,
    HistoryShape,
    CachedFrameMissing,
    ActivityWindow,
    NumericFailure,
    MembershipProducer,
    PostRoute,
}

impl PrimaryActivityError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::SourceBinding => "PRIMARY_ACTIVITY_SOURCE_BINDING",
            Self::SourceExtent => "PRIMARY_ACTIVITY_SOURCE_EXTENT",
            Self::CallbackSequenceComplete => "PRIMARY_ACTIVITY_CALLBACKS_COMPLETE",
            Self::CallbackPhase => "PRIMARY_ACTIVITY_CALLBACK_PHASE",
            Self::ReaderPosition => "PRIMARY_ACTIVITY_READER_POSITION",
            Self::ReaderFrameShape => "PRIMARY_ACTIVITY_READER_FRAME_SHAPE",
            Self::NonfinitePcm => "PRIMARY_ACTIVITY_PCM_NONFINITE",
            Self::HistoryShape => "PRIMARY_ACTIVITY_HISTORY_SHAPE",
            Self::CachedFrameMissing => "PRIMARY_ACTIVITY_CACHE_MISSING",
            Self::ActivityWindow => "PRIMARY_ACTIVITY_WINDOW",
            Self::NumericFailure => "PRIMARY_ACTIVITY_NUMERIC",
            Self::MembershipProducer => "PRIMARY_ACTIVITY_MEMBERSHIP",
            Self::PostRoute => "PRIMARY_ACTIVITY_POST_ROUTE",
        }
    }
}

#[derive(Debug)]
pub enum PrimaryActivityInput {
    NormalRead {
        reader_relative_sample: AdmSampleIndex,
        pcm_word_rows: Vec<Vec<u32>>,
    },
    CachedDrain,
}

#[derive(Debug, PartialEq)]
pub struct PrimaryActivityOutput {
    callback_index: FrontendCallbackIndex,
    phase: PrimaryActivityPhase,
    host_frame: Option<FrontendCallbackIndex>,
    reader_relative_sample: AdmSampleIndex,
    actual_source_samples: u16,
    advertised_frame_samples: u16,
    quiet37: [u8; 37],
    prefix_writer_indices: Vec<u8>,
    field0_mask_hex: String,
    field4_mask_hex: String,
    postroute_rows: [Installed48PostRouteRow; 48],
}

impl PrimaryActivityOutput {
    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn phase(&self) -> PrimaryActivityPhase {
        self.phase
    }

    pub const fn host_frame(&self) -> Option<FrontendCallbackIndex> {
        self.host_frame
    }

    pub const fn reader_relative_sample(&self) -> AdmSampleIndex {
        self.reader_relative_sample
    }

    pub const fn actual_source_samples(&self) -> u16 {
        self.actual_source_samples
    }

    pub const fn advertised_frame_samples(&self) -> u16 {
        self.advertised_frame_samples
    }

    pub const fn quiet37(&self) -> &[u8; 37] {
        &self.quiet37
    }

    pub fn prefix_writer_indices(&self) -> &[u8] {
        &self.prefix_writer_indices
    }

    pub fn field0_mask_hex(&self) -> &str {
        &self.field0_mask_hex
    }

    pub fn field4_mask_hex(&self) -> &str {
        &self.field4_mask_hex
    }

    pub fn postroute_rows(&self) -> &[Installed48PostRouteRow; 48] {
        &self.postroute_rows
    }
}

#[derive(Debug)]
pub struct PrimaryActivityProposal {
    output: PrimaryActivityOutput,
    after_state: PrimaryActivityState,
    membership_route: PrimaryPrefixRoute,
}

impl PrimaryActivityProposal {
    pub fn output(&self) -> &PrimaryActivityOutput {
        &self.output
    }

    pub fn into_parts(
        self,
    ) -> (
        PrimaryActivityOutput,
        PrimaryActivityState,
        PrimaryPrefixRoute,
    ) {
        (self.output, self.after_state, self.membership_route)
    }
}

/// Dynamic state proposed by this narrow activity owner; parent transaction remains external.
#[derive(Debug, PartialEq)]
pub struct PrimaryActivityState {
    source_receipt_sha256: [u8; 32],
    source_samples: u64,
    normal_callback_count: u64,
    total_callback_count: u64,
    next_callback: FrontendCallbackIndex,
    reader_cursor: AdmSampleIndex,
    last_reader_start: Option<AdmSampleIndex>,
    regular_history: Box<[Box<[FiniteF32]>]>,
    cached_frame: Option<PrimaryAudioFrame>,
}

impl PrimaryActivityState {
    pub const fn next_callback(&self) -> FrontendCallbackIndex {
        self.next_callback
    }

    pub const fn reader_cursor(&self) -> AdmSampleIndex {
        self.reader_cursor
    }

    pub const fn normal_callback_count(&self) -> u64 {
        self.normal_callback_count
    }

    pub const fn total_callback_count(&self) -> u64 {
        self.total_callback_count
    }
}

/// Uses the frozen 1536-sample triangle and selected primary source extent.
pub struct PrimaryInstalled48ActivityProducer {
    source_receipt_sha256: [u8; 32],
    source_samples: u64,
    normal_callback_count: u64,
    total_callback_count: u64,
    activity_window: Box<[FiniteF32]>,
    membership: PrefixMembershipProducer,
}

impl PrimaryInstalled48ActivityProducer {
    pub fn new(source_map: &Installed48SourceMap) -> Result<Self, PrimaryActivityError> {
        let binding = source_map.source_binding();
        let source_samples = binding.source_samples().value();
        if source_samples != SELECTED_SOURCE_SAMPLES || binding.data_bytes() / 114 != source_samples
        {
            return Err(PrimaryActivityError::SourceExtent);
        }
        let normal_callback_count = source_samples.div_ceil(FRAME_SAMPLES as u64);
        if normal_callback_count != 4_247 {
            return Err(PrimaryActivityError::SourceExtent);
        }
        let total_callback_count = normal_callback_count + CACHED_DRAIN_CALLBACKS;
        let activity_window =
            triangle_window().map_err(|_| PrimaryActivityError::ActivityWindow)?;
        if activity_window.len() != FRAME_SAMPLES * 2 {
            return Err(PrimaryActivityError::ActivityWindow);
        }
        let membership =
            PrefixMembershipProducer::for_selected_primary_p11(source_map.p11_geometry())
                .map_err(|_| PrimaryActivityError::MembershipProducer)?;
        Ok(Self {
            source_receipt_sha256: *binding.source_receipt_sha256(),
            source_samples,
            normal_callback_count,
            total_callback_count,
            activity_window,
            membership,
        })
    }

    pub fn cold_state(&self) -> Result<PrimaryActivityState, PrimaryActivityError> {
        let regular_history = fresh_history()?;
        Ok(PrimaryActivityState {
            source_receipt_sha256: self.source_receipt_sha256,
            source_samples: self.source_samples,
            normal_callback_count: self.normal_callback_count,
            total_callback_count: self.total_callback_count,
            next_callback: FrontendCallbackIndex::new(0),
            reader_cursor: AdmSampleIndex::new(0),
            last_reader_start: None,
            regular_history,
            cached_frame: None,
        })
    }

    /// Proposes one normal read or cached drain callback and its after-history.
    pub fn produce_next(
        &self,
        source_map: &Installed48SourceMap,
        before: &PrimaryActivityState,
        input: PrimaryActivityInput,
    ) -> Result<PrimaryActivityProposal, PrimaryActivityError> {
        if before.source_receipt_sha256 != self.source_receipt_sha256
            || before.source_samples != self.source_samples
            || before.normal_callback_count != self.normal_callback_count
            || before.total_callback_count != self.total_callback_count
            || source_map.source_binding().source_receipt_sha256() != &self.source_receipt_sha256
        {
            return Err(PrimaryActivityError::SourceBinding);
        }
        if before.regular_history.len() != 37
            || before
                .regular_history
                .iter()
                .any(|row| row.len() != FRAME_SAMPLES)
        {
            return Err(PrimaryActivityError::HistoryShape);
        }
        let callback = before.next_callback.value();
        if callback >= self.total_callback_count {
            return Err(PrimaryActivityError::CallbackSequenceComplete);
        }
        let expected_phase = if callback < self.normal_callback_count {
            PrimaryActivityPhase::NormalRead
        } else {
            PrimaryActivityPhase::CachedDrain
        };
        let (
            frame,
            after_cache,
            actual_samples,
            host_frame,
            reader_relative_sample,
            after_cursor,
            after_last_start,
        ) = match (expected_phase, input) {
            (
                PrimaryActivityPhase::NormalRead,
                PrimaryActivityInput::NormalRead {
                    reader_relative_sample,
                    pcm_word_rows,
                },
            ) => {
                if reader_relative_sample.value() != before.reader_cursor.value()
                    || reader_relative_sample.value() >= self.source_samples
                {
                    return Err(PrimaryActivityError::ReaderPosition);
                }
                let remaining = self.source_samples - before.reader_cursor.value();
                let expected_samples = remaining.min(FRAME_SAMPLES as u64) as usize;
                let frame = PrimaryAudioFrame::from_normal_read(pcm_word_rows, expected_samples)?;
                let after_cursor = AdmSampleIndex::new(
                    before
                        .reader_cursor
                        .value()
                        .checked_add(expected_samples as u64)
                        .ok_or(PrimaryActivityError::SourceExtent)?,
                );
                (
                    frame.clone(),
                    Some(frame),
                    expected_samples,
                    Some(FrontendCallbackIndex::new(callback)),
                    reader_relative_sample,
                    after_cursor,
                    Some(reader_relative_sample),
                )
            }
            (PrimaryActivityPhase::CachedDrain, PrimaryActivityInput::CachedDrain) => {
                if before.reader_cursor.value() != self.source_samples {
                    return Err(PrimaryActivityError::ReaderPosition);
                }
                let frame = before
                    .cached_frame
                    .as_ref()
                    .ok_or(PrimaryActivityError::CachedFrameMissing)?
                    .clone();
                let last_start = before
                    .last_reader_start
                    .ok_or(PrimaryActivityError::CachedFrameMissing)?;
                (
                    frame.clone(),
                    Some(frame),
                    0,
                    None,
                    last_start,
                    before.reader_cursor,
                    before.last_reader_start,
                )
            }
            _ => return Err(PrimaryActivityError::CallbackPhase),
        };

        let quiet37 = produce_quiet37(
            source_map,
            &before.regular_history,
            &frame,
            &self.activity_window,
        )?;
        let membership_route = source_map
            .produce_primary_prefix_route(&quiet37, &self.membership)
            .map_err(|_| PrimaryActivityError::MembershipProducer)?;
        let postroute_rows = source_map
            .postroute_rows(&quiet37, membership_route.writers())
            .map_err(|_| PrimaryActivityError::PostRoute)?;
        let (field0_mask_hex, field4_mask_hex) =
            field_masks_hex(&postroute_rows).map_err(|_| PrimaryActivityError::PostRoute)?;
        let after_history = source_map
            .physical_to_global_source()
            .iter()
            .map(|binding| {
                frame
                    .channel(binding.physical_host().value())
                    .map(|row| row.to_vec().into_boxed_slice())
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        let next_callback = FrontendCallbackIndex::new(
            callback
                .checked_add(1)
                .ok_or(PrimaryActivityError::CallbackSequenceComplete)?,
        );
        let after_state = PrimaryActivityState {
            source_receipt_sha256: before.source_receipt_sha256,
            source_samples: before.source_samples,
            normal_callback_count: before.normal_callback_count,
            total_callback_count: before.total_callback_count,
            next_callback,
            reader_cursor: after_cursor,
            last_reader_start: after_last_start,
            regular_history: after_history,
            cached_frame: after_cache,
        };
        let output = PrimaryActivityOutput {
            callback_index: before.next_callback,
            phase: expected_phase,
            host_frame,
            reader_relative_sample,
            actual_source_samples: u16::try_from(actual_samples)
                .map_err(|_| PrimaryActivityError::ReaderFrameShape)?,
            advertised_frame_samples: FRAME_SAMPLES as u16,
            quiet37,
            prefix_writer_indices: membership_route.writer_indices(),
            field0_mask_hex,
            field4_mask_hex,
            postroute_rows,
        };
        Ok(PrimaryActivityProposal {
            output,
            after_state,
            membership_route,
        })
    }
}

fn produce_quiet37(
    source_map: &Installed48SourceMap,
    history: &[Box<[FiniteF32]>],
    frame: &PrimaryAudioFrame,
    activity_window: &[FiniteF32],
) -> Result<[u8; 37], PrimaryActivityError> {
    if history.len() != 37 || source_map.physical_to_global_source().len() != 37 {
        return Err(PrimaryActivityError::HistoryShape);
    }
    let mut quiet37 = [1_u8; 37];
    for (regular_ordinal, binding) in source_map.physical_to_global_source().iter().enumerate() {
        let row = source_map
            .row_for_source(binding.source_index())
            .ok_or(PrimaryActivityError::SourceBinding)?;
        let mute = row
            .input_mute()
            .ok_or(PrimaryActivityError::SourceBinding)?;
        if mute > 1 {
            return Err(PrimaryActivityError::SourceBinding);
        }
        let current = frame.channel(binding.physical_host().value())?;
        let witness =
            triangle_activity_witness(&history[regular_ordinal], current, activity_window)
                .map_err(|_| PrimaryActivityError::NumericFailure)?;
        quiet37[regular_ordinal] = u8::from(!(witness && mute == 0));
    }
    Ok(quiet37)
}

#[derive(Debug, PartialEq)]
struct PrimaryAudioFrame {
    channels: Box<[Box<[FiniteF32]>]>,
}

impl Clone for PrimaryAudioFrame {
    fn clone(&self) -> Self {
        Self {
            channels: self
                .channels
                .iter()
                .map(|row| row.to_vec().into_boxed_slice())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }
}

impl PrimaryAudioFrame {
    fn from_normal_read(
        pcm_word_rows: Vec<Vec<u32>>,
        actual_samples: usize,
    ) -> Result<Self, PrimaryActivityError> {
        if pcm_word_rows.len() != 38
            || actual_samples == 0
            || actual_samples > FRAME_SAMPLES
            || pcm_word_rows.iter().any(|row| row.len() != actual_samples)
        {
            return Err(PrimaryActivityError::ReaderFrameShape);
        }
        let positive_zero =
            FiniteF32::new(0.0).map_err(|_| PrimaryActivityError::NumericFailure)?;
        let mut channels = Vec::with_capacity(38);
        for row in pcm_word_rows {
            let mut values = Vec::with_capacity(FRAME_SAMPLES);
            for word in row {
                values.push(
                    FiniteF32::new(f32::from_bits(word))
                        .map_err(|_| PrimaryActivityError::NonfinitePcm)?,
                );
            }
            values.resize(FRAME_SAMPLES, positive_zero);
            channels.push(values.into_boxed_slice());
        }
        Ok(Self {
            channels: channels.into_boxed_slice(),
        })
    }

    fn channel(&self, physical_host: u8) -> Result<&[FiniteF32], PrimaryActivityError> {
        self.channels
            .get(usize::from(physical_host))
            .map(Box::as_ref)
            .ok_or(PrimaryActivityError::ReaderFrameShape)
    }
}

fn fresh_history() -> Result<Box<[Box<[FiniteF32]>]>, PrimaryActivityError> {
    let zero = FiniteF32::new(0.0).map_err(|_| PrimaryActivityError::NumericFailure)?;
    Ok((0..37)
        .map(|_| vec![zero; FRAME_SAMPLES].into_boxed_slice())
        .collect::<Vec<_>>()
        .into_boxed_slice())
}
