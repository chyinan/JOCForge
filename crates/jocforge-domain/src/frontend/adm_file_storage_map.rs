// pattern: Functional Core
//! Typed converter-local ADM storage slots and channel-input lookup values.

use crate::frontend::selected_adm_graph::{SelectedAdmGraph, SelectedTrackKind};

const MAX_ADM_TRACK_COUNT: usize = u8::MAX as usize + 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmFileStorageTrackSlot(u8);

impl AdmFileStorageTrackSlot {
    pub const fn new(value: u8) -> Self {
        Self(value)
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
    entries_by_track_slot: Vec<AdmFileStorageChannelEntry>,
}

impl AdmFileStorageChannelMap {
    pub fn entries_by_track_slot(&self) -> &[AdmFileStorageChannelEntry] {
        &self.entries_by_track_slot
    }

    pub fn entry_for_slot(
        &self,
        slot: AdmFileStorageTrackSlot,
    ) -> Option<AdmFileStorageChannelEntry> {
        self.entries_by_track_slot
            .get(usize::from(slot.value()))
            .copied()
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
    let track_count = graph.tracks().len();
    if track_count == 0 || track_count > MAX_ADM_TRACK_COUNT {
        return Err(AdmFileStorageMapError::TrackSlotOutOfRange(
            u16::try_from(track_count).unwrap_or(u16::MAX),
        ));
    }
    let mut entries_by_slot = vec![None; track_count];

    for track in graph.tracks() {
        let track_index = track.track_index().value();
        let zero_based_slot = track_index
            .checked_sub(1)
            .ok_or(AdmFileStorageMapError::ChnaTrackIndexZero)?;
        let slot_value = u8::try_from(zero_based_slot)
            .map_err(|_| AdmFileStorageMapError::TrackSlotOutOfRange(zero_based_slot))?;
        let slot = AdmFileStorageTrackSlot::new(slot_value);
        let index = usize::from(slot.value());
        if index >= track_count {
            return Err(AdmFileStorageMapError::TrackSlotOutOfRange(zero_based_slot));
        }
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
    Ok(AdmFileStorageChannelMap {
        entries_by_track_slot: ordered_entries,
    })
}
