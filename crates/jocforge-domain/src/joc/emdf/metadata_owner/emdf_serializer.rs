use super::MetadataOwnerError;
use super::payload_queue::QueuedBody;
use serde_json::{Value, json};
#[cfg(test)]
use sha2::{Digest, Sha256};

const EMDF_HEADER_BITS: usize = 32;

pub(super) struct EmdfOutput {
    pub bytes: Vec<u8>,
    pub terminal_layout: Value,
}

pub(super) fn serialize(
    wrapper_configuration: &Value,
    payloads: &[(u8, &QueuedBody)],
) -> Result<EmdfOutput, MetadataOwnerError> {
    let mut body_writer = BitWriter::default();
    body_writer.write(0, 5)?;

    for (payload_id, body) in payloads {
        body_writer.write(u64::from(*payload_id), 5)?;
        let payload_configuration = wrapper_configuration["payloads"]
            .get(payload_id.to_string())
            .ok_or(MetadataOwnerError::EmdfSerialization)?;
        write_configuration(&mut body_writer, payload_configuration)?;
        write_unsigned_variable(&mut body_writer, body.payload.len() as u64, 8, 2)?;
        body_writer.write_bytes(&body.payload);
    }

    let terminal_start = body_writer.len() + EMDF_HEADER_BITS;
    body_writer.write(0, 5)?;
    let primary_start = body_writer.len() + EMDF_HEADER_BITS;
    body_writer.write(2, 2)?;
    let secondary_start = body_writer.len() + EMDF_HEADER_BITS;
    body_writer.write(1, 2)?;
    let primary_protection_start = body_writer.len() + EMDF_HEADER_BITS;
    body_writer.write(0, 32)?;
    let secondary_protection_start = body_writer.len() + EMDF_HEADER_BITS;
    body_writer.write(0, 8)?;
    let padding_start = body_writer.len() + EMDF_HEADER_BITS;
    body_writer.pad_to_byte();
    let body_bytes = body_writer.into_bytes();
    let wire_bytes = body_bytes.len() + 4;
    if wire_bytes > 511 {
        return Err(MetadataOwnerError::EmdfSerialization);
    }

    let terminal_padding_bits = body_bytes.len() * 8 + EMDF_HEADER_BITS - padding_start;
    let mut bytes = Vec::with_capacity(wire_bytes);
    bytes.extend_from_slice(&[0x58, 0x38]);
    bytes.extend_from_slice(&(body_bytes.len() as u16).to_be_bytes());
    bytes.extend_from_slice(&body_bytes);

    let terminal_layout = json!({
        "bit_order": "MSB-first",
        "terminal_id": {
            "value": 0,
            "start_bit": terminal_start,
            "width_bits": 5,
        },
        "primary_length_code": {
            "value": 2,
            "start_bit": primary_start,
            "width_bits": 2,
            "byte_count": 4,
        },
        "secondary_length_code": {
            "value": 1,
            "start_bit": secondary_start,
            "width_bits": 2,
            "byte_count": 1,
        },
        "primary_protection_range_bits": [primary_protection_start, secondary_protection_start],
        "secondary_protection_range_bits": [secondary_protection_start, secondary_protection_start + 8],
        "terminal_padding_range_bits": [padding_start, padding_start + terminal_padding_bits],
        "terminal_padding_bits": terminal_padding_bits,
        "wire_bytes": wire_bytes,
        "body_bytes": body_bytes.len(),
    });

    Ok(EmdfOutput {
        bytes,
        terminal_layout,
    })
}

fn write_configuration(
    writer: &mut BitWriter,
    configuration: &Value,
) -> Result<(), MetadataOwnerError> {
    let sample_offset = optional_integer(configuration, "sample_offset")?;
    writer.write(u64::from(sample_offset.is_some()), 1)?;
    if let Some(offset) = sample_offset {
        writer.write_checked(offset, 11)?;
        writer.write(0, 1)?;
    }

    let duration = optional_integer(configuration, "duration")?;
    writer.write(u64::from(duration.is_some()), 1)?;
    if let Some(value) = duration {
        write_unsigned_variable(writer, value, 11, 2)?;
    }

    let group_id = optional_integer(configuration, "group_id")?;
    writer.write(u64::from(group_id.is_some()), 1)?;
    if let Some(value) = group_id {
        write_unsigned_variable(writer, value, 2, 8)?;
    }

    let codec_data_present = required_boolean(configuration, "codec_data_present")?;
    writer.write(u64::from(codec_data_present), 1)?;
    let discard_unknown_payload = required_boolean(configuration, "discard_unknown_payload")?;
    writer.write(u64::from(discard_unknown_payload), 1)?;

    let mut aligned = false;
    if !discard_unknown_payload && sample_offset.is_none() {
        aligned = required_boolean(configuration, "payload_frame_aligned")?;
        writer.write(u64::from(aligned), 1)?;
        if aligned {
            let create_duplicate = required_boolean(configuration, "create_duplicate")?;
            let remove_duplicate = required_boolean(configuration, "remove_duplicate")?;
            writer.write(u64::from(create_duplicate), 1)?;
            writer.write(u64::from(remove_duplicate), 1)?;
        }
    }

    if !discard_unknown_payload && (sample_offset.is_some() || aligned) {
        writer.write_checked(required_integer(configuration, "priority")?, 5)?;
        writer.write_checked(required_integer(configuration, "processing_allowed")?, 2)?;
    }

    Ok(())
}

fn optional_integer(configuration: &Value, field: &str) -> Result<Option<u64>, MetadataOwnerError> {
    match configuration.get(field) {
        Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_u64()
            .map(Some)
            .ok_or(MetadataOwnerError::EmdfSerialization),
        None => Err(MetadataOwnerError::EmdfSerialization),
    }
}

fn required_integer(configuration: &Value, field: &str) -> Result<u64, MetadataOwnerError> {
    configuration
        .get(field)
        .and_then(Value::as_u64)
        .ok_or(MetadataOwnerError::EmdfSerialization)
}

fn required_boolean(configuration: &Value, field: &str) -> Result<bool, MetadataOwnerError> {
    configuration
        .get(field)
        .and_then(Value::as_bool)
        .ok_or(MetadataOwnerError::EmdfSerialization)
}

fn write_unsigned_variable(
    writer: &mut BitWriter,
    mut value: u64,
    group_width: usize,
    maximum_groups: usize,
) -> Result<(), MetadataOwnerError> {
    let mask = (1_u64 << group_width) - 1;
    let mut groups = Vec::new();
    loop {
        groups.push(value & mask);
        if value < (1_u64 << group_width) {
            break;
        }
        value = value / (1_u64 << group_width) - 1;
        if groups.len() >= maximum_groups {
            return Err(MetadataOwnerError::EmdfSerialization);
        }
    }
    for (index, group) in groups.iter().rev().enumerate() {
        writer.write(*group, group_width)?;
        writer.write(u64::from(index + 1 < groups.len()), 1)?;
    }
    Ok(())
}

#[derive(Default)]
struct BitWriter {
    bits: Vec<bool>,
}

impl BitWriter {
    fn len(&self) -> usize {
        self.bits.len()
    }

    fn write(&mut self, value: u64, width: usize) -> Result<(), MetadataOwnerError> {
        if width < 64 && value >= (1_u64 << width) {
            return Err(MetadataOwnerError::EmdfSerialization);
        }
        for bit in (0..width).rev() {
            self.bits.push(((value >> bit) & 1) != 0);
        }
        Ok(())
    }

    fn write_checked(&mut self, value: u64, width: usize) -> Result<(), MetadataOwnerError> {
        self.write(value, width)
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            for bit in (0..8).rev() {
                self.bits.push(((*byte >> bit) & 1) != 0);
            }
        }
    }

    fn pad_to_byte(&mut self) {
        while self.bits.len() % 8 != 0 {
            self.bits.push(false);
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        self.bits
            .chunks_exact(8)
            .map(|chunk| {
                chunk
                    .iter()
                    .fold(0_u8, |byte, bit| (byte << 1) | u8::from(*bit))
            })
            .collect()
    }
}

#[cfg(test)]
pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    super::hex_encode(&Sha256::digest(bytes))
}
