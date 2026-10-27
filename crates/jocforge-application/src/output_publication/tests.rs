use super::{OutputOwner, OutputRootBindingError, OutputStagingOwnerId, ProgrammeOutputBinding};
use jocforge_domain::transaction::{AttemptId, ProgrammeId};
use jocforge_domain::{
    profile::{EmdfCarriageProfile, EncodingProfile, resolve_selected_profile},
    publication::{OutputProvider, OutputSourceIdentity},
    timeline::{CodecSourceIndex, JocDispatchIndex},
};
use sha2::{Digest, Sha256};

fn output_provider() -> OutputProvider {
    OutputProvider {
        profile_id: "selected-output-provider.v1".to_owned(),
        provider_sha256: [0x5a; 32],
        frame_bytes: 4096,
        suppressed_sources: 2,
        staging_name_template: "attempt-{attempt_id}.partial".to_owned(),
        intent_name_template: "attempt-{attempt_id}.intent".to_owned(),
    }
}

#[test]
fn private_f15_projection_matches_both_washed_canonical_provider_hashes() {
    let deployed_profile = resolve_selected_profile(&EncodingProfile::selected())
        .expect("selected default carriage resolves");
    let etsi_profile = resolve_selected_profile(&EncodingProfile::selected_with_carriage(
        EmdfCarriageProfile::EtsiEmdf,
    ))
    .expect("selected ETSI carriage resolves");

    let deployed = super::project_selected_f15_output_provider(deployed_profile)
        .expect("deployed compatibility provider projection");
    let etsi = super::project_selected_f15_output_provider(etsi_profile)
        .expect("ETSI EMDF provider projection");

    assert_eq!(
        deployed.provider_sha256,
        [
            0xe5, 0xd1, 0x8a, 0x8b, 0x14, 0x0e, 0x76, 0x82, 0x79, 0xef, 0x0c, 0xc2, 0x82, 0xaa,
            0xfc, 0x34, 0x95, 0xf4, 0x27, 0x20, 0x03, 0x0e, 0xd9, 0x55, 0x22, 0xbf, 0x8e, 0xd2,
            0x40, 0xb2, 0x21, 0xc5,
        ]
    );
    assert_eq!(
        etsi.provider_sha256,
        [
            0xd4, 0xba, 0x53, 0x3d, 0x06, 0xdd, 0xf9, 0xbd, 0xa3, 0x30, 0x02, 0x38, 0x91, 0x91,
            0x9e, 0x8a, 0x23, 0x3e, 0xe8, 0xa1, 0x26, 0x11, 0xee, 0xc0, 0x51, 0x70, 0x80, 0x12,
            0x03, 0x4b, 0x16, 0x87,
        ]
    );
    assert_eq!(deployed.profile_id, "selected-keyless-output-policy.v1");
    assert_eq!(etsi.profile_id, "selected-keyless-output-policy.v1");
    assert_eq!(deployed.frame_bytes, 4_096);
    assert_eq!(etsi.frame_bytes, 4_096);
    assert_eq!(deployed.suppressed_sources, 2);
    assert_eq!(etsi.suppressed_sources, 2);
    assert_ne!(deployed.provider_sha256, etsi.provider_sha256);
}

fn binding(
    programme: &str,
    attempt: u64,
    staging_owner: &str,
    product_name: &str,
) -> ProgrammeOutputBinding {
    ProgrammeOutputBinding::for_test_values(
        ProgrammeId::new(programme).expect("programme id is nonempty"),
        AttemptId::new(attempt),
        OutputStagingOwnerId::new(staging_owner).expect("staging identity is nonempty"),
        output_provider(),
        product_name,
        1536,
    )
    .expect("typed output binding is valid")
}

fn untyped_owner(attempt: &str) -> OutputOwner {
    let configuration = jocforge_domain::publication::OutputConfiguration::new(
        output_provider(),
        attempt,
        "product.ec3",
        1536,
    )
    .expect("test output owner configuration");
    OutputOwner::cold(configuration)
}

fn suppressed_source_proposal(root: &ProgrammeOutputBinding) -> super::OutputSourceProposal {
    let frame = [0_u8; 4096];
    let frame_sha256: [u8; 32] = Sha256::digest(frame).into();
    root.output_owner()
        .prepare_source(
            OutputSourceIdentity::new(CodecSourceIndex::new(0), JocDispatchIndex::new(1), None),
            &frame,
            &frame_sha256,
        )
        .expect("source zero is a valid suppressed proposal")
}

#[test]
fn empty_staging_owner_identity_is_rejected_without_output_state() {
    assert!(matches!(
        OutputStagingOwnerId::new(""),
        Err(OutputRootBindingError::EmptyStagingOwnerId)
    ));
}

#[test]
fn typed_attempts_render_as_canonical_decimal_and_keep_staging_identity_separate() {
    for (value, rendered) in [
        (0, "0"),
        (1, "1"),
        (10, "10"),
        (u64::MAX, "18446744073709551615"),
    ] {
        let owner_id = format!("staging-{rendered}");
        let output = binding("programme-A", value, &owner_id, "product-A.ec3");
        assert_eq!(output.output_owner().configuration().attempt_id(), rendered);
        assert_eq!(output.attempt_id(), AttemptId::new(value));
        assert_eq!(output.staging_owner_id().as_str(), owner_id);
    }
}

#[test]
fn separate_programmes_may_both_start_at_zero_and_restart_with_new_staging_identity() {
    let first = binding("programme-A", 0, "opaque-A-0", "A.ec3");
    let second = binding("programme-B", 0, "opaque-B-0", "B.ec3");
    let restarted = binding("programme-A", 1, "opaque-A-1", "A.ec3");

    assert_eq!(first.attempt_id(), second.attempt_id());
    assert_ne!(first.programme_id(), second.programme_id());
    assert_ne!(first.staging_owner_id(), second.staging_owner_id());
    assert_eq!(restarted.attempt_id(), AttemptId::new(1));
    assert_eq!(restarted.output_owner().configuration().attempt_id(), "1");
    assert_ne!(first.staging_owner_id(), restarted.staging_owner_id());
}

#[test]
fn stale_typed_binding_rejects_without_changing_output_state() {
    let output = binding("programme-A", 0, "opaque-A-0", "A.ec3");
    let before = output.output_owner().state_json();

    assert_eq!(
        output.validate_lineage(&ProgrammeId::new("programme-B").unwrap(), AttemptId::new(0)),
        Err(super::OutputPublicationPreparationError::ProgrammeIdentityMismatch)
    );
    assert_eq!(
        output.validate_lineage(&ProgrammeId::new("programme-A").unwrap(), AttemptId::new(1)),
        Err(super::OutputPublicationPreparationError::AttemptIdentityMismatch)
    );
    assert_eq!(output.output_owner().state_json(), before);
}

#[test]
fn bound_source_proposal_rejects_cross_programme_and_stale_generation_before_io() {
    let output = binding("programme-A", 0, "opaque-A-0", "A.ec3");
    let same_attempt_other_programme = binding("programme-B", 0, "opaque-B-0", "A.ec3");
    let before = output.output_owner().state_json();
    let bound_proposal = output
        .bind_source_proposal(suppressed_source_proposal(&output))
        .unwrap();

    assert!(output.validate_source_proposal(&bound_proposal).is_ok());
    assert_eq!(
        same_attempt_other_programme.validate_source_proposal(&bound_proposal),
        Err(OutputRootBindingError::ProgrammeIdentityMismatch)
    );

    let after_owner = match suppressed_source_proposal(&output) {
        super::OutputSourceProposal::Suppressed(proposal) => {
            proposal.into_after().into_after_owner()
        }
        super::OutputSourceProposal::Append(_) => panic!("source zero must stay suppressed"),
    };
    let stale_generation = ProgrammeOutputBinding::bind_test_owner(
        ProgrammeId::new("programme-A").unwrap(),
        AttemptId::new(0),
        OutputStagingOwnerId::new("opaque-A-0").unwrap(),
        after_owner,
    )
    .unwrap();
    assert_eq!(
        stale_generation.validate_source_proposal(&bound_proposal),
        Err(OutputRootBindingError::ProposalBindingMismatch)
    );
    assert_eq!(output.output_owner().state_json(), before);
    assert_eq!(
        same_attempt_other_programme
            .output_owner()
            .configuration()
            .attempt_id(),
        "0"
    );
}

#[test]
fn output_owner_with_a_noncanonical_or_stale_attempt_cannot_be_bound() {
    for stale_or_noncanonical in ["00", "01", "+1", "-1", " 1", "1 ", "0x1", "1_000", "1"] {
        let result = ProgrammeOutputBinding::bind_test_owner(
            ProgrammeId::new("programme-A").unwrap(),
            AttemptId::new(0),
            OutputStagingOwnerId::new("opaque-A-0").unwrap(),
            untyped_owner(stale_or_noncanonical),
        );
        assert!(
            matches!(result, Err(OutputRootBindingError::AttemptBindingMismatch)),
            "accepted stale or noncanonical attempt {stale_or_noncanonical:?}"
        );
    }
}
