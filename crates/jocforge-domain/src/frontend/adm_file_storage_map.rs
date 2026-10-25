// pattern: Functional Core
//! Typed converter-local ADM storage slots and channel-input lookup values.

use crate::frontend::selected_adm_graph::{SelectedAdmGraph, SelectedTrackKind};

const SELECTED_TRACK_COUNT: usize = 38;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmFileStorageTrackSlot(u8);

impl AdmFileStorageTrackSlot {
    pub fn new(value: u8) -> Result<Self, AdmFileStorageMapError> {
        if usize::from(value) >= SELECTED_TRACK_COUNT {
            return Err(AdmFileStorageMapError::TrackSlotOutOfRange(u16::from(
                value,
            )));
        }
        Ok(Self(value))
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConverterChannelInputId(u32);

impl ConverterChannelInputId {
    pub const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmFileStorageChannelEntry {
    track_slot: AdmFileStorageTrackSlot,
    channel_input_id: ConverterChannelInputId,
}

impl AdmFileStorageChannelEntry {
    pub const fn track_slot(self) -> AdmFileStorageTrackSlot {
        self.track_slot
    }

    pub const fn channel_input_id(self) -> ConverterChannelInputId {
        self.channel_input_id
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmFileStorageMapError {
    ChnaTrackIndexZero,
    TrackSlotOutOfRange(u16),
    DuplicateTrackSlot(u8),
    MissingTrackSlot(u8),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmFileStorageChannelMap {
    entries_by_track_slot: [AdmFileStorageChannelEntry; SELECTED_TRACK_COUNT],
}

impl AdmFileStorageChannelMap {
    pub const fn entries_by_track_slot(
        &self,
    ) -> &[AdmFileStorageChannelEntry; SELECTED_TRACK_COUNT] {
        &self.entries_by_track_slot
    }

    pub fn entry_for_slot(&self, slot: AdmFileStorageTrackSlot) -> AdmFileStorageChannelEntry {
        self.entries_by_track_slot[usize::from(slot.value())]
    }
}

/// Projects the selected CHNA graph into the converter-local slot/value map.
///
/// Object values use the decoded channel-input ID. Bed values use the
/// graph-produced decoded ID plus bed ordinal. Neither value substitutes for
/// the separate zero-based CHNA track slot. This map does not resolve a PCM
/// row or meter frame descriptor.
pub fn build_adm_file_storage_channel_map(
    graph: &SelectedAdmGraph,
) -> Result<AdmFileStorageChannelMap, AdmFileStorageMapError> {
    let mut entries_by_slot: [Option<AdmFileStorageChannelEntry>; SELECTED_TRACK_COUNT] =
        [None; SELECTED_TRACK_COUNT];

    for track in graph.tracks() {
        let track_index = track.track_index().value();
        let zero_based_slot = track_index
            .checked_sub(1)
            .ok_or(AdmFileStorageMapError::ChnaTrackIndexZero)?;
        let slot_value = u8::try_from(zero_based_slot)
            .map_err(|_| AdmFileStorageMapError::TrackSlotOutOfRange(zero_based_slot))?;
        let slot = AdmFileStorageTrackSlot::new(slot_value)?;
        let index = usize::from(slot.value());
        let channel_input_id = match track.kind() {
            SelectedTrackKind::DirectSpeakersBed { bed_input_id, .. } => bed_input_id,
            SelectedTrackKind::Object { semantic_key } => semantic_key,
        };
        let entry = AdmFileStorageChannelEntry {
            track_slot: slot,
            channel_input_id: ConverterChannelInputId(channel_input_id),
        };
        if entries_by_slot[index].replace(entry).is_some() {
            return Err(AdmFileStorageMapError::DuplicateTrackSlot(slot_value));
        }
    }

    let ordered_entries = entries_by_slot
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            entry.ok_or_else(|| {
                AdmFileStorageMapError::MissingTrackSlot(u8::try_from(index).unwrap_or(u8::MAX))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let entries_by_track_slot = ordered_entries
        .try_into()
        .map_err(|_| AdmFileStorageMapError::MissingTrackSlot(u8::MAX))?;

    Ok(AdmFileStorageChannelMap {
        entries_by_track_slot,
    })
}
