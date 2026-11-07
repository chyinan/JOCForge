# Line DRC serializer callee return boundary

Reference-only Wash supplement. It confirms the bounded callee body ends immediately after a zero initialization, not a DRC payload or BSI write. `IMPLEMENTATION_READY=NO`.

The serializer callee `FUN_1801c24a0` writes U32 zero at the object passed in RCX, offset `+0x0`; the only following instruction is `RET`. No later field, output write, snapshot `+0xA0` alias, or BSI relation exists inside this function body. The pointer's semantic field and owner/generation remain unbound.