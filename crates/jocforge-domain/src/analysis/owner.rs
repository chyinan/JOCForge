// pattern: Functional Core
//! Selected F07-F10 analysis-owner state and noncommitting source stages.

use super::exponent_plan::{
    AnalysisExponentPlan, AnalysisExponentPlanError, stage_f09_f10_source_plan,
};
use super::gain_history::{
    FULLBAND_GAIN_CHANNELS, GAIN_BLOCK_SAMPLES, GAIN_BLOCKS_PER_FRAME, GAIN_CHANNELS,
    GainHistoryState, PeakReductionProfile, process_gain_block,
};
use super::input_history::InputHistory;
use super::mean_square::{MeanSquareError, checked_mean_square_256};
use super::source_frame::{
    F07SourceFrameBeforeState, F07SourceFrameError, FBW_EXTERNAL_ROWS, InstalledAudioSourceFrame,
    LFE_EXTERNAL_ROW, SharedAnalysisReceipt, stage_f07a_source_frame,
};
use super::structure_refresh::{F07_BLOCKS_PER_FRAME, F07_FULLBAND_LANES};
use super::transient_channel::TransientChannelState;
use super::transient_parameters::TransientParameters;
use crate::codec::db_selector::{DbSelectorError, db_selector_from_means};
use crate::codec::policy_parameters::{PolicyParameters, PolicyProfile};
use crate::meter::prepared_job_identity::PreparedJobInputIdentity;
use crate::timeline::{CodecSourceIndex, FrontendCallbackIndex, source_for_callback};
use crate::transaction::{
    AttemptId, CallbackTransactionId, GtcOwnerId, GtcOwnerReceipt, GtcOwnerReceiptError,
    GtcOwnerStateSnapshot, SourceTransaction, SourceTransactionStageError,
};
use crate::transform::exponent_inputs::{
    FullBandTransformExponentInputs, LfeTransformExponentInputs, TransformExponentInputError,
    f08_full_band_exponent_inputs, f08_lfe_exponent_inputs,
};
use crate::transform::fbw_transform::{
    F08FbwTransformError, F08FbwTransformOutput, f08_fbw_transform_block,
};
use crate::transform::fullband_exponent_pipeline::NumericPlanReceipt;
use crate::transform::lfe_transform::{
    F08LfeTransformError, F08LfeTransformOutput, f08_lfe_transform_block,
};
use crate::transform::math_generated_parameters::MathGeneratedTransformParameters;
use crate::transform::window_fold::{F08WindowFoldError, f08_window_fold_block};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::Arc;

const EXTERNAL_AUDIO_LANES: usize = 6;
const TRANSFORM_HISTORY_WORDS_PER_LANE: usize = 128;
const GAIN_RANGE_CHANNELS: usize = 6;
const ANALYSIS_OWNER_ID: &str = "analysis";
const ANALYSIS_OWNER_STATE_REF: &str = "owners.analysis";

fn json_integer_array<T, I>(values: I) -> Value
where
    I: IntoIterator<Item = T>,
    Value: From<T>,
{
    Value::Array(values.into_iter().map(Value::from).collect())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisOwnerStateReferenceError {
    IncompleteRoot,
    InvalidTransientI1,
    CanonicalSerialization,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisOwnerGtcReceiptError {
    StateReference(AnalysisOwnerStateReferenceError),
    PreparedInputIdentityMissing,
    PreparedInputIdentityMismatch,
    SourceStageRequired,
    Gtc(GtcOwnerReceiptError),
}

/// Read-only canonical state view for one commit-eligible Analysis owner root.
///
/// The snapshot borrows the complete typed root and carries its canonical V6 owner
/// JSON plus the current and cold SHA-256 values. It does not stage or commit state.
#[derive(Debug, Eq, PartialEq)]
pub struct AnalysisOwnerStateReference<'a> {
    root: &'a AnalysisOwnerRoot,
    generation: u64,
    canonical_bytes: Vec<u8>,
    state_sha256: [u8; 32],
    cold_sha256: [u8; 32],
}

impl AnalysisOwnerStateReference<'_> {
    pub const fn owner_id(&self) -> &'static str {
        ANALYSIS_OWNER_ID
    }

    pub const fn state_ref(&self) -> &'static str {
        ANALYSIS_OWNER_STATE_REF
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn state(&self) -> &AnalysisOwnerRoot {
        self.root
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub const fn state_sha256(&self) -> &[u8; 32] {
        &self.state_sha256
    }

    pub const fn cold_sha256(&self) -> &[u8; 32] {
        &self.cold_sha256
    }

    /// Projects the complete canonical Analysis state into the registered GTC owner slot.
    ///
    /// The returned snapshot borrows these canonical bytes; it is not a parent commit token.
    pub fn gtc_snapshot(&self) -> Result<GtcOwnerStateSnapshot<'_>, GtcOwnerReceiptError> {
        GtcOwnerStateSnapshot::from_canonical_bytes(
            GtcOwnerId::Analysis,
            self.generation,
            self.cold_sha256,
            &self.canonical_bytes,
        )
    }
}

/// The four persistent F07-F10 analysis families.
///
/// This root has no source lifecycle, commit, failure, EOF, previous-short, metadata,
/// codec, DRC, payload, or publication state. The GTC parent supplies source identities
/// and remains the sole owner of commit/discard authority.
#[derive(Debug, Eq, PartialEq)]
pub struct AnalysisOwnerRoot {
    prepared_job_input_identity: Option<Arc<PreparedJobInputIdentity>>,
    input_history_generation: u64,
    input_histories: [InputHistory; EXTERNAL_AUDIO_LANES],
    transient_generation: u64,
    transient_states: [TransientChannelState; F07_FULLBAND_LANES],
    transform_generation: u64,
    primary_fbw_history_u32_words: [u32; 640],
    companion_fbw_history_u32_words: [u32; 640],
    primary_lfe_history_u32_words: [u32; TRANSFORM_HISTORY_WORDS_PER_LANE],
    companion_lfe_history_u32_words: [u32; TRANSFORM_HISTORY_WORDS_PER_LANE],
    policy_generation: u64,
    gain_range_previous_peak_f32_bits_u32: [u32; GAIN_RANGE_CHANNELS],
    gain_range_completed_blocks_u64: u64,
    db_per_bit_selector_u16: u16,
    db_per_bit_completed_blocks_u64: u64,
}

impl AnalysisOwnerRoot {
    /// Creates the exact cold analysis root: all generations and state words are zero.
    pub fn cold() -> Self {
        Self::cold_with_prepared_identity(None)
    }

    /// Creates a cold MAIN analysis root carrying the exact identity capability admitted by F01E.
    pub fn cold_for_prepared_input(
        prepared_job_input_identity: Arc<PreparedJobInputIdentity>,
    ) -> Self {
        Self::cold_with_prepared_identity(Some(prepared_job_input_identity))
    }

    fn cold_with_prepared_identity(
        prepared_job_input_identity: Option<Arc<PreparedJobInputIdentity>>,
    ) -> Self {
        Self {
            prepared_job_input_identity,
            input_history_generation: 0,
            input_histories: std::array::from_fn(|_| InputHistory::cold()),
            transient_generation: 0,
            transient_states: std::array::from_fn(|_| TransientChannelState::cold()),
            transform_generation: 0,
            primary_fbw_history_u32_words: [0; 640],
            companion_fbw_history_u32_words: [0; 640],
            primary_lfe_history_u32_words: [0; TRANSFORM_HISTORY_WORDS_PER_LANE],
            companion_lfe_history_u32_words: [0; TRANSFORM_HISTORY_WORDS_PER_LANE],
            policy_generation: 0,
            gain_range_previous_peak_f32_bits_u32: [0; GAIN_RANGE_CHANNELS],
            gain_range_completed_blocks_u64: 0,
            db_per_bit_selector_u16: 0,
            db_per_bit_completed_blocks_u64: 0,
        }
    }

    /// Replaces this analysis root with the canonical cold value.
    ///
    /// The parent owns attempt advancement and retires the prior attempt before calling reset.
    pub fn reset(&mut self) {
        *self = Self::cold_with_prepared_identity(self.prepared_job_input_identity.clone());
    }

    /// Returns the F01E input identity capability for a bound MAIN root.
    pub fn prepared_job_input_identity(&self) -> Option<&Arc<PreparedJobInputIdentity>> {
        self.prepared_job_input_identity.as_ref()
    }

    pub const fn input_history_generation(&self) -> u64 {
        self.input_history_generation
    }

    pub const fn input_histories(&self) -> &[InputHistory; EXTERNAL_AUDIO_LANES] {
        &self.input_histories
    }

    pub const fn transient_generation(&self) -> u64 {
        self.transient_generation
    }

    pub const fn transient_states(&self) -> &[TransientChannelState; F07_FULLBAND_LANES] {
        &self.transient_states
    }

    pub const fn transform_generation(&self) -> u64 {
        self.transform_generation
    }

    pub const fn primary_fbw_history_u32_words(&self) -> &[u32; 640] {
        &self.primary_fbw_history_u32_words
    }

    pub const fn companion_fbw_history_u32_words(&self) -> &[u32; 640] {
        &self.companion_fbw_history_u32_words
    }

    pub const fn primary_lfe_history_u32_words(&self) -> &[u32; TRANSFORM_HISTORY_WORDS_PER_LANE] {
        &self.primary_lfe_history_u32_words
    }

    pub const fn companion_lfe_history_u32_words(
        &self,
    ) -> &[u32; TRANSFORM_HISTORY_WORDS_PER_LANE] {
        &self.companion_lfe_history_u32_words
    }

    pub const fn policy_generation(&self) -> u64 {
        self.policy_generation
    }

    pub const fn gain_range_previous_peak_f32_bits_u32(&self) -> &[u32; GAIN_RANGE_CHANNELS] {
        &self.gain_range_previous_peak_f32_bits_u32
    }

    pub const fn gain_range_completed_blocks_u64(&self) -> u64 {
        self.gain_range_completed_blocks_u64
    }

    pub const fn db_per_bit_selector_u16(&self) -> u16 {
        self.db_per_bit_selector_u16
    }

    pub const fn db_per_bit_completed_blocks_u64(&self) -> u64 {
        self.db_per_bit_completed_blocks_u64
    }

    /// Returns the complete sorted-key JSON owner snapshot for a commit-eligible root.
    ///
    /// The V6 projection contains only the four persistent F07-F10 families. Candidate,
    /// F11/F12, publication, and other owner state are deliberately outside this value.
    pub fn state_reference(
        &self,
    ) -> Result<AnalysisOwnerStateReference<'_>, AnalysisOwnerStateReferenceError> {
        let generation = self
            .common_generation()
            .ok_or(AnalysisOwnerStateReferenceError::IncompleteRoot)?;
        let canonical_bytes = self.canonical_state_bytes()?;
        let state_sha256 = Sha256::digest(&canonical_bytes).into();
        let cold_bytes = Self::cold().canonical_state_bytes()?;
        let cold_sha256 = Sha256::digest(&cold_bytes).into();
        Ok(AnalysisOwnerStateReference {
            root: self,
            generation,
            canonical_bytes,
            state_sha256,
            cold_sha256,
        })
    }

    /// Creates the Analysis owner's GTC receipt for exact committed-before and staged-after
    /// roots. Both roots must carry the same admission-created resource capability; replacement
    /// authority remains with the global callback parent.
    pub fn gtc_owner_receipt_to(
        &self,
        after: &Self,
        transaction_id: CallbackTransactionId,
    ) -> Result<GtcOwnerReceipt, AnalysisOwnerGtcReceiptError> {
        let before_identity = self
            .prepared_job_input_identity()
            .ok_or(AnalysisOwnerGtcReceiptError::PreparedInputIdentityMissing)?;
        let after_identity = after
            .prepared_job_input_identity()
            .ok_or(AnalysisOwnerGtcReceiptError::PreparedInputIdentityMissing)?;
        if !Arc::ptr_eq(before_identity, after_identity) {
            return Err(AnalysisOwnerGtcReceiptError::PreparedInputIdentityMismatch);
        }

        let before_reference = self
            .state_reference()
            .map_err(AnalysisOwnerGtcReceiptError::StateReference)?;
        let after_reference = after
            .state_reference()
            .map_err(AnalysisOwnerGtcReceiptError::StateReference)?;
        let before_snapshot = before_reference
            .gtc_snapshot()
            .map_err(AnalysisOwnerGtcReceiptError::Gtc)?;
        let after_snapshot = after_reference
            .gtc_snapshot()
            .map_err(AnalysisOwnerGtcReceiptError::Gtc)?;
        GtcOwnerReceipt::from_transition(before_snapshot, after_snapshot, transaction_id)
            .map_err(AnalysisOwnerGtcReceiptError::Gtc)
    }

    /// Creates the unchanged Analysis owner receipt for a callback without a codec source stage.
    ///
    /// The selected clock has no source at callback indices zero and one. The committed root
    /// must still carry the admission-created input capability; any later callback must provide
    /// the exact staged Analysis after-root through `gtc_owner_receipt_to`.
    pub fn gtc_unchanged_callback_receipt(
        &self,
        callback_index: FrontendCallbackIndex,
        transaction_id: CallbackTransactionId,
    ) -> Result<GtcOwnerReceipt, AnalysisOwnerGtcReceiptError> {
        if source_for_callback(callback_index).is_some() {
            return Err(AnalysisOwnerGtcReceiptError::SourceStageRequired);
        }
        if self.prepared_job_input_identity().is_none() {
            return Err(AnalysisOwnerGtcReceiptError::PreparedInputIdentityMissing);
        }
        let reference = self
            .state_reference()
            .map_err(AnalysisOwnerGtcReceiptError::StateReference)?;
        let snapshot = reference
            .gtc_snapshot()
            .map_err(AnalysisOwnerGtcReceiptError::Gtc)?;
        GtcOwnerReceipt::from_transition(snapshot, snapshot, transaction_id)
            .map_err(AnalysisOwnerGtcReceiptError::Gtc)
    }

    fn canonical_state_bytes(&self) -> Result<Vec<u8>, AnalysisOwnerStateReferenceError> {
        if !self.is_commit_eligible() {
            return Err(AnalysisOwnerStateReferenceError::IncompleteRoot);
        }
        if self.transient_states.iter().any(|state| {
            state
                .short_queue_i1()
                .iter()
                .chain(state.other_queue_i1())
                .any(|value| *value > 1)
        }) {
            return Err(AnalysisOwnerStateReferenceError::InvalidTransientI1);
        }

        let mut fields = BTreeMap::<String, Value>::new();
        fields.insert(
            "input_history_generation".to_owned(),
            Value::from(self.input_history_generation),
        );
        fields.insert(
            "input_history_u32_words".to_owned(),
            json_integer_array(
                self.input_histories
                    .iter()
                    .flat_map(|history| history.words().iter().copied()),
            ),
        );
        fields.insert(
            "transient_generation".to_owned(),
            Value::from(self.transient_generation),
        );
        fields.insert(
            "transient_two_history_u32_words".to_owned(),
            json_integer_array(
                self.transient_states
                    .iter()
                    .flat_map(TransientChannelState::two_history_words),
            ),
        );
        fields.insert(
            "transient_three_history_u32_words".to_owned(),
            json_integer_array(
                self.transient_states
                    .iter()
                    .flat_map(TransientChannelState::three_history_words),
            ),
        );
        fields.insert(
            "transient_peak_statistics_u32_words".to_owned(),
            json_integer_array(
                self.transient_states
                    .iter()
                    .flat_map(|state| state.peak_statistics_words().iter().copied()),
            ),
        );
        fields.insert(
            "transient_previous_mean_u32_words".to_owned(),
            json_integer_array(
                self.transient_states
                    .iter()
                    .map(TransientChannelState::previous_mean_word),
            ),
        );
        fields.insert(
            "transient_short_queue_i1".to_owned(),
            json_integer_array(
                self.transient_states
                    .iter()
                    .flat_map(|state| state.short_queue_i1().iter().copied()),
            ),
        );
        fields.insert(
            "transient_other_queue_i1".to_owned(),
            json_integer_array(
                self.transient_states
                    .iter()
                    .flat_map(|state| state.other_queue_i1().iter().copied()),
            ),
        );
        fields.insert(
            "transform_generation".to_owned(),
            Value::from(self.transform_generation),
        );
        fields.insert(
            "primary_fbw_history_u32_words".to_owned(),
            json_integer_array(self.primary_fbw_history_u32_words),
        );
        fields.insert(
            "companion_fbw_history_u32_words".to_owned(),
            json_integer_array(self.companion_fbw_history_u32_words),
        );
        fields.insert(
            "primary_lfe_history_u32_words".to_owned(),
            json_integer_array(self.primary_lfe_history_u32_words),
        );
        fields.insert(
            "companion_lfe_history_u32_words".to_owned(),
            json_integer_array(self.companion_lfe_history_u32_words),
        );
        fields.insert(
            "policy_generation".to_owned(),
            Value::from(self.policy_generation),
        );
        let mut gain_range = serde_json::Map::new();
        gain_range.insert(
            "completed_blocks_u64".to_owned(),
            Value::from(self.gain_range_completed_blocks_u64),
        );
        gain_range.insert(
            "previous_peak_f32_bits_u32".to_owned(),
            json_integer_array(self.gain_range_previous_peak_f32_bits_u32),
        );
        fields.insert("gain_range".to_owned(), Value::Object(gain_range));

        let mut db_per_bit = serde_json::Map::new();
        db_per_bit.insert(
            "completed_blocks_u64".to_owned(),
            Value::from(self.db_per_bit_completed_blocks_u64),
        );
        db_per_bit.insert(
            "selector_u16".to_owned(),
            Value::from(self.db_per_bit_selector_u16),
        );
        fields.insert("db_per_bit".to_owned(), Value::Object(db_per_bit));

        serde_json::to_vec(&fields)
            .map_err(|_| AnalysisOwnerStateReferenceError::CanonicalSerialization)
    }

    /// Reports whether this root satisfies the four-family committed-generation invariant.
    pub const fn is_commit_eligible(&self) -> bool {
        self.input_history_generation == self.transient_generation
            && self.transient_generation == self.transform_generation
            && self.transform_generation == self.policy_generation
    }

    fn common_generation(&self) -> Option<u64> {
        self.is_commit_eligible()
            .then_some(self.input_history_generation)
    }

    fn staged_copy(&self) -> Self {
        Self {
            prepared_job_input_identity: self.prepared_job_input_identity.clone(),
            input_history_generation: self.input_history_generation,
            input_histories: std::array::from_fn(|lane| self.input_histories[lane].staged_copy()),
            transient_generation: self.transient_generation,
            transient_states: std::array::from_fn(|lane| self.transient_states[lane].staged_copy()),
            transform_generation: self.transform_generation,
            primary_fbw_history_u32_words: self.primary_fbw_history_u32_words,
            companion_fbw_history_u32_words: self.companion_fbw_history_u32_words,
            primary_lfe_history_u32_words: self.primary_lfe_history_u32_words,
            companion_lfe_history_u32_words: self.companion_lfe_history_u32_words,
            policy_generation: self.policy_generation,
            gain_range_previous_peak_f32_bits_u32: self.gain_range_previous_peak_f32_bits_u32,
            gain_range_completed_blocks_u64: self.gain_range_completed_blocks_u64,
            db_per_bit_selector_u16: self.db_per_bit_selector_u16,
            db_per_bit_completed_blocks_u64: self.db_per_bit_completed_blocks_u64,
        }
    }

    /// Runs one noncommitting F07 source prepare and advances only input/transient families.
    pub fn prepare_f07_source(
        &self,
        attempt_id: AttemptId,
        transaction_id: CallbackTransactionId,
        source_index: CodecSourceIndex,
        installed_audio: &InstalledAudioSourceFrame,
        previous_short: [bool; F07_FULLBAND_LANES],
        transient_parameters: &TransientParameters,
    ) -> Result<F07AnalysisStage, AnalysisOwnerStageError> {
        let before_generation = self
            .common_generation()
            .ok_or(AnalysisOwnerStageError::InvalidCommittedGeneration)?;
        let after_generation = before_generation
            .checked_add(1)
            .ok_or(AnalysisOwnerStageError::GenerationOverflow)?;

        let before = F07SourceFrameBeforeState::new(
            std::array::from_fn(|lane| self.input_histories[lane].staged_copy()),
            std::array::from_fn(|lane| self.transient_states[lane].staged_copy()),
            previous_short,
        );
        let f07_proposal = stage_f07a_source_frame(&before, installed_audio, transient_parameters)
            .map_err(AnalysisOwnerStageError::F07)?;

        let mut after_state = self.staged_copy();
        let f07_after_state = f07_proposal.after_state();
        after_state.input_histories = std::array::from_fn(|lane| {
            f07_after_state.input_history_by_external_lane()[lane].staged_copy()
        });
        after_state.transient_states = std::array::from_fn(|lane| {
            f07_after_state.transient_state_by_fbw_lane()[lane].staged_copy()
        });
        after_state.input_history_generation = after_generation;
        after_state.transient_generation = after_generation;

        let before_generation =
            crate::analysis::source_frame::AnalysisOwnerGeneration::from_parent_snapshot(
                before_generation,
            );
        let after_generation =
            crate::analysis::source_frame::AnalysisOwnerGeneration::from_parent_snapshot(
                after_generation,
            );
        let mut source_transaction = match &self.prepared_job_input_identity {
            Some(prepared_job_input_identity) => SourceTransaction::new_for_prepared_input(
                attempt_id,
                transaction_id,
                source_index,
                before_generation,
                Arc::clone(prepared_job_input_identity),
            ),
            None => {
                SourceTransaction::new(attempt_id, transaction_id, source_index, before_generation)
            }
        };
        source_transaction
            .prepare_shared_analysis(after_generation, f07_proposal)
            .map_err(map_transaction_stage_error)?;

        Ok(F07AnalysisStage {
            after_state,
            source_transaction,
        })
    }
}

/// One staged F07 result and its parent-controlled shared-analysis receipt.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct F07AnalysisStage {
    after_state: AnalysisOwnerRoot,
    source_transaction: SourceTransaction,
}

impl F07AnalysisStage {
    pub const fn after_state(&self) -> &AnalysisOwnerRoot {
        &self.after_state
    }

    /// Returns the parent-issued identity and immutable F07 result for this source.
    pub fn shared_analysis(&self) -> Option<&SharedAnalysisReceipt> {
        self.source_transaction.shared_analysis()
    }

    pub(crate) fn into_parts(self) -> (AnalysisOwnerRoot, SourceTransaction) {
        (self.after_state, self.source_transaction)
    }

    /// Completes F08 over all six blocks from the same staged F07 result.
    pub fn complete_f08(
        self,
        parameters: &MathGeneratedTransformParameters,
    ) -> Result<F08AnalysisStage, AnalysisOwnerStageError> {
        let (mut after_state, source_transaction) = self.into_parts();
        let shared_analysis = source_transaction
            .shared_analysis()
            .ok_or(AnalysisOwnerStageError::SourceTransactionRejected)?;
        let proposal = shared_analysis.f07_proposal();
        let delayed_audio = proposal.output().delayed_audio();
        let short_flags = proposal.output().short_consumed();
        let mut fullband_spectra = Vec::with_capacity(F07_BLOCKS_PER_FRAME * F07_FULLBAND_LANES);
        let mut fullband_exponent_inputs =
            Vec::with_capacity(F07_BLOCKS_PER_FRAME * F07_FULLBAND_LANES);
        let mut lfe_spectra = Vec::with_capacity(F07_BLOCKS_PER_FRAME);
        let mut lfe_exponent_inputs = Vec::with_capacity(F07_BLOCKS_PER_FRAME);

        for block in 0..F07_BLOCKS_PER_FRAME {
            for lane in 0..F07_FULLBAND_LANES {
                let external_row = FBW_EXTERNAL_ROWS[lane];
                let offset = lane * TRANSFORM_HISTORY_WORDS_PER_LANE;
                let primary_before = &after_state.primary_fbw_history_u32_words
                    [offset..offset + TRANSFORM_HISTORY_WORDS_PER_LANE];
                let companion_before = &after_state.companion_fbw_history_u32_words
                    [offset..offset + TRANSFORM_HISTORY_WORDS_PER_LANE];
                let folded = f08_window_fold_block(
                    &delayed_audio[external_row][block],
                    primary_before,
                    companion_before,
                    parameters,
                )
                .map_err(AnalysisOwnerStageError::WindowFold)?;
                after_state.primary_fbw_history_u32_words
                    [offset..offset + TRANSFORM_HISTORY_WORDS_PER_LANE]
                    .copy_from_slice(folded.primary_history_after());
                after_state.companion_fbw_history_u32_words
                    [offset..offset + TRANSFORM_HISTORY_WORDS_PER_LANE]
                    .copy_from_slice(folded.companion_history_after());

                let short_flag = i32::from(short_flags[block][lane]);
                let spectra = f08_fbw_transform_block(&folded, short_flag, parameters)
                    .map_err(AnalysisOwnerStageError::FullbandTransform)?;
                let exponent_inputs = f08_full_band_exponent_inputs(&spectra)
                    .map_err(AnalysisOwnerStageError::ExponentInput)?;
                fullband_spectra.push(spectra);
                fullband_exponent_inputs.push(exponent_inputs);
            }

            let folded_lfe = f08_window_fold_block(
                &delayed_audio[LFE_EXTERNAL_ROW][block],
                &after_state.primary_lfe_history_u32_words,
                &after_state.companion_lfe_history_u32_words,
                parameters,
            )
            .map_err(AnalysisOwnerStageError::WindowFold)?;
            after_state
                .primary_lfe_history_u32_words
                .copy_from_slice(folded_lfe.primary_history_after());
            after_state
                .companion_lfe_history_u32_words
                .copy_from_slice(folded_lfe.companion_history_after());
            let spectra = f08_lfe_transform_block(&folded_lfe, parameters)
                .map_err(AnalysisOwnerStageError::LfeTransform)?;
            let exponent_inputs = f08_lfe_exponent_inputs(&spectra)
                .map_err(AnalysisOwnerStageError::ExponentInput)?;
            lfe_spectra.push(spectra);
            lfe_exponent_inputs.push(exponent_inputs);
        }

        let shared_generation = shared_analysis.identity().analysis_after_generation();
        after_state.transform_generation = shared_generation.value();

        Ok(F08AnalysisStage {
            after_state,
            source_transaction,
            plan: F08AnalysisPlan {
                fullband_spectra,
                fullband_exponent_inputs,
                lfe_spectra,
                lfe_exponent_inputs,
            },
        })
    }
}

/// Six-block F08 output retained for the later F09/F10 stages.
#[derive(Debug, Eq, PartialEq)]
pub struct F08AnalysisPlan {
    fullband_spectra: Vec<F08FbwTransformOutput>,
    fullband_exponent_inputs: Vec<FullBandTransformExponentInputs>,
    lfe_spectra: Vec<F08LfeTransformOutput>,
    lfe_exponent_inputs: Vec<LfeTransformExponentInputs>,
}

impl F08AnalysisPlan {
    pub fn is_six_block_complete(&self) -> bool {
        self.fullband_spectra.len() == F07_BLOCKS_PER_FRAME * F07_FULLBAND_LANES
            && self.fullband_exponent_inputs.len() == F07_BLOCKS_PER_FRAME * F07_FULLBAND_LANES
            && self.lfe_spectra.len() == F07_BLOCKS_PER_FRAME
            && self.lfe_exponent_inputs.len() == F07_BLOCKS_PER_FRAME
    }

    pub fn fullband_spectra(&self) -> &[F08FbwTransformOutput] {
        &self.fullband_spectra
    }

    pub fn fullband_exponent_inputs(&self) -> &[FullBandTransformExponentInputs] {
        &self.fullband_exponent_inputs
    }

    pub fn lfe_spectra(&self) -> &[F08LfeTransformOutput] {
        &self.lfe_spectra
    }

    pub fn lfe_exponent_inputs(&self) -> &[LfeTransformExponentInputs] {
        &self.lfe_exponent_inputs
    }
}

/// F08 has advanced only transform history. F09/F10 policy state is still at `G`, so this
/// result is deliberately not commit-eligible and does not produce a NumericPlanReceipt.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct F08AnalysisStage {
    after_state: AnalysisOwnerRoot,
    source_transaction: SourceTransaction,
    plan: F08AnalysisPlan,
}

impl F08AnalysisStage {
    pub const fn after_state(&self) -> &AnalysisOwnerRoot {
        &self.after_state
    }

    /// Returns the same parent-issued receipt used to stage this six-block F08 plan.
    pub fn shared_analysis(&self) -> Option<&SharedAnalysisReceipt> {
        self.source_transaction.shared_analysis()
    }

    pub const fn plan(&self) -> &F08AnalysisPlan {
        &self.plan
    }

    /// Stages selected F09G gain peaks and db-per-bit selector across six delayed blocks.
    ///
    /// The returned root still has the prior policy generation because the F09/F10
    /// exponent and normalization projection remains a separate source-local plan.
    pub fn complete_gain_db(
        self,
        policy_parameters: &PolicyParameters,
    ) -> Result<AnalysisGainDbStage, AnalysisOwnerStageError> {
        if !self.plan.is_six_block_complete() {
            return Err(AnalysisOwnerStageError::IncompleteF08Plan);
        }
        let (mut after_state, source_transaction, f08_plan) = self.into_parts();
        let shared_analysis = source_transaction
            .shared_analysis()
            .ok_or(AnalysisOwnerStageError::SourceTransactionRejected)?;
        let delayed_audio = shared_analysis.f07_proposal().output().delayed_audio();
        let mut gain_state = GainHistoryState::from_parts(
            after_state.gain_range_previous_peak_f32_bits_u32,
            [[0; GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME],
            [0; GAIN_CHANNELS],
            [[0; FULLBAND_GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME],
        );
        let mask = u16::try_from(PolicyProfile::selected().gain_layout_mask)
            .map_err(|_| AnalysisOwnerStageError::SelectedGainMaskInvalid)?;
        let mut gains_by_block = [[0_i16; FULLBAND_GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME];
        let mut db_per_bit_codes_by_block = [0_u8; GAIN_BLOCKS_PER_FRAME];
        let mut gain_output_buffer = [0_i16; FULLBAND_GAIN_CHANNELS + 1];

        for block in 0..GAIN_BLOCKS_PER_FRAME {
            let samples: [[f32; GAIN_BLOCK_SAMPLES]; GAIN_CHANNELS] =
                std::array::from_fn(|internal_lane| {
                    let external_row = if internal_lane < F07_FULLBAND_LANES {
                        FBW_EXTERNAL_ROWS[internal_lane]
                    } else {
                        LFE_EXTERNAL_ROW
                    };
                    delayed_audio[external_row][block].map(f32::from_bits)
                });
            if samples
                .iter()
                .flatten()
                .any(|sample| !sample.is_finite() || sample.abs() > f32::from_bits(0x3fc0_0000))
            {
                return Err(AnalysisOwnerStageError::GainHistoryInput);
            }

            let mut means = [0_u32; GAIN_CHANNELS];
            for (mean, samples) in means.iter_mut().zip(&samples) {
                *mean = checked_mean_square_256(samples)
                    .map_err(AnalysisOwnerStageError::MeanSquare)?;
            }

            let block_index =
                u8::try_from(block).map_err(|_| AnalysisOwnerStageError::BlockIndexOverflow)?;
            let gain_outcome = process_gain_block(
                &mut gain_state,
                block_index,
                mask,
                PeakReductionProfile::UnalignedScalar,
                &samples,
                &mut gain_output_buffer,
            );
            gains_by_block[block] = gain_outcome.gains_after();

            let selector = db_selector_from_means(
                &means,
                after_state.db_per_bit_selector_u16,
                policy_parameters,
            )
            .map_err(AnalysisOwnerStageError::DbSelector)?;
            let selector_code = u8::try_from(selector.selector_after())
                .map_err(|_| AnalysisOwnerStageError::DbSelectorCodeOverflow)?;
            let next_gain_block_count = after_state
                .gain_range_completed_blocks_u64
                .checked_add(1)
                .ok_or(AnalysisOwnerStageError::PolicyCounterOverflow)?;
            let next_db_block_count = after_state
                .db_per_bit_completed_blocks_u64
                .checked_add(1)
                .ok_or(AnalysisOwnerStageError::PolicyCounterOverflow)?;

            after_state.gain_range_previous_peak_f32_bits_u32 = *gain_state.previous_peak_bits();
            after_state.gain_range_completed_blocks_u64 = next_gain_block_count;
            after_state.db_per_bit_selector_u16 = selector.selector_after();
            after_state.db_per_bit_completed_blocks_u64 = next_db_block_count;
            db_per_bit_codes_by_block[block] = selector_code;
        }

        Ok(AnalysisGainDbStage {
            after_state,
            source_transaction,
            f08_plan,
            plan: F09GainDbPlan {
                gain_codes_by_block: gains_by_block,
                db_per_bit_codes_by_block,
            },
        })
    }

    pub(crate) fn into_parts(self) -> (AnalysisOwnerRoot, SourceTransaction, F08AnalysisPlan) {
        (self.after_state, self.source_transaction, self.plan)
    }
}

/// Source-local selected F09G results kept apart from the persistent owner projection.
#[derive(Debug, Eq, PartialEq)]
pub struct F09GainDbPlan {
    gain_codes_by_block: [[i16; FULLBAND_GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME],
    db_per_bit_codes_by_block: [u8; GAIN_BLOCKS_PER_FRAME],
}

impl F09GainDbPlan {
    pub const fn gain_codes_by_block(
        &self,
    ) -> &[[i16; FULLBAND_GAIN_CHANNELS]; GAIN_BLOCKS_PER_FRAME] {
        &self.gain_codes_by_block
    }

    pub const fn db_per_bit_codes_by_block(&self) -> &[u8; GAIN_BLOCKS_PER_FRAME] {
        &self.db_per_bit_codes_by_block
    }
}

/// F09G gain/db proposal after six successful staged blocks.
///
/// It is noncommitting and remains policy-generation incomplete until the same-source F09/F10
/// numeric plan is complete and the parent accepts the source.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct AnalysisGainDbStage {
    after_state: AnalysisOwnerRoot,
    source_transaction: SourceTransaction,
    f08_plan: F08AnalysisPlan,
    plan: F09GainDbPlan,
}

impl AnalysisGainDbStage {
    pub const fn after_state(&self) -> &AnalysisOwnerRoot {
        &self.after_state
    }

    pub fn shared_analysis(&self) -> Option<&SharedAnalysisReceipt> {
        self.source_transaction.shared_analysis()
    }

    pub const fn f08_plan(&self) -> &F08AnalysisPlan {
        &self.f08_plan
    }

    pub const fn plan(&self) -> &F09GainDbPlan {
        &self.plan
    }

    /// Completes the selected six-block F09/F10 plan without committing the after-root.
    pub fn complete_f09_f10(
        self,
        policy_parameters: &PolicyParameters,
    ) -> Result<AnalysisNumericPlanStage, AnalysisOwnerStageError> {
        let Self {
            mut after_state,
            source_transaction,
            f08_plan,
            plan: gain_db_plan,
        } = self;
        let shared_analysis = source_transaction
            .shared_analysis()
            .ok_or(AnalysisOwnerStageError::SourceTransactionRejected)?;
        let f07_output = shared_analysis.f07_proposal().output();
        let exponent_plan =
            stage_f09_f10_source_plan(&f08_plan, f07_output, &gain_db_plan, policy_parameters)
                .map_err(AnalysisOwnerStageError::ExponentPlan)?;
        let short_flags = std::array::from_fn(|block| {
            std::array::from_fn(|lane| f07_output.short_consumed()[block][lane] != 0)
        });
        let dither_flags = *f07_output.dither_flags();
        let previous_short_before = *f07_output.previous_short_before();
        after_state.policy_generation = shared_analysis
            .identity()
            .analysis_after_generation()
            .value();

        Ok(AnalysisNumericPlanStage {
            after_state,
            source_transaction,
            plan: Some(AnalysisNumericPlan {
                f08: f08_plan,
                gain_db: gain_db_plan,
                exponents: exponent_plan,
                short_flags,
                previous_short_before,
                dither_flags,
            }),
        })
    }
}

/// Complete same-source F07-F10 value plan. F11 remains an independent later producer.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct AnalysisNumericPlan {
    f08: F08AnalysisPlan,
    gain_db: F09GainDbPlan,
    exponents: AnalysisExponentPlan,
    short_flags: [[bool; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
    previous_short_before: [bool; F07_FULLBAND_LANES],
    dither_flags: [[bool; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME],
}

impl AnalysisNumericPlan {
    pub const fn f08(&self) -> &F08AnalysisPlan {
        &self.f08
    }

    pub const fn gain_db(&self) -> &F09GainDbPlan {
        &self.gain_db
    }

    pub const fn exponents(&self) -> &AnalysisExponentPlan {
        &self.exponents
    }

    pub const fn short_flags(&self) -> &[[bool; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME] {
        &self.short_flags
    }

    pub const fn previous_short_before(&self) -> &[bool; F07_FULLBAND_LANES] {
        &self.previous_short_before
    }

    pub const fn dither_flags(&self) -> &[[bool; F07_FULLBAND_LANES]; F07_BLOCKS_PER_FRAME] {
        &self.dither_flags
    }
}

/// Noncommitting proposed after-root plus the same-source F07-F10 numeric receipt boundary.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct AnalysisNumericPlanStage {
    after_state: AnalysisOwnerRoot,
    source_transaction: SourceTransaction,
    plan: Option<AnalysisNumericPlan>,
}

impl AnalysisNumericPlanStage {
    pub const fn after_state(&self) -> &AnalysisOwnerRoot {
        &self.after_state
    }

    pub fn shared_analysis(&self) -> Option<&SharedAnalysisReceipt> {
        self.source_transaction.shared_analysis()
    }

    pub const fn plan(&self) -> Option<&AnalysisNumericPlan> {
        self.plan.as_ref()
    }

    /// Creates the one numeric receipt from the same shared analysis identity/provider set.
    pub fn prepare_numeric_plan(
        &mut self,
    ) -> Result<NumericPlanReceipt<'_, AnalysisNumericPlan>, AnalysisOwnerStageError> {
        let plan = self
            .plan
            .take()
            .ok_or(AnalysisOwnerStageError::NumericPlanAlreadyPrepared)?;
        self.source_transaction
            .prepare_numeric_plan(plan)
            .map_err(map_transaction_stage_error)
    }

    /// Parent-only one-shot issuer for a downstream projection of this source's numeric plan.
    ///
    /// The existing AnalysisNumericPlan and its after-state stay in this stage while the
    /// downstream receipt is borrowed and consumed. SourceTransaction supplies the exact same
    /// SharedAnalysisReceipt and rejects a second numeric-plan receipt.
    pub(crate) fn prepare_selected_source_numeric_plan<T>(
        &mut self,
        plan: T,
    ) -> Result<NumericPlanReceipt<'_, T>, AnalysisOwnerStageError> {
        if self.plan.is_none() {
            return Err(AnalysisOwnerStageError::NumericPlanAlreadyPrepared);
        }
        self.source_transaction
            .prepare_numeric_plan(plan)
            .map_err(map_transaction_stage_error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisOwnerStageError {
    InvalidCommittedGeneration,
    GenerationOverflow,
    SourceTransactionRejected,
    IncompleteF08Plan,
    SelectedGainMaskInvalid,
    BlockIndexOverflow,
    GainHistoryInput,
    MeanSquare(MeanSquareError),
    DbSelector(DbSelectorError),
    DbSelectorCodeOverflow,
    PolicyCounterOverflow,
    ExponentPlan(AnalysisExponentPlanError),
    NumericPlanAlreadyPrepared,
    F07(F07SourceFrameError),
    WindowFold(F08WindowFoldError),
    FullbandTransform(F08FbwTransformError),
    LfeTransform(F08LfeTransformError),
    ExponentInput(TransformExponentInputError),
}

fn map_transaction_stage_error(_: SourceTransactionStageError) -> AnalysisOwnerStageError {
    AnalysisOwnerStageError::SourceTransactionRejected
}

#[cfg(test)]
mod tests {
    use super::{
        AnalysisOwnerGtcReceiptError, AnalysisOwnerRoot, AnalysisOwnerStageError,
        AnalysisOwnerStateReferenceError, F07_BLOCKS_PER_FRAME,
    };
    use crate::analysis::exponent_plan::{AnalysisBlockIndex, FullbandLaneIndex};
    use crate::analysis::input_history::INPUT_HISTORY_BLOCK_SAMPLES;
    use crate::analysis::source_frame::F07SourceFrameError;
    use crate::analysis::source_frame::InstalledAudioSourceFrame;
    use crate::analysis::source_frame::zeroed_shared_analysis_receipt_for_identity_tests;
    use crate::analysis::transient_channel::TransientChannelError;
    use crate::analysis::transient_parameters::load_transient_parameters;
    use crate::codec::exponent_normalization::{
        FULLBAND_EXPONENT_BINS, LFE_EXPONENT_BINS, normalize_fullband_exponents,
        normalize_lfe_exponents,
    };
    use crate::codec::exponent_refresh_state::refresh_exponent_state;
    use crate::codec::policy_parameters::{PolicyProfile, load_policy_parameters};
    use crate::meter::prepared_job_identity::{
        NormalizedJobConfigurationIdentity, PreparedAdmChunkSpan, PreparedAdmGraphProvenance,
        PreparedInputResourceIdentity, PreparedJobInputIdentity,
    };
    use crate::timeline::{CodecSourceIndex, FrontendCallbackIndex};
    use crate::transaction::{AttemptId, CallbackTransactionId, GtcOwnerId, GtcOwnerReceiptError};
    use crate::transform::math_generated_parameters::MathGeneratedTransformParameters;
    use serde_json::Value;
    use sha2::{Digest, Sha256};
    use std::collections::BTreeMap;
    use std::sync::Arc;

    const D05_PARAMETERS: &[u8] =
        include_bytes!("../../tests/fixtures/transient_channel/PARAMETERS.json");
    const F08_PARAMETERS: &[u8] =
        include_bytes!("../transform/ordinary_transform_math_parameters.json");
    const POLICY_PARAMETERS: &[u8] =
        include_bytes!("../../tests/fixtures/policy_authorized/PARAMETERS.json");
    const POLICY_AMENDMENT_VECTORS: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/F07-F12-POLICY-ROOT-STATE-AMENDMENT-V1/INPUT-ONLY-VECTORS.json"
    );
    const POLICY_AMENDMENT_VECTORS_SHA256: &str =
        "0c0fed50ae080623ec62771a2cfeaa0fd562a3c694db08cf2d1e9aa213a5a039";
    const ANALYSIS_COLD_PREIMAGE: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/ANALYSIS-ROOT-CANONICAL-STATE-REFERENCE-V1/COLD-ROOT-PREIMAGE.json"
    );
    const POLICY_ROOT_AMENDMENT: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/F07-F12-POLICY-ROOT-STATE-AMENDMENT-V1/POLICY-ROOT-AMENDMENT.json"
    );

    fn source_transaction() -> (AttemptId, CallbackTransactionId, CodecSourceIndex) {
        (
            AttemptId::new(3),
            CallbackTransactionId::new("analysis-root-test").unwrap(),
            CodecSourceIndex::new(0),
        )
    }

    fn prepared_job_identity() -> Arc<PreparedJobInputIdentity> {
        let resource = PreparedInputResourceIdentity::new([0x21; 32], 1_024);
        let graph = PreparedAdmGraphProvenance::new(
            resource,
            PreparedAdmChunkSpan::new(*b"axml", 10, 10, [0x31; 32]),
            PreparedAdmChunkSpan::new(*b"chna", 20, 10, [0x32; 32]),
            PreparedAdmChunkSpan::new(*b"dbmd", 30, 10, [0x33; 32]),
        );
        Arc::new(
            PreparedJobInputIdentity::new(
                resource,
                NormalizedJobConfigurationIdentity::new(
                    [0x22; 32],
                    [0x23; 32],
                    crate::frontend::job_configuration::SelectedInputKind::Adm,
                ),
                graph,
            )
            .unwrap(),
        )
    }

    fn complete_nonzero_root(
        prepared_identity: Option<Arc<PreparedJobInputIdentity>>,
    ) -> AnalysisOwnerRoot {
        let mut root = match prepared_identity {
            Some(identity) => AnalysisOwnerRoot::cold_for_prepared_input(identity),
            None => AnalysisOwnerRoot::cold(),
        };
        root.input_history_generation = 1;
        root.transient_generation = 1;
        root.transform_generation = 1;
        root.policy_generation = 1;
        root.gain_range_previous_peak_f32_bits_u32 = [0x3f80_0000; 6];
        root.gain_range_completed_blocks_u64 = 17;
        root.db_per_bit_selector_u16 = 2;
        root.db_per_bit_completed_blocks_u64 = 17;
        root
    }

    fn zero_audio() -> Box<InstalledAudioSourceFrame> {
        vec![[[0_u32; INPUT_HISTORY_BLOCK_SAMPLES]; F07_BLOCKS_PER_FRAME]; 6]
            .into_boxed_slice()
            .try_into()
            .unwrap()
    }

    fn varying_audio() -> Box<InstalledAudioSourceFrame> {
        let mut audio = zero_audio();
        for (channel, channel_blocks) in audio.iter_mut().enumerate() {
            for (block, samples) in channel_blocks.iter_mut().enumerate() {
                for (sample, word) in samples.iter_mut().enumerate() {
                    let value = if block == 0 {
                        0.0
                    } else {
                        let residue = (channel * 47 + block * 31 + sample * 13) % 257;
                        (i32::try_from(residue).unwrap() - 128) as f32 / 512.0
                    };
                    *word = value.to_bits();
                }
            }
        }
        audio
    }

    fn expected_fullband_strategies(
        refresh_markers: &[[i16; 5]; F07_BLOCKS_PER_FRAME],
        short_consumed: &[[u8; 5]; F07_BLOCKS_PER_FRAME],
    ) -> [[u8; 5]; F07_BLOCKS_PER_FRAME] {
        let mut strategies = [[0_u8; 5]; F07_BLOCKS_PER_FRAME];
        for lane in 0..5 {
            let mut refresh_distance = 0_usize;
            for block in (0..F07_BLOCKS_PER_FRAME).rev() {
                let strategy = match refresh_markers[block][lane] {
                    0 => {
                        refresh_distance += 1;
                        0
                    }
                    1 => {
                        let strategy = if refresh_distance < 2 {
                            3
                        } else if refresh_distance < 4 {
                            2
                        } else {
                            1
                        };
                        refresh_distance = 0;
                        strategy
                    }
                    _ => panic!("F09 returned a non-I1 refresh marker"),
                };
                strategies[block][lane] = if short_consumed[block][lane] == 1 {
                    3
                } else {
                    strategy
                };
            }
        }
        strategies
    }

    fn selected_policy_parameters() -> crate::codec::policy_parameters::PolicyParameters {
        load_policy_parameters(PolicyProfile::selected(), Some(POLICY_PARAMETERS)).unwrap()
    }

    #[test]
    fn cold_and_reset_match_exact_four_family_schema() {
        let mut root = AnalysisOwnerRoot::cold();
        assert!(root.is_commit_eligible());
        assert_eq!(root.input_history_generation(), 0);
        assert_eq!(root.transient_generation(), 0);
        assert_eq!(root.transform_generation(), 0);
        assert_eq!(root.policy_generation(), 0);
        assert_eq!(root.input_histories().len(), 6);
        assert_eq!(root.transient_states().len(), 5);
        assert_eq!(root.primary_fbw_history_u32_words().len(), 640);
        assert_eq!(root.companion_fbw_history_u32_words().len(), 640);
        assert_eq!(root.primary_lfe_history_u32_words().len(), 128);
        assert_eq!(root.companion_lfe_history_u32_words().len(), 128);
        assert_eq!(root.gain_range_previous_peak_f32_bits_u32().len(), 6);
        assert_eq!(root.gain_range_completed_blocks_u64(), 0);
        assert_eq!(root.db_per_bit_selector_u16(), 0);
        assert_eq!(root.db_per_bit_completed_blocks_u64(), 0);
        assert!(
            root.input_histories()
                .iter()
                .all(|history| history.words().iter().all(|word| *word == 0))
        );
        assert!(
            root.transient_states()
                .iter()
                .all(|state| state.canonical_bytes().iter().all(|byte| *byte == 0))
        );
        assert!(
            root.primary_fbw_history_u32_words()
                .iter()
                .chain(root.companion_fbw_history_u32_words())
                .chain(root.primary_lfe_history_u32_words())
                .chain(root.companion_lfe_history_u32_words())
                .all(|word| *word == 0)
        );
        assert!(
            root.gain_range_previous_peak_f32_bits_u32()
                .iter()
                .all(|word| *word == 0)
        );

        root.reset();
        assert_eq!(root, AnalysisOwnerRoot::cold());
    }

    #[test]
    fn owner_state_reference_serializes_the_complete_v6_root_as_sorted_json() {
        let mut root = AnalysisOwnerRoot::cold();
        root.input_history_generation = 1;
        root.transient_generation = 1;
        root.transform_generation = 1;
        root.policy_generation = 1;
        root.gain_range_previous_peak_f32_bits_u32 = [
            0x3dcc_cccd,
            0x3e4c_cccd,
            0x3e99_999a,
            0x3ecc_cccd,
            0x3f00_0000,
            0x3f19_999a,
        ];
        root.gain_range_completed_blocks_u64 = 17;
        root.db_per_bit_selector_u16 = 2;
        root.db_per_bit_completed_blocks_u64 = 17;

        let reference = root
            .state_reference()
            .expect("complete four-family root has a state reference");
        let parsed: BTreeMap<String, Value> =
            serde_json::from_slice(reference.canonical_bytes()).unwrap();
        let mut expected_keys = vec![
            "input_history_generation",
            "input_history_u32_words",
            "transient_generation",
            "transient_two_history_u32_words",
            "transient_three_history_u32_words",
            "transient_peak_statistics_u32_words",
            "transient_previous_mean_u32_words",
            "transient_short_queue_i1",
            "transient_other_queue_i1",
            "transform_generation",
            "primary_fbw_history_u32_words",
            "companion_fbw_history_u32_words",
            "primary_lfe_history_u32_words",
            "companion_lfe_history_u32_words",
            "policy_generation",
            "gain_range",
            "db_per_bit",
        ];
        expected_keys.sort_unstable();
        assert_eq!(
            parsed.keys().map(String::as_str).collect::<Vec<_>>(),
            expected_keys
        );
        assert_eq!(
            serde_json::to_vec(&parsed).unwrap(),
            reference.canonical_bytes(),
            "the preimage is compact UTF-8 JSON with sorted keys"
        );
        assert_eq!(reference.owner_id(), "analysis");
        assert_eq!(reference.state_ref(), "owners.analysis");
        assert_eq!(reference.generation(), 1);
        assert_eq!(reference.state(), &root);
        for (field, length) in [
            ("input_history_u32_words", 7_680),
            ("transient_two_history_u32_words", 20),
            ("transient_three_history_u32_words", 30),
            ("transient_peak_statistics_u32_words", 90),
            ("transient_previous_mean_u32_words", 5),
            ("transient_short_queue_i1", 20),
            ("transient_other_queue_i1", 20),
            ("primary_fbw_history_u32_words", 640),
            ("companion_fbw_history_u32_words", 640),
            ("primary_lfe_history_u32_words", 128),
            ("companion_lfe_history_u32_words", 128),
        ] {
            assert_eq!(parsed[field].as_array().unwrap().len(), length, "{field}");
        }
        assert_eq!(
            parsed["gain_range"]["previous_peak_f32_bits_u32"]
                .as_array()
                .unwrap()
                .len(),
            6
        );
        assert_eq!(
            parsed["gain_range"]["previous_peak_f32_bits_u32"],
            serde_json::json!([
                0x3dcc_cccd_u32,
                0x3e4c_cccd_u32,
                0x3e99_999a_u32,
                0x3ecc_cccd_u32,
                0x3f00_0000_u32,
                0x3f19_999a_u32
            ])
        );
        assert_eq!(parsed["gain_range"]["completed_blocks_u64"], 17);
        assert_eq!(parsed["db_per_bit"]["selector_u16"], 2);
        assert_eq!(parsed["db_per_bit"]["completed_blocks_u64"], 17);
        let current_hash: [u8; 32] = Sha256::digest(reference.canonical_bytes()).into();
        let cold_root = AnalysisOwnerRoot::cold();
        let cold_reference = cold_root.state_reference().unwrap();
        let cold_bytes = cold_reference.canonical_bytes().to_vec();
        let cold_hash: [u8; 32] = Sha256::digest(&cold_bytes).into();
        assert_eq!(reference.state_sha256(), &current_hash);
        assert_eq!(reference.cold_sha256(), cold_reference.cold_sha256());
        assert_eq!(cold_reference.state_sha256(), &cold_hash);
        assert_ne!(reference.state_sha256(), reference.cold_sha256());
        let cold_hash_hex = format!("{:x}", Sha256::digest(&cold_bytes));
        assert_eq!(cold_bytes, ANALYSIS_COLD_PREIMAGE);
        assert_eq!(cold_bytes.len(), 19_422);
        assert_eq!(
            cold_hash_hex, "008167df2194d7ffebc2ba0aa1e28f70067afda1a089160c46ad78b40e3c0f23",
            "cold identity matches the approved canonical preimage"
        );

        let amendment: Value = serde_json::from_slice(POLICY_ROOT_AMENDMENT).unwrap();
        assert_eq!(
            amendment["schema_version"],
            "F07_F12_OWNER_STATE_V6_POLICY_AMENDMENT"
        );
        assert_eq!(
            amendment["persistent_fields"]["gain_range"]["previous_peak_f32_bits_u32"],
            serde_json::json!([6])
        );
        assert_eq!(
            amendment["persistent_fields"]["gain_range"]["completed_blocks_u64"],
            0
        );
        assert_eq!(
            amendment["persistent_fields"]["db_per_bit"]["selector_u16"],
            0
        );
        assert_eq!(
            amendment["persistent_fields"]["db_per_bit"]["completed_blocks_u64"],
            0
        );

        root.reset();
        let reset_reference = root
            .state_reference()
            .expect("reset reconstructs a complete cold root");
        assert_eq!(root, AnalysisOwnerRoot::cold());
        assert_eq!(reset_reference.canonical_bytes(), cold_bytes);
        assert_eq!(
            reset_reference.state_sha256(),
            reset_reference.cold_sha256()
        );
    }

    #[test]
    fn gtc_snapshot_uses_the_complete_analysis_v6_preimage() {
        let root = AnalysisOwnerRoot::cold();
        let reference = root.state_reference().unwrap();
        let snapshot = reference.gtc_snapshot().unwrap();

        assert_eq!(snapshot.owner_id(), GtcOwnerId::Analysis);
        assert_eq!(snapshot.state_ref().as_str(), "owners.analysis");
        assert_eq!(snapshot.generation(), reference.generation());
        assert_eq!(snapshot.state_sha256(), *reference.state_sha256());
        assert_eq!(snapshot.cold_sha256(), *reference.cold_sha256());
        assert_eq!(snapshot.canonical_bytes(), reference.canonical_bytes());
        assert_eq!(snapshot.canonical_bytes().len(), 19_422);
    }

    #[test]
    fn callbacks_zero_and_one_produce_unchanged_analysis_owner_receipts() {
        let root = AnalysisOwnerRoot::cold_for_prepared_input(prepared_job_identity());

        for callback_index in [0, 1] {
            let transaction_id =
                CallbackTransactionId::new(&format!("analysis-callback-{callback_index}")).unwrap();
            let receipt = root
                .gtc_unchanged_callback_receipt(
                    FrontendCallbackIndex::new(callback_index),
                    transaction_id.clone(),
                )
                .unwrap();

            assert_eq!(receipt.owner_id(), GtcOwnerId::Analysis);
            assert_eq!(receipt.before_generation(), 0);
            assert_eq!(receipt.after_generation(), 0);
            assert_eq!(receipt.before_sha256(), receipt.after_sha256());
            assert!(!receipt.changed());
            assert_eq!(receipt.commit_count(), 0);
            assert_eq!(receipt.transaction_id(), &transaction_id);
        }

        assert_eq!(
            root.gtc_unchanged_callback_receipt(
                FrontendCallbackIndex::new(2),
                CallbackTransactionId::new("analysis-callback-2").unwrap(),
            ),
            Err(AnalysisOwnerGtcReceiptError::SourceStageRequired)
        );
    }

    #[test]
    fn changed_analysis_root_receipt_binds_cold_hash_generation_and_parent_token() {
        let identity = prepared_job_identity();
        let before = AnalysisOwnerRoot::cold_for_prepared_input(Arc::clone(&identity));
        let after = complete_nonzero_root(Some(Arc::clone(&identity)));
        let transaction_id = CallbackTransactionId::new("analysis-source-transaction").unwrap();

        let receipt = before
            .gtc_owner_receipt_to(&after, transaction_id.clone())
            .unwrap();
        let before_reference = before.state_reference().unwrap();
        let after_reference = after.state_reference().unwrap();

        assert!(Arc::ptr_eq(
            before.prepared_job_input_identity().unwrap(),
            after.prepared_job_input_identity().unwrap()
        ));
        assert_eq!(receipt.owner_id(), GtcOwnerId::Analysis);
        assert_eq!(receipt.before_generation(), before_reference.generation());
        assert_eq!(receipt.after_generation(), after_reference.generation());
        assert_eq!(receipt.before_sha256(), *before_reference.state_sha256());
        assert_eq!(receipt.after_sha256(), *after_reference.state_sha256());
        assert!(receipt.changed());
        assert_eq!(receipt.commit_count(), 1);
        assert_eq!(receipt.transaction_id(), &transaction_id);
        assert_eq!(before, AnalysisOwnerRoot::cold_for_prepared_input(identity));
    }

    #[test]
    fn analysis_owner_receipt_rejects_a_different_prepared_input_arc() {
        let before_identity = prepared_job_identity();
        let after_identity = prepared_job_identity();
        assert_eq!(*before_identity, *after_identity);
        assert!(!Arc::ptr_eq(&before_identity, &after_identity));

        let before = AnalysisOwnerRoot::cold_for_prepared_input(before_identity);
        let after = complete_nonzero_root(Some(after_identity));
        assert_eq!(
            before.gtc_owner_receipt_to(
                &after,
                CallbackTransactionId::new("analysis-foreign-input").unwrap(),
            ),
            Err(AnalysisOwnerGtcReceiptError::PreparedInputIdentityMismatch)
        );
    }

    #[test]
    fn analysis_owner_receipt_rejects_skipped_generation() {
        let identity = prepared_job_identity();
        let before = complete_nonzero_root(Some(Arc::clone(&identity)));
        let mut skipped_generation = complete_nonzero_root(Some(Arc::clone(&identity)));
        skipped_generation.input_history_generation = 3;
        skipped_generation.transient_generation = 3;
        skipped_generation.transform_generation = 3;
        skipped_generation.policy_generation = 3;

        assert_eq!(
            before.gtc_owner_receipt_to(
                &skipped_generation,
                CallbackTransactionId::new("analysis-generation-mismatch").unwrap(),
            ),
            Err(AnalysisOwnerGtcReceiptError::Gtc(
                GtcOwnerReceiptError::GenerationMismatch
            ))
        );
    }

    #[test]
    fn analysis_owner_gtc_receipts_reject_unbound_prepared_input_identity() {
        let before = AnalysisOwnerRoot::cold();
        let unbound_after = complete_nonzero_root(None);
        let transaction_id = CallbackTransactionId::new("analysis-unbound-input").unwrap();

        assert_eq!(
            before.gtc_owner_receipt_to(&unbound_after, transaction_id.clone()),
            Err(AnalysisOwnerGtcReceiptError::PreparedInputIdentityMissing)
        );
        assert_eq!(
            before.gtc_unchanged_callback_receipt(FrontendCallbackIndex::new(0), transaction_id,),
            Err(AnalysisOwnerGtcReceiptError::PreparedInputIdentityMissing)
        );
    }

    #[test]
    fn incomplete_analysis_root_cannot_be_published_as_an_owner_reference() {
        let mut partial = AnalysisOwnerRoot::cold();
        partial.input_history_generation = 1;
        assert_eq!(
            partial.state_reference(),
            Err(AnalysisOwnerStateReferenceError::IncompleteRoot)
        );
    }

    #[test]
    fn f07_and_f08_advance_only_their_approved_generation_families() {
        let root = AnalysisOwnerRoot::cold();
        let parameters = load_transient_parameters(Some(D05_PARAMETERS)).unwrap();
        let transform_parameters =
            MathGeneratedTransformParameters::from_provider_bytes(F08_PARAMETERS).unwrap();
        let (attempt, transaction, source) = source_transaction();
        let f07 = root
            .prepare_f07_source(
                attempt,
                transaction,
                source,
                &zero_audio(),
                [false; 5],
                &parameters,
            )
            .unwrap();
        assert!(!f07.after_state().is_commit_eligible());
        assert_eq!(f07.after_state().input_history_generation(), 1);
        assert_eq!(f07.after_state().transient_generation(), 1);
        assert_eq!(f07.after_state().transform_generation(), 0);
        assert_eq!(f07.after_state().policy_generation(), 0);
        let shared = f07.shared_analysis().unwrap();
        assert_eq!(shared.advancement_count(), 1);
        assert_eq!(shared.identity().source_index(), source);
        let provider_lineage = shared.provider_lineage();
        assert_eq!(
            provider_lineage.transient_sha256(),
            "130cd1a446ea8413a1d0d4c5eea631e890b7f1cd675975efa92697e258402991"
        );
        assert_eq!(
            provider_lineage.transform_sha256(),
            "f1fb1a16b3caedab6ddc583201b9454755fd8871ecdf7881a6a3ffbad10fddf1"
        );
        assert_eq!(
            provider_lineage.policy_sha256(),
            "a99d077900058eb4a862f5ea29bc87408bf6d7a4f4cc42e3da66ae6c68a93c6a"
        );
        assert_eq!(
            provider_lineage.difference_map_sha256(),
            "6cf3ae8d899b334d5aa87e0744e4c58b7dba1d92177ca07738726f8ea0ee2c6d"
        );

        let f08 = f07.complete_f08(&transform_parameters).unwrap();
        assert!(!f08.after_state().is_commit_eligible());
        assert_eq!(f08.after_state().input_history_generation(), 1);
        assert_eq!(f08.after_state().transient_generation(), 1);
        assert_eq!(f08.after_state().transform_generation(), 1);
        assert_eq!(f08.after_state().policy_generation(), 0);
        assert_eq!(f08.after_state().gain_range_completed_blocks_u64(), 0);
        assert_eq!(f08.after_state().db_per_bit_completed_blocks_u64(), 0);
        assert!(f08.plan().is_six_block_complete());
        assert_eq!(f08.plan().fullband_spectra().len(), 30);
        assert_eq!(f08.plan().fullband_exponent_inputs().len(), 30);
        assert_eq!(f08.plan().lfe_spectra().len(), 6);
        assert_eq!(f08.plan().lfe_exponent_inputs().len(), 6);
        assert!(matches!(
            f08.after_state().prepare_f07_source(
                attempt,
                CallbackTransactionId::new("analysis-root-after-f08").unwrap(),
                CodecSourceIndex::new(1),
                &zero_audio(),
                [false; 5],
                &parameters,
            ),
            Err(AnalysisOwnerStageError::InvalidCommittedGeneration)
        ));
    }

    #[test]
    fn prepared_input_capability_flows_from_cold_analysis_root_into_shared_source_receipt() {
        let prepared_identity = prepared_job_identity();
        let mut root = AnalysisOwnerRoot::cold_for_prepared_input(Arc::clone(&prepared_identity));
        let parameters = load_transient_parameters(Some(D05_PARAMETERS)).unwrap();
        let (attempt, transaction, source) = source_transaction();
        let f07 = root
            .prepare_f07_source(
                attempt,
                transaction,
                source,
                &zero_audio(),
                [false; 5],
                &parameters,
            )
            .unwrap();
        let shared = f07.shared_analysis().unwrap();
        assert!(Arc::ptr_eq(
            &prepared_identity,
            root.prepared_job_input_identity().unwrap()
        ));
        assert!(Arc::ptr_eq(
            &prepared_identity,
            f07.after_state().prepared_job_input_identity().unwrap()
        ));
        assert!(Arc::ptr_eq(
            &prepared_identity,
            shared.identity().prepared_job_input_identity().unwrap()
        ));

        drop(f07);
        root.reset();
        assert!(Arc::ptr_eq(
            &prepared_identity,
            root.prepared_job_input_identity().unwrap()
        ));
    }

    #[test]
    fn gain_and_db_stage_carries_the_two_approved_policy_histories() {
        let mut root = AnalysisOwnerRoot::cold();
        root.gain_range_previous_peak_f32_bits_u32 = [
            0x3dcc_cccd,
            0x3e4c_cccd,
            0x3e99_999a,
            0x3ecc_cccd,
            0x3f00_0000,
            0x3f19_999a,
        ];
        root.gain_range_completed_blocks_u64 = 17;
        root.db_per_bit_selector_u16 = 2;
        root.db_per_bit_completed_blocks_u64 = 17;
        let committed_before = root.staged_copy();

        let transient_parameters = load_transient_parameters(Some(D05_PARAMETERS)).unwrap();
        let transform_parameters =
            MathGeneratedTransformParameters::from_provider_bytes(F08_PARAMETERS).unwrap();
        let policy_parameters = selected_policy_parameters();
        let (attempt, transaction, source) = source_transaction();
        let f07 = root
            .prepare_f07_source(
                attempt,
                transaction,
                source,
                &zero_audio(),
                [false; 5],
                &transient_parameters,
            )
            .unwrap();
        let f08 = f07.complete_f08(&transform_parameters).unwrap();
        let gain_db = f08.complete_gain_db(&policy_parameters).unwrap();

        assert!(!gain_db.after_state().is_commit_eligible());
        assert_eq!(gain_db.after_state().policy_generation(), 0);
        assert_eq!(gain_db.after_state().gain_range_completed_blocks_u64(), 23);
        assert_eq!(gain_db.after_state().db_per_bit_completed_blocks_u64(), 23);
        assert_eq!(
            gain_db.after_state().db_per_bit_selector_u16(),
            u16::from(*gain_db.plan().db_per_bit_codes_by_block().last().unwrap())
        );
        assert!(
            gain_db
                .plan()
                .db_per_bit_codes_by_block()
                .iter()
                .all(|selector| *selector <= 3)
        );
        assert_eq!(gain_db.plan().gain_codes_by_block().len(), 6);
        assert_eq!(root, committed_before);
    }

    #[test]
    fn failure_after_a_f09g_block_drops_all_staged_policy_updates() {
        let mut root = AnalysisOwnerRoot::cold();
        root.gain_range_completed_blocks_u64 = 17;
        root.db_per_bit_selector_u16 = 1;
        root.db_per_bit_completed_blocks_u64 = u64::MAX - 2;
        let committed_before = root.staged_copy();

        let transient_parameters = load_transient_parameters(Some(D05_PARAMETERS)).unwrap();
        let transform_parameters =
            MathGeneratedTransformParameters::from_provider_bytes(F08_PARAMETERS).unwrap();
        let policy_parameters = selected_policy_parameters();
        let (attempt, transaction, source) = source_transaction();
        let f07 = root
            .prepare_f07_source(
                attempt,
                transaction,
                source,
                &zero_audio(),
                [false; 5],
                &transient_parameters,
            )
            .unwrap();
        let f08 = f07.complete_f08(&transform_parameters).unwrap();

        assert!(matches!(
            f08.complete_gain_db(&policy_parameters),
            Err(AnalysisOwnerStageError::PolicyCounterOverflow)
        ));
        assert_eq!(root, committed_before);
    }

    #[test]
    fn policy_root_input_only_vectors_are_hash_bound_without_expected_outputs() {
        use sha2::{Digest, Sha256};

        assert_eq!(
            format!("{:x}", Sha256::digest(POLICY_AMENDMENT_VECTORS)),
            POLICY_AMENDMENT_VECTORS_SHA256
        );
        let vectors: serde_json::Value = serde_json::from_slice(POLICY_AMENDMENT_VECTORS).unwrap();
        assert_eq!(vectors["schema"], "policy-root-projection-input-only.v1");
        assert_eq!(vectors["expected_projection_values_included"], false);
        assert_eq!(vectors["expected_hashes_included"], false);

        let cases = vectors["cases"].as_array().unwrap();
        let gain_carry = cases
            .iter()
            .find(|case| case["id"] == "gain_peak_cross_source_carry")
            .unwrap();
        assert_eq!(gain_carry["source"], 2);
        assert_eq!(gain_carry["block"], 5);
        assert_eq!(gain_carry["gain_completed_blocks_u64"], 17);
        assert_eq!(
            gain_carry["gain_previous_current_peak_f32_bits"]
                .as_array()
                .unwrap()
                .len(),
            6
        );

        let selector_carry = cases
            .iter()
            .find(|case| case["id"] == "db_selector_cross_source_carry")
            .unwrap();
        assert_eq!(selector_carry["db_selector_u16"], 2);
        assert_eq!(selector_carry["db_completed_blocks_u64"], 17);

        let local_work = cases
            .iter()
            .find(|case| case["id"] == "source_local_work_not_root")
            .unwrap();
        assert_eq!(local_work["fbw_work_shape"], serde_json::json!([5, 217]));
        assert_eq!(local_work["lfe_work_shape"], serde_json::json!([7]));
        assert_eq!(local_work["initial_value"], 0);
    }

    #[test]
    fn source_identity_is_parent_supplied_and_not_derived_from_generation() {
        let root = AnalysisOwnerRoot::cold();
        let parameters = load_transient_parameters(Some(D05_PARAMETERS)).unwrap();
        let attempt = AttemptId::new(3);
        let transaction = CallbackTransactionId::new("analysis-root-source-identity").unwrap();
        let source = CodecSourceIndex::new(37);
        let f07 = root
            .prepare_f07_source(
                attempt,
                transaction,
                source,
                &zero_audio(),
                [false; 5],
                &parameters,
            )
            .unwrap();

        let identity = f07.shared_analysis().unwrap().identity();
        assert_eq!(identity.source_index(), source);
        assert_eq!(identity.analysis_after_generation().value(), 1);
    }

    #[test]
    fn test_fixture_receipt_preserves_parent_source_identity() {
        let attempt = AttemptId::new(5);
        let transaction_id = CallbackTransactionId::new("identity-fixture-test").unwrap();
        let source = CodecSourceIndex::new(19);
        let receipt = zeroed_shared_analysis_receipt_for_identity_tests(
            attempt,
            transaction_id,
            source,
            0,
            1,
        );

        assert_eq!(receipt.advancement_count(), 1);
        assert_eq!(receipt.identity().source_index(), source);
        assert_eq!(receipt.identity().analysis_after_generation().value(), 1);
    }

    #[test]
    fn f07_prepared_root_cannot_start_another_source_before_policy_completion() {
        let root = AnalysisOwnerRoot::cold();
        let parameters = load_transient_parameters(Some(D05_PARAMETERS)).unwrap();
        let (attempt, transaction, source) = source_transaction();
        let f07 = root
            .prepare_f07_source(
                attempt,
                transaction,
                source,
                &zero_audio(),
                [false; 5],
                &parameters,
            )
            .unwrap();

        assert!(matches!(
            f07.after_state().prepare_f07_source(
                attempt,
                CallbackTransactionId::new("analysis-root-next-source").unwrap(),
                CodecSourceIndex::new(1),
                &zero_audio(),
                [false; 5],
                &parameters,
            ),
            Err(AnalysisOwnerStageError::InvalidCommittedGeneration)
        ));
        assert!(root.is_commit_eligible());
        assert_eq!(root.input_history_generation(), 0);
    }

    #[test]
    fn late_f07_block_failure_discards_all_staged_owner_values() {
        let root = AnalysisOwnerRoot::cold();
        let parameters = load_transient_parameters(Some(D05_PARAMETERS)).unwrap();
        let (attempt, transaction, source) = source_transaction();
        let mut audio = zero_audio();
        audio[5][5][INPUT_HISTORY_BLOCK_SAMPLES - 1] = f32::NAN.to_bits();
        assert!(matches!(
            root.prepare_f07_source(
                attempt,
                transaction,
                source,
                &audio,
                [false; 5],
                &parameters,
            ),
            Err(AnalysisOwnerStageError::F07(
                F07SourceFrameError::Transient(TransientChannelError::InputError)
            ))
        ));
        assert_eq!(root, AnalysisOwnerRoot::cold());
    }

    #[test]
    fn late_f08_failure_discards_the_cloned_analysis_root() {
        let mut root = AnalysisOwnerRoot::cold();
        let mut lfe_history = crate::analysis::input_history::InputHistory::cold();
        let maximum_block = [f32::MAX.to_bits(); INPUT_HISTORY_BLOCK_SAMPLES];
        for _ in 0..4 {
            let (_, next) = lfe_history.advance_block(&maximum_block);
            lfe_history = next;
        }
        root.input_histories[super::super::source_frame::LFE_EXTERNAL_ROW] = lfe_history;
        let committed_before = root.staged_copy();

        let parameters = load_transient_parameters(Some(D05_PARAMETERS)).unwrap();
        let transform_parameters =
            MathGeneratedTransformParameters::from_provider_bytes(F08_PARAMETERS).unwrap();
        let (attempt, transaction, source) = source_transaction();
        let f07 = root
            .prepare_f07_source(
                attempt,
                transaction,
                source,
                &zero_audio(),
                [false; 5],
                &parameters,
            )
            .unwrap();

        assert!(matches!(
            f07.complete_f08(&transform_parameters),
            Err(AnalysisOwnerStageError::WindowFold(_)
                | AnalysisOwnerStageError::FullbandTransform(_)
                | AnalysisOwnerStageError::LfeTransform(_)
                | AnalysisOwnerStageError::ExponentInput(_))
        ));
        assert_eq!(root, committed_before);
    }

    #[test]
    fn wrong_provider_identity_is_rejected_before_a_typed_transform_provider_exists() {
        let root = AnalysisOwnerRoot::cold();
        let parameters = load_transient_parameters(Some(D05_PARAMETERS)).unwrap();
        let (attempt, transaction, source) = source_transaction();
        let f07 = root
            .prepare_f07_source(
                attempt,
                transaction,
                source,
                &zero_audio(),
                [false; 5],
                &parameters,
            )
            .unwrap();
        assert!(MathGeneratedTransformParameters::from_provider_bytes(b"{}").is_err());
        assert!(load_transient_parameters(Some(b"{}")).is_err());
        assert_eq!(f07.after_state().transform_generation(), 0);
        assert_eq!(root, AnalysisOwnerRoot::cold());
    }

    #[test]
    fn f09_f10_stage_uses_structure_power_primary_and_independent_lfe_controls() {
        const FBW_LANES: usize = 5;
        const FBW_BINS: usize = FULLBAND_EXPONENT_BINS;
        const LFE_BINS: usize = LFE_EXPONENT_BINS;

        let root = AnalysisOwnerRoot::cold();
        let transient_parameters = load_transient_parameters(Some(D05_PARAMETERS)).unwrap();
        let transform_parameters =
            MathGeneratedTransformParameters::from_provider_bytes(F08_PARAMETERS).unwrap();
        let policy_parameters = selected_policy_parameters();
        let (attempt, transaction, source) = source_transaction();
        let f07 = root
            .prepare_f07_source(
                attempt,
                transaction,
                source,
                &varying_audio(),
                [false; FBW_LANES],
                &transient_parameters,
            )
            .unwrap();
        let f08 = f07.complete_f08(&transform_parameters).unwrap();
        let gain_db = f08.complete_gain_db(&policy_parameters).unwrap();

        let f07_output = gain_db.shared_analysis().unwrap().f07_proposal().output();
        let structure_refresh = *f07_output.structure_refresh();
        let short_consumed = *f07_output.short_consumed();
        assert_eq!(structure_refresh[0], [true; FBW_LANES]);
        assert_eq!(short_consumed[0], [0; FBW_LANES]);
        let fullband_power: [[[i16; FBW_BINS]; FBW_LANES]; F07_BLOCKS_PER_FRAME] =
            std::array::from_fn(|block| {
                std::array::from_fn(|lane| {
                    *gain_db.f08_plan().fullband_exponent_inputs()[block * FBW_LANES + lane]
                        .power_raw_exponents_i16()
                })
            });
        let fullband_primary: [[[i16; FBW_BINS]; FBW_LANES]; F07_BLOCKS_PER_FRAME] =
            std::array::from_fn(|block| {
                std::array::from_fn(|lane| {
                    *gain_db.f08_plan().fullband_exponent_inputs()[block * FBW_LANES + lane]
                        .primary_raw_exponents_i16()
                })
            });
        let lfe_power: [[i16; LFE_BINS]; F07_BLOCKS_PER_FRAME] = std::array::from_fn(|block| {
            *gain_db.f08_plan().lfe_exponent_inputs()[block].power_raw_exponents_i16()
        });
        let lfe_primary: [[i16; LFE_BINS]; F07_BLOCKS_PER_FRAME] = std::array::from_fn(|block| {
            *gain_db.f08_plan().lfe_exponent_inputs()[block].primary_raw_exponents_i16()
        });
        let gain_ranges = *gain_db.plan().gain_codes_by_block();

        assert!(
            fullband_power
                .iter()
                .flatten()
                .zip(fullband_primary.iter().flatten())
                .any(|(power, primary)| power != primary),
            "synthetic input must distinguish POWER refresh rows from PRIMARY F10 rows"
        );

        let mut expected_fullband_markers = [[0_i16; FBW_LANES]; F07_BLOCKS_PER_FRAME];
        let mut expected_first_power_refresh_current_after_i16 =
            [[[0_i16; FBW_BINS]; FBW_LANES]; F07_BLOCKS_PER_FRAME];
        for lane in 0..FBW_LANES {
            let mut work = [0_i16; FBW_BINS];
            for block in 0..F07_BLOCKS_PER_FRAME {
                let mut current = fullband_power[block][lane];
                let mut marker = i16::from(structure_refresh[block][lane]);
                refresh_exponent_state(&mut current, &mut work, &mut marker, 1, 0, 0, 128);
                expected_first_power_refresh_current_after_i16[block][lane] = current;
                expected_fullband_markers[block][lane] = marker;
            }
        }

        let mut expected_lfe_markers = [0_i16; F07_BLOCKS_PER_FRAME];
        let mut lfe_work = [0_i16; LFE_BINS];
        for block in 0..F07_BLOCKS_PER_FRAME {
            let mut current = lfe_power[block];
            let mut marker = i16::from(block == 0);
            refresh_exponent_state(&mut current, &mut lfe_work, &mut marker, 0, 0, 0, 7);
            expected_lfe_markers[block] = marker;
        }

        let expected_strategies =
            expected_fullband_strategies(&expected_fullband_markers, &short_consumed);
        let numeric = gain_db.complete_f09_f10(&policy_parameters).unwrap();
        let shared_analysis = numeric.shared_analysis().unwrap();
        assert_eq!(shared_analysis.identity().source_index(), source);
        assert_eq!(shared_analysis.advancement_count(), 1);
        let numeric_plan = numeric.plan().unwrap();
        let actual_plan = numeric_plan.exponents();
        assert_eq!(
            actual_plan.fullband_refresh_marker_after(),
            &expected_fullband_markers
        );
        for (block, expected_block) in expected_first_power_refresh_current_after_i16
            .iter()
            .enumerate()
        {
            for (lane, expected_value) in expected_block.iter().enumerate() {
                assert_eq!(
                    actual_plan
                        .first_power_refresh_current_after_i16(
                            AnalysisBlockIndex::new(u8::try_from(block).unwrap()),
                            FullbandLaneIndex::new(u8::try_from(lane).unwrap()),
                        )
                        .unwrap(),
                    expected_value
                );
            }
        }
        assert!(
            actual_plan
                .first_power_refresh_current_after_i16(
                    AnalysisBlockIndex::new(6),
                    FullbandLaneIndex::new(0)
                )
                .is_none()
        );
        assert!(
            actual_plan
                .first_power_refresh_current_after_i16(
                    AnalysisBlockIndex::new(0),
                    FullbandLaneIndex::new(5)
                )
                .is_none()
        );
        assert_eq!(actual_plan.fullband_strategy(), &expected_strategies);
        assert_eq!(
            actual_plan.lfe_refresh_marker_after(),
            &expected_lfe_markers
        );

        let profile = PolicyProfile::selected();
        for lane in 0..FBW_LANES {
            let mut block = 0;
            while block < F07_BLOCKS_PER_FRAME {
                let strategy = expected_strategies[block][lane];
                assert_ne!(strategy, 0, "selected source must start with an F10 anchor");
                let end = (block + 1..F07_BLOCKS_PER_FRAME)
                    .find(|next| expected_strategies[*next][lane] != 0)
                    .unwrap_or(F07_BLOCKS_PER_FRAME);
                let merged_primary = std::array::from_fn(|bin| {
                    (block..end)
                        .map(|source_block| fullband_primary[source_block][lane][bin])
                        .min()
                        .unwrap()
                });
                let expected_gain = (block..end)
                    .map(|source_block| gain_ranges[source_block][lane])
                    .min()
                    .unwrap();
                let span = match strategy {
                    1 => 1,
                    2 => 2,
                    3 => 4,
                    _ => unreachable!("strategy was generated from the selected contract"),
                };
                let normalized = normalize_fullband_exponents(
                    &merged_primary,
                    span,
                    profile.normalization_option,
                    i16::from(profile.normalization_gate),
                    Some(&policy_parameters),
                )
                .unwrap();
                let decoded = std::array::from_fn(|index| {
                    u8::try_from(normalized.expanded_exponents()[index]).unwrap()
                });
                for (source_block, strategy_row) in
                    expected_strategies.iter().enumerate().take(end).skip(block)
                {
                    let actual = actual_plan
                        .fullband_block(
                            AnalysisBlockIndex::new(u8::try_from(source_block).unwrap()),
                            FullbandLaneIndex::new(u8::try_from(lane).unwrap()),
                        )
                        .unwrap();
                    assert_eq!(actual.decoded_exponents(), &decoded);
                    assert_eq!(actual.gain_range(), u8::try_from(expected_gain).unwrap());
                    assert_eq!(actual.strategy(), strategy_row[lane]);
                }
                block = end;
            }
        }

        let mut block = 0;
        while block < F07_BLOCKS_PER_FRAME {
            let strategy = u8::try_from(expected_lfe_markers[block]).unwrap();
            assert_ne!(strategy, 0, "selected LFE plan must start with an anchor");
            let end = (block + 1..F07_BLOCKS_PER_FRAME)
                .find(|next| expected_lfe_markers[*next] != 0)
                .unwrap_or(F07_BLOCKS_PER_FRAME);
            let merged_primary = std::array::from_fn(|bin| {
                (block..end)
                    .map(|source_block| lfe_primary[source_block][bin])
                    .min()
                    .unwrap()
            });
            let normalized =
                normalize_lfe_exponents(&merged_primary, Some(&policy_parameters)).unwrap();
            let decoded = std::array::from_fn(|index| {
                u8::try_from(normalized.expanded_exponents()[index]).unwrap()
            });
            for (source_block, marker_after) in expected_lfe_markers
                .iter()
                .enumerate()
                .take(end)
                .skip(block)
            {
                let actual = actual_plan
                    .lfe_block(AnalysisBlockIndex::new(u8::try_from(source_block).unwrap()))
                    .unwrap();
                assert_eq!(actual.decoded_exponents(), decoded);
                assert_eq!(actual.strategy(), u8::try_from(*marker_after).unwrap());
            }
            block = end;
        }
    }
}
