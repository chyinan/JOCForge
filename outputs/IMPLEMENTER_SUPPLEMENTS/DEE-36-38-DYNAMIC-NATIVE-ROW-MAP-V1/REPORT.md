# DEE native 36/38 channel-map dimensions versus the 48-ID F03W axis

## Wash conclusion

The hash-checked Ghidra receipt identifies a native configuration row map whose length is derived from its input configuration count; it is not a fixed 48-row table. For a conditional 36-row candidate, rows 36/37 do not exist in that map. This does not prove how those rows relate to the separate 48-wide F03W global-source-ID domain.

## Native row construction

- `FUN_1804d43c0` iterates object source records using a count getter at vtable `+0xC0` and row getter at `+0xE0`. For each record it reads a U32 map-slot field at record `+0x4C`, writes a `0xFF` marker to map slot `+2`, copies a U16 field from record `+0x48` to map `+0`, and stores object ordinal `i` at map `+4`.
- Normal source count is object-source count plus the summed bed channel-vector lengths divided by 16. The configuration count is U16; map rows are six bytes.
- Normal composite mode copies each six-byte child row by ordinal.
- Special-map mode counts `0xFF` object markers per child, takes the maximum, then emits ten bed rows followed by object ordinal `k` at map row `10+k`. Under the conditional “all markers in selected child” model, 26 objects yield 36 rows and 28 yield 38.
- `FUN_180419720` copies map row `i` to core row `i` after allocating `6*count` bytes. The traced map is dynamic 36/38, not a 48-row table.

## Separate F03W domain and missing-slot semantics

The approved F03W model has a 48-wide global-source axis: IDs 0–10 are the prefix and regular IDs are 11–47. Under the approved physical-order rule with one LFE excluded from regular rows, a 36/10/26 input assigns regular IDs 11–45, leaving IDs 46/47 unassigned; a 38/10/28 input assigns 11–47. F13 payload/ID14 ordinals, native six-byte map rows, and F03W global-source IDs are distinct domains.

No zero-fill claim is supported. The receipt does not observe the selected native U16 config count for a user sample, the actual child partition/marker maximum, CHNA physical-slot identity to object-source record, map slot to F03R global source ID, or native rows 36/37 to IDs 46/47. Thus it closes native map dimensionality but leaves the two missing global-source semantics `UNBOUND`.
