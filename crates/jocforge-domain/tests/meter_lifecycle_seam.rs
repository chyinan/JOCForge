// pattern: Functional Core; test-only supplied trace validator
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

const FIXTURE_MANIFEST_SHA256: &str =
    "db592e311a52986df48af8f83427ae80749044db860418ff63b4c59770287816";
const PROVIDER_FILE: &str = "meter-selected-provider.json";
const VECTORS_FILE: &str = "METER-OWNER-VECTORS.json";
const SOURCE_IDENTITIES: &[(&str, u64, &str)] = &[
    (
        "cold",
        8973,
        "a2283d3e1dd06f32d40e360ee48018501d0b88b2a6dbd0ad8234d9e82d14b74a",
    ),
    (
        "input",
        2218,
        "5faceecd9b13bacf1033bdafbb72f5bec854304a5c1bd5cce5b804ee6b9a299f",
    ),
    (
        "results",
        126007,
        "70a891b8c55950d2657c2798a04bf4af004b03b4b9bb5b92b6533aefec98b74a",
    ),
];
const OWNER_IDS: &[&str] = &[
    "lifecycle",
    "meter_frontend_oar",
    "shared_dv32",
    "window_k_energy",
    "true_peak",
    "integrated",
    "ungated",
    "ordinary_history",
    "secondary",
    "none_statistics",
    "none_scheduler",
    "override_fifo",
    "programme_measurement",
];

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/meter_lifecycle")
}

fn digest_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut text = String::with_capacity(digest.len() * 2);
    for byte in digest {
        text.push(char::from(DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    text
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push(char::from(DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    text
}

fn canonical_json_sha256(value: &Value) -> String {
    digest_hex(&serde_json::to_vec(value).expect("canonical fixture JSON"))
}

fn read_fixture(name: &str) -> Value {
    let bytes = fs::read(fixture_root().join(name)).expect("fixture file exists");
    serde_json::from_slice(&bytes).expect("fixture JSON parses")
}

fn assert_fixture_manifest() {
    let root = fixture_root();
    let bytes =
        fs::read(root.join("METER-FIXTURE-MANIFEST.json")).expect("meter fixture manifest exists");
    assert_eq!(digest_hex(&bytes), FIXTURE_MANIFEST_SHA256);
    let manifest: Value = serde_json::from_slice(&bytes).expect("manifest JSON");
    let files = manifest["files"].as_array().expect("manifest files array");
    assert_eq!(files.len(), 7);
    for entry in files {
        let name = entry["file"].as_str().expect("manifest file name");
        let bytes = fs::read(root.join(name)).expect("manifested file exists");
        assert_eq!(bytes.len() as u64, entry["bytes"].as_u64().unwrap());
        assert_eq!(digest_hex(&bytes), entry["sha256"].as_str().unwrap());
    }
}

fn error_result(code: &str) -> Value {
    json!({
        "status": "ERROR",
        "error": code,
        "receipt": null,
        "measurement": null,
        "state_delta": null,
        "commit_count": 0,
    })
}

fn has_exact_keys(value: &Value, keys: &[&str]) -> bool {
    value.as_object().is_some_and(|object| {
        object.len() == keys.len() && keys.iter().all(|key| object.contains_key(*key))
    })
}

fn is_sha256(value: &Value) -> bool {
    value.as_str().is_some_and(|text| {
        text.len() == 64
            && text
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn owner_hash_maps_valid(trace: &Value) -> bool {
    let before = &trace["owner_before_sha256"];
    let after = &trace["owner_after_sha256"];
    if !has_exact_keys(before, OWNER_IDS) || !has_exact_keys(after, OWNER_IDS) {
        return false;
    }
    if !OWNER_IDS
        .iter()
        .all(|owner| is_sha256(&before[*owner]) && is_sha256(&after[*owner]))
    {
        return false;
    }

    let unchanged: BTreeSet<String> = OWNER_IDS
        .iter()
        .filter(|owner| before[**owner] == after[**owner])
        .map(|owner| (*owner).to_owned())
        .collect();
    let declared: BTreeSet<String> = trace["unchanged_numeric_owners"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    unchanged == declared && trace["numeric_owners_unchanged_on_failure"] == true
}

fn trace_hashes_valid(trace: &Value) -> bool {
    is_sha256(&trace["before_state_sha256"])
        && is_sha256(&trace["after_state_sha256"])
        && canonical_json_sha256(&trace["before"]) == trace["before_state_sha256"]
        && canonical_json_sha256(&trace["after"]) == trace["after_state_sha256"]
}

fn callback_events(blocks: u64, energy_records: u64) -> Vec<&'static str> {
    let mut events = vec![
        "meter_front",
        "meter_oar",
        "shared_dv32",
        "valid_audio_scheduler",
    ];
    for _ in 0..blocks {
        events.push("true_and_sample_peak");
        events.push("k_filter_energy");
    }
    for _ in 0..energy_records {
        events.push("integrated_add");
        events.push("ungated_add");
    }
    events.push("meter_callback_commit");
    events
}

fn finalize_events() -> Vec<&'static str> {
    vec![
        "true_peak_getter",
        "sample_peak_getter",
        "integrated_getter",
        "ungated_getter",
        "di_off_getters",
        "selected_loudness",
        "dialnorm",
        "freeze_programme_measurement",
        "meter_finalize_commit",
    ]
}

fn validate_callback_trace(trace: &Value, provider_sha256: &str) -> Result<Value, &'static str> {
    let input = &trace["input"];
    let before = &trace["before"];
    let after = &trace["after"];
    let Some(callback) = input["callback"].as_u64() else {
        return Err("METER_CLOCK");
    };
    if input["op"] != "meter_callback"
        || before["meter_callback"].as_u64() != Some(callback)
        || after["meter_callback"].as_u64() != callback.checked_add(1)
    {
        return Err("METER_CLOCK");
    }

    let Some(blocks) = input["complete_1024_blocks"].as_i64() else {
        return Err("METER_COUNTS");
    };
    let Some(energy_records) = input["energy_records"].as_i64() else {
        return Err("METER_COUNTS");
    };
    if blocks < 0 || energy_records < 0 || energy_records > blocks {
        return Err("METER_COUNTS");
    }
    let blocks = u64::try_from(blocks).map_err(|_| "METER_COUNTS")?;
    let energy_records = u64::try_from(energy_records).map_err(|_| "METER_COUNTS")?;
    let Some(after_blocks) = before["complete_1024_blocks"]
        .as_u64()
        .and_then(|count| count.checked_add(blocks))
    else {
        return Err("METER_COUNTS");
    };
    let Some(after_energy) = before["energy_records"]
        .as_u64()
        .and_then(|count| count.checked_add(energy_records))
    else {
        return Err("METER_COUNTS");
    };
    if after["complete_1024_blocks"].as_u64() != Some(after_blocks)
        || after["energy_records"].as_u64() != Some(after_energy)
        || before["integrated_generation"]
            .as_u64()
            .and_then(|count| count.checked_add(energy_records))
            != after["integrated_generation"].as_u64()
        || before["ungated_generation"]
            .as_u64()
            .and_then(|count| count.checked_add(energy_records))
            != after["ungated_generation"].as_u64()
    {
        return Err("METER_COUNTS");
    }

    for generation in [
        "meter_front_generation",
        "oar_generation",
        "dv32_generation",
    ] {
        if before[generation]
            .as_u64()
            .and_then(|value| value.checked_add(1))
            != after[generation].as_u64()
        {
            return Err("METER_CLOCK");
        }
    }
    if after["meter_finalized"] != false || !after["programme_measurement"].is_null() {
        return Err("METER_CLOCK");
    }

    let expected_events = callback_events(blocks, energy_records);
    if trace["events"] != json!(expected_events) {
        return Err("METER_EVENTS");
    }
    if !owner_hash_maps_valid(trace) {
        return Err("METER_OWNER_IDENTITY");
    }
    if !trace_hashes_valid(trace) {
        return Err("METER_OWNER_IDENTITY");
    }

    let input_metrics = input.get("metrics");
    let mut expected_observed = before["meter_observed"].clone();
    if let Some(metrics) = input_metrics {
        for name in [
            "true_peak_host",
            "sample_peak_host",
            "integrated_host",
            "ungated_host",
        ] {
            if let Some(value) = metrics.get(name) {
                expected_observed[name] = value.clone();
            }
        }
    }
    if after["meter_observed"] != expected_observed {
        return Err("METER_EVENTS");
    }

    let receipt = json!({
        "schema": "meter-fixture-validation-receipt.v1",
        "kind": "CALLBACK",
        "production_allowed": false,
        "callback": callback,
        "before_state_sha256": trace["before_state_sha256"],
        "recorded_after_state_sha256": trace["after_state_sha256"],
        "provider_sha256": provider_sha256,
        "state_delta_committed": false,
        "commit_count": 0,
    });
    Ok(json!({
        "status": "PASS",
        "receipt": receipt,
        "measurement": null,
        "state_delta": null,
        "commit_count": 0,
    }))
}

fn validate_measurement(measurement: &Value) -> Result<String, &'static str> {
    const MEASUREMENT_KEYS: [&str; 10] = [
        "dialogue_host",
        "speech_percentage_host",
        "itu_host",
        "ungated_host",
        "true_peak_host",
        "sample_peak_host",
        "selected_host",
        "gating_kind",
        "dialnorm",
        "digest",
    ];
    const NUMERIC_FIELDS: [&str; 7] = [
        "dialogue_host",
        "speech_percentage_host",
        "itu_host",
        "ungated_host",
        "true_peak_host",
        "sample_peak_host",
        "selected_host",
    ];
    if !has_exact_keys(measurement, &MEASUREMENT_KEYS)
        || NUMERIC_FIELDS
            .iter()
            .any(|field| measurement[*field].as_f64().is_none())
    {
        return Err("METER_NUMERIC");
    }
    if measurement["dialogue_host"].as_f64() != Some(-120.0)
        || measurement["speech_percentage_host"].as_f64() != Some(-1.0)
        || measurement["dialnorm"]
            .as_u64()
            .is_none_or(|value| !(1..=31).contains(&value))
        || !matches!(measurement["gating_kind"].as_u64(), Some(0 | 2))
    {
        return Err("METER_MEASUREMENT_DOMAIN");
    }
    let digest = measurement["digest"]
        .as_str()
        .ok_or("METER_MEASUREMENT_DIGEST")?;
    if !is_sha256(&measurement["digest"]) {
        return Err("METER_MEASUREMENT_DIGEST");
    }
    let mut material = measurement.clone();
    material
        .as_object_mut()
        .expect("measurement object")
        .remove("digest");
    if canonical_json_sha256(&material) != digest {
        return Err("METER_MEASUREMENT_DIGEST");
    }
    Ok(digest.to_owned())
}

fn validate_finalize_trace(
    trace: &Value,
    provider_sha256: &str,
    normal_callbacks: u64,
) -> Result<Value, &'static str> {
    let before = &trace["before"];
    let after = &trace["after"];
    if trace["input"]["op"] != "finalize"
        || before["meter_callback"].as_u64() != Some(normal_callbacks)
        || before["meter_finalized"] != false
        || !before["programme_measurement"].is_null()
    {
        return Err("METER_FINALIZE_PRECONDITION");
    }
    if after["meter_callback"] != before["meter_callback"]
        || after["complete_1024_blocks"] != before["complete_1024_blocks"]
        || after["energy_records"] != before["energy_records"]
        || after["meter_finalized"] != true
        || !after["programme_measurement"].is_object()
    {
        return Err("METER_FINALIZE_PRECONDITION");
    }
    if trace["events"] != json!(finalize_events()) {
        return Err("METER_EVENTS");
    }
    if !owner_hash_maps_valid(trace) {
        return Err("METER_OWNER_IDENTITY");
    }

    let measurement = &after["programme_measurement"];
    let measurement_sha256 = validate_measurement(measurement)?;
    if !trace_hashes_valid(trace) {
        return Err("METER_OWNER_IDENTITY");
    }
    let receipt = json!({
        "schema": "meter-fixture-validation-receipt.v1",
        "kind": "FINALIZE",
        "production_allowed": false,
        "callback": normal_callbacks,
        "before_state_sha256": trace["before_state_sha256"],
        "recorded_after_state_sha256": trace["after_state_sha256"],
        "provider_sha256": provider_sha256,
        "measurement_sha256": measurement_sha256,
        "state_delta_committed": false,
        "commit_count": 0,
    });
    Ok(json!({
        "status": "PASS",
        "receipt": receipt,
        "measurement": measurement,
        "state_delta": null,
        "commit_count": 0,
    }))
}

fn validate_trace(trace: &Value, provider_sha256: &str, normal_callbacks: u64) -> Value {
    let result = match trace["input"]["op"].as_str() {
        Some("meter_callback") => validate_callback_trace(trace, provider_sha256),
        Some("finalize") => validate_finalize_trace(trace, provider_sha256, normal_callbacks),
        _ => Err("METER_EVENTS"),
    };
    result.unwrap_or_else(error_result)
}

#[test]
fn meter_lifecycle_fixture_recomputes_hashes_and_boundaries_without_execution() {
    assert_fixture_manifest();
    let provider = read_fixture(PROVIDER_FILE);
    let machine_map = read_fixture("METER-MACHINE-MAP.json");
    let vectors = read_fixture(VECTORS_FILE);
    let independent = read_fixture("INDEPENDENT-CHECK.json");

    let provider_sha256 = vectors["provider"]["canonical_sha256"]
        .as_str()
        .expect("provider SHA-256");
    assert_eq!(canonical_json_sha256(&provider), provider_sha256);
    assert_eq!(provider, vectors["provider"]["value"]);
    assert_eq!(provider["profile_id"], "bs1770-4-di-off-selected.v1");
    assert_eq!(provider["sample_rate"], 48_000);
    assert_eq!(provider["callback_samples"], 1536);
    assert_eq!(provider["meter_zero_drains"], 3);
    assert_eq!(provider["production_allowed"], false);
    assert_eq!(
        vectors["source_identities"]["cold"]["bytes"],
        SOURCE_IDENTITIES[0].1
    );
    assert_eq!(
        vectors["source_identities"]["cold"]["sha256"],
        SOURCE_IDENTITIES[0].2
    );
    assert_eq!(
        vectors["source_identities"]["input"]["bytes"],
        SOURCE_IDENTITIES[1].1
    );
    assert_eq!(
        vectors["source_identities"]["input"]["sha256"],
        SOURCE_IDENTITIES[1].2
    );
    assert_eq!(
        vectors["source_identities"]["results"]["bytes"],
        SOURCE_IDENTITIES[2].1
    );
    assert_eq!(
        vectors["source_identities"]["results"]["sha256"],
        SOURCE_IDENTITIES[2].2
    );
    let provider_bindings = provider["provider_bindings"].as_object().unwrap();
    assert_eq!(provider_bindings.len(), 7);
    assert!(provider_bindings.values().all(is_sha256));

    let cold = &vectors["cold"];
    let cold_bytes = serde_json::to_vec(&cold["state"]).expect("cold canonical state");
    assert_eq!(digest_hex(&cold_bytes), cold["sha256"]);
    assert_eq!(
        hex(&cold_bytes),
        cold["canonical_utf8_hex"]
            .as_str()
            .expect("cold state bytes")
    );
    assert_eq!(cold["owner_order"], machine_map["owner_order"]);

    let success_steps = vectors["success_steps"].as_array().expect("success steps");
    assert_eq!(success_steps.len(), 5);
    let normal_callbacks = u64::try_from(
        success_steps
            .iter()
            .filter(|step| step["trace"]["input"]["op"] == "meter_callback")
            .count(),
    )
    .expect("normal callback count fits U64");
    assert_eq!(normal_callbacks, 4);

    let mut prior_trace: Option<&Value> = None;
    for step in success_steps {
        let trace = &step["trace"];
        if let Some(previous) = prior_trace {
            assert_eq!(trace["before"], previous["after"]);
            assert_eq!(trace["before_state_sha256"], previous["after_state_sha256"]);
            assert_eq!(trace["owner_before_sha256"], previous["owner_after_sha256"]);
        }
        let result = validate_trace(trace, provider_sha256, normal_callbacks);
        assert_eq!(result, step["expected"]);
        assert_eq!(result["state_delta"], Value::Null);
        assert_eq!(result["commit_count"], 0);
        assert_eq!(result["receipt"]["production_allowed"], false);
        assert_eq!(result["receipt"]["state_delta_committed"], false);
        prior_trace = Some(trace);
    }

    let failure_cases = vectors["failure_cases"].as_array().expect("failure cases");
    assert_eq!(failure_cases.len(), 10);
    for failure in failure_cases {
        let result = validate_trace(&failure["trace"], provider_sha256, normal_callbacks);
        assert_eq!(result, failure["expected"], "failure {}", failure["id"]);
        assert_eq!(result["receipt"], Value::Null);
        assert_eq!(result["measurement"], Value::Null);
        assert_eq!(result["state_delta"], Value::Null);
        assert_eq!(result["commit_count"], 0);
    }

    let reset = &vectors["reset"];
    assert_eq!(reset["equals_cold"], true);
    assert_eq!(reset["state"], cold["state"]);
    assert_eq!(
        digest_hex(&serde_json::to_vec(&reset["state"]).unwrap()),
        cold["sha256"]
    );

    let eof = &vectors["eof"];
    assert_eq!(provider["meter_zero_drains"], eof["meter_zero_drains"]);
    assert_eq!(eof["private_flushes"], 0);
    assert_eq!(machine_map["eof"]["private_flushes"], 0);

    assert_eq!(vectors["expected_used_as_seed"], false);
    assert_eq!(vectors["fixture_scope"]["contains_pcm"], false);
    assert_eq!(vectors["fixture_scope"]["real_adm_render"], false);
    assert_eq!(vectors["fixture_scope"]["real_filter_execution"], false);
    assert_eq!(vectors["fixture_scope"]["seam_commit_count"], 0);
    assert_eq!(machine_map["fixture_seam"]["production_allowed"], false);
    assert_eq!(machine_map["fixture_seam"]["commit_count"], 0);
    assert_eq!(machine_map["fixture_seam"]["state_delta"], Value::Null);
    assert_eq!(independent["checks"].as_array().unwrap().len(), 23);
    assert!(
        independent["checks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|check| check["pass"] == true)
    );
    assert_eq!(independent["all_pass"], true);
    assert_eq!(independent["imports_fixture_validator"], false);
    assert_eq!(independent["independent_dsp_replay"], false);
    assert_eq!(independent["expected_used_as_seed"], false);
}
