# Full-file matched PCM residual

This Wash package records a full-file comparison of the already-existing matched-control DEE and Rust E-AC-3 outputs generated from the same authorized ADM. Both files have the same 48 kHz, 5.1(side), 1024 kbps stream geometry and 4,247 AUs.

FFmpeg 8.1 was used only as a decode-only measurement tool. Its f32le outputs were streamed through bounded in-memory chunks; no PCM was written or retained. No encoding or signal-processing behavior was copied from FFmpeg.

Across the full 135.904 s, decoded channel index 3 has the largest RMSE (0.00619955), correlation 0.817673, and ranks first in RMSE in 30 of 34 four-second blocks. Its RMSE is 4.275 times the next channel. Channel index 4 has the largest single peak error, so channel 3 is the persistent RMS residual, not the only peak event.

This localizes the dominant remaining difference to one decoded output slot. It does not identify the DEE input-row identity or algorithmic cause, prove full PCM equality, or establish Atmos rendering/listening parity.
