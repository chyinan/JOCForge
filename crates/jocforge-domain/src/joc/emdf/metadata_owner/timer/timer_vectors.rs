use super::*;
use serde_json::Value;

const PRIMITIVE_INPUTS: &str =
    include_str!("../../../../../tests/fixtures/metadata_owner/id1_timer/PRIMITIVE-INPUTS.json");
const PRIMITIVE_OBSERVATIONS: &str = include_str!(
    "../../../../../tests/fixtures/metadata_owner/id1_timer/PRIMITIVE-OBSERVATIONS.json"
);
const CONTINUOUS_INPUT: &str =
    include_str!("../../../../../tests/fixtures/metadata_owner/id1_timer/CONTINUOUS-INPUT.json");
const CONTINUOUS_OBSERVATIONS: &str = include_str!(
    "../../../../../tests/fixtures/metadata_owner/id1_timer/CONTINUOUS-OBSERVATIONS.json"
);

#[test]
fn washed_timer_fixture_files_match_the_bound_supplement_hashes() {
    let files = [
        (
            "PRIMITIVE-INPUTS.json",
            PRIMITIVE_INPUTS.as_bytes(),
            "70a94a3ac292fe0342ffccaffc7ee024431da30260ad0a1d6210404ac786a445",
        ),
        (
            "PRIMITIVE-OBSERVATIONS.json",
            PRIMITIVE_OBSERVATIONS.as_bytes(),
            "3d1bedb70bb7edca6427a3dbcd703d92ab0b6f4e57e08651f244a418ad7442d9",
        ),
        (
            "CONTINUOUS-INPUT.json",
            CONTINUOUS_INPUT.as_bytes(),
            "f8f34e9ea015941682ea4b0298fdf8293e469cb63532bb72cfd410308c38f168",
        ),
        (
            "CONTINUOUS-OBSERVATIONS.json",
            CONTINUOUS_OBSERVATIONS.as_bytes(),
            "7055ee26d6b95102ff8d1a3b8765b4fdacbe57eb16fe84495d10c4f98fdf776a",
        ),
    ];
    for (name, bytes, expected) in files {
        assert_eq!(bytes.len(), expected_bytes(name));
        assert_eq!(sha256_hex(bytes), expected, "{name}");
    }
}

#[test]
fn timer_primitive_matches_all_hash_bound_state_config_and_flag_cases() {
    let input: Value = serde_json::from_str(PRIMITIVE_INPUTS).unwrap();
    let observed: Value = serde_json::from_str(PRIMITIVE_OBSERVATIONS).unwrap();
    let inputs = input["timer_cases"].as_array().unwrap();
    let outputs = observed["timer_cases"].as_array().unwrap();
    assert_eq!(inputs.len(), 17);
    assert_eq!(outputs.len(), inputs.len());

    for (input, expected) in inputs.iter().zip(outputs) {
        assert_eq!(input["name"], expected["name"]);
        let state = timer_state(&input["state"]);
        let config = config_values(&input["config"]);
        let samples = input["samples"].as_i64().unwrap();
        let counter_flag = input["counter_flag"].as_u64().unwrap() != 0;
        let result = state
            .advance(&config_values_json(&config), samples, counter_flag)
            .unwrap();

        assert_timer_state(&result.next, &expected["state"]);
        assert_eq!(result.output, config_values(&expected["config"]));
        assert_eq!(
            result.counter_flag,
            expected["counter_flag"].as_u64().unwrap() != 0
        );
    }
}

#[test]
fn selected_timer_consumer_matches_forty_serialized_id1_calls() {
    let input: Value = serde_json::from_str(CONTINUOUS_INPUT).unwrap();
    let observed: Value = serde_json::from_str(CONTINUOUS_OBSERVATIONS).unwrap();
    let mut state = timer_state(&input["cold_rule"]);
    assert_eq!(
        input["calls"].as_u64(),
        observed["calls"].as_array().map(|a| a.len() as u64)
    );
    let config = config_values(&input["input_config"]);
    let samples = input["samples_per_call"].as_i64().unwrap();

    for (call_index, expected) in observed["calls"].as_array().unwrap().iter().enumerate() {
        assert_eq!(expected["call"].as_u64(), Some(call_index as u64));
        assert_timer_state(&state, &expected["state_before"]);
        let result = state
            .advance(&config_values_json(&config), samples, false)
            .unwrap();
        assert_eq!(result.output, config_values(&expected["config_after"]));
        assert_timer_state(&result.next, &expected["state_after"]);
        let body = encode_id1_payload(&result.output).unwrap();
        assert_eq!(hex(&body), expected["body_hex"].as_str().unwrap());
        assert_eq!(
            body.len() as u64,
            expected["encoded_bytes"].as_u64().unwrap()
        );
        state = result.next;
    }

    assert_timer_state(&state, &observed["same_owner_reinit"]["state_before"]);
    assert_eq!(
        timer_state(&input["cold_rule"]),
        timer_state(&observed["same_owner_reinit"]["state_after"])
    );
}

fn timer_state(value: &Value) -> TimerState {
    TimerState {
        remaining: i64_array(&value["remaining"]),
        cached: i64_array(&value["cached"]),
        mode: value["mode"].as_i64().unwrap(),
        counter: value["counter_low_s16"].as_i64().unwrap(),
    }
}

fn assert_timer_state(actual: &TimerState, expected: &Value) {
    assert_eq!(actual.remaining, i64_array(&expected["remaining"]));
    assert_eq!(actual.cached, i64_array(&expected["cached"]));
    assert_eq!(actual.mode, expected["mode"].as_i64().unwrap());
    assert_eq!(
        actual.counter,
        expected["counter_low_s16"].as_i64().unwrap()
    );
    assert_eq!(expected["counter_extra_u16"].as_u64(), Some(0));
    assert_eq!(expected["group_s16"].as_i64(), Some(-1));
    assert_eq!(expected["codec_s16"].as_i64(), Some(-1));
}

fn i64_array(value: &Value) -> [i64; 6] {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_i64().unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn config_values(value: &Value) -> [i64; 9] {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_i64().unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn config_values_json(values: &[i64; 9]) -> Vec<Value> {
    values.iter().copied().map(Value::from).collect()
}

fn expected_bytes(name: &str) -> usize {
    match name {
        "PRIMITIVE-INPUTS.json" => 14_878,
        "PRIMITIVE-OBSERVATIONS.json" => 19_890,
        "CONTINUOUS-INPUT.json" => 693,
        "CONTINUOUS-OBSERVATIONS.json" => 50_035,
        _ => unreachable!("test fixture is explicitly enumerated"),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    super::super::hex_encode(&digest)
}

fn hex(bytes: &[u8]) -> String {
    super::super::hex_encode(bytes)
}
