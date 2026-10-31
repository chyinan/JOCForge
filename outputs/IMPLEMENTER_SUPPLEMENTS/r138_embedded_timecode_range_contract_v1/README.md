# R138 embedded-timecode range contract supplement

This sanitized, implementation-safe supplement corrects the R138 range profile. It contains semantic rules and evidence hashes only; it contains no binary addresses, module names, source paths, decompilation output, raw XML, or payload.

R138 selects `embedded_timecode`, with start/end tokens `86640` and `86664`, both frame-rate settings `not_indicated`, source rate 48 kHz, and `offset`/`ffoa` set to `auto`. The resolved timecode frame rate is not captured. The embedded-timecode branch parses both tokens using the separately resolved frame rate, converts the resulting seconds to sample indices with `round(sample_rate * seconds)`, checks them against the source bound, then subtracts `offset` to obtain reader-relative bounds. Do not fill in an FPS from the tokens or from an equal `offset`/`ffoa` pair.

The file-position `max(offset, ffoa)-offset` rule remains valid only for its conditional file-position / first-frame-of-action route. It is not R138's selected route. The range-to-IO-set layout edge is statically bound, while its runtime owner and generation are not captured. The spatial query is a geometry-derived F32 value; it has not been shown to consume the same time domain as the embedded-timecode range.

The selected R138 call position `173286144` remains a derivation from the existing observed absolute start, relative start, and reader cursor. It is not a native absolute-position field and does not replace the embedded-timecode conversion rule.

The source writer for `offset=auto` and the resolved timecode frame rate remain unbound for R138. The endpoint convention and range-to-spatial time-domain join also remain unbound. Model replay is `NOT_RUN`; Gate 1 is open.
