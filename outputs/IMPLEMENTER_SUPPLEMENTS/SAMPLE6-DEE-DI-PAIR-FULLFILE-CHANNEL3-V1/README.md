# Full-file DI/threshold pair: channel 3

This Wash supplement extends the hash-bound true/15 versus false/100 DEE control pair to the full programme for output channel index 3. The pair uses the same authorized ADM and matched 1024 kbps / film_light / 5.1 controls; DI and speech threshold change together.

With decoder DRC scale 0, channel 3 is exact in 6,521,344 of 6,523,392 sample frames (99.9686%). The remaining 2,048 samples occur only in four-second blocks 18 and 21; whole-file RMSE is 5.41e-7. With decoder DRC scale 1, channel 3 differs in all 34 blocks (RMSE 0.00193218).

This confirms the combined DI/threshold pair is not the cause of the persistent Rust-vs-DEE channel-3 residual observed at decoder DRC scale 0. It can change playback when decoder DRC is enabled. Individual DI versus threshold effects are not isolated, and no native channel identity or processing cause is inferred.
