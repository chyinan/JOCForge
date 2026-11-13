use jocforge_domain::{
    codec::{
        bap::{
            FLAT7X_D0_BAP_BIN_COUNT, FLAT7X_D0_BAP_BLOCK_COUNT, FLAT7X_D0_BAP_CHANNEL_COUNT,
            Flat7xD0BapFrameInput, Flat7xD0BapParameters, allocate_flat7x_d0_bap_frame,
        },
        flat7x_d0_exponents::prepare_flat7x_d0_d15_exponent_row,
        flat7x_d0_mantissa::{
            FLAT7X_D0_MANTISSA_QUANTIZER_POLICY, Flat7xD0AudioBlockIndex, Flat7xD0MantissaInput,
            pack_flat7x_d0_mantissa_block,
        },
        flat7x_d0_spectrum::{
            D0_PCM_SAMPLES_PER_BLOCK, Flat7xD0PcmBlockInput, Flat7xD0PcmWindow,
            transform_flat7x_d0_pcm_block,
        },
        grouped_quantizer::MantissaChannel,
        mantissa::quantize_mantissa,
    },
    transform::math_generated_parameters::MathGeneratedTransformParameters,
};

fn source_block() -> Flat7xD0PcmBlockInput {
    let mut channels = [Flat7xD0PcmWindow {
        previous_samples: [0.0; D0_PCM_SAMPLES_PER_BLOCK],
        new_samples: [0.0; D0_PCM_SAMPLES_PER_BLOCK],
    }; FLAT7X_D0_BAP_CHANNEL_COUNT];
    for (channel, level) in [0.25_f32, -0.1875, 0.3125, -0.125].into_iter().enumerate() {
        channels[channel].new_samples[channel] = level;
    }
    Flat7xD0PcmBlockInput { channels }
}

fn prepared_inputs(
    candidate: u16,
) -> (
    jocforge_domain::codec::flat7x_d0_spectrum::Flat7xD0SpectrumBlock,
    [jocforge_domain::codec::flat7x_d0_exponents::Flat7xD0ExponentRow; FLAT7X_D0_BAP_CHANNEL_COUNT],
    jocforge_domain::codec::bap::Flat7xD0BapFrame,
) {
    let parameters = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
    let spectrum = transform_flat7x_d0_pcm_block(&source_block(), &parameters).unwrap();
    let coefficients = spectrum.coefficient_bits_by_channel();
    let exponent_rows = std::array::from_fn(|channel| {
        prepare_flat7x_d0_d15_exponent_row(&coefficients[channel]).unwrap()
    });
    let decoded_exponents_by_block = std::array::from_fn(|_| {
        std::array::from_fn(|channel| *exponent_rows[channel].decoded_exponents())
    });
    let bap_parameters = Flat7xD0BapParameters::etsi_ts_102_366_v1_4_1();
    let bap_frame = allocate_flat7x_d0_bap_frame(
        &bap_parameters,
        &Flat7xD0BapFrameInput {
            decoded_exponents_by_block: &decoded_exponents_by_block,
            candidate,
            slow_decay_code: 2,
            fast_decay_code: 1,
            slow_gain_code: 1,
            db_per_bit_codes: [2, 2, 2, 2, 2, 3],
            floor_code: 7,
            fast_gain_code: 0,
            caller_all_active_fine_zero_by_block: [candidate.trailing_zeros() >= 4;
                FLAT7X_D0_BAP_BLOCK_COUNT],
        },
    )
    .unwrap();
    (spectrum, exponent_rows, bap_frame)
}

#[test]
fn d0_mantissa_uses_decoder_visible_exponents_for_each_coded_row() {
    let (spectrum, exponent_rows, bap_frame) = prepared_inputs(1_023);
    let input = Flat7xD0MantissaInput {
        spectrum: &spectrum,
        exponent_rows: &exponent_rows,
        bap_frame: &bap_frame,
        block_index: Flat7xD0AudioBlockIndex::new(0).unwrap(),
    };

    let output = pack_flat7x_d0_mantissa_block(&input).unwrap();
    let expected_channels = [
        MantissaChannel::LeftSurround,
        MantissaChannel::RightSurround,
        MantissaChannel::LeftRearSurround,
        MantissaChannel::RightRearSurround,
    ];
    assert_eq!(output.items().len(), 4 * FLAT7X_D0_BAP_BIN_COUNT);
    assert_eq!(
        output.quantizer_policy(),
        FLAT7X_D0_MANTISSA_QUANTIZER_POLICY
    );
    assert!(!output.production_allowed());

    let coefficients = spectrum.coefficient_bits_by_channel();
    let bap_rows = bap_frame.blocks()[0].rows();
    for (channel, expected_channel) in expected_channels.into_iter().enumerate() {
        for bin in 0..FLAT7X_D0_BAP_BIN_COUNT {
            let item = &output.items()[channel * FLAT7X_D0_BAP_BIN_COUNT + bin];
            assert_eq!(item.channel(), expected_channel);
            assert_eq!(usize::from(item.bin()), bin);
            let expected = quantize_mantissa(
                coefficients[channel][bin],
                i16::from(exponent_rows[channel].decoded_exponents()[bin]),
                bap_rows[channel][bin],
            )
            .unwrap();
            assert_eq!(
                (item.code(), item.width()),
                (expected.value(), expected.width())
            );
        }
    }
    assert_eq!(
        output.bit_count(),
        output
            .items()
            .iter()
            .map(|item| u64::from(item.width()))
            .sum::<u64>()
    );
}

#[test]
fn d0_mantissa_groups_quantize_with_the_selected_block_bap_rows() {
    let (spectrum, exponent_rows, bap_frame) = prepared_inputs(512);
    let input = Flat7xD0MantissaInput {
        spectrum: &spectrum,
        exponent_rows: &exponent_rows,
        bap_frame: &bap_frame,
        block_index: Flat7xD0AudioBlockIndex::new(5).unwrap(),
    };

    let output = pack_flat7x_d0_mantissa_block(&input).unwrap();

    let selected_bap_rows = bap_frame.blocks()[5].rows();
    for (channel, row) in selected_bap_rows.iter().enumerate() {
        for (bin, bap) in row.iter().copied().enumerate() {
            assert_eq!(
                output.items()[channel * FLAT7X_D0_BAP_BIN_COUNT + bin].bap(),
                bap
            );
        }
    }
    assert!(output.items().iter().any(|item| item.bap() > 0));
    assert!(!output.grouped_packs().is_empty());
    assert!(
        output
            .grouped_packs()
            .iter()
            .all(|group| matches!(group.bap(), 1 | 2 | 4))
    );
    assert_eq!(
        output.packed_bytes().len(),
        usize::try_from(output.bit_count().div_ceil(8)).unwrap()
    );
    assert!(output.grouped_packs().iter().all(|group| {
        u64::from(group.width()) <= output.bit_count().saturating_sub(group.start_bit())
    }));
    assert_eq!(Flat7xD0AudioBlockIndex::new(6), None);
}
