# Composite selector direct-input boundary

## Bound static rule

`FUN_1805506b0` writes selector byte `T+0x100`. It writes 1 when either the U8 key-2 lookup result on `param_7` is nonzero or the source-descriptor comparison `FUN_1805505d0(param_2)` is nonzero; otherwise it writes 0. The same `param_7` pointer is forwarded to `FUN_18054fa70`, which selects ordinary mode when `T+0x100 == 0` and extended mode when nonzero.

The key-2 lookup passes byte key value 2 to `FUN_18054ea90`. The helper traverses a byte-key tree and returns a longlong traversal/range result. This selector edge tests only whether that result is nonzero; it does not read the mapped option payload. The key's external name, payload type and value semantics remain unbound.

The descriptor comparator checks U16 descriptor count at `+0x0A`, then each six-byte row's U16 at `+0`, U8 tag at `+2`, and U16 at `+4`. Byte `+3` participates in a U32 load but not the comparison. In the previously sealed valid count range 1–128, mismatch returns nonzero and equality returns zero. `T+0x08` and `T+0x102` are separate fields and do not define this branch.

## Open boundary

The package does not bind `param_2` to a concrete opened-K owner/generation, does not identify the key-2 payload or XML/UI field, and does not capture the selected runtime mode. It therefore closes the direct Boolean rule, not a per-sample selection or audio effect.

The input-only vectors are a symbolic truth table derived from the static OR predicate. They are not sample vectors. No source code, decompilation, executable, PCM, payload, key or license is included. No DEE run or implementation contact occurred.

