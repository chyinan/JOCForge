# Atmos JOC ID-0 selector writer caller scope

## Incremental finding

The new hash-bound Ghidra package enumerates the direct in-image callers of
the `RenderBusConfigBase` constructor that writes `object+0x58` from caller
argument `param7`. The only direct caller located in this Atmos DLL is the
examined `RerenderBusConfig` constructor. Its visible type IDs are `3/4/5`,
so this constructor path is excluded from the selected type-ID-0 Speaker
Monitoring lookup.

Combined with the already washed lookup edge, the selector getter for the
selected ID-0 object remains `vtable+0x38`, with the base getter candidate
returning `object+0x58`. The new in-image caller finding does not identify an
ID-0 concrete class or writer. It does not show that the ID-0 object is
created by another module, supplied by the job, or assigned a default. Keep
all of those questions `UNBOUND`.

## MVP disposition

Preserve the typed selector as an explicit runtime input and retain the
existing zero/nonzero branch. Do not hardcode a default or add a user prompt
from this static evidence. Conditional MVP dispatch is not blocked; exact
per-run branch selection and audio equivalence remain open.

## Scope and exclusions

Ghidra 12.2 static analysis only. No DEE run, media/payload read, private key
or license read, or implementation contact occurred in this slice. This Wash
supplement contains no binary, disassembly, decompilation, PCM, or encoded
payload.
