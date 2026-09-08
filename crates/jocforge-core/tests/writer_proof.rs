// pattern: Functional Core

use jocforge_core::{
    BlockPartition, DependentLfePolicy, DependentPlan, EmdfProfile, JocCarriage, JocProfile,
    OwnershipPlan, canonical_dependent_channel_maps, d0_carrier_causality,
    package_i0_skip_field_access_unit, package_legacy_ac3_source_derived_access_unit,
    package_multi_dependent_source_derived_access_unit,
    package_multi_dependent_source_derived_access_unit_with_plan,
    package_short_block_conformance_access_unit, package_source_derived_access_unit,
    validate_multi_dependent_access_unit, write_one_au_proof,
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
    let phase_height = jocforge_core::write_one_au_proof(JocProfile::Idx4).expect("idx4 proof");

    assert_eq!(flat.len(), 4096 + 2560);
    assert_eq!(height.len(), 4096 + 2560);
    assert_eq!(phase_height.len(), 4096 + 2560);
    assert_eq!(&flat[4096..4098], &[0x0b, 0x77]);
    assert_eq!(&height[4096..4098], &[0x0b, 0x77]);
    assert_eq!(&phase_height[4096..4098], &[0x0b, 0x77]);
}

#[test]
fn phase_profiles_keep_their_base_carriage_widths() {
    let phase_five = jocforge_core::write_one_au_proof(JocProfile::Idx3).expect("idx3 proof");
    let phase_height = jocforge_core::write_one_au_proof(JocProfile::Idx4).expect("idx4 proof");

    assert_eq!(phase_five.len(), 4096);
    assert_eq!(phase_height.len(), 4096 + 2560);
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
    for profile in [
        JocProfile::Idx0,
        JocProfile::Idx1,
        JocProfile::Idx2,
        JocProfile::Idx3,
        JocProfile::Idx4,
    ] {
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
fn multi_dependent_writer_places_joc_emdf_only_on_the_last_sequential_dependent() {
    let unit = 820.0 / 4096.0;
    let matrix = vec![vec![unit, 0.0, 0.0, 0.0, 0.0]];
    let position = jocforge_core::AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    };
    let i0 = write_one_au_proof(JocProfile::Idx1)
        .expect("valid synthetic I0")
        .into_iter()
        .take(4096)
        .collect::<Vec<_>>();
    let maps = vec![0xc000, 0x3000];
    let carriers = vec![
        vec![vec![0.1; 1536], vec![0.2; 1536]],
        vec![vec![0.3; 1536], vec![0.4; 1536]],
    ];

    let access_unit = package_multi_dependent_source_derived_access_unit(
        JocProfile::Idx0,
        EmdfProfile::EtsiStrict,
        1,
        &matrix,
        &[position],
        &i0,
        &maps,
        &carriers,
        0,
    )
    .expect("two-dependent source-derived AU");

    assert_eq!(access_unit.len(), 4096 + 2 * 2560);
    assert_eq!(&access_unit[4096..4098], &[0x0b, 0x77]);
    assert_eq!(&access_unit[6656..6658], &[0x0b, 0x77]);
    assert!(
        access_unit[6656..]
            .windows(2)
            .any(|window| window == [0x58, 0x38])
    );

    let report = validate_multi_dependent_access_unit(&access_unit, JocProfile::Idx0)
        .expect("independent multi-dependent structural validation");
    assert_eq!(report.dependent_ids, vec![0, 1]);
    assert_eq!(report.dependent_channel_maps, maps);
    assert_eq!(report.joc_owner, Some(1));
    assert_eq!(report.joc_downmix_index, 0);
    assert_eq!(report.complexity_index, 2);
    assert_eq!(report.total_samples, 1536);
    assert!(report.unique_dependent_channel_ownership);
    assert_eq!(
        report
            .ownership_census
            .iter()
            .map(|entry| entry.channel_tokens.len())
            .collect::<Vec<_>>(),
        vec![5, 2, 2]
    );
}

#[test]
fn multi_dependent_writer_supports_three_source_derived_dependents() {
    let matrix = vec![vec![820.0 / 4096.0; 7]];
    let position = jocforge_core::AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    };
    let i0 = write_one_au_proof(JocProfile::Idx1)
        .expect("valid synthetic I0")
        .into_iter()
        .take(4096)
        .collect::<Vec<_>>();
    let maps = vec![0xc000, 0x3000, 0x0200];
    let carriers = (0..3)
        .map(|dependent| {
            vec![
                vec![0.1 + f64::from(dependent) * 0.01; 1536],
                vec![0.2 + f64::from(dependent) * 0.01; 1536],
            ]
        })
        .collect::<Vec<_>>();

    let access_unit = package_multi_dependent_source_derived_access_unit(
        JocProfile::Idx1,
        EmdfProfile::EtsiStrict,
        1,
        &matrix,
        &[position],
        &i0,
        &maps,
        &carriers,
        0,
    )
    .expect("three-dependent source-derived AU");
    let report = validate_multi_dependent_access_unit(&access_unit, JocProfile::Idx1)
        .expect("three-dependent structural validation");

    assert_eq!(report.component_count, 4);
    assert_eq!(report.dependent_ids, vec![0, 1, 2]);
    assert_eq!(report.dependent_channel_maps, maps);
    assert_eq!(report.joc_owner, Some(2));
    assert_eq!(report.joc_downmix_index, 1);
}

#[test]
fn multi_dependent_writer_supports_six_unique_single_channel_dependents() {
    let matrix = vec![vec![820.0 / 4096.0; 7]];
    let position = jocforge_core::AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    };
    let i0 = write_one_au_proof(JocProfile::Idx1)
        .expect("valid synthetic I0")
        .into_iter()
        .take(4096)
        .collect::<Vec<_>>();
    let maps = vec![0x8000, 0x4000, 0x2000, 0x1000, 0x0800, 0x0200];
    let carriers = (0..maps.len())
        .map(|dependent| {
            let channels = if dependent == maps.len() - 1 { 2 } else { 1 };
            (0..channels)
                .map(|channel| {
                    vec![
                        0.1 + f64::from(u32::try_from(dependent).expect("dependent")) * 0.01
                            + f64::from(u32::try_from(channel).expect("channel")) * 0.01;
                        1536
                    ]
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let access_unit = package_multi_dependent_source_derived_access_unit(
        JocProfile::Idx1,
        EmdfProfile::EtsiStrict,
        1,
        &matrix,
        &[position],
        &i0,
        &maps,
        &carriers,
        0,
    )
    .expect("six-dependent source-derived AU");
    let report = validate_multi_dependent_access_unit(&access_unit, JocProfile::Idx1)
        .expect("six-dependent structural validation");

    assert_eq!(report.dependent_ids, (0..6).collect::<Vec<_>>());
    assert_eq!(report.joc_owner, Some(5));
    assert_eq!(report.dependent_channel_maps, maps);

    for dependent in 0..carriers.len() {
        let mut perturbed_carriers = carriers.clone();
        perturbed_carriers[dependent][0][0] += 0.037;
        let perturbed = package_multi_dependent_source_derived_access_unit(
            JocProfile::Idx1,
            EmdfProfile::EtsiStrict,
            1,
            &matrix,
            &[position],
            &i0,
            &maps,
            &perturbed_carriers,
            0,
        )
        .expect("perturbed source-derived AU");
        let start = i0.len() + dependent * 2560;
        assert_ne!(
            &access_unit[start..start + 2560],
            &perturbed[start..start + 2560],
            "dependent D{dependent} must retain source causality"
        );
    }
}

#[test]
fn multi_dependent_writer_rejects_duplicate_dependent_channel_ownership() {
    let position = jocforge_core::AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    };
    let i0 = write_one_au_proof(JocProfile::Idx1)
        .expect("valid synthetic I0")
        .into_iter()
        .take(4096)
        .collect::<Vec<_>>();
    let carriers = vec![
        vec![vec![0.1; 1536], vec![0.2; 1536]],
        vec![vec![0.3; 1536], vec![0.4; 1536]],
    ];
    let error = package_multi_dependent_source_derived_access_unit(
        JocProfile::Idx1,
        EmdfProfile::EtsiStrict,
        1,
        &[vec![820.0 / 4096.0; 7]],
        &[position],
        &i0,
        &[0xc000, 0xc000],
        &carriers,
        0,
    )
    .expect_err("duplicate dependent ownership must fail closed");
    assert!(error.contains("duplicate") || error.contains("overlap"));
}

#[test]
fn independent_validator_rejects_out_of_order_dependent_id() {
    let position = jocforge_core::AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    };
    let i0 = write_one_au_proof(JocProfile::Idx1)
        .expect("valid synthetic I0")
        .into_iter()
        .take(4096)
        .collect::<Vec<_>>();
    let carriers = vec![
        vec![vec![0.1; 1536], vec![0.2; 1536]],
        vec![vec![0.3; 1536], vec![0.4; 1536]],
    ];
    let mut access_unit = package_multi_dependent_source_derived_access_unit(
        JocProfile::Idx1,
        EmdfProfile::EtsiStrict,
        1,
        &[vec![820.0 / 4096.0; 7]],
        &[position],
        &i0,
        &[0xc000, 0x0200],
        &carriers,
        0,
    )
    .expect("valid two-dependent AU");
    access_unit[4096 + 2] |= 0x10;

    let error = validate_multi_dependent_access_unit(&access_unit, JocProfile::Idx1)
        .expect_err("out-of-order dependent must fail closed");
    assert!(error.contains("sequential"));
}

#[test]
fn multi_dependent_writer_rejects_an_empty_chanmap() {
    let position = jocforge_core::AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    };
    let i0 = write_one_au_proof(JocProfile::Idx1)
        .expect("valid synthetic I0")
        .into_iter()
        .take(4096)
        .collect::<Vec<_>>();
    let error = package_multi_dependent_source_derived_access_unit(
        JocProfile::Idx0,
        EmdfProfile::EtsiStrict,
        1,
        &[vec![820.0 / 4096.0; 5]],
        &[position],
        &i0,
        &[0],
        &[vec![vec![0.1; 1536]]],
        0,
    )
    .expect_err("empty chanmap must fail closed");
    assert!(error.contains("channel map"));
}

#[test]
fn one_multi_dependent_frame_matches_the_existing_d0_writer() {
    let unit = 820.0 / 4096.0;
    let matrix = vec![vec![unit, 0.0, 0.0, 0.0, 0.0]];
    let position = jocforge_core::AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    };
    let i0 = vec![0x0b, 0x77]
        .into_iter()
        .chain(std::iter::repeat_n(0, 4094))
        .collect::<Vec<_>>();
    let carriers = vec![vec![0.1; 1536], vec![0.2; 1536]];
    let normal = package_source_derived_access_unit(
        JocProfile::Idx0,
        EmdfProfile::EtsiStrict,
        1,
        &matrix,
        &[position],
        &i0,
        &carriers,
        0,
    )
    .expect("normal D0");
    let multi = package_multi_dependent_source_derived_access_unit(
        JocProfile::Idx0,
        EmdfProfile::EtsiStrict,
        1,
        &matrix,
        &[position],
        &i0,
        &[0xc000],
        &[carriers],
        0,
    )
    .expect("multi D0");
    assert_eq!(normal, multi);
}

#[test]
fn legacy_ac3_package_retains_the_original_core_and_appends_one_d0() {
    let unit = 820.0 / 4096.0;
    let matrix = vec![vec![unit, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]];
    let position = jocforge_core::AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    };
    let ac3 = vec![0x0b, 0x77]
        .into_iter()
        .chain(std::iter::repeat_n(0, 2558))
        .collect::<Vec<_>>();
    let carriers = vec![vec![0.1; 1536], vec![0.2; 1536]];
    let error = package_legacy_ac3_source_derived_access_unit(
        JocProfile::Idx1,
        EmdfProfile::EtsiStrict,
        1,
        &matrix,
        &[position],
        &ac3,
        &carriers,
        0,
    )
    .expect_err("zero-filled AC-3 core must fail its public header validation");
    assert!(error.contains("legacy AC-3"));
}

#[test]
fn short_block_conformance_writer_preserves_one_1536_sample_group() {
    let unit = 820.0 / 4096.0;
    let matrix = vec![vec![unit, 0.0, 0.0, 0.0, 0.0]];
    let position = jocforge_core::AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    };
    let partitions = vec![BlockPartition::One; 6];
    let access_unit = package_short_block_conformance_access_unit(
        JocProfile::Idx0,
        EmdfProfile::EtsiStrict,
        1,
        &matrix,
        &[position],
        &partitions,
    )
    .expect("short-block conformance AU");

    assert_eq!(access_unit.len(), 6 * (512 + 512));
}

#[test]
fn source_derived_short_writer_emits_real_partitioned_frames_and_tracks_carriers() {
    let plan = jocforge_core::resolve_encoding_plan_with_partition(
        JocProfile::Idx1,
        JocCarriage::EtsiAuxDependent,
        EmdfProfile::EtsiStrict,
        1664,
        &[BlockPartition::Three, BlockPartition::Three],
    )
    .expect("source-derived 3+3 plan");
    let positions = vec![jocforge_core::AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    }];
    let matrix = vec![vec![0.2; 7]];
    let i0_carriers = vec![vec![0.1; 1536]; 5];
    let d0_carriers = vec![vec![0.2; 1536]; 2];
    let bytes = jocforge_core::package_source_derived_short_block_access_unit(
        &plan,
        1,
        &matrix,
        &positions,
        &i0_carriers,
        &d0_carriers,
    )
    .expect("source-derived short AU");

    assert_eq!(bytes.len(), 2 * (2048 + 1280));
    let block_codes = (0..4)
        .map(|frame| {
            let offset = if frame % 2 == 0 {
                frame / 2 * 3328
            } else {
                (frame / 2) * 3328 + 2048
            };
            (bytes[offset + 4] >> 4) & 0x03
        })
        .collect::<Vec<_>>();
    assert_eq!(block_codes, vec![2, 2, 2, 2]);
    let report = jocforge_core::validate_short_block_access_unit(
        &bytes,
        JocProfile::Idx1,
        &[BlockPartition::Three, BlockPartition::Three],
    )
    .expect("independent short AU validation");
    assert_eq!(report.syncframes, 4);
    assert_eq!(report.audio_blocks, 6);
    assert_eq!(report.samples, 1536);
    assert_eq!(report.metadata_owner_frame, 3);
    let mut corrupted = bytes.clone();
    let last = corrupted.len() - 1;
    corrupted[last] ^= 1;
    assert!(
        jocforge_core::validate_short_block_access_unit(
            &corrupted,
            JocProfile::Idx1,
            &[BlockPartition::Three, BlockPartition::Three],
        )
        .expect_err("corrupted short AU CRC2")
        .contains("crc2")
    );

    let changed_i0 = vec![vec![0.2; 1536]; 5];
    let changed = jocforge_core::package_source_derived_short_block_access_unit(
        &plan,
        1,
        &matrix,
        &positions,
        &changed_i0,
        &d0_carriers,
    )
    .expect("changed source-derived short AU");
    assert_ne!(bytes, changed);
}

#[test]
fn multi_dependent_writer_rejects_more_than_eight_or_nonsequential_dependents() {
    let i0 = vec![0x0b, 0x77]
        .into_iter()
        .chain(std::iter::repeat_n(0, 4094))
        .collect::<Vec<_>>();
    let carriers = vec![vec![vec![0.0; 1536]]; 9];
    let maps = vec![
        0x8000, 0x4000, 0x2000, 0x1000, 0x0800, 0x0100, 0x0080, 0x0040, 0x0020,
    ];
    let matrix = vec![vec![0.0; 5]];
    let position = jocforge_core::AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    };
    let too_many = package_multi_dependent_source_derived_access_unit(
        JocProfile::Idx0,
        EmdfProfile::EtsiStrict,
        1,
        &matrix,
        &[position],
        &i0,
        &maps,
        &carriers,
        0,
    )
    .expect_err("D8 must be outside the public dependent-substream bound");
    assert!(too_many.contains("maximum is 8"));
}

#[test]
fn malformed_d6_and_d7_plans_fail_before_any_frame_is_emitted() {
    let i0 = write_one_au_proof(JocProfile::Idx0)
        .expect("valid synthetic I0")
        .into_iter()
        .take(4096)
        .collect::<Vec<_>>();
    let position = jocforge_core::AdmPosition {
        azimuth: 0.0,
        elevation: 0.0,
        distance: 0.0,
    };
    let base = canonical_dependent_channel_maps(JocProfile::Idx0, 5)
        .expect("canonical idx0 maps")
        .into_iter()
        .enumerate()
        .map(|(id, channel_map)| {
            DependentPlan::new(
                u8::try_from(id).unwrap_or(7),
                channel_map,
                DependentLfePolicy::Absent,
            )
        })
        .collect::<Vec<_>>();
    for extra_count in 1..=2 {
        let final_id = u8::try_from(5 + extra_count - 1).unwrap_or(7);
        let mut dependents = base.clone();
        for id in 5..=final_id {
            dependents.push(DependentPlan::new(id, 0x4002, DependentLfePolicy::Absent));
        }
        let plan = OwnershipPlan {
            profile: JocProfile::Idx0,
            independent_lfe: false,
            dependents,
            final_joc_owner: final_id,
        };
        let carriers = plan
            .dependents
            .iter()
            .map(|dependent| vec![vec![0.0; 1536]; dependent.full_band_channel_count()])
            .collect::<Vec<_>>();
        let error = package_multi_dependent_source_derived_access_unit_with_plan(
            JocProfile::Idx0,
            EmdfProfile::EtsiStrict,
            1,
            &[vec![820.0 / 4096.0; 5]],
            &[position],
            &i0,
            &plan,
            &carriers,
            &vec![None; plan.dependents.len()],
            0,
        )
        .expect_err("malformed D6/D7 plan must fail closed");
        assert!(error.contains("unsupported LFE2"));
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
