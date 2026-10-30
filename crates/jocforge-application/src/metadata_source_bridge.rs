// pattern: Functional Core
//! Builds same-source JOC and metadata-body proposals from one ready MAIN callback.

use jocforge_domain::{
    joc::{
        coefficient_representation::RepresentationRetryProvider,
        emdf::metadata_bodies::{
            MetadataBodiesError, MetadataBodiesProposal, assemble_metadata_bodies,
        },
        emdf::metadata_owner::{
            MetadataOwner, MetadataOwnerError, MetadataSourceProposal, MetadataStageInput,
        },
        fit_score::FitScoreProvider,
        programme_state::{
            SelectedJocAfterProposal, SelectedJocState, SelectedJocStateError,
            stage_selected_joc_source_with_transaction,
        },
    },
    meter::measurement_dependents::MeasurementDependentOwnerInputs,
    spatial::{QmfReceiverFunctionalProvider, SpatialAudioCallbackOutcome},
    timeline::{
        AdmSampleCount, ClockError, PublicationDisposition, programme_counts,
        publication_for_source,
    },
    transaction::{CallbackTransactionId, GtcOwnerReceipt},
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
    Joc(SelectedJocStateError),
    MetadataBodies(MetadataBodiesError),
}

/// Typed providers and owner before-state for one source-scoped JOC/F13 body proposal.
pub struct MainMetadataBodiesInput<'a> {
    pub prepared: &'a PreparedJobInput,
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
    let MainMetadataBodiesInput {
        prepared,
        spatial_callback,
        joc_before,
        source_transaction_id,
        qmf_functional,
        fit_score,
        retry,
    } = input;
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
    let joc_after = stage_selected_joc_source_with_transaction(
        joc_before,
        projection,
        source_transaction_id.clone(),
        qmf_functional,
        fit_score,
        retry,
    )
    .map_err(MetadataSourceBridgeError::Joc)?;
    let joc_receipt = joc_after
        .gtc_owner_receipt(joc_before, source_transaction_id)
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
