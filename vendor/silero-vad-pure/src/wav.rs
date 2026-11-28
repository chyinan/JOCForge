//! Minimal RIFF/WAVE reader: PCM16 mono only.

use std::path::Path;

use crate::Error;

/// Read a PCM16 mono wav file; returns (sample_rate, samples normalized to
/// [-1, 1) by /32768).
pub fn read_wav(path: &Path) -> Result<(u32, Vec<f32>), Error> {
    let data = std::fs::read(path)?;
    if data.len() < 12 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return Err(Error::Wav("not a RIFF/WAVE file".into()));
    }
    let mut sample_rate = None;
    let mut pcm = None;
    let mut off = 12;
    while off + 8 <= data.len() {
        let id = &data[off..off + 4];
        let size = u32::from_le_bytes(data[off + 4..off + 8].try_into().unwrap()) as usize;
        let body = off + 8;
        if body + size > data.len() {
            return Err(Error::Wav("truncated chunk".into()));
        }
        match id {
            b"fmt " => {
                if size < 16 {
                    return Err(Error::Wav("short fmt chunk".into()));
                }
                let format = u16::from_le_bytes(data[body..body + 2].try_into().unwrap());
                let channels = u16::from_le_bytes(data[body + 2..body + 4].try_into().unwrap());
                if format != 1 {
                    return Err(Error::Wav(format!(
                        "unsupported format tag {format} (PCM16 only)"
                    )));
                }
                if channels != 1 {
                    return Err(Error::Wav(format!(
                        "unsupported channel count {channels} (mono only)"
                    )));
                }
                sample_rate = Some(u32::from_le_bytes(
                    data[body + 4..body + 8].try_into().unwrap(),
                ));
            }
            b"data" => {
                if !size.is_multiple_of(2) {
                    return Err(Error::Wav("odd data chunk size".into()));
                }
                let samples = data[body..body + size]
                    .chunks_exact(2)
                    .map(|c| i16::from_le_bytes(c.try_into().unwrap()) as f32 / 32768.0)
                    .collect();
                pcm = Some(samples);
            }
            _ => {}
        }
        off = body + size + (size & 1);
    }
    match (sample_rate, pcm) {
        (Some(sr), Some(samples)) => Ok((sr, samples)),
        _ => Err(Error::Wav("missing fmt or data chunk".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_test_wavs() {
        let (sr, x) = read_wav(Path::new("assets/aepyx.wav")).unwrap();
        assert_eq!(sr, 16000);
        assert!(x.len() > 512 * 100);
        assert!(x.iter().all(|v| (-1.0..1.0).contains(v)));
        let (sr8, x8) = read_wav(Path::new("assets/aepyx_8k.wav")).unwrap();
        assert_eq!(sr8, 8000);
        assert_eq!(x8.len(), x.len() / 2);
    }
}
