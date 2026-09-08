// pattern: Functional Core

use crate::{
    AdmPosition, BlockPartition, DependentLfePolicy, DependentPlan, EmdfProfile, JocEncodingPlan,
    JocProfile, OwnershipPlan, partition_frame_bytes, profile_spec, quantize_coefficient,
};

const FRAME_BYTES: usize = 4096;
const D0_FRAME_BYTES: usize = 2560;
const OAMD_ID: u64 = 11;
const JOC_ID: u64 = 14;
// The coarse OAMD fields quantize normalized horizontal coordinates to 32
// bins and non-negative height to the public four-bit range.
const OAMD_COORDINATE_MIN: f64 = -1.0;
const OAMD_COORDINATE_MAX: f64 = 1.0;
const OAMD_COORDINATE_CODE_MAX: f64 = 31.0;
const OAMD_HEIGHT_CODE_MAX: f64 = 15.0;
// These source-derived D0 controls are named so their syntax role and clean
// demonstrator rationale can be audited independently.
const SOURCE_FEATURE_SAMPLE_WINDOW: usize = 256;
const SOURCE_FEATURE_SCALE: f64 = 1_000_000.0;
const SOURCE_FEATURE_MODULUS: i64 = 64;
const D0_INITIAL_EXPONENT: u8 = 15;
const D0_MAX_EXPONENT: u8 = 24;
const D0_EXPONENT_STEP: u8 = 99;
const D0_ZERO_EXPONENT_DELTA: u8 = 62;
const D0_EXPONENT_COUNT: usize = 24;
const D0_MANTISSA_FIRST_BITS: u8 = 8;
const D0_MANTISSA_ZERO_COUNT: usize = 3;
const D0_MANTISSA_COUNT: usize = 35;

/// Writer-level causality evidence for one source-derived D0 input.
#[derive(Clone, Debug, PartialEq)]
pub struct D0CausalityMeasurement {
    pub input_index: usize,
    pub changed_byte_count: usize,
    pub changed: bool,
}

// Public ETSI TS 103 420 coarse generic Huffman tree (Annex A.1, MTX mode).
// A negative child is a leaf encoded as -symbol - 1.
const JOC_COARSE_HUFFMAN: [[i16; 2]; 95] = [
    [-1, 1],
    [2, -2],
    [-96, 3],
    [4, -3],
    [-95, 5],
    [6, 7],
    [-4, -94],
    [8, 9],
    [-5, -93],
    [10, 11],
    [-6, -92],
    [12, 13],
    [-7, -91],
    [14, 15],
    [16, -90],
    [-8, 17],
    [18, -89],
    [-9, 19],
    [20, 21],
    [-88, -10],
    [22, 23],
    [-11, -87],
    [24, 25],
    [26, -86],
    [-12, 27],
    [28, -85],
    [-13, 29],
    [30, 31],
    [32, -84],
    [-14, 33],
    [34, -15],
    [-83, 35],
    [36, 37],
    [-16, 38],
    [-17, -82],
    [39, 40],
    [41, -81],
    [42, 43],
    [44, 45],
    [46, 47],
    [48, 49],
    [50, 51],
    [52, -18],
    [-78, 53],
    [-19, 54],
    [55, 56],
    [57, 58],
    [-22, 59],
    [60, 61],
    [62, 63],
    [64, 65],
    [66, 67],
    [68, -20],
    [-21, -79],
    [-80, -25],
    [69, 70],
    [-26, 71],
    [72, 73],
    [74, 75],
    [76, 77],
    [78, 79],
    [80, 81],
    [82, 83],
    [84, 85],
    [86, 87],
    [88, 89],
    [90, 91],
    [92, 93],
    [94, -23],
    [-74, -75],
    [-72, -73],
    [-76, -77],
    [-34, -35],
    [-32, -33],
    [-38, -39],
    [-36, -37],
    [-30, -31],
    [-28, -29],
    [-50, -51],
    [-48, -49],
    [-54, -55],
    [-52, -53],
    [-42, -43],
    [-40, -41],
    [-46, -47],
    [-44, -45],
    [-66, -67],
    [-64, -65],
    [-70, -71],
    [-68, -69],
    [-58, -59],
    [-56, -57],
    [-62, -63],
    [-60, -61],
    [-24, -27],
];

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
/// JOC payload. Only the explicitly selected profile controls signaling
/// deviations; all other fields remain the current JOCForge configuration.
fn write_emdf(oamd: &[u8], joc: &[u8], emdf_profile: EmdfProfile) -> Result<Vec<u8>, String> {
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
        let codec_data_present = matches!(emdf_profile, EmdfProfile::EtsiStrict);
        container.push(u64::from(codec_data_present), 1);
        if codec_data_present {
            container.push(0, 8); // reserved codec data value
        }
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
    let spec = profile_spec(profile);
    let channel_count = u8::try_from(spec.input_labels.len())
        .map_err(|_| "JOC profile input channel count exceeds its field".to_owned())?;
    let downmix_index = spec.downmix_index;
    let mut bits = BitWriter::default();
    bits.push(u64::from(downmix_index), 3);
    let object_count = u8::try_from(matrix.len())
        .map_err(|_| "JOC coded-object count exceeds the six-bit field".to_owned())?;
    if !(1..=15).contains(&object_count) {
        return Err("JOC coded-object count must be 1..=15".to_owned());
    }
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
            push_coarse_huffman(&mut bits, coefficient.encoded_symbol)?;
        }
    }
    Ok(bits.padded_bytes())
}

fn push_coarse_huffman(bits: &mut BitWriter, symbol: u16) -> Result<(), String> {
    if symbol >= 96 {
        return Err(format!(
            "JOC coarse Huffman symbol {symbol} is outside 0..=95"
        ));
    }
    let mut path = Vec::new();
    if !find_huffman_path(0, symbol, &mut path) {
        return Err(format!(
            "JOC coarse Huffman symbol {symbol} has no public codeword"
        ));
    }
    bits.bits.extend(path);
    Ok(())
}

fn find_huffman_path(node: usize, symbol: u16, path: &mut Vec<bool>) -> bool {
    if node >= JOC_COARSE_HUFFMAN.len() {
        return false;
    }
    for (branch, child) in JOC_COARSE_HUFFMAN[node].iter().copied().enumerate() {
        path.push(branch == 1);
        if child < 0 {
            if u16::try_from(-child - 1).ok() == Some(symbol) {
                return true;
            }
        } else if let Ok(child) = usize::try_from(child)
            && find_huffman_path(child, symbol, path)
        {
            return true;
        }
        path.pop();
    }
    false
}

/// Writes one active dynamic-object OAMD element. The single position is an
/// intentional MVP fixture: remaining object slots are not claimed here.
fn write_oamd_payload(positions: &[AdmPosition]) -> Result<Vec<u8>, String> {
    if positions.is_empty() || positions.len() > 15 {
        return Err("OAMD supports 1..=15 dynamic object positions".to_owned());
    }
    if positions.iter().any(|position| !position.is_finite()) {
        return Err("OAMD positions must be finite".to_owned());
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
    let body = body.padded_bytes();

    let mut payload = BitWriter::default();
    payload.push(0, 2); // syntax version
    payload.push(u64::try_from(positions.len()).unwrap_or(0), 5); // base LFE plus active dynamic objects
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
    let rounded = ((value.clamp(OAMD_COORDINATE_MIN, OAMD_COORDINATE_MAX) + 1.0)
        * OAMD_COORDINATE_CODE_MAX)
        .round() as i64;
    u64::try_from(rounded).unwrap_or(0)
}

fn oamd_height(value: f64) -> u64 {
    let rounded = value.abs().clamp(0.0, OAMD_HEIGHT_CODE_MAX).round() as i64;
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

/// Writes one public 1/2/3-block E-AC-3 frame. The optional exponent view is
/// populated by the normal source-derived path and absent only for the
/// synthetic conformance fixture.
#[allow(clippy::too_many_arguments)]
fn write_public_short_frame(
    stream_type: u8,
    substream_id: u8,
    channel_map: Option<u16>,
    channel_count: usize,
    blocks: BlockPartition,
    convsync: bool,
    emdf: Option<&[u8]>,
    complexity_index: u8,
    frame_bytes: usize,
    source_exponent_codes: Option<&[u8]>,
    lfe_source_carrier: Option<&[f64]>,
    sample_offset: usize,
) -> Result<Vec<u8>, String> {
    validate_frame_bytes(frame_bytes, "short")?;
    if !matches!(
        blocks,
        BlockPartition::One | BlockPartition::Two | BlockPartition::Three
    ) {
        return Err("short conformance frame requires a 1/2/3-block partition".to_owned());
    }
    if !(1..=5).contains(&channel_count) || (stream_type == 1) != channel_map.is_some() {
        return Err("short conformance frame channel topology is invalid".to_owned());
    }
    let block_count = blocks.audio_blocks();
    let lfe_end = sample_offset
        .checked_add(block_count * 256)
        .ok_or_else(|| "short LFE sample window overflows usize".to_owned())?;
    if lfe_source_carrier.is_some_and(|carrier| {
        carrier.len() < lfe_end || carrier.iter().any(|sample| !sample.is_finite())
    }) {
        return Err(
            "short LFE source carrier does not contain the selected finite sample window"
                .to_owned(),
        );
    }
    if lfe_source_carrier.is_some() && (stream_type != 0 || channel_count != 5) {
        return Err("short LFE is only supported on the independent five-channel I0".to_owned());
    }
    let mut bits = BitWriter::default();
    for (value, width) in [
        (0x0b77, 16),
        (u64::from(stream_type), 2),
        (u64::from(substream_id), 3),
        (u64::try_from(frame_bytes / 2 - 1).unwrap_or(0), 11),
        (0, 2), // 48 kHz
        (u64::from(blocks.numblkscod()), 2),
        (
            if channel_count == 1 {
                1
            } else if channel_count == 2 {
                2
            } else {
                7
            },
            3,
        ),
        (u64::from(lfe_source_carrier.is_some()), 1),
        (16, 5),
        (31, 5),
        (0, 1), // no compression metadata
    ] {
        bits.push(value, width);
    }
    if let Some(channel_map) = channel_map {
        bits.push(1, 1);
        bits.push(u64::from(channel_map), 16);
    }
    bits.push(0, 1); // no mixing metadata
    bits.push(0, 1); // no informational metadata
    if stream_type == 0 {
        bits.push(u64::from(convsync), 1);
    }
    bits.push(u64::from(emdf.is_some()), 1);
    if emdf.is_some() {
        bits.push(1, 6);
        bits.push(0x01, 8);
        bits.push(u64::from(complexity_index), 8);
    }
    bits.push(u64::from(lfe_source_carrier.is_some()) * 2, 2);
    bits.push(0, 1); // no transient processing
    bits.push(
        if lfe_source_carrier.is_some() {
            0b0011000
        } else {
            0
        },
        7,
    ); // compact syntax flags; frame fast-gain only for independent LFE
    if channel_count > 1 {
        bits.push(0, 1); // coupling not in use in block zero
        for _ in 1..block_count {
            bits.push(0, 1); // coupling strategy reused/off
        }
    }
    for block in 0..block_count {
        for _ in 0..channel_count {
            bits.push(u64::from(block == 0), 2);
        }
    }
    if lfe_source_carrier.is_some() {
        for block in 0..block_count {
            bits.push(u64::from(block == 0), 1);
        }
    }
    if stream_type == 0 {
        bits.push(0, 1); // converter exponent strategy absent
    }
    if lfe_source_carrier.is_none() {
        bits.push(0, 6); // frame coarse SNR offset
        bits.push(0, 4); // frame fine SNR offset
    }
    if block_count > 1 {
        bits.push(0, 1); // no block-start information
    }
    bits.push(0, 1); // dynamic range absent
    bits.push(0, 1); // SPX not in use
    if channel_count == 2 {
        for _ in 0..4 {
            bits.push(0, 1); // rematrix flags
        }
    }
    for _ in 0..channel_count {
        bits.push(0, 6); // channel bandwidth code
    }
    for channel in 0..channel_count {
        push_source_exponents(&mut bits, D0_INITIAL_EXPONENT);
        if lfe_source_carrier.is_none()
            && let Some(codes) = source_exponent_codes
        {
            let code = codes.get(channel).copied().ok_or_else(|| {
                "source-derived short frame exponent count differs from channel count".to_owned()
            })?;
            let exponent_start = bits.bits.len().saturating_sub(4 + 7 * 24 + 2);
            bits.set(exponent_start, u64::from(code % (D0_MAX_EXPONENT + 1)), 4);
        }
    }
    if lfe_source_carrier.is_some() {
        push_source_lfe_exponents(&mut bits, D0_INITIAL_EXPONENT);
        bits.push(1, 1); // new bit-allocation parameters
        bits.push(0, 11); // bounded parameters shared by full-band and LFE
        bits.push(63, 6); // shared coarse SNR offset
        for _ in 0..channel_count {
            bits.push(15, 4); // full-band channel fine SNR offset
        }
        bits.push(15, 4); // LFE fine SNR offset
        bits.push(1, 1); // new fast-gain codes
        for _ in 0..channel_count {
            bits.push(4, 3); // full-band fast gain
        }
        bits.push(4, 3); // LFE fast gain
    }
    if lfe_source_carrier.is_none() {
        for _ in 0..channel_count {
            bits.push(0, 2); // gain range
        }
    }
    bits.push(0, 1); // converter SNR offset absent
    if let Some(lfe_source_carrier) = lfe_source_carrier {
        for block in 0..block_count {
            if block > 0 {
                bits.push(0, 1); // dynamic range absent
                bits.push(0, 1); // SPX strategy reused
                if channel_count == 2 {
                    bits.push(0, 1); // rematrix strategy reused
                }
                bits.push(0, 1); // bit-allocation parameters reused
                bits.push(0, 1); // SNR offsets reused
                bits.push(0, 1); // fast-gain codes reused
                bits.push(0, 1); // converter SNR offset absent
            }
            push_short_i0_zero_mantissas(&mut bits, channel_count);
            push_short_i0_lfe_mantissas(
                &mut bits,
                source_feature(lfe_source_carrier, sample_offset, block),
            );
        }
    } else {
        for _ in 1..block_count {
            bits.push(0, 1); // dynamic range absent
            bits.push(0, 1); // SPX strategy reused
            if channel_count == 2 {
                bits.push(0, 1); // rematrix strategy reused
            }
            bits.push(0, 1); // converter SNR offset absent
        }
    }
    let reserved_tail = emdf.map_or(17, |payload| {
        payload.len().saturating_mul(8).saturating_add(32)
    });
    if bits.bits.len() > frame_bytes.saturating_mul(8).saturating_sub(reserved_tail) {
        return Err(format!(
            "short frame capacity is insufficient: need {} bits, have {}",
            bits.bits.len().saturating_add(reserved_tail),
            frame_bytes.saturating_mul(8)
        ));
    }
    Ok(finish_frame(bits, emdf, frame_bytes))
}

#[allow(clippy::too_many_arguments)]
fn write_dependent_frame(emdf: Option<&[u8]>, channel_map: u16) -> Vec<u8> {
    write_dependent_frame_with_id(emdf, 0, channel_map)
}

fn write_dependent_frame_with_id(
    emdf: Option<&[u8]>,
    dependent_substream_id: u8,
    channel_map: u16,
) -> Vec<u8> {
    let mut bits = BitWriter::default();
    bits.push(0x0b77, 16);
    bits.push(1, 2); // dependent substream
    bits.push(u64::from(dependent_substream_id), 3); // dependent substream ID
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

/// Writes a small, source-derived dependent carrier. Both full-bandwidth
/// channels carry bounded conventional mantissas derived from each 256-sample
/// source window. This is an auditable PCM-to-bitstream bridge, not a
/// perceptual E-AC-3 encoder.
fn write_source_dependent_frame(
    emdf: &[u8],
    channel_map: u16,
    carriers: &[Vec<f64>],
    sample_offset: usize,
    complexity_index: u8,
    frame_bytes: usize,
) -> Result<Vec<u8>, String> {
    write_source_dependent_frame_with_options(
        Some(emdf),
        0,
        channel_map,
        carriers,
        sample_offset,
        complexity_index,
        frame_bytes,
    )
}

fn write_source_dependent_frame_with_options(
    emdf: Option<&[u8]>,
    dependent_substream_id: u8,
    channel_map: u16,
    carriers: &[Vec<f64>],
    sample_offset: usize,
    complexity_index: u8,
    frame_bytes: usize,
) -> Result<Vec<u8>, String> {
    write_source_dependent_frame_with_lfe_options(
        emdf,
        dependent_substream_id,
        channel_map,
        carriers,
        None,
        sample_offset,
        complexity_index,
        frame_bytes,
    )
}

#[allow(clippy::too_many_arguments)]
fn write_source_dependent_frame_with_lfe_options(
    emdf: Option<&[u8]>,
    dependent_substream_id: u8,
    channel_map: u16,
    carriers: &[Vec<f64>],
    lfe_carrier: Option<&[f64]>,
    sample_offset: usize,
    complexity_index: u8,
    frame_bytes: usize,
) -> Result<Vec<u8>, String> {
    validate_frame_bytes(frame_bytes, "D0")?;
    if dependent_substream_id > 7 {
        return Err("dependent substream id must be in D0..D7".to_owned());
    }
    if !(1..=2).contains(&carriers.len())
        || carriers
            .iter()
            .any(|carrier| carrier.len() < sample_offset.saturating_add(1536))
        || lfe_carrier.is_some_and(|carrier| carrier.len() < sample_offset.saturating_add(1536))
    {
        return Err(
            "source-derived dependent requires one or two 1536-sample carrier windows".to_owned(),
        );
    }
    if carriers
        .iter()
        .any(|carrier| carrier.iter().any(|sample| !sample.is_finite()))
        || lfe_carrier.is_some_and(|carrier| carrier.iter().any(|sample| !sample.is_finite()))
    {
        return Err("source-derived dependent carriers must contain finite PCM".to_owned());
    }
    let mapped_channel_count = crate::conformance::channel_map_channel_count(channel_map);
    let expected_channel_count = carriers.len() + usize::from(lfe_carrier.is_some());
    if mapped_channel_count != expected_channel_count {
        return Err(format!(
            "dependent chanmap carries {mapped_channel_count} channels but source-derived input has {expected_channel_count}"
        ));
    }
    let channel_tokens = crate::conformance::channel_map_tokens(channel_map)?;
    if channel_tokens.contains(&100) || channel_tokens.contains(&101) != lfe_carrier.is_some() {
        return Err("dependent LFE carrier does not match its public channel map".to_owned());
    }
    let mut bits = BitWriter::default();
    for (value, width) in [
        (0x0b77, 16),
        (1, 2), // dependent substream
        (u64::from(dependent_substream_id), 3),
        (u64::try_from(frame_bytes / 2 - 1).unwrap_or(0), 11),
        (0, 2),                                          // 48 kHz
        (3, 2),                                          // six blocks
        (u64::try_from(carriers.len()).unwrap_or(0), 3), // one or two full-band channels
        (u64::from(lfe_carrier.is_some()), 1),
        (16, 5), // E-AC-3
        (31, 5), // dialnorm
        (0, 1),  // no compression metadata
        (1, 1),  // custom channel map present
        (u64::from(channel_map), 16),
        (0, 1), // no mixing metadata
        (0, 1), // no informational metadata
    ] {
        bits.push(value, width);
    }
    bits.push(u64::from(emdf.is_some()), 1); // addbsi only on the EMDF carrier
    if emdf.is_some() {
        bits.push(1, 6); // two addbsi bytes
        bits.push(0x01, 8);
        bits.push(u64::from(complexity_index), 8);
    }
    bits.push(1, 1); // per-block exponent strategies
    bits.push(0, 1); // no AHT
    bits.push(1, 2); // per-block SNR strategy 1
    bits.push(0, 1); // no transient processing
    bits.push(16, 7); // bit allocation syntax enabled
    if carriers.len() == 2 {
        bits.push(0, 1); // coupling off in block zero
        for _ in 1..6 {
            bits.push(0, 1); // coupling strategy reused/off
        }
    }
    for block in 0..6 {
        for _ in 0..carriers.len() {
            bits.push(u64::from(block == 0), 2);
        }
    }
    if lfe_carrier.is_some() {
        for block in 0..6 {
            bits.push(u64::from(block == 0), 1);
        }
    }
    bits.push(0, 1); // no block-start information

    for block in 0..6 {
        if block == 0 {
            bits.push(0, 1); // dynamic range absent
            bits.push(0, 1); // SPX absent
            if carriers.len() == 2 {
                for _ in 0..4 {
                    bits.push(0, 1); // stereo rematrix flags
                }
            }
            for _ in 0..carriers.len() {
                bits.push(0, 6); // channel bandwidth code
            }
            for _ in carriers {
                push_source_exponents(&mut bits, D0_INITIAL_EXPONENT);
            }
            if lfe_carrier.is_some() {
                push_source_lfe_exponents(&mut bits, D0_INITIAL_EXPONENT);
            }
            bits.push(1, 1); // new bit-allocation parameters
            bits.push(0, 11); // conservative allocation parameters
            bits.push(63, 6); // coarse SNR offset
            bits.push(15, 4); // fine SNR offset
        } else {
            bits.push(0, 1); // dynamic range absent/reused
            bits.push(0, 1); // SPX strategy reused
            bits.push(0, 1); // rematrix strategy reused
            bits.push(0, 1); // bit-allocation parameters reused
            bits.push(0, 1); // SNR offsets reused
        }
        for carrier in carriers {
            push_source_mantissas(&mut bits, source_feature(carrier, sample_offset, block));
        }
        if let Some(carrier) = lfe_carrier {
            push_source_lfe_mantissas(&mut bits, source_feature(carrier, sample_offset, block));
        }
    }
    if let Some(emdf) = emdf {
        let minimum_tail = emdf
            .len()
            .checked_mul(8)
            .and_then(|value| value.checked_add(32))
            .ok_or_else(|| "source-derived D0 EMDF tail length overflow".to_owned())?;
        if bits.bits.len() > frame_bytes * 8 - minimum_tail {
            return Err("source-derived D0 audio leaves no room for JOC EMDF".to_owned());
        }
    }
    Ok(finish_frame(bits, emdf, frame_bytes))
}

fn push_source_exponents(bits: &mut BitWriter, initial: u8) {
    bits.push(u64::from(initial.min(D0_MAX_EXPONENT)), 4);
    bits.push(u64::from(D0_EXPONENT_STEP), 7); // bounded positive step
    for _ in 1..D0_EXPONENT_COUNT {
        bits.push(u64::from(D0_ZERO_EXPONENT_DELTA), 7); // hold exponent
    }
    bits.push(0, 2); // gain range
}

fn push_source_lfe_exponents(bits: &mut BitWriter, initial: u8) {
    bits.push(u64::from(initial.min(D0_MAX_EXPONENT)), 4);
    bits.push(87, 7);
    bits.push(87, 7);
}

fn push_source_mantissas(bits: &mut BitWriter, first_code: u8) {
    bits.push(u64::from(first_code), D0_MANTISSA_FIRST_BITS);
    for _ in 0..D0_MANTISSA_ZERO_COUNT {
        bits.push(0, 7);
    }
    for _ in 0..D0_MANTISSA_COUNT {
        bits.push(0, 7);
    }
}

fn push_source_lfe_mantissas(bits: &mut BitWriter, first_code: u8) {
    bits.push(u64::from(first_code), 8);
    bits.push(0, 7);
    bits.push(0, 7);
}

/// Writes the bounded zero full-band mantissas used by the independent short
/// I0 LFE variant. With the fixed side-info profile above, each full-band
/// channel has BAP sequence `[4, 3, 0..]`; the shared grouped-BAP state means
/// every other channel reuses the pending BAP-4 group code.
fn push_short_i0_zero_mantissas(bits: &mut BitWriter, channel_count: usize) {
    for channel in 0..channel_count {
        if channel % 2 == 0 {
            bits.push(60, 7); // BAP-4 group containing two zero codes
        }
        bits.push(3, 3); // BAP-3 zero code
    }
}

/// Writes one source-derived LFE mantissa view for the independent short I0.
/// The LFE BAP-4 group reuses the final full-band group's zero code; three
/// BAP-3 words carry the bounded six-bit source feature and a zero code, then
/// the BAP-1 group carries three zero codes.
fn push_short_i0_lfe_mantissas(bits: &mut BitWriter, feature: u8) {
    bits.push(u64::from(feature % 7), 3);
    bits.push(u64::from((feature / 7) % 7), 3);
    bits.push(3, 3);
    bits.push(13, 5); // BAP-1 group containing three zero codes
}

fn source_feature(carrier: &[f64], sample_offset: usize, block: usize) -> u8 {
    let start = sample_offset + block * SOURCE_FEATURE_SAMPLE_WINDOW;
    let sum = carrier[start..start + SOURCE_FEATURE_SAMPLE_WINDOW]
        .iter()
        .copied()
        .sum::<f64>();
    let value = (sum / SOURCE_FEATURE_SAMPLE_WINDOW as f64 * SOURCE_FEATURE_SCALE).round() as i64;
    value.rem_euclid(SOURCE_FEATURE_MODULUS) as u8
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
    let mut output = bits.fixed_bytes(frame_bytes);
    crate::eac3::rewrite_crc2(&mut output).expect("validated E-AC-3 frame has a CRC2 field");
    output
}

fn validate_frame_bytes(frame_bytes: usize, label: &str) -> Result<(), String> {
    if frame_bytes < 2 || frame_bytes % 2 != 0 || frame_bytes / 2 - 1 > 0x7ff {
        return Err(format!(
            "{label} frame size must be an even public 11-bit frame length"
        ));
    }
    if label == "D0" && frame_bytes < D0_FRAME_BYTES {
        return Err(format!(
            "D0 frame size {frame_bytes} is below the source-derived minimum {D0_FRAME_BYTES}"
        ));
    }
    Ok(())
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
    let emdf = write_emdf(&oamd, &joc, EmdfProfile::default())?;
    let output = match profile {
        JocProfile::Idx0 | JocProfile::Idx3 => write_five_channel_frame(Some(&emdf), true),
        JocProfile::Idx1 => [
            write_five_channel_frame(None, false),
            write_dependent_frame(Some(&emdf), 0x0200),
        ]
        .concat(),
        JocProfile::Idx2 | JocProfile::Idx4 => [
            write_five_channel_frame(None, false),
            write_dependent_frame(Some(&emdf), 0x0010),
        ]
        .concat(),
    };
    Ok(output)
}

/// Packages one externally encoded I0 frame with a JOC D0 carrying the
/// standards-defined OAMD/EMDF. The external carrier bytes remain intact;
/// JOCForge owns the dependent carriage and metadata bytes.
#[allow(clippy::too_many_arguments)]
pub fn package_source_derived_access_unit(
    profile: JocProfile,
    emdf_profile: EmdfProfile,
    sequence_count: u16,
    matrix: &[Vec<f64>],
    positions: &[AdmPosition],
    i0_frame: &[u8],
    d0_carriers: &[Vec<f64>],
    sample_offset: usize,
) -> Result<Vec<u8>, String> {
    package_source_derived_access_unit_with_frame_sizes(
        profile,
        emdf_profile,
        sequence_count,
        matrix,
        positions,
        i0_frame,
        d0_carriers,
        sample_offset,
        D0_FRAME_BYTES,
    )
}

/// Packages one source-derived access unit with an explicitly budgeted D0.
/// The I0 frame is retained from the external carrier encoder and may use any
/// legal even E-AC-3 frame size; the D0 size is checked before metadata is
/// inserted so a low-rate budget fails deterministically instead of truncating.
#[allow(clippy::too_many_arguments)]
pub fn package_source_derived_access_unit_with_frame_sizes(
    profile: JocProfile,
    emdf_profile: EmdfProfile,
    sequence_count: u16,
    matrix: &[Vec<f64>],
    positions: &[AdmPosition],
    i0_frame: &[u8],
    d0_carriers: &[Vec<f64>],
    sample_offset: usize,
    d0_frame_bytes: usize,
) -> Result<Vec<u8>, String> {
    if i0_frame.len() < 2 || i0_frame.len() % 2 != 0 || i0_frame.get(0..2) != Some(&[0x0b, 0x77]) {
        return Err("carrier encoder did not return one legal E-AC-3 I0 frame".to_owned());
    }
    if matrix.len() != positions.len() {
        return Err("JOC matrix rows must match reduced OAMD positions".to_owned());
    }
    let oamd = write_oamd_payload(positions)?;
    let joc = write_joc_payload(profile, sequence_count, matrix)?;
    let emdf = write_emdf(&oamd, &joc, emdf_profile)?;
    let d0 = write_source_dependent_frame(
        &emdf,
        profile_spec(profile).channel_map,
        d0_carriers,
        sample_offset,
        u8::try_from(positions.len() + 1)
            .map_err(|_| "JOC complexity index exceeds its eight-bit field".to_owned())?,
        d0_frame_bytes,
    )?;
    let mut output = Vec::with_capacity(i0_frame.len() + d0.len());
    output.extend_from_slice(i0_frame);
    output.extend_from_slice(&d0);
    Ok(output)
}

/// Packages the validated original-syntax AC-3 I0 plus E-AC-3 D0 JOC form.
/// This advanced path is limited to the five public idx0..idx4 topologies and
/// a 48-kHz, six-block, 640 kbps AC-3 core.
#[allow(clippy::too_many_arguments)]
pub fn package_legacy_ac3_source_derived_access_unit(
    profile: JocProfile,
    emdf_profile: EmdfProfile,
    sequence_count: u16,
    matrix: &[Vec<f64>],
    positions: &[AdmPosition],
    ac3_i0_frame: &[u8],
    d0_carriers: &[Vec<f64>],
    sample_offset: usize,
) -> Result<Vec<u8>, String> {
    if !valid_legacy_ac3_i0_frame(ac3_i0_frame) {
        return Err(
            "legacy AC-3 carriage requires one 48 kHz bsid 6/8 5.1 AC-3 I0 frame".to_owned(),
        );
    }
    if matrix.len() != positions.len() {
        return Err("JOC matrix rows must match reduced OAMD positions".to_owned());
    }
    let oamd = write_oamd_payload(positions)?;
    let joc = write_joc_payload(profile, sequence_count, matrix)?;
    let emdf = write_emdf(&oamd, &joc, emdf_profile)?;
    let d0 = write_source_dependent_frame(
        &emdf,
        profile_spec(profile).channel_map,
        d0_carriers,
        sample_offset,
        u8::try_from(positions.len() + 1)
            .map_err(|_| "JOC complexity index exceeds its eight-bit field".to_owned())?,
        D0_FRAME_BYTES,
    )?;
    let mut output = Vec::with_capacity(ac3_i0_frame.len() + d0.len());
    output.extend_from_slice(ac3_i0_frame);
    output.extend_from_slice(&d0);
    Ok(output)
}

/// Builds one synthetic General JOC interval from public 1/2/3-block
/// syncframe partitions. This is an advanced syntax/conformance path: audio
/// mantissas are bounded zero-energy fields, while OAMD/JOC metadata remains
/// fully serialized and carried on each D0 frame.
#[allow(clippy::too_many_arguments)]
pub fn package_short_block_conformance_access_unit(
    profile: JocProfile,
    emdf_profile: EmdfProfile,
    sequence_count: u16,
    matrix: &[Vec<f64>],
    positions: &[AdmPosition],
    partitions: &[BlockPartition],
) -> Result<Vec<u8>, String> {
    let plan = crate::conformance::resolve_short_block_plan(partitions)
        .map_err(|error| format!("invalid short-block conformance plan: {error}"))?;
    if plan.partitions().contains(&BlockPartition::Six) {
        if plan.partitions().len() != 1 {
            return Err("six-block partition cannot be mixed with short partitions".to_owned());
        }
        if positions.is_empty() {
            return Err("short-block conformance requires at least one position".to_owned());
        }
        let oamd = write_oamd_payload(positions)?;
        let joc = write_joc_payload(profile, sequence_count, matrix)?;
        let emdf = write_emdf(&oamd, &joc, emdf_profile)?;
        return Ok([
            write_five_channel_frame(None, false),
            write_dependent_frame(Some(&emdf), profile_spec(profile).channel_map),
        ]
        .concat());
    }
    if matrix.len() != positions.len() || positions.is_empty() {
        return Err(
            "short-block matrix and OAMD positions must be non-empty and aligned".to_owned(),
        );
    }
    let oamd = write_oamd_payload(positions)?;
    let joc = write_joc_payload(profile, sequence_count, matrix)?;
    let emdf = write_emdf(&oamd, &joc, emdf_profile)?;
    let channel_map = profile_spec(profile).channel_map;
    let complexity_index = u8::try_from(positions.len() + 1)
        .map_err(|_| "JOC complexity index exceeds its eight-bit field".to_owned())?;
    let mut output = Vec::new();
    for (index, partition) in plan.partitions().iter().copied().enumerate() {
        let i0 = write_public_short_frame(
            0,
            0,
            None,
            5,
            partition,
            index == 0,
            None,
            complexity_index,
            512,
            None,
            None,
            0,
        )?;
        let d0 = write_public_short_frame(
            1,
            0,
            Some(channel_map),
            2,
            partition,
            false,
            (index + 1 == plan.partitions().len()).then_some(emdf.as_slice()),
            complexity_index,
            512,
            None,
            None,
            0,
        )?;
        output.extend_from_slice(&i0);
        output.extend_from_slice(&d0);
    }
    Ok(output)
}

/// Packages one source-derived raw JOC interval as genuine short E-AC-3
/// syncframes. Each I0/D0 pair is independently serialized with the selected
/// public block count; only the final D0 owns the interval's JOC/OAMD EMDF.
///
/// This advanced path intentionally excludes exotic dependent topologies until
/// their short-frame timing and ownership contracts are independently closed.
/// The independent programme LFE is carried in every corresponding short I0
/// frame and remains outside the JOC reconstruction matrix.
#[allow(clippy::too_many_arguments)]
pub fn package_source_derived_short_block_access_unit(
    plan: &JocEncodingPlan,
    sequence_count: u16,
    matrix: &[Vec<f64>],
    positions: &[AdmPosition],
    i0_carriers: &[Vec<f64>],
    d0_carriers: &[Vec<f64>],
    i0_lfe_carrier: Option<&[f64]>,
) -> Result<Vec<u8>, String> {
    if plan.block_partition.is_default() {
        return Err(
            "source-derived short writer requires a non-default block partition".to_owned(),
        );
    }
    if crate::conformance::classify_block_partition(plan.block_partition.partitions())
        != crate::BlockPartitionStatus::PublicLegal
    {
        return Err(
            "source-derived short writer requires a publicly closed block partition".to_owned(),
        );
    }
    if matrix.len() != positions.len() || positions.is_empty() {
        return Err(
            "source-derived short matrix and OAMD positions must be non-empty and aligned"
                .to_owned(),
        );
    }
    if i0_carriers.len() != 5 || d0_carriers.len() != 2 {
        return Err(
            "source-derived short writer requires five I0 and two D0 full-band carriers".to_owned(),
        );
    }
    for carrier in i0_carriers.iter().chain(d0_carriers.iter()) {
        if carrier.len() < 1536 || carrier.iter().any(|sample| !sample.is_finite()) {
            return Err(
                "source-derived short carriers must contain 1536 finite samples".to_owned(),
            );
        }
    }
    if i0_lfe_carrier.is_some_and(|carrier| {
        carrier.len() < 1536 || carrier.iter().any(|sample| !sample.is_finite())
    }) {
        return Err("source-derived short LFE carrier must contain 1536 finite samples".to_owned());
    }
    let carrier_sizes = partition_frame_bytes(
        plan.rate.carrier_frame_bytes,
        plan.block_partition.partitions(),
    )?;
    let d0_sizes =
        partition_frame_bytes(plan.rate.d0_frame_bytes, plan.block_partition.partitions())?;
    let oamd = write_oamd_payload(positions)?;
    let joc = write_joc_payload(plan.profile, sequence_count, matrix)?;
    let emdf = write_emdf(&oamd, &joc, plan.emdf_profile)?;
    let complexity_index = u8::try_from(positions.len() + 1)
        .map_err(|_| "JOC complexity index exceeds its eight-bit field".to_owned())?;
    let mut output = Vec::new();
    for (index, range) in plan.block_partition.frame_ranges(0).into_iter().enumerate() {
        let i0_exponent_codes = i0_carriers
            .iter()
            .map(|carrier| source_feature(carrier, range.start_sample, 0))
            .collect::<Vec<_>>();
        let i0 = write_public_short_frame(
            0,
            0,
            None,
            5,
            plan.block_partition.partitions()[index],
            plan.block_partition.convsync()[index],
            None,
            complexity_index,
            carrier_sizes[index],
            Some(&i0_exponent_codes),
            i0_lfe_carrier,
            range.start_sample,
        )?;
        let d0_exponent_codes = d0_carriers
            .iter()
            .map(|carrier| source_feature(carrier, range.start_sample, 0))
            .collect::<Vec<_>>();
        let d0 = write_public_short_frame(
            1,
            0,
            Some(profile_spec(plan.profile).channel_map),
            2,
            plan.block_partition.partitions()[index],
            false,
            (index + 1 == plan.block_partition.partitions().len()).then_some(emdf.as_slice()),
            complexity_index,
            d0_sizes[index],
            Some(&d0_exponent_codes),
            None,
            range.start_sample,
        )?;
        output.extend_from_slice(&i0);
        output.extend_from_slice(&d0);
    }
    crate::conformance::validate_short_block_access_unit(
        &output,
        plan.profile,
        plan.block_partition.partitions(),
    )?;
    Ok(output)
}

fn valid_legacy_ac3_i0_frame(frame: &[u8]) -> bool {
    matches!(frame.len(), 1536 | 1792 | 2048 | 2304 | 2560)
        && frame.get(0..2) == Some(&[0x0b, 0x77])
        && read_bits(frame, 32, 2) == Some(0)
        && matches!(read_bits(frame, 40, 5), Some(6 | 8))
        && read_bits(frame, 48, 3) == Some(7)
}

fn read_bits(bytes: &[u8], offset: usize, width: usize) -> Option<u32> {
    if width > 32 || offset.checked_add(width)? > bytes.len().checked_mul(8)? {
        return None;
    }
    let mut value = 0_u32;
    for bit in 0..width {
        let position = offset + bit;
        value = (value << 1) | u32::from((bytes[position / 8] >> (7 - position % 8)) & 1);
    }
    Some(value)
}

/// Packages a general raw E-AC-3 JOC access unit with sequential D0..D7
/// dependents. Each dependent uses the profile's public replacement channel
/// map; only the last dependent carries the OAMD/JOC EMDF and type-A addbsi.
/// This is an advanced conformance path, not the normal product topology.
#[allow(clippy::too_many_arguments)]
pub fn package_multi_dependent_source_derived_access_unit(
    profile: JocProfile,
    emdf_profile: EmdfProfile,
    sequence_count: u16,
    matrix: &[Vec<f64>],
    positions: &[AdmPosition],
    i0_frame: &[u8],
    dependent_channel_maps: &[u16],
    dependent_carriers: &[Vec<Vec<f64>>],
    sample_offset: usize,
) -> Result<Vec<u8>, String> {
    let plans = dependent_channel_maps
        .iter()
        .enumerate()
        .map(|(index, channel_map)| {
            DependentPlan::new(
                u8::try_from(index).unwrap_or(7),
                *channel_map,
                DependentLfePolicy::Absent,
            )
        })
        .collect::<Vec<_>>();
    let plan = OwnershipPlan {
        profile,
        independent_lfe: false,
        dependents: plans,
        final_joc_owner: u8::try_from(dependent_carriers.len().saturating_sub(1)).unwrap_or(7),
    };
    let lfe_carriers = vec![None; dependent_carriers.len()];
    package_multi_dependent_source_derived_access_unit_with_plan(
        profile,
        emdf_profile,
        sequence_count,
        matrix,
        positions,
        i0_frame,
        &plan,
        dependent_carriers,
        &lfe_carriers,
        sample_offset,
    )
}

/// Packages a raw multi-dependent AU from an explicit ownership plan.
#[allow(clippy::too_many_arguments)]
pub fn package_multi_dependent_source_derived_access_unit_with_plan(
    profile: JocProfile,
    emdf_profile: EmdfProfile,
    sequence_count: u16,
    matrix: &[Vec<f64>],
    positions: &[AdmPosition],
    i0_frame: &[u8],
    ownership_plan: &OwnershipPlan,
    dependent_carriers: &[Vec<Vec<f64>>],
    dependent_lfe_carriers: &[Option<Vec<f64>>],
    sample_offset: usize,
) -> Result<Vec<u8>, String> {
    if i0_frame.len() < 2 || i0_frame.len() % 2 != 0 || i0_frame.get(0..2) != Some(&[0x0b, 0x77]) {
        return Err("carrier encoder did not return one legal E-AC-3 I0 frame".to_owned());
    }
    if ownership_plan.profile != profile {
        return Err("ownership plan profile does not match the selected profile".to_owned());
    }
    if dependent_carriers.len() != ownership_plan.dependents.len()
        || dependent_lfe_carriers.len() != ownership_plan.dependents.len()
    {
        return Err("dependent source carriers must match ownership plan count".to_owned());
    }
    let independent_lfe = read_bits(i0_frame, 39, 1)
        .ok_or_else(|| "carrier encoder returned an incomplete I0 LFE field".to_owned())?
        != 0;
    if independent_lfe != ownership_plan.independent_lfe {
        return Err("I0 LFE ownership does not match the ownership plan".to_owned());
    }
    crate::conformance::validate_source_dependent_ownership_plan(
        ownership_plan,
        &dependent_carriers.iter().map(Vec::len).collect::<Vec<_>>(),
        &dependent_lfe_carriers
            .iter()
            .map(Option::is_some)
            .collect::<Vec<_>>(),
    )?;
    if matrix.len() != positions.len() {
        return Err("JOC matrix rows must match reduced OAMD positions".to_owned());
    }
    let oamd = write_oamd_payload(positions)?;
    let joc = write_joc_payload(profile, sequence_count, matrix)?;
    let emdf = write_emdf(&oamd, &joc, emdf_profile)?;
    let complexity_index = u8::try_from(positions.len() + 1)
        .map_err(|_| "JOC complexity index exceeds its eight-bit field".to_owned())?;
    let mut output = Vec::with_capacity(i0_frame.len() + dependent_carriers.len() * D0_FRAME_BYTES);
    output.extend_from_slice(i0_frame);
    for (index, carriers) in dependent_carriers.iter().enumerate() {
        let substream_id = ownership_plan.dependents[index].id;
        let dependent = if index + 1 == dependent_carriers.len() {
            write_source_dependent_frame_with_lfe_options(
                Some(emdf.as_slice()),
                substream_id,
                ownership_plan.dependents[index].channel_map,
                carriers,
                dependent_lfe_carriers[index].as_deref(),
                sample_offset,
                complexity_index,
                D0_FRAME_BYTES,
            )?
        } else {
            write_source_dependent_frame_with_lfe_options(
                None,
                substream_id,
                ownership_plan.dependents[index].channel_map,
                carriers,
                dependent_lfe_carriers[index].as_deref(),
                sample_offset,
                complexity_index,
                D0_FRAME_BYTES,
            )?
        };
        output.extend_from_slice(&dependent);
    }
    Ok(output)
}

/// Packages one public I0-only skip-field AU. The external FFmpeg
/// frame remains the audio core; JOCForge owns the public EMDF and inserts it
/// into the I0 audio-block skip field without retaining a dependent substream.
#[allow(clippy::too_many_arguments)]
pub fn package_i0_skip_field_access_unit(
    profile: JocProfile,
    emdf_profile: EmdfProfile,
    sequence_count: u16,
    matrix: &[Vec<f64>],
    positions: &[AdmPosition],
    i0_frame: &[u8],
) -> Result<Vec<u8>, String> {
    if profile != JocProfile::Idx0 {
        return Err("I0 skip-field carriage currently supports idx0 only".to_owned());
    }
    if i0_frame.get(0..2) != Some(&[0x0b, 0x77])
        || !(i0_frame.len() == FRAME_BYTES || i0_frame.len() == 3584)
    {
        return Err("I0 skip-field carriage requires one public E-AC-3 I0 frame".to_owned());
    }
    if matrix.len() != positions.len() {
        return Err("JOC matrix rows must match reduced OAMD positions".to_owned());
    }
    let oamd = write_oamd_payload(positions)?;
    let joc = write_joc_payload(profile, sequence_count, matrix)?;
    let emdf = write_emdf(&oamd, &joc, emdf_profile)?;
    let i0_frame = if i0_frame.len() == FRAME_BYTES {
        i0_frame.to_vec()
    } else {
        crate::eac3::inflate_eac3_frame(i0_frame, FRAME_BYTES)?
    };
    crate::eac3::inject_joc_skipfld(
        &i0_frame,
        &emdf,
        u8::try_from(positions.len() + 1)
            .map_err(|_| "JOC complexity index exceeds its eight-bit field".to_owned())?,
    )
}

/// Verifies that each D0 source input changes the emitted dependent frame.
/// The fixture is deterministic and exercises the actual source-dependent D0
/// writer rather than only checking an in-memory carrier flag.
pub fn d0_carrier_causality(profile: JocProfile) -> Result<Vec<D0CausalityMeasurement>, String> {
    let first = (0..1536)
        .map(|sample| (f64::from(sample) * 0.011).sin() * 0.1)
        .collect::<Vec<_>>();
    let second = (0..1536)
        .map(|sample| (f64::from(sample) * 0.017 + 0.4).cos() * 0.1)
        .collect::<Vec<_>>();
    let emdf = vec![0x5a; 16];
    let channel_map = profile_spec(profile).channel_map;
    let baseline = write_source_dependent_frame(
        &emdf,
        channel_map,
        &[first.clone(), second.clone()],
        0,
        1,
        D0_FRAME_BYTES,
    )?;
    let mut measurements = Vec::with_capacity(2);
    for input_index in 0..2 {
        let source = if input_index == 0 {
            first.clone()
        } else {
            second.clone()
        };
        let mut perturbed = source;
        perturbed[17] += 0.017;
        let mut carriers = vec![first.clone(), second.clone()];
        carriers[input_index] = perturbed;
        let changed =
            write_source_dependent_frame(&emdf, channel_map, &carriers, 0, 1, D0_FRAME_BYTES)?;
        let changed_byte_count = baseline
            .iter()
            .zip(&changed)
            .filter(|(left, right)| left != right)
            .count();
        measurements.push(D0CausalityMeasurement {
            input_index,
            changed_byte_count,
            changed: changed_byte_count > 0,
        });
    }
    Ok(measurements)
}

/// Backward-compatible proof wrapper. Production source-derived encoding uses
/// [`package_source_derived_access_unit`]; this legacy entry point retains the
/// previous structural-D0 behavior for callers of the initial proof API.
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
    let emdf = write_emdf(&oamd, &joc, EmdfProfile::default())?;
    let d0 = write_dependent_frame(Some(&emdf), profile_spec(profile).channel_map);
    let mut output = Vec::with_capacity(FRAME_BYTES + d0.len());
    output.extend_from_slice(i0_frame);
    output.extend_from_slice(&d0);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{
        AdmPosition, D0_FRAME_BYTES, source_feature, write_emdf, write_five_channel_frame,
        write_oamd_payload, write_source_dependent_frame,
        write_source_dependent_frame_with_lfe_options,
    };
    use crate::{
        DependentLfePolicy, DependentPlan, EmdfProfile, JocProfile, OwnershipPlan,
        package_multi_dependent_source_derived_access_unit_with_plan,
        validate_multi_dependent_access_unit,
    };

    struct TestBitReader<'a> {
        bytes: &'a [u8],
        position: usize,
    }

    impl<'a> TestBitReader<'a> {
        fn new(bytes: &'a [u8], position: usize) -> Self {
            Self { bytes, position }
        }

        fn read(&mut self, width: usize) -> u64 {
            let mut value = 0;
            for _ in 0..width {
                let byte = self.bytes[self.position / 8];
                let shift = 7 - self.position % 8;
                value = (value << 1) | u64::from((byte >> shift) & 1);
                self.position += 1;
            }
            value
        }
    }

    fn codec_data_flags(emdf: &[u8]) -> Vec<bool> {
        let mut reader = TestBitReader::new(emdf, 32);
        reader.read(2);
        reader.read(3);
        let mut flags = Vec::new();
        loop {
            let payload_id = reader.read(5);
            if payload_id == 0 {
                break;
            }
            assert_eq!(reader.read(1), 0); // sample offset absent
            assert_eq!(reader.read(1), 0); // duration absent
            assert_eq!(reader.read(1), 1); // group ID present
            assert_eq!(reader.read(2), 1); // shared group ID
            assert_eq!(reader.read(1), 0); // group ID terminates
            let codec_data_present = reader.read(1) != 0;
            if codec_data_present {
                reader.read(8);
            }
            assert_eq!(reader.read(1), 0); // discard unknown payload false
            assert_eq!(reader.read(1), 1); // payload frame aligned
            reader.read(1); // create duplicate
            reader.read(1); // remove duplicate
            reader.read(5); // priority
            reader.read(2); // processing allowed
            let payload_bytes = reader.read(8);
            assert_eq!(reader.read(1), 0); // payload length terminates
            reader.read(usize::try_from(payload_bytes).unwrap_or(0) * 8);
            flags.push(codec_data_present);
        }
        flags
    }

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

    #[test]
    fn emdf_profile_controls_codec_data_bits_without_stale_codec_bytes() {
        let strict = write_emdf(&[0xaa], &[0xbb], EmdfProfile::EtsiStrict).expect("strict EMDF");
        let deployed =
            write_emdf(&[0xaa], &[0xbb], EmdfProfile::DeployedCompat).expect("deployed EMDF");

        assert_eq!(codec_data_flags(&strict), [true, true]);
        assert_eq!(codec_data_flags(&deployed), [false, false]);
    }

    #[test]
    fn source_derived_dependent_frame_changes_with_both_d0_inputs() {
        let first = (0..1536)
            .map(|sample| f64::from(u8::try_from(sample % 17).unwrap_or(0)) * 0.00001)
            .collect::<Vec<_>>();
        let second = (0..1536)
            .map(|sample| f64::from(u8::try_from(sample % 23).unwrap_or(0)) * 0.00001)
            .collect::<Vec<_>>();
        let mut changed_first = first.clone();
        changed_first[0] += 0.001;
        let mut changed_second = second.clone();
        changed_second[0] -= 0.001;
        let baseline = write_source_dependent_frame(
            &[0],
            0x0200,
            &[first.clone(), second.clone()],
            0,
            16,
            D0_FRAME_BYTES,
        )
        .expect("source-derived D0");
        let changed = write_source_dependent_frame(
            &[0],
            0x0200,
            &[changed_first, second.clone()],
            0,
            16,
            D0_FRAME_BYTES,
        )
        .expect("changed source-derived D0");
        let changed_second_frame = write_source_dependent_frame(
            &[0],
            0x0200,
            &[first, changed_second],
            0,
            16,
            D0_FRAME_BYTES,
        )
        .expect("second changed source-derived D0");

        assert_eq!(baseline.len(), D0_FRAME_BYTES);
        assert_ne!(baseline, changed);
        assert_ne!(baseline, changed_second_frame);
    }

    #[test]
    fn source_features_fit_the_bounded_conventional_mantissa_range() {
        let carrier = vec![0.801; 1536];

        assert!(source_feature(&carrier, 0, 0) < 64);
    }

    #[test]
    fn source_derived_dependent_frame_rejects_an_unvalidated_lower_rate_frame_size() {
        let carrier = vec![vec![0.01; 1536], vec![0.02; 1536]];

        let error = write_source_dependent_frame(&[0; 8], 0x0200, &carrier, 0, 16, 1536)
            .expect_err("lower-rate source-derived D0 is not validated");

        assert!(error.contains("below the source-derived minimum"));
    }

    #[test]
    fn source_derived_dependent_lfe_is_causally_encoded_and_structurally_owned() {
        let full_band = vec![vec![0.01; 1536]];
        let mut lfe = vec![0.0; 1536];
        lfe[0] = 0.75;
        let baseline = write_source_dependent_frame_with_lfe_options(
            None,
            0,
            0x4001,
            &full_band,
            Some(&lfe),
            0,
            16,
            D0_FRAME_BYTES,
        )
        .expect("source-derived dependent LFE");

        lfe[0] = -0.75;
        let changed = write_source_dependent_frame_with_lfe_options(
            None,
            0,
            0x4001,
            &full_band,
            Some(&lfe),
            0,
            16,
            D0_FRAME_BYTES,
        )
        .expect("changed source-derived dependent LFE");
        assert_ne!(baseline, changed);

        let plans = [
            DependentPlan::new(0, 0x4001, DependentLfePolicy::Supplement),
            DependentPlan::new(1, 0x8000, DependentLfePolicy::Absent),
            DependentPlan::new(2, 0x2000, DependentLfePolicy::Absent),
            DependentPlan::new(3, 0x1000, DependentLfePolicy::Absent),
            DependentPlan::new(4, 0x0800, DependentLfePolicy::Absent),
        ];
        let carriers = plans
            .iter()
            .map(|plan| {
                let count = plan.full_band_channel_count();
                (0..count).map(|_| vec![0.01; 1536]).collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let lfe_carriers = vec![Some(lfe), None, None, None, None];
        let matrix = vec![vec![0.1; 5]];
        let positions = [AdmPosition {
            azimuth: 0.0,
            elevation: 0.0,
            distance: 0.0,
        }];
        let bytes = package_multi_dependent_source_derived_access_unit_with_plan(
            JocProfile::Idx0,
            EmdfProfile::EtsiStrict,
            1,
            &matrix,
            &positions,
            &write_five_channel_frame(None, false),
            &OwnershipPlan {
                profile: JocProfile::Idx0,
                independent_lfe: false,
                dependents: plans.to_vec(),
                final_joc_owner: 4,
            },
            &carriers,
            &lfe_carriers,
            0,
        )
        .expect("package source-derived dependent LFE");
        let report = validate_multi_dependent_access_unit(&bytes, JocProfile::Idx0)
            .expect("validate source-derived dependent LFE");
        assert!(report.lfe_present);
        assert_eq!(report.joc_owner, Some(4));
    }
}
