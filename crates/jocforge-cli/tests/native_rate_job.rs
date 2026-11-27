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

#[test]
fn missing_output_directory_is_not_reported_as_a_relative_path() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.wav");
    fs::write(&input, short_adm()).unwrap();
    let parent = dir.path().join("not-yet-created");
    let output = parent.join("mix.ec3");
    let run = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .arg("encode")
        .arg(&input)
        .arg(&output)
        .output()
        .unwrap();
    let stderr = String::from_utf8(run.stderr).unwrap();
    assert_eq!(run.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.contains(&format!(
            "could not resolve output directory {}:",
            parent.display()
        )),
        "{stderr}"
    );
    assert!(!parent.exists());

    let relative = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .current_dir(dir.path())
        .arg("encode")
        .arg(input)
        .arg("relative.ec3")
        .output()
        .unwrap();
    assert_eq!(relative.status.code(), Some(2));
    assert!(
        String::from_utf8(relative.stderr)
            .unwrap()
            .contains("requires absolute input and output file paths")
    );
    assert!(!dir.path().join("relative.ec3").exists());
}

#[test]
fn missing_absolute_adm_reports_the_referenced_file_and_io_error() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.wav");
    let output = dir.path().join("out.ec3");
    let job = dir.path().join("job.xml");
    let vectors: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let local_path = dir
        .path()
        .to_str()
        .unwrap()
        .replace('&', "&amp;")
        .replace('<', "&lt;");
    let xml = vectors["config_cases"][0]["input_xml"]
        .as_str()
        .unwrap()
        .replace(
            "<path>fixture</path>",
            &format!("<path>{local_path}</path>"),
        );
    fs::write(&job, xml).unwrap();

    let error = fs::canonicalize(&input).unwrap_err();
    let expected = format!("could not resolve input file {}: {error}", input.display());
    for preflight_only in [true, false] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_jocforge"));
        command.args(["encode", "--job-xml"]).arg(&job);
        if preflight_only {
            command.arg("--preflight-only");
        }
        let run = command.output().unwrap();
        assert_eq!(run.status.code(), Some(2));
        let stderr = String::from_utf8(run.stderr).unwrap();
        assert!(stderr.contains(&expected), "{stderr}");
        assert!(!output.exists());
    }

    let relative = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .current_dir(dir.path())
        .args(["encode", "input.wav"])
        .arg(&output)
        .output()
        .unwrap();
    assert_eq!(relative.status.code(), Some(2));
    assert!(
        String::from_utf8(relative.stderr)
            .unwrap()
            .contains("requires absolute input and output file paths")
    );
    assert!(!output.exists());
}
