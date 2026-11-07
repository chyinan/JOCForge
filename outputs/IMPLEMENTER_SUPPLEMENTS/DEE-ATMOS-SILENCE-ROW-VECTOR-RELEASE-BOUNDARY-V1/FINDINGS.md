# Silence composite-row vector release boundary

## Bounded finding

The selected cleanup path forwards composite rows to `FUN_180173c70`, which clears row string members, then calls `FUN_18029c130` with the backing pointer and the number of 0x40-byte rows. `FUN_18029c130` validates allocation-size arithmetic, computes the storage byte count, and calls the generic free helper on the vector backing pointer.

It does not iterate rows or read per-row silence values at `+0x20/+0x38`. The selected cleanup branch therefore ends in generic row-vector storage release. It does not implement PCM zero-fill, Reader cursor/sample-range adjustment, PTS change, or AU-length change in the inspected function.

## Open boundary

This closes only the selected cleanup branch. The downstream allocator helper was not expanded, and no global consumer search was made. A separate audio consumer of the silence fields on another path remains `UNBOUND`; runtime remains `NOT_RUN`. Do not infer that silence options are globally ignored.

## Exclusions

The supplement contains sanitized findings and source hashes only; no Ghidra source/log, binary, media, PCM, payload, key, or license content.
