# DI Bool helper-state handoff boundary

## Bound edge

In the selected `FUN_1801af230` constructor path, the return from `FUN_1801aecf0` is stored at outer subcontext `+0x18`. The same constructor writes mode Bool `+0x164` on conditional paths. It then passes the helper-return pointer from `+0x18` to `FUN_1801af030` as `RCX`, with a separate local pointer as `RDX`. `FUN_1801af030` delegates to `FUN_1801aea50`, updates nested state `+0x24`, and writes status at the first argument's `+0x14`.

This establishes a static helper-pointer handoff to the initializer. It does not establish that Bool `+0x164` determines the helper-return object's fields, delegated initializer state, or the per-channel gate context used by the numeric consumer. The first unresolved lineage remains `outer +0x164 -> helper-return/delegated state`; status-context generation remains open.

## Disposition

This is not a DI implementation rule. The existing Human Cat A/B is a one-input observation; no new DEE/A-B/runtime run occurred. Preserve `UNBOUND/NOT_RUN` and do not infer that DI is ignored or applied from this constructor edge.

## Exclusions

The supplement contains sanitized findings and source hashes only; no Ghidra script/log, binary, media, PCM, encoded payload, key, or license content is included.
