// pattern: Imperative Shell
//! One callback of original synthetic ADM audio, followed by four cached drains.
use std::{fs, process::Command};

#[path = "support/synthetic_adm.rs"]
mod synthetic_adm;

#[test]
fn object_numeric_xml_whitespace_preserves_the_bitstream() {
    fn cue_with_numbers(pad: &str) -> Vec<u8> {
        let cue = synthetic_adm::synthetic_cue(64);
        let mut wave = cue[..12].to_vec();
        let mut chunks = &cue[12..];
        while !chunks.is_empty() {
            let size = u32::from_le_bytes(chunks[4..8].try_into().unwrap()) as usize;
            let end = 8 + size + size % 2;
            if &chunks[..4] == b"axml" {
                let xml = std::str::from_utf8(&chunks[8..8 + size]).unwrap()
                    .replace(r#"<position coordinate="X">0</position>"#, &format!(r#"<position coordinate="X">{pad}0{pad}</position>"#))
                    .replace(r#"<jumpPosition interpolationLength="0">"#, &format!(r#"<width>{pad}0.125{pad}</width><depth>{pad}0.125{pad}</depth><height>{pad}0.125{pad}</height><diffuse>{pad}0{pad}</diffuse><jumpPosition interpolationLength="{pad}0{pad}">"#));
                wave.extend(b"axml");
                wave.extend(u32::try_from(xml.len()).unwrap().to_le_bytes());
                wave.extend(xml.as_bytes());
                if !xml.len().is_multiple_of(2) {
                    wave.push(0);
                }
            } else {
                wave.extend(&chunks[..end]);
            }
            chunks = &chunks[end..];
        }
        let riff_size = u32::try_from(wave.len() - 8).unwrap();
        wave[4..8].copy_from_slice(&riff_size.to_le_bytes());
        wave
    }

    let encode = |pad| {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("cue.wav");
        let output = dir.path().join("cue.ec3");
        fs::write(&input, cue_with_numbers(pad)).unwrap();
        let run = Command::new(env!("CARGO_BIN_EXE_jocforge"))
            .arg("encode")
            .arg(input)
            .arg(&output)
            .args(["--data-rate", "384"])
            .env("JOCFORGE_CACHE_DIR", dir.path().join("cache"))
            .output()
            .unwrap();
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        fs::read(output).unwrap()
    };
    let compact = encode("");
    assert_eq!(compact.len(), 3072);
    assert_eq!(encode("\r\n \t"), compact);
}

#[test]
fn first_cached_drain_may_still_be_a_priming_callback() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("cue.wav");
    fs::write(&input, synthetic_adm::synthetic_cue(32)).unwrap();
    let output = dir.path().join("cue.ec3");
    let run = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .arg("encode")
        .arg(input)
        .arg(&output)
        .args(["--data-rate", "384"])
        .env("JOCFORGE_CACHE_DIR", dir.path().join("cache"))
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let au = fs::read(output).unwrap();
    assert_eq!(au.len(), 1536); // One complete 32 ms access unit at 384 kbps.
    assert_eq!(&au[..2], &[0x0b, 0x77]);
    assert_eq!(
        usize::from(u16::from_be_bytes([au[2], au[3]]) & 0x7ff) + 1,
        au.len() / 2
    );
}

#[test]
fn classic_and_extensible_pcm24_encode_the_same_adm() {
    let encode = |wav: Vec<u8>| {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("cue.wav");
        let output = dir.path().join("cue.ec3");
        fs::write(&input, wav).unwrap();
        let run = Command::new(env!("CARGO_BIN_EXE_jocforge"))
            .arg("encode")
            .arg(input)
            .arg(&output)
            .args(["--data-rate", "384", "--joc-layout", "idx3"])
            .env("JOCFORGE_CACHE_DIR", dir.path().join("cache"))
            .output()
            .unwrap();
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        fs::read(output).unwrap()
    };
    let classic = encode(synthetic_adm::synthetic_cue(96));
    let extensible = encode(synthetic_adm::synthetic_cue_with_extensible_fmt(96, true));
    assert_eq!(classic.len(), 3 * 1536);
    assert_eq!(extensible, classic);
}

#[test]
fn cancelling_from_the_last_encoding_update_does_not_publish() {
    use jocforge::encode_request::{EncodePhase, EncodeRequestError, encode_staged_file};
    use jocforge_application::meter_source_bridge::{CodecRateProfile, JocReconstructionLayout};
    use std::sync::atomic::{AtomicBool, Ordering};

    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("cue.wav");
    let output = dir.path().join("cue.ec3");
    fs::write(&input, synthetic_adm::synthetic_cue(32)).unwrap();
    let cancelled = AtomicBool::new(false);

    let result = encode_staged_file(
        &input,
        &output,
        CodecRateProfile::Kbps384,
        JocReconstructionLayout::ExistingSelectedFivePointOne,
        &|phase, done, total| {
            if matches!(phase, EncodePhase::Encoding) && done == total {
                cancelled.store(true, Ordering::Relaxed);
            }
        },
        &cancelled,
    );

    assert!(cancelled.load(Ordering::Relaxed));
    assert!(
        matches!(result, Err(EncodeRequestError::Cancelled)),
        "{result:?}"
    );
    assert!(!output.exists());
}

#[test]
fn relative_loudness_cache_override_stays_under_the_working_directory() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("cue.wav");
    fs::write(&input, synthetic_adm::synthetic_cue(32)).unwrap();
    let fallback = dir.path().join("default-cache");
    let run = |name| {
        let output = dir.path().join(name);
        let result = Command::new(env!("CARGO_BIN_EXE_jocforge"))
            .current_dir(dir.path())
            .arg("encode")
            .arg(&input)
            .arg(&output)
            .args(["--data-rate", "384"])
            .env("JOCFORGE_CACHE_DIR", "./project-cache")
            .env("LOCALAPPDATA", &fallback)
            .env("HOME", &fallback)
            .env("XDG_CACHE_HOME", &fallback)
            .output()
            .unwrap();
        let log = String::from_utf8_lossy(&result.stderr).into_owned();
        assert!(result.status.success(), "{log}");
        (fs::read(output).unwrap(), log)
    };

    let (cold, _) = run("cold.ec3");
    let requested = dir.path().join("project-cache");
    assert!(
        fs::read_dir(&requested).is_ok_and(|entries| entries.flatten().any(|entry| entry
            .path()
            .is_dir()
            && entry
                .file_name()
                .to_string_lossy()
                .starts_with("adm-loudness-"))),
        "explicit cache root was not used: {}",
        requested.display()
    );
    let (warm, log) = run("warm.ec3");
    assert!(log.contains("Loudness cache hit"), "{log}");
    assert_eq!(cold, warm);
    assert!(
        !fallback.exists(),
        "the default cache must remain untouched"
    );
}

#[test]
fn xml_downmix_reaches_both_ac3_core_paths() {
    let vectors: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    for args in [
        &["--joc-layout", "flat7x"][..],
        &["--joc-layout", "idx0", "--carriage", "legacy-ac3"][..],
    ] {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out");
        fs::create_dir(&out).unwrap();
        fs::write(
            dir.path().join("input.wav"),
            synthetic_adm::synthetic_cue(64),
        )
        .unwrap();
        let xml_path = |p: &std::path::Path| {
            p.to_str()
                .unwrap()
                .replace('&', "&amp;")
                .replace('<', "&lt;")
        };
        let job = dir.path().join("job.xml");
        let xml = vectors["config_cases"][0]["input_xml"]
            .as_str()
            .unwrap()
            .replacen(
                "<path>fixture</path>",
                &format!("<path>{}</path>", xml_path(dir.path())),
                1,
            )
            .replace(
                "<path>fixture</path>",
                &format!("<path>{}</path>", xml_path(&out)),
            )
            .replace("out.ec3", "mix.candidate.ec3")
            .replace("<data_rate>1024", "<data_rate>640")
            .replace(
                "<preferred_downmix_mode>loro",
                "<preferred_downmix_mode>ltrt-pl2",
            )
            .replace("<ltrt_center_mix_level>-3", "<ltrt_center_mix_level>+3")
            .replace(
                "<ltrt_surround_mix_level>-3",
                "<ltrt_surround_mix_level>-4.5",
            )
            .replace("<loro_center_mix_level>-3", "<loro_center_mix_level>-1.5")
            .replace(
                "<loro_surround_mix_level>-3",
                "<loro_surround_mix_level>-inf",
            );
        fs::write(&job, xml).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_jocforge"))
            .args(["encode", "--job-xml"])
            .arg(job)
            .args(args)
            .env("JOCFORGE_CACHE_DIR", dir.path().join("cache"))
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let stream = fs::read(out.join("mix.candidate.ec3")).unwrap();
        assert_eq!(stream.len(), 2 * 5120);
        for au in stream.chunks_exact(5120) {
            assert_eq!(&au[..2], &[0x0b, 0x77]);
            assert_eq!(au[5] >> 3, 6); // alternate AC-3 BSI
            let mut pos = if au[7] & 4 == 0 { 62 } else { 70 }; // after optional compr
            let mut bits = |n: usize| -> u8 {
                let value =
                    (pos..pos + n).fold(0, |v, i| (v << 1) | ((au[i / 8] >> (7 - i % 8)) & 1));
                pos += n;
                value
            };
            assert_eq!(bits(5), 0b00111); // no langcode/audprod; copyright, original, xbsi1
            // Annex D order: dmixmod, Lt/Rt center and surround, Lo/Ro center and surround.
            assert_eq!(
                [bits(2), bits(3), bits(3), bits(3), bits(3)],
                [3, 0, 5, 3, 7]
            );
        }
    }
}

#[test]
fn leqa_measures_a_burst_in_the_last_half_block() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();
    let xml: serde_json::Value = serde_json::from_str(include_str!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    let xml = xml["config_cases"][0]["input_xml"]
        .as_str()
        .unwrap()
        .replace(
            "<path>fixture</path>",
            &format!("<path>{}</path>", path.display()),
        )
        .replace(">1770-4<", ">LeqA<")
        .replace(">1024<", ">384<");
    let job = path.join("job.xml");
    let dialnorm = |ms: usize, source_start: usize| {
        let output = format!("out-{ms}-{source_start}.ec3");
        fs::write(&job, xml.replace("out.ec3", &output)).unwrap();
        let mut cue = synthetic_adm::synthetic_cue(ms);
        // The original fixture's final chunk is interleaved PCM24, 11 channels.
        let data_offset = cue.len() - ms * 48 * 33;
        assert_eq!(&cue[data_offset - 8..data_offset - 4], b"data");
        let pcm = &mut cue[data_offset..];
        pcm.fill(0);
        for sample in 0..512 {
            let pos = (source_start + sample) * 33;
            let pulse =
                (3_100_000.0 * (std::f64::consts::TAU * sample as f64 / 48.0).sin()).round() as i32;
            pcm[pos..pos + 3].copy_from_slice(&pulse.to_le_bytes()[..3]);
        }
        fs::write(path.join("input.wav"), cue).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_jocforge"))
            .arg("encode")
            .arg("--job-xml")
            .arg(&job)
            .env("JOCFORGE_CACHE_DIR", path.join("cache"))
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let ec3 = fs::read(path.join(output)).unwrap();
        assert_eq!(&ec3[..2], &[0x0b, 0x77]);
        ((ec3[5] & 7) << 2) | (ec3[6] >> 6)
    };
    let early = dialnorm(32, 0);
    let late = dialnorm(32, 1024);
    assert!(late < 31, "terminal tone was measured as silence");
    assert_eq!(early, late);
    // The extra millisecond is programme silence; the rest of its callback is not.
    assert_eq!(dialnorm(33, 1024), late);
    // Adding an equal duration of digital silence halves the mean energy.
    assert_eq!(dialnorm(64, 1024), late + 3);
}
