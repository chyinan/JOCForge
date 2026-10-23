// pattern: Functional Core
use jocforge_domain::meter::output_schedule::{
    MeterOutputScheduleError, calculate_meter_output_schedule,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const SCHEDULE_VECTORS_SHA256: &str =
    "a565e513970670967aa3aebd0372f9dbb5f9bbab670a54744ae454926b3f2671";
const STATE_REVIEW_SHA256: &str =
    "020f88cafa892119bac5fea5bc808d251798e98aec69897c6ab068da80637307";
const FIXTURE_DIRECTORY: &str = "tests/fixtures/meter_output_schedule";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_json(path: &str) -> (Vec<u8>, Value) {
    let bytes = std::fs::read(crate_root().join(path)).unwrap();
    let value = serde_json::from_slice(&bytes).unwrap();
    (bytes, value)
}

fn manifest_entry<'a>(manifest: &'a Value, file: &str) -> &'a Value {
    manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["file"] == file)
        .unwrap()
}

#[test]
fn source_assets_and_schedule_only_boundary_are_hash_bound() {
    let (manifest_bytes, manifest) = read_json("tests/fixtures/f01e_provider/MANIFEST.json");
    let (vectors_bytes, vectors) = read_json(&format!(
        "{FIXTURE_DIRECTORY}/METER-INTEGRATED-SCHEDULE-VERIFICATION.json"
    ));
    let (review_bytes, review) = read_json(&format!(
        "{FIXTURE_DIRECTORY}/METER-STATE-CLOCK-LOCAL-REVIEW-STATUS.json"
    ));

    assert_eq!(sha256(&manifest_bytes), PACKAGE_MANIFEST_SHA256);
    assert_eq!(sha256(&vectors_bytes), SCHEDULE_VECTORS_SHA256);
    assert_eq!(sha256(&review_bytes), STATE_REVIEW_SHA256);
    for (file, bytes, hash) in [
        (
            "specs/06-meter-window-and-output-clock.md",
            7_388,
            "c08dd41c42dd231ef23a683415f59b9b7db60375a3da2b8140501969933d073c",
        ),
        (
            "parameters/meter-k-state-bounds-v1.json",
            970,
            "2faaa9d0aeefb105a913589590fcbafb97362a54192b0b21c5fdda7da1974e68",
        ),
        (
            "test-vectors/METER-INTEGRATED-SCHEDULE-VERIFICATION.json",
            2_051_223,
            SCHEDULE_VECTORS_SHA256,
        ),
        (
            "test-vectors/METER-STATE-CLOCK-LOCAL-REVIEW-STATUS.json",
            10_646,
            STATE_REVIEW_SHA256,
        ),
    ] {
        let entry = manifest_entry(&manifest, file);
        assert_eq!(
            entry["bytes"].as_u64(),
            Some(u64::try_from(bytes).unwrap()),
            "{file}"
        );
        assert_eq!(entry["sha256"].as_str(), Some(hash), "{file}");
    }
    assert_eq!(vectors["cases"].as_array().unwrap().len(), 10);
    assert_eq!(vectors["schedule_cases"].as_array().unwrap().len(), 5);
    assert_eq!(vectors["schedule_cases"][0]["input_source_samples"], 1);
    assert_eq!(vectors["schedule_cases"][1]["input_source_samples"], 1024);
    assert_eq!(vectors["schedule_cases"][2]["input_source_samples"], 1536);
    assert_eq!(vectors["schedule_cases"][3]["input_source_samples"], 1537);
    assert_eq!(
        vectors["schedule_cases"][4]["input_source_samples"],
        6_522_906
    );
    assert_eq!(
        review["verdict"],
        "MST_REV_001_CLOSED_LOCAL_SCOPE_NO_NEW_LOCAL_ISSUES"
    );
    assert_eq!(review["recalculation"]["schedule_cases"], 5);
}

#[test]
fn five_synthetic_schedule_cases_match_spans_and_sample_accounting() {
    let (_, vectors) = read_json(&format!(
        "{FIXTURE_DIRECTORY}/METER-INTEGRATED-SCHEDULE-VERIFICATION.json"
    ));
    for case in vectors["schedule_cases"].as_array().unwrap() {
        let actual =
            calculate_meter_output_schedule(case["input_source_samples"].as_u64().unwrap())
                .unwrap_or_else(|error| panic!("N={}: {error:?}", case["input_source_samples"]));
        assert_eq!(
            actual.spans().len(),
            case["expected_spans"].as_array().unwrap().len(),
            "N={}",
            case["input_source_samples"]
        );
        for (index, (actual_span, expected_span)) in actual
            .spans()
            .iter()
            .zip(case["expected_spans"].as_array().unwrap())
            .enumerate()
        {
            assert_eq!(
                actual_span.callback_index(),
                expected_span["callback_index"].as_u64().unwrap(),
                "N={} span {index} callback",
                case["input_source_samples"]
            );
            assert_eq!(
                actual_span.phase_name(),
                expected_span["phase"].as_str().unwrap(),
                "N={} span {index} phase",
                case["input_source_samples"]
            );
            assert_eq!(
                actual_span.raw_frame_sample_begin(),
                expected_span["raw_frame_sample_begin"].as_u64().unwrap(),
                "N={} span {index} raw begin",
                case["input_source_samples"]
            );
            assert_eq!(
                actual_span.samples(),
                expected_span["samples"].as_u64().unwrap(),
                "N={} span {index} sample count",
                case["input_source_samples"]
            );
            assert_eq!(
                actual_span.valid_programme_sample_begin(),
                expected_span["valid_programme_sample_begin"]
                    .as_u64()
                    .unwrap(),
                "N={} span {index} programme begin",
                case["input_source_samples"]
            );
        }
        for (field, actual_value) in [
            ("normal_callbacks", actual.normal_callbacks()),
            ("drain_callbacks", actual.drain_callbacks()),
            ("published_samples", actual.published_samples()),
            ("source_padding_samples", actual.source_padding_samples()),
            ("consumed1024_samples", actual.consumed1024_samples()),
            ("complete960_windows", actual.complete960_windows()),
            ("uncommitted_pcm_samples", actual.uncommitted_pcm_samples()),
            (
                "uncommitted_energy_samples",
                actual.uncommitted_energy_samples(),
            ),
        ] {
            assert_eq!(
                actual_value,
                case[field].as_u64().unwrap(),
                "N={} {field}",
                case["input_source_samples"]
            );
        }
    }
}

#[test]
fn invalid_source_count_and_overflow_are_rejected() {
    assert_eq!(
        calculate_meter_output_schedule(0),
        Err(MeterOutputScheduleError::InvalidSourceSampleCount)
    );
    assert!(matches!(
        calculate_meter_output_schedule(u64::MAX),
        Err(MeterOutputScheduleError::ArithmeticOverflow
            | MeterOutputScheduleError::SpanCapacityUnavailable)
    ));
}
