# Channel-index-3 residual frequency diagnostic

This supplement analyzes the already matched full-file DEE/Rust pair. It uses FFmpeg 8.1 only to decode both files to in-memory f32le streams and compute scalar band statistics; no encoder or processing rule is copied.

Across 31 active four-second blocks, 78.1% of decoder channel-index-3 residual energy is below 120 Hz. The other five channels range from 1.7% to 36.7% in that band. Cross-channel residual correlation is moderate in the low band and near zero above 120 Hz.

This supports a low-frequency-weighted, mostly channel-local residual. It is consistent with investigating an LFE path, but does not identify channel index 3 as the DEE LFE input, prove a source-row mapping, or name a corrective filter/gain.
