// pattern: Functional Core
//! Selected coefficient representation degradation and payload retry proposal.

use crate::joc::coefficient_payload::{
    CoefficientPayloadInput, CoefficientPayloadObject, CoefficientPayloadOutcome,
    CoefficientPayloadPoint, CoefficientPayloadWriterFailure, selected_channel_count,
    write_selected_id14_coefficient_payload, write_selected_id14_coefficient_payload_for_profile,
};
use crate::{
    joc::{
        coefficient_fitting::FitSourceLineage, emdf::metadata_owner::MetadataPayloadBody,
        id14_body::Id14BodyProposal,
    },
    profile::ResolvedEncodingProfile,
    timeline::{CodecSourceIndex, JocDispatchIndex},
    transaction::CallbackTransactionId,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const FUNCTIONAL_SHA256: &str = "af08ef531f8708ab81cd6af0bf8ae71504ffb19ba76d5fb8571b1ae8db53c747";
const OBJECT_COUNT: usize = 15;
const BAND_COUNT: usize = 12;
const BAND_COUNTS: [usize; 8] = [1, 3, 5, 7, 9, 12, 15, 23];
const MAX_ATTEMPTS: u8 = 16;

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
    /// Caller-supplied row axis; it must match `downmix_index` and `full_words` exactly.
    pub channel_row_count: usize,
    /// ID14 coefficient selector; kept independent from the resolved JOC layout profile.
    pub downmix_index: u8,
    /// Caller-supplied initial representation mode. Selector 1 requires a sparse mode (2 or 3).
    pub initial_mode: u8,
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
    let mut objects = initial_objects(fit)?;
    let mut attempts = Vec::with_capacity(4);
    for attempt in 0..MAX_ATTEMPTS {
        let input = CoefficientPayloadInput {
            downmix_index: fit.downmix_index,
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
        if objects.iter().any(|object| object.mode == 0) {
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
                for (band, source) in map.iter().copied().enumerate() {
                    let channel = usize::from(object.selected_channels[band]);
                    object.channel_rows[channel][band] = object.channel_rows[channel][source];
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
    let expected_channel_count = usize::from(
        selected_channel_count(fit.downmix_index)
            .map_err(|_| RepresentationRetryError::FitShape)?,
    );
    if fit.channel_row_count != expected_channel_count
        || fit.initial_mode > 3
        || (fit.downmix_index == 1 && fit.initial_mode < 2)
        || fit.full_words.len() != OBJECT_COUNT * fit.channel_row_count * BAND_COUNT
        || fit.sparse_words.len() != OBJECT_COUNT * BAND_COUNT
        || fit.selected_channels.len() != OBJECT_COUNT * BAND_COUNT
        || fit.representation_offset.is_some_and(|offset| offset >= 24)
    {
        return Err(RepresentationRetryError::FitShape);
    }
    let mut objects = Vec::with_capacity(OBJECT_COUNT);
    for object in 0..OBJECT_COUNT {
        let channel_rows = (0..fit.channel_row_count)
            .map(|channel| {
                (0..BAND_COUNT)
                    .map(|band| {
                        let word = fit.full_words
                            [(object * fit.channel_row_count + channel) * BAND_COUNT + band];
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
                if usize::from(value) < fit.channel_row_count {
                    Ok(value)
                } else {
                    Err(RepresentationRetryError::FitShape)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        objects.push(ObjectRepresentation {
            band_enum: 5,
            mode: fit.initial_mode,
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
    use super::{
        BAND_COUNT, FitRepresentationInput, OBJECT_COUNT, RepresentationRetryError, initial_objects,
    };

    fn nonzero_fit_words(channel_count: usize) -> (Vec<u32>, Vec<u32>, Vec<u16>) {
        let full_words = (0..OBJECT_COUNT)
            .flat_map(|object| {
                (0..channel_count).flat_map(move |channel| {
                    (0..BAND_COUNT).map(move |band| {
                        let ordinal = (object * channel_count + channel) * BAND_COUNT + band + 1;
                        f32::from(u16::try_from(ordinal).unwrap()).to_bits()
                    })
                })
            })
            .collect();
        let sparse_words = (0..OBJECT_COUNT * BAND_COUNT)
            .map(|index| f32::from(u16::try_from(index + 1).unwrap()).to_bits())
            .collect();
        let selected_channels = (0..OBJECT_COUNT)
            .flat_map(|_| {
                (0..BAND_COUNT).map(move |band| u16::try_from(band % channel_count).unwrap())
            })
            .collect();
        (full_words, sparse_words, selected_channels)
    }

    #[test]
    fn explicit_seven_row_fit_preserves_each_callers_nonzero_row() {
        let (full_words, sparse_words, selected_channels) = nonzero_fit_words(7);
        let fit = FitRepresentationInput {
            full_words: &full_words,
            sparse_words: &sparse_words,
            selected_channels: &selected_channels,
            representation_offset: None,
            channel_row_count: 7,
            downmix_index: 1,
            initial_mode: 2,
        };

        let objects = initial_objects(fit).unwrap();
        assert_eq!(objects.len(), OBJECT_COUNT);
        assert_eq!(objects[0].channel_rows.len(), 7);
        assert_eq!(objects[0].mode, 2);
        for channel in 0..7 {
            let row = &objects[0].channel_rows[channel];
            assert_eq!(row.len(), BAND_COUNT);
            assert!(row.iter().all(|value| value.is_finite() && *value != 0.0));
            assert_eq!(
                row[0],
                f32::from(u16::try_from(channel * BAND_COUNT + 1).unwrap())
            );
        }
        for left in 0..7 {
            for right in left + 1..7 {
                assert_ne!(
                    objects[0].channel_rows[left],
                    objects[0].channel_rows[right]
                );
            }
        }
    }

    #[test]
    fn row_count_selector_and_sparse_mode_must_match_the_id14_shape() {
        let (full_words_7, sparse_words, selected_channels_7) = nonzero_fit_words(7);
        let seven_row_selector_three = FitRepresentationInput {
            full_words: &full_words_7,
            sparse_words: &sparse_words,
            selected_channels: &selected_channels_7,
            representation_offset: None,
            channel_row_count: 7,
            downmix_index: 3,
            initial_mode: 2,
        };
        assert!(matches!(
            initial_objects(seven_row_selector_three),
            Err(RepresentationRetryError::FitShape)
        ));

        let (full_words_6, sparse_words_6, selected_channels_6) = nonzero_fit_words(6);
        let six_row_selector_one = FitRepresentationInput {
            full_words: &full_words_6,
            sparse_words: &sparse_words_6,
            selected_channels: &selected_channels_6,
            representation_offset: None,
            channel_row_count: 6,
            downmix_index: 1,
            initial_mode: 2,
        };
        assert!(matches!(
            initial_objects(six_row_selector_one),
            Err(RepresentationRetryError::FitShape)
        ));

        let seven_row_full_mode = FitRepresentationInput {
            channel_row_count: 7,
            downmix_index: 1,
            initial_mode: 1,
            ..seven_row_selector_three
        };
        assert!(matches!(
            initial_objects(seven_row_full_mode),
            Err(RepresentationRetryError::FitShape)
        ));

        let (full_words_5, sparse_words_5, selected_channels_5) = nonzero_fit_words(5);
        let selector_one_with_five_rows = FitRepresentationInput {
            full_words: &full_words_5,
            sparse_words: &sparse_words_5,
            selected_channels: &selected_channels_5,
            representation_offset: None,
            channel_row_count: 5,
            downmix_index: 1,
            initial_mode: 2,
        };
        assert!(matches!(
            initial_objects(selector_one_with_five_rows),
            Err(RepresentationRetryError::FitShape)
        ));
    }

    #[test]
    fn selected_five_channel_fit_keeps_its_five_rows_and_selector_three() {
        let (full_words, sparse_words, selected_channels) = nonzero_fit_words(5);
        let fit = FitRepresentationInput {
            full_words: &full_words,
            sparse_words: &sparse_words,
            selected_channels: &selected_channels,
            representation_offset: None,
            channel_row_count: 5,
            downmix_index: 3,
            initial_mode: 1,
        };

        let objects = initial_objects(fit).unwrap();
        assert!(objects.iter().all(|object| object.channel_rows.len() == 5));
        assert!(objects.iter().all(|object| object.mode == 1));
        for (object_index, object) in objects.iter().enumerate() {
            for (channel, row) in object.channel_rows.iter().enumerate() {
                for (band, value) in row.iter().enumerate() {
                    let ordinal = (object_index * 5 + channel) * BAND_COUNT + band + 1;
                    assert_eq!(*value, f32::from(u16::try_from(ordinal).unwrap()));
                }
            }
        }
    }
}
