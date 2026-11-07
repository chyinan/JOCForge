# Alternate +0x164 reader candidate boundary

One existing static candidate in a separate DDP encoder image reads a field at +0x164, compares it, and passes a derived value onward. Its input owner comes from a child-record array in that EXE.

The established Bool-conditioned subcontext +0x164 belongs to a different Atmos DLL engine path. No typed pointer/owner edge joins the EXE child-record candidate to that DLL subcontext. This candidate therefore does not change the established result boundary: Bool-to-Float32 result causality remains UNBOUND. This is one candidate, not a global xref or absence claim.

Ghidra 12.2 static receipt only. No DEE run, runtime capture, media, PCM, payload, key, or license access occurred.
