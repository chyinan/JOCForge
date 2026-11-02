// pattern: Functional Core
//! F01E selected bed pack-to-physical-host identity join.
//!
//! The caller supplies graph-derived physical rows. This module joins by exact
//! `channel_format_id`; it does not parse AXML/CHNA or infer host order.

const SELECTED_BED_COUNT: usize = 10;

/// One F01G graph-produced physical bed track row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalBedTrack {
    channel_format_id: String,
    host_channel_index: u8,
    semantic_role: String,
    bed_id: u8,
}

impl PhysicalBedTrack {
    pub fn new(
        channel_format_id: String,
        host_channel_index: u8,
        semantic_role: String,
        bed_id: u8,
    ) -> Self {
        Self {
            channel_format_id,
            host_channel_index,
            semantic_role,
            bed_id,
        }
    }
}

/// Local selected route errors; they are not native error status claims.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BedChannelJoinError {
    PackDomain,
    PhysicalIdentity,
    HostDomain,
    LfeIdentity,
}

impl BedChannelJoinError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::PackDomain => "BED_PACK_DOMAIN",
            Self::PhysicalIdentity => "BED_PACK_PHYSICAL_IDENTITY",
            Self::HostDomain => "BED_HOST_DOMAIN",
            Self::LfeIdentity => "BED_LFE_IDENTITY",
        }
    }
}

/// Exact selected bed-pack order to physical host map and regular-control order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedBedChannelJoin {
    slots_by_pack: [u8; SELECTED_BED_COUNT],
    lfe_host: u8,
    object_hosts: Vec<u8>,
    regular_control_order: Vec<u8>,
}

impl SelectedBedChannelJoin {
    pub const fn slots_by_pack(&self) -> &[u8; SELECTED_BED_COUNT] {
        &self.slots_by_pack
    }

    pub const fn lfe_host(&self) -> u8 {
        self.lfe_host
    }

    pub fn object_hosts(&self) -> &[u8] {
        &self.object_hosts
    }

    pub fn regular_control_order(&self) -> &[u8] {
        &self.regular_control_order
    }
}

/// Joins the selected ten-channel DirectSpeakers pack to physical bed rows.
///
/// Object hosts are supplied by the same caller-owned F01G graph projection.
/// The result uses exact identity matching and the sorted physical non-LFE
/// order; pack ordinal never substitutes for a physical host index.
pub fn join_selected_bed_channels(
    pack_channel_format_ids: &[String],
    physical_bed_tracks: &[PhysicalBedTrack],
    object_host_slots: &[u8],
) -> Result<SelectedBedChannelJoin, BedChannelJoinError> {
    if pack_channel_format_ids.len() != SELECTED_BED_COUNT
        || physical_bed_tracks.len() != SELECTED_BED_COUNT
    {
        return Err(BedChannelJoinError::PackDomain);
    }
    let physical_host_count = physical_bed_tracks
        .len()
        .checked_add(object_host_slots.len())
        .ok_or(BedChannelJoinError::HostDomain)?;
    if physical_host_count == 0 || physical_host_count > usize::from(u8::MAX) + 1 {
        return Err(BedChannelJoinError::HostDomain);
    }

    let mut pack_ids = std::collections::BTreeSet::new();
    if pack_channel_format_ids
        .iter()
        .any(|channel_id| channel_id.is_empty() || !pack_ids.insert(channel_id.as_str()))
    {
        return Err(BedChannelJoinError::PackDomain);
    }

    let mut physical_by_id = std::collections::BTreeMap::new();
    let mut occupied_hosts = vec![false; physical_host_count];
    let mut lfe_host = None;
    for physical in physical_bed_tracks {
        if physical.channel_format_id.is_empty()
            || physical_by_id.contains_key(physical.channel_format_id.as_str())
        {
            return Err(BedChannelJoinError::PhysicalIdentity);
        }
        physical_by_id.insert(
            physical.channel_format_id.as_str(),
            physical.host_channel_index,
        );
        let host_index = usize::from(physical.host_channel_index);
        if host_index >= physical_host_count
            || std::mem::replace(&mut occupied_hosts[host_index], true)
        {
            return Err(BedChannelJoinError::HostDomain);
        }
        if physical.semantic_role == "lfe"
            && (physical.bed_id != 3 || lfe_host.replace(physical.host_channel_index).is_some())
        {
            return Err(BedChannelJoinError::LfeIdentity);
        }
    }

    if pack_ids.len() != physical_by_id.len()
        || pack_ids
            .iter()
            .any(|channel_id| !physical_by_id.contains_key(channel_id))
    {
        return Err(BedChannelJoinError::PhysicalIdentity);
    }

    let object_hosts = (0..physical_host_count)
        .filter(|host| !occupied_hosts[*host])
        .map(|host| u8::try_from(host).map_err(|_| BedChannelJoinError::HostDomain))
        .collect::<Result<Vec<_>, _>>()?;
    if object_hosts.len() != object_host_slots.len() {
        return Err(BedChannelJoinError::HostDomain);
    }

    for host in object_host_slots {
        let host_index = usize::from(*host);
        if host_index >= physical_host_count
            || std::mem::replace(&mut occupied_hosts[host_index], true)
        {
            return Err(BedChannelJoinError::HostDomain);
        }
    }
    if occupied_hosts.iter().any(|occupied| !occupied) {
        return Err(BedChannelJoinError::HostDomain);
    }

    let lfe_host = lfe_host.ok_or(BedChannelJoinError::LfeIdentity)?;
    let slots_by_pack = pack_channel_format_ids
        .iter()
        .map(|channel_id| {
            physical_by_id
                .get(channel_id.as_str())
                .copied()
                .ok_or(BedChannelJoinError::PhysicalIdentity)
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| BedChannelJoinError::PackDomain)?;
    let regular_control_order = (0..physical_host_count)
        .filter(|host| u8::try_from(*host).ok() != Some(lfe_host))
        .map(|host| u8::try_from(host).map_err(|_| BedChannelJoinError::HostDomain))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(SelectedBedChannelJoin {
        slots_by_pack,
        lfe_host,
        object_hosts,
        regular_control_order,
    })
}
