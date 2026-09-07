// pattern: Imperative Shell

use std::process::Command;

use jocforge_core::{AdmPosition, JocProfile, package_deployed_compat_access_unit};

fn ffmpeg_core(bitrate: &str, label: &str) -> Vec<u8> {
    let path =
        std::env::temp_dir().join(format!("jocforge-eac3-{label}-{}.ec3", std::process::id()));
    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "anoisesrc=color=white:sample_rate=48000:amplitude=0.2",
            "-filter_complex",
            "[0:a]pan=5.1|c0=c0|c1=c0|c2=c0|c3=c0|c4=c0|c5=c0",
            "-t",
            "0.032",
            "-c:a",
            "eac3",
            "-b:a",
            bitrate,
            "-channel_coupling",
            "0",
            "-f",
            "eac3",
            "-y",
        ])
        .arg(&path)
        .status()
        .expect("ffmpeg starts");
    assert!(status.success());
    let bytes = std::fs::read(&path).expect("FFmpeg core output");
    std::fs::remove_file(path).expect("remove FFmpeg core output");
    bytes
}

fn fixture_scene() -> (Vec<Vec<f64>>, Vec<AdmPosition>) {
    let positions = vec![
        AdmPosition {
            azimuth: 0.0,
            elevation: 0.0,
            distance: 0.0,
        };
        15
    ];
    let matrix = (0..15)
        .map(|object| {
            (0..5)
                .map(|channel| if object % 5 == channel { 0.2 } else { 0.0 })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    (matrix, positions)
}

#[test]
fn deployed_carriage_pads_public_core_and_inserts_skipfld() {
    let frame = ffmpeg_core("896k", "896");
    assert_eq!(frame.len(), 3584);
    let (matrix, positions) = fixture_scene();
    let output =
        package_deployed_compat_access_unit(JocProfile::Idx0, 1, &matrix, &positions, &frame)
            .expect("deployed-compatible carriage");
    assert_eq!(output.len(), 4096);
    assert_eq!(&output[0..2], &[0x0b, 0x77]);
}

#[test]
fn deployed_carriage_fails_closed_when_native_1024_core_has_no_skip_capacity() {
    let frame = ffmpeg_core("1024k", "1024-capacity");
    assert_eq!(frame.len(), 4096);
    let (matrix, positions) = fixture_scene();
    let error =
        package_deployed_compat_access_unit(JocProfile::Idx0, 1, &matrix, &positions, &frame)
            .expect_err("native 1024 core should not silently truncate EMDF");
    assert!(error.contains("insufficient public skip-field capacity"));
}
