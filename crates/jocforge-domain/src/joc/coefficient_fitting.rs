// pattern: Functional Core
//! Selected 15-object by 12-band coefficient fit proposal composition.

use crate::codec::owner::Flat7xD0SourceGenerationReceipt;
use crate::{
    joc::{
        coefficient_geometry::{
            CoefficientGeometryError, CoefficientGeometryProposal, CoefficientGeometryState,
            stage_geometry_from_qmf,
        },
        coefficient_normal_equations::{
            CoefficientNormalEquation, CoefficientNormalEquationError,
            build_normal_equations_for_carriers, build_selected_normal_equations,
        },
        coefficient_payload::{
            CoefficientPayloadInput, CoefficientPayloadObject, CoefficientPayloadPoint,
            Flat7xCoefficientRows,
        },
        coefficient_representation::{
            FitRepresentationInput, JocCoefficientRowLayout, RepresentationRetryError,
            RepresentationRetryProvider, RepresentationWriteResult,
            write_fit_representation_with_resolved_profile, write_fit_representation_with_retry,
        },
        coefficient_search::{
            CoefficientFitResult, CoefficientSearchError, search_coefficient_cell_for_carriers,
            search_selected_coefficient_cell,
        },
        fit_score::FitScoreProvider,
        seven_carrier_windows::{
            SevenCarrierQmfSourceIdentity, SevenCarrierQmfState, SevenCarrierWindowSet,
        },
        temporal_weights::{TemporalWeightError, TemporalWeightProposal, TemporalWeightState},
    },
    profile::{JocPayloadDownmixIndex, ResolvedEncodingProfile},
    spatial::{
        QmfReceiverFunctionalProvider,
        source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
        spatial_audio_state::SelectedJocQmfProjection,
    },
    timeline::{CodecSourceIndex, JocDispatchIndex},
    transaction::{CallbackLineageIdentity, CallbackTransactionId},
};

#[cfg(test)]
use crate::joc::coefficient_payload::{
    CoefficientPayloadWriterFailure, write_selected_id14_coefficient_payload,
};
#[cfg(test)]
use crate::joc::emdf::metadata_owner::MetadataPayloadBody;

const OBJECT_COUNT: usize = 15;
const CHANNEL_COUNT: usize = 5;
const BAND_COUNT: usize = 12;
const SEVEN_CHANNEL_COUNT: usize = 7;
#[cfg(test)]
const SOURCE_ROWS: usize = 16;
#[cfg(test)]
const LANE_COUNT: usize = 6;
const ZERO_BITS: u32 = 0x0000_0000;

#[cfg(test)]
use crate::joc::coefficient_geometry::{GeometryAccumulatorRow, stage_selected_geometry};
#[cfg(test)]
use crate::{timeline::FrontendCallbackIndex, transaction::AttemptId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoefficientFitState {
    pub(super) object_count: crate::joc::JocObjectCount,
    temporal: TemporalWeightState,
    geometry: CoefficientGeometryState,
    fit_generation: u64,
    seven_carrier_qmf: Option<SevenCarrierQmfState>,
}

impl CoefficientFitState {
    pub const fn cold() -> Self {
        Self {
            temporal: TemporalWeightState::cold(),
            geometry: CoefficientGeometryState::cold(),
            object_count: crate::joc::JocObjectCount::Fifteen,
            fit_generation: 0,
            seven_carrier_qmf: None,
        }
    }

    pub const fn temporal(&self) -> &TemporalWeightState {
        &self.temporal
    }

    pub const fn geometry(&self) -> &CoefficientGeometryState {
        &self.geometry
    }

    pub const fn fit_generation(&self) -> u64 {
        self.fit_generation
    }

    pub const fn seven_carrier_qmf_state(&self) -> Option<&SevenCarrierQmfState> {
        self.seven_carrier_qmf.as_ref()
    }

    /// Advances the source clock for external Flat7x rows while preserving the existing
    /// temporal and geometry values. No coefficient fitting or numeric update is performed.
    pub fn after_external_flat7x_rows(
        &self,
        dispatch: JocDispatchIndex,
    ) -> Result<Self, CoefficientFittingError> {
        if self.geometry.generation() != self.fit_generation
            || self.fit_generation.checked_add(1) != Some(dispatch.value())
        {
            return Err(CoefficientFittingError::Generation);
        }
        Ok(Self {
            temporal: self.temporal.clone(),
            geometry: self
                .geometry
                .with_generation_preserving_values(dispatch.value()),
            object_count: self.object_count,
            fit_generation: dispatch.value(),
            seven_carrier_qmf: self.seven_carrier_qmf.clone(),
        })
    }

    #[cfg(test)]
    pub(crate) const fn with_generation_for_test(generation: u64) -> Self {
        Self {
            temporal: TemporalWeightState::cold(),
            geometry: CoefficientGeometryState::with_generation_for_test(generation),
            object_count: crate::joc::JocObjectCount::Fifteen,
            fit_generation: generation,
            seven_carrier_qmf: None,
        }
    }
}

impl Default for CoefficientFitState {
    fn default() -> Self {
        Self::cold()
    }
}

#[derive(Debug)]
pub struct CoefficientFitProposal {
    after: CoefficientFitState,
    #[cfg(test)]
    row_states_after: [GeometryAccumulatorRow; SOURCE_ROWS],
    full_words: Vec<u32>,
    sparse_words: Vec<u32>,
    selected_channels: Vec<u16>,
    score_call_count: u32,
    representation_offset: Option<u8>,
    source_lineage: Option<FitSourceLineage>,
}

/// Diagnostic full/sparse coefficient rows from one seven-carrier QMF window set.
#[derive(Clone, Debug, PartialEq)]
pub struct SevenCarrierFitRows {
    downmix_index: JocPayloadDownmixIndex,
    qmf_source_identity: SevenCarrierQmfSourceIdentity,
    full_words: Vec<u32>,
    sparse_words: Vec<u32>,
    selected_channels: Vec<u16>,
    score_call_count: u32,
}

impl SevenCarrierFitRows {
    pub const fn qmf_source_identity(&self) -> &SevenCarrierQmfSourceIdentity {
        &self.qmf_source_identity
    }

    pub fn full_words(&self) -> &[u32] {
        &self.full_words
    }

    pub fn sparse_words(&self) -> &[u32] {
        &self.sparse_words
    }

    pub fn selected_channels(&self) -> &[u16] {
        &self.selected_channels
    }

    pub const fn score_call_count(&self) -> u32 {
        self.score_call_count
    }

    pub const fn row_layout(&self) -> JocCoefficientRowLayout {
        JocCoefficientRowLayout::SevenChannel
    }

    pub const fn payload_downmix_index(&self) -> JocPayloadDownmixIndex {
        self.downmix_index
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }

    pub fn coefficient_payload_input(
        &self,
        clip_gain_index: u8,
        sequence_count: u16,
        capacity_bytes: u16,
        representation_offset: Option<u8>,
    ) -> Result<CoefficientPayloadInput, CoefficientFittingError> {
        let mut objects = Vec::with_capacity(OBJECT_COUNT);
        for object in 0..OBJECT_COUNT {
            let channel_rows = (0..SEVEN_CHANNEL_COUNT)
                .map(|channel| {
                    (0..BAND_COUNT)
                        .map(|band| {
                            f32::from_bits(
                                self.full_words
                                    [(object * SEVEN_CHANNEL_COUNT + channel) * BAND_COUNT + band],
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let selected_channels = (0..BAND_COUNT)
                .map(|band| {
                    u8::try_from(self.selected_channels[object * BAND_COUNT + band])
                        .map_err(|_| CoefficientFittingError::Shape)
                })
                .collect::<Result<Vec<_>, _>>()?;
            objects.push(CoefficientPayloadObject {
                present: true,
                band_enum: 5,
                mode: 1,
                points: vec![CoefficientPayloadPoint {
                    channel_rows,
                    selected_channels,
                }],
                offsets: representation_offset.map(|offset| vec![offset]),
            });
        }
        Ok(CoefficientPayloadInput {
            downmix_index: self.payload_downmix_index().value(),
            extension_config: 0,
            clip_gain_index,
            sequence_count,
            time_slot_limit: 24,
            capacity_bytes,
            objects,
        })
    }
}

/// Same-source C=7 coefficient-fit proposal for the Flat7x reconstruction layout.
#[derive(Clone, Debug, PartialEq)]
pub struct SevenCarrierCoefficientFitProposal {
    after: CoefficientFitState,
    rows: SevenCarrierFitRows,
    target_lineage: FitSourceLineage,
    representation_offset: Option<u8>,
}

impl SevenCarrierCoefficientFitProposal {
    pub const fn after(&self) -> &CoefficientFitState {
        &self.after
    }

    pub const fn rows(&self) -> &SevenCarrierFitRows {
        &self.rows
    }

    pub const fn target_lineage(&self) -> &FitSourceLineage {
        &self.target_lineage
    }

    pub const fn qmf_source_identity(&self) -> &SevenCarrierQmfSourceIdentity {
        self.rows.qmf_source_identity()
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }

    pub fn write_payload_with_resolved_profile(
        &self,
        profile: ResolvedEncodingProfile,
        provider: &RepresentationRetryProvider,
        clip_gain_index: u8,
        sequence_count: u16,
        capacity_bytes: u16,
    ) -> Result<RepresentationWriteResult, RepresentationRetryError> {
        write_fit_representation_with_resolved_profile(
            provider,
            FitRepresentationInput {
                full_words: self.rows.full_words(),
                sparse_words: self.rows.sparse_words(),
                selected_channels: self.rows.selected_channels(),
                representation_offset: self.representation_offset,
                row_layout: JocCoefficientRowLayout::SevenChannel,
            },
            clip_gain_index,
            sequence_count,
            capacity_bytes,
            profile,
        )
    }
}

/// Fits the 15 target-slot rows against a same-source seven-carrier raw-QMF window set.
///
/// The diagnostic policy begins with all seven target roles eligible, then extends the existing
/// support-growth search in role order. Geometry-based five-channel eligibility and coefficient
/// payload ownership remain separate inputs and are not inferred here.
pub fn fit_seven_carrier_window_set(
    target_windows: &[u32],
    carrier_windows: &SevenCarrierWindowSet,
    weight_words: &[u32; 48],
    score_provider: &FitScoreProvider,
) -> Result<SevenCarrierFitRows, CoefficientFittingError> {
    // Match the reference level used to fit the seven-carrier reconstruction matrix.
    let mut carrier_words = carrier_windows.words_in_target_order();
    for word in &mut carrier_words {
        *word = (f32::from_bits(*word) * 0.25).to_bits();
    }
    let equations = build_normal_equations_for_carriers::<SEVEN_CHANNEL_COUNT>(
        target_windows,
        &carrier_words,
        weight_words,
    )
    .map_err(CoefficientFittingError::NormalEquation)?;
    if equations.len() != OBJECT_COUNT * BAND_COUNT {
        return Err(CoefficientFittingError::Shape);
    }

    let mut full_words = vec![ZERO_BITS; OBJECT_COUNT * SEVEN_CHANNEL_COUNT * BAND_COUNT];
    let mut sparse_words = vec![ZERO_BITS; OBJECT_COUNT * BAND_COUNT];
    let mut selected_channels = vec![0_u16; OBJECT_COUNT * BAND_COUNT];
    let mut score_call_count = 0_u32;
    let cells = fit_independent_cells(&equations, |equation| {
        search_coefficient_cell_for_carriers::<SEVEN_CHANNEL_COUNT>(
            score_provider,
            equation,
            target_windows,
            &carrier_words,
            weight_words,
            [true; SEVEN_CHANNEL_COUNT],
        )
    })?;
    for (equation, result) in equations.iter().zip(cells) {
        let object = usize::from(equation.object());
        let band = usize::from(equation.band());
        for channel in 0..SEVEN_CHANNEL_COUNT {
            full_words[(object * SEVEN_CHANNEL_COUNT + channel) * BAND_COUNT + band] =
                result.stored_words()[channel];
        }
        sparse_words[object * BAND_COUNT + band] = result.sparse_stored_word();
        selected_channels[object * BAND_COUNT + band] = u16::from(result.sparse_selected_channel());
        for stage in result.stages() {
            score_call_count = score_call_count
                .checked_add(u32::from(stage.candidate_count()))
                .ok_or(CoefficientFittingError::Shape)?;
        }
    }

    Ok(SevenCarrierFitRows {
        downmix_index: if carrier_windows.role_bindings()[5].role()
            == crate::profile::Flat7xJocCarrierRole::TopFrontLeft
        {
            JocPayloadDownmixIndex::HeightPair
        } else {
            JocPayloadDownmixIndex::SevenChannel
        },
        qmf_source_identity: carrier_windows.source_identity().clone(),
        full_words,
        sparse_words,
        selected_channels,
        score_call_count,
    })
}

/// Stages a C=7 fit from one spatial target-slot projection and its same-source ADM-QMF windows.
pub fn stage_seven_carrier_coefficient_fit_from_qmf(
    profile: ResolvedEncodingProfile,
    before: &CoefficientFitState,
    projection: &SelectedJocQmfProjection,
    carrier_windows: &SevenCarrierWindowSet,
    carrier_after_state: &SevenCarrierQmfState,
    qmf_provider: &QmfReceiverFunctionalProvider,
    score_provider: &FitScoreProvider,
) -> Result<SevenCarrierCoefficientFitProposal, CoefficientFittingError> {
    let expected_dispatch = before
        .fit_generation
        .checked_add(1)
        .ok_or(CoefficientFittingError::Generation)?;
    let source_identity = carrier_windows.source_identity();
    if before.geometry.generation() != before.fit_generation
        || projection.dispatch().value() != expected_dispatch
        || projection.source_index().dispatch().ok() != Some(projection.dispatch())
        || source_identity.callback_lineage() != projection.transaction_id()
        || source_identity.source_index() != projection.source_index()
        || !profile.joc_reconstruction_layout().uses_seven_carriers()
        || !profile.joc_payload_downmix_index().uses_seven_carriers()
    {
        return Err(CoefficientFittingError::Lineage);
    }
    let target_words = projection.target_object_window_words();
    let temporal = before
        .temporal
        .stage_target_windows48(target_words)
        .map_err(CoefficientFittingError::Temporal)?;
    let geometry = stage_geometry_from_qmf(
        &before.geometry,
        projection,
        temporal.after().weight_words(),
        qmf_provider,
    )
    .map_err(CoefficientFittingError::Geometry)?;
    let mut rows = fit_seven_carrier_window_set(
        target_words,
        carrier_windows,
        temporal.after().weight_words(),
        score_provider,
    )?;
    if rows.downmix_index == JocPayloadDownmixIndex::HeightPair
        && profile.joc_payload_downmix_index() == JocPayloadDownmixIndex::HeightPairWithPhase
    {
        rows.downmix_index = JocPayloadDownmixIndex::HeightPairWithPhase;
    }
    let fit_generation = geometry.after().generation();
    let representation_offset = temporal
        .before_segment()
        .changed()
        .then_some(temporal.before_segment().start());
    Ok(SevenCarrierCoefficientFitProposal {
        after: CoefficientFitState {
            temporal: temporal.into_after(),
            geometry: geometry.into_after(),
            fit_generation,
            object_count: before.object_count,
            seven_carrier_qmf: Some(carrier_after_state.clone()),
        },
        rows,
        target_lineage: FitSourceLineage::from_projection(projection),
        representation_offset,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FitSourceLineage {
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: CallbackLineageIdentity,
    source_index: CodecSourceIndex,
    dispatch: crate::timeline::JocDispatchIndex,
    spatial_before_generation: u64,
    spatial_before_digest: crate::spatial::SpatialAudioDigest,
}

impl FitSourceLineage {
    /// Derives the JOC source lineage from an already accepted QMF projection.
    pub fn from_projection(projection: &SelectedJocQmfProjection) -> Self {
        Self {
            programme_id: projection.programme_id().clone(),
            attempt: projection.attempt(),
            transaction_id: projection.transaction_id().clone(),
            source_index: projection.source_index(),
            dispatch: projection.dispatch(),
            spatial_before_generation: projection.spatial_before_generation(),
            spatial_before_digest: projection.spatial_before_digest(),
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

    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn dispatch(&self) -> crate::timeline::JocDispatchIndex {
        self.dispatch
    }

    pub const fn spatial_before_generation(&self) -> u64 {
        self.spatial_before_generation
    }

    pub const fn spatial_before_digest(&self) -> crate::spatial::SpatialAudioDigest {
        self.spatial_before_digest
    }

    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn for_test(
        programme_id: SourceFlagsProgrammeId,
        attempt: SourceFlagsAttempt,
        transaction_id: CallbackLineageIdentity,
        source_index: CodecSourceIndex,
        dispatch: crate::timeline::JocDispatchIndex,
        spatial_before_generation: u64,
        spatial_before_digest: crate::spatial::SpatialAudioDigest,
    ) -> Self {
        Self {
            programme_id,
            attempt,
            transaction_id,
            source_index,
            dispatch,
            spatial_before_generation,
            spatial_before_digest,
        }
    }
}

/// Validated row values returned by an authoritative Flat7x row provider.
#[derive(Clone, Debug)]
pub struct Flat7xCoefficientRowsProviderOutput<'a> {
    rows: Flat7xCoefficientRows<'a>,
    source_receipt: Flat7xCoefficientRowsSourceReceipt,
}

/// Provider-owned source lineage for one Flat7x coefficient-row output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xCoefficientRowsSourceReceipt {
    producer_transaction_id: CallbackTransactionId,
    source_lineage: FitSourceLineage,
    source_generation_receipt: Flat7xD0SourceGenerationReceipt,
}

impl Flat7xCoefficientRowsSourceReceipt {
    pub fn new(
        producer_transaction_id: CallbackTransactionId,
        source_lineage: FitSourceLineage,
        source_generation_receipt: Flat7xD0SourceGenerationReceipt,
    ) -> Result<Self, CoefficientFittingError> {
        let source_generation_lineage = source_generation_receipt.callback_lineage();
        if &producer_transaction_id != source_generation_receipt.source_transaction_id()
            || &source_lineage.transaction_id != source_generation_lineage
            || source_lineage.programme_id.as_str()
                != source_generation_lineage.programme_id().as_str()
            || source_lineage.attempt.value() != source_generation_lineage.attempt_id().value()
            || source_lineage.source_index != source_generation_receipt.source_index()
            || source_lineage.source_index.dispatch().ok() != Some(source_lineage.dispatch)
        {
            return Err(CoefficientFittingError::Lineage);
        }
        Ok(Self {
            producer_transaction_id,
            source_lineage,
            source_generation_receipt,
        })
    }

    pub fn producer_transaction_id(&self) -> &CallbackTransactionId {
        &self.producer_transaction_id
    }

    pub const fn source_lineage(&self) -> &FitSourceLineage {
        &self.source_lineage
    }

    pub const fn source_generation_receipt(&self) -> &Flat7xD0SourceGenerationReceipt {
        &self.source_generation_receipt
    }
}

impl<'a> Flat7xCoefficientRowsProviderOutput<'a> {
    pub const fn new(
        rows: Flat7xCoefficientRows<'a>,
        source_receipt: Flat7xCoefficientRowsSourceReceipt,
    ) -> Self {
        Self {
            rows,
            source_receipt,
        }
    }

    pub const fn rows(&self) -> Flat7xCoefficientRows<'a> {
        self.rows
    }

    pub const fn source_receipt(&self) -> &Flat7xCoefficientRowsSourceReceipt {
        &self.source_receipt
    }

    #[cfg(any(test, feature = "test-support"))]
    /// Creates source-bound test evidence without asserting a production provider.
    ///
    /// # Panics
    ///
    /// Panics only if the fixed synthetic transaction identifier is invalid.
    pub fn synthetic_for_test(
        rows: Flat7xCoefficientRows<'a>,
        source_lineage: FitSourceLineage,
    ) -> Self {
        let producer_transaction_id = CallbackTransactionId::new("synthetic-provider-source")
            .expect("synthetic transaction id is valid");
        let source_generation_receipt = Flat7xD0SourceGenerationReceipt::synthetic_for_test(
            producer_transaction_id.clone(),
            1,
            source_lineage.transaction_id().clone(),
            source_lineage.source_index(),
            0,
        );
        let source_receipt = Flat7xCoefficientRowsSourceReceipt::new(
            producer_transaction_id,
            source_lineage,
            source_generation_receipt,
        )
        .expect("test provider receipt is internally bound");
        Self::new(rows, source_receipt)
    }
}

/// Caller-provided Flat7x rows paired with the fit owner's same-source after-state and lineage.
#[derive(Clone, Debug)]
pub struct Flat7xCoefficientRowsProposal<'a> {
    rows: Flat7xCoefficientRows<'a>,
    after: CoefficientFitState,
    source_lineage: FitSourceLineage,
}

impl<'a> Flat7xCoefficientRowsProposal<'a> {
    pub(crate) fn new(
        rows: Flat7xCoefficientRows<'a>,
        after: CoefficientFitState,
        source_lineage: FitSourceLineage,
    ) -> Result<Self, CoefficientFittingError> {
        if after.fit_generation() != source_lineage.dispatch().value()
            || source_lineage.source_index().value().checked_add(1)
                != Some(source_lineage.dispatch().value())
        {
            return Err(CoefficientFittingError::Generation);
        }
        Ok(Self {
            rows,
            after,
            source_lineage,
        })
    }

    /// Binds provider output to the accepted projection that produced this source generation.
    pub fn from_provider_output(
        fit_before: &CoefficientFitState,
        expected_source_generation_receipt: &Flat7xD0SourceGenerationReceipt,
        output: &Flat7xCoefficientRowsProviderOutput<'a>,
    ) -> Result<Self, CoefficientFittingError> {
        if output.source_receipt.source_generation_receipt != *expected_source_generation_receipt {
            return Err(CoefficientFittingError::Lineage);
        }
        let source_lineage = output.source_receipt.source_lineage.clone();
        let after = fit_before.after_external_flat7x_rows(source_lineage.dispatch())?;
        Self::new(output.rows, after, source_lineage)
    }

    /// Binds provider output to an already projection-derived source lineage.
    #[cfg(any(test, feature = "test-support"))]
    pub fn from_source_lineage(
        output: &Flat7xCoefficientRowsProviderOutput<'a>,
        source_lineage: FitSourceLineage,
        after: CoefficientFitState,
    ) -> Result<Self, CoefficientFittingError> {
        if output.source_receipt.source_lineage != source_lineage {
            return Err(CoefficientFittingError::Lineage);
        }
        Self::new(output.rows, after, source_lineage)
    }

    pub const fn rows(&self) -> Flat7xCoefficientRows<'a> {
        self.rows
    }

    pub const fn after(&self) -> &CoefficientFitState {
        &self.after
    }

    pub const fn source_lineage(&self) -> &FitSourceLineage {
        &self.source_lineage
    }
}

impl CoefficientFitProposal {
    pub const fn after(&self) -> &CoefficientFitState {
        &self.after
    }

    pub fn full_words(&self) -> &[u32] {
        &self.full_words
    }

    pub fn sparse_words(&self) -> &[u32] {
        &self.sparse_words
    }

    pub fn selected_channels(&self) -> &[u16] {
        &self.selected_channels
    }

    pub const fn score_call_count(&self) -> u32 {
        self.score_call_count
    }

    pub const fn source_lineage(&self) -> Option<&FitSourceLineage> {
        self.source_lineage.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn initial_full_payload_input(
        &self,
        clip_gain_index: u8,
        sequence_count: u16,
    ) -> Result<CoefficientPayloadInput, CoefficientFittingError> {
        let mut objects = Vec::with_capacity(OBJECT_COUNT);
        for object in 0..OBJECT_COUNT {
            let channel_rows = (0..CHANNEL_COUNT)
                .map(|channel| {
                    (0..BAND_COUNT)
                        .map(|band| {
                            f32::from_bits(
                                self.full_words
                                    [(object * CHANNEL_COUNT + channel) * BAND_COUNT + band],
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let selected_channels = (0..BAND_COUNT)
                .map(|band| {
                    u8::try_from(self.selected_channels[object * BAND_COUNT + band])
                        .map_err(|_| CoefficientFittingError::Shape)
                })
                .collect::<Result<Vec<_>, _>>()?;
            objects.push(CoefficientPayloadObject {
                present: true,
                band_enum: 5,
                mode: 1,
                points: vec![CoefficientPayloadPoint {
                    channel_rows,
                    selected_channels,
                }],
                offsets: self.representation_offset.map(|offset| vec![offset]),
            });
        }
        Ok(CoefficientPayloadInput {
            downmix_index: 3,
            extension_config: 0,
            clip_gain_index,
            sequence_count,
            time_slot_limit: 24,
            capacity_bytes: 372,
            objects,
        })
    }

    #[cfg(test)]
    pub(crate) fn write_initial_metadata_body(
        &self,
        clip_gain_index: u8,
        sequence_count: u16,
        writer_generation: CodecSourceIndex,
    ) -> Result<MetadataPayloadBody, CoefficientBodyError> {
        let input = self
            .initial_full_payload_input(clip_gain_index, sequence_count)
            .map_err(|_| CoefficientBodyError::RepresentationShape)?;
        write_selected_id14_coefficient_payload(&input)
            .map_err(CoefficientBodyError::Writer)?
            .into_metadata_body(writer_generation)
            .ok_or(CoefficientBodyError::NoSuccessfulPayload)
    }

    pub(crate) fn write_payload_with_retry(
        &self,
        provider: &RepresentationRetryProvider,
        clip_gain_index: u8,
        sequence_count: u16,
        capacity_bytes: u16,
    ) -> Result<RepresentationWriteResult, RepresentationRetryError> {
        write_fit_representation_with_retry(
            provider,
            FitRepresentationInput {
                full_words: &self.full_words,
                sparse_words: &self.sparse_words,
                selected_channels: &self.selected_channels,
                representation_offset: self.representation_offset,
                row_layout: JocCoefficientRowLayout::FiveChannel,
            },
            clip_gain_index,
            sequence_count,
            capacity_bytes,
        )
    }

    pub(crate) fn write_payload_with_resolved_profile(
        &self,
        profile: ResolvedEncodingProfile,
        provider: &RepresentationRetryProvider,
        clip_gain_index: u8,
        sequence_count: u16,
        capacity_bytes: u16,
    ) -> Result<RepresentationWriteResult, RepresentationRetryError> {
        write_fit_representation_with_resolved_profile(
            provider,
            FitRepresentationInput {
                full_words: &self.full_words,
                sparse_words: &self.sparse_words,
                selected_channels: &self.selected_channels,
                representation_offset: self.representation_offset,
                row_layout: JocCoefficientRowLayout::FiveChannel,
            },
            clip_gain_index,
            sequence_count,
            capacity_bytes,
            profile,
        )
    }

    #[cfg(test)]
    pub(crate) const fn row_states_after(&self) -> &[GeometryAccumulatorRow; SOURCE_ROWS] {
        &self.row_states_after
    }

    pub fn into_after(self) -> CoefficientFitState {
        self.after
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoefficientFittingError {
    Temporal(TemporalWeightError),
    Geometry(CoefficientGeometryError),
    NormalEquation(CoefficientNormalEquationError),
    Search(CoefficientSearchError),
    Generation,
    Lineage,
    Shape,
}

impl std::fmt::Display for CoefficientFittingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for CoefficientFittingError {}

#[cfg(test)]
#[derive(Debug)]
pub enum CoefficientBodyError {
    RepresentationShape,
    Writer(CoefficientPayloadWriterFailure),
    NoSuccessfulPayload,
}

#[cfg(test)]
impl std::fmt::Display for CoefficientBodyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

#[cfg(test)]
impl std::error::Error for CoefficientBodyError {}

pub fn stage_coefficient_fit_from_qmf(
    before: &CoefficientFitState,
    projection: &SelectedJocQmfProjection,
    qmf_provider: &QmfReceiverFunctionalProvider,
    score_provider: &FitScoreProvider,
) -> Result<CoefficientFitProposal, CoefficientFittingError> {
    let expected_dispatch = before
        .fit_generation
        .checked_add(1)
        .ok_or(CoefficientFittingError::Generation)?;
    if before.geometry.generation() != before.fit_generation
        || projection.dispatch().value() != expected_dispatch
    {
        return Err(CoefficientFittingError::Generation);
    }
    let temporal = before
        .temporal
        .stage_target_windows48(projection.target_object_window_words())
        .map_err(CoefficientFittingError::Temporal)?;
    let geometry = stage_geometry_from_qmf(
        &before.geometry,
        projection,
        temporal.after().weight_words(),
        qmf_provider,
    )
    .map_err(CoefficientFittingError::Geometry)?;
    assemble_fit(
        temporal,
        geometry,
        projection.target_object_window_words(),
        projection.carrier_window_words(),
        score_provider,
        before.object_count,
        before.seven_carrier_qmf.clone(),
        Some(FitSourceLineage::from_projection(projection)),
    )
}

#[cfg(test)]
pub(crate) fn stage_coefficient_fit_with_geometry_inputs(
    before: &CoefficientFitState,
    target_words: &[u32],
    carrier_words: &[u32],
    rows_before: &[GeometryAccumulatorRow; SOURCE_ROWS],
    target_control_words: &[[u32; LANE_COUNT]; SOURCE_ROWS],
    qmf_provider: &QmfReceiverFunctionalProvider,
    score_provider: &FitScoreProvider,
) -> Result<CoefficientFitProposal, CoefficientFittingError> {
    let temporal = before
        .temporal
        .stage_target_windows48(target_words)
        .map_err(CoefficientFittingError::Temporal)?;
    let geometry = stage_selected_geometry(
        &before.geometry,
        rows_before,
        target_control_words,
        temporal.after().weight_words(),
        qmf_provider,
    )
    .map_err(CoefficientFittingError::Geometry)?;
    assemble_fit(
        temporal,
        geometry,
        target_words,
        carrier_words,
        score_provider,
        before.object_count,
        before.seven_carrier_qmf.clone(),
        None,
    )
}

/// Search cells independently; collect in original order, including error precedence.
fn fit_independent_cells<const C: usize>(
    equations: &[CoefficientNormalEquation<C>],
    search: impl Fn(
        &CoefficientNormalEquation<C>,
    ) -> Result<CoefficientFitResult<C>, CoefficientSearchError>
    + Sync,
) -> Result<Vec<CoefficientFitResult<C>>, CoefficientFittingError> {
    use rayon::prelude::*;
    let Some(pool) = crate::dsp_workers::pool() else {
        return equations
            .iter()
            .map(search)
            .collect::<Result<_, _>>()
            .map_err(CoefficientFittingError::Search);
    };
    let results: Vec<_> =
        pool.install(|| equations.par_iter().with_min_len(8).map(&search).collect());
    results
        .into_iter()
        .collect::<Result<_, _>>()
        .map_err(CoefficientFittingError::Search)
}

fn assemble_fit(
    temporal: TemporalWeightProposal,
    geometry: CoefficientGeometryProposal,
    target_words: &[u32],
    carrier_words: &[u32],
    score_provider: &FitScoreProvider,
    object_count: crate::joc::JocObjectCount,
    seven_carrier_qmf: Option<SevenCarrierQmfState>,
    source_lineage: Option<FitSourceLineage>,
) -> Result<CoefficientFitProposal, CoefficientFittingError> {
    let fit_generation = geometry.after().generation();
    let representation_offset = temporal
        .before_segment()
        .changed()
        .then_some(temporal.before_segment().start());
    let grouped;
    let target_words = if object_count == crate::joc::JocObjectCount::Fifteen {
        target_words
    } else {
        let stride = target_words.len() / OBJECT_COUNT;
        let mut merged = vec![0; target_words.len()];
        for object in 0..object_count as usize {
            let output = &mut merged[object * stride..(object + 1) * stride];
            for slot in object_count.source_slots(object) {
                for (dst, &src) in output
                    .iter_mut()
                    .zip(&target_words[slot * stride..(slot + 1) * stride])
                {
                    *dst = (f32::from_bits(*dst) + f32::from_bits(src)).to_bits();
                }
            }
        }
        grouped = merged;
        &grouped
    };
    let equations = build_selected_normal_equations(
        target_words,
        carrier_words,
        temporal.after().weight_words(),
    )
    .map_err(CoefficientFittingError::NormalEquation)?;
    if equations.len() != OBJECT_COUNT * BAND_COUNT {
        return Err(CoefficientFittingError::Shape);
    }
    let mut full_words = vec![ZERO_BITS; object_count as usize * CHANNEL_COUNT * BAND_COUNT];
    let mut sparse_words = vec![ZERO_BITS; object_count as usize * BAND_COUNT];
    let mut selected_channels = vec![0_u16; object_count as usize * BAND_COUNT];
    let mut score_call_count = 0_u32;
    let cells = fit_independent_cells(
        &equations[..object_count as usize * BAND_COUNT],
        |equation| {
            search_selected_coefficient_cell(
                score_provider,
                equation,
                target_words,
                carrier_words,
                temporal.after().weight_words(),
                if object_count == crate::joc::JocObjectCount::Fifteen {
                    geometry.eligibility()[usize::from(equation.object())]
                } else {
                    [true; CHANNEL_COUNT]
                },
            )
        },
    )?;
    for (equation, result) in equations.iter().zip(cells) {
        let object = usize::from(equation.object());
        let band = usize::from(equation.band());
        for channel in 0..CHANNEL_COUNT {
            full_words[(object * CHANNEL_COUNT + channel) * BAND_COUNT + band] =
                result.stored_words()[channel];
        }
        sparse_words[object * BAND_COUNT + band] = result.sparse_stored_word();
        selected_channels[object * BAND_COUNT + band] = u16::from(result.sparse_selected_channel());
        score_call_count = score_call_count
            .checked_add(
                result
                    .stages()
                    .iter()
                    .map(|stage| u32::from(stage.candidate_count()))
                    .sum::<u32>(),
            )
            .ok_or(CoefficientFittingError::Shape)?;
    }
    #[cfg(test)]
    let row_states_after = *geometry.row_states_after();
    Ok(CoefficientFitProposal {
        after: CoefficientFitState {
            temporal: temporal.into_after(),
            geometry: geometry.into_after(),
            fit_generation,
            object_count,
            seven_carrier_qmf,
        },
        #[cfg(test)]
        row_states_after,
        full_words,
        sparse_words,
        selected_channels,
        score_call_count,
        representation_offset,
        source_lineage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        frontend::{PackageManifestEvidence, ProviderFileEvidence},
        joc::{
            coefficient_geometry::GeometryAccumulatorRow,
            coefficient_representation::RepresentationRetryProvider,
            dispatch_state::{
                JocDispatchError, JocDispatchState, JocPayloadInput, stage_joc_payload,
            },
            fit_score::FitScoreProvider,
            programme_state::SelectedJocState,
            source_stage::assemble_staged_source,
        },
        profile::EncodingProfile,
        spatial::{
            SpatialAudioMachine, validate_f04_parameter_package_manifest,
            validate_qmf_receiver_functional_provider,
        },
    };
    use serde_json::Value;
    use std::sync::Arc;

    const INPUT: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_qmf_fit_checkpoints/INPUT-ONLY.json");
    const EXPECTED: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_qmf_fit_checkpoints/EXPECTED.json");
    const MANIFEST: &[u8] = include_bytes!("../../tests/fixtures/f01e_provider/MANIFEST.json");
    const QMF_FUNCTIONAL: &[u8] =
        include_bytes!("../../tests/fixtures/f04_parameters/qmf-receiver-functional-v1.json");
    const JOC_FUNCTIONAL: &[u8] = include_bytes!(
        "../../tests/fixtures/synthetic_qmf_fit_checkpoints/approved-joc-functional.json"
    );
    const JOC_MATH: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_qmf_fit_checkpoints/approved-joc-math.json");

    #[test]
    fn flat7x_rows_proposal_carries_same_source_after_generation() {
        // SYNTHETIC/TEST_ONLY shape values verify typed provenance, not audio coefficient fitting.
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
        let programme_id = SourceFlagsProgrammeId::new("flat7x-rows-proposal-test").unwrap();
        let attempt = SourceFlagsAttempt::new(1);
        let source_index = CodecSourceIndex::new(0);
        let dispatch = crate::timeline::JocDispatchIndex::new(1);
        let lineage = FitSourceLineage::for_test(
            programme_id.clone(),
            attempt,
            CallbackLineageIdentity::new(programme_id, attempt, FrontendCallbackIndex::new(2)),
            source_index,
            dispatch,
            0,
            SpatialAudioMachine::new_cold(EncodingProfile::selected())
                .unwrap()
                .digest(),
        );
        let fit_before = CoefficientFitState::cold();
        let provider_output =
            Flat7xCoefficientRowsProviderOutput::synthetic_for_test(rows, lineage.clone());
        let expected_generation_receipt = provider_output
            .source_receipt()
            .source_generation_receipt()
            .clone();
        let proposal = Flat7xCoefficientRowsProposal::from_provider_output(
            &fit_before,
            &expected_generation_receipt,
            &provider_output,
        )
        .unwrap();
        assert_eq!(proposal.after().fit_generation(), dispatch.value());
        assert_eq!(proposal.source_lineage(), &lineage);
        let mismatched_generation_receipt = expected_generation_receipt.with_generation_for_test(2);
        assert!(matches!(
            Flat7xCoefficientRowsProposal::from_provider_output(
                &fit_before,
                &mismatched_generation_receipt,
                &provider_output,
            ),
            Err(CoefficientFittingError::Lineage)
        ));
    }

    #[test]
    fn three_generations_match_full_sparse_selected_and_all_score_call_counts() {
        let input: Value = serde_json::from_slice(INPUT).unwrap();
        let expected: Value = serde_json::from_slice(EXPECTED).unwrap();
        let manifest = validate_f04_parameter_package_manifest(&PackageManifestEvidence::new(
            Arc::from(MANIFEST),
            Some("fit composition manifest".to_owned()),
        ))
        .unwrap();
        let qmf_provider = validate_qmf_receiver_functional_provider(
            &ProviderFileEvidence::new(
                Arc::from(QMF_FUNCTIONAL),
                Some("fit composition QMF provider".to_owned()),
            ),
            &manifest,
        )
        .unwrap();
        let score_provider =
            FitScoreProvider::from_approved_bytes(JOC_FUNCTIONAL, JOC_MATH).unwrap();
        let retry_provider =
            RepresentationRetryProvider::from_approved_bytes(JOC_FUNCTIONAL).unwrap();
        let target_controls = synthetic_targets(&input);
        let mut state = CoefficientFitState::cold();
        let mut rows = [GeometryAccumulatorRow::cold(); SOURCE_ROWS];
        let mut previous_targets = vec![ZERO_BITS; OBJECT_COUNT * 24 * 64 * 2];
        let mut previous_carriers = vec![ZERO_BITS; CHANNEL_COUNT * 24 * 64 * 2];
        let mut score_calls = 0_u64;
        let mut dispatch_state = JocDispatchState::cold();
        let spatial_digest = SpatialAudioMachine::new_cold(EncodingProfile::selected())
            .unwrap()
            .digest();
        let sequences = [0_u16, 0, 1];

        for generation in 0..3 {
            let programme_id = crate::transaction::test_programme_id("synthetic-fit-programme");
            let attempt = AttemptId::new(7);
            let current_carriers = synthetic_carrier_current24(&input, generation);
            let current_targets = synthetic_target_current24(&input, generation, &current_carriers);
            let target_windows =
                concatenate_windows(&previous_targets, &current_targets, OBJECT_COUNT);
            let carrier_windows =
                concatenate_windows(&previous_carriers, &current_carriers, CHANNEL_COUNT);
            let mut proposal = stage_coefficient_fit_with_geometry_inputs(
                &state,
                &target_windows,
                &carrier_windows,
                &rows,
                &target_controls,
                &qmf_provider,
                &score_provider,
            )
            .unwrap();
            let source_index = CodecSourceIndex::new(u64::try_from(generation).unwrap());
            let callback_index = FrontendCallbackIndex::new(
                u32::try_from(generation).unwrap().checked_add(2).unwrap(),
            );
            let transaction_id =
                CallbackLineageIdentity::new(programme_id.clone(), attempt, callback_index);
            proposal.source_lineage = Some(FitSourceLineage {
                programme_id: programme_id.clone(),
                attempt,
                transaction_id: transaction_id.clone(),
                source_index,
                dispatch: source_index.dispatch().unwrap(),
                spatial_before_generation: u64::try_from(generation).unwrap(),
                spatial_before_digest: spatial_digest,
            });
            let checkpoint = &expected["checkpoints"][generation]["fit_outputs"];
            assert_eq!(
                proposal.after().fit_generation(),
                u64::try_from(generation + 1).unwrap()
            );
            assert_eq!(
                proposal.after().geometry().generation(),
                proposal.after().fit_generation()
            );
            assert_eq!(
                proposal.full_words(),
                decode_u32_words(checkpoint["full_f32le_hex"].as_str().unwrap()).as_slice()
            );
            assert_eq!(
                proposal.sparse_words(),
                decode_u32_words(checkpoint["sparse_f32le_hex"].as_str().unwrap()).as_slice()
            );
            assert_eq!(
                proposal.selected_channels(),
                decode_u16_words(checkpoint["selected_u16le_hex"].as_str().unwrap()).as_slice()
            );
            let metadata_body = proposal
                .write_initial_metadata_body(128, sequences[generation], source_index)
                .unwrap();
            let representation = &expected["checkpoints"][generation]["representation"];
            assert_eq!(
                metadata_body.bytes.as_slice(),
                decode_hex(representation["payload_hex"].as_str().unwrap()).as_slice()
            );
            assert_eq!(
                metadata_body.writer_generation.value(),
                u64::try_from(generation).unwrap()
            );
            let producer_transaction_id = crate::transaction::CallbackTransactionId::new(&format!(
                "selected-fit-parent-{generation}"
            ))
            .unwrap();
            let payload_input = JocPayloadInput::for_test_with_transaction(
                programme_id.clone(),
                attempt,
                transaction_id.clone(),
                producer_transaction_id.clone(),
                source_index,
                128,
                0x3d80_0000,
            );
            if generation == 2 {
                let wrap_before =
                    JocDispatchState::with_clock_for_test(source_index.dispatch().unwrap(), 1_023);
                let wrap = stage_joc_payload(
                    &wrap_before,
                    &proposal,
                    &JocPayloadInput::for_test(
                        programme_id.clone(),
                        attempt,
                        transaction_id.clone(),
                        source_index,
                        128,
                        0x3d80_0000,
                    ),
                    &retry_provider,
                )
                .unwrap();
                assert_eq!(wrap.sequence_count(), 1_023);
                assert_eq!(wrap.clip_gain_index(), 128);
                assert_eq!(wrap.after().sequence_counter(), 1);

                for invalid_counter in [-2_i64, 1_024] {
                    let invalid_before = JocDispatchState::with_clock_for_test(
                        source_index.dispatch().unwrap(),
                        invalid_counter,
                    );
                    assert!(matches!(
                        stage_joc_payload(
                            &invalid_before,
                            &proposal,
                            &payload_input,
                            &retry_provider,
                        ),
                        Err(JocDispatchError::Sequence)
                    ));
                }
            }
            let diagnostic = proposal
                .write_payload_with_retry(&retry_provider, 128, sequences[generation], 80)
                .unwrap();
            let diagnostic_expected =
                &expected["checkpoints"][generation]["diagnostic_capacity_representation"];
            assert_eq!(
                diagnostic.outcome().payload().unwrap(),
                decode_hex(diagnostic_expected["payload_hex"].as_str().unwrap()).as_slice()
            );
            let expected_attempts = diagnostic_expected["attempts"].as_array().unwrap();
            assert_eq!(diagnostic.attempts().len(), expected_attempts.len());
            for (attempt, expected_attempt) in diagnostic.attempts().iter().zip(expected_attempts) {
                assert_eq!(u64::from(attempt.attempt()), expected_attempt["attempt"]);
                assert_eq!(
                    u64::from(attempt.required_bytes()),
                    expected_attempt["bytes"]
                );
                assert_eq!(u64::from(attempt.mode()), expected_attempt["modes"][0]);
                assert_eq!(
                    u64::from(attempt.band_enum()),
                    expected_attempt["band_enums"][0]
                );
                assert_eq!(
                    attempt.succeeded(),
                    expected_attempt["failure_boundary"].is_null()
                );
            }
            score_calls += u64::from(proposal.score_call_count());
            rows = *proposal.row_states_after();
            let source = assemble_staged_source(
                &dispatch_state,
                proposal,
                &payload_input,
                &retry_provider,
                None,
            )
            .unwrap();
            assert_eq!(source.id14_body().body().bytes, metadata_body.bytes);
            assert_eq!(source.id14_body().body().writer_generation, source_index);
            assert_eq!(source.lineage().source_index(), source_index);
            assert_eq!(source.id14_body().provenance().lineage(), source.lineage());
            assert_eq!(
                source.id14_body().provenance().producer_transaction_id(),
                Some(&producer_transaction_id)
            );
            assert_eq!(
                source.id14_body().provenance().producer_generation(),
                source_index
            );
            assert_eq!(
                source.id14_body().provenance().dispatch(),
                source_index.dispatch().unwrap()
            );
            assert_eq!(
                source.id14_body().provenance().writer().value(),
                source_index.value()
            );
            assert_eq!(
                source.id14_body().provenance().sequence_count(),
                sequences[generation]
            );
            assert_eq!(source.id14_body().provenance().object_count().value(), 15);
            assert!(
                source
                    .id14_body()
                    .provenance()
                    .matches_body(source_index, &source.id14_body().body().bytes)
            );
            let (fit_after, dispatch_after, _, _) = source.into_parts();
            assert_eq!(
                dispatch_after.sequence_counter(),
                i64::try_from(generation).unwrap()
            );
            let selected_state = SelectedJocState::from_staged(fit_after, dispatch_after).unwrap();
            assert_eq!(
                selected_state.generation(),
                u64::try_from(generation + 1).unwrap()
            );
            assert_eq!(selected_state.canonical_bytes().unwrap().len(), 22_790);
            state = selected_state.fit().clone();
            dispatch_state = selected_state.dispatch().clone();
            previous_targets = current_targets;
            previous_carriers = current_carriers;
        }
        let expected_fit_stage_calls = expected["checkpoints"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|checkpoint| checkpoint["cells"].as_array().unwrap())
            .flat_map(|cell| cell["result"]["stages"].as_array().unwrap())
            .map(|stage| stage["candidate_count"].as_u64().unwrap())
            .sum::<u64>();
        assert_eq!(score_calls, expected_fit_stage_calls);
        assert!(expected["coverage"]["score_calls"].as_u64().unwrap() >= score_calls);
    }

    fn synthetic_targets(input: &Value) -> [[u32; LANE_COUNT]; SOURCE_ROWS] {
        const CARRIER_LANES: [usize; 5] = [0, 1, 2, 4, 5];
        let mut targets = [[ZERO_BITS; LANE_COUNT]; SOURCE_ROWS];
        targets[0][3] = 0x3f80_0000;
        for object in 0..OBJECT_COUNT {
            for carrier in input["target_control_carriers"][object].as_array().unwrap() {
                targets[object + 1][CARRIER_LANES[carrier.as_u64().unwrap() as usize]] =
                    0x3f80_0000;
            }
        }
        targets
    }

    fn synthetic_carrier_current24(input: &Value, generation: usize) -> Vec<u32> {
        let amplitude = input["generations"][generation]["amplitude"]
            .as_f64()
            .unwrap() as f32;
        let mut output = Vec::with_capacity(CHANNEL_COUNT * 24 * 64 * 2);
        for carrier in 0..CHANNEL_COUNT {
            for slot in 0..24 {
                for bin in 0..64 {
                    output
                        .extend(synthetic_carrier(carrier, slot, bin, amplitude).map(f32::to_bits));
                }
            }
        }
        output
    }

    fn synthetic_target_current24(input: &Value, generation: usize, carriers: &[u32]) -> Vec<u32> {
        let transient_slot = input["generations"][generation]["transient_slot"]
            .as_u64()
            .unwrap() as usize;
        let multiplier = input["generations"][generation]["transient_multiplier"]
            .as_f64()
            .unwrap() as f32;
        let mut output = Vec::with_capacity(OBJECT_COUNT * 24 * 64 * 2);
        for object in 0..OBJECT_COUNT {
            let (alpha, beta) = match object % 3 {
                0 => (0.75_f32, 0.0_f32),
                1 => (0.75_f32, 0.5_f32),
                _ => (0.5_f32, -0.25_f32),
            };
            for slot in 0..24 {
                for bin in 0..64 {
                    let first = current_complex(carriers, object % 5, slot, bin);
                    let second = current_complex(carriers, (object + 2) % 5, slot, bin);
                    let scale = if generation == 1 && object == 0 && slot >= transient_slot {
                        multiplier
                    } else {
                        1.0
                    };
                    for component in 0..2 {
                        output.push(
                            ((alpha * first[component] + beta * second[component]) * scale)
                                .to_bits(),
                        );
                    }
                }
            }
        }
        output
    }

    fn current_complex(words: &[u32], row: usize, slot: usize, bin: usize) -> [f32; 2] {
        let offset = ((row * 24 + slot) * 64 + bin) * 2;
        [
            f32::from_bits(words[offset]),
            f32::from_bits(words[offset + 1]),
        ]
    }

    fn synthetic_carrier(carrier: usize, slot: usize, bin: usize, amplitude: f32) -> [f32; 2] {
        let bit_index = if carrier == 1 && bin == 0 { 0 } else { carrier };
        [
            amplitude
                * (1.0 + (bin % 3) as f32 / 4.0)
                * if (slot >> bit_index) & 1 == 0 {
                    1.0
                } else {
                    -1.0
                },
            amplitude / 2.0
                * if ((slot + 1) >> ((bit_index + 1) % 5)) & 1 == 0 {
                    1.0
                } else {
                    -1.0
                },
        ]
    }

    fn concatenate_windows(previous: &[u32], current: &[u32], rows: usize) -> Vec<u32> {
        let words_per_half = 24 * 64 * 2;
        let mut output = Vec::with_capacity(rows * words_per_half * 2);
        for row in 0..rows {
            let start = row * words_per_half;
            let end = start + words_per_half;
            output.extend_from_slice(&previous[start..end]);
            output.extend_from_slice(&current[start..end]);
        }
        output
    }

    fn decode_u32_words(hex: &str) -> Vec<u32> {
        decode_hex(hex)
            .chunks_exact(4)
            .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
            .collect()
    }

    fn decode_u16_words(hex: &str) -> Vec<u16> {
        decode_hex(hex)
            .chunks_exact(2)
            .map(|word| u16::from_le_bytes(word.try_into().unwrap()))
            .collect()
    }

    fn decode_hex(hex: &str) -> Vec<u8> {
        hex.as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }
}
