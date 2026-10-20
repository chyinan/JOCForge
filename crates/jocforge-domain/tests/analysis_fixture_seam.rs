// pattern: Functional Core; fixture-only analysis receipt validator
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

const FIXTURE_MANIFEST_SHA256: &str =
    "0fd24d23ca4237e8bdd2283671293dfa0b54755ea74e84316234b4cc4c195912";
const PROVIDER_FILE: &str = "analysis-selected-provider.json";
const VECTORS_FILE: &str = "ANALYSIS-OWNER-VECTORS.json";
const SOURCE_IDENTITIES: &[(&str, u64, &str)] = &[
    (
        "input",
        10_558,
        "59f7e83438bd1e7bd955cb4a9fe9c6dd7717939a651eb150d190edb2d91432d5",
    ),
    (
        "result",
        2_871_144,
        "ada7e443a21299dbdbc527f05f705c65cbe2b674d785b20ad8b339aa7347766a",
    ),
    (
        "provider_manifest",
        3_905,
        "2bde77646052a86b626c70d486069ddacec9231ea216fc6b44ef9a045401b1b9",
    ),
    (
        "state_schema",
        5_359,
        "7967752a88493648c24cf7c1183da61dc470e35a684bb48d6335c496e90c6f4e",
    ),
];
const OWNER_KEYS: &[&str] = &[
    "input_history",
    "transient",
    "primary_companion_lfe",
    "policy",
];

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/analysis_fixture")
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

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn decode_lower_hex(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    if bytes.len() % 2 != 0 {
        return None;
    }
    bytes
        .chunks_exact(2)
        .map(|pair| Some((hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?))
        .collect()
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
    let bytes = fs::read(root.join("ANALYSIS-FIXTURE-MANIFEST.json"))
        .expect("analysis fixture manifest exists");
    assert_eq!(digest_hex(&bytes), FIXTURE_MANIFEST_SHA256);
    let manifest: Value = serde_json::from_slice(&bytes).expect("manifest JSON");
    let files = manifest["files"].as_array().expect("manifest files array");
    assert_eq!(files.len(), 11);
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

fn is_integer(value: &Value) -> bool {
    value.as_i64().is_some() || value.as_u64().is_some()
}

fn is_u32(value: &Value) -> bool {
    value
        .as_u64()
        .is_some_and(|number| u32::try_from(number).is_ok())
}

fn is_u16(value: &Value) -> bool {
    value
        .as_u64()
        .is_some_and(|number| u16::try_from(number).is_ok())
}

fn u32_words(value: &Value, expected_len: usize) -> bool {
    value
        .as_array()
        .is_some_and(|words| words.len() == expected_len && words.iter().all(is_u32))
}

fn u16_words(value: &Value, expected_len: usize) -> bool {
    value
        .as_array()
        .is_some_and(|words| words.len() == expected_len && words.iter().all(is_u16))
}

fn i1_words(value: &Value, expected_len: usize) -> bool {
    value.as_array().is_some_and(|words| {
        words.len() == expected_len
            && words.iter().all(|word| {
                word.as_i64()
                    .is_some_and(|number| (-1..=1).contains(&number))
            })
    })
}

fn i1_projection_words(value: &Value, expected_len: usize) -> bool {
    value.as_array().is_some_and(|words| {
        words.len() == expected_len
            && words
                .iter()
                .all(|word| word.as_u64().is_some_and(|number| number <= 1))
    })
}

fn owner_projection(state: &Value, owner: &str) -> Option<Value> {
    match owner {
        "input_history" => Some(json!({
            "generation": state["input_history_generation"],
            "history_u32_words": state["input_history_u32_words"],
        })),
        "transient" => Some(json!({
            "generation": state["transient_generation"],
            "two_history_u32_words": state["transient_two_history_u32_words"],
            "three_history_u32_words": state["transient_three_history_u32_words"],
            "peak_statistics_u32_words": state["transient_peak_statistics_u32_words"],
            "previous_mean_u32_words": state["transient_previous_mean_u32_words"],
            "short_queue_i1": state["transient_short_queue_i1"],
            "other_queue_i1": state["transient_other_queue_i1"],
        })),
        "primary_companion_lfe" => Some(json!({
            "generation": state["transform_generation"],
            "primary_fbw_history_u32_words": state["primary_fbw_history_u32_words"],
            "companion_fbw_history_u32_words": state["companion_fbw_history_u32_words"],
            "primary_lfe_history_u32_words": state["primary_lfe_history_u32_words"],
            "companion_lfe_history_u32_words": state["companion_lfe_history_u32_words"],
        })),
        "policy" => Some(json!({
            "generation": state["policy_generation"],
            "gain_db_history_u32_words": state["gain_db_history_u32_words"],
            "exponent_refresh_history_u16_words": state["exponent_refresh_history_u16_words"],
            "normalized_anchor_u16_words": state["normalized_anchor_u16_words"],
        })),
        _ => None,
    }
}

fn owner_projection_shape_valid(owner: &str, value: &Value) -> bool {
    match owner {
        "input_history" => {
            has_exact_keys(value, &["generation", "history_u32_words"])
                && value["generation"].as_u64().is_some()
                && u32_words(&value["history_u32_words"], 7680)
        }
        "transient" => {
            has_exact_keys(
                value,
                &[
                    "generation",
                    "two_history_u32_words",
                    "three_history_u32_words",
                    "peak_statistics_u32_words",
                    "previous_mean_u32_words",
                    "short_queue_i1",
                    "other_queue_i1",
                ],
            ) && value["generation"].as_u64().is_some()
                && u32_words(&value["two_history_u32_words"], 20)
                && u32_words(&value["three_history_u32_words"], 30)
                && u32_words(&value["peak_statistics_u32_words"], 90)
                && u32_words(&value["previous_mean_u32_words"], 5)
                && i1_projection_words(&value["short_queue_i1"], 20)
                && i1_projection_words(&value["other_queue_i1"], 20)
        }
        "primary_companion_lfe" => {
            has_exact_keys(
                value,
                &[
                    "generation",
                    "primary_fbw_history_u32_words",
                    "companion_fbw_history_u32_words",
                    "primary_lfe_history_u32_words",
                    "companion_lfe_history_u32_words",
                ],
            ) && value["generation"].as_u64().is_some()
                && u32_words(&value["primary_fbw_history_u32_words"], 640)
                && u32_words(&value["companion_fbw_history_u32_words"], 640)
                && u32_words(&value["primary_lfe_history_u32_words"], 128)
                && u32_words(&value["companion_lfe_history_u32_words"], 128)
        }
        "policy" => {
            has_exact_keys(
                value,
                &[
                    "generation",
                    "gain_db_history_u32_words",
                    "exponent_refresh_history_u16_words",
                    "normalized_anchor_u16_words",
                ],
            ) && value["generation"].as_u64().is_some()
                && u32_words(&value["gain_db_history_u32_words"], 64)
                && u16_words(&value["exponent_refresh_history_u16_words"], 30)
                && u16_words(&value["normalized_anchor_u16_words"], 30)
        }
        _ => false,
    }
}

fn append_canonical_projection(value: &Value, output: &mut Vec<u8>) {
    match value {
        Value::Null => output.extend_from_slice(b"null"),
        Value::Bool(boolean) => output.extend_from_slice(if *boolean { b"true" } else { b"false" }),
        Value::Number(_) | Value::String(_) => {
            output.extend_from_slice(serde_json::to_string(value).unwrap().as_bytes());
        }
        Value::Array(items) => {
            output.push(b'[');
            for (index, item) in items.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                append_canonical_projection(item, output);
            }
            output.push(b']');
        }
        Value::Object(object) => {
            let mut entries = object.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            output.push(b'{');
            for (index, (key, item)) in entries.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                output.extend_from_slice(serde_json::to_string(key).unwrap().as_bytes());
                output.push(b':');
                append_canonical_projection(item, output);
            }
            output.push(b'}');
        }
    }
}

fn canonical_projection_bytes(value: &Value) -> Vec<u8> {
    let mut output = Vec::new();
    append_canonical_projection(value, &mut output);
    output
}

fn canonical_projection_sha256(value: &Value) -> String {
    digest_hex(&canonical_projection_bytes(value))
}

fn assert_source_zero_projection_vectors(
    projection_vectors: &Value,
    before_state: &Value,
    after_state: &Value,
) {
    assert_eq!(
        projection_vectors["schema"],
        "analysis-owner-projection-hash-vectors.v1"
    );
    assert_eq!(projection_vectors["expected_used_as_seed"], false);
    assert_eq!(projection_vectors["source_identity"]["bytes"], 2_871_144);
    assert_eq!(
        projection_vectors["source_identity"]["sha256"],
        "ada7e443a21299dbdbc527f05f705c65cbe2b674d785b20ad8b339aa7347766a"
    );
    let vectors = projection_vectors["vectors"].as_array().unwrap();
    assert_eq!(vectors.len(), 8);
    for vector in vectors {
        let owner = vector["owner"].as_str().unwrap();
        let side = vector["side"].as_str().unwrap();
        let state = match side {
            "before" => before_state,
            "after" => after_state,
            _ => panic!("unknown projection side"),
        };
        assert_eq!(vector["source"], 0);
        assert_eq!(vector["id"], format!("source0_{side}_{owner}"));
        let projection = owner_projection(state, owner).expect("known analysis owner");
        assert!(owner_projection_shape_valid(owner, &projection));
        assert_eq!(vector["value"], projection);

        let canonical_bytes = canonical_projection_bytes(&projection);
        let preimage = decode_lower_hex(vector["preimage_hex"].as_str().unwrap())
            .expect("owner projection preimage uses lowercase hex");
        assert_eq!(preimage, canonical_bytes);
        assert_eq!(
            preimage.len() as u64,
            vector["preimage_bytes"].as_u64().unwrap()
        );
        assert_eq!(digest_hex(&preimage), vector["sha256"]);
        assert_eq!(vector["frozen_row_sha256"], vector["sha256"]);
        assert_eq!(vector["canonicalization"]["encoding"], "UTF-8");
        assert_eq!(vector["canonicalization"]["sort_keys"], true);
        assert_eq!(vector["canonicalization"]["separators"], json!([",", ":"]));
        assert_eq!(vector["canonicalization"]["ensure_ascii"], false);
        assert_eq!(vector["canonicalization"]["domain_prefix"], Value::Null);
    }
}

fn analysis_state_shape_valid(state: &Value) -> bool {
    const STATE_KEYS: [&str; 21] = [
        "source",
        "retired",
        "failure_stage",
        "input_history_generation",
        "input_history_u32_words",
        "transient_generation",
        "transient_two_history_u32_words",
        "transient_three_history_u32_words",
        "transient_peak_statistics_u32_words",
        "transient_previous_mean_u32_words",
        "transient_short_queue_i1",
        "transient_other_queue_i1",
        "transform_generation",
        "primary_fbw_history_u32_words",
        "companion_fbw_history_u32_words",
        "primary_lfe_history_u32_words",
        "companion_lfe_history_u32_words",
        "policy_generation",
        "gain_db_history_u32_words",
        "exponent_refresh_history_u16_words",
        "normalized_anchor_u16_words",
    ];
    has_exact_keys(state, &STATE_KEYS)
        && state["source"].as_u64().is_some()
        && state["retired"].is_boolean()
        && (state["failure_stage"].is_null() || state["failure_stage"].is_string())
        && state["input_history_generation"].as_u64().is_some()
        && u32_words(&state["input_history_u32_words"], 7680)
        && state["transient_generation"].as_u64().is_some()
        && u32_words(&state["transient_two_history_u32_words"], 20)
        && u32_words(&state["transient_three_history_u32_words"], 30)
        && u32_words(&state["transient_peak_statistics_u32_words"], 90)
        && u32_words(&state["transient_previous_mean_u32_words"], 5)
        && i1_words(&state["transient_short_queue_i1"], 20)
        && i1_words(&state["transient_other_queue_i1"], 20)
        && state["transform_generation"].as_u64().is_some()
        && u32_words(&state["primary_fbw_history_u32_words"], 640)
        && u32_words(&state["companion_fbw_history_u32_words"], 640)
        && u32_words(&state["primary_lfe_history_u32_words"], 128)
        && u32_words(&state["companion_lfe_history_u32_words"], 128)
        && state["policy_generation"].as_u64().is_some()
        && u32_words(&state["gain_db_history_u32_words"], 64)
        && u16_words(&state["exponent_refresh_history_u16_words"], 30)
        && u16_words(&state["normalized_anchor_u16_words"], 30)
}

fn validate_record(record: &Value, provider_sha256: &str) -> Value {
    const SHARED_KEYS: [&str; 13] = [
        "schema",
        "source",
        "generation",
        "provider_sha256",
        "before_sha256",
        "after_sha256",
        "advancement_count",
        "input_shape",
        "short_shape",
        "finite",
        "audio_recipe_sha256",
        "spectra_sha256",
        "owner_after_sha256",
    ];
    const NUMERIC_KEYS: [&str; 12] = [
        "schema",
        "source",
        "generation",
        "provider_sha256",
        "shared_receipt_sha256",
        "six_stage_complete",
        "power_sha256",
        "expanded_exponents_sha256",
        "plan_sha256",
        "main_frame_codes",
        "converter_frame_codes",
        "converter_snr_offset",
    ];

    let before_state = &record["before_state"];
    let after_state = &record["after_state"];
    if !analysis_state_shape_valid(before_state) || !analysis_state_shape_valid(after_state) {
        return error_result("ANALYSIS_STATE_SHAPE");
    }
    let Some(source) = record["source"].as_u64() else {
        return error_result("ANALYSIS_SOURCE");
    };
    if before_state["source"].as_u64() != Some(source) {
        return error_result("ANALYSIS_SOURCE");
    }
    let Some(generation) = source.checked_add(1) else {
        return error_result("ANALYSIS_GENERATION");
    };

    let shared = &record["shared_receipt"];
    let numeric = &record["numeric_receipt"];
    if !has_exact_keys(shared, &SHARED_KEYS)
        || !has_exact_keys(numeric, &NUMERIC_KEYS)
        || shared["schema"] != "shared-analysis-receipt.v1"
        || numeric["schema"] != "numeric-plan-receipt.v1"
    {
        return error_result("ANALYSIS_LINEAGE");
    }
    if before_state["input_history_generation"].as_u64() != Some(source)
        || before_state["transient_generation"].as_u64() != Some(source)
        || before_state["transform_generation"].as_u64() != Some(source)
        || before_state["policy_generation"].as_u64() != Some(source)
        || shared["source"].as_u64() != Some(source)
        || numeric["source"].as_u64() != Some(source)
        || shared["generation"].as_u64() != Some(generation)
        || numeric["generation"].as_u64() != Some(generation)
    {
        return error_result("ANALYSIS_GENERATION");
    }
    if shared["provider_sha256"] != provider_sha256 || numeric["provider_sha256"] != provider_sha256
    {
        return error_result("ANALYSIS_PROVIDER");
    }

    if shared["input_shape"] != json!([6, 1536])
        || shared["short_shape"] != json!([6, 5])
        || shared["finite"] != true
    {
        return error_result("ANALYSIS_INPUT");
    }
    if shared["advancement_count"] != 1 {
        return error_result("ANALYSIS_ADVANCEMENT");
    }

    if after_state["source"].as_u64() != source.checked_add(1)
        || after_state["retired"] != false
        || !after_state["failure_stage"].is_null()
        || after_state["input_history_generation"].as_u64() != Some(generation)
        || after_state["transient_generation"].as_u64() != Some(generation)
        || after_state["transform_generation"].as_u64() != Some(generation)
        || after_state["policy_generation"].as_u64() != Some(generation)
    {
        return error_result("ANALYSIS_AFTER");
    }
    if shared["before_sha256"] != canonical_json_sha256(before_state)
        || shared["after_sha256"] != canonical_json_sha256(after_state)
    {
        return error_result("ANALYSIS_AFTER");
    }

    let owner_hashes = &shared["owner_after_sha256"];
    if !has_exact_keys(owner_hashes, OWNER_KEYS)
        || !OWNER_KEYS.iter().all(|key| is_sha256(&owner_hashes[*key]))
        || !is_sha256(&shared["audio_recipe_sha256"])
        || !is_sha256(&shared["spectra_sha256"])
    {
        return error_result("ANALYSIS_LINEAGE");
    }
    for owner in OWNER_KEYS {
        let Some(projection) = owner_projection(after_state, owner) else {
            return error_result("ANALYSIS_LINEAGE");
        };
        if !owner_projection_shape_valid(owner, &projection)
            || owner_hashes[*owner] != canonical_projection_sha256(&projection)
        {
            return error_result("ANALYSIS_LINEAGE");
        }
    }
    if numeric["shared_receipt_sha256"] != canonical_json_sha256(shared) {
        return error_result("ANALYSIS_LINEAGE");
    }

    let stage_completion = numeric["six_stage_complete"].as_array();
    if !stage_completion
        .is_some_and(|stages| stages.len() == 6 && stages.iter().all(|stage| stage == true))
    {
        return error_result("ANALYSIS_PLAN_INCOMPLETE");
    }
    if !is_sha256(&numeric["power_sha256"])
        || !is_sha256(&numeric["expanded_exponents_sha256"])
        || !is_sha256(&numeric["plan_sha256"])
        || numeric["main_frame_codes"]
            .as_array()
            .is_none_or(|codes| codes.len() != 5 || !codes.iter().all(is_integer))
        || numeric["converter_frame_codes"]
            .as_array()
            .is_none_or(|codes| codes.len() != 5 || !codes.iter().all(is_integer))
        || !is_integer(&numeric["converter_snr_offset"])
        || numeric["main_frame_codes"] == numeric["converter_frame_codes"]
    {
        return error_result("ANALYSIS_PLAN_SHAPE");
    }

    let receipt = json!({
        "schema": "analysis-fixture-validation-receipt.v1",
        "production_allowed": false,
        "source": source,
        "generation": generation,
        "provider_sha256": provider_sha256,
        "before_sha256": shared["before_sha256"],
        "recorded_after_sha256": shared["after_sha256"],
        "shared_receipt_sha256": canonical_json_sha256(shared),
        "numeric_plan_receipt_sha256": canonical_json_sha256(numeric),
        "state_delta_committed": false,
        "commit_count": 0,
    });
    json!({
        "status": "PASS",
        "receipt": receipt,
        "state_delta": null,
        "commit_count": 0,
    })
}

#[test]
fn shared_analysis_and_numeric_plan_receipts_are_bound_without_commit() {
    assert_fixture_manifest();
    let provider = read_fixture(PROVIDER_FILE);
    let machine_map = read_fixture("ANALYSIS-MACHINE-MAP.json");
    let vectors = read_fixture(VECTORS_FILE);
    let independent = read_fixture("INDEPENDENT-CHECK.json");
    let projection_contract = read_fixture("ANALYSIS-OWNER-PROJECTIONS.schema.json");
    let projection_vectors = read_fixture("ANALYSIS-OWNER-PROJECTION-VECTORS.json");
    let projection_independent = read_fixture("OWNER-PROJECTION-INDEPENDENT-CHECK.json");
    let provider_sha256 = vectors["provider"]["canonical_sha256"]
        .as_str()
        .expect("provider canonical SHA-256");
    assert_eq!(canonical_json_sha256(&provider), provider_sha256);
    assert_eq!(provider, vectors["provider"]["value"]);
    assert_eq!(provider["production_allowed"], false);
    assert_eq!(provider["profile_id"], "selected-f07-f11-5.1-48k.v1");
    assert_eq!(provider["expected_sources"], 4249);

    for (name, byte_count, sha256) in SOURCE_IDENTITIES {
        assert_eq!(vectors["source_identities"][name]["bytes"], *byte_count);
        assert_eq!(vectors["source_identities"][name]["sha256"], *sha256);
    }

    let cold = &vectors["cold"];
    let cold_state = &cold["state"];
    assert!(analysis_state_shape_valid(cold_state));
    let cold_bytes = serde_json::to_vec(cold_state).expect("cold state canonical JSON");
    assert_eq!(hex(&cold_bytes), cold["canonical_utf8_hex"]);
    assert_eq!(digest_hex(&cold_bytes), cold["sha256"]);
    assert_eq!(cold_state["source"], 0);

    let success_records = vectors["success_records"]
        .as_array()
        .expect("success records");
    assert_eq!(success_records.len(), 3);
    assert_eq!(
        projection_vectors["source_identity"]["bytes"],
        vectors["source_identities"]["result"]["bytes"]
    );
    assert_eq!(
        projection_vectors["source_identity"]["sha256"],
        vectors["source_identities"]["result"]["sha256"]
    );

    let projection_defs = &projection_contract["$defs"];
    assert_eq!(projection_contract["$id"], "analysis-owner-projections.v1");
    assert_eq!(projection_defs["u64"]["minimum"], 0);
    assert_eq!(projection_defs["u32"]["minimum"], 0);
    assert_eq!(projection_defs["u32"]["maximum"], 4_294_967_295_u64);
    assert_eq!(projection_defs["u16"]["minimum"], 0);
    assert_eq!(projection_defs["u16"]["maximum"], 65_535);
    assert_eq!(projection_defs["i1"]["minimum"], 0);
    assert_eq!(projection_defs["i1"]["maximum"], 1);
    for (owner, required_fields) in [
        ("input_history", json!(["generation", "history_u32_words"])),
        (
            "transient",
            json!([
                "generation",
                "two_history_u32_words",
                "three_history_u32_words",
                "peak_statistics_u32_words",
                "previous_mean_u32_words",
                "short_queue_i1",
                "other_queue_i1"
            ]),
        ),
        (
            "primary_companion_lfe",
            json!([
                "generation",
                "primary_fbw_history_u32_words",
                "companion_fbw_history_u32_words",
                "primary_lfe_history_u32_words",
                "companion_lfe_history_u32_words"
            ]),
        ),
        (
            "policy",
            json!([
                "generation",
                "gain_db_history_u32_words",
                "exponent_refresh_history_u16_words",
                "normalized_anchor_u16_words"
            ]),
        ),
    ] {
        assert_eq!(projection_defs[owner]["type"], "object");
        assert_eq!(projection_defs[owner]["additionalProperties"], false);
        assert_eq!(projection_defs[owner]["required"], required_fields);
    }
    assert_source_zero_projection_vectors(
        &projection_vectors,
        &success_records[0]["record"]["before_state"],
        &success_records[0]["record"]["after_state"],
    );

    let projection_checks = projection_independent["checks"].as_array().unwrap();
    assert_eq!(projection_checks.len(), 9);
    assert!(projection_checks.iter().all(|check| check["pass"] == true));
    assert_eq!(projection_independent["all_pass"], true);
    assert_eq!(projection_independent["imports_analysis_model"], false);
    assert_eq!(projection_independent["expected_used_as_seed"], false);

    let mut previous_after_state = cold_state.clone();
    for (expected_source, step) in success_records.iter().enumerate() {
        let record = &step["record"];
        assert_eq!(record["before_state"], previous_after_state);
        assert_eq!(
            record["source"].as_u64(),
            Some(u64::try_from(expected_source).unwrap())
        );
        let result = validate_record(record, provider_sha256);
        assert_eq!(result, step["expected"]);
        assert_eq!(result["state_delta"], Value::Null);
        assert_eq!(result["commit_count"], 0);
        assert_eq!(result["receipt"]["production_allowed"], false);
        assert_eq!(result["receipt"]["state_delta_committed"], false);
        previous_after_state = record["after_state"].clone();
    }

    for (receipt_key, invalid_schema) in [
        ("shared_receipt", "unknown-shared-schema.v1"),
        ("numeric_receipt", "unknown-numeric-schema.v1"),
    ] {
        let mut record = success_records[0]["record"].clone();
        record[receipt_key]["schema"] = json!(invalid_schema);
        let result = validate_record(&record, provider_sha256);
        assert_eq!(result["status"], "ERROR");
        assert_eq!(result["error"], "ANALYSIS_LINEAGE");
        assert_eq!(result["receipt"], Value::Null);
        assert_eq!(result["state_delta"], Value::Null);
        assert_eq!(result["commit_count"], 0);
    }

    for owner in OWNER_KEYS {
        let mut record = success_records[0]["record"].clone();
        record["shared_receipt"]["owner_after_sha256"][*owner] = json!("0".repeat(64));
        let result = validate_record(&record, provider_sha256);
        assert_eq!(result["status"], "ERROR");
        assert_eq!(result["error"], "ANALYSIS_LINEAGE");
        assert_eq!(result["receipt"], Value::Null);
        assert_eq!(result["state_delta"], Value::Null);
        assert_eq!(result["commit_count"], 0);
    }

    let failure_cases = vectors["failure_cases"].as_array().expect("failure cases");
    assert_eq!(failure_cases.len(), 12);
    for failure in failure_cases {
        let result = validate_record(&failure["record"], provider_sha256);
        assert_eq!(result, failure["expected"], "failure {}", failure["id"]);
        assert_eq!(result["receipt"], Value::Null);
        assert_eq!(result["state_delta"], Value::Null);
        assert_eq!(result["commit_count"], 0);
    }

    let fatal = &vectors["fatal_fixture"];
    assert_eq!(fatal["id"], "S3-ALL-CANDIDATES-FAIL");
    assert_eq!(fatal["analysis_numeric_state_unchanged"], true);
    let mut fatal_after_without_lifecycle = fatal["after_state"].clone();
    let mut fatal_before_without_lifecycle = fatal["before_state"].clone();
    for state in [
        &mut fatal_after_without_lifecycle,
        &mut fatal_before_without_lifecycle,
    ] {
        let fields = state.as_object_mut().unwrap();
        fields.remove("retired");
        fields.remove("failure_stage");
    }
    assert_eq!(
        fatal_after_without_lifecycle,
        fatal_before_without_lifecycle
    );
    assert_eq!(fatal["before_state"]["retired"], false);
    assert_eq!(fatal["after_state"]["retired"], true);
    assert_eq!(
        fatal["after_state"]["failure_stage"],
        "all_candidate_trials"
    );

    assert_eq!(vectors["reset"]["equals_cold"], true);
    assert_eq!(&vectors["reset"]["state"], cold_state);
    assert_eq!(
        digest_hex(&serde_json::to_vec(&vectors["reset"]["state"]).unwrap()),
        cold["sha256"]
    );
    assert_eq!(vectors["eof"]["expected_sources"], 4249);
    assert_eq!(vectors["eof"]["extra_analysis_flush"], 0);
    assert_eq!(
        vectors["consumer_boundaries"],
        json!({
            "F06ND": "SharedAnalysisReceipt only",
            "F12": "SharedAnalysisReceipt + NumericPlanReceipt",
            "F05JL": "no analysis ownership or direct advance"
        })
    );
    assert_eq!(
        machine_map["consumers"],
        json!({
            "F06ND": "SharedAnalysisReceipt",
            "F12": "SharedAnalysisReceipt + NumericPlanReceipt",
            "F05JL": "no direct ownership"
        })
    );
    assert_eq!(machine_map["fixture_seam"]["production_allowed"], false);
    assert_eq!(machine_map["fixture_seam"]["commit_count"], 0);
    assert_eq!(machine_map["fixture_seam"]["state_delta"], Value::Null);
    assert_eq!(machine_map["blocked"].as_array().unwrap().len(), 3);
    assert_eq!(vectors["real_adm_execution"], "NOT_RUN");
    assert_eq!(vectors["actual_fft_transform"], "NOT_RUN");
    assert_eq!(vectors["expected_used_as_seed"], false);
    assert_eq!(independent["checks"].as_array().unwrap().len(), 25);
    assert!(
        independent["checks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|check| check["pass"] == true)
    );
    assert_eq!(independent["all_pass"], true);
    assert_eq!(independent["imports_fixture_validator"], false);
    assert_eq!(independent["independent_transform_replay"], false);
    assert_eq!(independent["expected_used_as_seed"], false);
}
