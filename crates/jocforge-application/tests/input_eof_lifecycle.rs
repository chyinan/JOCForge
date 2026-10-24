// pattern: Imperative Shell
#[path = "support/input_eof_lifecycle_replay.rs"]
mod input_eof_lifecycle_replay;

use input_eof_lifecycle_replay::{
    ActiveFlushInput, AllocationRetirement, ExternalControlPosition, ExternalProcessInput,
    ExternalSourceIndex, FrontendSourceInput, InputEofLifecycle, OwnerCloseOrder,
    PostRetirementRead, ReaderCallIdentity, ReaderCursorObservation, ReaderSampleCounts,
    ReaderSourcePositionObservation, RelativeReaderCursorSamples, RendererSpanSamples,
    SelectedItemSampleCoordinate, SelectedItemSampleRange, SemanticQueueStatus,
    SourceControlRecord,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const FIXTURE_DIRECTORY: &str = "tests/fixtures/input_eof_lifecycle";
const READER_SOURCE_DIRECTORY: &str = "tests/fixtures/reader_source_join";
const HANDOFF_MANIFEST_SHA256: &str =
    "ef7fa983440fd11784fe9f24f64284240f00fd053853dc1cc28067d3b1cf0d42";
const HANDOFF_CHECK_SHA256: &str =
    "70c91dcf7c42ae0c57184af165d60df65de49482445bb22e1e9584011174ad51";
const SOURCE_MANIFEST_SHA256: &str =
    "139c118dd722fc1fe4e65bc23c82bc448fa80a00c4d9731a638582e9879b23e7";
const BASE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const CARRIER_HANDOFF_MANIFEST_SHA256: &str =
    "9c136d43219ccd8fc5623066864d27959f079c9cfe9dfcd10a5726e7fb20f049";
const BSI_HANDOFF_MANIFEST_SHA256: &str =
    "70f90cd5472299f66bba22c362e87fe2645b6f480d71dfeb763fab2eb769fe00";
const CARRIER_JOINS_SHA256: &str =
    "cebe647028b976ac3de9d23fb92cc946922ce8a0b1c2b5a79bae1fd135d41d65";
const READER_HANDOFF_MANIFEST_SHA256: &str =
    "7f6adb938741b61ac15c7aeca1dac759504e291af302ab3d460bd41fc64aa7b6";
const READER_HANDOFF_CHECK_SHA256: &str =
    "45f871629f970e49a31e29cfd31b086615c39ab9fbafc472abe601eaec35d026";
const READER_SOURCE_MANIFEST_SHA256: &str =
    "2276298e21d58276650062466941455f5ebadd0be4542ecbfae25588a71d1899";

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURE_DIRECTORY)
        .join(name)
}

fn fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(fixture_path(name)).unwrap()
}

fn fixture_json(name: &str) -> Value {
    serde_json::from_slice(&fixture_bytes(name)).unwrap()
}

fn reader_source_fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(READER_SOURCE_DIRECTORY)
            .join(name),
    )
    .unwrap()
}

fn reader_source_fixture_json(name: &str) -> Value {
    serde_json::from_slice(&reader_source_fixture_bytes(name)).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn assert_handoff_payload(manifest: &Value, name: &str) {
    let bytes = fixture_bytes(name);
    let entry = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == name)
        .unwrap();
    assert_eq!(entry["bytes"].as_u64().unwrap(), bytes.len() as u64);
    assert_eq!(entry["sha256"], sha256(&bytes), "{name}");
}

fn assert_reader_handoff_payload(manifest: &Value, name: &str) {
    let bytes = reader_source_fixture_bytes(name);
    let entry = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == name)
        .unwrap();
    assert_eq!(entry["bytes"].as_u64().unwrap(), bytes.len() as u64);
    assert_eq!(entry["sha256"], sha256(&bytes), "{name}");
}

fn source_records(position: u64, source_indices: &[Value]) -> [SourceControlRecord; 38] {
    assert_eq!(source_indices.len(), 38);
    std::array::from_fn(|index| {
        SourceControlRecord::new(
            ExternalSourceIndex::new(source_indices[index].as_u64().unwrap() as u16),
            ExternalControlPosition::new(position),
        )
    })
}

fn observation_for_work_origin<'a>(observations: &'a [Value], kind: &str, call: u64) -> &'a Value {
    observations
        .iter()
        .find(|observation| {
            observation["event"] == "renderer_output_span"
                && observation["origin"]["kind"] == kind
                && observation["origin"]["call"].as_u64() == Some(call)
        })
        .unwrap()
}

fn frontend_origin_for_work<'a>(
    observations: &'a [Value],
    kind: &str,
    call: u64,
) -> Option<&'a Value> {
    observations.iter().find(|observation| {
        observation["event"] == "frontend_input_origin"
            && observation["origin"]["kind"] == kind
            && observation["origin"]["call"].as_u64() == Some(call)
    })
}

fn source_step_input(inputs: &Value, frontend_invocation: u64) -> FrontendSourceInput {
    let index = usize::try_from(frontend_invocation - 1).unwrap();
    let planner_request = inputs["native_boundary_planner_requests"][index]
        .as_i64()
        .unwrap() as i32;
    let pending_bytes = inputs["native_boundary_id14_body_lengths"][index]
        .as_u64()
        .unwrap() as u16;
    FrontendSourceInput::new(planner_request, pending_bytes)
}

fn assert_snapshot(
    actual: input_eof_lifecycle_replay::InputEofLifecycleSnapshot,
    expected: &Value,
) {
    assert_eq!(
        actual.gate_progress_samples(),
        expected["gate_progress"].as_u64().unwrap() as u32
    );
    assert_eq!(
        actual.flush_remaining(),
        expected["flush_remaining"].as_u64().unwrap() as u32
    );
    assert_eq!(
        u32::from(actual.flush_started()),
        expected["flush_started"].as_u64().unwrap() as u32
    );
    assert_eq!(
        actual.planner_counter(),
        expected["planner_counter"].as_u64().unwrap() as u32
    );
    assert_eq!(
        u32::from(actual.planner_enabled()),
        expected["planner_enabled"].as_u64().unwrap() as u32
    );
    assert_eq!(
        u64::from(actual.planner_first_block()),
        expected["planner_first_block"].as_u64().unwrap()
    );
    let expected_segments = expected["planner_segments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_u64().unwrap() as u16)
        .collect::<Vec<_>>();
    assert_eq!(actual.planner_segments().as_slice(), expected_segments);
    assert_eq!(
        actual.last_prepared_reservation_bytes(),
        expected["last_prepared_reservation_bytes"]
            .as_i64()
            .unwrap() as i32
    );
    let expected_pending = expected["pending_id14_payload_bytes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_u64().unwrap() as u16)
        .collect::<Vec<_>>();
    assert_eq!(
        actual
            .pending_id14_payload_bytes()
            .map_or_else(Vec::new, |value| vec![value]),
        expected_pending
    );
    assert_eq!(actual.queue_occupied_count(), expected_pending.len() as u32);
    assert_eq!(
        actual.runtime_present(),
        expected["joc_runtime_present"].as_bool().unwrap()
    );
}

#[test]
fn lifecycle_handoff_payloads_and_cross_package_identities_are_hash_bound() {
    let manifest_bytes = fixture_bytes("HANDOFF-MANIFEST.json");
    let check_bytes = fixture_bytes("HANDOFF-CHECK.json");
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    let check: Value = serde_json::from_slice(&check_bytes).unwrap();

    assert_eq!(sha256(&manifest_bytes), HANDOFF_MANIFEST_SHA256);
    assert_eq!(sha256(&check_bytes), HANDOFF_CHECK_SHA256);
    assert_eq!(manifest["files"].as_array().unwrap().len(), 15);
    assert_eq!(
        manifest["source_package_manifest_sha256"],
        SOURCE_MANIFEST_SHA256
    );
    assert_eq!(
        manifest["base_package_manifest_sha256"],
        BASE_MANIFEST_SHA256
    );
    assert_eq!(check["handoff_manifest_sha256"], HANDOFF_MANIFEST_SHA256);
    assert_eq!(check["payload_hashes_match"], true);
    assert_eq!(check["whole_F13C_ready"], false);
    assert_eq!(check["implementation_ready"], false);

    for name in [
        "INPUTS.json",
        "EXPECTED.json",
        "CONTRACT.json",
        "EOF-ADDENDUM.md",
        "NATIVE-OBSERVATIONS.json",
        "CURRENT-OVERLAY.json",
        "INDEPENDENT-CHECK.json",
        "WASH-INDEPENDENT-CHECK.json",
    ] {
        assert_handoff_payload(&manifest, name);
    }

    let provenance = fixture_json("PROVENANCE.json");
    assert_eq!(
        provenance["handoff_manifest_sha256"],
        HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(
        provenance["related_bsi_handoff_manifest_sha256"],
        BSI_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(
        provenance["related_carrier_handoff_manifest_sha256"],
        CARRIER_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(
        sha256(&fixture_bytes("CARRIER-NATIVE-AU-JOINS.json")),
        CARRIER_JOINS_SHA256
    );

    let overlay = fixture_json("CURRENT-OVERLAY.json");
    assert_eq!(
        overlay["related_bsi_handoff_manifest_sha256"],
        BSI_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(
        overlay["related_carrier_handoff_manifest_sha256"],
        CARRIER_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(overlay["absolute_ADM_WAV_offset"], "NOT_BOUND");
    assert_eq!(overlay["same_owner_reset"], "NOT_RUN");
    assert_eq!(overlay["whole_F13C_ready"], "NO");

    let independent = fixture_json("INDEPENDENT-CHECK.json");
    assert_eq!(independent["replay"]["external_process_calls"], 4);
    assert_eq!(independent["replay"]["flush_calls"], 5);
    assert_eq!(independent["replay"]["active_flush_calls"], 4);
    assert_eq!(independent["replay"]["published_aus"], 4);
    assert_eq!(independent["replay"]["mismatch_count"], 0);
    assert_eq!(independent["replay"]["PCM_read"], false);
    assert_eq!(independent["replay"]["absolute_file_offset_bound"], false);
    assert_eq!(independent["replay"]["same_owner_reset_executed"], false);
}

#[test]
fn reader_source_position_package_is_bound_to_one_selected_capture() {
    let handoff_manifest_bytes = reader_source_fixture_bytes("HANDOFF-MANIFEST.json");
    let handoff_check_bytes = reader_source_fixture_bytes("HANDOFF-CHECK.json");
    let handoff_manifest: Value = serde_json::from_slice(&handoff_manifest_bytes).unwrap();
    let handoff_check: Value = serde_json::from_slice(&handoff_check_bytes).unwrap();
    assert_eq!(
        sha256(&handoff_manifest_bytes),
        READER_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(sha256(&handoff_check_bytes), READER_HANDOFF_CHECK_SHA256);
    assert_eq!(handoff_manifest["files"].as_array().unwrap().len(), 13);
    assert_eq!(
        handoff_manifest["source_package_manifest_sha256"],
        READER_SOURCE_MANIFEST_SHA256
    );
    assert_eq!(
        handoff_check["handoff_manifest_sha256"],
        READER_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(handoff_check["payload_hashes_match"], true);
    assert_eq!(handoff_check["whole_F13C_ready"], false);
    assert_eq!(handoff_check["implementation_ready"], false);

    for name in [
        "INPUTS.json",
        "NATIVE-OBSERVATIONS.json",
        "CONTRACT.json",
        "CURRENT-OVERLAY.json",
        "CHECK.json",
        "SOURCE-AUDIT.json",
        "WASH-INDEPENDENT-CHECK.json",
        "READER-SOURCE-ADDENDUM.md",
    ] {
        assert_reader_handoff_payload(&handoff_manifest, name);
    }

    let provenance = reader_source_fixture_json("PROVENANCE.json");
    assert_eq!(
        provenance["handoff_manifest_sha256"],
        READER_HANDOFF_MANIFEST_SHA256
    );
    assert_eq!(
        provenance["handoff_check_sha256"],
        READER_HANDOFF_CHECK_SHA256
    );
    assert_eq!(provenance["absolute_file_byte_offset"], "NOT_BOUND");
    assert_eq!(provenance["host_clock"], "NOT_BOUND");
    assert_eq!(provenance["same_owner_reset"], "NOT_RUN");
    assert_eq!(provenance["whole_F13C_ready"], false);

    let inputs = reader_source_fixture_json("INPUTS.json");
    let selected = &inputs["selected_source"];
    assert_eq!(selected["selection_index"], 0);
    assert_eq!(selected["start_sample"], 2_880_000);
    assert_eq!(selected["end_sample"], 2_886_000);
    assert_eq!(selected["duration_samples"], 6_000);
    assert_eq!(selected["frame_samples"], 1_536);
    assert_eq!(inputs["external_input_calls"].as_array().unwrap().len(), 4);

    let overlay = reader_source_fixture_json("CURRENT-OVERLAY.json");
    assert_eq!(
        overlay["selected_sample_coordinate"],
        "selected_item_start_sample + reader_cursor_before_samples, within one observed selected item"
    );
    assert_eq!(overlay["absolute_file_byte_offset"], "NOT_BOUND");
    assert_eq!(overlay["host_clock"], "NOT_BOUND");
    assert_eq!(overlay["same_owner_reset"], "NOT_RUN");
    assert_eq!(overlay["whole_F13C_ready"], "NO");

    let wash = reader_source_fixture_json("WASH-INDEPENDENT-CHECK.json");
    assert_eq!(wash["mismatches"], 0);
    assert_eq!(wash["PCM_values_read"], false);
    assert_eq!(wash["absolute_file_byte_offset_bound"], false);
    assert_eq!(wash["host_clock_bound"], false);
    assert_eq!(wash["constructor_descriptor_pointer_alias_proved"], false);
    assert_eq!(wash["same_owner_reset_run"], false);
    assert_eq!(wash["whole_F13C_ready"], false);
}

#[test]
fn reader_sample_coordinate_contract_does_not_promote_other_clock_domains() {
    let contract = reader_source_fixture_json("CONTRACT.json");
    let input = reader_source_fixture_json("INPUTS.json");
    let selected = &input["selected_source"];
    assert_eq!(contract["status"], "CLOSED_FOR_SELECTED_CAPTURE");
    assert_eq!(contract["scope"]["selected_item_index"], 0);
    assert_eq!(contract["scope"]["source_channels"], 38);
    assert_eq!(contract["scope"]["source_sample_rate_hz"], 48_000);
    assert_eq!(selected["start_sample"], 2_880_000);
    assert_eq!(selected["end_sample"], 2_886_000);

    let time_domains = &contract["time_domains"];
    assert_eq!(
        time_domains["control_position"],
        "selected-segment-relative sample position; equals reader cursor_before in this capture"
    );
    assert_eq!(
        time_domains["derived_source_sample_start"],
        "selected item start plus reader cursor_before; derived, not a separately emitted API field"
    );
    let limitations = contract["limitations"].as_array().unwrap();
    assert!(limitations.iter().any(|item| item == "One selected source/job only; no general claim for multi-item edits, seek, resampling, other rates, concurrency, or malformed inputs."));
    assert!(
        limitations
            .iter()
            .any(|item| item == "No PCM values or PCM content hashes were read or retained.")
    );
    assert!(limitations.iter().any(|item| item == "The selected-item constructor output and CompositeMaster descriptor are joined by selection index and exact field equality, not a proven pointer alias."));
}

#[test]
fn selected_process_flush_eof_and_close_sequence_matches_hash_bound_observations() {
    let inputs = fixture_json("INPUTS.json");
    let observations = fixture_json("NATIVE-OBSERVATIONS.json");
    let observation_rows = observations["observations"].as_array().unwrap();
    let reader_inputs = reader_source_fixture_json("INPUTS.json");
    let reader_native = reader_source_fixture_json("NATIVE-OBSERVATIONS.json");
    let reader_selected_source = &reader_inputs["selected_source"];
    let reader_calls = reader_native["external_input_calls"].as_array().unwrap();
    let source_indices = inputs["external_source_indices"].as_array().unwrap();
    let control_positions = inputs["external_control_positions"].as_array().unwrap();
    assert!(inputs["input_only"].as_bool().unwrap());
    assert_eq!(inputs["source_channels"], 38);
    assert_eq!(source_indices.len(), 38);
    assert_eq!(control_positions.len(), 4);
    assert_eq!(reader_calls.len(), 4);

    let mut lifecycle = InputEofLifecycle::cold();
    let mut process_receipts = Vec::new();
    let mut flush_receipts = Vec::new();
    let mut renderer_receipts = Vec::new();
    let mut frontend_receipts = Vec::new();
    let mut frontend_origins = Vec::new();

    for call in 1..=4_u64 {
        let control_position = control_positions[usize::try_from(call - 1).unwrap()]
            .as_u64()
            .unwrap();
        let records = source_records(control_position, source_indices);
        let reader_call = &reader_calls[usize::try_from(call - 1).unwrap()];
        let reader_source = ReaderSourcePositionObservation::new(
            ReaderCallIdentity::new(
                reader_call["input_api_call_ordinal"].as_u64().unwrap() as u32,
                reader_call["native_reader_frame_invocation"]
                    .as_u64()
                    .unwrap() as u32,
            ),
            SelectedItemSampleRange::new(
                reader_selected_source["selection_index"].as_u64().unwrap() as u16,
                SelectedItemSampleCoordinate::new(
                    reader_selected_source["start_sample"].as_u64().unwrap(),
                ),
                SelectedItemSampleCoordinate::new(
                    reader_selected_source["end_sample"].as_u64().unwrap(),
                ),
            ),
            ReaderCursorObservation::new(
                ExternalControlPosition::new(
                    reader_call["selected_segment_relative_control_position_samples"]
                        .as_u64()
                        .unwrap(),
                ),
                RelativeReaderCursorSamples::new(
                    reader_call["reader_cursor_before_samples"]
                        .as_u64()
                        .unwrap(),
                ),
                RelativeReaderCursorSamples::new(
                    reader_call["reader_cursor_after_samples"].as_u64().unwrap(),
                ),
            ),
            ReaderSampleCounts::new(
                reader_call["reader_requested_samples"].as_u64().unwrap() as u32,
                reader_call["read_float_requested_samples"]
                    .as_u64()
                    .unwrap() as u32,
                reader_call["read_float_actual_samples"].as_u64().unwrap() as u32,
                reader_call["returned_packet_samples"].as_u64().unwrap() as u32,
            ),
        );
        let reader_control_records = reader_call["control_records"].as_array().unwrap();
        assert_eq!(reader_control_records.len(), 38);
        for (source_index, control_record) in reader_control_records.iter().enumerate() {
            assert_eq!(
                control_record["source_index"].as_u64().unwrap(),
                source_indices[source_index].as_u64().unwrap()
            );
            assert_eq!(
                control_record["position_samples"].as_u64().unwrap(),
                control_position
            );
        }
        let renderer = observation_for_work_origin(observation_rows, "external_process", call);
        let source_origin = frontend_origin_for_work(observation_rows, "external_process", call);
        let frontend_source = source_origin.map(|origin| {
            let frontend_invocation = origin["frontend_invocation"].as_u64().unwrap();
            (
                source_step_input(&inputs, frontend_invocation),
                frontend_invocation,
            )
        });
        let receipt = lifecycle
            .process(ExternalProcessInput::new(
                &records,
                reader_source,
                RendererSpanSamples::new(renderer["samples"].as_u64().unwrap() as u32),
                frontend_source.map(|(source, _)| source),
            ))
            .unwrap();
        assert_eq!(receipt.call_number(), call as u32);
        assert_eq!(receipt.control_position().value(), control_position);
        assert_eq!(receipt.consumed_flag(), 1);
        assert_eq!(receipt.external_output_length(), 0);
        assert_eq!(receipt.return_i32(), 0);
        renderer_receipts.push(receipt.renderer_work());
        if let (Some(source), Some(origin)) = (receipt.frontend_source(), frontend_source) {
            frontend_receipts.push(source);
            frontend_origins.push(("external_process", call as u32, origin.1, source));
        }
        process_receipts.push(receipt);
    }

    let reader_input_calls = reader_inputs["external_input_calls"].as_array().unwrap();
    let mut derived_sample_starts = Vec::new();
    for (index, (process, reader_input)) in
        process_receipts.iter().zip(reader_input_calls).enumerate()
    {
        let reader = process.reader_source();
        let native = &reader_calls[index];
        assert_eq!(
            reader.identity().input_api_call_ordinal(),
            reader_input["input_api_call_ordinal"].as_u64().unwrap() as u32
        );
        assert_eq!(
            reader.identity().native_reader_frame_invocation(),
            native["native_reader_frame_invocation"].as_u64().unwrap() as u32
        );
        assert_eq!(reader.selected_item().selected_item_index(), 0);
        assert_eq!(reader.selected_item().start_sample().value(), 2_880_000);
        assert_eq!(reader.selected_item().end_sample().value(), 2_886_000);

        let control_position = reader_input["selected_segment_relative_control_position_samples"]
            .as_u64()
            .unwrap();
        assert_eq!(reader.cursor().control_position().value(), control_position);
        assert_eq!(reader.cursor().before().value(), control_position);
        assert_eq!(
            reader.cursor().after().value(),
            reader_input["reader_cursor_after_samples"]
                .as_u64()
                .unwrap()
        );
        assert_eq!(
            reader.cursor().control_position(),
            process.control_position()
        );

        let derived = reader.derived_selected_source_sample().unwrap().value();
        assert_eq!(
            derived,
            reader_input["derived_source_sample_start"]
                .as_u64()
                .unwrap()
        );
        assert_eq!(
            derived,
            native["derived_source_sample_start"].as_u64().unwrap()
        );
        derived_sample_starts.push(derived);

        let counts = reader.sample_counts();
        assert_eq!(
            u64::from(counts.reader_requested_samples()),
            reader_input["reader_requested_samples"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(counts.read_float_requested_samples()),
            reader_input["read_float_requested_samples"]
                .as_u64()
                .unwrap()
        );
        assert_eq!(
            u64::from(counts.read_float_actual_samples()),
            reader_input["read_float_actual_samples"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(counts.returned_packet_samples()),
            reader_input["returned_packet_samples"].as_u64().unwrap()
        );
        assert_eq!(counts.returned_packet_samples(), 1_536);
    }
    assert_eq!(
        derived_sample_starts,
        [2_880_000, 2_881_536, 2_883_072, 2_884_608]
    );
    assert_eq!(
        process_receipts
            .last()
            .unwrap()
            .reader_source()
            .sample_counts()
            .read_float_actual_samples(),
        1_392
    );

    for call in 1..=5_u64 {
        if call <= 4 {
            let renderer = observation_for_work_origin(observation_rows, "flush", call);
            let origin = frontend_origin_for_work(observation_rows, "flush", call).unwrap();
            let frontend_invocation = origin["frontend_invocation"].as_u64().unwrap();
            let source = source_step_input(&inputs, frontend_invocation);
            let receipt = lifecycle
                .flush(Some(ActiveFlushInput::new(
                    RendererSpanSamples::new(renderer["samples"].as_u64().unwrap() as u32),
                    source,
                )))
                .unwrap();
            renderer_receipts.push(receipt.renderer_work().unwrap());
            let frontend = receipt.frontend_source().unwrap();
            frontend_receipts.push(frontend);
            frontend_origins.push(("flush", call as u32, frontend_invocation, frontend));
            assert_eq!(receipt.call_number(), call as u32);
            flush_receipts.push(receipt);
        } else {
            let receipt = lifecycle.flush(None).unwrap();
            assert_eq!(receipt.call_number(), call as u32);
            flush_receipts.push(receipt);
        }
    }

    let expected = fixture_json("EXPECTED.json");
    let expected_actions = expected["actions"].as_array().unwrap();
    assert_eq!(expected_actions.len(), 9);
    for (receipt, expected_action) in process_receipts.iter().zip(expected_actions.iter().take(4)) {
        assert_eq!(expected_action["kind"], "process");
        assert_eq!(
            receipt.return_i32(),
            expected_action["return_i32"].as_i64().unwrap() as i32
        );
        assert_eq!(
            receipt.consumed_flag(),
            expected_action["consumed_flag"].as_u64().unwrap() as u32
        );
        assert_eq!(
            receipt.external_output_length(),
            expected_action["valid_bytes"].as_u64().unwrap() as u32
        );
        assert_snapshot(receipt.after(), &expected_action["after"]);
    }
    for (receipt, expected_action) in flush_receipts.iter().zip(expected_actions.iter().skip(4)) {
        assert_eq!(expected_action["kind"], "flush");
        assert_eq!(
            receipt.return_i32(),
            expected_action["return_i32"].as_i64().unwrap() as i32
        );
        assert_eq!(
            receipt.valid_output_bytes(),
            expected_action["valid_bytes"].as_u64().unwrap() as u32
        );
        assert_snapshot(receipt.after(), &expected_action["after"]);
    }

    let expected_renderer = expected["renderer_spans"].as_array().unwrap();
    assert_eq!(renderer_receipts.len(), 8);
    let mut prior_renderer_progress = 0;
    for (actual, expected_span) in renderer_receipts.iter().zip(expected_renderer) {
        assert_eq!(actual.progress_before(), prior_renderer_progress);
        assert_eq!(
            actual.invocation(),
            expected_span["work_invocation"].as_u64().unwrap() as u32
        );
        assert_eq!(
            actual.span().value(),
            expected_span["samples"].as_u64().unwrap() as u32
        );
        assert_eq!(
            actual.progress_after(),
            expected_span["progress_after"].as_u64().unwrap() as u32
        );
        prior_renderer_progress = actual.progress_after();
    }

    let expected_origins = expected["frontend_origins"].as_array().unwrap();
    assert_eq!(frontend_origins.len(), 6);
    assert_eq!(frontend_receipts.len(), 6);
    for ((kind, call, source_call, actual), expected_origin) in
        frontend_origins.iter().zip(expected_origins)
    {
        assert_eq!(*kind, expected_origin["origin_kind"].as_str().unwrap());
        assert_eq!(
            u64::from(*call),
            expected_origin["origin_call"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(actual.invocation()),
            expected_origin["frontend_invocation"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(actual.publication().valid_output_bytes()),
            expected_origin["valid_bytes"].as_u64().unwrap()
        );
        assert_eq!(
            actual
                .published_au_ordinal()
                .map(|ordinal| u64::from(ordinal.value())),
            expected_origin
                .get("published_au_ordinal")
                .and_then(Value::as_u64)
        );
        assert_eq!(
            *source_call,
            expected_origin["frontend_invocation"].as_u64().unwrap()
        );
        assert_eq!(actual.frontend_ordinal().value(), actual.invocation());
        assert_eq!(
            u64::from(actual.pending_id14_payload_bytes()),
            inputs["native_boundary_id14_body_lengths"][usize::try_from(*source_call - 1).unwrap()]
                .as_u64()
                .unwrap()
        );
    }

    let carrier_joins = fixture_json("CARRIER-NATIVE-AU-JOINS.json");
    let carrier_aus = carrier_joins["published_aus"].as_array().unwrap();
    let published_frontend = frontend_receipts
        .iter()
        .filter(|receipt| receipt.published_au_ordinal().is_some())
        .collect::<Vec<_>>();
    assert_eq!(published_frontend.len(), 4);
    for (ordinal, (frontend, native_au)) in published_frontend.iter().zip(carrier_aus).enumerate() {
        assert_eq!(
            frontend.published_au_ordinal().unwrap().value(),
            ordinal as u32
        );
        assert_eq!(
            u64::from(frontend.invocation()),
            native_au["frontend_source_call"].as_u64().unwrap()
        );
        assert_eq!(
            u64::try_from(frontend.planner().requested_body_bytes()).unwrap(),
            native_au["body_bytes"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(frontend.planner().first_block().value()),
            native_au["block"].as_u64().unwrap()
        );
        assert_eq!(
            native_au["joc_producer_invocation"].as_u64().unwrap() + 1,
            u64::from(frontend.invocation())
        );
    }

    let post_flush_before = flush_receipts.last().unwrap().after();
    let post_flush = lifecycle.process_after_flush_started().unwrap();
    assert_eq!(post_flush.return_i32(), -3);
    assert_eq!(post_flush.return_raw_u32(), 4_294_967_293);
    assert_eq!(post_flush.consumed_flag(), 1);
    assert_eq!(post_flush.output_length(), 0);
    assert_eq!(post_flush.state_after(), lifecycle.snapshot());
    assert_eq!(post_flush.state_after(), post_flush_before);
    let early = expected["early_returns"].as_array().unwrap();
    assert_eq!(early[0]["return_i32"], -3);
    assert_eq!(early[0]["consumed_flag_after"], 1);
    assert_eq!(early[0]["output_length_after"], "0");
    assert_eq!(post_flush_before.flush_remaining(), 0);

    let close = lifecycle.close();
    assert_eq!(
        close.return_i32(),
        expected["close_return_i32"].as_i64().unwrap() as i32
    );
    assert_snapshot(close.scalar_state_at_close(), &expected["close_state"]);
    let retirement = &expected["workspace_retirement"];
    assert_eq!(
        close.owner_close_order(),
        if retirement["owner_closed_before_free"].as_bool().unwrap() {
            OwnerCloseOrder::ClosedBeforeFree
        } else {
            OwnerCloseOrder::NotClosedBeforeFree
        }
    );
    assert_eq!(
        close.allocation_retirement(),
        if retirement["allocation_retired"].as_bool().unwrap() {
            AllocationRetirement::Retired
        } else {
            AllocationRetirement::StillAllocated
        }
    );
    assert_eq!(
        close.post_retirement_read(),
        if retirement["post_free_memory_read"].as_bool().unwrap() {
            PostRetirementRead::Read
        } else {
            PostRetirementRead::NoRead
        }
    );
    assert_eq!(
        close.semantic_queue_status(),
        SemanticQueueStatus::PendingStateRetained
    );
    let close_projection = close.scalar_state_at_close();
    assert_eq!(close_projection.external_process_calls(), 4);
    assert_eq!(close_projection.flush_calls(), 5);
    assert_eq!(close_projection.renderer_progress_samples(), 4_608);
    assert_eq!(close_projection.published_au_count(), 4);
}
