use super::MetadataOwnerError;
use serde_json::Value;
use sha2::{Digest, Sha256};

const TIMER_PERIODS: [i64; 6] = [24_000, 48_000, 48_000, 4_800, 24_000, 48_000];
const TIMER_FIELDS: [usize; 6] = [0, 5, 4, 6, 7, 8];
const TIMER_PRIORITY: [usize; 6] = [1, 2, 5, 0, 4, 3];
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TimerState {
    pub remaining: [i64; 6],
    pub cached: [i64; 6],
    pub mode: i64,
    /// The local v1 canonical `counter` slot carries the native counter_low_s16 value.
    pub counter: i64,
}

impl TimerState {
    pub fn cold() -> Self {
        Self {
            remaining: TIMER_PERIODS,
            cached: [-1, -1024, -1024, -1024, -1024, -1024],
            mode: 1,
            counter: -513,
        }
    }

    pub fn digest(&self) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"MetadataTimerStateV1\0");
        for value in self
            .remaining
            .into_iter()
            .chain(self.cached)
            .chain([self.mode, self.counter])
        {
            digest.update(value.to_le_bytes());
        }
        digest.finalize().into()
    }

    pub fn advance(
        &self,
        config: &[Value],
        samples: i64,
        counter_flag: bool,
    ) -> Result<TimerAdvance, MetadataOwnerError> {
        if !matches!(self.mode, 0 | 1)
            || config.len() != 9
            || samples < 0
            || i32::try_from(samples).is_err()
            || i16::try_from(self.counter).is_err()
        {
            return Err(MetadataOwnerError::Timer);
        }

        let mut input = [0_i64; 9];
        for (index, value) in config.iter().enumerate() {
            input[index] = value.as_i64().ok_or(MetadataOwnerError::Timer)?;
        }

        let mut next = self.clone();
        for remaining in &mut next.remaining {
            *remaining = remaining
                .checked_sub(samples)
                .ok_or(MetadataOwnerError::Timer)?;
        }

        let mut output_counter_flag = counter_flag;
        if (-512..=512).contains(&next.counter) {
            let counter = i16::try_from(next.counter).map_err(|_| MetadataOwnerError::Timer)?;
            let decremented = counter.wrapping_sub(1);
            next.counter = i64::from(decremented);
            if decremented == 0 {
                output_counter_flag = false;
            } else if i32::from(decremented).unsigned_abs().is_power_of_two() {
                output_counter_flag = true;
            }
        }

        if matches!(input[0], -2 | -1) && next.cached[0] == -1 {
            next.remaining[0] = TIMER_PERIODS[0];
        } else {
            next.cached[0] = input[0];
        }

        for timer in 1..6 {
            let value = input[TIMER_FIELDS[timer]];
            if matches!(value, -1024 | -512) {
                if next.cached[timer] == -1024 {
                    next.remaining[timer] = TIMER_PERIODS[timer];
                }
            } else if timer == 4 {
                if value > next.cached[timer] {
                    next.cached[timer] = value;
                }
            } else {
                next.cached[timer] = value;
            }
        }

        let mut output = input;
        let mut refreshed_field = None;
        if next.mode == 1 {
            output[0] = -2;
            output[4..=8].fill(-512);
            if !output_counter_flag {
                for timer in TIMER_PRIORITY {
                    if next.remaining[timer] <= 0 {
                        let field = TIMER_FIELDS[timer];
                        output[field] = next.cached[timer];
                        next.cached[timer] = if timer == 0 { -1 } else { -1024 };
                        next.remaining[timer] = TIMER_PERIODS[timer];
                        refreshed_field = Some(field as u8);
                        break;
                    }
                }
            }
        }

        Ok(TimerAdvance {
            next,
            output,
            refreshed_field,
            counter_flag: output_counter_flag,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TimerAdvance {
    pub next: TimerState,
    pub output: [i64; 9],
    pub refreshed_field: Option<u8>,
    pub counter_flag: bool,
}

#[cfg(test)]
mod timer_vectors;

pub(super) fn encode_id1_payload(fields: &[i64; 9]) -> Result<Vec<u8>, MetadataOwnerError> {
    let mut writer = BitWriter::default();
    writer.write(0, 2);

    if fields[0] >= 0 {
        writer.write(1, 1);
        writer.write_checked(fields[0], 3)?;
    } else {
        writer.write(0, 1);
    }

    writer.write_checked(fields[1], 4)?;
    if fields[1] != 0 {
        writer.write_checked(fields[2], 1)?;
        writer.write_checked(fields[3], 1)?;
    }

    let ranges = [
        (-116, 11, 7),
        (-116, 11, 7),
        (-232, 23, 8),
        (-232, 23, 8),
        (-15, 15, 5),
    ];
    for (field_index, (minimum, maximum, width)) in (4..=8).zip(ranges) {
        let value = fields[field_index];
        if value == -512 {
            writer.write(0, 1);
        } else {
            writer.write(1, 1);
            let clamped = value.clamp(minimum, maximum) - minimum;
            writer.write(
                u64::try_from(clamped).map_err(|_| MetadataOwnerError::WireField)?,
                width,
            );
        }
    }

    writer.write(0, 1);
    writer.pad_to_byte();
    Ok(writer.into_bytes())
}

#[derive(Default)]
struct BitWriter {
    bits: Vec<bool>,
}

impl BitWriter {
    fn write(&mut self, value: u64, width: usize) {
        for bit in (0..width).rev() {
            self.bits.push(((value >> bit) & 1) != 0);
        }
    }

    fn write_checked(&mut self, value: i64, width: usize) -> Result<(), MetadataOwnerError> {
        if value < 0 {
            return Err(MetadataOwnerError::WireField);
        }
        let value = u64::try_from(value).map_err(|_| MetadataOwnerError::WireField)?;
        if width < 64 && value >= (1_u64 << width) {
            return Err(MetadataOwnerError::WireField);
        }
        self.write(value, width);
        Ok(())
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
