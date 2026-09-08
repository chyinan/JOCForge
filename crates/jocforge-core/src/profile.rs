// pattern: Functional Core

use crate::{AdmObjectKind, AdmPosition, JocProfile};

/// Standards-defined JOC input topology used by the MVP.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JocProfileSpec {
    pub semantic_name: &'static str,
    pub downmix_index: u8,
    pub input_labels: &'static [&'static str],
    pub channel_map: u16,
    pub d0_source_indices: [usize; 2],
    pub phase_signaled: bool,
    pub base_profile: JocProfile,
}

const IDX0_LABELS: &[&str] = &["L", "R", "C", "Ls", "Rs"];
const IDX1_LABELS: &[&str] = &["L", "R", "C", "Ls", "Rs", "Lrs", "Rrs"];
const IDX2_LABELS: &[&str] = &["L", "R", "C", "Ls", "Rs", "Tfl", "Tfr"];

/// Returns the canonical Flat-7.X speaker slot for a public fixed-speaker
/// label. The slot order is the JOC reconstruction order, with LFE omitted.
#[must_use]
pub fn semantic_slot_for_label(label: &str) -> Option<usize> {
    match canonical_speaker_label(label)? {
        "L" => Some(0),
        "R" => Some(1),
        "C" => Some(2),
        "Ls" => Some(3),
        "Rs" => Some(4),
        "Lrs" | "Tfl" => Some(5),
        "Rrs" | "Tfr" => Some(6),
        _ => None,
    }
}

fn canonical_speaker_label(label: &str) -> Option<&'static str> {
    match label {
        "L" | "FL" | "RC_L" | "M-030" => Some("L"),
        "R" | "FR" | "RC_R" | "M+030" => Some("R"),
        "C" | "FC" | "RC_C" | "M+000" => Some("C"),
        "Ls" | "SL" | "RC_Ls" | "RC_Lss" => Some("Ls"),
        "Rs" | "SR" | "RC_Rs" | "RC_Rss" => Some("Rs"),
        "Lrs" | "Lb" | "BL" | "RC_Lrs" | "RC_Lb" => Some("Lrs"),
        "Rrs" | "Rb" | "BR" | "RC_Rrs" | "RC_Rb" => Some("Rrs"),
        "Tfl" | "TFL" | "RC_Lts" => Some("Tfl"),
        "Tfr" | "TFR" | "RC_Rts" => Some("Tfr"),
        _ => None,
    }
}

/// Returns the fixed-speaker slot only when the selected profile carries it.
#[must_use]
pub fn semantic_slot_for_profile_label(profile: JocProfile, label: &str) -> Option<usize> {
    let canonical = canonical_speaker_label(label)?;
    profile_spec(profile)
        .input_labels
        .iter()
        .position(|candidate| *candidate == canonical)
}

/// Returns a slot for an exact normalized synthetic speaker position.
/// Arbitrary object positions intentionally return `None`; their spatial
/// semantics remain OAMD metadata rather than a fixed base-channel claim.
#[must_use]
pub fn semantic_slot_for_position(position: AdmPosition) -> Option<usize> {
    let axis = |value: f64| {
        if (value + 1.0).abs() <= 1.0e-6 {
            Some(0_usize)
        } else if value.abs() <= 1.0e-6 {
            Some(1_usize)
        } else if (value - 1.0).abs() <= 1.0e-6 {
            Some(2_usize)
        } else {
            None
        }
    };
    if position.distance.abs() > 1.0e-6 {
        return None;
    }
    match (axis(position.azimuth), axis(position.elevation)) {
        (Some(0), Some(0)) => Some(0),
        (Some(2), Some(0)) => Some(1),
        (Some(1), Some(0)) => Some(2),
        (Some(0), Some(1)) => Some(3),
        (Some(2), Some(1)) => Some(4),
        (Some(0), Some(2)) => Some(5),
        (Some(2), Some(2)) => Some(6),
        _ => None,
    }
}

/// Returns the profile-owned slot for a carrier-eligible source channel.
///
/// DirectSpeakers use their public label when available. Object sources may
/// seed a fixed carrier only when their authored position is exactly a
/// canonical position in the selected profile; arbitrary object positions
/// remain OAMD semantics and return `None`.
#[must_use]
pub fn semantic_slot_for_source_channel(
    profile: JocProfile,
    kind: AdmObjectKind,
    speaker_label: Option<&str>,
    position: AdmPosition,
) -> Option<usize> {
    if kind == AdmObjectKind::DirectSpeakers {
        speaker_label
            .and_then(|label| {
                let canonical = label.rsplit(':').next().unwrap_or(label).trim();
                semantic_slot_for_profile_label(profile, canonical)
            })
            .or_else(|| semantic_slot_for_position(position))
    } else if kind == AdmObjectKind::Objects {
        semantic_slot_for_position(position)
    } else {
        None
    }
    .filter(|slot| *slot < profile_spec(profile).input_labels.len())
}

/// Returns the public Table-47 topology for a mandatory MVP profile.
#[must_use]
pub const fn profile_spec(profile: JocProfile) -> JocProfileSpec {
    match profile {
        JocProfile::Idx0 => JocProfileSpec {
            semantic_name: "5.X",
            downmix_index: 0,
            input_labels: IDX0_LABELS,
            channel_map: 0xc000,
            d0_source_indices: [0, 1],
            phase_signaled: false,
            base_profile: JocProfile::Idx0,
        },
        JocProfile::Idx1 => JocProfileSpec {
            semantic_name: "Flat-7.X",
            downmix_index: 1,
            input_labels: IDX1_LABELS,
            channel_map: 0x0200,
            d0_source_indices: [5, 6],
            phase_signaled: false,
            base_profile: JocProfile::Idx1,
        },
        JocProfile::Idx2 => JocProfileSpec {
            semantic_name: "5.X+2",
            downmix_index: 2,
            input_labels: IDX2_LABELS,
            channel_map: 0x0010,
            d0_source_indices: [5, 6],
            phase_signaled: false,
            base_profile: JocProfile::Idx2,
        },
        JocProfile::Idx3 => JocProfileSpec {
            semantic_name: "5.X Phase",
            downmix_index: 3,
            input_labels: IDX0_LABELS,
            channel_map: 0xc000,
            d0_source_indices: [0, 1],
            phase_signaled: true,
            base_profile: JocProfile::Idx0,
        },
        JocProfile::Idx4 => JocProfileSpec {
            semantic_name: "5.X+2 Phase",
            downmix_index: 4,
            input_labels: IDX2_LABELS,
            channel_map: 0x0010,
            d0_source_indices: [5, 6],
            phase_signaled: true,
            base_profile: JocProfile::Idx2,
        },
    }
}
