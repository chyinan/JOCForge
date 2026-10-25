// pattern: Imperative Shell
use super::*;
use crate::{
    frontend::{
        PackageManifestEvidence, ProviderFileEvidence, validate_f01e_package_manifest,
        validate_f01e_provider, validate_f03_package_manifest, validate_f03_target_cold_provider,
    },
    profile::EncodingProfile,
    spatial::{
        SpatialControlJoinMetadata, SpatialControlMachine,
        source_flags::{SourceFlagsAttempt, SourceFlagsProgrammeId},
        spatial_control::SpatialControlRenderAuthority,
    },
    timeline::FrontendCallbackIndex,
    transaction::{AttemptId, CallbackLineageIdentity},
};
use std::sync::Arc;

const MANIFEST_BYTES: &[u8] = include_bytes!("../../../tests/fixtures/f01e_provider/MANIFEST.json");
const F01E_PROVIDER_BYTES: &[u8] =
    include_bytes!("../../../tests/fixtures/f01e_provider/f01e-selected-constructor-v1.json");
const F03_PROVIDER_BYTES: &[u8] =
    include_bytes!("../../../tests/fixtures/target_cold/f03-selected-target-cold-v1.json");

fn test_programme_id() -> SourceFlagsProgrammeId {
    crate::transaction::test_programme_id("post-window-test-programme")
}

fn test_callback_transaction_id() -> CallbackLineageIdentity {
    CallbackLineageIdentity::new(
        test_programme_id(),
        AttemptId::new(3),
        FrontendCallbackIndex::new(0),
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::spatial) fn render_proposal_fixture(
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    spatial_control_after_proposal_identity: Arc<SpatialControlJoinMetadata>,
    phase: CallbackPhase,
    payload: Vec<f32>,
) -> Result<PostWindowRenderProposal, PostWindowFanoutError> {
    PostWindowRenderProposal::try_from_spatial_control_renderer(
        SpatialControlRenderAuthority::fixture(),
        programme_id,
        attempt,
        spatial_control_after_proposal_identity,
        phase,
        payload,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::spatial) fn current_features_audio_view_fixture(
    programme_id: SourceFlagsProgrammeId,
    attempt: SourceFlagsAttempt,
    spatial_control_after_proposal_identity: Arc<SpatialControlJoinMetadata>,
    phase: CallbackPhase,
    payload: Vec<f32>,
) -> Result<CurrentFeaturesAudioView, PostWindowFanoutError> {
    render_proposal_fixture(
        programme_id,
        attempt,
        spatial_control_after_proposal_identity,
        phase,
        payload,
    )
    .map(|proposal| proposal.fanout().0)
}

fn spatial_control() -> SpatialControlMachine {
    let profile = EncodingProfile::selected();
    let manifest = PackageManifestEvidence::new(Arc::from(MANIFEST_BYTES), None);
    let f01e_manifest = validate_f01e_package_manifest(&manifest).unwrap();
    let f01e = validate_f01e_provider(
        &ProviderFileEvidence::new(Arc::from(F01E_PROVIDER_BYTES), None),
        &f01e_manifest,
    )
    .unwrap();
    let f03_manifest = validate_f03_package_manifest(&manifest).unwrap();
    let f03 = validate_f03_target_cold_provider(
        &ProviderFileEvidence::new(Arc::from(F03_PROVIDER_BYTES), None),
        &f03_manifest,
    )
    .unwrap();
    SpatialControlMachine::new_cold(profile, &f01e, &f03).unwrap()
}

fn after_proposal_identity(machine: &SpatialControlMachine) -> Arc<SpatialControlJoinMetadata> {
    Arc::new(SpatialControlJoinMetadata::fixture(
        machine,
        test_callback_transaction_id(),
        FrontendCallbackIndex::new(0),
        machine.generation(),
        None,
        machine.profile(),
    ))
}

#[test]
fn fanout_returns_two_views_of_one_payload_with_explicit_phase_and_identity() {
    let machine = spatial_control();
    let after_proposal_identity = after_proposal_identity(&machine);
    let proposal = render_proposal_fixture(
        test_programme_id(),
        SourceFlagsAttempt::new(3),
        Arc::clone(&after_proposal_identity),
        CallbackPhase::CachedDrain,
        vec![0.0; PAYLOAD_SAMPLE_COUNT],
    )
    .unwrap();

    let (current_features, spatial_audio) = proposal.fanout();

    assert!(Arc::ptr_eq(
        &current_features.payload,
        &spatial_audio.payload
    ));
    assert_eq!(Arc::strong_count(&current_features.payload), 2);
    assert_eq!(current_features.payload.len(), PAYLOAD_SAMPLE_COUNT);
    assert_eq!(
        current_features.lineage.payload_byte_count,
        PAYLOAD_BYTE_COUNT
    );
    assert_eq!(current_features.lineage.phase, CallbackPhase::CachedDrain);
    assert_eq!(spatial_audio.lineage.phase, CallbackPhase::CachedDrain);
    assert_eq!(
        current_features.lineage.payload_sha256.0,
        [
            0x25, 0x04, 0x80, 0xec, 0xef, 0xa3, 0x65, 0x5c, 0x77, 0x4b, 0x6a, 0x4e, 0x5d, 0x4e,
            0xb1, 0x66, 0x75, 0xee, 0x4f, 0x18, 0x1c, 0x22, 0x85, 0xc2, 0x50, 0x20, 0xfa, 0xc6,
            0xe3, 0xaa, 0xf3, 0xb5,
        ]
    );
    assert!(Arc::ptr_eq(
        &current_features.lineage.render_stage_identity,
        &spatial_audio.lineage.render_stage_identity
    ));
    assert!(Arc::ptr_eq(
        &current_features
            .lineage
            .spatial_control_after_proposal_identity,
        &spatial_audio
            .lineage
            .spatial_control_after_proposal_identity
    ));
    assert_eq!(
        current_features.lineage.programme_id.as_ref(),
        &test_programme_id()
    );
    assert_eq!(current_features.lineage.attempt, SourceFlagsAttempt::new(3));
    for source in 0..SOURCE_COUNT {
        let source_samples = current_features.source_samples(source).unwrap();
        assert_eq!(source_samples.len(), SAMPLES_PER_SOURCE);
        assert!(source_samples.iter().all(|sample| sample.to_bits() == 0));
    }
    assert!(current_features.source_samples(SOURCE_COUNT).is_none());
}

#[test]
fn proposal_rejects_payloads_that_do_not_have_the_fixed_shape() {
    let machine = spatial_control();
    for sample_count in [PAYLOAD_SAMPLE_COUNT - 1, PAYLOAD_SAMPLE_COUNT + 1] {
        let result = render_proposal_fixture(
            test_programme_id(),
            SourceFlagsAttempt::new(3),
            after_proposal_identity(&machine),
            CallbackPhase::Normal,
            vec![0.0; sample_count],
        );
        assert_eq!(result.err(), Some(PostWindowFanoutError::PayloadShape));
    }
}

#[test]
fn proposal_rejects_non_finite_payload_samples() {
    let machine = spatial_control();
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut payload = vec![0.0; PAYLOAD_SAMPLE_COUNT];
        payload[PAYLOAD_SAMPLE_COUNT / 2] = invalid;
        let result = render_proposal_fixture(
            test_programme_id(),
            SourceFlagsAttempt::new(3),
            after_proposal_identity(&machine),
            CallbackPhase::Normal,
            payload,
        );
        assert_eq!(result.err(), Some(PostWindowFanoutError::NonFinitePayload));
    }
}

#[test]
fn payload_borrows_preserve_every_source_major_sample() {
    let machine = spatial_control();
    let payload = (0..PAYLOAD_SAMPLE_COUNT)
        .map(|index| index as f32)
        .collect();
    let proposal = render_proposal_fixture(
        test_programme_id(),
        SourceFlagsAttempt::new(3),
        after_proposal_identity(&machine),
        CallbackPhase::Normal,
        payload,
    )
    .unwrap();
    let (current_features, spatial_audio) = proposal.fanout();

    for source in 0..SOURCE_COUNT {
        let current_samples = current_features.source_samples(source).unwrap();
        let spatial_samples = spatial_audio.source_samples(source).unwrap();
        assert_eq!(current_samples.len(), SAMPLES_PER_SOURCE);
        assert_eq!(current_samples, spatial_samples);
        for (sample, value) in current_samples.iter().enumerate() {
            let source_major_index = source * SAMPLES_PER_SOURCE + sample;
            assert_eq!(value.to_bits(), (source_major_index as f32).to_bits());
        }
    }
}

#[test]
fn different_proposals_do_not_share_render_stage_identity_even_for_equal_audio() {
    let machine = spatial_control();
    let first = render_proposal_fixture(
        test_programme_id(),
        SourceFlagsAttempt::new(3),
        after_proposal_identity(&machine),
        CallbackPhase::Normal,
        vec![0.0; PAYLOAD_SAMPLE_COUNT],
    )
    .unwrap();
    let second = render_proposal_fixture(
        crate::transaction::test_programme_id("post-window-test-programme"),
        SourceFlagsAttempt::new(3),
        after_proposal_identity(&machine),
        CallbackPhase::Normal,
        vec![0.0; PAYLOAD_SAMPLE_COUNT],
    )
    .unwrap();
    let (first_current, first_spatial) = first.fanout();
    let (second_current, _) = second.fanout();

    assert_eq!(
        first_current.lineage.payload_sha256,
        second_current.lineage.payload_sha256
    );
    assert!(Arc::ptr_eq(&first_current.payload, &first_spatial.payload));
    assert!(!Arc::ptr_eq(
        &first_current.payload,
        &second_current.payload
    ));
    assert!(!Arc::ptr_eq(
        &first_current.lineage.render_stage_identity,
        &second_current.lineage.render_stage_identity
    ));
}
