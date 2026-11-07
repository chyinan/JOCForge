# Line DRC mode-1 snapshot boundary

Reference-only Wash supplement; it does not establish DRC effect or authorize a behavior change. `IMPLEMENTATION_READY=NO`.

Reusing the sealed Line token/map evidence, the selected static mode-1 path forwards the same snapshot base from `FUN_1801475f0` through `FUN_1801754f0` into `FUN_180175500`. That function passes the base to validator `FUN_180177230` and per-frame packer `FUN_1801764e0`.

The validator/packer trace is scoped: no direct read or copy of the typed candidate U32 at `snapshot+0xA0` was found on this edge. `FUN_180175500` reads `snapshot+0x130` for a different field. Thus pointer/aggregate flow is statically bound, but the `+0xA0` value is not bound to a codec/BSI input. XML runtime ownership, selected mode-1 generation, and indirect packer/helper relation remain open; audio effect is `NOT_RUN`.