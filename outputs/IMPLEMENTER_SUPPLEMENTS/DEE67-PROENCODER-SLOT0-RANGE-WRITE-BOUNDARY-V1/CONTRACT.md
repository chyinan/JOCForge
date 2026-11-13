# ProEncoder slot-0 range-write boundary

## Bounded observation

In the hash-bound candidate image, the `DDPJocProEncoder` vtable base is `0x140b36290`. Slot 0 points to `0x1402ac530`. Its first basic block has nine decoded instructions and stops at `CALL 0x1402abcf0` at `0x1402ac54d`.

Before that call, the block writes the vtable pointer to object offset `this+0x0` at `0x1402ac544`. It has zero direct writes to `this+0x190`, `this+0x198`, or the `this+0x140` subobject offsets `+0x50/+0x58`.

## Boundary

This establishes only the selected slot-0 first-block writes. It does not establish range population by any other slot/helper, nor an ADM row/D0 role or source-generation relationship. The external span populator remains `UNBOUND`; a caller-owned span interface is still a candidate seam with `production_allowed=false`.

This sanitized note contains no disassembly, source code, media, PCM, or payload bytes.
