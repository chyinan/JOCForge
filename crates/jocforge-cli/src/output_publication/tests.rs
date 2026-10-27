use super::ProgrammeOutputStaging;
use jocforge_application::{
    PreparedJobInput, SelectedAdmResourcePayloads,
    output_publication::{AttemptId, OutputRootBindingError, OutputStagingOwnerId},
    prepare_selected_job_input,
};
use jocforge_application::{ProgrammeId, SelectedWaveDescriptor};
use serde_json::Value;
use std::{
    fs,
    ops::Range,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

const AXML: &[u8] = include_bytes!(
    "../../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.xml"
);
const CHNA: &[u8] = include_bytes!(
    "../../../jocforge-domain/tests/fixtures/selected_adm_graph/source_track_associations.bin"
);
const DBMD: &[u8] = include_bytes!(
    "../../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.dbmd"
);
const JOB_VECTORS: &[u8] = include_bytes!(
    "../../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
);

struct ScratchDirectory(PathBuf);

impl ScratchDirectory {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jocforge-output-binding-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create isolated output binding directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScratchDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn prepared_job(programme_label: &str) -> PreparedJobInput {
    let vectors: Value = serde_json::from_slice(JOB_VECTORS).expect("job vectors");
    let case = &vectors["config_cases"][0];
    let descriptor = &case["pcm_descriptor"];
    let wave = SelectedWaveDescriptor::new(
        descriptor["sample_rate"].as_u64().unwrap() as u32,
        descriptor["channels"].as_u64().unwrap() as u16,
        descriptor["pcm_bits"].as_u64().unwrap() as u16,
    );
    let (resource, spans) = selected_resource();
    prepare_selected_job_input(
        ProgrammeId::new(programme_label).expect("programme id"),
        case["input_xml"].as_str().unwrap().as_bytes(),
        wave,
        descriptor["source_samples"].as_u64().unwrap(),
        Arc::clone(&resource),
        metadata_payloads(&resource, &spans),
    )
    .expect("selected input fixture is admissible")
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

fn append_chunk(bytes: &mut Vec<u8>, id: [u8; 4], payload: &[u8]) -> Range<usize> {
    bytes.extend_from_slice(&id);
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

#[test]
fn staging_preflight_accepts_the_same_binding_and_rejects_cross_programme_or_stale_attempt() {
    let prepared_a = prepared_job("programme-A");
    let prepared_b = prepared_job("programme-B");
    let binding_a =
        jocforge_application::output_publication::test_support::bind_selected_output_for_test(
            &prepared_a,
            AttemptId::new(0),
            OutputStagingOwnerId::new("opaque-programme-A-attempt-0").unwrap(),
        )
        .unwrap();
    let binding_b =
        jocforge_application::output_publication::test_support::bind_selected_output_for_test(
            &prepared_b,
            AttemptId::new(0),
            OutputStagingOwnerId::new("opaque-programme-B-attempt-0").unwrap(),
        )
        .unwrap();
    let restarted_a =
        jocforge_application::output_publication::test_support::bind_selected_output_for_test(
            &prepared_a,
            AttemptId::new(1),
            OutputStagingOwnerId::new("opaque-programme-A-attempt-1").unwrap(),
        )
        .unwrap();

    let directory = ScratchDirectory::new();
    let staging = ProgrammeOutputStaging::new(directory.path(), &binding_a);
    let staging_path = staging.staging_path();
    assert!(!staging_path.exists());

    assert!(staging.validate_binding(&binding_a).is_ok());
    assert_eq!(
        staging.validate_binding(&binding_b),
        Err(OutputRootBindingError::ProgrammeIdentityMismatch)
    );
    assert_eq!(
        staging.validate_binding(&restarted_a),
        Err(OutputRootBindingError::AttemptIdentityMismatch)
    );
    assert!(
        !staging_path.exists(),
        "rejected cross-root preflight must not create or write the staging file"
    );
    assert!(!staging.is_abandoned());
}
