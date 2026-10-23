// pattern: Functional Core
//! Stateless F03C selector with explicit F32 feature, position and response inputs.

use crate::numeric::{FiniteF32, r32_add, r32_mul, r32_sub};

const FEATURE_BANDS: usize = 24;
const RESPONSE_SLOTS: usize = 15;
const SELECTOR_CAPACITY: usize = 15;
const MAX_CANDIDATES: usize = 48;
const ONE_WORD: u32 = 0x3f80_0000;
const TWO_WORD: u32 = 0x4000_0000;
const HALF_WORD: u32 = 0x3f00_0000;
const SIX_WORD: u32 = 0x40c0_0000;
const COMPLEMENT_SCALE_WORD: u32 = 0x3f73_3333;
const MIN_AFFINITY_WORD: u32 = 0x3a83_126f;
const POWER_EXPONENT_WORD: u32 = 0x3e4c_cccd;
const TIE_WINDOW_WORD: u32 = 0x3c23_d70a;
const COVERAGE_EPSILON_WORD: u32 = 0x3400_0000;
const COVERAGE_LIMIT_WORD: u32 = 0x3f19_999a;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateSelectorInput {
    pub candidate_ids: Vec<u32>,
    /// Feature values in band-major `[24][candidate_count]` order.
    pub feature_u32: Vec<Vec<u32>>,
    pub positions_u32: Vec<[u32; 3]>,
    pub width_u32: Vec<u32>,
    pub attributes_u32: Vec<u32>,
    /// Basis responses in slot-major `[15][candidate_count]` order.
    pub response_u32: Vec<Vec<u32>>,
    pub previous_ids: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectorError {
    InvalidInput,
    NumericFailure,
    ChoiceUndefined,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectorStatus {
    CandidatesExhausted,
    CapacityReached,
}

impl SelectorStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CandidatesExhausted => "CANDIDATES_EXHAUSTED",
            Self::CapacityReached => "CAPACITY_REACHED",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectorScoreChoice {
    chosen: u32,
    ties: Vec<u32>,
    primary_max_u32: u32,
    secondary_max_u32: u32,
}

impl SelectorScoreChoice {
    pub const fn chosen(&self) -> u32 {
        self.chosen
    }

    pub fn ties(&self) -> &[u32] {
        &self.ties
    }

    pub const fn primary_max_u32(&self) -> u32 {
        self.primary_max_u32
    }

    pub const fn secondary_max_u32(&self) -> u32 {
        self.secondary_max_u32
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectorIteration {
    candidate_ids_before: Vec<u32>,
    gate_slot: Option<u8>,
    chosen_current_ordinal: u8,
    ties: Vec<u32>,
    primary_max_u32: u32,
    secondary_max_u32: u32,
    mask_after: Vec<u32>,
    coverage_u32: Vec<u32>,
    support_u32: Vec<u32>,
    surviving_ids: Vec<u32>,
}

impl SelectorIteration {
    pub fn candidate_ids_before(&self) -> &[u32] {
        &self.candidate_ids_before
    }

    pub const fn gate_slot(&self) -> Option<u8> {
        self.gate_slot
    }

    pub const fn chosen_current_ordinal(&self) -> u8 {
        self.chosen_current_ordinal
    }

    pub fn ties(&self) -> &[u32] {
        &self.ties
    }

    pub const fn primary_max_u32(&self) -> u32 {
        self.primary_max_u32
    }

    pub const fn secondary_max_u32(&self) -> u32 {
        self.secondary_max_u32
    }

    pub fn mask_after(&self) -> &[u32] {
        &self.mask_after
    }

    pub fn coverage_u32(&self) -> &[u32] {
        &self.coverage_u32
    }

    pub fn support_u32(&self) -> &[u32] {
        &self.support_u32
    }

    pub fn surviving_ids(&self) -> &[u32] {
        &self.surviving_ids
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateSelectorResult {
    status: SelectorStatus,
    selected_count: u8,
    selected_ids: Vec<u32>,
    selected_initial_ordinals: Vec<u32>,
    selected_attributes: Vec<u32>,
    initial_affinity_u32: Vec<Vec<u32>>,
    initial_complement_u32: Vec<Vec<u32>>,
    initial_support_u32: Vec<u32>,
    iterations: Vec<SelectorIteration>,
}

struct WorkingCandidates {
    ids: Vec<u32>,
    ordinals: Vec<u32>,
    positions: Vec<[u32; 3]>,
    widths: Vec<u32>,
    attributes: Vec<u32>,
    features: Vec<Vec<u32>>,
    responses: Vec<Vec<u32>>,
    affinity: Vec<Vec<u32>>,
    complement: Vec<Vec<u32>>,
    support: Vec<u32>,
}

impl WorkingCandidates {
    fn compress(&mut self, survivors: &[usize]) {
        self.ids = select_values(&self.ids, survivors);
        self.ordinals = select_values(&self.ordinals, survivors);
        self.positions = select_values(&self.positions, survivors);
        self.widths = select_values(&self.widths, survivors);
        self.attributes = select_values(&self.attributes, survivors);
        self.support = select_values(&self.support, survivors);
        self.features = self
            .features
            .iter()
            .map(|row| select_values(row, survivors))
            .collect();
        self.responses = self
            .responses
            .iter()
            .map(|row| select_values(row, survivors))
            .collect();
        self.affinity = survivors
            .iter()
            .map(|row| select_values(&self.affinity[*row], survivors))
            .collect();
        self.complement = survivors
            .iter()
            .map(|row| select_values(&self.complement[*row], survivors))
            .collect();
    }
}

impl CandidateSelectorResult {
    pub const fn status(&self) -> SelectorStatus {
        self.status
    }

    pub fn selected_ids(&self) -> &[u32] {
        &self.selected_ids
    }

    pub fn selected_initial_ordinals(&self) -> &[u32] {
        &self.selected_initial_ordinals
    }

    pub fn selected_attributes(&self) -> &[u32] {
        &self.selected_attributes
    }

    pub fn selected_count(&self) -> u8 {
        self.selected_count
    }

    pub fn initial_affinity_u32(&self) -> &[Vec<u32>] {
        &self.initial_affinity_u32
    }

    pub fn initial_complement_u32(&self) -> &[Vec<u32>] {
        &self.initial_complement_u32
    }

    pub fn initial_support_u32(&self) -> &[u32] {
        &self.initial_support_u32
    }

    pub fn iterations(&self) -> &[SelectorIteration] {
        &self.iterations
    }
}

/// Chooses the first primary score within 0.01 of the maximum after secondary tie resolution.
pub fn selector_score_choice(
    primary_u32: &[u32],
    secondary_u32: &[u32],
) -> Result<SelectorScoreChoice, SelectorError> {
    if primary_u32.is_empty()
        || primary_u32.len() != secondary_u32.len()
        || primary_u32
            .iter()
            .chain(secondary_u32)
            .any(|word| !f32::from_bits(*word).is_finite())
    {
        return Err(SelectorError::InvalidInput);
    }

    let first_primary = *primary_u32.first().ok_or(SelectorError::InvalidInput)?;
    let mut primary_max = f32::from_bits(first_primary);
    for word in primary_u32.iter().copied().skip(1) {
        let value = f32::from_bits(word);
        if value > primary_max {
            primary_max = value;
        }
    }
    let primary_max_u32 = primary_max.to_bits();
    let tie_threshold = subtract(primary_max_u32, TIE_WINDOW_WORD)?;
    let mut ties = Vec::new();
    for (index, word) in primary_u32.iter().enumerate() {
        if f32::from_bits(*word) >= f32::from_bits(tie_threshold) {
            ties.push(u32::try_from(index).map_err(|_| SelectorError::InvalidInput)?);
        }
    }
    if ties.is_empty() {
        return Err(SelectorError::ChoiceUndefined);
    }

    let first_tie = usize::try_from(*ties.first().ok_or(SelectorError::ChoiceUndefined)?)
        .map_err(|_| SelectorError::ChoiceUndefined)?;
    let mut secondary_max = f32::from_bits(secondary_u32[first_tie]);
    for ordinal in ties.iter().copied().skip(1) {
        let index = usize::try_from(ordinal).map_err(|_| SelectorError::ChoiceUndefined)?;
        let value = f32::from_bits(secondary_u32[index]);
        if value > secondary_max {
            secondary_max = value;
        }
    }
    let mut chosen = None;
    for ordinal in &ties {
        let index = usize::try_from(*ordinal).map_err(|_| SelectorError::ChoiceUndefined)?;
        if f32::from_bits(secondary_u32[index]) >= secondary_max {
            chosen = Some(*ordinal);
            break;
        }
    }
    let chosen = chosen.ok_or(SelectorError::ChoiceUndefined)?;
    Ok(SelectorScoreChoice {
        chosen,
        ties,
        primary_max_u32,
        secondary_max_u32: secondary_max.to_bits(),
    })
}

/// Computes the ordered, potentially asymmetric F03C pairwise affinity matrix.
pub fn pairwise_affinity(
    positions_u32: &[[u32; 3]],
    attributes_u32: &[u32],
    width_u32: &[u32],
) -> Result<Vec<Vec<u32>>, SelectorError> {
    let count = positions_u32.len();
    if !(1..=MAX_CANDIDATES).contains(&count)
        || attributes_u32.len() != count
        || width_u32.len() != count
        || positions_u32
            .iter()
            .flatten()
            .chain(width_u32)
            .any(|word| !f32::from_bits(*word).is_finite())
        || width_u32.iter().any(|word| f32::from_bits(*word) < 0.0)
    {
        return Err(SelectorError::InvalidInput);
    }

    let eleven = 11.0_f32.to_bits();
    let capacity = 15.0_f32.to_bits();
    let scale_ratio = divide(eleven, capacity)?;
    let scale_root = sqrt32(scale_ratio)?;
    let scale = multiply(scale_root, 0x3e4c_cccd)?;
    let mut affinity = vec![vec![0; count]; count];
    for i in 0..count {
        for j in 0..count {
            affinity[i][j] = if i == j {
                ONE_WORD
            } else if attributes_u32[i] != attributes_u32[j] {
                MIN_AFFINITY_WORD
            } else {
                let distance = squared_distance(positions_u32[i], positions_u32[j])?;
                let denominator = add(scale, width_u32[j])?;
                let denominator_squared = multiply(denominator, denominator)?;
                let ratio = divide(distance, denominator_squared)?;
                let clipped = if f32::from_bits(ratio) > 1.0 {
                    ONE_WORD
                } else {
                    ratio
                };
                let half_angle = multiply(clipped, HALF_WORD)?;
                let angle = multiply(half_angle, 0x4049_0fdb)?;
                let cosine = f32::from_bits(angle).cos();
                finite_word(if cosine < f32::from_bits(MIN_AFFINITY_WORD) {
                    f32::from_bits(MIN_AFFINITY_WORD)
                } else {
                    cosine
                })?
            };
        }
    }
    Ok(affinity)
}

/// Executes one complete cold F03C selector call from explicit same-callback inputs.
pub fn candidate_selector(
    input: CandidateSelectorInput,
) -> Result<CandidateSelectorResult, SelectorError> {
    validate_input(&input)?;
    let initial_affinity = pairwise_affinity(
        &input.positions_u32,
        &input.attributes_u32,
        &input.width_u32,
    )?;
    let mut affinity = initial_affinity.clone();
    let mut complement = vec![vec![0; input.candidate_ids.len()]; input.candidate_ids.len()];
    build_complement(&input.response_u32, &mut affinity, &mut complement)?;
    let support = initial_support(&input.feature_u32)?;
    let initial_support = support.clone();
    let slot_normalizers = initial_slot_normalizers(&input.response_u32, &support)?;
    let total_by_band = band_totals(&input.feature_u32)?;
    let normalizer_by_band = band_normalizers(&total_by_band)?;

    let initial_ordinals = (0..input.candidate_ids.len())
        .map(|ordinal| u32::try_from(ordinal).map_err(|_| SelectorError::InvalidInput))
        .collect::<Result<Vec<_>, _>>()?;
    let initial_complement = complement.clone();
    let previous_ids = input.previous_ids;
    let mut working = WorkingCandidates {
        ids: input.candidate_ids,
        ordinals: initial_ordinals,
        positions: input.positions_u32,
        widths: input.width_u32,
        attributes: input.attributes_u32,
        features: input.feature_u32,
        responses: input.response_u32,
        affinity,
        complement,
        support,
    };
    let mut mask = vec![0_u32; RESPONSE_SLOTS];
    let mut coverage = vec![0_u32; RESPONSE_SLOTS];
    let mut selected_ids = Vec::new();
    let mut selected_ordinals = Vec::new();
    let mut selected_attributes = Vec::new();
    let mut iterations = Vec::new();
    let status;

    loop {
        if working.ids.is_empty() {
            status = SelectorStatus::CandidatesExhausted;
            break;
        }
        let mut scores =
            score_candidates(&working, &total_by_band, &normalizer_by_band, &previous_ids)?;
        let gate_slot = first_eligible_slot(&mask, &coverage, &working.responses, &scores)?;
        if let Some(slot) = gate_slot {
            for (candidate, score) in scores.iter_mut().enumerate() {
                let response_value = working.responses[usize::from(slot)][candidate];
                let squared = multiply(response_value, response_value)?;
                *score = multiply(squared, *score)?;
            }
            mask[usize::from(slot)] = 1;
        }

        let choice = selector_score_choice(&scores, &working.support)?;
        let chosen = usize::try_from(choice.chosen).map_err(|_| SelectorError::InvalidInput)?;
        if chosen >= working.ids.len() {
            return Err(SelectorError::ChoiceUndefined);
        }
        let chosen_id = working.ids[chosen];
        let chosen_ordinal = working.ordinals[chosen];
        let chosen_attribute = working.attributes[chosen];
        selected_ids.push(chosen_id);
        selected_ordinals.push(chosen_ordinal);
        selected_attributes.push(chosen_attribute);

        for candidate in 0..working.ids.len() {
            let remaining = subtract(ONE_WORD, working.affinity[candidate][chosen])?;
            for feature_band in &mut working.features {
                feature_band[candidate] = multiply(remaining, feature_band[candidate])?;
            }
        }
        update_coverage_and_support(
            chosen,
            &working.responses,
            &working.complement,
            &working.affinity,
            &slot_normalizers,
            &mut coverage,
            &mut working.support,
        )?;

        let survivor_indices = find_survivors(
            chosen,
            &working.features,
            &working.positions,
            &working.attributes,
        )?;
        let surviving_ids = survivor_indices
            .iter()
            .map(|index| working.ids[*index])
            .collect();
        iterations.push(SelectorIteration {
            candidate_ids_before: working.ids.clone(),
            gate_slot,
            chosen_current_ordinal: u8::try_from(chosen)
                .map_err(|_| SelectorError::InvalidInput)?,
            ties: choice.ties,
            primary_max_u32: choice.primary_max_u32,
            secondary_max_u32: choice.secondary_max_u32,
            mask_after: mask.clone(),
            coverage_u32: coverage.clone(),
            support_u32: working.support.clone(),
            surviving_ids,
        });

        if selected_ids.len() == SELECTOR_CAPACITY {
            status = SelectorStatus::CapacityReached;
            break;
        }
        if survivor_indices.is_empty() {
            status = SelectorStatus::CandidatesExhausted;
            break;
        }
        working.compress(&survivor_indices);
    }

    Ok(CandidateSelectorResult {
        status,
        selected_count: u8::try_from(selected_ids.len())
            .map_err(|_| SelectorError::InvalidInput)?,
        selected_ids,
        selected_initial_ordinals: selected_ordinals,
        selected_attributes,
        initial_affinity_u32: initial_affinity,
        initial_complement_u32: initial_complement,
        initial_support_u32: initial_support,
        iterations,
    })
}

fn validate_input(input: &CandidateSelectorInput) -> Result<(), SelectorError> {
    let count = input.candidate_ids.len();
    if !(1..=MAX_CANDIDATES).contains(&count)
        || input.feature_u32.len() != FEATURE_BANDS
        || input.positions_u32.len() != count
        || input.width_u32.len() != count
        || input.attributes_u32.len() != count
        || input.response_u32.len() != RESPONSE_SLOTS
        || input.previous_ids.len() != RESPONSE_SLOTS
    {
        return Err(SelectorError::InvalidInput);
    }
    let mut previous_id = None;
    for candidate_id in &input.candidate_ids {
        if *candidate_id > 47 || previous_id.is_some_and(|previous| *candidate_id <= previous) {
            return Err(SelectorError::InvalidInput);
        }
        previous_id = Some(*candidate_id);
    }
    if input.feature_u32.iter().any(|row| {
        row.len() != count
            || row
                .iter()
                .any(|word| !f32::from_bits(*word).is_finite() || f32::from_bits(*word) < 0.0)
    }) || input.response_u32.iter().any(|row| {
        row.len() != count
            || row
                .iter()
                .any(|word| !f32::from_bits(*word).is_finite() || f32::from_bits(*word) < 0.0)
    }) || input
        .positions_u32
        .iter()
        .flatten()
        .any(|word| !is_finite(*word))
        || input
            .width_u32
            .iter()
            .any(|word| !is_finite(*word) || f32::from_bits(*word) < 0.0)
        || input
            .previous_ids
            .iter()
            .any(|id| *id != u32::MAX && *id > 47)
    {
        return Err(SelectorError::InvalidInput);
    }
    Ok(())
}

fn build_complement(
    response: &[Vec<u32>],
    affinity: &mut [Vec<u32>],
    complement: &mut [Vec<u32>],
) -> Result<(), SelectorError> {
    for first in 0..affinity.len() {
        for second in first + 1..affinity.len() {
            let mut first_only = false;
            let mut second_only = false;
            for slot_response in response {
                let left = f32::from_bits(slot_response[first]);
                let right = f32::from_bits(slot_response[second]);
                first_only |= left > 0.0 && right <= 0.0;
                second_only |= left <= 0.0 && right > 0.0;
            }
            if first_only && second_only {
                let reduced = multiply(affinity[first][second], COMPLEMENT_SCALE_WORD)?;
                let adjusted = if f32::from_bits(reduced) < f32::from_bits(MIN_AFFINITY_WORD) {
                    MIN_AFFINITY_WORD
                } else {
                    reduced
                };
                affinity[first][second] = adjusted;
                affinity[second][first] = adjusted;
                complement[first][second] = ONE_WORD;
                complement[second][first] = ONE_WORD;
            }
        }
    }
    Ok(())
}

fn initial_support(feature: &[Vec<u32>]) -> Result<Vec<u32>, SelectorError> {
    let count = feature[0].len();
    let mut support = vec![0; count];
    for candidate in 0..count {
        let mut total = 0;
        for band_features in feature.iter().take(FEATURE_BANDS) {
            let powered = power(band_features[candidate], POWER_EXPONENT_WORD)?;
            total = add(total, powered)?;
        }
        support[candidate] = total;
    }
    Ok(support)
}

fn initial_slot_normalizers(
    response: &[Vec<u32>],
    support: &[u32],
) -> Result<Vec<u32>, SelectorError> {
    let mut normalizers = vec![0; RESPONSE_SLOTS];
    for slot in 0..RESPONSE_SLOTS {
        let mut total = 0;
        for (candidate, support) in support.iter().copied().enumerate() {
            let squared = multiply(response[slot][candidate], response[slot][candidate])?;
            total = add(total, multiply(squared, support)?)?;
        }
        normalizers[slot] = total;
    }
    Ok(normalizers)
}

fn band_totals(feature: &[Vec<u32>]) -> Result<Vec<u32>, SelectorError> {
    let mut totals = vec![0; FEATURE_BANDS];
    for band in 0..FEATURE_BANDS {
        let mut total = 0;
        for value in &feature[band] {
            total = add(total, *value)?;
        }
        totals[band] = total;
    }
    Ok(totals)
}

fn band_normalizers(totals: &[u32]) -> Result<Vec<u32>, SelectorError> {
    totals
        .iter()
        .map(|total| {
            let shifted = add(*total, ONE_WORD)?;
            multiply(power(shifted, POWER_EXPONENT_WORD)?, TWO_WORD)
        })
        .collect()
}

fn score_candidates(
    working: &WorkingCandidates,
    totals: &[u32],
    normalizers: &[u32],
    previous_ids: &[u32],
) -> Result<Vec<u32>, SelectorError> {
    let mut scores = vec![0; working.ids.len()];
    for (candidate, score_slot) in scores.iter_mut().enumerate() {
        let previous = previous_ids.contains(&working.ids[candidate]);
        let mut score = 0;
        for (band, feature_band) in working.features.iter().enumerate() {
            let mut affinity_sum = 0;
            for (other, feature_value) in feature_band.iter().enumerate() {
                affinity_sum = add(
                    affinity_sum,
                    multiply(*feature_value, working.affinity[candidate][other])?,
                )?;
            }
            if previous {
                affinity_sum = multiply(affinity_sum, SIX_WORD)?;
            }
            let mut remainder = subtract(totals[band], affinity_sum)?;
            if f32::from_bits(remainder) < 0.0 {
                remainder = 0;
            }
            let powered = power(add(remainder, ONE_WORD)?, POWER_EXPONENT_WORD)?;
            let remaining_normalizer = multiply(powered, TWO_WORD)?;
            let band_score = subtract(normalizers[band], remaining_normalizer)?;
            score = add(score, band_score)?;
        }
        *score_slot = score;
    }
    Ok(scores)
}

fn first_eligible_slot(
    mask: &[u32],
    coverage: &[u32],
    response: &[Vec<u32>],
    scores: &[u32],
) -> Result<Option<u8>, SelectorError> {
    for slot in 0..RESPONSE_SLOTS {
        if mask[slot] != 0 {
            continue;
        }
        let low_coverage = add(
            multiply(coverage[slot], COVERAGE_EPSILON_WORD)?,
            coverage[slot],
        )?;
        if f32::from_bits(low_coverage) >= f32::from_bits(COVERAGE_LIMIT_WORD) {
            continue;
        }
        let has_positive = (0..scores.len()).any(|candidate| {
            f32::from_bits(response[slot][candidate]) > 0.0
                && f32::from_bits(scores[candidate]) > 0.0
        });
        if has_positive {
            return u8::try_from(slot)
                .map(Some)
                .map_err(|_| SelectorError::InvalidInput);
        }
    }
    Ok(None)
}

fn update_coverage_and_support(
    chosen: usize,
    response: &[Vec<u32>],
    complement: &[Vec<u32>],
    affinity: &[Vec<u32>],
    slot_normalizers: &[u32],
    coverage: &mut [u32],
    support: &mut [u32],
) -> Result<(), SelectorError> {
    for candidate in 0..support.len() {
        if complement[candidate][chosen] != 0 {
            continue;
        }
        let delta = power(
            subtract(ONE_WORD, affinity[candidate][chosen])?,
            POWER_EXPONENT_WORD,
        )?;
        let remain = subtract(ONE_WORD, delta)?;
        for slot in 0..RESPONSE_SLOTS {
            if f32::from_bits(response[slot][chosen]) <= 0.0 {
                continue;
            }
            let squared = multiply(response[slot][candidate], response[slot][candidate])?;
            let with_support = multiply(squared, support[candidate])?;
            let numerator = multiply(with_support, remain)?;
            let denominator = add(slot_normalizers[slot], COVERAGE_EPSILON_WORD)?;
            let increment = divide(numerator, denominator)?;
            coverage[slot] = add(coverage[slot], increment)?;
        }
        support[candidate] = multiply(delta, support[candidate])?;
    }
    Ok(())
}

fn find_survivors(
    chosen: usize,
    feature: &[Vec<u32>],
    positions: &[[u32; 3]],
    attributes: &[u32],
) -> Result<Vec<usize>, SelectorError> {
    let one_over_62 = divide(ONE_WORD, 62.0_f32.to_bits())?;
    let distance_threshold = multiply(one_over_62, one_over_62)?;
    let mut survivors = Vec::new();
    for candidate in 0..positions.len() {
        let mut total = 0;
        for band_features in feature.iter().take(FEATURE_BANDS) {
            total = add(total, band_features[candidate])?;
        }
        let distance = squared_distance(positions[candidate], positions[chosen])?;
        if f32::from_bits(total) > 0.5
            && (f32::from_bits(distance) > f32::from_bits(distance_threshold)
                || attributes[candidate] != attributes[chosen])
        {
            survivors.push(candidate);
        }
    }
    Ok(survivors)
}

fn squared_distance(left: [u32; 3], right: [u32; 3]) -> Result<u32, SelectorError> {
    let mut total = 0;
    for axis in 0..3 {
        let difference = subtract(left[axis], right[axis])?;
        total = add(total, multiply(difference, difference)?)?;
    }
    Ok(total)
}

fn select_values<T: Copy>(values: &[T], indices: &[usize]) -> Vec<T> {
    indices.iter().map(|index| values[*index]).collect()
}

fn is_finite(word: u32) -> bool {
    f32::from_bits(word).is_finite()
}

fn finite_word(value: f32) -> Result<u32, SelectorError> {
    if value.is_finite() {
        Ok(value.to_bits())
    } else {
        Err(SelectorError::NumericFailure)
    }
}

fn add(left: u32, right: u32) -> Result<u32, SelectorError> {
    let left = FiniteF32::new(f32::from_bits(left)).map_err(|_| SelectorError::NumericFailure)?;
    let right = FiniteF32::new(f32::from_bits(right)).map_err(|_| SelectorError::NumericFailure)?;
    r32_add(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| SelectorError::NumericFailure)
}

fn subtract(left: u32, right: u32) -> Result<u32, SelectorError> {
    let left = FiniteF32::new(f32::from_bits(left)).map_err(|_| SelectorError::NumericFailure)?;
    let right = FiniteF32::new(f32::from_bits(right)).map_err(|_| SelectorError::NumericFailure)?;
    r32_sub(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| SelectorError::NumericFailure)
}

fn multiply(left: u32, right: u32) -> Result<u32, SelectorError> {
    let left = FiniteF32::new(f32::from_bits(left)).map_err(|_| SelectorError::NumericFailure)?;
    let right = FiniteF32::new(f32::from_bits(right)).map_err(|_| SelectorError::NumericFailure)?;
    r32_mul(left, right)
        .map(FiniteF32::to_bits)
        .map_err(|_| SelectorError::NumericFailure)
}

fn divide(numerator: u32, denominator: u32) -> Result<u32, SelectorError> {
    let numerator = f32::from_bits(numerator);
    let denominator = f32::from_bits(denominator);
    if !numerator.is_finite() || !denominator.is_finite() || denominator == 0.0 {
        return Err(SelectorError::NumericFailure);
    }
    finite_word(numerator / denominator)
}

fn sqrt32(value: u32) -> Result<u32, SelectorError> {
    let value = f32::from_bits(value);
    if !value.is_finite() || value < 0.0 {
        return Err(SelectorError::NumericFailure);
    }
    finite_word(value.sqrt())
}

fn power(base: u32, exponent: u32) -> Result<u32, SelectorError> {
    let base = f32::from_bits(base);
    let exponent = f32::from_bits(exponent);
    if !base.is_finite() || !exponent.is_finite() || base < 0.0 {
        return Err(SelectorError::NumericFailure);
    }
    finite_word(base.powf(exponent))
}
