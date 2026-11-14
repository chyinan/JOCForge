// pattern: Functional Core
use jocforge_domain::{
    joc::coefficient_payload::{
        CoefficientPayloadInput, CoefficientPayloadObject, write_selected_id14_coefficient_payload,
        write_selected_id14_coefficient_payload_for_profile,
    },
    joc::coefficient_representation::JocCoefficientRowLayout,
    joc::huffman::{
        Flat7xTargetId14HeaderError, Flat7xTargetId14HeaderInput, HuffmanBitCount, Id14HeaderInput,
        Id14ObjectDescriptor, Id14Presence, Id14WriterReturnCode, Id14WriterState,
        write_id14_header, write_id14_header_for_flat7x_target, write_id14_header_for_profile,
    },
    profile::{
        CarrierCodecProfile, CodecRateProfile, EncodingProfile, Flat7xFrameCodec,
        Flat7xI0D0TargetPlan, Flat7xSubstreamRole, JocDownmixProfile, JocPayloadDownmixIndex,
        JocReconstructionLayout, ProfileError, ResolvedEncodingProfile, SubstreamTopology,
        resolve_selected_profile,
    },
};
use serde_json::Value;

const FLAT7X_INPUT_ONLY_VECTORS: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/JOCFORGE-FLAT7X-1024-PARTIAL-STRUCTURE-V1/INPUT-ONLY-VECTORS.json"
);

fn header_input(downmix_index: u32, object_count: u32, extension_config: u32) -> Id14HeaderInput {
    let descriptors = vec![
        Id14ObjectDescriptor {
            present: Id14Presence::Absent,
            kind_code: 0,
            mode_code: 0,
            nested_mode: 0,
            nested_count: 1,
            selected_index0: 0,
            selected_index1: 0,
        };
        usize::try_from(object_count).unwrap()
    ];
    Id14HeaderInput {
        downmix_index,
        object_count,
        extension_config,
        clip_gain_index: 0,
        sequence_count: 1,
        channel_limit: 24,
        descriptors: Some(descriptors),
        body: Vec::new(),
        body_valid_bits: HuffmanBitCount::new(0),
    }
}

fn resolved_flat7x_profile() -> ResolvedEncodingProfile {
    resolve_selected_profile(&EncodingProfile::selected_with_joc_reconstruction_layout(
        JocReconstructionLayout::Flat7x,
    ))
    .unwrap()
}

fn flat7x_target_header_input(
    downmix_index: u32,
    object_count: u32,
    extension_config: u32,
) -> Flat7xTargetId14HeaderInput {
    let input = header_input(downmix_index, object_count, extension_config);
    Flat7xTargetId14HeaderInput {
        downmix_index: input.downmix_index,
        object_count: input.object_count,
        extension_config: input.extension_config,
        clip_gain_index: input.clip_gain_index,
        sequence_count: input.sequence_count,
        channel_limit: input.channel_limit,
        descriptors: input.descriptors,
    }
}

fn first_bits(bytes: &[u8], count: usize) -> String {
    (0..count)
        .map(|bit| {
            let byte = bytes[bit / 8];
            let shift = 7 - u32::try_from(bit % 8).unwrap();
            if (byte >> shift) & 1 == 0 { '0' } else { '1' }
        })
        .collect()
}

fn read_bits(bytes: &[u8], start_bit: usize, width: usize) -> u32 {
    (0..width).fold(0_u32, |value, offset| {
        let bit = start_bit + offset;
        let byte = bytes[bit / 8];
        let shift = 7 - (bit % 8);
        (value << 1) | u32::from((byte >> shift) & 1)
    })
}

#[test]
fn wash_vectors_keep_layout_downmix_and_header_domains_separate() {
    let vectors: Value = serde_json::from_slice(FLAT7X_INPUT_ONLY_VECTORS).unwrap();
    assert_eq!(
        vectors["package_id"],
        "JOCFORGE-FLAT7X-1024-PARTIAL-STRUCTURE-V1"
    );
    assert!(!vectors["contains_media_or_bitstream"].as_bool().unwrap());

    let flat7x = resolved_flat7x_profile();
    assert_eq!(
        flat7x.joc_reconstruction_layout(),
        JocReconstructionLayout::Flat7x
    );
    assert_eq!(flat7x.joc_dmx_config_idx(), Some(1));
    assert_eq!(
        flat7x.joc_payload_downmix_index(),
        JocPayloadDownmixIndex::SevenChannel
    );
    assert_eq!(
        JocCoefficientRowLayout::for_payload_downmix_index(flat7x.joc_payload_downmix_index()),
        JocCoefficientRowLayout::SevenChannel
    );
    assert_eq!(
        flat7x.carrier_codec(),
        CarrierCodecProfile::Flat7xAc3I0DependentEac3D0
    );
    assert_eq!(
        flat7x.substream_topology(),
        SubstreamTopology::Flat7xIndependentI0DependentD0
    );
    assert_eq!(
        resolve_selected_profile(
            &EncodingProfile::selected_with_joc_reconstruction_layout(
                JocReconstructionLayout::Flat7x,
            )
            .with_joc_payload_downmix_index(JocPayloadDownmixIndex::FiveChannel),
        ),
        Err(ProfileError::UnsupportedProfile)
    );
    assert_eq!(
        flat7x.joc_downmix(),
        JocDownmixProfile::SelectedMode0Phase1Index3
    );
    assert_eq!(flat7x.codec_rate(), CodecRateProfile::Kbps1024);

    let selected_five_point_one = resolve_selected_profile(&EncodingProfile::selected()).unwrap();
    assert_eq!(selected_five_point_one.joc_dmx_config_idx(), None);
    assert_eq!(
        selected_five_point_one.joc_payload_downmix_index(),
        JocPayloadDownmixIndex::FiveChannel
    );
    assert_eq!(
        JocCoefficientRowLayout::for_payload_downmix_index(
            selected_five_point_one.joc_payload_downmix_index()
        ),
        JocCoefficientRowLayout::FiveChannel
    );
    assert_eq!(
        selected_five_point_one.carrier_codec(),
        CarrierCodecProfile::IndependentEac3
    );
    assert_eq!(
        selected_five_point_one.substream_topology(),
        SubstreamTopology::SelectedSingleIndependentZeroDependent
    );
    let target = Flat7xI0D0TargetPlan::approved_contract();
    assert_eq!(target.i0().codec(), Flat7xFrameCodec::Ac3);
    assert_eq!(target.i0().bsid(), 6);
    assert_eq!(target.i0().role(), Flat7xSubstreamRole::IndependentI0);
    assert_eq!(target.d0().codec(), Flat7xFrameCodec::Eac3);
    assert_eq!(target.d0().bsid(), 16);
    assert_eq!(target.d0().role(), Flat7xSubstreamRole::DependentD0);

    for vector in vectors["vectors"].as_array().unwrap() {
        match vector["id"].as_str().unwrap() {
            "flat7x-index1" => {
                assert_eq!(vector["expected"]["joc_dmx_config_idx"]["width_bits"], 3);
                assert_eq!(vector["expected"]["joc_dmx_config_idx"]["value"], 1);
                assert_eq!(vector["expected"]["joc_num_channels"], 7);
            }
            "tfl_tfr-distinguishing-index2" => {
                assert_eq!(vector["input"]["joc_dmx_config_idx"], 2);
                assert_eq!(JocReconstructionLayout::from_joc_dmx_config_idx(2), None);
                assert_eq!(flat7x.joc_dmx_config_idx(), Some(1));
            }
            "reserved-index-rejection" => {
                assert_eq!(vector["input"]["joc_dmx_config_idx"], 5);
                assert_eq!(JocReconstructionLayout::from_joc_dmx_config_idx(5), None);
                assert_eq!(
                    resolve_selected_profile(
                        &EncodingProfile::selected_with_joc_reconstruction_layout(
                            JocReconstructionLayout::Unsupported,
                        )
                    ),
                    Err(ProfileError::UnsupportedProfile)
                );
            }
            "dimension-separation" => {
                assert_eq!(vector["expected"]["JocDownmixProfile"]["mode"], 0);
                assert_eq!(vector["expected"]["JocDownmixProfile"]["phase"], 1);
                assert_eq!(vector["expected"]["JocDownmixProfile"]["internal_index"], 3);
                assert_eq!(vector["expected"]["joc_dmx_config_idx"], 1);
                assert_eq!(
                    flat7x.joc_downmix(),
                    JocDownmixProfile::SelectedMode0Phase1Index3
                );
                assert_eq!(
                    flat7x.joc_reconstruction_layout(),
                    JocReconstructionLayout::Flat7x
                );
            }
            "object-count-complexity-domain-separation" => {
                assert!(
                    vector["expected"]["fields_are_independent"]
                        .as_bool()
                        .unwrap()
                );
                assert!(
                    !vector["expected"]["derive_one_from_the_other"]
                        .as_bool()
                        .unwrap()
                );
                assert_eq!(vector["input"]["joc_num_objects_bits"]["width_bits"], 6);
                assert_eq!(vector["input"]["complexity_index_type_a"]["width_bits"], 8);
            }
            other => panic!("unexpected Wash input-only vector: {other}"),
        }
    }
}

#[test]
fn selected_five_point_one_resolution_and_header_bytes_remain_unchanged() {
    let profile = resolve_selected_profile(&EncodingProfile::selected()).unwrap();
    assert_eq!(
        profile.joc_reconstruction_layout(),
        JocReconstructionLayout::ExistingSelectedFivePointOne
    );
    assert_eq!(profile.joc_dmx_config_idx(), None);
    assert_eq!(
        profile.joc_downmix(),
        JocDownmixProfile::SelectedMode0Phase1Index3
    );

    let input = header_input(3, 3, 0);
    let mut established_writer = Id14WriterState::new(vec![0; 16]);
    let established = write_id14_header(&input, &mut established_writer).unwrap();
    assert_eq!(established.return_code(), Id14WriterReturnCode::Success);

    let mut profile_writer = Id14WriterState::new(vec![0; 16]);
    let through_profile =
        write_id14_header_for_profile(&input, profile, &mut profile_writer).unwrap();
    assert_eq!(through_profile.return_code(), Id14WriterReturnCode::Success);
    assert_eq!(profile_writer.buffer(), established_writer.buffer());
    assert_eq!(first_bits(profile_writer.buffer(), 12), "011000010000");
}

#[test]
fn flat7x_profile_writes_index_one_before_object_and_extension_fields() {
    let profile = resolved_flat7x_profile();
    let input = header_input(3, 3, 0);
    let mut writer = Id14WriterState::new(vec![0; 16]);
    let outcome = write_id14_header_for_profile(&input, profile, &mut writer).unwrap();

    assert_eq!(outcome.return_code(), Id14WriterReturnCode::Success);
    assert_eq!(first_bits(writer.buffer(), 12), "001000010000");
    assert_eq!(input.downmix_index, 3);
    assert_eq!(input.object_count, 3);
    assert_eq!(input.extension_config, 0);
}

#[test]
fn flat7x_target_header_writes_index1_and_count15_as_u6_count_minus_one() {
    let target = Flat7xI0D0TargetPlan::approved_contract();
    let profile = resolved_flat7x_profile();
    assert_eq!(target.joc_dmx_config_idx(), 1);
    assert_eq!(target.joc_spatial_index().value(), 1);
    assert_eq!(target.downmix_index().value(), 1);
    assert_eq!(target.downmix_index().coefficient_width(), 7);
    assert_eq!(target.joc_object_count(), 15);
    assert_eq!(target.complexity_index_type_a(), 16);
    assert_eq!(target.reconstruction_carriers().len(), 7);
    assert_eq!(target.accepted_physical_channel_counts(), [36, 38]);
    assert_eq!(profile.joc_dmx_config_idx(), Some(1));
    assert_eq!(
        profile.joc_downmix(),
        JocDownmixProfile::SelectedMode0Phase1Index3
    );

    let input = flat7x_target_header_input(1, 15, 0);
    let mut writer = Id14WriterState::new(vec![0xa5; 64]);
    let outcome = write_id14_header_for_flat7x_target(&input, Some(target), &mut writer).unwrap();

    assert_eq!(outcome.return_code(), Id14WriterReturnCode::Success);
    assert_eq!(first_bits(writer.buffer(), 9), "001001110");
    assert_eq!(writer.output_length().value(), 6);
    assert_eq!(input.downmix_index, 1);
    assert_eq!(input.object_count, 15);
}

#[test]
fn id14_slot_count_and_full_object_presence_flags_are_separate_inputs() {
    let target = Flat7xI0D0TargetPlan::approved_contract();
    let mut input = flat7x_target_header_input(1, 15, 0);
    input.descriptors = Some(vec![
        Id14ObjectDescriptor {
            present: Id14Presence::Present,
            kind_code: 0,
            mode_code: 1,
            nested_mode: 0,
            nested_count: 1,
            selected_index0: 0,
            selected_index1: 0,
        };
        15
    ]);
    let mut writer = Id14WriterState::new(vec![0_u8; 64]);
    let outcome = write_id14_header_for_flat7x_target(&input, Some(target), &mut writer).unwrap();

    assert_eq!(input.object_count, 15);
    assert_eq!(read_bits(writer.buffer(), 3, 6), 14);
    assert_eq!(outcome.valid_bit_count().value(), 150);
    for slot in 0..15 {
        let descriptor_start = 30 + slot * 8;
        assert_eq!(read_bits(writer.buffer(), descriptor_start, 1), 1);
        assert_eq!(read_bits(writer.buffer(), descriptor_start + 4, 2), 1);
    }
}

#[test]
fn flat7x_target_header_rejects_missing_contract_and_count_mismatch_atomically() {
    let target = Flat7xI0D0TargetPlan::approved_contract();
    let mut writer = Id14WriterState::new(vec![0xa5; 64]);
    let original = writer.clone();

    assert_eq!(
        write_id14_header_for_flat7x_target(
            &flat7x_target_header_input(3, 15, 0),
            None,
            &mut writer,
        ),
        Err(Flat7xTargetId14HeaderError::MissingTargetContract)
    );
    assert_eq!(writer, original);

    assert_eq!(
        write_id14_header_for_flat7x_target(
            &flat7x_target_header_input(3, 15, 0),
            Some(target),
            &mut writer,
        ),
        Err(Flat7xTargetId14HeaderError::DownmixIndexMismatch {
            expected: 1,
            actual: 3,
        })
    );
    assert_eq!(writer, original);

    assert_eq!(
        write_id14_header_for_flat7x_target(
            &flat7x_target_header_input(1, 14, 0),
            Some(target),
            &mut writer,
        ),
        Err(Flat7xTargetId14HeaderError::ObjectCountMismatch {
            expected: 15,
            actual: 14,
        })
    );
    assert_eq!(writer, original);
}

#[test]
fn profile_aware_payload_writer_keeps_payload_and_reconstruction_selectors_separate() {
    let input = CoefficientPayloadInput {
        downmix_index: 3,
        extension_config: 0,
        clip_gain_index: 0,
        sequence_count: 1,
        time_slot_limit: 24,
        capacity_bytes: 256,
        objects: vec![CoefficientPayloadObject {
            present: false,
            band_enum: 0,
            mode: 0,
            points: Vec::new(),
            offsets: None,
        }],
    };

    let selected_profile = resolve_selected_profile(&EncodingProfile::selected()).unwrap();
    let flat7x_profile = resolved_flat7x_profile();
    let selected_five_point_one = write_selected_id14_coefficient_payload(&input).unwrap();
    let flat7x =
        write_selected_id14_coefficient_payload_for_profile(&input, flat7x_profile).unwrap();

    assert_eq!(input.downmix_index, 3);
    assert_eq!(selected_profile.joc_dmx_config_idx(), None);
    assert_eq!(selected_profile.joc_payload_downmix_index().value(), 3);
    assert_eq!(flat7x_profile.joc_dmx_config_idx(), Some(1));
    assert_eq!(flat7x_profile.joc_payload_downmix_index().value(), 1);
    assert_eq!(
        selected_five_point_one.writer_return_code(),
        Some(Id14WriterReturnCode::Success)
    );
    assert_eq!(
        flat7x.writer_return_code(),
        Some(Id14WriterReturnCode::Success)
    );
    assert_eq!(
        first_bits(selected_five_point_one.payload().unwrap(), 3),
        "011"
    );
    assert_eq!(first_bits(flat7x.payload().unwrap(), 3), "001");
}

#[test]
fn flat7x_is_limited_to_the_existing_1024_kbps_core_row() {
    let profile = EncodingProfile::selected_with_codec_rate(CodecRateProfile::Kbps768)
        .with_joc_reconstruction_layout(JocReconstructionLayout::Flat7x);
    assert_eq!(
        resolve_selected_profile(&profile),
        Err(ProfileError::UnsupportedProfile)
    );
}
