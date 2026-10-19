// pattern: Functional Core
//! Previous-aware materialization of the decoded Type11 rows into 56-byte reader records.

use super::type11::DecodedOamdRow;
use super::{READER_ROWS, RECORD_BYTES, ReaderTargetError, read_record_word, write_record_word};

const COMPONENT_MAP: [u32; 7] = [0, 3, 2, 4, 1, 5, 6];

pub(super) fn materialize(
    previous_records: &[[u8; RECORD_BYTES]; READER_ROWS],
    rows: &[DecodedOamdRow; READER_ROWS],
) -> Result<[[u8; RECORD_BYTES]; READER_ROWS], ReaderTargetError> {
    let mut next = *previous_records;
    for row in 0..READER_ROWS {
        let decoded = rows[row];
        let preserved_word8 = read_record_word(&previous_records[row], 8);
        let record = &mut next[row];
        if row == 0 {
            if decoded.kind != 0 || decoded.mute {
                return Err(ReaderTargetError::ProfileUnsupported);
            }
            write_record_word(record, 0, 1);
            write_record_word(record, 1, 4);
            write_record_word(record, 2, 0);
            write_record_word(record, 3, 0);
            write_record_word(record, 4, 0);
            write_record_word(record, 5, 0);
            write_record_word(record, 6, 0);
            write_record_word(record, 7, 0);
            write_record_word(record, 8, preserved_word8);
            write_record_word(record, 9, 1);
            write_record_word(record, 10, 1);
            write_record_word(
                record,
                11,
                next_source_state(read_record_word(&previous_records[row], 11)),
            );
            write_record_word(record, 12, decoded.gain);
            write_record_word(record, 13, decoded.row_flag);
            continue;
        }
        if decoded.kind != 2 {
            return Err(ReaderTargetError::ProfileUnsupported);
        }
        if decoded.mute {
            write_record_word(record, 0, 2);
            write_record_word(record, 1, 16_384);
            write_record_word(record, 2, 16_384);
            write_record_word(record, 3, 0);
            write_record_word(record, 4, 0);
            write_record_word(record, 5, 0);
            write_record_word(record, 6, 1);
            write_record_word(record, 7, 0);
            write_record_word(record, 8, preserved_word8);
            write_record_word(record, 9, 1);
            write_record_word(record, 10, 1);
            write_record_word(record, 11, 3);
            write_record_word(record, 12, 0);
            write_record_word(record, 13, decoded.row_flag);
            continue;
        }
        let component = usize::try_from(decoded.component)
            .map_err(|_| ReaderTargetError::ProfileUnsupported)?;
        let mapped_component = *COMPONENT_MAP
            .get(component)
            .ok_or(ReaderTargetError::ProfileUnsupported)?;
        write_record_word(record, 0, 2);
        write_record_word(record, 1, decoded.x);
        write_record_word(record, 2, decoded.y);
        write_record_word(
            record,
            3,
            if decoded.z_valid == 0 {
                0
            } else {
                u32::from_le_bytes(decoded.z.to_le_bytes())
            },
        );
        write_record_word(record, 4, mapped_component);
        write_record_word(record, 5, decoded.diffuse);
        write_record_word(record, 6, decoded.z_valid);
        write_record_word(record, 7, u32::from_le_bytes(decoded.extra.to_le_bytes()));
        write_record_word(record, 8, preserved_word8);
        write_record_word(record, 9, 1);
        write_record_word(record, 10, 1);
        write_record_word(
            record,
            11,
            next_source_state(read_record_word(&previous_records[row], 11)),
        );
        write_record_word(record, 12, decoded.gain);
        write_record_word(record, 13, decoded.row_flag);
    }
    Ok(next)
}

fn next_source_state(previous: u32) -> u32 {
    1 + u32::from((1..=2).contains(&previous))
}
