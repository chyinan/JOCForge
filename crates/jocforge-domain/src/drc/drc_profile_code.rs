// pattern: Functional Core
//! Explicit string-to-U32 DRC profile mapping, without default selection or output wiring.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrcProfileNativeCode(u32);

impl DrcProfileNativeCode {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

/// Maps one explicit profile name to its washed native U32 code.
///
/// This value map does not choose a default, bind a host/XML property, or imply a BSI/audio
/// effect. Unknown spellings produce no map value.
pub fn native_drc_profile_code(profile: &str) -> Option<DrcProfileNativeCode> {
    let code = match profile {
        "film_light" => 0,
        "film_standard" => 1,
        "music_light" => 2,
        "music_standard" => 3,
        "speech" => 4,
        "none" => 5,
        _ => return None,
    };
    Some(DrcProfileNativeCode::new(code))
}
