//! Fragmented EC-3 audio with one complete JOC access unit per CMAF sample.
use jocforge_application::meter_source_bridge::JocReconstructionLayout;
use std::io::{self, Read, Write};
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
    let size = u32::try_from(payload.len() + 8)
        .expect("fixed CMAF initialization and fragment metadata fit a 32-bit box");
    [size.to_be_bytes().as_slice(), &kind, payload].concat()
}

fn bmff_full_box(kind: [u8; 4], flags: u32, payload: &[u8]) -> Vec<u8> {
    bmff_box(kind, &[flags.to_be_bytes().as_slice(), payload].concat())
}

fn dec3_box(profile: JocReconstructionLayout, bitrate_kbps: u32) -> Vec<u8> {
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
    let chan_loc = if profile.uses_height_pair() {
        64
    } else if profile.uses_seven_carriers() {
        2
    } else {
        0
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

pub(crate) fn init_segment(layout: JocReconstructionLayout, rate: u32) -> Vec<u8> {
    let dec3 = dec3_box(layout, rate);
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
    movie_header_fields[16..20].copy_from_slice(&0x10000_u32.to_be_bytes());
    movie_header_fields[20..22].copy_from_slice(&0x100_u16.to_be_bytes());
    identity_matrix(&mut movie_header_fields[32..68]);
    movie_header_fields[92..96].copy_from_slice(&2_u32.to_be_bytes());
    let movie_header = bmff_full_box(*b"mvhd", 0, &movie_header_fields);

    let mut tkhd_fields = vec![0_u8; 80];
    tkhd_fields[8..12].copy_from_slice(&1_u32.to_be_bytes());
    tkhd_fields[32..34].copy_from_slice(&0x100_u16.to_be_bytes());
    identity_matrix(&mut tkhd_fields[36..72]);
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
    let entry = audio_sample_entry(&dec3);
    let stsd = bmff_full_box(
        *b"stsd",
        0,
        &[1_u32.to_be_bytes().as_slice(), &entry].concat(),
    );
    let stbl = bmff_box(
        *b"stbl",
        &[
            stsd,
            bmff_full_box(*b"stts", 0, &[0; 4]),
            bmff_full_box(*b"stsc", 0, &[0; 4]),
            bmff_full_box(*b"stsz", 0, &[0; 8]),
            bmff_full_box(*b"stco", 0, &[0; 4]),
        ]
        .concat(),
    );
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

fn identity_matrix(bytes: &mut [u8]) {
    for (offset, value) in [(0, 0x10000_u32), (16, 0x10000), (32, 0x40000000)] {
        bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
    }
}

// Fixed one-sample movie fragment. The offset is measured from the moof
// (default-base-is-moof) to the first byte of the following mdat payload.
fn movie_fragment(sequence: u32, size: u32, offset: u32) -> Vec<u8> {
    let mfhd = bmff_full_box(*b"mfhd", 0, &sequence.to_be_bytes());
    let tfhd = bmff_full_box(
        *b"tfhd",
        0x020008,
        &[1u32.to_be_bytes(), CMAF_SAMPLE_DURATION.to_be_bytes()].concat(),
    );
    let decode_time = u64::from(sequence - 1) * u64::from(CMAF_SAMPLE_DURATION);
    let tfdt = bmff_full_box(*b"tfdt", 0x01000000, &decode_time.to_be_bytes());
    let trun = bmff_full_box(
        *b"trun",
        0x301,
        &[
            1u32.to_be_bytes(),
            offset.to_be_bytes(),
            CMAF_SAMPLE_DURATION.to_be_bytes(),
            size.to_be_bytes(),
        ]
        .concat(),
    );
    bmff_box(
        *b"moof",
        &[mfhd, bmff_box(*b"traf", &[tfhd, tfdt, trun].concat())].concat(),
    )
}

pub(crate) fn package(
    mut raw: impl Read,
    mut output: impl Write,
    layout: JocReconstructionLayout,
    rate: u32,
    count: u32,
) -> io::Result<()> {
    if count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "empty CMAF track",
        ));
    }
    let size = rate * 4;
    let offset = movie_fragment(1, size, 0).len() as u32 + 8;
    output.write_all(&init_segment(layout, rate))?;
    let mut sample = vec![0; size as usize];
    for sequence in 1..=count {
        raw.read_exact(&mut sample)?;
        output.write_all(&movie_fragment(sequence, size, offset))?;
        output.write_all(&(size + 8).to_be_bytes())?;
        output.write_all(b"mdat")?;
        output.write_all(&sample)?;
    }
    if raw.read(&mut [0])? != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "trailing CMAF sample data",
        ));
    }
    output.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_specific_channel_locations_and_joc_extension() {
        for (layout, locations) in [
            (JocReconstructionLayout::FivePointOneWithoutPhase, 0),
            (JocReconstructionLayout::Flat7x, 2),
            (JocReconstructionLayout::FivePointOnePlusHeight, 64),
            (JocReconstructionLayout::ExistingSelectedFivePointOne, 0),
            (JocReconstructionLayout::HeightPairWithPhase, 64),
        ] {
            let config = dec3_box(layout, 1664);
            let bits = u64::from_be_bytes(config[8..].try_into().unwrap());
            assert_eq!(bits >> 51, 1664);
            assert_eq!((bits >> 32) & 1, 0);
            assert_eq!((bits >> 25) & 15, 1);
            assert_eq!((bits >> 16) & 511, locations);
            assert_eq!(bits & 0xffff, 0x0110);
        }
    }

    #[test]
    fn fragment_offsets_and_decode_times_follow_the_encoded_samples() {
        let layout = JocReconstructionLayout::Flat7x;
        let rate = 1664;
        let size = rate as usize * 4;
        let mut raw = vec![0xa5; 2 * size];
        raw[size..].fill(0x5a);
        let mut track = Vec::new();
        package(&raw[..], &mut track, layout, rate, 2).unwrap();
        let mut cursor = init_segment(layout, rate).len();
        for (i, sample) in raw.chunks_exact(size).enumerate() {
            let moof_size =
                u32::from_be_bytes(track[cursor..cursor + 4].try_into().unwrap()) as usize;
            let moof = &track[cursor..cursor + moof_size];
            let find = |tag: &[u8]| moof.windows(4).position(|w| w == tag).unwrap();
            let start = find(b"tfdt") + 8;
            assert_eq!(
                u64::from_be_bytes(moof[start..start + 8].try_into().unwrap()),
                i as u64 * 1536
            );
            let start = find(b"trun") + 12;
            assert_eq!(
                u32::from_be_bytes(moof[start..start + 4].try_into().unwrap()) as usize,
                moof_size + 8
            );
            cursor += moof_size;
            assert_eq!(&track[cursor + 4..cursor + 8], b"mdat");
            cursor += 8;
            assert_eq!(&track[cursor..cursor + size], sample);
            cursor += size;
        }
        assert_eq!(cursor, track.len());
        assert!(package(&raw[..size - 1], Vec::new(), layout, rate, 1).is_err());
        assert!(package(&raw[..], Vec::new(), layout, rate, 1).is_err());
    }
}
