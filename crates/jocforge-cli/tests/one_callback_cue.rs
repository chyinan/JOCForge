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
