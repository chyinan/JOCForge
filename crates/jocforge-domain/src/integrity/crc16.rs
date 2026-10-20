// pattern: Functional Core
//! E-AC-3 frame CRC16 generation and residual verification.

/// CRC16 with polynomial 0x8005, init zero, MSB-first and xorout zero.
pub fn eac3_crc16(bytes: &[u8]) -> u16 {
    let mut crc = 0_u16;
    for byte in bytes {
        crc ^= u16::from(*byte) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x8005
            } else {
                crc << 1
            };
        }
    }
    crc
}
