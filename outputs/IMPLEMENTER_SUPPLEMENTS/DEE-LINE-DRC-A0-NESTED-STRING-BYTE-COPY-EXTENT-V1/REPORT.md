# Line-mode DRC nested string byte-copy boundary

## Bound generic copy

The selected-row string helper calls FUN_18059a7e0 with destination, source+start, and a U64 byte extent. In the inspected call, start is zero and the preceding helper clamps the requested sentinel length to the finite source length. The byte-copy helper uses the extent for small and chunked copy paths and selects a backward copy if source/destination overlap; otherwise it copies forward. Its return is the destination pointer, after which the caller writes string length and NUL terminator.

## Remaining boundary

This is a generic string-byte copy, not a codec/BSI record writer. The selected row has no typed owner join to snapshot+0xA0; no codec/BSI owner or later consumer is reached. Runtime string bytes, generation, and clock are not captured. No source bytes were read.

Wash independently verified the upstream manifest, seven payload hashes, SHA256SUMS, and package seal. The upstream agent reported normal and python -O checks passing; no checker script was included. Raw traces are excluded.
