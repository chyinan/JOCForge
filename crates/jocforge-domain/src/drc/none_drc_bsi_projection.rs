// pattern: Functional Core
//! Stateless high-byte projection of selected F06ND controls for F12 BSI.

use crate::drc::none_drc_scheduler::NoneDrcSchedulerFrameOutput;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoneDrcBsiProjection {
    pub rf_high_byte: u8,
    pub line_high_bytes: [u8; 6],
}

/// Projects only the selected same-frame RF and six line U16 control words.
pub fn project_scheduler_controls_to_bsi(
    scheduler: &NoneDrcSchedulerFrameOutput,
) -> NoneDrcBsiProjection {
    NoneDrcBsiProjection {
        rf_high_byte: project_u16_high_byte(scheduler.rf_packed_word_u16),
        line_high_bytes: scheduler.line_packed_words_u16.map(project_u16_high_byte),
    }
}

/// Returns the high byte of one U16 control word, exactly `word >> 8`.
pub const fn project_u16_high_byte(word: u16) -> u8 {
    (word >> 8) as u8
}
