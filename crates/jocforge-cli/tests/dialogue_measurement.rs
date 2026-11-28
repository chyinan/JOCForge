// pattern: Imperative Shell
//! Checks the normal CLI loudness selection and the dialnorm carried by each E-AC-3 frame.
use std::{fs, path::Path, process::Command};

#[path = "support/synthetic_adm.rs"]
mod synthetic_adm;

const VOICE: &[u8] =
    include_bytes!("../../jocforge-domain/tests/fixtures/dialogue/synthetic_speech.u8");

fn speech_with_surround_tone(
    mut wav: Vec<u8>,
    centre_gain: f64,
    centre_masker_peak: f64,
) -> Vec<u8> {
    let parsed = jocforge_io::pcm::riff_pcm24::parse_selected_riff_pcm24(&wav, None).unwrap();
    let offset = usize::try_from(parsed.data_offset()).unwrap();
    for (i, row) in wav[offset..].chunks_exact_mut(33).enumerate() {
        row.fill(0);
        // Centre speech and a louder independent left surround make the selected levels distinct.
        let voice = (i32::from(VOICE[(i / 6) % VOICE.len()]) - 128) << 16;
        let voice = (f64::from(voice) * centre_gain) as i32;
        let surround =
            (2_000_000.0 * (std::f64::consts::TAU * 770.0 * i as f64 / 48_000.0).sin()) as i32;
        let masker = (centre_masker_peak
            * (std::f64::consts::TAU * 211.0 * i as f64 / 48_000.0).sin())
            as i32;
        row[6..9].copy_from_slice(&(voice + masker).to_le_bytes()[..3]);
        row[12..15].copy_from_slice(&surround.to_le_bytes()[..3]);
    }
    wav
}

fn encode(root: &Path, args: &[&Path]) -> String {
    let run = Command::new(env!("CARGO_BIN_EXE_jocforge"))
        .arg("encode")
        .args(args)
        .env("JOCFORGE_CACHE_DIR", root.join("cache"))
        .output()
        .unwrap();
    let report = String::from_utf8_lossy(&run.stderr);
    assert!(run.status.success(), "{report}");
    report.into_owned()
}

fn field<'a>(report: &'a str, name: &str) -> &'a str {
    report
        .split_whitespace()
        .find_map(|word| word.strip_prefix(name))
        .unwrap_or_else(|| panic!("missing {name} in {report}"))
}

fn dialogue_loudness(report: &str) -> f32 {
    assert_eq!(field(report, "selection_reason="), "dialogue_threshold_met");
    field(report, "dialogue_loudness=")
        .strip_prefix("Some(")
        .unwrap()
        .trim_end_matches(')')
        .parse()
        .unwrap()
}

fn assert_report_percentage(report: &str, accepted: f32, total: f32) {
    let percentage: f32 = field(report, "speech_percent=")
        .strip_prefix("Some(")
        .unwrap()
        .trim_end_matches(')')
        .parse()
        .unwrap();
    assert_eq!(percentage.to_bits(), (100.0 * accepted / total).to_bits());
}

fn measurement_job_xml(root: &Path) -> String {
    let jobs: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
    ))
    .unwrap();
    jobs["config_cases"][0]["input_xml"]
        .as_str()
        .unwrap()
        .replace(
            ">fixture<",
            &format!(
                ">{}<",
                root.display()
                    .to_string()
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
            ),
        )
}

fn dialnorm_frames(path: &Path, frames: usize, loudness: f32) {
    let stream = fs::read(path).unwrap();
    let mut rest = stream.as_slice();
    let mut count = 0;
    let expected = (-loudness).round().clamp(1.0, 31.0) as i32;
    while !rest.is_empty() {
        assert_eq!(&rest[..2], &[0x0b, 0x77]);
        assert_eq!(rest[2] >> 3, 0, "expected independent substream zero");
        assert_eq!(rest[4] >> 4, 3, "48 kHz, six blocks per frame");
        let words_minus_one = usize::from(u16::from_be_bytes([rest[2], rest[3]]) & 0x7ff);
        let dialnorm = ((rest[5] & 7) << 2) | (rest[6] >> 6);
        assert_eq!(
            i32::from(dialnorm),
            expected,
            "frame {count} selected {loudness} LUFS"
        );
        rest = &rest[2 * (words_minus_one + 1)..];
        count += 1;
    }
    assert_eq!(count, frames);
}

#[test]
fn speech_duration_and_explicit_dialogue_setting_select_each_frames_dialnorm() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let input = root.join("input.wav");
    let output = root.join("measured.ec3");

    let short = synthetic_adm::synthetic_cue_samples(96_768);
    fs::write(&input, speech_with_surround_tone(short, 1.0, 0.0)).unwrap();
    let short = encode(root, &[&input, &output]);
    assert_eq!(field(&short, "selection_reason="), "no_speech");
    assert_eq!(field(&short, "speech_percent="), "Some(0.0)");
    let programme: f32 = field(&short, "programme_integrated=").parse().unwrap();
    dialnorm_frames(&output, 63, programme);

    // One additional source sample starts the 64th access unit.
    let output = root.join("padded.ec3");
    let padded = synthetic_adm::synthetic_cue_samples(96_769);
    fs::write(&input, speech_with_surround_tone(padded, 1.0, 0.0)).unwrap();
    let padded = encode(root, &[&input, &output]);
    assert_report_percentage(&padded, 53.0, 148.0);
    dialnorm_frames(&output, 64, dialogue_loudness(&padded));

    let output = root.join("dialogue.ec3");
    let full = synthetic_adm::synthetic_cue(2_048);
    fs::write(&input, speech_with_surround_tone(full, 1.0, 0.0)).unwrap();
    let normal = encode(root, &[&input, &output]);
    assert_report_percentage(&normal, 53.0, 148.0);
    let dialogue = dialogue_loudness(&normal);
    let programme: f32 = field(&normal, "programme_integrated=").parse().unwrap();
    assert!(programme - dialogue > 3.0, "{normal}");
    dialnorm_frames(&output, 64, dialogue);

    // The checked-in XML fixture explicitly disables dialogue intelligence and uses out.ec3.
    let job = root.join("programme.xml");
    fs::write(&job, measurement_job_xml(root)).unwrap();
    let explicit = encode(root, &[Path::new("--job-xml"), &job]);
    assert_eq!(field(&explicit, "selection_reason="), "dialogue_disabled");
    assert_eq!(
        field(&explicit, "programme_integrated="),
        field(&normal, "programme_integrated=")
    );
    dialnorm_frames(&root.join("out.ec3"), 64, programme);
}

#[test]
fn leqa_and_bs1770_select_dialogue_after_a_partial_final_frame() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let input = root.join("input.wav");
    // One sample after 2.048 s starts the 65th access unit. A -48.16 dBFS
    // peak tone shares the speech channel while the independent surround stays active.
    let wav = speech_with_surround_tone(synthetic_adm::synthetic_cue_samples(98_305), 1.0, 32768.0);
    fs::write(&input, wav).unwrap();
    let k_output = root.join("weighted.ec3");
    let k_report = encode(root, &[&input, &k_output]);
    assert_report_percentage(&k_report, 54.0, 149.0);
    let k_level = dialogue_loudness(&k_report);
    dialnorm_frames(&k_output, 65, k_level);

    let job = root.join("leqa.xml");
    let xml = measurement_job_xml(root)
        .replace("<metering_mode>1770-4<", "<metering_mode>LeqA<")
        .replace(
            "<dialogue_intelligence>false<",
            "<dialogue_intelligence>true<",
        )
        .replace("<speech_threshold>100<", "<speech_threshold>15<");
    fs::write(&job, xml).unwrap();
    let a_report = encode(root, &[Path::new("--job-xml"), &job]);
    assert_report_percentage(&a_report, 54.0, 149.0);
    let a_level = dialogue_loudness(&a_report);
    // Independent IEC A-weighting of the physical centre signal gives -24.756 dB
    // before the incomplete final access unit is padded.
    assert!((a_level + 24.756).abs() < 0.1, "{a_report}");
    assert!(k_level - a_level > 6.0, "{k_report}\n{a_report}");
    dialnorm_frames(&root.join("out.ec3"), 65, a_level);
}

#[test]
fn quiet_speech_uses_programme_dialnorm_with_a_zero_percentage_threshold() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let wav = speech_with_surround_tone(synthetic_adm::synthetic_cue(2_048), 1.0 / 256.0, 0.0);
    fs::write(root.join("input.wav"), wav).unwrap();
    let job = root.join("quiet-speech.xml");
    let xml = measurement_job_xml(root)
        .replace(
            "<dialogue_intelligence>false<",
            "<dialogue_intelligence>true<",
        )
        .replace("<speech_threshold>100<", "<speech_threshold>0<");
    fs::write(&job, xml).unwrap();
    let report = encode(root, &[Path::new("--job-xml"), &job]);
    let read_option = |name| -> f32 {
        field(&report, name)
            .strip_prefix("Some(")
            .unwrap()
            .trim_end_matches(')')
            .parse()
            .unwrap()
    };
    assert!(read_option("speech_percent=") > 0.0, "{report}");
    assert!(read_option("dialogue_loudness=") < -60.0, "{report}");
    assert_eq!(field(&report, "gating="), "LEVEL", "{report}");
    let programme: f32 = field(&report, "programme_integrated=").parse().unwrap();
    dialnorm_frames(&root.join("out.ec3"), 64, programme);
}

#[test]
fn manual_dialnorm_uses_the_requested_code_without_changing_measurement() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let input = speech_with_surround_tone(synthetic_adm::synthetic_cue_samples(98_304), 1.0, 0.0);
    fs::write(root.join("input.wav"), input).unwrap();
    let job = root.join("job.xml");
    let mut measured = None;
    for (index, setting) in [0, -1, -31, -25].into_iter().enumerate() {
        let name = format!("case-{index}.ec3");
        let xml = measurement_job_xml(root).replace("out.ec3", &name).replace(
            "</encode_to_atmos_ddp>",
            &format!("<custom_dialnorm>{setting}</custom_dialnorm></encode_to_atmos_ddp>"),
        );
        fs::write(&job, xml).unwrap();
        let report = encode(root, &[Path::new("--job-xml"), &job]);
        let level = field(&report, "programme_integrated=")
            .parse::<f32>()
            .unwrap();
        assert_eq!(*measured.get_or_insert(level), level);
        dialnorm_frames(
            &root.join(name),
            64,
            if setting == 0 { level } else { setting as f32 },
        );
    }
}
