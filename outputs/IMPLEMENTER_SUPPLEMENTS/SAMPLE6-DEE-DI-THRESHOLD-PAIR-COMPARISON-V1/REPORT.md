# DEE dialogue-intelligence / speech-threshold pair comparison

Two full-length DEE outputs for the same user-confirmed ADM were compared: the earlier user-run DI=true/threshold=15 output and a fresh matched-control DI=false/threshold=100 output. Both streams are 48 kHz, six-channel 5.1(side), 1024 kbps E-AC-3 JOC with 4,247 AUs. The true/15 XML body was user-pasted and is not hash-bound; the false/100 run is bound to the Wash-generated matched XML hash. Other shared controls are task-supplied as matching, not independently verified in two hashed XML files.

All AUs differ between the two outputs, but packet sizes and timing match. In AU window [2750,2875), decoder DRC scale 0 gives aggregate PCM RMSE 0.00018015; channel index 3 is exactly identical and the other channels differ only slightly. At decoder DRC scale 1, aggregate RMSE rises to 0.00732268, including channel 3 RMSE 0.00238174.

This establishes that the combined DI/threshold pair is associated with a small non-zero decoded-audio difference even with decoder DRC disabled, and a much larger playback difference with decoder DRC enabled. Because both settings changed together and the true/15 XML was not hash-bound, it does not isolate either control's contribution. It also shows that this pair does not explain the matched Rust-vs-DEE channel-3 residual when decoder DRC is disabled.

FFmpeg 8.1 was used only to decode to in-memory f32le pipes; no PCM or encoded output was saved or included.
