// pattern: Functional Core
use jocforge_domain::transform::lfe_transform::{
    LFE_TERMINAL_WRITE_ORDER, project_companion_lfe_terminal_window,
    project_main_lfe_terminal_window,
};

#[test]
fn terminal_projection_one_hot_vectors_match_the_bound_f32_coefficients() {
    assert_eq!(LFE_TERMINAL_WRITE_ORDER, [0, 2, 4, 6, 5, 3, 1]);

    let one_hot_cases = [
        (0, 0, 0x3ac9_0fd5, 0xbac9_0fd5),
        (1, 0, 0xbf7f_ffec, 0x3f7f_ffec),
        (2, 2, 0x3c62_2fff, 0xbc62_2fff),
        (3, 2, 0xbf7f_f9c1, 0x3f7f_f9c1),
        (4, 4, 0x3cd5_9aa6, 0xbcd5_9aa6),
        (5, 4, 0xbf7f_e9b8, 0x3f7f_e9b8),
        (6, 6, 0x3d1d_0a88, 0xbd1d_0a88),
        (7, 6, 0xbf7f_cfd1, 0x3f7f_cfd1),
        (10, 5, 0xbd10_7bb8, 0xbd10_7bb8),
        (11, 5, 0xbf7f_d737, 0xbf7f_d737),
        (12, 3, 0xbcbc_7a9b, 0xbcbc_7a9b),
        (13, 3, 0xbf7f_eea7, 0xbf7f_eea7),
        (14, 1, 0xbc2f_ed02, 0xbc2f_ed02),
        (15, 1, 0xbf7f_fc39, 0xbf7f_fc39),
    ];
    for (source_index, output_index, expected_main, expected_companion) in one_hot_cases {
        let mut input = [0.0_f32; 16];
        input[source_index] = 1.0;
        let main = project_main_lfe_terminal_window(&input);
        let companion = project_companion_lfe_terminal_window(&input);
        assert_eq!(
            main[output_index].to_bits(),
            expected_main,
            "main x{source_index}"
        );
        assert_eq!(
            companion[output_index].to_bits(),
            expected_companion,
            "companion x{source_index}"
        );
        assert!(
            main.iter()
                .enumerate()
                .all(|(index, value)| { index == output_index || *value == 0.0 })
        );
        assert!(
            companion
                .iter()
                .enumerate()
                .all(|(index, value)| { index == output_index || *value == 0.0 })
        );
    }

    for source_index in [8_usize, 9] {
        let mut input = [0.0_f32; 16];
        input[source_index] = 1.0;
        assert!(
            project_main_lfe_terminal_window(&input)
                .iter()
                .all(|value| *value == 0.0)
        );
        assert!(
            project_companion_lfe_terminal_window(&input)
                .iter()
                .all(|value| *value == 0.0)
        );
    }
}
