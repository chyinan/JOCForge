# DDP time_base setter helper container boundary

The nested setter passes a stack-local temporary to one helper. The helper's first bounded operations validate count/allocation conditions. One branch reads an allocator header; the ordinary cleanup path forwards the temporary storage pointer to a cleanup helper. The helper body does not receive a proven time_base descriptor address at +0x550 in this call.

The temporary container's relationship to a typed option descriptor or writer remains UNBOUND. No +0x550 write and no XML value/omission transfer is established. The cleanup helper body was not inspected. Static Ghidra evidence only; no DEE or media/payload/key/license access.
