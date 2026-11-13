use jocforge_domain::{
    codec::{
        bap::{
            FLAT7X_D0_BAP_BIN_COUNT, FLAT7X_D0_BAP_BLOCK_COUNT, FLAT7X_D0_BAP_CHANNEL_COUNT,
            Flat7xD0BapFrameInput, Flat7xD0BapParameters, allocate_flat7x_d0_bap_frame,
        },
        flat7x_d0_audio_frame::{Flat7xD0AudioFrameInput, produce_flat7x_d0_audio_frame},
        flat7x_d0_exponents::{Flat7xD0ExponentRow, prepare_flat7x_d0_d15_exponent_row},
        flat7x_d0_pcm_source::{
            Flat7xD0OverlapHistory, Flat7xD0PcmFrameInput, Flat7xD0PcmFrameSettings,
            produce_flat7x_d0_frame_from_pcm_source,
        },
        flat7x_d0_spectrum::{
            D0_PCM_SAMPLES_PER_BLOCK, Flat7xD0PcmBlockInput, Flat7xD0PcmWindow,
            Flat7xD0SpectrumBlock, transform_flat7x_d0_pcm_block,
        },
        frame_builder::Flat7xD0BsiStructure,
    },
    integrity::crc16::eac3_crc16,
    profile::Flat7xI0D0TargetPlan,
    transform::math_generated_parameters::MathGeneratedTransformParameters,
};

fn read_bits(bytes: &[u8], start_bit: u32, width: u8) -> u32 {
    (0..u32::from(width)).fold(0_u32, |value, offset| {
        let bit_offset = start_bit + offset;
        let byte = bytes[usize::try_from(bit_offset / 8).unwrap()];
        let bit = (byte >> (7 - (bit_offset % 8))) & 1;
        (value << 1) | u32::from(bit)
    })
}

fn synthetic_six_block_inputs() -> ([Flat7xD0SpectrumBlock; 6], [[Flat7xD0ExponentRow; 4]; 6]) {
    let parameters = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
    let mut previous = [[0.0_f32; D0_PCM_SAMPLES_PER_BLOCK]; 4];
    let mut spectra = Vec::with_capacity(6);
    let mut exponents = Vec::with_capacity(6);

    for block in 0..6 {
        let channels = std::array::from_fn(|channel| {
            let mut new_samples = [0.0_f32; D0_PCM_SAMPLES_PER_BLOCK];
            new_samples[(block * 19 + channel * 31) % D0_PCM_SAMPLES_PER_BLOCK] =
                [0.75_f32, -0.625, 0.5, -0.375][channel];
            new_samples[(block * 7 + channel * 11 + 97) % D0_PCM_SAMPLES_PER_BLOCK] =
                [-0.25_f32, 0.3125, -0.1875, 0.125][channel];
            Flat7xD0PcmWindow {
                previous_samples: previous[channel],
                new_samples,
            }
        });
        let spectrum =
            transform_flat7x_d0_pcm_block(&Flat7xD0PcmBlockInput { channels }, &parameters)
                .unwrap();
        let block_exponents = std::array::from_fn(|channel| {
            prepare_flat7x_d0_d15_exponent_row(&spectrum.coefficient_bits_by_channel()[channel])
                .unwrap()
        });
        previous = channels.map(|channel| channel.new_samples);
        spectra.push(spectrum);
        exponents.push(block_exponents);
    }

    (spectra.try_into().unwrap(), exponents.try_into().unwrap())
}

#[test]
fn d0_frame_search_produces_one_complete_crc_valid_six_block_candidate() {
    let (spectrum_by_block, exponent_rows_by_block) = synthetic_six_block_inputs();
    let bap_parameters = Flat7xD0BapParameters::etsi_ts_102_366_v1_4_1();
    let emdf_container = [0x5a_u8];

    let candidate = produce_flat7x_d0_audio_frame(&Flat7xD0AudioFrameInput {
        target: Flat7xI0D0TargetPlan::approved_contract(),
        bsi_structure: Flat7xD0BsiStructure {
            dialnorm: 16,
            compr: None,
            mixing: None,
            block_mix: [None; 6],
            information: None,
            addbsi: None,
        },
        complete_emdf_container: &emdf_container,
        bap_parameters: &bap_parameters,
        spectrum_by_block: &spectrum_by_block,
        exponent_rows_by_block: &exponent_rows_by_block,
        dynamic_range_codes: [None; 6],
    })
    .unwrap();

    assert_eq!(candidate.frame_bytes().len(), 4_096);
    assert!(candidate.receipt().allocation_candidate().value() > 0);
    assert_eq!(
        candidate.receipt().q_search_policy_id(),
        "JOCFORGE_FLAT7X_D0_MAX_ALLOCATED_BITS_Q_HIGH_TIE_V1"
    );
    assert_eq!(candidate.receipt().block_receipts().len(), 6);
    assert_eq!(
        candidate.receipt().allocated_mantissa_bits(),
        candidate
            .receipt()
            .block_receipts()
            .iter()
            .map(|receipt| receipt.mantissa_bits())
            .sum::<u32>()
    );
    assert!(
        candidate
            .receipt()
            .block_receipts()
            .iter()
            .all(|receipt| receipt.nonzero_bap_bins() > 0)
    );
    assert_eq!(
        candidate
            .receipt()
            .frame_assembly()
            .audio_frame_controls()
            .allocation_candidate(),
        candidate.receipt().allocation_candidate()
    );
    let frame_controls = candidate.receipt().frame_assembly().audio_frame_controls();
    assert_eq!(frame_controls.bit_length(), 23);
    assert_eq!(
        read_bits(candidate.frame_bytes(), frame_controls.start_bit(), 1),
        1
    );
    assert_eq!(
        read_bits(candidate.frame_bytes(), frame_controls.start_bit() + 12, 6),
        u32::from(candidate.receipt().allocation_candidate().coarse_snr_code())
    );
    assert_eq!(
        read_bits(candidate.frame_bytes(), frame_controls.start_bit() + 18, 4),
        u32::from(candidate.receipt().allocation_candidate().fine_snr_code())
    );
    let first_block = candidate.receipt().frame_assembly().layout().audio_blocks()[0]
        .leading_audio_span()
        .start_bit();
    assert_eq!(read_bits(candidate.frame_bytes(), first_block, 1), 0); // dynrnge absent
    assert_eq!(read_bits(candidate.frame_bytes(), first_block + 1, 4), 0); // four dithflag=0
    assert_eq!(read_bits(candidate.frame_bytes(), first_block + 5, 2), 0); // no SPX/coupling
    for channel in 0..4_u32 {
        assert_eq!(
            read_bits(candidate.frame_bytes(), first_block + 7 + channel * 2, 2),
            1
        );
    }
    assert_eq!(
        candidate
            .receipt()
            .frame_assembly()
            .crc2_integrity()
            .verified_residual(),
        0
    );
    assert_eq!(eac3_crc16(&candidate.frame_bytes()[2..]), 0);
    assert!(!candidate.production_allowed());
}

#[test]
fn d0_pcm_source_feeds_the_complete_frame_candidate_and_keeps_history_with_it() {
    let pcm_rows: [Vec<u32>; 4] = std::array::from_fn(|channel| {
        let mut row = vec![0.0_f32.to_bits(); 1_536];
        for block in 0..6 {
            row[block * 256 + 19 + channel * 7] = [0.1_f32, -0.12, 0.08, -0.09][channel].to_bits();
            row[block * 256 + 129 + channel * 9] =
                [0.04_f32, -0.03, 0.05, -0.045][channel].to_bits();
        }
        row
    });
    let pcm_row_refs = std::array::from_fn(|channel| pcm_rows[channel].as_slice());
    let transform_parameters =
        MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
    let bap_parameters = Flat7xD0BapParameters::etsi_ts_102_366_v1_4_1();
    let emdf_container = [0x5a_u8];

    let candidate = produce_flat7x_d0_frame_from_pcm_source(Flat7xD0PcmFrameInput {
        pcm_rows: pcm_row_refs,
        settings: Flat7xD0PcmFrameSettings {
            before: &Flat7xD0OverlapHistory::cold(),
            transform_parameters: &transform_parameters,
            target: Flat7xI0D0TargetPlan::approved_contract(),
            bsi_structure: Flat7xD0BsiStructure {
                dialnorm: 16,
                compr: None,
                mixing: None,
                block_mix: [None; 6],
                information: None,
                addbsi: None,
            },
            complete_emdf_container: &emdf_container,
            bap_parameters: &bap_parameters,
            dynamic_range_codes: [None; 6],
        },
    })
    .unwrap();

    assert_eq!(candidate.frame().frame_bytes().len(), 4_096);
    assert_eq!(candidate.block_series().spectra().len(), 6);
    assert_eq!(
        candidate.after_history().samples_by_channel()[0],
        pcm_rows[0][1_280..]
    );
    assert!(!candidate.production_allowed());
}

#[test]
fn raw_q_zero_remains_a_valid_zero_bap_input_vector() {
    let (_, exponent_rows_by_block) = synthetic_six_block_inputs();
    let decoded_exponents_by_block = std::array::from_fn(|block| {
        std::array::from_fn(|channel| *exponent_rows_by_block[block][channel].decoded_exponents())
    });
    let parameters = Flat7xD0BapParameters::etsi_ts_102_366_v1_4_1();
    let frame = allocate_flat7x_d0_bap_frame(
        &parameters,
        &Flat7xD0BapFrameInput {
            decoded_exponents_by_block: &decoded_exponents_by_block,
            candidate: 0,
            slow_decay_code: 2,
            fast_decay_code: 1,
            slow_gain_code: 1,
            db_per_bit_codes: [2; FLAT7X_D0_BAP_BLOCK_COUNT],
            floor_code: 7,
            fast_gain_code: 4,
            caller_all_active_fine_zero_by_block: [true; FLAT7X_D0_BAP_BLOCK_COUNT],
        },
    )
    .unwrap();

    assert_eq!(frame.candidate(), 0);
    assert_eq!(
        frame
            .blocks()
            .iter()
            .flat_map(|block| block.rows())
            .flatten()
            .count(),
        FLAT7X_D0_BAP_BLOCK_COUNT * FLAT7X_D0_BAP_CHANNEL_COUNT * FLAT7X_D0_BAP_BIN_COUNT
    );
    assert!(
        frame
            .blocks()
            .iter()
            .flat_map(|block| block.rows())
            .flatten()
            .all(|bap| *bap == 0)
    );
}

#[test]
fn d0_frame_search_rejects_an_empty_emdf_container_before_candidate_search() {
    let (spectrum_by_block, exponent_rows_by_block) = synthetic_six_block_inputs();
    let bap_parameters = Flat7xD0BapParameters::etsi_ts_102_366_v1_4_1();
    let empty_container = [];

    assert!(matches!(
        produce_flat7x_d0_audio_frame(&Flat7xD0AudioFrameInput {
            target: Flat7xI0D0TargetPlan::approved_contract(),
            bsi_structure: Flat7xD0BsiStructure {
                dialnorm: 16,
                compr: None,
                mixing: None,
                block_mix: [None; 6],
                information: None,
                addbsi: None,
            },
            complete_emdf_container: &empty_container,
            bap_parameters: &bap_parameters,
            spectrum_by_block: &spectrum_by_block,
            exponent_rows_by_block: &exponent_rows_by_block,
            dynamic_range_codes: [None; 6],
        }),
        Err(
            jocforge_domain::codec::flat7x_d0_audio_frame::Flat7xD0AudioFrameError::EmdfSkipField(
                _
            )
        )
    ));
}
