use jocforge_domain::drc::none_drc_bsi_projection::{
    NoneDrcBsiProjection, project_scheduler_controls_to_bsi, project_u16_high_byte,
};
use jocforge_domain::drc::none_drc_scheduler::NoneDrcSchedulerFrameOutput;

#[test]
fn scheduler_projection_reads_only_rf_and_six_line_control_words() {
    let scheduler = NoneDrcSchedulerFrameOutput {
        rf_packed_word_u16: 0x23A5,
        rf_flag_u16: 0xBEEF,
        line_packed_words_u16: [0x0ABC, 0x1357, 0x2468, 0x9ABC, 0xFEDC, 0x8051],
        line_flags_u16: [0xFF00, 0xFF01, 0xFF02, 0xFF03, 0xFF04, 0xFF05],
        rf_aux_packed_word_u16: 0xD00D,
        rf_limiter_packed_word_u16: 0xE00E,
        line_aux_packed_words_u16: [0xA001, 0xA102, 0xA203, 0xA304, 0xA405, 0xA506],
        line_limiter_packed_words_u16: [0xB001, 0xB102, 0xB203, 0xB304, 0xB405, 0xB506],
        line_minimum_signed_word: -12_345,
        peak_logs_six_words: [0x3F80_0000; 6],
        line_gain_words: [0x0000_0000; 6],
    };

    assert_eq!(
        project_scheduler_controls_to_bsi(&scheduler),
        NoneDrcBsiProjection {
            rf_high_byte: 0x23,
            line_high_bytes: [0x0A, 0x13, 0x24, 0x9A, 0xFE, 0x80],
        }
    );
}

#[test]
fn high_byte_mapping_is_exhaustive_over_the_u16_domain() {
    let mut word_counts_per_byte = [0_u16; 256];
    for word in u16::MIN..=u16::MAX {
        let projected = project_u16_high_byte(word);
        assert_eq!(projected, word.to_be_bytes()[0]);
        word_counts_per_byte[usize::from(projected)] += 1;
    }

    assert_eq!(word_counts_per_byte, [256; 256]);
}
