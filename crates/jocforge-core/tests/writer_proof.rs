// pattern: Functional Core

use jocforge_core::{JocProfile, write_one_au_proof};

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
