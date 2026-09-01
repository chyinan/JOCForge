// pattern: Functional Core

use crate::{AdmPosition, JocProfile, profile_spec, quantize_coefficient};

const FRAME_BYTES: usize = 4096;
const D0_FRAME_BYTES: usize = 2560;
const OAMD_ID: u64 = 11;
const JOC_ID: u64 = 14;
const JOC_ZERO_CODEWORD: &str = "0";
const JOC_UNIT_CODEWORD: &str = "11";

#[derive(Default)]
struct BitWriter {
    bits: Vec<bool>,
}

impl BitWriter {
    fn push(&mut self, value: u64, width: u8) {
        for shift in (0..width).rev() {
            self.bits.push(value & (1_u64 << shift) != 0);
        }
    }

    fn set(&mut self, position: usize, value: u64, width: u8) {
        for index in 0..usize::from(width) {
            let shift = usize::from(width) - index - 1;
            self.bits[position + index] = (value >> shift) & 1 != 0;
        }
    }

    fn padded_bytes(mut self) -> Vec<u8> {
        while self.bits.len() % 8 != 0 {
            self.bits.push(false);
        }
        pack_bits(&self.bits)
    }

    fn fixed_bytes(mut self, bytes: usize) -> Vec<u8> {
        self.bits.resize(bytes * 8, false);
        pack_bits(&self.bits)
    }
}

fn pack_bits(bits: &[bool]) -> Vec<u8> {
    let mut bytes = vec![0_u8; bits.len().div_ceil(8)];
    for (index, bit) in bits.iter().copied().enumerate() {
        if bit {
            bytes[index / 8] |= 0x80 >> (index % 8);
        }
    }
    bytes
}

/// Produces a bounded, deterministic JOC EMDF container with one OAMD and one
/// JOC payload. The field choices follow TS 103 420 table 56's strict profile.
fn write_emdf(oamd: &[u8], joc: &[u8]) -> Result<Vec<u8>, String> {
    if oamd.len() > u8::MAX.into() || joc.len() > u8::MAX.into() {
        return Err("JOC EMDF payload exceeds the MVP byte bound".to_owned());
    }
    let mut container = BitWriter::default();
    container.push(0, 2); // EMDF syntax version
    container.push(0, 3); // key identifier
    for (payload_id, payload) in [(OAMD_ID, oamd), (JOC_ID, joc)] {
        container.push(payload_id, 5);
        container.push(0, 1); // sample offset absent
        container.push(0, 1); // duration absent
        container.push(1, 1); // group ID present
        container.push(1, 2); // shared group ID
        container.push(0, 1); // group ID terminates
        container.push(1, 1); // codec data present
        container.push(0, 8); // reserved codec data value
        container.push(0, 1); // discard unknown payload false
        container.push(1, 1); // payload frame aligned
        container.push(0, 1); // create duplicate false
        container.push(0, 1); // remove duplicate false
        container.push(0, 5); // priority
        container.push(0, 2); // processing allowed
        container.push(u64::try_from(payload.len()).unwrap_or(u64::MAX), 8);
        container.push(0, 1); // payload length terminates
        for byte in payload {
            container.push(u64::from(*byte), 8);
        }
    }
    container.push(0, 5); // end payload list
    container.push(1, 2); // one-byte primary protection
    container.push(0, 2); // no secondary protection
    container.push(0, 8); // primary protection byte
    let body = container.padded_bytes();
    let body_len = u16::try_from(body.len()).map_err(|_| "EMDF container too large".to_owned())?;
    let mut output = vec![0x58, 0x38];
    output.extend_from_slice(&body_len.to_be_bytes());
    output.extend_from_slice(&body);
    Ok(output)
}

/// Writes a full-matrix JOC payload using the two public coarse coefficient
/// codewords needed by the MVP proof (zero and one quantizer step).
fn write_joc_payload(
    profile: JocProfile,
    sequence_count: u16,
    matrix: &[Vec<f64>],
) -> Result<Vec<u8>, String> {
    if sequence_count > 1023 {
        return Err("JOC sequence count exceeds its ten-bit field".to_owned());
    }
    let channel_count = match profile {
        JocProfile::Idx0 => 5_u8,
        JocProfile::Idx1 | JocProfile::Idx2 => 7_u8,
    };
    let downmix_index = match profile {
        JocProfile::Idx0 => 0_u8,
        JocProfile::Idx1 => 1_u8,
        JocProfile::Idx2 => 2_u8,
    };
    let mut bits = BitWriter::default();
    bits.push(u64::from(downmix_index), 3);
    let object_count = 15_u8;
    if matrix.len() != usize::from(object_count)
        || matrix
            .iter()
            .any(|row| row.len() != usize::from(channel_count))
    {
        return Err("JOC matrix dimensions do not match the selected profile".to_owned());
    }
    bits.push(u64::from(object_count - 1), 6); // object count minus one
    bits.push(0, 3); // no extension
    bits.push(2, 3); // clip gain X
    bits.push(17, 5); // clip gain Y
    bits.push(u64::from(sequence_count), 10);
    for _ in 0..object_count {
        bits.push(1, 1); // object present
        bits.push(0, 3); // one parameter band
        bits.push(0, 1); // full matrix
        bits.push(0, 1); // coarse 96-step quantizer
        bits.push(0, 1); // smooth interpolation
        bits.push(0, 1); // one data point
    }
    for row in matrix {
        for ideal in row {
            let coefficient = quantize_coefficient(*ideal)
                .map_err(|error| format!("failed to quantize JOC coefficient: {error}"))?;
            let codeword = match coefficient.encoded_symbol {
                0 => JOC_ZERO_CODEWORD,
                1 => JOC_UNIT_CODEWORD,
                symbol => {
                    return Err(format!(
                        "JOC MVP writer only supports public coarse symbols 0 and 1, got {symbol}"
                    ));
                }
            };
            for bit in codeword.bytes() {
                bits.bits.push(bit == b'1');
            }
        }
    }
    Ok(bits.padded_bytes())
}

/// Writes one active dynamic-object OAMD element. The single position is an
/// intentional MVP fixture: remaining object slots are not claimed here.
fn write_oamd_payload(positions: &[AdmPosition]) -> Result<Vec<u8>, String> {
    if positions.is_empty() || positions.len() > 15 {
        return Err("MVP OAMD supports 1..=15 dynamic object positions".to_owned());
    }
    let mut body = BitWriter::default();
    body.push(0, 1); // discard unknown
    body.push(0, 2); // sample offset
    body.push(0, 3); // one metadata block
    body.push(0, 6); // block offset factor
    body.push(0, 2); // zero ramp
    body.push(1, 1); // no reserved object-element data
    body.push(1, 1); // inactive base-LFE object slot
    body.push(0, 1); // no additional table data
    for position in positions {
        body.push(0, 1); // active dynamic object
        body.push(0, 2); // 0 dB gain
        body.push(1, 1); // default priority
        body.push(oamd_coordinate(position.azimuth), 6); // X
        body.push(oamd_coordinate(position.elevation), 6); // Y
        body.push(u64::from(position.distance >= 0.0), 1); // positive Z sign
        body.push(oamd_height(position.distance), 4); // Z magnitude
        body.push(0, 1); // inside-room distance
        body.push(0, 3); // horizontal zones
        body.push(1, 1); // elevation zone
        body.push(0, 2); // point object
        body.push(0, 1); // room anchored
        body.push(0, 1); // channel lock disabled
        body.push(0, 1); // no additional table data
    }
    for _ in positions.len()..15 {
        body.push(1, 1); // inactive object slot
        body.push(0, 1); // no additional table data
    }
    let body = body.padded_bytes();

    let mut payload = BitWriter::default();
    payload.push(0, 2); // syntax version
    payload.push(15, 5); // 16 total objects: base LFE plus 15 dynamic
    payload.push(1, 1); // dynamic-only programme
    payload.push(1, 1); // base LFE object present
    payload.push(0, 1); // no alternate object data
    payload.push(1, 4); // one object element
    payload.push(1, 4); // object element identifier
    push_variable_bits_max(
        &mut payload,
        u64::try_from(body.len().saturating_sub(1)).unwrap_or(0),
    )?;
    for byte in body {
        payload.push(u64::from(byte), 8);
    }
    Ok(payload.padded_bytes())
}

fn push_variable_bits_max(bits: &mut BitWriter, value: u64) -> Result<(), String> {
    let mut groups = vec![value & 0x0f];
    let mut remainder = value;
    while remainder > 0x0f {
        remainder = (remainder - 16) / 16;
        groups.push(remainder & 0x0f);
        if groups.len() > 4 {
            return Err("OAMD element body exceeds variable-size bound".to_owned());
        }
    }
    groups.reverse();
    let group_count = groups.len();
    for (index, group) in groups.into_iter().enumerate() {
        bits.push(group, 4);
        bits.push(u64::from(index + 1 < group_count), 1);
    }
    Ok(())
}

fn oamd_coordinate(value: f64) -> u64 {
    let rounded = ((value.clamp(-1.0, 1.0) + 1.0) * 31.0).round() as i64;
    u64::try_from(rounded).unwrap_or(0)
}

fn oamd_height(value: f64) -> u64 {
    let rounded = value.abs().clamp(0.0, 15.0).round() as i64;
    u64::try_from(rounded).unwrap_or(0)
}

fn write_five_channel_frame(emdf: Option<&[u8]>, joc_extension: bool) -> Vec<u8> {
    let exponent_delta_codes = [62_u8; 5];
    let mut bits = BitWriter::default();
    for (value, width) in [
        (0x0b77, 16),
        (0, 2),
        (0, 3),
        (2047, 11),
        (0, 2),
        (3, 2),
        (7, 3),
        (0, 1),
        (16, 5),
        (31, 5),
        (0, 1),
        (0, 1),
        (0, 1),
        (u64::from(joc_extension), 1),
    ] {
        bits.push(value, width);
    }
    if joc_extension {
        bits.push(1, 6); // two addbsi bytes
        bits.push(0x01, 8); // JOC extension type
        bits.push(16, 8); // fifteen dynamic slots plus base-LFE slot
    }
    bits.push(1, 1);
    bits.push(0, 1);
    bits.push(0, 2);
    bits.push(0, 1);
    bits.push(0, 7);
    bits.push(0, 1);
    for _ in 1..6 {
        bits.push(0, 1);
    }
    for block in 0..6 {
        for _ in 0..5 {
            bits.push(u64::from(block == 0), 2);
        }
    }
    for _ in 0..5 {
        bits.push(0, 5);
    }
    bits.push(0, 6);
    bits.push(0, 4);
    bits.push(0, 1);
    bits.push(0, 1);
    bits.push(0, 1);
    for _ in 0..5 {
        bits.push(0, 6);
    }
    for exponent_delta_code in exponent_delta_codes {
        bits.push(0, 4); // channel initial exponent
        for _ in 0..24 {
            bits.push(u64::from(exponent_delta_code), 7);
        }
        bits.push(0, 2);
    }
    bits.push(0, 1);
    for _ in 1..6 {
        bits.push(0, 1);
        bits.push(0, 1);
        bits.push(0, 1);
    }
    finish_frame(bits, emdf, FRAME_BYTES)
}

fn write_dependent_frame(emdf: Option<&[u8]>, channel_map: u16) -> Vec<u8> {
    let mut bits = BitWriter::default();
    bits.push(0x0b77, 16);
    bits.push(1, 2); // dependent substream
    bits.push(0, 3); // dependent substream ID
    bits.push(u64::try_from(D0_FRAME_BYTES / 2 - 1).unwrap_or(0), 11);
    bits.push(0, 2); // 48 kHz
    bits.push(3, 2); // six blocks
    bits.push(2, 3); // two full-band channels
    bits.push(0, 1); // no LFE
    bits.push(16, 5); // E-AC-3
    bits.push(31, 5); // dialnorm
    bits.push(0, 1); // no compression metadata
    bits.push(1, 1); // custom channel map present
    bits.push(u64::from(channel_map), 16);
    bits.push(0, 1); // no mixing metadata
    bits.push(0, 1); // no informational metadata
    bits.push(u64::from(emdf.is_some()), 1); // addbsi present
    if emdf.is_some() {
        bits.push(1, 6); // two addbsi bytes
        bits.push(0x01, 8); // JOC extension type
        bits.push(16, 8); // 16-slot complexity index
    }
    bits.push(1, 1); // per-block exponent strategies
    bits.push(0, 1); // no AHT
    bits.push(0, 2); // frame SNR offset
    bits.push(0, 1); // no transient processing
    bits.push(0, 7); // no optional syntax flags
    bits.push(0, 1); // coupling off in block zero
    for _ in 1..6 {
        bits.push(0, 1); // coupling strategy reused
    }
    for block in 0..6 {
        for _ in 0..2 {
            bits.push(u64::from(block == 0), 2); // D15 then reuse
        }
    }
    bits.push(0, 6); // frame coarse SNR offset
    bits.push(0, 4); // frame fine SNR offset
    bits.push(0, 1); // block-start information absent
    bits.push(0, 1); // dynamic range absent
    bits.push(0, 1); // SPX absent
    for _ in 0..4 {
        bits.push(0, 1); // stereo rematrix disabled
    }
    for _ in 0..2 {
        bits.push(0, 6); // channel bandwidth code
    }
    for exponent_delta_code in [82_u8, 86] {
        bits.push(15, 4); // channel initial exponent
        for _ in 0..24 {
            bits.push(u64::from(exponent_delta_code), 7);
        }
        bits.push(0, 2); // gain range
    }
    bits.push(0, 1); // no new bit-allocation parameters
    for _ in 1..6 {
        bits.push(0, 1); // dynamic range absent
        bits.push(0, 1); // SPX strategy reused
        bits.push(0, 1); // rematrix strategy reused
        bits.push(0, 1); // bit-allocation parameters reused
    }
    finish_frame(bits, emdf, D0_FRAME_BYTES)
}

fn finish_frame(mut bits: BitWriter, emdf: Option<&[u8]>, frame_bytes: usize) -> Vec<u8> {
    bits.bits.resize(frame_bytes * 8, false);
    if let Some(emdf) = emdf {
        let auxdatae_position = frame_bytes * 8 - 18;
        let length_position = auxdatae_position - 14;
        bits.set(
            length_position,
            u64::try_from(emdf.len() * 8).unwrap_or(u64::MAX),
            14,
        );
        bits.set(auxdatae_position, 1, 1);
        let start = length_position - emdf.len() * 8;
        for (index, byte) in emdf.iter().copied().enumerate() {
            bits.set(start + index * 8, u64::from(byte), 8);
        }
    }
    bits.fixed_bytes(frame_bytes)
}

/// Creates one new raw E-AC-3/JOC access unit from a transparent, finite test
/// carrier. The bytes are generated by JOCForge's own writer and are not
/// copied from any media or from a proprietary encoder.
pub fn write_one_au_proof(profile: JocProfile) -> Result<Vec<u8>, String> {
    let position = AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    };
    let positions = vec![position; 15];
    let oamd = write_oamd_payload(&positions)?;
    let spec = profile_spec(profile);
    let unit = 820.0 / 4096.0;
    let matrix = (0..15)
        .map(|object| {
            (0..spec.input_labels.len())
                .map(|channel| {
                    if channel == object % spec.input_labels.len() {
                        unit
                    } else {
                        0.0
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let joc = write_joc_payload(profile, 1, &matrix)?;
    let emdf = write_emdf(&oamd, &joc)?;
    let output = match profile {
        JocProfile::Idx0 => write_five_channel_frame(Some(&emdf), true),
        JocProfile::Idx1 => [
            write_five_channel_frame(None, false),
            write_dependent_frame(Some(&emdf), 0x0200),
        ]
        .concat(),
        JocProfile::Idx2 => [
            write_five_channel_frame(None, false),
            write_dependent_frame(Some(&emdf), 0x0010),
        ]
        .concat(),
    };
    Ok(output)
}

/// Packages one externally encoded 5-channel I0 frame with a JOC D0 carrying
/// the standards-defined OAMD/EMDF. The external carrier bytes remain intact;
/// JOCForge owns the dependent carriage and metadata bytes.
pub fn package_ffmpeg_access_unit(
    profile: JocProfile,
    sequence_count: u16,
    matrix: &[Vec<f64>],
    positions: &[AdmPosition],
    i0_frame: &[u8],
) -> Result<Vec<u8>, String> {
    if i0_frame.len() != FRAME_BYTES || i0_frame.get(0..2) != Some(&[0x0b, 0x77]) {
        return Err("carrier encoder did not return one 4096-byte E-AC-3 I0 frame".to_owned());
    }
    let oamd = write_oamd_payload(positions)?;
    let joc = write_joc_payload(profile, sequence_count, matrix)?;
    let emdf = write_emdf(&oamd, &joc)?;
    let d0 = write_dependent_frame(Some(&emdf), profile_spec(profile).channel_map);
    let mut output = Vec::with_capacity(FRAME_BYTES + D0_FRAME_BYTES);
    output.extend_from_slice(i0_frame);
    output.extend_from_slice(&d0);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{AdmPosition, write_oamd_payload};

    #[test]
    fn seven_channel_profiles_declare_the_sixteen_slot_oamd_domain() {
        let positions = vec![
            AdmPosition {
                azimuth: 0.0,
                elevation: 0.0,
                distance: 0.0,
            };
            15
        ];
        let payload = write_oamd_payload(&positions).expect("sixteen-slot OAMD");

        assert_eq!(payload[0], 0x1f);
    }
}
