// pattern: Functional Core
//! Validated same-callback source-flags production and its raw-history proposal.

use super::spatial_control::{
    SpatialControlDigest, SpatialControlMachine, SpatialControlProviderLineage,
};
use crate::{
    frontend::ProviderSha256,
    numeric::{FiniteF32, R32Error, r32_mul},
    profile::{EncodingProfile, resolve_selected_profile},
    timeline::FrontendCallbackIndex,
    transaction::CallbackTransactionId,
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

#[derive(Debug, Eq, PartialEq)]
pub struct SourceFlagsProgrammeId(Box<str>);

impl SourceFlagsProgrammeId {
    pub fn new(value: &str) -> Result<Self, SourceFlagsError> {
        if value.is_empty() {
            Err(SourceFlagsError::EmptyProgrammeIdentity)
        } else {
            Ok(Self(value.into()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceFlagsAttempt(u64);

impl SourceFlagsAttempt {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

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
    transaction_id: CallbackTransactionId,
    callback_index: FrontendCallbackIndex,
    before_history_generation: SourceFlagsHistoryGeneration,
    before_history_digest: SpatialControlDigest,
    profile_identity: EncodingProfile,
    provider_lineage: SpatialControlProviderLineage,
}

impl SourceFlagsParentLineage {
    pub fn new(
        programme_id: SourceFlagsProgrammeId,
        attempt: SourceFlagsAttempt,
        transaction_id: CallbackTransactionId,
        callback_index: FrontendCallbackIndex,
        owner: &SpatialControlMachine,
    ) -> Result<Self, SourceFlagsError> {
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
            parent: Arc::new(parent),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SourceFlagsDigest([u8; 32]);

impl SourceFlagsDigest {
    fn from_parts(domain: &[u8], parent: &SourceFlagsParentLineage, payload: &[u8]) -> Self {
        let mut bytes = Vec::with_capacity(
            domain.len()
                + parent.programme_id.as_str().len()
                + parent.transaction_id.as_str().len()
                + 32
                + payload.len(),
        );
        bytes.extend_from_slice(domain);
        bytes.extend_from_slice(parent.programme_id.as_str().as_bytes());
        bytes.extend_from_slice(&parent.attempt.value().to_le_bytes());
        bytes.extend_from_slice(parent.transaction_id.as_str().as_bytes());
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
    route_receipt: SourceFlagsDigest,
    writer_receipt: SourceFlagsDigest,
}

impl SourceFlagsLineage {
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
        let window = triangle_window()?;
        let mut payload = [[0_u8; 2]; SOURCE_COUNT];

        for (source, pair) in payload[..PREFIX_SOURCE_COUNT].iter_mut().enumerate() {
            *pair = if input.prefix_writers.written[source] {
                [0, 0]
            } else {
                [1, 1]
            };
        }

        let width_limit = f32::from_bits(WIDTH_LIMIT_BITS);
        let mut history_proposal = Vec::with_capacity(REGULAR_HOSTS);
        for (row, host) in input.route.regular_hosts.iter().copied().enumerate() {
            let current = input.current_pcm.row(host);
            let before = &history.rows[row];
            let control = input.controls.for_host(host)?;
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
            payload[usize::from(input.route.destination(host)?)] = pair;
            history_proposal.push(raw_frame(current));
        }

        let lineage = Arc::new(make_lineage(
            Arc::clone(&input.parent),
            &input.current_pcm,
            &input.controls,
            &input.route,
            &input.prefix_writers,
        )?);
        let packet = SourceFlagsPacket::from_producer(payload, Arc::clone(&lineage));
        packet.validate_payload()?;
        packet.validate_parent_binding(&input.parent)?;
        let raw_history_proposal = RawHistoryProposal {
            rows: history_proposal.into_boxed_slice(),
            lineage,
        };
        Ok(SourceFlagsProduced {
            packet,
            raw_history_proposal,
        })
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

fn preflight(
    input: &SourceFlagsInput,
    history: &BeforeRegularHistory,
) -> Result<(), SourceFlagsError> {
    if input.current_pcm.rows.len() != HOST_CHANNELS
        || input
            .current_pcm
            .rows
            .iter()
            .any(|row| row.len() != FRAME_SAMPLES)
    {
        return Err(SourceFlagsError::PcmShape);
    }
    if history.rows.len() != REGULAR_HOSTS
        || history.rows.iter().any(|row| row.len() != FRAME_SAMPLES)
    {
        return Err(SourceFlagsError::HistoryShape);
    }
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
    for host in input.route.regular_hosts.iter().copied() {
        input.controls.for_host(host)?;
        input.route.destination(host)?;
    }
    if input.route.regular_hosts.len() != REGULAR_HOSTS
        || input.route.destination_by_host.len() != HOST_CHANNELS
        || input.route.destination(input.route.lfe_host).is_ok()
    {
        return Err(SourceFlagsError::RouteNotBijective);
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
) -> Result<SourceFlagsLineage, SourceFlagsError> {
    let current_pcm_receipt = digest_current_pcm(current_pcm, &parent);
    let controls_receipt = digest_controls(controls, route, &parent)?;
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
) -> Result<SourceFlagsDigest, SourceFlagsError> {
    let mut bytes = Vec::with_capacity(REGULAR_HOSTS * 6);
    for host in route.regular_hosts.iter().copied() {
        let control = controls.for_host(host)?;
        bytes.push(host.value());
        bytes.push(u8::from(control.muted));
        bytes.extend_from_slice(&control.width.to_bits().to_le_bytes());
        bytes.extend_from_slice(&[0, 0]);
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
        route_receipt: SourceFlagsDigest([0; 32]),
        writer_receipt: SourceFlagsDigest([0; 32]),
    });
    SourceFlagsPacket::from_producer(payload, lineage)
}

#[cfg(test)]
mod source_flags_tests;
