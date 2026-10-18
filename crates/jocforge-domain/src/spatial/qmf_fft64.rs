// pattern: Functional Core
use super::qmf_mathematical_fft_provider::QmfMathematicalFftProvider;
use crate::numeric::{FiniteF32, FiniteF32Error, R32Error, r32_add, r32_mul, r32_sub};

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

        for base in (0..COMPLEX_POINT_COUNT).step_by(length) {
            for (j, root_words) in roots.iter().enumerate() {
                let left = base + j;
                let right = left + half_length;

                // Read both complex inputs before writing either butterfly destination.
                let a_real = finite(work[left * 2])?;
                let a_imag = finite(work[left * 2 + 1])?;
                let b_real = finite(work[right * 2])?;
                let b_imag = finite(work[right * 2 + 1])?;
                let c = finite(f32::from_bits(root_words[0]))?;
                let d = finite(f32::from_bits(root_words[1]))?;

                let tr = r32_sub(r32_mul(c, b_real)?, r32_mul(d, b_imag)?)?;
                let ti = r32_add(r32_mul(c, b_imag)?, r32_mul(d, b_real)?)?;
                let sum = [r32_add(a_real, tr)?, r32_add(a_imag, ti)?];
                let difference = [r32_sub(a_real, tr)?, r32_sub(a_imag, ti)?];

                work[left * 2] = sum[0].value();
                work[left * 2 + 1] = sum[1].value();
                work[right * 2] = difference[0].value();
                work[right * 2 + 1] = difference[1].value();
            }
        }
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

fn finite(value: f32) -> Result<FiniteF32, QmfFftError> {
    FiniteF32::new(value).map_err(Into::into)
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
