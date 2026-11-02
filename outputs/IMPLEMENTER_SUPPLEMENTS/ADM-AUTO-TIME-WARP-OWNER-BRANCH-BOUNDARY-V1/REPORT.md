# Auto-time branch and per-open owner boundary

## Wash disposition

The upstream Ghidra 12.2 supplement closes one control-flow edge for the same `PrivContext`: a `dont_change` selector chooses between `setRange` and `runWarpModeChange`/master-open. It clarifies why an opened owner cannot be assumed to be the source of values consumed in the other branch.

## Bounded static facts

- `FUN_180075a90` reads the same `PrivContext*` from `FilterConvertAtmosMezz+0xb8`; the `dont_change` comparison selects either `FUN_18007dc20` (`PrivContext::setRange`) or `FUN_18007eba0` (warp-mode change/master-open).
- The `runWarpModeChange` branch reads the filename and creates/opens/closes a per-open owner. In that selected function path, it does not write the offset/FFOA map entries and does not pass the owner to `setRange`.
- The `setRange` branch reads the `offset` and `first_frame_of_action` map keys. The sample-6 receipt does not record the branch selector value, so this package cannot establish which behavior occurred in that run.
- A static reader seek candidate has not been joined to the `setRange` sample position.

## Boundary and MVP

The edge from auto metadata into an effective numeric offset/FFOA remains `UNBOUND`; the `dont_change` branch value and reader-seek join remain unobserved. Default `file_position` MVP is not blocked. No DEE/native/media/key run occurred.
