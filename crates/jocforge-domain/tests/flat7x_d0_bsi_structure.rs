// pattern: Functional Core
//! Input-only composition tests for the selected D0 BSI structure writer.

use jocforge_domain::{
    codec::{
        bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
        frame_builder::{
            Flat7xD0AudioProductionMetadata, Flat7xD0BsiSegmentError, Flat7xD0BsiStructure,
            Flat7xD0BsiStructureError, Flat7xD0Ci16Addbsi, Flat7xD0InfoAddbsiError,
            Flat7xD0InformationMetadata, Flat7xD0MixingMetadataError, Flat7xD0MixingMetadataValues,
            append_flat7x_d0_info_and_addbsi, append_flat7x_d0_mixing_metadata_and_infomdate,
            measure_flat7x_d0_bsi_structure, write_flat7x_d0_bsi_structure,
            write_flat7x_d0_bsi_through_chanmap,
        },
    },
    profile::Flat7xI0D0TargetPlan,
};

fn example_structures() -> [Flat7xD0BsiStructure; 2] {
    [
        Flat7xD0BsiStructure {
            dialnorm: 16,
            compr: None,
            mixing: None,
            block_mix: [None; 6],
            information: Some(Flat7xD0InformationMetadata {
                bsmod: 3,
                copyrightb: true,
                origbs: false,
                audio_production: None,
                sourcefscod: 1,
            }),
            addbsi: Flat7xD0Ci16Addbsi::new(0x0110),
        },
        Flat7xD0BsiStructure {
            dialnorm: 16,
            compr: Some(0x42),
            mixing: Some(Flat7xD0MixingMetadataValues {
                mixmdo: 2,
                ltrtcmixlev: 1,
                lorocmixlev: 2,
                ltrtsurmixlev: 3,
                lorosurmixlev: 4,
            }),
            block_mix: [None; 6],
            information: Some(Flat7xD0InformationMetadata {
                bsmod: 4,
                copyrightb: false,
                origbs: true,
                audio_production: Some(Flat7xD0AudioProductionMetadata {
                    mixlevel: 18,
                    roomtyp: 1,
                    convtyp: 1,
                }),
                sourcefscod: 0,
            }),
            addbsi: Flat7xD0Ci16Addbsi::new(0x0110),
        },
    ]
}

fn append_components(
    bytes: &mut [u8],
    target: Flat7xI0D0TargetPlan,
    structure: Flat7xD0BsiStructure,
) -> u32 {
    let mut writer = FrameBitWriter::new(bytes, FrameBitCursor::new(0)).unwrap();
    let addbsi_bytes = structure.addbsi.bytes();
    write_flat7x_d0_bsi_through_chanmap(&mut writer, structure.dialnorm, structure.compr).unwrap();
    let infomdate = structure.information.is_some();
    append_flat7x_d0_mixing_metadata_and_infomdate(
        &mut writer,
        target,
        structure.mixing,
        structure.block_mix,
        infomdate,
    )
    .unwrap();
    append_flat7x_d0_info_and_addbsi(
        &mut writer,
        target,
        infomdate,
        structure.information,
        Some(&addbsi_bytes),
    )
    .unwrap()
    .bit_offset()
}

#[test]
fn dependent_bsi_diagnostic_omits_the_absent_frame_block_mix_flag() {
    let target = Flat7xI0D0TargetPlan::approved_contract();
    for structure in example_structures() {
        let mut composed = [0xa5_u8; 96];
        let mut sequential = composed;
        let end = {
            let mut writer = FrameBitWriter::new(&mut composed, FrameBitCursor::new(0)).unwrap();
            write_flat7x_d0_bsi_structure(&mut writer, target, structure)
                .unwrap()
                .bit_offset()
        };
        let sequential_end = append_components(&mut sequential, target, structure);

        assert_eq!(end + 1, sequential_end);
        assert_eq!(
            measure_flat7x_d0_bsi_structure(target, &structure)
                .unwrap()
                .value(),
            end
        );
        let chanmap_end = if structure.compr.is_some() { 76 } else { 68 };
        let mixing_value_bits = if structure.mixing.is_some() { 14 } else { 0 };
        let omitted_bit = chanmap_end + 1 + mixing_value_bits;
        for bit in 0..end {
            let component_bit = if bit < omitted_bit { bit } else { bit + 1 };
            assert_eq!(
                read_bit(&composed, bit),
                read_bit(&sequential, component_bit),
                "composed BSI differs from standard component output at bit {bit}"
            );
        }
    }
}

#[test]
fn dependent_bsi_diagnostic_rejects_any_block_mix_value_before_mutation() {
    let mut structure = example_structures()[0];
    structure.block_mix[0] = Some(1);
    assert_rejected_without_mutation(
        structure,
        96,
        Flat7xD0BsiStructureError::MixingMetadata(
            Flat7xD0MixingMetadataError::DiagnosticTrialRequiresNoBlockMix,
        ),
    );
}

#[test]
fn dependent_bsi_diagnostic_moves_audfrm_start_to_bit_93() {
    let structure = Flat7xD0BsiStructure {
        dialnorm: 16,
        compr: Some(1),
        mixing: Some(Flat7xD0MixingMetadataValues {
            mixmdo: 2,
            ltrtcmixlev: 1,
            lorocmixlev: 2,
            ltrtsurmixlev: 3,
            lorosurmixlev: 4,
        }),
        block_mix: [None; 6],
        information: None,
        addbsi: Flat7xD0Ci16Addbsi::new(0x0110),
    };
    let target = Flat7xI0D0TargetPlan::approved_contract();
    let mut bytes = [0_u8; 16];
    let end;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
        end = write_flat7x_d0_bsi_structure(&mut writer, target, structure)
            .unwrap()
            .bit_offset();
    }
    assert_eq!(end, 115);
    assert_eq!(
        measure_flat7x_d0_bsi_structure(target, &structure)
            .unwrap()
            .value(),
        115
    );
}

#[test]
fn target_ci16_scalar_is_written_as_the_required_two_byte_addbsi() {
    let target = Flat7xI0D0TargetPlan::approved_contract();
    assert_eq!(target.d0_ci16_addbsi_value(), 0x0110);
    assert_eq!(target.complexity_index_type_a(), 16);

    let structure = Flat7xD0BsiStructure {
        dialnorm: 16,
        compr: None,
        mixing: None,
        block_mix: [None; 6],
        information: None,
        addbsi: Flat7xD0Ci16Addbsi::new(target.d0_ci16_addbsi_value()),
    };
    let mut bytes = [0_u8; 16];
    let end;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
        end = write_flat7x_d0_bsi_structure(&mut writer, target, structure)
            .unwrap()
            .bit_offset();
    }

    assert!(!read_bit(&bytes, 68), "official target mixmdate is absent");
    assert!(!read_bit(&bytes, 69), "official target infomdate is absent");
    let addbsi_start = end - 23;
    assert_eq!(read_bit(&bytes, addbsi_start), true);
    assert_eq!(read_unsigned(&bytes, addbsi_start + 1, 6), 1);
    assert_eq!(read_unsigned(&bytes, addbsi_start + 7, 16), 0x0110);
}

#[test]
fn invalid_components_and_total_capacity_reject_without_mutation() {
    let mut invalid_dialnorm = example_structures()[0];
    invalid_dialnorm.dialnorm = 0;
    assert_rejected_without_mutation(
        invalid_dialnorm,
        96,
        Flat7xD0BsiStructureError::BsiSegment(Flat7xD0BsiSegmentError::DialnormOutOfRange {
            value: 0,
        }),
    );

    let mut invalid_mixing = example_structures()[1];
    invalid_mixing.mixing.as_mut().unwrap().mixmdo = 4;
    assert_rejected_without_mutation(
        invalid_mixing,
        96,
        Flat7xD0BsiStructureError::MixingMetadata(Flat7xD0MixingMetadataError::MixmdoOutOfRange {
            value: 4,
        }),
    );

    let mut invalid_information = example_structures()[0];
    invalid_information.information.as_mut().unwrap().bsmod = 8;
    assert_rejected_without_mutation(
        invalid_information,
        96,
        Flat7xD0BsiStructureError::InfoAddbsi(Flat7xD0InfoAddbsiError::FieldOutOfRange {
            field: "bsmod",
            value: 8,
        }),
    );

    assert_rejected_without_mutation(
        example_structures()[0],
        12,
        Flat7xD0BsiStructureError::BitWriter(FrameBitWriterError::BufferTooSmall),
    );
}

#[test]
fn nonzero_start_cursor_rejects_without_mutation() {
    let mut bytes = [0x5a_u8; 96];
    let original = bytes;
    let error;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(1)).unwrap();
        error = write_flat7x_d0_bsi_structure(
            &mut writer,
            Flat7xI0D0TargetPlan::approved_contract(),
            example_structures()[0],
        )
        .unwrap_err();
        cursor = writer.cursor();
    }
    assert_eq!(
        error,
        Flat7xD0BsiStructureError::CursorMismatch { actual: 1 }
    );
    assert_eq!(cursor.bit_offset(), 1);
    assert_eq!(bytes, original);
}

fn assert_rejected_without_mutation(
    structure: Flat7xD0BsiStructure,
    capacity_bytes: usize,
    expected_error: Flat7xD0BsiStructureError,
) {
    let mut bytes = vec![0x5a_u8; capacity_bytes];
    let original = bytes.clone();
    let error;
    let cursor;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
        error = write_flat7x_d0_bsi_structure(
            &mut writer,
            Flat7xI0D0TargetPlan::approved_contract(),
            structure,
        )
        .unwrap_err();
        cursor = writer.cursor();
    }
    assert_eq!(error, expected_error);
    assert_eq!(cursor.bit_offset(), 0);
    assert_eq!(bytes, original);
}

fn read_bit(bytes: &[u8], bit_offset: u32) -> bool {
    let byte = bytes[(bit_offset / 8) as usize];
    byte & (1_u8 << (7 - bit_offset % 8)) != 0
}

fn read_unsigned(bytes: &[u8], start_bit: u32, width: u8) -> u32 {
    (0..u32::from(width)).fold(0_u32, |value, offset| {
        (value << 1) | u32::from(read_bit(bytes, start_bit + offset))
    })
}
