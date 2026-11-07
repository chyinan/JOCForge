# Line DRC row owner first-break

## Reused observations

Two sealed, bounded ADM observations were reused without rerunning A/B. `music_standard` has a different per-block `dynrng` sequence from `film_light` in both measured windows. `music_light` matches the measured `dynrng` fields in those windows only; this does not establish a general alias.

## Static ownership refinement

In the inspected mode-1 path, the Line XML token conditionally maps to a plugin profile-tree U32 stored at snapshot `+0xA0` (`music_standard=3`, `music_light=2`). A same-snapshot pointer also reaches the per-frame packer. The selected row range is described by begin/end pointers at snapshot-relative `+0/+8`, with 0x20-byte rows, so the row table and `+0xA0` belong to the same static snapshot base.

The row contents remain distinct from the `+0xA0` profile U32: no value/alias transfer or reference to that field was established. The first row helper therefore does not close the profile value to a core selector or `dynrng` producer. Row-content-to-`+0xA0` and mode-to-core-selector/dynrng remain `UNBOUND`; runtime owner generation and selected-row identity remain `NOT_CAPTURED`.

## Implementation boundary

There is still no supported mode-to-curve, coefficient, or per-frame `dynrng` rule. Do not alias `music_light` to `film_light` globally from two windows, or substitute RF `compr` codes for Line curve selectors. No algorithm change is authorized by this boundary package.

## Exclusions

This handoff includes only sanitized findings and source receipt hashes. It contains no Ghidra script/log, binary, media, PCM, payload, key, or license content. No DEE, Ghidra, A/B, or PCM run was performed in the row-owner follow-up.
