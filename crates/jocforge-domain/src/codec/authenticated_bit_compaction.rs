// pattern: Functional Core
//! F14.1 keyless bit compaction over ordered closed exclusion intervals.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClosedBitExclusion {
    begin_bit: u64,
    end_bit: u64,
}

impl ClosedBitExclusion {
    pub const fn new(begin_bit: u64, end_bit: u64) -> Self {
        Self { begin_bit, end_bit }
    }

    pub const fn begin_bit(self) -> u64 {
        self.begin_bit
    }

    pub const fn end_bit(self) -> u64 {
        self.end_bit
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BitCompactionError {
    OddInputLength,
    ReversedRange,
    UnsortedOrOverlappingRanges,
    RangeOutOfBounds,
    SizeOverflow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactedBitStream {
    bytes: Vec<u8>,
    kept_bits: u64,
    word_count: u64,
    uncommitted_tail_bits: u8,
}

impl CompactedBitStream {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub const fn kept_bits(&self) -> u64 {
        self.kept_bits
    }

    pub const fn word_count(&self) -> u64 {
        self.word_count
    }

    pub const fn uncommitted_tail_bits(&self) -> u8 {
        self.uncommitted_tail_bits
    }
}

/// Compacts the remaining input bits in MSB-first order and applies F14.1 tail commit rules.
///
/// Input bytes are read as big-endian 16-bit words for bit numbering; output bytes retain that
/// network-order bit sequence. This primitive supports arbitrary even-byte inputs for isolated
/// verification. The production AU contract remains a separate fixed-4096-byte boundary.
pub fn compact_authenticated_bits(
    input: &[u8],
    exclusions: &[ClosedBitExclusion],
) -> Result<CompactedBitStream, BitCompactionError> {
    if input.len() % 2 != 0 {
        return Err(BitCompactionError::OddInputLength);
    }
    let input_bytes = u64::try_from(input.len()).map_err(|_| BitCompactionError::SizeOverflow)?;
    let total_bits = input_bytes
        .checked_mul(8)
        .ok_or(BitCompactionError::SizeOverflow)?;

    let mut prior_end = None;
    let mut excluded_bits = 0_u64;
    for range in exclusions {
        if range.begin_bit > range.end_bit {
            return Err(BitCompactionError::ReversedRange);
        }
        if range.end_bit >= total_bits {
            return Err(BitCompactionError::RangeOutOfBounds);
        }
        if prior_end.is_some_and(|end| range.begin_bit <= end) {
            return Err(BitCompactionError::UnsortedOrOverlappingRanges);
        }
        excluded_bits = excluded_bits
            .checked_add(range.end_bit - range.begin_bit + 1)
            .ok_or(BitCompactionError::SizeOverflow)?;
        prior_end = Some(range.end_bit);
    }
    let kept_bits = total_bits
        .checked_sub(excluded_bits)
        .ok_or(BitCompactionError::SizeOverflow)?;

    let mut compacted_bytes = Vec::with_capacity(input.len());
    let mut range_index = 0;
    let mut pending_byte = 0_u8;
    let mut pending_bits = 0_u8;
    for source_bit in 0..total_bits {
        while range_index < exclusions.len() && source_bit > exclusions[range_index].end_bit {
            range_index += 1;
        }
        if range_index < exclusions.len()
            && source_bit >= exclusions[range_index].begin_bit
            && source_bit <= exclusions[range_index].end_bit
        {
            continue;
        }

        let source_byte_index =
            usize::try_from(source_bit / 8).map_err(|_| BitCompactionError::SizeOverflow)?;
        let shift =
            7 - u8::try_from(source_bit % 8).map_err(|_| BitCompactionError::SizeOverflow)?;
        let bit = (input[source_byte_index] >> shift) & 1;
        pending_byte = (pending_byte << 1) | bit;
        pending_bits += 1;
        if pending_bits == 8 {
            compacted_bytes.push(pending_byte);
            pending_byte = 0;
            pending_bits = 0;
        }
    }

    if pending_bits > 0 {
        compacted_bytes.push(pending_byte << (8 - pending_bits));
    }

    let tail_modulus =
        u8::try_from(kept_bits % 16).map_err(|_| BitCompactionError::SizeOverflow)?;
    let uncommitted_tail_bits = if tail_modulus > 0 && tail_modulus < 8 {
        compacted_bytes.pop();
        tail_modulus
    } else {
        if tail_modulus == 8 {
            compacted_bytes.push(0);
        }
        0
    };
    if compacted_bytes.len() % 2 != 0 {
        return Err(BitCompactionError::SizeOverflow);
    }
    let word_count =
        u64::try_from(compacted_bytes.len() / 2).map_err(|_| BitCompactionError::SizeOverflow)?;

    Ok(CompactedBitStream {
        bytes: compacted_bytes,
        kept_bits,
        word_count,
        uncommitted_tail_bits,
    })
}
