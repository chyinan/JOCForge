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
