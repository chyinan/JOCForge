// pattern: Functional Core

use crate::JocProfile;

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
