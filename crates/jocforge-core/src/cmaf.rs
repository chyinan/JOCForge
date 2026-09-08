// pattern: Functional Core

use crate::{
    BlockPartition, CompositionRequest, CompositionResolver, EmdfProfile, JocCarriage,
    JocContainer, JocProfile, LfeOwnership,
};

const CMAF_SAMPLE_DURATION: u32 = 1536;

#[derive(Default)]
struct Bits {
    bits: Vec<bool>,
}

impl Bits {
    fn push(&mut self, value: u64, width: usize) {
        for shift in (0..width).rev() {
            self.bits.push(value & (1_u64 << shift) != 0);
        }
    }

    fn bytes(self) -> Vec<u8> {
        let byte_count = self.bits.len().div_ceil(8);
        let mut output = vec![0_u8; byte_count];
        for (index, bit) in self.bits.into_iter().enumerate() {
            if bit {
                output[index / 8] |= 0x80 >> (index % 8);
            }
        }
        output
    }
}

fn bmff_box(kind: [u8; 4], payload: &[u8]) -> Vec<u8> {
    let size = u32::try_from(payload.len() + 8).unwrap_or(u32::MAX);
    [size.to_be_bytes().as_slice(), &kind, payload].concat()
}

fn bmff_full_box(kind: [u8; 4], flags: u32, payload: &[u8]) -> Vec<u8> {
    [flags.to_be_bytes().as_slice(), payload]
        .concat()
        .pipe(|full| bmff_box(kind, &full))
}

trait Pipe: Sized {
    fn pipe<T>(self, function: impl FnOnce(Self) -> T) -> T {
        function(self)
    }
}

impl<T> Pipe for T {}

fn dec3_box(profile: JocProfile, bitrate_kbps: u32) -> Vec<u8> {
    let mut bits = Bits::default();
    bits.push(u64::from(bitrate_kbps.min(0x1fff)), 13);
    bits.push(0, 3); // one independent substream
    bits.push(0, 2); // 48 kHz
    bits.push(16, 5); // E-AC-3 bsid
    bits.push(0, 1); // reserved
    bits.push(0, 1); // asvc
    bits.push(0, 3); // bsmod
    bits.push(7, 3); // five full-band channels
    bits.push(0, 1); // no LFE in the canonical CMAF path
    bits.push(0, 3); // reserved
    bits.push(1, 4); // one dependent D0
    let chan_loc = match profile {
        JocProfile::Idx0 | JocProfile::Idx3 => 0,
        JocProfile::Idx1 => 2,
        JocProfile::Idx2 | JocProfile::Idx4 => 64,
    };
    bits.push(chan_loc, 9);
    bits.push(0, 7); // reserved
    bits.push(1, 1); // type-A JOC extension
    bits.push(16, 8); // complexity index
    bmff_box(*b"dec3", &bits.bytes())
}

fn audio_sample_entry(dec3: &[u8]) -> Vec<u8> {
    let mut payload = vec![0_u8; 6];
    payload.extend_from_slice(&1_u16.to_be_bytes());
    payload.extend_from_slice(&[0_u8; 8]);
    payload.extend_from_slice(&2_u16.to_be_bytes());
    payload.extend_from_slice(&16_u16.to_be_bytes());
    payload.extend_from_slice(&[0_u8; 4]);
    payload.extend_from_slice(&(48_000_u32 << 16).to_be_bytes());
    payload.extend_from_slice(dec3);
    bmff_box(*b"ec-3", &payload)
}

fn init_segment(dec3: &[u8]) -> Vec<u8> {
    let ftyp_payload = [
        b"cmfc".as_slice(),
        &0_u32.to_be_bytes(),
        b"isom".as_slice(),
        b"iso6".as_slice(),
        b"cmfc".as_slice(),
        b"ceao".as_slice(),
    ]
    .concat();
    let ftyp = bmff_box(*b"ftyp", &ftyp_payload);

    let mut movie_header_fields = vec![0_u8; 96];
    movie_header_fields[8..12].copy_from_slice(&48_000_u32.to_be_bytes());
    let movie_header = bmff_full_box(*b"mvhd", 0, &movie_header_fields);

    let mut tkhd_fields = vec![0_u8; 80];
    tkhd_fields[8..12].copy_from_slice(&1_u32.to_be_bytes());
    let tkhd = bmff_full_box(*b"tkhd", 7, &tkhd_fields);

    let mut media_header_fields = vec![0_u8; 20];
    media_header_fields[8..12].copy_from_slice(&48_000_u32.to_be_bytes());
    let media_header = bmff_full_box(*b"mdhd", 0, &media_header_fields);

    let mut hdlr_fields = vec![0_u8; 20];
    hdlr_fields[4..8].copy_from_slice(b"soun");
    hdlr_fields.extend_from_slice(b"SoundHandler\0");
    let hdlr = bmff_full_box(*b"hdlr", 0, &hdlr_fields);
    let smhd = bmff_full_box(*b"smhd", 0, &[0_u8; 4]);
    let url = bmff_full_box(*b"url ", 1, &[]);
    let dref = bmff_full_box(
        *b"dref",
        0,
        &[1_u32.to_be_bytes().as_slice(), &url].concat(),
    );
    let dinf = bmff_box(*b"dinf", &dref);
    let entry = audio_sample_entry(dec3);
    let stsd = bmff_full_box(
        *b"stsd",
        0,
        &[1_u32.to_be_bytes().as_slice(), &entry].concat(),
    );
    let stbl = bmff_box(*b"stbl", &stsd);
    let minf = bmff_box(*b"minf", &[smhd, dinf, stbl].concat());
    let mdia = bmff_box(*b"mdia", &[media_header, hdlr, minf].concat());
    let trak = bmff_box(*b"trak", &[tkhd, mdia].concat());
    let trex_fields = [
        1_u32.to_be_bytes().as_slice(),
        1_u32.to_be_bytes().as_slice(),
        CMAF_SAMPLE_DURATION.to_be_bytes().as_slice(),
        0_u32.to_be_bytes().as_slice(),
        0_u32.to_be_bytes().as_slice(),
    ]
    .concat();
    let trex = bmff_full_box(*b"trex", 0, &trex_fields);
    let mvex = bmff_box(*b"mvex", &trex);
    let moov = bmff_box(*b"moov", &[movie_header, trak, mvex].concat());
    [ftyp, moov].concat()
}

fn fragment(sample: &[u8], sequence: u32, decode_time: u64) -> Vec<u8> {
    let mfhd = bmff_full_box(*b"mfhd", 0, &sequence.to_be_bytes());
    let tfhd_fields = [
        1_u32.to_be_bytes().as_slice(),
        CMAF_SAMPLE_DURATION.to_be_bytes().as_slice(),
    ]
    .concat();
    let tfhd = bmff_full_box(*b"tfhd", 0x0002_0008, &tfhd_fields);
    let tfdt = bmff_full_box(*b"tfdt", 0x0100_0000, &decode_time.to_be_bytes());
    let trun_fields = [
        1_u32.to_be_bytes().as_slice(),
        0_u32.to_be_bytes().as_slice(),
        CMAF_SAMPLE_DURATION.to_be_bytes().as_slice(),
        u32::try_from(sample.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes()
            .as_slice(),
    ]
    .concat();
    let trun = bmff_full_box(*b"trun", 0x0000_0301, &trun_fields);
    let traf = bmff_box(*b"traf", &[tfhd.clone(), tfdt, trun].concat());
    let moof = bmff_box(*b"moof", &[mfhd, traf].concat());
    let data_offset = u32::try_from(moof.len() + 8).unwrap_or(u32::MAX);
    let trun_fields = [
        1_u32.to_be_bytes().as_slice(),
        data_offset.to_be_bytes().as_slice(),
        CMAF_SAMPLE_DURATION.to_be_bytes().as_slice(),
        u32::try_from(sample.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes()
            .as_slice(),
    ]
    .concat();
    let trun = bmff_full_box(*b"trun", 0x0000_0301, &trun_fields);
    let traf = bmff_box(
        *b"traf",
        &[
            tfhd,
            bmff_full_box(*b"tfdt", 0x0100_0000, &decode_time.to_be_bytes()),
            trun,
        ]
        .concat(),
    );
    let moof = bmff_box(
        *b"moof",
        &[bmff_full_box(*b"mfhd", 0, &sequence.to_be_bytes()), traf].concat(),
    );
    [moof, bmff_box(*b"mdat", sample)].concat()
}

/// Serializes a constrained CMAF JOC init segment and one fragmented sample
/// per complete 1,536-sample raw JOC access unit.
pub fn cmaf_joc_init_and_fragments(
    profile: JocProfile,
    emdf_profile: EmdfProfile,
    bitrate_kbps: u32,
    samples: &[Vec<u8>],
) -> Result<Vec<u8>, String> {
    let plan = CompositionResolver::resolve(&CompositionRequest {
        profile,
        container: JocContainer::Cmaf,
        carriage: JocCarriage::EtsiAuxDependent,
        emdf_profile,
        bitrate_kbps,
        partitions: vec![BlockPartition::Six],
        dependent_count: 1,
        lfe_ownership: LfeOwnership::Absent,
    })
    .map_err(|error| format!("invalid CMAF composition: {error}"))?;
    let expected_size = plan.rate.carrier_frame_bytes + plan.rate.d0_frame_bytes;
    if samples.is_empty()
        || samples
            .iter()
            .any(|sample| sample.len() != expected_size || sample.get(0..2) != Some(&[0x0b, 0x77]))
    {
        return Err("CMAF sample must be one complete 1536-sample JOC access unit".to_owned());
    }
    let dec3 = dec3_box(profile, plan.rate.total_bitrate_kbps);
    let mut output = init_segment(&dec3);
    for (index, sample) in samples.iter().enumerate() {
        output.extend_from_slice(&fragment(
            sample,
            u32::try_from(index + 1).unwrap_or(u32::MAX),
            u64::try_from(index).unwrap_or(u64::MAX) * u64::from(CMAF_SAMPLE_DURATION),
        ));
    }
    Ok(output)
}
