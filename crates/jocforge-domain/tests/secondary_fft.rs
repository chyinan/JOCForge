use jocforge_domain::transform::secondary_fft::{
    SecondaryFftError, SecondaryFftParameters, SecondaryFftSize, execute_secondary_fft,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const VERIFICATION: &[u8] =
    include_bytes!("fixtures/secondary_fft/SECONDARY-FFT-INVERSE-VERIFICATION.json");
const PARAMETER_SHA256: &str = "b452a929d7e4f7090f9466930e5c6047a8007192a30a3ed7c3cd7447dba073b1";

fn document() -> Value {
    serde_json::from_slice(VERIFICATION).expect("approved secondary FFT vectors parse")
}

fn input_words(complex_points: usize, recipe: usize) -> Vec<u32> {
    let word_count = complex_points * 2;
    match recipe {
        0 => vec![0; word_count],
        1 => vec![0x8000_0000; word_count],
        2 => {
            let mut words = vec![0; word_count];
            words[0] = 1.0_f32.to_bits();
            words
        }
        3 => {
            let mut words = vec![0; word_count];
            words[word_count - 1] = (-0.75_f32).to_bits();
            words
        }
        4 => (0..word_count)
            .map(|word| {
                let numerator = i16::try_from((37 * word + 11) % 257).unwrap() - 128;
                (f32::from(numerator) / 16.0).to_bits()
            })
            .collect(),
        5 => (0..word_count)
            .map(|word| {
                let numerator =
                    i16::try_from(word).unwrap() - i16::try_from(complex_points).unwrap();
                (f32::from(numerator) / 256.0).to_bits()
            })
            .collect(),
        6 => (0..word_count)
            .map(|word| {
                if word % 2 == 0 {
                    0.5_f32.to_bits()
                } else {
                    (-0.5_f32).to_bits()
                }
            })
            .collect(),
        _ => panic!("unknown approved input recipe"),
    }
}

fn expected_words(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .expect("expected output words")
        .iter()
        .map(|word| u32::try_from(word.as_u64().expect("U32 word")).unwrap())
        .collect()
}

fn output_sha256(words: &[u32]) -> String {
    let mut bytes = Vec::with_capacity(words.len() * 4);
    for word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
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
fn selected_fft64_and_fft128_match_all_direct_input_first_cases() {
    let parameters = SecondaryFftParameters::selected().unwrap();
    assert_eq!(hex(parameters.provider_sha256()), PARAMETER_SHA256);
    let vectors = document();
    let recipes = vectors["fft_recipes"].as_object().unwrap();
    let expected_recipes = [
        "2N positive zeros",
        "2N negative zeros",
        "unity at word0, all others positive zero",
        "-3/4 at word2N-1, all others positive zero",
        "R32((((37*j+11)%257)-128)/16), j0..2N-1",
        "R32((j-N)/256), j0..2N-1",
        "evenword1/2 oddword-1/2",
    ];
    for (recipe, expected) in expected_recipes.iter().enumerate() {
        assert_eq!(recipes[&recipe.to_string()].as_str(), Some(*expected));
    }

    let cases = vectors["fft_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 14);
    for case in cases {
        let complex_points = usize::try_from(case["complex_count"].as_u64().unwrap()).unwrap();
        let size = SecondaryFftSize::new(u16::try_from(complex_points).unwrap()).unwrap();
        let recipe = usize::try_from(case["recipe"].as_u64().unwrap()).unwrap();
        let actual = execute_secondary_fft(size, &input_words(complex_points, recipe), &parameters)
            .expect("selected FFT case executes");
        assert_eq!(actual, expected_words(&case["output_words"]));
        assert_eq!(case["mismatch_count"].as_u64(), Some(0));
        assert!(case["first_difference"].is_null());
    }
}

#[test]
fn selected_fft64_and_fft128_match_all_unity_basis_output_hashes() {
    let parameters = SecondaryFftParameters::selected().unwrap();
    let vectors = document();
    let basis = vectors["basis_cases"].as_array().unwrap();
    assert_eq!(basis.len(), 384);
    for case in basis {
        let complex_points = usize::try_from(case["complex_count"].as_u64().unwrap()).unwrap();
        let size = SecondaryFftSize::new(u16::try_from(complex_points).unwrap()).unwrap();
        let impulse_word = usize::try_from(case["impulse_word"].as_u64().unwrap()).unwrap();
        let mut input = vec![0; complex_points * 2];
        input[impulse_word] = 1.0_f32.to_bits();
        let output = execute_secondary_fft(size, &input, &parameters)
            .expect("selected unity basis case executes");
        assert_eq!(
            output_sha256(&output),
            case["expected_output_f32le_sha256"].as_str().unwrap()
        );
    }
}

#[test]
fn secondary_fft_rejects_unselected_sizes_and_invalid_inputs() {
    let parameters = SecondaryFftParameters::selected().unwrap();
    assert_eq!(
        SecondaryFftSize::new(256),
        Err(SecondaryFftError::UnsupportedProfile)
    );
    assert_eq!(
        execute_secondary_fft(SecondaryFftSize::Points64, &[0; 127], &parameters),
        Err(SecondaryFftError::InputError)
    );
    let mut nonfinite = vec![0; 128];
    nonfinite[7] = f32::NAN.to_bits();
    assert_eq!(
        execute_secondary_fft(SecondaryFftSize::Points64, &nonfinite, &parameters),
        Err(SecondaryFftError::InputError)
    );
}
