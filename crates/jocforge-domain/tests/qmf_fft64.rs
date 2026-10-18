// pattern: Functional Core
use jocforge_domain::frontend::{PackageManifestEvidence, ProviderFileEvidence};
use jocforge_domain::spatial::{
    F04ParameterProviderError, QmfFftError, QmfFftInput, QmfMathematicalFftProvider,
    ValidatedF04ParameterPackageManifest, execute_qmf_fft64,
    validate_f04_parameter_package_manifest, validate_qmf_mathematical_fft_provider,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;

const MANIFEST_BYTES: &[u8] = include_bytes!("fixtures/f01e_provider/MANIFEST.json");
const QMF_FFT_BYTES: &[u8] = include_bytes!("fixtures/f04_parameters/qmf-mathematical-fft-v1.json");

fn validated_manifest() -> ValidatedF04ParameterPackageManifest {
    let evidence = PackageManifestEvidence::new(
        Arc::<[u8]>::from(MANIFEST_BYTES),
        Some("shared repository manifest fixture".to_owned()),
    );
    validate_f04_parameter_package_manifest(&evidence)
        .expect("the shared package manifest validates the QMF FFT provider entry")
}

fn validated_provider() -> QmfMathematicalFftProvider {
    let evidence = ProviderFileEvidence::new(
        Arc::<[u8]>::from(QMF_FFT_BYTES),
        Some("approved mathematical QMF FFT provider fixture".to_owned()),
    );
    validate_qmf_mathematical_fft_provider(&evidence, &validated_manifest())
        .expect("the identity-matched mathematical provider validates")
}

fn execute(input: &[f32]) -> Result<jocforge_domain::spatial::QmfFftOutput, QmfFftError> {
    let prepared = QmfFftInput::new(input)?;
    execute_qmf_fft64(&prepared, &validated_provider())
}

#[test]
fn rejects_wrong_scalar_count_and_nonfinite_input_before_transform() {
    assert!(matches!(
        QmfFftInput::new(&[0.0; 127]),
        Err(QmfFftError::WrongScalarCount { actual: 127 })
    ));
    assert!(matches!(
        QmfFftInput::new(&[0.0; 129]),
        Err(QmfFftError::WrongScalarCount { actual: 129 })
    ));

    let mut input = [0.0; 128];
    input[73] = f32::NAN;
    assert!(matches!(
        QmfFftInput::new(&input),
        Err(QmfFftError::NonFiniteInput { scalar_index: 73 })
    ));
    input[73] = f32::INFINITY;
    assert!(matches!(
        QmfFftInput::new(&input),
        Err(QmfFftError::NonFiniteInput { scalar_index: 73 })
    ));
}

#[test]
fn zero_index_impulse_has_unscaled_constant_spectrum_in_natural_order() {
    let mut input = [0.0; 128];
    input[0] = 1.0;
    let output = execute(&input).expect("finite impulse transforms");

    for pair in output.interleaved().chunks_exact(2) {
        assert_eq!(pair[0].to_bits(), 1.0_f32.to_bits());
        assert_eq!(pair[1].to_bits(), 0.0_f32.to_bits());
    }
}

#[test]
fn finite_subnormal_input_is_preserved_by_the_stateless_transform() {
    let mut input = [0.0; 128];
    input[0] = f32::from_bits(1);
    let output = execute(&input).expect("finite subnormal input transforms");

    for pair in output.interleaved().chunks_exact(2) {
        assert_eq!(pair[0].to_bits(), 1);
        assert_eq!(pair[1].to_bits(), 0.0_f32.to_bits());
    }
}

#[test]
fn one_index_impulse_uses_negative_exponent_roots_and_natural_frequency_order() {
    let provider = validated_provider();
    let mut input = [0.0; 128];
    input[2] = 1.0;
    let output = execute_qmf_fft64(
        &QmfFftInput::new(&input).expect("finite fixed-size input"),
        &provider,
    )
    .expect("finite impulse transforms");
    let roots = provider.stages()[5].roots();

    for (root_index, root) in roots.iter().enumerate() {
        let forward = root_index * 2;
        assert_eq!(
            output.interleaved()[forward].to_bits(),
            f32::from_bits(root[0]).to_bits()
        );
        assert_eq!(
            output.interleaved()[forward + 1].to_bits(),
            f32::from_bits(root[1]).to_bits()
        );

        let mirrored = (root_index + 32) * 2;
        let mirrored_real = if root_index == 16 {
            0.0
        } else {
            -f32::from_bits(root[0])
        };
        assert_eq!(
            output.interleaved()[mirrored].to_bits(),
            mirrored_real.to_bits()
        );
        let mirrored_imag = if root_index == 0 {
            0.0
        } else {
            -f32::from_bits(root[1])
        };
        assert_eq!(
            output.interleaved()[mirrored + 1].to_bits(),
            mirrored_imag.to_bits()
        );
    }
}

#[test]
fn caller_prepares_analysis_scale_and_primitive_does_not_normalize_input() {
    let unscaled = [64.0; 128];
    let analysis_prepared = unscaled.map(|value| value / 64.0);

    let synthesis_output = execute(&unscaled).expect("finite unscaled synthesis input");
    let analysis_output = execute(&analysis_prepared).expect("finite caller-scaled analysis input");

    assert_eq!(
        synthesis_output.interleaved()[0].to_bits(),
        4096.0_f32.to_bits()
    );
    assert_eq!(
        analysis_output.interleaved()[0].to_bits(),
        64.0_f32.to_bits()
    );
    assert_eq!(
        synthesis_output.interleaved()[2].to_bits(),
        0.0_f32.to_bits()
    );
    assert_eq!(
        analysis_output.interleaved()[2].to_bits(),
        0.0_f32.to_bits()
    );
}

#[test]
fn nonfinite_intermediate_is_a_fatal_numeric_failure_without_output() {
    let input = [f32::MAX; 128];
    let prepared = QmfFftInput::new(&input).expect("finite maximum-value inputs");
    let result = execute_qmf_fft64(&prepared, &validated_provider());

    assert!(matches!(result, Err(QmfFftError::NumericFailure)));
}

#[test]
fn each_call_starts_from_its_own_input_and_returns_an_immutable_fixed_output() {
    let first_input = [0.0; 128];
    let mut second_input = [0.0; 128];
    second_input[0] = 1.0;

    let first = execute(&first_input).expect("zero input transforms");
    let second = execute(&second_input).expect("impulse transforms");
    let first_again = execute(&first_input).expect("zero input transforms again");

    assert!(
        first
            .interleaved()
            .iter()
            .all(|value| value.to_bits() == 0.0_f32.to_bits())
    );
    assert_eq!(second.interleaved()[0].to_bits(), 1.0_f32.to_bits());
    let first_bits = first
        .interleaved()
        .iter()
        .map(|value| value.to_bits())
        .collect::<Vec<_>>();
    let first_again_bits = first_again
        .interleaved()
        .iter()
        .map(|value| value.to_bits())
        .collect::<Vec<_>>();
    assert_eq!(first_bits, first_again_bits);
}

#[test]
fn actual_provider_has_the_mapped_stage_and_root_shape() {
    let provider = validated_provider();

    assert_eq!(
        provider
            .stages()
            .iter()
            .map(jocforge_domain::spatial::QmfFftRootStage::length)
            .collect::<Vec<_>>(),
        [2, 4, 8, 16, 32, 64]
    );
    assert_eq!(
        provider
            .stages()
            .iter()
            .map(|stage| stage.roots().len())
            .collect::<Vec<_>>(),
        [1, 2, 4, 8, 16, 32]
    );
    assert_eq!(
        provider
            .stages()
            .iter()
            .flat_map(jocforge_domain::spatial::QmfFftRootStage::roots)
            .count(),
        63
    );
    assert_eq!(provider.receipt().audio_acceptance(), "NOT_RUN");
}

#[test]
fn dyadic_input_recipe_matches_the_spec_ordered_r32_reference() {
    // This local mathematical assertion uses the approved root provider; it is not an audio vector.
    let mut input = [0.0; 128];
    for point in 0..64 {
        let real_numerator =
            f32::from(u8::try_from(point * 17 % 23).expect("recipe residue is below 23")) - 11.0;
        let imag_numerator =
            f32::from(u8::try_from(point * 7 % 19).expect("recipe residue is below 19")) - 9.0;
        input[point * 2] = real_numerator / 8.0;
        input[point * 2 + 1] = imag_numerator / 16.0;
    }

    let output = execute(&input).expect("finite dyadic input transforms");
    let bytes = output
        .interleaved()
        .iter()
        .flat_map(|value| value.to_bits().to_le_bytes())
        .collect::<Vec<_>>();

    assert_eq!(
        format!("{:x}", Sha256::digest(bytes)),
        "13e02d1613cbfca47516260119229649c01b5338988fefb4236182b93180167b"
    );
}

#[test]
fn provider_identity_failure_stays_a_provider_boundary_error() {
    let evidence = ProviderFileEvidence::new(
        Arc::<[u8]>::from(MANIFEST_BYTES),
        Some("crosswired provider bytes".to_owned()),
    );
    let result = validate_qmf_mathematical_fft_provider(&evidence, &validated_manifest());

    assert!(matches!(
        result,
        Err(F04ParameterProviderError::ProviderIdentityMismatch)
    ));
}
