# ADM row count to frame descriptor: conditional static lineage

## Static rule

For the identified `audio.atmos_mezz` → `encode_to_atmos_ddp` route, the ordinary-reader/PE branch statically traces an opened ADM/BWF descriptor through a source adapter and a composite descriptor into a process-frame descriptor. The selected runtime source instance, source-property owner, and generation were not observed; this contract is conditional on the identified factory and branch.

The ADM source count is computed as:

`individual_record_count + sum(group_entry_count for each group record)`

The accumulator is 32-bit. The opened descriptor stores the low 16 bits at `K+0x0A`; six-byte rows start at `K+0x10`. Individual rows use their UID-derived physical slot and tag 255; grouped rows use their UID-derived slot and the selected label tag.

The ordinary composite mode narrows the selected adapter count to U16 and copies exactly `count * 6` bytes. The extended mode, when its static condition is selected, computes `max(tag-255 rows across source adapters) + 10`, narrows to U16, creates ten prefix rows, and appends the maximum object-tagged suffix rows. Which mode a runtime job selects was not captured.

The reader resolves the exact composite descriptor `B` from `M+0x58`. The frame descriptor stores widened U16 `B+0x0A` at `D+0x08`; `D+0x10` is sample-frame size, not channel count; `D+0x28` points to the same `B`. In the inspected worker branch the same `D` address reaches the plain-reader getter and PE process wrapper. The PE wrapper allocates `N*1536*4` float storage and `N` channel pointers; the matrix copier iterates `N` rows for a 1536-sample frame. Its explicit `1536-or-zero-flush` check is a sample-frame-size condition, not an N-row limit.

No 36/38 row-count predicate was found before the opaque PE/JOC API boundary. This supports count-driven 36/38 mapping in the inspected wrapper; it does not prove full native acceptance for arbitrary N or define internal PE/JOC limits.

## Input-only vectors

- Ordinary mode preserves N if the opened K descriptor's native count is N.
- Extended mode: 10 bed rows plus 26 tag-255 object rows yields 36; 10 plus 28 yields 38.
- Both counts fit the U16 count fields and the inspected `N*6` row and `N*1536*4` allocation formulas.

These are formula vectors only; native values and runtime acceptance were not captured. Malformed indices, low-16 overflow acceptance, arbitrary ADM grammar, selected mode, and selected owner/generation remain out of scope.

## Evidence scope

- Upstream package: `ADM-FRAME-COUNT-LINEAGE-STATIC-V1`, manifest SHA-256 `46230ce4aa5fa9a0f060887282dbb7fe86c47c5e2ea67fbbe4e943eb97c31c0c`.
- Wash independently rehashed all six manifest files and all nine Ghidra receipt logs; all matched. The package's own checker receipt reports normal and `python -O` PASS; Wash did not rerun Python.
- Ghidra 12.2 read-only static analysis. Native execution, runtime counts, PCM/payload reads, and private key/license access are `NOT_RUN`/`NOT_READ`.
- This supplement excludes raw Ghidra logs, scripts, source, media, PCM, and payload.
