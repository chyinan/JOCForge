// pattern: Functional Core
//! Value-only 1664 kbps Flat7x I0/D0 target contract.
//!
//! This type records the Wash-approved design target and remains outside
//! `EncodingProfile` resolution. It has no frame writer, source-role mapper,
//! commit authority, or production admission path.

use super::production::{AuthoredInputProfile, CodecRateProfile, JocReconstructionLayout};
use crate::integrity::crc16::eac3_crc16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xSubstreamRole {
    IndependentI0,
    DependentD0,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Eac3CodedChannel {
    Left,
    Center,
    Right,
    LeftSurround,
    RightSurround,
    LeftRearSurround,
    RightRearSurround,
    LowFrequencyEffects,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReconstructionCarrierPosition {
    Left,
    Center,
    Right,
    LeftSurround,
    RightSurround,
    LeftRearSurround,
    RightRearSurround,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Eac3SubstreamId(u8);

impl Eac3SubstreamId {
    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Eac3Acmod(u8);

impl Eac3Acmod {
    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DependentChannelMap(u16);

impl DependentChannelMap {
    /// Creates a typed 16-bit numeric channel-map field value.
    ///
    /// This value carries no speaker-label or source-row semantics.
    pub const fn from_numeric_value(value: u16) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u16 {
        self.0
    }

    /// Target numeric bit positions in MSB-indexed `chanmap` order.
    pub const fn msb_indexed_set_bits(self) -> [u8; 3] {
        [3, 4, 6]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xFrameTarget {
    role: Flat7xSubstreamRole,
    substream_id: Eac3SubstreamId,
    acmod: Eac3Acmod,
    lfeon: bool,
    channel_map: Option<DependentChannelMap>,
    rate: CodecRateProfile,
    frame_bytes: u16,
    coded_channels: &'static [Eac3CodedChannel],
}

impl Flat7xFrameTarget {
    pub const fn role(self) -> Flat7xSubstreamRole {
        self.role
    }

    pub const fn substream_id(self) -> Eac3SubstreamId {
        self.substream_id
    }

    pub const fn acmod(self) -> Eac3Acmod {
        self.acmod
    }

    pub const fn lfeon(self) -> bool {
        self.lfeon
    }

    pub const fn channel_map(self) -> Option<DependentChannelMap> {
        self.channel_map
    }

    pub const fn rate(self) -> CodecRateProfile {
        self.rate
    }

    pub const fn frame_bytes(self) -> u16 {
        self.frame_bytes
    }

    pub const fn coded_channels(self) -> &'static [Eac3CodedChannel] {
        self.coded_channels
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xDownmixControls {
    Unbound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xOpenGate {
    I0ExactBsid,
    AdmSourceRowToD0RoleJoin,
    DownmixModeAndPhase,
    RuntimeSourceGenerationJoin,
    NativeD0AudioAlgorithm,
    NativeD0WriterOwner,
    AllAccessUnitEmdfFit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xEmdfOverflowPolicy {
    RejectWholeAccessUnitBeforePublish,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D0EmdfSkipFieldTarget {
    zero_based_audio_block: u8,
    field_count: u8,
    maximum_complete_container_bytes: u16,
    fragmentation_allowed: bool,
    overflow_policy: Flat7xEmdfOverflowPolicy,
}

impl D0EmdfSkipFieldTarget {
    pub const fn zero_based_audio_block(self) -> u8 {
        self.zero_based_audio_block
    }

    pub const fn field_count(self) -> u8 {
        self.field_count
    }

    pub const fn maximum_complete_container_bytes(self) -> u16 {
        self.maximum_complete_container_bytes
    }

    pub const fn fragmentation_allowed(self) -> bool {
        self.fragmentation_allowed
    }

    pub const fn overflow_policy(self) -> Flat7xEmdfOverflowPolicy {
        self.overflow_policy
    }
}

/// A design-only AU plan. It cannot be converted into a production profile or writer input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xI0D0TargetPlan {
    input_profile: AuthoredInputProfile,
    accepted_physical_channel_counts: [u16; 2],
    sample_rate_hz: u32,
    audio_blocks_per_frame: u8,
    samples_per_frame: u16,
    i0: Flat7xFrameTarget,
    d0: Flat7xFrameTarget,
    reconstruction_layout: JocReconstructionLayout,
    joc_dmx_config_idx: u8,
    reconstruction_carriers: [ReconstructionCarrierPosition; 7],
    coded_object_count: u8,
    complexity_index_type_a: u8,
    downmix_controls: Flat7xDownmixControls,
    d0_emdf_skip_field: D0EmdfSkipFieldTarget,
    open_gates: [Flat7xOpenGate; 7],
}

impl Flat7xI0D0TargetPlan {
    /// Returns the immutable target contract captured by the sanitized Wash handoff.
    pub const fn approved_contract() -> Self {
        const I0_CHANNELS: &[Eac3CodedChannel] = &[
            Eac3CodedChannel::Left,
            Eac3CodedChannel::Center,
            Eac3CodedChannel::Right,
            Eac3CodedChannel::LeftSurround,
            Eac3CodedChannel::RightSurround,
            Eac3CodedChannel::LowFrequencyEffects,
        ];
        const D0_CHANNELS: &[Eac3CodedChannel] = &[
            Eac3CodedChannel::LeftSurround,
            Eac3CodedChannel::RightSurround,
            Eac3CodedChannel::LeftRearSurround,
            Eac3CodedChannel::RightRearSurround,
        ];

        Self {
            input_profile: AuthoredInputProfile::SelectedAdmBwfPcm24At48Khz,
            accepted_physical_channel_counts: [36, 38],
            sample_rate_hz: 48_000,
            audio_blocks_per_frame: 6,
            samples_per_frame: 1_536,
            i0: Flat7xFrameTarget {
                role: Flat7xSubstreamRole::IndependentI0,
                substream_id: Eac3SubstreamId(0),
                acmod: Eac3Acmod(7),
                lfeon: true,
                channel_map: None,
                rate: CodecRateProfile::Kbps640,
                frame_bytes: 2_560,
                coded_channels: I0_CHANNELS,
            },
            d0: Flat7xFrameTarget {
                role: Flat7xSubstreamRole::DependentD0,
                substream_id: Eac3SubstreamId(0),
                acmod: Eac3Acmod(5),
                lfeon: false,
                channel_map: Some(DependentChannelMap(0x1a00)),
                rate: CodecRateProfile::Kbps1024,
                frame_bytes: 4_096,
                coded_channels: D0_CHANNELS,
            },
            reconstruction_layout: JocReconstructionLayout::Flat7x,
            joc_dmx_config_idx: 1,
            reconstruction_carriers: [
                ReconstructionCarrierPosition::Left,
                ReconstructionCarrierPosition::Right,
                ReconstructionCarrierPosition::Center,
                ReconstructionCarrierPosition::LeftSurround,
                ReconstructionCarrierPosition::RightSurround,
                ReconstructionCarrierPosition::LeftRearSurround,
                ReconstructionCarrierPosition::RightRearSurround,
            ],
            coded_object_count: 15,
            complexity_index_type_a: 16,
            downmix_controls: Flat7xDownmixControls::Unbound,
            d0_emdf_skip_field: D0EmdfSkipFieldTarget {
                zero_based_audio_block: 2,
                field_count: 1,
                maximum_complete_container_bytes: 511,
                fragmentation_allowed: false,
                overflow_policy: Flat7xEmdfOverflowPolicy::RejectWholeAccessUnitBeforePublish,
            },
            open_gates: [
                Flat7xOpenGate::I0ExactBsid,
                Flat7xOpenGate::AdmSourceRowToD0RoleJoin,
                Flat7xOpenGate::DownmixModeAndPhase,
                Flat7xOpenGate::RuntimeSourceGenerationJoin,
                Flat7xOpenGate::NativeD0AudioAlgorithm,
                Flat7xOpenGate::NativeD0WriterOwner,
                Flat7xOpenGate::AllAccessUnitEmdfFit,
            ],
        }
    }

    pub const fn input_profile(self) -> AuthoredInputProfile {
        self.input_profile
    }

    pub const fn accepted_physical_channel_counts(self) -> [u16; 2] {
        self.accepted_physical_channel_counts
    }

    pub const fn sample_rate_hz(self) -> u32 {
        self.sample_rate_hz
    }

    pub const fn audio_blocks_per_frame(self) -> u8 {
        self.audio_blocks_per_frame
    }

    pub const fn samples_per_frame(self) -> u16 {
        self.samples_per_frame
    }

    pub const fn i0(self) -> Flat7xFrameTarget {
        self.i0
    }

    pub const fn d0(self) -> Flat7xFrameTarget {
        self.d0
    }

    pub const fn au_order(self) -> [Flat7xSubstreamRole; 2] {
        [
            Flat7xSubstreamRole::IndependentI0,
            Flat7xSubstreamRole::DependentD0,
        ]
    }

    pub const fn aggregate_rate_kbps(self) -> Option<u32> {
        match (self.i0.rate.kbps(), self.d0.rate.kbps()) {
            (Some(i0), Some(d0)) => Some(i0 + d0),
            _ => None,
        }
    }

    pub const fn aggregate_frame_bytes(self) -> u32 {
        self.i0.frame_bytes as u32 + self.d0.frame_bytes as u32
    }

    pub const fn reconstruction_layout(self) -> JocReconstructionLayout {
        self.reconstruction_layout
    }

    pub const fn joc_dmx_config_idx(self) -> u8 {
        self.joc_dmx_config_idx
    }

    pub const fn reconstruction_carriers(self) -> [ReconstructionCarrierPosition; 7] {
        self.reconstruction_carriers
    }

    pub const fn coded_object_count(self) -> u8 {
        self.coded_object_count
    }

    pub const fn complexity_index_type_a(self) -> u8 {
        self.complexity_index_type_a
    }

    pub const fn downmix_controls(self) -> Flat7xDownmixControls {
        self.downmix_controls
    }

    pub const fn d0_emdf_skip_field(self) -> D0EmdfSkipFieldTarget {
        self.d0_emdf_skip_field
    }

    pub const fn same_source_generation_required(self) -> bool {
        true
    }

    pub const fn atomic_pair_publication_required(self) -> bool {
        true
    }

    pub const fn partial_i0_publication_allowed(self) -> bool {
        false
    }

    pub const fn production_allowed(self) -> bool {
        false
    }

    pub const fn open_gates(self) -> [Flat7xOpenGate; 7] {
        self.open_gates
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Caller-supplied generation label for target-plan equality checks only.
/// Equality here is not evidence of a runtime owner/source lineage join.
pub struct Flat7xSourceGeneration(u64);

impl Flat7xSourceGeneration {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Payload-free candidate metadata used only to validate the target AU plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct I0FrameExtent {
    source_generation: Flat7xSourceGeneration,
    frame_bytes: u16,
}

impl I0FrameExtent {
    pub const fn new(source_generation: Flat7xSourceGeneration, frame_bytes: u16) -> Self {
        Self {
            source_generation,
            frame_bytes,
        }
    }

    pub const fn source_generation(self) -> Flat7xSourceGeneration {
        self.source_generation
    }

    pub const fn frame_bytes(self) -> u16 {
        self.frame_bytes
    }
}

/// Payload-free candidate metadata used only to validate the target AU plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D0FrameExtent {
    source_generation: Flat7xSourceGeneration,
    frame_bytes: u16,
}

impl D0FrameExtent {
    pub const fn new(source_generation: Flat7xSourceGeneration, frame_bytes: u16) -> Self {
        Self {
            source_generation,
            frame_bytes,
        }
    }

    pub const fn source_generation(self) -> Flat7xSourceGeneration {
        self.source_generation
    }

    pub const fn frame_bytes(self) -> u16 {
        self.frame_bytes
    }
}

/// Metadata for the one complete EMDF container assigned to D0 block two.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D0EmdfSkipFieldExtent {
    source_generation: Flat7xSourceGeneration,
    zero_based_audio_block: u8,
    field_count: u8,
    complete_container_bytes: Option<u16>,
    fragmented: bool,
}

impl D0EmdfSkipFieldExtent {
    pub const fn new(
        source_generation: Flat7xSourceGeneration,
        zero_based_audio_block: u8,
        field_count: u8,
        complete_container_bytes: Option<u16>,
        fragmented: bool,
    ) -> Self {
        Self {
            source_generation,
            zero_based_audio_block,
            field_count,
            complete_container_bytes,
            fragmented,
        }
    }
}

/// Candidate members remain typed separately so an I0-only partial AU cannot be represented as
/// a successful whole-AU plan validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xAuCandidate {
    i0: Option<I0FrameExtent>,
    d0: Option<D0FrameExtent>,
    d0_emdf: Option<D0EmdfSkipFieldExtent>,
}

impl Flat7xAuCandidate {
    pub const fn new(
        i0: Option<I0FrameExtent>,
        d0: Option<D0FrameExtent>,
        d0_emdf: Option<D0EmdfSkipFieldExtent>,
    ) -> Self {
        Self { i0, d0, d0_emdf }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xAuPlanValidationError {
    I0FrameMissing,
    D0FrameMissing,
    D0EmdfFieldMissing,
    SourceGenerationMismatch,
    I0FrameExtentMismatch,
    D0FrameExtentMismatch,
    D0EmdfGenerationMismatch,
    D0EmdfBlockMismatch,
    D0EmdfFieldCountMismatch,
    D0EmdfFragmented,
    D0EmdfContainerMissing,
    D0EmdfContainerEmpty,
    D0EmdfContainerTooLarge,
}

/// Successful value-only validation of a complete target AU pair.
///
/// This value proves only that the supplied metadata fits the design contract. It has no frame
/// bytes, writer capability, owner receipts, or publication authority. Production remains disabled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xAuPlanValidation {
    source_generation: Flat7xSourceGeneration,
    i0: I0FrameExtent,
    d0: D0FrameExtent,
    d0_emdf_container_bytes: u16,
}

impl Flat7xAuPlanValidation {
    pub const fn production_allowed(self) -> bool {
        false
    }

    pub const fn source_generation(self) -> Flat7xSourceGeneration {
        self.source_generation
    }

    pub const fn i0(self) -> I0FrameExtent {
        self.i0
    }

    pub const fn d0(self) -> D0FrameExtent {
        self.d0
    }

    pub const fn d0_emdf_container_bytes(self) -> u16 {
        self.d0_emdf_container_bytes
    }

    pub const fn au_order(self) -> [Flat7xSubstreamRole; 2] {
        [
            Flat7xSubstreamRole::IndependentI0,
            Flat7xSubstreamRole::DependentD0,
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xAuByteSequenceError {
    I0FrameExtentMismatch,
    D0FrameExtentMismatch,
    I0FrameCrc,
    D0FrameCrc,
    AggregateExtentOverflow,
}

/// A noncommitting in-memory I0→D0 byte sequence candidate.
///
/// Construction checks only the validated target extents and each frame's E-AC-3 CRC. It does
/// not parse substream syntax, establish runtime source lineage, or grant output-owner authority.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct Flat7xAuByteSequenceCandidate {
    source_generation: Flat7xSourceGeneration,
    i0_frame_bytes: u16,
    d0_frame_bytes: u16,
    d0_emdf_container_bytes: u16,
    bytes: Box<[u8]>,
}

impl Flat7xAuByteSequenceCandidate {
    pub const fn source_generation(&self) -> Flat7xSourceGeneration {
        self.source_generation
    }

    pub const fn i0_frame_bytes(&self) -> u16 {
        self.i0_frame_bytes
    }

    pub const fn d0_frame_bytes(&self) -> u16 {
        self.d0_frame_bytes
    }

    pub const fn d0_emdf_container_bytes(&self) -> u16 {
        self.d0_emdf_container_bytes
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

/// Concatenates complete candidate bytes in I0→D0 order after the target metadata gate.
///
/// This is a byte-sequence boundary only. The caller-provided metadata validation and CRC checks
/// do not prove the frame roles, source owner/generation join, or D0 audio syntax.
pub fn compose_flat7x_au_byte_sequence(
    validation: &Flat7xAuPlanValidation,
    i0_frame: &[u8],
    d0_frame: &[u8],
) -> Result<Flat7xAuByteSequenceCandidate, Flat7xAuByteSequenceError> {
    if i0_frame.len() != usize::from(validation.i0().frame_bytes()) {
        return Err(Flat7xAuByteSequenceError::I0FrameExtentMismatch);
    }
    if d0_frame.len() != usize::from(validation.d0().frame_bytes()) {
        return Err(Flat7xAuByteSequenceError::D0FrameExtentMismatch);
    }
    if eac3_crc16(&i0_frame[2..]) != 0 {
        return Err(Flat7xAuByteSequenceError::I0FrameCrc);
    }
    if eac3_crc16(&d0_frame[2..]) != 0 {
        return Err(Flat7xAuByteSequenceError::D0FrameCrc);
    }

    let total_bytes = i0_frame
        .len()
        .checked_add(d0_frame.len())
        .ok_or(Flat7xAuByteSequenceError::AggregateExtentOverflow)?;
    let mut bytes = Vec::with_capacity(total_bytes);
    bytes.extend_from_slice(i0_frame);
    bytes.extend_from_slice(d0_frame);

    Ok(Flat7xAuByteSequenceCandidate {
        source_generation: validation.source_generation(),
        i0_frame_bytes: u16::try_from(i0_frame.len())
            .map_err(|_| Flat7xAuByteSequenceError::AggregateExtentOverflow)?,
        d0_frame_bytes: u16::try_from(d0_frame.len())
            .map_err(|_| Flat7xAuByteSequenceError::AggregateExtentOverflow)?,
        d0_emdf_container_bytes: validation.d0_emdf_container_bytes(),
        bytes: bytes.into_boxed_slice(),
    })
}

/// Validates the design-only I0→D0 target pair without inspecting or retaining payload bytes.
/// This does not admit a production encode or prove a runtime source-generation join.
pub fn validate_flat7x_au_candidate(
    plan: Flat7xI0D0TargetPlan,
    candidate: Flat7xAuCandidate,
) -> Result<Flat7xAuPlanValidation, Flat7xAuPlanValidationError> {
    let i0 = candidate
        .i0
        .ok_or(Flat7xAuPlanValidationError::I0FrameMissing)?;
    let d0 = candidate
        .d0
        .ok_or(Flat7xAuPlanValidationError::D0FrameMissing)?;
    let emdf = candidate
        .d0_emdf
        .ok_or(Flat7xAuPlanValidationError::D0EmdfFieldMissing)?;
    let source_generation = i0.source_generation();
    if d0.source_generation() != source_generation || emdf.source_generation != source_generation {
        return Err(Flat7xAuPlanValidationError::SourceGenerationMismatch);
    }
    if i0.frame_bytes() != plan.i0().frame_bytes() {
        return Err(Flat7xAuPlanValidationError::I0FrameExtentMismatch);
    }
    if d0.frame_bytes() != plan.d0().frame_bytes() {
        return Err(Flat7xAuPlanValidationError::D0FrameExtentMismatch);
    }
    if emdf.zero_based_audio_block != plan.d0_emdf_skip_field().zero_based_audio_block() {
        return Err(Flat7xAuPlanValidationError::D0EmdfBlockMismatch);
    }
    if emdf.field_count != plan.d0_emdf_skip_field().field_count() {
        return Err(Flat7xAuPlanValidationError::D0EmdfFieldCountMismatch);
    }
    if emdf.fragmented {
        return Err(Flat7xAuPlanValidationError::D0EmdfFragmented);
    }
    let complete_container_bytes = emdf
        .complete_container_bytes
        .ok_or(Flat7xAuPlanValidationError::D0EmdfContainerMissing)?;
    if complete_container_bytes == 0 {
        return Err(Flat7xAuPlanValidationError::D0EmdfContainerEmpty);
    }
    if complete_container_bytes > plan.d0_emdf_skip_field().maximum_complete_container_bytes() {
        return Err(Flat7xAuPlanValidationError::D0EmdfContainerTooLarge);
    }

    Ok(Flat7xAuPlanValidation {
        source_generation,
        i0,
        d0,
        d0_emdf_container_bytes: complete_container_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{
        bitio::{FrameBitCursor, FrameBitWriter},
        frame_builder::{Flat7xD0InfoAddbsiError, append_flat7x_d0_info_and_addbsi},
    };
    use crate::profile::{EncodingProfile, resolve_selected_profile};

    fn crc_valid_frame_extent(frame_bytes: usize, marker: u8) -> Vec<u8> {
        let frame_size = crate::codec::frame_size::Eac3FrameSize::from_frame_bytes(frame_bytes)
            .expect("target frame extent");
        let mut frame = vec![0_u8; frame_bytes];
        frame[0..2].copy_from_slice(&[0x0b, 0x77]);
        frame[100] = marker;
        for offset in 0..11 {
            let value = (frame_size.code().value() >> (10 - offset)) & 1;
            if value != 0 {
                let bit = 21 + offset;
                frame[bit / 8] |= 1 << (7 - (bit % 8));
            }
        }
        let crc_offset = frame.len() - 2;
        let crc = eac3_crc16(&frame[2..crc_offset]);
        frame[crc_offset..].copy_from_slice(&crc.to_be_bytes());
        assert_eq!(eac3_crc16(&frame[2..]), 0);
        frame
    }

    #[test]
    fn target_plan_keeps_i0_and_d0_as_distinct_ordered_frames() {
        let plan = Flat7xI0D0TargetPlan::approved_contract();

        assert_eq!(
            plan.au_order(),
            [
                Flat7xSubstreamRole::IndependentI0,
                Flat7xSubstreamRole::DependentD0,
            ]
        );
        assert_eq!(plan.i0().substream_id().value(), 0);
        assert_eq!(plan.i0().acmod().value(), 7);
        assert!(plan.i0().lfeon());
        assert_eq!(plan.i0().channel_map(), None);
        assert_eq!(plan.i0().rate(), CodecRateProfile::Kbps640);
        assert_eq!(plan.i0().frame_bytes(), 2_560);
        assert_eq!(plan.i0().coded_channels().len(), 6);

        assert_eq!(plan.d0().substream_id().value(), 0);
        assert_eq!(plan.d0().acmod().value(), 5);
        assert!(!plan.d0().lfeon());
        assert_eq!(
            plan.d0().channel_map().map(DependentChannelMap::value),
            Some(0x1a00)
        );
        assert_eq!(
            plan.d0()
                .channel_map()
                .map(DependentChannelMap::msb_indexed_set_bits),
            Some([3, 4, 6])
        );
        assert_eq!(plan.d0().rate(), CodecRateProfile::Kbps1024);
        assert_eq!(plan.d0().frame_bytes(), 4_096);
        assert_eq!(plan.d0().coded_channels().len(), 4);

        assert_eq!(plan.aggregate_rate_kbps(), Some(1_664));
        assert_eq!(plan.aggregate_frame_bytes(), 6_656);
        assert_eq!(plan.audio_blocks_per_frame(), 6);
        assert_eq!(plan.samples_per_frame(), 1_536);
    }

    #[test]
    fn flat7x_reconstruction_is_separate_from_unbound_downmix_controls() {
        let plan = Flat7xI0D0TargetPlan::approved_contract();

        assert_eq!(
            plan.reconstruction_layout(),
            JocReconstructionLayout::Flat7x
        );
        assert_eq!(plan.joc_dmx_config_idx(), 1);
        assert_eq!(
            plan.reconstruction_carriers(),
            [
                ReconstructionCarrierPosition::Left,
                ReconstructionCarrierPosition::Right,
                ReconstructionCarrierPosition::Center,
                ReconstructionCarrierPosition::LeftSurround,
                ReconstructionCarrierPosition::RightSurround,
                ReconstructionCarrierPosition::LeftRearSurround,
                ReconstructionCarrierPosition::RightRearSurround,
            ]
        );
        assert_eq!(plan.coded_object_count(), 15);
        assert_eq!(plan.complexity_index_type_a(), 16);
        assert_eq!(plan.downmix_controls(), Flat7xDownmixControls::Unbound);
    }

    #[test]
    fn d0_emdf_target_is_one_bounded_block_two_field_and_production_disabled() {
        let plan = Flat7xI0D0TargetPlan::approved_contract();
        let emdf = plan.d0_emdf_skip_field();

        assert_eq!(emdf.zero_based_audio_block(), 2);
        assert_eq!(emdf.field_count(), 1);
        assert_eq!(emdf.maximum_complete_container_bytes(), 511);
        assert!(!emdf.fragmentation_allowed());
        assert_eq!(
            emdf.overflow_policy(),
            Flat7xEmdfOverflowPolicy::RejectWholeAccessUnitBeforePublish
        );
        assert!(plan.same_source_generation_required());
        assert!(plan.atomic_pair_publication_required());
        assert!(!plan.partial_i0_publication_allowed());
        assert!(!plan.production_allowed());
        assert_eq!(plan.open_gates().len(), 7);
    }

    fn complete_candidate(
        i0_generation: u64,
        d0_generation: u64,
        emdf_generation: u64,
        emdf_block: u8,
        emdf_fields: u8,
        emdf_bytes: Option<u16>,
        fragmented: bool,
    ) -> Flat7xAuCandidate {
        Flat7xAuCandidate::new(
            Some(I0FrameExtent::new(
                Flat7xSourceGeneration::new(i0_generation),
                2_560,
            )),
            Some(D0FrameExtent::new(
                Flat7xSourceGeneration::new(d0_generation),
                4_096,
            )),
            Some(D0EmdfSkipFieldExtent::new(
                Flat7xSourceGeneration::new(emdf_generation),
                emdf_block,
                emdf_fields,
                emdf_bytes,
                fragmented,
            )),
        )
    }

    #[test]
    fn target_au_admission_returns_only_a_complete_same_generation_pair() {
        let plan = Flat7xI0D0TargetPlan::approved_contract();
        let candidate = complete_candidate(7, 7, 7, 2, 1, Some(511), false);

        let admission = validate_flat7x_au_candidate(plan, candidate).unwrap();

        assert_eq!(admission.source_generation().value(), 7);
        assert_eq!(admission.i0().frame_bytes(), 2_560);
        assert_eq!(admission.d0().frame_bytes(), 4_096);
        assert_eq!(admission.d0_emdf_container_bytes(), 511);
        assert!(!admission.production_allowed());
        assert_eq!(
            admission.au_order(),
            [
                Flat7xSubstreamRole::IndependentI0,
                Flat7xSubstreamRole::DependentD0,
            ]
        );
    }

    #[test]
    fn target_au_admission_fails_closed_on_missing_member_or_generation_mismatch() {
        let plan = Flat7xI0D0TargetPlan::approved_contract();
        let partial_i0 = Flat7xAuCandidate::new(
            Some(I0FrameExtent::new(Flat7xSourceGeneration::new(7), 2_560)),
            None,
            None,
        );
        assert_eq!(
            validate_flat7x_au_candidate(plan, partial_i0),
            Err(Flat7xAuPlanValidationError::D0FrameMissing)
        );

        let split_generation = complete_candidate(7, 8, 7, 2, 1, Some(16), false);
        assert_eq!(
            validate_flat7x_au_candidate(plan, split_generation),
            Err(Flat7xAuPlanValidationError::SourceGenerationMismatch)
        );
    }

    #[test]
    fn target_au_admission_rejects_wrong_frame_extent_and_skip_field_shape() {
        let plan = Flat7xI0D0TargetPlan::approved_contract();
        let wrong_d0_extent = Flat7xAuCandidate::new(
            Some(I0FrameExtent::new(Flat7xSourceGeneration::new(7), 2_560)),
            Some(D0FrameExtent::new(Flat7xSourceGeneration::new(7), 2_560)),
            Some(D0EmdfSkipFieldExtent::new(
                Flat7xSourceGeneration::new(7),
                2,
                1,
                Some(16),
                false,
            )),
        );
        assert_eq!(
            validate_flat7x_au_candidate(plan, wrong_d0_extent),
            Err(Flat7xAuPlanValidationError::D0FrameExtentMismatch)
        );

        for (block, fields, bytes, fragmented, expected) in [
            (
                1,
                1,
                Some(16),
                false,
                Flat7xAuPlanValidationError::D0EmdfBlockMismatch,
            ),
            (
                2,
                2,
                Some(16),
                false,
                Flat7xAuPlanValidationError::D0EmdfFieldCountMismatch,
            ),
            (
                2,
                1,
                Some(16),
                true,
                Flat7xAuPlanValidationError::D0EmdfFragmented,
            ),
            (
                2,
                1,
                None,
                false,
                Flat7xAuPlanValidationError::D0EmdfContainerMissing,
            ),
            (
                2,
                1,
                Some(0),
                false,
                Flat7xAuPlanValidationError::D0EmdfContainerEmpty,
            ),
            (
                2,
                1,
                Some(512),
                false,
                Flat7xAuPlanValidationError::D0EmdfContainerTooLarge,
            ),
        ] {
            let candidate = complete_candidate(7, 7, 7, block, fields, bytes, fragmented);
            assert_eq!(validate_flat7x_au_candidate(plan, candidate), Err(expected));
        }
    }

    #[test]
    fn byte_sequence_candidate_concatenates_i0_then_d0_and_keeps_production_closed() {
        let plan = Flat7xI0D0TargetPlan::approved_contract();
        let candidate = complete_candidate(7, 7, 7, 2, 1, Some(511), false);
        let validation = validate_flat7x_au_candidate(plan, candidate).unwrap();
        let i0 = crc_valid_frame_extent(2_560, 0x31);
        let d0 = crc_valid_frame_extent(4_096, 0x72);

        let sequence = compose_flat7x_au_byte_sequence(&validation, &i0, &d0).unwrap();

        assert_eq!(sequence.source_generation().value(), 7);
        assert_eq!(sequence.i0_frame_bytes(), 2_560);
        assert_eq!(sequence.d0_frame_bytes(), 4_096);
        assert_eq!(sequence.bytes().len(), 6_656);
        assert_eq!(&sequence.bytes()[..i0.len()], i0.as_slice());
        assert_eq!(&sequence.bytes()[i0.len()..], d0.as_slice());
        assert!(!sequence.production_allowed());
    }

    #[test]
    fn byte_sequence_candidate_rejects_frame_extent_or_crc_mismatch() {
        let plan = Flat7xI0D0TargetPlan::approved_contract();
        let candidate = complete_candidate(7, 7, 7, 2, 1, Some(16), false);
        let validation = validate_flat7x_au_candidate(plan, candidate).unwrap();
        let i0 = crc_valid_frame_extent(2_560, 0x31);
        let d0 = crc_valid_frame_extent(4_096, 0x72);

        assert_eq!(
            compose_flat7x_au_byte_sequence(&validation, &i0[..i0.len() - 1], &d0),
            Err(Flat7xAuByteSequenceError::I0FrameExtentMismatch)
        );
        let mut corrupt_d0 = d0.clone();
        corrupt_d0[100] ^= 1;
        assert_eq!(
            compose_flat7x_au_byte_sequence(&validation, &i0, &corrupt_d0),
            Err(Flat7xAuByteSequenceError::D0FrameCrc)
        );
    }

    #[test]
    fn existing_5_1_production_profile_resolves_unchanged() {
        let selected = EncodingProfile::selected();
        let resolved = resolve_selected_profile(&selected).unwrap();

        assert_eq!(
            resolved.joc_reconstruction_layout(),
            JocReconstructionLayout::ExistingSelectedFivePointOne
        );
        assert_eq!(resolved.codec_rate(), CodecRateProfile::Kbps1024);
    }

    #[test]
    fn d0_information_suffix_rejects_missing_or_wrong_chanmap_atomically() {
        let approved = Flat7xI0D0TargetPlan::approved_contract();
        for channel_map in [None, Some(DependentChannelMap::from_numeric_value(0x0200))] {
            let mut target = approved;
            target.d0.channel_map = channel_map;
            let mut bytes = [0xa5_u8; 16];
            let original = bytes;
            let error;
            let cursor;
            {
                let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(71)).unwrap();
                error = append_flat7x_d0_info_and_addbsi(&mut writer, target, false, None, None)
                    .unwrap_err();
                cursor = writer.cursor();
            }
            assert_eq!(error, Flat7xD0InfoAddbsiError::UnsupportedTargetContext);
            assert_eq!(cursor.bit_offset(), 71);
            assert_eq!(bytes, original);
        }
    }
}
