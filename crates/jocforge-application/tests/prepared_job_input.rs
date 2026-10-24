// pattern: Functional Core
use jocforge_application::{PreparedJobInputError, prepare_selected_job_input};
use jocforge_domain::frontend::{
    job_configuration::SelectedWaveDescriptor, selected_adm_graph::parse_selected_adm_graph,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::Arc;

#[test]
fn value_preparation_preserves_admission_and_binds_immutable_caller_bytes() {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    // Adapter validation is outside this value-level test. These bytes exercise
    // resource custody and hashing only, not RIFF parsing or audio admission.
    let resource: Arc<[u8]> = Arc::from([7_u8, 19, 31]);
    let expected_digest: [u8; 32] = Sha256::digest(resource.as_ref()).into();
    let graph = parse_selected_adm_graph(
        include_bytes!(
            "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.xml"
        ),
        include_bytes!(
            "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_track_associations.bin"
        ),
    )
    .unwrap();
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
            case["input_xml"].as_str().unwrap().as_bytes(),
            wave,
            descriptor["source_samples"].as_u64().unwrap(),
            Arc::clone(&resource),
            graph.clone(),
        );
        if case["expected"]["status"] == "ACCEPT" {
            let prepared = result.unwrap();
            assert_eq!(prepared.resource_sha256(), expected_digest);
            assert_eq!(prepared.resource(), resource.as_ref());
            assert_eq!(prepared.resource().as_ptr(), resource.as_ptr());
            assert_eq!(
                prepared.configuration().source_samples(),
                descriptor["source_samples"].as_u64().unwrap()
            );
            assert_eq!(prepared.admitted_object_events().events().len(), 28);
            assert_eq!(prepared.graph().receipt().physical_track_count(), 38);
            accepted += 1;
        } else {
            assert!(result.is_err());
            rejected += 1;
        }
    }
    assert_eq!((accepted, rejected), (3, 8));

    let accepted_case = &vectors["config_cases"][0];
    let error = prepare_selected_job_input(
        accepted_case["input_xml"].as_str().unwrap().as_bytes(),
        SelectedWaveDescriptor::new(48_000, 38, 24),
        13_888_072,
        Arc::clone(&resource),
        graph,
    )
    .unwrap_err();
    assert!(matches!(error, PreparedJobInputError::GraphAdmission(_)));
    assert!(jocforge_application::encode().is_err());
}
