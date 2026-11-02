# Native code13/code14 conditional audio selection boundary

## Best-supported native rule

For the normal selector branch, `base = selector == 0 ? 11 : 15`; `m` counts normalized U16 packet words with bit 15 set. The producer emits `code13=base+m` and `code14=m`, while `ID14.object_count=code13-code14=base`. `code13` is separately consumed as a BSI U8/control value; `ID14` stores `object_count-1` and drives a per-object loop with 28-byte records and float-buffer operations.

The pair is **not always metadata-only**. If `code14>0`, a skip index is unset, and `worker+0x182f0` exists, `FUN_1801c0770` scans up to `code13` 16-byte control records. Records with `+8==0` and type `+0x0c` in `{3,0x10}` set skip indices; those indices are omitted from the source-pointer list before per-object processing. The filter's runtime branch and worker table's ADM owner/generation remain unobserved.

## Audio identity boundary

The native six-byte packet `row+2/+4` fields are not yet typed to ADM AudioObject/ChannelFormat/CHNA. The optional 16-byte worker table is also not joined to an ADM generation. Do not implement this filter against ADM rows by matching counts or numeric values alone.

Across prior receipts, the authorized 38-row `人是猫` capture observed selector 5, `m=1`, code13=16, code14=1, ID14 count15; emitted U6 remains `NOT_RUN`. A 36-row `My jealousy` capture observed row10 normalized value `0x800a`, but not total `m`, code13 or code14. These are useful multi-ADM clues, not a runtime replay or a completed ADM-row mapping.

## Scope and next experiment

The source package checker passed normally and under `python -O`; Wash independently checked its manifest/seal and payload hashes. The slice ran no DEE job and read no PCM/payload/key/license. Next, bind one run's packet fields to named ADM rows and capture whether the worker source-selection table is populated; use the existing 36/38 row10 observations to choose a minimal comparison. Numeric/audio parity remains `NOT_RUN`.
