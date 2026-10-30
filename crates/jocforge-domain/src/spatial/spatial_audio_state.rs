// pattern: Functional Core
//! Typed F04 cold state and staging capabilities. No DSP or local commit authority lives here.
use crate::{
    profile::{EncodingProfile, resolve_selected_profile},
    spatial::{
        F04ParameterProviderLineage, ValidatedComponentParameterProvider,
        ValidatedWindowQ15Provider,
        post_window_fanout::CallbackPhase,
        source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
        source_geometry_outer::CompactOuterMatrix,
        spatial_control::{SpatialControlJoinMetadata, SpatialControlMachine},
        target_context::TargetSlotIndex,
    },
    timeline::{FrontendCallbackIndex, ReceiverCallbackCount},
    transaction::{
        CallbackLineageIdentity, CallbackTransactionId, GtcOwnerId, GtcOwnerReceipt,
        GtcOwnerReceiptError, GtcOwnerStateSnapshot,
    },
};
use sha2::{Digest, Sha256};
use std::fmt::Write as FmtWrite;
use std::sync::Arc;

pub(crate) mod component_stage;
pub(crate) mod descriptor_control;
pub(crate) mod direct_lfe;
pub(crate) mod fitting_mode0;
pub(crate) mod receiver_mode1;
pub(crate) mod spatial_audio_composer;
pub(crate) mod spatial_audio_receiver;
pub(crate) mod type10_assembler;

pub use component_stage::ComponentStageError;
pub use fitting_mode0::SelectedJocQmfProjection;

const EXPECTED_COLD_BYTES: usize = 813_733;
const NEGATIVE_ZERO_BITS: u32 = 0x8000_0000;
const SPATIAL_AUDIO_OWNER_ID: &str = "spatial_audio";
const SPATIAL_AUDIO_STATE_REF: &str = "owners.spatial_audio";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialAudioError {
    UnsupportedProfile,
    ProfileBindingMismatch,
    GenerationMismatch,
    BeforeDigestMismatch,
    NonFiniteFloat,
    NegativeZeroColdFloat,
    EncodingLengthOverflow,
    EncodedColdLengthMismatch,
    NonAsciiNodeName,
    UnknownNodeClass,
    UnknownFieldName,
    NodeFieldCountOverflow,
    DeclaredFieldCountMismatch,
    ColdValueMismatch,
    GtcReceipt(GtcOwnerReceiptError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialAudioDigest([u8; 32]);

/// Read-only GTC state-reference view over the complete SpatialAudio root.
///
/// `state_ref` is a logical owner key in the enclosing programme snapshot. The
/// byte slice is the existing full `F04CompositionState` encoding; it excludes
/// immutable profile admission metadata and carries no parent receipt/commit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialAudioOwnerStateReference<'a> {
    generation: u64,
    state_sha256: SpatialAudioDigest,
    cold_sha256: SpatialAudioDigest,
    canonical_bytes: &'a [u8],
}

impl<'a> SpatialAudioOwnerStateReference<'a> {
    pub const fn owner_id(&self) -> &'static str {
        SPATIAL_AUDIO_OWNER_ID
    }

    pub const fn state_ref(&self) -> &'static str {
        SPATIAL_AUDIO_STATE_REF
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn state_sha256(&self) -> SpatialAudioDigest {
        self.state_sha256
    }

    pub const fn cold_sha256(&self) -> SpatialAudioDigest {
        self.cold_sha256
    }

    pub fn canonical_bytes(&self) -> &'a [u8] {
        self.canonical_bytes
    }
}

const METER_COMPONENT_ROW_COUNT: usize = 15;
const METER_COMPONENT_SAMPLE_COUNT: usize = 1_536;
const METER_INSTALLED_SAMPLE_COUNT: usize = 3_072;

/// F04A component output paired with the exact pre-quiet C/d callback that produced it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterComponentOutputRows {
    callback_index: FrontendCallbackIndex,
    transaction_id: CallbackLineageIdentity,
    phase: CallbackPhase,
    active_target_slots: Box<[TargetSlotIndex]>,
    canonical_rows_f32_bits: Box<[u32]>,
}

impl MeterComponentOutputRows {
    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub const fn phase(&self) -> CallbackPhase {
        self.phase
    }

    pub fn active_target_slots(&self) -> &[TargetSlotIndex] {
        &self.active_target_slots
    }

    pub const fn canonical_rows_f32_bits(&self) -> &[u32] {
        &self.canonical_rows_f32_bits
    }

    pub fn row_for_target_slot(&self, target_slot: TargetSlotIndex) -> Option<&[u32]> {
        let slot = usize::from(target_slot.value());
        if slot >= METER_COMPONENT_ROW_COUNT {
            return None;
        }
        let start = slot.checked_mul(METER_COMPONENT_SAMPLE_COUNT)?;
        self.canonical_rows_f32_bits
            .get(start..start + METER_COMPONENT_SAMPLE_COUNT)
    }

    #[cfg(test)]
    pub(crate) fn fixture(
        callback_index: FrontendCallbackIndex,
        transaction_id: CallbackLineageIdentity,
        phase: CallbackPhase,
        active_target_slots: Vec<TargetSlotIndex>,
    ) -> Self {
        let mut canonical_rows_f32_bits =
            Vec::with_capacity(METER_COMPONENT_ROW_COUNT * METER_COMPONENT_SAMPLE_COUNT);
        for slot in 0..METER_COMPONENT_ROW_COUNT {
            canonical_rows_f32_bits.extend(std::iter::repeat_n(
                (slot as f32 + 1.0).to_bits(),
                METER_COMPONENT_SAMPLE_COUNT,
            ));
        }
        Self {
            callback_index,
            transaction_id,
            phase,
            active_target_slots: active_target_slots.into_boxed_slice(),
            canonical_rows_f32_bits: canonical_rows_f32_bits.into_boxed_slice(),
        }
    }
}

/// Immutable 15-slot overlap history snapshot used by marker energy before component execution.
#[derive(Clone, Debug, PartialEq)]
pub struct ComponentBeforeOverlapRows {
    callback_index: FrontendCallbackIndex,
    rows_f32: Box<[f32]>,
}

impl ComponentBeforeOverlapRows {
    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub fn row_for_target_slot(&self, target_slot: TargetSlotIndex) -> Option<&[f32]> {
        let row = usize::from(target_slot.value());
        if row >= METER_COMPONENT_ROW_COUNT {
            return None;
        }
        let start = row.checked_mul(METER_COMPONENT_SAMPLE_COUNT)?;
        self.rows_f32
            .get(start..start + METER_COMPONENT_SAMPLE_COUNT)
    }

    pub fn rows(&self) -> [&[f32]; METER_COMPONENT_ROW_COUNT] {
        std::array::from_fn(|row| {
            let start = row * METER_COMPONENT_SAMPLE_COUNT;
            &self.rows_f32[start..start + METER_COMPONENT_SAMPLE_COUNT]
        })
    }
}

/// Pure marker-energy preview from component.before and the current C/d mixer input.
/// Consuming this preview later advances the existing component owner exactly once.
#[must_use]
pub struct MeterComponentStagePreview {
    packet: component_stage::SpatialAudioInputPacket,
    mixer_preview: Box<[crate::numeric::FiniteF32]>,
    matrix: CompactOuterMatrix,
    active_target_slots: Box<[TargetSlotIndex]>,
    before_generation: u64,
    before_digest: SpatialAudioDigest,
    callback_index: FrontendCallbackIndex,
    transaction_id: CallbackLineageIdentity,
    phase: component_stage::ComponentCallbackPhase,
    current_mixer_rows_f32: Box<[f32]>,
    canonical_dense_matrix_f32_bits: Box<[u32]>,
    component_before_overlap: ComponentBeforeOverlapRows,
}

impl MeterComponentStagePreview {
    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub fn active_target_slots(&self) -> &[TargetSlotIndex] {
        &self.active_target_slots
    }

    pub fn previous_overlap_rows(&self) -> [&[f32]; METER_COMPONENT_ROW_COUNT] {
        self.component_before_overlap.rows()
    }

    pub const fn component_before_overlap(&self) -> &ComponentBeforeOverlapRows {
        &self.component_before_overlap
    }

    pub fn current_mixer_rows(&self) -> [&[f32]; METER_COMPONENT_ROW_COUNT] {
        std::array::from_fn(|row| {
            let start = row * METER_INSTALLED_SAMPLE_COUNT;
            &self.current_mixer_rows_f32[start..start + METER_INSTALLED_SAMPLE_COUNT]
        })
    }

    pub fn compact_matrix_f32(&self) -> Vec<f32> {
        self.matrix
            .c_rows_u32()
            .iter()
            .flat_map(|row| row.iter().copied().map(f32::from_bits))
            .collect()
    }

    /// Canonical D[15][48] from the same C/d adapter used by marker preview and F04A.
    pub const fn canonical_dense_matrix_f32_bits(&self) -> &[u32] {
        &self.canonical_dense_matrix_f32_bits
    }

    #[allow(clippy::too_many_arguments)]
    pub fn execute(
        self,
        spatial_audio: &SpatialAudioMachine,
        spatial_audio_join: &SpatialAudioJoinMetadata,
        spatial_control_join: &SpatialControlJoinMetadata,
        spatial_control: &SpatialControlMachine,
        window: &ValidatedWindowQ15Provider,
        parameters: &ValidatedComponentParameterProvider,
    ) -> Result<MeterComponentStageProposal, component_stage::ComponentStageError> {
        if spatial_audio.generation() != self.before_generation
            || spatial_audio.digest() != self.before_digest
            || spatial_audio_join.callback_index() != self.callback_index
            || spatial_audio_join.transaction_id() != &self.transaction_id
            || spatial_control_join.callback_index() != self.callback_index
            || spatial_control_join.transaction_id() != &self.transaction_id
        {
            return Err(component_stage::ComponentStageError::State);
        }
        let component = component_stage::produce_component_stage_with_preview(
            spatial_audio,
            spatial_audio_join,
            spatial_control_join,
            spatial_control,
            self.phase,
            self.packet,
            window,
            parameters,
            self.mixer_preview,
        )?;
        let meter_rows = component_output_rows_for_meter(&component, &self.matrix)?;
        Ok(MeterComponentStageProposal {
            component,
            meter_rows,
        })
    }
}

/// Exports the already-staged F04A output only when its exact C/d matrix matches the F03S result.
pub(crate) fn component_output_rows_for_meter(
    proposal: &component_stage::ComponentStageProposal,
    matrix: &CompactOuterMatrix,
) -> Result<MeterComponentOutputRows, component_stage::ComponentStageError> {
    let (component_output, receipt) = proposal.meter_view();
    let destinations = matrix
        .destinations()
        .iter()
        .map(|destination| destination.target_slot().value())
        .collect::<Vec<_>>();
    if receipt.callback_index() != matrix.value_generation()
        || receipt.transaction_id() != matrix.transaction_id()
        || receipt.matrix_destination_slots() != destinations
        || matrix.c_rows_u32().len() != destinations.len()
        || destinations.windows(2).any(|pair| pair[0] >= pair[1])
        || component_output.len() != METER_COMPONENT_ROW_COUNT * METER_COMPONENT_SAMPLE_COUNT
    {
        return Err(component_stage::ComponentStageError::Input);
    }

    let active_target_slots = destinations
        .into_iter()
        .map(TargetSlotIndex::from_validated)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let canonical_rows_f32_bits = component_output
        .iter()
        .map(|sample| sample.value().to_bits())
        .collect::<Vec<_>>()
        .into_boxed_slice();
    Ok(MeterComponentOutputRows {
        callback_index: receipt.callback_index(),
        transaction_id: receipt.transaction_id().clone(),
        phase: match receipt.phase() {
            component_stage::ComponentCallbackPhase::Normal => CallbackPhase::Normal,
            component_stage::ComponentCallbackPhase::CachedDrain => CallbackPhase::CachedDrain,
            component_stage::ComponentCallbackPhase::FreshZeroInputDrain => {
                CallbackPhase::FreshZeroInputDrain
            }
        },
        active_target_slots,
        canonical_rows_f32_bits,
    })
}

const fn callback_phase_for_component(
    phase: component_stage::ComponentCallbackPhase,
) -> CallbackPhase {
    match phase {
        component_stage::ComponentCallbackPhase::Normal => CallbackPhase::Normal,
        component_stage::ComponentCallbackPhase::CachedDrain => CallbackPhase::CachedDrain,
        component_stage::ComponentCallbackPhase::FreshZeroInputDrain => {
            CallbackPhase::FreshZeroInputDrain
        }
    }
}

/// Noncommitting F04A result paired with its canonical component rows for the meter adapter.
#[must_use]
pub struct MeterComponentStageProposal {
    #[allow(dead_code)] // The enclosing callback parent consumes this only at its commit point.
    component: component_stage::ComponentStageProposal,
    meter_rows: MeterComponentOutputRows,
}

impl MeterComponentStageProposal {
    pub const fn meter_rows(&self) -> &MeterComponentOutputRows {
        &self.meter_rows
    }

    pub const fn blocks_processed(&self) -> i64 {
        self.component.blocks_processed()
    }

    #[allow(dead_code)] // The parent commit consumer is intentionally outside this producer slice.
    pub(crate) fn into_parts(
        self,
    ) -> (
        component_stage::ComponentStageProposal,
        MeterComponentOutputRows,
    ) {
        (self.component, self.meter_rows)
    }
}

/// Noncommitting Meter callback result before F04C/Type10 receiver work.
#[must_use]
pub struct MeterSpatialAudioCallbackProposal {
    component_rows: MeterComponentOutputRows,
    spatial_audio_after: SpatialAudioAfterProposal,
}

impl MeterSpatialAudioCallbackProposal {
    pub const fn component_rows(&self) -> &MeterComponentOutputRows {
        &self.component_rows
    }

    pub const fn spatial_audio_after(&self) -> &SpatialAudioAfterProposal {
        &self.spatial_audio_after
    }

    pub fn into_parts(self) -> (SpatialAudioAfterProposal, MeterComponentOutputRows) {
        (self.spatial_audio_after, self.component_rows)
    }
}

impl SpatialAudioMachine {
    /// Builds a Meter-only F04A after-root from the staged component result.
    ///
    /// This proposal advances the callback clock and component overlap, ring,
    /// parameter, and filter-history leaves. Receiver, fitting-QMF, LFE, and
    /// receiver callback-count leaves remain copied from the before-root.
    pub fn stage_meter_component_after(
        &self,
        expected_join: &SpatialAudioJoinMetadata,
        expected_provider_lineage: [F04ParameterProviderLineage; 2],
        proposal: MeterComponentStageProposal,
    ) -> Result<MeterSpatialAudioCallbackProposal, component_stage::ComponentStageError> {
        let (component, component_rows) = proposal.into_parts();
        let (component_after, _, receipt) = component.into_parts();

        if receipt.spatial_audio_before_generation() != self.generation
            || receipt.spatial_audio_before_digest() != self.digest()
            || receipt.callback_index() != self.committed.callback_index()
            || expected_join.programme_id() != receipt.programme_id()
            || expected_join.attempt() != receipt.attempt()
            || expected_join.transaction_id() != receipt.transaction_id()
            || expected_join.callback_index() != receipt.callback_index()
            || expected_join.before_generation() != receipt.spatial_audio_before_generation()
            || expected_join.before_digest() != receipt.spatial_audio_before_digest()
            || expected_join.profile_identity() != receipt.profile_identity()
            || receipt.profile_identity() != self.profile_identity
            || receipt.provider_lineage() != expected_provider_lineage
            || component_rows.callback_index() != receipt.callback_index()
            || component_rows.transaction_id() != receipt.transaction_id()
            || component_rows.phase() != callback_phase_for_component(receipt.phase())
        {
            return Err(ComponentStageError::State);
        }

        let next_callback = receipt
            .callback_index()
            .value()
            .checked_add(1)
            .map(FrontendCallbackIndex::new)
            .ok_or(ComponentStageError::CallbackOrder)?;

        let mut state = self.committed.staged_copy();
        state.root.callback_index = next_callback;
        state.root.component_state = component_after.audio;
        state.root.component_parameter_state = component_after.parameters;
        state.root.component_filter_history = component_after.filter_history;
        let state =
            SpatialAudioState::from_root(state.root).map_err(|_| ComponentStageError::State)?;

        let parent = SpatialAudioJoinMetadata::new(
            receipt.programme_id().clone(),
            receipt.attempt(),
            receipt.transaction_id().clone(),
            receipt.callback_index(),
            receipt.spatial_audio_before_generation(),
            receipt.spatial_audio_before_digest(),
            receipt.profile_identity(),
        );

        Ok(MeterSpatialAudioCallbackProposal {
            component_rows,
            spatial_audio_after: SpatialAudioAfterProposal::new(state, parent),
        })
    }

    /// Copies the committed overlap rows for F05R56 energy preview without advancing F04A state.
    pub fn component_before_overlap_rows(
        &self,
    ) -> Result<ComponentBeforeOverlapRows, component_stage::ComponentStageError> {
        let mut rows_f32 =
            Vec::with_capacity(METER_COMPONENT_ROW_COUNT * METER_COMPONENT_SAMPLE_COUNT);
        for row in &self.committed.root.component_state.overlap_history_rows {
            if row.bytes.len() != METER_COMPONENT_SAMPLE_COUNT * 4 {
                return Err(component_stage::ComponentStageError::State);
            }
            for word in row.bytes.chunks_exact(4) {
                let bits = u32::from_le_bytes(
                    word.try_into()
                        .map_err(|_| component_stage::ComponentStageError::State)?,
                );
                let value = f32::from_bits(bits);
                if !value.is_finite() {
                    return Err(component_stage::ComponentStageError::State);
                }
                rows_f32.push(value);
            }
        }
        Ok(ComponentBeforeOverlapRows {
            callback_index: self.committed.root.callback_index,
            rows_f32: rows_f32.into_boxed_slice(),
        })
    }

    /// Prepares the pure current-C/d mixer preview and immutable component.before overlap rows.
    /// This advances no component, filter, or overlap state.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_component_stage_preview(
        &self,
        spatial_audio_join: &SpatialAudioJoinMetadata,
        spatial_control_join: &SpatialControlJoinMetadata,
        spatial_control: &SpatialControlMachine,
        phase: CallbackPhase,
        window: &ValidatedWindowQ15Provider,
        parameters: &ValidatedComponentParameterProvider,
        installed48_f32_bits: Vec<u32>,
        matrix: &CompactOuterMatrix,
    ) -> Result<MeterComponentStagePreview, component_stage::ComponentStageError> {
        if matrix.value_generation() != spatial_audio_join.callback_index()
            || matrix.transaction_id() != spatial_audio_join.transaction_id()
            || matrix.value_generation() != spatial_control_join.callback_index()
            || matrix.transaction_id() != spatial_control_join.transaction_id()
        {
            return Err(component_stage::ComponentStageError::Input);
        }
        let component_before_overlap = self.component_before_overlap_rows()?;
        if component_before_overlap.callback_index() != matrix.value_generation() {
            return Err(component_stage::ComponentStageError::CallbackOrder);
        }
        let coefficient_bits = matrix
            .c_rows_u32()
            .iter()
            .flat_map(|row| row.iter().copied())
            .collect::<Vec<_>>();
        let target_slots = matrix
            .destinations()
            .iter()
            .map(|destination| destination.target_slot().value())
            .collect::<Vec<_>>();
        let active_matrix = component_stage::ActiveComponentMatrix::from_producer_output(
            matrix.value_generation(),
            coefficient_bits,
            target_slots,
        )?;
        let canonical_dense_matrix_f32_bits = active_matrix.scatter_to_canonical_dense();
        let component_phase = match phase {
            CallbackPhase::Normal => component_stage::ComponentCallbackPhase::Normal,
            CallbackPhase::CachedDrain => component_stage::ComponentCallbackPhase::CachedDrain,
            CallbackPhase::FreshZeroInputDrain => {
                component_stage::ComponentCallbackPhase::FreshZeroInputDrain
            }
        };
        let packet = component_stage::SpatialAudioInputPacket::from_current_component_callback(
            self,
            spatial_audio_join,
            spatial_control_join,
            spatial_control,
            component_phase,
            window,
            parameters,
            installed48_f32_bits,
            active_matrix,
        )?;
        let mixer_preview = component_stage::preview_component_mixer(&packet)?;
        let current_mixer_rows_f32 = mixer_preview
            .iter()
            .map(|sample| sample.value())
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let active_target_slots = matrix
            .destinations()
            .iter()
            .map(|destination| destination.target_slot())
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Ok(MeterComponentStagePreview {
            packet,
            mixer_preview,
            matrix: matrix.clone(),
            before_generation: self.generation,
            before_digest: self.digest(),
            callback_index: matrix.value_generation(),
            transaction_id: matrix.transaction_id().clone(),
            phase: component_phase,
            active_target_slots,
            current_mixer_rows_f32,
            canonical_dense_matrix_f32_bits,
            component_before_overlap,
        })
    }

    /// Executes F04A once from the prepared marker preview and the same owner-before state.
    #[allow(clippy::too_many_arguments)]
    pub fn stage_component_for_meter(
        &self,
        spatial_audio_join: &SpatialAudioJoinMetadata,
        spatial_control_join: &SpatialControlJoinMetadata,
        spatial_control: &SpatialControlMachine,
        phase: CallbackPhase,
        window: &ValidatedWindowQ15Provider,
        parameters: &ValidatedComponentParameterProvider,
        installed48_f32_bits: Vec<u32>,
        matrix: &CompactOuterMatrix,
    ) -> Result<MeterComponentStageProposal, component_stage::ComponentStageError> {
        let preview = self.prepare_component_stage_preview(
            spatial_audio_join,
            spatial_control_join,
            spatial_control,
            phase,
            window,
            parameters,
            installed48_f32_bits,
            matrix,
        )?;
        preview.execute(
            self,
            spatial_audio_join,
            spatial_control_join,
            spatial_control,
            window,
            parameters,
        )
    }
}

impl SpatialAudioDigest {
    fn from_canonical_bytes(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        let mut value = [0_u8; 32];
        value.copy_from_slice(&digest);
        Self(value)
    }

    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }

    pub fn to_hex(self) -> String {
        let mut value = String::with_capacity(64);
        for byte in self.0 {
            write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
        }
        value
    }
}

/// Complete F04 cold root with private fields and immutable canonical bytes.
///
/// ```compile_fail
/// use jocforge_domain::spatial::SpatialAudioState;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<SpatialAudioState>();
/// ```
#[derive(Debug, PartialEq)]
pub struct SpatialAudioState {
    root: F04CompositionState,
    canonical_bytes: Box<[u8]>,
    digest: SpatialAudioDigest,
}

impl SpatialAudioState {
    fn new_cold() -> Result<Self, SpatialAudioError> {
        let root = F04CompositionState::cold();
        root.validate_cold()?;
        let state = Self::from_root(root)?;
        if state.canonical_bytes.len() != EXPECTED_COLD_BYTES {
            return Err(SpatialAudioError::EncodedColdLengthMismatch);
        }
        Ok(state)
    }

    fn from_root(root: F04CompositionState) -> Result<Self, SpatialAudioError> {
        let mut encoder = F04StateEncoder::with_capacity(EXPECTED_COLD_BYTES);
        root.encode(&mut encoder)?;
        let canonical_bytes = encoder.finish();
        let digest = SpatialAudioDigest::from_canonical_bytes(&canonical_bytes);
        Ok(Self {
            root,
            canonical_bytes: canonical_bytes.into_boxed_slice(),
            digest,
        })
    }

    fn staged_copy(&self) -> Self {
        Self {
            root: self.root.clone(),
            canonical_bytes: self.canonical_bytes.clone(),
            digest: self.digest,
        }
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub const fn digest(&self) -> SpatialAudioDigest {
        self.digest
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.root.callback_index
    }

    pub const fn receiver_callback_count(&self) -> ReceiverCallbackCount {
        self.root.receiver_callback_count
    }

    #[cfg(test)]
    fn validate_cold(&self) -> Result<(), SpatialAudioError> {
        self.root.validate_cold()
    }
}

/// One committed root owner. It exposes read-only state and no local lifecycle operation.
///
/// ```compile_fail
/// use jocforge_domain::spatial::SpatialAudioMachine;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<SpatialAudioMachine>();
/// ```
/// ```compile_fail
/// use jocforge_domain::spatial::{SpatialAudioMachine, SpatialAudioStaged};
/// fn try_local_commit(machine: &mut SpatialAudioMachine, staged: SpatialAudioStaged) {
///     machine.commit(staged);
/// }
/// ```
/// ```compile_fail
/// use jocforge_domain::spatial::SpatialAudioMachine;
/// fn try_mutate_committed(machine: &mut SpatialAudioMachine) {
///     let _ = machine.committed_mut();
/// }
/// ```
#[must_use]
#[derive(Debug)]
pub struct SpatialAudioMachine {
    committed: SpatialAudioState,
    generation: u64,
    cold_digest: SpatialAudioDigest,
    profile_identity: EncodingProfile,
}

impl SpatialAudioMachine {
    pub fn new_cold(profile_identity: EncodingProfile) -> Result<Self, SpatialAudioError> {
        resolve_selected_profile(&profile_identity)
            .map_err(|_| SpatialAudioError::UnsupportedProfile)?;
        let committed = SpatialAudioState::new_cold()?;
        let cold_digest = committed.digest();
        Ok(Self {
            committed,
            generation: 0,
            cold_digest,
            profile_identity,
        })
    }

    pub const fn committed(&self) -> &SpatialAudioState {
        &self.committed
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn digest(&self) -> SpatialAudioDigest {
        self.committed.digest()
    }

    /// Returns a read-only GTC owner-state snapshot for the complete root.
    pub fn state_reference(&self) -> SpatialAudioOwnerStateReference<'_> {
        SpatialAudioOwnerStateReference {
            generation: self.generation,
            state_sha256: self.committed.digest(),
            cold_sha256: self.cold_digest,
            canonical_bytes: self.committed.canonical_bytes(),
        }
    }

    pub const fn profile_identity(&self) -> EncodingProfile {
        self.profile_identity
    }

    pub fn prepare(
        &self,
        proposal: SpatialAudioAfterProposal,
    ) -> Result<SpatialAudioPrepared, SpatialAudioError> {
        self.validate_join(proposal.join_metadata())?;
        Ok(SpatialAudioPrepared {
            proposal,
            before_generation: self.generation,
            before_digest: self.committed.digest(),
        })
    }

    pub fn stage(
        &self,
        prepared: SpatialAudioPrepared,
    ) -> Result<SpatialAudioStaged, SpatialAudioError> {
        if prepared.before_generation != self.generation {
            return Err(SpatialAudioError::GenerationMismatch);
        }
        if prepared.before_digest != self.committed.digest() {
            return Err(SpatialAudioError::BeforeDigestMismatch);
        }
        self.validate_join(prepared.proposal.join_metadata())?;
        let after_digest = prepared.proposal.state.digest();
        Ok(SpatialAudioStaged {
            proposal: prepared.proposal,
            before_generation: prepared.before_generation,
            before_digest: prepared.before_digest,
            after_digest,
        })
    }

    /// Builds a new owner value and GTC receipt from the exact staged root without changing this
    /// machine. The application parent installs it only with the other callback owner candidates.
    pub fn replacement_candidate(
        &self,
        staged: SpatialAudioStaged,
        transaction_id: CallbackTransactionId,
    ) -> Result<SpatialAudioReplacementCandidate, SpatialAudioError> {
        if staged.before_generation != self.generation {
            return Err(SpatialAudioError::GenerationMismatch);
        }
        if staged.before_digest != self.committed.digest() {
            return Err(SpatialAudioError::BeforeDigestMismatch);
        }
        self.validate_join(staged.join_metadata())?;
        let expected_callback = staged
            .join_metadata()
            .callback_index()
            .value()
            .checked_add(1)
            .ok_or(SpatialAudioError::GenerationMismatch)?;
        if staged.after_state().callback_index().value() != expected_callback
            || staged.after_state().digest() != staged.after_digest
        {
            return Err(SpatialAudioError::GenerationMismatch);
        }
        let after_generation = self
            .generation
            .checked_add(1)
            .ok_or(SpatialAudioError::GenerationMismatch)?;
        let (committed, _, before_generation, before_digest) = staged.into_parts();
        if before_generation != self.generation || before_digest != self.committed.digest() {
            return Err(SpatialAudioError::BeforeDigestMismatch);
        }
        let next_machine = Self {
            committed,
            generation: after_generation,
            cold_digest: self.cold_digest,
            profile_identity: self.profile_identity,
        };
        let before_reference = self.state_reference();
        let after_reference = next_machine.state_reference();
        let before = GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::SpatialAudio,
            before_reference.generation(),
            before_reference.cold_sha256().0,
            before_reference.canonical_bytes(),
        )
        .map_err(SpatialAudioError::GtcReceipt)?;
        let after = GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::SpatialAudio,
            after_reference.generation(),
            after_reference.cold_sha256().0,
            after_reference.canonical_bytes(),
        )
        .map_err(SpatialAudioError::GtcReceipt)?;
        let receipt = GtcOwnerReceipt::from_transition(before, after, transaction_id)
            .map_err(SpatialAudioError::GtcReceipt)?;
        Ok(SpatialAudioReplacementCandidate {
            next_machine,
            receipt,
        })
    }

    fn validate_join(&self, join: &SpatialAudioJoinMetadata) -> Result<(), SpatialAudioError> {
        if join.profile_identity != self.profile_identity {
            return Err(SpatialAudioError::ProfileBindingMismatch);
        }
        if join.before_generation != self.generation {
            return Err(SpatialAudioError::GenerationMismatch);
        }
        if join.before_digest != self.committed.digest() {
            return Err(SpatialAudioError::BeforeDigestMismatch);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(in crate::spatial::spatial_audio_state) fn admit_staged_for_test(
        &mut self,
        staged: SpatialAudioStaged,
    ) {
        assert_eq!(staged.before_generation, self.generation);
        assert_eq!(staged.before_digest, self.committed.digest());
        let (state, _, before_generation, _) = staged.into_parts();
        self.committed = state;
        self.generation = before_generation
            .checked_add(1)
            .expect("test callback generation remains bounded");
    }
}

/// Parent-owned replacement value for the complete spatial-audio root.
#[must_use]
#[derive(Debug)]
pub struct SpatialAudioReplacementCandidate {
    next_machine: SpatialAudioMachine,
    receipt: GtcOwnerReceipt,
}

impl SpatialAudioReplacementCandidate {
    pub const fn next_machine(&self) -> &SpatialAudioMachine {
        &self.next_machine
    }

    pub const fn receipt(&self) -> &GtcOwnerReceipt {
        &self.receipt
    }

    pub fn into_parts(self) -> (SpatialAudioMachine, GtcOwnerReceipt) {
        (self.next_machine, self.receipt)
    }
}

/// Opaque full-root proposal for a future F04C producer; this slice adds no producer.
#[must_use]
#[derive(Debug)]
pub struct SpatialAudioAfterProposal {
    state: SpatialAudioState,
    join: SpatialAudioJoinMetadata,
}

impl SpatialAudioAfterProposal {
    #[allow(dead_code)]
    pub(crate) fn new(state: SpatialAudioState, join: SpatialAudioJoinMetadata) -> Self {
        Self { state, join }
    }

    pub const fn after_state(&self) -> &SpatialAudioState {
        &self.state
    }

    fn join_metadata(&self) -> &SpatialAudioJoinMetadata {
        &self.join
    }
}

/// Linear preparation capability consumed by `stage`.
///
/// ```compile_fail
/// use jocforge_domain::spatial::SpatialAudioPrepared;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<SpatialAudioPrepared>();
/// ```
#[must_use]
#[derive(Debug)]
pub struct SpatialAudioPrepared {
    proposal: SpatialAudioAfterProposal,
    before_generation: u64,
    before_digest: SpatialAudioDigest,
}

/// Linear staged full-root result. Only the enclosing callback coordinator may commit it.
///
/// ```compile_fail
/// use jocforge_domain::spatial::SpatialAudioStaged;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<SpatialAudioStaged>();
/// ```
#[must_use]
#[derive(Debug)]
pub struct SpatialAudioStaged {
    proposal: SpatialAudioAfterProposal,
    before_generation: u64,
    before_digest: SpatialAudioDigest,
    after_digest: SpatialAudioDigest,
}

impl SpatialAudioStaged {
    pub const fn before_generation(&self) -> u64 {
        self.before_generation
    }

    pub const fn before_digest(&self) -> SpatialAudioDigest {
        self.before_digest
    }

    pub const fn after_digest(&self) -> SpatialAudioDigest {
        self.after_digest
    }

    pub const fn after_state(&self) -> &SpatialAudioState {
        &self.proposal.state
    }

    pub const fn join_metadata(&self) -> &SpatialAudioJoinMetadata {
        &self.proposal.join
    }

    #[allow(dead_code)]
    pub(crate) fn into_parts(
        self,
    ) -> (
        SpatialAudioState,
        SpatialAudioJoinMetadata,
        u64,
        SpatialAudioDigest,
    ) {
        (
            self.proposal.state,
            self.proposal.join,
            self.before_generation,
            self.before_digest,
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct SpatialAudioJoinMetadata {
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: CallbackLineageIdentity,
    callback_index: FrontendCallbackIndex,
    before_generation: u64,
    before_digest: SpatialAudioDigest,
    profile_identity: EncodingProfile,
}

impl SpatialAudioJoinMetadata {
    #[allow(clippy::too_many_arguments)]
    #[allow(dead_code)]
    pub(crate) fn new(
        programme_id: SourceFlagsProgrammeId,
        attempt: SourceFlagsAttempt,
        transaction_id: CallbackLineageIdentity,
        callback_index: FrontendCallbackIndex,
        before_generation: u64,
        before_digest: SpatialAudioDigest,
        profile_identity: EncodingProfile,
    ) -> Self {
        Self {
            programme_id,
            attempt,
            transaction_id,
            callback_index,
            before_generation,
            before_digest,
            profile_identity,
        }
    }

    pub fn programme_id(&self) -> &SourceFlagsProgrammeId {
        &self.programme_id
    }

    pub const fn attempt(&self) -> SourceFlagsAttempt {
        self.attempt
    }

    pub const fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.transaction_id
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn before_generation(&self) -> u64 {
        self.before_generation
    }

    pub const fn before_digest(&self) -> SpatialAudioDigest {
        self.before_digest
    }

    pub const fn profile_identity(&self) -> EncodingProfile {
        self.profile_identity
    }
}

#[derive(Clone, Debug, PartialEq)]
struct F04CompositionState {
    callback_index: FrontendCallbackIndex,
    retired: bool,
    component_state: ComponentAudioState,
    component_parameter_state: ComponentFilterParameterState,
    component_filter_history: [RawBytes; 15],
    lfe_state: LfeDelayRingState,
    receiver_state: ModelState,
    fitting_qmf_state: QmfWindowState,
    receiver_callback_count: ReceiverCallbackCount,
}

#[derive(Clone, Debug, PartialEq)]
struct ComponentAudioState {
    overlap_history_rows: [RawBytes; 15],
    source_ring_rows: [RawBytes; 15],
}

#[derive(Clone, Debug, PartialEq)]
struct ComponentFilterParameterState {
    peak: f32,
    previous_gain: f32,
    hold: i64,
    envelope: f32,
    smoothed_gain: f32,
    target_gain: f32,
    output_level: f32,
    blocks_processed: i64,
}

#[derive(Clone, Debug, PartialEq)]
struct LfeDelayRingState {
    write_index: i64,
    read_index: i64,
    rows: [Option<Arc<[u8]>>; 3],
    input_frame_indexes: [Option<FrontendCallbackIndex>; 3],
    model_samples_consumed: i64,
}

#[derive(Clone, Debug, PartialEq)]
struct RowState {
    current: [RawWord; 6],
    target: [RawWord; 6],
    slope: [RawWord; 6],
    remaining: [RawWord; 6],
}

/// Immutable receiver accumulator fields needed by the downstream JOC geometry
/// check. This projection owns values only and has no spatial-audio authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::struct_field_names)]
pub(crate) struct JocGeometryRowState {
    current_words: [u32; 6],
    target_words: [u32; 6],
    slope_words: [u32; 6],
    remaining_words: [u32; 6],
}

impl JocGeometryRowState {
    fn from_owner(row: &RowState) -> Self {
        let words = |values: &[RawWord; 6]| values.map(|value| u32::from_le_bytes(value.0));
        Self {
            current_words: words(&row.current),
            target_words: words(&row.target),
            slope_words: words(&row.slope),
            remaining_words: words(&row.remaining),
        }
    }

    pub(crate) const fn current_words(&self) -> &[u32; 6] {
        &self.current_words
    }

    pub(crate) const fn target_words(&self) -> &[u32; 6] {
        &self.target_words
    }

    pub(crate) const fn slope_words(&self) -> &[u32; 6] {
        &self.slope_words
    }

    pub(crate) const fn remaining_words(&self) -> &[u32; 6] {
        &self.remaining_words
    }
}

#[derive(Clone, Debug, PartialEq)]
struct RingState {
    words: RawBytes,
    write_index: i64,
    read_index: i64,
}

#[derive(Clone, Debug, PartialEq)]
struct TemporalState {
    span: i64,
    stride_bytes: i64,
    index_28: i64,
    index_32: i64,
    buffers: Box<[RawBytes]>,
}

#[derive(Clone, Debug, PartialEq)]
struct DescriptorState {
    tail_peak: RawWord,
    hold_count: i64,
    smoothed: RawWord,
    exponent: i64,
    reference: RawWord,
    gain: RawWord,
}

#[derive(Clone, Debug, PartialEq)]
struct ModelState {
    filter_state: [RawWord; 16],
    row_states: [RowState; 16],
    transform_delay: [RawBytes; 6],
    first_ring: [RingState; 6],
    post_filter_state: RawBytes,
    analysis_history: [[RawBytes; 9]; 5],
    qmf_delay_history: [[RawBytes; 10]; 2],
    qmf_fir_history: [RawBytes; 2],
    synthesis_history: [RawBytes; 2],
    temporal_states: [TemporalState; 5],
    descriptor_state: DescriptorState,
    post_s09_delay: [RawBytes; 6],
    second_ring: [RingState; 6],
    target_coefficients: NoneSentinel,
}

#[derive(Clone, Debug, PartialEq)]
struct QmfWindowState {
    target_temporal: TemporalState,
    target_history: [[RawBytes; 9]; 15],
    previous_targets: [[RawBytes; 24]; 15],
    previous_carriers: [[RawBytes; 24]; 5],
    fitting_delay: [[RawBytes; 10]; 5],
    fitting_fir: [RawBytes; 5],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RawWord([u8; 4]);

#[derive(Clone, Debug, PartialEq)]
struct RawBytes {
    bytes: Box<[u8]>,
}

impl RawBytes {
    fn zeroed(length: usize) -> Self {
        Self {
            bytes: vec![0_u8; length].into_boxed_slice(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NoneSentinel;

impl F04CompositionState {
    fn cold() -> Self {
        Self {
            callback_index: FrontendCallbackIndex::new(0),
            retired: false,
            component_state: ComponentAudioState::cold(),
            component_parameter_state: ComponentFilterParameterState::cold(),
            component_filter_history: zero_raw_tuple::<15>(256),
            lfe_state: LfeDelayRingState::cold(),
            receiver_state: ModelState::cold(),
            fitting_qmf_state: QmfWindowState::cold(),
            receiver_callback_count: ReceiverCallbackCount::new(0),
        }
    }

    fn validate_cold(&self) -> Result<(), SpatialAudioError> {
        let parameters = &self.component_parameter_state;
        let float_bits = [
            parameters.peak.to_bits(),
            parameters.previous_gain.to_bits(),
            parameters.envelope.to_bits(),
            parameters.smoothed_gain.to_bits(),
            parameters.target_gain.to_bits(),
            parameters.output_level.to_bits(),
        ];
        if float_bits.contains(&NEGATIVE_ZERO_BITS) {
            return Err(SpatialAudioError::NegativeZeroColdFloat);
        }
        if float_bits
            .iter()
            .any(|bits| !f32::from_bits(*bits).is_finite())
        {
            return Err(SpatialAudioError::NonFiniteFloat);
        }
        if self.callback_index.value() != 0
            || self.retired
            || self.receiver_callback_count.value() != 0
            || float_bits
                != [
                    0,
                    0x3f80_0000,
                    0x3f80_0000,
                    0x3f80_0000,
                    0x3f80_0000,
                    0x3b80_0000,
                ]
            || parameters.hold != 0
            || parameters.blocks_processed != 0
            || self.lfe_state.write_index != 0
            || self.lfe_state.read_index != 1
            || self.lfe_state.rows.iter().any(Option::is_some)
            || self
                .lfe_state
                .input_frame_indexes
                .iter()
                .any(Option::is_some)
            || self.lfe_state.model_samples_consumed != 0
            || self.receiver_state.target_coefficients != NoneSentinel
        {
            return Err(SpatialAudioError::ColdValueMismatch);
        }
        Ok(())
    }

    fn encode(&self, encoder: &mut F04StateEncoder) -> Result<(), SpatialAudioError> {
        encoder.begin_dataclass("F04CompositionState", 9)?;
        encoder.field("callback_index")?;
        encoder.encode_i64(i64::from(self.callback_index.value()));
        encoder.field("retired")?;
        encoder.encode_bool(self.retired);
        encoder.field("component_state")?;
        self.component_state.encode(encoder)?;
        encoder.field("component_parameter_state")?;
        self.component_parameter_state.encode(encoder)?;
        encoder.field("component_filter_history")?;
        encoder.encode_raw_tuple(&self.component_filter_history)?;
        encoder.field("lfe_state")?;
        self.lfe_state.encode(encoder)?;
        encoder.field("receiver_state")?;
        self.receiver_state.encode(encoder)?;
        encoder.field("fitting_qmf_state")?;
        self.fitting_qmf_state.encode(encoder)?;
        encoder.field("receiver_callback_count")?;
        encoder.encode_i64(
            i64::try_from(self.receiver_callback_count.value())
                .map_err(|_| SpatialAudioError::EncodingLengthOverflow)?,
        );
        Ok(())
    }
}

impl ComponentAudioState {
    fn cold() -> Self {
        Self {
            overlap_history_rows: zero_raw_tuple::<15>(6_144),
            source_ring_rows: zero_raw_tuple::<15>(5_888),
        }
    }

    fn encode(&self, encoder: &mut F04StateEncoder) -> Result<(), SpatialAudioError> {
        encoder.begin_dataclass("ComponentAudioState", 2)?;
        encoder.field("overlap_history_rows")?;
        encoder.encode_raw_tuple(&self.overlap_history_rows)?;
        encoder.field("source_ring_rows")?;
        encoder.encode_raw_tuple(&self.source_ring_rows)
    }
}

impl ComponentFilterParameterState {
    fn cold() -> Self {
        Self {
            peak: 0.0,
            previous_gain: 1.0,
            hold: 0,
            envelope: 1.0,
            smoothed_gain: 1.0,
            target_gain: 1.0,
            output_level: f32::from_bits(0x3b80_0000),
            blocks_processed: 0,
        }
    }

    fn encode(&self, encoder: &mut F04StateEncoder) -> Result<(), SpatialAudioError> {
        encoder.begin_dataclass("ComponentFilterParameterState", 8)?;
        encoder.field("peak")?;
        encoder.encode_cold_f32(self.peak)?;
        encoder.field("previous_gain")?;
        encoder.encode_cold_f32(self.previous_gain)?;
        encoder.field("hold")?;
        encoder.encode_i64(self.hold);
        encoder.field("envelope")?;
        encoder.encode_cold_f32(self.envelope)?;
        encoder.field("smoothed_gain")?;
        encoder.encode_cold_f32(self.smoothed_gain)?;
        encoder.field("target_gain")?;
        encoder.encode_cold_f32(self.target_gain)?;
        encoder.field("output_level")?;
        encoder.encode_cold_f32(self.output_level)?;
        encoder.field("blocks_processed")?;
        encoder.encode_i64(self.blocks_processed);
        Ok(())
    }
}

impl LfeDelayRingState {
    fn cold() -> Self {
        Self {
            write_index: 0,
            read_index: 1,
            rows: [None, None, None],
            input_frame_indexes: [None, None, None],
            model_samples_consumed: 0,
        }
    }

    fn encode(&self, encoder: &mut F04StateEncoder) -> Result<(), SpatialAudioError> {
        encoder.begin_dataclass("LfeDelayRingState", 5)?;
        encoder.field("write_index")?;
        encoder.encode_i64(self.write_index);
        encoder.field("read_index")?;
        encoder.encode_i64(self.read_index);
        encoder.field("rows")?;
        encoder.encode_tuple_length(self.rows.len())?;
        for row in &self.rows {
            match row {
                Some(bytes) => encoder.encode_bytes(bytes)?,
                None => encoder.encode_none(),
            }
        }
        encoder.field("input_frame_indexes")?;
        encoder.encode_tuple_length(self.input_frame_indexes.len())?;
        for index in &self.input_frame_indexes {
            match index {
                Some(index) => encoder.encode_i64(i64::from(index.value())),
                None => encoder.encode_none(),
            }
        }
        encoder.field("model_samples_consumed")?;
        encoder.encode_i64(self.model_samples_consumed);
        Ok(())
    }
}

impl RowState {
    fn cold() -> Self {
        Self {
            current: [RawWord([0; 4]); 6],
            target: [RawWord([0; 4]); 6],
            slope: [RawWord([0; 4]); 6],
            remaining: [RawWord([0; 4]); 6],
        }
    }

    fn encode(&self, encoder: &mut F04StateEncoder) -> Result<(), SpatialAudioError> {
        encoder.begin_dataclass("RowState", 4)?;
        encoder.field("current")?;
        encoder.encode_raw_word_tuple(&self.current)?;
        encoder.field("target")?;
        encoder.encode_raw_word_tuple(&self.target)?;
        encoder.field("slope")?;
        encoder.encode_raw_word_tuple(&self.slope)?;
        encoder.field("remaining")?;
        encoder.encode_raw_word_tuple(&self.remaining)
    }
}

impl RingState {
    fn cold() -> Self {
        Self {
            words: RawBytes::zeroed(900),
            write_index: 224,
            read_index: 0,
        }
    }

    fn encode(&self, encoder: &mut F04StateEncoder) -> Result<(), SpatialAudioError> {
        encoder.begin_dataclass("RingState", 3)?;
        encoder.field("words")?;
        encoder.encode_bytes(&self.words.bytes)?;
        encoder.field("write_index")?;
        encoder.encode_i64(self.write_index);
        encoder.field("read_index")?;
        encoder.encode_i64(self.read_index);
        Ok(())
    }
}

impl TemporalState {
    fn cold(span: i64, lane_count: usize, bytes_per_lane: usize) -> Self {
        let buffers = (0..lane_count)
            .map(|_| RawBytes::zeroed(bytes_per_lane))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self {
            span,
            stride_bytes: 4,
            index_28: span,
            index_32: 0,
            buffers,
        }
    }

    fn encode(&self, encoder: &mut F04StateEncoder) -> Result<(), SpatialAudioError> {
        encoder.begin_dataclass("TemporalState", 5)?;
        encoder.field("span")?;
        encoder.encode_i64(self.span);
        encoder.field("stride_bytes")?;
        encoder.encode_i64(self.stride_bytes);
        encoder.field("index_28")?;
        encoder.encode_i64(self.index_28);
        encoder.field("index_32")?;
        encoder.encode_i64(self.index_32);
        encoder.field("buffers")?;
        encoder.encode_raw_slice_tuple(&self.buffers)
    }
}

impl DescriptorState {
    fn cold() -> Self {
        Self {
            tail_peak: RawWord([0; 4]),
            hold_count: 0,
            smoothed: RawWord([0; 4]),
            exponent: 2,
            reference: RawWord([0xab, 0xaa, 0x2a, 0x3d]),
            gain: RawWord([0x00, 0x00, 0x80, 0x3d]),
        }
    }

    fn encode(&self, encoder: &mut F04StateEncoder) -> Result<(), SpatialAudioError> {
        encoder.begin_dataclass("DescriptorState", 6)?;
        encoder.field("tail_peak")?;
        encoder.encode_bytes(&self.tail_peak.0)?;
        encoder.field("hold_count")?;
        encoder.encode_i64(self.hold_count);
        encoder.field("smoothed")?;
        encoder.encode_bytes(&self.smoothed.0)?;
        encoder.field("exponent")?;
        encoder.encode_i64(self.exponent);
        encoder.field("reference")?;
        encoder.encode_bytes(&self.reference.0)?;
        encoder.field("gain")?;
        encoder.encode_bytes(&self.gain.0)
    }
}

impl ModelState {
    fn cold() -> Self {
        Self {
            filter_state: [RawWord([0; 4]); 16],
            row_states: std::array::from_fn(|_| RowState::cold()),
            transform_delay: zero_raw_tuple::<6>(128),
            first_ring: std::array::from_fn(|_| RingState::cold()),
            post_filter_state: RawBytes::zeroed(32),
            analysis_history: zero_raw_matrix::<5, 9>(256),
            qmf_delay_history: zero_raw_matrix::<2, 10>(512),
            qmf_fir_history: zero_raw_tuple::<2>(160),
            synthesis_history: zero_raw_tuple::<2>(2_304),
            temporal_states: [
                TemporalState::cold(1_217, 3, 9_736),
                TemporalState::cold(63, 5, 504),
                TemporalState::cold(900, 1, 7_200),
                TemporalState::cold(256, 6, 2_048),
                TemporalState::cold(896, 15, 7_168),
            ],
            descriptor_state: DescriptorState::cold(),
            post_s09_delay: zero_raw_tuple::<6>(128),
            second_ring: std::array::from_fn(|_| RingState::cold()),
            target_coefficients: NoneSentinel,
        }
    }

    fn encode(&self, encoder: &mut F04StateEncoder) -> Result<(), SpatialAudioError> {
        encoder.begin_dataclass("ModelState", 14)?;
        encoder.field("filter_state")?;
        encoder.encode_raw_word_tuple(&self.filter_state)?;
        encoder.field("row_states")?;
        encoder.encode_row_tuple(&self.row_states)?;
        encoder.field("transform_delay")?;
        encoder.encode_raw_tuple(&self.transform_delay)?;
        encoder.field("first_ring")?;
        encoder.encode_ring_tuple(&self.first_ring)?;
        encoder.field("post_filter_state")?;
        encoder.encode_bytes(&self.post_filter_state.bytes)?;
        encoder.field("analysis_history")?;
        encoder.encode_raw_matrix(&self.analysis_history)?;
        encoder.field("qmf_delay_history")?;
        encoder.encode_raw_matrix(&self.qmf_delay_history)?;
        encoder.field("qmf_fir_history")?;
        encoder.encode_raw_tuple(&self.qmf_fir_history)?;
        encoder.field("synthesis_history")?;
        encoder.encode_raw_tuple(&self.synthesis_history)?;
        encoder.field("temporal_states")?;
        encoder.encode_temporal_tuple(&self.temporal_states)?;
        encoder.field("descriptor_state")?;
        self.descriptor_state.encode(encoder)?;
        encoder.field("post_s09_delay")?;
        encoder.encode_raw_tuple(&self.post_s09_delay)?;
        encoder.field("second_ring")?;
        encoder.encode_ring_tuple(&self.second_ring)?;
        encoder.field("target_coefficients")?;
        encoder.encode_none();
        Ok(())
    }
}

impl QmfWindowState {
    fn cold() -> Self {
        Self {
            target_temporal: TemporalState::cold(896, 15, 7_168),
            target_history: zero_raw_matrix::<15, 9>(256),
            previous_targets: zero_raw_matrix::<15, 24>(512),
            previous_carriers: zero_raw_matrix::<5, 24>(512),
            fitting_delay: zero_raw_matrix::<5, 10>(512),
            fitting_fir: zero_raw_tuple::<5>(160),
        }
    }

    fn encode(&self, encoder: &mut F04StateEncoder) -> Result<(), SpatialAudioError> {
        encoder.begin_dataclass("QmfWindowState", 6)?;
        encoder.field("target_temporal")?;
        self.target_temporal.encode(encoder)?;
        encoder.field("target_history")?;
        encoder.encode_raw_matrix(&self.target_history)?;
        encoder.field("previous_targets")?;
        encoder.encode_raw_matrix(&self.previous_targets)?;
        encoder.field("previous_carriers")?;
        encoder.encode_raw_matrix(&self.previous_carriers)?;
        encoder.field("fitting_delay")?;
        encoder.encode_raw_matrix(&self.fitting_delay)?;
        encoder.field("fitting_fir")?;
        encoder.encode_raw_tuple(&self.fitting_fir)
    }
}

fn zero_raw_tuple<const COUNT: usize>(length: usize) -> [RawBytes; COUNT] {
    std::array::from_fn(|_| RawBytes::zeroed(length))
}

fn zero_raw_matrix<const OUTER: usize, const INNER: usize>(
    length: usize,
) -> [[RawBytes; INNER]; OUTER] {
    std::array::from_fn(|_| zero_raw_tuple::<INNER>(length))
}

struct F04StateEncoder {
    bytes: Vec<u8>,
}

impl F04StateEncoder {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(capacity),
        }
    }

    #[cfg(test)]
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn begin_dataclass(
        &mut self,
        class_name: &str,
        field_count: usize,
    ) -> Result<(), SpatialAudioError> {
        if !class_name.is_ascii() {
            return Err(SpatialAudioError::NonAsciiNodeName);
        }
        let declared_field_count = match class_name {
            "F04CompositionState" => 9,
            "ComponentAudioState" => 2,
            "ComponentFilterParameterState" => 8,
            "LfeDelayRingState" | "TemporalState" => 5,
            "RowState" => 4,
            "RingState" => 3,
            "DescriptorState" | "QmfWindowState" => 6,
            "ModelState" => 14,
            _ => return Err(SpatialAudioError::UnknownNodeClass),
        };
        if field_count != declared_field_count {
            return Err(SpatialAudioError::DeclaredFieldCountMismatch);
        }
        let name_length = u16::try_from(class_name.len())
            .map_err(|_| SpatialAudioError::EncodingLengthOverflow)?;
        let field_count =
            u16::try_from(field_count).map_err(|_| SpatialAudioError::NodeFieldCountOverflow)?;
        self.bytes.push(b'D');
        self.bytes.extend_from_slice(&name_length.to_le_bytes());
        self.bytes.extend_from_slice(class_name.as_bytes());
        self.bytes.extend_from_slice(&field_count.to_le_bytes());
        Ok(())
    }

    fn field(&mut self, name: &str) -> Result<(), SpatialAudioError> {
        if !name.is_ascii() {
            return Err(SpatialAudioError::NonAsciiNodeName);
        }
        if !matches!(
            name,
            "callback_index"
                | "retired"
                | "component_state"
                | "component_parameter_state"
                | "component_filter_history"
                | "lfe_state"
                | "receiver_state"
                | "fitting_qmf_state"
                | "receiver_callback_count"
                | "overlap_history_rows"
                | "source_ring_rows"
                | "peak"
                | "previous_gain"
                | "hold"
                | "envelope"
                | "smoothed_gain"
                | "target_gain"
                | "output_level"
                | "blocks_processed"
                | "write_index"
                | "read_index"
                | "rows"
                | "input_frame_indexes"
                | "model_samples_consumed"
                | "current"
                | "target"
                | "slope"
                | "remaining"
                | "words"
                | "span"
                | "stride_bytes"
                | "index_28"
                | "index_32"
                | "buffers"
                | "tail_peak"
                | "hold_count"
                | "smoothed"
                | "exponent"
                | "reference"
                | "gain"
                | "filter_state"
                | "row_states"
                | "transform_delay"
                | "first_ring"
                | "post_filter_state"
                | "analysis_history"
                | "qmf_delay_history"
                | "qmf_fir_history"
                | "synthesis_history"
                | "temporal_states"
                | "descriptor_state"
                | "post_s09_delay"
                | "second_ring"
                | "target_coefficients"
                | "target_temporal"
                | "target_history"
                | "previous_targets"
                | "previous_carriers"
                | "fitting_delay"
                | "fitting_fir"
        ) {
            return Err(SpatialAudioError::UnknownFieldName);
        }
        let name_length =
            u16::try_from(name.len()).map_err(|_| SpatialAudioError::EncodingLengthOverflow)?;
        self.bytes.extend_from_slice(&name_length.to_le_bytes());
        self.bytes.extend_from_slice(name.as_bytes());
        Ok(())
    }

    fn encode_none(&mut self) {
        self.bytes.push(b'N');
    }

    fn encode_i64(&mut self, value: i64) {
        self.bytes.push(b'I');
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn encode_bool(&mut self, value: bool) {
        self.bytes.push(b'Z');
        self.bytes.push(u8::from(value));
    }

    fn encode_f32(&mut self, value: f32) -> Result<(), SpatialAudioError> {
        if !value.is_finite() {
            return Err(SpatialAudioError::NonFiniteFloat);
        }
        self.bytes.push(b'F');
        self.bytes.extend_from_slice(&value.to_bits().to_le_bytes());
        Ok(())
    }

    fn encode_cold_f32(&mut self, value: f32) -> Result<(), SpatialAudioError> {
        if value.to_bits() == NEGATIVE_ZERO_BITS {
            return Err(SpatialAudioError::NegativeZeroColdFloat);
        }
        self.encode_f32(value)
    }

    fn encode_bytes(&mut self, value: &[u8]) -> Result<(), SpatialAudioError> {
        let length =
            u64::try_from(value.len()).map_err(|_| SpatialAudioError::EncodingLengthOverflow)?;
        self.bytes.push(b'B');
        self.bytes.extend_from_slice(&length.to_le_bytes());
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    fn encode_tuple_length(&mut self, item_count: usize) -> Result<(), SpatialAudioError> {
        let count =
            u64::try_from(item_count).map_err(|_| SpatialAudioError::EncodingLengthOverflow)?;
        self.bytes.push(b'T');
        self.bytes.extend_from_slice(&count.to_le_bytes());
        Ok(())
    }

    fn encode_raw_tuple(&mut self, values: &[RawBytes]) -> Result<(), SpatialAudioError> {
        self.encode_tuple_length(values.len())?;
        for value in values {
            self.encode_bytes(&value.bytes)?;
        }
        Ok(())
    }

    fn encode_raw_slice_tuple(&mut self, values: &[RawBytes]) -> Result<(), SpatialAudioError> {
        self.encode_raw_tuple(values)
    }

    fn encode_raw_word_tuple<const COUNT: usize>(
        &mut self,
        values: &[RawWord; COUNT],
    ) -> Result<(), SpatialAudioError> {
        self.encode_tuple_length(COUNT)?;
        for value in values {
            self.encode_bytes(&value.0)?;
        }
        Ok(())
    }

    fn encode_row_tuple<const COUNT: usize>(
        &mut self,
        values: &[RowState; COUNT],
    ) -> Result<(), SpatialAudioError> {
        self.encode_tuple_length(COUNT)?;
        for value in values {
            value.encode(self)?;
        }
        Ok(())
    }

    fn encode_ring_tuple<const COUNT: usize>(
        &mut self,
        values: &[RingState; COUNT],
    ) -> Result<(), SpatialAudioError> {
        self.encode_tuple_length(COUNT)?;
        for value in values {
            value.encode(self)?;
        }
        Ok(())
    }

    fn encode_temporal_tuple<const COUNT: usize>(
        &mut self,
        values: &[TemporalState; COUNT],
    ) -> Result<(), SpatialAudioError> {
        self.encode_tuple_length(COUNT)?;
        for value in values {
            value.encode(self)?;
        }
        Ok(())
    }

    fn encode_raw_matrix<const OUTER: usize, const INNER: usize>(
        &mut self,
        values: &[[RawBytes; INNER]; OUTER],
    ) -> Result<(), SpatialAudioError> {
        self.encode_tuple_length(OUTER)?;
        for row in values {
            self.encode_raw_tuple(row)?;
        }
        Ok(())
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

#[cfg(test)]
mod tests;
