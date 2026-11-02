# F12 aggregate scalar to five-bit U5: scoped candidate rule

## Actionable candidate

For the tested `measure_only / meter=1770-4 / dialogue_intelligence=false / DRC=none / 5.1 / 1024 kbps` profile and bounded file-position window, the best-supported candidate is to interpret the DEE aggregate scalar as F32 `selected_lufs`, then calculate:

```text
if abs(F32(x - (-120.0))) < 0.000001: U5 = 31
else: t = clamp(trunc_toward_zero(F32(x - 0.5)), -31, -1); U5 = -t
```

This is ready for a scoped implementation trial; it is not a universal native-unit claim. Confidence is `0.82` (medium-high for the stated profile/sample evidence). Three different authorized inputs have same-run scalar/U5 observations matching the formula: `-20.665958 → 21`, `-14.418104 → 14`, and `-17.093924 → 17`. The First Love scalar `-21.010569` predicts `21` but has no observed U5 and is not counted as a match.

## Native observations and remaining lineage

Ghidra static evidence shows the aggregate stage initializes to `-120.0`, traverses 32-byte per-source entries, reads an F32 at `+0x14`, and retains the maximum. A separate open-config path reads `EmpireEnc+0x3c` F32 and runs a normalizer that checks the sentinel, subtracts `0.5` in F32, converts with `CVTTSS2SI` (truncate toward zero), and clamps to `[-31,-1]`. That normalized result is observed in a mismatch comparison/flag, not in a direct BSI write.

The separate DDPi path copies `stream+0x9c` through nested context fields and validates it in `[1,31]`; the measured scalar to that field and the final BSI U5 writer remain `UNBOUND`. The scalar unit is not stated in the native log. Keep this lineage distinction explicit, but do not make it a gate for the scoped MVP candidate.

The candidate affects the five-bit U5 at F12S bits 45–49. The three observed header prefixes kept other reported fields equal and values were all valid U5 codes. The checks read seven prefix bytes and zero payload-body bytes; no raw prefix was retained and no official full-file CLI cross-check was run.

## Provenance and limits

The same-run Hanmao receipt binds the 38-channel `人是猫` input SHA, a successful DEE attempt, aggregate scalar, and observed U5. Its corrected parent provenance distinguishes the cross-input `MANIFEST.json` digest from the `MANIFEST.sha256` sidecar-file digest; both cited payload files match. No new DEE run, PCM read, private-key read, or implementation source is included.
