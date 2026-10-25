use jocforge_domain::drc::none_drc_override_fifo::{
    NoneDrcOverrideFifoError, NoneDrcOverrideFifoState, ZeroLogicalRecord,
    transition_none_drc_override_fifo,
};
use jocforge_domain::timeline::CodecSourceIndex;

#[test]
fn cold_and_per_source_transitions_keep_four_logical_zero_records() {
    let mut state = NoneDrcOverrideFifoState::cold();
    assert_eq!(state.records(), &[ZeroLogicalRecord; 4]);
    assert_eq!(state.metadata_delay_samples(), 3_072);
    assert_eq!(state.append_slot(), 3);
    assert_eq!(state.frames_processed(), 0);

    for source in 0..8 {
        let transition =
            transition_none_drc_override_fifo(&state, CodecSourceIndex::new(source)).unwrap();
        let (consumed_record, next_state) = transition.into_parts();
        assert_eq!(consumed_record, ZeroLogicalRecord);
        state = next_state;
        assert_eq!(state.records(), &[ZeroLogicalRecord; 4]);
        assert_eq!(state.frames_processed(), source + 1);
    }
}

#[test]
fn fifo_rejects_a_source_generation_gap_without_advancing() {
    let before = NoneDrcOverrideFifoState::cold();
    assert_eq!(
        transition_none_drc_override_fifo(&before, CodecSourceIndex::new(1)),
        Err(NoneDrcOverrideFifoError::SourceGenerationMismatch),
    );
    assert_eq!(before, NoneDrcOverrideFifoState::cold());
    assert_eq!(
        NoneDrcOverrideFifoError::SourceGenerationMismatch.outcome_code(),
        "DRC_INPUT_ERROR",
    );
}
