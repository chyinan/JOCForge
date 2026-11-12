// pattern: Functional Core
//! Replaceable profile-specific ADM-row routing for the Flat7x I0/D0 source roles.
//!
//! The `PhysicalHostIndex` values come from the admitted ADM graph and address RIFF PCM rows.
//! The D0 `chanmap` locations use ETSI's separate MSB-indexed 16-bit domain. JOC reconstruction
//! carriers remain on `Flat7xI0D0TargetPlan`; this map does not reinterpret them.
//!
//! PROVISIONAL ASSUMPTION: selected DirectSpeakers bed roles `front_*`, `side_*`, `rear_*`, and
//! `lfe` are the source rows for the target I0 core and D0 mapped surround channels. The row
//! selection is replaceable and is not a DEE owner or runtime lineage finding.

use std::collections::BTreeMap;

use crate::frontend::selected_adm_graph::{PhysicalHostIndex, SelectedAdmGraph, SelectedTrackKind};

use super::{DependentChannelMap, Flat7xI0D0TargetPlan};

const D0_TARGET_CHANNEL_MAP: u16 = 0x1a00;
const I0_ROLE_ROWS: [(Flat7xI0CarrierRole, &str); 6] = [
    (Flat7xI0CarrierRole::Left, "front_left"),
    (Flat7xI0CarrierRole::Center, "front_center"),
    (Flat7xI0CarrierRole::Right, "front_right"),
    (Flat7xI0CarrierRole::LeftSurround, "side_left"),
    (Flat7xI0CarrierRole::RightSurround, "side_right"),
    (Flat7xI0CarrierRole::LowFrequencyEffects, "lfe"),
];

const REQUIRED_BED_ROLES: [&str; 8] = [
    "front_left",
    "front_center",
    "front_right",
    "lfe",
    "side_left",
    "side_right",
    "rear_left",
    "rear_right",
];

/// Zero-based physical PCM row from the selected ADM/CHNA graph.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AdmPcmRow(PhysicalHostIndex);

impl AdmPcmRow {
    pub const fn physical_host(self) -> PhysicalHostIndex {
        self.0
    }
}

/// I0's AC-3 core carrier order, independent of ADM row and D0 chanmap location types.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xI0CarrierRole {
    Left,
    Center,
    Right,
    LeftSurround,
    RightSurround,
    LowFrequencyEffects,
}

/// Semantic source role for one D0 coded channel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0SourceRole {
    LeftSurround,
    RightSurround,
    LeftRearSurround,
    RightRearSurround,
}

/// One enabled D0 `chanmap` location, indexed in ETSI's MSB-first table order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0ChannelMapLocation {
    LeftSurround,
    RightSurround,
    LeftRearRightRearPair,
}

impl Flat7xD0ChannelMapLocation {
    pub const fn msb_index(self) -> u8 {
        match self {
            Self::LeftSurround => 3,
            Self::RightSurround => 4,
            Self::LeftRearRightRearPair => 6,
        }
    }

    const fn numeric_mask(self) -> u16 {
        1_u16 << (15 - self.msb_index())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xI0RowBinding {
    role: Flat7xI0CarrierRole,
    row: AdmPcmRow,
}

impl Flat7xI0RowBinding {
    pub const fn role(self) -> Flat7xI0CarrierRole {
        self.role
    }

    pub const fn row(self) -> AdmPcmRow {
        self.row
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0ChannelSource {
    Single {
        role: Flat7xD0SourceRole,
        row: AdmPcmRow,
    },
    StereoPair {
        left_role: Flat7xD0SourceRole,
        left_row: AdmPcmRow,
        right_role: Flat7xD0SourceRole,
        right_row: AdmPcmRow,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0RowBinding {
    location: Flat7xD0ChannelMapLocation,
    source: Flat7xD0ChannelSource,
}

impl Flat7xD0RowBinding {
    pub const fn location(self) -> Flat7xD0ChannelMapLocation {
        self.location
    }

    pub const fn source(self) -> Flat7xD0ChannelSource {
        self.source
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xSourceRoleMapError {
    UnsupportedTarget,
    UnsupportedPhysicalRowCount { actual: u16 },
    PhysicalHostOutOfRange { row: u8, physical_rows: u16 },
    DuplicateBedRole(&'static str),
    MissingBedRole(&'static str),
    D0ChannelMapMismatch { actual: u16 },
    D0ChannelMapLocationMismatch,
}

/// Immutable source-row projection with an explicit, replaceable provisional assumption.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xProvisionalSourceRoleMap {
    physical_row_count: u16,
    i0_rows: [Flat7xI0RowBinding; 6],
    d0_rows: [Flat7xD0RowBinding; 3],
    d0_channel_map: DependentChannelMap,
}

impl Flat7xProvisionalSourceRoleMap {
    pub const fn physical_row_count(&self) -> u16 {
        self.physical_row_count
    }

    pub const fn i0_rows(&self) -> &[Flat7xI0RowBinding; 6] {
        &self.i0_rows
    }

    pub const fn d0_rows(&self) -> &[Flat7xD0RowBinding; 3] {
        &self.d0_rows
    }

    pub const fn d0_channel_map(&self) -> DependentChannelMap {
        self.d0_channel_map
    }

    pub const fn is_provisional(&self) -> bool {
        true
    }

    pub const fn assumption_id(&self) -> &'static str {
        "flat7x-direct-speaker-bed-rows-to-i0-d0-roles-v1"
    }
}

/// Builds the current selected-profile row map without conflating PCM rows, coded roles, or
/// `chanmap` bit locations. The relationship from ADM DirectSpeakers rows to I0/D0 roles remains
/// explicitly provisional and replaceable.
pub fn build_flat7x_provisional_source_role_map(
    graph: &SelectedAdmGraph,
    target: Flat7xI0D0TargetPlan,
) -> Result<Flat7xProvisionalSourceRoleMap, Flat7xSourceRoleMapError> {
    if target != Flat7xI0D0TargetPlan::approved_contract() {
        return Err(Flat7xSourceRoleMapError::UnsupportedTarget);
    }

    let physical_row_count = u16::try_from(graph.tracks().len())
        .map_err(|_| Flat7xSourceRoleMapError::UnsupportedPhysicalRowCount { actual: u16::MAX })?;
    if !target
        .accepted_physical_channel_counts()
        .contains(&physical_row_count)
    {
        return Err(Flat7xSourceRoleMapError::UnsupportedPhysicalRowCount {
            actual: physical_row_count,
        });
    }

    let channel_map = target
        .d0()
        .channel_map()
        .ok_or(Flat7xSourceRoleMapError::D0ChannelMapMismatch { actual: 0 })?;
    if channel_map.value() != D0_TARGET_CHANNEL_MAP {
        return Err(Flat7xSourceRoleMapError::D0ChannelMapMismatch {
            actual: channel_map.value(),
        });
    }

    let mut rows_by_role = BTreeMap::new();
    for track in graph.tracks() {
        let SelectedTrackKind::DirectSpeakersBed { role, .. } = track.kind() else {
            continue;
        };
        if !REQUIRED_BED_ROLES.contains(&role) {
            continue;
        }
        let row = AdmPcmRow(track.physical_host());
        if u16::from(row.physical_host().value()) >= physical_row_count {
            return Err(Flat7xSourceRoleMapError::PhysicalHostOutOfRange {
                row: row.physical_host().value(),
                physical_rows: physical_row_count,
            });
        }
        if rows_by_role.insert(role, row).is_some() {
            return Err(Flat7xSourceRoleMapError::DuplicateBedRole(role));
        }
    }

    for role in REQUIRED_BED_ROLES {
        if !rows_by_role.contains_key(role) {
            return Err(Flat7xSourceRoleMapError::MissingBedRole(role));
        }
    }

    let i0_rows = I0_ROLE_ROWS.map(|(role, speaker_role)| Flat7xI0RowBinding {
        role,
        row: rows_by_role[speaker_role],
    });

    // ETSI TS 102 366 V1.4.1 Table E.1.4 defines chanmap locations 3 and 4 as the
    // individual surround channels and location 6 as the Lrs/Rrs pair.
    let d0_rows = [
        Flat7xD0RowBinding {
            location: Flat7xD0ChannelMapLocation::LeftSurround,
            source: Flat7xD0ChannelSource::Single {
                role: Flat7xD0SourceRole::LeftSurround,
                row: rows_by_role["side_left"],
            },
        },
        Flat7xD0RowBinding {
            location: Flat7xD0ChannelMapLocation::RightSurround,
            source: Flat7xD0ChannelSource::Single {
                role: Flat7xD0SourceRole::RightSurround,
                row: rows_by_role["side_right"],
            },
        },
        Flat7xD0RowBinding {
            location: Flat7xD0ChannelMapLocation::LeftRearRightRearPair,
            source: Flat7xD0ChannelSource::StereoPair {
                left_role: Flat7xD0SourceRole::LeftRearSurround,
                left_row: rows_by_role["rear_left"],
                right_role: Flat7xD0SourceRole::RightRearSurround,
                right_row: rows_by_role["rear_right"],
            },
        },
    ];
    let actual_map = d0_rows
        .iter()
        .fold(0_u16, |map, entry| map | entry.location.numeric_mask());
    if actual_map != channel_map.value()
        || d0_rows.map(|entry| entry.location.msb_index()) != channel_map.msb_indexed_set_bits()
    {
        return Err(Flat7xSourceRoleMapError::D0ChannelMapLocationMismatch);
    }

    Ok(Flat7xProvisionalSourceRoleMap {
        physical_row_count,
        i0_rows,
        d0_rows,
        d0_channel_map: channel_map,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::selected_adm_graph::parse_selected_adm_graph;

    const AXML: &[u8] =
        include_bytes!("../../tests/fixtures/selected_adm_graph/source_metadata.xml");
    const CHNA: &[u8] =
        include_bytes!("../../tests/fixtures/selected_adm_graph/source_track_associations.bin");

    // The 36-row case is a fixture-derived CHNA prefix used to exercise the profile shape gate;
    // it is not presented as an independently captured 36-row ADM resource.
    fn graph_with_track_prefix(track_count: usize) -> SelectedAdmGraph {
        const CHNA_HEADER_BYTES: usize = 4;
        const CHNA_ENTRY_BYTES: usize = 40;
        let end = CHNA_HEADER_BYTES + track_count * CHNA_ENTRY_BYTES;
        let mut chna = CHNA[..end].to_vec();
        let count = u16::try_from(track_count).unwrap().to_le_bytes();
        chna[..2].copy_from_slice(&count);
        chna[2..4].copy_from_slice(&count);
        parse_selected_adm_graph(AXML, &chna).unwrap()
    }

    #[test]
    fn selected_graph_roles_produce_separate_i0_d0_row_and_chanmap_domains() {
        let graph = graph_with_track_prefix(38);
        let target = Flat7xI0D0TargetPlan::approved_contract();
        let map = build_flat7x_provisional_source_role_map(&graph, target).unwrap();

        assert_eq!(map.physical_row_count(), 38);
        assert!(map.is_provisional());
        assert_eq!(map.d0_channel_map().value(), D0_TARGET_CHANNEL_MAP);
        assert_eq!(
            map.i0_rows().map(|binding| binding.role()),
            [
                Flat7xI0CarrierRole::Left,
                Flat7xI0CarrierRole::Center,
                Flat7xI0CarrierRole::Right,
                Flat7xI0CarrierRole::LeftSurround,
                Flat7xI0CarrierRole::RightSurround,
                Flat7xI0CarrierRole::LowFrequencyEffects,
            ]
        );
        assert_eq!(
            map.d0_rows().map(|binding| binding.location().msb_index()),
            [3, 4, 6]
        );

        let row_for_role = |role: &str| {
            graph
                .tracks()
                .iter()
                .find_map(|track| match track.kind() {
                    SelectedTrackKind::DirectSpeakersBed { role: bed_role, .. }
                        if bed_role == role =>
                    {
                        Some(AdmPcmRow(track.physical_host()))
                    }
                    _ => None,
                })
                .unwrap()
        };
        assert_eq!(map.i0_rows()[0].row(), row_for_role("front_left"));
        assert_eq!(map.i0_rows()[5].row(), row_for_role("lfe"));
        assert_eq!(
            map.d0_rows()[0].source(),
            Flat7xD0ChannelSource::Single {
                role: Flat7xD0SourceRole::LeftSurround,
                row: row_for_role("side_left"),
            }
        );
        assert_eq!(
            map.d0_rows()[2].source(),
            Flat7xD0ChannelSource::StereoPair {
                left_role: Flat7xD0SourceRole::LeftRearSurround,
                left_row: row_for_role("rear_left"),
                right_role: Flat7xD0SourceRole::RightRearSurround,
                right_row: row_for_role("rear_right"),
            }
        );
    }

    #[test]
    fn fixture_derived_36_row_chna_prefix_uses_the_same_semantic_row_bindings() {
        let graph = graph_with_track_prefix(36);
        let target = Flat7xI0D0TargetPlan::approved_contract();
        let map = build_flat7x_provisional_source_role_map(&graph, target).unwrap();

        assert_eq!(map.physical_row_count(), 36);
        assert_eq!(map.d0_channel_map().value(), D0_TARGET_CHANNEL_MAP);
        let Flat7xD0ChannelSource::Single { row, .. } = map.d0_rows()[0].source() else {
            panic!("D0 chanmap location 3 is a single surround channel");
        };
        assert_eq!(map.i0_rows()[3].row(), row);
    }
}
