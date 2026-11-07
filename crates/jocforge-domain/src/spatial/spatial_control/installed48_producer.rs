// pattern: Functional Core
//! MAIN F03W installed48/post-window proposal from SpatialControl-owned before-state.

use crate::{
    frontend::normal_control_snapshot::{
        NormalControlAdmHeldSourceView, NormalControlAdmInputOutput, NormalControlHeldSourceView,
        NormalControlRow, NormalControlSnapshotOutput,
    },
    numeric::FiniteF32,
    source_axis::{SourceAxisBinding, SourceAxisError, SourceAxisOwnerGeneration},
    spatial::{
        diffusion_filter::DiffusionFilterParameters,
        diffusion_tail::DiffusionTailState,
        installed48_render::{
            Installed48Activity, Installed48RenderInput, Installed48RenderOutput,
            render_installed48,
        },
        installed48_source_map::{
            Installed48SourceMap, Installed48SourceMapError, PrimaryPrefixRoute,
        },
        post_window_fanout::{CallbackPhase, PostWindowFanoutError, PostWindowRenderProposal},
        prefix_membership::{PrefixMembershipError, PrefixMembershipProducer},
        source_flags::{SourceFlagsError, triangle_activity_witness, triangle_window},
        spatial_control::{SpatialControlMachine, SpatialControlState},
    },
    timeline::{AdmSampleIndex, FrontendCallbackIndex, NormalCallbackCount},
    transaction::CallbackLineageIdentity,
};
use std::sync::Arc;

const FRAME_SAMPLES: usize = 1_536;
const CACHED_DRAIN_COUNT: u64 = 4;

#[derive(Clone, Copy, Debug)]
pub enum SpatialControlInstalled48Callback<'a> {
    Normal {
        controls: &'a NormalControlSnapshotOutput,
        physical_pcm_rows: &'a [Vec<u32>],
    },
    NormalAdmInputOutput {
        controls: &'a NormalControlAdmInputOutput,
        physical_pcm_rows: &'a [Vec<u32>],
    },
    CachedDrain {
        controls: &'a NormalControlHeldSourceView,
    },
    CachedDrainAdmInput {
        controls: &'a NormalControlAdmHeldSourceView,
    },
}

#[derive(Clone, Copy)]
pub struct SpatialControlInstalled48Input<'a> {
    pub callback: SpatialControlInstalled48Callback<'a>,
    pub source_map: &'a Installed48SourceMap,
    pub diffusion_parameters: Option<&'a DiffusionFilterParameters>,
}

#[derive(Clone, Copy)]
enum NormalControlRows<'a> {
    Selected {
        prepared: &'a crate::frontend::normal_control_snapshot::NormalControlPreparedSnapshot,
        rows: &'a [NormalControlRow],
    },
    AdmInput(&'a NormalControlAdmInputOutput),
    AdmHeld(&'a NormalControlAdmHeldSourceView),
}

impl NormalControlRows<'_> {
    fn identity_matches(self, transaction_id: &CallbackLineageIdentity) -> bool {
        match self {
            Self::Selected { prepared, .. } => {
                transaction_id.programme_id() == prepared.programme_id()
                    && transaction_id.attempt_id() == prepared.attempt_id()
            }
            Self::AdmInput(output) => output.transaction_id() == transaction_id,
            Self::AdmHeld(held) => {
                transaction_id == held.delivery_transaction_id()
                    && transaction_id.programme_id() == held.prepared_snapshot().programme_id()
                    && transaction_id.attempt_id() == held.prepared_snapshot().attempt_id()
            }
        }
    }

    fn regular_audio_controls(
        self,
        source_map: &Installed48SourceMap,
    ) -> Result<
        Vec<crate::spatial::installed48_render::Installed48AudioControl>,
        Installed48SourceMapError,
    > {
        match self {
            Self::Selected { prepared, rows } => {
                source_map.regular_audio_controls_from_normal_rows(prepared, rows)
            }
            Self::AdmInput(output) => {
                source_map.regular_audio_controls_from_adm_input_output(output)
            }
            Self::AdmHeld(held) => source_map.regular_audio_controls_from_adm_held_view(held),
        }
    }

    fn prefix_route(
        self,
        source_map: &Installed48SourceMap,
        quiet_regular_rows: &[u8],
        producer: &PrefixMembershipProducer,
    ) -> Result<PrimaryPrefixRoute, Installed48SourceMapError> {
        match self {
            Self::Selected { prepared, rows } => source_map
                .produce_primary_prefix_route_from_normal_controls(
                    quiet_regular_rows,
                    prepared,
                    rows,
                    producer,
                ),
            Self::AdmInput(output) => source_map
                .produce_primary_prefix_route_from_adm_input_output(
                    quiet_regular_rows,
                    output,
                    producer,
                ),
            Self::AdmHeld(held) => source_map.produce_primary_prefix_route_from_adm_held_view(
                quiet_regular_rows,
                held,
                producer,
            ),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum SpatialControlInstalled48Error {
    CallbackIdentity,
    CallbackPhase,
    SourceExtent,
    SourcePcmShape,
    BeforeStateShape,
    CachedFrameUnavailable,
    CachedControlMismatch,
    NormalControlIdentity(Installed48SourceMapError),
    SourceAxis(SourceAxisError),
    PrefixMembership(PrefixMembershipError),
    Activity(SourceFlagsError),
    Render(&'static str),
    PostWindow(PostWindowFanoutError),
}

/// Pending F03W audio-owner delta. It carries no commit authority.
#[must_use]
pub struct SpatialControlInstalled48AfterProjection {
    callback_index: FrontendCallbackIndex,
    callback_phase: CallbackPhase,
    source_cursor: AdmSampleIndex,
    normal_callbacks: NormalCallbackCount,
    source_pcm_rows: Vec<Vec<u32>>,
    regular_raw_history_bits: Box<[u32]>,
    current_rows_bits: Box<[u32]>,
    diffusion_after: Vec<Option<DiffusionTailState>>,
}

#[must_use]
pub struct SpatialControlInstalled48AfterParts {
    pub callback_index: FrontendCallbackIndex,
    pub callback_phase: CallbackPhase,
    pub source_cursor: AdmSampleIndex,
    pub normal_callbacks: NormalCallbackCount,
    pub source_pcm_rows: Vec<Vec<u32>>,
    pub regular_raw_history_bits: Box<[u32]>,
    pub current_rows_bits: Box<[u32]>,
    pub diffusion_after: Vec<Option<DiffusionTailState>>,
}

impl SpatialControlInstalled48AfterProjection {
    pub const fn callback_index(&self) -> FrontendCallbackIndex {
        self.callback_index
    }

    pub const fn callback_phase(&self) -> CallbackPhase {
        self.callback_phase
    }

    pub const fn source_cursor(&self) -> AdmSampleIndex {
        self.source_cursor
    }

    pub const fn normal_callbacks(&self) -> NormalCallbackCount {
        self.normal_callbacks
    }

    pub fn into_parts(self) -> SpatialControlInstalled48AfterParts {
        SpatialControlInstalled48AfterParts {
            callback_index: self.callback_index,
            callback_phase: self.callback_phase,
            source_cursor: self.source_cursor,
            normal_callbacks: self.normal_callbacks,
            source_pcm_rows: self.source_pcm_rows,
            regular_raw_history_bits: self.regular_raw_history_bits,
            current_rows_bits: self.current_rows_bits,
            diffusion_after: self.diffusion_after,
        }
    }
}

/// One-shot installed48 proposal and its matching SpatialControl after-value projection.
#[must_use]
pub struct SpatialControlInstalled48Proposal {
    post_window: PostWindowRenderProposal,
    after: SpatialControlInstalled48AfterProjection,
    activity: Installed48Activity,
    membership_route: PrimaryPrefixRoute,
}

impl SpatialControlInstalled48Proposal {
    pub fn quiet_rows(&self) -> &[u8] {
        &self.activity.quiet
    }

    pub const fn membership_route(&self) -> &PrimaryPrefixRoute {
        &self.membership_route
    }

    pub fn prefix_writer_indices(&self) -> Vec<u8> {
        self.membership_route.writer_indices()
    }

    pub fn source_pcm_rows(&self) -> &[Vec<u32>] {
        &self.after.source_pcm_rows
    }

    pub fn into_parts(
        self,
    ) -> (
        PostWindowRenderProposal,
        SpatialControlInstalled48AfterProjection,
    ) {
        (self.post_window, self.after)
    }
}

/// Builds MAIN installed48 from MAIN's own PCM/history/cache and current typed F01E rows.
/// No Meter state, source-map control snapshot, or local owner commit is used.
pub fn stage_spatial_control_installed48(
    owner: &SpatialControlMachine,
    input: SpatialControlInstalled48Input<'_>,
) -> Result<SpatialControlInstalled48Proposal, SpatialControlInstalled48Error> {
    let before = owner.committed();
    let callback_index = before.callback_index();
    let physical_host_count = usize::from(input.source_map.physical_host_count());
    let regular_count = usize::from(input.source_map.regular_count());
    let (
        phase,
        transaction_id,
        control_rows,
        normal_callbacks,
        owner_generation,
        source_origin_generation,
        current_pcm_rows,
        source_cursor,
        actual_samples,
    ) = match input.callback {
        SpatialControlInstalled48Callback::Normal {
            controls,
            physical_pcm_rows,
        } => {
            validate_normal_callback(before, controls)?;
            (
                CallbackPhase::Normal,
                controls.transaction_id().clone(),
                NormalControlRows::Selected {
                    prepared: controls.prepared_identity(),
                    rows: controls.control38_rows(),
                },
                controls.normal_callbacks(),
                controls.next_owner_generation(),
                controls.normal_origin_generation(),
                validate_normal_pcm_rows(
                    physical_pcm_rows,
                    controls.actual_samples(),
                    physical_host_count,
                )?,
                before.source_cursor_samples(),
                controls.actual_samples(),
            )
        }
        SpatialControlInstalled48Callback::NormalAdmInputOutput {
            controls,
            physical_pcm_rows,
        } => {
            validate_adm_input_normal_callback(before, controls, input.source_map)?;
            (
                CallbackPhase::Normal,
                controls.transaction_id().clone(),
                NormalControlRows::AdmInput(controls),
                controls.normal_callbacks(),
                controls.next_owner_generation(),
                controls.normal_origin_generation(),
                validate_normal_pcm_rows(
                    physical_pcm_rows,
                    controls.actual_samples(),
                    physical_host_count,
                )?,
                before.source_cursor_samples(),
                controls.actual_samples(),
            )
        }
        SpatialControlInstalled48Callback::CachedDrain { controls } => {
            validate_cached_drain(before, controls, input.source_map)?;
            let cached_pcm = before
                .source_cache
                .last_normal_physical_pcm_rows_bits
                .as_ref()
                .ok_or(SpatialControlInstalled48Error::CachedFrameUnavailable)?;
            let current_pcm_rows = rows_from_cached_pcm(cached_pcm, physical_host_count)?;
            (
                CallbackPhase::CachedDrain,
                controls.delivery_transaction_id().clone(),
                NormalControlRows::Selected {
                    prepared: controls.prepared_identity(),
                    rows: controls.control38_rows(),
                },
                controls.normal_callbacks(),
                controls.owner_generation(),
                controls.source_origin_generation(),
                current_pcm_rows,
                before.source_cursor_samples(),
                0,
            )
        }
        SpatialControlInstalled48Callback::CachedDrainAdmInput { controls } => {
            validate_adm_input_cached_drain(before, controls, input.source_map)?;
            let cached_pcm = before
                .source_cache
                .last_normal_physical_pcm_rows_bits
                .as_ref()
                .ok_or(SpatialControlInstalled48Error::CachedFrameUnavailable)?;
            let current_pcm_rows = rows_from_cached_pcm(cached_pcm, physical_host_count)?;
            (
                CallbackPhase::CachedDrain,
                controls.delivery_transaction_id().clone(),
                NormalControlRows::AdmHeld(controls),
                NormalCallbackCount::new(controls.expected_normal_callback_count()),
                controls.owner_generation(),
                controls.source_origin_generation(),
                current_pcm_rows,
                before.source_cursor_samples(),
                0,
            )
        }
    };

    let current_index = transaction_id.callback_index();
    let source_axis = Arc::new(
        SourceAxisBinding::new(
            input
                .source_map
                .rows()
                .iter()
                .map(super::super::installed48_source_map::Installed48SourceRow::source_index)
                .collect::<Vec<_>>(),
            SourceAxisOwnerGeneration::new(owner_generation),
            *input.source_map.source_binding().source_receipt_sha256(),
            transaction_id.clone(),
            source_origin_generation,
        )
        .map_err(SpatialControlInstalled48Error::SourceAxis)?,
    );
    if current_index != callback_index
        || before.physical_host_count != physical_host_count
        || before.regular_tail_count != regular_count
        || before.active_source_ids.len() != input.source_map.rows().len()
        || before
            .active_source_ids
            .iter()
            .zip(input.source_map.rows())
            .any(|(before_id, map_row)| before_id != &map_row.source_index())
        || !control_rows.identity_matches(&transaction_id)
        || before.retired()
        || before.previous_pcm_bits().len() != physical_host_count * FRAME_SAMPLES
        || before.regular_raw_history_bits().len() != regular_count * FRAME_SAMPLES
        || before
            .previous_pcm_bits()
            .iter()
            .chain(before.regular_raw_history_bits())
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(SpatialControlInstalled48Error::BeforeStateShape);
    }

    let controls = control_rows
        .regular_audio_controls(input.source_map)
        .map_err(SpatialControlInstalled48Error::NormalControlIdentity)?;
    let activity = produce_activity(before, input.source_map, &current_pcm_rows, &controls)?;
    let membership =
        PrefixMembershipProducer::for_selected_primary_p11(input.source_map.p11_geometry())
            .map_err(SpatialControlInstalled48Error::PrefixMembership)?;
    let membership_route = control_rows
        .prefix_route(input.source_map, &activity.quiet, &membership)
        .map_err(SpatialControlInstalled48Error::NormalControlIdentity)?;

    let diffusion_before = before
        .render_audio
        .diffusion_tails
        .iter()
        .map(Option::as_ref)
        .collect::<Vec<_>>();
    let rendered = render_installed48(Installed48RenderInput {
        current_pcm_rows: &current_pcm_rows,
        previous_pcm_bits: before.previous_pcm_bits(),
        controls: &controls,
        activity: &activity,
        membership_route: &membership_route,
        source_map: input.source_map,
        diffusion_before: &diffusion_before,
        diffusion_parameters: input.diffusion_parameters,
    })
    .map_err(|error| SpatialControlInstalled48Error::Render(error.code()))?;

    let Installed48RenderOutput {
        postwindow_rows,
        current_rows_bits,
        regular_raw_history_bits,
        diffusion_after,
    } = rendered;
    let postwindow_payload = postwindow_rows
        .into_iter()
        .flatten()
        .map(f32::from_bits)
        .collect();
    let post_window = PostWindowRenderProposal::try_from_spatial_control_callback(
        SpatialControlMachine::render_authority(),
        owner,
        transaction_id,
        phase,
        source_axis,
        postwindow_payload,
    )
    .map_err(SpatialControlInstalled48Error::PostWindow)?;

    let source_cursor_after = source_cursor
        .value()
        .checked_add(u64::from(actual_samples))
        .ok_or(SpatialControlInstalled48Error::SourceExtent)?;
    if source_cursor_after > input.source_map.source_binding().source_samples().value()
        || (phase == CallbackPhase::CachedDrain
            && source_cursor_after != before.source_cursor_samples().value())
    {
        return Err(SpatialControlInstalled48Error::SourceExtent);
    }
    Ok(SpatialControlInstalled48Proposal {
        post_window,
        after: SpatialControlInstalled48AfterProjection {
            callback_index,
            callback_phase: phase,
            source_cursor: AdmSampleIndex::new(source_cursor_after),
            normal_callbacks,
            source_pcm_rows: current_pcm_rows,
            regular_raw_history_bits,
            current_rows_bits,
            diffusion_after,
        },
        activity,
        membership_route,
    })
}

fn validate_normal_callback(
    before: &SpatialControlState,
    controls: &NormalControlSnapshotOutput,
) -> Result<(), SpatialControlInstalled48Error> {
    let callback = before.callback_index();
    let source_samples = controls.prepared_identity().source_samples();
    let expected_normal_callbacks = source_samples.div_ceil(FRAME_SAMPLES as u64);
    let expected_actual_samples = source_samples
        .checked_sub(before.source_cursor_samples().value())
        .ok_or(SpatialControlInstalled48Error::SourceExtent)?
        .min(FRAME_SAMPLES as u64);
    let next_callback = callback
        .value()
        .checked_add(1)
        .ok_or(SpatialControlInstalled48Error::SourceExtent)?;
    if u64::from(callback.value()) >= expected_normal_callbacks
        || before.normal_callbacks().value() != u64::from(callback.value())
        || controls.value_generation() != callback
        || controls.normal_origin_generation() != callback
        || controls.transaction_id().callback_index() != callback
        || controls.query_sample() != before.source_cursor_samples().value()
        || u64::from(controls.actual_samples()) != expected_actual_samples
        || controls.normal_callbacks().value() != u64::from(next_callback)
        || controls.next_callback_index().value() != next_callback
        || controls.next_owner_generation() != u64::from(next_callback)
    {
        return Err(SpatialControlInstalled48Error::CallbackIdentity);
    }
    Ok(())
}

fn validate_adm_input_normal_callback(
    before: &SpatialControlState,
    controls: &NormalControlAdmInputOutput,
    source_map: &Installed48SourceMap,
) -> Result<(), SpatialControlInstalled48Error> {
    let callback = before.callback_index();
    let prepared = controls.prepared_identity();
    let source_samples = prepared.source_samples();
    let expected_normal_callbacks = source_samples.div_ceil(FRAME_SAMPLES as u64);
    let expected_actual_samples = source_samples
        .checked_sub(before.source_cursor_samples().value())
        .ok_or(SpatialControlInstalled48Error::SourceExtent)?
        .min(FRAME_SAMPLES as u64);
    let next_callback = callback
        .value()
        .checked_add(1)
        .ok_or(SpatialControlInstalled48Error::SourceExtent)?;
    if source_samples != source_map.source_binding().source_samples().value()
        || usize::from(source_map.physical_host_count()) != before.physical_host_count
        || usize::from(source_map.regular_count()) != before.regular_tail_count
        || u64::from(callback.value()) >= expected_normal_callbacks
        || before.normal_callbacks().value() != u64::from(callback.value())
        || controls.value_generation() != callback
        || controls.normal_origin_generation() != callback
        || controls.transaction_id().callback_index() != callback
        || controls.query_sample() != before.source_cursor_samples().value()
        || u64::from(controls.actual_samples()) != expected_actual_samples
        || controls.normal_callbacks().value() != u64::from(next_callback)
        || controls.next_callback_index().value() != next_callback
        || controls.next_owner_generation() != u64::from(next_callback)
        || controls.physical_rows().len() != usize::from(source_map.physical_host_count())
        || controls.regular_rows().len() != usize::from(source_map.regular_count())
    {
        return Err(SpatialControlInstalled48Error::CallbackIdentity);
    }
    Ok(())
}

fn validate_cached_drain(
    before: &SpatialControlState,
    controls: &NormalControlHeldSourceView,
    source_map: &Installed48SourceMap,
) -> Result<(), SpatialControlInstalled48Error> {
    let normal_count = controls.normal_callbacks().value();
    let expected_count = controls
        .prepared_identity()
        .source_samples()
        .div_ceil(FRAME_SAMPLES as u64);
    let callback = u64::from(before.callback_index().value());
    let delivery_end = normal_count
        .checked_add(CACHED_DRAIN_COUNT)
        .ok_or(SpatialControlInstalled48Error::SourceExtent)?;
    let origin = normal_count
        .checked_sub(1)
        .and_then(|value| u32::try_from(value).ok())
        .map(FrontendCallbackIndex::new)
        .ok_or(SpatialControlInstalled48Error::CallbackIdentity)?;
    let cached_rows = before
        .source_cache
        .last_control_snapshot
        .as_ref()
        .ok_or(SpatialControlInstalled48Error::CachedControlMismatch)?;
    if expected_count != normal_count
        || before.normal_callbacks().value() != normal_count
        || before.source_cursor_samples().value() != controls.prepared_identity().source_samples()
        || before.remaining_cached_drains() == 0
        || callback < normal_count
        || callback >= delivery_end
        || controls.delivery_generation() != before.callback_index()
        || controls.source_origin_generation() != origin
        || controls.source_origin_transaction_id().callback_index() != origin
        || controls.source_origin_transaction_id().programme_id()
            != controls.prepared_identity().programme_id()
        || controls.source_origin_transaction_id().attempt_id()
            != controls.prepared_identity().attempt_id()
        || cached_rows.len() != usize::from(source_map.physical_host_count())
        || cached_rows
            .iter()
            .zip(controls.control38_rows())
            .any(|(cached, row)| !cached_control_matches(cached, row))
    {
        return Err(SpatialControlInstalled48Error::CachedControlMismatch);
    }
    Ok(())
}

fn validate_adm_input_cached_drain(
    before: &SpatialControlState,
    controls: &NormalControlAdmHeldSourceView,
    source_map: &Installed48SourceMap,
) -> Result<(), SpatialControlInstalled48Error> {
    let normal_count = controls.expected_normal_callback_count();
    let expected_count = source_map
        .source_binding()
        .source_samples()
        .value()
        .div_ceil(FRAME_SAMPLES as u64);
    let callback = u64::from(before.callback_index().value());
    let delivery_end = normal_count
        .checked_add(CACHED_DRAIN_COUNT)
        .ok_or(SpatialControlInstalled48Error::SourceExtent)?;
    let origin = normal_count
        .checked_sub(1)
        .and_then(|value| u32::try_from(value).ok())
        .map(FrontendCallbackIndex::new)
        .ok_or(SpatialControlInstalled48Error::CallbackIdentity)?;
    let cached_rows = before
        .source_cache
        .last_control_snapshot
        .as_ref()
        .ok_or(SpatialControlInstalled48Error::CachedControlMismatch)?;
    if expected_count != normal_count
        || before.normal_callbacks().value() != normal_count
        || before.source_cursor_samples().value()
            != source_map.source_binding().source_samples().value()
        || before.remaining_cached_drains() == 0
        || callback < normal_count
        || callback >= delivery_end
        || controls.delivery_generation() != before.callback_index()
        || controls.source_origin_generation() != origin
        || controls.owner_generation() != normal_count
        || controls.source_origin_transaction_id().callback_index() != origin
        || controls.source_origin_transaction_id().programme_id()
            != controls.prepared_snapshot().programme_id()
        || controls.source_origin_transaction_id().attempt_id()
            != controls.prepared_snapshot().attempt_id()
        || controls.delivery_transaction_id().programme_id()
            != controls.prepared_snapshot().programme_id()
        || controls.delivery_transaction_id().attempt_id()
            != controls.prepared_snapshot().attempt_id()
        || cached_rows.len() != usize::from(source_map.physical_host_count())
        || cached_rows
            .iter()
            .zip(controls.physical_control_rows())
            .any(|(cached, row)| !cached_control_matches(cached, row))
    {
        return Err(SpatialControlInstalled48Error::CachedControlMismatch);
    }
    Ok(())
}

fn cached_control_matches(
    cached: &super::PhysicalControlSnapshotRow,
    row: &NormalControlRow,
) -> bool {
    match (cached, row) {
        (super::PhysicalControlSnapshotRow::Bed6(cached), NormalControlRow::Bed6 { words, .. }) => {
            cached == words
        }
        (
            super::PhysicalControlSnapshotRow::Object13(cached),
            NormalControlRow::Object13 { control, .. },
        ) => cached == &control.words(),
        _ => false,
    }
}

fn validate_normal_pcm_rows(
    rows: &[Vec<u32>],
    actual_samples: u32,
    expected_physical_rows: usize,
) -> Result<Vec<Vec<u32>>, SpatialControlInstalled48Error> {
    let actual_samples = usize::try_from(actual_samples)
        .map_err(|_| SpatialControlInstalled48Error::SourcePcmShape)?;
    if actual_samples == 0
        || actual_samples > FRAME_SAMPLES
        || rows.len() != expected_physical_rows
        || rows.iter().any(|row| {
            row.len() != FRAME_SAMPLES
                || row.iter().any(|word| !f32::from_bits(*word).is_finite())
                || row[actual_samples..].iter().any(|word| *word != 0)
        })
    {
        return Err(SpatialControlInstalled48Error::SourcePcmShape);
    }
    Ok(rows.to_vec())
}

fn rows_from_cached_pcm(
    cached: &[u32],
    physical_host_count: usize,
) -> Result<Vec<Vec<u32>>, SpatialControlInstalled48Error> {
    if cached.len() != physical_host_count * FRAME_SAMPLES {
        return Err(SpatialControlInstalled48Error::CachedFrameUnavailable);
    }
    Ok(cached
        .chunks_exact(FRAME_SAMPLES)
        .map(<[u32]>::to_vec)
        .collect())
}

fn produce_activity(
    before: &SpatialControlState,
    source_map: &Installed48SourceMap,
    current_pcm_rows: &[Vec<u32>],
    controls: &[crate::spatial::installed48_render::Installed48AudioControl],
) -> Result<Installed48Activity, SpatialControlInstalled48Error> {
    let window = triangle_window().map_err(SpatialControlInstalled48Error::Activity)?;
    let physical_host_count = usize::from(source_map.physical_host_count());
    let regular_count = usize::from(source_map.regular_count());
    if current_pcm_rows.len() != physical_host_count
        || before.regular_raw_history_bits().len() != regular_count * FRAME_SAMPLES
        || controls.len() != regular_count
    {
        return Err(SpatialControlInstalled48Error::BeforeStateShape);
    }
    let mut quiet = vec![0_u8; regular_count];
    let mut mute_reset = vec![false; regular_count];
    for (tail, binding) in source_map.physical_to_global_source().iter().enumerate() {
        let host = usize::from(binding.physical_host().value());
        let current_words = current_pcm_rows
            .get(host)
            .filter(|row| row.len() == FRAME_SAMPLES)
            .ok_or(SpatialControlInstalled48Error::SourcePcmShape)?;
        let history_words = before.regular_raw_history_bits()
            [tail * FRAME_SAMPLES..(tail + 1) * FRAME_SAMPLES]
            .iter()
            .copied();
        let before_frame = history_words
            .map(|word| {
                FiniteF32::new(f32::from_bits(word))
                    .map_err(|_| SpatialControlInstalled48Error::BeforeStateShape)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let current_frame = current_words
            .iter()
            .copied()
            .map(|word| {
                FiniteF32::new(f32::from_bits(word))
                    .map_err(|_| SpatialControlInstalled48Error::SourcePcmShape)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let witness = triangle_activity_witness(&before_frame, &current_frame, &window)
            .map_err(SpatialControlInstalled48Error::Activity)?;
        let control = controls[tail];
        if control.mute > 1 {
            return Err(SpatialControlInstalled48Error::NormalControlIdentity(
                Installed48SourceMapError::AudioControlDomain,
            ));
        }
        quiet[tail] = u8::from(!witness || control.mute != 0);
        mute_reset[tail] = witness && control.mute != 0;
    }
    Ok(Installed48Activity { quiet, mute_reset })
}

#[cfg(test)]
mod tests {
    use super::{
        SpatialControlInstalled48Callback, SpatialControlInstalled48Input,
        stage_spatial_control_installed48,
    };
    use crate::{
        frontend::{
            PackageManifestEvidence, ProviderFileEvidence, validate_f01e_package_manifest,
            validate_f01e_provider, validate_f03_package_manifest,
            validate_f03_target_cold_provider,
        },
        profile::EncodingProfile,
        spatial::{SpatialControlMachine, source_geometry_outer::tests::f01b_f01e_fixture},
        timeline::CodecSourceIndex,
    };
    use std::sync::Arc;

    const MANIFEST_BYTES: &[u8] =
        include_bytes!("../../../tests/fixtures/f01e_provider/MANIFEST.json");
    const F01E_PROVIDER_BYTES: &[u8] =
        include_bytes!("../../../tests/fixtures/f01e_provider/f01e-selected-constructor-v1.json");
    const F03_PROVIDER_BYTES: &[u8] =
        include_bytes!("../../../tests/fixtures/target_cold/f03-selected-target-cold-v1.json");

    fn cold_spatial_control() -> SpatialControlMachine {
        let manifest_evidence = PackageManifestEvidence::new(Arc::from(MANIFEST_BYTES), None);
        let f01e_manifest = validate_f01e_package_manifest(&manifest_evidence).unwrap();
        let f01e = validate_f01e_provider(
            &ProviderFileEvidence::new(Arc::from(F01E_PROVIDER_BYTES), None),
            &f01e_manifest,
        )
        .unwrap();
        let f03_manifest = validate_f03_package_manifest(&manifest_evidence).unwrap();
        let f03 = validate_f03_target_cold_provider(
            &ProviderFileEvidence::new(Arc::from(F03_PROVIDER_BYTES), None),
            &f03_manifest,
        )
        .unwrap();
        SpatialControlMachine::new_cold(EncodingProfile::selected(), &f01e, &f03).unwrap()
    }

    #[test]
    fn normal_f03w_proposal_uses_spatial_control_before_state_without_committing() {
        let fixture = f01b_f01e_fixture();
        let source_map = fixture.source_map;
        let (_, normal_controls) = fixture.normal.into_frontend_parent_parts();
        let owner = cold_spatial_control();
        let before_digest = owner.digest();
        let before_generation = owner.generation();
        let physical_pcm_rows = vec![vec![0_u32; 1_536]; 38];

        let proposal = stage_spatial_control_installed48(
            &owner,
            SpatialControlInstalled48Input {
                callback: SpatialControlInstalled48Callback::Normal {
                    controls: &normal_controls,
                    physical_pcm_rows: &physical_pcm_rows,
                },
                source_map: &source_map,
                diffusion_parameters: None,
            },
        )
        .unwrap();

        assert_eq!(proposal.quiet_rows(), &[1; 37]);
        assert!(proposal.membership_route().contributions().is_empty());
        assert!(proposal.prefix_writer_indices().is_empty());
        assert_eq!(proposal.source_pcm_rows(), physical_pcm_rows);
        assert_eq!(owner.digest(), before_digest);
        assert_eq!(owner.generation(), before_generation);
        assert!(owner.committed().source_cache_is_cold());

        let (post_window, after) = proposal.into_parts();
        let (_, spatial_audio) = post_window.fanout();
        assert_eq!(spatial_audio.sample_count(), 48 * 3_072);
        assert!(
            spatial_audio
                .source_samples(CodecSourceIndex::new(0))
                .unwrap()
                .iter()
                .all(|sample| sample.to_bits() == 0)
        );
        assert_eq!(
            after.callback_phase(),
            crate::spatial::post_window_fanout::CallbackPhase::Normal
        );
        assert_eq!(after.source_cursor().value(), 1_536);
        assert_eq!(after.source_pcm_rows.len(), 38);
        assert_eq!(after.regular_raw_history_bits.len(), 37 * 1_536);
        assert_eq!(after.current_rows_bits.len(), 48 * 1_536);
    }
}
