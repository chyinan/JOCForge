# Input timecode drop-frame candidate boundary

Reference-only Wash supplement. It binds two static facts for one input-only value: a candidate setter expression denotes byte offset `+0x188`, and a separate candidate reader reads its own object at `+0x188`. Their object identities are not joined.

If `29.97df` reaches the reused conversion function, it rewrites the rate text to `29.97` and writes a separate `drop_frame` property; the exact validator accepts `29.97` and rejects direct `29.97df`. This is conditional on the unbound route. The input XML value has not been shown to reach that conversion path, and no output timecode field was included.

The vector is descriptive and `NOT_RUN`. Do not infer ADM or AtmosMezz ownership from the candidate label. `IMPLEMENTATION_READY=NO`.
