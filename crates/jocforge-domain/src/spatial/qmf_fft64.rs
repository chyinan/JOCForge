// pattern: Functional Core
use super::qmf_mathematical_fft_provider::QmfMathematicalFftProvider;
use crate::numeric::{FiniteF32Error, R32Error};

const COMPLEX_POINT_COUNT: usize = 64;
const SCALAR_COUNT: usize = 128;
const STAGE_LENGTHS: [usize; 6] = [2, 4, 8, 16, 32, 64];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QmfFftError {
    WrongScalarCount { actual: usize },
    NonFiniteInput { scalar_index: usize },
    NumericFailure,
}

pub struct QmfFftInput([f32; SCALAR_COUNT]);

impl QmfFftInput {
    pub fn new(interleaved: &[f32]) -> Result<Self, QmfFftError> {
        let values: &[f32; SCALAR_COUNT] =
            interleaved
                .try_into()
                .map_err(|_| QmfFftError::WrongScalarCount {
                    actual: interleaved.len(),
                })?;
        if let Some((scalar_index, _)) = values
            .iter()
            .enumerate()
            .find(|(_, value)| !value.is_finite())
        {
            return Err(QmfFftError::NonFiniteInput { scalar_index });
        }
        Ok(Self(*values))
    }
}

pub struct QmfFftOutput([f32; SCALAR_COUNT]);

impl QmfFftOutput {
    pub const fn interleaved(&self) -> &[f32; SCALAR_COUNT] {
        &self.0
    }
}

pub fn execute_qmf_fft64(
    input: &QmfFftInput,
    provider: &QmfMathematicalFftProvider,
) -> Result<QmfFftOutput, QmfFftError> {
    let mut work = [0.0_f32; SCALAR_COUNT];
    for destination in 0..COMPLEX_POINT_COUNT {
        let source = reverse_six_bits(destination);
        work[destination * 2] = input.0[source * 2];
        work[destination * 2 + 1] = input.0[source * 2 + 1];
    }

    for (stage, expected_length) in provider.stages().iter().zip(STAGE_LENGTHS) {
        let length = usize::from(stage.length());
        let half_length = length / 2;
        let roots = stage.roots();
        if length != expected_length || roots.len() != half_length {
            return Err(QmfFftError::NumericFailure);
        }

        super::fft_butterflies::apply_stage(&mut work, roots);
    }
    // Butterflies contain only products and sums. A nonfinite intermediate
    // propagates to a final lane, including an infinity cancelled by another infinity.
    if work.iter().any(|x| !x.is_finite()) {
        return Err(QmfFftError::NumericFailure);
    }
    Ok(QmfFftOutput(work))
}

fn reverse_six_bits(value: usize) -> usize {
    let mut remaining = value;
    let mut reversed = 0;
    for _ in 0..6 {
        reversed = (reversed << 1) | (remaining & 1);
        remaining >>= 1;
    }
    reversed
}

impl From<FiniteF32Error> for QmfFftError {
    fn from(_: FiniteF32Error) -> Self {
        Self::NumericFailure
    }
}

impl From<R32Error> for QmfFftError {
    fn from(_: R32Error) -> Self {
        Self::NumericFailure
    }
}
