// pattern: Functional Core
//! Selected object-render callback state and noncommitting callback composition.

use std::num::NonZeroU64;

use super::{
    object_gain_mixer::{GainRow, ObjectGainMixError, PerTaskOutputFilter, stage_object_gain_mix},
    object_render_metadata::{
        CanonicalRenderRecord, ControlPacket, MetadataAdmission, ObjectRenderValueError,
        SecondaryGeometry, canonical_record_from_source, direct_selector_from_coded_mask,
        metadata_from_control, point_render_target, source_record_from_metadata,
    },
};

const SOURCE_COUNT: usize = 16;
const TASK_COUNT: usize = 8;
const SOURCES_PER_TASK: usize = 2;
const OUTPUT_CHANNELS: usize = 6;
const CALLBACK_SAMPLES: usize = 1_536;
const BLOCK_SAMPLES: usize = 32;
const BLOCKS_PER_CALLBACK: usize = CALLBACK_SAMPLES / BLOCK_SAMPLES;
const CONTROL_WORDS: usize = 13;
const FRAME_SAMPLE_COUNT: u64 = 1_536;
const DIRECT_SOURCE_COUNT: u32 = 1;
const SELECTED_CODED_MASK: u64 = 8;
const INTERPOLATION_BLOCKS: u32 = 48;
const STATE_HEADER: &[u8; 8] = b"OMCLB001";
const COLD_STATE_BYTES: usize = 1_569;
#[cfg(test)]
const WARM_STATE_BYTES: usize = 2_529;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RenderSourceSampleCount(u64);

impl RenderSourceSampleCount {
    pub fn try_new(value: u64) -> Result<Self, ObjectRendererError> {
        if value == 0 || value > i64::MAX as u64 {
            return Err(ObjectRendererError::InputError);
        }
        Ok(Self(value))
    }

    pub const fn value(self) -> u64 {
        self.0
    }

    fn normal_callbacks(self) -> Result<u64, ObjectRendererError> {
        self.0
            .checked_add(u64::try_from(CALLBACK_SAMPLES - 1).expect("callback extent fits u64"))
            .map(|value| value / u64::try_from(CALLBACK_SAMPLES).expect("callback extent fits u64"))
            .ok_or(ObjectRendererError::StateInvalid)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ObjectRenderCallbackIndex(u64);

impl ObjectRenderCallbackIndex {
    const COLD: Self = Self(0);

    pub const fn value(self) -> u64 {
        self.0
    }

    fn checked_next(self) -> Result<Self, ObjectRendererError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(ObjectRendererError::CallbackOrder)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ObjectRenderCallbackGeneration(NonZeroU64);

impl ObjectRenderCallbackGeneration {
    pub fn try_new(value: u64) -> Result<Self, ObjectRendererError> {
        NonZeroU64::new(value)
            .map(Self)
            .ok_or(ObjectRendererError::InputError)
    }

    pub const fn value(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectRenderCallbackPhase {
    Normal,
    Drain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectRendererError {
    InputError,
    StateInvalid,
    PipelineRetired,
    CallbackOrder,
    ProfileUnsupported,
    NumericFailure,
}

impl ObjectRendererError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputError => "OAR_INPUT_ERROR",
            Self::StateInvalid => "OAR_STATE_ERROR",
            Self::PipelineRetired => "OAR_PIPELINE_RETIRED",
            Self::CallbackOrder => "OAR_CALLBACK_ORDER",
            Self::ProfileUnsupported => "OAR_PROFILE_UNSUPPORTED",
            Self::NumericFailure => "OAR_NUMERIC_FAILURE",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObjectRenderTaskState {
    rows: [GainRow; SOURCES_PER_TASK],
    records: Option<[CanonicalRenderRecord; SOURCES_PER_TASK]>,
}

impl ObjectRenderTaskState {
    const fn cold() -> Self {
        Self {
            rows: [GainRow::cold(); SOURCES_PER_TASK],
            records: None,
        }
    }

    pub const fn rows(&self) -> &[GainRow; SOURCES_PER_TASK] {
        &self.rows
    }

    pub const fn records(&self) -> Option<&[CanonicalRenderRecord; SOURCES_PER_TASK]> {
        self.records.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObjectRendererState {
    source_samples: RenderSourceSampleCount,
    callback_index: ObjectRenderCallbackIndex,
    retired: bool,
    tasks: [ObjectRenderTaskState; TASK_COUNT],
}

impl ObjectRendererState {
    pub fn cold(source_samples: RenderSourceSampleCount) -> Self {
        Self {
            source_samples,
            callback_index: ObjectRenderCallbackIndex::COLD,
            retired: false,
            tasks: std::array::from_fn(|_| ObjectRenderTaskState::cold()),
        }
    }

    pub const fn source_samples(&self) -> RenderSourceSampleCount {
        self.source_samples
    }

    pub const fn callback_index(&self) -> ObjectRenderCallbackIndex {
        self.callback_index
    }

    pub const fn is_retired(&self) -> bool {
        self.retired
    }

    pub const fn tasks(&self) -> &[ObjectRenderTaskState; TASK_COUNT] {
        &self.tasks
    }

    pub fn retired_copy(&self) -> Result<Self, ObjectRendererError> {
        validate_state(self)?;
        let mut retired = self.clone();
        retired.retired = true;
        Ok(retired)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ObjectRendererError> {
        validate_state(self)?;
        let cache_count = self
            .tasks
            .iter()
            .filter(|task| task.records.is_some())
            .count();
        let mut bytes = Vec::with_capacity(COLD_STATE_BYTES + cache_count * 120);
        bytes.extend_from_slice(STATE_HEADER);
        bytes.extend_from_slice(&self.source_samples.value().to_le_bytes());
        bytes.extend_from_slice(&self.callback_index.value().to_le_bytes());
        bytes.push(u8::from(self.retired));
        for task in &self.tasks {
            for row in &task.rows {
                for words in [row.current(), row.target(), row.step()] {
                    for value in words {
                        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
                    }
                }
                for remaining in row.remaining() {
                    bytes.extend_from_slice(&remaining.to_le_bytes());
                }
            }
            match &task.records {
                None => bytes.push(0),
                Some(records) => {
                    bytes.push(1);
                    for record in records {
                        bytes.extend_from_slice(&record.to_le_bytes());
                    }
                }
            }
        }
        debug_assert_eq!(bytes.len(), COLD_STATE_BYTES + cache_count * 120);
        Ok(bytes)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectRenderCallbackPacket {
    generation: ObjectRenderCallbackGeneration,
    phase: ObjectRenderCallbackPhase,
    controls: Vec<Vec<u32>>,
    pcm_words: Vec<Vec<u32>>,
}

impl ObjectRenderCallbackPacket {
    pub fn new(
        generation: ObjectRenderCallbackGeneration,
        phase: ObjectRenderCallbackPhase,
        controls: Vec<Vec<u32>>,
        pcm_words: Vec<Vec<u32>>,
    ) -> Self {
        Self {
            generation,
            phase,
            controls,
            pcm_words,
        }
    }

    pub const fn generation(&self) -> ObjectRenderCallbackGeneration {
        self.generation
    }

    pub const fn phase(&self) -> ObjectRenderCallbackPhase {
        self.phase
    }

    #[cfg(test)]
    pub(crate) fn pcm_words(&self) -> &[Vec<u32>] {
        &self.pcm_words
    }

    #[cfg(test)]
    pub(crate) fn controls(&self) -> &[Vec<u32>] {
        &self.controls
    }
}

#[must_use]
pub struct ObjectRenderCallbackProposal {
    after: ObjectRendererState,
    output_words: [Vec<u32>; OUTPUT_CHANNELS],
    task_changed: [bool; TASK_COUNT],
    task_update_block: [Option<u8>; TASK_COUNT],
}

impl ObjectRenderCallbackProposal {
    pub const fn after(&self) -> &ObjectRendererState {
        &self.after
    }

    pub const fn output_words(&self) -> &[Vec<u32>; OUTPUT_CHANNELS] {
        &self.output_words
    }

    pub const fn task_changed(&self) -> &[bool; TASK_COUNT] {
        &self.task_changed
    }

    pub const fn task_update_block(&self) -> &[Option<u8>; TASK_COUNT] {
        &self.task_update_block
    }

    pub fn into_parts(self) -> (ObjectRendererState, [Vec<u32>; OUTPUT_CHANNELS]) {
        (self.after, self.output_words)
    }
}

pub fn stage_object_render_callback(
    before: &ObjectRendererState,
    packet: &ObjectRenderCallbackPacket,
) -> Result<ObjectRenderCallbackProposal, ObjectRendererError> {
    stage_object_render_callback_with_geometry(before, packet, &SecondaryGeometry::selected())
}

/// Stages one callback using the owner-validated OAR secondary geometry.
pub fn stage_object_render_callback_with_geometry(
    before: &ObjectRendererState,
    packet: &ObjectRenderCallbackPacket,
    geometry: &SecondaryGeometry,
) -> Result<ObjectRenderCallbackProposal, ObjectRendererError> {
    validate_state(before)?;
    if before.retired {
        return Err(ObjectRendererError::PipelineRetired);
    }
    validate_callback_clock(before, packet)?;
    validate_pcm(&packet.pcm_words)?;
    let metadata = validate_and_project_metadata(&packet.controls)?;

    let direct_selector =
        direct_selector_from_coded_mask(SELECTED_CODED_MASK).map_err(map_value_error)?;
    let mut records = Vec::with_capacity(SOURCE_COUNT);
    let mut targets = Vec::with_capacity(SOURCE_COUNT);
    for (source_index, source_metadata) in metadata.into_iter().enumerate() {
        let source = source_record_from_metadata(source_metadata).map_err(map_value_error)?;
        let record = canonical_record_from_source(
            &source,
            (source_index == 0).then_some(direct_selector),
            true,
        )
        .map_err(map_value_error)?;
        let target = point_render_target(&record, geometry).map_err(map_value_error)?;
        records.push(record);
        targets.push(target.values());
    }

    let mut after = before.clone();
    let mut output: [Vec<f32>; OUTPUT_CHANNELS] =
        std::array::from_fn(|_| vec![0.0_f32; CALLBACK_SAMPLES]);
    let mut task_changed = [false; TASK_COUNT];
    let mut task_update_block = [None; TASK_COUNT];
    for task_index in 0..TASK_COUNT {
        let source_begin = task_index * SOURCES_PER_TASK;
        let task_records = [
            records[source_begin].clone(),
            records[source_begin + 1].clone(),
        ];
        let changed = after.tasks[task_index].records.as_ref() != Some(&task_records);
        task_changed[task_index] = changed;
        task_update_block[task_index] = changed.then_some(0);

        let mut rows = after.tasks[task_index].rows;
        for local_source in 0..SOURCES_PER_TASK {
            let target = targets[source_begin + local_source].map(f64::from);
            rows[local_source] = rows[local_source]
                .with_target(&target)
                .map_err(|error| map_mix_input_error(&error))?;
        }

        let mut task_output: [Vec<f32>; OUTPUT_CHANNELS] =
            std::array::from_fn(|_| vec![0.0_f32; CALLBACK_SAMPLES]);
        for block in 0..BLOCKS_PER_CALLBACK {
            let block_begin = block * BLOCK_SAMPLES;
            let source_blocks: [[f32; BLOCK_SAMPLES]; SOURCES_PER_TASK] =
                std::array::from_fn(|local_source| {
                    std::array::from_fn(|sample| {
                        f32::from_bits(
                            packet.pcm_words[source_begin + local_source][block_begin + sample],
                        )
                    })
                });
            let staged = stage_object_gain_mix(
                &rows,
                &source_blocks,
                u8::from(changed && block == 0),
                INTERPOLATION_BLOCKS,
                PerTaskOutputFilter::Disabled,
            )
            .map_err(|error| map_mix_input_error(&error))?;
            let (block_output, rows_after, _) = staged.into_parts();
            for channel in 0..OUTPUT_CHANNELS {
                for sample in 0..BLOCK_SAMPLES {
                    let value = block_output[channel][sample];
                    if !value.is_finite() {
                        return Err(ObjectRendererError::NumericFailure);
                    }
                    task_output[channel][block_begin + sample] = value;
                }
            }
            rows = rows_after
                .try_into()
                .map_err(|_| ObjectRendererError::StateInvalid)?;
            validate_gain_rows(&rows).map_err(|_| ObjectRendererError::NumericFailure)?;
        }

        after.tasks[task_index].rows = rows;
        after.tasks[task_index].records = Some(task_records);
        for channel in 0..OUTPUT_CHANNELS {
            for sample in 0..CALLBACK_SAMPLES {
                let sum = add_r32(output[channel][sample], task_output[channel][sample]);
                if !sum.is_finite() {
                    return Err(ObjectRendererError::NumericFailure);
                }
                output[channel][sample] = sum;
            }
        }
    }

    after.callback_index = before.callback_index.checked_next()?;
    let output_words = output.map(|channel| channel.into_iter().map(f32::to_bits).collect());
    Ok(ObjectRenderCallbackProposal {
        after,
        output_words,
        task_changed,
        task_update_block,
    })
}

fn validate_callback_clock(
    state: &ObjectRendererState,
    packet: &ObjectRenderCallbackPacket,
) -> Result<(), ObjectRendererError> {
    let normal_callbacks = state.source_samples.normal_callbacks()?;
    let total_callbacks = normal_callbacks
        .checked_add(3)
        .ok_or(ObjectRendererError::StateInvalid)?;
    let callback_index = state.callback_index.value();
    if callback_index >= total_callbacks {
        return Err(ObjectRendererError::CallbackOrder);
    }
    let expected_generation = callback_index
        .checked_add(1)
        .ok_or(ObjectRendererError::CallbackOrder)?;
    let expected_phase = if callback_index < normal_callbacks {
        ObjectRenderCallbackPhase::Normal
    } else {
        ObjectRenderCallbackPhase::Drain
    };
    if packet.generation.value() != expected_generation || packet.phase != expected_phase {
        return Err(ObjectRendererError::CallbackOrder);
    }
    Ok(())
}

fn validate_pcm(pcm_words: &[Vec<u32>]) -> Result<(), ObjectRendererError> {
    if pcm_words.len() != SOURCE_COUNT || pcm_words.iter().any(|row| row.len() != CALLBACK_SAMPLES)
    {
        return Err(ObjectRendererError::InputError);
    }
    if pcm_words
        .iter()
        .flatten()
        .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(ObjectRendererError::InputError);
    }
    Ok(())
}

fn validate_and_project_metadata(
    controls: &[Vec<u32>],
) -> Result<Vec<super::object_render_metadata::SourceMetadata>, ObjectRendererError> {
    if controls.len() != SOURCE_COUNT || controls.iter().any(|row| row.len() != CONTROL_WORDS) {
        return Err(ObjectRendererError::InputError);
    }
    let mut projected = Vec::with_capacity(SOURCE_COUNT);
    for (source_index, words) in controls.iter().enumerate() {
        let mut bytes = [0_u8; CONTROL_WORDS * 4];
        for (slot, word) in words.iter().copied().enumerate() {
            bytes[slot * 4..slot * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        let control = ControlPacket::from_bytes(&bytes).map_err(map_value_error)?;
        let admission = MetadataAdmission::try_new(
            u32::try_from(source_index).map_err(|_| ObjectRendererError::InputError)?,
            DIRECT_SOURCE_COUNT,
            FRAME_SAMPLE_COUNT,
        )
        .map_err(map_value_error)?;
        projected.push(metadata_from_control(&control, admission).map_err(map_value_error)?);
    }
    Ok(projected)
}

fn validate_state(state: &ObjectRendererState) -> Result<(), ObjectRendererError> {
    let total_callbacks = state
        .source_samples
        .normal_callbacks()?
        .checked_add(3)
        .ok_or(ObjectRendererError::StateInvalid)?;
    if state.callback_index.value() > total_callbacks {
        return Err(ObjectRendererError::StateInvalid);
    }
    for task in &state.tasks {
        validate_gain_rows(&task.rows)?;
    }
    Ok(())
}

fn validate_gain_rows(rows: &[GainRow]) -> Result<(), ObjectRendererError> {
    if rows.iter().any(|row| {
        row.current()
            .iter()
            .chain(row.target())
            .chain(row.step())
            .any(|value| !value.is_finite())
    }) {
        return Err(ObjectRendererError::StateInvalid);
    }
    Ok(())
}

fn map_value_error(error: ObjectRenderValueError) -> ObjectRendererError {
    match error {
        ObjectRenderValueError::InvalidControlLength { .. }
        | ObjectRenderValueError::FrameSampleCountOutOfRange
        | ObjectRenderValueError::NonFiniteControlValue { .. }
        | ObjectRenderValueError::NonFiniteMetadataValue { .. }
        | ObjectRenderValueError::GainOutOfRange
        | ObjectRenderValueError::InvalidSourceLength { .. } => ObjectRendererError::InputError,
        ObjectRenderValueError::MissingDirectSelector
        | ObjectRenderValueError::ExtensionPresenceUnbound
        | ObjectRenderValueError::UnboundSpread
        | ObjectRenderValueError::InvalidCodedMask
        | ObjectRenderValueError::UnsupportedRoutingProfile
        | ObjectRenderValueError::InvalidTaskIndex
        | ObjectRenderValueError::RoutingClassificationMismatch
        | ObjectRenderValueError::UnsupportedGeometry
        | ObjectRenderValueError::UnsupportedRecordType
        | ObjectRenderValueError::UnsupportedDirectSelector
        | ObjectRenderValueError::UnsupportedComponent
        | ObjectRenderValueError::UnsupportedObjectMode
        | ObjectRenderValueError::UnsupportedSourceOffset
        | ObjectRenderValueError::RecordExtensionPresent
        | ObjectRenderValueError::HorizontalCoordinateOutOfRange
        | ObjectRenderValueError::VerticalCoordinateOutOfRange => {
            ObjectRendererError::ProfileUnsupported
        }
    }
}

fn map_mix_input_error(error: &ObjectGainMixError) -> ObjectRendererError {
    match error {
        ObjectGainMixError::NonFiniteGainRowValue { .. }
        | ObjectGainMixError::GainRowValueOutsideF32 { .. }
        | ObjectGainMixError::RemainingOutsideU32 { .. } => ObjectRendererError::StateInvalid,
        ObjectGainMixError::WrongGainRowFieldLength { .. }
        | ObjectGainMixError::TooManyGainRows { .. }
        | ObjectGainMixError::SourceCountMismatch { .. }
        | ObjectGainMixError::NonFiniteSourceSample { .. }
        | ObjectGainMixError::InvalidUpdateFlag { .. }
        | ObjectGainMixError::InterpolationBlocksOutOfRange { .. } => {
            ObjectRendererError::InputError
        }
        ObjectGainMixError::UnsupportedPerTaskOutputFilter => {
            ObjectRendererError::ProfileUnsupported
        }
    }
}

#[inline(never)]
fn add_r32(left: f32, right: f32) -> f32 {
    left + right
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::*;

    fn digest(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    fn output_digest(output: &[Vec<u32>; OUTPUT_CHANNELS]) -> String {
        let mut bytes = Vec::with_capacity(OUTPUT_CHANNELS * CALLBACK_SAMPLES * 4);
        for word in output.iter().flatten() {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        digest(&bytes)
    }

    fn stable_controls(direct_enumeration: u32) -> Vec<Vec<u32>> {
        vec![
            vec![
                1,
                0,
                direct_enumeration,
                2,
                0,
                2_143_289_344,
                0,
                0,
                0,
                0,
                0,
                0,
                1,
            ],
            vec![0, 0, 3, 2, 0, 1_056_964_608, 0, 0, 0, 0, 0, 0, 1],
            vec![0, 0, 3, 2, 0, 1_065_353_216, 0, 0, 0, 0, 0, 0, 1],
            vec![0, 0, 3, 2, 0, 0, 1_056_964_608, 0, 0, 0, 0, 0, 1],
            vec![
                0,
                0,
                3,
                2,
                0,
                1_056_964_608,
                1_056_964_608,
                0,
                0,
                0,
                0,
                0,
                1,
            ],
            vec![
                0,
                0,
                3,
                2,
                0,
                1_065_353_216,
                1_056_964_608,
                0,
                0,
                0,
                0,
                0,
                1,
            ],
            vec![0, 0, 3, 2, 0, 0, 1_065_353_216, 0, 0, 0, 0, 0, 1],
            vec![
                0,
                0,
                3,
                2,
                0,
                1_056_964_608,
                1_065_353_216,
                0,
                0,
                0,
                0,
                0,
                1,
            ],
            vec![
                0,
                0,
                3,
                2,
                0,
                1_065_353_216,
                1_065_353_216,
                0,
                0,
                0,
                0,
                0,
                1,
            ],
            vec![0, 0, 3, 2, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![0, 0, 3, 2, 0, 1_056_964_608, 0, 0, 0, 0, 0, 0, 1],
            vec![0, 0, 3, 2, 0, 1_065_353_216, 0, 0, 0, 0, 0, 0, 1],
            vec![0, 0, 3, 2, 0, 0, 1_056_964_608, 0, 0, 0, 0, 0, 1],
            vec![
                0,
                0,
                3,
                2,
                0,
                1_056_964_608,
                1_056_964_608,
                0,
                0,
                0,
                0,
                0,
                1,
            ],
            vec![
                0,
                0,
                3,
                2,
                0,
                1_065_353_216,
                1_056_964_608,
                0,
                0,
                0,
                0,
                0,
                1,
            ],
            vec![0, 0, 3, 2, 0, 0, 1_065_353_216, 0, 0, 0, 0, 0, 1],
        ]
    }

    fn affine_pcm(ordinal: u32) -> Vec<Vec<u32>> {
        (0..SOURCE_COUNT)
            .map(|source| {
                (0..CALLBACK_SAMPLES)
                    .map(|sample| {
                        let centered = i32::try_from(sample % BLOCK_SAMPLES).unwrap() - 16;
                        let scaled = (centered as f32 * (source + 1) as f32) / 32_768.0_f32;
                        let ordinal_bias =
                            (i32::try_from(ordinal % 3).unwrap() - 1) as f32 / 1_024.0_f32;
                        (scaled + ordinal_bias).to_bits()
                    })
                    .collect()
            })
            .collect()
    }

    fn impulse_pcm(ordinal: u32) -> Vec<Vec<u32>> {
        (0..SOURCE_COUNT)
            .map(|source| {
                let impulse_sample = (97 * source
                    + 13 * usize::try_from(ordinal).expect("test ordinal fits usize"))
                    % CALLBACK_SAMPLES;
                (0..CALLBACK_SAMPLES)
                    .map(|sample| {
                        if sample == impulse_sample {
                            ((source + 1) as f32 / 32.0_f32).to_bits()
                        } else {
                            0
                        }
                    })
                    .collect()
            })
            .collect()
    }

    fn packet(
        generation: u64,
        phase: ObjectRenderCallbackPhase,
        controls: Vec<Vec<u32>>,
        pcm_words: Vec<Vec<u32>>,
    ) -> ObjectRenderCallbackPacket {
        ObjectRenderCallbackPacket::new(
            ObjectRenderCallbackGeneration::try_new(generation).unwrap(),
            phase,
            controls,
            pcm_words,
        )
    }

    #[test]
    fn cold_state_matches_the_approved_complete_encoding() {
        let state = ObjectRendererState::cold(RenderSourceSampleCount::try_new(4_625).unwrap());
        let bytes = state.canonical_bytes().unwrap();
        assert_eq!(bytes.len(), COLD_STATE_BYTES);
        assert_eq!(
            digest(&bytes),
            "49dc6b1c902471e1c1ce469268f5d6bfc0a84910d2c9ef2c7aa3cb8f28c07522"
        );
        assert!(state.tasks().iter().all(|task| task.records().is_none()));
    }

    #[test]
    fn validated_geometry_entrypoint_matches_selected_renderer_entrypoint() {
        let before = ObjectRendererState::cold(RenderSourceSampleCount::try_new(4_625).unwrap());
        let callback = packet(
            1,
            ObjectRenderCallbackPhase::Normal,
            stable_controls(1),
            affine_pcm(1),
        );
        let geometry = SecondaryGeometry::try_from_rows(*SecondaryGeometry::selected().rows())
            .expect("the selected provider rows satisfy the renderer geometry contract");

        let selected = stage_object_render_callback(&before, &callback).unwrap();
        let supplied =
            stage_object_render_callback_with_geometry(&before, &callback, &geometry).unwrap();

        assert_eq!(supplied.output_words(), selected.output_words());
        assert_eq!(supplied.after(), selected.after());
    }

    #[test]
    fn stable_first_two_callbacks_match_output_state_and_record_clock() {
        let before = ObjectRendererState::cold(RenderSourceSampleCount::try_new(4_625).unwrap());
        let before_copy = before.clone();
        let first_packet = packet(
            1,
            ObjectRenderCallbackPhase::Normal,
            stable_controls(1),
            affine_pcm(1),
        );
        let first_packet_copy = first_packet.clone();
        let first = stage_object_render_callback(&before, &first_packet).unwrap();
        assert_eq!(before, before_copy);
        assert_eq!(first_packet, first_packet_copy);
        assert_eq!(
            first.after().canonical_bytes().unwrap().len(),
            WARM_STATE_BYTES
        );
        assert_eq!(
            output_digest(first.output_words()),
            "2a736fefe10b1360ef302085fdd8f9418435bd58365939a557d2974a59ef0722"
        );
        assert_eq!(
            digest(&first.after().canonical_bytes().unwrap()),
            "87debe8a097276738cab4eb1b54957e99963a8077b15fb4a33ca235be8433c07"
        );
        assert!(first.task_changed().iter().all(|changed| *changed));
        assert!(
            first
                .task_update_block()
                .iter()
                .all(|block| *block == Some(0))
        );

        let second = stage_object_render_callback(
            first.after(),
            &packet(
                2,
                ObjectRenderCallbackPhase::Normal,
                stable_controls(2),
                affine_pcm(2),
            ),
        )
        .unwrap();
        assert_eq!(
            output_digest(second.output_words()),
            "cab3ef76532a447bbc5ee6f75d95f16ecd9c5dbca61ccd8fadc28a903ba7ec8a"
        );
        assert_eq!(
            digest(&second.after().canonical_bytes().unwrap()),
            "e074c6816c3ab70caf007414bb3fbd59309ea654cfc2281caeca8f127576e8f5"
        );
        assert!(second.task_changed().iter().all(|changed| !*changed));
        assert!(second.task_update_block().iter().all(Option::is_none));
    }

    #[test]
    fn stable_sequence_runs_four_normal_and_exactly_three_drain_callbacks() {
        let mut state = ObjectRendererState::cold(RenderSourceSampleCount::try_new(4_625).unwrap());
        let steps = [
            (
                ObjectRenderCallbackPhase::Normal,
                affine_pcm(1),
                "2a736fefe10b1360ef302085fdd8f9418435bd58365939a557d2974a59ef0722",
                "87debe8a097276738cab4eb1b54957e99963a8077b15fb4a33ca235be8433c07",
            ),
            (
                ObjectRenderCallbackPhase::Normal,
                affine_pcm(2),
                "cab3ef76532a447bbc5ee6f75d95f16ecd9c5dbca61ccd8fadc28a903ba7ec8a",
                "e074c6816c3ab70caf007414bb3fbd59309ea654cfc2281caeca8f127576e8f5",
            ),
            (
                ObjectRenderCallbackPhase::Normal,
                impulse_pcm(3),
                "2e3a6cfc455c64ebee9f6c4111e6b4d303d79b2fc3133b97058cce9e2a1a9cfb",
                "f6b66cb28b95bfb41fcb65617c71c523ca2a3ca498165204f8fd5ea0f7b61f81",
            ),
            (
                ObjectRenderCallbackPhase::Normal,
                affine_pcm(4),
                "72067fd730452155f106de7a22ec7177f247bb9e850754bf898bfade8ebefa9c",
                "c79a181e841bef18e4da902f5a3ddebfdee8c17bec89533f0bec19fa4462cac9",
            ),
            (
                ObjectRenderCallbackPhase::Drain,
                affine_pcm(5),
                "cab3ef76532a447bbc5ee6f75d95f16ecd9c5dbca61ccd8fadc28a903ba7ec8a",
                "9760f7712678f1b2b001cf3a077ae93f5a26ed12d66aa5b5a8477960e15c0c4d",
            ),
            (
                ObjectRenderCallbackPhase::Drain,
                impulse_pcm(6),
                "4d738a25734fa3a952be8cec61cbef18db69b5b7d4e71dd54f7599d2828da940",
                "04738d307b95e8594ee0ad4cbe09dfda960c1f79d9bd173dd732ebcaa4b82508",
            ),
            (
                ObjectRenderCallbackPhase::Drain,
                affine_pcm(7),
                "72067fd730452155f106de7a22ec7177f247bb9e850754bf898bfade8ebefa9c",
                "b7e05744fc3ffd942c5ccc9aae10e8db6207e28cc43b047644b21e2fc4b23233",
            ),
        ];

        for (step, (phase, pcm_words, output_sha256, state_sha256)) in steps.into_iter().enumerate()
        {
            let generation = u64::try_from(step + 1).unwrap();
            let controls = stable_controls(if step == 0 { 1 } else { 2 });
            let proposal = stage_object_render_callback(
                &state,
                &packet(generation, phase, controls, pcm_words),
            )
            .unwrap();
            assert_eq!(output_digest(proposal.output_words()), output_sha256);
            assert_eq!(
                digest(&proposal.after().canonical_bytes().unwrap()),
                state_sha256
            );
            state = proposal.into_parts().0;
        }

        assert_eq!(state.callback_index().value(), 7);
        assert_eq!(
            stage_object_render_callback(
                &state,
                &packet(
                    8,
                    ObjectRenderCallbackPhase::Drain,
                    stable_controls(2),
                    affine_pcm(8),
                ),
            )
            .err(),
            Some(ObjectRendererError::CallbackOrder)
        );
    }

    #[test]
    fn clock_profile_numeric_and_precedence_fail_without_mutating_state() {
        let state = ObjectRendererState::cold(RenderSourceSampleCount::try_new(4_625).unwrap());
        let snapshot = state.clone();
        assert_eq!(
            stage_object_render_callback(
                &state,
                &packet(
                    2,
                    ObjectRenderCallbackPhase::Normal,
                    stable_controls(1),
                    affine_pcm(1),
                ),
            )
            .err(),
            Some(ObjectRendererError::CallbackOrder)
        );
        assert_eq!(
            stage_object_render_callback(
                &state,
                &packet(
                    1,
                    ObjectRenderCallbackPhase::Drain,
                    stable_controls(1),
                    affine_pcm(1),
                ),
            )
            .err(),
            Some(ObjectRendererError::CallbackOrder)
        );

        let mut unsupported = stable_controls(1);
        unsupported[1][11] = 1;
        assert_eq!(
            stage_object_render_callback(
                &state,
                &packet(
                    1,
                    ObjectRenderCallbackPhase::Normal,
                    unsupported.clone(),
                    affine_pcm(1),
                ),
            )
            .err(),
            Some(ObjectRendererError::ProfileUnsupported)
        );
        unsupported[15][5] = f32::NAN.to_bits();
        assert_eq!(
            stage_object_render_callback(
                &state,
                &packet(
                    1,
                    ObjectRenderCallbackPhase::Normal,
                    unsupported,
                    affine_pcm(1),
                ),
            )
            .err(),
            Some(ObjectRendererError::InputError)
        );

        let retired = state.retired_copy().unwrap();
        assert_eq!(
            stage_object_render_callback(
                &retired,
                &packet(1, ObjectRenderCallbackPhase::Normal, Vec::new(), Vec::new(),),
            )
            .err(),
            Some(ObjectRendererError::PipelineRetired)
        );

        let huge = vec![vec![f32::MAX.to_bits(); CALLBACK_SAMPLES]; SOURCE_COUNT];
        assert_eq!(
            stage_object_render_callback(
                &state,
                &packet(
                    1,
                    ObjectRenderCallbackPhase::Normal,
                    stable_controls(1),
                    huge,
                ),
            )
            .err(),
            Some(ObjectRendererError::NumericFailure)
        );
        assert_eq!(state, snapshot);
    }
}
