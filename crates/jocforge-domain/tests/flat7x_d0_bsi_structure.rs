// pattern: Functional Core
//! Input-only composition tests for the selected D0 BSI structure writer.

use jocforge_domain::{
    codec::{
        bitio::{FrameBitCursor, FrameBitWriter, FrameBitWriterError},
        frame_builder::{
            Flat7xD0AudioProductionMetadata, Flat7xD0BsiSegmentError, Flat7xD0BsiStructure,
            Flat7xD0BsiStructureError, Flat7xD0InfoAddbsiError, Flat7xD0InformationMetadata,
            Flat7xD0MixingMetadataError, Flat7xD0MixingMetadataValues,
            append_flat7x_d0_info_and_addbsi, append_flat7x_d0_mixing_metadata_and_infomdate,
            write_flat7x_d0_bsi_structure, write_flat7x_d0_bsi_through_chanmap,
        },
    },
    profile::Flat7xI0D0TargetPlan,
};

fn example_structures() -> [Flat7xD0BsiStructure<'static>; 2] {
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
            addbsi: Some(&[0x12, 0x34]),
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
            block_mix: [Some(1), None, Some(3), None, None, None],
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
            addbsi: None,
        },
    ]
}

fn append_components(
    bytes: &mut [u8],
    target: Flat7xI0D0TargetPlan,
    structure: Flat7xD0BsiStructure<'_>,
) -> u32 {
    let mut writer = FrameBitWriter::new(bytes, FrameBitCursor::new(0)).unwrap();
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
        structure.addbsi,
    )
    .unwrap()
    .bit_offset()
}

#[test]
fn typed_structure_matches_ordered_wash_approved_component_writers() {
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

        assert_eq!(end, sequential_end);
        assert_eq!(composed, sequential);
    }
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

    let mut invalid_addbsi = example_structures()[0];
    invalid_addbsi.addbsi = Some(&[]);
    assert_rejected_without_mutation(
        invalid_addbsi,
        96,
        Flat7xD0BsiStructureError::InfoAddbsi(Flat7xD0InfoAddbsiError::AddbsiLengthOutOfRange {
            bytes: 0,
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
    structure: Flat7xD0BsiStructure<'_>,
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
