# RF DRC snapshot +0xA4 first-consumer boundary

## Same snapshot owner path

On the statically traced selector-mode-1 path, `FUN_1801475f0` passes one local snapshot at `RBP+0x400` to `FUN_1801535b0` as `param_2`, then later passes that same address to `FUN_1801754f0`. `FUN_1801754f0` forwards the same pointer to `FUN_180177230` and to per-frame helper `FUN_1801764e0`.

The existing bounded decompilation of `FUN_1801754f0` and `FUN_180177230` contains no direct `+0xA4` access. A focused Ghidra 12.2 scan of `FUN_1801764e0` found zero direct `+0xA4` memory operands and zero decompiled `+0xA4` references. This is a function-scoped negative finding, not a global absence claim. The separate `+0xE0` RF config path is not an alias or transfer for `+0xA4`.

## Exact first break

The snapshot owner is bound through the first same-owner consumer and per-frame helper, but no typed read/copy of snapshot `+0xA4` is established in this path. The typed receiver of the per-frame output object is also `UNBOUND`.

The next bounded edge is the output pointer produced through `FUN_1801764e0` (`&plStack_140` in `FUN_1801754f0`) to its immediate vtable `+0x18` receiver. Follow only whether that receiver gets a value originating from the same snapshot owner's `+0xA4`; stop the lineage when ownership is lost.

Runtime RF string/snapshot values and selected mode remain `NOT_CAPTURED`; typed `+0xA4` consumer and DRC effect remain `UNBOUND`. No DEE execution, payload/PCM/key/license read, or implementation contact occurred.

## Provenance

Upstream manifest SHA-256: `471360aa4221542f6ba7d47e22687e1b37cf171ae6283f842d9615212b583e10`; upstream package seal SHA-256: `c7acded5f269782aa878a05d3c74d4625c2c6fc64802d331893b901f202dcf0d0`. Upstream normal and `python -O` package checks passed. This supplement excludes Ghidra scripts, logs, traces, and binary/media payloads.
