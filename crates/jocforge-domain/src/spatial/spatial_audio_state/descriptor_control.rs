// pattern: Functional Core
//! F04D descriptor-control stage boundary. The selected index is staged before temporal3;
//! the one-shot result then binds the post-temporal3 gain-ramp input.

use super::{DescriptorState, RawWord, SpatialAudioJoinMetadata};
use crate::{
    numeric::{FiniteF32, r32_add, r32_mul, r32_sub},
    spatial::post_window_fanout::CallbackPhase,
    spatial::{DescriptorMathematicalProvider, QmfReceiverFunctionalProvider},
};

const LANE_COUNT: usize = 6;
const PEAK_LANES: [usize; 5] = [0, 1, 2, 4, 5];
const FRAME_SAMPLES: usize = 1_536;
const HEAD_SAMPLES: usize = 1_280;
const RAMP_SAMPLES: usize = 256;
const THRESHOLD_COUNT: usize = 127;
const GAIN_COUNT: usize = 128;
const QMF_PROVIDER_BYTES: u64 = 20_325;
const QMF_PROVIDER_SHA256: &str =
    "30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c";
const DESCRIPTOR_PROVIDER_BYTES: u64 = 4_861;
const DESCRIPTOR_PROVIDER_SHA256: &str =
    "a493fbdaf2dab30e940cae9976ed3305f3af083e5614ab0cbd6889c71a02b197";
const RISE_OFFSET_BITS: u32 = 0x3c00_0000;
const RAMP_START_BITS: u32 = 0x3d80_0000;
const RAMP_DELTA_BITS: u32 = 0x3b80_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(super) enum DescriptorError {
    ParameterUnavailable,
    ParameterInvalid,
    HoldRange,
    InputShape,
    InputNonFinite,
    NonFiniteIntermediate,
}

impl DescriptorError {
    #[allow(dead_code)]
    pub(super) const fn code(self) -> &'static str {
        match self {
            Self::ParameterUnavailable => "RECEIVER_PARAMETER_UNAVAILABLE",
            Self::ParameterInvalid => "RECEIVER_PARAMETER_INVALID",
            Self::HoldRange => "F04D_HOLD_RANGE",
            Self::InputShape => "F04D_INPUT_SHAPE",
            Self::InputNonFinite => "F04D_INPUT_NONFINITE",
            Self::NonFiniteIntermediate => "NONFINITE_INTERMEDIATE",
        }
    }
}

/// The validated upstream provider pair is reduced to only the F04D-selected fields.
#[allow(dead_code)]
pub(super) struct DescriptorParameterInput<'a> {
    selected_threshold_words: Option<&'a [u32]>,
    descriptor_scalar_words: Option<[u32; 6]>,
    gain_words: Option<&'a [u32]>,
    qmf_identity: Option<(u64, String)>,
    descriptor_identity: Option<(u64, String)>,
}

impl<'a> DescriptorParameterInput<'a> {
    #[allow(dead_code)]
    pub(super) fn from_validated_providers(
        receiver: Option<&'a QmfReceiverFunctionalProvider>,
        descriptor: Option<&'a DescriptorMathematicalProvider>,
    ) -> Self {
        let (selected_threshold_words, descriptor_scalar_words, qmf_identity) =
            if let Some(provider) = receiver {
                let scalars = provider.scalars();
                (
                    Some(provider.descriptor_threshold_words().as_slice()),
                    Some([
                        scalars.descriptor_level_offset(),
                        scalars.descriptor_level_linear(),
                        scalars.descriptor_level_quadratic(),
                        scalars.descriptor_headroom_step(),
                        scalars.descriptor_result_scale(),
                        scalars.descriptor_rise(),
                    ]),
                    Some((
                        provider.receipt().provider_file_bytes(),
                        provider.receipt().provider_file_sha256().to_hex(),
                    )),
                )
            } else {
                (None, None, None)
            };
        let (gain_words, descriptor_identity) = if let Some(provider) = descriptor {
            (
                Some(provider.gain_words().as_slice()),
                Some((
                    provider.receipt().provider_file_bytes(),
                    provider.receipt().provider_file_sha256().to_hex(),
                )),
            )
        } else {
            (None, None)
        };
        Self {
            selected_threshold_words,
            descriptor_scalar_words,
            gain_words,
            qmf_identity,
            descriptor_identity,
        }
    }
}

/// Callback-local view of F04R temporal0..2 scratch. Shape and finite checks remain ordered
/// inside F04D after required providers and the semantic hold field have been validated.
#[allow(dead_code)]
pub(super) struct DescriptorControlInputView<'a> {
    parent: &'a SpatialAudioJoinMetadata,
    phase: CallbackPhase,
    lanes: &'a [&'a [f32]],
}

impl<'a> DescriptorControlInputView<'a> {
    #[allow(dead_code)]
    pub(super) fn from_f04r_temporal2(
        parent: &'a SpatialAudioJoinMetadata,
        phase: CallbackPhase,
        lanes: &'a [&'a [f32]],
    ) -> Self {
        Self {
            parent,
            phase,
            lanes,
        }
    }
}

/// Non-Clone control result carried across F04R temporal3 and consumed by the ramp view.
#[must_use]
#[derive(Debug)]
#[allow(dead_code)]
pub(super) struct DescriptorControlSelection<'a> {
    parent: &'a SpatialAudioJoinMetadata,
    phase: CallbackPhase,
    index: u8,
    target_gain: RawWord,
    ramp_start_gain: RawWord,
    exponent: i64,
    after_tail_peak: RawWord,
    after_hold_count: i64,
    after_smoothed: RawWord,
    after_reference: RawWord,
    #[cfg(test)]
    control_trace: DescriptorControlTrace,
}

/// F04R temporal3 output tied to the exact pre-temporal3 selection result.
#[allow(dead_code)]
pub(super) struct DescriptorGainInputView<'parent, 'audio> {
    selection: DescriptorControlSelection<'parent>,
    lanes: &'audio [&'audio [f32]],
}

impl<'parent, 'audio> DescriptorGainInputView<'parent, 'audio> {
    #[allow(dead_code)]
    pub(super) fn from_f04r_temporal3(
        selection: DescriptorControlSelection<'parent>,
        lanes: &'audio [&'audio [f32]],
    ) -> Self {
        Self { selection, lanes }
    }
}

#[must_use]
#[allow(dead_code)]
pub(super) struct DescriptorStageProposal<'a> {
    parent: &'a SpatialAudioJoinMetadata,
    phase: CallbackPhase,
    selected_index: u8,
    s09_return: [Box<[FiniteF32]>; LANE_COUNT],
    after: DescriptorState,
    #[cfg(test)]
    control_trace: DescriptorControlTrace,
    #[cfg(test)]
    ramp_trace: DescriptorRampTrace,
}

impl<'a> DescriptorStageProposal<'a> {
    #[allow(dead_code)]
    pub(super) fn into_parts(
        self,
    ) -> (
        &'a SpatialAudioJoinMetadata,
        CallbackPhase,
        u8,
        [Box<[FiniteF32]>; LANE_COUNT],
        DescriptorState,
    ) {
        (
            self.parent,
            self.phase,
            self.selected_index,
            self.s09_return,
            self.after,
        )
    }
}

/// Stage one callback using only the owner-private descriptor state and validated provider pair.
/// `DescriptorState` has no external deserializer: the cold root seeds it and the successful
/// F04D after-proposal is its only transition. Finite peak/smoothing values, exponent/reference
/// preservation, and positive provider gain therefore remain closed owner invariants. The hold
/// field is checked explicitly because its semantic U32 range is narrower than canonical I64LE.
#[allow(dead_code)]
pub(super) fn stage_descriptor_control<'parent>(
    before: &DescriptorState,
    input: &DescriptorControlInputView<'parent>,
    parameters: &DescriptorParameterInput<'_>,
) -> Result<DescriptorControlSelection<'parent>, DescriptorError> {
    let parameters = validate_parameters(parameters)?;
    let hold = u32::try_from(before.hold_count).map_err(|_| DescriptorError::HoldRange)?;
    validate_input_shape(input.lanes)?;
    validate_input_finite(input.lanes)?;

    let old_tail_peak = finite_state_word(before.tail_peak)?;
    let old_smoothed = finite_state_word(before.smoothed)?;
    let old_reference = finite_state_word(before.reference)?;
    let (head, tail, tail_peak) = peak_for_lanes(input.lanes, old_tail_peak)?;
    let (level, level_trace) =
        level_from_peak(head, tail, tail_peak, parameters.descriptor_scalars)?;
    #[cfg(not(test))]
    let _ = &level_trace;
    let slope = r32_mul(
        r32_sub(level, old_reference).map_err(map_numeric_error)?,
        finite_constant(6.0)?,
    )
    .map_err(map_numeric_error)?;
    let candidate = r32_sub(
        finite_clamped(slope.value(), -1.0, 1.0)?,
        finite_word(RISE_OFFSET_BITS)?,
    )
    .map_err(map_numeric_error)?;
    let rise =
        r32_add(old_smoothed, parameters.descriptor_scalars[5]).map_err(map_numeric_error)?;
    let (next_hold, next_smoothed, hold_branch) = if rise.value() <= candidate.value() {
        if hold == 0 {
            (0, rise, "RELEASE_RISE")
        } else {
            (hold - 1, old_smoothed, "RELEASE_HOLD_DECREMENT")
        }
    } else {
        (5, candidate, "ATTACK")
    };
    #[cfg(not(test))]
    let _ = hold_branch;
    let lookup = finite_constant(next_smoothed.value().min(0.0))?;
    #[cfg(test)]
    let mut comparisons: Vec<DescriptorThresholdComparison> = Vec::with_capacity(THRESHOLD_COUNT);
    let mut ordinal = 0_usize;
    while ordinal < THRESHOLD_COUNT {
        let threshold_word = parameters.selected_threshold_words[ordinal];
        let threshold = finite_word(threshold_word)?;
        let strict_less = lookup.value() < threshold.value();
        #[cfg(test)]
        comparisons.push(DescriptorThresholdComparison {
            ordinal: ordinal as u64,
            lookup_u32: lookup.to_bits(),
            threshold_u32: threshold_word,
            strict_less,
        });
        if !strict_less {
            break;
        }
        ordinal += 1;
    }
    let target_gain_word = parameters.gain_words[ordinal];
    let target_gain = finite_word(target_gain_word)?;
    if target_gain.value() <= 0.0 {
        return Err(DescriptorError::ParameterInvalid);
    }
    let index = 128_u16 + u16::try_from(ordinal).map_err(|_| DescriptorError::ParameterInvalid)?;
    let index = u8::try_from(index).map_err(|_| DescriptorError::ParameterInvalid)?;
    let old_exponent = before.exponent;

    Ok(DescriptorControlSelection {
        parent: input.parent,
        phase: input.phase,
        index,
        target_gain: RawWord(target_gain_word.to_le_bytes()),
        ramp_start_gain: before.gain,
        exponent: old_exponent,
        after_tail_peak: RawWord(tail.to_bits().to_le_bytes()),
        after_hold_count: i64::from(next_hold),
        after_smoothed: RawWord(next_smoothed.to_bits().to_le_bytes()),
        after_reference: before.reference,
        #[cfg(test)]
        control_trace: DescriptorControlTrace {
            head_u32: level_trace.head_u32,
            tail_u32: level_trace.tail_u32,
            peak_u32: level_trace.peak_u32,
            clamped_peak_u32: level_trace.clamped_peak_u32,
            exponent_estimate: level_trace.exponent_estimate,
            shift: u64::from(level_trace.shift),
            n_u32: level_trace.n_u32,
            poly_u32: level_trace.poly_u32,
            level_u32: level.to_bits(),
            slope_u32: slope.to_bits(),
            candidate_u32: candidate.to_bits(),
            rise_u32: rise.to_bits(),
            hold_branch,
            lookup_u32: lookup.to_bits(),
            comparisons,
            ordinal: ordinal as u64,
            index: u64::from(index),
            target_gain_u32: target_gain_word,
        },
    })
}

#[allow(dead_code)]
pub(super) fn apply_descriptor_gain<'parent>(
    input: DescriptorGainInputView<'parent, '_>,
) -> Result<DescriptorStageProposal<'parent>, DescriptorError> {
    validate_input_shape(input.lanes)?;
    let DescriptorGainInputView { selection, lanes } = input;
    validate_input_finite(lanes)?;
    let target = finite_state_word(selection.target_gain)?;
    let mut current = finite_state_word(selection.ramp_start_gain)?;
    if target.value() <= 0.0 || current.value() <= 0.0 {
        return Err(DescriptorError::NonFiniteIntermediate);
    }
    let exponent =
        i32::try_from(selection.exponent).map_err(|_| DescriptorError::NonFiniteIntermediate)?;
    let scale = FiniteF32::new(2.0_f32.powi(exponent))
        .map_err(|_| DescriptorError::NonFiniteIntermediate)?;
    let ramp_increment = r32_mul(
        r32_sub(target, current).map_err(map_numeric_error)?,
        finite_word(RAMP_DELTA_BITS)?,
    )
    .map_err(map_numeric_error)?;
    let ramp_start = finite_word(RAMP_START_BITS)?;
    let mut output: [Vec<FiniteF32>; LANE_COUNT] =
        std::array::from_fn(|_| Vec::with_capacity(FRAME_SAMPLES));
    #[cfg(test)]
    let mut current_before_each_ramp_sample_u32 = Vec::with_capacity(RAMP_SAMPLES);

    for sample_index in 0..RAMP_SAMPLES {
        #[cfg(test)]
        current_before_each_ramp_sample_u32.push(current.to_bits());
        let alpha = finite_divide(ramp_start, current)?;
        for (lane_index, lane) in lanes.iter().enumerate() {
            let sample =
                FiniteF32::new(lane[sample_index]).map_err(|_| DescriptorError::InputNonFinite)?;
            let ramped = r32_mul(alpha, sample).map_err(map_numeric_error)?;
            let scaled = r32_mul(ramped, scale).map_err(map_numeric_error)?;
            output[lane_index].push(scaled);
        }
        current = r32_add(current, ramp_increment).map_err(map_numeric_error)?;
    }
    #[cfg(test)]
    let current_after_256_adds = current;
    let tail_alpha = finite_divide(ramp_start, target)?;
    for sample_index in RAMP_SAMPLES..FRAME_SAMPLES {
        for (lane_index, lane) in lanes.iter().enumerate() {
            let sample =
                FiniteF32::new(lane[sample_index]).map_err(|_| DescriptorError::InputNonFinite)?;
            let ramped = r32_mul(tail_alpha, sample).map_err(map_numeric_error)?;
            let scaled = r32_mul(ramped, scale).map_err(map_numeric_error)?;
            output[lane_index].push(scaled);
        }
    }
    let s09_return =
        std::array::from_fn(|lane| std::mem::take(&mut output[lane]).into_boxed_slice());
    let after = DescriptorState {
        tail_peak: selection.after_tail_peak,
        hold_count: selection.after_hold_count,
        smoothed: selection.after_smoothed,
        exponent: selection.exponent,
        reference: selection.after_reference,
        gain: selection.target_gain,
    };
    Ok(DescriptorStageProposal {
        parent: selection.parent,
        phase: selection.phase,
        selected_index: selection.index,
        s09_return,
        after,
        #[cfg(test)]
        control_trace: selection.control_trace,
        #[cfg(test)]
        ramp_trace: DescriptorRampTrace {
            scale_u32: scale.to_bits(),
            delta_u32: ramp_increment.to_bits(),
            current_before_each_ramp_sample_u32,
            current_after_256_adds_u32: current_after_256_adds.to_bits(),
            tail_alpha_u32: tail_alpha.to_bits(),
            committed_gain_u32: target.to_bits(),
            committed_gain_uses_target_not_current_after_256: true,
        },
    })
}

struct ValidatedParameters<'a> {
    selected_threshold_words: &'a [u32],
    descriptor_scalars: [FiniteF32; 6],
    gain_words: &'a [u32],
}

fn validate_parameters<'a>(
    parameters: &'a DescriptorParameterInput<'a>,
) -> Result<ValidatedParameters<'a>, DescriptorError> {
    let (Some(selected_threshold_words), Some(descriptor_scalar_words), Some(gain_words)) = (
        parameters.selected_threshold_words,
        parameters.descriptor_scalar_words,
        parameters.gain_words,
    ) else {
        return Err(DescriptorError::ParameterUnavailable);
    };
    let Some((qmf_bytes, qmf_sha256)) = parameters.qmf_identity.as_ref() else {
        return Err(DescriptorError::ParameterUnavailable);
    };
    let Some((descriptor_bytes, descriptor_sha256)) = parameters.descriptor_identity.as_ref()
    else {
        return Err(DescriptorError::ParameterUnavailable);
    };
    if *qmf_bytes != QMF_PROVIDER_BYTES
        || qmf_sha256 != QMF_PROVIDER_SHA256
        || *descriptor_bytes != DESCRIPTOR_PROVIDER_BYTES
        || descriptor_sha256 != DESCRIPTOR_PROVIDER_SHA256
        || selected_threshold_words.len() != THRESHOLD_COUNT
        || gain_words.len() != GAIN_COUNT
    {
        return Err(DescriptorError::ParameterInvalid);
    }
    if selected_threshold_words
        .iter()
        .chain(gain_words)
        .any(|word| !f32::from_bits(*word).is_finite())
        || gain_words.iter().any(|word| f32::from_bits(*word) <= 0.0)
    {
        return Err(DescriptorError::ParameterInvalid);
    }
    let mut descriptor_scalars = [finite_constant(0.0)?; 6];
    for (slot, word) in descriptor_scalars.iter_mut().zip(descriptor_scalar_words) {
        *slot = finite_word(word).map_err(|_| DescriptorError::ParameterInvalid)?;
    }
    Ok(ValidatedParameters {
        selected_threshold_words,
        descriptor_scalars,
        gain_words,
    })
}

#[allow(dead_code)]
struct LevelTrace {
    head_u32: u32,
    tail_u32: u32,
    peak_u32: u32,
    clamped_peak_u32: u32,
    exponent_estimate: i64,
    shift: u32,
    n_u32: Option<u32>,
    poly_u32: Option<u32>,
}

fn peak_for_lanes(
    lanes: &[&[f32]],
    old_tail_peak: FiniteF32,
) -> Result<(FiniteF32, FiniteF32, FiniteF32), DescriptorError> {
    let mut head_max = 0.0_f32;
    let mut tail_max = 0.0_f32;
    for lane_index in PEAK_LANES {
        for sample in &lanes[lane_index][..HEAD_SAMPLES] {
            head_max = head_max.max(sample.abs());
        }
        for sample in &lanes[lane_index][HEAD_SAMPLES..FRAME_SAMPLES] {
            tail_max = tail_max.max(sample.abs());
        }
    }
    let head = finite_constant(head_max)?;
    let tail = finite_constant(tail_max)?;
    let peak = finite_constant(old_tail_peak.value().max(head.value()).max(tail.value()))?;
    Ok((head, tail, peak))
}

fn level_from_peak(
    head: FiniteF32,
    tail: FiniteF32,
    tail_peak: FiniteF32,
    scalars: [FiniteF32; 6],
) -> Result<(FiniteF32, LevelTrace), DescriptorError> {
    let peak = tail_peak;
    let clamped = finite_clamped(peak.value(), -1.0, 1.0)?;
    let clamped_word = clamped.to_bits();
    let exponent_estimate = if clamped_word == 0 {
        -127
    } else {
        i64::try_from((2_u64 * u64::from(clamped_word)) >> 24)
            .map_err(|_| DescriptorError::NonFiniteIntermediate)?
            - 126
    };
    let shift = u32::try_from((-exponent_estimate).clamp(0, 60))
        .map_err(|_| DescriptorError::NonFiniteIntermediate)?;
    let (level, n_u32, poly_u32) = if peak.value() == 0.0 || shift >= 48 {
        (finite_constant(1.0)?, None, None)
    } else {
        let shift_scale_bits = (shift + 127) << 23;
        let shift_scale = finite_word(shift_scale_bits)?;
        let n = r32_mul(peak, shift_scale).map_err(map_numeric_error)?;
        let n_linear = r32_mul(n, scalars[1]).map_err(map_numeric_error)?;
        let offset_minus_linear = r32_sub(scalars[0], n_linear).map_err(map_numeric_error)?;
        let n_squared = r32_mul(n, n).map_err(map_numeric_error)?;
        let quadratic = r32_mul(n_squared, scalars[2]).map_err(map_numeric_error)?;
        let polynomial = r32_add(offset_minus_linear, quadratic).map_err(map_numeric_error)?;
        let polynomial_scale = r32_mul(polynomial, scalars[4]).map_err(map_numeric_error)?;
        let shift_headroom =
            r32_mul(finite_constant(shift as f32)?, scalars[3]).map_err(map_numeric_error)?;
        let level = r32_add(polynomial_scale, shift_headroom).map_err(map_numeric_error)?;
        (level, Some(n.to_bits()), Some(polynomial.to_bits()))
    };
    Ok((
        level,
        LevelTrace {
            head_u32: head.to_bits(),
            tail_u32: tail.to_bits(),
            peak_u32: peak.to_bits(),
            clamped_peak_u32: clamped.to_bits(),
            exponent_estimate,
            shift,
            n_u32,
            poly_u32,
        },
    ))
}

fn validate_input_shape(lanes: &[&[f32]]) -> Result<(), DescriptorError> {
    if lanes.len() != LANE_COUNT || lanes.iter().any(|lane| lane.len() != FRAME_SAMPLES) {
        return Err(DescriptorError::InputShape);
    }
    Ok(())
}

fn validate_input_finite(lanes: &[&[f32]]) -> Result<(), DescriptorError> {
    if lanes
        .iter()
        .any(|lane| lane.iter().any(|sample| !sample.is_finite()))
    {
        return Err(DescriptorError::InputNonFinite);
    }
    Ok(())
}

fn finite_state_word(word: RawWord) -> Result<FiniteF32, DescriptorError> {
    finite_word(u32::from_le_bytes(word.0))
}

fn finite_word(word: u32) -> Result<FiniteF32, DescriptorError> {
    FiniteF32::new(f32::from_bits(word)).map_err(|_| DescriptorError::NonFiniteIntermediate)
}

fn finite_constant(value: f32) -> Result<FiniteF32, DescriptorError> {
    FiniteF32::new(value).map_err(|_| DescriptorError::NonFiniteIntermediate)
}

fn finite_clamped(value: f32, min: f32, max: f32) -> Result<FiniteF32, DescriptorError> {
    finite_constant(value.clamp(min, max))
}

fn finite_divide(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, DescriptorError> {
    FiniteF32::new(left.value() / right.value()).map_err(|_| DescriptorError::NonFiniteIntermediate)
}

const fn map_numeric_error(_: crate::numeric::R32Error) -> DescriptorError {
    DescriptorError::NonFiniteIntermediate
}

#[cfg(test)]
#[derive(Debug)]
pub(super) struct DescriptorControlTrace {
    pub(super) head_u32: u32,
    pub(super) tail_u32: u32,
    pub(super) peak_u32: u32,
    pub(super) clamped_peak_u32: u32,
    pub(super) exponent_estimate: i64,
    pub(super) shift: u64,
    pub(super) n_u32: Option<u32>,
    pub(super) poly_u32: Option<u32>,
    pub(super) level_u32: u32,
    pub(super) slope_u32: u32,
    pub(super) candidate_u32: u32,
    pub(super) rise_u32: u32,
    pub(super) hold_branch: &'static str,
    pub(super) lookup_u32: u32,
    pub(super) comparisons: Vec<DescriptorThresholdComparison>,
    pub(super) ordinal: u64,
    pub(super) index: u64,
    pub(super) target_gain_u32: u32,
}

#[cfg(test)]
#[derive(Debug)]
pub(super) struct DescriptorThresholdComparison {
    pub(super) ordinal: u64,
    pub(super) lookup_u32: u32,
    pub(super) threshold_u32: u32,
    pub(super) strict_less: bool,
}

#[cfg(test)]
#[derive(Debug)]
pub(super) struct DescriptorRampTrace {
    pub(super) scale_u32: u32,
    pub(super) delta_u32: u32,
    pub(super) current_before_each_ramp_sample_u32: Vec<u32>,
    pub(super) current_after_256_adds_u32: u32,
    pub(super) tail_alpha_u32: u32,
    pub(super) committed_gain_u32: u32,
    pub(super) committed_gain_uses_target_not_current_after_256: bool,
}

#[cfg(test)]
mod tests;
