use std::convert::Infallible;

use jocforge_domain::codec::bap::{
    BapBlock, BapFrame, BapFrameInput, BapOutputError, BapProfileInput, BapRowInput,
    CONVERTER_FAST_GAIN_CODE, MAIN_FAST_GAIN_CODE, allocate_bap_frame, allocate_bap_row,
    load_bap_parameters,
};
use jocforge_domain::codec::converter_search::{
    ConverterFrameSearchError, ConverterSearchInput, converter_mantissa_costs,
    search_converter_offset, search_converter_offset_from_bap,
    search_converter_offset_from_bap_frame,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const INPUTS_SHA256: &str = "a2291556af70fb0b7b50144cf2603ec5a4b52352b1ad03c9c234aeb7209e576d";
const INPUTS: &str = include_str!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-ALLOCATION-PRODUCER-SEARCH-V1/INPUT-ONLY-VECTORS.json"
);
const PUBLIC_BAP_PROVIDER: &[u8] = include_bytes!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-PUBLIC-BAP-PROVIDER-V1/public-bap-48k-v1.json"
);

fn empty_block() -> BapBlock {
    BapBlock::new([[0_u8; 217]; 5], [0_u8; 7]).expect("zero BAP rows are valid")
}

fn approved_inputs() -> Value {
    assert_eq!(
        format!("{:x}", Sha256::digest(INPUTS.as_bytes())),
        INPUTS_SHA256
    );
    serde_json::from_str(INPUTS).unwrap()
}

fn block_with_cells(cells: &Value) -> BapBlock {
    let mut fullband = [[0_u8; 217]; 5];
    let mut lfe = [0_u8; 7];
    for cell in cells.as_array().expect("cells are an array") {
        let cell = cell.as_array().expect("cell is a triple");
        let channel = usize::try_from(cell[0].as_u64().expect("channel is unsigned")).unwrap();
        let bin = usize::try_from(cell[1].as_u64().expect("bin is unsigned")).unwrap();
        let code = u8::try_from(cell[2].as_u64().expect("BAP is unsigned")).unwrap();
        if channel < 5 {
            fullband[channel][bin] = code;
        } else {
            lfe[bin] = code;
        }
    }
    BapBlock::new(fullband, lfe).expect("approved cost-layout BAP codes are in range")
}

fn converter_input() -> ConverterSearchInput {
    ConverterSearchInput {
        header_cost: 91,
        fixed_costs: [2_693, 35, 35, 35, 35, 35],
        auxiliary_cost: 1,
    }
}

fn bap_frame_for_candidate(candidate: u16, fast_gain_code: u8) -> BapFrame {
    let fullband_exponents = [[[24_u8; 217]; 5]; 6];
    let lfe_exponents = [[24_u8; 7]; 6];
    let decoded_exponents_by_block = std::array::from_fn(|block| {
        [
            fullband_exponents[block][0].as_slice(),
            fullband_exponents[block][1].as_slice(),
            fullband_exponents[block][2].as_slice(),
            fullband_exponents[block][3].as_slice(),
            fullband_exponents[block][4].as_slice(),
            lfe_exponents[block].as_slice(),
        ]
    });
    let input = BapFrameInput {
        decoded_exponents_by_block: &decoded_exponents_by_block,
        end_exclusive_by_channel: [217, 217, 217, 217, 217, 7],
        candidate,
        slow_decay_code: 2,
        fast_decay_code: 1,
        slow_gain_code: 1,
        db_per_bit_codes: [2; 6],
        floor_code: 7,
        fast_gain_code,
        caller_all_active_fine_zero_by_block: [candidate.trailing_zeros() >= 4; 6],
        profile: BapProfileInput::selected(),
    };
    allocate_bap_frame(Some(PUBLIC_BAP_PROVIDER), &input)
        .expect("the selected profile and generated exponent rows are valid")
}

fn bap_row_input(
    decoded_exponents: &[u8],
    end_exclusive: usize,
    candidate: u16,
    fast_gain_code: u8,
    caller_all_active_fine_zero: bool,
) -> BapRowInput<'_> {
    BapRowInput {
        decoded_exponents,
        end_exclusive,
        candidate,
        slow_decay_code: 2,
        fast_decay_code: 1,
        slow_gain_code: 1,
        db_per_bit_code: 2,
        floor_code: 7,
        fast_gain_code,
        caller_all_active_fine_zero,
        profile: BapProfileInput::selected(),
    }
}

fn recipe_exponents(id: &str, end: usize) -> Vec<u8> {
    match id {
        "lfe_floor0" => vec![0; end],
        "fbw_const6" => vec![6; end],
        "fbw_const12" => vec![12; end],
        "fbw_ramp24" => (0..end)
            .map(|bin| u8::try_from(bin % 25).expect("the ramp exponent is at most 24"))
            .collect(),
        _ => panic!("unexpected approved exponent recipe {id}"),
    }
}

#[test]
fn bap_provider_is_hash_bound_and_uses_the_approved_error_labels() {
    assert!(load_bap_parameters(Some(PUBLIC_BAP_PROVIDER)).is_ok());
    assert_eq!(
        load_bap_parameters(None).unwrap_err().code(),
        "PARAMETER_UNAVAILABLE"
    );

    let mut changed_provider = PUBLIC_BAP_PROVIDER.to_vec();
    changed_provider[0] = b'[';
    assert_eq!(
        load_bap_parameters(Some(&changed_provider))
            .unwrap_err()
            .code(),
        "BAP_PARAMETER_ERROR"
    );
}

#[test]
fn f12a_runs_all_approved_input_only_recipes_without_promoting_expected_outputs() {
    let inputs = approved_inputs();
    let cases = &inputs["production_bap_and_converter_inputs"]["BAP_cases"];
    let candidate_values = cases["candidate"].as_array().unwrap();
    let gain_values = cases["fast_gain_code"].as_array().unwrap();
    let recipes = cases["decoded_exponent_recipes"].as_array().unwrap();
    let mut case_count = 0;

    for recipe in recipes {
        let id = recipe["id"].as_str().unwrap();
        let end = usize::try_from(recipe["end"].as_u64().unwrap()).unwrap();
        let exponents = recipe_exponents(id, end);
        for candidate_value in candidate_values {
            let candidate = u16::try_from(candidate_value.as_u64().unwrap()).unwrap();
            let caller_context = candidate.trailing_zeros() >= 4;
            for gain_value in gain_values {
                let gain = u8::try_from(gain_value.as_u64().unwrap()).unwrap();
                let output = allocate_bap_row(
                    Some(PUBLIC_BAP_PROVIDER),
                    &bap_row_input(&exponents, end, candidate, gain, caller_context),
                )
                .unwrap();
                assert_eq!(output.len(), end, "{id}, q={candidate}, gain={gain}");
                assert!(output.iter().all(|code| *code <= 15));
                case_count += 1;
            }
        }
    }
    assert_eq!(case_count, 56);
}

#[test]
fn f12a_preserves_bypass_context_and_main_converter_gain_distinction() {
    let zero_exponents = [0_u8; 7];
    let bypass = allocate_bap_row(
        Some(PUBLIC_BAP_PROVIDER),
        &bap_row_input(&zero_exponents, 7, 0, CONVERTER_FAST_GAIN_CODE, true),
    )
    .unwrap();
    assert_eq!(bypass, [0_u8; 7]);

    let no_bypass = allocate_bap_row(
        Some(PUBLIC_BAP_PROVIDER),
        &bap_row_input(&zero_exponents, 7, 0, CONVERTER_FAST_GAIN_CODE, false),
    )
    .unwrap();
    assert_eq!(no_bypass.len(), 7);

    let exponents = [12_u8; 217];
    let main = allocate_bap_row(
        Some(PUBLIC_BAP_PROVIDER),
        &bap_row_input(&exponents, 217, 400, MAIN_FAST_GAIN_CODE, true),
    )
    .unwrap();
    let converter = allocate_bap_row(
        Some(PUBLIC_BAP_PROVIDER),
        &bap_row_input(&exponents, 217, 400, CONVERTER_FAST_GAIN_CODE, true),
    )
    .unwrap();
    assert_ne!(main, converter);
}

#[test]
fn f12a_frame_preserves_six_block_and_six_channel_axes() {
    let mut fullband_exponents = [[[0_u8; 217]; 5]; 6];
    let mut lfe_exponents = [[0_u8; 7]; 6];
    for (fullband_block, lfe_block) in fullband_exponents.iter_mut().zip(&mut lfe_exponents) {
        for (channel, row) in fullband_block.iter_mut().enumerate() {
            let level = [6_u8, 12, 24, 0, 6][channel];
            row.fill(level);
        }
        lfe_block.fill(0);
    }
    let decoded_exponents_by_block = std::array::from_fn(|block| {
        [
            fullband_exponents[block][0].as_slice(),
            fullband_exponents[block][1].as_slice(),
            fullband_exponents[block][2].as_slice(),
            fullband_exponents[block][3].as_slice(),
            fullband_exponents[block][4].as_slice(),
            lfe_exponents[block].as_slice(),
        ]
    });
    let input = BapFrameInput {
        decoded_exponents_by_block: &decoded_exponents_by_block,
        end_exclusive_by_channel: [217, 217, 217, 217, 217, 7],
        candidate: 400,
        slow_decay_code: 2,
        fast_decay_code: 1,
        slow_gain_code: 1,
        db_per_bit_codes: [0, 1, 2, 3, 0, 1],
        floor_code: 7,
        fast_gain_code: MAIN_FAST_GAIN_CODE,
        caller_all_active_fine_zero_by_block: [true; 6],
        profile: BapProfileInput::selected(),
    };

    let frame = allocate_bap_frame(Some(PUBLIC_BAP_PROVIDER), &input).unwrap();
    assert_eq!(frame.candidate_q(), 400);
    for (block, (output_block, block_rows)) in frame
        .blocks()
        .iter()
        .zip(decoded_exponents_by_block.iter())
        .enumerate()
    {
        for (channel, decoded_exponents) in block_rows.iter().take(5).enumerate() {
            let row = allocate_bap_row(
                Some(PUBLIC_BAP_PROVIDER),
                &BapRowInput {
                    decoded_exponents,
                    end_exclusive: 217,
                    candidate: 400,
                    slow_decay_code: 2,
                    fast_decay_code: 1,
                    slow_gain_code: 1,
                    db_per_bit_code: input.db_per_bit_codes[block],
                    floor_code: 7,
                    fast_gain_code: MAIN_FAST_GAIN_CODE,
                    caller_all_active_fine_zero: true,
                    profile: BapProfileInput::selected(),
                },
            )
            .unwrap();
            assert_eq!(output_block.fullband_rows()[channel].as_slice(), row);
        }
        let lfe = allocate_bap_row(
            Some(PUBLIC_BAP_PROVIDER),
            &BapRowInput {
                decoded_exponents: block_rows[5],
                end_exclusive: 7,
                candidate: 400,
                slow_decay_code: 2,
                fast_decay_code: 1,
                slow_gain_code: 1,
                db_per_bit_code: input.db_per_bit_codes[block],
                floor_code: 7,
                fast_gain_code: MAIN_FAST_GAIN_CODE,
                caller_all_active_fine_zero: true,
                profile: BapProfileInput::selected(),
            },
        )
        .unwrap();
        assert_eq!(output_block.lfe_row().as_slice(), lfe);
    }

    let short_lfe = [0_u8; 6];
    let invalid_rows_by_block = std::array::from_fn(|block| {
        let mut rows = decoded_exponents_by_block[block];
        if block == 5 {
            rows[5] = &short_lfe;
        }
        rows
    });
    let invalid_input = BapFrameInput {
        decoded_exponents_by_block: &invalid_rows_by_block,
        ..input
    };
    assert_eq!(
        allocate_bap_frame(Some(PUBLIC_BAP_PROVIDER), &invalid_input)
            .unwrap_err()
            .code(),
        "BAP_INPUT_ERROR"
    );
    let context_conflict = BapFrameInput {
        caller_all_active_fine_zero_by_block: [false; 6],
        ..input
    };
    assert_eq!(
        allocate_bap_frame(Some(PUBLIC_BAP_PROVIDER), &context_conflict)
            .unwrap_err()
            .code(),
        "BAP_INPUT_ERROR"
    );
}

#[test]
fn f12a_rejections_keep_the_specified_local_error_categories() {
    let valid_exponents = [12_u8; 7];
    let mut input = bap_row_input(&valid_exponents, 7, 16, MAIN_FAST_GAIN_CODE, false);
    assert_eq!(
        allocate_bap_row(None, &input).unwrap_err().code(),
        "PARAMETER_UNAVAILABLE"
    );

    input.profile.sample_rate_hz = 0;
    assert_eq!(
        allocate_bap_row(Some(PUBLIC_BAP_PROVIDER), &input)
            .unwrap_err()
            .code(),
        "BAP_INPUT_ERROR"
    );
    input.profile.sample_rate_hz = 44_100;
    assert_eq!(
        allocate_bap_row(Some(PUBLIC_BAP_PROVIDER), &input)
            .unwrap_err()
            .code(),
        "UNSUPPORTED_PROFILE"
    );
    input.profile = BapProfileInput {
        coupling: true,
        ..BapProfileInput::selected()
    };
    assert_eq!(
        allocate_bap_row(Some(PUBLIC_BAP_PROVIDER), &input)
            .unwrap_err()
            .code(),
        "UNSUPPORTED_PROFILE"
    );
    input.profile = BapProfileInput {
        delta: true,
        ..BapProfileInput::selected()
    };
    assert_eq!(
        allocate_bap_row(Some(PUBLIC_BAP_PROVIDER), &input)
            .unwrap_err()
            .code(),
        "UNSUPPORTED_PROFILE"
    );
    input.profile = BapProfileInput {
        aht: true,
        ..BapProfileInput::selected()
    };
    assert_eq!(
        allocate_bap_row(Some(PUBLIC_BAP_PROVIDER), &input)
            .unwrap_err()
            .code(),
        "UNSUPPORTED_PROFILE"
    );
    input.profile = BapProfileInput::selected();
    input.candidate = 1_024;
    assert_eq!(
        allocate_bap_row(Some(PUBLIC_BAP_PROVIDER), &input)
            .unwrap_err()
            .code(),
        "BAP_INPUT_ERROR"
    );
    input.candidate = 17;
    input.caller_all_active_fine_zero = true;
    assert_eq!(
        allocate_bap_row(Some(PUBLIC_BAP_PROVIDER), &input)
            .unwrap_err()
            .code(),
        "BAP_INPUT_ERROR"
    );
    input.caller_all_active_fine_zero = false;
    input.fast_gain_code = 8;
    assert_eq!(
        allocate_bap_row(Some(PUBLIC_BAP_PROVIDER), &input)
            .unwrap_err()
            .code(),
        "BAP_INPUT_ERROR"
    );
    input.fast_gain_code = MAIN_FAST_GAIN_CODE;
    input.db_per_bit_code = 4;
    assert_eq!(
        allocate_bap_row(Some(PUBLIC_BAP_PROVIDER), &input)
            .unwrap_err()
            .code(),
        "BAP_INPUT_ERROR"
    );
    input.db_per_bit_code = 0;
    input.end_exclusive = 6;
    assert_eq!(
        allocate_bap_row(Some(PUBLIC_BAP_PROVIDER), &input)
            .unwrap_err()
            .code(),
        "UNSUPPORTED_PROFILE"
    );
    input.end_exclusive = 7;
    let excessive_exponents = [25_u8; 7];
    input.decoded_exponents = &excessive_exponents;
    assert_eq!(
        allocate_bap_row(Some(PUBLIC_BAP_PROVIDER), &input)
            .unwrap_err()
            .code(),
        "BAP_INPUT_ERROR"
    );
    input.decoded_exponents = &valid_exponents[..6];
    assert_eq!(
        allocate_bap_row(Some(PUBLIC_BAP_PROVIDER), &input)
            .unwrap_err()
            .code(),
        "BAP_INPUT_ERROR"
    );
}

#[test]
fn converter_cost_joins_grouped_bap_across_channels_and_resets_each_block() {
    let inputs = approved_inputs();
    let cases = inputs["typed_grouped_and_residual_boundaries"]["cross_channel_cases"]
        .as_array()
        .unwrap();

    let expected_costs = [5_i16, 7, 7];
    for (case, expected) in cases.iter().zip(expected_costs) {
        let block = block_with_cells(&case["nonzero_cells"]);
        let costs = converter_mantissa_costs(&[block; 6]);
        assert_eq!(costs, [expected; 6]);
    }

    let first_case = block_with_cells(&cases[0]["nonzero_cells"]);
    let mut blocks = [empty_block(); 6];
    blocks[0] = first_case;
    blocks[1] = first_case;
    assert_eq!(converter_mantissa_costs(&blocks), [5, 5, 0, 0, 0, 0]);
}

#[test]
fn converter_cost_preserves_the_approved_nonmonotone_histogram_example() {
    let inputs = approved_inputs();
    let boundary = &inputs["typed_grouped_and_residual_boundaries"]["cost_nonmonotone"];
    let before = block_with_cells(&boundary["before_nonzero_cells"]);
    let after = block_with_cells(&boundary["after_nonzero_cells"]);

    assert_eq!(converter_mantissa_costs(&[before; 6]), [12; 6]);
    assert_eq!(converter_mantissa_costs(&[after; 6]), [7; 6]);
}

#[test]
fn converter_residual_accepts_zero_and_rejects_one_bit_over() {
    let input = converter_input();
    for (first_block_cost, expected) in [
        (9_980_i16, Some(671_u16)),
        (9_981, Some(671)),
        (9_982, Some(0)),
    ] {
        let result = search_converter_offset(&input, |_| [first_block_cost, 0, 0, 0, 0, 0]);
        assert_eq!(result.best_offset(), expected);
    }
}

#[test]
fn converter_bap_search_computes_costs_before_applying_the_offset_cap() {
    let input = ConverterSearchInput {
        header_cost: 91,
        fixed_costs: [0; 6],
        auxiliary_cost: 0,
    };
    let mut produced_candidates = Vec::new();
    let result = search_converter_offset_from_bap(&input, |candidate| {
        produced_candidates.push(candidate);
        Ok::<_, Infallible>([empty_block(); 6])
    })
    .unwrap();

    assert_eq!(result.candidate_order(), produced_candidates.as_slice());
    assert!(produced_candidates.contains(&672));
    assert_eq!(result.best_offset(), Some(671));
}

#[test]
fn converter_frame_search_rejects_a_candidate_tag_mismatch_before_costing() {
    let input = ConverterSearchInput {
        header_cost: 91,
        fixed_costs: [0; 6],
        auxiliary_cost: 0,
    };
    let mismatched_frame = bap_frame_for_candidate(0, CONVERTER_FAST_GAIN_CODE);
    let result = search_converter_offset_from_bap_frame(&input, |_| {
        Ok::<_, Infallible>(mismatched_frame.clone())
    });

    assert_eq!(
        result.unwrap_err(),
        ConverterFrameSearchError::CandidateMismatch {
            requested: 512,
            received: 0,
        }
    );
}

#[test]
fn converter_frame_search_uses_the_matching_candidate_bap_costs() {
    let input = ConverterSearchInput {
        header_cost: 91,
        fixed_costs: [0; 6],
        auxiliary_cost: 0,
    };
    let mut produced_candidates = Vec::new();
    let result = search_converter_offset_from_bap_frame(&input, |candidate| {
        let candidate = u16::try_from(candidate).unwrap();
        produced_candidates.push(candidate);
        Ok::<_, Infallible>(bap_frame_for_candidate(candidate, CONVERTER_FAST_GAIN_CODE))
    })
    .unwrap();

    let searched_candidates = result
        .candidate_order()
        .iter()
        .map(|candidate| u16::try_from(*candidate).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(produced_candidates, searched_candidates);
}

#[test]
fn converter_bap_search_uses_the_zero_mantissa_no_best_fallback() {
    let input = ConverterSearchInput {
        header_cost: 91,
        fixed_costs: [0; 6],
        auxiliary_cost: 0,
    };
    let maximum_cost_block = BapBlock::new([[15_u8; 217]; 5], [15_u8; 7]).unwrap();
    let mut produced_candidates = Vec::new();
    let result = search_converter_offset_from_bap(&input, |candidate| {
        produced_candidates.push(candidate);
        Ok::<_, Infallible>([maximum_cost_block; 6])
    })
    .unwrap();

    assert_eq!(result.best_offset(), Some(0));
    assert_eq!(produced_candidates.as_slice(), result.candidate_order());
    assert!(!produced_candidates.contains(&0));
}

#[test]
fn converter_bap_block_rejects_codes_outside_the_approved_range() {
    let mut fullband = [[0_u8; 217]; 5];
    fullband[2][4] = 16;
    assert_eq!(
        BapBlock::new(fullband, [0_u8; 7]),
        Err(BapOutputError::CodeOutOfRange {
            channel: 2,
            bin: 4,
            code: 16,
        })
    );
}
