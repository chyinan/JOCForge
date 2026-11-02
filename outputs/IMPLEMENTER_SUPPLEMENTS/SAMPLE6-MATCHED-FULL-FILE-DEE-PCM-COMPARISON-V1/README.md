# Matched full-file DEE/Rust PCM window

This Wash package covers one same-input, same-control comparison for the authorized “人是猫” ADM. The DEE XML uses atmos_mezz for the ADM input and is bound by SHA-256 in EVIDENCE.json.

Both outputs are full-length 48 kHz, six-channel 5.1(side), 1024 kbps E-AC-3 JOC files with 4,247 access units. The comparison decodes AU interval [2750,2875) (192,000 samples per channel) in memory. No PCM, encoded payload, key, or license material is included.

At decoder DRC scale 1, aggregate RMSE is 0.00232047215; at DRC scale 0 it is 0.00225856435. All channel best lags are zero. Decoder channel index 3 remains the dominant residual in both conditions (RMSE 0.00547723 and 0.00545409 respectively); its native channel/bed identity is being checked separately.

The other five channels are close in this window (DRC scale 0 RMSE 0.000379–0.000456). This is strong evidence that the remaining measured error is concentrated in one channel. It does not establish full-track PCM equality, identify the native cause, or verify Atmos rendering/listening parity.

FFmpeg 8.1 was used only to decode both existing outputs for an in-memory comparison. No FFmpeg encoding or audio-processing rule is proposed for JOCForge.
