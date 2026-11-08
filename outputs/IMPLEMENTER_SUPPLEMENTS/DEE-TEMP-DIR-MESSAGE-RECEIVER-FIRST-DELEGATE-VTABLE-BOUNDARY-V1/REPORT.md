# Temp message receiver boundary

Reference-only Wash supplement. It binds the first delegate receiver's static object identity and the resolved function boundary for its vtable slot. It does not establish the delegate's file, cleanup, queue, or other runtime effect. `IMPLEMENTATION_READY=NO`.

The constructor writes the `dee::Messages::Message` vtable into the caller-provided object and returns that same object. In the inspected caller, the object is the 80-byte stack local `local_98`; its returned pointer flows as argument 3 to the next function and then as entry R8 to the delegate wrapper. The wrapper reads this object's vtable and dispatches slot `+0x18`, which resolves to function boundary `FUN_14006c760`.

The target body was not inspected. The path object's `clean_temp+0x90` field is not an explicit input to the observed call chain. Cleanup behavior, the relationship between the path object and this message object, and runtime owner/generation remain `UNBOUND` or `NOT_CAPTURED`.

The source package was independently checked normally and with `python -O`; its manifest and checksum-file hashes are recorded in the evidence. No DEE/WinDbg run or media, PCM, payload, key, or license access is included.
