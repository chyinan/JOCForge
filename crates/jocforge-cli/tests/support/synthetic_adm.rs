// pattern: Imperative Shell
//! Original synthetic ADM shared by the CLI and WASM integration tests.
use std::fmt::Write;

fn chunk(dst: &mut Vec<u8>, name: [u8; 4], payload: &[u8]) {
    dst.extend(name);
    dst.extend(u32::try_from(payload.len()).unwrap().to_le_bytes());
    dst.extend(payload);
    if payload.len() % 2 == 1 {
        dst.push(0);
    }
}

/// Generates the team's original 11-track test cue at 48 kHz.
pub fn synthetic_cue(duration_ms: usize) -> Vec<u8> {
    synthetic_cue_samples(duration_ms * 48)
}

pub fn synthetic_cue_samples(samples: usize) -> Vec<u8> {
    assert!(samples < 60 * 48_000);
    // Five ADM decimal places give 25 time units for every 12 PCM samples.
    let duration = format!(
        "00:00:{:02}.{:05}",
        samples / 48_000,
        (samples % 48_000 * 25 + 6) / 12
    );
    let mut formats = String::new();
    let mut chna = [11u16.to_le_bytes(); 2].concat();
    for (kind, labels) in [
        (
            1,
            &["L", "R", "C", "LFE", "Ls", "Rs", "Lrs", "Rrs", "Ltm", "Rtm"][..],
        ),
        (3, &["Cue"][..]),
    ] {
        let typ = if kind == 1 {
            "DirectSpeakers"
        } else {
            "Objects"
        };
        let obj = if kind == 1 { "AO_1001" } else { "AO_100B" };
        let pack = format!("AP_{kind:04X}1001");
        let mut tracks = String::new();
        let mut channels = String::new();
        for (j, label) in labels.iter().enumerate() {
            let i = if kind == 1 {
                u16::try_from(j + 1).unwrap()
            } else {
                11
            };
            let uid = format!("ATU_{i:08X}");
            let id = format!("{kind:04X}{:04X}", 4097 + j);
            let (track, stream, channel) = (
                format!("AT_{id}_01"),
                format!("AS_{id}"),
                format!("AC_{id}"),
            );
            write!(tracks, "<audioTrackUIDRef>{uid}</audioTrackUIDRef>").unwrap();
            write!(
                channels,
                "<audioChannelFormatIDRef>{channel}</audioChannelFormatIDRef>"
            )
            .unwrap();
            let position = if kind == 1 {
                format!("<speakerLabel>RC_{label}</speakerLabel>")
            } else {
                "<jumpPosition interpolationLength=\"0\">1</jumpPosition>".into()
            };
            write!(formats, r#"
              <audioChannelFormat audioChannelFormatID="{channel}" audioChannelFormatName="{label}" typeDefinition="{typ}" typeLabel="{kind:04X}">
                <audioBlockFormat audioBlockFormatID="AB_{id}_00000001" rtime="00:00:00.00000" duration="{duration}">
                    <cartesian>1</cartesian><position coordinate="X">0</position><position coordinate="Y">1</position><position coordinate="Z">0</position>{position}
                </audioBlockFormat>
              </audioChannelFormat>
              <audioStreamFormat audioStreamFormatID="{stream}" audioStreamFormatName="{label}" formatLabel="0001" formatDefinition="PCM"><audioChannelFormatIDRef>{channel}</audioChannelFormatIDRef><audioTrackFormatIDRef>{track}</audioTrackFormatIDRef></audioStreamFormat>
              <audioTrackFormat audioTrackFormatID="{track}" audioTrackFormatName="{label}" formatLabel="0001" formatDefinition="PCM"><audioStreamFormatIDRef>{stream}</audioStreamFormatIDRef></audioTrackFormat>
              <audioTrackUID UID="{uid}" sampleRate="48000" bitDepth="24"><audioTrackFormatIDRef>{track}</audioTrackFormatIDRef><audioPackFormatIDRef>{pack}</audioPackFormatIDRef></audioTrackUID>"#).unwrap();
            chna.extend(i.to_le_bytes());
            chna.extend(uid.as_bytes());
            chna.extend(track.as_bytes());
            chna.extend(pack.as_bytes());
            chna.push(0);
        }
        write!(formats,r#"<audioObject audioObjectID="{obj}" audioObjectName="{typ}" start="00:00:00.00000" duration="{duration}"><audioPackFormatIDRef>{pack}</audioPackFormatIDRef>{tracks}</audioObject><audioPackFormat audioPackFormatID="{pack}" audioPackFormatName="{typ}" typeDefinition="{typ}" typeLabel="{kind:04X}">{channels}</audioPackFormat>"#).unwrap();
    }
    let axml = format!(
        r#"<ebuCoreMain xmlns="urn:ebu:metadata-schema:ebuCore_2017"><coreMetadata><format><audioFormatExtended>
        <audioProgramme audioProgrammeID="APR_1001" audioProgrammeName="Cue"><audioContentIDRef>ACO_1001</audioContentIDRef></audioProgramme>
        <audioContent audioContentID="ACO_1001" audioContentName="Cue"><audioObjectIDRef>AO_1001</audioObjectIDRef><audioObjectIDRef>AO_100B</audioObjectIDRef></audioContent>
        {formats}</audioFormatExtended></format></coreMetadata></ebuCoreMain>"#
    );
    let mut dbmd = vec![0; 4];
    let mut tracks = vec![0; 142 + 2 * 11];
    tracks[..4].copy_from_slice(&0xf872_6fbdu32.to_le_bytes());
    tracks[4..6].copy_from_slice(&11u16.to_le_bytes());
    for (id, data) in [(9, vec![0; 248]), (10, tracks)] {
        dbmd.push(id);
        let len = u16::try_from(data.len()).unwrap();
        dbmd.extend(len.to_le_bytes());
        let checksum = data
            .iter()
            .copied()
            .fold(len.to_le_bytes()[0], u8::wrapping_add)
            .wrapping_neg();
        dbmd.extend(data);
        dbmd.push(checksum);
    }
    dbmd.push(0);
    let mut wav = b"RIFF\0\0\0\0WAVE".to_vec();
    let fmt = [
        &1u16.to_le_bytes()[..],
        &11u16.to_le_bytes(),
        &48000u32.to_le_bytes(),
        &(48000u32 * 33).to_le_bytes(),
        &33u16.to_le_bytes(),
        &24u16.to_le_bytes(),
    ]
    .concat();
    chunk(&mut wav, *b"fmt ", &fmt);
    chunk(&mut wav, *b"axml", axml.as_bytes());
    chunk(&mut wav, *b"chna", &chna);
    chunk(&mut wav, *b"dbmd", &dbmd);
    let mut pcm = vec![0; samples * 33];
    for (i, sample) in pcm.chunks_exact_mut(33).enumerate() {
        sample[32] = if i % 48 < 24 { 8 } else { 248 };
    }
    chunk(&mut wav, *b"data", &pcm);
    let size = u32::try_from(wav.len() - 8).unwrap();
    wav[4..8].copy_from_slice(&size.to_le_bytes());
    wav
}
