// pattern: Imperative Shell
use std::collections::{BTreeMap, BTreeSet};

use jocforge_domain::joc::clip_producer::{
    CLIP_CHANNELS, CLIP_FRAME_SAMPLES, ClipFunctionalConstants, ClipFunctionalParameters,
    ClipProducerState, ClipSampleFrame,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const MANIFEST: &[u8] = include_bytes!("fixtures/native_clip_producer/WASH-MANIFEST.json");
const INDEPENDENT_CHECK: &[u8] =
    include_bytes!("fixtures/native_clip_producer/INDEPENDENT-CHECK.json");
const DELIVERY_ENVELOPE: &[u8] =
    include_bytes!("fixtures/native_clip_producer/DELIVERY-ENVELOPE.json");
const PAYLOADS: &[(&str, &[u8])] = &[
    (
        "check_clip_lineage.py",
        include_bytes!("fixtures/native_clip_producer/check_clip_lineage.py"),
    ),
    (
        "CHECK-NORMAL.json",
        include_bytes!("fixtures/native_clip_producer/CHECK-NORMAL.json"),
    ),
    (
        "CHECK-OPTIMIZED.json",
        include_bytes!("fixtures/native_clip_producer/CHECK-OPTIMIZED.json"),
    ),
    (
        "CONTRACT.md",
        include_bytes!("fixtures/native_clip_producer/CONTRACT.md"),
    ),
    (
        "FUNCTIONAL-PARAMETERS.json",
        include_bytes!("fixtures/native_clip_producer/FUNCTIONAL-PARAMETERS.json"),
    ),
    (
        "INPUT-ONLY.json",
        include_bytes!("fixtures/native_clip_producer/INPUT-ONLY.json"),
    ),
    (
        "MODEL-NATIVE-DIFFERENCES.json",
        include_bytes!("fixtures/native_clip_producer/MODEL-NATIVE-DIFFERENCES.json"),
    ),
    (
        "NATIVE-OBSERVATIONS.json",
        include_bytes!("fixtures/native_clip_producer/NATIVE-OBSERVATIONS.json"),
    ),
    (
        "PARAMETER-GENERATION-CHECK.json",
        include_bytes!("fixtures/native_clip_producer/PARAMETER-GENERATION-CHECK.json"),
    ),
    (
        "PROVENANCE.json",
        include_bytes!("fixtures/native_clip_producer/PROVENANCE.json"),
    ),
    (
        "SOURCE-IDENTITIES.json",
        include_bytes!("fixtures/native_clip_producer/SOURCE-IDENTITIES.json"),
    ),
    (
        "WASH-VALIDATION.json",
        include_bytes!("fixtures/native_clip_producer/WASH-VALIDATION.json"),
    ),
];

#[test]
fn washed_native_clip_package_keeps_exact_seals_and_parameter_provider() {
    verify_package(
        "5c7a332f844ef63952ab41041d84bea56e5846adea651796761f0ef48b2e9a51",
        "56c76403ac7b1edc60b11faa00ed66b6327e88f8e726047aa6479b8f1db63c4d",
        "5d8131af418516efee15e8e367ffb04929ea789199649748736828f742a08d1d",
    );
    let parameters: Value = serde_json::from_slice(PAYLOADS[4].1).unwrap();
    assert_eq!(parameters["threshold_words"].as_array().unwrap().len(), 128);
    assert_eq!(
        parameters["attenuation_words"].as_array().unwrap().len(),
        128
    );
    assert_eq!(
        parameters["classification"],
        "NON_SECRET_CLIP_FUNCTIONAL_CONSTANTS_ONLY"
    );
}

#[test]
fn all_synthetic_sample_generations_match_clip_state_and_compensation_receipts() {
    let parameters_json: Value = serde_json::from_slice(PAYLOADS[4].1).unwrap();
    let parameters = functional_parameters(&parameters_json);
    let input_cases: Vec<Value> = serde_json::from_slice(PAYLOADS[5].1).unwrap();
    let observations: Value = serde_json::from_slice(PAYLOADS[7].1).unwrap();
    let observations_by_name: BTreeMap<&str, &Value> = observations["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| (case["name"].as_str().unwrap(), case))
        .collect();
    assert_eq!(input_cases.len(), 393);
    assert_eq!(observations_by_name.len(), 393);

    let mut total_generations = 0_usize;
    let mut output_indices = BTreeSet::<u32>::new();
    for case in &input_cases {
        let name = case["name"].as_str().unwrap();
        let native_case = observations_by_name[name];
        let gain_exponent = i32::try_from(case["gain_exponent"].as_i64().unwrap()).unwrap();
        let mut state = ClipProducerState::cold(gain_exponent, &parameters).unwrap();
        if !case["initial"].is_null() {
            state = state
                .with_envelope_seed(
                    u32::try_from(case["initial"]["level_word"].as_u64().unwrap()).unwrap(),
                    u32::try_from(case["initial"]["hold"].as_u64().unwrap()).unwrap(),
                )
                .unwrap();
        }

        let frames = case["frames"].as_array().unwrap();
        let generations = native_case["generations"].as_array().unwrap();
        assert_eq!(frames.len(), generations.len(), "{name}");
        for (frame_index, (frame_recipe, generation)) in frames.iter().zip(generations).enumerate()
        {
            let frame = sample_frame(frame_recipe);
            let before = &generation["before"];
            assert_eq!(
                state.previous_tail_peak_word(),
                before["peak_word"],
                "{name}/{frame_index}"
            );
            assert_eq!(state.hold_counter(), before["hold"], "{name}/{frame_index}");
            assert_eq!(
                state.envelope_level_word(),
                before["level_word"],
                "{name}/{frame_index}"
            );
            assert_eq!(
                state.normalization_word(),
                before["normalization_word"],
                "{name}/{frame_index}"
            );
            assert_eq!(
                state.previous_attenuation_word(),
                before["previous_attenuation_word"],
                "{name}/{frame_index}"
            );
            assert_eq!(
                state.delay_read_index(),
                before["delay_read"],
                "{name}/{frame_index}"
            );
            assert_eq!(
                state.delay_write_index(),
                before["delay_write"],
                "{name}/{frame_index}"
            );

            let output = if case["kind"] == "wrapper" {
                state.process_wrapper(&frame, &parameters)
            } else {
                state.produce_clip_index(&frame, &parameters)
            }
            .unwrap_or_else(|error| panic!("{name}/{frame_index}: {error:?}"));
            assert_eq!(
                output.clip_index_u32(),
                generation["clip"],
                "{name}/{frame_index}"
            );
            assert!(
                (128..=255).contains(&output.clip_index_u32()),
                "{name}/{frame_index}"
            );
            if let Some(expected_rows_hash) =
                generation["synthetic_compensated_rows_sha256"].as_str()
            {
                assert_eq!(
                    compensated_frame_sha256(output.compensated_frame().unwrap()),
                    expected_rows_hash,
                    "{name}/{frame_index}"
                );
            } else {
                assert_eq!(case["kind"], "producer", "{name}/{frame_index}");
                assert!(output.compensated_frame().is_none(), "{name}/{frame_index}");
            }

            let after = &generation["after"];
            let next = output.after_state();
            assert_eq!(
                next.previous_tail_peak_word(),
                after["peak_word"],
                "{name}/{frame_index}"
            );
            assert_eq!(next.hold_counter(), after["hold"], "{name}/{frame_index}");
            assert_eq!(
                next.envelope_level_word(),
                after["level_word"],
                "{name}/{frame_index}"
            );
            assert_eq!(
                next.previous_attenuation_word(),
                after["previous_attenuation_word"],
                "{name}/{frame_index}"
            );
            assert_eq!(
                next.delay_read_index(),
                after["delay_read"],
                "{name}/{frame_index}"
            );
            assert_eq!(
                next.delay_write_index(),
                after["delay_write"],
                "{name}/{frame_index}"
            );

            let payload_hex = generation["payload_hex"].as_str().unwrap();
            let payload = decode_hex(payload_hex);
            assert_eq!(
                read_bits(&payload, 12, 8),
                output.clip_index_u32(),
                "clip writer field {name}/{frame_index}"
            );
            output_indices.insert(output.clip_index_u32());
            state = next.clone();
            total_generations += 1;
        }
    }

    assert_eq!(total_generations, 434);
    assert_eq!(output_indices.len(), 128);
    assert_eq!(output_indices.iter().next(), Some(&128));
    assert_eq!(output_indices.iter().next_back(), Some(&255));
}

fn functional_parameters(value: &Value) -> ClipFunctionalParameters {
    let thresholds = words::<128>(&value["threshold_words"]);
    let attenuation = words::<128>(&value["attenuation_words"]);
    let constants = &value["math_constants"];
    ClipFunctionalParameters::new(
        thresholds,
        attenuation,
        ClipFunctionalConstants {
            polynomial_a: u32::try_from(constants["polynomial_a"].as_u64().unwrap()).unwrap(),
            polynomial_b: u32::try_from(constants["polynomial_b"].as_u64().unwrap()).unwrap(),
            polynomial_c: u32::try_from(constants["polynomial_c"].as_u64().unwrap()).unwrap(),
            exponent_scale: u32::try_from(constants["exponent_scale"].as_u64().unwrap()).unwrap(),
            release_step: u32::try_from(constants["release_step"].as_u64().unwrap()).unwrap(),
            level_bias: u32::try_from(constants["level_bias"].as_u64().unwrap()).unwrap(),
        },
    )
    .unwrap()
}

fn words<const N: usize>(value: &Value) -> [u32; N] {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn sample_frame(recipe: &Value) -> Box<ClipSampleFrame> {
    let mut frame: Box<ClipSampleFrame> = vec![[0.0_f32; CLIP_FRAME_SAMPLES]; CLIP_CHANNELS]
        .into_boxed_slice()
        .try_into()
        .expect("fixture always declares six channel rows");
    for spike in recipe["spikes"].as_array().unwrap() {
        let fields = spike.as_array().unwrap();
        let channel = usize::try_from(fields[0].as_u64().unwrap()).unwrap();
        let sample = usize::try_from(fields[1].as_u64().unwrap()).unwrap();
        let word = u32::try_from(fields[2].as_u64().unwrap()).unwrap();
        frame[channel][sample] = f32::from_bits(word);
    }
    frame
}

fn compensated_frame_sha256(frame: &[[f32; CLIP_FRAME_SAMPLES]]) -> String {
    let mut digest = Sha256::new();
    for row in frame {
        for sample in row {
            digest.update(sample.to_bits().to_le_bytes());
        }
    }
    hex_digest(digest.finalize())
}

fn decode_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = char::from(pair[0]).to_digit(16).unwrap();
            let low = char::from(pair[1]).to_digit(16).unwrap();
            u8::try_from((high << 4) | low).unwrap()
        })
        .collect()
}

fn read_bits(bytes: &[u8], start: usize, width: u8) -> u32 {
    (0..width).fold(0_u32, |value, offset| {
        let bit_index = start + usize::from(offset);
        (value << 1) | u32::from((bytes[bit_index / 8] >> (7 - (bit_index % 8))) & 1)
    })
}

fn verify_package(manifest_sha256: &str, check_sha256: &str, envelope_sha256: &str) {
    assert_eq!(hex_digest(Sha256::digest(MANIFEST)), manifest_sha256);
    assert_eq!(hex_digest(Sha256::digest(INDEPENDENT_CHECK)), check_sha256);
    assert_eq!(
        hex_digest(Sha256::digest(DELIVERY_ENVELOPE)),
        envelope_sha256
    );
    let manifest: Value = serde_json::from_slice(MANIFEST).unwrap();
    let check: Value = serde_json::from_slice(INDEPENDENT_CHECK).unwrap();
    let envelope: Value = serde_json::from_slice(DELIVERY_ENVELOPE).unwrap();
    assert_eq!(check["manifest_sha256"], manifest_sha256);
    assert_eq!(check["status"], "PASS");
    assert_eq!(
        manifest["payloads"].as_array().unwrap().len(),
        PAYLOADS.len()
    );
    let files: BTreeMap<&str, &[u8]> = PAYLOADS
        .iter()
        .map(|(name, bytes)| (*name, *bytes))
        .collect();
    for entry in manifest["payloads"].as_array().unwrap() {
        let name = entry["file"].as_str().unwrap();
        let bytes = files[name];
        assert_eq!(
            entry["bytes"].as_u64().unwrap(),
            u64::try_from(bytes.len()).unwrap(),
            "{name}"
        );
        assert_eq!(entry["sha256"], hex_digest(Sha256::digest(bytes)), "{name}");
    }
    let bindings: BTreeMap<&str, &Value> = envelope["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|binding| (binding["file"].as_str().unwrap(), binding))
        .collect();
    for (name, bytes, hash) in [
        ("MANIFEST.json", MANIFEST, manifest_sha256),
        ("INDEPENDENT-CHECK.json", INDEPENDENT_CHECK, check_sha256),
    ] {
        assert_eq!(
            bindings[name]["bytes"].as_u64().unwrap(),
            u64::try_from(bytes.len()).unwrap()
        );
        assert_eq!(bindings[name]["sha256"], hash);
    }
}

fn hex_digest(digest: impl AsRef<[u8]>) -> String {
    use std::fmt::Write as _;

    let mut value = String::with_capacity(digest.as_ref().len() * 2);
    for byte in digest.as_ref() {
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    value
}
