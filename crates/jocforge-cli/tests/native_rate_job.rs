// pattern: Imperative Shell
//! 比较同一个 384 kbps 作业从文件 CLI 和公共自定义入口产生的原生 AU，不解码音频。
use jocforge::encode_request::{
    FileEncodeOptions, FileRenderTiming, encode_staged_file_with_options,
};
use jocforge_application::meter_source_bridge::{CodecRateProfile, JocReconstructionLayout};
use std::{fs, process::Command, sync::atomic::AtomicBool};

fn short_adm() -> Vec<u8> {
    let mut wav = b"RIFF\0\0\0\0WAVE".to_vec();
    let mut chunk = |id: &[u8; 4], data: &[u8]| {
        wav.extend(id);
        wav.extend(u32::try_from(data.len()).unwrap().to_le_bytes());
        wav.extend(data);
        if data.len() % 2 != 0 {
            wav.push(0);
        }
    };
    let align = 38_u16 * 3;
    let format = [
        &1_u16.to_le_bytes()[..],
        &38_u16.to_le_bytes(),
        &48000_u32.to_le_bytes(),
        &(48000 * u32::from(align)).to_le_bytes(),
        &align.to_le_bytes(),
        &24_u16.to_le_bytes(),
    ]
    .concat();
    let mut pcm = vec![0; 1536 * 4 * usize::from(align)];
    for (i, frame) in pcm.chunks_exact_mut(usize::from(align)).enumerate() {
        frame[3 * 11 + 2] = if i % 48 < 24 { 8 } else { 248 };
    }
    chunk(b"fmt ", &format);
    chunk(b"data", &pcm);
    chunk(
        b"axml",
        include_bytes!(
            "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.xml"
        ),
    );
    chunk(
        b"chna",
        include_bytes!(
            "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_track_associations.bin"
        ),
    );
    chunk(
        b"dbmd",
        include_bytes!(
            "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.dbmd"
        ),
    );
    let size = u32::try_from(wav.len() - 8).unwrap();
    wav[4..8].copy_from_slice(&size.to_le_bytes());
    wav
}

#[test]
fn desktop_equivalent_custom_job_matches_explicit_native_cli_at_384() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.wav");
    let cli = dir.path().join("native.ec3");
    let public = dir.path().join("custom.mp4");
    fs::write(&input, short_adm()).unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .arg("encode")
        .arg(&input)
        .arg(&cli)
        .args([
            "--data-rate",
            "384",
            "--joc-layout",
            "idx3",
            "--joc-objects",
            "15",
            "--container",
            "raw",
            "--carriage",
            "skipfld",
            "--emdf-profile",
            "deployed",
            "--block-partition",
            "6",
        ])
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let options: FileEncodeOptions = serde_json::from_value(serde_json::json!({
        "joc_objects":"15", "surround_trim_5_1":"auto", "height_trim_5_1":"auto"
    }))
    .unwrap();
    encode_staged_file_with_options(
        &input,
        &public,
        CodecRateProfile::Kbps384,
        JocReconstructionLayout::ExistingSelectedFivePointOne,
        (&FileRenderTiming::default(), &options),
        &|_| {},
        &AtomicBool::new(false),
    )
    .unwrap();
    let stream = fs::read(public).unwrap();
    assert_eq!(stream, fs::read(cli).unwrap());
    assert!(!stream.is_empty());
    let frames = stream.chunks_exact(1536);
    assert!(frames.remainder().is_empty());
    for frame in frames {
        assert_eq!(&frame[..2], &[0x0b, 0x77]);
        assert_eq!(frame[2] >> 3, 0); // 独立子流 0
        assert_eq!((u16::from_be_bytes([frame[2], frame[3]]) & 2047) + 1, 768);
        assert_eq!(frame[4], 0x3f); // 48 kHz，6 块，3/2+LFE
        assert_eq!(frame[5] >> 3, 16);
    }
}
