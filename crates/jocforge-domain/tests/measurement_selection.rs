use jocforge_domain::meter::measurement_selection::{
    MeasurementAggregate, MeasurementRecordCandidate, id1_preferred_from_measurement_i32,
    processor_value_to_measurement_i32,
};
use jocforge_domain::numeric::FiniteF32;
use serde_json::Value;
use sha2::{Digest, Sha256};

const INPUT_ONLY_VECTORS: &str =
    include_str!("fixtures/measurement_selection/input-only-vectors.json");
const INPUT_ONLY_VECTORS_SHA256: &str =
    "4128af1949606008688427092f7b6a503e772a5dda2dd9bd768984d734eee062";

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
fn aggregate_keeps_zero_without_winner_and_retains_first_strict_tie_winner() {
    let vectors = vectors();
    let aggregate_cases = vectors["aggregate_cases"].as_array().unwrap();
    let no_winner = aggregate_cases
        .iter()
        .find(|case| case["case"] == "no_winning_record")
        .unwrap();
    let cold = MeasurementAggregate::cold();
    assert_eq!(cold.record_bytes(), &[0; 44]);
    assert_eq!(
        cold.opaque_i32_member(),
        vector_i32(no_winner, "selected_record_i32")
    );
    assert!(!cold.has_winning_record());
    assert_eq!(
        id1_preferred_from_measurement_i32(cold.opaque_i32_member()),
        vector_i32(no_winner, "id1_preferred_i32")
    );
    assert_eq!(cold.loudness_comparator_bits(), (-120.0_f32).to_bits());

    let (at_threshold, did_win) = cold.consider(MeasurementRecordCandidate::new(
        [0x11; 44],
        FiniteF32::new(-120.0).unwrap(),
        1,
    ));
    assert!(!did_win);
    assert_eq!(at_threshold.record_bytes(), &[0; 44]);
    assert_eq!(at_threshold.opaque_i32_member(), 0);

    let strict_tie = aggregate_cases
        .iter()
        .find(|case| case["case"] == "strict_winner_then_equal_tie")
        .unwrap();
    let initial = at_threshold;
    let (winner, did_win) = initial.consider(MeasurementRecordCandidate::new(
        [0x2a; 44],
        FiniteF32::new(-20.0).unwrap(),
        vector_i32(strict_tie, "selected_record_i32"),
    ));
    assert!(did_win);
    assert!(winner.has_winning_record());
    assert_eq!(winner.record_bytes(), &[0x2a; 44]);

    let (after_tie, did_win) = winner.consider(MeasurementRecordCandidate::new(
        [0x7b; 44],
        FiniteF32::new(-20.0).unwrap(),
        1,
    ));
    assert!(!did_win);
    assert_eq!(after_tie.record_bytes(), &[0x2a; 44]);
    assert_eq!(
        after_tie.opaque_i32_member(),
        vector_i32(strict_tie, "selected_record_i32")
    );
    assert_eq!(
        id1_preferred_from_measurement_i32(after_tie.opaque_i32_member()),
        vector_i32(strict_tie, "id1_preferred_i32")
    );
}
