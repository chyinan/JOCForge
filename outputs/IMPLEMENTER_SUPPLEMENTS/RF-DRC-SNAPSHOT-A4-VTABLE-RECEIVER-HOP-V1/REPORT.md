# RF snapshot +0xA4 to constructed per-frame receiver boundary

## Bound pointer hop

`FUN_1801764e0` receives the snapshot pointer as `param_3` and writes its out-pointer with an object from one of three constructors (`FUN_1801850a0`, `FUN_180185100`, `FUN_180186e40`), selected by other snapshot fields. `FUN_1801754f0` loads that output as `plVar4` and calls vtable slot `+0x18` with `plVar4` as receiver and a local frame buffer as explicit data argument.

The bounded helper body contains no direct snapshot `+0xA4` access. The immediate vtable call does not pass the snapshot pointer or an explicit scalar sourced from `+0xA4`. This bounds only the checked hop: the constructed receiver's concrete class/slot target and any indirect copy of `+0xA4` remain `UNBOUND`; this is not a global no-consumer claim.

## Next edge

The next bounded trace is the selected constructor's vtable `+0x18` target and immediate typed input. Continue only while preserving the same snapshot owner; do not infer field identity from offsets.

Runtime RF values and selected constructor remain `NOT_CAPTURED`. No DEE execution, PCM/EC-3 payload read, or key/license access occurred.

## Provenance

Upstream manifest SHA-256: `09889d43ea572c9b6da76e7bbaa6e51ce125b42f9e15ce546d61244685a56a1e`; upstream package seal SHA-256: `e43d427ea8c68ef1bad1b1995e45b10a7b6c2f0a19569ba10c431b82653056d5`. The upstream package checker passed in normal and `python -O` modes. Raw Ghidra traces/scripts are excluded.
