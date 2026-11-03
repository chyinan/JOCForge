// pattern: Functional Core
use jocforge_domain::drc::drc_profile_code::{DrcProfileNativeCode, native_drc_profile_code};

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
fn unregistered_profile_values_have_no_fallback() {
    assert_eq!(native_drc_profile_code(""), None);
    assert_eq!(native_drc_profile_code("FILM_LIGHT"), None);
    assert_eq!(native_drc_profile_code("auto"), None);
}
