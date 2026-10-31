// pattern: Functional Core
//! Boundary that assembles validated typed receipts. It cannot calculate upstream values.

use crate::codec::bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError};
use crate::codec::frame_syntax::{
    CodecBlockIndex, SelectedBsiDrcFields, SelectedBsiMixingInformationValues,
    SelectedComplexityIndexTypeA,
};
use crate::timeline::CodecSourceIndex;

const DIALNORM_BIT_OFFSET: u32 = 45;
const SELECTED_FIXED_PREFIX_BITS: u32 = 45;
const COMPLEXITY_INDEX_TYPE_A_BIT_OFFSET: u32 = 105;
const COMPLEXITY_INDEX_TYPE_A_WIDTH: u8 = 8;
const LINE_BLOCK_COUNT: u8 = 6;
const SELECTED_DRC_HEADER_BITS_WITHOUT_COMPR: u32 = 6;
const SELECTED_DRC_HEADER_BITS_WITH_COMPR: u32 = 14;
const SELECTED_MIXING_INFORMATION_BITS: u32 = 31;
const SELECTED_FIXED_PREFIX_FIELDS: [(u32, u8); 9] = [
    (0x0B77, 16),
    (0, 2),
    (0, 3),
    (0x07FF, 11),
    (0, 2),
    (3, 2),
    (7, 3),
    (1, 1),
    (16, 5),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedEac3PrefixError {
    CursorMismatch { actual: u32 },
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for SelectedEac3PrefixError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Writes the selected 48 kHz / 1024 kbps / 5.1 fixed E-AC-3 prefix at bit 0.
///
/// The prefix ends at bit 44; the next caller-owned field begins at bit 45.
/// Three display-pad bits are not emitted.
pub fn write_selected_eac3_fixed_prefix(
    writer: &mut FrameBitWriter<'_>,
) -> Result<FrameBitCursor, SelectedEac3PrefixError> {
    let actual = writer.cursor().bit_offset();
    if actual != 0 {
        return Err(SelectedEac3PrefixError::CursorMismatch { actual });
    }
    writer.preflight_capacity(SELECTED_FIXED_PREFIX_BITS)?;
    for (value, width) in SELECTED_FIXED_PREFIX_FIELDS {
        writer.write_unsigned(value, width)?;
    }
    Ok(writer.cursor())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedComplexityFieldWriteError {
    CursorMismatch { actual: u32 },
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for SelectedComplexityFieldWriteError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Writes only the selected F12 additional-BSI complexity U8 at bits 105..112.
///
/// The caller owns presence/length/identifier fields and positions the cursor.
/// This field writer does not derive the model value, select a carrier block,
/// identify the native BSI producer, or assemble a complete frame.
pub fn write_selected_complexity_index_type_a(
    writer: &mut FrameBitWriter<'_>,
    complexity: SelectedComplexityIndexTypeA,
) -> Result<FrameBitCursor, SelectedComplexityFieldWriteError> {
    let actual = writer.cursor().bit_offset();
    if actual != COMPLEXITY_INDEX_TYPE_A_BIT_OFFSET {
        return Err(SelectedComplexityFieldWriteError::CursorMismatch { actual });
    }
    writer.preflight_capacity(u32::from(COMPLEXITY_INDEX_TYPE_A_WIDTH))?;
    writer.write_unsigned(u32::from(complexity.value()), COMPLEXITY_INDEX_TYPE_A_WIDTH)?;
    Ok(writer.cursor())
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedBsiMixingInformationField {
    Dmixmod,
    Ltrtcmixlev,
    Lorocmixlev,
    Ltrtsurmixlev,
    Lorosurmixlev,
    Bsmod,
    Dsurexmod,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedBsiMixingInformationWriteError {
    HeaderCursorMismatch {
        actual: u32,
    },
    FieldOutOfRange {
        field: SelectedBsiMixingInformationField,
        value: u8,
    },
    BitWriter(FrameBitWriterError),
}

impl From<FrameBitWriterError> for SelectedBsiMixingInformationWriteError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Writes selected-profile `mixmdate` and `infomdate` syntax after the DRC header.
///
/// The fixed acmod=7/lfeon=1/strmtyp=0 profile emits the selected conditional
/// fields and fixed zero presence flags from the approved frame-assembly
/// contract. The caller supplies normalized profile values; this writer does
/// not produce or bind their source owner/generation.
pub fn write_selected_bsi_mixing_information(
    writer: &mut FrameBitWriter<'_>,
    fields: &SelectedBsiMixingInformationValues,
) -> Result<FrameBitCursor, SelectedBsiMixingInformationWriteError> {
    let actual = writer.cursor().bit_offset();
    let cursor_without_compr = DIALNORM_BIT_OFFSET + SELECTED_DRC_HEADER_BITS_WITHOUT_COMPR;
    let cursor_with_compr = DIALNORM_BIT_OFFSET + SELECTED_DRC_HEADER_BITS_WITH_COMPR;
    if actual != cursor_without_compr && actual != cursor_with_compr {
        return Err(SelectedBsiMixingInformationWriteError::HeaderCursorMismatch { actual });
    }

    for (field, value, maximum) in [
        (
            SelectedBsiMixingInformationField::Dmixmod,
            fields.dmixmod,
            3,
        ),
        (
            SelectedBsiMixingInformationField::Ltrtcmixlev,
            fields.ltrtcmixlev,
            7,
        ),
        (
            SelectedBsiMixingInformationField::Lorocmixlev,
            fields.lorocmixlev,
            7,
        ),
        (
            SelectedBsiMixingInformationField::Ltrtsurmixlev,
            fields.ltrtsurmixlev,
            7,
        ),
        (
            SelectedBsiMixingInformationField::Lorosurmixlev,
            fields.lorosurmixlev,
            7,
        ),
        (SelectedBsiMixingInformationField::Bsmod, fields.bsmod, 7),
        (
            SelectedBsiMixingInformationField::Dsurexmod,
            fields.dsurexmod,
            3,
        ),
    ] {
        if value > maximum {
            return Err(SelectedBsiMixingInformationWriteError::FieldOutOfRange { field, value });
        }
    }

    writer.preflight_capacity(SELECTED_MIXING_INFORMATION_BITS)?;

    // The selected profile has mixmdate=1; conditional mix levels follow MSB-first.
    writer.write_bit(true)?;
    writer.write_unsigned(u32::from(fields.dmixmod), 2)?;
    writer.write_unsigned(u32::from(fields.ltrtcmixlev), 3)?;
    writer.write_unsigned(u32::from(fields.lorocmixlev), 3)?;
    writer.write_unsigned(u32::from(fields.ltrtsurmixlev), 3)?;
    writer.write_unsigned(u32::from(fields.lorosurmixlev), 3)?;

    // Selected zero presence flags suppress their optional payloads.
    writer.write_bit(false)?; // lfemixlevcode
    writer.write_bit(false)?; // pgmscle
    writer.write_bit(false)?; // extpgmscle
    writer.write_unsigned(0, 2)?; // mixdef=0, no additional payload
    writer.write_bit(false)?; // frmmixcfginfoe

    // The selected profile has infomdate=1; acmod=7 omits dsurmod/dheadphonmod.
    writer.write_bit(true)?;
    writer.write_unsigned(u32::from(fields.bsmod), 3)?;
    writer.write_bit(fields.copyrightb)?;
    writer.write_bit(fields.origbs)?;
    writer.write_unsigned(u32::from(fields.dsurexmod), 2)?;
    writer.write_bit(false)?; // audprodie
    writer.write_bit(fields.sourcefscod)?;

    Ok(writer.cursor())
}

#[cfg(test)]
mod selected_bsi_mixing_information_tests {
    use super::*;

    fn append_bits(bits: &mut Vec<bool>, value: u32, width: u8) {
        for shift in (0..width).rev() {
            bits.push((value >> shift) & 1 != 0);
        }
    }

    fn bit_at(bytes: &[u8], bit_offset: u32) -> bool {
        (bytes[(bit_offset / 8) as usize] >> (7 - bit_offset % 8)) & 1 != 0
    }

    fn fixture_fields() -> SelectedBsiMixingInformationValues {
        SelectedBsiMixingInformationValues {
            dmixmod: 2,
            ltrtcmixlev: 1,
            lorocmixlev: 2,
            ltrtsurmixlev: 4,
            lorosurmixlev: 7,
            bsmod: 5,
            copyrightb: true,
            origbs: false,
            dsurexmod: 3,
            sourcefscod: true,
        }
    }

    #[test]
    fn writes_selected_mixing_and_information_fields_msb_first() {
        let start = DIALNORM_BIT_OFFSET + SELECTED_DRC_HEADER_BITS_WITHOUT_COMPR;
        let before = [0xa5_u8; 16];
        let mut bytes = before;
        let cursor = {
            let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
            write_selected_bsi_mixing_information(&mut writer, &fixture_fields()).unwrap()
        };

        let mut expected = Vec::with_capacity(SELECTED_MIXING_INFORMATION_BITS as usize);
        append_bits(&mut expected, 1, 1); // mixmdate
        append_bits(&mut expected, 2, 2); // dmixmod
        append_bits(&mut expected, 1, 3); // ltrtcmixlev
        append_bits(&mut expected, 2, 3); // lorocmixlev
        append_bits(&mut expected, 4, 3); // ltrtsurmixlev
        append_bits(&mut expected, 7, 3); // lorosurmixlev
        append_bits(&mut expected, 0, 1); // lfemixlevcode
        append_bits(&mut expected, 0, 1); // pgmscle
        append_bits(&mut expected, 0, 1); // extpgmscle
        append_bits(&mut expected, 0, 2); // mixdef
        append_bits(&mut expected, 0, 1); // frmmixcfginfoe
        append_bits(&mut expected, 1, 1); // infomdate
        append_bits(&mut expected, 5, 3); // bsmod
        append_bits(&mut expected, 1, 1); // copyrightb
        append_bits(&mut expected, 0, 1); // origbs
        append_bits(&mut expected, 3, 2); // dsurexmod
        append_bits(&mut expected, 0, 1); // audprodie
        append_bits(&mut expected, 1, 1); // sourcefscod

        assert_eq!(expected.len(), SELECTED_MIXING_INFORMATION_BITS as usize);
        assert_eq!(
            cursor.bit_offset(),
            start + SELECTED_MIXING_INFORMATION_BITS
        );
        for (offset, bit) in expected.into_iter().enumerate() {
            assert_eq!(bit_at(&bytes, start + offset as u32), bit);
        }
        for bit in 0..start {
            assert_eq!(bit_at(&bytes, bit), bit_at(&before, bit));
        }
        for bit in (start + SELECTED_MIXING_INFORMATION_BITS)..128 {
            assert_eq!(bit_at(&bytes, bit), bit_at(&before, bit));
        }
    }

    #[test]
    fn accepts_the_header_cursor_after_either_drc_compre_branch() {
        let start = DIALNORM_BIT_OFFSET + SELECTED_DRC_HEADER_BITS_WITH_COMPR;
        let mut bytes = [0_u8; 16];
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
        let cursor = write_selected_bsi_mixing_information(&mut writer, &fixture_fields()).unwrap();
        assert_eq!(
            cursor.bit_offset(),
            start + SELECTED_MIXING_INFORMATION_BITS
        );
    }

    #[test]
    fn rejects_invalid_fields_cursor_and_capacity_before_mutation() {
        let start = DIALNORM_BIT_OFFSET + SELECTED_DRC_HEADER_BITS_WITHOUT_COMPR;
        let mut bytes = [0xa5_u8; 16];
        let before = bytes;
        let mut invalid = fixture_fields();
        invalid.dmixmod = 4;
        let error = {
            let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start)).unwrap();
            write_selected_bsi_mixing_information(&mut writer, &invalid).unwrap_err()
        };
        assert_eq!(
            error,
            SelectedBsiMixingInformationWriteError::FieldOutOfRange {
                field: SelectedBsiMixingInformationField::Dmixmod,
                value: 4,
            }
        );
        assert_eq!(bytes, before);

        let mut bytes = before;
        let error = {
            let mut writer =
                FrameBitWriter::new(&mut bytes, FrameBitCursor::new(start + 1)).unwrap();
            write_selected_bsi_mixing_information(&mut writer, &fixture_fields()).unwrap_err()
        };
        assert_eq!(
            error,
            SelectedBsiMixingInformationWriteError::HeaderCursorMismatch { actual: start + 1 }
        );
        assert_eq!(bytes, before);

        let mut short = [0xa5_u8; 10];
        let before = short;
        let error = {
            let mut writer = FrameBitWriter::new(&mut short, FrameBitCursor::new(start)).unwrap();
            write_selected_bsi_mixing_information(&mut writer, &fixture_fields()).unwrap_err()
        };
        assert_eq!(
            error,
            SelectedBsiMixingInformationWriteError::BitWriter(FrameBitWriterError::BufferTooSmall)
        );
        assert_eq!(short, before);
    }
}
