# 59.94 numeric file-position vector

## Bounded addition to the active JOC writer rule

For the selected 38-channel ADM BWF fixture, a 59.94 filter-rate override, file-position time base and delimiter-free numeric frame tokens, the active source conversion uses nominal rate 60 with scale 1001/1000. A nonzero inclusive end token is advanced one frame. The active JOC writer truncates the positive normalized sample products toward zero; `SampleRange` rounding remains limited to validation.

Run-55 used tokens 2160 and 2172 at 48 kHz. The source interval is 0.216875 seconds; the separately probed encoded track is 0.224 seconds. The final product is 1,740,138.4 samples, truncated to exclusive end 1,740,138. The native reference is a nonempty 1024 kbps, six-channel EC-3/JOC stream of 28,672 bytes.

This vector extends the 29.97 active-writer result but does not itself distinguish round from truncation because the fractional part is .4; v4's 29.97 dynamic capture distinguishes those operations. The 23.976 branch remains static-only and unrun. No model output/comparison ran. Do not conflate source interval with encoded-track duration.
