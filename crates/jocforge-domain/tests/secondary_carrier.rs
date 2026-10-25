use jocforge_domain::transform::secondary_carrier::{
    SecondaryCarrierError, assemble_secondary_carrier,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const OVERLAP_CARRIER_VECTORS: &[u8] =
    include_bytes!("fixtures/secondary_overlap/overlap-verification.json");
const LOGICAL_TO_OUTPUT_ROW: [usize; 5] = [0, 2, 1, 4, 5];
const LOGICAL_LANE_COUNT: usize = 5;
const BLOCK_COUNT: usize = 6;
const BLOCK_SAMPLES: usize = 256;
const OUTPUT_ROW_COUNT: usize = 6;
const CARRIER_SAMPLES: usize = 1_536;

fn input_only_carrier_recipe(vector: &Value) -> Vec<Vec<Vec<u32>>> {
    assert_eq!(
        vector["carrier_recipe"].as_str(),
        Some("blocks[lane][block][i]=R32((lane+1)*8+block+i/512); lane0..4,block0..5,i0..255")
    );
    (0..LOGICAL_LANE_COUNT)
        .map(|lane| {
            (0..BLOCK_COUNT)
                .map(|block| {
                    (0..BLOCK_SAMPLES)
                        .map(|sample| {
                            let base = u16::try_from((lane + 1) * 8 + block).unwrap();
                            let fraction = u16::try_from(sample).unwrap();
                            (f32::from(base) + f32::from(fraction) / 512.0).to_bits()
                        })
                        .collect()
                })
                .collect()
        })
        .collect()
}

fn sha256_words(words: &[u32]) -> String {
    let bytes = words
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>();
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn input_only_recipe_matches_carrier_and_row_hashes_with_selected_routing() {
    let vector: Value =
        serde_json::from_slice(OVERLAP_CARRIER_VECTORS).expect("approved overlap vector parses");
    let blocks = input_only_carrier_recipe(&vector);
    let carrier = assemble_secondary_carrier(&blocks).expect("selected carrier recipe is valid");
    let rows = carrier.rows();

    let serialized_rows = rows.iter().flatten().copied().collect::<Vec<_>>();
    assert_eq!(
        sha256_words(&serialized_rows),
        vector["carrier_expected_sha256"].as_str().unwrap()
    );
    let expected_row_hashes = vector["carrier_expected_row_sha256s"]
        .as_array()
        .expect("six expected row hashes");
    assert_eq!(expected_row_hashes.len(), OUTPUT_ROW_COUNT);
    for (row, expected_hash) in rows.iter().zip(expected_row_hashes) {
        assert_eq!(sha256_words(row), expected_hash.as_str().unwrap());
        assert!(
            row[LOGICAL_LANE_COUNT * BLOCK_SAMPLES..]
                .iter()
                .all(|word| *word == 0)
        );
    }
    assert!(rows[3].iter().all(|word| *word == 0));

    for (logical_lane, output_row) in LOGICAL_TO_OUTPUT_ROW.into_iter().enumerate() {
        for (output_block, input_block) in (1..BLOCK_COUNT).enumerate() {
            let destination = output_block * BLOCK_SAMPLES;
            assert_eq!(
                &rows[output_row][destination..destination + BLOCK_SAMPLES],
                blocks[logical_lane][input_block].as_slice()
            );
        }
    }
    assert!(rows.iter().all(|row| row.len() == CARRIER_SAMPLES));
}

#[test]
fn carrier_rejects_wrong_shapes_and_nonfinite_block_words() {
    let vector: Value =
        serde_json::from_slice(OVERLAP_CARRIER_VECTORS).expect("approved overlap vector parses");
    let valid = input_only_carrier_recipe(&vector);
    assert_eq!(
        assemble_secondary_carrier(&valid[..LOGICAL_LANE_COUNT - 1]),
        Err(SecondaryCarrierError::InputShape)
    );

    let mut wrong_block_count = valid.clone();
    wrong_block_count[0].pop();
    assert_eq!(
        assemble_secondary_carrier(&wrong_block_count),
        Err(SecondaryCarrierError::InputShape)
    );

    let mut wrong_block_size = valid.clone();
    wrong_block_size[0][0].pop();
    assert_eq!(
        assemble_secondary_carrier(&wrong_block_size),
        Err(SecondaryCarrierError::InputShape)
    );

    let mut nonfinite = valid;
    nonfinite[0][0][0] = f32::NAN.to_bits();
    assert_eq!(
        assemble_secondary_carrier(&nonfinite),
        Err(SecondaryCarrierError::InputNonFinite)
    );
}
