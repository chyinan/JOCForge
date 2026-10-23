// pattern: Functional Core
//! Internal F12 grouped quantizer helper for the washed BAP1/2/4 slice only.

use super::mantissa::{MantissaQuantizerError, quantize_mantissa};

const BAP1_GROUP: usize = 0;
const BAP2_GROUP: usize = 1;
const BAP4_GROUP: usize = 2;
const WORD_BITS: u8 = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroupedBap {
    Bap1,
    Bap2,
    Bap4,
}

impl GroupedBap {
    pub const fn value(self) -> u8 {
        match self {
            Self::Bap1 => 1,
            Self::Bap2 => 2,
            Self::Bap4 => 4,
        }
    }

    const fn group_index(self) -> usize {
        match self {
            Self::Bap1 => BAP1_GROUP,
            Self::Bap2 => BAP2_GROUP,
            Self::Bap4 => BAP4_GROUP,
        }
    }

    const fn group_size(self) -> u8 {
        match self {
            Self::Bap1 | Self::Bap2 => 3,
            Self::Bap4 => 2,
        }
    }

    const fn width(self) -> u8 {
        match self {
            Self::Bap1 => 5,
            Self::Bap2 | Self::Bap4 => 7,
        }
    }

    const fn fill_code(self) -> u16 {
        match self {
            Self::Bap1 => 1,
            Self::Bap2 => 2,
            Self::Bap4 => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GroupedItemInput {
    pub bap: GroupedBap,
    pub coefficient_word: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GroupedItemObservation {
    bap: GroupedBap,
    code: u16,
    width: u8,
}

impl GroupedItemObservation {
    pub const fn bap(&self) -> GroupedBap {
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
    value: u16,
    width: u8,
    start_bits: u64,
}

impl GroupedPackObservation {
    pub const fn value(&self) -> u16 {
        self.value
    }

    pub const fn width(&self) -> u8 {
        self.width
    }

    pub const fn start_bits(&self) -> u64 {
        self.start_bits
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct GroupBuffer {
    codes: [u16; 3],
    reservation_ids: [u64; 3],
    count: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ReservedField {
    id: u64,
    width: u8,
    code: Option<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupedQuantizerState {
    groups: [GroupBuffer; 3],
    reservations: Vec<ReservedField>,
    next_reservation_id: u64,
    pending_bit_count: u8,
    pending_accumulator: u16,
    effective_bits: u64,
    emitted_word_memory_bytes: Vec<u8>,
}

impl GroupedQuantizerState {
    pub fn cold() -> Self {
        Self {
            groups: [GroupBuffer::default(); 3],
            reservations: Vec::new(),
            next_reservation_id: 0,
            pending_bit_count: 0,
            pending_accumulator: 0,
            effective_bits: 0,
            emitted_word_memory_bytes: Vec::new(),
        }
    }

    pub fn group_counts(&self) -> [u32; 3] {
        self.groups.map(|group| u32::from(group.count))
    }

    pub const fn pending_bit_count(&self) -> u8 {
        self.pending_bit_count
    }

    pub const fn pending_accumulator_u16(&self) -> u16 {
        self.pending_accumulator
    }

    pub const fn effective_bits(&self) -> u64 {
        self.effective_bits
    }

    pub fn emitted_word_memory_bytes(&self) -> &[u8] {
        &self.emitted_word_memory_bytes
    }

    /// Quantizes items, reserves each item width, and compacts completed groups in place.
    ///
    /// This helper has no logical capacity argument and does not promise rollback.
    pub fn quantize(
        &mut self,
        inputs: &[GroupedItemInput],
    ) -> Result<GroupedQuantizerCall, GroupedQuantizerError> {
        let mut items = Vec::with_capacity(inputs.len());
        let mut packs = Vec::new();
        for input in inputs {
            let code = quantized_code(input.coefficient_word, input.bap)?;
            let item = GroupedItemObservation {
                bap: input.bap,
                code,
                width: input.bap.width(),
            };
            items.push(item);
            if let Some(pack) = self.reserve_group_code(input.bap, code)? {
                packs.push(pack);
            }
        }
        Ok(GroupedQuantizerCall {
            return_code: 0,
            items,
            packs,
        })
    }

    /// Fills and compacts existing incomplete reservations without reserving more bits.
    pub fn flush(&mut self) -> Result<GroupedQuantizerFlush, GroupedQuantizerError> {
        let mut items = Vec::new();
        let mut packs = Vec::new();
        for bap in [GroupedBap::Bap1, GroupedBap::Bap2, GroupedBap::Bap4] {
            let group_index = bap.group_index();
            let group_size = bap.group_size();
            let existing_count = self.groups[group_index].count;
            if existing_count == 0 {
                continue;
            }
            while self.groups[group_index].count < group_size {
                let count = usize::from(self.groups[group_index].count);
                let code = bap.fill_code();
                self.groups[group_index].codes[count] = code;
                self.groups[group_index].count += 1;
                if bap != GroupedBap::Bap4 {
                    items.push(GroupedItemObservation {
                        bap,
                        code,
                        width: bap.width(),
                    });
                }
            }
            if self.groups[group_index].count == group_size {
                let value = grouped_value(bap, self.groups[group_index].codes);
                packs.push(self.compact_group(bap, value, existing_count)?);
                self.groups[group_index].count = 0;
            }
        }
        Ok(GroupedQuantizerFlush {
            return_code: 0,
            items,
            packs,
        })
    }

    fn reserve_group_code(
        &mut self,
        bap: GroupedBap,
        code: u16,
    ) -> Result<Option<GroupedPackObservation>, GroupedQuantizerError> {
        let group_index = bap.group_index();
        let count = usize::from(self.groups[group_index].count);
        let id = self.next_reservation_id;
        self.next_reservation_id = self
            .next_reservation_id
            .checked_add(1)
            .ok_or(GroupedQuantizerError::BitCountOverflow)?;
        self.effective_bits = self
            .effective_bits
            .checked_add(u64::from(bap.width()))
            .ok_or(GroupedQuantizerError::BitCountOverflow)?;
        self.reservations.push(ReservedField {
            id,
            width: bap.width(),
            code: None,
        });
        self.groups[group_index].codes[count] = code;
        self.groups[group_index].reservation_ids[count] = id;
        self.groups[group_index].count += 1;
        if self.groups[group_index].count != bap.group_size() {
            self.rebuild_writer_state();
            return Ok(None);
        }

        let value = grouped_value(bap, self.groups[group_index].codes);
        let packed = self.compact_group(bap, value, self.groups[group_index].count)?;
        self.groups[group_index].count = 0;
        Ok(Some(packed))
    }

    fn compact_group(
        &mut self,
        bap: GroupedBap,
        value: u16,
        reservation_count: u8,
    ) -> Result<GroupedPackObservation, GroupedQuantizerError> {
        let group_index = bap.group_index();
        let ids = self.groups[group_index].reservation_ids;
        let first_id = ids[0];
        let first_position = self
            .reservations
            .iter()
            .position(|field| field.id == first_id)
            .ok_or(GroupedQuantizerError::MissingReservation)?;
        let absolute_start_bits = self.reservations[..first_position]
            .iter()
            .try_fold(0_u64, |bits, field| {
                bits.checked_add(u64::from(field.width))
            })
            .ok_or(GroupedQuantizerError::BitCountOverflow)?;
        let start_bits = absolute_start_bits % u64::from(WORD_BITS);
        let removed_width = u64::from(bap.width())
            .checked_mul(u64::from(reservation_count.saturating_sub(1)))
            .ok_or(GroupedQuantizerError::BitCountOverflow)?;
        self.effective_bits = self
            .effective_bits
            .checked_sub(removed_width)
            .ok_or(GroupedQuantizerError::BitCountOverflow)?;
        self.reservations[first_position].code = Some(value);
        let removed_ids = &ids[1..usize::from(reservation_count)];
        self.reservations
            .retain(|field| field.id == first_id || !removed_ids.contains(&field.id));
        self.rebuild_writer_state();
        Ok(GroupedPackObservation {
            value,
            width: bap.width(),
            start_bits,
        })
    }

    fn rebuild_writer_state(&mut self) {
        self.emitted_word_memory_bytes.clear();
        self.pending_bit_count = 0;
        self.pending_accumulator = 0;
        let mut word = 0_u16;
        let mut bits_in_word = 0_u8;
        for field in &self.reservations {
            let value = field.code.unwrap_or(0);
            for bit_index in (0..field.width).rev() {
                word = (word << 1) | ((value >> bit_index) & 1);
                bits_in_word += 1;
                if bits_in_word == WORD_BITS {
                    self.emitted_word_memory_bytes
                        .extend_from_slice(&word.to_le_bytes());
                    word = 0;
                    bits_in_word = 0;
                }
            }
        }
        self.pending_bit_count = bits_in_word;
        self.pending_accumulator = if bits_in_word == 0 {
            0
        } else {
            word << (WORD_BITS - bits_in_word)
        };
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupedQuantizerCall {
    return_code: i32,
    items: Vec<GroupedItemObservation>,
    packs: Vec<GroupedPackObservation>,
}

impl GroupedQuantizerCall {
    pub const fn return_code(&self) -> i32 {
        self.return_code
    }

    pub fn items(&self) -> &[GroupedItemObservation] {
        &self.items
    }

    pub fn packs(&self) -> &[GroupedPackObservation] {
        &self.packs
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupedQuantizerFlush {
    return_code: i32,
    items: Vec<GroupedItemObservation>,
    packs: Vec<GroupedPackObservation>,
}

impl GroupedQuantizerFlush {
    pub const fn return_code(&self) -> i32 {
        self.return_code
    }

    pub fn items(&self) -> &[GroupedItemObservation] {
        &self.items
    }

    pub fn packs(&self) -> &[GroupedPackObservation] {
        &self.packs
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroupedQuantizerError {
    UnobservedScalarQuantizerFailure(MantissaQuantizerError),
    BitCountOverflow,
    MissingReservation,
}

fn quantized_code(coefficient_word: u32, bap: GroupedBap) -> Result<u16, GroupedQuantizerError> {
    match quantize_mantissa(coefficient_word, bap.value()) {
        Ok(code) => Ok(code.value()),
        Err(
            MantissaQuantizerError::NonFiniteInput | MantissaQuantizerError::NonFiniteIntermediate,
        ) => Ok(0),
        Err(error) => Err(GroupedQuantizerError::UnobservedScalarQuantizerFailure(
            error,
        )),
    }
}

fn grouped_value(bap: GroupedBap, codes: [u16; 3]) -> u16 {
    match bap {
        GroupedBap::Bap1 => 9 * codes[0] + 3 * codes[1] + codes[2],
        GroupedBap::Bap2 => 25 * codes[0] + 5 * codes[1] + codes[2],
        GroupedBap::Bap4 => 11 * codes[0] + codes[1],
    }
}
