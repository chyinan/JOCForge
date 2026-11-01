# DDPi stream `+0xA8` to codec-state boundary

## Static field handoff

In the hash-identified DDP encoder, internal setparams key `0x15` copies a 32-bit value to per-stream state `+0xA8`. The inspected type table (entries 0–13) has no descriptor row for ID `0x15`, so its semantic API name remains `UNBOUND`.

`FUN_140350D00` passes the per-stream object to `FUN_140317600`, which calls `FUN_1403471F0` with source base `stream+0x0C`. That helper's relative `+0x9C` read is therefore `stream+0xA8`. It copies the value to the child codec-state `+0x94`, except source `-1` is replaced with `0x33`.

## Object-identity distinction

`FUN_140318610` checks a different per-stream object field at `+0x94` and reports an Audio Coding Mode error for invalid values. The shared numeric displacement does not establish that this validator sees the child codec-state pointer written by `FUN_1403471F0`. The child state's semantic type remains `UNBOUND`.

## Writer boundary

The connected route reaches the DDP stuffer, but no direct read of the copied child-state field was tied to the standard E-AC-3 header U5 or any BSI bit. The last direct consumer of source `stream+0xA8` is the copy/default helper; `FUN_140317600` remains the next function for the unresolved writer edge.

## Disposition

Reference only; `implementation_ready=false`. Keep ID `0x15` semantic name, codec-state `+0x94` meaning, standard header field/bit offset, and any U5 mapping `UNBOUND`. These vectors cover only the static copy/default operation, not encoded output. No DEE run or payload/key access occurred.
