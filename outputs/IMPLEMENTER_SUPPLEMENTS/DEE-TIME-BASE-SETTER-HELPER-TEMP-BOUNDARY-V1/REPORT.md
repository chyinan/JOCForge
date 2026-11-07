# DDP time_base setter helper argument boundary

The selected nested setter calls one direct helper with a stack-local temporary. The bounded argument setup passes the temporary's stack address, not the nested stage owner's option-descriptor address at +0x550. The helper body was not inspected.

The first unresolved edge is temporary property object to a typed option descriptor/writer. The +0x550 write and XML time_base value/omission transfer remain UNBOUND. Static Ghidra 12.2 only; no DEE, media, PCM, payload, key, or license access.
