// pattern: Functional Core
//! Flat7x D0 audio-frame controls and the replaceable shared-q allocation search policy.

use super::bitio::{FrameBitWriter, FrameBitWriterError};
use super::{
    bap::{FLAT7X_D0_BAP_BLOCK_COUNT, FLAT7X_D0_BAP_CHANNEL_COUNT},
    flat7x_d0_exponents::FLAT7X_D0_D15_STRATEGY_CODE,
};

const INITIAL_FRAME_CONTROL_BITS: u32 = 12;
const FRAME_COUPLING_STATE_BITS: u32 = FLAT7X_D0_BAP_BLOCK_COUNT as u32;
const CHANNEL_EXPONENT_STRATEGY_WIDTH_BITS: u8 = 2;
const CHANNEL_EXPONENT_STRATEGY_BITS: u32 = FLAT7X_D0_BAP_BLOCK_COUNT as u32
    * FLAT7X_D0_BAP_CHANNEL_COUNT as u32
    * CHANNEL_EXPONENT_STRATEGY_WIDTH_BITS as u32;
const FRAME_SNR_OFFSET_BITS: u32 = 10;
const BLOCK_START_INFO_BITS: u32 = 1;

pub const FLAT7X_D0_AUDIO_FRAME_CONTROL_BITS: u32 = INITIAL_FRAME_CONTROL_BITS
    + FRAME_COUPLING_STATE_BITS
    + CHANNEL_EXPONENT_STRATEGY_BITS
    + FRAME_SNR_OFFSET_BITS
    + BLOCK_START_INFO_BITS;
pub const FLAT7X_D0_AUDIO_FRAME_CONTROL_POLICY_ID: &str = "JOCFORGE_FLAT7X_D0_AUDFRM_POLICY_V1";
pub const FLAT7X_D0_ALLOCATION_SEARCH_POLICY_ID: &str =
    "JOCFORGE_FLAT7X_D0_MAX_ALLOCATED_BITS_Q_HIGH_TIE_V1";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Flat7xD0AllocationCandidateQ(u16);

impl Flat7xD0AllocationCandidateQ {
    pub const fn new(value: u16) -> Option<Self> {
        if value == 0 || value > 1_023 {
            None
        } else {
            Some(Self(value))
        }
    }

    pub const fn value(self) -> u16 {
        self.0
    }

    pub const fn coarse_snr_code(self) -> u16 {
        self.0 >> 4
    }

    pub const fn fine_snr_code(self) -> u16 {
        self.0 & 0x0f
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0AudioFrameControlPolicy {
    allocation_candidate: Flat7xD0AllocationCandidateQ,
}

impl Flat7xD0AudioFrameControlPolicy {
    pub const fn new(allocation_candidate: Flat7xD0AllocationCandidateQ) -> Self {
        Self {
            allocation_candidate,
        }
    }

    pub const fn allocation_candidate(self) -> Flat7xD0AllocationCandidateQ {
        self.allocation_candidate
    }

    pub const fn policy_id(self) -> &'static str {
        FLAT7X_D0_AUDIO_FRAME_CONTROL_POLICY_ID
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0AudioFrameControlReceipt {
    start_bit: u32,
    end_bit: u32,
    skipflde_bit_offset: u32,
    channel_exponent_strategy_start_bit: u32,
    channel_exponent_strategy_end_bit: u32,
    allocation_candidate: Flat7xD0AllocationCandidateQ,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0AudioFrameControlError {
    BitWriter(FrameBitWriterError),
    ExtentMismatch { actual_bits: u32 },
}

impl From<FrameBitWriterError> for Flat7xD0AudioFrameControlError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

impl Flat7xD0AudioFrameControlReceipt {
    pub const fn start_bit(self) -> u32 {
        self.start_bit
    }

    pub const fn end_bit(self) -> u32 {
        self.end_bit
    }

    pub const fn skipflde_bit_offset(self) -> u32 {
        self.skipflde_bit_offset
    }

    pub const fn channel_exponent_strategy_start_bit(self) -> u32 {
        self.channel_exponent_strategy_start_bit
    }

    pub const fn channel_exponent_strategy_end_bit(self) -> u32 {
        self.channel_exponent_strategy_end_bit
    }

    pub const fn bit_length(self) -> u32 {
        self.end_bit - self.start_bit
    }

    pub const fn allocation_candidate(self) -> Flat7xD0AllocationCandidateQ {
        self.allocation_candidate
    }

    pub const fn policy_id(self) -> &'static str {
        FLAT7X_D0_AUDIO_FRAME_CONTROL_POLICY_ID
    }

    pub const fn production_allowed(self) -> bool {
        false
    }
}

/// Writes the selected D0 `audfrm` controls and the shared raw frame SNR codes.
///
/// `frmcsnroffst` and `frmfsnroffst` are raw syntax codes. Their numerical SNR interpretation
/// remains the ETSI bit-allocation rule; this function writes their six-bit/four-bit fields only.
/// The policy chooses long blocks, D15, no AHT, no transient pre-noise processing, dither syntax
/// with all block dither flags off, default allocation parameters, no fast-gain syntax, no delta
/// allocation, skip-field syntax, no spectral extension, and no block-start information.
pub fn write_flat7x_d0_audio_frame_controls(
    writer: &mut FrameBitWriter<'_>,
    policy: Flat7xD0AudioFrameControlPolicy,
) -> Result<Flat7xD0AudioFrameControlReceipt, Flat7xD0AudioFrameControlError> {
    let start_bit = writer.cursor().bit_offset();
    writer.preflight_capacity(FLAT7X_D0_AUDIO_FRAME_CONTROL_BITS)?;

    writer.write_bit(true)?; // expstre=1
    writer.write_bit(false)?; // ahte=0
    writer.write_unsigned(0, 2)?; // snroffststr=0, shared frame-level SNR offsets
    writer.write_bit(false)?; // transproce=0
    writer.write_bit(false)?; // blkswe=0
    writer.write_bit(true)?; // dithflage=1
    writer.write_bit(false)?; // bamode=0, ETSI default allocation parameters
    writer.write_bit(false)?; // frmfgaincode=0, decoder default fast gain
    writer.write_bit(false)?; // dbaflde=0
    let skipflde_bit_offset = writer.cursor().bit_offset();
    writer.write_bit(true)?; // skipflde=1
    writer.write_bit(false)?; // spxattene=0
    writer.write_bit(false)?; // cplinu[0]=0; cplstre[0] is implicit 1.
    for _ in 1..FLAT7X_D0_BAP_BLOCK_COUNT {
        writer.write_bit(false)?; // cplstre[blk]=0; cplinu reuses the previous 0.
    }
    let channel_exponent_strategy_start_bit = writer.cursor().bit_offset();
    for _ in 0..FLAT7X_D0_BAP_BLOCK_COUNT {
        for _ in 0..FLAT7X_D0_BAP_CHANNEL_COUNT {
            writer.write_unsigned(
                u32::from(FLAT7X_D0_D15_STRATEGY_CODE),
                CHANNEL_EXPONENT_STRATEGY_WIDTH_BITS,
            )?;
        }
    }
    let channel_exponent_strategy_end_bit = writer.cursor().bit_offset();
    writer.write_unsigned(u32::from(policy.allocation_candidate.coarse_snr_code()), 6)?;
    writer.write_unsigned(u32::from(policy.allocation_candidate.fine_snr_code()), 4)?;
    writer.write_bit(false)?; // blkstrtinfoe=0 for the selected six-block frame

    let end_bit = writer.cursor().bit_offset();
    if end_bit - start_bit != FLAT7X_D0_AUDIO_FRAME_CONTROL_BITS {
        return Err(Flat7xD0AudioFrameControlError::ExtentMismatch {
            actual_bits: end_bit - start_bit,
        });
    }
    Ok(Flat7xD0AudioFrameControlReceipt {
        start_bit,
        end_bit,
        skipflde_bit_offset,
        channel_exponent_strategy_start_bit,
        channel_exponent_strategy_end_bit,
        allocation_candidate: policy.allocation_candidate,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        FLAT7X_D0_AUDIO_FRAME_CONTROL_BITS, Flat7xD0AllocationCandidateQ,
        Flat7xD0AudioFrameControlPolicy, write_flat7x_d0_audio_frame_controls,
    };
    use crate::codec::bitio::{FrameBitCursor, FrameBitWriter};

    #[test]
    fn allocation_candidate_splits_raw_coarse_and_fine_codes() {
        let candidate = Flat7xD0AllocationCandidateQ::new(0x2a7).unwrap();
        assert_eq!(candidate.coarse_snr_code(), 0x2a);
        assert_eq!(candidate.fine_snr_code(), 0x7);
        assert_eq!(Flat7xD0AllocationCandidateQ::new(0), None);
        assert_eq!(Flat7xD0AllocationCandidateQ::new(1_024), None);
    }

    #[test]
    fn frame_controls_write_the_exact_selected_policy_extent() {
        let candidate = Flat7xD0AllocationCandidateQ::new(0x2a7).unwrap();
        let mut bytes = [0_u8; 10];
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();

        let receipt = write_flat7x_d0_audio_frame_controls(
            &mut writer,
            Flat7xD0AudioFrameControlPolicy::new(candidate),
        )
        .unwrap();

        assert_eq!(receipt.start_bit(), 0);
        assert_eq!(receipt.end_bit(), 77);
        assert_eq!(receipt.end_bit(), FLAT7X_D0_AUDIO_FRAME_CONTROL_BITS);
        assert_eq!(receipt.allocation_candidate(), candidate);
        assert_eq!(receipt.policy_id(), "JOCFORGE_FLAT7X_D0_AUDFRM_POLICY_V1");
        assert!(!receipt.production_allowed());

        assert_eq!(receipt.skipflde_bit_offset(), 10);
        assert_eq!(read_bits(&bytes, 10, 1), 1);
        assert_eq!(read_bits(&bytes, 12, 6), 0);
        assert_eq!(receipt.channel_exponent_strategy_start_bit(), 18);
        assert_eq!(receipt.channel_exponent_strategy_end_bit(), 66);
        for strategy_index in 0..(6 * 4) {
            assert_eq!(read_bits(&bytes, 18 + strategy_index * 2, 2), 1);
        }
        assert_eq!(read_bits(&bytes, 66, 6), 0x2a);
        assert_eq!(read_bits(&bytes, 72, 4), 0x7);
        assert_eq!(read_bits(&bytes, 76, 1), 0);
    }

    fn read_bits(bytes: &[u8], start_bit: u32, width: u8) -> u32 {
        (start_bit..start_bit + u32::from(width)).fold(0, |value, bit_index| {
            let byte = bytes[(bit_index / 8) as usize];
            let shift = 7 - (bit_index % 8) as u8;
            (value << 1) | u32::from((byte >> shift) & 1)
        })
    }
}
