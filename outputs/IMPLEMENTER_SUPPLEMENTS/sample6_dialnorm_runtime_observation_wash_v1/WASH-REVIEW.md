# Wash review: sample-6 DialNorm runtime observation

For one hash-bound sample-6 full-file DEE run, WinDbg bound the same DDP owner input member and the later serializer consumer. The owner U16 was `0x0019`; after a post-arm DWORD write of `0x00000019`, a later same-owner serializer consumed U16 `0x0019` with count 1 / width 5 (low bits `0x19`), and the bit writer returned 0.

This does not establish that the post-arm write changed the value: the pre-write value was not captured, the value is unchanged, and the setter copy into the serializer member occurred before that later write. The native F06 finalize member, units, producer/default and generation join remain unbound. The log's `-25.02` measurement units are unspecified and are not joined to `0x0019`; do not infer a conversion rule.

The output was hash-captured by R4 and is not included. FFprobe/OpenJOC are posthoc format/JOC metadata only; no audio playback, render, or decoded PCM was performed. Use only as a one-run structural/value reference for this exact input/profile, never as a general DialNorm algorithm or default.
