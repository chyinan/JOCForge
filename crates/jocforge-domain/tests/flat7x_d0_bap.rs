use jocforge_domain::codec::bap::{
    BapError, FLAT7X_D0_BAP_BIN_COUNT, FLAT7X_D0_BAP_BLOCK_COUNT, FLAT7X_D0_BAP_CHANNEL_COUNT,
    FLAT7X_D0_BAP_PROFILE_ID, FLAT7X_D0_BAP_TABLE_SET_SHA256, Flat7xD0BapFrameInput,
    Flat7xD0BapParameters, Flat7xD0BapRowInput, allocate_flat7x_d0_bap_frame,
    allocate_flat7x_d0_bap_row,
};

fn d0_parameters() -> Flat7xD0BapParameters {
    Flat7xD0BapParameters::etsi_ts_102_366_v1_4_1()
}

fn hex_digest(bytes: [u8; 32]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(64);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}

fn row_input(exponents: &[u8; FLAT7X_D0_BAP_BIN_COUNT]) -> Flat7xD0BapRowInput<'_> {
    Flat7xD0BapRowInput {
        decoded_exponents: exponents,
        candidate: 1_023,
        slow_decay_code: 2,
        fast_decay_code: 1,
        slow_gain_code: 1,
        db_per_bit_code: 2,
        floor_code: 7,
        fast_gain_code: 0,
        caller_all_active_fine_zero: false,
    }
}

#[test]
fn provider_binds_etsi_table_set_to_the_d0_four_by_253_profile() {
    let parameters = d0_parameters();

    assert_eq!(FLAT7X_D0_BAP_PROFILE_ID, "FLAT7X_D0_FBW253_4CH_NO_LFE_V1");
    assert_eq!(FLAT7X_D0_BAP_BIN_COUNT, 253);
    assert_eq!(FLAT7X_D0_BAP_CHANNEL_COUNT, 4);
    assert_eq!(FLAT7X_D0_BAP_BLOCK_COUNT, 6);
    assert_eq!(
        hex_digest(parameters.table_set_sha256()),
        FLAT7X_D0_BAP_TABLE_SET_SHA256
    );
    assert_eq!(
        FLAT7X_D0_BAP_TABLE_SET_SHA256,
        "5a9ea2d179dc8017316006e8ffd23b2f5e30f1b363e56740f94787b4ef8e96fe"
    );
}

#[test]
fn d0_row_allocator_returns_exactly_253_bap_codes() {
    let parameters = d0_parameters();
    let exponents = [15_u8; FLAT7X_D0_BAP_BIN_COUNT];

    let row = allocate_flat7x_d0_bap_row(&parameters, &row_input(&exponents)).unwrap();

    assert_eq!(row.len(), 253);
    assert!(row.iter().all(|code| *code <= 15));
    assert!(row.iter().any(|code| *code > 0));
}

#[test]
fn d0_row_allocator_rejects_invalid_exponent_codes() {
    let parameters = d0_parameters();
    let mut exponents = [15_u8; FLAT7X_D0_BAP_BIN_COUNT];
    exponents[FLAT7X_D0_BAP_BIN_COUNT - 1] = 25;

    assert_eq!(
        allocate_flat7x_d0_bap_row(&parameters, &row_input(&exponents)),
        Err(BapError::InputError)
    );
}

#[test]
fn d0_frame_allocator_preserves_six_blocks_and_four_channel_rows() {
    let parameters = d0_parameters();
    let exponents = [[[15_u8; FLAT7X_D0_BAP_BIN_COUNT]; FLAT7X_D0_BAP_CHANNEL_COUNT];
        FLAT7X_D0_BAP_BLOCK_COUNT];
    let input = Flat7xD0BapFrameInput {
        decoded_exponents_by_block: &exponents,
        candidate: 1_023,
        slow_decay_code: 2,
        fast_decay_code: 1,
        slow_gain_code: 1,
        db_per_bit_codes: [2; FLAT7X_D0_BAP_BLOCK_COUNT],
        floor_code: 7,
        fast_gain_code: 0,
        caller_all_active_fine_zero_by_block: [false; FLAT7X_D0_BAP_BLOCK_COUNT],
    };

    let frame = allocate_flat7x_d0_bap_frame(&parameters, &input).unwrap();

    assert_eq!(frame.candidate(), 1_023);
    assert_eq!(frame.blocks().len(), 6);
    assert_eq!(frame.table_set_sha256(), parameters.table_set_sha256());
    for block in frame.blocks() {
        assert_eq!(block.rows().len(), 4);
        for row in block.rows() {
            assert_eq!(row.len(), 253);
            assert!(row.iter().all(|code| *code <= 15));
        }
    }
    let single_row = allocate_flat7x_d0_bap_row(&parameters, &row_input(&exponents[0][0])).unwrap();
    assert_eq!(frame.blocks()[0].rows()[0], single_row);
}
