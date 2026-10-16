// pattern: Functional Core
use jocforge_domain::{
    frontend::{
        F03TargetProviderError, PackageManifestEvidence, ProviderFileEvidence,
        ValidatedF01EColdProvider, ValidatedF03TargetColdProvider, validate_f01e_package_manifest,
        validate_f01e_provider, validate_f03_package_manifest, validate_f03_target_cold_provider,
    },
    profile::{
        AuthoredInputProfile, CodecRateProfile, CoreCarrierProfile, EmdfCarriageProfile,
        EncodingProfile, JocDownmixProfile, SubstreamTopology,
    },
    spatial::{PacketReadiness, SpatialControlError, SpatialControlMachine},
    timeline::AdmSampleIndex,
    transaction::{CallbackTransactionId, CallbackTransactionIdError},
};
use std::sync::Arc;

const MANIFEST_BYTES: &[u8] = include_bytes!("fixtures/f01e_provider/MANIFEST.json");
const F01E_PROVIDER_BYTES: &[u8] =
    include_bytes!("fixtures/f01e_provider/f01e-selected-constructor-v1.json");
const F03_PROVIDER_BYTES: &[u8] =
    include_bytes!("fixtures/target_cold/f03-selected-target-cold-v1.json");
const PROFILE_IDENTITY_VECTORS: &[u8] =
    include_bytes!("fixtures/spatial_control_state_v5/PROFILE-IDENTITY-MISMATCH-VECTORS.json");

fn validated_cold_providers() -> (ValidatedF01EColdProvider, ValidatedF03TargetColdProvider) {
    let f01e_manifest_evidence = PackageManifestEvidence::new(Arc::from(MANIFEST_BYTES), None);
    let f01e_manifest = validate_f01e_package_manifest(&f01e_manifest_evidence).unwrap();
    let f01e_evidence = ProviderFileEvidence::new(Arc::from(F01E_PROVIDER_BYTES), None);
    let f01e = validate_f01e_provider(&f01e_evidence, &f01e_manifest).unwrap();

    let f03_manifest_evidence = PackageManifestEvidence::new(Arc::from(MANIFEST_BYTES), None);
    let f03_manifest = validate_f03_package_manifest(&f03_manifest_evidence).unwrap();
    let f03_evidence = ProviderFileEvidence::new(Arc::from(F03_PROVIDER_BYTES), None);
    let f03 = validate_f03_target_cold_provider(&f03_evidence, &f03_manifest).unwrap();
    (f01e, f03)
}

fn selected_machine() -> SpatialControlMachine {
    let (f01e, f03) = validated_cold_providers();
    SpatialControlMachine::new_cold(EncodingProfile::selected(), &f01e, &f03).unwrap()
}

#[test]
fn cold_root_has_the_complete_v5_canonical_preimage_and_digest() {
    let machine = selected_machine();
    let state = machine.committed();
    let vector: serde_json::Value = serde_json::from_slice(PROFILE_IDENTITY_VECTORS).unwrap();
    let before = &vector["cases"][0]["before"];
    let lineage = machine.provider_lineage();

    assert_eq!(machine.generation(), 0);
    assert_eq!(machine.profile(), EncodingProfile::selected());
    assert_eq!(state.source_cursor_samples(), AdmSampleIndex::new(0));
    assert_eq!(state.canonical_bytes().len(), 768_582);
    assert_eq!(
        state.digest().to_hex(),
        "cb9b512e03d82604270a22cf60e288f9ea8f6aa808517a6319a27f860552bcc0"
    );
    assert_eq!(state.event_control_canonical_bytes().len(), 4_402);
    assert_eq!(
        machine.digest().to_hex(),
        before["state_sha256"].as_str().unwrap()
    );
    assert_eq!(
        lineage.f01e_manifest_sha256().to_hex(),
        before["f01e_receipt_lineage"]["manifest_sha256"]
            .as_str()
            .unwrap()
    );
    assert_eq!(
        lineage.f01e_provider_file_sha256().to_hex(),
        before["f01e_receipt_lineage"]["provider_file_sha256"]
            .as_str()
            .unwrap()
    );
    assert_eq!(
        lineage.f01e_canonical_sha256().to_hex(),
        before["f01e_receipt_lineage"]["canonical_sha256"]
            .as_str()
            .unwrap()
    );
    assert_eq!(
        lineage.f03_manifest_sha256().to_hex(),
        before["f03_receipt_lineage"]["manifest_sha256"]
            .as_str()
            .unwrap()
    );
    assert_eq!(
        lineage.f03_provider_file_sha256().to_hex(),
        before["f03_receipt_lineage"]["provider_file_sha256"]
            .as_str()
            .unwrap()
    );
    assert_eq!(
        lineage.f03_target_projection_sha256().to_hex(),
        before["f03_receipt_lineage"]["target_projection_sha256"]
            .as_str()
            .unwrap()
    );
    assert_eq!(state.source_vector_bits().len(), 48 * 24);
    assert_eq!(state.source_scalar_bits().len(), 48);
    assert!(state.source_vector_bits().iter().all(|bits| *bits == 0));
    assert!(state.source_scalar_bits().iter().all(|bits| *bits == 0));
    assert_eq!(state.target_active_u8(), &[0; 15]);
    assert_eq!(state.previous_marker_active_u8(), &[0; 15]);
    assert_eq!(state.callback_index().value(), 0);
    assert_eq!(state.source_cursor_samples().value(), 0);
    assert_eq!(state.normal_callbacks().value(), 0);
    assert_eq!(state.remaining_cached_drains(), 4);
    assert!(!state.retired());
    assert!(state.source_cache_is_cold());
    assert_eq!(state.regular_raw_history_bits().len(), 37 * 1536);
    assert_eq!(state.previous_pcm_bits().len(), 38 * 1536);
    assert_eq!(state.previous_current_rows_bits().len(), 48 * 1536);
    assert!(
        state
            .regular_raw_history_bits()
            .iter()
            .all(|bits| *bits == 0)
    );
    assert!(state.previous_pcm_bits().iter().all(|bits| *bits == 0));
    assert!(
        state
            .previous_current_rows_bits()
            .iter()
            .all(|bits| *bits == 0)
    );
    assert_eq!(state.target_current_xyz_u32(), &[0; 45]);
    assert_eq!(state.target_current_id_u32(), &[u32::MAX; 15]);
    assert_eq!(state.target_current_attribute_u32(), &[0x0000_0b00; 15]);
    assert_eq!(state.target_previous_xyz_u32(), &[0; 45]);
    assert_eq!(state.target_previous_id_u32(), &[u32::MAX; 15]);
    assert_eq!(state.target_previous_attribute_u32(), &[0x0000_0b00; 15]);
    assert_eq!(state.assignment_i32(), &[-1; 15]);
    assert_eq!(state.markers_i32(), &[0; 15]);
    assert_eq!(state.matrix_f32_bits().len(), 15 * 48);
    assert!(state.matrix_f32_bits().iter().all(|bits| *bits == 0));
    assert_eq!(state.writer_set_record_count(), 0);
}

#[test]
fn packet_producers_are_not_ready_in_a_cold_machine() {
    let machine = selected_machine();
    assert_eq!(machine.packet_readiness(), PacketReadiness::NotReady);
}

#[test]
fn profile_only_mismatch_rejects_without_changing_root_identity_or_generation() {
    let machine = selected_machine();
    let before_digest = machine.digest();
    let before_generation = machine.generation();
    let unselected = EncodingProfile::new(
        AuthoredInputProfile::Unsupported,
        CoreCarrierProfile::SelectedFivePointOneSixCarrier,
        JocDownmixProfile::SelectedMode0Phase1Index3,
        SubstreamTopology::SelectedSingleIndependentZeroDependent,
        CodecRateProfile::Selected48Khz1024Kbps6Blocks1536Samples4096Bytes,
        EmdfCarriageProfile::DeployedCompat,
    );

    let vector: serde_json::Value = serde_json::from_slice(PROFILE_IDENTITY_VECTORS).unwrap();
    let case = &vector["cases"][0];
    let actual = machine.validate_profile_identity(&unselected);
    let after_digest = machine.digest();
    let after_generation = machine.generation();

    assert_eq!(machine.profile(), EncodingProfile::selected());
    assert_eq!(
        before_digest.to_hex(),
        case["before"]["state_sha256"].as_str().unwrap()
    );
    assert_eq!(
        before_generation,
        case["before"]["generation"].as_u64().unwrap()
    );
    assert_eq!(
        case["before"]["f01e_receipt_lineage"],
        case["proposal"]["f01e_receipt_lineage"]
    );
    assert_eq!(
        case["before"]["f03_receipt_lineage"],
        case["proposal"]["f03_receipt_lineage"]
    );
    assert_eq!(
        case["before"]["state_sha256"],
        case["proposal"]["state_sha256"]
    );
    assert_eq!(case["before"]["generation"], case["proposal"]["generation"]);
    assert_eq!(case["expected"]["result"], "REJECT_PROFILE_BINDING");
    assert_eq!(case["expected"]["state_sha256"], "unchanged");
    assert_eq!(actual, Err(SpatialControlError::ProfileBindingMismatch));
    assert_eq!(after_digest, before_digest);
    assert_eq!(after_generation, before_generation);
    assert_eq!(case["id"], "same_hash_rejects_unselected_profile_identity");
    assert_eq!(
        after_generation,
        case["expected"]["generation"].as_u64().unwrap()
    );
}

#[test]
fn provider_identity_mismatch_is_rejected_without_mutating_existing_cold_root() {
    let machine = selected_machine();
    let before_digest = machine.digest();
    let before_generation = machine.generation();

    let manifest_evidence = PackageManifestEvidence::new(Arc::from(MANIFEST_BYTES), None);
    let manifest = validate_f03_package_manifest(&manifest_evidence).unwrap();
    let mut changed_provider = F03_PROVIDER_BYTES.to_vec();
    changed_provider[0] ^= 1;
    let changed_evidence = ProviderFileEvidence::new(Arc::from(changed_provider), None);
    let result = validate_f03_target_cold_provider(&changed_evidence, &manifest);

    assert!(matches!(
        result,
        Err(F03TargetProviderError::ProviderIdentityMismatch)
    ));
    assert_eq!(machine.digest(), before_digest);
    assert_eq!(machine.generation(), before_generation);
}

#[test]
fn callback_transaction_id_rejects_empty_identity() {
    assert_eq!(
        CallbackTransactionId::new(""),
        Err(CallbackTransactionIdError::Empty)
    );
    assert_eq!(
        CallbackTransactionId::new("callback-1").unwrap().as_str(),
        "callback-1"
    );
}
