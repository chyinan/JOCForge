//! Pure gain-row transition and 32-sample object-to-carrier mixing.
//!
//! Functional Core: this module validates complete value inputs and returns an
//! owned staged result. It performs no I/O and does not commit owner state.

const OUTPUT_CHANNELS: usize = 6;
const BLOCK_SAMPLES: usize = 32;
const MAX_GAIN_ROWS: usize = 128;
const MAX_INTERPOLATION_BLOCKS: u32 = 2_048;
const GAIN_ROWS_MAGIC: [u8; 8] = *b"OMIXROW1";

const EPSILON: f32 = 1.0_f32 / 1_000_000.0_f32;
const DELTA_GATE: f32 = 1.0_f32 / 10_000.0_f32;

/// The gain-row array whose value failed validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GainRowField {
    Current,
    Target,
    Step,
    Remaining,
}

/// Constructor-selected per-task output-filter setting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PerTaskOutputFilter {
    Disabled,
    Enabled,
}

/// A fail-closed error from gain-row construction or staged mixing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObjectGainMixError {
    WrongGainRowFieldLength {
        field: GainRowField,
        expected: usize,
        actual: usize,
    },
    NonFiniteGainRowValue {
        field: GainRowField,
        channel: usize,
    },
    GainRowValueOutsideF32 {
        field: GainRowField,
        channel: usize,
    },
    RemainingOutsideU32 {
        channel: usize,
    },
    TooManyGainRows {
        maximum: usize,
        actual: usize,
    },
    SourceCountMismatch {
        gain_rows: usize,
        source_blocks: usize,
    },
    NonFiniteSourceSample {
        source: usize,
        sample: usize,
    },
    InvalidUpdateFlag {
        actual: u8,
    },
    InterpolationBlocksOutOfRange {
        maximum: u32,
        actual: u32,
    },
    UnsupportedPerTaskOutputFilter,
}

/// One source's six-channel gain-ramp state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GainRow {
    current: [f32; OUTPUT_CHANNELS],
    target: [f32; OUTPUT_CHANNELS],
    step: [f32; OUTPUT_CHANNELS],
    remaining: [u32; OUTPUT_CHANNELS],
}

impl GainRow {
    /// Returns the approved all-positive-zero cold state.
    #[must_use]
    pub const fn cold() -> Self {
        Self {
            current: [0.0; OUTPUT_CHANNELS],
            target: [0.0; OUTPUT_CHANNELS],
            step: [0.0; OUTPUT_CHANNELS],
            remaining: [0; OUTPUT_CHANNELS],
        }
    }

    /// Builds a complete row for cold-state restoration and isolated tests.
    pub fn try_from_parts(
        current: &[f64],
        target: &[f64],
        step: &[f64],
        remaining: &[u64],
    ) -> Result<Self, ObjectGainMixError> {
        let current = convert_gain_values(current, GainRowField::Current)?;
        let target = convert_gain_values(target, GainRowField::Target)?;
        let step = convert_gain_values(step, GainRowField::Step)?;
        let remaining = convert_remaining(remaining)?;

        Ok(Self {
            current,
            target,
            step,
            remaining,
        })
    }

    /// Returns a new row with a complete six-channel target installed.
    ///
    /// Current, step, and remaining are preserved exactly.
    pub fn with_target(self, target: &[f64]) -> Result<Self, ObjectGainMixError> {
        Ok(Self {
            target: convert_gain_values(target, GainRowField::Target)?,
            ..self
        })
    }

    #[must_use]
    pub const fn current(&self) -> &[f32; OUTPUT_CHANNELS] {
        &self.current
    }

    #[must_use]
    pub const fn target(&self) -> &[f32; OUTPUT_CHANNELS] {
        &self.target
    }

    #[must_use]
    pub const fn step(&self) -> &[f32; OUTPUT_CHANNELS] {
        &self.step
    }

    #[must_use]
    pub const fn remaining(&self) -> &[u32; OUTPUT_CHANNELS] {
        &self.remaining
    }

    fn validate(&self) -> Result<(), ObjectGainMixError> {
        validate_f32_values(&self.current, GainRowField::Current)?;
        validate_f32_values(&self.target, GainRowField::Target)?;
        validate_f32_values(&self.step, GainRowField::Step)
    }
}

impl Default for GainRow {
    fn default() -> Self {
        Self::cold()
    }
}

/// The numeric branch taken for one non-skipped source/channel pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GainMixBranch {
    Interpolated,
    ConstantMultiplyAdd,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GainMixSourceIndex(u8);

impl GainMixSourceIndex {
    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GainMixChannelIndex(u8);

impl GainMixChannelIndex {
    pub const fn value(self) -> u8 {
        self.0
    }
}

/// Branch evidence in exact source-major, channel-major execution order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GainMixProof {
    source: GainMixSourceIndex,
    channel: GainMixChannelIndex,
    branch: GainMixBranch,
    remaining_after: u32,
}

impl GainMixProof {
    #[must_use]
    pub const fn source(&self) -> GainMixSourceIndex {
        self.source
    }

    #[must_use]
    pub const fn channel(&self) -> GainMixChannelIndex {
        self.channel
    }

    #[must_use]
    pub const fn branch(&self) -> GainMixBranch {
        self.branch
    }

    #[must_use]
    pub const fn remaining_after(&self) -> u32 {
        self.remaining_after
    }
}

/// Owned result of one successfully staged 32-sample transition.
#[derive(Debug, PartialEq)]
pub struct StagedObjectGainMix {
    output: [[f32; BLOCK_SAMPLES]; OUTPUT_CHANNELS],
    rows_after: Vec<GainRow>,
    proof: Vec<GainMixProof>,
}

impl StagedObjectGainMix {
    #[must_use]
    pub const fn output(&self) -> &[[f32; BLOCK_SAMPLES]; OUTPUT_CHANNELS] {
        &self.output
    }

    #[must_use]
    pub fn rows_after(&self) -> &[GainRow] {
        &self.rows_after
    }

    #[must_use]
    pub fn proof(&self) -> &[GainMixProof] {
        &self.proof
    }

    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        [[f32; BLOCK_SAMPLES]; OUTPUT_CHANNELS],
        Vec<GainRow>,
        Vec<GainMixProof>,
    ) {
        (self.output, self.rows_after, self.proof)
    }
}

/// Stages one output-filter-disabled 32-sample mix without mutating `rows_before`.
pub fn stage_object_gain_mix(
    rows_before: &[GainRow],
    source_blocks: &[[f32; BLOCK_SAMPLES]],
    update_flag: u8,
    interpolation_blocks: u32,
    output_filter: PerTaskOutputFilter,
) -> Result<StagedObjectGainMix, ObjectGainMixError> {
    validate_mix_inputs(
        rows_before,
        source_blocks,
        update_flag,
        interpolation_blocks,
        output_filter,
    )?;

    let mut output = [[0.0_f32; BLOCK_SAMPLES]; OUTPUT_CHANNELS];
    let mut rows_after = rows_before.to_vec();
    let mut proof = Vec::with_capacity(rows_before.len() * OUTPUT_CHANNELS);

    for (source, (row, samples)) in rows_after.iter_mut().zip(source_blocks.iter()).enumerate() {
        for (channel, output_channel) in output.iter_mut().enumerate() {
            let previous = row.current[channel];
            let target = row.target[channel];

            if target.abs() < EPSILON && previous.abs() < EPSILON {
                continue;
            }

            let difference = subtract_r32(target, previous);
            if difference.abs() >= DELTA_GATE {
                if update_flag == 1 {
                    row.remaining[channel] = interpolation_blocks;
                    if interpolation_blocks > 0 {
                        let reciprocal = divide_r32(1.0, interpolation_blocks as f32);
                        row.step[channel] = multiply_r32(difference, reciprocal);
                    }
                }
            } else {
                row.remaining[channel] = 0;
                row.current[channel] = target;
            }

            if row.remaining[channel] > 0 {
                for (sample, source_sample) in samples.iter().copied().enumerate() {
                    let ramp = divide_r32(sample as f32, BLOCK_SAMPLES as f32);
                    let ramp_step = multiply_r32(ramp, row.step[channel]);
                    let gain = add_r32(ramp_step, row.current[channel]);
                    let weighted_sample = multiply_r32(gain, source_sample);
                    output_channel[sample] = add_r32(weighted_sample, output_channel[sample]);
                }
                row.current[channel] = add_r32(row.current[channel], row.step[channel]);
                row.remaining[channel] -= 1;
                proof.push(GainMixProof {
                    source: GainMixSourceIndex(source as u8),
                    channel: GainMixChannelIndex(channel as u8),
                    branch: GainMixBranch::Interpolated,
                    remaining_after: row.remaining[channel],
                });
            } else {
                for (sample, source_sample) in samples.iter().copied().enumerate() {
                    let weighted_sample = multiply_r32(target, source_sample);
                    output_channel[sample] = add_r32(weighted_sample, output_channel[sample]);
                }
                row.current[channel] = target;
                proof.push(GainMixProof {
                    source: GainMixSourceIndex(source as u8),
                    channel: GainMixChannelIndex(channel as u8),
                    branch: GainMixBranch::ConstantMultiplyAdd,
                    remaining_after: 0,
                });
            }
        }
    }

    Ok(StagedObjectGainMix {
        output,
        rows_after,
        proof,
    })
}

/// Serializes complete gain rows using the approved canonical row-state layout.
pub fn canonical_gain_rows_bytes(rows: &[GainRow]) -> Result<Vec<u8>, ObjectGainMixError> {
    validate_rows(rows)?;

    let mut bytes = Vec::with_capacity(12 + rows.len() * 96);
    bytes.extend_from_slice(&GAIN_ROWS_MAGIC);
    bytes.extend_from_slice(&(rows.len() as u32).to_le_bytes());
    for row in rows {
        for values in [&row.current, &row.target, &row.step] {
            for value in values {
                bytes.extend_from_slice(&value.to_bits().to_le_bytes());
            }
        }
        for remaining in row.remaining {
            bytes.extend_from_slice(&remaining.to_le_bytes());
        }
    }
    Ok(bytes)
}

/// Serializes a complete six-channel output block in channel/sample order.
#[must_use]
pub fn canonical_mix_output_bytes(output: &[[f32; BLOCK_SAMPLES]; OUTPUT_CHANNELS]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(OUTPUT_CHANNELS * BLOCK_SAMPLES * 4);
    for channel in output {
        for sample in channel {
            bytes.extend_from_slice(&sample.to_bits().to_le_bytes());
        }
    }
    bytes
}

fn validate_mix_inputs(
    rows: &[GainRow],
    source_blocks: &[[f32; BLOCK_SAMPLES]],
    update_flag: u8,
    interpolation_blocks: u32,
    output_filter: PerTaskOutputFilter,
) -> Result<(), ObjectGainMixError> {
    if output_filter != PerTaskOutputFilter::Disabled {
        return Err(ObjectGainMixError::UnsupportedPerTaskOutputFilter);
    }
    if update_flag > 1 {
        return Err(ObjectGainMixError::InvalidUpdateFlag {
            actual: update_flag,
        });
    }
    if interpolation_blocks > MAX_INTERPOLATION_BLOCKS {
        return Err(ObjectGainMixError::InterpolationBlocksOutOfRange {
            maximum: MAX_INTERPOLATION_BLOCKS,
            actual: interpolation_blocks,
        });
    }
    validate_rows(rows)?;
    if rows.len() != source_blocks.len() {
        return Err(ObjectGainMixError::SourceCountMismatch {
            gain_rows: rows.len(),
            source_blocks: source_blocks.len(),
        });
    }
    for (source, samples) in source_blocks.iter().enumerate() {
        for (sample, value) in samples.iter().enumerate() {
            if !value.is_finite() {
                return Err(ObjectGainMixError::NonFiniteSourceSample { source, sample });
            }
        }
    }
    Ok(())
}

fn validate_rows(rows: &[GainRow]) -> Result<(), ObjectGainMixError> {
    if rows.len() > MAX_GAIN_ROWS {
        return Err(ObjectGainMixError::TooManyGainRows {
            maximum: MAX_GAIN_ROWS,
            actual: rows.len(),
        });
    }
    for row in rows {
        row.validate()?;
    }
    Ok(())
}

fn convert_gain_values(
    values: &[f64],
    field: GainRowField,
) -> Result<[f32; OUTPUT_CHANNELS], ObjectGainMixError> {
    if values.len() != OUTPUT_CHANNELS {
        return Err(ObjectGainMixError::WrongGainRowFieldLength {
            field,
            expected: OUTPUT_CHANNELS,
            actual: values.len(),
        });
    }

    let mut converted = [0.0_f32; OUTPUT_CHANNELS];
    for (channel, value) in values.iter().copied().enumerate() {
        if !value.is_finite() {
            return Err(ObjectGainMixError::NonFiniteGainRowValue { field, channel });
        }
        let converted_value = value as f32;
        if !converted_value.is_finite() {
            return Err(ObjectGainMixError::GainRowValueOutsideF32 { field, channel });
        }
        converted[channel] = converted_value;
    }
    Ok(converted)
}

fn convert_remaining(values: &[u64]) -> Result<[u32; OUTPUT_CHANNELS], ObjectGainMixError> {
    if values.len() != OUTPUT_CHANNELS {
        return Err(ObjectGainMixError::WrongGainRowFieldLength {
            field: GainRowField::Remaining,
            expected: OUTPUT_CHANNELS,
            actual: values.len(),
        });
    }

    let mut converted = [0_u32; OUTPUT_CHANNELS];
    for (channel, value) in values.iter().copied().enumerate() {
        converted[channel] = u32::try_from(value)
            .map_err(|_| ObjectGainMixError::RemainingOutsideU32 { channel })?;
    }
    Ok(converted)
}

fn validate_f32_values(
    values: &[f32; OUTPUT_CHANNELS],
    field: GainRowField,
) -> Result<(), ObjectGainMixError> {
    for (channel, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(ObjectGainMixError::NonFiniteGainRowValue { field, channel });
        }
    }
    Ok(())
}

#[inline(never)]
fn add_r32(left: f32, right: f32) -> f32 {
    left + right
}

#[inline(never)]
fn subtract_r32(left: f32, right: f32) -> f32 {
    left - right
}

#[inline(never)]
fn multiply_r32(left: f32, right: f32) -> f32 {
    left * right
}

#[inline(never)]
fn divide_r32(left: f32, right: f32) -> f32 {
    left / right
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::*;

    const ZERO_TARGET: [f64; OUTPUT_CHANNELS] = [0.0; OUTPUT_CHANNELS];

    fn source_ramp() -> [f32; BLOCK_SAMPLES] {
        std::array::from_fn(|sample| (sample as f32 - 16.0) / 16.0)
    }

    fn digest(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    fn row_with_target(target: &[f64]) -> GainRow {
        GainRow::cold()
            .with_target(target)
            .expect("approved target must construct")
    }

    fn stage_one(
        rows: &[GainRow],
        update_flag: u8,
        interpolation_blocks: u32,
    ) -> StagedObjectGainMix {
        let sources = vec![source_ramp(); rows.len()];
        stage_object_gain_mix(
            rows,
            &sources,
            update_flag,
            interpolation_blocks,
            PerTaskOutputFilter::Disabled,
        )
        .expect("approved vector input must stage")
    }

    fn assert_transition_hashes(
        staged: &StagedObjectGainMix,
        output_sha256: &str,
        state_sha256: &str,
    ) {
        assert_eq!(
            digest(&canonical_mix_output_bytes(staged.output())),
            output_sha256
        );
        assert_eq!(
            digest(
                &canonical_gain_rows_bytes(staged.rows_after())
                    .expect("staged rows must serialize"),
            ),
            state_sha256
        );
    }

    #[test]
    fn cold_and_target_install_are_complete_pure_values() {
        let cold = GainRow::cold();
        assert_eq!(cold.current().map(f32::to_bits), [0; OUTPUT_CHANNELS]);
        assert_eq!(cold.target().map(f32::to_bits), [0; OUTPUT_CHANNELS]);
        assert_eq!(cold.step().map(f32::to_bits), [0; OUTPUT_CHANNELS]);
        assert_eq!(cold.remaining(), &[0; OUTPUT_CHANNELS]);

        let warm = GainRow::try_from_parts(
            &[0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
            &ZERO_TARGET,
            &[0.25, 0.0, 0.0, 0.0, 0.0, 0.0],
            &[7, 0, 0, 0, 0, 0],
        )
        .expect("valid warm row");
        let installed = warm
            .with_target(&[1.0, 0.5, 0.25, 0.0, 0.0, 0.0])
            .expect("valid target");

        assert_eq!(
            warm.current().map(f32::to_bits),
            installed.current().map(f32::to_bits)
        );
        assert_eq!(
            warm.step().map(f32::to_bits),
            installed.step().map(f32::to_bits)
        );
        assert_eq!(warm.remaining(), installed.remaining());
        assert_eq!(
            installed.target().map(f32::to_bits),
            [1.0, 0.5, 0.25, 0.0, 0.0, 0.0].map(f32::to_bits)
        );
    }

    #[test]
    fn approved_ramp_and_threshold_vectors_match() {
        let ramp48_targets = [
            [1.0, 0.5, 0.0, 0.0, 0.0, 0.0],
            [0.25, 0.0, 0.75, 0.0, 0.0, 0.0],
        ];
        let mut ramp48_rows: Vec<_> = ramp48_targets.iter().map(|v| row_with_target(v)).collect();
        let first = stage_one(&ramp48_rows, 1, 48);
        assert_transition_hashes(
            &first,
            "77138a3a15200cb46b6b64e9c7675010c6c4047e5d01245fb05e9ff621030538",
            "ea16508ee91ec7a40ee7e28a116562e889eb4e80d06759f3c6b44d48808bcda8",
        );
        ramp48_rows = first.into_parts().1;
        let mut last = None;
        for _ in 1..50 {
            let staged = stage_one(&ramp48_rows, 0, 48);
            ramp48_rows = staged.rows_after().to_vec();
            last = Some(staged);
        }
        assert_transition_hashes(
            last.as_ref().expect("fifty blocks include a final block"),
            "48c955b5e39d7bf43c7e0e43fb884f9c2be7ba7eafe920803d4aab2313dfc1e1",
            "fe58f1cf3da5faeb662404eff1118925e21b4515d295f9f26b8a81e70cd9dddf",
        );

        let below_delta = DELTA_GATE.next_down();
        let staged = stage_one(
            &[row_with_target(&[
                f64::from(below_delta),
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
            ])],
            1,
            48,
        );
        assert_transition_hashes(
            &staged,
            "67cbe44bd9095d4e725c1856542bbd6370c7cc75093d3def70267c5e520fcbf1",
            "ccd7467b5fd6068aa89c2fdb276f4006f3d8f7c49216b79fdc9ff81fc2d263d6",
        );
        assert_eq!(
            staged.proof()[0].branch(),
            GainMixBranch::ConstantMultiplyAdd
        );

        let staged = stage_one(
            &[row_with_target(&[
                f64::from(DELTA_GATE),
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
            ])],
            1,
            48,
        );
        assert_transition_hashes(
            &staged,
            "38db29cd894945aaa687501290c973329413cf232ddaac9c4a2fdfba8c7c5214",
            "515fcd6d211a9e426c8f8d7f37a8ee74ebd11275f62699b4e0b181d280f3c059",
        );
        assert_eq!(staged.proof()[0].branch(), GainMixBranch::Interpolated);

        let staged = stage_one(
            &[row_with_target(&[
                f64::from(EPSILON.next_down()),
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
            ])],
            1,
            48,
        );
        assert_transition_hashes(
            &staged,
            "ef115a0e0c15cdc41958ca46b5b14b456115f4baec5e3ca68599d2a8f435e3b8",
            "abd8fc4ae3e13c5a461bd535e93e037f8550e640caa92f313f2e8d8663d69e36",
        );
        assert!(staged.proof().is_empty());

        let staged = stage_one(
            &[row_with_target(&[
                f64::from(EPSILON),
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
            ])],
            1,
            48,
        );
        assert_transition_hashes(
            &staged,
            "04cc004d58ece05ae84857b7d9aee5d0ebf4a87dc6f7062481e0efb92373ab53",
            "86d16411b2daf4d4bff440a14405b116a41abbae64b56154d155afc93dc047bc",
        );
        assert_eq!(
            staged.proof()[0].branch(),
            GainMixBranch::ConstantMultiplyAdd
        );
    }

    #[test]
    fn approved_immediate_short_retarget_and_warm_skip_vectors_match() {
        let unit_target = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        let immediate = stage_one(&[row_with_target(&unit_target)], 1, 0);
        assert_transition_hashes(
            &immediate,
            "86717cd39b1a0a73ff01193b2cbb917c0d5a9faf647f5892f0efdea640027cfa",
            "2bd7ee681c33ac8af0d4697205a6aaf297ffea64d44ca43f44f449a65f1e1cb5",
        );

        let mut short_rows = vec![row_with_target(&unit_target)];
        let first = stage_one(&short_rows, 1, 2);
        assert_transition_hashes(
            &first,
            "7087eb6407ad74cdd149804e913f2e7010d59b9ed692695fdef81b011f682952",
            "ee3bea7e067fd8b752f5d0858c5e4aa60a46a317c4f0dd41cfaedad378e8d5b2",
        );
        short_rows = first.into_parts().1;
        for _ in 1..4 {
            short_rows = stage_one(&short_rows, 0, 2).into_parts().1;
        }
        assert_eq!(
            digest(&canonical_gain_rows_bytes(&short_rows).expect("valid rows")),
            "99b19ccfd4df4ca3ae04f2e59556510c1d3aacfcebc312b0e9746e51648b57ee"
        );

        let mut retarget_rows = vec![row_with_target(&unit_target)];
        retarget_rows = stage_one(&retarget_rows, 1, 4).into_parts().1;
        retarget_rows[0] = retarget_rows[0]
            .with_target(&[0.5, 0.0, 0.0, 0.0, 0.0, 0.0])
            .expect("valid retarget");
        let mut retarget_last = stage_one(&retarget_rows, 1, 4);
        retarget_rows = retarget_last.rows_after().to_vec();
        for _ in 2..7 {
            retarget_last = stage_one(&retarget_rows, 0, 4);
            retarget_rows = retarget_last.rows_after().to_vec();
        }
        assert_transition_hashes(
            &retarget_last,
            "a3751f837eb1cc6474221c75653003b6e7b212cb4306061cf0b073e3d3962ed6",
            "ca499622ed9a68005a6c8bc787ac9e1544fcb77434e041e6584e6ad1f5c33753",
        );

        let warm = GainRow::try_from_parts(
            &[0.000_000_5, 0.0, 0.0, 0.0, 0.0, 0.0],
            &ZERO_TARGET,
            &[0.01, 0.0, 0.0, 0.0, 0.0, 0.0],
            &[7, 0, 0, 0, 0, 0],
        )
        .expect("valid warm row")
        .with_target(&[0.000_000_5, 0.0, 0.0, 0.0, 0.0, 0.0])
        .expect("valid warm target");
        let warm_staged = stage_one(&[warm], 1, 48);
        assert_transition_hashes(
            &warm_staged,
            "ef115a0e0c15cdc41958ca46b5b14b456115f4baec5e3ca68599d2a8f435e3b8",
            "dfd4d481a84b148aca83b7d8f39ef8716ce0b5f71a8d4b7fd312e828827b51b6",
        );
        assert!(warm_staged.proof().is_empty());
    }

    #[test]
    fn approved_no_update_ramp_down_and_reset_vectors_match() {
        let unit_target = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        let no_update = stage_one(&[row_with_target(&unit_target)], 0, 48);
        assert_transition_hashes(
            &no_update,
            "86717cd39b1a0a73ff01193b2cbb917c0d5a9faf647f5892f0efdea640027cfa",
            "2bd7ee681c33ac8af0d4697205a6aaf297ffea64d44ca43f44f449a65f1e1cb5",
        );
        assert_eq!(
            no_update.proof()[0].branch(),
            GainMixBranch::ConstantMultiplyAdd
        );

        let mut ramp_down_rows = vec![row_with_target(&unit_target)];
        ramp_down_rows = stage_one(&ramp_down_rows, 0, 2).into_parts().1;
        ramp_down_rows[0] = ramp_down_rows[0]
            .with_target(&ZERO_TARGET)
            .expect("zero target is valid");
        let mut ramp_down_last = stage_one(&ramp_down_rows, 1, 2);
        ramp_down_rows = ramp_down_last.rows_after().to_vec();
        for _ in 2..5 {
            ramp_down_last = stage_one(&ramp_down_rows, 0, 2);
            ramp_down_rows = ramp_down_last.rows_after().to_vec();
        }
        assert_transition_hashes(
            &ramp_down_last,
            "ef115a0e0c15cdc41958ca46b5b14b456115f4baec5e3ca68599d2a8f435e3b8",
            "bd409fe6456aaa273d8c4bd8f97825bfedca865c7ab53484dbfc7e1efb4fc684",
        );

        let mut reset_rows = vec![GainRow::cold()];
        let mut reset_last = None;
        for block in 0..5 {
            if block == 0 || block == 2 {
                reset_rows = vec![row_with_target(&unit_target)];
            }
            let staged = stage_one(&reset_rows, u8::from(block == 0 || block == 2), 2);
            reset_rows = staged.rows_after().to_vec();
            reset_last = Some(staged);
        }
        assert_transition_hashes(
            reset_last
                .as_ref()
                .expect("five blocks include a final block"),
            "86717cd39b1a0a73ff01193b2cbb917c0d5a9faf647f5892f0efdea640027cfa",
            "99b19ccfd4df4ca3ae04f2e59556510c1d3aacfcebc312b0e9746e51648b57ee",
        );
    }

    #[test]
    fn approved_ordered_task_sum_matches_explicit_washed_vector() {
        const EXPECTED_TASK_HASHES: [&str; 8] = [
            "4ce515108aca1f5ce8c376ad0e502781e850efa8dccf42c89e17b2dcd95101a4",
            "6a95a1a12ad2fce06bc6ea5e36da9e83285cde0b3cc231262c67d816e3f518db",
            "58e29a2c74f8630998f6cc62456b6407fdcf6877270ee75d4dae0ba1eec3519f",
            "c8d111c679a3da9195a0352877a79e6585c43130d4468e2f3a39e75e4e83c865",
            "73570d233e2d7d827aac4e830564cbc20079db34a85a242d824e77a3c1ce1e4d",
            "968fe4cb466914937111771dd7fe8a5ccf88d356cde1ce063891de6c136b4896",
            "390edd8f8116b3d546091fe11169bb4f9c06fcd95deafc3c9a06d55ff9b06ca7",
            "2dd1c1628e7bafa1b5bbe1e4f0b200b1f5f647fe52c796ee6c23e107a5af3ffe",
        ];

        assert_eq!((1.0_f32 / 48.0).to_bits(), 1_017_817_771);

        let mut rows = Vec::with_capacity(16);
        let mut sources = Vec::with_capacity(16);
        for source in 0..16 {
            let target: [f64; OUTPUT_CHANNELS] = std::array::from_fn(|channel| {
                f64::from((((source + 1) * (channel + 1)) % 7) as f32 / 8.0)
            });
            rows.push(row_with_target(&target));
            sources.push(std::array::from_fn(|sample| {
                ((source + 1) as f32 * (sample as f32 - 16.0)) / 64.0
            }));
        }

        assert_eq!(
            rows[0].target().map(f32::to_bits),
            [
                1_040_187_392,
                1_048_576_000,
                1_052_770_304,
                1_056_964_608,
                1_059_061_760,
                1_061_158_912,
            ]
        );
        assert_eq!(
            rows[15].target().map(f32::to_bits),
            [
                1_048_576_000,
                1_056_964_608,
                1_061_158_912,
                1_040_187_392,
                1_052_770_304,
                1_059_061_760,
            ]
        );
        assert_eq!(
            sources[0][..4]
                .iter()
                .copied()
                .map(f32::to_bits)
                .collect::<Vec<_>>(),
            [3_196_059_648, 3_195_011_072, 3_193_962_496, 3_192_913_920]
        );
        assert_eq!(
            sources[15][28..]
                .iter()
                .copied()
                .map(f32::to_bits)
                .collect::<Vec<_>>(),
            [1_077_936_128, 1_078_984_704, 1_080_033_280, 1_081_081_856]
        );

        let mut task_outputs = Vec::with_capacity(8);
        for (task, expected_hash) in EXPECTED_TASK_HASHES.iter().enumerate() {
            let start = task * 2;
            let staged = stage_object_gain_mix(
                &rows[start..start + 2],
                &sources[start..start + 2],
                1,
                48,
                PerTaskOutputFilter::Disabled,
            )
            .expect("approved task input must stage");
            assert_eq!(
                digest(&canonical_mix_output_bytes(staged.output())),
                *expected_hash
            );
            task_outputs.push(*staged.output());
        }

        let mut sum = [[0.0_f32; BLOCK_SAMPLES]; OUTPUT_CHANNELS];
        for task_output in task_outputs {
            for (sum_channel, task_channel) in sum.iter_mut().zip(task_output) {
                for (sum_sample, task_sample) in sum_channel.iter_mut().zip(task_channel) {
                    *sum_sample = add_r32(*sum_sample, task_sample);
                }
            }
        }

        assert_eq!(
            sum[0][..4]
                .iter()
                .copied()
                .map(f32::to_bits)
                .collect::<Vec<_>>(),
            [0, 3_152_740_352, 3_160_102_230, 3_164_127_232]
        );
        assert_eq!(
            digest(&canonical_mix_output_bytes(&sum)),
            "271901fe90348fce49a20dbe83f21781e9d582bf23558452256062a0c016116e"
        );
    }

    #[test]
    fn invalid_inputs_fail_before_any_state_can_be_committed() {
        let row = row_with_target(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let rows = [row];
        let sources = [source_ramp()];

        assert_eq!(
            stage_object_gain_mix(&rows, &sources, 2, 48, PerTaskOutputFilter::Disabled,),
            Err(ObjectGainMixError::InvalidUpdateFlag { actual: 2 })
        );
        assert_eq!(
            stage_object_gain_mix(&rows, &sources, 1, 2_049, PerTaskOutputFilter::Disabled,),
            Err(ObjectGainMixError::InterpolationBlocksOutOfRange {
                maximum: 2_048,
                actual: 2_049,
            })
        );
        assert_eq!(
            stage_object_gain_mix(&rows, &sources, 1, 48, PerTaskOutputFilter::Enabled,),
            Err(ObjectGainMixError::UnsupportedPerTaskOutputFilter)
        );
        assert_eq!(
            stage_object_gain_mix(&rows, &[], 1, 48, PerTaskOutputFilter::Disabled,),
            Err(ObjectGainMixError::SourceCountMismatch {
                gain_rows: 1,
                source_blocks: 0,
            })
        );

        let mut non_finite_sources = sources;
        non_finite_sources[0][7] = f32::NAN;
        assert_eq!(
            stage_object_gain_mix(
                &rows,
                &non_finite_sources,
                1,
                48,
                PerTaskOutputFilter::Disabled,
            ),
            Err(ObjectGainMixError::NonFiniteSourceSample {
                source: 0,
                sample: 7,
            })
        );
        assert_eq!(rows, [row]);

        assert_eq!(
            row.with_target(&[0.0; 5]),
            Err(ObjectGainMixError::WrongGainRowFieldLength {
                field: GainRowField::Target,
                expected: 6,
                actual: 5,
            })
        );
        assert_eq!(
            row.with_target(&[f64::INFINITY, 0.0, 0.0, 0.0, 0.0, 0.0]),
            Err(ObjectGainMixError::NonFiniteGainRowValue {
                field: GainRowField::Target,
                channel: 0,
            })
        );
        assert_eq!(
            row.with_target(&[f64::MAX, 0.0, 0.0, 0.0, 0.0, 0.0]),
            Err(ObjectGainMixError::GainRowValueOutsideF32 {
                field: GainRowField::Target,
                channel: 0,
            })
        );

        let empty = stage_object_gain_mix(&[], &[], 0, 0, PerTaskOutputFilter::Disabled)
            .expect("zero sources are approved");
        assert!(
            empty
                .output()
                .iter()
                .flatten()
                .all(|sample| sample.to_bits() == 0)
        );
        assert!(empty.rows_after().is_empty());
        assert!(empty.proof().is_empty());
    }
}
