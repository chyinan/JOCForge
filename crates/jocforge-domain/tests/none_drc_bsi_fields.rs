use jocforge_domain::codec::bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError};
use jocforge_domain::codec::frame_builder::{SelectedBsiDrcWriter, SelectedBsiDrcWriterError};
use jocforge_domain::codec::frame_syntax::{
    CodecBlockIndex, SelectedBsiDrcSyntaxError, bind_selected_bsi_drc_fields,
};
use jocforge_domain::drc::none_drc_callback::{NoneDrcCallbackInput, transition_none_drc_callback};
use jocforge_domain::drc::none_drc_scheduler::NoneDrcSchedulerState;
use jocforge_domain::drc::none_drc_statistics::{FiniteStatisticsPlanes, NoneDrcStatisticsState};
use jocforge_domain::numeric::FiniteF32;
use jocforge_domain::timeline::{CodecSourceIndex, FrontendCallbackIndex};

#[test]
fn selected_drc_fields_bind_to_same_source_and_write_at_specified_cursors() {
    let samples = zero_planes();
    let statistics_state = NoneDrcStatisticsState::cold();
    let scheduler_state = NoneDrcSchedulerState::cold();
    let callback_input = NoneDrcCallbackInput {
        callback_index: FrontendCallbackIndex::new(2),
        primary_samples: &samples,
        secondary_samples: &samples,
        dialnorm: 24,
    };
    let f06nd = transition_none_drc_callback(&statistics_state, &scheduler_state, &callback_input)
        .unwrap()
        .unwrap();
    let source = CodecSourceIndex::new(0);
    let fields = bind_selected_bsi_drc_fields(source, &f06nd).unwrap();
    let scheduler = &f06nd.frame().output.scheduler;

    assert_eq!(fields.source_index(), source);
    assert_eq!(fields.dialnorm(), 24);
    assert_eq!(
        fields.rf_code(),
        Some(scheduler.rf_packed_word_u16.to_be_bytes()[0]),
    );
    for block_value in 0..6 {
        let block = CodecBlockIndex::new(block_value).unwrap();
        assert_eq!(
            fields.line_code(block),
            Some(scheduler.line_packed_words_u16[usize::from(block_value)].to_be_bytes()[0]),
        );
    }

    let mut bytes = [0_u8; 32];
    let mut drc_writer = SelectedBsiDrcWriter::new(fields);
    let mut cursor = {
        let mut bit_writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(45)).unwrap();
        drc_writer.write_header_drc(&mut bit_writer).unwrap();
        bit_writer.cursor().bit_offset()
    };
    assert_eq!(cursor, 59);
    assert_eq!(read_bits(&bytes, 45, 5), 24);
    assert_eq!(read_bits(&bytes, 50, 1), 1);
    assert_eq!(
        read_bits(&bytes, 51, 8),
        u32::from(fields.rf_code().unwrap())
    );
    for block_value in 0..6 {
        let block = CodecBlockIndex::new(block_value).unwrap();
        let line_code = fields.line_code(block).unwrap();
        let (field_offset, next_cursor) = {
            let mut bit_writer =
                FrameBitWriter::new(&mut bytes, FrameBitCursor::new(cursor)).unwrap();
            bit_writer
                .skip_zeroes(u32::from((block_value % 3) + 1))
                .unwrap();
            let field_offset = bit_writer.cursor().bit_offset();
            drc_writer.write_line_drc(&mut bit_writer, block).unwrap();
            (field_offset, bit_writer.cursor().bit_offset())
        };
        assert_eq!(read_bits(&bytes, field_offset, 1), 1);
        assert_eq!(read_bits(&bytes, field_offset + 1, 8), u32::from(line_code),);
        cursor = next_cursor;
    }

    assert!(drc_writer.is_complete());
    assert_eq!(drc_writer.finish().unwrap().source_index(), source);
}

#[test]
fn bsi_field_binding_rejects_a_different_frame_source() {
    let f06nd = successful_f06nd_transition();
    assert_eq!(
        bind_selected_bsi_drc_fields(CodecSourceIndex::new(1), &f06nd),
        Err(SelectedBsiDrcSyntaxError::SourceGenerationMismatch),
    );
    assert_eq!(
        CodecBlockIndex::new(6),
        Err(SelectedBsiDrcSyntaxError::BlockIndexOutOfRange(6)),
    );
}

#[test]
fn wrong_cursor_order_and_capacity_fail_without_partial_field_writes() {
    let fields =
        bind_selected_bsi_drc_fields(CodecSourceIndex::new(0), &successful_f06nd_transition())
            .unwrap();
    let mut bytes = [0xA5_u8; 8];
    let mut drc_writer = SelectedBsiDrcWriter::new(fields);

    let mut wrong_bytes = [0xA5_u8; 8];
    let before_bad_header = wrong_bytes;
    let mut wrong_cursor_writer = SelectedBsiDrcWriter::new(fields);
    let wrong_cursor_offset = {
        let mut wrong_cursor =
            FrameBitWriter::new(&mut wrong_bytes, FrameBitCursor::new(44)).unwrap();
        assert_eq!(
            wrong_cursor_writer.write_header_drc(&mut wrong_cursor),
            Err(SelectedBsiDrcWriterError::HeaderCursorMismatch { actual: 44 }),
        );
        wrong_cursor.cursor().bit_offset()
    };
    assert_eq!(wrong_cursor_offset, 44);
    assert_eq!(wrong_bytes, before_bad_header);

    let cursor_before_line = {
        let mut bit_writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(45)).unwrap();
        drc_writer.write_header_drc(&mut bit_writer).unwrap();
        bit_writer.cursor().bit_offset()
    };
    let before_line = bytes;
    let cursor_after_failed_writes = {
        let mut bit_writer =
            FrameBitWriter::new(&mut bytes, FrameBitCursor::new(cursor_before_line)).unwrap();
        assert_eq!(
            drc_writer.write_line_drc(&mut bit_writer, CodecBlockIndex::new(1).unwrap()),
            Err(SelectedBsiDrcWriterError::BlockOutOfOrder {
                expected: 0,
                actual: 1,
            }),
        );
        assert_eq!(
            drc_writer.write_line_drc(&mut bit_writer, CodecBlockIndex::new(0).unwrap()),
            Err(SelectedBsiDrcWriterError::BitWriter(
                FrameBitWriterError::BufferTooSmall,
            )),
        );
        bit_writer.cursor().bit_offset()
    };
    assert_eq!(cursor_after_failed_writes, cursor_before_line);
    assert!(!drc_writer.is_complete());
    assert_eq!(bytes, before_line);
}

#[test]
fn bsi_writer_rejects_invalid_header_and_completion_lifecycle() {
    let fields =
        bind_selected_bsi_drc_fields(CodecSourceIndex::new(0), &successful_f06nd_transition())
            .unwrap();
    let mut bytes = [0_u8; 32];
    let mut bit_writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(45)).unwrap();
    let mut drc_writer = SelectedBsiDrcWriter::new(fields);

    assert_eq!(
        drc_writer.write_line_drc(&mut bit_writer, CodecBlockIndex::new(0).unwrap()),
        Err(SelectedBsiDrcWriterError::HeaderNotWritten),
    );
    assert_eq!(
        drc_writer.finish(),
        Err(SelectedBsiDrcWriterError::HeaderNotWritten),
    );

    let mut drc_writer = SelectedBsiDrcWriter::new(fields);
    drc_writer.write_header_drc(&mut bit_writer).unwrap();
    assert_eq!(
        drc_writer.write_header_drc(&mut bit_writer),
        Err(SelectedBsiDrcWriterError::HeaderAlreadyWritten),
    );
    assert_eq!(
        drc_writer.finish(),
        Err(SelectedBsiDrcWriterError::IncompleteBlockFields { written: 0 }),
    );
}

fn successful_f06nd_transition()
-> jocforge_domain::drc::none_drc_callback::NoneDrcCallbackTransition {
    let samples = zero_planes();
    let input = NoneDrcCallbackInput {
        callback_index: FrontendCallbackIndex::new(2),
        primary_samples: &samples,
        secondary_samples: &samples,
        dialnorm: 24,
    };
    transition_none_drc_callback(
        &NoneDrcStatisticsState::cold(),
        &NoneDrcSchedulerState::cold(),
        &input,
    )
    .unwrap()
    .unwrap()
}

fn zero_planes() -> FiniteStatisticsPlanes {
    std::array::from_fn(|_| std::array::from_fn(|_| FiniteF32::new(0.0).expect("zero is finite")))
}

fn read_bits(bytes: &[u8], bit_offset: u32, width: u8) -> u32 {
    (0..u32::from(width)).fold(0, |value, bit| {
        let absolute = bit_offset + bit;
        let byte = bytes[usize::try_from(absolute / 8).unwrap()];
        let shift = 7 - (absolute % 8);
        (value << 1) | u32::from((byte >> shift) & 1)
    })
}
