use jocforge_domain::transform::secondary_overlap::{
    SecondaryOverlapCarry, SecondaryOverlapError, SecondaryOverlapParameters,
    overlap_secondary_block,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const VERIFICATION: &[u8] = include_bytes!("fixtures/secondary_overlap/overlap-verification.json");
const WINDOW_SHA256: &str = "45e8cbe4c07bd1b59bc009ed6ac77d7911a3a309b01308131da97e640da95b23";

fn document() -> Value {
    serde_json::from_slice(VERIFICATION).expect("approved overlap vectors parse")
}

fn input_words(recipe: usize) -> [u32; 256] {
    match recipe {
        0 => [0; 256],
        1 => {
            let mut words = [0; 256];
            for index in [0, 63, 64, 127, 128, 191, 192, 255] {
                words[index] = 1.0_f32.to_bits();
            }
            words
        }
        2 => std::array::from_fn(|index| {
            let numerator = i16::try_from((37 * index + 11) % 257).unwrap() - 128;
            (f32::from(numerator) / 128.0).to_bits()
        }),
        3 => [0x8000_0000; 256],
        4 => std::array::from_fn(|index| {
            if index % 2 == 0 {
                1.0_f32.to_bits()
            } else {
                (-1.0_f32).to_bits()
            }
        }),
        5 => std::array::from_fn(|index| {
            let numerator = i16::try_from(index).unwrap() - 128;
            (f32::from(numerator) / 256.0).to_bits()
        }),
        _ => panic!("unknown approved overlap recipe"),
    }
}

fn words(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .expect("expected word array")
        .iter()
        .map(|word| u32::try_from(word.as_u64().expect("U32 word")).unwrap())
        .collect()
}

fn hash_words(words: &[u32]) -> String {
    let bytes = words
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>();
    format!("{:x}", Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}

#[test]
fn six_overlap_blocks_match_words_and_continue_only_actual_carry() {
    let parameters = SecondaryOverlapParameters::selected().unwrap();
    let vectors = document();
    assert_eq!(hex(parameters.window_words_sha256()), WINDOW_SHA256);
    assert_eq!(vectors["window_words_sha256"].as_str(), Some(WINDOW_SHA256));

    let recipes = vectors["recipes"].as_object().unwrap();
    let expected_recipes = [
        "256 positive zeros",
        "unity at i in {0,63,64,127,128,191,192,255}, otherwise positive zero",
        "R32((((37*i+11)%257)-128)/128), i0..255",
        "256 negative zeros",
        "i even +1, odd -1",
        "R32((i-128)/256), i0..255",
    ];
    for (recipe, expected) in expected_recipes.iter().enumerate() {
        assert_eq!(recipes[&recipe.to_string()].as_str(), Some(*expected));
    }

    let cases = vectors["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 6);
    let mut carry = SecondaryOverlapCarry::cold();
    for (step, case) in cases.iter().enumerate() {
        assert_eq!(
            case["sequence_step"].as_u64(),
            Some(u64::try_from(step).unwrap())
        );
        let recipe = usize::try_from(case["input_recipe_id"].as_u64().unwrap()).unwrap();
        let input = input_words(recipe);
        assert_eq!(hash_words(&input), case["input_sha256"].as_str().unwrap());
        let result = overlap_secondary_block(&input, &carry, &parameters).unwrap();
        let expected_output = words(&case["expected_output_words"]);
        let expected_carry = words(&case["expected_next_state_words"]);
        assert_eq!(result.output_words().as_slice(), expected_output.as_slice());
        assert_eq!(
            result.carry_after().words().as_slice(),
            expected_carry.as_slice()
        );
        carry = result.into_parts().1;
    }
}

#[test]
fn overlap_rejects_invalid_shapes_and_nonfinite_words() {
    let parameters = SecondaryOverlapParameters::selected().unwrap();
    let carry = SecondaryOverlapCarry::cold();
    assert_eq!(
        overlap_secondary_block(&[0; 255], &carry, &parameters),
        Err(SecondaryOverlapError::InputError)
    );
    let mut nonfinite = [0; 256];
    nonfinite[17] = f32::INFINITY.to_bits();
    assert_eq!(
        overlap_secondary_block(&nonfinite, &carry, &parameters),
        Err(SecondaryOverlapError::InputError)
    );
}
