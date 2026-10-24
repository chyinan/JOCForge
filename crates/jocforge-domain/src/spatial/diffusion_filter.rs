// pattern: Functional Core
//! One selected-profile F04F block preview. Tail creation, removal, callback scheduling, and parent commit remain caller-owned.

use std::fmt::Write as _;

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::frontend::{PackageManifestEvidence, ProviderFileEvidence, ProviderSha256};

const PACKAGE_BYTES: usize = 76_302;
const PACKAGE_SHA256: &str = "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const COEFFICIENT_PATH: &str = "parameters/diffusion-mathematical-v1.json";
const COEFFICIENT_BYTES: usize = 1_467;
const COEFFICIENT_SHA256: &str = "d690512cd204e97fc04a35c03519ed3005e26cb57408432fe30bfde63c2319aa";
const CONSTRUCTOR_PATH: &str = "parameters/diffusion-constructor-mathematical-v1.json";
const CONSTRUCTOR_BYTES: usize = 1_770;
const CONSTRUCTOR_SHA256: &str = "7a710d68b00c14d6b9b3f9fa3313579c5739ccce408655ea3252ad1ee4acd106";
const SAMPLE_RATE_HZ: u32 = 48_000;
const MAX_BLOCK_SAMPLES: usize = 4_096;
const STAGE_COUNT: usize = 4;
const LANE_COUNT: usize = 4;
const CONFIGURATION_COUNT: usize = 10;
const BANK_COUNT: usize = 4;
const PREFIX_COUNT: usize = 11;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiffusionFilterError {
    ObservedManifestEvidence,
    PackageManifestIdentity,
    PackageManifestSyntax,
    PackageManifestEntry,
    ObservedProviderEvidence,
    ProviderIdentity,
    ProviderSyntax,
    ProviderSchema,
    ParameterDomain,
    StateShape,
    InputShape,
    InputNonfinite,
    NumericFailure,
}

impl DiffusionFilterError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ObservedManifestEvidence => "DIFFUSION_PROVIDER_MANIFEST_EVIDENCE",
            Self::PackageManifestIdentity => "DIFFUSION_PROVIDER_MANIFEST_IDENTITY",
            Self::PackageManifestSyntax => "DIFFUSION_PROVIDER_MANIFEST_SYNTAX",
            Self::PackageManifestEntry => "DIFFUSION_PROVIDER_MANIFEST_ENTRY",
            Self::ObservedProviderEvidence => "DIFFUSION_PROVIDER_OBSERVED_EVIDENCE",
            Self::ProviderIdentity => "DIFFUSION_PROVIDER_IDENTITY",
            Self::ProviderSyntax => "DIFFUSION_PROVIDER_SYNTAX",
            Self::ProviderSchema => "DIFFUSION_PROVIDER_SCHEMA",
            Self::ParameterDomain => "DIFFUSION_PROVIDER_PARAMETER_DOMAIN",
            Self::StateShape => "F04_STATE_ERROR",
            Self::InputShape | Self::InputNonfinite => "F04_INPUT_ERROR",
            Self::NumericFailure => "F04_NUMERIC_FAILURE",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ManifestEntry {
    path: &'static str,
    bytes: usize,
    sha256: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedDiffusionFilterPackageManifest {
    manifest_sha256: ProviderSha256,
    coefficient: ManifestEntry,
    constructor: ManifestEntry,
}

impl ValidatedDiffusionFilterPackageManifest {
    pub const fn manifest_sha256(&self) -> ProviderSha256 {
        self.manifest_sha256
    }
}

/// Validates the approved package identity and both diffusion parameter entries before parsing provider JSON.
pub fn validate_diffusion_filter_package_manifest(
    evidence: &PackageManifestEvidence,
) -> Result<ValidatedDiffusionFilterPackageManifest, DiffusionFilterError> {
    let actual = ProviderSha256::digest(evidence.bytes());
    if evidence.observed_byte_length() != evidence.bytes().len() as u64
        || evidence.observed_sha256() != actual
    {
        return Err(DiffusionFilterError::ObservedManifestEvidence);
    }
    if evidence.bytes().len() != PACKAGE_BYTES || actual.to_hex() != PACKAGE_SHA256 {
        return Err(DiffusionFilterError::PackageManifestIdentity);
    }

    let document: Value = serde_json::from_slice(evidence.bytes())
        .map_err(|_| DiffusionFilterError::PackageManifestSyntax)?;
    let files = document
        .as_object()
        .and_then(|object| object.get("files"))
        .and_then(Value::as_array)
        .ok_or(DiffusionFilterError::PackageManifestEntry)?;

    let coefficient = manifest_entry(
        files,
        COEFFICIENT_PATH,
        COEFFICIENT_BYTES,
        COEFFICIENT_SHA256,
    )?;
    let constructor = manifest_entry(
        files,
        CONSTRUCTOR_PATH,
        CONSTRUCTOR_BYTES,
        CONSTRUCTOR_SHA256,
    )?;

    Ok(ValidatedDiffusionFilterPackageManifest {
        manifest_sha256: actual,
        coefficient,
        constructor,
    })
}

fn manifest_entry(
    files: &[Value],
    path: &'static str,
    expected_bytes: usize,
    expected_sha256: &'static str,
) -> Result<ManifestEntry, DiffusionFilterError> {
    let mut matches = files.iter().filter(|entry| {
        entry
            .as_object()
            .and_then(|object| object.get("file"))
            .and_then(Value::as_str)
            == Some(path)
    });
    let entry = matches
        .next()
        .and_then(Value::as_object)
        .ok_or(DiffusionFilterError::PackageManifestEntry)?;
    if matches.next().is_some() || entry.len() != 3 {
        return Err(DiffusionFilterError::PackageManifestEntry);
    }
    let bytes = entry
        .get("bytes")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(DiffusionFilterError::PackageManifestEntry)?;
    let sha256 = entry
        .get("sha256")
        .and_then(Value::as_str)
        .ok_or(DiffusionFilterError::PackageManifestEntry)?;
    if bytes != expected_bytes || sha256 != expected_sha256 {
        return Err(DiffusionFilterError::PackageManifestEntry);
    }
    Ok(ManifestEntry {
        path,
        bytes,
        sha256: expected_sha256,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiffusionFilterParameters {
    coefficients: [[f32; LANE_COUNT]; STAGE_COUNT],
    modes: [i32; PREFIX_COUNT],
    filter_lanes: [i32; PREFIX_COUNT],
    configuration_words: [u32; CONFIGURATION_COUNT],
    smoothing_words: [u32; BANK_COUNT],
    bank_lengths: [usize; BANK_COUNT],
    ring_length: usize,
}

impl DiffusionFilterParameters {
    pub const fn modes(&self) -> &[i32; PREFIX_COUNT] {
        &self.modes
    }

    pub const fn filter_lanes(&self) -> &[i32; PREFIX_COUNT] {
        &self.filter_lanes
    }

    pub const fn bank_lengths(&self) -> [u32; BANK_COUNT] {
        [
            self.bank_lengths[0] as u32,
            self.bank_lengths[1] as u32,
            self.bank_lengths[2] as u32,
            self.bank_lengths[3] as u32,
        ]
    }

    pub const fn ring_length(&self) -> u32 {
        self.ring_length as u32
    }

    pub fn cold_state(&self) -> Result<DiffusionFilterState, DiffusionFilterError> {
        if self.ring_length == 0
            || self.ring_length > MAX_BLOCK_SAMPLES
            || self
                .bank_lengths
                .iter()
                .any(|length| *length == 0 || *length > MAX_BLOCK_SAMPLES)
        {
            return Err(DiffusionFilterError::ParameterDomain);
        }
        let smoothing = self.smoothing_words.map(f32::from_bits);
        let slow_complement = r32_sub(1.0, smoothing[1])?;
        let fast_complement = r32_sub(1.0, smoothing[2])?;
        let coefficients = self.coefficients;
        let banks = self
            .bank_lengths
            .map(|length| vec![[0.0; LANE_COUNT]; length]);
        Ok(DiffusionFilterState {
            sample_rate_hz: SAMPLE_RATE_HZ,
            fade_pending: true,
            coefficients,
            banks,
            bank_indices: [0; BANK_COUNT],
            ring: vec![0.0; self.ring_length],
            ring_read_index: 0,
            ring_write_index: 0,
            gain0: 1.0,
            gain1: 1.0,
            gain_smoothing0: smoothing[0],
            gain_smoothing1: smoothing[0],
            slow_coefficient: smoothing[1],
            slow_complement,
            slow_accumulator: 0.0,
            fast_coefficient: smoothing[2],
            fast_complement,
            fast_accumulator: 0.0,
            input_coefficient: smoothing[3],
            input_accumulator: 0.0,
        })
    }

    fn envelope_epsilon(&self) -> f32 {
        f32::from_bits(self.configuration_words[8])
    }

    fn envelope_limit(&self) -> f32 {
        f32::from_bits(self.configuration_words[9])
    }
}

/// Parses the two identity-bound generated parameter assets into selected-profile values.
pub fn validate_diffusion_filter_parameters(
    coefficient_evidence: &ProviderFileEvidence,
    constructor_evidence: &ProviderFileEvidence,
    manifest: &ValidatedDiffusionFilterPackageManifest,
) -> Result<DiffusionFilterParameters, DiffusionFilterError> {
    let coefficient_bytes = validate_file_evidence(coefficient_evidence, manifest.coefficient)?;
    let constructor_bytes = validate_file_evidence(constructor_evidence, manifest.constructor)?;
    let coefficient_document: Value = serde_json::from_slice(coefficient_bytes)
        .map_err(|_| DiffusionFilterError::ProviderSyntax)?;
    let constructor_document: Value = serde_json::from_slice(constructor_bytes)
        .map_err(|_| DiffusionFilterError::ProviderSyntax)?;

    if string_field(&coefficient_document, "schema")? != "diffusion-mathematical-parameters.v1"
        || string_field(&coefficient_document, "route")? != "HADAMARD4_AND_PREFIX_PAIR_CYCLE_V1"
        || string_field(&constructor_document, "schema")?
            != "diffusion-constructor-mathematical-parameters.v1"
        || string_field(&constructor_document, "route")?
            != "DIFFUSION_FIXED_48K_CONSTRUCTOR_MATH_V1"
        || u64_field(&constructor_document, "sample_rate")? != u64::from(SAMPLE_RATE_HZ)
    {
        return Err(DiffusionFilterError::ProviderSchema);
    }

    let coefficient_rows = coefficient_document
        .get("coefficient_words")
        .and_then(Value::as_array)
        .filter(|rows| rows.len() == STAGE_COUNT)
        .ok_or(DiffusionFilterError::ProviderSchema)?;
    let mut coefficients = [[0.0_f32; LANE_COUNT]; STAGE_COUNT];
    for (stage, row) in coefficient_rows.iter().enumerate() {
        let words = parse_hex_words(row, LANE_COUNT)?;
        for (lane, word) in words.into_iter().enumerate() {
            coefficients[stage][lane] = f32::from_bits(word);
        }
    }
    if coefficients
        .iter()
        .flatten()
        .any(|value| !value.is_finite())
    {
        return Err(DiffusionFilterError::ParameterDomain);
    }

    let modes = parse_i32_array(coefficient_document.get("mode"), PREFIX_COUNT)?;
    let filter_lanes = parse_i32_array(coefficient_document.get("filter_lane"), PREFIX_COUNT)?;
    if modes.iter().any(|mode| !(-1..=1).contains(mode))
        || filter_lanes.iter().any(|lane| !(-1..=3).contains(lane))
    {
        return Err(DiffusionFilterError::ParameterDomain);
    }

    let configuration_words =
        parse_hex_array::<CONFIGURATION_COUNT>(constructor_document.get("configuration_words"))?;
    let bank_ratio_words =
        parse_hex_array::<BANK_COUNT>(constructor_document.get("bank_ratio_words"))?;
    let smoothing_argument_words =
        parse_hex_array::<BANK_COUNT>(constructor_document.get("smoothing_argument_words"))?;
    let smoothing_words =
        parse_hex_array::<BANK_COUNT>(constructor_document.get("smoothing_words"))?;
    let bank_lengths = parse_usize_array::<BANK_COUNT>(constructor_document.get("bank_lengths"))?;
    let ring_length = usize::try_from(u64_field(&constructor_document, "ring_length")?)
        .map_err(|_| DiffusionFilterError::ParameterDomain)?;
    let selected_floats = configuration_words
        .iter()
        .chain(bank_ratio_words.iter())
        .chain(smoothing_argument_words.iter())
        .chain(smoothing_words.iter())
        .copied()
        .map(f32::from_bits);
    if selected_floats.into_iter().any(|value| !value.is_finite())
        || f32::from_bits(configuration_words[0]) <= 0.0
        || f32::from_bits(configuration_words[1]) <= 0.0
        || f32::from_bits(configuration_words[2]) <= 0.0
        || f32::from_bits(configuration_words[3]) <= 0.0
        || f32::from_bits(configuration_words[4]) <= 0.0
        || f32::from_bits(configuration_words[5]) <= 0.0
        || f32::from_bits(configuration_words[6]) <= 0.0
        || f32::from_bits(configuration_words[7]) <= 0.0
        || f32::from_bits(configuration_words[8]) <= 0.0
        || f32::from_bits(configuration_words[9]) <= 0.0
        || bank_ratio_words
            .iter()
            .any(|word| f32::from_bits(*word) <= 0.0)
        || smoothing_words
            .iter()
            .any(|word| f32::from_bits(*word) <= 0.0 || f32::from_bits(*word) > 1.0)
        || bank_lengths
            .iter()
            .any(|length| *length == 0 || *length > MAX_BLOCK_SAMPLES)
        || ring_length == 0
        || ring_length > MAX_BLOCK_SAMPLES
    {
        return Err(DiffusionFilterError::ParameterDomain);
    }

    Ok(DiffusionFilterParameters {
        coefficients,
        modes,
        filter_lanes,
        configuration_words,
        smoothing_words,
        bank_lengths,
        ring_length,
    })
}

fn validate_file_evidence(
    evidence: &ProviderFileEvidence,
    entry: ManifestEntry,
) -> Result<&[u8], DiffusionFilterError> {
    let actual = ProviderSha256::digest(evidence.bytes());
    if evidence.observed_byte_length() != evidence.bytes().len() as u64
        || evidence.observed_sha256() != actual
    {
        return Err(DiffusionFilterError::ObservedProviderEvidence);
    }
    if evidence.bytes().len() != entry.bytes || actual.to_hex() != entry.sha256 {
        return Err(DiffusionFilterError::ProviderIdentity);
    }
    Ok(evidence.bytes())
}

fn string_field<'a>(document: &'a Value, key: &str) -> Result<&'a str, DiffusionFilterError> {
    document
        .as_object()
        .and_then(|object| object.get(key))
        .and_then(Value::as_str)
        .ok_or(DiffusionFilterError::ProviderSchema)
}

fn u64_field(document: &Value, key: &str) -> Result<u64, DiffusionFilterError> {
    document
        .as_object()
        .and_then(|object| object.get(key))
        .and_then(Value::as_u64)
        .ok_or(DiffusionFilterError::ProviderSchema)
}

fn parse_hex_words(value: &Value, count: usize) -> Result<Vec<u32>, DiffusionFilterError> {
    let entries = value
        .as_array()
        .filter(|entries| entries.len() == count)
        .ok_or(DiffusionFilterError::ProviderSchema)?;
    entries
        .iter()
        .map(|entry| {
            let word = entry
                .as_str()
                .filter(|word| word.len() == 8 && word.bytes().all(|b| b.is_ascii_hexdigit()))
                .ok_or(DiffusionFilterError::ProviderSchema)?;
            u32::from_str_radix(word, 16).map_err(|_| DiffusionFilterError::ProviderSchema)
        })
        .collect()
}

fn parse_hex_array<const N: usize>(
    value: Option<&Value>,
) -> Result<[u32; N], DiffusionFilterError> {
    value
        .ok_or(DiffusionFilterError::ProviderSchema)
        .and_then(|value| parse_hex_words(value, N))?
        .try_into()
        .map_err(|_| DiffusionFilterError::ProviderSchema)
}

fn parse_i32_array<const N: usize>(
    value: Option<&Value>,
    expected: usize,
) -> Result<[i32; N], DiffusionFilterError> {
    if expected != N {
        return Err(DiffusionFilterError::ProviderSchema);
    }
    let entries = value
        .and_then(Value::as_array)
        .filter(|entries| entries.len() == N)
        .ok_or(DiffusionFilterError::ProviderSchema)?;
    entries
        .iter()
        .map(|entry| {
            entry
                .as_i64()
                .and_then(|value| i32::try_from(value).ok())
                .ok_or(DiffusionFilterError::ProviderSchema)
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| DiffusionFilterError::ProviderSchema)
}

fn parse_usize_array<const N: usize>(
    value: Option<&Value>,
) -> Result<[usize; N], DiffusionFilterError> {
    let entries = value
        .and_then(Value::as_array)
        .filter(|entries| entries.len() == N)
        .ok_or(DiffusionFilterError::ProviderSchema)?;
    entries
        .iter()
        .map(|entry| {
            entry
                .as_u64()
                .and_then(|value| usize::try_from(value).ok())
                .ok_or(DiffusionFilterError::ProviderSchema)
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| DiffusionFilterError::ProviderSchema)
}

#[derive(Debug, PartialEq)]
pub struct DiffusionFilterState {
    sample_rate_hz: u32,
    fade_pending: bool,
    coefficients: [[f32; LANE_COUNT]; STAGE_COUNT],
    banks: [Vec<[f32; LANE_COUNT]>; BANK_COUNT],
    bank_indices: [usize; BANK_COUNT],
    ring: Vec<f32>,
    ring_read_index: usize,
    ring_write_index: usize,
    gain0: f32,
    gain1: f32,
    gain_smoothing0: f32,
    gain_smoothing1: f32,
    slow_coefficient: f32,
    slow_complement: f32,
    slow_accumulator: f32,
    fast_coefficient: f32,
    fast_complement: f32,
    fast_accumulator: f32,
    input_coefficient: f32,
    input_accumulator: f32,
}

impl DiffusionFilterState {
    pub const fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    pub fn bank_lengths(&self) -> [u32; BANK_COUNT] {
        [
            self.banks[0].len() as u32,
            self.banks[1].len() as u32,
            self.banks[2].len() as u32,
            self.banks[3].len() as u32,
        ]
    }

    pub fn ring_length(&self) -> u32 {
        self.ring.len() as u32
    }

    pub fn sha256_hex(&self) -> String {
        let digest = Sha256::digest(self.canonical_bytes());
        digest
            .iter()
            .fold(String::with_capacity(64), |mut output, byte| {
                write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
                output
            })
    }

    fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"DSTATE1\0");
        push_u32(&mut bytes, self.sample_rate_hz);
        push_u32(&mut bytes, u32::from(self.fade_pending));
        for stage in &self.coefficients {
            for lane in stage {
                push_f32(&mut bytes, *lane);
            }
        }
        for stage in 0..BANK_COUNT {
            push_u32(&mut bytes, self.banks[stage].len() as u32);
            push_u32(&mut bytes, self.bank_indices[stage] as u32);
            for entry in &self.banks[stage] {
                for lane in entry {
                    push_f32(&mut bytes, *lane);
                }
            }
        }
        push_u32(&mut bytes, self.ring.len() as u32);
        push_u32(&mut bytes, self.ring_read_index as u32);
        push_u32(&mut bytes, self.ring_write_index as u32);
        for value in &self.ring {
            push_f32(&mut bytes, *value);
        }
        for value in [
            self.gain0,
            self.gain1,
            self.gain_smoothing0,
            self.gain_smoothing1,
            self.slow_coefficient,
            self.slow_complement,
            self.slow_accumulator,
            self.fast_coefficient,
            self.fast_complement,
            self.fast_accumulator,
            self.input_coefficient,
            self.input_accumulator,
        ] {
            push_f32(&mut bytes, value);
        }
        bytes
    }

    pub(crate) fn validate(
        &self,
        parameters: &DiffusionFilterParameters,
    ) -> Result<(), DiffusionFilterError> {
        let smoothing = parameters.smoothing_words.map(f32::from_bits);
        let slow_complement = r32_sub(1.0, smoothing[1])?;
        let fast_complement = r32_sub(1.0, smoothing[2])?;
        if self.sample_rate_hz != SAMPLE_RATE_HZ
            || !same_f32_matrix(&self.coefficients, &parameters.coefficients)
            || self.bank_lengths() != parameters.bank_lengths.map(|length| length as u32)
            || self.ring.len() != parameters.ring_length
            || self.ring.is_empty()
            || self.ring.len() > MAX_BLOCK_SAMPLES
            || self.ring_read_index >= self.ring.len()
            || self.ring_write_index >= self.ring.len()
            || !same_f32(self.gain_smoothing0, smoothing[0])
            || !same_f32(self.gain_smoothing1, smoothing[0])
            || !same_f32(self.slow_coefficient, smoothing[1])
            || !same_f32(self.slow_complement, slow_complement)
            || !same_f32(self.fast_coefficient, smoothing[2])
            || !same_f32(self.fast_complement, fast_complement)
            || !same_f32(self.input_coefficient, smoothing[3])
            || self
                .bank_indices
                .iter()
                .zip(&self.banks)
                .any(|(index, bank)| {
                    *index >= bank.len() || bank.is_empty() || bank.len() > MAX_BLOCK_SAMPLES
                })
        {
            return Err(DiffusionFilterError::StateShape);
        }
        if self
            .coefficients
            .iter()
            .flatten()
            .any(|value| !value.is_finite())
            || self
                .banks
                .iter()
                .flat_map(|bank| bank.iter().flatten())
                .any(|value| !value.is_finite())
            || self.ring.iter().any(|value| !value.is_finite())
            || [
                self.gain0,
                self.gain1,
                self.gain_smoothing0,
                self.gain_smoothing1,
                self.slow_coefficient,
                self.slow_complement,
                self.slow_accumulator,
                self.fast_coefficient,
                self.fast_complement,
                self.fast_accumulator,
                self.input_coefficient,
                self.input_accumulator,
            ]
            .iter()
            .any(|value| !value.is_finite())
        {
            return Err(DiffusionFilterError::StateShape);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct DiffusionFilterProposal {
    lanes: [Vec<f32>; LANE_COUNT],
    next_state: DiffusionFilterState,
}

impl DiffusionFilterProposal {
    pub fn into_parts(self) -> ([Vec<f32>; LANE_COUNT], DiffusionFilterState) {
        (self.lanes, self.next_state)
    }
}

/// Computes one F04F preview from an immutable before-state. It never commits or mutates caller-owned state/input.
pub fn preview_filter_block(
    parameters: &DiffusionFilterParameters,
    before: &DiffusionFilterState,
    input: &[f32],
) -> Result<DiffusionFilterProposal, DiffusionFilterError> {
    before.validate(parameters)?;
    if input.len() > MAX_BLOCK_SAMPLES {
        return Err(DiffusionFilterError::InputShape);
    }
    if input.iter().any(|sample| !sample.is_finite()) {
        return Err(DiffusionFilterError::InputNonfinite);
    }
    if !parameters.envelope_epsilon().is_finite()
        || parameters.envelope_epsilon() <= 0.0
        || !parameters.envelope_limit().is_finite()
        || parameters.envelope_limit() <= 0.0
    {
        return Err(DiffusionFilterError::ParameterDomain);
    }

    let mut after = copy_state(before);
    let mut lanes = std::array::from_fn(|_| Vec::with_capacity(input.len()));
    if input.is_empty() {
        after.fade_pending = false;
        return Ok(DiffusionFilterProposal {
            lanes,
            next_state: after,
        });
    }

    let mut working = input.to_vec();
    if after.fade_pending {
        let half = working.len() / 2;
        if half > 0 {
            let denominator = half as f32;
            for (index, sample) in working[..half].iter_mut().enumerate() {
                let ramp = r32_div(index as f32, denominator)?;
                *sample = r32_mul(ramp, *sample)?;
            }
        }
    }
    after.fade_pending = false;

    for sample in working {
        let detected = r32_mul(
            r32_add(sample, after.input_accumulator)?,
            after.input_coefficient,
        )?;
        after.input_accumulator = r32_sub(detected, sample)?;

        after.gain1 = r32_add(
            r32_mul(r32_sub(after.gain1, 1.0)?, after.gain_smoothing1)?,
            1.0,
        )?;
        after.gain0 = r32_add(
            r32_mul(r32_sub(after.gain0, 1.0)?, after.gain_smoothing0)?,
            1.0,
        )?;

        let level = r32_add(detected.abs(), parameters.envelope_epsilon())?;
        let (fast_value, fast_saved) = envelope_step(
            level,
            after.fast_complement,
            after.fast_coefficient,
            after.fast_accumulator,
            parameters.envelope_epsilon(),
        )?;
        after.fast_accumulator = fast_saved;
        let (slow_value, slow_saved) = envelope_step(
            level,
            after.slow_complement,
            after.slow_coefficient,
            after.slow_accumulator,
            parameters.envelope_epsilon(),
        )?;
        after.slow_accumulator = slow_saved;

        let slow_limit = r32_mul(slow_value, parameters.envelope_limit())?;
        if r32_mul(fast_value, after.gain0)? > slow_limit {
            after.gain0 = r32_div(slow_limit, fast_value)?;
        }
        let gain1_limit = r32_div(
            r32_mul(fast_value, parameters.envelope_limit())?,
            slow_value,
        )?;
        if after.gain1 > gain1_limit {
            after.gain1 = gain1_limit;
        }

        let delayed = after.ring[after.ring_read_index];
        after.ring[after.ring_write_index] = sample;
        after.ring_read_index = (after.ring_read_index + 1) % after.ring.len();
        after.ring_write_index = (after.ring_write_index + 1) % after.ring.len();

        let mut values = [r32_mul(delayed, after.gain0)?; LANE_COUNT];
        for stage in 0..STAGE_COUNT {
            let bank_index = after.bank_indices[stage];
            let old_bank_values = after.banks[stage][bank_index];
            let mut new_bank_values = [0.0; LANE_COUNT];
            let mut next_values = [0.0; LANE_COUNT];
            for lane in 0..LANE_COUNT {
                let coefficient = after.coefficients[stage][lane];
                let numerator =
                    r32_add(values[lane], r32_mul(old_bank_values[lane], coefficient)?)?;
                next_values[lane] =
                    r32_sub(old_bank_values[lane], r32_mul(numerator, coefficient)?)?;
                new_bank_values[lane] = numerator;
            }
            after.banks[stage][bank_index] = new_bank_values;
            values = next_values;
            after.bank_indices[stage] = (bank_index + 1) % after.banks[stage].len();
        }
        for lane in 0..LANE_COUNT {
            lanes[lane].push(r32_mul(values[lane], after.gain1)?);
        }
    }

    after.validate(parameters)?;
    if lanes.iter().flatten().any(|value| !value.is_finite()) {
        return Err(DiffusionFilterError::NumericFailure);
    }
    Ok(DiffusionFilterProposal {
        lanes,
        next_state: after,
    })
}

fn envelope_step(
    level: f32,
    complement: f32,
    coefficient: f32,
    accumulator: f32,
    epsilon: f32,
) -> Result<(f32, f32), DiffusionFilterError> {
    let next = r32_add(
        r32_mul(level, complement)?,
        r32_mul(coefficient, accumulator)?,
    )?;
    let for_comparison = if next == 0.0 { epsilon } else { next };
    Ok((for_comparison, next))
}

fn copy_state(state: &DiffusionFilterState) -> DiffusionFilterState {
    DiffusionFilterState {
        sample_rate_hz: state.sample_rate_hz,
        fade_pending: state.fade_pending,
        coefficients: state.coefficients,
        banks: state.banks.clone(),
        bank_indices: state.bank_indices,
        ring: state.ring.clone(),
        ring_read_index: state.ring_read_index,
        ring_write_index: state.ring_write_index,
        gain0: state.gain0,
        gain1: state.gain1,
        gain_smoothing0: state.gain_smoothing0,
        gain_smoothing1: state.gain_smoothing1,
        slow_coefficient: state.slow_coefficient,
        slow_complement: state.slow_complement,
        slow_accumulator: state.slow_accumulator,
        fast_coefficient: state.fast_coefficient,
        fast_complement: state.fast_complement,
        fast_accumulator: state.fast_accumulator,
        input_coefficient: state.input_coefficient,
        input_accumulator: state.input_accumulator,
    }
}

fn r32_add(left: f32, right: f32) -> Result<f32, DiffusionFilterError> {
    checked(left + right)
}

fn r32_sub(left: f32, right: f32) -> Result<f32, DiffusionFilterError> {
    r32_add(left, -right)
}

fn r32_mul(left: f32, right: f32) -> Result<f32, DiffusionFilterError> {
    checked(left * right)
}

fn r32_div(left: f32, right: f32) -> Result<f32, DiffusionFilterError> {
    checked(left / right)
}

fn checked(value: f32) -> Result<f32, DiffusionFilterError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(DiffusionFilterError::NumericFailure)
    }
}

fn same_f32(left: f32, right: f32) -> bool {
    left.to_bits() == right.to_bits()
}

fn same_f32_matrix(
    left: &[[f32; LANE_COUNT]; STAGE_COUNT],
    right: &[[f32; LANE_COUNT]; STAGE_COUNT],
) -> bool {
    left.iter()
        .flatten()
        .zip(right.iter().flatten())
        .all(|(left, right)| same_f32(*left, *right))
}

fn push_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn push_f32(output: &mut Vec<u8>, value: f32) {
    output.extend_from_slice(&value.to_bits().to_le_bytes());
}

#[cfg(test)]
mod tests;
