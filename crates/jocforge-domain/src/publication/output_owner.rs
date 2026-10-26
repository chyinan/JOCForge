// pattern: Functional Core
//! Noncommitting keyless output-owner proposals for the selected codec source clock.

use crate::{
    integrity::crc16::eac3_crc16,
    timeline::{CodecSourceIndex, JocDispatchIndex, PublishedAuIndex},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const FRAME_BYTES: usize = 4_096;
const FRAME_BYTES_U32: u32 = FRAME_BYTES as u32;
const SUPPRESSED_SOURCES: u64 = 2;
const SOURCE_FRAME_SAMPLES: u64 = 1_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputOwnerError {
    ProviderJson,
    ProviderProfile,
    ProviderFrameSize,
    ProviderSuppressionCount,
    EmptyAttemptId,
    EmptyOutputName,
    OutputNameTooLong,
    EmptyProgramme,
    CountOverflow,
    SourceOrder,
    DispatchOrder,
    AuOrder,
    FrameLength,
    FrameHash,
    FrameCrc,
    StateEncoding,
    OutputOffset,
    OutputLedger,
    FinalizeLedger,
    DurableReceipt,
    FinalizeCounts,
    FinalizeReceipt,
}

impl OutputOwnerError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ProviderJson => "OUTPUT_PROVIDER_JSON",
            Self::ProviderProfile => "OUTPUT_PROVIDER_PROFILE",
            Self::ProviderFrameSize => "OUTPUT_PROVIDER_FRAME_SIZE",
            Self::ProviderSuppressionCount => "OUTPUT_PROVIDER_SUPPRESSION_COUNT",
            Self::EmptyAttemptId => "OUTPUT_ATTEMPT_ID",
            Self::EmptyOutputName => "OUTPUT_NAME",
            Self::OutputNameTooLong => "OUTPUT_NAME_LENGTH",
            Self::EmptyProgramme => "OUTPUT_PROGRAMME_EMPTY",
            Self::CountOverflow => "OUTPUT_COUNT_OVERFLOW",
            Self::SourceOrder => "OUTPUT_SOURCE_CLOCK",
            Self::DispatchOrder => "OUTPUT_DISPATCH_CLOCK",
            Self::AuOrder => "OUTPUT_AU_CLOCK",
            Self::FrameLength => "OUTPUT_FRAME_LENGTH",
            Self::FrameHash => "OUTPUT_FRAME_HASH",
            Self::FrameCrc => "OUTPUT_FRAME_CRC",
            Self::StateEncoding => "OUTPUT_STATE_ENCODING",
            Self::OutputOffset => "OUTPUT_OFFSET",
            Self::OutputLedger => "OUTPUT_LEDGER",
            Self::FinalizeLedger => "OUTPUT_FINALIZE_LEDGER",
            Self::DurableReceipt => "OUTPUT_DURABLE_RECEIPT",
            Self::FinalizeCounts => "OUTPUT_FINALIZE_COUNTS",
            Self::FinalizeReceipt => "OUTPUT_FINALIZE_RECEIPT",
        }
    }
}

impl std::fmt::Display for OutputOwnerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for OutputOwnerError {}

/// Validated F15P profile and runtime extent. The primary 4,249/4,247 example
/// values are not read from the provider; counts derive from this programme's N.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputProvider {
    pub profile_id: String,
    pub provider_sha256: [u8; 32],
    pub frame_bytes: u32,
    pub suppressed_sources: u64,
    pub staging_name_template: String,
    pub intent_name_template: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputConfiguration {
    profile_id: String,
    provider_sha256: [u8; 32],
    attempt_id: String,
    staging_name: String,
    intent_name: String,
    product_name: String,
    source_samples: u64,
    expected_sources: u64,
    expected_published_aus: u64,
    expected_product_bytes: u64,
}

impl OutputConfiguration {
    /// Builds the selected-profile configuration from an already admitted provider.
    /// Provider identity uses the provider's canonical JSON bytes; IO names remain
    /// caller-selected outputs of the approved F15P path policy.
    #[cfg(test)]
    pub fn from_provider_json(
        provider_bytes: &[u8],
        source_samples: u64,
        attempt_id: &str,
        product_name: &str,
    ) -> Result<Self, OutputOwnerError> {
        let provider: Value =
            serde_json::from_slice(provider_bytes).map_err(|_| OutputOwnerError::ProviderJson)?;
        if provider.get("schema").and_then(Value::as_str) != Some("output-owner-provider.v1") {
            return Err(OutputOwnerError::ProviderJson);
        }

        let profile_id = provider
            .get("profile_id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or(OutputOwnerError::ProviderProfile)?
            .to_owned();
        let frame_bytes = provider
            .get("frame_bytes")
            .and_then(Value::as_u64)
            .ok_or(OutputOwnerError::ProviderFrameSize)?;
        if frame_bytes != FRAME_BYTES as u64 {
            return Err(OutputOwnerError::ProviderFrameSize);
        }
        let suppressed_sources = provider
            .get("suppressed_sources")
            .and_then(Value::as_u64)
            .ok_or(OutputOwnerError::ProviderSuppressionCount)?;
        if suppressed_sources != SUPPRESSED_SOURCES {
            return Err(OutputOwnerError::ProviderSuppressionCount);
        }

        let staging_template = provider
            .get("staging_name_template")
            .and_then(Value::as_str)
            .ok_or(OutputOwnerError::ProviderJson)?;
        let intent_template = provider
            .get("intent_name_template")
            .and_then(Value::as_str)
            .ok_or(OutputOwnerError::ProviderJson)?;
        Self::new(
            OutputProvider {
                profile_id,
                provider_sha256: sha256(
                    &serde_json::to_vec(&provider).map_err(|_| OutputOwnerError::ProviderJson)?,
                ),
                frame_bytes: u32::try_from(frame_bytes)
                    .map_err(|_| OutputOwnerError::ProviderFrameSize)?,
                suppressed_sources,
                staging_name_template: staging_template.to_owned(),
                intent_name_template: intent_template.to_owned(),
            },
            attempt_id,
            product_name,
            source_samples,
        )
    }

    pub fn new(
        provider: OutputProvider,
        attempt_id: &str,
        product_name: &str,
        source_samples: u64,
    ) -> Result<Self, OutputOwnerError> {
        let profile_id = provider.profile_id;
        if profile_id.is_empty() {
            return Err(OutputOwnerError::ProviderProfile);
        }
        if provider.frame_bytes != FRAME_BYTES_U32 {
            return Err(OutputOwnerError::ProviderFrameSize);
        }
        if provider.suppressed_sources != SUPPRESSED_SOURCES {
            return Err(OutputOwnerError::ProviderSuppressionCount);
        }
        if attempt_id.is_empty() {
            return Err(OutputOwnerError::EmptyAttemptId);
        }
        let staging_name = provider
            .staging_name_template
            .replace("{attempt_id}", attempt_id);
        let intent_name = provider
            .intent_name_template
            .replace("{attempt_id}", attempt_id);
        if staging_name.is_empty() || intent_name.is_empty() || product_name.is_empty() {
            return Err(OutputOwnerError::EmptyOutputName);
        }
        if source_samples == 0 {
            return Err(OutputOwnerError::EmptyProgramme);
        }
        for value in [
            profile_id.as_str(),
            attempt_id,
            staging_name.as_str(),
            intent_name.as_str(),
            product_name,
        ] {
            if u16::try_from(value.len()).is_err() {
                return Err(OutputOwnerError::OutputNameTooLong);
            }
        }

        let quotient = source_samples / SOURCE_FRAME_SAMPLES;
        let remainder = u64::from(source_samples % SOURCE_FRAME_SAMPLES != 0);
        let published_aus = quotient
            .checked_add(remainder)
            .ok_or(OutputOwnerError::CountOverflow)?;
        let expected_sources = published_aus
            .checked_add(SUPPRESSED_SOURCES)
            .ok_or(OutputOwnerError::CountOverflow)?;
        let expected_product_bytes = published_aus
            .checked_mul(FRAME_BYTES as u64)
            .ok_or(OutputOwnerError::CountOverflow)?;
        if published_aus > u64::from(u32::MAX) {
            return Err(OutputOwnerError::CountOverflow);
        }

        Ok(Self {
            profile_id,
            provider_sha256: provider.provider_sha256,
            attempt_id: attempt_id.to_owned(),
            staging_name: staging_name.clone(),
            intent_name: intent_name.clone(),
            product_name: product_name.to_owned(),
            source_samples,
            expected_sources,
            expected_published_aus: published_aus,
            expected_product_bytes,
        })
    }

    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    pub const fn provider_sha256(&self) -> &[u8; 32] {
        &self.provider_sha256
    }

    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub fn staging_name(&self) -> &str {
        &self.staging_name
    }

    pub fn intent_name(&self) -> &str {
        &self.intent_name
    }

    pub fn product_name(&self) -> &str {
        &self.product_name
    }

    pub const fn source_samples(&self) -> u64 {
        self.source_samples
    }

    pub const fn expected_sources(&self) -> u64 {
        self.expected_sources
    }

    pub const fn expected_published_aus(&self) -> u64 {
        self.expected_published_aus
    }

    pub const fn expected_product_bytes(&self) -> u64 {
        self.expected_product_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputSourceIdentity {
    source_index: CodecSourceIndex,
    dispatch: JocDispatchIndex,
    au_index: Option<PublishedAuIndex>,
}

impl OutputSourceIdentity {
    pub const fn new(
        source_index: CodecSourceIndex,
        dispatch: JocDispatchIndex,
        au_index: Option<PublishedAuIndex>,
    ) -> Self {
        Self {
            source_index,
            dispatch,
            au_index,
        }
    }

    pub const fn source_index(self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn dispatch(self) -> JocDispatchIndex {
        self.dispatch
    }

    pub const fn au_index(self) -> Option<PublishedAuIndex> {
        self.au_index
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LedgerEntry {
    au_index: PublishedAuIndex,
    offset: u64,
    length: u32,
    frame_sha256: [u8; 32],
    crc_residual: u16,
}

impl LedgerEntry {
    fn as_json(&self) -> Value {
        json!({
            "au_index": self.au_index.value(),
            "offset": self.offset,
            "length": self.length,
            "frame_sha256": hex_lower(&self.frame_sha256),
            "crc_residual": self.crc_residual,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OutputState {
    profile_id: String,
    provider_sha256: [u8; 32],
    attempt_id: String,
    staging_name: String,
    generation: u64,
    next_source_index: CodecSourceIndex,
    published_au_count: u64,
    durable_length: u64,
    ledger: Vec<LedgerEntry>,
    finalized: bool,
    product_name: String,
    final_product_sha256: Option<[u8; 32]>,
}

impl OutputState {
    fn from_configuration(configuration: &OutputConfiguration) -> Self {
        Self {
            profile_id: configuration.profile_id.clone(),
            provider_sha256: configuration.provider_sha256,
            attempt_id: configuration.attempt_id.clone(),
            staging_name: configuration.staging_name.clone(),
            generation: 0,
            next_source_index: CodecSourceIndex::new(0),
            published_au_count: 0,
            durable_length: 0,
            ledger: Vec::new(),
            finalized: false,
            product_name: configuration.product_name.clone(),
            final_product_sha256: None,
        }
    }

    fn as_json(&self) -> Value {
        json!({
            "profile_id": self.profile_id,
            "provider_sha256": hex_lower(&self.provider_sha256),
            "attempt_id": self.attempt_id,
            "staging_name": self.staging_name,
            "generation": self.generation,
            "next_source_index": self.next_source_index.value(),
            "published_au_count": self.published_au_count,
            "durable_length": self.durable_length,
            "ledger": self.ledger.iter().map(LedgerEntry::as_json).collect::<Vec<_>>(),
            "finalized": self.finalized,
            "product_name": self.product_name,
            "final_product_sha256": self.final_product_sha256.map(|sha| hex_lower(&sha)),
        })
    }

    fn canonical_bytes(&self) -> Result<Vec<u8>, OutputOwnerError> {
        let mut bytes = b"OutputStateV1\0".to_vec();
        append_text(&mut bytes, &self.profile_id)?;
        append_text(&mut bytes, &self.attempt_id)?;
        append_text(&mut bytes, &self.staging_name)?;
        bytes.extend_from_slice(&self.provider_sha256);
        bytes.extend_from_slice(&self.generation.to_le_bytes());
        bytes.extend_from_slice(&self.next_source_index.value().to_le_bytes());
        bytes.extend_from_slice(&self.published_au_count.to_le_bytes());
        bytes.extend_from_slice(&self.durable_length.to_le_bytes());
        bytes.extend_from_slice(
            &u32::try_from(self.ledger.len())
                .map_err(|_| OutputOwnerError::StateEncoding)?
                .to_le_bytes(),
        );
        for entry in &self.ledger {
            bytes.extend_from_slice(&entry.au_index.value().to_le_bytes());
            bytes.extend_from_slice(&entry.offset.to_le_bytes());
            bytes.extend_from_slice(&entry.length.to_le_bytes());
            bytes.extend_from_slice(&entry.frame_sha256);
            bytes.extend_from_slice(&entry.crc_residual.to_le_bytes());
        }
        bytes.push(u8::from(self.finalized));
        append_text(&mut bytes, &self.product_name)?;
        match self.final_product_sha256 {
            Some(sha256) => {
                bytes.push(1);
                bytes.extend_from_slice(&sha256);
            }
            None => bytes.push(0),
        }
        Ok(bytes)
    }

    fn sha256(&self) -> Result<[u8; 32], OutputOwnerError> {
        Ok(sha256(&self.canonical_bytes()?))
    }

    fn sha256_hex(&self) -> Result<String, OutputOwnerError> {
        Ok(hex_lower(&self.sha256()?))
    }
}

#[must_use]
#[derive(Debug)]
pub struct OutputOwner {
    configuration: OutputConfiguration,
    state: OutputState,
}

impl OutputOwner {
    pub fn cold(configuration: OutputConfiguration) -> Self {
        let state = OutputState::from_configuration(&configuration);
        Self {
            configuration,
            state,
        }
    }

    pub fn prepare_source(
        &self,
        identity: OutputSourceIdentity,
        frame: &[u8],
        declared_frame_sha256: &[u8; 32],
    ) -> Result<OutputSourceProposal, OutputOwnerError> {
        let source = identity.source_index.value();
        if self.state.finalized
            || source != self.state.next_source_index.value()
            || source >= self.configuration.expected_sources
        {
            return Err(OutputOwnerError::SourceOrder);
        }
        let expected_dispatch = source
            .checked_add(1)
            .ok_or(OutputOwnerError::DispatchOrder)?;
        if identity.dispatch.value() != expected_dispatch {
            return Err(OutputOwnerError::DispatchOrder);
        }
        let expected_au = source
            .checked_sub(SUPPRESSED_SOURCES)
            .map(PublishedAuIndex::new);
        if identity.au_index != expected_au {
            return Err(OutputOwnerError::AuOrder);
        }
        if frame.len() != FRAME_BYTES {
            return Err(OutputOwnerError::FrameLength);
        }
        let frame_sha256 = sha256(frame);
        if &frame_sha256 != declared_frame_sha256 {
            return Err(OutputOwnerError::FrameHash);
        }
        let crc_residual = eac3_crc16(&frame[2..]);
        if crc_residual != 0 {
            return Err(OutputOwnerError::FrameCrc);
        }

        let before_state_sha256 = self.state.sha256()?;
        let mut after_state = self.state.clone();
        after_state.generation = after_state
            .generation
            .checked_add(1)
            .ok_or(OutputOwnerError::CountOverflow)?;
        after_state.next_source_index = CodecSourceIndex::new(
            source
                .checked_add(1)
                .ok_or(OutputOwnerError::CountOverflow)?,
        );

        if source < SUPPRESSED_SOURCES {
            let after_state_sha256 = after_state.sha256()?;
            return Ok(OutputSourceProposal::Suppressed(
                OutputSuppressedSourceProposal {
                    receipt: OutputSuppressedSourceReceipt {
                        source_index: identity.source_index,
                        dispatch: identity.dispatch,
                        before_sha256: before_state_sha256,
                        after_sha256: after_state_sha256,
                    },
                    after_owner: Self {
                        configuration: self.configuration.clone(),
                        state: after_state,
                    },
                },
            ));
        }

        let au_index = identity.au_index.ok_or(OutputOwnerError::AuOrder)?;
        let offset = au_index
            .value()
            .checked_mul(FRAME_BYTES as u64)
            .ok_or(OutputOwnerError::OutputOffset)?;
        if offset != self.state.durable_length {
            return Err(OutputOwnerError::OutputOffset);
        }
        let entry = LedgerEntry {
            au_index,
            offset,
            length: FRAME_BYTES_U32,
            frame_sha256,
            crc_residual,
        };
        after_state.published_au_count = after_state
            .published_au_count
            .checked_add(1)
            .ok_or(OutputOwnerError::CountOverflow)?;
        after_state.durable_length = after_state
            .durable_length
            .checked_add(FRAME_BYTES as u64)
            .ok_or(OutputOwnerError::CountOverflow)?;
        after_state.ledger.push(entry.clone());

        let proposed_after_sha256 = after_state.sha256()?;
        let frame_bytes: [u8; FRAME_BYTES] = frame
            .try_into()
            .map_err(|_| OutputOwnerError::FrameLength)?;
        let proposal = OutputAppendProposal {
            attempt_id: self.state.attempt_id.clone(),
            staging_name: self.state.staging_name.clone(),
            identity,
            expected_offset: offset,
            frame_bytes: Box::new(frame_bytes),
            frame_sha256,
            before_state_sha256,
            proposed_after_sha256,
            ledger_entry: entry,
            after_owner: Self {
                configuration: self.configuration.clone(),
                state: after_state,
            },
        };
        Ok(OutputSourceProposal::Append(proposal))
    }

    pub fn prepare_finalize(&self) -> Result<OutputFinalizeProposal, OutputOwnerError> {
        if self.state.finalized
            || self.state.next_source_index.value() != self.configuration.expected_sources
            || self.state.published_au_count != self.configuration.expected_published_aus
            || u64::try_from(self.state.ledger.len())
                .map_err(|_| OutputOwnerError::FinalizeCounts)?
                != self.configuration.expected_published_aus
            || self.state.durable_length != self.configuration.expected_product_bytes
        {
            return Err(OutputOwnerError::FinalizeCounts);
        }
        for (index, entry) in self.state.ledger.iter().enumerate() {
            let au_index = u64::try_from(index).map_err(|_| OutputOwnerError::FinalizeLedger)?;
            let expected_offset = au_index
                .checked_mul(FRAME_BYTES as u64)
                .ok_or(OutputOwnerError::FinalizeLedger)?;
            if entry.au_index.value() != au_index
                || entry.offset != expected_offset
                || entry.length != FRAME_BYTES_U32
                || entry.crc_residual != 0
            {
                return Err(OutputOwnerError::FinalizeLedger);
            }
        }
        let ledger_sha256 = sha256(&ledger_preimage(&self.state.ledger)?);
        Ok(OutputFinalizeProposal {
            attempt_id: self.state.attempt_id.clone(),
            staging_name: self.state.staging_name.clone(),
            product_name: self.state.product_name.clone(),
            state_sha256: self.state.sha256()?,
            expected_length: self.state.durable_length,
            ledger_sha256,
            before_state_sha256: self.state.sha256()?,
            configuration: self.configuration.clone(),
            before_state: self.state.clone(),
        })
    }

    pub fn state_json(&self) -> Value {
        self.state.as_json()
    }

    pub fn state_sha256(&self) -> Result<String, OutputOwnerError> {
        self.state.sha256_hex()
    }

    pub fn configuration(&self) -> &OutputConfiguration {
        &self.configuration
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputSuppressedSourceReceipt {
    source_index: CodecSourceIndex,
    dispatch: JocDispatchIndex,
    before_sha256: [u8; 32],
    after_sha256: [u8; 32],
}

impl OutputSuppressedSourceReceipt {
    pub const fn source_index(self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn dispatch(self) -> JocDispatchIndex {
        self.dispatch
    }

    pub const fn before_sha256(&self) -> &[u8; 32] {
        &self.before_sha256
    }

    pub const fn after_sha256(&self) -> &[u8; 32] {
        &self.after_sha256
    }

    pub fn as_json(self) -> Value {
        json!({
            "schema": "output-suppressed-source-receipt.v1",
            "owner_id": "output",
            "source_index": self.source_index.value(),
            "dispatch": self.dispatch.value(),
            "au_index": null,
            "before_sha256": hex_lower(&self.before_sha256),
            "after_sha256": hex_lower(&self.after_sha256),
            "append_required": false,
            "commit_count": 1,
        })
    }
}

#[must_use]
#[derive(Debug)]
pub struct OutputSuppressedSourceProposal {
    receipt: OutputSuppressedSourceReceipt,
    after_owner: OutputOwner,
}

impl OutputSuppressedSourceProposal {
    pub const fn receipt(&self) -> OutputSuppressedSourceReceipt {
        self.receipt
    }

    pub fn into_after(self) -> OutputSourceAfterProposal {
        let receipt = OutputSourceCommitReceipt::Suppressed(self.receipt);
        OutputSourceAfterProposal {
            receipt,
            durable_append_receipt: None,
            after_owner: self.after_owner,
        }
    }
}

#[must_use]
#[derive(Debug)]
pub enum OutputSourceProposal {
    Suppressed(OutputSuppressedSourceProposal),
    Append(OutputAppendProposal),
}

#[must_use]
#[derive(Debug)]
pub struct OutputAppendProposal {
    attempt_id: String,
    staging_name: String,
    identity: OutputSourceIdentity,
    expected_offset: u64,
    frame_bytes: Box<[u8; FRAME_BYTES]>,
    frame_sha256: [u8; 32],
    before_state_sha256: [u8; 32],
    proposed_after_sha256: [u8; 32],
    ledger_entry: LedgerEntry,
    after_owner: OutputOwner,
}

impl OutputAppendProposal {
    pub fn as_json(&self) -> Value {
        json!({
            "schema": "output-append-proposal.v1",
            "attempt_id": self.attempt_id,
            "staging_name": self.staging_name,
            "source_index": self.identity.source_index.value(),
            "dispatch": self.identity.dispatch.value(),
            "au_index": self.identity.au_index.map(PublishedAuIndex::value),
            "expected_offset": self.expected_offset,
            "frame_bytes": FRAME_BYTES,
            "frame_sha256": hex_lower(&self.frame_sha256),
            "frame_hex": hex_lower(self.frame_bytes.as_slice()),
            "before_state_sha256": hex_lower(&self.before_state_sha256),
            "proposed_after_sha256": hex_lower(&self.proposed_after_sha256),
            "ledger_entry": self.ledger_entry.as_json(),
        })
    }

    pub fn frame_bytes(&self) -> &[u8; FRAME_BYTES] {
        &self.frame_bytes
    }

    pub const fn identity(&self) -> OutputSourceIdentity {
        self.identity
    }

    pub const fn expected_offset(&self) -> u64 {
        self.expected_offset
    }

    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub fn staging_name(&self) -> &str {
        &self.staging_name
    }

    pub const fn frame_sha256(&self) -> &[u8; 32] {
        &self.frame_sha256
    }

    pub const fn before_state_sha256(&self) -> &[u8; 32] {
        &self.before_state_sha256
    }

    pub const fn proposed_after_sha256(&self) -> &[u8; 32] {
        &self.proposed_after_sha256
    }

    pub fn proposal_sha256(&self) -> Result<[u8; 32], OutputOwnerError> {
        let bytes =
            serde_json::to_vec(&self.as_json()).map_err(|_| OutputOwnerError::StateEncoding)?;
        Ok(sha256(&bytes))
    }

    /// Constructs the typed receipt only after the IO shell completed all writes,
    /// readback checks and file fsync. This does not perform filesystem operations.
    pub fn record_durable_io_success(
        &self,
        write_calls: Vec<u64>,
        readback_sha256: [u8; 32],
        crc_residual: u16,
    ) -> Result<OutputDurableAppendReceipt, OutputOwnerError> {
        let written = write_calls.iter().try_fold(0_u64, |sum, count| {
            if *count == 0 {
                return None;
            }
            sum.checked_add(*count)
        });
        if written != Some(FRAME_BYTES as u64)
            || readback_sha256 != self.frame_sha256
            || crc_residual != 0
        {
            return Err(OutputOwnerError::DurableReceipt);
        }
        Ok(OutputDurableAppendReceipt {
            proposal_sha256: self.proposal_sha256()?,
            attempt_id: self.attempt_id.clone(),
            au_index: self.identity.au_index.ok_or(OutputOwnerError::AuOrder)?,
            offset: self.expected_offset,
            bytes_written: FRAME_BYTES_U32,
            write_calls,
            readback_sha256,
            crc_residual,
            file_fsync: true,
            durable: true,
        })
    }

    pub fn complete(
        self,
        receipt: OutputDurableAppendReceipt,
    ) -> Result<OutputSourceAfterProposal, OutputOwnerError> {
        if receipt.attempt_id != self.attempt_id
            || receipt.au_index != self.identity.au_index.ok_or(OutputOwnerError::AuOrder)?
            || receipt.offset != self.expected_offset
            || receipt.bytes_written != FRAME_BYTES_U32
            || receipt.readback_sha256 != self.frame_sha256
            || receipt.crc_residual != 0
            || !receipt.file_fsync
            || !receipt.durable
            || receipt.proposal_sha256 != self.proposal_sha256()?
            || self.after_owner.state.sha256()? != self.proposed_after_sha256
        {
            return Err(OutputOwnerError::DurableReceipt);
        }
        let after_owner = self.after_owner;
        let after_sha256 = after_owner.state.sha256()?;
        let source_receipt = OutputOwnerCommitReceipt {
            before_sha256: self.before_state_sha256,
            after_sha256,
            au_index: receipt.au_index,
        };
        Ok(OutputSourceAfterProposal {
            receipt: OutputSourceCommitReceipt::Published(source_receipt),
            durable_append_receipt: Some(receipt),
            after_owner,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputDurableAppendReceipt {
    proposal_sha256: [u8; 32],
    attempt_id: String,
    au_index: PublishedAuIndex,
    offset: u64,
    bytes_written: u32,
    write_calls: Vec<u64>,
    readback_sha256: [u8; 32],
    crc_residual: u16,
    file_fsync: bool,
    durable: bool,
}

impl OutputDurableAppendReceipt {
    pub fn as_json(&self) -> Value {
        json!({
            "schema": "durable-append-receipt.v1",
            "proposal_sha256": hex_lower(&self.proposal_sha256),
            "attempt_id": self.attempt_id,
            "au_index": self.au_index.value(),
            "offset": self.offset,
            "bytes_written": self.bytes_written,
            "write_calls": self.write_calls,
            "readback_sha256": hex_lower(&self.readback_sha256),
            "crc_residual": self.crc_residual,
            "file_fsync": self.file_fsync,
            "durable": self.durable,
        })
    }

    pub const fn au_index(&self) -> PublishedAuIndex {
        self.au_index
    }

    pub const fn offset(&self) -> u64 {
        self.offset
    }

    pub const fn bytes_written(&self) -> u32 {
        self.bytes_written
    }

    pub fn write_calls(&self) -> &[u64] {
        &self.write_calls
    }

    pub const fn readback_sha256(&self) -> &[u8; 32] {
        &self.readback_sha256
    }

    pub const fn crc_residual(&self) -> u16 {
        self.crc_residual
    }

    pub const fn file_fsync(&self) -> bool {
        self.file_fsync
    }

    pub const fn durable(&self) -> bool {
        self.durable
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputOwnerCommitReceipt {
    before_sha256: [u8; 32],
    after_sha256: [u8; 32],
    au_index: PublishedAuIndex,
}

impl OutputOwnerCommitReceipt {
    pub const fn before_sha256(&self) -> &[u8; 32] {
        &self.before_sha256
    }

    pub const fn after_sha256(&self) -> &[u8; 32] {
        &self.after_sha256
    }

    pub const fn au_index(&self) -> PublishedAuIndex {
        self.au_index
    }

    pub fn as_json(self) -> Value {
        json!({
            "schema": "output-owner-commit-receipt.v1",
            "owner_id": "output",
            "before_sha256": hex_lower(&self.before_sha256),
            "after_sha256": hex_lower(&self.after_sha256),
            "au_index": self.au_index.value(),
            "commit_count": 1,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputSourceCommitReceipt {
    Suppressed(OutputSuppressedSourceReceipt),
    Published(OutputOwnerCommitReceipt),
}

impl OutputSourceCommitReceipt {
    pub fn as_json(self) -> Value {
        match self {
            Self::Suppressed(receipt) => receipt.as_json(),
            Self::Published(receipt) => receipt.as_json(),
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct OutputSourceAfterProposal {
    receipt: OutputSourceCommitReceipt,
    durable_append_receipt: Option<OutputDurableAppendReceipt>,
    after_owner: OutputOwner,
}

impl OutputSourceAfterProposal {
    pub const fn receipt(&self) -> OutputSourceCommitReceipt {
        self.receipt
    }

    pub const fn durable_append_receipt(&self) -> Option<&OutputDurableAppendReceipt> {
        self.durable_append_receipt.as_ref()
    }

    pub fn after_owner(&self) -> &OutputOwner {
        &self.after_owner
    }

    pub fn into_after_owner(self) -> OutputOwner {
        self.after_owner
    }
}

#[must_use]
#[derive(Debug)]
pub struct OutputFinalizeProposal {
    attempt_id: String,
    staging_name: String,
    product_name: String,
    state_sha256: [u8; 32],
    expected_length: u64,
    ledger_sha256: [u8; 32],
    before_state_sha256: [u8; 32],
    before_state: OutputState,
    configuration: OutputConfiguration,
}

impl OutputFinalizeProposal {
    pub fn as_json(&self) -> Value {
        json!({
            "schema": "output-finalize-proposal.v1",
            "attempt_id": self.attempt_id,
            "staging_name": self.staging_name,
            "product_name": self.product_name,
            "state_sha256": hex_lower(&self.state_sha256),
            "expected_length": self.expected_length,
            "ledger_sha256": hex_lower(&self.ledger_sha256),
        })
    }

    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub fn staging_name(&self) -> &str {
        &self.staging_name
    }

    pub fn product_name(&self) -> &str {
        &self.product_name
    }

    pub const fn expected_length(&self) -> u64 {
        self.expected_length
    }

    pub const fn state_sha256(&self) -> &[u8; 32] {
        &self.state_sha256
    }

    pub const fn ledger_sha256(&self) -> &[u8; 32] {
        &self.ledger_sha256
    }

    pub fn proposal_sha256(&self) -> Result<[u8; 32], OutputOwnerError> {
        let bytes =
            serde_json::to_vec(&self.as_json()).map_err(|_| OutputOwnerError::StateEncoding)?;
        Ok(sha256(&bytes))
    }

    /// Constructs the terminal receipt only after rename and parent-directory fsync.
    pub fn record_durable_io_success(
        &self,
        product_sha256: [u8; 32],
        product_length: u64,
    ) -> Result<OutputFinalExposureReceipt, OutputOwnerError> {
        if product_length != self.expected_length {
            return Err(OutputOwnerError::FinalizeReceipt);
        }
        Ok(OutputFinalExposureReceipt {
            proposal_sha256: self.proposal_sha256()?,
            attempt_id: self.attempt_id.clone(),
            product_sha256,
            product_length,
            atomic_replace: true,
            parent_fsync: true,
        })
    }

    pub fn complete(
        self,
        receipt: OutputFinalExposureReceipt,
    ) -> Result<OutputFinalizedProposal, OutputOwnerError> {
        if receipt.proposal_sha256 != self.proposal_sha256()?
            || receipt.attempt_id != self.attempt_id
            || receipt.product_length != self.expected_length
            || !receipt.atomic_replace
            || !receipt.parent_fsync
        {
            return Err(OutputOwnerError::FinalizeReceipt);
        }
        let mut after_state = self.before_state;
        after_state.finalized = true;
        after_state.final_product_sha256 = Some(receipt.product_sha256);
        Ok(OutputFinalizedProposal {
            receipt,
            after_owner: OutputOwner {
                configuration: self.configuration,
                state: after_state,
            },
            before_state_sha256: self.before_state_sha256,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputFinalExposureReceipt {
    proposal_sha256: [u8; 32],
    attempt_id: String,
    product_sha256: [u8; 32],
    product_length: u64,
    atomic_replace: bool,
    parent_fsync: bool,
}

impl OutputFinalExposureReceipt {
    pub fn as_json(&self) -> Value {
        json!({
            "schema": "final-exposure-receipt.v1",
            "proposal_sha256": hex_lower(&self.proposal_sha256),
            "attempt_id": self.attempt_id,
            "product_sha256": hex_lower(&self.product_sha256),
            "product_length": self.product_length,
            "atomic_replace": self.atomic_replace,
            "parent_fsync": self.parent_fsync,
        })
    }

    pub const fn product_sha256(&self) -> &[u8; 32] {
        &self.product_sha256
    }

    pub const fn product_length(&self) -> u64 {
        self.product_length
    }
}

#[must_use]
#[derive(Debug)]
pub struct OutputFinalizedProposal {
    receipt: OutputFinalExposureReceipt,
    after_owner: OutputOwner,
    before_state_sha256: [u8; 32],
}

impl OutputFinalizedProposal {
    pub const fn receipt(&self) -> &OutputFinalExposureReceipt {
        &self.receipt
    }

    pub const fn before_state_sha256(&self) -> &[u8; 32] {
        &self.before_state_sha256
    }

    pub fn after_owner(&self) -> &OutputOwner {
        &self.after_owner
    }

    pub fn into_after_owner(self) -> OutputOwner {
        self.after_owner
    }
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn hex_lower(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn append_text(bytes: &mut Vec<u8>, value: &str) -> Result<(), OutputOwnerError> {
    let encoded = value.as_bytes();
    bytes.extend_from_slice(
        &u16::try_from(encoded.len())
            .map_err(|_| OutputOwnerError::OutputNameTooLong)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(encoded);
    Ok(())
}

fn ledger_preimage(ledger: &[LedgerEntry]) -> Result<Vec<u8>, OutputOwnerError> {
    let mut bytes = b"OutputDurableLedgerV1\0".to_vec();
    bytes.extend_from_slice(
        &u32::try_from(ledger.len())
            .map_err(|_| OutputOwnerError::StateEncoding)?
            .to_le_bytes(),
    );
    for entry in ledger {
        bytes.extend_from_slice(&entry.au_index.value().to_le_bytes());
        bytes.extend_from_slice(&entry.offset.to_le_bytes());
        bytes.extend_from_slice(&entry.length.to_le_bytes());
        bytes.extend_from_slice(&entry.frame_sha256);
        bytes.extend_from_slice(&entry.crc_residual.to_le_bytes());
    }
    Ok(bytes)
}
