// pattern: Functional Core
use jocforge_domain::drc::drc_control_buffer::{
    DrcControlBufferError, drc_control_field_start_bit, encode_drc_control_buffer,
};

#[test]
fn relative_start_bit_matches_washed_input_only_vectors() {
    let mut record = [0_u8; 9];
    assert_eq!(drc_control_field_start_bit(&record).unwrap().value(), 6);

    record[0] = 3;
    assert_eq!(drc_control_field_start_bit(&record).unwrap().value(), 10);

    record = [0; 9];
    record[4] = 1;
    assert_eq!(drc_control_field_start_bit(&record).unwrap().value(), 9);

    record = [0; 9];
    record[8] = 1;
    assert_eq!(drc_control_field_start_bit(&record).unwrap().value(), 11);
}

#[test]
fn one_hot_control_codes_write_msb_first_into_the_three_byte_buffer() {
    let buffer = encode_drc_control_buffer(&[0; 9], true, 0b001, 0b100).unwrap();

    assert_eq!(buffer.as_bytes(), &[0x02, 0x60, 0x00]);
}

#[test]
fn local_control_buffer_validation_rejects_unrepresentable_inputs() {
    assert_eq!(
        drc_control_field_start_bit(&[0; 8]),
        Err(DrcControlBufferError::RecordPrefixTooShort)
    );
    assert_eq!(
        encode_drc_control_buffer(&[0; 9], false, 8, 0),
        Err(DrcControlBufferError::CodeOutOfRange)
    );

    let mut record = [0_u8; 9];
    record[0] = 3;
    record[2] = 1;
    record[5] = 1;
    record[8] = 1;
    assert_eq!(
        encode_drc_control_buffer(&record, true, 0, 0),
        Err(DrcControlBufferError::FieldsOutsideBuffer)
    );
}
