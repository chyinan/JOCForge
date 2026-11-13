use jocforge_domain::codec::bitio::{FrameBitCursor, FrameBitWriter};
use jocforge_domain::codec::flat7x_d0_exponents::{
    FLAT7X_D0_D15_EXPONENTS_PER_ROW, FLAT7X_D0_D15_GROUP_COUNT, FLAT7X_D0_D15_PREPROCESSOR_ID,
    prepare_flat7x_d0_d15_exponent_row, write_flat7x_d0_d15_exponent_row,
};

#[test]
fn d15_row_caps_u4_seed_and_limits_decoded_differences_to_two() {
    let coefficients = [0.0_f32.to_bits(); FLAT7X_D0_D15_EXPONENTS_PER_ROW];

    let row = prepare_flat7x_d0_d15_exponent_row(&coefficients).unwrap();

    assert_eq!(row.preprocessor_id(), FLAT7X_D0_D15_PREPROCESSOR_ID);
    assert_eq!(row.raw_exponents(), &[24; FLAT7X_D0_D15_EXPONENTS_PER_ROW]);
    assert_eq!(row.seed().value(), 15);
    assert_eq!(&row.decoded_exponents()[..7], &[15, 17, 19, 21, 23, 24, 24]);
    assert_eq!(row.groups().len(), FLAT7X_D0_D15_GROUP_COUNT);
    assert_eq!(row.groups()[0].value(), 124);
    assert_eq!(row.groups()[1].value(), 117);
    assert_eq!(row.groups()[2].value(), 62);
    assert_eq!(row.groups()[FLAT7X_D0_D15_GROUP_COUNT - 1].value(), 62);
    assert!(row.decoded_exponents().iter().all(|value| *value <= 24));
    assert!(
        row.decoded_exponents()
            .windows(2)
            .all(|pair| i16::from(pair[1]) - i16::from(pair[0]) <= 2
                && i16::from(pair[0]) - i16::from(pair[1]) <= 2)
    );
}

#[test]
fn d15_raw_exponents_follow_coefficient_leading_zero_counts() {
    let mut coefficients = [0.5_f32.to_bits(); FLAT7X_D0_D15_EXPONENTS_PER_ROW];
    coefficients[1] = 0.25_f32.to_bits();
    coefficients[2] = 0.125_f32.to_bits();
    coefficients[3] = (-0.25_f32).to_bits();

    let row = prepare_flat7x_d0_d15_exponent_row(&coefficients).unwrap();

    assert_eq!(&row.raw_exponents()[..4], &[0, 1, 2, 1]);
    assert_eq!(&row.decoded_exponents()[..4], &[0, 1, 2, 1]);
}

#[test]
fn lower_only_preprocessor_handles_upward_and_downward_raw_exponent_spikes() {
    let patterns = [
        ([24, 0, 24], [2, 0, 2, 4, 6, 8]),
        ([0, 24, 0], [0, 2, 0, 2, 4, 6]),
        ([24, 24, 0], [4, 2, 0, 2, 4, 6]),
        ([0, 0, 24], [0, 0, 2, 4, 6, 8]),
    ];

    for (raw_prefix, decoded_prefix) in patterns {
        let mut coefficients = [0.0_f32.to_bits(); FLAT7X_D0_D15_EXPONENTS_PER_ROW];
        for (index, raw) in raw_prefix.into_iter().enumerate() {
            coefficients[index] = if raw == 24 {
                0.0_f32.to_bits()
            } else {
                0.5_f32.to_bits()
            };
        }
        let row = prepare_flat7x_d0_d15_exponent_row(&coefficients).unwrap();

        assert_eq!(&row.raw_exponents()[..raw_prefix.len()], &raw_prefix);
        assert_eq!(
            &row.decoded_exponents()[..decoded_prefix.len()],
            &decoded_prefix
        );
        assert!(
            row.decoded_exponents()
                .iter()
                .zip(row.raw_exponents())
                .all(|(decoded, raw)| decoded <= raw)
        );
        assert!(
            row.decoded_exponents()
                .windows(2)
                .all(|pair| i16::from(pair[1]) - i16::from(pair[0]) <= 2
                    && i16::from(pair[0]) - i16::from(pair[1]) <= 2)
        );
    }
}

#[test]
fn lower_only_d15_projection_preserves_invariants_for_10000_deterministic_rows() {
    let mut state = 0x6d2b_79f5_u32;
    for _ in 0..10_000 {
        let mut coefficients = [0_u32; FLAT7X_D0_D15_EXPONENTS_PER_ROW];
        for coefficient in &mut coefficients {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let raw_exponent = u8::try_from(state % 25).unwrap();
            *coefficient = coefficient_word_for_raw_exponent(raw_exponent);
        }

        let row = prepare_flat7x_d0_d15_exponent_row(&coefficients).unwrap();
        assert!(row.decoded_exponents()[0] <= 15);
        assert!(
            row.decoded_exponents()
                .iter()
                .zip(row.raw_exponents())
                .all(|(decoded, raw)| decoded <= raw)
        );
        assert!(
            row.decoded_exponents()
                .windows(2)
                .all(|pair| i16::from(pair[1]) - i16::from(pair[0]) <= 2
                    && i16::from(pair[0]) - i16::from(pair[1]) <= 2)
        );
        assert_eq!(row.groups().len(), FLAT7X_D0_D15_GROUP_COUNT);
    }
}

#[test]
fn d15_row_rejects_non_finite_spectrum_before_producing_any_groups() {
    let mut coefficients = [0.0_f32.to_bits(); FLAT7X_D0_D15_EXPONENTS_PER_ROW];
    coefficients[252] = f32::INFINITY.to_bits();

    assert!(prepare_flat7x_d0_d15_exponent_row(&coefficients).is_err());
}

#[test]
fn d0_d15_row_writes_u4_and_all_84_u7_groups_through_the_shared_serializer() {
    let coefficients = [0.0_f32.to_bits(); FLAT7X_D0_D15_EXPONENTS_PER_ROW];
    let row = prepare_flat7x_d0_d15_exponent_row(&coefficients).unwrap();
    let mut bytes = [0xa5_u8; 74];
    let receipt = {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
        let receipt = write_flat7x_d0_d15_exponent_row(&mut writer, &row).unwrap();
        assert_eq!(writer.cursor().bit_offset(), 592);
        receipt
    };

    assert_eq!(receipt.start_bit(), 0);
    assert_eq!(receipt.end_bit(), 592);
    assert_eq!(receipt.group_count(), 84);
    assert_eq!(read_bits(&bytes, 0, 4), u32::from(row.seed().value()));
    for (group_index, group) in row.groups().iter().enumerate() {
        let start_bit = 4 + u32::try_from(group_index).unwrap() * 7;
        assert_eq!(read_bits(&bytes, start_bit, 7), u32::from(group.value()));
    }
    assert!(!row.production_allowed());
}

fn read_bits(bytes: &[u8], start_bit: u32, bit_length: u8) -> u32 {
    let mut value = 0_u32;
    for offset in 0..u32::from(bit_length) {
        let bit = start_bit + offset;
        let byte_index = usize::try_from(bit / 8).unwrap();
        let shift = u8::try_from(7 - bit % 8).unwrap();
        value = (value << 1) | u32::from((bytes[byte_index] >> shift) & 1);
    }
    value
}

fn coefficient_word_for_raw_exponent(raw_exponent: u8) -> u32 {
    if raw_exponent == 24 {
        0.0_f32.to_bits()
    } else {
        u32::from(126 - raw_exponent) << 23
    }
}
