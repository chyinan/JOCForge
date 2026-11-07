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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineDdpComprCode(u32);

impl LineDdpComprCode {
    pub const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineDrcSerializedCode(u8);

impl LineDrcSerializedCode {
    pub const fn value(self) -> u8 {
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

/// Maps one exact Line profile label to the washed DDP_COMPR U32 category.
///
/// This table is static evidence only. It does not prove selected-plugin transfer, mode-1 path
/// selection, serializer activation, or PCM/audio effect.
pub fn line_drc_compr_code(profile: &str) -> Option<LineDdpComprCode> {
    let code = match profile {
        "film_standard" => 1,
        "film_light" => 2,
        "music_standard" => 3,
        "music_light" => 4,
        "speech" => 5,
        "none" => 0,
        _ => return None,
    };
    Some(LineDdpComprCode(code))
}

/// Maps one exact Line profile label to its washed three-bit value if serialized.
///
/// The returned code does not imply that a selected job reaches the guarded serializer path.
pub fn line_drc_serialized_code(profile: &str) -> Option<LineDrcSerializedCode> {
    let code = match profile {
        "film_standard" => 0b001,
        "film_light" => 0b010,
        "music_standard" => 0b011,
        "music_light" => 0b100,
        "speech" => 0b101,
        "none" => 0b000,
        _ => return None,
    };
    Some(LineDrcSerializedCode(code))
}
