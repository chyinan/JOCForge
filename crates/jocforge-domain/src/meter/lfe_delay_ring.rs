// pattern: Functional Core
//! Independent-meter direct-LFE delay ring and F06OF OAR packet assembly.

use super::{
    object_renderer::{
        ObjectRenderCallbackGeneration, ObjectRenderCallbackPacket, ObjectRenderCallbackPhase,
    },
    sample_block_queue::{
        MeterCallbackCount, MeterCallbackIndex, MeterCallbackPhase, MeterSampleCount,
        MeterSourceSampleCount,
    },
};
use crate::{
    joc::oamd::reader_target::{
        HistoryProducerGeneration, ReaderInvocationIndex, ReaderTargetControlLineage,
        ReaderTargetControlRows,
    },
    spatial::{spatial_audio_state::MeterComponentOutputRows, target_context::TargetSlotIndex},
    timeline::FrontendCallbackIndex,
    transaction::CallbackLineageIdentity,
};

const LFE_DELAY_SLOTS: u8 = 3;
const LFE_DELAY_SAMPLES: u64 = 3_072;
const CALLBACK_SAMPLES: usize = 1_536;
const CALLBACK_SAMPLES_U64: u64 = 1_536;
const CALLBACK_SAMPLES_MINUS_ONE_U64: u64 = 1_535;
const REGULAR_SOURCE_COUNT: usize = 15;
const TOTAL_SOURCE_COUNT: usize = REGULAR_SOURCE_COUNT + 1;
const CONTROL_WORD_COUNT: usize = 13;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterLfeError {
    InvalidSourceCount,
    ArithmeticOverflow,
    StateInvalid,
    CallbackOrder,
    GenerationMismatch,
    PhaseMismatch,
    InputShape,
    InputNonFinite,
    DrainInputNonZero,
}

impl MeterLfeError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidSourceCount => "METER_LFE_SOURCE_LENGTH_INVALID",
            Self::ArithmeticOverflow => "METER_LFE_ARITHMETIC_OVERFLOW",
            Self::StateInvalid => "METER_LFE_STATE_INVALID",
            Self::CallbackOrder => "METER_LFE_CALLBACK_ORDER",
            Self::GenerationMismatch => "METER_LFE_GENERATION_MISMATCH",
            Self::PhaseMismatch => "METER_LFE_PHASE_MISMATCH",
            Self::InputShape => "METER_LFE_INPUT_SHAPE",
            Self::InputNonFinite => "METER_LFE_INPUT_NONFINITE",
            Self::DrainInputNonZero => "METER_LFE_DRAIN_INPUT_NONZERO",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeterLfeCallbackOrigin(MeterCallbackIndex);

impl MeterLfeCallbackOrigin {
    pub const fn callback_index(self) -> MeterCallbackIndex {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DelayedLfeFrame {
    origin: MeterLfeCallbackOrigin,
    sample_words: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterLfeDelayState {
    source_samples: MeterSourceSampleCount,
    normal_callbacks: u64,
    callback_index: MeterCallbackIndex,
    samples_consumed: u64,
    write_index: u8,
    read_index: u8,
    slots: [Option<DelayedLfeFrame>; 3],
}

impl MeterLfeDelayState {
    pub fn cold(source_samples: MeterSourceSampleCount) -> Result<Self, MeterLfeError> {
        let source_length = source_samples.value();
        if source_length == 0 {
            return Err(MeterLfeError::InvalidSourceCount);
        }
        let normal_callbacks = source_length
            .checked_add(CALLBACK_SAMPLES_MINUS_ONE_U64)
            .ok_or(MeterLfeError::ArithmeticOverflow)?
            / CALLBACK_SAMPLES_U64;
        normal_callbacks
            .checked_add(3)
            .ok_or(MeterLfeError::ArithmeticOverflow)?;
        Ok(Self {
            source_samples,
            normal_callbacks,
            callback_index: MeterCallbackIndex::new(0),
            samples_consumed: 0,
            write_index: 0,
            read_index: 1,
            slots: std::array::from_fn(|_| None),
        })
    }

    pub fn reset(&mut self) -> Result<(), MeterLfeError> {
        *self = Self::cold(self.source_samples)?;
        Ok(())
    }

    pub const fn source_samples(&self) -> MeterSourceSampleCount {
        self.source_samples
    }

    pub const fn normal_callbacks(&self) -> MeterCallbackCount {
        MeterCallbackCount::new(self.normal_callbacks)
    }

    pub const fn callback_index(&self) -> MeterCallbackIndex {
        self.callback_index
    }

    pub const fn samples_consumed(&self) -> MeterSampleCount {
        MeterSampleCount::new(self.samples_consumed)
    }

    pub const fn write_index(&self) -> u8 {
        self.write_index
    }

    pub const fn read_index(&self) -> u8 {
        self.read_index
    }

    pub fn slot_origin(&self, slot: u8) -> Option<MeterLfeCallbackOrigin> {
        self.slots
            .get(usize::from(slot))
            .and_then(Option::as_ref)
            .map(|frame| frame.origin)
    }

    pub fn slot_samples(&self, slot: u8) -> Option<&[u32]> {
        self.slots
            .get(usize::from(slot))
            .and_then(Option::as_ref)
            .map(|frame| frame.sample_words.as_slice())
    }

    /// Functional Core: returns the complete local meter-root value for this
    /// persistent delay ring. PCM is kept only in the returned in-memory value.
    pub(crate) fn canonical_root_value(&self) -> Result<serde_json::Value, MeterLfeError> {
        self.validate()?;
        let mut slots = Vec::with_capacity(usize::from(LFE_DELAY_SLOTS));
        for slot in 0..LFE_DELAY_SLOTS {
            let value = match (self.slot_origin(slot), self.slot_samples(slot)) {
                (None, None) => serde_json::Value::Null,
                (Some(origin), Some(sample_words)) if sample_words.len() == CALLBACK_SAMPLES => {
                    serde_json::json!({
                        "origin_callback": origin.callback_index().value(),
                        "sample_words_u32": sample_words,
                    })
                }
                _ => return Err(MeterLfeError::StateInvalid),
            };
            slots.push(value);
        }

        Ok(serde_json::json!({
            "source_samples": self.source_samples.value(),
            "normal_callbacks": self.normal_callbacks,
            "callback_index": self.callback_index.value(),
            "samples_consumed": self.samples_consumed,
            "write_index": self.write_index,
            "read_index": self.read_index,
            "slots": slots,
        }))
    }

    fn validate(&self) -> Result<(), MeterLfeError> {
        let expected_callbacks = self
            .normal_callbacks
            .checked_add(3)
            .ok_or(MeterLfeError::StateInvalid)?;
        let expected_samples = self
            .callback_index
            .value()
            .checked_mul(CALLBACK_SAMPLES_U64)
            .ok_or(MeterLfeError::StateInvalid)?;
        if self.source_samples.value() == 0
            || self.normal_callbacks == 0
            || self.callback_index.value() > expected_callbacks
            || self.samples_consumed != expected_samples
            || self.write_index >= LFE_DELAY_SLOTS
            || self.read_index >= LFE_DELAY_SLOTS
            || self.read_index != (self.write_index + 1) % LFE_DELAY_SLOTS
        {
            return Err(MeterLfeError::StateInvalid);
        }
        for frame in self.slots.iter().flatten() {
            if frame.sample_words.len() != CALLBACK_SAMPLES
                || frame
                    .sample_words
                    .iter()
                    .any(|word| !f32::from_bits(*word).is_finite())
                || frame.origin.0.value() >= self.callback_index.value()
            {
                return Err(MeterLfeError::StateInvalid);
            }
        }
        Ok(())
    }

    fn expected_phase(&self) -> Result<MeterCallbackPhase, MeterLfeError> {
        let total = self
            .normal_callbacks
            .checked_add(3)
            .ok_or(MeterLfeError::ArithmeticOverflow)?;
        match self.callback_index.value() {
            index if index < self.normal_callbacks => Ok(MeterCallbackPhase::NormalSource),
            index if index < total => Ok(MeterCallbackPhase::ZeroInputDrain),
            _ => Err(MeterLfeError::CallbackOrder),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct MeterLfeDelayProposal {
    after: MeterLfeDelayState,
    output_words: Vec<u32>,
    output_origin: Option<MeterLfeCallbackOrigin>,
}

impl MeterLfeDelayProposal {
    pub const fn after(&self) -> &MeterLfeDelayState {
        &self.after
    }

    pub fn output_words(&self) -> &[u32] {
        &self.output_words
    }

    pub const fn output_origin(&self) -> Option<MeterLfeCallbackOrigin> {
        self.output_origin
    }

    pub fn into_parts(self) -> (MeterLfeDelayState, Vec<u32>, Option<MeterLfeCallbackOrigin>) {
        (self.after, self.output_words, self.output_origin)
    }
}

pub fn stage_meter_lfe_delay(
    before: &MeterLfeDelayState,
    callback_index: MeterCallbackIndex,
    phase: MeterCallbackPhase,
    input_words: &[u32],
) -> Result<MeterLfeDelayProposal, MeterLfeError> {
    before.validate()?;
    if callback_index != before.callback_index {
        return Err(MeterLfeError::CallbackOrder);
    }
    if phase != before.expected_phase()? {
        return Err(MeterLfeError::PhaseMismatch);
    }
    if input_words.len() != CALLBACK_SAMPLES {
        return Err(MeterLfeError::InputShape);
    }
    if input_words
        .iter()
        .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(MeterLfeError::InputNonFinite);
    }
    if phase == MeterCallbackPhase::ZeroInputDrain && input_words.iter().any(|word| *word != 0) {
        return Err(MeterLfeError::DrainInputNonZero);
    }

    let next_write = (before.write_index + 1) % LFE_DELAY_SLOTS;
    let next_read = (before.read_index + 1) % LFE_DELAY_SLOTS;
    let mut after = before.clone();
    after.write_index = next_write;
    after.read_index = next_read;
    after.slots[usize::from(next_write)] = Some(DelayedLfeFrame {
        origin: MeterLfeCallbackOrigin(callback_index),
        sample_words: input_words.to_vec(),
    });
    after.samples_consumed = before
        .samples_consumed
        .checked_add(CALLBACK_SAMPLES_U64)
        .ok_or(MeterLfeError::ArithmeticOverflow)?;
    after.callback_index = MeterCallbackIndex::new(
        before
            .callback_index
            .value()
            .checked_add(1)
            .ok_or(MeterLfeError::ArithmeticOverflow)?,
    );

    let (output_words, output_origin) = if before.samples_consumed < LFE_DELAY_SAMPLES {
        (vec![0; CALLBACK_SAMPLES], None)
    } else {
        let frame = after.slots[usize::from(next_read)]
            .as_ref()
            .ok_or(MeterLfeError::StateInvalid)?;
        (frame.sample_words.clone(), Some(frame.origin))
    };
    after.validate()?;
    Ok(MeterLfeDelayProposal {
        after,
        output_words,
        output_origin,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterFrontendCallbackInput {
    callback_index: MeterCallbackIndex,
    upstream_generation: ObjectRenderCallbackGeneration,
    phase: MeterCallbackPhase,
    current_lfe_words: Vec<u32>,
    regular_pcm_words: Vec<Vec<u32>>,
    regular_control_words: Vec<Vec<u32>>,
    spatial_callback_index: Option<FrontendCallbackIndex>,
    spatial_transaction_id: Option<CallbackLineageIdentity>,
    active_target_slots: Option<Box<[TargetSlotIndex]>>,
    target_control_lineages: Option<[ReaderTargetControlLineage; REGULAR_SOURCE_COUNT]>,
}

impl MeterFrontendCallbackInput {
    pub fn new(
        callback_index: MeterCallbackIndex,
        upstream_generation: ObjectRenderCallbackGeneration,
        phase: MeterCallbackPhase,
        current_lfe_words: Vec<u32>,
        regular_pcm_words: Vec<Vec<u32>>,
        regular_control_words: Vec<Vec<u32>>,
    ) -> Self {
        Self {
            callback_index,
            upstream_generation,
            phase,
            current_lfe_words,
            regular_pcm_words,
            regular_control_words,
            spatial_callback_index: None,
            spatial_transaction_id: None,
            active_target_slots: None,
            target_control_lineages: None,
        }
    }

    /// Builds the F06OF row view from same-callback F04A component rows and F05R56 controls.
    /// The spatial-control parent forwards admitted target rows; this adapter does not commit
    /// either owner or rewrite any row/history lineage.
    pub fn from_spatial_outputs(
        callback_index: MeterCallbackIndex,
        upstream_generation: ObjectRenderCallbackGeneration,
        phase: MeterCallbackPhase,
        current_lfe_words: Vec<u32>,
        component_rows: &MeterComponentOutputRows,
        target_controls: &ReaderTargetControlRows,
    ) -> Result<Self, MeterLfeError> {
        let target_callback = target_controls.callback_index();
        if component_rows.callback_index() != target_callback
            || component_rows.transaction_id() != target_controls.transaction_id()
            || u64::from(target_callback.value()) != callback_index.value()
        {
            return Err(MeterLfeError::CallbackOrder);
        }
        let expected_meter_phase = match component_rows.phase() {
            crate::spatial::post_window_fanout::CallbackPhase::Normal => {
                MeterCallbackPhase::NormalSource
            }
            crate::spatial::post_window_fanout::CallbackPhase::CachedDrain => {
                MeterCallbackPhase::ZeroInputDrain
            }
        };
        if phase != expected_meter_phase {
            return Err(MeterLfeError::PhaseMismatch);
        }
        if component_rows.active_target_slots() != target_controls.active_target_slots()
            || component_rows.canonical_rows_f32_bits().len()
                != REGULAR_SOURCE_COUNT * CALLBACK_SAMPLES
            || current_lfe_words.len() != CALLBACK_SAMPLES
        {
            return Err(MeterLfeError::InputShape);
        }
        if current_lfe_words
            .iter()
            .any(|word| !f32::from_bits(*word).is_finite())
        {
            return Err(MeterLfeError::InputNonFinite);
        }
        if phase == MeterCallbackPhase::ZeroInputDrain
            && current_lfe_words.iter().any(|word| *word != 0)
        {
            return Err(MeterLfeError::DrainInputNonZero);
        }
        if upstream_generation.value()
            != callback_index
                .value()
                .checked_add(1)
                .ok_or(MeterLfeError::ArithmeticOverflow)?
        {
            return Err(MeterLfeError::GenerationMismatch);
        }
        let expected_invocation = callback_index
            .value()
            .checked_sub(2)
            .map(ReaderInvocationIndex::new);
        if target_controls.reader_invocation() != expected_invocation {
            return Err(MeterLfeError::CallbackOrder);
        }

        let mut regular_pcm_words = Vec::with_capacity(REGULAR_SOURCE_COUNT);
        let mut regular_control_words = Vec::with_capacity(REGULAR_SOURCE_COUNT);
        for slot in 0..REGULAR_SOURCE_COUNT {
            let target_slot = TargetSlotIndex::from_validated(slot as u8);
            let component_row = component_rows
                .row_for_target_slot(target_slot)
                .ok_or(MeterLfeError::InputShape)?;
            let control_row = target_controls
                .rows()
                .get(slot)
                .ok_or(MeterLfeError::InputShape)?;
            if control_row.target_slot() != target_slot
                || control_row.producer_row()
                    != u8::try_from(slot).map_err(|_| MeterLfeError::InputShape)?
            {
                return Err(MeterLfeError::InputShape);
            }
            if let HistoryProducerGeneration::Callback(generation) =
                control_row.producer_generation()
            {
                if generation.value() > target_callback.value() {
                    return Err(MeterLfeError::CallbackOrder);
                }
            }
            regular_pcm_words.push(component_row.to_vec());
            regular_control_words.push(control_row.control52_words().to_vec());
        }

        let target_control_lineages = target_controls.lineages();
        Ok(Self {
            callback_index,
            upstream_generation,
            phase,
            current_lfe_words,
            regular_pcm_words,
            regular_control_words,
            spatial_callback_index: Some(target_callback),
            spatial_transaction_id: Some(target_controls.transaction_id().clone()),
            active_target_slots: Some(
                component_rows
                    .active_target_slots()
                    .to_vec()
                    .into_boxed_slice(),
            ),
            target_control_lineages: Some(target_control_lineages),
        })
    }

    pub const fn callback_index(&self) -> MeterCallbackIndex {
        self.callback_index
    }

    pub const fn upstream_generation(&self) -> ObjectRenderCallbackGeneration {
        self.upstream_generation
    }

    pub const fn phase(&self) -> MeterCallbackPhase {
        self.phase
    }

    pub fn current_lfe_words(&self) -> &[u32] {
        &self.current_lfe_words
    }

    pub fn regular_pcm_words(&self) -> &[Vec<u32>] {
        &self.regular_pcm_words
    }

    pub fn regular_control_words(&self) -> &[Vec<u32>] {
        &self.regular_control_words
    }

    pub const fn spatial_callback_index(&self) -> Option<FrontendCallbackIndex> {
        self.spatial_callback_index
    }

    pub fn spatial_transaction_id(&self) -> Option<&CallbackLineageIdentity> {
        self.spatial_transaction_id.as_ref()
    }

    pub fn active_target_slots(&self) -> Option<&[TargetSlotIndex]> {
        self.active_target_slots.as_deref()
    }

    pub const fn target_control_lineages(
        &self,
    ) -> Option<&[ReaderTargetControlLineage; REGULAR_SOURCE_COUNT]> {
        self.target_control_lineages.as_ref()
    }
}

#[must_use]
pub struct MeterFrontendOarPacketProposal {
    after_lfe: MeterLfeDelayState,
    packet: ObjectRenderCallbackPacket,
    lfe_output_origin: Option<MeterLfeCallbackOrigin>,
    spatial_callback_index: Option<FrontendCallbackIndex>,
    spatial_transaction_id: Option<CallbackLineageIdentity>,
    active_target_slots: Option<Box<[TargetSlotIndex]>>,
    target_control_lineages: Option<[ReaderTargetControlLineage; REGULAR_SOURCE_COUNT]>,
}

impl MeterFrontendOarPacketProposal {
    pub const fn after_lfe(&self) -> &MeterLfeDelayState {
        &self.after_lfe
    }

    pub const fn packet(&self) -> &ObjectRenderCallbackPacket {
        &self.packet
    }

    pub const fn lfe_output_origin(&self) -> Option<MeterLfeCallbackOrigin> {
        self.lfe_output_origin
    }

    pub const fn spatial_callback_index(&self) -> Option<FrontendCallbackIndex> {
        self.spatial_callback_index
    }

    pub fn spatial_transaction_id(&self) -> Option<&CallbackLineageIdentity> {
        self.spatial_transaction_id.as_ref()
    }

    pub fn active_target_slots(&self) -> Option<&[TargetSlotIndex]> {
        self.active_target_slots.as_deref()
    }

    pub const fn target_control_lineages(
        &self,
    ) -> Option<&[ReaderTargetControlLineage; REGULAR_SOURCE_COUNT]> {
        self.target_control_lineages.as_ref()
    }

    pub fn into_parts(
        self,
    ) -> (
        MeterLfeDelayState,
        ObjectRenderCallbackPacket,
        Option<MeterLfeCallbackOrigin>,
    ) {
        (self.after_lfe, self.packet, self.lfe_output_origin)
    }
}

pub fn stage_meter_frontend_oar_packet(
    before_lfe: &MeterLfeDelayState,
    input: MeterFrontendCallbackInput,
) -> Result<MeterFrontendOarPacketProposal, MeterLfeError> {
    if input.callback_index != before_lfe.callback_index {
        return Err(MeterLfeError::CallbackOrder);
    }
    let expected_generation = input
        .callback_index
        .value()
        .checked_add(1)
        .ok_or(MeterLfeError::ArithmeticOverflow)?;
    if input.upstream_generation.value() != expected_generation {
        return Err(MeterLfeError::GenerationMismatch);
    }
    let lfe = stage_meter_lfe_delay(
        before_lfe,
        input.callback_index,
        input.phase,
        &input.current_lfe_words,
    )?;
    if input.regular_pcm_words.len() != REGULAR_SOURCE_COUNT
        || input
            .regular_pcm_words
            .iter()
            .any(|row| row.len() != CALLBACK_SAMPLES)
        || input.regular_control_words.len() != REGULAR_SOURCE_COUNT
        || input
            .regular_control_words
            .iter()
            .any(|row| row.len() != CONTROL_WORD_COUNT)
    {
        return Err(MeterLfeError::InputShape);
    }
    if input
        .regular_pcm_words
        .iter()
        .flatten()
        .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(MeterLfeError::InputNonFinite);
    }

    let spatial_callback_index = input.spatial_callback_index;
    let spatial_transaction_id = input.spatial_transaction_id.clone();
    let active_target_slots = input.active_target_slots.clone();
    let target_control_lineages = input.target_control_lineages;
    let (after_lfe, delayed_lfe, origin) = lfe.into_parts();
    let mut pcm_words = Vec::with_capacity(TOTAL_SOURCE_COUNT);
    pcm_words.push(delayed_lfe);
    pcm_words.extend(input.regular_pcm_words);
    let mut controls = Vec::with_capacity(TOTAL_SOURCE_COUNT);
    controls.push(vec![0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    controls.extend(input.regular_control_words);
    let packet = ObjectRenderCallbackPacket::new(
        input.upstream_generation,
        match input.phase {
            MeterCallbackPhase::NormalSource => ObjectRenderCallbackPhase::Normal,
            MeterCallbackPhase::ZeroInputDrain => ObjectRenderCallbackPhase::Drain,
        },
        controls,
        pcm_words,
    );
    Ok(MeterFrontendOarPacketProposal {
        after_lfe,
        packet,
        lfe_output_origin: origin,
        spatial_callback_index,
        spatial_transaction_id,
        active_target_slots,
        target_control_lineages,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;
    fn input_frame(word: u32) -> Vec<u32> {
        vec![word; CALLBACK_SAMPLES]
    }

    fn regular_pcm() -> Vec<Vec<u32>> {
        (0..REGULAR_SOURCE_COUNT)
            .map(|row| {
                (0..CALLBACK_SAMPLES)
                    .map(|sample| {
                        let centered = i32::try_from(sample % 17).unwrap() - 8;
                        (((row + 1) as f32 * centered as f32) / 16.0_f32).to_bits()
                    })
                    .collect()
            })
            .collect()
    }

    fn regular_controls() -> Vec<Vec<u32>> {
        let mut controls = vec![vec![0; CONTROL_WORD_COUNT]; REGULAR_SOURCE_COUNT];
        for row in &mut controls {
            row[2] = 3;
            row[3] = 2;
            row[12] = 1;
        }
        controls
    }

    #[test]
    fn seven_generation_lfe_ring_matches_the_approved_two_callback_delay() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/meter_frontend_oar/lfe_delay_inputs.json"
        ))
        .unwrap();
        let source_samples =
            MeterSourceSampleCount::new(fixture["source_samples"].as_u64().unwrap()).unwrap();
        let mut state = MeterLfeDelayState::cold(source_samples).unwrap();
        let input_cases = fixture["input_frames"].as_array().unwrap();
        let mut input_words = Vec::with_capacity(input_cases.len());
        for case in input_cases {
            let callback = MeterCallbackIndex::new(case["callback_index"].as_u64().unwrap());
            let phase = if case["phase"].as_str().unwrap() == "NORMAL" {
                MeterCallbackPhase::NormalSource
            } else {
                MeterCallbackPhase::ZeroInputDrain
            };
            let word = case["constant_f32_word"].as_u64().unwrap() as u32;
            let current = input_frame(word);
            let proposal = stage_meter_lfe_delay(&state, callback, phase, &current).unwrap();
            if callback.value() < 2 {
                assert_eq!(proposal.output_origin(), None);
                assert!(proposal.output_words().iter().all(|sample| *sample == 0));
            } else {
                let origin = callback.value() - 2;
                assert_eq!(
                    proposal
                        .output_origin()
                        .map(MeterLfeCallbackOrigin::callback_index),
                    Some(MeterCallbackIndex::new(origin))
                );
                assert_eq!(
                    proposal.output_words(),
                    input_words[usize::try_from(origin).unwrap()]
                );
            }
            input_words.push(current);
            state = proposal.into_parts().0;
        }
        assert_eq!(state.callback_index().value(), 7);
        assert_eq!(state.samples_consumed().value(), 7 * 1_536);
        assert_eq!(state.write_index(), 1);
        assert_eq!(state.read_index(), 2);
    }

    #[test]
    fn packet_assembly_pairs_delayed_lfe_with_same_callback_regular_rows_and_controls() {
        let source_samples = MeterSourceSampleCount::new(1).unwrap();
        let mut lfe_state = MeterLfeDelayState::cold(source_samples).unwrap();
        for callback in 0..4_u64 {
            let phase = if callback == 0 {
                MeterCallbackPhase::NormalSource
            } else {
                MeterCallbackPhase::ZeroInputDrain
            };
            let lfe_word = if callback == 0 { 0x3f80_0000 } else { 0 };
            let input = MeterFrontendCallbackInput::new(
                MeterCallbackIndex::new(callback),
                ObjectRenderCallbackGeneration::try_new(callback + 1).unwrap(),
                phase,
                input_frame(lfe_word),
                regular_pcm(),
                regular_controls(),
            );
            let proposal = stage_meter_frontend_oar_packet(&lfe_state, input).unwrap();
            assert_eq!(proposal.packet().generation().value(), callback + 1);
            assert_eq!(
                proposal.packet().phase(),
                match phase {
                    MeterCallbackPhase::NormalSource => ObjectRenderCallbackPhase::Normal,
                    MeterCallbackPhase::ZeroInputDrain => ObjectRenderCallbackPhase::Drain,
                }
            );
            assert_eq!(proposal.packet().pcm_words().len(), TOTAL_SOURCE_COUNT);
            assert_eq!(proposal.packet().controls().len(), TOTAL_SOURCE_COUNT);
            assert_eq!(proposal.packet().pcm_words()[0].len(), CALLBACK_SAMPLES);
            if callback < 2 {
                assert!(
                    proposal.packet().pcm_words()[0]
                        .iter()
                        .all(|word| *word == 0)
                );
                assert_eq!(proposal.lfe_output_origin(), None);
            } else {
                assert_eq!(
                    proposal
                        .lfe_output_origin()
                        .unwrap()
                        .callback_index()
                        .value(),
                    callback - 2
                );
            }
            assert_eq!(
                proposal.packet().controls()[0],
                [0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            );
            assert_eq!(proposal.packet().controls()[1], regular_controls()[0]);
            assert_eq!(proposal.packet().pcm_words()[1], regular_pcm()[0]);
            lfe_state = proposal.into_parts().0;
        }

        let before = lfe_state.clone();
        let mismatched_generation = MeterFrontendCallbackInput::new(
            MeterCallbackIndex::new(4),
            ObjectRenderCallbackGeneration::try_new(6).unwrap(),
            MeterCallbackPhase::ZeroInputDrain,
            input_frame(0),
            regular_pcm(),
            regular_controls(),
        );
        assert_eq!(
            stage_meter_frontend_oar_packet(&lfe_state, mismatched_generation).err(),
            Some(MeterLfeError::GenerationMismatch)
        );
        assert_eq!(lfe_state, before);
    }

    #[test]
    fn lfe_errors_leave_the_before_ring_and_packets_unchanged() {
        let state = MeterLfeDelayState::cold(MeterSourceSampleCount::new(1).unwrap()).unwrap();
        let snapshot = state.clone();
        let mut nonfinite = input_frame(0);
        nonfinite[17] = f32::NAN.to_bits();
        assert_eq!(
            stage_meter_lfe_delay(
                &state,
                MeterCallbackIndex::new(0),
                MeterCallbackPhase::NormalSource,
                &nonfinite,
            )
            .err(),
            Some(MeterLfeError::InputNonFinite)
        );
        assert_eq!(
            MeterLfeError::InputNonFinite.code(),
            "METER_LFE_INPUT_NONFINITE"
        );
        assert_eq!(state, snapshot);
        assert_eq!(
            stage_meter_lfe_delay(
                &state,
                MeterCallbackIndex::new(0),
                MeterCallbackPhase::NormalSource,
                &input_frame(0)[..CALLBACK_SAMPLES - 1],
            )
            .err(),
            Some(MeterLfeError::InputShape)
        );
        assert_eq!(state, snapshot);
    }

    #[test]
    fn spatial_outputs_bind_component_and_control_rows_by_target_slot_and_keep_s0_lineage() {
        use crate::{
            joc::oamd::reader_target::HistoryProducerGeneration,
            spatial::{post_window_fanout::CallbackPhase, target_context::TargetSlotIndex},
            timeline::FrontendCallbackIndex,
            transaction::{AttemptId, CallbackLineageIdentity, ProgrammeId},
        };

        let frontend_callback = FrontendCallbackIndex::new(0);
        let programme = ProgrammeId::new("METERROWTEST1").unwrap();
        let attempt = AttemptId::new(0);
        let transaction = CallbackLineageIdentity::new(programme, attempt, frontend_callback);
        let active_slots = vec![
            TargetSlotIndex::from_validated(2),
            TargetSlotIndex::from_validated(8),
        ];
        let component_rows = MeterComponentOutputRows::fixture(
            frontend_callback,
            transaction.clone(),
            CallbackPhase::Normal,
            active_slots.clone(),
        );
        let generations = std::array::from_fn(|row| {
            if row == 0 {
                HistoryProducerGeneration::Cold
            } else {
                HistoryProducerGeneration::Callback(frontend_callback)
            }
        });
        let target_controls = ReaderTargetControlRows::fixture(
            frontend_callback,
            transaction.clone(),
            active_slots.clone(),
            None,
            generations,
        );

        let input = MeterFrontendCallbackInput::from_spatial_outputs(
            MeterCallbackIndex::new(0),
            ObjectRenderCallbackGeneration::try_new(1).unwrap(),
            MeterCallbackPhase::NormalSource,
            input_frame(0x3f00_0000),
            &component_rows,
            &target_controls,
        )
        .unwrap();

        assert_eq!(input.spatial_callback_index(), Some(frontend_callback));
        assert_eq!(input.spatial_transaction_id(), Some(&transaction));
        assert_eq!(input.active_target_slots(), Some(active_slots.as_slice()));
        assert_eq!(input.regular_pcm_words().len(), REGULAR_SOURCE_COUNT);
        assert_eq!(input.regular_control_words().len(), REGULAR_SOURCE_COUNT);
        for slot in 0..REGULAR_SOURCE_COUNT {
            assert_eq!(
                input.regular_pcm_words()[slot][0],
                ((slot + 1) as f32).to_bits()
            );
            assert_eq!(
                input.regular_control_words()[slot],
                vec![slot as u32 + 1; 13]
            );
        }
        let lineages = *input.target_control_lineages().unwrap();
        assert_eq!(
            lineages[0].producer_generation(),
            HistoryProducerGeneration::Cold
        );
        assert_eq!(
            lineages[1].producer_generation(),
            HistoryProducerGeneration::Callback(frontend_callback)
        );
        for (slot, lineage) in lineages.iter().enumerate() {
            assert_eq!(lineage.target_slot().value(), slot as u8);
            assert_eq!(lineage.producer_row(), slot as u8);
        }

        let lfe = MeterLfeDelayState::cold(MeterSourceSampleCount::new(1_536).unwrap()).unwrap();
        let packet = stage_meter_frontend_oar_packet(&lfe, input).unwrap();
        assert_eq!(packet.lfe_output_origin(), None);
        assert_eq!(
            packet.packet().controls()[0],
            [0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(packet.packet().controls()[1], vec![1; 13]);
        assert_eq!(packet.packet().pcm_words()[1][0], 1.0_f32.to_bits());
        assert_eq!(packet.target_control_lineages().unwrap(), &lineages);
    }
}
