// pattern: Functional Core
use jocforge_application::{
    PreparedJobInputError, SelectedAdmResourcePayloads, prepare_selected_job_input,
};
use jocforge_domain::frontend::job_configuration::SelectedWaveDescriptor;
use jocforge_domain::transaction::ProgrammeId;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{ops::Range, sync::Arc};

const AXML: &[u8] =
    include_bytes!("../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.xml");
const CHNA: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_track_associations.bin"
);
const DBMD: &[u8] =
    include_bytes!("../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.dbmd");

fn test_programme_id() -> ProgrammeId {
    ProgrammeId::new("JOCForgeTestID01").unwrap()
}

#[test]
fn value_preparation_binds_graph_and_identity_to_one_immutable_resource() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    // This synthetic container exercises identity and chunk binding only; it is
    // not evidence of selected-profile source admission or audio processing.
    let (resource, spans) = selected_resource();
    let expected_digest: [u8; 32] = Sha256::digest(resource.as_ref()).into();
    let mut accepted = 0;
    let mut rejected = 0;
    for case in vectors["config_cases"].as_array().unwrap() {
        let descriptor = &case["pcm_descriptor"];
        let wave = SelectedWaveDescriptor::new(
            descriptor["sample_rate"].as_u64().unwrap() as u32,
            descriptor["channels"].as_u64().unwrap() as u16,
            descriptor["pcm_bits"].as_u64().unwrap() as u16,
        );
        let result = prepare_selected_job_input(
            test_programme_id(),
            case["input_xml"].as_str().unwrap().as_bytes(),
            wave,
            descriptor["source_samples"].as_u64().unwrap(),
            Arc::clone(&resource),
            metadata_payloads(&resource, &spans),
        );
        if case["expected"]["status"] == "ACCEPT" {
            let prepared = result.unwrap();
            assert_eq!(prepared.resource_identity().input_sha256(), expected_digest);
            assert_eq!(
                prepared.resource_identity().resource_bytes(),
                resource.len() as u64
            );
            assert_eq!(prepared.resource(), resource.as_ref());
            assert_eq!(prepared.resource().as_ptr(), resource.as_ptr());
            assert_eq!(prepared.programme_id(), &test_programme_id());
            assert_eq!(
                prepared.configuration().source_samples(),
                descriptor["source_samples"].as_u64().unwrap()
            );
            assert_eq!(prepared.admitted_object_events().events().len(), 28);
            assert_eq!(
                prepared.graph().graph().receipt().physical_track_count(),
                38
            );
            assert_eq!(
                prepared.graph().provenance().parent_resource(),
                prepared.resource_identity()
            );
            assert_eq!(prepared.graph().provenance().axml().chunk_id(), *b"axml");
            assert_eq!(prepared.graph().provenance().chna().chunk_id(), *b"chna");
            assert_eq!(prepared.graph().provenance().dbmd().chunk_id(), *b"dbmd");
            let xml_sha256: [u8; 32] =
                Sha256::digest(case["input_xml"].as_str().unwrap().as_bytes()).into();
            assert_eq!(prepared.job_xml_sha256(), xml_sha256);
            assert_eq!(
                prepared.identity_envelope()["job_xml_sha256"],
                hex_lower(&xml_sha256)
            );
            let config_bytes = serde_json::to_vec(prepared.identity_envelope()).unwrap();
            assert_eq!(
                prepared.config_sha256(),
                <[u8; 32]>::from(Sha256::digest(config_bytes))
            );
            accepted += 1;
        } else {
            assert!(result.is_err());
            rejected += 1;
        }
    }
    assert_eq!((accepted, rejected), (3, 8));

    let accepted_case = &vectors["config_cases"][0];
    let error = prepare_selected_job_input(
        test_programme_id(),
        accepted_case["input_xml"].as_str().unwrap().as_bytes(),
        SelectedWaveDescriptor::new(48_000, 38, 24),
        13_888_072,
        Arc::clone(&resource),
        metadata_payloads(&resource, &spans),
    )
    .unwrap_err();
    assert!(matches!(error, PreparedJobInputError::GraphAdmission(_)));
    assert!(jocforge_application::encode().is_err());
}

#[test]
fn metadata_from_a_different_resource_allocation_is_rejected() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let (resource, _) = selected_resource();
    let foreign_axml = AXML.to_vec();
    let foreign_chna = CHNA.to_vec();
    let foreign_dbmd = DBMD.to_vec();
    let metadata = SelectedAdmResourcePayloads::new(&foreign_axml, &foreign_chna, &foreign_dbmd);
    let accepted_case = &vectors["config_cases"][0];
    let result = prepare_selected_job_input(
        test_programme_id(),
        accepted_case["input_xml"].as_str().unwrap().as_bytes(),
        SelectedWaveDescriptor::new(48_000, 38, 24),
        accepted_case["pcm_descriptor"]["source_samples"]
            .as_u64()
            .unwrap(),
        resource,
        metadata,
    );
    assert_eq!(
        result.unwrap_err(),
        PreparedJobInputError::MetadataPayloadBinding
    );
}

#[test]
fn programme_id_is_caller_supplied_and_separate_from_content_identity() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = &vectors["config_cases"][0];
    let descriptor = &case["pcm_descriptor"];
    let wave = SelectedWaveDescriptor::new(
        descriptor["sample_rate"].as_u64().unwrap() as u32,
        descriptor["channels"].as_u64().unwrap() as u16,
        descriptor["pcm_bits"].as_u64().unwrap() as u16,
    );
    let (resource, spans) = selected_resource();
    let first = prepare_selected_job_input(
        ProgrammeId::new("JOCForgeTestID01").unwrap(),
        case["input_xml"].as_str().unwrap().as_bytes(),
        wave,
        descriptor["source_samples"].as_u64().unwrap(),
        Arc::clone(&resource),
        metadata_payloads(&resource, &spans),
    )
    .unwrap();
    let second = prepare_selected_job_input(
        ProgrammeId::new("JOCForgeOtherID1").unwrap(),
        case["input_xml"].as_str().unwrap().as_bytes(),
        wave,
        descriptor["source_samples"].as_u64().unwrap(),
        Arc::clone(&resource),
        metadata_payloads(&resource, &spans),
    )
    .unwrap();

    assert_ne!(first.programme_id(), second.programme_id());
    assert_eq!(first.resource_identity(), second.resource_identity());
    assert_eq!(first.job_xml_sha256(), second.job_xml_sha256());
    assert_eq!(first.config_sha256(), second.config_sha256());
    assert!(first.identity_envelope().get("programme_id").is_none());
}

fn selected_resource() -> (Arc<[u8]>, [Range<usize>; 3]) {
    let mut bytes = b"RIFF\0\0\0\0WAVE".to_vec();
    append_chunk(&mut bytes, *b"fmt ", &selected_format_payload());
    append_chunk(&mut bytes, *b"data", &[0_u8; 114]);
    let axml = append_chunk(&mut bytes, *b"axml", AXML);
    let chna = append_chunk(&mut bytes, *b"chna", CHNA);
    let dbmd = append_chunk(&mut bytes, *b"dbmd", DBMD);
    let riff_size = u32::try_from(bytes.len() - 8).unwrap();
    bytes[4..8].copy_from_slice(&riff_size.to_le_bytes());
    (Arc::from(bytes.into_boxed_slice()), [axml, chna, dbmd])
}

fn selected_format_payload() -> Vec<u8> {
    let mut format = Vec::with_capacity(16);
    format.extend_from_slice(&1_u16.to_le_bytes());
    format.extend_from_slice(&38_u16.to_le_bytes());
    format.extend_from_slice(&48_000_u32.to_le_bytes());
    format.extend_from_slice(&5_472_000_u32.to_le_bytes());
    format.extend_from_slice(&114_u16.to_le_bytes());
    format.extend_from_slice(&24_u16.to_le_bytes());
    format
}

fn append_chunk(bytes: &mut Vec<u8>, chunk_id: [u8; 4], payload: &[u8]) -> Range<usize> {
    bytes.extend_from_slice(&chunk_id);
    bytes.extend_from_slice(&u32::try_from(payload.len()).unwrap().to_le_bytes());
    let start = bytes.len();
    bytes.extend_from_slice(payload);
    let end = bytes.len();
    if payload.len() % 2 != 0 {
        bytes.push(0);
    }
    start..end
}

fn metadata_payloads<'a>(
    resource: &'a [u8],
    spans: &'a [Range<usize>; 3],
) -> SelectedAdmResourcePayloads<'a> {
    SelectedAdmResourcePayloads::new(
        &resource[spans[0].clone()],
        &resource[spans[1].clone()],
        &resource[spans[2].clone()],
    )
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
