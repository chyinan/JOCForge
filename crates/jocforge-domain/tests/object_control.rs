// pattern: Imperative Shell
use jocforge_domain::frontend::object_control::{
    ObjectControlError, SelectedObjectEvent, build_selected_object_control52,
};

const EVENT_VECTORS: &str =
    include_str!("fixtures/job_configuration/job-configuration-vectors.json");

#[test]
fn selected_event_cases_produce_expected_control52_words_after_computation() {
    let vectors: serde_json::Value = serde_json::from_str(EVENT_VECTORS).unwrap();
    let cases = vectors["event_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 3);
    for case in cases {
        let input = &case["input"];
        let event = event_from_json(input);
        let row = build_selected_object_control52(event).unwrap();
        assert_eq!(
            row.words(),
            expected_words(&case["expected"]["control52_words"])
        );
    }
}

#[test]
fn all_28_selected_object_value_rows_match_the_hash_bound_words() {
    let vectors: serde_json::Value = serde_json::from_str(EVENT_VECTORS).unwrap();
    let cases = vectors["actual_fixture_object_controls"]
        .as_array()
        .unwrap();
    assert_eq!(cases.len(), 28);
    for case in cases {
        let row = build_selected_object_control52(event_from_json(&case["input"])).unwrap();
        assert_eq!(
            row.words(),
            expected_words(&case["expected_control52_words"]),
            "physical ordinal {}",
            case["physical_object_ordinal"]
        );
    }
}

#[test]
fn selected_object_value_builder_rejects_start_duration_and_host_domain_errors() {
    let vectors: serde_json::Value = serde_json::from_str(EVENT_VECTORS).unwrap();
    let input = &vectors["event_cases"][0]["input"];
    let duration = input["duration_samples"].as_u64().unwrap();
    let event = event_with_values(input, 10, 1, duration, duration);
    assert_eq!(
        build_selected_object_control52(event).unwrap_err(),
        ObjectControlError::EventStart
    );

    let event = event_with_values(input, 10, 0, 1, duration);
    assert_eq!(
        build_selected_object_control52(event).unwrap_err(),
        ObjectControlError::EventDuration
    );

    let event = event_with_values(input, 38, 0, duration, duration);
    assert_eq!(
        build_selected_object_control52(event).unwrap_err(),
        ObjectControlError::HostDomain
    );
}

fn event_from_json(input: &serde_json::Value) -> SelectedObjectEvent {
    let duration = input["duration_samples"].as_u64().unwrap();
    let host = u8::try_from(input["host_channel_index"].as_u64().unwrap()).unwrap();
    let start = input["start_sample"].as_u64().unwrap();
    event_with_values(input, host, start, duration, duration)
}

fn event_with_values(
    input: &serde_json::Value,
    host: u8,
    start: u64,
    duration: u64,
    programme_samples: u64,
) -> SelectedObjectEvent {
    SelectedObjectEvent::new(
        host,
        start,
        duration,
        programme_samples,
        optional_number(input, "x"),
        optional_number(input, "y"),
        optional_number(input, "z"),
        optional_number(input, "width"),
        optional_number(input, "depth"),
        optional_number(input, "height"),
        input["diffuse"]
            .as_i64()
            .map(|value| i32::try_from(value).unwrap()),
    )
}

fn optional_number(input: &serde_json::Value, field: &str) -> Option<f64> {
    input.get(field).and_then(serde_json::Value::as_f64)
}

fn expected_words(value: &serde_json::Value) -> [u32; 13] {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|word| u32::from_str_radix(word.as_str().unwrap(), 16).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
