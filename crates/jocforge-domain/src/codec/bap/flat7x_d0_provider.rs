// ETSI TS 102 366 V1.4.1 (2017-09), Tables 6.6–6.12 and 6.14–6.16.
// This D0-only table set is independently materialized from the public standard.

use sha2::{Digest, Sha256};

use super::{
    BapBand, BapError, BapParameters, BapProfileInput, BapRowInput, compute_bap_row,
    validate_caller_context, validate_candidate_codes, validate_exponent_row,
};

pub const FLAT7X_D0_BAP_PROFILE_ID: &str = "FLAT7X_D0_FBW253_4CH_NO_LFE_V1";
pub const FLAT7X_D0_BAP_BIN_COUNT: usize = 253;
pub const FLAT7X_D0_BAP_CHANNEL_COUNT: usize = 4;
pub const FLAT7X_D0_BAP_BLOCK_COUNT: usize = 6;
pub const FLAT7X_D0_BAP_TABLE_SET_SHA256: &str =
    "5a9ea2d179dc8017316006e8ffd23b2f5e30f1b363e56740f94787b4ef8e96fe";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xD0BapParameters {
    tables: BapParameters,
}

impl Flat7xD0BapParameters {
    pub fn etsi_ts_102_366_v1_4_1() -> Self {
        let mut tables = BapParameters {
            provider_sha256: [0; 32],
            slow_decay: D0_SLOW_DECAY,
            fast_decay: D0_FAST_DECAY,
            slow_gain: D0_SLOW_GAIN,
            db_knee: D0_DB_KNEE,
            floor: D0_FLOOR,
            fast_gain: D0_FAST_GAIN,
            log_addition: D0_LOG_ADDITION,
            bands: D0_BANDS,
            hearing_48k: D0_HEARING_48K,
            bap_pointer: D0_BAP_POINTER,
        };
        tables.provider_sha256 = table_set_sha256(&tables);
        Self { tables }
    }

    pub const fn table_set_sha256(&self) -> [u8; 32] {
        self.tables.provider_sha256
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0BapRowInput<'a> {
    pub decoded_exponents: &'a [u8; FLAT7X_D0_BAP_BIN_COUNT],
    pub candidate: u16,
    pub slow_decay_code: u8,
    pub fast_decay_code: u8,
    pub slow_gain_code: u8,
    pub db_per_bit_code: u8,
    pub floor_code: u8,
    pub fast_gain_code: u8,
    pub caller_all_active_fine_zero: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0BapFrameInput<'a> {
    pub decoded_exponents_by_block: &'a [[[u8; FLAT7X_D0_BAP_BIN_COUNT]; FLAT7X_D0_BAP_CHANNEL_COUNT];
            FLAT7X_D0_BAP_BLOCK_COUNT],
    pub candidate: u16,
    pub slow_decay_code: u8,
    pub fast_decay_code: u8,
    pub slow_gain_code: u8,
    pub db_per_bit_codes: [u8; FLAT7X_D0_BAP_BLOCK_COUNT],
    pub floor_code: u8,
    pub fast_gain_code: u8,
    pub caller_all_active_fine_zero_by_block: [bool; FLAT7X_D0_BAP_BLOCK_COUNT],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0BapBlock {
    rows: [[u8; FLAT7X_D0_BAP_BIN_COUNT]; FLAT7X_D0_BAP_CHANNEL_COUNT],
}

impl Flat7xD0BapBlock {
    pub const fn rows(&self) -> &[[u8; FLAT7X_D0_BAP_BIN_COUNT]; FLAT7X_D0_BAP_CHANNEL_COUNT] {
        &self.rows
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xD0BapFrame {
    candidate: u16,
    table_set_sha256: [u8; 32],
    blocks: [Flat7xD0BapBlock; FLAT7X_D0_BAP_BLOCK_COUNT],
}

impl Flat7xD0BapFrame {
    pub const fn candidate(&self) -> u16 {
        self.candidate
    }

    pub const fn table_set_sha256(&self) -> [u8; 32] {
        self.table_set_sha256
    }

    pub const fn blocks(&self) -> &[Flat7xD0BapBlock; FLAT7X_D0_BAP_BLOCK_COUNT] {
        &self.blocks
    }
}

pub fn allocate_flat7x_d0_bap_row(
    parameters: &Flat7xD0BapParameters,
    input: &Flat7xD0BapRowInput<'_>,
) -> Result<[u8; FLAT7X_D0_BAP_BIN_COUNT], BapError> {
    validate_exponent_row(input.decoded_exponents, FLAT7X_D0_BAP_BIN_COUNT)?;
    validate_candidate_codes(
        input.candidate,
        input.slow_decay_code,
        input.fast_decay_code,
        input.slow_gain_code,
        input.db_per_bit_code,
        input.floor_code,
        input.fast_gain_code,
    )?;
    validate_caller_context(input.candidate, input.caller_all_active_fine_zero)?;
    let generic_input = BapRowInput {
        decoded_exponents: input.decoded_exponents,
        end_exclusive: FLAT7X_D0_BAP_BIN_COUNT,
        candidate: input.candidate,
        slow_decay_code: input.slow_decay_code,
        fast_decay_code: input.fast_decay_code,
        slow_gain_code: input.slow_gain_code,
        db_per_bit_code: input.db_per_bit_code,
        floor_code: input.floor_code,
        fast_gain_code: input.fast_gain_code,
        caller_all_active_fine_zero: input.caller_all_active_fine_zero,
        profile: BapProfileInput::selected(),
    };
    compute_bap_row(&generic_input, &parameters.tables)?
        .try_into()
        .map_err(|_| BapError::ParameterError)
}

pub fn allocate_flat7x_d0_bap_frame(
    parameters: &Flat7xD0BapParameters,
    input: &Flat7xD0BapFrameInput<'_>,
) -> Result<Flat7xD0BapFrame, BapError> {
    validate_candidate_codes(
        input.candidate,
        input.slow_decay_code,
        input.fast_decay_code,
        input.slow_gain_code,
        input.db_per_bit_codes[0],
        input.floor_code,
        input.fast_gain_code,
    )?;
    if input.db_per_bit_codes.iter().any(|code| *code > 3) {
        return Err(BapError::InputError);
    }
    let expected_all_active_fine_zero = input.candidate.trailing_zeros() >= 4;
    if input
        .caller_all_active_fine_zero_by_block
        .iter()
        .any(|value| *value != expected_all_active_fine_zero)
    {
        return Err(BapError::InputError);
    }

    let mut blocks = [Flat7xD0BapBlock {
        rows: [[0; FLAT7X_D0_BAP_BIN_COUNT]; FLAT7X_D0_BAP_CHANNEL_COUNT],
    }; FLAT7X_D0_BAP_BLOCK_COUNT];
    for (block_index, block) in blocks.iter_mut().enumerate() {
        for channel in 0..FLAT7X_D0_BAP_CHANNEL_COUNT {
            block.rows[channel] = allocate_flat7x_d0_bap_row(
                parameters,
                &Flat7xD0BapRowInput {
                    decoded_exponents: &input.decoded_exponents_by_block[block_index][channel],
                    candidate: input.candidate,
                    slow_decay_code: input.slow_decay_code,
                    fast_decay_code: input.fast_decay_code,
                    slow_gain_code: input.slow_gain_code,
                    db_per_bit_code: input.db_per_bit_codes[block_index],
                    floor_code: input.floor_code,
                    fast_gain_code: input.fast_gain_code,
                    caller_all_active_fine_zero: input.caller_all_active_fine_zero_by_block
                        [block_index],
                },
            )?;
        }
    }
    Ok(Flat7xD0BapFrame {
        candidate: input.candidate,
        table_set_sha256: parameters.table_set_sha256(),
        blocks,
    })
}

fn table_set_sha256(parameters: &BapParameters) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(FLAT7X_D0_BAP_PROFILE_ID.as_bytes());
    digest.update([0]);
    digest.update(b"ETSI TS 102 366 V1.4.1");
    digest.update([0]);
    for value in parameters
        .slow_decay
        .iter()
        .chain(parameters.fast_decay.iter())
        .chain(parameters.slow_gain.iter())
        .chain(parameters.db_knee.iter())
        .chain(parameters.floor.iter())
        .chain(parameters.fast_gain.iter())
        .chain(parameters.log_addition.iter())
        .chain(parameters.hearing_48k.iter())
    {
        digest.update(value.to_be_bytes());
    }
    for band in &parameters.bands {
        digest.update(band.start.to_be_bytes());
        digest.update([band.size]);
    }
    digest.update(parameters.bap_pointer);
    digest.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::{BapBand, Flat7xD0BapParameters};
    use crate::codec::bap::validate_band_coverage;

    #[test]
    fn etsi_table_landmarks_and_253_bin_coverage_are_bound() {
        let parameters = Flat7xD0BapParameters::etsi_ts_102_366_v1_4_1();
        let tables = &parameters.tables;

        assert_eq!(tables.slow_decay, [0x0f, 0x11, 0x13, 0x15]);
        assert_eq!(tables.fast_decay, [0x3f, 0x53, 0x67, 0x7b]);
        assert_eq!(tables.bands[0], BapBand { start: 0, size: 1 });
        assert_eq!(
            tables.bands[49],
            BapBand {
                start: 229,
                size: 24
            }
        );
        assert_eq!(tables.log_addition[0], 0x0040);
        assert_eq!(tables.log_addition[255], 0);
        assert_eq!(tables.hearing_48k[0], 0x04d0);
        assert_eq!(tables.hearing_48k[49], 0x0840);
        assert_eq!(tables.bap_pointer[0], 0);
        assert_eq!(tables.bap_pointer[63], 15);
        validate_band_coverage(&tables.bands).unwrap();
    }
}

const D0_SLOW_DECAY: [i16; 4] = [0x000f, 0x0011, 0x0013, 0x0015];
const D0_FAST_DECAY: [i16; 4] = [0x003f, 0x0053, 0x0067, 0x007b];
const D0_SLOW_GAIN: [i16; 4] = [0x0540, 0x04d8, 0x0478, 0x0410];
const D0_DB_KNEE: [i16; 4] = [0x0000, 0x0700, 0x0900, 0x0b00];
const D0_FLOOR: [i16; 8] = [
    0x02f0, 0x02b0, 0x0270, 0x0230, 0x01f0, 0x0170, 0x00f0, -2048,
];
const D0_FAST_GAIN: [i16; 8] = [
    0x0080, 0x0100, 0x0180, 0x0200, 0x0280, 0x0300, 0x0380, 0x0400,
];
const D0_LOG_ADDITION: [i16; 256] = [
    0x0040, 0x003f, 0x003e, 0x003d, 0x003c, 0x003b, 0x003a, 0x0039, 0x0038, 0x0037, 0x0036, 0x0035,
    0x0034, 0x0034, 0x0033, 0x0032, 0x0031, 0x0030, 0x002f, 0x002f, 0x002e, 0x002d, 0x002c, 0x002c,
    0x002b, 0x002a, 0x0029, 0x0029, 0x0028, 0x0027, 0x0026, 0x0026, 0x0025, 0x0024, 0x0024, 0x0023,
    0x0023, 0x0022, 0x0021, 0x0021, 0x0020, 0x0020, 0x001f, 0x001e, 0x001e, 0x001d, 0x001d, 0x001c,
    0x001c, 0x001b, 0x001b, 0x001a, 0x001a, 0x0019, 0x0019, 0x0018, 0x0018, 0x0017, 0x0017, 0x0016,
    0x0016, 0x0015, 0x0015, 0x0015, 0x0014, 0x0014, 0x0013, 0x0013, 0x0013, 0x0012, 0x0012, 0x0012,
    0x0011, 0x0011, 0x0011, 0x0010, 0x0010, 0x0010, 0x000f, 0x000f, 0x000f, 0x000e, 0x000e, 0x000e,
    0x000d, 0x000d, 0x000d, 0x000d, 0x000c, 0x000c, 0x000c, 0x000c, 0x000b, 0x000b, 0x000b, 0x000b,
    0x000a, 0x000a, 0x000a, 0x000a, 0x000a, 0x0009, 0x0009, 0x0009, 0x0009, 0x0009, 0x0008, 0x0008,
    0x0008, 0x0008, 0x0008, 0x0008, 0x0007, 0x0007, 0x0007, 0x0007, 0x0007, 0x0007, 0x0006, 0x0006,
    0x0006, 0x0006, 0x0006, 0x0006, 0x0006, 0x0006, 0x0005, 0x0005, 0x0005, 0x0005, 0x0005, 0x0005,
    0x0005, 0x0005, 0x0004, 0x0004, 0x0004, 0x0004, 0x0004, 0x0004, 0x0004, 0x0004, 0x0004, 0x0004,
    0x0004, 0x0003, 0x0003, 0x0003, 0x0003, 0x0003, 0x0003, 0x0003, 0x0003, 0x0003, 0x0003, 0x0003,
    0x0003, 0x0003, 0x0003, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002,
    0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0002, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000,
];
const D0_HEARING_48K: [i16; 50] = [
    0x04d0, 0x04d0, 0x0440, 0x0400, 0x03e0, 0x03c0, 0x03b0, 0x03b0, 0x03a0, 0x03a0, 0x03a0, 0x03a0,
    0x03a0, 0x0390, 0x0390, 0x0390, 0x0380, 0x0380, 0x0370, 0x0370, 0x0360, 0x0360, 0x0350, 0x0350,
    0x0340, 0x0340, 0x0330, 0x0320, 0x0310, 0x0300, 0x02f0, 0x02f0, 0x02f0, 0x02f0, 0x0300, 0x0310,
    0x0340, 0x0390, 0x03e0, 0x0420, 0x0460, 0x0490, 0x04a0, 0x0460, 0x0440, 0x0440, 0x0520, 0x0800,
    0x0840, 0x0840,
];
const D0_BAP_POINTER: [u8; 64] = [
    0, 1, 1, 1, 1, 1, 2, 2, 3, 3, 3, 4, 4, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 9, 9, 9, 9,
    10, 10, 10, 10, 11, 11, 11, 11, 12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14, 14, 14, 14, 14,
    15, 15, 15, 15, 15, 15, 15, 15, 15,
];
const D0_BANDS: [BapBand; 50] = [
    BapBand { start: 0, size: 1 },
    BapBand { start: 1, size: 1 },
    BapBand { start: 2, size: 1 },
    BapBand { start: 3, size: 1 },
    BapBand { start: 4, size: 1 },
    BapBand { start: 5, size: 1 },
    BapBand { start: 6, size: 1 },
    BapBand { start: 7, size: 1 },
    BapBand { start: 8, size: 1 },
    BapBand { start: 9, size: 1 },
    BapBand { start: 10, size: 1 },
    BapBand { start: 11, size: 1 },
    BapBand { start: 12, size: 1 },
    BapBand { start: 13, size: 1 },
    BapBand { start: 14, size: 1 },
    BapBand { start: 15, size: 1 },
    BapBand { start: 16, size: 1 },
    BapBand { start: 17, size: 1 },
    BapBand { start: 18, size: 1 },
    BapBand { start: 19, size: 1 },
    BapBand { start: 20, size: 1 },
    BapBand { start: 21, size: 1 },
    BapBand { start: 22, size: 1 },
    BapBand { start: 23, size: 1 },
    BapBand { start: 24, size: 1 },
    BapBand { start: 25, size: 1 },
    BapBand { start: 26, size: 1 },
    BapBand { start: 27, size: 1 },
    BapBand { start: 28, size: 3 },
    BapBand { start: 31, size: 3 },
    BapBand { start: 34, size: 3 },
    BapBand { start: 37, size: 3 },
    BapBand { start: 40, size: 3 },
    BapBand { start: 43, size: 3 },
    BapBand { start: 46, size: 3 },
    BapBand { start: 49, size: 6 },
    BapBand { start: 55, size: 6 },
    BapBand { start: 61, size: 6 },
    BapBand { start: 67, size: 6 },
    BapBand { start: 73, size: 6 },
    BapBand { start: 79, size: 6 },
    BapBand {
        start: 85,
        size: 12,
    },
    BapBand {
        start: 97,
        size: 12,
    },
    BapBand {
        start: 109,
        size: 12,
    },
    BapBand {
        start: 121,
        size: 12,
    },
    BapBand {
        start: 133,
        size: 24,
    },
    BapBand {
        start: 157,
        size: 24,
    },
    BapBand {
        start: 181,
        size: 24,
    },
    BapBand {
        start: 205,
        size: 24,
    },
    BapBand {
        start: 229,
        size: 24,
    },
];
