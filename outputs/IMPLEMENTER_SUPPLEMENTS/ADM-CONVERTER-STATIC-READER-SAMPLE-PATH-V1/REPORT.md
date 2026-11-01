# ADM converter static sample-read path

## Result

The selected converter image has a statically traced path from its `AdmFileStorage` read entry to a bounded file-read callback. The path explains cursor and byte-offset handling for an already supplied row table. It does not establish which physical BWF rows populate that table, the runtime callback target, actual reads or EOF, or any link to the measurement owner. Treat this as an input-adapter model only; it does not establish an E-AC-3/JOC audio rule or explain the reported attempt-10 import failure.

## Hash-bound source and static facts

Source package: `adm_storage_reader_static_trace_wash_v2`, source MANIFEST SHA-256 `273fea3edf9a06b7670a846be254710cbfc923d60d862a7d5f4b02ca26882f2c`. Its normal and optimized checkers both reported `PACKAGE_VERIFIED files=31` and the same target image SHA-256.

Target image: `D:\Programs\dee71joc\test_dee\dee_audio_filter_convert_atmos_mezz.dll`, SHA-256 `9972be4f952ca5f9121fa83ef941b29d4c1e6baf0ff1f708baaaedffa1bd76a8`, size 6,286,336 bytes, analyzed in Ghidra 12.2. The separately named E-drive image has a different hash and is outside this evidence.

The washed static trace is limited to these source-identified behaviors:

1. `AdmFileStorage` vtable slot `+0x20` dispatches to image RVA `0x223430`, which enters `FUN_1802195e0`.
2. `FUN_1802195e0` obtains the current position and available extent, clamps the requested amount to the remaining extent, seeks its nested source, clears the supplied rows, calls `FUN_18022d1e0`, then advances its cursor by the callee's returned amount.
3. `FUN_18022d1e0` reads channel count from source offset `+0xc8` and row-offset table from `+0xe8`. It adjusts existing row pointers by `sample_offset * 4` and passes the table to the format-reader state at `+0xc0`; this trace does not construct the row table from CHNA.
4. `FUN_180236c40` stores an input callback table and assigns `FUN_180241460` to reader-state offset `+0xf0`. `FUN_180241460` bounds a sample-byte request against the data chunk and file extent, computes `data_payload_offset + relative_byte_offset`, and tail-dispatches callback-table slot `+0x18` with context, absolute offset, destination, and bounded length. The caller checks returned counts for 1–4 byte sample units.
5. RIFF chunk-header reads use callback-table slot `+0x10` for an 8-byte header. That scan cursor is distinct from the outer ADM sample cursor.

## Limits

- **UNBOUND:** `CHNA track_index - 1` / `ConverterChannelInputId` to runtime row-pointer ordering and the concrete `+0x18` callback instance.
- **NOT_RUN:** actual return counts, EOF behavior, converter/encoder execution, playback.
- **UNBOUND:** meter descriptor identity and owner/generation join.
- No BWF `data` bytes, PCM, EC3 output, or private key material were read in this analysis.
- No claim is made that this static path is present in the attempt-10 process or binary.

The next smallest runtime closure, if later authorized, is a single metadata-only read against this exact image hash: capture the pre-call row-pointer identities, request/cursor/extent, callback target and returned count without dereferencing or persisting destination bytes. Until then the runtime edges remain open.
