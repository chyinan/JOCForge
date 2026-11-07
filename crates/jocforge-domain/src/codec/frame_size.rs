// pattern: Functional Core
//! Typed E-AC-3 frame-size projection for the admitted carrier-rate profile.

use crate::profile::CodecRateProfile;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Eac3FrameSizeCode(u16);

impl Eac3FrameSizeCode {
    pub const fn new(value: u16) -> Option<Self> {
        if value <= 0x07ff {
            Some(Self(value))
        } else {
            None
        }
    }

    pub const fn value(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Eac3FrameSize {
    code: Eac3FrameSizeCode,
    bytes: usize,
    codec_rate: CodecRateProfile,
}

impl Eac3FrameSize {
    pub const fn from_codec_rate(rate: CodecRateProfile) -> Option<Self> {
        match (rate.frame_size_code(), rate.frame_bytes()) {
            (Some(code), Some(bytes)) if bytes >= 2 => match Eac3FrameSizeCode::new(code) {
                Some(code) => Some(Self {
                    code,
                    bytes,
                    codec_rate: rate,
                }),
                None => None,
            },
            _ => None,
        }
    }

    pub const fn from_frame_bytes(bytes: usize) -> Option<Self> {
        match bytes {
            1_536 => Some(Self {
                code: Eac3FrameSizeCode(767),
                bytes,
                codec_rate: CodecRateProfile::Kbps384,
            }),
            1_792 => Some(Self {
                code: Eac3FrameSizeCode(895),
                bytes,
                codec_rate: CodecRateProfile::Kbps448,
            }),
            2_304 => Some(Self {
                code: Eac3FrameSizeCode(1_151),
                bytes,
                codec_rate: CodecRateProfile::Kbps576,
            }),
            2_560 => Some(Self {
                code: Eac3FrameSizeCode(1_279),
                bytes,
                codec_rate: CodecRateProfile::Kbps640,
            }),
            3_072 => Some(Self {
                code: Eac3FrameSizeCode(1_535),
                bytes,
                codec_rate: CodecRateProfile::Kbps768,
            }),
            4_096 => Some(Self {
                code: Eac3FrameSizeCode(2_047),
                bytes,
                codec_rate: CodecRateProfile::Kbps1024,
            }),
            _ => None,
        }
    }

    pub const fn code(self) -> Eac3FrameSizeCode {
        self.code
    }

    pub const fn bytes(self) -> usize {
        self.bytes
    }

    pub const fn codec_rate(self) -> CodecRateProfile {
        self.codec_rate
    }

    pub const fn bits(self) -> u32 {
        (self.bytes * 8) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::Eac3FrameSize;
    use crate::profile::CodecRateProfile;

    #[test]
    fn washed_single_rate_rows_project_to_observed_frame_size_fields() {
        let rows = [
            (CodecRateProfile::Kbps384, 767, 1_536),
            (CodecRateProfile::Kbps448, 895, 1_792),
            (CodecRateProfile::Kbps576, 1_151, 2_304),
            (CodecRateProfile::Kbps640, 1_279, 2_560),
            (CodecRateProfile::Kbps768, 1_535, 3_072),
            (CodecRateProfile::Kbps1024, 2_047, 4_096),
        ];

        for (rate, expected_code, expected_bytes) in rows {
            let size = Eac3FrameSize::from_codec_rate(rate).unwrap();
            assert_eq!(size.code().value(), expected_code);
            assert_eq!(size.bytes(), expected_bytes);
            assert_eq!(size.bits(), (expected_bytes * 8) as u32);
        }

        assert!(Eac3FrameSize::from_codec_rate(CodecRateProfile::Unsupported).is_none());
    }
}
