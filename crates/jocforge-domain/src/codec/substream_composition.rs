//! E-AC-3 syncframes grouped into six-block programme access units.
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

use super::substream_topology::{BlockPartition, EmdfLocation, FramePlan, SubstreamPlan};

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
        let last = self.frames.len() - 1;
        let sizes = if self.partition == BlockPartition::Six {
            six_block_sizes(&weights, metadata.emdf.len(), bytes)?
        } else {
            Vec::new()
        };
        let blocks = self.partition.blocks();
        let mut components = Vec::with_capacity((6 / blocks) * self.frames.len());
        for start in (0..6).step_by(blocks) {
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
                        start..start + blocks,
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
                        start..start + blocks,
                        &source.bap_parameters,
                    )?);
                }
                let config = Syncframe {
                    index,
                    plan: frame,
                    rows,
                    bytes: sizes.get(index).map_or(4096, |words| words * 2),
                    metadata: &metadata,
                    carrier: (index == last && start + blocks == 6).then_some(self.emdf_location),
                    start,
                    partition: self.partition,
                };
                components.push(config);
            }
        }
        if self.partition != BlockPartition::Six {
            // Reserve each actual header, exponents, EMDF and CRC before sharing the
            // remaining whole words. Metadata occurs once per programme interval.
            let mut scratch = [0u8; 4096];
            let mut minimum = 0usize;
            for component in &mut components {
                let bits = component.write(&mut scratch, 0, None)? as usize;
                component.bytes = (bits + component.tail_bits()).div_ceil(16) * 2;
                minimum += component.bytes;
            }
            let mut words = bytes.checked_sub(minimum).ok_or_else(|| {
                format!("partition requires {minimum} syntax bytes, interval budget is {bytes}")
            })? / 2;
            let mut remaining_weight = weights.iter().sum::<usize>() * (6 / blocks);
            for component in &mut components {
                let weight = weights[component.index];
                let extra = (words * weight / remaining_weight).min((4096 - component.bytes) / 2);
                component.bytes += 2 * extra;
                remaining_weight -= weight;
                words -= extra;
            }
            if words != 0 {
                return Err("short-frame size exceeds representable capacity".into());
            }
        }
        let mut output = Vec::with_capacity(bytes);
        for component in components {
            output.extend(component.encode(&source.bap_parameters)?);
        }
        Ok(output)
    }
}

// Preserve the six-block allocation policy and its rounding for existing encodes.
fn six_block_sizes(weights: &[usize], emdf_len: usize, bytes: usize) -> Result<Vec<usize>, String> {
    let reserved = emdf_len.div_ceil(2) + 12;
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
    Ok(sizes)
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
        range: std::ops::Range<usize>,
        tables: &BapParameters,
    ) -> Result<Self, String> {
        let n = samples[0].len();
        let mut peak = [0u32; 253];
        for block in &samples[range] {
            for (p, &v) in peak.iter_mut().zip(*block) {
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
    carrier: Option<EmdfLocation>,
    start: usize,
    partition: BlockPartition,
}
impl Syncframe<'_> {
    fn tail_bits(&self) -> usize {
        if self.carrier == Some(EmdfLocation::FrameEnd) {
            self.metadata.emdf.len() * 8 + 32
        } else {
            18
        }
    }

    fn encode(&self, tables: &BapParameters) -> Result<Vec<u8>, String> {
        let mut buffer = vec![0; self.bytes];
        let mut alloc: Vec<Vec<u8>> = self
            .rows
            .iter()
            .map(|r| vec![0; r.samples[0].len()])
            .collect();
        let syntax = self.write(&mut buffer, 0, None)? as usize;
        if syntax + self.tail_bits() > self.bytes * 8 {
            return Err(format!(
                "substream {} metadata exceeds {} bytes",
                self.index, self.bytes
            ));
        }
        let budget = (self.bytes * 8 - syntax - self.tail_bits()) / self.partition.blocks();
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
        if used + self.tail_bits() > self.bytes * 8 {
            return Err("dependent frame overflow".into());
        }
        if self.carrier == Some(EmdfLocation::FrameEnd) {
            let emdf = self.metadata.emdf;
            let length_bits = u16::try_from(emdf.len() * 8)
                .ok()
                .filter(|n| *n < 1 << 14)
                .ok_or("EMDF exceeds auxdatal field")?;
            let length_pos = self.bytes - 4;
            buffer[length_pos - emdf.len()..length_pos].copy_from_slice(emdf);
            // auxdatal (14 bits), auxdatae=1, crcrsv=0 precede the CRC16.
            buffer[length_pos..length_pos + 2]
                .copy_from_slice(&((length_bits << 2) | 2).to_be_bytes());
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
        bits!(self.partition.code(), 2);
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
        if self.index == 0 && self.partition != BlockPartition::Six {
            bits!(self.start == 0, 1);
        } // convsync
        bits!(self.carrier.is_some(), 1);
        if self.carrier.is_some() {
            bits!(1, 6);
            bits!(1, 8);
            bits!(16, 8);
        }
        let blocks = self.partition.blocks();
        if blocks == 6 {
            bits!(1, 1);
            bits!(0, 1);
        } // implicit exponent strategies and no AHT on short frames
        bits!(0, 2); // shared frame-level SNR offset
        bits!(0, 1); // transient pre-noise processing disabled
        bits!(1, 1); // preserve the source block switches
        bits!(1, 1); // explicit dither flags
        bits!(0, 1); // standard bit allocation parameters
        bits!(0, 1); // standard fast gain
        bits!(0, 1); // no delta bit allocation
        bits!(self.carrier == Some(EmdfLocation::SkipField), 1); // only the metadata owner has skip fields
        bits!(0, 1); // no spectral extension attenuation
        if n > 1 {
            bits!(0, blocks as u8);
        }
        for block in 0..blocks {
            for _ in 0..n {
                bits!(if block == 0 { 3 } else { 0 }, 2);
            }
        }
        if self.plan.lfe {
            bits!(1u32 << (blocks - 1), blocks as u8);
        }
        if self.index == 0 {
            if blocks != 6 {
                bits!(0, 1);
            } else {
                for _ in 0..n {
                    bits!(0, 5);
                }
            }
        } // independent six-block converter exponent strategies
        bits!(q, 10);
        if blocks > 1 {
            bits!(0, 1);
        }
        let order = [
            MantissaChannel::Left,
            MantissaChannel::Center,
            MantissaChannel::Right,
            MantissaChannel::LeftSurround,
            MantissaChannel::RightSurround,
            MantissaChannel::Lfe,
        ];
        for local in 0..blocks {
            let block = self.start + local;
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
                bits!(0, if local == 0 { 4 } else { 1 });
            }
            if local == 0 {
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
            if self.carrier == Some(EmdfLocation::SkipField) {
                let metadata_block = if blocks == 6 { 2 } else { blocks - 1 };
                bits!(local == metadata_block, 1);
                if local == metadata_block {
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_independent_frames_mark_only_the_first_interval_for_converter_sync() {
        let tables = super::super::bap::load_bap_parameters(Some(include_bytes!(
            "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-PUBLIC-BAP-PROVIDER-V1/public-bap-48k-v1.json"
        ))).unwrap();
        let metadata = FrameMetadata {
            emdf: &[],
            dialnorm: 24,
            compr: None,
            dynrng: [None; 6],
            mixing: Flat7xD0MixingMetadataValues {
                mixmdo: 2,
                ltrtcmixlev: 4,
                lorocmixlev: 4,
                ltrtsurmixlev: 4,
                lorosurmixlev: 4,
            },
        };
        let plan = FramePlan {
            channels: vec![0, 1, 2, 3, 4],
            map: 0,
            lfe: false,
        };
        let samples = [0u32; 217];
        for (token, code) in [("1x6", 0), ("1+1+1+1+1+1", 0), ("2+2+2", 1), ("3+3", 2)] {
            let partition = token.parse::<BlockPartition>().unwrap();
            let blocks = partition.blocks();
            for start in (0..6).step_by(blocks) {
                let rows = (0..5)
                    .map(|_| {
                        SpectralRow::new(
                            [&samples[..]; 6],
                            [false; 6],
                            false,
                            start..start + blocks,
                            &tables,
                        )
                        .unwrap()
                    })
                    .collect();
                let frame = Syncframe {
                    index: 0,
                    plan: &plan,
                    rows,
                    bytes: 1024,
                    metadata: &metadata,
                    carrier: None,
                    start,
                    partition,
                }
                .encode(&tables)
                .unwrap();
                assert_eq!((frame[4] >> 4) & 3, code);
                // Independent 3/2, no LFE or compr, with the programme mix metadata.
                assert_eq!(frame[9] & 0x80 != 0, start == 0);
                assert_eq!(eac3_crc16(&frame[2..]), 0);
            }
        }
        assert!("1+2+3".parse::<BlockPartition>().is_err());
    }
    #[test]
    fn frame_end_writes_auxdatal_in_bits_and_protects_the_container_with_crc() {
        let tables = super::super::bap::load_bap_parameters(Some(include_bytes!(
            "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-PUBLIC-BAP-PROVIDER-V1/public-bap-48k-v1.json"
        ))).unwrap();
        let emdf: Vec<u8> = (0..511).map(|i| (i % 251) as u8).collect();
        let metadata = FrameMetadata {
            emdf: &emdf,
            dialnorm: 24,
            compr: None,
            dynrng: [None; 6],
            mixing: Flat7xD0MixingMetadataValues {
                mixmdo: 2,
                ltrtcmixlev: 4,
                lorocmixlev: 4,
                ltrtsurmixlev: 4,
                lorosurmixlev: 4,
            },
        };
        let plan = FramePlan {
            channels: vec![0],
            map: 0x8000,
            lfe: false,
        };
        let samples = [0u32; 217];
        let row = SpectralRow::new([&samples; 6], [false; 6], false, 5..6, &tables).unwrap();
        let mut syncframe = Syncframe {
            index: 1,
            plan: &plan,
            rows: vec![row],
            bytes: 1024,
            metadata: &metadata,
            carrier: Some(EmdfLocation::FrameEnd),
            start: 5,
            partition: BlockPartition::One,
        };
        let frame = syncframe.encode(&tables).unwrap();
        let end = frame.len() - 4;
        assert_eq!(&frame[end - emdf.len()..end], emdf.as_slice());
        let trailer = u16::from_be_bytes([frame[end], frame[end + 1]]);
        assert_eq!(usize::from(trailer >> 2), emdf.len() * 8);
        assert_eq!(trailer & 2, 2);
        assert_eq!(eac3_crc16(&frame[2..]), 0);
        syncframe.bytes = 512;
        assert!(syncframe.encode(&tables).is_err());
    }
}
