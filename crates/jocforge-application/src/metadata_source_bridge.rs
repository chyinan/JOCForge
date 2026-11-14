// pattern: Functional Core
//! Builds same-source JOC and metadata-body proposals from one ready MAIN callback.

use jocforge_domain::{
    joc::{
        coefficient_fitting::{Flat7xCoefficientRowsProposal, Flat7xCoefficientRowsProviderOutput},
        coefficient_representation::RepresentationRetryProvider,
        emdf::metadata_bodies::{
            MetadataBodiesError, MetadataBodiesProposal, assemble_metadata_bodies,
        },
        emdf::metadata_owner::{MetadataOwnerError, MetadataSourceProposal, MetadataStageInput},
        fit_score::FitScoreProvider,
        programme_state::{
            JocSourceStageErrorKind, SelectedJocAfterProposal, SelectedJocStateError,
            stage_selected_joc_source_with_flat7x_rows,
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
    let rows = Flat7xCoefficientRowsProposal::from_provider_output(projection, provider_output)
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
    prepare_main_metadata_bodies_for_callback_inner(input, gtc_transaction_id, None)
}

/// Prepares the Flat7x JOC/F13 body proposal from explicit same-source seven-row input.
pub fn prepare_main_metadata_bodies_for_callback_with_flat7x_rows(
    input: MainMetadataBodiesInput<'_>,
    gtc_transaction_id: CallbackTransactionId,
    flat7x_rows: Flat7xCoefficientRowsProviderOutput<'_>,
) -> Result<MainMetadataBodiesCandidate, MetadataSourceBridgeError> {
    prepare_main_metadata_bodies_for_callback_inner(input, gtc_transaction_id, Some(flat7x_rows))
}

fn prepare_main_metadata_bodies_for_callback_inner(
    input: MainMetadataBodiesInput<'_>,
    gtc_transaction_id: CallbackTransactionId,
    flat7x_rows: Option<Flat7xCoefficientRowsProviderOutput<'_>>,
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
    let joc_source_route =
        select_main_metadata_joc_source_route(joc_profile, flat7x_rows.is_some())
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
            let output = flat7x_rows.ok_or_else(flat7x_source_unavailable)?;
            stage_flat7x_joc_source_from_provider_output(
                joc_before,
                projection,
                source_transaction_id.clone(),
                output,
                joc_profile,
            )
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
        joc::programme_state::{JocSourceStageErrorKind, SelectedJocStateError},
        joc::{
            coefficient_fitting::{CoefficientFitState, Flat7xCoefficientRowsProviderOutput},
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
            transaction_id,
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
        let input = CoefficientPayloadInput {
            downmix_index: 1,
            extension_config: 0,
            clip_gain_index: 128,
            sequence_count: 0,
            time_slot_limit: 24,
            capacity_bytes: 372,
            objects: (0..15)
                .map(|_| CoefficientPayloadObject {
                    present: true,
                    band_enum: 0,
                    mode: 1,
                    points: vec![point.clone()],
                    offsets: None,
                })
                .collect(),
        };
        let rows = Flat7xCoefficientRows::new(&input).unwrap();
        let provider_output = Flat7xCoefficientRowsProviderOutput::new(
            rows,
            CoefficientFitState::with_generation_for_test(1),
        );
        let before = SelectedJocState::cold();
        let producer_transaction_id = CallbackTransactionId::new("flat7x-source-s0").unwrap();
        let flat7x =
            resolve_selected_profile(&EncodingProfile::selected_with_joc_reconstruction_layout(
                JocReconstructionLayout::Flat7x,
            ))
            .unwrap();

        let after = stage_flat7x_joc_source_from_provider_output(
            &before,
            &projection,
            producer_transaction_id.clone(),
            provider_output,
            flat7x,
        )
        .unwrap();

        assert_eq!(after.after().generation(), 1);
        let provenance = after.id14_body().provenance();
        assert_eq!(provenance.producer_generation().value(), 0);
        assert_eq!(provenance.dispatch().value(), 1);
        assert_eq!(
            provenance.producer_transaction_id(),
            Some(&producer_transaction_id)
        );
    }
}
