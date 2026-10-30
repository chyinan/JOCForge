// pattern: Functional Core
//! Synthetic callback-join coverage only; this is not audio/DSP conformance evidence.

use jocforge::meter_source_bridge::{
    MeterSourceCoordinatorError, prepare_selected_adm_bwf_input, ready_to_finalize,
    reset_meter_source_and_prepass, stage_meter_source_oar_callback, view_for_source_proposal,
};
use jocforge_application::{
    PreparedJobInput, SelectedAdmResourcePayloads,
    meter_source_bridge::{
        AdmFileStorageTrackSlot, IndependentMeterPrepass, KMatrixMode, KMatrixProfile,
        MeterCallbackCount, MeterCallbackIndex, MeterCallbackPhase, MeterFrontendCallbackInput,
        MeterSampleCount, MeterSourceBridgeError, MeterSourceCallbackFrame,
        MeterSourceCallbackFrameParts, MeterSourceCallbackView, MeterSourceIdentity,
        MeterSourcePosition, MeterSourceReadIndex, MeterSourceSampleCount, MeterUpstreamGeneration,
        ObjectRenderCallbackGeneration, ProgrammeId, SelectedTrackKind, SelectedWaveDescriptor,
        SharedGainDelayParameters, generate_k_matrix,
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
fn selected_adm_riff_preparation_binds_metadata_and_pcm_to_one_resource() {
    let (resource, _) = selected_resource();
    let parsed = parse_selected_riff_pcm24(&resource, None).unwrap();
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let job_xml = vectors["config_cases"][0]["input_xml"]
        .as_str()
        .unwrap()
        .as_bytes();

    let prepared = prepare_selected_adm_bwf_input(
        ProgrammeId::new("selected-adm-resource-binding").unwrap(),
        job_xml,
        Arc::clone(&resource),
        &parsed,
    )
    .unwrap();

    assert_eq!(prepared.configuration().source_samples(), 1);
    assert_eq!(
        prepared.resource_identity().resource_bytes(),
        u64::try_from(resource.len()).unwrap()
    );
    assert_eq!(
        prepared.resource_identity().input_sha256(),
        parsed.source_sha256()
    );
    assert_eq!(prepared.graph().graph().tracks().len(), 38);
    assert_eq!(
        prepared.primary_control_binding().unwrap().data_bytes(),
        114
    );
    assert_eq!(
        prepared.primary_metadata_controls().unwrap().rows().len(),
        38
    );

    let foreign_resource: Arc<[u8]> = Arc::from(resource.as_ref().to_vec().into_boxed_slice());
    let foreign_parsed = parse_selected_riff_pcm24(&foreign_resource, None).unwrap();
    assert_eq!(
        prepare_selected_adm_bwf_input(
            ProgrammeId::new("foreign-adm-resource-binding").unwrap(),
            job_xml,
            Arc::clone(&resource),
            &foreign_parsed,
        )
        .unwrap_err(),
        jocforge::meter_source_bridge::SelectedAdmPreparationError::PreparedJobInput(
            jocforge_application::PreparedJobInputError::MetadataPayloadBinding
        )
    );
}

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
        let expected_read_index_before = u64::from(callback != 0);
        assert_eq!(
            source_view.source_read_index_before().value(),
            expected_read_index_before
        );
        assert_eq!(source_view.source_read_index_after().value(), 1);

        let input = frontend_input(&source_view, callback);
        let handoff = source_view.bind_frontend_input(input).unwrap();
        let (meter_receipt, source_receipt) =
            stage_meter_source_oar_callback(&mut cursor, &mut prepass, proposal, handoff)
                .unwrap()
                .commit();
        assert_eq!(meter_receipt.callback_index().value(), callback);
        assert_eq!(source_receipt.callback_index().value(), callback);
        assert_eq!(source_receipt.upstream_generation().value(), callback + 1);
        assert_eq!(
            source_receipt.source_read_index_before().value(),
            expected_read_index_before
        );
        assert_eq!(source_receipt.source_read_index_after().value(), 1);
        assert_eq!(cursor.callback_index().value(), callback + 1);
        assert_eq!(cursor.read_index().value(), 1);
    }

    assert_eq!(cursor.source_cursor().value(), parsed.total_samples());
    assert!(ready_to_finalize(&cursor, &prepass));

    reset_meter_source_and_prepass(&mut cursor, &mut prepass, &prepared, &parsed).unwrap();
    assert_eq!(cursor.callback_index().value(), 0);
    assert_eq!(cursor.source_cursor().value(), 0);
    assert_eq!(cursor.read_index().value(), 0);
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
fn source_frame_extent_must_match_the_prepared_adm_job() {
    let prepared = prepared_job_input();
    let rows = vec![vec![0; 1_536]; 38];
    let frame = callback_frame(&prepared, &rows, 2);

    assert_eq!(
        MeterSourceCallbackView::new(&prepared, frame).err(),
        Some(MeterSourceBridgeError::SourceExtentMismatch)
    );
}

#[test]
fn meter_oar_bridge_rejects_non_adm_input_kind() {
    let prepared = prepared_job_input_for_case(1);
    let rows = vec![vec![0; 1_536]; 38];
    let frame = callback_frame(&prepared, &rows, prepared.configuration().source_samples());

    assert_eq!(
        MeterSourceCallbackView::new(&prepared, frame).err(),
        Some(MeterSourceBridgeError::SelectedInputKindMismatch)
    );
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

#[test]
fn pcm_rows_bind_to_chna_track_and_slot_map_in_physical_order() {
    let (resource, spans) = selected_resource_with_row_markers();
    let prepared = prepared_job_input_from_resource(0, &resource, &spans);
    let parsed = parse_selected_riff_pcm24(&resource, None).unwrap();
    let identity = MeterPcmSourceIdentity::from_selected_resource_sha256(
        prepared.resource_identity().input_sha256(),
    );
    let source_samples = MeterSourceSampleCount::new(parsed.total_samples()).unwrap();
    let cursor = MeterPcmSourceCursor::cold(&parsed, identity, source_samples).unwrap();
    let proposal = stage_meter_pcm_source_frame(&cursor, cursor.callback_index(), &parsed).unwrap();
    let source_view = view_for_source_proposal(&prepared, &proposal).unwrap();
    let row_map = source_view.physical_row_map();
    let graph = prepared.graph().graph();
    let converter_map = prepared.converter_storage_map().unwrap();

    assert_eq!(row_map.resource_identity(), prepared.resource_identity());
    assert_eq!(row_map.axml_sha256(), graph.receipt().axml_sha256());
    assert_eq!(row_map.chna_sha256(), graph.receipt().chna_sha256());
    assert_eq!(row_map.dbmd_sha256(), prepared.dbmd().payload_sha256());
    assert_eq!(row_map.entries_by_pcm_row().len(), 38);

    for binding in row_map.entries_by_pcm_row() {
        let row = usize::from(binding.pcm_row().value());
        let row_value = u8::try_from(row).unwrap();
        let slot = AdmFileStorageTrackSlot::new(row_value).unwrap();
        let converter_entry = converter_map.channels().entry_for_slot(slot);
        let graph_track = graph
            .tracks()
            .iter()
            .find(|track| track.track_index() == binding.track_index())
            .unwrap();

        assert_eq!(binding.track_index().value(), u16::from(row_value) + 1);
        assert_eq!(binding.physical_host().value(), row_value);
        assert_eq!(converter_entry.track_slot(), slot);
        assert_eq!(
            binding.channel_input_id(),
            converter_entry.channel_input_id()
        );
        assert_eq!(
            binding.dbmd_track_index().value(),
            u16::from(binding.physical_host().value())
        );
        match graph_track.kind() {
            SelectedTrackKind::DirectSpeakersBed { bed_input_id, .. } => {
                assert_eq!(binding.channel_input_id().value(), bed_input_id);
            }
            SelectedTrackKind::Object { semantic_key } => {
                assert_eq!(binding.channel_input_id().value(), semantic_key);
            }
        }

        let expected_word = ((f32::from(row_value) + 1.0) / 8_388_608.0).to_bits();
        assert_eq!(
            source_view.physical_rows()[row][0],
            expected_word,
            "physical PCM row {row} must keep its input marker"
        );
    }
}

#[test]
fn pcm_cursor_from_a_different_prepared_resource_is_rejected() {
    let (prepared_resource, prepared_spans) = selected_resource_with_row_markers();
    let prepared = prepared_job_input_from_resource(0, &prepared_resource, &prepared_spans);
    let (foreign_resource, _) = selected_resource();
    let parsed = parse_selected_riff_pcm24(&foreign_resource, None).unwrap();
    let foreign_identity =
        MeterPcmSourceIdentity::from_selected_resource_sha256(parsed.source_sha256());
    let source_samples = MeterSourceSampleCount::new(parsed.total_samples()).unwrap();
    let cursor = MeterPcmSourceCursor::cold(&parsed, foreign_identity, source_samples).unwrap();
    let proposal = stage_meter_pcm_source_frame(&cursor, cursor.callback_index(), &parsed).unwrap();

    assert_eq!(
        view_for_source_proposal(&prepared, &proposal).err(),
        Some(MeterSourceCoordinatorError::Bridge(
            MeterSourceBridgeError::ResourceIdentityMismatch
        ))
    );
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
    prepared_job_input_for_case(0)
}

fn prepared_job_input_for_case(case_index: usize) -> PreparedJobInput {
    let (resource, spans) = selected_resource();
    prepared_job_input_from_resource(case_index, &resource, &spans)
}

fn prepared_job_input_from_resource(
    case_index: usize,
    resource: &Arc<[u8]>,
    spans: &[Range<usize>; 3],
) -> PreparedJobInput {
    let vectors: Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let case = &vectors["config_cases"][case_index];
    let descriptor = &case["pcm_descriptor"];
    let source_samples = parse_selected_riff_pcm24(resource, None)
        .unwrap()
        .total_samples();
    prepare_selected_job_input(
        ProgrammeId::new("JOCForgeTestID01").unwrap(),
        case["input_xml"].as_str().unwrap().as_bytes(),
        SelectedWaveDescriptor::new(
            descriptor["sample_rate"].as_u64().unwrap() as u32,
            descriptor["channels"].as_u64().unwrap() as u16,
            descriptor["pcm_bits"].as_u64().unwrap() as u16,
        ),
        source_samples,
        Arc::clone(resource),
        metadata_payloads(resource, spans),
    )
    .unwrap()
}

fn callback_frame<'a>(
    prepared: &PreparedJobInput,
    rows: &'a [Vec<u32>],
    source_samples: u64,
) -> MeterSourceCallbackFrame<'a> {
    callback_frame_with_read_indices(prepared, rows, source_samples, 0, 1)
}

fn callback_frame_with_read_indices<'a>(
    prepared: &PreparedJobInput,
    rows: &'a [Vec<u32>],
    source_samples: u64,
    source_read_index_before: u64,
    source_read_index_after: u64,
) -> MeterSourceCallbackFrame<'a> {
    let normal_callbacks = source_samples.div_ceil(1_536);
    let host_input_count = source_samples.min(1_536);
    MeterSourceCallbackFrame::new(MeterSourceCallbackFrameParts {
        callback_index: MeterCallbackIndex::new(0),
        upstream_generation: MeterUpstreamGeneration::new(1),
        phase: MeterCallbackPhase::NormalSource,
        source_identity: MeterSourceIdentity::from_sha256(
            prepared.resource_identity().input_sha256(),
        ),
        source_samples: MeterSourceSampleCount::new(source_samples).unwrap(),
        normal_callbacks: MeterCallbackCount::new(normal_callbacks),
        source_cursor_before: MeterSourcePosition::new(0),
        source_cursor_after: MeterSourcePosition::new(host_input_count),
        source_read_index_before: MeterSourceReadIndex::new(source_read_index_before),
        source_read_index_after: MeterSourceReadIndex::new(source_read_index_after),
        host_input_count: MeterSampleCount::new(host_input_count),
        source_padding_count: MeterSampleCount::new(1_536 - host_input_count),
        rows,
    })
}

#[test]
fn application_source_boundary_rejects_a_normal_read_without_read_index_advance() {
    let prepared = prepared_job_input();
    let rows = vec![vec![0; 1_536]; 38];
    let frame = callback_frame_with_read_indices(
        &prepared,
        &rows,
        prepared.configuration().source_samples(),
        0,
        0,
    );

    assert!(matches!(
        MeterSourceCallbackView::new(&prepared, frame),
        Err(MeterSourceBridgeError::SourceClock)
    ));
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

fn selected_resource_with_row_markers() -> (Arc<[u8]>, [Range<usize>; 3]) {
    let (resource, spans) = selected_resource();
    let parsed = parse_selected_riff_pcm24(&resource, None).unwrap();
    let data_offset = usize::try_from(parsed.data_offset()).unwrap();
    let mut bytes = resource.as_ref().to_vec();
    for row in 0..38 {
        let marker = i32::try_from(row + 1).unwrap().to_le_bytes();
        let start = data_offset + row * 3;
        bytes[start..start + 3].copy_from_slice(&marker[..3]);
    }
    (Arc::from(bytes.into_boxed_slice()), spans)
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
