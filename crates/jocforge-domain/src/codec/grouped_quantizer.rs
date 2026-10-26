// pattern: Functional Core
//! Block-local F12 mantissa quantization, grouping, and MSB-first packing.

use super::mantissa::{MantissaQuantizerError, quantize_mantissa};

/// Channel order used to flatten one block before mantissa grouping.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MantissaChannel {
    Left,
    Center,
    Right,
    LeftSurround,
    RightSurround,
    Lfe,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MantissaBlockItemInput {
    pub channel: MantissaChannel,
    pub bin: u16,
    pub coefficient_word: u32,
    pub decoded_exponent: i16,
    pub bap: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MantissaItemObservation {
    channel: MantissaChannel,
    bin: u16,
    bap: u8,
    code: u16,
    width: u8,
}

impl MantissaItemObservation {
    pub const fn channel(&self) -> MantissaChannel {
        self.channel
    }

    pub const fn bin(&self) -> u16 {
        self.bin
    }

    pub const fn bap(&self) -> u8 {
        self.bap
    }

    pub const fn code(&self) -> u16 {
        self.code
    }

    pub const fn width(&self) -> u8 {
        self.width
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GroupedPackObservation {
    bap: u8,
    value: u16,
    width: u8,
    start_bit: u64,
}

impl GroupedPackObservation {
    pub const fn bap(&self) -> u8 {
        self.bap
    }

    pub const fn value(&self) -> u16 {
        self.value
    }

    pub const fn width(&self) -> u8 {
        self.width
    }

    /// Absolute bit offset of this group field within the block bitstream.
    pub const fn start_bit(&self) -> u64 {
        self.start_bit
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MantissaBlockOutput {
    items: Vec<MantissaItemObservation>,
    grouped_packs: Vec<GroupedPackObservation>,
    packed_bytes: Vec<u8>,
    bit_count: u64,
}

impl MantissaBlockOutput {
    pub fn items(&self) -> &[MantissaItemObservation] {
        &self.items
    }

    pub fn grouped_packs(&self) -> &[GroupedPackObservation] {
        &self.grouped_packs
    }

    pub fn packed_bytes(&self) -> &[u8] {
        &self.packed_bytes
    }

    pub const fn bit_count(&self) -> u64 {
        self.bit_count
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MantissaBlockError {
    Quantizer(MantissaQuantizerError),
    DuplicateItemPosition { channel: MantissaChannel, bin: u16 },
    BitCountOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PackedField {
    value: u16,
    width: u8,
    grouped_bap: Option<u8>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct PendingGroup {
    first_item_index: usize,
    codes: [u16; 3],
    count: u8,
}

/// Quantizes and packs one block in channel order [L,C,R,Ls,Rs,LFE], then bin order.
///
/// Every call starts with empty BAP1/2/4 groups. Grouped fields occupy the first
/// item's position; partial groups use their F12 filler codes. Errors describe
/// this block candidate so the caller can retry another candidate if required.
pub fn pack_block_mantissas(
    inputs: &[MantissaBlockItemInput],
) -> Result<MantissaBlockOutput, MantissaBlockError> {
    let mut ordered_inputs = inputs.iter().collect::<Vec<_>>();
    ordered_inputs.sort_by_key(|item| (item.channel, item.bin));
    for pair in ordered_inputs.windows(2) {
        if pair[0].channel == pair[1].channel && pair[0].bin == pair[1].bin {
            return Err(MantissaBlockError::DuplicateItemPosition {
                channel: pair[0].channel,
                bin: pair[0].bin,
            });
        }
    }

    let mut items = Vec::with_capacity(ordered_inputs.len());
    for input in ordered_inputs {
        let code = quantize_mantissa(input.coefficient_word, input.decoded_exponent, input.bap)
            .map_err(MantissaBlockError::Quantizer)?;
        items.push(MantissaItemObservation {
            channel: input.channel,
            bin: input.bin,
            bap: input.bap,
            code: code.value(),
            width: code.width(),
        });
    }

    let mut fields = vec![None; items.len()];
    let mut groups = [PendingGroup::default(); 3];
    for (item_index, item) in items.iter().enumerate() {
        if let Some(group_index) = group_index(item.bap) {
            let group = &mut groups[group_index];
            if group.count == 0 {
                group.first_item_index = item_index;
            }
            group.codes[usize::from(group.count)] = item.code;
            group.count += 1;
            if group.count == group_size(item.bap) {
                fields[group.first_item_index] = Some(PackedField {
                    value: grouped_value(item.bap, group.codes),
                    width: item.width,
                    grouped_bap: Some(item.bap),
                });
                *group = PendingGroup::default();
            }
        } else if item.bap != 0 {
            fields[item_index] = Some(PackedField {
                value: item.code,
                width: item.width,
                grouped_bap: None,
            });
        }
    }

    for (group_index, bap) in [1, 2, 4].into_iter().enumerate() {
        let group = &mut groups[group_index];
        if group.count == 0 {
            continue;
        }
        let group_size = usize::from(group_size(bap));
        let count = usize::from(group.count);
        group.codes[count..group_size].fill(fill_code(bap));
        let width = items[group.first_item_index].width;
        fields[group.first_item_index] = Some(PackedField {
            value: grouped_value(bap, group.codes),
            width,
            grouped_bap: Some(bap),
        });
    }

    let bit_count = fields
        .iter()
        .flatten()
        .try_fold(0_u64, |total, field| {
            total.checked_add(u64::from(field.width))
        })
        .ok_or(MantissaBlockError::BitCountOverflow)?;
    let byte_count =
        usize::try_from(bit_count.div_ceil(8)).map_err(|_| MantissaBlockError::BitCountOverflow)?;
    let mut packed_bytes = vec![0_u8; byte_count];
    let mut grouped_packs = Vec::new();
    let mut bit_cursor = 0_u64;

    for field in fields.into_iter().flatten() {
        if let Some(bap) = field.grouped_bap {
            grouped_packs.push(GroupedPackObservation {
                bap,
                value: field.value,
                width: field.width,
                start_bit: bit_cursor,
            });
        }
        for source_bit in (0..field.width).rev() {
            if (field.value >> source_bit) & 1 != 0 {
                let byte_index = usize::try_from(bit_cursor / 8)
                    .map_err(|_| MantissaBlockError::BitCountOverflow)?;
                let bit_in_byte = u8::try_from(bit_cursor % 8)
                    .map_err(|_| MantissaBlockError::BitCountOverflow)?;
                packed_bytes[byte_index] |= 1_u8 << (7 - bit_in_byte);
            }
            bit_cursor = bit_cursor
                .checked_add(1)
                .ok_or(MantissaBlockError::BitCountOverflow)?;
        }
    }

    Ok(MantissaBlockOutput {
        items,
        grouped_packs,
        packed_bytes,
        bit_count,
    })
}

const fn group_index(bap: u8) -> Option<usize> {
    match bap {
        1 => Some(0),
        2 => Some(1),
        4 => Some(2),
        _ => None,
    }
}

const fn group_size(bap: u8) -> u8 {
    match bap {
        1 | 2 => 3,
        4 => 2,
        _ => 0,
    }
}

const fn fill_code(bap: u8) -> u16 {
    match bap {
        1 => 1,
        2 => 2,
        4 => 5,
        _ => 0,
    }
}

fn grouped_value(bap: u8, codes: [u16; 3]) -> u16 {
    match bap {
        1 => 9 * codes[0] + 3 * codes[1] + codes[2],
        2 => 25 * codes[0] + 5 * codes[1] + codes[2],
        4 => 11 * codes[0] + codes[1],
        _ => 0,
    }
}
