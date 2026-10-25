// pattern: Functional Core
//! Boundary that assembles validated typed receipts. It cannot calculate upstream values.

use crate::codec::bitio::{FrameBitWriter, FrameBitWriterError};
use crate::codec::frame_syntax::{CodecBlockIndex, SelectedBsiDrcFields};
use crate::timeline::CodecSourceIndex;

const DIALNORM_BIT_OFFSET: u32 = 45;
const LINE_BLOCK_COUNT: u8 = 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedBsiDrcWriterError {
    HeaderAlreadyWritten,
    HeaderCursorMismatch { actual: u32 },
    HeaderNotWritten,
    BlockOutOfOrder { expected: u8, actual: u8 },
    IncompleteBlockFields { written: u8 },
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for SelectedBsiDrcWriterError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedBsiDrcFieldsWritten {
    source_index: CodecSourceIndex,
}

impl SelectedBsiDrcFieldsWritten {
    pub const fn source_index(self) -> CodecSourceIndex {
        self.source_index
    }
}

/// Writes the selected F06ND DRC fields into caller-owned F12 frame scratch.
///
/// The header segment starts at bit 45. The caller interleaves each line DRC
/// field at the current cursor after that block's optional short and dither
/// syntax, and must call blocks in order 0 through 5. This stage owns no frame
/// state, CRC, candidate retry or parent commit.
pub struct SelectedBsiDrcWriter {
    fields: SelectedBsiDrcFields,
    header_written: bool,
    next_block: u8,
}

impl SelectedBsiDrcWriter {
    pub const fn new(fields: SelectedBsiDrcFields) -> Self {
        Self {
            fields,
            header_written: false,
            next_block: 0,
        }
    }

    /// Writes U5 dialnorm, `compre`, then optional U8 `compr` at bit 45.
    pub fn write_header_drc(
        &mut self,
        writer: &mut FrameBitWriter<'_>,
    ) -> Result<(), SelectedBsiDrcWriterError> {
        if self.header_written {
            return Err(SelectedBsiDrcWriterError::HeaderAlreadyWritten);
        }
        let cursor = writer.cursor().bit_offset();
        if cursor != DIALNORM_BIT_OFFSET {
            return Err(SelectedBsiDrcWriterError::HeaderCursorMismatch { actual: cursor });
        }

        let dialnorm = u32::from(self.fields.dialnorm());
        let (value, width) = match self.fields.rf_code() {
            Some(code) => ((dialnorm << 9) | (1 << 8) | u32::from(code), 14),
            None => (dialnorm << 1, 6),
        };
        writer.write_unsigned(value, width)?;
        self.header_written = true;
        Ok(())
    }

    /// Writes one line-presence bit and its optional U8 code at the live cursor.
    pub fn write_line_drc(
        &mut self,
        writer: &mut FrameBitWriter<'_>,
        block: CodecBlockIndex,
    ) -> Result<(), SelectedBsiDrcWriterError> {
        if !self.header_written {
            return Err(SelectedBsiDrcWriterError::HeaderNotWritten);
        }
        let actual = block.value();
        if actual != self.next_block {
            return Err(SelectedBsiDrcWriterError::BlockOutOfOrder {
                expected: self.next_block,
                actual,
            });
        }

        let (value, width) = match self.fields.line_code(block) {
            Some(code) => ((1 << 8) | u32::from(code), 9),
            None => (0, 1),
        };
        writer.write_unsigned(value, width)?;
        self.next_block += 1;
        Ok(())
    }

    pub const fn is_complete(&self) -> bool {
        self.header_written && self.next_block == LINE_BLOCK_COUNT
    }

    pub fn finish(self) -> Result<SelectedBsiDrcFieldsWritten, SelectedBsiDrcWriterError> {
        if !self.header_written {
            return Err(SelectedBsiDrcWriterError::HeaderNotWritten);
        }
        if self.next_block != LINE_BLOCK_COUNT {
            return Err(SelectedBsiDrcWriterError::IncompleteBlockFields {
                written: self.next_block,
            });
        }
        Ok(SelectedBsiDrcFieldsWritten {
            source_index: self.fields.source_index(),
        })
    }
}
