# Force-D45 history and selector boundary

## Static DEE rules

For the hash-bound single-pass DDP plugin, five eight-byte lane records at owner `+0x135C0` are cold-zeroed. The first U16 in each lane is copied without scaling into a six-block-by-five-lane history at owner `+0x15208`, subject to `param_1==0` and the active-lane gate. A cross-image scan found no later direct source-word write in the seven scanned images, but indirect or unscanned writers are not excluded.

The Force-D45 selector reads U16 history cells from `owner+0x140F0 + 2*(block+6*channel)` and stages codes `0..3` at `owner+0x1BAE8 + 2*(0x1D+channel+5*block)`. For the six-slot branch, zero-run thresholds are 2 and 4; a short-window branch changes those thresholds. A separate nonzero control word can promote codes 1/2 to 3. Selector table matching and missing-selector status are gates before staging.

The staging table reaches the DDP stuffer setup path, but no named EC3 metadata field or wire bit range was identified. The output cells remain semantically `UNKNOWN` at the bitstream boundary.

## Runtime boundary

Attempts 10 and 11 did not hit owner/history targets. Attempt 12 loaded the hash-matched single-pass plugin but the WAVE stage failed before PCM-frame delivery because it could not access MediaInfo. No runtime Force-D45 values or bitstream were captured. The exact same-root DEE/MediaInfo/plugin combination is unavailable in the checked paths.

## Disposition

This package gives bounded static DEE rules and input-only branch examples. It does not establish that the active R138 run uses this exact owner, the meaning of the staged codes, or their serialized output. Keep those joins `UNBOUND/UNKNOWN`; do not substitute another encoder's behavior.
