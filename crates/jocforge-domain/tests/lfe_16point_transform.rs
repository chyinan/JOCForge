use jocforge_domain::transform::lfe_transform::{
    transform_lfe_16point_block, transform_lfe_16point_butterfly_row,
};
use serde_json::Value;

const INPUT_ONLY_VECTORS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../outputs/IMPLEMENTER_SUPPLEMENTS/DEE-LFE-16POINT-TERMINAL-TRANSFORM-V1/INPUT-ONLY-VECTORS.json"
));
const COEFFICIENTS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../outputs/IMPLEMENTER_SUPPLEMENTS/DEE-LFE-16POINT-TERMINAL-TRANSFORM-V1/COEFFICIENTS.json"
));

struct ReferenceCoefficients {
    window_0: [u32; 8],
    window_1: [u32; 8],
    cosine: [u32; 8],
    permutation: [usize; 8],
    output_scale: u32,
    rotation: u32,
}

impl ReferenceCoefficients {
    fn from_wash_payload() -> Self {
        let payload: Value = serde_json::from_str(COEFFICIENTS).expect("coefficient JSON");
        Self {
            window_0: table_bits(&payload, "W0"),
            window_1: table_bits(&payload, "W1"),
            cosine: table_bits(&payload, "C"),
            permutation: table_bits(&payload, "P").map(|value| value as usize),
            output_scale: scalar_bits(&payload, "S_out"),
            rotation: scalar_bits(&payload, "rotation_q"),
        }
    }
}

fn table_bits(payload: &Value, name: &str) -> [u32; 8] {
    payload["tables_and_scalars"][name]["values"]
        .as_array()
        .expect("table array")
        .iter()
        .map(|entry| parse_bits(entry["raw_bits"].as_str().expect("raw bits")))
        .collect::<Vec<_>>()
        .try_into()
        .expect("eight table values")
}

fn scalar_bits(payload: &Value, name: &str) -> u32 {
    parse_bits(
        payload["tables_and_scalars"][name]["values"][0]["raw_bits"]
            .as_str()
            .expect("scalar raw bits"),
    )
}

fn parse_bits(value: &str) -> u32 {
    u32::from_str_radix(value.strip_prefix("0x").expect("hex prefix"), 16).expect("32-bit hex")
}

fn f32_from_bits(bits: u32) -> f32 {
    f32::from_bits(bits)
}

fn flip_sign(value: f32) -> f32 {
    f32::from_bits(value.to_bits() ^ 0x8000_0000)
}

fn reference_butterfly(input: &[u32; 16]) -> [u32; 16] {
    let v = input.map(f32_from_bits);
    let mut a = [0.0_f32; 16];
    for index in 0..4 {
        a[index] = v[index] + v[8 + index];
        a[4 + index] = v[4 + index] + v[12 + index];
        a[8 + index] = v[index] - v[8 + index];
    }
    a[12] = flip_sign(v[5] - v[13]);
    a[13] = v[4] - v[12];
    a[14] = flip_sign(v[7] - v[15]);
    a[15] = v[6] - v[14];

    let mut b = [0.0_f32; 16];
    b[0] = a[0] + a[4];
    b[4] = a[0] - a[4];
    b[1] = a[1] + a[5];
    b[5] = a[1] - a[5];
    b[2] = a[2] + a[6];
    b[3] = a[2] + a[7];
    b[6] = flip_sign(a[3] - a[7]);
    b[7] = a[2] - a[6];
    b[8..].copy_from_slice(&a[8..]);

    let mut c = b;
    for index in 0..4 {
        c[8 + index] = b[8 + index] - b[12 + index];
        c[12 + index] = b[12 + index] + b[8 + index];
    }

    let rotation = f32_from_bits(ReferenceCoefficients::from_wash_payload().rotation);
    let q10 = rotation * c[10];
    let q11 = rotation * c[11];
    let q14 = rotation * c[14];
    let q15 = rotation * c[15];
    c[10] = q10 + q11;
    c[11] = q11 - q10;
    c[14] = q15 - q14;
    c[15] = flip_sign(q14) - q15;
    c.map(f32::to_bits)
}

fn reference_block(input: &[u32; 256], coefficients: &ReferenceCoefficients) -> [u32; 16] {
    let mut rows = [[0.0_f32; 16]; 16];
    for row in 0..16 {
        let mut gathered = [0_u32; 16];
        for group in 0..8 {
            gathered[2 * group] = input[32 * group + 2 * row];
            gathered[2 * group + 1] = input[32 * group + 2 * row + 1];
        }
        rows[row] = reference_butterfly(&gathered).map(f32_from_bits);
    }

    let output_scale = f32_from_bits(coefficients.output_scale);
    let mut output = [0_u32; 16];
    for bin in 0..8 {
        let column = 2 * coefficients.permutation[bin];
        let cosine = f32_from_bits(coefficients.cosine[bin]);
        let window_0 = f32_from_bits(coefficients.window_0[bin]);
        let window_1 = f32_from_bits(coefficients.window_1[bin]);
        let mut r = 0.0_f32;
        let mut q = 0.0_f32;
        let mut a = rows[15][column];
        let mut b = rows[15][column + 1];

        for top in [14, 9, 4] {
            let r1 = (cosine * a - r) + rows[top][column];
            let q1 = (cosine * b - q) + rows[top][column + 1];
            let a1 = (cosine * r1 - a) + rows[top - 1][column];
            let u1 = (cosine * q1 - b) + rows[top - 1][column + 1];
            let b1 = (cosine * a1 - r1) + rows[top - 2][column];
            let v1 = (cosine * u1 - q1) + rows[top - 2][column + 1];
            r = (cosine * b1 - a1) + rows[top - 3][column];
            q = (cosine * v1 - u1) + rows[top - 3][column + 1];
            a = (cosine * r - b1) + rows[top - 4][column];
            b = (cosine * q - v1) + rows[top - 4][column + 1];
        }

        output[2 * bin] = (((a - r) * window_0 + q * window_1) * output_scale).to_bits();
        output[2 * bin + 1] = (((b - r * window_1) - q * window_0) * output_scale).to_bits();
    }
    output
}

#[test]
fn input_only_vectors_match_the_sanitized_16point_equation_model() {
    let vectors: Value = serde_json::from_str(INPUT_ONLY_VECTORS).expect("vector JSON");
    for vector in vectors["vectors"].as_array().expect("vector list") {
        let input: [u32; 16] = vector["input_f32_bits"]
            .as_array()
            .expect("input bits")
            .iter()
            .map(|value| parse_bits(value.as_str().expect("input bit string")))
            .collect::<Vec<_>>()
            .try_into()
            .expect("16 input values");
        let actual = transform_lfe_16point_butterfly_row(&input);
        assert_eq!(
            actual,
            reference_butterfly(&input),
            "vector {}",
            vector["id"].as_str().expect("vector id")
        );
    }
}

#[test]
fn full_16point_block_matches_equation_model_for_synthetic_blocks() {
    let coefficients = ReferenceCoefficients::from_wash_payload();
    let mut sample_zero = [0_u32; 256];
    sample_zero[0] = 0x3f80_0000;
    let mut sample_last = [0_u32; 256];
    sample_last[255] = 0x3f80_0000;
    let ramp = std::array::from_fn(|index| ((index + 1) as f32).to_bits());

    for (label, input) in [
        ("zero", [0_u32; 256]),
        ("sample zero", sample_zero),
        ("sample last", sample_last),
        ("integer ramp", ramp),
    ] {
        assert_eq!(
            transform_lfe_16point_block(&input),
            reference_block(&input, &coefficients),
            "{label}"
        );
    }
}
