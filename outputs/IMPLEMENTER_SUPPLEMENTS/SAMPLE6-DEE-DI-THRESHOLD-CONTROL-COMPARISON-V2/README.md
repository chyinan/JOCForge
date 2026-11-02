# Hash-bound DEE dialogue-intelligence / speech-threshold control pair

This version replaces the earlier Wash supplement whose true/15 XML body was not hash-bound. A fresh full-file DEE run used the Wash-generated true/15 XML with the same authorized ADM; its XML hash, ADM hash, and output hash are now recorded. The output hash is byte-identical to the user-supplied true/15 DEE file.

The paired false/100 output is the fresh matched-control DEE run. Both outputs are 48 kHz, six-channel 5.1(side), 1024 kbps E-AC-3 JOC with 4,247 AUs and matching packet sizes/timing. Only the combined dialogue_intelligence/speech_threshold pair differs according to the two XML receipts.

In AU window [2750,2875), decoder DRC scale 0 yields a small non-zero PCM delta (RMSE 0.00018015), while channel index 3 is bit-identical. Decoder DRC scale 1 yields a much larger delta (RMSE 0.00732268). Thus this control pair affects decoded audio, but the comparison does not isolate DI from threshold and does not explain the persistent Rust-vs-DEE channel-index-3 residual when decoder DRC is disabled.

FFmpeg 8.1 was used only to decode to in-memory f32le pipes. No PCM, encoded payload, key, or license material is included.
