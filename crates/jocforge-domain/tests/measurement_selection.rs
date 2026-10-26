use jocforge_domain::meter::measurement_receipt::ProgrammeGatingKind;
use jocforge_domain::meter::measurement_selection::{
    Id1PreferredValue, MeasurementAggregate, MeasurementRecordCandidate,
    id1_preferred_from_measurement_i32, processor_value_to_measurement_i32,
};
use jocforge_domain::numeric::FiniteF32;
use serde_json::Value;
use sha2::{Digest, Sha256};

const INPUT_ONLY_VECTORS: &str =
    include_str!("fixtures/measurement_selection/input-only-vectors.json");
const INPUT_ONLY_VECTORS_SHA256: &str =
    "4128af1949606008688427092f7b6a503e772a5dda2dd9bd768984d734eee062";
const AGGREGATION_INPUTS: &str =
    include_str!("fixtures/measurement_selection/aggregation-input-only-vectors.json");
const AGGREGATION_INPUTS_SHA256: &str =
    "3bf4a13488c10c9460a99e50e2d64a819c2da1af1d722cc58edd629f5c0d5ff4";

fn vectors() -> Value {
    assert_eq!(
        format!("{:x}", Sha256::digest(INPUT_ONLY_VECTORS.as_bytes())),
        INPUT_ONLY_VECTORS_SHA256
    );
    serde_json::from_str(INPUT_ONLY_VECTORS).expect("hash-bound input-only vectors")
}

fn vector_i32(value: &Value, field: &str) -> i32 {
    i32::try_from(
        value[field]
            .as_i64()
            .expect("signed I32 value in input-only vector"),
    )
    .expect("input-only vector value fits I32")
}

fn vector_u32(value: &Value, field: &str) -> u32 {
    u32::try_from(value[field].as_u64().expect("U32 value in vector"))
        .expect("vector value fits U32")
}

fn vector_record(value: &Value, field: &str) -> [u8; 44] {
    let hex = value[field].as_str().expect("opaque record hex");
    assert_eq!(hex.len(), 88);
    std::array::from_fn(|index| {
        u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).expect("record byte hex")
    })
}

fn aggregation_vectors() -> Value {
    assert_eq!(
        format!("{:x}", Sha256::digest(AGGREGATION_INPUTS.as_bytes())),
        AGGREGATION_INPUTS_SHA256
    );
    serde_json::from_str(AGGREGATION_INPUTS).expect("hash-bound aggregation input recipes")
}

#[test]
fn strict_i32_booleanization_matches_all_input_only_controls() {
    let vectors = vectors();
    for case in vectors["strict_i32_cases"].as_array().unwrap() {
        assert_eq!(
            id1_preferred_from_measurement_i32(vector_i32(case, "source_i32")),
            vector_i32(case, "id1_preferred_i32")
        );
    }
}

#[test]
fn processor_i32_remap_then_booleanization_matches_all_input_only_controls() {
    let vectors = vectors();
    for case in vectors["processor_tag_cases"].as_array().unwrap() {
        let measurement_i32 =
            processor_value_to_measurement_i32(vector_i32(case, "processor_raw_i32"));
        assert_eq!(measurement_i32, vector_i32(case, "measurement_result_i32"));
        assert_eq!(
            id1_preferred_from_measurement_i32(measurement_i32),
            vector_i32(case, "id1_preferred_i32")
        );
    }
}

#[test]
fn public_result_domain_separates_internal_level_from_id1_preferred() {
    let cases = [
        (ProgrammeGatingKind::Ungated, 0, 0),
        (ProgrammeGatingKind::Dialogue, 1, 1),
        (ProgrammeGatingKind::Level, 2, 0),
    ];
    for (semantic_kind, public_kind, preferred) in cases {
        let public_result_kind = semantic_kind.public_result_kind();
        assert_eq!(public_result_kind.value(), public_kind);
        assert_eq!(
            Id1PreferredValue::from_public_result_kind(public_result_kind).value(),
            preferred
        );
    }
}

#[test]
fn aggregate_keeps_zero_without_winner_and_retains_first_strict_tie_winner() {
    let vectors = aggregation_vectors();
    assert_eq!(vectors["input_only"], true);
    assert_eq!(
        vectors["record_layout"],
        "OPAQUE_44_BYTES_MEMBER_CARRIED_SEPARATELY_NO_OFFSET_ASSERTED"
    );

    for case in vectors["cases"].as_array().unwrap() {
        let initial_record = vector_record(case, "initial_record_hex");
        let initial_comparator_bits = vector_u32(case, "initial_comparator_f32_bits");
        let initial_member = vector_i32(case, "initial_member_i32");
        let mut aggregate = MeasurementAggregate::cold();
        assert_eq!(aggregate.record_bytes(), &initial_record);
        assert_eq!(
            aggregate.loudness_comparator_bits(),
            initial_comparator_bits
        );
        assert_eq!(aggregate.opaque_i32_member(), initial_member);

        let mut expected_record = initial_record;
        let mut expected_comparator_bits = initial_comparator_bits;
        let mut expected_member = initial_member;
        let mut expected_wins = Vec::new();

        for candidate in case["candidates"].as_array().unwrap() {
            let record = vector_record(candidate, "record_hex");
            let loudness_bits = vector_u32(candidate, "loudness_f32_bits");
            let loudness = FiniteF32::new(f32::from_bits(loudness_bits)).unwrap();
            let wins = loudness.value() > f32::from_bits(expected_comparator_bits);
            let (next, did_win) = aggregate.consider(MeasurementRecordCandidate::new(
                record,
                loudness,
                vector_i32(candidate, "member_i32"),
            ));
            assert_eq!(did_win, wins, "{}", case["id"].as_str().unwrap());
            expected_wins.push(did_win);
            if wins {
                expected_record = record;
                expected_comparator_bits = loudness_bits;
                expected_member = vector_i32(candidate, "member_i32");
            }
            aggregate = next;
            assert_eq!(aggregate.record_bytes(), &expected_record);
            assert_eq!(
                aggregate.loudness_comparator_bits(),
                expected_comparator_bits
            );
            assert_eq!(aggregate.opaque_i32_member(), expected_member);
        }

        let expected_case_wins: &[bool] = match case["id"].as_str().unwrap() {
            "initial_comparator_tie_no_winner" => &[false],
            "strict_winner_then_equal_tie" => &[true, false],
            "later_strict_winner" => &[true, true],
            "empty_candidates" => &[],
            other => panic!("unexpected aggregation input recipe: {other}"),
        };
        assert_eq!(expected_wins, expected_case_wins);
        assert_eq!(
            id1_preferred_from_measurement_i32(aggregate.opaque_i32_member()),
            i32::from(aggregate.opaque_i32_member() == 1)
        );
        assert_eq!(
            aggregate.has_winning_record(),
            expected_wins.iter().any(|won| *won)
        );
    }
}
