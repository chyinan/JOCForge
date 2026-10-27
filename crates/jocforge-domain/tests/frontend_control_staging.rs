// pattern: Functional Core
use jocforge_domain::{
    frontend::{
        FrontendControlError, FrontendControlMachine, FrontendControlPhase,
        PackageManifestEvidence, ProviderFileEvidence, validate_f01e_package_manifest,
        validate_f01e_provider,
    },
    timeline::FrontendCallbackIndex,
    transaction::{AttemptId, CallbackLineageIdentity, ProgrammeId},
};
use std::path::PathBuf;

fn programme_id() -> ProgrammeId {
    ProgrammeId::new("F01EFrontTest001").unwrap()
}

fn callback_transaction(callback_index: u32) -> CallbackLineageIdentity {
    CallbackLineageIdentity::new(
        programme_id(),
        AttemptId::new(0),
        FrontendCallbackIndex::new(callback_index),
    )
}

fn validated_provider() -> jocforge_domain::frontend::ValidatedF01EColdProvider {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/f01e_provider");
    let manifest = PackageManifestEvidence::new(
        std::fs::read(root.join("MANIFEST.json")).unwrap().into(),
        None,
    );
    let provider = ProviderFileEvidence::new(
        std::fs::read(root.join("f01e-selected-constructor-v1.json"))
            .unwrap()
            .into(),
        None,
    );
    let manifest = validate_f01e_package_manifest(&manifest).unwrap();
    validate_f01e_provider(&provider, &manifest).unwrap()
}

#[test]
fn provider_constructs_exact_cold_root_and_digest() {
    let provider = validated_provider();
    let expected_digest = provider.receipt().canonical_sha256();
    let machine =
        FrontendControlMachine::from_provider(provider, programme_id(), AttemptId::new(0)).unwrap();
    let state = machine.committed();
    assert_eq!(state.schema(), "F01E_SELECTED_COLD_V1");
    assert_eq!(state.callback_index().value(), 0);
    assert_eq!(state.normal_callbacks().value(), 0);
    assert_eq!(state.remaining_cached_drains(), 4);
    assert_eq!(state.phase(), FrontendControlPhase::BeforeFirstNormal);
    assert_eq!(state.objects().len(), 28);
    assert_eq!(state.objects()[27].object_ordinal(), 27);
    assert!(state.last_control38_snapshot().is_none());
    assert!(state.last_regular37_snapshot().is_none());
    assert!(state.dbmd_binding().is_none());
    assert!(state.physical_binding().is_none());
    assert_eq!(state.normalized_config().sample_rate_hz(), 48_000);
    assert_eq!(state.normalized_config().channels(), 38);
    assert_eq!(state.normalized_config().pcm_bits(), 24);
    assert_eq!(state.normalized_config().source_samples(), 6_522_906);
    assert_eq!(state.normalized_config().frame_samples(), 1536);
    assert_eq!(state.canonical_bytes().len(), 4402);
    assert_eq!(state.digest().to_hex(), expected_digest.to_hex());
}

#[test]
fn cold_owner_state_reference_preserves_the_complete_provider_state() {
    let provider = validated_provider();
    let expected_digest = provider.receipt().canonical_sha256();
    let machine =
        FrontendControlMachine::from_provider(provider, programme_id(), AttemptId::new(0)).unwrap();
    let before = machine.committed().canonical_bytes().to_vec();
    let reference = machine.state_reference();

    assert_eq!(reference.owner_id(), "frontend_control");
    assert_eq!(reference.state_ref(), "owners.frontend_control");
    assert_eq!(reference.generation(), 0);
    assert!(std::ptr::eq(reference.state(), machine.committed()));
    assert_eq!(reference.canonical_bytes(), before.as_slice());
    assert_eq!(reference.state_sha256().to_hex(), expected_digest.to_hex());
    assert_eq!(reference.cold_sha256().to_hex(), expected_digest.to_hex());
    assert_eq!(machine.committed().canonical_bytes(), before.as_slice());
}

#[test]
fn prepare_and_stage_are_read_only_and_emit_join_metadata() {
    let provider = validated_provider();
    let receipt = provider.receipt().clone();
    let machine =
        FrontendControlMachine::from_provider(provider, programme_id(), AttemptId::new(0)).unwrap();
    let before = machine.committed().digest();
    let prepared = machine
        .prepare(callback_transaction(0), 0, &receipt)
        .unwrap();
    assert_eq!(machine.generation(), 0);
    assert_eq!(machine.committed().digest(), before);
    let staged = machine.stage(prepared).unwrap();
    assert_eq!(machine.generation(), 0);
    assert_eq!(machine.committed().digest(), before);
    assert_eq!(staged.after_digest(), before);
    assert_eq!(staged.staged_state_digest(), before);
    assert_eq!(
        staged.join_metadata().transaction_id(),
        &callback_transaction(0)
    );
    assert_eq!(staged.join_metadata().callback_index().value(), 0);
    assert_eq!(staged.join_metadata().before_generation(), 0);
    assert_eq!(staged.join_metadata().before_digest(), before);
}

#[test]
fn prepare_rejects_programme_callback_and_generation_mismatch() {
    let provider = validated_provider();
    let receipt = provider.receipt().clone();
    let machine =
        FrontendControlMachine::from_provider(provider, programme_id(), AttemptId::new(0)).unwrap();
    assert!(matches!(
        machine.prepare(
            CallbackLineageIdentity::new(
                ProgrammeId::new("OtherFrontendID1").unwrap(),
                AttemptId::new(0),
                FrontendCallbackIndex::new(0),
            ),
            0,
            &receipt
        ),
        Err(FrontendControlError::CallbackIdentityMismatch)
    ));
    assert!(matches!(
        machine.prepare(callback_transaction(1), 0, &receipt),
        Err(FrontendControlError::CallbackIndexMismatch)
    ));
    assert!(matches!(
        machine.prepare(callback_transaction(0), 1, &receipt),
        Err(FrontendControlError::GenerationMismatch)
    ));
}
