// pattern: Imperative Shell
use std::collections::BTreeMap;

use jocforge_domain::joc::coefficient_payload::{
    CoefficientPayloadInput, CoefficientPayloadObject, CoefficientPayloadPoint,
    CoefficientPayloadWriterFailure, write_selected_id14_coefficient_payload,
};
use jocforge_domain::joc::huffman::Id14WriterReturnCode;
use serde_json::Value;
use sha2::{Digest, Sha256};

const MANIFEST: &[u8] =
    include_bytes!("fixtures/id14_coefficient_payload_writer/WASH-MANIFEST.json");
const INDEPENDENT_CHECK: &[u8] =
    include_bytes!("fixtures/id14_coefficient_payload_writer/INDEPENDENT-CHECK.json");
const DELIVERY_ENVELOPE: &[u8] =
    include_bytes!("fixtures/id14_coefficient_payload_writer/DELIVERY-ENVELOPE.json");
const PAYLOADS: &[(&str, &[u8])] = &[
    (
        "approved-codebooks.json",
        include_bytes!("fixtures/id14_coefficient_payload_writer/approved-codebooks.json"),
    ),
    (
        "check_payload_writer.py",
        include_bytes!("fixtures/id14_coefficient_payload_writer/check_payload_writer.py"),
    ),
    (
        "CHECK-NORMAL.json",
        include_bytes!("fixtures/id14_coefficient_payload_writer/CHECK-NORMAL.json"),
    ),
    (
        "CHECK-OPTIMIZED.json",
        include_bytes!("fixtures/id14_coefficient_payload_writer/CHECK-OPTIMIZED.json"),
    ),
    (
        "CONTRACT.md",
        include_bytes!("fixtures/id14_coefficient_payload_writer/CONTRACT.md"),
    ),
    (
        "FROZEN-MODEL-POSTCOMPARE.json",
        include_bytes!("fixtures/id14_coefficient_payload_writer/FROZEN-MODEL-POSTCOMPARE.json"),
    ),
    (
        "INPUT-ONLY.json",
        include_bytes!("fixtures/id14_coefficient_payload_writer/INPUT-ONLY.json"),
    ),
    (
        "NATIVE-CODEBOOK-POSTCOMPARE.json",
        include_bytes!("fixtures/id14_coefficient_payload_writer/NATIVE-CODEBOOK-POSTCOMPARE.json"),
    ),
    (
        "NATIVE-OBSERVATIONS.json",
        include_bytes!("fixtures/id14_coefficient_payload_writer/NATIVE-OBSERVATIONS.json"),
    ),
    (
        "SOURCE-IDENTITIES.json",
        include_bytes!("fixtures/id14_coefficient_payload_writer/SOURCE-IDENTITIES.json"),
    ),
    (
        "WASH-VALIDATION.json",
        include_bytes!("fixtures/id14_coefficient_payload_writer/WASH-VALIDATION.json"),
    ),
];

#[test]
fn washed_package_and_codebooks_keep_their_approved_identities() {
    verify_package(
        "4af8b1c919d7627749bc5dd10610c1268830185c90b1c28e41aba71320c03315",
        "b79f939c7bec7b74969ebfdcad39b4ea1635ac082c0fdd18afb2941b56144f05",
        "c5b4d0448270b0dfcf9e96f416db6bd6cab3c94c4d08307d30559dbb72c1c035",
    );
    assert_eq!(
        sha256(PAYLOADS[0].1),
        "515d635adc6099493dfd27f787bdf2b9f19a4b4bb1180860b0e820a23fb5d5ae"
    );
    assert_eq!(
        PAYLOADS[0].1,
        include_bytes!("../src/joc/huffman/codebooks.json")
    );

    let wash: Value = serde_json::from_slice(PAYLOADS[10].1).unwrap();
    assert_eq!(
        wash["status"],
        "SCOPED_WRITER_SLICE_READY_UPSTREAM_AND_FULL_F13C_NOT_READY"
    );
    assert_eq!(
        wash["independent_validation"]["normal_checker"],
        "15 cases, 588 codebook symbols, PASS"
    );
    assert_eq!(
        wash["independent_validation"]["optimized_checker"],
        "15 cases, 588 codebook symbols, PASS"
    );
}

#[test]
fn successful_writer_transfers_exact_owned_body_and_explicit_generation() {
    use jocforge_domain::timeline::CodecSourceIndex;

    let inputs: Vec<Value> = serde_json::from_slice(PAYLOADS[6].1).unwrap();
    let mut successes = 0;
    let mut failures = 0;
    let mut scratch_failures = 0;
    let mut writer_failures = 0;
    for case in &inputs {
        let Ok(output) = write_selected_id14_coefficient_payload(&coefficient_input(case)) else {
            continue;
        };
        let expected = output.payload().map(<[u8]>::to_vec);
        let length = output.output_length_bytes();
        let original = output.clone();
        let generation = CodecSourceIndex::new(23);
        let body = output.try_into_metadata_body(generation);
        match (expected, body) {
            (Some(expected), Ok(body)) => {
                assert_eq!(body.writer_generation, generation);
                assert_eq!(body.bytes, expected);
                assert_eq!(Some(body.bytes.len() as u64), length);
                successes += 1;
            }
            (None, Err(outcome)) => {
                assert_eq!(outcome, original);
                assert!(outcome.descriptor_copy().is_none());
                if outcome.coefficient_scratch_exceeded() {
                    assert_eq!(outcome.writer_return_code(), None);
                    scratch_failures += 1;
                } else {
                    assert!(outcome.writer_return_code().is_some());
                    writer_failures += 1;
                }
                failures += 1;
            }
            _ => panic!("writer success and metadata admission disagree"),
        }
    }
    assert!(successes > 0);
    assert!(failures > 0);
    assert!(scratch_failures > 0);
    assert!(writer_failures > 0);
}

#[test]
fn controlled_inputs_match_model_payloads_and_native_writer_boundaries() {
    let inputs: Vec<Value> = serde_json::from_slice(PAYLOADS[6].1).unwrap();
    let model: Value = serde_json::from_slice(PAYLOADS[5].1).unwrap();
    let native: Value = serde_json::from_slice(PAYLOADS[8].1).unwrap();
    let model_rows: BTreeMap<&str, &Value> = model["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["name"].as_str().unwrap(), row))
        .collect();
    let native_rows: BTreeMap<&str, &Value> = native["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["name"].as_str().unwrap(), row))
        .collect();
    assert_eq!(inputs.len(), 15);
    assert_eq!(model_rows.len(), 15);
    assert_eq!(native_rows.len(), 15);

    for case in &inputs {
        let name = case["name"].as_str().unwrap();
        let input = coefficient_input(case);
        let model_row = model_rows[name];
        let native_row = native_rows[name];

        if name == "offset_at_slot_limit" {
            assert_eq!(model_row["model"], "REJECTED");
            assert_eq!(native_row["writer_return"], 0);
            assert_eq!(native_row["writer_status"], 0);
            assert_eq!(
                write_selected_id14_coefficient_payload(&input).unwrap_err(),
                CoefficientPayloadWriterFailure::OffsetOutsideSelectedWindow
            );
            continue;
        }

        let output = write_selected_id14_coefficient_payload(&input)
            .unwrap_or_else(|error| panic!("{name}: {error:?}"));
        let expected_body_bits = model_row["coefficient_bits"]
            .as_u64()
            .or_else(|| native_row["body_bits"].as_u64())
            .unwrap_or(0);
        assert_eq!(
            output.body_bit_count().value(),
            expected_body_bits,
            "{name}"
        );

        if name == "coefficient_scratch_overflow" {
            assert!(output.coefficient_scratch_exceeded(), "{name}");
            assert_eq!(output.writer_return_code(), None, "{name}");
            assert_eq!(output.output_length_bytes(), None, "{name}");
            assert!(output.payload().is_none(), "{name}");
            assert!(output.descriptor_copy().is_none(), "{name}");
            continue;
        }

        assert_eq!(
            output.writer_return_code().map(Id14WriterReturnCode::value),
            native_row["writer_return"]
                .as_u64()
                .map(|value| u8::try_from(value).unwrap()),
            "{name}"
        );
        assert_eq!(
            output.writer_status_word(),
            Some(native_row["writer_status"].as_u64().unwrap() as u32),
            "{name}"
        );
        assert_eq!(
            output.output_length_bytes(),
            Some(native_row["length_in_out"].as_u64().unwrap()),
            "{name}"
        );

        if native_row["writer_return"] != 0 {
            assert!(
                output.payload().is_none(),
                "failed writes do not commit output: {name}"
            );
            assert!(
                output.descriptor_copy().is_none(),
                "failed writes do not publish a descriptor: {name}"
            );
            continue;
        }

        if model_row["model"] == "ENCODED" && native_row["native_payload_match"] != false {
            let payload = output.payload().unwrap();
            assert_eq!(
                sha256(payload),
                model_row["payload_sha256"].as_str().unwrap(),
                "{name}"
            );
            let descriptor = output.descriptor_copy().unwrap();
            assert_eq!(descriptor.payload_id(), 14, "{name}");
            assert_eq!(
                descriptor.length_bytes(),
                u64::try_from(payload.len()).unwrap(),
                "{name}"
            );
            assert_eq!(descriptor.body(), payload, "{name}");
            assert_ne!(
                descriptor.body().as_ptr(),
                payload.as_ptr(),
                "cache copy owns distinct storage: {name}"
            );
        }
    }
}

#[test]
fn non_finite_coefficients_are_rejected_before_writer_output_creation() {
    let inputs: Vec<Value> = serde_json::from_slice(PAYLOADS[6].1).unwrap();
    let mut input = coefficient_input(&inputs[0]);
    input.objects[0].points[0].channel_rows[0][0] = f32::NAN;
    assert_eq!(
        write_selected_id14_coefficient_payload(&input).unwrap_err(),
        CoefficientPayloadWriterFailure::NonFiniteCoefficient
    );
}

fn coefficient_input(case: &Value) -> CoefficientPayloadInput {
    let objects = case["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|object| {
            let selected = object["selected"].as_array().unwrap();
            let points = object["points"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(point_index, point)| CoefficientPayloadPoint {
                    channel_rows: point
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|row| {
                            row.as_array()
                                .unwrap()
                                .iter()
                                .map(|value| value.as_f64().unwrap() as f32)
                                .collect()
                        })
                        .collect(),
                    selected_channels: selected[point_index]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|channel| u8::try_from(channel.as_u64().unwrap()).unwrap())
                        .collect(),
                })
                .collect();
            let offsets = if object["offsets"].is_null() {
                None
            } else {
                Some(
                    object["offsets"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|offset| u8::try_from(offset.as_u64().unwrap()).unwrap())
                        .collect(),
                )
            };

            CoefficientPayloadObject {
                present: object["present"].as_bool().unwrap(),
                band_enum: u8::try_from(object["band_enum"].as_u64().unwrap()).unwrap(),
                mode: u8::try_from(object["mode"].as_u64().unwrap()).unwrap(),
                points,
                offsets,
            }
        })
        .collect();

    CoefficientPayloadInput {
        downmix_index: u8::try_from(case["downmix"].as_u64().unwrap()).unwrap(),
        extension_config: u8::try_from(case["extension"].as_u64().unwrap()).unwrap(),
        clip_gain_index: u8::try_from(case["clip"].as_u64().unwrap()).unwrap(),
        sequence_count: u16::try_from(case["sequence"].as_u64().unwrap()).unwrap(),
        time_slot_limit: u8::try_from(case["max_offset"].as_u64().unwrap()).unwrap(),
        capacity_bytes: u16::try_from(case["capacity"].as_u64().unwrap()).unwrap(),
        objects,
    }
}

fn verify_package(manifest_sha256: &str, check_sha256: &str, envelope_sha256: &str) {
    assert_eq!(sha256(MANIFEST), manifest_sha256);
    assert_eq!(sha256(INDEPENDENT_CHECK), check_sha256);
    assert_eq!(sha256(DELIVERY_ENVELOPE), envelope_sha256);

    let manifest: Value = serde_json::from_slice(MANIFEST).unwrap();
    let check: Value = serde_json::from_slice(INDEPENDENT_CHECK).unwrap();
    let envelope: Value = serde_json::from_slice(DELIVERY_ENVELOPE).unwrap();
    assert_eq!(check["manifest_sha256"], manifest_sha256);
    assert_eq!(check["status"], "PASS");
    assert_eq!(
        manifest["payloads"].as_array().unwrap().len(),
        PAYLOADS.len()
    );

    let payloads: BTreeMap<&str, &[u8]> = PAYLOADS
        .iter()
        .map(|(name, bytes)| (*name, *bytes))
        .collect();
    for entry in manifest["payloads"].as_array().unwrap() {
        let name = entry["file"].as_str().unwrap();
        let bytes = payloads[name];
        assert_eq!(
            entry["bytes"].as_u64().unwrap(),
            u64::try_from(bytes.len()).unwrap(),
            "{name}"
        );
        assert_eq!(entry["sha256"], sha256(bytes), "{name}");
    }

    let bindings: BTreeMap<&str, &Value> = envelope["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|binding| (binding["file"].as_str().unwrap(), binding))
        .collect();
    for (name, bytes, digest) in [
        ("MANIFEST.json", MANIFEST, manifest_sha256),
        ("INDEPENDENT-CHECK.json", INDEPENDENT_CHECK, check_sha256),
    ] {
        assert_eq!(
            bindings[name]["bytes"].as_u64().unwrap(),
            u64::try_from(bytes.len()).unwrap()
        );
        assert_eq!(bindings[name]["sha256"], digest);
    }
}

fn sha256(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut value = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    value
}
