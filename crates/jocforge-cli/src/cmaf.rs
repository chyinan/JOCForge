//! Fragmented EC-3 audio with one complete JOC access unit per CMAF sample.
use crate::encode_request::JocObjectCount;
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

fn dec3_box(profile: JocReconstructionLayout, bitrate_kbps: u32, lfe: bool) -> Vec<u8> {
    dec3_for_track(
        profile,
        bitrate_kbps,
        16,
        1,
        lfe,
        0,
        JocObjectCount::Fifteen,
    )
}

fn dec3_for_track(
    profile: JocReconstructionLayout,
    bitrate_kbps: u32,
    bsid: u8,
    dependents: u8,
    lfe: bool,
    bsmod: u8,
    object_count: JocObjectCount,
) -> Vec<u8> {
    let mut bits = Bits::default();
    bits.push(u64::from(bitrate_kbps.min(0x1fff)), 13);
    bits.push(0, 3); // one independent substream
    bits.push(0, 2); // 48 kHz
    bits.push(u64::from(bsid), 5); // E-AC-3 bsid
    bits.push(0, 1); // reserved
    bits.push(0, 1); // asvc
    bits.push(u64::from(bsmod), 3); // bsmod
    bits.push(7, 3); // five full-band channels
    bits.push(u64::from(lfe), 1);
    bits.push(0, 3); // reserved
    bits.push(u64::from(dependents), 4);
    let chan_loc = if profile.uses_height_pair() {
        64
    } else if profile.uses_seven_carriers() {
        2
    } else {
        0
    };
    if dependents == 0 {
        bits.push(0, 1);
    } else {
        bits.push(chan_loc, 9);
    }
    bits.push(0, 7); // reserved
    bits.push(1, 1); // type-A JOC extension
    bits.push(object_count as u64 + 1, 8); // Includes the separate LFE object.
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

pub(crate) fn init_segment(layout: JocReconstructionLayout, rate: u32, lfe: bool) -> Vec<u8> {
    let dec3 = dec3_box(layout, rate, lfe);
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

    let (movie_header, trak) = track_boxes(&dec3, None);
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

fn track_boxes(dec3: &[u8], samples: Option<(u32, u32, u64)>) -> (Vec<u8>, Vec<u8>) {
    let header = |kind, flags, fields: &[u8], duration_offset: usize| {
        if let Some((count, _, _)) = samples {
            let duration = u64::from(count) * u64::from(CMAF_SAMPLE_DURATION);
            bmff_full_box(
                kind,
                0x01000000 | flags,
                &[
                    &[0u8; 16][..],
                    &fields[8..duration_offset],
                    &duration.to_be_bytes(),
                    &fields[duration_offset + 4..],
                ]
                .concat(),
            )
        } else {
            bmff_full_box(kind, flags, fields)
        }
    };
    let mut movie_header_fields = vec![0_u8; 96];
    movie_header_fields[8..12].copy_from_slice(&48_000_u32.to_be_bytes());
    movie_header_fields[16..20].copy_from_slice(&0x10000_u32.to_be_bytes());
    movie_header_fields[20..22].copy_from_slice(&0x100_u16.to_be_bytes());
    identity_matrix(&mut movie_header_fields[32..68]);
    movie_header_fields[92..96].copy_from_slice(&2_u32.to_be_bytes());
    let movie_header = header(*b"mvhd", 0, &movie_header_fields, 12);

    let mut tkhd_fields = vec![0_u8; 80];
    tkhd_fields[8..12].copy_from_slice(&1_u32.to_be_bytes());
    tkhd_fields[32..34].copy_from_slice(&0x100_u16.to_be_bytes());
    identity_matrix(&mut tkhd_fields[36..72]);
    let tkhd = header(*b"tkhd", 7, &tkhd_fields, 16);

    let mut media_header_fields = vec![0_u8; 20];
    media_header_fields[8..12].copy_from_slice(&48_000_u32.to_be_bytes());
    let media_header = header(*b"mdhd", 0, &media_header_fields, 12);

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
    let stbl = if let Some((count, size, offset)) = samples {
        let stts = bmff_full_box(
            *b"stts",
            0,
            &[
                1u32.to_be_bytes(),
                count.to_be_bytes(),
                1536u32.to_be_bytes(),
            ]
            .concat(),
        );
        let stsc = bmff_full_box(
            *b"stsc",
            0,
            &[
                1u32.to_be_bytes(),
                1u32.to_be_bytes(),
                count.to_be_bytes(),
                1u32.to_be_bytes(),
            ]
            .concat(),
        );
        let stsz = bmff_full_box(
            *b"stsz",
            0,
            &[size.to_be_bytes(), count.to_be_bytes()].concat(),
        );
        let co64 = bmff_full_box(
            *b"co64",
            0,
            &[1u32.to_be_bytes().as_slice(), &offset.to_be_bytes()].concat(),
        );
        bmff_box(*b"stbl", &[stsd, stts, stsc, stsz, co64].concat())
    } else {
        bmff_box(
            *b"stbl",
            &[
                stsd,
                bmff_full_box(*b"stts", 0, &[0; 4]),
                bmff_full_box(*b"stsc", 0, &[0; 4]),
                bmff_full_box(*b"stsz", 0, &[0; 8]),
                bmff_full_box(*b"stco", 0, &[0; 4]),
            ]
            .concat(),
        )
    };
    let minf = bmff_box(*b"minf", &[smhd, dinf, stbl].concat());
    let mdia = bmff_box(*b"mdia", &[media_header, hdlr, minf].concat());
    let trak = bmff_box(*b"trak", &[tkhd, mdia].concat());
    (movie_header, trak)
}

/// Wrap a native constant-rate JOC stream in a conventional MP4 audio track.
/// Each sample retains the complete six-block interval and all dependent streams.
pub fn write_composed_m4a(
    input: &std::path::Path,
    output: &std::path::Path,
    layout: JocReconstructionLayout,
    rate: u32,
    dependents: u8,
    cancelled: &std::sync::atomic::AtomicBool,
) -> io::Result<()> {
    write_m4a_with_object_count(
        input,
        output,
        layout,
        rate,
        dependents,
        JocObjectCount::Fifteen,
        cancelled,
    )
}

pub fn write_m4a_with_object_count(
    input: &std::path::Path,
    output: &std::path::Path,
    layout: JocReconstructionLayout,
    rate: u32,
    dependents: u8,
    object_count: JocObjectCount,
    cancelled: &std::sync::atomic::AtomicBool,
) -> io::Result<()> {
    use std::{
        fs::File,
        io::{BufReader, BufWriter, Seek},
        sync::atomic::Ordering,
    };
    let mut raw = BufReader::new(File::open(input)?);
    let len = raw.get_ref().metadata()?.len();
    let size = rate
        .checked_mul(4)
        .filter(|s| *s > 0 && *s <= 8191 * 4)
        .ok_or_else(|| io::Error::other("invalid JOC bitrate"))?;
    if len == 0 || len % u64::from(size) != 0 || dependents > 6 {
        return Err(io::Error::other("invalid JOC access unit extent"));
    }
    let count = u32::try_from(len / u64::from(size))
        .map_err(|_| io::Error::other("too many MP4 samples"))?;
    let mut first = [0u8; 7];
    raw.read_exact(&mut first)?;
    raw.rewind()?;
    let bsid = first[5] >> 3;
    if first[..2] != [0x0b, 0x77] || !matches!(bsid, 6 | 8 | 16) {
        return Err(io::Error::other("invalid independent syncframe"));
    }
    let (lfe, bsmod) = if bsid == 16 {
        (first[4] & 1 != 0, 0)
    } else {
        (first[6] & 1 != 0, first[5] & 7)
    };
    let dec3 = dec3_for_track(layout, rate, bsid, dependents, lfe, bsmod, object_count);
    let ftyp = bmff_box(*b"ftyp", b"M4A \0\0\0\0M4A isomiso6");
    let (mvhd, trak) = track_boxes(&dec3, Some((count, size, ftyp.len() as u64 + 16)));
    let moov = bmff_box(*b"moov", &[mvhd, trak].concat());
    let mut file = tempfile::NamedTempFile::new_in(
        output
            .parent()
            .ok_or_else(|| io::Error::other("missing output directory"))?,
    )?;
    {
        let mut out = BufWriter::new(file.as_file_mut());
        out.write_all(&ftyp)?;
        out.write_all(&1u32.to_be_bytes())?;
        out.write_all(b"mdat")?;
        out.write_all(&(len + 16).to_be_bytes())?;
        let mut sample = vec![0; size as usize];
        for _ in 0..count {
            if cancelled.load(Ordering::Relaxed) {
                return Err(io::Error::other("cancelled"));
            }
            raw.read_exact(&mut sample)?;
            out.write_all(&sample)?;
        }
        out.write_all(&moov)?;
        out.flush()?;
    }
    file.as_file().sync_all()?;
    file.persist_noclobber(output)?;
    Ok(())
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

    let mut sample = vec![0; size as usize];
    for sequence in 1..=count {
        raw.read_exact(&mut sample)?;
        if sequence == 1 {
            output.write_all(&init_segment(layout, rate, sample[4] & 1 != 0))?;
        }
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
            let config = dec3_box(layout, 1664, false);
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
        let mut cursor = init_segment(layout, rate, true).len();
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
