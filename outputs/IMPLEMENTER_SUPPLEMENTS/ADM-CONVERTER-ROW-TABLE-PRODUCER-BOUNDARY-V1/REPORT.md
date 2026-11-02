# ADM converter row-table producer boundary

## Result

The hash-bound converter image statically connects an `AdmFileStorage` object at `this+0x98` to its `DlbWavFile` reader. The reader's channel count at `+0xc8` comes from parsed `fmt` information, and its `+0xe8` table allocates `channel_count * 8` bytes for row pointers. On the selected read path, `FUN_1802195e0` passes the existing caller row list to `FUN_18022d1e0`, which writes `internal_row[i] = caller_row[i] + sample_offset * 4` in ordinal order before conversion. The typed `writeData<int>` path similarly supplies row pointers from `MultiChannelBuffer<int> +0x18` to `FUN_18022de30`/`FUN_18022dee0`.

These routines preserve the order of a caller-supplied row-pointer list; they do not choose rows from a CHNA key. The upstream converter map binds `CHNA track_index-1` to a typed `ConverterChannelInputId` descriptor, but the connection from that descriptor to the ordered physical BWF rows remains `UNBOUND`. Numeric equality in the 38-row sample is not used as identity.

## Authorized input handling

The package reports a pure-ASCII alias of the authorized 38-row “人是猫” ADM with the inherited file SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`. The source copy and alias hash read the whole file; no PCM values were decoded, interpreted, or exported. The target converter PE SHA-256 is `9972be4f952ca5f9121fa83ef941b29d4c1e6baf0ff1f708baaaedffa1bd76a8`.

## Limits

- **CLOSED_STATIC:** row pointer allocation, caller-order copy and sample-offset adjustment.
- **UNBOUND:** CHNA / `ConverterChannelInputId` to `MultiChannelBuffer` row order and physical BWF slot.
- **NOT_RUN:** runtime pointer values, reader/EOF, playback; no DEE/converter run was made because the runtime callback/keyless boundary was not proven.

This is an input-adapter rule, not a JOC or E-AC-3 coding rule. Do not infer channel identity from row index or equal numeric IDs.
