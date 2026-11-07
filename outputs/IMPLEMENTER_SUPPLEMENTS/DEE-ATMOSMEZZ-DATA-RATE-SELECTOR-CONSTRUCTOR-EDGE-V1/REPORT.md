# AtmosMezz data_rate string-to-selector and constructor edge

## Static mapping

The hash-bound Ghidra 12.2 trace identifies a static lookup table mapping XML text to U32 selector codes: `384→0`, `448→1`, `576→2`, `640→3`, `768→4`, `1024→5`. The selected job XML contains `data_rate=1024`; statically, if that text is supplied to this lookup, the result is selector code 5. This is not a runtime observation.

`FUN_1801535b0` looks up the workflow-parameter rate text and writes the mapped U32 into frame-local `+0xD8`. When `processor+0xFC8 == 2`, the static path passes that frame record through `FUN_1801746c0 → FUN_1801746d0`; the `+0xD8` field address reaches `FUN_180177e50/eb0`, which copies its first U32 into provider state `param_1[0]`. `FUN_180178dc0` then reads this code and writes `open_config[10]`. The constructor transfer is statically bound, conditional on the `+0xFC8==2` branch.

## Exact remaining join

The remaining edge is whether this hash-bound job's XML `data_rate` value populates the selected runtime `Priv+0x6A` workflow-parameter map for the same invocation, and whether `processor+0xFC8==2` is selected. No runtime capture exists. Therefore the static six-value mapping is now known, but all-rate runtime support, actual writer behavior, and full encode success remain unproven.

## Provenance

Upstream Wash-only `MANIFEST.sha256` file SHA-256: `5c92d48a60682928e4fb9abd6f04c7e2062c19e31b27587a93ddf8452f284e81`. Wash recomputed its three listed payload hashes, twelve referenced Ghidra analysis artifact hashes, and the selected job XML, schema, and parent evidence hashes. No separate upstream package seal was supplied. The upstream source modified no parent package and did not run DEE or read media/PCM/payload/key/license. Raw logs/scripts are not copied into this supplement.
