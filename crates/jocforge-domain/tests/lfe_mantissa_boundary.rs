// pattern: Functional Core
use jocforge_domain::codec::mantissa::{
    LfeGateError, LfeGateProfile, LfeMantissaError, LfeMantissaMachine, MantissaBoundaryError,
    MantissaDownstreamOutcome, clip_lfe_main_coefficients, clip_primary_coefficients,
    selected_lfe_gate,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const SOURCE_SPEC_SHA256: &str = "9aff19192a8b5cba7db8bec26b05106edf214b4149a5f1c9d4be040691c80951";
const SOURCE_VECTORS_SHA256: &str =
    "70d63104353ff6f721ae521be4469a9dda15e48d27e22030b3c46f0220c9b6be";
const SOURCE_VERIFICATION_SHA256: &str =
    "deb5d1037aef990b39596af96ad79bab3085d5fe1623da8b6de7111d87f9da20";
const SOURCE_REVIEW_SHA256: &str =
    "fddeabae5f51da3d78a5e3659d445d691e527dc1a264ebeece1a5b119c57e5c8";
const SOURCE_PARAMETER_SHA256: &str =
    "310afbab35e19096e19433a40fee3a69271b18e8b70f6977380a6e7cba590962";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixtures() -> PathBuf {
    crate_root().join("tests/fixtures/lfe_gate_mantissa")
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn vectors() -> Value {
    serde_json::from_slice(&std::fs::read(fixtures().join("input-vectors.json")).unwrap()).unwrap()
}

#[test]
fn lfe_gate_mantissa_material_is_bound_to_the_approved_package() {
    let manifest_bytes =
        std::fs::read(crate_root().join("tests/fixtures/f01e_provider/MANIFEST.json")).unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    assert_eq!(sha256(&manifest_bytes), PACKAGE_MANIFEST_SHA256);

    for (path, bytes, expected_sha256, local_name) in [
        (
            "specs/09-lfe-gate-mantissa-boundaries.md",
            4_786,
            SOURCE_SPEC_SHA256,
            "source-spec.md",
        ),
        (
            "test-vectors/LFE-GATE-MANTISSA-INPUT-VECTORS.json",
            4_979,
            SOURCE_VECTORS_SHA256,
            "input-vectors.json",
        ),
        (
            "test-vectors/LFE-GATE-MANTISSA-VERIFICATION-RESULT.json",
            4_930,
            SOURCE_VERIFICATION_SHA256,
            "verification-result.json",
        ),
        (
            "test-vectors/LFE-GATE-MANTISSA-LOCAL-REVIEW-STATUS.json",
            2_019,
            SOURCE_REVIEW_SHA256,
            "local-review-status.json",
        ),
        (
            "parameters/lfe-gate-mantissa-boundaries-v1.json",
            747,
            SOURCE_PARAMETER_SHA256,
            "parameters.json",
        ),
    ] {
        let entry = manifest["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["file"] == path)
            .unwrap();
        let local_bytes = std::fs::read(fixtures().join(local_name)).unwrap();
        assert_eq!(entry["bytes"], bytes);
        assert_eq!(entry["sha256"], expected_sha256);
        assert_eq!(u64::try_from(local_bytes.len()).unwrap(), bytes);
        assert_eq!(sha256(&local_bytes), expected_sha256);
    }
    let review: Value = serde_json::from_slice(
        &std::fs::read(fixtures().join("local-review-status.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        review["decision"]["local_selected_Q06_IMPLEMENTATION_READY"],
        "YES"
    );
    assert_eq!(
        review["decision"]["whole_package_IMPLEMENTATION_READY"],
        "NO"
    );
}

#[test]
fn strict_clip_and_raw_exponent_match_all_boundary_vectors() {
    let vectors = vectors();
    for case in vectors["coefficient_cases"].as_array().unwrap() {
        let input = word(case["input_u32"].as_str().unwrap());
        let expected_word = word(case["expected_clipped_u32"].as_str().unwrap());
        let expected_exponent = u8::try_from(case["expected_exponent"].as_u64().unwrap()).unwrap();
        let row = clip_primary_coefficients(&[input]).unwrap();
        assert_eq!(row.clipped_words(), &[expected_word], "{}", case["id"]);
        assert_eq!(row.raw_exponents(), &[expected_exponent], "{}", case["id"]);
        assert_eq!(
            row.clipped_flags(),
            &[case["clipped"].as_bool().unwrap()],
            "{}",
            case["id"]
        );

        let lfe_row = clip_lfe_main_coefficients(&[input; 7]).unwrap();
        assert_eq!(lfe_row.clipped_words()[0], expected_word, "{}", case["id"]);
        assert_eq!(
            lfe_row.raw_exponents()[0],
            expected_exponent,
            "{}",
            case["id"]
        );
        assert_eq!(
            lfe_row.clipped_flags()[0],
            case["clipped"].as_bool().unwrap()
        );
    }
}

#[test]
fn coefficient_shape_and_nonfinite_rejections_match_the_contract() {
    let vectors = vectors();
    for case in vectors["rejection_cases"].as_array().unwrap() {
        if case["kind"] != "coefficient" {
            continue;
        }
        let words = case["input_u32"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| word(value.as_str().unwrap()))
            .collect::<Vec<_>>();
        let result = match case["route"].as_str().unwrap() {
            "PRIMARY" => clip_primary_coefficients(&words),
            "LFE" => clip_lfe_main_coefficients(&words),
            route => panic!("unmapped coefficient route {route}"),
        };
        assert_eq!(result, Err(MantissaBoundaryError::Input), "{}", case["id"]);
    }
}

#[test]
fn selected_profile_gate_is_zero_and_other_profiles_are_not_admitted() {
    let vectors = vectors();
    let selected = vectors["gate_producer_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "selected")
        .unwrap();
    let input = &selected["input"];
    let profile = LfeGateProfile {
        sample_rate_hz: 48_000,
        bit_rate_kbps: 1_024,
        acmod: 7,
        blocks: u8::try_from(input["blocks"].as_u64().unwrap()).unwrap(),
        aht_enabled: input["aht_enabled"].as_bool().unwrap(),
    };
    assert_eq!(
        selected_lfe_gate(profile),
        Ok(i16::try_from(selected["expected_gate"].as_i64().unwrap()).unwrap())
    );

    let mut unsupported = profile;
    unsupported.aht_enabled = true;
    assert_eq!(
        selected_lfe_gate(unsupported),
        Err(LfeGateError::UnsupportedProfile)
    );
    unsupported = profile;
    unsupported.blocks = 5;
    assert_eq!(
        selected_lfe_gate(unsupported),
        Err(LfeGateError::UnsupportedProfile)
    );
    unsupported = profile;
    unsupported.sample_rate_hz = 44_100;
    assert_eq!(
        selected_lfe_gate(unsupported),
        Err(LfeGateError::UnsupportedProfile)
    );
}

#[test]
fn primary_scan_admits_zero_through_253_and_lfe_is_exactly_seven() {
    assert!(
        clip_primary_coefficients(&[])
            .unwrap()
            .raw_exponents()
            .is_empty()
    );
    assert_eq!(
        clip_primary_coefficients(&[0; 253])
            .unwrap()
            .raw_exponents()
            .len(),
        253
    );
    assert_eq!(
        clip_primary_coefficients(&[0; 254]),
        Err(MantissaBoundaryError::Input)
    );
    assert!(clip_lfe_main_coefficients(&[0; 7]).is_ok());
    assert_eq!(
        clip_lfe_main_coefficients(&[0; 6]),
        Err(MantissaBoundaryError::Input)
    );
    assert_eq!(
        clip_lfe_main_coefficients(&[0; 8]),
        Err(MantissaBoundaryError::Input)
    );
}

#[test]
fn lfe_staged_row_and_exponents_commit_or_retire_as_one_unit() {
    let vectors = vectors();
    let selected = LfeGateProfile {
        sample_rate_hz: 48_000,
        bit_rate_kbps: 1_024,
        acmod: 7,
        blocks: 6,
        aht_enabled: false,
    };

    for case in vectors["transaction_cases"].as_array().unwrap() {
        let mut machine = LfeMantissaMachine::new(selected).unwrap();
        let words = case["initial_words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| word(value.as_str().unwrap()))
            .collect::<Vec<_>>();
        let staged = machine.stage_lfe_coefficients(&words).unwrap();
        assert_eq!(
            staged.row().clipped_words(),
            case["expected_clipped_words"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| word(value.as_str().unwrap()))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            staged.row().raw_exponents(),
            case["expected_raw_exponents"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| u8::try_from(value.as_u64().unwrap()).unwrap())
                .collect::<Vec<_>>()
        );
        let outcome = if case["downstream_status"] == "PASS" {
            MantissaDownstreamOutcome::Accepted
        } else {
            MantissaDownstreamOutcome::Failed
        };
        let receipt = machine.finish_source(staged, outcome);
        assert_eq!(receipt.committed(), case["expected_commit_count"] == 1);
        assert_eq!(machine.retired(), case["expected_retired"]);
        assert_eq!(
            machine.committed_row().is_some(),
            case["expected_commit_count"] == 1
        );
    }
}

#[test]
fn retired_lfe_stage_rejects_more_work_and_reset_retries_from_cold() {
    let profile = LfeGateProfile {
        sample_rate_hz: 48_000,
        bit_rate_kbps: 1_024,
        acmod: 7,
        blocks: 6,
        aht_enabled: false,
    };
    let input = [
        0x3f80_0001,
        0xbf80_0001,
        0,
        1,
        0x3f00_0000,
        0x3eff_ffff,
        0x7f7f_ffff,
    ];
    let mut machine = LfeMantissaMachine::new(profile).unwrap();
    let staged = machine.stage_lfe_coefficients(&input).unwrap();
    machine.finish_source(staged, MantissaDownstreamOutcome::Failed);
    assert_eq!(
        machine.stage_lfe_coefficients(&input),
        Err(LfeMantissaError::Retired)
    );

    machine.reset();
    assert_eq!(machine.gate(), 0);
    assert!(!machine.retired());
    assert!(machine.committed_row().is_none());
    let staged = machine.stage_lfe_coefficients(&input).unwrap();
    let receipt = machine.finish_source(staged, MantissaDownstreamOutcome::Accepted);
    assert!(receipt.committed());
    assert!(!machine.retired());
}

#[test]
fn later_downstream_failure_preserves_the_previous_committed_bank() {
    let profile = LfeGateProfile {
        sample_rate_hz: 48_000,
        bit_rate_kbps: 1_024,
        acmod: 7,
        blocks: 6,
        aht_enabled: false,
    };
    let mut machine = LfeMantissaMachine::new(profile).unwrap();
    let first = [0; 7];
    let staged = machine.stage_lfe_coefficients(&first).unwrap();
    machine.finish_source(staged, MantissaDownstreamOutcome::Accepted);
    let before = machine.committed_row().unwrap().clone();

    let second = [0x3f80_0001; 7];
    let staged = machine.stage_lfe_coefficients(&second).unwrap();
    let receipt = machine.finish_source(staged, MantissaDownstreamOutcome::Failed);
    assert!(!receipt.committed());
    assert!(machine.retired());
    assert_eq!(machine.committed_row(), Some(&before));
}

fn word(hex: &str) -> u32 {
    u32::from_str_radix(hex, 16).unwrap()
}
