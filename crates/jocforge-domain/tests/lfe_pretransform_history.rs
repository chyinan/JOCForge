use jocforge_domain::transform::lfe_transform::{
    LfeSelectorZeroHistory, SELECTOR_ZERO_LFE_RECURRENCE_PRIMARY_COEFFICIENT_BITS,
    SELECTOR_ZERO_LFE_RECURRENCE_SECONDARY_COEFFICIENT_BITS, apply_selector_zero_lfe_recurrence,
    pretransform_selector_zero_lfe_block, project_companion_lfe_terminal_window,
    project_main_lfe_terminal_window, transform_lfe_16point_block,
    transform_selector_zero_lfe_block,
};

const WASHED_LFE_RECURRENCE_VECTORS: &str = include_str!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/DEE-LFE-RECURRENCE-COEFFICIENT-VECTORS-V1/INPUT-ONLY-VECTORS.json"
);

fn washed_recurrence_table(name: &str) -> [u32; 128] {
    let document: serde_json::Value = serde_json::from_str(WASHED_LFE_RECURRENCE_VECTORS)
        .unwrap_or_else(|error| panic!("invalid washed LFE recurrence vectors: {error}"));
    let vectors = document
        .get("vectors")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("washed recurrence vector list is missing"));
    let vector = vectors
        .iter()
        .find(|vector| vector.get("name").and_then(serde_json::Value::as_str) == Some(name))
        .unwrap_or_else(|| panic!("washed recurrence vector {name} is missing"));
    let elements = vector
        .get("elements")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("washed recurrence vector {name} has no elements"));
    assert_eq!(
        elements.len(),
        128,
        "washed recurrence vector {name} length"
    );
    let mut words = [0_u32; 128];
    for (index, element) in elements.iter().enumerate() {
        let raw = element
            .get("raw_word_le_hex")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("washed recurrence vector {name} word {index} is missing"));
        let digits = raw.strip_prefix("0x").unwrap_or(raw);
        words[index] = u32::from_str_radix(digits, 16).unwrap_or_else(|error| {
            panic!("invalid washed recurrence word {name}[{index}]: {error}")
        });
    }
    words
}

#[test]
fn selector_zero_recurrence_constants_match_wash_exact_words() {
    assert_eq!(
        SELECTOR_ZERO_LFE_RECURRENCE_PRIMARY_COEFFICIENT_BITS,
        washed_recurrence_table("zero_branch_table_a")
    );
    assert_eq!(
        SELECTOR_ZERO_LFE_RECURRENCE_SECONDARY_COEFFICIENT_BITS,
        washed_recurrence_table("zero_branch_table_b")
    );
}

#[test]
fn selector_zero_pretransform_one_hot_vectors_match_washed_values() {
    let history = LfeSelectorZeroHistory::new_record_zeroed();
    let mut input = [0_u32; 256];

    input[0] = 0x3f80_0000;
    let main_first = pretransform_selector_zero_lfe_block(&history, &input);
    assert_eq!(main_first.main_work_bits()[128], 0x3f80_0000);
    assert_eq!(
        main_first.after_history().main_history_bits()[64],
        0xb90e_9922
    );
    assert_eq!(main_first.companion_work_bits()[129], 0x3f80_0000);
    assert_eq!(
        main_first.after_history().companion_history_bits()[64],
        0x390e_9922
    );

    input = [0; 256];
    input[255] = 0x3f80_0000;
    let last_sample = pretransform_selector_zero_lfe_block(&history, &input);
    assert_eq!(last_sample.main_work_bits()[128], 0x390e_9922);
    assert_eq!(
        last_sample.after_history().main_history_bits()[64],
        0x3f80_0000
    );
    assert_eq!(last_sample.companion_work_bits()[129], 0xb90e_9922);
    assert_eq!(
        last_sample.after_history().companion_history_bits()[64],
        0x3f80_0000
    );
}

#[test]
fn history_is_carried_between_blocks_and_not_reset_per_block() {
    let mut input = [0_u32; 256];
    input[0] = 0x3f80_0000;
    let first =
        pretransform_selector_zero_lfe_block(&LfeSelectorZeroHistory::new_record_zeroed(), &input);

    let second = pretransform_selector_zero_lfe_block(first.after_history(), &[0; 256]);

    assert_eq!(second.main_work_bits()[129], 0xb90e_9922);
    assert_eq!(second.companion_work_bits()[128], 0x390e_9922);
    assert_eq!(second.after_history().main_history_bits()[64], 0);
    assert_eq!(second.after_history().companion_history_bits()[64], 0);
}

#[test]
fn selector_zero_recurrence_group0_matches_washed_input_only_vector() {
    let mut input = [0_u32; 256];
    input[..4].copy_from_slice(&[0x3f80_0000, 0, 0, 0]);
    let mut coefficient_a = [0_u32; 128];
    let mut coefficient_b = [0_u32; 128];
    coefficient_a[0] = 0xbf7f_ffec;
    coefficient_b[0] = 0xbac9_0fd5;

    let output = apply_selector_zero_lfe_recurrence(&input, &coefficient_a, &coefficient_b);

    assert_eq!(&output[..4], &[0x3f7f_ffec, 0xbac9_0fd5, 0, 0]);
    assert!(output[4..].iter().all(|word| *word == 0));
}

#[test]
fn selector_zero_lfe_block_composes_the_staged_transforms() {
    let mut input = [0_u32; 256];
    input[0] = 0x3f80_0000;
    let coefficient_a = washed_recurrence_table("zero_branch_table_a");
    let coefficient_b = washed_recurrence_table("zero_branch_table_b");

    let history = LfeSelectorZeroHistory::new_record_zeroed();
    let pretransform = pretransform_selector_zero_lfe_block(&history, &input);
    let main_recurrence = apply_selector_zero_lfe_recurrence(
        pretransform.main_work_bits(),
        &coefficient_a,
        &coefficient_b,
    );
    let companion_recurrence = apply_selector_zero_lfe_recurrence(
        pretransform.companion_work_bits(),
        &coefficient_a,
        &coefficient_b,
    );
    let main_sixteen = transform_lfe_16point_block(&main_recurrence).map(f32::from_bits);
    let companion_sixteen = transform_lfe_16point_block(&companion_recurrence).map(f32::from_bits);
    let expected_main = project_main_lfe_terminal_window(&main_sixteen).map(f32::to_bits);
    let expected_companion =
        project_companion_lfe_terminal_window(&companion_sixteen).map(f32::to_bits);

    let result = transform_selector_zero_lfe_block(&history, &input);

    assert_eq!(result.main_terminal_bits(), &expected_main);
    assert_eq!(result.companion_terminal_bits(), &expected_companion);
    assert_eq!(result.after_history(), pretransform.after_history());
}
