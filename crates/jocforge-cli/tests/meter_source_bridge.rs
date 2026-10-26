// pattern: Functional Core
//! Synthetic callback-join coverage only; this is not audio/DSP conformance evidence.

use jocforge::meter_source_bridge::{
    MeterSourceCoordinatorError, ready_to_finalize, reset_meter_source_and_prepass,
    stage_meter_source_oar_callback, view_for_source_proposal,
};
use jocforge_application::{
    PreparedJobInput, SelectedAdmResourcePayloads,
    meter_source_bridge::{
        IndependentMeterPrepass, KMatrixMode, KMatrixProfile, MeterCallbackIndex,
        MeterCallbackPhase, MeterFrontendCallbackInput, MeterSourceBridgeError,
        MeterSourceCallbackView, MeterSourceSampleCount, ObjectRenderCallbackGeneration,
        ProgrammeId, SelectedWaveDescriptor, SharedGainDelayParameters, generate_k_matrix,
    },
    prepare_selected_job_input,
};
use jocforge_io::pcm::{
    meter_source_cursor::{
        MeterPcmSourceCursor, MeterPcmSourceIdentity, stage_meter_pcm_source_frame,
    },
    riff_pcm24::parse_selected_riff_pcm24,
};
use serde_json::Value;
use std::{ops::Range, sync::Arc};

const AXML: &[u8] =
    include_bytes!("../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.xml");
const CHNA: &[u8] = include_bytes!(
    "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_track_associations.bin"
);
const DBMD: &[u8] =
    include_bytes!("../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.dbmd");
const ZERO_METER_FIXTURE: &[u8] =
    include_bytes!("../../jocforge-domain/tests/fixtures/post_render_meter/zero_programme.json");

#[test]
fn selected_pcm_callback_joins_direct_lfe_and_oar_clock_through_three_drains() {
    let prepared = prepared_job_input();
    let (resource, _) = selected_resource();
    let parsed = parse_selected_riff_pcm24(&resource, None).unwrap();
    assert_eq!(parsed.total_samples(), 1);

    let identity = MeterPcmSourceIdentity::from_selected_resource_sha256(
        prepared.resource_identity().input_sha256(),
    );
    let sample_count = MeterSourceSampleCount::new(parsed.total_samples()).unwrap();
    let mut cursor = MeterPcmSourceCursor::cold(&parsed, identity, sample_count).unwrap();
    let mut prepass = prepass(sample_count);

    for callback in 0..4 {
        let proposal =
            stage_meter_pcm_source_frame(&cursor, cursor.callback_index(), &parsed).unwrap();
        let source_view = view_for_source_proposal(&prepared, &proposal).unwrap();
        assert_eq!(source_view.callback_index().value(), callback);
        assert_eq!(source_view.upstream_generation().value(), callback + 1);
        assert_eq!(
            source_view.phase(),
            if callback == 0 {
                MeterCallbackPhase::NormalSource
            } else {
                MeterCallbackPhase::ZeroInputDrain
            }
        );
        assert_eq!(source_view.physical_rows().len(), 38);
        assert_eq!(source_view.physical_rows()[0].len(), 1_536);
        assert_eq!(
            source_view.host_input_count().value(),
            u64::from(callback == 0)
        );
        assert_eq!(
            source_view.source_padding_count().value(),
            if callback == 0 { 1_535 } else { 1_536 }
        );

        let input = frontend_input(&source_view, callback);
        let handoff = source_view.bind_frontend_input(input).unwrap();
        let (meter_receipt, source_receipt) =
            stage_meter_source_oar_callback(&mut cursor, &mut prepass, proposal, handoff)
                .unwrap()
                .commit();
        assert_eq!(meter_receipt.callback_index().value(), callback);
        assert_eq!(source_receipt.callback_index().value(), callback);
        assert_eq!(source_receipt.upstream_generation().value(), callback + 1);
        assert_eq!(cursor.callback_index().value(), callback + 1);
    }

    assert_eq!(cursor.source_cursor().value(), parsed.total_samples());
    assert!(ready_to_finalize(&cursor, &prepass));

    reset_meter_source_and_prepass(&mut cursor, &mut prepass, &prepared, &parsed).unwrap();
    assert_eq!(cursor.callback_index().value(), 0);
    assert_eq!(cursor.source_cursor().value(), 0);
    assert_eq!(prepass.next_callback_index().value(), 0);
}

#[test]
fn frontend_handoff_rejects_wrong_clock_and_direct_lfe_row() {
    let prepared = prepared_job_input();
    let (resource, _) = selected_resource();
    let parsed = parse_selected_riff_pcm24(&resource, None).unwrap();
    let identity = MeterPcmSourceIdentity::from_selected_resource_sha256(
        prepared.resource_identity().input_sha256(),
    );
    let sample_count = MeterSourceSampleCount::new(parsed.total_samples()).unwrap();
    let cursor = MeterPcmSourceCursor::cold(&parsed, identity, sample_count).unwrap();

    let proposal = stage_meter_pcm_source_frame(&cursor, cursor.callback_index(), &parsed).unwrap();
    let source_view = view_for_source_proposal(&prepared, &proposal).unwrap();
    let wrong_clock = MeterFrontendCallbackInput::new(
        MeterCallbackIndex::new(1),
        ObjectRenderCallbackGeneration::try_new(2).unwrap(),
        MeterCallbackPhase::NormalSource,
        source_view.direct_lfe_words().to_vec(),
        vec![vec![0; 1_536]; 15],
        object_controls(),
    );
    assert_eq!(
        source_view.bind_frontend_input(wrong_clock).err(),
        Some(MeterSourceBridgeError::FrontendClock)
    );
    drop(proposal);

    let proposal = stage_meter_pcm_source_frame(&cursor, cursor.callback_index(), &parsed).unwrap();
    let source_view = view_for_source_proposal(&prepared, &proposal).unwrap();
    let wrong_lfe = MeterFrontendCallbackInput::new(
        MeterCallbackIndex::new(0),
        ObjectRenderCallbackGeneration::try_new(1).unwrap(),
        MeterCallbackPhase::NormalSource,
        vec![1; 1_536],
        vec![vec![0; 1_536]; 15],
        object_controls(),
    );
    assert_eq!(
        source_view.bind_frontend_input(wrong_lfe).err(),
        Some(MeterSourceBridgeError::DirectLfeMismatch)
    );
    drop(proposal);
    assert_eq!(cursor.callback_index().value(), 0);
}

#[test]
fn source_and_prepass_extents_must_match_before_prepass_staging() {
    let prepared = prepared_job_input();
    let (resource, _) = selected_resource();
    let parsed = parse_selected_riff_pcm24(&resource, None).unwrap();
    let identity = MeterPcmSourceIdentity::from_selected_resource_sha256(
        prepared.resource_identity().input_sha256(),
    );
    let source_count = MeterSourceSampleCount::new(parsed.total_samples()).unwrap();
    let mut cursor = MeterPcmSourceCursor::cold(&parsed, identity, source_count).unwrap();
    let mut prepass = prepass(MeterSourceSampleCount::new(1_536).unwrap());

    let proposal = stage_meter_pcm_source_frame(&cursor, cursor.callback_index(), &parsed).unwrap();
    let source_view = view_for_source_proposal(&prepared, &proposal).unwrap();
    let input = frontend_input(&source_view, 0);
    let handoff = source_view.bind_frontend_input(input).unwrap();
    assert_eq!(
        stage_meter_source_oar_callback(&mut cursor, &mut prepass, proposal, handoff).err(),
        Some(MeterSourceCoordinatorError::PrepassClockMismatch)
    );
    assert_eq!(cursor.callback_index().value(), 0);
    assert_eq!(prepass.next_callback_index().value(), 0);
}

fn frontend_input(
    source: &MeterSourceCallbackView<'_>,
    callback: u64,
) -> MeterFrontendCallbackInput {
    MeterFrontendCallbackInput::new(
        source.callback_index(),
        ObjectRenderCallbackGeneration::try_new(callback + 1).unwrap(),
        source.phase(),
        source.direct_lfe_words().to_vec(),
        vec![vec![0; 1_536]; 15],
        object_controls(),
    )
}

fn object_controls() -> Vec<Vec<u32>> {
    let fixture: Value = serde_json::from_slice(ZERO_METER_FIXTURE).unwrap();
    let words = fixture["oar_packet"]["object_control_words"]
        .as_array()
        .unwrap()
        .iter()
        .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>();
    vec![words; 15]
}

fn prepass(sample_count: MeterSourceSampleCount) -> IndependentMeterPrepass {
    const SELECTED_KERNEL_Q15: [u16; 32] = [
        20, 177, 491, 958, 1_573, 2_331, 3_224, 4_244, 5_381, 6_624, 7_961, 9_379, 10_864, 12_403,
        13_980, 15_580, 17_188, 18_788, 20_365, 21_904, 23_389, 24_807, 26_144, 27_387, 28_524,
        29_544, 30_437, 31_195, 31_810, 32_277, 32_591, 32_748,
    ];
    let gain = SharedGainDelayParameters::selected(&SELECTED_KERNEL_Q15).unwrap();
    let matrix = generate_k_matrix(KMatrixProfile {
        sample_rate_hz: 48_000,
        mode: KMatrixMode::Modern,
        stage_count: 2,
    })
    .unwrap();
    IndependentMeterPrepass::cold(sample_count, gain, matrix).unwrap()
}

fn prepared_job_input() -> PreparedJobInput {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = &vectors["config_cases"][0];
    let descriptor = &case["pcm_descriptor"];
    let (resource, spans) = selected_resource();
    prepare_selected_job_input(
        ProgrammeId::new("JOCForgeTestID01").unwrap(),
        case["input_xml"].as_str().unwrap().as_bytes(),
        SelectedWaveDescriptor::new(
            descriptor["sample_rate"].as_u64().unwrap() as u32,
            descriptor["channels"].as_u64().unwrap() as u16,
            descriptor["pcm_bits"].as_u64().unwrap() as u16,
        ),
        descriptor["source_samples"].as_u64().unwrap(),
        Arc::clone(&resource),
        metadata_payloads(&resource, &spans),
    )
    .unwrap()
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
