//! Synthetic Flat7x F15 values for downstream IO integration tests.
//!
//! The fixture bypasses the production F13/F12 producers. It is available only when the
//! non-default `test-support` feature is enabled and grants no production authority.

use super::{
    Flat7xAuAppendProposal, Flat7xAuIntegrityFrameReceipt, Flat7xAuOutputOwner,
    Flat7xAuSourceReceiptInput,
};
use crate::{
    codec::frame_size::Eac3FrameSize,
    integrity::{
        crc16::eac3_crc16,
        keyless_checksum::sha256,
        programme_owner::{
            IntegrityF12SourceReceipts, IntegrityIdentity, IntegrityPayloadEvidence,
            synthetic_f12_receipts_for_flat7x_test,
        },
    },
    joc::emdf::metadata_owner::{
        MetadataSourceProposal,
        test_support::{
            Flat7xReceiptGateMetadataTestInputs, flat7x_receipt_gate_metadata_for_test,
        },
    },
    profile::{
        D0EmdfSkipFieldExtent, D0FrameExtent, Flat7xAuCandidate, Flat7xI0D0TargetPlan,
        Flat7xSourceGeneration, I0FrameExtent, compose_flat7x_au_byte_sequence,
        validate_flat7x_au_candidate,
    },
    timeline::{CodecSourceIndex, JocDispatchIndex, PublishedAuIndex},
};

/// Owned same-AU F13/F12 values for one synthetic pair-append IO test.
pub struct Flat7xAuAppendTestFixture {
    attempt_id: String,
    metadata: MetadataSourceProposal,
    i0_integrity: IntegrityF12SourceReceipts,
    d0_integrity: IntegrityF12SourceReceipts,
    i0_frame: Vec<u8>,
    d0_frame: Vec<u8>,
    source_generation: Flat7xSourceGeneration,
    d0_emdf_container_bytes: u16,
}

impl Flat7xAuAppendTestFixture {
    /// Builds one positive synthetic receipt-pair vector from the registered Wash input fixture.
    ///
    /// # Panics
    ///
    /// Panics when fixture bytes are missing, malformed, or inconsistent with the reviewed contract.
    pub fn positive_pair(input: Flat7xReceiptGateMetadataTestInputs<'_>) -> Self {
        let vectors: serde_json::Value = serde_json::from_slice(input.f15_pair_vectors)
            .expect("Wash receipt-pair vector is valid JSON");
        let pair = &vectors["positive_pair"];
        let identity = &pair["integrity_identity"];
        let (metadata, zero_slot_container, protected_container) =
            flat7x_receipt_gate_metadata_for_test(input);

        let source_index = CodecSourceIndex::new(identity["source_index"].as_u64().unwrap());
        let dispatch = JocDispatchIndex::new(identity["dispatch"].as_u64().unwrap());
        let au_index = PublishedAuIndex::new(identity["published_au_index"].as_u64().unwrap());
        let transaction_id = identity["transaction_id"].as_str().unwrap();
        let programme_id = identity["programme_id"].as_str().unwrap();
        let attempt = identity["attempt"].as_u64().unwrap();

        let i0_frame = crc_valid_frame(2_560, 0x31);
        let d0_frame = crc_valid_frame(4_096, 0x72);
        let shared_identity = IntegrityIdentity {
            programme_id: programme_id.to_owned(),
            attempt,
            transaction_id: transaction_id.to_owned(),
            source_index,
            dispatch,
            au_index: Some(au_index),
        };
        let i0_integrity = synthetic_f12_receipts_for_flat7x_test(
            shared_identity.clone(),
            (10, 11),
            &i0_frame,
            IntegrityPayloadEvidence::AbsentPublishedPayloadV1,
            None,
            None,
        );
        let d0_integrity = synthetic_f12_receipts_for_flat7x_test(
            shared_identity,
            (20, 21),
            &d0_frame,
            IntegrityPayloadEvidence::ProtectedEmdf {
                zero_slot_sha256: sha256(&zero_slot_container),
                protected_sha256: protected_container.output_sha256(),
                message_sha256: sha256(&zero_slot_container[4..]),
                primary_crc32: protected_container.primary_crc32(),
                secondary_crc8: protected_container.secondary_crc8(),
                primary_range: protected_container.primary_range(),
                secondary_range: protected_container.secondary_range(),
                container_bytes: protected_container.bytes().len(),
            },
            Some(crate::codec::bitio::FrameBitCursor::new(512)),
            Some(protected_container.output_sha256()),
        );

        Self {
            attempt_id: attempt.to_string(),
            metadata,
            i0_integrity,
            d0_integrity,
            i0_frame,
            d0_frame,
            source_generation: Flat7xSourceGeneration::new(
                pair["f15_candidate"]["i0_source_generation_label"]
                    .as_u64()
                    .unwrap(),
            ),
            d0_emdf_container_bytes: u16::try_from(protected_container.bytes().len()).unwrap(),
        }
    }

    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    /// Returns exact pair bytes for a staging readback assertion.
    pub fn expected_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.i0_frame.len() + self.d0_frame.len());
        bytes.extend_from_slice(&self.i0_frame);
        bytes.extend_from_slice(&self.d0_frame);
        bytes
    }

    /// Creates a one-AU pair proposal with the requested owner attempt label.
    ///
    /// # Panics
    ///
    /// Panics if the fixture no longer satisfies its embedded target shape or integrity checks.
    pub fn append_proposal(
        &self,
        owner_attempt_id: &str,
    ) -> Result<Flat7xAuAppendProposal<'_>, crate::publication::Flat7xAuAttemptRetirement> {
        let target = Flat7xI0D0TargetPlan::approved_contract();
        let candidate = Flat7xAuCandidate::new(
            Some(I0FrameExtent::new(
                self.source_generation,
                target.i0().frame_bytes(),
            )),
            Some(D0FrameExtent::new(
                self.source_generation,
                target.d0().frame_bytes(),
            )),
            Some(D0EmdfSkipFieldExtent::new(
                self.source_generation,
                2,
                1,
                Some(self.d0_emdf_container_bytes),
                false,
            )),
        );
        let validation = validate_flat7x_au_candidate(target, candidate)
            .expect("test fixture has the exact approved Flat7x pair shape");
        let sequence = compose_flat7x_au_byte_sequence(&validation, &self.i0_frame, &self.d0_frame)
            .expect("test fixture frames have the exact pair extents and CRCs");
        Flat7xAuOutputOwner::cold(owner_attempt_id, 1)
            .expect("test owner attempt is nonempty")
            .stage_next_access_unit(
                sequence,
                Flat7xAuSourceReceiptInput::new(
                    Some(&self.metadata),
                    Some(Flat7xAuIntegrityFrameReceipt::independent_i0(
                        &self.i0_integrity,
                    )),
                    Some(Flat7xAuIntegrityFrameReceipt::dependent_d0(
                        &self.d0_integrity,
                    )),
                ),
            )
    }
}

fn crc_valid_frame(frame_bytes: usize, marker: u8) -> Vec<u8> {
    let size = Eac3FrameSize::from_frame_bytes(frame_bytes).expect("selected pair frame size");
    let mut bytes = vec![0_u8; frame_bytes];
    bytes[..2].copy_from_slice(&[0x0b, 0x77]);
    bytes[100] = marker;
    for offset in 0..11 {
        let value = (size.code().value() >> (10 - offset)) & 1;
        if value != 0 {
            let bit = 21 + offset;
            bytes[bit / 8] |= 1 << (7 - (bit % 8));
        }
    }
    let crc_offset = bytes.len() - 2;
    let crc = eac3_crc16(&bytes[2..crc_offset]);
    bytes[crc_offset..].copy_from_slice(&crc.to_be_bytes());
    assert_eq!(eac3_crc16(&bytes[2..]), 0);
    bytes
}
