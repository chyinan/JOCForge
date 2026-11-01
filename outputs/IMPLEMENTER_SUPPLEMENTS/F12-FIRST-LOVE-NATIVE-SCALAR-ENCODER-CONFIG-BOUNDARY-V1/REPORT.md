# First Love native scalar to encoder-config boundary

## Scope

This supplement records one authorized `measure_only` scalar observation and the static Ghidra path that copies the aggregate float into the PE encoder configuration. It does not assign a unit to the scalar or close its conversion to the standard E-AC-3 BSI DialNorm field.

## Native observation

On the authorized First Love input identity inherited from the prior receipt, a bounded `measure_only` job with meter mode `1770-4`, dialogue intelligence off, threshold 100, and `file_position` window `2160–2208` exited 0 with the PE backend and reported scalar `-21.010569`. The native log stated no unit. The input SHA was not recomputed in this continuation; file size and last-write time matched the prior identity receipt. Its inherited authorized hash differs from the three earlier cross-sample hashes, but this observation does not establish a universal scalar rule. A bounded RIFF/AXML scan read 78,630 bytes of metadata and chunk headers and read zero bytes from the PCM data chunk; DBMD was not parsed. The temporary output file was created, then removed without reading any of its contents.

## Static float handoff

The Ghidra trace identifies a binary32 max aggregator initialized to `-120.0` over per-source float fields at row offset `+0x14`, stride `0x20`. It copies the aggregate into owner `+0xF6C`, then copies four bytes to encode-config `+0x114`. The PE branch passes the subobject at `+0xD8`; a bit-preserving qword copy from subobject `+0x38` places the scalar bits at the `EmpireEnc` state field relative `+0x3C`. No unit conversion, rounding, or quantization was found before the encoder API wrapper.

This bounds a config handoff, not final bitstream behavior. A separate per-frame write to the same `EmpireEnc +0x3C` comes from `param_4[10]`; its relation to the measured scalar is not proven. Another `+0x3C` in an `EmpireLoudness` object is an enum mapped from `Compr` to `DDP_COMPR` and is a different field.

## Open behavior

The scalar's native unit, any meter scale/quantization, the per-frame overwrite's source, the final BSI DialNorm field, and plugin-internal rounding remain unproven. The candidate half-step values in the companion boundary note are hypothetical LUFS-input discriminators only; they were not run natively and predict no serialized value.

No private key/HMAC/license material was read. DEE processed the authorized input internally for the measurement. No PCM, payload, bitstream, or output contents were manually read or retained. No new user option is indicated.
