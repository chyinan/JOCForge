# data_rate XML property-loop first break

## Bound caller branch

In the bounded `FUN_14002a0b0` property branch, candidate node `local_1978` is first passed to opaque boolean gate `FUN_140054fe0` at VA `0x14002b5f3`. This caller branch contains no `data_rate` key comparison or field-specific value conversion.

On the gate's true branch, the node is passed to `FUN_1400244c0` at VA `0x14002b63e`; its output is then iterated as generic 0x68-byte records through `FUN_140072630`. This identifies the next helper to inspect but does not prove that it creates the `data_rate` leaf record.

## First break

The `data_rate` leaf identity, field-specific key/value/type conversion, and record writer remain `UNBOUND`. Helper bodies and actual plugin selection are outside this bounded slice; runtime is `NOT_RUN`. Do not treat the candidate node or generic record loop as a confirmed rate field.

## Exclusions

This supplement contains sanitized facts and source receipts only. It has no binary, raw Ghidra decompilation, XML, media, payload, PCM, key, or license content.
