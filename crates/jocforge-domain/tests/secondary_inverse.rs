use jocforge_domain::transform::{
    secondary_fft::SecondaryFftParameters,
    secondary_inverse::{SecondaryInverseError, execute_secondary_inverse256},
};
use serde_json::Value;

const VERIFICATION: &[u8] =
    include_bytes!("fixtures/secondary_fft/SECONDARY-FFT-INVERSE-VERIFICATION.json");

fn document() -> Value {
    serde_json::from_slice(VERIFICATION).expect("approved secondary inverse vectors parse")
}

fn input_words(recipe: usize) -> Vec<u32> {
    match recipe {
        0 => vec![0; 256],
        1 => vec![0x8000_0000; 256],
        2 => {
            let mut words = vec![0; 256];
            for index in [0, 1, 2, 3, 216, 217, 252, 253, 254, 255] {
                words[index] = 1.0_f32.to_bits();
            }
            words
        }
        3 => (0..256)
            .map(|word| {
                let numerator = i16::try_from((37 * word + 11) % 257).unwrap() - 128;
                (f32::from(numerator) / 16.0).to_bits()
            })
            .collect(),
        4 => (0..256)
            .map(|word| {
                let numerator = i16::try_from(word).unwrap() - 128;
                (f32::from(numerator) / 256.0).to_bits()
            })
            .collect(),
        _ => panic!("unknown approved inverse recipe"),
    }
}

fn expected_words(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .expect("expected inverse output words")
        .iter()
        .map(|word| u32::try_from(word.as_u64().expect("U32 word")).unwrap())
        .collect()
}

#[test]
fn long_and_short_inverse256_match_all_actual_fft_input_first_vectors() {
    let vectors: Value = document();
    let recipes = vectors["inverse_and_chain_recipes"].as_object().unwrap();
    let expected_recipes = [
        "256 positive zeros",
        "256 negative zeros",
        "unity at indices0,1,2,3,216,217,252,253,254,255, all others positive zero",
        "R32((((37*j+11)%257)-128)/16), j0..255",
        "R32((j-128)/256), j0..255",
    ];
    for (recipe, expected) in expected_recipes.iter().enumerate() {
        assert_eq!(recipes[&recipe.to_string()].as_str(), Some(*expected));
    }

    let parameters = SecondaryFftParameters::selected().unwrap();
    let cases = vectors["inverse_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 10);
    for case in cases {
        let recipe = usize::try_from(case["recipe"].as_u64().unwrap()).unwrap();
        let short = case["short"].as_bool().unwrap();
        let actual = execute_secondary_inverse256(&input_words(recipe), short, &parameters)
            .expect("selected inverse256 case executes");
        assert_eq!(actual.as_slice(), expected_words(&case["output_words"]));
    }
}

#[test]
fn secondary_inverse_rejects_wrong_shape_and_nonfinite_spectrum() {
    let parameters = SecondaryFftParameters::selected().unwrap();
    assert_eq!(
        execute_secondary_inverse256(&[0; 255], false, &parameters),
        Err(SecondaryInverseError::InputError)
    );
    let mut nonfinite = [0; 256];
    nonfinite[17] = f32::NAN.to_bits();
    assert_eq!(
        execute_secondary_inverse256(&nonfinite, true, &parameters),
        Err(SecondaryInverseError::InputError)
    );
}
