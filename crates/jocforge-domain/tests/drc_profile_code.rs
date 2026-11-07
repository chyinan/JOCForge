// pattern: Functional Core
use jocforge_domain::drc::drc_profile_code::{
    DrcProfileNativeCode, LineDdpComprCode, LineDrcSerializedCode, line_drc_compr_code,
    line_drc_serialized_code, native_drc_profile_code,
};

#[test]
fn explicit_drc_profile_strings_map_to_the_washed_native_u32_values() {
    for (profile, expected) in [
        ("film_light", 0_u32),
        ("film_standard", 1),
        ("music_light", 2),
        ("music_standard", 3),
        ("speech", 4),
        ("none", 5),
    ] {
        assert_eq!(
            native_drc_profile_code(profile),
            Some(DrcProfileNativeCode::new(expected))
        );
    }
}

#[test]
fn washed_line_profile_compr_and_serialized_values_are_explicit_categories() {
    for (profile, compr, serialized) in [
        ("film_standard", 1, 0b001),
        ("film_light", 2, 0b010),
        ("music_standard", 3, 0b011),
        ("music_light", 4, 0b100),
        ("speech", 5, 0b101),
        ("none", 0, 0b000),
    ] {
        assert_eq!(
            line_drc_compr_code(profile).map(LineDdpComprCode::value),
            Some(compr)
        );
        assert_eq!(
            line_drc_serialized_code(profile).map(LineDrcSerializedCode::value),
            Some(serialized)
        );
    }
}

#[test]
fn unregistered_profile_values_have_no_fallback() {
    assert_eq!(native_drc_profile_code(""), None);
    assert_eq!(native_drc_profile_code("FILM_LIGHT"), None);
    assert_eq!(native_drc_profile_code("auto"), None);
    assert_eq!(line_drc_compr_code("auto"), None);
    assert_eq!(line_drc_serialized_code("auto"), None);
}
