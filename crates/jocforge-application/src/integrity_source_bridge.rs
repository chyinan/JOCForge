// pattern: Functional Core
//! Protects F13's dynamic zero-slot EMDF container before the F12 source-frame writer.
//!
//! This bridge owns no frame bytes, CRC16, root installation or filesystem operation. The
//! admitted F15 source provider supplies the public checksum profile; F12 remains the only
//! writer for carrier placement and the complete E-AC-3 frame.

use jocforge_domain::{
    codec::{
        selected_frame_trial::EmdfFrameInput, source_frame_writer::SelectedSourceFrameCandidate,
    },
    integrity::keyless_checksum::{
        EmdfProtectionBitRange, EmdfProtectionError, KeylessIntegrityProvider,
        ProtectedEmdfContainer, protect_emdf_container,
    },
    integrity::programme_owner::{
        F12CompletedFrameInput, F14kPayloadInput, IntegrityF12SourceInput, IntegrityIdentity,
    },
    joc::emdf::{
        metadata_owner::{MetadataOwnerError, MetadataSourceProposal},
        protected_container::{CarrierBlock, CurrentEmdfCarriageMode},
    },
    timeline::{
        AdmSampleCount, ClockError, CodecSourceIndex, JocDispatchIndex, PublicationDisposition,
        PublishedAuIndex, programme_counts, publication_for_source, source_for_callback,
    },
    transaction::{AttemptId, CallbackTransactionId, ProgrammeId},
};

pub use jocforge_domain::integrity::programme_owner::{
    IntegrityError, IntegrityF12SourceReceipts, IntegrityGtcF12SourceProposal,
    IntegrityOwnerReceiptProposal, IntegrityProgrammeOwner,
};

use crate::{PreparedJobInput, output_publication::AdmittedProductionPlan};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrityColdRootError {
    PlanAdmission,
    SourceSchedule(ClockError),
    Owner(IntegrityError),
}

/// Builds the cold Integrity root from the provider already admitted by the selected F15 plan.
///
/// This keeps the production CLI from reopening a provider file or constructing a parallel
/// provider value. The expected source count comes from the same prepared input schedule.
pub fn cold_integrity_owner_for_plan(
    prepared: &PreparedJobInput,
    admitted_plan: &AdmittedProductionPlan,
    attempt: AttemptId,
) -> Result<IntegrityProgrammeOwner, IntegrityColdRootError> {
    if !admitted_plan.is_bound_to(prepared) {
        return Err(IntegrityColdRootError::PlanAdmission);
    }
    let sample_count = AdmSampleCount::new(prepared.configuration().source_samples())
        .map_err(IntegrityColdRootError::SourceSchedule)?;
    let expected_source_count = programme_counts(sample_count)
        .map_err(IntegrityColdRootError::SourceSchedule)?
        .as_tuple()
        .3;
    IntegrityProgrammeOwner::cold_from_provider_for_rate(
        prepared.programme_id().as_str(),
        attempt.value(),
        expected_source_count,
        admitted_plan.source_providers().keyless_integrity(),
        prepared.configuration().selected_profile().codec_rate(),
    )
    .map_err(IntegrityColdRootError::Owner)
}

/// F14K outcome for one source, prior to F12's frame write and CRC16.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum F14kPayloadOutcome {
    /// Source 0/1 has no published F13 payload. F12 still writes its complete frame.
    AbsentPublishedPayloadV1,
    /// F14K protected a published F13 container using its dynamic terminal bit ranges.
    ProtectedEmdf {
        container_bytes: usize,
        primary_crc32: u32,
        secondary_crc8: u8,
        primary_range: EmdfProtectionBitRange,
        secondary_range: EmdfProtectionBitRange,
    },
}

/// Noncommitting F14K source lineage and payload result passed toward F12/F15.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct F14kSourceCandidate {
    source_index: CodecSourceIndex,
    dispatch: JocDispatchIndex,
    au_index: Option<PublishedAuIndex>,
    outcome: F14kPayloadOutcome,
    protected_container: Option<ProtectedEmdfContainer>,
}

impl F14kSourceCandidate {
    pub const fn source_index(&self) -> CodecSourceIndex {
        self.source_index
    }

    pub const fn dispatch(&self) -> JocDispatchIndex {
        self.dispatch
    }

    pub const fn au_index(&self) -> Option<PublishedAuIndex> {
        self.au_index
    }

    pub const fn outcome(&self) -> F14kPayloadOutcome {
        self.outcome
    }

    pub const fn protected_container(&self) -> Option<&ProtectedEmdfContainer> {
        self.protected_container.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum F14kSourceBridgeError {
    SourceClock,
    SourceLineage,
    F13PayloadShape,
    TerminalLayoutMissing,
    CarrierBlock,
    Protection(EmdfProtectionError),
    MetadataOwner(MetadataOwnerError),
    IntegrityOwner(IntegrityError),
}

/// Applies F14K to one exact F13 proposal and binds the protected parse for the F12 adapter.
///
/// The caller obtains `provider` from `admitted_plan.source_providers().keyless_integrity()`.
/// The returned candidate carries source, dispatch, and optional AU domains for F15 admission.
/// It creates no `IntegrityProgrammeOwner` after-state: that owner's current combined stage
/// also writes carrier bytes and CRC16, which would duplicate F12's wire ownership.
pub fn protect_metadata_source_for_frame(
    metadata: &mut MetadataSourceProposal,
    provider: &KeylessIntegrityProvider,
) -> Result<F14kSourceCandidate, F14kSourceBridgeError> {
    let receipt = metadata.receipt();
    let source_index = receipt.source_index();
    let expected_dispatch = source_index
        .dispatch()
        .map_err(|_| F14kSourceBridgeError::SourceClock)?;
    let expected_au_index = match publication_for_source(source_index)
        .map_err(|_| F14kSourceBridgeError::SourceClock)?
    {
        PublicationDisposition::Suppressed { .. } => None,
        PublicationDisposition::Published(consumption) => Some(consumption.au),
    };
    if receipt.dispatch() != expected_dispatch
        || receipt.au_index() != expected_au_index
        || source_for_callback(receipt.transaction_id().callback_index()) != Some(source_index)
        || metadata.consumer_transaction_id() != receipt.consumer_transaction_id()
    {
        return Err(F14kSourceBridgeError::SourceLineage);
    }

    if source_index.value() < 2 {
        return absent_payload_candidate(
            source_index,
            expected_dispatch,
            expected_au_index,
            metadata.emdf().is_some(),
            metadata.terminal_layout().is_some(),
            metadata.parsed_container().is_some(),
        );
    }

    if expected_au_index.is_none() || metadata.parsed_container().is_some() {
        return Err(F14kSourceBridgeError::F13PayloadShape);
    }
    let zero_slot_container = metadata
        .emdf()
        .ok_or(F14kSourceBridgeError::F13PayloadShape)?;
    let terminal_layout = metadata
        .terminal_layout()
        .ok_or(F14kSourceBridgeError::TerminalLayoutMissing)?;
    let candidate = published_payload_candidate(
        source_index,
        expected_dispatch,
        expected_au_index,
        provider,
        zero_slot_container,
        terminal_layout,
    )?;
    let carrier_block = CarrierBlock::new(2).map_err(|_| F14kSourceBridgeError::CarrierBlock)?;
    metadata
        .bind_protected_container(
            candidate
                .protected_container()
                .ok_or(F14kSourceBridgeError::F13PayloadShape)?
                .bytes(),
            CurrentEmdfCarriageMode::DeployedCompat,
            carrier_block,
        )
        .map_err(F14kSourceBridgeError::MetadataOwner)?;
    Ok(candidate)
}

fn absent_payload_candidate(
    source_index: CodecSourceIndex,
    dispatch: JocDispatchIndex,
    au_index: Option<PublishedAuIndex>,
    has_emdf: bool,
    has_terminal_layout: bool,
    has_parsed_container: bool,
) -> Result<F14kSourceCandidate, F14kSourceBridgeError> {
    if source_index.value() >= 2
        || au_index.is_some()
        || has_emdf
        || has_terminal_layout
        || has_parsed_container
    {
        return Err(F14kSourceBridgeError::F13PayloadShape);
    }
    Ok(F14kSourceCandidate {
        source_index,
        dispatch,
        au_index: None,
        outcome: F14kPayloadOutcome::AbsentPublishedPayloadV1,
        protected_container: None,
    })
}

pub(crate) fn published_payload_candidate(
    source_index: CodecSourceIndex,
    dispatch: JocDispatchIndex,
    au_index: Option<PublishedAuIndex>,
    provider: &KeylessIntegrityProvider,
    zero_slot_container: &[u8],
    terminal_layout: &serde_json::Value,
) -> Result<F14kSourceCandidate, F14kSourceBridgeError> {
    if source_index.value() < 2 || au_index.is_none() {
        return Err(F14kSourceBridgeError::F13PayloadShape);
    }
    let protected = protect_emdf_container(provider, zero_slot_container, terminal_layout)
        .map_err(F14kSourceBridgeError::Protection)?;
    Ok(F14kSourceCandidate {
        source_index,
        dispatch,
        au_index,
        outcome: F14kPayloadOutcome::ProtectedEmdf {
            container_bytes: protected.bytes().len(),
            primary_crc32: protected.primary_crc32(),
            secondary_crc8: protected.secondary_crc8(),
            primary_range: protected.primary_range(),
            secondary_range: protected.secondary_range(),
        },
        protected_container: Some(protected),
    })
}

/// Creates the noncommitting F14 owner next-root and typed source/dispatch/AU receipts after F12.
///
/// The source-frame candidate is the only source of completed frame bytes and the dynamic carrier
/// cursor. This adapter cross-checks its typed payload and source key against F13/F14 before
/// asking the Integrity owner to validate CRC16 and stage its after-root.
pub fn stage_integrity_after_f12(
    integrity_owner: &IntegrityProgrammeOwner,
    metadata: &MetadataSourceProposal,
    integrity: &F14kSourceCandidate,
    frame: &SelectedSourceFrameCandidate<'_>,
    provider: &KeylessIntegrityProvider,
    programme_id: &ProgrammeId,
    attempt: AttemptId,
    transaction_id: &CallbackTransactionId,
) -> Result<IntegrityGtcF12SourceProposal, F14kSourceBridgeError> {
    let metadata_receipt = metadata.receipt();
    let source_index = integrity.source_index();
    if metadata_receipt.source_index() != source_index
        || metadata_receipt.dispatch() != integrity.dispatch()
        || metadata_receipt.au_index() != integrity.au_index()
        || frame.frame().source_index() != source_index
        || metadata.consumer_transaction_id() != Some(transaction_id)
        || metadata_receipt.programme_id().as_str() != programme_id.as_str()
        || metadata_receipt.attempt().value() != attempt.value()
    {
        return Err(F14kSourceBridgeError::SourceLineage);
    }

    let payload = match (integrity.outcome(), integrity.protected_container()) {
        (F14kPayloadOutcome::AbsentPublishedPayloadV1, None)
            if matches!(frame.emdf_input(), EmdfFrameInput::AbsentPublishedPayloadV1) =>
        {
            F14kPayloadInput::AbsentPublishedPayloadV1
        }
        (F14kPayloadOutcome::ProtectedEmdf { .. }, Some(protected))
            if matches!(frame.emdf_input(), EmdfFrameInput::Published(_))
                && metadata
                    .parsed_container()
                    .is_some_and(|parsed| parsed.bytes() == protected.bytes()) =>
        {
            F14kPayloadInput::ProtectedEmdf {
                zero_slot_container: metadata
                    .emdf()
                    .ok_or(F14kSourceBridgeError::F13PayloadShape)?,
                protected_container: protected,
            }
        }
        _ => return Err(F14kSourceBridgeError::F13PayloadShape),
    };

    let identity = IntegrityIdentity {
        programme_id: programme_id.as_str().to_owned(),
        attempt: attempt.value(),
        transaction_id: transaction_id.as_str().to_owned(),
        source_index,
        dispatch: integrity.dispatch(),
        au_index: integrity.au_index(),
    };
    integrity_owner
        .stage_f12_source_for_callback(
            IntegrityF12SourceInput {
                identity,
                payload,
                completed_frame: F12CompletedFrameInput {
                    frame: frame.frame().frame_bytes(),
                    emdf_container_start: frame.frame().emdf_container_start(),
                },
            },
            provider,
            programme_id,
            attempt,
            transaction_id,
            metadata_receipt.transaction_id(),
        )
        .map_err(F14kSourceBridgeError::IntegrityOwner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jocforge_domain::integrity::keyless_checksum::validate_keyless_provider;

    fn provider() -> KeylessIntegrityProvider {
        validate_keyless_provider(include_bytes!(
            "../../jocforge-domain/tests/fixtures/integrity_owner/integrity-public-provider.json"
        ))
        .unwrap()
    }

    fn zero_slot_container() -> [u8; 11] {
        [0x58, 0x38, 0x00, 0x07, 0x04, 0x80, 0, 0, 0, 0, 0]
    }

    fn terminal_layout() -> serde_json::Value {
        serde_json::json!({
            "bit_order": "MSB-first",
            "terminal_id": { "value": 0, "start_bit": 32, "width_bits": 5 },
            "primary_length_code": { "value": 2, "start_bit": 37, "width_bits": 2, "byte_count": 4 },
            "secondary_length_code": { "value": 1, "start_bit": 39, "width_bits": 2, "byte_count": 1 },
            "primary_protection_range_bits": [41, 73],
            "secondary_protection_range_bits": [73, 81],
            "terminal_padding_range_bits": [81, 88],
            "terminal_padding_bits": 7,
            "wire_bytes": 11,
            "body_bytes": 7
        })
    }

    #[test]
    fn published_candidate_carries_f14k_checksums_and_dynamic_ranges() {
        let candidate = published_payload_candidate(
            CodecSourceIndex::new(2),
            JocDispatchIndex::new(3),
            Some(PublishedAuIndex::new(0)),
            &provider(),
            &zero_slot_container(),
            &terminal_layout(),
        )
        .unwrap();
        assert_eq!(candidate.source_index().value(), 2);
        assert_eq!(candidate.dispatch().value(), 3);
        assert_eq!(candidate.au_index(), Some(PublishedAuIndex::new(0)));
        assert!(matches!(
            candidate.outcome(),
            F14kPayloadOutcome::ProtectedEmdf {
                primary_range,
                secondary_range,
                ..
            } if primary_range.start_bit() == 41 && secondary_range.start_bit() == 73
        ));
    }

    #[test]
    fn published_candidate_rejects_wrong_dynamic_range_and_missing_au() {
        let mut invalid_layout = terminal_layout();
        invalid_layout["primary_protection_range_bits"] = serde_json::json!([57, 89]);
        invalid_layout["secondary_protection_range_bits"] = serde_json::json!([89, 97]);
        assert_eq!(
            published_payload_candidate(
                CodecSourceIndex::new(2),
                JocDispatchIndex::new(3),
                Some(PublishedAuIndex::new(0)),
                &provider(),
                &zero_slot_container(),
                &invalid_layout,
            ),
            Err(F14kSourceBridgeError::Protection(
                EmdfProtectionError::SlotRange
            ))
        );
        assert_eq!(
            published_payload_candidate(
                CodecSourceIndex::new(2),
                JocDispatchIndex::new(3),
                None,
                &provider(),
                &zero_slot_container(),
                &terminal_layout(),
            ),
            Err(F14kSourceBridgeError::F13PayloadShape)
        );
    }

    #[test]
    fn suppressed_sources_keep_absent_payload_and_no_au_receipt() {
        for source in 0..2 {
            let candidate = absent_payload_candidate(
                CodecSourceIndex::new(source),
                JocDispatchIndex::new(source + 1),
                None,
                false,
                false,
                false,
            )
            .unwrap();
            assert_eq!(candidate.source_index().value(), source);
            assert_eq!(candidate.au_index(), None);
            assert_eq!(
                candidate.outcome(),
                F14kPayloadOutcome::AbsentPublishedPayloadV1
            );
        }
        assert_eq!(
            absent_payload_candidate(
                CodecSourceIndex::new(2),
                JocDispatchIndex::new(3),
                Some(PublishedAuIndex::new(0)),
                false,
                false,
                false,
            ),
            Err(F14kSourceBridgeError::F13PayloadShape)
        );
    }
}
