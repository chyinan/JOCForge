// pattern: Functional Core
//! Selected source configuration record import into caller-owned owner and metadata bytes.

use crate::numeric::{FiniteF32, r32_mul};

/// Fixed byte extent of the selected source configuration record.
pub const SOURCE_CONFIGURATION_BYTES: usize = 2752;

/// Fixed byte extent of the selected encoder owner buffer.
pub const ENCODER_OWNER_BYTES: usize = 4096;

/// Fixed byte extent of the selected metadata buffer.
pub const METADATA_BUFFER_BYTES: usize = 64;

const ADVANCED_OWNER_BASE: usize = 272;
const ADVANCED_SOURCE_BASE: usize = 280;

const RATE_SCALE_F32_BITS: [u32; 4] = [1_051_372_203, 1_052_361_336, 1_056_964_608, 0];
const BLOCK_THRESHOLD_I16: [i16; 12] = [
    6144, 3072, 2048, 1024, 5644, 2822, 1881, 940, 4096, 2048, 1365, 682,
];
const BLOCK_SIZE_I16: [i16; 6] = [6, 3, 2, -1, -1, 1];
const CHANNEL_COUNT_A_I16: [i16; 16] = [2, 1, 2, 3, 2, 3, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0];
const CHANNEL_COUNT_B_I16: [i16; 16] = [0, 0, 0, 0, 1, 1, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0];
const COMPATIBILITY_RATES_I16: [i16; 19] = [
    32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384, 448, 512, 576, 640,
];

#[derive(Clone, Copy)]
struct CopyField {
    owner_offset: usize,
    source_offset: usize,
    bytes: usize,
}

const BASIC_COPY_ORDER: [CopyField; 32] = [
    CopyField {
        owner_offset: 4,
        source_offset: 12,
        bytes: 4,
    },
    CopyField {
        owner_offset: 8,
        source_offset: 16,
        bytes: 4,
    },
    CopyField {
        owner_offset: 12,
        source_offset: 20,
        bytes: 4,
    },
    CopyField {
        owner_offset: 16,
        source_offset: 24,
        bytes: 2,
    },
    CopyField {
        owner_offset: 20,
        source_offset: 28,
        bytes: 4,
    },
    CopyField {
        owner_offset: 24,
        source_offset: 32,
        bytes: 4,
    },
    CopyField {
        owner_offset: 28,
        source_offset: 36,
        bytes: 4,
    },
    CopyField {
        owner_offset: 32,
        source_offset: 40,
        bytes: 4,
    },
    CopyField {
        owner_offset: 36,
        source_offset: 44,
        bytes: 4,
    },
    CopyField {
        owner_offset: 44,
        source_offset: 52,
        bytes: 4,
    },
    CopyField {
        owner_offset: 48,
        source_offset: 56,
        bytes: 4,
    },
    CopyField {
        owner_offset: 52,
        source_offset: 60,
        bytes: 4,
    },
    CopyField {
        owner_offset: 56,
        source_offset: 64,
        bytes: 4,
    },
    CopyField {
        owner_offset: 60,
        source_offset: 68,
        bytes: 4,
    },
    CopyField {
        owner_offset: 128,
        source_offset: 136,
        bytes: 4,
    },
    CopyField {
        owner_offset: 64,
        source_offset: 72,
        bytes: 4,
    },
    CopyField {
        owner_offset: 68,
        source_offset: 76,
        bytes: 4,
    },
    CopyField {
        owner_offset: 72,
        source_offset: 80,
        bytes: 4,
    },
    CopyField {
        owner_offset: 76,
        source_offset: 84,
        bytes: 4,
    },
    CopyField {
        owner_offset: 80,
        source_offset: 88,
        bytes: 4,
    },
    CopyField {
        owner_offset: 84,
        source_offset: 92,
        bytes: 4,
    },
    CopyField {
        owner_offset: 88,
        source_offset: 96,
        bytes: 4,
    },
    CopyField {
        owner_offset: 92,
        source_offset: 100,
        bytes: 4,
    },
    CopyField {
        owner_offset: 96,
        source_offset: 104,
        bytes: 4,
    },
    CopyField {
        owner_offset: 100,
        source_offset: 108,
        bytes: 4,
    },
    CopyField {
        owner_offset: 104,
        source_offset: 112,
        bytes: 4,
    },
    CopyField {
        owner_offset: 108,
        source_offset: 116,
        bytes: 4,
    },
    CopyField {
        owner_offset: 112,
        source_offset: 120,
        bytes: 4,
    },
    CopyField {
        owner_offset: 116,
        source_offset: 124,
        bytes: 4,
    },
    CopyField {
        owner_offset: 120,
        source_offset: 128,
        bytes: 4,
    },
    CopyField {
        owner_offset: 124,
        source_offset: 132,
        bytes: 4,
    },
    CopyField {
        owner_offset: 136,
        source_offset: 144,
        bytes: 4,
    },
];

const BASIC_TAIL_COPY_ORDER: [CopyField; 10] = [
    CopyField {
        owner_offset: 148,
        source_offset: 156,
        bytes: 4,
    },
    CopyField {
        owner_offset: 132,
        source_offset: 140,
        bytes: 4,
    },
    CopyField {
        owner_offset: 152,
        source_offset: 160,
        bytes: 4,
    },
    CopyField {
        owner_offset: 156,
        source_offset: 164,
        bytes: 4,
    },
    CopyField {
        owner_offset: 160,
        source_offset: 168,
        bytes: 4,
    },
    CopyField {
        owner_offset: 164,
        source_offset: 172,
        bytes: 4,
    },
    CopyField {
        owner_offset: 168,
        source_offset: 176,
        bytes: 4,
    },
    CopyField {
        owner_offset: 172,
        source_offset: 180,
        bytes: 4,
    },
    CopyField {
        owner_offset: 176,
        source_offset: 184,
        bytes: 2,
    },
    CopyField {
        owner_offset: 180,
        source_offset: 188,
        bytes: 4,
    },
];

const ADVANCED_ORDERED_DWORD_OFFSETS: [usize; 23] = [
    4, 8, 12, 16, 20, 24, 32, 28, 36, 40, 44, 48, 52, 56, 64, 60, 68, 72, 76, 80, 84, 88, 244,
];

const ADVANCED_FINAL_COPY_ORDER: [(usize, usize); 11] = [
    (176, 4),
    (180, 12),
    (192, 2),
    (196, 4),
    (200, 2),
    (208, 8),
    (236, 4),
    (240, 4),
    (2456, 4),
    (2452, 4),
    (2460, 4),
];

/// Imports the selected source record into caller-owned buffers.
///
/// The source is a fixed 2,752-byte record; owner and metadata begin in whatever state the
/// caller supplied. Version-prefix rejection returns before any write. Other failures retain
/// exactly the writes already performed by the ordered import; there is no rollback.
///
/// # Panics
///
/// Panics if the caller violates the package precondition that the selected frequency and
/// format index the supplied tables, or if a selected arithmetic intermediate cannot be
/// represented in its declared integer/F32 domain. Those out-of-domain cases have no washed
/// native error mapping.
pub fn import_selected_configuration(
    source: &[u8; SOURCE_CONFIGURATION_BYTES],
    owner: &mut [u8; ENCODER_OWNER_BYTES],
    metadata: &mut [u8; METADATA_BUFFER_BYTES],
) -> i32 {
    let version = read_i16(source, 2);
    if version < 512 {
        return 140;
    }

    for field in BASIC_COPY_ORDER {
        copy_field(source, owner, field);
    }

    let first_normalized = match read_i32(source, 148) {
        -1 => 51,
        value => value,
    };
    write_i32(owner, 140, first_normalized);
    let second_normalized = match read_i32(source, 152) {
        -1 => 51,
        value => value,
    };
    write_i32(owner, 144, second_normalized);

    for field in BASIC_TAIL_COPY_ORDER {
        copy_field(source, owner, field);
    }

    if version >= 514 {
        for owner_offset in (188..=228).step_by(4) {
            copy_bytes(source, owner, owner_offset + 8, owner_offset, 4);
        }
    }

    for relative_offset in ADVANCED_ORDERED_DWORD_OFFSETS {
        copy_bytes(
            source,
            owner,
            ADVANCED_SOURCE_BASE + relative_offset,
            ADVANCED_OWNER_BASE + relative_offset,
            4,
        );
    }

    for row in 0..6 {
        copy_bytes(
            source,
            owner,
            ADVANCED_SOURCE_BASE + 92 + 2 * row,
            ADVANCED_OWNER_BASE + 92 + 2 * row,
            2,
        );
        copy_bytes(
            source,
            owner,
            ADVANCED_SOURCE_BASE + 248 + 2 * row,
            ADVANCED_OWNER_BASE + 248 + 2 * row,
            2,
        );
        for column in 0..6 {
            let matrix_offset = 104 + 2 * (6 * row + column);
            copy_bytes(
                source,
                owner,
                ADVANCED_SOURCE_BASE + matrix_offset,
                ADVANCED_OWNER_BASE + matrix_offset,
                2,
            );
            let second_matrix_offset = 260 + 2 * (6 * row + column);
            copy_bytes(
                source,
                owner,
                ADVANCED_SOURCE_BASE + second_matrix_offset,
                ADVANCED_OWNER_BASE + second_matrix_offset,
                2,
            );
        }
    }

    if version >= 514 {
        copy_advanced(source, owner, 244, 4);
        copy_advanced(source, owner, 248, 12);
        copy_advanced(source, owner, 260, 72);
        copy_advanced(source, owner, 332, 2);
        copy_advanced(source, owner, 334, 2);
        copy_advanced(source, owner, 336, 64);
        copy_advanced(source, owner, 400, 2);
        copy_advanced(source, owner, 402, 2);
        copy_advanced(source, owner, 404, 2048);
    }

    for (relative_offset, bytes) in ADVANCED_FINAL_COPY_ORDER {
        copy_advanced(source, owner, relative_offset, bytes);
    }

    let compatibility_class = owner_u32(owner, 4).wrapping_sub(1) & 0xffff_fffd == 0;
    let advanced_selector_offset = ADVANCED_OWNER_BASE + 84;
    if read_i32(owner, advanced_selector_offset) == -1 {
        write_i32(owner, advanced_selector_offset, 0);
    }

    let selector = if compatibility_class {
        6
    } else {
        let frequency = usize::try_from(owner_i32(owner, 12))
            .expect("selected configuration frequency is nonnegative by precondition");
        let rate = owner_i16(owner, 16);
        let mut block = 3_i32;
        while rate > BLOCK_THRESHOLD_I16[4 * frequency + usize::try_from(block).unwrap()] {
            block -= 1;
        }
        if block < 0 {
            return 59;
        }
        if block == 3 { 6 } else { block + 1 }
    };
    write_i32(owner, advanced_selector_offset, selector);

    let block_size = BLOCK_SIZE_I16[usize::try_from(selector - 1).unwrap()];
    write_i16(metadata, 0, block_size);

    let modes = [
        owner_i32(owner, 152),
        owner_i32(owner, 156),
        owner_i32(owner, 160),
        owner_i32(owner, 164),
    ];
    if modes.iter().all(|value| *value == modes[0]) {
        write_i32(metadata, 4, 0);
        write_i32(metadata, 8, modes[0]);
    } else {
        write_i32(metadata, 4, 1);
        write_i32(metadata, 8, 0);
    }
    if owner_u32(owner, ADVANCED_OWNER_BASE + 88) != 0 {
        metadata[4..12].fill(0);
    }
    write_i16(
        metadata,
        12,
        if selector == 6 {
            3
        } else {
            (selector - 1) as i16
        },
    );

    let frequency = usize::try_from(owner_i32(owner, 12))
        .expect("selected configuration frequency is nonnegative by precondition");
    let rate_scale = f32::from_bits(RATE_SCALE_F32_BITS[frequency]);
    let q = if rate_scale < 1.0 {
        let scale_operand = finite(2_147_483_600.0_f32);
        let scaled = r32_mul(finite(rate_scale), scale_operand)
            .expect("selected rate scale product is finite")
            .value();
        let lower_bound = -2_147_483_600.0_f32;
        round_rne_positive_i32(scaled.max(lower_bound))
    } else {
        i32::MAX
    };

    let selector_rate = selector * i32::from(owner_i16(owner, 16));
    let product = i64::from(selector_rate) * i64::from(q);
    let whole = product >> 31;
    let fractional =
        u32::try_from(product & 0x7fff_ffff).expect("fractional part is a 31-bit unsigned value");
    let whole_bytes = whole.to_le_bytes();
    metadata[16..20].copy_from_slice(&whole_bytes[..4]);
    let fractional_word = u32_to_f32_rne(fractional);
    let fractional_scale = f32::from_bits(0x3000_0000);
    let fraction = r32_mul(finite(fractional_word), finite(fractional_scale))
        .expect("fractional metadata scale remains finite")
        .value();
    write_f32(metadata, 20, fraction);
    metadata[24..26].copy_from_slice(&whole_bytes[..2]);

    if !compatibility_class {
        write_i16(metadata, 26, -1);
    } else if let Some(index) = COMPATIBILITY_RATES_I16
        .iter()
        .position(|rate| *rate == owner_i16(owner, 16))
    {
        write_i16(metadata, 26, i16::try_from(index).unwrap());
    } else {
        return 59;
    }

    let all_unmapped = (196..=228)
        .step_by(4)
        .all(|offset| owner_i32(owner, offset) == -1);
    write_i32(owner, 188, if all_unmapped { 0 } else { 3 });

    let channel_map_result = channel_map_return_value(owner);
    if channel_map_result != 0 {
        return channel_map_result;
    }

    0
}

fn copy_field(
    source: &[u8; SOURCE_CONFIGURATION_BYTES],
    owner: &mut [u8; ENCODER_OWNER_BYTES],
    field: CopyField,
) {
    copy_bytes(
        source,
        owner,
        field.source_offset,
        field.owner_offset,
        field.bytes,
    );
}

fn copy_advanced(
    source: &[u8; SOURCE_CONFIGURATION_BYTES],
    owner: &mut [u8; ENCODER_OWNER_BYTES],
    relative_offset: usize,
    bytes: usize,
) {
    copy_bytes(
        source,
        owner,
        ADVANCED_SOURCE_BASE + relative_offset,
        ADVANCED_OWNER_BASE + relative_offset,
        bytes,
    );
}

fn copy_bytes(
    source: &[u8; SOURCE_CONFIGURATION_BYTES],
    owner: &mut [u8; ENCODER_OWNER_BYTES],
    source_offset: usize,
    owner_offset: usize,
    bytes: usize,
) {
    owner[owner_offset..owner_offset + bytes]
        .copy_from_slice(&source[source_offset..source_offset + bytes]);
}

fn read_i16(bytes: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn owner_i16(owner: &[u8; ENCODER_OWNER_BYTES], offset: usize) -> i16 {
    read_i16(owner, offset)
}

fn owner_u32(owner: &[u8; ENCODER_OWNER_BYTES], offset: usize) -> u32 {
    read_u32(owner, offset)
}

fn owner_i32(owner: &[u8; ENCODER_OWNER_BYTES], offset: usize) -> i32 {
    read_i32(owner, offset)
}

fn write_i16(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_f32(bytes: &mut [u8], offset: usize, value: f32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_bits().to_le_bytes());
}

fn finite(value: f32) -> FiniteF32 {
    FiniteF32::new(value).expect("selected configuration scale operands are finite")
}

fn round_rne_positive_i32(value: f32) -> i32 {
    let bits = value.to_bits();
    let exponent_bits = (bits >> 23) & 0xff;
    let fraction_bits = bits & 0x007f_ffff;
    let (significand, exponent) = if exponent_bits == 0 {
        (u64::from(fraction_bits), -126_i32)
    } else {
        (
            u64::from(fraction_bits | 0x0080_0000),
            i32::try_from(exponent_bits).unwrap() - 127,
        )
    };

    let rounded = if exponent < -1 {
        0_u64
    } else if exponent == -1 {
        u64::from(significand > 0x0080_0000)
    } else if exponent >= 23 {
        significand << usize::try_from(exponent - 23).unwrap()
    } else {
        let shift = u32::try_from(23 - exponent).unwrap();
        let integer = significand >> shift;
        let remainder_mask = (1_u64 << shift) - 1;
        let remainder = significand & remainder_mask;
        let halfway = 1_u64 << (shift - 1);
        if remainder > halfway || (remainder == halfway && integer % 2 == 1) {
            integer + 1
        } else {
            integer
        }
    };
    i32::try_from(rounded).expect("selected rate scale rounds within signed I32 range")
}

fn u32_to_f32_rne(value: u32) -> f32 {
    if value == 0 {
        return 0.0;
    }

    let most_significant_bit = value.ilog2();
    let exponent_bits;
    let significand;
    if most_significant_bit <= 23 {
        exponent_bits = most_significant_bit + 127;
        significand = u64::from(value) << (23 - most_significant_bit);
    } else {
        let shift = most_significant_bit - 23;
        let mut rounded_significand = u64::from(value >> shift);
        let remainder_mask = (1_u32 << shift) - 1;
        let remainder = value & remainder_mask;
        let halfway = 1_u32 << (shift - 1);
        if remainder > halfway || (remainder == halfway && rounded_significand % 2 == 1) {
            rounded_significand += 1;
        }
        if rounded_significand == 1_u64 << 24 {
            exponent_bits = most_significant_bit + 128;
            significand = rounded_significand >> 1;
        } else {
            exponent_bits = most_significant_bit + 127;
            significand = rounded_significand;
        }
    }

    let exponent_bits = exponent_bits << 23;
    let fraction_bits = u32::try_from(significand & 0x007f_ffff).unwrap();
    f32::from_bits(exponent_bits | fraction_bits)
}

fn channel_map_return_value(owner: &[u8; ENCODER_OWNER_BYTES]) -> i32 {
    if owner_u32(owner, 132) != 1 || owner_i16(owner, ADVANCED_OWNER_BASE + 192) == 0 {
        return 0;
    }

    let mut selector_mask = owner_i16(owner, ADVANCED_OWNER_BASE + 192);
    let mut channel_count = 0_i32;
    for index in 0..14 {
        if selector_mask < 0 {
            channel_count += 1;
            if ((32_768_i32 >> index) & 0x0674) != 0 {
                channel_count += 1;
            }
        }
        selector_mask = selector_mask.wrapping_mul(2);
    }

    let format = usize::try_from(owner_i32(owner, 44))
        .expect("format table index is valid by selected import precondition");
    let required = i32::from(CHANNEL_COUNT_A_I16[format]) + i32::from(CHANNEL_COUNT_B_I16[format]);
    if channel_count == required { 0 } else { 161 }
}

#[cfg(test)]
mod tests {
    use super::{
        ADVANCED_OWNER_BASE, BLOCK_SIZE_I16, BLOCK_THRESHOLD_I16, CHANNEL_COUNT_A_I16,
        CHANNEL_COUNT_B_I16, COMPATIBILITY_RATES_I16, ENCODER_OWNER_BYTES, RATE_SCALE_F32_BITS,
        channel_map_return_value, read_i16, write_i16, write_i32,
    };
    use serde_json::Value;
    use sha2::{Digest, Sha256};
    use std::path::PathBuf;

    fn parameters() -> Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/selected_configuration_import/PARAMETERS.json");
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
    }

    fn sha256(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    fn i16_bytes(values: &[i16]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    fn u32_bytes(values: &[u32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    fn check_i16_table(parameters: &Value, name: &str, actual: &[i16]) {
        let expected: Vec<i16> = parameters[name]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| i16::try_from(value.as_i64().unwrap()).unwrap())
            .collect();
        assert_eq!(actual, expected.as_slice(), "{name} table values");
        let identity = parameters["identities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|identity| identity["name"] == name)
            .unwrap();
        let bytes = i16_bytes(actual);
        assert_eq!(identity["bytes"].as_u64().unwrap(), bytes.len() as u64);
        assert_eq!(identity["sha256"].as_str().unwrap(), sha256(&bytes));
    }

    #[test]
    fn static_configuration_tables_match_hash_bound_parameter_assets() {
        let parameters = parameters();
        check_i16_table(&parameters, "block_threshold_i16", &BLOCK_THRESHOLD_I16);
        check_i16_table(&parameters, "block_size_i16", &BLOCK_SIZE_I16);
        check_i16_table(&parameters, "channel_count_a_i16", &CHANNEL_COUNT_A_I16);
        check_i16_table(&parameters, "channel_count_b_i16", &CHANNEL_COUNT_B_I16);
        check_i16_table(
            &parameters,
            "compatibility_rates_i16",
            &COMPATIBILITY_RATES_I16,
        );

        let rate_scale: Vec<u32> = parameters["rate_scale_f32_bits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
            .collect();
        assert_eq!(rate_scale, RATE_SCALE_F32_BITS);
        let identity = parameters["identities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|identity| identity["name"] == "rate_scale_f32_bits")
            .unwrap();
        let bytes = u32_bytes(&RATE_SCALE_F32_BITS);
        assert_eq!(identity["bytes"].as_u64().unwrap(), bytes.len() as u64);
        assert_eq!(identity["sha256"].as_str().unwrap(), sha256(&bytes));
    }

    #[test]
    fn static_derived_channel_map_formula_returns_161_without_runtime_vector_claim() {
        let mut owner = [0_u8; ENCODER_OWNER_BYTES];
        write_i32(&mut owner, 132, 1);
        write_i32(&mut owner, 44, 7);
        write_i16(&mut owner, ADVANCED_OWNER_BASE + 192, i16::MIN);
        assert_eq!(channel_map_return_value(&owner), 161);
        assert_eq!(read_i16(&owner, ADVANCED_OWNER_BASE + 192), i16::MIN);
    }
}
