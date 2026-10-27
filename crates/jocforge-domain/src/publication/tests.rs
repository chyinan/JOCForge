use super::*;
use crate::timeline::{CodecSourceIndex, JocDispatchIndex, PublishedAuIndex};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

const OUTPUT_OWNER_MANIFEST: &[u8] =
    include_bytes!("../../tests/fixtures/output_owner/OUTPUT-OWNER-MANIFEST.json");
const OUTPUT_OWNER_VECTORS: &[u8] =
    include_bytes!("../../tests/fixtures/output_owner/OUTPUT-OWNER-VECTORS.json");
const SYNTHETIC_PROVIDER: &[u8] =
    include_bytes!("../../tests/fixtures/output_owner/synthetic-output-provider.json");
const OUTPUT_OWNER_MANIFEST_SHA256: &str =
    "d8d97f07d82d07dfa05772bb60bc90c6cab46e908f04366f24cc1398a16d169f";

fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn digest_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(64);
    for byte in digest(bytes) {
        text.push(char::from(DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    text
}

fn decode_hex(text: &str) -> Vec<u8> {
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = char::from(pair[0]).to_digit(16).expect("hex high nibble");
            let low = char::from(pair[1]).to_digit(16).expect("hex low nibble");
            ((high << 4) | low) as u8
        })
        .collect()
}

fn fixture_vectors() -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/output_owner");
    let manifest: Value = serde_json::from_slice(OUTPUT_OWNER_MANIFEST).expect("manifest JSON");
    assert_eq!(
        digest_hex(OUTPUT_OWNER_MANIFEST),
        OUTPUT_OWNER_MANIFEST_SHA256
    );
    for entry in manifest["files"].as_array().expect("manifest files") {
        let file = entry["file"].as_str().expect("manifest path");
        let bytes = std::fs::read(root.join(file)).expect("manifested output-owner file");
        assert_eq!(bytes.len() as u64, entry["bytes"].as_u64().unwrap());
        assert_eq!(digest_hex(&bytes), entry["sha256"].as_str().unwrap());
    }
    serde_json::from_slice(OUTPUT_OWNER_VECTORS).expect("output-owner vectors JSON")
}

fn synthetic_owner() -> OutputOwner {
    let configuration = OutputConfiguration::from_provider_json(
        SYNTHETIC_PROVIDER,
        4_608,
        "synthetic-0",
        "product.ec3",
    )
    .expect("selected synthetic provider and extent");
    OutputOwner::cold(configuration)
}

fn source_identity(source: u64) -> OutputSourceIdentity {
    OutputSourceIdentity::new(
        CodecSourceIndex::new(source),
        JocDispatchIndex::new(source + 1),
        source.checked_sub(2).map(PublishedAuIndex::new),
    )
}

#[test]
fn owner_state_reference_carries_complete_bytes_and_instance_cold_hash() {
    let vectors = fixture_vectors();
    let owner = synthetic_owner();
    let before = owner.state_reference().expect("canonical owner snapshot");
    let cold = &vectors["cold"];
    let expected_bytes = decode_hex(cold["canonical_hex"].as_str().unwrap());
    let expected_sha: [u8; 32] = decode_hex(cold["sha256"].as_str().unwrap())
        .try_into()
        .expect("cold SHA-256 width");

    assert_eq!(before.owner_id(), "output");
    assert_eq!(before.state_ref(), "owners.output");
    assert_eq!(before.generation(), 0);
    assert_eq!(before.canonical_bytes(), expected_bytes.as_slice());
    assert_eq!(before.state_sha256(), &expected_sha);
    assert_eq!(before.cold_sha256(), &expected_sha);

    let frame = [0_u8; 4_096];
    let proposal = owner
        .prepare_source(source_identity(0), &frame, &digest(&frame))
        .expect("suppressed source proposal");
    let OutputSourceProposal::Suppressed(proposal) = proposal else {
        panic!("source zero must be suppressed")
    };
    let after_owner = proposal.into_after().into_after_owner();
    let after = after_owner
        .state_reference()
        .expect("canonical proposed owner snapshot");

    assert_eq!(after.generation(), 1);
    assert_eq!(after.cold_sha256(), before.cold_sha256());
    assert_ne!(after.state_sha256(), before.state_sha256());
}

#[test]
fn source_proposals_match_suppression_writer_and_durable_ledger_vectors() {
    let vectors = fixture_vectors();
    let calls = vectors["continuous_calls"]
        .as_array()
        .expect("continuous calls");
    let mut owner = synthetic_owner();
    assert_eq!(owner.state_json(), vectors["cold"]["state"]);

    // The current transaction contract still requires complete scratch frames
    // for sources0/1, even though the local F15P fixture predates that clause.
    let zero_frame = [0_u8; 4_096];
    let zero_frame_sha256 = digest(&zero_frame);
    for source in 0..2 {
        let proposal = owner
            .prepare_source(source_identity(source), &zero_frame, &zero_frame_sha256)
            .expect("suppressed source proposal");
        let OutputSourceProposal::Suppressed(proposal) = proposal else {
            panic!("source {source} must be suppressed")
        };
        assert_eq!(
            proposal.receipt().as_json(),
            calls[source as usize]["prepared"]["receipt"]
        );
        owner = proposal.into_after().into_after_owner();
        assert_eq!(
            owner.state_json(),
            calls[source as usize]["committed_state"]
        );
    }

    for source in 2..5 {
        let call = &calls[source as usize];
        let frame = decode_hex(call["input"]["frame_hex"].as_str().unwrap());
        let declared_sha256 = call["input"]["declared_sha256"].as_str().unwrap();
        let frame_sha256: [u8; 32] = decode_hex(declared_sha256)
            .try_into()
            .expect("SHA-256 width");
        let proposal = owner
            .prepare_source(source_identity(source), &frame, &frame_sha256)
            .expect("published source proposal");
        let OutputSourceProposal::Append(proposal) = proposal else {
            panic!("source {source} must append")
        };
        assert_eq!(proposal.as_json(), call["prepared"]["proposal"]);

        let durable = proposal
            .record_durable_io_success(vec![u64::try_from(frame.len()).unwrap()], digest(&frame), 0)
            .expect("typed durable receipt after exact synthetic I/O");
        let after = proposal.complete(durable).expect("proposal/receipt join");
        let commit_receipt = after.receipt().as_json();
        assert_eq!(commit_receipt["schema"], "output-owner-commit-receipt.v1");
        assert_eq!(commit_receipt["owner_id"], "output");
        assert_eq!(commit_receipt["au_index"], source - 2);
        assert_eq!(commit_receipt["commit_count"], 1);
        owner = after.into_after_owner();
        assert_eq!(owner.state_json(), call["committed_state"]);
    }

    let finalize = owner.prepare_finalize().expect("terminal output proposal");
    assert_eq!(finalize.as_json(), vectors["finalize_proposal"]);
}

#[test]
fn source_preflight_rejects_wrong_clock_hash_crc_and_frame_length() {
    let owner = synthetic_owner();
    let frame = [0_u8; 4_096];
    let good_sha = digest(&frame);

    assert_eq!(
        owner.prepare_source(
            OutputSourceIdentity::new(
                CodecSourceIndex::new(0),
                JocDispatchIndex::new(2),
                None,
            ),
            &frame,
            &good_sha,
        )
        .unwrap_err(),
        OutputOwnerError::DispatchOrder
    );
    assert_eq!(
        owner
            .prepare_source(source_identity(0), &frame, &[0; 32],)
            .unwrap_err(),
        OutputOwnerError::FrameHash
    );

    let mut bad_crc = [0_u8; 4_096];
    bad_crc[2] = 1;
    assert_eq!(
        owner
            .prepare_source(source_identity(0), &bad_crc, &digest(&bad_crc))
            .unwrap_err(),
        OutputOwnerError::FrameCrc
    );
    assert_eq!(
        owner
            .prepare_source(source_identity(0), &[0; 4_095], &good_sha)
            .unwrap_err(),
        OutputOwnerError::FrameLength
    );
}
