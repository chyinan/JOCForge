// pattern: Functional Core

use crate::JocProfile;

/// Standards-defined JOC input topology used by the MVP.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JocProfileSpec {
    pub downmix_index: u8,
    pub input_labels: &'static [&'static str],
    pub channel_map: u16,
}

const IDX0_LABELS: &[&str] = &["L", "R", "C", "Ls", "Rs"];
const IDX1_LABELS: &[&str] = &["L", "R", "C", "Ls", "Rs", "Lrs", "Rrs"];
const IDX2_LABELS: &[&str] = &["L", "R", "C", "Ls", "Rs", "Tfl", "Tfr"];

/// Returns the public Table-47 topology for a mandatory MVP profile.
#[must_use]
pub const fn profile_spec(profile: JocProfile) -> JocProfileSpec {
    match profile {
        JocProfile::Idx0 => JocProfileSpec {
            downmix_index: 0,
            input_labels: IDX0_LABELS,
            channel_map: 0xc000,
        },
        JocProfile::Idx1 => JocProfileSpec {
            downmix_index: 1,
            input_labels: IDX1_LABELS,
            channel_map: 0x0200,
        },
        JocProfile::Idx2 => JocProfileSpec {
            downmix_index: 2,
            input_labels: IDX2_LABELS,
            channel_map: 0x0010,
        },
    }
}
