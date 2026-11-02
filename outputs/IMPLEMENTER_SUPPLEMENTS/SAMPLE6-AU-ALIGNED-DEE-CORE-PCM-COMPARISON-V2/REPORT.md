# Sample 6 AU-aligned 5.1 core PCM comparison, per-channel

The same authorized `人是猫` ADM (`7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`) was compared over source samples `[4,224,000,4,416,000)`: 192,000 frames per channel, exactly 125 E-AC-3 AUs. The DEE output is 48 kHz, six-channel 5.1(side), 1024 kbps; the Rust full-run crop is AU `[2750,2875)`. Both windows were decoded as `pcm_f32le` with FFmpeg 8.1.

| Decoded channel index | Mean absolute error | RMSE | Maximum absolute error |
|---:|---:|---:|---:|
| 0 | 0.0009079804 | 0.0090477978 | 0.3048092052 |
| 1 | 0.0008499758 | 0.0089214353 | 0.3036606759 |
| 2 | 0.0004154149 | 0.0037042021 | 0.1799558358 |
| 3 | 0.0042203270 | 0.0054798588 | 0.0184657630 |
| 4 | 0.0006096388 | 0.0046081547 | 0.2434435161 |
| 5 | 0.0005502590 | 0.0043449612 | 0.2128001194 |

All six decoded channels differ. Channel index 3 has the highest mean absolute error; indices 0 and 1 have the highest RMSE and peaks. Channel labels, per-channel correlation, best gain, and best lag were not captured, so this pattern narrows the comparison but does not identify the cause. Do not infer a channel remap or a codec-stage difference from these error magnitudes alone.

The aggregate comparison remains: DEE PCM hash `fcd37d2218c98229a48cff1e25644315ecf62267acdd27da82d6d8524daf8025` (source-attested; DEE M4A was deleted before Wash could recompute it) versus Rust PCM hash `6d9a5de71c530e65d87aa7ef203b3900ff50beec336018c7b269ed061747f2f7` (independently recomputed by Wash); RMSE `0.0063942474`, max absolute error `0.3048092052`. No PCM was persisted.

Scope is E-AC-3 5.1 core/bed only. This does not compare Atmos objects or spatial rendering, does not establish full-track parity, and does not attribute the residual to a specific audio algorithm. No private key, license material, or encoded payload is included.
