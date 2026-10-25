use jocforge_domain::meter::dialnorm::selected_dialnorm_from_lufs;
use jocforge_domain::numeric::FiniteF32;
use serde_json::Value;
use sha2::{Digest, Sha256};

const INPUT_VECTORS: &[u8] = include_bytes!("fixtures/selected_dialnorm/input-only-vectors.json");
const INPUT_VECTORS_SHA256: &str =
    "865ad85dbef724b6896257baa9454db23d0bd82020297330ada4c8a65b816088";

#[test]
fn selected_lufs_input_vectors_recompute_dialnorm_values() {
    assert_eq!(hex(&Sha256::digest(INPUT_VECTORS)), INPUT_VECTORS_SHA256);
    let document: Value = serde_json::from_slice(INPUT_VECTORS).unwrap();
    let vectors = document["vectors"].as_array().unwrap();
    for vector in vectors
        .iter()
        .filter(|case| case["id"].as_str().unwrap().contains("DIALNORM"))
    {
        let input = &vector["input"];
        let selected_lufs = if let Some(word) = input["selected_lufs_f32_word"].as_str() {
            let word = u32::from_str_radix(word.trim_start_matches("0x"), 16).unwrap();
            f32::from_bits(word)
        } else {
            input["selected_lufs_f32_decimal"].as_f64().unwrap() as f32
        };
        let lufs = FiniteF32::new(selected_lufs).unwrap();
        let dialnorm = selected_dialnorm_from_lufs(lufs).unwrap();
        assert_eq!(
            u64::from(dialnorm.value()),
            vector["expected"]["dialnorm"].as_u64().unwrap(),
            "{}",
            vector["id"],
        );
        assert_eq!(
            vector["expected"]["dialnorm_five_bits"]
                .as_str()
                .unwrap()
                .len(),
            5,
        );
    }
}

#[test]
fn selected_lufs_values_outside_dialnorm_range_clamp_after_truncation() {
    assert_eq!(
        selected_dialnorm_from_lufs(FiniteF32::new(f32::MAX).unwrap())
            .unwrap()
            .value(),
        1,
    );
    assert_eq!(
        selected_dialnorm_from_lufs(FiniteF32::new(f32::MIN).unwrap())
            .unwrap()
            .value(),
        31,
    );
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}
