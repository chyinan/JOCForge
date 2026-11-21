//! Six-block E-AC-3 access units with programme-owned channel maps.
use super::{
    bap::{BapParameters, BapProfileInput, BapRowInput, PreparedBapRow, prepare_bap_row},
    bitio::{FrameBitCursor, FrameBitWriter},
    flat7x_d0_exponents::{D0ExponentStrategy, Flat7xD0ExponentRow, prepare_d0_exponent_row},
    flat7x_d0_spectrum::Flat7xD0SpectrumBlock,
    frame_builder::Flat7xD0MixingMetadataValues,
    grouped_quantizer::{MantissaBlockItemInput, MantissaChannel, pack_block_mantissas},
    selected_frame_trial::SelectedSourceNumericPlan,
};
use crate::integrity::crc16::eac3_crc16;

use super::substream_topology::{FramePlan, SubstreamPlan};

impl SubstreamPlan {
    pub fn encode(
        &self,
        source: &SelectedSourceNumericPlan,
        extended: Option<&[Flat7xD0SpectrumBlock; 6]>,
        metadata: FrameMetadata<'_>,
        bytes: usize,
    ) -> Result<Vec<u8>, String> {
        // The last dependent carries metadata. Every syncframe has an even size
        // and its own CRC. Share the remaining words in proportion to channel count.
        let weights: Vec<_> = self
            .frames
            .iter()
            .map(|f| 4 * f.channels.len() + usize::from(f.lfe))
            .collect();
        if self.frames.iter().any(|frame| frame.channels.contains(&6)) != extended.is_some() {
            return Err("spectral rows do not match the substream plan".into());
        }
        let reserved = metadata.emdf.len().div_ceil(2) + 12;
        let available = (bytes / 2)
            .checked_sub(reserved)
            .ok_or("insufficient access unit budget")?;
        let total: usize = weights.iter().sum();
        let mut sizes: Vec<usize> = weights
            .iter()
            .map(|w| (available * w / total).min(2048))
            .collect();
        let last = sizes.len() - 1;
        sizes[last] = (sizes[last] + reserved).min(2048);
        let mut left = bytes / 2 - sizes.iter().sum::<usize>();
        while left > 0 {
            let mut progress = false;
            for size in &mut sizes {
                if *size < 2048 && left > 0 {
                    *size += 1;
                    left -= 1;
                    progress = true;
                }
            }
            if !progress {
                return Err("access unit exceeds syncframe capacity".into());
            }
        }
        let mut output = Vec::with_capacity(bytes);
        for (index, frame) in self.frames.iter().enumerate() {
            let mut rows = Vec::new();
            for &lane in &frame.channels {
                let replaced = self.frames[1..]
                    .iter()
                    .any(|dependent| dependent.channels.contains(&lane));
                let use_extra = extended.is_some() && lane >= 3 && (index != 0 || !replaced);
                let samples: [&[u32]; 6] = std::array::from_fn(|block| {
                    if use_extra {
                        &extended.unwrap()[block].coefficient_bits_by_channel()[lane - 3][..]
                    } else {
                        &source.coefficients_by_block[block].fullband_words[lane][..]
                    }
                });
                let switches =
                    std::array::from_fn(|block| !use_extra && source.short_flags[block][lane]);
                rows.push(SpectralRow::new(
                    samples,
                    switches,
                    false,
                    &source.bap_parameters,
                )?);
            }
            if frame.lfe {
                let samples = std::array::from_fn(|block| {
                    source.coefficients_by_block[block].lfe_words.as_slice()
                });
                rows.push(SpectralRow::new(
                    samples,
                    [false; 6],
                    true,
                    &source.bap_parameters,
                )?);
            }
            let config = Syncframe {
                index,
                plan: frame,
                rows,
                bytes: sizes[index] * 2,
                metadata: &metadata,
                carry: index == last,
            };
            output.extend(config.encode(&source.bap_parameters)?);
        }
        Ok(output)
    }
}

pub struct FrameMetadata<'a> {
    pub emdf: &'a [u8],
    pub dialnorm: u8,
    pub compr: Option<u8>,
    pub dynrng: [Option<u8>; 6],
    pub mixing: Flat7xD0MixingMetadataValues,
}

struct SpectralRow<'a> {
    samples: [&'a [u32]; 6],
    switches: [bool; 6],
    exponents: Flat7xD0ExponentRow,
    mask: PreparedBapRow,
}
impl<'a> SpectralRow<'a> {
    fn new(
        samples: [&'a [u32]; 6],
        switches: [bool; 6],
        lfe: bool,
        tables: &BapParameters,
    ) -> Result<Self, String> {
        let n = samples[0].len();
        let mut peak = [0u32; 253];
        for block in samples {
            for (p, &v) in peak.iter_mut().zip(block) {
                *p = (*p).max(v & 0x7fff_ffff);
            }
        }
        let strategy = if lfe {
            D0ExponentStrategy::D15
        } else {
            D0ExponentStrategy::D45
        };
        let exponents = prepare_d0_exponent_row(&peak, strategy).map_err(|e| format!("{e:?}"))?;
        let mask = prepare_bap_row(
            &BapRowInput {
                decoded_exponents: &exponents.decoded_exponents()[..n],
                end_exclusive: n,
                candidate: 1,
                slow_decay_code: 2,
                fast_decay_code: 1,
                slow_gain_code: 1,
                db_per_bit_code: 2,
                floor_code: 7,
                fast_gain_code: 4,
                caller_all_active_fine_zero: false,
                profile: BapProfileInput::selected(),
            },
            tables,
        )
        .map_err(|e| format!("{e:?}"))?;
        Ok(Self {
            samples,
            switches,
            exponents,
            mask,
        })
    }
}

struct Syncframe<'a> {
    index: usize,
    plan: &'a FramePlan,
    rows: Vec<SpectralRow<'a>>,
    bytes: usize,
    metadata: &'a FrameMetadata<'a>,
    carry: bool,
}
impl Syncframe<'_> {
    fn encode(&self, tables: &BapParameters) -> Result<Vec<u8>, String> {
        let mut buffer = vec![0; self.bytes];
        let mut alloc: Vec<Vec<u8>> = self
            .rows
            .iter()
            .map(|r| vec![0; r.samples[0].len()])
            .collect();
        let syntax = self.write(&mut buffer, 0, None)? as usize;
        if syntax + 18 > self.bytes * 8 {
            return Err(format!(
                "substream {} metadata exceeds {} bytes",
                self.index, self.bytes
            ));
        }
        let budget = (self.bytes * 8 - syntax - 18) / 6;
        let mut lo = 0u16;
        let mut hi = 1024u16;
        while hi - lo > 1 {
            let q = (lo + hi) / 2;
            for (r, baps) in self.rows.iter().zip(&mut alloc) {
                r.mask.allocate(q, tables, baps);
            }
            if mantissa_bits(&alloc) <= budget {
                lo = q
            } else {
                hi = q
            }
        }
        for (r, baps) in self.rows.iter().zip(&mut alloc) {
            if lo == 0 {
                baps.fill(0)
            } else {
                r.mask.allocate(lo, tables, baps)
            }
        }
        buffer.fill(0);
        let used = self.write(&mut buffer, lo, Some(&alloc))? as usize;
        if used + 18 > self.bytes * 8 {
            return Err("dependent frame overflow".into());
        }
        let end = self.bytes - 2;
        let mut crc = eac3_crc16(&buffer[2..end]);
        if crc == 0x0b77 {
            buffer[end - 1] |= 1;
            crc = eac3_crc16(&buffer[2..end]);
        }
        buffer[end..].copy_from_slice(&crc.to_be_bytes());
        Ok(buffer)
    }
    fn write(&self, buf: &mut [u8], q: u16, baps: Option<&[Vec<u8>]>) -> Result<u32, String> {
        let mut w =
            FrameBitWriter::new(buf, FrameBitCursor::new(0)).map_err(|e| format!("{e:?}"))?;
        macro_rules! bits {
            ($value:expr,$width:expr) => {
                w.write_unsigned(($value) as u32, $width)
                    .map_err(|e| format!("substream bits: {e:?}"))?
            };
        }
        let n = self.plan.channels.len();
        let acmod = match n {
            1 => 1,
            2 => 2,
            5 => 7,
            _ => return Err("invalid substream width".into()),
        };
        bits!(0x0b77, 16);
        bits!(self.index != 0, 2);
        bits!(self.index.saturating_sub(1), 3);
        bits!(self.bytes / 2 - 1, 11);
        bits!(0, 2);
        bits!(3, 2);
        bits!(acmod, 3);
        bits!(self.plan.lfe, 1);
        bits!(16, 5);
        bits!(self.metadata.dialnorm, 5);
        bits!(self.metadata.compr.is_some(), 1);
        if let Some(code) = self.metadata.compr {
            bits!(code, 8);
        }
        if self.index > 0 {
            bits!(1, 1);
            bits!(self.plan.map, 16);
        }
        bits!(self.index == 0, 1); // I0 supplies the programme downmix settings.
        if self.index == 0 {
            let mix = self.metadata.mixing;
            bits!(mix.mixmdo, 2);
            bits!(mix.ltrtcmixlev, 3);
            bits!(mix.lorocmixlev, 3);
            bits!(mix.ltrtsurmixlev, 3);
            bits!(mix.lorosurmixlev, 3);
            if self.plan.lfe {
                bits!(0, 1);
            }
            bits!(0, 5); // pgmscle, extpgmscle, mixdef and frmmixcfginfoe
        }
        bits!(0, 1); // infomdate
        bits!(self.carry, 1);
        if self.carry {
            bits!(1, 6);
            bits!(1, 8);
            bits!(16, 8);
        }
        bits!(1, 1); // explicit per-block exponent strategies
        bits!(0, 1); // adaptive hybrid transform disabled
        bits!(0, 2); // shared frame-level SNR offset
        bits!(0, 1); // transient pre-noise processing disabled
        bits!(1, 1); // preserve the source block switches
        bits!(1, 1); // explicit dither flags
        bits!(0, 1); // standard bit allocation parameters
        bits!(0, 1); // standard fast gain
        bits!(0, 1); // no delta bit allocation
        bits!(self.carry, 1); // only the metadata owner has skip fields
        bits!(0, 1); // no spectral extension attenuation
        if n > 1 {
            bits!(0, 6);
        }
        for block in 0..6 {
            for _ in 0..n {
                bits!(if block == 0 { 3 } else { 0 }, 2);
            }
        }
        if self.plan.lfe {
            bits!(32, 6);
        }
        if self.index == 0 {
            for _ in 0..n {
                bits!(0, 5);
            }
        } // independent six-block converter exponent strategies
        bits!(q, 10);
        bits!(0, 1);
        let order = [
            MantissaChannel::Left,
            MantissaChannel::Center,
            MantissaChannel::Right,
            MantissaChannel::LeftSurround,
            MantissaChannel::RightSurround,
            MantissaChannel::Lfe,
        ];
        for block in 0..6 {
            for r in &self.rows[..n] {
                bits!(r.switches[block], 1);
            }
            bits!(0, n as u8);
            bits!(self.metadata.dynrng[block].is_some(), 1);
            if let Some(code) = self.metadata.dynrng[block] {
                bits!(code, 8);
            }
            bits!(0, 1); // SPX disabled
            if n == 2 {
                bits!(0, if block == 0 { 4 } else { 1 });
            }
            if block == 0 {
                for row in &self.rows[..n] {
                    bits!((row.samples[0].len() - 73) / 3, 6);
                }
                for (i, r) in self.rows.iter().enumerate() {
                    let lfe = i == n;
                    bits!(r.exponents.seed().value(), 4);
                    let groups = (r.samples[0].len() - 1) / if lfe { 3 } else { 12 };
                    for group in &r.exponents.groups()[..groups] {
                        bits!(group.value(), 7);
                    }
                    if !lfe {
                        bits!(0, 2);
                    }
                }
            }
            if self.index == 0 {
                bits!(0, 1);
            } // convsnroffste
            if self.carry {
                bits!(block == 2, 1);
                if block == 2 {
                    bits!(self.metadata.emdf.len(), 9);
                    w.write_bytes(self.metadata.emdf)
                        .map_err(|e| format!("{e:?}"))?;
                }
            }
            if let Some(baps) = baps {
                let mut items = Vec::new();
                for (i, (r, row_bap)) in self.rows.iter().zip(baps).enumerate() {
                    for (bin, (&word, &bap)) in r.samples[block].iter().zip(row_bap).enumerate() {
                        items.push(MantissaBlockItemInput {
                            channel: order[i],
                            bin: bin as u16,
                            coefficient_word: word,
                            decoded_exponent: i16::from(r.exponents.decoded_exponents()[bin]),
                            bap,
                        });
                    }
                }
                let packed = pack_block_mantissas(&items).map_err(|e| format!("{e:?}"))?;
                w.write_bit_slice(packed.packed_bytes(), 0, packed.bit_count() as u32)
                    .map_err(|e| format!("{e:?}"))?;
            }
        }
        Ok(w.cursor().bit_offset())
    }
}

fn mantissa_bits(rows: &[Vec<u8>]) -> usize {
    let mut counts = [0usize; 16];
    for row in rows {
        for &b in row {
            counts[usize::from(b)] += 1;
        }
    }
    let widths = [0, 0, 0, 3, 0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16];
    counts.iter().zip(widths).map(|(n, b)| n * b).sum::<usize>()
        + counts[1].div_ceil(3) * 5
        + counts[2].div_ceil(3) * 7
        + counts[4].div_ceil(2) * 7
}
