// pattern: Functional Core
//! Validated same-callback source-flags production and its raw-history proposal.

use super::spatial_control::{
    SpatialControlDigest, SpatialControlMachine, SpatialControlProviderLineage,
};
use crate::{
    frontend::{
        ProviderSha256,
        normal_control_snapshot::{
            NormalControlHeldSourceView, NormalControlPreparedSnapshot, NormalControlRow,
            NormalControlSnapshotProposal, NormalControlTrackKind,
        },
        primary_metadata_control::{PrimaryControl38Fields, PrimaryMetadataControlRows},
        selected_adm_graph::{SelectedAdmGraph, SelectedTrackKind},
    },
    numeric::{FiniteF32, R32Error, r32_mul},
    profile::{EncodingProfile, ResolvedEncodingProfile, resolve_selected_profile},
    timeline::{FrontendCallbackIndex, NormalCallbackCount},
    transaction::{AttemptId, CallbackLineageIdentity, ProgrammeId},
};
use sha2::{Digest, Sha256};
use std::sync::Arc;

const HOST_CHANNELS: usize = 38;
const REGULAR_HOSTS: usize = 37;
const SOURCE_COUNT: usize = 48;
const PREFIX_SOURCE_COUNT: usize = 11;
const FRAME_SAMPLES: usize = 1536;
const WINDOW_WORDS: usize = FRAME_SAMPLES * 2;
const ACTIVITY_EPSILON_BITS: u32 = 0x3400_0000;
const WIDTH_LIMIT_BITS: u32 = 0x3e4c_cccd;
const SELECTED_FEATURE_COMPONENT_ENUM: u8 = 0;
const SELECTED_FEATURE_DIFFUSE: u8 = 0;
const BED_SOURCE_WIDTH_POSITIVE_ZERO_BITS: u32 = 0x0000_0000;
const WINDOW_SHA256: &str = "134a5a788d42b0532f3ec46112ad978cc14fb3c46cb0d82f02fee5f129b7c919";

type FiniteFrame = Box<[FiniteF32; FRAME_SAMPLES]>;
pub type RawHistoryFrame = [u32; FRAME_SAMPLES];
type RawFrame = Box<RawHistoryFrame>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceFlagsError {
    InvalidHost,
    InvalidRouteRecord,
    RouteHostCount,
    RouteHostDuplicate,
    RouteLfeCount,
    RouteLfeDestination,
    RouteNotBijective,
    UnsupportedRouteBranch,
    RegularControlCount,
    RegularControlDuplicate,
    RegularControlHostMismatch,
    InvalidMute,
    NonfiniteWidth,
    UnsupportedComponentOrDiffuse,
    PrimaryMetadataIdentityMismatch,
    PrimaryControlRowMismatch,
    PrimaryRegularSourceMapMismatch,
    SourceControlClock,
    SourceControlClockMismatch,
    PcmShape,
    NonfinitePcm,
    HistoryShape,
    MissingBeforeHistory,
    PrefixWriterDomain,
    EmptyProgrammeIdentity,
    UnsupportedProfile,
    ProviderLineageMismatch,
    StaleCallbackGeneration,
    WrongHistoryGeneration,
    HistoryLineageMismatch,
    WindowKnownAnswerMismatch,
    NonfiniteProduct,
    InvalidFlagPair,
    LineageMismatch,
}

impl SourceFlagsError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidHost => "HOST_DOMAIN",
            Self::InvalidRouteRecord => "ROUTE_RECORD",
            Self::RouteHostCount => "ROUTE_HOST_COUNT",
            Self::RouteHostDuplicate => "ROUTE_HOST_DUPLICATE",
            Self::RouteLfeCount => "ROUTE_LFE_COUNT",
            Self::RouteLfeDestination => "ROUTE_LFE_DESTINATION",
            Self::RouteNotBijective => "ROUTE_NOT_BIJECTIVE",
            Self::UnsupportedRouteBranch => "UNSUPPORTED_ROUTE_BRANCH",
            Self::RegularControlCount => "CONTROL_COUNT",
            Self::RegularControlDuplicate => "CONTROL_HOST_DUPLICATE",
            Self::RegularControlHostMismatch => "CONTROL_HOST_MISMATCH",
            Self::InvalidMute => "INVALID_MUTE",
            Self::NonfiniteWidth => "NONFINITE_WIDTH",
            Self::UnsupportedComponentOrDiffuse => "UNSUPPORTED_COMPONENT_OR_DIFFUSE",
            Self::PrimaryMetadataIdentityMismatch => "PRIMARY_METADATA_IDENTITY_MISMATCH",
            Self::PrimaryControlRowMismatch => "PRIMARY_CONTROL_ROW_MISMATCH",
            Self::PrimaryRegularSourceMapMismatch => "PRIMARY_REGULAR_SOURCE_MAP_MISMATCH",
            Self::SourceControlClock => "SOURCE_CONTROL_CLOCK",
            Self::SourceControlClockMismatch => "SOURCE_CONTROL_CLOCK_MISMATCH",
            Self::PcmShape => "PCM_SHAPE",
            Self::NonfinitePcm => "NONFINITE_PCM",
            Self::HistoryShape => "HISTORY_SHAPE",
            Self::MissingBeforeHistory => "MISSING_BEFORE_HISTORY",
            Self::PrefixWriterDomain => "PREFIX_WRITER_DOMAIN",
            Self::EmptyProgrammeIdentity => "PROGRAMME_IDENTITY",
            Self::UnsupportedProfile => "UNSUPPORTED_PROFILE",
            Self::ProviderLineageMismatch => "PROVIDER_LINEAGE_MISMATCH",
            Self::StaleCallbackGeneration => "STALE_CALLBACK_GENERATION",
            Self::WrongHistoryGeneration => "WRONG_HISTORY_GENERATION",
            Self::HistoryLineageMismatch => "HISTORY_LINEAGE_MISMATCH",
            Self::WindowKnownAnswerMismatch => "WINDOW_KAT_MISMATCH",
            Self::NonfiniteProduct => "NONFINITE_PRODUCT",
            Self::InvalidFlagPair => "INVALID_FLAG_PAIR",
            Self::LineageMismatch => "LINEAGE_MISMATCH",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceFlagsHistoryGeneration(u64);

impl SourceFlagsHistoryGeneration {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

pub type SourceFlagsProgrammeId = ProgrammeId;
pub type SourceFlagsAttempt = AttemptId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostId(u8);

impl HostId {
    pub fn new(value: u8) -> Result<Self, SourceFlagsError> {
        if usize::from(value) < HOST_CHANNELS {
            Ok(Self(value))
        } else {
            Err(SourceFlagsError::InvalidHost)
        }
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

/// Selected-profile F03R feature controls, distinct from authored audio controls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedFeatureFlagsControl {
    profile_identity: ResolvedEncodingProfile,
    callback_index: FrontendCallbackIndex,
    host: HostId,
    component_enum: u8,
    diffuse: u8,
}

impl SelectedFeatureFlagsControl {
    /// LOCAL_TRANSLATION: selected-profile feature inputs are `(component=0,diffuse=0)`.
    pub fn from_admitted_profile(
        profile_identity: ResolvedEncodingProfile,
        callback_index: FrontendCallbackIndex,
        host_channel_index: u8,
    ) -> Result<Self, SourceFlagsError> {
        let selected_profile = resolve_selected_profile(&EncodingProfile::selected_with_carriage(
            profile_identity.carriage(),
        ))
        .map_err(|_| SourceFlagsError::UnsupportedProfile)?;
        if selected_profile != profile_identity {
            return Err(SourceFlagsError::UnsupportedProfile);
        }
        Ok(Self {
            profile_identity,
            callback_index,
            host: HostId::new(host_channel_index)?,
            component_enum: SELECTED_FEATURE_COMPONENT_ENUM,
            diffuse: SELECTED_FEATURE_DIFFUSE,
        })
    }

    pub const fn profile_identity(self) -> ResolvedEncodingProfile {
        self.profile_identity
    }

    pub const fn callback_index(self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn host(self) -> HostId {
        self.host
    }

    pub const fn component_enum(self) -> u8 {
        self.component_enum
    }

    pub const fn diffuse(self) -> u8 {
        self.diffuse
    }

    pub fn into_regular_control(
        self,
        mute: u8,
        width: f32,
    ) -> Result<RegularControl, SourceFlagsError> {
        RegularControl::from_raw(
            self.host.value(),
            mute,
            width,
            self.component_enum,
            self.diffuse,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostRole {
    Regular,
    Lfe,
}

/// Untrusted semantic route record. The producer only accepts a fully validated record set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostRouteRecord {
    host: HostId,
    role: HostRole,
    enabled: bool,
    destination: Option<u8>,
}

impl HostRouteRecord {
    pub fn new(
        host: u8,
        role: HostRole,
        enabled: bool,
        destination: Option<u8>,
    ) -> Result<Self, SourceFlagsError> {
        if destination.is_some_and(|value| usize::from(value) >= SOURCE_COUNT) {
            return Err(SourceFlagsError::InvalidRouteRecord);
        }
        Ok(Self {
            host: HostId::new(host)?,
            role,
            enabled,
            destination,
        })
    }
}

/// Validated exact 38-host semantic route with one excluded LFE and ordered 11..47 destinations.
#[derive(Debug)]
pub struct ValidatedSemanticRoute {
    lfe_host: HostId,
    regular_hosts: Box<[HostId]>,
    destination_by_host: Box<[Option<u8>]>,
    receipt_sha256: ProviderSha256,
}

impl ValidatedSemanticRoute {
    pub fn new(records: Vec<HostRouteRecord>) -> Result<Self, SourceFlagsError> {
        if records.len() != HOST_CHANNELS {
            return Err(SourceFlagsError::RouteHostCount);
        }
        let mut seen = [false; HOST_CHANNELS];
        let mut lfe = None;
        let mut regular_hosts = Vec::with_capacity(REGULAR_HOSTS);
        let mut destination_by_host = Vec::with_capacity(HOST_CHANNELS);
        destination_by_host.resize(HOST_CHANNELS, None);
        let mut canonical = Vec::with_capacity(HOST_CHANNELS * 4);
        let mut next_destination = 11_u8;
        for record in records {
            let slot = usize::from(record.host.value());
            if std::mem::replace(&mut seen[slot], true) {
                return Err(SourceFlagsError::RouteHostDuplicate);
            }
            if !record.enabled {
                return Err(SourceFlagsError::UnsupportedRouteBranch);
            }
            canonical.push(record.host.value());
            canonical.push(u8::from(record.role == HostRole::Lfe));
            canonical.push(u8::from(record.enabled));
            match record.role {
                HostRole::Lfe => {
                    if lfe.replace(record.host).is_some() {
                        return Err(SourceFlagsError::RouteLfeCount);
                    }
                    if record.destination.is_some() {
                        return Err(SourceFlagsError::RouteLfeDestination);
                    }
                    canonical.push(u8::MAX);
                }
                HostRole::Regular => {
                    let destination = record
                        .destination
                        .ok_or(SourceFlagsError::RouteNotBijective)?;
                    if usize::from(destination) != usize::from(next_destination)
                        || !(11..=47).contains(&destination)
                    {
                        return Err(SourceFlagsError::RouteNotBijective);
                    }
                    regular_hosts.push(record.host);
                    destination_by_host[usize::from(record.host.value())] = Some(destination);
                    canonical.push(destination);
                    next_destination += 1;
                }
            }
        }
        if seen.iter().any(|present| !present) {
            return Err(SourceFlagsError::RouteHostCount);
        }
        if regular_hosts.len() != REGULAR_HOSTS || next_destination != 48 {
            return Err(SourceFlagsError::RouteLfeCount);
        }
        let lfe_host = lfe.ok_or(SourceFlagsError::RouteLfeCount)?;
        Ok(Self {
            lfe_host,
            regular_hosts: regular_hosts.into_boxed_slice(),
            destination_by_host: destination_by_host.into_boxed_slice(),
            receipt_sha256: ProviderSha256::digest(&canonical),
        })
    }

    pub fn lfe_host(&self) -> HostId {
        self.lfe_host
    }

    pub fn regular_hosts(&self) -> &[HostId] {
        &self.regular_hosts
    }

    /// Returns the admitted global source for one physical host, or `None` for LFE.
    pub fn destination_for_host(&self, host: HostId) -> Option<u8> {
        self.destination_by_host[usize::from(host.value())]
    }

    fn destination(&self, host: HostId) -> Result<u8, SourceFlagsError> {
        self.destination_by_host[usize::from(host.value())]
            .ok_or(SourceFlagsError::RouteNotBijective)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegularControl {
    host: HostId,
    muted: bool,
    width: FiniteF32,
}

impl RegularControl {
    pub fn from_raw(
        host: u8,
        mute: u8,
        width: f32,
        component: u8,
        diffuse: u8,
    ) -> Result<Self, SourceFlagsError> {
        let host = HostId::new(host)?;
        if mute > 1 {
            return Err(SourceFlagsError::InvalidMute);
        }
        let width = FiniteF32::new(width).map_err(|_| SourceFlagsError::NonfiniteWidth)?;
        if component != 0 || diffuse != 0 {
            return Err(SourceFlagsError::UnsupportedComponentOrDiffuse);
        }
        Ok(Self {
            host,
            muted: mute == 1,
            width,
        })
    }

    pub const fn host(self) -> HostId {
        self.host
    }

    pub const fn is_muted(self) -> bool {
        self.muted
    }

    pub const fn width(self) -> FiniteF32 {
        self.width
    }
}

#[derive(Debug)]
pub struct RegularControls {
    values: Box<[RegularControl]>,
}

impl RegularControls {
    pub fn new(values: Vec<RegularControl>) -> Result<Self, SourceFlagsError> {
        if values.len() != REGULAR_HOSTS {
            return Err(SourceFlagsError::RegularControlCount);
        }
        let mut seen = [false; HOST_CHANNELS];
        for control in &values {
            let slot = usize::from(control.host.value());
            if std::mem::replace(&mut seen[slot], true) {
                return Err(SourceFlagsError::RegularControlDuplicate);
            }
        }
        Ok(Self {
            values: values.into_boxed_slice(),
        })
    }

    fn for_host(&self, host: HostId) -> Result<RegularControl, SourceFlagsError> {
        self.values
            .iter()
            .copied()
            .find(|control| control.host == host)
            .ok_or(SourceFlagsError::RegularControlHostMismatch)
    }

    pub fn rows(&self) -> &[RegularControl] {
        &self.values
    }
}

/// Callback binding for the source-control snapshot used by F03R.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceControlSnapshotClock {
    callback_index: FrontendCallbackIndex,
    normal_callbacks: NormalCallbackCount,
    source_origin: FrontendCallbackIndex,
    cached_drain: bool,
}

impl SourceControlSnapshotClock {
    pub fn for_normal_callback(
        callback_index: FrontendCallbackIndex,
        normal_callbacks: NormalCallbackCount,
    ) -> Result<Self, SourceFlagsError> {
        if normal_callbacks.value() == 0
            || u64::from(callback_index.value()) >= normal_callbacks.value()
        {
            return Err(SourceFlagsError::SourceControlClock);
        }
        Ok(Self {
            callback_index,
            normal_callbacks,
            source_origin: callback_index,
            cached_drain: false,
        })
    }

    pub fn for_cached_drain(
        callback_index: FrontendCallbackIndex,
        normal_callbacks: NormalCallbackCount,
    ) -> Result<Self, SourceFlagsError> {
        if normal_callbacks.value() == 0
            || u64::from(callback_index.value()) < normal_callbacks.value()
        {
            return Err(SourceFlagsError::SourceControlClock);
        }
        let last_normal = normal_callbacks
            .value()
            .checked_sub(1)
            .and_then(|value| u32::try_from(value).ok())
            .map(FrontendCallbackIndex::new)
            .ok_or(SourceFlagsError::SourceControlClock)?;
        Ok(Self {
            callback_index,
            normal_callbacks,
            source_origin: last_normal,
            cached_drain: true,
        })
    }

    pub const fn callback_index(self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn normal_callbacks(self) -> NormalCallbackCount {
        self.normal_callbacks
    }

    pub const fn source_origin(self) -> FrontendCallbackIndex {
        self.source_origin
    }

    pub const fn is_cached_drain(self) -> bool {
        self.cached_drain
    }

    #[cfg(test)]
    pub(crate) const fn fixture_with_source_origin(
        callback_index: FrontendCallbackIndex,
        normal_callbacks: NormalCallbackCount,
        source_origin: FrontendCallbackIndex,
        cached_drain: bool,
    ) -> Self {
        Self {
            callback_index,
            normal_callbacks,
            source_origin,
            cached_drain,
        }
    }
}

/// Source-control projection from one validated physical metadata snapshot.
#[must_use]
pub struct PrimaryRegularSourceControlProjection {
    clock: SourceControlSnapshotClock,
    profile_identity: ResolvedEncodingProfile,
    route: ValidatedSemanticRoute,
    controls: RegularControls,
}

impl PrimaryRegularSourceControlProjection {
    pub const fn clock(&self) -> SourceControlSnapshotClock {
        self.clock
    }

    pub const fn profile_identity(&self) -> ResolvedEncodingProfile {
        self.profile_identity
    }

    pub const fn route(&self) -> &ValidatedSemanticRoute {
        &self.route
    }

    pub const fn controls(&self) -> &RegularControls {
        &self.controls
    }
}

/// Projects the approved bed/object F01E fields to the selected F03R regular domain.
pub fn project_primary_regular_source_controls(
    control_rows: &PrimaryMetadataControlRows,
    graph: &SelectedAdmGraph,
    profile_identity: ResolvedEncodingProfile,
    clock: SourceControlSnapshotClock,
) -> Result<PrimaryRegularSourceControlProjection, SourceFlagsError> {
    if profile_identity != control_rows.source_binding().selected_profile() {
        return Err(SourceFlagsError::UnsupportedProfile);
    }
    let metadata_hashes = control_rows.source_binding().metadata_hashes();
    if metadata_hashes.axml() != *graph.receipt().axml_sha256()
        || metadata_hashes.chna() != *graph.receipt().chna_sha256()
    {
        return Err(SourceFlagsError::PrimaryMetadataIdentityMismatch);
    }

    let direct_lfe_host = graph.receipt().direct_lfe_host().value();
    if usize::from(direct_lfe_host) >= HOST_CHANNELS || control_rows.row_count() != HOST_CHANNELS {
        return Err(SourceFlagsError::PrimaryControlRowMismatch);
    }
    let mut destination_by_host = [None; HOST_CHANNELS];
    for binding in graph.regular_source_bindings() {
        let host = usize::from(binding.physical_host().value());
        if host >= HOST_CHANNELS || host == usize::from(direct_lfe_host) {
            return Err(SourceFlagsError::PrimaryRegularSourceMapMismatch);
        }
        let destination = u8::try_from(binding.global_source().value())
            .map_err(|_| SourceFlagsError::PrimaryRegularSourceMapMismatch)?;
        if destination_by_host[host].replace(destination).is_some() {
            return Err(SourceFlagsError::PrimaryRegularSourceMapMismatch);
        }
    }

    let mut route_records = Vec::with_capacity(HOST_CHANNELS);
    let mut regular_controls = Vec::with_capacity(REGULAR_HOSTS);
    let mut direct_lfe_seen = false;
    for (row_index, row) in control_rows.rows().iter().enumerate() {
        let host = row.track().physical_host().value();
        if usize::from(host) != row_index
            || graph
                .tracks()
                .iter()
                .find(|track| track.physical_host().value() == host)
                != Some(row.track())
        {
            return Err(SourceFlagsError::PrimaryControlRowMismatch);
        }
        let is_direct_lfe = host == direct_lfe_host;
        match (row.track().kind(), row.fields()) {
            (
                SelectedTrackKind::DirectSpeakersBed { bed_id, .. },
                PrimaryControl38Fields::Bed { .. },
            ) if (bed_id == 3) == is_direct_lfe => {}
            (SelectedTrackKind::Object { .. }, PrimaryControl38Fields::Object { .. })
                if !is_direct_lfe => {}
            _ => return Err(SourceFlagsError::PrimaryControlRowMismatch),
        }

        if is_direct_lfe {
            if direct_lfe_seen {
                return Err(SourceFlagsError::PrimaryControlRowMismatch);
            }
            direct_lfe_seen = true;
            route_records.push(HostRouteRecord::new(host, HostRole::Lfe, true, None)?);
            continue;
        }

        let destination = destination_by_host[usize::from(host)]
            .ok_or(SourceFlagsError::PrimaryRegularSourceMapMismatch)?;
        route_records.push(HostRouteRecord::new(
            host,
            HostRole::Regular,
            true,
            Some(destination),
        )?);

        let (mute_word, width) = match row.fields() {
            PrimaryControl38Fields::Bed { words, .. } => (
                words[0],
                f32::from_bits(BED_SOURCE_WIDTH_POSITIVE_ZERO_BITS),
            ),
            PrimaryControl38Fields::Object { control52 } => {
                let words = control52.words();
                (words[0], f32::from_bits(words[9]))
            }
        };
        let mute = u8::try_from(mute_word).map_err(|_| SourceFlagsError::InvalidMute)?;
        let feature_control = SelectedFeatureFlagsControl::from_admitted_profile(
            profile_identity,
            clock.callback_index(),
            host,
        )?;
        regular_controls.push(feature_control.into_regular_control(mute, width)?);
    }

    if !direct_lfe_seen || regular_controls.len() != REGULAR_HOSTS {
        return Err(SourceFlagsError::PrimaryControlRowMismatch);
    }
    let route = ValidatedSemanticRoute::new(route_records)?;
    let controls = RegularControls::new(regular_controls)?;
    Ok(PrimaryRegularSourceControlProjection {
        clock,
        profile_identity,
        route,
        controls,
    })
}

/// Projects the current normal snapshot's tagged controls into the F03R regular-host domain.
/// The proposal already carries the admitted graph, physical map, DBMD identity and source
/// origin; `graph` is checked against that prepared identity before any projection is returned.
pub fn project_normal_control_snapshot_source_controls(
    proposal: &NormalControlSnapshotProposal,
    graph: &SelectedAdmGraph,
    profile_identity: ResolvedEncodingProfile,
) -> Result<PrimaryRegularSourceControlProjection, SourceFlagsError> {
    let prepared = proposal.after_state().prepared_identity().as_ref();
    if !prepared.matches_graph(graph) {
        return Err(SourceFlagsError::PrimaryMetadataIdentityMismatch);
    }
    project_normal_control_snapshot_source_controls_from_prepared(
        proposal,
        prepared,
        profile_identity,
    )
}

/// Core projection for the exact prepared map, also usable by frontend unit tests.
pub(crate) fn project_normal_control_snapshot_source_controls_from_prepared(
    proposal: &NormalControlSnapshotProposal,
    prepared: &NormalControlPreparedSnapshot,
    profile_identity: ResolvedEncodingProfile,
) -> Result<PrimaryRegularSourceControlProjection, SourceFlagsError> {
    if proposal.after_state().prepared_identity().as_ref() != prepared
        || prepared.profile_identity() != profile_identity
    {
        return Err(SourceFlagsError::UnsupportedProfile);
    }

    let callback_index = proposal.transaction_id().callback_index();
    let callback_count = proposal.after_state().next_normal_callbacks();
    let expected_next = callback_index
        .value()
        .checked_add(1)
        .ok_or(SourceFlagsError::SourceControlClockMismatch)?;
    let query_sample = u64::from(callback_index.value())
        .checked_mul(
            u64::try_from(FRAME_SAMPLES)
                .map_err(|_| SourceFlagsError::SourceControlClockMismatch)?,
        )
        .ok_or(SourceFlagsError::SourceControlClockMismatch)?;
    let expected_actual = prepared.source_samples().saturating_sub(query_sample);
    if proposal.value_generation() != callback_index
        || proposal.normal_origin_generation() != callback_index
        || proposal.query_sample() != query_sample
        || query_sample >= prepared.source_samples()
        || u64::from(proposal.actual_samples()) != expected_actual.min(FRAME_SAMPLES as u64)
        || callback_count.value() != u64::from(expected_next)
        || proposal.after_state().next_callback_index().value() != expected_next
    {
        return Err(SourceFlagsError::SourceControlClockMismatch);
    }
    let clock = SourceControlSnapshotClock::for_normal_callback(callback_index, callback_count)?;
    if clock.is_cached_drain() || clock.source_origin() != proposal.normal_origin_generation() {
        return Err(SourceFlagsError::SourceControlClockMismatch);
    }

    project_normal_control_rows(
        proposal.control38_rows(),
        proposal.regular37_rows(),
        prepared,
        profile_identity,
        clock,
    )
}

/// Projects the committed last-normal F01E values for a main cached-audio delivery g.
/// The control rows remain sourced from Q-1; only the projection delivery clock advances.
pub fn project_held_normal_control_snapshot_source_controls(
    held: &NormalControlHeldSourceView,
    graph: &SelectedAdmGraph,
    profile_identity: ResolvedEncodingProfile,
) -> Result<PrimaryRegularSourceControlProjection, SourceFlagsError> {
    let prepared = held.prepared_identity();
    if !prepared.matches_graph(graph) {
        return Err(SourceFlagsError::PrimaryMetadataIdentityMismatch);
    }
    if prepared.profile_identity() != profile_identity {
        return Err(SourceFlagsError::UnsupportedProfile);
    }
    let clock = SourceControlSnapshotClock::for_cached_drain(
        held.delivery_generation(),
        held.normal_callbacks(),
    )?;
    if clock.source_origin() != held.source_origin_generation()
        || held.delivery_transaction_id().programme_id() != prepared.programme_id()
        || held.delivery_transaction_id().attempt_id() != prepared.attempt_id()
        || held.delivery_transaction_id().callback_index() != clock.callback_index()
    {
        return Err(SourceFlagsError::SourceControlClockMismatch);
    }
    project_normal_control_rows(
        held.control38_rows(),
        held.regular37_rows(),
        prepared,
        profile_identity,
        clock,
    )
}

fn project_normal_control_rows(
    control38_rows: &[NormalControlRow; HOST_CHANNELS],
    regular37_rows: &[NormalControlRow; REGULAR_HOSTS],
    prepared: &NormalControlPreparedSnapshot,
    profile_identity: ResolvedEncodingProfile,
    clock: SourceControlSnapshotClock,
) -> Result<PrimaryRegularSourceControlProjection, SourceFlagsError> {
    if control38_rows.len() != HOST_CHANNELS
        || regular37_rows.len() != REGULAR_HOSTS
        || prepared.physical_tracks().len() != HOST_CHANNELS
    {
        return Err(SourceFlagsError::PrimaryControlRowMismatch);
    }
    let callback_index = clock.callback_index();
    let direct_lfe_host = prepared.lfe_host();
    if usize::from(direct_lfe_host) >= HOST_CHANNELS {
        return Err(SourceFlagsError::PrimaryControlRowMismatch);
    }

    let mut destination_by_host = [None; HOST_CHANNELS];
    for binding in prepared.regular_source_bindings() {
        let host = usize::from(binding.physical_host());
        if host >= HOST_CHANNELS || host == usize::from(direct_lfe_host) {
            return Err(SourceFlagsError::PrimaryRegularSourceMapMismatch);
        }
        let destination = u8::try_from(binding.global_source())
            .map_err(|_| SourceFlagsError::PrimaryRegularSourceMapMismatch)?;
        let is_selected_global_source = (11..=47).contains(&destination);
        if !is_selected_global_source || destination_by_host[host].replace(destination).is_some() {
            return Err(SourceFlagsError::PrimaryRegularSourceMapMismatch);
        }
    }

    let mut route_records = Vec::with_capacity(HOST_CHANNELS);
    let mut direct_lfe_seen = false;
    for (host_index, row) in control38_rows.iter().enumerate() {
        let host = row.physical_host();
        let track = &prepared.physical_tracks()[host_index];
        if usize::from(host) != host_index
            || track.physical_host() != host
            || track.track_uid() != row.track_uid()
            || track.channel_format_id() != row.channel_format_id()
            || row.dbmd().track_index() != u16::from(host)
        {
            return Err(SourceFlagsError::PrimaryControlRowMismatch);
        }
        match (host == direct_lfe_host, row, track.kind()) {
            (
                true,
                NormalControlRow::Bed6 {
                    words,
                    unproduced_tail,
                    ..
                },
                NormalControlTrackKind::Bed {
                    bed_id: 3,
                    semantic_role,
                    ..
                },
            ) if semantic_role == "lfe"
                && *words == [0, 0, 3, 2, 1, 3]
                && unproduced_tail.offset_bytes() == 24
                && unproduced_tail.byte_count() == 28 =>
            {
                if direct_lfe_seen {
                    return Err(SourceFlagsError::PrimaryControlRowMismatch);
                }
                direct_lfe_seen = true;
                route_records.push(HostRouteRecord::new(host, HostRole::Lfe, true, None)?);
            }
            (
                false,
                NormalControlRow::Bed6 { words, .. },
                NormalControlTrackKind::Bed {
                    bed_id,
                    semantic_role,
                    ..
                },
            ) if *bed_id != 3
                && semantic_role != "lfe"
                && words[0..5] == [0, 0, 3, 2, 1]
                && words[5] == u32::from(*bed_id) =>
            {
                route_records.push(HostRouteRecord::new(
                    host,
                    HostRole::Regular,
                    true,
                    Some(
                        destination_by_host[host_index]
                            .ok_or(SourceFlagsError::PrimaryRegularSourceMapMismatch)?,
                    ),
                )?);
            }
            (
                false,
                NormalControlRow::Object13 {
                    semantic_key,
                    local_owner_ordinal,
                    control,
                    ..
                },
                NormalControlTrackKind::Object {
                    semantic_key: track_key,
                    local_owner_ordinal: track_ordinal,
                },
            ) if semantic_key == track_key
                && local_owner_ordinal == track_ordinal
                && control.words()[11] == 0
                && control.words()[12] == 1 =>
            {
                let binding = prepared
                    .object_owner_bindings()
                    .iter()
                    .find(|binding| binding.track_uid() == row.track_uid())
                    .ok_or(SourceFlagsError::PrimaryControlRowMismatch)?;
                if binding.local_owner_ordinal() != *local_owner_ordinal
                    || binding.physical_host() != host
                    || binding.semantic_key() != *semantic_key
                    || binding.channel_format_id() != row.channel_format_id()
                {
                    return Err(SourceFlagsError::PrimaryControlRowMismatch);
                }
                route_records.push(HostRouteRecord::new(
                    host,
                    HostRole::Regular,
                    true,
                    Some(
                        destination_by_host[host_index]
                            .ok_or(SourceFlagsError::PrimaryRegularSourceMapMismatch)?,
                    ),
                )?);
            }
            _ => return Err(SourceFlagsError::PrimaryControlRowMismatch),
        }
    }
    if !direct_lfe_seen {
        return Err(SourceFlagsError::PrimaryControlRowMismatch);
    }

    let mut regular_controls = Vec::with_capacity(REGULAR_HOSTS);
    for (rank, (row, expected_host)) in regular37_rows
        .iter()
        .zip(prepared.regular37_hosts())
        .enumerate()
    {
        let host = row.physical_host();
        if host != *expected_host
            || host == direct_lfe_host
            || control38_rows[usize::from(host)] != *row
        {
            return Err(SourceFlagsError::RegularControlHostMismatch);
        }
        let binding = prepared.regular_source_bindings()[rank];
        if binding.physical_host() != host
            || destination_by_host[usize::from(host)]
                != Some(
                    u8::try_from(binding.global_source())
                        .map_err(|_| SourceFlagsError::PrimaryRegularSourceMapMismatch)?,
                )
        {
            return Err(SourceFlagsError::PrimaryRegularSourceMapMismatch);
        }
        let source_view = row.source_flags_view();
        let mute =
            u8::try_from(source_view.mute_word()).map_err(|_| SourceFlagsError::InvalidMute)?;
        let feature_control = SelectedFeatureFlagsControl::from_admitted_profile(
            profile_identity,
            callback_index,
            host,
        )?;
        regular_controls.push(
            feature_control
                .into_regular_control(mute, f32::from_bits(source_view.spread_word()))?,
        );
    }

    let route = ValidatedSemanticRoute::new(route_records)?;
    let controls = RegularControls::new(regular_controls)?;
    Ok(PrimaryRegularSourceControlProjection {
        clock,
        profile_identity,
        route,
        controls,
    })
}

/// Exact finite host-major PCM input; shape and finiteness are validated before retention.
#[derive(Debug)]
pub struct CurrentHostPcm {
    rows: Box<[FiniteFrame]>,
    receipt_sha256: ProviderSha256,
}

impl CurrentHostPcm {
    pub fn new(rows: Vec<Vec<f32>>) -> Result<Self, SourceFlagsError> {
        if rows.len() != HOST_CHANNELS {
            return Err(SourceFlagsError::PcmShape);
        }
        if rows.iter().any(|row| row.len() != FRAME_SAMPLES) {
            return Err(SourceFlagsError::PcmShape);
        }
        let rows = rows
            .into_iter()
            .map(finite_frame)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        let mut canonical = Vec::with_capacity(HOST_CHANNELS * FRAME_SAMPLES * 4);
        for row in &rows {
            for value in row.iter() {
                canonical.extend_from_slice(&value.to_bits().to_le_bytes());
            }
        }
        Ok(Self {
            rows,
            receipt_sha256: ProviderSha256::digest(&canonical),
        })
    }

    fn row(&self, host: HostId) -> &FiniteFrame {
        &self.rows[usize::from(host.value())]
    }

    #[cfg(test)]
    fn row_bits(&self, host: u8) -> Vec<u32> {
        self.rows[usize::from(host)]
            .iter()
            .map(|sample| sample.to_bits())
            .collect()
    }
}

/// Exact validated regular-host history owned by the enclosing spatial-control root.
#[derive(Debug)]
pub struct BeforeRegularHistory {
    owner_generation: SourceFlagsHistoryGeneration,
    owner_digest: SpatialControlDigest,
    profile_identity: EncodingProfile,
    provider_lineage: SpatialControlProviderLineage,
    regular_hosts: Box<[HostId]>,
    rows: Box<[FiniteFrame]>,
}

impl BeforeRegularHistory {
    pub fn from_owner(
        owner: &SpatialControlMachine,
        route: &ValidatedSemanticRoute,
    ) -> Result<Self, SourceFlagsError> {
        let bits = owner.committed().regular_raw_history_bits();
        if bits.len() != REGULAR_HOSTS * FRAME_SAMPLES {
            return Err(SourceFlagsError::HistoryShape);
        }
        let rows = bits
            .chunks_exact(FRAME_SAMPLES)
            .map(|row| row.iter().map(|word| f32::from_bits(*word)).collect())
            .collect();
        let provider_lineage = owner.provider_lineage();
        Self::from_rows_with_receipt(
            SourceFlagsHistoryGeneration::new(owner.generation()),
            owner.digest(),
            owner.profile(),
            &provider_lineage,
            route,
            rows,
        )
    }

    fn from_rows_with_receipt(
        owner_generation: SourceFlagsHistoryGeneration,
        owner_digest: SpatialControlDigest,
        profile_identity: EncodingProfile,
        provider_lineage: &SpatialControlProviderLineage,
        route: &ValidatedSemanticRoute,
        rows: Vec<Vec<f32>>,
    ) -> Result<Self, SourceFlagsError> {
        if rows.len() != REGULAR_HOSTS {
            return Err(SourceFlagsError::HistoryShape);
        }
        if rows.iter().any(|row| row.len() != FRAME_SAMPLES) {
            return Err(SourceFlagsError::HistoryShape);
        }
        let rows = rows
            .into_iter()
            .map(finite_frame)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        Ok(Self {
            owner_generation,
            owner_digest,
            profile_identity,
            provider_lineage: *provider_lineage,
            regular_hosts: route.regular_hosts().to_vec().into_boxed_slice(),
            rows,
        })
    }
}

#[derive(Debug)]
pub struct PrefixWriterSet {
    written: [bool; PREFIX_SOURCE_COUNT],
    receipt_sha256: ProviderSha256,
}

impl PrefixWriterSet {
    pub fn new(sources: Vec<u8>) -> Result<Self, SourceFlagsError> {
        let mut written = [false; PREFIX_SOURCE_COUNT];
        for source in sources {
            let slot = usize::from(source);
            if slot >= PREFIX_SOURCE_COUNT {
                return Err(SourceFlagsError::PrefixWriterDomain);
            }
            written[slot] = true;
        }
        let canonical: Vec<u8> = written
            .iter()
            .enumerate()
            .filter_map(|(source, is_written)| is_written.then_some(source as u8))
            .collect();
        Ok(Self {
            written,
            receipt_sha256: ProviderSha256::digest(&canonical),
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SourceFlagsParentLineage {
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    transaction_id: CallbackLineageIdentity,
    callback_index: FrontendCallbackIndex,
    before_history_generation: SourceFlagsHistoryGeneration,
    before_history_digest: SpatialControlDigest,
    profile_identity: EncodingProfile,
    provider_lineage: SpatialControlProviderLineage,
}

impl SourceFlagsParentLineage {
    pub fn new(
        transaction_id: CallbackLineageIdentity,
        owner: &SpatialControlMachine,
    ) -> Result<Self, SourceFlagsError> {
        let programme_id = transaction_id.programme_id().clone();
        let attempt = transaction_id.attempt_id();
        let callback_index = transaction_id.callback_index();
        if callback_index != owner.committed().callback_index() {
            return Err(SourceFlagsError::StaleCallbackGeneration);
        }
        let profile_identity = owner.profile();
        let provider_lineage = owner.provider_lineage();
        resolve_selected_profile(&profile_identity)
            .map_err(|_| SourceFlagsError::UnsupportedProfile)?;
        if provider_lineage.f01e_provider_generation() != 0
            || provider_lineage.f03_provider_generation() != 0
        {
            return Err(SourceFlagsError::ProviderLineageMismatch);
        }
        Ok(Self {
            programme_id,
            attempt,
            transaction_id,
            callback_index,
            before_history_generation: SourceFlagsHistoryGeneration::new(owner.generation()),
            before_history_digest: owner.digest(),
            profile_identity,
            provider_lineage,
        })
    }
}

#[derive(Debug)]
pub struct SourceFlagsInput {
    current_pcm: CurrentHostPcm,
    before_history: Option<BeforeRegularHistory>,
    controls: RegularControls,
    route: ValidatedSemanticRoute,
    prefix_writers: PrefixWriterSet,
    source_control_clock: Option<SourceControlSnapshotClock>,
    parent: Arc<SourceFlagsParentLineage>,
}

impl SourceFlagsInput {
    pub fn new(
        current_pcm: CurrentHostPcm,
        before_history: Option<BeforeRegularHistory>,
        controls: RegularControls,
        route: ValidatedSemanticRoute,
        prefix_writers: PrefixWriterSet,
        parent: SourceFlagsParentLineage,
    ) -> Self {
        Self {
            current_pcm,
            before_history,
            controls,
            route,
            prefix_writers,
            source_control_clock: None,
            parent: Arc::new(parent),
        }
    }

    pub fn from_primary_regular_source_controls(
        current_pcm: CurrentHostPcm,
        before_history: Option<BeforeRegularHistory>,
        projection: PrimaryRegularSourceControlProjection,
        prefix_writers: PrefixWriterSet,
        parent: SourceFlagsParentLineage,
    ) -> Result<Self, SourceFlagsError> {
        let profile = resolve_selected_profile(&parent.profile_identity)
            .map_err(|_| SourceFlagsError::UnsupportedProfile)?;
        if projection.profile_identity != profile {
            return Err(SourceFlagsError::UnsupportedProfile);
        }
        if projection.clock.callback_index() != parent.callback_index {
            return Err(SourceFlagsError::StaleCallbackGeneration);
        }
        Ok(Self {
            current_pcm,
            before_history,
            controls: projection.controls,
            route: projection.route,
            prefix_writers,
            source_control_clock: Some(projection.clock),
            parent: Arc::new(parent),
        })
    }

    pub(super) const fn source_control_clock(&self) -> Option<SourceControlSnapshotClock> {
        self.source_control_clock
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SourceFlagsDigest([u8; 32]);

impl SourceFlagsDigest {
    fn from_parts(domain: &[u8], parent: &SourceFlagsParentLineage, payload: &[u8]) -> Self {
        let programme_id_bytes = parent.programme_id.as_str().as_bytes();
        let mut bytes = Vec::with_capacity(
            domain.len() + 8 + programme_id_bytes.len() + 8 + 4 + 8 + payload.len(),
        );
        bytes.extend_from_slice(domain);
        bytes.extend_from_slice(
            &u64::try_from(programme_id_bytes.len())
                .expect("programme identity length fits U64")
                .to_le_bytes(),
        );
        bytes.extend_from_slice(programme_id_bytes);
        bytes.extend_from_slice(&parent.attempt.value().to_le_bytes());
        bytes.extend_from_slice(&parent.callback_index.value().to_le_bytes());
        bytes.extend_from_slice(&parent.before_history_generation.value().to_le_bytes());
        bytes.extend_from_slice(payload);
        let digest = Sha256::digest(bytes);
        let mut value = [0_u8; 32];
        value.copy_from_slice(&digest);
        Self(value)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct SourceFlagsLineage {
    parent: Arc<SourceFlagsParentLineage>,
    current_pcm_receipt: SourceFlagsDigest,
    controls_receipt: SourceFlagsDigest,
    source_control_clock: Option<SourceControlSnapshotClock>,
    route_receipt: SourceFlagsDigest,
    writer_receipt: SourceFlagsDigest,
}

impl SourceFlagsLineage {
    pub(super) fn programme_id(&self) -> &SourceFlagsProgrammeId {
        &self.parent.programme_id
    }

    pub(super) fn attempt(&self) -> SourceFlagsAttempt {
        self.parent.attempt
    }

    pub(super) fn transaction_id(&self) -> &CallbackLineageIdentity {
        &self.parent.transaction_id
    }

    pub(crate) fn callback_index(&self) -> FrontendCallbackIndex {
        self.parent.callback_index
    }

    pub(crate) fn before_history_generation(&self) -> SourceFlagsHistoryGeneration {
        self.parent.before_history_generation
    }

    pub(crate) fn before_history_digest(&self) -> SpatialControlDigest {
        self.parent.before_history_digest
    }

    pub(crate) fn profile_identity(&self) -> EncodingProfile {
        self.parent.profile_identity
    }

    pub(crate) fn provider_lineage(&self) -> SpatialControlProviderLineage {
        self.parent.provider_lineage
    }
}

/// Raw-current-history proposal for the existing owner; it never mutates committed history.
#[derive(Debug)]
pub struct RawHistoryProposal {
    rows: Box<[RawFrame]>,
    lineage: Arc<SourceFlagsLineage>,
}

impl RawHistoryProposal {
    pub fn rows(&self) -> &[Box<RawHistoryFrame>] {
        &self.rows
    }

    fn lineage(&self) -> &SourceFlagsLineage {
        &self.lineage
    }
}

pub struct SourceFlagsProduced {
    pub(crate) packet: SourceFlagsPacket,
    pub(crate) raw_history_proposal: RawHistoryProposal,
}

/// Lineage-free selected F03R values for an owner that supplies its own meter state.
/// This value carries no parent receipt, packet capability or commit authority.
#[derive(Debug)]
pub struct SourceFlagsValues {
    payload: [[u8; 2]; SOURCE_COUNT],
    raw_history_rows: Box<[RawFrame]>,
}

impl SourceFlagsValues {
    pub const fn payload(&self) -> &[[u8; 2]; SOURCE_COUNT] {
        &self.payload
    }

    pub fn raw_history_rows(&self) -> impl ExactSizeIterator<Item = &RawHistoryFrame> + '_ {
        self.raw_history_rows.iter().map(Box::as_ref)
    }
}

impl SourceFlagsProduced {
    pub fn into_parts(self) -> Result<(SourceFlagsPacket, RawHistoryProposal), SourceFlagsError> {
        self.packet
            .validate_lineage(self.raw_history_proposal.lineage())?;
        Ok((self.packet, self.raw_history_proposal))
    }
}

/// Opaque linear packet capability produced only by this module.
#[must_use]
pub struct SourceFlagsPacket {
    payload: Box<[[u8; 2]; SOURCE_COUNT]>,
    lineage: Arc<SourceFlagsLineage>,
}

impl SourceFlagsPacket {
    fn from_producer(payload: [[u8; 2]; SOURCE_COUNT], lineage: Arc<SourceFlagsLineage>) -> Self {
        Self {
            payload: Box::new(payload),
            lineage,
        }
    }

    pub(super) fn payload(&self) -> &[[u8; 2]; SOURCE_COUNT] {
        &self.payload
    }

    pub(super) fn lineage(&self) -> &SourceFlagsLineage {
        self.lineage.as_ref()
    }

    pub(super) fn validate_lineage(
        &self,
        candidate: &SourceFlagsLineage,
    ) -> Result<(), SourceFlagsError> {
        if self.lineage().callback_index() != candidate.callback_index() {
            return Err(SourceFlagsError::StaleCallbackGeneration);
        }
        if self.lineage.as_ref() == candidate {
            Ok(())
        } else {
            Err(SourceFlagsError::LineageMismatch)
        }
    }

    pub(super) fn validate_payload(&self) -> Result<(), SourceFlagsError> {
        if self
            .payload()
            .iter()
            .any(|pair| pair == &[0, 1] || pair.iter().any(|flag| *flag > 1))
        {
            Err(SourceFlagsError::InvalidFlagPair)
        } else {
            Ok(())
        }
    }

    pub(super) fn validate_parent_binding(
        &self,
        candidate: &SourceFlagsParentLineage,
    ) -> Result<(), SourceFlagsError> {
        let lineage = self.lineage();
        if lineage.callback_index() != candidate.callback_index {
            return Err(SourceFlagsError::StaleCallbackGeneration);
        }
        if lineage.parent.as_ref() != candidate
            || lineage.before_history_generation() != candidate.before_history_generation
            || lineage.before_history_digest() != candidate.before_history_digest
            || lineage.profile_identity() != candidate.profile_identity
            || lineage.provider_lineage() != candidate.provider_lineage
        {
            return Err(SourceFlagsError::LineageMismatch);
        }
        Ok(())
    }
}

/// Pure source-major flags and exact raw-history proposal production.
pub struct SourceFlagsProducer;

impl SourceFlagsProducer {
    #[allow(clippy::needless_pass_by_value)]
    pub fn produce(input: SourceFlagsInput) -> Result<SourceFlagsProduced, SourceFlagsError> {
        let history = input
            .before_history
            .as_ref()
            .ok_or(SourceFlagsError::MissingBeforeHistory)?;
        preflight(&input, history)?;
        let values = compute_source_flags_values(
            &input.current_pcm,
            &history.rows,
            &input.controls,
            &input.route,
            &input.prefix_writers,
        )?;
        let SourceFlagsValues {
            payload,
            raw_history_rows,
        } = values;
        let lineage = Arc::new(make_lineage(
            Arc::clone(&input.parent),
            &input.current_pcm,
            &input.controls,
            &input.route,
            &input.prefix_writers,
            input.source_control_clock,
        )?);
        let packet = SourceFlagsPacket::from_producer(payload, Arc::clone(&lineage));
        packet.validate_payload()?;
        packet.validate_parent_binding(&input.parent)?;
        let raw_history_proposal = RawHistoryProposal {
            rows: raw_history_rows,
            lineage,
        };
        Ok(SourceFlagsProduced {
            packet,
            raw_history_proposal,
        })
    }

    /// Runs the same selected F03R value transition for an independently owned meter state.
    pub fn produce_meter_values(
        current_pcm_rows: &[Vec<u32>],
        previous_regular_rows: &[Vec<u32>],
        controls: &RegularControls,
        route: &ValidatedSemanticRoute,
        prefix_writer_sources: &[u8],
    ) -> Result<SourceFlagsValues, SourceFlagsError> {
        let current_rows = current_pcm_rows
            .iter()
            .map(|row| row.iter().copied().map(f32::from_bits).collect())
            .collect();
        let current_pcm = CurrentHostPcm::new(current_rows)?;
        let history_rows = previous_regular_rows
            .iter()
            .map(|row| row.iter().copied().map(f32::from_bits).collect::<Vec<_>>())
            .map(finite_frame)
            .collect::<Result<Vec<_>, _>>()?;
        let prefix_writers = PrefixWriterSet::new(prefix_writer_sources.to_vec())?;
        compute_source_flags_values(
            &current_pcm,
            &history_rows,
            controls,
            route,
            &prefix_writers,
        )
    }

    #[cfg(test)]
    pub(crate) fn window_sha256() -> Result<String, SourceFlagsError> {
        let window = triangle_window()?;
        let mut bytes = Vec::with_capacity(WINDOW_WORDS * 4);
        for value in &window {
            bytes.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        Ok(hex_encode(&Sha256::digest(bytes)))
    }
}

fn compute_source_flags_values(
    current_pcm: &CurrentHostPcm,
    history_rows: &[FiniteFrame],
    controls: &RegularControls,
    route: &ValidatedSemanticRoute,
    prefix_writers: &PrefixWriterSet,
) -> Result<SourceFlagsValues, SourceFlagsError> {
    preflight_value_shapes(current_pcm, history_rows, controls, route)?;
    let window = triangle_window()?;
    let mut payload = [[0_u8; 2]; SOURCE_COUNT];

    for (source, pair) in payload[..PREFIX_SOURCE_COUNT].iter_mut().enumerate() {
        *pair = if prefix_writers.written[source] {
            [0, 0]
        } else {
            [1, 1]
        };
    }

    let width_limit = f32::from_bits(WIDTH_LIMIT_BITS);
    let mut history_proposal = Vec::with_capacity(REGULAR_HOSTS);
    for (row, host) in route.regular_hosts.iter().copied().enumerate() {
        let current = current_pcm.row(host);
        let before = &history_rows[row];
        let control = controls.for_host(host)?;
        let witness = triangle_activity_witness(&before[..], &current[..], &window)?;

        let activity = witness && !control.muted;
        let processing_inactive = !activity;
        let selection_suppressed = control.muted;
        let pair = if activity && control.width.value() >= width_limit {
            [1, 1]
        } else {
            [
                u8::from(processing_inactive),
                u8::from(selection_suppressed),
            ]
        };
        if pair == [0, 1] || pair.iter().any(|flag| *flag > 1) {
            return Err(SourceFlagsError::InvalidFlagPair);
        }
        payload[usize::from(route.destination(host)?)] = pair;
        history_proposal.push(raw_frame(current));
    }
    Ok(SourceFlagsValues {
        payload,
        raw_history_rows: history_proposal.into_boxed_slice(),
    })
}

fn preflight(
    input: &SourceFlagsInput,
    history: &BeforeRegularHistory,
) -> Result<(), SourceFlagsError> {
    preflight_value_shapes(
        &input.current_pcm,
        &history.rows,
        &input.controls,
        &input.route,
    )?;
    if history.owner_generation != input.parent.before_history_generation {
        return Err(SourceFlagsError::WrongHistoryGeneration);
    }
    if history.owner_digest != input.parent.before_history_digest {
        return Err(SourceFlagsError::HistoryLineageMismatch);
    }
    if history.profile_identity != input.parent.profile_identity
        || history.provider_lineage != input.parent.provider_lineage
    {
        return Err(SourceFlagsError::HistoryLineageMismatch);
    }
    if history.regular_hosts.as_ref() != input.route.regular_hosts.as_ref() {
        return Err(SourceFlagsError::RegularControlHostMismatch);
    }
    if input.controls.values.len() != REGULAR_HOSTS {
        return Err(SourceFlagsError::RegularControlCount);
    }
    resolve_selected_profile(&input.parent.profile_identity)
        .map_err(|_| SourceFlagsError::UnsupportedProfile)?;
    if input.parent.provider_lineage.f01e_provider_generation() != 0
        || input.parent.provider_lineage.f03_provider_generation() != 0
    {
        return Err(SourceFlagsError::ProviderLineageMismatch);
    }
    Ok(())
}

fn preflight_value_shapes(
    current_pcm: &CurrentHostPcm,
    history_rows: &[FiniteFrame],
    controls: &RegularControls,
    route: &ValidatedSemanticRoute,
) -> Result<(), SourceFlagsError> {
    if current_pcm.rows.len() != HOST_CHANNELS
        || current_pcm
            .rows
            .iter()
            .any(|row| row.len() != FRAME_SAMPLES)
    {
        return Err(SourceFlagsError::PcmShape);
    }
    if history_rows.len() != REGULAR_HOSTS
        || history_rows.iter().any(|row| row.len() != FRAME_SAMPLES)
    {
        return Err(SourceFlagsError::HistoryShape);
    }
    if controls.values.len() != REGULAR_HOSTS {
        return Err(SourceFlagsError::RegularControlCount);
    }
    for host in route.regular_hosts.iter().copied() {
        controls.for_host(host)?;
        route.destination(host)?;
    }
    if route.regular_hosts.len() != REGULAR_HOSTS
        || route.destination_by_host.len() != HOST_CHANNELS
        || route.destination(route.lfe_host).is_ok()
    {
        return Err(SourceFlagsError::RouteNotBijective);
    }
    Ok(())
}

pub(crate) fn triangle_window() -> Result<Box<[FiniteF32]>, SourceFlagsError> {
    let divisor =
        FiniteF32::new(1536.0).map_err(|_| SourceFlagsError::WindowKnownAnswerMismatch)?;
    let mut window = Vec::with_capacity(WINDOW_WORDS);
    for index in 0..FRAME_SAMPLES {
        let numerator = FiniteF32::new(index as f32)
            .map_err(|_| SourceFlagsError::WindowKnownAnswerMismatch)?;
        window.push(
            FiniteF32::new(numerator.value() / divisor.value())
                .map_err(|_| SourceFlagsError::WindowKnownAnswerMismatch)?,
        );
    }
    for index in 0..FRAME_SAMPLES {
        let numerator = FiniteF32::new((FRAME_SAMPLES - 1 - index) as f32)
            .map_err(|_| SourceFlagsError::WindowKnownAnswerMismatch)?;
        window.push(
            FiniteF32::new(numerator.value() / divisor.value())
                .map_err(|_| SourceFlagsError::WindowKnownAnswerMismatch)?,
        );
    }
    let mut bytes = Vec::with_capacity(WINDOW_WORDS * 4);
    for value in &window {
        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    if hex_sha256(&bytes) != WINDOW_SHA256 {
        return Err(SourceFlagsError::WindowKnownAnswerMismatch);
    }
    Ok(window.into_boxed_slice())
}

pub(crate) fn triangle_activity_witness(
    before: &[FiniteF32],
    current: &[FiniteF32],
    window: &[FiniteF32],
) -> Result<bool, SourceFlagsError> {
    if before.len() != FRAME_SAMPLES || current.len() != FRAME_SAMPLES {
        return Err(SourceFlagsError::HistoryShape);
    }
    if window.len() != WINDOW_WORDS {
        return Err(SourceFlagsError::WindowKnownAnswerMismatch);
    }
    let epsilon = f32::from_bits(ACTIVITY_EPSILON_BITS);
    for sample_index in 0..FRAME_SAMPLES {
        let previous_product =
            r32_mul(before[sample_index], window[sample_index]).map_err(map_product_error)?;
        let current_product = r32_mul(current[sample_index], window[FRAME_SAMPLES + sample_index])
            .map_err(map_product_error)?;
        if previous_product.value().abs() > epsilon || current_product.value().abs() > epsilon {
            return Ok(true);
        }
    }
    Ok(false)
}

fn make_lineage(
    parent: Arc<SourceFlagsParentLineage>,
    current_pcm: &CurrentHostPcm,
    controls: &RegularControls,
    route: &ValidatedSemanticRoute,
    prefix_writers: &PrefixWriterSet,
    source_control_clock: Option<SourceControlSnapshotClock>,
) -> Result<SourceFlagsLineage, SourceFlagsError> {
    let current_pcm_receipt = digest_current_pcm(current_pcm, &parent);
    let controls_receipt = digest_controls(controls, route, &parent, source_control_clock)?;
    let route_identity = route.receipt_sha256.to_hex();
    let route_receipt =
        SourceFlagsDigest::from_parts(b"source-flags-route-v1", &parent, route_identity.as_bytes());
    let writer_identity = prefix_writers.receipt_sha256.to_hex();
    let writer_receipt = SourceFlagsDigest::from_parts(
        b"source-flags-prefix-writers-v1",
        &parent,
        writer_identity.as_bytes(),
    );
    Ok(SourceFlagsLineage {
        parent,
        current_pcm_receipt,
        controls_receipt,
        source_control_clock,
        route_receipt,
        writer_receipt,
    })
}

fn digest_current_pcm(
    pcm: &CurrentHostPcm,
    parent: &SourceFlagsParentLineage,
) -> SourceFlagsDigest {
    let identity = pcm.receipt_sha256.to_hex();
    SourceFlagsDigest::from_parts(b"source-flags-current-pcm-v1", parent, identity.as_bytes())
}

fn digest_controls(
    controls: &RegularControls,
    route: &ValidatedSemanticRoute,
    parent: &SourceFlagsParentLineage,
    source_control_clock: Option<SourceControlSnapshotClock>,
) -> Result<SourceFlagsDigest, SourceFlagsError> {
    let mut bytes = Vec::with_capacity(REGULAR_HOSTS * 6);
    for host in route.regular_hosts.iter().copied() {
        let control = controls.for_host(host)?;
        bytes.push(host.value());
        bytes.push(u8::from(control.muted));
        bytes.extend_from_slice(&control.width.to_bits().to_le_bytes());
        bytes.extend_from_slice(&[0, 0]);
    }
    if let Some(clock) = source_control_clock {
        bytes.extend_from_slice(b"source-control-clock-v1");
        bytes.push(u8::from(clock.is_cached_drain()));
        bytes.extend_from_slice(&clock.callback_index().value().to_le_bytes());
        bytes.extend_from_slice(&clock.normal_callbacks().value().to_le_bytes());
        bytes.extend_from_slice(&clock.source_origin().value().to_le_bytes());
    }
    Ok(SourceFlagsDigest::from_parts(
        b"source-flags-controls-v1",
        parent,
        &bytes,
    ))
}

fn raw_frame(frame: &FiniteFrame) -> RawFrame {
    Box::new(std::array::from_fn(|index| frame[index].to_bits()))
}

fn finite_frame(row: Vec<f32>) -> Result<FiniteFrame, SourceFlagsError> {
    if row.len() != FRAME_SAMPLES {
        return Err(SourceFlagsError::PcmShape);
    }
    let values = row
        .into_iter()
        .map(|value| FiniteF32::new(value).map_err(|_| SourceFlagsError::NonfinitePcm))
        .collect::<Result<Vec<_>, _>>()?;
    let boxed = values.into_boxed_slice();
    boxed.try_into().map_err(|_| SourceFlagsError::PcmShape)
}

fn map_product_error(_: R32Error) -> SourceFlagsError {
    SourceFlagsError::NonfiniteProduct
}

fn hex_sha256(bytes: &[u8]) -> String {
    hex_encode(&Sha256::digest(bytes))
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let value = usize::from(*byte);
        output.push(char::from(HEX_DIGITS[value >> 4]));
        output.push(char::from(HEX_DIGITS[value & 0x0f]));
    }
    output
}

/// Isolated consumer-vector fixture; not a production packet constructor.
#[cfg(test)]
pub(super) fn current_features_test_packet(
    payload: [[u8; 2]; SOURCE_COUNT],
    parent: SourceFlagsParentLineage,
) -> SourceFlagsPacket {
    let lineage = Arc::new(SourceFlagsLineage {
        parent: Arc::new(parent),
        current_pcm_receipt: SourceFlagsDigest([0; 32]),
        controls_receipt: SourceFlagsDigest([0; 32]),
        source_control_clock: None,
        route_receipt: SourceFlagsDigest([0; 32]),
        writer_receipt: SourceFlagsDigest([0; 32]),
    });
    SourceFlagsPacket::from_producer(payload, lineage)
}

#[cfg(test)]
mod source_flags_tests;
