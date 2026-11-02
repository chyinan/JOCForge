# Selector-8 writer xref exclusions

## Wash disposition

The upstream R2 audit verified a bounded set of static xrefs and indirect-call candidates for `FUN_1804d6980`. It found no selected-instance caller in the scoped path. This is a candidate-exclusion receipt, not a positive owner binding.

## Candidate results

- The exact `FUN_1804d6980` code slot is the `atmos_storage_master_s` vtable slot at `0x1807f9c70`; three additional references are data-table references, not code callers.
- The bounded callback-path scan found two `+0xa8` calls in `FUN_1804d43c0`; both use the already-rejected inner `[R12]` receiver and return an iteration upper bound.
- Three `+0xa8` calls in `FUN_18054a900` use newly constructed `MasterSourceImpl` or `BlankMasterSource` receivers. Their vtable slots resolve to `FUN_1803ae440`, which only writes `param_2` to `this+8`; it is not the `FUN_1804d6980` writer.
- The refreshed table lookup shows `0x180823430` is a `BlankMasterSource` slot for `FUN_1803ae440`, not a reference to `FUN_1804d6980`.

## Boundary

The selected `atmos_storage_master_s` selector-8 writer caller, `param_3`, qword-0 pointee type, and that pointee's `+0x18` first writer remain `UNBOUND`; selected job remains `NOT_OBSERVED`. The next proof needs a single metadata-only observation at the writer entry and a one-byte construction-time watch, as described in the upstream receipt. No DEE run, media, PCM, key, or license access occurred.
