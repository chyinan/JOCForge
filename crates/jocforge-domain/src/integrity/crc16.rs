// pattern: Functional Core
//! E-AC-3 frame CRC16 generation and residual verification.

// Each row is the contribution of one byte followed by 0..7 zero bytes.
const TABLE: [[u16; 256]; 8] = {
    let mut table = [[0; 256]; 8];
    let mut byte = 0;
    while byte < 256 {
        let mut crc = (byte as u16) << 8;
        let mut suffix = 0;
        while suffix < 8 {
            let mut bit = 0;
            while bit < 8 {
                crc = if crc & 0x8000 != 0 {
                    (crc << 1) ^ 0x8005
                } else {
                    crc << 1
                };
                bit += 1;
            }
            table[suffix][byte] = crc;
            suffix += 1;
        }
        byte += 1;
    }
    table
};

/// CRC16 with polynomial 0x8005, init zero, MSB-first and xorout zero.
pub fn eac3_crc16(bytes: &[u8]) -> u16 {
    let mut crc = 0_u16;
    let mut chunks = bytes.chunks_exact(8);
    for block in &mut chunks {
        let [hi, lo] = crc.to_be_bytes();
        crc = TABLE[7][usize::from(block[0] ^ hi)]
            ^ TABLE[6][usize::from(block[1] ^ lo)]
            ^ TABLE[5][usize::from(block[2])]
            ^ TABLE[4][usize::from(block[3])]
            ^ TABLE[3][usize::from(block[4])]
            ^ TABLE[2][usize::from(block[5])]
            ^ TABLE[1][usize::from(block[6])]
            ^ TABLE[0][usize::from(block[7])];
    }
    for &byte in chunks.remainder() {
        crc = (crc << 8) ^ TABLE[0][usize::from(crc.to_be_bytes()[0] ^ byte)];
    }
    crc
}

#[cfg(test)]
mod tests {
    #[test]
    fn all_prefixes_match_bitwise_recurrence() {
        let data: Vec<_> = (0..4097u32)
            .map(|i| i.wrapping_mul(2654435761).to_be_bytes()[0])
            .collect();
        let mut reference = 0u16;
        for (i, byte) in data.iter().enumerate() {
            reference ^= u16::from(*byte) << 8;
            for _ in 0..8 {
                reference = if reference & 0x8000 != 0 {
                    (reference << 1) ^ 0x8005
                } else {
                    reference << 1
                };
            }
            assert_eq!(super::eac3_crc16(&data[..=i]), reference);
        }
    }
}
