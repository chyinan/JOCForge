// pattern: Imperative Shell
//! Production composition of one selected MAIN F03W/F04 callback from admitted input values.

use crate::PreparedJobInput;
use jocforge_domain::meter::prepared_job_identity::PreparedJobInputIdentity;
use jocforge_domain::{
    frontend::{
        FrontendControlMachine, FrontendControlReplacementCandidate, ValidatedF01EColdProvider,
        ValidatedF03TargetColdProvider, ValidatedFeatureFft4096MathProvider,
        ValidatedFeatureProcessorMathProvider,
        normal_control_snapshot::{
            NormalControlAdmHeldSourceView, NormalControlAdmInputOutput,
            NormalControlAdmInputSnapshot, NormalControlBeforeState, NormalControlHeldSourceView,
            NormalControlPreparedSnapshot, NormalControlSnapshotOutput,
            propose_next_normal_control_adm_input, propose_normal_control_snapshot,
        },
    },
    profile::EncodingProfile,
    profile::resolve_selected_profile,
    source_axis::{SourceAxisBinding, SourceAxisError, SourceAxisOwnerGeneration},
    spatial::{
        SpatialAudioMachine, SpatialControlMachine, SpatialProductionCallbackParentParts,
        SpatialProductionCallbackProposal, ValidatedComponentParameterProvider,
        ValidatedWindowQ15Provider,
        diffusion_filter::DiffusionFilterParameters,
        frontend_stages::{
            FrontendFeatureStageError, stage_spatial_control_frontend_features_from_installed48,
        },
        installed48_source_map::{
            Installed48SourceMap, Installed48SourceMapError, build_selected_installed48_source_map,
        },
        production_callback::selected_audio_provider_bundle::SelectedSpatialAudioProviderBundle,
        production_callback::{
            NormalControlCallbackSource, SpatialProductionCallbackError,
            SpatialProductionCallbackInputs, continue_spatial_production_callback,
        },
        selected_p11_geometry::SelectedP11Geometry,
        selected_p15_geometry::SelectedP15Geometry,
        source_flags::{
            BeforeRegularHistory, CurrentHostPcm, PrefixWriterSet, SourceFlagsError,
            SourceFlagsInput, SourceFlagsParentLineage,
            project_held_normal_control_adm_input_source_controls,
            project_held_normal_control_snapshot_source_controls,
            project_normal_control_adm_input_output_source_controls,
            project_normal_control_snapshot_output_source_controls,
        },
        source_geometry_outer::{
            GlobalSourceGeometryError, build_global_source_geometry_snapshot_from_adm_input_output,
            build_global_source_geometry_snapshot_from_output,
            build_held_global_source_geometry_snapshot,
            build_held_global_source_geometry_snapshot_from_adm_input,
        },
        spatial_control::{
            SpatialControlInstalled48Callback, SpatialControlInstalled48Error,
            SpatialControlInstalled48Input, stage_spatial_control_installed48,
        },
    },
    timeline::FrontendCallbackIndex,
    transaction::{
        AttemptId, CallbackLineageIdentity, CallbackTransactionId, GtcOwnerId, GtcOwnerReceipt,
    },
};
use std::{ops::Deref, sync::Arc};

const MAIN_SPATIAL_CALLBACK_STACK_BYTES: usize = 16 * 1024 * 1024;
const MAIN_NORMAL_CALLBACK_SAMPLES: u64 = 1_536;

/// Cold owner values for the MAIN callback, constructed from the admitted F01E/F03 providers.
#[must_use]
pub struct MainSpatialColdRoots {
    frontend_control: Arc<FrontendControlMachine>,
    spatial_control: SpatialControlMachine,
    spatial_audio: SpatialAudioMachine,
    normal_control_prepared: Arc<NormalControlAdmInputSnapshot>,
}

impl MainSpatialColdRoots {
    pub fn from_selected_providers(
        prepared: &PreparedJobInput,
        attempt: AttemptId,
        f01e: &ValidatedF01EColdProvider,
        f03_target: &ValidatedF03TargetColdProvider,
        p11_geometry: &SelectedP11Geometry,
    ) -> Result<Self, MainSpatialColdRootsError> {
        let selected_profile = prepared.configuration().selected_profile();
        let profile = EncodingProfile::selected_with_rate_and_carriage(
            selected_profile.codec_rate(),
            selected_profile.carriage(),
        );
        let cold_owner = prepared
            .f01e_cold_owner(f01e, attempt)
            .map_err(MainSpatialColdRootsError::PreparedInput)?;
        let normal_control_prepared = Arc::clone(
            &prepared
                .normal_control_adm_input_snapshot(&cold_owner)
                .map_err(MainSpatialColdRootsError::PreparedInput)?,
        );
        let primary_controls = prepared
            .primary_metadata_controls()
            .ok_or(MainSpatialColdRootsError::PrimaryControlsUnavailable)?;
        let source_map = build_selected_installed48_source_map(primary_controls, p11_geometry)
            .map_err(MainSpatialColdRootsError::SourceMap)?;
        let spatial_control = SpatialControlMachine::new_cold_for_source_map(
            profile,
            cold_owner.provider(),
            f03_target,
            &source_map,
        )
        .map_err(MainSpatialColdRootsError::SpatialControl)?;
        let frontend_control = cold_owner.into_machine();
        let spatial_audio = SpatialAudioMachine::new_cold(profile)
            .map_err(MainSpatialColdRootsError::SpatialAudio)?;
        Ok(Self {
            frontend_control: Arc::new(frontend_control),
            spatial_control,
            spatial_audio,
            normal_control_prepared,
        })
    }

    pub fn frontend_control(&self) -> &FrontendControlMachine {
        self.frontend_control.as_ref()
    }

    pub const fn spatial_control(&self) -> &SpatialControlMachine {
        &self.spatial_control
    }

    pub const fn spatial_audio(&self) -> &SpatialAudioMachine {
        &self.spatial_audio
    }

    pub fn normal_control_prepared(&self) -> &Arc<NormalControlAdmInputSnapshot> {
        &self.normal_control_prepared
    }
}

#[derive(Debug)]
pub enum MainSpatialColdRootsError {
    PreparedInput(crate::PreparedJobInputError),
    SourceMap(Installed48SourceMapError),
    PrimaryControlsUnavailable,
    UnsupportedPhysicalRowCount,
    FrontendControl(jocforge_domain::frontend::FrontendControlError),
    SpatialControl(jocforge_domain::spatial::SpatialControlError),
    SpatialAudio(jocforge_domain::spatial::SpatialAudioError),
}

/// F01E normal controls and their matching noncommitting owner replacement candidate.
#[must_use]
pub struct MainFrontendSnapshotCandidate {
    frontend_control: FrontendControlReplacementCandidate,
    controls: NormalControlSnapshotOutput,
}

#[must_use]
pub struct MainFrontendAdmInputSnapshotCandidate {
    frontend_control: FrontendControlReplacementCandidate,
    controls: NormalControlAdmInputOutput,
}

impl MainFrontendAdmInputSnapshotCandidate {
    pub fn into_parts(
        self,
    ) -> (
        FrontendControlReplacementCandidate,
        NormalControlAdmInputOutput,
    ) {
        (self.frontend_control, self.controls)
    }
}

impl MainFrontendSnapshotCandidate {
    pub const fn frontend_control(&self) -> &FrontendControlReplacementCandidate {
        &self.frontend_control
    }

    pub const fn controls(&self) -> &NormalControlSnapshotOutput {
        &self.controls
    }

    pub fn into_parts(
        self,
    ) -> (
        FrontendControlReplacementCandidate,
        NormalControlSnapshotOutput,
    ) {
        (self.frontend_control, self.controls)
    }
}

#[derive(Debug)]
pub enum MainFrontendSnapshotError {
    PreparedInput(crate::PreparedJobInputError),
    BeforeState(jocforge_domain::frontend::normal_control_snapshot::NormalControlSnapshotError),
    Proposal(jocforge_domain::frontend::normal_control_snapshot::NormalControlSnapshotError),
    Stage(jocforge_domain::frontend::FrontendControlError),
    Candidate(jocforge_domain::frontend::FrontendControlError),
}

/// Produces one F01E normal snapshot from the current prepared-input-bound frontend root.
pub fn stage_main_frontend_snapshot(
    prepared: &PreparedJobInput,
    frontend_before: &FrontendControlMachine,
    attempt: AttemptId,
    transaction_id: CallbackTransactionId,
) -> Result<MainFrontendSnapshotCandidate, MainFrontendSnapshotError> {
    if frontend_before.committed().programme_id() != prepared.programme_id()
        || frontend_before.committed().attempt_id() != attempt
    {
        return Err(MainFrontendSnapshotError::Stage(
            jocforge_domain::frontend::FrontendControlError::CallbackIdentityMismatch,
        ));
    }
    let prepared_snapshot: Arc<NormalControlPreparedSnapshot> = prepared
        .normal_control_prepared_snapshot(frontend_before.committed())
        .map_err(MainFrontendSnapshotError::PreparedInput)?;
    let before = NormalControlBeforeState::from_frontend_state(
        frontend_before.committed(),
        frontend_before.generation(),
    )
    .map_err(MainFrontendSnapshotError::BeforeState)?;
    let lineage = CallbackLineageIdentity::new(
        prepared.programme_id().clone(),
        frontend_before.committed().attempt_id(),
        frontend_before.committed().callback_index(),
    );
    let normal = propose_normal_control_snapshot(&before, &prepared_snapshot, lineage)
        .map_err(MainFrontendSnapshotError::Proposal)?;
    let (staged, controls) = frontend_before
        .stage_normal_snapshot(normal)
        .map_err(MainFrontendSnapshotError::Stage)?
        .into_parts();
    let frontend_control = frontend_before
        .replacement_candidate(staged, transaction_id)
        .map_err(MainFrontendSnapshotError::Candidate)?;
    Ok(MainFrontendSnapshotCandidate {
        frontend_control,
        controls,
    })
}

/// Produces one normal MAIN F01E proposal from its admitted dynamic per-input snapshot.
///
/// The typed dynamic owner and source rows remain separate from Meter state. This stage returns
/// an owner candidate and row-shaped control output; the enclosing programme callback still
/// validates and installs them with the other owners.
pub fn stage_main_frontend_adm_input_snapshot(
    prepared: &PreparedJobInput,
    frontend_before: &FrontendControlMachine,
    dynamic_prepared: &Arc<NormalControlAdmInputSnapshot>,
    attempt: AttemptId,
    transaction_id: CallbackTransactionId,
) -> Result<MainFrontendAdmInputSnapshotCandidate, MainFrontendSnapshotError> {
    let prepared_identity = prepared.domain_identity_capability();
    let frontend_state = frontend_before.committed();
    if !Arc::ptr_eq(
        dynamic_prepared.prepared_input_identity(),
        &prepared_identity,
    ) || dynamic_prepared.programme_id() != prepared.programme_id()
        || dynamic_prepared.attempt_id() != attempt
        || frontend_state.programme_id() != prepared.programme_id()
        || frontend_state.attempt_id() != attempt
        || frontend_state.schema() != "F01E_ADM_INPUT_COLD_PROJECTION_V1"
            && frontend_state.schema() != "F01E_ADM_INPUT_NORMAL_STATE_V1"
    {
        return Err(MainFrontendSnapshotError::PreparedInput(
            crate::PreparedJobInputError::PreparedIdentityMismatch,
        ));
    }
    let callback_index = frontend_state.callback_index();
    let lineage =
        CallbackLineageIdentity::new(prepared.programme_id().clone(), attempt, callback_index);
    let (staged, controls) = if callback_index.value() == 0 {
        let proposal = dynamic_prepared
            .propose_first_normal_callback(frontend_state, frontend_before.generation(), lineage)
            .map_err(MainFrontendSnapshotError::Proposal)?;
        let controls = proposal.output(Arc::clone(dynamic_prepared));
        let staged = frontend_before
            .stage_normal_adm_input(Arc::clone(dynamic_prepared), proposal)
            .map_err(MainFrontendSnapshotError::Stage)?;
        (staged, controls)
    } else {
        let before = frontend_state
            .dynamic_after_state()
            .ok_or(MainFrontendSnapshotError::BeforeState(
                jocforge_domain::frontend::normal_control_snapshot::NormalControlSnapshotError::StateShapeMismatch,
            ))?;
        let proposal = propose_next_normal_control_adm_input(before, dynamic_prepared, lineage)
            .map_err(MainFrontendSnapshotError::Proposal)?;
        let controls = proposal.output(Arc::clone(dynamic_prepared));
        let staged = frontend_before
            .stage_normal_adm_input_continuation(Arc::clone(dynamic_prepared), proposal)
            .map_err(MainFrontendSnapshotError::Stage)?;
        (staged, controls)
    };
    let frontend_control = frontend_before
        .replacement_candidate(staged, transaction_id)
        .map_err(MainFrontendSnapshotError::Candidate)?;
    Ok(MainFrontendAdmInputSnapshotCandidate {
        frontend_control,
        controls,
    })
}

/// Inputs for one normal MAIN callback. PCM has already been read, padded, and identity-bound by IO.
pub enum MainNormalControlOutput {
    Selected(NormalControlSnapshotOutput),
    AdmInput(NormalControlAdmInputOutput),
}

impl MainNormalControlOutput {
    pub fn normal_callbacks(&self) -> jocforge_domain::timeline::NormalCallbackCount {
        match self {
            Self::Selected(controls) => controls.normal_callbacks(),
            Self::AdmInput(controls) => controls.normal_callbacks(),
        }
    }
}

pub struct MainSpatialCallbackInput<'a> {
    pub prepared: &'a PreparedJobInput,
    pub controls: MainNormalControlOutput,
    pub physical_pcm_rows: &'a [Vec<u32>],
    pub spatial_control: &'a SpatialControlMachine,
    pub spatial_audio: &'a SpatialAudioMachine,
    pub p11_geometry: &'a SelectedP11Geometry,
    pub p15_geometry: &'a SelectedP15Geometry,
    pub diffusion_parameters: &'a DiffusionFilterParameters,
    pub feature_processor: &'a ValidatedFeatureProcessorMathProvider,
    pub feature_fft: &'a ValidatedFeatureFft4096MathProvider,
    pub window: &'a ValidatedWindowQ15Provider,
    pub component_parameters: &'a ValidatedComponentParameterProvider,
    pub spatial_audio_providers: Option<&'a SelectedSpatialAudioProviderBundle>,
}

/// Inputs for a MAIN cached-drain callback. PCM is read only from SpatialControl's committed cache.
pub struct MainSpatialCachedCallbackInput<'a> {
    pub prepared: &'a PreparedJobInput,
    pub controls: MainHeldControlOutput,
    pub spatial_control: &'a SpatialControlMachine,
    pub spatial_audio: &'a SpatialAudioMachine,
    pub p11_geometry: &'a SelectedP11Geometry,
    pub p15_geometry: &'a SelectedP15Geometry,
    pub diffusion_parameters: &'a DiffusionFilterParameters,
    pub feature_processor: &'a ValidatedFeatureProcessorMathProvider,
    pub feature_fft: &'a ValidatedFeatureFft4096MathProvider,
    pub window: &'a ValidatedWindowQ15Provider,
    pub component_parameters: &'a ValidatedComponentParameterProvider,
    pub spatial_audio_providers: Option<&'a SelectedSpatialAudioProviderBundle>,
}

pub enum MainHeldControlOutput {
    Selected(NormalControlHeldSourceView),
    AdmInput(NormalControlAdmHeldSourceView),
}

enum MainF01eCallback<'a> {
    NormalSelected {
        controls: Box<NormalControlSnapshotOutput>,
        physical_pcm_rows: &'a [Vec<u32>],
    },
    NormalAdmInput {
        controls: Box<NormalControlAdmInputOutput>,
        physical_pcm_rows: &'a [Vec<u32>],
    },
    CachedDrainSelected {
        controls: Box<NormalControlHeldSourceView>,
    },
    CachedDrainAdmInput {
        controls: Box<NormalControlAdmHeldSourceView>,
    },
}

struct MainSpatialCallbackWork<'a> {
    prepared: &'a PreparedJobInput,
    callback: MainF01eCallback<'a>,
    spatial_control: &'a SpatialControlMachine,
    spatial_audio: &'a SpatialAudioMachine,
    p11_geometry: &'a SelectedP11Geometry,
    p15_geometry: &'a SelectedP15Geometry,
    diffusion_parameters: &'a DiffusionFilterParameters,
    feature_processor: &'a ValidatedFeatureProcessorMathProvider,
    feature_fft: &'a ValidatedFeatureFft4096MathProvider,
    window: &'a ValidatedWindowQ15Provider,
    component_parameters: &'a ValidatedComponentParameterProvider,
    spatial_audio_providers: Option<&'a SelectedSpatialAudioProviderBundle>,
}

#[derive(Debug)]
pub enum MainSpatialCallbackError {
    UnsupportedProfile,
    ReceiverProvidersRequired,
    WorkerUnavailable,
    WorkerPanicked,
    PrimaryControlsUnavailable,
    SourceMap(Installed48SourceMapError),
    SourceAxis(SourceAxisError),
    SourceFlags(SourceFlagsError),
    Installed48(SpatialControlInstalled48Error),
    Frontend(FrontendFeatureStageError),
    SourceGeometry(GlobalSourceGeometryError),
    Callback(SpatialProductionCallbackError),
}

fn source_axis_binding(
    source_map: &Installed48SourceMap,
    owner_generation: SourceAxisOwnerGeneration,
    transaction_id: CallbackLineageIdentity,
    source_origin_generation: FrontendCallbackIndex,
) -> Result<Arc<SourceAxisBinding>, MainSpatialCallbackError> {
    let source_ids = source_map
        .rows()
        .iter()
        .map(|row| row.source_index())
        .collect::<Vec<_>>();
    let axis = SourceAxisBinding::new(
        source_ids,
        owner_generation,
        *source_map.source_binding().source_receipt_sha256(),
        transaction_id,
        source_origin_generation,
    )
    .map_err(MainSpatialCallbackError::SourceAxis)?;
    Ok(Arc::new(axis))
}

/// Noncommitting MAIN callback proposal bound to the exact admitted input identity capability.
#[must_use]
pub struct MainSpatialCallbackCandidate {
    prepared_identity: Arc<PreparedJobInputIdentity>,
    proposal: SpatialProductionCallbackProposal,
}

/// The validated F01E/F03W/F04 after-roots for one callback. This is a partial owner proposal;
/// ReaderEof and the remaining programme owners still belong to the enclosing parent.
#[must_use]
pub struct MainSpatialCallbackOwnerProposal {
    next_roots: MainSpatialColdRoots,
    transaction_id: CallbackTransactionId,
    callback_index: FrontendCallbackIndex,
    receipts: [GtcOwnerReceipt; 3],
}

impl MainSpatialCallbackOwnerProposal {
    pub const fn next_roots(&self) -> &MainSpatialColdRoots {
        &self.next_roots
    }

    pub const fn transaction_id(&self) -> &CallbackTransactionId {
        &self.transaction_id
    }

    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn receipts(&self) -> &[GtcOwnerReceipt; 3] {
        &self.receipts
    }

    pub fn into_parts(
        self,
    ) -> (
        MainSpatialColdRoots,
        CallbackTransactionId,
        FrontendCallbackIndex,
        [GtcOwnerReceipt; 3],
    ) {
        (
            self.next_roots,
            self.transaction_id,
            self.callback_index,
            self.receipts,
        )
    }
}

/// Parent-facing F03W/F04 values paired with their admitted input identity capability.
#[must_use]
pub struct MainSpatialCallbackParentParts {
    prepared_identity: Arc<PreparedJobInputIdentity>,
    spatial: SpatialProductionCallbackParentParts,
}

impl MainSpatialCallbackParentParts {
    pub const fn spatial(&self) -> &SpatialProductionCallbackParentParts {
        &self.spatial
    }

    pub const fn prepared_identity(&self) -> &Arc<PreparedJobInputIdentity> {
        &self.prepared_identity
    }

    pub fn into_parts(
        self,
    ) -> (
        Arc<PreparedJobInputIdentity>,
        SpatialProductionCallbackParentParts,
    ) {
        (self.prepared_identity, self.spatial)
    }
}

impl MainSpatialCallbackCandidate {
    pub const fn proposal(&self) -> &SpatialProductionCallbackProposal {
        &self.proposal
    }

    pub fn into_parent_parts(self) -> MainSpatialCallbackParentParts {
        MainSpatialCallbackParentParts {
            prepared_identity: self.prepared_identity,
            spatial: self.proposal.into_parent_parts(),
        }
    }

    pub(crate) fn is_bound_to(&self, prepared: &PreparedJobInput) -> bool {
        Arc::ptr_eq(
            &self.prepared_identity,
            &prepared.domain_identity_capability(),
        )
    }
}

impl MainSpatialColdRoots {
    /// Validates a cached MAIN callback while retaining the committed F01E last-normal root.
    ///
    /// Cached delivery advances the spatial owners but does not produce a new F01E snapshot. The
    /// frontend receipt therefore records an unchanged transition at the current delivery clock.
    pub fn prepare_spatial_after_cached_callback(
        &self,
        prepared: &PreparedJobInput,
        spatial_callback: MainSpatialCallbackCandidate,
        transaction_id: CallbackTransactionId,
    ) -> Result<MainSpatialCallbackOwnerProposal, MainSpatialCallbackParentError> {
        if !spatial_callback.is_bound_to(prepared) {
            return Err(MainSpatialCallbackParentError::PreparedInputMismatch);
        }

        let frontend_callback_index = self.frontend_control.committed().callback_index();
        let source_transaction = spatial_callback.transaction_id();
        let delivery_callback = spatial_callback.callback_index();
        let expected_normal_callbacks = prepared
            .configuration()
            .source_samples()
            .div_ceil(MAIN_NORMAL_CALLBACK_SAMPLES);
        if self.frontend_control.committed().programme_id() != prepared.programme_id()
            || source_transaction.programme_id() != prepared.programme_id()
            || source_transaction.attempt_id() != self.frontend_control.committed().attempt_id()
        {
            return Err(MainSpatialCallbackParentError::CallbackIndexMismatch);
        }
        let callback_index = validate_cached_delivery_clock(
            frontend_callback_index,
            delivery_callback,
            source_transaction.callback_index(),
            self.spatial_control.committed().callback_index(),
            self.spatial_audio.committed().callback_index(),
            expected_normal_callbacks,
        )?;

        let (prepared_identity, spatial_parts) = spatial_callback.into_parent_parts().into_parts();
        if !Arc::ptr_eq(&prepared_identity, &prepared.domain_identity_capability()) {
            return Err(MainSpatialCallbackParentError::PreparedInputMismatch);
        }
        let spatial_control_staged = self
            .spatial_control
            .stage(
                self.spatial_control
                    .prepare(spatial_parts.spatial_control_after)
                    .map_err(MainSpatialCallbackParentError::SpatialControl)?,
            )
            .map_err(MainSpatialCallbackParentError::SpatialControl)?;
        let spatial_control = self
            .spatial_control
            .replacement_candidate(spatial_control_staged, transaction_id.clone())
            .map_err(MainSpatialCallbackParentError::SpatialControl)?;
        let spatial_audio_staged = self
            .spatial_audio
            .stage(
                self.spatial_audio
                    .prepare(spatial_parts.spatial_audio_after)
                    .map_err(MainSpatialCallbackParentError::SpatialAudio)?,
            )
            .map_err(MainSpatialCallbackParentError::SpatialAudio)?;
        let spatial_audio = self
            .spatial_audio
            .replacement_candidate(spatial_audio_staged, transaction_id.clone())
            .map_err(MainSpatialCallbackParentError::SpatialAudio)?;

        let frontend_reference = self.frontend_control.state_reference();
        let frontend_snapshot = frontend_reference
            .gtc_snapshot()
            .map_err(|_| MainSpatialCallbackParentError::ReceiptMismatch)?;
        let frontend_receipt = GtcOwnerReceipt::from_transition(
            frontend_snapshot,
            frontend_snapshot,
            transaction_id.clone(),
        )
        .map_err(|_| MainSpatialCallbackParentError::ReceiptMismatch)?;

        let (spatial_control, spatial_control_receipt, _) = spatial_control.into_parts();
        let (spatial_audio, spatial_audio_receipt) = spatial_audio.into_parts();
        let expected_callback = FrontendCallbackIndex::new(
            callback_index
                .value()
                .checked_add(1)
                .ok_or(MainSpatialCallbackParentError::CallbackIndexOverflow)?,
        );
        let receipts = [
            frontend_receipt,
            spatial_control_receipt,
            spatial_audio_receipt,
        ];
        let owners = [
            GtcOwnerId::FrontendControl,
            GtcOwnerId::SpatialControl,
            GtcOwnerId::SpatialAudio,
        ];
        let before_generations = [
            self.frontend_control.generation(),
            self.spatial_control.generation(),
            self.spatial_audio.generation(),
        ];
        let after_generations = [
            self.frontend_control.generation(),
            spatial_control.generation(),
            spatial_audio.generation(),
        ];
        if receipts[0].owner_id() != owners[0]
            || receipts[0].transaction_id() != &transaction_id
            || receipts[0].before_generation() != before_generations[0]
            || receipts[0].after_generation() != before_generations[0]
            || receipts[0].before_sha256() != receipts[0].after_sha256()
            || receipts[0].changed()
            || receipts[0].commit_count() != 0
            || receipts[1..]
                .iter()
                .zip(owners[1..].iter())
                .zip(before_generations[1..].iter())
                .zip(after_generations[1..].iter())
                .any(|(((receipt, owner), before), after)| {
                    receipt.owner_id() != *owner
                        || receipt.transaction_id() != &transaction_id
                        || receipt.before_generation() != *before
                        || receipt.after_generation() != *after
                        || !receipt.changed()
                        || receipt.commit_count() != 1
                        || before.checked_add(1) != Some(*after)
                })
            || spatial_control.committed().callback_index() != expected_callback
            || spatial_audio.committed().callback_index() != expected_callback
        {
            return Err(MainSpatialCallbackParentError::ReceiptMismatch);
        }

        Ok(MainSpatialCallbackOwnerProposal {
            next_roots: MainSpatialColdRoots {
                frontend_control: Arc::clone(&self.frontend_control),
                spatial_control,
                spatial_audio,
                normal_control_prepared: Arc::clone(&self.normal_control_prepared),
            },
            transaction_id,
            callback_index,
            receipts,
        })
    }

    /// Validates the three MAIN spatial owner transitions and returns their shared after-value.
    ///
    /// This proposal is intentionally incomplete for programme commit: the caller must join its
    /// receipts and roots with ReaderEof, Analysis, DRC, JOC, Metadata, Integrity, Codec, Meter,
    /// and Output before installing a callback root set.
    pub fn prepare_spatial_after_callback(
        &self,
        prepared: &PreparedJobInput,
        frontend: FrontendControlReplacementCandidate,
        spatial_callback: MainSpatialCallbackCandidate,
        transaction_id: CallbackTransactionId,
    ) -> Result<MainSpatialCallbackOwnerProposal, MainSpatialCallbackParentError> {
        if !spatial_callback.is_bound_to(prepared) {
            return Err(MainSpatialCallbackParentError::PreparedInputMismatch);
        }
        let callback_index = self.frontend_control.committed().callback_index();
        if spatial_callback.callback_index() != callback_index
            || frontend.next_machine().committed().callback_index().value()
                != callback_index
                    .value()
                    .checked_add(1)
                    .ok_or(MainSpatialCallbackParentError::CallbackIndexOverflow)?
        {
            return Err(MainSpatialCallbackParentError::CallbackIndexMismatch);
        }

        let (prepared_identity, spatial_parts) = spatial_callback.into_parent_parts().into_parts();
        if !Arc::ptr_eq(&prepared_identity, &prepared.domain_identity_capability()) {
            return Err(MainSpatialCallbackParentError::PreparedInputMismatch);
        }
        let spatial_control_staged = self
            .spatial_control
            .stage(
                self.spatial_control
                    .prepare(spatial_parts.spatial_control_after)
                    .map_err(MainSpatialCallbackParentError::SpatialControl)?,
            )
            .map_err(MainSpatialCallbackParentError::SpatialControl)?;
        let spatial_control = self
            .spatial_control
            .replacement_candidate(spatial_control_staged, transaction_id.clone())
            .map_err(MainSpatialCallbackParentError::SpatialControl)?;
        let spatial_audio_staged = self
            .spatial_audio
            .stage(
                self.spatial_audio
                    .prepare(spatial_parts.spatial_audio_after)
                    .map_err(MainSpatialCallbackParentError::SpatialAudio)?,
            )
            .map_err(MainSpatialCallbackParentError::SpatialAudio)?;
        let spatial_audio = self
            .spatial_audio
            .replacement_candidate(spatial_audio_staged, transaction_id.clone())
            .map_err(MainSpatialCallbackParentError::SpatialAudio)?;

        let (frontend_control, frontend_receipt) = frontend.into_parts();
        let (spatial_control, spatial_control_receipt, _) = spatial_control.into_parts();
        let (spatial_audio, spatial_audio_receipt) = spatial_audio.into_parts();
        let expected_callback = FrontendCallbackIndex::new(
            callback_index
                .value()
                .checked_add(1)
                .ok_or(MainSpatialCallbackParentError::CallbackIndexOverflow)?,
        );
        let receipts = [
            frontend_receipt,
            spatial_control_receipt,
            spatial_audio_receipt,
        ];
        let owners = [
            GtcOwnerId::FrontendControl,
            GtcOwnerId::SpatialControl,
            GtcOwnerId::SpatialAudio,
        ];
        let before_generations = [
            self.frontend_control.generation(),
            self.spatial_control.generation(),
            self.spatial_audio.generation(),
        ];
        let after_generations = [
            frontend_control.generation(),
            spatial_control.generation(),
            spatial_audio.generation(),
        ];
        if receipts
            .iter()
            .zip(owners)
            .zip(before_generations)
            .zip(after_generations)
            .any(|(((receipt, owner), before), after)| {
                receipt.owner_id() != owner
                    || receipt.transaction_id() != &transaction_id
                    || receipt.before_generation() != before
                    || receipt.after_generation() != after
                    || !receipt.changed()
                    || receipt.commit_count() != 1
                    || before.checked_add(1) != Some(after)
            })
            || frontend_control.committed().callback_index() != expected_callback
            || spatial_control.committed().callback_index() != expected_callback
            || spatial_audio.committed().callback_index() != expected_callback
        {
            return Err(MainSpatialCallbackParentError::ReceiptMismatch);
        }

        Ok(MainSpatialCallbackOwnerProposal {
            next_roots: MainSpatialColdRoots {
                frontend_control: Arc::new(frontend_control),
                spatial_control,
                spatial_audio,
                normal_control_prepared: Arc::clone(&self.normal_control_prepared),
            },
            transaction_id,
            callback_index,
            receipts,
        })
    }
}

fn validate_cached_delivery_clock(
    frontend_callback_index: FrontendCallbackIndex,
    delivery_callback: FrontendCallbackIndex,
    source_lineage_callback_index: FrontendCallbackIndex,
    spatial_control_callback_index: FrontendCallbackIndex,
    spatial_audio_callback_index: FrontendCallbackIndex,
    normal_callbacks: u64,
) -> Result<FrontendCallbackIndex, MainSpatialCallbackParentError> {
    let expected_frontend_callback_index = FrontendCallbackIndex::new(
        u32::try_from(normal_callbacks)
            .map_err(|_| MainSpatialCallbackParentError::CallbackIndexOverflow)?,
    );
    let delivery_index = u64::from(delivery_callback.value());
    let expected_drain_end = normal_callbacks
        .checked_add(4)
        .ok_or(MainSpatialCallbackParentError::CallbackIndexOverflow)?;
    if normal_callbacks == 0
        || frontend_callback_index != expected_frontend_callback_index
        || source_lineage_callback_index != delivery_callback
        || spatial_control_callback_index != delivery_callback
        || spatial_audio_callback_index != delivery_callback
        || delivery_index < normal_callbacks
        || delivery_index >= expected_drain_end
    {
        return Err(MainSpatialCallbackParentError::CallbackIndexMismatch);
    }
    Ok(delivery_callback)
}

#[derive(Debug)]
pub enum MainSpatialCallbackParentError {
    PreparedInputMismatch,
    CallbackIndexMismatch,
    CallbackIndexOverflow,
    ReceiptMismatch,
    SpatialControl(jocforge_domain::spatial::SpatialControlError),
    SpatialAudio(jocforge_domain::spatial::SpatialAudioError),
}

#[cfg(test)]
mod cached_delivery_clock_tests {
    use super::{MainSpatialCallbackParentError, validate_cached_delivery_clock};
    use jocforge_domain::timeline::FrontendCallbackIndex;

    #[test]
    fn cached_drains_keep_frontend_at_normal_boundary_while_delivery_advances() {
        let normal_callbacks = 4_247_u64;
        let frontend_index = FrontendCallbackIndex::new(4_247);
        for value in 4_247..4_251 {
            let delivery = FrontendCallbackIndex::new(value);
            assert!(matches!(
                validate_cached_delivery_clock(
                    frontend_index,
                    delivery,
                    delivery,
                    delivery,
                    delivery,
                    normal_callbacks,
                ),
                Ok(index) if index == delivery
            ));
        }
    }

    #[test]
    fn cached_delivery_clock_rejects_stale_frontend_source_or_spatial_indexes() {
        let normal_callbacks = 4_247_u64;
        let frontend_index = FrontendCallbackIndex::new(4_247);
        let delivery = FrontendCallbackIndex::new(4_248);
        assert!(matches!(
            validate_cached_delivery_clock(
                FrontendCallbackIndex::new(4_246),
                delivery,
                delivery,
                delivery,
                delivery,
                normal_callbacks,
            ),
            Err(MainSpatialCallbackParentError::CallbackIndexMismatch)
        ));
        assert!(matches!(
            validate_cached_delivery_clock(
                frontend_index,
                delivery,
                FrontendCallbackIndex::new(4_247),
                delivery,
                delivery,
                normal_callbacks,
            ),
            Err(MainSpatialCallbackParentError::CallbackIndexMismatch)
        ));
        assert!(matches!(
            validate_cached_delivery_clock(
                frontend_index,
                FrontendCallbackIndex::new(4_251),
                FrontendCallbackIndex::new(4_251),
                FrontendCallbackIndex::new(4_251),
                FrontendCallbackIndex::new(4_251),
                normal_callbacks,
            ),
            Err(MainSpatialCallbackParentError::CallbackIndexMismatch)
        ));
    }
}

impl Deref for MainSpatialCallbackCandidate {
    type Target = SpatialProductionCallbackProposal;

    fn deref(&self) -> &Self::Target {
        &self.proposal
    }
}

/// Stages one real MAIN callback through F03W, F03F, selector/outer, component and receiver.
///
/// The returned proposal retains both owner after-values for the enclosing programme transaction.
/// This function neither installs an owner root nor authorizes output publication.
pub fn stage_main_spatial_callback(
    input: MainSpatialCallbackInput<'_>,
) -> Result<MainSpatialCallbackCandidate, MainSpatialCallbackError> {
    let prepared_identity = input.prepared.domain_identity_capability();
    let MainSpatialCallbackInput {
        prepared,
        controls,
        physical_pcm_rows,
        spatial_control,
        spatial_audio,
        p11_geometry,
        p15_geometry,
        diffusion_parameters,
        feature_processor,
        feature_fft,
        window,
        component_parameters,
        spatial_audio_providers,
    } = input;
    let callback = match controls {
        MainNormalControlOutput::Selected(controls) => MainF01eCallback::NormalSelected {
            controls: Box::new(controls),
            physical_pcm_rows,
        },
        MainNormalControlOutput::AdmInput(controls) => MainF01eCallback::NormalAdmInput {
            controls: Box::new(controls),
            physical_pcm_rows,
        },
    };
    let proposal = stage_main_spatial_callback_work(MainSpatialCallbackWork {
        prepared,
        callback,
        spatial_control,
        spatial_audio,
        p11_geometry,
        p15_geometry,
        diffusion_parameters,
        feature_processor,
        feature_fft,
        window,
        component_parameters,
        spatial_audio_providers,
    })?;
    Ok(MainSpatialCallbackCandidate {
        prepared_identity,
        proposal,
    })
}

/// Stages one cached MAIN callback using the exact last-normal F01E controls and SpatialControl PCM cache.
pub fn stage_main_spatial_cached_callback(
    input: MainSpatialCachedCallbackInput<'_>,
) -> Result<MainSpatialCallbackCandidate, MainSpatialCallbackError> {
    let prepared_identity = input.prepared.domain_identity_capability();
    let MainSpatialCachedCallbackInput {
        prepared,
        controls,
        spatial_control,
        spatial_audio,
        p11_geometry,
        p15_geometry,
        diffusion_parameters,
        feature_processor,
        feature_fft,
        window,
        component_parameters,
        spatial_audio_providers,
    } = input;
    let callback = match controls {
        MainHeldControlOutput::Selected(controls) => MainF01eCallback::CachedDrainSelected {
            controls: Box::new(controls),
        },
        MainHeldControlOutput::AdmInput(controls) => MainF01eCallback::CachedDrainAdmInput {
            controls: Box::new(controls),
        },
    };
    let proposal = stage_main_spatial_callback_work(MainSpatialCallbackWork {
        prepared,
        callback,
        spatial_control,
        spatial_audio,
        p11_geometry,
        p15_geometry,
        diffusion_parameters,
        feature_processor,
        feature_fft,
        window,
        component_parameters,
        spatial_audio_providers,
    })?;
    Ok(MainSpatialCallbackCandidate {
        prepared_identity,
        proposal,
    })
}

fn stage_main_spatial_callback_work(
    input: MainSpatialCallbackWork<'_>,
) -> Result<SpatialProductionCallbackProposal, MainSpatialCallbackError> {
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("jocforge-main-spatial-callback".to_owned())
            .stack_size(MAIN_SPATIAL_CALLBACK_STACK_BYTES)
            .spawn_scoped(scope, move || stage_main_spatial_callback_inner(input))
            .map_err(|_| MainSpatialCallbackError::WorkerUnavailable)?;
        worker
            .join()
            .map_err(|_| MainSpatialCallbackError::WorkerPanicked)?
    })
}

fn stage_main_spatial_callback_inner(
    input: MainSpatialCallbackWork<'_>,
) -> Result<SpatialProductionCallbackProposal, MainSpatialCallbackError> {
    let MainSpatialCallbackWork {
        prepared,
        callback,
        spatial_control,
        spatial_audio,
        p11_geometry,
        p15_geometry,
        diffusion_parameters,
        feature_processor,
        feature_fft,
        window,
        component_parameters,
        spatial_audio_providers,
    } = input;
    let profile = resolve_selected_profile(&spatial_control.profile())
        .map_err(|_| MainSpatialCallbackError::UnsupportedProfile)?;
    if profile != prepared.configuration().selected_profile() {
        return Err(MainSpatialCallbackError::UnsupportedProfile);
    }
    let primary_controls = prepared
        .primary_metadata_controls()
        .ok_or(MainSpatialCallbackError::PrimaryControlsUnavailable)?;
    let source_map = build_selected_installed48_source_map(primary_controls, p11_geometry)
        .map_err(MainSpatialCallbackError::SourceMap)?;
    let (transaction_id, source_control_clock, source_geometry, source_snapshot, frontend) =
        match callback {
            MainF01eCallback::NormalSelected {
                controls,
                physical_pcm_rows,
            } => {
                let source_projection = project_normal_control_snapshot_output_source_controls(
                    &controls,
                    prepared.graph().graph(),
                    profile,
                )
                .map_err(MainSpatialCallbackError::SourceFlags)?;
                let source_control_clock = source_projection.clock();
                let source_geometry = build_global_source_geometry_snapshot_from_output(
                    prepared.graph().graph(),
                    &source_map,
                    &controls,
                )
                .map_err(MainSpatialCallbackError::SourceGeometry)?;
                let source_axis = source_axis_binding(
                    &source_map,
                    SourceAxisOwnerGeneration::new(controls.next_owner_generation()),
                    controls.transaction_id().clone(),
                    controls.normal_origin_generation(),
                )?;
                let before_history =
                    BeforeRegularHistory::from_owner(spatial_control, source_projection.route())
                        .map_err(MainSpatialCallbackError::SourceFlags)?;
                let transaction_id = controls.transaction_id().clone();
                let parent = SourceFlagsParentLineage::new(transaction_id.clone(), spatial_control)
                    .map_err(MainSpatialCallbackError::SourceFlags)?;
                let installed48 = stage_spatial_control_installed48(
                    spatial_control,
                    SpatialControlInstalled48Input {
                        callback: SpatialControlInstalled48Callback::Normal {
                            controls: &controls,
                            physical_pcm_rows,
                        },
                        source_map: &source_map,
                        diffusion_parameters: Some(diffusion_parameters),
                    },
                )
                .map_err(MainSpatialCallbackError::Installed48)?;
                let prefix_writers = PrefixWriterSet::new(installed48.prefix_writer_indices())
                    .map_err(MainSpatialCallbackError::SourceFlags)?;
                let current_pcm = physical_pcm_rows
                    .iter()
                    .map(|row| row.iter().copied().map(f32::from_bits).collect())
                    .collect();
                let source_flags = SourceFlagsInput::from_primary_regular_source_controls(
                    CurrentHostPcm::new(current_pcm)
                        .map_err(MainSpatialCallbackError::SourceFlags)?,
                    Some(before_history),
                    source_projection,
                    prefix_writers,
                    parent,
                )
                .map_err(MainSpatialCallbackError::SourceFlags)?
                .with_source_axis(Arc::clone(&source_axis));
                let frontend = stage_spatial_control_frontend_features_from_installed48(
                    spatial_control,
                    source_flags,
                    installed48,
                    feature_processor,
                    feature_fft,
                )
                .map_err(MainSpatialCallbackError::Frontend)?;
                (
                    transaction_id,
                    source_control_clock,
                    source_geometry,
                    NormalControlCallbackSource::NormalOutput(controls),
                    frontend,
                )
            }
            MainF01eCallback::NormalAdmInput {
                controls,
                physical_pcm_rows,
            } => {
                let source_projection = project_normal_control_adm_input_output_source_controls(
                    &controls,
                    prepared.graph().graph(),
                    profile,
                )
                .map_err(MainSpatialCallbackError::SourceFlags)?;
                let source_control_clock = source_projection.clock();
                let source_geometry = build_global_source_geometry_snapshot_from_adm_input_output(
                    prepared.graph().graph(),
                    &source_map,
                    &controls,
                )
                .map_err(MainSpatialCallbackError::SourceGeometry)?;
                let source_axis = source_axis_binding(
                    &source_map,
                    SourceAxisOwnerGeneration::new(controls.next_owner_generation()),
                    controls.transaction_id().clone(),
                    controls.normal_origin_generation(),
                )?;
                let before_history =
                    BeforeRegularHistory::from_owner(spatial_control, source_projection.route())
                        .map_err(MainSpatialCallbackError::SourceFlags)?;
                let transaction_id = controls.transaction_id().clone();
                let parent = SourceFlagsParentLineage::new(transaction_id.clone(), spatial_control)
                    .map_err(MainSpatialCallbackError::SourceFlags)?;
                let installed48 = stage_spatial_control_installed48(
                    spatial_control,
                    SpatialControlInstalled48Input {
                        callback: SpatialControlInstalled48Callback::NormalAdmInputOutput {
                            controls: &controls,
                            physical_pcm_rows,
                        },
                        source_map: &source_map,
                        diffusion_parameters: Some(diffusion_parameters),
                    },
                )
                .map_err(MainSpatialCallbackError::Installed48)?;
                let prefix_writers = PrefixWriterSet::new(installed48.prefix_writer_indices())
                    .map_err(MainSpatialCallbackError::SourceFlags)?;
                let current_pcm = physical_pcm_rows
                    .iter()
                    .map(|row| row.iter().copied().map(f32::from_bits).collect())
                    .collect();
                let source_flags = SourceFlagsInput::from_primary_regular_source_controls(
                    CurrentHostPcm::new(current_pcm)
                        .map_err(MainSpatialCallbackError::SourceFlags)?,
                    Some(before_history),
                    source_projection,
                    prefix_writers,
                    parent,
                )
                .map_err(MainSpatialCallbackError::SourceFlags)?
                .with_source_axis(Arc::clone(&source_axis));
                let frontend = stage_spatial_control_frontend_features_from_installed48(
                    spatial_control,
                    source_flags,
                    installed48,
                    feature_processor,
                    feature_fft,
                )
                .map_err(MainSpatialCallbackError::Frontend)?;
                (
                    transaction_id,
                    source_control_clock,
                    source_geometry,
                    NormalControlCallbackSource::NormalAdmInputOutput(controls),
                    frontend,
                )
            }
            MainF01eCallback::CachedDrainAdmInput { controls } => {
                let source_projection = project_held_normal_control_adm_input_source_controls(
                    &controls,
                    prepared.graph().graph(),
                    profile,
                )
                .map_err(MainSpatialCallbackError::SourceFlags)?;
                let source_control_clock = source_projection.clock();
                let source_geometry = build_held_global_source_geometry_snapshot_from_adm_input(
                    prepared.graph().graph(),
                    &source_map,
                    &controls,
                )
                .map_err(MainSpatialCallbackError::SourceGeometry)?;
                let source_axis = source_axis_binding(
                    &source_map,
                    SourceAxisOwnerGeneration::new(controls.owner_generation()),
                    controls.delivery_transaction_id().clone(),
                    controls.source_origin_generation(),
                )?;
                let before_history =
                    BeforeRegularHistory::from_owner(spatial_control, source_projection.route())
                        .map_err(MainSpatialCallbackError::SourceFlags)?;
                let transaction_id = controls.delivery_transaction_id().clone();
                let parent = SourceFlagsParentLineage::new(transaction_id.clone(), spatial_control)
                    .map_err(MainSpatialCallbackError::SourceFlags)?;
                let installed48 = stage_spatial_control_installed48(
                    spatial_control,
                    SpatialControlInstalled48Input {
                        callback: SpatialControlInstalled48Callback::CachedDrainAdmInput {
                            controls: &controls,
                        },
                        source_map: &source_map,
                        diffusion_parameters: Some(diffusion_parameters),
                    },
                )
                .map_err(MainSpatialCallbackError::Installed48)?;
                let prefix_writers = PrefixWriterSet::new(installed48.prefix_writer_indices())
                    .map_err(MainSpatialCallbackError::SourceFlags)?;
                let current_pcm = installed48
                    .source_pcm_rows()
                    .iter()
                    .map(|row| row.iter().copied().map(f32::from_bits).collect())
                    .collect();
                let source_flags = SourceFlagsInput::from_primary_regular_source_controls(
                    CurrentHostPcm::new(current_pcm)
                        .map_err(MainSpatialCallbackError::SourceFlags)?,
                    Some(before_history),
                    source_projection,
                    prefix_writers,
                    parent,
                )
                .map_err(MainSpatialCallbackError::SourceFlags)?
                .with_source_axis(Arc::clone(&source_axis));
                let frontend = stage_spatial_control_frontend_features_from_installed48(
                    spatial_control,
                    source_flags,
                    installed48,
                    feature_processor,
                    feature_fft,
                )
                .map_err(MainSpatialCallbackError::Frontend)?;
                (
                    transaction_id,
                    source_control_clock,
                    source_geometry,
                    NormalControlCallbackSource::CachedAdmHold(controls),
                    frontend,
                )
            }
            MainF01eCallback::CachedDrainSelected { controls } => {
                let source_projection = project_held_normal_control_snapshot_source_controls(
                    &controls,
                    prepared.graph().graph(),
                    profile,
                )
                .map_err(MainSpatialCallbackError::SourceFlags)?;
                let source_control_clock = source_projection.clock();
                let source_geometry = build_held_global_source_geometry_snapshot(
                    prepared.graph().graph(),
                    &source_map,
                    &controls,
                )
                .map_err(MainSpatialCallbackError::SourceGeometry)?;
                let source_axis = source_axis_binding(
                    &source_map,
                    SourceAxisOwnerGeneration::new(controls.owner_generation()),
                    controls.delivery_transaction_id().clone(),
                    controls.source_origin_generation(),
                )?;
                let before_history =
                    BeforeRegularHistory::from_owner(spatial_control, source_projection.route())
                        .map_err(MainSpatialCallbackError::SourceFlags)?;
                let transaction_id = controls.delivery_transaction_id().clone();
                let parent = SourceFlagsParentLineage::new(transaction_id.clone(), spatial_control)
                    .map_err(MainSpatialCallbackError::SourceFlags)?;
                let installed48 = stage_spatial_control_installed48(
                    spatial_control,
                    SpatialControlInstalled48Input {
                        callback: SpatialControlInstalled48Callback::CachedDrain {
                            controls: &controls,
                        },
                        source_map: &source_map,
                        diffusion_parameters: Some(diffusion_parameters),
                    },
                )
                .map_err(MainSpatialCallbackError::Installed48)?;
                let prefix_writers = PrefixWriterSet::new(installed48.prefix_writer_indices())
                    .map_err(MainSpatialCallbackError::SourceFlags)?;
                let current_pcm = installed48
                    .source_pcm_rows()
                    .iter()
                    .map(|row| row.iter().copied().map(f32::from_bits).collect())
                    .collect();
                let source_flags = SourceFlagsInput::from_primary_regular_source_controls(
                    CurrentHostPcm::new(current_pcm)
                        .map_err(MainSpatialCallbackError::SourceFlags)?,
                    Some(before_history),
                    source_projection,
                    prefix_writers,
                    parent,
                )
                .map_err(MainSpatialCallbackError::SourceFlags)?
                .with_source_axis(Arc::clone(&source_axis));
                let frontend = stage_spatial_control_frontend_features_from_installed48(
                    spatial_control,
                    source_flags,
                    installed48,
                    feature_processor,
                    feature_fft,
                )
                .map_err(MainSpatialCallbackError::Frontend)?;
                (
                    transaction_id,
                    source_control_clock,
                    source_geometry,
                    NormalControlCallbackSource::CachedHold(controls),
                    frontend,
                )
            }
        };

    let spatial_audio_joc_providers = if transaction_id.callback_index().value() >= 2 {
        Some(
            spatial_audio_providers
                .ok_or(MainSpatialCallbackError::ReceiverProvidersRequired)?
                .spatial_audio_joc_providers(),
        )
    } else {
        spatial_audio_providers.map(SelectedSpatialAudioProviderBundle::spatial_audio_joc_providers)
    };

    continue_spatial_production_callback(SpatialProductionCallbackInputs {
        frontend,
        source_snapshot,
        source_geometry,
        source_control_clock,
        source_map: &source_map,
        p15_geometry,
        spatial_control,
        spatial_audio,
        spatial_audio_joc_providers,
        window,
        parameters: component_parameters,
    })
    .map_err(MainSpatialCallbackError::Callback)
}
