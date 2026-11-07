# Line DRC serializer +0x42f0 pointer first-use boundary

Reference-only Wash supplement. It records the first subsequent use of an input pointer, not the pointer's semantic type or a coded-field/BSI effect. `IMPLEMENTATION_READY=NO`.

In `FUN_1802001a0`, the 64-bit pointer loaded from `[RCX+0x42f0]` at `0x1802001be` is next used unchanged as RCX at `0x1802001cb` for direct callee `FUN_1801c24a0`. The intervening instructions do not rewrite RCX. The callee body was not inspected.

The pointer type/owner and same-record link to mode-1 snapshot `+0xA0` remain unbound. No serializer output/BSI relation or audio effect is established.