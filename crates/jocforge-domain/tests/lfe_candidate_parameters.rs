use jocforge_domain::transform::math_generated_parameters::{
    MathGeneratedTransformParameterError, MathGeneratedTransformParameters,
};

const MATH_PROVIDER: &[u8] =
    include_bytes!("../src/transform/ordinary_transform_math_parameters.json");
const F08_CANDIDATE_PROVIDER: &[u8] =
    include_bytes!("../src/transform/lfe_candidate_transform_parameters.json");

#[test]
fn washed_f08_lfe_candidate_has_an_independent_provider_identity() {
    assert!(MathGeneratedTransformParameters::from_provider_bytes(MATH_PROVIDER).is_ok());
    assert!(
        MathGeneratedTransformParameters::from_f08_lfe_candidate_provider_bytes(
            F08_CANDIDATE_PROVIDER
        )
        .is_ok()
    );
    assert_eq!(
        MathGeneratedTransformParameters::from_provider_bytes(F08_CANDIDATE_PROVIDER),
        Err(MathGeneratedTransformParameterError::ParameterIdentityError)
    );
    assert_eq!(
        MathGeneratedTransformParameters::from_f08_lfe_candidate_provider_bytes(MATH_PROVIDER),
        Err(MathGeneratedTransformParameterError::ParameterIdentityError)
    );

    let candidate = MathGeneratedTransformParameters::from_f08_lfe_candidate_provider_bytes(
        F08_CANDIDATE_PROVIDER,
    )
    .unwrap();
    assert_eq!(
        candidate.lfe_recurrence_f32_bits(),
        &[
            0x4000_0000,
            0x3fff_b10f,
            0x3ffe_c46d,
            0x3ffd_3aac,
            0x3ffb_14be,
            0x3ffd_3aac,
            0x3ffe_c46d,
            0x3fff_b10f,
        ]
    );
    assert_eq!(
        candidate.lfe_cosine_f32_bits(),
        &[
            0x3f80_0000,
            0x3f7f_b10f,
            0x3f7e_c46d,
            0x3f7d_3aac,
            0x3f7b_14be,
            0x3f7d_3aac,
            0x3f7e_c46d,
            0x3f7f_b10f,
        ]
    );
    assert_eq!(
        candidate.lfe_sine_f32_bits(),
        &[
            0x0000_0000,
            0x3d48_fb30,
            0x3dc8_bd36,
            0x3e16_4083,
            0xbe47_c5c2,
            0xbe16_4083,
            0xbdc8_bd36,
            0xbd48_fb30,
        ]
    );
}
