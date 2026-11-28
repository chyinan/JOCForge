//! Parser for the custom `silero_vad.bin` weight blob (PLAN.md §3.2).
//!
//! Layout (little-endian): magic "SVAD", u32 version, u32 n_tensors, then per
//! tensor: u32 name_len, name bytes, u32 ndim, ndim * u32 dims, f32 data.

use std::collections::HashMap;
use std::path::Path;

use crate::Error;

/// A parsed tensor: dims + row-major f32 data.
#[derive(Debug, Clone, PartialEq)]
pub struct Tensor {
    pub dims: Vec<usize>,
    pub data: Vec<f32>,
}

/// All tensors of the blob, keyed by name.
#[derive(Debug, Clone)]
pub struct Weights {
    tensors: HashMap<String, Tensor>,
}

impl Weights {
    /// The embedded default weights (`assets/silero_vad.bin`).
    pub fn embedded() -> Result<Self, Error> {
        Self::from_bytes(include_bytes!("../assets/silero_vad.bin"))
    }

    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let bytes = std::fs::read(path)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let mut r = Reader { bytes, off: 0 };
        if r.take(4)? != b"SVAD" {
            return Err(Error::Weights("bad magic".into()));
        }
        let version = r.u32()?;
        if version != 1 {
            return Err(Error::Weights(format!("unsupported version {version}")));
        }
        let n_tensors = r.u32()? as usize;
        let mut tensors = HashMap::with_capacity(n_tensors);
        for _ in 0..n_tensors {
            let name_len = r.u32()? as usize;
            let name = std::str::from_utf8(r.take(name_len)?)
                .map_err(|e| Error::Weights(format!("invalid tensor name utf8: {e}")))?
                .to_string();
            let ndim = r.u32()? as usize;
            if ndim > 4 {
                return Err(Error::Weights(format!("tensor {name}: ndim {ndim} > 4")));
            }
            let mut dims = Vec::with_capacity(ndim);
            let mut count = 1usize;
            for _ in 0..ndim {
                let d = r.u32()? as usize;
                count = count
                    .checked_mul(d)
                    .ok_or_else(|| Error::Weights(format!("tensor {name}: dims overflow")))?;
                dims.push(d);
            }
            let data = r.f32s(count)?;
            if tensors
                .insert(name.clone(), Tensor { dims, data })
                .is_some()
            {
                return Err(Error::Weights(format!("duplicate tensor {name}")));
            }
        }
        if r.off != bytes.len() {
            return Err(Error::Weights("trailing bytes after last tensor".into()));
        }
        Ok(Self { tensors })
    }

    /// Remove a tensor by name, checking its dims.
    pub(crate) fn take(&mut self, name: &str, expected_dims: &[usize]) -> Result<Tensor, Error> {
        let t = self
            .tensors
            .remove(name)
            .ok_or_else(|| Error::Weights(format!("missing tensor {name}")))?;
        if t.dims != expected_dims {
            return Err(Error::Weights(format!(
                "tensor {name}: dims {:?} != expected {:?}",
                t.dims, expected_dims
            )));
        }
        Ok(t)
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    off: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], Error> {
        let end = self
            .off
            .checked_add(n)
            .filter(|&e| e <= self.bytes.len())
            .ok_or_else(|| Error::Weights("unexpected end of data".into()))?;
        let s = &self.bytes[self.off..end];
        self.off = end;
        Ok(s)
    }

    fn u32(&mut self) -> Result<u32, Error> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn f32s(&mut self, n: usize) -> Result<Vec<f32>, Error> {
        let bytes = self.take(
            n.checked_mul(4)
                .ok_or_else(|| Error::Weights("tensor data length overflow".into()))?,
        )?;
        Ok(bytes
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
            .collect())
    }
}

/// Weights for one sample-rate branch of the model.
///
/// Note: the 8k/16k branches of the ONNX model carry fully independent
/// weights (verified at export time), so each `SileroVad` holds one of these.
pub(crate) struct RateWeights {
    /// STFT basis, [2 * n_bins, kernel]; row i < n_bins is cos, else sin part.
    pub stft_basis: Tensor,
    /// Encoder conv weights [Cout, Cin, 3] and biases [Cout], layers 0..4.
    pub enc_w: [Tensor; 4],
    /// NEON-friendly repack of `enc_w`: [Cout, Cin, 4] with a zero 4th lane
    /// (one-time conversion at load; the .bin format is unchanged).
    #[cfg(target_arch = "aarch64")]
    pub enc_w_packed: [Vec<f32>; 4],
    pub enc_b: [Vec<f32>; 4],
    /// LSTM weights, [512, 256] = rows of [w_ih | w_hh]; input vector [x | h].
    pub lstm_w: Tensor,
    /// LSTM biases, [512] = b_ih + b_hh (pre-added at export).
    pub lstm_b: Vec<f32>,
    /// Head 1x1 conv weight flattened, [128].
    pub head_w: Vec<f32>,
    pub head_b: f32,
}

/// Pack conv weights [cout, cin, k] (k <= 4) into [cout, cin, 4] with a zero
/// 4th lane, the layout the NEON direct-conv kernel streams.
#[cfg(target_arch = "aarch64")]
pub(crate) fn pack_conv_w(w: &[f32], cout: usize, cin: usize, k: usize) -> Vec<f32> {
    assert!(k <= 4);
    assert_eq!(w.len(), cout * cin * k);
    let mut packed = vec![0.0f32; cout * cin * 4];
    for oc in 0..cout {
        for c in 0..cin {
            packed[(oc * cin + c) * 4..(oc * cin + c) * 4 + k]
                .copy_from_slice(&w[(oc * cin + c) * k..(oc * cin + c) * k + k]);
        }
    }
    packed
}

impl RateWeights {
    pub(crate) fn select(w: &mut Weights, sample_rate: crate::SampleRate) -> Result<Self, Error> {
        let suffix = match sample_rate {
            crate::SampleRate::Hz16000 => "16k",
            crate::SampleRate::Hz8000 => "8k",
        };
        let (basis_dims, enc0_cin) = match sample_rate {
            crate::SampleRate::Hz16000 => ([258, 256], 129),
            crate::SampleRate::Hz8000 => ([130, 128], 65),
        };
        let t =
            |w: &mut Weights, name: &str, dims: &[usize]| w.take(&format!("{name}_{suffix}"), dims);
        let enc_w = [
            t(w, "enc0_w", &[128, enc0_cin, 3])?,
            t(w, "enc1_w", &[64, 128, 3])?,
            t(w, "enc2_w", &[64, 64, 3])?,
            t(w, "enc3_w", &[128, 64, 3])?,
        ];
        #[cfg(target_arch = "aarch64")]
        let enc_w_packed = {
            let pack = |t: &Tensor| pack_conv_w(&t.data, t.dims[0], t.dims[1], t.dims[2]);
            [
                pack(&enc_w[0]),
                pack(&enc_w[1]),
                pack(&enc_w[2]),
                pack(&enc_w[3]),
            ]
        };
        Ok(Self {
            stft_basis: t(w, "stft_basis", &basis_dims)?,
            #[cfg(target_arch = "aarch64")]
            enc_w_packed,
            enc_w,
            enc_b: [
                t(w, "enc0_b", &[128])?.data,
                t(w, "enc1_b", &[64])?.data,
                t(w, "enc2_b", &[64])?.data,
                t(w, "enc3_b", &[128])?.data,
            ],
            lstm_w: t(w, "lstm_w", &[512, 256])?,
            lstm_b: t(w, "lstm_b", &[512])?.data,
            head_w: t(w, "head_w", &[128])?.data,
            head_b: t(w, "head_b", &[1])?.data[0],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_embedded() {
        let mut w = Weights::embedded().unwrap();
        assert_eq!(w.tensors.len(), 26);
        RateWeights::select(&mut w, crate::SampleRate::Hz16000).unwrap();
        RateWeights::select(&mut w, crate::SampleRate::Hz8000).unwrap();
        assert!(w.tensors.is_empty());
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = include_bytes!("../assets/silero_vad.bin").to_vec();
        bytes[0] = b'X';
        assert!(Weights::from_bytes(&bytes).is_err());
    }

    #[test]
    fn rejects_truncated() {
        let bytes = &include_bytes!("../assets/silero_vad.bin")[..1000];
        assert!(Weights::from_bytes(bytes).is_err());
    }
}
