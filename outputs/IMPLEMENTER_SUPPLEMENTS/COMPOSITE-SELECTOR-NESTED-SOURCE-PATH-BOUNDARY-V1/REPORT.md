# Nested composite-source path boundary

## Bound static path producer

The upstream Ghidra receipt follows the per-item reader route from an indexed 32-byte source-entry vector. Its string-like field begins at record offset 0 and is copied as a NUL-terminated string into the first 32 bytes of a 64-byte per-item carrier row. The carrier row is copied into the reader configuration/owner and later supplied to the storage-open dispatcher, either inline or through its heap-string pointer.

This binds the nested path's structural source and carrier flow. It does not identify the symbolic job/config field that owns the source-entry vector, recover the path bytes, or establish the runtime generation.

## Handler and K-count boundary

The prior handler-selection receipt remains the next gate: the actual nested path suffix is not captured, so its storage handler is UNBOUND. Do not substitute the top-level ADM/BWF filename. The prior candidate K-count formula is not re-derived here and is not applicable to a selected J+0x08 handler until that suffix-to-handler edge is resolved.

No new Ghidra query or native run occurred in this addendum. No path value, media, PCM, payload, key, or license was read. The upstream package is static-only, Ghidra 12.2, read-only/no-analysis, and its checker passed normally and under python -O.
