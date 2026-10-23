// pattern: Functional Core
//! The selected ID14 header/body bit writer. Parent retry and section-header meanings remain external.

/// Values observed for the one-bit `present` field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Id14Presence {
    Absent,
    Present,
    ObservedNonBooleanTwo,
}

impl Id14Presence {
    const fn raw_value(self) -> u32 {
        match self {
            Self::Absent => 0,
            Self::Present => 1,
            Self::ObservedNonBooleanTwo => 2,
        }
    }

    const fn is_present(self) -> bool {
        !matches!(self, Self::Absent)
    }

    const fn raw_bit(self) -> u32 {
        self.raw_value() & 1
    }
}

/// Semantic view of one descriptor record; the native 28-byte ABI layout is not reproduced.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Id14ObjectDescriptor {
    pub present: Id14Presence,
    pub kind_code: u32,
    pub mode_code: u32,
    pub nested_mode: u32,
    pub nested_count: u32,
    pub selected_index0: u32,
    pub selected_index1: u32,
}

/// Header controls, optional descriptor list, and a body bitstream to append without alignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Id14HeaderInput {
    pub downmix_index: u32,
    pub object_count: u32,
    pub extension_config: u32,
    pub clip_gain_index: u32,
    pub sequence_count: u32,
    pub channel_limit: u32,
    pub descriptors: Option<Vec<Id14ObjectDescriptor>>,
    pub body: Vec<u8>,
    pub body_valid_bits: super::HuffmanBitCount,
}

/// The writer's separate native return-code class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Id14WriterReturnCode {
    Success,
    InvalidInput,
    CapacityExceeded,
    FieldOutOfRange,
}

impl Id14WriterReturnCode {
    pub const fn value(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::InvalidInput => 1,
            Self::CapacityExceeded => 2,
            Self::FieldOutOfRange => 3,
        }
    }
}

/// A Rust input-shape guard not represented as a native writer return code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Id14InputShapeError {
    DescriptorCountMismatch,
    BodyValidBitsExceedBuffer,
}

/// Output byte length remains the original capacity on any writer failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Id14OutputLength(u64);

impl Id14OutputLength {
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Writer storage and the separate status word returned by the selected native path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Id14WriterState {
    buffer: Vec<u8>,
    output_length: Id14OutputLength,
    status_word: u32,
}

impl Id14WriterState {
    /// Creates a writer whose initial output length is the supplied buffer capacity.
    pub fn new(buffer: Vec<u8>) -> Self {
        let output_length = u64::try_from(buffer.len()).unwrap_or(u64::MAX);
        Self {
            buffer,
            output_length: Id14OutputLength(output_length),
            status_word: 1,
        }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub const fn output_length(&self) -> Id14OutputLength {
        self.output_length
    }

    pub const fn status_word(&self) -> u32 {
        self.status_word
    }
}

/// Return code, bit cursor, and final padding from one writer attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Id14WriterOutcome {
    return_code: Id14WriterReturnCode,
    valid_bit_count: super::HuffmanBitCount,
    padding_zero_bits: u8,
}

impl Id14WriterOutcome {
    pub const fn return_code(self) -> Id14WriterReturnCode {
        self.return_code
    }

    pub const fn valid_bit_count(self) -> super::HuffmanBitCount {
        self.valid_bit_count
    }

    pub const fn padding_zero_bits(self) -> u8 {
        self.padding_zero_bits
    }
}

/// Writes the fixed header, descriptors, body, optional trailer, and final zero padding.
///
/// On writer failures, prior bits may remain in the attempt buffer while output length stays
/// unchanged. The caller owns any retry or parent-state transaction.
pub fn write_id14_header(
    input: &Id14HeaderInput,
    writer: &mut Id14WriterState,
) -> Result<Id14WriterOutcome, Id14InputShapeError> {
    let body_capacity_bits = u64::try_from(input.body.len())
        .unwrap_or(u64::MAX)
        .saturating_mul(8);
    if input.body_valid_bits.value() > body_capacity_bits {
        return Err(Id14InputShapeError::BodyValidBitsExceedBuffer);
    }
    if (1..=64).contains(&input.object_count)
        && input.descriptors.as_ref().is_some_and(|descriptors| {
            u64::try_from(descriptors.len()).unwrap_or(u64::MAX) != u64::from(input.object_count)
        })
    {
        return Err(Id14InputShapeError::DescriptorCountMismatch);
    }

    writer.status_word = 1;
    let capacity_bits = u64::try_from(writer.buffer.len())
        .unwrap_or(u64::MAX)
        .saturating_mul(8);
    let mut bit_cursor = 0_u64;

    macro_rules! fail {
        ($code:expr) => {
            return Ok(Id14WriterOutcome {
                return_code: $code,
                valid_bit_count: super::HuffmanBitCount::new(bit_cursor),
                padding_zero_bits: 0,
            })
        };
    }

    if input.downmix_index > 7 {
        fail!(Id14WriterReturnCode::FieldOutOfRange);
    }
    if input.downmix_index > 4 {
        writer.status_word = 0;
    }
    if !write_field(
        writer,
        &mut bit_cursor,
        input.downmix_index,
        3,
        capacity_bits,
    ) {
        fail!(Id14WriterReturnCode::CapacityExceeded);
    }

    if !(1..=64).contains(&input.object_count) {
        fail!(Id14WriterReturnCode::FieldOutOfRange);
    }
    if input.object_count > 15 {
        writer.status_word = 0;
    }
    if !write_field(
        writer,
        &mut bit_cursor,
        input.object_count - 1,
        6,
        capacity_bits,
    ) {
        fail!(Id14WriterReturnCode::CapacityExceeded);
    }

    if input.extension_config > 7 {
        fail!(Id14WriterReturnCode::FieldOutOfRange);
    }
    if input.extension_config != 0 {
        writer.status_word = 0;
    }
    if !write_field(
        writer,
        &mut bit_cursor,
        input.extension_config,
        3,
        capacity_bits,
    ) {
        fail!(Id14WriterReturnCode::CapacityExceeded);
    }

    let Some(descriptors) = input.descriptors.as_ref() else {
        fail!(Id14WriterReturnCode::InvalidInput);
    };

    if input.clip_gain_index > 255 {
        fail!(Id14WriterReturnCode::FieldOutOfRange);
    }
    if !write_field(
        writer,
        &mut bit_cursor,
        input.clip_gain_index,
        8,
        capacity_bits,
    ) {
        fail!(Id14WriterReturnCode::CapacityExceeded);
    }

    if input.sequence_count > 1023 {
        fail!(Id14WriterReturnCode::FieldOutOfRange);
    }
    if !write_field(
        writer,
        &mut bit_cursor,
        input.sequence_count,
        10,
        capacity_bits,
    ) {
        fail!(Id14WriterReturnCode::CapacityExceeded);
    }

    for descriptor in descriptors {
        if !write_field(
            writer,
            &mut bit_cursor,
            descriptor.present.raw_bit(),
            1,
            capacity_bits,
        ) {
            fail!(Id14WriterReturnCode::CapacityExceeded);
        }
        if !descriptor.present.is_present() {
            continue;
        }

        if descriptor.kind_code > 7 {
            fail!(Id14WriterReturnCode::FieldOutOfRange);
        }
        if !write_field(
            writer,
            &mut bit_cursor,
            descriptor.kind_code,
            3,
            capacity_bits,
        ) {
            fail!(Id14WriterReturnCode::CapacityExceeded);
        }

        if descriptor.mode_code > 3 {
            fail!(Id14WriterReturnCode::FieldOutOfRange);
        }
        if !write_field(
            writer,
            &mut bit_cursor,
            descriptor.mode_code,
            2,
            capacity_bits,
        ) {
            fail!(Id14WriterReturnCode::CapacityExceeded);
        }

        if descriptor.nested_mode > 1 {
            fail!(Id14WriterReturnCode::FieldOutOfRange);
        }
        if !write_field(
            writer,
            &mut bit_cursor,
            descriptor.nested_mode,
            1,
            capacity_bits,
        ) {
            fail!(Id14WriterReturnCode::CapacityExceeded);
        }

        if !(1..=2).contains(&descriptor.nested_count) {
            fail!(Id14WriterReturnCode::FieldOutOfRange);
        }
        if !write_field(
            writer,
            &mut bit_cursor,
            descriptor.nested_count - 1,
            1,
            capacity_bits,
        ) {
            fail!(Id14WriterReturnCode::CapacityExceeded);
        }

        if descriptor.nested_mode == 1 {
            if descriptor.selected_index0 > 31 {
                fail!(Id14WriterReturnCode::FieldOutOfRange);
            }
            if descriptor.selected_index0 >= input.channel_limit {
                writer.status_word = 0;
            }
            if !write_field(
                writer,
                &mut bit_cursor,
                descriptor.selected_index0,
                5,
                capacity_bits,
            ) {
                fail!(Id14WriterReturnCode::CapacityExceeded);
            }
            if descriptor.nested_count == 2 {
                if descriptor.selected_index1 > 31 {
                    fail!(Id14WriterReturnCode::FieldOutOfRange);
                }
                if descriptor.selected_index1 >= input.channel_limit
                    || descriptor.selected_index0 >= descriptor.selected_index1
                {
                    writer.status_word = 0;
                }
                if !write_field(
                    writer,
                    &mut bit_cursor,
                    descriptor.selected_index1,
                    5,
                    capacity_bits,
                ) {
                    fail!(Id14WriterReturnCode::CapacityExceeded);
                }
            }
        }
    }

    if !write_body_bits(
        writer,
        &mut bit_cursor,
        &input.body,
        input.body_valid_bits.value(),
        capacity_bits,
    ) {
        fail!(Id14WriterReturnCode::CapacityExceeded);
    }
    if input.extension_config != 0
        && !write_field(writer, &mut bit_cursor, 0xd0e1, 16, capacity_bits)
    {
        fail!(Id14WriterReturnCode::CapacityExceeded);
    }

    let padding_zero_bits = ((8 - (bit_cursor % 8)) % 8) as u8;
    if !write_field(writer, &mut bit_cursor, 0, padding_zero_bits, capacity_bits) {
        fail!(Id14WriterReturnCode::CapacityExceeded);
    }

    writer.output_length = Id14OutputLength(bit_cursor / 8);
    Ok(Id14WriterOutcome {
        return_code: Id14WriterReturnCode::Success,
        valid_bit_count: super::HuffmanBitCount::new(bit_cursor - u64::from(padding_zero_bits)),
        padding_zero_bits,
    })
}

fn write_field(
    writer: &mut Id14WriterState,
    bit_cursor: &mut u64,
    value: u32,
    bit_length: u8,
    capacity_bits: u64,
) -> bool {
    if bit_cursor
        .checked_add(u64::from(bit_length))
        .is_none_or(|end| end > capacity_bits)
    {
        return false;
    }
    for shift in (0..bit_length).rev() {
        write_bit(writer, bit_cursor, ((value >> shift) & 1) as u8);
    }
    true
}

fn write_body_bits(
    writer: &mut Id14WriterState,
    bit_cursor: &mut u64,
    body: &[u8],
    valid_bits: u64,
    capacity_bits: u64,
) -> bool {
    for source_bit in 0..valid_bits {
        if *bit_cursor >= capacity_bits {
            return false;
        }
        let source_byte = usize::try_from(source_bit / 8).unwrap_or(usize::MAX);
        if source_byte >= body.len() {
            return false;
        }
        let source_index = u8::try_from(source_bit % 8).unwrap_or(0);
        let value = (body[source_byte] >> (7 - source_index)) & 1;
        write_bit(writer, bit_cursor, value);
    }
    true
}

fn write_bit(writer: &mut Id14WriterState, bit_cursor: &mut u64, value: u8) {
    let byte_index = usize::try_from(*bit_cursor / 8).expect("capacity was prevalidated");
    let bit_index = u8::try_from(7 - (*bit_cursor % 8)).expect("bit offset is in 0..8");
    if bit_index == 7 {
        writer.buffer[byte_index] = 0;
    }
    let mask = 1_u8 << bit_index;
    if value == 0 {
        writer.buffer[byte_index] &= !mask;
    } else {
        writer.buffer[byte_index] |= mask;
    }
    *bit_cursor += 1;
}
