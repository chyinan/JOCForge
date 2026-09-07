// pattern: Functional Core

use jocforge_core::{
    EmdfProfile, JocProfile, d0_carrier_causality, package_i0_skip_field_access_unit,
    package_source_derived_access_unit, write_one_au_proof,
};

fn ffmpeg_i0_frame(label: &str) -> Vec<u8> {
    let path = std::env::temp_dir().join(format!(
        "jocforge-profile-test-{label}-{}.ec3",
        std::process::id(),
    ));
    let status = std::process::Command::new("ffmpeg")
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
            "896k",
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
    let frame = std::fs::read(&path).expect("FFmpeg I0 output");
    std::fs::remove_file(path).expect("remove FFmpeg I0 output");
    assert_eq!(frame.len(), 3584);
    frame
}

#[test]
fn writer_produces_a_complete_idx0_processing_unit() {
    let access_unit = write_one_au_proof(JocProfile::Idx0).expect("writer proof");

    assert_eq!(
        access_unit.len(),
        4096,
        "idx0 proof uses one 4096-byte I0 AU"
    );
    assert_eq!(&access_unit[0..2], &[0x0b, 0x77]);
    assert!(access_unit.windows(2).any(|window| window == [0x58, 0x38]));
}

#[test]
fn seven_input_profiles_use_i0_plus_2560_byte_d0_carriage() {
    let flat = jocforge_core::write_one_au_proof(JocProfile::Idx1).expect("idx1 proof");
    let height = jocforge_core::write_one_au_proof(JocProfile::Idx2).expect("idx2 proof");

    assert_eq!(flat.len(), 4096 + 2560);
    assert_eq!(height.len(), 4096 + 2560);
    assert_eq!(&flat[4096..4098], &[0x0b, 0x77]);
    assert_eq!(&height[4096..4098], &[0x0b, 0x77]);
}

#[test]
fn writer_accepts_the_actual_reduced_object_count() {
    let unit = 820.0 / 4096.0;
    let matrix = (0..3)
        .map(|object| {
            (0..5)
                .map(|channel| if object == channel { unit } else { 0.0 })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let positions = vec![
        jocforge_core::AdmPosition {
            azimuth: 0.0,
            elevation: 0.0,
            distance: 0.0,
        };
        3
    ];
    let i0 = vec![0x0b, 0x77]
        .into_iter()
        .chain(std::iter::repeat_n(0, 4094))
        .collect::<Vec<_>>();
    let d0_carriers = vec![vec![0.0; 1536], vec![0.0; 1536]];

    let access_unit = jocforge_core::package_source_derived_access_unit(
        JocProfile::Idx0,
        EmdfProfile::EtsiStrict,
        1,
        &matrix,
        &positions,
        &i0,
        &d0_carriers,
        0,
    )
    .expect("three-object JOC access unit");

    assert_eq!(access_unit.len(), 4096 + 2560);
}

#[test]
fn writer_encodes_general_coarse_matrix_symbols() {
    let unit = 820.0 / 4096.0;
    let matrix = vec![vec![2.0 * unit, -unit, 0.0, 0.0, 0.0]];
    let position = jocforge_core::AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    };
    let i0 = vec![0x0b, 0x77]
        .into_iter()
        .chain(std::iter::repeat_n(0, 4094))
        .collect::<Vec<_>>();
    let d0_carriers = vec![vec![0.0; 1536], vec![0.0; 1536]];

    let access_unit = jocforge_core::package_source_derived_access_unit(
        JocProfile::Idx0,
        EmdfProfile::EtsiStrict,
        1,
        &matrix,
        &[position],
        &i0,
        &d0_carriers,
        0,
    )
    .expect("general coarse matrix symbols");

    assert_eq!(access_unit.len(), 4096 + 2560);
}

#[test]
fn source_derived_d0_writer_changes_for_each_dependent_input() {
    for profile in [JocProfile::Idx0, JocProfile::Idx1, JocProfile::Idx2] {
        let measurements = d0_carrier_causality(profile).expect("D0 causality fixture");
        assert_eq!(measurements.len(), 2);
        assert!(
            measurements
                .iter()
                .all(|measurement| measurement.changed && measurement.changed_byte_count > 0)
        );
    }
}

#[test]
fn i0_skip_field_profile_changes_only_emdf_signaling() {
    let unit = 820.0 / 4096.0;
    let matrix = vec![vec![unit, 0.0, 0.0, 0.0, 0.0]];
    let positions = vec![jocforge_core::AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    }];
    let i0 = ffmpeg_i0_frame("skip");

    let strict = package_i0_skip_field_access_unit(
        JocProfile::Idx0,
        EmdfProfile::EtsiStrict,
        1,
        &matrix,
        &positions,
        &i0,
    )
    .expect("strict I0 skip-field AU");
    let deployed = package_i0_skip_field_access_unit(
        JocProfile::Idx0,
        EmdfProfile::DeployedCompat,
        1,
        &matrix,
        &positions,
        &i0,
    )
    .expect("deployed I0 skip-field AU");

    assert_eq!(strict.len(), 4096);
    assert_eq!(deployed.len(), strict.len());
    assert_ne!(deployed, strict);
}

#[test]
fn emdf_profile_is_orthogonal_to_source_derived_carriage() {
    let unit = 820.0 / 4096.0;
    let matrix = vec![vec![unit, 0.0, 0.0, 0.0, 0.0]];
    let positions = vec![jocforge_core::AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    }];
    let i0 = vec![0x0b, 0x77]
        .into_iter()
        .chain(std::iter::repeat_n(0, 4094))
        .collect::<Vec<_>>();
    let d0_carriers = vec![vec![0.0; 1536], vec![0.0; 1536]];

    let strict = package_source_derived_access_unit(
        JocProfile::Idx0,
        EmdfProfile::EtsiStrict,
        1,
        &matrix,
        &positions,
        &i0,
        &d0_carriers,
        0,
    )
    .expect("strict source-derived AU");
    let deployed = package_source_derived_access_unit(
        JocProfile::Idx0,
        EmdfProfile::DeployedCompat,
        1,
        &matrix,
        &positions,
        &i0,
        &d0_carriers,
        0,
    )
    .expect("deployed-profile source-derived AU");

    assert_eq!(strict.len(), 4096 + 2560);
    assert_eq!(deployed.len(), strict.len());
    assert_ne!(deployed, strict);
}
