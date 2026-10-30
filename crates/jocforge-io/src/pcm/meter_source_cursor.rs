// pattern: Functional Core
//! In-memory selected PCM24 source cursor for the independent meter schedule.

use jocforge_domain::{
    meter::sample_block_queue::{
        MeterCallbackCount, MeterCallbackIndex, MeterCallbackPhase, MeterSampleCount,
        MeterSourceSampleCount, MeterUpstreamGeneration,
    },
    meter::source_frame::MeterSourceReadIndex,
    timeline::AdmSampleIndex,
};

use super::{riff_pcm24::ParsedRiffPcm24, riff_pcm24::RiffPcm24Error};

pub const METER_SOURCE_CHANNELS: usize = 38;
pub const METER_SOURCE_FRAME_SAMPLES: usize = 1_536;
const METER_FRAME_SAMPLES_U64: u64 = 1_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterSourceCursorError {
    InvalidSourceExtent,
    SourceProfileMismatch,
    SourceExtentMismatch,
    SourceIdentityMismatch,
    CallbackOrder,
    ArithmeticOverflow,
    Reader(RiffPcm24Error),
}

impl MeterSourceCursorError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidSourceExtent => "METER_SOURCE_EXTENT_INVALID",
            Self::SourceProfileMismatch => "METER_SOURCE_PROFILE_MISMATCH",
            Self::SourceExtentMismatch => "METER_SOURCE_EXTENT_MISMATCH",
            Self::SourceIdentityMismatch => "METER_SOURCE_IDENTITY_MISMATCH",
            Self::CallbackOrder => "METER_SOURCE_CALLBACK_ORDER",
            Self::ArithmeticOverflow => "METER_SOURCE_ARITHMETIC_OVERFLOW",
            Self::Reader(_) => "METER_SOURCE_READER_ERROR",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MeterSourcePosition(u64);

impl MeterSourcePosition {
    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MeterPcmSourceIdentity([u8; 32]);

impl MeterPcmSourceIdentity {
    pub const fn from_selected_resource_sha256(value: [u8; 32]) -> Self {
        Self(value)
    }

    pub const fn sha256(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterPcmSourceCursor {
    source_identity: MeterPcmSourceIdentity,
    source_samples: MeterSourceSampleCount,
    normal_callbacks: MeterCallbackCount,
    callback_index: MeterCallbackIndex,
    source_cursor: MeterSourcePosition,
    read_index: MeterSourceReadIndex,
}

impl MeterPcmSourceCursor {
    pub fn cold(
        parsed_source: &ParsedRiffPcm24<'_>,
        selected_resource_identity: MeterPcmSourceIdentity,
        selected_source_samples: MeterSourceSampleCount,
    ) -> Result<Self, MeterSourceCursorError> {
        validate_selected_source(parsed_source)?;
        if parsed_source.source_sha256() != selected_resource_identity.sha256() {
            return Err(MeterSourceCursorError::SourceIdentityMismatch);
        }
        if parsed_source.total_samples() != selected_source_samples.value() {
            return Err(MeterSourceCursorError::SourceExtentMismatch);
        }
        let source_samples = selected_source_samples;
        let source_length = source_samples.value();
        if source_length == 0 || source_length > i64::MAX as u64 {
            return Err(MeterSourceCursorError::InvalidSourceExtent);
        }
        let normal_callbacks = source_length
            .checked_add(METER_FRAME_SAMPLES_U64 - 1)
            .ok_or(MeterSourceCursorError::ArithmeticOverflow)?
            / METER_FRAME_SAMPLES_U64;
        Ok(Self {
            source_identity: selected_resource_identity,
            source_samples,
            normal_callbacks: MeterCallbackCount::new(normal_callbacks),
            callback_index: MeterCallbackIndex::new(0),
            source_cursor: MeterSourcePosition(0),
            read_index: MeterSourceReadIndex::new(0),
        })
    }

    pub const fn source_samples(&self) -> MeterSourceSampleCount {
        self.source_samples
    }

    pub const fn source_identity(&self) -> MeterPcmSourceIdentity {
        self.source_identity
    }

    pub const fn normal_callbacks(&self) -> MeterCallbackCount {
        self.normal_callbacks
    }

    pub const fn callback_index(&self) -> MeterCallbackIndex {
        self.callback_index
    }

    pub const fn source_cursor(&self) -> MeterSourcePosition {
        self.source_cursor
    }

    pub const fn read_index(&self) -> MeterSourceReadIndex {
        self.read_index
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterRawInputFrame {
    callback_index: MeterCallbackIndex,
    upstream_generation: MeterUpstreamGeneration,
    phase: MeterCallbackPhase,
    source_cursor_before: MeterSourcePosition,
    source_cursor_after: MeterSourcePosition,
    source_read_index_before: MeterSourceReadIndex,
    source_read_index_after: MeterSourceReadIndex,
    host_input_count: MeterSampleCount,
    source_padding_count: MeterSampleCount,
    raw_input_rows: Vec<Vec<u32>>,
}

impl MeterRawInputFrame {
    pub const fn callback_index(&self) -> MeterCallbackIndex {
        self.callback_index
    }

    pub const fn upstream_generation(&self) -> MeterUpstreamGeneration {
        self.upstream_generation
    }

    pub const fn phase(&self) -> MeterCallbackPhase {
        self.phase
    }

    pub const fn source_cursor_before(&self) -> MeterSourcePosition {
        self.source_cursor_before
    }

    pub const fn source_cursor_after(&self) -> MeterSourcePosition {
        self.source_cursor_after
    }

    pub const fn source_read_index_before(&self) -> MeterSourceReadIndex {
        self.source_read_index_before
    }

    pub const fn source_read_index_after(&self) -> MeterSourceReadIndex {
        self.source_read_index_after
    }

    pub const fn host_input_count(&self) -> MeterSampleCount {
        self.host_input_count
    }

    pub const fn source_padding_count(&self) -> MeterSampleCount {
        self.source_padding_count
    }

    pub fn rows(&self) -> &[Vec<u32>] {
        &self.raw_input_rows
    }
}

#[must_use]
pub struct MeterPcmSourceProposal {
    after: MeterPcmSourceCursor,
    frame: MeterRawInputFrame,
}

impl MeterPcmSourceProposal {
    pub const fn after(&self) -> &MeterPcmSourceCursor {
        &self.after
    }

    pub const fn frame(&self) -> &MeterRawInputFrame {
        &self.frame
    }

    pub fn into_parts(self) -> (MeterPcmSourceCursor, MeterRawInputFrame) {
        (self.after, self.frame)
    }
}

/// Stages one normal-source or zero-input drain read from an already parsed
/// selected RIFF byte slice. The parsed source remains borrowed and no original
/// source bytes enter the cursor state.
pub fn stage_meter_pcm_source_frame(
    before: &MeterPcmSourceCursor,
    callback_index: MeterCallbackIndex,
    parsed_source: &ParsedRiffPcm24<'_>,
) -> Result<MeterPcmSourceProposal, MeterSourceCursorError> {
    validate_selected_source(parsed_source)?;
    if parsed_source.total_samples() != before.source_samples.value() {
        return Err(MeterSourceCursorError::SourceExtentMismatch);
    }
    if parsed_source.source_sha256() != before.source_identity.sha256() {
        return Err(MeterSourceCursorError::SourceIdentityMismatch);
    }
    if callback_index != before.callback_index {
        return Err(MeterSourceCursorError::CallbackOrder);
    }
    let total_callbacks = before
        .normal_callbacks
        .value()
        .checked_add(3)
        .ok_or(MeterSourceCursorError::ArithmeticOverflow)?;
    if callback_index.value() >= total_callbacks {
        return Err(MeterSourceCursorError::CallbackOrder);
    }

    let normal = callback_index.value() < before.normal_callbacks.value();
    let phase = if normal {
        MeterCallbackPhase::NormalSource
    } else {
        MeterCallbackPhase::ZeroInputDrain
    };
    let cursor_before = before.source_cursor.value();
    let (host_input_count, source_cursor_after, mut rows) = if normal {
        let remaining = before
            .source_samples
            .value()
            .checked_sub(cursor_before)
            .ok_or(MeterSourceCursorError::InvalidSourceExtent)?;
        let actual = remaining.min(METER_FRAME_SAMPLES_U64);
        if actual == 0 {
            return Err(MeterSourceCursorError::InvalidSourceExtent);
        }
        let first_sample = AdmSampleIndex::new(cursor_before);
        let decoded = parsed_source
            .read_pcm24_chunk(first_sample, actual)
            .map_err(MeterSourceCursorError::Reader)?;
        let read_count = decoded.actual_frames().value();
        if read_count != actual {
            return Err(MeterSourceCursorError::Reader(RiffPcm24Error::InvalidInput));
        }
        let rows = decoded.into_planar_f32_bits();
        let next_cursor = cursor_before
            .checked_add(read_count)
            .ok_or(MeterSourceCursorError::ArithmeticOverflow)?;
        (read_count, next_cursor, rows)
    } else {
        (0, cursor_before, vec![Vec::new(); METER_SOURCE_CHANNELS])
    };

    for row in &mut rows {
        if normal
            && row.len()
                != usize::try_from(host_input_count)
                    .map_err(|_| MeterSourceCursorError::ArithmeticOverflow)?
        {
            return Err(MeterSourceCursorError::SourceProfileMismatch);
        }
        row.resize(METER_SOURCE_FRAME_SAMPLES, 0);
    }
    if rows.len() != METER_SOURCE_CHANNELS
        || rows
            .iter()
            .any(|row| row.len() != METER_SOURCE_FRAME_SAMPLES)
    {
        return Err(MeterSourceCursorError::SourceProfileMismatch);
    }

    let source_read_index_before = before.read_index;
    let source_read_index_after = if normal {
        let next_read_index = source_read_index_before
            .value()
            .checked_add(1)
            .ok_or(MeterSourceCursorError::ArithmeticOverflow)?;
        MeterSourceReadIndex::new(next_read_index)
    } else {
        source_read_index_before
    };

    let next_index = callback_index
        .value()
        .checked_add(1)
        .ok_or(MeterSourceCursorError::ArithmeticOverflow)?;
    let next_generation = next_index;
    let after = MeterPcmSourceCursor {
        source_identity: before.source_identity,
        source_samples: before.source_samples,
        normal_callbacks: before.normal_callbacks,
        callback_index: MeterCallbackIndex::new(next_index),
        source_cursor: MeterSourcePosition(source_cursor_after),
        read_index: source_read_index_after,
    };
    let frame = MeterRawInputFrame {
        callback_index,
        upstream_generation: MeterUpstreamGeneration::new(next_generation),
        phase,
        source_cursor_before: MeterSourcePosition(cursor_before),
        source_cursor_after: MeterSourcePosition(source_cursor_after),
        source_read_index_before,
        source_read_index_after,
        host_input_count: MeterSampleCount::new(host_input_count),
        source_padding_count: MeterSampleCount::new(METER_FRAME_SAMPLES_U64 - host_input_count),
        raw_input_rows: rows,
    };
    Ok(MeterPcmSourceProposal { after, frame })
}

fn validate_selected_source(source: &ParsedRiffPcm24<'_>) -> Result<(), MeterSourceCursorError> {
    if source.sample_rate_hz() != 48_000
        || usize::from(source.channels()) != METER_SOURCE_CHANNELS
        || source.block_align() != 114
    {
        return Err(MeterSourceCursorError::SourceProfileMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pcm::riff_pcm24::parse_selected_riff_pcm24;
    use serde_json::Value;

    fn input_recipe() -> Value {
        serde_json::from_str(include_str!(
            "../../tests/fixtures/meter_source_cursor/input_only.json"
        ))
        .unwrap()
    }

    fn put_u16(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn put_u32(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn in_memory_pcm24_wave(recipe: &Value) -> Vec<u8> {
        let sample_count = usize::try_from(recipe["source_samples"].as_u64().unwrap()).unwrap();
        let channel_count = usize::try_from(recipe["channels"].as_u64().unwrap()).unwrap();
        let sample_rate = u32::try_from(recipe["sample_rate_hz"].as_u64().unwrap()).unwrap();
        let bits_per_sample = u16::try_from(recipe["bits_per_sample"].as_u64().unwrap()).unwrap();
        let block_align = u16::try_from(channel_count * usize::from(bits_per_sample) / 8).unwrap();
        let byte_rate = sample_rate * u32::from(block_align);
        let channel_multiplier = usize::try_from(
            recipe["sample_rule"]["channel_multiplier"]
                .as_u64()
                .unwrap(),
        )
        .unwrap();
        let sample_modulus =
            usize::try_from(recipe["sample_rule"]["sample_modulus"].as_u64().unwrap()).unwrap();
        let signed_offset =
            i32::try_from(recipe["sample_rule"]["signed_offset"].as_i64().unwrap()).unwrap();
        let mut data = Vec::with_capacity(sample_count * usize::from(block_align));
        for sample in 0..sample_count {
            for channel in 0..channel_count {
                let signed = i32::try_from(channel * channel_multiplier + sample % sample_modulus)
                    .unwrap()
                    + signed_offset;
                let packed = signed & 0x00ff_ffff;
                data.extend_from_slice(&packed.to_le_bytes()[..3]);
            }
        }
        let mut fmt = Vec::with_capacity(16);
        put_u16(&mut fmt, 1);
        put_u16(&mut fmt, u16::try_from(channel_count).unwrap());
        put_u32(&mut fmt, sample_rate);
        put_u32(&mut fmt, byte_rate);
        put_u16(&mut fmt, block_align);
        put_u16(&mut fmt, bits_per_sample);

        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        let riff_size = 4 + 8 + fmt.len() + 8 + data.len();
        put_u32(&mut bytes, u32::try_from(riff_size).unwrap());
        bytes.extend_from_slice(b"WAVEfmt ");
        put_u32(&mut bytes, u32::try_from(fmt.len()).unwrap());
        bytes.extend_from_slice(&fmt);
        bytes.extend_from_slice(b"data");
        put_u32(&mut bytes, u32::try_from(data.len()).unwrap());
        bytes.extend_from_slice(&data);
        bytes
    }

    #[test]
    fn in_memory_pcm24_source_produces_normal_padding_and_three_zero_drains() {
        let recipe = input_recipe();
        let source_sample_count = recipe["source_samples"].as_u64().unwrap();
        let normal_callbacks = recipe["normal_callbacks"].as_u64().unwrap();
        let total_callbacks = normal_callbacks + recipe["zero_input_drains"].as_u64().unwrap();
        let source_bytes = in_memory_pcm24_wave(&recipe);
        let parsed = parse_selected_riff_pcm24(&source_bytes, None).unwrap();
        let selected_source_samples = MeterSourceSampleCount::new(source_sample_count).unwrap();
        let selected_source_identity =
            MeterPcmSourceIdentity::from_selected_resource_sha256(parsed.source_sha256());
        let mut state =
            MeterPcmSourceCursor::cold(&parsed, selected_source_identity, selected_source_samples)
                .unwrap();
        assert_eq!(state.source_identity().sha256(), parsed.source_sha256());
        assert_eq!(state.read_index().value(), 0);

        for callback in 0..total_callbacks {
            let cursor_before = callback * METER_FRAME_SAMPLES_U64;
            let expected_read = if callback < normal_callbacks {
                (source_sample_count - cursor_before).min(METER_FRAME_SAMPLES_U64)
            } else {
                0
            };
            let proposal =
                stage_meter_pcm_source_frame(&state, MeterCallbackIndex::new(callback), &parsed)
                    .unwrap();
            let frame = proposal.frame();
            let expected_read_index_before = callback.min(normal_callbacks);
            let expected_read_index_after = if callback < normal_callbacks {
                callback + 1
            } else {
                normal_callbacks
            };
            let expected_phase = if callback < normal_callbacks {
                MeterCallbackPhase::NormalSource
            } else {
                MeterCallbackPhase::ZeroInputDrain
            };
            assert_eq!(frame.phase(), expected_phase);
            assert_eq!(frame.host_input_count().value(), expected_read);
            assert_eq!(
                frame.source_padding_count().value(),
                METER_FRAME_SAMPLES_U64 - expected_read
            );
            assert_eq!(frame.upstream_generation().value(), callback + 1);
            assert_eq!(
                frame.source_read_index_before().value(),
                expected_read_index_before
            );
            assert_eq!(
                frame.source_read_index_after().value(),
                expected_read_index_after
            );
            assert_eq!(
                proposal.after().read_index().value(),
                expected_read_index_after
            );
            assert_eq!(
                frame.source_cursor_before().value(),
                cursor_before.min(source_sample_count)
            );
            let cursor_after = if callback < normal_callbacks {
                (cursor_before + expected_read).min(source_sample_count)
            } else {
                source_sample_count
            };
            assert_eq!(frame.source_cursor_after().value(), cursor_after);
            assert!(
                frame
                    .rows()
                    .iter()
                    .all(|row| row.len() == METER_SOURCE_FRAME_SAMPLES)
            );
            if expected_read != 0 {
                let expected_rows = parsed
                    .read_pcm24_hop(AdmSampleIndex::new(cursor_before), expected_read)
                    .unwrap();
                let actual_count = usize::try_from(expected_read).unwrap();
                for (expected, actual) in expected_rows.iter().zip(frame.rows()) {
                    assert_eq!(&actual[..actual_count], expected);
                    assert!(actual[actual_count..].iter().all(|word| *word == 0));
                }
            } else {
                assert!(frame.rows().iter().flatten().all(|word| *word == 0));
            }
            state = proposal.into_parts().0;
        }
        assert_eq!(state.callback_index().value(), total_callbacks);
        assert_eq!(state.source_cursor().value(), source_sample_count);
        assert_eq!(state.read_index().value(), normal_callbacks);
    }

    #[test]
    fn dropping_normal_source_proposal_keeps_committed_read_index_cold() {
        let recipe = input_recipe();
        let source_bytes = in_memory_pcm24_wave(&recipe);
        let parsed = parse_selected_riff_pcm24(&source_bytes, None).unwrap();
        let source_samples = MeterSourceSampleCount::new(parsed.total_samples()).unwrap();
        let source_identity =
            MeterPcmSourceIdentity::from_selected_resource_sha256(parsed.source_sha256());
        let state = MeterPcmSourceCursor::cold(&parsed, source_identity, source_samples).unwrap();

        assert_eq!(state.read_index().value(), 0);
        let proposal =
            stage_meter_pcm_source_frame(&state, state.callback_index(), &parsed).unwrap();
        assert_eq!(proposal.frame().source_read_index_before().value(), 0);
        assert_eq!(proposal.frame().source_read_index_after().value(), 1);
        drop(proposal);

        assert_eq!(state.callback_index().value(), 0);
        assert_eq!(state.source_cursor().value(), 0);
        assert_eq!(state.read_index().value(), 0);
    }

    #[test]
    fn wrong_profile_and_clock_do_not_advance_the_source_cursor() {
        let recipe = input_recipe();
        let source_bytes = in_memory_pcm24_wave(&recipe);
        let parsed = parse_selected_riff_pcm24(&source_bytes, None).unwrap();
        let source_samples = MeterSourceSampleCount::new(parsed.total_samples()).unwrap();
        let source_identity =
            MeterPcmSourceIdentity::from_selected_resource_sha256(parsed.source_sha256());
        let state = MeterPcmSourceCursor::cold(&parsed, source_identity, source_samples).unwrap();
        assert!(matches!(
            MeterPcmSourceCursor::cold(
                &parsed,
                MeterPcmSourceIdentity::from_selected_resource_sha256([0; 32]),
                source_samples,
            ),
            Err(MeterSourceCursorError::SourceIdentityMismatch)
        ));
        assert!(matches!(
            MeterPcmSourceCursor::cold(
                &parsed,
                source_identity,
                MeterSourceSampleCount::new(source_samples.value() - 1).unwrap(),
            ),
            Err(MeterSourceCursorError::SourceExtentMismatch)
        ));
        assert_eq!(
            MeterSourceCursorError::SourceIdentityMismatch.code(),
            "METER_SOURCE_IDENTITY_MISMATCH"
        );
        assert_eq!(
            stage_meter_pcm_source_frame(&state, MeterCallbackIndex::new(1), &parsed,).err(),
            Some(MeterSourceCursorError::CallbackOrder)
        );
        assert_eq!(state.callback_index().value(), 0);
        let mut other_bytes = source_bytes.clone();
        let data_offset = usize::try_from(parsed.data_offset()).unwrap();
        other_bytes[data_offset] ^= 1;
        let other_source = parse_selected_riff_pcm24(&other_bytes, None).unwrap();
        assert_eq!(
            stage_meter_pcm_source_frame(&state, MeterCallbackIndex::new(0), &other_source).err(),
            Some(MeterSourceCursorError::SourceIdentityMismatch)
        );
        assert_eq!(state.callback_index().value(), 0);
    }

    #[test]
    fn approved_1050_sample_source_tail_is_486_positive_zero_words_in_all_38_rows() {
        let mut recipe = input_recipe();
        recipe["source_samples"] = Value::from(1_050_u64);
        let source_bytes = in_memory_pcm24_wave(&recipe);
        let parsed = parse_selected_riff_pcm24(&source_bytes, None).unwrap();
        let source_samples = MeterSourceSampleCount::new(1_050).unwrap();
        let source_identity =
            MeterPcmSourceIdentity::from_selected_resource_sha256(parsed.source_sha256());
        let state = MeterPcmSourceCursor::cold(&parsed, source_identity, source_samples).unwrap();

        let frame = stage_meter_pcm_source_frame(&state, MeterCallbackIndex::new(0), &parsed)
            .unwrap()
            .frame()
            .clone();
        assert_eq!(frame.host_input_count().value(), 1_050);
        assert_eq!(frame.source_read_index_before().value(), 0);
        assert_eq!(frame.source_read_index_after().value(), 1);
        assert_eq!(frame.source_padding_count().value(), 486);
        assert_eq!(frame.rows().len(), 38);
        let expected_rows = parsed
            .read_pcm24_hop(AdmSampleIndex::new(0), 1_050)
            .unwrap();
        for (row, expected) in frame.rows().iter().zip(expected_rows) {
            assert_eq!(row.len(), 1_536);
            assert_eq!(&row[..1_050], expected);
            assert!(row[1_050..].iter().all(|word| *word == 0));
        }
    }
}
