#![cfg(feature = "test-support")]

use std::io::Cursor;

use jocforge_application::flat7x_pair_output::{
    Flat7xPairIoBinding, Flat7xPairIoHandoffError, handoff_source_receipted_pair_for_io,
};
use jocforge_domain::{
    joc::emdf::metadata_owner::test_support::Flat7xReceiptGateMetadataTestInputs,
    profile::Flat7xSourceGeneration,
    publication::{Flat7xAuAttemptRetirementReason, test_support::Flat7xAuAppendTestFixture},
    timeline::{CodecSourceIndex, JocDispatchIndex, PublishedAuIndex},
};
use jocforge_io::flat7x_au_append::{Flat7xAuAppendIoError, append_flat7x_au_to_memory_for_test};
use serde_json::Value;

const F15_PAIR_VECTORS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/FLAT7X-F15-RECEIPT-PAIR-INPUT-ONLY-V1/INPUT-ONLY-VECTORS.json"
);
const METADATA_OWNER_VECTORS: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/metadata_owner/METADATA-OWNER-VECTORS.json"
);
const METADATA_PUBLIC_PROVIDER: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/metadata_owner/metadata-public-provider.json"
);
const METADATA_WRAPPER_CONFIG: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/metadata_owner/metadata-wrapper-config.json"
);
const KEYLESS_INTEGRITY_PROVIDER: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/integrity_owner/integrity-public-provider.json"
);

fn source_receipted_pair_fixture() -> Flat7xAuAppendTestFixture {
    Flat7xAuAppendTestFixture::positive_pair(Flat7xReceiptGateMetadataTestInputs {
        f15_pair_vectors: F15_PAIR_VECTORS,
        metadata_owner_vectors: METADATA_OWNER_VECTORS,
        metadata_public_provider: METADATA_PUBLIC_PROVIDER,
        metadata_wrapper_config: METADATA_WRAPPER_CONFIG,
        keyless_integrity_provider: KEYLESS_INTEGRITY_PROVIDER,
    })
}

fn binding_from_fixture(
    fixture: &Flat7xAuAppendTestFixture,
    source_generation: Flat7xSourceGeneration,
    source_index: CodecSourceIndex,
    i0_frame_bytes: u16,
) -> Flat7xPairIoBinding {
    let vectors: Value = serde_json::from_slice(F15_PAIR_VECTORS).unwrap();
    let pair = &vectors["positive_pair"];
    let identity = &pair["integrity_identity"];
    let f13 = &pair["f13_metadata"];
    let i0 = &pair["i0_integrity_receipt"];
    let d0 = &pair["d0_integrity_receipt"];
    let f15 = &pair["f15_candidate"];

    Flat7xPairIoBinding::new(
        fixture.attempt_id().to_owned(),
        f15["f15_owner_generation_before"].as_u64().unwrap(),
        source_generation,
        (
            f13["metadata_owner_generation_before"].as_u64().unwrap(),
            f13["metadata_owner_generation_after_candidate"]
                .as_u64()
                .unwrap(),
        ),
        (
            i0["integrity_owner_generation"][0].as_u64().unwrap(),
            i0["integrity_owner_generation"][1].as_u64().unwrap(),
        ),
        (
            d0["integrity_owner_generation"][0].as_u64().unwrap(),
            d0["integrity_owner_generation"][1].as_u64().unwrap(),
        ),
        source_index,
        JocDispatchIndex::new(identity["dispatch"].as_u64().unwrap()),
        PublishedAuIndex::new(identity["published_au_index"].as_u64().unwrap()),
        0,
        i0_frame_bytes,
        u16::try_from(d0["frame_bytes"].as_u64().unwrap()).unwrap(),
    )
}

#[test]
fn source_receipted_pair_handoff_checks_identity_generations_and_extents_before_memory_io() {
    let fixture = source_receipted_pair_fixture();
    let vectors: Value = serde_json::from_slice(F15_PAIR_VECTORS).unwrap();
    let pair = &vectors["positive_pair"];
    let identity = &pair["integrity_identity"];
    let f15 = &pair["f15_candidate"];
    let source_generation =
        Flat7xSourceGeneration::new(f15["i0_source_generation_label"].as_u64().unwrap());
    let source_index = CodecSourceIndex::new(identity["source_index"].as_u64().unwrap());
    let i0_frame_bytes = u16::try_from(
        pair["i0_integrity_receipt"]["frame_bytes"]
            .as_u64()
            .unwrap(),
    )
    .unwrap();
    let binding = binding_from_fixture(&fixture, source_generation, source_index, i0_frame_bytes);

    let wrong_generation = binding_from_fixture(
        &fixture,
        Flat7xSourceGeneration::new(source_generation.value() + 1),
        source_index,
        i0_frame_bytes,
    );
    assert!(matches!(
        handoff_source_receipted_pair_for_io(
            fixture.append_proposal(fixture.attempt_id()).unwrap(),
            wrong_generation,
        ),
        Err(Flat7xPairIoHandoffError::SourceGenerationMismatch)
    ));

    let wrong_source = binding_from_fixture(
        &fixture,
        source_generation,
        CodecSourceIndex::new(source_index.value() + 1),
        i0_frame_bytes,
    );
    assert!(matches!(
        handoff_source_receipted_pair_for_io(
            fixture.append_proposal(fixture.attempt_id()).unwrap(),
            wrong_source,
        ),
        Err(Flat7xPairIoHandoffError::SourceIdentityMismatch)
    ));

    let wrong_extent = binding_from_fixture(
        &fixture,
        source_generation,
        source_index,
        i0_frame_bytes - 1,
    );
    assert!(matches!(
        handoff_source_receipted_pair_for_io(
            fixture.append_proposal(fixture.attempt_id()).unwrap(),
            wrong_extent,
        ),
        Err(Flat7xPairIoHandoffError::FrameExtentMismatch)
    ));

    let expected_bytes = fixture.expected_bytes();
    let proposal = fixture.append_proposal(fixture.attempt_id()).unwrap();
    let handoff = handoff_source_receipted_pair_for_io(proposal, binding.clone()).unwrap();
    assert_eq!(handoff.binding(), &binding);
    assert_eq!(handoff.bytes(), expected_bytes.as_slice());
    assert!(!handoff.production_allowed());

    let mut sink = Cursor::new(Vec::new());
    let failure =
        match append_flat7x_au_to_memory_for_test(&mut sink, handoff.into_append_proposal()) {
            Ok(_) => panic!("the in-memory sink cannot produce a durable after-owner"),
            Err(failure) => failure,
        };
    assert!(matches!(
        failure.error(),
        Flat7xAuAppendIoError::ReceiptCandidateRejected
    ));
    assert_eq!(
        failure.retirement().reason(),
        Flat7xAuAttemptRetirementReason::AppendReceiptCandidateMismatch
    );
    assert!(failure.retirement().committed_owner_unchanged());
    assert!(failure.retirement().fresh_programme_cold_required());
    assert_eq!(failure.retirement().next_access_unit().value(), 0);
    assert_eq!(failure.retirement().durable_length(), 0);
    assert_eq!(sink.into_inner(), expected_bytes);
}
