# DEE time endpoint metadata-index boundary

The bounded Ghidra receipt follows stored start/end indices to the first owner-bound consumer: they select two entries in a per-source metadata table while I/O sets are built. The evidence does not connect either value to input PCM frame slicing or a reader frame-count comparison. Nearby readers terminate through their own `hasNext`/container iteration; their termination is not joined to these endpoints.

Keep the endpoint-to-audio-frame clip `UNBOUND`, the inclusive/exclusive rule `NOT_CAPTURED`, runtime source-map selection `NOT_CAPTURED`, embedded-timecode clock alignment `UNBOUND`, and final audio effect `NOT_RUN`. The source query did not redo XML parsing or rounding and ran no DEE or decoder.

This supplement contains only sanitized behavior and parent seal hashes. It contains no source, decompilation, binary, raw XML, PCM, payload, key, authentication material, local absolute path, or raw native address.
