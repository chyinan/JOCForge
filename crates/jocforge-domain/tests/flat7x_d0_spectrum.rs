use jocforge_domain::{
    codec::flat7x_d0_spectrum::{
        D0_PCM_SAMPLES_PER_BLOCK, Flat7xD0PcmBlockInput, Flat7xD0PcmWindow, Flat7xD0SpectrumError,
        transform_flat7x_d0_pcm_block,
    },
    transform::math_generated_parameters::MathGeneratedTransformParameters,
};

fn zero_window() -> Flat7xD0PcmWindow {
    Flat7xD0PcmWindow {
        previous_samples: [0.0; D0_PCM_SAMPLES_PER_BLOCK],
        new_samples: [0.0; D0_PCM_SAMPLES_PER_BLOCK],
    }
}

#[test]
fn d0_pcm_block_produces_four_253_bin_rows_and_measured_gain_ranges() {
    let parameters = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
    let mut channels = [zero_window(); 4];
    channels[0].new_samples[0] = 0.5;
    channels[1].new_samples[0] = 0.25;
    channels[2].new_samples[0] = 0.125;
    channels[3].new_samples[0] = 0.124;

    let output =
        transform_flat7x_d0_pcm_block(&Flat7xD0PcmBlockInput { channels }, &parameters).unwrap();

    assert_eq!(output.gain_range_codes(), &[0, 1, 2, 3]);
    let coefficients = output.coefficient_bits_by_channel();
    assert_eq!(coefficients.len(), 4);
    assert!(coefficients[0].iter().any(|word| *word != 0));
    assert!(
        coefficients
            .iter()
            .all(|row| row.iter().any(|word| *word != 0))
    );
    assert!(
        coefficients
            .iter()
            .flatten()
            .all(|word| f32::from_bits(*word).is_finite())
    );
    assert!(!output.production_allowed());
}

#[test]
fn d0_pcm_block_rejects_out_of_range_samples_before_returning_spectra() {
    let parameters = MathGeneratedTransformParameters::selected_math_generated_v1().unwrap();
    let mut channels = [zero_window(); 4];
    channels[2].new_samples[255] = f32::NAN;

    assert_eq!(
        transform_flat7x_d0_pcm_block(&Flat7xD0PcmBlockInput { channels }, &parameters),
        Err(Flat7xD0SpectrumError::InvalidPcm)
    );
}
