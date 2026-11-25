// pattern: Imperative Shell
//! One callback of original synthetic ADM audio, followed by four cached drains.
use std::{fs, process::Command};

#[path = "support/synthetic_adm.rs"]
mod synthetic_adm;

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
