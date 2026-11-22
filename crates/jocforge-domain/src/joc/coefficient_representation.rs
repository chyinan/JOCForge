// pattern: Functional Core
//! Selected coefficient representation degradation and payload retry proposal.

use crate::joc::coefficient_payload::{
    CoefficientPayloadInput, CoefficientPayloadObject, CoefficientPayloadOutcome,
    CoefficientPayloadPoint, CoefficientPayloadWriterFailure,
    write_selected_id14_coefficient_payload, write_selected_id14_coefficient_payload_for_profile,
};
use crate::{
    joc::{
        coefficient_fitting::FitSourceLineage, emdf::metadata_owner::MetadataPayloadBody,
        id14_body::Id14BodyProposal,
    },
    profile::{JocPayloadDownmixIndex, ResolvedEncodingProfile},
    timeline::{CodecSourceIndex, JocDispatchIndex},
    transaction::CallbackTransactionId,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const FUNCTIONAL_SHA256: &str = "af08ef531f8708ab81cd6af0bf8ae71504ffb19ba76d5fb8571b1ae8db53c747";
const BAND_COUNT: usize = 12;
const BAND_COUNTS: [usize; 8] = [1, 3, 5, 7, 9, 12, 15, 23];
const MAX_ATTEMPTS: u8 = 16;

/// Required channel-row shape for one ID14 coefficient body.
///
/// Existing fit inputs provide five rows. A seven-row input is a separate producer seam and
/// requires its own source values before it can be selected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JocCoefficientRowLayout {
    FiveChannel,
    SevenChannel,
}

impl JocCoefficientRowLayout {
    pub const fn for_payload_downmix_index(index: JocPayloadDownmixIndex) -> Self {
        match index {
            JocPayloadDownmixIndex::FiveChannel
            | JocPayloadDownmixIndex::FiveChannelWithoutPhase => Self::FiveChannel,
            JocPayloadDownmixIndex::SevenChannel
            | JocPayloadDownmixIndex::HeightPair
            | JocPayloadDownmixIndex::HeightPairWithPhase => Self::SevenChannel,
        }
    }

    const fn channel_count(self) -> usize {
        match self {
            Self::FiveChannel => 5,
            Self::SevenChannel => 7,
        }
    }
}

#[derive(Debug)]
pub struct RepresentationRetryProvider {
    lower_maps: [Vec<usize>; 5],
}

impl RepresentationRetryProvider {
    pub fn from_approved_bytes(bytes: &[u8]) -> Result<Self, RepresentationRetryError> {
        if format!("{:x}", Sha256::digest(bytes)) != FUNCTIONAL_SHA256 {
            return Err(RepresentationRetryError::ProviderIdentity);
        }
        let root: Value =
            serde_json::from_slice(bytes).map_err(|_| RepresentationRetryError::ProviderShape)?;
        if root["selected_profile4"]["values"] != serde_json::json!([5, 1, 1, 372]) {
            return Err(RepresentationRetryError::ProviderShape);
        }
        let maps = &root["lower_map25"]["values_by_old_enum"];
        let mut parsed = Vec::with_capacity(5);
        for old_enum in 1..=5 {
            let expected = BAND_COUNTS[old_enum - 1];
            let values = maps[old_enum.to_string()]
                .as_array()
                .ok_or(RepresentationRetryError::ProviderShape)?;
            if values.len() != expected {
                return Err(RepresentationRetryError::ProviderShape);
            }
            let upper = BAND_COUNTS[old_enum];
            parsed.push(
                values
                    .iter()
                    .map(|value| {
                        let index = usize::try_from(
                            value
                                .as_u64()
                                .ok_or(RepresentationRetryError::ProviderShape)?,
                        )
                        .map_err(|_| RepresentationRetryError::ProviderShape)?;
                        if index < upper {
                            Ok(index)
                        } else {
                            Err(RepresentationRetryError::ProviderShape)
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        let lower_maps: [Vec<usize>; 5] = parsed
            .try_into()
            .map_err(|_| RepresentationRetryError::ProviderShape)?;
        Ok(Self { lower_maps })
    }

    fn map(&self, old_enum: u8) -> Result<&[usize], RepresentationRetryError> {
        if !(1..=5).contains(&old_enum) {
            return Err(RepresentationRetryError::ProviderShape);
        }
        Ok(&self.lower_maps[usize::from(old_enum - 1)])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FitRepresentationInput<'a> {
    pub full_words: &'a [u32],
    pub sparse_words: &'a [u32],
    pub selected_channels: &'a [u16],
    pub representation_offset: Option<u8>,
    pub row_layout: JocCoefficientRowLayout,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepresentationAttempt {
    attempt: u8,
    required_bytes: u16,
    mode: u8,
    band_enum: u8,
    succeeded: bool,
}

impl RepresentationAttempt {
    pub const fn attempt(&self) -> u8 {
        self.attempt
    }

    pub const fn required_bytes(&self) -> u16 {
        self.required_bytes
    }

    pub const fn mode(&self) -> u8 {
        self.mode
    }

    pub const fn band_enum(&self) -> u8 {
        self.band_enum
    }

    pub const fn succeeded(&self) -> bool {
        self.succeeded
    }
}

#[derive(Debug)]
pub struct RepresentationWriteResult {
    outcome: CoefficientPayloadOutcome,
    attempts: Vec<RepresentationAttempt>,
}

impl RepresentationWriteResult {
    pub(crate) fn from_single_outcome(outcome: CoefficientPayloadOutcome) -> Self {
        Self {
            outcome,
            attempts: Vec::new(),
        }
    }

    pub const fn outcome(&self) -> &CoefficientPayloadOutcome {
        &self.outcome
    }

    pub fn attempts(&self) -> &[RepresentationAttempt] {
        &self.attempts
    }

    pub fn into_outcome(self) -> CoefficientPayloadOutcome {
        self.outcome
    }

    pub fn into_metadata_body(
        self,
        writer_generation: CodecSourceIndex,
    ) -> Option<MetadataPayloadBody> {
        self.outcome.into_metadata_body(writer_generation)
    }

    pub(crate) fn into_id14_body_proposal(
        self,
        writer_generation: CodecSourceIndex,
        dispatch: JocDispatchIndex,
        expected_sequence_count: u16,
        producer_transaction_id: Option<CallbackTransactionId>,
        lineage: FitSourceLineage,
    ) -> Option<Id14BodyProposal> {
        self.outcome.into_id14_body_proposal(
            writer_generation,
            dispatch,
            expected_sequence_count,
            producer_transaction_id,
            lineage,
        )
    }
}

#[derive(Debug)]
pub enum RepresentationRetryError {
    ProviderIdentity,
    ProviderShape,
    FitShape,
    CapacityDomain,
    CoefficientRowLayoutMismatch {
        expected: JocCoefficientRowLayout,
        actual: JocCoefficientRowLayout,
    },
    Writer(CoefficientPayloadWriterFailure),
    MinimumRepresentationExceedsCapacity,
}

impl std::fmt::Display for RepresentationRetryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for RepresentationRetryError {}

pub fn write_fit_representation_with_retry(
    provider: &RepresentationRetryProvider,
    fit: FitRepresentationInput<'_>,
    clip_gain_index: u8,
    sequence_count: u16,
    capacity_bytes: u16,
) -> Result<RepresentationWriteResult, RepresentationRetryError> {
    write_fit_representation_with_profile(
        provider,
        fit,
        clip_gain_index,
        sequence_count,
        capacity_bytes,
        None,
    )
}

pub fn write_fit_representation_with_resolved_profile(
    provider: &RepresentationRetryProvider,
    fit: FitRepresentationInput<'_>,
    clip_gain_index: u8,
    sequence_count: u16,
    capacity_bytes: u16,
    profile: ResolvedEncodingProfile,
) -> Result<RepresentationWriteResult, RepresentationRetryError> {
    write_fit_representation_with_profile(
        provider,
        fit,
        clip_gain_index,
        sequence_count,
        capacity_bytes,
        Some(profile),
    )
}

fn write_fit_representation_with_profile(
    provider: &RepresentationRetryProvider,
    fit: FitRepresentationInput<'_>,
    clip_gain_index: u8,
    sequence_count: u16,
    capacity_bytes: u16,
    profile: Option<ResolvedEncodingProfile>,
) -> Result<RepresentationWriteResult, RepresentationRetryError> {
    if !(1..=543).contains(&capacity_bytes) {
        return Err(RepresentationRetryError::CapacityDomain);
    }
    let payload_downmix_index = profile.map_or(
        JocPayloadDownmixIndex::FiveChannel,
        ResolvedEncodingProfile::joc_payload_downmix_index,
    );
    validate_row_layout(payload_downmix_index, fit.row_layout)?;
    let mut objects = initial_objects(fit)?;
    let downmix_index = payload_downmix_index.value();
    let mut attempts = Vec::with_capacity(4);
    for attempt in 0..MAX_ATTEMPTS {
        let input = CoefficientPayloadInput {
            downmix_index,
            extension_config: 0,
            clip_gain_index,
            sequence_count,
            time_slot_limit: 24,
            capacity_bytes,
            objects: objects
                .iter()
                .map(ObjectRepresentation::payload_object)
                .collect(),
        };
        let outcome = match profile {
            Some(profile) => write_selected_id14_coefficient_payload_for_profile(&input, profile),
            None => write_selected_id14_coefficient_payload(&input),
        }
        .map_err(RepresentationRetryError::Writer)?;
        let required_bits = outcome
            .header_bit_count()
            .checked_add(outcome.body_bit_count().value())
            .ok_or(RepresentationRetryError::FitShape)?;
        let required_bytes = u16::try_from(required_bits.div_ceil(8))
            .map_err(|_| RepresentationRetryError::FitShape)?;
        let succeeded = outcome.payload().is_some();
        attempts.push(RepresentationAttempt {
            attempt,
            required_bytes,
            mode: objects[0].mode,
            band_enum: objects[0].band_enum,
            succeeded,
        });
        if succeeded {
            return Ok(RepresentationWriteResult { outcome, attempts });
        }
        if objects.iter().any(|object| matches!(object.mode, 1 | 3)) {
            for object in objects
                .iter_mut()
                .filter(|object| matches!(object.mode, 1 | 3))
            {
                object.mode -= 1;
            }
            continue;
        }
        // Preserve all seven channel gains when a Flat7x payload runs out of
        // space. Sparse coding changes the gain of every unselected channel.
        if !payload_downmix_index.uses_seven_carriers()
            && objects.iter().any(|object| object.mode == 0)
        {
            for object in objects.iter_mut().filter(|object| object.mode == 0) {
                object.mode = 2;
                for band in 0..object.band_count() {
                    let channel = usize::from(object.selected_channels[band]);
                    object.channel_rows[channel][band] = object.sparse_backup[band];
                }
            }
            continue;
        }
        if objects.iter().any(|object| object.band_enum > 0) {
            for object in objects.iter_mut().filter(|object| object.band_enum > 0) {
                let map = provider.map(object.band_enum)?;
                if payload_downmix_index.uses_seven_carriers() {
                    for row in &mut object.channel_rows {
                        *row = map.iter().map(|&source| row[source]).collect();
                    }
                } else {
                    for (band, source) in map.iter().copied().enumerate() {
                        let channel = usize::from(object.selected_channels[band]);
                        object.channel_rows[channel][band] = object.channel_rows[channel][source];
                    }
                }
                object.band_enum -= 1;
                let new_count = object.band_count();
                for row in &mut object.channel_rows {
                    row.truncate(new_count);
                }
                object.selected_channels.truncate(new_count);
                object.sparse_backup.truncate(new_count);
            }
            continue;
        }
        return Err(RepresentationRetryError::MinimumRepresentationExceedsCapacity);
    }
    Err(RepresentationRetryError::MinimumRepresentationExceedsCapacity)
}

fn validate_row_layout(
    payload_downmix_index: JocPayloadDownmixIndex,
    actual: JocCoefficientRowLayout,
) -> Result<(), RepresentationRetryError> {
    let expected = JocCoefficientRowLayout::for_payload_downmix_index(payload_downmix_index);
    if actual == expected {
        Ok(())
    } else {
        Err(RepresentationRetryError::CoefficientRowLayoutMismatch { expected, actual })
    }
}

#[derive(Clone, Debug)]
struct ObjectRepresentation {
    band_enum: u8,
    mode: u8,
    channel_rows: Vec<Vec<f32>>,
    selected_channels: Vec<u8>,
    sparse_backup: Vec<f32>,
    offsets: Option<Vec<u8>>,
}

impl ObjectRepresentation {
    fn band_count(&self) -> usize {
        BAND_COUNTS[usize::from(self.band_enum)]
    }

    fn payload_object(&self) -> CoefficientPayloadObject {
        CoefficientPayloadObject {
            present: true,
            band_enum: self.band_enum,
            mode: self.mode,
            points: vec![CoefficientPayloadPoint {
                channel_rows: self.channel_rows.clone(),
                selected_channels: self.selected_channels.clone(),
            }],
            offsets: self.offsets.clone(),
        }
    }
}

fn initial_objects(
    fit: FitRepresentationInput<'_>,
) -> Result<Vec<ObjectRepresentation>, RepresentationRetryError> {
    let channel_count = fit.row_layout.channel_count();
    let count = fit.selected_channels.len() / BAND_COUNT;
    if (channel_count == 7 && count != 15)
        || !matches!(count, 8 | 12 | 15)
        || fit.full_words.len() != count * channel_count * BAND_COUNT
        || fit.sparse_words.len() != count * BAND_COUNT
        || fit.selected_channels.len() != count * BAND_COUNT
        || fit.representation_offset.is_some_and(|offset| offset >= 24)
    {
        return Err(RepresentationRetryError::FitShape);
    }
    let mut objects = Vec::with_capacity(count);
    for object in 0..count {
        let channel_rows = (0..channel_count)
            .map(|channel| {
                (0..BAND_COUNT)
                    .map(|band| {
                        let word =
                            fit.full_words[(object * channel_count + channel) * BAND_COUNT + band];
                        let value = f32::from_bits(word);
                        if value.is_finite() {
                            Ok(value)
                        } else {
                            Err(RepresentationRetryError::FitShape)
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        let sparse_backup = (0..BAND_COUNT)
            .map(|band| {
                let value = f32::from_bits(fit.sparse_words[object * BAND_COUNT + band]);
                if value.is_finite() {
                    Ok(value)
                } else {
                    Err(RepresentationRetryError::FitShape)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let selected_channels = (0..BAND_COUNT)
            .map(|band| {
                let value = u8::try_from(fit.selected_channels[object * BAND_COUNT + band])
                    .map_err(|_| RepresentationRetryError::FitShape)?;
                if usize::from(value) < channel_count {
                    Ok(value)
                } else {
                    Err(RepresentationRetryError::FitShape)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        objects.push(ObjectRepresentation {
            band_enum: 5,
            mode: 1,
            channel_rows,
            selected_channels,
            sparse_backup,
            offsets: fit.representation_offset.map(|offset| vec![offset]),
        });
    }
    Ok(objects)
}

#[cfg(test)]
mod tests {
    use super::{JocCoefficientRowLayout, RepresentationRetryError, validate_row_layout};
    use crate::profile::JocPayloadDownmixIndex;

    #[test]
    fn flat7x_budget_retry_reduces_bands_and_keeps_all_seven_rows() {
        use crate::profile::{EncodingProfile, JocReconstructionLayout, resolve_selected_profile};

        let profile =
            resolve_selected_profile(&EncodingProfile::selected_with_joc_reconstruction_layout(
                JocReconstructionLayout::Flat7x,
            ))
            .unwrap();
        let maps = include_str!(
            "../../tests/fixtures/synthetic_qmf_fit_checkpoints/approved-joc-functional.json"
        )
        .replace("\r\n", "\n");
        let provider =
            super::RepresentationRetryProvider::from_approved_bytes(maps.as_bytes()).unwrap();
        let full: Vec<_> = (0..15 * 7 * 12)
            .map(|i| ((i % 31) as f32 / 128.0).to_bits())
            .collect();
        let fit = super::FitRepresentationInput {
            full_words: &full,
            sparse_words: &[0; 15 * 12],
            selected_channels: &[0; 15 * 12],
            row_layout: JocCoefficientRowLayout::SevenChannel,
            representation_offset: None,
        };
        let result = super::write_fit_representation_with_resolved_profile(
            &provider, fit, 128, 0, 372, profile,
        )
        .unwrap();
        let last = result.attempts().last().unwrap();
        assert!(last.succeeded() && last.band_enum() < 5);
        assert!(result.attempts().iter().all(|attempt| attempt.mode() < 2));
    }

    #[test]
    fn row_layout_gate_preserves_five_channel_and_exposes_flat7x_input_seam() {
        assert!(
            validate_row_layout(
                JocPayloadDownmixIndex::FiveChannel,
                JocCoefficientRowLayout::FiveChannel,
            )
            .is_ok()
        );

        assert!(matches!(
            validate_row_layout(
                JocPayloadDownmixIndex::SevenChannel,
                JocCoefficientRowLayout::FiveChannel,
            ),
            Err(RepresentationRetryError::CoefficientRowLayoutMismatch {
                expected: JocCoefficientRowLayout::SevenChannel,
                actual: JocCoefficientRowLayout::FiveChannel,
            })
        ));
    }
}
