// pattern: Functional Core
//! Builds same-source JOC and metadata-body proposals from one ready MAIN callback.

use jocforge_domain::{
    codec::owner::Flat7xD0SourceGenerationReceipt,
    joc::{
        coefficient_fitting::{
            FitSourceLineage, Flat7xCoefficientRowsProposal, Flat7xCoefficientRowsProviderOutput,
        },
        coefficient_representation::RepresentationRetryProvider,
        emdf::metadata_bodies::{
            MetadataBodiesError, MetadataBodiesProposal, assemble_metadata_bodies,
        },
        emdf::metadata_owner::{MetadataOwnerError, MetadataSourceProposal, MetadataStageInput},
        fit_score::FitScoreProvider,
        programme_state::{
            JocSourceStageErrorKind, SelectedJocAfterProposal, SelectedJocStateError,
            stage_selected_joc_source_with_flat7x_rows,
            stage_selected_joc_source_with_seven_carrier_fit,
            stage_selected_joc_source_with_transaction_for_profile,
        },
    },
    meter::measurement_dependents::MeasurementDependentOwnerInputs,
    profile::{
        CarrierCodecProfile, JocPayloadDownmixIndex, JocReconstructionLayout,
        ResolvedEncodingProfile, SubstreamTopology,
    },
    spatial::{
        QmfReceiverFunctionalProvider, SpatialAudioCallbackOutcome,
        spatial_audio_state::SelectedJocQmfProjection,
    },
    timeline::{
        AdmSampleCount, ClockError, PublicationDisposition, programme_counts,
        publication_for_source,
    },
    transaction::{CallbackTransactionId, GtcOwnerReceipt},
};

pub use jocforge_domain::joc::{
    emdf::metadata_owner::{MetadataEofParentHandoff, MetadataEofProposal, MetadataOwner},
    programme_state::SelectedJocState,
};

use crate::source_frame_bridge::Flat7xSevenCarrierCoefficientCandidate;
use crate::{PreparedJobInput, spatial_callback::MainSpatialCallbackCandidate};

/// Failures while deriving the F13 owner from a same-input Meter receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataOwnerPreparationError {
    MeasurementIdentityMismatch,
    SourceSchedule(ClockError),
    Owner(MetadataOwnerError),
}

/// Creates the cold F13 owner from the Meter-derived ID1 value and approved provider bytes.
///
/// `expected_sources` comes from the selected typed callback/source schedule. The owner is only
/// returned as a value; the programme parent still owns installation and source commits.
pub fn prepare_metadata_owner_from_measurement(
    prepared: &PreparedJobInput,
    measurement: &MeasurementDependentOwnerInputs,
    provider_file: &[u8],
    wrapper_file: &[u8],
) -> Result<MetadataOwner, MetadataOwnerPreparationError> {
    let receipt = measurement.receipt();
    let id1 = measurement.id1_configuration();
    if receipt.programme_id() != prepared.programme_id()
        || *receipt.input_sha256() != prepared.domain_identity().resource().input_sha256()
        || receipt.config_sha256()
            != &prepared
                .domain_identity()
                .normalized_configuration()
                .config_sha256()
        || id1.programme_id() != receipt.programme_id()
        || id1.attempt() != receipt.attempt()
        || id1.measurement_input_sha256() != receipt.input_sha256()
        || id1.measurement_config_sha256() != receipt.config_sha256()
        || id1.meter_terminal_sha256() != receipt.meter_terminal_sha256()
    {
        return Err(MetadataOwnerPreparationError::MeasurementIdentityMismatch);
    }

    let source_samples = AdmSampleCount::new(prepared.configuration().source_samples())
        .map_err(MetadataOwnerPreparationError::SourceSchedule)?;
    let expected_sources = programme_counts(source_samples)
        .map_err(MetadataOwnerPreparationError::SourceSchedule)?
        .as_tuple()
        .3;
    MetadataOwner::new(id1.clone(), expected_sources, provider_file, wrapper_file)
        .map_err(MetadataOwnerPreparationError::Owner)
}

/// Errors while binding the F05JOC proposal and F13 body set to one MAIN callback.
#[derive(Debug)]
pub enum MetadataSourceBridgeError {
    SpatialCallbackIdentityMismatch,
    ProgrammeIdentityMismatch,
    SpatialAudioStillPriming,
    ReaderTargetReceiptMissing,
    JocProducerTransactionMismatch {
        run_id: String,
        input_sha256: String,
        stage: &'static str,
        owner_id: &'static str,
        expected_producer_transaction_id: String,
        actual_producer_transaction_id: Option<String>,
        parent_callback_transaction_id: String,
        callback_index: u64,
        callback_phase: &'static str,
        source_index: u64,
        dispatch: u64,
        owner_before_generation: u64,
        proposed_owner_after_generation: u64,
        producer_generation: u64,
        writer_generation: u64,
    },
    Joc(SelectedJocStateError),
    Flat7xSourceGenerationReceiptMissing,
    MetadataBodies(MetadataBodiesError),
}

/// Typed providers and owner before-state for one source-scoped JOC/F13 body proposal.
pub struct MainMetadataBodiesInput<'a> {
    pub prepared: &'a PreparedJobInput,
    pub joc_profile: ResolvedEncodingProfile,
    pub spatial_callback: &'a MainSpatialCallbackCandidate,
    pub joc_before: &'a SelectedJocState,
    pub source_transaction_id: CallbackTransactionId,
    pub qmf_functional: &'a QmfReceiverFunctionalProvider,
    pub fit_score: &'a FitScoreProvider,
    pub retry: &'a RepresentationRetryProvider,
}

/// Noncommitting F05JOC after-state and F13 source body set for one ready callback.
#[must_use]
pub struct MainMetadataBodiesCandidate {
    joc_after: SelectedJocAfterProposal,
    joc_receipt: GtcOwnerReceipt,
    metadata_bodies: MetadataBodiesProposal,
}

/// Binds Flat7x provider output to one accepted QMF source before JOC dispatch and ID14 staging.
pub fn stage_flat7x_joc_source_from_provider_output(
    before: &SelectedJocState,
    projection: &SelectedJocQmfProjection,
    producer_transaction_id: CallbackTransactionId,
    expected_source_generation_receipt: &Flat7xD0SourceGenerationReceipt,
    provider_output: Flat7xCoefficientRowsProviderOutput<'_>,
    profile: ResolvedEncodingProfile,
) -> Result<SelectedJocAfterProposal, SelectedJocStateError> {
    if select_main_metadata_joc_source_route(profile, true)?
        != MainMetadataJocSourceRoute::Flat7xRows
    {
        return Err(SelectedJocStateError::Source(
            JocSourceStageErrorKind::Flat7xCoefficientProfileMismatch,
        ));
    }
    let rows = Flat7xCoefficientRowsProposal::from_provider_output(
        before.fit(),
        expected_source_generation_receipt,
        &provider_output,
    )
    .map_err(|_| {
        SelectedJocStateError::Source(JocSourceStageErrorKind::Flat7xCoefficientLineageMismatch)
    })?;
    stage_selected_joc_source_with_flat7x_rows(
        before,
        projection,
        producer_transaction_id,
        rows,
        profile,
    )
}

/// Sends the local C=7 fit rows through the existing Flat7x row validator and ID14 writer seam.
///
/// The Codec owner must supply the real D0 source-generation receipt for this source before this
/// function can create the row receipt. The returned JOC after-state remains a proposal.
pub fn stage_flat7x_joc_source_from_seven_carrier_fit(
    before: &SelectedJocState,
    projection: &SelectedJocQmfProjection,
    candidate: &crate::source_frame_bridge::Flat7xSevenCarrierCoefficientCandidate,
    expected_source_generation_receipt: &Flat7xD0SourceGenerationReceipt,
    retry_provider: &RepresentationRetryProvider,
    profile: ResolvedEncodingProfile,
) -> Result<SelectedJocAfterProposal, SelectedJocStateError> {
    if select_main_metadata_joc_source_route(profile, true)?
        != MainMetadataJocSourceRoute::Flat7xRows
    {
        return Err(SelectedJocStateError::Source(
            JocSourceStageErrorKind::Flat7xCoefficientProfileMismatch,
        ));
    }

    let fit = candidate.fit();
    let fit_lineage = FitSourceLineage::from_projection(projection);
    let source_identity = fit.qmf_source_identity();
    let role_receipt = candidate.carrier_windows().role_map_receipt();
    if fit.target_lineage() != &fit_lineage
        || source_identity.callback_lineage() != projection.transaction_id()
        || source_identity.source_index() != projection.source_index()
        || source_identity.source_transaction_id()
            != expected_source_generation_receipt.source_transaction_id()
        || source_identity.callback_lineage()
            != expected_source_generation_receipt.callback_lineage()
        || source_identity.source_index() != expected_source_generation_receipt.source_index()
        || source_identity.analysis_generation()
            != expected_source_generation_receipt.analysis_generation()
        || role_receipt.source_index() != source_identity.source_index()
        || role_receipt.source_transaction_id() != source_identity.source_transaction_id()
        || role_receipt
            .shared_analysis_identity()
            .analysis_after_generation()
            != source_identity.analysis_generation()
    {
        return Err(SelectedJocStateError::Source(
            JocSourceStageErrorKind::Flat7xCoefficientLineageMismatch,
        ));
    }

    stage_selected_joc_source_with_seven_carrier_fit(
        before,
        projection,
        expected_source_generation_receipt.source_transaction_id(),
        fit,
        expected_source_generation_receipt,
        retry_provider,
        profile,
    )
}

/// Failures while staging F13 source state from same-source ID11/ID2/ID14 bodies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MainMetadataSourceStageError {
    SourceClock(ClockError),
    MetadataOwner(MetadataOwnerError),
}

/// Noncommitting F05JOC and F13 owner-source candidates for one codec source.
#[must_use]
pub struct MainMetadataSourceCandidate {
    joc_after: SelectedJocAfterProposal,
    joc_receipt: GtcOwnerReceipt,
    metadata_source: MetadataSourceProposal,
}

impl MainMetadataSourceCandidate {
    pub const fn joc_after(&self) -> &SelectedJocAfterProposal {
        &self.joc_after
    }

    pub const fn joc_receipt(&self) -> &GtcOwnerReceipt {
        &self.joc_receipt
    }

    pub const fn metadata_source(&self) -> &MetadataSourceProposal {
        &self.metadata_source
    }

    pub fn into_parts(
        self,
    ) -> (
        SelectedJocAfterProposal,
        GtcOwnerReceipt,
        MetadataSourceProposal,
    ) {
        (self.joc_after, self.joc_receipt, self.metadata_source)
    }
}

impl MainMetadataBodiesCandidate {
    pub const fn joc_after(&self) -> &SelectedJocAfterProposal {
        &self.joc_after
    }

    pub const fn joc_receipt(&self) -> &GtcOwnerReceipt {
        &self.joc_receipt
    }

    pub const fn metadata_bodies(&self) -> &MetadataBodiesProposal {
        &self.metadata_bodies
    }

    pub fn into_parts(
        self,
    ) -> (
        SelectedJocAfterProposal,
        GtcOwnerReceipt,
        MetadataBodiesProposal,
    ) {
        (self.joc_after, self.joc_receipt, self.metadata_bodies)
    }
}

/// Stages one prepared ID11/ID2/ID14 body set into the caller-owned MetadataOwner clone.
///
/// The source index and AU projection come from the typed body lineage. The owner returns an
/// EMDF container candidate for published sources and still advances its producer/queue state
/// for suppressed sources. The caller retains all owner installation authority.
pub fn stage_main_metadata_source(
    metadata_owner: &MetadataOwner,
    bodies_candidate: MainMetadataBodiesCandidate,
    consumer_transaction_id: CallbackTransactionId,
) -> Result<MainMetadataSourceCandidate, MainMetadataSourceStageError> {
    let (joc_after, joc_receipt, metadata_bodies) = bodies_candidate.into_parts();
    let au_index = match publication_for_source(metadata_bodies.lineage().source_index())
        .map_err(MainMetadataSourceStageError::SourceClock)?
    {
        PublicationDisposition::Suppressed { .. } => None,
        PublicationDisposition::Published(consumption) => Some(consumption.au),
    };
    let metadata_source = metadata_owner
        .stage_source(MetadataStageInput {
            au_index,
            transaction_id: consumer_transaction_id,
            bodies: metadata_bodies,
        })
        .map_err(MainMetadataSourceStageError::MetadataOwner)?;

    Ok(MainMetadataSourceCandidate {
        joc_after,
        joc_receipt,
        metadata_source,
    })
}

/// Stages F05JOC and the F13 ID11/ID2/ID14 body set from one selected MAIN callback.
///
/// The metadata owner still needs a finalized measurement-derived ID1 configuration. This
/// function returns body values only; it creates no MetadataSourceProposal, EMDF container, or
/// owner commit.
pub fn prepare_main_metadata_bodies_from_spatial_callback(
    input: MainMetadataBodiesInput<'_>,
) -> Result<MainMetadataBodiesCandidate, MetadataSourceBridgeError> {
    let gtc_transaction_id = input.source_transaction_id.clone();
    prepare_main_metadata_bodies_for_callback(input, gtc_transaction_id)
}

/// Prepares same-source F05/F13 body values and binds the JOC GTC receipt to its parent CALLBACK.
pub fn prepare_main_metadata_bodies_for_callback(
    input: MainMetadataBodiesInput<'_>,
    gtc_transaction_id: CallbackTransactionId,
) -> Result<MainMetadataBodiesCandidate, MetadataSourceBridgeError> {
    prepare_main_metadata_bodies_for_callback_inner(input, gtc_transaction_id, None, None, None)
}

/// Prepares the Flat7x JOC/F13 body proposal from explicit same-source seven-row input.
pub fn prepare_main_metadata_bodies_for_callback_with_flat7x_rows(
    input: MainMetadataBodiesInput<'_>,
    gtc_transaction_id: CallbackTransactionId,
    flat7x_rows: Flat7xCoefficientRowsProviderOutput<'_>,
    source_generation_receipt: &Flat7xD0SourceGenerationReceipt,
) -> Result<MainMetadataBodiesCandidate, MetadataSourceBridgeError> {
    prepare_main_metadata_bodies_for_callback_inner(
        input,
        gtc_transaction_id,
        Some(flat7x_rows),
        Some(source_generation_receipt),
        None,
    )
}

/// Prepares the Flat7x JOC/F13 body proposal from locally fitted same-source seven-carrier windows.
pub fn prepare_main_metadata_bodies_for_callback_with_seven_carrier_fit(
    input: MainMetadataBodiesInput<'_>,
    gtc_transaction_id: CallbackTransactionId,
    fit_candidate: Flat7xSevenCarrierCoefficientCandidate,
    source_generation_receipt: &Flat7xD0SourceGenerationReceipt,
) -> Result<MainMetadataBodiesCandidate, MetadataSourceBridgeError> {
    prepare_main_metadata_bodies_for_callback_inner(
        input,
        gtc_transaction_id,
        None,
        Some(source_generation_receipt),
        Some(fit_candidate),
    )
}

fn prepare_main_metadata_bodies_for_callback_inner(
    input: MainMetadataBodiesInput<'_>,
    gtc_transaction_id: CallbackTransactionId,
    flat7x_rows: Option<Flat7xCoefficientRowsProviderOutput<'_>>,
    source_generation_receipt: Option<&Flat7xD0SourceGenerationReceipt>,
    flat7x_fit_candidate: Option<Flat7xSevenCarrierCoefficientCandidate>,
) -> Result<MainMetadataBodiesCandidate, MetadataSourceBridgeError> {
    let MainMetadataBodiesInput {
        prepared,
        joc_profile,
        spatial_callback,
        joc_before,
        source_transaction_id,
        qmf_functional,
        fit_score,
        retry,
    } = input;
    let joc_source_route = select_main_metadata_joc_source_route(
        joc_profile,
        flat7x_rows.is_some() || flat7x_fit_candidate.is_some(),
    )
    .map_err(MetadataSourceBridgeError::Joc)?;
    if !spatial_callback.is_bound_to(prepared) {
        return Err(MetadataSourceBridgeError::SpatialCallbackIdentityMismatch);
    }
    let proposal = spatial_callback.proposal();
    if proposal.transaction_id().programme_id() != prepared.programme_id() {
        return Err(MetadataSourceBridgeError::ProgrammeIdentityMismatch);
    }
    let projection = match proposal.spatial_audio_outcome() {
        SpatialAudioCallbackOutcome::Priming => {
            return Err(MetadataSourceBridgeError::SpatialAudioStillPriming);
        }
        SpatialAudioCallbackOutcome::Joc(projection) => projection,
    };
    let reader_receipt = proposal
        .spatial_control_after_proposal()
        .reader_target_receipt()
        .ok_or(MetadataSourceBridgeError::ReaderTargetReceiptMissing)?;
    let joc_after = match joc_source_route {
        MainMetadataJocSourceRoute::Flat7xRows => {
            let expected_source_generation_receipt = source_generation_receipt
                .ok_or(MetadataSourceBridgeError::Flat7xSourceGenerationReceiptMissing)?;
            if let Some(fit_candidate) = flat7x_fit_candidate {
                stage_flat7x_joc_source_from_seven_carrier_fit(
                    joc_before,
                    projection,
                    &fit_candidate,
                    expected_source_generation_receipt,
                    retry,
                    joc_profile,
                )
            } else {
                let output = flat7x_rows.ok_or_else(flat7x_source_unavailable)?;
                stage_flat7x_joc_source_from_provider_output(
                    joc_before,
                    projection,
                    source_transaction_id.clone(),
                    expected_source_generation_receipt,
                    output,
                    joc_profile,
                )
            }
        }
        MainMetadataJocSourceRoute::SelectedFivePointOneFit => {
            stage_selected_joc_source_with_transaction_for_profile(
                joc_before,
                projection,
                source_transaction_id.clone(),
                qmf_functional,
                fit_score,
                retry,
                joc_profile,
            )
        }
    }
    .map_err(MetadataSourceBridgeError::Joc)?;
    let producer_provenance = joc_after.id14_body().provenance();
    if producer_provenance.producer_transaction_id() != Some(&source_transaction_id) {
        let identity = proposal.transaction_id();
        let attempt = identity.attempt_id();
        let generation_after = joc_after.after().generation();
        return Err(MetadataSourceBridgeError::JocProducerTransactionMismatch {
            run_id: format!(
                "{}-attempt-{}",
                prepared.programme_id().as_str(),
                attempt.value()
            ),
            input_sha256: sha256_hex(&prepared.resource_identity().input_sha256()),
            stage: "F05JOC GTC owner receipt admission",
            owner_id: "joc",
            expected_producer_transaction_id: source_transaction_id.as_str().to_owned(),
            actual_producer_transaction_id: producer_provenance
                .producer_transaction_id()
                .map(|value| value.as_str().to_owned()),
            parent_callback_transaction_id: gtc_transaction_id.as_str().to_owned(),
            callback_index: u64::from(identity.callback_index().value()),
            callback_phase: "source",
            source_index: producer_provenance.producer_generation().value(),
            dispatch: producer_provenance.dispatch().value(),
            owner_before_generation: joc_before.generation(),
            proposed_owner_after_generation: generation_after,
            producer_generation: producer_provenance.producer_generation().value(),
            writer_generation: producer_provenance.writer().value(),
        });
    }
    let joc_receipt = joc_after
        .gtc_owner_receipt_for_parent(joc_before, &source_transaction_id, gtc_transaction_id)
        .map_err(MetadataSourceBridgeError::Joc)?;
    let metadata_bodies = assemble_metadata_bodies(
        prepared.configuration(),
        projection,
        reader_receipt,
        joc_after.id14_body().clone(),
    )
    .map_err(MetadataSourceBridgeError::MetadataBodies)?;

    Ok(MainMetadataBodiesCandidate {
        joc_after,
        joc_receipt,
        metadata_bodies,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MainMetadataJocSourceRoute {
    SelectedFivePointOneFit,
    Flat7xRows,
}

fn select_main_metadata_joc_source_route(
    profile: ResolvedEncodingProfile,
    flat7x_rows_present: bool,
) -> Result<MainMetadataJocSourceRoute, SelectedJocStateError> {
    let selected_five_point_one = profile.carrier_codec() == CarrierCodecProfile::IndependentEac3
        && profile.substream_topology()
            == SubstreamTopology::SelectedSingleIndependentZeroDependent
        && profile.joc_reconstruction_layout()
            == JocReconstructionLayout::ExistingSelectedFivePointOne
        && profile.joc_payload_downmix_index() == JocPayloadDownmixIndex::FiveChannel;
    let flat7x = profile.carrier_codec() == CarrierCodecProfile::Flat7xAc3I0DependentEac3D0
        && profile.substream_topology() == SubstreamTopology::Flat7xIndependentI0DependentD0
        && profile.joc_reconstruction_layout() == JocReconstructionLayout::Flat7x
        && profile.joc_payload_downmix_index() == JocPayloadDownmixIndex::SevenChannel;

    if selected_five_point_one && !flat7x_rows_present {
        return Ok(MainMetadataJocSourceRoute::SelectedFivePointOneFit);
    }
    if flat7x && flat7x_rows_present {
        return Ok(MainMetadataJocSourceRoute::Flat7xRows);
    }
    if flat7x {
        return Err(SelectedJocStateError::Source(
            JocSourceStageErrorKind::Flat7xCoefficientSourceUnavailable,
        ));
    }
    Err(SelectedJocStateError::Source(
        JocSourceStageErrorKind::Flat7xCoefficientProfileMismatch,
    ))
}

fn flat7x_source_unavailable() -> MetadataSourceBridgeError {
    MetadataSourceBridgeError::Joc(SelectedJocStateError::Source(
        JocSourceStageErrorKind::Flat7xCoefficientSourceUnavailable,
    ))
}

fn sha256_hex(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod flat7x_rows_route_tests {
    use super::{
        MainMetadataJocSourceRoute, SelectedJocState, select_main_metadata_joc_source_route,
        stage_flat7x_joc_source_from_provider_output,
    };
    use jocforge_domain::{
        codec::owner::Flat7xD0SourceGenerationReceipt,
        codec::{
            ac3_bsi_tail::Flat7xI0Ac3BsiTailInput,
            access_unit_composer::{
                Flat7xI0D0OpaqueFrameSpans, OpaqueD0FrameBytes, OpaqueI0FrameBytes,
                write_flat7x_i0_d0_access_unit_bytes,
            },
            bitio::{FrameBitCursor, FrameBitWriter},
            d0_frame_assembler::{
                D0Block2SkipFieldInput, D0BlockTwoAudioSpans, D0FrameAudioBlockInput,
                Flat7xD0FrameAssemblyError, Flat7xD0FrameBitInput, Flat7xD0FrameTailPolicy,
                OpaqueD0AudioBitSpan, assemble_flat7x_d0_frame_bits_with_tail_policy,
            },
            d0_frame_layout::{CallerOwnedUnboundD0Block2InsertionPoint, D0SixBlockLayoutError},
            emdf_skip_field::{D0EmdfSkipFieldWriteError, flat7x_d0_block2_skipfld_bit_length},
            flat7x_d0_frame_controls::{
                Flat7xD0AllocationCandidateQ, Flat7xD0AudioFrameControlPolicy,
            },
            frame_builder::{Flat7xD0BsiStructure, Flat7xD0Ci16Addbsi},
            i0_frame_assembler::{
                CallerOwnedI0CrcrsvSeed, Flat7xI0Ac3FrameAssemblyReceipt,
                Flat7xI0Ac3FrameHeaderFields, Flat7xI0Ac3FrameInput, I0AuxDataInput,
                OpaqueI0Ac3BitSpan, assemble_flat7x_i0_ac3_frame_from_fields,
            },
        },
        integrity::crc16::eac3_crc16,
        joc::programme_state::{JocSourceStageErrorKind, SelectedJocStateError},
        joc::{
            coefficient_fitting::{
                FitSourceLineage, Flat7xCoefficientRowsProviderOutput,
                Flat7xCoefficientRowsSourceReceipt,
            },
            coefficient_payload::{
                CoefficientPayloadInput, CoefficientPayloadObject, CoefficientPayloadPoint,
                Flat7xCoefficientRows,
            },
        },
        profile::{EncodingProfile, JocReconstructionLayout, resolve_selected_profile},
        spatial::{SpatialAudioMachine, spatial_audio_state::SelectedJocQmfProjection},
        timeline::{CodecSourceIndex, FrontendCallbackIndex},
        transaction::{AttemptId, CallbackLineageIdentity, CallbackTransactionId, ProgrammeId},
    };

    fn read_bits(bytes: &[u8], start_bit: u32, width: u8) -> u32 {
        (0..u32::from(width)).fold(0, |value, offset| {
            let bit = start_bit + offset;
            (value << 1) | u32::from((bytes[(bit / 8) as usize] >> (7 - bit % 8)) & 1)
        })
    }

    fn synthetic_d0_frame_input<'a>(
        blocks: &'a [[u8; 1]; 6],
        block_two_before: &'a [u8; 1],
        block_two_after: &'a [u8; 1],
        skip_fields: &'a [D0Block2SkipFieldInput<'a>],
    ) -> Flat7xD0FrameBitInput<'a> {
        let span = |bytes: &'a [u8]| OpaqueD0AudioBitSpan::new(bytes, 0, 8);
        Flat7xD0FrameBitInput {
            audio_frame_controls: Flat7xD0AudioFrameControlPolicy::new(
                Flat7xD0AllocationCandidateQ::new(1).unwrap(),
            ),
            audio_blocks: [
                D0FrameAudioBlockInput::Opaque(span(&blocks[0])),
                D0FrameAudioBlockInput::Opaque(span(&blocks[1])),
                D0FrameAudioBlockInput::BlockTwo(D0BlockTwoAudioSpans {
                    before_skipfield: span(block_two_before),
                    skip_fields,
                    after_skipfield: span(block_two_after),
                }),
                D0FrameAudioBlockInput::Opaque(span(&blocks[3])),
                D0FrameAudioBlockInput::Opaque(span(&blocks[4])),
                D0FrameAudioBlockInput::Opaque(span(&blocks[5])),
            ],
        }
    }

    fn synthetic_d0_bsi() -> Flat7xD0BsiStructure {
        Flat7xD0BsiStructure {
            dialnorm: 16,
            compr: None,
            mixing: None,
            block_mix: [None; 6],
            information: None,
            addbsi: Flat7xD0Ci16Addbsi::new([0x01, 0x10]),
        }
    }

    fn synthetic_i0_frame(
        target: jocforge_domain::profile::Flat7xI0D0TargetPlan,
    ) -> ([u8; 2_560], Flat7xI0Ac3FrameAssemblyReceipt) {
        let audio_block_bits = [3_399_u32, 3_399, 3_399, 3_399, 3_399, 3_398];
        let audio_bytes: [Vec<u8>; 6] = std::array::from_fn(|index| {
            let bytes = usize::try_from(audio_block_bits[index].div_ceil(8)).unwrap();
            vec![0x30 + u8::try_from(index).unwrap(); bytes]
        });
        let audio_blocks = std::array::from_fn(|index| {
            OpaqueI0Ac3BitSpan::new(&audio_bytes[index], 0, audio_block_bits[index])
        });
        let input = Flat7xI0Ac3FrameInput {
            header_fields: Flat7xI0Ac3FrameHeaderFields {
                frmsizecod: 36,
                bsmod: 0,
                cmixlev: 0,
                surmixlev: 0,
                dialnorm: 16,
                compr: None,
            },
            bsi_tail: Flat7xI0Ac3BsiTailInput {
                langcode: None,
                audio_production: None,
                copyright: false,
                original_bitstream: false,
                time_code_1: None,
                time_code_2: None,
                additional_bsi: None,
            },
            audio_blocks,
            auxdata: I0AuxDataInput::Absent,
            crcrsv_seed: CallerOwnedI0CrcrsvSeed { value: false },
        };
        let mut frame_bytes = [0_u8; 2_560];
        let receipt;
        {
            let mut writer = FrameBitWriter::new(&mut frame_bytes, FrameBitCursor::new(0)).unwrap();
            receipt = assemble_flat7x_i0_ac3_frame_from_fields(target, input, &mut writer).unwrap();
        }
        (frame_bytes, receipt)
    }

    #[test]
    fn application_route_keeps_five_point_one_and_requires_flat7x_provider_output() {
        let five_point_one = resolve_selected_profile(&EncodingProfile::selected()).unwrap();
        let flat7x =
            resolve_selected_profile(&EncodingProfile::selected_with_joc_reconstruction_layout(
                JocReconstructionLayout::Flat7x,
            ))
            .unwrap();

        assert_eq!(
            select_main_metadata_joc_source_route(five_point_one, false),
            Ok(MainMetadataJocSourceRoute::SelectedFivePointOneFit)
        );
        assert_eq!(
            select_main_metadata_joc_source_route(flat7x, true),
            Ok(MainMetadataJocSourceRoute::Flat7xRows)
        );
        assert!(matches!(
            select_main_metadata_joc_source_route(flat7x, false),
            Err(SelectedJocStateError::Source(
                JocSourceStageErrorKind::Flat7xCoefficientSourceUnavailable
            ))
        ));
        assert!(matches!(
            select_main_metadata_joc_source_route(five_point_one, true),
            Err(SelectedJocStateError::Source(
                JocSourceStageErrorKind::Flat7xCoefficientProfileMismatch
            ))
        ));
    }

    #[cfg(feature = "test-support")]
    #[test]
    fn flat7x_provider_output_crosses_application_bridge_with_same_source_generation() {
        // SYNTHETIC/TEST_ONLY rows and identity-only projection exercise routing/provenance only;
        // they contain no audio samples and do not enter a production coefficient provider.
        let programme_id = ProgrammeId::new("flat7x-application-bridge-test").unwrap();
        let attempt = AttemptId::new(1);
        let callback_index = FrontendCallbackIndex::new(2);
        let transaction_id =
            CallbackLineageIdentity::new(programme_id.clone(), attempt, callback_index);
        let projection = SelectedJocQmfProjection::identity_only_for_test(
            programme_id,
            attempt,
            transaction_id.clone(),
            0,
            SpatialAudioMachine::new_cold(EncodingProfile::selected())
                .unwrap()
                .digest(),
            callback_index,
            CodecSourceIndex::new(0),
        );
        let point = CoefficientPayloadPoint {
            channel_rows: (0..7).map(|_| vec![0.25]).collect(),
            selected_channels: Vec::new(),
        };
        let objects = (0..15)
            .map(|_| CoefficientPayloadObject {
                present: true,
                band_enum: 0,
                mode: 1,
                points: vec![point.clone()],
                offsets: None,
            })
            .collect::<Vec<_>>();
        let input = CoefficientPayloadInput {
            downmix_index: 1,
            extension_config: 0,
            clip_gain_index: 128,
            sequence_count: 0,
            time_slot_limit: 24,
            capacity_bytes: 372,
            objects,
        };
        assert_eq!(input.downmix_index, 1);
        assert_eq!(input.objects.len(), 15);
        assert!(input.objects.iter().all(|object| {
            object.present
                && object.mode == 1
                && object.points.len() == 1
                && object.points[0].channel_rows.len() == 7
                && object.points[0]
                    .channel_rows
                    .iter()
                    .all(|row| row.len() == 1)
        }));
        let rows = Flat7xCoefficientRows::new(&input).unwrap();
        let before = SelectedJocState::cold();
        let producer_transaction_id = CallbackTransactionId::new("flat7x-source-s0").unwrap();
        let source_lineage = FitSourceLineage::from_projection(&projection);
        let source_generation_receipt = Flat7xD0SourceGenerationReceipt::synthetic_for_test(
            producer_transaction_id.clone(),
            1,
            transaction_id,
            CodecSourceIndex::new(0),
            0,
        );
        let source_receipt = Flat7xCoefficientRowsSourceReceipt::new(
            producer_transaction_id.clone(),
            source_lineage,
            source_generation_receipt.clone(),
        )
        .unwrap();
        let provider_output = Flat7xCoefficientRowsProviderOutput::new(rows, source_receipt);
        let flat7x =
            resolve_selected_profile(&EncodingProfile::selected_with_joc_reconstruction_layout(
                JocReconstructionLayout::Flat7x,
            ))
            .unwrap();

        let after = stage_flat7x_joc_source_from_provider_output(
            &before,
            &projection,
            producer_transaction_id.clone(),
            &source_generation_receipt,
            provider_output,
            flat7x,
        )
        .unwrap();

        assert_eq!(after.after().generation(), 1);
        assert_eq!(after.after().fit().temporal(), before.fit().temporal());
        assert_eq!(
            after.after().fit().geometry().carry18_words(),
            before.fit().geometry().carry18_words()
        );
        assert_eq!(
            after.after().fit().geometry().previous24_words(),
            before.fit().geometry().previous24_words()
        );
        assert_eq!(after.after().fit().geometry().generation(), 1);
        let provenance = after.id14_body().provenance();
        assert_eq!(provenance.producer_generation().value(), 0);
        assert_eq!(provenance.dispatch().value(), 1);
        assert_eq!(
            provenance.producer_transaction_id(),
            Some(&producer_transaction_id)
        );
        assert_eq!(provenance.object_count().value(), 15);
        let id14_body = &after.id14_body().body().bytes;
        assert!(!id14_body.is_empty());
        assert_eq!(read_bits(id14_body, 0, 3), 1); // serialized Flat7x selector
        assert_eq!(read_bits(id14_body, 3, 6), 14); // serialized object_count - 1
    }

    #[cfg(feature = "test-support")]
    #[test]
    fn synthetic_i0_d0_headers_and_pair_extent_match_flat7x_target() {
        let target = jocforge_domain::profile::Flat7xI0D0TargetPlan::approved_contract();
        // Wash source-probe receipt reports the sample's I0 AC-3 / D0 E-AC-3 distinction.
        assert_eq!(
            target.i0().role(),
            jocforge_domain::profile::Flat7xSubstreamRole::IndependentI0
        );
        assert_eq!(
            target.i0().codec(),
            jocforge_domain::profile::Flat7xFrameCodec::Ac3
        );
        assert_eq!(target.i0().substream_id().value(), 0);
        assert_eq!(target.i0().bsid(), 6);
        assert_eq!(
            target.d0().role(),
            jocforge_domain::profile::Flat7xSubstreamRole::DependentD0
        );
        assert_eq!(
            target.d0().codec(),
            jocforge_domain::profile::Flat7xFrameCodec::Eac3
        );
        assert_eq!(target.d0().substream_id().value(), 0);
        assert_eq!(target.d0().bsid(), 16);
        // The source sample uses 640 kbps / 2560 bytes for D0; selected target stays 1024 / 4096.
        assert_eq!(target.i0().rate().kbps(), Some(640));
        assert_eq!(target.d0().rate().kbps(), Some(1_024));
        assert_eq!(target.aggregate_rate_kbps(), Some(1_664));
        assert_eq!(target.i0().frame_bytes(), 2_560);
        assert_eq!(target.d0().frame_bytes(), 4_096);
        assert_eq!(target.aggregate_frame_bytes(), 6_656);
        assert_eq!(target.sample_rate_hz(), 48_000);
        assert_eq!(target.audio_blocks_per_frame(), 6);
        assert_eq!(target.samples_per_frame(), 1_536);
        // Wash receipt v2 identifies component[1] as D0 and reports chanmap [6656].
        assert_eq!(target.d0().channel_map().unwrap().value(), 0x1a00);
        assert_eq!(synthetic_d0_bsi().addbsi.bytes(), [0x01, 0x10]);

        // This is deliberately independent of the ID14 body test. These SYNTHETIC/TEST_ONLY
        // bytes exercise opaque skipfld framing only; they are not a complete EMDF container.
        let opaque_skipfield_test_bytes = [0x5a_u8; 16];
        let skip_fields = [D0Block2SkipFieldInput::synthetic_for_test(
            CallerOwnedUnboundD0Block2InsertionPoint::new(8),
            &opaque_skipfield_test_bytes,
        )
        .unwrap()];
        let synthetic_blocks = [[0x20_u8]; 6];
        let block_two_before = [0x31_u8];
        let block_two_after = [0x32_u8];
        let d0_input = synthetic_d0_frame_input(
            &synthetic_blocks,
            &block_two_before,
            &block_two_after,
            &skip_fields,
        );
        let mut d0_frame = [0_u8; 4_096];
        let d0_receipt;
        {
            let mut writer = FrameBitWriter::new(&mut d0_frame, FrameBitCursor::new(0)).unwrap();
            d0_receipt = assemble_flat7x_d0_frame_bits_with_tail_policy(
                target,
                synthetic_d0_bsi(),
                d0_input,
                Flat7xD0FrameTailPolicy::ZeroFillUnusedAuxBits,
                &mut writer,
            )
            .unwrap();
        }
        assert!(!d0_receipt.production_allowed());
        assert_eq!(d0_receipt.layout().bsi_span().end_bit(), 93);
        assert_eq!(d0_receipt.target_frame_bytes(), 4_096);
        assert_eq!(d0_receipt.layout().audio_blocks().len(), 6);
        assert_eq!(read_bits(&d0_frame, 0, 16), 0x0b77);
        assert_eq!(read_bits(&d0_frame, 16, 2), 1); // Dependent E-AC-3 strmtyp
        assert_eq!(read_bits(&d0_frame, 18, 3), 0); // substreamid
        assert_eq!(read_bits(&d0_frame, 21, 11), 2_047); // 4096-byte target frmsiz
        assert_eq!(read_bits(&d0_frame, 32, 2), 0); // fscod: 48 kHz
        assert_eq!(read_bits(&d0_frame, 34, 2), 3); // numblkscod: six blocks
        assert_eq!(read_bits(&d0_frame, 40, 5), 16); // BSID
        assert_eq!(read_bits(&d0_frame, 51, 1), 1); // chanmape
        assert_eq!(read_bits(&d0_frame, 52, 16), 0x1a00);
        assert_eq!(read_bits(&d0_frame, 77, 16), 0x0110); // CI16 addbsi bytes
        assert_eq!(eac3_crc16(&d0_frame[2..]), 0);

        let skip_layout = d0_receipt.layout().block_two_skip_field();
        let skip_length = u16::try_from(opaque_skipfield_test_bytes.len()).unwrap();
        assert_eq!(skip_layout.payload_bytes(), skip_length);
        assert!(skip_layout.payload_bytes() <= 511);
        assert_eq!(
            skip_layout.bit_span().bit_length(),
            flat7x_d0_block2_skipfld_bit_length(skip_length).unwrap()
        );
        let skip_start = skip_layout.bit_span().start_bit();
        assert_eq!(read_bits(&d0_frame, skip_start, 1), 1); // skiple: complete field
        assert_eq!(
            read_bits(&d0_frame, skip_start + 1, 9),
            u32::from(skip_length)
        );
        for (index, byte) in opaque_skipfield_test_bytes.iter().enumerate() {
            assert_eq!(
                read_bits(
                    &d0_frame,
                    skip_start + 10 + u32::try_from(index * 8).unwrap(),
                    8,
                ),
                u32::from(*byte)
            );
        }

        let oversized_opaque_skipfield_bytes = [0x5a_u8; 512];
        let oversized_skip_fields = [D0Block2SkipFieldInput::synthetic_for_test(
            CallerOwnedUnboundD0Block2InsertionPoint::new(8),
            &oversized_opaque_skipfield_bytes,
        )
        .unwrap()];
        let oversized_input = synthetic_d0_frame_input(
            &synthetic_blocks,
            &block_two_before,
            &block_two_after,
            &oversized_skip_fields,
        );
        let mut rejected_d0_frame = [0xa5_u8; 4_096];
        let original_rejected_d0_frame = rejected_d0_frame;
        let overflow_result;
        let overflow_cursor;
        {
            let mut writer =
                FrameBitWriter::new(&mut rejected_d0_frame, FrameBitCursor::new(0)).unwrap();
            overflow_result = assemble_flat7x_d0_frame_bits_with_tail_policy(
                target,
                synthetic_d0_bsi(),
                oversized_input,
                Flat7xD0FrameTailPolicy::ZeroFillUnusedAuxBits,
                &mut writer,
            );
            overflow_cursor = writer.cursor().bit_offset();
        }
        assert!(matches!(
            overflow_result,
            Err(Flat7xD0FrameAssemblyError::Layout(
                D0SixBlockLayoutError::SkipField(D0EmdfSkipFieldWriteError::ContainerTooLarge)
            ))
        ));
        assert_eq!(overflow_cursor, 0);
        assert_eq!(rejected_d0_frame, original_rejected_d0_frame);

        let (i0_frame, i0_receipt) = synthetic_i0_frame(target);
        assert!(!i0_receipt.production_allowed());
        assert_eq!(i0_receipt.frame().target_frame_bytes(), 2_560);
        assert_eq!(i0_receipt.frame().audio_block_spans().len(), 6);
        assert_eq!(read_bits(&i0_frame, 0, 16), 0x0b77);
        assert_eq!(read_bits(&i0_frame, 32, 2), 0); // fscod: 48 kHz
        assert_eq!(read_bits(&i0_frame, 34, 6), 36); // 640-kbps AC-3 frame-size code
        assert_eq!(read_bits(&i0_frame, 40, 5), 6); // BSID
        assert_eq!(eac3_crc16(&i0_frame[2..]), 0);
        let mut access_unit = [0_u8; 6_656];
        let access_unit_receipt;
        {
            let mut writer = FrameBitWriter::new(&mut access_unit, FrameBitCursor::new(0)).unwrap();
            access_unit_receipt = write_flat7x_i0_d0_access_unit_bytes(
                target,
                Flat7xI0D0OpaqueFrameSpans {
                    i0: OpaqueI0FrameBytes::new(&i0_frame),
                    d0: OpaqueD0FrameBytes::new(&d0_frame),
                },
                &mut writer,
            )
            .unwrap();
        }
        assert_eq!(access_unit_receipt.i0_span().start_byte(), 0);
        assert_eq!(access_unit_receipt.i0_span().byte_length(), 2_560);
        assert_eq!(access_unit_receipt.d0_span().start_byte(), 2_560);
        assert_eq!(access_unit_receipt.d0_span().byte_length(), 4_096);
        assert_eq!(access_unit_receipt.total_byte_length(), 6_656);
        assert_eq!(
            &access_unit[..2_560],
            &i0_frame,
            "I0 must be the first frame in the one-AU composition"
        );
        assert_eq!(&access_unit[2_560..], &d0_frame);
        assert!(!access_unit_receipt.production_allowed());
    }
}
