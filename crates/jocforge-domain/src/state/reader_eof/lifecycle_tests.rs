use super::{
    Control38Snapshot, Host38Frame, ReaderEofDigest, ReaderEofError, ReaderEofMachine,
    ReaderEofPhase, ReaderEofState,
};
use crate::timeline::{FrontendCallbackIndex, NormalCallbackCount};

fn next_state(current: &ReaderEofState) -> ReaderEofState {
    ReaderEofState::from_parts(
        FrontendCallbackIndex::new(current.callback_index().value() + 1),
        current.source_cursor_samples() + 1_536,
        NormalCallbackCount::new(current.normal_callbacks().value() + 1),
        current.remaining_cached_drains(),
        None,
        None,
        ReaderEofPhase::Normal,
    )
    .unwrap()
}

#[test]
fn generated_cold_constructor_has_exact_fields_and_serialization() {
    let machine = ReaderEofMachine::cold();
    let state = machine.committed();
    assert_eq!(machine.generation(), 0);
    assert_eq!(state.callback_index().value(), 0);
    assert_eq!(state.source_cursor_samples(), 0);
    assert_eq!(state.normal_callbacks().value(), 0);
    assert_eq!(state.remaining_cached_drains(), 4);
    assert!(state.last_normal_host38().is_none());
    assert!(state.last_control_snapshot().is_none());
    assert_eq!(state.phase(), ReaderEofPhase::Unset);
    let mut expected = vec![0_u8; 31];
    expected[24] = 4;
    assert_eq!(state.canonical_bytes(), expected);
    assert_eq!(
        state.digest(),
        ReaderEofDigest::from_canonical_bytes(&state.canonical_bytes())
    );
    assert_eq!(
        state.digest().bytes(),
        [
            0x8b, 0x33, 0x91, 0xd6, 0x21, 0xf8, 0x23, 0xe0, 0x1c, 0xbd, 0xa7, 0xdc, 0x93, 0x5c,
            0x1b, 0x65, 0x9b, 0x87, 0x79, 0xb5, 0xf4, 0x88, 0xd1, 0x23, 0xa4, 0x0a, 0x2a, 0x75,
            0x3d, 0xa9, 0xdb, 0xd9
        ]
    );
}

#[test]
fn owner_state_reference_tracks_complete_cold_and_committed_bytes() {
    let mut machine = ReaderEofMachine::cold();
    let cold_state = ReaderEofState::cold();
    let cold_reference = machine.state_reference();

    assert_eq!(cold_reference.owner_id(), "reader_eof");
    assert_eq!(cold_reference.state_ref(), "owners.reader_eof");
    assert_eq!(cold_reference.generation(), 0);
    assert_eq!(
        cold_reference.canonical_bytes(),
        cold_state.canonical_bytes()
    );
    assert_eq!(cold_reference.state_sha256(), cold_state.digest());
    assert_eq!(cold_reference.cold_sha256(), cold_state.digest());

    let staged = machine
        .stage(
            machine
                .prepare("tx-state-reference", next_state(machine.committed()))
                .unwrap(),
        )
        .unwrap();
    machine.commit(staged).unwrap();

    let committed_reference = machine.state_reference();
    assert_eq!(committed_reference.generation(), 1);
    assert_eq!(
        committed_reference.canonical_bytes(),
        machine.committed().canonical_bytes()
    );
    assert_eq!(
        committed_reference.state_sha256(),
        machine.committed().digest()
    );
    assert_eq!(committed_reference.cold_sha256(), cold_state.digest());
    assert_eq!(
        cold_reference.canonical_bytes(),
        cold_state.canonical_bytes()
    );
}

#[test]
fn canonical_options_use_presence_then_row_major_little_endian_payload() {
    let mut host = vec![0.0_f32; 38 * 1_536];
    host[0] = -0.0;
    host[38 * 1_536 - 1] = 1.0;
    let mut control = vec![0_u32; 38 * 13];
    control[0] = 0x1122_3344;
    let state = ReaderEofState::from_parts(
        FrontendCallbackIndex::new(1),
        1_536,
        NormalCallbackCount::new(1),
        4,
        Some(Host38Frame::new(host).unwrap()),
        Some(Control38Snapshot::new(control).unwrap()),
        ReaderEofPhase::Normal,
    )
    .unwrap();
    let bytes = state.canonical_bytes();
    assert_eq!(&bytes[0..8], &1_u64.to_le_bytes());
    assert_eq!(bytes[28], 1);
    assert_eq!(&bytes[29..33], &0x8000_0000_u32.to_le_bytes());
    let control_presence = 29 + 38 * 1_536 * 4;
    assert_eq!(bytes[control_presence], 1);
    assert_eq!(
        &bytes[control_presence + 1..control_presence + 5],
        &0x1122_3344_u32.to_le_bytes()
    );
    assert_eq!(*bytes.last().unwrap(), ReaderEofPhase::Normal as u8);
}

#[test]
fn prepare_stage_abort_and_commit_preserve_transaction_rules() {
    let mut machine = ReaderEofMachine::cold();
    let before = machine.committed().digest();
    let prepared = machine
        .prepare("tx-abort", next_state(machine.committed()))
        .unwrap();
    assert_eq!(machine.generation(), 0);
    assert_eq!(machine.committed().digest(), before);
    let staged = machine.stage(prepared).unwrap();
    assert_eq!(machine.committed().digest(), before);
    machine.abort(staged).unwrap();
    assert_eq!(machine.generation(), 0);
    assert_eq!(machine.committed().digest(), before);

    let commit_staged = machine
        .stage(
            machine
                .prepare("tx-commit", next_state(machine.committed()))
                .unwrap(),
        )
        .unwrap();
    let receipt = machine.commit(commit_staged).unwrap();
    assert_eq!(receipt.owner_id().as_str(), "reader_eof");
    assert_eq!(
        (receipt.before_generation(), receipt.after_generation()),
        (0, 1)
    );
    assert_eq!((receipt.changed(), receipt.commit_count()), (true, 1));
    assert_eq!(receipt.transaction_id(), "tx-commit");
    assert_eq!(receipt.before_digest(), before);
    assert_eq!(receipt.after_digest(), machine.committed().digest());
    assert_eq!(machine.committed().callback_index().value(), 1);
}

#[test]
fn independently_prepared_stale_token_is_consumed_and_rejected_after_commit() {
    let mut machine = ReaderEofMachine::cold();
    let first = machine
        .stage(
            machine
                .prepare("tx-first", next_state(machine.committed()))
                .unwrap(),
        )
        .unwrap();
    let stale = machine
        .stage(
            machine
                .prepare("tx-stale", next_state(machine.committed()))
                .unwrap(),
        )
        .unwrap();
    machine.commit(first).unwrap();
    assert_eq!(
        machine.commit(stale),
        Err(ReaderEofError::GenerationMismatch)
    );
}

#[test]
fn fatal_retires_and_only_consuming_full_restart_returns_to_cold() {
    let mut machine = ReaderEofMachine::cold();
    let staged = machine
        .stage(
            machine
                .prepare("tx-fatal", next_state(machine.committed()))
                .unwrap(),
        )
        .unwrap();
    machine.fatal(staged).unwrap();
    assert!(machine.is_retired());
    assert_eq!(machine.generation(), 0);
    assert_eq!(
        machine.prepare("tx-after", next_state(machine.committed())),
        Err(ReaderEofError::Retired)
    );
    let restarted = machine.full_restart();
    assert!(!restarted.is_retired());
    assert_eq!(restarted.generation(), 0);
    assert_eq!(restarted.committed(), &ReaderEofState::cold());
}

#[test]
fn shape_phase_transaction_and_generation_validation_rejects_before_mutation() {
    assert_eq!(
        Host38Frame::new(vec![0.0; 1]),
        Err(ReaderEofError::HostShape)
    );
    assert_eq!(
        Control38Snapshot::new(vec![0; 1]),
        Err(ReaderEofError::ControlShape)
    );
    let machine = ReaderEofMachine::cold();
    assert_eq!(
        machine.prepare("", next_state(machine.committed())),
        Err(ReaderEofError::EmptyTransactionId)
    );
    let skipped = ReaderEofState::from_parts(
        FrontendCallbackIndex::new(2),
        0,
        NormalCallbackCount::new(0),
        4,
        None,
        None,
        ReaderEofPhase::Normal,
    )
    .unwrap();
    assert_eq!(
        machine.prepare("tx-skip", skipped),
        Err(ReaderEofError::GenerationMismatch)
    );
    assert_eq!(machine.generation(), 0);
}
