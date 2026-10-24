// pattern: Functional Core
use super::{
    PrefixMembershipError, PrefixMembershipParameters, PrefixMembershipProducer,
    PrefixMembershipResult, hex_lower,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const INPUTS: &str =
    include_str!("../../../tests/fixtures/primary_installed48/MEMBERSHIP-INPUT-ONLY.json");
const EXPECTED: &str =
    include_str!("../../../tests/fixtures/primary_installed48/MEMBERSHIP-EXPECTED.json");

#[test]
fn selected_synthetic_gain_cases_match_after_computation() {
    let inputs: Value = serde_json::from_str(INPUTS).unwrap();
    let producer = PrefixMembershipProducer::new(
        Some(PrefixMembershipParameters::selected_authorized()),
        Some(synthetic_basis(&inputs)),
    )
    .unwrap();
    let produced_cases = inputs["full_gain_cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| {
            (
                case["case_id"].as_str().unwrap().to_owned(),
                producer.produce(&input_record(&case["input"])).unwrap(),
            )
        })
        .collect::<Vec<_>>();

    let expected: Value = serde_json::from_str(EXPECTED).unwrap();
    let expected_cases = expected["full_gain_cases"].as_array().unwrap();
    assert_eq!(produced_cases.len(), expected_cases.len());
    for (case_id, produced) in &produced_cases {
        let expected_case = expected_cases
            .iter()
            .find(|case| case["case_id"].as_str() == Some(case_id.as_str()))
            .unwrap();
        assert_gain_matches(produced, &expected_case["expected"]);
    }
}

#[test]
fn authorized_primitive_recipes_match_postcomputed_hashes() {
    let inputs: Value = serde_json::from_str(INPUTS).unwrap();
    let producer = PrefixMembershipProducer::new(
        Some(PrefixMembershipParameters::selected_authorized()),
        Some(synthetic_basis(&inputs)),
    )
    .unwrap();
    let mut recipe_outputs = Vec::new();
    for recipe in inputs["recipes"].as_array().unwrap() {
        let id = recipe["id"].as_str().unwrap();
        let first = recipe["q_first"].as_u64().unwrap() as i32;
        let last = recipe["q_last"].as_u64().unwrap() as i32;
        let mut digest = Sha256::new();
        let mut count = 0_u64;
        for q in first..=last {
            let value = q as f32 / 32_768.0;
            match id {
                "WIDTH-Q15" => {
                    let mapped =
                        super::map_width(&producer.parameters, super::q15_to_f32(q).unwrap())
                            .unwrap();
                    hash_word(&mut digest, mapped);
                    count += 1;
                }
                "POWER-FIXED" => {
                    for exponent in [2.0_f32, 6.0_f32] {
                        hash_word(
                            &mut digest,
                            super::pow_value(&producer.parameters, value, exponent).unwrap(),
                        );
                        count += 1;
                    }
                }
                "POWER-MAPPED" => {
                    let mapped =
                        super::map_width(&producer.parameters, super::q15_to_f32(q).unwrap())
                            .unwrap();
                    let exponent = super::power(&producer.parameters, mapped).unwrap();
                    let reciprocal = super::m32(1.0_f64 / f64::from(exponent)).unwrap();
                    for power in [exponent, reciprocal] {
                        hash_word(
                            &mut digest,
                            super::pow_value(&producer.parameters, value, power).unwrap(),
                        );
                        count += 1;
                    }
                }
                _ => panic!("unexpected input-only membership recipe"),
            }
        }
        recipe_outputs.push((id.to_owned(), count, hex_lower(&digest.finalize())));
    }

    let expected: Value = serde_json::from_str(EXPECTED).unwrap();
    for (id, count, digest) in recipe_outputs {
        let case = expected["recipes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"].as_str() == Some(id.as_str()))
            .unwrap();
        assert_eq!(count, case["count"].as_u64().unwrap());
        assert_eq!(digest, case["output_u32le_sha256"].as_str().unwrap());
    }
}

#[test]
fn invalid_protocol_controls_and_missing_supply_fail_closed() {
    let inputs: Value = serde_json::from_str(INPUTS).unwrap();
    let basis = synthetic_basis(&inputs);
    assert_eq!(
        PrefixMembershipProducer::new(None, Some(basis)).unwrap_err(),
        PrefixMembershipError::ParameterUnavailable
    );
    assert_eq!(
        PrefixMembershipProducer::new(
            Some(PrefixMembershipParameters::selected_authorized()),
            None
        )
        .unwrap_err(),
        PrefixMembershipError::ParameterUnavailable
    );
    assert_eq!(
        PrefixMembershipProducer::new(
            Some(PrefixMembershipParameters::selected_authorized()),
            Some([[0.5, 0.5, 1.1]; 11])
        )
        .unwrap_err(),
        PrefixMembershipError::ParameterDomain
    );

    let producer = PrefixMembershipProducer::new(
        Some(PrefixMembershipParameters::selected_authorized()),
        Some(basis),
    )
    .unwrap();
    assert_eq!(
        producer.produce(&[0; 31]).unwrap_err(),
        PrefixMembershipError::Shape
    );
    let mut unsupported_mode = input_record(&inputs["full_gain_cases"][0]["input"]);
    unsupported_mode[12..16].copy_from_slice(&1_u32.to_le_bytes());
    assert_eq!(
        producer.produce(&unsupported_mode).unwrap_err(),
        PrefixMembershipError::ControlDomain
    );
    let mut nonfinite_xyz = input_record(&inputs["full_gain_cases"][0]["input"]);
    nonfinite_xyz[..4].copy_from_slice(&0x7fc0_0000_u32.to_le_bytes());
    assert_eq!(
        producer.produce(&nonfinite_xyz).unwrap_err(),
        PrefixMembershipError::InputNonfinite
    );
}

#[test]
fn positive_widths_that_map_below_threshold_are_rejected() {
    let inputs: Value = serde_json::from_str(INPUTS).unwrap();
    let producer = PrefixMembershipProducer::new(
        Some(PrefixMembershipParameters::selected_authorized()),
        Some(synthetic_basis(&inputs)),
    )
    .unwrap();
    let mut errors = Vec::new();
    for rejection in inputs["rejections"].as_array().unwrap() {
        let xyz = rejection["xyz_rationals"]
            .as_array()
            .unwrap()
            .iter()
            .map(rational_word)
            .collect::<Vec<_>>();
        let record = make_input_record(
            xyz.try_into().unwrap(),
            0,
            rational_word(&rejection["width_rational"]),
            0,
            0,
            1,
        );
        errors.push(producer.produce(&record).unwrap_err());
    }

    for (error, rejection) in errors.iter().zip(inputs["rejections"].as_array().unwrap()) {
        assert_eq!(error.code(), rejection["expected_error"].as_str().unwrap());
    }
}

fn synthetic_basis(input: &Value) -> [[f32; 3]; 11] {
    input["synthetic_basis_points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|point| {
            point
                .as_array()
                .unwrap()
                .iter()
                .map(|coordinate| coordinate.as_f64().unwrap() as f32)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap()
        })
        .collect::<Vec<[f32; 3]>>()
        .try_into()
        .unwrap()
}

fn input_record(input: &Value) -> Vec<u8> {
    let xyz: [u32; 3] = input["xyz_words"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::from_str_radix(value.as_str().unwrap(), 16).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    make_input_record(
        xyz,
        input["uniform_width_mode"].as_u64().unwrap() as u32,
        u32::from_str_radix(input["raw_width_word"].as_str().unwrap(), 16).unwrap(),
        input["extra"].as_u64().unwrap() as u32,
        input["component_enum"].as_u64().unwrap() as u32,
        input["enabled"].as_u64().unwrap() as u32,
    )
}

fn make_input_record(
    xyz: [u32; 3],
    mode: u32,
    width: u32,
    extra: u32,
    component_enum: u32,
    enabled: u32,
) -> Vec<u8> {
    xyz.into_iter()
        .chain([mode, width, extra, component_enum, enabled])
        .flat_map(u32::to_le_bytes)
        .collect()
}

fn rational_word(value: &Value) -> u32 {
    let pair = value.as_array().unwrap();
    (pair[0].as_i64().unwrap() as f32 / pair[1].as_i64().unwrap() as f32).to_bits()
}

fn hash_word(digest: &mut Sha256, value: f32) {
    digest.update(value.to_bits().to_le_bytes());
}

fn assert_gain_matches(produced: &PrefixMembershipResult, expected_gain: &Value) {
    assert_eq!(
        *produced.axis_words(),
        expected_words_3x11(&expected_gain["axis_words"])
    );
    assert_eq!(
        *produced.boundary_words(),
        expected_words_11(&expected_gain["boundary_words"])
    );
    assert_eq!(
        *produced.interior_words(),
        expected_words_11(&expected_gain["interior_words"])
    );
    assert_eq!(
        *produced.spread_words(),
        expected_words_11(&expected_gain["spread_words"])
    );
    assert_eq!(
        *produced.column_words(),
        expected_words_11(&expected_gain["column_words"])
    );
    assert_eq!(
        produced.sin_squared_word(),
        expected_gain["sin2_word"].as_u64().unwrap() as u32
    );
    assert_eq!(
        *produced.unadjusted_words(),
        expected_words_11(&expected_gain["unadjusted_words"])
    );
    assert_eq!(
        *produced.gain_words(),
        expected_words_11(&expected_gain["final_words"])
    );
    assert_eq!(
        produced.positive_members(),
        expected_indices(&expected_gain["positive_members"])
    );
    let expected_cells = expected_gain["cell_words"].as_array().unwrap();
    assert_eq!(produced.cells().len(), expected_cells.len());
    for (actual, expected) in produced.cells().iter().zip(expected_cells) {
        assert_eq!(actual.axis(), expected["axis"].as_u64().unwrap() as u8);
        assert_eq!(actual.plane(), expected["plane"].as_u64().unwrap() as u8);
        assert_eq!(
            actual.position_index(),
            expected["position_index"].as_u64().unwrap() as u8
        );
        assert_eq!(
            actual.width_index(),
            expected["width_index"].as_u64().unwrap() as u8
        );
        assert_eq!(actual.words(), expected_words_dynamic(&expected["words"]));
    }
}

fn expected_words_3x11(value: &Value) -> [[u32; 11]; 3] {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(expected_words_11)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn expected_words_11(value: &Value) -> [u32; 11] {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|word| word.as_u64().unwrap() as u32)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn expected_words_dynamic(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|word| word.as_u64().unwrap() as u32)
        .collect()
}

fn expected_indices(value: &Value) -> Vec<u8> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|index| index.as_u64().unwrap() as u8)
        .collect()
}
