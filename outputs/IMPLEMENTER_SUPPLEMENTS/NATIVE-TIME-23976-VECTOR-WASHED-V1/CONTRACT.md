# 23.976 numeric file-position vector

## Bounded addition to the active JOC writer rule

For the selected 38-channel ADM BWF fixture, a 23.976 filter-rate override, file-position time base and delimiter-free numeric frame tokens, the static special-rate mapping uses nominal rate 24 with scale 1001/1000. A nonzero inclusive end token is advanced one frame. The active JOC writer truncates positive normalized sample products toward zero; `SampleRange` rounds for validation only.

Run-56 used tokens 2160 and 2172 at 48 kHz. This gives `[4,324,320, 4,350,346)` samples, a 26,026-sample source interval of 0.542208333 seconds. The nonempty native reference is a 1024 kbps, six-channel EC-3/JOC stream of 69,632 bytes and 0.544 seconds; that encoded-track duration is distinct from the selected source interval.

Both sample products are integral, so this vector independently confirms the 23.976 conversion and native range but does not distinguish rounding from truncation. The 29.97 dynamic writer capture in v4 establishes that distinction. No model output or comparison ran.
