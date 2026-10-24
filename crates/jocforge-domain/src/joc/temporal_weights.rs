// pattern: Functional Core
//! Selected JOC temporal detector and 48-slot weight proposal.

use crate::numeric::{FiniteF32, r32_add, r32_mul};

const OBJECT_COUNT: usize = 15;
const WINDOW_SLOTS: usize = 48;
const CURRENT_SLOTS: usize = 24;
const COMPLEX_BINS: usize = 64;
const COMPLEX_COMPONENTS: usize = 2;
const ENERGY_HISTORY_SLOTS: usize = 16;
const FIRST_ENERGY_BIN: usize = 5;

const ZERO_BITS: u32 = 0x0000_0000;
const ONE_BITS: u32 = 0x3f80_0000;
const ONE_OVER_128_BITS: u32 = 0x3c00_0000;
const ONE_OVER_16_BITS: u32 = 0x3d80_0000;
const ONE_OVER_24_BITS: u32 = 0x3d2a_aaab;
const ONE_OVER_9_BITS: u32 = 0x3de3_8e39;
const ONE_E_MINUS_9_BITS: u32 = 0x3089_705f;
const SIXTEEN_BITS: u32 = 0x4180_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TemporalSegment {
    changed: bool,
    count: u8,
    start: u8,
    end: u8,
}

impl TemporalSegment {
    pub const fn changed(self) -> bool {
        self.changed
    }

    pub const fn count(self) -> u8 {
        self.count
    }

    pub const fn start(self) -> u8 {
        self.start
    }

    pub const fn end(self) -> u8 {
        self.end
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalWeightState {
    segment: TemporalSegment,
    energy_history_words: [[u32; ENERGY_HISTORY_SLOTS]; OBJECT_COUNT],
    last_changes: [u8; OBJECT_COUNT],
    first: bool,
    weight_words: [u32; WINDOW_SLOTS],
}

impl TemporalWeightState {
    pub const fn cold() -> Self {
        Self {
            segment: TemporalSegment {
                changed: false,
                count: 1,
                start: 23,
                end: 0,
            },
            energy_history_words: [[ZERO_BITS; ENERGY_HISTORY_SLOTS]; OBJECT_COUNT],
            last_changes: [0; OBJECT_COUNT],
            first: true,
            weight_words: [ZERO_BITS; WINDOW_SLOTS],
        }
    }

    pub const fn segment(&self) -> TemporalSegment {
        self.segment
    }

    pub const fn energy_history_words(&self) -> &[[u32; ENERGY_HISTORY_SLOTS]; OBJECT_COUNT] {
        &self.energy_history_words
    }

    pub const fn last_changes(&self) -> &[u8; OBJECT_COUNT] {
        &self.last_changes
    }

    pub const fn first(&self) -> bool {
        self.first
    }

    pub const fn weight_words(&self) -> &[u32; WINDOW_SLOTS] {
        &self.weight_words
    }

    pub fn canonical_payload_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1_229);
        for value in [
            i32::from(self.segment.changed),
            i32::from(self.segment.count),
            i32::from(self.segment.start),
            i32::from(self.segment.end),
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for word in self.energy_history_words.iter().flatten() {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        for value in self.last_changes {
            bytes.extend_from_slice(&i32::from(value).to_le_bytes());
        }
        bytes.push(u8::from(self.first));
        for word in self.weight_words {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes
    }

    /// Stages one detector transition from object-major 15x48x64 complex-F32
    /// windows. Only the current half (slots24..47) is consumed by the detector.
    #[allow(clippy::needless_range_loop)] // Detector search/history order is index-defined.
    pub fn stage_target_windows48(
        &self,
        target_words: &[u32],
    ) -> Result<TemporalWeightProposal, TemporalWeightError> {
        if target_words.len() != OBJECT_COUNT * WINDOW_SLOTS * COMPLEX_BINS * COMPLEX_COMPONENTS {
            return Err(TemporalWeightError::InputShape);
        }

        let mut energy_cache = [[None; CURRENT_SLOTS]; OBJECT_COUNT];
        let threshold = if self.first {
            finite_word(ZERO_BITS)?
        } else {
            multiply(finite_word(SIXTEEN_BITS)?, finite_word(ONE_E_MINUS_9_BITS)?)?
        };
        let mut last_changes = self.last_changes;
        let mut detected_position = None;

        'position: for position in ENERGY_HISTORY_SLOTS..ENERGY_HISTORY_SLOTS + CURRENT_SLOTS {
            for object in 0..OBJECT_COUNT {
                if position < usize::from(last_changes[object]) + 8 {
                    continue;
                }
                let current = current_energy(
                    target_words,
                    &mut energy_cache,
                    object,
                    position - ENERGY_HISTORY_SLOTS,
                )?;
                if current.value() < threshold.value() {
                    continue;
                }
                let mut average = finite_word(ZERO_BITS)?;
                for history_position in position - ENERGY_HISTORY_SLOTS..position {
                    let value = if history_position < ENERGY_HISTORY_SLOTS {
                        finite_word(self.energy_history_words[object][history_position])?
                    } else {
                        current_energy(
                            target_words,
                            &mut energy_cache,
                            object,
                            history_position - ENERGY_HISTORY_SLOTS,
                        )?
                    };
                    average = add(average, multiply(value, finite_word(ONE_OVER_16_BITS)?)?)?;
                }
                let reduced = multiply(current, finite_word(ONE_OVER_9_BITS)?)?;
                if reduced.value() > average.value() {
                    last_changes[object] =
                        u8::try_from(position).map_err(|_| TemporalWeightError::InputShape)?;
                    detected_position = Some(
                        u8::try_from(position - ENERGY_HISTORY_SLOTS)
                            .map_err(|_| TemporalWeightError::InputShape)?,
                    );
                    break 'position;
                }
            }
        }

        let segment = TemporalSegment {
            changed: detected_position.is_some(),
            count: 1,
            start: detected_position.unwrap_or(23),
            end: 0,
        };
        for change in &mut last_changes {
            *change = (*change).saturating_sub(24);
        }

        let mut energy_history_words = self.energy_history_words;
        for object in 0..OBJECT_COUNT {
            let change = usize::from(last_changes[object]);
            let history_start = if change + 8 < ENERGY_HISTORY_SLOTS {
                0
            } else {
                change - 8
            };
            if history_start < ENERGY_HISTORY_SLOTS {
                for history_slot in history_start..ENERGY_HISTORY_SLOTS {
                    energy_history_words[object][history_slot] =
                        current_energy(target_words, &mut energy_cache, object, history_slot + 8)?
                            .to_bits();
                }
            }
        }

        let weight_words = build_weights(self.segment, segment)?;
        let computed_energy_count = u16::try_from(
            energy_cache
                .iter()
                .flatten()
                .filter(|entry| entry.is_some())
                .count(),
        )
        .map_err(|_| TemporalWeightError::InputShape)?;
        Ok(TemporalWeightProposal {
            before_segment: self.segment,
            after: Self {
                segment,
                energy_history_words,
                last_changes,
                first: false,
                weight_words,
            },
            detected_position,
            threshold_word: threshold.to_bits(),
            computed_energy_count,
        })
    }
}

impl Default for TemporalWeightState {
    fn default() -> Self {
        Self::cold()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TemporalWeightProposal {
    before_segment: TemporalSegment,
    after: TemporalWeightState,
    detected_position: Option<u8>,
    threshold_word: u32,
    computed_energy_count: u16,
}

impl TemporalWeightProposal {
    pub const fn before_segment(&self) -> TemporalSegment {
        self.before_segment
    }

    pub const fn after(&self) -> &TemporalWeightState {
        &self.after
    }

    pub const fn detected_position(&self) -> Option<u8> {
        self.detected_position
    }

    pub const fn threshold_word(&self) -> u32 {
        self.threshold_word
    }

    pub const fn computed_energy_count(&self) -> u16 {
        self.computed_energy_count
    }

    pub fn into_after(self) -> TemporalWeightState {
        self.after
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemporalWeightError {
    InputShape,
    InputNonFinite,
    NumericNonFinite,
}

impl TemporalWeightError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputShape => "JOC_TEMPORAL_INPUT_SHAPE",
            Self::InputNonFinite => "JOC_TEMPORAL_INPUT_NONFINITE",
            Self::NumericNonFinite => "JOC_TEMPORAL_NUMERIC_NONFINITE",
        }
    }
}

impl std::fmt::Display for TemporalWeightError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for TemporalWeightError {}

fn current_energy(
    target_words: &[u32],
    cache: &mut [[Option<u32>; CURRENT_SLOTS]; OBJECT_COUNT],
    object: usize,
    current_slot: usize,
) -> Result<FiniteF32, TemporalWeightError> {
    if let Some(word) = cache[object][current_slot] {
        return finite_word(word);
    }
    let mut energy = finite_word(ZERO_BITS)?;
    for bin in FIRST_ENERGY_BIN..COMPLEX_BINS {
        for component in 0..COMPLEX_COMPONENTS {
            let offset = (((object * WINDOW_SLOTS + CURRENT_SLOTS + current_slot) * COMPLEX_BINS
                + bin)
                * COMPLEX_COMPONENTS)
                + component;
            let value = finite_word(target_words[offset])?;
            let square = multiply(value, value)?;
            let term = multiply(square, finite_word(ONE_OVER_128_BITS)?)?;
            energy = add(term, energy)?;
        }
    }
    cache[object][current_slot] = Some(energy.to_bits());
    Ok(energy)
}

fn build_weights(
    previous: TemporalSegment,
    current: TemporalSegment,
) -> Result<[u32; WINDOW_SLOTS], TemporalWeightError> {
    let mut up = [ZERO_BITS; CURRENT_SLOTS];
    let mut value = finite_word(ZERO_BITS)?;
    let step = finite_word(ONE_OVER_24_BITS)?;
    for word in up.iter_mut().take(CURRENT_SLOTS - 1) {
        value = add(value, step)?;
        *word = value.to_bits();
    }
    up[CURRENT_SLOTS - 1] = ONE_BITS;

    let mut down = [ZERO_BITS; CURRENT_SLOTS];
    for (destination, source) in down
        .iter_mut()
        .take(CURRENT_SLOTS - 1)
        .zip(up.iter().take(CURRENT_SLOTS - 1).rev())
    {
        *destination = *source;
    }

    let mut weights = [ZERO_BITS; WINDOW_SLOTS];
    for slot in 0..CURRENT_SLOTS {
        weights[slot] = if previous.changed {
            if slot >= usize::from(previous.start) {
                ONE_BITS
            } else {
                ZERO_BITS
            }
        } else {
            up[slot]
        };
        weights[CURRENT_SLOTS + slot] = if current.changed {
            if slot < usize::from(current.start) {
                ONE_BITS
            } else {
                ZERO_BITS
            }
        } else {
            down[slot]
        };
    }
    Ok(weights)
}

fn finite_word(word: u32) -> Result<FiniteF32, TemporalWeightError> {
    FiniteF32::new(f32::from_bits(word)).map_err(|_| TemporalWeightError::InputNonFinite)
}

fn multiply(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, TemporalWeightError> {
    r32_mul(left, right).map_err(|_| TemporalWeightError::NumericNonFinite)
}

fn add(left: FiniteF32, right: FiniteF32) -> Result<FiniteF32, TemporalWeightError> {
    r32_add(left, right).map_err(|_| TemporalWeightError::NumericNonFinite)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use sha2::{Digest, Sha256};

    #[test]
    fn cold_temporal_payload_matches_the_approved_family_identity() {
        let bytes = TemporalWeightState::cold().canonical_payload_bytes();
        assert_eq!(bytes.len(), 1_229);
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            "afa3ea6e59fb39634aa09884412c07dbeb6a3f081899837dc342a1f6f3205534"
        );
    }

    const FIXTURE_ROOT: &str = "tests/fixtures/synthetic_qmf_fit_checkpoints";
    const MANIFEST: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_qmf_fit_checkpoints/WASH-MANIFEST.json");
    const INPUT: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_qmf_fit_checkpoints/INPUT-ONLY.json");
    const EXPECTED: &[u8] =
        include_bytes!("../../tests/fixtures/synthetic_qmf_fit_checkpoints/EXPECTED.json");
    const PAYLOADS: &[(&str, &[u8])] = &[
        (
            "approved-huffman.json",
            include_bytes!(
                "../../tests/fixtures/synthetic_qmf_fit_checkpoints/approved-huffman.json"
            ),
        ),
        (
            "approved-joc-functional.json",
            include_bytes!(
                "../../tests/fixtures/synthetic_qmf_fit_checkpoints/approved-joc-functional.json"
            ),
        ),
        (
            "approved-joc-math.json",
            include_bytes!(
                "../../tests/fixtures/synthetic_qmf_fit_checkpoints/approved-joc-math.json"
            ),
        ),
        (
            "CONTRACT.md",
            include_bytes!("../../tests/fixtures/synthetic_qmf_fit_checkpoints/CONTRACT.md"),
        ),
        ("EXPECTED.json", EXPECTED),
        ("INPUT-ONLY.json", INPUT),
    ];

    #[test]
    fn washed_checkpoint_payloads_match_their_manifest() {
        let manifest: Value = serde_json::from_slice(MANIFEST).unwrap();
        assert_eq!(
            manifest["payloads"].as_array().unwrap().len(),
            PAYLOADS.len()
        );
        for entry in manifest["payloads"].as_array().unwrap() {
            let name = entry["file"].as_str().unwrap();
            let payload = PAYLOADS
                .iter()
                .find_map(|(candidate, bytes)| (*candidate == name).then_some(*bytes))
                .unwrap_or_else(|| panic!("unregistered fixture payload {name}"));
            assert_eq!(payload.len() as u64, entry["bytes"].as_u64().unwrap());
            assert_eq!(
                format!("{:x}", Sha256::digest(payload)),
                entry["sha256"].as_str().unwrap()
            );
        }
        assert!(FIXTURE_ROOT.ends_with("synthetic_qmf_fit_checkpoints"));
    }

    #[test]
    fn three_synthetic_generations_match_temporal_state_and_weight_checkpoints() {
        let input: Value = serde_json::from_slice(INPUT).unwrap();
        let expected: Value = serde_json::from_slice(EXPECTED).unwrap();
        let mut state = TemporalWeightState::cold();
        let mut previous = vec![ZERO_BITS; OBJECT_COUNT * CURRENT_SLOTS * COMPLEX_BINS * 2];

        for generation in 0..3 {
            let current = synthetic_target_current24(&input, generation);
            let windows = concatenate_object_windows(&previous, &current);
            let proposal = state.stage_target_windows48(&windows).unwrap();
            let checkpoint = &expected["checkpoints"][generation];
            assert_state(&state, &checkpoint["temporal"]["before"]);
            assert_state(proposal.after(), &checkpoint["temporal"]["after"]);
            assert_eq!(
                proposal.after().weight_words(),
                decode_u32_words(checkpoint["weights_f32le_hex"].as_str().unwrap()).as_slice()
            );
            assert_eq!(
                proposal.detected_position().into_iter().collect::<Vec<_>>(),
                checkpoint["temporal"]["trace"]["detected_positions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_u64().unwrap() as u8)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                u64::from(proposal.computed_energy_count()),
                checkpoint["temporal"]["trace"]["model_energy_cache_defined"]
                    .as_u64()
                    .unwrap()
            );
            assert_eq!(
                proposal.threshold_word(),
                f64_wrapper_to_f32_word(&checkpoint["temporal"]["trace"]["threshold"])
            );
            state = proposal.into_after();
            previous = current;
        }
    }

    fn assert_state(actual: &TemporalWeightState, expected: &Value) {
        let segment = &expected["segment"];
        assert_eq!(
            actual.segment.changed,
            segment["changed"].as_u64().unwrap() != 0
        );
        assert_eq!(
            u64::from(actual.segment.count),
            segment["count"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(actual.segment.start),
            segment["start"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(actual.segment.end),
            segment["end"].as_u64().unwrap()
        );
        assert_eq!(actual.first, expected["first"].as_bool().unwrap());
        for object in 0..OBJECT_COUNT {
            assert_eq!(
                u64::from(actual.last_changes[object]),
                expected["last_changes"][object].as_u64().unwrap()
            );
            for slot in 0..ENERGY_HISTORY_SLOTS {
                assert_eq!(
                    actual.energy_history_words[object][slot],
                    f64_wrapper_to_f32_word(&expected["energy_history"][object][slot]),
                    "object {object} history slot {slot}"
                );
            }
        }
        if let Some(weights) = expected.get("weights") {
            for slot in 0..WINDOW_SLOTS {
                assert_eq!(
                    actual.weight_words[slot],
                    f64_wrapper_to_f32_word(&weights[slot]),
                    "weight slot {slot}"
                );
            }
        }
    }

    fn synthetic_target_current24(input: &Value, generation: usize) -> Vec<u32> {
        let amplitude = input["generations"][generation]["amplitude"]
            .as_f64()
            .unwrap() as f32;
        let transient_slot = input["generations"][generation]["transient_slot"]
            .as_u64()
            .unwrap() as usize;
        let multiplier = input["generations"][generation]["transient_multiplier"]
            .as_f64()
            .unwrap() as f32;
        let mut output = Vec::with_capacity(OBJECT_COUNT * CURRENT_SLOTS * COMPLEX_BINS * 2);
        for object in 0..OBJECT_COUNT {
            let (alpha, beta) = match object % 3 {
                0 => (0.75_f32, 0.0_f32),
                1 => (0.75_f32, 0.5_f32),
                _ => (0.5_f32, -0.25_f32),
            };
            for slot in 0..CURRENT_SLOTS {
                for bin in 0..COMPLEX_BINS {
                    let carrier_a = synthetic_carrier(object % 5, slot, bin, amplitude);
                    let carrier_z = synthetic_carrier((object + 2) % 5, slot, bin, amplitude);
                    let scale = if generation == 1 && object == 0 && slot >= transient_slot {
                        multiplier
                    } else {
                        1.0
                    };
                    for component in 0..2 {
                        let first = alpha * carrier_a[component];
                        let second = beta * carrier_z[component];
                        let mixed = first + second;
                        output.push((mixed * scale).to_bits());
                    }
                }
            }
        }
        output
    }

    fn synthetic_carrier(carrier: usize, slot: usize, bin: usize, amplitude: f32) -> [f32; 2] {
        let bit_index = if carrier == 1 && bin == 0 { 0 } else { carrier };
        let factor = 1.0_f32 + (bin % 3) as f32 / 4.0_f32;
        let real_sign = if (slot >> bit_index) & 1 == 0 {
            1.0_f32
        } else {
            -1.0_f32
        };
        let imaginary_bit = (bit_index + 1) % 5;
        let imaginary_sign = if ((slot + 1) >> imaginary_bit) & 1 == 0 {
            1.0_f32
        } else {
            -1.0_f32
        };
        [
            (amplitude * factor) * real_sign,
            (amplitude / 2.0_f32) * imaginary_sign,
        ]
    }

    fn concatenate_object_windows(previous: &[u32], current: &[u32]) -> Vec<u32> {
        let words_per_half = CURRENT_SLOTS * COMPLEX_BINS * 2;
        let mut output = Vec::with_capacity(OBJECT_COUNT * words_per_half * 2);
        for object in 0..OBJECT_COUNT {
            let start = object * words_per_half;
            let end = start + words_per_half;
            output.extend_from_slice(&previous[start..end]);
            output.extend_from_slice(&current[start..end]);
        }
        output
    }

    fn f64_wrapper_to_f32_word(value: &Value) -> u32 {
        let bytes = decode_hex(value["f64le"].as_str().unwrap());
        (f64::from_le_bytes(bytes.try_into().unwrap()) as f32).to_bits()
    }

    fn decode_u32_words(hex: &str) -> Vec<u32> {
        decode_hex(hex)
            .chunks_exact(4)
            .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
            .collect()
    }

    fn decode_hex(hex: &str) -> Vec<u8> {
        hex.as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                let text = std::str::from_utf8(pair).unwrap();
                u8::from_str_radix(text, 16).unwrap()
            })
            .collect()
    }
}
