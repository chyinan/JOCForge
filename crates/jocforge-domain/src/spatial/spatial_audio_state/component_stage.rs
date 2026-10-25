// pattern: Functional Core
//! F04A component consumer stage. Production packet construction belongs to the future spatial_control producer.

use super::{
    ComponentAudioState, ComponentFilterParameterState, RawBytes, SpatialAudioJoinMetadata,
    SpatialAudioMachine,
};
use crate::{
    numeric::{FiniteF32, R32Error, r32_add, r32_mul, r32_sub},
    profile::{EncodingProfile, resolve_selected_profile},
    spatial::source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
    spatial::{
        F04ParameterProviderLineage, SpatialControlDigest, SpatialControlJoinMetadata,
        SpatialControlMachine, SpatialControlProviderLineage, ValidatedComponentParameterProvider,
        ValidatedWindowQ15Provider,
    },
    timeline::FrontendCallbackIndex,
    transaction::CallbackLineageIdentity,
};

const SOURCE_ROW_COUNT: usize = 48;
const INSTALLED_SAMPLES: usize = 3072;
const COMPONENT_ROW_COUNT: usize = 15;
const MATRIX_COLUMN_COUNT: usize = 48;
const FRAME_SAMPLES: usize = 1536;
const RING_SAMPLES: usize = 1472;
const BLOCK_SAMPLES: usize = 64;
const BLOCK_COUNT: u8 = 24;
const F32_BYTES: usize = 4;
const INSTALLED_WORDS: usize = SOURCE_ROW_COUNT * INSTALLED_SAMPLES;
#[cfg(test)]
const MATRIX_WORDS: usize = COMPONENT_ROW_COUNT * MATRIX_COLUMN_COUNT;
const COMPONENT_WORDS: usize = COMPONENT_ROW_COUNT * FRAME_SAMPLES;
const FILTER_HISTORY_BYTES: usize = BLOCK_SAMPLES * F32_BYTES;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ComponentStageError {
    State,
    PipelineRetired,
    CallbackOrder,
    Input,
    ProfileUnsupported,
    NumericFailure,
}

impl ComponentStageError {
    // The deferred parent transaction layer consumes this local translation.
    #[allow(dead_code)]
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::State => "F04_STATE_ERROR",
            Self::PipelineRetired => "F04_PIPELINE_RETIRED",
            Self::CallbackOrder => "F04_CALLBACK_ORDER",
            Self::Input => "F04_INPUT_ERROR",
            Self::ProfileUnsupported => "F04_PROFILE_UNSUPPORTED",
            Self::NumericFailure => "F04_NUMERIC_FAILURE",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
// The future spatial_control producer supplies the callback phase for validation.
#[allow(dead_code)]
pub(crate) enum ComponentCallbackPhase {
    Normal,
    CachedDrain,
}

/// Opaque future-producer packet; fields are intentionally private and have no production constructor in E2.
#[must_use]
pub(crate) struct SpatialAudioInputPacket {
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: CallbackLineageIdentity,
    callback_index: FrontendCallbackIndex,
    phase: ComponentCallbackPhase,
    profile_identity: EncodingProfile,
    spatial_control_before_generation: u64,
    spatial_control_before_digest: SpatialControlDigest,
    spatial_control_provider_lineage: SpatialControlProviderLineage,
    provider_lineage: [F04ParameterProviderLineage; 2],
    installed48_f32_bits: Box<[u32]>,
    matrix_rows: u8,
    matrix_f32_bits: Box<[u32]>,
    destinations: Box<[u8]>,
}

#[must_use]
pub(crate) struct ComponentStageProposal {
    after: ComponentAfterState,
    component_output: Box<[FiniteF32]>,
    receipt: ComponentStageReceipt,
}

// The deferred parent composer consumes this complete component after-state.
#[allow(dead_code)]
pub(super) struct ComponentAfterState {
    pub(super) audio: ComponentAudioState,
    pub(super) parameters: ComponentFilterParameterState,
    pub(super) filter_history: [RawBytes; COMPONENT_ROW_COUNT],
}

// The deferred parent composer forwards this typed receipt without local commit authority.
#[allow(dead_code)]
pub(super) struct ComponentStageReceipt {
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: CallbackLineageIdentity,
    callback_index: FrontendCallbackIndex,
    phase: ComponentCallbackPhase,
    profile_identity: EncodingProfile,
    spatial_audio_before_generation: u64,
    spatial_audio_before_digest: super::SpatialAudioDigest,
    spatial_control_before_generation: u64,
    spatial_control_before_digest: SpatialControlDigest,
    spatial_control_provider_lineage: SpatialControlProviderLineage,
    provider_lineage: [F04ParameterProviderLineage; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ComponentBlockIndex(u8);

impl ComponentBlockIndex {
    fn all() -> impl Iterator<Item = Self> {
        (0..BLOCK_COUNT).map(Self)
    }

    fn sample_offset(self) -> usize {
        usize::from(self.0) * BLOCK_SAMPLES
    }
}

struct ComponentStateView<'a> {
    component_state: &'a ComponentAudioState,
    parameters: &'a ComponentFilterParameterState,
    filter_history: &'a [RawBytes; COMPONENT_ROW_COUNT],
}

struct ValidatedComponentPayload {
    installed48: Box<[FiniteF32]>,
    matrix: Box<[FiniteF32]>,
    destinations: Box<[u8]>,
    matrix_rows: usize,
}

struct ComponentControllerParameters {
    coefficients: [FiniteF32; 5],
    reciprocal_pivot: FiniteF32,
    target_pivot: FiniteF32,
    envelope_weight: FiniteF32,
    gain_scale: FiniteF32,
    clamp_limit: FiniteF32,
    hold_limit: i64,
}

struct OverlapStage {
    samples: Box<[FiniteF32]>,
    next_overlap: [RawBytes; COMPONENT_ROW_COUNT],
}

struct FilterStage {
    samples: Box<[FiniteF32]>,
    parameters: ComponentFilterParameterState,
    history: [RawBytes; COMPONENT_ROW_COUNT],
}

struct ComponentFilterBlock {
    // The F04A.8 primitive vector assertions consume this callback-local diagnostic.
    #[allow(dead_code)]
    preclip: Box<[FiniteF32]>,
    samples: Box<[FiniteF32]>,
}

struct FilterBlockSettings<'a> {
    kernel: &'a [FiniteF32; BLOCK_SAMPLES],
    output_level: FiniteF32,
    previous_gain: FiniteF32,
    smoothed_gain: FiniteF32,
    gain_scale: FiniteF32,
    clamp_limit: FiniteF32,
}

#[cfg(test)]
struct ComponentPacketFixture<'a> {
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: CallbackLineageIdentity,
    callback_index: FrontendCallbackIndex,
    phase: ComponentCallbackPhase,
    profile_identity: EncodingProfile,
    spatial_control: &'a SpatialControlMachine,
    provider_lineage: [F04ParameterProviderLineage; 2],
    installed48_f32_bits: Vec<u32>,
    matrix_rows: u8,
    matrix_f32_bits: Vec<u32>,
    destinations: Vec<u8>,
}

struct ParameterTransition {
    after: ComponentFilterParameterState,
    #[allow(dead_code)]
    current_peak: FiniteF32,
    #[allow(dead_code)]
    scaled_peak: FiniteF32,
    #[allow(dead_code)]
    attenuation: FiniteF32,
    #[allow(dead_code)]
    peak_witness: Option<(usize, usize)>,
    #[allow(dead_code)]
    gain_branch: GainBranch,
}

#[derive(Clone, Copy)]
enum GainBranch {
    HoldCounterIncrement,
    TargetRecurrenceAndSmoothedClamp,
    GainFallsToEnvelope,
}

impl GainBranch {
    // Vector assertion labels are not used as runtime control flow.
    #[allow(dead_code)]
    const fn as_str(self) -> &'static str {
        match self {
            Self::HoldCounterIncrement => "hold_counter_increment",
            Self::TargetRecurrenceAndSmoothedClamp => "target_recurrence_and_smoothed_clamp",
            Self::GainFallsToEnvelope => "gain_falls_to_envelope",
        }
    }
}

impl SpatialAudioInputPacket {
    #[cfg(test)]
    fn fixture(fixture: ComponentPacketFixture<'_>) -> Self {
        Self {
            programme_id: fixture.programme_id,
            attempt: fixture.attempt,
            transaction_id: fixture.transaction_id,
            callback_index: fixture.callback_index,
            phase: fixture.phase,
            profile_identity: fixture.profile_identity,
            spatial_control_before_generation: fixture.spatial_control.generation(),
            spatial_control_before_digest: fixture.spatial_control.digest(),
            spatial_control_provider_lineage: fixture.spatial_control.provider_lineage(),
            provider_lineage: fixture.provider_lineage,
            installed48_f32_bits: fixture.installed48_f32_bits.into_boxed_slice(),
            matrix_rows: fixture.matrix_rows,
            matrix_f32_bits: fixture.matrix_f32_bits.into_boxed_slice(),
            destinations: fixture.destinations.into_boxed_slice(),
        }
    }
}

impl ComponentStageProposal {
    #[cfg(test)]
    pub(in crate::spatial::spatial_audio_state) fn fixture_for_type10(
        spatial_audio_join: &SpatialAudioJoinMetadata,
        phase: ComponentCallbackPhase,
        spatial_control: &SpatialControlMachine,
        provider_lineage: [F04ParameterProviderLineage; 2],
        spatial_control_provider_lineage: Option<&SpatialControlProviderLineage>,
        component_output: Vec<FiniteF32>,
    ) -> Self {
        Self {
            after: ComponentAfterState {
                audio: ComponentAudioState::cold(),
                parameters: ComponentFilterParameterState::cold(),
                filter_history: std::array::from_fn(|_| RawBytes::zeroed(FILTER_HISTORY_BYTES)),
            },
            component_output: component_output.into_boxed_slice(),
            receipt: ComponentStageReceipt {
                programme_id: spatial_audio_join.programme_id().clone(),
                attempt: spatial_audio_join.attempt(),
                transaction_id: spatial_audio_join.transaction_id().clone(),
                callback_index: spatial_audio_join.callback_index(),
                phase,
                profile_identity: spatial_audio_join.profile_identity(),
                spatial_audio_before_generation: spatial_audio_join.before_generation(),
                spatial_audio_before_digest: spatial_audio_join.before_digest(),
                spatial_control_before_generation: spatial_control.generation(),
                spatial_control_before_digest: spatial_control.digest(),
                spatial_control_provider_lineage: spatial_control_provider_lineage
                    .copied()
                    .unwrap_or_else(|| spatial_control.provider_lineage()),
                provider_lineage,
            },
        }
    }

    #[allow(dead_code)]
    pub(super) fn into_parts(
        self,
    ) -> (ComponentAfterState, Box<[FiniteF32]>, ComponentStageReceipt) {
        (self.after, self.component_output, self.receipt)
    }
}

impl ComponentStageReceipt {
    pub(super) fn programme_id(&self) -> &SourceFlagsProgrammeId {
        &self.programme_id
    }

    pub(super) const fn attempt(&self) -> SourceFlagsAttempt {
        self.attempt
    }

    pub(super) fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub(super) const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub(super) const fn phase(&self) -> ComponentCallbackPhase {
        self.phase
    }

    pub(super) const fn profile_identity(&self) -> EncodingProfile {
        self.profile_identity
    }

    pub(super) const fn spatial_audio_before_generation(&self) -> u64 {
        self.spatial_audio_before_generation
    }

    pub(super) const fn spatial_audio_before_digest(&self) -> super::SpatialAudioDigest {
        self.spatial_audio_before_digest
    }

    pub(super) const fn spatial_control_before_generation(&self) -> u64 {
        self.spatial_control_before_generation
    }

    pub(super) const fn spatial_control_before_digest(&self) -> SpatialControlDigest {
        self.spatial_control_before_digest
    }

    pub(super) const fn spatial_control_provider_lineage(&self) -> SpatialControlProviderLineage {
        self.spatial_control_provider_lineage
    }

    pub(super) const fn provider_lineage(&self) -> [F04ParameterProviderLineage; 2] {
        self.provider_lineage
    }
}

#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
// Consumes the opaque packet capability for one component stage.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn produce_component_stage(
    spatial_audio: &SpatialAudioMachine,
    spatial_audio_join: &SpatialAudioJoinMetadata,
    spatial_control_join: &SpatialControlJoinMetadata,
    spatial_control: &SpatialControlMachine,
    expected_phase: ComponentCallbackPhase,
    packet: SpatialAudioInputPacket,
    window: &ValidatedWindowQ15Provider,
    parameters: &ValidatedComponentParameterProvider,
) -> Result<ComponentStageProposal, ComponentStageError> {
    let before = &spatial_audio.committed.root;
    if before.retired {
        return Err(ComponentStageError::PipelineRetired);
    }

    let selected_profile = spatial_audio.profile_identity();
    for profile in [
        selected_profile,
        spatial_audio_join.profile_identity(),
        spatial_control_join.profile_identity(),
        spatial_control.profile(),
        packet.profile_identity,
    ] {
        if resolve_selected_profile(&profile).is_err() || profile != selected_profile {
            return Err(ComponentStageError::ProfileUnsupported);
        }
    }

    if spatial_audio_join.callback_index() != before.callback_index
        || spatial_control_join.callback_index() != before.callback_index
        || packet.callback_index != before.callback_index
        || packet.phase != expected_phase
    {
        return Err(ComponentStageError::CallbackOrder);
    }

    if spatial_audio_join.before_generation() != spatial_audio.generation()
        || spatial_audio_join.before_digest() != spatial_audio.digest()
        || packet.spatial_control_before_generation != spatial_control.generation()
        || packet.spatial_control_before_digest != spatial_control.digest()
        || spatial_control_join.before_generation() != spatial_control.generation()
        || spatial_control_join.before_digest() != spatial_control.digest()
    {
        return Err(ComponentStageError::State);
    }

    let before_state = ComponentStateView {
        component_state: &before.component_state,
        parameters: &before.component_parameter_state,
        filter_history: &before.component_filter_history,
    };
    validate_component_state(&before_state)?;

    if packet.programme_id != *spatial_audio_join.programme_id()
        || packet.attempt != spatial_audio_join.attempt()
        || packet.transaction_id != *spatial_audio_join.transaction_id()
        || packet.transaction_id != *spatial_control_join.transaction_id()
        || packet.profile_identity != spatial_audio_join.profile_identity()
        || packet.spatial_control_provider_lineage != spatial_control.provider_lineage()
        || spatial_control_join.provider_lineage() != spatial_control.provider_lineage()
        || packet.provider_lineage
            != [
                *window.receipt().validated_manifest_lineage(),
                *parameters.receipt().validated_manifest_lineage(),
            ]
        || !provider_qualification_is_scoped(parameters)
    {
        return Err(ComponentStageError::Input);
    }

    let payload = validate_packet_payload(&packet)?;
    let controller = ComponentControllerParameters::from_provider(parameters)?;
    let kernel = component_kernel_from_provider(window)?;
    let matrix = mix_matrix(&payload)?;
    let overlap = make_overlap(before_state.component_state, &matrix)?;
    let filtered = filter_component(
        &overlap.samples,
        before_state.parameters,
        before_state.filter_history,
        &kernel,
        &controller,
    )?;
    let (component_output, component_state) = apply_source_ring(
        before_state.component_state,
        overlap.next_overlap,
        &filtered.samples,
    )?;

    let receipt = ComponentStageReceipt {
        programme_id: packet.programme_id,
        attempt: packet.attempt,
        transaction_id: packet.transaction_id,
        callback_index: packet.callback_index,
        phase: packet.phase,
        profile_identity: packet.profile_identity,
        spatial_audio_before_generation: spatial_audio_join.before_generation(),
        spatial_audio_before_digest: spatial_audio_join.before_digest(),
        spatial_control_before_generation: packet.spatial_control_before_generation,
        spatial_control_before_digest: packet.spatial_control_before_digest,
        spatial_control_provider_lineage: packet.spatial_control_provider_lineage,
        provider_lineage: packet.provider_lineage,
    };

    Ok(ComponentStageProposal {
        after: ComponentAfterState {
            parameters: filtered.parameters,
            audio: component_state,
            filter_history: filtered.history,
        },
        component_output,
        receipt,
    })
}

fn validate_component_state(before: &ComponentStateView<'_>) -> Result<(), ComponentStageError> {
    for row in &before.component_state.overlap_history_rows {
        validate_raw_row(row, FRAME_SAMPLES * F32_BYTES)?;
    }
    for row in &before.component_state.source_ring_rows {
        validate_raw_row(row, RING_SAMPLES * F32_BYTES)?;
    }
    for row in before.filter_history {
        validate_raw_row(row, FILTER_HISTORY_BYTES)?;
    }

    let state = before.parameters;
    let scalar_values = [
        state.peak,
        state.previous_gain,
        state.envelope,
        state.smoothed_gain,
        state.target_gain,
        state.output_level,
    ];
    if scalar_values
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
        || state.hold < 0
        || state.hold > i64::from(i32::MAX)
        || state.blocks_processed < 0
        || state
            .blocks_processed
            .checked_add(i64::from(BLOCK_COUNT))
            .is_none()
    {
        return Err(ComponentStageError::State);
    }
    Ok(())
}

fn validate_raw_row(row: &RawBytes, expected_bytes: usize) -> Result<(), ComponentStageError> {
    if row.bytes.len() != expected_bytes {
        return Err(ComponentStageError::State);
    }
    for word in row.bytes.chunks_exact(F32_BYTES) {
        let bits = u32::from_le_bytes(word.try_into().map_err(|_| ComponentStageError::State)?);
        if !f32::from_bits(bits).is_finite() {
            return Err(ComponentStageError::State);
        }
    }
    Ok(())
}

fn validate_packet_payload(
    packet: &SpatialAudioInputPacket,
) -> Result<ValidatedComponentPayload, ComponentStageError> {
    let rows = usize::from(packet.matrix_rows);
    if rows > COMPONENT_ROW_COUNT
        || packet.installed48_f32_bits.len() != INSTALLED_WORDS
        || packet.matrix_f32_bits.len() != rows * MATRIX_COLUMN_COUNT
        || packet.destinations.len() != rows
    {
        return Err(ComponentStageError::Input);
    }

    let mut seen_destinations = [false; COMPONENT_ROW_COUNT];
    for destination in packet.destinations.iter().copied() {
        let destination = usize::from(destination);
        if destination >= COMPONENT_ROW_COUNT || seen_destinations[destination] {
            return Err(ComponentStageError::Input);
        }
        seen_destinations[destination] = true;
    }

    let installed48 = finite_values_from_bits(&packet.installed48_f32_bits)?;
    let matrix = finite_values_from_bits(&packet.matrix_f32_bits)?;
    Ok(ValidatedComponentPayload {
        installed48,
        matrix,
        destinations: packet.destinations.clone(),
        matrix_rows: rows,
    })
}

fn finite_values_from_bits(bits: &[u32]) -> Result<Box<[FiniteF32]>, ComponentStageError> {
    bits.iter()
        .copied()
        .map(|word| FiniteF32::new(f32::from_bits(word)).map_err(|_| ComponentStageError::Input))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn provider_qualification_is_scoped(provider: &ValidatedComponentParameterProvider) -> bool {
    let receipt = provider.receipt();
    !receipt.asset_implementation_ready()
        && receipt.authorized_scope()
            == "INTERNAL_SELECTED_PROFILE_PROVIDER_VALIDATION_AND_FUTURE_CONSUMPTION"
        && receipt.dsp_consumption_status() == "NOT_YET_VALIDATED"
}

impl ComponentControllerParameters {
    fn from_provider(
        provider: &ValidatedComponentParameterProvider,
    ) -> Result<Self, ComponentStageError> {
        let words = provider.coefficient_words();
        let coefficients = [
            finite_provider_word(words[0])?,
            finite_provider_word(words[1])?,
            finite_provider_word(words[2])?,
            finite_provider_word(words[3])?,
            finite_provider_word(words[4])?,
        ];
        let gain_scale = finite_provider_word(0x3b80_0000)?;
        let clamp_limit = finite_provider_word(0x3f80_0000)?;
        Ok(Self {
            coefficients,
            reciprocal_pivot: finite_provider_word(provider.reciprocal_pivot_word())?,
            target_pivot: finite_provider_word(provider.target_pivot_word())?,
            envelope_weight: finite_provider_word(provider.envelope_weight_word())?,
            gain_scale,
            clamp_limit,
            hold_limit: 1,
        })
    }
}

fn finite_provider_word(bits: u32) -> Result<FiniteF32, ComponentStageError> {
    FiniteF32::new(f32::from_bits(bits)).map_err(|_| ComponentStageError::Input)
}

fn component_kernel_from_provider(
    provider: &ValidatedWindowQ15Provider,
) -> Result<[FiniteF32; BLOCK_SAMPLES], ComponentStageError> {
    let mut kernel = [finite_constant(0.0)?; BLOCK_SAMPLES];
    for (index, q15) in provider.component_kernel64().iter().copied().enumerate() {
        let exact_f32 = f32::from(q15) / 32_768.0_f32;
        kernel[index] = FiniteF32::new(exact_f32).map_err(|_| ComponentStageError::Input)?;
    }
    Ok(kernel)
}

fn finite_constant(value: f32) -> Result<FiniteF32, ComponentStageError> {
    FiniteF32::new(value).map_err(|_| ComponentStageError::NumericFailure)
}

fn mix_matrix(
    payload: &ValidatedComponentPayload,
) -> Result<Box<[FiniteF32]>, ComponentStageError> {
    let zero = finite_constant(0.0)?;
    let mut mixer = vec![zero; COMPONENT_ROW_COUNT * INSTALLED_SAMPLES].into_boxed_slice();
    for logical_row in 0..payload.matrix_rows {
        let destination = usize::from(payload.destinations[logical_row]);
        for source_row in 0..SOURCE_ROW_COUNT {
            let coefficient = payload.matrix[logical_row * MATRIX_COLUMN_COUNT + source_row];
            if coefficient.value() <= 0.0 {
                continue;
            }
            let source_offset = source_row * INSTALLED_SAMPLES;
            let destination_offset = destination * INSTALLED_SAMPLES;
            for sample in 0..INSTALLED_SAMPLES {
                let product = r32_mul(payload.installed48[source_offset + sample], coefficient)
                    .map_err(map_numeric_error)?;
                let output_index = destination_offset + sample;
                mixer[output_index] =
                    r32_add(mixer[output_index], product).map_err(map_numeric_error)?;
            }
        }
    }
    Ok(mixer)
}

fn make_overlap(
    before: &ComponentAudioState,
    mixer: &[FiniteF32],
) -> Result<OverlapStage, ComponentStageError> {
    if mixer.len() != COMPONENT_ROW_COUNT * INSTALLED_SAMPLES {
        return Err(ComponentStageError::State);
    }
    let mut samples = vec![finite_constant(0.0)?; COMPONENT_WORDS].into_boxed_slice();
    let mut next_overlap = Vec::with_capacity(COMPONENT_ROW_COUNT);
    for row in 0..COMPONENT_ROW_COUNT {
        let row_offset = row * INSTALLED_SAMPLES;
        for sample in 0..FRAME_SAMPLES {
            let old = raw_finite_at(&before.overlap_history_rows[row], sample)?;
            samples[row * FRAME_SAMPLES + sample] =
                r32_add(mixer[row_offset + sample], old).map_err(map_numeric_error)?;
        }
        next_overlap.push(raw_bytes_from_finite(
            &mixer[row_offset + FRAME_SAMPLES..row_offset + INSTALLED_SAMPLES],
        ));
    }
    let next_overlap = next_overlap
        .try_into()
        .map_err(|_| ComponentStageError::State)?;
    Ok(OverlapStage {
        samples,
        next_overlap,
    })
}

fn raw_finite_at(row: &RawBytes, sample_index: usize) -> Result<FiniteF32, ComponentStageError> {
    let start = sample_index
        .checked_mul(F32_BYTES)
        .ok_or(ComponentStageError::State)?;
    let end = start
        .checked_add(F32_BYTES)
        .ok_or(ComponentStageError::State)?;
    let word = row
        .bytes
        .get(start..end)
        .ok_or(ComponentStageError::State)?;
    let bits = u32::from_le_bytes(word.try_into().map_err(|_| ComponentStageError::State)?);
    FiniteF32::new(f32::from_bits(bits)).map_err(|_| ComponentStageError::State)
}

fn raw_bytes_from_finite(samples: &[FiniteF32]) -> RawBytes {
    let mut bytes = Vec::with_capacity(samples.len() * F32_BYTES);
    for sample in samples {
        bytes.extend_from_slice(&sample.to_bits().to_le_bytes());
    }
    RawBytes {
        bytes: bytes.into_boxed_slice(),
    }
}

fn map_numeric_error(_: R32Error) -> ComponentStageError {
    ComponentStageError::NumericFailure
}

fn update_parameters(
    old: &ComponentFilterParameterState,
    block_input: &[FiniteF32],
    parameters: &ComponentControllerParameters,
) -> Result<ParameterTransition, ComponentStageError> {
    if block_input.len() != COMPONENT_ROW_COUNT * BLOCK_SAMPLES {
        return Err(ComponentStageError::State);
    }
    let mut current_peak = finite_constant(0.0)?;
    let mut peak_witness = None;
    for row in 0..COMPONENT_ROW_COUNT {
        for sample in 0..BLOCK_SAMPLES {
            let value = block_input[row * BLOCK_SAMPLES + sample];
            let magnitude = FiniteF32::new(f32::from_bits(value.to_bits() & 0x7fff_ffff))
                .map_err(|_| ComponentStageError::NumericFailure)?;
            if magnitude.value() > current_peak.value() {
                current_peak = magnitude;
                peak_witness = Some((row, sample));
            }
        }
    }

    let old_peak = finite_state(old.peak)?;
    let old_envelope = finite_state(old.envelope)?;
    let scaled_peak =
        r32_mul(max_first(current_peak, old_peak), old_envelope).map_err(map_numeric_error)?;
    let attenuation = reciprocal_attenuation(scaled_peak, parameters.reciprocal_pivot)?;

    let old_smoothed = finite_state(old.smoothed_gain)?;
    let old_target = finite_state(old.target_gain)?;
    let maximum = r32_mul(old_envelope, attenuation).map_err(map_numeric_error)?;
    let previous_gain = old_smoothed;
    let mut working_hold = old.hold;
    let mut target = old_target;
    let mut smoothed = old_smoothed;
    let gain_branch;

    if old_smoothed.value() > maximum.value() {
        gain_branch = GainBranch::GainFallsToEnvelope;
        target = attenuation;
        working_hold = 0;
        smoothed = maximum;
    } else if working_hold < parameters.hold_limit {
        gain_branch = GainBranch::HoldCounterIncrement;
        working_hold = working_hold
            .checked_add(1)
            .ok_or(ComponentStageError::State)?;
    } else {
        gain_branch = GainBranch::TargetRecurrenceAndSmoothedClamp;
        let attenuation_term =
            r32_mul(parameters.coefficients[4], attenuation).map_err(map_numeric_error)?;
        let target_retain = r32_sub(attenuation, attenuation_term).map_err(map_numeric_error)?;
        let target_smooth =
            r32_mul(parameters.coefficients[4], old_target).map_err(map_numeric_error)?;
        target = r32_add(target_retain, target_smooth).map_err(map_numeric_error)?;

        let candidate = r32_mul(target, old_envelope).map_err(map_numeric_error)?;
        if candidate.value() > smoothed.value() {
            smoothed = candidate;
        }
        if smoothed.value() > maximum.value() {
            smoothed = maximum;
        }
    }

    let envelope = if target.value() < parameters.target_pivot.value() {
        let term0 = r32_mul(target, old_envelope).map_err(map_numeric_error)?;
        let term1 = r32_mul(term0, parameters.envelope_weight).map_err(map_numeric_error)?;
        let term2 = r32_mul(term1, finite_constant(2.0)?).map_err(map_numeric_error)?;
        let term3 = r32_mul(term2, parameters.coefficients[1]).map_err(map_numeric_error)?;
        let retain =
            r32_mul(old_envelope, parameters.coefficients[0]).map_err(map_numeric_error)?;
        r32_add(term3, retain).map_err(map_numeric_error)?
    } else {
        let retain =
            r32_mul(old_envelope, parameters.coefficients[2]).map_err(map_numeric_error)?;
        r32_add(retain, parameters.coefficients[3]).map_err(map_numeric_error)?
    };

    let blocks_processed = old
        .blocks_processed
        .checked_add(1)
        .ok_or(ComponentStageError::State)?;
    Ok(ParameterTransition {
        after: ComponentFilterParameterState {
            peak: current_peak.value(),
            previous_gain: previous_gain.value(),
            hold: working_hold,
            envelope: envelope.value(),
            smoothed_gain: smoothed.value(),
            target_gain: target.value(),
            output_level: f32::from_bits(0x3b80_0000),
            blocks_processed,
        },
        current_peak,
        scaled_peak,
        attenuation,
        peak_witness,
        gain_branch,
    })
}

fn reciprocal_attenuation(
    scaled_peak: FiniteF32,
    reciprocal_pivot: FiniteF32,
) -> Result<FiniteF32, ComponentStageError> {
    let one = finite_constant(1.0)?;
    if scaled_peak.value() <= one.value() {
        return Ok(one);
    }

    let half = finite_constant(0.5)?;
    let mut normalized = scaled_peak;
    let mut exponent: i32 = 0;
    while normalized.value() >= one.value() {
        normalized = r32_mul(normalized, half).map_err(map_numeric_error)?;
        exponent = exponent
            .checked_sub(1)
            .ok_or(ComponentStageError::NumericFailure)?;
    }

    let biased_exponent = exponent
        .checked_add(128)
        .ok_or(ComponentStageError::NumericFailure)?;
    if !(1..=254).contains(&biased_exponent) {
        return Err(ComponentStageError::NumericFailure);
    }
    let exponent_bits =
        u32::try_from(biased_exponent).map_err(|_| ComponentStageError::NumericFailure)?;
    let scale = FiniteF32::new(f32::from_bits(exponent_bits << 23))
        .map_err(|_| ComponentStageError::NumericFailure)?;

    let mut reciprocal = if normalized.value() <= reciprocal_pivot.value() {
        let difference = r32_sub(one, normalized).map_err(map_numeric_error)?;
        r32_mul(difference, finite_constant(2.0)?).map_err(map_numeric_error)?
    } else {
        let half_n = r32_mul(half, normalized).map_err(map_numeric_error)?;
        r32_sub(one, half_n).map_err(map_numeric_error)?
    };

    for _ in 0..4 {
        let product = r32_mul(reciprocal, normalized).map_err(map_numeric_error)?;
        let difference = r32_sub(half, product).map_err(map_numeric_error)?;
        let correction0 = r32_mul(difference, reciprocal).map_err(map_numeric_error)?;
        let correction = r32_mul(correction0, finite_constant(2.0)?).map_err(map_numeric_error)?;
        reciprocal = r32_add(reciprocal, correction).map_err(map_numeric_error)?;
    }

    let reciprocal_scaled = r32_mul(one, reciprocal).map_err(map_numeric_error)?;
    r32_mul(reciprocal_scaled, scale).map_err(map_numeric_error)
}

fn finite_state(value: f32) -> Result<FiniteF32, ComponentStageError> {
    FiniteF32::new(value).map_err(|_| ComponentStageError::State)
}

fn max_first(left: FiniteF32, right: FiniteF32) -> FiniteF32 {
    if right.value() > left.value() {
        right
    } else {
        left
    }
}

fn min_first(left: FiniteF32, right: FiniteF32) -> FiniteF32 {
    if right.value() < left.value() {
        right
    } else {
        left
    }
}

fn filter_component(
    source: &[FiniteF32],
    old_parameters: &ComponentFilterParameterState,
    old_history: &[RawBytes; COMPONENT_ROW_COUNT],
    kernel: &[FiniteF32; BLOCK_SAMPLES],
    controller: &ComponentControllerParameters,
) -> Result<FilterStage, ComponentStageError> {
    if source.len() != COMPONENT_WORDS {
        return Err(ComponentStageError::State);
    }
    let mut history = Vec::with_capacity(COMPONENT_ROW_COUNT * BLOCK_SAMPLES);
    for row in old_history {
        for sample in 0..BLOCK_SAMPLES {
            history.push(raw_finite_at(row, sample)?);
        }
    }
    let mut history = history.into_boxed_slice();
    let mut output = vec![finite_constant(0.0)?; COMPONENT_WORDS].into_boxed_slice();
    let mut parameters = copy_parameter_state(old_parameters);
    for block in ComponentBlockIndex::all() {
        let offset = block.sample_offset();
        let mut block_input = vec![finite_constant(0.0)?; COMPONENT_ROW_COUNT * BLOCK_SAMPLES];
        for row in 0..COMPONENT_ROW_COUNT {
            for sample in 0..BLOCK_SAMPLES {
                block_input[row * BLOCK_SAMPLES + sample] =
                    source[row * FRAME_SAMPLES + offset + sample];
            }
        }
        let next_parameters = update_parameters(&parameters, &block_input, controller)?.after;
        let old_output_level = finite_state(parameters.output_level)?;
        let next_previous_gain = finite_state(next_parameters.previous_gain)?;
        let next_smoothed_gain = finite_state(next_parameters.smoothed_gain)?;
        let block_output = filter_component_block(
            &block_input,
            &mut history,
            &FilterBlockSettings {
                kernel,
                output_level: old_output_level,
                previous_gain: next_previous_gain,
                smoothed_gain: next_smoothed_gain,
                gain_scale: controller.gain_scale,
                clamp_limit: controller.clamp_limit,
            },
        )?;
        for row in 0..COMPONENT_ROW_COUNT {
            let block_row_start = row * BLOCK_SAMPLES;
            let output_row_start = row * FRAME_SAMPLES + offset;
            output[output_row_start..output_row_start + BLOCK_SAMPLES].copy_from_slice(
                &block_output.samples[block_row_start..block_row_start + BLOCK_SAMPLES],
            );
        }
        parameters = next_parameters;
    }

    let mut next_history = Vec::with_capacity(COMPONENT_ROW_COUNT);
    for row in 0..COMPONENT_ROW_COUNT {
        let start = row * BLOCK_SAMPLES;
        next_history.push(raw_bytes_from_finite(
            &history[start..start + BLOCK_SAMPLES],
        ));
    }
    let next_history = next_history
        .try_into()
        .map_err(|_| ComponentStageError::State)?;
    Ok(FilterStage {
        samples: output,
        parameters,
        history: next_history,
    })
}

fn filter_component_block(
    source: &[FiniteF32],
    history: &mut [FiniteF32],
    settings: &FilterBlockSettings<'_>,
) -> Result<ComponentFilterBlock, ComponentStageError> {
    let block_words = COMPONENT_ROW_COUNT * BLOCK_SAMPLES;
    if source.len() != block_words || history.len() != block_words {
        return Err(ComponentStageError::Input);
    }
    let alpha =
        r32_mul(settings.output_level, settings.previous_gain).map_err(map_numeric_error)?;
    let beta = r32_mul(settings.smoothed_gain, settings.gain_scale).map_err(map_numeric_error)?;
    let two_fifty_six = finite_constant(256.0)?;
    let scale = r32_mul(two_fifty_six, settings.gain_scale).map_err(map_numeric_error)?;
    let negative_limit =
        FiniteF32::new(-settings.clamp_limit.value()).map_err(|_| ComponentStageError::State)?;
    let mut preclip = vec![finite_constant(0.0)?; block_words].into_boxed_slice();
    let mut samples = vec![finite_constant(0.0)?; block_words].into_boxed_slice();

    for row in 0..COMPONENT_ROW_COUNT {
        for sample in 0..BLOCK_SAMPLES {
            let index = row * BLOCK_SAMPLES + sample;
            let old_history = history[index];
            let history_beta = r32_mul(old_history, beta).map_err(map_numeric_error)?;
            let a = r32_mul(settings.kernel[sample], history_beta).map_err(map_numeric_error)?;
            let history_alpha = r32_mul(old_history, alpha).map_err(map_numeric_error)?;
            let b = r32_mul(settings.kernel[BLOCK_SAMPLES - 1 - sample], history_alpha)
                .map_err(map_numeric_error)?;
            let sum = r32_add(a, b).map_err(map_numeric_error)?;
            let intermediate = r32_mul(sum, two_fifty_six).map_err(map_numeric_error)?;
            preclip[index] = intermediate;
            let upper_clamped = min_first(settings.clamp_limit, intermediate);
            let clamped = max_first(negative_limit, upper_clamped);
            samples[index] = r32_mul(clamped, scale).map_err(map_numeric_error)?;
            history[index] = source[index];
        }
    }
    Ok(ComponentFilterBlock { preclip, samples })
}

fn copy_parameter_state(state: &ComponentFilterParameterState) -> ComponentFilterParameterState {
    ComponentFilterParameterState {
        peak: state.peak,
        previous_gain: state.previous_gain,
        hold: state.hold,
        envelope: state.envelope,
        smoothed_gain: state.smoothed_gain,
        target_gain: state.target_gain,
        output_level: state.output_level,
        blocks_processed: state.blocks_processed,
    }
}

fn apply_source_ring(
    before: &ComponentAudioState,
    next_overlap: [RawBytes; COMPONENT_ROW_COUNT],
    filtered: &[FiniteF32],
) -> Result<(Box<[FiniteF32]>, ComponentAudioState), ComponentStageError> {
    if filtered.len() != COMPONENT_WORDS {
        return Err(ComponentStageError::State);
    }
    let mut component_output = vec![finite_constant(0.0)?; COMPONENT_WORDS].into_boxed_slice();
    let mut next_ring = Vec::with_capacity(COMPONENT_ROW_COUNT);
    for row in 0..COMPONENT_ROW_COUNT {
        let row_offset = row * FRAME_SAMPLES;
        for sample in 0..RING_SAMPLES {
            component_output[row_offset + sample] =
                raw_finite_at(&before.source_ring_rows[row], sample)?;
        }
        component_output[row_offset + RING_SAMPLES..row_offset + FRAME_SAMPLES]
            .copy_from_slice(&filtered[row_offset..row_offset + BLOCK_SAMPLES]);
        next_ring.push(raw_bytes_from_finite(
            &filtered[row_offset + BLOCK_SAMPLES..row_offset + FRAME_SAMPLES],
        ));
    }
    let next_ring = next_ring
        .try_into()
        .map_err(|_| ComponentStageError::State)?;
    Ok((
        component_output,
        ComponentAudioState {
            overlap_history_rows: next_overlap,
            source_ring_rows: next_ring,
        },
    ))
}

#[cfg(test)]
pub(in crate::spatial::spatial_audio_state) mod tests;
