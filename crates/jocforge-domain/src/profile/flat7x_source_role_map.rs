// pattern: Functional Core
//! Replaceable profile-specific ADM-row routing for Flat7x I0/D0 and JOC source roles.
//!
//! The `PhysicalHostIndex` values come from the admitted ADM graph and address RIFF PCM rows.
//! The D0 `chanmap` locations use ETSI's separate MSB-indexed 16-bit domain. JOC reconstruction
//! JOC carrier roles have their own row bindings; codec coded-row order remains separate.
//!
//! PROVISIONAL ASSUMPTION: selected DirectSpeakers bed roles `front_*`, `side_*`, `rear_*`, and
//! `lfe` are the source rows for the target I0 core and D0 mapped surround channels. The row
//! selection is replaceable and is not a DEE owner or runtime lineage finding.

use std::collections::BTreeMap;

use crate::frontend::selected_adm_graph::{PhysicalHostIndex, SelectedAdmGraph, SelectedTrackKind};

use super::{DependentChannelMap, Flat7xI0D0TargetPlan};

const D0_TARGET_CHANNEL_MAP: u16 = 0x1a00;
const PCM_SAMPLES_PER_SOURCE: usize = 1_536;
const I0_ROLE_ROWS: [(Flat7xI0CarrierRole, &str); 6] = [
    (Flat7xI0CarrierRole::Left, "front_left"),
    (Flat7xI0CarrierRole::Center, "front_center"),
    (Flat7xI0CarrierRole::Right, "front_right"),
    (Flat7xI0CarrierRole::LeftSurround, "side_left"),
    (Flat7xI0CarrierRole::RightSurround, "side_right"),
    (Flat7xI0CarrierRole::LowFrequencyEffects, "lfe"),
];
const JOC_CARRIER_ROLE_ROWS: [(Flat7xJocCarrierRole, &str); 7] = [
    (Flat7xJocCarrierRole::Left, "front_left"),
    (Flat7xJocCarrierRole::Right, "front_right"),
    (Flat7xJocCarrierRole::Center, "front_center"),
    (Flat7xJocCarrierRole::LeftSurround, "side_left"),
    (Flat7xJocCarrierRole::RightSurround, "side_right"),
    (Flat7xJocCarrierRole::LeftRearSurround, "rear_left"),
    (Flat7xJocCarrierRole::RightRearSurround, "rear_right"),
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

impl Flat7xD0ChannelSource {
    pub const fn single_row(self) -> Option<AdmPcmRow> {
        match self {
            Self::Single { row, .. } => Some(row),
            Self::StereoPair { .. } => None,
        }
    }

    pub const fn pair_rows(self) -> Option<(AdmPcmRow, AdmPcmRow)> {
        match self {
            Self::Single { .. } => None,
            Self::StereoPair {
                left_row,
                right_row,
                ..
            } => Some((left_row, right_row)),
        }
    }
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

/// I0 PCM row view in AC-3 core order `[L, C, R, Ls, Rs, LFE]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xI0PcmRows<'a> {
    rows: [&'a [u32]; 6],
}

impl<'a> Flat7xI0PcmRows<'a> {
    pub const fn rows(self) -> [&'a [u32]; 6] {
        self.rows
    }
}

/// D0 PCM row view in enabled E-AC-3 `chanmap` order `[Ls, Rs, Lrs, Rrs]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0PcmRows<'a> {
    rows: [&'a [u32]; 4],
}

impl<'a> Flat7xD0PcmRows<'a> {
    pub const fn rows(self) -> [&'a [u32]; 4] {
        self.rows
    }
}

/// Semantic order for the seven JOC reconstruction carrier inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xJocCarrierRole {
    Left,
    Right,
    Center,
    LeftSurround,
    RightSurround,
    LeftRearSurround,
    RightRearSurround,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xJocCarrierRowBinding {
    role: Flat7xJocCarrierRole,
    row: AdmPcmRow,
}

impl Flat7xJocCarrierRowBinding {
    pub const fn role(self) -> Flat7xJocCarrierRole {
        self.role
    }

    pub const fn row(self) -> AdmPcmRow {
        self.row
    }
}

/// Borrowed ADM PCM rows in JOC target order `[L, R, C, Ls, Rs, Lrs, Rrs]`.
///
/// The physical rows come from the admitted source role map. LFE and top-side rows remain outside
/// this seven-row view; I0 and D0 keep their own coded-row views. The current role-map policy is
/// still provisional.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SevenCarrierPcmInput<'a> {
    rows_in_target_order: [&'a [u32]; 7],
    role_bindings: [Flat7xJocCarrierRowBinding; 7],
}

impl<'a> SevenCarrierPcmInput<'a> {
    pub const fn rows_in_target_order(self) -> [&'a [u32]; 7] {
        self.rows_in_target_order
    }

    pub const fn role_bindings(&self) -> &[Flat7xJocCarrierRowBinding; 7] {
        &self.role_bindings
    }

    pub const fn row(self, role: Flat7xJocCarrierRole) -> &'a [u32] {
        let index = match role {
            Flat7xJocCarrierRole::Left => 0,
            Flat7xJocCarrierRole::Right => 1,
            Flat7xJocCarrierRole::Center => 2,
            Flat7xJocCarrierRole::LeftSurround => 3,
            Flat7xJocCarrierRole::RightSurround => 4,
            Flat7xJocCarrierRole::LeftRearSurround => 5,
            Flat7xJocCarrierRole::RightRearSurround => 6,
        };
        self.rows_in_target_order[index]
    }
}

/// Borrowed source PCM projection after applying the provisional profile-specific role map.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xProvisionalSourcePcmView<'a> {
    physical_row_count: u16,
    i0: Flat7xI0PcmRows<'a>,
    d0: Flat7xD0PcmRows<'a>,
    seven_carriers: SevenCarrierPcmInput<'a>,
}

impl<'a> Flat7xProvisionalSourcePcmView<'a> {
    pub const fn physical_row_count(self) -> u16 {
        self.physical_row_count
    }

    pub const fn i0(self) -> Flat7xI0PcmRows<'a> {
        self.i0
    }

    pub const fn d0(self) -> Flat7xD0PcmRows<'a> {
        self.d0
    }

    /// Returns independently selected ADM role rows in JOC target order.
    pub const fn seven_carrier_input(self) -> SevenCarrierPcmInput<'a> {
        self.seven_carriers
    }

    pub const fn provisional(self) -> bool {
        true
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xSourceRoleMapError {
    UnsupportedTarget,
    UnsupportedPhysicalRowCount { actual: u16 },
    PhysicalHostOutOfRange { row: u8, physical_rows: u16 },
    PhysicalPcmRowCountMismatch { expected: u16, actual: usize },
    PhysicalPcmSampleCountMismatch { row: usize, actual: usize },
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
    joc_carrier_rows: [Flat7xJocCarrierRowBinding; 7],
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

    pub const fn joc_carrier_rows(&self) -> &[Flat7xJocCarrierRowBinding; 7] {
        &self.joc_carrier_rows
    }

    pub const fn d0_channel_map(&self) -> DependentChannelMap {
        self.d0_channel_map
    }

    pub const fn is_provisional(&self) -> bool {
        true
    }

    pub const fn assumption_id(&self) -> &'static str {
        "flat7x-direct-speaker-bed-rows-to-i0-d0-joc-roles-v1"
    }
}

/// Projects one callback's physical PCM rows through the profile-specific I0 and D0 bindings.
/// The function preserves the independent index domains and only returns borrowed row slices.
pub fn project_flat7x_source_pcm_rows<'a>(
    role_map: &Flat7xProvisionalSourceRoleMap,
    physical_pcm_rows: &'a [Vec<u32>],
) -> Result<Flat7xProvisionalSourcePcmView<'a>, Flat7xSourceRoleMapError> {
    if physical_pcm_rows.len() != usize::from(role_map.physical_row_count()) {
        return Err(Flat7xSourceRoleMapError::PhysicalPcmRowCountMismatch {
            expected: role_map.physical_row_count(),
            actual: physical_pcm_rows.len(),
        });
    }
    for (row, samples) in physical_pcm_rows.iter().enumerate() {
        if samples.len() != PCM_SAMPLES_PER_SOURCE {
            return Err(Flat7xSourceRoleMapError::PhysicalPcmSampleCountMismatch {
                row,
                actual: samples.len(),
            });
        }
    }

    let mut i0_rows = [&[][..]; 6];
    for (destination, binding) in i0_rows.iter_mut().zip(role_map.i0_rows) {
        *destination = physical_pcm_rows
            .get(usize::from(binding.row.physical_host().value()))
            .ok_or(Flat7xSourceRoleMapError::PhysicalHostOutOfRange {
                row: binding.row.physical_host().value(),
                physical_rows: role_map.physical_row_count(),
            })?;
    }

    let mut d0_rows = [&[][..]; 4];
    let mut next_d0_row = 0;
    for binding in role_map.d0_rows {
        let mut append = |row: AdmPcmRow| -> Result<(), Flat7xSourceRoleMapError> {
            let Some(destination) = d0_rows.get_mut(next_d0_row) else {
                return Err(Flat7xSourceRoleMapError::D0ChannelMapLocationMismatch);
            };
            *destination = physical_pcm_rows
                .get(usize::from(row.physical_host().value()))
                .ok_or(Flat7xSourceRoleMapError::PhysicalHostOutOfRange {
                    row: row.physical_host().value(),
                    physical_rows: role_map.physical_row_count(),
                })?;
            next_d0_row += 1;
            Ok(())
        };
        match binding.source {
            Flat7xD0ChannelSource::Single { row, .. } => append(row)?,
            Flat7xD0ChannelSource::StereoPair {
                left_row,
                right_row,
                ..
            } => {
                append(left_row)?;
                append(right_row)?;
            }
        }
    }
    if next_d0_row != d0_rows.len() {
        return Err(Flat7xSourceRoleMapError::D0ChannelMapLocationMismatch);
    }

    let mut seven_carrier_rows = [&[][..]; 7];
    for (destination, binding) in seven_carrier_rows.iter_mut().zip(role_map.joc_carrier_rows) {
        *destination = physical_pcm_rows
            .get(usize::from(binding.row.physical_host().value()))
            .ok_or(Flat7xSourceRoleMapError::PhysicalHostOutOfRange {
                row: binding.row.physical_host().value(),
                physical_rows: role_map.physical_row_count,
            })?;
    }

    Ok(Flat7xProvisionalSourcePcmView {
        physical_row_count: role_map.physical_row_count(),
        i0: Flat7xI0PcmRows { rows: i0_rows },
        d0: Flat7xD0PcmRows { rows: d0_rows },
        seven_carriers: SevenCarrierPcmInput {
            rows_in_target_order: seven_carrier_rows,
            role_bindings: role_map.joc_carrier_rows,
        },
    })
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
    let joc_carrier_rows =
        JOC_CARRIER_ROLE_ROWS.map(|(role, speaker_role)| Flat7xJocCarrierRowBinding {
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
        joc_carrier_rows,
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
            map.i0_rows().map(Flat7xI0RowBinding::role),
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

        let physical_rows = (0..38)
            .map(|row| vec![u32::try_from(row).unwrap(); PCM_SAMPLES_PER_SOURCE])
            .collect::<Vec<_>>();
        let pcm_view = project_flat7x_source_pcm_rows(&map, &physical_rows).unwrap();
        assert_eq!(pcm_view.physical_row_count(), 38);
        assert!(pcm_view.provisional());
        assert_eq!(
            pcm_view.i0().rows().map(|row| row[0]),
            map.i0_rows()
                .map(|binding| u32::from(binding.row().physical_host().value()))
        );
        assert_eq!(
            pcm_view.d0().rows().map(|row| row[0]),
            [
                u32::from(
                    map.d0_rows()[0]
                        .source()
                        .single_row()
                        .unwrap()
                        .physical_host()
                        .value()
                ),
                u32::from(
                    map.d0_rows()[1]
                        .source()
                        .single_row()
                        .unwrap()
                        .physical_host()
                        .value()
                ),
                u32::from(
                    map.d0_rows()[2]
                        .source()
                        .pair_rows()
                        .unwrap()
                        .0
                        .physical_host()
                        .value()
                ),
                u32::from(
                    map.d0_rows()[2]
                        .source()
                        .pair_rows()
                        .unwrap()
                        .1
                        .physical_host()
                        .value()
                ),
            ]
        );
        assert_eq!(
            pcm_view
                .seven_carrier_input()
                .rows_in_target_order()
                .map(|row| row[0]),
            [0, 1, 2, 4, 5, 6, 7]
        );
        assert_eq!(
            pcm_view
                .seven_carrier_input()
                .row(Flat7xJocCarrierRole::LeftRearSurround)[0],
            6
        );
        assert_eq!(pcm_view.i0().rows()[5][0], 3);

        let one_hot_physical_rows = (0..38)
            .map(|row| {
                let marker = if row < 10 { 1_u32 << row } else { 0 };
                vec![marker; PCM_SAMPLES_PER_SOURCE]
            })
            .collect::<Vec<_>>();
        let one_hot_view = project_flat7x_source_pcm_rows(&map, &one_hot_physical_rows).unwrap();
        assert_eq!(
            map.joc_carrier_rows()
                .map(|binding| binding.row().physical_host().value()),
            [0, 1, 2, 4, 5, 6, 7]
        );
        assert_eq!(
            one_hot_view
                .seven_carrier_input()
                .rows_in_target_order()
                .map(|row| row[0]),
            [1, 2, 4, 16, 32, 64, 128]
        );
        assert_eq!(
            one_hot_view.i0().rows().map(|row| row[0]),
            [1, 4, 2, 16, 32, 8]
        );
        assert_eq!(
            one_hot_view.d0().rows().map(|row| row[0]),
            [16, 32, 64, 128]
        );
    }

    #[test]
    fn fixture_derived_36_row_chna_prefix_uses_the_same_semantic_row_bindings() {
        let graph = graph_with_track_prefix(36);
        let target = Flat7xI0D0TargetPlan::approved_contract();
        let map = build_flat7x_provisional_source_role_map(&graph, target).unwrap();

        assert_eq!(map.physical_row_count(), 36);
        assert_eq!(map.d0_channel_map().value(), D0_TARGET_CHANNEL_MAP);
        let physical_rows = (0..36)
            .map(|row| vec![u32::try_from(row).unwrap(); PCM_SAMPLES_PER_SOURCE])
            .collect::<Vec<_>>();
        let pcm_view = project_flat7x_source_pcm_rows(&map, &physical_rows).unwrap();
        assert_eq!(pcm_view.physical_row_count(), 36);
        let Flat7xD0ChannelSource::Single { row, .. } = map.d0_rows()[0].source() else {
            panic!("D0 chanmap location 3 is a single surround channel");
        };
        assert_eq!(map.i0_rows()[3].row(), row);
        assert_eq!(
            pcm_view.i0().rows()[3],
            physical_rows[usize::from(row.physical_host().value())]
        );
    }

    #[test]
    fn d0_pcm_view_contains_only_the_four_target_chanmap_channels() {
        let d0_values = [[20_u32; 3], [21_u32; 3], [22_u32; 3], [23_u32; 3]];
        let d0 = Flat7xD0PcmRows {
            rows: [&d0_values[0], &d0_values[1], &d0_values[2], &d0_values[3]],
        };

        assert_eq!(d0.rows().map(|row| row[0]), [20, 21, 22, 23]);
        assert_eq!(d0.rows().len(), 4);
    }
}
