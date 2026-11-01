// pattern: Functional Core
//! Private F15 plan and destination admission boundary.
//!
//! This module loads owner-specific values from the approved runtime assets,
//! validates them with the owning domain parsers, and checks their shared
//! parent/input/profile binding before output work.

use super::{OutputRootBindingError, OutputStagingOwnerId, SelectedOutputProviderProjection};
use crate::PreparedJobInput;
use jocforge_domain::{
    codec::bap::{
        BapParameters, CONVERTER_FAST_GAIN_CODE, MAIN_FAST_GAIN_CODE, load_bap_parameters,
    },
    integrity::keyless_checksum::{KeylessIntegrityProvider, validate_keyless_provider},
    joc::{
        coefficient_representation::RepresentationRetryProvider,
        emdf::metadata_owner::ValidatedMetadataOwnerProvider, fit_score::FitScoreProvider,
        huffman::HuffmanCodebooks,
    },
    meter::prepared_job_identity::PreparedJobInputIdentity,
    profile::{
        EmdfCarriageProfile, EncodingProfile, ProfileError, ResolvedEncodingProfile,
        resolve_selected_profile,
    },
    timeline::{AdmSampleCount, ClockError, ProgrammeCounts, programme_counts},
    transaction::{AttemptId, ProgrammeId},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::marker::PhantomData;
use std::sync::Arc;

const CODEC_WIRE_SELECTED: &[u8] =
    include_bytes!("../../assets/providers/codec_wire_selected.json");
const PUBLIC_BAP_48K: &[u8] = include_bytes!("../../assets/providers/public_bap_48k.json");
const LIVE_JOC_COORDINATOR: &[u8] =
    include_bytes!("../../assets/providers/live_joc_coordinator.json");
const JOC_FUNCTIONAL_AUTHORIZED: &[u8] =
    include_bytes!("../../assets/providers/joc_functional_authorized.json");
const JOC_GENERATED_MATH: &[u8] = include_bytes!("../../assets/providers/joc_generated_math.json");
const JOC_TIMER_ORDER: &[u8] = include_bytes!("../../assets/providers/joc_timer_order.json");
const METADATA_PUBLIC_PROVIDER: &[u8] =
    include_bytes!("../../assets/providers/metadata_public_provider.json");
const METADATA_WRAPPER_CONFIG: &[u8] =
    include_bytes!("../../assets/providers/metadata_wrapper_config.json");
const INTEGRITY_PUBLIC_PROVIDER: &[u8] =
    include_bytes!("../../assets/providers/integrity_public_provider.json");

const CODEC_WIRE_SELECTED_SHA256: &str =
    "a7a0a721a9e515d197a2d12fc0c9cb942b2b393f9019383a66ce2635f7aaa0dd";
const LIVE_JOC_COORDINATOR_SHA256: &str =
    "d66eca446693535cf3a9fc1177a0e31a41dc9f3b6cffaaa42e79672fb0862e3b";
const JOC_FUNCTIONAL_AUTHORIZED_SHA256: &str =
    "af08ef531f8708ab81cd6af0bf8ae71504ffb19ba76d5fb8571b1ae8db53c747";
const JOC_GENERATED_MATH_SHA256: &str =
    "d42b7e8ebaabf27bdf519148986b70aa6de56f4eb6bfd0179cd0bd011e284fe7";
const JOC_TIMER_ORDER_SHA256: &str =
    "b425f8dffc0d914d660c46bcce5226c80d5fc680ab7c7f02e590fd6f889cb50b";
const INTEGRITY_PUBLIC_PROVIDER_SHA256: &str =
    "b0f52c55d38b9fba45ec39fc956cd8cd895ce2cfbc1f77d9493d99d2b8636a4d";

const SELECTED_F15_DEPENDENCY_MANIFEST_SHA256: &str =
    "a2cf06839d3da81d14240ccd6526eebc0b878721d48a03fd0d0813f26961f6ce";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct F15DependencyManifestIdentity(&'static str);

impl F15DependencyManifestIdentity {
    const fn selected() -> Self {
        Self(SELECTED_F15_DEPENDENCY_MANIFEST_SHA256)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum F15ProviderRole {
    CodecF12,
    JocF05Jl,
    F13Metadata,
    F14Integrity,
}

pub(super) struct CodecF12Role;
pub(super) struct JocF05JlRole;
pub(super) struct F13MetadataRole;
pub(super) struct F14IntegrityRole;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SelectedCodecWireProfile {
    sample_rate_hz: u32,
    bit_rate_bps: u32,
    frame_samples: u16,
    frame_bytes: u16,
    audio_blocks: u8,
    acmod: u8,
    lfe_on: u8,
    fullband_channels: u8,
    fullband_end_bin: u16,
    lfe_end_bin: u8,
}

pub(super) struct CodecF12ProviderValue {
    _wire_bytes: Arc<[u8]>,
    profile: SelectedCodecWireProfile,
    bap: BapParameters,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SelectedJocCoordinatorProfile {
    sample_rate_hz: u32,
    bit_rate_bps: u32,
    object_count: u8,
    carrier_count: u8,
    band_count: u8,
    fit_cell_count: u16,
    qmf_slots_per_generation: u8,
    samples_per_generation: u16,
    capacity_bytes: u16,
    dispatch_count: u16,
    published_writer_count: u16,
}

pub(super) struct JocF05JlProviderValue {
    _coordinator_bytes: Arc<[u8]>,
    profile: SelectedJocCoordinatorProfile,
    _representation_retry: RepresentationRetryProvider,
    _fit_score: FitScoreProvider,
    _huffman: HuffmanCodebooks,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SelectedMetadataTimerValues {
    period_samples: [u32; 6],
    sample_rate_hz: u32,
    hop_samples: u16,
    timer_to_input_field: [u8; 6],
    expiry_priority_timer_indices: [u8; 6],
    expiry_priority_output_fields: [u8; 6],
    cold_cached: [i16; 6],
    mode: u8,
    counter_low_s16: i16,
    body_map_order: [u8; 4],
    queue_mutation_order: [u8; 4],
    serialization_order: [u8; 4],
}

pub(super) struct F13MetadataProviderValue {
    metadata_owner: ValidatedMetadataOwnerProvider,
    timer: SelectedMetadataTimerValues,
}

pub(super) struct F14IntegrityProviderValue {
    provider: KeylessIntegrityProvider,
}

/// Private marker implemented by each selected owner-specific validated value.
pub(super) trait F15RoleValidatedValue<Role> {
    fn matches_parent_profile(
        &self,
        _profile_dimensions: EncodingProfile,
        _resolved_profile: ResolvedEncodingProfile,
    ) -> bool {
        true
    }
}

fn exact_json_asset(bytes: &[u8], expected_bytes: u64, expected_sha256: &str) -> Result<Value, ()> {
    if u64::try_from(bytes.len()).ok() != Some(expected_bytes)
        || sha256_hex(bytes) != expected_sha256
    {
        return Err(());
    }
    serde_json::from_slice(bytes).map_err(|_| ())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn object_field<'a>(
    value: &'a Value,
    name: &str,
) -> Result<&'a serde_json::Map<String, Value>, ()> {
    value.get(name).and_then(Value::as_object).ok_or(())
}

fn unsigned_field<T>(object: &serde_json::Map<String, Value>, name: &str) -> Result<T, ()>
where
    T: TryFrom<u64>,
{
    object
        .get(name)
        .and_then(Value::as_u64)
        .and_then(|value| T::try_from(value).ok())
        .ok_or(())
}

fn signed_field<T>(object: &serde_json::Map<String, Value>, name: &str) -> Result<T, ()>
where
    T: TryFrom<i64>,
{
    object
        .get(name)
        .and_then(Value::as_i64)
        .and_then(|value| T::try_from(value).ok())
        .ok_or(())
}

fn unsigned_u32_array6(value: &Value) -> Result<[u32; 6], ()> {
    let values = value.as_array().ok_or(())?;
    let parsed = values
        .iter()
        .map(|value| {
            value
                .as_u64()
                .and_then(|word| u32::try_from(word).ok())
                .ok_or(())
        })
        .collect::<Result<Vec<_>, _>>()?;
    parsed.try_into().map_err(|_| ())
}

fn unsigned_u8_array6(value: &Value) -> Result<[u8; 6], ()> {
    let values = value.as_array().ok_or(())?;
    let parsed = values
        .iter()
        .map(|value| {
            value
                .as_u64()
                .and_then(|word| u8::try_from(word).ok())
                .ok_or(())
        })
        .collect::<Result<Vec<_>, _>>()?;
    parsed.try_into().map_err(|_| ())
}

fn signed_i16_array6(value: &Value) -> Result<[i16; 6], ()> {
    let values = value.as_array().ok_or(())?;
    let parsed = values
        .iter()
        .map(|value| {
            value
                .as_i64()
                .and_then(|word| i16::try_from(word).ok())
                .ok_or(())
        })
        .collect::<Result<Vec<_>, _>>()?;
    parsed.try_into().map_err(|_| ())
}

fn unsigned_u8_array4(value: &Value) -> Result<[u8; 4], ()> {
    let values = value.as_array().ok_or(())?;
    let parsed = values
        .iter()
        .map(|value| {
            value
                .as_u64()
                .and_then(|word| u8::try_from(word).ok())
                .ok_or(())
        })
        .collect::<Result<Vec<_>, _>>()?;
    parsed.try_into().map_err(|_| ())
}

fn indexed_u8_object_array6(value: &Value) -> Result<[u8; 6], ()> {
    let object = value.as_object().ok_or(())?;
    let parsed = (0_u8..6)
        .map(|index| {
            object
                .get(&index.to_string())
                .and_then(Value::as_u64)
                .and_then(|value| u8::try_from(value).ok())
                .ok_or(())
        })
        .collect::<Result<Vec<_>, _>>()?;
    parsed.try_into().map_err(|_| ())
}

fn validate_codec_f12_provider() -> Result<CodecF12ProviderValue, ()> {
    let wire = exact_json_asset(CODEC_WIRE_SELECTED, 3_081, CODEC_WIRE_SELECTED_SHA256)?;
    if wire.get("schema").and_then(Value::as_str) != Some("codec-wire-selected-constants.draft.v1")
    {
        return Err(());
    }
    let profile_value = wire.get("selected_profile").ok_or(())?;
    let profile_object = profile_value.as_object().ok_or(())?;
    let profile = SelectedCodecWireProfile {
        sample_rate_hz: unsigned_field(profile_object, "sample_rate_hz")?,
        bit_rate_bps: unsigned_field(profile_object, "bit_rate_bps")?,
        frame_samples: unsigned_field(profile_object, "frame_samples")?,
        frame_bytes: unsigned_field(profile_object, "frame_bytes")?,
        audio_blocks: unsigned_field(profile_object, "audio_blocks")?,
        acmod: unsigned_field(profile_object, "acmod")?,
        lfe_on: unsigned_field(profile_object, "lfe_on")?,
        fullband_channels: unsigned_field(profile_object, "fullband_channels")?,
        fullband_end_bin: unsigned_field(profile_object, "fullband_end_bin")?,
        lfe_end_bin: unsigned_field(profile_object, "lfe_end_bin")?,
    };
    if profile
        != (SelectedCodecWireProfile {
            sample_rate_hz: 48_000,
            bit_rate_bps: 1_024_000,
            frame_samples: 1_536,
            frame_bytes: 4_096,
            audio_blocks: 6,
            acmod: 7,
            lfe_on: 1,
            fullband_channels: 5,
            fullband_end_bin: 217,
            lfe_end_bin: 7,
        })
        || profile.fullband_channels != 5
        || profile.fullband_end_bin != 217
        || profile.lfe_end_bin != 7
    {
        return Err(());
    }

    let header_fields = wire
        .get("transport_header_fixed_fields_msb_first")
        .and_then(Value::as_array)
        .ok_or(())?;
    let expected_header = [
        ("syncword", 16_u64, 2_935_u64),
        ("strmtyp", 2, 0),
        ("substreamid", 3, 0),
        ("frmsiz", 11, 2_047),
        ("fscod", 2, 0),
        ("numblkscod", 2, 3),
        ("acmod", 3, 7),
        ("lfeon", 1, 1),
        ("bsid", 5, 16),
    ];
    if header_fields.len() != expected_header.len()
        || header_fields
            .iter()
            .zip(expected_header)
            .any(|(value, expected)| {
                value.get("name").and_then(Value::as_str) != Some(expected.0)
                    || value.get("width").and_then(Value::as_u64) != Some(expected.1)
                    || value.get("value").and_then(Value::as_u64) != Some(expected.2)
            })
    {
        return Err(());
    }

    let allocation = object_field(&wire, "allocation")?;
    if unsigned_field::<u8>(allocation, "main_fast_gain_code")? != MAIN_FAST_GAIN_CODE
        || unsigned_field::<u8>(allocation, "converter_fast_gain_code")? != CONVERTER_FAST_GAIN_CODE
        || unsigned_field::<u64>(allocation, "actual_frame_bits")? != 32_768
        || unsigned_field::<u64>(allocation, "actual_body_reserve_bits")? != 34
    {
        return Err(());
    }

    let bap = load_bap_parameters(Some(PUBLIC_BAP_48K)).map_err(|_| ())?;
    if bap.provider_sha256() != sha256_bytes(PUBLIC_BAP_48K) {
        return Err(());
    }
    Ok(CodecF12ProviderValue {
        _wire_bytes: Arc::from(CODEC_WIRE_SELECTED),
        profile,
        bap,
    })
}

fn validate_live_joc_coordinator() -> Result<SelectedJocCoordinatorProfile, ()> {
    let coordinator = exact_json_asset(LIVE_JOC_COORDINATOR, 8_538, LIVE_JOC_COORDINATOR_SHA256)?;
    if coordinator.get("schema").and_then(Value::as_str)
        != Some("selected-live-joc-coordinator-parameters.v1")
    {
        return Err(());
    }
    let profile = object_field(&coordinator, "profile")?;
    let clocks = object_field(&coordinator, "clocks")?;
    let cold = object_field(&coordinator, "cold")?;
    let value = SelectedJocCoordinatorProfile {
        sample_rate_hz: unsigned_field(profile, "sample_rate_hz")?,
        bit_rate_bps: unsigned_field(profile, "bit_rate_bps")?,
        object_count: unsigned_field(profile, "objects")?,
        carrier_count: unsigned_field(profile, "carriers")?,
        band_count: unsigned_field(profile, "bands")?,
        fit_cell_count: unsigned_field(profile, "fit_cells")?,
        qmf_slots_per_generation: unsigned_field(profile, "qmf_slots_per_generation")?,
        samples_per_generation: unsigned_field(profile, "samples_per_generation")?,
        capacity_bytes: unsigned_field(profile, "capacity_bytes")?,
        dispatch_count: unsigned_field(clocks, "dispatches")?,
        published_writer_count: unsigned_field(clocks, "published_writers")?,
    };
    if value
        != (SelectedJocCoordinatorProfile {
            sample_rate_hz: 48_000,
            bit_rate_bps: 1_024_000,
            object_count: 15,
            carrier_count: 5,
            band_count: 12,
            fit_cell_count: 180,
            qmf_slots_per_generation: 48,
            samples_per_generation: 1_536,
            capacity_bytes: 372,
            dispatch_count: 4_249,
            published_writer_count: 4_247,
        })
        || unsigned_field::<u8>(clocks, "first_dispatch")? != 1
        || unsigned_field::<u8>(clocks, "callback_one_based_offset")? != 2
        || unsigned_field::<u8>(clocks, "callback_zero_based_offset")? != 1
        || signed_field::<i8>(clocks, "codec_source_offset")? != -1
        || signed_field::<i8>(clocks, "writer_offset")? != -1
        || unsigned_field::<u8>(cold, "next_dispatch")? != 1
        || signed_field::<i8>(cold, "sequence_counter")? != -1
        || unsigned_field::<u8>(cold, "successful_dispatches")? != 0
        || unsigned_field::<u8>(cold, "receiver_qmf_generation")? != 0
        || unsigned_field::<u8>(cold, "temporal_generation")? != 0
        || unsigned_field::<u8>(cold, "geometry_generation")? != 0
        || unsigned_field::<u8>(cold, "fit_generation")? != 0
        || unsigned_field::<u8>(cold, "committed_writer_count")? != 0
        || cold.get("retired").and_then(Value::as_bool) != Some(false)
        || coordinator["receipt_schema"]["stages"]
            != json!([
                "receiver_qmf",
                "temporal",
                "geometry",
                "fit180",
                "profile_retry",
                "huffman_id14"
            ])
    {
        return Err(());
    }
    Ok(value)
}

fn validate_joc_f05jl_provider() -> Result<JocF05JlProviderValue, ()> {
    let coordinator_profile = validate_live_joc_coordinator()?;
    let functional = exact_json_asset(
        JOC_FUNCTIONAL_AUTHORIZED,
        2_448,
        JOC_FUNCTIONAL_AUTHORIZED_SHA256,
    )?;
    if functional.get("schema").and_then(Value::as_str)
        != Some("joc.selected-authorized-functional-parameters.v1")
    {
        return Err(());
    }
    exact_json_asset(JOC_GENERATED_MATH, 39_081, JOC_GENERATED_MATH_SHA256)?;
    let representation_retry =
        RepresentationRetryProvider::from_approved_bytes(JOC_FUNCTIONAL_AUTHORIZED)
            .map_err(|_| ())?;
    let fit_score =
        FitScoreProvider::from_approved_bytes(JOC_FUNCTIONAL_AUTHORIZED, JOC_GENERATED_MATH)
            .map_err(|_| ())?;
    let huffman = HuffmanCodebooks::selected().map_err(|_| ())?;
    Ok(JocF05JlProviderValue {
        _coordinator_bytes: Arc::from(LIVE_JOC_COORDINATOR),
        profile: coordinator_profile,
        _representation_retry: representation_retry,
        _fit_score: fit_score,
        _huffman: huffman,
    })
}

fn validate_selected_metadata_timer() -> Result<SelectedMetadataTimerValues, ()> {
    let timer = exact_json_asset(JOC_TIMER_ORDER, 1_940, JOC_TIMER_ORDER_SHA256)?;
    if timer.get("schema").and_then(Value::as_str)
        != Some("joc-timer-payload-order-functional-parameters.v1")
    {
        return Err(());
    }
    let id1 = object_field(&timer, "id1_timer")?;
    let payload = object_field(&timer, "payload_timeline")?;
    let value = SelectedMetadataTimerValues {
        period_samples: unsigned_u32_array6(&id1["period_samples"])?,
        sample_rate_hz: unsigned_field(id1, "sample_rate")?,
        hop_samples: unsigned_field(id1, "hop_samples")?,
        timer_to_input_field: indexed_u8_object_array6(&id1["timer_to_input_field"])?,
        expiry_priority_timer_indices: unsigned_u8_array6(&id1["expiry_priority_timer_indices"])?,
        expiry_priority_output_fields: unsigned_u8_array6(&id1["expiry_priority_output_fields"])?,
        cold_cached: signed_i16_array6(&id1["cold_cached"])?,
        mode: unsigned_field(id1, "mode")?,
        counter_low_s16: signed_field(id1, "counter_low_s16")?,
        body_map_order: unsigned_u8_array4(&payload["body_map_order"])?,
        queue_mutation_order: unsigned_u8_array4(&payload["push_pop_order"])?,
        serialization_order: unsigned_u8_array4(&payload["serialization_order"])?,
    };
    if value.sample_rate_hz != 48_000
        || value.hop_samples != 1_536
        || value.period_samples != [24_000, 48_000, 48_000, 4_800, 24_000, 48_000]
        || value.timer_to_input_field != [0, 5, 4, 6, 7, 8]
        || value.expiry_priority_timer_indices != [1, 2, 5, 0, 4, 3]
        || value.expiry_priority_output_fields != [5, 4, 8, 0, 7, 6]
        || value.cold_cached != [-1, -1_024, -1_024, -1_024, -1_024, -1_024]
        || value.mode != 1
        || value.counter_low_s16 != -513
        || value.body_map_order != [11, 14, 2, 1]
        || value.queue_mutation_order != [11, 1, 2, 14]
        || value.serialization_order != [11, 14, 2, 1]
        || timer["joc_complexity"]["selected_value"] != json!(16)
    {
        return Err(());
    }
    Ok(value)
}

fn validate_f13_metadata_provider() -> Result<F13MetadataProviderValue, ()> {
    let metadata_owner = ValidatedMetadataOwnerProvider::from_approved_bytes(
        METADATA_PUBLIC_PROVIDER,
        METADATA_WRAPPER_CONFIG,
    )
    .map_err(|_| ())?;
    let timer = validate_selected_metadata_timer()?;
    let metadata_json: Value =
        serde_json::from_slice(metadata_owner.provider_file()).map_err(|_| ())?;
    if metadata_json["profile_id"] != json!("selected-48k-1536-four-queue.v1")
        || metadata_json["sample_rate"] != json!(timer.sample_rate_hz)
        || metadata_json["source_samples"] != json!(timer.hop_samples)
        || metadata_json["timer_period_samples"] != json!(timer.period_samples)
        || metadata_json["queue_order"] != json!(timer.queue_mutation_order)
        || metadata_json["serialization_order"] != json!(timer.serialization_order)
        || metadata_json["suppressed_sources"] != json!(2)
    {
        return Err(());
    }
    Ok(F13MetadataProviderValue {
        metadata_owner,
        timer,
    })
}

fn validate_f14_integrity_provider() -> Result<F14IntegrityProviderValue, ()> {
    if sha256_hex(INTEGRITY_PUBLIC_PROVIDER) != INTEGRITY_PUBLIC_PROVIDER_SHA256 {
        return Err(());
    }
    let provider = validate_keyless_provider(INTEGRITY_PUBLIC_PROVIDER).map_err(|_| ())?;
    if provider.profile_id() != "keyless-crc32-crc8-crc16.v1" {
        return Err(());
    }
    Ok(F14IntegrityProviderValue { provider })
}

#[derive(Clone)]
struct F15PlanParentContext {
    programme_id: ProgrammeId,
    prepared_input_identity: Arc<PreparedJobInputIdentity>,
    profile_dimensions: EncodingProfile,
    resolved_profile: ResolvedEncodingProfile,
    dependency_manifest_identity: F15DependencyManifestIdentity,
    source_samples: u64,
}

impl F15PlanParentContext {
    fn from_prepared_input(input: &PreparedJobInput) -> Result<Self, F15PlanAdmissionError> {
        let resolved_profile = input.configuration().selected_profile();
        let profile_dimensions =
            EncodingProfile::selected_with_carriage(resolved_profile.carriage());
        let re_resolved = resolve_selected_profile(&profile_dimensions)
            .map_err(F15PlanAdmissionError::Profile)?;
        if re_resolved != resolved_profile {
            return Err(F15PlanAdmissionError::ResolvedProfileMismatch);
        }
        Ok(Self {
            programme_id: input.programme_id().clone(),
            prepared_input_identity: input.domain_identity_capability(),
            profile_dimensions,
            resolved_profile,
            dependency_manifest_identity: F15DependencyManifestIdentity::selected(),
            source_samples: input.configuration().source_samples(),
        })
    }

    #[cfg(test)]
    fn for_test(
        programme_id: ProgrammeId,
        prepared_input_identity: Arc<PreparedJobInputIdentity>,
        profile_dimensions: EncodingProfile,
        source_samples: u64,
    ) -> Result<Self, F15PlanAdmissionError> {
        let resolved_profile = resolve_selected_profile(&profile_dimensions)
            .map_err(F15PlanAdmissionError::Profile)?;
        Ok(Self {
            programme_id,
            prepared_input_identity,
            profile_dimensions,
            resolved_profile,
            dependency_manifest_identity: F15DependencyManifestIdentity::selected(),
            source_samples,
        })
    }

    fn admission_binding(&self) -> OwnerAdmissionBinding {
        OwnerAdmissionBinding {
            programme_id: self.programme_id.clone(),
            prepared_input_identity: Arc::clone(&self.prepared_input_identity),
            profile_dimensions: self.profile_dimensions,
            resolved_profile: self.resolved_profile,
            carriage: self.resolved_profile.carriage(),
            dependency_manifest_identity: self.dependency_manifest_identity,
        }
    }
}

impl F15RoleValidatedValue<CodecF12Role> for CodecF12ProviderValue {
    fn matches_parent_profile(
        &self,
        profile_dimensions: EncodingProfile,
        resolved_profile: ResolvedEncodingProfile,
    ) -> bool {
        self.profile.sample_rate_hz == 48_000
            && self.profile.bit_rate_bps == 1_024_000
            && self.profile.frame_samples == 1_536
            && self.profile.frame_bytes == 4_096
            && self.profile.audio_blocks == 6
            && self.bap.provider_sha256() == sha256_bytes(PUBLIC_BAP_48K)
            && profile_dimensions
                == EncodingProfile::selected_with_carriage(resolved_profile.carriage())
    }
}

impl F15RoleValidatedValue<JocF05JlRole> for JocF05JlProviderValue {
    fn matches_parent_profile(
        &self,
        profile_dimensions: EncodingProfile,
        resolved_profile: ResolvedEncodingProfile,
    ) -> bool {
        self.profile.sample_rate_hz == 48_000
            && self.profile.bit_rate_bps == 1_024_000
            && self.profile.object_count == 15
            && self.profile.carrier_count == 5
            && self.profile.band_count == 12
            && self.profile.fit_cell_count == 180
            && self.profile.qmf_slots_per_generation == 48
            && self.profile.samples_per_generation == 1_536
            && self.profile.capacity_bytes == 372
            && self.profile.dispatch_count == 4_249
            && self.profile.published_writer_count == 4_247
            && profile_dimensions
                == EncodingProfile::selected_with_carriage(resolved_profile.carriage())
    }
}

impl F15RoleValidatedValue<F13MetadataRole> for F13MetadataProviderValue {
    fn matches_parent_profile(
        &self,
        profile_dimensions: EncodingProfile,
        resolved_profile: ResolvedEncodingProfile,
    ) -> bool {
        self.timer.sample_rate_hz == 48_000
            && self.timer.hop_samples == 1_536
            && self.timer.queue_mutation_order == [11, 1, 2, 14]
            && self.timer.serialization_order == [11, 14, 2, 1]
            && resolved_profile.carriage() == EmdfCarriageProfile::DeployedCompat
            && sha256_hex(self.metadata_owner.provider_file())
                == "6d94c55c8c011fd4762e60ec92ed491f19b17540c72ba82094a36f6e666f2ef7"
            && sha256_hex(self.metadata_owner.wrapper_file())
                == "722e93ad671e4faab6ae9f33125bd49f8f369f717db0c9a3600cd861cdbd5a56"
            && profile_dimensions
                == EncodingProfile::selected_with_carriage(resolved_profile.carriage())
    }
}

impl F15RoleValidatedValue<F14IntegrityRole> for F14IntegrityProviderValue {
    fn matches_parent_profile(
        &self,
        profile_dimensions: EncodingProfile,
        resolved_profile: ResolvedEncodingProfile,
    ) -> bool {
        self.provider.profile_id() == "keyless-crc32-crc8-crc16.v1"
            && self.provider.provider_sha256().to_hex()
                == "57b117ab76c597ad6f6d65af65eecc17d2df78cf0874d092fe1f67585e47bd75"
            && profile_dimensions
                == EncodingProfile::selected_with_carriage(resolved_profile.carriage())
    }
}

#[derive(Clone)]
struct OwnerAdmissionBinding {
    programme_id: ProgrammeId,
    prepared_input_identity: Arc<PreparedJobInputIdentity>,
    profile_dimensions: EncodingProfile,
    resolved_profile: ResolvedEncodingProfile,
    carriage: EmdfCarriageProfile,
    dependency_manifest_identity: F15DependencyManifestIdentity,
}

/// A role-specific handle retains the actual immutable value returned by its
/// validator. It is not a manifest/hash-only or PASS-boolean receipt.
struct OwnerIssuedProviderAdmission<Role, Value: F15RoleValidatedValue<Role>> {
    value: Arc<Value>,
    binding: OwnerAdmissionBinding,
    role: PhantomData<fn() -> Role>,
}

impl<Role, Value: F15RoleValidatedValue<Role>> OwnerIssuedProviderAdmission<Role, Value> {
    fn from_owner_validated_value(value: Arc<Value>, binding: OwnerAdmissionBinding) -> Self {
        Self {
            value,
            binding,
            role: PhantomData,
        }
    }
}

struct F15ProviderAdmissionCandidates<
    CodecValue: F15RoleValidatedValue<CodecF12Role>,
    JocValue: F15RoleValidatedValue<JocF05JlRole>,
    MetadataValue: F15RoleValidatedValue<F13MetadataRole>,
    IntegrityValue: F15RoleValidatedValue<F14IntegrityRole>,
> {
    codec_f12: Option<OwnerIssuedProviderAdmission<CodecF12Role, CodecValue>>,
    joc_f05jl: Option<OwnerIssuedProviderAdmission<JocF05JlRole, JocValue>>,
    f13_metadata: Option<OwnerIssuedProviderAdmission<F13MetadataRole, MetadataValue>>,
    f14_integrity: Option<OwnerIssuedProviderAdmission<F14IntegrityRole, IntegrityValue>>,
}

#[allow(dead_code)] // Owner values remain alive in the plan for later source-stage consumption.
struct ValidatedF15ProviderAdmissions<
    CodecValue: F15RoleValidatedValue<CodecF12Role>,
    JocValue: F15RoleValidatedValue<JocF05JlRole>,
    MetadataValue: F15RoleValidatedValue<F13MetadataRole>,
    IntegrityValue: F15RoleValidatedValue<F14IntegrityRole>,
> {
    codec_f12: OwnerIssuedProviderAdmission<CodecF12Role, CodecValue>,
    joc_f05jl: OwnerIssuedProviderAdmission<JocF05JlRole, JocValue>,
    f13_metadata: OwnerIssuedProviderAdmission<F13MetadataRole, MetadataValue>,
    f14_integrity: OwnerIssuedProviderAdmission<F14IntegrityRole, IntegrityValue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NormalizedFinalDestinationIdentity(String);

impl NormalizedFinalDestinationIdentity {
    /// Wraps the exact normalized identity returned by the trusted parent/IO
    /// admission boundary; this constructor performs no filesystem access.
    fn from_parent_validated(value: &str) -> Result<Self, F15PlanAdmissionError> {
        if value.is_empty() || value.contains('\0') {
            return Err(F15PlanAdmissionError::InvalidFinalDestinationIdentity);
        }
        Ok(Self(value.to_owned()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ParentReservedStagingNamespace(String);

impl ParentReservedStagingNamespace {
    fn from_parent_reservation(value: &str) -> Result<Self, F15PlanAdmissionError> {
        if value.is_empty() || value.contains('\0') {
            return Err(F15PlanAdmissionError::InvalidStagingNamespace);
        }
        Ok(Self(value.to_owned()))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FinalDestinationPolicy {
    NoOverwrite,
}

/// Opaque parent-issued pre-I/O capability. Durable write/exposure receipts
/// remain owned by the IO shell and are not pre-minted here.
struct ParentValidatedDestinationCapability {
    programme_id: ProgrammeId,
    attempt_id: AttemptId,
    staging_owner_id: OutputStagingOwnerId,
    final_destination: NormalizedFinalDestinationIdentity,
    staging_namespace: ParentReservedStagingNamespace,
    policy: FinalDestinationPolicy,
}

impl ParentValidatedDestinationCapability {
    fn from_parent_validation(
        programme_id: ProgrammeId,
        attempt_id: AttemptId,
        staging_owner_id: OutputStagingOwnerId,
        final_destination: NormalizedFinalDestinationIdentity,
        staging_namespace: ParentReservedStagingNamespace,
    ) -> Result<Self, F15PlanAdmissionError> {
        if final_destination.0 == staging_namespace.0 {
            return Err(F15PlanAdmissionError::DestinationAndStagingAlias);
        }
        Ok(Self {
            programme_id,
            attempt_id,
            staging_owner_id,
            final_destination,
            staging_namespace,
            policy: FinalDestinationPolicy::NoOverwrite,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct F15DynamicExtents {
    programme_counts: ProgrammeCounts,
    published_output_bytes: u64,
}

/// Private, parent-owned F15 aggregate. It carries four actual typed values and
/// a pre-I/O destination capability, but does not construct OutputOwner or write files.
#[allow(dead_code)]
pub(super) struct ValidatedEncodingPlan<
    CodecValue: F15RoleValidatedValue<CodecF12Role>,
    JocValue: F15RoleValidatedValue<JocF05JlRole>,
    MetadataValue: F15RoleValidatedValue<F13MetadataRole>,
    IntegrityValue: F15RoleValidatedValue<F14IntegrityRole>,
> {
    programme_id: ProgrammeId,
    prepared_input_identity: Arc<PreparedJobInputIdentity>,
    profile_dimensions: EncodingProfile,
    resolved_profile: ResolvedEncodingProfile,
    dependency_manifest_identity: F15DependencyManifestIdentity,
    providers: ValidatedF15ProviderAdmissions<CodecValue, JocValue, MetadataValue, IntegrityValue>,
    destination: ParentValidatedDestinationCapability,
    extents: F15DynamicExtents,
    output_provider_projection: SelectedOutputProviderProjection,
    output_binding: super::ProgrammeOutputBinding,
}

impl<
    CodecValue: F15RoleValidatedValue<CodecF12Role>,
    JocValue: F15RoleValidatedValue<JocF05JlRole>,
    MetadataValue: F15RoleValidatedValue<F13MetadataRole>,
    IntegrityValue: F15RoleValidatedValue<F14IntegrityRole>,
> ValidatedEncodingPlan<CodecValue, JocValue, MetadataValue, IntegrityValue>
{
    pub(super) const fn output_binding(&self) -> &super::ProgrammeOutputBinding {
        &self.output_binding
    }

    pub(super) fn final_destination(&self) -> &str {
        &self.destination.final_destination.0
    }

    pub(super) fn staging_namespace(&self) -> &str {
        &self.destination.staging_namespace.0
    }

    pub(super) fn is_bound_to(&self, input: &PreparedJobInput) -> bool {
        self.programme_id == *input.programme_id()
            && Arc::ptr_eq(
                &self.prepared_input_identity,
                &input.domain_identity_capability(),
            )
    }
}

pub(super) type SelectedF15Plan = ValidatedEncodingPlan<
    CodecF12ProviderValue,
    JocF05JlProviderValue,
    F13MetadataProviderValue,
    F14IntegrityProviderValue,
>;

impl SelectedF15Plan {
    pub(super) fn bap_parameters(&self) -> &BapParameters {
        &self.providers.codec_f12.value.bap
    }

    pub(super) fn fit_score_provider(&self) -> &FitScoreProvider {
        &self.providers.joc_f05jl.value._fit_score
    }

    pub(super) fn representation_retry_provider(&self) -> &RepresentationRetryProvider {
        &self.providers.joc_f05jl.value._representation_retry
    }

    pub(super) fn metadata_owner_provider(&self) -> &ValidatedMetadataOwnerProvider {
        &self.providers.f13_metadata.value.metadata_owner
    }

    pub(super) fn keyless_integrity_provider(&self) -> &KeylessIntegrityProvider {
        &self.providers.f14_integrity.value.provider
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum F15PlanAdmissionError {
    Profile(ProfileError),
    ResolvedProfileMismatch,
    MissingOwnerAdmission(F15ProviderRole),
    ProgrammeIdentityMismatch(F15ProviderRole),
    PreparedInputIdentityMismatch(F15ProviderRole),
    ProfileDimensionMismatch(F15ProviderRole),
    ResolvedProfileMismatchForRole(F15ProviderRole),
    CarriageMismatch(F15ProviderRole),
    DependencyManifestMismatch(F15ProviderRole),
    ApprovedDependencyManifestMismatch,
    OwnerProviderValue(F15ProviderRole),
    DestinationCapabilityMissing,
    DestinationProgrammeMismatch,
    DestinationAttemptMismatch,
    DestinationStagingOwnerMismatch,
    InvalidFinalDestinationIdentity,
    InvalidStagingNamespace,
    DestinationAndStagingAlias,
    ZeroSampleCount,
    ArithmeticOverflow,
    OutputProviderProjection(OutputRootBindingError),
    OutputConfiguration(OutputRootBindingError),
}

fn validate_owner_binding<Role, Value: F15RoleValidatedValue<Role>>(
    role: F15ProviderRole,
    handle: &OwnerIssuedProviderAdmission<Role, Value>,
    context: &F15PlanParentContext,
) -> Result<(), F15PlanAdmissionError> {
    if handle.binding.programme_id != context.programme_id {
        return Err(F15PlanAdmissionError::ProgrammeIdentityMismatch(role));
    }
    if !Arc::ptr_eq(
        &handle.binding.prepared_input_identity,
        &context.prepared_input_identity,
    ) {
        return Err(F15PlanAdmissionError::PreparedInputIdentityMismatch(role));
    }
    if handle.binding.profile_dimensions != context.profile_dimensions {
        return Err(F15PlanAdmissionError::ProfileDimensionMismatch(role));
    }
    if handle.binding.resolved_profile != context.resolved_profile {
        return Err(F15PlanAdmissionError::ResolvedProfileMismatchForRole(role));
    }
    if handle.binding.carriage != context.resolved_profile.carriage() {
        return Err(F15PlanAdmissionError::CarriageMismatch(role));
    }
    if handle.binding.dependency_manifest_identity != context.dependency_manifest_identity {
        return Err(F15PlanAdmissionError::DependencyManifestMismatch(role));
    }
    if !handle
        .value
        .matches_parent_profile(context.profile_dimensions, context.resolved_profile)
    {
        return Err(F15PlanAdmissionError::OwnerProviderValue(role));
    }
    Ok(())
}

type SelectedProviderAdmissionCandidates = F15ProviderAdmissionCandidates<
    CodecF12ProviderValue,
    JocF05JlProviderValue,
    F13MetadataProviderValue,
    F14IntegrityProviderValue,
>;

fn load_selected_owner_admissions(
    context: &F15PlanParentContext,
) -> Result<SelectedProviderAdmissionCandidates, F15PlanAdmissionError> {
    let binding = context.admission_binding();
    let codec = Arc::new(
        validate_codec_f12_provider()
            .map_err(|()| F15PlanAdmissionError::OwnerProviderValue(F15ProviderRole::CodecF12))?,
    );
    let joc = Arc::new(
        validate_joc_f05jl_provider()
            .map_err(|()| F15PlanAdmissionError::OwnerProviderValue(F15ProviderRole::JocF05Jl))?,
    );
    let metadata =
        Arc::new(validate_f13_metadata_provider().map_err(|()| {
            F15PlanAdmissionError::OwnerProviderValue(F15ProviderRole::F13Metadata)
        })?);
    let integrity =
        Arc::new(validate_f14_integrity_provider().map_err(|()| {
            F15PlanAdmissionError::OwnerProviderValue(F15ProviderRole::F14Integrity)
        })?);
    Ok(F15ProviderAdmissionCandidates {
        codec_f12: Some(OwnerIssuedProviderAdmission::from_owner_validated_value(
            codec,
            binding.clone(),
        )),
        joc_f05jl: Some(OwnerIssuedProviderAdmission::from_owner_validated_value(
            joc,
            binding.clone(),
        )),
        f13_metadata: Some(OwnerIssuedProviderAdmission::from_owner_validated_value(
            metadata,
            binding.clone(),
        )),
        f14_integrity: Some(OwnerIssuedProviderAdmission::from_owner_validated_value(
            integrity, binding,
        )),
    })
}

/// Parent-only admission gate. No runtime receipt is constructed here; every
/// present role must already carry the owning validator's typed loaded value.
fn admit_validated_encoding_plan<
    CodecValue: F15RoleValidatedValue<CodecF12Role>,
    JocValue: F15RoleValidatedValue<JocF05JlRole>,
    MetadataValue: F15RoleValidatedValue<F13MetadataRole>,
    IntegrityValue: F15RoleValidatedValue<F14IntegrityRole>,
>(
    context: F15PlanParentContext,
    current_attempt: AttemptId,
    current_staging_owner: &OutputStagingOwnerId,
    admissions: F15ProviderAdmissionCandidates<CodecValue, JocValue, MetadataValue, IntegrityValue>,
    destination: Option<ParentValidatedDestinationCapability>,
) -> Result<
    ValidatedEncodingPlan<CodecValue, JocValue, MetadataValue, IntegrityValue>,
    F15PlanAdmissionError,
> {
    if context.dependency_manifest_identity != F15DependencyManifestIdentity::selected() {
        return Err(F15PlanAdmissionError::ApprovedDependencyManifestMismatch);
    }

    let codec_f12 = admissions
        .codec_f12
        .ok_or(F15PlanAdmissionError::MissingOwnerAdmission(
            F15ProviderRole::CodecF12,
        ))?;
    let joc_f05jl = admissions
        .joc_f05jl
        .ok_or(F15PlanAdmissionError::MissingOwnerAdmission(
            F15ProviderRole::JocF05Jl,
        ))?;
    let f13_metadata =
        admissions
            .f13_metadata
            .ok_or(F15PlanAdmissionError::MissingOwnerAdmission(
                F15ProviderRole::F13Metadata,
            ))?;
    let f14_integrity =
        admissions
            .f14_integrity
            .ok_or(F15PlanAdmissionError::MissingOwnerAdmission(
                F15ProviderRole::F14Integrity,
            ))?;

    validate_owner_binding(F15ProviderRole::CodecF12, &codec_f12, &context)?;
    validate_owner_binding(F15ProviderRole::JocF05Jl, &joc_f05jl, &context)?;
    validate_owner_binding(F15ProviderRole::F13Metadata, &f13_metadata, &context)?;
    validate_owner_binding(F15ProviderRole::F14Integrity, &f14_integrity, &context)?;

    let destination = destination.ok_or(F15PlanAdmissionError::DestinationCapabilityMissing)?;
    if destination.programme_id != context.programme_id {
        return Err(F15PlanAdmissionError::DestinationProgrammeMismatch);
    }
    if destination.attempt_id != current_attempt {
        return Err(F15PlanAdmissionError::DestinationAttemptMismatch);
    }
    if destination.staging_owner_id != *current_staging_owner {
        return Err(F15PlanAdmissionError::DestinationStagingOwnerMismatch);
    }
    if destination.final_destination.0.is_empty()
        || destination.staging_namespace.0.is_empty()
        || destination.final_destination.0 == destination.staging_namespace.0
        || destination.policy != FinalDestinationPolicy::NoOverwrite
    {
        return Err(F15PlanAdmissionError::InvalidFinalDestinationIdentity);
    }

    let sample_count =
        AdmSampleCount::new(context.source_samples).map_err(|error| match error {
            ClockError::ZeroSampleCount => F15PlanAdmissionError::ZeroSampleCount,
            ClockError::ArithmeticOverflow => F15PlanAdmissionError::ArithmeticOverflow,
        })?;
    let counts =
        programme_counts(sample_count).map_err(|_| F15PlanAdmissionError::ArithmeticOverflow)?;
    let projection = SelectedOutputProviderProjection::project_selected_f15_output_provider(
        context.resolved_profile,
    )
    .map_err(F15PlanAdmissionError::OutputProviderProjection)?;
    let published_output_bytes = counts
        .as_tuple()
        .4
        .checked_mul(u64::from(projection.output_provider().frame_bytes))
        .ok_or(F15PlanAdmissionError::ArithmeticOverflow)?;
    let product_name = std::path::Path::new(&destination.final_destination.0)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or(F15PlanAdmissionError::InvalidFinalDestinationIdentity)?;
    let attempt_label = current_attempt.value().to_string();
    let output_configuration = super::OutputConfiguration::new(
        projection.output_provider().clone(),
        &attempt_label,
        product_name,
        context.source_samples,
    )
    .map_err(|error| {
        F15PlanAdmissionError::OutputConfiguration(OutputRootBindingError::OutputOwner(error))
    })?;
    let output_owner = super::OutputOwner::cold(output_configuration);
    let output_binding = super::ProgrammeOutputBinding {
        programme_id: context.programme_id.clone(),
        attempt_id: current_attempt,
        staging_owner_id: current_staging_owner.clone(),
        output_owner,
    };

    Ok(ValidatedEncodingPlan {
        programme_id: context.programme_id,
        prepared_input_identity: context.prepared_input_identity,
        profile_dimensions: context.profile_dimensions,
        resolved_profile: context.resolved_profile,
        dependency_manifest_identity: context.dependency_manifest_identity,
        providers: ValidatedF15ProviderAdmissions {
            codec_f12,
            joc_f05jl,
            f13_metadata,
            f14_integrity,
        },
        destination,
        extents: F15DynamicExtents {
            programme_counts: counts,
            published_output_bytes,
        },
        output_provider_projection: projection,
        output_binding,
    })
}

/// Admits the selected runtime provider values and the parent-preflighted output path.
pub(super) fn admit_selected_f15_plan(
    input: &PreparedJobInput,
    attempt_id: AttemptId,
    staging_owner_id: &OutputStagingOwnerId,
    final_destination: &str,
) -> Result<SelectedF15Plan, F15PlanAdmissionError> {
    let context = F15PlanParentContext::from_prepared_input(input)?;
    let admissions = load_selected_owner_admissions(&context)?;
    let final_path = std::path::Path::new(final_destination);
    if !final_path.is_absolute() {
        return Err(F15PlanAdmissionError::InvalidFinalDestinationIdentity);
    }
    let parent = final_path
        .parent()
        .and_then(std::path::Path::to_str)
        .filter(|value| !value.is_empty())
        .ok_or(F15PlanAdmissionError::InvalidFinalDestinationIdentity)?;
    let projection = SelectedOutputProviderProjection::project_selected_f15_output_provider(
        context.resolved_profile,
    )
    .map_err(F15PlanAdmissionError::OutputProviderProjection)?;
    let attempt_label = attempt_id.value().to_string();
    let staging_name = projection
        .output_provider()
        .staging_name_template
        .replace("{attempt_id}", &attempt_label);
    let staging_namespace = std::path::Path::new(parent)
        .join(staging_name)
        .to_str()
        .map(str::to_owned)
        .ok_or(F15PlanAdmissionError::InvalidStagingNamespace)?;
    let destination = ParentValidatedDestinationCapability::from_parent_validation(
        input.programme_id().clone(),
        attempt_id,
        staging_owner_id.clone(),
        NormalizedFinalDestinationIdentity::from_parent_validated(final_destination)?,
        ParentReservedStagingNamespace::from_parent_reservation(&staging_namespace)?,
    )?;
    admit_validated_encoding_plan(
        context,
        attempt_id,
        staging_owner_id,
        admissions,
        Some(destination),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use jocforge_domain::{
        frontend::job_configuration::SelectedInputKind,
        meter::prepared_job_identity::{
            NormalizedJobConfigurationIdentity, PreparedAdmChunkSpan, PreparedAdmGraphProvenance,
            PreparedInputResourceIdentity,
        },
        profile::{EmdfCarriageProfile, EncodingProfile},
    };

    struct TestProviderValue(u8);

    impl F15RoleValidatedValue<CodecF12Role> for TestProviderValue {}
    impl F15RoleValidatedValue<JocF05JlRole> for TestProviderValue {}
    impl F15RoleValidatedValue<F13MetadataRole> for TestProviderValue {}
    impl F15RoleValidatedValue<F14IntegrityRole> for TestProviderValue {}

    #[test]
    fn selected_f15_owner_values_load_from_approved_runtime_assets() {
        let codec = validate_codec_f12_provider().expect("selected F12 assets validate");
        assert_eq!(codec.profile.frame_bytes, 4_096);
        assert_eq!(codec.bap.provider_sha256(), sha256_bytes(PUBLIC_BAP_48K));

        let joc = validate_joc_f05jl_provider().expect("selected F05 assets validate");
        assert_eq!(joc.profile.object_count, 15);
        assert_eq!(joc.profile.capacity_bytes, 372);

        let metadata = validate_f13_metadata_provider().expect("selected F13 assets validate");
        assert_eq!(metadata.timer.hop_samples, 1_536);
        assert_eq!(metadata.timer.serialization_order, [11, 14, 2, 1]);
        assert_eq!(
            sha256_hex(metadata.metadata_owner.provider_file()),
            "6d94c55c8c011fd4762e60ec92ed491f19b17540c72ba82094a36f6e666f2ef7"
        );

        let integrity = validate_f14_integrity_provider().expect("selected F14 asset validates");
        assert_eq!(
            integrity.provider.profile_id(),
            "keyless-crc32-crc8-crc16.v1"
        );
    }

    fn identity(resource_byte: u8) -> Arc<PreparedJobInputIdentity> {
        let resource = PreparedInputResourceIdentity::new([resource_byte; 32], 1_000);
        let graph = PreparedAdmGraphProvenance::new(
            resource,
            PreparedAdmChunkSpan::new(*b"axml", 100, 50, [0x41; 32]),
            PreparedAdmChunkSpan::new(*b"chna", 200, 30, [0x42; 32]),
            PreparedAdmChunkSpan::new(*b"dbmd", 300, 40, [0x43; 32]),
        );
        Arc::new(
            PreparedJobInputIdentity::new(
                resource,
                NormalizedJobConfigurationIdentity::new(
                    [0x22; 32],
                    [0x33; 32],
                    SelectedInputKind::Adm,
                ),
                graph,
            )
            .expect("test identity has valid resource-bound graph spans"),
        )
    }

    fn parent_context(source_samples: u64, resource_byte: u8) -> F15PlanParentContext {
        let dimensions = EncodingProfile::selected();
        F15PlanParentContext::for_test(
            ProgrammeId::new("plan-test-programme").expect("nonempty test programme"),
            identity(resource_byte),
            dimensions,
            source_samples,
        )
        .expect("selected test profile resolves")
    }

    fn role_handle<Role>(
        context: &F15PlanParentContext,
        value: u8,
    ) -> OwnerIssuedProviderAdmission<Role, TestProviderValue>
    where
        TestProviderValue: F15RoleValidatedValue<Role>,
    {
        OwnerIssuedProviderAdmission::from_owner_validated_value(
            Arc::new(TestProviderValue(value)),
            context.admission_binding(),
        )
    }

    fn complete_admissions(
        context: &F15PlanParentContext,
    ) -> F15ProviderAdmissionCandidates<
        TestProviderValue,
        TestProviderValue,
        TestProviderValue,
        TestProviderValue,
    > {
        F15ProviderAdmissionCandidates {
            codec_f12: Some(role_handle(context, 1)),
            joc_f05jl: Some(role_handle(context, 2)),
            f13_metadata: Some(role_handle(context, 3)),
            f14_integrity: Some(role_handle(context, 4)),
        }
    }

    fn destination(
        context: &F15PlanParentContext,
        attempt: AttemptId,
        staging_owner_id: OutputStagingOwnerId,
        final_path: &str,
        staging_namespace: &str,
    ) -> ParentValidatedDestinationCapability {
        ParentValidatedDestinationCapability::from_parent_validation(
            context.programme_id.clone(),
            attempt,
            staging_owner_id,
            NormalizedFinalDestinationIdentity::from_parent_validated(final_path)
                .expect("test identity is nonempty"),
            ParentReservedStagingNamespace::from_parent_reservation(staging_namespace)
                .expect("test staging namespace is nonempty"),
        )
        .expect("final and staging identities are distinct")
    }

    fn admit_for_test(
        context: F15PlanParentContext,
        attempt: AttemptId,
        staging_owner_id: &OutputStagingOwnerId,
        admissions: F15ProviderAdmissionCandidates<
            TestProviderValue,
            TestProviderValue,
            TestProviderValue,
            TestProviderValue,
        >,
        destination: Option<ParentValidatedDestinationCapability>,
    ) -> Result<
        ValidatedEncodingPlan<
            TestProviderValue,
            TestProviderValue,
            TestProviderValue,
            TestProviderValue,
        >,
        F15PlanAdmissionError,
    > {
        admit_validated_encoding_plan(context, attempt, staging_owner_id, admissions, destination)
    }

    #[test]
    fn plan_requires_all_four_typed_owner_values_before_destination_admission() {
        let context = parent_context(6_522_906, 0x11);
        let attempt = AttemptId::new(4);
        let staging_owner = OutputStagingOwnerId::new("stage-4").unwrap();
        let mut admissions = complete_admissions(&context);
        admissions.codec_f12 = None;
        assert!(matches!(
            admit_for_test(context.clone(), attempt, &staging_owner, admissions, None),
            Err(F15PlanAdmissionError::MissingOwnerAdmission(
                F15ProviderRole::CodecF12
            ))
        ));

        let mut admissions = complete_admissions(&context);
        admissions.joc_f05jl = None;
        assert!(matches!(
            admit_for_test(context.clone(), attempt, &staging_owner, admissions, None),
            Err(F15PlanAdmissionError::MissingOwnerAdmission(
                F15ProviderRole::JocF05Jl
            ))
        ));

        let mut admissions = complete_admissions(&context);
        admissions.f13_metadata = None;
        assert!(matches!(
            admit_for_test(context.clone(), attempt, &staging_owner, admissions, None),
            Err(F15PlanAdmissionError::MissingOwnerAdmission(
                F15ProviderRole::F13Metadata
            ))
        ));

        let mut admissions = complete_admissions(&context);
        admissions.f14_integrity = None;
        assert!(matches!(
            admit_for_test(context, attempt, &staging_owner, admissions, None),
            Err(F15PlanAdmissionError::MissingOwnerAdmission(
                F15ProviderRole::F14Integrity
            ))
        ));
    }

    #[test]
    fn plan_rejects_role_input_attempt_staging_and_destination_mismatches() {
        let context = parent_context(6_522_906, 0x11);
        let attempt = AttemptId::new(4);
        let staging_owner = OutputStagingOwnerId::new("stage-4").unwrap();
        let cap = destination(
            &context,
            attempt,
            staging_owner.clone(),
            "normalized://out/product.ec3",
            "staging://owner-4/attempt-4",
        );

        let foreign_context = parent_context(6_522_906, 0x12);
        let mut admissions = complete_admissions(&context);
        admissions.codec_f12 = Some(role_handle(&foreign_context, 1));
        assert!(matches!(
            admit_for_test(
                context.clone(),
                attempt,
                &staging_owner,
                admissions,
                Some(cap)
            ),
            Err(F15PlanAdmissionError::PreparedInputIdentityMismatch(
                F15ProviderRole::CodecF12
            ))
        ));

        let cap = destination(
            &context,
            AttemptId::new(3),
            staging_owner.clone(),
            "normalized://out/product.ec3",
            "staging://owner-4/attempt-3",
        );
        assert!(matches!(
            admit_for_test(
                context.clone(),
                attempt,
                &staging_owner,
                complete_admissions(&context),
                Some(cap)
            ),
            Err(F15PlanAdmissionError::DestinationAttemptMismatch)
        ));

        let wrong_owner = OutputStagingOwnerId::new("stage-other").unwrap();
        let cap = destination(
            &context,
            attempt,
            wrong_owner,
            "normalized://out/product.ec3",
            "staging://owner-other/attempt-4",
        );
        assert!(matches!(
            admit_for_test(
                context.clone(),
                attempt,
                &staging_owner,
                complete_admissions(&context),
                Some(cap)
            ),
            Err(F15PlanAdmissionError::DestinationStagingOwnerMismatch)
        ));
    }

    #[test]
    fn plan_rejects_profile_carriage_and_dependency_mismatches() {
        let context = parent_context(6_522_906, 0x11);
        let attempt = AttemptId::new(4);
        let staging_owner = OutputStagingOwnerId::new("stage-4").unwrap();
        let cap = destination(
            &context,
            attempt,
            staging_owner.clone(),
            "normalized://out/product.ec3",
            "staging://owner-4/attempt-4",
        );

        let alternate_dimensions =
            EncodingProfile::selected_with_carriage(EmdfCarriageProfile::EtsiEmdf);
        let alternate_context = F15PlanParentContext::for_test(
            context.programme_id.clone(),
            Arc::clone(&context.prepared_input_identity),
            alternate_dimensions,
            context.source_samples,
        )
        .expect("selected ETSI profile resolves");
        let mut admissions = complete_admissions(&context);
        admissions.codec_f12 = Some(role_handle(&alternate_context, 1));
        assert!(matches!(
            admit_for_test(
                context.clone(),
                attempt,
                &staging_owner,
                admissions,
                Some(cap)
            ),
            Err(F15PlanAdmissionError::ProfileDimensionMismatch(
                F15ProviderRole::CodecF12
            ))
        ));

        let cap = destination(
            &context,
            attempt,
            staging_owner.clone(),
            "normalized://out/product.ec3",
            "staging://owner-4/attempt-4",
        );
        let mut admissions = complete_admissions(&context);
        let codec = admissions.codec_f12.as_mut().unwrap();
        codec.binding.carriage = EmdfCarriageProfile::EtsiEmdf;
        assert!(matches!(
            admit_for_test(
                context.clone(),
                attempt,
                &staging_owner,
                admissions,
                Some(cap)
            ),
            Err(F15PlanAdmissionError::CarriageMismatch(
                F15ProviderRole::CodecF12
            ))
        ));

        let cap = destination(
            &context,
            attempt,
            staging_owner.clone(),
            "normalized://out/product.ec3",
            "staging://owner-4/attempt-4",
        );
        let mut admissions = complete_admissions(&context);
        let metadata = admissions.f13_metadata.as_mut().unwrap();
        metadata.binding.dependency_manifest_identity =
            F15DependencyManifestIdentity("unapproved-manifest");
        assert!(matches!(
            admit_for_test(
                context.clone(),
                attempt,
                &staging_owner,
                admissions,
                Some(cap)
            ),
            Err(F15PlanAdmissionError::DependencyManifestMismatch(
                F15ProviderRole::F13Metadata
            ))
        ));
    }

    #[test]
    fn plan_rejects_missing_destination_and_checked_count_overflow() {
        let context = parent_context(6_522_906, 0x11);
        let attempt = AttemptId::new(4);
        let staging_owner = OutputStagingOwnerId::new("stage-4").unwrap();
        assert!(matches!(
            admit_for_test(
                context.clone(),
                attempt,
                &staging_owner,
                complete_admissions(&context),
                None
            ),
            Err(F15PlanAdmissionError::DestinationCapabilityMissing)
        ));

        let overflow_context = parent_context(u64::MAX, 0x11);
        let overflow_owner = OutputStagingOwnerId::new("stage-overflow").unwrap();
        let overflow_destination = destination(
            &overflow_context,
            attempt,
            overflow_owner.clone(),
            "normalized://out/overflow.ec3",
            "staging://owner-overflow/attempt-4",
        );
        assert!(matches!(
            admit_for_test(
                overflow_context.clone(),
                attempt,
                &overflow_owner,
                complete_admissions(&overflow_context),
                Some(overflow_destination)
            ),
            Err(F15PlanAdmissionError::ArithmeticOverflow)
        ));

        let alias = ParentValidatedDestinationCapability::from_parent_validation(
            overflow_context.programme_id.clone(),
            attempt,
            overflow_owner,
            NormalizedFinalDestinationIdentity::from_parent_validated("same-identity").unwrap(),
            ParentReservedStagingNamespace::from_parent_reservation("same-identity").unwrap(),
        );
        assert!(matches!(
            alias,
            Err(F15PlanAdmissionError::DestinationAndStagingAlias)
        ));
    }

    #[test]
    fn private_plan_derives_checked_counts_and_output_projection_without_io() {
        let context = parent_context(6_522_906, 0x11);
        let attempt = AttemptId::new(4);
        let staging_owner = OutputStagingOwnerId::new("stage-4").unwrap();
        let cap = destination(
            &context,
            attempt,
            staging_owner.clone(),
            "normalized://out/product.ec3",
            "staging://owner-4/attempt-4",
        );
        let admissions = complete_admissions(&context);
        let plan = admit_for_test(
            context.clone(),
            attempt,
            &staging_owner,
            admissions,
            Some(cap),
        )
        .expect("all typed test values and pre-I/O capability bind to one parent input");
        assert_eq!(
            plan.extents.programme_counts.as_tuple(),
            (4_247, 486, 4_251, 4_249, 4_247, 1_050)
        );
        assert_eq!(plan.extents.published_output_bytes, 17_395_712);
        assert_eq!(
            plan.output_provider_projection
                .output_provider()
                .frame_bytes,
            4_096
        );
        assert_eq!(
            plan.output_provider_projection
                .output_provider()
                .suppressed_sources,
            2
        );
        assert_eq!(plan.profile_dimensions, EncodingProfile::selected());
        assert_eq!(
            plan.resolved_profile.carriage(),
            EmdfCarriageProfile::DeployedCompat
        );
        assert_eq!(
            plan.dependency_manifest_identity,
            F15DependencyManifestIdentity::selected()
        );
        assert_eq!(plan.destination.policy, FinalDestinationPolicy::NoOverwrite);
        assert_eq!(plan.providers.codec_f12.value.0, 1);
        assert_eq!(plan.providers.joc_f05jl.value.0, 2);
        assert_eq!(plan.providers.f13_metadata.value.0, 3);
        assert_eq!(plan.providers.f14_integrity.value.0, 4);
        assert_eq!(plan.programme_id.as_str(), "plan-test-programme");
        assert!(Arc::ptr_eq(
            &plan.prepared_input_identity,
            &plan.providers.codec_f12.binding.prepared_input_identity
        ));
    }
}
