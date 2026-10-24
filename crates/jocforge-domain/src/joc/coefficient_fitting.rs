// pattern: Functional Core
//! Selected 15-object by 12-band coefficient fit proposal composition.

use crate::{
    joc::{
        coefficient_geometry::{
            CoefficientGeometryError, CoefficientGeometryProposal, CoefficientGeometryState,
            stage_geometry_from_qmf,
        },
        coefficient_normal_equations::{
            CoefficientNormalEquationError, build_selected_normal_equations,
        },
        coefficient_payload::{
            CoefficientPayloadInput, CoefficientPayloadObject, CoefficientPayloadPoint,
            CoefficientPayloadWriterFailure, write_selected_id14_coefficient_payload,
        },
        coefficient_representation::{
            FitRepresentationInput, RepresentationRetryError, RepresentationRetryProvider,
            RepresentationWriteResult, write_fit_representation_with_retry,
        },
        coefficient_search::{CoefficientSearchError, search_selected_coefficient_cell},
        emdf::metadata_owner::MetadataPayloadBody,
        fit_score::FitScoreProvider,
        temporal_weights::{TemporalWeightError, TemporalWeightProposal, TemporalWeightState},
    },
    spatial::{QmfReceiverFunctionalProvider, spatial_audio_state::SelectedJocQmfProjection},
    timeline::CodecSourceIndex,
};

const OBJECT_COUNT: usize = 15;
const CHANNEL_COUNT: usize = 5;
const BAND_COUNT: usize = 12;
#[cfg(test)]
const SOURCE_ROWS: usize = 16;
#[cfg(test)]
const LANE_COUNT: usize = 6;
const ZERO_BITS: u32 = 0x0000_0000;

#[cfg(test)]
use crate::joc::coefficient_geometry::{GeometryAccumulatorRow, stage_selected_geometry};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoefficientFitState {
    temporal: TemporalWeightState,
    geometry: CoefficientGeometryState,
    fit_generation: u64,
}

impl CoefficientFitState {
    pub const fn cold() -> Self {
        Self {
            temporal: TemporalWeightState::cold(),
            geometry: CoefficientGeometryState::cold(),
            fit_generation: 0,
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

    #[cfg(test)]
    pub(crate) const fn with_generation_for_test(generation: u64) -> Self {
        Self {
            temporal: TemporalWeightState::cold(),
            geometry: CoefficientGeometryState::with_generation_for_test(generation),
            fit_generation: generation,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FitSourceLineage {
    programme_id: Box<str>,
    attempt: u64,
    transaction_id: Box<str>,
    source_index: CodecSourceIndex,
    dispatch: crate::timeline::JocDispatchIndex,
    spatial_before_generation: u64,
    spatial_before_digest: crate::spatial::SpatialAudioDigest,
}

impl FitSourceLineage {
    pub fn programme_id(&self) -> &str {
        &self.programme_id
    }

    pub const fn attempt(&self) -> u64 {
        self.attempt
    }

    pub fn transaction_id(&self) -> &str {
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

    pub fn initial_full_payload_input(
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

    pub fn write_initial_metadata_body(
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

    pub fn write_payload_with_retry(
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
            },
            clip_gain_index,
            sequence_count,
            capacity_bytes,
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
    Shape,
}

impl std::fmt::Display for CoefficientFittingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for CoefficientFittingError {}

#[derive(Debug)]
pub enum CoefficientBodyError {
    RepresentationShape,
    Writer(CoefficientPayloadWriterFailure),
    NoSuccessfulPayload,
}

impl std::fmt::Display for CoefficientBodyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

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
        Some(FitSourceLineage {
            programme_id: projection.programme_id().into(),
            attempt: projection.attempt(),
            transaction_id: projection.transaction_id().into(),
            source_index: projection.source_index(),
            dispatch: projection.dispatch(),
            spatial_before_generation: projection.spatial_before_generation(),
            spatial_before_digest: projection.spatial_before_digest(),
        }),
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
        None,
    )
}

fn assemble_fit(
    temporal: TemporalWeightProposal,
    geometry: CoefficientGeometryProposal,
    target_words: &[u32],
    carrier_words: &[u32],
    score_provider: &FitScoreProvider,
    source_lineage: Option<FitSourceLineage>,
) -> Result<CoefficientFitProposal, CoefficientFittingError> {
    let fit_generation = geometry.after().generation();
    let representation_offset = temporal
        .before_segment()
        .changed()
        .then_some(temporal.before_segment().start());
    let equations = build_selected_normal_equations(
        target_words,
        carrier_words,
        temporal.after().weight_words(),
    )
    .map_err(CoefficientFittingError::NormalEquation)?;
    if equations.len() != OBJECT_COUNT * BAND_COUNT {
        return Err(CoefficientFittingError::Shape);
    }
    let mut full_words = vec![ZERO_BITS; OBJECT_COUNT * CHANNEL_COUNT * BAND_COUNT];
    let mut sparse_words = vec![ZERO_BITS; OBJECT_COUNT * BAND_COUNT];
    let mut selected_channels = vec![0_u16; OBJECT_COUNT * BAND_COUNT];
    let mut score_call_count = 0_u32;
    for equation in &equations {
        let object = usize::from(equation.object());
        let band = usize::from(equation.band());
        let result = search_selected_coefficient_cell(
            score_provider,
            equation,
            target_words,
            carrier_words,
            temporal.after().weight_words(),
            geometry.eligibility()[object],
        )
        .map_err(CoefficientFittingError::Search)?;
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
            dispatch_state::{JocDispatchState, JocPayloadInput, stage_joc_payload},
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
            proposal.source_lineage = Some(FitSourceLineage {
                programme_id: "synthetic-fit-programme".into(),
                attempt: 7,
                transaction_id: "synthetic-fit-transaction".into(),
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
            let payload_input = JocPayloadInput::for_test(
                "synthetic-fit-programme",
                7,
                "synthetic-fit-transaction",
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
                        "synthetic-fit-programme",
                        7,
                        "synthetic-fit-transaction",
                        source_index,
                        128,
                        0x3d80_0000,
                    ),
                    &retry_provider,
                )
                .unwrap();
                assert_eq!(wrap.sequence_count(), 1_023);
                assert_eq!(wrap.after().sequence_counter(), 1);
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
            let source =
                assemble_staged_source(&dispatch_state, proposal, &payload_input, &retry_provider)
                    .unwrap();
            assert_eq!(source.id14_body().bytes, metadata_body.bytes);
            assert_eq!(source.id14_body().writer_generation, source_index);
            assert_eq!(source.lineage().source_index(), source_index);
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
