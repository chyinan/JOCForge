// pattern: Functional Core
//! Selected typed coefficient quantization, Huffman body construction, and ID14 writing.

use std::fmt;

use super::coefficient_quantization::{
    CoefficientPrecision, CoefficientQuantizationError, quantize_selected_coefficient,
};
use super::huffman::{
    F05QConsumerError, F05QConsumerInput, F05QConsumerPointInput, HuffmanBitCount,
    HuffmanCodebooks, HuffmanError, Id14HeaderInput, Id14InputShapeError, Id14ObjectDescriptor,
    Id14Presence, Id14WriterReturnCode, Id14WriterState, write_id14_header,
};

const BAND_COEFFICIENT_COUNTS: [usize; 8] = [1, 3, 5, 7, 9, 12, 15, 23];
const COEFFICIENT_BODY_SCRATCH_LIMIT_BITS: u64 = 4_344;

/// Inputs to the controlled selected ID14 coefficient-payload writer.
#[derive(Clone, Debug, PartialEq)]
pub struct CoefficientPayloadInput {
    pub downmix_index: u8,
    pub extension_config: u8,
    pub clip_gain_index: u8,
    pub sequence_count: u16,
    pub time_slot_limit: u8,
    pub capacity_bytes: u16,
    pub objects: Vec<CoefficientPayloadObject>,
}

/// One descriptor's coefficient source and optional time-slot offsets.
#[derive(Clone, Debug, PartialEq)]
pub struct CoefficientPayloadObject {
    pub present: bool,
    pub band_enum: u8,
    pub mode: u8,
    pub points: Vec<CoefficientPayloadPoint>,
    pub offsets: Option<Vec<u8>>,
}

/// Channel-major coefficients for one point and its sparse channel selection.
#[derive(Clone, Debug, PartialEq)]
pub struct CoefficientPayloadPoint {
    pub channel_rows: Vec<Vec<f32>>,
    pub selected_channels: Vec<u8>,
}

/// The local failure class for selected input preflight and codebook admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoefficientPayloadWriterFailure {
    UnsupportedSelectedProfile,
    InputShape,
    NonFiniteCoefficient,
    CoefficientScaledI32Range,
    OffsetOutsideSelectedWindow,
    OutputCapacityAllocation,
    Huffman(HuffmanError),
    HeaderInputShape(Id14InputShapeError),
}

impl From<F05QConsumerError> for CoefficientPayloadWriterFailure {
    fn from(error: F05QConsumerError) -> Self {
        match error {
            F05QConsumerError::InputShape => Self::InputShape,
            F05QConsumerError::Huffman(error) => Self::Huffman(error),
        }
    }
}

impl From<Id14InputShapeError> for CoefficientPayloadWriterFailure {
    fn from(error: Id14InputShapeError) -> Self {
        Self::HeaderInputShape(error)
    }
}

impl fmt::Display for CoefficientPayloadWriterFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSelectedProfile => {
                formatter.write_str("unsupported selected ID14 coefficient-writer profile")
            }
            Self::InputShape => formatter.write_str("invalid selected coefficient input shape"),
            Self::NonFiniteCoefficient => {
                formatter.write_str("selected coefficient input is non-finite")
            }
            Self::CoefficientScaledI32Range => {
                formatter.write_str("selected coefficient exceeds the scaled I32 domain")
            }
            Self::OffsetOutsideSelectedWindow => {
                formatter.write_str("selected time-slot offset is outside the valid window")
            }
            Self::OutputCapacityAllocation => {
                formatter.write_str("selected output capacity cannot be allocated")
            }
            Self::Huffman(error) => fmt::Display::fmt(error, formatter),
            Self::HeaderInputShape(error) => {
                write!(formatter, "invalid ID14 header input: {error:?}")
            }
        }
    }
}

impl std::error::Error for CoefficientPayloadWriterFailure {}

/// The successful local ID14 descriptor copy owns separate bytes from the writer output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Id14DescriptorCopy {
    body: Vec<u8>,
}

impl Id14DescriptorCopy {
    pub const fn payload_id(&self) -> u8 {
        14
    }

    pub fn length_bytes(&self) -> u64 {
        u64::try_from(self.body.len()).unwrap_or(u64::MAX)
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }
}

/// One local candidate result. Failed writer attempts expose no output bytes or descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoefficientPayloadOutcome {
    header_bit_count: u64,
    body_bit_count: HuffmanBitCount,
    writer_return_code: Option<Id14WriterReturnCode>,
    writer_status_word: Option<u32>,
    writer_valid_bit_count: Option<HuffmanBitCount>,
    padding_zero_bits: Option<u8>,
    output_length_bytes: Option<u64>,
    coefficient_scratch_exceeded: bool,
    payload: Option<Vec<u8>>,
    descriptor_copy: Option<Id14DescriptorCopy>,
}

impl CoefficientPayloadOutcome {
    /// Transfers the successful descriptor copy into a metadata body without
    /// recomputing bytes or length. The caller supplies an already admitted typed
    /// writer clock; this adapter establishes no parent clock conversion or commit.
    pub fn into_metadata_body(
        self,
        writer_generation: crate::timeline::CodecSourceIndex,
    ) -> Option<super::emdf::metadata_owner::MetadataPayloadBody> {
        self.descriptor_copy.map(
            |descriptor| super::emdf::metadata_owner::MetadataPayloadBody {
                writer_generation,
                bytes: descriptor.body,
            },
        )
    }

    pub const fn header_bit_count(&self) -> u64 {
        self.header_bit_count
    }

    pub const fn body_bit_count(&self) -> HuffmanBitCount {
        self.body_bit_count
    }

    pub const fn writer_return_code(&self) -> Option<Id14WriterReturnCode> {
        self.writer_return_code
    }

    pub const fn writer_status_word(&self) -> Option<u32> {
        self.writer_status_word
    }

    pub const fn writer_valid_bit_count(&self) -> Option<HuffmanBitCount> {
        self.writer_valid_bit_count
    }

    pub const fn padding_zero_bits(&self) -> Option<u8> {
        self.padding_zero_bits
    }

    pub const fn output_length_bytes(&self) -> Option<u64> {
        self.output_length_bytes
    }

    pub const fn coefficient_scratch_exceeded(&self) -> bool {
        self.coefficient_scratch_exceeded
    }

    pub fn payload(&self) -> Option<&[u8]> {
        self.payload.as_deref()
    }

    pub const fn descriptor_copy(&self) -> Option<&Id14DescriptorCopy> {
        self.descriptor_copy.as_ref()
    }
}

/// Quantizes selected F32 coefficient inputs, writes their Huffman body and ID14 header,
/// and creates a same-call owned descriptor copy on success.
///
/// The clip byte is an explicit input; its upstream producer is not included here. The
/// operation uses a zero-commit local API policy on writer failure. It does not claim that
/// the native writer rolls back its already-written prefix or that a parent transaction exists.
pub fn write_selected_id14_coefficient_payload(
    input: &CoefficientPayloadInput,
) -> Result<CoefficientPayloadOutcome, CoefficientPayloadWriterFailure> {
    let channel_count = selected_channel_count(input.downmix_index)?;
    validate_input(input, channel_count)?;
    let header_bit_count = count_header_bits(input);
    let codebooks =
        HuffmanCodebooks::selected().map_err(CoefficientPayloadWriterFailure::Huffman)?;
    let (body_bytes, body_bit_count, coefficient_scratch_exceeded) =
        build_coefficient_body(input, channel_count, &codebooks)?;

    if coefficient_scratch_exceeded {
        return Ok(CoefficientPayloadOutcome {
            header_bit_count,
            body_bit_count: HuffmanBitCount::new(body_bit_count),
            writer_return_code: None,
            writer_status_word: None,
            writer_valid_bit_count: None,
            padding_zero_bits: None,
            output_length_bytes: None,
            coefficient_scratch_exceeded: true,
            payload: None,
            descriptor_copy: None,
        });
    }

    let descriptors = input
        .objects
        .iter()
        .map(header_descriptor)
        .collect::<Result<Vec<_>, _>>()?;
    let header_input = Id14HeaderInput {
        downmix_index: u32::from(input.downmix_index),
        object_count: u32::try_from(input.objects.len())
            .map_err(|_| CoefficientPayloadWriterFailure::InputShape)?,
        extension_config: u32::from(input.extension_config),
        clip_gain_index: u32::from(input.clip_gain_index),
        sequence_count: u32::from(input.sequence_count),
        // This existing header API field is the writer's offset-validity threshold, not
        // a channel count. The selected contract uses a 24-time-slot window.
        channel_limit: u32::from(input.time_slot_limit),
        descriptors: Some(descriptors),
        body: body_bytes,
        body_valid_bits: HuffmanBitCount::new(body_bit_count),
    };

    let capacity = usize::from(input.capacity_bytes);
    let mut buffer = Vec::new();
    buffer
        .try_reserve_exact(capacity)
        .map_err(|_| CoefficientPayloadWriterFailure::OutputCapacityAllocation)?;
    buffer.resize(capacity, 0);
    let mut writer = Id14WriterState::new(buffer);
    let writer_outcome = write_id14_header(&header_input, &mut writer)?;
    let writer_return_code = writer_outcome.return_code();
    let mut payload = None;
    let mut descriptor_copy = None;
    if writer_return_code == Id14WriterReturnCode::Success {
        let length = usize::try_from(writer.output_length().value())
            .map_err(|_| CoefficientPayloadWriterFailure::InputShape)?;
        let output = writer.buffer()[..length].to_vec();
        descriptor_copy = Some(Id14DescriptorCopy {
            body: output.clone(),
        });
        payload = Some(output);
    }

    Ok(CoefficientPayloadOutcome {
        header_bit_count,
        body_bit_count: HuffmanBitCount::new(body_bit_count),
        writer_return_code: Some(writer_return_code),
        writer_status_word: Some(writer.status_word()),
        writer_valid_bit_count: Some(writer_outcome.valid_bit_count()),
        padding_zero_bits: Some(writer_outcome.padding_zero_bits()),
        output_length_bytes: Some(writer.output_length().value()),
        coefficient_scratch_exceeded: false,
        payload,
        descriptor_copy,
    })
}

fn selected_channel_count(downmix_index: u8) -> Result<u8, CoefficientPayloadWriterFailure> {
    match downmix_index {
        1 => Ok(7),
        3 => Ok(5),
        _ => Err(CoefficientPayloadWriterFailure::UnsupportedSelectedProfile),
    }
}

fn validate_input(
    input: &CoefficientPayloadInput,
    channel_count: u8,
) -> Result<(), CoefficientPayloadWriterFailure> {
    if input.extension_config != 0
        || input.time_slot_limit != 24
        || input.objects.is_empty()
        || input.objects.len() > 15
        || input.sequence_count > 1023
    {
        return Err(CoefficientPayloadWriterFailure::UnsupportedSelectedProfile);
    }

    for object in input.objects.iter().filter(|object| object.present) {
        let band_count = *BAND_COEFFICIENT_COUNTS
            .get(usize::from(object.band_enum))
            .ok_or(CoefficientPayloadWriterFailure::InputShape)?;
        if object.mode > 3 || !(1..=2).contains(&object.points.len()) {
            return Err(CoefficientPayloadWriterFailure::InputShape);
        }
        if input.downmix_index == 1 && object.mode < 2 {
            return Err(CoefficientPayloadWriterFailure::UnsupportedSelectedProfile);
        }
        if let Some(offsets) = &object.offsets {
            if offsets.len() != object.points.len() {
                return Err(CoefficientPayloadWriterFailure::InputShape);
            }
            if offsets
                .iter()
                .any(|&offset| offset <= 31 && offset >= input.time_slot_limit)
            {
                return Err(CoefficientPayloadWriterFailure::OffsetOutsideSelectedWindow);
            }
        }

        for point in &object.points {
            if point.channel_rows.len() != usize::from(channel_count)
                || point.channel_rows.iter().any(|row| row.len() != band_count)
            {
                return Err(CoefficientPayloadWriterFailure::InputShape);
            }
            if object.mode >= 2
                && (point.selected_channels.len() != band_count
                    || point
                        .selected_channels
                        .iter()
                        .any(|&channel| channel >= channel_count))
            {
                return Err(CoefficientPayloadWriterFailure::InputShape);
            }
        }
    }
    Ok(())
}

fn count_header_bits(input: &CoefficientPayloadInput) -> u64 {
    30 + input
        .objects
        .iter()
        .map(|object| {
            if object.present {
                let offset_count = object.offsets.as_ref().map_or(0, Vec::len);
                8 + 5 * u64::try_from(offset_count).unwrap_or(u64::MAX)
            } else {
                1
            }
        })
        .sum::<u64>()
}

fn header_descriptor(
    object: &CoefficientPayloadObject,
) -> Result<Id14ObjectDescriptor, CoefficientPayloadWriterFailure> {
    if !object.present {
        return Ok(Id14ObjectDescriptor {
            present: Id14Presence::Absent,
            kind_code: 0,
            mode_code: 0,
            nested_mode: 0,
            nested_count: 1,
            selected_index0: 0,
            selected_index1: 0,
        });
    }
    let point_count = u32::try_from(object.points.len())
        .map_err(|_| CoefficientPayloadWriterFailure::InputShape)?;
    let offsets = object.offsets.as_deref().unwrap_or(&[]);
    Ok(Id14ObjectDescriptor {
        present: Id14Presence::Present,
        kind_code: u32::from(object.band_enum),
        mode_code: u32::from(object.mode),
        nested_mode: u32::from(!offsets.is_empty()),
        nested_count: point_count,
        selected_index0: offsets.first().copied().map_or(0, u32::from),
        selected_index1: offsets.get(1).copied().map_or(0, u32::from),
    })
}

fn build_coefficient_body(
    input: &CoefficientPayloadInput,
    channel_count: u8,
    codebooks: &HuffmanCodebooks,
) -> Result<(Vec<u8>, u64, bool), CoefficientPayloadWriterFailure> {
    let mut body = BitAccumulator::default();
    let mut overflow = false;
    for object in input.objects.iter().filter(|object| object.present) {
        let band_count = BAND_COEFFICIENT_COUNTS[usize::from(object.band_enum)];
        let points = object
            .points
            .iter()
            .map(|point| {
                if object.mode < 2 {
                    let quantized_rows = point
                        .channel_rows
                        .iter()
                        .map(|row| {
                            row.iter()
                                .take(band_count)
                                .map(|&coefficient| quantize_coefficient(coefficient, object.mode))
                                .collect::<Result<Vec<_>, _>>()
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(F05QConsumerPointInput::Full { quantized_rows })
                } else {
                    let quantized_coefficients = point
                        .selected_channels
                        .iter()
                        .enumerate()
                        .map(|(band, &channel)| {
                            let coefficient = point.channel_rows[usize::from(channel)][band];
                            quantize_coefficient(coefficient, object.mode)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(F05QConsumerPointInput::Sparse {
                        selected_channels: point
                            .selected_channels
                            .iter()
                            .map(|&channel| i64::from(channel))
                            .collect(),
                        quantized_coefficients,
                    })
                }
            })
            .collect::<Result<Vec<_>, CoefficientPayloadWriterFailure>>()?;
        let encoded = codebooks.encode_f05q_consumer(&F05QConsumerInput {
            mode: object.mode,
            channel_count,
            points,
        })?;
        let total_bits = body
            .total_bits
            .checked_add(encoded.bitstream().valid_bit_count().value())
            .ok_or(CoefficientPayloadWriterFailure::InputShape)?;
        if total_bits > COEFFICIENT_BODY_SCRATCH_LIMIT_BITS {
            body.total_bits = total_bits;
            overflow = true;
        } else if overflow {
            body.total_bits = total_bits;
        } else {
            body.append(
                encoded.bitstream().bytes(),
                encoded.bitstream().valid_bit_count(),
            )?;
        }
    }
    if overflow {
        Ok((Vec::new(), body.total_bits, true))
    } else {
        let total_bits = body.total_bits;
        Ok((body.finish(), total_bits, false))
    }
}

fn quantize_coefficient(value: f32, mode: u8) -> Result<i64, CoefficientPayloadWriterFailure> {
    let precision =
        CoefficientPrecision::from_wire_mode(mode).map_err(map_coefficient_quantization_error)?;
    quantize_selected_coefficient(value, precision)
        .map(i64::from)
        .map_err(map_coefficient_quantization_error)
}

fn map_coefficient_quantization_error(
    error: CoefficientQuantizationError,
) -> CoefficientPayloadWriterFailure {
    match error {
        CoefficientQuantizationError::NonFiniteInput => {
            CoefficientPayloadWriterFailure::NonFiniteCoefficient
        }
        CoefficientQuantizationError::ScaledI32Range => {
            CoefficientPayloadWriterFailure::CoefficientScaledI32Range
        }
        CoefficientQuantizationError::UnsupportedMode
        | CoefficientQuantizationError::NumericNonFinite
        | CoefficientQuantizationError::QuantizedCodeRange => {
            CoefficientPayloadWriterFailure::InputShape
        }
    }
}

#[derive(Default)]
struct BitAccumulator {
    bytes: Vec<u8>,
    total_bits: u64,
}

impl BitAccumulator {
    fn append(
        &mut self,
        bytes: &[u8],
        valid_bits: HuffmanBitCount,
    ) -> Result<(), CoefficientPayloadWriterFailure> {
        let required = usize::try_from(valid_bits.value())
            .map_err(|_| CoefficientPayloadWriterFailure::InputShape)?;
        if required > bytes.len().saturating_mul(8) {
            return Err(CoefficientPayloadWriterFailure::InputShape);
        }
        for source_bit in 0..required {
            let source_mask = 1 << (7 - (source_bit % 8));
            let bit = (bytes[source_bit / 8] & source_mask) != 0;
            if self.total_bits % 8 == 0 {
                self.bytes.push(0);
            }
            if bit {
                let target_byte = usize::try_from(self.total_bits / 8)
                    .map_err(|_| CoefficientPayloadWriterFailure::InputShape)?;
                let target_shift = 7 - (self.total_bits % 8);
                self.bytes[target_byte] |= 1 << target_shift;
            }
            self.total_bits += 1;
        }
        Ok(())
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}
