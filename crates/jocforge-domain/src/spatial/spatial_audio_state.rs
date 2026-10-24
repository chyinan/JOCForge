// pattern: Functional Core
//! Typed F04 cold state and staging capabilities. No DSP or local commit authority lives here.
use crate::{
    profile::{EncodingProfile, resolve_selected_profile},
    spatial::source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
    timeline::{FrontendCallbackIndex, ReceiverCallbackCount},
    transaction::CallbackTransactionId,
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

pub use fitting_mode0::SelectedJocQmfProjection;

const EXPECTED_COLD_BYTES: usize = 813_733;
const NEGATIVE_ZERO_BITS: u32 = 0x8000_0000;

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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialAudioDigest([u8; 32]);

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
    profile_identity: EncodingProfile,
}

impl SpatialAudioMachine {
    pub fn new_cold(profile_identity: EncodingProfile) -> Result<Self, SpatialAudioError> {
        resolve_selected_profile(&profile_identity)
            .map_err(|_| SpatialAudioError::UnsupportedProfile)?;
        Ok(Self {
            committed: SpatialAudioState::new_cold()?,
            generation: 0,
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
    transaction_id: CallbackTransactionId,
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
        transaction_id: CallbackTransactionId,
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

    pub const fn transaction_id(&self) -> &CallbackTransactionId {
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
        encoder.encode_i64(
            i64::try_from(self.callback_index.value())
                .map_err(|_| SpatialAudioError::EncodingLengthOverflow)?,
        );
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
                Some(index) => encoder.encode_i64(
                    i64::try_from(index.value())
                        .map_err(|_| SpatialAudioError::EncodingLengthOverflow)?,
                ),
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
