// pattern: Functional Core
//! Six-block Flat7x D0 audio production and source-independent q-fit candidate search.

use super::{
    bap::{
        BapError, FLAT7X_D0_BAP_BLOCK_COUNT, FLAT7X_D0_BAP_CHANNEL_COUNT, Flat7xD0BapFrameInput,
        Flat7xD0BapParameters, allocate_flat7x_d0_bap_frame,
    },
    bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
    d0_frame_assembler::{
        D0_CRC2_FIELD_BITS, D0_TARGET_FRAME_BITS, D0Block2SkipFieldInput,
        Flat7xD0FrameAssemblyError, Flat7xD0FrameAssemblyReceipt, Flat7xD0FrameBitInput,
        Flat7xD0FrameTailPolicy, assemble_flat7x_d0_frame_bits_with_tail_policy,
    },
    d0_frame_layout::{CallerOwnedUnboundD0Block2InsertionPoint, D0SixBlockLayoutError},
    emdf_skip_field::{D0EmdfSkipFieldWriteError, flat7x_d0_block2_skipfld_bit_length},
    flat7x_d0_audio_block::{
        FLAT7X_D0_DEFAULT_FAST_GAIN_CODE, Flat7xD0AudioBlockCandidate, Flat7xD0AudioBlockError,
        Flat7xD0AudioBlockInput, Flat7xD0AudioBlockReceipt, measure_flat7x_d0_audio_block_bit_cost,
        write_flat7x_d0_audio_block,
    },
    flat7x_d0_exponents::Flat7xD0ExponentRow,
    flat7x_d0_frame_controls::{
        FLAT7X_D0_ALLOCATION_SEARCH_POLICY_ID, FLAT7X_D0_AUDIO_FRAME_CONTROL_BITS,
        Flat7xD0AllocationCandidateQ, Flat7xD0AudioFrameControlPolicy,
    },
    flat7x_d0_mantissa::Flat7xD0AudioBlockIndex,
    flat7x_d0_spectrum::Flat7xD0SpectrumBlock,
    frame_builder::{
        Flat7xD0BsiStructure, Flat7xD0BsiStructureError, measure_flat7x_d0_bsi_structure,
    },
    grouped_quantizer::MantissaBlockError,
    mantissa::MantissaQuantizerError,
};
use crate::profile::Flat7xI0D0TargetPlan;

pub const FLAT7X_D0_DEFAULT_SLOW_DECAY_CODE: u8 = 2;
pub const FLAT7X_D0_DEFAULT_FAST_DECAY_CODE: u8 = 1;
pub const FLAT7X_D0_DEFAULT_SLOW_GAIN_CODE: u8 = 1;
pub const FLAT7X_D0_DEFAULT_DB_PER_BIT_CODE: u8 = 2;
pub const FLAT7X_D0_DEFAULT_FLOOR_CODE: u8 = 7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0AudioFrameInput<'a> {
    pub target: Flat7xI0D0TargetPlan,
    pub bsi_structure: Flat7xD0BsiStructure<'a>,
    pub complete_emdf_container: &'a [u8],
    pub bap_parameters: &'a Flat7xD0BapParameters,
    pub spectrum_by_block: &'a [Flat7xD0SpectrumBlock; FLAT7X_D0_BAP_BLOCK_COUNT],
    pub exponent_rows_by_block:
        &'a [[Flat7xD0ExponentRow; FLAT7X_D0_BAP_CHANNEL_COUNT]; FLAT7X_D0_BAP_BLOCK_COUNT],
    pub dynamic_range_codes: [Option<u8>; FLAT7X_D0_BAP_BLOCK_COUNT],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Flat7xD0AudioFrameReceipt {
    allocation_candidate: Flat7xD0AllocationCandidateQ,
    allocated_mantissa_bits: u32,
    bsi_bit_length: u32,
    block_receipts: [Flat7xD0AudioBlockReceipt; FLAT7X_D0_BAP_BLOCK_COUNT],
    frame_assembly: Flat7xD0FrameAssemblyReceipt,
}

impl Flat7xD0AudioFrameReceipt {
    pub const fn allocation_candidate(self) -> Flat7xD0AllocationCandidateQ {
        self.allocation_candidate
    }

    pub const fn bsi_bit_length(self) -> u32 {
        self.bsi_bit_length
    }

    pub const fn allocated_mantissa_bits(self) -> u32 {
        self.allocated_mantissa_bits
    }

    pub const fn block_receipts(self) -> [Flat7xD0AudioBlockReceipt; FLAT7X_D0_BAP_BLOCK_COUNT] {
        self.block_receipts
    }

    pub const fn frame_assembly(self) -> Flat7xD0FrameAssemblyReceipt {
        self.frame_assembly
    }

    pub const fn q_search_policy_id(self) -> &'static str {
        FLAT7X_D0_ALLOCATION_SEARCH_POLICY_ID
    }

    pub const fn production_allowed(self) -> bool {
        false
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xD0AudioFrameCandidate {
    frame_bytes: Vec<u8>,
    receipt: Flat7xD0AudioFrameReceipt,
}

impl Flat7xD0AudioFrameCandidate {
    pub fn frame_bytes(&self) -> &[u8] {
        &self.frame_bytes
    }

    pub const fn receipt(&self) -> Flat7xD0AudioFrameReceipt {
        self.receipt
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flat7xD0AudioFrameError {
    BsiStructure(Flat7xD0BsiStructureError),
    EmdfSkipField(D0EmdfSkipFieldWriteError),
    Bap(BapError),
    AudioBlock {
        block_index: u8,
        error: Flat7xD0AudioBlockError,
    },
    FrameAssembly(Flat7xD0FrameAssemblyError),
    BitWriter(FrameBitWriterError),
    NoFittingAllocationCandidate,
}

impl From<Flat7xD0BsiStructureError> for Flat7xD0AudioFrameError {
    fn from(error: Flat7xD0BsiStructureError) -> Self {
        Self::BsiStructure(error)
    }
}

impl From<D0EmdfSkipFieldWriteError> for Flat7xD0AudioFrameError {
    fn from(error: D0EmdfSkipFieldWriteError) -> Self {
        Self::EmdfSkipField(error)
    }
}

impl From<BapError> for Flat7xD0AudioFrameError {
    fn from(error: BapError) -> Self {
        Self::Bap(error)
    }
}

impl From<Flat7xD0FrameAssemblyError> for Flat7xD0AudioFrameError {
    fn from(error: Flat7xD0FrameAssemblyError) -> Self {
        Self::FrameAssembly(error)
    }
}

impl From<FrameBitWriterError> for Flat7xD0AudioFrameError {
    fn from(error: FrameBitWriterError) -> Self {
        Self::BitWriter(error)
    }
}

/// Produces the maximum allocated-bit D0 frame candidate that fits the complete frame.
///
/// The search binds `q` to the exact frame-level SNR codes, computes a fresh six-block BAP frame,
/// measures exact grouped mantissa width for every q, then applies the actual mantissa quantizer
/// and frame writer only to candidates that can beat the current best. Candidate zero remains a
/// valid raw syntax vector with zero BAP but is excluded from usable audio candidates. No monotonic
/// q-to-cost assumption is used; equal bit costs keep the higher q encountered first. The D0 frame
/// assembler validates BSI, audfrm, skip-field, zero-filled auxbits, exact extent, and CRC2. This
/// producer has no source-history owner or F12/F13/F15 receipt and remains production-disabled.
pub fn produce_flat7x_d0_audio_frame(
    input: &Flat7xD0AudioFrameInput<'_>,
) -> Result<Flat7xD0AudioFrameCandidate, Flat7xD0AudioFrameError> {
    let bsi_bit_length =
        measure_flat7x_d0_bsi_structure(input.target, &input.bsi_structure)?.value();
    let emdf_length = u16::try_from(input.complete_emdf_container.len()).map_err(|_| {
        Flat7xD0AudioFrameError::EmdfSkipField(D0EmdfSkipFieldWriteError::ContainerTooLarge)
    })?;
    let skipfield_bits = flat7x_d0_block2_skipfld_bit_length(emdf_length)?;

    let decoded_exponents_by_block = std::array::from_fn(|block| {
        std::array::from_fn(|channel| {
            *input.exponent_rows_by_block[block][channel].decoded_exponents()
        })
    });

    let maximum_content_bits = D0_TARGET_FRAME_BITS
        .checked_sub(D0_CRC2_FIELD_BITS + 1)
        .ok_or(Flat7xD0AudioFrameError::NoFittingAllocationCandidate)?;
    let mut best_candidate = None;
    let mut best_content_bits = 0_u32;

    for raw_q in (1_u16..=1_023).rev() {
        let candidate = Flat7xD0AllocationCandidateQ::new(raw_q)
            .ok_or(Flat7xD0AudioFrameError::NoFittingAllocationCandidate)?;
        let fine_zero = candidate.value().trailing_zeros() >= 4;
        let bap_frame = allocate_flat7x_d0_bap_frame(
            input.bap_parameters,
            &Flat7xD0BapFrameInput {
                decoded_exponents_by_block: &decoded_exponents_by_block,
                candidate: candidate.value(),
                slow_decay_code: FLAT7X_D0_DEFAULT_SLOW_DECAY_CODE,
                fast_decay_code: FLAT7X_D0_DEFAULT_FAST_DECAY_CODE,
                slow_gain_code: FLAT7X_D0_DEFAULT_SLOW_GAIN_CODE,
                db_per_bit_codes: [FLAT7X_D0_DEFAULT_DB_PER_BIT_CODE; FLAT7X_D0_BAP_BLOCK_COUNT],
                floor_code: FLAT7X_D0_DEFAULT_FLOOR_CODE,
                fast_gain_code: FLAT7X_D0_DEFAULT_FAST_GAIN_CODE,
                caller_all_active_fine_zero_by_block: [fine_zero; FLAT7X_D0_BAP_BLOCK_COUNT],
            },
        )?;

        if bap_frame
            .blocks()
            .iter()
            .all(|block| block.rows().iter().flatten().all(|bap| *bap == 0))
        {
            continue;
        }

        let frame_controls = Flat7xD0AudioFrameControlPolicy::new(candidate);
        let mut block_audio_bits = 0_u32;
        let mut allocated_mantissa_bits = 0_u32;
        for block_index in 0..FLAT7X_D0_BAP_BLOCK_COUNT {
            let index_value = u8::try_from(block_index)
                .map_err(|_| Flat7xD0AudioFrameError::NoFittingAllocationCandidate)?;
            let block_input = Flat7xD0AudioBlockInput {
                block_index: Flat7xD0AudioBlockIndex::new(index_value)
                    .ok_or(Flat7xD0AudioFrameError::NoFittingAllocationCandidate)?,
                spectrum: &input.spectrum_by_block[usize::from(index_value)],
                exponent_rows: &input.exponent_rows_by_block[usize::from(index_value)],
                bap_frame: &bap_frame,
                frame_control_policy: frame_controls,
                dynamic_range_code: input.dynamic_range_codes[usize::from(index_value)],
            };
            let block_cost =
                measure_flat7x_d0_audio_block_bit_cost(&block_input).map_err(|error| {
                    Flat7xD0AudioFrameError::AudioBlock {
                        block_index: index_value,
                        error,
                    }
                })?;
            block_audio_bits = block_audio_bits
                .checked_add(block_cost.bit_length())
                .ok_or(Flat7xD0AudioFrameError::NoFittingAllocationCandidate)?;
            allocated_mantissa_bits = allocated_mantissa_bits
                .checked_add(block_cost.mantissa_bits())
                .ok_or(Flat7xD0AudioFrameError::NoFittingAllocationCandidate)?;
        }
        let candidate_content_bits = bsi_bit_length
            .checked_add(FLAT7X_D0_AUDIO_FRAME_CONTROL_BITS)
            .and_then(|bits| bits.checked_add(block_audio_bits))
            .and_then(|bits| bits.checked_add(skipfield_bits))
            .ok_or(Flat7xD0AudioFrameError::NoFittingAllocationCandidate)?;
        if candidate_content_bits > maximum_content_bits
            || candidate_content_bits <= best_content_bits
        {
            continue;
        }

        let mut block_candidates = Vec::with_capacity(FLAT7X_D0_BAP_BLOCK_COUNT);
        let mut candidate_rejected = false;
        for block_index in 0..FLAT7X_D0_BAP_BLOCK_COUNT {
            let index_value = u8::try_from(block_index)
                .map_err(|_| Flat7xD0AudioFrameError::NoFittingAllocationCandidate)?;
            let block_index = Flat7xD0AudioBlockIndex::new(index_value)
                .ok_or(Flat7xD0AudioFrameError::NoFittingAllocationCandidate)?;
            match write_flat7x_d0_audio_block(&Flat7xD0AudioBlockInput {
                block_index,
                spectrum: &input.spectrum_by_block[usize::from(index_value)],
                exponent_rows: &input.exponent_rows_by_block[usize::from(index_value)],
                bap_frame: &bap_frame,
                frame_control_policy: frame_controls,
                dynamic_range_code: input.dynamic_range_codes[usize::from(index_value)],
            }) {
                Ok(block_candidate) => block_candidates.push(block_candidate),
                Err(error) if is_candidate_local_mantissa_error(error) => {
                    candidate_rejected = true;
                    break;
                }
                Err(error) => {
                    return Err(Flat7xD0AudioFrameError::AudioBlock {
                        block_index: index_value,
                        error,
                    });
                }
            }
        }
        if candidate_rejected {
            continue;
        }
        let block_candidates: [Flat7xD0AudioBlockCandidate; FLAT7X_D0_BAP_BLOCK_COUNT] =
            block_candidates
                .try_into()
                .map_err(|_| Flat7xD0AudioFrameError::NoFittingAllocationCandidate)?;
        let block_receipts = block_candidates
            .each_ref()
            .map(Flat7xD0AudioBlockCandidate::receipt);
        if block_receipts
            .iter()
            .all(|receipt| receipt.nonzero_bap_bins() == 0)
        {
            continue;
        }

        let skip_fields = [D0Block2SkipFieldInput {
            insertion_point: CallerOwnedUnboundD0Block2InsertionPoint::new(
                block_receipts[2].prefix_before_skipfield_bits(),
            ),
            complete_emdf_container: input.complete_emdf_container,
        }];
        let audio_blocks = [
            block_candidates[0]
                .as_frame_audio_block_input(&[])
                .ok_or(Flat7xD0AudioFrameError::NoFittingAllocationCandidate)?,
            block_candidates[1]
                .as_frame_audio_block_input(&[])
                .ok_or(Flat7xD0AudioFrameError::NoFittingAllocationCandidate)?,
            block_candidates[2]
                .as_frame_audio_block_input(&skip_fields)
                .ok_or(Flat7xD0AudioFrameError::NoFittingAllocationCandidate)?,
            block_candidates[3]
                .as_frame_audio_block_input(&[])
                .ok_or(Flat7xD0AudioFrameError::NoFittingAllocationCandidate)?,
            block_candidates[4]
                .as_frame_audio_block_input(&[])
                .ok_or(Flat7xD0AudioFrameError::NoFittingAllocationCandidate)?,
            block_candidates[5]
                .as_frame_audio_block_input(&[])
                .ok_or(Flat7xD0AudioFrameError::NoFittingAllocationCandidate)?,
        ];
        let frame_input = Flat7xD0FrameBitInput {
            audio_frame_controls: Flat7xD0AudioFrameControlPolicy::new(candidate),
            audio_blocks,
        };
        let mut frame_bytes = vec![0_u8; usize::from(input.target.d0().frame_bytes())];
        let frame_assembly = {
            let mut writer = FrameBitWriter::new(&mut frame_bytes, FrameBitCursor::new(0))?;
            match assemble_flat7x_d0_frame_bits_with_tail_policy(
                input.target,
                input.bsi_structure,
                frame_input,
                Flat7xD0FrameTailPolicy::ZeroFillUnusedAuxBits,
                &mut writer,
            ) {
                Ok(receipt) => receipt,
                Err(error) if is_candidate_frame_budget_error(error) => continue,
                Err(error) => return Err(Flat7xD0AudioFrameError::FrameAssembly(error)),
            }
        };
        best_content_bits = candidate_content_bits;
        best_candidate = Some(Flat7xD0AudioFrameCandidate {
            frame_bytes,
            receipt: Flat7xD0AudioFrameReceipt {
                allocation_candidate: candidate,
                allocated_mantissa_bits,
                bsi_bit_length,
                block_receipts,
                frame_assembly,
            },
        });
    }

    best_candidate.ok_or(Flat7xD0AudioFrameError::NoFittingAllocationCandidate)
}

fn is_candidate_local_mantissa_error(error: Flat7xD0AudioBlockError) -> bool {
    matches!(
        error,
        Flat7xD0AudioBlockError::Mantissa(MantissaBlockError::Quantizer(
            MantissaQuantizerError::ReservedCodeSentinel
                | MantissaQuantizerError::IntegerOutOfRange
        ))
    )
}

fn is_candidate_frame_budget_error(error: Flat7xD0FrameAssemblyError) -> bool {
    matches!(
        error,
        Flat7xD0FrameAssemblyError::FrameTailNoSpace { .. }
            | Flat7xD0FrameAssemblyError::Layout(
                D0SixBlockLayoutError::FrameSpanExceedsTarget { .. }
            )
    )
}
